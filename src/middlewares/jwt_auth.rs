use axum::{extract::Request, http::header, middleware::Next, response::Response};

use crate::utils::http_resp::JsonResponse;

use super::current_user::authorize_current_user;

pub async fn auth_guard(mut req: Request, next: Next) -> Result<Response, JsonResponse<()>> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|token| token.to_str().ok());

    let token = if let Some(token) = token {
        token
    } else {
        return Err(JsonResponse::Unauthorized {
            message: "Unauthorized".to_string(),
        });
    };

    if let Ok(current_user) = authorize_current_user(token).await {
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
