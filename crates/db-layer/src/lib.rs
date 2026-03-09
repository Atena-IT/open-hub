pub mod queries;

use sqlx::postgres::PgPoolOptions;
pub use sqlx::{postgres::PgRow, PgPool};

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
