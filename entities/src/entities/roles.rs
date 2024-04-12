//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub guild_sf: i32,
    #[sea_orm(unique)]
    pub role_sf: i32,
    pub name: String,
    pub position: i32,
    pub global_perms: i32,
    pub has_color: bool,
    pub has_icon: bool,
    pub is_hoisted: bool,
    pub is_mentionable: bool,
    pub is_bot_role: bool,
    pub is_integration_role: bool,
    pub is_booster_role: bool,
    pub is_purchasable: bool,
    pub is_linked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channel_roles::Entity")]
    ChannelRoles,
    #[sea_orm(has_many = "super::user_roles::Entity")]
    UserRoles,
}

impl Related<super::channel_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChannelRoles.def()
    }
}

impl Related<super::user_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRoles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
