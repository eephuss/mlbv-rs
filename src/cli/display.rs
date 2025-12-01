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
    let compact_theme = compact_table_theme();
    let mut table = Table::new(&rows);

    table
        .modify((0, 0), header_date_str) // Replace matchup header with date + dow
        .modify(Columns::one(0), Alignment::left()) // Left-align times
        .modify(Columns::one(1), Alignment::center()) // Center-align series
        .modify(Columns::one(2), Alignment::center()); // Center-align scores

    match display_mode {
        DisplayMode::Standard => {
            table
                .with(table_theme)
                .modify(Columns::one(0), Width::increase(33)) // Set minimum width for matchup column
                .modify(Columns::one(0), Width::wrap(33).keep_words(true)) // Wrap to next line if too long
                .modify(Columns::one(1), Width::increase(6)) // Series
                .modify(Columns::one(2), Width::increase(5)) // Score
                .modify(Columns::one(3), Width::wrap(10).keep_words(true)) // State
                .modify(Columns::one(4), Width::wrap(16)) // Feeds
                .modify(Columns::one(5), Width::wrap(10)); // Highlights
        }
        DisplayMode::Condensed => {
            table
                .with(table_theme)
                .modify(Columns::one(0), Width::wrap(17)) // Wrap to next line if too long
                .modify(Columns::one(1), Width::increase(6)) // Series
                .modify(Columns::one(2), Width::increase(5)) // Score
                .modify(Columns::one(3), Width::wrap(10).keep_words(true)) // State
                .modify(Columns::one(4), Width::wrap(16)) // Feeds
                .modify(Columns::one(5), Width::wrap(10)); // Highlights
        }
        DisplayMode::Compact => {
            table
                .with(compact_theme)
                .modify(Rows::one(0), Span::column(6)) // Span header across all columns
                .modify(Columns::one(1), Width::increase(3)) // Series
                .modify(Columns::one(2), Width::increase(5)) // Score
                .modify(Columns::one(3), Width::wrap(3)) // State
                .modify(Columns::one(4), Width::wrap(7)) // Feeds
                .modify(Columns::one(5), Width::wrap(3)); // Highlights
        }
    }

    ScheduleTable { table, rows }
}

pub fn color_favorite_teams(
    sched_table: ScheduleTable,
    config: &AppConfig,
    display_mode: &DisplayMode,
) -> Table {
    let mut table = sched_table.table;
    let is_standard = display_mode == &DisplayMode::Standard;
    let fav_teams = &config.favorites.teams;
    let fav_color = &config.favorites.color;

    for (idx, row) in sched_table.rows.iter().enumerate() {
        let row_num = idx + 1;

        let matched_team = fav_teams.iter().find_map(|&code| {
            let team = Team::find_by_code(code);
            let code_str = code.to_string();

            if (is_standard && row.matchup.contains(team.nickname))
                || (!is_standard && row.matchup.contains(&code_str))
            {
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

fn prepare_score(game: &GameData, scores: bool) -> String {
    if !scores {
        return String::new();
    };

    let Some(linescore) = &game.linescore else {
        return String::new();
    };

    let away_score = &linescore.teams.away.runs.unwrap_or(0);
    let home_score = &linescore.teams.home.runs.unwrap_or(0);
    format!("{away_score}-{home_score}")
}

fn prepare_state(game: &GameData, display_mode: &DisplayMode) -> String {
    let status_code = &game.status.status_code;
    let coded_game_state = &game.status.coded_game_state;
    let abstract_state = &game.status.abstract_game_state;
    let detailed_state = &game.status.detailed_state;

    let is_compact = matches!(display_mode, DisplayMode::Compact);
    let is_in_progress = status_code == "I" || matches!(coded_game_state.as_str(), "M" | "N");

    if is_in_progress {
        if let Some(linescore) = &game.linescore
            && let Some(inning) = linescore.current_inning
            && let Some(is_top) = linescore.is_top_inning
        {
            let half_str = match (is_compact, is_top) {
                (true, true) => "T",
                (true, false) => "B",
                (false, true) => "Top",
                (false, false) => "Bottom",
            };
            return format!("{} {}", half_str, inning);
        }
        return if is_compact {
            status_code.clone()
        } else {
            abstract_state.clone()
        };
    }

    // Handle games that completed in anything other than 9 innings
    // Does not include suspended, canceled, or postponed games
    if matches!(coded_game_state.as_str(), "F" | "O")
        && let Some(linescore) = &game.linescore
        && let Some(inning) = linescore.current_inning
        && inning != 9
    {
        return if is_compact {
            format!("F{}", inning)
        } else if let Some(reason) = &game.status.reason {
            format!("Final ({}):\n{}", inning, reason)
        } else {
            format!("Final ({})", inning)
        };
    }

    // Default: use status_code for compact, detailed_state otherwise
    if is_compact {
        status_code.clone()
    } else {
        detailed_state.clone()
    }
}

fn prepare_feeds(game: &GameData, display_mode: &DisplayMode) -> String {
    let Some(feeds) = &game.broadcasts else {
        return String::new();
    };

    let is_compact = matches!(display_mode, DisplayMode::Compact);
    let (separator, spacing) = match is_compact {
        true => (",", ""),
        false => (", ", " "),
    };

    let format_feed = |feed: &str, is_national: bool| -> String {
        match (is_compact, is_national) {
            (true, true) => "Nat".to_string(),
            (true, false) => feed
                .chars()
                .next()
                .unwrap_or('?')
                .to_string()
                .to_uppercase(),
            (false, true) => "National".to_string(),
            (false, false) => title_case_feed(feed),
        }
    };

    let mut tv_feeds: Vec<String> = feeds
        .iter()
        .filter(|f| f.kind == "TV" && f.language == "en" && f.available_for_streaming)
        .map(|f| format_feed(&f.home_away, f.is_national))
        .collect();
    tv_feeds.sort();
    let tv_feeds = tv_feeds.join(separator);

    let mut radio_feeds: Vec<String> = feeds
        .iter()
        .filter(|f| f.kind != "TV" && f.language == "en" && f.available_for_streaming)
        .map(|f| format_feed(&f.home_away, false))
        .collect();
    radio_feeds.sort();
    let radio_feeds = radio_feeds.join(separator);

    match (tv_feeds.is_empty(), radio_feeds.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("📺 {spacing}{tv_feeds}"),
        (true, false) => format!("📻 {spacing}{radio_feeds}"),
        (false, false) => {
            if tv_feeds == radio_feeds {
                format!("📺📻{spacing}{tv_feeds}")
            } else {
                format!("📺  {spacing}{tv_feeds}\n📻  {spacing}{radio_feeds}")
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
    scores: bool,
) -> (Vec<GameRow>, String) {
    let weekday = schedule.date.format("%A");
    let header_date = format!("{} {}", schedule.date, weekday);

    let rows: Vec<GameRow> = schedule
        .games
        .iter()
        .map(|game| GameRow {
            matchup: prepare_matchup(game, display_mode),
            series: prepare_series(game),
            score: prepare_score(game, scores),
            state: prepare_state(game, display_mode),
            feeds: prepare_feeds(game, display_mode),
            highlights: prepare_highlights(game, display_mode),
        })
        .collect();

    (rows, header_date)
}
