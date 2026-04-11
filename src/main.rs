use std::fs;
use std::io::Write;
use HockeySim::data::player;
use HockeySim::data::draft::Draft;
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::stats::PlayerStats;
use HockeySim::data::team::{Team, TeamIdentity};
use HockeySim::league_settings;
use HockeySim::testing::league_helper;
fn main() {

    let mut name = player::NameData::read_or_new("names");

   println!("Name, {}",name.random_full_name().unwrap());

let league = league_settings::League::empty("Major Hockey League".parse().unwrap());

    let mut team = Team::new(/* TeamIdentity */TeamIdentity::new("Edmonton".parse().unwrap(), "Edmonton".parse().unwrap(), /* String */"EDM".parse().unwrap(), Conference::east(), Division::pacific()), /* Vec<PlayerRecord> */Vec::new(), /* Vec<StaffMember> */Vec::new());

    let mut draft = Draft::new_draft_with_bias(0.1,4);

    for i in 1..=50
    {
        let pl = player::random_prospect(0.5,false);
        team.add_player(PlayerRecord::new_with_contract(pl.name(), pl.age, pl, DraftStatus::Undrafted, /* PlayerStats */PlayerStats::skater_default(), /* Option<Contract> */None))

    }
league_helper::allocate_league(&league).expect("TODO: panic message");
league_helper::allocate_team(&league, &team);
    league_helper::write_team_data(&league, &team);
    league_helper::write_draft("Draft_data.json".parse().unwrap(), &draft,league);
}





fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
