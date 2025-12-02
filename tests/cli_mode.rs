use chrono::Local;
use clap::Parser;
use mlbv_rs::cli::args::{Cli, CliMode};
use mlbv_rs::data::teamdata::TeamCode;

#[test]
fn parses_init_mode() {
    let cli = Cli::parse_from(["mlbv-rs", "--init"]);
    let mode = cli.to_mode().expect("to_mode failed");
    assert!(matches!(mode, CliMode::Init));
}

#[test]
fn parses_play_stream_with_team() {
    let cli = Cli::parse_from(["mlbv-rs", "--team", "wsh"]);
    let mode = cli.to_mode().expect("to_mode failed");
    match mode {
        CliMode::PlayStream { team_code, .. } => {
            assert_eq!(team_code, TeamCode::Wsh);
        }
        _ => panic!("Expected PlayStream mode"),
    }
}

#[test]
fn parses_range_schedule_mode() {
    let cli = Cli::parse_from(["mlbv-rs", "--days", "3"]);
    let mode = cli.to_mode().expect("to_mode failed");
    assert!(matches!(mode, CliMode::RangeSchedule { .. }));
}

#[test]
fn parses_day_schedule_default() {
    let cli = Cli::parse_from(["mlbv-rs"]);
    let mode = cli.to_mode().expect("to_mode failed");
    match mode {
        CliMode::DaySchedule { date, filter } => {
            let today = Local::now().date_naive();
            let _filter = filter;
            assert_eq!(date, today);
        }
        _ => panic!("Expected DaySchedule mode"),
    }
}

#[test]
fn parses_play_recap_with_team() {
    let cli = Cli::parse_from(["mlbv-rs", "--recap", "--team", "tor"]);
    let mode = cli.to_mode().expect("to_mode failed");
    match mode {
        CliMode::PlayRecap {
            team_code: Some(team),
            ..
        } => {
            assert_eq!(team, TeamCode::Tor);
        }
        _ => panic!("Expected PlayRecap mode with team"),
    }
}

#[test]
fn parses_play_recap_all_games() {
    let cli = Cli::parse_from(["mlbv-rs", "--recap"]);
    let mode = cli.to_mode().expect("to_mode failed");
    match mode {
        CliMode::PlayRecap {
            team_code: None, ..
        } => {}
        _ => panic!("Expected PlayRecap mode without team"),
    }
}

#[test]
fn parses_condensed_game() {
    let cli = Cli::parse_from(["mlbv-rs", "--condensed", "--team", "nyy"]);
    let mode = cli.to_mode().expect("to_mode failed");
    match mode {
        CliMode::PlayCondensedGame { team_code, .. } => {
            assert_eq!(team_code, TeamCode::Nyy);
        }
        _ => panic!("Expected PlayCondensedGame mode"),
    }
}

#[test]
fn date_mutually_exclusive() {
    // Test that --date and --yesterday are mutually exclusive
    let result = Cli::try_parse_from(["mlbv-rs", "--date", "2024-10-01", "--yesterday"]);
    assert!(
        result.is_err(),
        "Should reject mutually exclusive date args"
    );
}
