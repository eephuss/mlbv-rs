mod api;
mod cli;
mod config;
mod data;
mod player;

use crate::{api::mediagateway::streams::MediaType, cli::display::combine_schedule_tables};
use crate::api::session::MlbSession;
use crate::cli::Cli;
use crate::config::AppConfig;
use crate::data::teamdata::Team;
use anyhow::Result;
use chrono::{Duration, Local};
use clap::Parser;
use std::cmp;

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
    let today = Local::now().date_naive();

    // Use user-provided date. If none, return today's date
    let date = if let Some(game_date) = cli.date {
        game_date.0 // 0 extracts NaiveDate from GameDate
    } else if cli.yesterday {
        today - Duration::days(1)
    } else if cli.tomorrow {
        today + Duration::days(1)
    } else {
        today
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
    } else if let Some(days) = cli.days {
        let offset_date = today + Duration::days(days);
        let (start_date, end_date) = (cmp::min(today, offset_date), cmp::max(today, offset_date));

        if let Some(schedule) = session.fetch_schedule_by_range(&start_date, &end_date).await? {
            let combined_table = combine_schedule_tables(schedule);
            println!("{}", combined_table);
        } else {
            println!("No games scheduled between {start_date} and {end_date}")
        }
    } else if let Some(schedule) = session.fetch_schedule_by_date(&date).await? {
        let table = schedule.prepare_schedule_table();
        println!("{}", table)
    } else {
        // TODO: Detect when in off-season and add cute "see you next spring!" message.
        println!("No games scheduled for {date}");
    }

    Ok(())
}
