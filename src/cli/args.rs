use chrono::{Duration, Local};
use clap::{ArgGroup, Parser};

use crate::api::mediagateway::streams::{FeedType, MediaType};
use crate::api::stats::schedule::GameDate;
use crate::data::teamdata::TeamCode;

/// Stream live/archived MLB.tv games; view stats, schedules and highlights
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    long_about = "mlbv is a command-line interface to the MLB.tv service and MLB Stats API.\n\n\
                  Stream live and archived games with your MLB.tv subscription.\n\
                  View stats, check schedules and watch highlights for free.\n\n\
                  EXAMPLES:\n  \
                  mlbv-rs --team wsh                     # Watch today's Nationals game\n  \
                  mlbv-rs --team bos --yesterday         # Yesterday's Red Sox game\n  \
                  mlbv-rs --team nym --condensed         # Condensed game for Mets\n  \
                  mlbv-rs --recap --yesterday            # Play all recaps from yesterday\n  \
                  mlbv-rs --days 7                       # Show schedule for next 7 days\n  \
                  mlbv-rs --date 2024-10-01 --team lad   # Dodgers game on specific date"
)]
#[command(group(
    ArgGroup::new("date_group")
        .args(["date", "days", "tomorrow", "yesterday"])
        .multiple(false)
        .required(false)
))]
pub struct Cli {
    /// Re-initialize config file (prompts for MLB.tv credentials)
    #[arg(long)]
    pub init: bool,

    /// Team code to watch or filter by (e.g., wsh, cle, bos, lad)
    #[arg(short, long)]
    pub team: Option<TeamCode>,

    /// Specific date; defaults to today if none provided
    #[arg(
        short,
        long,
        long_help = "Specify a date for games. Accepts multiple formats:\n  \
                     YYYY-MM-DD (e.g., 2024-10-01)\n  \
                     MM-DD-YYYY (e.g., 10-01-2024)\n  \
                     MM/DD/YYYY (e.g., 10/01/2024)\n\
                     Defaults to today if not specified."
    )]
    pub date: Option<GameDate>,

    /// Shortcut for tomorrow's date
    #[arg(long)]
    pub tomorrow: bool,

    /// Shortcut for yesterday's date
    #[arg(long)]
    pub yesterday: bool,

    /// Display schedule for N days from today; accepts negative values
    #[arg(
        long,
        conflicts_with("team"),
        long_help = "Show schedule for multiple days:\n  \
                     Positive: future days (--days 7 shows next week)\n  \
                     Negative: past days (--days -3 shows last 3 days)\n\
                     Cannot be used with --team."
    )]
    pub days: Option<i64>,

    /// Select feed type: home, away, or national
    #[arg(short, long)]
    pub feed: Option<FeedType>,

    /// Select audio-only broadcast
    #[arg(long)]
    pub audio: bool,

    /// Play condensed game (~10 min)
    #[arg(
        short,
        long,
        conflicts_with("recap"),
        requires("team"),
        long_help = "Replay of the game condensed down to ~10 min.\n\
                     Requires --team to be specified.\n\
                     Cannot be combined with --recap."
    )]
    pub condensed: bool,

    /// Play game recap(s) (~3-5 min)
    #[arg(
        short,
        long,
        conflicts_with("condensed"),
        long_help = "Key plays and highlights from each game, ~3-5 min.\n\
                     Without --team: plays all recaps for the day\n\
                     With --team: plays recap for specified team only\n\
                     Cannot be combined with --condensed."
    )]
    pub recap: bool,

    /// Game number for doubleheaders (1 or 2)
    #[arg(
        short,
        long,
        long_help = "Specify which game to watch during doubleheaders.\n\
                     Valid values: 1 or 2\n\
                     If not specified, live game is preferred, otherwise defaults to game 1."
    )]
    pub game_number: Option<u8>,

    /// Print stream URL without launching player
    #[arg(short, long)]
    pub url: bool,

    /// Show scores (overrides config file setting)
    #[arg(long, conflicts_with = "no_scores")]
    pub scores: bool,

    /// Hide scores (overrides config file setting)
    #[arg(long, conflicts_with = "scores")]
    pub no_scores: bool,

    /// Verbose logging (-v, -vv, -vvv for more detail)
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        long_help = "Control logging level:\n  \
                     (none): Warnings only\n  \
                     -v:     Info messages\n  \
                     -vv:    Debug messages\n  \
                     -vvv:   Trace (very detailed)"
    )]
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
