use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use entity::{prelude::User, user};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;

use crate::{routes::make_resp::make_resp_from_db_err, utils::jwt::jwt_encode};

use super::AppState;

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
    payload: Result<Json<CreateUser>, JsonRejection>,
) -> impl IntoResponse {
    let Json(input) = match payload {
        Ok(input) => input,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "code": 400,
                    "message": err.to_string()
                })),
            );
        }
    };

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

            (
                StatusCode::OK,
                Json(json!({
                    "code": 200,
                    "data": user_json
                })),
            )
        }
        Err(err) => make_resp_from_db_err(&err),
    }
}

#[debug_handler]
async fn user_signin(
    State(state): State<AppState>,
    payload: Result<Json<LoginUser>, JsonRejection>,
) -> impl IntoResponse {
    let Json(input) = match payload {
        Ok(input) => input,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "code": 400,
                    "message": err.to_string()
                })),
            );
        }
    };

    let user_model = match User::find()
        .filter(user::Column::Email.eq(&input.email))
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(user) => user,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "code": 404,
                        "message": format!("No user found with email {}", &input.email)
                    })),
                )
            }
        },
        Err(err) => return make_resp_from_db_err(&err),
    };

    let token = match jwt_encode(&user_model) {
        Ok(t) => t,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": 500,
                    "message": err.to_string()
                })),
            )
        }
    };

    let mut json = json!(&user_model).clone();
    let user_json = json.as_object_mut().unwrap();
    user_json.remove("password");
    user_json.insert("token".to_string(), json!(token));

    (
        StatusCode::OK,
        Json(json!({
            "code": 200,
            "data": user_json
        })),
    )
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginUser {
    email: String,
    password: String,
}
