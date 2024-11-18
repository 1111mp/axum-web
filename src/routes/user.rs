/**
 * ! https://docs.rs/axum/latest/axum/middleware/index.html#ordering
 * ! The public router and protected router depend on the execution order of middleware
 *
 * ! .merge(routes::user::create_protected_route())
 * ! .route_layer(middleware::from_fn(middlewares::cookie_auth::cookie_guard))
 * ! .merge(routes::user::create_public_route()),
 *
 * !             requests
 * !                |
 * !                v
 * !  +-------- public_route -------+
 * !  | +------ cookie_auth ------+ |
 * !  | | +-- protected_route --+ | |
 * !  | | |                     | | |
 * !  | | |       handler       | | |
 * !  | | |                     | | |
 * !  | | +-- protected_route --+ | |
 * !  | +------ cookie_auth ------+ |
 * !  +-------- public_route -------+
 * !                |
 * !                v
 * !            responses
 */
use std::env;

use axum::{extract::State, routing, Router};
use axum_macros::debug_handler;
use entity::{post, prelude::Post, prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    http_exception, http_exception_or,
    utils::{
        exception::HttpException,
        extractor::{Body, Param, Query},
        http_resp::HttpResponse,
        jwt::jwt_encode,
        schema::RespError,
    },
};

use super::AppState;

pub fn create_public_route() -> Router<AppState> {
    Router::new().nest("/v1/user", make_public_api())
}

pub fn create_protected_route() -> Router<AppState> {
    Router::new().nest("/v1/user", make_protected_api())
}

fn make_public_api() -> Router<AppState> {
    Router::new()
        .route("/", routing::post(create_one))
        .route("/login", routing::post(user_login))
}

fn make_protected_api() -> Router<AppState> {
    Router::new()
        .route("/:id", routing::delete(delete_one))
        .route("/signout", routing::post(user_signout))
}

/// Create new User
///
/// Tries to create a new User or fails with 409 conflict if already exists.
#[utoipa::path(
    post,
    path = "/api/v1/user",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created successfully", body = RespForUser),
        (status = 409, description = "User already exists", body = RespError),
    )
)]
#[debug_handler]
pub(crate) async fn create_one(
    State(state): State<AppState>,
    cookies: Cookies,
    Body(input): Body<CreateUser>,
) -> Result<HttpResponse<user::Model>, HttpException> {
    let user = user::ActiveModel {
        name: Set(input.name),
        email: Set(input.email),
        password: Set(input.password),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let token = jwt_encode(&user).map_err(|_| HttpException::UnauthorizedException(None))?;
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let mut cookie = Cookie::new(name, token);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(HttpResponse::Json {
        message: None,
        data: Some(user),
    })
}

/// User Login
///
/// If successful, identity credentials are returned
#[utoipa::path(
    post,
    path = "/api/v1/user/login",
    request_body = LoginUser,
    responses(
        (status = 200, description = "User created successfully", headers(("Set-Cookie" = String, description = "identity credentials")), body = RespForUser),
        (status = 400, description = "User not found", body = RespError),
    )
)]
#[debug_handler]
pub(crate) async fn user_login(
    State(state): State<AppState>,
    cookies: Cookies,
    Body(input): Body<LoginUser>,
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

    let token = jwt_encode(&user).map_err(|_| HttpException::UnauthorizedException(None))?;
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let mut cookie = Cookie::new(name, token);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(HttpResponse::Json {
        message: None,
        data: Some(user),
    })
}

/// User Logout
///
/// User logout
#[utoipa::path(
    post,
    path = "/api/v1/user/signout",
    request_body = Option<RedirectParam>,
    responses(
        (status = 200, description = "User logout successfully", body = RespError),
        (status = 401, description = "Unauthorized to logout", body = RespError),
    ),
    security(
        ("app_auth_key" = [])
    )
)]
#[debug_handler]
async fn user_signout(
    cookies: Cookies,
    Body(input): Body<RedirectParam>,
) -> Result<HttpResponse<()>, HttpException> {
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let cookie = Cookie::from(name);
    cookies.remove(cookie);

    let uri = input.uri.unwrap_or("/login".to_string());
    Ok(HttpResponse::RedirectTo { uri })
}

/// Delete User by id
///
/// Delete User by id. Returns either 200 success of 404 with RespError if User is not found.
#[utoipa::path(
        delete,
        path = "/api/v1/user/{id}",
        responses(
            (status = 200, description = "User delete done successfully", body = RespError),
            (status = 401, description = "Unauthorized to delete User", body = RespError),
            (status = 404, description = "User not found", body = RespError)
        ),
        params(
            ("id" = i32, Path, description = "User database id"),
            ("thoroughly" = Option<bool>, Query, description = "Whether to completely delete all user related information, default value is false")
        ),
        security(
            ("app_auth_key" = [])
        )
    )]
#[debug_handler]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    cookies: Cookies,
    Param(input): Param<DeleteUser>,
    Query(opt): Query<DeleteUserOpt>,
) -> Result<HttpResponse<()>, HttpException> {
    let thoroughly = if let Some(thoroughly) = opt.thoroughly {
        thoroughly
    } else {
        // default value is false
        false
    };

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

    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let cookie = Cookie::from(name);
    cookies.remove(cookie);

    Ok(HttpResponse::Json {
        message: Some(format!(
            "The user {} has been successfully deleted",
            input.id
        )),
        data: None,
    })
}

/// Item create user.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct CreateUser {
    #[validate(length(min = 1, message = "Invalid name"))]
    name: String,
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct DeleteUser {
    #[validate(range(min = 1, message = "Invalid id"))]
    id: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct DeleteUserOpt {
    thoroughly: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct RedirectParam {
    uri: Option<String>,
}

/**
 * ! schema for swagger
 */
#[derive(ToSchema)]
pub(crate) struct RespForUser {
    pub code: i32,
    pub message: String,
    pub data: UserInfo,
}

#[derive(ToSchema)]
pub(crate) struct UserInfo {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub token: String,
    pub create_at: String,
    pub update_at: String,
}
