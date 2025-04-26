mod app;
mod dtos;
mod exception;
mod extractors;
mod guards;
mod logger;
mod routes;
mod swagger;
mod utils;

fn main() {
    let result = app::start();

    if let Some(err) = result.err() {
        eprintln!("{err}");
    }
}
