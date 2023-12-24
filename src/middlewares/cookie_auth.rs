use std::env;

use axum::{extract::Request, http::header, middleware::Next, response::Response};
use tower_cookies::Cookie;

use crate::utils::http_resp::JsonResponse;

use super::current_user::authorize_current_user;

pub async fn cookie_guard(mut req: Request, next: Next) -> Result<Response, JsonResponse<()>> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok());

    let cookies = if let Some(cookies) = cookies {
        cookies
    } else {
        return Err(JsonResponse::Unauthorized {
            message: "Unauthorized".to_string(),
        });
    };

    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let token = if let Some(token) = get_cookie_value(cookies, name.as_str()) {
        token
    } else {
        return Err(JsonResponse::Unauthorized {
            message: "Unauthorized".to_string(),
        });
    };

    if let Ok(current_user) = authorize_current_user(token.as_str()).await {
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(JsonResponse::Unauthorized {
            message: "Unauthorized".to_string(),
        })
    }
}

// get cookie value by name
fn get_cookie_value<'a>(cookies: &'a str, name: &str) -> Option<String> {
    for cookie in Cookie::split_parse_encoded(cookies) {
        let cookie = cookie.unwrap();
        if cookie.name() == name {
            return Some(cookie.value().to_owned());
        }
    }

    None
}
