use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
mod aggregation_service;
mod db;
use eyre::Result;

#[derive(Debug, Clone)]
pub struct AggregationRpc {
    pub db_pool: SqlitePool,
}

pub async fn new_db(database_url: &str) -> Result<SqlitePool> {
    let db_pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| eyre::eyre!("failed to connect to database: {}", e))?;
    Ok(db_pool)
}
