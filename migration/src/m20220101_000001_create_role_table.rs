use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    // table essentials
                    .col(
                        ColumnDef::new(Roles::Id)
                            .integer()
                            .primary_key()
                            .auto_increment()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Roles::DiscordId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    // values
                    .col(ColumnDef::new(Roles::Name).string().not_null())
                    .col(ColumnDef::new(Roles::Position).integer().not_null())
                    .col(ColumnDef::new(Roles::GlobalPerms).integer().not_null())
                    // booleans
                    .col(
                        ColumnDef::new(Roles::HasColor)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::HasIcon)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsHoisted)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsMentionable)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsBotRole)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsIntegrationRole)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsBoosterRole)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsPurchasable)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .col(
                        ColumnDef::new(Roles::IsLinked)
                            .boolean()
                            .not_null()
                            .default(SimpleExpr::Constant(Value::Bool(Some(false)))),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Roles {
    Table,
    Id,
    // Values
    DiscordId,
    Name,
    Position,
    GlobalPerms,
    // booleans
    HasColor,
    HasIcon,
    IsHoisted,
    IsMentionable,
    IsBotRole,
    IsIntegrationRole,
    IsBoosterRole,
    IsPurchasable,
    IsLinked,
}
