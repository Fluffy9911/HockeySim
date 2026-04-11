use std::fs;
use std::io::Write;
use std::option::Option;
use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use serde::{Deserialize, Serialize};
use crate::data::projection::Projection;
use crate::data::projection::DevelopmentCurve;
use crate::randoms::choices;
use rand::{random_range, random_ratio, rng};
use rand::prelude::IndexedRandom;
use serde::de::Unexpected::Option as OtherOption;
use crate::data::player::Type::{GOALIE, SKATER};
use crate::data::projection;
use crate::randoms::choices::biased_random_range;

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
pub struct NameData{
    first_names: Vec<String>,
    last_names: Vec<String>,
    team_names: Vec<String>




}

#[derive(Serialize, Deserialize)]
pub enum PlayType {
    SNIPER,
    OFD,
    DFD,
    PWF,
    DF,
    TWD,
    PLAYMAKER,
    BUTTERFLY,
    REACTIVE,
    HYBRID,
}

pub fn random_type() -> Type{

    let r = random_ratio(1,2);
    match r {
        true => {

            SKATER

        }
        false =>{

            GOALIE
        }


    }
}


pub fn random_position(ig:bool)-> Position{

    let i = random_range(1..=6);

    match i {
        1=> Position::CENTER,
        2=> Position::RW,
        3=>Position::LW,
        4=>Position::LD,
        5=>Position::RD,
        6=>{
            if ig {
                Position::GOALIE
            }else{
                //re-roll so we don't bias
                random_position(ig)

            }
        }

        _ => {
            Position::CENTER
        }
    }


}

pub fn random_playtype_from_pos(pos: &Position) -> PlayType{

    match pos {

        Position::GOALIE => {
            let g = random_range(1..=3);
            match g {
                1=> PlayType::BUTTERFLY,
                2=>PlayType::HYBRID,
                3=>PlayType::REACTIVE,

                _ => {PlayType::HYBRID}
            }

        }
        (Position::CENTER|Position::LW|Position::RW)=>{
            let p = random_range(1..=4);

match p {

    1=> PlayType::PLAYMAKER,
    2=>PlayType::PWF,
    3=>PlayType::SNIPER,
    4=>PlayType::DF,
    _=>PlayType::SNIPER
}

        }

        (Position::RD|Position::LD) =>{

            let d = random_range(1..=4);

            match d {
                1=>PlayType::SNIPER,
                2=>PlayType::PLAYMAKER,
                3=>PlayType::DFD,
                4=>PlayType::OFD,
                5=>PlayType::TWD,
_=>PlayType::OFD


            }


        }



    }



}

pub fn random_prospect(quality:f32,goalie:bool)-> Player {
    let age = random_range(17..=19);
    let overall = biased_random_range(50,100,quality);
    let pt: Type;
    let mut gm: Option<GoalieMovement> = None;
    if goalie {
        pt = GOALIE;
        gm = Option::Some(GoalieMovement::random(quality));
    }
    else{
        pt = SKATER;
    }
    let pos = random_position(goalie);
    let play = random_playtype_from_pos(&pos);
    let proj = projection::Projection::from_quality(quality);
let name = Player::random_name();
    let skat_type = SkatingType::random();
    let skating = SkatingStats::random(quality,skat_type);

    let mut p = Player::new(name.0, name.1, /* i8 */age, /* i8 */overall as i8, /* Type */pt, /* Position */pos, /* player::PlayType */play, /* SkatingStats */skating, /* std::option::Option<GoalieMovement> */gm, /* Projection */proj);
p.guess_overall();
    p


}

#[derive(Serialize, Deserialize)]
pub struct Player {
    first_name: String,
    last_name: String,
    pub age:i8,
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
    pub fn overall(&self) -> i8 {
       self.overall
    }
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
    pub fn random_name() -> (String, String) {
        let first_names = [
            "Alice", "Bob", "Charlie", "Diana", "Ethan",
            "Fiona", "George", "Hannah", "Ian", "Julia"
        ];

        let last_names = [
            "Smith", "Johnson", "Williams", "Brown", "Jones",
            "Garcia", "Miller", "Davis", "Martinez", "Taylor"
        ];

        let mut rng = rand::rng();

        let first = first_names.choose(&mut rng).unwrap().to_string();
        let last = last_names.choose(&mut rng).unwrap().to_string();

        (first, last)
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


    pub fn age_develop(&mut self,cb:i8){

        self.develop(cb,self.age+1);
        self.age=self.age+1;
        self.guess_overall();
    }
    pub fn develop(&mut self, coaching_bonus: i8, age: i8) {
        let in_window = self.is_in_growth_window(age);
        let age_factor = self.age_factor(age, in_window);
        let curve_bonus = self.get_curve_bonus(age);
        let growth_pressure = self.growth_pressure(coaching_bonus);
        let total_delta = age_factor + curve_bonus + growth_pressure - Self::injury_penalty(self.projection().development_profile().injury_risk());
        let max_rating = self.get_max_rating();

        self.skate_stats_mut().apply_delta(total_delta, total_delta, total_delta, max_rating);

        if let Some(movement) = self.goalie_movement_mut() {
            movement.apply_delta(total_delta, total_delta, total_delta, max_rating);
        }
    }

    pub fn get_max_rating(&mut self) -> i8 {
        self.projection().development_profile().ceiling().max(self.projection().development_profile().floor())
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
                if age <= 22 { 1 } else { 0 }
            }
            DevelopmentCurve::LINEAR => 0,
            DevelopmentCurve::LATE => {
                if age >= 24 { 1 } else { 0 }
            }
            DevelopmentCurve::BOOM_BUST => {
                if self.projection().development_profile().consistency() >= 60 { 1 } else { -1 }
            }
        }
    }

    fn age_factor(&mut self, age: i8, in_window: bool) -> i8 {
        if in_window { 2 } else if age < self.projection().development_profile().growth_window_start() { 1 } else { -1 }
    }

    pub fn is_in_growth_window(&mut self, age: i8) -> bool {
        age >= self.projection().development_profile().growth_window_start() && age <= self.projection().development_profile().growth_window_end()
    }

    pub fn coach_skating(&mut self,coach: i8){
        let age = self.is_in_growth_window(self.age);
       
        if age {





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


    pub fn guess_overall(&mut self) {
        let mut values: Vec<i32> = Vec::new();

        // Collect skating stats (adjust these based on your actual struct fields)
        values.push(self.skate_stats.speed() as i32);
        values.push(self.skate_stats.acceleration() as i32);
        values.push(self.skate_stats.edges() as i32);

        // If goalie, include goalie movement stats
        if let Some(movement) = &self.goalie_movement {
            values.push(movement.push() as i32);
            values.push(movement.side() as i32);
            values.push(movement.up_down() as i32);
        }

        // Compute average
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

    pub fn name(&self)-> String{
        format!("{}{}",self.first_name,self.last_name)
        
    }
    
    
}



impl NameData {
    pub fn new()-> NameData{ NameData{first_names:Vec::new(),last_names: Vec::new(),team_names: Vec::new()} }
    fn get_path(name: &str) -> String {
        format!("data/NameData/{}.json", name)
    }

    fn ensure_dir() {
        let path = "data/NameData";
        if let Err(e) = fs::create_dir_all(path) {
            println!("Error creating directory: {:?}", e);
        }
    }

    pub fn save(&self, name: &str) {
        Self::ensure_dir();

        let file_path = Self::get_path(name);

        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
        {
            Ok(mut file) => {
                if let Ok(json) = serde_json::to_string_pretty(self) {
                    let _ = file.write_all(json.as_bytes());
                }
            }
            Err(e) => {
                println!("Error opening file {}: {:?}", file_path, e);
            }
        }
    }

    pub fn load(name: &str) -> Option<NameData> {
        Self::ensure_dir();

        let file_path = Self::get_path(name);

        match fs::read_to_string(&file_path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(data) => Some(data),
                    Err(e) => {
                        println!("JSON parse error: {:?}", e);
                        None
                    }
                }
            }
            Err(_) => None
        }
    }

    pub fn read_or_new(name: &str) -> NameData {
        Self::load(name).unwrap_or_else(|| {
            let data = NameData::new();
            data.save(name);
            data
        })
    }

    // ---------------------------
    // Data manipulation functions
    // ---------------------------

    pub fn add_first_name(&mut self, name: String) {
        self.first_names.push(name);
    }

    pub fn add_last_name(&mut self, name: String) {
        self.last_names.push(name);
    }

    pub fn add_team_name(&mut self, name: String) {
        self.team_names.push(name);
    }

    pub fn remove_first_name(&mut self, name: &str) {
        self.first_names.retain(|n| n != name);
    }

    pub fn remove_last_name(&mut self, name: &str) {
        self.last_names.retain(|n| n != name);
    }

    pub fn remove_team_name(&mut self, name: &str) {
        self.team_names.retain(|n| n != name);
    }

    pub fn clear_all(&mut self) {
        self.first_names.clear();
        self.last_names.clear();
        self.team_names.clear();
    }

    // ---------------------------
    // Query helpers
    // ---------------------------

    pub fn random_first_name(&self) -> Option<&String> {

        let mut rng = rand::rng();
        self.first_names.choose(&mut rng)
    }

    pub fn random_last_name(&self) -> Option<&String> {

        let mut rng = rand::rng();
        self.last_names.choose(&mut rng)
    }

    pub fn random_team_name(&self) -> Option<&String> {

        let mut rng = rand::rng();
        self.team_names.choose(&mut rng)
    }

    pub fn random_full_name(&self) -> Option<String> {
        match (self.random_first_name(), self.random_last_name()) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            _ => None
        }
    }

    // ---------------------------
    // Utility
    // ---------------------------

    pub fn exists(name: &str) -> bool {
        let file_path = Self::get_path(name);
        fs::metadata(file_path).is_ok()
    }

    pub fn delete(name: &str) {
        let file_path = Self::get_path(name);
        if let Err(e) = fs::remove_file(file_path) {
            println!("Error deleting file: {:?}", e);
        }
    }

    pub fn list_files() -> Vec<String> {
        let path = "data/NameData";
        let mut result = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    if let Some(name_str) = name.to_str() {
                        result.push(name_str.to_string());
                    }
                }
            }
        }

        result
    }
    pub fn add_first_names(&mut self, names: Vec<String>) {
        self.first_names.extend(names);
    }

    pub fn add_last_names(&mut self, names: Vec<String>) {
        self.last_names.extend(names);
    }

    pub fn add_team_names(&mut self, names: Vec<String>) {
        self.team_names.extend(names);
    }
}