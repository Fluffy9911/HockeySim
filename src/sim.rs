use crate::data::helper::PlayerRecord;
use crate::data::player::{PlayType, Position};
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

fn offense_score(roster: &[PlayerRecord]) -> f32 {
    if roster.is_empty() {
        return 0.25;
    }

    let total = roster
        .iter()
        .filter(|player| !matches!(player.player().position(), Position::GOALIE))
        .map(|player| {
            let skating = player.player().skate_stats();
            let base = (skating.speed() as f32 + skating.edges() as f32 + skating.acceleration() as f32) / 300.0;
            let projection = player.player().projection().development_profile().ceiling() as f32 / 100.0;
            let style = match player.player().play_type() {
                PlayType::SNIPER | PlayType::PLAYMAKER | PlayType::OFD => 0.10,
                PlayType::PWF => 0.07,
                _ => 0.03,
            };
            base * 0.65 + projection * 0.25 + style
        })
        .sum::<f32>();

    (total / roster.len() as f32).clamp(0.0, 1.5)
}

fn defense_score(roster: &[PlayerRecord]) -> f32 {
    if roster.is_empty() {
        return 0.25;
    }

    let total = roster
        .iter()
        .filter(|player| !matches!(player.player().position(), Position::GOALIE))
        .map(|player| {
            let skating = player.player().skate_stats();
            let base = (skating.edges() as f32 * 0.45 + skating.acceleration() as f32 * 0.25 + skating.speed() as f32 * 0.20)
                / 100.0;
            let play_bonus = match player.player().play_type() {
                PlayType::DFD | PlayType::DF => 0.18,
                PlayType::PWF => 0.10,
                _ => 0.03,
            };
            let position_bonus = match player.player().position() {
                Position::RD | Position::LD => 0.10,
                _ => 0.04,
            };
            base * 0.55 + play_bonus + position_bonus
        })
        .sum::<f32>();

    (total / roster.len() as f32).clamp(0.0, 1.5)
}

fn goaltending_score(roster: &[PlayerRecord]) -> f32 {
    let goalies: Vec<&PlayerRecord> = roster
        .iter()
        .filter(|player| matches!(player.player().position(), Position::GOALIE))
        .collect();

    if goalies.is_empty() {
        return 0.20;
    }

    let total = goalies
        .iter()
        .map(|goalie| {
            let skating = goalie.player().skate_stats();
            let movement = goalie.player().goalie_movement();
            let movement_score = match movement {
                Some(movement) => {
                    (movement.side() as f32 + movement.up_down() as f32 + movement.push() as f32) / 300.0
                }
                None => 0.30,
            };
            let style_bonus = match goalie.player().play_type() {
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

fn style_bias(roster: &[PlayerRecord], staff: &[StaffMember], settings: &SimulationSettings) -> f32 {
    if roster.is_empty() {
        return 0.0;
    }

    let player_bias = roster
        .iter()
        .map(|player| match player.player().play_type() {
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
    match league.and_then(|league| league.standing_for_team(team.identity().abbreviation())) {
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
