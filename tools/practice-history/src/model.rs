//! Versioned history documents. Every optional field still requires a JSON key.

use std::collections::BTreeMap;

use acoustic_analysis::AnalysisProfile;
use serde::{Deserialize, Serialize};

use crate::{BlobRef, OperationId, RunId, Sha256, TrialId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedAt {
    pub utc_millis: i64,
    pub offset_minutes: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    CapturedWav,
    Imported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Wav,
    Mp3,
    Aac,
    Amr,
    ThreeGp,
    ThreeGpp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureWarning {
    PartialRecording,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioSnapshot {
    pub original: BlobRef,
    pub format: AudioFormat,
    pub rate: u32,
    pub channels: u16,
    pub frames: u64,
    pub origin: Origin,
    pub warnings: Vec<CaptureWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceTag {
    pub host: String,
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSnapshot {
    pub text: String,
    pub sha256: Sha256,
    #[serde(deserialize_with = "Option::deserialize")]
    pub source: Option<SourceTag>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalSnapshot {
    pub text: String,
    pub version: Sha256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureProfile {
    #[serde(deserialize_with = "Option::deserialize")]
    pub device: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub distance: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub gain: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub environment: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub confirmed_at: Option<RecordedAt>,
    pub version: Sha256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenInput {
    pub audio: AudioSnapshot,
    pub reference: ReferenceSnapshot,
    #[serde(deserialize_with = "Option::deserialize")]
    pub goal: Option<GoalSnapshot>,
    pub capture: CaptureProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PracticeClaim {
    Reference,
    Practice {
        performed_at: RecordedAt,
        self_speaker_confirmed_at: RecordedAt,
        time_confirmed_at: RecordedAt,
        duplicate_confirmed: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum MatchEvidence {
    HumanConfirmed {
        at: RecordedAt,
        audio: Sha256,
        reference: Sha256,
    },
    AiAssessed {
        run: RunId,
    },
    AiMismatch {
        run: RunId,
    },
    AiUnassessable {
        run: RunId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialRevision {
    pub schema_version: u32,
    pub id: TrialId,
    pub revision: u64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub previous: Option<BlobRef>,
    pub created_at: RecordedAt,
    pub input: FrozenInput,
    pub claim: PracticeClaim,
    pub match_evidence: Vec<MatchEvidence>,
    pub analysis: BTreeMap<RunId, BlobRef>,
    pub advice: BTreeMap<RunId, BlobRef>,
    pub reflections: Vec<BlobRef>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub trashed_at: Option<RecordedAt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecoderIdentity {
    pub name: String,
    pub version: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub binary_sha256: Option<Sha256>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisRunState {
    Running,
    Complete,
    Partial,
    Failed,
    Cancelled,
    TimedOut,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceRunState {
    Sending,
    Assessed,
    Unassessable,
    ReferenceMismatch,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendCertainty {
    NotSent,
    MayHaveBeenSent,
    ResponseReceived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureDomain {
    Storage,
    Analysis,
    Advice,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureCode {
    pub domain: FailureDomain,
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRun {
    pub schema_version: u32,
    pub id: RunId,
    pub trial: TrialId,
    pub audio: Sha256,
    pub pcm: Sha256,
    pub decoder: DecoderIdentity,
    pub profile: AnalysisProfile,
    pub state: AnalysisRunState,
    pub started_at: RecordedAt,
    #[serde(deserialize_with = "Option::deserialize")]
    pub ended_at: Option<RecordedAt>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub result: Option<BlobRef>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub failure: Option<FailureCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceVersions {
    pub provider: String,
    pub prompt: String,
    pub parser: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceRun {
    pub schema_version: u32,
    pub id: RunId,
    pub trial: TrialId,
    pub audio: Sha256,
    pub reference: Sha256,
    pub confirmed_at: RecordedAt,
    pub confirmation: Sha256,
    pub destination: String,
    pub versions: AdviceVersions,
    pub state: AdviceRunState,
    pub send: SendCertainty,
    pub started_at: RecordedAt,
    #[serde(deserialize_with = "Option::deserialize")]
    pub ended_at: Option<RecordedAt>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub result: Option<BlobRef>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub failure: Option<FailureCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceAssessment {
    Assessed,
    ReferenceMismatch,
    Unassessable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceImprovement {
    pub reference_excerpt: String,
    pub observation: String,
    pub practice: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceUsage {
    #[serde(deserialize_with = "Option::deserialize")]
    pub prompt_tokens: Option<u64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub completion_tokens: Option<u64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdviceResult {
    pub schema_version: u32,
    pub assessment: AdviceAssessment,
    #[serde(deserialize_with = "Option::deserialize")]
    pub heard_text: Option<String>,
    pub summary: String,
    pub strengths: Vec<String>,
    pub improvements: Vec<AdviceImprovement>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub unassessable_reason: Option<String>,
    pub requested_model: String,
    pub requested_effort: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub actual_model: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub actual_effort: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub usage: Option<AdviceUsage>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reflection {
    pub schema_version: u32,
    pub trial: TrialId,
    pub at: RecordedAt,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRoot {
    pub schema_version: u32,
    pub revision: u64,
    pub next_save_sequence: u64,
    pub trials: BTreeMap<TrialId, TrialHead>,
    pub receipts: BTreeMap<OperationId, OperationReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialHead {
    pub document: BlobRef,
    pub admitted_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationReceipt {
    pub request: Sha256,
    pub committed_revision: u64,
    pub affected: Vec<TrialId>,
}
