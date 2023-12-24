use std::{env, time::Duration};

use axum::{
    http::{header, Method},
    middleware, Router,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;

use crate::{
    middlewares,
    routes::{self, AppState},
};

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let server_url = format!("{host}:{port}");

    let db = Database::connect(db_url).await?;
    // Apply all pending migrations
    Migrator::up(&db, None).await?;
    // Drop all tables from the database, then reapply all migrations
    // Migrator::fresh(&db).await?;

    let state = AppState { db };

    // build our application with a single route
    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(routes::post::create_route())
                .route_layer(middleware::from_fn(middlewares::cookie_auth::cookie_guard))
                .merge(routes::user::create_route()),
        )
        .layer(
            ServiceBuilder::new()
                .layer(
                    // https://github.com/tower-rs/tower-http/issues/194
                    CorsLayer::new()
                        .allow_credentials(true)
                        .allow_headers([
                            header::ACCEPT,
                            header::ACCEPT_LANGUAGE,
                            header::AUTHORIZATION,
                            header::CONTENT_LANGUAGE,
                            header::CONTENT_TYPE,
                        ])
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::DELETE,
                            Method::HEAD,
                            Method::OPTIONS,
                        ])
                        .allow_origin([
                            "http://127.0.0.1:3000".parse().unwrap(),
                            "http://127.0.0.1:1212".parse().unwrap(),
                        ])
                        .max_age(Duration::from_secs(60 * 60)),
                )
                .layer(CookieManagerLayer::new()),
        )
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
