use HockeySim::data::contract::{Contract, ContractLimits, ContractType, TeamContractSettings};
use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::Projection;
use HockeySim::data::staff::{StaffDevelopment, StaffMember, StaffRatings, StaffRole};
use HockeySim::data::stats::{GoalieStats, PlayerStats, TeamStats};
use HockeySim::data::team::{Conference, Division, Team, TeamIdentity};

#[test]
fn player_record_tracks_skater_stats() {
    let player = Player::new_skater(
        Position::CENTER,
        PlayType::PLAYMAKER,
        SkatingType::QUICK,
        SkatingStats::new(80, 79, 78, SkatingType::QUICK),
    );
    let stats = PlayerStats::new(12, 6, 10, 7, 8, 42, 2, 5, 1, 0, 1, 1, 110, 90, 14, 9, 12, 6, 220, None);
    let record = PlayerRecord::new_with_stats(
        "Test Skater".to_string(),
        22,
        player,
        Projection::from_quality(0.7),
        DraftStatus::undrafted(),
        stats,
    );

    assert_eq!(record.stats().points(), 16);
    assert_eq!(record.stats().shots(), 42);
    assert!(record.stats().goalie_stats().is_none());
}

#[test]
fn goalie_stats_report_percentages() {
    let goalie = GoalieStats::new(25, 15, 7, 3, 800, 742, 58, 3, 14, 1, 1500);

    assert!(goalie.save_percentage() > 0.92);
    assert!(goalie.goals_against_average() > 2.0);
}

#[test]
fn team_exposes_staff_groups_and_team_stats() {
    let team = Team::new_with_stats(
        TeamIdentity::new(
            "Vancouver".to_string(),
            "Voyagers".to_string(),
            "VAN".to_string(),
            Conference::WEST,
            Division::PACIFIC,
        ),
        Vec::new(),
        vec![
            StaffMember::new("Bench Boss".to_string(), 49, StaffRole::HEAD_COACH, StaffRatings::new(82, 84, 70, 88)),
            StaffMember::new("Lead Scout".to_string(), 53, StaffRole::HEAD_SCOUT, StaffRatings::new(60, 66, 91, 74)),
            StaffMember::new("Dev 1".to_string(), 41, StaffRole::DEVELOPMENT_COACH, StaffRatings::new(78, 72, 67, 69)),
            StaffMember::new("Scout 2".to_string(), 39, StaffRole::SCOUT, StaffRatings::new(55, 58, 83, 61)),
        ],
        TeamStats::new(10, 6, 3, 1, 36, 27, 320, 284, 9, 31, 6, 29, 290, 270, 210, 130),
    );

    assert!(team.head_coach().is_some());
    assert!(team.head_scout().is_some());
    assert_eq!(team.development_coaches().len(), 1);
    assert_eq!(team.scouts().len(), 2);
    assert_eq!(team.team_stats().points(), 13);
    assert!(team.team_stats().power_play_percentage() > 0.25);
}

#[test]
fn staff_can_develop_toward_potential() {
    let ratings = StaffRatings::new(60, 62, 58, 61);
    let mut coach = StaffMember::new_with_development(
        "Rising Coach".to_string(),
        36,
        StaffRole::DEVELOPMENT_COACH,
        ratings,
        StaffDevelopment::new(60, 85, 30, 80),
    );

    let before = coach.ratings().teaching();
    coach.develop();

    assert!(coach.ratings().teaching() >= before);
    assert!(coach.development().current_level() <= coach.development().potential());
}

#[test]
fn goalie_player_record_defaults_goalie_stats() {
    let goalie = Player::new_goalie(
        PlayType::HYBRID,
        SkatingType::STRONG,
        SkatingStats::new(70, 72, 68, SkatingType::STRONG),
        GoalieMovement::new(78, 80, 79),
    );
    let record = PlayerRecord::new(
        "Goalie".to_string(),
        24,
        goalie,
        Projection::from_quality(0.6),
        DraftStatus::undrafted(),
    );

    assert!(record.stats().goalie_stats().is_some());
}

#[test]
fn contracts_validate_against_custom_limits() {
    let limits = ContractLimits::new(1, 3, 0.5, 4.0, 0.5, 4.0, 0.0, 1.0, 0.0, 0.5, 0, 0, 0, 0);
    let valid = Contract::new(ContractType::ENTRY_LEVEL, 3, 2.8, 2.2, 0.5, 0.25, 0, 0);
    let invalid = Contract::new(ContractType::STANDARD, 6, 8.5, 7.5, 2.0, 1.0, 1, 1);

    assert!(valid.validate(&limits).is_ok());
    assert!(invalid.validate(&limits).is_err());
}

#[test]
fn team_tracks_custom_contract_settings_and_cap_hit() {
    let contract_settings = TeamContractSettings::new(
        100.0,
        50.0,
        60,
        5,
        ContractLimits::new(1, 10, 0.5, 20.0, 0.5, 18.0, 0.0, 12.0, 0.0, 8.0, 0, 1, 0, 1),
    );
    let player = Player::new_skater(
        Position::CENTER,
        PlayType::PLAYMAKER,
        SkatingType::QUICK,
        SkatingStats::new(80, 79, 78, SkatingType::QUICK),
    );
    let contract = Contract::new(ContractType::STANDARD, 5, 7.5, 7.0, 1.0, 0.0, 1, 0);
    let record = PlayerRecord::new_with_contract(
        "Signed Player".to_string(),
        26,
        player,
        Projection::from_quality(0.75),
        DraftStatus::undrafted(),
        PlayerStats::skater_default(),
        Some(contract),
    );
    let team = Team::new_with_contract_settings(
        TeamIdentity::new(
            "Vancouver".to_string(),
            "Voyagers".to_string(),
            "VAN".to_string(),
            Conference::WEST,
            Division::PACIFIC,
        ),
        vec![record],
        Vec::new(),
        TeamStats::default(),
        contract_settings,
    );

    assert_eq!(team.contract_settings().max_contracts(), 60);
    assert_eq!(team.active_contract_count(), 1);
    assert!(team.total_cap_hit_millions() > 7.0);
}
