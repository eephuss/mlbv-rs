mod config;
mod gamedata;
mod session;
mod streams;

use crate::streams::{play_game_stream};
use crate::{config::AppConfig};
use crate::session::MlbSession;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::load()?;
    let date = "2025-10-20";
    let team = "Seattle Mariners";

    let session = MlbSession::new()?
        .login_and_authorize(&cfg.credentials.username, &cfg.credentials.password)
        .await?;

    // TODO: Add cute messaging depending if there are no games during season (off-day) or offseason (see you next spring!)
    play_game_stream(&session, team, date, "VIDEO", "NETWORK", None).await?;

    Ok(())
}
