use std::env;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use axum_macros::debug_handler;
use bcrypt::verify;
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

use crate::utils::{
    http_resp::{make_resp_from_db_err, JsonResponse},
    jwt::jwt_encode,
};

use super::{extractor::JsonParser, AppState};

pub fn create_route() -> Router<AppState> {
    Router::new().nest("/user", make_api())
}

fn make_api() -> Router<AppState> {
    Router::new()
        .route("/signup", post(user_signup))
        .route("/signin", post(user_signin))
}

#[debug_handler]
async fn user_signup(
    State(state): State<AppState>,
    JsonParser(input): JsonParser<CreateUser>,
) -> Response {
    let ret = user::ActiveModel {
        name: Set(input.name),
        email: Set(input.email),
        password: Set(input.password),
        ..Default::default()
    }
    .insert(&state.db)
    .await;

    match ret {
        Ok(user) => {
            let mut json = json!(&user);
            let user_json = json.as_object_mut().unwrap();
            user_json.remove("password");

            JsonResponse::OK {
                message: "success".to_string(),
                data: Some(user_json),
            }
            .into_response()
        }
        Err(err) => make_resp_from_db_err(&err),
    }
}

#[debug_handler]
async fn user_signin(
    State(state): State<AppState>,
    cookies: Cookies,
    JsonParser(input): JsonParser<LoginUser>,
) -> Response {
    let user_model = match User::find()
        .filter(user::Column::Email.eq(&input.email))
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(user) => user,
            None => {
                return JsonResponse::<()>::NotFound {
                    // error message is "Invalid email or password" maybe better
                    message: format!("No user found with email {}", &input.email),
                }
                .into_response();
            }
        },
        Err(err) => return make_resp_from_db_err(&err),
    };

    // verify password
    let valid = if let Ok(valid) = verify(&input.password, &user_model.password) {
        valid
    } else {
        return JsonResponse::<()>::Unauthorized {
            message: "Invalid email or password".to_string(),
        }
        .into_response();
    };

    if !valid {
        return JsonResponse::<()>::Unauthorized {
            message: "Invalid email or password".to_string(),
        }
        .into_response();
    }

    let token = match jwt_encode(&user_model) {
        Ok(t) => t,
        Err(err) => {
            return JsonResponse::<()>::InternalServerError {
                message: err.to_string(),
            }
            .into_response();
        }
    };

    let name = env::var("APP_AUTH_KEY").unwrap_or("app_auth_key".to_string());
    let mut cookie = Cookie::new(name.clone(), token.clone());
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookies.add(cookie);

    let mut json = json!(&user_model).clone();
    let user_json = json.as_object_mut().unwrap();
    user_json.remove("password");

    JsonResponse::OK {
        message: "success".to_string(),
        data: Some(user_json),
    }
    .into_response()
}

#[derive(Debug, Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, message = "Invalid name"))]
    name: String,
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}

#[derive(Debug, Deserialize, Validate)]
struct LoginUser {
    #[validate(email(message = "Invalid email"))]
    email: String,
    #[validate(length(min = 8, message = "Invalid password"))]
    password: String,
}
