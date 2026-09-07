#[allow(unused)]
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn establish_connection(db_url: &str) -> Result<SqlitePool, sqlx::Error>
{
    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Established connection with the database");

    Ok(pool)
}

pub async fn finish_connection(pool: &SqlitePool) -> Result<(), sqlx::Error>
{
    pool.close().await;

    Ok(())
}