mod app;
mod dtos;
mod extensions;
mod guards;
mod routes;
mod swagger;
mod utils;

fn main() {
    let result = app::start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
