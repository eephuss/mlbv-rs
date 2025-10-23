#![allow(dead_code)] // Shush unused refernce warnings until I know what fields are needed

use serde::Deserialize;
use serde_json::json;
use crate::{gamedata::{find_team_games, get_games_by_date, select_doubleheader_game}, session::{MlbSession, Authorized}};
use std::process::Command;

const MEDIA_GATEWAY_URL: & 'static str = "https://media-gateway.mlb.com/graphql";
const CONTENT_SEARCH_GQL: &str = include_str!("queries/content_search.gql");
const INIT_SESSION_GQL: &str = include_str!("queries/init_session.gql");
const INIT_PLAYBACK_SESSION_GQL: &str = include_str!("queries/init_playback_session.gql");

#[derive(Debug, Deserialize)]
struct ContentSearchResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    #[serde(rename = "contentSearch")]
    content_search: ContentSearchResults
}

#[derive(Debug, Deserialize)]
pub struct ContentSearchResults {
    total: u16,
    pub content: Vec<StreamData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamData {
    audio_tracks: Vec<AudioTrack>,
    content_id: String,
    pub media_id: String,
    content_type: String,
    content_restrictions: Vec<String>,
    content_restriction_details: Vec<ContentRestrictionDetail>,
    sport_id: u8,
    feed_type: String,
    pub call_sign: String,
    media_state: MediaState,
    pub fields: Vec<Field>,
    milestones: Vec<Milestone>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioTrack {
    language: String,
    name: String,
    rendition_name: String,
    track_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentRestrictionDetail {
    code: String,
    // details: serde_json::Value, // TODO: Find an example of content with restrictions to flesh this out.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    state: String,
    media_type: String,
    content_experience: String,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Milestone {
    milestone_type: String,
    relative_time: i32,
    absolute_time: String,
    title: String,
    keywords: Vec<Keyword>,
}

#[derive(Debug, Deserialize)]
struct Keyword {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct InitSessionResponse {
    data: InitSessionData,
}

#[derive(Debug, Deserialize)]
struct InitSessionData {
    #[serde(rename = "initSession")]
    init_session: InitSessionResults,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitSessionResults {
    device_id: String,
    session_id: String,
    entitlements: Vec<Entitlement>,
    location: Location,
    client_experience: String,
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Entitlement {
    code: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    country_code: String,
    // region_name: Option<>
    zip_code: String,
    latitude: f32,
    longitude: f32,
}

#[derive(Debug, Deserialize)]
struct InitPlaybackSessionResponse {
    data: InitPlaybackSessionData,
}

#[derive(Debug, Deserialize)]
struct InitPlaybackSessionData {
    #[serde(rename = "initPlaybackSession")]
    init_playback_session: InitPlaybackSessionResults,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitPlaybackSessionResults {
    playback_session_id: String,
    pub playback: Playback,
    // ad_scenarios: Vec<AdScenario>,
    // as_experience: AdExperience,
    // heartbeat_info: HeartbeatInfo,
    // tracking_obj: TrackingObj,
}

#[derive(Debug, Deserialize)]
pub struct Playback {
    pub url: String,
    token: String,
    pub expiration: String,
    cdn: String,
}

// This query fetches the available feeds for a specified game_pk.
pub async fn get_available_feeds(session: &MlbSession<Authorized>, game_pk: &str) -> anyhow::Result<Vec<StreamData>> {
    let access_token = &session.state.okta_tokens.access_token;

    let variables_query = format!(
        "GamePk={} AND ContentType=\"GAME\" RETURNING HomeTeamId, HomeTeamName, AwayTeamId, AwayTeamName, Date, MediaType, ContentExperience, MediaState, PartnerCallLetters",
        game_pk
    );

    let req_body = json!({
        "operationName": "contentSearch",
        "query": CONTENT_SEARCH_GQL,
        "variables": {
            "limit": 16,
            "query": variables_query
        }
    });

    let res = session.client
        .post(MEDIA_GATEWAY_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("x-bamsdk-version", "3.4")
        .header("x-bamsdk-platform", "macintosh")
        .header("Origin", "https://www.mlb.com")
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let res_body: ContentSearchResponse = res.json().await?;
    Ok(res_body.data.content_search.content)
}

// Initializes a session and outputs the requisite IDs.
async fn init_media_session(session: &MlbSession<Authorized>) -> anyhow::Result<(String, String)> {
    let access_token = &session.state.okta_tokens.access_token;

    let req_body = json!({
        "operationName": "initSession",
        "query": INIT_SESSION_GQL,
        "variables": {
            "device": {},
            "clientType": "WEB"
        }
    });

    let res = session.client
        .post(MEDIA_GATEWAY_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("x-bamsdk-version", "3.4")
        .header("x-bamsdk-platform", "macintosh")
        .header("Origin", "https://www.mlb.com")
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let res_body: InitSessionResponse = res.json().await?;

    let session_id = res_body.data.init_session.session_id;
    let device_id = res_body.data.init_session.device_id;

    Ok((session_id, device_id))
}

// Use mediaID, sessionID and deviceID to initPlaybackSession and retrieve stream URI. 
pub async fn init_playback_session(session: &MlbSession<Authorized>, media_id: &str) -> anyhow::Result<InitPlaybackSessionResults> {
    let access_token = &session.state.okta_tokens.access_token;
    let (session_id, device_id) = init_media_session(&session).await?;

    let req_body = json!({
        "operationName": "initPlaybackSession",
        "query": INIT_PLAYBACK_SESSION_GQL,
        "variables": {
            "adCapabilities": ["GOOGLE_STANDALONE_AD_PODS"],
            "deviceId": device_id.as_str(),
            "mediaId": media_id,
            "quality": "PLACEHOLDER",
            "sessionId": session_id.as_str()
        }
    });

    let res = session.client
        .post(MEDIA_GATEWAY_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("x-bamsdk-version", "3.4")
        .header("x-bamsdk-platform", "macintosh")
        .header("Origin", "https://www.mlb.com")
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let res_body: InitPlaybackSessionResponse = res.json().await?;

    Ok(res_body.data.init_playback_session)
}

fn select_game_feed(stream_data: Vec<StreamData>, media_type: &str, feed_type: &str) -> Option<StreamData> {
    // Filter out inactive streams.
    let mut active_streams: Vec<StreamData> = stream_data
        .into_iter()
        .filter(|stream| stream.media_state.state != "OFF")
        .collect();

    if active_streams.is_empty() {
        println!("No playable feeds are available for this game.");
        return None;
    }

    // Return stream matching user preferences if one available.
    if let Some(pos) = active_streams.iter().position(|stream| {
        stream.feed_type == feed_type &&
        stream.media_state.media_type == media_type
    }) {
        println!("Feed matching user preferences found");
        return Some(active_streams.swap_remove(pos))
    }

    // If no matches, fall back to National broadcast. Most common cause of match failure is national-only broadcasts.
    if let Some(pos) = active_streams.iter().position(|stream| {
        stream.feed_type == "NETWORK" && // TODO: Find all codes for National broadcasts + make into enum.
        stream.media_state.media_type == media_type
    }) {
        println!("{feed_type} broadcast not found. Defaulting to National feed.");
        return Some(active_streams.swap_remove(pos))
    }

    // If user has selected audio-only they may be bandwitch constrained. Falling back to video inappropriate.
    if media_type == "AUDIO" {
        println!("Only video feeds are available for this game. User requested audio only.");
        return None;
    }

    // If no matches and no National broadcast try audio feed. User may be blacked out from video but audio accessible.
    if let Some(pos) = active_streams.iter().position(|stream| {
        stream.feed_type == feed_type &&
        stream.media_state.media_type == "AUDIO"
    }) {
        println!("No video feeds are available for this game. User may be blacked out. Defaulting to audio feed.");
        return Some(active_streams.swap_remove(pos))
    }

    // If none of the above conditions are met but there are somehow still active feeds remaining, just grab first one.
    active_streams.into_iter().next()
}

fn play_stream(url: &str, media_player: &str) -> anyhow::Result<()> {
    Command::new(media_player).arg(url).spawn()?;

    Ok(())
}

// Currently requires authenticated session. Reauth has to happen before this is called. Is that what I want?
pub async fn play_game_stream(
    session: &MlbSession<Authorized>,
    team: &str,
    date_opt: &str,
    media_type: &str, // TODO: Make this a proper enum
    feed_type: Option<&str>, // TODO: Make this a proper enum
    game_number: Option<&u8>,
    // media_player: Option<MediaPlayer>,
) -> anyhow::Result<()> {
    // Get games for specified date. If none, end here.
    let Some(schedule) = get_games_by_date(session, Some(date_opt)).await? else {
        println!("No games scheduled for {:?}", date_opt);
        return Ok(());
    };

    // If there are any games, is specified team playing? If not, end here.
    let Some(team_games) = find_team_games(schedule, team)? else {
        println!("Looks like the {team} aren't playing today.");
        return Ok(());        
    };
    
    // If there's a doubleheader, select which game to retrieve.
    let game_data = select_doubleheader_game(team_games, game_number)?;
    let game_pk = game_data.game_pk.to_string();

    // If user specifies a feed type, use it. Else match to team.
    let feed_type = feed_type.unwrap_or(
        if game_data.teams.home.team.name == team { "HOME" } else { "AWAY" }
    );

    // Get available feeds for selected game_pk
    let stream_data: Vec<StreamData> = get_available_feeds(session, &game_pk).await?;

    // Select most appropriate feed given user preferences
    let Some(stream_data) = select_game_feed(stream_data, media_type, feed_type) else {
        println!("No streams found that meet user criteria.");
        return Ok(());
    };
    let media_id = &stream_data.media_id;

    // Initialize a playback session containing stream URL.
    let playback_session = init_playback_session(&session, media_id).await?;

    let media_player = "mpv.exe";
    // Send playback URL and other relevant info to media player.
    play_stream(playback_session.playback.url.as_str(), media_player)?;

    Ok(())
}