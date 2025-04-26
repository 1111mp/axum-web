mod body_extractor;
mod param_extractor;
mod query_extractor;

pub use body_extractor::*;
pub use param_extractor::*;
pub use query_extractor::*;

use axum::{
    extract::{
        path::ErrorKind,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    ValidationErrors(#[from] ValidationErrors),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    PathRejection(#[from] PathRejection),

    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),
}

impl ParserError {
    fn into_json_response(
        status: StatusCode,
        message: String,
        location: Option<String>,
    ) -> Response {
        (
            status,
            Json(ResponseFromError {
                status_code: status.as_u16(),
                message,
                location,
            }),
        )
            .into_response()
    }
}

impl IntoResponse for ParserError {
    fn into_response(self) -> Response {
        match self {
            ParserError::ValidationErrors(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                Self::into_json_response(StatusCode::BAD_REQUEST, message, None)
            }
            ParserError::JsonRejection(rejection) => {
                Self::into_json_response(StatusCode::BAD_REQUEST, rejection.body_text(), None)
            }
            ParserError::PathRejection(rejection) => match rejection {
                PathRejection::FailedToDeserializePathParams(inner) => {
                    let mut status = StatusCode::BAD_REQUEST;

                    let kind = inner.into_kind();
                    match &kind {
                        ErrorKind::WrongNumberOfParameters { .. } => {
                            Self::into_json_response(status, kind.to_string(), None)
                        }

                        ErrorKind::ParseErrorAtKey { key, .. } => {
                            Self::into_json_response(status, kind.to_string(), Some(key.clone()))
                        }

                        ErrorKind::ParseErrorAtIndex { index, .. } => Self::into_json_response(
                            status,
                            kind.to_string(),
                            Some(index.to_string()),
                        ),

                        ErrorKind::ParseError { .. } => {
                            Self::into_json_response(status, kind.to_string(), None)
                        }

                        ErrorKind::InvalidUtf8InPathParam { key } => {
                            Self::into_json_response(status, kind.to_string(), Some(key.clone()))
                        }

                        ErrorKind::UnsupportedType { .. } => {
                            // this error is caused by the programmer using an unsupported type
                            // (such as nested maps) so respond with `500` instead
                            status = StatusCode::INTERNAL_SERVER_ERROR;
                            Self::into_json_response(status, kind.to_string(), None)
                        }

                        ErrorKind::Message(msg) => {
                            Self::into_json_response(status, msg.clone(), None)
                        }

                        _ => Self::into_json_response(
                            status,
                            format!("Unhandled deserialization error: {kind}"),
                            None,
                        ),
                    }
                }
                PathRejection::MissingPathParams(error) => {
                    Self::into_json_response(StatusCode::BAD_REQUEST, error.to_string(), None)
                }
                _ => Self::into_json_response(
                    StatusCode::BAD_REQUEST,
                    format!("Unhandled path rejection: {rejection}"),
                    None,
                ),
            },
            ParserError::QueryRejection(rejection) => match rejection {
                QueryRejection::FailedToDeserializeQueryString(inner) => {
                    Self::into_json_response(StatusCode::BAD_REQUEST, inner.body_text(), None)
                }
                _ => Self::into_json_response(
                    StatusCode::BAD_REQUEST,
                    format!("Unhandled path rejection: {rejection}"),
                    None,
                ),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseFromError {
    status_code: u16,
    message: String,
    location: Option<String>,
}
