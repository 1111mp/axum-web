use tracing::{level_filters::LevelFilter, subscriber::set_global_default};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};

pub fn logger_init() -> (WorkerGuard, WorkerGuard) {
    let logger_dir = std::env::var("LOG_DIR").unwrap_or("./logs".to_string());
    let info_file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app.info")
        .filename_suffix("log")
        // maybe will replace with size-based rotation
        // wait PR: https://github.com/tokio-rs/tracing/pull/2497
        .max_log_files(14)
        .build(&logger_dir)
        .expect("Initializing rolling info file appender failed");
    let error_file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app.error")
        .filename_suffix("log")
        // maybe will replace with size-based rotation
        // wait PR: https://github.com/tokio-rs/tracing/pull/2497
        .max_log_files(14)
        .build(&logger_dir)
        .expect("Initializing rolling error file appender failed");
    let (info_non_blocking, info_guard) = tracing_appender::non_blocking(info_file_appender);
    let (error_non_blocking, error_guard) = tracing_appender::non_blocking(error_file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::Layer::new().with_writer(std::io::stdout))
        .with(
            fmt::Layer::new()
                .with_writer(info_non_blocking)
                .with_ansi(false)
                .with_filter(LevelFilter::INFO),
        )
        .with(
            fmt::Layer::new()
                .with_writer(error_non_blocking)
                .with_ansi(false)
                .with_filter(LevelFilter::ERROR),
        );

    set_global_default(subscriber).expect("Unable to set a global subscriber");
    (info_guard, error_guard)
}
