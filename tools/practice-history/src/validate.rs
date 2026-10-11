//! Pure structural validation. Hosts must also verify referenced bytes and hashes.

use std::collections::BTreeSet;
use std::io::{self, Write};

use serde::Serialize;

use crate::ids::validate_bytes;
use crate::*;

type Check = Result<(), ContractError>;

fn error(code: ContractErrorCode, field: &'static str) -> ContractError {
    ContractError { code, field }
}

fn require(condition: bool, field: &'static str) -> Check {
    if condition {
        Ok(())
    } else {
        Err(error(ContractErrorCode::InvalidValue, field))
    }
}

fn schema(version: u32) -> Check {
    if version == 1 {
        Ok(())
    } else {
        Err(error(ContractErrorCode::UnknownVersion, "schema_version"))
    }
}

fn text(value: &str, maximum: usize, field: &'static str) -> Check {
    require(!value.trim().is_empty(), field)?;
    if value.chars().take(maximum.saturating_add(1)).count() > maximum {
        Err(error(ContractErrorCode::LimitExceeded, field))
    } else {
        Ok(())
    }
}

fn metadata(value: &str, field: &'static str) -> Check {
    text(value, 128, field)?;
    require(!value.chars().any(char::is_control), field)
}

impl RecordedAt {
    pub fn validate(&self) -> Check {
        require(
            (-62_135_596_800_000..=253_402_300_799_999).contains(&self.utc_millis),
            "recorded_at.utc_millis",
        )?;
        require(
            (-1439..=1439).contains(&self.offset_minutes),
            "recorded_at.offset_minutes",
        )
    }
}

impl AudioSnapshot {
    pub fn validate(&self) -> Check {
        self.original.validate()?;
        validate_bytes(self.original.bytes, 6 * 1024 * 1024, "audio.original.bytes")?;
        require((8_000..=48_000).contains(&self.rate), "audio.rate")?;
        require(matches!(self.channels, 1 | 2), "audio.channels")?;
        require(self.frames > 0, "audio.frames")?;
        if self.frames > u64::from(self.rate) * 60 {
            return Err(error(ContractErrorCode::LimitExceeded, "audio.frames"));
        }
        Ok(())
    }
}

impl SourceTag {
    pub fn validate(&self) -> Check {
        // These are opaque host identifiers, not paths or a shared ID syntax.
        text(&self.host, usize::MAX, "source.host")?;
        text(&self.id, usize::MAX, "source.id")?;
        text(&self.version, usize::MAX, "source.version")
    }
}

impl ReferenceSnapshot {
    pub fn validate(&self) -> Check {
        text(&self.text, 10_000, "reference.text")?;
        self.sha256.validate()?;
        if let Some(source) = &self.source {
            source.validate()?;
        }
        Ok(())
    }
}

impl GoalSnapshot {
    pub fn validate(&self) -> Check {
        text(&self.text, 1_000, "goal.text")?;
        self.version.validate()
    }
}

impl CaptureProfile {
    pub fn validate(&self) -> Check {
        for (value, field) in [
            (&self.device, "capture.device"),
            (&self.distance, "capture.distance"),
            (&self.gain, "capture.gain"),
            (&self.environment, "capture.environment"),
        ] {
            if let Some(value) = value {
                text(value, usize::MAX, field)?;
            }
        }
        if let Some(at) = &self.confirmed_at {
            at.validate()?;
        }
        self.version.validate()
    }
}

impl FrozenInput {
    pub fn validate(&self) -> Check {
        self.audio.validate()?;
        self.reference.validate()?;
        if let Some(goal) = &self.goal {
            goal.validate()?;
        }
        self.capture.validate()
    }
}

impl PracticeClaim {
    pub fn validate(&self) -> Check {
        if let Self::Practice {
            performed_at,
            self_speaker_confirmed_at,
            time_confirmed_at,
            ..
        } = self
        {
            performed_at.validate()?;
            self_speaker_confirmed_at.validate()?;
            time_confirmed_at.validate()?;
        }
        Ok(())
    }
}

impl MatchEvidence {
    pub fn validate(&self) -> Check {
        match self {
            Self::HumanConfirmed {
                at,
                audio,
                reference,
            } => {
                at.validate()?;
                audio.validate()?;
                reference.validate()
            }
            Self::AiAssessed { run } | Self::AiMismatch { run } | Self::AiUnassessable { run } => {
                run.validate()
            }
        }
    }
}

impl DecoderIdentity {
    pub fn validate(&self) -> Check {
        text(&self.name, usize::MAX, "decoder.name")?;
        text(&self.version, usize::MAX, "decoder.version")?;
        if let Some(hash) = &self.binary_sha256 {
            hash.validate()?;
        }
        Ok(())
    }
}

impl FailureCode {
    pub fn validate(&self) -> Check {
        text(&self.code, 64, "failure.code")?;
        require(
            self.code
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
            "failure.code",
        )
    }
}

impl AdviceVersions {
    pub fn validate(&self) -> Check {
        text(&self.provider, usize::MAX, "versions.provider")?;
        text(&self.prompt, usize::MAX, "versions.prompt")?;
        text(&self.parser, usize::MAX, "versions.parser")
    }
}

impl AdviceImprovement {
    pub fn validate(&self) -> Check {
        text(
            &self.reference_excerpt,
            1_000,
            "improvement.reference_excerpt",
        )?;
        text(&self.observation, 1_000, "improvement.observation")?;
        text(&self.practice, 1_000, "improvement.practice")
    }
}

impl AdviceUsage {
    pub fn validate(&self) -> Check {
        // u64 enforces nonnegative counters. Missing counters stay unknown, and
        // provider-reported totals are not reconstructed from partial metadata.
        Ok(())
    }
}

impl Reflection {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        self.trial.validate()?;
        self.at.validate()?;
        text(&self.text, 2_000, "reflection.text")
    }
}

impl TrialHead {
    pub fn validate(&self) -> Check {
        self.document.validate()?;
        require(self.admitted_sequence > 0, "trial_head.admitted_sequence")
    }
}

impl OperationReceipt {
    pub fn validate(&self) -> Check {
        self.request.validate()?;
        require(self.committed_revision > 0, "receipt.committed_revision")?;
        for id in &self.affected {
            id.validate()?;
        }
        Ok(())
    }
}

/// Enforces the minimum encoded document size without allocating JSON bytes.
/// Hosts still bound the actual input bytes before serde decoding.
struct JsonBudget(usize);

impl Write for JsonBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.0 {
            return Err(io::Error::other("document size limit"));
        }
        self.0 -= bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn document_size(value: &impl Serialize, maximum: usize, field: &'static str) -> Check {
    serde_json::to_writer(JsonBudget(maximum), value)
        .map_err(|_| error(ContractErrorCode::LimitExceeded, field))
}

impl TrialRevision {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        self.id.validate()?;
        require(self.revision > 0, "trial.revision")?;
        require(
            (self.revision == 1) == self.previous.is_none(),
            "trial.previous",
        )?;
        if let Some(previous) = &self.previous {
            previous.validate()?;
        }
        self.created_at.validate()?;
        self.input.validate()?;
        self.claim.validate()?;
        if let Some(at) = &self.trashed_at {
            at.validate()?;
        }
        for (id, reference) in self.analysis.iter().chain(self.advice.iter()) {
            id.validate()?;
            reference.validate()?;
        }
        for reference in &self.reflections {
            reference.validate()?;
        }
        for evidence in &self.match_evidence {
            evidence.validate()?;
            let matches_input = match evidence {
                MatchEvidence::HumanConfirmed {
                    audio, reference, ..
                } => {
                    audio == &self.input.audio.original.sha256
                        && reference == &self.input.reference.sha256
                }
                MatchEvidence::AiAssessed { run }
                | MatchEvidence::AiMismatch { run }
                | MatchEvidence::AiUnassessable { run } => self.advice.contains_key(run),
            };
            if !matches_input {
                return Err(error(
                    ContractErrorCode::ReferenceMismatch,
                    "trial.match_evidence",
                ));
            }
        }
        document_size(self, 256 * 1024, "trial.bytes")
    }
}

enum RunPhase {
    Active,
    Succeeded,
    Failed,
}

fn run_state(
    phase: RunPhase,
    started_at: &RecordedAt,
    ended_at: &Option<RecordedAt>,
    result: &Option<BlobRef>,
    failure: &Option<FailureCode>,
) -> Check {
    started_at.validate()?;
    if let Some(at) = ended_at {
        at.validate()?;
    }
    if let Some(reference) = result {
        reference.validate()?;
    }
    if let Some(code) = failure {
        code.validate()?;
    }
    let valid = match phase {
        RunPhase::Active => ended_at.is_none() && result.is_none() && failure.is_none(),
        RunPhase::Succeeded => ended_at.is_some() && result.is_some() && failure.is_none(),
        RunPhase::Failed => ended_at.is_some() && result.is_none() && failure.is_some(),
    };
    require(valid, "run.state")
}

impl AnalysisRun {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        self.id.validate()?;
        self.trial.validate()?;
        self.audio.validate()?;
        self.pcm.validate()?;
        self.decoder.validate()?;
        self.profile
            .validate()
            .map_err(|_| error(ContractErrorCode::InvalidValue, "analysis.profile"))?;
        let phase = match self.state {
            AnalysisRunState::Running => RunPhase::Active,
            AnalysisRunState::Complete | AnalysisRunState::Partial => RunPhase::Succeeded,
            AnalysisRunState::Failed
            | AnalysisRunState::Cancelled
            | AnalysisRunState::TimedOut
            | AnalysisRunState::Interrupted => RunPhase::Failed,
        };
        run_state(
            phase,
            &self.started_at,
            &self.ended_at,
            &self.result,
            &self.failure,
        )
    }
}

impl AdviceRun {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        self.id.validate()?;
        self.trial.validate()?;
        self.audio.validate()?;
        self.reference.validate()?;
        self.confirmed_at.validate()?;
        self.confirmation.validate()?;
        text(&self.destination, usize::MAX, "advice.destination")?;
        self.versions.validate()?;
        let phase = match self.state {
            AdviceRunState::Sending => {
                require(self.send != SendCertainty::ResponseReceived, "advice.send")?;
                RunPhase::Active
            }
            AdviceRunState::Assessed
            | AdviceRunState::Unassessable
            | AdviceRunState::ReferenceMismatch => {
                require(self.send == SendCertainty::ResponseReceived, "advice.send")?;
                RunPhase::Succeeded
            }
            AdviceRunState::Failed | AdviceRunState::Interrupted => RunPhase::Failed,
        };
        run_state(
            phase,
            &self.started_at,
            &self.ended_at,
            &self.result,
            &self.failure,
        )
    }
}

impl AdviceResult {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        text(&self.summary, 2_000, "advice_result.summary")?;
        for (value, field) in [
            (&self.requested_model, "advice_result.requested_model"),
            (&self.requested_effort, "advice_result.requested_effort"),
        ] {
            metadata(value, field)?;
        }
        for (value, field) in [
            (&self.actual_model, "advice_result.actual_model"),
            (&self.actual_effort, "advice_result.actual_effort"),
        ] {
            if let Some(value) = value {
                metadata(value, field)?;
            }
        }
        if let Some(usage) = &self.usage {
            usage.validate()?;
        }
        if self.strengths.len() > 8 || self.improvements.len() > 8 {
            return Err(error(
                ContractErrorCode::LimitExceeded,
                "advice_result.feedback",
            ));
        }
        for strength in &self.strengths {
            text(strength, 1_000, "advice_result.strengths")?;
        }
        for improvement in &self.improvements {
            improvement.validate()?;
        }
        if let Some(heard) = &self.heard_text {
            text(heard, 10_000, "advice_result.heard_text")?;
        }
        if let Some(reason) = &self.unassessable_reason {
            text(reason, 2_000, "advice_result.unassessable_reason")?;
        }
        match self.assessment {
            AdviceAssessment::Assessed | AdviceAssessment::ReferenceMismatch => {
                require(self.heard_text.is_some(), "advice_result.heard_text")?;
                require(
                    self.unassessable_reason.is_none(),
                    "advice_result.unassessable_reason",
                )?;
            }
            AdviceAssessment::Unassessable => {
                require(self.heard_text.is_none(), "advice_result.heard_text")?;
                require(
                    self.unassessable_reason.is_some(),
                    "advice_result.unassessable_reason",
                )?;
            }
        }
        if self.assessment != AdviceAssessment::Assessed {
            require(
                self.strengths.is_empty() && self.improvements.is_empty(),
                "advice_result.feedback",
            )?;
        }
        Ok(())
    }
}

impl HistoryRoot {
    pub fn validate(&self) -> Check {
        schema(self.schema_version)?;
        require(
            self.revision > 0 || (self.trials.is_empty() && self.receipts.is_empty()),
            "root.revision",
        )?;
        require(self.next_save_sequence > 0, "root.next_save_sequence")?;
        let mut sequences = BTreeSet::new();
        for (id, head) in &self.trials {
            id.validate()?;
            head.validate()?;
            require(
                head.admitted_sequence < self.next_save_sequence,
                "root.next_save_sequence",
            )?;
            if !sequences.insert(head.admitted_sequence) {
                return Err(error(ContractErrorCode::Conflict, "root.admitted_sequence"));
            }
        }
        for (id, receipt) in &self.receipts {
            id.validate()?;
            receipt.validate()?;
            require(
                receipt.committed_revision <= self.revision,
                "receipt.committed_revision",
            )?;
            if receipt
                .affected
                .iter()
                .any(|trial| !self.trials.contains_key(trial))
            {
                return Err(error(
                    ContractErrorCode::ReferenceMismatch,
                    "receipt.affected",
                ));
            }
        }
        document_size(self, 8 * 1024 * 1024, "root.bytes")
    }
}
