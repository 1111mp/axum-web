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
    routing::post,
    Router,
};
use axum_macros::debug_handler;
use bcrypt::verify;
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

use crate::utils::{
    exception::KnownError,
    extractor::{JsonParser, QueryParser},
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
        .route("/signup", post(user_signup))
        .route("/signin", post(user_signin))
}

fn make_protected_api() -> Router<AppState> {
    Router::new().route("/signout", post(user_signout))
}

#[debug_handler]
async fn user_signup(
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

#[debug_handler]
async fn user_signin(
    State(state): State<AppState>,
    cookies: Cookies,
    JsonParser(input): JsonParser<LoginUser>,
) -> Result<Response, KnownError> {
    let model = User::find()
        .filter(user::Column::Email.eq(&input.email))
        .one(&state.db)
        .await?;

    let user = if let Some(user) = model {
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

#[debug_handler]
async fn user_signout(
    cookies: Cookies,
    QueryParser(input): QueryParser<RedirectParam>,
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

#[derive(Debug, Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, message = "Invalid name"))]
    name: String,
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate)]
struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate)]
struct RedirectParam {
    uri: Option<String>,
}
