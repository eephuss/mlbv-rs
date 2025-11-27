use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum League {
    American,
    National,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivisionRegion {
    East,
    Central,
    West,
}

#[derive(Debug)]
#[allow(dead_code)] // Hush warnings until these are used.
pub struct Division {
    pub name: DivisionRegion,
    pub league: League,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "String")]
pub enum TeamCode {
    Ari,
    Ath,
    Atl,
    Bal,
    Bos,
    Chc,
    Cin,
    Cws,
    Cle,
    Col,
    Det,
    Hou,
    Kcr,
    Laa,
    Lad,
    Mil,
    Min,
    Mia,
    Nyy,
    Nym,
    // Oak,
    Phi,
    Pit,
    Sdp,
    Sea,
    Sfg,
    Stl,
    Tbr,
    Tex,
    Tor,
    Wsh,
}

impl fmt::Display for TeamCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self).to_uppercase();
        write!(f, "{s}")
    }
}

impl FromStr for TeamCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ARI" => Ok(Self::Ari),
            "ATH" => Ok(Self::Ath),
            "ATL" => Ok(Self::Atl),
            "BAL" => Ok(Self::Bal),
            "BOS" => Ok(Self::Bos),
            "CHC" => Ok(Self::Chc),
            "CIN" => Ok(Self::Cin),
            "CWS" => Ok(Self::Cws),
            "CLE" => Ok(Self::Cle),
            "COL" => Ok(Self::Col),
            "DET" => Ok(Self::Det),
            "HOU" => Ok(Self::Hou),
            "KCR" => Ok(Self::Kcr),
            "LAA" => Ok(Self::Laa),
            "LAD" => Ok(Self::Lad),
            "MIL" => Ok(Self::Mil),
            "MIN" => Ok(Self::Min),
            "MIA" => Ok(Self::Mia),
            "NYY" => Ok(Self::Nyy),
            "NYM" => Ok(Self::Nym),
            "OAK" => Ok(Self::Ath), // Continue to map OAK to ATH
            "PHI" => Ok(Self::Phi),
            "PIT" => Ok(Self::Pit),
            "SDP" => Ok(Self::Sdp),
            "SEA" => Ok(Self::Sea),
            "SFG" => Ok(Self::Sfg),
            "STL" => Ok(Self::Stl),
            "TBR" => Ok(Self::Tbr),
            "TEX" => Ok(Self::Tex),
            "TOR" => Ok(Self::Tor),
            "WSH" => Ok(Self::Wsh),
            _ => anyhow::bail!("Invalid team code: {s}"),
        }
    }
}

impl TryFrom<String> for TeamCode {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        TeamCode::from_str(&s)
    }
}

impl TeamCode {
    pub fn team(self) -> &'static Team {
        Team::find_by_code(self)
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)] // Hush warnings until these are used.
pub struct Team {
    pub id: u32,
    pub code: TeamCode,
    pub name: &'static str,
    pub nickname: &'static str,
    pub division: &'static Division,
    pub primary_color: TeamColor,
}

#[derive(Copy, Clone, Debug)]
pub enum TeamColor {
    Red,
    Blue,
    Orange,
    Green,
    Yellow,
    Magenta,
    Cyan,
    // Black,
    White,
}

impl TeamColor {
    pub fn to_tabled_color(self) -> tabled::settings::Color {
        match self {
            TeamColor::Red => tabled::settings::Color::FG_RED,
            TeamColor::Blue => tabled::settings::Color::FG_BLUE,
            TeamColor::Orange => tabled::settings::Color::FG_BRIGHT_RED, // No orange in ANSI
            TeamColor::Green => tabled::settings::Color::FG_GREEN,
            TeamColor::Yellow => tabled::settings::Color::FG_YELLOW,
            TeamColor::Magenta => tabled::settings::Color::FG_MAGENTA,
            TeamColor::Cyan => tabled::settings::Color::FG_CYAN,
            // TeamColor::Black => tabled::settings::Color::FG_BRIGHT_BLACK, // Needs to be visible
            TeamColor::White => tabled::settings::Color::FG_WHITE,
        }
    }
}

impl Team {
    /// Find a team by its code (e.g., "wsh", "tor")
    pub fn find_by_code(code: TeamCode) -> &'static Team {
        TEAMS
            .iter()
            .find(|team| team.code == code)
            .expect("TeamCode not found in TEAMS constant - this is a bug")
    }

    pub fn find_by_name(name: &str) -> Option<&'static Team> {
        TEAMS.iter().find(|team| team.name == name)
    }

    pub fn find_by_id(id: &u32) -> &'static Team {
        TEAMS
            .iter()
            .find(|team| team.id == *id)
            .expect("Team ID not found in TEAMS constant - this is a bug")
    }
}

pub const AL_EAST: Division = Division {
    name: DivisionRegion::East,
    league: League::American,
};
pub const AL_CENTRAL: Division = Division {
    name: DivisionRegion::Central,
    league: League::American,
};
pub const AL_WEST: Division = Division {
    name: DivisionRegion::West,
    league: League::American,
};

pub const NL_EAST: Division = Division {
    name: DivisionRegion::East,
    league: League::National,
};
pub const NL_CENTRAL: Division = Division {
    name: DivisionRegion::Central,
    league: League::National,
};
pub const NL_WEST: Division = Division {
    name: DivisionRegion::West,
    league: League::National,
};

pub const TEAMS: &[Team] = &[
    Team {
        id: 108,
        code: TeamCode::Laa,
        name: "Los Angeles Angels",
        nickname: "Angels",
        division: &AL_WEST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 109,
        code: TeamCode::Ari,
        name: "Arizona Diamondbacks",
        nickname: "Diamondbacks",
        division: &NL_WEST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 110,
        code: TeamCode::Bal,
        name: "Baltimore Orioles",
        nickname: "Orioles",
        division: &AL_EAST,
        primary_color: TeamColor::Orange,
    },
    Team {
        id: 111,
        code: TeamCode::Bos,
        name: "Boston Red Sox",
        nickname: "Red Sox",
        division: &AL_EAST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 112,
        code: TeamCode::Chc,
        name: "Chicago Cubs",
        nickname: "Cubs",
        division: &NL_CENTRAL,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 113,
        code: TeamCode::Cin,
        name: "Cincinnati Reds",
        nickname: "Reds",
        division: &NL_CENTRAL,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 114,
        code: TeamCode::Cle,
        name: "Cleveland Guardians",
        nickname: "Guardians",
        division: &AL_CENTRAL,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 115,
        code: TeamCode::Col,
        name: "Colorado Rockies",
        nickname: "Rockies",
        division: &NL_WEST,
        primary_color: TeamColor::Magenta,
    },
    Team {
        id: 116,
        code: TeamCode::Det,
        name: "Detroit Tigers",
        nickname: "Tigers",
        division: &AL_CENTRAL,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 117,
        code: TeamCode::Hou,
        name: "Houston Astros",
        nickname: "Astros",
        division: &AL_WEST,
        primary_color: TeamColor::Orange,
    },
    Team {
        id: 118,
        code: TeamCode::Kcr,
        name: "Kansas City Royals",
        nickname: "Royals",
        division: &AL_CENTRAL,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 119,
        code: TeamCode::Lad,
        name: "Los Angeles Dodgers",
        nickname: "Dodgers",
        division: &NL_WEST,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 120,
        code: TeamCode::Wsh,
        name: "Washington Nationals",
        nickname: "Nationals",
        division: &NL_EAST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 121,
        code: TeamCode::Nym,
        name: "New York Mets",
        nickname: "Mets",
        division: &NL_EAST,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 133,
        code: TeamCode::Ath, // TODO: Change code if they ever make it to Vegas. LVA?
        name: "Athletics",   // TODO: Update/re-add city name too.
        nickname: "Athletics",
        division: &AL_WEST,
        primary_color: TeamColor::Green,
    },
    Team {
        id: 134,
        code: TeamCode::Pit,
        name: "Pittsburgh Pirates",
        nickname: "Pirates",
        division: &NL_CENTRAL,
        primary_color: TeamColor::Yellow,
    },
    Team {
        id: 135,
        code: TeamCode::Sdp,
        name: "San Diego Padres",
        nickname: "Padres",
        division: &NL_WEST,
        primary_color: TeamColor::Yellow,
    },
    Team {
        id: 136,
        code: TeamCode::Sea,
        name: "Seattle Mariners",
        nickname: "Mariners",
        division: &AL_WEST,
        primary_color: TeamColor::Cyan,
    },
    Team {
        id: 137,
        code: TeamCode::Sfg,
        name: "San Francisco Giants",
        nickname: "Giants",
        division: &NL_WEST,
        primary_color: TeamColor::Orange,
    },
    Team {
        id: 138,
        code: TeamCode::Stl,
        name: "St. Louis Cardinals",
        nickname: "Cardinals",
        division: &NL_CENTRAL,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 139,
        code: TeamCode::Tbr,
        name: "Tampa Bay Rays",
        nickname: "Rays",
        division: &AL_EAST,
        primary_color: TeamColor::Green,
    },
    Team {
        id: 140,
        code: TeamCode::Tex,
        name: "Texas Rangers",
        nickname: "Rangers",
        division: &AL_WEST,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 141,
        code: TeamCode::Tor,
        name: "Toronto Blue Jays",
        nickname: "Blue Jays",
        division: &AL_EAST,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 142,
        code: TeamCode::Min,
        name: "Minnesota Twins",
        nickname: "Twins",
        division: &AL_CENTRAL,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 143,
        code: TeamCode::Phi,
        name: "Philadelphia Phillies",
        nickname: "Phillies",
        division: &NL_EAST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 144,
        code: TeamCode::Atl,
        name: "Atlanta Braves",
        nickname: "Braves",
        division: &NL_EAST,
        primary_color: TeamColor::Red,
    },
    Team {
        id: 145,
        code: TeamCode::Cws,
        name: "Chicago White Sox",
        nickname: "White Sox",
        division: &AL_CENTRAL,
        primary_color: TeamColor::White,
    },
    Team {
        id: 146,
        code: TeamCode::Mia,
        name: "Miami Marlins",
        nickname: "Marlins",
        division: &NL_EAST,
        primary_color: TeamColor::Cyan,
    },
    Team {
        id: 147,
        code: TeamCode::Nyy,
        name: "New York Yankees",
        nickname: "Yankees",
        division: &AL_EAST,
        primary_color: TeamColor::Blue,
    },
    Team {
        id: 158,
        code: TeamCode::Mil,
        name: "Milwaukee Brewers",
        nickname: "Brewers",
        division: &NL_CENTRAL,
        primary_color: TeamColor::Blue,
    },
];
