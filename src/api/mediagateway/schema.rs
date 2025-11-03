// This file is for reference only: large serde structs and schema mappings that were previously
// in streams.rs have been moved here to keep the active module small and easy to follow.

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