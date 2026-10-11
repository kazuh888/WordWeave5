//! N01 public ownership oracle. Handwritten documents, no history-writer output.
#![cfg(windows)]
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wordweave5::{
    audio_history::{BlobRef, HistoryReader, RootRef, Sha256},
    store::{Progress, Storage},
};
const ROOT_A: &[u8] =
    br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
const SHA_A: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
const SHA_B: &str = "7852cc9a55f3f85fdfba7b2edb9ecdf814cf35787df7fe13fd6ac2b85b73af97";
const FULL: &str = r#"{"version":2,"audio_history":null,"settings":{"topic":"candidate-topic","minutes":7,"font_scale":1.25},"memories":{"candidate-key":{"due":21,"last":20,"stability":2.5,"reviews":3,"lapses":1}},"reviews":[{"key":"candidate-key","at":22,"date":"2026-10-08","grade":"Good","assisted":true,"method":"candidate-method","first":false,"elapsed_days":1.5,"seconds":23,"self_assessed":true}],"suspended":["candidate-suspended"],"deleted_entries":["candidate-deleted"],"chat_action":["candidate-chat",{"operation":"Append","base":"candidate-base","entry_id":"candidate-entry"}],"ai_calls":{"2026-10-08":24},"study_seconds":{"2026-10-08":25},"deck_versions":{"candidate-deck":"candidate-version"},"writing_logs":[{"key":"candidate-writing","at":26,"problem":{"japanese":"合成","situation":"candidate-situation","target":"candidate-target","reference":"candidate-reference"},"answer":"candidate-answer","feedback":"candidate-feedback","revision":true}],"japanese_drafts":{"candidate-draft":"保持する"},"chats":[{"id":"candidate-chat","title":"candidate-title","memo":"candidate-memo","draft":"candidate-unsent","exchanges":[]}],"material_draft":{"mode":"New","baseline":null,"candidate":{"id":"candidate-entry","base":"base","meaning":"meaning","level":"level","business":"business","elevated":"elevated","register":"register","usage":"usage","context":"context","example":"example","translation":"translation","answers":["answer"],"question":"question","explanation":"explanation","tag":"tag"},"source":{"entry_id":"candidate-entry","conversation_id":"candidate-chat","exchange_indices":[],"at":27,"mode":"New"},"notices":["candidate-notice"]},"material_sources":[{"entry_id":"candidate-entry","conversation_id":"candidate-chat","exchange_indices":[],"at":28,"mode":"Append"}]}"#;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ww5-i2-normal-save-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.0) {
            eprintln!(
                "owned normal-save fixture retained: {}: {e}",
                self.0.display()
            );
        }
    }
}
fn root(hash: &str, bytes: u64) -> RootRef {
    RootRef::new(BlobRef {
        sha256: Sha256::parse(hash).unwrap(),
        bytes,
    })
}
fn tree(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.file_name().unwrap() == "app.lock" {
                continue;
            }
            if p.is_dir() {
                out.insert(p.strip_prefix(base).unwrap().to_owned(), vec![]);
                walk(base, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(base).unwrap().to_owned(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}
fn seed(storage: &Storage) {
    let dir = storage.dir.join("audio-history/assets");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{SHA_A}.bin")), ROOT_A).unwrap();
    let raw = format!(
        r#"{{"version":2,"audio_history":{{"sha256":"{SHA_A}","bytes":82}},"settings":{{"topic":"latest-disk"}},"memories":{{}},"reviews":[],"suspended":[],"ai_calls":{{}},"study_seconds":{{}}}}"#
    );
    fs::write(storage.dir.join("progress.json"), raw).unwrap();
    let verified = HistoryReader::new(storage)
        .verify_closure(&root(SHA_A, 82))
        .expect("handwritten root is valid");
    assert_eq!(verified.objects, vec![root(SHA_A, 82).as_blob().clone()]);
    assert_eq!(verified.total_bytes, 82);
}
fn candidate(history: Option<RootRef>) -> Progress {
    let mut p: Progress = serde_json::from_str(FULL).unwrap();
    p.validate()
        .expect("all-field candidate fixture is valid before ignored history replacement");
    p.audio_history = history;
    p
}

#[test]
fn n01_save_progress_inherits_latest_a_and_preserves_every_candidate_field_and_input() {
    for stale in [None, Some(root(SHA_B, 82)), Some(root(SHA_B, 0))] {
        let temp = Temp::new();
        let storage = Storage::at(temp.0.join("host")).unwrap();
        seed(&storage);
        let objects = tree(&storage.dir.join("audio-history"));
        let input = candidate(stale);
        let untouched = serde_json::to_vec(&input).unwrap();
        let mut expected = input.clone();
        expected.audio_history = Some(root(SHA_A, 82));
        let saved = storage.save_progress(&input).unwrap();
        assert_eq!(
            serde_json::to_value(&saved).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        let disk: Progress =
            serde_json::from_slice(&fs::read(storage.dir.join("progress.json")).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(disk).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(serde_json::to_vec(&input).unwrap(), untouched);
        assert_eq!(tree(&storage.dir.join("audio-history")), objects);
    }
}
#[test]
fn n01_legacy_save_is_the_same_root_preserving_wrapper() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    seed(&storage);
    let input = candidate(None);
    let untouched = serde_json::to_vec(&input).unwrap();
    storage.save(&input).unwrap();
    let disk = storage.read_progress().unwrap();
    let mut expected = input.clone();
    expected.audio_history = Some(root(SHA_A, 82));
    assert_eq!(
        serde_json::to_value(disk).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert_eq!(serde_json::to_vec(&input).unwrap(), untouched);
}
#[test]
fn n01_new_file_ignores_candidate_root_and_returns_none_without_importing_objects() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let input = candidate(Some(root(SHA_B, 0)));
    let untouched = serde_json::to_vec(&input).unwrap();
    let mut expected = input.clone();
    expected.audio_history = None;
    let saved = storage.save_progress(&input).unwrap();
    assert_eq!(
        serde_json::to_value(saved).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert_eq!(
        serde_json::to_value(storage.read_progress().unwrap()).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert_eq!(serde_json::to_vec(&input).unwrap(), untouched);
    assert!(!storage.dir.join("audio-history").exists());
    assert!(!storage.dir.join("backups").exists());
}
