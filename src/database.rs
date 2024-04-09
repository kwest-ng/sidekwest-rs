use std::time::Duration;

use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn db_connect(url: &str) -> Result<DatabaseConnection> {
    let mut opts = ConnectOptions::new(url);
    opts.connect_timeout(Duration::from_secs(1))
    .idle_timeout(Duration::from_secs(60))
    .max_connections(10)
    .min_connections(1)
    .sqlx_logging(true)
    .test_before_acquire(true)
    ;
    let db = Database::connect(opts).await?;
    Ok(db)
}
