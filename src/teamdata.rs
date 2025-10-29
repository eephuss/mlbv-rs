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
pub struct Division {
    pub name: DivisionRegion,
    pub league: League,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TeamCode {
    ARI,
    ATH,
    ATL,
    BAL,
    BOS,
    CHC,
    CIN,
    CWS,
    CLE,
    COL,
    DET,
    HOU,
    KCR,
    LAA,
    LAD,
    MIL,
    MIN,
    MIA,
    NYY,
    NYM,
    // OAK,
    PHI,
    PIT,
    SDP,
    SEA,
    SFG,
    STL,
    TBR,
    TEX,
    TOR,
    WSH,
}

impl std::str::FromStr for TeamCode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ARI" => Ok(Self::ARI),
            "ATH" => Ok(Self::ATH),
            "ATL" => Ok(Self::ATL),
            "BAL" => Ok(Self::BAL),
            "BOS" => Ok(Self::BOS),
            "CHC" => Ok(Self::CHC),
            "CIN" => Ok(Self::CIN),
            "CWS" => Ok(Self::CWS),
            "CLE" => Ok(Self::CLE),
            "COL" => Ok(Self::COL),
            "DET" => Ok(Self::DET),
            "HOU" => Ok(Self::HOU),
            "KCR" => Ok(Self::KCR),
            "LAA" => Ok(Self::LAA),
            "LAD" => Ok(Self::LAD),
            "MIL" => Ok(Self::MIL),
            "MIN" => Ok(Self::MIN),
            "MIA" => Ok(Self::MIA),
            "NYY" => Ok(Self::NYY),
            "NYM" => Ok(Self::NYM),
            "PHI" => Ok(Self::PHI),
            "PIT" => Ok(Self::PIT),
            "SDP" => Ok(Self::SDP),
            "SEA" => Ok(Self::SEA),
            "SFG" => Ok(Self::SFG),
            "STL" => Ok(Self::STL),
            "TBR" => Ok(Self::TBR),
            "TEX" => Ok(Self::TEX),
            "TOR" => Ok(Self::TOR),
            "WSH" => Ok(Self::WSH),
            "OAK" => anyhow::bail!("The OAK code has been retired and replaced with ATH"),
            _ => anyhow::bail!("Invalid team code: {s}")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Team {
    pub id: u32,
    pub code: TeamCode,
    pub name: &'static str,
    pub division: &'static Division,
}

impl Team {
    /// Find a team by its code (e.g., "wsh", "tor")
    pub fn find_by_code(code: TeamCode) -> Option<&'static Team> {
        TEAMS.iter().find(|team| team.code == code)
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
        code: TeamCode::LAA,
        name: "Los Angeles Angels",
        division: &AL_WEST,
    },
    Team {
        id: 109,
        code: TeamCode::ARI,
        name: "Arizona Diamondbacks",
        division: &NL_WEST,
    },
    Team {
        id: 110,
        code: TeamCode::BAL,
        name: "Baltimore Orioles",
        division: &AL_EAST,
    },
    Team {
        id: 111,
        code: TeamCode::BOS,
        name: "Boston Red Sox",
        division: &AL_EAST,
    },
    Team {
        id: 112,
        code: TeamCode::CHC,
        name: "Chicago Cubs",
        division: &NL_CENTRAL,
    },
    Team {
        id: 113,
        code: TeamCode::CIN,
        name: "Cincinnati Reds",
        division: &NL_CENTRAL,
    },
    Team {
        id: 114,
        code: TeamCode::CLE,
        name: "Cleveland Guardians",
        division: &AL_CENTRAL,
    },
    Team {
        id: 115,
        code: TeamCode::COL,
        name: "Colorado Rockies",
        division: &NL_WEST,
    },
    Team {
        id: 116,
        code: TeamCode::DET,
        name: "Detroit Tigers",
        division: &AL_CENTRAL,
    },
    Team {
        id: 117,
        code: TeamCode::HOU,
        name: "Houston Astros",
        division: &AL_WEST,
    },
    Team {
        id: 118,
        code: TeamCode::KCR,
        name: "Kansas City Royals",
        division: &AL_CENTRAL,
    },
    Team {
        id: 119,
        code: TeamCode::LAD,
        name: "Los Angeles Dodgers",
        division: &NL_WEST,
    },
    Team {
        id: 120,
        code: TeamCode::WSH,
        name: "Washington Nationals",
        division: &NL_EAST,
    },
    Team {
        id: 121,
        code: TeamCode::NYM,
        name: "New York Mets",
        division: &NL_EAST,
    },
    Team {
        id: 133,
        code: TeamCode::ATH, // TODO: Change code if they ever make it to Vegas. LVA?
        name: "Athletics",   // TODO: Update/re-add city name too.
        division: &AL_WEST,
    },
    Team {
        id: 134,
        code: TeamCode::PIT,
        name: "Pittsburgh Pirates",
        division: &NL_CENTRAL,
    },
    Team {
        id: 135,
        code: TeamCode::SDP,
        name: "San Diego Padres",
        division: &NL_WEST,
    },
    Team {
        id: 136,
        code: TeamCode::SEA,
        name: "Seattle Mariners",
        division: &AL_WEST,
    },
    Team {
        id: 137,
        code: TeamCode::SFG,
        name: "San Francisco Giants",
        division: &NL_WEST,
    },
    Team {
        id: 138,
        code: TeamCode::STL,
        name: "St. Louis Cardinals",
        division: &NL_CENTRAL,
    },
    Team {
        id: 139,
        code: TeamCode::TBR,
        name: "Tampa Bay Rays",
        division: &AL_EAST,
    },
    Team {
        id: 140,
        code: TeamCode::TEX,
        name: "Texas Rangers",
        division: &AL_WEST,
    },
    Team {
        id: 141,
        code: TeamCode::TOR,
        name: "Toronto Blue Jays",
        division: &AL_EAST,
    },
    Team {
        id: 142,
        code: TeamCode::MIN,
        name: "Minnesota Twins",
        division: &AL_CENTRAL,
    },
    Team {
        id: 143,
        code: TeamCode::PHI,
        name: "Philadelphia Phillies",
        division: &NL_EAST,
    },
    Team {
        id: 144,
        code: TeamCode::ATL,
        name: "Atlanta Braves",
        division: &NL_EAST,
    },
    Team {
        id: 145,
        code: TeamCode::CWS,
        name: "Chicago White Sox",
        division: &AL_CENTRAL,
    },
    Team {
        id: 146,
        code: TeamCode::MIA,
        name: "Miami Marlins",
        division: &NL_EAST,
    },
    Team {
        id: 147,
        code: TeamCode::NYY,
        name: "New York Yankees",
        division: &AL_EAST,
    },
    Team {
        id: 158,
        code: TeamCode::MIL,
        name: "Milwaukee Brewers",
        division: &NL_CENTRAL,
    },
];
