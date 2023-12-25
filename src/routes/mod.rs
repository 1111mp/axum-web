use sea_orm::DatabaseConnection;

mod exception;
mod extractor;
pub mod post;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}
