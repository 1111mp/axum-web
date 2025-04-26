use crate::{
    app::AppState,
    dtos::post_dtos::QueryPostDto,
    exception::HttpException,
    extractors::Param,
    guards::Claims,
    http_exception_or,
    swagger::{post_schemas::PostSchema, ResponseSchema},
};

use std::sync::Arc;

use axum::extract::State;
use axum_macros::debug_handler;
use entity::{post, prelude::Post};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::HttpResponse;

pub fn protected_route() -> OpenApiRouter<Arc<AppState>> {
    let router = OpenApiRouter::new()
        .routes(routes!(get_one))
        .routes(routes!(get_all));

    OpenApiRouter::new().nest("/post", router)
}

/// List all Post items
///
/// List all Post items from database storage.
#[utoipa::path(
        get,
        path = "",
        responses(
            (status = 200, description = "List all posts successfully", body = ResponseSchema<Vec<PostSchema>>)
        ),
        security(
            ("app_auth_key" = [])
        )
    )]
#[debug_handler]
async fn get_all(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<HttpResponse<Vec<post::Model>>, HttpException> {
    let posts = Post::find()
        .filter(post::Column::UserId.eq(claims.user_id))
        .all(&state.db)
        .await?;

    Ok(HttpResponse::Json {
        message: None,
        payload: Some(posts),
    })
}

/// Get Post items
///
/// Get Post details from database storage.
#[utoipa::path(
    get,
    path = "/{id}",
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
    State(state): State<Arc<AppState>>,
    Param(query): Param<QueryPostDto>,
) -> Result<HttpResponse<post::Model>, HttpException> {
    let post = http_exception_or!(
        Post::find_by_id(query.id).one(&state.db).await?,
        NotFoundException,
        format!("No post found with id {}", &query.id)
    );

    Ok(HttpResponse::Json {
        message: None,
        payload: Some(post),
    })
}
