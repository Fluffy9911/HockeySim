use serde::{Deserialize, Serialize};
#[derive(Serialize,Deserialize)]
pub struct Line{

    lw:i8,
    rw:i8,
    center:i8,



}
#[derive(Serialize,Deserialize)]
pub struct Pairing {

    rd:i8,
    ld:i8,

}
#[derive(Serialize,Deserialize)]
pub struct GoalieTandem {

    starter:i8,
    backup:i8,
    percent:i8,


}

#[derive(Serialize,Deserialize)]
pub struct Loadout{
    l1:Line,
    l2:Line,
    l3:Line,
    l4:Line,
    d1:Pairing,
    d2:Pairing,
    d3:Pairing,
    goalies:GoalieTandem

}

impl Loadout {
    pub(crate) fn builder() -> LoadoutBuilder {
        LoadoutBuilder::new()
    }
}

impl Line {
    pub fn new(lw: i8, center: i8, rw: i8) -> Self {
        Self { lw, center, rw }
    }

    pub fn lw(&self) -> i8 {
        self.lw
    }

    pub fn center(&self) -> i8 {
        self.center
    }

    pub fn rw(&self) -> i8 {
        self.rw
    }

    pub fn set_lw(&mut self, lw: i8) {
        self.lw = lw;
    }

    pub fn set_center(&mut self, center: i8) {
        self.center = center;
    }

    pub fn set_rw(&mut self, rw: i8) {
        self.rw = rw;
    }
}
impl Pairing {
    pub fn new(ld: i8, rd: i8) -> Self {
        Self { ld, rd }
    }

    pub fn ld(&self) -> i8 {
        self.ld
    }

    pub fn rd(&self) -> i8 {
        self.rd
    }

    pub fn set_ld(&mut self, ld: i8) {
        self.ld = ld;
    }

    pub fn set_rd(&mut self, rd: i8) {
        self.rd = rd;
    }
}
impl GoalieTandem {
    pub fn new(starter: i8, backup: i8, percent: i8) -> Self {
        Self {
            starter,
            backup,
            percent,
        }
    }

    pub fn starter(&self) -> i8 {
        self.starter
    }

    pub fn backup(&self) -> i8 {
        self.backup
    }

    pub fn percent(&self) -> i8 {
        self.percent
    }

    pub fn set_starter(&mut self, starter: i8) {
        self.starter = starter;
    }

    pub fn set_backup(&mut self, backup: i8) {
        self.backup = backup;
    }

    pub fn set_percent(&mut self, percent: i8) {
        self.percent = percent;
    }
}

pub struct LoadoutBuilder {
    l1: Option<Line>,
    l2: Option<Line>,
    l3: Option<Line>,
    l4: Option<Line>,
    d1: Option<Pairing>,
    d2: Option<Pairing>,
    d3: Option<Pairing>,
    goalies: Option<GoalieTandem>,
}

impl Loadout{

    pub fn none()-> Loadout{

        LoadoutBuilder::new().l1_raw(0,0,0).l2_raw(0,0,0).l3_raw(0,0,0).l4_raw(0,0,0).d1_raw(0,0).d2_raw(0,0).d3_raw(0,0).goalies_raw(0,0,1).build()

    }


}

impl LoadoutBuilder {
    pub fn new() -> Self {
        Self {
            l1: None,
            l2: None,
            l3: None,
            l4: None,
            d1: None,
            d2: None,
            d3: None,
            goalies: None,
        }
    }

    // --- Line setters (struct) ---

    pub fn l1(mut self, line: Line) -> Self {
        self.l1 = Some(line);
        self
    }

    pub fn l2(mut self, line: Line) -> Self {
        self.l2 = Some(line);
        self
    }

    pub fn l3(mut self, line: Line) -> Self {
        self.l3 = Some(line);
        self
    }

    pub fn l4(mut self, line: Line) -> Self {
        self.l4 = Some(line);
        self
    }

    // --- Line setters (raw values) ---

    pub fn l1_raw(mut self, lw: i8, c: i8, rw: i8) -> Self {
        self.l1 = Some(Line::new(lw, c, rw));
        self
    }

    pub fn l2_raw(mut self, lw: i8, c: i8, rw: i8) -> Self {
        self.l2 = Some(Line::new(lw, c, rw));
        self
    }

    pub fn l3_raw(mut self, lw: i8, c: i8, rw: i8) -> Self {
        self.l3 = Some(Line::new(lw, c, rw));
        self
    }

    pub fn l4_raw(mut self, lw: i8, c: i8, rw: i8) -> Self {
        self.l4 = Some(Line::new(lw, c, rw));
        self
    }

    // --- Pairings ---

    pub fn d1(mut self, pairing: Pairing) -> Self {
        self.d1 = Some(pairing);
        self
    }

    pub fn d2(mut self, pairing: Pairing) -> Self {
        self.d2 = Some(pairing);
        self
    }

    pub fn d3(mut self, pairing: Pairing) -> Self {
        self.d3 = Some(pairing);
        self
    }

    pub fn d1_raw(mut self, ld: i8, rd: i8) -> Self {
        self.d1 = Some(Pairing::new(ld, rd));
        self
    }

    pub fn d2_raw(mut self, ld: i8, rd: i8) -> Self {
        self.d2 = Some(Pairing::new(ld, rd));
        self
    }

    pub fn d3_raw(mut self, ld: i8, rd: i8) -> Self {
        self.d3 = Some(Pairing::new(ld, rd));
        self
    }

    // --- Goalies ---

    pub fn goalies(mut self, goalies: GoalieTandem) -> Self {
        self.goalies = Some(goalies);
        self
    }

    pub fn goalies_raw(mut self, starter: i8, backup: i8, percent: i8) -> Self {
        self.goalies = Some(GoalieTandem::new(starter, backup, percent));
        self
    }

    // --- Build ---

    pub fn build(self) -> Loadout {
        Loadout {
            l1: self.l1.expect("l1 is required"),
            l2: self.l2.expect("l2 is required"),
            l3: self.l3.expect("l3 is required"),
            l4: self.l4.expect("l4 is required"),
            d1: self.d1.expect("d1 is required"),
            d2: self.d2.expect("d2 is required"),
            d3: self.d3.expect("d3 is required"),
            goalies: self.goalies.expect("goalies are required"),
        }
    }
}