use HockeySim::data::helper::{DraftStatus, PlayerRecord};
use HockeySim::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use HockeySim::data::player::{PlayType, Player, Position};
use HockeySim::data::projection::{Projection, ProjectionGenerationSettings};
use HockeySim::data::staff::{StaffMember, StaffRatings, StaffRole};
use HockeySim::data::team::{Conference, Division, Team, TeamIdentity, TeamLevel};
use HockeySim::sim::{GameContext, League, LeagueRules, SimulationEngine, TeamStanding};

#[test]
fn simulate_game_returns_non_tied_final_score() {
    let engine = SimulationEngine::default();
    let home = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.82,
        88,
    );
    let away = build_team(
        "Seattle".to_string(),
        "Emeralds".to_string(),
        "SEA".to_string(),
        0.56,
        67,
    );

    let game = engine.simulate_game(&home, &away, None, GameContext::regular_season(), 42);

    assert_eq!(game.home_team(), "VAN");
    assert_eq!(game.away_team(), "SEA");
    assert_ne!(game.home_goals(), game.away_goals());
    assert!(game.home_goals() >= 0);
    assert!(game.away_goals() >= 0);
}

#[test]
fn league_record_game_updates_points_and_order() {
    let engine = SimulationEngine::default();
    let home = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.81,
        85,
    );
    let away = build_team(
        "Calgary".to_string(),
        "Wranglers".to_string(),
        "CGY".to_string(),
        0.42,
        58,
    );
    let mut league = League::new(
        "Test League".to_string(),
        vec![TeamStanding::new("VAN".to_string()), TeamStanding::new("CGY".to_string())],
    );

    let game = engine.simulate_game(&home, &away, Some(&league), GameContext::new(12), 7);
    league.record_game(&game);

    let first = &league.standings()[0];
    let second = &league.standings()[1];

    assert!(first.points() >= second.points());
    assert_eq!(first.games_played(), 1);
    assert_eq!(second.games_played(), 1);
    assert_eq!(first.goals_for() + second.goals_for(), game.home_goals() + game.away_goals());
}

#[test]
fn standings_factor_changes_team_profile() {
    let engine = SimulationEngine::default();
    let team = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.67,
        76,
    );
    let cold_league = League::new(
        "Cold".to_string(),
        vec![build_standing("VAN", 8, 10, 2, 52, 63)],
    );
    let hot_league = League::new(
        "Hot".to_string(),
        vec![build_standing("VAN", 15, 4, 1, 71, 49)],
    );

    let cold_profile = engine.analyze_team(&team, Some(&cold_league));
    let hot_profile = engine.analyze_team(&team, Some(&hot_league));

    assert!(hot_profile.standings_factor() > cold_profile.standings_factor());
    assert!(hot_profile.overall() > cold_profile.overall());
}

#[test]
fn best_of_seven_series_generates_verbose_logs() {
    let engine = SimulationEngine::default();
    let higher_seed = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.79,
        86,
    );
    let lower_seed = build_team(
        "Seattle".to_string(),
        "Emeralds".to_string(),
        "SEA".to_string(),
        0.71,
        79,
    );
    let league = League::new(
        "Playoff League".to_string(),
        vec![
            build_standing("VAN", 51, 22, 9, 263, 210),
            build_standing("SEA", 46, 27, 9, 248, 221),
        ],
    );

    let series = engine.simulate_best_of_seven(&higher_seed, &lower_seed, Some(&league), 20260407);

    println!("{}", series.summary_log());

    assert_eq!(series.wins_needed(), 4);
    assert!(series.games().len() >= 4);
    assert!(series.games().len() <= 7);
    assert!(
        (series.higher_seed_wins() == 4 && series.lower_seed_wins() < 4)
            || (series.lower_seed_wins() == 4 && series.higher_seed_wins() < 4)
    );
    assert!(series.winner() == "VAN" || series.winner() == "SEA");
    assert!(series.summary_log().contains("Game 1"));
    assert!(series.summary_log().contains("profile => overall"));
    assert!(series.summary_log().contains("Series after Game"));
}

#[test]
fn custom_league_supports_minor_affiliates() {
    let mut parent = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.79,
        86,
    );
    let minor = Team::new_full(
        TeamIdentity::new(
            "Abbotsford".to_string(),
            "Pilots".to_string(),
            "ABB".to_string(),
            Conference::WEST,
            Division::PACIFIC,
        ),
        TeamLevel::MINOR_PRO,
        vec![build_skater("Prospect Center", Position::CENTER, PlayType::PLAYMAKER, 0.61)],
        Vec::new(),
        HockeySim::data::stats::TeamStats::default(),
        HockeySim::data::contract::TeamContractSettings::nhl_default(),
        Vec::new(),
    );
    parent.add_affiliate_team("ABB".to_string());

    let mut league = League::new_custom(
        "Custom Major League".to_string(),
        TeamLevel::MAJOR_PRO,
        LeagueRules::new(
            3,
            1,
            0,
            25,
            7,
            false,
            None,
            vec![TeamLevel::MINOR_PRO],
        ),
        Vec::new(),
        vec![],
    );

    league.register_team(&parent);
    league.register_team(&minor);

    assert_eq!(league.rules().points_for_win(), 3);
    assert!(!league.rules().allow_shootout());
    assert_eq!(league.team_registry().len(), 2);
    assert_eq!(parent.affiliate_team_abbreviations(), &["ABB".to_string()]);
    assert!(matches!(minor.level(), TeamLevel::MINOR_PRO));
    assert_eq!(league.team_registry()[0].affiliate_team_abbreviations(), &["ABB".to_string()]);
}

#[test]
fn randomness_allows_upsets_across_many_games() {
    let engine = SimulationEngine::default();
    let favorite = build_team(
        "Vancouver".to_string(),
        "Voyagers".to_string(),
        "VAN".to_string(),
        0.84,
        88,
    );
    let underdog = build_team(
        "Calgary".to_string(),
        "Wranglers".to_string(),
        "CGY".to_string(),
        0.46,
        60,
    );

    let mut underdog_wins = 0;
    for seed in 0..64_u64 {
        let game = engine.simulate_game(&favorite, &underdog, None, GameContext::new(seed as i16 + 1), 10_000 + seed);
        if game.away_goals() > game.home_goals() {
            underdog_wins += 1;
        }
    }

    assert!(underdog_wins > 0);
    assert!(underdog_wins < 40);
}

fn build_standing(
    abbreviation: &str,
    wins: i16,
    losses: i16,
    overtime_losses: i16,
    goals_for: i16,
    goals_against: i16,
) -> TeamStanding {
    TeamStanding::from_record(
        abbreviation.to_string(),
        wins,
        losses,
        overtime_losses,
        goals_for,
        goals_against,
    )
}

fn build_team(
    city: String,
    name: String,
    abbreviation: String,
    quality: f32,
    coach_tactical: i8,
) -> Team {
    let mut roster = Vec::new();
    roster.push(build_skater("Top Center", Position::CENTER, PlayType::PLAYMAKER, quality));
    roster.push(build_skater("Sniper Wing", Position::LW, PlayType::SNIPER, quality));
    roster.push(build_skater("Two Way Wing", Position::RW, PlayType::PWF, quality - 0.05));
    roster.push(build_skater("Blue Line 1", Position::LD, PlayType::DFD, quality));
    roster.push(build_skater("Blue Line 2", Position::RD, PlayType::OFD, quality - 0.02));
    roster.push(build_goalie("Starter", quality + 0.03));

    let staff = vec![
        StaffMember::new(
            "Head Coach".to_string(),
            48,
            StaffRole::HEAD_COACH,
            StaffRatings::new(80, coach_tactical, 72, 84),
        ),
        StaffMember::new(
            "Goalie Coach".to_string(),
            44,
            StaffRole::GOALIE_COACH,
            StaffRatings::new(77, 68, 64, 71),
        ),
    ];

    Team::new(
        TeamIdentity::new(city, name, abbreviation, Conference::WEST, Division::PACIFIC),
        roster,
        staff,
    )
}

fn build_skater(name: &str, position: Position, play_type: PlayType, quality: f32) -> PlayerRecord {
    let rating = scale_rating(quality);
    let player = Player::new_skater(
        position,
        play_type,
        SkatingType::QUICK,
        SkatingStats::new(rating, rating - 2, rating - 1, SkatingType::QUICK),
    );
    PlayerRecord::new(
        name.to_string(),
        21,
        player,
        Projection::from_quality_with_settings(
            quality,
            ProjectionGenerationSettings::new(0.7, 0.7, 0.7, 0.2, 0.2, 0.8, 0.8),
        ),
        DraftStatus::undrafted(),
    )
}

fn build_goalie(name: &str, quality: f32) -> PlayerRecord {
    let rating = scale_rating(quality);
    let player = Player::new_goalie(
        PlayType::HYBRID,
        SkatingType::STRONG,
        SkatingStats::new(rating - 6, rating - 3, rating - 5, SkatingType::STRONG),
        GoalieMovement::new(rating, rating - 1, rating + 1),
    );
    PlayerRecord::new(
        name.to_string(),
        23,
        player,
        Projection::from_quality_with_settings(
            quality,
            ProjectionGenerationSettings::new(0.75, 0.7, 0.6, 0.18, 0.2, 0.75, 0.76),
        ),
        DraftStatus::undrafted(),
    )
}

fn scale_rating(quality: f32) -> i8 {
    (55.0 + quality.clamp(0.0, 1.0) * 35.0).round() as i8
}
