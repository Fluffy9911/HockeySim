use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerStats {
    games_played: i16,
    goals: i16,
    assists: i16,
    plus_minus: i16,
    penalty_minutes: i16,
    shots: i16,
    power_play_goals: i16,
    power_play_assists: i16,
    short_handed_goals: i16,
    short_handed_assists: i16,
    game_winning_goals: i16,
    overtime_goals: i16,
    faceoff_wins: i16,
    faceoff_losses: i16,
    hits: i16,
    blocked_shots: i16,
    takeaways: i16,
    giveaways: i16,
    time_on_ice_minutes: i32,
    goalie_stats: Option<GoalieStats>,
}

#[derive(Serialize, Deserialize)]
pub struct GoalieStats {
    starts: i16,
    wins: i16,
    losses: i16,
    overtime_losses: i16,
    shots_against: i16,
    saves: i16,
    goals_against: i16,
    shutouts: i16,
    power_play_goals_against: i16,
    short_handed_goals_against: i16,
    time_on_ice_minutes: i32,
}

#[derive(Serialize, Deserialize)]
pub struct TeamStats {
    games_played: i16,
    wins: i16,
    losses: i16,
    overtime_losses: i16,
    points: i16,
    goals_for: i16,
    goals_against: i16,
    shots_for: i16,
    shots_against: i16,
    power_play_goals: i16,
    power_play_opportunities: i16,
    penalty_kill_goals_against: i16,
    penalty_kill_opportunities: i16,
    faceoff_wins: i16,
    faceoff_losses: i16,
    hits: i16,
    blocked_shots: i16,
}

impl PlayerStats {
    pub fn new(
        games_played: i16,
        goals: i16,
        assists: i16,
        plus_minus: i16,
        penalty_minutes: i16,
        shots: i16,
        power_play_goals: i16,
        power_play_assists: i16,
        short_handed_goals: i16,
        short_handed_assists: i16,
        game_winning_goals: i16,
        overtime_goals: i16,
        faceoff_wins: i16,
        faceoff_losses: i16,
        hits: i16,
        blocked_shots: i16,
        takeaways: i16,
        giveaways: i16,
        time_on_ice_minutes: i32,
        goalie_stats: Option<GoalieStats>,
    ) -> PlayerStats {
        PlayerStats {
            games_played,
            goals,
            assists,
            plus_minus,
            penalty_minutes,
            shots,
            power_play_goals,
            power_play_assists,
            short_handed_goals,
            short_handed_assists,
            game_winning_goals,
            overtime_goals,
            faceoff_wins,
            faceoff_losses,
            hits,
            blocked_shots,
            takeaways,
            giveaways,
            time_on_ice_minutes,
            goalie_stats,
        }
    }

    pub fn skater_default() -> PlayerStats {
        PlayerStats::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, None)
    }

    pub fn goalie_default() -> PlayerStats {
        PlayerStats::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, Some(GoalieStats::default()))
    }

    pub fn is_default(&self) -> bool {
        self.games_played == 0
            && self.goals == 0
            && self.assists == 0
            && self.plus_minus == 0
            && self.penalty_minutes == 0
            && self.shots == 0
            && self.power_play_goals == 0
            && self.power_play_assists == 0
            && self.short_handed_goals == 0
            && self.short_handed_assists == 0
            && self.game_winning_goals == 0
            && self.overtime_goals == 0
            && self.faceoff_wins == 0
            && self.faceoff_losses == 0
            && self.hits == 0
            && self.blocked_shots == 0
            && self.takeaways == 0
            && self.giveaways == 0
            && self.time_on_ice_minutes == 0
            && self.goalie_stats.as_ref().map(|s| s.is_default()).unwrap_or(true)
    }

    pub fn points(&self) -> i16 { self.goals + self.assists }
    pub fn games_played(&self) -> i16 { self.games_played }
    pub fn goals(&self) -> i16 { self.goals }
    pub fn assists(&self) -> i16 { self.assists }
    pub fn plus_minus(&self) -> i16 { self.plus_minus }
    pub fn penalty_minutes(&self) -> i16 { self.penalty_minutes }
    pub fn shots(&self) -> i16 { self.shots }
    pub fn power_play_goals(&self) -> i16 { self.power_play_goals }
    pub fn power_play_assists(&self) -> i16 { self.power_play_assists }
    pub fn short_handed_goals(&self) -> i16 { self.short_handed_goals }
    pub fn short_handed_assists(&self) -> i16 { self.short_handed_assists }
    pub fn game_winning_goals(&self) -> i16 { self.game_winning_goals }
    pub fn overtime_goals(&self) -> i16 { self.overtime_goals }
    pub fn faceoff_wins(&self) -> i16 { self.faceoff_wins }
    pub fn faceoff_losses(&self) -> i16 { self.faceoff_losses }
    pub fn hits(&self) -> i16 { self.hits }
    pub fn blocked_shots(&self) -> i16 { self.blocked_shots }
    pub fn takeaways(&self) -> i16 { self.takeaways }
    pub fn giveaways(&self) -> i16 { self.giveaways }
    pub fn time_on_ice_minutes(&self) -> i32 { self.time_on_ice_minutes }
    pub fn goalie_stats(&self) -> Option<&GoalieStats> { self.goalie_stats.as_ref() }
    pub fn goalie_stats_mut(&mut self) -> Option<&mut GoalieStats> { self.goalie_stats.as_mut() }

    pub fn record_skater_game(
        &mut self,
        goals: i16,
        assists: i16,
        plus_minus: i16,
        penalty_minutes: i16,
        shots: i16,
        power_play_goals: i16,
        power_play_assists: i16,
        hits: i16,
        blocked_shots: i16,
        time_on_ice_minutes: i32,
    ) {
        self.games_played += 1;
        self.goals += goals;
        self.assists += assists;
        self.plus_minus += plus_minus;
        self.penalty_minutes += penalty_minutes;
        self.shots += shots;
        self.power_play_goals += power_play_goals;
        self.power_play_assists += power_play_assists;
        self.hits += hits;
        self.blocked_shots += blocked_shots;
        self.time_on_ice_minutes += time_on_ice_minutes;
    }

    pub fn record_goalie_game(
        &mut self,
        win: bool,
        overtime_loss: bool,
        shots_against: i16,
        saves: i16,
        goals_against: i16,
        shutout: bool,
        time_on_ice_minutes: i32,
    ) {
        self.games_played += 1;
        self.time_on_ice_minutes += time_on_ice_minutes;
        if let Some(goalie_stats) = self.goalie_stats.as_mut() {
            goalie_stats.starts += 1;
            goalie_stats.shots_against += shots_against;
            goalie_stats.saves += saves;
            goalie_stats.goals_against += goals_against;
            goalie_stats.time_on_ice_minutes += time_on_ice_minutes;
            if shutout {
                goalie_stats.shutouts += 1;
            }
            if win {
                goalie_stats.wins += 1;
            } else if overtime_loss {
                goalie_stats.overtime_losses += 1;
            } else {
                goalie_stats.losses += 1;
            }
        }
    }
}

impl GoalieStats {
    pub fn new(
        starts: i16,
        wins: i16,
        losses: i16,
        overtime_losses: i16,
        shots_against: i16,
        saves: i16,
        goals_against: i16,
        shutouts: i16,
        power_play_goals_against: i16,
        short_handed_goals_against: i16,
        time_on_ice_minutes: i32,
    ) -> GoalieStats {
        GoalieStats {
            starts,
            wins,
            losses,
            overtime_losses,
            shots_against,
            saves,
            goals_against,
            shutouts,
            power_play_goals_against,
            short_handed_goals_against,
            time_on_ice_minutes,
        }
    }

    pub fn default() -> GoalieStats {
        GoalieStats::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    }

    pub fn is_default(&self) -> bool {
        self.starts == 0
            && self.wins == 0
            && self.losses == 0
            && self.overtime_losses == 0
            && self.shots_against == 0
            && self.saves == 0
            && self.goals_against == 0
            && self.shutouts == 0
            && self.power_play_goals_against == 0
            && self.short_handed_goals_against == 0
            && self.time_on_ice_minutes == 0
    }

    pub fn save_percentage(&self) -> f32 {
        if self.shots_against == 0 { 0.0 } else { self.saves as f32 / self.shots_against as f32 }
    }

    pub fn goals_against_average(&self) -> f32 {
        if self.time_on_ice_minutes == 0 {
            0.0
        } else {
            self.goals_against as f32 * 60.0 / self.time_on_ice_minutes as f32
        }
    }

    pub fn starts(&self) -> i16 { self.starts }
    pub fn wins(&self) -> i16 { self.wins }
    pub fn losses(&self) -> i16 { self.losses }
    pub fn overtime_losses(&self) -> i16 { self.overtime_losses }
    pub fn shots_against(&self) -> i16 { self.shots_against }
    pub fn saves(&self) -> i16 { self.saves }
    pub fn goals_against(&self) -> i16 { self.goals_against }
    pub fn shutouts(&self) -> i16 { self.shutouts }
    pub fn power_play_goals_against(&self) -> i16 { self.power_play_goals_against }
    pub fn short_handed_goals_against(&self) -> i16 { self.short_handed_goals_against }
    pub fn time_on_ice_minutes(&self) -> i32 { self.time_on_ice_minutes }
}

impl TeamStats {
    pub fn new(
        games_played: i16,
        wins: i16,
        losses: i16,
        overtime_losses: i16,
        goals_for: i16,
        goals_against: i16,
        shots_for: i16,
        shots_against: i16,
        power_play_goals: i16,
        power_play_opportunities: i16,
        penalty_kill_goals_against: i16,
        penalty_kill_opportunities: i16,
        faceoff_wins: i16,
        faceoff_losses: i16,
        hits: i16,
        blocked_shots: i16,
    ) -> TeamStats {
        TeamStats {
            games_played,
            wins,
            losses,
            overtime_losses,
            points: wins * 2 + overtime_losses,
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
        }
    }

    pub fn default() -> TeamStats {
        TeamStats::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    }

    pub fn games_played(&self) -> i16 { self.games_played }
    pub fn wins(&self) -> i16 { self.wins }
    pub fn losses(&self) -> i16 { self.losses }
    pub fn overtime_losses(&self) -> i16 { self.overtime_losses }
    pub fn points(&self) -> i16 { self.points }
    pub fn goals_for(&self) -> i16 { self.goals_for }
    pub fn goals_against(&self) -> i16 { self.goals_against }
    pub fn shots_for(&self) -> i16 { self.shots_for }
    pub fn shots_against(&self) -> i16 { self.shots_against }
    pub fn power_play_goals(&self) -> i16 { self.power_play_goals }
    pub fn power_play_opportunities(&self) -> i16 { self.power_play_opportunities }
    pub fn penalty_kill_goals_against(&self) -> i16 { self.penalty_kill_goals_against }
    pub fn penalty_kill_opportunities(&self) -> i16 { self.penalty_kill_opportunities }
    pub fn faceoff_wins(&self) -> i16 { self.faceoff_wins }
    pub fn faceoff_losses(&self) -> i16 { self.faceoff_losses }
    pub fn hits(&self) -> i16 { self.hits }
    pub fn blocked_shots(&self) -> i16 { self.blocked_shots }

    pub fn goal_differential(&self) -> i16 { self.goals_for - self.goals_against }

    pub fn power_play_percentage(&self) -> f32 {
        if self.power_play_opportunities == 0 { 0.0 } else { self.power_play_goals as f32 / self.power_play_opportunities as f32 }
    }

    pub fn penalty_kill_percentage(&self) -> f32 {
        if self.penalty_kill_opportunities == 0 {
            0.0
        } else {
            1.0 - (self.penalty_kill_goals_against as f32 / self.penalty_kill_opportunities as f32)
        }
    }

    pub fn record_game(
        &mut self,
        win: bool,
        overtime_loss: bool,
        goals_for: i16,
        goals_against: i16,
        shots_for: i16,
        shots_against: i16,
        power_play_goals: i16,
        power_play_opportunities: i16,
        penalty_kill_goals_against: i16,
        penalty_kill_opportunities: i16,
        faceoff_wins: i16,
        faceoff_losses: i16,
        hits: i16,
        blocked_shots: i16,
    ) {
        self.games_played += 1;
        self.goals_for += goals_for;
        self.goals_against += goals_against;
        self.shots_for += shots_for;
        self.shots_against += shots_against;
        self.power_play_goals += power_play_goals;
        self.power_play_opportunities += power_play_opportunities;
        self.penalty_kill_goals_against += penalty_kill_goals_against;
        self.penalty_kill_opportunities += penalty_kill_opportunities;
        self.faceoff_wins += faceoff_wins;
        self.faceoff_losses += faceoff_losses;
        self.hits += hits;
        self.blocked_shots += blocked_shots;

        if win {
            self.wins += 1;
            self.points += 2;
        } else if overtime_loss {
            self.overtime_losses += 1;
            self.points += 1;
        } else {
            self.losses += 1;
        }
    }
}
