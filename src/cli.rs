use clap::Parser;

use crate::api::stats::gamedata::GameDate;
use crate::api::mediagateway::streams::FeedType;
use crate::data::teamdata::TeamCode;

/// MLBV - Command-line utility for MLB.tv and stats API
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Team code (3-letter, e.g. wsh, nym, bos)
    #[arg(short, long)]
    pub team: Option<TeamCode>,

    /// Date to use (defaults to current date)
    #[arg(short, long, conflicts_with = "yesterday", conflicts_with = "tomorrow")]
    pub date: Option<GameDate>,

    /// Shortcut: fetch tomorrow's games
    #[arg(long, conflicts_with = "date", conflicts_with = "yesterday")]
    pub tomorrow: bool,

    /// Shortcut: fetch yesterday's games
    #[arg(long, conflicts_with = "date", conflicts_with = "tomorrow")]
    pub yesterday: bool,

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
