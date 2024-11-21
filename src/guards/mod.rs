use std::sync::LazyLock;

pub mod cookie_guard;
pub mod jwt_guard;

pub static APP_AUTH_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("APP_AUTH_KEY").expect("APP_AUTH_KEY must be set"));
