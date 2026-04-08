use crate::data::contract::{ContractLimits, TeamContractSettings};
use crate::data::helper::PlayerRecord;
use crate::data::staff::{StaffDevelopment, StaffMember, StaffRatings, StaffRole};
use crate::data::stats::TeamStats;
use crate::data::team::{ Team, TeamIdentity, TeamLevel};
use crate::sim::{GameContext, League, LeagueRules, LeagueTeamEntry, SimulationEngine, SimulatedGame, TeamStanding};
use crate::data::game::names::*;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ScheduledGameResult {
    home_goals: i16,
    away_goals: i16,
    overtime: bool,
    shootout: bool,
}

pub struct ScheduledGame {
    day: i16,
    home_team: String,
    away_team: String,
    result: Option<ScheduledGameResult>,
}

pub struct Season {
    year: i16,
    games_per_matchup: i16,
    completed_games: i16,
    schedule: Vec<ScheduledGame>,
}

pub struct LeagueState {
    league: League,
    teams: Vec<Team>,
    season: Option<Season>,
}

impl ScheduledGameResult {
    pub fn new(home_goals: i16, away_goals: i16, overtime: bool, shootout: bool) -> ScheduledGameResult {
        ScheduledGameResult {
            home_goals,
            away_goals,
            overtime,
            shootout,
        }
    }

    pub fn home_goals(&self) -> i16 { self.home_goals }
    pub fn away_goals(&self) -> i16 { self.away_goals }
    pub fn overtime(&self) -> bool { self.overtime }
    pub fn shootout(&self) -> bool { self.shootout }
}

impl ScheduledGame {
    pub fn new(day: i16, home_team: String, away_team: String) -> ScheduledGame {
        ScheduledGame {
            day,
            home_team,
            away_team,
            result: None,
        }
    }

    pub fn day(&self) -> i16 { self.day }
    pub fn home_team(&self) -> &str { &self.home_team }
    pub fn away_team(&self) -> &str { &self.away_team }
    pub fn result(&self) -> Option<&ScheduledGameResult> { self.result.as_ref() }
    pub fn played(&self) -> bool { self.result.is_some() }
}

impl Season {
    pub fn new(year: i16, games_per_matchup: i16, schedule: Vec<ScheduledGame>) -> Season {
        Season {
            year,
            games_per_matchup,
            completed_games: 0,
            schedule,
        }
    }

    pub fn year(&self) -> i16 { self.year }
    pub fn games_per_matchup(&self) -> i16 { self.games_per_matchup }
    pub fn completed_games(&self) -> i16 { self.completed_games }
    pub fn schedule(&self) -> &[ScheduledGame] { &self.schedule }
}

impl LeagueState {
    pub fn new(league: League, teams: Vec<Team>) -> LeagueState {
        LeagueState {
            league,
            teams,
            season: None,
        }
    }

    pub fn league(&self) -> &League {
        &self.league
    }

    pub fn teams(&self) -> &[Team] {
        &self.teams
    }

    pub fn teams_mut(&mut self) -> &mut [Team] {
        &mut self.teams
    }

    pub fn season(&self) -> Option<&Season> {
        self.season.as_ref()
    }

    pub fn create_basic_season(&mut self, year: i16, games_per_matchup: i16) {
        let mut schedule = Vec::new();
        let mut day = 1;

        for i in 0..self.teams.len() {
            for j in (i + 1)..self.teams.len() {
                for game_number in 0..games_per_matchup {
                    let home_first = game_number % 2 == 0;
                    let home = if home_first {
                        self.teams[i].identity().abbreviation().to_string()
                    } else {
                        self.teams[j].identity().abbreviation().to_string()
                    };
                    let away = if home_first {
                        self.teams[j].identity().abbreviation().to_string()
                    } else {
                        self.teams[i].identity().abbreviation().to_string()
                    };
                    schedule.push(ScheduledGame::new(day, home, away));
                    day += 1;
                }
            }
        }

        self.season = Some(Season::new(year, games_per_matchup, schedule));
    }

    pub fn create_target_games_season(&mut self, year: i16, target_games_per_team: i16) {
        self.create_target_games_season_for_level(year, target_games_per_team, TeamLevel::MAJOR_PRO);
    }

    pub fn create_target_games_season_for_level(
        &mut self,
        year: i16,
        target_games_per_team: i16,
        level: TeamLevel,
    ) {
        let mut schedule = Vec::new();
        let mut day = 1;
        let team_indices: Vec<usize> = self
            .teams
            .iter()
            .enumerate()
            .filter(|(_, team)| same_team_level(team.level(), &level))
            .map(|(index, _)| index)
            .collect();
        let team_count = team_indices.len();
        let mut games_per_team = vec![0_i16; team_count];

        for i in 0..team_count {
            for j in (i + 1)..team_count {
                schedule.push(ScheduledGame::new(
                    day,
                    self.teams[team_indices[i]].identity().abbreviation().to_string(),
                    self.teams[team_indices[j]].identity().abbreviation().to_string(),
                ));
                day += 1;
                schedule.push(ScheduledGame::new(
                    day,
                    self.teams[team_indices[j]].identity().abbreviation().to_string(),
                    self.teams[team_indices[i]].identity().abbreviation().to_string(),
                ));
                day += 1;
                games_per_team[i] += 2;
                games_per_team[j] += 2;
            }
        }

        let mut round = 0;
        while games_per_team.iter().any(|games| *games < target_games_per_team) {
            let mut added_any = false;

            for i in 0..team_count {
                for j in (i + 1)..team_count {
                    if games_per_team[i] >= target_games_per_team || games_per_team[j] >= target_games_per_team {
                        continue;
                    }

                    let home_is_left = (round + i + j) % 2 == 0;
                    let (home, away) = if home_is_left {
                        (
                            self.teams[team_indices[i]].identity().abbreviation().to_string(),
                            self.teams[team_indices[j]].identity().abbreviation().to_string(),
                        )
                    } else {
                        (
                            self.teams[team_indices[j]].identity().abbreviation().to_string(),
                            self.teams[team_indices[i]].identity().abbreviation().to_string(),
                        )
                    };

                    schedule.push(ScheduledGame::new(day, home, away));
                    day += 1;
                    games_per_team[i] += 1;
                    games_per_team[j] += 1;
                    added_any = true;

                    if games_per_team.iter().all(|games| *games >= target_games_per_team) {
                        break;
                    }
                }

                if games_per_team.iter().all(|games| *games >= target_games_per_team) {
                    break;
                }
            }

            if !added_any {
                break;
            }
            round += 1;
        }

        self.season = Some(Season::new(year, target_games_per_team, schedule));
    }

    pub fn simulate_regular_season(&mut self, engine: &SimulationEngine, seed: u64) -> Result<Vec<String>, String> {
        let Some(season) = self.season.as_mut() else {
            return Err("season has not been created".to_string());
        };

        let mut logs = Vec::new();

        for index in 0..season.schedule.len() {
            if season.schedule[index].played() {
                continue;
            }

            let home_abbr = season.schedule[index].home_team().to_string();
            let away_abbr = season.schedule[index].away_team().to_string();
            let home_index = find_team_index(&self.teams, &home_abbr)?;
            let away_index = find_team_index(&self.teams, &away_abbr)?;

            let game = {
                let home_team = &self.teams[home_index];
                let away_team = &self.teams[away_index];
                engine.simulate_game(
                    home_team,
                    away_team,
                    Some(&self.league),
                    GameContext::new(index as i16 + 1),
                    seed ^ ((index as u64 + 1) * 65_537),
                )
            };

            season.schedule[index].result = Some(ScheduledGameResult::new(
                game.home_goals(),
                game.away_goals(),
                game.overtime(),
                game.shootout(),
            ));
            season.completed_games += 1;
            self.league.record_game(&game);
            apply_game_to_teams(&mut self.teams, home_index, away_index, &game);
            logs.push(format!(
                "Day {}: {} {} - {} {}{}",
                season.schedule[index].day(),
                game.away_team(),
                game.away_goals(),
                game.home_goals(),
                game.home_team(),
                decision_suffix(&game),
            ));
        }

        self.run_player_and_staff_development();
        Ok(logs)
    }

    pub fn run_player_and_staff_development(&mut self) {
        for team in &mut self.teams {
            let development_coaches = team.development_coaches();
            let coaching_bonus = development_coaches
                .iter()
                .map(|coach| coach.ratings().teaching() as i16)
                .sum::<i16>()
                / (development_coaches.len().max(1) as i16);

            for player in team.roster_mut().iter_mut() {
                player.develop((coaching_bonus / 20) as i8);
                player.age_one_year();
            }

            for staff_member in team.staff_mut().iter_mut() {
                staff_member.develop();
                staff_member.age_one_year();
            }
        }
    }
}

#[derive(Default)]
struct LeagueMeta {
    name: String,
    level: TeamLevel,
    points_for_win: i16,
    points_for_overtime_loss: i16,
    points_for_loss: i16,
    max_roster_size: i16,
    playoff_series_length: i16,
    allow_shootout: bool,
    parent_league: Option<String>,
    affiliated_minor_levels: Vec<TeamLevel>,
}

struct SeasonMeta {
    year: i16,
    games_per_matchup: i16,
    completed_games: i16,
}

struct TeamBuilder {
    city: String,
    name: String,
    abbreviation: String,
    conference: Conference,
    division: Division,
    level: TeamLevel,
    affiliate_team_abbreviations: Vec<String>,
    team_stats: TeamStats,
    contract_settings: TeamContractSettings,
    roster: Vec<PlayerRecord>,
    staff: Vec<StaffMember>,
}

impl Default for TeamLevel {
    fn default() -> TeamLevel {
        TeamLevel::MAJOR_PRO
    }
}

impl LeagueState {






}

fn collect_block(lines: &[&str], start: usize, end_marker: &str) -> Result<(Vec<String>, usize), String> {
    let mut block = Vec::new();
    let mut index = start;

    while index < lines.len() {
        if lines[index].trim() == end_marker {
            return Ok((block, index + 1));
        }
        block.push(lines[index].to_string());
        index += 1;
    }

    Err(format!("missing block end marker: {end_marker}"))
}

fn collect_owned_lines(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

fn write_league_metadata_text(league: &League) -> String {
    let mut output = String::new();
    output.push_str(&format!("name={}\n", escape_text(league.name())));
    output.push_str(&format!("level={}\n", team_level_to_str(league.level())));
    output.push_str(&format!("points_for_win={}\n", league.rules().points_for_win()));
    output.push_str(&format!(
        "points_for_overtime_loss={}\n",
        league.rules().points_for_overtime_loss()
    ));
    output.push_str(&format!("points_for_loss={}\n", league.rules().points_for_loss()));
    output.push_str(&format!("max_roster_size={}\n", league.rules().max_roster_size()));
    output.push_str(&format!(
        "playoff_series_length={}\n",
        league.rules().playoff_series_length()
    ));
    output.push_str(&format!("allow_shootout={}\n", bool_to_str(league.rules().allow_shootout())));
    output.push_str(&format!(
        "parent_league={}\n",
        league.rules().parent_league().unwrap_or("none")
    ));
    output.push_str(&format!(
        "affiliated_minor_levels={}\n",
        join_levels(league.rules().affiliated_minor_levels())
    ));
    output
}

fn write_standings_text(standings: &[TeamStanding]) -> String {
    let mut output = String::new();
    for standing in standings {
        output.push_str("STANDING_BEGIN\n");
        output.push_str(&format!("team_abbreviation={}\n", standing.team_abbreviation()));
        output.push_str(&format!("games_played={}\n", standing.games_played()));
        output.push_str(&format!("wins={}\n", standing.wins()));
        output.push_str(&format!("losses={}\n", standing.losses()));
        output.push_str(&format!("overtime_losses={}\n", standing.overtime_losses()));
        output.push_str(&format!("goals_for={}\n", standing.goals_for()));
        output.push_str(&format!("goals_against={}\n", standing.goals_against()));
        output.push_str(&format!("points={}\n", standing.points()));
        output.push_str("STANDING_END\n");
    }
    output
}

fn write_season_text(season: &Season) -> String {
    let mut output = String::new();
    output.push_str(&format!("year={}\n", season.year()));
    output.push_str(&format!("games_per_matchup={}\n", season.games_per_matchup()));
    output.push_str(&format!("completed_games={}\n", season.completed_games()));
    output
}

fn write_schedule_text(schedule: &[ScheduledGame]) -> String {
    let mut output = String::new();
    for game in schedule {
        output.push_str("SCHEDULED_GAME_BEGIN\n");
        output.push_str(&format!("day={}\n", game.day()));
        output.push_str(&format!("home_team={}\n", game.home_team()));
        output.push_str(&format!("away_team={}\n", game.away_team()));
        if let Some(result) = game.result() {
            output.push_str("played=true\n");
            output.push_str(&format!("home_goals={}\n", result.home_goals()));
            output.push_str(&format!("away_goals={}\n", result.away_goals()));
            output.push_str(&format!("overtime={}\n", bool_to_str(result.overtime())));
            output.push_str(&format!("shootout={}\n", bool_to_str(result.shootout())));
        } else {
            output.push_str("played=false\n");
        }
        output.push_str("SCHEDULED_GAME_END\n");
    }
    output
}


fn write_staff_text(staff_member: &StaffMember) -> String {
    let mut output = String::new();
    write_staff(&mut output, staff_member);
    output
}







fn sanitize_file_name(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim_matches('_').to_string()
}




fn rebuild_team_stats(games_played: i16, wins: i16, losses: i16, overtime_losses: i16, key: &str, value: i16, current: &TeamStats) -> TeamStats {
    let mut goals_for = current.goals_for();
    let mut goals_against = current.goals_against();
    let mut shots_for = current.shots_for();
    let mut shots_against = current.shots_against();
    let mut power_play_goals = current.power_play_goals();
    let mut power_play_opportunities = current.power_play_opportunities();
    let mut penalty_kill_goals_against = current.penalty_kill_goals_against();
    let mut penalty_kill_opportunities = current.penalty_kill_opportunities();
    let mut faceoff_wins = current.faceoff_wins();
    let mut faceoff_losses = current.faceoff_losses();
    let mut hits = current.hits();
    let mut blocked_shots = current.blocked_shots();

    match key {
        "team_stats_goals_for" => goals_for = value,
        "team_stats_goals_against" => goals_against = value,
        "team_stats_shots_for" => shots_for = value,
        "team_stats_shots_against" => shots_against = value,
        "team_stats_power_play_goals" => power_play_goals = value,
        "team_stats_power_play_opportunities" => power_play_opportunities = value,
        "team_stats_penalty_kill_goals_against" => penalty_kill_goals_against = value,
        "team_stats_penalty_kill_opportunities" => penalty_kill_opportunities = value,
        "team_stats_faceoff_wins" => faceoff_wins = value,
        "team_stats_faceoff_losses" => faceoff_losses = value,
        "team_stats_hits" => hits = value,
        "team_stats_blocked_shots" => blocked_shots = value,
        _ => {}
    }

    TeamStats::new(
        games_played,
        wins,
        losses,
        overtime_losses,
        goals_for,
        goals_against,
        shots_for,
        shots_against,
        power_play_goals,
        power_play_opportunities,
        penalty_kill_goals_against,
        penalty_kill_opportunities,
        faceoff_wins,
        faceoff_losses,
        hits,
        blocked_shots,
    )
}

fn rebuild_contract_settings(key: &str, value: &str, current: &TeamContractSettings) -> Result<TeamContractSettings, String> {
    let mut salary_cap_max_millions = current.salary_cap_max_millions();
    let mut salary_floor_min_millions = current.salary_floor_min_millions();
    let mut max_contracts = current.max_contracts();
    let mut max_retained_salary_slots = current.max_retained_salary_slots();
    let mut min_years = current.limits().min_years();
    let mut max_years = current.limits().max_years();
    let mut min_cap_hit_millions = current.limits().min_cap_hit_millions();
    let mut max_cap_hit_millions = current.limits().max_cap_hit_millions();
    let mut min_salary_millions = current.limits().min_salary_millions();
    let mut max_salary_millions = current.limits().max_salary_millions();
    let mut min_signing_bonus_millions = current.limits().min_signing_bonus_millions();
    let mut max_signing_bonus_millions = current.limits().max_signing_bonus_millions();
    let mut min_performance_bonus_millions = current.limits().min_performance_bonus_millions();
    let mut max_performance_bonus_millions = current.limits().max_performance_bonus_millions();
    let mut min_no_trade_clauses = current.limits().min_no_trade_clauses();
    let mut max_no_trade_clauses = current.limits().max_no_trade_clauses();
    let mut min_no_move_clauses = current.limits().min_no_move_clauses();
    let mut max_no_move_clauses = current.limits().max_no_move_clauses();

    match key {
        "contract_salary_cap_max_millions" => salary_cap_max_millions = parse_f32(value, key)?,
        "contract_salary_floor_min_millions" => salary_floor_min_millions = parse_f32(value, key)?,
        "contract_max_contracts" => max_contracts = parse_i16(value, key)?,
        "contract_max_retained_salary_slots" => max_retained_salary_slots = parse_i8(value, key)?,
        "contract_limit_min_years" => min_years = parse_i8(value, key)?,
        "contract_limit_max_years" => max_years = parse_i8(value, key)?,
        "contract_limit_min_cap_hit_millions" => min_cap_hit_millions = parse_f32(value, key)?,
        "contract_limit_max_cap_hit_millions" => max_cap_hit_millions = parse_f32(value, key)?,
        "contract_limit_min_salary_millions" => min_salary_millions = parse_f32(value, key)?,
        "contract_limit_max_salary_millions" => max_salary_millions = parse_f32(value, key)?,
        "contract_limit_min_signing_bonus_millions" => min_signing_bonus_millions = parse_f32(value, key)?,
        "contract_limit_max_signing_bonus_millions" => max_signing_bonus_millions = parse_f32(value, key)?,
        "contract_limit_min_performance_bonus_millions" => min_performance_bonus_millions = parse_f32(value, key)?,
        "contract_limit_max_performance_bonus_millions" => max_performance_bonus_millions = parse_f32(value, key)?,
        "contract_limit_min_no_trade_clauses" => min_no_trade_clauses = parse_i8(value, key)?,
        "contract_limit_max_no_trade_clauses" => max_no_trade_clauses = parse_i8(value, key)?,
        "contract_limit_min_no_move_clauses" => min_no_move_clauses = parse_i8(value, key)?,
        "contract_limit_max_no_move_clauses" => max_no_move_clauses = parse_i8(value, key)?,
        _ => {}
    }

    Ok(TeamContractSettings::new(
        salary_cap_max_millions,
        salary_floor_min_millions,
        max_contracts,
        max_retained_salary_slots,
        ContractLimits::new(
            min_years,
            max_years,
            min_cap_hit_millions,
            max_cap_hit_millions,
            min_salary_millions,
            max_salary_millions,
            min_signing_bonus_millions,
            max_signing_bonus_millions,
            min_performance_bonus_millions,
            max_performance_bonus_millions,
            min_no_trade_clauses,
            max_no_trade_clauses,
            min_no_move_clauses,
            max_no_move_clauses,
        ),
    ))
}

fn parse_staff_block(lines: &[String]) -> Result<StaffMember, String> {
    let mut name = None;
    let mut age = None;
    let mut role = None;
    let mut teaching = None;
    let mut tactical = None;
    let mut evaluation = None;
    let mut leadership = None;
    let mut current_level = None;
    let mut potential = None;
    let mut growth_rate = None;
    let mut consistency = None;

    for line in lines {
        let (key, value) = split_key_value(line)?;
        match key {
            "name" => name = Some(unescape_text(value)),
            "age" => age = Some(parse_i8(value, key)?),
            "role" => role = Some(parse_staff_role(value)?),
            "teaching" => teaching = Some(parse_i8(value, key)?),
            "tactical" => tactical = Some(parse_i8(value, key)?),
            "evaluation" => evaluation = Some(parse_i8(value, key)?),
            "leadership" => leadership = Some(parse_i8(value, key)?),
            "current_level" => current_level = Some(parse_i8(value, key)?),
            "potential" => potential = Some(parse_i8(value, key)?),
            "growth_rate" => growth_rate = Some(parse_i8(value, key)?),
            "consistency" => consistency = Some(parse_i8(value, key)?),
            _ => {}
        }
    }

    Ok(StaffMember::new_with_development(
        required(name, "name")?,
        required(age, "age")?,
        required(role, "role")?,
        StaffRatings::new(
            required(teaching, "teaching")?,
            required(tactical, "tactical")?,
            required(evaluation, "evaluation")?,
            required(leadership, "leadership")?,
        ),
        StaffDevelopment::new(
            required(current_level, "current_level")?,
            required(potential, "potential")?,
            required(growth_rate, "growth_rate")?,
            required(consistency, "consistency")?,
        ),
    ))
}

fn parse_season_block(lines: &[String]) -> Result<SeasonMeta, String> {
    let mut year = None;
    let mut games_per_matchup = None;
    let mut completed_games = None;

    for line in lines {
        let (key, value) = split_key_value(line)?;
        match key {
            "year" => year = Some(parse_i16(value, key)?),
            "games_per_matchup" => games_per_matchup = Some(parse_i16(value, key)?),
            "completed_games" => completed_games = Some(parse_i16(value, key)?),
            _ => {}
        }
    }

    Ok(SeasonMeta {
        year: required(year, "year")?,
        games_per_matchup: required(games_per_matchup, "games_per_matchup")?,
        completed_games: required(completed_games, "completed_games")?,
    })
}

fn parse_scheduled_game_block(lines: &[String]) -> Result<ScheduledGame, String> {
    let mut day = None;
    let mut home_team = None;
    let mut away_team = None;
    let mut played = false;
    let mut home_goals = 0;
    let mut away_goals = 0;
    let mut overtime = false;
    let mut shootout = false;

    for line in lines {
        let (key, value) = split_key_value(line)?;
        match key {
            "day" => day = Some(parse_i16(value, key)?),
            "home_team" => home_team = Some(value.to_string()),
            "away_team" => away_team = Some(value.to_string()),
            "played" => played = parse_bool(value, key)?,
            "home_goals" => home_goals = parse_i16(value, key)?,
            "away_goals" => away_goals = parse_i16(value, key)?,
            "overtime" => overtime = parse_bool(value, key)?,
            "shootout" => shootout = parse_bool(value, key)?,
            _ => {}
        }
    }

    let mut game = ScheduledGame::new(required(day, "day")?, required(home_team, "home_team")?, required(away_team, "away_team")?);
    if played {
        game.result = Some(ScheduledGameResult::new(home_goals, away_goals, overtime, shootout));
    }
    Ok(game)
}

fn apply_game_to_teams(teams: &mut [Team], home_index: usize, away_index: usize, game: &SimulatedGame) {
    let (home_team, away_team) = two_teams_mut(teams, home_index, away_index);
    let home_shots = estimate_shots(game.home_goals());
    let away_shots = estimate_shots(game.away_goals());
    let home_win = game.home_goals() > game.away_goals();
    let away_win = game.away_goals() > game.home_goals();

    home_team.team_stats_mut().record_game(
        home_win,
        !home_win && game.overtime(),
        game.home_goals(),
        game.away_goals(),
        home_shots,
        away_shots,
        estimate_power_play_goals(game.home_goals()),
        3,
        estimate_power_play_goals(game.away_goals()),
        3,
        30,
        28,
        18,
        14,
    );
    away_team.team_stats_mut().record_game(
        away_win,
        !away_win && game.overtime(),
        game.away_goals(),
        game.home_goals(),
        away_shots,
        home_shots,
        estimate_power_play_goals(game.away_goals()),
        3,
        estimate_power_play_goals(game.home_goals()),
        3,
        28,
        30,
        17,
        15,
    );

    apply_player_boxscore(home_team, game.home_goals(), game.away_goals(), home_shots, home_win, game.overtime());
    apply_player_boxscore(away_team, game.away_goals(), game.home_goals(), away_shots, away_win, game.overtime());
}

fn apply_player_boxscore(team: &mut Team, goals_for: i16, goals_against: i16, shots_for: i16, win: bool, overtime: bool) {
    let goalie_index = team
        .roster()
        .iter()
        .position(|player| matches!(player.player().position(), crate::data::player::Position::GOALIE));
    let skater_indices: Vec<usize> = team
        .roster()
        .iter()
        .enumerate()
        .filter_map(|(index, player)| {
            if matches!(player.player().position(), crate::data::player::Position::GOALIE) {
                None
            } else {
                Some(index)
            }
        })
        .collect();

    for (line_index, player_index) in skater_indices.iter().enumerate() {
        let player = &mut team.roster_mut()[*player_index];
        let base_shots = if line_index < 3 { 3 } else if line_index < 6 { 2 } else { 1 };
        let goal_credit = if (line_index as i16) < goals_for { 1 } else { 0 };
        let assist_credit = if line_index > 0 && (line_index as i16) <= goals_for { 1 } else { 0 };
        let plus_minus = if win { 1 } else if overtime { 0 } else { -1 };
        player.stats_mut().record_skater_game(
            goal_credit,
            assist_credit,
            plus_minus,
            0,
            base_shots.min(shots_for),
            0,
            0,
            1,
            1,
            if line_index < 6 { 18 } else { 12 },
        );
    }

    if let Some(goalie_index) = goalie_index {
        let goalie = &mut team.roster_mut()[goalie_index];
        let shots_against = estimate_shots(goals_against);
        let saves = (shots_against - goals_against).max(0);
        goalie.stats_mut().record_goalie_game(
            win,
            !win && overtime,
            shots_against,
            saves,
            goals_against,
            goals_against == 0,
            60,
        );
    }
}

fn write_team_stats(output: &mut String, team_stats: &TeamStats) {
    output.push_str(&format!("team_stats_games_played={}\n", team_stats.games_played()));
    output.push_str(&format!("team_stats_wins={}\n", team_stats.wins()));
    output.push_str(&format!("team_stats_losses={}\n", team_stats.losses()));
    output.push_str(&format!("team_stats_overtime_losses={}\n", team_stats.overtime_losses()));
    output.push_str(&format!("team_stats_goals_for={}\n", team_stats.goals_for()));
    output.push_str(&format!("team_stats_goals_against={}\n", team_stats.goals_against()));
    output.push_str(&format!("team_stats_shots_for={}\n", team_stats.shots_for()));
    output.push_str(&format!("team_stats_shots_against={}\n", team_stats.shots_against()));
    output.push_str(&format!("team_stats_power_play_goals={}\n", team_stats.power_play_goals()));
    output.push_str(&format!(
        "team_stats_power_play_opportunities={}\n",
        team_stats.power_play_opportunities()
    ));
    output.push_str(&format!(
        "team_stats_penalty_kill_goals_against={}\n",
        team_stats.penalty_kill_goals_against()
    ));
    output.push_str(&format!(
        "team_stats_penalty_kill_opportunities={}\n",
        team_stats.penalty_kill_opportunities()
    ));
    output.push_str(&format!("team_stats_faceoff_wins={}\n", team_stats.faceoff_wins()));
    output.push_str(&format!("team_stats_faceoff_losses={}\n", team_stats.faceoff_losses()));
    output.push_str(&format!("team_stats_hits={}\n", team_stats.hits()));
    output.push_str(&format!("team_stats_blocked_shots={}\n", team_stats.blocked_shots()));
}

fn write_contract_settings(output: &mut String, settings: &TeamContractSettings) {
    output.push_str(&format!("contract_salary_cap_max_millions={:.3}\n", settings.salary_cap_max_millions()));
    output.push_str(&format!("contract_salary_floor_min_millions={:.3}\n", settings.salary_floor_min_millions()));
    output.push_str(&format!("contract_max_contracts={}\n", settings.max_contracts()));
    output.push_str(&format!(
        "contract_max_retained_salary_slots={}\n",
        settings.max_retained_salary_slots()
    ));
    output.push_str(&format!("contract_limit_min_years={}\n", settings.limits().min_years()));
    output.push_str(&format!("contract_limit_max_years={}\n", settings.limits().max_years()));
    output.push_str(&format!("contract_limit_min_cap_hit_millions={:.3}\n", settings.limits().min_cap_hit_millions()));
    output.push_str(&format!("contract_limit_max_cap_hit_millions={:.3}\n", settings.limits().max_cap_hit_millions()));
    output.push_str(&format!("contract_limit_min_salary_millions={:.3}\n", settings.limits().min_salary_millions()));
    output.push_str(&format!("contract_limit_max_salary_millions={:.3}\n", settings.limits().max_salary_millions()));
    output.push_str(&format!(
        "contract_limit_min_signing_bonus_millions={:.3}\n",
        settings.limits().min_signing_bonus_millions()
    ));
    output.push_str(&format!(
        "contract_limit_max_signing_bonus_millions={:.3}\n",
        settings.limits().max_signing_bonus_millions()
    ));
    output.push_str(&format!(
        "contract_limit_min_performance_bonus_millions={:.3}\n",
        settings.limits().min_performance_bonus_millions()
    ));
    output.push_str(&format!(
        "contract_limit_max_performance_bonus_millions={:.3}\n",
        settings.limits().max_performance_bonus_millions()
    ));
    output.push_str(&format!("contract_limit_min_no_trade_clauses={}\n", settings.limits().min_no_trade_clauses()));
    output.push_str(&format!("contract_limit_max_no_trade_clauses={}\n", settings.limits().max_no_trade_clauses()));
    output.push_str(&format!("contract_limit_min_no_move_clauses={}\n", settings.limits().min_no_move_clauses()));
    output.push_str(&format!("contract_limit_max_no_move_clauses={}\n", settings.limits().max_no_move_clauses()));
}

fn write_staff(output: &mut String, staff_member: &StaffMember) {
    output.push_str(&format!("name={}\n", escape_text(staff_member.name())));
    output.push_str(&format!("age={}\n", staff_member.age()));
    output.push_str(&format!("role={}\n", staff_role_to_str(staff_member.role())));
    output.push_str(&format!("teaching={}\n", staff_member.ratings().teaching()));
    output.push_str(&format!("tactical={}\n", staff_member.ratings().tactical()));
    output.push_str(&format!("evaluation={}\n", staff_member.ratings().evaluation()));
    output.push_str(&format!("leadership={}\n", staff_member.ratings().leadership()));
    output.push_str(&format!("current_level={}\n", staff_member.development().current_level()));
    output.push_str(&format!("potential={}\n", staff_member.development().potential()));
    output.push_str(&format!("growth_rate={}\n", staff_member.development().growth_rate()));
    output.push_str(&format!("consistency={}\n", staff_member.development().consistency()));
}

fn find_team_index(teams: &[Team], abbreviation: &str) -> Result<usize, String> {
    teams
        .iter()
        .position(|team| team.identity().abbreviation() == abbreviation)
        .ok_or_else(|| format!("missing team: {abbreviation}"))
}

fn two_teams_mut(teams: &mut [Team], left_index: usize, right_index: usize) -> (&mut Team, &mut Team) {
    if left_index < right_index {
        let (left, right) = teams.split_at_mut(right_index);
        (&mut left[left_index], &mut right[0])
    } else {
        let (left, right) = teams.split_at_mut(left_index);
        (&mut right[0], &mut left[right_index])
    }
}

fn estimate_shots(goals: i16) -> i16 {
    24 + goals * 4
}

fn estimate_power_play_goals(goals: i16) -> i16 {
    if goals >= 4 { 1 } else { 0 }
}

fn decision_suffix(game: &SimulatedGame) -> &'static str {
    if game.shootout() {
        " SO"
    } else if game.overtime() {
        " OT"
    } else {
        ""
    }
}

fn required<T>(value: Option<T>, field: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required field: {field}"))
}

fn split_key_value<'a>(line: &'a str) -> Result<(&'a str, &'a str), String> {
    line.split_once('=').ok_or_else(|| format!("invalid line: {line}"))
}

fn parse_i8(value: &str, field: &str) -> Result<i8, String> {
    value.parse::<i8>().map_err(|_| format!("invalid i8 for {field}: {value}"))
}

fn parse_i16(value: &str, field: &str) -> Result<i16, String> {
    value.parse::<i16>().map_err(|_| format!("invalid i16 for {field}: {value}"))
}

fn parse_f32(value: &str, field: &str) -> Result<f32, String> {
    value.parse::<f32>().map_err(|_| format!("invalid f32 for {field}: {value}"))
}

fn parse_bool(value: &str, field: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid bool for {field}: {value}")),
    }
}

fn bool_to_str(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn escape_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn unescape_text(value: &str) -> String {
    value.replace("\\n", "\n").replace("\\\\", "\\")
}

fn team_level_to_str(value: &TeamLevel) -> &'static str {
    match value {
        TeamLevel::MAJOR_PRO => "MAJOR_PRO",
        TeamLevel::MINOR_PRO => "MINOR_PRO",
        TeamLevel::JUNIOR => "JUNIOR",
        TeamLevel::COLLEGE => "COLLEGE",
        TeamLevel::INTERNATIONAL => "INTERNATIONAL",
        TeamLevel::OTHER => "OTHER",
    }
}

fn parse_team_level(value: &str) -> Result<TeamLevel, String> {
    match value {
        "MAJOR_PRO" => Ok(TeamLevel::MAJOR_PRO),
        "MINOR_PRO" => Ok(TeamLevel::MINOR_PRO),
        "JUNIOR" => Ok(TeamLevel::JUNIOR),
        "COLLEGE" => Ok(TeamLevel::COLLEGE),
        "INTERNATIONAL" => Ok(TeamLevel::INTERNATIONAL),
        "OTHER" => Ok(TeamLevel::OTHER),
        _ => Err(format!("invalid team level: {value}")),
    }
}






fn join_levels(levels: &[TeamLevel]) -> String {
    levels.iter().map(team_level_to_str).collect::<Vec<_>>().join(",")
}

fn parse_levels(value: &str) -> Result<Vec<TeamLevel>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value.split(',').map(parse_team_level).collect()
}

fn same_team_level(left: &TeamLevel, right: &TeamLevel) -> bool {
    matches!(
        (left, right),
        (TeamLevel::MAJOR_PRO, TeamLevel::MAJOR_PRO)
            | (TeamLevel::MINOR_PRO, TeamLevel::MINOR_PRO)
            | (TeamLevel::JUNIOR, TeamLevel::JUNIOR)
            | (TeamLevel::COLLEGE, TeamLevel::COLLEGE)
            | (TeamLevel::INTERNATIONAL, TeamLevel::INTERNATIONAL)
            | (TeamLevel::OTHER, TeamLevel::OTHER)
    )
}

fn staff_role_to_str(value: &StaffRole) -> &'static str {
    match value {
        StaffRole::GENERAL_MANAGER => "GENERAL_MANAGER",
        StaffRole::ASSISTANT_GENERAL_MANAGER => "ASSISTANT_GENERAL_MANAGER",
        StaffRole::HEAD_COACH => "HEAD_COACH",
        StaffRole::ASSISTANT_COACH => "ASSISTANT_COACH",
        StaffRole::DEVELOPMENT_COACH => "DEVELOPMENT_COACH",
        StaffRole::HEAD_SCOUT => "HEAD_SCOUT",
        StaffRole::GOALIE_COACH => "GOALIE_COACH",
        StaffRole::SKATING_COACH => "SKATING_COACH",
        StaffRole::SCOUT => "SCOUT",
        StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT => "DIRECTOR_OF_PLAYER_DEVELOPMENT",
        StaffRole::OWNER => "OWNER",
    }
}

fn parse_staff_role(value: &str) -> Result<StaffRole, String> {
    match value {
        "GENERAL_MANAGER" => Ok(StaffRole::GENERAL_MANAGER),
        "ASSISTANT_GENERAL_MANAGER" => Ok(StaffRole::ASSISTANT_GENERAL_MANAGER),
        "HEAD_COACH" => Ok(StaffRole::HEAD_COACH),
        "ASSISTANT_COACH" => Ok(StaffRole::ASSISTANT_COACH),
        "DEVELOPMENT_COACH" => Ok(StaffRole::DEVELOPMENT_COACH),
        "HEAD_SCOUT" => Ok(StaffRole::HEAD_SCOUT),
        "GOALIE_COACH" => Ok(StaffRole::GOALIE_COACH),
        "SKATING_COACH" => Ok(StaffRole::SKATING_COACH),
        "SCOUT" => Ok(StaffRole::SCOUT),
        "DIRECTOR_OF_PLAYER_DEVELOPMENT" => Ok(StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT),
        "OWNER" => Ok(StaffRole::OWNER),
        _ => Err(format!("invalid staff role: {value}")),
    }
}
