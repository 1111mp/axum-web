use sea_orm::DatabaseConnection;

pub mod post;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}
