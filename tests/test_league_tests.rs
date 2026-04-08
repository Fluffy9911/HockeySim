use HockeySim::data::team::TeamLevel;
use HockeySim::sim::SimulationEngine;
use HockeySim::test_league::{TestLeagueBuilder, TestLeagueConfig};

#[test]
fn autobuilder_creates_full_teams() {
    let builder = TestLeagueBuilder::new(TestLeagueConfig::new(
        "Builder Test League".to_string(),
        6,
        2027,
        2,
        0.58,
        0.03,
    ));

    let teams = builder.build_teams();

    assert_eq!(teams.len(), 6);
    assert_eq!(teams[0].roster().len(), 23);
    assert!(teams[0].staff().len() >= 6);
    assert_eq!(teams[0].affiliate_team_abbreviations().len(), 1);

    let goalie_count = teams[0]
        .roster()
        .iter()
        .filter(|player| matches!(player.player().position(), HockeySim::data::player::Position::GOALIE))
        .count();
    assert_eq!(goalie_count, 2);
}

#[test]
fn autobuilder_creates_fifty_man_organizations() {
    let builder = TestLeagueBuilder::new(TestLeagueConfig::new(
        "Organization Test League".to_string(),
        4,
        2028,
        2,
        0.60,
        0.02,
    ));
    let teams = builder.build_organization_teams();

    assert_eq!(teams.len(), 8);

    let major = teams
        .iter()
        .filter(|team| matches!(team.level(), TeamLevel::MAJOR_PRO))
        .count();
    let minor = teams
        .iter()
        .filter(|team| matches!(team.level(), TeamLevel::MINOR_PRO))
        .count();

    assert_eq!(major, 4);
    assert_eq!(minor, 4);
    assert_eq!(teams[0].roster().len() + teams[1].roster().len(), 50);
}

#[test]
fn autobuilder_can_run_a_basic_season() {
    let builder = TestLeagueBuilder::new(TestLeagueConfig::new(
        "Season Test League".to_string(),
        4,
        2028,
        2,
        0.60,
        0.02,
    ));
    let state = builder
        .simulate_season(&SimulationEngine::default(), 9001)
        .expect("season should simulate");

    let season = state.season().expect("season should exist");
    assert_eq!(season.schedule().len(), 12);
    assert_eq!(state.league().standings().len(), 4);
    assert_eq!(state.teams().len(), 8);
    assert!(state.teams()[0].team_stats().games_played() > 0);
    assert!(state.teams()[0].roster()[0].stats().games_played() > 0);
    assert_eq!(state.teams()[1].team_stats().games_played(), 0);
}

#[test]
fn config_can_load_from_text() {
    let config = HockeySim::test_league::TestLeagueConfig::read_text(
        "league_name=Config League\nteam_count=8\nseason_year=2030\ngames_per_matchup=2\nbase_quality=0.55\nquality_step=0.03\n",
    )
    .expect("config should parse");

    assert_eq!(config.league_name(), "Config League");
    assert_eq!(config.team_count(), 8);
    assert_eq!(config.season_year(), 2030);
    assert_eq!(config.games_per_matchup(), 2);
}
