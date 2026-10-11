//! I1b-2b independent recovery oracles. All roots/intents are hand-built.
//! Real isolated local files; the only doubles are host-head and I/O fault seams.
#![cfg(windows)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use wordweave5::{assets, audio_history::*, store::Storage};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Fixture {
    storage: Storage,
    _temp: Temp,
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "ww5-recovery-tests-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        Self {
            storage: Storage::at(dir.join("host")).unwrap(),
            _temp: Temp(dir),
        }
    }
    fn history(&self) -> PathBuf {
        self.storage.dir.join("audio-history")
    }
    fn put_bytes(&self, bytes: &[u8]) -> BlobRef {
        let reference = blob(bytes);
        let path = object_path(&self.history(), &reference);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
        reference
    }
    fn put(&self, document: &impl Serialize) -> BlobRef {
        self.put_bytes(&serde_json::to_vec(document).unwrap())
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
        let reference = RootRef::new(self.put(&root));
        HistoryReader::new(&self.storage)
            .verify_closure(&reference)
            .expect("independent starting closure must be valid");
        (root, reference)
    }
    fn install(&self, plan: &Plan, published_prefix: usize) {
        self.intent(&plan.operation, &plan.intent());
        for bytes in plan.objects.iter().take(published_prefix) {
            self.put_bytes(bytes);
        }
    }
    fn intent(&self, operation: &OperationId, intent: &Value) {
        let path = intent_path(&self.history(), operation);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(intent).unwrap()).unwrap();
    }
    fn reopen(&mut self) {
        // A replacement isolated lock releases the actual host lock before reopen.
        let placeholder = Storage::at(self._temp.0.join("placeholder")).unwrap();
        let old = std::mem::replace(&mut self.storage, placeholder);
        let path = old.dir.clone();
        drop(old);
        self.storage = Storage::at(path).unwrap();
    }
}
#[derive(Default)]
struct Head {
    current: RefCell<Option<RootRef>>,
    fail: Cell<bool>,
}
impl Head {
    fn at(root: RootRef) -> Self {
        Self {
            current: RefCell::new(Some(root)),
            fail: Cell::new(false),
        }
    }
}
impl CommittedHistoryHead for Head {
    fn read_current(&self, _: &Storage) -> Result<Option<RootRef>, HistoryError> {
        if self.fail.get() {
            Err(HistoryError {
                kind: HistoryErrorKind::Io,
            })
        } else {
            Ok(self.current.borrow().clone())
        }
    }
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
        let mutation_json = exact.unwrap_or_else(|| serde_json::to_string(&create(&t)).unwrap());
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
    fn reflection(
        t: &TrialRevision,
        root: &HistoryRoot,
        base: RootRef,
        operation: OperationId,
    ) -> Self {
        let note = Reflection {
            schema_version: 1,
            trial: t.id.clone(),
            at: at(),
            text: "Keep earlier result. 日本語".into(),
        };
        let mutation = HistoryMutation::AddReflection {
            target: target(t),
            reflection: note.clone(),
        };
        let mut next = t.clone();
        next.revision += 1;
        next.previous = Some(target(t).expected);
        next.reflections
            .push(blob(&serde_json::to_vec(&note).unwrap()));
        Self::update(
            next,
            root,
            base,
            operation,
            mutation,
            vec![serde_json::to_vec(&note).unwrap()],
        )
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
fn object_path(history: &Path, r: &BlobRef) -> PathBuf {
    history
        .join("assets")
        .join(format!("{}.bin", r.sha256.as_str()))
}
fn intent_path(history: &Path, operation: &OperationId) -> PathBuf {
    history
        .join("recovery")
        .join(format!("{}.json", operation.as_str()))
}
fn read<T: DeserializeOwned>(history: &Path, reference: &BlobRef) -> T {
    serde_json::from_slice(&fs::read(object_path(history, reference)).unwrap()).unwrap()
}
fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !path.exists() {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            let relative = p.strip_prefix(base).unwrap().to_path_buf();
            if p.is_dir() {
                out.insert(relative, vec![]);
                visit(base, &p, out);
            } else {
                out.insert(relative, fs::read(p).unwrap());
            }
        }
    }
    let mut map = BTreeMap::new();
    visit(path, path, &mut map);
    map
}
fn assert_preserved(path: &Path, before: &BTreeMap<PathBuf, Vec<u8>>, except: Option<&Path>) {
    let after = tree(path);
    for (p, bytes) in before {
        if Some(p.as_path()) != except {
            assert_eq!(after.get(p), Some(bytes), "old bytes changed: {p:?}");
        }
    }
}
fn ready(outcome: Result<PrepareOutcome, HistoryError>) -> PreparedHistory {
    match outcome.unwrap() {
        PrepareOutcome::Ready(p) => p,
        other => panic!("expected Ready: {other:?}"),
    }
}
fn published(outcome: Result<PublishOutcome, HistoryError>) -> PublishedHistory {
    match outcome.unwrap() {
        PublishOutcome::Published(p) => p,
        other => panic!("expected Published: {other:?}"),
    }
}
fn kind<T: std::fmt::Debug>(result: Result<T, HistoryError>, expected: HistoryErrorKind) {
    assert_eq!(result.unwrap_err().kind, expected);
}
fn candidate<'a>(view: &'a RecoveryView, operation: &OperationId) -> &'a RecoveryCandidate {
    let matching: Vec<_> = view
        .candidates
        .iter()
        .filter(|c| &c.operation == operation)
        .collect();
    assert_eq!(matching.len(), 1, "exactly one candidate: {view:?}");
    matching[0]
}
fn assert_plan(history: &Path, actual: &PublishedHistory, expected: &Plan) {
    assert_eq!(actual, &expected.publication());
    let root: HistoryRoot = read(history, actual.next.as_blob());
    assert_eq!(root, expected.root);
    let t: TrialRevision = read(history, &root.trials[&expected.trial.id].document);
    assert_eq!(t, expected.trial);
    for (reference, bytes) in expected.refs().iter().zip(&expected.objects) {
        assert_eq!(fs::read(object_path(history, reference)).unwrap(), *bytes);
    }
}

#[test]
fn handwritten_create_and_update_oracles_are_valid_without_writer() {
    let f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, p.objects.len());
    HistoryReader::new(&f.storage)
        .verify_closure(&p.root_ref())
        .unwrap();
    let u = Plan::reflection(&p.trial, &p.root, p.root_ref(), op(2));
    f.install(&u, u.objects.len());
    HistoryReader::new(&f.storage)
        .verify_closure(&u.root_ref())
        .unwrap();
    assert_plan(&f.history(), &u.publication(), &u);
    let result_fixture = Fixture::new();
    let (_, p) = advice_plan(&result_fixture);
    result_fixture.install(&p, p.objects.len());
    HistoryReader::new(&result_fixture.storage)
        .verify_closure(&p.root_ref())
        .unwrap();
}

#[test]
fn w03_missing_result_requires_saved_bytes_and_never_restarts_the_sending_run() {
    let mut f = Fixture::new();
    let (base, p) = advice_plan(&f);
    f.install(&p, 0);
    f.reopen();
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(base);
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    let c = candidate(&view, &op(1));
    assert_eq!(c.state, RecoveryState::PayloadMissing);
    assert!(c.missing.contains(&p.refs()[0]));
    kind(
        w.read_recovery_request(&op(1), vec![]),
        HistoryErrorKind::MissingObject,
    );
    assert_eq!(tree(&history), before);
    let result: Arc<[u8]> = p.objects[0].clone().into();
    let req = w
        .read_recovery_request(&op(1), vec![result.clone()])
        .unwrap();
    let prepared = ready(w.resume(&op(1), &req));
    assert_plan(&history, &published(w.publish(&prepared)), &p);
    assert_preserved(&history, &before, None);
    assert_eq!(result.as_ref(), p.objects[0]);
    drop(w);
    let run_id = p.trial.advice.keys().next().unwrap();
    let view = HistoryReader::new(&f.storage)
        .read_advice(&p.trial, run_id)
        .unwrap();
    assert_eq!(view.run.state, AdviceRunState::Assessed);
    assert_eq!(view.result, Some(advice_result()));
}

#[test]
fn w03_existing_result_blob_resumes_without_caller_payload_and_keeps_old_sending_revision() {
    let mut f = Fixture::new();
    let (base, p) = advice_plan(&f);
    f.install(&p, 1);
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(base);
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    let prepared = ready(w.resume(&op(1), &req));
    assert_plan(&history, &published(w.publish(&prepared)), &p);
    assert_preserved(&history, &before, None);
    let old: TrialRevision = read(&history, p.trial.previous.as_ref().unwrap());
    let old_run: AdviceRun = read(&history, old.advice.values().next().unwrap());
    assert_eq!(old_run.state, AdviceRunState::Sending);
    assert!(old_run.result.is_none());
}

#[test]
fn w08_empty_recovery_reads_do_not_create_directories() {
    let mut f = Fixture::new();
    let head = Head::default();
    let history = f.history();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert!(view.candidates.is_empty());
    assert!(view.diagnostics.is_empty());
    assert!(view.next_after.is_none());
    kind(
        w.read_recovery_request(&op(1), vec![]),
        HistoryErrorKind::MissingObject,
    );
    assert!(!history.exists());
}

#[test]
fn w02_missing_original_requires_exact_caller_bytes_and_never_synthesizes_audio() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 0);
    f.reopen();
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    let c = candidate(&view, &op(1));
    assert_eq!(c.state, RecoveryState::OriginalMissing);
    assert!(c.missing.contains(&p.refs()[0]));
    kind(
        w.read_recovery_request(&op(1), vec![]),
        HistoryErrorKind::MissingObject,
    );
    assert!(w
        .read_recovery_request(&op(1), vec![Arc::from(wav(99))])
        .is_err());
    assert_eq!(tree(&history), before);
    let caller: Arc<[u8]> = p.objects[0].clone().into();
    let req = w
        .read_recovery_request(&op(1), vec![caller.clone()])
        .unwrap();
    let prepared = ready(w.resume(&op(1), &req));
    assert_plan(&history, &published(w.publish(&prepared)), &p);
    assert_eq!(caller.as_ref(), p.objects[0]);
    assert!(head.current.borrow().is_none());
}

#[test]
fn w02_original_only_restarts_and_restores_same_trial_without_payload_or_new_id() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    f.reopen();
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(
        candidate(&view, &op(1)).state,
        RecoveryState::AssociationReady
    );
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    assert_eq!(req.mutation_json(), p.mutation_json);
    assert_eq!(tree(&history), before);
    let prepared = ready(w.resume(&op(1), &req));
    let publication = published(w.publish(&prepared));
    assert_plan(&history, &publication, &p);
    assert_preserved(&history, &before, None);
    kind(w.confirm_committed(&publication), HistoryErrorKind::Busy);
}

#[test]
fn w02_published_root_is_not_commit_and_restart_reuses_exact_objects() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    f.reopen();
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(
        candidate(&view, &op(1)).state,
        RecoveryState::PublishedUncommitted
    );
    assert!(candidate(&view, &op(1)).missing.is_empty());
    kind(
        w.confirm_committed(&p.publication()),
        HistoryErrorKind::Busy,
    );
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    let prepared = ready(w.resume(&op(1), &req));
    assert_eq!(prepared.quota_plan().added_object_bytes, 0);
    assert_plan(&history, &published(w.publish(&prepared)), &p);
    assert_eq!(tree(&history), before);
    assert!(head.current.borrow().is_none());
}

#[test]
fn w03_exact_mutation_whitespace_survives_restart_and_semantic_copy_conflicts() {
    let mut f = Fixture::new();
    let t = trial(1);
    let exact = format!(
        " \n{}\t",
        serde_json::to_string_pretty(&create(&t)).unwrap()
    );
    let p = Plan::create_on(t.clone(), op(1), &empty_root(), None, Some(exact.clone()));
    f.install(&p, 1);
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let canonical = FrozenHistoryRequest::new(create(&t), vec![]).unwrap();
    kind(w.resume(&op(1), &canonical), HistoryErrorKind::Conflict);
    assert_eq!(tree(&history), before);
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    assert_eq!(req.mutation_json(), exact);
    assert_eq!(req.request_hash(), &digest(exact.as_bytes()));
    assert_eq!(req.mutation(), &create(&t));
    let prepared = ready(w.resume(&op(1), &req));
    assert_plan(&history, &published(w.publish(&prepared)), &p);
}

#[test]
fn w03_committed_receipt_and_later_root_are_idempotent_and_cleanup_is_specific() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    let later = Plan::create_on(trial(2), op(2), &p.root, Some(p.root_ref()), None);
    f.install(&later, 3);
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(later.root_ref());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    assert_eq!(
        candidate(&w.recover(None), &op(1)).state,
        RecoveryState::AlreadyCommitted
    );
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    let resumed = match w.resume(&op(1), &req).unwrap() {
        PrepareOutcome::AlreadyCommitted(c) => c,
        other => panic!("unexpected {other:?}"),
    };
    let committed = w.confirm_committed(&p.publication()).unwrap();
    assert_eq!(committed, resumed);
    assert_eq!(committed.current, later.root_ref());
    assert_eq!(committed.receipt, p.root.receipts[&op(1)]);
    assert_eq!(tree(&history), before);
    w.finish(&committed).unwrap();
    assert!(!intent_path(&history, &op(1)).exists());
    assert!(intent_path(&history, &op(2)).exists());
    assert_preserved(
        &history,
        &before,
        Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
    );
    w.finish(&committed).unwrap(); // cleanup retry is also idempotent
    let root: HistoryRoot = read(&history, later.root_ref().as_blob());
    assert_eq!(root, later.root);
}

#[test]
fn w06_create_rebases_onto_later_unrelated_trial_and_preserves_old_plan_objects() {
    let mut f = Fixture::new();
    let old = Plan::create(trial(1), op(1));
    f.install(&old, 3);
    let (current, current_ref) = f.seed(&[trial(2)]);
    let expected = Plan::create_on(trial(1), op(1), &current, Some(current_ref.clone()), None);
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(current_ref);
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    let prepared = ready(w.resume(&op(1), &req));
    let out = published(w.publish(&prepared));
    assert_plan(&history, &out, &expected);
    assert_preserved(
        &history,
        &before,
        Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
    );
    let intent: Value =
        serde_json::from_slice(&fs::read(intent_path(&history, &op(1))).unwrap()).unwrap();
    assert_eq!(intent, expected.intent());
}

#[test]
fn w06_create_existing_id_without_receipt_conflicts_even_when_input_matches() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    let (_, base) = f.seed(&[trial(1)]);
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(base);
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    assert_eq!(
        candidate(&w.recover(None), &op(1)).state,
        RecoveryState::Conflict
    );
    let req = FrozenHistoryRequest::new(create(&p.trial), vec![]).unwrap();
    kind(w.resume(&op(1), &req), HistoryErrorKind::Conflict);
    assert_eq!(tree(&history), before);
}

#[test]
fn w06_update_rebases_only_when_expected_target_is_unchanged() {
    for change_target in [false, true] {
        let mut f = Fixture::new();
        let t = trial(1);
        let other = trial(2);
        let (base_root, base) = f.seed(&[t.clone(), other.clone()]);
        let p = Plan::reflection(&t, &base_root, base.clone(), op(1));
        f.install(&p, 0);
        let concurrent = Plan::reflection(
            if change_target { &t } else { &other },
            &base_root,
            base,
            op(2),
        );
        f.install(&concurrent, concurrent.objects.len());
        let history = f.history();
        let before = tree(&history);
        let head = Head::at(concurrent.root_ref());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let req =
            FrozenHistoryRequest::new(serde_json::from_str(&p.mutation_json).unwrap(), vec![])
                .unwrap();
        if change_target {
            assert_eq!(
                candidate(&w.recover(None), &op(1)).state,
                RecoveryState::Conflict
            );
            kind(w.resume(&op(1), &req), HistoryErrorKind::Conflict);
            assert_eq!(tree(&history), before);
        } else {
            let expected = Plan::reflection(&t, &concurrent.root, concurrent.root_ref(), op(1));
            let prepared = ready(w.resume(&op(1), &req));
            assert_plan(&history, &published(w.publish(&prepared)), &expected);
            assert_preserved(
                &history,
                &before,
                Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
            );
        }
    }
}

#[test]
fn w05_rebase_quota_counts_only_new_objects_and_exact_intent_replacement_delta() {
    for below in [false, true] {
        let mut f = Fixture::new();
        let old = Plan::create(trial(1), op(1));
        f.install(&old, 3);
        let (root, base) = f.seed(&[trial(2)]);
        let expected = Plan::create_on(trial(1), op(1), &root, Some(base.clone()), None);
        let history = f.history();
        let before = tree(&history);
        let retained: u64 = before.values().map(|b| b.len() as u64).sum();
        let added: u64 = expected
            .refs()
            .iter()
            .filter(|r| !object_path(&history, r).exists())
            .map(|r| r.bytes)
            .sum();
        let delta = serde_json::to_vec(&expected.intent()).unwrap().len() as i64
            - fs::metadata(intent_path(&history, &op(1))).unwrap().len() as i64;
        let projected = (retained as i64 + added as i64 + delta) as u64;
        assert!(delta > 0, "new base reference makes replacement longer");
        let head = Head::at(base);
        let mut w = HistoryWriter::new(
            &mut f.storage,
            &head,
            HistoryQuota {
                limit_bytes: projected - u64::from(below),
            },
        )
        .unwrap();
        let req = w.read_recovery_request(&op(1), vec![]).unwrap();
        if below {
            kind(w.resume(&op(1), &req), HistoryErrorKind::QuotaExceeded);
            assert_eq!(tree(&history), before);
        } else {
            let prepared = ready(w.resume(&op(1), &req));
            let q = prepared.quota_plan();
            assert_eq!(q.before.retained_bytes, retained);
            assert_eq!(q.added_object_bytes, added);
            assert_eq!(q.intent_delta_bytes, delta);
            assert_eq!(q.projected_bytes, projected);
            assert_plan(&history, &published(w.publish(&prepared)), &expected);
            assert_eq!(
                tree(&history).values().map(|b| b.len() as u64).sum::<u64>(),
                projected
            );
        }
    }
}

#[test]
fn w08_structurally_valid_but_forged_old_plan_is_recomputed_before_resume() {
    let mut f = Fixture::new();
    let mut p = Plan::create(trial(1), op(1));
    p.root.revision = 2;
    p.root.receipts.get_mut(&op(1)).unwrap().committed_revision = 2;
    *p.objects.last_mut().unwrap() = serde_json::to_vec(&p.root).unwrap();
    f.install(&p, 3);
    HistoryReader::new(&f.storage)
        .verify_closure(&p.root_ref())
        .unwrap();
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(candidate(&view, &op(1)).state, RecoveryState::Damaged);
    assert!(!view.diagnostics.is_empty());
    assert!(w.read_recovery_request(&op(1), vec![]).is_err());
    let req = FrozenHistoryRequest::new(create(&p.trial), vec![]).unwrap();
    assert!(w.resume(&op(1), &req).is_err());
    assert_eq!(tree(&history), before);
}

#[test]
fn w06_rebase_validates_old_base_closure_before_replacing_its_intent() {
    let mut f = Fixture::new();
    let (old_root, old_ref) = f.seed(&[trial(2)]);
    let p = Plan::create_on(trial(1), op(1), &old_root, Some(old_ref.clone()), None);
    f.install(&p, 1);
    let (_, current) = f.seed(&[trial(3)]);
    let history = f.history();
    fs::remove_file(object_path(&history, old_ref.as_blob())).unwrap();
    let before = tree(&history);
    let head = Head::at(current);
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let req = FrozenHistoryRequest::new(create(&p.trial), vec![]).unwrap();
    assert!(w.resume(&op(1), &req).is_err());
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_invalid_intents_are_diagnosed_without_hiding_healthy_candidate_or_writing() {
    for mode in 0..10 {
        let mut f = Fixture::new();
        let bad = Plan::create(trial(1), op(1));
        let good = Plan::create(trial(2), op(2));
        f.install(&good, 1);
        let mut value = bad.intent();
        match mode {
            0 => value["schema_version"] = json!(2),
            1 => value["unknown"] = json!("must reject"),
            2 => value["operation"] = json!(op(99)),
            3 => {
                value.as_object_mut().unwrap().remove("base");
            }
            4 => value["request"] = json!(digest(b"different exact request")),
            5 => {
                let a = value["planned_objects"].as_array_mut().unwrap();
                a.swap(0, 1);
            }
            6 => {
                let a = value["planned_objects"].as_array_mut().unwrap();
                a.insert(1, json!(blob(b"extra")));
            }
            7 => value["planned_objects"][0]["bytes"] = json!(999),
            8 => {
                let mut mutation: Value = serde_json::from_str(&bad.mutation_json).unwrap();
                mutation["create_trial"]["unknown"] = json!(true);
                let exact = serde_json::to_string(&mutation).unwrap();
                value["request"] = json!(digest(exact.as_bytes()));
                value["mutation_json"] = json!(exact);
            }
            _ => {
                value["mutation_json"] = json!("x".repeat(524_289));
            }
        }
        f.intent(&op(1), &value);
        let history = f.history();
        let before = tree(&history);
        let head = Head::default();
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let view = w.recover(None);
        assert!(!view.diagnostics.is_empty(), "mode {mode}");
        assert_eq!(
            candidate(&view, &op(2)).state,
            RecoveryState::AssociationReady,
            "mode {mode}"
        );
        assert!(
            w.read_recovery_request(&op(1), vec![bad.objects[0].clone().into()])
                .is_err(),
            "mode {mode}"
        );
        let req =
            FrozenHistoryRequest::new(create(&bad.trial), vec![bad.objects[0].clone().into()])
                .unwrap();
        assert!(w.resume(&op(1), &req).is_err(), "mode {mode}");
        assert_eq!(tree(&history), before);
    }
}

#[test]
fn w08_invalid_filename_and_truncated_json_do_not_block_valid_candidates() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    let history = f.history();
    fs::write(history.join("recovery/not-an-operation.json"), b"{}").unwrap();
    fs::write(intent_path(&history, &op(2)), b"{broken").unwrap();
    let before = tree(&history);
    let head = Head::default();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(
        candidate(&view, &op(1)).state,
        RecoveryState::AssociationReady
    );
    assert!(view.diagnostics.len() >= 2);
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_corrupt_published_object_is_damaged_and_never_overwritten_by_caller_bytes() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    let history = f.history();
    fs::write(object_path(&history, &p.refs()[0]), b"broken").unwrap();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(candidate(&view, &op(1)).state, RecoveryState::Damaged);
    assert!(!view.diagnostics.is_empty());
    let req =
        FrozenHistoryRequest::new(create(&p.trial), vec![p.objects[0].clone().into()]).unwrap();
    assert!(w.resume(&op(1), &req).is_err());
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_pagination_is_operation_ordered_bounded_and_advances_past_damaged_intent() {
    let mut f = Fixture::new();
    for n in (1..=34).rev() {
        f.install(&Plan::create(trial(n), op(n)), 0);
    }
    let history = f.history();
    fs::write(intent_path(&history, &op(17)), b"{").unwrap();
    let before = tree(&history);
    let head = Head::default();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let first = w.recover(None);
    assert!(first.candidates.len() <= 32);
    assert_eq!(first.next_after, Some(op(32)));
    assert!(!first.diagnostics.is_empty());
    let ids: Vec<_> = first
        .candidates
        .iter()
        .map(|c| c.operation.clone())
        .collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]));
    for n in (1..=32).filter(|n| *n != 17) {
        assert!(ids.contains(&op(n)));
    }
    let last = w.recover(first.next_after.as_ref());
    assert!(last.next_after.is_none());
    assert!(last.diagnostics.is_empty());
    assert_eq!(
        last.candidates
            .iter()
            .map(|c| c.operation.clone())
            .collect::<Vec<_>>(),
        vec![op(33), op(34)]
    );
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_exactly_32_candidates_have_no_spurious_next_page() {
    let mut f = Fixture::new();
    for n in 1..=32 {
        f.install(&Plan::create(trial(n), op(n)), 0);
    }
    let history = f.history();
    let before = tree(&history);
    let head = Head::default();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert_eq!(view.candidates.len(), 32);
    assert!(view.next_after.is_none());
    assert!(view.diagnostics.is_empty());
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_intent_path_directory_is_unsafe_and_other_candidates_remain_visible() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    let history = f.history();
    let unsafe_path = intent_path(&history, &op(2));
    fs::create_dir(&unsafe_path).unwrap();
    fs::write(unsafe_path.join("preserve.bin"), b"independent sentinel").unwrap();
    let before = tree(&history);
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert!(view
        .diagnostics
        .iter()
        .any(|e| e.kind == HistoryErrorKind::UnsafePath));
    assert_eq!(
        candidate(&view, &op(1)).state,
        RecoveryState::AssociationReady
    );
    kind(
        w.read_recovery_request(&op(2), vec![]),
        HistoryErrorKind::UnsafePath,
    );
    let req = FrozenHistoryRequest::new(create(&trial(2)), vec![]).unwrap();
    kind(w.resume(&op(2), &req), HistoryErrorKind::UnsafePath);
    assert_eq!(tree(&history), before);
}

#[test]
fn w08_unreadable_or_damaged_current_head_never_offers_resume_as_ready() {
    for unreadable in [false, true] {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        f.install(&p, 1);
        let head = Head::at(RootRef::new(blob(b"unavailable root")));
        head.fail.set(unreadable);
        let history = f.history();
        let before = tree(&history);
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let view = w.recover(None);
        assert!(!view.diagnostics.is_empty());
        assert!(view
            .candidates
            .iter()
            .all(|c| matches!(c.state, RecoveryState::Damaged | RecoveryState::Conflict)));
        let req = FrozenHistoryRequest::new(create(&p.trial), vec![]).unwrap();
        assert!(w.resume(&op(1), &req).is_err());
        assert_eq!(tree(&history), before);
    }
}

#[test]
fn w08_head_change_during_recovery_requires_new_snapshot() {
    struct MovingHead {
        calls: Cell<usize>,
        next: RootRef,
    }
    impl CommittedHistoryHead for MovingHead {
        fn read_current(&self, _: &Storage) -> Result<Option<RootRef>, HistoryError> {
            let n = self.calls.get();
            self.calls.set(n + 1);
            Ok((n > 0).then(|| self.next.clone()))
        }
    }
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    let (_, root) = f.seed(&[trial(2)]);
    let head = MovingHead {
        calls: Cell::new(0),
        next: root,
    };
    let history = f.history();
    let before = tree(&history);
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let view = w.recover(None);
    assert!(view
        .diagnostics
        .iter()
        .any(|e| e.kind == HistoryErrorKind::Conflict));
    assert!(view
        .candidates
        .iter()
        .all(|c| matches!(c.state, RecoveryState::Conflict | RecoveryState::Damaged)));
    assert_eq!(tree(&history), before);
}

#[test]
fn w06_resume_obeys_fresh_head_and_pending_material_guard_without_writes() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    let head = Head::default();
    let history = f.history();
    let pending = f.storage.dir.join("pending-material-commit.json");
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let req = w.read_recovery_request(&op(1), vec![]).unwrap();
    fs::write(&pending, b"synthetic pending").unwrap();
    let before = tree(&history);
    kind(w.resume(&op(1), &req), HistoryErrorKind::PendingMaterial);
    assert_eq!(tree(&history), before);
    fs::remove_file(pending).unwrap();
    head.fail.set(true);
    kind(w.resume(&op(1), &req), HistoryErrorKind::Io);
    assert_eq!(tree(&history), before);
}

struct FailAt(HistoryCheckpoint);
impl HistoryFaults for FailAt {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
        if point == self.0 {
            Err(HistoryError {
                kind: HistoryErrorKind::Io,
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn w03_finish_faults_preserve_saved_receipt_and_retry_after_restart() {
    for point in [
        HistoryCheckpoint::BeforeFinish,
        HistoryCheckpoint::AfterFinish,
    ] {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        f.install(&p, 3);
        let head = Head::at(p.root_ref());
        let history = f.history();
        let before = tree(&history);
        let faults = FailAt(point);
        {
            let mut w =
                HistoryWriter::with_faults(&mut f.storage, &head, HistoryQuota::v1(), &faults)
                    .unwrap();
            let c = w.confirm_committed(&p.publication()).unwrap();
            kind(w.finish(&c), HistoryErrorKind::Io);
            assert_eq!(
                w.confirm_committed(&p.publication()).unwrap().receipt,
                c.receipt
            );
        }
        assert_eq!(
            intent_path(&history, &op(1)).exists(),
            point == HistoryCheckpoint::BeforeFinish
        );
        assert_preserved(
            &history,
            &before,
            Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
        );
        f.reopen();
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let c = w.confirm_committed(&p.publication()).unwrap();
        w.finish(&c).unwrap();
        assert!(!intent_path(&history, &op(1)).exists());
        assert_eq!(*head.current.borrow(), Some(p.root_ref()));
    }
}

#[test]
fn w03_finish_rechecks_head_and_never_deletes_replaced_request_intent() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    let other = Plan::create(trial(2), op(1));
    let history = f.history();
    let head = Head::at(p.root_ref());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let c = w.confirm_committed(&p.publication()).unwrap();
    *head.current.borrow_mut() = None;
    let before = tree(&history);
    kind(w.finish(&c), HistoryErrorKind::Busy);
    assert_eq!(tree(&history), before);
    *head.current.borrow_mut() = Some(p.root_ref());
    fs::write(
        intent_path(&history, &op(1)),
        serde_json::to_vec(&other.intent()).unwrap(),
    )
    .unwrap();
    let before = tree(&history);
    kind(w.finish(&c), HistoryErrorKind::Conflict);
    assert_eq!(tree(&history), before);
}

#[test]
fn w03_confirm_rejects_wrong_receipt_request_affected_and_unrelated_head() {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        f.install(&p, 3);
        let (_, other_ref) = f.seed(&[trial(2)]);
        let mut root = p.root.clone();
        if mode == 0 {
            root.receipts.get_mut(&op(1)).unwrap().request = digest(b"wrong");
        }
        if mode == 1 {
            let t = trial(2);
            root.trials.insert(
                t.id.clone(),
                TrialHead {
                    document: f.put(&t),
                    admitted_sequence: 2,
                },
            );
            root.next_save_sequence = 3;
            root.receipts.get_mut(&op(1)).unwrap().affected = vec![t.id];
        }
        let head = Head::at(if mode == 2 {
            other_ref
        } else {
            RootRef::new(f.put(&root))
        });
        let history = f.history();
        let before = tree(&history);
        let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        kind(
            w.confirm_committed(&p.publication()),
            HistoryErrorKind::Conflict,
        );
        assert_eq!(tree(&history), before);
    }
}

#[test]
fn w03_public_tokens_cannot_forge_request_receipt_affected_or_current_for_cleanup() {
    for mode in 0..7 {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        f.install(&p, 3);
        let unrelated = Plan::create(trial(2), op(2));
        f.install(&unrelated, 3);
        let head = Head::at(p.root_ref());
        let history = f.history();
        let before = tree(&history);
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        if mode == 0 || mode == 6 {
            let mut forged = p.publication();
            if mode == 0 {
                forged.request = digest(b"forged request");
            } else {
                forged.next = unrelated.root_ref();
            }
            assert!(w.confirm_committed(&forged).is_err());
        } else {
            let mut forged = w.confirm_committed(&p.publication()).unwrap();
            match mode {
                1 => forged.receipt.request = digest(b"forged request"),
                2 => forged.receipt.affected = vec![tid(2)],
                3 => forged.current = unrelated.root_ref(),
                4 => forged.operation = op(2),
                _ => forged.receipt.committed_revision += 1,
            }
            assert!(w.finish(&forged).is_err(), "mode {mode}");
        }
        assert_eq!(tree(&history), before, "mode {mode}");
    }
}

#[test]
fn w02_real_prepare_publish_faults_restart_from_only_the_bytes_that_reached_disk() {
    let points = [
        (HistoryCheckpoint::BeforeIntent, None),
        (
            HistoryCheckpoint::AfterIntent,
            Some(RecoveryState::OriginalMissing),
        ),
        (
            HistoryCheckpoint::BeforeObject { index: 0 },
            Some(RecoveryState::OriginalMissing),
        ),
        (
            HistoryCheckpoint::AfterObject { index: 0 },
            Some(RecoveryState::AssociationReady),
        ),
        (
            HistoryCheckpoint::BeforeObject { index: 1 },
            Some(RecoveryState::AssociationReady),
        ),
        (
            HistoryCheckpoint::AfterObject { index: 1 },
            Some(RecoveryState::AssociationReady),
        ),
        (
            HistoryCheckpoint::BeforeObject { index: 2 },
            Some(RecoveryState::AssociationReady),
        ),
        (
            HistoryCheckpoint::AfterObject { index: 2 },
            Some(RecoveryState::PublishedUncommitted),
        ),
        (
            HistoryCheckpoint::BeforeClosure,
            Some(RecoveryState::PublishedUncommitted),
        ),
        (
            HistoryCheckpoint::AfterClosure,
            Some(RecoveryState::PublishedUncommitted),
        ),
    ];
    for (point, expected_state) in points {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        let head = Head::default();
        let req =
            FrozenHistoryRequest::new(create(&p.trial), vec![p.objects[0].clone().into()]).unwrap();
        let faults = FailAt(point);
        {
            let mut w =
                HistoryWriter::with_faults(&mut f.storage, &head, HistoryQuota::v1(), &faults)
                    .unwrap();
            if matches!(
                point,
                HistoryCheckpoint::BeforeIntent | HistoryCheckpoint::AfterIntent
            ) {
                kind(w.prepare(op(1), None, &req), HistoryErrorKind::Io);
            } else {
                let prepared = ready(w.prepare(op(1), None, &req));
                kind(w.publish(&prepared), HistoryErrorKind::Io);
            }
        }
        f.reopen();
        let history = f.history();
        let before = tree(&history);
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let view = w.recover(None);
        if let Some(state) = expected_state {
            assert_eq!(
                candidate(&view, &op(1)).state,
                state,
                "checkpoint {point:?}"
            );
            assert_eq!(tree(&history), before);
            let restored = w
                .read_recovery_request(&op(1), vec![p.objects[0].clone().into()])
                .unwrap();
            let prepared = ready(w.resume(&op(1), &restored));
            assert_plan(&history, &published(w.publish(&prepared)), &p);
            assert_preserved(&history, &before, None);
        } else {
            assert!(view.candidates.is_empty());
            assert!(view.diagnostics.is_empty());
            assert!(!history.exists());
        }
        assert!(head.current.borrow().is_none());
    }
}

#[test]
fn w03_confirm_and_finish_require_readable_complete_latest_closure() {
    for io_failure in [false, true] {
        let mut f = Fixture::new();
        let p = Plan::create(trial(1), op(1));
        f.install(&p, 3);
        let head = Head::at(p.root_ref());
        let history = f.history();
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let committed = w.confirm_committed(&p.publication()).unwrap();
        if io_failure {
            head.fail.set(true);
        } else {
            fs::remove_file(object_path(&history, &p.refs()[0])).unwrap();
        }
        let before = tree(&history);
        assert!(w.confirm_committed(&p.publication()).is_err());
        assert!(w.finish(&committed).is_err());
        assert_eq!(tree(&history), before);
        assert!(intent_path(&history, &op(1)).exists());
    }
}

#[test]
fn w03_finish_accepts_later_receipt_preserving_root_after_confirmation() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 3);
    let later = Plan::create_on(trial(2), op(2), &p.root, Some(p.root_ref()), None);
    f.install(&later, 3);
    let history = f.history();
    let before = tree(&history);
    let head = Head::at(p.root_ref());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let committed = w.confirm_committed(&p.publication()).unwrap();
    *head.current.borrow_mut() = Some(later.root_ref());
    w.finish(&committed).unwrap();
    assert!(!intent_path(&history, &op(1)).exists());
    assert!(intent_path(&history, &op(2)).exists());
    assert_preserved(
        &history,
        &before,
        Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
    );
}

#[test]
fn w02_drop_after_resume_keeps_intent_audio_and_host_head_uncommitted() {
    let mut f = Fixture::new();
    let p = Plan::create(trial(1), op(1));
    f.install(&p, 1);
    let head = Head::default();
    let history = f.history();
    let before = tree(&history);
    {
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let req = w.read_recovery_request(&op(1), vec![]).unwrap();
        drop(ready(w.resume(&op(1), &req)));
    }
    assert_eq!(tree(&history), before);
    assert!(head.current.borrow().is_none());
    f.reopen();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    assert_eq!(
        candidate(&w.recover(None), &op(1)).state,
        RecoveryState::AssociationReady
    );
}

#[test]
fn w05_resume_rebase_serialized_intent_exact_cap_includes_escapes_and_preserves_on_overflow() {
    // Independent wire oracle: explicitly encode the schema fields, never writer output.
    fn intent_bytes(p: &Plan) -> Vec<u8> {
        format!(
            "{{\"schema_version\":1,\"operation\":{},\"request\":{},\"mutation_json\":{},\"base\":{},\"planned_root\":{},\"planned_objects\":{}}}",
            serde_json::to_string(&p.operation).unwrap(),
            serde_json::to_string(&digest(p.mutation_json.as_bytes())).unwrap(),
            serde_json::to_string(&p.mutation_json).unwrap(),
            serde_json::to_string(&p.base).unwrap(),
            serde_json::to_string(&p.root_ref()).unwrap(),
            serde_json::to_string(&p.refs()).unwrap(),
        )
        .into_bytes()
    }

    for wanted in [524_287usize, 524_288, 524_289] {
        let mut f = Fixture::new();
        let temp = f._temp.0.clone();
        eprintln!("R1 target={wanted} temp={}", temp.display());
        let mut t = trial(1);
        t.input.reference.text = "Quote: \"hello\"; path: C:\\synthetic\n復元".into();
        t.input.reference.sha256 = digest(t.input.reference.text.as_bytes());
        let (current, current_ref) = f.seed(&[trial(2)]);
        let short = Plan::create_on(t.clone(), op(1), &current, Some(current_ref.clone()), None);
        let short_bytes = intent_bytes(&short);
        // Legal trailing whitespace changes the exact request hash, not the mutation.
        let exact = format!(
            "{}{}",
            short.mutation_json,
            " ".repeat(wanted.checked_sub(short_bytes.len()).unwrap())
        );
        assert_eq!(
            serde_json::from_str::<HistoryMutation>(&exact).unwrap(),
            create(&t)
        );
        let expected = Plan::create_on(
            t.clone(),
            op(1),
            &current,
            Some(current_ref.clone()),
            Some(exact.clone()),
        );
        let expected_bytes = intent_bytes(&expected);
        assert_eq!(expected_bytes.len(), wanted, "independent encoded oracle");
        assert!(expected.mutation_json.len() < wanted);
        assert!(exact.contains("\\\"hello\\\""));
        assert!(exact.contains("C:\\\\synthetic\\n"));
        let old = Plan::create_on(t, op(1), &empty_root(), None, Some(exact));
        let old_bytes = serde_json::to_vec(&old.intent()).unwrap();
        assert!(
            old_bytes.len() < 524_288,
            "old intent is readable for every boundary"
        );
        assert!(old_bytes.len() < expected_bytes.len());
        f.install(&old, old.objects.len());
        let history = f.history();
        let before = tree(&history);
        let head = Head::at(current_ref.clone());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let request = w.read_recovery_request(&op(1), vec![]).unwrap();
        assert_eq!(request.mutation_json(), old.mutation_json);
        assert_eq!(
            *request.request_hash(),
            digest(old.mutation_json.as_bytes())
        );
        if wanted > 524_288 {
            for _ in 0..2 {
                kind(w.resume(&op(1), &request), HistoryErrorKind::LimitExceeded);
                assert_eq!(
                    tree(&history),
                    before,
                    "rejected resume/retry must not write"
                );
                assert_eq!(fs::read(intent_path(&history, &op(1))).unwrap(), old_bytes);
            }
        } else {
            let prepared = ready(w.resume(&op(1), &request));
            let replacement = fs::read(intent_path(&history, &op(1))).unwrap();
            assert_eq!(replacement.len(), wanted);
            assert_eq!(replacement, expected_bytes, "exact independent wire oracle");
            assert_eq!(
                serde_json::from_slice::<Value>(&replacement).unwrap(),
                expected.intent()
            );
            assert_plan(&history, &published(w.publish(&prepared)), &expected);
            assert_preserved(
                &history,
                &before,
                Some(&PathBuf::from(format!("recovery/{}.json", op(1).as_str()))),
            );
        }
        assert_eq!(*head.current.borrow(), Some(current_ref));
        drop(w);
        drop(f);
        assert!(
            !temp.exists(),
            "isolated R1 fixture cleanup failed: {}",
            temp.display()
        );
    }
}

#[test]
fn w08_recover_scans_100000_entries_but_diagnoses_100001_without_writing() {
    let started = std::time::Instant::now();
    let mut f = Fixture::new();
    let temp = f._temp.0.clone();
    let history = f.history();
    let recovery = history.join("recovery");
    fs::create_dir_all(&recovery).unwrap();
    f.put_bytes(b"Synthetic orphan sentinel: recover must preserve every asset.");
    eprintln!("R2 temp={} creating 100000 real entries", temp.display());
    for n in 1..=100_000 {
        if n <= 32 || n >= 99_969 {
            f.install(&Plan::create(trial(n), op(n)), 0);
        } else {
            // Valid names exercise the actual full scan. Unselected bodies deliberately
            // are not valid intents: this test does not claim to validate their contents.
            fs::write(intent_path(&history, &op(n)), b"{}").unwrap();
        }
        if n % 25_000 == 0 {
            eprintln!("R2 created {n} entries after {:?}", started.elapsed());
        }
    }
    assert_eq!(fs::read_dir(&recovery).unwrap().count(), 100_000);
    let before = tree(&history);
    let head = Head::default();
    let w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    for (after, first, next) in [(None, 1, Some(op(32))), (Some(op(99_968)), 99_969, None)] {
        let view = w.recover(after.as_ref());
        assert!(
            view.diagnostics.is_empty(),
            "100000-entry scan must succeed: {view:?}"
        );
        assert_eq!(view.next_after, next);
        assert_eq!(
            view.candidates
                .iter()
                .map(|c| c.operation.clone())
                .collect::<Vec<_>>(),
            (first..first + 32).map(op).collect::<Vec<_>>()
        );
        for (c, n) in view.candidates.iter().zip(first..first + 32) {
            assert_eq!(c.state, RecoveryState::OriginalMissing);
            let p = Plan::create(trial(n), op(n));
            assert_eq!(c.request, digest(p.mutation_json.as_bytes()));
            assert!(c.missing.contains(&p.refs()[0]));
        }
        assert_eq!(tree(&history), before, "successful recover must not write");
        eprintln!(
            "R2 accepted page starting {first} after {:?}",
            started.elapsed()
        );
    }
    let extra = intent_path(&history, &op(100_001));
    fs::write(&extra, b"{}").unwrap();
    assert_eq!(fs::read_dir(&recovery).unwrap().count(), 100_001);
    let before_overflow = tree(&history);
    let view = w.recover(None);
    assert!(
        view.diagnostics
            .iter()
            .any(|e| e.kind == HistoryErrorKind::LimitExceeded),
        "100001 entries must produce LimitExceeded, not an empty success: {view:?}"
    );
    assert_eq!(
        tree(&history),
        before_overflow,
        "overflow recover must not write"
    );
    assert!(head.current.borrow().is_none());
    eprintln!(
        "R2 overflow diagnosed after {:?}; cleaning fixture",
        started.elapsed()
    );
    drop(w);
    drop(f);
    assert!(
        !temp.exists(),
        "isolated R2 fixture cleanup failed: {}",
        temp.display()
    );
    eprintln!("R2 cleanup complete after {:?}", started.elapsed());
}
