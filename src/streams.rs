#![allow(dead_code)] // Shush unused refernce warnings until I know what fields are needed

use crate::gamedata;
use crate::session::{Authorized, MlbSession};
use anyhow::Result;
use serde::Deserialize;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const MEDIA_GATEWAY_URL: &str = "https://media-gateway.mlb.com/graphql";
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
    content_search: ContentSearchResults,
}

#[derive(Debug, Deserialize)]
pub struct ContentSearchResults {
    total: u16,
    pub content: Vec<StreamData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FeedType {
    Home,
    Away,
    Network,
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
    feed_type: FeedType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MediaType {
    Audio,
    Video,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    state: String,
    media_type: MediaType,
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
    code: String,
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

impl MlbSession<Authorized> {
    // This query fetches the available feeds for a specified game_pk.
    pub async fn get_available_feeds(&self, game_pk: &u64) -> Result<ContentSearchResults> {
        let access_token = &self.state.okta_tokens.access_token;

        let variables_query = format!(
            "GamePk={game_pk} \
            AND ContentType=\"GAME\" \
            RETURNING \
            HomeTeamId, \
            HomeTeamName, \
            AwayTeamId, \
            AwayTeamName, \
            Date, \
            MediaType, \
            ContentExperience, \
            MediaState, \
            PartnerCallLetters"
        );

        let req_body = serde_json::json!({
            "operationName": "contentSearch",
            "query": CONTENT_SEARCH_GQL,
            "variables": {
                "limit": 16,
                "query": variables_query
            }
        });

        let res = self
            .client
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

        Ok(res_body.data.content_search)
    }

    // Initializes a session and outputs the requisite IDs.
    async fn init_media_session(&self) -> Result<(String, String)> {
        let access_token = &self.state.okta_tokens.access_token;

        let req_body = serde_json::json!({
            "operationName": "initSession",
            "query": INIT_SESSION_GQL,
            "variables": {
                "device": {},
                "clientType": "WEB"
            }
        });

        let res = self
            .client
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
    pub async fn init_playback_session(
        &self,
        media_id: &str,
    ) -> Result<InitPlaybackSessionResults> {
        let access_token = &self.state.okta_tokens.access_token;
        let (session_id, device_id) = self.init_media_session().await?;

        let req_body = serde_json::json!({
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

        let res = self
            .client
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
}

impl ContentSearchResults {
    fn select_feed(&self, media_type: MediaType, feed_type: FeedType) -> Option<&StreamData> {
        self.content.iter().find(|stream| {
            stream.feed_type == feed_type
                && stream.media_state.media_type == media_type
                && stream.media_state.state != "OFF"
        })
    }

    fn find_best_feed(&self, media_type: MediaType, feed_type: FeedType) -> Option<&StreamData> {
        // TODO: Do I want an enum to represent these various states? How best to handle logging?
        let search_prefs = vec![
            (media_type, feed_type),         // Exact match to user preferences.
            (media_type, FeedType::Network), // Fallback to national broadcast if home/away feeds aren't available.
            (MediaType::Audio, feed_type),   // Audio typically available if video is blacked out.
        ];

        // Loop through search preferences in order and return the first matching stream.
        for (m_type, f_type) in search_prefs {
            if let Some(stream) = self.select_feed(m_type, f_type) {
                println!("Selected feed: {f_type:?}, {m_type:?}");
                return Some(stream);
            }
        }

        // Last resort: pick any active stream. TODO: Message if results empty.
        self.content
            .iter()
            .find(|stream| stream.media_state.state != "OFF")
    }
}

fn find_in_path(command: &str) -> io::Result<PathBuf> {
    // TODO: Do I need this here? I'm using anyhow everywhere else.
    let not_found_error = io::Error::new(
        io::ErrorKind::NotFound,
        format!("Command '{}' not found in PATH", command),
    );

    #[cfg(target_os = "windows")]
    let output = Command::new("where").arg(command).output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("which").arg(command).output()?;

    if !output.status.success() {
        return Err(not_found_error);
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout_str.lines().map(str::trim).collect();

    let path = lines
        .iter()
        .find(|s| s.ends_with(".exe"))
        .or_else(|| lines.first())
        .ok_or(not_found_error)?;

    Ok(PathBuf::from(path))
}

pub fn resolve_media_player(media_player: &str) -> io::Result<(String, Vec<String>)> {
    // Use specified player if found in PATH
    if let Ok(path) = find_in_path(media_player) {
        return Ok((path.to_string_lossy().into_owned(), Vec::new()));
    }

    // Otherwise fall back to OS default
    #[cfg(target_os = "windows")]
    {
        Ok(("cmd".into(), vec!["/C".into(), "start".into()]))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(("open".into(), Vec::new()))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Ok(("xdg-open".into(), Vec::new()))
    }
}

fn play_stream(command: &str, args: Vec<String>) -> Result<()> {
    Command::new(command).args(args).spawn()?;

    Ok(())
}

pub async fn find_and_play_stream(
    session: &MlbSession<Authorized>,
    team: &str,
    date: &str,
    media_type: MediaType,
    feed_type: FeedType,
    game_number: Option<&u8>,
    // media_player: Option<MediaPlayer>,
) -> Result<()> {
    // Fetch schedule and filter for team games on specified date.
    let Some(team_games) = session
        .get_games_by_date(date)
        .await?
        .and_then(|s| s.find_team_games(team))
    else {
        println!("Looks like the {team} aren't playing on {date}.");
        return Ok(());
    };
    let game_data = gamedata::select_game(team_games, game_number)?;

    // Fetch available streams for selected game.
    let stream_data = session.get_available_feeds(&game_data.game_pk).await?;
    let Some(stream_data) = stream_data.find_best_feed(media_type, feed_type) else {
        println!("No streams found that meet user criteria.");
        return Ok(());
    };

    // Initialize a playback session containing stream URL.
    let playback_session = session.init_playback_session(&stream_data.media_id).await?;
    let (command, mut args) = resolve_media_player("mpv")?;
    args.push(playback_session.playback.url);

    // Send playback URL and other relevant info to media player.
    play_stream(&command, args)?;

    Ok(())
}
