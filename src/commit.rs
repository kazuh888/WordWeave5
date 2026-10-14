//! Recoverable two-file commit. A pending intent blocks unrelated progress saves.
#[cfg(all(test, windows, feature = "early-trial"))]
mod early_trial_paths_tests {
    use super::*;
    use crate::store::{
        i2_gate_tests::{tree, Temp},
        ProgressCheckpoint, ProgressFaultError, ProgressFaults, Storage,
    };
    const OLD: &str = r#"{"version":2,"audio_history":null,"settings":{"topic":"synthetic old"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
    fn history() -> String {
        OLD.replace(
            "\"audio_history\":null",
            &format!(
                "\"audio_history\":{{\"sha256\":\"{}\",\"bytes\":12}}",
                "a".repeat(64)
            ),
        )
    }
    fn seed(root: &Path, old: Option<&str>, next: &str) -> Vec<u8> {
        let raw = serde_json::to_vec(&serde_json::json!({"old_deck":"synthetic old deck", "old_progress":old, "deck":model::BUILTIN_DECK, "progress":next})).unwrap();
        fs::write(root.join("custom.tsv"), b"synthetic old deck").unwrap();
        fs::write(root.join("progress.json"), OLD).unwrap();
        fs::write(root.join(PENDING), &raw).unwrap();
        raw
    }

    #[test]
    fn et04_pending_preflight_is_read_only_for_absent_and_nonhistory_intents() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("trial")).unwrap();
        let before = tree(&temp.root);
        assert!(check_early_trial_recovery(&storage.dir).is_ok());
        assert_eq!(tree(&temp.root), before);
        for old in [None, Some(OLD)] {
            seed(&storage.dir, old, OLD);
            let before = tree(&temp.root);
            assert!(check_early_trial_recovery(&storage.dir).is_ok());
            assert_eq!(
                tree(&temp.root),
                before,
                "preflight may not replay or archive"
            );
        }
    }

    #[test]
    fn et04_pending_preflight_rejects_history_on_either_side_and_strict_decode_errors() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("trial")).unwrap();
        let history = history();
        let decoded: Progress =
            serde_json::from_str(&history).expect("FIXTURE_FAILURE: structural history");
        assert!(decoded.audio_history.is_some());
        let unknown = OLD.replace("\"version\":2", "\"version\":999");
        let duplicate = OLD.replacen('{', "{\"audio_history\":null,", 1);
        for bad in [
            history.as_str(),
            unknown.as_str(),
            duplicate.as_str(),
            "broken progress",
        ] {
            for (old, next) in [(bad, OLD), (OLD, bad)] {
                seed(&storage.dir, Some(old), next);
                let before = tree(&temp.root);
                assert!(check_early_trial_recovery(&storage.dir).is_err());
                assert_eq!(tree(&temp.root), before);
            }
        }
        fs::write(storage.dir.join(PENDING), b"broken intent").unwrap();
        let before = tree(&temp.root);
        assert!(check_early_trial_recovery(&storage.dir).is_err());
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn et04_pending_preflight_enforces_existing_intent_byte_limit_before_read() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("trial")).unwrap();
        let path = storage.dir.join(PENDING);
        let file = fs::File::create(&path).unwrap();
        file.set_len(DEFAULT_LIMITS.intent + 1).unwrap();
        drop(file);
        assert!(check_early_trial_recovery(&storage.dir).is_err());
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            DEFAULT_LIMITS.intent + 1
        );
        assert_eq!(
            fs::read_dir(&storage.dir).unwrap().count(),
            2,
            "only preexisting app.lock and oversized pending"
        );
    }

    #[test]
    fn et_d01_nonhistory_recovery_fault_after_actual_write_retains_partial_state_then_retries() {
        struct AfterDeck;
        impl ProgressFaults for AfterDeck {
            fn check(&self, point: ProgressCheckpoint) -> Result<(), ProgressFaultError> {
                if point == ProgressCheckpoint::AfterDeck {
                    Err(ProgressFaultError::Io)
                } else {
                    Ok(())
                }
            }
        }
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("trial")).unwrap();
        let next = OLD.replace("synthetic old", "synthetic recovered");
        let raw = seed(&storage.dir, Some(OLD), &next);
        fs::write(temp.root.join("normal-sentinel"), b"normal preserved").unwrap();
        let before = tree(&temp.root);
        // Exercise the real existing postwrite failure independently of the
        // new read-only preflight (covered by the tests above).
        let guard = acquire_progress_write().unwrap();
        let result = recover_with_limits_and_faults_under_guard(
            &storage.dir,
            DEFAULT_LIMITS,
            &guard,
            &AfterDeck,
        );
        assert!(result.is_err());
        let mut expected = before;
        expected.insert(
            "trial/custom.tsv".into(),
            model::BUILTIN_DECK.as_bytes().to_vec(),
        );
        assert_eq!(
            tree(&temp.root),
            expected,
            "actual postwrite failure changes deck only; never claim unchanged"
        );
        assert_eq!(fs::read(storage.dir.join(PENDING)).unwrap(), raw);
        drop(guard);
        recover(&storage.dir).unwrap();
        assert_eq!(
            fs::read(storage.dir.join("progress.json")).unwrap(),
            next.as_bytes()
        );
        assert!(!storage.dir.join(PENDING).exists());
        assert_eq!(
            fs::read(temp.root.join("normal-sentinel")).unwrap(),
            b"normal preserved"
        );
    }
}

use crate::{
    audio_history::{HistoryError, HistoryErrorKind, HistoryLimits, HistoryReader},
    model,
    store::{
        acquire_progress_write, atomic_write, Progress, ProgressGateError, ProgressWriteGuard,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[cfg(feature = "early-trial")]
pub(crate) fn check_early_trial_recovery(root: &Path) -> Result<(), String> {
    if !pending_checked(root).map_err(PendingCheckError::message)? {
        return Ok(());
    }
    let bytes = read_recovery_bytes(&root.join(PENDING), DEFAULT_LIMITS.intent)
        .map_err(recovery_error_message)?
        .ok_or_else(|| recovery_error_message(recovery_error(HistoryErrorKind::Io)))?;
    let intent: Intent =
        serde_json::from_slice(&bytes).map_err(|_| "保存途中の記録が不正です。".to_owned())?;
    if intent.deck.len() as u64 > DEFAULT_LIMITS.deck
        || intent.progress.len() as u64 > DEFAULT_LIMITS.progress
        || intent
            .old_deck
            .as_ref()
            .is_some_and(|s| s.len() as u64 > DEFAULT_LIMITS.deck)
        || intent
            .old_progress
            .as_ref()
            .is_some_and(|s| s.len() as u64 > DEFAULT_LIMITS.progress)
    {
        return Err("保存途中の記録が読取上限を超えています。".into());
    }
    for raw in std::iter::once(intent.progress.as_str()).chain(intent.old_progress.as_deref()) {
        let (_, progress) = crate::store::decode_progress_for_material(raw.as_bytes())
            .map_err(recovery_error_message)?;
        if progress.audio_history.is_some() {
            return Err("音声履歴を含むため、早期試用では開けません。".into());
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PendingCheckError {
    Io,
    UnsafePath,
}

impl PendingCheckError {
    pub(crate) fn message(self) -> String {
        match self {
            Self::Io => "保存途中の教材更新の状態を確認できません。元ファイルは変更していません。",
            Self::UnsafePath => "保存先に安全でないパスがあります。元ファイルは変更していません。",
        }
        .into()
    }
}

fn check_pending_metadata(
    metadata: &fs::Metadata,
    directory: bool,
) -> Result<(), PendingCheckError> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(PendingCheckError::UnsafePath);
        }
    }
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err(PendingCheckError::UnsafePath);
    }
    Ok(())
}

/// Only a missing leaf below a verified directory ancestry means no pending work.
pub(crate) fn pending_checked(root: &Path) -> Result<bool, PendingCheckError> {
    let absolute = std::path::absolute(root).map_err(|_| PendingCheckError::Io)?;
    let ancestors: Vec<_> = absolute.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|_| PendingCheckError::Io)?;
        check_pending_metadata(&metadata, true)?;
    }
    match fs::symlink_metadata(absolute.join(PENDING)) {
        Ok(metadata) => {
            check_pending_metadata(&metadata, false)?;
            Ok(true)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(PendingCheckError::Io),
    }
}

pub(crate) fn recover_under_guard(root: &Path, guard: &ProgressWriteGuard) -> Result<(), String> {
    recover_with_limits_under_guard(root, DEFAULT_LIMITS, guard)
}

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
    pending_checked(root).unwrap_or(true)
}
pub fn save_progress(root: &Path, deck: &str, candidate: &Progress) -> Result<Progress, String> {
    save_progress_with_limits_and_faults(
        root,
        deck,
        candidate,
        DEFAULT_LIMITS,
        &crate::store::NoProgressFaults,
    )
}

fn save_progress_with_limits_and_faults(
    root: &Path,
    deck: &str,
    candidate: &Progress,
    limits: Limits,
    faults: &dyn crate::store::ProgressFaults,
) -> Result<Progress, String> {
    save_progress_with_intent_publisher(root, deck, candidate, limits, faults, atomic_write)
}

fn save_progress_with_intent_publisher<P>(
    root: &Path,
    deck: &str,
    candidate: &Progress,
    limits: Limits,
    faults: &dyn crate::store::ProgressFaults,
    publish_intent: P,
) -> Result<Progress, String>
where
    P: FnOnce(&Path, &[u8]) -> Result<(), String>,
{
    use crate::store::ProgressCheckpoint as CP;
    let guard = acquire_progress_write().map_err(ProgressGateError::message)?;
    guard
        .ensure_writable()
        .map_err(ProgressGateError::message)?;
    if pending_checked(root).map_err(PendingCheckError::message)? {
        return Err("保存途中の教材更新があります。再起動して復旧してください。".into());
    }
    let old_deck = read_recovery_bytes(&root.join("custom.tsv"), limits.deck)
        .map_err(recovery_error_message)?
        .map(String::from_utf8)
        .transpose()
        .map_err(|_| "旧教材の文字コードが不正です。元ファイルは変更していません。")?;
    let old_progress = read_recovery_bytes(&root.join("progress.json"), limits.progress)
        .map_err(recovery_error_message)?;
    let old = old_progress
        .as_deref()
        .map(crate::store::decode_progress_for_material)
        .transpose()
        .map_err(recovery_error_message)?;
    let latest_root = old.and_then(|(_, progress)| progress.audio_history);
    if let Some(history) = &latest_root {
        HistoryReader::for_root(root, HistoryLimits::v1())
            .and_then(|reader| reader.verify_closure(history))
            .map_err(recovery_error_message)?;
    }
    let mut saved = candidate.clone();
    saved.audio_history = latest_root.clone();
    saved.validate()?;
    model::parse_deck(deck)?;
    let next = serde_json::to_string_pretty(&saved).map_err(|e| e.to_string())?;
    if deck.len() as u64 > limits.deck || next.len() as u64 > limits.progress {
        return Err("教材・学習記録の保存上限を超えています。".into());
    }
    if saved.audio_history != latest_root {
        return Err("教材保存の旧新音声履歴が一致しません。".into());
    }
    let intent = Intent {
        old_deck,
        old_progress: old_progress
            .map(String::from_utf8)
            .transpose()
            .map_err(|_| "旧学習記録の文字コードが不正です。")?,
        deck: deck.into(),
        progress: next,
    };
    // Preserve the original strings; normalization never rewrites the old snapshot.
    let bytes = serde_json::to_vec(&intent).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limits.intent {
        return Err(
            "教材更新の復旧記録が保存上限を超えています。元ファイルは変更していません。".into(),
        );
    }
    crate::store::preserve_v1_before_v2_under_guard(
        root,
        intent.old_progress.as_deref().map(str::as_bytes),
        2,
        limits.progress,
        &guard,
        faults,
    )
    .map_err(crate::store::MigrationError::message)?;
    faults
        .check(CP::BeforeMaterialIntent)
        .map_err(|_| recovery_incomplete())?;
    let pending_path = root.join(PENDING);
    // Before publication a changed snapshot is a refusal, not an unknown write.
    if read_recovery_bytes(&pending_path, limits.intent)
        .map_err(recovery_error_message)?
        .is_some()
        || !material_metadata_matches(
            root,
            limits,
            intent.old_deck.as_deref().map(str::as_bytes),
            intent.old_progress.as_deref().map(str::as_bytes),
        )
        .map_err(recovery_error_message)?
    {
        return Err("教材保存前に元ファイルが変化しました。変更せず停止しました。".into());
    }
    guard
        .ensure_writable()
        .map_err(ProgressGateError::message)?;
    let _publish_result = publish_intent(&pending_path, &bytes);
    let after = faults.check(CP::AfterMaterialIntent);
    let published = (|| {
        let pending = read_recovery_bytes(&pending_path, limits.intent)?;
        let old_matches = material_metadata_matches(
            root,
            limits,
            intent.old_deck.as_deref().map(str::as_bytes),
            intent.old_progress.as_deref().map(str::as_bytes),
        )?;
        Ok::<_, HistoryError>((pending, old_matches))
    })();
    match published {
        Ok((Some(actual), true)) if actual == bytes => {}
        Ok((None, true)) => return Err(recovery_incomplete()),
        _ => return Err(recovery_unknown(&guard)),
    }
    after.map_err(|_| recovery_incomplete())?;
    recover_with_limits_and_faults_under_guard(root, limits, &guard, faults)?;
    // R's missing-pending no-op alone is not proof of this save's completion.
    let finished = (|| {
        let pending = read_recovery_bytes(&pending_path, limits.intent)?;
        let new_matches = material_metadata_matches(
            root,
            limits,
            Some(intent.deck.as_bytes()),
            Some(intent.progress.as_bytes()),
        )?;
        Ok::<_, HistoryError>(pending.is_none() && new_matches)
    })();
    if !matches!(finished, Ok(true)) {
        return Err(recovery_unknown(&guard));
    }
    Ok(saved)
}

fn material_metadata_matches(
    root: &Path,
    limits: Limits,
    deck: Option<&[u8]>,
    progress: Option<&[u8]>,
) -> Result<bool, HistoryError> {
    let actual_deck = read_recovery_bytes(&root.join("custom.tsv"), limits.deck)?;
    let actual_progress = read_recovery_bytes(&root.join("progress.json"), limits.progress)?;
    Ok(actual_deck.as_deref() == deck && actual_progress.as_deref() == progress)
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
    save_with_limits_and_faults(
        root,
        deck,
        progress,
        limits,
        &crate::store::NoProgressFaults,
    )
}

fn save_with_limits_and_faults(
    root: &Path,
    deck: &str,
    progress: &Progress,
    limits: Limits,
    faults: &dyn crate::store::ProgressFaults,
) -> Result<(), String> {
    save_progress_with_limits_and_faults(root, deck, progress, limits, faults).map(|_| ())
}
pub fn recover(root: &Path) -> Result<(), String> {
    recover_with_limits(root, DEFAULT_LIMITS)
}
fn recover_with_limits(root: &Path, limits: Limits) -> Result<(), String> {
    let guard = acquire_progress_write().map_err(ProgressGateError::message)?;
    recover_with_limits_under_guard(root, limits, &guard)
}
fn recover_with_limits_and_faults_under_guard(
    root: &Path,
    limits: Limits,
    guard: &ProgressWriteGuard,
    faults: &dyn crate::store::ProgressFaults,
) -> Result<(), String> {
    recover_with_io_under_guard(root, limits, guard, faults, atomic_write, |from, to| {
        fs::rename(from, to)
    })
}

fn recover_with_io_under_guard<W, A>(
    root: &Path,
    limits: Limits,
    guard: &ProgressWriteGuard,
    faults: &dyn crate::store::ProgressFaults,
    mut write: W,
    archive: A,
) -> Result<(), String>
where
    W: FnMut(&Path, &[u8]) -> Result<(), String>,
    A: FnOnce(&Path, &Path) -> std::io::Result<()>,
{
    use crate::store::ProgressCheckpoint as CP;
    guard
        .ensure_writable()
        .map_err(ProgressGateError::message)?;
    if !pending_checked(root).map_err(PendingCheckError::message)? {
        return Ok(());
    }
    let pending_path = root.join(PENDING);
    let pending_bytes = read_recovery_bytes(&pending_path, limits.intent)
        .map_err(recovery_error_message)?
        .ok_or_else(|| recovery_error_message(recovery_error(HistoryErrorKind::Io)))?;
    let intent: Intent = serde_json::from_slice(&pending_bytes)
        .map_err(|_| "保存途中の記録が不正です。元ファイルは保持しました。".to_owned())?;
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
    let (new_wire, next) = crate::store::decode_progress_for_material(intent.progress.as_bytes())
        .map_err(recovery_error_message)?;
    let old = intent
        .old_progress
        .as_deref()
        .map(|raw| crate::store::decode_progress_for_material(raw.as_bytes()))
        .transpose()
        .map_err(recovery_error_message)?;
    let old_root = old
        .as_ref()
        .and_then(|(_, progress)| progress.audio_history.as_ref());
    if old_root != next.audio_history.as_ref() {
        return Err("教材復旧の旧新音声履歴が一致しません。元ファイルは保持しました。".into());
    }
    if let Some(history) = &next.audio_history {
        HistoryReader::for_root(root, HistoryLimits::v1())
            .and_then(|reader| reader.verify_closure(history))
            .map_err(recovery_error_message)?;
    }
    let snapshot = RecoverySnapshot {
        root,
        limits,
        intent: &intent,
        pending: &pending_bytes,
    };
    snapshot.known_state().map_err(recovery_error_message)?;
    let stamp = chrono::Utc::now()
        .timestamp_nanos_opt()
        .ok_or("教材復旧の退避名を決定できません。")?;
    let archive_path = root.join("backups").join(format!(
        "material-commit-{}-{stamp}.json",
        std::process::id()
    ));
    recovery_archive_destination(root, &archive_path).map_err(recovery_error_message)?;

    // Migration uses intent old bytes even when current progress is already new.
    let migration = if new_wire == 2 && old.as_ref().is_some_and(|(wire, _)| *wire == 1) {
        crate::store::preserve_v1_before_v2_under_guard(
            root,
            intent.old_progress.as_deref().map(str::as_bytes),
            new_wire,
            limits.progress,
            guard,
            faults,
        )
        .map_err(crate::store::MigrationError::message)
    } else {
        Ok(())
    };
    snapshot.recheck(guard)?;
    migration?;

    for (index, name, bytes, before, after) in [
        (
            0,
            "custom.tsv",
            intent.deck.as_bytes(),
            CP::BeforeDeck,
            CP::AfterDeck,
        ),
        (
            1,
            "progress.json",
            intent.progress.as_bytes(),
            CP::BeforeMaterialProgress,
            CP::AfterMaterialProgress,
        ),
    ] {
        let state = snapshot.checkpoint(before, guard, faults)?;
        if index == 1 && !state[0] {
            return Err(recovery_incomplete());
        }
        guard
            .ensure_writable()
            .map_err(ProgressGateError::message)?;
        let _write_result = write(&root.join(name), bytes);
        let state = snapshot.recheck(guard)?;
        if !state[index] {
            return Err(recovery_incomplete());
        }
        let state = snapshot.checkpoint(after, guard, faults)?;
        if !state[index] || (index == 1 && !state[0]) {
            return Err(recovery_incomplete());
        }
    }
    if snapshot.checkpoint(CP::BeforeMaterialFinish, guard, faults)? != [true, true] {
        return Err(recovery_incomplete());
    }
    // A failure before rename is retryable when metadata and pending remain known.
    let prepare_archive = (|| {
        let exists = recovery_archive_destination(root, &archive_path)?;
        if !exists {
            fs::create_dir(root.join("backups"))
                .map_err(|_| recovery_error(HistoryErrorKind::Io))?;
        }
        recovery_archive_destination(root, &archive_path)?;
        Ok::<_, HistoryError>(())
    })();
    if snapshot.recheck(guard)? != [true, true] {
        return Err(recovery_incomplete());
    }
    prepare_archive.map_err(recovery_error_message)?;
    guard
        .ensure_writable()
        .map_err(ProgressGateError::message)?;
    let _archive_result = archive(&pending_path, &archive_path);
    let _finish_fault = faults.check(CP::AfterMaterialFinish);
    let finish = (|| {
        let state = snapshot.metadata_state()?;
        let pending = read_recovery_bytes(&pending_path, limits.intent)?;
        let archived = read_recovery_bytes(&archive_path, limits.intent)?;
        Ok::<_, HistoryError>((state, pending, archived))
    })();
    match finish {
        Ok(([true, true], None, Some(bytes))) if bytes == pending_bytes => Ok(()),
        Ok(([true, true], Some(pending), archived))
            if pending == pending_bytes
                && archived
                    .as_deref()
                    .is_none_or(|bytes| bytes == pending_bytes) =>
        {
            Err(recovery_incomplete())
        }
        _ => Err(recovery_unknown(guard)),
    }
}

fn read_recovery_bytes(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, HistoryError> {
    use std::io::Read;
    let absolute = std::path::absolute(path).map_err(|_| recovery_error(HistoryErrorKind::Io))?;
    let parent = absolute
        .parent()
        .ok_or_else(|| recovery_error(HistoryErrorKind::UnsafePath))?;
    recovery_directory(parent)?;
    let Some(metadata) = recovery_metadata(&absolute, false)? else {
        return Ok(None);
    };
    if metadata.len() > limit {
        return Err(recovery_error(HistoryErrorKind::LimitExceeded));
    }
    // A missing leaf at open/read is no longer a verified absence.
    let file = fs::File::open(&absolute).map_err(|_| recovery_error(HistoryErrorKind::Io))?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| recovery_error(HistoryErrorKind::Io))?;
    if bytes.len() as u64 > limit {
        return Err(recovery_error(HistoryErrorKind::LimitExceeded));
    }
    Ok(Some(bytes))
}

fn recovery_error(kind: HistoryErrorKind) -> HistoryError {
    HistoryError { kind }
}

fn recovery_error_message(error: HistoryError) -> String {
    match error.kind {
        HistoryErrorKind::UnsafePath => "教材復旧先に安全でないパスがあります。",
        HistoryErrorKind::LimitExceeded => "教材復旧の読取上限を超えています。",
        HistoryErrorKind::UnsupportedVersion => "教材復旧のデータ版に対応していません。",
        HistoryErrorKind::Io => "教材復旧のファイルを確認できません。",
        _ => "教材復旧のデータが不正または記録と一致しません。",
    }
    .into()
}

fn recovery_incomplete() -> String {
    "教材復旧は未完了です。保存途中の記録を保持して停止しました。".into()
}

fn recovery_unknown(guard: &ProgressWriteGuard) -> String {
    guard.block_writes();
    "教材復旧の結果を確認できません。回復確認が必要なため書き込みを停止しました。".into()
}

fn recovery_metadata(path: &Path, directory: bool) -> Result<Option<fs::Metadata>, HistoryError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(recovery_error(HistoryErrorKind::Io)),
    };
    check_pending_metadata(&metadata, directory).map_err(|error| {
        recovery_error(match error {
            PendingCheckError::UnsafePath => HistoryErrorKind::UnsafePath,
            PendingCheckError::Io => HistoryErrorKind::Io,
        })
    })?;
    Ok(Some(metadata))
}

fn recovery_directory(path: &Path) -> Result<(), HistoryError> {
    let absolute = std::path::absolute(path).map_err(|_| recovery_error(HistoryErrorKind::Io))?;
    let ancestors: Vec<_> = absolute.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        recovery_metadata(ancestor, true)?.ok_or_else(|| recovery_error(HistoryErrorKind::Io))?;
    }
    Ok(())
}

/// Returns whether the normal backups directory already exists; never creates it.
fn recovery_archive_destination(root: &Path, path: &Path) -> Result<bool, HistoryError> {
    recovery_directory(root)?;
    let exists = recovery_metadata(&root.join("backups"), true)?.is_some();
    if recovery_metadata(path, false)?.is_some() {
        return Err(recovery_error(HistoryErrorKind::Conflict));
    }
    Ok(exists)
}

struct RecoverySnapshot<'a> {
    root: &'a Path,
    limits: Limits,
    intent: &'a Intent,
    pending: &'a [u8],
}

impl RecoverySnapshot<'_> {
    /// Each bool records exact new bytes; old==new deliberately prefers new.
    fn metadata_state(&self) -> Result<[bool; 2], HistoryError> {
        let mut state = [false; 2];
        for (index, name, old, new, limit) in [
            (
                0,
                "custom.tsv",
                &self.intent.old_deck,
                &self.intent.deck,
                self.limits.deck,
            ),
            (
                1,
                "progress.json",
                &self.intent.old_progress,
                &self.intent.progress,
                self.limits.progress,
            ),
        ] {
            let actual = read_recovery_bytes(&self.root.join(name), limit)?;
            if actual.as_deref() == Some(new.as_bytes()) {
                state[index] = true;
            } else if actual.as_deref() != old.as_deref().map(str::as_bytes) {
                return Err(recovery_error(HistoryErrorKind::Conflict));
            }
        }
        Ok(state)
    }

    fn known_state(&self) -> Result<[bool; 2], HistoryError> {
        let state = self.metadata_state()?;
        if read_recovery_bytes(&self.root.join(PENDING), self.limits.intent)?.as_deref()
            != Some(self.pending)
        {
            return Err(recovery_error(HistoryErrorKind::Conflict));
        }
        Ok(state)
    }

    fn recheck(&self, guard: &ProgressWriteGuard) -> Result<[bool; 2], String> {
        self.known_state().map_err(|_| recovery_unknown(guard))
    }

    fn checkpoint(
        &self,
        point: crate::store::ProgressCheckpoint,
        guard: &ProgressWriteGuard,
        faults: &dyn crate::store::ProgressFaults,
    ) -> Result<[bool; 2], String> {
        let fault = faults.check(point);
        let state = self.recheck(guard)?;
        fault.map_err(|_| recovery_incomplete())?;
        Ok(state)
    }
}

fn recover_with_limits_under_guard(
    root: &Path,
    limits: Limits,
    guard: &ProgressWriteGuard,
) -> Result<(), String> {
    recover_with_limits_and_faults_under_guard(root, limits, guard, &crate::store::NoProgressFaults)
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

#[cfg(all(test, windows))]
mod i2_gate_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};

    #[test]
    fn pending_normal_parent_distinguishes_missing_leaf_from_regular_file() {
        let temp = Temp::new();
        let before = tree(&temp.root);
        assert_eq!(pending_checked(&temp.root), Ok(false));
        assert!(!pending(&temp.root));
        assert_eq!(tree(&temp.root), before);
        fs::write(
            temp.root.join(PENDING),
            b"pending marker contents are not decoded by presence check",
        )
        .unwrap();
        let before = tree(&temp.root);
        assert_eq!(pending_checked(&temp.root), Ok(true));
        assert!(pending(&temp.root));
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn pending_directory_leaf_and_non_directory_ancestry_are_unsafe() {
        let temp = Temp::new();
        let host = temp.root.join("host");
        fs::create_dir_all(host.join(PENDING)).unwrap();
        let file = temp.root.join("file");
        fs::write(&file, b"not a directory").unwrap();
        let before = tree(&temp.root);
        for root in [&host, &file, &file.join("child")] {
            assert_eq!(pending_checked(root), Err(PendingCheckError::UnsafePath));
            assert!(pending(root));
            assert!(save(root, crate::model::BUILTIN_DECK, &Progress::default()).is_err());
            assert!(recover(root).is_err());
        }
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn pending_os_io_rejection_fails_closed_without_any_write() {
        let temp = Temp::new();
        let invalid = temp.root.join("nul\0component");
        let os_error = fs::symlink_metadata(&invalid)
            .expect_err("the fixture must reach a real OS path error");
        assert_ne!(
            os_error.kind(),
            std::io::ErrorKind::NotFound,
            "NotFound would not prove unknown I/O"
        );
        eprintln!(
            "I/O fixture OS rejection: kind={:?}, code={:?}",
            os_error.kind(),
            os_error.raw_os_error()
        );
        let before = tree(&temp.root);
        assert_eq!(pending_checked(&invalid), Err(PendingCheckError::Io));
        assert!(
            pending(&invalid),
            "public bool must fail closed on the independently proven I/O error"
        );
        assert!(save(&invalid, crate::model::BUILTIN_DECK, &Progress::default()).is_err());
        assert!(recover(&invalid).is_err());
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn pending_missing_root_is_an_error_and_never_created() {
        let temp = Temp::new();
        let absent = temp.root.join("absent");
        let before = tree(&temp.root);
        assert_eq!(pending_checked(&absent), Err(PendingCheckError::Io));
        assert!(pending(&absent));
        assert!(save(&absent, crate::model::BUILTIN_DECK, &Progress::default()).is_err());
        assert!(recover(&absent).is_err());
        assert_eq!(tree(&temp.root), before);
        assert!(!absent.exists());
    }

    #[test]
    fn pending_reparse_ancestor_root_and_leaf_are_unsafe_without_mutation() {
        for placement in 0..3 {
            let mut temp = Temp::new();
            let parent = temp.root.join("real-parent");
            let host = parent.join("host");
            fs::create_dir_all(&host).unwrap();
            fs::write(host.join("sentinel"), b"preserve target").unwrap();
            assert_eq!(
                pending_checked(&host),
                Ok(false),
                "normal directory ancestry is valid"
            );
            let exposed = match placement {
                0 => {
                    let link = temp.root.join("linked-parent");
                    temp.junction(&link, &parent);
                    link.join("host")
                }
                1 => {
                    let link = temp.root.join("linked-root");
                    temp.junction(&link, &host);
                    link
                }
                _ => {
                    let target = temp.root.join("leaf-target");
                    fs::create_dir(&target).unwrap();
                    temp.junction(&host.join(PENDING), &target);
                    host.clone()
                }
            };
            assert_eq!(
                fs::read(exposed.join("sentinel")).unwrap(),
                b"preserve target"
            );
            let before = tree(&temp.root);
            assert_eq!(
                pending_checked(&exposed),
                Err(PendingCheckError::UnsafePath)
            );
            assert!(pending(&exposed));
            assert!(save(&exposed, crate::model::BUILTIN_DECK, &Progress::default()).is_err());
            assert!(recover(&exposed).is_err());
            assert_eq!(tree(&temp.root), before);
        }
    }
}

#[cfg(all(test, windows))]
mod i2_migration_tests {
    use super::*;
    use crate::store::{
        i2_gate_tests::{tree, Temp},
        i2_migration_tests::{Faults, HASH, RAW_V1},
        ProgressCheckpoint,
    };

    fn seed(root: &Path, old: &[u8]) {
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("custom.tsv"), model::BUILTIN_DECK).unwrap();
        fs::write(root.join("progress.json"), old).unwrap();
        fs::write(root.join("assets/keep.bin"), b"synthetic material asset").unwrap();
    }

    #[test]
    fn m05_v2_material_private_entry_matches_public_save_with_exact_migration() {
        let temp = Temp::new();
        let control = temp.root.join("control");
        let root = temp.root.join("subject");
        seed(&control, RAW_V1.as_bytes());
        seed(&root, RAW_V1.as_bytes());
        let mut next: Progress = serde_json::from_str(RAW_V1).unwrap();
        next.validate().unwrap();
        next.settings.topic = "new synthetic topic".into();
        save_with_limits(&control, model::BUILTIN_DECK, &next, DEFAULT_LIMITS)
            .expect("existing material save is the fixture control");
        let faults = Faults::new(None);
        let result =
            save_with_limits_and_faults(&root, model::BUILTIN_DECK, &next, DEFAULT_LIMITS, &faults);
        assert!(
            result.is_ok(),
            "v2 private entry must match public material save: {result:?}"
        );
        assert_eq!(
            *faults.seen.borrow(),
            [
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup,
                ProgressCheckpoint::BeforeMaterialIntent,
                ProgressCheckpoint::AfterMaterialIntent,
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup,
                ProgressCheckpoint::BeforeDeck,
                ProgressCheckpoint::AfterDeck,
                ProgressCheckpoint::BeforeMaterialProgress,
                ProgressCheckpoint::AfterMaterialProgress,
                ProgressCheckpoint::BeforeMaterialFinish,
                ProgressCheckpoint::AfterMaterialFinish,
            ]
        );
        assert_eq!(
            fs::read(root.join("progress.json")).unwrap(),
            fs::read(control.join("progress.json")).unwrap()
        );
        assert_eq!(
            fs::read(root.join("assets/keep.bin")).unwrap(),
            b"synthetic material asset"
        );
        assert!(!root.join(PENDING).exists());
        let hash_path = root.join(format!("backups/progress-v1-{HASH}.json"));
        assert_eq!(fs::read(&hash_path).unwrap(), RAW_V1.as_bytes());
        let archives: Vec<_> = fs::read_dir(root.join("backups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| *p != hash_path)
            .collect();
        assert_eq!(archives.len(), 1);
        let archived: serde_json::Value =
            serde_json::from_slice(&fs::read(&archives[0]).unwrap()).unwrap();
        assert_eq!(archived["old_progress"].as_str(), Some(RAW_V1));
        let wire: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("progress.json")).unwrap()).unwrap();
        assert_eq!(wire["version"], 2);
        assert_eq!(wire.get("audio_history"), Some(&serde_json::Value::Null));
    }

    #[test]
    fn m05_v2_material_migration_faults_never_publish_pending_or_metadata() {
        for fail in [
            ProgressCheckpoint::BeforeMigrationBackup,
            ProgressCheckpoint::AfterMigrationBackup,
        ] {
            let temp = Temp::new();
            let root = temp.root.join("subject");
            seed(&root, RAW_V1.as_bytes());
            let before = tree(&temp.root);
            let faults = Faults::new(Some(fail));
            let next: Progress = serde_json::from_str(RAW_V1).unwrap();
            let result = save_with_limits_and_faults(
                &root,
                model::BUILTIN_DECK,
                &next,
                DEFAULT_LIMITS,
                &faults,
            );
            assert!(result.is_err());
            let expected_seen = if fail == ProgressCheckpoint::BeforeMigrationBackup {
                vec![fail]
            } else {
                vec![ProgressCheckpoint::BeforeMigrationBackup, fail]
            };
            assert_eq!(*faults.seen.borrow(), expected_seen);
            let mut expected = before;
            if fail == ProgressCheckpoint::AfterMigrationBackup {
                expected.insert(std::path::PathBuf::from("subject/backups"), vec![]);
                expected.insert(
                    std::path::PathBuf::from(format!("subject/backups/progress-v1-{HASH}.json")),
                    RAW_V1.as_bytes().to_vec(),
                );
            }
            assert!(tree(&temp.root) == expected);
        }
    }

    #[test]
    fn m05_v1_material_preserves_old_source_and_escaped_intent_limits() {
        for oversized_old in [true, false] {
            let temp = Temp::new();
            let control = temp.root.join("control");
            let root = temp.root.join("subject");
            let old = if oversized_old {
                format!("{RAW_V1}{}", " ".repeat(8192))
            } else {
                RAW_V1.into()
            };
            seed(&control, old.as_bytes());
            seed(&root, old.as_bytes());
            let next = Progress::default();
            let new_len = serde_json::to_string_pretty(&next).unwrap().len() as u64;
            let raw_intent_len = 2 * model::BUILTIN_DECK.len() as u64 + old.len() as u64 + new_len;
            let limits = Limits {
                deck: DECK_LIMIT,
                progress: if oversized_old {
                    new_len
                } else {
                    PROGRESS_LIMIT
                },
                intent: if oversized_old {
                    DEFAULT_LIMITS.intent
                } else {
                    raw_intent_len
                },
            };
            let baseline = tree(&temp.root);
            assert!(
                save_with_limits(&control, model::BUILTIN_DECK, &next, limits).is_err(),
                "existing limits must reject the fixture"
            );
            assert!(tree(&temp.root) == baseline);
            let faults = Faults::new(Some(ProgressCheckpoint::BeforeMigrationBackup));
            assert!(save_with_limits_and_faults(
                &root,
                model::BUILTIN_DECK,
                &next,
                limits,
                &faults
            )
            .is_err());
            assert!(faults.seen.borrow().is_empty());
            assert!(
                tree(&temp.root) == baseline,
                "rejected v1 material save must not publish pending or backup"
            );
        }
    }
}

#[cfg(all(test, windows))]
mod i2_material_recovery_tests {
    use super::*;
    use crate::{
        audio_history::*,
        model,
        store::{
            i2_gate_tests::{tree, Temp},
            i2_migration_tests::{Faults, HASH, RAW_V1},
            NoProgressFaults, ProgressCheckpoint as CP, ProgressFaultError, ProgressFaults,
            ProgressHistoryHead, Storage,
        },
    };
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        fs::{File, OpenOptions},
        os::windows::fs::OpenOptionsExt,
        path::PathBuf,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    const V2: &str = r#"{ "version":2,"audio_history":null,"settings":{"topic":"next raw"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{} }"#;
    const A: &[u8] =
        br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
    const AH: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
    const SMALL: Limits = Limits {
        deck: 100_000,
        progress: 100_000,
        intent: 500_000,
    };
    type Ledger = BTreeMap<PathBuf, Vec<u8>>;
    struct F {
        storage: Storage,
        temp: Temp,
        old: Option<String>,
        next: String,
        old_deck: Option<String>,
        deck: String,
        raw: Vec<u8>,
    }
    impl F {
        fn new(old: Option<&str>, next: &str, state: u8) -> Self {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            let mut f = Self {
                storage,
                temp,
                old: old.map(str::to_owned),
                next: next.into(),
                old_deck: Some("old deck need not parse".into()),
                deck: model::BUILTIN_DECK.into(),
                raw: vec![],
            };
            fs::write(
                f.root().join("keep.wav"),
                b"synthetic recording must survive",
            )
            .unwrap();
            fs::write(f.temp.root.join("neighbor"), b"neighbor must survive").unwrap();
            f.seed(state);
            f
        }
        fn root(&self) -> &Path {
            &self.storage.dir
        }
        fn path(&self, name: &str) -> PathBuf {
            self.root().join(name)
        }
        fn seed(&mut self, state: u8) {
            for (name, old, new, bit) in [
                ("custom.tsv", &self.old_deck, &self.deck, 1),
                ("progress.json", &self.old, &self.next, 2),
            ] {
                if let Some(raw) = if state & bit == 0 {
                    old.as_deref()
                } else {
                    Some(new.as_str())
                } {
                    fs::write(self.path(name), raw).unwrap();
                } else {
                    match fs::remove_file(self.path(name)) {
                        Ok(()) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => panic!("fixture remove: {e}"),
                    }
                }
            }
            // Only string escaping uses serde; no Progress/Intent product writer supplies the oracle.
            let q = |s: &str| serde_json::to_string(s).unwrap();
            self.raw = format!(
                "{{\n \"old_deck\":{}, \"old_progress\":{},\n \"deck\":{}, \"progress\":{}\n}}\n",
                self.old_deck
                    .as_deref()
                    .map(q)
                    .unwrap_or_else(|| "null".into()),
                self.old.as_deref().map(q).unwrap_or_else(|| "null".into()),
                q(&self.deck),
                q(&self.next)
            )
            .into_bytes();
            fs::write(self.path(PENDING), &self.raw).unwrap();
        }
        fn ledger(&self) -> Ledger {
            tree(&self.temp.root)
        }
        fn hash_path(&self) -> PathBuf {
            self.path(&format!("backups/progress-v1-{HASH}.json"))
        }
        fn exact_limits(&self) -> Limits {
            Limits {
                deck: self
                    .deck
                    .len()
                    .max(self.old_deck.as_ref().map_or(0, String::len))
                    as u64,
                progress: self
                    .next
                    .len()
                    .max(self.old.as_ref().map_or(0, String::len)) as u64,
                intent: self.raw.len() as u64,
            }
        }
        fn io<W, Aa>(
            &self,
            limits: Limits,
            faults: &dyn ProgressFaults,
            write: W,
            archive: Aa,
        ) -> Result<(), String>
        where
            W: FnMut(&Path, &[u8]) -> Result<(), String>,
            Aa: FnOnce(&Path, &Path) -> std::io::Result<()>,
        {
            let guard = acquire_progress_write().map_err(ProgressGateError::message)?;
            let result =
                recover_with_io_under_guard(self.root(), limits, &guard, faults, write, archive);
            drop(guard);
            result
        }
        fn faults(&self, faults: &dyn ProgressFaults) -> Result<(), String> {
            let g = acquire_progress_write().map_err(ProgressGateError::message)?;
            let r = recover_with_limits_and_faults_under_guard(self.root(), SMALL, &g, faults);
            drop(g);
            r
        }
        fn done(&self, mut expected: Ledger, migration: bool) {
            expected.remove(Path::new("host/pending-material-commit.json"));
            expected.insert("host/custom.tsv".into(), self.deck.as_bytes().to_vec());
            expected.insert("host/progress.json".into(), self.next.as_bytes().to_vec());
            expected.insert("host/backups".into(), vec![]);
            if migration {
                expected.insert(
                    format!("host/backups/progress-v1-{HASH}.json").into(),
                    RAW_V1.as_bytes().to_vec(),
                );
            }
            let new_archives: Vec<_> = fs::read_dir(self.path("backups"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| {
                    p.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("material-commit-")
                        && !expected.contains_key(p.strip_prefix(&self.temp.root).unwrap())
                })
                .collect();
            assert_eq!(
                new_archives.len(),
                1,
                "one new archive, no invented pending"
            );
            assert_eq!(fs::read(&new_archives[0]).unwrap(), self.raw);
            expected.insert(
                new_archives[0]
                    .strip_prefix(&self.temp.root)
                    .unwrap()
                    .into(),
                self.raw.clone(),
            );
            assert_eq!(self.ledger(), expected);
            let before = self.ledger();
            recover(self.root()).unwrap();
            assert_eq!(self.ledger(), before);
        }
        fn root_a(&mut self) -> RootRef {
            fs::create_dir_all(self.path("audio-history/assets")).unwrap();
            let r = RootRef::new(BlobRef {
                sha256: Sha256::parse(AH).unwrap(),
                bytes: 82,
            });
            fs::write(blob(self.root(), r.as_blob()), A).unwrap();
            let closure = HistoryReader::new(&self.storage)
                .verify_closure(&r)
                .expect("manual A fixture");
            assert_eq!(closure.objects, vec![r.as_blob().clone()]);
            assert_eq!(closure.total_bytes, 82);
            r
        }
        fn closure3(&mut self) -> (RootRef, BlobRef, BlobRef) {
            fs::create_dir_all(self.path("audio-history/assets")).unwrap();
            let original = put(self.root(), b"independent synthetic original");
            let trial = TrialRevision {
                schema_version: 1,
                id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
                revision: 1,
                previous: None,
                created_at: RecordedAt {
                    utc_millis: 1_700_000_000_000,
                    offset_minutes: 540,
                },
                input: FrozenInput {
                    audio: AudioSnapshot {
                        original: original.clone(),
                        format: AudioFormat::Wav,
                        rate: 8000,
                        channels: 1,
                        frames: 1,
                        origin: Origin::Imported,
                        warnings: vec![],
                    },
                    reference: ReferenceSnapshot {
                        text: "Synthetic reference".into(),
                        sha256: digest(b"Synthetic reference"),
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
            };
            trial.validate().unwrap();
            let tr = put(self.root(), &serde_json::to_vec(&trial).unwrap());
            let root = HistoryRoot {
                schema_version: 1,
                revision: 1,
                next_save_sequence: 2,
                trials: BTreeMap::from([(
                    trial.id,
                    TrialHead {
                        document: tr.clone(),
                        admitted_sequence: 1,
                    },
                )]),
                receipts: BTreeMap::new(),
            };
            root.validate().unwrap();
            let rr = RootRef::new(put(self.root(), &serde_json::to_vec(&root).unwrap()));
            let mut expected = vec![rr.as_blob().clone(), tr.clone(), original.clone()];
            expected.sort_by(|a, b| a.sha256.cmp(&b.sha256));
            let closure = HistoryReader::new(&self.storage)
                .verify_closure(&rr)
                .expect("hand-built three-object closure");
            assert_eq!(closure.objects, expected);
            assert_eq!(
                closure.total_bytes,
                expected.iter().map(|b| b.bytes).sum::<u64>()
            );
            (rr, tr, original)
        }
    }
    fn digest(bytes: &[u8]) -> Sha256 {
        Sha256::parse(&crate::assets::sha256(bytes).unwrap()).unwrap()
    }
    fn blob(root: &Path, b: &BlobRef) -> PathBuf {
        root.join(format!("audio-history/assets/{}.bin", b.sha256.as_str()))
    }
    fn put(root: &Path, bytes: &[u8]) -> BlobRef {
        let b = BlobRef {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
        };
        fs::write(blob(root, &b), bytes).unwrap();
        b
    }
    fn with_root(r: &RootRef) -> String {
        V2.replace(
            "\"audio_history\":null",
            &format!(
                r#""audio_history":{{"sha256":"{}","bytes":{}}}"#,
                r.as_blob().sha256.as_str(),
                r.as_blob().bytes
            ),
        )
    }
    fn writable() {
        let result = acquire_progress_write().map(|guard| guard.ensure_writable());
        assert!(matches!(result, Ok(Ok(()))));
    }
    fn kind<T>(r: Result<T, HistoryError>, k: HistoryErrorKind) {
        assert_eq!(r.err().expect("typed rejection").kind, k);
    }
    fn rename(from: &Path, to: &Path) -> std::io::Result<()> {
        fs::rename(from, to)
    }
    fn reject(f: &F, limits: Limits) {
        let before = f.ledger();
        let writes = Cell::new(0);
        let archives = Cell::new(0);
        let faults = Faults::new(None);
        let r = f.io(
            limits,
            &faults,
            |_, _| {
                writes.set(writes.get() + 1);
                Ok(())
            },
            |_, _| {
                archives.set(archives.get() + 1);
                Ok(())
            },
        );
        assert!(r.is_err());
        assert_eq!((writes.get(), archives.get()), (0, 0));
        assert!(faults.seen.borrow().is_empty());
        assert_eq!(f.ledger(), before);
        writable();
    }
    fn child_cases(case: &str, count: usize, body: impl Fn(usize)) {
        const ENV: &str = "WW5_I2_R_CHILD";
        const MODE: &str = "WW5_I2_R_MODE";
        if std::env::var(ENV).as_deref() == Ok(case) {
            let mode = std::env::var(MODE).unwrap().parse().unwrap();
            body(mode);
            println!("WW5_R_CHILD_OK:{case}:{mode}");
            return;
        }
        for mode in 0..count {
            let temp = Temp::new();
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([case, "--exact", "--nocapture", "--test-threads=1"])
                .env(ENV, case)
                .env(MODE, mode.to_string())
                .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut timeout = false;
            loop {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    timeout = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let output = child.wait_with_output().unwrap();
            let out = String::from_utf8_lossy(&output.stdout);
            let err = String::from_utf8_lossy(&output.stderr);
            assert!(!timeout, "15s child killed/reaped: {out}\n{err}");
            assert!(output.status.success(), "scenario {mode}: {out}\n{err}");
            assert!(
                out.contains("running 1 test")
                    && out.contains("1 passed; 0 failed")
                    && out.contains(&format!("WW5_R_CHILD_OK:{case}:{mode}"))
            );
            for line in err.lines().filter(|line| line.starts_with("R ")) {
                eprintln!("{line}");
            }
        }
    }
    #[test]
    fn r01_codec_bridge_keeps_source_wire_and_strict_error_categories() {
        for (raw, wire) in [(RAW_V1, 1), (V2, 2)] {
            let (actual, p) = crate::store::decode_progress_for_material(raw.as_bytes()).unwrap();
            assert_eq!(actual, wire);
            assert_eq!(p.version, 2);
            assert_eq!(p.audio_history, None);
            p.validate().unwrap();
        }
        for raw in [
            "{}".to_owned(),
            V2.replace("\"version\":2", "\"version\":2,\"version\":2"),
            V2.replace(
                "\"audio_history\":null",
                "\"audio_history\":null,\"audio_history\":null",
            ),
            r#"{"version":2}"#.to_owned(),
        ] {
            kind(
                crate::store::decode_progress_for_material(raw.as_bytes()),
                HistoryErrorKind::InvalidDocument,
            );
        }
        kind(
            crate::store::decode_progress_for_material(
                V2.replace("\"version\":2", "\"version\":77").as_bytes(),
            ),
            HistoryErrorKind::UnsupportedVersion,
        );
    }
    #[test]
    fn r01_missing_pending_is_noop_without_callbacks_or_checkpoint() {
        let f = F::new(Some(V2), V2, 0);
        fs::remove_file(f.path(PENDING)).unwrap();
        let before = f.ledger();
        let faults = Faults::new(None);
        let calls = Cell::new(0);
        let r = f.io(
            SMALL,
            &faults,
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        );
        assert!(r.is_ok());
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert_eq!(f.ledger(), before);
    }
    #[test]
    fn r02_same_root_and_none_are_positive_controls() {
        for has_root in [true, false] {
            let mut f = F::new(Some(V2), V2, 0);
            if has_root {
                let a = f.root_a();
                f.next = with_root(&a);
                f.old = Some(f.next.replace("next raw", "old raw"));
                f.seed(0);
            }
            let before = f.ledger();
            f.io(SMALL, &NoProgressFaults, atomic_write, rename)
                .unwrap();
            f.done(before, false);
        }
    }
    #[test]
    fn r03_exact_and_one_byte_over_each_embedded_and_pending_limit() {
        for field in 0..5 {
            let mut f = F::new(Some(V2), V2, 0);
            match field {
                0 => f.old_deck = Some("x".repeat(f.deck.len() + 1)),
                1 => f.deck.push('\n'),
                2 => f.old = Some(format!("{} ", V2)),
                3 => f.next.push(' '),
                _ => {}
            }
            f.seed(0);
            let exact = f.exact_limits();
            let mut small = exact;
            match field {
                0 | 1 => small.deck -= 1,
                2 | 3 => small.progress -= 1,
                _ => small.intent -= 1,
            }
            reject(&f, small);
            let before = f.ledger();
            f.io(exact, &NoProgressFaults, atomic_write, rename)
                .unwrap();
            f.done(before, false);
        }
    }
    #[test]
    fn r03_preflight_invalid_documents_third_current_and_backup_type_are_unchanged() {
        for mode in 0..9 {
            let mut f = F::new(Some(V2), V2, 0);
            match mode {
                0 => f.next = "{broken".into(),
                1 => f.old = Some("{broken".into()),
                2 => f.next = V2.replace("\"version\":2", "\"version\":99"),
                3 => f.old = Some(V2.replace("\"version\":2", "\"version\":99")),
                4 => f.deck = "not a valid new deck".into(),
                _ => {}
            }
            f.seed(0);
            match mode {
                5 => fs::write(f.path("progress.json"), b"third bytes").unwrap(),
                6 => fs::write(f.path("custom.tsv"), b"third bytes").unwrap(),
                7 => {
                    fs::remove_file(f.path("custom.tsv")).unwrap();
                }
                8 => fs::write(f.path("backups"), b"not a directory").unwrap(),
                _ => {}
            }
            reject(&f, SMALL);
        }
    }
    #[test]
    fn r03_full_three_object_closure_not_just_root_is_required_before_public_replay() {
        for mode in 0..3 {
            let mut f = F::new(Some(V2), V2, 0);
            let (mut root, tr, original) = f.closure3();
            match mode {
                0 => fs::remove_file(blob(f.root(), &original)).unwrap(),
                1 => fs::write(blob(f.root(), &original), b"damaged original").unwrap(),
                _ => {
                    let raw = fs::read(blob(f.root(), &tr)).unwrap();
                    let text = String::from_utf8(raw).unwrap().replacen(
                        "\"schema_version\":1",
                        "\"schema_version\":99",
                        1,
                    );
                    let next_trial = put(f.root(), text.as_bytes());
                    let mut doc: HistoryRoot =
                        serde_json::from_slice(&fs::read(blob(f.root(), root.as_blob())).unwrap())
                            .unwrap();
                    doc.trials.values_mut().next().unwrap().document = next_trial;
                    root = RootRef::new(put(f.root(), &serde_json::to_vec(&doc).unwrap()));
                }
            }
            assert!(HistoryReader::new(&f.storage)
                .read_snapshot(Some(&root))
                .root
                .is_some());
            assert!(HistoryReader::new(&f.storage)
                .verify_closure(&root)
                .is_err());
            f.next = with_root(&root);
            f.old = Some(f.next.clone());
            f.seed(0);
            let before = f.ledger();
            assert!(
                recover(f.root()).is_err(),
                "local closure defect must prevent all metadata writes"
            );
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn r03_raw_reader_exact_missing_limit_and_unsafe_paths() {
        let f = F::new(Some(V2), V2, 0);
        let p = f.path("raw-probe");
        fs::write(&p, b"not decoded").unwrap();
        assert_eq!(
            read_recovery_bytes(&p, 11).unwrap(),
            Some(b"not decoded".to_vec())
        );
        kind(read_recovery_bytes(&p, 10), HistoryErrorKind::LimitExceeded);
        fs::remove_file(&p).unwrap();
        assert_eq!(read_recovery_bytes(&p, 1).unwrap(), None);
        fs::create_dir(&p).unwrap();
        kind(read_recovery_bytes(&p, 1), HistoryErrorKind::UnsafePath);
        kind(
            read_recovery_bytes(&f.path("absent-parent/leaf"), 1),
            HistoryErrorKind::Io,
        );
        let parent = f.path("file-parent");
        fs::write(&parent, b"file").unwrap();
        kind(
            read_recovery_bytes(&parent.join("leaf"), 1),
            HistoryErrorKind::UnsafePath,
        );
    }
    #[test]
    fn r03_raw_reader_real_read_denial_and_unlocked_positive_control() {
        let f = F::new(Some(V2), V2, 0);
        let p = f.path(PENDING);
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&p)
            .unwrap();
        assert_eq!(fs::read(&p).unwrap_err().raw_os_error(), Some(32));
        eprintln!("R raw-reader probe: actual read denied 32");
        kind(read_recovery_bytes(&p, SMALL.intent), HistoryErrorKind::Io);
        drop(held);
        assert_eq!(
            read_recovery_bytes(&p, SMALL.intent).unwrap(),
            Some(f.raw.clone())
        );
    }
    #[test]
    fn r03_reparse_root_ancestor_leaf_and_archive_parent_are_preserved() {
        for mode in 0..4 {
            let mut f = F::new(Some(V2), V2, 0);
            let target = f.temp.root.join("target");
            fs::create_dir(&target).unwrap();
            fs::write(target.join("keep"), b"junction target preserved").unwrap();
            let (root, path) = match mode {
                0 => {
                    let link = f.temp.root.join("root-link");
                    let root = f.root().to_owned();
                    f.temp.junction(&link, &root);
                    (link.clone(), link.join(PENDING))
                }
                1 => {
                    let link = f.temp.root.join("ancestor-link");
                    let root = f.temp.root.clone();
                    f.temp.junction(&link, &root);
                    (link.join("host"), link.join("host").join(PENDING))
                }
                2 => {
                    let link = f.path("custom.tsv");
                    fs::remove_file(&link).unwrap();
                    f.temp.junction(&link, &target);
                    (f.root().to_owned(), link)
                }
                _ => {
                    let link = f.path("backups");
                    f.temp.junction(&link, &target);
                    (f.root().to_owned(), f.path(PENDING))
                }
            };
            let before = f.ledger();
            if mode < 3 {
                kind(
                    read_recovery_bytes(&path, SMALL.intent),
                    HistoryErrorKind::UnsafePath,
                );
            }
            assert!(recover(&root).is_err());
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn r04_intent_old_v1_backup_precedes_writes_from_every_partial_state() {
        for state in 0..4 {
            let f = F::new(Some(RAW_V1), V2, state);
            let before = f.ledger();
            let calls = Cell::new(0);
            let exact_at_write = Cell::new(true);
            let faults = Faults::new(None);
            f.io(
                SMALL,
                &faults,
                |p, b| {
                    calls.set(calls.get() + 1);
                    exact_at_write.set(
                        exact_at_write.get()
                            && fs::read(f.hash_path()).ok().as_deref() == Some(RAW_V1.as_bytes()),
                    );
                    atomic_write(p, b)
                },
                rename,
            )
            .unwrap();
            assert_eq!(calls.get(), 2);
            assert!(exact_at_write.get());
            assert_eq!(
                &faults.seen.borrow()[..2],
                &[CP::BeforeMigrationBackup, CP::AfterMigrationBackup]
            );
            f.done(before, true);
        }
    }
    #[test]
    fn r04_existing_exact_backup_is_preserved_and_conflicts_never_write() {
        for mode in 0..4 {
            let f = F::new(Some(RAW_V1), V2, 3);
            fs::create_dir(f.path("backups")).unwrap();
            let contents = match mode {
                0 => RAW_V1.to_string(),
                1 => format!("{RAW_V1} "),
                2 => "{broken".into(),
                _ => V2.replace("\"version\":2", "\"version\":99"),
            };
            fs::write(f.hash_path(), contents.as_bytes()).unwrap();
            let before = f.ledger();
            if mode == 0 {
                let readonly = OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .open(f.hash_path())
                    .unwrap();
                f.faults(&NoProgressFaults).unwrap();
                drop(readonly);
                f.done(before, true);
            } else {
                reject(&f, SMALL);
            }
        }
    }
    #[test]
    fn r04_migration_faults_keep_exact_boundary_and_retry() {
        for point in [CP::BeforeMigrationBackup, CP::AfterMigrationBackup] {
            let f = F::new(Some(RAW_V1), V2, 0);
            let mut expected = f.ledger();
            let faults = Faults::new(Some(point));
            assert!(f.faults(&faults).is_err());
            let want = if point == CP::BeforeMigrationBackup {
                vec![CP::BeforeMigrationBackup]
            } else {
                expected.insert("host/backups".into(), vec![]);
                expected.insert(
                    format!("host/backups/progress-v1-{HASH}.json").into(),
                    RAW_V1.as_bytes().to_vec(),
                );
                vec![CP::BeforeMigrationBackup, CP::AfterMigrationBackup]
            };
            assert_eq!(*faults.seen.borrow(), want);
            assert_eq!(f.ledger(), expected);
            writable();
            f.faults(&NoProgressFaults).unwrap();
            f.done(expected, true);
        }
    }
    #[test]
    fn r04_existing_backup_actual_read_denial_preserves_metadata_then_retries() {
        let f = F::new(Some(RAW_V1), V2, 0);
        fs::create_dir(f.path("backups")).unwrap();
        fs::write(f.hash_path(), RAW_V1).unwrap();
        let before = f.ledger();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(f.hash_path())
            .unwrap();
        assert_eq!(
            fs::read(f.hash_path()).unwrap_err().raw_os_error(),
            Some(32)
        );
        let writes = Cell::new(0);
        let result = f.io(
            SMALL,
            &NoProgressFaults,
            |p, b| {
                writes.set(writes.get() + 1);
                atomic_write(p, b)
            },
            rename,
        );
        drop(held);
        assert!(result.is_err());
        assert_eq!(writes.get(), 0);
        assert_eq!(f.ledger(), before);
        writable();
        f.faults(&NoProgressFaults).unwrap();
        f.done(before, true);
    }
    #[test]
    fn r05_no_write_ok_and_err_are_known_interruptions_then_retry() {
        for stage in 0..2 {
            for ok in [false, true] {
                let f = F::new(Some(&V2.replace("next raw", "old raw")), V2, 0);
                let mut expected = f.ledger();
                let calls = Cell::new(0);
                let archives = Cell::new(0);
                let result = f.io(
                    SMALL,
                    &NoProgressFaults,
                    |p, b| {
                        let n = calls.get();
                        calls.set(n + 1);
                        if n == stage {
                            if ok {
                                Ok(())
                            } else {
                                Err("injected no write".into())
                            }
                        } else {
                            atomic_write(p, b)
                        }
                    },
                    |_, _| {
                        archives.set(archives.get() + 1);
                        Ok(())
                    },
                );
                assert!(result.is_err());
                assert_eq!(calls.get(), stage + 1);
                assert_eq!(archives.get(), 0);
                if stage == 1 {
                    expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
                }
                assert_eq!(f.ledger(), expected);
                writable();
                assert!(f.storage.save(&Progress::default()).is_err());
                recover(f.root()).unwrap();
                f.done(expected, false);
            }
        }
    }
    #[test]
    fn r05_actual_atomic_then_err_and_equal_old_new_are_success() {
        for equal in [false, true] {
            let mut f = F::new(Some(V2), V2, 0);
            if equal {
                f.old_deck = Some(f.deck.clone());
            } else {
                f.old = Some(V2.replace("next raw", "old raw"));
            }
            f.seed(0);
            let before = f.ledger();
            let calls = Cell::new(0);
            f.io(
                SMALL,
                &NoProgressFaults,
                |p, b| {
                    calls.set(calls.get() + 1);
                    if !equal {
                        atomic_write(p, b)?;
                    }
                    Err("same ambiguous result".into())
                },
                rename,
            )
            .unwrap();
            assert_eq!(calls.get(), 2);
            f.done(before, false);
        }
    }
    #[test]
    fn r05_six_material_checkpoints_preserve_known_states_and_finish_confirmation() {
        let points = [
            CP::BeforeDeck,
            CP::AfterDeck,
            CP::BeforeMaterialProgress,
            CP::AfterMaterialProgress,
            CP::BeforeMaterialFinish,
            CP::AfterMaterialFinish,
        ];
        for (index, point) in points.into_iter().enumerate() {
            let f = F::new(Some(&V2.replace("next raw", "old raw")), V2, 0);
            let mut expected = f.ledger();
            let faults = Faults::new(Some(point));
            let result = f.faults(&faults);
            assert_eq!(*faults.seen.borrow(), points[..=index]);
            if index == 5 {
                assert!(result.is_ok());
                f.done(expected, false);
            } else {
                assert!(result.is_err());
                if index >= 1 {
                    expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
                }
                if index >= 3 {
                    expected.insert("host/progress.json".into(), f.next.as_bytes().to_vec());
                }
                assert_eq!(f.ledger(), expected);
                writable();
                recover(f.root()).unwrap();
                f.done(expected, false);
            }
        }
    }
    #[test]
    fn r06_archive_status_is_decided_by_exact_files_not_return_value() {
        for mode in 0..5 {
            let f = F::new(Some(V2), V2, 0);
            let before = f.ledger();
            let archive_path = RefCell::new(None);
            let calls = Cell::new(0);
            let result = f.io(SMALL, &NoProgressFaults, atomic_write, |from, to| {
                calls.set(calls.get() + 1);
                *archive_path.borrow_mut() = Some(to.to_owned());
                match mode {
                    0 => Ok(()),
                    1 => Err(std::io::Error::other("no rename")),
                    2 => {
                        fs::copy(from, to)?;
                        Err(std::io::Error::other("exact duplicate, pending retained"))
                    }
                    3 => {
                        fs::rename(from, to)?;
                        Err(std::io::Error::other("rename happened"))
                    }
                    _ => fs::rename(from, to),
                }
            });
            assert_eq!(calls.get(), 1);
            if mode >= 3 {
                assert!(result.is_ok());
                f.done(before, false);
            } else {
                assert!(result.is_err());
                let mut expected = before;
                expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
                expected.insert("host/progress.json".into(), f.next.as_bytes().to_vec());
                expected.insert("host/backups".into(), vec![]);
                if mode == 2 {
                    let path = archive_path.borrow().clone().unwrap();
                    expected.insert(
                        path.strip_prefix(&f.temp.root).unwrap().into(),
                        f.raw.clone(),
                    );
                }
                assert_eq!(f.ledger(), expected);
                writable();
                assert!(f.storage.save(&Progress::default()).is_err());
                recover(f.root()).unwrap();
                f.done(expected, false);
            }
        }
    }
    #[test]
    fn r06_real_archive_rename_denial_is_known_pending_and_unlock_control_succeeds() {
        let f = F::new(Some(V2), V2, 0);
        let before = f.ledger();
        let held = RefCell::new(None);
        let actual = RefCell::new(None);
        let target = RefCell::new(None);
        let calls = Cell::new(0);
        let result = f.io(SMALL, &NoProgressFaults, atomic_write, |from, to| {
            calls.set(calls.get() + 1);
            let handle = OpenOptions::new().read(true).share_mode(3).open(from)?;
            let r = fs::rename(from, to);
            *actual.borrow_mut() =
                Some((r.is_err(), r.as_ref().err().and_then(|e| e.raw_os_error())));
            *target.borrow_mut() = Some(to.to_owned());
            *held.borrow_mut() = Some(handle);
            r
        });
        drop(held.borrow_mut().take());
        assert_eq!(calls.get(), 1);
        assert!(result.is_err());
        let (failed, raw) = actual.borrow().unwrap();
        assert!(failed);
        eprintln!("R archive real rename denial raw={raw:?}");
        let mut expected = before;
        expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
        expected.insert("host/progress.json".into(), f.next.as_bytes().to_vec());
        expected.insert("host/backups".into(), vec![]);
        assert_eq!(f.ledger(), expected);
        writable();
        let to = target.borrow().clone().unwrap();
        fs::rename(f.path(PENDING), &to).expect("same rename after handle release");
        assert_eq!(fs::read(to).unwrap(), f.raw);
        f.done(expected, false);
    }
    #[test]
    fn r06_directory_creation_failure_before_archive_is_not_unknown() {
        struct Obstacle<'a>(&'a F);
        impl ProgressFaults for Obstacle<'_> {
            fn check(&self, p: CP) -> Result<(), ProgressFaultError> {
                if p == CP::BeforeMaterialFinish {
                    fs::write(self.0.path("backups"), b"fixture obstacle").unwrap();
                }
                Ok(())
            }
        }
        let f = F::new(Some(V2), V2, 0);
        let mut expected = f.ledger();
        let archives = Cell::new(0);
        let result = f.io(SMALL, &Obstacle(&f), atomic_write, |_, _| {
            archives.set(archives.get() + 1);
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(archives.get(), 0);
        expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
        expected.insert("host/progress.json".into(), f.next.as_bytes().to_vec());
        expected.insert("host/backups".into(), b"fixture obstacle".to_vec());
        assert_eq!(f.ledger(), expected);
        writable();
        fs::remove_file(f.path("backups")).unwrap();
        expected.remove(Path::new("host/backups"));
        recover(f.root()).unwrap();
        f.done(expected, false);
    }

    #[test]
    fn r04_pending_identity_is_rechecked_after_migration_callback_before_metadata() {
        child_cases("commit::i2_material_recovery_tests::r04_pending_identity_is_rechecked_after_migration_callback_before_metadata",1,|_|{
            struct Changed<'a>{f:&'a F,seen:RefCell<Vec<CP>>}
            impl ProgressFaults for Changed<'_>{fn check(&self,p:CP)->Result<(),ProgressFaultError>{self.seen.borrow_mut().push(p);if p==CP::AfterMigrationBackup{fs::write(self.f.path(PENDING),b"external changed pending").unwrap();}Ok(())}}
            let mut f=F::new(Some(RAW_V1),V2,0);let a=f.root_a();let other=Storage::at(f.temp.root.join("other")).unwrap();let mut expected=f.ledger();let faults=Changed{f:&f,seen:RefCell::new(vec![])};let writes=Cell::new(0);let archive=Cell::new(0);let result=f.io(SMALL,&faults,|_,_|{writes.set(writes.get()+1);Ok(())},|_,_|{archive.set(archive.get()+1);Ok(())});assert!(result.is_err());assert_eq!(*faults.seen.borrow(),[CP::BeforeMigrationBackup,CP::AfterMigrationBackup]);assert_eq!((writes.get(),archive.get()),(0,0));expected.insert("host/backups".into(),vec![]);expected.insert(format!("host/backups/progress-v1-{HASH}.json").into(),RAW_V1.as_bytes().to_vec());expected.insert("host/pending-material-commit.json".into(),b"external changed pending".to_vec());assert_eq!(f.ledger(),expected);drop(faults);blocked(&mut f,&other,&a);
        });
    }
    fn blocked(f: &mut F, other: &Storage, a: &RootRef) {
        assert_eq!(
            acquire_progress_write().map(drop),
            Err(ProgressGateError::WriteBlocked)
        );
        let before = f.ledger();
        let p = Progress::default();
        assert!(f.storage.save(&p).is_err());
        assert!(f.storage.load().is_err());
        assert!(save(f.root(), model::BUILTIN_DECK, &p).is_err());
        assert!(recover(f.root()).is_err());
        assert!(other.save(&p).is_err());
        assert!(save(&other.dir, model::BUILTIN_DECK, &p).is_err());
        let pure = f
            .storage
            .read_progress()
            .expect("metadata progress remains a valid pure-read fixture");
        assert_eq!(pure.version, 2);
        let request = FrozenHistoryRequest::new(
            HistoryMutation::SetTrash {
                target: TrialTarget {
                    id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
                    expected: a.as_blob().clone(),
                },
                trashed_at: None,
            },
            vec![],
        )
        .unwrap();
        let mut writer =
            HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1()).unwrap();
        let result = writer.prepare(
            OperationId::parse("22222222-2222-4222-8222-000000000001").unwrap(),
            None,
            &request,
        );
        assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
        drop(writer);
        assert_eq!(f.ledger(), before);
        assert!(HistoryReader::new(&f.storage)
            .read_snapshot(Some(a))
            .root
            .is_some());
    }
    #[test]
    fn r05_unknown_write_states_block_all_writers_without_rollback() {
        child_cases("commit::i2_material_recovery_tests::r05_unknown_write_states_block_all_writers_without_rollback",6,|mode|{
            let mut f=F::new(Some(V2),V2,0);let a=f.root_a();f.next=with_root(&a);f.old=Some(f.next.clone());f.seed(0);let other=Storage::at(f.temp.root.join("other")).unwrap();let mut expected=f.ledger();let calls=Cell::new(0);let archives=Cell::new(0);let held=RefCell::<Option<File>>::new(None);let actual_failed=Cell::new(false);let denied=Cell::new(None);
            let result=f.io(SMALL,&NoProgressFaults,|path,bytes|{
                calls.set(calls.get()+1);
                if mode==2{let file=OpenOptions::new().read(true).share_mode(0).open(path).map_err(|e|e.to_string())?;denied.set(fs::read(path).err().and_then(|e|e.raw_os_error()));let actual=atomic_write(path,bytes);actual_failed.set(actual.is_err());*held.borrow_mut()=Some(file);return actual;}
                atomic_write(path,bytes)?;match mode{0=>fs::write(path,b"external third deck").map_err(|e|e.to_string())?,1=>fs::remove_file(path).map_err(|e|e.to_string())?,3=>fs::remove_file(f.path(PENDING)).map_err(|e|e.to_string())?,4=>fs::write(f.path(PENDING),b"external third pending").map_err(|e|e.to_string())?,5=>{let file=OpenOptions::new().read(true).share_mode(0).open(f.path(PENDING)).map_err(|e|e.to_string())?;denied.set(fs::read(f.path(PENDING)).err().and_then(|e|e.raw_os_error()));*held.borrow_mut()=Some(file);},_=>{}}Err("ambiguous write result".into())
            },|_,_|{archives.set(archives.get()+1);Ok(())});drop(held.borrow_mut().take());assert_eq!(calls.get(),1,"write must actually execute before classifying unknown state");assert_eq!(archives.get(),0);assert!(result.is_err());
            match mode{0=>{expected.insert("host/custom.tsv".into(),b"external third deck".to_vec());},1=>{expected.remove(Path::new("host/custom.tsv"));},2=>{assert!(actual_failed.get());assert_eq!(denied.get(),Some(32));eprintln!("R write actual atomic Err and read denial 32 reached");},_=>{expected.insert("host/custom.tsv".into(),f.deck.as_bytes().to_vec());}}
            if mode==3{expected.remove(Path::new("host/pending-material-commit.json"));}if mode==4{expected.insert("host/pending-material-commit.json".into(),b"external third pending".to_vec());}if mode==5{assert_eq!(denied.get(),Some(32));eprintln!("R pending postwrite actual read denial 32 reached");}assert_eq!(f.ledger(),expected);blocked(&mut f,&other,&a);
            if mode==2{atomic_write(&f.path("custom.tsv"),f.deck.as_bytes()).expect("same path atomic write succeeds after release, fixture-only control");expected.insert("host/custom.tsv".into(),f.deck.as_bytes().to_vec());assert_eq!(f.ledger(),expected);eprintln!("R write unlocked same-path atomic control succeeded after product ledger/block checks");}
        });
    }
    #[test]
    fn r06_unknown_finish_states_block_without_recreating_pending_or_archive() {
        child_cases("commit::i2_material_recovery_tests::r06_unknown_finish_states_block_without_recreating_pending_or_archive",5,|mode|{
            let mut f=F::new(Some(V2),V2,0);let a=f.root_a();f.next=with_root(&a);f.old=Some(f.next.clone());f.seed(0);let other=Storage::at(f.temp.root.join("other")).unwrap();let mut expected=f.ledger();let writes=Cell::new(0);let archives=Cell::new(0);let held=RefCell::<Option<File>>::new(None);let denied=Cell::new(None);let archive_path=RefCell::new(None);
            let result=f.io(SMALL,&NoProgressFaults,|path,bytes|{writes.set(writes.get()+1);atomic_write(path,bytes)},|from,to|{archives.set(archives.get()+1);*archive_path.borrow_mut()=Some(to.to_owned());match mode{0=>{fs::rename(from,to)?;fs::write(to,b"third archive")?;},1=>fs::remove_file(from)?,2=>{fs::rename(from,to)?;let file=OpenOptions::new().read(true).share_mode(0).open(to)?;denied.set(fs::read(to).err().and_then(|e|e.raw_os_error()));*held.borrow_mut()=Some(file);},3=>{fs::rename(from,to)?;fs::write(f.path("progress.json"),V2)?;},_=>fs::write(to,b"third archive while pending retained")?}Err(std::io::Error::other("ambiguous archive outcome"))});drop(held.borrow_mut().take());assert_eq!((writes.get(),archives.get()),(2,1),"both writes and archive must be reached");assert!(result.is_err());expected.insert("host/custom.tsv".into(),f.deck.as_bytes().to_vec());expected.insert("host/progress.json".into(),f.next.as_bytes().to_vec());expected.insert("host/backups".into(),vec![]);let archive=archive_path.borrow().clone().unwrap();if mode!=4{expected.remove(Path::new("host/pending-material-commit.json"));}match mode{0=>{expected.insert(archive.strip_prefix(&f.temp.root).unwrap().into(),b"third archive".to_vec());},1=>{},2|3=>{expected.insert(archive.strip_prefix(&f.temp.root).unwrap().into(),f.raw.clone());},_=>{expected.insert(archive.strip_prefix(&f.temp.root).unwrap().into(),b"third archive while pending retained".to_vec());}}
            if mode==3{expected.insert("host/progress.json".into(),V2.as_bytes().to_vec());}if mode==2{assert_eq!(denied.get(),Some(32));assert_eq!(fs::read(&archive).unwrap(),f.raw);eprintln!("R archive postrename read denied 32; same file readable after release");}assert_eq!(f.ledger(),expected);blocked(&mut f,&other,&a);
        });
    }
}

#[cfg(all(test, windows))]
mod i2_material_save_tests {
    use super::*;
    use crate::{
        audio_history::*,
        model,
        store::{
            i2_gate_tests::{tree, Temp},
            i2_migration_tests::{HASH, RAW_V1},
            NoProgressFaults, ProgressCheckpoint as CP, ProgressFaultError, ProgressFaults,
            ProgressHistoryHead, Storage,
        },
    };
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        fs::{File, OpenOptions},
        io::Write,
        os::windows::fs::OpenOptionsExt,
        path::PathBuf,
        process::{Command, Stdio},
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc, Arc, Barrier,
        },
        time::{Duration, Instant},
    };
    const V2: &str = r#"{ "version":2,"audio_history":null,"settings":{"topic":"old raw"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{} }"#;
    const A: &[u8] =
        br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
    const AH: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
    const SMALL: Limits = Limits {
        deck: 100_000,
        progress: 100_000,
        intent: 500_000,
    };
    type Ledger = BTreeMap<PathBuf, Vec<u8>>;
    fn points(migrate: bool) -> Vec<CP> {
        let mut p = vec![];
        if migrate {
            p.extend([CP::BeforeMigrationBackup, CP::AfterMigrationBackup]);
        }
        p.extend([CP::BeforeMaterialIntent, CP::AfterMaterialIntent]);
        if migrate {
            p.extend([CP::BeforeMigrationBackup, CP::AfterMigrationBackup]);
        }
        p.extend([
            CP::BeforeDeck,
            CP::AfterDeck,
            CP::BeforeMaterialProgress,
            CP::AfterMaterialProgress,
            CP::BeforeMaterialFinish,
            CP::AfterMaterialFinish,
        ]);
        p
    }
    struct Faults {
        seen: RefCell<Vec<CP>>,
        fail: Option<(CP, usize)>,
    }
    impl Faults {
        fn new(fail: Option<(CP, usize)>) -> Self {
            Self {
                seen: RefCell::new(vec![]),
                fail,
            }
        }
    }
    impl ProgressFaults for Faults {
        fn check(&self, p: CP) -> Result<(), ProgressFaultError> {
            let mut seen = self.seen.borrow_mut();
            seen.push(p);
            if self.fail == Some((p, seen.iter().filter(|q| **q == p).count())) {
                Err(ProgressFaultError::Io)
            } else {
                Ok(())
            }
        }
    }
    struct Hook<F: Fn(CP) -> Result<(), ProgressFaultError>>(F);
    impl<F: Fn(CP) -> Result<(), ProgressFaultError>> ProgressFaults for Hook<F> {
        fn check(&self, p: CP) -> Result<(), ProgressFaultError> {
            (self.0)(p)
        }
    }
    struct Fixture {
        storage: Storage,
        temp: Temp,
        old: Option<String>,
        old_deck: String,
        deck: String,
        candidate: Progress,
        root: Option<RootRef>,
    }
    impl Fixture {
        fn new(old: Option<&str>) -> Self {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            let mut candidate: Progress = serde_json::from_str(V2).unwrap();
            candidate.settings.topic = "candidate quote \" and backslash \\ 日本語\n".into();
            candidate.validate().unwrap();
            let f = Self {
                storage,
                temp,
                old: old.map(str::to_owned),
                old_deck: "unparseable old deck \"exact\"\n".into(),
                deck: format!("{}\n", model::BUILTIN_DECK),
                candidate,
                root: None,
            };
            fs::write(f.path("custom.tsv"), &f.old_deck).unwrap();
            if let Some(old) = &f.old {
                fs::write(f.path("progress.json"), old).unwrap();
            }
            fs::write(f.path("keep.wav"), b"synthetic media preserved").unwrap();
            fs::write(f.temp.root.join("neighbor"), b"neighbor preserved").unwrap();
            f
        }
        fn path(&self, name: &str) -> PathBuf {
            self.storage.dir.join(name)
        }
        fn ledger(&self) -> Ledger {
            tree(&self.temp.root)
        }
        fn expected(&self) -> Progress {
            let mut p = self.candidate.clone();
            p.audio_history = self.root.clone();
            p
        }
        fn next(&self) -> Vec<u8> {
            serde_json::to_vec_pretty(&self.expected()).unwrap()
        }
        fn intent(&self) -> Vec<u8> {
            let q = |s: &str| serde_json::to_string(s).unwrap();
            format!(
                "{{\"old_deck\":{},\"old_progress\":{},\"deck\":{},\"progress\":{}}}",
                q(&self.old_deck),
                self.old.as_deref().map(q).unwrap_or_else(|| "null".into()),
                q(&self.deck),
                q(std::str::from_utf8(&self.next()).unwrap())
            )
            .into_bytes()
        }
        fn limits(&self) -> Limits {
            Limits {
                deck: self.deck.len().max(self.old_deck.len()) as u64,
                progress: self
                    .next()
                    .len()
                    .max(self.old.as_ref().map_or(0, String::len)) as u64,
                intent: self.intent().len() as u64,
            }
        }
        fn call<P: FnOnce(&Path, &[u8]) -> Result<(), String>>(
            &self,
            limits: Limits,
            faults: &dyn ProgressFaults,
            publish: P,
        ) -> Result<Progress, String> {
            save_progress_with_intent_publisher(
                &self.storage.dir,
                &self.deck,
                &self.candidate,
                limits,
                faults,
                publish,
            )
        }
        fn hash(&self) -> PathBuf {
            self.path(&format!("backups/progress-v1-{HASH}.json"))
        }
        fn with_migration(&self, mut ledger: Ledger) -> Ledger {
            if self.old.as_deref() == Some(RAW_V1) {
                ledger.insert("host/backups".into(), vec![]);
                ledger.insert(
                    format!("host/backups/progress-v1-{HASH}.json").into(),
                    RAW_V1.as_bytes().to_vec(),
                );
            }
            ledger
        }
        fn complete(&self, mut expected: Ledger, raw: &[u8]) {
            expected = self.with_migration(expected);
            expected.remove(Path::new("host/pending-material-commit.json"));
            expected.insert("host/custom.tsv".into(), self.deck.as_bytes().to_vec());
            expected.insert("host/progress.json".into(), self.next());
            expected.insert("host/backups".into(), vec![]);
            let archives: Vec<_> = fs::read_dir(self.path("backups"))
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
            expected.insert(
                archives[0].strip_prefix(&self.temp.root).unwrap().into(),
                raw.to_vec(),
            );
            assert!(
                self.ledger() == expected,
                "exact metadata/media/backup ledger; no daily backup or temp"
            );
        }
        fn set_root(&mut self, r: RootRef) {
            let raw = V2.replace(
                "\"audio_history\":null",
                &format!(
                    r#""audio_history":{{"sha256":"{}","bytes":{}}}"#,
                    r.as_blob().sha256.as_str(),
                    r.as_blob().bytes
                ),
            );
            self.old = Some(raw.clone());
            self.root = Some(r);
            fs::write(self.path("progress.json"), raw).unwrap();
        }
        fn install_a(&mut self) -> RootRef {
            fs::create_dir_all(self.path("audio-history/assets")).unwrap();
            let r = RootRef::new(BlobRef {
                sha256: Sha256::parse(AH).unwrap(),
                bytes: 82,
            });
            fs::write(blob(&self.storage.dir, r.as_blob()), A).unwrap();
            let c = HistoryReader::new(&self.storage)
                .verify_closure(&r)
                .unwrap();
            assert_eq!(c.objects, vec![r.as_blob().clone()]);
            assert_eq!(c.total_bytes, 82);
            r
        }
        fn closure3(&mut self) -> (RootRef, BlobRef, BlobRef) {
            fs::create_dir_all(self.path("audio-history/assets")).unwrap();
            let original = put(&self.storage.dir, b"independent T original");
            let trial = TrialRevision {
                schema_version: 1,
                id: trial_id(),
                revision: 1,
                previous: None,
                created_at: RecordedAt {
                    utc_millis: 1_700_000_000_000,
                    offset_minutes: 540,
                },
                input: input(original.clone()),
                claim: PracticeClaim::Reference,
                match_evidence: vec![],
                analysis: BTreeMap::new(),
                advice: BTreeMap::new(),
                reflections: vec![],
                trashed_at: None,
            };
            trial.validate().unwrap();
            let tr = put(&self.storage.dir, &serde_json::to_vec(&trial).unwrap());
            let root = HistoryRoot {
                schema_version: 1,
                revision: 1,
                next_save_sequence: 2,
                trials: BTreeMap::from([(
                    trial.id,
                    TrialHead {
                        document: tr.clone(),
                        admitted_sequence: 1,
                    },
                )]),
                receipts: BTreeMap::new(),
            };
            root.validate().unwrap();
            let rr = RootRef::new(put(&self.storage.dir, &serde_json::to_vec(&root).unwrap()));
            let mut expected = vec![rr.as_blob().clone(), tr.clone(), original.clone()];
            expected.sort_by(|a, b| a.sha256.cmp(&b.sha256));
            let c = HistoryReader::new(&self.storage)
                .verify_closure(&rr)
                .unwrap();
            assert_eq!(c.objects, expected);
            assert_eq!(c.total_bytes, expected.iter().map(|b| b.bytes).sum::<u64>());
            (rr, tr, original)
        }
    }
    fn trial_id() -> TrialId {
        TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap()
    }
    fn digest(bytes: &[u8]) -> Sha256 {
        Sha256::parse(&crate::assets::sha256(bytes).unwrap()).unwrap()
    }
    fn blob(root: &Path, b: &BlobRef) -> PathBuf {
        root.join(format!("audio-history/assets/{}.bin", b.sha256.as_str()))
    }
    fn put(root: &Path, bytes: &[u8]) -> BlobRef {
        let b = BlobRef {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
        };
        fs::write(blob(root, &b), bytes).unwrap();
        b
    }
    fn input(original: BlobRef) -> FrozenInput {
        FrozenInput {
            audio: AudioSnapshot {
                original,
                format: AudioFormat::Wav,
                rate: 8000,
                channels: 1,
                frames: 1,
                origin: Origin::Imported,
                warnings: vec![],
            },
            reference: ReferenceSnapshot {
                text: "Synthetic reference".into(),
                sha256: digest(b"Synthetic reference"),
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
        }
    }
    fn request() -> FrozenHistoryRequest {
        let bytes = b"T synthetic original";
        FrozenHistoryRequest::new(
            HistoryMutation::CreateTrial {
                id: trial_id(),
                created_at: RecordedAt {
                    utc_millis: 1_700_000_000_000,
                    offset_minutes: 540,
                },
                input: input(BlobRef {
                    sha256: digest(bytes),
                    bytes: bytes.len() as u64,
                }),
                claim: PracticeClaim::Reference,
                duplicate: None,
            },
            vec![Arc::from(bytes.as_slice())],
        )
        .unwrap()
    }
    fn operation() -> OperationId {
        OperationId::parse("22222222-2222-4222-8222-000000000001").unwrap()
    }
    fn writable() {
        assert!(matches!(
            acquire_progress_write().map(|g| g.ensure_writable()),
            Ok(Ok(()))
        ));
    }
    fn reject(f: &Fixture, limits: Limits) {
        let before = f.ledger();
        let input = serde_json::to_vec(&f.candidate).unwrap();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        assert!(f
            .call(limits, &faults, |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            })
            .is_err());
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert!(
            f.ledger() == before,
            "preflight must preserve all bytes/directories"
        );
        assert_eq!(serde_json::to_vec(&f.candidate).unwrap(), input);
        writable();
    }
    fn isolated(case: &str, count: usize, body: impl Fn(usize)) {
        const ENV: &str = "WW5_I2_T_CHILD";
        if std::env::var(ENV).as_deref() == Ok(case) {
            let mode = std::env::var("WW5_I2_T_MODE").unwrap().parse().unwrap();
            body(mode);
            println!("WW5_T_CHILD_OK:{case}:{mode}");
            return;
        }
        for mode in 0..count {
            let temp = Temp::new();
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([case, "--exact", "--nocapture", "--test-threads=1"])
                .env(ENV, case)
                .env("WW5_I2_T_MODE", mode.to_string())
                .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut timeout = false;
            loop {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    timeout = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            let out = child.wait_with_output().unwrap();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(!timeout, "bounded child killed/reaped\n{stdout}\n{stderr}");
            assert!(out.status.success(), "scenario {mode}\n{stdout}\n{stderr}");
            assert!(
                stdout.contains("running 1 test")
                    && stdout.contains("1 passed; 0 failed")
                    && stdout.contains(&format!("WW5_T_CHILD_OK:{case}:{mode}"))
            );
            for line in stderr.lines().filter(|l| l.starts_with("T ")) {
                eprintln!("{line}");
            }
        }
    }
    #[test]
    fn t02_preflight_rejections_preserve_all_bytes_before_migration() {
        for mode in 0..10 {
            let mut f = Fixture::new(Some(V2));
            match mode {
                0 => fs::write(f.path(PENDING), b"existing pending").unwrap(),
                1 => fs::create_dir(f.path(PENDING)).unwrap(),
                2 => fs::write(f.path("progress.json"), b"{broken").unwrap(),
                3 => fs::write(
                    f.path("progress.json"),
                    V2.replace("\"version\":2", "\"version\":99"),
                )
                .unwrap(),
                4 => f.candidate.settings.minutes = 0,
                5 => f.candidate.version = 1,
                6 => f.deck = "not a new deck".into(),
                7 => fs::write(f.path("custom.tsv"), [0xff, 0xfe]).unwrap(),
                8 => fs::write(
                    f.path("progress.json"),
                    V2.replace("\"version\":2", "\"version\":2,\"version\":2"),
                )
                .unwrap(),
                _ => fs::write(
                    f.path("progress.json"),
                    V2.replace(
                        "\"audio_history\":null",
                        "\"audio_history\":null,\"audio_history\":null",
                    ),
                )
                .unwrap(),
            }
            reject(&f, SMALL);
        }
    }
    #[test]
    fn t02_exact_and_one_byte_over_old_new_and_escaped_intent_limits() {
        for field in 0..5 {
            let mut f = Fixture::new(Some(V2));
            match field {
                0 => {
                    f.old_deck = "x".repeat(f.deck.len() + 1);
                    fs::write(f.path("custom.tsv"), &f.old_deck).unwrap();
                }
                1 => {
                    let old = format!("{}{}", V2, " ".repeat(f.next().len() + 1));
                    f.old = Some(old.clone());
                    fs::write(f.path("progress.json"), old).unwrap();
                }
                2 => f.deck.push('\n'),
                3 => f.candidate.settings.topic.push_str("escaped \"\\\n extra"),
                _ => {}
            }
            let exact = f.limits();
            let mut smaller = exact;
            match field {
                0 | 2 => smaller.deck -= 1,
                1 | 3 => smaller.progress -= 1,
                _ => smaller.intent -= 1,
            }
            reject(&f, smaller);
            let before = f.ledger();
            let saved = save_progress_with_limits_and_faults(
                &f.storage.dir,
                &f.deck,
                &f.candidate,
                exact,
                &NoProgressFaults,
            )
            .unwrap();
            assert_eq!(serde_json::to_vec_pretty(&saved).unwrap(), f.next());
            f.complete(before, &f.intent());
        }
    }
    #[test]
    fn t02_local_closure_defects_are_rejected_before_pending() {
        for mode in 0..3 {
            let mut f = Fixture::new(Some(V2));
            let (mut root, trial, original) = f.closure3();
            match mode {
                0 => fs::remove_file(blob(&f.storage.dir, &original)).unwrap(),
                1 => fs::write(blob(&f.storage.dir, &original), b"damaged T original").unwrap(),
                _ => {
                    let raw = String::from_utf8(fs::read(blob(&f.storage.dir, &trial)).unwrap())
                        .unwrap()
                        .replacen("\"schema_version\":1", "\"schema_version\":99", 1);
                    let tr = put(&f.storage.dir, raw.as_bytes());
                    let mut doc: HistoryRoot = serde_json::from_slice(
                        &fs::read(blob(&f.storage.dir, root.as_blob())).unwrap(),
                    )
                    .unwrap();
                    doc.trials.values_mut().next().unwrap().document = tr;
                    root = RootRef::new(put(&f.storage.dir, &serde_json::to_vec(&doc).unwrap()));
                }
            }
            assert!(HistoryReader::new(&f.storage)
                .read_snapshot(Some(&root))
                .root
                .is_some());
            assert!(HistoryReader::new(&f.storage)
                .verify_closure(&root)
                .is_err());
            f.set_root(root);
            reject(&f, SMALL);
        }
    }
    #[test]
    fn t02_strict_reparse_root_pending_and_metadata_preserve_targets() {
        for mode in 0..4 {
            let mut f = Fixture::new(Some(V2));
            let target = f.temp.root.join("target");
            fs::create_dir(&target).unwrap();
            fs::write(target.join("keep"), b"external fixture target").unwrap();
            let path = match mode {
                0 => f.temp.root.join("root-alias"),
                1 => f.path(PENDING),
                2 => f.path("progress.json"),
                _ => f.path("custom.tsv"),
            };
            if mode >= 2 {
                fs::remove_file(&path).unwrap();
            }
            let destination = if mode == 0 {
                f.storage.dir.clone()
            } else {
                target
            };
            f.temp.junction(&path, &destination);
            let before = f.ledger();
            let root = if mode == 0 {
                path
            } else {
                f.storage.dir.clone()
            };
            assert!(save_progress(&root, &f.deck, &f.candidate).is_err());
            assert!(f.ledger() == before);
            writable();
        }
    }
    #[test]
    fn t02_real_old_read_denial_and_unlocked_success_control() {
        for name in ["progress.json", "custom.tsv"] {
            let f = Fixture::new(Some(V2));
            let before = f.ledger();
            let held = OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(f.path(name))
                .unwrap();
            assert_eq!(fs::read(f.path(name)).unwrap_err().raw_os_error(), Some(32));
            eprintln!("T old source read probe {name}: sharing violation 32");
            let calls = Cell::new(0);
            let r = f.call(SMALL, &NoProgressFaults, |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            });
            drop(held);
            assert!(r.is_err());
            assert_eq!(calls.get(), 0);
            assert!(f.ledger() == before);
            writable();
            let saved = save_progress(&f.storage.dir, &f.deck, &f.candidate).unwrap();
            assert_eq!(serde_json::to_vec_pretty(&saved).unwrap(), f.next());
            f.complete(before, &f.intent());
        }
    }
    #[test]
    fn t03_exact_old_intent_and_twelve_or_eight_checkpoint_sequences() {
        for old in [Some(RAW_V1), Some(V2), None] {
            let f = Fixture::new(old);
            let before = f.ledger();
            let raw = f.intent();
            let calls = Cell::new(0);
            let published = RefCell::new(None);
            let migration_seen = Cell::new(false);
            let faults = Faults::new(None);
            let result = f
                .call(SMALL, &faults, |p, b| {
                    calls.set(calls.get() + 1);
                    *published.borrow_mut() = Some((p.to_owned(), b.to_vec()));
                    migration_seen.set(if old == Some(RAW_V1) {
                        fs::read(f.hash()).ok().as_deref() == Some(RAW_V1.as_bytes())
                    } else {
                        !f.hash().exists()
                    });
                    atomic_write(p, b)
                })
                .unwrap();
            assert_eq!(calls.get(), 1);
            assert_eq!(*published.borrow(), Some((f.path(PENDING), raw.clone())));
            assert!(migration_seen.get());
            assert_eq!(*faults.seen.borrow(), points(old == Some(RAW_V1)));
            assert_eq!(serde_json::to_vec_pretty(&result).unwrap(), f.next());
            f.complete(before, &raw);
        }
    }
    #[test]
    fn t03_migration_and_before_intent_faults_preserve_exact_stage() {
        for point in [
            CP::BeforeMigrationBackup,
            CP::AfterMigrationBackup,
            CP::BeforeMaterialIntent,
        ] {
            let f = Fixture::new(Some(RAW_V1));
            let before = f.ledger();
            let faults = Faults::new(Some((point, 1)));
            let calls = Cell::new(0);
            assert!(f
                .call(SMALL, &faults, |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(())
                })
                .is_err());
            assert_eq!(calls.get(), 0);
            let all = points(true);
            let end = all.iter().position(|p| *p == point).unwrap();
            assert_eq!(*faults.seen.borrow(), all[..=end]);
            let expected = if point == CP::BeforeMigrationBackup {
                before
            } else {
                f.with_migration(before)
            };
            assert!(f.ledger() == expected);
            writable();
        }
    }
    #[test]
    fn t03_before_publish_external_change_is_not_rebased_or_process_blocked() {
        for mode in 0..3 {
            let f = Fixture::new(Some(RAW_V1));
            let mut expected = f.with_migration(f.ledger());
            let seen = RefCell::new(vec![]);
            let faults = Hook(|p| {
                seen.borrow_mut().push(p);
                if p == CP::BeforeMaterialIntent {
                    match mode {
                        0 => fs::write(f.path("custom.tsv"), b"external deck").unwrap(),
                        1 => fs::write(f.path("progress.json"), V2).unwrap(),
                        _ => fs::write(f.path(PENDING), b"external pending").unwrap(),
                    }
                }
                Ok(())
            });
            let calls = Cell::new(0);
            assert!(f
                .call(SMALL, &faults, |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(())
                })
                .is_err());
            assert_eq!(calls.get(), 0);
            assert_eq!(*seen.borrow(), points(true)[..3]);
            match mode {
                0 => {
                    expected.insert("host/custom.tsv".into(), b"external deck".to_vec());
                }
                1 => {
                    expected.insert("host/progress.json".into(), V2.as_bytes().to_vec());
                }
                _ => {
                    expected.insert(
                        "host/pending-material-commit.json".into(),
                        b"external pending".to_vec(),
                    );
                }
            }
            assert!(f.ledger() == expected);
            writable();
        }
    }
    #[test]
    fn t04_no_intent_write_ok_or_err_is_unsaved_not_success() {
        for old in [Some(RAW_V1), Some(V2), None] {
            for publisher_ok in [true, false] {
                let f = Fixture::new(old);
                let expected = f.with_migration(f.ledger());
                let calls = Cell::new(0);
                let faults = Faults::new(None);
                assert!(f
                    .call(SMALL, &faults, |_, _| {
                        calls.set(calls.get() + 1);
                        if publisher_ok {
                            Ok(())
                        } else {
                            Err("same ambiguous publisher result".into())
                        }
                    })
                    .is_err());
                assert_eq!(calls.get(), 1);
                assert_eq!(
                    *faults.seen.borrow(),
                    points(old == Some(RAW_V1))[..if old == Some(RAW_V1) { 4 } else { 2 }]
                );
                assert!(f.ledger() == expected);
                writable();
            }
        }
    }
    #[test]
    fn t04_actual_intent_atomic_then_err_recovers_and_returns_actual_progress() {
        let mut f = Fixture::new(Some(V2));
        let a = f.install_a();
        f.set_root(a);
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let saved = f
            .call(SMALL, &faults, |p, b| {
                calls.set(calls.get() + 1);
                atomic_write(p, b)?;
                Err("same ambiguous publisher result".into())
            })
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(*faults.seen.borrow(), points(false));
        assert_eq!(serde_json::to_vec_pretty(&saved).unwrap(), f.next());
        f.complete(before, &f.intent());
    }
    #[test]
    fn t04_after_intent_fault_keeps_exact_pending_and_old_metadata_for_recovery() {
        for old in [Some(RAW_V1), Some(V2), None] {
            let f = Fixture::new(old);
            let mut expected = f.with_migration(f.ledger());
            let faults = Faults::new(Some((CP::AfterMaterialIntent, 1)));
            let calls = Cell::new(0);
            assert!(f
                .call(SMALL, &faults, |p, b| {
                    calls.set(calls.get() + 1);
                    atomic_write(p, b)
                })
                .is_err());
            assert_eq!(calls.get(), 1);
            assert_eq!(
                *faults.seen.borrow(),
                points(old == Some(RAW_V1))[..if old == Some(RAW_V1) { 4 } else { 2 }]
            );
            expected.insert("host/pending-material-commit.json".into(), f.intent());
            assert!(f.ledger() == expected);
            writable();
            recover(&f.storage.dir).unwrap();
            f.complete(expected, &f.intent());
        }
    }
    #[test]
    fn t05_r_second_migration_fault_occurrence_keeps_pending_and_verified_backup() {
        for point in [CP::BeforeMigrationBackup, CP::AfterMigrationBackup] {
            let f = Fixture::new(Some(RAW_V1));
            let mut expected = f.with_migration(f.ledger());
            let faults = Faults::new(Some((point, 2)));
            assert!(save_progress_with_limits_and_faults(
                &f.storage.dir,
                &f.deck,
                &f.candidate,
                SMALL,
                &faults
            )
            .is_err());
            let end = if point == CP::BeforeMigrationBackup {
                5
            } else {
                6
            };
            assert_eq!(*faults.seen.borrow(), points(true)[..end]);
            expected.insert("host/pending-material-commit.json".into(), f.intent());
            assert!(f.ledger() == expected);
            writable();
            recover(&f.storage.dir).unwrap();
            f.complete(expected, &f.intent());
        }
    }
    #[test]
    fn t05_full_save_propagates_all_r_checkpoints_and_after_finish_returns_success() {
        for (index, point) in [
            CP::BeforeDeck,
            CP::AfterDeck,
            CP::BeforeMaterialProgress,
            CP::AfterMaterialProgress,
            CP::BeforeMaterialFinish,
            CP::AfterMaterialFinish,
        ]
        .into_iter()
        .enumerate()
        {
            let f = Fixture::new(Some(V2));
            let mut expected = f.ledger();
            let faults = Faults::new(Some((point, 1)));
            let result = save_progress_with_limits_and_faults(
                &f.storage.dir,
                &f.deck,
                &f.candidate,
                SMALL,
                &faults,
            );
            assert_eq!(*faults.seen.borrow(), points(false)[..index + 3]);
            if index == 5 {
                assert_eq!(
                    serde_json::to_vec_pretty(&result.unwrap()).unwrap(),
                    f.next()
                );
                f.complete(expected, &f.intent());
            } else {
                assert!(result.is_err());
                expected.insert("host/pending-material-commit.json".into(), f.intent());
                if index >= 1 {
                    expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
                }
                if index >= 3 {
                    expected.insert("host/progress.json".into(), f.next());
                }
                assert!(f.ledger() == expected);
                writable();
                recover(&f.storage.dir).unwrap();
                f.complete(expected, &f.intent());
            }
        }
    }

    fn blocked(f: &mut Fixture, other: &Storage, a: &RootRef) {
        assert_eq!(
            acquire_progress_write().map(drop),
            Err(ProgressGateError::WriteBlocked)
        );
        let before = f.ledger();
        assert!(f.storage.save(&f.candidate).is_err());
        assert!(f.storage.load().is_err());
        assert!(save(&f.storage.dir, &f.deck, &f.candidate).is_err());
        assert!(save_progress(&f.storage.dir, &f.deck, &f.candidate).is_err());
        assert!(recover(&f.storage.dir).is_err());
        assert!(other.save(&f.candidate).is_err());
        assert!(save_progress(&other.dir, &f.deck, &f.candidate).is_err());
        let p = f
            .storage
            .read_progress()
            .expect("pure read remains available for this valid fixture");
        assert_eq!(p.version, 2);
        let mut writer =
            HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1()).unwrap();
        assert_eq!(
            writer
                .prepare(operation(), None, &request())
                .unwrap_err()
                .kind,
            HistoryErrorKind::Io
        );
        drop(writer);
        assert!(f.ledger() == before, "blocked writers must not touch bytes");
        assert!(HistoryReader::new(&f.storage)
            .read_snapshot(Some(a))
            .root
            .is_some());
    }
    #[test]
    fn t04_unknown_intent_publish_states_block_without_repair() {
        const CASE: &str = "commit::i2_material_save_tests::t04_unknown_intent_publish_states_block_without_repair";
        isolated(CASE, 7, |mode| {
            let mut f = Fixture::new(if mode == 6 { None } else { Some(V2) });
            let a = f.install_a();
            let other = Storage::at(f.temp.root.join("other")).unwrap();
            let mut expected = f.ledger();
            let calls = Cell::new(0);
            let held = RefCell::<Option<File>>::new(None);
            let denied = Cell::new(None);
            let actual_failed = Cell::new(false);
            let faults = Faults::new(None);
            let result = f.call(SMALL, &faults, |path, bytes| {
                calls.set(calls.get() + 1);
                if path != f.path(PENDING) {
                    return Err("fixture refuses unexpected publish path".into());
                }
                if mode == 5 {
                    let mut file = OpenOptions::new()
                        .create_new(true)
                        .read(true)
                        .write(true)
                        .share_mode(0)
                        .open(path)
                        .map_err(|e| e.to_string())?;
                    file.write_all(b"fixture locked pending")
                        .map_err(|e| e.to_string())?;
                    file.sync_all().map_err(|e| e.to_string())?;
                    denied.set(fs::read(path).err().and_then(|e| e.raw_os_error()));
                    let actual = atomic_write(path, bytes);
                    actual_failed.set(actual.is_err());
                    *held.borrow_mut() = Some(file);
                    return actual;
                }
                atomic_write(path, bytes)?;
                match mode {
                    0 => fs::write(path, b"third pending").map_err(|e| e.to_string())?,
                    1 => {
                        let file = OpenOptions::new()
                            .read(true)
                            .share_mode(0)
                            .open(path)
                            .map_err(|e| e.to_string())?;
                        denied.set(fs::read(path).err().and_then(|e| e.raw_os_error()));
                        *held.borrow_mut() = Some(file);
                    }
                    2 => fs::write(
                        f.path("progress.json"),
                        V2.replace("old raw", "external third"),
                    )
                    .map_err(|e| e.to_string())?,
                    3 => fs::remove_file(f.path("custom.tsv")).map_err(|e| e.to_string())?,
                    4 => fs::write(f.path("custom.tsv"), &f.deck).map_err(|e| e.to_string())?,
                    6 => fs::write(f.path("progress.json"), V2).map_err(|e| e.to_string())?,
                    _ => {}
                }
                Err("same ambiguous publisher result".into())
            });
            drop(held.borrow_mut().take());
            assert_eq!(calls.get(), 1, "publisher must be reached");
            assert!(result.is_err());
            assert_eq!(*faults.seen.borrow(), points(false)[..2]);
            expected.insert(
                "host/pending-material-commit.json".into(),
                match mode {
                    0 => b"third pending".to_vec(),
                    5 => b"fixture locked pending".to_vec(),
                    _ => f.intent(),
                },
            );
            match mode {
                2 => {
                    expected.insert(
                        "host/progress.json".into(),
                        V2.replace("old raw", "external third").into_bytes(),
                    );
                }
                3 => {
                    expected.remove(Path::new("host/custom.tsv"));
                }
                4 => {
                    expected.insert("host/custom.tsv".into(), f.deck.as_bytes().to_vec());
                }
                6 => {
                    expected.insert("host/progress.json".into(), V2.as_bytes().to_vec());
                }
                _ => {}
            }
            if mode == 1 || mode == 5 {
                assert_eq!(denied.get(), Some(32));
                eprintln!("T pending actual read denial 32 mode={mode}");
            }
            if mode == 5 {
                assert!(
                    actual_failed.get(),
                    "same production atomic itself must fail"
                );
            }
            assert!(f.ledger() == expected);
            blocked(&mut f, &other, &a);
            if mode == 5 {
                atomic_write(&f.path(PENDING), &f.intent())
                    .expect("same atomic succeeds after unlocking, fixture-only control");
                expected.insert("host/pending-material-commit.json".into(), f.intent());
                assert!(f.ledger() == expected);
                eprintln!("T pending atomic failure and unlocked same-path success verified after block/ledger assertions");
            } else if mode == 1 {
                assert_eq!(fs::read(f.path(PENDING)).unwrap(), f.intent());
            }
        });
    }
    #[test]
    fn t05_unknown_r_fault_propagates_block_through_full_save() {
        const CASE: &str = "commit::i2_material_save_tests::t05_unknown_r_fault_propagates_block_through_full_save";
        isolated(CASE, 1, |_| {
            let mut f = Fixture::new(Some(V2));
            let a = f.install_a();
            let other = Storage::at(f.temp.root.join("other")).unwrap();
            let mut expected = f.ledger();
            let seen = RefCell::new(vec![]);
            let faults = Hook(|p| {
                seen.borrow_mut().push(p);
                if p == CP::AfterDeck {
                    fs::write(f.path("custom.tsv"), b"external R fault deck").unwrap();
                    Err(ProgressFaultError::Io)
                } else {
                    Ok(())
                }
            });
            let result = save_progress_with_limits_and_faults(
                &f.storage.dir,
                &f.deck,
                &f.candidate,
                SMALL,
                &faults,
            );
            assert!(result.is_err());
            assert_eq!(*seen.borrow(), points(false)[..4]);
            expected.insert("host/pending-material-commit.json".into(), f.intent());
            expected.insert("host/custom.tsv".into(), b"external R fault deck".to_vec());
            assert!(f.ledger() == expected);
            drop(faults);
            blocked(&mut f, &other, &a);
        });
    }
    #[test]
    fn t06_callback_reentry_rejects_public_material_normal_and_history_without_deadlock() {
        const CASE: &str = "commit::i2_material_save_tests::t06_callback_reentry_rejects_public_material_normal_and_history_without_deadlock";
        isolated(CASE, 1, |_| {
            let f = Fixture::new(Some(V2));
            let mut other = Storage::at(f.temp.root.join("other")).unwrap();
            let before = f.ledger();
            let calls = Cell::new(0);
            let gate = RefCell::new(None);
            let nested = RefCell::new(vec![]);
            let unchanged = Cell::new(false);
            let saved = f
                .call(SMALL, &NoProgressFaults, |path, bytes| {
                    calls.set(calls.get() + 1);
                    let snapshot = f.ledger();
                    *gate.borrow_mut() = Some(acquire_progress_write().map(drop));
                    nested
                        .borrow_mut()
                        .push(save_progress(&f.storage.dir, &f.deck, &f.candidate).is_err());
                    nested
                        .borrow_mut()
                        .push(f.storage.save(&f.candidate).is_err());
                    let mut writer =
                        HistoryWriter::new(&mut other, &ProgressHistoryHead, HistoryQuota::v1())
                            .map_err(|e| format!("{e:?}"))?;
                    nested.borrow_mut().push(matches!(
                        writer.prepare(operation(), None, &request()),
                        Err(e) if e.kind == HistoryErrorKind::Busy
                    ));
                    drop(writer);
                    unchanged.set(f.ledger() == snapshot);
                    atomic_write(path, bytes)
                })
                .unwrap();
            assert_eq!(calls.get(), 1);
            assert_eq!(*gate.borrow(), Some(Err(ProgressGateError::Reentrant)));
            assert_eq!(*nested.borrow(), [true, true, true]);
            assert!(unchanged.get());
            assert_eq!(serde_json::to_vec_pretty(&saved).unwrap(), f.next());
            f.complete(before, &f.intent());
            writable();
        });
    }
    #[test]
    fn t06_same_alias_and_distinct_root_material_publish_intervals_do_not_overlap() {
        const CASE: &str = "commit::i2_material_save_tests::t06_same_alias_and_distinct_root_material_publish_intervals_do_not_overlap";
        isolated(CASE, 1, |_| {
            for same_root in [true, false] {
                let mut a = Fixture::new(Some(V2));
                let mut b = Fixture::new(Some(V2));
                let ra = a.install_a();
                a.set_root(ra.clone());
                let rb = b.install_a();
                b.set_root(rb);
                let roots = [
                    a.storage.dir.clone(),
                    if same_root {
                        a.storage.dir.join(".")
                    } else {
                        b.storage.dir.clone()
                    },
                ];
                let active = Arc::new(AtomicUsize::new(0));
                let overlap = Arc::new(AtomicBool::new(false));
                let held = Arc::new(AtomicUsize::new(0));
                let barrier = Arc::new(Barrier::new(3));
                let (tx, rx) = mpsc::channel();
                let mut handles = vec![];
                let mut releases = vec![];
                for (i, root) in roots.into_iter().enumerate() {
                    let (release_tx, release_rx) = mpsc::channel();
                    releases.push(release_tx);
                    let mut candidate = a.candidate.clone();
                    candidate.settings.topic = format!("T worker {i}");
                    let deck = a.deck.clone();
                    let active = active.clone();
                    let overlap = overlap.clone();
                    let held = held.clone();
                    let barrier = barrier.clone();
                    let tx = tx.clone();
                    handles.push(std::thread::spawn(move || {
                        barrier.wait();
                        let r = save_progress_with_intent_publisher(
                            &root,
                            &deck,
                            &candidate,
                            SMALL,
                            &NoProgressFaults,
                            |path, bytes| {
                                if active.fetch_add(1, Ordering::SeqCst) != 0 {
                                    overlap.store(true, Ordering::SeqCst);
                                }
                                if matches!(
                                    acquire_progress_write(),
                                    Err(ProgressGateError::Reentrant)
                                ) {
                                    held.fetch_add(1, Ordering::SeqCst);
                                }
                                let _ = tx.send(i);
                                let r = if release_rx.recv_timeout(Duration::from_secs(3)).is_ok() {
                                    atomic_write(path, bytes)
                                } else {
                                    Err("bounded release absent".into())
                                };
                                active.fetch_sub(1, Ordering::SeqCst);
                                r
                            },
                        );
                        (i, r)
                    }));
                }
                drop(tx);
                barrier.wait();
                let mut order = vec![];
                for _ in 0..2 {
                    match rx.recv_timeout(Duration::from_secs(4)) {
                        Ok(i) => {
                            order.push(i);
                            let _ = releases[i].send(());
                        }
                        Err(_) => break,
                    }
                }
                drop(releases);
                let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                assert_eq!(order.len(), 2, "observe both actual intervals, not silence");
                assert!(!overlap.load(Ordering::SeqCst));
                assert_eq!(active.load(Ordering::SeqCst), 0);
                assert_eq!(held.load(Ordering::SeqCst), 2);
                for (i, r) in results {
                    let p = r.unwrap();
                    assert_eq!(p.settings.topic, format!("T worker {i}"));
                    assert_eq!(p.audio_history, Some(ra.clone()));
                }
                if same_root {
                    assert_eq!(
                        a.storage.read_progress().unwrap().settings.topic,
                        format!("T worker {}", order[1])
                    );
                } else {
                    assert_eq!(
                        a.storage.read_progress().unwrap().settings.topic,
                        "T worker 0"
                    );
                    assert_eq!(
                        b.storage.read_progress().unwrap().settings.topic,
                        "T worker 1"
                    );
                }
                writable();
            }
        });
    }
    #[test]
    fn t06_public_normal_history_and_material_share_the_gate_under_competition() {
        const CASE: &str = "commit::i2_material_save_tests::t06_public_normal_history_and_material_share_the_gate_under_competition";
        isolated(CASE, 1, |_| {
            struct Interval {
                active: Arc<AtomicUsize>,
                overlap: Arc<AtomicBool>,
                held: Arc<AtomicUsize>,
                tx: mpsc::Sender<usize>,
                release: mpsc::Receiver<()>,
            }
            impl HistoryFaults for Interval {
                fn check(&self, p: HistoryCheckpoint) -> Result<(), HistoryError> {
                    if p == HistoryCheckpoint::BeforeIntent {
                        if self.active.fetch_add(1, Ordering::SeqCst) != 0 {
                            self.overlap.store(true, Ordering::SeqCst);
                        }
                        if matches!(acquire_progress_write(), Err(ProgressGateError::Reentrant)) {
                            self.held.fetch_add(1, Ordering::SeqCst);
                        }
                        let _ = self.tx.send(1);
                        if self.release.recv_timeout(Duration::from_secs(3)).is_err() {
                            return Err(HistoryError {
                                kind: HistoryErrorKind::Io,
                            });
                        }
                    }
                    if p == HistoryCheckpoint::AfterIntent {
                        self.active.fetch_sub(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            }
            let t = Fixture::new(Some(V2));
            let n = Fixture::new(Some(V2));
            let h = Fixture::new(None);
            let troot = t.storage.dir.clone();
            let nroot = n.storage.dir.clone();
            let hroot = h.storage.dir.clone();
            let mut tp = t.candidate.clone();
            tp.settings.topic = "material result".into();
            let mut np = n.candidate.clone();
            np.settings.topic = "normal result".into();
            let deck = t.deck.clone();
            let active = Arc::new(AtomicUsize::new(0));
            let overlap = Arc::new(AtomicBool::new(false));
            let held = Arc::new(AtomicUsize::new(0));
            let barrier = Arc::new(Barrier::new(4));
            let (tx, rx) = mpsc::channel();
            let (t_release, t_rx) = mpsc::channel();
            let (h_release, h_rx) = mpsc::channel();
            let ta = active.clone();
            let to = overlap.clone();
            let tg = held.clone();
            let tb = barrier.clone();
            let tt = tx.clone();
            let target = troot.clone();
            let t_worker = std::thread::spawn(move || {
                tb.wait();
                save_progress_with_intent_publisher(
                    &target,
                    &deck,
                    &tp,
                    SMALL,
                    &NoProgressFaults,
                    |path, bytes| {
                        if ta.fetch_add(1, Ordering::SeqCst) != 0 {
                            to.store(true, Ordering::SeqCst);
                        }
                        if matches!(acquire_progress_write(), Err(ProgressGateError::Reentrant)) {
                            tg.fetch_add(1, Ordering::SeqCst);
                        }
                        let _ = tt.send(0);
                        let r = if t_rx.recv_timeout(Duration::from_secs(3)).is_ok() {
                            atomic_write(path, bytes)
                        } else {
                            Err("bounded material release absent".into())
                        };
                        ta.fetch_sub(1, Ordering::SeqCst);
                        r
                    },
                )
            });
            let nb = barrier.clone();
            let n_worker = std::thread::spawn(move || {
                nb.wait();
                n.storage.save_progress(&np)
            });
            let hb = barrier.clone();
            let spy = Interval {
                active: active.clone(),
                overlap: overlap.clone(),
                held: held.clone(),
                tx: tx.clone(),
                release: h_rx,
            };
            let h_worker = std::thread::spawn(move || {
                let mut storage = h.storage;
                let mut writer = HistoryWriter::with_faults(
                    &mut storage,
                    &ProgressHistoryHead,
                    HistoryQuota::v1(),
                    &spy,
                )
                .unwrap();
                hb.wait();
                writer.prepare(operation(), None, &request()).map(|_| ())
            });
            drop(tx);
            barrier.wait();
            let mut observed = vec![];
            for _ in 0..2 {
                match rx.recv_timeout(Duration::from_secs(4)) {
                    Ok(i) => {
                        observed.push(i);
                        let _ = if i == 0 {
                            t_release.send(())
                        } else {
                            h_release.send(())
                        };
                    }
                    Err(_) => break,
                }
            }
            drop(t_release);
            drop(h_release);
            let tr = t_worker.join().unwrap();
            let nr = n_worker.join().unwrap();
            let hr = h_worker.join().unwrap();
            assert_eq!(observed.len(), 2);
            assert!(observed.contains(&0) && observed.contains(&1));
            assert!(!overlap.load(Ordering::SeqCst));
            assert_eq!(active.load(Ordering::SeqCst), 0);
            assert_eq!(held.load(Ordering::SeqCst), 2);
            assert_eq!(tr.unwrap().settings.topic, "material result");
            assert_eq!(nr.unwrap().settings.topic, "normal result");
            hr.unwrap();
            let read = |p: &Path| {
                serde_json::from_slice::<Progress>(&fs::read(p.join("progress.json")).unwrap())
                    .unwrap()
            };
            assert_eq!(read(&troot).settings.topic, "material result");
            assert_eq!(read(&nroot).settings.topic, "normal result");
            assert!(!hroot.join("progress.json").exists());
            writable();
        });
    }
}
