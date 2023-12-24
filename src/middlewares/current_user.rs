use std::env;

use jsonwebtoken::{errors::Error, Algorithm, DecodingKey, Validation};
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

use crate::utils::jwt::Claims;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentUser {
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

pub async fn authorize_current_user(token: &str) -> Result<CurrentUser, Error> {
    let secret: String = env::var("JWT_SECRET").unwrap_or("jwt_secret".to_string());
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;

    let user = CurrentUser::new(&claims.claims);

    Ok(user)
}
