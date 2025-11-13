use crate::api::session::{Authorized, MlbSession};
use crate::api::stats::schedule;
use crate::data::teamdata::Team;
use crate::player;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::str::FromStr;

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
    pub content: Vec<StreamData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamData {
    pub media_id: String,
    feed_type: FeedType,
    language: String,
    media_state: MediaState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FeedType {
    Home,
    Away,
    Network,
}

impl FromStr for FeedType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "home" => Ok(Self::Home),
            "away" => Ok(Self::Away),
            "national" => Ok(Self::Network),
            _ => anyhow::bail!("Invalid feed type: {s}; expected 'home', 'away' or 'national'"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    state: String,
    media_type: MediaType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MediaType {
    Audio,
    Video,
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
    pub playback: Playback,
}

#[derive(Debug, Deserialize)]
pub struct Playback {
    pub url: String,
}

impl MlbSession<Authorized> {
    // This query fetches the available feeds for a specified game_pk.
    pub async fn fetch_available_feeds(&self, game_pk: &u64) -> Result<ContentSearchResults> {
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
            .await
            .context("Failed to send content search request")?
            .error_for_status()
            .context("Content search request returned unsuccessful status")?;

        let res_body: ContentSearchResponse = res
            .json()
            .await
            .context("Failed to parse content search response")?;

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
            .await
            .context("Failed to send init media session request")?
            .error_for_status()
            .context("Init media session request returned unsuccessful status")?;

        let res_body: InitSessionResponse = res
            .json()
            .await
            .context("Failed to parse init media session response")?;

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
            .await
            .context("Failed to send init playback session request")?
            .error_for_status()
            .context("Init playback session request returned unsuccessful status")?;

        let res_body: InitPlaybackSessionResponse = res
            .json()
            .await
            .context("Failed to parse init playback session response")?;

        Ok(res_body.data.init_playback_session)
    }

    pub async fn find_and_play_stream(
        &self,
        team: &Team,
        date: NaiveDate,
        media_type: MediaType,
        feed_type: Option<FeedType>,
        game_number: Option<u8>,
        media_player: Option<&str>,
    ) -> Result<()> {
        // Fetch schedule and filter for team games on specified date.
        let Some(team_games) = self
            .fetch_schedule_by_date(&date)
            .await?
            .and_then(|s| s.find_team_games(team))
        else {
            tracing::info!("No games found for the {} on {}", team.name, date);
            return Ok(());
        };

        // Return a single game then match feed type to team's home/away status if not provided.
        let game_data = schedule::select_game(team_games, game_number)?;
        let feed_type = feed_type.unwrap_or_else(|| {
            if game_data.teams.home.team.name == team.name {
                FeedType::Home
            } else {
                FeedType::Away
            }
        });

        // Fetch available streams for selected game.
        let stream_data = self.fetch_available_feeds(&game_data.game_pk).await?;
        let Some(stream_data) = stream_data.find_best_feed(media_type, feed_type) else {
            tracing::warn!("No streams available; user may not have access to this content");
            return Ok(());
        };

        // Initialize a playback session containing stream URL.
        let playback_session = self.init_playback_session(&stream_data.media_id).await?;
        player::play_stream_url(playback_session.playback.url, media_player)?;

        Ok(())
    }
}

impl ContentSearchResults {
    fn select_feed(&self, media_type: MediaType, feed_type: FeedType) -> Option<&StreamData> {
        self.content.iter().find(|stream| {
            stream.feed_type == feed_type
                && stream.media_state.media_type == media_type
                && stream.media_state.state != "OFF"
                && stream.language == "en" // TODO: Make language preference configurable. es + fr
        })
    }

    fn find_best_feed(&self, media_type: MediaType, feed_type: FeedType) -> Option<&StreamData> {
        let search_prefs = vec![
            (
                media_type,
                feed_type,
                "Found feed matching user preferences",
            ),
            (
                media_type,
                FeedType::Network,
                "Home/away feed not found; falling back to national",
            ),
            (
                MediaType::Audio,
                feed_type,
                "Video feed not found; user may be blacked out; trying audio",
            ),
        ];

        // Loop through search preferences in order and return the first matching stream.
        for (m_type, f_type, message) in search_prefs {
            if let Some(stream) = self.select_feed(m_type, f_type) {
                tracing::info!(?f_type, ?m_type, "{message}");
                return Some(stream);
            } else {
                tracing::debug!(?f_type, ?m_type, "No streams found for this combination");
            }
        }

        // Last resort: pick any active stream. TODO: Message if results empty.
        tracing::warn!("Couldn't find stream matching user preferences; grabbing first available");
        self.content
            .iter()
            .find(|stream| stream.media_state.state != "OFF")
    }
}
