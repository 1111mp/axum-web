use super::AppState;

use crate::{
    http_exception,
    utils::{exception::HttpException, http_resp::HttpResponse},
};

use axum::{body::Bytes, extract::Multipart, routing, BoxError, Router};
use axum_macros::debug_handler;
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

const UPLOADS_DIRECTORY: &str = "uploads";

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/v1/post", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new().route("/", routing::post(upload_handler))
}

#[debug_handler]
async fn upload_handler(mut multipart: Multipart) -> Result<HttpResponse<()>, HttpException> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        stream_to_file(&file_name, field).await?;
    }

    Ok(HttpResponse::Json {
        message: None,
        data: None,
    })
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), HttpException>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        http_exception!(BadRequestException, "Invalid path");
    }

    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
    let mut file = BufWriter::new(File::create(path).await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;
    Ok(())
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
