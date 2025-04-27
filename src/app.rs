use crate::{api_doc::ApiDoc, events, logger, routes};

use axum::http::{header, HeaderName, Method, Request};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use socketioxide::{handler::ConnectHandler, SocketIo};
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
    // Exiting the context of `main` will drop the `_guard` and any remaining logs should get flushed
    let _guard = logger::logger_init();

    let host = env::var("SERVER_HOST").unwrap_or("0.0.0.0".to_string());
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

    let app_state = Arc::new(AppState { db, redis_pool });

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
        .split_for_parts();

    // socket
    let (layer, io) = SocketIo::builder()
        .with_state(events::store::Clients::default())
        .build_layer();
    io.ns(
        "/socket",
        events::handlers::on_connection.with(events::handlers::authenticate_middleware),
    );

    let app = router
        .layer(middleware)
        // swagger ui
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", api))
        .fallback(routes::fallback)
        .layer(layer)
        .with_state(app_state);

    info!("Starting server");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;

    info!("Listening on {host}:{port}");

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub redis_pool: Pool<RedisConnectionManager>,
}
