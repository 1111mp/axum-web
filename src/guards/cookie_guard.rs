use std::sync::LazyLock;

use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_cookies::Cookies;
use tracing::info;

pub static APP_AUTH_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("APP_AUTH_KEY").expect("APP_AUTH_KEY must be set"));

pub struct CookieGuard;

impl<S> FromRequestParts<S> for CookieGuard
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await?;
        let cookie = cookies
            .get(APP_AUTH_KEY.as_str())
            .map(|c| c.value().to_string())
            .ok_or((StatusCode::UNAUTHORIZED, "UnAuthorized"))?;

        info!("cookie: {}", cookie);

        let claims = super::jwt_decode(&cookie).map_err(|err| {
            info!("err: {}", err.to_string());
            (StatusCode::UNAUTHORIZED, "UnAuthorized")
        })?;

        info!("claims: {:?}", claims);

        parts.extensions.insert(claims);
        Ok(Self)
    }
}
