use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct MyResponse<T: Serialize> {
    pub code: StatusCode,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> From<MyResponse<T>> for Response {
    fn from(inner: MyResponse<T>) -> Self {
        (
            inner.code,
            Json(json!({
                "code": inner.code.as_u16(),
                "message": inner.message,
                "data": inner.data
            })),
        )
            .into_response()
    }
}

impl<T: Serialize> IntoResponse for MyResponse<T> {
    fn into_response(self) -> Response {
        (
            self.code,
            Json(json!({
                "code": self.code.as_u16(),
                "message": self.message,
                "data": self.data
            })),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub enum JsonResponse<T: Serialize> {
    OK { message: String, data: Option<T> },

    RedirectTo { uri: String },

    BadRequest { message: String },
    Unauthorized { message: String },
    NotFound { message: String },

    InternalServerError { message: String },
}

impl<T: Serialize> From<JsonResponse<T>> for Response {
    fn from(json_response: JsonResponse<T>) -> Self {
        match json_response {
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
