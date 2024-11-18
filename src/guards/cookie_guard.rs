use crate::utils::{exception::HttpException, jwt::jwt_decode};

use anyhow::Result;
use axum::{extract::Request, http::header, middleware::Next, response::Response};
use std::env;
use tower_cookies::Cookie;

pub async fn cookie_guard(mut req: Request, next: Next) -> Result<Response, HttpException> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok())
        .ok_or_else(|| HttpException::UnauthorizedException(None))?;
    let name = env::var("APP_AUTH_KEY").unwrap_or_else(|_| "app_auth_key".to_string());
    let token = get_cookie_value(cookies, &name)
        .ok_or_else(|| HttpException::UnauthorizedException(None))?;
    let claims = jwt_decode(&token).map_err(|_| HttpException::UnauthorizedException(None))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

/// get cookie value by name
fn get_cookie_value(cookies: &str, name: &str) -> Option<String> {
    Cookie::split_parse(cookies)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name() == name)
        .map(|cookie| cookie.value().to_string())
}