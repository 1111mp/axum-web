use crate::singleton;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::env;

#[derive(Debug)]
pub struct Config {
    // server
    server_host: String,
    server_port: u16,

    // database
    database_url: String,

    // redis
    redis_url: String,

    // auth
    app_auth_key: String,
    jwt_secret: String,
    jwt_keys: Keys,

    // log
    log_dir: String,
    log_level: String,
}

singleton!(Config, CONFIG);

impl Config {
    fn new() -> Self {
        Self::from_env()
    }

    fn must_get(key: &str) -> String {
        env::var(key)
            .unwrap_or_else(|_| panic!("❌ Missing required environment variable: {}", key))
    }

    fn must_get_parsed<T, F>(key: &str, parse: F) -> T
    where
        F: FnOnce(String) -> Option<T>,
    {
        let raw = Self::must_get(key);
        parse(raw).unwrap_or_else(|| panic!("❌ Invalid format for environment variable: {}", key))
    }

    fn from_env() -> Self {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(3000);

        let database_url = Self::must_get("DATABASE_URL");
        let redis_url = Self::must_get("REDIS_URL");

        let app_auth_key =
            env::var("APP_AUTH_KEY").unwrap_or_else(|_| "default_auth_key".to_string());
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret".to_string());
        let jwt_keys = Keys::new(jwt_secret.as_bytes());

        let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Self {
            server_host,
            server_port,
            database_url,
            redis_url,
            app_auth_key,
            jwt_secret,
            jwt_keys,
            log_dir,
            log_level,
        }
    }

    // getters
    pub fn server_host(&self) -> &str {
        &self.server_host
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    pub fn app_auth_key(&self) -> &str {
        &self.app_auth_key
    }

    pub fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub fn jwt_keys(&self) -> &Keys {
        &self.jwt_keys
    }

    pub fn log_dir(&self) -> &str {
        &self.log_dir
    }

    pub fn log_level(&self) -> &str {
        &self.log_level
    }
}

#[derive(Debug)]
pub struct Keys {
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

    pub fn encoding(&self) -> &EncodingKey {
        &self.encoding
    }

    pub fn decoding(&self) -> &DecodingKey {
        &self.decoding
    }
}
