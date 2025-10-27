mod config;
mod gamedata;
mod session;
mod streams;
mod teams;

use crate::config::AppConfig;
use crate::session::MlbSession;
use crate::streams::{FeedType, MediaType};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = AppConfig::load()?;
    let date = "2025-09-24";
    let team = "Washington Nationals";

    let session = MlbSession::new()?
        .authorize(&cfg.credentials.username, &cfg.credentials.password)
        .await?;

    // TODO: Add cute messaging depending if there are no games during season (off-day) or offseason (see you next spring!)
    streams::find_and_play_stream(&session, team, date, MediaType::Video, FeedType::Away, None)
        .await?;

    Ok(())
}
