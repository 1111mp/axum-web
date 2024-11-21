//! convert errors into responses

use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::SqlErr;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpException {
    /// 400
    #[error("Bad Request")]
    BadRequestException(Option<String>),
    /// 401
    #[error("Unauthorized")]
    UnauthorizedException(Option<String>),
    /// 403
    #[error("Forbidden")]
    ForbiddenException(Option<String>),
    /// 404
    #[error("Not Found")]
    NotFoundException(Option<String>),
    /// 405
    #[error("Method Not Allowed")]
    MethodNotAllowedException(Option<String>),
    /// 406
    #[error("Not Acceptable")]
    NotAcceptableException(Option<String>),
    /// 408
    #[error("Request Timeout")]
    RequestTimeoutException(Option<String>),
    /// 409
    #[error("Conflict")]
    ConflictException(Option<String>),
    /// 410
    #[error("Gone")]
    GoneException(Option<String>),
    /// 412
    #[error("Precondition Failed")]
    PreconditionFailedException(Option<String>),
    /// 413
    #[error("Payload Too Large")]
    PayloadTooLargeException(Option<String>),
    /// 415
    #[error("Unsupported Media Type")]
    UnsupportedMediaTypeException(Option<String>),
    /// 418
    #[error("I'm a teapot")]
    ImATeapotException(Option<String>),
    /// 422
    #[error("Unprocessable Entity")]
    UnprocessableEntityException(Option<String>),
    /// 500
    #[error("Internal Server Error")]
    InternalServerErrorException(Option<String>),
    /// 501
    #[error("Not Implemented")]
    NotImplementedException(Option<String>),
    /// 502
    #[error("Bad Gateway")]
    BadGatewayException(Option<String>),
    /// 503
    #[error("Service Unavailable")]
    ServiceUnavailableException(Option<String>),
    /// 504
    #[error("Gateway Timeout")]
    GatewayTimeoutException(Option<String>),
    /// 505
    #[error("HTTP Version Not Supported")]
    HttpVersionNotSupportedException(Option<String>),

    /// Other Error
    #[error("{0}")]
    DbException(#[from] sea_orm::DbErr),
}

impl HttpException {
    fn status_and_default_message(&self) -> (StatusCode, String) {
        match self {
            HttpException::BadRequestException(_) => {
                (StatusCode::BAD_REQUEST, "Bad Request".to_string())
            }
            HttpException::UnauthorizedException(_) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            HttpException::ForbiddenException(_) => {
                (StatusCode::FORBIDDEN, "Forbidden".to_string())
            }
            HttpException::NotFoundException(_) => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            HttpException::MethodNotAllowedException(_) => (
                StatusCode::METHOD_NOT_ALLOWED,
                "Method Not Allowed".to_string(),
            ),
            HttpException::NotAcceptableException(_) => {
                (StatusCode::NOT_ACCEPTABLE, "Not Acceptable".to_string())
            }
            HttpException::RequestTimeoutException(_) => {
                (StatusCode::REQUEST_TIMEOUT, "Request Timeout".to_string())
            }
            HttpException::ConflictException(_) => (StatusCode::CONFLICT, "Conflict".to_string()),
            HttpException::GoneException(_) => (StatusCode::GONE, "Gone".to_string()),
            HttpException::PreconditionFailedException(_) => (
                StatusCode::PRECONDITION_FAILED,
                "Precondition Failed".to_string(),
            ),
            HttpException::PayloadTooLargeException(_) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload Too Large".to_string(),
            ),
            HttpException::UnsupportedMediaTypeException(_) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported Media Type".to_string(),
            ),
            HttpException::ImATeapotException(_) => {
                (StatusCode::IM_A_TEAPOT, "I'm a teapot".to_string())
            }
            HttpException::UnprocessableEntityException(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Unprocessable Entity".to_string(),
            ),
            HttpException::InternalServerErrorException(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            HttpException::NotImplementedException(_) => {
                (StatusCode::NOT_IMPLEMENTED, "Not Implemented".to_string())
            }
            HttpException::BadGatewayException(_) => {
                (StatusCode::BAD_GATEWAY, "Bad Gateway".to_string())
            }
            HttpException::ServiceUnavailableException(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable".to_string(),
            ),
            HttpException::GatewayTimeoutException(_) => {
                (StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout".to_string())
            }
            HttpException::HttpVersionNotSupportedException(_) => (
                StatusCode::HTTP_VERSION_NOT_SUPPORTED,
                "HTTP Version Not Supported".to_string(),
            ),
            HttpException::DbException(db_err) => match db_err.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(msg)) => (StatusCode::BAD_REQUEST, msg),
                Some(SqlErr::ForeignKeyConstraintViolation(msg)) => (StatusCode::BAD_REQUEST, msg),
                Some(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unknown Database Error".to_string(),
                ),
                None => (StatusCode::INTERNAL_SERVER_ERROR, db_err.to_string()),
            },
        }
    }
}

impl IntoResponse for HttpException {
    fn into_response(self) -> Response {
        let (status, default_message) = self.status_and_default_message();

        // Use custom message if any, otherwise use default message
        let message = match self {
            HttpException::BadRequestException(Some(msg))
            | HttpException::UnauthorizedException(Some(msg))
            | HttpException::ForbiddenException(Some(msg))
            | HttpException::NotFoundException(Some(msg))
            | HttpException::MethodNotAllowedException(Some(msg))
            | HttpException::NotAcceptableException(Some(msg))
            | HttpException::RequestTimeoutException(Some(msg))
            | HttpException::ConflictException(Some(msg))
            | HttpException::GoneException(Some(msg))
            | HttpException::PreconditionFailedException(Some(msg))
            | HttpException::PayloadTooLargeException(Some(msg))
            | HttpException::UnsupportedMediaTypeException(Some(msg))
            | HttpException::ImATeapotException(Some(msg))
            | HttpException::UnprocessableEntityException(Some(msg))
            | HttpException::InternalServerErrorException(Some(msg))
            | HttpException::NotImplementedException(Some(msg))
            | HttpException::BadGatewayException(Some(msg))
            | HttpException::ServiceUnavailableException(Some(msg))
            | HttpException::GatewayTimeoutException(Some(msg))
            | HttpException::HttpVersionNotSupportedException(Some(msg)) => msg,
            _ => default_message.to_string(),
        };

        let body = axum::Json(json!({
            "statusCode": status.as_u16(),
            "message": message,
        }));

        (status, body).into_response()
    }
}

impl From<io::Error> for HttpException {
    fn from(err: io::Error) -> Self {
        HttpException::InternalServerErrorException(Some(err.to_string()))
    }
}

impl From<redis::RedisError> for HttpException {
    fn from(err: redis::RedisError) -> Self {
        HttpException::InternalServerErrorException(Some(err.to_string()))
    }
}

impl From<bb8::RunError<redis::RedisError>> for HttpException {
    fn from(err: bb8::RunError<redis::RedisError>) -> Self {
        HttpException::InternalServerErrorException(Some(err.to_string()))
    }
}

#[macro_export]
macro_rules! http_exception {
    ($variant:ident, $msg:expr) => {
        return Err(HttpException::$variant(Some($msg.to_string())))
    };
}

#[macro_export]
macro_rules! http_exception_or {
    ($expr:expr, $variant:ident, $msg:expr) => {
        match $expr {
            Some(val) => val,
            None => return Err(HttpException::$variant(Some($msg.to_string()))),
        }
    };
    ($result:expr, $variant:ident) => {
        match $result {
            Ok(val) => val,
            Err(err) => return Err(HttpException::$variant(Some(err.to_string()))),
        }
    };
}
