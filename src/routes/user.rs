use super::{HttpResponse, JsonResponse};
use crate::{
    core::{config, exception::HttpException, state},
    dtos::user_dtos::{CreateUserDto, DeleteUserDto, DeleteUserParam, LoginUserDto, RedirectParam},
    extractors::{Body, Param, Query},
    guards::jwt_encode,
    http_exception, http_exception_or,
};
use axum::extract::State;
use axum_macros::debug_handler;
use entity::{post, prelude::Post, prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn public_route() -> OpenApiRouter<Arc<state::AppState>> {
    let router = OpenApiRouter::new()
        .routes(routes!(create_one))
        .routes(routes!(login));

    OpenApiRouter::new().nest("/user", router)
}

pub fn protected_route() -> OpenApiRouter<Arc<state::AppState>> {
    let router = OpenApiRouter::new().routes(routes!(delete_one, signout));

    OpenApiRouter::new().nest("/user", router)
}

/// Create new User
///
/// Tries to create a new User or fails with 409 conflict if already exists.
#[utoipa::path(
  post,
  path = "",
  request_body = CreateUserDto,
  responses(
    (status = 200, description = "User created successfully", body = JsonResponse<UserSchema>),
    (status = 409, description = "User already exists"),
  ),
  tag = crate::api_doc::USER_TAG
)]
#[debug_handler]
pub(crate) async fn create_one(
    State(state): State<Arc<state::AppState>>,
    cookies: Cookies,
    Body(input): Body<CreateUserDto>,
) -> Result<HttpResponse<user::Model>, HttpException> {
    let user = user::ActiveModel {
        name: Set(input.name),
        email: Set(input.email),
        password: Set(input.password),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let config = config::Config::global();
    let token = jwt_encode(user.id, config.jwt_keys().encoding())
        .map_err(|_| HttpException::UnauthorizedException(None))?;
    let cookie = Cookie::build((config.app_auth_key(), token))
        .secure(true)
        .http_only(true)
        .build();
    cookies.add(cookie);

    Ok(HttpResponse::Json {
        message: None,
        payload: Some(user),
    })
}

/// User Login
///
/// If successful, identity credentials are returned
#[utoipa::path(
  post,
  path = "/login",
  request_body = LoginUserDto,
  responses(
    (status = 200, description = "User created successfully", headers(("Set-Cookie" = String, description = "identity credentials")), body = JsonResponse<UserSchema>),
    (status = 400, description = "User not found"),
  ),
  tag = crate::api_doc::USER_TAG
)]
#[debug_handler]
pub(crate) async fn login(
    State(state): State<Arc<state::AppState>>,
    cookies: Cookies,
    Body(input): Body<LoginUserDto>,
) -> Result<HttpResponse<user::Model>, HttpException> {
    let user = http_exception_or!(
        User::find()
            .filter(user::Column::Email.eq(&input.email))
            .one(&state.db)
            .await?,
        NotFoundException,
        format!("No user found with email {}", &input.email)
    );

    // verify password
    let is_valid_password = bcrypt::verify(&input.password, &user.password).unwrap_or(false);
    if !is_valid_password {
        http_exception!(UnauthorizedException, "Invalid email or password");
    }

    let config = config::Config::global();
    let token = jwt_encode(user.id, config.jwt_keys().encoding())
        .map_err(|_| HttpException::UnauthorizedException(None))?;
    let cookie = Cookie::build((config.app_auth_key(), token))
        .secure(true)
        .http_only(true)
        .build();
    cookies.add(cookie);

    Ok(HttpResponse::Json {
        message: None,
        payload: Some(user),
    })
}

/// User Logout
///
/// User logout
#[utoipa::path(
  post,
  path = "/signout",
  request_body = Option<RedirectParam>,
  responses(
    (status = 200, description = "User logout successfully"),
    (status = 401, description = "Unauthorized"),
  ),
  security(
    ("cookie_security" = [])
  ),
  tag = crate::api_doc::USER_TAG
)]
#[debug_handler]
async fn signout(
    cookies: Cookies,
    Body(input): Body<RedirectParam>,
) -> Result<HttpResponse<()>, HttpException> {
    let config = config::Config::global();
    cookies.remove(Cookie::from(config.app_auth_key()));

    let uri = input.uri.unwrap_or("/login".to_string());
    Ok(HttpResponse::RedirectTo { uri })
}

/// Delete User by id
///
/// Delete User by id. Returns either 200 success of 404 with RespError if User is not found.
#[utoipa::path(
	delete,
	path = "/{id}",
	responses(
		(status = 200, description = "User delete done successfully"),
		(status = 401, description = "Unauthorized to delete User"),
		(status = 404, description = "User not found")
		),
	params(
		("id" = i32, Path, description = "User database id"),
		("thoroughly" = Option<bool>, Query, description = "Whether to completely delete all user related information, default value is false")
	),
	security(
		("cookie_security" = [])
	),
  tag = crate::api_doc::USER_TAG
)]
#[debug_handler]
pub(crate) async fn delete_one(
    State(state): State<Arc<state::AppState>>,
    cookies: Cookies,
    Param(input): Param<DeleteUserParam>,
    Query(dto): Query<DeleteUserDto>,
) -> Result<HttpResponse<()>, HttpException> {
    let thoroughly = dto.thoroughly.unwrap_or(false);
    let txn = state.db.begin().await?;
    User::delete_by_id(input.id).exec(&txn).await?;
    if thoroughly {
        // All information under this user needs to be deleted
        // delete posts
        Post::delete_many()
            .filter(post::Column::UserId.eq(input.id))
            .exec(&txn)
            .await?;
    }
    txn.commit().await?;
    let config = config::Config::global();
    cookies.remove(Cookie::from(config.app_auth_key()));

    Ok(HttpResponse::Json {
        message: Some(format!(
            "The user {} has been successfully deleted",
            input.id
        )),
        payload: None,
    })
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct UserSchema {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub token: String,
    pub created_at: String,
    pub updated_at: String,
}
