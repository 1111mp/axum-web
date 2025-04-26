// https://github.com/tokio-rs/axum/blob/main/examples/customize-path-rejection/src/main.rs
// We define our own `Param` extractor that customizes the error from `axum::extract::Path`

use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct Param<T>(pub T);

impl<S, T> FromRequestParts<S> for Param<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = super::ParserError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request_parts(parts, state).await?;
        value.validate()?;
        Ok(Param(value))
    }
}
