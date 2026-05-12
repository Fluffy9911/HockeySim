use std::ops::Index;
use crate::league_settings::League;
use crate::sim_engine::Sim;

impl Sim{

    pub fn ensure_league(&mut self, league:League) -> usize {
        if self.leagues.contains(&league) {
            self.leagues.iter().position(|x| *x == league).unwrap()
        } else {
            self.leagues.push(league);
            self.leagues.len() - 1
        }


    }

}

impl PartialEq for League {
    fn eq(&self, other: &Self) -> bool {

        self.name == other.name

    }
}