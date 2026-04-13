use std::fs;
use std::io::Write;
use HockeySim::data::player;
use HockeySim::data::draft::Draft;
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::stats::PlayerStats;
use HockeySim::data::team::{Team, TeamIdentity};
use HockeySim::league_settings;
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



}





fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
