use rand::RngExt;
use serde::{Deserialize, Serialize};
use crate::data::dates::GameDate;
use crate::data::location::Location;
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