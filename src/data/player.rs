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
pub enum DevelopmentCurve {
    EARLY,
    LINEAR,
    LATE,
    BOOM_BUST,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    player_type: Type,
    position: Position,
    play_type: PlayType,
    skate_stats: SkatingStats,
    goalie_movement: Option<GoalieMovement>,
    // Development-related fields
    ceiling: i8,
    floor: i8,
    growth_rate: i8,
    consistency: i8,
    coachability: i8,
    work_ethic: i8,
    injury_risk: i8,
    growth_window_start: i8,
    growth_window_end: i8,
    curve: DevelopmentCurve,
}

impl Player {
    pub fn new(
        player_type: Type,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: Option<GoalieMovement>,
        ceiling: i8,
        floor: i8,
        growth_rate: i8,
        consistency: i8,
        coachability: i8,
        work_ethic: i8,
        injury_risk: i8,
        growth_window_start: i8,
        growth_window_end: i8,
        curve: DevelopmentCurve,
    ) -> Player {
        Player {
            player_type,
            position,
            play_type,
            skate_stats,
            goalie_movement,
            ceiling,
            floor,
            growth_rate,
            consistency,
            coachability,
            work_ethic,
            injury_risk,
            growth_window_start,
            growth_window_end,
            curve,
        }
    }

    pub fn new_skater(
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        ceiling: i8,
        floor: i8,
        growth_rate: i8,
        consistency: i8,
        coachability: i8,
        work_ethic: i8,
        injury_risk: i8,
        growth_window_start: i8,
        growth_window_end: i8,
        curve: DevelopmentCurve,
    ) -> Player {
        Player {
            player_type: Type::SKATER,
            position,
            play_type,
            skate_stats,
            goalie_movement: None,
            ceiling,
            floor,
            growth_rate,
            consistency,
            coachability,
            work_ethic,
            injury_risk,
            growth_window_start,
            growth_window_end,
            curve,
        }
    }

    pub fn new_goalie(
        play_type: PlayType,
        skate_type: SkatingType,
        skate_stats: SkatingStats,
        goalie_movement: GoalieMovement,
        ceiling: i8,
        floor: i8,
        growth_rate: i8,
        consistency: i8,
        coachability: i8,
        work_ethic: i8,
        injury_risk: i8,
        growth_window_start: i8,
        growth_window_end: i8,
        curve: DevelopmentCurve,
    ) -> Player {
        Player {
            player_type: Type::GOALIE,
            position: Position::GOALIE,
            play_type,
            skate_stats,
            goalie_movement: Some(goalie_movement),
            ceiling,
            floor,
            growth_rate,
            consistency,
            coachability,
            work_ethic,
            injury_risk,
            growth_window_start,
            growth_window_end,
            curve,
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

    // Development-related getters
    pub fn ceiling(&self) -> i8 {
        self.ceiling
    }

    pub fn floor(&self) -> i8 {
        self.floor
    }

    pub fn growth_rate(&self) -> i8 {
        self.growth_rate
    }

    pub fn consistency(&self) -> i8 {
        self.consistency
    }

    pub fn coachability(&self) -> i8 {
        self.coachability
    }

    pub fn work_ethic(&self) -> i8 {
        self.work_ethic
    }

    pub fn injury_risk(&self) -> i8 {
        self.injury_risk
    }

    pub fn growth_window_start(&self) -> i8 {
        self.growth_window_start
    }

    pub fn growth_window_end(&self) -> i8 {
        self.growth_window_end
    }

    pub fn curve(&self) -> &DevelopmentCurve {
        &self.curve
    }

    // Development method
    pub fn develop(&mut self, coaching_bonus: i8, age: i8) {
        let in_window = age >= self.growth_window_start() && age <= self.growth_window_end();
        let age_factor = if in_window { 2 } else if age < self.growth_window_start() { 1 } else { -1 };
        let curve_bonus = match self.curve() {
            DevelopmentCurve::EARLY => {
                if age <= 22 { 1 } else { 0 }
            }
            DevelopmentCurve::LINEAR => 0,
            DevelopmentCurve::LATE => {
                if age >= 24 { 1 } else { 0 }
            }
            DevelopmentCurve::BOOM_BUST => {
                if self.consistency() >= 60 { 1 } else { -1 }
            }
        };
        let growth_pressure = ((self.growth_rate() as i16
            + self.coachability() as i16
            + self.work_ethic() as i16
            + coaching_bonus as i16)
            / 45) as i8;
        let total_delta = age_factor + curve_bonus + growth_pressure - Self::injury_penalty(self.injury_risk());
        let max_rating = self.ceiling().max(self.floor());

        self.skate_stats_mut().apply_delta(total_delta, total_delta, total_delta, max_rating);

        if let Some(movement) = self.goalie_movement_mut() {
            movement.apply_delta(total_delta, total_delta, total_delta, max_rating);
        }
    }

    fn injury_penalty(injury_risk: i8) -> i8 {
        if injury_risk >= 80 {
            2
        } else if injury_risk >= 60 {
            1
        } else {
            0
        }
    }
}
