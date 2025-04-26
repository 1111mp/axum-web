// https://github.com/tokio-rs/axum/blob/main/examples/customize-extractor-error/src/custom_extractor.rs
// We define our own `Body` extractor that customizes the error from `axum::Json`

use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct Body<T>(pub T);

impl<S, T> FromRequest<S> for Body<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = super::ParserError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(Body(value))
    }
}
