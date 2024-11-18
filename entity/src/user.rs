//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use bcrypt::hash;
use sea_orm::{entity::prelude::*, ActiveValue, Set};
use serde::{ser::SerializeStruct, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub password: String,
    #[sea_orm(unique)]
    pub email: String,
    pub create_at: Option<DateTimeUtc>,
    pub update_at: Option<DateTimeUtc>,
}

impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Model", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("email", &self.email)?;
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
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Will be triggered before insert / update
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if let ActiveValue::Set(password) = self.password {
            let password_hash = match hash(password, 10) {
                Ok(hash) => hash,
                Err(_) => {
                    return Err(DbErr::Custom(format!(
                        "[before_save] Invalid password, insert: {}",
                        insert
                    )))
                }
            };
            self.password = Set(password_hash);
        }

        Ok(self)
    }
}
