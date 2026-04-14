use rand::RngExt;
use serde::{Deserialize, Serialize};
use crate::randoms::choices::biased_random_range;

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
#[derive(Serialize, Deserialize)]
pub struct Skills{
    shot_accuracy:i8,
    shot_power:i8,
    offense:i8,
    defense:i8,
    hands:i8,
    mentality:i8,
    face_off:i8,
    durability:i8,
    passing:i8,
    physicality:i8,
    fighting:i8,
    discipline:i8

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
    pub fn random() -> GameView {
        Self::new(
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            Self::random_play(),
        )
    }

    pub fn biased(bias: f32) -> GameView {
        Self::new(
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            Self::random_play(),
        )
    }

    fn random_play() -> PlayType {
        let mut r = rand::rng();
        match r.random_range(0..4) {
            0 => PlayType::SMART,
            1 => PlayType::QUICK,
            2 => PlayType::SUPREME,
            _ => PlayType::SLOW,
        }
    }
}

fn random_range_i8(p0: i32, p1: i32) -> i8 {
    rand::rng().random_range(p0..=p1) as i8
}

impl Skills {


    pub fn new(
        shot_accuracy: i8,
        shot_power: i8,
        offense: i8,
        defense: i8,
        hands: i8,
        mentality: i8,
        face_off: i8,
        durability: i8,
        passing: i8,
        physicality: i8,
        fighting: i8,
        discipline: i8,
    ) -> Skills {
        Skills {
            shot_accuracy,
            shot_power,
            offense,
            defense,
            hands,
            mentality,
            face_off,
            durability,
            passing,
            physicality,
            fighting,
            discipline,
        }
    }

    pub fn random() -> Skills {
        Self::new(
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
            random_range_i8(1, 100),
        )
    }

    pub fn biased(bias: f32) -> Skills {
        Self::new(
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
            biased_random_range(1, 100, bias) as i8,
        )
    }


    pub fn shot_accuracy(&self) -> i8 {
        self.shot_accuracy
    }

    pub fn shot_power(&self) -> i8 {
        self.shot_power
    }

    pub fn offense(&self) -> i8 {
        self.offense
    }

    pub fn defense(&self) -> i8 {
        self.defense
    }

    pub fn hands(&self) -> i8 {
        self.hands
    }

    pub fn mentality(&self) -> i8 {
        self.mentality
    }

    pub fn face_off(&self) -> i8 {
        self.face_off
    }

    pub fn durability(&self) -> i8 {
        self.durability
    }

    pub fn passing(&self) -> i8 {
        self.passing
    }

    pub fn physicality(&self) -> i8 {
        self.physicality
    }

    pub fn fighting(&self) -> i8 {
        self.fighting
    }

    pub fn discipline(&self) -> i8 {
        self.discipline
    }

    pub fn set_shot_accuracy(&mut self, shot_accuracy: i8) {
        self.shot_accuracy = shot_accuracy;
    }

    pub fn set_shot_power(&mut self, shot_power: i8) {
        self.shot_power = shot_power;
    }

    pub fn set_offense(&mut self, offense: i8) {
        self.offense = offense;
    }

    pub fn set_defense(&mut self, defense: i8) {
        self.defense = defense;
    }

    pub fn set_hands(&mut self, hands: i8) {
        self.hands = hands;
    }

    pub fn set_mentality(&mut self, mentality: i8) {
        self.mentality = mentality;
    }

    pub fn set_face_off(&mut self, face_off: i8) {
        self.face_off = face_off;
    }

    pub fn set_durability(&mut self, durability: i8) {
        self.durability = durability;
    }

    pub fn set_passing(&mut self, passing: i8) {
        self.passing = passing;
    }

    pub fn set_physicality(&mut self, physicality: i8) {
        self.physicality = physicality;
    }

    pub fn set_fighting(&mut self, fighting: i8) {
        self.fighting = fighting;
    }

    pub fn set_discipline(&mut self, discipline: i8) {
        self.discipline = discipline;
    }
}

