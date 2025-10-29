use clap::Parser;

use crate::streams::FeedType;
use crate::teamdata::TeamCode;

/// MLBV - Command-line utility for MLB.tv and stats API
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Team code (3-letter, e.g. wsh, nym, bos)
    #[arg(short, long)]
    pub team: Option<TeamCode>,

    /// Date to use (defaults to current date)
    #[arg(short, long)]
    pub date: Option<String>,

    /// Preferred feed to return (home, away, national)
    #[arg(short, long)]
    pub feed: Option<FeedType>,

    /// Return audio broadcasts only
    #[arg(short, long)]
    pub audio: bool,

    /// Specify game number 1 or 2 for double-headers
    #[arg(short, long)]
    pub game_number: Option<u8>,

    /// Increase verbosity (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
