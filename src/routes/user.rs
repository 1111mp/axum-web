use crate::{
    app::AppState,
    dtos::user_dtos::{CreateUserDto, DeleteUserDto, DeleteUserParam, LoginUserDto, RedirectParam},
    exception::HttpException,
    extractors::{Body, Param, Query},
    guards::{jwt_encode, APP_AUTH_KEY},
    http_exception, http_exception_or,
    swagger::{user_schemas::UserSchema, ErrorResponseSchema},
};

use std::sync::Arc;

use axum::extract::State;
use axum_macros::debug_handler;
use entity::{post, prelude::Post, prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tower_cookies::{Cookie, Cookies};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::HttpResponse;

pub fn public_route() -> OpenApiRouter<Arc<AppState>> {
    let router = OpenApiRouter::new()
        .routes(routes!(create_one))
        .routes(routes!(login));

    OpenApiRouter::new().nest("/user", router)
}

pub fn protected_route() -> OpenApiRouter<Arc<AppState>> {
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
        (status = 200, description = "User created successfully", body = UserSchema),
        (status = 409, description = "User already exists", body = ErrorResponseSchema),
    ),
    tag = "User"
)]
#[debug_handler]
pub(crate) async fn create_one(
    State(state): State<Arc<AppState>>,
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

    let token = jwt_encode(user.id).map_err(|_| HttpException::UnauthorizedException(None))?;
    let cookie = Cookie::build((APP_AUTH_KEY.as_str(), token))
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
        (status = 200, description = "User created successfully", headers(("Set-Cookie" = String, description = "identity credentials")), body = UserSchema),
        (status = 400, description = "User not found", body = ErrorResponseSchema),
    ),
    tag = "User"
)]
#[debug_handler]
pub(crate) async fn login(
    State(state): State<Arc<AppState>>,
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

    let token = jwt_encode(user.id).map_err(|_| HttpException::UnauthorizedException(None))?;
    let cookie = Cookie::build((APP_AUTH_KEY.as_str(), token))
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
        (status = 200, description = "User logout successfully", body = ErrorResponseSchema),
        (status = 401, description = "Unauthorized to logout", body = ErrorResponseSchema),
    ),
    security(
        ("app_auth_key" = [])
    ),
    tag = "User"
)]
#[debug_handler]
async fn signout(
    cookies: Cookies,
    Body(input): Body<RedirectParam>,
) -> Result<HttpResponse<()>, HttpException> {
    cookies.remove(Cookie::from(APP_AUTH_KEY.as_str()));

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
            (status = 200, description = "User delete done successfully", body = ErrorResponseSchema),
            (status = 401, description = "Unauthorized to delete User", body = ErrorResponseSchema),
            (status = 404, description = "User not found", body = ErrorResponseSchema)
        ),
        params(
            ("id" = i32, Path, description = "User database id"),
            ("thoroughly" = Option<bool>, Query, description = "Whether to completely delete all user related information, default value is false")
        ),
        security(
            ("app_auth_key" = [])
        ),
        tag = "User"
    )]
#[debug_handler]
pub(crate) async fn delete_one(
    State(state): State<Arc<AppState>>,
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
    cookies.remove(Cookie::from(APP_AUTH_KEY.as_str()));

    Ok(HttpResponse::Json {
        message: Some(format!(
            "The user {} has been successfully deleted",
            input.id
        )),
        payload: None,
    })
}
