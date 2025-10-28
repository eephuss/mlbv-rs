mod config;
mod gamedata;
mod session;
mod streams;
mod teams;

use crate::config::AppConfig;
use crate::session::MlbSession;
use crate::streams::{FeedType, MediaType};
use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = AppConfig::load()?;

    // Subscribe to tracing logs.
    let log_level = if cfg.debug { tracing::Level::DEBUG } else { tracing::Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_level(true)
        .compact()
        .init();

    let date = "2025-09-14";
    let team = "Boston Red Sox";
    // teams::TEAMS.matc

    let session = MlbSession::new()?
        .authorize(&cfg.credentials.username, &cfg.credentials.password)
        .await?
        .find_and_play_stream(team, date, MediaType::Video, FeedType::Away, None, Some("mpv"))
        .await?;

    Ok(())
}
