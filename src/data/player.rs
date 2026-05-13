use std::cmp::min;

use rand::prelude::IndexedRandom;
use rand::{random_range, random_ratio};
use serde::{Deserialize, Serialize};

use crate::data::contract::Contract;
use crate::data::dates::GameDate;
use crate::data::general_data::Type::{GOALIE, SKATER};
use crate::data::general_data::{NameData, PlayType, Position, Type};
use crate::data::helper::DraftStatus;
use crate::data::location::Location;
use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use crate::data::playing::{GameView, Skills};
use crate::data::projection;
use crate::data::projection::{DevelopmentCurve, Projection};
use crate::data::stats::PlayerStats;
use crate::randoms::choices;
use crate::randoms::choices::biased_random_range;
use crate::{sim, sim_engine};

pub fn random_type() -> Type {
    match random_ratio(1, 2) {
        true => SKATER,
        false => GOALIE,
    }
}

pub fn random_position(ig: bool) -> Position {
    let i = random_range(1..=6);

    match i {
        1 => Position::CENTER,
        2 => Position::RW,
        3 => Position::LW,
        4 => Position::LD,
        5 => Position::RD,
        6 => {
            if ig {
                Position::GOALIE
            } else {
                random_position(ig)
            }
        }
        _ => Position::CENTER,
    }
}

pub fn random_playtype_from_pos(pos: &Position) -> PlayType {
    match pos {
        Position::GOALIE => match random_range(1..=3) {
            1 => PlayType::BUTTERFLY,
            2 => PlayType::HYBRID,
            3 => PlayType::REACTIVE,
            _ => PlayType::HYBRID,
        },
        Position::CENTER | Position::LW | Position::RW => match random_range(1..=4) {
            1 => PlayType::PLAYMAKER,
            2 => PlayType::PWF,
            3 => PlayType::SNIPER,
            4 => PlayType::DF,
            _ => PlayType::SNIPER,
        },
        Position::RD | Position::LD => match random_range(1..=4) {
            1 => PlayType::SNIPER,
            2 => PlayType::PLAYMAKER,
            3 => PlayType::DFD,
            4 => PlayType::OFD,
            5 => PlayType::TWD,
            _ => PlayType::OFD,
        },
    }
}

pub fn random_prospect(quality: f32, goalie: bool, names: &NameData) -> Player {
    let age = random_range(17..=19);
    let overall = biased_random_range(50, 100, quality);
    let player_type = if goalie { GOALIE } else { SKATER };
    let goalie_movement = if goalie {
        Some(GoalieMovement::random(quality))
    } else {
        None
    };
    let position = random_position(goalie);
    let play_type = random_playtype_from_pos(&position);
    let projection = projection::Projection::from_quality(quality);
    let (first_name, last_name) = sim_engine::SIM_CONTEXT.get().unwrap().core().load_name_data().random_name();
    let skating = SkatingStats::random(quality, SkatingType::random());
    let view = GameView::biased(quality);
    let skills = Skills::biased(quality);
    let birth_location = random_birth_location(names, age);
    let height_cm = random_height_cm(&position);
    let weight_kg = random_weight_kg(&position, goalie);
    let mut player = Player::new(
        first_name,
        last_name,
        age,
        overall as i8,
        player_type,
        position.clone(),
        play_type,
        skating,
        goalie_movement,
        projection,
        view,
        skills,
        height_cm,
        weight_kg,
        birth_location,
        DraftStatus::undrafted(),
        default_stats_for_position(&position),
        None,
    );
    player.guess_overall();
    player
}

pub fn random_prospect_of_position(
    quality: f32,
    goalie: bool,
    pos: Position,
    names: &NameData,
) -> Player {
    let age = random_range(17..=19);
    let overall = biased_random_range(50, 100, quality);
    let player_type = if goalie { GOALIE } else { SKATER };
    let goalie_movement = if goalie {
        Some(GoalieMovement::random(quality))
    } else {
        None
    };
    let play_type = random_playtype_from_pos(&pos);
    let projection = Projection::from_quality(quality);
    let (first_name, last_name) = sim_engine::SIM_CONTEXT.get().unwrap().core().load_name_data().random_name();
    let skating = SkatingStats::random(quality, SkatingType::random());
    let view = GameView::biased(quality);
    let skills = Skills::biased(quality);
    let birth_location = random_birth_location(names, age);
    let height_cm = random_height_cm(&pos);
    let weight_kg = random_weight_kg(&pos, goalie);
    let mut player = Player::new(
        first_name,
        last_name,
        age,
        overall as i8,
        player_type,
        pos.clone(),
        play_type,
        skating,
        goalie_movement,
        projection,
        view,
        skills,
        height_cm,
        weight_kg,
        birth_location,
        DraftStatus::undrafted(),
        default_stats_for_position(&pos),
        None,
    );
    player.guess_overall();
    player
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    first_name: String,
    last_name: String,
    pub age: i8,
    overall: i8,
    player_type: Type,
    position: Position,
    play_type: PlayType,
    skate_stats: SkatingStats,
    goalie_movement: Option<GoalieMovement>,
    projection: Projection,
    view: GameView,
    skills: Skills,
    height_cm: u8,
    weight_kg: u8,
    birth_location: Location,
    draft_status: DraftStatus,
    stats: PlayerStats,
    contract: Option<Contract>,
}

impl Player {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        first_name: String,
        last_name: String,
        age: i8,
        overall: i8,
        player_type: Type,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: Option<GoalieMovement>,
        projection: Projection,
        view: GameView,
        skills: Skills,
        height_cm: u8,
        weight_kg: u8,
        birth_location: Location,
        draft_status: DraftStatus,
        stats: PlayerStats,
        contract: Option<Contract>,
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
            projection,
            view,
            skills,
            height_cm,
            weight_kg,
            birth_location,
            draft_status,
            stats,
            contract,
        }
    }



    #[allow(clippy::too_many_arguments)]
    pub fn new_skater(
        first_name: String,
        last_name: String,
        age: i8,
        overall: i8,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        projection: Projection,
        view: GameView,
        skills: Skills,
        height_cm: u8,
        weight_kg: u8,
        birth_location: Location,
    ) -> Player {
        Player::new(
            first_name,
            last_name,
            age,
            overall,
            Type::SKATER,
            position.clone(),
            play_type,
            skate_stats,
            None,
            projection,
            view,
            skills,
            height_cm,
            weight_kg,
            birth_location,
            DraftStatus::undrafted(),
            default_stats_for_position(&position),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_goalie(
        first_name: String,
        last_name: String,
        age: i8,
        overall: i8,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: GoalieMovement,
        projection: Projection,
        view: GameView,
        skills: Skills,
        height_cm: u8,
        weight_kg: u8,
        birth_location: Location,
    ) -> Player {
        Player::new(
            first_name,
            last_name,
            age,
            overall,
            Type::GOALIE,
            Position::GOALIE,
            play_type,
            skate_stats,
            Some(goalie_movement),
            projection,
            view,
            skills,
            height_cm,
            weight_kg,
            birth_location,
            DraftStatus::undrafted(),
            PlayerStats::goalie_default(),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_random_overrall_goalie(
        first_name: String,
        last_name: String,
        age: i8,
        play_type: PlayType,
        skate_stats: SkatingStats,
        goalie_movement: GoalieMovement,
        projection: Projection,
        view: GameView,
        skills: Skills,
        height_cm: u8,
        weight_kg: u8,
        birth_location: Location,
    ) -> Player {
        Self::new_goalie(
            first_name,
            last_name,
            age,
            choices::random_range_inclusive(0, 100) as i8,
            play_type,
            skate_stats,
            goalie_movement,
            projection,
            view,
            skills,
            height_cm,
            weight_kg,
            birth_location,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_random_overrall_player(
        first_name: String,
        last_name: String,
        age: i8,
        position: Position,
        play_type: PlayType,
        skate_stats: SkatingStats,
        projection: Projection,
        view: GameView,
        skills: Skills,
        height_cm: u8,
        weight_kg: u8,
        birth_location: Location,
    ) -> Player {
        Self::new_skater(
            first_name,
            last_name,
            age,
            choices::random_range_inclusive(0, 100) as i8,
            position,
            play_type,
            skate_stats,
            projection,
            view,
            skills,
            height_cm,
            weight_kg,
            birth_location,
        )
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

    pub fn height_cm(&self) -> u8 {
        self.height_cm
    }

    pub fn weight_kg(&self) -> u8 {
        self.weight_kg
    }

    pub fn birth_location(&self) -> &Location {
        &self.birth_location
    }

    pub fn draft_status(&self) -> &DraftStatus {
        &self.draft_status
    }

    pub fn set_draft_status(&mut self, draft_status: DraftStatus) {
        self.draft_status = draft_status;
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

    pub fn age_develop(&mut self, cb: i8) {
        self.develop(cb, self.age + 1);
        self.age += 1;
        self.guess_overall();
    }

    pub fn develop(&mut self, coaching_bonus: i8, age: i8) {
        let in_window = self.is_in_growth_window(age);
        let age_factor = self.age_factor(age, in_window);
        let curve_bonus = self.get_curve_bonus(age);
        let growth_pressure = self.growth_pressure(coaching_bonus);
        let total_delta = age_factor
            + curve_bonus
            + growth_pressure
            - Self::injury_penalty(self.projection().development_profile().injury_risk());
        let max_rating = self.get_max_rating();

        self.skate_stats_mut()
            .apply_delta(total_delta, total_delta, total_delta, max_rating);

        if let Some(movement) = self.goalie_movement_mut() {
            movement.apply_delta(total_delta, total_delta, total_delta, max_rating);
        }
    }

    pub fn get_max_rating(&mut self) -> i8 {
        self.projection()
            .development_profile()
            .ceiling()
            .max(self.projection().development_profile().floor())
    }

    pub fn growth_pressure(&mut self, coaching_bonus: i8) -> i8 {
        ((self.projection().development_profile().growth_rate() as i16
            + self.projection().development_profile().coachability() as i16
            + self.projection().development_profile().work_ethic() as i16
            + coaching_bonus as i16)
            / 45) as i8
    }

    pub fn get_curve_bonus(&mut self, age: i8) -> i8 {
        match self.projection().development_profile().curve() {
            DevelopmentCurve::EARLY => {
                if age <= 22 {
                    1
                } else {
                    0
                }
            }
            DevelopmentCurve::LINEAR => 0,
            DevelopmentCurve::LATE => {
                if age >= 24 {
                    1
                } else {
                    0
                }
            }
            DevelopmentCurve::BOOM_BUST => {
                if self.projection().development_profile().consistency() >= 60 {
                    1
                } else {
                    -1
                }
            }
        }
    }

    fn age_factor(&mut self, age: i8, in_window: bool) -> i8 {
        if in_window {
            2
        } else if age < self.projection().development_profile().growth_window_start() {
            1
        } else {
            -1
        }
    }

    pub fn is_in_growth_window(&mut self, age: i8) -> bool {
        age >= self.projection().development_profile().growth_window_start()
            && age <= self.projection().development_profile().growth_window_end()
    }

    pub fn coach_skating(&mut self, _coach: i8) {}

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

    pub fn guess_overall(&mut self) {
        let mut values: Vec<i32> = vec![
            self.skate_stats.speed() as i32,
            self.skate_stats.acceleration() as i32,
            self.skate_stats.edges() as i32,
            self.skills.offense() as i32,
            self.skills.defense() as i32,
            self.skills.mentality() as i32,
        ];

        if let Some(movement) = &self.goalie_movement {
            values.push(movement.push() as i32);
            values.push(movement.side() as i32);
            values.push(movement.up_down() as i32);
        }

        if let Some(avg) = Self::average(&values) {
            self.overall = avg.round() as i8;
        }
    }

    fn average(nums: &[i32]) -> Option<f64> {
        if nums.is_empty() {
            return None;
        }

        let sum: i32 = nums.iter().sum();
        Some(sum as f64 / nums.len() as f64)
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn overall(&self) -> i8 {
        self.overall
    }

    pub fn view(&self) -> &GameView {
        &self.view
    }

    pub fn skills(&self) -> &Skills {
        &self.skills
    }
}

fn default_stats_for_position(position: &Position) -> PlayerStats {
    if matches!(position, Position::GOALIE) {
        PlayerStats::goalie_default()
    } else {
        PlayerStats::skater_default()
    }
}



fn random_birth_location(names: &NameData, age: i8) -> Location {
    let year = (30_i32 - age as i32).max(0) as u32;
    let birth_date = GameDate::start_of_year(year);
    names
        .random_place()
        .map(|place| place.random_location(birth_date))
        .unwrap_or_else(|| Location::new("Unknown", "Unknown", birth_date))
}

fn random_height_cm(position: &Position) -> u8 {
    match position {
        Position::GOALIE => random_range(180..=198),
        Position::LD | Position::RD => random_range(178..=196),
        _ => random_range(170..=194),
    }
}

fn random_weight_kg(position: &Position, goalie: bool) -> u8 {
    if goalie {
        random_range(78..=105)
    } else {
        match position {
            Position::LD | Position::RD => random_range(80..=104),
            _ => random_range(72..=100),
        }
    }
}

pub fn is_goalie(player: &Player) -> bool {
    player.goalie_movement().is_some()
}

pub fn scale(value: i8, percent: f32) -> i8 {
    min((value as f32 * percent) as i8, 100)
}

pub fn develop_skating(skating: &mut SkatingStats, percent: f32) {
    skating.set_speed(scale(skating.speed(), percent));
    skating.set_acceleration(scale(skating.acceleration(), percent));
    skating.set_edges(scale(skating.edges(), percent));
}

pub fn develop_shooting(skills: &mut Skills, percent: f32) {
    skills.set_shot_accuracy(scale(skills.shot_accuracy(), percent));
    skills.set_shot_power(scale(skills.shot_power(), percent));
}

pub fn develop_offensive(skills: &mut Skills, percent: f32) {
    skills.set_offense(scale(skills.offense(), percent));
    skills.set_hands(scale(skills.hands(), percent));
    skills.set_passing(scale(skills.passing(), percent));
}

pub fn develop_defensive(skills: &mut Skills, percent: f32) {
    skills.set_defense(scale(skills.defense(), percent));
    skills.set_discipline(scale(skills.discipline(), percent));
}

pub fn develop_physical(skills: &mut Skills, percent: f32) {
    skills.set_physicality(scale(skills.physicality(), percent));
    skills.set_fighting(scale(skills.fighting(), percent));
    skills.set_durability(scale(skills.durability(), percent));
}

pub fn develop_mental(skills: &mut Skills, percent: f32) {
    skills.set_mentality(scale(skills.mentality(), percent));
    skills.set_discipline(scale(skills.discipline(), percent));
}

pub fn develop_faceoff(skills: &mut Skills, percent: f32) {
    skills.set_face_off(scale(skills.face_off(), percent));
    skills.set_mentality(scale(skills.mentality(), percent));
}
