use std::env;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;

use crate::{
    routes::AppState,
    utils::{exception::KnownError, http_resp::JsonResponse},
};

use super::current_user::authorize_current_user;

// usage: https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html
pub async fn auth_guard(
    State(mut state): State<AppState>,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut req: Request,
    next: Next,
) -> Result<Response, KnownError> {
    let headers = req.headers();
    // userid
    let user_id = req.headers().get("userid").and_then(|id| id.to_str().ok());
    let user_id = if let Some(user_id) = user_id {
        user_id
    } else {
        return Ok(JsonResponse::<()>::Unauthorized {
            message: "Unauthorized".to_string(),
        }
        .into_response());
    };
    // authorization
    let token_key = headers
        .get(header::AUTHORIZATION)
        .and_then(|token_key| token_key.to_str().ok());
    let token_key = if let Some(token_key) = token_key {
        token_key
    } else {
        return Ok(JsonResponse::<()>::Unauthorized {
            message: "Unauthorized".to_string(),
        }
        .into_response());
    };

    let app_auth_key = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let key = format!("{app_auth_key}_{user_id}");
    let token_value: String = state.redis.hget(&key, token_key).await?;

    if let Ok(current_user) = authorize_current_user(&token_value).await {
        // refresh token expiration time. (1 hour)
        state.redis.expire(&key, 60 * 60).await?;
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Ok(JsonResponse::<()>::Unauthorized {
            message: "Unauthorized".to_string(),
        }
        .into_response())
    }
}
