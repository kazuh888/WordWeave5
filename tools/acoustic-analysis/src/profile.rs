//! Versioned measurement settings; a profile ID fixes every measurement setting.

use serde::{Deserialize, Serialize};

use crate::{AnalysisError, AnalysisErrorKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisProfile {
    pub schema_version: u32,
    pub profile_id: String,
    pub preprocessing: PreprocessProfile,
    pub energy: EnergyProfile,
    pub pitch: PitchProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessProfile {
    Pcm16MeanChannelsV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyProfile {
    pub window_ms: u32,
    pub hop_ms: u32,
    pub low_threshold_dbfs: f64,
    pub min_low_interval_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchProfile {
    pub floor_hz: f64,
    pub top_hz: f64,
    pub step_seconds: f64,
    pub max_candidates: u16,
    pub very_accurate: bool,
    pub attenuation_at_top: f64,
    pub silence_threshold: f64,
    pub voicing_threshold: f64,
    pub octave_cost: f64,
    pub octave_jump_cost: f64,
    pub voiced_unvoiced_cost: f64,
}

impl AnalysisProfile {
    pub fn v1() -> Self {
        Self {
            schema_version: 1,
            profile_id: "reading-v1".to_owned(),
            preprocessing: PreprocessProfile::Pcm16MeanChannelsV1,
            energy: EnergyProfile {
                window_ms: 25,
                hop_ms: 10,
                low_threshold_dbfs: -40.0,
                min_low_interval_ms: 200,
            },
            pitch: PitchProfile {
                floor_hz: 50.0,
                top_hz: 800.0,
                step_seconds: 0.010,
                max_candidates: 15,
                very_accurate: false,
                attenuation_at_top: 0.03,
                silence_threshold: 0.09,
                voicing_threshold: 0.50,
                octave_cost: 0.055,
                octave_jump_cost: 0.35,
                voiced_unvoiced_cost: 0.14,
            },
        }
    }

    pub fn validate(&self) -> Result<(), AnalysisError> {
        if *self != Self::v1() {
            return Err(AnalysisError {
                kind: AnalysisErrorKind::UnsupportedProfile,
            });
        }
        Ok(())
    }
}
