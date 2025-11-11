// Copyright (C) 2025 Tom Cole
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

mod api;
mod cli;
mod config;
mod data;
mod player;

use crate::api::session::MlbSession;
use crate::cli::Cli;
use crate::cli::args::CliMode;
use crate::cli::display;
use crate::config::AppConfig;
use crate::data::teamdata::Team;
use anyhow::Result;
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
    let mode = cli.to_mode()?;

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

    match mode {
        CliMode::Init => {
            AppConfig::generate_config()?;
        }
        CliMode::PlayStream {
            team,
            date,
            media_type,
            feed_type,
            game_number,
        } => {
            let team = Team::find_by_code(team).ok_or_else(|| anyhow::anyhow!("Invalid team code"))?;
            session
                .authorize(&cfg.credentials.username, &cfg.credentials.password)
                .await?
                .find_and_play_stream(
                    team,
                    date,
                    media_type,
                    feed_type,
                    game_number,
                    cfg.stream.video_player,
                )
                .await?;
        }
        CliMode::RangeSchedule {
            start_date,
            end_date,
        } => {
            if let Some(schedule) = session
                .fetch_schedule_by_range(&start_date, &end_date)
                .await?
            {
                let combined_table = display::combine_schedule_tables(schedule);
                println!("{}", combined_table);
            } else {
                println!("No games scheduled between {start_date} and {end_date}");
            }
        }
        CliMode::DaySchedule { date } => {
            if let Some(schedule) = session.fetch_schedule_by_date(&date).await? {
                let table = display::prepare_schedule_table(schedule);
                println!("{}", table)
            } else {
                // TODO: Detect when in off-season and add cute "see you next spring!" message.
                println!("No games scheduled for {date}");
            }
        }
    }

    Ok(())
}
