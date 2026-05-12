use std::fs;
use std::io::Write;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use rand::prelude::IndexedRandom;
use crate::data::dates::GameDate;
use crate::data::location::{Location, Places};
use crate::league_settings::SimulationEngine;

#[derive(Serialize,Deserialize)]
pub struct General{
  flags:u8,
  height:u8,
    weight:u8,
    birth:Location
}




pub trait Growable{

    fn grow_month(&mut self,id_a:u64,id_b:u64,id_c:u64, simulation_engine: SimulationEngine);

    fn grow_year(&mut self,id_a:u64,id_b:u64,id_c:u64, simulation_engine: SimulationEngine);


}

impl General {

pub fn new(height:u8,weight:u8,flags:u8,birth:Location)-> General{
  General {flags , height, weight, birth }
}





pub fn can_grow(&self) -> bool {

    let mask = 1 << 5;
    (self.flags & mask) == (GROWING & mask)

}


    pub fn check_growth(&self,g:u8) -> bool {

        let mask = 1 << 6;

        (self.flags & mask) == g


    }


}


impl Growable for General {
    fn grow_month(&mut self, id_a: u64, id_b: u64, id_c: u64, simulation_engine: SimulationEngine) {

            let can_grow = self.can_grow();

            if !can_grow {
                return
            }

            let mut factor = rand::rng().random_range(1.0..=1.5);

            if self.check_growth(GROWTH_HIGH) {

    self.height = (self.height as f64 * factor) as u8;
                factor = rand::rng().random_range(1.0..=1.5);
     self.weight = (self.weight as f64 * factor) as u8;






        }else{

                factor = rand::rng().random_range(1.0..=1.15);
                self.height = (self.height as f64 * factor) as u8;
                factor = rand::rng().random_range(1.0..=1.15);
                self.weight = (self.weight as f64 * factor) as u8;


            }
    }

    fn grow_year(&mut self, id_a: u64, id_b: u64, id_c: u64, simulation_engine: SimulationEngine) {

    }
}


impl PartialEq for GameDate {
    fn eq(&self, other: &Self) -> bool {
        (self.day == other.day) && (self.month == other.month) && (self.year == other.year)



    }
}



static LEFT_SHOT: u8 = 0b0_0000000;
static RIGHT_SHOT: u8 = 0b01_0000000;

static GROWTH_HIGH:u8 = 0b0_10_00000;
static GROWTH_LOW: u8 = 0b0_00_00000;
static GROWING: u8 = 0b0_01_00000;
static NOT_GROWING: u8 = GROWTH_LOW;

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
    team_names: Vec<String>,
    places: Vec<Places>




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

impl NameData {
    pub fn new()-> NameData{ NameData{first_names:Vec::new(),last_names: Vec::new(),team_names: Vec::new(), places: Vec::new()} }
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

    pub fn add_place(&mut self, place :Places){

        self.places.push(place);

    }

    pub fn random_place(&self) -> Option<&Places> {

        self.places.choose(&mut rand::rng())

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