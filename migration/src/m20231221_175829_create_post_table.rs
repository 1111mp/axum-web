use sea_orm_migration::{prelude::*, sea_orm::EnumIter};

use crate::m20231221_180214_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Post::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Post::Title).string().not_null().unique_key())
                    .col(ColumnDef::new(Post::Text).string().not_null())
                    .col(
                        ColumnDef::new(Post::Category)
                            .enumeration(Category::Table, [Category::Feed, Category::Story])
                            .default("Story"),
                    )
                    .col(
                        ColumnDef::new(Post::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Post::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    // Create Foreign Key
                    .col(ColumnDef::new(Post::UserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-post-user-id")
                            .from(Post::Table, Post::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Or
        // Create Foreign Key
        // manager
        //     .create_foreign_key(
        //         sea_query::ForeignKey::create()
        //             .name("key-post-user-id")
        //             .from(Post::Table, Post::UserId)
        //             .to(User::Table, User::Id)
        //             .to_owned(),
        //     )
        //     .await?;

        // Create Index
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-post_title")
                    .table(Post::Table)
                    .col(Post::Title)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
    Category,
    UserId,
    #[sea_orm(column_name = "created_at")]
    CreatedAt,
    #[sea_orm(column_name = "updated_at")]
    UpdatedAt,
}

#[derive(Iden, EnumIter)]
pub enum Category {
    Table,
    #[iden = "Feed"]
    Feed,
    #[iden = "Story"]
    Story,
}
