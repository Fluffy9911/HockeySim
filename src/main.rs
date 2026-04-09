use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Write;
use HockeySim::data::movement::{SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::{DevelopmentCurve, Projection};
use HockeySim::data::{player, team};
use HockeySim::data::team::TeamIdentity;
use HockeySim::sim;
use HockeySim::sim::*;
fn main() {

let league = sim::League::empty("Major Hockey League".parse().unwrap());

for i in 1..10{

    let prospect = player::random_prospect(0.5,false);
 let pt = format!("data/{}",prospect.name());
write_to_file(&*pt, &*serde_json::to_string_pretty(&prospect).unwrap())
}



}
fn write_to_file(path: &str, content: &str) {
    fs::write(path, content).expect("Failed to write file");
}
