use std::fs;
use std::io::Write;
use HockeySim::data::player;
use HockeySim::data::draft::Draft;
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::player::{random_prospect_of_position, NameData, Player, Position};
use HockeySim::data::stats::PlayerStats;
use HockeySim::data::team::{Team, TeamIdentity};
use HockeySim::{league_settings, sim};
use HockeySim::savestate::savedata;
use HockeySim::savestate::savedata::{CoreConfig, FileType, SaveInfo};
use HockeySim::testing::league_helper;
fn main() {


    
let mut info = SaveInfo::new();


  let mut config = info.create_save("test");

savedata::create_initial_state(&mut config);

}


fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
