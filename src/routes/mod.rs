use sea_orm::DatabaseConnection;

pub mod post;
pub mod user;

pub mod make_resp;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}
