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
use axum::{http::Uri, middleware, Router};

use crate::{app::AppState, guards, utils::exception::HttpException};

pub mod post;
pub mod upload;
pub mod user;

pub fn build() -> Router<AppState> {
    let api_router = Router::new()
        .merge(user::protected_route())
        .merge(post::protected_route())
        .route_layer(middleware::from_fn(guards::cookie_guard::cookie_guard))
        .merge(upload::protected_route())
        .merge(user::public_route());

    Router::new().nest("/api/v1", api_router)
}

pub async fn fallback(uri: Uri) -> HttpException {
    HttpException::NotFoundException(Some(format!("No route for {uri}")))
}
