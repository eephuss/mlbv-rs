mod api;
mod cli;
mod config;
mod data;
mod player;

use crate::api::mediagateway::streams::MediaType;
use crate::api::session::MlbSession;
use crate::cli::Cli;
use crate::config::AppConfig;
use crate::data::teamdata::Team;
use anyhow::Result;
use chrono::{Duration, Local};
use clap::Parser;

fn main() -> Result<()> {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);

        std::process::exit(1);
    }
    Ok(())
}

#[tokio::main]
async fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.init {
        AppConfig::generate_config()?
    };
    let cfg = AppConfig::load()?;
    let log_level = match cfg.debug {
        true => tracing::Level::DEBUG,
        false => tracing::Level::INFO,
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_level(true)
        .compact()
        .init();

    let session = MlbSession::new()?;

    // Use user-provided date. If none, return today's date
    let date = if let Some(game_date) = cli.date {
        game_date.0 // 0 extracts NaiveDate from GameDate
    } else if cli.yesterday {
        Local::now().date_naive() - Duration::days(1)
    } else if cli.tomorrow {
        Local::now().date_naive() + Duration::days(1)
    } else {
        Local::now().date_naive()
    };

    // Assume users want video broadcast unless they opt out
    let media_type = match cli.audio {
        true => MediaType::Audio,
        false => MediaType::Video,
    };

    // If user specified a team, then look for stream to play
    if let Some(team_code) = cli.team {
        let Some(team) = Team::find_by_code(team_code) else {
            anyhow::bail!("Invalid team code.")
        };

        session
            .authorize(&cfg.credentials.username, &cfg.credentials.password)
            .await?
            .find_and_play_stream(
                team,
                date,
                media_type,
                cli.feed,
                cli.game_number,
                cfg.stream.video_player,
            )
            .await?
    } else if let Some(schedule) = session.fetch_games_by_date(&date).await? {
        schedule.display_schedule();
    } else {
        // TODO: Detect when in off-season and add cute "see you next spring!" message.
        println!("No games scheduled for {date}");
    }

    Ok(())
}
