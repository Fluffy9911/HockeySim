use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use serde::{Deserialize, Serialize};
use crate::data::projection::Projection;
use crate::data::projection::DevelopmentCurve;
use crate::randoms::choices;

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
    first_name: String,
    last_name: String,
    age:i8,
    overall:i8,
    player_type: Type,
    position: Position,
    play_type: PlayType,
    skate_stats: SkatingStats,
    goalie_movement: Option<GoalieMovement>,

    projection:
    Projection,
}

impl Player {
    pub fn new(first_name: String, last_name: String, age:i8,overall:i8,
        player_type: Type,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: Option<GoalieMovement>,
        projection: Projection
    ) -> Player {
        Player {
            first_name,
            last_name,
            age,
            overall,
            player_type,
            position,
            play_type,
            skate_stats,
            goalie_movement,
            projection
        }
    }

    pub fn new_skater(first_name: String, last_name: String, age:i8,overall:i8,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,

       projection: Projection
    ) -> Player {
        Player {
            first_name,
            last_name,
            age,
            overall,
            player_type: Type::SKATER,
            position,
            play_type,
            skate_stats,
            goalie_movement: None,
           projection
        }
    }

    pub fn new_goalie(first_name: String, last_name: String, age:i8,overall:i8,
        play_type: PlayType,
        skate_type: SkatingType,
        skate_stats: SkatingStats,
        goalie_movement: GoalieMovement,

        projection: Projection
    ) -> Player {
        Player {
            first_name,
            last_name,
            age,
            overall,
            player_type: Type::GOALIE,
            position: Position::GOALIE,
            play_type,
            skate_stats,
            goalie_movement: Some(goalie_movement),

            projection
        }
    }
    pub fn new_random_overrall_goalie(first_name: String, last_name: String, age:i8,
                                      play_type: PlayType,
                                      skate_type: SkatingType,
                                      skate_stats: SkatingStats,
                                      goalie_movement: GoalieMovement,

                                      projection: Projection)-> Player {

        Self::new_goalie(first_name, last_name, age, choices::random_range_inclusive(0, 100) as i8, play_type, skate_type, skate_stats, goalie_movement, projection)
    }
    pub fn new_random_overrall_player(first_name: String, last_name: String, age:i8,
                                      position: Position,
                                      play_type: PlayType,
                                      skate_stats: SkatingStats,




                                      projection: Projection)-> Player {

        Self::new_skater(first_name, last_name, age, choices::random_range_inclusive(0, 100) as i8, position,play_type, skate_stats, projection)
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


    pub fn develop(&mut self, coaching_bonus: i8, age: i8) {
        let in_window = age >= self.projection().development_profile().growth_window_start() && age <= self.projection().development_profile().growth_window_end();
        let age_factor = if in_window { 2 } else if age < self.projection().development_profile().growth_window_start() { 1 } else { -1 };
        let curve_bonus = match self.projection().development_profile().curve() {
            DevelopmentCurve::EARLY => {
                if age <= 22 { 1 } else { 0 }
            }
            DevelopmentCurve::LINEAR => 0,
            DevelopmentCurve::LATE => {
                if age >= 24 { 1 } else { 0 }
            }
            DevelopmentCurve::BOOM_BUST => {
                if self.projection().development_profile().consistency() >= 60 { 1 } else { -1 }
            }
        };
        let growth_pressure = ((self.projection().development_profile().growth_rate() as i16
            + self.projection().development_profile().coachability() as i16
            + self.projection().development_profile().work_ethic() as i16
            + coaching_bonus as i16)
            / 45) as i8;
        let total_delta = age_factor + curve_bonus + growth_pressure - Self::injury_penalty(self.projection().development_profile().injury_risk());
        let max_rating = self.projection().development_profile().ceiling().max(self.projection().development_profile().floor());

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

    pub fn projection(&self) -> &Projection {

        &self.projection

    }

}
