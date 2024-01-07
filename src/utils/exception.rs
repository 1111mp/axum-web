//! convert errors into responses

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{DbErr, SqlErr};
use serde_json::json;

#[derive(Debug)]
pub enum KnownError {
    KnownDbError(DbErr),
    KnownBcryptError(bcrypt::BcryptError),
    KnownJwtError(jsonwebtoken::errors::Error),
    KnownRedisError(redis::RedisError),
}

/// This makes it possible to use `?` to automatically convert a `DbErr`
/// into an `KnownError`.
impl From<DbErr> for KnownError {
    fn from(inner: DbErr) -> Self {
        KnownError::KnownDbError(inner)
    }
}

/// This makes it possible to use `?` to automatically convert a `bcrypt::BcryptError`
/// into an `KnownError`.
impl From<bcrypt::BcryptError> for KnownError {
    fn from(inner: bcrypt::BcryptError) -> Self {
        KnownError::KnownBcryptError(inner)
    }
}

/// This makes it possible to use `?` to automatically convert a `jsonwebtoken::errors::Error`
/// into an `KnownError`.
impl From<jsonwebtoken::errors::Error> for KnownError {
    fn from(inner: jsonwebtoken::errors::Error) -> Self {
        KnownError::KnownJwtError(inner)
    }
}

/// This makes it possible to use `?` to automatically convert a `redis::RedisError`
/// into an `KnownError`.
impl From<redis::RedisError> for KnownError {
    fn from(inner: redis::RedisError) -> Self {
        KnownError::KnownRedisError(inner)
    }
}

impl IntoResponse for KnownError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            KnownError::KnownDbError(db_err) => match db_err.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(message)) => (StatusCode::CONFLICT, message),
                Some(sql_err) => (StatusCode::INTERNAL_SERVER_ERROR, sql_err.to_string()),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unknow Database Error".to_string(),
                ),
            },
            KnownError::KnownBcryptError(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password".to_string(),
            ),
            KnownError::KnownJwtError(jwt_err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, jwt_err.to_string())
            }
            KnownError::KnownRedisError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("[redis]: {}", error.to_string()),
            ),
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
