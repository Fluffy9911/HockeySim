use crate::data::contract::{Contract, ContractType};
use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use crate::data::player::{PlayType as PlayerPlayType, PlayType, Player, Position, Type};
use crate::data::projection::{
    DevelopmentCurve, DevelopmentProfile, DraftProjection, ProjMax, Projection,
};
use crate::data::stats::{GoalieStats, PlayerStats};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DraftStatus {
    Undrafted,
    Drafted(DraftData),
}

#[derive(Serialize, Deserialize)]
pub struct DraftData {
    draft_year: i16,
    draft_round: i8,
    overall_pick: i16,
    team: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerRecord {
    name: String,
    age: i8,
    player: Player,
    projection: Projection,
    draft_status: DraftStatus,
    stats: PlayerStats,
    contract: Option<Contract>,
}

impl DraftData {
    pub fn new(draft_year: i16, draft_round: i8, overall_pick: i16, team: String) -> DraftData {
        DraftData {
            draft_year,
            draft_round,
            overall_pick,
            team,
        }
    }

    pub fn draft_year(&self) -> i16 {
        self.draft_year
    }

    pub fn draft_round(&self) -> i8 {
        self.draft_round
    }

    pub fn overall_pick(&self) -> i16 {
        self.overall_pick
    }

    pub fn team(&self) -> &str {
        &self.team
    }
}

impl DraftStatus {
    pub fn undrafted() -> DraftStatus {
        DraftStatus::Undrafted
    }

    pub fn drafted(draft_data: DraftData) -> DraftStatus {
        DraftStatus::Drafted(draft_data)
    }
}

impl PlayerRecord {
    pub fn new(
        name: String,
        age: i8,
        player: Player,
        projection: Projection,
        draft_status: DraftStatus,
    ) -> PlayerRecord {
        let stats = default_stats_for_player(&player);
        PlayerRecord {
            name,
            age,
            player,
            projection,
            draft_status,
            stats,
            contract: None,
        }
    }

    pub fn new_with_stats(
        name: String,
        age: i8,
        player: Player,
        projection: Projection,
        draft_status: DraftStatus,
        stats: PlayerStats,
    ) -> PlayerRecord {
        PlayerRecord {
            name,
            age,
            player,
            projection,
            draft_status,
            stats,
            contract: None,
        }
    }

    pub fn new_with_contract(
        name: String,
        age: i8,
        player: Player,
        projection: Projection,
        draft_status: DraftStatus,
        stats: PlayerStats,
        contract: Option<Contract>,
    ) -> PlayerRecord {
        PlayerRecord {
            name,
            age,
            player,
            projection,
            draft_status,
            stats,
            contract,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn age(&self) -> i8 {
        self.age
    }

    pub fn age_one_year(&mut self) {
        self.age += 1;
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn draft_status(&self) -> &DraftStatus {
        &self.draft_status
    }

    pub fn stats(&self) -> &PlayerStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut PlayerStats {
        &mut self.stats
    }

    pub fn contract(&self) -> Option<&Contract> {
        self.contract.as_ref()
    }

    pub fn set_contract(&mut self, contract: Option<Contract>) {
        self.contract = contract;
    }
    pub fn injury_penalty(injury_risk: i8) -> i8 {
        if injury_risk >= 80 {
            2
        } else if injury_risk >= 60 {
            1
        } else {
            0
        }
    }
    pub fn develop(&mut self, coaching_bonus: i8) {
        let profile = self.projection.development_profile();
        let in_window = self.age >= profile.growth_window_start() && self.age <= profile.growth_window_end();
        let age_factor = if in_window { 2 } else if self.age < profile.growth_window_start() { 1 } else { -1 };
        let curve_bonus = match profile.curve() {
            DevelopmentCurve::EARLY => {
                if self.age <= 22 { 1 } else { 0 }
            }
            DevelopmentCurve::LINEAR => 0,
            DevelopmentCurve::LATE => {
                if self.age >= 24 { 1 } else { 0 }
            }
            DevelopmentCurve::BOOM_BUST => {
                if profile.consistency() >= 60 { 1 } else { -1 }
            }
        };
        let growth_pressure = ((profile.growth_rate() as i16
            + profile.coachability() as i16
            + profile.work_ethic() as i16
            + coaching_bonus as i16)
            / 45) as i8;
        let total_delta = age_factor + curve_bonus + growth_pressure - Self::injury_penalty(profile.injury_risk());
        let max_rating = profile.ceiling().max(profile.floor());

        self.player
            .skate_stats_mut()
            .apply_delta(total_delta, total_delta, total_delta, max_rating);

        if let Some(movement) = self.player.goalie_movement_mut() {
            movement.apply_delta(total_delta, total_delta, total_delta, max_rating);
        }
    }




    fn require_field<T>(value: Option<T>, field: &str) -> Result<T, String> {
        value.ok_or_else(|| format!("Missing required field: {field}"))
    }

    fn parse_i8(value: &str, field: &str) -> Result<i8, String> {
        value
            .parse::<i8>()
            .map_err(|_| format!("Invalid i8 for {field}: {value}"))
    }

    fn parse_i16(value: &str, field: &str) -> Result<i16, String> {
        value
            .parse::<i16>()
            .map_err(|_| format!("Invalid i16 for {field}: {value}"))
    }

    fn parse_i32(value: &str, field: &str) -> Result<i32, String> {
        value
            .parse::<i32>()
            .map_err(|_| format!("Invalid i32 for {field}: {value}"))
    }

    fn parse_f32(value: &str, field: &str) -> Result<f32, String> {
        value
            .parse::<f32>()
            .map_err(|_| format!("Invalid f32 for {field}: {value}"))
    }

    fn escape_text(value: &str) -> String {
        value.replace('\\', "\\\\").replace('\n', "\\n")
    }

    fn unescape_text(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('\\') => result.push('\\'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(ch);
            }
        }

        result
    }



    fn contract_type_to_str(value: &ContractType) -> &'static str {
        match value {
            ContractType::ENTRY_LEVEL => "ENTRY_LEVEL",
            ContractType::STANDARD => "STANDARD",
            ContractType::BRIDGE => "BRIDGE",
            ContractType::EXTENSION => "EXTENSION",
            ContractType::TWO_WAY => "TWO_WAY",
            ContractType::ONE_WAY => "ONE_WAY",
            ContractType::PROFESSIONAL_TRYOUT => "PROFESSIONAL_TRYOUT",
        }
    }

    fn parse_contract_type(value: &str) -> Result<ContractType, String> {
        match value {
            "ENTRY_LEVEL" => Ok(ContractType::ENTRY_LEVEL),
            "STANDARD" => Ok(ContractType::STANDARD),
            "BRIDGE" => Ok(ContractType::BRIDGE),
            "EXTENSION" => Ok(ContractType::EXTENSION),
            "TWO_WAY" => Ok(ContractType::TWO_WAY),
            "ONE_WAY" => Ok(ContractType::ONE_WAY),
            "PROFESSIONAL_TRYOUT" => Ok(ContractType::PROFESSIONAL_TRYOUT),
            _ => Err(format!("Invalid contract_type: {value}")),
        }
    }



    fn player_type_to_str(value: &Type) -> &'static str {
        match value {
            Type::SKATER => "SKATER",
            Type::GOALIE => "GOALIE",
        }
    }

    fn parse_player_type(value: &str) -> Result<Type, String> {
        match value {
            "SKATER" => Ok(Type::SKATER),
            "GOALIE" => Ok(Type::GOALIE),
            _ => Err(format!("Invalid player_type: {value}")),
        }
    }

    fn position_to_str(value: &Position) -> &'static str {
        match value {
            Position::CENTER => "CENTER",
            Position::LW => "LW",
            Position::RW => "RW",
            Position::RD => "RD",
            Position::LD => "LD",
            Position::GOALIE => "GOALIE",
        }
    }

    fn parse_position(value: &str) -> Result<Position, String> {
        match value {
            "CENTER" => Ok(Position::CENTER),
            "LW" => Ok(Position::LW),
            "RW" => Ok(Position::RW),
            "RD" => Ok(Position::RD),
            "LD" => Ok(Position::LD),
            "GOALIE" => Ok(Position::GOALIE),
            _ => Err(format!("Invalid position: {value}")),
        }
    }

    fn player_play_type_to_str(value: &PlayerPlayType) -> &'static str {
        match value {
            PlayerPlayType::SNIPER => "SNIPER",
            PlayerPlayType::OFD => "OFD",
            PlayerPlayType::DFD => "DFD",
            PlayerPlayType::PWF => "PWF",
            PlayerPlayType::DF => "DF",
            PlayerPlayType::PLAYMAKER => "PLAYMAKER",
            PlayerPlayType::BUTTERFLY => "BUTTERFLY",
            PlayerPlayType::REACTIVE => "REACTIVE",
            PlayerPlayType::HYBRID => "HYBRID",
        }
    }

    fn parse_player_play_type(value: &str) -> Result<PlayerPlayType, String> {
        match value {
            "SNIPER" => Ok(PlayerPlayType::SNIPER),
            "OFD" => Ok(PlayerPlayType::OFD),
            "DFD" => Ok(PlayerPlayType::DFD),
            "PWF" => Ok(PlayerPlayType::PWF),
            "DF" => Ok(PlayerPlayType::DF),
            "PLAYMAKER" => Ok(PlayerPlayType::PLAYMAKER),
            "BUTTERFLY" => Ok(PlayerPlayType::BUTTERFLY),
            "REACTIVE" => Ok(PlayerPlayType::REACTIVE),
            "HYBRID" => Ok(PlayerPlayType::HYBRID),
            _ => Err(format!("Invalid play_type: {value}")),
        }
    }

    fn skating_type_to_str(value: &SkatingType) -> &'static str {
        match value {
            SkatingType::QUICK => "QUICK",
            SkatingType::SLOW => "SLOW",
            SkatingType::STRONG => "STRONG",
            SkatingType::NIMBLE => "NIMBLE",
        }
    }

    fn parse_skating_type(value: &str) -> Result<SkatingType, String> {
        match value {
            "QUICK" => Ok(SkatingType::QUICK),
            "SLOW" => Ok(SkatingType::SLOW),
            "STRONG" => Ok(SkatingType::STRONG),
            "NIMBLE" => Ok(SkatingType::NIMBLE),
            _ => Err(format!("Invalid skate_type: {value}")),
        }
    }

    fn proj_max_to_str(value: &ProjMax) -> &'static str {
        match value {
            ProjMax::MINOR => "MINOR",
            ProjMax::MINOR_TOP => "MINOR_TOP",
            ProjMax::BOTTOM_6 => "BOTTOM_6",
            ProjMax::MID_6 => "MID_6",
            ProjMax::TOP4 => "TOP4",
            ProjMax::TOP2 => "TOP2",
            ProjMax::TOP1 => "TOP1",
            ProjMax::FRANCHISE => "FRANCHISE",
            ProjMax::SUPERSTAR => "SUPERSTAR",
            ProjMax::ELITE => "ELITE",
            ProjMax::GENERATIONAL => "GENERATIONAL",
        }
    }

    fn parse_proj_max(value: &str) -> Result<ProjMax, String> {
        match value {
            "MINOR" => Ok(ProjMax::MINOR),
            "MINOR_TOP" => Ok(ProjMax::MINOR_TOP),
            "BOTTOM_6" => Ok(ProjMax::BOTTOM_6),
            "MID_6" => Ok(ProjMax::MID_6),
            "TOP4" => Ok(ProjMax::TOP4),
            "TOP2" => Ok(ProjMax::TOP2),
            "TOP1" => Ok(ProjMax::TOP1),
            "FRANCHISE" => Ok(ProjMax::FRANCHISE),
            "SUPERSTAR" => Ok(ProjMax::SUPERSTAR),
            "ELITE" => Ok(ProjMax::ELITE),
            "GENERATIONAL" => Ok(ProjMax::GENERATIONAL),
            _ => Err(format!("Invalid max_projection: {value}")),
        }
    }

    fn development_curve_to_str(value: &DevelopmentCurve) -> &'static str {
        match value {
            DevelopmentCurve::EARLY => "EARLY",
            DevelopmentCurve::LINEAR => "LINEAR",
            DevelopmentCurve::LATE => "LATE",
            DevelopmentCurve::BOOM_BUST => "BOOM_BUST",
        }
    }

    fn parse_development_curve(value: &str) -> Result<DevelopmentCurve, String> {
        match value {
            "EARLY" => Ok(DevelopmentCurve::EARLY),
            "LINEAR" => Ok(DevelopmentCurve::LINEAR),
            "LATE" => Ok(DevelopmentCurve::LATE),
            "BOOM_BUST" => Ok(DevelopmentCurve::BOOM_BUST),
            _ => Err(format!("Invalid development_curve: {value}")),
        }
    }

    fn default_stats_for_player(player: &Player) -> PlayerStats {
        if matches!(player.position(), Position::GOALIE) {
            PlayerStats::goalie_default()
        } else {
            PlayerStats::skater_default()
        }
    }

    fn build_player_stats(
        player: &Player,
        games_played: Option<i16>,
        goals: Option<i16>,
        assists: Option<i16>,
        plus_minus: Option<i16>,
        penalty_minutes: Option<i16>,
        shots: Option<i16>,
        power_play_goals: Option<i16>,
        power_play_assists: Option<i16>,
        short_handed_goals: Option<i16>,
        short_handed_assists: Option<i16>,
        game_winning_goals: Option<i16>,
        overtime_goals: Option<i16>,
        faceoff_wins: Option<i16>,
        faceoff_losses: Option<i16>,
        hits: Option<i16>,
        blocked_shots: Option<i16>,
        takeaways: Option<i16>,
        giveaways: Option<i16>,
        time_on_ice_minutes: Option<i32>,
        goalie_starts: Option<i16>,
        goalie_wins: Option<i16>,
        goalie_losses: Option<i16>,
        goalie_ot_losses: Option<i16>,
        goalie_shots_against: Option<i16>,
        goalie_saves: Option<i16>,
        goalie_goals_against: Option<i16>,
        goalie_shutouts: Option<i16>,
        goalie_power_play_goals_against: Option<i16>,
        goalie_short_handed_goals_against: Option<i16>,
        goalie_time_on_ice_minutes: Option<i32>,
    ) -> PlayerStats {
        let goalie_stats = if matches!(player.position(), Position::GOALIE) {
            Some(GoalieStats::new(
                goalie_starts.unwrap_or(0),
                goalie_wins.unwrap_or(0),
                goalie_losses.unwrap_or(0),
                goalie_ot_losses.unwrap_or(0),
                goalie_shots_against.unwrap_or(0),
                goalie_saves.unwrap_or(0),
                goalie_goals_against.unwrap_or(0),
                goalie_shutouts.unwrap_or(0),
                goalie_power_play_goals_against.unwrap_or(0),
                goalie_short_handed_goals_against.unwrap_or(0),
                goalie_time_on_ice_minutes.unwrap_or(0),
            ))
        } else {
            None
        };

        PlayerStats::new(
            games_played.unwrap_or(0),
            goals.unwrap_or(0),
            assists.unwrap_or(0),
            plus_minus.unwrap_or(0),
            penalty_minutes.unwrap_or(0),
            shots.unwrap_or(0),
            power_play_goals.unwrap_or(0),
            power_play_assists.unwrap_or(0),
            short_handed_goals.unwrap_or(0),
            short_handed_assists.unwrap_or(0),
            game_winning_goals.unwrap_or(0),
            overtime_goals.unwrap_or(0),
            faceoff_wins.unwrap_or(0),
            faceoff_losses.unwrap_or(0),
            hits.unwrap_or(0),
            blocked_shots.unwrap_or(0),
            takeaways.unwrap_or(0),
            giveaways.unwrap_or(0),
            time_on_ice_minutes.unwrap_or(0),
            goalie_stats,
        )
    }

    fn write_stats_block(stats: &PlayerStats) -> String {
        if stats.is_default() {
            return "\n".to_string();
        }

        let mut block = format!(
            concat!(
            "\n",
            "games_played={}\n",
            "goals={}\n",
            "assists={}\n",
            "plus_minus={}\n",
            "penalty_minutes={}\n",
            "shots={}\n",
            "power_play_goals={}\n",
            "power_play_assists={}\n",
            "short_handed_goals={}\n",
            "short_handed_assists={}\n",
            "game_winning_goals={}\n",
            "overtime_goals={}\n",
            "faceoff_wins={}\n",
            "faceoff_losses={}\n",
            "hits={}\n",
            "blocked_shots={}\n",
            "takeaways={}\n",
            "giveaways={}\n",
            "time_on_ice_minutes={}\n"
            ),
            stats.games_played(),
            stats.goals(),
            stats.assists(),
            stats.plus_minus(),
            stats.penalty_minutes(),
            stats.shots(),
            stats.power_play_goals(),
            stats.power_play_assists(),
            stats.short_handed_goals(),
            stats.short_handed_assists(),
            stats.game_winning_goals(),
            stats.overtime_goals(),
            stats.faceoff_wins(),
            stats.faceoff_losses(),
            stats.hits(),
            stats.blocked_shots(),
            stats.takeaways(),
            stats.giveaways(),
            stats.time_on_ice_minutes(),
        );

        if let Some(goalie_stats) = stats.goalie_stats() {
            block.push_str(&format!(
                concat!(
                "goalie_starts={}\n",
                "goalie_wins={}\n",
                "goalie_losses={}\n",
                "goalie_ot_losses={}\n",
                "goalie_shots_against={}\n",
                "goalie_saves={}\n",
                "goalie_goals_against={}\n",
                "goalie_shutouts={}\n",
                "goalie_power_play_goals_against={}\n",
                "goalie_short_handed_goals_against={}\n",
                "goalie_time_on_ice_minutes={}\n"
                ),
                goalie_stats.starts(),
                goalie_stats.wins(),
                goalie_stats.losses(),
                goalie_stats.overtime_losses(),
                goalie_stats.shots_against(),
                goalie_stats.saves(),
                goalie_stats.goals_against(),
                goalie_stats.shutouts(),
                goalie_stats.power_play_goals_against(),
                goalie_stats.short_handed_goals_against(),
                goalie_stats.time_on_ice_minutes(),
            ));
        }

        block
    }
}


//TODO
fn default_stats_for_player(p0: &Player) -> PlayerStats {
    PlayerStats::skater_default()

}
