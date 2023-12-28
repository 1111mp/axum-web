use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub enum JsonResponse<T: Serialize> {
    OK { message: String, data: Option<T> },

    RedirectTo { uri: String },

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
            }
            .into_response(),

            JsonResponse::RedirectTo { uri } => Redirect::to(uri.as_str()).into_response(),

            JsonResponse::BadRequest { message } => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                  "code": 400,
                  "message": message
                })),
            )
                .into_response(),
            JsonResponse::Unauthorized { message } => (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                  "code": 401,
                  "message": message
                })),
            )
                .into_response(),
            JsonResponse::NotFound { message } => (
                StatusCode::NOT_FOUND,
                Json(json!({
                  "code": 404,
                  "message": message
                })),
            )
                .into_response(),

            JsonResponse::InternalServerError { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                  "code": 500,
                  "message": message
                })),
            )
                .into_response(),
        }
    }
}
