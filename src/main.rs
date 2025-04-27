mod api_doc;
mod app;
mod dtos;
mod events;
mod exception;
mod extractors;
mod guards;
mod logger;
mod routes;
mod utils;

fn main() {
    let result = app::start();

    if let Some(err) = result.err() {
        eprintln!("{err}");
    }
}
