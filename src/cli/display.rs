use crate::api::stats::schedule::DaySchedule;
use chrono::{DateTime, Local};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Concat, Style, Theme, object::Columns, style::HorizontalLine},
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
    #[tabled(rename = "Feeds")]
    pub feeds: String,
    #[tabled(rename = "Highlights")]
    pub highlights: String,
}

fn schedule_table_theme() -> Theme {
    let style = Style::modern()
        .remove_horizontal() // Remove internal horizontal lines
        .horizontals([(1, HorizontalLine::inherit(Style::modern()))]) // Re-create just the header border
        .remove_frame();

    Theme::from_style(style)
}

pub fn format_schedule_table(game_rows: Vec<GameRow>, date_str: &str) -> tabled::Table {
    let table_theme = schedule_table_theme();

    let mut table = Table::new(game_rows);
    table
        .with(table_theme)
        .modify((0, 0), date_str) // Replace matchup header with date + dow
        .modify(Columns::first(), Alignment::left()) // Left-align times
        .modify(Columns::one(3), Alignment::right()); // Right-align scores

    table
}

pub fn prepare_schedule_table(schedule: DaySchedule) -> Table {
    let weekday = schedule.date.format("%A");
    let header_date = format!("{} {}", schedule.date, weekday);

    let mut rows = Vec::new();

    for game in &schedule.games {
        let game_time = DateTime::parse_from_rfc3339(&game.game_date)
            .map(|dt| {
                dt.with_timezone(&Local)
                    .format("%I:%M %p")
                    .to_string()
                    .to_lowercase()
            })
            .unwrap_or_else(|_| "TBD".to_string());

        let away_team = &game.teams.away.team.name;
        let home_team = &game.teams.home.team.name;
        let matchup = format!("{game_time} - {away_team} at {home_team}");

        let games_in_series = &game.games_in_series;
        let series_game_number = &game.series_game_number;
        let series = format!("{series_game_number}/{games_in_series}");

        let away_score = &game.linescore.teams.away.runs.unwrap_or(0);
        let home_score = &game.linescore.teams.home.runs.unwrap_or(0);
        let score = format!("{away_score}-{home_score}");

        let state = String::from(&game.status.abstract_game_state);

        let feeds = {
            let mut feeds: Vec<&str> = game
                .broadcasts
                .iter()
                .filter(|feed| feed.kind == "TV")
                .filter(|feed| feed.available_for_streaming)
                .map(|feed| match feed.is_national {
                    true => "national",
                    false => &feed.home_away,
                })
                .collect();
            feeds.sort();
            feeds.join(", ")
        };

        let highlights = if let Some(highlights) = &game.content.media.epg_alternate {
            let mut highlight_types: Vec<String> = highlights
                .iter()
                .map(|h| h.title.to_string())
                .collect();

            highlight_types.sort();
            highlight_types.join(", ")
        } else {
            "None".to_string()
        };

        rows.push(GameRow {
            matchup,
            series,
            score,
            state,
            feeds,
            highlights,
        });
    }
    format_schedule_table(rows, &header_date)
}

pub fn combine_schedule_tables(schedule: Vec<DaySchedule>) -> tabled::Table {
    let mut tables = Vec::new();
    let mut offsets = Vec::new();
    let mut total_rows = 0;

    for day in schedule {
        let table = prepare_schedule_table(day);
        let rows = table.count_rows();
        tables.push(table);
        total_rows += rows;
        offsets.push(total_rows); // cumulative row index where next table starts
    }

    let mut combined_table: tabled::Table = tables
        .into_iter()
        .reduce(|mut acc, t| {
            acc.with(Concat::vertical(t));
            acc
        })
        .expect("Tables failed to combine");

    let empty_line = tabled::grid::config::HorizontalLine::empty();
    let header_line = HorizontalLine::inherit(Style::modern().remove_frame());

    let mut theme = schedule_table_theme();
    for &row in &offsets {
        theme.insert_horizontal_line(row, empty_line);
        theme.insert_horizontal_line(row + 1, header_line);
    }
    combined_table.with(theme);

    combined_table
}
