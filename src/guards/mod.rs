use std::sync::LazyLock;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{
    decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

mod cookie_guard;
mod jwt_guard;

pub use cookie_guard::*;
pub use jwt_guard::*;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    #[serde(with = "jwt_numeric_date")]
    pub iat: OffsetDateTime,
    #[serde(with = "jwt_numeric_date")]
    pub exp: OffsetDateTime,
}

/// https://github.com/Keats/jsonwebtoken/blob/master/examples/custom_time.rs
impl Claims {
    /// If a token should always be equal to its representation after serializing and deserializing
    /// again, this function must be used for construction. `OffsetDateTime` contains a microsecond
    /// field but JWT timestamps are defined as UNIX timestamps (seconds). This function normalizes
    /// the timestamps.
    pub fn new(user_id: i32, iat: OffsetDateTime, exp: OffsetDateTime) -> Self {
        // normalize the timestamps by stripping of microseconds
        let iat = iat
            .date()
            .with_hms_milli(iat.hour(), iat.minute(), iat.second(), 0)
            .unwrap()
            .assume_utc();
        let exp = exp
            .date()
            .with_hms_milli(exp.hour(), exp.minute(), exp.second(), 0)
            .unwrap()
            .assume_utc();

        Self { user_id, iat, exp }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or((StatusCode::UNAUTHORIZED, "UnAuthorized"))?;

        Ok(claims.clone())
    }
}

pub fn jwt_encode(user_id: i32) -> anyhow::Result<String, Error> {
    let iat = OffsetDateTime::now_utc();
    let exp = iat + Duration::days(1);
    let claims = Claims::new(user_id, iat, exp);

    encode(&Header::default(), &claims, &KEYS.encoding)
}

pub fn jwt_decode(token: &str) -> anyhow::Result<Claims> {
    let decoded = decode::<Claims>(token, &KEYS.decoding, &Validation::default())?;
    Ok(decoded.claims)
}

mod jwt_numeric_date {
    //! Custom serialization of OffsetDateTime to conform with the JWT spec (RFC 7519 section 2, "Numeric Date")
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

    /// Serializes an OffsetDateTime to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.unix_timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        OffsetDateTime::from_unix_timestamp(i64::deserialize(deserializer)?)
            .map_err(|_| serde::de::Error::custom("invalid Unix timestamp value"))
    }
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
