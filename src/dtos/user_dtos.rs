use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Item create user.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct CreateUserDto {
    #[validate(length(min = 1, message = "Invalid name"))]
    pub name: String,
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct LoginUserDto {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct DeleteUserParam {
    #[validate(range(min = 1, message = "Invalid id"))]
    pub id: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct DeleteUserDto {
    pub thoroughly: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct RedirectParam {
    pub uri: Option<String>,
}
