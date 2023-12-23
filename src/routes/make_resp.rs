use axum::{http::StatusCode, Json};
use sea_orm::{DbErr, SqlErr};
use serde_json::{json, Value};

pub fn make_resp_from_db_err(err: &DbErr) -> (StatusCode, axum::Json<Value>) {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "message": msg
            })),
        ),
        Some(sql_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "message": sql_err.to_string()
            })),
        ),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": 500,
                "message": "Unknow Database Error"
            })),
        ),
    }
}
