//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use super::sea_orm_active_enums::Category;
use sea_orm::entity::prelude::*;
use serde::{ser::SerializeStruct, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub title: String,
    pub text: String,
    pub category: Option<Category>,
    pub create_at: Option<DateTimeUtc>,
    pub update_at: Option<DateTimeUtc>,
    pub user_id: i32,
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Model", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("category", &self.category)?;
        state.serialize_field(
            "create_at",
            &(if let Some(create_at) = &self.create_at {
                Some(create_at.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            }),
        )?;
        state.serialize_field(
            "update_at",
            &(if let Some(update_at) = &self.update_at {
                Some(update_at.format("%Y-%m-%d %H:%M:%S").to_string())
            } else {
                None
            }),
        )?;
        state.end()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
