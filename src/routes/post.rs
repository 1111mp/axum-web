use crate::{
    app::AppState,
    dtos::post_dtos::QueryPostDto,
    extensions::user_execution::UserInfo,
    http_exception_or,
    swagger::{post_schemas::PostSchema, ResponseSchema},
    utils::{exception::HttpException, extractor::Param, http_resp::HttpResponse},
};

use axum::{extract::State, routing::get, Extension, Router};
use axum_macros::debug_handler;
use entity::{post, prelude::Post};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub fn protected_route() -> Router<AppState> {
    let router = Router::new()
        .route("/", get(get_all))
        .route("/:id", get(get_one));

    Router::new().nest("/post", router)
}

/// List all Post items
///
/// List all Post items from database storage.
#[utoipa::path(
        get,
        path = "/api/v1/post",
        responses(
            (status = 200, description = "List all posts successfully", body = ResponseSchema<Vec<PostSchema>>)
        ),
        security(
            ("app_auth_key" = [])
        )
    )]
#[debug_handler]
async fn get_all(
    State(state): State<AppState>,
    Extension(user): Extension<UserInfo>,
) -> Result<HttpResponse<Vec<post::Model>>, HttpException> {
    let posts = Post::find()
        .filter(post::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?;

    Ok(HttpResponse::Json {
        message: None,
        data: Some(posts),
    })
}

/// Get Post items
///
/// Get Post details from database storage.
#[utoipa::path(
    get,
    path = "/api/v1/post/{id}",
    responses(
        (status = 200, description = "Get Post details successfully", body = PostSchema)
    ),
    params(
        ("id" = i32, Path, description = "Post database id"),
    ),
    security(
        ("app_auth_key" = [])
    )
)]
#[debug_handler]
async fn get_one(
    State(state): State<AppState>,
    Param(query): Param<QueryPostDto>,
) -> Result<HttpResponse<post::Model>, HttpException> {
    let post = http_exception_or!(
        Post::find_by_id(query.id).one(&state.db).await?,
        NotFoundException,
        format!("No post found with id {}", &query.id)
    );

    Ok(HttpResponse::Json {
        message: None,
        data: Some(post),
    })
}
