use HockeySim::data::contract::{Contract, ContractType};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::Projection;
use HockeySim::data::staff::{StaffMember, StaffRatings, StaffRole};
use HockeySim::data::team::{Conference, Division, Team, TeamIdentity};
use HockeySim::season::LeagueState;
use HockeySim::sim::{League, SimulationEngine};
use std::fs;

#[test]
fn league_state_saves_loads_and_runs_basic_season() {
    let teams = vec![build_team("ALP", "Alpha City", "Alpha"), build_team("BET", "Beta City", "Beta")];
    let league = League::new("Test League".to_string(), Vec::new());
    let mut state = LeagueState::new(league, teams);

    state.create_basic_season(2026, 4);
    let logs = state
        .simulate_regular_season(&SimulationEngine::default(), 4242)
        .expect("season simulation should run");

    assert_eq!(logs.len(), 4);
    assert_eq!(state.season().expect("season should exist").schedule().len(), 4);
    assert_eq!(state.league().standings().len(), 2);
    assert!(state.teams()[0].team_stats().games_played() > 0);
    assert!(state.teams()[0].roster()[0].stats().games_played() > 0);
    assert!(state.teams()[0].roster()[0].age() >= 25);

    let output_dir = "target/test-output";
    fs::create_dir_all(output_dir).expect("should create test output directory");
    let path = format!("{output_dir}/league_state_roundtrip");
    state.save_to_directory(&path).expect("state should save");

    assert!(std::path::Path::new(&format!("{path}/league/metadata.txt")).exists());
    assert!(std::path::Path::new(&format!("{path}/league/team/ALP/team.txt")).exists());
    assert!(std::path::Path::new(&format!("{path}/league/team/ALP/player")).exists());

    let loaded = LeagueState::load_from_directory(&path).expect("state should load");
    assert_eq!(loaded.teams().len(), 2);
    assert_eq!(loaded.league().standings().len(), 2);
    assert_eq!(loaded.season().expect("loaded season").schedule().len(), 4);
    assert_eq!(loaded.teams()[0].roster().len(), 4);
    assert_eq!(loaded.teams()[0].staff().len(), 2);
    assert!(loaded.teams()[0].roster()[0].contract().is_some());
    assert!(loaded.teams()[0].team_stats().games_played() > 0);
}

fn build_team(abbreviation: &str, city: &str, name: &str) -> Team {
    let mut first_line_center = build_skater("Top Center", Position::CENTER, PlayType::PLAYMAKER, 0.76);
    first_line_center.set_contract(Some(Contract::new(
        ContractType::STANDARD,
        4,
        8.5,
        8.0,
        1.0,
        0.0,
        0,
        0,
    )));

    Team::new(
        TeamIdentity::new(
            city.to_string(),
            name.to_string(),
            abbreviation.to_string(),
            Conference::EAST,
            Division::OTHER,
        ),
        vec![
            first_line_center,
            build_skater("Scoring Wing", Position::LW, PlayType::SNIPER, 0.74),
            build_skater("Two Way Wing", Position::RW, PlayType::PWF, 0.70),
            build_goalie("Starter Goalie", 0.78),
        ],
        vec![
            StaffMember::new(
                "Head Coach".to_string(),
                46,
                StaffRole::HEAD_COACH,
                StaffRatings::new(80, 79, 72, 81),
            ),
            StaffMember::new(
                "Development Coach".to_string(),
                39,
                StaffRole::DEVELOPMENT_COACH,
                StaffRatings::new(84, 68, 66, 73),
            ),
        ],
    )
}

fn build_skater(name: &str, position: Position, play_type: PlayType, quality: f32) -> PlayerRecord {
    let rating = scale_rating(quality);
    PlayerRecord::new(
        name.to_string(),
        24,
        Player::new_skater(
            position,
            play_type,
            SkatingType::QUICK,
            SkatingStats::new(rating, rating - 1, rating - 2, SkatingType::QUICK),
        ),
        Projection::from_quality(quality),
        DraftStatus::undrafted(),
    )
}

fn build_goalie(name: &str, quality: f32) -> PlayerRecord {
    let rating = scale_rating(quality);
    PlayerRecord::new(
        name.to_string(),
        25,
        Player::new_goalie(
            PlayType::HYBRID,
            SkatingType::STRONG,
            SkatingStats::new(rating - 4, rating - 3, rating - 4, SkatingType::STRONG),
            GoalieMovement::new(rating, rating - 1, rating + 1),
        ),
        Projection::from_quality(quality),
        DraftStatus::undrafted(),
    )
}

fn scale_rating(quality: f32) -> i8 {
    (58.0 + quality.clamp(0.0, 1.0) * 34.0).round() as i8
}
