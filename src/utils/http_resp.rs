use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{DbErr, SqlErr};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub enum JsonResponse<T: Serialize> {
    OK { message: String, data: Option<T> },
    BadRequest { message: String },
    Unauthorized { message: String },
    NotFound { message: String },

    InternalServerError { message: String },
}

impl<T: Serialize> IntoResponse for JsonResponse<T> {
    fn into_response(self) -> Response {
        match self {
            JsonResponse::OK { message, data } => match data {
                Some(data) => (
                    StatusCode::OK,
                    Json(json!({
                      "code": 200,
                      "messsage": message,
                      "data": data
                    })),
                ),
                None => (
                    StatusCode::OK,
                    Json(json!({
                      "code": 200,
                      "messsage": message,
                    })),
                ),
            },
            JsonResponse::BadRequest { message } => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                  "code": 400,
                  "message": message
                })),
            ),
            JsonResponse::Unauthorized { message } => (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                  "code": 401,
                  "message": message
                })),
            ),
            JsonResponse::NotFound { message } => (
                StatusCode::NOT_FOUND,
                Json(json!({
                  "code": 404,
                  "message": message
                })),
            ),

            JsonResponse::InternalServerError { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                  "code": 500,
                  "message": message
                })),
            ),
        }
        .into_response()
    }
}

pub fn make_resp_from_db_err(err: &DbErr) -> Response {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(msg)) => {
            JsonResponse::<()>::InternalServerError { message: msg }
        }
        Some(sql_err) => JsonResponse::InternalServerError {
            message: sql_err.to_string(),
        },
        None => JsonResponse::InternalServerError {
            message: "Unknow Database Error".to_string(),
        },
    }
    .into_response()
}
