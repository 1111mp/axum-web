use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct QueryPostDto {
    #[validate(range(min = 1, message = "Invalid id"))]
    pub id: i32,
}
