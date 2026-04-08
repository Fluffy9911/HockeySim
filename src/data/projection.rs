use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub enum ProjMax {
    MINOR,
    MINOR_TOP,
    BOTTOM_6,
    MID_6,
    TOP4,
    TOP2,
    TOP1,
    FRANCHISE,
    SUPERSTAR,
    ELITE,
    GENERATIONAL,
}

#[derive(Serialize, Deserialize)]
pub enum DevelopmentCurve {
    EARLY,
    LINEAR,
    LATE,
    BOOM_BUST,
}

#[derive(Serialize, Deserialize)]
pub struct DraftProjection {
    draft_round_grade: i8,
    overall_pick_estimate: i16,
    projection_confidence: i8,
    scouting_visibility: i8,
    max_projection: ProjMax,
}

#[derive(Serialize, Deserialize)]
pub struct DevelopmentProfile {
    ceiling: i8,
    floor: i8,
    growth_rate: i8,
    consistency: i8,
    coachability: i8,
    work_ethic: i8,
    injury_risk: i8,
    growth_window_start: i8,
    growth_window_end: i8,
    curve: DevelopmentCurve,
}

#[derive(Serialize, Deserialize)]
pub struct Projection {
    draft_projection: DraftProjection,
    development_profile: DevelopmentProfile,
}

pub struct ProjectionGenerationSettings {
    actualization: f32,
    certainty: f32,
    visibility: f32,
    volatility: f32,
    injury_risk: f32,
    coachability: f32,
    work_ethic: f32,
}

impl DraftProjection {
    pub fn new(
        draft_round_grade: i8,
        overall_pick_estimate: i16,
        projection_confidence: i8,
        scouting_visibility: i8,
        max_projection: ProjMax,
    ) -> DraftProjection {
        DraftProjection {
            draft_round_grade,
            overall_pick_estimate,
            projection_confidence,
            scouting_visibility,
            max_projection,
        }
    }

    pub fn draft_round_grade(&self) -> i8 {
        self.draft_round_grade
    }

    pub fn overall_pick_estimate(&self) -> i16 {
        self.overall_pick_estimate
    }

    pub fn projection_confidence(&self) -> i8 {
        self.projection_confidence
    }

    pub fn scouting_visibility(&self) -> i8 {
        self.scouting_visibility
    }

    pub fn max_projection(&self) -> &ProjMax {
        &self.max_projection
    }
}

impl DevelopmentProfile {
    pub fn new(
        ceiling: i8,
        floor: i8,
        growth_rate: i8,
        consistency: i8,
        coachability: i8,
        work_ethic: i8,
        injury_risk: i8,
        growth_window_start: i8,
        growth_window_end: i8,
        curve: DevelopmentCurve,
    ) -> DevelopmentProfile {
        DevelopmentProfile {
            ceiling,
            floor,
            growth_rate,
            consistency,
            coachability,
            work_ethic,
            injury_risk,
            growth_window_start,
            growth_window_end,
            curve,
        }
    }

    pub fn ceiling(&self) -> i8 { self.ceiling }
    pub fn floor(&self) -> i8 { self.floor }
    pub fn growth_rate(&self) -> i8 { self.growth_rate }
    pub fn consistency(&self) -> i8 { self.consistency }
    pub fn coachability(&self) -> i8 { self.coachability }
    pub fn work_ethic(&self) -> i8 { self.work_ethic }
    pub fn injury_risk(&self) -> i8 { self.injury_risk }
    pub fn growth_window_start(&self) -> i8 { self.growth_window_start }
    pub fn growth_window_end(&self) -> i8 { self.growth_window_end }
    pub fn curve(&self) -> &DevelopmentCurve { &self.curve }
}

impl Projection {
    pub fn new(
        draft_projection: DraftProjection,
        development_profile: DevelopmentProfile,
    ) -> Projection {
        Projection { draft_projection, development_profile }
    }

    pub fn random() -> Projection {
        let mut rng = ProjectionRng::from_time();
        let quality = rng.next_f32();
        let settings = ProjectionGenerationSettings::new(
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
        );
        Projection::from_quality_with_settings(quality, settings)
    }

    pub fn from_quality(quality: f32) -> Projection {
        Projection::from_quality_with_settings(quality, ProjectionGenerationSettings::default_balanced())
    }

    pub fn from_quality_with_settings(
        quality: f32,
        settings: ProjectionGenerationSettings,
    ) -> Projection {
        let normalized_quality = clamp_unit(quality);
        let actualization = settings.actualization();
        let certainty = settings.certainty();
        let visibility = settings.visibility();
        let volatility = settings.volatility();
        let injury_risk = settings.injury_risk();
        let coachability = settings.coachability();
        let work_ethic = settings.work_ethic();

        let max_projection = ProjMax::from_quality(normalized_quality);
        let draft_round_grade = scale_to_i8(1.0 - normalized_quality, 1, 7);
        let overall_pick_estimate = scale_to_i16((1.0 - normalized_quality).powf(1.35), 1, 224);
        let projection_confidence =
            scale_to_i8(0.35 + 0.35 * certainty + 0.30 * normalized_quality, 35, 99);
        let scouting_visibility = scale_to_i8(visibility, 20, 99);

        let ceiling = scale_to_i8(normalized_quality, 25, 99);
        let realized_band = 0.15 + (0.75 * actualization);
        let floor_target = (ceiling as f32) * realized_band;
        let floor = floor_target.round().clamp(10.0, ceiling as f32) as i8;
        let growth_rate =
            scale_to_i8((normalized_quality * 0.55) + (actualization * 0.45), 15, 95);
        let consistency = scale_to_i8((1.0 - volatility) * 0.6 + actualization * 0.4, 20, 99);
        let injury_value = scale_to_i8(injury_risk, 1, 99);
        let coachability_value = scale_to_i8(coachability, 1, 99);
        let work_ethic_value = scale_to_i8(work_ethic, 1, 99);
        let curve = DevelopmentCurve::from_profile(normalized_quality, actualization, volatility);
        let (growth_window_start, growth_window_end) = growth_window_for_curve(&curve);

        let draft_projection = DraftProjection::new(
            draft_round_grade,
            overall_pick_estimate,
            projection_confidence,
            scouting_visibility,
            max_projection,
        );

        let development_profile = DevelopmentProfile::new(
            ceiling,
            floor,
            growth_rate,
            consistency,
            coachability_value,
            work_ethic_value,
            injury_value,
            growth_window_start,
            growth_window_end,
            curve,
        );

        Projection { draft_projection, development_profile }
    }

    pub fn draft_projection(&self) -> &DraftProjection {
        &self.draft_projection
    }

    pub fn development_profile(&self) -> &DevelopmentProfile {
        &self.development_profile
    }
}

impl ProjectionGenerationSettings {
    pub fn new(
        actualization: f32,
        certainty: f32,
        visibility: f32,
        volatility: f32,
        injury_risk: f32,
        coachability: f32,
        work_ethic: f32,
    ) -> ProjectionGenerationSettings {
        ProjectionGenerationSettings {
            actualization: clamp_unit(actualization),
            certainty: clamp_unit(certainty),
            visibility: clamp_unit(visibility),
            volatility: clamp_unit(volatility),
            injury_risk: clamp_unit(injury_risk),
            coachability: clamp_unit(coachability),
            work_ethic: clamp_unit(work_ethic),
        }
    }

    pub fn default_balanced() -> ProjectionGenerationSettings {
        ProjectionGenerationSettings {
            actualization: 0.55,
            certainty: 0.5,
            visibility: 0.5,
            volatility: 0.35,
            injury_risk: 0.3,
            coachability: 0.55,
            work_ethic: 0.55,
        }
    }

    pub fn actualization(&self) -> f32 { self.actualization }
    pub fn certainty(&self) -> f32 { self.certainty }
    pub fn visibility(&self) -> f32 { self.visibility }
    pub fn volatility(&self) -> f32 { self.volatility }
    pub fn injury_risk(&self) -> f32 { self.injury_risk }
    pub fn coachability(&self) -> f32 { self.coachability }
    pub fn work_ethic(&self) -> f32 { self.work_ethic }
}

impl ProjMax {
    fn from_quality(quality: f32) -> ProjMax {
        match (clamp_unit(quality) * 11.0).floor() as i32 {
            0 => ProjMax::MINOR,
            1 => ProjMax::MINOR_TOP,
            2 => ProjMax::BOTTOM_6,
            3 => ProjMax::MID_6,
            4 => ProjMax::TOP4,
            5 => ProjMax::TOP2,
            6 => ProjMax::TOP1,
            7 => ProjMax::FRANCHISE,
            8 => ProjMax::SUPERSTAR,
            9 => ProjMax::ELITE,
            _ => ProjMax::GENERATIONAL,
        }
    }
}

impl DevelopmentCurve {
    fn from_profile(quality: f32, actualization: f32, volatility: f32) -> DevelopmentCurve {
        if volatility >= 0.75 {
            DevelopmentCurve::BOOM_BUST
        } else if quality >= 0.72 && actualization >= 0.6 {
            DevelopmentCurve::EARLY
        } else if quality <= 0.35 && actualization <= 0.45 {
            DevelopmentCurve::LATE
        } else {
            DevelopmentCurve::LINEAR
        }
    }
}

fn growth_window_for_curve(curve: &DevelopmentCurve) -> (i8, i8) {
    match curve {
        DevelopmentCurve::EARLY => (18, 23),
        DevelopmentCurve::LINEAR => (18, 27),
        DevelopmentCurve::LATE => (20, 29),
        DevelopmentCurve::BOOM_BUST => (18, 26),
    }
}

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn scale_to_i8(value: f32, min: i8, max: i8) -> i8 {
    let scaled = min as f32 + (clamp_unit(value) * (max - min) as f32);
    scaled.round() as i8
}

fn scale_to_i16(value: f32, min: i16, max: i16) -> i16 {
    let scaled = min as f32 + (clamp_unit(value) * (max - min) as f32);
    scaled.round() as i16
}

struct ProjectionRng {
    state: u64,
}

impl ProjectionRng {
    fn from_time() -> ProjectionRng {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        ProjectionRng { state: seed | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}
