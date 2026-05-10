use bb8::Pool;
use bb8_redis::RedisConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis_pool: Pool<RedisConnectionManager>,
}
