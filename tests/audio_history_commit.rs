//! Independent P public pointer oracle. Only synthetic hand-built history is used.
#![cfg(windows)]
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wordweave5::{
    assets,
    audio_history::*,
    model,
    store::{HistoryCommitOutcome, Progress, ProgressHistoryHead, Storage},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp {
    root: PathBuf,
}
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ww5-i2-pointer-public-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self { root }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.root) {
            eprintln!(
                "owned pointer fixture retained {}: {e}",
                self.root.display()
            );
        }
    }
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.insert(p.strip_prefix(root).unwrap().into(), vec![]);
                walk(root, &p, out);
            } else if p.file_name().unwrap() != "app.lock" {
                out.insert(p.strip_prefix(root).unwrap().into(), fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
fn crate_model_deck() -> &'static str {
    model::BUILTIN_DECK
}

const RAW_V1: &str = r#"{ "japanese_drafts" : {"lesson":"保持する下書き"}, "version" : 1, "settings":{"topic":"合成・日本語"}, "memories":{"lesson|Recall":{"due":10,"last":9,"stability":1.5,"reviews":2,"lapses":0}}, "reviews":[], "suspended":["kept-entry"], "ai_calls":{"2026-10-08":3}, "study_seconds":{"2026-10-08":45}, "chats":[{"id":"fixture-chat","title":"保存会話","memo":"保持メモ","draft":"未送信","exchanges":[]}], "material_sources":[{"entry_id":"kept-entry","conversation_id":"fixture-chat","exchange_indices":[],"at":1,"mode":"Append"}] }"#;
const V1_SHA: &str = "a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989";
type Ledger = BTreeMap<PathBuf, Vec<u8>>;
struct Fixture {
    storage: Storage,
    temp: Temp,
    plan: Plan,
    old: Option<Vec<u8>>,
}
impl Fixture {
    fn new(wire: u8, with_base: bool) -> Self {
        assert!(wire == 2 || !with_base);
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        fs::write(temp.root.join("neighbor"), b"synthetic neighbor").unwrap();
        fs::write(storage.dir.join("keep.wav"), b"synthetic host original").unwrap();
        fs::write(storage.dir.join("custom.tsv"), crate_model_deck()).unwrap();
        fs::create_dir_all(storage.dir.join("audio-history/assets")).unwrap();
        let mut base = empty_root();
        let base_ref = if with_base {
            let old = Plan::create(trial(1), op(90));
            for bytes in &old.objects {
                put_at(&storage, bytes);
            }
            base = old.root.clone();
            Some(old.root_ref())
        } else {
            None
        };
        let plan = Plan::create_on(trial(2), op(1), &base, base_ref, None);
        for bytes in &plan.objects {
            put_at(&storage, bytes);
        }
        let intent = serde_json::to_vec(&plan.intent()).unwrap();
        let recovery = storage.dir.join("audio-history/recovery");
        fs::create_dir_all(&recovery).unwrap();
        fs::write(
            recovery.join(format!("{}.json", plan.operation.as_str())),
            intent,
        )
        .unwrap();
        let old = match wire {
            0 => None,
            1 => {
                assert_eq!(RAW_V1.len(), 550);
                assert_eq!(assets::sha256(RAW_V1.as_bytes()).unwrap(), V1_SHA);
                Some(RAW_V1.as_bytes().to_vec())
            }
            2 => {
                let mut value: Value = serde_json::from_str(&raw_full_v1()).unwrap();
                value["version"] = json!(2);
                value["audio_history"] = serde_json::to_value(&plan.base).unwrap();
                Some(serde_json::to_vec(&value).unwrap())
            }
            _ => unreachable!(),
        };
        if let Some(raw) = &old {
            let p: Progress = serde_json::from_slice(raw).unwrap();
            p.validate().unwrap();
            fs::write(storage.dir.join("progress.json"), raw).unwrap();
        }
        let f = Self {
            storage,
            temp,
            plan,
            old,
        };
        f.healthy(&f.plan.root_ref());
        if let Some(base) = &f.plan.base {
            f.healthy(base);
        }
        f
    }
    fn path(&self) -> PathBuf {
        self.storage.dir.join("progress.json")
    }
    fn history(&self) -> PathBuf {
        self.storage.dir.join("audio-history")
    }
    fn intent_path(&self) -> PathBuf {
        self.history()
            .join(format!("recovery/{}.json", self.plan.operation.as_str()))
    }
    fn asset(&self, r: &BlobRef) -> PathBuf {
        self.history()
            .join(format!("assets/{}.bin", r.sha256.as_str()))
    }
    fn healthy(&self, root: &RootRef) {
        HistoryReader::new(&self.storage)
            .verify_closure(root)
            .expect("hand-built closure");
    }
    fn ledger(&self) -> Ledger {
        tree(&self.temp.root)
    }
    fn snapshot(&self) -> Progress {
        self.old
            .as_ref()
            .map(|raw| serde_json::from_slice(raw).unwrap())
            .unwrap_or_default()
    }
    fn next(&self) -> Vec<u8> {
        let mut expected = self.snapshot();
        expected.audio_history = Some(self.plan.root_ref());
        expected.validate().unwrap();
        serde_json::to_vec_pretty(&expected).unwrap()
    }
    fn expected(&self, current: RootRef) -> HistoryCommitOutcome {
        HistoryCommitOutcome::Committed(CommittedHistory {
            current,
            operation: self.plan.operation.clone(),
            receipt: self.plan.root.receipts[&self.plan.operation].clone(),
        })
    }
    fn set_head(&self, root: &RootRef) {
        let mut p = self.snapshot();
        p.audio_history = Some(root.clone());
        p.validate().unwrap();
        fs::write(self.path(), serde_json::to_vec_pretty(&p).unwrap()).unwrap();
    }
    fn committed_ledger(&self, mut before: Ledger) -> Ledger {
        before.insert(PathBuf::from("host/progress.json"), self.next());
        before
    }
    fn later(&self) -> Plan {
        let later = Plan::create_on(
            trial(3),
            op(3),
            &self.plan.root,
            Some(self.plan.root_ref()),
            None,
        );
        for raw in &later.objects {
            put_at(&self.storage, raw);
        }
        self.healthy(&later.root_ref());
        later
    }
    fn public_control(&mut self) -> Result<CommittedHistory, HistoryError> {
        HistoryWriter::new(&mut self.storage, &ProgressHistoryHead, HistoryQuota::v1())
            .unwrap()
            .confirm_committed(&self.plan.publication())
    }
}
fn put_at(storage: &Storage, raw: &[u8]) -> BlobRef {
    let r = blob(raw);
    fs::write(
        storage
            .dir
            .join(format!("audio-history/assets/{}.bin", r.sha256.as_str())),
        raw,
    )
    .unwrap();
    r
}
fn raw_full_v1() -> String {
    // Literal sentinels, not a Progress serializer: all legacy top-level fields.
    RAW_V1.replace(r#""settings":{"topic":"合成・日本語"}"#, r#""settings":{"codex_path":"fixture-codex","codex_model":"fixture-model","codex_effort":"high","examples_per_word":7,"batch_words":8,"minutes":9,"new_per_day":4,"skills":["Usage"],"topic":"合成・日本語","font_scale":1.25,"ai_daily_limit":11,"voice_id":"fixture-voice","slow_speech":true,"speech_rate":1.5,"speech_volume":0.75,"speech_repeat":true,"page_tint":{"rgb":[1,2,3],"depth":21},"input_tint":{"rgb":[4,5,6],"depth":22}}"#)
            .replace(r#""reviews":[]"#, r#""reviews":[{"key":"lesson|Recall","at":12,"date":"2026-10-08","grade":"Good","assisted":true,"method":"fixture-method","first":false,"elapsed_days":1.25,"seconds":13,"self_assessed":true}]"#)
            .replacen("{", concat!("{",
                r#""deleted_entries":["deleted-sentinel"],"chat_action":["fixture-chat",{"operation":"Append","base":"base-sentinel","entry_id":"kept-entry"}],"deck_versions":{"deck-key":"deck-revision"},"writing_logs":[{"key":"writing-key","at":14,"problem":{"japanese":"合成問題","situation":"situation","target":"target","reference":"reference"},"answer":"answer-sentinel","feedback":"feedback-sentinel","revision":true}],"material_draft":{"mode":"New","baseline":null,"candidate":{"id":"draft-id","base":"draft-base","meaning":"draft-meaning","level":"draft-level","business":"draft-business","elevated":"draft-elevated","register":"draft-register","usage":"draft-usage","context":"draft-context","example":"draft-example","translation":"draft-translation","answers":["draft-answer"],"question":"draft-question","explanation":"draft-explanation","tag":"draft-tag"},"source":{"entry_id":"draft-id","conversation_id":"fixture-chat","exchange_indices":[],"at":15,"mode":"New"},"notices":["draft-notice"]},"#), 1)
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

#[test]
fn p01_public_commit_changes_only_root_and_preserves_every_latest_field() {
    let f = Fixture::new(2, true);
    let before = f.ledger();
    let token = f.plan.publication();
    assert_eq!(
        ProgressHistoryHead.read_current(&f.storage).unwrap(),
        token.base
    );
    assert_ne!(
        token.base,
        Some(token.next.clone()),
        "publication alone is not commit"
    );
    let actual = f.storage.commit_history(&token, None).unwrap();
    assert_eq!(actual, f.expected(token.next.clone()));
    assert_eq!(fs::read(f.path()).unwrap(), f.next());
    assert_eq!(f.ledger(), f.committed_ledger(before));
    f.healthy(&token.next);
    let disk = f.storage.read_progress().unwrap();
    assert_eq!(serde_json::to_vec_pretty(&disk).unwrap(), f.next());
    let root = f.storage.dir.clone();
    let expected = f.next();
    let keep = f.temp.root.clone();
    drop(f.storage);
    let reopened = Storage::at(root).unwrap();
    assert_eq!(
        serde_json::to_vec_pretty(&reopened.read_progress().unwrap()).unwrap(),
        expected
    );
    drop(reopened);
    assert!(keep.exists());
}
#[test]
fn p03_public_retry_after_cleanup_keeps_ids_receipt_and_later_root() {
    let mut f = Fixture::new(2, true);
    let token = f.plan.publication();
    assert_eq!(
        f.storage.commit_history(&token, None).unwrap(),
        f.expected(token.next.clone())
    );
    let saved = f.public_control().unwrap();
    struct StopFinish;
    impl HistoryFaults for StopFinish {
        fn check(&self, point: HistoryCheckpoint) -> Result<(), HistoryError> {
            if point == HistoryCheckpoint::BeforeFinish {
                Err(HistoryError {
                    kind: HistoryErrorKind::Io,
                })
            } else {
                Ok(())
            }
        }
    }
    let before_finish = f.ledger();
    assert_eq!(
        HistoryWriter::with_faults(
            &mut f.storage,
            &ProgressHistoryHead,
            HistoryQuota::v1(),
            &StopFinish
        )
        .unwrap()
        .finish(&saved)
        .unwrap_err()
        .kind,
        HistoryErrorKind::Io
    );
    assert_eq!(f.ledger(), before_finish);
    let cancel = AtomicBool::new(true);
    assert_eq!(
        f.storage.commit_history(&token, Some(&cancel)).unwrap(),
        f.expected(token.next.clone())
    );
    assert_eq!(f.ledger(), before_finish);
    HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1())
        .unwrap()
        .finish(&saved)
        .unwrap();
    assert!(!f.intent_path().exists());
    let later = f.later();
    f.set_head(&later.root_ref());
    let before = f.ledger();
    let cancel = AtomicBool::new(true);
    for _ in 0..2 {
        assert_eq!(
            f.storage.commit_history(&token, Some(&cancel)).unwrap(),
            f.expected(later.root_ref())
        );
        assert_eq!(f.ledger(), before);
    }
}
#[test]
fn p06_public_cancel_preserves_published_objects_and_same_operation_retry() {
    let f = Fixture::new(2, true);
    let token = f.plan.publication();
    let before = f.ledger();
    let cancel = AtomicBool::new(true);
    assert_eq!(
        f.storage.commit_history(&token, Some(&cancel)).unwrap(),
        HistoryCommitOutcome::CancelledBeforeCommit
    );
    assert_eq!(f.ledger(), before);
    cancel.store(false, Ordering::Release);
    assert_eq!(
        f.storage.commit_history(&token, Some(&cancel)).unwrap(),
        f.expected(token.next.clone())
    );
    assert_eq!(f.ledger(), f.committed_ledger(before));
}
#[test]
fn p02_public_forged_next_and_pending_cannot_authorize_pointer() {
    for pending in [false, true] {
        let f = Fixture::new(2, true);
        let mut token = f.plan.publication();
        let kind = if pending {
            fs::write(
                f.storage.dir.join("pending-material-commit.json"),
                b"synthetic pending",
            )
            .unwrap();
            HistoryErrorKind::PendingMaterial
        } else {
            let mut root = f.plan.root.clone();
            root.trials.remove(&tid(1));
            root.receipts.remove(&op(90));
            token.next = RootRef::new(put_at(&f.storage, &serde_json::to_vec(&root).unwrap()));
            f.healthy(&token.next);
            HistoryErrorKind::Conflict
        };
        let before = f.ledger();
        assert_eq!(
            f.storage.commit_history(&token, None).unwrap_err().kind,
            kind
        );
        assert_eq!(f.ledger(), before);
    }
}
