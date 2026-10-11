//! Independent R public recovery oracle: input strings, never save-generated intents.
#![cfg(windows)]
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use wordweave5::{
    audio_history::*,
    commit, model,
    store::{Progress, Storage},
};
const V1: &str = r#"{ "version":1,"settings":{"topic":"old raw"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{} }"#;
const V2: &str = r#"{ "version":2,"audio_history":null,"settings":{"topic":"new raw"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{} }"#;
const A: &[u8] =
    br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
const AH: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
const BH: &str = "7852cc9a55f3f85fdfba7b2edb9ecdf814cf35787df7fe13fd6ac2b85b73af97";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ww5-i2-material-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.0) {
            eprintln!("owned material fixture retained {}: {e}", self.0.display());
        }
    }
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, p: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.insert(p.strip_prefix(root).unwrap().into(), vec![]);
                walk(root, &p, out)
            } else if p.file_name().unwrap() != "app.lock" {
                out.insert(p.strip_prefix(root).unwrap().into(), fs::read(p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
fn root_json(hash: &str) -> String {
    V2.replace(
        "\"audio_history\":null",
        &format!(r#""audio_history":{{"sha256":"{hash}","bytes":82}}"#),
    )
}
fn intent(old: Option<&str>, next: &str) -> Vec<u8> {
    let q = |s: &str| serde_json::to_string(s).unwrap();
    format!(
        "{{\n  \"old_deck\":{}, \"old_progress\":{},\n \"deck\":{}, \"progress\":{}\n}}\n",
        q("intentionally unparseable old deck"),
        old.map(q).unwrap_or_else(|| "null".into()),
        q(model::BUILTIN_DECK),
        q(next)
    )
    .into_bytes()
}
fn seed(p: &Path, old: Option<&str>, next: &str, state: u8) -> Vec<u8> {
    fs::write(
        p.join("custom.tsv"),
        if state & 1 == 0 {
            "intentionally unparseable old deck"
        } else {
            model::BUILTIN_DECK
        },
    )
    .unwrap();
    if let Some(raw) = if state & 2 == 0 { old } else { Some(next) } {
        fs::write(p.join("progress.json"), raw).unwrap();
    }
    fs::write(p.join("keep.wav"), b"synthetic original media").unwrap();
    let raw = intent(old, next);
    fs::write(p.join("pending-material-commit.json"), &raw).unwrap();
    raw
}
fn install_roots(p: &Path) {
    fs::create_dir_all(p.join("audio-history/assets")).unwrap();
    fs::write(p.join(format!("audio-history/assets/{AH}.bin")), A).unwrap();
    fs::write(
        p.join(format!("audio-history/assets/{BH}.bin")),
        String::from_utf8(A.to_vec())
            .unwrap()
            .replace("\"revision\":0", "\"revision\":1"),
    )
    .unwrap();
    let storage = Storage::at(p.to_owned()).unwrap();
    for h in [AH, BH] {
        let r = RootRef::new(BlobRef {
            sha256: Sha256::parse(h).unwrap(),
            bytes: 82,
        });
        let closure = HistoryReader::new(&storage)
            .verify_closure(&r)
            .expect("manual root fixture");
        assert_eq!(closure.objects, vec![r.as_blob().clone()]);
        assert_eq!(closure.total_bytes, 82);
    }
}
#[test]
fn r01_public_wire1_exact_replay_all_partial_states_and_idempotent_archive() {
    for absent in [false, true] {
        for state in 0..4 {
            let t = Temp::new();
            let next = V1.replace("old raw", "next raw");
            let old = if absent { None } else { Some(V1) };
            let raw = seed(&t.0, old, &next, state);
            let before = tree(&t.0);
            commit::recover(&t.0).unwrap();
            assert_eq!(
                fs::read(t.0.join("progress.json")).unwrap(),
                next.as_bytes()
            );
            let runtime: Progress = serde_json::from_str(&next).unwrap();
            assert_eq!(runtime.version, 2);
            assert_eq!(runtime.audio_history, None);
            assert_eq!(runtime.settings.topic, "next raw");
            let entries: Vec<_> = fs::read_dir(t.0.join("backups"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            assert_eq!(entries.len(), 1);
            assert!(entries[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("material-commit-"));
            assert_eq!(fs::read(&entries[0]).unwrap(), raw);
            let mut expected = before;
            expected.remove(Path::new("pending-material-commit.json"));
            expected.insert("custom.tsv".into(), model::BUILTIN_DECK.as_bytes().to_vec());
            expected.insert("progress.json".into(), next.as_bytes().to_vec());
            expected.insert("backups".into(), vec![]);
            expected.insert(entries[0].strip_prefix(&t.0).unwrap().into(), raw);
            assert_eq!(tree(&t.0), expected);
            commit::recover(&t.0).unwrap();
            assert_eq!(tree(&t.0), expected);
        }
    }
}
#[test]
fn r02_public_root_authority_and_invalid_old_reject_before_any_write() {
    let a = root_json(AH);
    let b = root_json(BH);
    let same_hash_wrong_bytes = a.replace("\"bytes\":82", "\"bytes\":83");
    let structurally_valid_old: Progress = serde_json::from_str(&same_hash_wrong_bytes).unwrap();
    structurally_valid_old.validate().unwrap();
    let old_root = structurally_valid_old.audio_history.as_ref().unwrap();
    assert_eq!(old_root.as_blob().sha256.as_str(), AH);
    assert_eq!(old_root.as_blob().bytes, 83);
    let unknown = V2.replace("\"version\":2", "\"version\":99");
    for (old, next) in [
        (Some(a.as_str()), V2),
        (Some(a.as_str()), b.as_str()),
        (Some(same_hash_wrong_bytes.as_str()), a.as_str()),
        (None, a.as_str()),
        (Some("{broken"), V2),
        (Some(unknown.as_str()), V2),
    ] {
        for state in 0..4 {
            let t = Temp::new();
            install_roots(&t.0);
            seed(&t.0, old, next, state);
            let before = tree(&t.0);
            assert!(
                commit::recover(&t.0).is_err(),
                "root authority/invalid old must fail even when disk already has new bytes"
            );
            assert_eq!(tree(&t.0), before);
        }
    }
}
#[test]
fn r04_public_already_new_still_preserves_exact_old_v1_from_pending() {
    // Reuse M's independently hashed raw UTF-8 fixture, not a product serializer.
    const OLD: &str = r#"{ "japanese_drafts" : {"lesson":"保持する下書き"}, "version" : 1, "settings":{"topic":"合成・日本語"}, "memories":{"lesson|Recall":{"due":10,"last":9,"stability":1.5,"reviews":2,"lapses":0}}, "reviews":[], "suspended":["kept-entry"], "ai_calls":{"2026-10-08":3}, "study_seconds":{"2026-10-08":45}, "chats":[{"id":"fixture-chat","title":"保存会話","memo":"保持メモ","draft":"未送信","exchanges":[]}], "material_sources":[{"entry_id":"kept-entry","conversation_id":"fixture-chat","exchange_indices":[],"at":1,"mode":"Append"}] }"#;
    let t = Temp::new();
    let raw = seed(&t.0, Some(OLD), V2, 3);
    commit::recover(&t.0).unwrap();
    assert_eq!(fs::read(t.0.join("backups/progress-v1-a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989.json")).expect("intent old bytes must be backed up even when current is new"),OLD.as_bytes());
    assert_eq!(fs::read(t.0.join("progress.json")).unwrap(), V2.as_bytes());
    let archives: Vec<_> = fs::read_dir(t.0.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("material-commit-")
        })
        .collect();
    assert_eq!(archives.len(), 1);
    assert_eq!(fs::read(&archives[0]).unwrap(), raw);
}

// T additions are separate from the fixed R public oracle above.
mod t_material_save {
    use super::*;
    const FULL: &str = r#"{"version":2,"audio_history":null,"settings":{"topic":"candidate-topic","minutes":7,"font_scale":1.25},"memories":{"candidate-key":{"due":21,"last":20,"stability":2.5,"reviews":3,"lapses":1}},"reviews":[{"key":"candidate-key","at":22,"date":"2026-10-08","grade":"Good","assisted":true,"method":"candidate-method","first":false,"elapsed_days":1.5,"seconds":23,"self_assessed":true}],"suspended":["candidate-suspended"],"deleted_entries":["candidate-deleted"],"chat_action":["candidate-chat",{"operation":"Append","base":"candidate-base","entry_id":"candidate-entry"}],"ai_calls":{"2026-10-08":24},"study_seconds":{"2026-10-08":25},"deck_versions":{"candidate-deck":"candidate-version"},"writing_logs":[{"key":"candidate-writing","at":26,"problem":{"japanese":"合成","situation":"candidate-situation","target":"candidate-target","reference":"candidate-reference"},"answer":"candidate-answer","feedback":"candidate-feedback","revision":true}],"japanese_drafts":{"candidate-draft":"保持する"},"chats":[{"id":"candidate-chat","title":"candidate-title","memo":"candidate-memo","draft":"candidate-unsent","exchanges":[]}],"material_draft":{"mode":"New","baseline":null,"candidate":{"id":"candidate-entry","base":"base","meaning":"meaning","level":"level","business":"business","elevated":"elevated","register":"register","usage":"usage","context":"context","example":"example","translation":"translation","answers":["answer"],"question":"question","explanation":"explanation","tag":"tag"},"source":{"entry_id":"candidate-entry","conversation_id":"candidate-chat","exchange_indices":[],"at":27,"mode":"New"},"notices":["candidate-notice"]},"material_sources":[{"entry_id":"candidate-entry","conversation_id":"candidate-chat","exchange_indices":[],"at":28,"mode":"Append"}]}"#;
    fn reference(hash: &str, bytes: u64) -> RootRef {
        RootRef::new(BlobRef {
            sha256: Sha256::parse(hash).unwrap(),
            bytes,
        })
    }
    fn candidate(root: Option<RootRef>) -> Progress {
        let mut p: Progress = serde_json::from_str(FULL).unwrap();
        p.validate().unwrap();
        p.audio_history = root;
        p
    }
    fn check_saved(
        root: &Path,
        mut before: BTreeMap<PathBuf, Vec<u8>>,
        old: Option<&[u8]>,
        deck: &str,
        expected: &Progress,
    ) {
        let next = serde_json::to_vec_pretty(expected).unwrap();
        assert_eq!(fs::read(root.join("progress.json")).unwrap(), next);
        assert_eq!(fs::read(root.join("custom.tsv")).unwrap(), deck.as_bytes());
        assert!(!root.join("pending-material-commit.json").exists());
        let archives: Vec<_> = fs::read_dir(root.join("backups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(
            archives.len(),
            1,
            "T adds no daily backup for old v2 or absent progress"
        );
        assert!(archives[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("material-commit-"));
        let raw = fs::read(&archives[0]).unwrap();
        let intent: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(intent.as_object().unwrap().len(), 4);
        assert_eq!(intent["old_progress"].as_str().map(str::as_bytes), old);
        assert_eq!(
            intent["old_deck"].as_str(),
            Some("unparseable old deck retained exactly")
        );
        assert_eq!(intent["deck"].as_str(), Some(deck));
        assert_eq!(intent["progress"].as_str().unwrap().as_bytes(), next);
        before.insert("custom.tsv".into(), deck.as_bytes().to_vec());
        before.insert("progress.json".into(), next);
        before.insert("backups".into(), vec![]);
        before.insert(archives[0].strip_prefix(root).unwrap().into(), raw);
        assert_eq!(tree(root), before);
    }
    #[test]
    fn t01_public_save_inherits_latest_root_preserves_all_fields_and_input() {
        for stale in [None, Some(reference(BH, 82)), Some(reference(BH, 0))] {
            let t = Temp::new();
            install_roots(&t.0);
            let raw = root_json(AH);
            fs::write(t.0.join("progress.json"), &raw).unwrap();
            fs::write(
                t.0.join("custom.tsv"),
                b"unparseable old deck retained exactly",
            )
            .unwrap();
            fs::write(t.0.join("ink.bin"), b"synthetic ink").unwrap();
            let input = candidate(stale);
            let input_bytes = serde_json::to_vec(&input).unwrap();
            let mut expected = input.clone();
            expected.audio_history = Some(reference(AH, 82));
            let before = tree(&t.0);
            let deck = format!("{}\n", model::BUILTIN_DECK);
            let saved = commit::save_progress(&t.0, &deck, &input).unwrap();
            assert_eq!(
                serde_json::to_value(saved).unwrap(),
                serde_json::to_value(&expected).unwrap()
            );
            assert_eq!(serde_json::to_vec(&input).unwrap(), input_bytes);
            check_saved(&t.0, before, Some(raw.as_bytes()), &deck, &expected);
        }
    }
    #[test]
    fn t01_legacy_save_wrapper_preserves_latest_root() {
        let t = Temp::new();
        install_roots(&t.0);
        let raw = root_json(AH);
        fs::write(t.0.join("progress.json"), &raw).unwrap();
        fs::write(
            t.0.join("custom.tsv"),
            b"unparseable old deck retained exactly",
        )
        .unwrap();
        let input = candidate(None);
        let unchanged = serde_json::to_vec(&input).unwrap();
        let mut expected = input.clone();
        expected.audio_history = Some(reference(AH, 82));
        let before = tree(&t.0);
        commit::save(&t.0, model::BUILTIN_DECK, &input).unwrap();
        assert_eq!(serde_json::to_vec(&input).unwrap(), unchanged);
        check_saved(
            &t.0,
            before,
            Some(raw.as_bytes()),
            model::BUILTIN_DECK,
            &expected,
        );
    }
    #[test]
    fn t01_missing_old_progress_cannot_import_a_candidate_root() {
        let t = Temp::new();
        fs::write(
            t.0.join("custom.tsv"),
            b"unparseable old deck retained exactly",
        )
        .unwrap();
        let input = candidate(Some(reference(BH, 0)));
        let untouched = serde_json::to_vec(&input).unwrap();
        let mut expected = input.clone();
        expected.audio_history = None;
        let before = tree(&t.0);
        let saved = commit::save_progress(&t.0, model::BUILTIN_DECK, &input).unwrap();
        assert_eq!(
            serde_json::to_value(saved).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(serde_json::to_vec(&input).unwrap(), untouched);
        assert!(!t.0.join("audio-history").exists());
        check_saved(&t.0, before, None, model::BUILTIN_DECK, &expected);
    }
}
