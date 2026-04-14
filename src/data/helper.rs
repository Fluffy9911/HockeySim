use crate::data::contract::{Contract, ContractType};
use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use crate::data::player::{PlayType as PlayerPlayType, PlayType, Player, Position, Type};
use crate::data::stats::{GoalieStats, PlayerStats};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DraftStatus {
    Undrafted,
    Drafted(DraftData),
}

#[derive(Serialize, Deserialize)]
pub struct DraftData {
    draft_year: i16,
    draft_round: i8,
    overall_pick: i16,
    team: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerRecord {
    player: Player,
    stats: PlayerStats,
    contract: Option<Contract>,
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

impl PlayerRecord {
    pub fn new(

        player: Player,
        draft_status: DraftStatus,
    ) -> PlayerRecord {
        let stats = default_stats_for_player(&player);
        PlayerRecord {

            player,
            stats,
            contract: None,
        }
    }

    pub fn new_with_stats(

        player: Player,
        draft_status: DraftStatus,
        stats: PlayerStats,
    ) -> PlayerRecord {
        PlayerRecord {

            player,
            stats,
            contract: None,
        }
    }

    pub fn new_with_contract(

        player: Player,
        draft_status: DraftStatus,
        stats: PlayerStats,
        contract: Option<Contract>,
    ) -> PlayerRecord {
        PlayerRecord {

            player,
            stats,
            contract,
        }
    }


    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn stats(&self) -> &PlayerStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut PlayerStats {
        &mut self.stats
    }

    pub fn contract(&self) -> Option<&Contract> {
        self.contract.as_ref()
    }

    pub fn set_contract(&mut self, contract: Option<Contract>) {
        self.contract = contract;
    }


    fn default_stats_for_player(player: &Player) -> PlayerStats {
        if matches!(player.position(), Position::GOALIE) {
            PlayerStats::goalie_default()
        } else {
            PlayerStats::skater_default()
        }
    }

    // ... rest of the file remains unchanged
}

fn default_stats_for_player(p0: &Player) -> PlayerStats{
    PlayerStats::skater_default()
}

