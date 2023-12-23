use axum::{
    extract::{rejection::PathRejection, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;
use entity::prelude::Post;
use sea_orm::EntityTrait;
use serde_json::json;

use crate::routes::make_resp::make_resp_from_db_err;

use super::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/post", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new().route("/:id", get(get_one))
}

#[debug_handler]
async fn get_one(
    payload: Result<Path<i32>, PathRejection>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Path(id) = match payload {
        Ok(id) => id,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                  "code": 400,
                  "message": err.to_string()
                })),
            );
        }
    };

    let ret = Post::find_by_id(id).one(&state.db).await;

    match ret {
        Ok(opt) => match opt {
            Some(post) => (
                StatusCode::OK,
                Json(json!({
                  "code": 200,
                  "data": post
                })),
            ),
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({
                  "code": 404,
                  "message": format!("No post found with id {}", &id)
                })),
            ),
        },
        Err(err) => make_resp_from_db_err(&err),
    }
}
