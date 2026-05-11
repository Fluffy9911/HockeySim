use serde::{Serialize, Deserialize};


use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct GameDate {
    pub year: u32,
    pub month: u8, // 1-12
    pub day: u8,   // 1-30
}

#[derive(Debug)]
pub enum DateError {
    InvalidMonth,
    InvalidDay,
    ParseError,
}

impl GameDate {
    // =========================
    // CALENDAR CONSTANTS
    // =========================
    pub const DAYS_PER_MONTH: u32 = 30;
    pub const MONTHS_PER_YEAR: u32 = 12;
    pub const DAYS_PER_YEAR: u32 = Self::DAYS_PER_MONTH * Self::MONTHS_PER_YEAR;

    // =========================
    // CONSTRUCTOR
    // =========================
    pub fn new(year: u32, month: u8, day: u8) -> Result<Self, DateError> {
        if month == 0 || month > 12 {
            return Err(DateError::InvalidMonth);
        }
        if day == 0 || day > 30 {
            return Err(DateError::InvalidDay);
        }

        Ok(Self { year, month, day })
    }

    // =========================
    // INDEX CONVERSION
    // =========================
    pub fn to_day_index(&self) -> u32 {
        self.year * Self::DAYS_PER_YEAR
            + (self.month as u32 - 1) * Self::DAYS_PER_MONTH
            + (self.day as u32 - 1)
    }

    pub fn from_day_index(mut days: u32) -> Self {
        let year = days / Self::DAYS_PER_YEAR;
        days %= Self::DAYS_PER_YEAR;

        let month = (days / Self::DAYS_PER_MONTH) + 1;
        let day = (days % Self::DAYS_PER_MONTH) + 1;

        Self {
            year,
            month: month as u8,
            day: day as u8,
        }
    }

    // =========================
    // WEEKDAY (0 = MONDAY)
    // =========================
    pub fn weekday(&self) -> u8 {
        (self.to_day_index() % 7) as u8
    }

    pub fn weekday_name(&self) -> &'static str {
        match self.weekday() {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            5 => "Saturday",
            6 => "Sunday",
            _ => unreachable!(),
        }
    }

    // =========================
    // MONTH NAMES
    // =========================
    pub fn month_name(&self) -> &'static str {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr",
            "May", "Jun", "Jul", "Aug",
            "Sep", "Oct", "Nov", "Dec",
        ];
        MONTHS[(self.month - 1) as usize]
    }

    // =========================
    // DATE MATH
    // =========================
    pub fn add_days(&self, days: i32) -> Self {
        let idx = self.to_day_index() as i32 + days;
        Self::from_day_index(idx.max(0) as u32)
    }

    pub fn days_between(&self, other: Self) -> i32 {
        other.to_day_index() as i32 - self.to_day_index() as i32
    }

    // =========================
    // HELPERS
    // =========================
    pub fn start_of_year(year: u32) -> Self {
        Self { year, month: 1, day: 1 }
    }

    pub fn end_of_year(year: u32) -> Self {
        Self { year, month: 12, day: 30 }
    }

    pub fn is_same_day(&self, other: &Self) -> bool {
        self.year == other.year &&
            self.month == other.month &&
            self.day == other.day
    }
}


impl fmt::Display for GameDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}, Year {} ({})",
            self.month_name(),
            self.day,
            self.year,
            self.weekday_name()
        )
    }
}


impl FromStr for GameDate {
    type Err = DateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(DateError::ParseError);
        }

        let year = parts[0].parse().map_err(|_| DateError::ParseError)?;
        let month = parts[1].parse().map_err(|_| DateError::ParseError)?;
        let day = parts[2].parse().map_err(|_| DateError::ParseError)?;

        GameDate::new(year, month, day)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<u8> for Weekday {
    fn from(v: u8) -> Self {
        match v {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            6 => Weekday::Sunday,
            _ => unreachable!(),
        }
    }
}

impl GameDate {
    pub fn weekday_enum(&self) -> Weekday {
        Weekday::from(self.weekday())
    }
}
