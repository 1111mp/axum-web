use crate::{
    api_doc::ApiDoc,
    core::{config, logger, state},
    events, routes,
};
use axum::http::{header, HeaderName, Method, Request};
use bb8_redis::RedisConnectionManager;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use socketioxide::{handler::ConnectHandler, SocketIo};
use std::{sync::Arc, time::Duration};
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

#[dotenvy::load(path = ".env.prod")]
#[dotenvy::load(path = ".env.local")]
#[dotenvy::load(path = ".env")]
#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    let config = config::Config::global();
    // Exiting the context of `main` will drop the `_guard` and any remaining logs should get flushed
    let _guard = logger::logger_init(config);

    // database
    let db = Database::connect(config.database_url()).await?;
    // Apply all pending migrations
    // Migrator::up(&db, None).await?;
    // Drop all tables from the database, then reapply all migrations
    // Migrator::fresh(&db).await?;

    // Sync the schema with the database
    // https://www.sea-ql.org/SeaORM/zh-CN/docs/generate-entity/entity-first/
    // db.get_schema_registry("axum-web::entity::*")
    //     .sync(&db)
    //     .await?;

    info!("Successfully connected to the database");

    // redis
    let manager = RedisConnectionManager::new(config.redis_url())?;
    let redis_pool = bb8::Pool::builder().build(manager).await?;

    info!("Successfully connected to the redis");

    let app_state = Arc::new(state::AppState { db, redis_pool });

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

    let addr = format!("{}:{}", config.server_host(), config.server_port(),);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Listening on {}", &addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown");

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
          info!("Ctrl+C received, shutting down");
        },
        _ = terminate => {
          info!("Terminate signal received, shutting down");
        },
    }

    info!("Received shutdown signal, shutting down...");
}
