use HockeySim::data::staff::StaffMember;
use HockeySim::data::team::{Team, TeamLevel};
use HockeySim::sim::SimulationEngine;
use HockeySim::test_league::{TestLeagueBuilder, TestLeagueConfig};
use std::fs;

fn main() {
    run_full_season_scenario().expect("failed to run 32-team season scenario");
}

fn run_full_season_scenario() -> Result<(), String> {
    let engine = SimulationEngine::default();
    let builder = TestLeagueBuilder::new(TestLeagueConfig::new(
        "32 Team Season Scenario".to_string(),
        32,
        2026,
        82,
        0.54,
        0.01,
    ));

    let initial_teams = builder.build_organization_teams();
    let mut state = builder.build_organization_league_state();
    state.create_target_games_season_for_level(2026, 82, TeamLevel::MAJOR_PRO);

    println!("32 Team Full Season Simulation");
    println!("Season Year: 2026");
    println!("Organizations: {}", state.teams().iter().filter(|team| matches!(team.level(), TeamLevel::MAJOR_PRO)).count());
    println!("Teams In State: {}", state.teams().len());
    println!("Players In State: {}", state.teams().iter().map(|team| team.roster().len()).sum::<usize>());
    println!(
        "Games Scheduled: {}",
        state.season().ok_or_else(|| "season missing".to_string())?.schedule().len()
    );
    println!();

    let logs = state.simulate_regular_season(&engine, 20260407)?;
    println!("Season Simulation Complete");
    println!("Games Played: {}", logs.len());
    println!();

    let initial_major_teams: Vec<&Team> = initial_teams
        .iter()
        .filter(|team| matches!(team.level(), TeamLevel::MAJOR_PRO))
        .collect();
    let final_major_teams: Vec<&Team> = state
        .teams()
        .iter()
        .filter(|team| matches!(team.level(), TeamLevel::MAJOR_PRO))
        .collect();
    let report = build_full_report(&initial_major_teams, &final_major_teams, state.league().standings());
    println!("{report}");

    fs::create_dir_all("data/generated")
        .map_err(|error| format!("failed to create output directory: {error}"))?;
    state.save_to_directory("data/generated/season_2026_32_team_state")?;
    fs::write("data/generated/season_2026_32_team_report.txt", &report)
        .map_err(|error| format!("failed to write report file: {error}"))?;

    println!("Saved state: data/generated/season_2026_32_team_state");
    println!("Saved report: data/generated/season_2026_32_team_report.txt");
    Ok(())
}

fn build_full_report(
    initial_teams: &[&Team],
    final_teams: &[&Team],
    standings: &[HockeySim::sim::TeamStanding],
) -> String {
    let mut report = String::new();
    report.push_str("Final Standings\n");
    report.push_str("==============\n");

    for standing in standings {
        report.push_str(&format!(
            "{} | GP {} | W {} | L {} | OTL {} | PTS {} | GF {} | GA {}\n",
            standing.team_abbreviation(),
            standing.games_played(),
            standing.wins(),
            standing.losses(),
            standing.overtime_losses(),
            standing.points(),
            standing.goals_for(),
            standing.goals_against(),
        ));
    }

    report.push('\n');
    report.push_str("Team Reports\n");
    report.push_str("============\n");

    for (initial, final_team) in initial_teams.iter().zip(final_teams.iter()) {
        report.push_str(&build_team_report(initial, final_team));
        report.push('\n');
    }

    report
}

fn build_team_report(initial: &Team, final_team: &Team) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{} {} ({})\n",
        final_team.identity().city(),
        final_team.identity().name(),
        final_team.identity().abbreviation(),
    ));
    output.push_str(&format!(
        "  Team Stats: GP {} | W {} | L {} | OTL {} | PTS {} | GF {} | GA {} | SF {} | SA {}\n",
        final_team.team_stats().games_played(),
        final_team.team_stats().wins(),
        final_team.team_stats().losses(),
        final_team.team_stats().overtime_losses(),
        final_team.team_stats().points(),
        final_team.team_stats().goals_for(),
        final_team.team_stats().goals_against(),
        final_team.team_stats().shots_for(),
        final_team.team_stats().shots_against(),
    ));
    output.push_str(&format!(
        "  Roster Growth: avg {:.2} -> {:.2} ({:+.2})\n",
        average_team_rating(initial),
        average_team_rating(final_team),
        average_team_rating(final_team) - average_team_rating(initial),
    ));

    let mut player_growth: Vec<(String, f32)> = initial
        .roster()
        .iter()
        .zip(final_team.roster().iter())
        .map(|(before, after)| {
            (
                after.name().to_string(),
                player_rating(after) - player_rating(before),
            )
        })
        .collect();
    player_growth.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));

    output.push_str("  Top Player Growth:\n");
    for (name, delta) in player_growth.iter().take(3) {
        output.push_str(&format!("    {} {:+.2}\n", name, delta));
    }

    let mut coach_growth: Vec<(String, f32)> = initial
        .staff()
        .iter()
        .zip(final_team.staff().iter())
        .map(|(before, after)| (after.name().to_string(), staff_rating(after) - staff_rating(before)))
        .collect();
    coach_growth.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));

    output.push_str("  Coach Growth:\n");
    for (name, delta) in coach_growth.iter().take(2) {
        output.push_str(&format!("    {} {:+.2}\n", name, delta));
    }

    let top_scorer = final_team
        .roster()
        .iter()
        .filter(|player| !matches!(player.player().position(), HockeySim::data::player::Position::GOALIE))
        .max_by_key(|player| player.stats().points());

    if let Some(top_scorer) = top_scorer {
        output.push_str(&format!(
            "  Top Scorer: {} | G {} | A {} | P {}\n",
            top_scorer.name(),
            top_scorer.stats().goals(),
            top_scorer.stats().assists(),
            top_scorer.stats().points(),
        ));
    }

    let starting_goalie = final_team
        .roster()
        .iter()
        .filter(|player| matches!(player.player().position(), HockeySim::data::player::Position::GOALIE))
        .max_by_key(|goalie| goalie.stats().goalie_stats().map(|stats| stats.starts()).unwrap_or(0));

    if let Some(goalie) = starting_goalie {
        if let Some(goalie_stats) = goalie.stats().goalie_stats() {
            output.push_str(&format!(
                "  Starting Goalie: {} | Starts {} | W {} | L {} | SV% {:.3}\n",
                goalie.name(),
                goalie_stats.starts(),
                goalie_stats.wins(),
                goalie_stats.losses(),
                goalie_stats.save_percentage(),
            ));
        }
    }

    output
}

fn average_team_rating(team: &Team) -> f32 {
    let total = team.roster().iter().map(player_rating).sum::<f32>();
    total / team.roster().len().max(1) as f32
}

fn player_rating(player: &HockeySim::data::helper::PlayerRecord) -> f32 {
    let skating = player.player().skate_stats();
    let skating_average = (skating.speed() as f32 + skating.edges() as f32 + skating.acceleration() as f32) / 3.0;

    if let Some(movement) = player.player().goalie_movement() {
        let movement_average = (movement.side() as f32 + movement.up_down() as f32 + movement.push() as f32) / 3.0;
        skating_average * 0.35 + movement_average * 0.65
    } else {
        skating_average
    }
}

fn staff_rating(staff_member: &StaffMember) -> f32 {
    let ratings = staff_member.ratings();
    (ratings.teaching() as f32
        + ratings.tactical() as f32
        + ratings.evaluation() as f32
        + ratings.leadership() as f32)
        / 4.0
}
