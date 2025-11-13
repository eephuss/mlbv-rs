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

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl std::str::FromStr for TeamCode {
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
            "OAK" => anyhow::bail!("The OAK code has been retired and replaced with ATH"),
            _ => anyhow::bail!("Invalid team code: {s}"),
        }
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
    pub division: &'static Division,
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
        division: &AL_WEST,
    },
    Team {
        id: 109,
        code: TeamCode::Ari,
        name: "Arizona Diamondbacks",
        division: &NL_WEST,
    },
    Team {
        id: 110,
        code: TeamCode::Bal,
        name: "Baltimore Orioles",
        division: &AL_EAST,
    },
    Team {
        id: 111,
        code: TeamCode::Bos,
        name: "Boston Red Sox",
        division: &AL_EAST,
    },
    Team {
        id: 112,
        code: TeamCode::Chc,
        name: "Chicago Cubs",
        division: &NL_CENTRAL,
    },
    Team {
        id: 113,
        code: TeamCode::Cin,
        name: "Cincinnati Reds",
        division: &NL_CENTRAL,
    },
    Team {
        id: 114,
        code: TeamCode::Cle,
        name: "Cleveland Guardians",
        division: &AL_CENTRAL,
    },
    Team {
        id: 115,
        code: TeamCode::Col,
        name: "Colorado Rockies",
        division: &NL_WEST,
    },
    Team {
        id: 116,
        code: TeamCode::Det,
        name: "Detroit Tigers",
        division: &AL_CENTRAL,
    },
    Team {
        id: 117,
        code: TeamCode::Hou,
        name: "Houston Astros",
        division: &AL_WEST,
    },
    Team {
        id: 118,
        code: TeamCode::Kcr,
        name: "Kansas City Royals",
        division: &AL_CENTRAL,
    },
    Team {
        id: 119,
        code: TeamCode::Lad,
        name: "Los Angeles Dodgers",
        division: &NL_WEST,
    },
    Team {
        id: 120,
        code: TeamCode::Wsh,
        name: "Washington Nationals",
        division: &NL_EAST,
    },
    Team {
        id: 121,
        code: TeamCode::Nym,
        name: "New York Mets",
        division: &NL_EAST,
    },
    Team {
        id: 133,
        code: TeamCode::Ath, // TODO: Change code if they ever make it to Vegas. LVA?
        name: "Athletics",   // TODO: Update/re-add city name too.
        division: &AL_WEST,
    },
    Team {
        id: 134,
        code: TeamCode::Pit,
        name: "Pittsburgh Pirates",
        division: &NL_CENTRAL,
    },
    Team {
        id: 135,
        code: TeamCode::Sdp,
        name: "San Diego Padres",
        division: &NL_WEST,
    },
    Team {
        id: 136,
        code: TeamCode::Sea,
        name: "Seattle Mariners",
        division: &AL_WEST,
    },
    Team {
        id: 137,
        code: TeamCode::Sfg,
        name: "San Francisco Giants",
        division: &NL_WEST,
    },
    Team {
        id: 138,
        code: TeamCode::Stl,
        name: "St. Louis Cardinals",
        division: &NL_CENTRAL,
    },
    Team {
        id: 139,
        code: TeamCode::Tbr,
        name: "Tampa Bay Rays",
        division: &AL_EAST,
    },
    Team {
        id: 140,
        code: TeamCode::Tex,
        name: "Texas Rangers",
        division: &AL_WEST,
    },
    Team {
        id: 141,
        code: TeamCode::Tor,
        name: "Toronto Blue Jays",
        division: &AL_EAST,
    },
    Team {
        id: 142,
        code: TeamCode::Min,
        name: "Minnesota Twins",
        division: &AL_CENTRAL,
    },
    Team {
        id: 143,
        code: TeamCode::Phi,
        name: "Philadelphia Phillies",
        division: &NL_EAST,
    },
    Team {
        id: 144,
        code: TeamCode::Atl,
        name: "Atlanta Braves",
        division: &NL_EAST,
    },
    Team {
        id: 145,
        code: TeamCode::Cws,
        name: "Chicago White Sox",
        division: &AL_CENTRAL,
    },
    Team {
        id: 146,
        code: TeamCode::Mia,
        name: "Miami Marlins",
        division: &NL_EAST,
    },
    Team {
        id: 147,
        code: TeamCode::Nyy,
        name: "New York Yankees",
        division: &AL_EAST,
    },
    Team {
        id: 158,
        code: TeamCode::Mil,
        name: "Milwaukee Brewers",
        division: &NL_CENTRAL,
    },
];
