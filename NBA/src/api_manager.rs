use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

const API_URL: &str = "https://api.balldontlie.io/v1";

#[derive(Deserialize)]
struct Page<T> {
    data: Vec<T>,
    meta: Option<PageMeta>,
}

#[derive(Deserialize)]
struct PageMeta {
    next_cursor: Option<i64>,
}

#[derive(Deserialize)]
struct ApiTeam {
    id: i64,
    abbreviation: String,
    full_name: String,
    city: String,
}

#[derive(Deserialize)]
struct ApiPlayer {
    id: i64,
    first_name: String,
    last_name: String,
    position: Option<String>,
}

#[derive(Deserialize)]
struct ApiGame {
    id: i64,
    date: String,
    home_team: ApiTeamRef,
    visitor_team: ApiTeamRef,
    home_team_score: Option<i64>,
    visitor_team_score: Option<i64>,
}

#[derive(Deserialize)]
struct ApiStat {
    player: ApiPlayerRef,
    game: ApiGameRef,
    team: ApiTeamRef,
    min: Option<String>,
    pts: Option<f64>,
    ast: Option<f64>,
    reb: Option<f64>,
    plus_minus: Option<f64>,
}

#[derive(Deserialize)]
struct ApiPlayerRef {
    id: i64,
}

#[derive(Deserialize)]
struct ApiGameRef {
    id: i64,
}

#[derive(Deserialize)]
struct ApiTeamRef {
    id: i64,
}

pub async fn request_player_leaderboard(api_key: &str) -> Result<String> {
    let client = Client::new();

    let res = client
        .get("https://api.balldontlie.io/v1/players")
        .header("Authorization", api_key)
        .send()
        .await
        .context("failed to send request")?;

    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res
        .text()
        .await
        .context("failed to read response body")?;

    Ok(body)
}

pub async fn sync_season_data(pool: &SqlitePool, api_key: &str, season: i64) -> Result<()> {
    let client = Client::new();
    println!("Syncing teams...");
    let team_ids = sync_teams(pool, &client, api_key).await?;
    println!("Syncing players...");
    sync_players(pool, &client, api_key).await?;
    println!("Syncing season {season}...");
    let season_id = sync_season(pool, season).await?;
    println!("Syncing games...");
    sync_games(pool, &client, api_key, season, season_id, &team_ids).await?;
    println!("Syncing player stats...");
    sync_stats(pool, &client, api_key, season, &team_ids).await?;
    Ok(())
}

async fn sync_season(pool: &SqlitePool, year: i64) -> Result<i64> {
    sqlx::query("INSERT INTO seasons (year) VALUES (?) ON CONFLICT(year, league) DO NOTHING")
        .bind(year)
        .execute(pool)
        .await?;

    Ok(sqlx::query_scalar("SELECT id FROM seasons WHERE year = ? AND league = 'NBA'")
        .bind(year)
        .fetch_one(pool)
        .await?)
}

async fn sync_teams(pool: &SqlitePool, client: &Client, api_key: &str) -> Result<HashMap<i64, i64>> {
    let teams = fetch_all::<ApiTeam>(client, api_key, "/teams").await?;
    let team_count = teams.len();
    let mut team_ids = HashMap::new();

    for team in teams {
        let existing_id = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM teams WHERE abbreviation = ? OR id = ? LIMIT 1",
        )
        .bind(&team.abbreviation)
        .bind(team.id)
        .fetch_optional(pool)
        .await?;

        let local_id = existing_id.unwrap_or(team.id);
        sqlx::query(
            "INSERT INTO teams (id, abbreviation, name, city) VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, city = excluded.city",
        )
        .bind(local_id)
        .bind(team.abbreviation)
        .bind(team.full_name)
        .bind(team.city)
        .execute(pool)
        .await?;
        team_ids.insert(team.id, local_id);
    }

    println!("Synced {team_count} teams");
    Ok(team_ids)
}

async fn sync_players(pool: &SqlitePool, client: &Client, api_key: &str) -> Result<()> {
    let players = fetch_all::<ApiPlayer>(client, api_key, "/players").await?;
    let player_count = players.len();

    for player in players {
        let position = player.position.filter(|value| matches!(value.as_str(), "G" | "F" | "C"));
        sqlx::query(
            "INSERT INTO players (id, first_name, last_name, position) VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET first_name = excluded.first_name,
             last_name = excluded.last_name, position = excluded.position",
        )
        .bind(player.id)
        .bind(player.first_name)
        .bind(player.last_name)
        .bind(position)
        .execute(pool)
        .await?;
    }

    println!("Synced {player_count} players");
    Ok(())
}

async fn fetch_all<T>(client: &Client, api_key: &str, endpoint: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let mut records = Vec::new();
    let mut cursor: Option<i64> = None;
    let mut page_number = 0;

    loop {
        page_number += 1;
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        let url = match cursor {
            Some(value) => format!("{API_URL}{endpoint}{separator}per_page=100&cursor={value}"),
            None => format!("{API_URL}{endpoint}{separator}per_page=100"),
        };
        let mut response = None;

        for attempt in 0..5 {
            let current = client
                .get(url.clone())
                .header("Authorization", api_key)
                .send()
                .await
                .context(format!("failed to request BallDontLie data from {url}"))?;

            if current.status().as_u16() != 429 {
                response = Some(current);
                break;
            }

            let retry_seconds = current
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(2_u64.saturating_pow(attempt + 1));
            println!("Rate limited on page {page_number}; retrying in {retry_seconds}s...");
            sleep(Duration::from_secs(retry_seconds.min(60))).await;
        }

        let response = response.ok_or_else(|| anyhow::anyhow!(
            "BallDontLie rate limit persisted after 5 retries for {url}"
        ))?;
        let status = response.status();
        let body = response.text().await.context("failed to read BallDontLie response")?;
        if !status.is_success() {
            return Err(anyhow::anyhow!("BallDontLie returned HTTP {status} for {url}: {body}"));
        }
        let page = serde_json::from_str::<Page<T>>(&body)
            .with_context(|| format!("failed to decode BallDontLie response from {endpoint}"))?;

        records.extend(page.data);
        cursor = page.meta.and_then(|meta| meta.next_cursor);
        if cursor.is_none() {
            break;
        }
        sleep(Duration::from_millis(1200)).await;
    }

    Ok(records)
}

async fn sync_games(pool: &SqlitePool, client: &Client, api_key: &str, season: i64, season_id: i64, team_ids: &HashMap<i64, i64>) -> Result<()> {
    let games = fetch_all::<ApiGame>(client, api_key, &format!("/games?seasons[]={season}")).await?;
    let game_count = games.len();

    for game in games {
        let home_team_id = team_ids.get(&game.home_team.id).copied().context("home team was not imported")?;
        let away_team_id = team_ids.get(&game.visitor_team.id).copied().context("away team was not imported")?;
        sqlx::query(
            "INSERT INTO games (id, season_id, game_date, home_team_id, away_team_id, home_score, away_score)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET season_id = excluded.season_id,
             game_date = excluded.game_date, home_team_id = excluded.home_team_id,
             away_team_id = excluded.away_team_id, home_score = excluded.home_score,
             away_score = excluded.away_score",
        )
        .bind(game.id)
        .bind(season_id)
        .bind(game.date)
        .bind(home_team_id)
        .bind(away_team_id)
        .bind(game.home_team_score)
        .bind(game.visitor_team_score)
        .execute(pool)
        .await?;
    }

    println!("Synced {game_count} games for {season}");
    Ok(())
}

async fn sync_stats(pool: &SqlitePool, client: &Client, api_key: &str, season: i64, team_ids: &HashMap<i64, i64>) -> Result<()> {
    let stats = fetch_all::<ApiStat>(client, api_key, &format!("/stats?seasons[]={season}")).await?;
    let stat_count = stats.len();

    for stat in stats {
        let team_id = team_ids.get(&stat.team.id).copied().context("stat team was not imported")?;
        sqlx::query(
            "INSERT INTO player_game_stats
             (player_id, game_id, team_id, minutes, points, assists, rebounds, plus_minus)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(player_id, game_id) DO UPDATE SET team_id = excluded.team_id,
             minutes = excluded.minutes, points = excluded.points, assists = excluded.assists,
             rebounds = excluded.rebounds, plus_minus = excluded.plus_minus",
        )
        .bind(stat.player.id)
        .bind(stat.game.id)
        .bind(team_id)
        .bind(parse_minutes(stat.min.as_deref()))
        .bind(stat.pts)
        .bind(stat.ast)
        .bind(stat.reb)
        .bind(stat.plus_minus)
        .execute(pool)
        .await?;
    }

    println!("Synced {stat_count} player game stats for {season}");
    Ok(())
}

fn parse_minutes(value: Option<&str>) -> Option<f64> {
    let value = value?;
    if let Ok(minutes) = value.parse::<f64>() {
        return Some(minutes);
    }
    let mut parts = value.split(':');
    let minutes = parts.next()?.parse::<f64>().ok()?;
    let seconds = parts.next()?.parse::<f64>().ok()?;
    Some(minutes + seconds / 60.0)
}