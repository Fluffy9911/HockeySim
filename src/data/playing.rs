use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum PlayType {
    SMART,
    QUICK,
    SUPREME,
    SLOW,
}

#[derive(Serialize, Deserialize)]
pub struct GameView{
    scan:i8,
    predicting:i8,
    smart:i8,
    play:PlayType
}

impl GameView{

    pub fn new(scan:i8,predicting:i8,smart:i8,play_type: PlayType)->GameView{
        GameView {scan,predicting,smart,play: play_type}
    }

    pub fn scan(&self) -> i8 {
        self.scan
    }

    pub fn predicting(&self) -> i8 {
        self.predicting
    }

    pub fn smart(&self) -> i8 {
        self.smart
    }

    pub fn play(&self) -> &PlayType {
        &self.play
    }

}
