use crate::{logger, routes, swagger::ApiDoc};

use axum::{
    http::{header, HeaderName, Method, Request},
    Router,
};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::{env, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{info, info_span};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

const REQUEST_ID_HEADER: &str = "x-request-id";

#[dotenvy::load]
#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    let _guard = logger::logger_init();

    let host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let db_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let redis_url = env::var("REDIS_URL").map_err(|_| anyhow::anyhow!("REDIS_URL must be set"))?;

    // database
    let db = Database::connect(db_url).await?;
    // Apply all pending migrations
    Migrator::up(&db, None).await?;
    // Drop all tables from the database, then reapply all migrations
    // Migrator::fresh(&db).await?;

    info!("Successfully connected to the database");

    // redis
    let manager = RedisConnectionManager::new(redis_url)?;
    let redis_pool = bb8::Pool::builder().build(manager).await?;

    info!("Successfully connected to the redis");

    info!("Server running...");

    let state = Arc::new(AppState { db, redis_pool });

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let middleware = ServiceBuilder::new()
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
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
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
                    x_request_id,
                ])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::HEAD,
                    Method::OPTIONS,
                ])
                .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
                    origin.as_bytes().ends_with(b".domain.net")
                }))
                .max_age(Duration::from_secs(3600)),
        );
    // build our application with a single route
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", routes::router())
        .layer(middleware)
        .split_for_parts();

    let app = router
        // swagger ui
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", api))
        .fallback(routes::fallback)
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis_pool: Pool<RedisConnectionManager>,
}
