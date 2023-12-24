use axum::{
    extract::{rejection::PathRejection, Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use axum_macros::debug_handler;
use entity::prelude::Post;
use sea_orm::EntityTrait;

use crate::{
    middlewares::current_user::CurrentUser,
    utils::http_resp::{make_resp_from_db_err, JsonResponse},
};

use super::AppState;

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/post", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new().route("/:id", get(get_one))
}

#[debug_handler]
async fn get_one(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    payload: Result<Path<i32>, PathRejection>,
) -> Response {
    let Path(id) = match payload {
        Ok(id) => id,
        Err(err) => {
            return JsonResponse::<()>::BadRequest {
                message: err.to_string(),
            }
            .into_response();
        }
    };

    println!("{:?}", current_user);

    let ret = Post::find_by_id(id).one(&state.db).await;

    match ret {
        Ok(opt) => match opt {
            Some(post) => JsonResponse::OK {
                message: "success".to_string(),
                data: Some(post),
            }
            .into_response(),
            None => JsonResponse::<()>::NotFound {
                message: format!("No post found with id {}", &id),
            }
            .into_response(),
        },
        Err(err) => make_resp_from_db_err(&err),
    }
}
