use super::Context;
use color_eyre::Result;
use entities::{prelude::*, roles};
use poise::serenity_prelude::Role;
use sea_orm::{prelude::*, IntoActiveModel};

pub async fn download_roles(ctx: &Context<'_>) -> Result<()> {
    let db = &ctx.data().db;
    let guild = &ctx.guild().unwrap();
    let roles = &guild.roles;
    for role in roles.values() {
        if let Some(_) = get_role_model(role, db).await? {
            continue;
        }
        let model = role_to_model(role).await?;
        model.insert(db).await?;
    }
    Ok(())
}

async fn role_to_model(role: &Role) -> Result<roles::ActiveModel> {
    use sea_orm::ActiveValue::Set;
    let mut model = <roles::ActiveModel as ActiveModelTrait>::default();
    model.role_sf = Set(role.id.get());
    todo!();
    Ok(model)
}

async fn get_role_model(role: &Role, db: &DatabaseConnection) -> Result<Option<roles::ActiveModel>> {
    let model = Roles::find()
        .filter(roles::Column::RoleSf.eq(role.id.get()))
        .one(db)
        .await?
        .map(|x| x.into_active_model());
    Ok(model)
}

