use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use axum_macros::debug_handler;
use entity::prelude::Post;
use sea_orm::EntityTrait;
use serde::Deserialize;
use validator::Validate;

use crate::{
    middlewares::current_user::CurrentUser,
    routes::extractor::PathParser,
    utils::http_resp::{make_resp_from_db_err, JsonResponse},
};

use super::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/post", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new().route("/:id", get(get_one))
}

#[derive(Debug, Deserialize, Validate)]
struct PostParam {
    #[validate(range(min = 1, message = "Invalid id"))]
    id: i32,
}

#[debug_handler]
async fn get_one(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathParser(param): PathParser<PostParam>,
) -> Response {
    println!("{:?}", current_user);

    let ret = Post::find_by_id(param.id).one(&state.db).await;

    match ret {
        Ok(opt) => match opt {
            Some(post) => JsonResponse::OK {
                message: "success".to_string(),
                data: Some(post),
            }
            .into_response(),
            None => JsonResponse::<()>::NotFound {
                message: format!("No post found with id {}", &param.id),
            }
            .into_response(),
        },
        Err(err) => make_resp_from_db_err(&err),
    }
}
