use crate::api::stats::schedule::DaySchedule;
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

pub fn combine_schedule_tables(schedule: Vec<DaySchedule>) -> tabled::Table {
    let mut tables = Vec::new();
    let mut offsets = Vec::new();
    let mut total_rows = 0;

    for day in schedule {
        let table = day.prepare_schedule_table();
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
