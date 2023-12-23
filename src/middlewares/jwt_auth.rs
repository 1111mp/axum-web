use std::env;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

use jsonwebtoken::{errors::Error, Algorithm, DecodingKey, Validation};

use crate::utils::jwt::Claims;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CurrentUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub create_at: Option<DateTimeUtc>,
    pub update_at: Option<DateTimeUtc>,
}

impl CurrentUser {
    pub fn new(claims: &Claims) -> Self {
        Self {
            id: claims.id,
            name: claims.name.clone(),
            email: claims.email.clone(),
            create_at: claims.create_at,
            update_at: claims.update_at,
        }
    }
}

pub async fn auth_guard(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = if let Some(token) = auth_header {
        token
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Ok(current_user) = authorize_current_user(token).await {
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize_current_user(token: &str) -> Result<CurrentUser, Error> {
    let secret = env::var("JWT_SECRET").unwrap_or("jwt_secret".to_string());
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    let user = CurrentUser::new(&claims.claims);

    Ok(user)
}
