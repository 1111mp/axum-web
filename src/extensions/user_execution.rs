use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

use crate::utils::jwt::Claims;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub create_at: Option<DateTimeUtc>,
    pub update_at: Option<DateTimeUtc>,
}
