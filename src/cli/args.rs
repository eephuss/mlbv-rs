use clap::{ArgGroup, Parser};

use crate::api::mediagateway::streams::FeedType;
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

    /// Number of days to display. Use negative number to go back from today.
    #[arg(long, conflicts_with("team"))]
    pub days: Option<i64>,

    /// Preferred feed to return (home, away, national)
    #[arg(short, long)]
    pub feed: Option<FeedType>,

    /// Return audio broadcasts only
    #[arg(long)]
    pub audio: bool,

    /// Specify game number 1 or 2 for double-headers
    #[arg(short, long)]
    pub game_number: Option<u8>,

    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
