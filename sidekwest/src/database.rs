use std::path::PathBuf;
use std::time::Duration;

use color_eyre::Result;
use migration::MigratorTrait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, ConnectOptions, Database};

pub async fn test_database() -> Result<()> {
    let path = PathBuf::from("ephemeral-test-db.sqlite");
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let db = db_connect(&url).await?;
    migration::Migrator::fresh(&db).await?;
    let role = entities::roles::ActiveModel {
        discord_id: ActiveValue::set(1),
        name: ActiveValue::set("hello".to_owned()),
        position: ActiveValue::set(1),
        global_perms: ActiveValue::set(1),
        ..Default::default()
    };
    role.insert(&db).await?;
    std::fs::remove_file(path)?;
    Ok(())
}

pub async fn db_connect(url: &str) -> Result<DatabaseConnection> {
    let mut opts = ConnectOptions::new(url);
    opts.connect_timeout(Duration::from_secs(1))
        .idle_timeout(Duration::from_secs(60))
        .max_connections(10)
        .min_connections(1)
        .sqlx_logging(true)
        .test_before_acquire(true);
    let db = Database::connect(opts).await?;
    Ok(db)
}
