use crate::{
    app::AppState,
    utils::{exception::HttpException, http_resp::HttpResponse},
};

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Router,
};
use axum_macros::debug_handler;
use axum_typed_multipart::{BaseMultipart, FieldData, TryFromMultipart, TypedMultipartError};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use utoipa::ToSchema;

const UPLOADS_DIRECTORY: &str = "uploads";

pub fn protected_route() -> Router<AppState> {
    let router = Router::new()
        .route("/", routing::post(upload_handler))
        // 200M
        .layer(DefaultBodyLimit::max(1024 * 1024 * 200));

    Router::new().nest("/upload", router)
}

#[derive(TryFromMultipart, ToSchema)]
struct FileUpload {
    /// File's name
    #[schema(value_type = String)]
    pub name: String,

    /// File or files to upload
    #[form_data(limit = "200MiB")]
    #[schema(value_type = Vec<u8>)]
    pub file: FieldData<NamedTempFile>,
}

#[utoipa::path(
		post,
		path = "/api/v1/upload",
		request_body(content_type = "multipart/form-data", content = FileUpload),
		tag = "Upload"
)]
#[debug_handler]
async fn upload_handler(
    input: SelfTypedMultipart<FileUpload>,
) -> Result<HttpResponse<PathBuf>, HttpException> {
    let path = Path::new(UPLOADS_DIRECTORY).join(&input.name);
    input
        .data
        .file
        .contents
        .persist(&path)
        .map_err(|err| HttpException::InternalServerErrorException(Some(err.to_string())))?;

    Ok(HttpResponse::Json {
        message: None,
        data: Some(path),
    })
}

// Step 1: Define a custom error type.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MultipartException {
    message: String,
    status_code: u16,
}

// Step 2: Implement `IntoResponse` for the custom error type.
impl IntoResponse for MultipartException {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, axum::Json(self)).into_response()
    }
}

// Step 3: Implement `From<TypedMultipartError>` for the custom error type.
impl From<TypedMultipartError> for MultipartException {
    fn from(error: TypedMultipartError) -> Self {
        Self {
            message: error.to_string(),
            status_code: error.get_status().into(),
        }
    }
}

// Step 4: Define a type alias for the multipart request (Optional).
type SelfTypedMultipart<T> = BaseMultipart<T, MultipartException>;
