//! S-approved M05 oracle: wire1 preservation and first public wire2 saves.
#![cfg(windows)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wordweave5::{
    commit, model,
    store::{Progress, Storage},
};

// Handwritten UTF-8, intentionally noncanonical order/spacing. Never generated
// from Progress serialization. The independent .NET SHA-256 is recorded below.
const RAW_V1: &str = r#"{ "japanese_drafts" : {"lesson":"保持する下書き"}, "version" : 1, "settings":{"topic":"合成・日本語"}, "memories":{"lesson|Recall":{"due":10,"last":9,"stability":1.5,"reviews":2,"lapses":0}}, "reviews":[], "suspended":["kept-entry"], "ai_calls":{"2026-10-08":3}, "study_seconds":{"2026-10-08":45}, "chats":[{"id":"fixture-chat","title":"保存会話","memo":"保持メモ","draft":"未送信","exchanges":[]}], "material_sources":[{"entry_id":"kept-entry","conversation_id":"fixture-chat","exchange_indices":[],"at":1,"mode":"Append"}] }"#;
const HASH: &str = "a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ww5-i2-migration-{}-{}-{}",
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
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!(
                "owned migration fixture retained: {}: {error}",
                self.0.display()
            );
        }
    }
}
fn seed(root: &Path) {
    let progress: Progress = serde_json::from_str(RAW_V1).expect("handwritten v1 decodes");
    progress.validate().expect("handwritten v1 validates");
    assert_eq!(RAW_V1.len(), 550);
    fs::write(root.join("progress.json"), RAW_V1).unwrap();
    fs::write(root.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
    fs::create_dir(root.join("assets")).unwrap();
    fs::write(
        root.join("assets/recording.bin"),
        b"preserve synthetic recording bytes",
    )
    .unwrap();
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.insert(path.strip_prefix(root).unwrap().to_owned(), vec![]);
                visit(root, &path, out);
            } else if path.file_name().unwrap() != "app.lock" {
                out.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
fn no_migration_backup(root: &Path) {
    assert!(!root
        .join(format!("backups/progress-v1-{HASH}.json"))
        .exists());
    if root.join("backups").exists() {
        assert!(fs::read_dir(root.join("backups")).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("progress-v1-")));
    }
}
fn assert_kept(progress: &Progress) {
    assert_eq!(progress.version, 2);
    assert_eq!(progress.memories["lesson|Recall"].reviews, 2);
    assert_eq!(progress.japanese_drafts["lesson"], "保持する下書き");
    assert_eq!(progress.chats[0].draft, "未送信");
    assert_eq!(progress.material_sources[0].conversation_id, "fixture-chat");
    assert_eq!(progress.ai_calls["2026-10-08"], 3);
    assert_eq!(progress.study_seconds["2026-10-08"], 45);
}

#[test]
fn m05_v1_load_is_read_only_and_first_v2_save_keeps_both_exact_backups() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    seed(&storage.dir);
    let before = tree(&storage.dir);
    let mut candidate = storage.load().unwrap();
    assert_kept(&candidate);
    assert!(tree(&storage.dir) == before, "load changed exact bytes");
    candidate.settings.topic = "保存だけ変更".into();
    storage.save(&candidate).unwrap();
    let loaded = storage.load().unwrap();
    assert_kept(&loaded);
    assert_eq!(loaded.settings.topic, "保存だけ変更");
    let backups: Vec<_> = fs::read_dir(storage.dir.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 2);
    let hash_name = format!("progress-v1-{HASH}.json");
    assert_eq!(
        backups
            .iter()
            .filter(|p| p.file_name().unwrap() == hash_name.as_str())
            .count(),
        1
    );
    assert_eq!(
        backups
            .iter()
            .filter(|p| p
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("progress-")
                && p.file_name().unwrap() != hash_name.as_str())
            .count(),
        1
    );
    for path in &backups {
        assert_eq!(fs::read(path).unwrap(), RAW_V1.as_bytes());
    }
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(storage.dir.join("progress.json")).unwrap()).unwrap();
    assert_eq!(wire["version"], 2);
    assert_eq!(wire.get("audio_history"), Some(&serde_json::Value::Null));
    assert_eq!(
        fs::read(storage.dir.join("assets/recording.bin")).unwrap(),
        b"preserve synthetic recording bytes"
    );
}

#[test]
fn m05_new_v2_material_save_keeps_old_exact_intent_and_hash_backup() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    seed(&storage.dir);
    let mut candidate: Progress = serde_json::from_str(RAW_V1).unwrap();
    candidate.settings.topic = "教材と保存".into();
    commit::save(&storage.dir, model::BUILTIN_DECK, &candidate).unwrap();
    assert!(!commit::pending(&storage.dir));
    let loaded = storage.load().unwrap();
    assert_kept(&loaded);
    assert_eq!(loaded.settings.topic, "教材と保存");
    let backups: Vec<_> = fs::read_dir(storage.dir.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 2);
    let hash_path = storage.dir.join(format!("backups/progress-v1-{HASH}.json"));
    assert_eq!(fs::read(&hash_path).unwrap(), RAW_V1.as_bytes());
    let archives: Vec<_> = backups.iter().filter(|p| **p != hash_path).collect();
    assert_eq!(archives.len(), 1);
    assert!(archives[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("material-commit-"));
    let intent: serde_json::Value =
        serde_json::from_slice(&fs::read(archives[0]).unwrap()).unwrap();
    assert_eq!(
        intent["old_progress"].as_str().unwrap().as_bytes(),
        RAW_V1.as_bytes()
    );
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(storage.dir.join("progress.json")).unwrap()).unwrap();
    assert_eq!(wire["version"], 2);
    assert_eq!(wire.get("audio_history"), Some(&serde_json::Value::Null));
}

#[test]
fn m05_handwritten_v1_pending_replays_exact_bytes_from_each_partial_state() {
    for state in 0..3 {
        let temp = Temp::new();
        let storage = Storage::at(temp.0.join("host")).unwrap();
        seed(&storage.dir);
        let next = RAW_V1.replace("合成・日本語", "復旧した日本語");
        let deck = format!("{}\n", model::BUILTIN_DECK);
        let pending = serde_json::to_vec(&serde_json::json!({"old_deck":model::BUILTIN_DECK,"old_progress":RAW_V1,"deck":deck,"progress":next})).unwrap();
        fs::write(storage.dir.join("pending-material-commit.json"), &pending).unwrap();
        if state >= 1 {
            fs::write(storage.dir.join("custom.tsv"), &deck).unwrap();
        }
        if state == 2 {
            fs::write(storage.dir.join("progress.json"), &next).unwrap();
        }
        commit::recover(&storage.dir).unwrap();
        assert_eq!(
            fs::read(storage.dir.join("custom.tsv")).unwrap(),
            deck.as_bytes()
        );
        assert_eq!(
            fs::read(storage.dir.join("progress.json")).unwrap(),
            next.as_bytes()
        );
        assert!(!commit::pending(&storage.dir));
        let archives: Vec<_> = fs::read_dir(storage.dir.join("backups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(archives.len(), 1);
        assert_eq!(fs::read(&archives[0]).unwrap(), pending);
        let before = tree(&storage.dir);
        commit::recover(&storage.dir).unwrap();
        assert!(tree(&storage.dir) == before, "repeat recover changed files");
        no_migration_backup(&storage.dir);
        let runtime: Progress = serde_json::from_slice(next.as_bytes()).unwrap();
        assert_kept(&runtime);
    }
}
