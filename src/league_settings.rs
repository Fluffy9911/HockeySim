use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::data::general_data::NameData;
use crate::data::player;
use crate::data::player::Player;
use crate::data::team::{Team, TeamLevel};
use crate::savestate::savedata::{create_path_for_type, FileType, SaveContext};
use crate::sim_helper;
use crate::sim_helper::*;

pub struct SimulationEngine {
    settings: SimulationSettings,
}

pub struct SimulationSettings {
  pub  home_ice_advantage: f32,
    pub  coach_weight: f32,
    pub play_style_weight: f32,
    pub standings_weight: f32,
    pub randomness_weight: f32,
}

#[derive(Serialize,Deserialize)]
pub struct League {
    pub name: String,
    level: TeamLevel,
    rules: LeagueRules,
    teams: Vec<Team>,
    standings: Vec<TeamStanding>,
    free_agents: Vec<Player>
}
#[derive(Serialize,Deserialize)]
pub struct LeagueRules {
    points_for_win: i16,
    points_for_overtime_loss: i16,
    points_for_loss: i16,
    max_roster_size: i16,
    playoff_series_length: i16,
    allow_shootout: bool,
    parent_league: Option<String>,
    affiliated_minor_levels: Vec<TeamLevel>,
}
#[derive(Serialize,Deserialize)]
pub struct TeamStanding {
    team_abbreviation: String,
    pub(crate) games_played: i16,
    pub(crate) wins: i16,
    pub(crate) losses: i16,
    pub(crate) overtime_losses: i16,
    pub(crate) goals_for: i16,
    pub(crate) goals_against: i16,
    pub(crate) points: i16,
}
#[derive(Serialize,Deserialize)]
pub struct TeamProfile {
    overall: f32,
    offense: f32,
    defense: f32,
    goaltending: f32,
    coaching: f32,
    style_bias: f32,
    standings_factor: f32,
}

pub struct GameContext {
    season_game_number: i16,
}
#[derive(Serialize,Deserialize)]
pub struct SimulatedGame {
    home_team: String,
    away_team: String,
    home_goals: i16,
    away_goals: i16,
    overtime: bool,
    shootout: bool,
    home_profile: TeamProfile,
    away_profile: TeamProfile,
}

pub struct PlayoffSeries {
    higher_seed: String,
    lower_seed: String,
    wins_needed: i16,
    higher_seed_wins: i16,
    lower_seed_wins: i16,
    games: Vec<SeriesGameLog>,
    winner: String,
}

pub struct SeriesGameLog {
    game_number: i16,
    venue_team: String,
    matchup: String,
    score_line: String,
    decision: String,
    momentum_note: String,
    home_profile_summary: String,
    away_profile_summary: String,
    game: SimulatedGame,
}

#[derive(Serialize, Deserialize)]
struct LeagueDiskData {
    name: String,
    level: TeamLevel,
    rules: LeagueRules,
    standings: Vec<TeamStanding>,
    free_agents: Vec<Player>,
}

impl SimulationEngine {
    pub fn new(settings: SimulationSettings) -> SimulationEngine {
        SimulationEngine { settings }
    }

    pub fn default() -> SimulationEngine {
        SimulationEngine {
            settings: SimulationSettings::default(),
        }
    }

    pub fn settings(&self) -> &SimulationSettings {
        &self.settings
    }

    pub fn analyze_team(&self, team: &Team, league: Option<&League>) -> TeamProfile {
        sim_helper::build_team_profile(team, league, &self.settings)
    }

    pub fn simulate_game(
        &self,
        home_team: &Team,
        away_team: &Team,
        league: Option<&League>,
        context: GameContext,
        seed: u64,
    ) -> SimulatedGame {
        let home_profile = self.analyze_team(home_team, league);
        let away_profile = self.analyze_team(away_team, league);

        let mut rng = SimRng::new(seed ^ context.season_game_number as u64);
        let game_chaos = sim_helper::centered_random(&mut rng) * self.settings.randomness_weight() * 0.55;
        let home_variance = sim_helper::centered_random(&mut rng) * self.settings.randomness_weight() * 0.75 + game_chaos;
        let away_variance = sim_helper::centered_random(&mut rng) * self.settings.randomness_weight() * 0.75 + game_chaos;
        let home_strength = home_profile.overall()
            + self.settings.home_ice_advantage()
            + sim_helper::weighted_gap(home_profile.coaching(), away_profile.coaching(), self.settings.coach_weight())
            + sim_helper::weighted_gap(home_profile.style_bias(), away_profile.style_bias(), self.settings.play_style_weight())
            + sim_helper::weighted_gap(
                home_profile.standings_factor(),
                away_profile.standings_factor(),
                self.settings.standings_weight(),
            )
            + home_variance;
        let away_strength = away_profile.overall() + away_variance;

        let base_home = sim_helper::team_goal_expectancy(home_profile.offense(), away_profile.defense(), away_profile.goaltending());
        let base_away = sim_helper::team_goal_expectancy(away_profile.offense(), home_profile.defense(), home_profile.goaltending());

        let home_goals = sim_helper::finalize_goal_total(
            base_home + (home_strength - away_strength) * 1.1,
            &self.settings,
            &mut rng,
        );
        let away_goals = sim_helper::finalize_goal_total(
            base_away + (away_strength - home_strength) * 0.9,
            &self.settings,
            &mut rng,
        );

        let (home_goals, away_goals, overtime, shootout) = sim_helper::resolve_tie(home_goals, away_goals, &mut rng);

        SimulatedGame {
            home_team: home_team.abbreviation().to_string(),
            away_team: away_team.abbreviation().to_string(),
            home_goals,
            away_goals,
            overtime,
            shootout,
            home_profile,
            away_profile,
        }
    }

    pub fn simulate_best_of_seven(
        &self,
        higher_seed: &Team,
        lower_seed: &Team,
        league: Option<&League>,
        seed: u64,
    ) -> PlayoffSeries {
        let mut higher_seed_wins = 0;
        let mut lower_seed_wins = 0;
        let mut games = Vec::new();
        let schedule = [true, true, false, false, true, false, true];

        for (index, higher_seed_home) in schedule.iter().enumerate() {
            if higher_seed_wins == 4 || lower_seed_wins == 4 {
                break;
            }

            let game_number = index as i16 + 1;
            let (home_team, away_team, venue_team) = if *higher_seed_home {
                (
                    higher_seed,
                    lower_seed,
                    higher_seed.abbreviation().to_string(),
                )
            } else {
                (
                    lower_seed,
                    higher_seed,
                    lower_seed.abbreviation().to_string(),
                )
            };

            let game = self.simulate_game(
                home_team,
                away_team,
                league,
                GameContext::new(100 + game_number),
                seed ^ (game_number as u64 * 7919),
            );

            let higher_seed_won = if *higher_seed_home {
                game.home_goals() > game.away_goals()
            } else {
                game.away_goals() > game.home_goals()
            };

            if higher_seed_won {
                higher_seed_wins += 1;
            } else {
                lower_seed_wins += 1;
            }

            let decision = sim_helper::format_decision(&game);
            let matchup = format!(
                "{} at {}",
                game.away_team(),
                game.home_team(),
            );
            let score_line = format!(
                "{} {} - {} {}",
                game.away_team(),
                game.away_goals(),
                game.home_goals(),
                game.home_team(),
            );
            let momentum_note = format!(
                "Series after Game {}: {} {}, {} {}",
                game_number,
                higher_seed.abbreviation(),
                higher_seed_wins,
                lower_seed.abbreviation(),
                lower_seed_wins,
            );

            games.push(SeriesGameLog {
                game_number,
                venue_team,
                matchup,
                score_line,
                decision,
                momentum_note,
                home_profile_summary: sim_helper::summarize_profile(game.home_team(), game.home_profile()),
                away_profile_summary: sim_helper::summarize_profile(game.away_team(), game.away_profile()),
                game,
            });
        }

        let winner = if higher_seed_wins > lower_seed_wins {
            higher_seed.abbreviation().to_string()
        } else {
            lower_seed.abbreviation().to_string()
        };

        PlayoffSeries {
            higher_seed: higher_seed.abbreviation().to_string(),
            lower_seed: lower_seed.abbreviation().to_string(),
            wins_needed: 4,
            higher_seed_wins,
            lower_seed_wins,
            games,
            winner,
        }
    }
}

impl SimulationSettings {
    pub fn new(
        home_ice_advantage: f32,
        coach_weight: f32,
        play_style_weight: f32,
        standings_weight: f32,
        randomness_weight: f32,
    ) -> SimulationSettings {
        SimulationSettings {
            home_ice_advantage,
            coach_weight,
            play_style_weight,
            standings_weight,
            randomness_weight,
        }
    }

    pub fn default() -> SimulationSettings {
        SimulationSettings {
            home_ice_advantage: 0.18,
            coach_weight: 0.12,
            play_style_weight: 0.08,
            standings_weight: 0.06,
            randomness_weight: 0.22,
        }
    }

    pub fn home_ice_advantage(&self) -> f32 {
        self.home_ice_advantage
    }

    pub fn coach_weight(&self) -> f32 {
        self.coach_weight
    }

    pub fn play_style_weight(&self) -> f32 {
        self.play_style_weight
    }

    pub fn standings_weight(&self) -> f32 {
        self.standings_weight
    }

    pub fn randomness_weight(&self) -> f32 {
        self.randomness_weight
    }
}

impl League {
    pub fn new(name: String, standings: Vec<TeamStanding>) -> League {
        League {
            name,
            level: TeamLevel::MAJOR_PRO,
            rules: LeagueRules::nhl_style(),
            teams: Vec::new(),
            standings,free_agents:Vec::new()
        }
    }

    pub fn empty(name: String) -> League {
        League {
            name,
            level: TeamLevel::MAJOR_PRO,
            rules: LeagueRules::nhl_style(),
            teams: Vec::new(),
            standings: Vec::new(),
            free_agents:Vec::new()
        }
    }

    pub fn new_custom(
        name: String,
        level: TeamLevel,
        rules: LeagueRules,
        teams: Vec<Team>,
        standings: Vec<TeamStanding>,
    ) -> League {
        League {
            name,
            level,
            rules,
            teams,
            standings,
            free_agents:Vec::new()
        }

    }

    pub fn add_team(&mut self,team:Team){
        if self
            .standing_for_team(team.abbreviation())
            .is_none()
        {
            self.standings
                .push(TeamStanding::new(team.abbreviation().to_string()));
        }
        self.teams.push(team)
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn level(&self) -> &TeamLevel {
        &self.level
    }

    pub fn rules(&self) -> &LeagueRules {
        &self.rules
    }

    pub fn teams(&self) -> &[Team] {
        &self.teams
    }

    pub fn teams_mut(&mut self) -> &mut [Team] {
        &mut self.teams
    }

    pub fn standings(&self) -> &[TeamStanding] {
        &self.standings
    }

    pub fn standing_for_team(&self, abbreviation: &str) -> Option<&TeamStanding> {
        self.standings
            .iter()
            .find(|standing| standing.team_abbreviation() == abbreviation)
    }

    pub fn register_team(&mut self, team: &Team) {
        if self
            .teams
            .iter()
            .all(|entry| entry.abbreviation() != team.abbreviation())
        {
            self.add_team(clone_team(team));
        }
    }

    pub fn record_game(&mut self, game: &SimulatedGame) {
        sim_helper::update_standing(
            &mut self.standings,
            &self.rules,
            &game.home_team,
            game.home_goals,
            game.away_goals,
            game.overtime,
        );
        sim_helper::update_standing(
            &mut self.standings,
            &self.rules,
            &game.away_team,
            game.away_goals,
            game.home_goals,
            game.overtime,
        );
        self.standings.sort_by(|left, right| {
            right
                .points()
                .cmp(&left.points())
                .then(right.goal_differential().cmp(&left.goal_differential()))
                .then(right.wins().cmp(&left.wins()))
        });
    }

    pub fn save_to_context(&self, ctx: &SaveContext) -> Result<(), String> {
        let league_dir = self.league_dir(ctx);
        let teams_dir = format!("{}/teams", league_dir);
        fs::create_dir_all(&teams_dir).map_err(|error| error.to_string())?;

        let disk_data = LeagueDiskData {
            name: self.name.clone(),
            level: self.level,
            rules: clone_league_rules(&self.rules),
            standings: clone_standings(&self.standings),
            free_agents: clone_players(&self.free_agents),
        };

        let league_json = serde_json::to_string_pretty(&disk_data).map_err(|error| error.to_string())?;
        fs::write(self.league_file_path(ctx), league_json).map_err(|error| error.to_string())?;

        if Path::new(&teams_dir).exists() {
            for entry in fs::read_dir(&teams_dir).map_err(|error| error.to_string())? {
                let entry = entry.map_err(|error| error.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).map_err(|error| error.to_string())?;
                }
            }
        }

        for team in &self.teams {
            let team_path = format!("{}/{}.json", teams_dir, team.abbreviation());
            let team_json = serde_json::to_string_pretty(team).map_err(|error| error.to_string())?;
            fs::write(team_path, team_json).map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    pub fn load_from_context(ctx: &SaveContext, league_name: &str) -> Result<League, String> {
        let league_root = create_path_for_type(ctx.core(), &FileType::LEAGUE_DATA);
        let league_dir = format!("{}/{}", league_root, league_name);
        let league_file = format!("{}/{}.json", league_dir, league_name);
        let disk_data: LeagueDiskData = serde_json::from_str(
            &fs::read_to_string(&league_file).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

        let teams_dir = format!("{}/teams", league_dir);
        let mut teams = Vec::new();
        if Path::new(&teams_dir).exists() {
            let mut team_paths = fs::read_dir(&teams_dir)
                .map_err(|error| error.to_string())?
                .filter_map(|entry| entry.ok().map(|item| item.path()))
                .filter(|path| path.is_file())
                .collect::<Vec<_>>();
            team_paths.sort();

            for path in team_paths {
                let team: Team = serde_json::from_str(
                    &fs::read_to_string(path).map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
                teams.push(team);
            }
        }

        Ok(League {
            name: disk_data.name,
            level: disk_data.level,
            rules: disk_data.rules,
            teams,
            standings: disk_data.standings,
            free_agents: disk_data.free_agents,
        })
    }

    fn league_dir(&self, ctx: &SaveContext) -> String {
        format!(
            "{}/{}",
            create_path_for_type(ctx.core(), &FileType::LEAGUE_DATA),
            self.name
        )
    }

    fn league_file_path(&self, ctx: &SaveContext) -> String {
        format!("{}/{}.json", self.league_dir(ctx), self.name)
    }

    pub fn add_new_free_agent(&mut self,new_free_agent:Player){
        self.free_agents.push(new_free_agent);
    }

}

impl LeagueRules {
    pub fn new(
        points_for_win: i16,
        points_for_overtime_loss: i16,
        points_for_loss: i16,
        max_roster_size: i16,
        playoff_series_length: i16,
        allow_shootout: bool,
        parent_league: Option<String>,
        affiliated_minor_levels: Vec<TeamLevel>,
    ) -> LeagueRules {
        LeagueRules {
            points_for_win,
            points_for_overtime_loss,
            points_for_loss,
            max_roster_size,
            playoff_series_length,
            allow_shootout,
            parent_league,
            affiliated_minor_levels,
        }
    }

    pub fn nhl_style() -> LeagueRules {
        LeagueRules::new(
            2,
            1,
            0,
            23,
            7,
            true,
            None,
            vec![TeamLevel::MINOR_PRO],
        )
    }

    pub fn points_for_win(&self) -> i16 { self.points_for_win }
    pub fn points_for_overtime_loss(&self) -> i16 { self.points_for_overtime_loss }
    pub fn points_for_loss(&self) -> i16 { self.points_for_loss }
    pub fn max_roster_size(&self) -> i16 { self.max_roster_size }
    pub fn playoff_series_length(&self) -> i16 { self.playoff_series_length }
    pub fn allow_shootout(&self) -> bool { self.allow_shootout }
    pub fn parent_league(&self) -> Option<&str> { self.parent_league.as_deref() }
    pub fn affiliated_minor_levels(&self) -> &[TeamLevel] { &self.affiliated_minor_levels }
}

impl TeamStanding {
    pub fn new(team_abbreviation: String) -> TeamStanding {
        TeamStanding {
            team_abbreviation,
            games_played: 0,
            wins: 0,
            losses: 0,
            overtime_losses: 0,
            goals_for: 0,
            goals_against: 0,
            points: 0,
        }
    }

    pub fn from_record(
        team_abbreviation: String,
        wins: i16,
        losses: i16,
        overtime_losses: i16,
        goals_for: i16,
        goals_against: i16,
    ) -> TeamStanding {
        TeamStanding {
            team_abbreviation,
            games_played: wins + losses + overtime_losses,
            wins,
            losses,
            overtime_losses,
            goals_for,
            goals_against,
            points: wins * 2 + overtime_losses,
        }
    }

    pub fn new_full(
        team_abbreviation: String,
        games_played: i16,
        wins: i16,
        losses: i16,
        overtime_losses: i16,
        goals_for: i16,
        goals_against: i16,
        points: i16,
    ) -> TeamStanding {
        TeamStanding {
            team_abbreviation,
            games_played,
            wins,
            losses,
            overtime_losses,
            goals_for,
            goals_against,
            points,
        }
    }

    pub fn team_abbreviation(&self) -> &str {
        &self.team_abbreviation
    }

    pub fn games_played(&self) -> i16 {
        self.games_played
    }

    pub fn wins(&self) -> i16 {
        self.wins
    }

    pub fn losses(&self) -> i16 {
        self.losses
    }

    pub fn overtime_losses(&self) -> i16 {
        self.overtime_losses
    }

    pub fn goals_for(&self) -> i16 {
        self.goals_for
    }

    pub fn goals_against(&self) -> i16 {
        self.goals_against
    }

    pub fn points(&self) -> i16 {
        self.points
    }

    pub fn point_percentage(&self) -> f32 {
        if self.games_played == 0 {
            0.5
        } else {
            self.points as f32 / (self.games_played as f32 * 2.0)
        }
    }

    pub fn goal_differential(&self) -> i16 {
        self.goals_for - self.goals_against
    }
}

impl TeamProfile {
    pub fn new(
        overall: f32,
        offense: f32,
        defense: f32,
        goaltending: f32,
        coaching: f32,
        style_bias: f32,
        standings_factor: f32,
    ) -> TeamProfile {
        TeamProfile {
            overall,
            offense,
            defense,
            goaltending,
            coaching,
            style_bias,
            standings_factor,
        }
    }

    pub fn overall(&self) -> f32 {
        self.overall
    }

    pub fn offense(&self) -> f32 {
        self.offense
    }

    pub fn defense(&self) -> f32 {
        self.defense
    }

    pub fn goaltending(&self) -> f32 {
        self.goaltending
    }

    pub fn coaching(&self) -> f32 {
        self.coaching
    }

    pub fn style_bias(&self) -> f32 {
        self.style_bias
    }

    pub fn standings_factor(&self) -> f32 {
        self.standings_factor
    }
}

impl GameContext {
    pub fn new(season_game_number: i16) -> GameContext {
        GameContext { season_game_number }
    }

    pub fn regular_season() -> GameContext {
        GameContext {
            season_game_number: 1,
        }
    }

    pub fn season_game_number(&self) -> i16 {
        self.season_game_number
    }
}

impl SimulatedGame {
    pub fn home_team(&self) -> &str {
        &self.home_team
    }

    pub fn away_team(&self) -> &str {
        &self.away_team
    }

    pub fn home_goals(&self) -> i16 {
        self.home_goals
    }

    pub fn away_goals(&self) -> i16 {
        self.away_goals
    }

    pub fn overtime(&self) -> bool {
        self.overtime
    }

    pub fn shootout(&self) -> bool {
        self.shootout
    }

    pub fn home_profile(&self) -> &TeamProfile {
        &self.home_profile
    }

    pub fn away_profile(&self) -> &TeamProfile {
        &self.away_profile
    }
}

impl PlayoffSeries {
    pub fn higher_seed(&self) -> &str {
        &self.higher_seed
    }

    pub fn lower_seed(&self) -> &str {
        &self.lower_seed
    }

    pub fn wins_needed(&self) -> i16 {
        self.wins_needed
    }

    pub fn higher_seed_wins(&self) -> i16 {
        self.higher_seed_wins
    }

    pub fn lower_seed_wins(&self) -> i16 {
        self.lower_seed_wins
    }

    pub fn games(&self) -> &[SeriesGameLog] {
        &self.games
    }

    pub fn winner(&self) -> &str {
        &self.winner
    }

    pub fn summary_log(&self) -> String {
        let mut output = format!(
            "Playoff series: {} vs {}\nResult: {} {} - {} {}\n",
            self.higher_seed,
            self.lower_seed,
            self.higher_seed,
            self.higher_seed_wins,
            self.lower_seed_wins,
            self.lower_seed,
        );

        for game in &self.games {
            output.push_str(&game.log_line());
            output.push('\n');
            output.push_str("  ");
            output.push_str(game.home_profile_summary());
            output.push('\n');
            output.push_str("  ");
            output.push_str(game.away_profile_summary());
            output.push('\n');
            output.push_str("  ");
            output.push_str(game.momentum_note());
            output.push('\n');
        }

        output
    }
}

impl SeriesGameLog {
    pub fn game_number(&self) -> i16 {
        self.game_number
    }

    pub fn venue_team(&self) -> &str {
        &self.venue_team
    }

    pub fn matchup(&self) -> &str {
        &self.matchup
    }

    pub fn score_line(&self) -> &str {
        &self.score_line
    }

    pub fn decision(&self) -> &str {
        &self.decision
    }

    pub fn momentum_note(&self) -> &str {
        &self.momentum_note
    }

    pub fn home_profile_summary(&self) -> &str {
        &self.home_profile_summary
    }

    pub fn away_profile_summary(&self) -> &str {
        &self.away_profile_summary
    }

    pub fn game(&self) -> &SimulatedGame {
        &self.game
    }

    pub fn log_line(&self) -> String {
        format!(
            "Game {} at {} | {} | {} | {}",
            self.game_number,
            self.venue_team,
            self.matchup,
            self.score_line,
            self.decision,
        )
    }
}

pub fn add_free_agent(l: &mut League, p:Player){

    l.free_agents.push(p)


}

pub fn sort_agents_by_age(l: &mut League){

    l.free_agents.sort_by(|a, b| b.age.cmp(&a.age));

}

pub fn generate_free_agent(l:&mut League,quality:f32,names:&NameData){

    let p= player::random_prospect(quality, /* bool */rand::rng().random_ratio(1,4), names);

    add_free_agent(l,p);

}

fn clone_league_rules(rules: &LeagueRules) -> LeagueRules {
    LeagueRules::new(
        rules.points_for_win,
        rules.points_for_overtime_loss,
        rules.points_for_loss,
        rules.max_roster_size,
        rules.playoff_series_length,
        rules.allow_shootout,
        rules.parent_league.clone(),
        rules.affiliated_minor_levels.to_vec(),
    )
}

fn clone_standings(standings: &[TeamStanding]) -> Vec<TeamStanding> {
    standings
        .iter()
        .map(|standing| {
            TeamStanding::new_full(
                standing.team_abbreviation.clone(),
                standing.games_played,
                standing.wins,
                standing.losses,
                standing.overtime_losses,
                standing.goals_for,
                standing.goals_against,
                standing.points,
            )
        })
        .collect()
}

fn clone_players(players: &[Player]) -> Vec<Player> {
    players
        .iter()
        .map(|player| serde_json::from_str(&serde_json::to_string(player).unwrap()).unwrap())
        .collect()
}

fn clone_team(team: &Team) -> Team {
    serde_json::from_str(&serde_json::to_string(team).unwrap()).unwrap()
}
