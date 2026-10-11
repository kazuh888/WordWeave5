//! Independent S public boundary tests; synthetic local data only.
#![cfg(windows)]
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wordweave5::{
    audio_history::{CommittedHistoryHead, RootRef},
    store::{Progress, ProgressHistoryHead, Storage},
};

const V1: &str = r#"{"version":1,"settings":{},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
const ROOT: &str =
    r#"{"sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","bytes":17}"#;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ww5-i2-progress-{}-{}-{}",
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
                "owned progress fixture retained: {}: {error}",
                self.0.display()
            );
        }
    }
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() == "app.lock" {
                continue;
            }
            if path.is_dir() {
                out.insert(path.strip_prefix(root).unwrap().to_owned(), vec![]);
                walk(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn s01_default_is_valid_v2_with_required_serialized_null_and_roundtrip() {
    let p = Progress::default();
    assert_eq!(p.version, 2);
    p.validate().unwrap();
    assert!(p.audio_history.is_none());
    let raw = serde_json::to_vec(&p).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(value["version"], 2);
    assert_eq!(value.get("audio_history"), Some(&serde_json::Value::Null));
    let read: Progress = serde_json::from_slice(&raw).unwrap();
    read.validate().unwrap();
    assert_eq!(serde_json::to_vec(&read).unwrap(), raw);
    assert_eq!(
        read.settings.font_scale.to_bits(),
        p.settings.font_scale.to_bits()
    );
    let mut wrong = p;
    wrong.version = 1;
    assert!(
        wrong.validate().is_err(),
        "runtime v1 is not an accepted write candidate"
    );
}
#[test]
fn s02_public_deserialize_normalizes_v1_and_retains_v2_root_without_history_files() {
    let p: Progress = serde_json::from_str(V1).unwrap();
    assert_eq!(p.version, 2);
    assert!(p.audio_history.is_none());
    let raw = V1.replace(
        "\"version\":1",
        &format!("\"version\":2,\"audio_history\":{ROOT}"),
    );
    let p: Progress = serde_json::from_str(&raw).unwrap();
    p.validate().unwrap();
    assert_eq!(
        p.audio_history,
        Some(serde_json::from_str::<RootRef>(ROOT).unwrap())
    );
    assert_eq!(
        serde_json::to_value(p).unwrap()["audio_history"],
        serde_json::from_str::<serde_json::Value>(ROOT).unwrap()
    );
}
#[test]
fn s03_pure_read_and_head_never_recover_pending_or_require_closure() {
    for history in ["null", ROOT] {
        let temp = Temp::new();
        let storage = Storage::at(temp.0.join("host")).unwrap();
        let raw = V1.replace(
            "\"version\":1",
            &format!("\"version\":2,\"audio_history\":{history}"),
        );
        fs::write(storage.dir.join("progress.json"), &raw).unwrap();
        fs::write(
            storage.dir.join("pending-material-commit.json"),
            b"intentionally unreadable pending",
        )
        .unwrap();
        fs::create_dir(storage.dir.join("assets")).unwrap();
        fs::write(
            storage.dir.join("assets/keep.bin"),
            b"synthetic media sentinel",
        )
        .unwrap();
        let before = tree(&storage.dir);
        let p = storage.read_progress().unwrap();
        assert_eq!(p.version, 2);
        let expected: Option<RootRef> = serde_json::from_str(history).unwrap();
        assert_eq!(p.audio_history, expected);
        assert_eq!(
            ProgressHistoryHead.read_current(&storage).unwrap(),
            expected
        );
        assert!(tree(&storage.dir) == before);
        assert!(!storage.dir.join("audio-history").exists());
    }
}
#[test]
fn s03_verified_missing_leaf_returns_default_and_none_without_creating_files() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    fs::write(
        storage.dir.join("pending-material-commit.json"),
        b"leave pending untouched",
    )
    .unwrap();
    let before = tree(&storage.dir);
    let p = storage.read_progress().unwrap();
    assert_eq!(p.version, 2);
    assert!(p.audio_history.is_none());
    assert_eq!(ProgressHistoryHead.read_current(&storage).unwrap(), None);
    assert!(tree(&storage.dir) == before);
    assert!(!storage.dir.join("progress.json").exists());
}
