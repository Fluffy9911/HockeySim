use crate::data::contract::{ContractLimits, TeamContractSettings};
use crate::data::helper::PlayerRecord;
use crate::data::staff::{StaffDevelopment, StaffMember, StaffRatings, StaffRole};
use crate::data::stats::TeamStats;
use crate::data::team::{Conference, Division, Team, TeamIdentity, TeamLevel};
use crate::sim::{GameContext, League, LeagueRules, LeagueTeamEntry, SimulationEngine, SimulatedGame, TeamStanding};
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
    pub fn write_text(&self) -> String {
        let mut output = String::new();
        output.push_str("LEAGUE_BEGIN\n");
        output.push_str(&format!("name={}\n", escape_text(self.league.name())));
        output.push_str(&format!("level={}\n", team_level_to_str(self.league.level())));
        output.push_str(&format!("points_for_win={}\n", self.league.rules().points_for_win()));
        output.push_str(&format!(
            "points_for_overtime_loss={}\n",
            self.league.rules().points_for_overtime_loss()
        ));
        output.push_str(&format!("points_for_loss={}\n", self.league.rules().points_for_loss()));
        output.push_str(&format!("max_roster_size={}\n", self.league.rules().max_roster_size()));
        output.push_str(&format!(
            "playoff_series_length={}\n",
            self.league.rules().playoff_series_length()
        ));
        output.push_str(&format!("allow_shootout={}\n", bool_to_str(self.league.rules().allow_shootout())));
        output.push_str(&format!(
            "parent_league={}\n",
            self.league.rules().parent_league().unwrap_or("none")
        ));
        output.push_str(&format!(
            "affiliated_minor_levels={}\n",
            join_levels(self.league.rules().affiliated_minor_levels())
        ));
        output.push_str("LEAGUE_END\n");

        for standing in self.league.standings() {
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

        for team in &self.teams {
            output.push_str("TEAM_BEGIN\n");
            output.push_str(&format!("city={}\n", escape_text(team.identity().city())));
            output.push_str(&format!("name={}\n", escape_text(team.identity().name())));
            output.push_str(&format!("abbreviation={}\n", team.identity().abbreviation()));
            output.push_str(&format!("conference={}\n", conference_to_str(team.identity().conference())));
            output.push_str(&format!("division={}\n", division_to_str(team.identity().division())));
            output.push_str(&format!("level={}\n", team_level_to_str(team.level())));
            output.push_str(&format!("affiliate_team_abbreviations={}\n", team.affiliate_team_abbreviations().join(",")));
            write_team_stats(&mut output, team.team_stats());
            write_contract_settings(&mut output, team.contract_settings());

            for player in team.roster() {
                output.push_str("PLAYER_BEGIN\n");
                output.push_str(&player.write_text());
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str("PLAYER_END\n");
            }

            for staff_member in team.staff() {
                output.push_str("STAFF_BEGIN\n");
                write_staff(&mut output, staff_member);
                output.push_str("STAFF_END\n");
            }

            output.push_str("TEAM_END\n");
        }

        if let Some(season) = &self.season {
            output.push_str("SEASON_BEGIN\n");
            output.push_str(&format!("year={}\n", season.year()));
            output.push_str(&format!("games_per_matchup={}\n", season.games_per_matchup()));
            output.push_str(&format!("completed_games={}\n", season.completed_games()));
            output.push_str("SEASON_END\n");

            for game in season.schedule() {
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
        }

        output
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        fs::write(path, self.write_text()).map_err(|error| format!("failed to write {path}: {error}"))
    }

    pub fn save_to_directory(&self, path: &str) -> Result<(), String> {
        let root = Path::new(path);
        let league_dir = root.join("league");
        let team_root = league_dir.join("team");

        fs::create_dir_all(&team_root)
            .map_err(|error| format!("failed to create {}: {error}", team_root.display()))?;

        fs::write(league_dir.join("metadata.txt"), write_league_metadata_text(&self.league))
            .map_err(|error| format!("failed to write league metadata: {error}"))?;
        fs::write(league_dir.join("standings.txt"), write_standings_text(self.league.standings()))
            .map_err(|error| format!("failed to write standings: {error}"))?;

        if let Some(season) = &self.season {
            fs::write(league_dir.join("season.txt"), write_season_text(season))
                .map_err(|error| format!("failed to write season metadata: {error}"))?;
            fs::write(league_dir.join("schedule.txt"), write_schedule_text(season.schedule()))
                .map_err(|error| format!("failed to write schedule: {error}"))?;
        }

        for team in &self.teams {
            let team_dir = team_root.join(team.identity().abbreviation());
            let player_dir = team_dir.join("player");
            let staff_dir = team_dir.join("staff");

            fs::create_dir_all(&player_dir)
                .map_err(|error| format!("failed to create {}: {error}", player_dir.display()))?;
            fs::create_dir_all(&staff_dir)
                .map_err(|error| format!("failed to create {}: {error}", staff_dir.display()))?;

            fs::write(team_dir.join("team.txt"), write_team_metadata_text(team))
                .map_err(|error| format!("failed to write team file for {}: {error}", team.identity().abbreviation()))?;

            for (index, player) in team.roster().iter().enumerate() {
                let filename = format!("{:03}_{}.txt", index + 1, sanitize_file_name(player.name()));
                fs::write(player_dir.join(filename), player.write_text())
                    .map_err(|error| format!("failed to write player file for {}: {error}", team.identity().abbreviation()))?;
            }

            for (index, staff_member) in team.staff().iter().enumerate() {
                let filename = format!("{:03}_{}.txt", index + 1, sanitize_file_name(staff_member.name()));
                fs::write(staff_dir.join(filename), write_staff_text(staff_member))
                    .map_err(|error| format!("failed to write staff file for {}: {error}", team.identity().abbreviation()))?;
            }
        }

        Ok(())
    }

    pub fn read_text(input: &str) -> Result<LeagueState, String> {
        let mut league_meta = LeagueMeta::default();
        let mut standings = Vec::new();
        let mut teams = Vec::new();
        let mut season_meta = None;
        let mut scheduled_games = Vec::new();
        let lines: Vec<&str> = input.lines().collect();
        let mut index = 0;

        while index < lines.len() {
            match lines[index].trim() {
                "LEAGUE_BEGIN" => {
                    let (block, next_index) = collect_block(&lines, index + 1, "LEAGUE_END")?;
                    league_meta = parse_league_block(&block)?;
                    index = next_index;
                }
                "STANDING_BEGIN" => {
                    let (block, next_index) = collect_block(&lines, index + 1, "STANDING_END")?;
                    standings.push(parse_standing_block(&block)?);
                    index = next_index;
                }
                "TEAM_BEGIN" => {
                    let (team, next_index) = parse_team_block(&lines, index + 1)?;
                    teams.push(team);
                    index = next_index;
                }
                "SEASON_BEGIN" => {
                    let (block, next_index) = collect_block(&lines, index + 1, "SEASON_END")?;
                    season_meta = Some(parse_season_block(&block)?);
                    index = next_index;
                }
                "SCHEDULED_GAME_BEGIN" => {
                    let (block, next_index) = collect_block(&lines, index + 1, "SCHEDULED_GAME_END")?;
                    scheduled_games.push(parse_scheduled_game_block(&block)?);
                    index = next_index;
                }
                _ => index += 1,
            }
        }

        let team_registry = teams.iter().map(LeagueTeamEntry::from_team).collect();
        let rules = LeagueRules::new(
            league_meta.points_for_win,
            league_meta.points_for_overtime_loss,
            league_meta.points_for_loss,
            league_meta.max_roster_size,
            league_meta.playoff_series_length,
            league_meta.allow_shootout,
            league_meta.parent_league,
            league_meta.affiliated_minor_levels,
        );
        let league = League::new_custom(league_meta.name, league_meta.level, rules, team_registry, standings);
        let season = season_meta.map(|meta| Season {
            year: meta.year,
            games_per_matchup: meta.games_per_matchup,
            completed_games: meta.completed_games,
            schedule: scheduled_games,
        });

        Ok(LeagueState { league, teams, season })
    }

    pub fn load_from_file(path: &str) -> Result<LeagueState, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
        LeagueState::read_text(&text)
    }

    pub fn load_from_directory(path: &str) -> Result<LeagueState, String> {
        let root = Path::new(path);
        let league_dir = root.join("league");
        let team_root = league_dir.join("team");

        let league_metadata = fs::read_to_string(league_dir.join("metadata.txt"))
            .map_err(|error| format!("failed to read league metadata: {error}"))?;
        let league_meta = parse_league_block(&collect_owned_lines(&league_metadata))?;

        let standings_path = league_dir.join("standings.txt");
        let standings = if standings_path.exists() {
            parse_standings_text(&fs::read_to_string(&standings_path).map_err(|error| format!("failed to read standings: {error}"))?)?
        } else {
            Vec::new()
        };

        let mut teams = Vec::new();
        if team_root.exists() {
            let mut team_dirs: Vec<PathBuf> = fs::read_dir(&team_root)
                .map_err(|error| format!("failed to read {}: {error}", team_root.display()))?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
            team_dirs.sort();

            for team_dir in team_dirs {
                teams.push(read_team_from_directory(&team_dir)?);
            }
        }

        let team_registry = teams.iter().map(LeagueTeamEntry::from_team).collect();
        let rules = LeagueRules::new(
            league_meta.points_for_win,
            league_meta.points_for_overtime_loss,
            league_meta.points_for_loss,
            league_meta.max_roster_size,
            league_meta.playoff_series_length,
            league_meta.allow_shootout,
            league_meta.parent_league,
            league_meta.affiliated_minor_levels,
        );
        let league = League::new_custom(league_meta.name, league_meta.level, rules, team_registry, standings);

        let season = if league_dir.join("season.txt").exists() {
            let season_text = fs::read_to_string(league_dir.join("season.txt"))
                .map_err(|error| format!("failed to read season metadata: {error}"))?;
            let meta = parse_season_block(&collect_owned_lines(&season_text))?;
            let schedule_path = league_dir.join("schedule.txt");
            let schedule = if schedule_path.exists() {
                parse_schedule_text(&fs::read_to_string(schedule_path).map_err(|error| format!("failed to read schedule: {error}"))?)?
            } else {
                Vec::new()
            };
            Some(Season {
                year: meta.year,
                games_per_matchup: meta.games_per_matchup,
                completed_games: meta.completed_games,
                schedule,
            })
        } else {
            None
        };

        Ok(LeagueState { league, teams, season })
    }
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

fn write_team_metadata_text(team: &Team) -> String {
    let mut output = String::new();
    output.push_str(&format!("city={}\n", escape_text(team.identity().city())));
    output.push_str(&format!("name={}\n", escape_text(team.identity().name())));
    output.push_str(&format!("abbreviation={}\n", team.identity().abbreviation()));
    output.push_str(&format!("conference={}\n", conference_to_str(team.identity().conference())));
    output.push_str(&format!("division={}\n", division_to_str(team.identity().division())));
    output.push_str(&format!("level={}\n", team_level_to_str(team.level())));
    output.push_str(&format!(
        "affiliate_team_abbreviations={}\n",
        team.affiliate_team_abbreviations().join(",")
    ));
    write_team_stats(&mut output, team.team_stats());
    write_contract_settings(&mut output, team.contract_settings());
    output
}

fn write_staff_text(staff_member: &StaffMember) -> String {
    let mut output = String::new();
    write_staff(&mut output, staff_member);
    output
}

fn parse_standings_text(input: &str) -> Result<Vec<TeamStanding>, String> {
    let lines: Vec<&str> = input.lines().collect();
    let mut standings = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if lines[index].trim() == "STANDING_BEGIN" {
            let (block, next_index) = collect_block(&lines, index + 1, "STANDING_END")?;
            standings.push(parse_standing_block(&block)?);
            index = next_index;
        } else {
            index += 1;
        }
    }

    Ok(standings)
}

fn parse_schedule_text(input: &str) -> Result<Vec<ScheduledGame>, String> {
    let lines: Vec<&str> = input.lines().collect();
    let mut schedule = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if lines[index].trim() == "SCHEDULED_GAME_BEGIN" {
            let (block, next_index) = collect_block(&lines, index + 1, "SCHEDULED_GAME_END")?;
            schedule.push(parse_scheduled_game_block(&block)?);
            index = next_index;
        } else {
            index += 1;
        }
    }

    Ok(schedule)
}

fn read_team_from_directory(team_dir: &Path) -> Result<Team, String> {
    let metadata = fs::read_to_string(team_dir.join("team.txt"))
        .map_err(|error| format!("failed to read {}: {error}", team_dir.join("team.txt").display()))?;
    let mut builder = TeamBuilder {
        city: String::new(),
        name: String::new(),
        abbreviation: String::new(),
        conference: Conference::EAST,
        division: Division::OTHER,
        level: TeamLevel::MAJOR_PRO,
        affiliate_team_abbreviations: Vec::new(),
        team_stats: TeamStats::default(),
        contract_settings: TeamContractSettings::nhl_default(),
        roster: Vec::new(),
        staff: Vec::new(),
    };

    for line in collect_owned_lines(&metadata) {
        let (key, value) = split_key_value(&line)?;
        match key {
            "city" => builder.city = unescape_text(value),
            "name" => builder.name = unescape_text(value),
            "abbreviation" => builder.abbreviation = value.to_string(),
            "conference" => builder.conference = parse_conference(value)?,
            "division" => builder.division = parse_division(value)?,
            "level" => builder.level = parse_team_level(value)?,
            "affiliate_team_abbreviations" => {
                builder.affiliate_team_abbreviations =
                    if value.is_empty() { Vec::new() } else { value.split(',').map(|entry| entry.to_string()).collect() };
            }
            _ => apply_team_builder_field(&mut builder, key, value)?,
        }
    }

    let player_dir = team_dir.join("player");
    if player_dir.exists() {
        let mut player_files: Vec<PathBuf> = fs::read_dir(&player_dir)
            .map_err(|error| format!("failed to read {}: {error}", player_dir.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();
        player_files.sort();

        for path in player_files {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            builder.roster.push(PlayerRecord::read_text(&text)?);
        }
    }

    let staff_dir = team_dir.join("staff");
    if staff_dir.exists() {
        let mut staff_files: Vec<PathBuf> = fs::read_dir(&staff_dir)
            .map_err(|error| format!("failed to read {}: {error}", staff_dir.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();
        staff_files.sort();

        for path in staff_files {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            builder.staff.push(parse_staff_block(&collect_owned_lines(&text))?);
        }
    }

    Ok(Team::new_full(
        TeamIdentity::new(
            builder.city,
            builder.name,
            builder.abbreviation,
            builder.conference,
            builder.division,
        ),
        builder.level,
        builder.roster,
        builder.staff,
        builder.team_stats,
        builder.contract_settings,
        builder.affiliate_team_abbreviations,
    ))
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

fn parse_league_block(lines: &[String]) -> Result<LeagueMeta, String> {
    let mut meta = LeagueMeta::default();

    for line in lines {
        let (key, value) = split_key_value(line)?;
        match key {
            "name" => meta.name = unescape_text(value),
            "level" => meta.level = parse_team_level(value)?,
            "points_for_win" => meta.points_for_win = parse_i16(value, key)?,
            "points_for_overtime_loss" => meta.points_for_overtime_loss = parse_i16(value, key)?,
            "points_for_loss" => meta.points_for_loss = parse_i16(value, key)?,
            "max_roster_size" => meta.max_roster_size = parse_i16(value, key)?,
            "playoff_series_length" => meta.playoff_series_length = parse_i16(value, key)?,
            "allow_shootout" => meta.allow_shootout = parse_bool(value, key)?,
            "parent_league" => meta.parent_league = if value == "none" { None } else { Some(value.to_string()) },
            "affiliated_minor_levels" => meta.affiliated_minor_levels = parse_levels(value)?,
            _ => {}
        }
    }

    Ok(meta)
}

fn parse_standing_block(lines: &[String]) -> Result<TeamStanding, String> {
    let mut team_abbreviation = None;
    let mut games_played = None;
    let mut wins = None;
    let mut losses = None;
    let mut overtime_losses = None;
    let mut goals_for = None;
    let mut goals_against = None;
    let mut points = None;

    for line in lines {
        let (key, value) = split_key_value(line)?;
        match key {
            "team_abbreviation" => team_abbreviation = Some(value.to_string()),
            "games_played" => games_played = Some(parse_i16(value, key)?),
            "wins" => wins = Some(parse_i16(value, key)?),
            "losses" => losses = Some(parse_i16(value, key)?),
            "overtime_losses" => overtime_losses = Some(parse_i16(value, key)?),
            "goals_for" => goals_for = Some(parse_i16(value, key)?),
            "goals_against" => goals_against = Some(parse_i16(value, key)?),
            "points" => points = Some(parse_i16(value, key)?),
            _ => {}
        }
    }

    Ok(TeamStanding::new_full(
        required(team_abbreviation, "team_abbreviation")?,
        required(games_played, "games_played")?,
        required(wins, "wins")?,
        required(losses, "losses")?,
        required(overtime_losses, "overtime_losses")?,
        required(goals_for, "goals_for")?,
        required(goals_against, "goals_against")?,
        required(points, "points")?,
    ))
}

fn parse_team_block(lines: &[&str], start: usize) -> Result<(Team, usize), String> {
    let mut builder = TeamBuilder {
        city: String::new(),
        name: String::new(),
        abbreviation: String::new(),
        conference: Conference::EAST,
        division: Division::OTHER,
        level: TeamLevel::MAJOR_PRO,
        affiliate_team_abbreviations: Vec::new(),
        team_stats: TeamStats::default(),
        contract_settings: TeamContractSettings::nhl_default(),
        roster: Vec::new(),
        staff: Vec::new(),
    };
    let mut index = start;

    while index < lines.len() {
        match lines[index].trim() {
            "TEAM_END" => {
                let team = Team::new_full(
                    TeamIdentity::new(
                        builder.city,
                        builder.name,
                        builder.abbreviation,
                        builder.conference,
                        builder.division,
                    ),
                    builder.level,
                    builder.roster,
                    builder.staff,
                    builder.team_stats,
                    builder.contract_settings,
                    builder.affiliate_team_abbreviations,
                );
                return Ok((team, index + 1));
            }
            "PLAYER_BEGIN" => {
                let (block, next_index) = collect_block(lines, index + 1, "PLAYER_END")?;
                builder.roster.push(PlayerRecord::read_text(&block.join("\n"))?);
                index = next_index;
            }
            "STAFF_BEGIN" => {
                let (block, next_index) = collect_block(lines, index + 1, "STAFF_END")?;
                builder.staff.push(parse_staff_block(&block)?);
                index = next_index;
            }
            raw => {
                let (key, value) = split_key_value(raw)?;
                match key {
                    "city" => builder.city = unescape_text(value),
                    "name" => builder.name = unescape_text(value),
                    "abbreviation" => builder.abbreviation = value.to_string(),
                    "conference" => builder.conference = parse_conference(value)?,
                    "division" => builder.division = parse_division(value)?,
                    "level" => builder.level = parse_team_level(value)?,
                    "affiliate_team_abbreviations" => {
                        builder.affiliate_team_abbreviations =
                            if value.is_empty() { Vec::new() } else { value.split(',').map(|entry| entry.to_string()).collect() };
                    }
                    _ => apply_team_builder_field(&mut builder, key, value)?,
                }
                index += 1;
            }
        }
    }

    Err("unterminated TEAM block".to_string())
}

fn apply_team_builder_field(builder: &mut TeamBuilder, key: &str, value: &str) -> Result<(), String> {
    match key {
        "team_stats_games_played" => builder.team_stats = TeamStats::new(
            parse_i16(value, key)?,
            builder.team_stats.wins(),
            builder.team_stats.losses(),
            builder.team_stats.overtime_losses(),
            builder.team_stats.goals_for(),
            builder.team_stats.goals_against(),
            builder.team_stats.shots_for(),
            builder.team_stats.shots_against(),
            builder.team_stats.power_play_goals(),
            builder.team_stats.power_play_opportunities(),
            builder.team_stats.penalty_kill_goals_against(),
            builder.team_stats.penalty_kill_opportunities(),
            builder.team_stats.faceoff_wins(),
            builder.team_stats.faceoff_losses(),
            builder.team_stats.hits(),
            builder.team_stats.blocked_shots(),
        ),
        "team_stats_wins" => builder.team_stats = TeamStats::new(
            builder.team_stats.games_played(),
            parse_i16(value, key)?,
            builder.team_stats.losses(),
            builder.team_stats.overtime_losses(),
            builder.team_stats.goals_for(),
            builder.team_stats.goals_against(),
            builder.team_stats.shots_for(),
            builder.team_stats.shots_against(),
            builder.team_stats.power_play_goals(),
            builder.team_stats.power_play_opportunities(),
            builder.team_stats.penalty_kill_goals_against(),
            builder.team_stats.penalty_kill_opportunities(),
            builder.team_stats.faceoff_wins(),
            builder.team_stats.faceoff_losses(),
            builder.team_stats.hits(),
            builder.team_stats.blocked_shots(),
        ),
        "team_stats_losses" => builder.team_stats = TeamStats::new(
            builder.team_stats.games_played(),
            builder.team_stats.wins(),
            parse_i16(value, key)?,
            builder.team_stats.overtime_losses(),
            builder.team_stats.goals_for(),
            builder.team_stats.goals_against(),
            builder.team_stats.shots_for(),
            builder.team_stats.shots_against(),
            builder.team_stats.power_play_goals(),
            builder.team_stats.power_play_opportunities(),
            builder.team_stats.penalty_kill_goals_against(),
            builder.team_stats.penalty_kill_opportunities(),
            builder.team_stats.faceoff_wins(),
            builder.team_stats.faceoff_losses(),
            builder.team_stats.hits(),
            builder.team_stats.blocked_shots(),
        ),
        "team_stats_overtime_losses" => builder.team_stats = TeamStats::new(
            builder.team_stats.games_played(),
            builder.team_stats.wins(),
            builder.team_stats.losses(),
            parse_i16(value, key)?,
            builder.team_stats.goals_for(),
            builder.team_stats.goals_against(),
            builder.team_stats.shots_for(),
            builder.team_stats.shots_against(),
            builder.team_stats.power_play_goals(),
            builder.team_stats.power_play_opportunities(),
            builder.team_stats.penalty_kill_goals_against(),
            builder.team_stats.penalty_kill_opportunities(),
            builder.team_stats.faceoff_wins(),
            builder.team_stats.faceoff_losses(),
            builder.team_stats.hits(),
            builder.team_stats.blocked_shots(),
        ),
        "team_stats_goals_for" | "team_stats_goals_against" | "team_stats_shots_for" | "team_stats_shots_against"
        | "team_stats_power_play_goals" | "team_stats_power_play_opportunities"
        | "team_stats_penalty_kill_goals_against" | "team_stats_penalty_kill_opportunities"
        | "team_stats_faceoff_wins" | "team_stats_faceoff_losses" | "team_stats_hits" | "team_stats_blocked_shots" => {
            builder.team_stats = rebuild_team_stats(
                builder.team_stats.games_played(),
                builder.team_stats.wins(),
                builder.team_stats.losses(),
                builder.team_stats.overtime_losses(),
                key,
                parse_i16(value, key)?,
                &builder.team_stats,
            );
        }
        "contract_salary_cap_max_millions" | "contract_salary_floor_min_millions" | "contract_max_contracts"
        | "contract_max_retained_salary_slots" | "contract_limit_min_years" | "contract_limit_max_years"
        | "contract_limit_min_cap_hit_millions" | "contract_limit_max_cap_hit_millions"
        | "contract_limit_min_salary_millions" | "contract_limit_max_salary_millions"
        | "contract_limit_min_signing_bonus_millions" | "contract_limit_max_signing_bonus_millions"
        | "contract_limit_min_performance_bonus_millions" | "contract_limit_max_performance_bonus_millions"
        | "contract_limit_min_no_trade_clauses" | "contract_limit_max_no_trade_clauses"
        | "contract_limit_min_no_move_clauses" | "contract_limit_max_no_move_clauses" => {
            builder.contract_settings = rebuild_contract_settings(key, value, &builder.contract_settings)?;
        }
        _ => {}
    }
    Ok(())
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

fn conference_to_str(value: &Conference) -> &'static str {
    match value {
        Conference::EAST => "EAST",
        Conference::WEST => "WEST",
    }
}

fn parse_conference(value: &str) -> Result<Conference, String> {
    match value {
        "EAST" => Ok(Conference::EAST),
        "WEST" => Ok(Conference::WEST),
        _ => Err(format!("invalid conference: {value}")),
    }
}

fn division_to_str(value: &Division) -> &'static str {
    match value {
        Division::ATLANTIC => "ATLANTIC",
        Division::METROPOLITAN => "METROPOLITAN",
        Division::CENTRAL => "CENTRAL",
        Division::PACIFIC => "PACIFIC",
        Division::OTHER => "OTHER",
    }
}

fn parse_division(value: &str) -> Result<Division, String> {
    match value {
        "ATLANTIC" => Ok(Division::ATLANTIC),
        "METROPOLITAN" => Ok(Division::METROPOLITAN),
        "CENTRAL" => Ok(Division::CENTRAL),
        "PACIFIC" => Ok(Division::PACIFIC),
        "OTHER" => Ok(Division::OTHER),
        _ => Err(format!("invalid division: {value}")),
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
