use rand::RngExt;
use crate::data::general_data::{PlayType, Position, Type};
use crate::data::player;
use crate::data::player::Player;
use crate::data::staff::{StaffMember, StaffRole};
use crate::data::team::{Team, TeamLevel};
use crate::league_settings::{League, LeagueRules, SimulatedGame, SimulationSettings, TeamProfile, TeamStanding};




pub(crate) fn build_team_profile(team: &Team, league: Option<&League>, settings: &SimulationSettings) -> TeamProfile {
    let offense = offense_score(team.roster());
    let defense = defense_score(team.roster());
    let goaltending = goaltending_score(team.roster());
    let coaching = coaching_score(team.staff());
    let style_bias = style_bias(team.roster(), team.staff(), settings);
    let standings_factor = standings_factor(team, league);
    let overall =
        offense * 0.36 + defense * 0.28 + goaltending * 0.22 + coaching * 0.14 + standings_factor * 0.08;

    TeamProfile::new(
        overall,
        offense,
        defense,
        goaltending,
        coaching,
        style_bias,
        standings_factor,
    )
}

fn offense_score(roster: &[Player]) -> f32 {
    if roster.is_empty() {
        return 0.25;
    }

    let total = roster
        .iter()
        .filter(|player| !matches!(player.position(), Position::GOALIE))
        .map(|player| {
            let skating = player.skate_stats();
            let base = (skating.speed() as f32 + skating.edges() as f32 + skating.acceleration() as f32) / 300.0;
            let projection = player.projection().development_profile().ceiling() as f32 / 100.0;
            let style = match player.play_type() {
                PlayType::SNIPER | PlayType::PLAYMAKER | PlayType::OFD => 0.10,
                PlayType::PWF => 0.07,
                _ => 0.03,
            };
            base * 0.65 + projection * 0.25 + style
        })
        .sum::<f32>();

    (total / roster.len() as f32).clamp(0.0, 1.5)
}

fn defense_score(roster: &[Player]) -> f32 {
    if roster.is_empty() {
        return 0.25;
    }

    let total = roster
        .iter()
        .filter(|player| !matches!(player.position(), Position::GOALIE))
        .map(|player| {
            let skating = player.skate_stats();
            let base = (skating.edges() as f32 * 0.45 + skating.acceleration() as f32 * 0.25 + skating.speed() as f32 * 0.20)
                / 100.0;
            let play_bonus = match player.play_type() {
                PlayType::DFD | PlayType::DF => 0.18,
                PlayType::PWF => 0.10,
                _ => 0.03,
            };
            let position_bonus = match player.position() {
                Position::RD | Position::LD => 0.10,
                _ => 0.04,
            };
            base * 0.55 + play_bonus + position_bonus
        })
        .sum::<f32>();

    (total / roster.len() as f32).clamp(0.0, 1.5)
}

fn goaltending_score(roster: &[Player]) -> f32 {
    let goalies: Vec<&Player> = roster
        .iter()
        .filter(|player| matches!(player.position(), Position::GOALIE))
        .collect();

    if goalies.is_empty() {
        return 0.20;
    }

    let total = goalies
        .iter()
        .map(|goalie| {
            let skating = goalie.skate_stats();
            let movement = goalie.goalie_movement();
            let movement_score = match movement {
                Some(movement) => {
                    (movement.side() as f32 + movement.up_down() as f32 + movement.push() as f32) / 300.0
                }
                None => 0.30,
            };
            let style_bonus = match goalie.play_type() {
                PlayType::BUTTERFLY | PlayType::HYBRID | PlayType::REACTIVE => 0.12,
                _ => 0.04,
            };
            let skating_score =
                (skating.speed() as f32 + skating.edges() as f32 + skating.acceleration() as f32) / 300.0;

            movement_score * 0.60 + skating_score * 0.20 + style_bonus + 0.15
        })
        .sum::<f32>();

    (total / goalies.len() as f32).clamp(0.0, 1.5)
}

fn coaching_score(staff: &[StaffMember]) -> f32 {
    if staff.is_empty() {
        return 0.35;
    }

    let mut weighted_total = 0.0;
    let mut total_weight = 0.0;

    for member in staff {
        let role_weight = match member.role() {
            StaffRole::HEAD_COACH => 1.0,
            StaffRole::ASSISTANT_COACH => 0.65,
            StaffRole::GOALIE_COACH | StaffRole::SKATING_COACH => 0.45,
            StaffRole::GENERAL_MANAGER | StaffRole::ASSISTANT_GENERAL_MANAGER => 0.25,
            _ => 0.15,
        };
        let ratings = member.ratings();
        let score = (ratings.teaching() as f32 * 0.35
            + ratings.tactical() as f32 * 0.35
            + ratings.leadership() as f32 * 0.20
            + ratings.evaluation() as f32 * 0.10)
            / 100.0;

        weighted_total += score * role_weight;
        total_weight += role_weight;
    }

    if total_weight == 0.0 {
        0.35
    } else {
        (weighted_total / total_weight).clamp(0.0, 1.5)
    }
}

fn style_bias(roster: &[Player], staff: &[StaffMember], settings: &SimulationSettings) -> f32 {
    if roster.is_empty() {
        return 0.0;
    }

    let player_bias = roster
        .iter()
        .map(|player| match player.play_type() {
            PlayType::SNIPER | PlayType::OFD | PlayType::PLAYMAKER| PlayType::TWD => 0.18,
            PlayType::PWF => 0.10,
            PlayType::DFD | PlayType::DF => -0.10,
            PlayType::BUTTERFLY => -0.03,
            PlayType::REACTIVE => -0.01,
            PlayType::HYBRID => 0.04,

        })
        .sum::<f32>()
        / roster.len() as f32;

    let coach_tactical = staff
        .iter()
        .find(|member| matches!(member.role(), StaffRole::HEAD_COACH))
        .map(|coach| coach.ratings().tactical() as f32 / 100.0 - 0.5)
        .unwrap_or(0.0);

    (player_bias + coach_tactical * settings.play_style_weight() * 2.0).clamp(-0.5, 0.5)
}

fn standings_factor(team: &Team, league: Option<&League>) -> f32 {
    match league.and_then(|league| league.standing_for_team(team.abbreviation())) {
        Some(standing) => {
            let points = standing.point_percentage();
            let differential = standing.goal_differential() as f32 / 82.0;
            (points - 0.5) * 0.7 + differential.clamp(-0.3, 0.3) * 0.3
        }
        None => 0.0,
    }
}

pub(crate) fn team_goal_expectancy(offense: f32, opposing_defense: f32, opposing_goaltending: f32) -> f32 {
    let raw = 2.75 + offense * 1.65 - opposing_defense * 0.85 - opposing_goaltending * 0.95;
    raw.clamp(0.8, 6.2)
}

pub(crate) fn finalize_goal_total(expectancy: f32, settings: &SimulationSettings, rng: &mut SimRng) -> i16 {
    let volatility_noise = centered_random(rng) * 3.2 * settings.randomness_weight();
    let finishing_noise = centered_random(rng) * 2.1 * settings.randomness_weight();
    let swing = if rng.next_f32() < settings.randomness_weight() * 0.45 {
        if rng.next_f32() < 0.5 { -1.0 } else { 1.0 }
    } else {
        0.0
    };
    let total = (expectancy + volatility_noise + finishing_noise + swing).round().clamp(0.0, 9.0);
    total as i16
}

pub(crate) fn resolve_tie(home_goals: i16, away_goals: i16, rng: &mut SimRng) -> (i16, i16, bool, bool) {
    if home_goals != away_goals {
        return (home_goals, away_goals, false, false);
    }

    if rng.next_f32() >= 0.45 {
        if rng.next_f32() >= 0.5 {
            (home_goals + 1, away_goals, true, false)
        } else {
            (home_goals, away_goals + 1, true, false)
        }
    } else if rng.next_f32() >= 0.5 {
        (home_goals + 1, away_goals, true, true)
    } else {
        (home_goals, away_goals + 1, true, true)
    }
}

pub(crate) fn format_decision(game: &SimulatedGame) -> String {
    if game.shootout() {
        "decided in shootout".to_string()
    } else if game.overtime() {
        "decided in overtime".to_string()
    } else {
        "decided in regulation".to_string()
    }
}

pub(crate) fn summarize_profile(team: &str, profile: &TeamProfile) -> String {
    format!(
        "{} profile => overall {:.3}, offense {:.3}, defense {:.3}, goaltending {:.3}, coaching {:.3}, style {:.3}, standings {:.3}",
        team,
        profile.overall(),
        profile.offense(),
        profile.defense(),
        profile.goaltending(),
        profile.coaching(),
        profile.style_bias(),
        profile.standings_factor(),
    )
}

pub(crate) fn weighted_gap(left: f32, right: f32, weight: f32) -> f32 {
    (left - right) * weight
}

pub(crate) fn centered_random(rng: &mut SimRng) -> f32 {
    ((rng.next_f32() + rng.next_f32() + rng.next_f32()) / 3.0 - 0.5) * 2.0
}

pub(crate) fn update_standing(
    standings: &mut Vec<TeamStanding>,
    rules: &LeagueRules,
    team_abbreviation: &str,
    goals_for: i16,
    goals_against: i16,
    overtime: bool,
) {
    if let Some(standing) = standings
        .iter_mut()
        .find(|standing| standing.team_abbreviation() == team_abbreviation)
    {
        standing.games_played += 1;
        standing.goals_for += goals_for;
        standing.goals_against += goals_against;

        if goals_for > goals_against {
            standing.wins += 1;
            standing.points += rules.points_for_win();
        } else if overtime {
            standing.overtime_losses += 1;
            standing.points += rules.points_for_overtime_loss();
        } else {
            standing.losses += 1;
            standing.points += rules.points_for_loss();
        }
    } else {
        let mut standing = TeamStanding::new(team_abbreviation.to_string());
        standing.games_played = 1;
        standing.goals_for = goals_for;
        standing.goals_against = goals_against;

        if goals_for > goals_against {
            standing.wins = 1;
            standing.points = rules.points_for_win();
        } else if overtime {
            standing.overtime_losses = 1;
            standing.points = rules.points_for_overtime_loss();
        } else {
            standing.losses = 1;
            standing.points = rules.points_for_loss();
        }

        standings.push(standing);
    }
}

pub(crate) struct SimRng {
    state: u64,
}

impl SimRng {
    pub(crate) fn new(seed: u64) -> SimRng {
        SimRng { state: seed | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}

trait TeamLevelClone {
   fn clone_level(&self) -> TeamLevel;


}

impl TeamLevelClone for TeamLevel {
    fn clone_level(&self) -> TeamLevel {
        match self {
            TeamLevel::MAJOR_PRO => TeamLevel::MAJOR_PRO,
            TeamLevel::MINOR_PRO => TeamLevel::MINOR_PRO,
            TeamLevel::JUNIOR => TeamLevel::JUNIOR,
            TeamLevel::COLLEGE => TeamLevel::COLLEGE,
            TeamLevel::INTERNATIONAL => TeamLevel::INTERNATIONAL,
            TeamLevel::OTHER => TeamLevel::OTHER,
        }
    }
}


pub fn get_goalies(team:&Team)-> (&Player,&Player){

    let mut gs:Vec<&Player> = Vec::new();

    for player in team.roster(){

        if player::is_goalie(player) {
            gs.push(
                player
            );
        }

    }

    if gs.len() < 2 {

         ()

    }

    (gs[0], gs[1])


}

pub fn get_defense(player:&Player)-> i8{

    let mut raw: i8 = player.skills().defense();

    let pos = match player.position()
    {
        Position::LD => 1.1,
        Position::RD => 1.1,
        _=> 1.0,

    };

    let player_type_bonus = match player.play_type() {
        PlayType::DFD => 1.15,
        PlayType::TWD=> 1.1,
        PlayType::DF=> 1.2,
        PlayType::SNIPER => 0.9,
        PlayType::PWF=> 1.01,
        _=> 1.0
    };

    let type_bonus = match player.player_type(){
        Type::GOALIE => 1.75,
        _=> 1.0
    };

    let mut bonus_avg: f64 = (pos + (player_type_bonus*player_type_bonus)+type_bonus) / 3.0;
    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg).min(100.0) as i8;

    raw
    
    
    
}

pub fn get_offense(player: &Player) -> i8 {
    let mut raw: i8 = player.skills().offense();

    let pos = match player.position() {
        Position::LW => 1.1,
        Position::RW => 1.1,
        Position::CENTER  => 1.15,
        _ => 1.0,
    };

    let player_type_bonus = match player.play_type() {
        PlayType::SNIPER => 1.2,
        PlayType::PWF    => 1.1,
        PlayType::TWD    => 1.05,
        PlayType::DFD    => 0.9,
        PlayType::DF     => 0.85,
        _ => 1.0,
    };

    let type_bonus = match player.player_type() {
        Type::GOALIE => 0.5,
        _ => 1.0,
    };

    let mut bonus_avg: f64 =
        (pos + (player_type_bonus * player_type_bonus) + type_bonus) / 3.0;

    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg).min(100.0) as i8;

    raw
}

pub fn get_line_bonus(player:&Player) -> f32{

    let mut raw: f32 =( player.skills().passing()+player.view().predicting()+player.view().smart())as f32 / 3.0;

    let pos = match player.position(){
        Position::CENTER => 1.1,
        _=>1.0
    };

    let type_bonus = match player.play_type(){

        PlayType::PLAYMAKER=> 1.25,
        PlayType::SNIPER=> 0.90,
        PlayType::OFD=> 1.01,
        PlayType::PWF=> 1.10,
        _=> 1.0

    };

    let mut bonus_avg: f64 =
        (pos + (type_bonus * type_bonus) + type_bonus) / 3.0;

    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg).min(100.0) as f32;

    raw


}
pub fn get_awareness_bonus(player: &Player) -> f32 {
    let mut raw: f32 = (
        player.view().scan() +
            player.view().predicting() +
            player.view().smart()
    ) as f32 / 3.0;

    let pos = match player.position() {
        Position::CENTER => 1.1,
        Position::LD | Position::RD => 1.05,
        _ => 1.0,
    };

    let type_bonus = match player.play_type() {
        PlayType::TWD => 1.15,
        PlayType::PLAYMAKER => 1.1,
        PlayType::DFD => 1.1,
        _ => 1.0,
    };

    let mut bonus_avg: f32 =
        (pos + (type_bonus * type_bonus) + type_bonus) / 3.0;

    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg as f64).min(100.0) as f32;

    raw
}
pub fn get_shooting_bonus(player: &Player) -> f32 {
    let mut raw: f32 = (
        player.skills().shot_accuracy() +
            player.skills().shot_power()
    ) as f32 / 2.0;

    let pos = match player.position() {
        Position::LW | Position::RW => 1.1,
        _ => 1.0,
    };

    let type_bonus = match player.play_type() {
        PlayType::SNIPER => 1.25,
        PlayType::PWF => 1.1,
        PlayType::PLAYMAKER => 1.05,
        _ => 1.0,
    };

    let mut bonus_avg: f32 =
        (pos + (type_bonus * type_bonus) + type_bonus) / 3.0;

    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg as f64).min(100.0) as f32;

    raw
}

pub fn get_play_driving_bonus(player: &Player) -> f32 {
    let mut raw: f32 = (
        player.skills().passing() +
            player.skills().offense() +
            player.view().smart()
    ) as f32 / 3.0;

    let pos = match player.position() {
        Position::CENTER => 1.15,
        _ => 1.0,
    };

    let type_bonus = match player.play_type() {
        PlayType::PLAYMAKER => 1.25,
        PlayType::TWD => 1.05,
        PlayType::SNIPER => 0.95,
        _ => 1.0,
    };

    let mut bonus_avg: f32 =
        (pos + (type_bonus * type_bonus) + type_bonus) / 3.0;

    bonus_avg = bonus_avg.min(2.0);

    raw = (raw as f64 * bonus_avg as f64).min(100.0) as f32;

    raw
}

fn avg(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

pub fn forward_line_offense_bonus(line: &[Player]) -> f32 {
    let base: Vec<f32> = line.iter()
        .map(|p| get_offense(p) as f32)
        .collect();

    let mut raw = avg(&base);

    // synergy: reward diversity (playmaker + sniper + power)
    let mut has_playmaker = false;
    let mut has_sniper = false;
    let mut has_power = false;

    for p in line {
        match p.play_type() {
            PlayType::PLAYMAKER => has_playmaker = true,
            PlayType::SNIPER => has_sniper = true,
            PlayType::PWF => has_power = true,
            _ => {}
        }
    }

    let synergy_bonus = match (has_playmaker, has_sniper, has_power) {
        (true, true, true) => 1.15,
        (true, true, _) => 1.1,
        _ => 1.0,
    };

    raw *= synergy_bonus;

    raw.min(100.0)
}

pub fn forward_line_defense_bonus(line: &[Player]) -> f32 {
    let base: Vec<f32> = line.iter()
        .map(|p| get_defense(p) as f32)
        .collect();

    let mut raw = avg(&base);

    let mut defensive_players = 0;

    for p in line {
        match p.play_type() {
            PlayType::TWD | PlayType::DF => defensive_players += 1,
            _ => {}
        }
    }

    let synergy_bonus = match defensive_players {
        3 => 1.15,
        2 => 1.1,
        _ => 1.0,
    };

    raw *= synergy_bonus;

    raw.min(100.0)
}

pub fn defense_pair_offense_bonus(pair: &[Player]) -> f32 {
    let base: Vec<f32> = pair.iter()
        .map(|p| get_offense(p) as f32)
        .collect();

    let mut raw = avg(&base);

    let mut has_ofd = false;

    for p in pair {
        if let PlayType::OFD = p.play_type() {
            has_ofd = true;
        }
    }

    let synergy_bonus = if has_ofd { 1.1 } else { 1.0 };

    raw *= synergy_bonus;

    raw.min(100.0)
}
fn biased_vs(a: f32, b: f32) -> f32 {
    let mut r = rand::rng();

    // normalize difference into 0..1
    let total = a + b;
    let bias = if total == 0.0 { 0.5 } else { a / total };

    // random roll (0..1), skew toward stronger side
    let roll: f32 = r.random();
    roll * (1.0 - bias) + bias
}
pub fn defense_pair_defense_bonus(pair: &[Player]) -> f32 {
    let base: Vec<f32> = pair.iter()
        .map(|p| get_defense(p) as f32)
        .collect();

    let mut raw = avg(&base);

    let mut has_dfd = false;
    let mut has_ofd = false;

    for p in pair {
        match p.play_type() {
            PlayType::DFD => has_dfd = true,
            PlayType::OFD => has_ofd = true,
            _ => {}
        }
    }

    // classic "stay-at-home + offensive D" pairing bonus
    let synergy_bonus = match (has_dfd, has_ofd) {
        (true, true) => 1.15,
        (true, false) => 1.05,
        _ => 1.0,
    };

    raw *= synergy_bonus;

    raw.min(100.0)
}

pub fn simulate_matchup(
    fwd_line_a: &[Player],
    def_pair_a: &[Player],
    fwd_line_b: &[Player],
    def_pair_b: &[Player],
) {
    // --- TEAM A metrics ---
    let a_off = forward_line_offense_bonus(fwd_line_a);
    let a_def = forward_line_defense_bonus(fwd_line_a)
        + defense_pair_defense_bonus(def_pair_a);

    // --- TEAM B metrics ---
    let b_off = forward_line_offense_bonus(fwd_line_b);
    let b_def = forward_line_defense_bonus(fwd_line_b)
        + defense_pair_defense_bonus(def_pair_b);

    println!("--- MATCHUP ---");
    println!("Team A Offense: {:.2}", a_off);
    println!("Team A Defense: {:.2}", a_def);
    println!("Team B Offense: {:.2}", b_off);
    println!("Team B Defense: {:.2}", b_def);

    // --- A attacking B ---
    let a_attack_strength = a_off;
    let b_defense_strength = b_def;

    let a_roll = biased_vs(a_attack_strength, b_defense_strength);

    // --- B attacking A ---
    let b_attack_strength = b_off;
    let a_defense_strength = a_def;

    let b_roll = biased_vs(b_attack_strength, a_defense_strength);

    println!("\n--- RESULTS ---");

    // interpret rolls
    if a_roll > 0.55 {
        println!("Team A is likely to SCORE ⚡");
    } else if a_roll > 0.48 {
        println!("Team A generates a strong chance");
    } else {
        println!("Team A is shut down");
    }

    if b_roll > 0.55 {
        println!("Team B is likely to SCORE ⚡");
    } else if b_roll > 0.48 {
        println!("Team B generates a strong chance");
    } else {
        println!("Team B is shut down");
    }

    // head-to-head comparison
    println!("\n--- EDGE ---");

    if a_roll > b_roll {
        println!("➡ Team A has the edge");
    } else if b_roll > a_roll {
        println!("➡ Team B has the edge");
    } else {
        println!("➡ Even matchup");
    }
}






