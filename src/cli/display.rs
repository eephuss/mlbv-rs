use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns, style::HorizontalLine},
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

// TODO: Current formatting attempts to match original mlbv. How faithful do we want to be here?
pub fn format_schedule_table(game_rows: Vec<GameRow>, date_str: &str) -> tabled::Table {
    let table_style = Style::modern()
        .remove_horizontal() // Remove internal horizontal lines
        .horizontals([(1, HorizontalLine::inherit(Style::modern()))]) // Re-create just the header border
        .remove_frame(); // Remove the outline around the table

    let mut table = Table::new(game_rows);
    table
        .with(table_style)
        .modify((0, 0), date_str) // Replace matchup header with date + dow
        .modify(Columns::first(), Alignment::left()) // Left-align times
        .modify(Columns::one(3), Alignment::right()); // Right-align scores

    table
}
