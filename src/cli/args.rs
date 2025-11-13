use chrono::{Duration, Local};
use clap::{ArgGroup, Parser};

use crate::api::mediagateway::streams::{FeedType, MediaType};
use crate::api::stats::schedule::GameDate;
use crate::data::teamdata::TeamCode;

/// MLBV - Command-line utility for MLB.tv and stats API
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,)]
#[command(group(
    ArgGroup::new("date_group")
        .args(["date", "days", "tomorrow", "yesterday"])
        .multiple(false)
        .required(false)
))]
pub struct Cli {
    /// Re-initializes application config file
    #[arg(long)]
    pub init: bool,

    /// Team code (3-letter, e.g. wsh, nym, bos)
    #[arg(short, long)]
    pub team: Option<TeamCode>,

    /// Date to use (defaults to current date)
    #[arg(short, long)]
    pub date: Option<GameDate>,

    /// Shortcut: fetch tomorrow's games
    #[arg(long)]
    pub tomorrow: bool,

    /// Shortcut: fetch yesterday's games
    #[arg(long)]
    pub yesterday: bool,

    /// Number of days to display; use negative number to go back from today
    #[arg(long, conflicts_with("team"))]
    pub days: Option<i64>,

    /// Preferred feed to return (home, away, national)
    #[arg(short, long)]
    pub feed: Option<FeedType>,

    /// Play audio broadcasts only
    #[arg(long)]
    pub audio: bool,

    /// Play condensed game; user must specify a team
    #[arg(short, long, conflicts_with("recap"), requires("team"))]
    pub condensed: bool,

    /// Play daily recaps
    #[arg(short, long, conflicts_with("condensed"))]
    pub recap: bool,

    /// Specify game number 1 or 2 for double-headers
    #[arg(short, long)]
    pub game_number: Option<u8>,

    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub enum CliMode {
    Init,
    PlayStream {
        team_code: TeamCode,
        date: chrono::NaiveDate,
        media_type: MediaType,
        feed_type: Option<FeedType>,
        game_number: Option<u8>,
    },
    PlayCondensedGame {
        team_code: TeamCode,
        date: chrono::NaiveDate,
        game_number: Option<u8>,
    },
    PlayRecap {
        date: chrono::NaiveDate,
        team_code: Option<TeamCode>,
        game_number: Option<u8>,
        // filter: String,
    },
    RangeSchedule {
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    },
    DaySchedule {
        date: chrono::NaiveDate,
    },
}

impl Cli {
    pub fn to_mode(&self) -> anyhow::Result<CliMode> {
        if self.init {
            return Ok(CliMode::Init);
        }

        // Assume current date if None provided
        let today = Local::now().date_naive();
        let date = if let Some(game_date) = &self.date {
            game_date.0
        } else if self.yesterday {
            today - Duration::days(1)
        } else if self.tomorrow {
            today + Duration::days(1)
        } else {
            today
        };

        // Assume users want video broadcast unless they opt for Audio specifically
        let media_type = if self.audio {
            MediaType::Audio
        } else {
            MediaType::Video
        };

        if self.recap {
            return Ok(CliMode::PlayRecap {
                date,
                team_code: self.team,
                game_number: self.game_number,
            });
        }

        if let Some(team_code) = self.team {
            if self.condensed {
                return Ok(CliMode::PlayCondensedGame {
                    team_code,
                    date,
                    game_number: self.game_number,
                });
            }
            return Ok(CliMode::PlayStream {
                team_code,
                date,
                feed_type: self.feed,
                game_number: self.game_number,
                media_type,
            });
        }

        if let Some(days) = self.days {
            let offset_date = today + chrono::Duration::days(days);
            let (start_date, end_date) = (
                std::cmp::min(today, offset_date),
                std::cmp::max(today, offset_date),
            );
            return Ok(CliMode::RangeSchedule {
                start_date,
                end_date,
            });
        }

        Ok(CliMode::DaySchedule { date })
    }
}
