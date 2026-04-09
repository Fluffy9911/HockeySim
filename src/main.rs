use HockeySim::data::movement::{SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::{DevelopmentCurve, Projection};
use HockeySim::data::team;
use HockeySim::data::team::TeamIdentity;
use HockeySim::sim;
use HockeySim::sim::*;
fn main() {

let league = sim::League::empty("Major Hockey League".parse().unwrap());

    let mut prospect = Player::new_skater(Position::CENTER, PlayType::PLAYMAKER, SkatingStats::random(0.75, SkatingType::NIMBLE), DevelopmentCurve::EARLY, Projection::from_quality(75.0));


    println!("Prospect: {}",serde_json::to_string_pretty(&prospect).unwrap());
prospect.develop(2,19);
    println!("Prospect: {}",serde_json::to_string_pretty(&prospect).unwrap());
}
