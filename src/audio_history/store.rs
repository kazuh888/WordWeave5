//! Immutable publication boundary. Publishing objects is not a host commit.
//! I1b-2a signatures are fixed before independent behavioral tests.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::store::{acquire_progress_write, ProgressGateError};

use super::closure::{contract_error, related, ReadSession, DOCUMENT_CAP};
use super::mutation::{conflict, ObjectPlan, PlanOutcome};
use super::recovery::{self, RecoveryIntent};
use super::*;

fn gate_error(err: ProgressGateError) -> HistoryError {
    error(match err {
        ProgressGateError::Reentrant => HistoryErrorKind::Busy,
        ProgressGateError::Poisoned | ProgressGateError::WriteBlocked => HistoryErrorKind::Io,
    })
}

pub trait CommittedHistoryHead {
    fn read_current(&self, storage: &Storage) -> Result<Option<RootRef>, HistoryError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryQuota {
    pub limit_bytes: u64,
}

impl HistoryQuota {
    pub fn v1() -> Self {
        Self {
            limit_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialTarget {
    pub id: TrialId,
    pub expected: BlobRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DuplicateConfirmation {
    pub original: Sha256,
    pub matches: Vec<TrialId>,
    pub confirmed_at: RecordedAt,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum HistoryMutation {
    CreateTrial {
        id: TrialId,
        created_at: RecordedAt,
        input: FrozenInput,
        claim: PracticeClaim,
        #[serde(deserialize_with = "Option::deserialize")]
        duplicate: Option<DuplicateConfirmation>,
    },
    ConfirmPractice {
        target: TrialTarget,
        claim: PracticeClaim,
        #[serde(deserialize_with = "Option::deserialize")]
        duplicate: Option<DuplicateConfirmation>,
    },
    AddAnalysisRun {
        target: TrialTarget,
        run: AnalysisRun,
    },
    AddAdviceRun {
        target: TrialTarget,
        run: AdviceRun,
    },
    AddReflection {
        target: TrialTarget,
        reflection: Reflection,
    },
    AddMatchEvidence {
        target: TrialTarget,
        evidence: MatchEvidence,
    },
    SetTrash {
        target: TrialTarget,
        #[serde(deserialize_with = "Option::deserialize")]
        trashed_at: Option<RecordedAt>,
    },
}

pub struct FrozenHistoryRequest {
    mutation: HistoryMutation,
    mutation_json: String,
    request: Sha256,
    pub(super) payloads: BTreeMap<Sha256, Arc<[u8]>>,
}

impl fmt::Debug for FrozenHistoryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrozenHistoryRequest")
            .field("request", &self.request)
            .field("payload_count", &self.payloads.len())
            .finish_non_exhaustive()
    }
}

impl FrozenHistoryRequest {
    pub fn new(mutation: HistoryMutation, payloads: Vec<Arc<[u8]>>) -> Result<Self, HistoryError> {
        mutation.validate()?;
        let mutation_json = serde_json::to_string(&mutation)
            .map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        Self::from_exact(mutation_json, payloads)
    }

    fn from_exact(mutation_json: String, payloads: Vec<Arc<[u8]>>) -> Result<Self, HistoryError> {
        if mutation_json.len() > recovery::INTENT_CAP {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        let mutation: HistoryMutation = serde_json::from_str(&mutation_json)
            .map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        mutation.validate()?;
        let request = hash_bytes(mutation_json.as_bytes())?;
        let mut retained: BTreeMap<Sha256, Arc<[u8]>> = BTreeMap::new();
        for bytes in payloads {
            if bytes.len() as u64 > DOCUMENT_CAP {
                return Err(error(HistoryErrorKind::LimitExceeded));
            }
            let hash = hash_bytes(&bytes)?;
            related(mutation.payload_refs().iter().any(|reference| {
                reference.sha256 == hash && reference.bytes == bytes.len() as u64
            }))?;
            if let Some(previous) = retained.get(&hash) {
                related(previous.as_ref() == bytes.as_ref())?;
            } else {
                retained.insert(hash, bytes);
            }
        }
        Ok(Self {
            mutation,
            mutation_json,
            request,
            payloads: retained,
        })
    }

    pub fn request_hash(&self) -> &Sha256 {
        &self.request
    }
    pub fn mutation_json(&self) -> &str {
        &self.mutation_json
    }
    pub fn mutation(&self) -> &HistoryMutation {
        &self.mutation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotaLedger {
    pub object_bytes: u64,
    pub recovery_bytes: u64,
    pub residual_bytes: u64,
    pub retained_bytes: u64,
    pub limit_bytes: u64,
    pub warning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotaPlan {
    pub before: QuotaLedger,
    pub added_object_bytes: u64,
    pub intent_delta_bytes: i64,
    pub projected_bytes: u64,
}

pub struct PreparedHistory {
    quota: QuotaPlan,
    session: Arc<()>,
    intent: RecoveryIntent,
    intent_bytes: Vec<u8>,
    plan: ObjectPlan,
}

impl fmt::Debug for PreparedHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedHistory")
            .field("operation", &self.intent.operation)
            .field("request", &self.intent.request)
            .field("root", &self.plan.root)
            .field("object_count", &self.plan.objects.len())
            .field("quota", &self.quota)
            .finish_non_exhaustive()
    }
}

impl PreparedHistory {
    pub fn quota_plan(&self) -> &QuotaPlan {
        &self.quota
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedHistory {
    pub base: Option<RootRef>,
    pub next: RootRef,
    pub operation: OperationId,
    pub request: Sha256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedHistory {
    pub current: RootRef,
    pub operation: OperationId,
    pub receipt: OperationReceipt,
}

#[derive(Debug)]
pub enum PrepareOutcome {
    Ready(PreparedHistory),
    AlreadyCommitted(CommittedHistory),
    DuplicateConfirmationRequired(Vec<TrialId>),
}

#[derive(Debug)]
pub enum PublishOutcome {
    Published(PublishedHistory),
    AlreadyCommitted(CommittedHistory),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryCheckpoint {
    BeforeIntent,
    AfterIntent,
    BeforeObject { index: usize },
    AfterObject { index: usize },
    BeforeClosure,
    AfterClosure,
    BeforeFinish,
    AfterFinish,
}

pub trait HistoryFaults {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError>;
}

pub struct NoHistoryFaults;

impl HistoryFaults for NoHistoryFaults {
    fn check(&self, _point: HistoryCheckpoint) -> Result<(), HistoryError> {
        Ok(())
    }
}

pub struct HistoryWriter<'a> {
    storage: &'a mut Storage,
    head: &'a dyn CommittedHistoryHead,
    quota: HistoryQuota,
    faults: &'a dyn HistoryFaults,
    session: Arc<()>,
}

struct CurrentHistory {
    reference: Option<RootRef>,
    document: HistoryRoot,
    closure: Vec<BlobRef>,
}

impl<'a> HistoryWriter<'a> {
    pub(crate) fn verify_publication_under_guard(
        storage: &Storage,
        published: &PublishedHistory,
        guard: &crate::store::ProgressWriteGuard,
    ) -> Result<(), HistoryError> {
        guard.ensure_writable().map_err(gate_error)?;
        recovery::guard(storage)?;
        published.operation.validate().map_err(contract_error)?;
        published.request.validate().map_err(contract_error)?;
        if let Some(base) = &published.base {
            base.validate().map_err(contract_error)?;
        }
        published.next.validate().map_err(contract_error)?;
        let head = crate::store::ProgressHistoryHead;
        conflict(head.read_current(storage)? == published.base)?;
        let (intent, raw) = recovery::read_intent(storage, &published.operation)?
            .ok_or_else(|| error(HistoryErrorKind::MissingObject))?;
        conflict(
            intent.operation == published.operation
                && intent.request == published.request
                && intent.base == published.base
                && intent.planned_root == published.next,
        )?;
        let request = FrozenHistoryRequest::from_exact(intent.mutation_json.clone(), vec![])?;
        let reader = publication_reader(storage)?;
        let base = read_history_at(&reader, published.base.clone())?;
        conflict(!base.document.receipts.contains_key(&published.operation))?;
        let (plan, missing) = checked_old_plan_with_reader(&reader, &intent, &request)?;
        if !missing.is_empty() {
            return Err(error(HistoryErrorKind::MissingObject));
        }
        let plan = plan.complete()?;
        let next = read_history_at(&reader, Some(plan.root))?;
        let receipt = next
            .document
            .receipts
            .get(&published.operation)
            .ok_or_else(|| error(HistoryErrorKind::RelationshipMismatch))?;
        related(
            receipt.request == published.request
                && receipt.affected == [request.mutation().affected().clone()]
                && receipt.committed_revision == next.document.revision,
        )?;
        recovery::guard(storage)?;
        conflict(head.read_current(storage)? == published.base)?;
        let latest = recovery::read_intent(storage, &published.operation)?
            .ok_or_else(|| error(HistoryErrorKind::Conflict))?;
        conflict(latest.1 == raw)
    }

    pub(crate) fn confirm_publication(
        storage: &Storage,
        published: &PublishedHistory,
    ) -> Result<CommittedHistory, HistoryError> {
        confirm_with_reader_and_head(
            storage,
            &publication_reader(storage)?,
            &crate::store::ProgressHistoryHead,
            published,
        )
    }

    pub fn new(
        storage: &'a mut Storage,
        head: &'a dyn CommittedHistoryHead,
        quota: HistoryQuota,
    ) -> Result<Self, HistoryError> {
        Self::with_faults(storage, head, quota, &NoHistoryFaults)
    }

    pub fn with_faults(
        storage: &'a mut Storage,
        head: &'a dyn CommittedHistoryHead,
        quota: HistoryQuota,
        faults: &'a dyn HistoryFaults,
    ) -> Result<Self, HistoryError> {
        if quota.limit_bytes == 0 {
            return Err(error(HistoryErrorKind::InvalidDocument));
        }
        recovery::paths(storage)?;
        Ok(Self {
            storage,
            head,
            quota,
            faults,
            session: Arc::new(()),
        })
    }

    pub fn prepare(
        &mut self,
        operation: OperationId,
        base: Option<&RootRef>,
        request: &FrozenHistoryRequest,
    ) -> Result<PrepareOutcome, HistoryError> {
        let _guard = acquire_progress_write().map_err(gate_error)?;
        operation.validate().map_err(contract_error)?;
        recovery::guard(self.storage)?;
        let inventory = recovery::inventory(self.storage, self.quota)?;
        let current = self.current(inventory.ledger.retained_bytes)?;
        if let Some(committed) = Self::receipt(
            &current,
            &operation,
            request.request_hash(),
            request.mutation().affected(),
        )? {
            return Ok(PrepareOutcome::AlreadyCommitted(committed));
        }
        conflict(current.reference.as_ref() == base)?;
        if let Some((intent, _)) = recovery::read_intent(self.storage, &operation)? {
            conflict(
                intent.request == *request.request_hash()
                    && intent.mutation_json == request.mutation_json(),
            )?;
            return Err(error(HistoryErrorKind::Busy));
        }
        if inventory.ledger.retained_bytes > self.quota.limit_bytes {
            return Err(error(HistoryErrorKind::QuotaExceeded));
        }
        let reader = self.reader(inventory.ledger.retained_bytes)?;
        let plan = match mutation::plan(&reader, current.document, &operation, request)? {
            PlanOutcome::Duplicate(ids) => {
                return Ok(PrepareOutcome::DuplicateConfirmationRequired(ids))
            }
            PlanOutcome::Planned(plan) => plan,
        };
        // Replacing the root retains all old Trial revisions, but the base root
        // itself is no longer a closure edge. Bound nodes before creating intent.
        let mut nodes: BTreeSet<_> = current.closure.iter().map(|r| &r.sha256).collect();
        if let Some(root) = &current.reference {
            nodes.remove(&root.as_blob().sha256);
        }
        for object in &plan.objects {
            nodes.insert(&object.reference.sha256);
        }
        if nodes.len() > 100_000 {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        let intent = RecoveryIntent {
            schema_version: 1,
            operation,
            request: request.request_hash().clone(),
            mutation_json: request.mutation_json().to_owned(),
            base: current.reference,
            planned_root: plan.root.clone(),
            planned_objects: plan.objects.iter().map(|o| o.reference.clone()).collect(),
        };
        let intent_bytes = intent.encode()?;
        let added_object_bytes = Self::missing_bytes(&inventory, &plan)?;
        let intent_delta_bytes = i64::try_from(intent_bytes.len())
            .map_err(|_| error(HistoryErrorKind::LimitExceeded))?;
        let projected_bytes = recovery::add(
            recovery::add(inventory.ledger.retained_bytes, added_object_bytes)?,
            intent_bytes.len() as u64,
        )?;
        if projected_bytes > self.quota.limit_bytes {
            return Err(error(HistoryErrorKind::QuotaExceeded));
        }
        let quota = QuotaPlan {
            before: inventory.ledger,
            added_object_bytes,
            intent_delta_bytes,
            projected_bytes,
        };
        self.faults.check(HistoryCheckpoint::BeforeIntent)?;
        self.unchanged(&intent.base)?;
        recovery::save_intent(self.storage, &intent, &intent_bytes)?;
        self.faults.check(HistoryCheckpoint::AfterIntent)?;
        Ok(PrepareOutcome::Ready(PreparedHistory {
            quota,
            session: Arc::clone(&self.session),
            intent,
            intent_bytes,
            plan,
        }))
    }

    pub fn publish(&mut self, prepared: &PreparedHistory) -> Result<PublishOutcome, HistoryError> {
        let _guard = acquire_progress_write().map_err(gate_error)?;
        conflict(Arc::ptr_eq(&self.session, &prepared.session))?;
        recovery::guard(self.storage)?;
        let inventory = recovery::inventory(self.storage, self.quota)?;
        let current = self.current(inventory.ledger.retained_bytes)?;
        let mutation: HistoryMutation = serde_json::from_str(&prepared.intent.mutation_json)
            .map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
        if let Some(committed) = Self::receipt(
            &current,
            &prepared.intent.operation,
            &prepared.intent.request,
            mutation.affected(),
        )? {
            return Ok(PublishOutcome::AlreadyCommitted(committed));
        }
        conflict(current.reference == prepared.intent.base)?;
        let Some((intent, bytes)) =
            recovery::read_intent(self.storage, &prepared.intent.operation)?
        else {
            return Err(error(HistoryErrorKind::MissingObject));
        };
        conflict(intent == prepared.intent && bytes == prepared.intent_bytes)?;
        let missing = Self::missing_bytes(&inventory, &prepared.plan)?;
        if inventory.ledger.retained_bytes > self.quota.limit_bytes
            || recovery::add(inventory.ledger.retained_bytes, missing)? > self.quota.limit_bytes
        {
            return Err(error(HistoryErrorKind::QuotaExceeded));
        }
        for (index, object) in prepared.plan.objects.iter().enumerate() {
            self.faults
                .check(HistoryCheckpoint::BeforeObject { index })?;
            self.unchanged(&prepared.intent.base)?;
            // Recheck the intent before each object. A removed or replaced intent
            // must never grant publication authority to a surviving memory token.
            let stored = recovery::read_intent(self.storage, &prepared.intent.operation)?;
            conflict(stored.as_ref().is_some_and(|(intent, bytes)| {
                intent == &prepared.intent && bytes == &prepared.intent_bytes
            }))?;
            crate::assets::AssetStore::new(self.storage.dir.join("audio-history"))
                .put(crate::assets::AssetKind::FileBlob, &object.bytes)
                .map_err(|_| error(HistoryErrorKind::Io))?;
            let reader = self.reader(self.quota.limit_bytes)?;
            let actual = ReadSession::new(&reader).bytes(&object.reference, DOCUMENT_CAP)?;
            related(actual.as_slice() == object.bytes.as_ref())?;
            self.faults
                .check(HistoryCheckpoint::AfterObject { index })?;
        }
        self.faults.check(HistoryCheckpoint::BeforeClosure)?;
        self.unchanged(&prepared.intent.base)?;
        self.reader(self.quota.limit_bytes)?
            .verify_closure(&prepared.plan.root)?;
        self.faults.check(HistoryCheckpoint::AfterClosure)?;
        self.unchanged(&prepared.intent.base)?;
        Ok(PublishOutcome::Published(PublishedHistory {
            base: prepared.intent.base.clone(),
            next: prepared.plan.root.clone(),
            operation: prepared.intent.operation.clone(),
            request: prepared.intent.request.clone(),
        }))
    }

    pub fn quota(&self) -> Result<QuotaLedger, HistoryError> {
        Ok(recovery::inventory(self.storage, self.quota)?.ledger)
    }

    /// A bounded read-only snapshot; candidate states never grant write authority.
    pub fn recover(&self, after: Option<&OperationId>) -> RecoveryView {
        let mut view = RecoveryView {
            candidates: Vec::new(),
            next_after: None,
            diagnostics: Vec::new(),
        };
        let result = (|| -> Result<(), HistoryError> {
            let (operations, diagnostics) = recovery::operations(self.storage)?;
            view.diagnostics.extend(diagnostics);
            if let Some(after) = after {
                after.validate().map_err(contract_error)?;
                if !operations.contains(after) {
                    return Err(error(HistoryErrorKind::InvalidDocument));
                }
            }
            let current = self.current(u64::MAX)?;
            let remaining: Vec<_> = operations
                .iter()
                .filter(|id| after.is_none_or(|after| *id > after))
                .collect();
            for operation in remaining.iter().take(32) {
                let intent = match self.required_intent(operation) {
                    Ok((intent, _)) => intent,
                    Err(err) => {
                        view.diagnostics.push(err);
                        continue;
                    }
                };
                let mut candidate = RecoveryCandidate {
                    operation: intent.operation.clone(),
                    request: intent.request.clone(),
                    state: RecoveryState::Damaged,
                    missing: vec![],
                };
                match self.classify(&intent, &current) {
                    Ok((state, missing)) => {
                        candidate.state = state;
                        candidate.missing = missing;
                    }
                    Err(err) => {
                        if err.kind == HistoryErrorKind::Conflict {
                            candidate.state = RecoveryState::Conflict;
                        }
                        view.diagnostics.push(err);
                    }
                }
                view.candidates.push(candidate);
            }
            if remaining.len() > 32 {
                view.next_after = Some(remaining[31].clone());
            }
            // Do not return readiness computed from a head that moved mid-scan.
            conflict(self.head.read_current(self.storage)? == current.reference)?;
            recovery::paths(self.storage)?;
            Ok(())
        })();
        if let Err(err) = result {
            for candidate in &mut view.candidates {
                candidate.state = if err.kind == HistoryErrorKind::Conflict {
                    RecoveryState::Conflict
                } else {
                    RecoveryState::Damaged
                };
            }
            view.diagnostics.push(err);
        }
        view
    }

    pub fn read_recovery_request(
        &self,
        operation: &OperationId,
        payloads: Vec<Arc<[u8]>>,
    ) -> Result<FrozenHistoryRequest, HistoryError> {
        let (intent, _) = self.required_intent(operation)?;
        let request = FrozenHistoryRequest::from_exact(intent.mutation_json.clone(), payloads)?;
        self.checked_old_plan(&intent, &request)?.0.complete()?;
        Ok(request)
    }

    pub fn resume(
        &mut self,
        operation: &OperationId,
        request: &FrozenHistoryRequest,
    ) -> Result<PrepareOutcome, HistoryError> {
        let _guard = acquire_progress_write().map_err(gate_error)?;
        operation.validate().map_err(contract_error)?;
        recovery::guard(self.storage)?;
        let (old, old_bytes) = self.required_intent(operation)?;
        conflict(
            old.request == *request.request_hash() && old.mutation_json == request.mutation_json(),
        )?;
        let inventory = recovery::inventory(self.storage, self.quota)?;
        let current = self.current(inventory.ledger.retained_bytes)?;
        // Even a rebased or already-committed operation must have a real old plan.
        self.checked_old_plan(&old, request)?;
        if let Some(committed) = Self::receipt(
            &current,
            operation,
            request.request_hash(),
            request.mutation().affected(),
        )? {
            return Ok(PrepareOutcome::AlreadyCommitted(committed));
        }
        if inventory.ledger.retained_bytes > self.quota.limit_bytes {
            return Err(error(HistoryErrorKind::QuotaExceeded));
        }
        let reader = self.reader(inventory.ledger.retained_bytes)?;
        let plan = match mutation::plan(&reader, current.document, operation, request)? {
            PlanOutcome::Planned(plan) => plan,
            PlanOutcome::Duplicate(ids) => {
                return Ok(PrepareOutcome::DuplicateConfirmationRequired(ids))
            }
        };
        self.check_nodes(&current.closure, current.reference.as_ref(), &plan)?;
        let intent = RecoveryIntent {
            schema_version: 1,
            operation: operation.clone(),
            request: request.request_hash().clone(),
            mutation_json: request.mutation_json().to_owned(),
            base: current.reference,
            planned_root: plan.root.clone(),
            planned_objects: plan.objects.iter().map(|o| o.reference.clone()).collect(),
        };
        let intent_bytes = if intent == old {
            old_bytes.clone()
        } else {
            intent.encode()?
        };
        let added_object_bytes = Self::missing_bytes(&inventory, &plan)?;
        let intent_delta_bytes = i64::try_from(intent_bytes.len())
            .map_err(|_| error(HistoryErrorKind::LimitExceeded))?
            - i64::try_from(old_bytes.len()).map_err(|_| error(HistoryErrorKind::LimitExceeded))?;
        let without_old = inventory
            .ledger
            .retained_bytes
            .checked_sub(old_bytes.len() as u64)
            .ok_or_else(|| error(HistoryErrorKind::RelationshipMismatch))?;
        let projected_bytes = recovery::add(
            recovery::add(without_old, added_object_bytes)?,
            intent_bytes.len() as u64,
        )?;
        if projected_bytes > self.quota.limit_bytes {
            return Err(error(HistoryErrorKind::QuotaExceeded));
        }
        let quota = QuotaPlan {
            before: inventory.ledger,
            added_object_bytes,
            intent_delta_bytes,
            projected_bytes,
        };
        self.faults.check(HistoryCheckpoint::BeforeIntent)?;
        self.unchanged(&intent.base)?;
        recovery::replace_intent(self.storage, &old, &old_bytes, &intent, &intent_bytes)?;
        self.faults.check(HistoryCheckpoint::AfterIntent)?;
        Ok(PrepareOutcome::Ready(PreparedHistory {
            quota,
            session: Arc::clone(&self.session),
            intent,
            intent_bytes,
            plan,
        }))
    }

    pub fn confirm_committed(
        &self,
        published: &PublishedHistory,
    ) -> Result<CommittedHistory, HistoryError> {
        confirm_with_reader_and_head(self.storage, &self.reader(u64::MAX)?, self.head, published)
    }

    pub fn finish(&mut self, committed: &CommittedHistory) -> Result<(), HistoryError> {
        let _guard = acquire_progress_write().map_err(gate_error)?;
        recovery::guard(self.storage)?;
        self.checked_committed(committed)?;
        let Some((intent, bytes)) = recovery::read_intent(self.storage, &committed.operation)?
        else {
            return Ok(());
        };
        conflict(intent.request == committed.receipt.request)?;
        let request = FrozenHistoryRequest::from_exact(intent.mutation_json.clone(), vec![])?;
        conflict(committed.receipt.affected == [request.mutation().affected().clone()])?;
        self.checked_old_plan(&intent, &request)?.0.complete()?;
        // Verify the intent's planned receipt, not merely the public token fields.
        let planned = self.at(Some(intent.planned_root.clone()), u64::MAX)?;
        conflict(planned.document.receipts.get(&committed.operation) == Some(&committed.receipt))?;
        self.faults.check(HistoryCheckpoint::BeforeFinish)?;
        self.checked_committed(committed)?;
        recovery::remove_intent(self.storage, &intent, &bytes)?;
        self.faults.check(HistoryCheckpoint::AfterFinish)
    }

    fn checked_committed(&self, committed: &CommittedHistory) -> Result<(), HistoryError> {
        committed.operation.validate().map_err(contract_error)?;
        let original = self.at(Some(committed.current.clone()), u64::MAX)?;
        conflict(original.document.receipts.get(&committed.operation) == Some(&committed.receipt))?;
        let latest = self.current(u64::MAX)?;
        let Some(receipt) = latest.document.receipts.get(&committed.operation) else {
            return Err(error(if latest.reference.is_none() {
                HistoryErrorKind::Busy
            } else {
                HistoryErrorKind::Conflict
            }));
        };
        conflict(receipt == &committed.receipt)?;
        self.unchanged(&latest.reference)
    }

    fn required_intent(
        &self,
        operation: &OperationId,
    ) -> Result<(RecoveryIntent, Vec<u8>), HistoryError> {
        recovery::read_intent(self.storage, operation)?
            .ok_or_else(|| error(HistoryErrorKind::MissingObject))
    }

    /// Rebuild on the original base and compare every reference and existing byte.
    /// Missing external leaves remain absent; generated metadata stays available.
    fn checked_old_plan(
        &self,
        intent: &RecoveryIntent,
        request: &FrozenHistoryRequest,
    ) -> Result<(mutation::RecoveryPlan, Vec<BlobRef>), HistoryError> {
        checked_old_plan_with_reader(&self.reader(u64::MAX)?, intent, request)
    }

    fn classify(
        &self,
        intent: &RecoveryIntent,
        current: &CurrentHistory,
    ) -> Result<(RecoveryState, Vec<BlobRef>), HistoryError> {
        let request = FrozenHistoryRequest::from_exact(intent.mutation_json.clone(), vec![])?;
        let (_, missing) = self.checked_old_plan(intent, &request)?;
        if Self::receipt(
            current,
            &intent.operation,
            &intent.request,
            request.mutation().affected(),
        )?
        .is_some()
        {
            return Ok((RecoveryState::AlreadyCommitted, missing));
        }
        let reader = self.reader(u64::MAX)?;
        if matches!(
            mutation::recovery_plan(
                &reader,
                current.document.clone(),
                &intent.operation,
                &request,
                true
            )?,
            mutation::RecoveryPlanOutcome::Duplicate(_)
        ) {
            return Err(error(HistoryErrorKind::Conflict));
        }
        let state = if request
            .mutation()
            .payload_refs()
            .iter()
            .any(|r| missing.contains(r))
        {
            if matches!(request.mutation(), HistoryMutation::CreateTrial { .. }) {
                RecoveryState::OriginalMissing
            } else {
                RecoveryState::PayloadMissing
            }
        } else if missing.is_empty() {
            RecoveryState::PublishedUncommitted
        } else {
            RecoveryState::AssociationReady
        };
        Ok((state, missing))
    }

    fn check_nodes(
        &self,
        closure: &[BlobRef],
        base: Option<&RootRef>,
        plan: &ObjectPlan,
    ) -> Result<(), HistoryError> {
        let mut nodes: BTreeSet<_> = closure.iter().map(|r| &r.sha256).collect();
        if let Some(root) = base {
            nodes.remove(&root.as_blob().sha256);
        }
        nodes.extend(plan.objects.iter().map(|o| &o.reference.sha256));
        if nodes.len() > 100_000 {
            return Err(error(HistoryErrorKind::LimitExceeded));
        }
        Ok(())
    }

    fn reader(&self, retained: u64) -> Result<HistoryReader<'_>, HistoryError> {
        HistoryReader::with_limits(
            self.storage,
            HistoryLimits {
                max_nodes: 100_000,
                max_total_bytes: self.quota.limit_bytes.max(retained),
            },
        )
    }

    fn current(&self, retained: u64) -> Result<CurrentHistory, HistoryError> {
        let reference = self.head.read_current(self.storage)?;
        self.at(reference, retained)
    }

    fn at(
        &self,
        reference: Option<RootRef>,
        retained: u64,
    ) -> Result<CurrentHistory, HistoryError> {
        read_history_at(&self.reader(retained)?, reference)
    }

    fn receipt(
        current: &CurrentHistory,
        operation: &OperationId,
        request: &Sha256,
        affected: &TrialId,
    ) -> Result<Option<CommittedHistory>, HistoryError> {
        let Some(receipt) = current.document.receipts.get(operation) else {
            return Ok(None);
        };
        conflict(&receipt.request == request && receipt.affected == [affected.clone()])?;
        let root = current
            .reference
            .clone()
            .ok_or_else(|| error(HistoryErrorKind::InvalidDocument))?;
        Ok(Some(CommittedHistory {
            current: root,
            operation: operation.clone(),
            receipt: receipt.clone(),
        }))
    }

    fn unchanged(&self, base: &Option<RootRef>) -> Result<(), HistoryError> {
        recovery::guard(self.storage)?;
        conflict(self.head.read_current(self.storage)? == *base)
    }

    fn missing_bytes(
        inventory: &recovery::Inventory,
        plan: &ObjectPlan,
    ) -> Result<u64, HistoryError> {
        let mut added = 0;
        for object in &plan.objects {
            if let Some(existing) = inventory.objects.get(&object.reference.sha256) {
                related(existing == &object.reference)?;
            } else {
                added = recovery::add(added, object.reference.bytes)?;
            }
        }
        Ok(added)
    }
}

fn publication_reader(storage: &Storage) -> Result<HistoryReader<'_>, HistoryError> {
    HistoryReader::with_limits(
        storage,
        HistoryLimits {
            max_nodes: 100_000,
            max_total_bytes: u64::MAX,
        },
    )
}

fn read_history_at(
    reader: &HistoryReader<'_>,
    reference: Option<RootRef>,
) -> Result<CurrentHistory, HistoryError> {
    let (document, closure) = match &reference {
        Some(reference) => {
            let closure = reader.verify_closure(reference)?.objects;
            let document =
                ReadSession::new(reader).document(reference.as_blob(), 8 * 1024 * 1024)?;
            (document, closure)
        }
        None => (
            HistoryRoot {
                schema_version: 1,
                revision: 0,
                next_save_sequence: 1,
                trials: BTreeMap::new(),
                receipts: BTreeMap::new(),
            },
            vec![],
        ),
    };
    Ok(CurrentHistory {
        reference,
        document,
        closure,
    })
}

/// Shared original-base recomputation; missing leaves remain absent.
fn checked_old_plan_with_reader(
    reader: &HistoryReader<'_>,
    intent: &RecoveryIntent,
    request: &FrozenHistoryRequest,
) -> Result<(mutation::RecoveryPlan, Vec<BlobRef>), HistoryError> {
    conflict(
        intent.request == *request.request_hash()
            && intent.mutation_json == request.mutation_json(),
    )?;
    let base = read_history_at(reader, intent.base.clone())?;
    let plan =
        match mutation::recovery_plan(reader, base.document, &intent.operation, request, true)? {
            mutation::RecoveryPlanOutcome::Planned(plan) => plan,
            mutation::RecoveryPlanOutcome::Duplicate(_) => {
                return Err(error(HistoryErrorKind::RelationshipMismatch))
            }
        };
    related(
        plan.root == intent.planned_root
            && plan
                .objects
                .iter()
                .map(|o| &o.reference)
                .eq(intent.planned_objects.iter()),
    )?;
    let mut missing = vec![];
    let mut reads = ReadSession::new(reader);
    for object in &plan.objects {
        match reads.bytes(&object.reference, DOCUMENT_CAP) {
            Ok(bytes) => {
                if let Some(expected) = &object.bytes {
                    related(bytes.as_slice() == expected.as_ref())?;
                }
            }
            Err(err) if err.kind == HistoryErrorKind::MissingObject => {
                missing.push(object.reference.clone())
            }
            Err(err) => return Err(err),
        }
    }
    if missing.is_empty() {
        reader.verify_closure(&plan.root)?;
    }
    Ok((plan, missing))
}

fn confirm_with_reader_and_head(
    storage: &Storage,
    reader: &HistoryReader<'_>,
    head: &dyn CommittedHistoryHead,
    published: &PublishedHistory,
) -> Result<CommittedHistory, HistoryError> {
    recovery::guard(storage)?;
    published.operation.validate().map_err(contract_error)?;
    published.request.validate().map_err(contract_error)?;
    let planned = read_history_at(reader, Some(published.next.clone()))?;
    let expected = planned
        .document
        .receipts
        .get(&published.operation)
        .ok_or_else(|| error(HistoryErrorKind::Conflict))?;
    conflict(expected.request == published.request)?;
    if let Some((intent, _)) = recovery::read_intent(storage, &published.operation)? {
        conflict(
            intent.request == published.request
                && intent.base == published.base
                && intent.planned_root == published.next,
        )?;
        let request = FrozenHistoryRequest::from_exact(intent.mutation_json.clone(), vec![])?;
        checked_old_plan_with_reader(reader, &intent, &request)?
            .0
            .complete()?;
        conflict(expected.affected == [request.mutation().affected().clone()])?;
    }
    let current = read_history_at(reader, head.read_current(storage)?)?;
    let Some(receipt) = current.document.receipts.get(&published.operation) else {
        return Err(error(if current.reference == published.base {
            HistoryErrorKind::Busy
        } else {
            HistoryErrorKind::Conflict
        }));
    };
    conflict(receipt == expected)?;
    let reference = current
        .reference
        .ok_or_else(|| error(HistoryErrorKind::Conflict))?;
    recovery::guard(storage)?;
    conflict(head.read_current(storage)? == Some(reference.clone()))?;
    Ok(CommittedHistory {
        current: reference,
        operation: published.operation.clone(),
        receipt: receipt.clone(),
    })
}

#[cfg(all(test, windows))]
mod i2_publication_authority_tests {
    use super::*;
    use crate::{
        assets,
        store::{
            i2_gate_tests::{tree, Temp},
            Progress, ProgressGateError, ProgressHistoryHead,
        },
    };
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::{
        fs::{self, OpenOptions},
        os::windows::fs::OpenOptionsExt,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    type Ledger = BTreeMap<PathBuf, Vec<u8>>;
    struct Fixture {
        storage: Storage,
        temp: Temp,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(temp.root.join("neighbor"), b"synthetic neighbor").unwrap();
            fs::write(storage.dir.join("keep.wav"), b"synthetic host media").unwrap();
            Self { storage, temp }
        }
        fn history(&self) -> PathBuf {
            self.storage.dir.join("audio-history")
        }
        fn path(&self, r: &BlobRef) -> PathBuf {
            self.history()
                .join(format!("assets/{}.bin", r.sha256.as_str()))
        }
        fn intent_path(&self, operation: &OperationId) -> PathBuf {
            self.history()
                .join(format!("recovery/{}.json", operation.as_str()))
        }
        fn ledger(&self) -> Ledger {
            tree(&self.temp.root)
        }
        fn put_bytes(&self, bytes: &[u8]) -> BlobRef {
            let r = blob(bytes);
            let path = self.path(&r);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
            r
        }
        fn put(&self, value: &impl Serialize) -> BlobRef {
            self.put_bytes(&serde_json::to_vec(value).unwrap())
        }
        fn seed(&self, trials: &[TrialRevision]) -> (HistoryRoot, RootRef) {
            let mut root = empty_root();
            root.revision = 1;
            for t in trials {
                self.put_bytes(&wav(t.id.as_str().as_bytes()[35]));
                root.trials.insert(
                    t.id.clone(),
                    TrialHead {
                        document: self.put(t),
                        admitted_sequence: root.next_save_sequence,
                    },
                );
                root.next_save_sequence += 1;
            }
            let r = RootRef::new(self.put(&root));
            self.healthy(&r);
            (root, r)
        }
        fn healthy(&self, r: &RootRef) {
            HistoryReader::new(&self.storage)
                .verify_closure(r)
                .expect("hand-built closure must be valid before mutation");
        }
        fn intent(&self, operation: &OperationId, value: &Value) {
            let p = self.intent_path(operation);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, serde_json::to_vec(value).unwrap()).unwrap();
        }
        fn objects(&self, p: &Plan) {
            for bytes in &p.objects {
                self.put_bytes(bytes);
            }
            self.healthy(&p.root_ref());
        }
        fn install(&self, p: &Plan) {
            self.objects(p);
            self.intent(&p.operation, &p.intent());
        }
        fn head(&self, root: Option<&RootRef>) {
            // Independent disk fixture, not a call to a root-preserving writer.
            let raw = format!(
                r#"{{"version":2,"audio_history":{},"settings":{{"topic":"disk sentinel"}},"memories":{{}},"reviews":[],"suspended":[],"ai_calls":{{}},"study_seconds":{{}}}}"#,
                serde_json::to_string(&root).unwrap()
            );
            let p: Progress = serde_json::from_str(&raw).unwrap();
            p.validate().unwrap();
            fs::write(self.storage.dir.join("progress.json"), raw).unwrap();
        }
        fn seeded_plan(&self) -> Plan {
            let (mut base, _) = self.seed(&[trial(1)]);
            base.receipts.insert(
                op(90),
                OperationReceipt {
                    request: digest(b"unrelated old exact request"),
                    committed_revision: 1,
                    affected: vec![tid(1)],
                },
            );
            let base_ref = RootRef::new(self.put(&base));
            self.healthy(&base_ref);
            let p = Plan::create_on(trial(2), op(1), &base, Some(base_ref), None);
            self.install(&p);
            self.head(p.base.as_ref());
            p
        }
        fn verify(&self, published: &PublishedHistory) -> Result<(), HistoryError> {
            let guard = acquire_progress_write().unwrap();
            let r = HistoryWriter::verify_publication_under_guard(&self.storage, published, &guard);
            assert_eq!(
                guard.ensure_writable(),
                Ok(()),
                "verification refusal must not set process block"
            );
            r
        }
        fn public_confirm(
            &mut self,
            published: &PublishedHistory,
            head: &dyn CommittedHistoryHead,
            quota: u64,
        ) -> Result<CommittedHistory, HistoryError> {
            HistoryWriter::new(&mut self.storage, head, HistoryQuota { limit_bytes: quota })
                .unwrap()
                .confirm_committed(published)
        }
    }
    struct Head(Option<RootRef>);
    impl CommittedHistoryHead for Head {
        fn read_current(&self, _: &Storage) -> Result<Option<RootRef>, HistoryError> {
            Ok(self.0.clone())
        }
    }

    fn rejected(f: &Fixture, p: &PublishedHistory, kind: HistoryErrorKind) {
        let before = f.ledger();
        assert_eq!(f.verify(p).unwrap_err().kind, kind);
        assert!(f.ledger() == before, "all file/directory bytes unchanged");
    }
    fn confirmed(f: &mut Fixture, p: &Plan, current: &RootRef) {
        let before = f.ledger();
        let token = p.publication();
        let expected = CommittedHistory {
            current: current.clone(),
            operation: p.operation.clone(),
            receipt: p.root.receipts[&p.operation].clone(),
        };
        assert_eq!(
            f.public_confirm(&token, &ProgressHistoryHead, 1).unwrap(),
            expected,
            "existing public confirmation is the independent positive control"
        );
        assert_eq!(
            HistoryWriter::confirm_publication(&f.storage, &token).unwrap(),
            expected
        );
        assert!(f.ledger() == before, "all file/directory bytes unchanged");
    }
    #[test]
    fn v01_exact_publication_preserves_unrelated_history_and_every_byte() {
        for with_base in [true, false] {
            let mut f = Fixture::new();
            let p = if with_base {
                f.seeded_plan()
            } else {
                let p = Plan::create(trial(2), op(1));
                f.install(&p);
                f.head(None);
                p
            };
            if with_base {
                assert_eq!(p.root.trials.len(), 2);
                assert_eq!(p.root.next_save_sequence, 3);
                assert_eq!(
                    p.root.receipts.keys().cloned().collect::<Vec<_>>(),
                    vec![op(1), op(90)]
                );
            }
            let (intent, _) = recovery::read_intent(&f.storage, &p.operation)
                .unwrap()
                .unwrap();
            let exact = FrozenHistoryRequest::from_exact(p.mutation_json.clone(), vec![]).unwrap();
            {
                let writer =
                    HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1())
                        .unwrap();
                let (plan, missing) = writer.checked_old_plan(&intent, &exact).unwrap();
                assert!(missing.is_empty());
                let actual = plan.complete().unwrap();
                assert_eq!(actual.root, p.root_ref());
                assert_eq!(
                    actual
                        .objects
                        .iter()
                        .map(|o| o.reference.clone())
                        .collect::<Vec<_>>(),
                    p.refs()
                );
            }
            let before = f.ledger();
            assert_eq!(f.verify(&p.publication()), Ok(()));
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            assert_eq!(
                ProgressHistoryHead.read_current(&f.storage).unwrap(),
                p.base
            );
        }
    }
    #[test]
    fn v02_self_consistent_forged_roots_do_not_grant_authority() {
        for mode in 0..3 {
            for align_intent in [false, true] {
                let f = Fixture::new();
                let mut p = f.seeded_plan();
                match mode {
                    0 => {
                        p.root.trials.remove(&tid(1));
                        p.root.receipts.remove(&op(90));
                    }
                    1 => {
                        p.root.receipts.remove(&op(90));
                    }
                    _ => p.root.next_save_sequence += 1,
                }
                *p.objects.last_mut().unwrap() = serde_json::to_vec(&p.root).unwrap();
                f.objects(&p);
                if align_intent {
                    f.intent(&p.operation, &p.intent());
                    recovery::read_intent(&f.storage, &p.operation)
                        .unwrap()
                        .unwrap();
                }
                rejected(
                    &f,
                    &p.publication(),
                    if align_intent {
                        HistoryErrorKind::RelationshipMismatch
                    } else {
                        HistoryErrorKind::Conflict
                    },
                );
            }
        }
    }
    #[test]
    fn v03_public_token_fields_are_bound_to_disk_and_strict_intent() {
        for mode in 0..5 {
            let f = Fixture::new();
            let p = f.seeded_plan();
            let mut token = p.publication();
            match mode {
                0 => {
                    let raw = fs::read(f.path(p.base.as_ref().unwrap().as_blob())).unwrap();
                    let base: HistoryRoot = serde_json::from_slice(&raw).unwrap();
                    let other = Plan::create_on(trial(2), op(2), &base, p.base.clone(), None);
                    f.install(&other);
                    token.operation = op(2);
                }
                1 => token.request = digest(b"other request"),
                2 => token.base = None,
                3 => {
                    token.next = RootRef::new(BlobRef {
                        sha256: token.next.as_blob().sha256.clone(),
                        bytes: token.next.as_blob().bytes + 1,
                    })
                }
                _ => {
                    token.base = Some(RootRef::new(BlobRef {
                        sha256: token.base.as_ref().unwrap().as_blob().sha256.clone(),
                        bytes: token.base.as_ref().unwrap().as_blob().bytes + 1,
                    }))
                }
            };
            rejected(&f, &token, HistoryErrorKind::Conflict);
        }
    }
    #[test]
    fn v03_strict_intent_and_ordered_planned_refs_cannot_be_substituted() {
        for mode in 0..10 {
            let f = Fixture::new();
            let p = f.seeded_plan();
            let mut value = p.intent();
            let kind = match mode {
                0 => {
                    value["schema_version"] = json!(99);
                    HistoryErrorKind::UnsupportedVersion
                }
                1 => {
                    value["unknown"] = json!(true);
                    HistoryErrorKind::InvalidDocument
                }
                2 => {
                    value.as_object_mut().unwrap().remove("base");
                    HistoryErrorKind::InvalidDocument
                }
                3 => {
                    value["operation"] = json!(op(77));
                    HistoryErrorKind::RelationshipMismatch
                }
                4 => {
                    value["mutation_json"] = json!(format!(" {}", p.mutation_json));
                    HistoryErrorKind::RelationshipMismatch
                }
                5 => {
                    value["planned_objects"].as_array_mut().unwrap().swap(0, 1);
                    HistoryErrorKind::RelationshipMismatch
                }
                6 => {
                    value["planned_objects"].as_array_mut().unwrap().remove(1);
                    HistoryErrorKind::RelationshipMismatch
                }
                7 => {
                    value["planned_objects"][1] = json!(f.put_bytes(b"different immutable object"));
                    HistoryErrorKind::RelationshipMismatch
                }
                8 => HistoryErrorKind::LimitExceeded,
                _ => HistoryErrorKind::InvalidDocument,
            };
            f.intent(&p.operation, &value);
            if mode == 8 {
                let mut raw = fs::read(f.intent_path(&p.operation)).unwrap();
                raw.resize(512 * 1024 + 1, b' ');
                fs::write(f.intent_path(&p.operation), raw).unwrap();
            }
            if mode == 9 {
                let raw = fs::read_to_string(f.intent_path(&p.operation))
                    .unwrap()
                    .replacen("{", "{\"schema_version\":1,", 1);
                fs::write(f.intent_path(&p.operation), raw).unwrap();
            }
            if (5..=7).contains(&mode) {
                recovery::read_intent(&f.storage, &p.operation)
                    .expect("structurally valid planned-ref mutation")
                    .unwrap();
            } else {
                assert_eq!(
                    recovery::read_intent(&f.storage, &p.operation)
                        .unwrap_err()
                        .kind,
                    kind,
                    "strict fixture diagnosis"
                );
            }
            rejected(&f, &p.publication(), kind);
        }
    }
    #[test]
    fn v03_exact_whitespace_request_is_accepted_but_semantic_copy_is_not() {
        let f = Fixture::new();
        let first = f.seeded_plan();
        let base: HistoryRoot = serde_json::from_slice(
            &fs::read(f.path(first.base.as_ref().unwrap().as_blob())).unwrap(),
        )
        .unwrap();
        let p = Plan::create_on(
            trial(2),
            op(1),
            &base,
            first.base,
            Some(format!(" \n{}\n ", first.mutation_json)),
        );
        f.install(&p);
        let before = f.ledger();
        assert_eq!(f.verify(&p.publication()), Ok(()));
        assert!(f.ledger() == before, "all file/directory bytes unchanged");
        let mut semantic = p.publication();
        semantic.request = FrozenHistoryRequest::new(create(&p.trial), vec![])
            .unwrap()
            .request_hash()
            .clone();
        assert_ne!(semantic.request, p.publication().request);
        rejected(&f, &semantic, HistoryErrorKind::Conflict);
    }
    #[test]
    fn v04_generated_metadata_missing_is_rejected_even_when_plan_is_complete() {
        for index in [1, 2] {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            let absent = p.refs()[index].clone();
            fs::remove_file(f.path(&absent)).unwrap();
            let before = f.ledger();
            let (intent, _) = recovery::read_intent(&f.storage, &p.operation)
                .unwrap()
                .unwrap();
            let request =
                FrozenHistoryRequest::from_exact(p.mutation_json.clone(), vec![]).unwrap();
            {
                let w =
                    HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1())
                        .unwrap();
                let (plan, missing) = w.checked_old_plan(&intent, &request).unwrap();
                assert_eq!(missing, vec![absent]);
                let complete = plan
                    .complete()
                    .expect("generated bytes exist in memory, not on disk");
                assert_eq!(complete.root, p.root_ref());
                assert_eq!(
                    complete
                        .objects
                        .iter()
                        .map(|o| o.reference.clone())
                        .collect::<Vec<_>>(),
                    p.refs()
                );
            }
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            rejected(&f, &p.publication(), HistoryErrorKind::MissingObject);
        }
    }
    #[test]
    fn v04_external_audio_and_result_missing_are_not_recreated() {
        for result_leaf in [false, true] {
            let mut f = Fixture::new();
            let p = if result_leaf {
                let (base, p) = advice_plan(&f);
                f.install(&p);
                f.head(Some(&base));
                p
            } else {
                f.seeded_plan()
            };
            let absent = p.refs()[0].clone();
            fs::remove_file(f.path(&absent)).unwrap();
            let (intent, _) = recovery::read_intent(&f.storage, &p.operation)
                .unwrap()
                .unwrap();
            let request =
                FrozenHistoryRequest::from_exact(p.mutation_json.clone(), vec![]).unwrap();
            {
                let w =
                    HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1())
                        .unwrap();
                let (plan, missing) = w.checked_old_plan(&intent, &request).unwrap();
                assert_eq!(missing, vec![absent]);
                assert_eq!(
                    plan.complete().unwrap_err().kind,
                    HistoryErrorKind::MissingObject
                );
            }
            rejected(&f, &p.publication(), HistoryErrorKind::MissingObject);
        }
    }
    #[test]
    fn v04_real_read_denial_and_object_corruption_preserve_all_bytes() {
        for mode in 0..4 {
            let f = Fixture::new();
            let p = f.seeded_plan();
            let path = if mode < 2 {
                f.path(&p.refs()[0])
            } else {
                f.intent_path(&p.operation)
            };
            if mode < 2 {
                let mut bytes = fs::read(&path).unwrap();
                if mode == 0 {
                    bytes.push(0);
                } else {
                    bytes[0] ^= 1;
                }
                fs::write(path, bytes).unwrap();
                rejected(&f, &p.publication(), HistoryErrorKind::DamagedObject);
            } else {
                let target = if mode == 2 {
                    path
                } else {
                    f.path(&p.refs()[0])
                };
                let before = f.ledger();
                let held = OpenOptions::new()
                    .read(true)
                    .share_mode(0)
                    .open(&target)
                    .unwrap();
                assert_eq!(fs::read(&target).unwrap_err().raw_os_error(), Some(32));
                let result = f.verify(&p.publication());
                drop(held);
                assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
                assert!(f.ledger() == before, "all file/directory bytes unchanged");
                assert_eq!(
                    f.verify(&p.publication()),
                    Ok(()),
                    "same fixture succeeds after unlocking"
                );
                assert!(f.ledger() == before, "all file/directory bytes unchanged");
                eprintln!("V actual read32 and unlocked success mode={mode}");
            }
        }
    }

    #[test]
    fn v05_pending_head_and_base_failures_are_readonly_and_do_not_block() {
        for mode in 0..6 {
            let f = Fixture::new();
            let p = f.seeded_plan();
            let kind = match mode {
                0 => {
                    fs::write(
                        f.storage.dir.join("pending-material-commit.json"),
                        b"not parsed by history guard",
                    )
                    .unwrap();
                    fs::write(
                        f.intent_path(&p.operation),
                        b"broken intent hidden by earlier guard",
                    )
                    .unwrap();
                    HistoryErrorKind::PendingMaterial
                }
                1 => {
                    fs::create_dir(f.storage.dir.join("pending-material-commit.json")).unwrap();
                    HistoryErrorKind::UnsafePath
                }
                2 => {
                    fs::write(f.storage.dir.join("progress.json"), b"{broken").unwrap();
                    HistoryErrorKind::InvalidDocument
                }
                3 => {
                    let other = RootRef::new(f.put(&empty_root()));
                    f.head(Some(&other));
                    HistoryErrorKind::Conflict
                }
                4 => {
                    let root: HistoryRoot = serde_json::from_slice(
                        &fs::read(f.path(p.base.as_ref().unwrap().as_blob())).unwrap(),
                    )
                    .unwrap();
                    let leaf = &root.trials[&tid(1)].document;
                    let mut raw = fs::read(f.path(leaf)).unwrap();
                    raw[0] ^= 1;
                    fs::write(f.path(leaf), raw).unwrap();
                    HistoryErrorKind::DamagedObject
                }
                _ => {
                    fs::remove_file(f.intent_path(&p.operation)).unwrap();
                    HistoryErrorKind::MissingObject
                }
            };
            rejected(&f, &p.publication(), kind);
            if mode < 2 {
                let before = f.ledger();
                assert_eq!(
                    HistoryWriter::confirm_publication(&f.storage, &p.publication())
                        .unwrap_err()
                        .kind,
                    kind
                );
                assert!(f.ledger() == before, "all file/directory bytes unchanged");
            }
        }
    }
    #[test]
    fn v05_strict_intent_paths_and_asset_junctions_are_not_followed() {
        for mode in 0..2 {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            if mode == 0 {
                fs::remove_file(f.intent_path(&p.operation)).unwrap();
                fs::create_dir(f.intent_path(&p.operation)).unwrap();
            } else {
                let original = f.history().join("assets");
                let target = f.temp.root.join("owned-assets-target");
                fs::rename(&original, &target).unwrap();
                f.temp.junction(&original, &target);
            }
            rejected(&f, &p.publication(), HistoryErrorKind::UnsafePath);
            let before = f.ledger();
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &p.publication())
                    .unwrap_err()
                    .kind,
                HistoryErrorKind::UnsafePath
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
        }
    }
    #[test]
    fn v06_v07_committed_next_or_later_root_survives_cleanup_without_new_authority() {
        for later in [false, true] {
            for cleanup in [false, true] {
                let mut f = Fixture::new();
                let p = f.seeded_plan();
                let current = if later {
                    let next = Plan::create_on(trial(3), op(3), &p.root, Some(p.root_ref()), None);
                    f.install(&next);
                    next.root_ref()
                } else {
                    p.root_ref()
                };
                f.head(Some(&current));
                if cleanup {
                    fs::remove_file(f.intent_path(&p.operation)).unwrap();
                }
                confirmed(&mut f, &p, &current);
                rejected(&f, &p.publication(), HistoryErrorKind::Conflict);
                let committed = CommittedHistory {
                    current: current.clone(),
                    operation: p.operation.clone(),
                    receipt: p.root.receipts[&p.operation].clone(),
                };
                let mut expected = f.ledger();
                expected.remove(Path::new(&format!(
                    "host/audio-history/recovery/{}.json",
                    p.operation.as_str()
                )));
                {
                    let mut writer = HistoryWriter::new(
                        &mut f.storage,
                        &ProgressHistoryHead,
                        HistoryQuota::v1(),
                    )
                    .unwrap();
                    writer.finish(&committed).unwrap();
                    writer.finish(&committed).unwrap();
                }
                assert!(
                    f.ledger() == expected,
                    "only specific intent cleanup allowed"
                );
                confirmed(&mut f, &p, &current);
            }
        }
    }
    #[test]
    fn v06_reused_operation_receipt_cannot_be_overwritten_by_a_new_valid_mutation() {
        let f = Fixture::new();
        let p = f.seeded_plan();
        f.head(Some(&p.root_ref()));
        let reused = Plan::create_on(
            trial(3),
            p.operation.clone(),
            &p.root,
            Some(p.root_ref()),
            None,
        );
        f.install(&reused);
        recovery::read_intent(&f.storage, &reused.operation)
            .unwrap()
            .unwrap();
        rejected(&f, &reused.publication(), HistoryErrorKind::Conflict);
    }
    #[test]
    fn v07_disk_confirm_distinguishes_base_busy_from_other_head_conflict() {
        for mode in 0..2 {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            if mode == 1 {
                let empty = RootRef::new(f.put(&empty_root()));
                f.head(Some(&empty));
            }
            let kind = if mode == 0 {
                HistoryErrorKind::Busy
            } else {
                HistoryErrorKind::Conflict
            };
            let before = f.ledger();
            assert_eq!(
                f.public_confirm(&p.publication(), &ProgressHistoryHead, 1)
                    .unwrap_err()
                    .kind,
                kind
            );
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &p.publication())
                    .unwrap_err()
                    .kind,
                kind
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
        }
    }
    #[test]
    fn v07_confirm_checks_receipt_content_remaining_intent_and_latest_closure() {
        for mode in 0..8 {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            f.head(Some(&p.root_ref()));
            let kind = if mode < 3 {
                let mut root = p.root.clone();
                let receipt = root.receipts.get_mut(&p.operation).unwrap();
                match mode {
                    0 => receipt.request = digest(b"different receipt request"),
                    1 => receipt.affected = vec![tid(1)],
                    _ => receipt.committed_revision = 1,
                };
                let r = RootRef::new(f.put(&root));
                f.healthy(&r);
                f.head(Some(&r));
                HistoryErrorKind::Conflict
            } else {
                match mode {
                    3 => {
                        fs::write(f.intent_path(&p.operation), b"{bad").unwrap();
                        HistoryErrorKind::InvalidDocument
                    }
                    4 => {
                        let mut raw = fs::read(f.path(p.root_ref().as_blob())).unwrap();
                        raw[0] ^= 1;
                        fs::write(f.path(p.root_ref().as_blob()), raw).unwrap();
                        HistoryErrorKind::DamagedObject
                    }
                    5 => {
                        let later =
                            Plan::create_on(trial(3), op(3), &p.root, Some(p.root_ref()), None);
                        f.install(&later);
                        f.head(Some(&later.root_ref()));
                        fs::remove_file(f.path(&later.refs()[0])).unwrap();
                        HistoryErrorKind::MissingObject
                    }
                    6 => {
                        let mut token = p.publication();
                        token.request = digest(b"wrong public request");
                        let before = f.ledger();
                        assert_eq!(
                            f.public_confirm(&token, &ProgressHistoryHead, 1)
                                .unwrap_err()
                                .kind,
                            HistoryErrorKind::Conflict
                        );
                        assert_eq!(
                            HistoryWriter::confirm_publication(&f.storage, &token)
                                .unwrap_err()
                                .kind,
                            HistoryErrorKind::Conflict
                        );
                        assert!(f.ledger() == before, "all file/directory bytes unchanged");
                        continue;
                    }
                    _ => {
                        let mut intent = p.intent();
                        intent["schema_version"] = json!(99);
                        f.intent(&p.operation, &intent);
                        HistoryErrorKind::UnsupportedVersion
                    }
                }
            };
            let before = f.ledger();
            assert_eq!(
                f.public_confirm(&p.publication(), &ProgressHistoryHead, 1)
                    .unwrap_err()
                    .kind,
                kind,
                "existing public control mode {mode}"
            );
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &p.publication())
                    .unwrap_err()
                    .kind,
                kind
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
        }
    }
    #[test]
    fn v08_custom_head_remains_public_only_and_new_confirm_reads_disk() {
        let mut f = Fixture::new();
        let p = f.seeded_plan();
        let later = Plan::create_on(trial(3), op(3), &p.root, Some(p.root_ref()), None);
        f.install(&later);
        let custom = Head(Some(later.root_ref()));
        let before = f.ledger();
        let old = f.public_confirm(&p.publication(), &custom, 1).unwrap();
        assert_eq!(old.current, later.root_ref());
        assert_eq!(old.receipt, p.root.receipts[&p.operation]);
        assert_eq!(
            HistoryWriter::confirm_publication(&f.storage, &p.publication())
                .unwrap_err()
                .kind,
            HistoryErrorKind::Busy
        );
        assert!(f.ledger() == before, "all file/directory bytes unchanged");
        f.head(Some(&p.root_ref()));
        let before = f.ledger();
        let actual = HistoryWriter::confirm_publication(&f.storage, &p.publication()).unwrap();
        assert_eq!(actual.current, p.root_ref());
        assert_ne!(actual.current, old.current);
        assert_eq!(f.public_confirm(&p.publication(), &custom, 1).unwrap(), old);
        assert!(f.ledger() == before, "all file/directory bytes unchanged");
    }
    #[test]
    fn v08_retained_reader_quota_and_exact_root_document_cap_are_preserved() {
        for extra in [0, 1] {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            fs::remove_file(f.intent_path(&p.operation)).unwrap();
            let mut raw = serde_json::to_vec(&p.root).unwrap();
            raw.resize(8 * 1024 * 1024 + extra, b' ');
            let next = RootRef::new(f.put_bytes(&raw));
            let mut token = p.publication();
            token.next = next.clone();
            if extra == 0 {
                f.head(Some(&next));
            } else {
                // The oversized next belongs only to the public token, not disk Progress.
                assert_eq!(
                    ProgressHistoryHead.read_current(&f.storage).unwrap(),
                    p.base
                );
                f.healthy(p.base.as_ref().unwrap());
            }
            let before = f.ledger();
            {
                let writer = HistoryWriter::new(
                    &mut f.storage,
                    &ProgressHistoryHead,
                    HistoryQuota { limit_bytes: 1 },
                )
                .unwrap();
                let reader = writer.reader(u64::MAX).unwrap();
                assert_eq!(reader.limits.max_nodes, 100_000);
                assert_eq!(reader.limits.max_total_bytes, u64::MAX);
            }
            let old = f.public_confirm(&token, &ProgressHistoryHead, 1);
            let new = HistoryWriter::confirm_publication(&f.storage, &token);
            if extra == 0 {
                let expected = CommittedHistory {
                    current: next,
                    operation: p.operation.clone(),
                    receipt: p.root.receipts[&p.operation].clone(),
                };
                assert_eq!(old.unwrap(), expected);
                assert_eq!(new.unwrap(), expected);
            } else {
                assert_eq!(old.unwrap_err().kind, HistoryErrorKind::LimitExceeded);
                assert_eq!(new.unwrap_err().kind, HistoryErrorKind::LimitExceeded);
            }
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
        }
    }

    fn isolated(case: &str, body: impl FnOnce()) {
        const ENV: &str = "WW5_I2_V_CHILD";
        if std::env::var(ENV).as_deref() == Ok(case) {
            body();
            println!("WW5_V_CHILD_OK:{case}");
            return;
        }
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([case, "--exact", "--nocapture", "--test-threads=1"])
            .env(ENV, case)
            .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut timed_out = false;
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                timed_out = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let out = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !timed_out,
            "child timeout killed/reaped\n{stdout}\n{stderr}"
        );
        assert!(out.status.success(), "{stdout}\n{stderr}");
        assert!(
            stdout.contains("running 1 test")
                && stdout.contains("1 passed; 0 failed")
                && stdout.contains(&format!("WW5_V_CHILD_OK:{case}"))
        );
    }
    #[test]
    fn v09_held_guard_verification_and_both_confirms_do_not_reacquire_gate() {
        const CASE:&str="audio_history::store::i2_publication_authority_tests::v09_held_guard_verification_and_both_confirms_do_not_reacquire_gate";
        isolated(CASE, || {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            let guard = acquire_progress_write().unwrap();
            let before = f.ledger();
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::Reentrant
            );
            assert_eq!(
                HistoryWriter::verify_publication_under_guard(&f.storage, &p.publication(), &guard),
                Ok(())
            );
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &p.publication())
                    .unwrap_err()
                    .kind,
                HistoryErrorKind::Busy
            );
            assert_eq!(
                f.public_confirm(&p.publication(), &ProgressHistoryHead, 1)
                    .unwrap_err()
                    .kind,
                HistoryErrorKind::Busy
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            f.head(Some(&p.root_ref()));
            confirmed(&mut f, &p, &p.root_ref());
            assert_eq!(guard.ensure_writable(), Ok(()));
            drop(guard);
            assert!(acquire_progress_write().is_ok());
        });
    }
    #[test]
    fn v09_write_block_rejects_verify_but_leaves_both_confirms_pure() {
        const CASE:&str="audio_history::store::i2_publication_authority_tests::v09_write_block_rejects_verify_but_leaves_both_confirms_pure";
        isolated(CASE, || {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            let guard = acquire_progress_write().unwrap();
            let before = f.ledger();
            assert_eq!(
                HistoryWriter::verify_publication_under_guard(&f.storage, &p.publication(), &guard),
                Ok(())
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            guard.block_writes();
            let before = f.ledger();
            assert_eq!(
                HistoryWriter::verify_publication_under_guard(&f.storage, &p.publication(), &guard)
                    .unwrap_err()
                    .kind,
                HistoryErrorKind::Io
            );
            assert_eq!(
                guard.ensure_writable(),
                Err(ProgressGateError::WriteBlocked)
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            f.head(Some(&p.root_ref()));
            confirmed(&mut f, &p, &p.root_ref());
            assert_eq!(
                guard.ensure_writable(),
                Err(ProgressGateError::WriteBlocked)
            );
            drop(guard);
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::WriteBlocked
            );
            confirmed(&mut f, &p, &p.root_ref());
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::WriteBlocked
            );
        });
    }
    #[test]
    fn v09_poison_prevents_guard_acquisition_without_preventing_readonly_confirmation() {
        const CASE:&str="audio_history::store::i2_publication_authority_tests::v09_poison_prevents_guard_acquisition_without_preventing_readonly_confirmation";
        isolated(CASE, || {
            let mut f = Fixture::new();
            let p = f.seeded_plan();
            f.head(Some(&p.root_ref()));
            let before = f.ledger();
            let guard = acquire_progress_write().unwrap();
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                let _held = guard;
                panic!("isolated intentional G poison");
            }));
            assert!(panic.is_err());
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::Poisoned
            );
            confirmed(&mut f, &p, &p.root_ref());
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::Poisoned
            );
            assert!(f.ledger() == before, "all file/directory bytes unchanged");
            // No valid ProgressWriteGuard exists after poison: do not pretend
            // verify was directly invoked with an impossible poisoned guard.
        });
    }

    fn digest(bytes: &[u8]) -> Sha256 {
        Sha256::parse(&assets::sha256(bytes).unwrap()).unwrap()
    }
    fn blob(bytes: &[u8]) -> BlobRef {
        BlobRef {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
        }
    }
    fn op(n: u64) -> OperationId {
        OperationId::parse(&format!("33333333-3333-4333-8333-{n:012x}")).unwrap()
    }
    fn tid(n: u64) -> TrialId {
        TrialId::parse(&format!("44444444-4444-4444-8444-{n:012x}")).unwrap()
    }
    fn at() -> RecordedAt {
        RecordedAt {
            utc_millis: 1_700_000_000_000,
            offset_minutes: 540,
        }
    }
    fn wav(sample: u8) -> Vec<u8> {
        let mut b = b"RIFF".to_vec();
        b.extend(38u32.to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(8000u32.to_le_bytes());
        b.extend(16000u32.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(2u32.to_le_bytes());
        b.extend(i16::from(sample).to_le_bytes());
        b
    }
    fn trial(n: u64) -> TrialRevision {
        let id = tid(n);
        TrialRevision {
            schema_version: 1,
            id: id.clone(),
            revision: 1,
            previous: None,
            created_at: at(),
            input: FrozenInput {
                audio: AudioSnapshot {
                    original: blob(&wav(id.as_str().as_bytes()[35])),
                    format: AudioFormat::Wav,
                    rate: 8000,
                    channels: 1,
                    frames: 1,
                    origin: Origin::Imported,
                    warnings: vec![],
                },
                reference: ReferenceSnapshot {
                    text: "Keep this sentence.\n復元".into(),
                    sha256: digest("Keep this sentence.\n復元".as_bytes()),
                    source: None,
                },
                goal: None,
                capture: CaptureProfile {
                    device: None,
                    distance: None,
                    gain: None,
                    environment: None,
                    confirmed_at: None,
                    version: digest(
                        br#"{"device":null,"distance":null,"gain":null,"environment":null}"#,
                    ),
                },
            },
            claim: PracticeClaim::Reference,
            match_evidence: vec![],
            analysis: BTreeMap::new(),
            advice: BTreeMap::new(),
            reflections: vec![],
            trashed_at: None,
        }
    }
    fn create(t: &TrialRevision) -> HistoryMutation {
        HistoryMutation::CreateTrial {
            id: t.id.clone(),
            created_at: t.created_at,
            input: t.input.clone(),
            claim: t.claim.clone(),
            duplicate: None,
        }
    }
    fn target(t: &TrialRevision) -> TrialTarget {
        TrialTarget {
            id: t.id.clone(),
            expected: blob(&serde_json::to_vec(t).unwrap()),
        }
    }
    fn empty_root() -> HistoryRoot {
        HistoryRoot {
            schema_version: 1,
            revision: 0,
            next_save_sequence: 1,
            trials: BTreeMap::new(),
            receipts: BTreeMap::new(),
        }
    }

    fn advice_result() -> AdviceResult {
        AdviceResult {
            schema_version: 1,
            assessment: AdviceAssessment::Assessed,
            heard_text: Some("Keep this sentence.".into()),
            summary: "Synthetic completed result".into(),
            strengths: vec![],
            improvements: vec![],
            unassessable_reason: None,
            requested_model: "fixture".into(),
            requested_effort: "fixture".into(),
            actual_model: None,
            actual_effort: None,
            usage: None,
        }
    }
    fn advice_plan(f: &Fixture) -> (RootRef, Plan) {
        let mut t = trial(1);
        let mut run = AdviceRun {
            schema_version: 1,
            id: RunId::parse("55555555-5555-4555-8555-000000000001").unwrap(),
            trial: t.id.clone(),
            audio: t.input.audio.original.sha256.clone(),
            reference: t.input.reference.sha256.clone(),
            confirmed_at: at(),
            confirmation: digest(b"visible synthetic confirmation"),
            destination: "https://fixture.invalid".into(),
            versions: AdviceVersions {
                provider: "fixture".into(),
                prompt: "1".into(),
                parser: "1".into(),
            },
            state: AdviceRunState::Sending,
            send: SendCertainty::MayHaveBeenSent,
            started_at: at(),
            ended_at: None,
            result: None,
            failure: None,
        };
        t.advice.insert(run.id.clone(), f.put(&run));
        let (root, base) = f.seed(&[t.clone()]);
        let result_bytes = serde_json::to_vec(&advice_result()).unwrap();
        run.state = AdviceRunState::Assessed;
        run.send = SendCertainty::ResponseReceived;
        run.ended_at = Some(at());
        run.result = Some(blob(&result_bytes));
        let mutation = HistoryMutation::AddAdviceRun {
            target: target(&t),
            run: run.clone(),
        };
        let mut next = t.clone();
        next.revision += 1;
        next.previous = Some(target(&t).expected);
        next.advice
            .insert(run.id.clone(), blob(&serde_json::to_vec(&run).unwrap()));
        let p = Plan::update(
            next,
            &root,
            base.clone(),
            op(1),
            mutation,
            vec![result_bytes, serde_json::to_vec(&run).unwrap()],
        );
        (base, p)
    }

    // This is an independent schema oracle, not a call to writer planning.
    struct Plan {
        operation: OperationId,
        base: Option<RootRef>,
        mutation_json: String,
        trial: TrialRevision,
        root: HistoryRoot,
        objects: Vec<Vec<u8>>,
    }
    impl Plan {
        fn create(t: TrialRevision, operation: OperationId) -> Self {
            Self::create_on(t, operation, &empty_root(), None, None)
        }
        fn create_on(
            t: TrialRevision,
            operation: OperationId,
            base_root: &HistoryRoot,
            base: Option<RootRef>,
            exact: Option<String>,
        ) -> Self {
            let mutation_json =
                exact.unwrap_or_else(|| serde_json::to_string(&create(&t)).unwrap());
            let mut root = base_root.clone();
            root.revision += 1;
            root.trials.insert(
                t.id.clone(),
                TrialHead {
                    document: blob(&serde_json::to_vec(&t).unwrap()),
                    admitted_sequence: root.next_save_sequence,
                },
            );
            root.next_save_sequence += 1;
            root.receipts.insert(
                operation.clone(),
                OperationReceipt {
                    request: digest(mutation_json.as_bytes()),
                    committed_revision: root.revision,
                    affected: vec![t.id.clone()],
                },
            );
            let objects = vec![
                wav(t.id.as_str().as_bytes()[35]),
                serde_json::to_vec(&t).unwrap(),
                serde_json::to_vec(&root).unwrap(),
            ];
            Self {
                operation,
                base,
                mutation_json,
                trial: t,
                root,
                objects,
            }
        }
        fn update(
            t: TrialRevision,
            root: &HistoryRoot,
            base: RootRef,
            operation: OperationId,
            mutation: HistoryMutation,
            mut leaves: Vec<Vec<u8>>,
        ) -> Self {
            let mutation_json = serde_json::to_string(&mutation).unwrap();
            let mut root = root.clone();
            root.revision += 1;
            root.trials.get_mut(&t.id).unwrap().document = blob(&serde_json::to_vec(&t).unwrap());
            root.receipts.insert(
                operation.clone(),
                OperationReceipt {
                    request: digest(mutation_json.as_bytes()),
                    committed_revision: root.revision,
                    affected: vec![t.id.clone()],
                },
            );
            leaves.push(serde_json::to_vec(&t).unwrap());
            leaves.push(serde_json::to_vec(&root).unwrap());
            Self {
                operation,
                base: Some(base),
                mutation_json,
                trial: t,
                root,
                objects: leaves,
            }
        }
        fn root_ref(&self) -> RootRef {
            RootRef::new(blob(&serde_json::to_vec(&self.root).unwrap()))
        }
        fn refs(&self) -> Vec<BlobRef> {
            self.objects.iter().map(|b| blob(b)).collect()
        }
        fn intent(&self) -> Value {
            json!({"schema_version":1,"operation":self.operation,"request":digest(self.mutation_json.as_bytes()),
            "mutation_json":self.mutation_json,"base":self.base,"planned_root":self.root_ref(),"planned_objects":self.refs()})
        }
        fn publication(&self) -> PublishedHistory {
            PublishedHistory {
                base: self.base.clone(),
                next: self.root_ref(),
                operation: self.operation.clone(),
                request: digest(self.mutation_json.as_bytes()),
            }
        }
    }
}
