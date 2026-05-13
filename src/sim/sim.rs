use crate::data::dates::GameDate;
use crate::league_settings::League;
use crate::savestate::savedata::{FileType, SaveContext};
use crate::sim_engine::Sim;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SimDiskData {
    current_date: GameDate,
    league_names: Vec<String>,
}

impl Sim {

    pub fn create_or_load(ctx:&SaveContext) -> Sim {

if ctx.exists(FileType::LEAGUE_DATA,"Sim.json"){

let sim_data: SimDiskData = ctx.read_struct(FileType::LEAGUE_DATA,"Sim.json").unwrap();
let leagues = sim_data
    .league_names
    .iter()
    .map(|league_name| League::load_from_context(ctx, league_name).unwrap())
    .collect();

    Sim { current_date: sim_data.current_date, leagues }

}else {

    let sim = Sim{current_date: GameDate::new(2025,1,1).unwrap(),leagues: Vec::new()};
    let sim_data = SimDiskData { current_date: sim.current_date, league_names: Vec::new() };

    ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(&sim_data).unwrap()).expect("Error");

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
        for league in &self.leagues {
            league.save_to_context(ctx).expect("Error");
        }

        let sim_data = SimDiskData {
            current_date: self.current_date,
            league_names: self.leagues.iter().map(|league| league.name().to_string()).collect(),
        };

        ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(&sim_data).unwrap()).expect("Error");

    }



}
