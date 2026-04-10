use std::fs;
use crate::data::draft::Draft;
use crate::data::team::Team;
use crate::sim::League;


pub static LEAGUES_DATA: &str = "Data/League";
pub fn allocate_league(l:&League)-> std::io::Result<()> {

    fs::create_dir_all(league_path(l))?;
    Ok(())

}

pub fn league_path(l:&League)-> String {

    LEAGUES_DATA.to_owned() +"/"+ &*l.name
}
pub fn allocate_team(l:&League,team:&Team) ->  std::io::Result<()>{

    allocate_league(l).expect("TODO: panic message");

    fs::create_dir_all(team_path(l, team))



}

pub fn team_path(l: &League, team: &Team) -> String {
    league_path(l) + "/" + team.identity().name()
}

pub fn write_team_data(l:&League, team:&Team) -> std::io::Result<()> {

    fs::write(team_path(l,team)+"/"+ &*team.identity().name().to_owned() +"_data", serde_json::to_string_pretty(&team).unwrap())


}

pub fn write_draft(pth:String ,draft:&Draft,l:League){
    let p = league_path(&l)+"/"+ &*pth;
    
    fs::write(p,serde_json::to_string_pretty(draft).unwrap()).expect("Error LOL")
    
}