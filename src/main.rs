mod cli;
mod config;
mod gamedata;
mod session;
mod streams;
mod teamdata;

use crate::cli::Cli;
use crate::config::AppConfig;
use crate::session::MlbSession;
use crate::streams::MediaType;
use crate::teamdata::Team;
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
    let cfg = AppConfig::load()?;
    let log_level = if cfg.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_level(true)
        .compact()
        .init();

    let session = MlbSession::new()?;
    let cli = Cli::parse();

    // Use user-provided date. If none, return today's date.
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
    let media_type = if cli.audio {
        MediaType::Audio
    } else {
        MediaType::Video
    };

    // If user specified a team, then play
    if let Some(team_code) = cli.team {
        let team_name = match Team::find_by_code(team_code) {
            Some(team) => team.name,
            None => anyhow::bail!("Invalid team code."),
        };

        session
            .authorize(&cfg.credentials.username, &cfg.credentials.password)
            .await?
            .find_and_play_stream(
                team_name,
                date,
                media_type,
                cli.feed,
                cli.game_number,
                cfg.stream.video_player,
            )
            .await?
    } else {
        if let Some(schedule) = session.fetch_games_by_date(&date).await? {
            schedule.display_game_data();
        } else {
            // TODO: Detect when in off-season and add cute "see you next spring!" message.
            println!("No games scheduled for {date}");
        }
    }
    Ok(())
}
