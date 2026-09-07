use anyhow::{Result, Context};
use reqwest;

pub async fn request_player_leaderboard(api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();

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