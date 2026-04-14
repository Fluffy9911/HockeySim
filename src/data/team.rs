use crate::data::contract::TeamContractSettings;
use crate::data::helper::PlayerRecord;
use crate::data::staff::{StaffMember, StaffRole};
use crate::data::stats::TeamStats;
use serde::{Deserialize, Serialize};
use crate::data::game::names::*;
use crate::data::line::{GoalieTandem, Line, Loadout, Pairing};
use crate::data::player::Position;

#[derive(Serialize, Deserialize,Copy,Clone)]
pub enum TeamLevel {
    MAJOR_PRO,
    MINOR_PRO,
    JUNIOR,
    COLLEGE,
    INTERNATIONAL,
    OTHER,
}

#[derive(Serialize, Deserialize)]
pub struct TeamIdentity {
    city: String,
   pub name: String,
    abbreviation: String,
    conference: Conference,
    division: Division,
}

#[derive(Serialize, Deserialize)]
pub struct Team {
   pub identity: TeamIdentity,
    level: TeamLevel,
    roster: Vec<PlayerRecord>,
    staff: Vec<StaffMember>,
    team_stats: TeamStats,
    contract_settings: TeamContractSettings,
    lines:Loadout
}





impl TeamIdentity {
    pub fn new(
        city: String,
        name: String,
        abbreviation: String,
        conference: Conference,
        division: Division,
    ) -> TeamIdentity {
        TeamIdentity { city, name, abbreviation, conference, division }
    }

    pub fn city(&self) -> &str { &self.city }
    pub fn name(&self) -> &str { &self.name }
    pub fn abbreviation(&self) -> &str { &self.abbreviation }
    pub fn conference(&self) -> &Conference { &self.conference }
    pub fn division(&self) -> &Division { &self.division }
}

impl Team {
    pub fn new(identity: TeamIdentity, roster: Vec<PlayerRecord>, staff: Vec<StaffMember>,) -> Team {
        Team {
            identity,
            level: TeamLevel::MAJOR_PRO,
            roster,
            staff,
            team_stats: TeamStats::default(),
            contract_settings: TeamContractSettings::nhl_default(),
            lines: Loadout::none()
        }
    }

    pub fn new_with_stats(
        identity: TeamIdentity,
        roster: Vec<PlayerRecord>,
        staff: Vec<StaffMember>,
        team_stats: TeamStats,
    ) -> Team {
        Team {
            identity,
            level: TeamLevel::MAJOR_PRO,
            roster,
            staff,
            team_stats,
            contract_settings: TeamContractSettings::nhl_default(),
            lines: Loadout::none()
        }
    }

    pub fn new_with_contract_settings(
        identity: TeamIdentity,
        roster: Vec<PlayerRecord>,
        staff: Vec<StaffMember>,
        team_stats: TeamStats,
        contract_settings: TeamContractSettings,
    ) -> Team {
        Team {
            identity,
            level: TeamLevel::MAJOR_PRO,
            roster,
            staff,
            team_stats,
            contract_settings,
            lines: Loadout::none()
        }
    }

    pub fn new_full(
        identity: TeamIdentity,
        level: TeamLevel,
        roster: Vec<PlayerRecord>,
        staff: Vec<StaffMember>,
        team_stats: TeamStats,
        contract_settings: TeamContractSettings,lines:Loadout
    ) -> Team {
        Team { identity, level, roster, staff, team_stats, contract_settings,lines }
    }

    pub fn identity(&self) -> &TeamIdentity { &self.identity }
    pub fn roster(&self) -> &[PlayerRecord] { &self.roster }
    pub fn roster_mut(&mut self) -> &mut [PlayerRecord] { &mut self.roster }
    pub fn level(&self) -> &TeamLevel { &self.level }
    pub fn staff(&self) -> &[StaffMember] { &self.staff }
    pub fn staff_mut(&mut self) -> &mut [StaffMember] { &mut self.staff }
    pub fn team_stats(&self) -> &TeamStats { &self.team_stats }
    pub fn team_stats_mut(&mut self) -> &mut TeamStats { &mut self.team_stats }
    pub fn contract_settings(&self) -> &TeamContractSettings { &self.contract_settings }

    pub fn set_contract_settings(&mut self, contract_settings: TeamContractSettings) {
        self.contract_settings = contract_settings;
    }



    pub fn add_player(&mut self, player: PlayerRecord) {
        self.roster.push(player);
    }

    pub fn add_players(&mut self, players:&mut Vec<PlayerRecord>) {

        self.roster.append(players)
    }

    pub fn add_staff_member(&mut self, staff_member: StaffMember) {
        self.staff.push(staff_member);
    }

    pub fn head_coach(&self) -> Option<&StaffMember> {
        self.staff.iter().find(|m| matches!(m.role(), StaffRole::HEAD_COACH))
    }

    pub fn head_scout(&self) -> Option<&StaffMember> {
        self.staff.iter().find(|m| matches!(m.role(), StaffRole::HEAD_SCOUT))
    }

    pub fn development_coaches(&self) -> Vec<&StaffMember> {
        self.staff
            .iter()
            .filter(|m| {
                matches!(
                    m.role(),
                    StaffRole::DEVELOPMENT_COACH
                        | StaffRole::GOALIE_COACH
                        | StaffRole::SKATING_COACH
                        | StaffRole::DIRECTOR_OF_PLAYER_DEVELOPMENT
                )
            })
            .collect()
    }

    pub fn scouts(&self) -> Vec<&StaffMember> {
        self.staff
            .iter()
            .filter(|m| matches!(m.role(), StaffRole::HEAD_SCOUT | StaffRole::SCOUT))
            .collect()
    }

    pub fn active_contract_count(&self) -> usize {
        self.roster.iter().filter(|p| p.contract().is_some()).count()
    }

    pub fn total_cap_hit_millions(&self) -> f32 {
        self.roster
            .iter()
            .filter_map(|p| p.contract())
            .map(|c| c.cap_hit_millions())
            .sum()
    }
}
pub fn auto_assign_lines(team: &mut Team) {
    let roster = &team.roster;

    // --- Collect indices by position ---
    let mut lw: Vec<(usize, i8)> = vec![];
    let mut c: Vec<(usize, i8)> = vec![];
    let mut rw: Vec<(usize, i8)> = vec![];
    let mut ld: Vec<(usize, i8)> = vec![];
    let mut rd: Vec<(usize, i8)> = vec![];
    let mut g: Vec<(usize, i8)> = vec![];

    for (i, p) in roster.iter().enumerate() {
        let entry = (i, p.player().overall());

        match p.player().position() {
            Position::LW => lw.push(entry),
            Position::CENTER => c.push(entry),
            Position::RW => rw.push(entry),
            Position::LD => ld.push(entry),
            Position::RD => rd.push(entry),
            Position::GOALIE => g.push(entry),
        }
    }

    // --- Sort each group by overall DESC ---
    let sort_desc = |v: &mut Vec<(usize, i8)>| {
        v.sort_by(|a, b| b.1.cmp(&a.1));
    };

    sort_desc(&mut lw);
    sort_desc(&mut c);
    sort_desc(&mut rw);
    sort_desc(&mut ld);
    sort_desc(&mut rd);
    sort_desc(&mut g);

    // Helper to safely get index or fallback
    let get = |v: &Vec<(usize, i8)>, i: usize| -> i8 {
        v.get(i).map(|x| x.0 as i8).unwrap_or(-1)
    };

    // --- Build lines ---
    let l1 = Line::new(get(&lw, 0), get(&c, 0), get(&rw, 0));
    let l2 = Line::new(get(&lw, 1), get(&c, 1), get(&rw, 1));
    let l3 = Line::new(get(&lw, 2), get(&c, 2), get(&rw, 2));
    let l4 = Line::new(get(&lw, 3), get(&c, 3), get(&rw, 3));

    // --- Defense pairings ---
    let d1 = Pairing::new(get(&ld, 0), get(&rd, 0));
    let d2 = Pairing::new(get(&ld, 1), get(&rd, 1));
    let d3 = Pairing::new(get(&ld, 2), get(&rd, 2));

    // --- Goalies ---
    let starter = get(&g, 0);
    let backup = get(&g, 1);

    let goalies = GoalieTandem::new(starter, backup, 70);

    // --- Assign to team ---
    team.lines = Loadout::builder()
        .l1(l1)
        .l2(l2)
        .l3(l3)
        .l4(l4)
        .d1(d1)
        .d2(d2)
        .d3(d3)
        .goalies(goalies)
        .build();
}