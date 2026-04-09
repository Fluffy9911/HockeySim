use serde::{Deserialize, Serialize};
use randoms::choices::*;
use crate::randoms;

#[derive(Serialize, Deserialize)]
pub enum SkatingType {
    QUICK,
    SLOW,
    STRONG,
    NIMBLE,
}

impl SkatingType {

    

}
#[derive(Serialize, Deserialize)]
pub struct SkatingStats {
    speed: i8,
    edges: i8,
    acceleration: i8,
    skate_type: SkatingType,
}

#[derive(Serialize, Deserialize)]
pub struct GoalieMovement {
    side: i8,
    up_down: i8,
    push: i8,
}

impl SkatingStats {
    pub fn new(speed: i8, edges: i8, acceleration: i8, skate_type: SkatingType) -> SkatingStats {
        SkatingStats {
            speed,
            edges,
            acceleration,
            skate_type,
        }
    }

    pub fn random(bias:f32,st:SkatingType) -> SkatingStats {


        Self::new(biased_random_range(0, 100, bias) as i8, biased_random_range(0, 100, bias) as i8, biased_random_range(0, 100, bias) as i8, st)
    }

    pub fn speed(&self) -> i8 {
        self.speed
    }

    pub fn edges(&self) -> i8 {
        self.edges
    }

    pub fn acceleration(&self) -> i8 {
        self.acceleration
    }

    pub fn skate_type(&self) -> &SkatingType {
        &self.skate_type
    }

    pub fn apply_delta(&mut self, speed_delta: i8, edges_delta: i8, acceleration_delta: i8, max_rating: i8) {
        self.speed = adjust_rating(self.speed, speed_delta, max_rating);
        self.edges = adjust_rating(self.edges, edges_delta, max_rating);
        self.acceleration = adjust_rating(self.acceleration, acceleration_delta, max_rating);
    }
}

impl GoalieMovement {
    pub fn new(side: i8, up_down: i8, push: i8) -> GoalieMovement {
        GoalieMovement { side, up_down, push }
    }

    pub fn side(&self) -> i8 {
        self.side
    }

    pub fn up_down(&self) -> i8 {
        self.up_down
    }

    pub fn push(&self) -> i8 {
        self.push
    }

    pub fn apply_delta(&mut self, side_delta: i8, up_down_delta: i8, push_delta: i8, max_rating: i8) {
        self.side = adjust_rating(self.side, side_delta, max_rating);
        self.up_down = adjust_rating(self.up_down, up_down_delta, max_rating);
        self.push = adjust_rating(self.push, push_delta, max_rating);
    }
}

fn adjust_rating(current: i8, delta: i8, max_rating: i8) -> i8 {
    (current + delta).clamp(1, max_rating)
}
