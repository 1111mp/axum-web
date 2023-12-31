mod app;
mod routes;
mod middlewares;
mod utils;
mod swagger;

fn main() {
    let result = app::start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
