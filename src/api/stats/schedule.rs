use crate::api::session::MlbSession;
use crate::data::teamdata::Team;
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use serde::de::{self, Deserializer, Unexpected};
use std::fmt;

#[derive(Debug, Deserialize)]
struct ScheduleResponse {
    dates: Vec<DaySchedule>,
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
    pub date: NaiveDate,
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_pk: u64,
    pub game_date: String,
    pub status: GameStatus,
    pub teams: Matchup,
    pub linescore: Linescore,
    pub broadcasts: Option<Vec<Broadcast>>, // May be missing for future-dated games.
    pub content: Content,
    pub games_in_series: u8,
    pub series_game_number: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub abstract_game_state: String,
}

#[derive(Debug, Deserialize)]
pub struct Matchup {
    pub home: GameTeamStats,
    pub away: GameTeamStats,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub media: Media,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub epg_alternate: Option<Vec<Highlight>>,
}

#[derive(Debug, Deserialize)]
pub struct Highlight {
    items: Vec<HighlightDetails>,
    pub title: HighlightType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightType {
    CondensedGame,
    Recap,
}

impl<'de> Deserialize<'de> for HighlightType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "extended highlights" => Ok(HighlightType::CondensedGame),
            "daily recap" => Ok(HighlightType::Recap),
            other => Err(de::Error::invalid_value(
                Unexpected::Str(other),
                &"either 'Extended Highlights' or 'Daily Recap'",
            )),
        }
    }
}

impl fmt::Display for HighlightType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HighlightType::CondensedGame => write!(f, "Condensed Game"),
            HighlightType::Recap => write!(f, "Recap"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HighlightDetails {
    playbacks: Vec<HighlightPlayback>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HighlightPlayback {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTeamStats {
    pub team: GameTeam,
}

#[derive(Debug, Deserialize)]
pub struct GameTeam {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub teams: ScoreTeams,
}

#[derive(Debug, Deserialize)]
pub struct ScoreTeams {
    pub home: Score,
    pub away: Score,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub runs: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Broadcast {
    #[serde(rename = "type")]
    pub kind: String,
    pub is_national: bool,
    pub home_away: String,
    pub available_for_streaming: bool,
}

impl<State> MlbSession<State> {
    async fn make_schedule_request(
        &self,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> Result<ScheduleResponse> {
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
            "https://statsapi.mlb.com/api/v1/schedule?sportId=1&startDate={s}&endDate={e}&{h}",
            s = start_date,
            e = end_date,
            h = hydrate
        );

        let res = self
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

        Ok(body)
    }

    pub async fn fetch_schedule_by_range(
        &self,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> Result<Option<Vec<DaySchedule>>> {
        let resp = self.make_schedule_request(start_date, end_date).await?;

        match resp.dates.len() {
            0 => Ok(None), // If no games are scheduled, then no dates are returned.
            _ => Ok(Some(resp.dates)),
        }
    }

    pub async fn fetch_schedule_by_date(&self, date: &NaiveDate) -> Result<Option<DaySchedule>> {
        let resp = self.make_schedule_request(date, date).await?;

        match resp.dates.len() {
            0 => Ok(None), // If no games are scheduled, then no dates are returned.
            1 => Ok(Some(resp.dates.into_iter().next().unwrap())),
            n => anyhow::bail!("Expected 1 date but got {n} - possible API change."),
        }
    }

    pub async fn find_highlight_playback_url(
        &self,
        team: &Team,
        date: NaiveDate,
        highlight_type: HighlightType,
        game_number: Option<u8>,
    ) -> Result<Option<String>> {
        let Some(team_games) = self
            .fetch_schedule_by_date(&date)
            .await?
            .and_then(|s| s.find_team_games(team))
        else {
            tracing::info!("No games found for the {} on {}", team.name, date);
            return Ok(None);
        };

        let game_data = select_game(team_games, game_number)?;
        let Some(url) = game_data.find_highlight(highlight_type, "highBit") else {
            tracing::info!(
                "No high bitrate {} found for the {} on {}",
                highlight_type,
                team.name,
                date
            );
            return Ok(None);
        };

        Ok(Some(url))
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
            0 => None,             // Your team isn't playing today.
            _ => Some(team_games), // Your team has a game or doubleheader today.
        }
    }
}

impl GameData {
    fn find_highlight(&self, highlight_type: HighlightType, quality: &str) -> Option<String> {
        self.content
            .media
            .epg_alternate
            .as_ref()?
            .iter()
            .find(|h| h.title == highlight_type)?
            .items
            .iter()
            .flat_map(|details| details.playbacks.iter())
            .find(|pb| pb.name == quality)
            .map(|pb| pb.url.clone())
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
                // TODO: This assumes that the vector is ordered chronologically. May not be true.
                //       Can use game_number attribute of GameData if needed.
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
