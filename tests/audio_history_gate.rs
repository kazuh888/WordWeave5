//! I2a-G: independent public-entry tests. All data is isolated and synthetic.
#![cfg(windows)]

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wordweave5::{
    assets,
    audio_history::*,
    commit, model,
    store::{atomic_write, Progress, Storage},
};

const CHILD: &str = "WW5_I2_GATE_CASE";
const CHILD_ROOT: &str = "WW5_I2_GATE_ROOT";
const RAW: &[u8] = b"isolated synthetic original for gate tests";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let parent = std::env::var_os(CHILD_ROOT)
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = parent.join(format!(
            "ww5-i2-gate-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        // No link fixtures in this target. Each path was uniquely created here.
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("gate fixture cleanup failed: {}: {error}", self.0.display());
        }
    }
}

fn isolated(case: &str, body: impl FnOnce()) {
    if std::env::var(CHILD).as_deref() == Ok(case) {
        body();
        println!("WW5_GATE_CHILD_OK:{case}");
        return;
    }
    let temp = Temp::new();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([case, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, case)
        .env(CHILD_ROOT, &temp.0)
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
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !timed_out,
        "child {case} exceeded 15 seconds; killed and reaped\n{stdout}\n{stderr}"
    );
    assert!(
        output.status.success(),
        "child {case} failed: {}\n{stdout}\n{stderr}",
        output.status
    );
    assert!(
        stdout.contains("running 1 test") && stdout.contains("1 passed; 0 failed"),
        "not exactly one passing child: {stdout}"
    );
    assert!(
        stdout.contains(case) && stdout.contains(&format!("WW5_GATE_CHILD_OK:{case}")),
        "missing exact-case completion marker: {stdout}"
    );
}

fn tree(base: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            assert!(
                !metadata.file_type().is_symlink(),
                "no link traversal in gate fixture"
            );
            let relative = path.strip_prefix(base).unwrap().to_owned();
            if metadata.is_dir() {
                result.insert(relative, vec![]);
                visit(base, &path, result);
            } else if path.file_name().unwrap() == "app.lock" {
                // The live Storage owns an exclusive lock. Content files are
                // exact bytes; the live lock is accounted by its fixed length.
                result.insert(relative, metadata.len().to_le_bytes().to_vec());
            } else {
                result.insert(relative, fs::read(&path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(base, base, &mut result);
    result
}

fn digest(bytes: &[u8]) -> Sha256 {
    Sha256::parse(&assets::sha256(bytes).unwrap()).unwrap()
}
fn operation() -> OperationId {
    OperationId::parse("22222222-2222-4222-8222-000000000001").unwrap()
}
fn request() -> FrozenHistoryRequest {
    let input = FrozenInput {
        audio: AudioSnapshot {
            original: BlobRef {
                sha256: digest(RAW),
                bytes: RAW.len() as u64,
            },
            format: AudioFormat::Wav,
            rate: 8_000,
            channels: 1,
            frames: 1,
            origin: Origin::Imported,
            warnings: vec![],
        },
        reference: ReferenceSnapshot {
            text: "Exact synthetic reference.".into(),
            sha256: digest(b"Exact synthetic reference."),
            source: None,
        },
        goal: None,
        capture: CaptureProfile {
            device: None,
            distance: None,
            gain: None,
            environment: None,
            confirmed_at: None,
            version: digest(br#"{"device":null,"distance":null,"gain":null,"environment":null}"#),
        },
    };
    FrozenHistoryRequest::new(
        HistoryMutation::CreateTrial {
            id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
            created_at: RecordedAt {
                utc_millis: 1_700_000_000_000,
                offset_minutes: 540,
            },
            input,
            claim: PracticeClaim::Reference,
            duplicate: None,
        },
        vec![Arc::from(RAW)],
    )
    .unwrap()
}

#[derive(Default)]
struct Head(RefCell<Option<RootRef>>);
impl CommittedHistoryHead for Head {
    fn read_current(&self, _: &Storage) -> Result<Option<RootRef>, HistoryError> {
        Ok(self.0.borrow().clone())
    }
}
fn ready(outcome: Result<PrepareOutcome, HistoryError>) -> PreparedHistory {
    match outcome.unwrap() {
        PrepareOutcome::Ready(value) => value,
        other => panic!("expected Ready, got {other:?}"),
    }
}
fn published(outcome: Result<PublishOutcome, HistoryError>) -> PublishedHistory {
    match outcome.unwrap() {
        PublishOutcome::Published(value) => value,
        other => panic!("expected Published, got {other:?}"),
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Save,
    Load,
    MaterialSave,
    MaterialRecover,
    Prepare,
    Publish,
    Resume,
    Finish,
}
#[derive(Debug, PartialEq, Eq)]
enum Failure {
    Text,
    History(HistoryErrorKind),
}
type Outcome = Result<(), Failure>;
fn text(result: Result<impl Sized, String>) -> Outcome {
    result.map(|_| ()).map_err(|_| Failure::Text)
}
fn history(result: Result<impl Sized, HistoryError>) -> Outcome {
    result.map(|_| ()).map_err(|e| Failure::History(e.kind))
}

fn with_operation(kind: Operation, check: impl FnOnce(&mut dyn FnMut() -> Outcome, &Path)) {
    let temp = Temp::new();
    let host = temp.0.join("host");
    let mut storage = Storage::at(host.clone()).unwrap();
    let old = serde_json::to_string(&Progress::default()).unwrap();
    fs::write(host.join("progress.json"), &old).unwrap();
    fs::write(host.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
    fs::write(temp.0.join("neighbor"), b"untouched neighboring host").unwrap();
    let mut candidate = Progress::default();
    candidate.settings.topic = "synthetic update".into();
    if matches!(kind, Operation::Load | Operation::MaterialRecover) {
        let pending = serde_json::json!({
            "old_deck": model::BUILTIN_DECK, "old_progress": old,
            "deck": model::BUILTIN_DECK, "progress": serde_json::to_string(&candidate).unwrap()
        });
        fs::write(
            host.join("pending-material-commit.json"),
            serde_json::to_vec(&pending).unwrap(),
        )
        .unwrap();
    }
    match kind {
        Operation::Save => check(&mut || text(storage.save(&candidate)), &temp.0),
        Operation::Load => check(&mut || text(storage.load()), &temp.0),
        Operation::MaterialSave => check(
            &mut || text(commit::save(&host, model::BUILTIN_DECK, &candidate)),
            &temp.0,
        ),
        Operation::MaterialRecover => check(&mut || text(commit::recover(&host)), &temp.0),
        Operation::Prepare | Operation::Publish | Operation::Resume | Operation::Finish => {
            let head = Head::default();
            let request = request();
            let mut writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            match kind {
                Operation::Prepare => check(
                    &mut || history(writer.prepare(operation(), None, &request)),
                    &temp.0,
                ),
                Operation::Publish => {
                    let prepared = ready(writer.prepare(operation(), None, &request));
                    check(&mut || history(writer.publish(&prepared)), &temp.0);
                }
                Operation::Resume => {
                    ready(writer.prepare(operation(), None, &request));
                    check(
                        &mut || history(writer.resume(&operation(), &request)),
                        &temp.0,
                    );
                }
                Operation::Finish => {
                    let prepared = ready(writer.prepare(operation(), None, &request));
                    let published = published(writer.publish(&prepared));
                    *head.0.borrow_mut() = Some(published.next.clone());
                    let committed = writer.confirm_committed(&published).unwrap();
                    check(&mut || history(writer.finish(&committed)), &temp.0);
                }
                _ => unreachable!(),
            }
        }
    }
}

struct InvokeOnce<'a> {
    action: RefCell<Option<&'a mut dyn FnMut() -> Outcome>>,
    outcome: RefCell<Option<Outcome>>,
}
impl HistoryFaults for InvokeOnce<'_> {
    fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
        if point == HistoryCheckpoint::BeforeIntent {
            if let Some(action) = self.action.borrow_mut().take() {
                *self.outcome.borrow_mut() = Some(action());
            }
        }
        Ok(())
    }
}

fn reentry(kind: Operation) {
    // Prove every inner operation can succeed on its own before testing refusal.
    with_operation(kind, |action, _| {
        assert_eq!(action(), Ok(()), "normal {kind:?} fixture")
    });
    with_operation(kind, |action, inner_root| {
        let temp = Temp::new();
        let mut outer = Storage::at(temp.0.join("outer")).unwrap();
        let before = tree(inner_root);
        let faults = InvokeOnce {
            action: RefCell::new(Some(action)),
            outcome: RefCell::new(None),
        };
        let head = Head::default();
        let request = request();
        let mut writer =
            HistoryWriter::with_faults(&mut outer, &head, HistoryQuota::v1(), &faults).unwrap();
        let prepared = ready(writer.prepare(operation(), None, &request));
        assert!(
            tree(inner_root) == before,
            "reentrant {kind:?} changed inner-root bytes"
        );
        let observed = faults
            .outcome
            .borrow_mut()
            .take()
            .expect("BeforeIntent callback ran");
        match kind {
            Operation::Prepare | Operation::Publish | Operation::Resume | Operation::Finish => {
                assert_eq!(observed, Err(Failure::History(HistoryErrorKind::Busy)))
            }
            _ => assert_eq!(observed, Err(Failure::Text)),
        }
        // The rejected inner call must not poison the outer operation or leave
        // the thread marked busy after the guard is dropped.
        published(writer.publish(&prepared));
        drop(writer);
        drop(faults);
        assert_eq!(
            action(),
            Ok(()),
            "inner operation must succeed after outer guard release"
        );
    });
}

macro_rules! reentry_test {
    ($name:ident, $operation:ident) => {
        #[test]
        fn $name() {
            isolated(stringify!($name), || reentry(Operation::$operation));
        }
    };
}
reentry_test!(reentry_storage_save, Save);
reentry_test!(reentry_storage_load, Load);
reentry_test!(reentry_material_save, MaterialSave);
reentry_test!(reentry_material_recover, MaterialRecover);
reentry_test!(reentry_history_prepare, Prepare);
reentry_test!(reentry_history_publish, Publish);
reentry_test!(reentry_history_resume, Resume);
reentry_test!(reentry_history_finish, Finish);

#[test]
fn normal_public_operations_complete_without_recursive_gate_deadlock() {
    isolated(
        "normal_public_operations_complete_without_recursive_gate_deadlock",
        || {
            for kind in [
                Operation::Save,
                Operation::Load,
                Operation::MaterialSave,
                Operation::MaterialRecover,
                Operation::Prepare,
                Operation::Publish,
                Operation::Resume,
                Operation::Finish,
            ] {
                with_operation(kind, |action, _| {
                    assert_eq!(action(), Ok(()), "normal {kind:?}")
                });
            }
        },
    );
}

#[test]
fn head_callback_reentry_rejects_save_without_changing_progress() {
    isolated(
        "head_callback_reentry_rejects_save_without_changing_progress",
        || {
            struct SavingHead(RefCell<Vec<bool>>);
            impl CommittedHistoryHead for SavingHead {
                fn read_current(&self, storage: &Storage) -> Result<Option<RootRef>, HistoryError> {
                    self.0
                        .borrow_mut()
                        .push(storage.save(&Progress::default()).is_err());
                    Ok(None)
                }
            }
            let temp = Temp::new();
            let mut storage = Storage::at(temp.0.join("host")).unwrap();
            let head = SavingHead(RefCell::new(vec![]));
            let request = request();
            let progress = storage.dir.join("progress.json");
            fs::write(&progress, b"preserve exact progress sentinel").unwrap();
            let mut writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            ready(writer.prepare(operation(), None, &request));
            let observed = head.0.borrow();
            assert!(!observed.is_empty() && observed.iter().all(|rejected| *rejected));
            assert_eq!(
                fs::read(progress).unwrap(),
                b"preserve exact progress sentinel"
            );
        },
    );
}

#[test]
fn public_pending_bool_is_fail_closed_for_missing_or_non_directory_root() {
    let temp = Temp::new();
    let missing = temp.0.join("missing");
    let file = temp.0.join("file-parent");
    fs::write(&file, b"normal file cannot be a storage directory").unwrap();
    let before = tree(&temp.0);
    assert!(
        commit::pending(&missing),
        "missing root is not a normal-parent missing leaf"
    );
    assert!(commit::pending(&file));
    assert!(commit::pending(&file.join("child")));
    assert!(
        tree(&temp.0) == before,
        "pending query changed fixture bytes"
    );
}

#[test]
fn pending_leaf_directory_blocks_public_writers_and_preserves_bytes() {
    isolated(
        "pending_leaf_directory_blocks_public_writers_and_preserves_bytes",
        || {
            let temp = Temp::new();
            let storage = Storage::at(temp.0.join("host")).unwrap();
            fs::create_dir(storage.dir.join("pending-material-commit.json")).unwrap();
            fs::write(storage.dir.join("progress.json"), b"preserved progress").unwrap();
            let before = tree(&temp.0);
            assert!(commit::pending(&storage.dir));
            assert!(storage.save(&Progress::default()).is_err());
            assert!(storage.load().is_err());
            assert!(commit::save(&storage.dir, model::BUILTIN_DECK, &Progress::default()).is_err());
            assert!(commit::recover(&storage.dir).is_err());
            assert!(
                tree(&temp.0) == before,
                "pending rejection changed fixture bytes"
            );
        },
    );
}

#[test]
fn normal_parent_missing_pending_leaf_is_false_without_mutation() {
    let temp = Temp::new();
    let before = tree(&temp.0);
    assert!(!commit::pending(&temp.0));
    assert!(
        tree(&temp.0) == before,
        "missing-pending query changed fixture bytes"
    );
}

#[test]
fn panic_inside_writer_poison_stops_metadata_and_history_prepare() {
    isolated(
        "panic_inside_writer_poison_stops_metadata_and_history_prepare",
        || {
            struct PanicBeforeIntent;
            impl HistoryFaults for PanicBeforeIntent {
                fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
                    if point == HistoryCheckpoint::BeforeIntent {
                        panic!("intentional gate poison");
                    }
                    Ok(())
                }
            }
            // Build valid operation fixtures before poisoning; no reset API is used.
            with_operation(Operation::Save, |action, root| {
                let temp = Temp::new();
                let mut storage = Storage::at(temp.0.join("host")).unwrap();
                let head = Head::default();
                let request = request();
                let mut writer = HistoryWriter::with_faults(
                    &mut storage,
                    &head,
                    HistoryQuota::v1(),
                    &PanicBeforeIntent,
                )
                .unwrap();
                let before = tree(root);
                let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    writer.prepare(operation(), None, &request)
                }));
                assert!(panic.is_err(), "panic must occur inside the callback");
                assert_eq!(
                    action(),
                    Err(Failure::Text),
                    "poison must stop Storage::save"
                );
                assert!(tree(root) == before, "poisoned save changed fixture bytes");
                drop(writer);
                let no_faults = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1());
                let mut writer = no_faults.unwrap();
                assert!(matches!(
                    writer.prepare(operation(), None, &request),
                    Err(HistoryError {
                        kind: HistoryErrorKind::Io
                    })
                ));
                let progress_path = temp.0.join("host/progress.json");
                let before = tree(&temp.0);
                drop(writer);
                assert!(storage.load().is_err());
                assert!(commit::recover(&storage.dir).is_err());
                assert!(
                    commit::save(&storage.dir, model::BUILTIN_DECK, &Progress::default()).is_err()
                );
                assert!(!progress_path.exists());
                assert!(
                    tree(&temp.0) == before,
                    "poisoned metadata operation changed fixture bytes"
                );
            });
        },
    );
}

#[test]
fn unrelated_roots_have_nonoverlapping_writer_intervals_and_both_finish() {
    isolated(
        "unrelated_roots_have_nonoverlapping_writer_intervals_and_both_finish",
        || {
            struct Hold {
                id: usize,
                entered: mpsc::Sender<usize>,
                release: RefCell<mpsc::Receiver<()>>,
                active: Arc<AtomicUsize>,
                overlaps: Arc<AtomicUsize>,
            }
            impl HistoryFaults for Hold {
                fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
                    if point == HistoryCheckpoint::BeforeIntent {
                        if self.active.fetch_add(1, Ordering::SeqCst) != 0 {
                            self.overlaps.fetch_add(1, Ordering::SeqCst);
                        }
                        self.entered.send(self.id).unwrap();
                        self.release
                            .borrow()
                            .recv_timeout(Duration::from_secs(3))
                            .expect("bounded callback release");
                        self.active.fetch_sub(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            }
            let active = Arc::new(AtomicUsize::new(0));
            let overlaps = Arc::new(AtomicUsize::new(0));
            let (entered_tx, entered_rx) = mpsc::channel();
            let (started_tx, started_rx) = mpsc::channel();
            let mut releases = Vec::new();
            let mut handles = Vec::new();
            for id in 0..2 {
                let (release_tx, release_rx) = mpsc::channel();
                releases.push(release_tx);
                let entered = entered_tx.clone();
                let started = started_tx.clone();
                let active = Arc::clone(&active);
                let overlaps = Arc::clone(&overlaps);
                handles.push(thread::spawn(move || {
                    let temp = Temp::new();
                    let mut storage = Storage::at(temp.0.join(format!("host-{id}"))).unwrap();
                    let head = Head::default();
                    let request = request();
                    let faults = Hold {
                        id,
                        entered,
                        release: RefCell::new(release_rx),
                        active,
                        overlaps,
                    };
                    let mut writer = HistoryWriter::with_faults(
                        &mut storage,
                        &head,
                        HistoryQuota::v1(),
                        &faults,
                    )
                    .unwrap();
                    started.send(id).unwrap();
                    let prepared = ready(writer.prepare(operation(), None, &request));
                    let publication = published(writer.publish(&prepared));
                    drop(writer);
                    let closure = HistoryReader::new(&storage)
                        .verify_closure(&publication.next)
                        .unwrap();
                    assert_eq!(closure.objects.len(), 3);
                    assert_eq!(
                        HistoryReader::new(&storage)
                            .read_original(&BlobRef {
                                sha256: digest(RAW),
                                bytes: RAW.len() as u64
                            })
                            .unwrap(),
                        RAW
                    );
                    publication.next
                }));
            }
            started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let first = entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            // This scheduling window alone is not the oracle: both positive
            // admissions, max overlap, success, and independent bytes are checked.
            let second_early = entered_rx.recv_timeout(Duration::from_millis(150)).ok();
            releases[first].send(()).unwrap();
            let second = second_early
                .unwrap_or_else(|| entered_rx.recv_timeout(Duration::from_secs(3)).unwrap());
            assert_ne!(first, second);
            releases[second].send(()).unwrap();
            let roots: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
            assert_eq!(roots.len(), 2);
            assert_eq!(active.load(Ordering::SeqCst), 0);
            assert_eq!(
                overlaps.load(Ordering::SeqCst),
                0,
                "writer intervals overlapped"
            );
            assert!(
                second_early.is_none(),
                "second callback entered while first still held its write interval"
            );
        },
    );
}

#[test]
fn callback_can_use_gate_free_reader_head_confirmation_and_atomic_write() {
    isolated(
        "callback_can_use_gate_free_reader_head_confirmation_and_atomic_write",
        || {
            let temp = Temp::new();
            let mut storage = Storage::at(temp.0.join("inner")).unwrap();
            let head = Head::default();
            let request = request();
            let mut reader_writer =
                HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            let prepared = ready(reader_writer.prepare(operation(), None, &request));
            let publication = published(reader_writer.publish(&prepared));
            *head.0.borrow_mut() = Some(publication.next.clone());
            let confirmed = reader_writer.confirm_committed(&publication).unwrap();
            drop(reader_writer);
            let before = tree(&temp.0);
            let external = Temp::new();
            let mut action = || {
                let reader = HistoryReader::new(&storage);
                assert_eq!(
                    reader
                        .verify_closure(&publication.next)
                        .unwrap()
                        .objects
                        .len(),
                    3
                );
                assert_eq!(
                    head.read_current(&storage).unwrap(),
                    Some(publication.next.clone())
                );
                let writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
                assert_eq!(writer.confirm_committed(&publication).unwrap(), confirmed);
                atomic_write(&external.0.join("synthetic-export"), b"exact bytes").unwrap();
                Ok(())
            };
            let mut outer = Storage::at(external.0.join("outer")).unwrap();
            let outer_head = Head::default();
            let faults = InvokeOnce {
                action: RefCell::new(Some(&mut action)),
                outcome: RefCell::new(None),
            };
            let mut writer =
                HistoryWriter::with_faults(&mut outer, &outer_head, HistoryQuota::v1(), &faults)
                    .unwrap();
            ready(writer.prepare(operation(), None, &request));
            assert_eq!(faults.outcome.borrow_mut().take(), Some(Ok(())));
            assert!(
                tree(&temp.0) == before,
                "gate-free reads changed history bytes"
            );
            assert_eq!(
                fs::read(external.0.join("synthetic-export")).unwrap(),
                b"exact bytes"
            );
        },
    );
}
