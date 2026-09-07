#[allow(unused)]
use sqlx::{Sqlite, SqlitePool};
use dotenvy::dotenv;
use std::env;
use serde::Serialize;
use rocket::{fairing::{Fairing, Info, Kind}, http::Header, response::Response, Request, State};

use crate::sql_manager::finish_connection;

#[macro_use] extern crate rocket;
extern crate tokio;

pub mod hasher;
pub mod sql_manager;
pub mod api_manager;

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Allow frontend CORS requests",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    }
}

#[rocket::main]
#[allow(unused)]
async fn main() -> Result<(), rocket::Error>
{
    dotenv()
        .expect("Failed to set up dotenv");

    let db_url = env::var("DATABASE_URL").expect("No database url");
    let api_key = env::var("API_KEY").unwrap_or_default();

    let pool: SqlitePool = sql_manager::establish_connection(&db_url)
        .await
        .expect("failed to establish database connection");

    if env::args().nth(1).as_deref() == Some("sync") {
        if api_key.is_empty() {
            panic!("API_KEY is required when running cargo run -- sync <season>");
        }
        let season = env::args()
            .nth(2)
            .unwrap_or_else(|| "2024".to_string())
            .parse::<i64>()
            .expect("season must be a four-digit year");
        if let Err(error) = api_manager::sync_season_data(&pool, &api_key, season).await {
            eprintln!("BallDontLie sync failed: {error:#}");
            finish_connection(&pool)
                .await
                .expect("Closing failing!");
            return Ok(());
        }
        finish_connection(&pool)
            .await
            .expect("Closing failing!");
        return Ok(());
    }

    let _ = rocket::build()
        .attach(Cors)
        .manage(pool.clone())
        .mount("/", routes![leaderboard, players_leaderboard])
        .launch()
        .await?;

    finish_connection(&pool)
        .await.expect("Closing failing!");

    Ok(())
}

#[derive(Serialize)]
struct LeaderboardPlayer {
    rank: usize,
    id: i64,
    name: String,
    ppg: f64,
    rpg: f64,
    apg: f64,
    bpm: f64,
}

#[get("/leaderboard?<season>&<limit>")]
async fn leaderboard(
    pool: &State<SqlitePool>,
    season: i64,
    limit: Option<i64>,
) -> Result<String, rocket::response::status::Custom<String>>
{
    let limit = limit.unwrap_or(50).clamp(1, 100);
    let rows = sqlx::query_as::<_, (i64, String, String, f64, f64, f64, f64)>(
        "SELECT p.id, p.first_name, p.last_name,
                COALESCE(AVG(s.points), 0.0),
                COALESCE(AVG(s.rebounds), 0.0),
                COALESCE(AVG(s.assists), 0.0),
                COALESCE(AVG(s.plus_minus), 0.0)
         FROM players p
         JOIN player_game_stats s ON s.player_id = p.id
         JOIN games g ON g.id = s.game_id
         JOIN seasons se ON se.id = g.season_id
         WHERE se.year = ?
         GROUP BY p.id, p.first_name, p.last_name
         ORDER BY AVG(s.plus_minus) DESC
         LIMIT ?",
    )
    .bind(season)
    .bind(limit)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| rocket::response::status::Custom(
            rocket::http::Status::InternalServerError,
            e.to_string(),
        ))?;

    let players = rows.into_iter().enumerate().map(|(index, row)| LeaderboardPlayer {
        rank: index + 1,
        id: row.0,
        name: format!("{} {}", row.1, row.2),
        ppg: row.3,
        rpg: row.4,
        apg: row.5,
        bpm: row.6,
    }).collect::<Vec<_>>();

    serde_json::to_string(&players).map_err(|e| rocket::response::status::Custom(
        rocket::http::Status::InternalServerError,
        e.to_string(),
    ))
}

#[get("/players?<season>&<limit>")]
async fn players_leaderboard(
    pool: &State<SqlitePool>,
    season: i64,
    limit: Option<i64>,
) -> Result<String, rocket::response::status::Custom<String>> {
    leaderboard(pool, season, limit).await
}