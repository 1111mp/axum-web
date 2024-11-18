use utoipa::ToSchema;

#[derive(ToSchema)]
pub(crate) struct RespError {
    code: i32,
    message: String,
}
