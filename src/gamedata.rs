#![allow(dead_code)] // Shush unused refernce warnings until I know what fields are needed

use crate::session::MlbSession;
use crate::teamdata::Team;
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use serde::Deserialize;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns, style::HorizontalLine},
};

#[derive(Debug, Deserialize)]
struct ScheduleResponse {
    dates: Vec<DaySchedule>,
    #[serde(rename = "totalGames")]
    total_games: u32,
}

#[derive(Clone, Debug)]
pub struct GameDate(pub NaiveDate);

impl std::str::FromStr for GameDate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse a few common date formats before giving up.
        let formats = ["%Y-%m-%d", "%m-%d-%Y", "%m/%d/%Y"];
        for fmt in formats {
            if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
                if date.year() < 2022 {
                    anyhow::bail!("MLB.tv archives only go back to the start of 2022.");
                }
                return Ok(GameDate(date));
            }
        }
        anyhow::bail!("Invalid date format: '{s}'; expected YYYY-MM-DD");
    }
}

#[derive(Debug, Deserialize)]
pub struct DaySchedule {
    date: NaiveDate,
    games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_pk: u64,
    game_guid: String,
    link: String,
    game_type: String,
    season: String,
    game_date: String,
    official_date: String,
    status: GameStatus,
    pub teams: Matchup,
    pub linescore: Linescore,
    venue: GameVenue,
    broadcasts: Vec<Broadcast>,
    pub content: Content,
    is_tie: Option<bool>,
    game_number: u8,
    public_facing: bool,
    double_header: String,
    gameday_type: String,
    tiebreaker: String,
    #[serde(rename = "calendarEventID")]
    calendar_event_id: String,
    season_display: String,
    day_night: String,
    description: Option<String>,
    scheduled_innings: u8,
    reverse_home_away_status: bool,
    inning_break_length: Option<u8>,
    games_in_series: u8,
    series_game_number: u8,
    series_description: String,
    record_source: String,
    if_necessary: String,
    if_necessary_description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameStatus {
    abstract_game_state: String,
    coded_game_state: String,
    detailed_state: String,
    status_code: String,
    #[serde(rename = "startTimeTBD")]
    start_time_tbd: bool,
    abstract_game_code: String,
}

#[derive(Debug, Deserialize)]
struct GameVenue {
    id: u32,
    name: String,
    link: String,
}

#[derive(Debug, Deserialize)]
pub struct Matchup {
    pub home: GameTeamStats,
    pub away: GameTeamStats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTeamStats {
    pub score: Option<u8>,
    pub team: GameTeam,
    pub is_winner: Option<bool>,
    pub split_squad: bool,
    pub series_number: u8,
}

#[derive(Debug, Deserialize)]
pub struct GameTeam {
    pub id: u32,
    pub name: String,
    pub link: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub current_inning: Option<u8>,
    pub current_inning_ordinal: Option<String>,
    pub inning_state: Option<String>,
    pub inning_half: Option<String>,
    pub is_top_inning: Option<bool>,
    pub scheduled_innings: u8,
    // pub innings: Inning,
    pub teams: ScoreTeams,
    // pub defense: Defense,
    // pub offense: Offense,
    pub balls: Option<u8>,
    pub strikes: Option<u8>,
    pub outs: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct ScoreTeams {
    home: Score,
    away: Score,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    runs: Option<u8>,
    hits: Option<u8>,
    errors: Option<u8>,
    left_on_base: Option<u8>,
    is_winner: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Broadcast {
    id: u32,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    language: String,
    is_national: bool,
    call_sign: String,
    video_resolution: Option<BroadcastResolution>,
    availability: BroadcastAvailability,
    media_state: BroadcastMediaState,
    color_space: Option<BroadcastColorSpace>,
    broadcast_date: String,
    media_id: String,
    game_date_broadcast_guid: String,
    home_away: String,
    free_game: bool,
    available_for_streaming: bool,
    post_game_show: bool,
    mvpd_auth_required: bool,
    free_game_status: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastResolution {
    code: String,
    resolution_short: String,
    resolution_full: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastAvailability {
    availability_id: u8,
    availability_code: String,
    availability_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastMediaState {
    media_state_id: u8,
    media_state_code: String,
    media_state_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastColorSpace {
    code: String,
    color_space_full: String,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub link: String,
    pub media: Media,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    epg_alternate: Option<Vec<MediaDetails>>,
    free_game: bool,
    enhanced_game: bool,
}

#[derive(Debug, Deserialize)]
struct MediaDetails {
    items: Vec<MediaInstance>,
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaInstance {
    #[serde(rename = "type")]
    kind: String,
    state: String,
    date: String,
    id: String,
    headline: String,
    seo_title: String,
    slug: String,
    blurb: String,
    no_index: bool,
    media_playback_id: String,
    title: String,
    description: String,
    duration: String,
    media_playback_url: String,
    playbacks: Vec<Playback>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Playback {
    name: String,
    url: String,
    width: String,
    height: String,
}

#[derive(Tabled)]
struct GameRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Matchup")]
    matchup: String,
    #[tabled(rename = "Series")]
    series: String,
    #[tabled(rename = "Score")]
    score: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Feeds")]
    feeds: String,
}

// TODO: Update to use start and end date logic.
pub async fn fetch_games_by_date<State>(
    session: &MlbSession<State>,
    date: &NaiveDate,
) -> Result<Option<DaySchedule>> {
    let hydrate = concat!(
        "hydrate=,",
        "broadcasts(all),",
        "game(content(media(epg)),",
        "editorial(preview,recap)),",
        "linescore,",
        "team,",
        "probablePitcher(note)",
    );
    let url = format!(
        "https://statsapi.mlb.com/api/v1/schedule?sportId=1&startDate={d}&endDate={d}&{h}",
        d = date,
        h = hydrate
    );

    let res = session
        .client
        .get(url)
        .header("Connection", "close")
        .send()
        .await
        .context("Failed to send schedule request")?
        .error_for_status()
        .context("Schedule fetch returned unsuccessful status")?;

    let body: ScheduleResponse = res
        .json()
        .await
        .context("Failed to parse schedule response")?;

    match body.dates.len() {
        0 => {
            tracing::info!("Schedule returned no games for {date}.");
            Ok(None) // If no games are scheduled, then no dates are returned.
        }
        1 => Ok(Some(body.dates.into_iter().next().unwrap())),
        n => anyhow::bail!("Expected 1 date but got {n} - possible API change."),
    }
}

impl<State> MlbSession<State> {
    pub async fn fetch_games_by_date(&self, date: &NaiveDate) -> Result<Option<DaySchedule>> {
        fetch_games_by_date(self, date).await
    }
}

impl DaySchedule {
    pub fn find_team_games(self, team: &Team) -> Option<Vec<GameData>> {
        let team_games: Vec<GameData> = self
            .games
            .into_iter()
            .filter(|game| {
                let home = &game.teams.home.team.name;
                let away = &game.teams.away.team.name;
                home == team.name || away == team.name
            })
            .collect();

        match team_games.len() {
            0 => None,
            _ => Some(team_games), // Your team has a game or doubleheader today.
        }
    }

    pub fn display_game_data(&self) {
        let weekday = self.date.format("%A");
        let header_date = format!("{} {}", self.date, weekday);

        let mut rows = Vec::new();

        for game in &self.games {
            let time = DateTime::parse_from_rfc3339(&game.game_date)
                .map(|dt| {
                    dt.with_timezone(&Local)
                        .format("%I:%M %p")
                        .to_string()
                        .to_lowercase()
                })
                .unwrap_or_else(|_| "TBD".to_string());

            let away_team = &game.teams.away.team.name;
            let home_team = &game.teams.home.team.name;
            let matchup = format!("{} at {}", away_team, home_team);

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

            rows.push(GameRow {
                time,
                matchup,
                series,
                score,
                state,
                feeds,
            });
        }

        let table_style = Style::modern()
            .horizontals([(1, HorizontalLine::inherit(Style::modern()))])
            .remove_horizontal()
            .remove_frame();

        let mut table = Table::new(rows);
        table
            .with(table_style)
            // .modify(Columns::first(), Alignment::right())
            .modify(Columns::one(4), Alignment::right());

        println!("{header_date}");
        println!("{}", table);
    }
}

pub fn select_game(team_games: Vec<GameData>, game_number: Option<u8>) -> Result<GameData> {
    match team_games.len() {
        0 => anyhow::bail!("Got empty vector where at least 1 game was expected. Aborting."),
        1 => Ok(team_games.into_iter().next().unwrap()), // Not much to do if there's only 1 game that day.
        _ => {
            // If a valid game_number is specified, return that game.
            if let Some(n) = game_number {
                if [1, 2].contains(&n) {
                    tracing::debug!("User requested game number {n}");
                    Ok(team_games.into_iter().nth((n - 1) as usize).unwrap())
                } else {
                    tracing::warn!("User provided invalid game number: {n}");
                    anyhow::bail!("Invalid game number")
                }
            } else {
                tracing::debug!("Doubleheader deteced but no game number specified");

                // If no game number provided, prefer live game. If no live games, return game 1.
                // TODO: This assumes that vector the vector is ordered chronologically.
                //       May not be true. Can use game_number attribute of GameData if needed.
                let mut iter = team_games.into_iter();
                let game_one = iter.next().unwrap();
                let game_two = iter.next().unwrap();

                if game_two.status.abstract_game_state == "Live" {
                    tracing::info!("Game 2 is currently live; defaulting to live broadcast");
                    Ok(game_two)
                } else {
                    tracing::info!("Defaulting to Game 1");
                    Ok(game_one)
                }
            }
        }
    }
}
