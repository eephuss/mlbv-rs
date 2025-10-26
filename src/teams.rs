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

#[derive(Debug)]
pub struct Team {
    pub id: u32,
    pub code: &'static str,
    pub name: &'static str,
    pub division: &'static Division,
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
        code: "LAA",
        name: "Los Angeles Angels",
        division: &AL_WEST,
    },
    Team {
        id: 109,
        code: "ARI",
        name: "Arizona Diamondbacks",
        division: &NL_WEST,
    },
    Team {
        id: 110,
        code: "BAL",
        name: "Baltimore Orioles",
        division: &AL_EAST,
    },
    Team {
        id: 111,
        code: "BOS",
        name: "Boston Red Sox",
        division: &AL_EAST,
    },
    Team {
        id: 112,
        code: "CHC",
        name: "Chicago Cubs",
        division: &NL_CENTRAL,
    },
    Team {
        id: 113,
        code: "CIN",
        name: "Cincinnati Reds",
        division: &NL_CENTRAL,
    },
    Team {
        id: 114,
        code: "CLE",
        name: "Cleveland Guardians",
        division: &AL_CENTRAL,
    },
    Team {
        id: 115,
        code: "COL",
        name: "Colorado Rockies",
        division: &NL_WEST,
    },
    Team {
        id: 116,
        code: "DET",
        name: "Detroit Tigers",
        division: &AL_CENTRAL,
    },
    Team {
        id: 117,
        code: "HOU",
        name: "Houston Astros",
        division: &AL_WEST,
    },
    Team {
        id: 118,
        code: "KCR",
        name: "Kansas City Royals",
        division: &AL_CENTRAL,
    },
    Team {
        id: 119,
        code: "LAD",
        name: "Los Angeles Dodgers",
        division: &NL_WEST,
    },
    Team {
        id: 120,
        code: "WSH",
        name: "Washington Nationals",
        division: &NL_EAST,
    },
    Team {
        id: 121,
        code: "NYM",
        name: "New York Mets",
        division: &NL_EAST,
    },
    Team {
        id: 133,
        code: "OAK",       // TODO: Change code whenever they move.
        name: "Athletics", // TODO: Update/add city name too.
        division: &AL_WEST,
    },
    Team {
        id: 134,
        code: "PIT",
        name: "Pittsburgh Pirates",
        division: &NL_CENTRAL,
    },
    Team {
        id: 135,
        code: "SDP",
        name: "San Diego Padres",
        division: &NL_WEST,
    },
    Team {
        id: 136,
        code: "SEA",
        name: "Seattle Mariners",
        division: &AL_WEST,
    },
    Team {
        id: 137,
        code: "SFG",
        name: "San Francisco Giants",
        division: &NL_WEST,
    },
    Team {
        id: 138,
        code: "STL",
        name: "St. Louis Cardinals",
        division: &NL_CENTRAL,
    },
    Team {
        id: 139,
        code: "TBR",
        name: "Tampa Bay Rays",
        division: &AL_EAST,
    },
    Team {
        id: 140,
        code: "TEX",
        name: "Texas Rangers",
        division: &AL_WEST,
    },
    Team {
        id: 141,
        code: "TOR",
        name: "Toronto Blue Jays",
        division: &AL_EAST,
    },
    Team {
        id: 142,
        code: "MIN",
        name: "Minnesota Twins",
        division: &AL_CENTRAL,
    },
    Team {
        id: 143,
        code: "PHI",
        name: "Philadelphia Phillies",
        division: &NL_EAST,
    },
    Team {
        id: 144,
        code: "ATL",
        name: "Atlanta Braves",
        division: &NL_EAST,
    },
    Team {
        id: 145,
        code: "CWS",
        name: "Chicago White Sox",
        division: &AL_CENTRAL,
    },
    Team {
        id: 146,
        code: "MIA",
        name: "Miami Marlins",
        division: &NL_EAST,
    },
    Team {
        id: 147,
        code: "NYY",
        name: "New York Yankees",
        division: &AL_EAST,
    },
    Team {
        id: 158,
        code: "MIL",
        name: "Milwaukee Brewers",
        division: &NL_CENTRAL,
    },
];
