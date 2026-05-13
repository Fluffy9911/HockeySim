use std::fs;
use std::io::Write;
use HockeySim::league_settings::League;
use HockeySim::{sim, sim_engine};
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::general_data::Position;
use HockeySim::data::player::{random_playtype_from_pos, random_position};
use HockeySim::data::{player, team};
use HockeySim::data::location::Places;
use HockeySim::savestate::savedata::SaveInfo;
fn main() {


    
let mut info = SaveInfo::new();
    sim_engine::begin_sim("Test_Sim");
    let mut nd = sim_engine::SIM_CONTEXT.get().unwrap().core.load_name_data();

    nd.add_place(Places::new("Canada"));

    //nd.save("HockeySim_names");



let nhl = League::empty("NHL".parse().unwrap());

    let mut sim = sim_engine::Sim::create_or_load(sim_engine::SIM_CONTEXT.get().unwrap());

    let lid = sim.ensure_league(nhl);

    let team = team::Team::new(/* String */"Edmonton".to_string(), /* String */"Oilers".to_string(), /* String */"EDM".to_string(), /* Conference */Conference::West, /* Division */Division::Pacific, /* Vec<Player> */Vec::new(), /* Vec<StaffMember> */Vec::new());

    sim.get_league(lid as i32).add_team(team);



    let player = player::random_prospect_of_position(0.75, false, random_position(false), &sim_engine::SIM_CONTEXT.get().unwrap().core.load_name_data());

    sim.get_league(lid as i32).add_new_free_agent(player);

sim.save_data(sim_engine::SIM_CONTEXT.get().unwrap());
}


fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
