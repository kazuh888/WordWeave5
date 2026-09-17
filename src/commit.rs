//! Recoverable two-file commit. A pending intent blocks unrelated progress saves.
use crate::{
    model,
    store::{atomic_write, Progress},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const PENDING: &str = "pending-material-commit.json";
pub(crate) const DECK_LIMIT: u64 = 64_000_000;
pub(crate) const PROGRESS_LIMIT: u64 = 100_000_000;
#[derive(Clone, Copy)]
struct Limits {
    deck: u64,
    progress: u64,
    intent: u64,
}
const DEFAULT_LIMITS: Limits = Limits {
    deck: DECK_LIMIT,
    progress: PROGRESS_LIMIT,
    intent: 350_000_000,
};
#[derive(Serialize, Deserialize)]
struct Intent {
    old_deck: Option<String>,
    old_progress: Option<String>,
    deck: String,
    progress: String,
}
pub fn pending(root: &Path) -> bool {
    root.join(PENDING).exists()
}
fn read(path: &Path, limit: u64) -> Result<Option<String>, String> {
    use std::io::Read;
    let file = match fs::File::open(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
        Ok(file) => Ok(Some(file)),
    }?;
    let Some(file) = file else {
        return Ok(None);
    };
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err("保存ファイルが大きすぎます。".into());
    }
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > limit {
        return Err("保存ファイルが大きすぎます。".into());
    }
    Ok(Some(text))
}
pub fn save(root: &Path, deck: &str, progress: &Progress) -> Result<(), String> {
    save_with_limits(root, deck, progress, DEFAULT_LIMITS)
}
fn save_with_limits(
    root: &Path,
    deck: &str,
    progress: &Progress,
    limits: Limits,
) -> Result<(), String> {
    if pending(root) {
        return Err("保存途中の教材更新があります。再起動して復旧してください。".into());
    }
    model::parse_deck(deck)?;
    progress.validate()?;
    let next = serde_json::to_string_pretty(progress).map_err(|e| e.to_string())?;
    if deck.len() as u64 > limits.deck || next.len() as u64 > limits.progress {
        return Err("教材・学習記録の保存上限を超えています。".into());
    }
    let intent = Intent {
        old_deck: read(&root.join("custom.tsv"), limits.deck)?,
        old_progress: read(&root.join("progress.json"), limits.progress)?,
        deck: deck.into(),
        progress: next,
    };
    // JSON escapes the embedded files; their combined raw sizes are not the intent size.
    let bytes = serde_json::to_vec(&intent).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limits.intent {
        return Err(
            "教材更新の復旧記録が保存上限を超えています。元ファイルは変更していません。".into(),
        );
    }
    atomic_write(&root.join(PENDING), &bytes)?;
    recover_with_limits(root, limits)
}
pub fn recover(root: &Path) -> Result<(), String> {
    recover_with_limits(root, DEFAULT_LIMITS)
}
fn recover_with_limits(root: &Path, limits: Limits) -> Result<(), String> {
    let Some(text) = read(&root.join(PENDING), limits.intent)? else {
        return Ok(());
    };
    let intent: Intent = serde_json::from_str(&text)
        .map_err(|e| format!("保存途中の記録を読めません。元ファイルは保持しました：{e}"))?;
    if intent.deck.len() as u64 > limits.deck
        || intent.progress.len() as u64 > limits.progress
        || intent
            .old_deck
            .as_ref()
            .is_some_and(|s| s.len() as u64 > limits.deck)
        || intent
            .old_progress
            .as_ref()
            .is_some_and(|s| s.len() as u64 > limits.progress)
    {
        return Err(
            "復旧記録内の教材・学習記録が保存上限を超えています。元ファイルは保持しました。".into(),
        );
    }
    model::parse_deck(&intent.deck)?;
    let next: Progress = serde_json::from_str(&intent.progress).map_err(|e| e.to_string())?;
    next.validate()?;
    // Check both files before changing either one. Never overwrite a foreign edit.
    for (name, old, new, limit) in [
        ("custom.tsv", &intent.old_deck, &intent.deck, limits.deck),
        (
            "progress.json",
            &intent.old_progress,
            &intent.progress,
            limits.progress,
        ),
    ] {
        let actual = read(&root.join(name), limit)?;
        if actual.as_ref() != old.as_ref() && actual.as_deref() != Some(new) {
            return Err(format!(
                "{name}が保存途中の記録と一致しません。外部変更を上書きせず停止しました。"
            ));
        }
    }
    atomic_write(&root.join("custom.tsv"), intent.deck.as_bytes())?;
    atomic_write(&root.join("progress.json"), intent.progress.as_bytes())?;
    let backups = root.join("backups");
    fs::create_dir_all(&backups).map_err(|e| e.to_string())?;
    // Retain the intent (including both previous files) as a recovery point.
    fs::rename(
        root.join(PENDING),
        backups.join(format!(
            "material-commit-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        )),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_intent_is_rejected_before_publishing_pending() {
        let root = root();
        let next = Progress::default();
        fs::write(root.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
        let previous = serde_json::to_string(&next).unwrap();
        fs::write(root.join("progress.json"), &previous).unwrap();
        // Each file and even their combined raw bytes fit, but JSON escaping adds bytes.
        let raw_len = 2 * model::BUILTIN_DECK.len()
            + previous.len()
            + serde_json::to_string_pretty(&next).unwrap().len();
        let limits = Limits {
            intent: raw_len as u64,
            ..DEFAULT_LIMITS
        };
        assert!(save_with_limits(&root, model::BUILTIN_DECK, &next, limits).is_err());
        assert!(!pending(&root));
        assert_eq!(
            fs::read_to_string(root.join("custom.tsv")).unwrap(),
            model::BUILTIN_DECK
        );
        assert_eq!(
            fs::read_to_string(root.join("progress.json")).unwrap(),
            previous
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn recovery_checks_each_next_file_before_changing_either() {
        for field in ["deck", "progress"] {
            let root = root();
            let i = intent();
            let mut limits = DEFAULT_LIMITS;
            if field == "deck" {
                limits.deck = i.deck.len() as u64 - 1;
            } else {
                limits.progress = i.progress.len() as u64 - 1;
            }
            fs::write(root.join(PENDING), serde_json::to_vec(&i).unwrap()).unwrap();
            assert!(recover_with_limits(&root, limits).is_err());
            assert!(pending(&root));
            assert!(!root.join("custom.tsv").exists());
            assert!(!root.join("progress.json").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn exact_serialized_intent_limit_can_be_recovered() {
        let root = root();
        let next = Progress::default();
        let i = Intent {
            old_deck: None,
            old_progress: None,
            deck: model::BUILTIN_DECK.into(),
            progress: serde_json::to_string_pretty(&next).unwrap(),
        };
        let limits = Limits {
            deck: i.deck.len() as u64,
            progress: i.progress.len() as u64,
            intent: serde_json::to_vec(&i).unwrap().len() as u64,
        };
        save_with_limits(&root, &i.deck, &next, limits).unwrap();
        assert!(!pending(&root));
        assert_eq!(
            fs::read_to_string(root.join("progress.json")).unwrap(),
            i.progress
        );
        fs::remove_dir_all(root).unwrap();
    }
    fn root() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "ww-commit-{}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    fn intent() -> Intent {
        Intent {
            old_deck: None,
            old_progress: None,
            deck: model::BUILTIN_DECK.into(),
            progress: serde_json::to_string(&Progress::default()).unwrap(),
        }
    }
    #[test]
    fn finishes_half_written_commit_and_keeps_recovery_point() {
        let root = root();
        let i = intent();
        fs::write(root.join(PENDING), serde_json::to_vec(&i).unwrap()).unwrap();
        fs::write(root.join("custom.tsv"), &i.deck).unwrap();
        recover(&root).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("progress.json")).unwrap(),
            i.progress
        );
        assert!(!pending(&root));
        assert_eq!(fs::read_dir(root.join("backups")).unwrap().count(), 1);
        recover(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn foreign_changes_are_preserved_and_block_further_saves() {
        let root = root();
        let i = intent();
        fs::write(root.join(PENDING), serde_json::to_vec(&i).unwrap()).unwrap();
        fs::write(root.join("progress.json"), "foreign edit").unwrap();
        assert!(recover(&root).is_err());
        assert!(!root.join("custom.tsv").exists());
        assert!(save(&root, &i.deck, &Progress::default()).is_err());
        assert_eq!(
            fs::read_to_string(root.join("progress.json")).unwrap(),
            "foreign edit"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
