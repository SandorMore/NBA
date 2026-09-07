#[allow(unused)]
use sqlx::{Sqlite, SqlitePool};
use dotenvy::dotenv;
use std::env;
use rocket::State;

use crate::sql_manager::finish_connection;

#[macro_use] extern crate rocket;
extern crate tokio;

pub mod hasher;
pub mod sql_manager;
pub mod api_manager;

struct ApiKey(String);

#[rocket::main]
#[allow(unused)]
async fn main() -> Result<(), rocket::Error>
{
    dotenv()
        .expect("Failed to set up dotenv");

    let db_url = env::var("DATABASE_URL").expect("No database url");
    let api_key = env::var("API_KEY").expect("No api key found");

    let pool: SqlitePool = sql_manager::establish_connection(&db_url)
        .await
        .expect("failed to establish database connection");

    let _ = rocket::build()
        .manage(ApiKey(api_key))
        .manage(pool.clone())
        .mount("/", routes![players])
        .launch()
        .await?;

    finish_connection(&pool)
        .await.expect("Closing failing!");

    Ok(())
}

#[get("/players")]
async fn players(api_key: &State<ApiKey>) -> Result<String, rocket::response::status::Custom<String>>
{
    api_manager::request_player_leaderboard(&api_key.0)
        .await
        .map_err(|e| rocket::response::status::Custom(
            rocket::http::Status::InternalServerError,
            e.to_string(),
        ))
}