use HockeySim::data::movement::{SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::{DevelopmentCurve, Projection};
use HockeySim::data::{player, team};
use HockeySim::data::team::TeamIdentity;
use HockeySim::sim;
use HockeySim::sim::*;
fn main() {

let league = sim::League::empty("Major Hockey League".parse().unwrap());

    let mut prospect = player::random_prospect(0.75,false);

    println!("Prospect: {}",serde_json::to_string_pretty(&prospect).unwrap());
    for i in 1..10{

        prospect.age_develop(17);
        println!("Overall: {}",prospect.overall())
    }


    println!("Prospect: {}",serde_json::to_string_pretty(&prospect).unwrap());

}
