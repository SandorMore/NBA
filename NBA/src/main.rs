#[allow(unused)]

use sqlx::{Sqlite, SqlitePool};
use dotenvy::dotenv;
use std::env;


#[macro_use] extern crate rocket;
#[macro_use] extern crate tokio;


pub mod hasher;
pub mod sql_manager;

#[rocket::main]
async fn main() -> Result<(), rocket::Error>
{
    dotenv();
    
    let db_url = env::var("DATABASE_URL")
        .expect("No database url");

    let pool: SqlitePool = sql_manager::establish_connection(&db_url)
        .await
        .expect("failed to establish database connection");

    rocket::build()
        .mount("/", routes![front_page])
        .launch()
        .await?;

    Ok(())
}

#[get("/")]
fn front_page() -> &'static str
{
    println!("Main page opened");
    return "hello world";
}   