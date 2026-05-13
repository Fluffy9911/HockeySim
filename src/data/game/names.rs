use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Conference {
    East,
    West,
    Custom(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Division {
    Atlantic,
    Pacific,
    Custom { name: String, team_count: Option<i32> },
}

impl Conference {
    pub fn east() -> Conference {
        Conference::East
    }

    pub fn west() -> Conference {
        Conference::West
    }

    pub fn custom(name: impl Into<String>) -> Conference {
        Conference::Custom(name.into())
    }

    pub fn name(&self) -> &str {
        match self {
            Conference::East => "East",
            Conference::West => "West",
            Conference::Custom(name) => name.as_str(),
        }
    }
}

impl Division {
    pub fn atlantic() -> Division {
        Division::Atlantic
    }

    pub fn pacific() -> Division {
        Division::Pacific
    }

    pub fn custom(name: impl Into<String>, team_count: Option<i32>) -> Division {
        Division::Custom {
            name: name.into(),
            team_count,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Division::Atlantic => "Atlantic",
            Division::Pacific => "Pacific",
            Division::Custom { name, .. } => name.as_str(),
        }
    }

    pub fn team_count(&self) -> Option<i32> {
        match self {
            Division::Atlantic | Division::Pacific => Some(8),
            Division::Custom { team_count, .. } => *team_count,
        }
    }
}
