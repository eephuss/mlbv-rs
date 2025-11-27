use crate::{
    api::stats::schedule::{DaySchedule, GameData},
    config::AppConfig,
    data::teamdata::Team,
};
use chrono::{DateTime, Local};
use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Span, Style, Theme, Width,
        object::{Columns, Rows},
        style::HorizontalLine,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Standard,  // >= 97 chars
    Condensed, // >= 80 and < 97 chars
    Compact,   // < 80 chars
}

impl DisplayMode {
    /// Determine the appropriate display mode based on terminal width
    pub fn from_terminal_width() -> Self {
        let (width, _) = terminal_size::terminal_size()
            .map(|(w, h)| (w.0 as usize, h.0 as usize))
            .unwrap_or((80, 24)); // Default fallback to 80x24

        tracing::debug!("Terminal width: {}", width);

        if width >= 97 {
            Self::Standard
        } else if width >= 80 {
            Self::Condensed
        } else {
            Self::Compact
        }
    }
}

// Schedule display logic
#[derive(Tabled)]
pub struct GameRow {
    #[tabled(rename = "Matchup")]
    pub matchup: String,
    #[tabled(rename = "Series")]
    pub series: String,
    #[tabled(rename = "Score")]
    pub score: String,
    #[tabled(rename = "State")]
    pub state: String,
    #[tabled(rename = "Available Feeds")]
    pub feeds: String,
    #[tabled(rename = "Highlights")]
    pub highlights: String,
}

pub struct ScheduleTable {
    pub table: Table,
    pub rows: Vec<GameRow>,
}

fn schedule_table_theme() -> Theme {
    let style = Style::modern()
        .remove_horizontal() // Remove internal horizontal lines
        .horizontals([(1, HorizontalLine::inherit(Style::modern()))]) // Re-create just the header border
        .remove_frame();

    Theme::from_style(style)
}

fn compact_table_theme() -> Theme {
    let style = Style::modern()
        .remove_horizontal()
        .horizontals([(
            1,
            HorizontalLine::inherit(Style::modern()).intersection('┬'),
        )])
        .remove_frame();

    Theme::from_style(style)
}

fn title_case_feed(feed: &str) -> String {
    match feed {
        "home" => "Home".to_string(),
        "away" => "Away".to_string(),
        other => other.to_string(),
    }
}

pub fn create_schedule_table(
    rows: Vec<GameRow>,
    header_date_str: &str,
    display_mode: &DisplayMode,
) -> ScheduleTable {
    let table_theme = schedule_table_theme();
    let mut table = Table::new(&rows);

    table
        .with(table_theme)
        .modify((0, 0), header_date_str) // Replace matchup header with date + dow
        .modify(Columns::one(0), Alignment::left()); // Left-align times

    match display_mode {
        DisplayMode::Standard => {
            table
                .modify(Columns::one(0), Width::increase(33)) // Set minimum width for matchup column
                .modify(Columns::one(0), Width::wrap(33).keep_words(true)) // Wrap to next line if too long
                .modify(Columns::one(1), Width::wrap(6)) // Series
                .modify(Columns::one(2), Width::wrap(5)) // Score
                .modify(Columns::one(3), Width::wrap(10).keep_words(true)) // State
                .modify(Columns::one(4), Width::wrap(16)) // Feeds
                .modify(Columns::one(5), Width::wrap(10)); // Highlights
        }
        DisplayMode::Condensed => {
            table
                .modify(Columns::one(0), Width::wrap(17)) // Wrap to next line if too long
                .modify(Columns::one(1), Width::wrap(6)) // Series
                .modify(Columns::one(2), Width::wrap(5)) // Score
                .modify(Columns::one(3), Width::wrap(10).keep_words(true)) // State
                .modify(Columns::one(4), Width::wrap(16)) // Feeds
                .modify(Columns::one(5), Width::wrap(10)); // Highlights
        }
        DisplayMode::Compact => {
            let compact_theme = compact_table_theme();
            table
                .with(compact_theme)
                .modify(Rows::one(0), Span::column(6)) // Span header across all columns
                // .modify(Columns::one(0), Width::wrap(13)) // Wrap to next line if too long
                .modify(Columns::one(1), Width::wrap(3)) // Series
                .modify(Columns::one(2), Width::wrap(5)) // Score
                .modify(Columns::one(3), Width::wrap(3)) // State
                .modify(Columns::one(4), Width::wrap(7)) // Feeds
                .modify(Columns::one(5), Width::wrap(3)); // Highlights
        }
    }

    ScheduleTable { table, rows }
}

pub fn color_favorite_teams(sched_table: ScheduleTable, config: &AppConfig) -> Table {
    let mut table = sched_table.table;

    let fav_teams = &config.favorites.teams;
    let fav_color = &config.favorites.color;

    for (idx, row) in sched_table.rows.iter().enumerate() {
        let row_num = idx + 1;

        let matched_team = fav_teams.iter().find_map(|&code| {
            let team = Team::find_by_code(code);
            if row.matchup.contains(team.nickname) {
                Some(code)
            } else {
                None
            }
        });

        if let Some(team_code) = matched_team
            && let Some(color) = fav_color.to_tabled_color(Some(team_code))
        {
            table.modify(Rows::one(row_num), color);
        }
    }

    table
}

fn prepare_matchup(game: &GameData, display_mode: &DisplayMode) -> String {
    let away_team = Team::find_by_id(&game.teams.away.team.id);
    let home_team = Team::find_by_id(&game.teams.home.team.id);

    let time_format = match display_mode {
        DisplayMode::Compact => "%H:%M",
        _ => "%I:%M%p",
    };

    let game_time = DateTime::parse_from_rfc3339(&game.game_date)
        .map(|dt| {
            dt.with_timezone(&Local)
                .format(time_format)
                .to_string()
                .to_lowercase()
        })
        .unwrap_or_else(|_| "TBD".to_string());

    match display_mode {
        DisplayMode::Standard => {
            format!(
                "{} {} at {}",
                game_time, away_team.nickname, home_team.nickname
            )
        }
        DisplayMode::Condensed => {
            format!("{} {} @ {}", game_time, away_team.code, home_team.code)
        }
        DisplayMode::Compact => {
            format!("{} {}@{}", game_time, away_team.code, home_team.code)
        }
    }
}

fn prepare_series(game: &GameData) -> String {
    format!("{}/{}", &game.series_game_number, &game.games_in_series)
}

fn prepare_score(game: &GameData) -> String {
    let Some(linescore) = &game.linescore else {
        return String::new();
    };

    let away_score = &linescore.teams.away.runs.unwrap_or(0);
    let home_score = &linescore.teams.home.runs.unwrap_or(0);
    format!("{away_score}-{home_score}")
}

fn prepare_state(game: &GameData, display_mode: &DisplayMode) -> String {
    match display_mode {
        DisplayMode::Compact => game.status.status_code.clone(),
        _ => game.status.abstract_game_state.clone(),
    }
}

fn prepare_feeds(game: &GameData, display_mode: &DisplayMode) -> String {
    let Some(feeds) = &game.broadcasts else {
        return String::new();
    };

    let tv_feeds = feeds
        .iter()
        .filter(|feed| feed.kind == "TV")
        .filter(|feed| feed.language == "en")
        .filter(|feed| feed.available_for_streaming);

    let radio_feeds = feeds
        .iter()
        .filter(|feed| feed.kind != "TV")
        .filter(|feed| feed.language == "en")
        .filter(|feed| feed.available_for_streaming);

    let (tv_feeds, radio_feeds) = match display_mode {
        DisplayMode::Compact => {
            let mut tv_feed_types: Vec<String> = tv_feeds
                .map(|feed| match feed.is_national {
                    true => "Nat".to_string(),
                    false => feed
                        .home_away
                        .chars()
                        .next()
                        .expect("Couldn't get first character of home_away")
                        .to_string()
                        .to_uppercase(),
                })
                .collect();
            tv_feed_types.sort();
            let tv_feed_types = tv_feed_types.join(",");

            let mut radio_feed_types: Vec<String> = radio_feeds
                .map(|feed| {
                    feed.home_away
                        .chars()
                        .next()
                        .expect("Couldn't get first character of home_away")
                        .to_string()
                        .to_uppercase()
                })
                .collect();
            radio_feed_types.sort();
            let radio_feed_types = radio_feed_types.join(",");

            (tv_feed_types, radio_feed_types)
        }
        _ => {
            let mut tv_feed_types: Vec<String> = tv_feeds
                .map(|feed| match feed.is_national {
                    true => "National".to_string(),
                    false => title_case_feed(&feed.home_away),
                })
                .collect();
            tv_feed_types.sort();
            let tv_feed_types = tv_feed_types.join(", ");

            let mut radio_feed_types: Vec<String> = radio_feeds
                .map(|feed| title_case_feed(&feed.home_away))
                .collect();
            radio_feed_types.sort();
            let radio_feed_types = radio_feed_types.join(", ");

            (format!(" {tv_feed_types}"), format!(" {radio_feed_types}"))
        }
    };

    match (tv_feeds.is_empty(), radio_feeds.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("📺  {tv_feeds}"),
        (true, false) => format!("📻  {radio_feeds}"),
        (false, false) => {
            if tv_feeds == radio_feeds {
                format!("📺📻{tv_feeds}")
            } else {
                format!("📺  {tv_feeds}\n📻  {radio_feeds}")
            }
        }
    }
}

fn prepare_highlights(game: &GameData, display_mode: &DisplayMode) -> String {
    if let Some(media) = &game.content.media
        && let Some(highlights) = &media.epg_alternate
    {
        match display_mode {
            DisplayMode::Compact => {
                let mut highlight_types: Vec<String> = highlights
                    .iter()
                    .map(|h| {
                        h.title
                            .to_string()
                            .chars()
                            .next()
                            .expect("Highlight title should not be empty")
                            .to_string()
                    })
                    .collect();
                highlight_types.sort();
                highlight_types.join(",")
            }
            _ => {
                let mut highlight_types: Vec<String> =
                    highlights.iter().map(|h| h.title.to_string()).collect();
                highlight_types.sort();
                highlight_types.join(", ")
            }
        }
    } else {
        String::new()
    }
}

pub fn prepare_schedule_data(
    schedule: DaySchedule,
    display_mode: &DisplayMode,
) -> (Vec<GameRow>, String) {
    let weekday = schedule.date.format("%A");
    let header_date = format!("{} {}", schedule.date, weekday);

    let mut rows = Vec::new();

    for game in &schedule.games {
        let matchup = prepare_matchup(game, display_mode);
        let series = prepare_series(game);
        let score = prepare_score(game);
        let state = prepare_state(game, display_mode);
        let feeds = prepare_feeds(game, display_mode);
        let highlights = prepare_highlights(game, display_mode);

        rows.push(GameRow {
            matchup,
            series,
            score,
            state,
            feeds,
            highlights,
        });
    }
    (rows, header_date)
}
