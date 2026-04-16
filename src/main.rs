use std::fs;
use std::io::Write;
use HockeySim::data::player;
use HockeySim::data::draft::Draft;
use HockeySim::data::game::names::{Conference, Division};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::player::{random_prospect_of_position, NameData, Player, Position};
use HockeySim::data::stats::PlayerStats;
use HockeySim::data::team::{Team, TeamIdentity};
use HockeySim::{league_settings, sim, sim_engine};
use HockeySim::data::dates::GameDate;
use HockeySim::league_settings::League;
use HockeySim::savestate::savedata;
use HockeySim::savestate::savedata::{CoreConfig, FileType, SaveInfo};
use HockeySim::sim_engine::Schedule;
use HockeySim::testing::league_helper;
fn main() {


    
let mut info = SaveInfo::new();



  let mut config = info.create_save("test");

    let mut ctx = savedata::SaveContext::new(config);

savedata::create_initial_state(&mut ctx.core);


    let mut sim = sim_engine::Sim::create_or_load(&ctx);


    let t1 = Team::new(TeamIdentity::new("Edmonton".parse().unwrap(), "Edmonton Oilers".parse().unwrap(), "EDM".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());
    let t2 = Team::new(TeamIdentity::new("Calgary".parse().unwrap(), "Calgary Flames".parse().unwrap(), "CGY".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t3 = Team::new(TeamIdentity::new("Vancouver".parse().unwrap(), "Vancouver Canucks".parse().unwrap(), "VAN".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t4 = Team::new(TeamIdentity::new("Seattle".parse().unwrap(), "Seattle Kraken".parse().unwrap(), "SEA".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t5 = Team::new(TeamIdentity::new("Vegas".parse().unwrap(), "Vegas Golden Knights".parse().unwrap(), "VGK".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t6 = Team::new(TeamIdentity::new("Los Angeles".parse().unwrap(), "Los Angeles Kings".parse().unwrap(), "LAK".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t7 = Team::new(TeamIdentity::new("San Jose".parse().unwrap(), "San Jose Sharks".parse().unwrap(), "SJS".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t8 = Team::new(TeamIdentity::new("Anaheim".parse().unwrap(), "Anaheim Ducks".parse().unwrap(), "ANA".parse().unwrap(), Conference::west(), Division::pacific()), Vec::new(), Vec::new());

    let t9 = Team::new(TeamIdentity::new("Toronto".parse().unwrap(), "Toronto Maple Leafs".parse().unwrap(), "TOR".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t10 = Team::new(TeamIdentity::new("Montreal".parse().unwrap(), "Montreal Canadiens".parse().unwrap(), "MTL".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t11 = Team::new(TeamIdentity::new("Boston".parse().unwrap(), "Boston Bruins".parse().unwrap(), "BOS".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t12 = Team::new(TeamIdentity::new("Ottawa".parse().unwrap(), "Ottawa Senators".parse().unwrap(), "OTT".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t13 = Team::new(TeamIdentity::new("Tampa Bay".parse().unwrap(), "Tampa Bay Lightning".parse().unwrap(), "TBL".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t14 = Team::new(TeamIdentity::new("Florida".parse().unwrap(), "Florida Panthers".parse().unwrap(), "FLA".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    let t15 = Team::new(TeamIdentity::new("Detroit".parse().unwrap(), "Detroit Red Wings".parse().unwrap(), "DET".parse().unwrap(), Conference::east(), Division::atlantic()), Vec::new(), Vec::new());

    {
        let league: &mut League = sim.get_league(0);
        league.add_team(t1);

        league.add_team(t2);
        league.add_team(t3);
        league.add_team(t4);
        league.add_team(t5);
        league.add_team(t6);
        league.add_team(t7);
        league.add_team(t8);
        league.add_team(t9);
        league.add_team(t10);
        league.add_team(t11);
        league.add_team(t12);
        league.add_team(t13);
        league.add_team(t14);
        league.add_team(t15);
        let s = sim_engine::Schedule::schedule_season_games(&GameDate::new(2025, 1, 1).unwrap(), &GameDate::new(2026, 1, 1).unwrap(), Vec::new(), league);

        ctx.write_struct(FileType::LEAGUE_DATA, "schedule.json", &s);

    }




    sim.advance_date();



    sim.save_data(&ctx);

    let schedule = Schedule::new();


}


fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
