use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::{routes::AppState, utils::http_resp::JsonResponse};

use super::current_user::authorize_current_user;

// usage: https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html
pub async fn auth_guard(
    State(state): State<AppState>,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut req: Request,
    next: Next,
) -> Result<Response, JsonResponse<()>> {
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
