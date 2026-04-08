use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ContractType {
    ENTRY_LEVEL,
    STANDARD,
    BRIDGE,
    EXTENSION,
    TWO_WAY,
    ONE_WAY,
    PROFESSIONAL_TRYOUT,
}

#[derive(Serialize, Deserialize)]
pub struct ContractLimits {
    min_years: i8,
    max_years: i8,
    min_cap_hit_millions: f32,
    max_cap_hit_millions: f32,
    min_salary_millions: f32,
    max_salary_millions: f32,
    min_signing_bonus_millions: f32,
    max_signing_bonus_millions: f32,
    min_performance_bonus_millions: f32,
    max_performance_bonus_millions: f32,
    min_no_trade_clauses: i8,
    max_no_trade_clauses: i8,
    min_no_move_clauses: i8,
    max_no_move_clauses: i8,
}

#[derive(Serialize, Deserialize)]
pub struct Contract {
    contract_type: ContractType,
    years: i8,
    cap_hit_millions: f32,
    salary_millions: f32,
    signing_bonus_millions: f32,
    performance_bonus_millions: f32,
    no_trade_clauses: i8,
    no_move_clauses: i8,
}

#[derive(Serialize, Deserialize)]
pub struct TeamContractSettings {
    salary_cap_max_millions: f32,
    salary_floor_min_millions: f32,
    max_contracts: i16,
    max_retained_salary_slots: i8,
    limits: ContractLimits,
}

impl ContractLimits {
    pub fn new(
        min_years: i8,
        max_years: i8,
        min_cap_hit_millions: f32,
        max_cap_hit_millions: f32,
        min_salary_millions: f32,
        max_salary_millions: f32,
        min_signing_bonus_millions: f32,
        max_signing_bonus_millions: f32,
        min_performance_bonus_millions: f32,
        max_performance_bonus_millions: f32,
        min_no_trade_clauses: i8,
        max_no_trade_clauses: i8,
        min_no_move_clauses: i8,
        max_no_move_clauses: i8,
    ) -> ContractLimits {
        ContractLimits {
            min_years,
            max_years,
            min_cap_hit_millions,
            max_cap_hit_millions,
            min_salary_millions,
            max_salary_millions,
            min_signing_bonus_millions,
            max_signing_bonus_millions,
            min_performance_bonus_millions,
            max_performance_bonus_millions,
            min_no_trade_clauses,
            max_no_trade_clauses,
            min_no_move_clauses,
            max_no_move_clauses,
        }
    }

    pub fn nhl_default() -> ContractLimits {
        ContractLimits::new(1, 8, 0.775, 18.0, 0.775, 16.0, 0.0, 15.0, 0.0, 5.0, 0, 1, 0, 1)
    }

    pub fn validate(&self, contract: &Contract) -> Result<(), String> {
        validate_range_i8(contract.years, self.min_years, self.max_years, "years")?;
        validate_range_f32(contract.cap_hit_millions, self.min_cap_hit_millions, self.max_cap_hit_millions, "cap_hit_millions")?;
        validate_range_f32(contract.salary_millions, self.min_salary_millions, self.max_salary_millions, "salary_millions")?;
        validate_range_f32(contract.signing_bonus_millions, self.min_signing_bonus_millions, self.max_signing_bonus_millions, "signing_bonus_millions")?;
        validate_range_f32(contract.performance_bonus_millions, self.min_performance_bonus_millions, self.max_performance_bonus_millions, "performance_bonus_millions")?;
        validate_range_i8(contract.no_trade_clauses, self.min_no_trade_clauses, self.max_no_trade_clauses, "no_trade_clauses")?;
        validate_range_i8(contract.no_move_clauses, self.min_no_move_clauses, self.max_no_move_clauses, "no_move_clauses")?;
        Ok(())
    }

    pub fn min_years(&self) -> i8 { self.min_years }
    pub fn max_years(&self) -> i8 { self.max_years }
    pub fn min_cap_hit_millions(&self) -> f32 { self.min_cap_hit_millions }
    pub fn max_cap_hit_millions(&self) -> f32 { self.max_cap_hit_millions }
    pub fn min_salary_millions(&self) -> f32 { self.min_salary_millions }
    pub fn max_salary_millions(&self) -> f32 { self.max_salary_millions }
    pub fn min_signing_bonus_millions(&self) -> f32 { self.min_signing_bonus_millions }
    pub fn max_signing_bonus_millions(&self) -> f32 { self.max_signing_bonus_millions }
    pub fn min_performance_bonus_millions(&self) -> f32 { self.min_performance_bonus_millions }
    pub fn max_performance_bonus_millions(&self) -> f32 { self.max_performance_bonus_millions }
    pub fn min_no_trade_clauses(&self) -> i8 { self.min_no_trade_clauses }
    pub fn max_no_trade_clauses(&self) -> i8 { self.max_no_trade_clauses }
    pub fn min_no_move_clauses(&self) -> i8 { self.min_no_move_clauses }
    pub fn max_no_move_clauses(&self) -> i8 { self.max_no_move_clauses }
}

impl Contract {
    pub fn new(
        contract_type: ContractType,
        years: i8,
        cap_hit_millions: f32,
        salary_millions: f32,
        signing_bonus_millions: f32,
        performance_bonus_millions: f32,
        no_trade_clauses: i8,
        no_move_clauses: i8,
    ) -> Contract {
        Contract {
            contract_type,
            years,
            cap_hit_millions,
            salary_millions,
            signing_bonus_millions,
            performance_bonus_millions,
            no_trade_clauses,
            no_move_clauses,
        }
    }

    pub fn validate(&self, limits: &ContractLimits) -> Result<(), String> {
        limits.validate(self)
    }

    pub fn contract_type(&self) -> &ContractType { &self.contract_type }
    pub fn years(&self) -> i8 { self.years }
    pub fn cap_hit_millions(&self) -> f32 { self.cap_hit_millions }
    pub fn salary_millions(&self) -> f32 { self.salary_millions }
    pub fn signing_bonus_millions(&self) -> f32 { self.signing_bonus_millions }
    pub fn performance_bonus_millions(&self) -> f32 { self.performance_bonus_millions }
    pub fn no_trade_clauses(&self) -> i8 { self.no_trade_clauses }
    pub fn no_move_clauses(&self) -> i8 { self.no_move_clauses }
}

impl TeamContractSettings {
    pub fn new(
        salary_cap_max_millions: f32,
        salary_floor_min_millions: f32,
        max_contracts: i16,
        max_retained_salary_slots: i8,
        limits: ContractLimits,
    ) -> TeamContractSettings {
        TeamContractSettings {
            salary_cap_max_millions,
            salary_floor_min_millions,
            max_contracts,
            max_retained_salary_slots,
            limits,
        }
    }

    pub fn nhl_default() -> TeamContractSettings {
        TeamContractSettings::new(88.0, 65.0, 50, 3, ContractLimits::nhl_default())
    }

    pub fn salary_cap_max_millions(&self) -> f32 { self.salary_cap_max_millions }
    pub fn salary_floor_min_millions(&self) -> f32 { self.salary_floor_min_millions }
    pub fn max_contracts(&self) -> i16 { self.max_contracts }
    pub fn max_retained_salary_slots(&self) -> i8 { self.max_retained_salary_slots }
    pub fn limits(&self) -> &ContractLimits { &self.limits }
}

fn validate_range_i8(value: i8, min: i8, max: i8, field: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{field} out of range: {value} not in [{min}, {max}]"))
    } else {
        Ok(())
    }
}

fn validate_range_f32(value: f32, min: f32, max: f32, field: &str) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{field} out of range: {value} not in [{min}, {max}]"))
    } else {
        Ok(())
    }
}
