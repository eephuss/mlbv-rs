use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns, style::HorizontalLine},
};

// Schedule display logic
#[derive(Tabled)]
pub struct GameRow {
    #[tabled(rename = "Time")]
    pub time: String,
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

pub fn format_schedule_table(game_rows: Vec<GameRow>) -> tabled::Table {
    let table_style = Style::modern()
        .horizontals([(1, HorizontalLine::inherit(Style::modern()))])
        .remove_horizontal()
        .remove_frame();

    let mut table = Table::new(game_rows);
    table
        .with(table_style)
        .modify(Columns::first(), Alignment::right())
        .modify(Columns::one(4), Alignment::right());

    table
}