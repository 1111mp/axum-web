use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("user")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string_uniq("name"))
                    .col(string("password"))
                    .col(string_uniq("email"))
                    .col(date_time("created_at"))
                    .col(date_time("updated_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-user-name")
                    .table("user")
                    .col("name")
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("post")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("title"))
                    .col(string("text"))
                    .col(enumeration("category", "category", ["Feed", "Store"]))
                    .col(date_time("created_at"))
                    .col(date_time("updated_at"))
                    .col(integer("user_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-post-user-id")
                            .from("post", "user_id")
                            .to("user", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-user-name").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("user").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("post").to_owned())
            .await?;

        Ok(())
    }
}
