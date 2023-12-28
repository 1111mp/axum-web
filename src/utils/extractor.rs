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
// We define our own `JsonParser` extractor that customizes the error from `axum::Json`
#[derive(Debug)]
pub struct JsonParser<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for JsonParser<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ParserError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state).await?;

        value.validate()?;
        Ok(JsonParser(value))
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/customize-path-rejection/src/main.rs
// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
#[derive(Debug)]
pub struct PathParser<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for PathParser<T>
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
        Ok(PathParser(value))
    }
}

// We define our own `Query` extractor that customizes the error from `axum::extract::Query`
#[derive(Debug)]
pub struct QueryParser<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for QueryParser<T>
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
        Ok(QueryParser(value))
    }
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    AxumPathRejection(#[from] PathRejection),

    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
}

impl IntoResponse for ParserError {
    fn into_response(self) -> Response {
        match self {
            ParserError::ValidationError(_) => {
                let status = StatusCode::BAD_REQUEST;
                (
                    status,
                    axum::Json(json!({
                        "code": status.as_u16(),
                        "message": self.to_string()
                    })),
                )
            }
            ParserError::AxumJsonRejection(rejection) => {
                let status = rejection.status();
                let payload = json!({
                    "code": status.as_u16(),
                    "message": rejection.body_text(),
                });

                (status, axum::Json(payload))
            }
            ParserError::AxumPathRejection(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
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
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                PathError {
                                    code: status.as_u16(),
                                    message: kind.to_string(),
                                    location: None,
                                }
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

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => {
                        let status = StatusCode::INTERNAL_SERVER_ERROR;
                        (
                            status,
                            PathError {
                                code: status.as_u16(),
                                message: error.body_text(),
                                location: None,
                            },
                        )
                    }
                    _ => {
                        let status = StatusCode::INTERNAL_SERVER_ERROR;
                        (
                            status,
                            PathError {
                                code: status.as_u16(),
                                message: format!("Unhandled path rejection: {rejection}"),
                                location: None,
                            },
                        )
                    }
                };

                (status, axum::Json(json!(body)))
            }
            ParserError::AxumQueryRejection(rejection) => {
                let status = rejection.status();
                let payload = json!({
                    "code": status.as_u16(),
                    "message": rejection.body_text(),
                });

                (status, axum::Json(payload))
            }
        }
        .into_response()
    }
}

#[derive(Serialize)]
pub struct PathError {
    code: u16,
    message: String,
    location: Option<String>,
}
