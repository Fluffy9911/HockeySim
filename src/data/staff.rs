use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum StaffRole {
    GENERAL_MANAGER,
    ASSISTANT_GENERAL_MANAGER,
    HEAD_COACH,
    ASSISTANT_COACH,
    DEVELOPMENT_COACH,
    HEAD_SCOUT,
    GOALIE_COACH,
    SKATING_COACH,
    SCOUT,
    DIRECTOR_OF_PLAYER_DEVELOPMENT,
    OWNER,
}

#[derive(Serialize, Deserialize)]
pub struct StaffRatings {
    teaching: i8,
    tactical: i8,
    evaluation: i8,
    leadership: i8,
}

#[derive(Serialize, Deserialize)]
pub struct StaffDevelopment {
    current_level: i8,
    potential: i8,
    growth_rate: i8,
    consistency: i8,
}

#[derive(Serialize, Deserialize)]
pub struct StaffMember {
    name: String,
    age: i8,
    role: StaffRole,
    ratings: StaffRatings,
    development: StaffDevelopment,
}

impl StaffRatings {
    pub fn new(teaching: i8, tactical: i8, evaluation: i8, leadership: i8) -> StaffRatings {
        StaffRatings { teaching, tactical, evaluation, leadership }
    }

    pub fn teaching(&self) -> i8 { self.teaching }
    pub fn tactical(&self) -> i8 { self.tactical }
    pub fn evaluation(&self) -> i8 { self.evaluation }
    pub fn leadership(&self) -> i8 { self.leadership }
}

impl StaffMember {
    pub fn new(name: String, age: i8, role: StaffRole, ratings: StaffRatings) -> StaffMember {
        StaffMember {
            name,
            age,
            role,
            ratings,
            development: StaffDevelopment::default(),
        }
    }

    pub fn new_with_development(
        name: String,
        age: i8,
        role: StaffRole,
        ratings: StaffRatings,
        development: StaffDevelopment,
    ) -> StaffMember {
        StaffMember { name, age, role, ratings, development }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn age(&self) -> i8 { self.age }
    pub fn role(&self) -> &StaffRole { &self.role }
    pub fn ratings(&self) -> &StaffRatings { &self.ratings }
    pub fn ratings_mut(&mut self) -> &mut StaffRatings { &mut self.ratings }
    pub fn development(&self) -> &StaffDevelopment { &self.development }

    pub fn age_one_year(&mut self) {
        self.age += 1;
    }

    pub fn develop(&mut self) {
        let room_to_grow = (self.development.potential - self.development.current_level).max(0);
        let growth = ((room_to_grow as i16 * self.development.growth_rate as i16) / 100).max(1) as i8;
        let stabilized_growth = ((growth as i16 * (50 + self.development.consistency as i16)) / 100) as i8;
        let applied = stabilized_growth.max(0);

        self.ratings.teaching = increase_rating(self.ratings.teaching, applied, self.development.potential);
        self.ratings.tactical = increase_rating(self.ratings.tactical, applied, self.development.potential);
        self.ratings.evaluation = increase_rating(self.ratings.evaluation, applied, self.development.potential);
        self.ratings.leadership = increase_rating(self.ratings.leadership, applied, self.development.potential);
        self.development.current_level = average_rating(&self.ratings);
    }
}

impl StaffDevelopment {
    pub fn new(current_level: i8, potential: i8, growth_rate: i8, consistency: i8) -> StaffDevelopment {
        StaffDevelopment { current_level, potential, growth_rate, consistency }
    }

    pub fn default() -> StaffDevelopment {
        StaffDevelopment { current_level: 55, potential: 75, growth_rate: 18, consistency: 60 }
    }

    pub fn current_level(&self) -> i8 { self.current_level }
    pub fn potential(&self) -> i8 { self.potential }
    pub fn growth_rate(&self) -> i8 { self.growth_rate }
    pub fn consistency(&self) -> i8 { self.consistency }
}

fn increase_rating(current: i8, growth: i8, max: i8) -> i8 {
    (current + growth).min(max)
}

fn average_rating(ratings: &StaffRatings) -> i8 {
    ((ratings.teaching as i16
        + ratings.tactical as i16
        + ratings.evaluation as i16
        + ratings.leadership as i16)
        / 4) as i8
}
