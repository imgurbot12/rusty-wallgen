//! Color Gradiant Implementation

use std::{fmt::Display, str::FromStr};

use serde::{de::Error, Deserialize, Serialize};

const GRADIANT_STD: [(u8, u8); 9] = [
    (32, 50),
    (42, 46),
    (49, 40),
    (56, 39),
    (64, 38),
    (76, 37),
    (90, 33),
    (94, 29),
    (100, 20),
];
const GRADIANT_VIB: [(u8, u8); 9] = [
    (18, 99),
    (32, 97),
    (48, 95),
    (55, 90),
    (70, 80),
    (80, 70),
    (88, 60),
    (94, 40),
    (99, 24),
];
const GRADIANT_PASTEL: [(u8, u8); 9] = [
    (18, 99),
    (17, 66),
    (24, 29),
    (39, 41),
    (51, 37),
    (58, 34),
    (72, 30),
    (84, 26),
    (99, 22),
];
const GRADIANT_MONO: [(u8, u8); 9] = [
    (10, 0),
    (17, 0),
    (24, 0),
    (39, 0),
    (51, 0),
    (58, 0),
    (72, 0),
    (84, 0),
    (99, 0),
];

/// Supported Color Gradiants used for Color Generation
#[derive(Debug, Clone, PartialEq)]
pub enum Gradiant {
    Auto,
    Standard,
    Vibrant,
    Pastel,
    Mono,
}

impl Gradiant {
    pub fn gradiant(&self) -> Vec<(u8, u8)> {
        match self {
            Self::Auto => GRADIANT_STD,
            Self::Standard => GRADIANT_STD,
            Self::Vibrant => GRADIANT_VIB,
            Self::Pastel => GRADIANT_PASTEL,
            Self::Mono => GRADIANT_MONO,
        }
        .to_vec()
    }
}

impl Display for Gradiant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Auto => "auto",
                Self::Standard => "standard",
                Self::Vibrant => "vibrant",
                Self::Pastel => "pastel",
                Self::Mono => "mono",
            }
        )
    }
}

impl FromStr for Gradiant {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "standard" => Ok(Self::Standard),
            "vibrant" => Ok(Self::Vibrant),
            "pastel" => Ok(Self::Pastel),
            "mono" => Ok(Self::Mono),
            _ => Err(format!("invalid palette: {s:?}")),
        }
    }
}

impl Serialize for Gradiant {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = self.to_string();
        serializer.collect_str(&s)
    }
}

impl<'de> Deserialize<'de> for Gradiant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Gradiant::from_str(s).map_err(D::Error::custom)
    }
}
