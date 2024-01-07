use crate::routes::post as Post;
use crate::routes::user as User;
use crate::utils::schema as Schema;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://127.0.0.1:3000", description = "Local dev server")
    ),
    paths(
        User::create_one, User::delete_one, User::user_login, User::user_signout,
        Post::get_all, Post::get_one
    ),
    components(
        schemas(
            Schema::RespError,
            User::RespForUser, User::UserInfo, User::CreateUser, User::DeleteUser, User::DeleteUserOpt, User::LoginUser, User::RedirectParam,
            Post::RespForPost, Post::RespForPosts, Post::PostInfo,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "User", description = "User items management API"),
        (name = "Post", description = "Post items management API")
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
