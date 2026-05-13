use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Loadout {
    forward_lines: [[i8; 3]; 4],
    defense_pairs: [[i8; 2]; 3],
    goalies: [i8; 2],
    starter_share: i8,
}

impl Loadout {
    pub fn new(
        forward_lines: [[i8; 3]; 4],
        defense_pairs: [[i8; 2]; 3],
        goalies: [i8; 2],
        starter_share: i8,
    ) -> Loadout {
        Loadout {
            forward_lines,
            defense_pairs,
            goalies,
            starter_share,
        }
    }

    pub fn none() -> Loadout {
        Loadout::new([[0, 0, 0]; 4], [[0, 0]; 3], [0, 0], 1)
    }

    pub fn forward_lines(&self) -> &[[i8; 3]; 4] {
        &self.forward_lines
    }

    pub fn defense_pairs(&self) -> &[[i8; 2]; 3] {
        &self.defense_pairs
    }

    pub fn goalies(&self) -> &[i8; 2] {
        &self.goalies
    }

    pub fn starter_share(&self) -> i8 {
        self.starter_share
    }

    pub fn set_forward_line(&mut self, index: usize, line: [i8; 3]) {
        if let Some(slot) = self.forward_lines.get_mut(index) {
            *slot = line;
        }
    }

    pub fn set_defense_pair(&mut self, index: usize, pair: [i8; 2]) {
        if let Some(slot) = self.defense_pairs.get_mut(index) {
            *slot = pair;
        }
    }

    pub fn set_goalies(&mut self, goalies: [i8; 2]) {
        self.goalies = goalies;
    }

    pub fn set_starter_share(&mut self, starter_share: i8) {
        self.starter_share = starter_share;
    }
}
