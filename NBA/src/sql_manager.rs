#[allow(unused)]
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn establish_connection(db_url: &str) -> Result<SqlitePool, sqlx::Error>
{
    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    println!("Established connection with the database");

    Ok(pool)
}

pub async fn finish_connection(pool: &SqlitePool) -> Result<(), sqlx::Error>
{
    pool.close().await;

    Ok(())
}