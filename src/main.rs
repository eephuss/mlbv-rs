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
use chrono::Utc;
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
    // TODO: Handle invalid date inputs and conflicts with today and tomorrow flags.
    let date = match cli.date {
        Some(s) => s.to_string(),
        None => Utc::now().format("%Y-%m-%d").to_string(),
    };

    // Assume users want video broadcast unless they opt out
    let media_type = if cli.audio {
        MediaType::Audio
    } else {
        MediaType::Video
    };

    // If user specifies a feed type, use it. Else match to team.
    let feed_type = cli.feed;

    // Get game-number if provided
    let game_number = cli.game_number;

    // Get media_player from config
    let media_player = cfg.stream.video_player;

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
                &date,
                media_type,
                feed_type,
                game_number,
                media_player,
            )
            .await?
    } else {
        let schedule = session.fetch_games_by_date(&date).await?;

        println!("{:#?}", schedule)
    }

    Ok(())
}
