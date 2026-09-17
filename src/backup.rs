//! Explicit learning-data backup: deck, progress, source snapshots and all media originals.
use crate::{
    assets::{sha256, AssetRef, AssetStore},
    model::{self, Entry},
    store::{atomic_write, Progress, Storage},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    deck_sha256: String,
    progress_sha256: String,
    assets: Vec<AssetRef>,
}
pub struct Prepared {
    pub progress: Progress,
    pub deck: Vec<Entry>,
    pub path: PathBuf,
    assets: Vec<AssetRef>,
}
#[derive(Clone, Copy)]
struct Limits {
    progress: u64,
    manifest: u64,
}
const DEFAULT_LIMITS: Limits = Limits {
    progress: crate::commit::PROGRESS_LIMIT,
    manifest: 8_000_000,
};

pub fn verify_references(root: &Path, progress: &Progress) -> Result<(), String> {
    let store = AssetStore::new(root.to_owned());
    let chats = progress.chats.iter().flat_map(|c| {
        c.draft_attachments
            .iter()
            .chain(c.exchanges.iter().flat_map(|e| e.attachments.iter()))
    });
    let sources = progress
        .material_sources
        .iter()
        .chain(progress.material_draft.iter().map(|d| &d.source))
        .flat_map(|s| {
            s.snapshots
                .iter()
                .flat_map(|s| s.exchange.attachments.iter())
        });
    let mut seen = std::collections::HashSet::new();
    for a in chats.chain(sources) {
        a.validate()?;
        for r in std::iter::once(&a.original)
            .chain(a.image.iter())
            .chain(a.background.iter())
        {
            if seen.insert((r.id.clone(), r.kind, r.bytes)) {
                store.read(r)?;
            }
        }
    }
    Ok(())
}
pub fn export(root: &Path, dest: &Path, deck: &[Entry], progress: &Progress) -> Result<(), String> {
    export_with_limits(root, dest, deck, progress, DEFAULT_LIMITS)
}
fn export_with_limits(
    root: &Path,
    dest: &Path,
    deck: &[Entry],
    progress: &Progress,
    limits: Limits,
) -> Result<(), String> {
    reject_asset_destination(root, dest)?;
    let deck = model::deck_text(deck);
    model::parse_deck(&deck)?;
    if deck.len() as u64 > crate::commit::DECK_LIMIT {
        return Err("バックアップの教材が大きすぎます。".into());
    }
    let bytes = progress_bytes(progress, limits.progress)?;
    verify_references(root, progress)?;
    let store = AssetStore::new(root.to_owned());
    let assets = store.inventory()?;
    let manifest = Manifest {
        version: 1,
        deck_sha256: sha256(deck.as_bytes())?,
        progress_sha256: sha256(&bytes)?,
        assets,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    if manifest_bytes.len() as u64 > limits.manifest {
        return Err("バックアップの管理情報が大きすぎます。".into());
    }
    // Never merge into or overwrite an existing backup directory.
    fs::create_dir(dest).map_err(|e| e.to_string())?;
    let target = AssetStore::new(dest.to_owned());
    for r in &manifest.assets {
        target.put(r.kind, &store.read(r)?)?;
    }
    atomic_write(&dest.join("custom.tsv"), deck.as_bytes())?;
    atomic_write(&dest.join("progress.json"), &bytes)?;
    atomic_write(&dest.join("learning-backup.json"), &manifest_bytes)
}
fn reject_asset_destination(root: &Path, dest: &Path) -> Result<(), String> {
    if root.exists() {
        let assets = root.join("assets");
        let assets = if assets.exists() {
            fs::canonicalize(assets)
        } else {
            fs::canonicalize(root).map(|p| p.join("assets"))
        }
        .map_err(|e| e.to_string())?;
        let parent = fs::canonicalize(dest.parent().ok_or("退避先の親フォルダーがありません。")?)
            .map_err(|e| e.to_string())?;
        let destination = parent.join(dest.file_name().ok_or("退避先の名前がありません。")?);
        if destination.starts_with(assets) {
            return Err("媒体原本のassetsフォルダー内はバックアップ先に指定できません。別の保存先を選んでください。".into());
        }
    }
    Ok(())
}
fn progress_bytes(progress: &Progress, limit: u64) -> Result<Vec<u8>, String> {
    progress.validate()?;
    let bytes = serde_json::to_vec_pretty(progress).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("バックアップの学習記録が大きすぎます。".into());
    }
    Ok(bytes)
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    if f.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err("バックアップが大きすぎます。".into());
    }
    let mut bytes = Vec::new();
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("バックアップが大きすぎます。".into());
    }
    Ok(bytes)
}
pub fn prepare(path: PathBuf) -> Result<Prepared, String> {
    prepare_with_limits(path, DEFAULT_LIMITS)
}
fn prepare_with_limits(path: PathBuf, limits: Limits) -> Result<Prepared, String> {
    let manifest: Manifest =
        serde_json::from_slice(&read(&path.join("learning-backup.json"), limits.manifest)?)
            .map_err(|e| e.to_string())?;
    if manifest.version != 1 {
        return Err("バックアップ形式が未対応です。".into());
    }
    let deck = read(&path.join("custom.tsv"), crate::commit::DECK_LIMIT)?;
    let progress = read(&path.join("progress.json"), limits.progress)?;
    if sha256(&deck)? != manifest.deck_sha256 || sha256(&progress)? != manifest.progress_sha256 {
        return Err("教材または学習記録の破損を検出しました。復元しません。".into());
    }
    let deck = model::parse_deck(std::str::from_utf8(&deck).map_err(|e| e.to_string())?)?;
    let progress: Progress = serde_json::from_slice(&progress).map_err(|e| e.to_string())?;
    progress.validate()?;
    let store = AssetStore::new(path.clone());
    for r in &manifest.assets {
        store.read(r)?;
    }
    verify_references(&path, &progress)?;
    Ok(Prepared {
        path,
        deck,
        progress,
        assets: manifest.assets,
    })
}
/// Preserve unsaved drafts independently of the on-disk rollback copy.
pub fn preserve_current_progress(storage: &Storage, current: &Progress) -> Result<(), String> {
    let current_bytes = progress_bytes(current, DEFAULT_LIMITS.progress)?;
    let backups = storage.dir.join("backups");
    fs::create_dir_all(&backups).map_err(|e| e.to_string())?;
    let snapshot = backups.join(format!(
        "progress-memory-before-restore-{}-{}.json",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    atomic_write(&snapshot, &current_bytes)
}
impl Prepared {
    pub fn media_count(&self) -> usize {
        self.assets.len()
    }
    pub fn restore(
        self,
        storage: &Storage,
        current: &Progress,
    ) -> Result<(Vec<Entry>, Progress), String> {
        // The UI may contain a newer draft than progress.json. Keep both recovery points.
        let mut progress = self.progress;
        // Restoring old data cannot replenish the locally counted generation allowance.
        for (day, used) in &current.ai_calls {
            let n = progress.ai_calls.entry(day.clone()).or_default();
            *n = (*n).max(*used);
        }
        let source = AssetStore::new(self.path);
        let target = AssetStore::new(storage.dir.clone());
        // Verify first, then publish immutable media, then commit both metadata files.
        for r in &self.assets {
            source.read(r)?;
        }
        preserve_current_progress(storage, current)?;
        for r in &self.assets {
            target.put(r.kind, &source.read(r)?)?;
        }
        verify_references(&storage.dir, &progress)?;
        crate::commit::save(&storage.dir, &model::deck_text(&self.deck), &progress)?;
        Ok((self.deck, progress))
    }
}
#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn refuses_backup_inside_original_assets_before_creating_anything() {
        let root = std::env::temp_dir().join(format!(
            "ww-backup-self-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir(&root).unwrap();
        let deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
        let progress = Progress::default();
        assert!(export(&root, &root.join("assets"), &deck, &progress).is_err());
        assert!(!root.join("assets").exists());
        let store = AssetStore::new(root.clone());
        store
            .put(crate::assets::AssetKind::AudioWav, b"RIFF\x04\0\0\0WAVE")
            .unwrap();
        let dest = root.join("assets/nested-backup");
        assert!(export(&root, &dest, &deck, &progress).is_err());
        assert!(!dest.exists());
        assert_eq!(store.inventory().unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn memory_snapshot_rejects_invalid_or_oversized_progress() {
        let mut progress = Progress::default();
        let exact = serde_json::to_vec_pretty(&progress).unwrap().len() as u64;
        assert!(progress_bytes(&progress, exact - 1).is_err());
        assert!(progress_bytes(&progress, exact).is_ok());
        progress.settings.minutes = 0;
        assert!(progress_bytes(&progress, DEFAULT_LIMITS.progress).is_err());
    }
    #[test]
    fn restore_preserves_current_unsaved_draft_in_separate_recovery_file() {
        let root = std::env::temp_dir().join(format!(
            "ww-backup-memory-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source");
        let dest = root.join("backup");
        let storage = Storage::at(root.join("restored")).unwrap();
        let deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
        let disk = Progress::default();
        storage.save(&disk).unwrap();
        export(&source, &dest, &deck, &disk).unwrap();
        let mut current = disk.clone();
        let mut chat = crate::chat::Conversation::new();
        chat.draft = "ディスクへ未保存の下書き".into();
        current.chats.push(chat);
        prepare(dest).unwrap().restore(&storage, &current).unwrap();
        let paths = fs::read_dir(storage.dir.join("backups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("progress-memory-before-restore-")
            })
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 1);
        let saved: Progress = serde_json::from_slice(&fs::read(&paths[0]).unwrap()).unwrap();
        assert_eq!(saved.chats[0].draft, current.chats[0].draft);
        let disk_backups = fs::read_dir(storage.dir.join("backups"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("material-commit-")
            })
            .collect::<Vec<_>>();
        assert_eq!(disk_backups.len(), 1);
        let intent: serde_json::Value =
            serde_json::from_slice(&fs::read(&disk_backups[0]).unwrap()).unwrap();
        let old_disk: Progress =
            serde_json::from_str(intent["old_progress"].as_str().unwrap()).unwrap();
        assert!(old_disk.chats.is_empty());
        assert!(storage.load().unwrap().chats.is_empty());
        drop(storage);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_unrestorable_sizes_before_creating_destination() {
        let root = std::env::temp_dir().join(format!(
            "ww-backup-limits-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source");
        let deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
        let progress = Progress::default();
        let progress_bytes = serde_json::to_vec_pretty(&progress).unwrap();
        for (name, limits) in [
            (
                "progress",
                Limits {
                    progress: progress_bytes.len() as u64 - 1,
                    manifest: 8_000_000,
                },
            ),
            (
                "manifest",
                Limits {
                    progress: 100_000_000,
                    manifest: 1,
                },
            ),
        ] {
            let dest = root.join(name);
            assert!(export_with_limits(&source, &dest, &deck, &progress, limits).is_err());
            assert!(!dest.exists());
        }
        let first = root.join("first");
        export(&source, &first, &deck, &progress).unwrap();
        let limits = Limits {
            progress: fs::metadata(first.join("progress.json")).unwrap().len(),
            manifest: fs::metadata(first.join("learning-backup.json"))
                .unwrap()
                .len(),
        };
        let exact = root.join("exact");
        export_with_limits(&source, &exact, &deck, &progress, limits).unwrap();
        assert!(prepare_with_limits(exact, limits).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn backup_restores_detached_originals_and_rejects_corruption() {
        let root = std::env::temp_dir().join(format!(
            "ww-backup-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source");
        let dest = root.join("backup");
        let restored = root.join("restored");
        let store = AssetStore::new(source.clone());
        let original = store
            .put(crate::assets::AssetKind::AudioWav, b"RIFF\x04\0\0\0WAVE")
            .unwrap();
        let deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
        let progress = Progress::default();
        export(&source, &dest, &deck, &progress).unwrap();
        assert!(export(&source, &dest, &deck, &progress).is_err());
        let prepared = prepare(dest.clone()).unwrap();
        assert_eq!(prepared.media_count(), 1);
        let storage = Storage::at(restored.clone()).unwrap();
        prepared.restore(&storage, &progress).unwrap();
        assert_eq!(
            AssetStore::new(restored).read(&original).unwrap(),
            b"RIFF\x04\0\0\0WAVE"
        );
        fs::write(dest.join("progress.json"), b"corrupt").unwrap();
        assert!(prepare(dest).is_err());
        drop(storage);
        fs::remove_dir_all(root).unwrap();
    }
}
