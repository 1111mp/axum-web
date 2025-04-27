/**
 *  https://docs.rs/axum/latest/axum/middleware/index.html#ordering
 *  The public router and protected router depend on the execution order of middleware
 *
 * 	Router::new()
 *  .merge(routes::user::create_protected_route())
 *  .route_layer(middleware::from_fn(middlewares::cookie_auth::cookie_guard))
 *  .merge(routes::user::create_public_route()),
 *
 *              requests
 *                 |
 *                 v
 *   +-------- public_route -------+
 *   | +------ cookie_auth ------+ |
 *   | | +-- protected_route --+ | |
 *   | | |                     | | |
 *   | | |       handler       | | |
 *   | | |                     | | |
 *   | | +-- protected_route --+ | |
 *   | +------ cookie_auth ------+ |
 *   +-------- public_route -------+
 *                 |
 *                 v
 *             responses
 */
use crate::{
    app::AppState,
    exception::HttpException,
    guards::{self, CookieGuard},
};

use std::sync::Arc;

use axum::{
    http::{StatusCode, Uri},
    middleware,
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

pub mod post;
pub mod upload;
pub mod user;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    let api_v1_router = OpenApiRouter::new()
        .merge(user::protected_route())
        .merge(post::protected_route())
        .route_layer(middleware::from_extractor::<CookieGuard>())
        .merge(upload::protected_route())
        .merge(user::public_route());

    OpenApiRouter::new().nest("/v1", api_v1_router)
}

pub async fn fallback(uri: Uri) -> HttpException {
    HttpException::NotFoundException(Some(format!("No route for {uri}")))
}

pub enum HttpResponse<T> {
    Json {
        payload: Option<T>,
        message: Option<String>,
    },

    RedirectTo {
        uri: String,
    },
}

impl<T: Serialize> IntoResponse for HttpResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpResponse::Json { payload, message } => {
                let status = StatusCode::OK;
                let body = JsonResponse {
                    status_code: status.as_u16(),
                    payload,
                    message,
                };

                (status, Json(body)).into_response()
            }
            HttpResponse::RedirectTo { uri } => Redirect::temporary(&uri).into_response(),
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct JsonResponse<T> {
    status_code: u16,
    payload: T,
    message: Option<String>,
}
