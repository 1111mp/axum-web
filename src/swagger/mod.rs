use crate::dtos::user_dtos;
use crate::routes::post as Post;
use crate::routes::upload as Upload;
use crate::routes::user as User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

pub mod post_schemas;
pub mod user_schemas;

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://127.0.0.1:3000", description = "Local dev server")
    ),
    components(
        schemas(
			post_schemas::PostSchema,
			user_schemas::UserSchema, user_dtos::CreateUserDto, user_dtos::DeleteUserParam, user_dtos::DeleteUserDto, user_dtos::LoginUserDto, user_dtos::RedirectParam,
		)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "User", description = "User API endpoints"),
        (name = "Post", description = "Post API endpoints"),
        (name = "Upload", description = "Upload API endpoints")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "app_auth_key",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("app_auth_key"))),
            )
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseSchema<T> {
    pub status_code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(ToSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ErrorResponseSchema {
    pub status_code: i32,
    pub message: Option<String>,
}
