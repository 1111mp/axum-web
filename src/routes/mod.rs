use bb8::Pool;
use bb8_redis::RedisConnectionManager;

pub mod post;
pub mod upload;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis_pool: Pool<RedisConnectionManager>,
}
