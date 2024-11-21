use super::{Deserialize, Serialize, ToSchema};

#[derive(ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PostSchema {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub text: String,
    #[schema(default = "Feed")]
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}
