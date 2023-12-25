//! convert errors into responses

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{DbErr, SqlErr};
use serde_json::json;

#[derive(Debug)]
pub enum CatchedError {
    CatchedDbError(DbErr),
    CatchedBcryptError(bcrypt::BcryptError),
    CatchedJwtError(jsonwebtoken::errors::Error),
}

/// This makes it possible to use `?` to automatically convert a `DbErr`
/// into an `CatchedError`.
impl From<DbErr> for CatchedError {
    fn from(inner: DbErr) -> Self {
        CatchedError::CatchedDbError(inner)
    }
}

/// This makes it possible to use `?` to automatically convert a `bcrypt::BcryptError`
/// into an `CatchedError`.
impl From<bcrypt::BcryptError> for CatchedError {
    fn from(inner: bcrypt::BcryptError) -> Self {
        CatchedError::CatchedBcryptError(inner)
    }
}

/// This makes it possible to use `?` to automatically convert a `jsonwebtoken::errors::Error`
/// into an `CatchedError`.
impl From<jsonwebtoken::errors::Error> for CatchedError {
    fn from(inner: jsonwebtoken::errors::Error) -> Self {
        CatchedError::CatchedJwtError(inner)
    }
}

impl IntoResponse for CatchedError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            CatchedError::CatchedDbError(db_err) => match db_err.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(message)) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, message)
                }
                Some(sql_err) => (StatusCode::INTERNAL_SERVER_ERROR, sql_err.to_string()),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unknow Database Error".to_string(),
                ),
            },
            CatchedError::CatchedBcryptError(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            CatchedError::CatchedJwtError(jwt_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, jwt_err.to_string())
            }
        };

        (
            status,
            Json(json!({
              "code": status.as_u16(),
              "message": message
            })),
        )
            .into_response()
    }
}
