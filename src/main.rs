mod api_doc;
mod app;
mod core;
mod dtos;
mod events;
mod extractors;
mod guards;
mod routes;
mod utils;

fn main() {
    let result = app::start();

    if let Some(err) = result.err() {
        eprintln!("{err}");
    }
}
