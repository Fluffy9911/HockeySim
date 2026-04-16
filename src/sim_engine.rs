use serde::{Deserialize, Serialize};
use crate::data::dates::GameDate;
use crate::league_settings::{League, SimulatedGame};
use crate::savestate;
use crate::savestate::savedata::{CoreConfig, FileType, SaveContext};

#[derive(Serialize,Deserialize)]
pub struct Sim{

    current_date: GameDate,
    leagues: Vec<League>

}

impl Sim {
    pub fn get_league(&mut self, p0: i32) -> &mut League {
         self.leagues.get_mut(p0 as usize).unwrap()
    }
}

#[derive(Serialize,Deserialize)]
pub struct Match{
    date:GameDate,
    t1:i64,
    t2:i64,
    score: Option<SimulatedGame>

}
#[derive(Serialize,Deserialize)]
pub struct Schedule{

    games: Vec<Match>



}

impl Match {
    pub fn new(date: GameDate, t1: i64, t2: i64) -> Self {
        Self {
            date,
            t1,
            t2,
            score: None,
        }
    }

    pub fn set_score(&mut self, score: SimulatedGame) {
        self.score = Some(score);
    }

    pub fn is_played(&self) -> bool {
        self.score.is_some()
    }
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            games: Vec::new(),
        }
    }

    pub fn add_match(&mut self, game: Match) {
        self.games.push(game);
    }

    pub fn total_games(&self) -> usize {
        self.games.len()
    }

    pub fn played_games(&self) -> usize {
        self.games.iter().filter(|g| g.score.is_some()).count()
    }

    pub fn unplayed_games(&self) -> usize {
        self.games.iter().filter(|g| g.score.is_none()).count()
    }
    pub fn get_games_for_date(&self, date: GameDate) -> Vec<&Match> {
        self.games
            .iter()
            .filter(|g| g.date == date)
            .collect()
    }

    pub fn schedule_game(&mut self, date: GameDate, t1: i64, t2: i64) {
        // prevent same team playing itself
        if t1 == t2 {
            return;
        }

        // prevent duplicate matchup on same date
        let exists = self.games.iter().any(|g| {
            g.date == date &&
                ((g.t1 == t1 && g.t2 == t2) || (g.t1 == t2 && g.t2 == t1))
        });

        if !exists {
            self.games.push(Match::new(date, t1, t2));
        }
    }



    pub fn schedule_season_games(
        start: &GameDate,
        end: &GameDate,
        no_games: Vec<&GameDate>,
        league: &League,
    ) -> Schedule {
        let mut schedule = Schedule::new();

        let team_count = league.team_registry().len();
        let max_per_day = team_count / 2;



        let mut current_date = start.clone();

        while current_date <= *end {
            // skip blocked dates
            if no_games.iter().any(|d| **d == current_date) {
                current_date = current_date.add_days(1);
                continue;
            }

            // simple pairing using indices
            for i in 0..max_per_day {
                let t1 = i as i64;
                let t2 = (team_count - 1 - i) as i64;

                schedule.schedule_game(current_date.clone(), t1, t2);
            }

            current_date = current_date.add_days(1);
        }

        schedule
    }

}


impl Sim {

    pub fn create_or_load(ctx:&SaveContext) -> Sim {

if ctx.exists(FileType::LEAGUE_DATA,"Sim.json"){

let sim = ctx.read_struct(FileType::LEAGUE_DATA,"Sim.json").unwrap();

    sim

}else {

    let sim = Sim{current_date: GameDate::new(2025,1,1).unwrap(),leagues: Vec::new()};

    ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(&sim).unwrap()).expect("Error");

    sim

}


    }


    pub fn add_league(&mut self,league:League){

        self.leagues.push(league);


    }

    pub fn advance_date(&mut self){
     self.current_date =   self.current_date.add_days(30);

    }

    pub fn save_data(&self,ctx:&SaveContext){
        ctx.write_file(FileType::LEAGUE_DATA, "Sim.json", &*serde_json::to_string_pretty(self).unwrap()).expect("Error");

    }



}
