use std::fs;
use std::io::Write;
use HockeySim::data::player;
use HockeySim::data::draft::Draft;
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::player::{random_prospect_of_position, Player, Position};
use HockeySim::data::stats::PlayerStats;
use HockeySim::data::team::{Team, TeamIdentity};
use HockeySim::{league_settings, sim};
use HockeySim::savestate::savestate;
use HockeySim::savestate::savestate::{CoreConfig, FileType};
use HockeySim::testing::league_helper;
fn main() {
let core_data = CoreConfig{sim_id: "HockeySim".parse().unwrap(),data_id: "Sim".parse().unwrap(),game_id: "TestGame".parse().unwrap(),version_id: "0.10".parse().unwrap() };

    savestate::ensure_dir_type(&core_data, &FileType::CORE_DATA);

    savestate::ensure_dir_type(&core_data, &FileType::TEAM_DATA);

    savestate::ensure_dir_type(&core_data, &FileType::PLAYER_DATA);

    savestate::ensure_dir_type(&core_data, &FileType::LEAGUE_DATA);

    savestate::ensure_dir_type(&core_data, &FileType::SAVE_DATA);

    let name_data = core_data.load_name_data();

    let line_a = player::generate_prospect_line(0.5);

    let pairs_a = player::generate_prospect_line(0.5);

    let line_b = vec![player::random_prospect_of_position(0.0,false,Position::LW),random_prospect_of_position(0.75,false,Position::CENTER),random_prospect_of_position(0.0,false,Position::RW)];

    let pairs_b = vec![random_prospect_of_position(0.2,false,Position::LD),random_prospect_of_position(0.2,false,Position::RD)]   ;

    sim::simulate_matchup(&*line_a, &*pairs_a, &*line_b, &*pairs_b);

   // savestate::write_structs(&core_data, &FileType::PLAYER_DATA, vec![("test.json".parse().unwrap(), player::random_prospect_of_position(0.5, false, Position::CENTER))]).expect("REASON")

    savestate::write_struct(&core_data, &FileType::PLAYER_DATA, "testplayer.json", &line_a[0]);

}



fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
