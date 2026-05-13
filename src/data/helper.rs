use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum DraftStatus {
    Undrafted,
    Drafted(DraftData),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DraftData {
    draft_year: i16,
    draft_round: i8,
    overall_pick: i16,
    team: String,
}

impl DraftData {
    pub fn new(draft_year: i16, draft_round: i8, overall_pick: i16, team: String) -> DraftData {
        DraftData {
            draft_year,
            draft_round,
            overall_pick,
            team,
        }
    }

    pub fn draft_year(&self) -> i16 {
        self.draft_year
    }

    pub fn draft_round(&self) -> i8 {
        self.draft_round
    }

    pub fn overall_pick(&self) -> i16 {
        self.overall_pick
    }

    pub fn team(&self) -> &str {
        &self.team
    }
}

impl DraftStatus {
    pub fn undrafted() -> DraftStatus {
        DraftStatus::Undrafted
    }

    pub fn drafted(draft_data: DraftData) -> DraftStatus {
        DraftStatus::Drafted(draft_data)
    }
}
