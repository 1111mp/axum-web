use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

// usage: https://docs.rs/axum/latest/axum/middleware/fn.from_fn_with_state.html
// pub async fn jwt_guard(
//     State(state): State<AppState>,
//     mut req: Request,
//     next: Next,
// ) -> Result<Response, HttpException> {
//     let headers = req.headers();
//     // userid
//     let user_id = req
//         .headers()
//         .get("userid")
//         .and_then(|id| id.to_str().ok())
//         .ok_or_else(|| HttpException::UnauthorizedException(None))?;
//     // authorization
//     let token_key = headers
//         .get(header::AUTHORIZATION)
//         .and_then(|token_key| token_key.to_str().ok())
//         .ok_or_else(|| HttpException::UnauthorizedException(None))?;
//     let key = format!("{}_{user_id}", APP_AUTH_KEY.as_str());
//     let mut connect = state.redis_pool.get().await?;
//     let token_value: String = connect.hget(&key, token_key).await?;
//     let claims =
//         jwt_decode(&token_value).map_err(|_| HttpException::UnauthorizedException(None))?;
//     // Refresh token expiration (1 hour)
//     let _: () = connect.expire(&key, 60 * 60).await?;
//     // Insert current user into request extensions
//     req.extensions_mut().insert(claims);
//     // Continue to the next handler
//     let response: Response = next.run(req).await;
//     Ok(response)
// }

pub struct JWTGuard;

impl<S> FromRequestParts<S> for JWTGuard
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Unauthorized"))?;

        let claims = super::jwt_decode(bearer.token()).map_err(|err| {
            tracing::error!(%err);
            (StatusCode::UNAUTHORIZED, "Unauthorized")
        })?;

        parts.extensions.insert(claims);
        Ok(Self)
    }
}
