// This file is for reference only: large serde structs and schema mappings that were previously
// in schedule.rs have been moved here to keep the active module small and easy to follow.

#[derive(Debug, Deserialize)]
struct ScheduleResponse {
    dates: Vec<DaySchedule>,
    #[serde(rename = "totalGames")]
    total_games: u32,
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
    reason: Option<String>,
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
    // pub innings: Inning,  // Not yet defined.
    pub teams: ScoreTeams,
    // pub defense: Defense, // Not yet defined.
    // pub offense: Offense, // Not yet defined.
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