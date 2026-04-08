use crate::data::contract::{Contract, ContractType};
use crate::data::helper::{DraftData, DraftStatus, PlayerRecord};
use crate::data::movement::{GoalieMovement, SkatingStats, SkatingType};
use crate::data::player::{PlayType, Player, Position};
use crate::data::projection::{Projection, ProjectionGenerationSettings};
use crate::data::staff::{StaffMember, StaffRatings, StaffRole};
use crate::data::team::{Conference, Division, Team, TeamIdentity, TeamLevel};
use crate::season::LeagueState;
use crate::sim::{League, LeagueRules, LeagueTeamEntry, SimulationEngine, TeamStanding};
use std::fs;

pub struct TestLeagueConfig {
    league_name: String,
    team_count: usize,
    season_year: i16,
    games_per_matchup: i16,
    base_quality: f32,
    quality_step: f32,
}

pub struct TestLeagueBuilder {
    config: TestLeagueConfig,
}

enum TeamArchetype {
    Contender,
    Competitive,
    Rebuilding,
}

struct OrganizationRosters {
    major: Vec<PlayerRecord>,
    minor: Vec<PlayerRecord>,
}

impl TestLeagueConfig {
    pub fn new(
        league_name: String,
        team_count: usize,
        season_year: i16,
        games_per_matchup: i16,
        base_quality: f32,
        quality_step: f32,
    ) -> TestLeagueConfig {
        TestLeagueConfig {
            league_name,
            team_count,
            season_year,
            games_per_matchup,
            base_quality,
            quality_step,
        }
    }

    pub fn default() -> TestLeagueConfig {
        TestLeagueConfig::new("Autobuilt Test League".to_string(), 4, 2026, 4, 0.60, 0.04)
    }

    pub fn league_name(&self) -> &str { &self.league_name }
    pub fn team_count(&self) -> usize { self.team_count }
    pub fn season_year(&self) -> i16 { self.season_year }
    pub fn games_per_matchup(&self) -> i16 { self.games_per_matchup }
    pub fn base_quality(&self) -> f32 { self.base_quality }
    pub fn quality_step(&self) -> f32 { self.quality_step }

    pub fn read_text(input: &str) -> Result<TestLeagueConfig, String> {
        let mut league_name = None;
        let mut team_count = None;
        let mut season_year = None;
        let mut games_per_matchup = None;
        let mut base_quality = None;
        let mut quality_step = None;

        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid config line: {line}"))?;

            match key {
                "league_name" => league_name = Some(value.to_string()),
                "team_count" => team_count = Some(parse_usize(value, key)?),
                "season_year" => season_year = Some(parse_i16(value, key)?),
                "games_per_matchup" => games_per_matchup = Some(parse_i16(value, key)?),
                "base_quality" => base_quality = Some(parse_f32(value, key)?),
                "quality_step" => quality_step = Some(parse_f32(value, key)?),
                _ => {}
            }
        }

        Ok(TestLeagueConfig::new(
            require_value(league_name, "league_name")?,
            require_value(team_count, "team_count")?,
            require_value(season_year, "season_year")?,
            require_value(games_per_matchup, "games_per_matchup")?,
            require_value(base_quality, "base_quality")?,
            require_value(quality_step, "quality_step")?,
        ))
    }

    pub fn load_from_file(path: &str) -> Result<TestLeagueConfig, String> {
        let text = fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
        TestLeagueConfig::read_text(&text)
    }
}

impl TestLeagueBuilder {
    pub fn new(config: TestLeagueConfig) -> TestLeagueBuilder {
        TestLeagueBuilder { config }
    }

    pub fn default() -> TestLeagueBuilder {
        TestLeagueBuilder::new(TestLeagueConfig::default())
    }

    pub fn build_teams(&self) -> Vec<Team> {
        (0..self.config.team_count())
            .map(|index| self.build_major_team(index))
            .collect()
    }

    pub fn build_organization_teams(&self) -> Vec<Team> {
        let mut organizations = Vec::new();

        for index in 0..self.config.team_count() {
            let (major_team, minor_team) = self.build_team_pair(index);
            organizations.push(major_team);
            organizations.push(minor_team);
        }

        organizations
    }

    pub fn build_league_state(&self) -> LeagueState {
        let teams = self.build_teams();
        let league = League::new(self.config.league_name().to_string(), Vec::new());
        let mut state = LeagueState::new(league, teams);
        state.create_basic_season(self.config.season_year(), self.config.games_per_matchup());
        state
    }

    pub fn build_organization_league_state(&self) -> LeagueState {
        let teams = self.build_organization_teams();
        let team_registry = teams.iter().map(LeagueTeamEntry::from_team).collect();
        let standings = teams
            .iter()
            .filter(|team| matches!(team.level(), TeamLevel::MAJOR_PRO))
            .map(|team| TeamStanding::new(team.identity().abbreviation().to_string()))
            .collect();
        let league = League::new_custom(
            self.config.league_name().to_string(),
            TeamLevel::MAJOR_PRO,
            LeagueRules::nhl_style(),
            team_registry,
            standings,
        );
        let mut state = LeagueState::new(league, teams);
        state.create_target_games_season_for_level(
            self.config.season_year(),
            self.config.games_per_matchup(),
            TeamLevel::MAJOR_PRO,
        );
        state
    }

    pub fn simulate_season(&self, engine: &SimulationEngine, seed: u64) -> Result<LeagueState, String> {
        let mut state = self.build_organization_league_state();
        state.simulate_regular_season(engine, seed)?;
        Ok(state)
    }

    fn build_team_pair(&self, index: usize) -> (Team, Team) {
        let archetype = archetype_for_index(index, self.config.team_count());
        let quality = team_quality(index, self.config.base_quality(), self.config.quality_step(), &archetype)
            .clamp(0.35, 0.92);
        let abbreviation = format!("T{:02}", index + 1);
        let affiliate_abbreviation = format!("F{:02}", index + 1);
        let city = format!("Test City {}", index + 1);
        let name = format!("Club {}", index + 1);
        let affiliate_name = format!("Farm {}", index + 1);
        let rosters = build_organization_rosters(index, quality, &archetype, &abbreviation);

        let mut major_team = Team::new_full(
            TeamIdentity::new(
                city.clone(),
                name,
                abbreviation.clone(),
                if index % 2 == 0 { Conference::EAST } else { Conference::WEST },
                division_for_index(index),
            ),
            TeamLevel::MAJOR_PRO,
            rosters.major,
            build_staff(index, quality, &archetype, false),
            crate::data::stats::TeamStats::default(),
            crate::data::contract::TeamContractSettings::nhl_default(),
            vec![affiliate_abbreviation.clone()],
        );
        major_team.add_affiliate_team(affiliate_abbreviation.clone());

        let minor_team = Team::new_full(
            TeamIdentity::new(
                city,
                affiliate_name,
                affiliate_abbreviation,
                if index % 2 == 0 { Conference::EAST } else { Conference::WEST },
                division_for_index(index),
            ),
            TeamLevel::MINOR_PRO,
            rosters.minor,
            build_staff(index, (quality - 0.07).clamp(0.30, 0.86), &archetype, true),
            crate::data::stats::TeamStats::default(),
            crate::data::contract::TeamContractSettings::new(
                30.0,
                10.0,
                50,
                0,
                crate::data::contract::ContractLimits::new(1, 5, 0.100, 3.5, 0.100, 3.0, 0.0, 0.5, 0.0, 0.5, 0, 0, 0, 0),
            ),
            Vec::new(),
        );

        (major_team, minor_team)
    }

    fn build_major_team(&self, index: usize) -> Team {
        self.build_team_pair(index).0
    }
}

fn division_for_index(index: usize) -> Division {
    match index % 4 {
        0 => Division::ATLANTIC,
        1 => Division::METROPOLITAN,
        2 => Division::CENTRAL,
        _ => Division::PACIFIC,
    }
}

fn build_organization_rosters(
    team_index: usize,
    quality: f32,
    archetype: &TeamArchetype,
    organization_abbreviation: &str,
) -> OrganizationRosters {
    OrganizationRosters {
        major: build_major_roster(team_index, quality, archetype, organization_abbreviation),
        minor: build_minor_roster(team_index, quality, archetype, organization_abbreviation),
    }
}

fn build_major_roster(
    team_index: usize,
    quality: f32,
    archetype: &TeamArchetype,
    organization_abbreviation: &str,
) -> Vec<PlayerRecord> {
    let age_bias = archetype_age_bias(archetype);
    let prospect_bump = archetype_projection_bump(archetype);
    let mut roster = Vec::new();

    let forward_positions = [
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::LW,
        Position::RW,
    ];
    let forward_styles = [
        PlayType::PLAYMAKER,
        PlayType::SNIPER,
        PlayType::PWF,
        PlayType::PLAYMAKER,
        PlayType::SNIPER,
        PlayType::PWF,
        PlayType::DF,
        PlayType::PWF,
        PlayType::DF,
        PlayType::DF,
        PlayType::DF,
        PlayType::PWF,
        PlayType::SNIPER,
        PlayType::DF,
    ];

    for slot in 0..forward_positions.len() {
        let line = slot / 3;
        roster.push(build_skater(
            generated_player_name(team_index, slot, false),
            clone_position(&forward_positions[slot]),
            clone_play_type(&forward_styles[slot]),
            quality + forward_quality_offset(slot),
            (23 + line as i8 + age_bias).clamp(18, 36),
            (quality + line_quality_projection_offset(line) + prospect_bump).clamp(0.35, 0.98),
            slot < 3,
            false,
            archetype,
            organization_abbreviation,
            slot as i16 + 1,
        ));
    }

    let defense_positions = [
        Position::LD,
        Position::RD,
        Position::LD,
        Position::RD,
        Position::LD,
        Position::RD,
        Position::LD,
    ];
    let defense_styles = [
        PlayType::OFD,
        PlayType::DFD,
        PlayType::DFD,
        PlayType::OFD,
        PlayType::DFD,
        PlayType::OFD,
        PlayType::DFD,
    ];

    for slot in 0..defense_positions.len() {
        roster.push(build_skater(
            generated_player_name(team_index, 20 + slot, false),
            clone_position(&defense_positions[slot]),
            clone_play_type(&defense_styles[slot]),
            quality + defense_quality_offset(slot),
            (25 + (slot / 2) as i8 + age_bias).clamp(18, 36),
            (quality + defense_quality_offset(slot) + prospect_bump).clamp(0.35, 0.98),
            slot < 2,
            false,
            archetype,
            organization_abbreviation,
            15 + slot as i16,
        ));
    }

    roster.push(build_goalie(
        generated_player_name(team_index, 40, true),
        quality + 0.03,
        (27 + age_bias).clamp(18, 36),
        (quality + 0.02 + prospect_bump).clamp(0.35, 0.98),
        true,
        false,
        archetype,
        organization_abbreviation,
        40,
    ));
    roster.push(build_goalie(
        generated_player_name(team_index, 41, true),
        quality - 0.03,
        (29 + age_bias).clamp(18, 36),
        (quality - 0.01 + prospect_bump).clamp(0.35, 0.98),
        false,
        false,
        archetype,
        organization_abbreviation,
        41,
    ));

    roster
}

fn build_minor_roster(
    team_index: usize,
    quality: f32,
    archetype: &TeamArchetype,
    organization_abbreviation: &str,
) -> Vec<PlayerRecord> {
    let age_bias = archetype_age_bias(archetype) - 3;
    let prospect_bump = archetype_projection_bump(archetype) + 0.06;
    let mut roster = Vec::new();

    let forward_positions = [
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
        Position::CENTER,
        Position::LW,
        Position::RW,
    ];

    for slot in 0..forward_positions.len() {
        let line = slot / 3;
        roster.push(build_skater(
            generated_player_name(team_index, 60 + slot, false),
            clone_position(&forward_positions[slot]),
            if slot % 4 == 0 { PlayType::PLAYMAKER } else if slot % 3 == 0 { PlayType::SNIPER } else { PlayType::PWF },
            quality + farm_forward_quality_offset(slot),
            (20 + line as i8 + age_bias).clamp(18, 31),
            (quality + farm_projection_offset(slot) + prospect_bump).clamp(0.40, 0.99),
            false,
            true,
            archetype,
            organization_abbreviation,
            60 + slot as i16,
        ));
    }

    for slot in 0..9 {
        roster.push(build_skater(
            generated_player_name(team_index, 90 + slot, false),
            if slot % 2 == 0 { Position::LD } else { Position::RD },
            if slot % 3 == 0 { PlayType::OFD } else { PlayType::DFD },
            quality + farm_defense_quality_offset(slot),
            (21 + (slot / 3) as i8 + age_bias).clamp(18, 31),
            (quality + farm_defense_quality_offset(slot) + prospect_bump).clamp(0.40, 0.99),
            false,
            true,
            archetype,
            organization_abbreviation,
            90 + slot as i16,
        ));
    }

    for slot in 0..3 {
        roster.push(build_goalie(
            generated_player_name(team_index, 120 + slot, true),
            quality + farm_goalie_quality_offset(slot),
            (21 + slot as i8 + age_bias).clamp(18, 32),
            (quality + farm_goalie_quality_offset(slot) + prospect_bump).clamp(0.42, 0.99),
            false,
            true,
            archetype,
            organization_abbreviation,
            120 + slot as i16,
        ));
    }

    roster
}

fn build_staff(team_index: usize, quality: f32, archetype: &TeamArchetype, is_minor: bool) -> Vec<StaffMember> {
    let staff_bonus = match archetype {
        TeamArchetype::Contender => 4,
        TeamArchetype::Competitive => 0,
        TeamArchetype::Rebuilding => -2,
    } + if is_minor { -3 } else { 0 };
    let tactical = (scale_rating(quality) + staff_bonus).clamp(38, 99);
    let staff_roles = if is_minor {
        vec![
            StaffRole::HEAD_COACH,
            StaffRole::ASSISTANT_COACH,
            StaffRole::DEVELOPMENT_COACH,
            StaffRole::GOALIE_COACH,
            StaffRole::HEAD_SCOUT,
            StaffRole::SCOUT,
        ]
    } else {
        vec![
            StaffRole::GENERAL_MANAGER,
            StaffRole::HEAD_COACH,
            StaffRole::ASSISTANT_COACH,
            StaffRole::DEVELOPMENT_COACH,
            StaffRole::GOALIE_COACH,
            StaffRole::HEAD_SCOUT,
            StaffRole::SCOUT,
            StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT,
        ]
    };

    staff_roles
        .iter()
        .enumerate()
        .map(|(slot, role)| {
            let role_bias = match role {
                StaffRole::GENERAL_MANAGER => (0, -2, 5, 4),
                StaffRole::HEAD_COACH => (-2, 2, -4, 3),
                StaffRole::ASSISTANT_COACH => (-1, -2, -5, 1),
                StaffRole::DEVELOPMENT_COACH => (4, -5, -4, 0),
                StaffRole::GOALIE_COACH => (3, -6, -5, 0),
                StaffRole::HEAD_SCOUT => (0, -6, 6, 1),
                StaffRole::SCOUT => (-1, -7, 4, 0),
                StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT => (5, -4, 1, 2),
                _ => (0, 0, 0, 0),
            };

            StaffMember::new(
                generated_staff_name(team_index, slot, is_minor),
                if is_minor { 38 + slot as i8 } else { 41 + slot as i8 },
                role_clone(role),
                StaffRatings::new(
                    (tactical + role_bias.0).clamp(35, 99),
                    (tactical + role_bias.1).clamp(35, 99),
                    (tactical + role_bias.2).clamp(35, 99),
                    (tactical + role_bias.3).clamp(35, 99),
                ),
            )
        })
        .collect()
}

fn build_skater(
    name: String,
    position: Position,
    play_type: PlayType,
    quality: f32,
    age: i8,
    projection_quality: f32,
    top_player: bool,
    farm_player: bool,
    archetype: &TeamArchetype,
    organization_abbreviation: &str,
    draft_slot: i16,
) -> PlayerRecord {
    let rating = scale_rating(quality);
    let mut player = PlayerRecord::new(
        name,
        age,
        Player::new_skater(
            position,
            play_type,
            if rating >= 82 { SkatingType::QUICK } else { SkatingType::STRONG },
            SkatingStats::new(rating, (rating - 2).clamp(1, 99), (rating - 1).clamp(1, 99), SkatingType::QUICK),
        ),
        Projection::from_quality_with_settings(
            projection_quality,
            projection_settings_for_archetype(archetype, farm_player),
        ),
        build_draft_status(age, organization_abbreviation, draft_slot, farm_player),
    );

    player.set_contract(Some(player_contract(age, quality, top_player, farm_player)));
    player
}

fn build_goalie(
    name: String,
    quality: f32,
    age: i8,
    projection_quality: f32,
    starter: bool,
    farm_player: bool,
    archetype: &TeamArchetype,
    organization_abbreviation: &str,
    draft_slot: i16,
) -> PlayerRecord {
    let rating = scale_rating(quality);
    let mut goalie = PlayerRecord::new(
        name,
        age,
        Player::new_goalie(
            PlayType::HYBRID,
            if rating >= 80 { SkatingType::QUICK } else { SkatingType::STRONG },
            SkatingStats::new((rating - 5).clamp(1, 99), (rating - 3).clamp(1, 99), (rating - 4).clamp(1, 99), SkatingType::STRONG),
            GoalieMovement::new(rating, (rating - 1).clamp(1, 99), (rating + 1).clamp(1, 99)),
        ),
        Projection::from_quality_with_settings(
            projection_quality,
            goalie_projection_settings_for_archetype(archetype, farm_player),
        ),
        build_draft_status(age, organization_abbreviation, draft_slot, farm_player),
    );

    goalie.set_contract(Some(goalie_contract(age, quality, starter, farm_player)));
    goalie
}

fn build_draft_status(age: i8, organization_abbreviation: &str, draft_slot: i16, farm_player: bool) -> DraftStatus {
    if age <= 18 {
        DraftStatus::undrafted()
    } else {
        let draft_year = 2026 - (age as i16 - 18);
        let round = if farm_player { ((draft_slot % 6) + 2) as i8 } else { ((draft_slot % 4) + 1) as i8 };
        DraftStatus::drafted(DraftData::new(draft_year, round, draft_slot, organization_abbreviation.to_string()))
    }
}

fn player_contract(age: i8, quality: f32, top_player: bool, farm_player: bool) -> Contract {
    if farm_player {
        return Contract::new(
            ContractType::TWO_WAY,
            if age <= 22 { 2 } else { 1 },
            (0.85 + quality * 0.65).clamp(0.85, 1.60),
            (0.75 + quality * 0.55).clamp(0.75, 1.35),
            0.0,
            if age <= 21 { 0.10 } else { 0.0 },
            0,
            0,
        );
    }

    if age <= 21 {
        return Contract::new(
            ContractType::ENTRY_LEVEL,
            3,
            (0.90 + quality * 0.55).clamp(0.90, 1.45),
            (0.85 + quality * 0.45).clamp(0.85, 1.25),
            0.05,
            0.10,
            0,
            0,
        );
    }

    if top_player {
        return Contract::new(
            ContractType::STANDARD,
            6,
            (6.5 + quality * 4.0).clamp(6.5, 10.8),
            (6.0 + quality * 3.6).clamp(6.0, 9.8),
            0.8,
            0.2,
            1,
            0,
        );
    }

    Contract::new(
        ContractType::BRIDGE,
        if age <= 25 { 3 } else { 2 },
        (1.5 + quality * 3.2).clamp(1.5, 5.2),
        (1.3 + quality * 2.9).clamp(1.3, 4.8),
        0.15,
        0.05,
        0,
        0,
    )
}

fn goalie_contract(age: i8, quality: f32, starter: bool, farm_player: bool) -> Contract {
    if farm_player {
        return Contract::new(
            ContractType::TWO_WAY,
            if age <= 23 { 2 } else { 1 },
            (0.90 + quality * 0.60).clamp(0.90, 1.55),
            (0.80 + quality * 0.50).clamp(0.80, 1.30),
            0.0,
            0.05,
            0,
            0,
        );
    }

    if starter {
        return Contract::new(
            ContractType::STANDARD,
            if age <= 27 { 4 } else { 5 },
            (4.8 + quality * 3.6).clamp(4.8, 8.0),
            (4.5 + quality * 3.2).clamp(4.5, 7.5),
            0.35,
            0.05,
            0,
            0,
        );
    }

    if age <= 22 {
        return Contract::new(
            ContractType::ENTRY_LEVEL,
            3,
            (0.90 + quality * 0.50).clamp(0.90, 1.35),
            (0.85 + quality * 0.45).clamp(0.85, 1.20),
            0.05,
            0.10,
            0,
            0,
        );
    }

    Contract::new(
        ContractType::ONE_WAY,
        2,
        (1.25 + quality * 1.8).clamp(1.25, 3.2),
        (1.10 + quality * 1.6).clamp(1.10, 2.9),
        0.10,
        0.0,
        0,
        0,
    )
}

fn projection_settings_for_archetype(archetype: &TeamArchetype, farm_player: bool) -> ProjectionGenerationSettings {
    let base: (f32, f32, f32, f32, f32, f32, f32) = match archetype {
        TeamArchetype::Contender => (0.78, 0.75, 0.76, 0.18, 0.24, 0.68, 0.70),
        TeamArchetype::Competitive => (0.68, 0.70, 0.70, 0.26, 0.22, 0.74, 0.76),
        TeamArchetype::Rebuilding => (0.58, 0.64, 0.66, 0.32, 0.20, 0.84, 0.86),
    };
    let farm_bump = if farm_player { 0.08 } else { 0.0 };
    ProjectionGenerationSettings::new(
        (base.0 - if farm_player { 0.08 } else { 0.0 }).clamp(0.35, 0.95),
        (base.1 - if farm_player { 0.06 } else { 0.0 }).clamp(0.35, 0.95),
        (base.2 - if farm_player { 0.04 } else { 0.0 }).clamp(0.35, 0.95),
        base.3 + if farm_player { 0.04 } else { 0.0 },
        base.4,
        (base.5 + farm_bump).clamp(0.0, 1.0),
        (base.6 + farm_bump).clamp(0.0, 1.0),
    )
}

fn goalie_projection_settings_for_archetype(archetype: &TeamArchetype, farm_player: bool) -> ProjectionGenerationSettings {
    let base: (f32, f32, f32, f32, f32, f32, f32) = match archetype {
        TeamArchetype::Contender => (0.76, 0.72, 0.68, 0.18, 0.24, 0.68, 0.70),
        TeamArchetype::Competitive => (0.67, 0.68, 0.62, 0.24, 0.20, 0.72, 0.74),
        TeamArchetype::Rebuilding => (0.55, 0.62, 0.60, 0.30, 0.18, 0.82, 0.84),
    };
    let farm_bump = if farm_player { 0.10 } else { 0.0 };
    ProjectionGenerationSettings::new(
        (base.0 - if farm_player { 0.10 } else { 0.0 }).clamp(0.35, 0.95),
        (base.1 - if farm_player { 0.06 } else { 0.0 }).clamp(0.35, 0.95),
        (base.2 - if farm_player { 0.04 } else { 0.0 }).clamp(0.35, 0.95),
        base.3 + if farm_player { 0.04 } else { 0.0 },
        base.4,
        (base.5 + farm_bump).clamp(0.0, 1.0),
        (base.6 + farm_bump).clamp(0.0, 1.0),
    )
}

fn generated_player_name(team_index: usize, slot: usize, goalie: bool) -> String {
    let first = FIRST_NAMES[mix_index(team_index, slot, if goalie { 11 } else { 5 }) % FIRST_NAMES.len()];
    let last = LAST_NAMES[mix_index(team_index, slot, if goalie { 23 } else { 17 }) % LAST_NAMES.len()];
    format!("{first} {last}")
}

fn generated_staff_name(team_index: usize, slot: usize, is_minor: bool) -> String {
    let first = FIRST_NAMES[mix_index(team_index, slot, if is_minor { 31 } else { 29 }) % FIRST_NAMES.len()];
    let last = LAST_NAMES[mix_index(team_index, slot, if is_minor { 41 } else { 37 }) % LAST_NAMES.len()];
    format!("{first} {last}")
}

fn mix_index(team_index: usize, slot: usize, salt: usize) -> usize {
    team_index
        .wrapping_mul(97)
        .wrapping_add(slot.wrapping_mul(53))
        .wrapping_add(salt.wrapping_mul(19))
}

fn forward_quality_offset(slot: usize) -> f32 {
    match slot {
        0..=2 => 0.07,
        3..=5 => 0.03,
        6..=8 => -0.02,
        9..=11 => -0.07,
        _ => -0.10,
    }
}

fn line_quality_projection_offset(line: usize) -> f32 {
    match line {
        0 => 0.05,
        1 => 0.02,
        2 => -0.02,
        _ => -0.05,
    }
}

fn defense_quality_offset(slot: usize) -> f32 {
    match slot {
        0 | 1 => 0.04,
        2 | 3 => -0.01,
        4 | 5 => -0.05,
        _ => -0.08,
    }
}

fn farm_forward_quality_offset(slot: usize) -> f32 {
    match slot {
        0..=2 => -0.06,
        3..=5 => -0.10,
        6..=8 => -0.14,
        9..=11 => -0.18,
        _ => -0.22,
    }
}

fn farm_defense_quality_offset(slot: usize) -> f32 {
    match slot {
        0..=1 => -0.08,
        2..=3 => -0.12,
        4..=5 => -0.16,
        _ => -0.20,
    }
}

fn farm_goalie_quality_offset(slot: usize) -> f32 {
    match slot {
        0 => -0.08,
        1 => -0.13,
        _ => -0.18,
    }
}

fn farm_projection_offset(slot: usize) -> f32 {
    match slot {
        0..=2 => 0.10,
        3..=5 => 0.08,
        6..=8 => 0.06,
        9..=11 => 0.04,
        _ => 0.02,
    }
}

fn scale_rating(quality: f32) -> i8 {
    (56.0 + quality.clamp(0.0, 1.0) * 36.0).round() as i8
}

fn archetype_for_index(index: usize, team_count: usize) -> TeamArchetype {
    let percentile = index as f32 / team_count.max(1) as f32;
    if percentile < 0.25 {
        TeamArchetype::Contender
    } else if percentile < 0.75 {
        TeamArchetype::Competitive
    } else {
        TeamArchetype::Rebuilding
    }
}

fn team_quality(index: usize, base_quality: f32, quality_step: f32, archetype: &TeamArchetype) -> f32 {
    match archetype {
        TeamArchetype::Contender => base_quality + 0.16 + quality_step * index as f32,
        TeamArchetype::Competitive => base_quality + 0.02 + quality_step * (index as f32 * 0.45),
        TeamArchetype::Rebuilding => base_quality - 0.10 + quality_step * (index as f32 * 0.20),
    }
}

fn archetype_age_bias(archetype: &TeamArchetype) -> i8 {
    match archetype {
        TeamArchetype::Contender => 2,
        TeamArchetype::Competitive => 0,
        TeamArchetype::Rebuilding => -3,
    }
}

fn archetype_projection_bump(archetype: &TeamArchetype) -> f32 {
    match archetype {
        TeamArchetype::Contender => -0.01,
        TeamArchetype::Competitive => 0.02,
        TeamArchetype::Rebuilding => 0.14,
    }
}

fn role_clone(role: &StaffRole) -> StaffRole {
    match role {
        StaffRole::GENERAL_MANAGER => StaffRole::GENERAL_MANAGER,
        StaffRole::ASSISTANT_GENERAL_MANAGER => StaffRole::ASSISTANT_GENERAL_MANAGER,
        StaffRole::HEAD_COACH => StaffRole::HEAD_COACH,
        StaffRole::ASSISTANT_COACH => StaffRole::ASSISTANT_COACH,
        StaffRole::DEVELOPMENT_COACH => StaffRole::DEVELOPMENT_COACH,
        StaffRole::HEAD_SCOUT => StaffRole::HEAD_SCOUT,
        StaffRole::GOALIE_COACH => StaffRole::GOALIE_COACH,
        StaffRole::SKATING_COACH => StaffRole::SKATING_COACH,
        StaffRole::SCOUT => StaffRole::SCOUT,
        StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT => StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT,
        StaffRole::OWNER => StaffRole::OWNER,
    }
}

fn clone_position(position: &Position) -> Position {
    match position {
        Position::CENTER => Position::CENTER,
        Position::LW => Position::LW,
        Position::RW => Position::RW,
        Position::RD => Position::RD,
        Position::LD => Position::LD,
        Position::GOALIE => Position::GOALIE,
    }
}

fn clone_play_type(play_type: &PlayType) -> PlayType {
    match play_type {
        PlayType::SNIPER => PlayType::SNIPER,
        PlayType::OFD => PlayType::OFD,
        PlayType::DFD => PlayType::DFD,
        PlayType::PWF => PlayType::PWF,
        PlayType::DF => PlayType::DF,
        PlayType::PLAYMAKER => PlayType::PLAYMAKER,
        PlayType::BUTTERFLY => PlayType::BUTTERFLY,
        PlayType::REACTIVE => PlayType::REACTIVE,
        PlayType::HYBRID => PlayType::HYBRID,
    }
}

fn require_value<T>(value: Option<T>, field: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required config field: {field}"))
}

fn parse_i16(value: &str, field: &str) -> Result<i16, String> {
    value.parse::<i16>().map_err(|_| format!("invalid {field}: {value}"))
}

fn parse_usize(value: &str, field: &str) -> Result<usize, String> {
    value.parse::<usize>().map_err(|_| format!("invalid {field}: {value}"))
}

fn parse_f32(value: &str, field: &str) -> Result<f32, String> {
    value.parse::<f32>().map_err(|_| format!("invalid {field}: {value}"))
}

const FIRST_NAMES: [&str; 64] = [
    "Liam", "Noah", "Mason", "Lucas", "Ethan", "Owen", "Jack", "Logan",
    "Carter", "Wyatt", "Hudson", "Caleb", "Dylan", "Parker", "Nolan", "Connor",
    "Brady", "Cole", "Gavin", "Jace", "Tyler", "Reid", "Hayes", "Blake",
    "Aiden", "Griffin", "Roman", "Cade", "Joel", "Adam", "Ben", "Micah",
    "Jonah", "Colin", "Tanner", "Eli", "Milo", "Sawyer", "Austin", "Nate",
    "Rhys", "Brooks", "Finn", "Declan", "Landon", "Tristan", "Spencer", "Trevor",
    "Alex", "Ryan", "Matt", "Chris", "Jake", "Sam", "Nick", "Grant",
    "Shane", "Drew", "Coleman", "Jared", "Kieran", "Malik", "Teo", "Vince",
];

const LAST_NAMES: [&str; 64] = [
    "Anderson", "Bennett", "Carson", "Donovan", "Ellis", "Foster", "Graves", "Hayes",
    "Irwin", "Jensen", "Keller", "Larson", "Morrison", "Nolan", "Olsen", "Parker",
    "Quinn", "Reilly", "Sullivan", "Turner", "Underwood", "Vaughn", "Walker", "Xu",
    "Young", "Zimmer", "Adler", "Bishop", "Callahan", "Drake", "Emerson", "Fisher",
    "Gibson", "Hart", "Iverson", "Jamieson", "Keane", "Lindholm", "Macklin", "Novak",
    "Ortega", "Prescott", "Rasmussen", "Shepard", "Talbot", "Ulrich", "Volkov", "Whitaker",
    "York", "Zarnecki", "Bourne", "Cross", "Dawson", "Easton", "Flint", "Gallagher",
    "Holloway", "Ingram", "Jeffers", "Keegan", "Locke", "Mercer", "Nystrom", "Osborne",
];
