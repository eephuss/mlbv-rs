use crate::{api::stats::schedule::DaySchedule, config::AppConfig, data::teamdata::Team};
use chrono::{DateTime, Local};
use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Style, Theme, Width,
        object::{Columns, Rows},
        style::HorizontalLine,
    },
};

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
    #[tabled(rename = "TV")]
    pub tv_feeds: String,
    #[tabled(rename = "Radio")]
    pub radio_feeds: String,
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

fn title_case_feed(feed: &str) -> String {
    match feed {
        "home" => "Home".to_string(),
        "away" => "Away".to_string(),
        other => other.to_string(),
    }
}

pub fn create_schedule_table(rows: Vec<GameRow>, header_date_str: &str) -> ScheduleTable {
    let table_theme = schedule_table_theme();

    let mut table = Table::new(&rows);
    table
        .with(table_theme)
        .modify((0, 0), header_date_str) // Replace matchup header with date + dow
        .modify(Columns::one(0), Alignment::left()) // Left-align times
        .modify(Columns::one(0), Width::increase(34)) // Set minimum width for matchup column
        .modify(Columns::one(0), Width::wrap(34).keep_words(true)) // Wrap to next line if too long
        .modify(Columns::one(1), Width::wrap(7)) // Series
        .modify(Columns::one(2), Width::wrap(7)) // Score
        .modify(Columns::one(3), Width::wrap(10).keep_words(true)) // State
        .modify(Columns::one(4), Width::wrap(12).keep_words(true)) // TV
        .modify(Columns::one(5), Width::wrap(12).keep_words(true)) // Radio
        .modify(Columns::one(6), Width::wrap(12).keep_words(true));

    ScheduleTable { table, rows }
}

pub fn color_favorite_teams(sched_table: ScheduleTable, config: &AppConfig) -> Table {
    let mut table = sched_table.table;

    if let Some(fav_teams) = &config.favorites.teams {
        let team_names = fav_teams
            .iter()
            .map(|&code| Team::find_by_code(code).name)
            .collect::<Vec<&'static str>>();
        for (idx, row) in sched_table.rows.iter().enumerate() {
            let row_num = idx + 1;
            if team_names.iter().any(|team| row.matchup.contains(team)) {
                table.modify(Rows::one(row_num), Color::FG_BLUE);
            }
        }
    }

    table
}

pub fn prepare_schedule_data(schedule: DaySchedule) -> (Vec<GameRow>, String) {
    let weekday = schedule.date.format("%A");
    let header_date = format!("{} {}", schedule.date, weekday);

    let mut rows = Vec::new();

    for game in &schedule.games {
        let game_time = DateTime::parse_from_rfc3339(&game.game_date)
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%I:%M%p")
                    .to_string()
                    .to_lowercase()
            })
            .unwrap_or_else(|_| "TBD".to_string());

        let away_team = Team::find_by_id(&game.teams.away.team.id).nickname;
        let home_team = Team::find_by_id(&game.teams.home.team.id).nickname;
        let matchup = format!("{game_time} - {away_team} at {home_team}");

        let games_in_series = &game.games_in_series;
        let series_game_number = &game.series_game_number;
        let series = format!("{series_game_number}/{games_in_series}");

        let score = if let Some(linescore) = &game.linescore {
            let away_score = &linescore.teams.away.runs.unwrap_or(0);
            let home_score = &linescore.teams.home.runs.unwrap_or(0);
            format!("{away_score}-{home_score}")
        } else {
            "".to_string()
        };

        let state = String::from(&game.status.abstract_game_state);

        let tv_feeds = if let Some(broadcasts) = &game.broadcasts {
            let mut feeds: Vec<String> = broadcasts
                .iter()
                .filter(|feed| feed.kind == "TV")
                .filter(|feed| feed.language == "en")
                .filter(|feed| feed.available_for_streaming)
                .map(|feed| match feed.is_national {
                    true => "National".to_string(),
                    false => title_case_feed(&feed.home_away),
                })
                .collect();
            feeds.sort();
            feeds.join(", ")
        } else {
            "".to_string()
        };

        let radio_feeds = if let Some(broadcasts) = &game.broadcasts {
            let mut feeds: Vec<String> = broadcasts
                .iter()
                .filter(|feed| feed.kind != "TV")
                .filter(|feed| feed.language == "en")
                .filter(|feed| feed.available_for_streaming)
                .map(|feed| title_case_feed(&feed.home_away))
                .collect();
            feeds.sort();
            feeds.join(", ")
        } else {
            "".to_string()
        };

        let highlights = if let Some(media) = &game.content.media
            && let Some(highlights) = &media.epg_alternate
        {
            let mut highlight_types: Vec<String> =
                highlights.iter().map(|h| h.title.to_string()).collect();
            highlight_types.sort();
            highlight_types.join(", ")
        } else {
            "".to_string()
        };

        rows.push(GameRow {
            matchup,
            series,
            score,
            state,
            tv_feeds,
            radio_feeds,
            highlights,
        });
    }
    (rows, header_date)
}
