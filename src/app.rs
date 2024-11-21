use crate::{routes, swagger::ApiDoc};

use axum::{
    http::{header, HeaderName, Method, Request},
    Router,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::{env, time::Duration};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info_span;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

const REQUEST_ID_HEADER: &str = "x-request-id";

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let redis_url = env::var("REDIS_URL").map_err(|_| anyhow::anyhow!("REDIS_URL must be set"))?;
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let allowed_origins = env::var("CORS_ALLOW_ORIGINS")
        .unwrap_or_else(|_| "http://127.0.0.1:3000,http://127.0.0.1:1212".to_string());

    // database
    let db = Database::connect(db_url).await?;
    // Apply all pending migrations
    Migrator::up(&db, None).await?;
    // Drop all tables from the database, then reapply all migrations
    // Migrator::fresh(&db).await?;

    // redis
    let manager = RedisConnectionManager::new(redis_url)?;
    let redis_pool = bb8::Pool::builder().build(manager).await?;

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);

    let state = AppState { db, redis_pool };
    // build our application with a single route
    let app = Router::new()
        .nest("/", routes::build())
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    x_request_id.clone(),
                    MakeRequestUuid,
                ))
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                        let request_id = request.headers().get(REQUEST_ID_HEADER);
                        match request_id {
                            Some(request_id) => info_span!(
                                "request",
                                request_id = ?request_id,
                                method = %request.method(),
                                uri = %request.uri(),
                            ),
                            None => info_span!(
                                "request",
                                method = %request.method(),
                                uri = %request.uri(),
                            ),
                        }
                    }),
                )
                .layer(PropagateRequestIdLayer::new(x_request_id)),
        )
        .layer(CookieManagerLayer::new())
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
                .allow_origin(
                    allowed_origins
                        .split(',')
                        .filter_map(|origin| origin.parse().ok())
                        .collect::<Vec<_>>(),
                )
                .max_age(Duration::from_secs(3600)),
        )
        // swagger ui
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
        // .fallback(routes::fallback)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let addr = format!("{host}:{port}").parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis_pool: Pool<RedisConnectionManager>,
}
