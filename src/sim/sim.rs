use crate::data::dates::GameDate;
use crate::league_settings::League;
use crate::savestate::savedata::{FileType, SaveContext};
use crate::sim_engine::Sim;

impl Sim {

    pub fn create_or_load(ctx:&SaveContext) -> Sim {

if ctx.exists(FileType::LEAGUE_DATA,"Sim.json"){

let sim = ctx.read_struct(FileType::LEAGUE_DATA,"Sim.json").unwrap();

    sim

}else {

    let sim = Sim{current_date: GameDate::new(2025,1,1).unwrap(),leagues: Vec::new()};

    ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(&sim).unwrap()).expect("Error");

    sim

}


    }


    pub fn add_league(&mut self,league:League){

        self.leagues.push(league);


    }

    pub fn advance_date(&mut self){
     self.current_date =   self.current_date.add_days(30);

    }

    pub fn save_data(&self,ctx:&SaveContext){
        ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(self).unwrap()).expect("Error");

    }



}