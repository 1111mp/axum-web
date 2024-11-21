use crate::extensions::user_execution::UserInfo;

use anyhow::Result;
use chrono::{Duration, Utc};
use entity::user;
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iat: i64,
    pub exp: i64,
    pub user: UserInfo,
}

impl From<Claims> for UserInfo {
    fn from(claims: Claims) -> Self {
        claims.user
    }
}

pub fn jwt_encode(user: &user::Model) -> Result<String, Error> {
    let now = Utc::now();
    let expire = now + Duration::days(1);
    let claims = Claims {
        iat: now.timestamp(),
        exp: expire.timestamp(),
        user: UserInfo {
            id: user.id,
            name: user.name.clone(),
            email: user.email.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        },
    };

    encode(&Header::default(), &claims, &KEYS.encoding)
}

pub fn jwt_decode(token: &str) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(token, &KEYS.decoding, &Validation::default())?;
    Ok(token_data.claims)
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
