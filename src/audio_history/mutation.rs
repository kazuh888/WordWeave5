//! A bounded mutation becomes exact immutable object bytes before any write.

use std::{collections::BTreeMap, sync::Arc};

use serde::Serialize;

use super::closure::{contract_error, related, Document, ReadSession, DOCUMENT_CAP, REPORT_CAP};
use super::results::{check_advice_result, check_analysis_result};
use super::*;

const TRIAL_CAP: u64 = 256 * 1024;
const ROOT_CAP: u64 = 8 * 1024 * 1024;

pub(super) fn conflict(valid: bool) -> Result<(), HistoryError> {
    if valid {
        Ok(())
    } else {
        Err(error(HistoryErrorKind::Conflict))
    }
}

impl HistoryMutation {
    pub(super) fn affected(&self) -> &TrialId {
        match self {
            Self::CreateTrial { id, .. } => id,
            Self::ConfirmPractice { target, .. }
            | Self::AddAnalysisRun { target, .. }
            | Self::AddAdviceRun { target, .. }
            | Self::AddReflection { target, .. }
            | Self::AddMatchEvidence { target, .. }
            | Self::SetTrash { target, .. } => &target.id,
        }
    }

    fn target(&self) -> Option<&TrialTarget> {
        match self {
            Self::CreateTrial { .. } => None,
            Self::ConfirmPractice { target, .. }
            | Self::AddAnalysisRun { target, .. }
            | Self::AddAdviceRun { target, .. }
            | Self::AddReflection { target, .. }
            | Self::AddMatchEvidence { target, .. }
            | Self::SetTrash { target, .. } => Some(target),
        }
    }

    pub(super) fn payload_refs(&self) -> Vec<&BlobRef> {
        match self {
            Self::CreateTrial { input, .. } => vec![&input.audio.original],
            Self::AddAnalysisRun { run, .. } => run.result.iter().collect(),
            Self::AddAdviceRun { run, .. } => run.result.iter().collect(),
            _ => vec![],
        }
    }

    pub(super) fn validate(&self) -> Result<(), HistoryError> {
        self.affected().validate().map_err(contract_error)?;
        if let Some(target) = self.target() {
            target.expected.validate().map_err(contract_error)?;
        }
        match self {
            Self::CreateTrial {
                id,
                created_at,
                input,
                claim,
                duplicate,
            } => {
                check_trial(&initial_trial(
                    id.clone(),
                    *created_at,
                    input.clone(),
                    claim.clone(),
                ))?;
                check_confirmation(duplicate)?;
            }
            Self::ConfirmPractice {
                claim, duplicate, ..
            } => {
                claim.validate().map_err(contract_error)?;
                check_confirmation(duplicate)?;
            }
            Self::AddAnalysisRun { run, .. } => {
                run.check()?;
            }
            Self::AddAdviceRun { run, .. } => {
                run.check()?;
            }
            Self::AddReflection { reflection, .. } => {
                reflection.check()?;
            }
            Self::AddMatchEvidence { evidence, .. } => {
                evidence.validate().map_err(contract_error)?;
            }
            Self::SetTrash { trashed_at, .. } => {
                if let Some(at) = trashed_at {
                    at.validate().map_err(contract_error)?;
                }
            }
        }
        Ok(())
    }
}

fn check_confirmation(confirmation: &Option<DuplicateConfirmation>) -> Result<(), HistoryError> {
    if let Some(value) = confirmation {
        value.original.validate().map_err(contract_error)?;
        value.confirmed_at.validate().map_err(contract_error)?;
        for id in &value.matches {
            id.validate().map_err(contract_error)?;
        }
    }
    Ok(())
}

fn initial_trial(
    id: TrialId,
    created_at: RecordedAt,
    input: FrozenInput,
    claim: PracticeClaim,
) -> TrialRevision {
    TrialRevision {
        schema_version: 1,
        id,
        revision: 1,
        previous: None,
        created_at,
        input,
        claim,
        match_evidence: vec![],
        analysis: BTreeMap::new(),
        advice: BTreeMap::new(),
        reflections: vec![],
        trashed_at: None,
    }
}

#[derive(Debug)]
pub(super) struct PlannedObject {
    pub reference: BlobRef,
    pub bytes: Arc<[u8]>,
}

#[derive(Debug)]
pub(super) struct ObjectPlan {
    pub root: RootRef,
    pub objects: Vec<PlannedObject>,
}

pub(super) enum PlanOutcome {
    Planned(ObjectPlan),
    Duplicate(Vec<TrialId>),
}

/// Read-only planning may retain an absent external leaf, never invented bytes.
/// Only `complete` can turn this into a publishable plan.
pub(super) struct RecoveryObject {
    pub reference: BlobRef,
    pub bytes: Option<Arc<[u8]>>,
}

pub(super) struct RecoveryPlan {
    pub root: RootRef,
    pub objects: Vec<RecoveryObject>,
}

impl RecoveryPlan {
    pub fn complete(self) -> Result<ObjectPlan, HistoryError> {
        let objects = self
            .objects
            .into_iter()
            .map(|object| {
                Ok(PlannedObject {
                    reference: object.reference,
                    bytes: object
                        .bytes
                        .ok_or_else(|| error(HistoryErrorKind::MissingObject))?,
                })
            })
            .collect::<Result<_, HistoryError>>()?;
        Ok(ObjectPlan {
            root: self.root,
            objects,
        })
    }
}

pub(super) enum RecoveryPlanOutcome {
    Planned(RecoveryPlan),
    Duplicate(Vec<TrialId>),
}

fn append(
    objects: &mut Vec<RecoveryObject>,
    bytes: Arc<[u8]>,
    cap: u64,
) -> Result<BlobRef, HistoryError> {
    if bytes.len() as u64 > cap {
        return Err(error(HistoryErrorKind::LimitExceeded));
    }
    let reference = BlobRef {
        sha256: hash_bytes(&bytes)?,
        bytes: bytes.len() as u64,
    };
    reference.validate().map_err(contract_error)?;
    if let Some(old) = objects
        .iter()
        .find(|old| old.reference.sha256 == reference.sha256)
    {
        related(old.reference == reference && old.bytes.as_ref() == Some(&bytes))?;
    } else {
        if objects.len() >= 8 {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        objects.push(RecoveryObject {
            reference: reference.clone(),
            bytes: Some(bytes),
        });
    }
    Ok(reference)
}

fn append_document(
    objects: &mut Vec<RecoveryObject>,
    value: &(impl Document + Serialize),
    cap: u64,
) -> Result<BlobRef, HistoryError> {
    value.check()?;
    let bytes = serde_json::to_vec(value).map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
    append(objects, bytes.into(), cap)
}

fn payload(
    reader: &HistoryReader<'_>,
    request: &FrozenHistoryRequest,
    reference: &BlobRef,
    cap: u64,
) -> Result<Arc<[u8]>, HistoryError> {
    if reference.bytes > cap {
        return Err(error(HistoryErrorKind::LimitExceeded));
    }
    reference.validate().map_err(contract_error)?;
    if let Some(bytes) = request.payloads.get(&reference.sha256) {
        related(bytes.len() as u64 == reference.bytes)?;
        return Ok(Arc::clone(bytes));
    }
    Ok(ReadSession::new(reader).bytes(reference, cap)?.into())
}

fn recovery_payload(
    reader: &HistoryReader<'_>,
    request: &FrozenHistoryRequest,
    reference: &BlobRef,
    cap: u64,
    allow_missing: bool,
) -> Result<Option<Arc<[u8]>>, HistoryError> {
    match payload(reader, request, reference, cap) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if allow_missing && err.kind == HistoryErrorKind::MissingObject => Ok(None),
        Err(err) => Err(err),
    }
}

fn append_payload(
    objects: &mut Vec<RecoveryObject>,
    reference: &BlobRef,
    bytes: Option<Arc<[u8]>>,
    cap: u64,
) -> Result<(), HistoryError> {
    if let Some(bytes) = bytes {
        related(append(objects, bytes, cap)? == *reference)?;
    } else {
        // There is at most one external payload in every current mutation.
        objects.push(RecoveryObject {
            reference: reference.clone(),
            bytes: None,
        });
    }
    Ok(())
}

fn decode<T: Document>(bytes: &[u8]) -> Result<T, HistoryError> {
    let document: T =
        serde_json::from_slice(bytes).map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
    document.check()?;
    Ok(document)
}

fn read_trial(
    reader: &HistoryReader<'_>,
    reference: &BlobRef,
) -> Result<TrialRevision, HistoryError> {
    ReadSession::new(reader).document(reference, TRIAL_CAP)
}

fn duplicates(
    reader: &HistoryReader<'_>,
    root: &HistoryRoot,
    trial: &TrialRevision,
    confirmation: &Option<DuplicateConfirmation>,
) -> Result<Option<Vec<TrialId>>, HistoryError> {
    let mut matches = Vec::new();
    for (id, head) in &root.trials {
        if id != &trial.id
            && read_trial(reader, &head.document)?
                .input
                .audio
                .original
                .sha256
                == trial.input.audio.original.sha256
        {
            matches.push(id.clone());
        }
    }
    let confirmed = confirmation.as_ref().is_some_and(|confirmation| {
        confirmation.original == trial.input.audio.original.sha256
            && confirmation.matches == matches
    });
    if (!matches.is_empty() || confirmation.is_some()) && !confirmed {
        return Ok(Some(matches));
    }
    if let PracticeClaim::Practice {
        duplicate_confirmed,
        ..
    } = trial.claim
    {
        related(duplicate_confirmed == confirmed)?;
    }
    Ok(None)
}

/// Old revisions also reserve their RunIds, even if an imported head omits one.
fn run_used(
    reader: &HistoryReader<'_>,
    root: &HistoryRoot,
    id: &RunId,
    owner: &TrialId,
    analysis: bool,
) -> Result<bool, HistoryError> {
    let mut reads = ReadSession::new(reader);
    let mut used = false;
    for (trial_id, head) in &root.trials {
        let mut reference = Some(head.document.clone());
        while let Some(current) = reference {
            let trial: TrialRevision = reads.document(&current, TRIAL_CAP)?;
            if trial.analysis.contains_key(id) {
                conflict(analysis && trial_id == owner)?;
                used = true;
            }
            if trial.advice.contains_key(id) {
                conflict(!analysis && trial_id == owner)?;
                used = true;
            }
            reference = trial.previous;
        }
    }
    Ok(used)
}

fn analysis_change(
    reader: &HistoryReader<'_>,
    root: &HistoryRoot,
    trial: &TrialRevision,
    run: &AnalysisRun,
) -> Result<(), HistoryError> {
    related(run.trial == trial.id && run.audio == trial.input.audio.original.sha256)?;
    let used = run_used(reader, root, &run.id, &trial.id, true)?;
    if let Some(reference) = trial.analysis.get(&run.id) {
        let old: AnalysisRun = ReadSession::new(reader).document(reference, DOCUMENT_CAP)?;
        conflict(
            old.state == AnalysisRunState::Running
                && run.state != AnalysisRunState::Running
                && old.id == run.id
                && old.trial == run.trial
                && old.audio == run.audio
                && old.pcm == run.pcm
                && old.decoder == run.decoder
                && old.profile == run.profile
                && old.started_at == run.started_at,
        )?;
        related(
            run.ended_at
                .is_some_and(|end| end.utc_millis >= run.started_at.utc_millis),
        )?;
    } else {
        conflict(!used && run.state == AnalysisRunState::Running)?;
    }
    Ok(())
}

fn advice_change(
    reader: &HistoryReader<'_>,
    root: &HistoryRoot,
    trial: &TrialRevision,
    run: &AdviceRun,
) -> Result<(), HistoryError> {
    related(
        run.trial == trial.id
            && run.audio == trial.input.audio.original.sha256
            && run.reference == trial.input.reference.sha256,
    )?;
    let used = run_used(reader, root, &run.id, &trial.id, false)?;
    if let Some(reference) = trial.advice.get(&run.id) {
        let old: AdviceRun = ReadSession::new(reader).document(reference, DOCUMENT_CAP)?;
        conflict(
            old.state == AdviceRunState::Sending
                && run.state != AdviceRunState::Sending
                && old.id == run.id
                && old.trial == run.trial
                && old.audio == run.audio
                && old.reference == run.reference
                && old.confirmation == run.confirmation
                && old.confirmed_at == run.confirmed_at
                && old.destination == run.destination
                && old.versions == run.versions
                && old.started_at == run.started_at,
        )?;
        related(
            run.ended_at
                .is_some_and(|end| end.utc_millis >= run.started_at.utc_millis),
        )?;
        if run.state == AdviceRunState::Interrupted {
            related(run.send == SendCertainty::MayHaveBeenSent)?;
        }
    } else {
        conflict(
            !used && run.state == AdviceRunState::Sending && run.send == SendCertainty::NotSent,
        )?;
    }
    Ok(())
}

pub(super) fn plan(
    reader: &HistoryReader<'_>,
    root: HistoryRoot,
    operation: &OperationId,
    request: &FrozenHistoryRequest,
) -> Result<PlanOutcome, HistoryError> {
    match recovery_plan(reader, root, operation, request, false)? {
        RecoveryPlanOutcome::Planned(plan) => Ok(PlanOutcome::Planned(plan.complete()?)),
        RecoveryPlanOutcome::Duplicate(ids) => Ok(PlanOutcome::Duplicate(ids)),
    }
}

pub(super) fn recovery_plan(
    reader: &HistoryReader<'_>,
    mut root: HistoryRoot,
    operation: &OperationId,
    request: &FrozenHistoryRequest,
    allow_missing: bool,
) -> Result<RecoveryPlanOutcome, HistoryError> {
    let mutation = request.mutation();
    mutation.validate()?;
    let mut objects = Vec::new();
    let trial = if let HistoryMutation::CreateTrial {
        id,
        created_at,
        input,
        claim,
        duplicate,
    } = mutation
    {
        conflict(!root.trials.contains_key(id))?;
        let trial = initial_trial(id.clone(), *created_at, input.clone(), claim.clone());
        if let Some(matches) = duplicates(reader, &root, &trial, duplicate)? {
            return Ok(RecoveryPlanOutcome::Duplicate(matches));
        }
        let bytes = recovery_payload(
            reader,
            request,
            &input.audio.original,
            ORIGINAL_CAP,
            allow_missing,
        )?;
        append_payload(&mut objects, &input.audio.original, bytes, ORIGINAL_CAP)?;
        let sequence = root.next_save_sequence;
        root.next_save_sequence = root
            .next_save_sequence
            .checked_add(1)
            .ok_or_else(|| error(HistoryErrorKind::LimitExceeded))?;
        // This placeholder is replaced by the checked serialized Trial below.
        root.trials.insert(
            id.clone(),
            TrialHead {
                document: input.audio.original.clone(),
                admitted_sequence: sequence,
            },
        );
        trial
    } else {
        let target = mutation
            .target()
            .ok_or_else(|| error(HistoryErrorKind::InvalidDocument))?;
        let head = root
            .trials
            .get(&target.id)
            .ok_or_else(|| error(HistoryErrorKind::Conflict))?;
        conflict(head.document == target.expected)?;
        let mut trial = read_trial(reader, &head.document)?;
        if !matches!(mutation, HistoryMutation::SetTrash { .. }) {
            conflict(trial.trashed_at.is_none())?;
        }
        match mutation {
            HistoryMutation::ConfirmPractice {
                claim, duplicate, ..
            } => {
                conflict(
                    matches!(trial.claim, PracticeClaim::Reference)
                        && matches!(claim, PracticeClaim::Practice { .. }),
                )?;
                trial.claim = claim.clone();
                if let Some(matches) = duplicates(reader, &root, &trial, duplicate)? {
                    return Ok(RecoveryPlanOutcome::Duplicate(matches));
                }
            }
            HistoryMutation::AddAnalysisRun { run, .. } => {
                analysis_change(reader, &root, &trial, run)?;
                if let Some(reference) = &run.result {
                    let bytes =
                        recovery_payload(reader, request, reference, REPORT_CAP, allow_missing)?;
                    if let Some(bytes) = &bytes {
                        check_analysis_result(&trial, run, &decode::<AnalysisReport>(bytes)?)?;
                    }
                    append_payload(&mut objects, reference, bytes, REPORT_CAP)?;
                }
                let reference = append_document(&mut objects, run, DOCUMENT_CAP)?;
                trial.analysis.insert(run.id.clone(), reference);
            }
            HistoryMutation::AddAdviceRun { run, .. } => {
                advice_change(reader, &root, &trial, run)?;
                if let Some(reference) = &run.result {
                    let bytes =
                        recovery_payload(reader, request, reference, DOCUMENT_CAP, allow_missing)?;
                    if let Some(bytes) = &bytes {
                        check_advice_result(&trial, run, &decode::<AdviceResult>(bytes)?)?;
                    }
                    append_payload(&mut objects, reference, bytes, DOCUMENT_CAP)?;
                }
                let reference = append_document(&mut objects, run, DOCUMENT_CAP)?;
                trial.advice.insert(run.id.clone(), reference);
            }
            HistoryMutation::AddReflection { reflection, .. } => {
                related(reflection.trial == trial.id)?;
                trial
                    .reflections
                    .push(append_document(&mut objects, reflection, DOCUMENT_CAP)?);
            }
            HistoryMutation::AddMatchEvidence { evidence, .. } => {
                let expected = match evidence {
                    MatchEvidence::AiAssessed { run } => Some((run, AdviceAssessment::Assessed)),
                    MatchEvidence::AiMismatch { run } => {
                        Some((run, AdviceAssessment::ReferenceMismatch))
                    }
                    MatchEvidence::AiUnassessable { run } => {
                        Some((run, AdviceAssessment::Unassessable))
                    }
                    MatchEvidence::HumanConfirmed { .. } => None,
                };
                if let Some((run, expected)) = expected {
                    related(
                        reader
                            .read_advice(&trial, run)?
                            .result
                            .as_ref()
                            .map(|r| r.assessment)
                            == Some(expected),
                    )?;
                }
                trial.match_evidence.push(evidence.clone());
            }
            HistoryMutation::SetTrash { trashed_at, .. } => {
                conflict(trial.trashed_at.is_some() != trashed_at.is_some())?;
                if trashed_at.is_some() {
                    for run in trial.analysis.keys() {
                        if reader.read_analysis(&trial, run)?.run.state == AnalysisRunState::Running
                        {
                            return Err(error(HistoryErrorKind::Busy));
                        }
                    }
                    for run in trial.advice.keys() {
                        if reader.read_advice(&trial, run)?.run.state == AdviceRunState::Sending {
                            return Err(error(HistoryErrorKind::Busy));
                        }
                    }
                }
                trial.trashed_at = *trashed_at;
            }
            HistoryMutation::CreateTrial { .. } => {
                return Err(error(HistoryErrorKind::InvalidDocument))
            }
        }
        trial.revision = trial
            .revision
            .checked_add(1)
            .ok_or_else(|| error(HistoryErrorKind::LimitExceeded))?;
        trial.previous = Some(head.document.clone());
        trial
    };
    check_trial(&trial)?;
    let trial_ref = append_document(&mut objects, &trial, TRIAL_CAP)?;
    root.trials
        .get_mut(&trial.id)
        .ok_or_else(|| error(HistoryErrorKind::InvalidDocument))?
        .document = trial_ref;
    root.revision = root
        .revision
        .checked_add(1)
        .ok_or_else(|| error(HistoryErrorKind::LimitExceeded))?;
    root.receipts.insert(
        operation.clone(),
        OperationReceipt {
            request: request.request_hash().clone(),
            committed_revision: root.revision,
            affected: vec![trial.id.clone()],
        },
    );
    let root = RootRef::new(append_document(&mut objects, &root, ROOT_CAP)?);
    Ok(RecoveryPlanOutcome::Planned(RecoveryPlan { root, objects }))
}
