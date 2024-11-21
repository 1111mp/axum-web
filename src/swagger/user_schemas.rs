use super::{Deserialize, Serialize, ToSchema};

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserSchema {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub token: String,
    pub created_at: String,
    pub updated_at: String,
}
