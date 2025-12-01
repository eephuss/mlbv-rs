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
    pub linescore: Option<Linescore>, // May be missing for rescheduled games.
    pub broadcasts: Option<Vec<Broadcast>>, // May be missing for future-dated games.
    pub content: Content,
    pub game_number: u8, // Used for double-headers
    pub games_in_series: u8,
    pub series_game_number: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    pub abstract_game_state: String,
    pub detailed_state: String,
    pub status_code: String,
    pub reason: Option<String>,
    pub coded_game_state: String,
}

#[derive(Debug, Deserialize)]
pub struct Matchup {
    pub home: GameTeamStats,
    pub away: GameTeamStats,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub media: Option<Media>,
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
            HighlightType::CondensedGame => write!(f, "CG"),
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
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linescore {
    pub teams: ScoreTeams,
    pub current_inning: Option<u8>,
    pub is_top_inning: Option<bool>,
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
    pub language: String,
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
        let mut dates = self.make_schedule_request(date, date).await?.dates;

        match dates.len() {
            0 => Ok(None), // If no games are scheduled, then no dates are returned.
            1 => Ok(dates.pop()),
            n => anyhow::bail!(
                "Expected exactly 1 schedule for {date} but got {n} - possible API change."
            ),
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
            println!("No games found for the {} on {}", team.name, date);
            return Ok(None);
        };

        let game_data = select_game(team_games, game_number)?;
        let Some(url) = game_data.find_highlight(highlight_type, "highBit") else {
            println!(
                "No high bitrate {} found for the {} on {}",
                highlight_type, team.name, date
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
            .as_ref()?
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

pub fn select_game(mut team_games: Vec<GameData>, game_number: Option<u8>) -> Result<GameData> {
    match team_games.len() {
        0 => anyhow::bail!("Expected at least 1 game but got 0; aborting"),
        1 => Ok(team_games.pop().expect("len == 1; you should never see me")),
        _ => {
            // If a valid game_number is specified, return that game.
            if let Some(n) = game_number {
                if n != 1 && n != 2 {
                    anyhow::bail!("Invalid game number - {n}");
                }
                if let Some(pos) = team_games.iter().position(|g| g.game_number == n) {
                    return Ok(team_games.swap_remove(pos));
                } else {
                    anyhow::bail!("Game {n} not found in schedule");
                }
            }
            tracing::info!("Doubleheader detected but no game number specified");

            // Prefer live game; else game 1.
            if let Some(pos) = team_games
                .iter()
                .position(|g| g.status.abstract_game_state == "Live")
            {
                tracing::info!("Live game found; selecting that game for playback");
                return Ok(team_games.swap_remove(pos));
            }
            if let Some(pos) = team_games.iter().position(|g| g.game_number == 1) {
                tracing::info!("No live game; defaulting to game 1 of doubleheader");
                return Ok(team_games.swap_remove(pos));
            }

            // Defensive fallback (should be unreachable if API is consistent).
            Ok(team_games.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn mock_game(pk: u64, state: &str, game_number: u8) -> GameData {
        GameData {
            game_pk: pk,
            game_date: "2024-10-01T19:00:00Z".to_string(),
            status: GameStatus {
                abstract_game_state: state.to_string(),
                detailed_state: "Final".to_string(),
                status_code: "F".to_string(),
                reason: None,
                coded_game_state: "F".to_string(),
            },
            teams: Matchup {
                home: GameTeamStats {
                    team: GameTeam {
                        id: 120,
                        name: "Washington Nationals".to_string(),
                    },
                },
                away: GameTeamStats {
                    team: GameTeam {
                        id: 141,
                        name: "Toronto Blue Jays".to_string(),
                    },
                },
            },
            linescore: Some(Linescore {
                current_inning: Some(7),
                is_top_inning: Some(true),
                teams: ScoreTeams {
                    home: Score { runs: Some(3) },
                    away: Score { runs: Some(2) },
                },
            }),
            broadcasts: Some(vec![]),
            content: Content {
                media: Some(Media {
                    epg_alternate: None,
                }),
            },
            game_number,
            games_in_series: 3,
            series_game_number: 1,
        }
    }

    #[test]
    fn select_game_returns_single_game() {
        let games = vec![mock_game(12345, "Final", 1)];
        let result = select_game(games, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().game_pk, 12345);
    }

    #[test]
    fn select_game_errors_on_empty_vector() {
        let games = vec![];
        let result = select_game(games, None);
        assert!(result.is_err());
    }

    #[test]
    fn select_game_selects_by_game_number() {
        let games = vec![mock_game(11111, "Final", 1), mock_game(22222, "Final", 2)];
        let result = select_game(games, Some(2));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().game_pk, 22222);
    }

    #[test]
    fn select_game_prefers_live_game() {
        let games = vec![mock_game(11111, "Final", 1), mock_game(22222, "Live", 2)];
        let result = select_game(games, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().game_pk, 22222);
    }

    #[test]
    fn select_game_defaults_to_game_one() {
        let games = vec![mock_game(11111, "Final", 1), mock_game(22222, "Final", 2)];
        let result = select_game(games, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().game_pk, 11111);
    }

    #[test]
    fn select_game_rejects_invalid_game_number() {
        let games = vec![mock_game(11111, "Final", 1), mock_game(22222, "Final", 2)];
        let result = select_game(games, Some(3));
        assert!(result.is_err());
    }

    #[test]
    fn game_date_from_str_parses_valid_formats() {
        assert!(GameDate::from_str("2024-10-01").is_ok());
        assert!(GameDate::from_str("10-01-2024").is_ok());
        assert!(GameDate::from_str("10/01/2024").is_ok());
    }

    #[test]
    fn game_date_from_str_rejects_old_dates() {
        let result = GameDate::from_str("2021-10-01");
        assert!(result.is_err());
    }

    #[test]
    fn game_date_from_str_rejects_invalid_format() {
        let result = GameDate::from_str("invalid-date");
        assert!(result.is_err());
    }
}
