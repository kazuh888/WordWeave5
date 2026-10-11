//! Independent I1b-2a oracles: hand-built immutable documents and synthetic PCM.
//! Never uses the writer to construct the committed starting state.
#![cfg(windows)]

use acoustic_analysis::AnalysisProfile;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
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
        let path = std::env::temp_dir().join(format!(
            "ww5-writer-tests-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self {
            storage: Storage::at(path.join("host")).unwrap(),
            _temp: Temp(path),
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
    fn put(&self, value: &impl Serialize) -> BlobRef {
        self.put_bytes(&serde_json::to_vec(value).unwrap())
    }
    fn seed(&self, trials: &[TrialRevision]) -> (HistoryRoot, RootRef) {
        self.put_bytes(&wav());
        let root = HistoryRoot {
            schema_version: 1,
            revision: 1,
            next_save_sequence: trials.len() as u64 + 1,
            trials: trials
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    (
                        t.id.clone(),
                        TrialHead {
                            document: self.put(t),
                            admitted_sequence: i as u64 + 1,
                        },
                    )
                })
                .collect(),
            receipts: BTreeMap::new(),
        };
        let reference = RootRef::new(self.put(&root));
        HistoryReader::new(&self.storage)
            .verify_closure(&reference)
            .expect("handwritten fixture closure must be valid");
        (root, reference)
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
fn tid(n: u64) -> TrialId {
    TrialId::parse(&format!("00000000-0000-4000-8000-{n:012x}")).unwrap()
}
fn rid(n: u64) -> RunId {
    RunId::parse(&format!("11111111-1111-4111-8111-{n:012x}")).unwrap()
}
fn op(n: u64) -> OperationId {
    OperationId::parse(&format!("22222222-2222-4222-8222-{n:012x}")).unwrap()
}
fn at() -> RecordedAt {
    RecordedAt {
        utc_millis: 1_700_000_000_000,
        offset_minutes: 540,
    }
}
fn wav() -> Vec<u8> {
    // One signed PCM16 frame at 8kHz, mono; exact original is never decoded here.
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
    b.extend(1234i16.to_le_bytes());
    b
}
fn trial(n: u64) -> TrialRevision {
    TrialRevision {
        schema_version: 1,
        id: tid(n),
        revision: 1,
        previous: None,
        created_at: at(),
        input: FrozenInput {
            audio: AudioSnapshot {
                original: blob(&wav()),
                format: AudioFormat::Wav,
                rate: 8000,
                channels: 1,
                frames: 1,
                origin: Origin::Imported,
                warnings: vec![CaptureWarning::PartialRecording],
            },
            reference: ReferenceSnapshot {
                text: "Keep this exact sentence.\n日本語".into(),
                sha256: digest("Keep this exact sentence.\n日本語".as_bytes()),
                source: None,
            },
            goal: Some(GoalSnapshot {
                text: "Slowly".into(),
                version: digest(b"Slowly"),
            }),
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
fn practice(duplicate_confirmed: bool) -> PracticeClaim {
    PracticeClaim::Practice {
        performed_at: at(),
        self_speaker_confirmed_at: at(),
        time_confirmed_at: at(),
        duplicate_confirmed,
    }
}
fn reflection(t: &TrialRevision) -> Reflection {
    Reflection {
        schema_version: 1,
        trial: t.id.clone(),
        at: at(),
        text: "Keep earlier work.".into(),
    }
}
fn analysis(t: &TrialRevision, n: u64) -> AnalysisRun {
    AnalysisRun {
        schema_version: 1,
        id: rid(n),
        trial: t.id.clone(),
        audio: t.input.audio.original.sha256.clone(),
        pcm: digest(b"PCM"),
        decoder: DecoderIdentity {
            name: "synthetic".into(),
            version: "1".into(),
            binary_sha256: None,
        },
        profile: AnalysisProfile::v1(),
        state: AnalysisRunState::Running,
        started_at: at(),
        ended_at: None,
        result: None,
        failure: None,
    }
}
fn advice(t: &TrialRevision, n: u64) -> AdviceRun {
    AdviceRun {
        schema_version: 1,
        id: rid(n),
        trial: t.id.clone(),
        audio: t.input.audio.original.sha256.clone(),
        reference: t.input.reference.sha256.clone(),
        confirmed_at: at(),
        confirmation: digest(b"confirmation"),
        destination: "https://fixture.invalid".into(),
        versions: AdviceVersions {
            provider: "fixture".into(),
            prompt: "1".into(),
            parser: "1".into(),
        },
        state: AdviceRunState::Sending,
        send: SendCertainty::NotSent,
        started_at: at(),
        ended_at: None,
        result: None,
        failure: None,
    }
}
fn feedback() -> AdviceResult {
    AdviceResult {
        schema_version: 1,
        assessment: AdviceAssessment::Assessed,
        heard_text: Some("Keep this exact sentence.".into()),
        summary: "Synthetic result".into(),
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
fn object_path(history: &Path, reference: &BlobRef) -> PathBuf {
    history
        .join("assets")
        .join(format!("{}.bin", reference.sha256.as_str()))
}
fn read<T: DeserializeOwned>(history: &Path, reference: &BlobRef) -> T {
    serde_json::from_slice(&fs::read(object_path(history, reference)).unwrap()).unwrap()
}
fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !path.exists() {
            return;
        }
        for e in fs::read_dir(path).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.insert(p.strip_prefix(base).unwrap().to_owned(), vec![]);
                visit(base, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(base).unwrap().to_owned(),
                    fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(path, path, &mut result);
    result
}
fn preserved(history: &Path, old: &BTreeMap<PathBuf, Vec<u8>>) {
    let now = tree(history);
    for (path, bytes) in old {
        assert_eq!(now.get(path), Some(bytes), "changed old object {path:?}");
    }
}
fn ready(value: Result<PrepareOutcome, HistoryError>) -> PreparedHistory {
    match value.unwrap() {
        PrepareOutcome::Ready(p) => p,
        other => panic!("expected Ready, got {other:?}"),
    }
}
fn published(value: Result<PublishOutcome, HistoryError>) -> PublishedHistory {
    match value.unwrap() {
        PublishOutcome::Published(p) => p,
        other => panic!("expected Published, got {other:?}"),
    }
}
fn kind<T: std::fmt::Debug>(value: Result<T, HistoryError>, expected: HistoryErrorKind) {
    assert_eq!(value.unwrap_err().kind, expected);
}
fn request(m: HistoryMutation) -> FrozenHistoryRequest {
    FrozenHistoryRequest::new(m, vec![]).unwrap()
}
fn publish_mutation(
    f: &mut Fixture,
    base: &RootRef,
    m: HistoryMutation,
    payload: Vec<Arc<[u8]>>,
) -> PublishedHistory {
    let req = FrozenHistoryRequest::new(m, payload).unwrap();
    let head = Head::at(base.clone());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let prepared = ready(w.prepare(op(10), Some(base), &req));
    published(w.publish(&prepared))
}
fn assert_update(
    f: &Fixture,
    old_root: &HistoryRoot,
    old_trial: &TrialRevision,
    publication: &PublishedHistory,
    mut expected: TrialRevision,
) {
    expected.revision = old_trial.revision + 1;
    expected.previous = Some(target(old_trial).expected);
    let root: HistoryRoot = read(&f.history(), publication.next.as_blob());
    assert_eq!(root.revision, old_root.revision + 1);
    assert_eq!(root.next_save_sequence, old_root.next_save_sequence);
    assert_eq!(root.trials.len(), old_root.trials.len());
    assert_eq!(
        root.trials[&expected.id].admitted_sequence,
        old_root.trials[&expected.id].admitted_sequence
    );
    let actual: TrialRevision = read(&f.history(), &root.trials[&expected.id].document);
    assert_eq!(actual, expected);
    let mut expected_root = old_root.clone();
    expected_root.revision += 1;
    expected_root.trials.get_mut(&expected.id).unwrap().document =
        blob(&serde_json::to_vec(&expected).unwrap());
    expected_root.receipts.insert(
        op(10),
        OperationReceipt {
            request: publication.request.clone(),
            committed_revision: expected_root.revision,
            affected: vec![expected.id.clone()],
        },
    );
    assert_eq!(root, expected_root);
    HistoryReader::new(&f.storage)
        .verify_closure(&publication.next)
        .unwrap();
}

#[test]
fn handwritten_fixture_is_valid_without_writer() {
    let f = Fixture::new();
    let t = trial(1);
    t.validate().unwrap();
    analysis(&t, 1).validate().unwrap();
    advice(&t, 2).validate().unwrap();
    feedback().validate().unwrap();
    f.seed(&[t]);
    assert_eq!(wav().len(), 46);
}

#[test]
fn w01_frozen_request_hashes_exact_json_and_keeps_caller_arc() {
    let t = trial(1);
    let m = create(&t);
    let exact = serde_json::to_string(&m).unwrap();
    let bytes: Arc<[u8]> = wav().into();
    let req = FrozenHistoryRequest::new(m.clone(), vec![bytes.clone(), bytes.clone()]).unwrap();
    assert_eq!(req.mutation_json(), exact);
    assert_eq!(req.request_hash(), &digest(exact.as_bytes()));
    assert_eq!(req.mutation(), &m);
    assert_eq!(&*bytes, wav());
    drop(req);
    assert_eq!(&*bytes, wav());
}

#[test]
fn w01_unreferenced_and_mismatched_payloads_are_rejected() {
    for bytes in [b"unreferenced".to_vec(), {
        let mut x = wav();
        x[44] ^= 1;
        x
    }] {
        assert!(FrozenHistoryRequest::new(create(&trial(1)), vec![bytes.into()]).is_err());
    }
}

#[test]
fn w01_create_publishes_exact_frozen_input_but_does_not_commit_head() {
    let mut f = Fixture::new();
    let history = f.history();
    let t = trial(1);
    let bytes: Arc<[u8]> = wav().into();
    let source = f._temp.0.join("selected.wav");
    fs::write(&source, &*bytes).unwrap();
    let req = FrozenHistoryRequest::new(create(&t), vec![bytes.clone()]).unwrap();
    fs::write(&source, b"changed selection source").unwrap();
    let head = Head::default();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    assert!(!history.exists(), "constructor must not create directories");
    let prepared = ready(w.prepare(op(1), None, &req));
    assert!(!object_path(&history, &t.input.audio.original).exists());
    let p = published(w.publish(&prepared));
    assert_eq!(*head.current.borrow(), None);
    assert_eq!(p.base, None);
    assert_eq!(p.operation, op(1));
    assert_eq!(p.request, *req.request_hash());
    let actual: HistoryRoot = read(&history, p.next.as_blob());
    let expected = HistoryRoot {
        schema_version: 1,
        revision: 1,
        next_save_sequence: 2,
        trials: BTreeMap::from([(
            t.id.clone(),
            TrialHead {
                document: blob(&serde_json::to_vec(&t).unwrap()),
                admitted_sequence: 1,
            },
        )]),
        receipts: BTreeMap::from([(
            op(1),
            OperationReceipt {
                request: req.request_hash().clone(),
                committed_revision: 1,
                affected: vec![t.id.clone()],
            },
        )]),
    };
    assert_eq!(actual, expected);
    assert_eq!(
        read::<TrialRevision>(&history, &actual.trials[&t.id].document),
        t
    );
    assert_eq!(
        fs::read(object_path(&history, &t.input.audio.original)).unwrap(),
        &*bytes
    );
    assert_eq!(fs::read(source).unwrap(), b"changed selection source");
}

struct Fault {
    point: HistoryCheckpoint,
    armed: Cell<bool>,
    history: PathBuf,
    operation: OperationId,
    original: BlobRef,
    observed_intent: Cell<bool>,
}
impl HistoryFaults for Fault {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
        if matches!(point, HistoryCheckpoint::BeforeObject { index: 0 }) {
            let v: serde_json::Value = serde_json::from_slice(
                &fs::read(
                    self.history
                        .join("recovery")
                        .join(format!("{}.json", self.operation.as_str())),
                )
                .expect("intent before original"),
            )
            .unwrap();
            assert_eq!(v["operation"], self.operation.as_str());
            assert_eq!(v["schema_version"], 1);
            assert!(!object_path(&self.history, &self.original).exists());
            self.observed_intent.set(true);
        }
        if self.armed.get() && point == self.point {
            self.armed.set(false);
            Err(HistoryError {
                kind: HistoryErrorKind::Io,
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn w02_fault_boundaries_preserve_bytes_and_publish_only_after_intent() {
    for point in [
        HistoryCheckpoint::BeforeIntent,
        HistoryCheckpoint::AfterIntent,
        HistoryCheckpoint::BeforeObject { index: 0 },
        HistoryCheckpoint::AfterObject { index: 0 },
        HistoryCheckpoint::BeforeObject { index: 1 },
        HistoryCheckpoint::AfterObject { index: 1 },
        HistoryCheckpoint::BeforeObject { index: 2 },
        HistoryCheckpoint::AfterObject { index: 2 },
        HistoryCheckpoint::BeforeClosure,
        HistoryCheckpoint::AfterClosure,
    ] {
        let mut f = Fixture::new();
        let history = f.history();
        let old = f.put_bytes(b"old orphan must survive");
        let before = tree(&history);
        let t = trial(1);
        let bytes: Arc<[u8]> = wav().into();
        let req = FrozenHistoryRequest::new(create(&t), vec![bytes.clone()]).unwrap();
        let head = Head::default();
        let fault = Fault {
            point,
            armed: Cell::new(true),
            history: history.clone(),
            operation: op(1),
            original: t.input.audio.original.clone(),
            observed_intent: Cell::new(false),
        };
        let mut w =
            HistoryWriter::with_faults(&mut f.storage, &head, HistoryQuota::v1(), &fault).unwrap();
        let preparation = w.prepare(op(1), None, &req);
        let expected_count = match point {
            HistoryCheckpoint::BeforeIntent | HistoryCheckpoint::AfterIntent => {
                kind(preparation, HistoryErrorKind::Io);
                0
            }
            _ => {
                let p = ready(preparation);
                kind(w.publish(&p), HistoryErrorKind::Io);
                match point {
                    HistoryCheckpoint::BeforeObject { index } => index,
                    HistoryCheckpoint::AfterObject { index } => index + 1,
                    _ => 3,
                }
            }
        };
        preserved(&history, &before);
        assert_eq!(&*bytes, wav());
        assert_eq!(*head.current.borrow(), None);
        let count = fs::read_dir(history.join("assets"))
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|x| x == "bin")
            })
            .count();
        assert_eq!(count, expected_count + 1, "boundary {point:?}");
        assert_eq!(
            fs::read(object_path(&history, &old)).unwrap(),
            b"old orphan must survive"
        );
        let intent = history
            .join("recovery")
            .join(format!("{}.json", op(1).as_str()));
        assert_eq!(intent.exists(), point != HistoryCheckpoint::BeforeIntent);
        if expected_count > 0 {
            assert!(fault.observed_intent.get());
        }
    }
}

#[test]
fn w03_token_from_another_writer_session_is_rejected_and_intent_is_busy() {
    let mut f = Fixture::new();
    let history = f.history();
    let t = trial(1);
    let req = FrozenHistoryRequest::new(create(&t), vec![wav().into()]).unwrap();
    let head = Head::default();
    let fault = Fault {
        point: HistoryCheckpoint::AfterObject { index: 0 },
        armed: Cell::new(true),
        history: history.clone(),
        operation: op(1),
        original: t.input.audio.original.clone(),
        observed_intent: Cell::new(false),
    };
    let mut w =
        HistoryWriter::with_faults(&mut f.storage, &head, HistoryQuota::v1(), &fault).unwrap();
    let p = ready(w.prepare(op(1), None, &req));
    let total = p.quota_plan().projected_bytes;
    kind(w.publish(&p), HistoryErrorKind::Io);
    drop(w);
    // Cross-session token is intentionally invalid; explicit resume is I1b-2b.
    let mut w =
        HistoryWriter::new(&mut f.storage, &head, HistoryQuota { limit_bytes: total }).unwrap();
    assert!(w.publish(&p).is_err());
    kind(w.prepare(op(1), None, &req), HistoryErrorKind::Busy);
}

struct Once(Cell<bool>);
impl HistoryFaults for Once {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
        if point == (HistoryCheckpoint::AfterObject { index: 0 }) && self.0.replace(false) {
            Err(HistoryError {
                kind: HistoryErrorKind::Io,
            })
        } else {
            Ok(())
        }
    }
}
#[test]
fn w03_same_session_retry_is_idempotent_and_keeps_the_intent() {
    let mut f = Fixture::new();
    let history = f.history();
    let req = FrozenHistoryRequest::new(create(&trial(1)), vec![wav().into()]).unwrap();
    let head = Head::default();
    let fault = Once(Cell::new(true));
    let (objects, intent, _) = create_plan(&trial(1), &op(1));
    let mut w = HistoryWriter::with_faults(
        &mut f.storage,
        &head,
        HistoryQuota {
            limit_bytes: objects + intent,
        },
        &fault,
    )
    .unwrap();
    let p = ready(w.prepare(op(1), None, &req));
    kind(w.publish(&p), HistoryErrorKind::Io);
    let first = published(w.publish(&p));
    let before = tree(&history);
    let again = published(w.publish(&p));
    assert_eq!(again, first);
    assert_eq!(tree(&history), before);
    assert_eq!(
        w.quota().unwrap().retained_bytes,
        p.quota_plan().projected_bytes
    );
}

#[test]
fn w03_receipt_precedes_stale_base_and_rejects_different_request_or_affected() {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let t = trial(1);
        let req = request(create(&t));
        let (mut root, _) = f.seed(&[t]);
        root.revision = 9;
        root.receipts.insert(
            op(1),
            OperationReceipt {
                request: if mode == 1 {
                    digest(b"different")
                } else {
                    req.request_hash().clone()
                },
                committed_revision: 1,
                affected: vec![if mode == 2 { tid(2) } else { tid(1) }],
            },
        );
        if mode == 2 {
            let t2 = trial(2);
            root.trials.insert(
                tid(2),
                TrialHead {
                    document: f.put(&t2),
                    admitted_sequence: 2,
                },
            );
            root.next_save_sequence = 3;
        }
        let base = RootRef::new(f.put(&root));
        let head = Head::at(base.clone());
        let before = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let out = w.prepare(op(1), None, &req);
        if mode == 0 {
            match out.unwrap() {
                PrepareOutcome::AlreadyCommitted(c) => {
                    assert_eq!(c.current, base);
                    assert_eq!(c.receipt.committed_revision, 1);
                }
                x => panic!("{x:?}"),
            }
        } else {
            kind(out, HistoryErrorKind::Conflict);
        }
        assert_eq!(tree(&f.history()), before);
    }
}

#[test]
fn w03_pending_intent_conflicts_with_changed_request() {
    let mut f = Fixture::new();
    let head = Head::default();
    let a = FrozenHistoryRequest::new(create(&trial(1)), vec![wav().into()]).unwrap();
    let b = FrozenHistoryRequest::new(create(&trial(2)), vec![wav().into()]).unwrap();
    let history = f.history();
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    let p = ready(w.prepare(op(1), None, &a));
    drop(p);
    let before = tree(&history);
    kind(w.prepare(op(1), None, &a), HistoryErrorKind::Busy);
    kind(w.prepare(op(1), None, &b), HistoryErrorKind::Conflict);
    assert_eq!(tree(&history), before);
}

#[test]
fn w04_new_runs_keep_other_maps_reflections_and_admission_sequence() {
    for is_advice in [false, true] {
        let mut f = Fixture::new();
        let mut t = trial(1);
        let old_run = analysis(&t, 1);
        t.analysis.insert(old_run.id.clone(), f.put(&old_run));
        t.reflections.push(f.put(&reflection(&t)));
        let (root, base) = f.seed(&[t.clone()]);
        let before = tree(&f.history());
        let mut expected = t.clone();
        let m = if is_advice {
            let r = advice(&t, 2);
            expected
                .advice
                .insert(r.id.clone(), blob(&serde_json::to_vec(&r).unwrap()));
            HistoryMutation::AddAdviceRun {
                target: target(&t),
                run: r,
            }
        } else {
            let r = analysis(&t, 2);
            expected
                .analysis
                .insert(r.id.clone(), blob(&serde_json::to_vec(&r).unwrap()));
            HistoryMutation::AddAnalysisRun {
                target: target(&t),
                run: r,
            }
        };
        let p = publish_mutation(&mut f, &base, m, vec![]);
        assert_update(&f, &root, &t, &p, expected);
        preserved(&f.history(), &before);
    }
}

#[test]
fn w04_running_analysis_accepts_failed_cancelled_timeout_and_interrupted_once() {
    for state in [
        AnalysisRunState::Failed,
        AnalysisRunState::Cancelled,
        AnalysisRunState::TimedOut,
        AnalysisRunState::Interrupted,
    ] {
        let mut f = Fixture::new();
        let mut t = trial(1);
        let running = analysis(&t, 1);
        let running_ref = f.put(&running);
        t.analysis.insert(running.id.clone(), running_ref.clone());
        let (root, base) = f.seed(&[t.clone()]);
        let before = tree(&f.history());
        let mut terminal = running.clone();
        terminal.state = state;
        terminal.ended_at = Some(at());
        terminal.failure = Some(FailureCode {
            domain: FailureDomain::Analysis,
            code: "interrupted".into(),
        });
        let mut expected = t.clone();
        expected.analysis.insert(
            terminal.id.clone(),
            blob(&serde_json::to_vec(&terminal).unwrap()),
        );
        let p = publish_mutation(
            &mut f,
            &base,
            HistoryMutation::AddAnalysisRun {
                target: target(&t),
                run: terminal.clone(),
            },
            vec![],
        );
        assert_update(&f, &root, &t, &p, expected.clone());
        preserved(&f.history(), &before);
        assert_eq!(read::<AnalysisRun>(&f.history(), &running_ref), running);
        expected.revision += 1;
        expected.previous = Some(target(&t).expected);
        let head = Head::at(p.next.clone());
        let req = request(HistoryMutation::AddAnalysisRun {
            target: target(&expected),
            run: terminal,
        });
        let after = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        kind(
            w.prepare(op(11), Some(&p.next), &req),
            HistoryErrorKind::Conflict,
        );
        assert_eq!(tree(&f.history()), after);
    }
}

#[test]
fn w04_sending_advice_saves_received_result_and_preserves_unknown_actual_model() {
    let mut f = Fixture::new();
    let mut t = trial(1);
    let mut run = advice(&t, 2);
    t.advice.insert(run.id.clone(), f.put(&run));
    let (root, base) = f.seed(&[t.clone()]);
    let before = tree(&f.history());
    let bytes = serde_json::to_vec(&feedback()).unwrap();
    run.state = AdviceRunState::Assessed;
    run.send = SendCertainty::ResponseReceived;
    run.ended_at = Some(at());
    run.result = Some(blob(&bytes));
    let mut expected = t.clone();
    expected
        .advice
        .insert(run.id.clone(), blob(&serde_json::to_vec(&run).unwrap()));
    let p = publish_mutation(
        &mut f,
        &base,
        HistoryMutation::AddAdviceRun {
            target: target(&t),
            run,
        },
        vec![bytes.clone().into()],
    );
    assert_update(&f, &root, &t, &p, expected);
    preserved(&f.history(), &before);
    assert_eq!(
        fs::read(object_path(&f.history(), &blob(&bytes))).unwrap(),
        bytes
    );
}

#[test]
fn w04_run_immutable_fields_and_cross_kind_id_reuse_are_rejected() {
    for variant in 0..4 {
        let mut f = Fixture::new();
        let mut t = trial(1);
        let mut run = analysis(&t, 1);
        t.analysis.insert(run.id.clone(), f.put(&run));
        let (_, base) = f.seed(&[t.clone()]);
        run.state = AnalysisRunState::Interrupted;
        run.ended_at = Some(at());
        run.failure = Some(FailureCode {
            domain: FailureDomain::Analysis,
            code: "interrupted".into(),
        });
        let m = match variant {
            0 => {
                run.pcm = digest(b"different");
                HistoryMutation::AddAnalysisRun {
                    target: target(&t),
                    run,
                }
            }
            1 => {
                run.started_at.utc_millis -= 1;
                HistoryMutation::AddAnalysisRun {
                    target: target(&t),
                    run,
                }
            }
            2 => {
                run.decoder.version = "2".into();
                HistoryMutation::AddAnalysisRun {
                    target: target(&t),
                    run,
                }
            }
            _ => HistoryMutation::AddAdviceRun {
                target: target(&t),
                run: advice(&t, 1),
            },
        };
        let req = request(m);
        let head = Head::at(base.clone());
        let before = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        kind(
            w.prepare(op(10), Some(&base), &req),
            HistoryErrorKind::Conflict,
        );
        assert_eq!(tree(&f.history()), before);
    }
}

#[test]
fn w04_reflection_and_match_evidence_append_without_erasing_old_values() {
    for evidence in [false, true] {
        let mut f = Fixture::new();
        let mut t = trial(1);
        let old = Reflection {
            text: "Earlier reflection".into(),
            ..reflection(&t)
        };
        t.reflections.push(f.put(&old));
        let (root, base) = f.seed(&[t.clone()]);
        let mut expected = t.clone();
        let before = tree(&f.history());
        let m = if evidence {
            let e = MatchEvidence::HumanConfirmed {
                at: at(),
                audio: t.input.audio.original.sha256.clone(),
                reference: t.input.reference.sha256.clone(),
            };
            expected.match_evidence.push(e.clone());
            HistoryMutation::AddMatchEvidence {
                target: target(&t),
                evidence: e,
            }
        } else {
            let r = reflection(&t);
            expected
                .reflections
                .push(blob(&serde_json::to_vec(&r).unwrap()));
            HistoryMutation::AddReflection {
                target: target(&t),
                reflection: r,
            }
        };
        let p = publish_mutation(&mut f, &base, m, vec![]);
        assert_update(&f, &root, &t, &p, expected);
        preserved(&f.history(), &before);
    }
}

#[test]
fn w06_stale_base_target_pending_and_unreadable_head_never_write() {
    for mode in 0..4 {
        let f = Fixture::new();
        let t = trial(1);
        let (_, base) = f.seed(&[t.clone()]);
        let head = Head::at(base.clone());
        let mut tgt = target(&t);
        if mode == 1 {
            tgt.expected = blob(b"stale target");
        }
        if mode == 2 {
            fs::write(
                f.storage.dir.join("pending-material-commit.json"),
                b"pending",
            )
            .unwrap();
        }
        if mode == 3 {
            head.fail.set(true);
        }
        let req = request(HistoryMutation::AddReflection {
            target: tgt,
            reflection: reflection(&t),
        });
        let storage_dir = f.storage.dir.clone();
        // Windows prevents reading app.lock while Storage holds its exclusive
        // lock. Snapshot every byte, including app.lock, between lock lifetimes.
        drop(f.storage);
        let before = tree(&storage_dir);
        let result = {
            let mut storage = Storage::at(storage_dir.clone()).unwrap();
            let result = match HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()) {
                Ok(mut w) => w.prepare(op(1), if mode == 0 { None } else { Some(&base) }, &req),
                Err(e) => Err(e),
            };
            result
        };
        kind(
            result,
            match mode {
                2 => HistoryErrorKind::PendingMaterial,
                3 => HistoryErrorKind::Io,
                _ => HistoryErrorKind::Conflict,
            },
        );
        assert_eq!(tree(&storage_dir), before);
    }
}

#[test]
fn w06_head_or_pending_change_between_prepare_and_publish_stops_objects() {
    for pending in [false, true] {
        let mut f = Fixture::new();
        let t = trial(1);
        let (_, base) = f.seed(&[t.clone()]);
        let head = Head::at(base.clone());
        let history = f.history();
        let storage_dir = f.storage.dir.clone();
        let req = request(HistoryMutation::AddReflection {
            target: target(&t),
            reflection: reflection(&t),
        });
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let p = ready(w.prepare(op(1), Some(&base), &req));
        let before = tree(&history);
        if pending {
            fs::write(storage_dir.join("pending-material-commit.json"), b"pending").unwrap();
        } else {
            *head.current.borrow_mut() = None;
        }
        kind(
            w.publish(&p),
            if pending {
                HistoryErrorKind::PendingMaterial
            } else {
                HistoryErrorKind::Conflict
            },
        );
        assert_eq!(tree(&history), before);
    }
}

#[test]
fn w06_second_storage_cannot_take_the_existing_lock() {
    let f = Fixture::new();
    let before = tree(&f.history());
    assert!(Storage::at(f.storage.dir.clone()).is_err());
    assert_eq!(tree(&f.history()), before);
}

#[test]
fn w07_duplicate_confirmation_includes_active_trash_and_reference() {
    for confirmed in [false, true] {
        let mut f = Fixture::new();
        let a = trial(1);
        let mut b = trial(2);
        b.trashed_at = Some(at());
        let (old, base) = f.seed(&[a, b]);
        let t = trial(3);
        let mut m = create(&t);
        if let HistoryMutation::CreateTrial { duplicate, .. } = &mut m {
            if confirmed {
                *duplicate = Some(DuplicateConfirmation {
                    original: t.input.audio.original.sha256.clone(),
                    matches: vec![tid(1), tid(2)],
                    confirmed_at: at(),
                });
            }
        }
        let req = request(m);
        let head = Head::at(base.clone());
        let history = f.history();
        let before = tree(&history);
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let out = w.prepare(op(1), Some(&base), &req);
        if confirmed {
            let p = ready(out);
            let next = published(w.publish(&p));
            let root: HistoryRoot = read(&history, next.next.as_blob());
            assert_eq!(root.trials.len(), 3);
            assert_eq!(root.next_save_sequence, 4);
            assert_eq!(root.trials[&tid(3)].admitted_sequence, 3);
            for (id, old_head) in old.trials {
                assert_eq!(root.trials[&id], old_head);
            }
            preserved(&history, &before);
        } else {
            match out.unwrap() {
                PrepareOutcome::DuplicateConfirmationRequired(ids) => {
                    assert_eq!(ids, vec![tid(1), tid(2)])
                }
                x => panic!("{x:?}"),
            };
            assert_eq!(tree(&history), before);
        }
    }
}

#[test]
fn w07_stale_duplicate_set_requires_reconfirmation_and_practice_keeps_identity() {
    for stale in [true, false] {
        let mut f = Fixture::new();
        let t = trial(1);
        let other = trial(2);
        let (root, base) = f.seed(&[t.clone(), other]);
        let m = HistoryMutation::ConfirmPractice {
            target: target(&t),
            claim: practice(true),
            duplicate: Some(DuplicateConfirmation {
                original: t.input.audio.original.sha256.clone(),
                matches: if stale { vec![] } else { vec![tid(2)] },
                confirmed_at: at(),
            }),
        };
        if stale {
            let req = request(m);
            let head = Head::at(base.clone());
            let before = tree(&f.history());
            let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
            match w.prepare(op(10), Some(&base), &req).unwrap() {
                PrepareOutcome::DuplicateConfirmationRequired(ids) => assert_eq!(ids, vec![tid(2)]),
                x => panic!("{x:?}"),
            };
            assert_eq!(tree(&f.history()), before);
        } else {
            let mut expected = t.clone();
            expected.claim = practice(true);
            let p = publish_mutation(&mut f, &base, m, vec![]);
            assert_update(&f, &root, &t, &p, expected);
        }
    }
}

#[test]
fn w07_running_or_sending_blocks_trash_without_losing_data() {
    for is_advice in [false, true] {
        let mut f = Fixture::new();
        let mut t = trial(1);
        if is_advice {
            let r = advice(&t, 1);
            t.advice.insert(r.id.clone(), f.put(&r));
        } else {
            let r = analysis(&t, 1);
            t.analysis.insert(r.id.clone(), f.put(&r));
        }
        let (_, base) = f.seed(&[t.clone()]);
        let head = Head::at(base.clone());
        let req = request(HistoryMutation::SetTrash {
            target: target(&t),
            trashed_at: Some(at()),
        });
        let before = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        kind(w.prepare(op(1), Some(&base), &req), HistoryErrorKind::Busy);
        assert_eq!(tree(&f.history()), before);
    }
}

#[test]
fn w07_trash_and_restore_preserve_same_id_original_and_results() {
    for restore in [false, true] {
        let mut f = Fixture::new();
        let mut t = trial(1);
        let mut r = analysis(&t, 1);
        r.state = AnalysisRunState::Cancelled;
        r.ended_at = Some(at());
        r.failure = Some(FailureCode {
            domain: FailureDomain::Analysis,
            code: "cancelled".into(),
        });
        t.analysis.insert(r.id.clone(), f.put(&r));
        t.reflections.push(f.put(&reflection(&t)));
        if restore {
            t.trashed_at = Some(at());
        }
        let (root, base) = f.seed(&[t.clone()]);
        let before = tree(&f.history());
        let mut expected = t.clone();
        expected.trashed_at = if restore { None } else { Some(at()) };
        let p = publish_mutation(
            &mut f,
            &base,
            HistoryMutation::SetTrash {
                target: target(&t),
                trashed_at: expected.trashed_at,
            },
            vec![],
        );
        assert_update(&f, &root, &t, &p, expected);
        preserved(&f.history(), &before);
    }
}

#[test]
fn w07_trash_rejects_late_results_reflection_and_confirmation() {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let mut t = trial(1);
        t.trashed_at = Some(at());
        let (_, base) = f.seed(&[t.clone()]);
        let m = match mode {
            0 => HistoryMutation::AddAnalysisRun {
                target: target(&t),
                run: analysis(&t, 1),
            },
            1 => HistoryMutation::AddReflection {
                target: target(&t),
                reflection: reflection(&t),
            },
            _ => HistoryMutation::ConfirmPractice {
                target: target(&t),
                claim: practice(false),
                duplicate: None,
            },
        };
        let req = request(m);
        let head = Head::at(base.clone());
        let before = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        kind(
            w.prepare(op(1), Some(&base), &req),
            HistoryErrorKind::Conflict,
        );
        assert_eq!(tree(&f.history()), before);
    }
}

// Independent exact byte oracle. Key order does not affect serialized length;
// embedded mutation_json is the exact DTO JSON, including its escaping.
fn create_plan(t: &TrialRevision, operation: &OperationId) -> (u64, u64, Vec<BlobRef>) {
    let mutation_json = serde_json::to_string(&create(t)).unwrap();
    let trial_ref = blob(&serde_json::to_vec(t).unwrap());
    let root = HistoryRoot {
        schema_version: 1,
        revision: 1,
        next_save_sequence: 2,
        trials: BTreeMap::from([(
            t.id.clone(),
            TrialHead {
                document: trial_ref.clone(),
                admitted_sequence: 1,
            },
        )]),
        receipts: BTreeMap::from([(
            operation.clone(),
            OperationReceipt {
                request: digest(mutation_json.as_bytes()),
                committed_revision: 1,
                affected: vec![t.id.clone()],
            },
        )]),
    };
    let root_ref = RootRef::new(blob(&serde_json::to_vec(&root).unwrap()));
    let refs = vec![
        t.input.audio.original.clone(),
        trial_ref,
        root_ref.as_blob().clone(),
    ];
    let intent = json!({ "schema_version":1,"operation":operation,"request":digest(mutation_json.as_bytes()),"mutation_json":mutation_json,"base":null,"planned_root":root_ref,"planned_objects":refs });
    (
        refs.iter().map(|r| r.bytes).sum(),
        serde_json::to_vec(&intent).unwrap().len() as u64,
        refs,
    )
}

#[test]
fn w05_exact_quota_plan_accepts_limit_and_rejects_one_byte_below_before_write() {
    let t = trial(1);
    let (objects, intent, _) = create_plan(&t, &op(1));
    for delta in [-1i64, 0, 1] {
        let mut f = Fixture::new();
        let history = f.history();
        let head = Head::default();
        let req = FrozenHistoryRequest::new(create(&t), vec![wav().into()]).unwrap();
        let limit = (objects + intent) as i64 + delta;
        let mut w = HistoryWriter::new(
            &mut f.storage,
            &head,
            HistoryQuota {
                limit_bytes: limit as u64,
            },
        )
        .unwrap();
        let out = w.prepare(op(1), None, &req);
        if delta < 0 {
            kind(out, HistoryErrorKind::QuotaExceeded);
            assert!(!history.exists());
        } else {
            let p = ready(out);
            assert_eq!(p.quota_plan().before.retained_bytes, 0);
            assert_eq!(p.quota_plan().added_object_bytes, objects);
            assert_eq!(p.quota_plan().intent_delta_bytes, intent as i64);
            assert_eq!(p.quota_plan().projected_bytes, objects + intent);
            published(w.publish(&p));
            assert_eq!(w.quota().unwrap().retained_bytes, objects + intent);
        }
    }
}

#[test]
fn w05_ledger_counts_orphans_temp_and_trash_once_and_warns_at_ceil_eighty_percent() {
    let mut f = Fixture::new();
    let mut t = trial(1);
    t.trashed_at = Some(at());
    f.seed(&[t]);
    f.put_bytes(b"orphan");
    fs::write(f.history().join("assets/.tmp-fixture"), b"staging").unwrap();
    let files = tree(&f.history());
    let object_bytes: u64 = files
        .iter()
        .filter(|(p, _)| p.extension().is_some_and(|x| x == "bin"))
        .map(|(_, b)| b.len() as u64)
        .sum();
    let total = object_bytes + 7;
    let head = Head::default();
    for limit in [total * 5 / 4, total * 5 / 4 + 1, u64::MAX] {
        let w =
            HistoryWriter::new(&mut f.storage, &head, HistoryQuota { limit_bytes: limit }).unwrap();
        let q = w.quota().unwrap();
        assert_eq!(q.object_bytes, object_bytes);
        assert_eq!(q.residual_bytes, 7);
        assert_eq!(q.recovery_bytes, 0);
        assert_eq!(q.retained_bytes, total);
        assert_eq!(q.warning, u128::from(total) * 5 >= u128::from(limit) * 4);
    }
    assert_eq!(tree(&f.history()), files);
}

#[test]
fn w05_lowered_quota_preserves_existing_closure_and_refuses_new_save() {
    let mut f = Fixture::new();
    let t = trial(1);
    let (_, base) = f.seed(&[t.clone()]);
    let before = tree(&f.history());
    let head = Head::at(base.clone());
    let req = request(HistoryMutation::AddReflection {
        target: target(&t),
        reflection: reflection(&t),
    });
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota { limit_bytes: 1 }).unwrap();
    assert!(w.quota().unwrap().retained_bytes > 1);
    kind(
        w.prepare(op(1), Some(&base), &req),
        HistoryErrorKind::QuotaExceeded,
    );
    assert_eq!(tree(&f.history()), before);
}

#[test]
fn w05_zero_quota_and_sequence_overflow_do_not_create_history() {
    let mut f = Fixture::new();
    let head = Head::default();
    assert!(HistoryWriter::new(&mut f.storage, &head, HistoryQuota { limit_bytes: 0 }).is_err());
    assert!(!f.history().exists());
    let t = trial(1);
    let (mut root, _) = f.seed(&[t.clone()]);
    root.next_save_sequence = u64::MAX;
    let base = RootRef::new(f.put(&root));
    let head = Head::at(base.clone());
    let mut m = create(&trial(2));
    if let HistoryMutation::CreateTrial { duplicate, .. } = &mut m {
        *duplicate = Some(DuplicateConfirmation {
            original: t.input.audio.original.sha256,
            matches: vec![tid(1)],
            confirmed_at: at(),
        });
    }
    let req = request(m);
    let before = tree(&f.history());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    kind(
        w.prepare(op(1), Some(&base), &req),
        HistoryErrorKind::LimitExceeded,
    );
    assert_eq!(tree(&f.history()), before);
}

fn advice_intent_size(
    t: &TrialRevision,
    root: &HistoryRoot,
    base: &RootRef,
    run: &AdviceRun,
) -> usize {
    let m = HistoryMutation::AddAdviceRun {
        target: target(t),
        run: run.clone(),
    };
    let exact = serde_json::to_string(&m).unwrap();
    let run_ref = blob(&serde_json::to_vec(run).unwrap());
    let mut next_trial = t.clone();
    next_trial.revision += 1;
    next_trial.previous = Some(target(t).expected);
    next_trial.advice.insert(run.id.clone(), run_ref.clone());
    let trial_ref = blob(&serde_json::to_vec(&next_trial).unwrap());
    let mut next_root = root.clone();
    next_root.revision += 1;
    next_root.trials.get_mut(&t.id).unwrap().document = trial_ref.clone();
    next_root.receipts.insert(
        op(10),
        OperationReceipt {
            request: digest(exact.as_bytes()),
            committed_revision: next_root.revision,
            affected: vec![t.id.clone()],
        },
    );
    let root_ref = RootRef::new(blob(&serde_json::to_vec(&next_root).unwrap()));
    serde_json::to_vec(&json!({"schema_version":1,"operation":op(10),"request":digest(exact.as_bytes()),"mutation_json":exact,"base":base,"planned_root":root_ref,"planned_objects":[run_ref,trial_ref,root_ref.as_blob()]})).unwrap().len()
}

#[test]
fn w05_intent_escaped_exact_bytes_512k_boundary_is_checked_before_any_write() {
    for wanted in [524_287usize, 524_288, 524_289] {
        let mut f = Fixture::new();
        let t = trial(1);
        let (root, base) = f.seed(&[t.clone()]);
        let mut run = advice(&t, 1);
        // Backslashes are escaped once in the mutation and again in intent.
        // A run document may be this large; the Trial still only stores its ref.
        run.destination = format!("https://fixture.invalid/{}", "\\".repeat(129_000));
        let current = advice_intent_size(&t, &root, &base, &run);
        assert!(current < wanted);
        run.destination.push_str(&"x".repeat(wanted - current));
        assert_eq!(advice_intent_size(&t, &root, &base, &run), wanted);
        run.validate().unwrap();
        let mutation = HistoryMutation::AddAdviceRun {
            target: target(&t),
            run,
        };
        assert!(
            serde_json::to_vec(&mutation).unwrap().len() < 524_288,
            "only the escaped intent crosses this boundary"
        );
        let req = request(mutation);
        let head = Head::at(base.clone());
        let before = tree(&f.history());
        let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
        let out = w.prepare(op(10), Some(&base), &req);
        if wanted > 524_288 {
            kind(out, HistoryErrorKind::LimitExceeded);
            assert_eq!(tree(&f.history()), before);
        } else {
            let prepared = ready(out);
            assert_eq!(prepared.quota_plan().intent_delta_bytes, wanted as i64);
            let p = published(w.publish(&prepared));
            drop(w);
            HistoryReader::new(&f.storage)
                .verify_closure(&p.next)
                .unwrap();
            preserved(&f.history(), &before);
        }
    }
}

#[test]
fn w01_missing_payload_is_not_replaced_with_empty_audio() {
    let mut f = Fixture::new();
    let head = Head::default();
    let req = request(create(&trial(1)));
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    kind(
        w.prepare(op(1), None, &req),
        HistoryErrorKind::MissingObject,
    );
    assert!(!f.history().exists());
}

#[test]
fn w04_missing_result_does_not_modify_running_advice() {
    let mut f = Fixture::new();
    let mut t = trial(1);
    let mut run = advice(&t, 1);
    t.advice.insert(run.id.clone(), f.put(&run));
    let (_, base) = f.seed(&[t.clone()]);
    run.state = AdviceRunState::Assessed;
    run.send = SendCertainty::ResponseReceived;
    run.ended_at = Some(at());
    run.result = Some(blob(&serde_json::to_vec(&feedback()).unwrap()));
    let req = request(HistoryMutation::AddAdviceRun {
        target: target(&t),
        run,
    });
    let head = Head::at(base.clone());
    let before = tree(&f.history());
    let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
    kind(
        w.prepare(op(10), Some(&base), &req),
        HistoryErrorKind::MissingObject,
    );
    assert_eq!(tree(&f.history()), before);
}

#[test]
fn w04_interrupted_advice_keeps_may_have_been_sent_certainty() {
    let mut f = Fixture::new();
    let mut t = trial(1);
    let mut run = advice(&t, 1);
    t.advice.insert(run.id.clone(), f.put(&run));
    let (root, base) = f.seed(&[t.clone()]);
    let before = tree(&f.history());
    run.state = AdviceRunState::Interrupted;
    run.send = SendCertainty::MayHaveBeenSent;
    run.ended_at = Some(at());
    run.failure = Some(FailureCode {
        domain: FailureDomain::Advice,
        code: "interrupted".into(),
    });
    let mut expected = t.clone();
    expected
        .advice
        .insert(run.id.clone(), blob(&serde_json::to_vec(&run).unwrap()));
    let p = publish_mutation(
        &mut f,
        &base,
        HistoryMutation::AddAdviceRun {
            target: target(&t),
            run,
        },
        vec![],
    );
    assert_update(&f, &root, &t, &p, expected);
    preserved(&f.history(), &before);
}

#[test]
fn w06_damaged_existing_object_cannot_be_repaired_by_overwrite() {
    let mut f = Fixture::new();
    let t = trial(1);
    let (_, base) = f.seed(&[t.clone()]);
    fs::write(
        object_path(&f.history(), &t.input.audio.original),
        b"corrupt old original",
    )
    .unwrap();
    let req = request(HistoryMutation::AddReflection {
        target: target(&t),
        reflection: reflection(&t),
    });
    let head = Head::at(base.clone());
    let before = tree(&f.history());
    let out = match HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()) {
        Ok(mut w) => w.prepare(op(10), Some(&base), &req),
        Err(e) => Err(e),
    };
    kind(out, HistoryErrorKind::DamagedObject);
    assert_eq!(tree(&f.history()), before);
}

#[test]
fn w06_publish_stops_when_intent_directory_is_replaced_by_regular_file() {
    let mut f = Fixture::new();
    let history = f.history();
    fs::create_dir(&history).unwrap();
    fs::write(history.join("recovery"), b"must never overwrite").unwrap();
    let req = FrozenHistoryRequest::new(create(&trial(1)), vec![wav().into()]).unwrap();
    let head = Head::default();
    let before = tree(&history);
    let out = match HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()) {
        Ok(mut w) => w.prepare(op(1), None, &req),
        Err(e) => Err(e),
    };
    assert!(out.is_err());
    assert_eq!(tree(&history), before);
}

#[test]
fn w04_successful_partial_analysis_keeps_exact_report_and_previous_run() {
    use acoustic_analysis::{
        AnalysisReport, AnalysisStatus, Measures, MissingReason, PitchMeasure, SampleValue,
        TimedValue,
    };
    let mut f = Fixture::new();
    let mut t = trial(1);
    let mut run = analysis(&t, 1);
    let old_run = f.put(&run);
    t.analysis.insert(run.id.clone(), old_run.clone());
    let (root, base) = f.seed(&[t.clone()]);
    let before = tree(&f.history());
    // The report is a manually calculated one-frame constant signal oracle.
    let rms = 20.0 * (1234.0_f64 / 32768.0).log10();
    let report = AnalysisReport {
        schema_version: 1,
        metrics_version: "acoustic-metrics-v1".into(),
        library_version: "0.1.0".into(),
        profile: AnalysisProfile::v1(),
        sample_rate: 8000,
        channels: 1,
        input_frames: 1,
        pitch_engine: None,
        status: AnalysisStatus::Partial,
        measures: Measures {
            duration_seconds: 1.0 / 8000.0,
            rms_dbfs: SampleValue::Finite(rms),
            rms_frames: vec![TimedValue {
                time_seconds: 1.0 / 16000.0,
                value: SampleValue::Finite(rms),
            }],
            low_intervals: vec![],
            low_total_seconds: 0.0,
            low_fraction: 0.0,
            pitch: PitchMeasure {
                frames: vec![],
                median_hz: SampleValue::Missing(MissingReason::TooShort),
                p10_hz: SampleValue::Missing(MissingReason::TooShort),
                p90_hz: SampleValue::Missing(MissingReason::TooShort),
                valid_frame_fraction: SampleValue::Missing(MissingReason::TooShort),
            },
        },
        quality: vec![],
    };
    report.validate().unwrap();
    let bytes = serde_json::to_vec(&report).unwrap();
    run.state = AnalysisRunState::Partial;
    run.ended_at = Some(at());
    run.result = Some(blob(&bytes));
    let mut expected = t.clone();
    expected
        .analysis
        .insert(run.id.clone(), blob(&serde_json::to_vec(&run).unwrap()));
    let p = publish_mutation(
        &mut f,
        &base,
        HistoryMutation::AddAnalysisRun {
            target: target(&t),
            run,
        },
        vec![bytes.clone().into()],
    );
    assert_update(&f, &root, &t, &p, expected);
    preserved(&f.history(), &before);
    assert_eq!(
        fs::read(object_path(&f.history(), &blob(&bytes))).unwrap(),
        bytes
    );
    assert_eq!(
        read::<AnalysisRun>(&f.history(), &old_run).state,
        AnalysisRunState::Running
    );
}

struct PendingAfterOriginal(PathBuf);
impl HistoryFaults for PendingAfterOriginal {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
        if point == (HistoryCheckpoint::AfterObject { index: 0 }) {
            fs::write(
                self.0.join("pending-material-commit.json"),
                b"synthetic concurrent pending",
            )
            .unwrap();
        }
        Ok(())
    }
}
#[test]
fn w06_pending_created_after_original_stops_before_trial_publication() {
    let mut f = Fixture::new();
    let history = f.history();
    let t = trial(1);
    let req = FrozenHistoryRequest::new(create(&t), vec![wav().into()]).unwrap();
    let head = Head::default();
    let fault = PendingAfterOriginal(f.storage.dir.clone());
    let mut w =
        HistoryWriter::with_faults(&mut f.storage, &head, HistoryQuota::v1(), &fault).unwrap();
    let p = ready(w.prepare(op(1), None, &req));
    kind(w.publish(&p), HistoryErrorKind::PendingMaterial);
    assert_eq!(
        fs::read(object_path(&history, &t.input.audio.original)).unwrap(),
        wav()
    );
    assert!(!object_path(&history, &blob(&serde_json::to_vec(&t).unwrap())).exists());
    assert_eq!(*head.current.borrow(), None);
}

#[test]
fn w05_original_six_mib_boundary_and_overflow_have_no_truncation() {
    for len in [6 * 1024 * 1024, 6 * 1024 * 1024 + 1] {
        let mut f = Fixture::new();
        let history = f.history();
        let mut t = trial(1);
        let mut bytes = wav();
        bytes.resize(len, 0x55);
        t.input.audio.original = blob(&bytes);
        let head = Head::default();
        // This is the storage byte boundary only; codec acceptance is I6.
        let out = FrozenHistoryRequest::new(create(&t), vec![bytes.clone().into()]);
        if len > 6 * 1024 * 1024 {
            match out {
                Err(e) => assert_eq!(e.kind, HistoryErrorKind::LimitExceeded),
                Ok(req) => {
                    let mut w =
                        HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
                    kind(
                        w.prepare(op(1), None, &req),
                        HistoryErrorKind::LimitExceeded,
                    );
                }
            }
            assert!(!history.exists());
        } else {
            let req = out.unwrap();
            let mut w = HistoryWriter::new(&mut f.storage, &head, HistoryQuota::v1()).unwrap();
            let p = ready(w.prepare(op(1), None, &req));
            published(w.publish(&p));
            assert_eq!(
                fs::read(object_path(&history, &t.input.audio.original)).unwrap(),
                bytes
            );
        }
    }
}

// I1b-W1 / W06 / AH-AC04/14: real recovery junctions must never redirect writes.
mod recovery_junction {
    use super::*;
    use std::os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    };

    struct LinkedTemp {
        temp: Option<Temp>,
        links: Vec<PathBuf>,
    }
    impl LinkedTemp {
        fn junction(&mut self, link: &Path, target: &Path) {
            let root = &self.temp.as_ref().unwrap().0;
            assert!(link.starts_with(root) && target.starts_with(root));
            assert!(!link.exists());
            assert!(!target.starts_with(link));
            // Register before setup: even an unwind must unlink before recursion.
            self.links.push(link.to_owned());
            fs::create_dir(link).expect("create isolated junction entry");
            create_junction(link, target).expect(
                "create native junction: fixture/privilege failure is not a behavioral RED",
            );
            assert_ne!(
                fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
                0,
                "fixture must really be a Windows reparse point"
            );
            assert_eq!(
                fs::canonicalize(link).unwrap(),
                fs::canonicalize(target).unwrap()
            );
            assert_eq!(
                tree(link),
                tree(target),
                "link must resolve to all target bytes"
            );
        }
    }
    impl Drop for LinkedTemp {
        fn drop(&mut self) {
            let mut safe_to_clean = true;
            for path in &self.links {
                if let Err(error) = fs::remove_dir(path) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        safe_to_clean = false;
                        eprintln!(
                            "preserving isolated fixture after junction unlink failure: {error}"
                        );
                    }
                }
            }
            if !safe_to_clean {
                if let Some(temp) = self.temp.take() {
                    // Never allow Temp's recursive cleanup through a surviving link.
                    std::mem::forget(temp);
                }
            }
        }
    }

    fn create_junction(link: &Path, target: &Path) -> std::io::Result<()> {
        use std::{ffi::c_void, ptr};
        #[link(name = "kernel32")]
        extern "system" {
            fn DeviceIoControl(
                device: *mut c_void,
                code: u32,
                input: *const c_void,
                input_len: u32,
                output: *mut c_void,
                output_len: u32,
                returned: *mut u32,
                overlapped: *mut c_void,
            ) -> i32;
        }
        // Native IO_REPARSE_TAG_MOUNT_POINT; no shell, UAC or setting changes.
        let canonical = fs::canonicalize(target)?;
        let canonical: Vec<u16> = canonical.as_os_str().encode_wide().collect();
        let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
        assert!(canonical.starts_with(&prefix));
        let print_name = &canonical[prefix.len()..];
        let substitute: Vec<u16> = r"\??\"
            .encode_utf16()
            .chain(print_name.iter().copied())
            .collect();
        let sub_bytes = u16::try_from(substitute.len() * 2).unwrap();
        let print_bytes = u16::try_from(print_name.len() * 2).unwrap();
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&0xa000_0003_u32.to_le_bytes());
        buffer.extend_from_slice(&(8 + sub_bytes + 2 + print_bytes + 2).to_le_bytes());
        for field in [0_u16, 0, sub_bytes, sub_bytes + 2, print_bytes] {
            buffer.extend_from_slice(&field.to_le_bytes());
        }
        for value in substitute
            .iter()
            .copied()
            .chain([0])
            .chain(print_name.iter().copied())
            .chain([0])
        {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0x0200_0000 | 0x0020_0000)
            .open(link)?;
        let mut returned = 0;
        // SAFETY: owned handle and buffers live throughout this synchronous call.
        let success = unsafe {
            DeviceIoControl(
                file.as_raw_handle(),
                0x0009_00a4,
                buffer.as_ptr().cast(),
                u32::try_from(buffer.len()).unwrap(),
                ptr::null_mut(),
                0,
                &mut returned,
                ptr::null_mut(),
            )
        };
        if success == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    // Independent intent oracle, derived from the documented schema and mutation.
    fn reflection_intent(
        t: &TrialRevision,
        root: &HistoryRoot,
        base: &RootRef,
        req: &FrozenHistoryRequest,
        operation: OperationId,
    ) -> serde_json::Value {
        let leaf = blob(&serde_json::to_vec(&reflection(t)).unwrap());
        let mut next_trial = t.clone();
        next_trial.revision += 1;
        next_trial.previous = Some(target(t).expected);
        next_trial.reflections.push(leaf.clone());
        let trial_ref = blob(&serde_json::to_vec(&next_trial).unwrap());
        let mut next_root = root.clone();
        next_root.revision += 1;
        next_root.trials.get_mut(&t.id).unwrap().document = trial_ref.clone();
        next_root.receipts.insert(
            operation.clone(),
            OperationReceipt {
                request: req.request_hash().clone(),
                committed_revision: next_root.revision,
                affected: vec![t.id.clone()],
            },
        );
        let root_ref = RootRef::new(blob(&serde_json::to_vec(&next_root).unwrap()));
        json!({"schema_version":1,"operation":operation,"request":req.request_hash(),
            "mutation_json":req.mutation_json(),"base":base,"planned_root":root_ref,
            "planned_objects":[leaf,trial_ref,root_ref.as_blob()]})
    }

    #[test]
    fn w06_prepare_rejects_recovery_junction_without_changing_any_bytes() {
        let f = Fixture::new();
        let t = trial(1);
        let (root, base) = f.seed(&[t.clone()]);
        let history = f.history();
        let host = f.storage.dir.clone();
        let req = request(HistoryMutation::AddReflection {
            target: target(&t),
            reflection: reflection(&t),
        });
        let Fixture { storage, _temp } = f;
        drop(storage);
        let mut fixture = LinkedTemp {
            temp: Some(_temp),
            links: vec![],
        };
        let target = fixture.temp.as_ref().unwrap().0.join("isolated-recovery");
        fs::create_dir(&target).unwrap();
        fs::write(
            target.join("sentinel"),
            b"synthetic linked target: preserve exactly",
        )
        .unwrap();
        let intent_name = format!("{}.json", op(9).as_str());
        let intent = serde_json::to_vec(&reflection_intent(&t, &root, &base, &req, op(9))).unwrap();
        fs::write(target.join(&intent_name), &intent).unwrap();
        let objects_before = tree(&history.join("assets"));
        let lock_before = fs::read(host.join("app.lock")).unwrap();
        {
            let mut storage = Storage::at(host.clone()).unwrap();
            let head = Head::at(base.clone());
            let mut writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            fixture.junction(&history.join("recovery"), &target);
            let before = tree(&history);
            let target_before = tree(&target);
            for _ in 0..2 {
                kind(
                    writer.prepare(op(10), Some(&base), &req),
                    HistoryErrorKind::UnsafePath,
                );
                assert_eq!(tree(&history), before);
                assert_eq!(tree(&target), target_before);
                assert_eq!(
                    tree(&history.join("assets")),
                    objects_before,
                    "additional objects must be zero"
                );
                assert_eq!(
                    fs::read(object_path(&history, &t.input.audio.original)).unwrap(),
                    wav()
                );
                assert_eq!(fs::read(target.join(&intent_name)).unwrap(), intent);
                assert_eq!(*head.current.borrow(), Some(base.clone()));
            }
        }
        assert_eq!(fs::read(host.join("app.lock")).unwrap(), lock_before);
    }

    #[test]
    fn w06_same_writer_publish_rejects_recovery_junction_and_preserves_prepared_intent() {
        let f = Fixture::new();
        let t = trial(1);
        let (root, base) = f.seed(&[t.clone()]);
        let history = f.history();
        let host = f.storage.dir.clone();
        let req = request(HistoryMutation::AddReflection {
            target: target(&t),
            reflection: reflection(&t),
        });
        let expected_intent = reflection_intent(&t, &root, &base, &req, op(10));
        let Fixture { storage, _temp } = f;
        drop(storage);
        let mut fixture = LinkedTemp {
            temp: Some(_temp),
            links: vec![],
        };
        let target = fixture.temp.as_ref().unwrap().0.join("moved-recovery");
        let lock_before = fs::read(host.join("app.lock")).unwrap();
        {
            let mut storage = Storage::at(host.clone()).unwrap();
            let head = Head::at(base.clone());
            let objects_before = tree(&history.join("assets"));
            let mut writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            let prepared = ready(writer.prepare(op(10), Some(&base), &req));
            let recovery = history.join("recovery");
            let intent_name = format!("{}.json", op(10).as_str());
            let intent =
                fs::read(recovery.join(&intent_name)).expect("successful prepare saved intent");
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&intent).unwrap(),
                expected_intent
            );
            assert_eq!(
                tree(&history.join("assets")),
                objects_before,
                "prepare must not publish objects"
            );
            fs::rename(&recovery, &target).unwrap();
            fs::write(
                target.join("sentinel"),
                b"synthetic linked target: preserve exactly",
            )
            .unwrap();
            fixture.junction(&recovery, &target);
            let before = tree(&history);
            let target_before = tree(&target);
            // Keep the exact writer, Storage and Prepared session across both calls.
            for _ in 0..2 {
                kind(writer.publish(&prepared), HistoryErrorKind::UnsafePath);
                assert_eq!(tree(&history), before);
                assert_eq!(tree(&target), target_before);
                assert_eq!(
                    tree(&history.join("assets")),
                    objects_before,
                    "additional objects must be zero"
                );
                assert_eq!(
                    fs::read(object_path(&history, &t.input.audio.original)).unwrap(),
                    wav()
                );
                assert_eq!(fs::read(target.join(&intent_name)).unwrap(), intent);
                assert_eq!(*head.current.borrow(), Some(base.clone()));
            }
        }
        assert_eq!(fs::read(host.join("app.lock")).unwrap(), lock_before);
    }
}
