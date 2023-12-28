use std::env;

use chrono::{Duration, Utc};
use entity::user;
use jsonwebtoken::{errors::Error, EncodingKey, Header};
use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub create_at: Option<DateTimeUtc>,
    pub update_at: Option<DateTimeUtc>,
    pub iat: i64,
    pub exp: i64,
}

pub fn jwt_encode(user: &user::Model) -> Result<String, Error> {
    let now = Utc::now();
    let expire = now + Duration::days(1);
    let claims = Claims {
        id: user.id,
        name: user.name.clone(),
        email: user.email.clone(),
        create_at: user.create_at,
        update_at: user.update_at,
        iat: now.timestamp(),
        exp: expire.timestamp(),
    };
    let secret = env::var("JWT_SECRET").unwrap_or("jwt_secret".to_string());

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}
