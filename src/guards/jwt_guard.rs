// use super::current_user::authorize_current_user;
use crate::{
    routes::AppState,
    utils::{exception::HttpException, jwt::jwt_decode},
};

use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::env;

// usage: https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html
pub async fn jwt_guard(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, HttpException> {
    let headers = req.headers();
    // userid
    let user_id = req
        .headers()
        .get("userid")
        .and_then(|id| id.to_str().ok())
        .ok_or_else(|| HttpException::UnauthorizedException(None))?;
    // authorization
    let token_key = headers
        .get(header::AUTHORIZATION)
        .and_then(|token_key| token_key.to_str().ok())
        .ok_or_else(|| HttpException::UnauthorizedException(None))?;
    let app_auth_key = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let key = format!("{app_auth_key}_{user_id}");
    let mut connect = state.redis_pool.get().await?;
    let token_value: String = connect.hget(&key, token_key).await?;
    let claims =
        jwt_decode(&token_value).map_err(|_| HttpException::UnauthorizedException(None))?;
    // Refresh token expiration (1 hour)
    let _: () = connect.expire(&key, 60 * 60).await?;
    // Insert current user into request extensions
    req.extensions_mut().insert(claims);
    // Continue to the next handler
    let response: Response = next.run(req).await;
    Ok(response)
}
