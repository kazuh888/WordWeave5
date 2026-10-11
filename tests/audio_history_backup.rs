//! AH-AC16 / I2-T06: independent legacy backup consumer and preservation oracles.
#![cfg(windows)]
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use wordweave5::{
    assets::{self, AssetKind, AssetStore},
    audio_history::{BlobRef, HistoryReader, RootRef, Sha256},
    backup, model,
    store::{Progress, Storage},
};

const ROOT_A: &[u8] =
    br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
const SHA_A: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
const RAW_V1: &str = r#"{"version":1,"settings":{"topic":"教材と独立した設定","font_scale":1.25},"memories":{"lesson|Recall":{"due":10,"last":9,"stability":1.5,"reviews":2,"lapses":0}},"reviews":[{"key":"lesson|Recall","at":12,"date":"2026-10-08","grade":"Good","assisted":true,"method":"fixture","first":false,"elapsed_days":1.25,"seconds":13,"self_assessed":true}],"suspended":["kept-entry"],"deleted_entries":["deleted-entry"],"chat_action":null,"ai_calls":{"2026-10-08":2,"2026-10-09":8},"study_seconds":{"2026-10-08":45},"deck_versions":{"deck":"revision"},"writing_logs":[{"key":"writing","at":14,"problem":{"japanese":"合成問題","situation":"situation","target":"target","reference":"reference"},"answer":"answer","feedback":"feedback","revision":true}],"japanese_drafts":{"lesson":"保持する日本語"},"chats":[{"id":"backup-chat","title":"保存会話","memo":"保持メモ","draft":"backupの下書き","exchanges":[]}],"material_draft":null,"material_sources":[{"entry_id":"kept-entry","conversation_id":"backup-chat","exchange_indices":[],"at":1,"mode":"Append"}]}"#;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ww5-i2b-backup-{}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        eprintln!("I2b-B fixture-created: {}", path.display());
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.0) {
            Ok(()) => eprintln!("I2b-B fixture-removed: {}", self.0.display()),
            Err(e) => eprintln!("I2b-B fixture-retained: {}: {e}", self.0.display()),
        }
    }
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            let relative = p.strip_prefix(root).unwrap().to_owned();
            if p.is_dir() {
                out.insert(relative, vec![]);
                walk(root, &p, out);
            } else if p.file_name().unwrap() != "app.lock" {
                out.insert(relative, fs::read(p).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, root, &mut result);
    result
}
fn value(progress: &Progress) -> Value {
    serde_json::to_value(progress).unwrap()
}
fn candidate() -> Progress {
    let p: Progress = serde_json::from_str(RAW_V1).expect("handwritten legacy fixture");
    p.validate().unwrap();
    p
}
fn root_ref(hash: &str, bytes: u64) -> RootRef {
    RootRef::new(BlobRef {
        sha256: Sha256::parse(hash).unwrap(),
        bytes,
    })
}
fn write_progress(storage: &Storage, progress: &Progress) {
    progress.validate().unwrap();
    fs::write(
        storage.dir.join("progress.json"),
        serde_json::to_vec_pretty(progress).unwrap(),
    )
    .unwrap();
}
fn install_a(storage: &Storage) -> RootRef {
    assert_eq!(ROOT_A.len(), 82);
    assert_eq!(assets::sha256(ROOT_A).unwrap(), SHA_A);
    fs::create_dir_all(storage.dir.join("audio-history/assets")).unwrap();
    fs::write(
        storage
            .dir
            .join(format!("audio-history/assets/{SHA_A}.bin")),
        ROOT_A,
    )
    .unwrap();
    let r = root_ref(SHA_A, 82);
    HistoryReader::new(storage)
        .verify_closure(&r)
        .expect("independent closure must be healthy");
    r
}
fn stale(with_b: bool) -> Progress {
    let mut p = candidate();
    p.chats[0].draft = "current未保存の日本語\n\\引用\"".into();
    p.japanese_drafts
        .insert("lesson".into(), "current日本語".into());
    p.settings.topic = "current設定".into();
    p.ai_calls = BTreeMap::from([
        ("2026-10-08".into(), 7),
        ("2026-10-09".into(), 1),
        ("2026-10-10".into(), 3),
    ]);
    p.audio_history = with_b.then(|| root_ref(&"b".repeat(64), 82));
    p.validate().unwrap();
    p
}
fn memory_snapshots(storage: &Storage) -> Vec<Progress> {
    fs::read_dir(storage.dir.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("progress-memory-before-restore-")
        })
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect()
}
fn legacy_bundle(path: &Path, raw: &[u8], version: u32) {
    fs::create_dir(path).unwrap();
    fs::write(path.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
    fs::write(path.join("progress.json"), raw).unwrap();
    let manifest = json!({"version":version,"deck_sha256":assets::sha256(model::BUILTIN_DECK.as_bytes()).unwrap(),"progress_sha256":assets::sha256(raw).unwrap(),"assets":[]});
    fs::write(
        path.join("learning-backup.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let reread: Value =
        serde_json::from_slice(&fs::read(path.join("learning-backup.json")).unwrap()).unwrap();
    assert_eq!(
        reread["progress_sha256"],
        assets::sha256(&fs::read(path.join("progress.json")).unwrap()).unwrap(),
        "schema test must have valid hash"
    );
}

fn preserve_latest(with_b: bool) {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = Some(a.clone());
    write_progress(&storage, &disk);
    let current = stale(with_b);
    let caller = value(&current);
    let before = tree(&temp.0);
    backup::preserve_current_progress(&storage, &current).unwrap();
    assert_eq!(value(&current), caller);
    let mut expected = current.clone();
    expected.audio_history = Some(a);
    let saved = memory_snapshots(&storage);
    assert_eq!(saved.len(), 1);
    assert_eq!(
        value(&saved[0]),
        value(&expected),
        "memory snapshot must use latest disk root and all current nonhistory fields"
    );
    let after = tree(&temp.0);
    for (p, bytes) in before {
        assert_eq!(after.get(&p), Some(&bytes), "{}", p.display());
    }
}
#[test]
fn b01_memory_stale_none_uses_disk_a_and_preserves_current_draft() {
    preserve_latest(false);
}
#[test]
fn b01_memory_stale_b_uses_disk_a_and_preserves_current_draft() {
    preserve_latest(true);
}

fn restore_latest(with_b: bool) {
    let temp = Temp::new();
    let bundle = temp.0.join("legacy");
    legacy_bundle(&bundle, RAW_V1.as_bytes(), 1);
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = Some(a.clone());
    disk.settings.topic = "diskのみの設定".into();
    disk.ai_calls.insert("2026-10-08".into(), 99);
    write_progress(&storage, &disk);
    fs::write(storage.dir.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
    let history_before = tree(&storage.dir.join("audio-history"));
    let source_before = tree(&bundle);
    let current = stale(with_b);
    let current_before = value(&current);
    let (deck, restored) = backup::prepare(bundle.clone())
        .unwrap()
        .restore(&storage, &current)
        .unwrap();
    let mut expected = candidate();
    expected.audio_history = Some(a.clone());
    expected.ai_calls = BTreeMap::from([
        ("2026-10-08".into(), 7),
        ("2026-10-09".into(), 8),
        ("2026-10-10".into(), 3),
    ]);
    assert_eq!(
        value(&storage.read_progress().unwrap()),
        value(&expected),
        "actual committed metadata"
    );
    assert_eq!(
        model::deck_text(&deck),
        model::deck_text(&model::parse_deck(model::BUILTIN_DECK).unwrap())
    );
    assert_eq!(value(&current), current_before);
    assert_eq!(tree(&bundle), source_before);
    assert_eq!(tree(&storage.dir.join("audio-history")), history_before);
    let mut expected_memory = current.clone();
    expected_memory.audio_history = Some(a);
    let memories = memory_snapshots(&storage);
    assert_eq!(memories.len(), 1);
    // Compare both results before asserting so a stale memory root does not hide a stale return.
    let memory_matches = value(&memories[0]) == value(&expected_memory);
    let returned_matches = value(&restored) == value(&expected);
    assert!(
        memory_matches && returned_matches,
        "memory latest-root={memory_matches}, returned actual commit={returned_matches}"
    );
}
#[test]
fn b01_restore_stale_none_returns_disk_a_backup_draft_and_daily_maxima() {
    restore_latest(false);
}
#[test]
fn b01_restore_stale_b_returns_disk_a_backup_draft_and_daily_maxima() {
    restore_latest(true);
}

#[test]
fn b01_missing_progress_leaf_does_not_resurrect_current_history() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let current = stale(true);
    let mut expected = current.clone();
    expected.audio_history = None;
    backup::preserve_current_progress(&storage, &current).unwrap();
    assert!(!storage.dir.join("progress.json").exists());
    assert_eq!(value(&memory_snapshots(&storage)[0]), value(&expected));
}

fn export_refusal(disk_some: bool, candidate_some: bool) {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = disk_some.then_some(a);
    write_progress(&storage, &disk);
    let mut p = candidate();
    p.audio_history = candidate_some.then(|| root_ref(&"b".repeat(64), 82));
    let before = tree(&temp.0);
    let caller = value(&p);
    let dest = temp.0.join("export");
    let result = backup::export(
        &storage.dir,
        &dest,
        &model::parse_deck(model::BUILTIN_DECK).unwrap(),
        &p,
    );
    assert!(
        result.is_err(),
        "v1 cannot represent either source of history"
    );
    assert!(!dest.exists());
    assert_eq!(tree(&temp.0), before);
    assert_eq!(value(&p), caller);
}
#[test]
fn b02_export_candidate_some_refuses_before_destination() {
    export_refusal(false, true);
}
#[test]
fn b02_export_disk_some_refuses_before_destination() {
    export_refusal(true, false);
}
#[test]
fn b02_export_both_some_refuses_before_destination() {
    export_refusal(true, true);
}

#[test]
fn b02_export_no_history_is_explicit_wire1_without_root_and_roundtrips_all_fields() {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("source")).unwrap();
    let p = candidate();
    write_progress(&storage, &p);
    let source_before = tree(&storage.dir);
    let deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
    let dest = temp.0.join("export");
    backup::export(&storage.dir, &dest, &deck, &p).unwrap();
    let raw = fs::read(dest.join("progress.json")).unwrap();
    let wire: Value = serde_json::from_slice(&raw).unwrap();
    let mut expected = value(&p);
    expected["version"] = json!(1);
    expected.as_object_mut().unwrap().remove("audio_history");
    assert_eq!(
        wire, expected,
        "independent wire1 projection retains every nonhistory field"
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(dest.join("learning-backup.json")).unwrap()).unwrap();
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["progress_sha256"], assets::sha256(&raw).unwrap());
    assert_eq!(
        manifest["deck_sha256"],
        assets::sha256(&fs::read(dest.join("custom.tsv")).unwrap()).unwrap()
    );
    let target = Storage::at(temp.0.join("restored")).unwrap();
    let (_, restored) = backup::prepare(dest)
        .unwrap()
        .restore(&target, &Progress::default())
        .unwrap();
    assert_eq!(value(&restored), value(&p));
    assert_eq!(value(&target.read_progress().unwrap()), value(&p));
    assert_eq!(tree(&storage.dir), source_before);
}

fn prepare_bad(case: &str) {
    let temp = Temp::new();
    let path = temp.0.join("backup");
    let root = format!(r#"{{"sha256":"{SHA_A}","bytes":82}}"#);
    let v2 = RAW_V1.replace("\"version\":1", "\"version\":2,\"audio_history\":null");
    let raw = match case {
        "root-v1" => RAW_V1.replacen('{', &format!("{{\"audio_history\":{root},"), 1),
        "root-v2" => v2.replace(
            "\"audio_history\":null",
            &format!("\"audio_history\":{root}"),
        ),
        "version" => v2.replace("\"version\":2", "\"version\":99"),
        "duplicate-version" => RAW_V1.replacen('{', "{\"version\":1,", 1),
        "duplicate-root" => v2.replacen('{', "{\"audio_history\":null,", 1),
        "field" => RAW_V1.replacen('{', "{\"unrecognized\":1,", 1),
        "json" => "{".into(),
        "manifest" => RAW_V1.into(),
        _ => unreachable!(),
    };
    legacy_bundle(
        &path,
        raw.as_bytes(),
        if case == "manifest" { 99 } else { 1 },
    );
    let before = tree(&temp.0);
    assert!(
        backup::prepare(path).is_err(),
        "schema refusal for {case}, with correct hashes"
    );
    assert_eq!(tree(&temp.0), before);
}
#[test]
fn b03_prepare_rejects_wire1_root_with_correct_hash() {
    prepare_bad("root-v1");
}
#[test]
fn b03_prepare_rejects_wire2_root_with_correct_hash() {
    prepare_bad("root-v2");
}
#[test]
fn b03_prepare_rejects_unknown_progress_version_with_correct_hash() {
    prepare_bad("version");
}
#[test]
fn b03_prepare_rejects_duplicate_version_with_correct_hash() {
    prepare_bad("duplicate-version");
}
#[test]
fn b03_prepare_rejects_duplicate_root_with_correct_hash() {
    prepare_bad("duplicate-root");
}
#[test]
fn b03_prepare_rejects_unknown_field_with_correct_hash() {
    prepare_bad("field");
}
#[test]
fn b03_prepare_rejects_broken_json_with_correct_hash() {
    prepare_bad("json");
}
#[test]
fn b03_prepare_rejects_unknown_manifest_with_correct_hash() {
    prepare_bad("manifest");
}
#[test]
fn b03_prepare_accepts_existing_wire2_null_compatibility_without_mutation() {
    let temp = Temp::new();
    let path = temp.0.join("backup");
    let raw = RAW_V1.replace("\"version\":1", "\"version\":2,\"audio_history\":null");
    legacy_bundle(&path, raw.as_bytes(), 1);
    let before = tree(&temp.0);
    assert_eq!(
        value(&backup::prepare(path).unwrap().progress),
        value(&candidate())
    );
    assert_eq!(tree(&temp.0), before);
}

fn preservation_refusal(case: &str) {
    let temp = Temp::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = Some(a);
    write_progress(&storage, &disk);
    match case {
        "pending" => fs::write(
            storage.dir.join("pending-material-commit.json"),
            b"must never recover",
        )
        .unwrap(),
        "pending-dir" => fs::create_dir(storage.dir.join("pending-material-commit.json")).unwrap(),
        "unknown-progress" => fs::write(
            storage.dir.join("progress.json"),
            RAW_V1.replace("\"version\":1", "\"version\":99"),
        )
        .unwrap(),
        "json" => fs::write(storage.dir.join("progress.json"), b"{").unwrap(),
        "missing-root" => fs::remove_file(
            storage
                .dir
                .join(format!("audio-history/assets/{SHA_A}.bin")),
        )
        .unwrap(),
        "unknown-root" => {
            let raw=br#"{"schema_version":999,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
            let hash = assets::sha256(raw).unwrap();
            fs::write(
                storage.dir.join(format!("audio-history/assets/{hash}.bin")),
                raw,
            )
            .unwrap();
            disk.audio_history = Some(root_ref(&hash, raw.len() as u64));
            write_progress(&storage, &disk);
        }
        "backup-file" => fs::write(storage.dir.join("backups"), b"preserve existing file").unwrap(),
        _ => unreachable!(),
    }
    let current = stale(false);
    let before = tree(&temp.0);
    let caller = value(&current);
    for _ in 0..2 {
        assert!(
            backup::preserve_current_progress(&storage, &current).is_err(),
            "{case}"
        );
        assert_eq!(tree(&temp.0), before);
        assert_eq!(value(&current), caller);
    }
}
#[test]
fn b01_pending_refuses_preservation_without_recovery_or_retry_writes() {
    preservation_refusal("pending");
}
#[test]
fn b01_invalid_pending_path_refuses_preservation() {
    preservation_refusal("pending-dir");
}
#[test]
fn b01_unknown_disk_progress_refuses_preservation() {
    preservation_refusal("unknown-progress");
}
#[test]
fn b01_broken_disk_progress_refuses_preservation() {
    preservation_refusal("json");
}
#[test]
fn b01_missing_closure_refuses_preservation() {
    preservation_refusal("missing-root");
}
#[test]
fn b01_unknown_closure_refuses_preservation() {
    preservation_refusal("unknown-root");
}
#[test]
fn b01_memory_destination_file_refuses_without_overwrite() {
    preservation_refusal("backup-file");
}

#[test]
fn b01_restore_rechecks_source_media_before_memory_or_metadata_mutation() {
    let temp = Temp::new();
    let source = temp.0.join("source");
    let bundle = temp.0.join("backup");
    let media = AssetStore::new(source.clone())
        .put(AssetKind::AudioWav, b"RIFF\x04\0\0\0WAVE")
        .unwrap();
    backup::export(
        &source,
        &bundle,
        &model::parse_deck(model::BUILTIN_DECK).unwrap(),
        &candidate(),
    )
    .unwrap();
    let prepared = backup::prepare(bundle.clone()).unwrap();
    let asset = tree(&bundle)
        .keys()
        .find(|p| p.to_string_lossy().contains(&media.id))
        .unwrap()
        .clone();
    fs::write(bundle.join(asset), b"corrupt after confirmation").unwrap();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = Some(a);
    write_progress(&storage, &disk);
    let before = tree(&temp.0);
    let current = stale(true);
    let caller = value(&current);
    assert!(prepared.restore(&storage, &current).is_err());
    assert_eq!(tree(&temp.0), before);
    assert_eq!(value(&current), caller);
}

#[test]
fn b02_export_pending_and_invalid_disk_refuse_before_destination_and_preserve_bytes() {
    for case in ["pending", "pending-dir", "json", "version"] {
        let temp = Temp::new();
        let storage = Storage::at(temp.0.join("source")).unwrap();
        write_progress(&storage, &candidate());
        match case {
            "pending" => fs::write(
                storage.dir.join("pending-material-commit.json"),
                b"must not recover",
            )
            .unwrap(),
            "pending-dir" => {
                fs::create_dir(storage.dir.join("pending-material-commit.json")).unwrap()
            }
            "json" => fs::write(storage.dir.join("progress.json"), b"{").unwrap(),
            _ => fs::write(
                storage.dir.join("progress.json"),
                RAW_V1.replace("\"version\":1", "\"version\":99"),
            )
            .unwrap(),
        }
        let before = tree(&temp.0);
        let dest = temp.0.join("export");
        assert!(
            backup::export(
                &storage.dir,
                &dest,
                &model::parse_deck(model::BUILTIN_DECK).unwrap(),
                &candidate()
            )
            .is_err(),
            "{case}"
        );
        assert_eq!(tree(&temp.0), before);
        assert!(!dest.exists());
    }
}

#[test]
fn b01_restore_commit_refusal_keeps_prior_memory_and_immutable_media_but_not_new_metadata() {
    let temp = Temp::new();
    let source = temp.0.join("source");
    let bundle = temp.0.join("backup");
    let media = AssetStore::new(source.clone())
        .put(AssetKind::AudioWav, b"RIFF\x04\0\0\0WAVE")
        .unwrap();
    backup::export(
        &source,
        &bundle,
        &model::parse_deck(model::BUILTIN_DECK).unwrap(),
        &candidate(),
    )
    .unwrap();
    let prepared = backup::prepare(bundle.clone()).unwrap();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    let a = install_a(&storage);
    let mut disk = candidate();
    disk.audio_history = Some(a.clone());
    write_progress(&storage, &disk);
    // commit rejects a directory at its old deck path, after restore's memory/media steps.
    fs::create_dir(storage.dir.join("custom.tsv")).unwrap();
    fs::write(storage.dir.join("custom.tsv/keep"), b"directory sentinel").unwrap();
    let before = tree(&storage.dir);
    let source_before = tree(&bundle);
    let current = stale(true);
    assert!(prepared.restore(&storage, &current).is_err());
    let after = tree(&storage.dir);
    for (p, bytes) in before {
        assert_eq!(after.get(&p), Some(&bytes), "{}", p.display());
    }
    assert!(!storage.dir.join("pending-material-commit.json").exists());
    assert_eq!(
        AssetStore::new(storage.dir.clone()).read(&media).unwrap(),
        b"RIFF\x04\0\0\0WAVE"
    );
    let memories = memory_snapshots(&storage);
    assert_eq!(memories.len(), 1);
    let mut expected = current;
    expected.audio_history = Some(a);
    assert_eq!(value(&memories[0]), value(&expected));
    assert_eq!(tree(&bundle), source_before);
}
