//! Bounded local acoustic measurements and versioned report contracts.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

mod measure;
#[cfg(feature = "praat")]
pub mod praat;
#[cfg(feature = "praat")]
mod praat_protocol;
#[cfg(all(test, feature = "praat"))]
#[path = "../tests/support/praat_unit.rs"]
mod praat_unit;
#[cfg(feature = "praat")]
mod process;
#[cfg(all(test, feature = "praat", windows))]
#[path = "../tests/support/process_unit.rs"]
mod process_unit;
pub mod profile;
mod validate;
pub use profile::{AnalysisProfile, EnergyProfile, PitchProfile, PreprocessProfile};

// A persisted Option is nullable, but its key is still required. Supplying a
// field deserializer prevents derive from silently defaulting a missing key.
fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PcmView<'a> {
    samples: &'a [i16],
    sample_rate: u32,
    channels: u16,
}

impl<'a> PcmView<'a> {
    pub fn new(samples: &'a [i16], sample_rate: u32, channels: u16) -> Result<Self, AnalysisError> {
        if samples.is_empty()
            || !(8000..=48000).contains(&sample_rate)
            || !(1..=2).contains(&channels)
            || !samples.len().is_multiple_of(usize::from(channels))
            || samples.len() / usize::from(channels) > sample_rate as usize * 60
        {
            return Err(AnalysisError {
                kind: AnalysisErrorKind::InvalidInput,
            });
        }
        Ok(Self {
            samples,
            sample_rate,
            channels,
        })
    }

    pub fn samples(&self) -> &'a [i16] {
        self.samples
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisRequest<'a> {
    pub pcm: PcmView<'a>,
    pub profile: &'a AnalysisProfile,
    pub limits: &'a AnalysisLimits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisLimits {
    pub wall_time: Duration,
    pub max_frames: usize,
    pub max_buffer_bytes: usize,
}

impl AnalysisLimits {
    pub fn v1() -> Self {
        Self {
            wall_time: Duration::from_secs(60),
            max_frames: 6001,
            max_buffer_bytes: 64 * 1024 * 1024,
        }
    }
}

pub fn analyze(
    request: AnalysisRequest<'_>,
    pitch: &dyn PitchBackend,
    cancel: &CancellationToken,
) -> Result<AnalysisReport, AnalysisError> {
    measure::analyze(request, pitch, cancel)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonoPcm<'a> {
    pub samples: &'a [f64],
    pub sample_rate: u32,
}

pub trait PitchBackend: Send + Sync {
    fn identity(&self) -> Option<EngineIdentity>;

    fn extract(
        &self,
        pcm: MonoPcm<'_>,
        profile: &PitchProfile,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError>;
}

pub struct MissingPitchBackend;

impl PitchBackend for MissingPitchBackend {
    fn identity(&self) -> Option<EngineIdentity> {
        None
    }

    fn extract(
        &self,
        _pcm: MonoPcm<'_>,
        _profile: &PitchProfile,
        _deadline: Instant,
        _cancel: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError> {
        Err(AnalysisError {
            kind: AnalysisErrorKind::BackendUnavailable,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchTrack {
    pub frames: Vec<TimedValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineIdentity {
    pub name: String,
    pub version: String,
    #[serde(deserialize_with = "required_option")]
    pub binary_sha256: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub script_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "value",
    rename_all = "snake_case"
)]
pub enum SampleValue {
    Finite(f64),
    DigitalSilence,
    Missing(MissingReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingReason {
    Unvoiced,
    TooShort,
    BackendUnavailable,
    BackendFailed,
    InvalidOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimedValue {
    pub time_seconds: f64,
    pub value: SampleValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeInterval {
    pub start_seconds: f64,
    pub end_seconds: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchMeasure {
    pub frames: Vec<TimedValue>,
    pub median_hz: SampleValue,
    pub p10_hz: SampleValue,
    pub p90_hz: SampleValue,
    pub valid_frame_fraction: SampleValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measures {
    pub duration_seconds: f64,
    pub rms_dbfs: SampleValue,
    pub rms_frames: Vec<TimedValue>,
    pub low_intervals: Vec<TimeInterval>,
    pub low_total_seconds: f64,
    pub low_fraction: f64,
    pub pitch: PitchMeasure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityKind {
    StereoMixed,
    ClipCandidate,
    NoVoicedFrames,
    ShortInput,
    BackendWarning,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quality {
    pub kind: QualityKind,
    #[serde(deserialize_with = "required_option")]
    pub fraction: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisReport {
    pub schema_version: u32,
    pub metrics_version: String,
    pub library_version: String,
    pub profile: AnalysisProfile,
    pub sample_rate: u32,
    pub channels: u16,
    pub input_frames: u64,
    #[serde(deserialize_with = "required_option")]
    pub pitch_engine: Option<EngineIdentity>,
    pub status: AnalysisStatus,
    pub measures: Measures,
    pub quality: Vec<Quality>,
}

impl AnalysisReport {
    pub fn validate(&self) -> Result<(), AnalysisError> {
        validate::report(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisErrorKind {
    InvalidInput,
    UnsupportedProfile,
    BackendUnavailable,
    BackendFailed,
    TimedOut,
    Cancelled,
    ResourceLimit,
    InvalidOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisError {
    pub kind: AnalysisErrorKind,
}
