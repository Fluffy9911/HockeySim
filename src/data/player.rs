use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Type {
    SKATER,
    GOALIE,
}

#[derive(Serialize, Deserialize)]
pub enum Position {
    CENTER,
    LW,
    RW,
    RD,
    LD,
    GOALIE,
}

#[derive(Serialize, Deserialize)]
pub enum PlayType {
    SNIPER,
    OFD,
    DFD,
    PWF,
    DF,
    PLAYMAKER,
    BUTTERFLY,
    REACTIVE,
    HYBRID,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    player_type: Type,
    position: Position,
    play_type: PlayType,
    skate_stats: SkatingStats,
    goalie_movement: Option<GoalieMovement>,
}

impl Player {
    pub fn new(
        player_type: Type,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: Option<GoalieMovement>,
    ) -> Player {
        Player {
            player_type,
            position,
            play_type,

            skate_stats,
            goalie_movement,
        }
    }

    pub fn new_skater(
        position: Position,
        play_type: PlayType,

        skate_stats: SkatingStats,
    ) -> Player {
        Player {
            player_type: Type::SKATER,
            position,
            play_type,

            skate_stats,
            goalie_movement: None,
        }
    }

    pub fn new_goalie(
        play_type: PlayType,
        skate_type: SkatingType,
        skate_stats: SkatingStats,
        goalie_movement: GoalieMovement,
    ) -> Player {
        Player {
            player_type: Type::GOALIE,
            position: Position::GOALIE,
            play_type,

            skate_stats,
            goalie_movement: Some(goalie_movement),
        }
    }

    pub fn player_type(&self) -> &Type {
        &self.player_type
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn play_type(&self) -> &PlayType {
        &self.play_type
    }

  

    pub fn skate_stats(&self) -> &SkatingStats {
        &self.skate_stats
    }

    pub fn skate_stats_mut(&mut self) -> &mut SkatingStats {
        &mut self.skate_stats
    }

    pub fn goalie_movement(&self) -> Option<&GoalieMovement> {
        self.goalie_movement.as_ref()
    }

    pub fn goalie_movement_mut(&mut self) -> Option<&mut GoalieMovement> {
        self.goalie_movement.as_mut()
    }
}
