use crate::{app::AppState, exception::HttpException, guards::Claims, http_exception_or};

use std::sync::Arc;

use axum::extract::{Path, State};
use axum_macros::debug_handler;
use entity::{post, prelude::Post};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{HttpResponse, JsonResponse};

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
		(status = 200, description = "List all posts successfully", body = JsonResponse<Vec<PostSchema>>)
	),
	security(
    ("cookie_security" = [])
  ),
	tag = crate::api_doc::POST_TAG
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

/// Query Post items
///
/// Query Post details from database storage.
#[utoipa::path(
  get,
  path = "/{id}",
  responses(
    (status = 200, description = "Query Post details successfully", body = JsonResponse<PostSchema>),
		(status = 404, description = "Post not found")
  ),
  params(
    ("id" = i32, Path, description = "Post database id"),
  ),
  security(
    ("cookie_security" = [])
  ),
	tag = crate::api_doc::POST_TAG
)]
#[debug_handler]
async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<HttpResponse<post::Model>, HttpException> {
    let post = http_exception_or!(
        Post::find_by_id(id).one(&state.db).await?,
        NotFoundException,
        format!("No post found with id {}", id)
    );

    Ok(HttpResponse::Json {
        message: None,
        payload: Some(post),
    })
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PostSchema {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub text: String,
    #[schema(default = "Feed")]
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}
