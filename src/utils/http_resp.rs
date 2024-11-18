use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub enum HttpResponse<T> {
    Json {
        data: Option<T>,
        message: Option<String>,
    },

    RedirectTo {
        uri: String,
    },
}

impl<T: Serialize> IntoResponse for HttpResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpResponse::Json { data, message } => {
                let body = json!({
                    "data": data,
                    "message": message,
                    "statusCode": StatusCode::OK.as_u16(),
                });

                Json(body).into_response()
            }
            HttpResponse::RedirectTo { uri } => Redirect::temporary(&uri).into_response(),
        }
    }
}
