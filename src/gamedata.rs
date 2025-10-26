#![allow(dead_code)] // Shush unused refernce warnings until I know what fields are needed

use crate::session::MlbSession;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ScheduleResponse {
    dates: Vec<DaySchedule>,
    #[serde(rename = "totalGames")]
    total_games: u32,
}

#[derive(Debug, Deserialize)]
pub struct DaySchedule {
    date: String,
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
    inning_break_length: u8,
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
    pub home: TeamStats,
    pub away: TeamStats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamStats {
    pub score: Option<u8>,
    pub team: Team,
    pub is_winner: Option<bool>,
    pub split_squad: bool,
    pub series_number: u8,
}

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: u8,
    pub name: String,
    pub link: String,
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

// TODO: Update to use start and end date logic.
pub async fn get_games_by_date<State>(
    session: &MlbSession<State>,
    date: &str,
) -> Result<Option<DaySchedule>> {
    // let date = match date_opt {
    //     Some(s) => s.to_string(),
    //     None => Utc::now().format("%Y-%m-%d").to_string(),
    // };
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
        .await?;

    let body: ScheduleResponse = res.json().await?;

    match body.dates.len() {
        0 => Ok(None), // If no games are scheduled, then no dates are returned.
        1 => Ok(Some(body.dates.into_iter().next().unwrap())),
        n => anyhow::bail!("Expected 1 date but got {n} - possible API change."),
    }
}

impl<State> MlbSession<State> {
    pub async fn get_games_by_date(&self, date: &str) -> Result<Option<DaySchedule>> {
        get_games_by_date(self, date).await
    }
}

pub fn find_team_games(schedule: DaySchedule, team: &str) -> Result<Option<Vec<GameData>>> {
    let team_games: Vec<GameData> = schedule
        .games
        .into_iter()
        .filter(|game| {
            let home = &game.teams.home.team.name;
            let away = &game.teams.away.team.name;
            home == team || away == team
        })
        .collect();

    match team_games.len() {
        0 => Ok(None),                 // Your team isn't playing today.
        1 | 2 => Ok(Some(team_games)), // Your team has a game or doubleheader today.
        n => anyhow::bail!("Teams play a maximum of 2 games per day. Got {n}"),
    }
}

pub fn select_game(team_games: Vec<GameData>, game_number: Option<&u8>) -> Result<GameData> {
    match team_games.len() {
        0 => anyhow::bail!(
            "I thought your team was playing today but there's no game data here. Aborting."
        ),
        1 => Ok(team_games.into_iter().next().unwrap()), // Not much to do if there's only 1 game that day.
        2 => {
            // If a valid game_number is specified, return that game.
            if let Some(n) = game_number {
                if [1, 2].contains(n) {
                    Ok(team_games.into_iter().nth((n - 1) as usize).unwrap())
                } else {
                    anyhow::bail!("Invalid game number.")
                }
            } else {
                // If no game number provided, prefer live game. If no live games, return game 1.
                let mut iter = team_games.into_iter();
                let game_one = iter.next().unwrap();
                let game_two = iter.next().unwrap();

                if game_two.status.abstract_game_state == "Live" {
                    Ok(game_two)
                } else {
                    Ok(game_one)
                }
            }
        }
        n => anyhow::bail!(
            "The last MLB tripleheader was played in 1920. Your team probably isn't playing {n} games today. Likely API change."
        ),
    }
}
