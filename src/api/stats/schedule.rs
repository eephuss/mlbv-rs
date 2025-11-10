use crate::api::session::MlbSession;
use crate::cli::display::{self, GameRow};
use crate::data::teamdata::Team;
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use serde::Deserialize;
use tabled::Table;

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
    date: NaiveDate,
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_pk: u64,
    game_date: String,
    status: GameStatus,
    pub teams: Matchup,
    pub linescore: Linescore,
    broadcasts: Vec<Broadcast>,
    games_in_series: u8,
    series_game_number: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameStatus {
    abstract_game_state: String,
}

#[derive(Debug, Deserialize)]
pub struct Matchup {
    pub home: GameTeamStats,
    pub away: GameTeamStats,
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
    home: Score,
    away: Score,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    runs: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Broadcast {
    #[serde(rename = "type")]
    kind: String,
    is_national: bool,
    home_away: String,
    available_for_streaming: bool,
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

    pub fn prepare_schedule_table(&self) -> Table {
        let weekday = self.date.format("%A");
        let header_date = format!("{} {}", self.date, weekday);

        let mut rows = Vec::new();

        for game in &self.games {
            let game_time = DateTime::parse_from_rfc3339(&game.game_date)
                .map(|dt| {
                    dt.with_timezone(&Local)
                        .format("%I:%M %p")
                        .to_string()
                        .to_lowercase()
                })
                .unwrap_or_else(|_| "TBD".to_string());

            let away_team = &game.teams.away.team.name;
            let home_team = &game.teams.home.team.name;
            let matchup = format!("{game_time} - {away_team} at {home_team}");

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
                matchup,
                series,
                score,
                state,
                feeds,
            });
        }
        display::format_schedule_table(rows, &header_date)
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
