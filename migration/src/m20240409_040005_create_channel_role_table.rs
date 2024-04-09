use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_role_table::Roles;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    // Table essentials
                    .table(ChannelRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChannelRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // Values
                    .col(ColumnDef::new(ChannelRoles::ChannelId).integer().not_null())
                    .col(ColumnDef::new(ChannelRoles::RoleId).integer().not_null())
                    .col(ColumnDef::new(ChannelRoles::Perms).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelRoles::Table, ChannelRoles::RoleId)
                            .to(Roles::Table, Roles::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChannelRoles {
    Table,
    Id,
    // Values
    ChannelId,
    RoleId,
    Perms,
}
