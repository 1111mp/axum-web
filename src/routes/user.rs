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

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing, Router,
};
use axum_macros::debug_handler;
use bcrypt::verify;
use entity::{post, prelude::Post, prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use utoipa::ToSchema;
use validator::Validate;

use crate::utils::{
    exception::KnownError,
    extractor::{JsonParser, PathParser, QueryParser},
    http_resp::JsonResponse,
    jwt::jwt_encode,
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
        .route("/signin", routing::post(user_signin))
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
    JsonParser(input): JsonParser<CreateUser>,
) -> Result<Response, KnownError> {
    let user = user::ActiveModel {
        name: Set(input.name),
        email: Set(input.email),
        password: Set(input.password),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let token = jwt_encode(&user)?;
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let mut cookie = Cookie::new(name, token);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(JsonResponse::OK {
        message: "success".to_string(),
        data: Some(user),
    }
    .into_response())
}

/// User Login
///
/// If successful, identity credentials are returned
#[utoipa::path(
    post,
    path = "/api/v1/user/signin",
    request_body = LoginUser,
    responses(
        (status = 200, description = "User created successfully", headers(("Set-Cookie" = String, description = "identity credentials")), body = RespForUser),
        (status = 400, description = "User not found", body = RespError),
    )
)]
#[debug_handler]
pub(crate) async fn user_signin(
    State(state): State<AppState>,
    cookies: Cookies,
    JsonParser(input): JsonParser<LoginUser>,
) -> Result<Response, KnownError> {
    let user = User::find()
        .filter(user::Column::Email.eq(&input.email))
        .one(&state.db)
        .await?;

    let user = if let Some(user) = user {
        user
    } else {
        return Ok(JsonResponse::<()>::NotFound {
            // error message is "Invalid email or password" maybe better
            message: format!("No user found with email {}", &input.email),
        }
        .into_response());
    };

    // verify password
    if !verify(&input.password, &user.password)? {
        return Ok(JsonResponse::<()>::Unauthorized {
            message: "Invalid email or password".to_string(),
        }
        .into_response());
    }

    let token = jwt_encode(&user)?;
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let mut cookie = Cookie::new(name, token);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(JsonResponse::OK {
        message: "success".to_string(),
        data: Some(user),
    }
    .into_response())
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
    JsonParser(input): JsonParser<RedirectParam>,
) -> Result<Response, KnownError> {
    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let cookie = Cookie::from(name);
    cookies.remove(cookie);

    let uri = if let Some(uri) = input.uri {
        uri
    } else {
        // default redirect uri
        "/login".to_string()
    };
    Ok(JsonResponse::<()>::RedirectTo { uri }.into_response())
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
    PathParser(input): PathParser<DeleteUser>,
    QueryParser(opt): QueryParser<DeleteUserOpt>,
) -> Result<Response, KnownError> {
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

    Ok(JsonResponse::<()>::OK {
        message: format!("The user {} has been successfully deleted", input.id),
        data: None,
    }
    .into_response())
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
