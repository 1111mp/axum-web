use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use axum_macros::debug_handler;
use entity::{post, prelude::Post};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use validator::Validate;

use crate::{
    middlewares::current_user::CurrentUser,
    utils::{exception::KnownError, extractor::PathParser, http_resp::JsonResponse},
};

use super::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/v1/post", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all))
        .route("/:id", get(get_one))
}

#[derive(Debug, Deserialize, Validate)]
struct PostParam {
    #[validate(range(min = 1, message = "Invalid id"))]
    id: i32,
}

#[debug_handler]
async fn get_all(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response, KnownError> {
    let posts = Post::find()
        .filter(post::Column::UserId.eq(current_user.id))
        .all(&state.db)
        .await?;

    Ok(JsonResponse::OK {
        message: "successed".to_string(),
        data: Some(posts),
    }
    .into_response())
}

#[debug_handler]
async fn get_one(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PathParser(param): PathParser<PostParam>,
) -> Result<Response, KnownError> {
    println!("{:?}", current_user);

    let model = Post::find_by_id(param.id).one(&state.db).await?;

    match model {
        Some(post) => Ok(JsonResponse::OK {
            message: "success".to_string(),
            data: Some(post),
        }
        .into_response()),
        None => Ok(JsonResponse::<()>::NotFound {
            message: format!("No post found with id {}", &param.id),
        }
        .into_response()),
    }
}
