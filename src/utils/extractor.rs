use axum::{
    async_trait,
    extract::{
        path::ErrorKind,
        rejection::{JsonRejection, PathRejection, QueryRejection},
        FromRequest, FromRequestParts, Request,
    },
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use thiserror::Error;
use validator::Validate;

// https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/custom_extractor.rs
// We define our own `Body` extractor that customizes the error from `axum::Json`
#[derive(Debug)]
pub struct Body<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Body<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ParserError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state).await?;

        value.validate()?;
        Ok(Body(value))
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/customize-path-rejection/src/main.rs
// We define our own `Param` extractor that customizes the error from `axum::extract::Path`
#[derive(Debug)]
pub struct Param<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Param<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = ParserError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(value) =
            axum::extract::Path::<T>::from_request_parts(parts, state).await?;

        value.validate()?;
        Ok(Param(value))
    }
}

// We define our own `Query` extractor that customizes the error from `axum::extract::Query`
#[derive(Debug)]
pub struct Query<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Query<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = ParserError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) =
            axum::extract::Query::<T>::from_request_parts(parts, state).await?;

        value.validate()?;
        Ok(Query(value))
    }
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    ValidationExtractorRejection(#[from] validator::ValidationErrors),

    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),

    #[error(transparent)]
    PathExtractorRejection(#[from] PathRejection),

    #[error(transparent)]
    QueryExtractorRejection(#[from] QueryRejection),
}

impl ParserError {
    fn to_json_response(
        status: StatusCode,
        message: impl ToString,
    ) -> (StatusCode, serde_json::Value) {
        (
            status,
            json!({
                "code": status.as_u16(),
                "message": message.to_string(),
            }),
        )
    }
}

impl IntoResponse for ParserError {
    fn into_response(self) -> Response {
        let (status, payload) = match self {
            ParserError::ValidationExtractorRejection(_) => {
                Self::to_json_response(StatusCode::BAD_REQUEST, self.to_string())
            }
            ParserError::JsonExtractorRejection(rejection) => {
                Self::to_json_response(rejection.status(), rejection.body_text())
            }
            ParserError::PathExtractorRejection(rejection) => match rejection {
                PathRejection::FailedToDeserializePathParams(inner) => {
                    let kind = inner.into_kind();
                    handle_path_error(StatusCode::BAD_REQUEST, kind)
                }
                PathRejection::MissingPathParams(error) => {
                    Self::to_json_response(StatusCode::INTERNAL_SERVER_ERROR, error.body_text())
                }
                _ => Self::to_json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled path rejection: {rejection}"),
                ),
            },
            ParserError::QueryExtractorRejection(rejection) => {
                Self::to_json_response(rejection.status(), rejection.body_text())
            }
        };

        (status, axum::Json(payload)).into_response()
    }
}

fn handle_path_error(status: StatusCode, kind: ErrorKind) -> (StatusCode, serde_json::Value) {
    let body = match &kind {
        ErrorKind::WrongNumberOfParameters { .. } => PathError {
            code: status.as_u16(),
            message: kind.to_string(),
            location: None,
        },
        ErrorKind::ParseErrorAtKey { key, .. } => PathError {
            code: status.as_u16(),
            message: kind.to_string(),
            location: Some(key.clone()),
        },
        ErrorKind::ParseErrorAtIndex { index, .. } => PathError {
            code: status.as_u16(),
            message: kind.to_string(),
            location: Some(index.to_string()),
        },
        ErrorKind::ParseError { .. } => PathError {
            code: status.as_u16(),
            message: kind.to_string(),
            location: None,
        },
        ErrorKind::InvalidUtf8InPathParam { key } => PathError {
            code: status.as_u16(),
            message: kind.to_string(),
            location: Some(key.clone()),
        },
        ErrorKind::UnsupportedType { .. } => {
            let status = StatusCode::INTERNAL_SERVER_ERROR;
            return (
                status,
                json!(PathError {
                    code: status.as_u16(),
                    message: kind.to_string(),
                    location: None,
                }),
            );
        }
        ErrorKind::Message(msg) => PathError {
            code: status.as_u16(),
            message: msg.clone(),
            location: None,
        },
        _ => PathError {
            code: status.as_u16(),
            message: format!("Unhandled deserialization error: {kind}"),
            location: None,
        },
    };

    (status, json!(body))
}

#[derive(Serialize)]
pub struct PathError {
    code: u16,
    message: String,
    location: Option<String>,
}
