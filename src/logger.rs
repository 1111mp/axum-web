use tracing::subscriber::set_global_default;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

pub fn logger_init() -> WorkerGuard {
    let log_dir = std::env::var("LOG_DIR").unwrap_or("./logs".to_string());
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        // will replace with `with_max_bytes`
        // wait for https://github.com/tokio-rs/tracing/pull/2497
        .max_log_files(14)
        .build(log_dir)
        .expect("Initializing rolling file appender failed");
    let (no_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::Layer::new().with_writer(std::io::stdout))
        .with(fmt::Layer::new().with_writer(no_blocking).with_ansi(false));

    set_global_default(subscriber).expect("Unable to set a global subscriber");
    _guard
}
