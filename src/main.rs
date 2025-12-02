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
use crate::api::stats::schedule;
use crate::cli::Cli;
use crate::cli::args::CliMode;
use crate::cli::display::{self, DisplayMode};
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
    let display_mode = DisplayMode::from_terminal_width();

    let log_level = match cli.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_level(true)
        .compact()
        .init();

    let cfg = AppConfig::load()?;
    let session = MlbSession::new()?;
    let media_player = &cfg.stream.video_player;

    let scores = if cli.scores {
        true
    } else if cli.no_scores {
        false
    } else {
        cfg.display.scores
    };

    match mode {
        CliMode::Init => {
            AppConfig::generate_config()?;
        }
        CliMode::PlayStream {
            team_code,
            date,
            media_type,
            feed_type,
            game_number,
        } => {
            let team = team_code.team();
            if let Some(url) = session
                .authorize(&cfg.credentials.username, &cfg.credentials.password)
                .await?
                .find_stream_playback_url(team, date, media_type, feed_type, game_number)
                .await?
            {
                player::handle_playback_url(url, &cli, Some(media_player))?
            }
        }
        CliMode::PlayCondensedGame {
            team_code,
            date,
            game_number,
        } => {
            let team = team_code.team();
            let highlight_type = schedule::HighlightType::CondensedGame;
            if let Some(url) = session
                .find_highlight_playback_url(team, date, highlight_type, game_number, None)
                .await?
            {
                player::handle_playback_url(url, &cli, Some(media_player))?
            }
        }
        CliMode::PlayRecap {
            date,
            team_code,
            game_number,
            filter,
        } => {
            let highlight_type = schedule::HighlightType::Recap;
            let filter = filter.as_ref();

            // If user provided a team, fetch recap for that team
            if let Some(team_code) = team_code {
                let team = team_code.team();
                if let Some(url) = session
                    .find_highlight_playback_url(team, date, highlight_type, game_number, filter)
                    .await?
                {
                    player::handle_playback_url(url, &cli, Some(media_player))?
                }
            } else if let Some(schedule) = session.fetch_schedule_by_date(&date, filter).await? {
                // If no team provided, fetch recaps for all teams on specified day.
                let matchups: Vec<(String, String)> = schedule
                    .games
                    .into_iter()
                    .map(|g| (g.teams.away.team.name, g.teams.home.team.name))
                    .collect();

                println!("Found {} recap(s) for {date}:", matchups.len());
                for (away, home) in &matchups {
                    println!("    {} at {}", away, home);
                }
                for (away, home) in matchups {
                    println!("Playing: {} at {}", away, home);
                    let team = Team::find_by_name(&home)
                        .ok_or_else(|| anyhow::anyhow!("Invalid team name"))?;
                    if let Some(url) = session
                        .find_highlight_playback_url(
                            team,
                            date,
                            highlight_type,
                            game_number,
                            filter,
                        )
                        .await?
                    {
                        player::handle_playback_url(url, &cli, Some(media_player))?
                    }
                }
            }
        }
        CliMode::RangeSchedule {
            start_date,
            end_date,
            filter,
        } => {
            if let Some(schedules) = session
                .fetch_schedule_by_range(&start_date, &end_date, filter.as_ref())
                .await?
            {
                for (idx, schedule) in schedules.into_iter().enumerate() {
                    if idx > 0 {
                        println!(); // Blank line between days
                    }
                    let (rows, header_date) =
                        display::prepare_schedule_data(schedule, &display_mode, scores);
                    let table = display::create_schedule_table(rows, &header_date, &display_mode);
                    let color_table = display::color_favorite_teams(table, &cfg, &display_mode);
                    println!("{}", color_table);
                }
            } else {
                println!("No games scheduled between {start_date} and {end_date}");
            }
        }
        CliMode::DaySchedule { date, filter } => {
            if let Some(schedule) = session
                .fetch_schedule_by_date(&date, filter.as_ref())
                .await?
            {
                let (rows, header_date) =
                    display::prepare_schedule_data(schedule, &display_mode, scores);
                let table = display::create_schedule_table(rows, &header_date, &display_mode);
                let color_table = display::color_favorite_teams(table, &cfg, &display_mode);
                println!("{}", color_table)
            } else {
                // TODO: Detect when in off-season and add cute "see you next spring!" message.
                println!("No games scheduled for {date}");
            }
        }
    }

    Ok(())
}
