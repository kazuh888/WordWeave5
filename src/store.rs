use crate::{
    audio_history::{CommittedHistoryHead, HistoryError, HistoryErrorKind, RootRef},
    model::Skill,
    scheduler::{Grade, Memory},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ProgressGateError {
    Poisoned,
    WriteBlocked,
    Reentrant,
}

static PROGRESS_WRITE: Mutex<()> = Mutex::new(());
// Only a holder of PROGRESS_WRITE may inspect or change this flag.
static PROGRESS_WRITE_BLOCKED: AtomicBool = AtomicBool::new(false);
thread_local! {
    static PROGRESS_WRITE_HELD: Cell<bool> = const { Cell::new(false) };
}

impl ProgressGateError {
    pub(crate) fn message(self) -> String {
        match self {
            Self::Poisoned => "保存処理で異常が発生しました。書き込みを停止しています。",
            Self::WriteBlocked => "保存状態の確認が必要です。書き込みを停止しています。",
            Self::Reentrant => "保存処理中に別の保存を開始できません。",
        }
        .into()
    }
}

/// Holds the process-wide writer exclusion. MutexGuard also makes this !Send.
#[derive(Debug)]
pub(crate) struct ProgressWriteGuard {
    _lock: MutexGuard<'static, ()>,
}

pub(crate) fn acquire_progress_write() -> Result<ProgressWriteGuard, ProgressGateError> {
    let guard = acquire_progress_exclusion()?;
    guard.ensure_writable()?;
    Ok(guard)
}

fn acquire_progress_exclusion() -> Result<ProgressWriteGuard, ProgressGateError> {
    if PROGRESS_WRITE_HELD.with(Cell::get) {
        return Err(ProgressGateError::Reentrant);
    }
    let lock = PROGRESS_WRITE
        .lock()
        .map_err(|_| ProgressGateError::Poisoned)?;
    PROGRESS_WRITE_HELD.with(|held| held.set(true));
    Ok(ProgressWriteGuard { _lock: lock })
}

impl ProgressWriteGuard {
    pub(crate) fn ensure_writable(&self) -> Result<(), ProgressGateError> {
        if PROGRESS_WRITE_BLOCKED.load(Ordering::Relaxed) {
            Err(ProgressGateError::WriteBlocked)
        } else {
            Ok(())
        }
    }

    pub(crate) fn block_writes(&self) {
        PROGRESS_WRITE_BLOCKED.store(true, Ordering::Relaxed);
    }
}

impl Drop for ProgressWriteGuard {
    fn drop(&mut self) {
        PROGRESS_WRITE_HELD.with(|held| held.set(false));
        // The field's normal drop releases the mutex, retaining panic poison.
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MigrationError {
    Gate(ProgressGateError),
    InvalidDocument,
    UnsupportedVersion,
    LimitExceeded,
    UnsafePath,
    Conflict,
    Io,
}

impl MigrationError {
    pub(crate) fn message(self) -> String {
        match self {
            Self::Gate(error) => return error.message(),
            Self::InvalidDocument => "移行前の学習記録が不正です。元ファイルは変更していません。",
            Self::UnsupportedVersion => {
                "学習記録の版に対応していません。元ファイルは変更していません。"
            }
            Self::LimitExceeded => {
                "移行前の学習記録が保存上限を超えています。元ファイルは変更していません。"
            }
            Self::UnsafePath => {
                "移行退避先に安全でないパスがあります。元ファイルは変更していません。"
            }
            Self::Conflict => "既存の移行退避が元の学習記録と一致しません。上書きしていません。",
            Self::Io => "学習記録の移行退避を確認できません。保存を停止しました。",
        }
        .into()
    }
}

/// Return the original wire version, not the normalized runtime version.
pub(crate) fn progress_wire_version(bytes: &[u8]) -> Result<u32, MigrationError> {
    decode_progress(bytes)
        .map(|decoded| decoded.wire_version)
        .map_err(ProgressReadError::migration_error)
}

pub(crate) fn decode_progress_for_material(bytes: &[u8]) -> Result<(u32, Progress), HistoryError> {
    decode_progress(bytes)
        .map(|decoded| (decoded.wire_version, decoded.progress))
        .map_err(ProgressReadError::history_error)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressFaultError {
    Io,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressCheckpoint {
    BeforeMigrationBackup,
    AfterMigrationBackup,
    BeforePointer,
    AfterPointer,
    BeforeMaterialIntent,
    AfterMaterialIntent,
    BeforeDeck,
    AfterDeck,
    BeforeMaterialProgress,
    AfterMaterialProgress,
    BeforeMaterialFinish,
    AfterMaterialFinish,
}

pub trait ProgressFaults {
    fn check(&self, point: ProgressCheckpoint) -> Result<(), ProgressFaultError>;
}

pub struct NoProgressFaults;

impl ProgressFaults for NoProgressFaults {
    fn check(&self, _point: ProgressCheckpoint) -> Result<(), ProgressFaultError> {
        Ok(())
    }
}

pub(crate) fn preserve_v1_before_v2_under_guard(
    root: &Path,
    old_bytes: Option<&[u8]>,
    next_wire_version: u32,
    limit: u64,
    guard: &ProgressWriteGuard,
    faults: &dyn ProgressFaults,
) -> Result<(), MigrationError> {
    preserve_v1_before_v2_with_publisher(
        root,
        old_bytes,
        next_wire_version,
        limit,
        guard,
        faults,
        atomic_write,
    )
}

fn preserve_v1_before_v2_with_publisher<P>(
    root: &Path,
    old_bytes: Option<&[u8]>,
    next_wire_version: u32,
    limit: u64,
    guard: &ProgressWriteGuard,
    faults: &dyn ProgressFaults,
    publish: P,
) -> Result<(), MigrationError>
where
    P: FnOnce(&Path, &[u8]) -> Result<(), String>,
{
    guard.ensure_writable().map_err(MigrationError::Gate)?;
    match next_wire_version {
        1 => return Ok(()),
        2 => {}
        _ => return Err(MigrationError::UnsupportedVersion),
    }
    let Some(old_bytes) = old_bytes else {
        return Ok(());
    };
    if old_bytes.len() as u64 > limit {
        return Err(MigrationError::LimitExceeded);
    }
    let old = decode_progress(old_bytes).map_err(ProgressReadError::migration_error)?;
    if old.wire_version == 2 {
        return Ok(());
    }
    let hash = crate::assets::sha256(old_bytes).map_err(|_| MigrationError::Io)?;
    let backups = root.join("backups");
    let path = backups.join(format!("progress-v1-{hash}.json"));
    migration_paths(root, &backups, &path)?;
    let existing = migration_read(&path, limit)?;
    if existing.as_deref().is_some_and(|bytes| bytes != old_bytes) {
        return Err(MigrationError::Conflict);
    }
    faults
        .check(ProgressCheckpoint::BeforeMigrationBackup)
        .map_err(|_| MigrationError::Io)?;
    guard.ensure_writable().map_err(MigrationError::Gate)?;
    migration_paths(root, &backups, &path)?;
    // Recheck after the callback; no existing mismatched file can be overwritten.
    let now = migration_read(&path, limit)?;
    if now.as_deref().is_some_and(|bytes| bytes != old_bytes) {
        return Err(MigrationError::Conflict);
    }
    if existing.is_some() && now.is_none() {
        // An already verified backup is never rewritten, including after removal.
        return Err(MigrationError::Io);
    }
    if now.is_none() {
        if migration_metadata(&backups, true)?.is_none() {
            fs::create_dir(&backups).map_err(|_| MigrationError::Io)?;
        }
        migration_paths(root, &backups, &path)?;
        // Return status is not proof of publication: even Err can follow rename.
        let _publication = publish(&path, old_bytes);
    }
    migration_paths(root, &backups, &path)?;
    let readback = migration_read(&path, limit)?.ok_or(MigrationError::Io)?;
    if readback != old_bytes {
        return Err(MigrationError::Conflict);
    }
    faults
        .check(ProgressCheckpoint::AfterMigrationBackup)
        .map_err(|_| MigrationError::Io)
}

fn migration_metadata(
    path: &Path,
    directory: bool,
) -> Result<Option<fs::Metadata>, MigrationError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(MigrationError::Io),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(MigrationError::UnsafePath);
        }
    }
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err(MigrationError::UnsafePath);
    }
    Ok(Some(metadata))
}

fn migration_root(root: &Path) -> Result<(), MigrationError> {
    let absolute = std::path::absolute(root).map_err(|_| MigrationError::Io)?;
    let ancestors: Vec<_> = absolute.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        migration_metadata(ancestor, true)?.ok_or(MigrationError::Io)?;
    }
    Ok(())
}

fn migration_paths(root: &Path, backups: &Path, leaf: &Path) -> Result<(), MigrationError> {
    migration_root(root)?;
    migration_metadata(backups, true)?;
    migration_metadata(leaf, false)?;
    Ok(())
}

fn migration_read(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, MigrationError> {
    use std::io::Read;
    let Some(metadata) = migration_metadata(path, false)? else {
        return Ok(None);
    };
    if metadata.len() > limit {
        return Err(MigrationError::LimitExceeded);
    }
    let file = File::open(path).map_err(|_| MigrationError::Io)?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| MigrationError::Io)?;
    if bytes.len() as u64 > limit {
        return Err(MigrationError::LimitExceeded);
    }
    if bytes.len() as u64 != metadata.len() {
        return Err(MigrationError::Conflict);
    }
    Ok(Some(bytes))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TintChoice {
    pub rgb: [u8; 3],
    pub depth: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub codex_path: String,
    pub codex_model: String,
    pub codex_effort: String,
    pub examples_per_word: usize,
    pub batch_words: usize,
    pub minutes: u32,
    pub new_per_day: usize,
    pub skills: Vec<Skill>,
    pub topic: String,
    pub font_scale: f32,
    pub ai_daily_limit: u32,
    pub voice_id: String,
    pub slow_speech: bool,
    /// None preserves the old normal/slow selection until the user changes speed.
    pub speech_rate: Option<f64>,
    pub speech_volume: f64,
    pub speech_repeat: bool,
    pub page_tint: TintChoice,
    pub input_tint: TintChoice,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            codex_path: "codex".into(),
            codex_model: String::new(),
            codex_effort: String::new(),
            examples_per_word: 6,
            batch_words: 5,
            minutes: 5,
            new_per_day: 3,
            skills: vec![Skill::Recall, Skill::Usage],
            topic: "すべて".into(),
            font_scale: 0.8,
            ai_daily_limit: 10,
            voice_id: String::new(),
            slow_speech: false,
            speech_rate: None,
            speech_volume: 0.5,
            speech_repeat: false,
            page_tint: TintChoice::default(),
            input_tint: TintChoice::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub key: String,
    pub at: i64,
    pub date: String,
    pub grade: Grade,
    pub assisted: bool,
    pub method: String,
    pub first: bool,
    pub elapsed_days: f64,
    pub seconds: u32,
    pub self_assessed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Progress {
    pub version: u32,
    pub settings: Settings,
    pub memories: BTreeMap<String, Memory>,
    pub reviews: Vec<Review>,
    pub suspended: BTreeSet<String>,
    /// Recoverable deletion: hide from library and scheduling, retain all data.
    #[serde(default)]
    pub deleted_entries: BTreeSet<String>,
    #[serde(default)]
    pub chat_action: Option<(String, crate::chat_action::Action)>,
    pub ai_calls: BTreeMap<String, u32>,
    pub study_seconds: BTreeMap<String, u32>,
    #[serde(default)]
    pub deck_versions: BTreeMap<String, String>,
    #[serde(default)]
    pub writing_logs: Vec<WritingLog>,
    #[serde(default)]
    pub japanese_drafts: BTreeMap<String, String>,
    #[serde(default)]
    pub chats: Vec<crate::chat::Conversation>,
    #[serde(default)]
    pub material_draft: Option<crate::material::Draft>,
    #[serde(default)]
    pub material_sources: Vec<crate::material::Source>,
    pub audio_history: Option<RootRef>,
}

impl<'de> Deserialize<'de> for Progress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ProgressWire::deserialize(deserializer)?;
        normalize_progress(wire)
            .map(|decoded| decoded.progress)
            .map_err(|error| serde::de::Error::custom(error.message()))
    }
}

#[derive(Debug)]
enum HistoryField {
    Missing,
    Present(Option<RootRef>),
}

impl Default for HistoryField {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de> Deserialize<'de> for HistoryField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<RootRef>::deserialize(deserializer).map(Self::Present)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressWire {
    version: u32,
    settings: Settings,
    memories: BTreeMap<String, Memory>,
    reviews: Vec<Review>,
    suspended: BTreeSet<String>,
    #[serde(default)]
    deleted_entries: BTreeSet<String>,
    #[serde(default)]
    chat_action: Option<(String, crate::chat_action::Action)>,
    ai_calls: BTreeMap<String, u32>,
    study_seconds: BTreeMap<String, u32>,
    #[serde(default)]
    deck_versions: BTreeMap<String, String>,
    #[serde(default)]
    writing_logs: Vec<WritingLog>,
    #[serde(default)]
    japanese_drafts: BTreeMap<String, String>,
    #[serde(default)]
    chats: Vec<crate::chat::Conversation>,
    #[serde(default)]
    material_draft: Option<crate::material::Draft>,
    #[serde(default)]
    material_sources: Vec<crate::material::Source>,
    #[serde(default)]
    audio_history: HistoryField,
}

#[derive(Debug)]
struct DecodedProgress {
    wire_version: u32,
    progress: Progress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgressReadError {
    InvalidDocument,
    UnsupportedVersion,
    LimitExceeded,
    UnsafePath,
    Io,
}

impl ProgressReadError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidDocument => "学習記録が不正です。元ファイルは変更していません。",
            Self::UnsupportedVersion => {
                "学習記録の版に対応していません。元ファイルは変更していません。"
            }
            Self::LimitExceeded => {
                "学習記録が読取上限を超えています。元ファイルは変更していません。"
            }
            Self::UnsafePath => {
                "学習記録に安全でないパスがあります。元ファイルは変更していません。"
            }
            Self::Io => "学習記録を読み取れません。元ファイルは変更していません。",
        }
    }

    fn migration_error(self) -> MigrationError {
        match self {
            Self::InvalidDocument => MigrationError::InvalidDocument,
            Self::UnsupportedVersion => MigrationError::UnsupportedVersion,
            Self::LimitExceeded => MigrationError::LimitExceeded,
            Self::UnsafePath => MigrationError::UnsafePath,
            Self::Io => MigrationError::Io,
        }
    }

    fn history_error(self) -> HistoryError {
        HistoryError {
            kind: match self {
                Self::InvalidDocument => HistoryErrorKind::InvalidDocument,
                Self::UnsupportedVersion => HistoryErrorKind::UnsupportedVersion,
                Self::LimitExceeded => HistoryErrorKind::LimitExceeded,
                Self::UnsafePath => HistoryErrorKind::UnsafePath,
                Self::Io => HistoryErrorKind::Io,
            },
        }
    }
}

fn normalize_progress(wire: ProgressWire) -> Result<DecodedProgress, ProgressReadError> {
    let audio_history = match (wire.version, wire.audio_history) {
        (1, HistoryField::Missing | HistoryField::Present(None)) => None,
        (2, HistoryField::Present(root)) => root,
        (1 | 2, _) => return Err(ProgressReadError::InvalidDocument),
        _ => return Err(ProgressReadError::UnsupportedVersion),
    };
    let progress = Progress {
        version: 2,
        settings: wire.settings,
        memories: wire.memories,
        reviews: wire.reviews,
        suspended: wire.suspended,
        deleted_entries: wire.deleted_entries,
        chat_action: wire.chat_action,
        ai_calls: wire.ai_calls,
        study_seconds: wire.study_seconds,
        deck_versions: wire.deck_versions,
        writing_logs: wire.writing_logs,
        japanese_drafts: wire.japanese_drafts,
        chats: wire.chats,
        material_draft: wire.material_draft,
        material_sources: wire.material_sources,
        audio_history,
    };
    progress
        .validate()
        .map_err(|_| ProgressReadError::InvalidDocument)?;
    Ok(DecodedProgress {
        wire_version: wire.version,
        progress,
    })
}

fn decode_progress(bytes: &[u8]) -> Result<DecodedProgress, ProgressReadError> {
    let wire = serde_json::from_slice::<ProgressWire>(bytes)
        .map_err(|_| ProgressReadError::InvalidDocument)?;
    normalize_progress(wire)
}

fn progress_metadata(
    path: &Path,
    directory: bool,
) -> Result<Option<fs::Metadata>, ProgressReadError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(ProgressReadError::Io),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(ProgressReadError::UnsafePath);
        }
    }
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err(ProgressReadError::UnsafePath);
    }
    Ok(Some(metadata))
}

fn preserve_daily_backup_with_publisher<D>(
    root: &Path,
    old_bytes: &[u8],
    publish_daily: D,
) -> Result<(), String>
where
    D: FnOnce(&Path, &[u8]) -> Result<(), String>,
{
    let backups = root.join("backups");
    let path = backups.join(format!(
        "progress-{}.json",
        chrono::Local::now().format("%Y-%m-%d")
    ));
    migration_paths(root, &backups, &path).map_err(MigrationError::message)?;
    if migration_metadata(&path, false)
        .map_err(MigrationError::message)?
        .is_some()
    {
        return Ok(());
    }
    if migration_metadata(&backups, true)
        .map_err(MigrationError::message)?
        .is_none()
    {
        fs::create_dir(&backups).map_err(|_| "日別退避先を作成できません。".to_owned())?;
    }
    // Recheck after directory creation; never replace an existing daily backup.
    migration_paths(root, &backups, &path).map_err(MigrationError::message)?;
    if migration_metadata(&path, false)
        .map_err(MigrationError::message)?
        .is_some()
    {
        return Ok(());
    }
    publish_daily(&path, old_bytes)
}

enum ProgressRootPolicy {
    RequireExisting,
    AllowAbsent,
}

fn progress_root_exists(
    absolute_root: &Path,
    policy: ProgressRootPolicy,
) -> Result<bool, ProgressReadError> {
    let ancestors: Vec<_> = absolute_root.ancestors().collect();
    for ancestor in ancestors.into_iter().rev() {
        if progress_metadata(ancestor, true)?.is_none() {
            return match policy {
                ProgressRootPolicy::RequireExisting => Err(ProgressReadError::Io),
                ProgressRootPolicy::AllowAbsent => Ok(false),
            };
        }
    }
    Ok(true)
}

fn read_progress_bytes(root: &Path, limit: u64) -> Result<Option<Vec<u8>>, ProgressReadError> {
    use std::io::Read;
    let absolute = std::path::absolute(root).map_err(|_| ProgressReadError::Io)?;
    progress_root_exists(&absolute, ProgressRootPolicy::RequireExisting)?;
    let path = absolute.join("progress.json");
    let Some(metadata) = progress_metadata(&path, false)? else {
        // Only a missing leaf under verified ordinary ancestors is an empty store.
        return Ok(None);
    };
    if metadata.len() > limit {
        return Err(ProgressReadError::LimitExceeded);
    }
    // Even NotFound here is I/O failure: the previously observed leaf vanished.
    let file = File::open(&path).map_err(|_| ProgressReadError::Io)?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ProgressReadError::Io)?;
    if bytes.len() as u64 > limit {
        return Err(ProgressReadError::LimitExceeded);
    }
    Ok(Some(bytes))
}

fn read_progress_typed(
    root: &Path,
    limit: u64,
) -> Result<Option<DecodedProgress>, ProgressReadError> {
    read_progress_bytes(root, limit)?
        .map(|bytes| decode_progress(&bytes))
        .transpose()
}

pub(crate) fn read_progress_for_backup(root: &Path) -> Result<Option<Progress>, String> {
    let absolute =
        std::path::absolute(root).map_err(|_| ProgressReadError::Io.message().to_owned())?;
    if !progress_root_exists(&absolute, ProgressRootPolicy::AllowAbsent)
        .map_err(|error| error.message().to_owned())?
    {
        return Ok(None);
    }
    if crate::commit::pending_checked(&absolute).map_err(|error| error.message())? {
        return Err("未完了の教材保存があります。バックアップを停止しました。".into());
    }
    // Recheck the ancestors: disappearance after the first check is an I/O error.
    read_progress_typed(&absolute, crate::commit::PROGRESS_LIMIT)
        .map(|decoded| decoded.map(|decoded| decoded.progress))
        .map_err(|error| error.message().to_owned())
}

pub struct ProgressHistoryHead;

impl CommittedHistoryHead for ProgressHistoryHead {
    fn read_current(&self, storage: &Storage) -> Result<Option<RootRef>, HistoryError> {
        read_progress_typed(&storage.dir, crate::commit::PROGRESS_LIMIT)
            .map(|decoded| decoded.and_then(|decoded| decoded.progress.audio_history))
            .map_err(ProgressReadError::history_error)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WritingLog {
    pub key: String,
    pub at: i64,
    pub problem: crate::learning::Exercise,
    pub answer: String,
    pub feedback: String,
    pub revision: bool,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            version: 2,
            settings: Settings::default(),
            memories: BTreeMap::new(),
            reviews: Vec::new(),
            suspended: BTreeSet::new(),
            deleted_entries: BTreeSet::new(),
            chat_action: None,
            ai_calls: BTreeMap::new(),
            study_seconds: BTreeMap::new(),
            deck_versions: BTreeMap::new(),
            writing_logs: Vec::new(),
            japanese_drafts: BTreeMap::new(),
            chats: Vec::new(),
            material_draft: None,
            material_sources: Vec::new(),
            audio_history: None,
        }
    }
}
impl Progress {
    /// Mutates only the deletion marker. Callers persist a clone before publishing it.
    pub fn set_chat_deleted(&mut self, id: &str, deleted: bool) -> Result<(), String> {
        let index = self
            .chats
            .iter()
            .position(|c| c.id == id)
            .ok_or("会話が見つかりません。")?;
        if self.chats[index].deleted_at.is_some() == deleted {
            return Ok(());
        }
        if deleted
            && (self
                .chat_action
                .as_ref()
                .is_some_and(|(chat_id, _)| chat_id == id)
                || self
                    .material_draft
                    .as_ref()
                    .is_some_and(|draft| draft.source.conversation_id == id))
        {
            return Err(
                "この会話の教材操作・教材案を確定または取り消してから削除してください。".into(),
            );
        }
        let previous = self.chats[index].deleted_at;
        self.chats[index].deleted_at = deleted.then(|| chrono::Utc::now().timestamp());
        if let Err(error) = self.validate() {
            self.chats[index].deleted_at = previous;
            return Err(error);
        }
        Ok(())
    }
    pub fn complete_chat(
        &mut self,
        id: &str,
        question: String,
        reply: crate::chat_action::ChatReply,
    ) -> Result<(), String> {
        let mut next = self.clone();
        let chat = next
            .chats
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or("回答の保存先の会話が見つかりません。")?;
        chat.complete(question, reply.answer, reply.execution)?;
        if !chat.title_manual {
            chat.title = reply.title;
        }
        next.chat_action = reply
            .action
            .filter(|a| a.operation != crate::chat_action::Operation::Organize)
            .map(|a| (id.to_owned(), a));
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn reconcile_deck(&mut self, deck: &[crate::model::Entry]) -> usize {
        let mut changed = 0;
        for entry in deck {
            let fingerprint = entry.fingerprint();
            if self
                .deck_versions
                .get(&entry.id)
                .is_some_and(|old| old != &fingerprint)
            {
                let prefix = format!("{}:", entry.id);
                self.memories.retain(|key, _| !key.starts_with(&prefix));
                changed += 1;
            }
            self.deck_versions.insert(entry.id.clone(), fingerprint);
        }
        changed
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.settings.page_tint.depth > 100 || self.settings.input_tint.depth > 100 {
            return Err("配色の濃さは0から100までの整数で指定してください。".into());
        }
        if self
            .settings
            .speech_rate
            .is_some_and(|rate| !rate.is_finite() || !(0.5..=4.0).contains(&rate))
            || !self.settings.speech_volume.is_finite()
            || !(0.0..=1.0).contains(&self.settings.speech_volume)
            || self.settings.codex_effort.len() > 64
            || !self
                .settings
                .codex_effort
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err("読み上げ速度・音量またはeffortの設定値が不正です。".into());
        }
        if let Some(draft) = &self.material_draft {
            draft.validate_evidence()?;
        }
        for source in &self.material_sources {
            source.validate()?;
        }
        // An editable proposal may be incomplete; validate it only at commit.
        if serde_json::to_vec(&self.material_draft)
            .map_err(|e| e.to_string())?
            .len()
            > 4_000_000
            || self.material_sources.len() > 10000
        {
            return Err("教材案または会話参照の保存上限を超えました。".into());
        }
        if self.chats.iter().filter(|c| c.deleted_at.is_none()).count() > crate::chat::MAX_CHATS {
            return Err(
                "通常のチャットは50件までです。不要な会話をごみ箱へ移動してください。".into(),
            );
        }
        if self.chats.iter().filter(|c| c.deleted_at.is_some()).count()
            > crate::chat::MAX_TRASH_CHATS
        {
            return Err("チャットごみ箱は500件までです。原本保護のため自動削除せず、この操作を停止しました。".into());
        }
        let mut chat_ids = BTreeSet::new();
        for chat in &self.chats {
            chat.validate()?;
            if !chat_ids.insert(&chat.id) {
                return Err("会話IDが重複しています。".into());
            }
        }
        if let Some((id, action)) = &self.chat_action {
            if !chat_ids.contains(id)
                || action.base.chars().count() > 200
                || action.base.chars().any(char::is_control)
                || action.entry_id.as_ref().is_some_and(|s| s.len() > 1000)
            {
                return Err("保存されたチャット操作が不正です。".into());
            }
        }
        if serde_json::to_vec(&self.chats)
            .map_err(|e| e.to_string())?
            .len()
            > 20_000_000
        {
            return Err("チャットの合計が20MBを超えています。".into());
        }
        if self.version != 2 {
            return Err("対応していないデータ形式です。".into());
        }
        if let Some(root) = &self.audio_history {
            root.validate()
                .map_err(|_| "音声履歴の参照が不正です。".to_owned())?;
        }
        if self.settings.minutes == 0
            || self.settings.minutes > 30
            || self.settings.new_per_day > 20
            || !(3..=12).contains(&self.settings.examples_per_word)
            || !(1..=3000).contains(&self.settings.batch_words)
            || self.settings.skills.is_empty()
            || self.settings.ai_daily_limit > 1000
            || !(0.5..=1.6).contains(&self.settings.font_scale)
        {
            return Err("設定値が範囲外です。".into());
        }
        if self.memories.values().any(|m| {
            !m.stability.is_finite()
                || m.stability <= 0.0
                || m.stability > 180.0
                || m.cue_chars > 2
                || m.last < 0
                || m.due < 0
        }) {
            return Err("復習データが不正です。".into());
        }
        Ok(())
    }
    pub fn record(
        &mut self,
        key: String,
        grade: Grade,
        assisted: bool,
        method: &str,
        now: i64,
        date: &str,
        seconds: u32,
        self_assessed: bool,
    ) {
        let m = self.memories.entry(key.clone()).or_default();
        let first = m.reviews == 0;
        let elapsed_days = if first {
            0.0
        } else {
            now.saturating_sub(m.last).max(0) as f64 / 86400.0
        };
        let effective = if assisted && matches!(grade, Grade::Good | Grade::Easy) {
            Grade::Hard
        } else {
            grade
        };
        m.apply(effective, assisted, now);
        self.reviews.push(Review {
            key,
            at: now,
            date: date.into(),
            grade: effective,
            assisted,
            method: method.into(),
            first,
            elapsed_days,
            seconds,
            self_assessed,
        });
    }
    pub fn observed_retention(&self, days: f64) -> Option<(usize, usize)> {
        let relevant: Vec<_> = self
            .reviews
            .iter()
            .filter(|r| r.elapsed_days >= days && !r.assisted && !r.first)
            .collect();
        let total = relevant.len();
        if total == 0 {
            None
        } else {
            Some((
                relevant
                    .iter()
                    .filter(|r| matches!(r.grade, Grade::Good | Grade::Easy))
                    .count(),
                total,
            ))
        }
    }
}

pub struct Storage {
    pub dir: PathBuf,
    _lock: File,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HistoryCommitOutcome {
    Committed(crate::audio_history::CommittedHistory),
    CancelledBeforeCommit,
}

impl Storage {
    pub fn commit_history(
        &self,
        published: &crate::audio_history::PublishedHistory,
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<HistoryCommitOutcome, HistoryError> {
        self.commit_history_with_limit_and_faults(
            published,
            cancel,
            crate::commit::PROGRESS_LIMIT,
            &NoProgressFaults,
        )
    }

    fn commit_history_with_limit_and_faults(
        &self,
        published: &crate::audio_history::PublishedHistory,
        cancel: Option<&std::sync::atomic::AtomicBool>,
        limit: u64,
        faults: &dyn ProgressFaults,
    ) -> Result<HistoryCommitOutcome, HistoryError> {
        self.commit_history_with_publisher(published, cancel, limit, faults, atomic_write)
    }

    fn commit_history_with_publisher<P>(
        &self,
        published: &crate::audio_history::PublishedHistory,
        cancel: Option<&std::sync::atomic::AtomicBool>,
        limit: u64,
        faults: &dyn ProgressFaults,
        publish: P,
    ) -> Result<HistoryCommitOutcome, HistoryError>
    where
        P: FnOnce(&Path, &[u8]) -> Result<(), String>,
    {
        use crate::audio_history::HistoryWriter;

        let guard = acquire_progress_write().map_err(history_commit_gate_error)?;
        history_commit_pending(&self.dir)?;
        let old_bytes =
            read_progress_bytes(&self.dir, limit).map_err(ProgressReadError::history_error)?;
        let mut next = old_bytes
            .as_deref()
            .map(decode_progress)
            .transpose()
            .map_err(ProgressReadError::history_error)?
            .map(|decoded| decoded.progress)
            .unwrap_or_default();

        // A completed operation wins over cancellation, including after intent cleanup.
        match HistoryWriter::confirm_publication(self, published) {
            Ok(committed) => {
                self.history_commit_snapshot_unchanged(old_bytes.as_deref(), limit)?;
                if next.audio_history.as_ref() != Some(&committed.current) {
                    return Err(HistoryError {
                        kind: HistoryErrorKind::Conflict,
                    });
                }
                return Ok(HistoryCommitOutcome::Committed(committed));
            }
            Err(error) if error.kind == HistoryErrorKind::Busy => {}
            Err(error) => return Err(error),
        }
        if next.audio_history != published.base {
            return Err(HistoryError {
                kind: HistoryErrorKind::Conflict,
            });
        }
        // Busy is only a candidate: strict intent and exact mutation authority remain mandatory.
        HistoryWriter::verify_publication_under_guard(self, published, &guard)?;
        self.history_commit_snapshot_unchanged(old_bytes.as_deref(), limit)?;
        next.audio_history = Some(published.next.clone());
        next.validate().map_err(|_| HistoryError {
            kind: HistoryErrorKind::InvalidDocument,
        })?;
        let next_bytes = serde_json::to_vec_pretty(&next).map_err(|_| HistoryError {
            kind: HistoryErrorKind::InvalidDocument,
        })?;
        if next_bytes.len() as u64 > limit {
            return Err(HistoryError {
                kind: HistoryErrorKind::LimitExceeded,
            });
        }
        preserve_v1_before_v2_under_guard(
            &self.dir,
            old_bytes.as_deref(),
            2,
            limit,
            &guard,
            faults,
        )
        .map_err(history_commit_migration_error)?;

        if faults.check(ProgressCheckpoint::BeforePointer).is_err() {
            return self.history_commit_readback(
                published,
                old_bytes.as_deref(),
                &next_bytes,
                limit,
                &guard,
                true,
            );
        }
        self.history_commit_snapshot_unchanged(old_bytes.as_deref(), limit)?;
        guard.ensure_writable().map_err(history_commit_gate_error)?;
        let path = self.dir.join("progress.json");
        // No callback, wait or further validation may intervene after this final acquire load.
        if cancel.is_some_and(|cancel| cancel.load(Ordering::Acquire)) {
            return Ok(HistoryCommitOutcome::CancelledBeforeCommit);
        }
        let publish_failed = publish(&path, &next_bytes).is_err();
        let after_failed = faults.check(ProgressCheckpoint::AfterPointer).is_err();
        self.history_commit_readback(
            published,
            old_bytes.as_deref(),
            &next_bytes,
            limit,
            &guard,
            publish_failed || after_failed,
        )
    }

    fn history_commit_snapshot_unchanged(
        &self,
        old_bytes: Option<&[u8]>,
        limit: u64,
    ) -> Result<(), HistoryError> {
        history_commit_pending(&self.dir)?;
        let current =
            read_progress_bytes(&self.dir, limit).map_err(ProgressReadError::history_error)?;
        if current.as_deref() != old_bytes {
            return Err(HistoryError {
                kind: HistoryErrorKind::Conflict,
            });
        }
        Ok(())
    }

    fn history_commit_readback(
        &self,
        published: &crate::audio_history::PublishedHistory,
        old_bytes: Option<&[u8]>,
        next_bytes: &[u8],
        limit: u64,
        guard: &ProgressWriteGuard,
        failed: bool,
    ) -> Result<HistoryCommitOutcome, HistoryError> {
        // Any failure to prove exact old or exact new is an unknown outcome, never a rollback.
        let observed = (|| {
            history_commit_pending(&self.dir)?;
            let actual =
                read_progress_bytes(&self.dir, limit).map_err(ProgressReadError::history_error)?;
            let decoded = actual
                .as_deref()
                .map(decode_progress)
                .transpose()
                .map_err(ProgressReadError::history_error)?;
            if actual.as_deref() == Some(next_bytes) {
                let committed =
                    crate::audio_history::HistoryWriter::confirm_publication(self, published)?;
                if committed.current != published.next {
                    return Err(HistoryError {
                        kind: HistoryErrorKind::Conflict,
                    });
                }
                self.history_commit_snapshot_unchanged(Some(next_bytes), limit)?;
                Ok(Some(committed))
            } else if actual.as_deref() == old_bytes {
                if let Some(root) = decoded.and_then(|decoded| decoded.progress.audio_history) {
                    crate::audio_history::HistoryReader::with_limits(
                        self,
                        crate::audio_history::HistoryLimits {
                            max_nodes: 100_000,
                            max_total_bytes: u64::MAX,
                        },
                    )?
                    .verify_closure(&root)?;
                }
                self.history_commit_snapshot_unchanged(old_bytes, limit)?;
                Ok(None)
            } else {
                Err(HistoryError {
                    kind: HistoryErrorKind::Conflict,
                })
            }
        })();
        match observed {
            Ok(Some(committed)) => Ok(HistoryCommitOutcome::Committed(committed)),
            Ok(None) => Err(HistoryError {
                kind: if failed {
                    HistoryErrorKind::Io
                } else {
                    HistoryErrorKind::Busy
                },
            }),
            Err(_) => {
                guard.block_writes();
                Err(HistoryError {
                    kind: HistoryErrorKind::Io,
                })
            }
        }
    }
}

fn history_commit_gate_error(error: ProgressGateError) -> HistoryError {
    HistoryError {
        kind: match error {
            ProgressGateError::Reentrant => HistoryErrorKind::Busy,
            ProgressGateError::Poisoned | ProgressGateError::WriteBlocked => HistoryErrorKind::Io,
        },
    }
}

fn history_commit_migration_error(error: MigrationError) -> HistoryError {
    HistoryError {
        kind: match error {
            MigrationError::Gate(error) => return history_commit_gate_error(error),
            MigrationError::InvalidDocument => HistoryErrorKind::InvalidDocument,
            MigrationError::UnsupportedVersion => HistoryErrorKind::UnsupportedVersion,
            MigrationError::LimitExceeded => HistoryErrorKind::LimitExceeded,
            MigrationError::UnsafePath => HistoryErrorKind::UnsafePath,
            MigrationError::Conflict => HistoryErrorKind::Conflict,
            MigrationError::Io => HistoryErrorKind::Io,
        },
    }
}

fn history_commit_pending(root: &Path) -> Result<(), HistoryError> {
    let pending = crate::commit::pending_checked(root).map_err(|error| HistoryError {
        kind: match error {
            crate::commit::PendingCheckError::UnsafePath => HistoryErrorKind::UnsafePath,
            crate::commit::PendingCheckError::Io => HistoryErrorKind::Io,
        },
    })?;
    if pending {
        return Err(HistoryError {
            kind: HistoryErrorKind::PendingMaterial,
        });
    }
    Ok(())
}

impl Storage {
    pub fn save_progress(&self, candidate: &Progress) -> Result<Progress, String> {
        self.save_progress_with_limit_and_faults(
            candidate,
            crate::commit::PROGRESS_LIMIT,
            &NoProgressFaults,
        )
    }

    fn save_progress_with_limit_and_faults(
        &self,
        candidate: &Progress,
        limit: u64,
        faults: &dyn ProgressFaults,
    ) -> Result<Progress, String> {
        self.save_progress_with_publisher(candidate, limit, faults, atomic_write)
    }

    fn save_progress_with_publisher<P>(
        &self,
        candidate: &Progress,
        limit: u64,
        faults: &dyn ProgressFaults,
        publish: P,
    ) -> Result<Progress, String>
    where
        P: FnOnce(&Path, &[u8]) -> Result<(), String>,
    {
        self.save_progress_with_publishers(candidate, limit, faults, publish, atomic_write)
    }

    fn save_progress_with_publishers<P, D>(
        &self,
        candidate: &Progress,
        limit: u64,
        faults: &dyn ProgressFaults,
        publish: P,
        publish_daily: D,
    ) -> Result<Progress, String>
    where
        P: FnOnce(&Path, &[u8]) -> Result<(), String>,
        D: FnOnce(&Path, &[u8]) -> Result<(), String>,
    {
        let guard = acquire_progress_write().map_err(ProgressGateError::message)?;
        if crate::commit::pending_checked(&self.dir)
            .map_err(crate::commit::PendingCheckError::message)?
        {
            return Err(
                "教材の保存途中です。ほかの記録を上書きせず再起動して復旧してください。".into(),
            );
        }
        // Decode the same raw snapshot used by migration, daily backup and readback.
        let old_bytes =
            read_progress_bytes(&self.dir, limit).map_err(|error| error.message().to_owned())?;
        let old = old_bytes
            .as_deref()
            .map(decode_progress)
            .transpose()
            .map_err(|error| error.message().to_owned())?;
        let latest_root = old.and_then(|decoded| decoded.progress.audio_history);
        if let Some(root) = &latest_root {
            crate::audio_history::HistoryReader::new(self)
                .verify_closure(root)
                .map_err(|_| {
                    "現在の音声履歴を確認できません。学習記録は変更していません。".to_owned()
                })?;
        }
        let mut next = candidate.clone();
        next.audio_history = latest_root;
        next.validate()?;
        let next_bytes = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
        if next_bytes.len() as u64 > limit {
            return Err("学習記録が保存上限を超えています。元ファイルは変更していません。".into());
        }
        preserve_v1_before_v2_under_guard(
            &self.dir,
            old_bytes.as_deref(),
            next.version,
            limit,
            &guard,
            faults,
        )
        .map_err(MigrationError::message)?;
        if let Some(bytes) = &old_bytes {
            preserve_daily_backup_with_publisher(&self.dir, bytes, publish_daily)?;
        }
        guard
            .ensure_writable()
            .map_err(ProgressGateError::message)?;
        // A publisher error may follow a successful replace; its text is not evidence.
        let _published = publish(&self.dir.join("progress.json"), &next_bytes);
        match read_progress_bytes(&self.dir, limit) {
            Ok(actual) if actual.as_deref() == Some(next_bytes.as_slice()) => Ok(next),
            Ok(actual) if actual == old_bytes => {
                Err("学習記録は保存されていません。元の記録を保持しています。".into())
            }
            _ => {
                guard.block_writes();
                Err("保存結果を確認できません。回復確認が必要なため書き込みを停止しました。".into())
            }
        }
    }

    pub fn read_progress(&self) -> Result<Progress, String> {
        read_progress_typed(&self.dir, crate::commit::PROGRESS_LIMIT)
            .map(|decoded| decoded.map(|decoded| decoded.progress).unwrap_or_default())
            .map_err(|error| error.message().to_owned())
    }

    pub fn open() -> Result<Self, String> {
        Self::at(resolve_storage_root()?)
    }
    pub fn at(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(dir.join("app.lock"))
            .map_err(|e| e.to_string())?;
        fs2::FileExt::try_lock_exclusive(&lock)
            .map_err(|_| "ほかのWordWeave 5が起動中です。閉じてから再起動してください。")?;
        Ok(Self { dir, _lock: lock })
    }
    pub fn load(&self) -> Result<Progress, String> {
        let guard = acquire_progress_write().map_err(ProgressGateError::message)?;
        crate::commit::recover_under_guard(&self.dir, &guard)?;
        self.read_progress()
    }
    pub fn save(&self, p: &Progress) -> Result<(), String> {
        self.save_with_limit(p, crate::commit::PROGRESS_LIMIT)
    }

    fn save_with_limit(&self, p: &Progress, limit: u64) -> Result<(), String> {
        self.save_with_limit_and_faults(p, limit, &NoProgressFaults)
    }

    fn save_with_limit_and_faults(
        &self,
        p: &Progress,
        limit: u64,
        faults: &dyn ProgressFaults,
    ) -> Result<(), String> {
        self.save_progress_with_limit_and_faults(p, limit, faults)
            .map(|_| ())
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    atomic_write_with_temp(path, bytes, &atomic_temp_path(path))
}

fn atomic_temp_path(path: &Path) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.with_extension(format!("tmp-{}-{stamp}", std::process::id()))
}

fn atomic_write_with_temp(path: &Path, bytes: &[u8], temp: &Path) -> Result<(), String> {
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e: std::io::Error| e.to_string())
}

pub fn resolve_storage_root() -> Result<PathBuf, String> {
    let root = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATAが見つかりません。")?;
    Ok(Path::new(&root).join("WordWeave5"))
}

pub fn write_external_file(root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), String> {
    let guard = acquire_external_export().map_err(ProgressGateError::message)?;
    write_external_file_under_guard(root, dest, bytes, &guard)
}

pub fn create_external_directory(root: &Path, dest: &Path) -> Result<(), String> {
    let guard = acquire_external_export().map_err(ProgressGateError::message)?;
    create_external_directory_under_guard(root, dest, &guard).map(|_| ())
}

#[derive(Debug)]
pub(crate) struct ExternalExportGuard {
    _guard: ProgressWriteGuard,
}

pub(crate) fn acquire_external_export() -> Result<ExternalExportGuard, ProgressGateError> {
    Ok(ExternalExportGuard {
        _guard: acquire_progress_exclusion()?,
    })
}

pub(crate) fn write_external_file_under_guard(
    root: &Path,
    dest: &Path,
    bytes: &[u8],
    guard: &ExternalExportGuard,
) -> Result<(), String> {
    let checked = prepare_external_destination(root, dest, ExternalDestinationKind::File, guard)
        .map_err(ExternalDestinationError::message)?;
    let temp = atomic_temp_path(&checked.destination);
    publish_external_file(&checked, bytes, &temp)
}

pub(crate) fn create_external_directory_under_guard(
    root: &Path,
    dest: &Path,
    guard: &ExternalExportGuard,
) -> Result<PathBuf, String> {
    let checked =
        prepare_external_destination(root, dest, ExternalDestinationKind::Directory, guard)
            .map_err(ExternalDestinationError::message)?;
    create_checked_external_directory(&checked)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExternalDestinationKind {
    File,
    Directory,
}

#[derive(Debug)]
enum ExternalDestinationError {
    UnsafePath,
    ProtectedRoot,
    IdentityUnavailable,
    Io,
    Changed,
    AlreadyExists,
}

impl ExternalDestinationError {
    fn message(self) -> String {
        match self {
            Self::Changed => "保存先が変更されました。別の外部保存先を選んでください。",
            Self::AlreadyExists => "保存先が既に存在します。別の外部保存先を選んでください。",
            _ => "安全な保存先を確認できません。別の外部保存先を選んでください。",
        }
        .into()
    }
}

struct CheckedExternalDestination<'g> {
    _guard: &'g ExternalExportGuard,
    kind: ExternalDestinationKind,
    destination: PathBuf,
    #[cfg(windows)]
    state: external_destination_windows::CheckedState,
}

fn prepare_external_destination<'g>(
    root: &Path,
    dest: &Path,
    kind: ExternalDestinationKind,
    guard: &'g ExternalExportGuard,
) -> Result<CheckedExternalDestination<'g>, ExternalDestinationError> {
    #[cfg(windows)]
    {
        let cwd = std::env::current_dir().map_err(|_| ExternalDestinationError::Io)?;
        external_destination_windows::prepare(root, dest, kind, guard, &cwd)
    }
    #[cfg(not(windows))]
    {
        let _ = (root, dest, kind, guard);
        Err(ExternalDestinationError::IdentityUnavailable)
    }
}

fn recheck_external_destination(
    checked: &CheckedExternalDestination<'_>,
) -> Result<(), ExternalDestinationError> {
    #[cfg(windows)]
    {
        external_destination_windows::recheck(&checked.state)
    }
    #[cfg(not(windows))]
    {
        let _ = checked;
        Err(ExternalDestinationError::IdentityUnavailable)
    }
}

fn publish_external_file(
    checked: &CheckedExternalDestination<'_>,
    bytes: &[u8],
    temp: &Path,
) -> Result<(), String> {
    if checked.kind != ExternalDestinationKind::File {
        return Err(ExternalDestinationError::UnsafePath.message());
    }
    #[cfg(windows)]
    {
        let checked_temp = external_destination_windows::prepare(
            &checked.state.root,
            temp,
            ExternalDestinationKind::File,
            checked._guard,
            &checked.state.cwd,
        )
        .map_err(ExternalDestinationError::message)?;
        if checked_temp.state.leaf_present
            || checked_temp.state.parent_identity != checked.state.parent_identity
            || external_destination_windows::same_planned_name(
                checked_temp
                    .destination
                    .file_name()
                    .ok_or_else(|| ExternalDestinationError::UnsafePath.message())?,
                checked
                    .destination
                    .file_name()
                    .ok_or_else(|| ExternalDestinationError::UnsafePath.message())?,
            )
            .map_err(ExternalDestinationError::message)?
        {
            return Err(ExternalDestinationError::UnsafePath.message());
        }
        recheck_external_destination(&checked_temp).map_err(ExternalDestinationError::message)?;
        recheck_external_destination(checked).map_err(ExternalDestinationError::message)?;
        atomic_write_with_temp(&checked.destination, bytes, &checked_temp.destination)
    }
    #[cfg(not(windows))]
    {
        let _ = (bytes, temp);
        Err(ExternalDestinationError::IdentityUnavailable.message())
    }
}

fn create_checked_external_directory(
    checked: &CheckedExternalDestination<'_>,
) -> Result<PathBuf, String> {
    if checked.kind != ExternalDestinationKind::Directory {
        return Err(ExternalDestinationError::UnsafePath.message());
    }
    recheck_external_destination(checked).map_err(ExternalDestinationError::message)?;
    fs::create_dir(&checked.destination).map_err(|error| error.to_string())?;
    Ok(checked.destination.clone())
}

#[cfg(windows)]
mod external_destination_windows {
    use super::*;
    use std::{
        ffi::{c_void, OsStr, OsString},
        os::windows::{
            ffi::{OsStrExt, OsStringExt},
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawHandle,
        },
        path::{Component, Prefix},
    };

    type Error = ExternalDestinationError;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) struct FileIdentity {
        volume_serial: u64,
        file_id: [u8; 16],
    }

    #[repr(C)]
    struct HandleInformation {
        attributes: u32,
        creation_time: [u32; 2],
        access_time: [u32; 2],
        write_time: [u32; 2],
        volume_serial: u32,
        size_high: u32,
        size_low: u32,
        links: u32,
        index_high: u32,
        index_low: u32,
    }

    const _: () = assert!(std::mem::size_of::<FileIdentity>() == 24);
    const _: () = assert!(std::mem::align_of::<FileIdentity>() == 8);
    const _: () = assert!(std::mem::size_of::<HandleInformation>() == 52);

    #[link(name = "kernel32")]
    extern "system" {
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: i32,
            information: *mut c_void,
            size: u32,
        ) -> i32;
        fn GetFileInformationByHandle(
            handle: *mut c_void,
            information: *mut HandleInformation,
        ) -> i32;
        fn GetFileType(handle: *mut c_void) -> u32;
        fn GetFinalPathNameByHandleW(
            handle: *mut c_void,
            path: *mut u16,
            capacity: u32,
            flags: u32,
        ) -> u32;
        fn CompareStringOrdinal(
            first: *const u16,
            first_length: i32,
            second: *const u16,
            second_length: i32,
            ignore_case: i32,
        ) -> i32;
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Fingerprint {
        identity: FileIdentity,
        directory: bool,
        links: u32,
    }

    struct Observation {
        path: PathBuf,
        fingerprint: Option<Fingerprint>,
        // Keeping the handle alive also prevents recycling an observed file ID.
        handle: Option<File>,
    }

    enum ProtectedRoot {
        Existing(FileIdentity),
        Planned {
            ancestor: FileIdentity,
            suffix: Vec<OsString>,
        },
    }

    pub(super) struct CheckedState {
        pub(super) root: PathBuf,
        pub(super) cwd: PathBuf,
        pub(super) parent_identity: FileIdentity,
        pub(super) leaf_present: bool,
        observations: Vec<Observation>,
    }

    fn observe(path: &Path) -> Result<Observation, Error> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Observation {
                    path: path.to_owned(),
                    fingerprint: None,
                    handle: None,
                });
            }
            Err(_) => return Err(Error::Io),
        };
        if metadata.file_attributes() & 0x400 != 0
            || metadata.file_type().is_symlink()
            || (!metadata.is_dir() && !metadata.is_file())
        {
            return Err(Error::UnsafePath);
        }
        let handle = OpenOptions::new()
            .access_mode(0x80) // FILE_READ_ATTRIBUTES, without a content-read request.
            .share_mode(7) // READ | WRITE | DELETE, including during replacement.
            .custom_flags(0x0020_0000 | 0x0200_0000) // OPEN_REPARSE_POINT | BACKUP_SEMANTICS
            .open(path)
            .map_err(|_| Error::Io)?;
        let mut information: HandleInformation = unsafe { std::mem::zeroed() };
        let mut identity = FileIdentity {
            volume_serial: 0,
            file_id: [0; 16],
        };
        // SAFETY: the live File owns the handle; both repr(C) output buffers have
        // the Windows ABI layout/size asserted above. FileIdInfo is class 18.
        unsafe {
            if GetFileType(handle.as_raw_handle()) != 1 {
                return Err(Error::UnsafePath);
            }
            if GetFileInformationByHandle(handle.as_raw_handle(), &mut information) == 0
                || GetFileInformationByHandleEx(
                    handle.as_raw_handle(),
                    18,
                    (&mut identity as *mut FileIdentity).cast(),
                    std::mem::size_of::<FileIdentity>() as u32,
                ) == 0
            {
                return Err(Error::IdentityUnavailable);
            }
        }
        if information.attributes & 0x400 != 0 {
            return Err(Error::UnsafePath);
        }
        Ok(Observation {
            path: path.to_owned(),
            fingerprint: Some(Fingerprint {
                identity,
                directory: information.attributes & 0x10 != 0,
                links: information.links,
            }),
            handle: Some(handle),
        })
    }

    fn final_path(handle: &File) -> Result<PathBuf, Error> {
        let mut buffer = vec![0u16; 260];
        loop {
            // SAFETY: handle is live and buffer exposes capacity writable u16s.
            // FILE_NAME_NORMALIZED (0) | VOLUME_NAME_GUID (1) is mandatory.
            let length = unsafe {
                GetFinalPathNameByHandleW(
                    handle.as_raw_handle(),
                    buffer.as_mut_ptr(),
                    buffer.len() as u32,
                    1,
                )
            } as usize;
            if length == 0 || length > 32768 {
                return Err(Error::IdentityUnavailable);
            }
            if length >= buffer.len() {
                buffer.resize(length + 1, 0);
                continue;
            }
            let path = PathBuf::from(OsString::from_wide(&buffer[..length]));
            if !matches!(path.components().next(), Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), Prefix::Verbatim(name) if volume_guid(name)))
            {
                return Err(Error::IdentityUnavailable);
            }
            return Ok(path);
        }
    }

    fn volume_guid(name: &OsStr) -> bool {
        let Some(name) = name.to_str() else {
            return false;
        };
        let bytes = name.as_bytes();
        bytes.len() == 44
            && bytes[..7].eq_ignore_ascii_case(b"Volume{")
            && bytes[43] == b'}'
            && bytes[7..43].iter().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    *byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
    }

    fn validate_name(name: &OsStr) -> Result<(), Error> {
        let units: Vec<u16> = name.encode_wide().collect();
        if units == [46] || units == [46, 46] {
            return Ok(());
        }
        if units.is_empty()
            || matches!(units.last(), Some(32 | 46))
            || units.iter().any(|unit| {
                *unit < 32
                    || [b':', b'/', b'\\', b'<', b'>', b'"', b'|', b'?', b'*']
                        .iter()
                        .any(|byte| *unit == u16::from(*byte))
            })
        {
            return Err(Error::UnsafePath);
        }
        let stem: Vec<u16> = units
            .iter()
            .take_while(|unit| **unit != 46)
            .map(|unit| {
                if (97..=122).contains(unit) {
                    *unit - 32
                } else {
                    *unit
                }
            })
            .collect();
        let reserved = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]
            .iter()
            .any(|name| stem.iter().copied().eq(name.encode_utf16()));
        let numbered = stem.len() == 4
            && (stem[..3] == [67, 79, 77] || stem[..3] == [76, 80, 84])
            && matches!(stem[3], 49..=57 | 0xb9 | 0xb2 | 0xb3);
        if reserved || numbered {
            return Err(Error::UnsafePath);
        }
        Ok(())
    }

    fn validate_path(path: &Path) -> Result<(), Error> {
        if path.as_os_str().is_empty() || path.as_os_str().encode_wide().any(|unit| unit == 0) {
            return Err(Error::UnsafePath);
        }
        for component in path.components() {
            match component {
                Component::Prefix(prefix) => match prefix.kind() {
                    Prefix::Disk(_) | Prefix::VerbatimDisk(_) if path.has_root() => {}
                    Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                        validate_name(server)?;
                        validate_name(share)?;
                    }
                    Prefix::Verbatim(name) if volume_guid(name) && path.has_root() => {}
                    _ => return Err(Error::UnsafePath),
                },
                Component::Normal(name) => validate_name(name)?,
                _ => {}
            }
        }
        Ok(())
    }

    // Observe each existing component before processing '..'. Canonicalizing
    // first would hide a junction in, for example, junction/../safe-output.
    fn walk(
        absolute: &Path,
        leaf_directory: bool,
        allow_missing: bool,
        observations: &mut Vec<Observation>,
    ) -> Result<PathBuf, Error> {
        let mut path = PathBuf::new();
        let mut parts = absolute.components().peekable();
        while let Some(component) = parts.next() {
            match component {
                Component::Prefix(prefix) => {
                    path.push(prefix.as_os_str());
                    continue;
                }
                Component::RootDir => path.push(component.as_os_str()),
                Component::CurDir => continue,
                Component::ParentDir => {
                    // The preceding component was inspected as a directory.
                    if observations
                        .last()
                        .and_then(|value| value.fingerprint)
                        .is_none()
                    {
                        return Err(Error::UnsafePath);
                    }
                    path.pop();
                }
                Component::Normal(name) if name == OsStr::new(".") => continue,
                Component::Normal(name) if name == OsStr::new("..") => {
                    if observations
                        .last()
                        .and_then(|value| value.fingerprint)
                        .is_none()
                    {
                        return Err(Error::UnsafePath);
                    }
                    path.pop();
                }
                Component::Normal(name) => path.push(name),
            }
            let observed = observe(&path)?;
            let require_directory = leaf_directory || parts.peek().is_some();
            match observed.fingerprint {
                Some(value) if require_directory && !value.directory => {
                    return Err(Error::UnsafePath);
                }
                None if !allow_missing && parts.peek().is_some() => return Err(Error::Io),
                _ => {}
            }
            observations.push(observed);
        }
        Ok(path)
    }

    pub(super) fn same_planned_name(first: &OsStr, second: &OsStr) -> Result<bool, Error> {
        let first: Vec<u16> = first.encode_wide().collect();
        let second: Vec<u16> = second.encode_wide().collect();
        let first_len = i32::try_from(first.len()).map_err(|_| Error::UnsafePath)?;
        let second_len = i32::try_from(second.len()).map_err(|_| Error::UnsafePath)?;
        // SAFETY: these slices remain live for their explicitly supplied lengths.
        match unsafe {
            CompareStringOrdinal(first.as_ptr(), first_len, second.as_ptr(), second_len, 1)
        } {
            0 => Err(Error::IdentityUnavailable),
            2 => Ok(true),
            _ => Ok(false),
        }
    }

    fn protected_root(root: &Path, observations: &[Observation]) -> Result<ProtectedRoot, Error> {
        let mut suffix = Vec::new();
        for path in root.ancestors() {
            let observed = observations
                .iter()
                .rev()
                .find(|value| value.path == path)
                .ok_or(Error::IdentityUnavailable)?;
            if let Some(value) = observed.fingerprint {
                // Require GUID support for the protected anchor as well.
                final_path(observed.handle.as_ref().ok_or(Error::IdentityUnavailable)?)?;
                if suffix.is_empty() {
                    return Ok(ProtectedRoot::Existing(value.identity));
                }
                suffix.reverse();
                return Ok(ProtectedRoot::Planned {
                    ancestor: value.identity,
                    suffix,
                });
            }
            suffix.push(path.file_name().ok_or(Error::UnsafePath)?.to_owned());
        }
        Err(Error::IdentityUnavailable)
    }

    fn check_region(
        protected: &ProtectedRoot,
        destination: &Path,
        ancestors: &[Observation],
    ) -> Result<(), Error> {
        for observed in ancestors {
            let identity = observed
                .fingerprint
                .ok_or(Error::IdentityUnavailable)?
                .identity;
            match protected {
                ProtectedRoot::Existing(root) if identity == *root => {
                    return Err(Error::ProtectedRoot);
                }
                ProtectedRoot::Planned { ancestor, suffix } if identity == *ancestor => {
                    let relative = destination
                        .strip_prefix(&observed.path)
                        .map_err(|_| Error::IdentityUnavailable)?;
                    let names: Vec<_> =
                        relative.components().map(|part| part.as_os_str()).collect();
                    let mut common = true;
                    for (first, second) in suffix.iter().zip(&names) {
                        if !same_planned_name(first, second)? {
                            common = false;
                            break;
                        }
                    }
                    // Reject both descendants and uncreated ancestors of the planned root.
                    if common {
                        return Err(Error::ProtectedRoot);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn prepare<'g>(
        root: &Path,
        dest: &Path,
        kind: ExternalDestinationKind,
        guard: &'g ExternalExportGuard,
        cwd: &Path,
    ) -> Result<CheckedExternalDestination<'g>, Error> {
        validate_path(root)?;
        validate_path(dest)?;
        let absolute_root = cwd.join(root);
        let absolute_dest = cwd.join(dest);
        validate_path(&absolute_root)?;
        validate_path(&absolute_dest)?;
        let mut observations = Vec::new();
        let normalized_root = walk(&absolute_root, true, true, &mut observations)?;
        let protected = protected_root(&normalized_root, &observations)?;
        let normalized_dest = walk(&absolute_dest, false, false, &mut observations)?;
        let leaf = observations.last().ok_or(Error::UnsafePath)?.fingerprint;
        match (kind, leaf) {
            (ExternalDestinationKind::Directory, Some(_)) => return Err(Error::AlreadyExists),
            (ExternalDestinationKind::File, Some(value)) if value.directory || value.links != 1 => {
                return Err(Error::UnsafePath);
            }
            _ => {}
        }
        let parent = normalized_dest.parent().ok_or(Error::UnsafePath)?;
        let parent_observed = observations
            .iter()
            .rev()
            .find(|value| value.path == parent)
            .ok_or(Error::IdentityUnavailable)?;
        let parent_identity = parent_observed
            .fingerprint
            .filter(|value| value.directory)
            .ok_or(Error::Io)?
            .identity;
        let resolved_parent = final_path(parent_observed.handle.as_ref().ok_or(Error::Io)?)?;
        let destination =
            resolved_parent.join(normalized_dest.file_name().ok_or(Error::UnsafePath)?);
        let mut resolved_ancestors = Vec::new();
        for ancestor in resolved_parent.ancestors() {
            let observed = observe(ancestor)?;
            if !observed.fingerprint.is_some_and(|value| value.directory) {
                return Err(Error::IdentityUnavailable);
            }
            resolved_ancestors.push(observed);
        }
        if resolved_ancestors
            .first()
            .and_then(|value| value.fingerprint)
            .map(|value| value.identity)
            != Some(parent_identity)
        {
            return Err(Error::Changed);
        }
        check_region(&protected, &destination, &resolved_ancestors)?;
        observations.extend(resolved_ancestors);
        let resolved_leaf = observe(&destination)?;
        if resolved_leaf.fingerprint != leaf {
            return Err(Error::Changed);
        }
        observations.push(resolved_leaf);
        Ok(CheckedExternalDestination {
            _guard: guard,
            kind,
            destination,
            state: CheckedState {
                // Preserve the original traversal (including '..') for temp preparation.
                root: absolute_root,
                cwd: cwd.to_owned(),
                parent_identity,
                leaf_present: leaf.is_some(),
                observations,
            },
        })
    }

    pub(super) fn recheck(state: &CheckedState) -> Result<(), Error> {
        for prior in &state.observations {
            let now = observe(&prior.path).map_err(|error| match error {
                Error::UnsafePath => Error::Changed,
                other => other,
            })?;
            if now.fingerprint != prior.fingerprint {
                return Err(Error::Changed);
            }
        }
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod i2_export_destination_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};
    use std::{
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        process::{Command, Stdio},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    const PAYLOAD: &[u8] = "外部退避の未保存内容\n".as_bytes();
    const CHILD: &str = "WW5_EXPORT_LIB_CHILD";

    fn isolated(name: &str, body: impl FnOnce()) {
        let case = format!("store::i2_export_destination_tests::{name}");
        if std::env::var(CHILD).as_deref() == Ok(case.as_str()) {
            body();
            println!("WW5_H_LIB_CHILD_OK:{case}");
            return;
        }
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([case.as_str(), "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, &case)
            .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut timeout = false;
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                timeout = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "WW5_H_LIB_PARENT:{case}:exit={}:timeout={timeout}\n{stdout}\n{stderr}",
            output.status
        );
        assert!(!timeout, "child killed/reaped after 20 seconds");
        assert!(output.status.success(), "child behavior/fixture failure");
        assert!(
            stdout.contains("running 1 test")
                && stdout.contains("1 passed; 0 failed")
                && stdout.contains(&format!("WW5_H_LIB_CHILD_OK:{case}"))
        );
    }

    fn paths(temp: &Temp) -> (PathBuf, PathBuf) {
        let root = temp.root.join("host");
        let outside = temp.root.join("external");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(root.join("progress.json"), b"protected exact bytes").unwrap();
        fs::write(root.join("pending-material-commit.json"), b"do not recover").unwrap();
        (root, outside)
    }

    #[test]
    fn h05_write_block_allows_external_file_directory_and_keeps_normal_block() {
        isolated(
            "h05_write_block_allows_external_file_directory_and_keeps_normal_block",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let protected = tree(&root);
                let guard = acquire_progress_write().unwrap();
                guard.block_writes();
                drop(guard);
                write_external_file(&root, &outside.join("saved.txt"), PAYLOAD).unwrap();
                create_external_directory(&root, &outside.join("bundle")).unwrap();
                assert_eq!(fs::read(outside.join("saved.txt")).unwrap(), PAYLOAD);
                assert!(outside.join("bundle").is_dir());
                assert_eq!(tree(&root), protected);
                for _ in 0..2 {
                    assert!(matches!(
                        acquire_progress_write(),
                        Err(ProgressGateError::WriteBlocked)
                    ));
                }
                let ext = acquire_external_export().unwrap();
                write_external_file_under_guard(&root, &outside.join("second.txt"), PAYLOAD, &ext)
                    .unwrap();
                drop(ext);
                assert!(matches!(
                    acquire_progress_write(),
                    Err(ProgressGateError::WriteBlocked)
                ));
            },
        );
    }

    #[test]
    fn h05_poison_refuses_both_gate_entries_and_public_export_without_mutation() {
        isolated(
            "h05_poison_refuses_both_gate_entries_and_public_export_without_mutation",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let before = tree(&temp.root);
                assert!(std::panic::catch_unwind(|| {
                    let _guard = acquire_progress_write().unwrap();
                    panic!("intentional isolated gate poison");
                })
                .is_err());
                for _ in 0..2 {
                    assert!(matches!(
                        acquire_progress_write(),
                        Err(ProgressGateError::Poisoned)
                    ));
                    assert!(matches!(
                        acquire_external_export(),
                        Err(ProgressGateError::Poisoned)
                    ));
                    assert!(write_external_file(&root, &outside.join("file"), PAYLOAD).is_err());
                    assert!(create_external_directory(&root, &outside.join("directory")).is_err());
                }
                assert_eq!(tree(&temp.root), before);
            },
        );
    }

    #[test]
    fn h05_external_guard_panic_poison_is_shared_with_normal_writer() {
        isolated(
            "h05_external_guard_panic_poison_is_shared_with_normal_writer",
            || {
                assert!(std::panic::catch_unwind(|| {
                    let _guard = acquire_external_export().unwrap();
                    panic!("intentional isolated external poison");
                })
                .is_err());
                assert!(matches!(
                    acquire_external_export(),
                    Err(ProgressGateError::Poisoned)
                ));
                assert!(matches!(
                    acquire_progress_write(),
                    Err(ProgressGateError::Poisoned)
                ));
            },
        );
    }

    #[test]
    fn h05_reentry_both_directions_is_bounded_and_drop_restores_ownership() {
        isolated(
            "h05_reentry_both_directions_is_bounded_and_drop_restores_ownership",
            || {
                let normal = acquire_progress_write().unwrap();
                assert!(matches!(
                    acquire_external_export(),
                    Err(ProgressGateError::Reentrant)
                ));
                drop(normal);
                let external = acquire_external_export().unwrap();
                assert!(matches!(
                    acquire_progress_write(),
                    Err(ProgressGateError::Reentrant)
                ));
                assert!(matches!(
                    acquire_external_export(),
                    Err(ProgressGateError::Reentrant)
                ));
                drop(external);
                drop(acquire_progress_write().unwrap());
                drop(acquire_external_export().unwrap());
            },
        );
    }

    #[test]
    fn h05_one_external_guard_supports_directory_then_repeated_files() {
        isolated(
            "h05_one_external_guard_supports_directory_then_repeated_files",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let protected = tree(&root);
                let guard = acquire_external_export().unwrap();
                let resolved =
                    create_external_directory_under_guard(&root, &outside.join("bundle"), &guard)
                        .unwrap();
                assert!(resolved.is_absolute());
                assert_eq!(
                    fs::canonicalize(&resolved).unwrap(),
                    fs::canonicalize(outside.join("bundle")).unwrap()
                );
                for name in ["custom.tsv", "progress.json", "complete.json"] {
                    write_external_file_under_guard(&root, &resolved.join(name), PAYLOAD, &guard)
                        .unwrap();
                    assert_eq!(fs::read(resolved.join(name)).unwrap(), PAYLOAD);
                }
                drop(guard);
                assert_eq!(tree(&root), protected);
                drop(acquire_progress_write().unwrap());
            },
        );
    }

    #[test]
    fn h05_external_guard_excludes_other_thread_normal_writer_until_drop() {
        isolated(
            "h05_external_guard_excludes_other_thread_normal_writer_until_drop",
            || {
                let external = acquire_external_export().unwrap();
                let (started_tx, started_rx) = mpsc::channel();
                let (entered_tx, entered_rx) = mpsc::channel();
                let join = thread::spawn(move || {
                    started_tx.send(()).unwrap();
                    let _normal = acquire_progress_write().unwrap();
                    entered_tx.send(()).unwrap();
                });
                started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                assert_eq!(
                    entered_rx.recv_timeout(Duration::from_millis(200)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                );
                drop(external);
                entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                join.join().unwrap();
            },
        );
    }

    #[test]
    fn h05_normal_guard_excludes_other_thread_external_writer_until_drop() {
        isolated(
            "h05_normal_guard_excludes_other_thread_external_writer_until_drop",
            || {
                let normal = acquire_progress_write().unwrap();
                let (started_tx, started_rx) = mpsc::channel();
                let (entered_tx, entered_rx) = mpsc::channel();
                let join = thread::spawn(move || {
                    started_tx.send(()).unwrap();
                    let _external = acquire_external_export().unwrap();
                    entered_tx.send(()).unwrap();
                });
                started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                assert_eq!(
                    entered_rx.recv_timeout(Duration::from_millis(200)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                );
                drop(normal);
                entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                join.join().unwrap();
            },
        );
    }

    #[test]
    fn h04_legacy_atomic_write_remains_gate_free_under_both_guards() {
        isolated(
            "h04_legacy_atomic_write_remains_gate_free_under_both_guards",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let normal = acquire_progress_write().unwrap();
                atomic_write(&outside.join("normal.txt"), PAYLOAD).unwrap();
                drop(normal);
                let external = acquire_external_export().unwrap();
                atomic_write(&outside.join("external.txt"), PAYLOAD).unwrap();
                drop(external);
                assert_eq!(fs::read(outside.join("normal.txt")).unwrap(), PAYLOAD);
                assert_eq!(fs::read(outside.join("external.txt")).unwrap(), PAYLOAD);
                assert_eq!(
                    fs::read(root.join("progress.json")).unwrap(),
                    b"protected exact bytes"
                );
            },
        );
    }

    #[test]
    fn h06_prepare_is_read_only_and_typed_initial_refusals_are_distinct() {
        isolated(
            "h06_prepare_is_read_only_and_typed_initial_refusals_are_distinct",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let before = tree(&temp.root);
                let guard = acquire_external_export().unwrap();
                assert!(matches!(
                    prepare_external_destination(
                        &root,
                        &root.join("new"),
                        ExternalDestinationKind::File,
                        &guard
                    ),
                    Err(ExternalDestinationError::ProtectedRoot)
                ));
                assert!(matches!(
                    prepare_external_destination(
                        &root,
                        &outside.join("NUL.txt"),
                        ExternalDestinationKind::File,
                        &guard
                    ),
                    Err(ExternalDestinationError::UnsafePath)
                ));
                assert!(matches!(
                    prepare_external_destination(
                        &root,
                        &outside,
                        ExternalDestinationKind::Directory,
                        &guard
                    ),
                    Err(ExternalDestinationError::AlreadyExists)
                ));
                let checked = prepare_external_destination(
                    &root,
                    &outside.join("new.txt"),
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                assert!(recheck_external_destination(&checked).is_ok());
                assert_eq!(tree(&temp.root), before);
            },
        );
    }

    fn changed_file(which: &str) {
        let temp = Temp::new();
        let (mut root, outside) = paths(&temp);
        let dest = outside.join("target.txt");
        let present = matches!(
            which,
            "leaf_replace" | "leaf_disappear" | "hardlink_add" | "leaf_becomes_directory"
        );
        if present {
            fs::write(&dest, b"old destination").unwrap();
        }
        if which == "root_appears" {
            root = temp.root.join("planned");
        }
        if which == "planned_ancestor_appears" {
            root = temp.root.join("planned/deeper");
        }
        let guard = acquire_external_export().unwrap();
        let checked =
            prepare_external_destination(&root, &dest, ExternalDestinationKind::File, &guard)
                .unwrap();
        match which {
            "parent_replace" => {
                fs::rename(&outside, temp.root.join("parked-parent")).unwrap();
                fs::create_dir(&outside).unwrap();
            }
            "leaf_replace" => {
                fs::rename(&dest, outside.join("old-leaf")).unwrap();
                fs::write(&dest, b"replacement by fixture").unwrap();
            }
            "leaf_disappear" => {
                fs::rename(&dest, outside.join("old-leaf")).unwrap();
            }
            "leaf_appears" => {
                fs::write(&dest, b"created by fixture").unwrap();
            }
            "hardlink_add" => {
                fs::hard_link(&dest, outside.join("alias")).unwrap();
            }
            "leaf_becomes_directory" => {
                fs::rename(&dest, outside.join("old-leaf")).unwrap();
                fs::create_dir(&dest).unwrap();
            }
            "root_appears" => fs::create_dir(&root).unwrap(),
            "planned_ancestor_appears" => fs::create_dir(root.parent().unwrap()).unwrap(),
            "root_moves" => {
                fs::rename(&root, temp.root.join("parked-root")).unwrap();
                fs::create_dir(&root).unwrap();
            }
            _ => unreachable!(),
        }
        let before = tree(&temp.root); // Fixture mutations are the new baseline.
        assert!(
            matches!(
                recheck_external_destination(&checked),
                Err(ExternalDestinationError::Changed)
            ),
            "expected Changed after {which}"
        );
        assert!(publish_external_file(&checked, PAYLOAD, &outside.join("fixed-temp")).is_err());
        assert_eq!(
            tree(&temp.root),
            before,
            "export changed fixture after {which}"
        );
        drop(checked);
        drop(guard);
        if matches!(
            which,
            "leaf_replace"
                | "leaf_disappear"
                | "leaf_appears"
                | "parent_replace"
                | "root_appears"
                | "planned_ancestor_appears"
                | "root_moves"
        ) {
            write_external_file(&root, &dest, PAYLOAD).unwrap();
            assert_eq!(
                fs::read(&dest).unwrap(),
                PAYLOAD,
                "a new explicit operation may accept the new safe identity"
            );
        }
    }
    #[test]
    fn h06_file_parent_replaced_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_file_parent_replaced_is_changed_and_retry_is_fresh",
            || {
                changed_file("parent_replace");
            },
        );
    }
    #[test]
    fn h06_file_leaf_replaced_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_file_leaf_replaced_is_changed_and_retry_is_fresh",
            || {
                changed_file("leaf_replace");
            },
        );
    }
    #[test]
    fn h06_file_leaf_disappeared_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_file_leaf_disappeared_is_changed_and_retry_is_fresh",
            || {
                changed_file("leaf_disappear");
            },
        );
    }
    #[test]
    fn h06_file_absent_leaf_created_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_file_absent_leaf_created_is_changed_and_retry_is_fresh",
            || {
                changed_file("leaf_appears");
            },
        );
    }
    #[test]
    fn h06_file_hardlink_added_is_changed() {
        isolated("h06_file_hardlink_added_is_changed", || {
            changed_file("hardlink_add");
        });
    }
    #[test]
    fn h06_file_leaf_became_directory_is_changed() {
        isolated("h06_file_leaf_became_directory_is_changed", || {
            changed_file("leaf_becomes_directory");
        });
    }
    #[test]
    fn h06_planned_root_appeared_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_planned_root_appeared_is_changed_and_retry_is_fresh",
            || {
                changed_file("root_appears");
            },
        );
    }
    #[test]
    fn h06_planned_ancestor_appeared_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_planned_ancestor_appeared_is_changed_and_retry_is_fresh",
            || {
                changed_file("planned_ancestor_appears");
            },
        );
    }
    #[test]
    fn h06_existing_root_replaced_is_changed_and_retry_is_fresh() {
        isolated(
            "h06_existing_root_replaced_is_changed_and_retry_is_fresh",
            || {
                changed_file("root_moves");
            },
        );
    }

    fn changed_directory(which: &str) {
        let temp = Temp::new();
        let (mut root, outside) = paths(&temp);
        let dest = outside.join("bundle");
        if which == "root_appears" {
            root = temp.root.join("planned");
        }
        let guard = acquire_external_export().unwrap();
        let checked =
            prepare_external_destination(&root, &dest, ExternalDestinationKind::Directory, &guard)
                .unwrap();
        match which {
            "parent_replace" => {
                fs::rename(&outside, temp.root.join("parked-parent")).unwrap();
                fs::create_dir(&outside).unwrap();
            }
            "leaf_appears" => {
                fs::create_dir(&dest).unwrap();
                fs::write(dest.join("sentinel"), b"fixture bytes").unwrap();
            }
            "root_appears" => fs::create_dir(&root).unwrap(),
            _ => unreachable!(),
        }
        let before = tree(&temp.root);
        assert!(matches!(
            recheck_external_destination(&checked),
            Err(ExternalDestinationError::Changed)
        ));
        assert!(create_checked_external_directory(&checked).is_err());
        assert_eq!(tree(&temp.root), before);
    }
    #[test]
    fn h06_directory_parent_replaced_is_changed() {
        isolated("h06_directory_parent_replaced_is_changed", || {
            changed_directory("parent_replace");
        });
    }
    #[test]
    fn h06_directory_absent_leaf_created_is_changed() {
        isolated("h06_directory_absent_leaf_created_is_changed", || {
            changed_directory("leaf_appears");
        });
    }
    #[test]
    fn h06_directory_planned_root_appeared_is_changed() {
        isolated("h06_directory_planned_root_appeared_is_changed", || {
            changed_directory("root_appears");
        });
    }

    #[test]
    fn h06_real_identity_denial_is_not_absence_when_fixture_is_available() {
        isolated(
            "h06_real_identity_denial_is_not_absence_when_fixture_is_available",
            || {
                #[repr(C)]
                struct Identity {
                    volume: u64,
                    id: [u8; 16],
                }
                #[link(name = "kernel32")]
                extern "system" {
                    fn GetFileInformationByHandleEx(
                        handle: *mut std::ffi::c_void,
                        class: i32,
                        buffer: *mut std::ffi::c_void,
                        size: u32,
                    ) -> i32;
                }
                fn probe(path: &Path) -> std::io::Result<()> {
                    let file = OpenOptions::new()
                        .access_mode(0x80)
                        .share_mode(7)
                        .custom_flags(0x0020_0000)
                        .open(path)?;
                    let mut id = Identity {
                        volume: 0,
                        id: [0; 16],
                    };
                    // SAFETY: FileIdInfo is class 18; repr(C) storage is 24 bytes with 8-byte alignment.
                    let ok = unsafe {
                        GetFileInformationByHandleEx(
                            file.as_raw_handle(),
                            18,
                            (&mut id as *mut Identity).cast(),
                            std::mem::size_of::<Identity>() as u32,
                        )
                    };
                    if ok == 0 {
                        Err(std::io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                }
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let dest = outside.join("target.txt");
                fs::write(&dest, b"preserved exact target").unwrap();
                probe(&dest)
                    .expect("FIXTURE_FAILURE: normal attributes/identity probe must succeed");
                let guard = acquire_external_export().unwrap();
                let checked = prepare_external_destination(
                    &root,
                    &dest,
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                let before = tree(&temp.root);
                let exclusive = match OpenOptions::new().read(true).share_mode(0).open(&dest) {
                    Ok(handle) => handle,
                    Err(e) => {
                        eprintln!("WW5_H_FIXTURE_UNAVAILABLE:h06_real_identity_denial:exclusive fixture handle failed:{e}; NOT BEHAVIOR PASS OR RED");
                        return;
                    }
                };
                if probe(&dest).is_ok() {
                    drop(exclusive);
                    eprintln!("WW5_H_FIXTURE_UNAVAILABLE:h06_real_identity_denial:FILE_READ_ATTRIBUTES/FileIdInfo succeeds under share_mode(0); NOT BEHAVIOR PASS OR RED");
                    return;
                }
                eprintln!("WW5_H_FIXTURE_CONFIRMED:h06_real_identity_denial:independent attributes/identity probe failed");
                let rechecked = recheck_external_destination(&checked);
                let published =
                    publish_external_file(&checked, PAYLOAD, &outside.join("fixed-temp"));
                drop(exclusive);
                assert!(matches!(
                    rechecked,
                    Err(ExternalDestinationError::Io
                        | ExternalDestinationError::IdentityUnavailable)
                ));
                assert!(published.is_err());
                assert_eq!(tree(&temp.root), before);
                drop(checked);
                drop(guard);
                probe(&dest).unwrap();
                write_external_file(&root, &dest, PAYLOAD).unwrap();
                assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
            },
        );
    }

    #[test]
    fn h07_fixed_temp_equal_to_planned_root_is_rejected_before_io() {
        isolated(
            "h07_fixed_temp_equal_to_planned_root_is_rejected_before_io",
            || {
                let temp = Temp::new();
                let (_, outside) = paths(&temp);
                let planned = outside.join("note.tmp-fixed");
                let dest = outside.join("note.txt");
                fs::write(&dest, b"existing destination").unwrap();
                let before = tree(&temp.root);
                let guard = acquire_external_export().unwrap();
                let checked = prepare_external_destination(
                    &planned,
                    &dest,
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                assert!(publish_external_file(&checked, PAYLOAD, &planned).is_err());
                assert_eq!(tree(&temp.root), before);
                assert!(!planned.exists());
            },
        );
    }
    #[test]
    fn h07_absent_destination_cannot_be_its_own_temp() {
        isolated("h07_absent_destination_cannot_be_its_own_temp", || {
            let temp = Temp::new();
            let (root, outside) = paths(&temp);
            let dest = outside.join("absent-destination.txt");
            assert!(matches!(
                fs::symlink_metadata(&dest),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound
            ));
            let before = tree(&temp.root);
            assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
            let guard = acquire_external_export().unwrap();
            let checked =
                prepare_external_destination(&root, &dest, ExternalDestinationKind::File, &guard)
                    .unwrap();
            assert_eq!(tree(&temp.root), before, "prepare must not create a temp");
            println!("WW5_H_TEMP1_PREPARED_ABSENT");
            assert!(publish_external_file(&checked, PAYLOAD, &dest).is_err());
            assert!(matches!(
                fs::symlink_metadata(&dest),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound
            ));
            assert_eq!(tree(&temp.root), before);
            assert_eq!(
                fs::read_dir(&outside).unwrap().count(),
                0,
                "temp increment must be zero"
            );
            println!("WW5_H_TEMP1_REJECTED_UNCHANGED");
        });
    }
    #[test]
    fn h07_existing_temp_sentinel_is_not_deleted_or_replaced() {
        isolated(
            "h07_existing_temp_sentinel_is_not_deleted_or_replaced",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let dest = outside.join("note.txt");
                let fixed = outside.join("fixed-temp");
                fs::write(&dest, b"old destination").unwrap();
                fs::write(&fixed, b"existing temp sentinel").unwrap();
                let before = tree(&temp.root);
                let guard = acquire_external_export().unwrap();
                let checked = prepare_external_destination(
                    &root,
                    &dest,
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                assert!(publish_external_file(&checked, PAYLOAD, &fixed).is_err());
                assert_eq!(tree(&temp.root), before);
            },
        );
    }
    #[test]
    fn h07_normal_fixed_temp_publishes_exact_bytes_and_disappears() {
        isolated(
            "h07_normal_fixed_temp_publishes_exact_bytes_and_disappears",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let dest = outside.join("note.txt");
                let fixed = outside.join("fixed-temp");
                fs::write(&dest, b"old destination").unwrap();
                let protected = tree(&root);
                let guard = acquire_external_export().unwrap();
                let checked = prepare_external_destination(
                    &root,
                    &dest,
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                publish_external_file(&checked, PAYLOAD, &fixed).unwrap();
                assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
                assert!(!fixed.exists());
                assert_eq!(tree(&root), protected);
                assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
            },
        );
    }
    #[test]
    fn h07_wrong_parent_or_protected_temp_is_rejected_before_io() {
        isolated(
            "h07_wrong_parent_or_protected_temp_is_rejected_before_io",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let dest = outside.join("note.txt");
                let guard = acquire_external_export().unwrap();
                let checked = prepare_external_destination(
                    &root,
                    &dest,
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                for fixed in [temp.root.join("wrong-parent"), root.join("new-temp")] {
                    let before = tree(&temp.root);
                    assert!(publish_external_file(&checked, PAYLOAD, &fixed).is_err());
                    assert_eq!(tree(&temp.root), before);
                }
            },
        );
    }
    #[test]
    fn h07_final_operation_requires_matching_file_or_directory_mode() {
        isolated(
            "h07_final_operation_requires_matching_file_or_directory_mode",
            || {
                let temp = Temp::new();
                let (root, outside) = paths(&temp);
                let guard = acquire_external_export().unwrap();
                let file = prepare_external_destination(
                    &root,
                    &outside.join("file"),
                    ExternalDestinationKind::File,
                    &guard,
                )
                .unwrap();
                let dir = prepare_external_destination(
                    &root,
                    &outside.join("directory"),
                    ExternalDestinationKind::Directory,
                    &guard,
                )
                .unwrap();
                let before = tree(&temp.root);
                assert!(create_checked_external_directory(&file).is_err());
                assert!(publish_external_file(&dir, PAYLOAD, &outside.join("fixed-temp")).is_err());
                assert_eq!(tree(&temp.root), before);
                let created = create_checked_external_directory(&dir).unwrap();
                assert!(created.is_absolute());
                assert!(created.is_dir());
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_ui_legacy_palette_keys_default_without_changing_other_settings() {
        let root = std::env::temp_dir().join(format!(
            "ww-feedback-ui-legacy-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let storage = Storage::at(root.clone()).unwrap();
        let mut old = Progress::default();
        old.settings.ai_daily_limit = 37;
        old.settings.topic = "保存済みの合成設定".into();
        let mut legacy = serde_json::to_value(&old).unwrap();
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("page_tint");
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("input_tint");
        fs::write(
            root.join("progress.json"),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        let saved = fs::read(root.join("progress.json")).unwrap();
        let loaded = storage.load().unwrap();
        let settings = serde_json::to_value(&loaded.settings).unwrap();
        assert_eq!(settings["page_tint"]["rgb"], serde_json::json!([0, 0, 0]));
        assert_eq!(settings["page_tint"]["depth"], 0);
        assert_eq!(settings["input_tint"]["rgb"], serde_json::json!([0, 0, 0]));
        assert_eq!(settings["input_tint"]["depth"], 0);
        assert_eq!(loaded.settings.ai_daily_limit, 37);
        assert_eq!(loaded.settings.topic, "保存済みの合成設定");
        assert_eq!(fs::read(root.join("progress.json")).unwrap(), saved);
        drop(storage);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_invalid_persisted_palette_is_rejected_without_repair() {
        let root = std::env::temp_dir().join(format!(
            "ww-feedback-ui-invalid-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let storage = Storage::at(root.clone()).unwrap();
        let path = root.join("progress.json");
        for invalid in [
            serde_json::json!({"rgb":[0,0,0], "depth":101}),
            serde_json::json!({"rgb":[0,0,0], "depth":-1}),
            serde_json::json!({"rgb":[0,0,0], "depth":1.5}),
            serde_json::json!({"rgb":[0,0,0], "depth":"40"}),
            serde_json::json!({"rgb":[256,0,0], "depth":40}),
        ] {
            let mut document = serde_json::to_value(Progress::default()).unwrap();
            document["settings"]["page_tint"] = invalid;
            let original = serde_json::to_vec_pretty(&document).unwrap();
            fs::write(&path, &original).unwrap();
            assert!(
                storage.load().is_err(),
                "invalid palette was accepted: {document}"
            );
            assert_eq!(
                fs::read(&path).unwrap(),
                original,
                "load repaired user data"
            );
        }
        drop(storage);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_chat_action_preserves_transcript_and_draft() {
        let mut p = Progress::default();
        let mut chat = crate::chat::Conversation::new();
        chat.draft = "追加して".into();
        let id = chat.id.clone();
        p.chats.push(chat);
        let before = serde_json::to_value(&p).unwrap();
        let reply = crate::chat_action::ChatReply {
            answer: "追加案を確認してください".into(),
            title: "追加".into(),
            execution: Default::default(),
            action: Some(crate::chat_action::Action {
                operation: crate::chat_action::Operation::Append,
                base: "make".into(),
                entry_id: Some("x".repeat(1001)),
            }),
        };
        assert!(p.complete_chat(&id, "追加して".into(), reply).is_err());
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
    }
    #[test]
    fn completed_reply_does_not_replace_a_learner_edited_chat_title() {
        let mut p = Progress::default();
        let mut chat = crate::chat::Conversation::new();
        chat.rename("自分で付けたタイトル").unwrap();
        let id = chat.id.clone();
        p.chats.push(chat);
        p.complete_chat(
            &id,
            "question".into(),
            crate::chat_action::ChatReply {
                answer: "answer".into(),
                title: "AIが提案したタイトル".into(),
                execution: Default::default(),
                action: None,
            },
        )
        .unwrap();
        assert_eq!(p.chats[0].title, "自分で付けたタイトル");
        assert!(p.chats[0].title_manual);
    }
    #[test]
    fn progress_roundtrip_and_observed_retention() {
        let mut p = Progress::default();
        p.record(
            "x:recall".into(),
            Grade::Good,
            false,
            "keyboard",
            0,
            "a",
            5,
            false,
        );
        p.record(
            "x:recall".into(),
            Grade::Good,
            false,
            "keyboard",
            8 * 86400,
            "b",
            5,
            false,
        );
        assert_eq!(p.observed_retention(7.0), Some((1, 1)));
        let q: Progress = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(q.memories["x:recall"].reviews, 2);
        let mut old = serde_json::to_value(&p).unwrap();
        old.as_object_mut().unwrap().remove("writing_logs");
        old.as_object_mut().unwrap().remove("chats");
        old.as_object_mut().unwrap().remove("material_draft");
        old.as_object_mut().unwrap().remove("material_sources");
        old.as_object_mut().unwrap().remove("deleted_entries");
        old.as_object_mut().unwrap().remove("chat_action");
        let migrated: Progress = serde_json::from_value(old.clone()).unwrap();
        assert!(migrated.chats.is_empty());
        assert!(migrated.material_draft.is_none() && migrated.material_sources.is_empty());
        assert!(migrated.deleted_entries.is_empty() && migrated.chat_action.is_none());
        let settings = old["settings"].as_object_mut().unwrap();
        for name in [
            "codex_path",
            "codex_model",
            "examples_per_word",
            "batch_words",
            "speech_volume",
            "speech_repeat",
        ] {
            settings.remove(name);
        }
        settings.insert(
            "api_base".into(),
            serde_json::json!("https://old.example/v1"),
        );
        settings.insert("text_model".into(), serde_json::json!("old-api-model"));
        let legacy: Progress = serde_json::from_value(old).unwrap();
        assert_eq!(legacy.settings.codex_path, "codex");
        assert_eq!(legacy.settings.examples_per_word, 6);
        assert!(legacy.settings.codex_model.is_empty());
        assert_eq!(legacy.settings.speech_volume, 0.5);
        assert!(!legacy.settings.speech_repeat);
        assert!(migrated.writing_logs.is_empty());
        assert_eq!(migrated.memories["x:recall"].reviews, 2);
    }
    #[test]
    fn playback_volume_validation_rejects_non_finite_and_out_of_range_values() {
        for volume in [-0.01, 1.01, f64::NAN, f64::INFINITY] {
            let mut progress = Progress::default();
            progress.settings.speech_volume = volume;
            assert!(progress.validate().is_err(), "accepted volume {volume}");
        }
        assert_eq!(Progress::default().settings.speech_volume, 0.5);
    }
    #[test]
    fn edited_content_resets_only_that_word() {
        let mut d = crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap();
        let mut p = Progress::default();
        p.reconcile_deck(&d);
        p.memories
            .insert(Skill::Recall.key(&d[0].id), Memory::default());
        p.memories
            .insert(Skill::Recall.key(&d[1].id), Memory::default());
        d[0].usage.push_str(" 改訂");
        assert_eq!(p.reconcile_deck(&d), 1);
        assert!(!p.memories.contains_key(&Skill::Recall.key(&d[0].id)));
        assert!(p.memories.contains_key(&Skill::Recall.key(&d[1].id)));
    }
    #[test]
    fn arbitrary_json_is_not_a_valid_progress_backup() {
        assert!(serde_json::from_str::<Progress>("{}").is_err());
    }
    #[test]
    fn corrupt_progress_is_never_overwritten_by_load() {
        let dir = std::env::temp_dir().join(format!(
            "wordweave-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let s = Storage::at(dir.clone()).unwrap();
        fs::write(dir.join("progress.json"), b"BROKEN").unwrap();
        assert!(s.load().is_err());
        assert_eq!(
            fs::read_to_string(dir.join("progress.json")).unwrap(),
            "BROKEN"
        );
        drop(s);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn writes_replace_and_another_instance_cannot_lock() {
        let dir = std::env::temp_dir().join(format!(
            "wordweave-write-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let s = Storage::at(dir.clone()).unwrap();
        assert!(Storage::at(dir.clone()).is_err());
        s.save(&Progress::default()).unwrap();
        let mut p = Progress::default();
        p.settings.new_per_day = 1;
        s.save(&p).unwrap();
        assert_eq!(s.load().unwrap().settings.new_per_day, 1);
        drop(s);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn oversized_pretty_progress_is_rejected_before_backup_or_replacement() {
        let dir = std::env::temp_dir().join(format!(
            "wordweave-save-limit-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let storage = Storage::at(dir.clone()).unwrap();
        let original = Progress::default();
        storage.save(&original).unwrap();
        let original_bytes = fs::read(dir.join("progress.json")).unwrap();
        let mut next = original.clone();
        next.settings.topic = "日本語もバイト数で判定する".into();
        let expected = serde_json::to_vec_pretty(&next).unwrap();
        let exact_limit = expected.len() as u64;
        assert!(serde_json::to_vec(&next).unwrap().len() < expected.len() - 1);

        assert!(storage.save_with_limit(&next, exact_limit - 1).is_err());
        assert_eq!(fs::read(dir.join("progress.json")).unwrap(), original_bytes);
        assert!(!dir.join("backups").exists());

        storage.save_with_limit(&next, exact_limit).unwrap();
        assert_eq!(fs::read(dir.join("progress.json")).unwrap(), expected);
        assert_eq!(storage.load().unwrap().settings.topic, next.settings.topic);
        let backups: Vec<_> = fs::read_dir(dir.join("backups"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original_bytes);
        drop(storage);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn chat_history_draft_and_reported_settings_survive_storage_reopen() {
        let dir = std::env::temp_dir().join(format!(
            "wordweave-chat-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let s = Storage::at(dir.clone()).unwrap();
        let mut p = Progress::default();
        let mut chat = crate::chat::Conversation::new();
        chat.memo = "社外メール用".into();
        chat.draft = "最初の質問".into();
        chat.complete(
            "最初の質問".into(),
            "回答".into(),
            crate::execution::Execution {
                model: Some("test-model".into()),
                effort: None,
                at: 123,
            },
        )
        .unwrap();
        chat.exchanges[0].pinned = true;
        chat.draft = "次の質問".into();
        p.chats.push(chat);
        s.save(&p).unwrap();
        drop(s);
        let reopened = Storage::at(dir.clone()).unwrap();
        let restored = reopened.load().unwrap();
        let chat = &restored.chats[0];
        assert_eq!(chat.draft, "次の質問");
        assert_eq!(chat.memo, "社外メール用");
        assert!(chat.exchanges[0].pinned);
        assert_eq!(chat.exchanges[0].execution.effort, None);
        assert_eq!(
            chat.exchanges[0].execution.model.as_deref(),
            Some("test-model")
        );
        assert_eq!(crate::chat::prepare(chat).unwrap().included, 1);
        drop(reopened);
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(all(test, windows))]
pub(crate) mod i2_gate_tests {
    use super::*;
    use crate::audio_history::*;
    use std::{
        cell::RefCell,
        process::{Command, Stdio},
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };

    const CHILD: &str = "WW5_I2_GATE_LIB_CASE";
    const CHILD_ROOT: &str = "WW5_I2_GATE_LIB_ROOT";
    static NEXT: AtomicU64 = AtomicU64::new(0);

    pub(crate) struct Temp {
        pub(crate) root: PathBuf,
        links: Vec<PathBuf>,
    }
    impl Temp {
        pub(crate) fn new() -> Self {
            let parent = std::env::var_os(CHILD_ROOT)
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = parent.join(format!(
                "ww5-i2-gate-lib-{}-{nonce}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self {
                root,
                links: vec![],
            }
        }

        pub(crate) fn junction(&mut self, link: &Path, target: &Path) {
            use std::os::windows::{
                ffi::OsStrExt,
                fs::{MetadataExt, OpenOptionsExt},
                io::AsRawHandle,
            };
            use std::{ffi::c_void, ptr};
            #[link(name = "kernel32")]
            extern "system" {
                fn DeviceIoControl(
                    device: *mut c_void,
                    code: u32,
                    input: *const c_void,
                    input_len: u32,
                    output: *mut c_void,
                    output_len: u32,
                    returned: *mut u32,
                    overlapped: *mut c_void,
                ) -> i32;
            }
            assert!(link.starts_with(&self.root) && target.starts_with(&self.root));
            assert!(!link.exists());
            self.links.push(link.to_owned());
            fs::create_dir(link).unwrap();
            let canonical = fs::canonicalize(target).unwrap();
            let canonical: Vec<u16> = canonical.as_os_str().encode_wide().collect();
            let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
            assert!(canonical.starts_with(&prefix));
            let print = &canonical[prefix.len()..];
            let substitute: Vec<u16> = r"\??\"
                .encode_utf16()
                .chain(print.iter().copied())
                .collect();
            let sub_bytes = u16::try_from(substitute.len() * 2).unwrap();
            let print_bytes = u16::try_from(print.len() * 2).unwrap();
            let mut buffer = Vec::new();
            buffer.extend_from_slice(&0xa000_0003_u32.to_le_bytes());
            buffer.extend_from_slice(&(8 + sub_bytes + 2 + print_bytes + 2).to_le_bytes());
            for field in [0_u16, 0, sub_bytes, sub_bytes + 2, print_bytes] {
                buffer.extend_from_slice(&field.to_le_bytes());
            }
            for value in substitute
                .iter()
                .copied()
                .chain([0])
                .chain(print.iter().copied())
                .chain([0])
            {
                buffer.extend_from_slice(&value.to_le_bytes());
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(0x0200_0000 | 0x0020_0000)
                .open(link)
                .unwrap();
            let mut returned = 0;
            // SAFETY: owned handle, live buffer and returned pointer are valid
            // for this synchronous call. No overlapped or output buffer is used.
            let success = unsafe {
                DeviceIoControl(
                    file.as_raw_handle(),
                    0x0009_00a4,
                    buffer.as_ptr().cast(),
                    u32::try_from(buffer.len()).unwrap(),
                    ptr::null_mut(),
                    0,
                    &mut returned,
                    ptr::null_mut(),
                )
            };
            assert_ne!(
                success,
                0,
                "junction fixture setup failed, not behavior RED: {}",
                std::io::Error::last_os_error()
            );
            assert_ne!(
                fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
                0
            );
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            for link in &self.links {
                if let Err(error) = fs::remove_dir(link) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        eprintln!(
                            "preserving fixture after unlink failure: {}: {error}",
                            self.root.display()
                        );
                        return;
                    }
                }
            }
            if let Err(error) = fs::remove_dir_all(&self.root) {
                eprintln!("gate cleanup failed: {}: {error}", self.root.display());
            }
        }
    }

    pub(crate) fn tree(base: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(base: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
            use std::os::windows::fs::MetadataExt;
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                let relative = path.strip_prefix(base).unwrap().to_owned();
                if metadata.file_attributes() & 0x400 != 0 {
                    result.insert(relative, b"reparse entry, no traversal".to_vec());
                } else if metadata.is_dir() {
                    result.insert(relative, vec![]);
                    visit(base, &path, result);
                } else if path.file_name().unwrap() == "app.lock" {
                    result.insert(relative, metadata.len().to_le_bytes().to_vec());
                } else {
                    result.insert(relative, fs::read(path).unwrap());
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(base, base, &mut result);
        result
    }

    fn isolated(case: &str, body: impl FnOnce()) {
        if std::env::var(CHILD).as_deref() == Ok(case) {
            body();
            println!("WW5_GATE_LIB_OK:{case}");
            return;
        }
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([case, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, case)
            .env(CHILD_ROOT, &temp.root)
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
            "child {case} exceeded 15 seconds; killed/reaped\n{stdout}\n{stderr}"
        );
        assert!(
            output.status.success(),
            "child {case}: {}\n{stdout}\n{stderr}",
            output.status
        );
        assert!(
            stdout.contains("running 1 test")
                && stdout.contains("1 passed; 0 failed")
                && stdout.contains(&format!("WW5_GATE_LIB_OK:{case}")),
            "wrong child result: {stdout}"
        );
    }

    fn request() -> FrozenHistoryRequest {
        let raw = b"gate-only synthetic original";
        let digest = |bytes: &[u8]| Sha256::parse(&crate::assets::sha256(bytes).unwrap()).unwrap();
        let mutation = HistoryMutation::CreateTrial {
            id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
            created_at: RecordedAt {
                utc_millis: 1_700_000_000_000,
                offset_minutes: 540,
            },
            input: FrozenInput {
                audio: AudioSnapshot {
                    original: BlobRef {
                        sha256: digest(raw),
                        bytes: raw.len() as u64,
                    },
                    format: AudioFormat::Wav,
                    rate: 8_000,
                    channels: 1,
                    frames: 1,
                    origin: Origin::Imported,
                    warnings: vec![],
                },
                reference: ReferenceSnapshot {
                    text: "Gate reference".into(),
                    sha256: digest(b"Gate reference"),
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
            duplicate: None,
        };
        FrozenHistoryRequest::new(mutation, vec![Arc::from(raw.as_slice())]).unwrap()
    }

    #[test]
    fn acquisition_reentry_and_drop_leave_thread_ownership_consistent() {
        isolated(
            "store::i2_gate_tests::acquisition_reentry_and_drop_leave_thread_ownership_consistent",
            || {
                let guard = acquire_progress_write().expect("valid first gate acquisition");
                assert_eq!(guard.ensure_writable(), Ok(()));
                assert!(matches!(
                    acquire_progress_write(),
                    Err(ProgressGateError::Reentrant)
                ));
                drop(guard);
                let next = acquire_progress_write().expect("guard drop clears thread ownership");
                assert_eq!(next.ensure_writable(), Ok(()));
                drop(next);
                // The guard is created and dropped on this other thread, never moved.
                thread::spawn(|| {
                    let guard =
                        acquire_progress_write().expect("unrelated thread acquires normally");
                    assert_eq!(guard.ensure_writable(), Ok(()));
                })
                .join()
                .unwrap();
            },
        );
    }

    #[test]
    fn block_applies_to_existing_guard_under_guard_and_all_writer_entries() {
        isolated("store::i2_gate_tests::block_applies_to_existing_guard_under_guard_and_all_writer_entries", || {
            struct Head(RefCell<Option<RootRef>>);
            impl CommittedHistoryHead for Head {
                fn read_current(&self, _: &Storage) -> Result<Option<RootRef>, HistoryError> { Ok(self.0.borrow().clone()) }
            }
            let temp = Temp::new();
            let mut storage = Storage::at(temp.root.join("host")).unwrap();
            let host = storage.dir.clone();
            let head = Head(RefCell::new(None));
            let request = request();
            let operation = OperationId::parse("22222222-2222-4222-8222-000000000001").unwrap();
            let mut writer = HistoryWriter::new(&mut storage, &head, HistoryQuota::v1()).unwrap();
            let prepared = match writer.prepare(operation.clone(), None, &request).unwrap() { PrepareOutcome::Ready(p) => p, other => panic!("{other:?}") };
            let published = match writer.publish(&prepared).unwrap() { PublishOutcome::Published(p) => p, other => panic!("{other:?}") };
            *head.0.borrow_mut() = Some(published.next.clone());
            let committed = writer.confirm_committed(&published).unwrap();
            let before = tree(&temp.root);
            let guard = acquire_progress_write().expect("valid gate before blocking");
            assert_eq!(guard.ensure_writable(), Ok(()));
            guard.block_writes();
            assert_eq!(guard.ensure_writable(), Err(ProgressGateError::WriteBlocked));
            assert!(crate::commit::recover_under_guard(&host, &guard).is_err());
            assert_eq!(tree(&temp.root), before);
            drop(guard);
            assert!(matches!(acquire_progress_write(), Err(ProgressGateError::WriteBlocked)));
            for result in [
                writer.prepare(operation.clone(), Some(&published.next), &request).map(|_| ()),
                writer.publish(&prepared).map(|_| ()),
                writer.resume(&operation, &request).map(|_| ()),
                writer.finish(&committed),
            ] { assert!(matches!(result, Err(HistoryError { kind: HistoryErrorKind::Io })), "blocked history entry returned {result:?}"); }
            assert_eq!(writer.confirm_committed(&published).unwrap(), committed, "confirmation stays read-only while writes are blocked");
            drop(writer);
            assert!(storage.save(&Progress::default()).is_err());
            assert!(storage.load().is_err());
            assert!(crate::commit::save(&host, crate::model::BUILTIN_DECK, &Progress::default()).is_err());
            assert!(crate::commit::recover(&host).is_err());
            assert!(HistoryReader::new(&storage).verify_closure(&published.next).is_ok());
            assert_eq!(tree(&temp.root), before);
        });
    }

    #[test]
    fn poisoned_guard_is_not_cleared_and_blocks_metadata_retry() {
        isolated(
            "store::i2_gate_tests::poisoned_guard_is_not_cleared_and_blocks_metadata_retry",
            || {
                let temp = Temp::new();
                let storage = Storage::at(temp.root.join("host")).unwrap();
                let before = tree(&temp.root);
                let guard = acquire_progress_write().expect("valid guard before deliberate poison");
                assert_eq!(guard.ensure_writable(), Ok(()));
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    let _guard = guard;
                    panic!("deliberate private gate poison");
                }));
                assert!(result.is_err());
                for _ in 0..2 {
                    assert!(matches!(
                        acquire_progress_write(),
                        Err(ProgressGateError::Poisoned)
                    ));
                    assert!(storage.save(&Progress::default()).is_err());
                    assert!(storage.load().is_err());
                    assert!(crate::commit::recover(&storage.dir).is_err());
                }
                assert_eq!(tree(&temp.root), before);
            },
        );
    }
}

#[cfg(all(test, windows))]
pub(crate) mod i2_migration_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};
    use std::{cell::RefCell, os::windows::fs::OpenOptionsExt};

    // Independent handwritten source, .NET SHA-256 computed before tests.
    pub(crate) const RAW_V1: &str = r#"{ "japanese_drafts" : {"lesson":"保持する下書き"}, "version" : 1, "settings":{"topic":"合成・日本語"}, "memories":{"lesson|Recall":{"due":10,"last":9,"stability":1.5,"reviews":2,"lapses":0}}, "reviews":[], "suspended":["kept-entry"], "ai_calls":{"2026-10-08":3}, "study_seconds":{"2026-10-08":45}, "chats":[{"id":"fixture-chat","title":"保存会話","memo":"保持メモ","draft":"未送信","exchanges":[]}], "material_sources":[{"entry_id":"kept-entry","conversation_id":"fixture-chat","exchange_indices":[],"at":1,"mode":"Append"}] }"#;
    pub(crate) const HASH: &str =
        "a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989";
    const LIMIT: u64 = 100_000;
    struct Fixture {
        temp: Temp,
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = Temp::new();
            let root = temp.root.join("parent/host");
            fs::create_dir_all(root.join("assets")).unwrap();
            let source: Progress =
                serde_json::from_str(RAW_V1).expect("handwritten source decodes");
            source.validate().expect("handwritten source validates");
            assert_eq!(RAW_V1.len(), 550);
            fs::write(root.join("progress.json"), RAW_V1).unwrap();
            fs::write(root.join("custom.tsv"), b"unmodified deck sentinel").unwrap();
            fs::write(
                root.join("pending-material-commit.json"),
                b"unmodified pending sentinel",
            )
            .unwrap();
            fs::write(
                root.join("assets/recording.bin"),
                b"unmodified synthetic recording",
            )
            .unwrap();
            fs::write(temp.root.join("neighbor"), b"unmodified neighbor").unwrap();
            Self { temp, root }
        }
        fn backup(&self) -> PathBuf {
            self.root.join(format!("backups/progress-v1-{HASH}.json"))
        }
        fn ledger(&self) -> BTreeMap<PathBuf, Vec<u8>> {
            tree(&self.temp.root)
        }
    }
    pub(crate) struct Faults {
        pub(crate) seen: RefCell<Vec<ProgressCheckpoint>>,
        fail: Option<ProgressCheckpoint>,
    }
    impl Faults {
        pub(crate) fn new(fail: Option<ProgressCheckpoint>) -> Self {
            Self {
                seen: RefCell::new(vec![]),
                fail,
            }
        }
    }
    impl ProgressFaults for Faults {
        fn check(&self, point: ProgressCheckpoint) -> Result<(), ProgressFaultError> {
            self.seen.borrow_mut().push(point);
            if self.fail == Some(point) {
                Err(ProgressFaultError::Io)
            } else {
                Ok(())
            }
        }
    }
    fn call(
        root: &Path,
        bytes: Option<&[u8]>,
        target: u32,
        limit: u64,
        faults: &dyn ProgressFaults,
    ) -> Result<(), MigrationError> {
        let guard = acquire_progress_write().expect("valid unpoisoned gate");
        let result = preserve_v1_before_v2_under_guard(root, bytes, target, limit, &guard, faults);
        let writable = guard.ensure_writable();
        drop(guard);
        assert_eq!(
            writable,
            Ok(()),
            "migration errors must not add a global write block"
        );
        result
    }
    fn publish<P: FnOnce(&Path, &[u8]) -> Result<(), String>>(
        root: &Path,
        bytes: Option<&[u8]>,
        target: u32,
        limit: u64,
        faults: &dyn ProgressFaults,
        publisher: P,
    ) -> Result<(), MigrationError> {
        let guard = acquire_progress_write().expect("valid unpoisoned gate");
        let result = preserve_v1_before_v2_with_publisher(
            root, bytes, target, limit, &guard, faults, publisher,
        );
        let writable = guard.ensure_writable();
        drop(guard);
        assert_eq!(writable, Ok(()));
        result
    }
    fn with_backup(mut before: BTreeMap<PathBuf, Vec<u8>>) -> BTreeMap<PathBuf, Vec<u8>> {
        before.insert(PathBuf::from("parent/host/backups"), vec![]);
        before.insert(
            PathBuf::from(format!("parent/host/backups/progress-v1-{HASH}.json")),
            RAW_V1.as_bytes().to_vec(),
        );
        before
    }

    #[test]
    fn m01_exact_raw_v1_uses_fixed_hash_even_when_daily_backup_exists() {
        let f = Fixture::new();
        fs::create_dir(f.root.join("backups")).unwrap();
        fs::write(
            f.root.join("backups/progress-2026-10-08.json"),
            b"a daily backup is not the migration source",
        )
        .unwrap();
        let before = f.ledger();
        let faults = Faults::new(None);
        let result = call(&f.root, Some(RAW_V1.as_bytes()), 2, 550, &faults);
        assert_eq!(result, Ok(()));
        assert_eq!(fs::read(f.backup()).unwrap(), RAW_V1.as_bytes());
        assert!(
            f.ledger() == with_backup(before),
            "only exact fixed-hash backup may be added"
        );
        assert_eq!(
            *faults.seen.borrow(),
            [
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup
            ]
        );
    }
    #[test]
    fn m02_existing_exact_backup_is_verified_without_publisher_rewrite() {
        let f = Fixture::new();
        fs::create_dir(f.root.join("backups")).unwrap();
        fs::write(f.backup(), RAW_V1).unwrap();
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &faults,
            |_, _| {
                calls.set(calls.get() + 1);
                Err("must not rewrite matching backup".into())
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 0);
        assert!(f.ledger() == before);
    }
    #[test]
    fn m02_existing_same_name_different_bytes_are_never_repaired() {
        for bytes in [
            b"corrupt backup".as_slice(),
            br#"{"version":99}"#.as_slice(),
        ] {
            let f = Fixture::new();
            fs::create_dir(f.root.join("backups")).unwrap();
            fs::write(f.backup(), bytes).unwrap();
            let before = f.ledger();
            let calls = Cell::new(0);
            let faults = Faults::new(None);
            let result = publish(
                &f.root,
                Some(RAW_V1.as_bytes()),
                2,
                LIMIT,
                &faults,
                |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            );
            assert_eq!(result, Err(MigrationError::Conflict));
            assert_eq!(calls.get(), 0);
            assert!(faults.seen.borrow().is_empty());
            assert!(f.ledger() == before);
        }
    }
    #[test]
    fn m03_target_one_is_noop_before_source_limit_path_or_faults() {
        let f = Fixture::new();
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(Some(ProgressCheckpoint::BeforeMigrationBackup));
        let invalid = f.root.join("invalid\0root");
        let result = publish(
            &invalid,
            Some(b"not JSON and larger than zero"),
            1,
            0,
            &faults,
            |_, _| {
                calls.set(calls.get() + 1);
                Err("must not publish".into())
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert!(f.ledger() == before);
    }
    #[test]
    fn m03_unknown_target_is_rejected_before_source_processing() {
        let f = Fixture::new();
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(&f.root, Some(b"invalid"), 99, 0, &faults, |_, _| {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert_eq!(result, Err(MigrationError::UnsupportedVersion));
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert!(f.ledger() == before);
    }
    #[test]
    fn m03_no_old_file_needs_no_backup_or_faults() {
        let f = Fixture::new();
        fs::remove_file(f.root.join("progress.json")).unwrap();
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(&f.root, None, 2, LIMIT, &faults, |_, _| {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert!(f.ledger() == before);
    }
    #[test]
    fn m03_invalid_raw_version_or_structure_is_rejected_without_artifacts() {
        let versions = [
            "",
            "\"version\":null,",
            "\"version\":\"1\",",
            "\"version\":1.0,",
            "\"version\":1,\"version\":1,",
        ];
        let mut cases: Vec<(String, MigrationError)> = versions
            .into_iter()
            .map(|replacement| {
                (
                    RAW_V1.replace("\"version\" : 1,", replacement),
                    MigrationError::InvalidDocument,
                )
            })
            .collect();
        cases.extend([
            ("{".into(), MigrationError::InvalidDocument),
            (
                RAW_V1.replace("\"stability\":1.5", "\"stability\":0"),
                MigrationError::InvalidDocument,
            ),
            (
                RAW_V1
                    .replacen("{", "{\"audio_history\":null,", 1)
                    .replace("\"version\" : 1", "\"version\" : 99"),
                MigrationError::UnsupportedVersion,
            ),
            (
                RAW_V1.replace("\"version\" : 1", "\"version\" : 2"),
                MigrationError::InvalidDocument,
            ),
        ]);
        let v2 = RAW_V1
            .replacen("{", "{\"audio_history\":null,", 1)
            .replace("\"version\" : 1", "\"version\" : 2");
        cases.extend([
            (
                v2.replace("\"audio_history\":null", "\"audio_history\":{}"),
                MigrationError::InvalidDocument,
            ),
            (
                v2.replacen("{", "{\"unknown_top\":0,", 1),
                MigrationError::InvalidDocument,
            ),
            (
                v2.replacen("{", "{\"audio_history\":null,", 1),
                MigrationError::InvalidDocument,
            ),
        ]);
        let mut observed = Vec::new();
        for (bytes, expected) in cases {
            let f = Fixture::new();
            let before = f.ledger();
            let faults = Faults::new(None);
            let result = call(&f.root, Some(bytes.as_bytes()), 2, LIMIT, &faults);
            observed.push((
                result,
                expected,
                f.ledger() == before,
                faults.seen.borrow().is_empty(),
            ));
        }
        for (result, expected, unchanged, no_faults) in observed {
            assert_eq!(result, Err(expected));
            assert!(unchanged && no_faults);
        }
    }
    #[test]
    fn m03_valid_old_v2_null_or_root_needs_no_migration() {
        for history in [
            "null",
            r#"{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":12}"#,
        ] {
            let bytes = RAW_V1
                .replacen("{", &format!("{{\"audio_history\":{history},"), 1)
                .replace("\"version\" : 1", "\"version\" : 2");
            let f = Fixture::new();
            let before = f.ledger();
            let faults = Faults::new(None);
            let calls = Cell::new(0);
            let result = publish(
                &f.root,
                Some(bytes.as_bytes()),
                2,
                LIMIT,
                &faults,
                |_, _| {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            );
            assert_eq!(result, Ok(()));
            assert_eq!(calls.get(), 0);
            assert!(faults.seen.borrow().is_empty());
            assert!(f.ledger() == before);
        }
    }
    #[test]
    fn m03_target_two_rejects_one_byte_over_limit_without_writes() {
        let f = Fixture::new();
        let before = f.ledger();
        let faults = Faults::new(None);
        let result = call(&f.root, Some(RAW_V1.as_bytes()), 2, 549, &faults);
        assert_eq!(result, Err(MigrationError::LimitExceeded));
        assert!(f.ledger() == before);
        assert!(faults.seen.borrow().is_empty());
    }
    #[test]
    fn m03_wrong_types_and_real_junctions_are_rejected_before_backup() {
        let mut observations = Vec::new();
        for placement in 0..6 {
            let mut f = Fixture::new();
            let mut exposed = f.root.clone();
            match placement {
                0 => {
                    fs::write(f.root.join("backups"), b"not a directory").unwrap();
                }
                1 => {
                    fs::create_dir(f.root.join("backups")).unwrap();
                    fs::create_dir(f.backup()).unwrap();
                }
                2 => {
                    let link = f.temp.root.join("alias-root");
                    f.temp.junction(&link, &f.root);
                    exposed = link;
                }
                3 => {
                    let link = f.temp.root.join("alias-parent");
                    let parent = f.temp.root.join("parent");
                    f.temp.junction(&link, &parent);
                    exposed = link.join("host");
                }
                4 => {
                    let target = f.temp.root.join("backup-target");
                    fs::create_dir(&target).unwrap();
                    f.temp.junction(&f.root.join("backups"), &target);
                }
                _ => {
                    fs::create_dir(f.root.join("backups")).unwrap();
                    let target = f.temp.root.join("leaf-target");
                    fs::create_dir(&target).unwrap();
                    let leaf = f.backup();
                    f.temp.junction(&leaf, &target);
                }
            }
            assert_eq!(
                fs::read(exposed.join("progress.json")).unwrap(),
                RAW_V1.as_bytes(),
                "path fixture exposes the valid source"
            );
            let before = f.ledger();
            let faults = Faults::new(None);
            let result = call(&exposed, Some(RAW_V1.as_bytes()), 2, LIMIT, &faults);
            observations.push((
                placement,
                result,
                f.ledger() == before,
                faults.seen.borrow().is_empty(),
            ));
        }
        for (placement, result, unchanged, no_faults) in observations {
            assert_eq!(
                result,
                Err(MigrationError::UnsafePath),
                "placement={placement}"
            );
            assert!(unchanged && no_faults);
        }
    }
    #[test]
    fn m04_before_backup_fault_leaves_the_entire_ledger_unchanged() {
        let f = Fixture::new();
        let before = f.ledger();
        let faults = Faults::new(Some(ProgressCheckpoint::BeforeMigrationBackup));
        let result = call(&f.root, Some(RAW_V1.as_bytes()), 2, LIMIT, &faults);
        assert_eq!(result, Err(MigrationError::Io));
        assert_eq!(
            *faults.seen.borrow(),
            [ProgressCheckpoint::BeforeMigrationBackup]
        );
        assert!(f.ledger() == before);
    }
    #[test]
    fn m04_after_backup_fault_retains_exact_backup_and_allows_retry() {
        let f = Fixture::new();
        let before = f.ledger();
        let faults = Faults::new(Some(ProgressCheckpoint::AfterMigrationBackup));
        let result = call(&f.root, Some(RAW_V1.as_bytes()), 2, LIMIT, &faults);
        assert_eq!(result, Err(MigrationError::Io));
        assert_eq!(
            *faults.seen.borrow(),
            [
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup
            ]
        );
        assert!(f.ledger() == with_backup(before));
        let confirmed = f.ledger();
        let calls = Cell::new(0);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &NoProgressFaults,
            |_, _| {
                calls.set(calls.get() + 1);
                Err("retry must not rewrite".into())
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 0);
        assert!(f.ledger() == confirmed);
    }
    #[test]
    fn m04_publisher_error_without_write_is_not_backup_success() {
        let f = Fixture::new();
        let before = f.ledger();
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &faults,
            |_, _| {
                calls.set(calls.get() + 1);
                Err("synthetic no-write failure".into())
            },
        );
        assert_eq!(result, Err(MigrationError::Io));
        assert_eq!(calls.get(), 1);
        assert!(!f.backup().exists());
        let mut after = f.ledger();
        after.remove(&PathBuf::from("parent/host/backups"));
        assert!(
            after == before,
            "only a new empty backups directory may remain"
        );
        assert_eq!(
            *faults.seen.borrow(),
            [ProgressCheckpoint::BeforeMigrationBackup]
        );
    }
    #[test]
    fn m04_real_atomic_write_followed_by_error_is_confirmed_by_exact_readback() {
        let f = Fixture::new();
        let before = f.ledger();
        let calls = Cell::new(0);
        let sent = RefCell::new(None);
        let faults = Faults::new(None);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &faults,
            |path, bytes| {
                calls.set(calls.get() + 1);
                *sent.borrow_mut() = Some((path.to_owned(), bytes.to_vec()));
                atomic_write(path, bytes)?;
                Err("synthetic post-rename failure".into())
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 1);
        assert_eq!(
            *sent.borrow(),
            Some((f.backup(), RAW_V1.as_bytes().to_vec()))
        );
        assert!(f.ledger() == with_backup(before));
        assert_eq!(
            *faults.seen.borrow(),
            [
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup
            ]
        );
    }
    #[test]
    fn m04_existing_backup_read_denial_is_real_io_and_preserves_bytes() {
        let f = Fixture::new();
        fs::create_dir(f.root.join("backups")).unwrap();
        fs::write(f.backup(), RAW_V1).unwrap();
        let before = f.ledger();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(f.backup())
            .unwrap();
        let denied = fs::read(f.backup()).expect_err("fixture must deny an actual read");
        eprintln!(
            "existing backup read denial: kind={:?}, raw={:?}",
            denied.kind(),
            denied.raw_os_error()
        );
        assert_eq!(
            denied.raw_os_error(),
            Some(32),
            "expected Windows sharing violation, not an unrelated fixture error"
        );
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &faults,
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        );
        drop(held);
        assert_eq!(result, Err(MigrationError::Io));
        assert_eq!(calls.get(), 0);
        assert!(faults.seen.borrow().is_empty());
        assert!(f.ledger() == before);
    }
    #[test]
    fn m04_published_backup_with_denied_readback_does_not_report_success() {
        let f = Fixture::new();
        let before = f.ledger();
        let held = RefCell::new(None);
        let denied = RefCell::new(None);
        let calls = Cell::new(0);
        let faults = Faults::new(None);
        let result = publish(
            &f.root,
            Some(RAW_V1.as_bytes()),
            2,
            LIMIT,
            &faults,
            |path, bytes| {
                calls.set(calls.get() + 1);
                atomic_write(path, bytes)?;
                let file = OpenOptions::new()
                    .read(true)
                    .share_mode(0)
                    .open(path)
                    .map_err(|e| e.to_string())?;
                *denied.borrow_mut() = fs::read(path).err().map(|e| (e.kind(), e.raw_os_error()));
                *held.borrow_mut() = Some(file);
                Err("synthetic post-write error with held read denial".into())
            },
        );
        drop(held.borrow_mut().take());
        assert_eq!(
            calls.get(),
            1,
            "publisher must reach the real read-denial fixture"
        );
        eprintln!("new backup read denial: {:?}", denied.borrow());
        assert_eq!(
            denied.borrow().as_ref().map(|e| e.1),
            Some(Some(32)),
            "actual read must fail with sharing violation"
        );
        assert_eq!(result, Err(MigrationError::Io));
        assert!(f.ledger() == with_backup(before));
        assert_eq!(
            *faults.seen.borrow(),
            [ProgressCheckpoint::BeforeMigrationBackup]
        );
    }
    #[test]
    fn m05_v2_save_rejects_oversized_old_source_without_changes() {
        let large = format!("{RAW_V1}{}", " ".repeat(8192));
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        fs::write(storage.dir.join("progress.json"), &large).unwrap();
        let candidate = Progress::default();
        let limit = serde_json::to_vec_pretty(&candidate).unwrap().len() as u64;
        let before = tree(&temp.root);
        let faults = Faults::new(None);
        assert!(storage
            .save_with_limit_and_faults(&candidate, limit, &faults)
            .is_err());
        assert!(faults.seen.borrow().is_empty());
        assert!(tree(&temp.root) == before);
        let f = Fixture::new();
        let before = f.ledger();
        let limit = serde_json::to_vec_pretty(&Progress::default())
            .unwrap()
            .len() as u64;
        assert!(large.len() as u64 > limit);
        let result = call(&f.root, Some(large.as_bytes()), 2, limit, &NoProgressFaults);
        assert_eq!(result, Err(MigrationError::LimitExceeded));
        assert!(f.ledger() == before);
    }
    #[test]
    fn m05_v2_save_rejects_malformed_old_source_without_changes() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        fs::write(storage.dir.join("progress.json"), b"broken old document").unwrap();
        let before = tree(&temp.root);
        let faults = Faults::new(None);
        assert!(storage
            .save_with_limit_and_faults(&Progress::default(), LIMIT, &faults)
            .is_err());
        assert!(faults.seen.borrow().is_empty());
        assert!(tree(&temp.root) == before);
    }
    #[test]
    fn m05_first_v2_save_migration_checkpoints_precede_metadata() {
        for fail in [
            None,
            Some(ProgressCheckpoint::BeforeMigrationBackup),
            Some(ProgressCheckpoint::AfterMigrationBackup),
        ] {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), RAW_V1).unwrap();
            fs::write(storage.dir.join("keep.bin"), b"synthetic preserved bytes").unwrap();
            let before = tree(&temp.root);
            let faults = Faults::new(fail);
            let candidate: Progress = serde_json::from_str(RAW_V1).unwrap();
            let result = storage.save_with_limit_and_faults(&candidate, LIMIT, &faults);
            let expected = if fail == Some(ProgressCheckpoint::BeforeMigrationBackup) {
                vec![ProgressCheckpoint::BeforeMigrationBackup]
            } else {
                vec![
                    ProgressCheckpoint::BeforeMigrationBackup,
                    ProgressCheckpoint::AfterMigrationBackup,
                ]
            };
            assert_eq!(*faults.seen.borrow(), expected);
            let backup = storage.dir.join(format!("backups/progress-v1-{HASH}.json"));
            if fail.is_some() {
                assert!(result.is_err());
                let mut expected = before;
                if fail == Some(ProgressCheckpoint::AfterMigrationBackup) {
                    expected.insert(PathBuf::from("host/backups"), vec![]);
                    expected.insert(
                        PathBuf::from(format!("host/backups/progress-v1-{HASH}.json")),
                        RAW_V1.as_bytes().to_vec(),
                    );
                }
                assert!(tree(&temp.root) == expected);
            } else {
                result.unwrap();
                assert_eq!(fs::read(backup).unwrap(), RAW_V1.as_bytes());
                let wire: serde_json::Value =
                    serde_json::from_slice(&fs::read(storage.dir.join("progress.json")).unwrap())
                        .unwrap();
                assert_eq!(wire["version"], 2);
                assert_eq!(wire.get("audio_history"), Some(&serde_json::Value::Null));
            }
        }
    }
}

#[cfg(all(test, windows))]
mod i2_backup_path_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};
    use std::os::windows::fs::OpenOptionsExt;

    const RAW: &str = r#"{"version":1,"settings":{"topic":"未保存とは別のdisk"},"memories":{},"reviews":[],"suspended":["kept"],"ai_calls":{"2026-10-11":7},"study_seconds":{"2026-10-11":31},"japanese_drafts":{"lesson":"日本語\\引用\""},"chats":[{"id":"backup-reader","title":"会話","memo":"読取fixture","draft":"保持する下書き","exchanges":[]}]}"#;
    const ROOT: &str = r#"{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":82}"#;
    fn v2(root: &str) -> String {
        RAW.replace(
            "\"version\":1",
            &format!("\"version\":2,\"audio_history\":{root}"),
        )
    }

    #[test]
    fn r01_wire1_wire2_none_and_some_preserve_fields_without_writes() {
        for raw in [RAW.to_owned(), v2("null"), v2(ROOT)] {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), &raw).unwrap();
            let before = tree(&temp.root);
            let expected: Progress = serde_json::from_str(&raw).expect("valid independent fixture");
            let result = read_progress_for_backup(&storage.dir).unwrap().unwrap();
            assert_eq!(
                serde_json::to_value(result).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            assert_eq!(
                tree(&temp.root),
                before,
                "read must not lock, migrate or write"
            );
        }
    }

    #[test]
    fn r02_absent_root_is_backup_none_but_strict_reader_storage_and_head_reject() {
        let temp = Temp::new();
        let mut storage = Storage::at(temp.root.join("host")).unwrap();
        for root in [
            temp.root.join("missing"),
            temp.root.join("missing/deep/host"),
        ] {
            let before = tree(&temp.root);
            assert!(read_progress_for_backup(&root).unwrap().is_none());
            assert_eq!(
                read_progress_typed(&root, 10000).unwrap_err(),
                ProgressReadError::Io
            );
            storage.dir = root;
            assert!(storage.read_progress().is_err());
            assert_eq!(
                ProgressHistoryHead.read_current(&storage).unwrap_err().kind,
                HistoryErrorKind::Io
            );
            assert_eq!(tree(&temp.root), before);
        }
    }

    #[test]
    fn r02_missing_leaf_under_existing_root_is_none_for_both_readers() {
        let temp = Temp::new();
        let root = temp.root.join("host");
        fs::create_dir(&root).unwrap();
        let before = tree(&temp.root);
        assert!(read_progress_for_backup(&root).unwrap().is_none());
        assert!(read_progress_typed(&root, 10000).unwrap().is_none());
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn r03_file_and_reparse_ancestors_before_missing_leaf_are_not_absence() {
        for placement in 0..5 {
            let mut temp = Temp::new();
            let host = temp.root.join("host");
            fs::create_dir(&host).unwrap();
            let exposed = match placement {
                0 => {
                    let p = temp.root.join("file");
                    fs::write(&p, b"keep").unwrap();
                    p
                }
                1 => {
                    let p = temp.root.join("file");
                    fs::write(&p, b"keep").unwrap();
                    p.join("absent/deep")
                }
                2 => {
                    fs::create_dir(host.join("progress.json")).unwrap();
                    host.clone()
                }
                3 => {
                    let p = temp.root.join("alias");
                    temp.junction(&p, &host);
                    p.join("absent/deep")
                }
                _ => {
                    let p = host.join("progress.json");
                    let target = temp.root.join("target");
                    fs::create_dir(&target).unwrap();
                    temp.junction(&p, &target);
                    host.clone()
                }
            };
            let before = tree(&temp.root);
            assert!(
                read_progress_for_backup(&exposed).is_err(),
                "placement {placement}"
            );
            assert_eq!(
                read_progress_typed(&exposed, 10000).unwrap_err(),
                ProgressReadError::UnsafePath
            );
            assert_eq!(tree(&temp.root), before);
        }
    }

    #[test]
    fn r04_pending_file_and_directory_block_backup_but_not_pure_read() {
        for directory in [false, true] {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), RAW).unwrap();
            let pending = storage.dir.join("pending-material-commit.json");
            if directory {
                fs::create_dir(&pending).unwrap();
            } else {
                fs::write(&pending, b"must not recover").unwrap();
            }
            let before = tree(&temp.root);
            assert!(read_progress_for_backup(&storage.dir).is_err());
            assert_eq!(
                read_progress_typed(&storage.dir, 10000)
                    .unwrap()
                    .unwrap()
                    .wire_version,
                1
            );
            assert_eq!(
                storage.read_progress().unwrap().chats[0].draft,
                "保持する下書き"
            );
            assert_eq!(tree(&temp.root), before);
        }
    }

    #[test]
    fn r05_unknown_malformed_duplicate_and_unknown_field_refuse_without_writes() {
        for raw in [
            v2("null").replace("\"version\":2", "\"version\":99"),
            "{".into(),
            v2("null").replacen('{', "{\"version\":2,", 1),
            v2("null").replacen('{', "{\"audio_history\":null,", 1),
            v2("null").replacen('{', "{\"unknown\":true,", 1),
        ] {
            let temp = Temp::new();
            let root = temp.root.join("host");
            fs::create_dir(&root).unwrap();
            fs::write(root.join("progress.json"), &raw).unwrap();
            assert!(
                serde_json::from_str::<Progress>(&raw).is_err(),
                "negative fixture"
            );
            let before = tree(&temp.root);
            assert!(read_progress_for_backup(&root).is_err());
            assert!(read_progress_typed(&root, 10000).is_err());
            assert_eq!(tree(&temp.root), before);
        }
    }

    #[test]
    fn r05_exact_bound_and_production_oversize_remain_strict() {
        let temp = Temp::new();
        let root = temp.root.join("host");
        fs::create_dir(&root).unwrap();
        let path = root.join("progress.json");
        fs::write(&path, RAW).unwrap();
        let before = tree(&temp.root);
        assert!(read_progress_for_backup(&root).unwrap().is_some());
        assert!(read_progress_typed(&root, RAW.len() as u64)
            .unwrap()
            .is_some());
        assert_eq!(
            read_progress_typed(&root, RAW.len() as u64 - 1).unwrap_err(),
            ProgressReadError::LimitExceeded
        );
        assert_eq!(tree(&temp.root), before);
        let file = File::create(&path).unwrap();
        file.set_len(crate::commit::PROGRESS_LIMIT + 1).unwrap();
        drop(file);
        let before = tree(&temp.root);
        assert!(read_progress_for_backup(&root).is_err());
        assert_eq!(
            read_progress_typed(&root, crate::commit::PROGRESS_LIMIT).unwrap_err(),
            ProgressReadError::LimitExceeded
        );
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn r05_actual_sharing_denial_is_io_and_retry_reads_identical_bytes() {
        let temp = Temp::new();
        let root = temp.root.join("host");
        fs::create_dir(&root).unwrap();
        let path = root.join("progress.json");
        fs::write(&path, RAW).unwrap();
        let before = tree(&temp.root);
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert_eq!(
            fs::read(&path).unwrap_err().raw_os_error(),
            Some(32),
            "fixture must fail real read"
        );
        let backup = read_progress_for_backup(&root);
        let strict = read_progress_typed(&root, 10000);
        drop(held);
        assert!(backup.is_err());
        assert_eq!(strict.unwrap_err(), ProgressReadError::Io);
        assert_eq!(tree(&temp.root), before);
        assert_eq!(
            read_progress_for_backup(&root).unwrap().unwrap().chats[0].draft,
            "保持する下書き"
        );
        assert_eq!(tree(&temp.root), before);
    }

    #[test]
    fn b_wb1_write_block_refuses_both_backup_entries_without_parent_contamination() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        const CASE: &str = "store::i2_backup_path_tests::b_wb1_write_block_refuses_both_backup_entries_without_parent_contamination";
        const CHILD: &str = "WW5_I2_BACKUP_WRITE_BLOCK_CHILD";
        const MARKER: &str = "WW5_BACKUP_WRITE_BLOCK_CHILD_OK";
        if std::env::var(CHILD).as_deref() == Ok(CASE) {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), RAW).unwrap();
            let mut current: Progress = serde_json::from_str(RAW).unwrap();
            current.chats[0].draft = "保存停止中の未保存下書き".into();
            current.validate().unwrap();
            assert!(storage.read_progress().unwrap().audio_history.is_none());
            assert!(current.audio_history.is_none());
            let deck = crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap();
            let destination = temp.root.join("external-backup");
            let caller = serde_json::to_value(&current).unwrap();
            let before = tree(&temp.root);
            let guard = acquire_progress_write().unwrap();
            guard.block_writes();
            drop(guard);

            assert!(crate::backup::preserve_current_progress(&storage, &current).is_err());
            assert_eq!(tree(&temp.root), before, "memory refusal must not write");
            assert_eq!(serde_json::to_value(&current).unwrap(), caller);
            assert!(!destination.exists());
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::WriteBlocked
            );

            // b-B interim contract; I2d-H/B owns future external-export availability.
            assert!(crate::backup::export(&storage.dir, &destination, &deck, &current).is_err());
            assert_eq!(tree(&temp.root), before, "export refusal must not write");
            assert_eq!(serde_json::to_value(&current).unwrap(), caller);
            assert!(!destination.exists());
            assert_eq!(
                acquire_progress_write().unwrap_err(),
                ProgressGateError::WriteBlocked
            );
            drop(storage);
            drop(temp);
            println!("{MARKER}");
            return;
        }

        let guard = acquire_progress_write().expect("parent begins writable");
        assert_eq!(guard.ensure_writable(), Ok(()));
        drop(guard);
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([CASE, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, CASE)
            .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
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
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "B-WB1 child status={} timeout={timed_out}\n{stdout}\n{stderr}",
            output.status
        );
        assert!(!timed_out, "child killed/reaped after 15s");
        assert!(output.status.success(), "child must complete successfully");
        assert!(
            stdout.contains("running 1 test")
                && stdout.contains("1 passed; 0 failed")
                && stdout.contains(MARKER),
            "zero-case success must not pass: {stdout}"
        );
        assert!(
            tree(&temp.root).is_empty(),
            "child-owned fixtures must be removed"
        );
        let guard = acquire_progress_write().expect("child block must not contaminate parent");
        assert_eq!(guard.ensure_writable(), Ok(()));
    }
}

#[cfg(all(test, windows))]
mod i2_progress_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};
    use crate::store::i2_migration_tests::RAW_V1;
    use std::{
        os::windows::fs::OpenOptionsExt,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    const MIN_V1: &str = r#"{"version":1,"settings":{},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
    const ROOT: &str = r#"{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":12}"#;
    fn v2(history: &str) -> String {
        MIN_V1.replace(
            "\"version\":1",
            &format!("\"version\":2,\"audio_history\":{history}"),
        )
    }
    fn raw_full_v1() -> String {
        // Literal sentinels, not a Progress serializer: all legacy top-level fields.
        RAW_V1.replace(r#""settings":{"topic":"合成・日本語"}"#, r#""settings":{"codex_path":"fixture-codex","codex_model":"fixture-model","codex_effort":"high","examples_per_word":7,"batch_words":8,"minutes":9,"new_per_day":4,"skills":["Usage"],"topic":"合成・日本語","font_scale":1.25,"ai_daily_limit":11,"voice_id":"fixture-voice","slow_speech":true,"speech_rate":1.5,"speech_volume":0.75,"speech_repeat":true,"page_tint":{"rgb":[1,2,3],"depth":21},"input_tint":{"rgb":[4,5,6],"depth":22}}"#)
            .replace(r#""reviews":[]"#, r#""reviews":[{"key":"lesson|Recall","at":12,"date":"2026-10-08","grade":"Good","assisted":true,"method":"fixture-method","first":false,"elapsed_days":1.25,"seconds":13,"self_assessed":true}]"#)
            .replacen("{", concat!("{",
                r#""deleted_entries":["deleted-sentinel"],"chat_action":["fixture-chat",{"operation":"Append","base":"base-sentinel","entry_id":"kept-entry"}],"deck_versions":{"deck-key":"deck-revision"},"writing_logs":[{"key":"writing-key","at":14,"problem":{"japanese":"合成問題","situation":"situation","target":"target","reference":"reference"},"answer":"answer-sentinel","feedback":"feedback-sentinel","revision":true}],"material_draft":{"mode":"New","baseline":null,"candidate":{"id":"draft-id","base":"draft-base","meaning":"draft-meaning","level":"draft-level","business":"draft-business","elevated":"draft-elevated","register":"draft-register","usage":"draft-usage","context":"draft-context","example":"draft-example","translation":"draft-translation","answers":["draft-answer"],"question":"draft-question","explanation":"draft-explanation","tag":"draft-tag"},"source":{"entry_id":"draft-id","conversation_id":"fixture-chat","exchange_indices":[],"at":15,"mode":"New"},"notices":["draft-notice"]},"#), 1)
    }
    fn subset(expected: &serde_json::Value, actual: &serde_json::Value, path: &str) {
        match expected {
            serde_json::Value::Object(fields) => {
                for (key, value) in fields {
                    if path.is_empty() && key == "version" {
                        continue;
                    }
                    subset(
                        value,
                        actual
                            .get(key)
                            .unwrap_or_else(|| panic!("lost {path}/{key}")),
                        &format!("{path}/{key}"),
                    );
                }
            }
            serde_json::Value::Array(values) => {
                let found = actual.as_array().expect(path);
                assert_eq!(found.len(), values.len(), "{path}");
                for (i, (a, b)) in values.iter().zip(found).enumerate() {
                    subset(a, b, &format!("{path}/{i}"));
                }
            }
            _ => assert_eq!(actual, expected, "{path}"),
        }
    }
    fn kind(bytes: &[u8]) -> Result<(), ProgressReadError> {
        decode_progress(bytes).map(|_| ())
    }

    #[test]
    fn s01_all_legacy_fields_survive_public_and_typed_normalization() {
        let raw = raw_full_v1();
        let expected: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let public: Progress =
            serde_json::from_str(&raw).expect("handwritten full source must decode");
        public
            .validate()
            .expect("full source must be structurally valid");
        subset(&expected, &serde_json::to_value(&public).unwrap(), "");
        assert_eq!(public.version, 2);
        let decoded = decode_progress(raw.as_bytes()).unwrap();
        assert_eq!(decoded.wire_version, 1);
        assert_eq!(decoded.progress.version, 2);
        subset(
            &expected,
            &serde_json::to_value(&decoded.progress).unwrap(),
            "",
        );
        let normalized =
            normalize_progress(serde_json::from_str::<ProgressWire>(&raw).unwrap()).unwrap();
        assert_eq!(normalized.wire_version, 1);
        assert_eq!(normalized.progress.version, 2);
        subset(
            &expected,
            &serde_json::to_value(normalized.progress).unwrap(),
            "",
        );
    }
    #[test]
    fn s01_missing_legacy_fields_and_settings_keep_exact_defaults() {
        let p: Progress = serde_json::from_str(MIN_V1).unwrap();
        let s = &p.settings;
        assert_eq!(
            (&s.codex_path, &s.codex_model, &s.codex_effort),
            (&"codex".to_owned(), &String::new(), &String::new())
        );
        assert_eq!(
            (
                s.examples_per_word,
                s.batch_words,
                s.minutes,
                s.new_per_day,
                s.ai_daily_limit
            ),
            (6, 5, 5, 3, 10)
        );
        assert_eq!(s.skills, [Skill::Recall, Skill::Usage]);
        assert_eq!(s.topic, "すべて");
        assert_eq!(s.font_scale, 0.8);
        assert_eq!(
            (
                s.voice_id.as_str(),
                s.slow_speech,
                s.speech_rate,
                s.speech_volume,
                s.speech_repeat
            ),
            ("", false, None, 0.5, false)
        );
        assert_eq!(
            s.page_tint,
            TintChoice {
                rgb: [0, 0, 0],
                depth: 0
            }
        );
        assert_eq!(
            s.input_tint,
            TintChoice {
                rgb: [0, 0, 0],
                depth: 0
            }
        );
        assert!(
            p.deleted_entries.is_empty()
                && p.chat_action.is_none()
                && p.deck_versions.is_empty()
                && p.writing_logs.is_empty()
                && p.japanese_drafts.is_empty()
                && p.chats.is_empty()
                && p.material_draft.is_none()
                && p.material_sources.is_empty()
                && p.audio_history.is_none()
        );
        assert_eq!(p.version, 2);
        assert_eq!(decode_progress(MIN_V1.as_bytes()).unwrap().wire_version, 1);
    }
    #[test]
    fn s02_null_and_structural_root_are_accepted_without_closure_io() {
        for history in ["null", ROOT] {
            let raw = v2(history);
            let p: Progress = serde_json::from_str(&raw).unwrap();
            p.validate().unwrap();
            let expected: Option<RootRef> = serde_json::from_str(history).unwrap();
            assert_eq!(p.audio_history, expected);
            let decoded = decode_progress(raw.as_bytes()).unwrap();
            assert_eq!(decoded.wire_version, 2);
            assert_eq!(decoded.progress.audio_history, expected);
            let serialized = serde_json::to_value(&p).unwrap();
            assert_eq!(
                serialized.get("audio_history"),
                Some(&serde_json::from_str::<serde_json::Value>(history).unwrap())
            );
        }
        let v1null = MIN_V1.replacen("{", "{\"audio_history\":null,", 1);
        assert_eq!(decode_progress(v1null.as_bytes()).unwrap().wire_version, 1);
    }
    #[test]
    fn s02_strict_raw_documents_reject_duplicates_missing_types_and_invalid_structure() {
        let mut cases = vec![
            v2("null").replace(",\"audio_history\":null", ""),
            MIN_V1.replacen("{", &format!("{{\"audio_history\":{ROOT},"), 1),
            v2("null").replacen("{", "{\"version\":2,", 1),
            v2("null").replacen("{", "{\"audio_history\":null,", 1),
            v2("null").replacen("{", "{\"unrecognized\":0,", 1),
            v2("{}"),
            v2(&ROOT.replace("\"bytes\":12", "\"bytes\":0")),
            v2(&ROOT.replace("\"bytes\":12", "\"bytes\":8388609")),
            v2(&ROOT.replace("aaaaaaaa", "zzzzzzzz")),
            format!("{} false", v2("null")),
            "{".into(),
            MIN_V1.replace("\"settings\":{}", "\"settings\":{\"minutes\":0}"),
        ];
        for replacement in [
            "",
            "\"version\":null,",
            "\"version\":\"1\",",
            "\"version\":1.0,",
        ] {
            cases.push(MIN_V1.replace("\"version\":1,", replacement));
        }
        for field in [
            "settings",
            "memories",
            "reviews",
            "suspended",
            "ai_calls",
            "study_seconds",
        ] {
            let mut value: serde_json::Value = serde_json::from_str(MIN_V1).unwrap();
            value.as_object_mut().unwrap().remove(field);
            cases.push(value.to_string());
        }
        let observations: Vec<_> = cases
            .iter()
            .map(|raw| {
                (
                    kind(raw.as_bytes()),
                    serde_json::from_str::<Progress>(raw).is_err(),
                )
            })
            .collect();
        for (i, (typed, public_rejected)) in observations.into_iter().enumerate() {
            assert_eq!(typed, Err(ProgressReadError::InvalidDocument), "row {i}");
            assert!(public_rejected, "row {i}");
        }
    }
    #[test]
    fn s02_well_shaped_unknown_version_is_typed_unsupported() {
        let raw = v2("null").replace("\"version\":2", "\"version\":99");
        assert_eq!(
            kind(raw.as_bytes()),
            Err(ProgressReadError::UnsupportedVersion)
        );
        assert!(serde_json::from_str::<Progress>(&raw).is_err());
    }
    #[test]
    fn s03_typed_read_has_exact_limit_and_only_verified_missing_leaf_is_none() {
        let temp = Temp::new();
        let root = temp.root.join("host");
        fs::create_dir(&root).unwrap();
        assert!(read_progress_typed(&root, 1).unwrap().is_none());
        fs::write(root.join("progress.json"), MIN_V1).unwrap();
        let before = tree(&temp.root);
        let exact = read_progress_typed(&root, MIN_V1.len() as u64)
            .unwrap()
            .unwrap();
        assert_eq!((exact.wire_version, exact.progress.version), (1, 2));
        assert_eq!(
            read_progress_typed(&root, MIN_V1.len() as u64 - 1).unwrap_err(),
            ProgressReadError::LimitExceeded
        );
        assert!(tree(&temp.root) == before);
    }
    #[test]
    fn s03_unsafe_and_missing_paths_never_become_empty_history() {
        let mut observed = vec![];
        for placement in 0..8 {
            let mut temp = Temp::new();
            let parent = temp.root.join("parent");
            let root = parent.join("host");
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("progress.json"), MIN_V1).unwrap();
            let mut exposed = root.clone();
            let expected = if placement < 2 {
                ProgressReadError::Io
            } else {
                ProgressReadError::UnsafePath
            };
            match placement {
                0 => exposed = temp.root.join("absent"),
                1 => exposed = temp.root.join("absent/host"),
                2 => {
                    exposed = temp.root.join("file-root");
                    fs::write(&exposed, b"file").unwrap();
                }
                3 => {
                    let file = temp.root.join("file-parent");
                    fs::write(&file, b"file").unwrap();
                    exposed = file.join("host");
                }
                4 => {
                    fs::remove_file(root.join("progress.json")).unwrap();
                    fs::create_dir(root.join("progress.json")).unwrap();
                }
                5 => {
                    let link = temp.root.join("alias");
                    temp.junction(&link, &root);
                    exposed = link;
                }
                6 => {
                    let link = temp.root.join("alias");
                    temp.junction(&link, &parent);
                    exposed = link.join("host");
                }
                _ => {
                    fs::remove_file(root.join("progress.json")).unwrap();
                    let target = temp.root.join("target");
                    fs::create_dir(&target).unwrap();
                    temp.junction(&root.join("progress.json"), &target);
                }
            }
            let before = tree(&temp.root);
            let result = read_progress_typed(&exposed, 10000).map(|_| ());
            observed.push((placement, result, expected, tree(&temp.root) == before));
        }
        for (placement, result, expected, same) in observed {
            assert_eq!(result, Err(expected), "placement {placement}");
            assert!(same);
        }
    }
    #[test]
    fn s03_real_read_denial_is_io_for_typed_read_public_read_and_head() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        let path = storage.dir.join("progress.json");
        fs::write(&path, MIN_V1).unwrap();
        let before = tree(&temp.root);
        let held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        let error = fs::read(&path).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(32));
        eprintln!(
            "S read fixture: actual read denied with {:?}",
            error.raw_os_error()
        );
        let typed = read_progress_typed(&storage.dir, 10000).map(|_| ());
        let public = storage.read_progress();
        let head = ProgressHistoryHead.read_current(&storage);
        drop(held);
        assert_eq!(typed, Err(ProgressReadError::Io));
        assert!(public.is_err());
        assert_eq!(head.unwrap_err().kind, HistoryErrorKind::Io);
        assert!(tree(&temp.root) == before);
    }
    #[test]
    fn s03_head_preserves_typed_error_categories() {
        for (raw, expected) in [
            ("{".to_owned(), HistoryErrorKind::InvalidDocument),
            (
                v2("null").replace("\"version\":2", "\"version\":99"),
                HistoryErrorKind::UnsupportedVersion,
            ),
        ] {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), raw).unwrap();
            let before = tree(&temp.root);
            assert_eq!(
                ProgressHistoryHead.read_current(&storage).unwrap_err().kind,
                expected
            );
            assert!(storage.read_progress().is_err());
            assert!(tree(&temp.root) == before);
        }
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        fs::create_dir(storage.dir.join("progress.json")).unwrap();
        let before = tree(&temp.root);
        assert_eq!(
            ProgressHistoryHead.read_current(&storage).unwrap_err().kind,
            HistoryErrorKind::UnsafePath
        );
        assert!(tree(&temp.root) == before);
    }
    #[test]
    fn s03_head_reports_limit_exceeded_without_reading_or_rewriting_large_leaf() {
        let temp = Temp::new();
        let storage = Storage::at(temp.root.join("host")).unwrap();
        let path = storage.dir.join("progress.json");
        let file = File::create(&path).unwrap();
        file.set_len(crate::commit::PROGRESS_LIMIT + 1).unwrap();
        drop(file);
        let result = ProgressHistoryHead.read_current(&storage);
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            crate::commit::PROGRESS_LIMIT + 1
        );
        assert_eq!(fs::read_dir(&storage.dir).unwrap().count(), 2);
        assert_eq!(result.unwrap_err().kind, HistoryErrorKind::LimitExceeded);
    }
    #[test]
    fn s03_write_blocked_pure_read_uses_no_gate_and_leaves_pending_untouched() {
        const CASE:&str="store::i2_progress_tests::s03_write_blocked_pure_read_uses_no_gate_and_leaves_pending_untouched";
        const CHILD: &str = "WW5_I2_PROGRESS_CHILD";
        if std::env::var(CHILD).as_deref() == Ok(CASE) {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            fs::write(storage.dir.join("progress.json"), MIN_V1).unwrap();
            fs::write(
                storage.dir.join("pending-material-commit.json"),
                b"must never recover",
            )
            .unwrap();
            let before = tree(&temp.root);
            let guard = acquire_progress_write().unwrap();
            guard.block_writes();
            drop(guard);
            assert_eq!(storage.read_progress().unwrap().version, 2);
            assert_eq!(ProgressHistoryHead.read_current(&storage).unwrap(), None);
            assert!(storage.load().is_err());
            assert!(tree(&temp.root) == before);
            println!("WW5_PROGRESS_CHILD_OK");
            return;
        }
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([CASE, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, CASE)
            .env("WW5_I2_GATE_LIB_ROOT", &temp.root)
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
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!timed_out, "child killed/reaped after 15s");
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(
            stdout.contains("running 1 test")
                && stdout.contains("1 passed; 0 failed")
                && stdout.contains("WW5_PROGRESS_CHILD_OK")
        );
    }
}

#[cfg(all(test, windows))]
mod i2_normal_save_tests {
    use super::*;
    use crate::{
        audio_history::*,
        store::i2_gate_tests::{tree, Temp},
        store::i2_migration_tests::{Faults, HASH, RAW_V1},
    };
    use std::{
        cell::RefCell,
        os::windows::fs::OpenOptionsExt,
        process::{Command, Stdio},
        sync::{atomic::AtomicUsize, mpsc, Arc, Barrier},
        time::{Duration, Instant},
    };
    const LIMIT: u64 = 100_000;
    const A: &[u8] =
        br#"{"schema_version":1,"revision":0,"next_save_sequence":1,"trials":{},"receipts":{}}"#;
    const A_HASH: &str = "ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0";
    const MIN: &str = r#"{"version":2,"audio_history":null,"settings":{},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
    struct Fixture {
        storage: Storage,
        temp: Temp,
        a: RootRef,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = Temp::new();
            let storage = Storage::at(temp.root.join("host")).unwrap();
            let a = RootRef::new(BlobRef {
                sha256: Sha256::parse(A_HASH).unwrap(),
                bytes: 82,
            });
            fs::create_dir_all(storage.dir.join("audio-history/assets")).unwrap();
            fs::write(blob_path(&storage.dir, a.as_blob()), A).unwrap();
            let raw = MIN.replace(
                "\"audio_history\":null",
                &format!(r#""audio_history":{{"sha256":"{A_HASH}","bytes":82}}"#),
            );
            fs::write(storage.dir.join("progress.json"), raw).unwrap();
            fs::write(storage.dir.join("custom.tsv"), crate::model::BUILTIN_DECK).unwrap();
            fs::write(
                storage.dir.join("keep.bin"),
                b"synthetic outside-history bytes",
            )
            .unwrap();
            let actual = HistoryReader::new(&storage)
                .verify_closure(&a)
                .expect("handwritten root fixture");
            assert_eq!(actual.objects, vec![a.as_blob().clone()]);
            assert_eq!(actual.total_bytes, 82);
            Self { storage, temp, a }
        }
        fn path(&self) -> PathBuf {
            self.storage.dir.join("progress.json")
        }
        fn ledger(&self) -> BTreeMap<PathBuf, Vec<u8>> {
            tree(&self.temp.root)
        }
        fn candidate(&self) -> Progress {
            let mut p: Progress = serde_json::from_str(MIN).unwrap();
            p.settings.topic = "candidate topic".into();
            p
        }
        fn next(&self, p: &Progress) -> Vec<u8> {
            let mut next = p.clone();
            next.audio_history = Some(self.a.clone());
            serde_json::to_vec_pretty(&next).unwrap()
        }
        fn daily(&self) -> PathBuf {
            self.storage.dir.join("backups").join(format!(
                "progress-{}.json",
                chrono::Local::now().format("%Y-%m-%d")
            ))
        }
        fn hash(&self) -> PathBuf {
            self.storage
                .dir
                .join(format!("backups/progress-v1-{HASH}.json"))
        }
        fn with_trial(&mut self) -> (BlobRef, BlobRef) {
            let original = put(&self.storage.dir, b"independent synthetic original");
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
            let trial_ref = put(&self.storage.dir, &serde_json::to_vec(&trial).unwrap());
            let root = HistoryRoot {
                schema_version: 1,
                revision: 1,
                next_save_sequence: 2,
                trials: BTreeMap::from([(
                    trial.id,
                    TrialHead {
                        document: trial_ref.clone(),
                        admitted_sequence: 1,
                    },
                )]),
                receipts: BTreeMap::new(),
            };
            root.validate().unwrap();
            self.a = RootRef::new(put(&self.storage.dir, &serde_json::to_vec(&root).unwrap()));
            let mut current: Progress = serde_json::from_str(MIN).unwrap();
            current.audio_history = Some(self.a.clone());
            fs::write(self.path(), serde_json::to_vec(&current).unwrap()).unwrap();
            let mut expected = vec![
                self.a.as_blob().clone(),
                trial_ref.clone(),
                original.clone(),
            ];
            expected.sort_by(|a, b| a.sha256.cmp(&b.sha256));
            let actual = HistoryReader::new(&self.storage)
                .verify_closure(&self.a)
                .unwrap();
            assert_eq!(actual.objects, expected);
            assert_eq!(
                actual.total_bytes,
                expected.iter().map(|r| r.bytes).sum::<u64>()
            );
            (original, trial_ref)
        }
    }
    fn digest(bytes: &[u8]) -> Sha256 {
        Sha256::parse(&crate::assets::sha256(bytes).unwrap()).unwrap()
    }
    fn blob_path(root: &Path, r: &BlobRef) -> PathBuf {
        root.join("audio-history/assets")
            .join(format!("{}.bin", r.sha256.as_str()))
    }
    fn put(root: &Path, bytes: &[u8]) -> BlobRef {
        let r = BlobRef {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
        };
        fs::write(blob_path(root, &r), bytes).unwrap();
        r
    }
    fn writable() {
        let guard =
            acquire_progress_write().expect("pre-publish/known-old failure must not set block");
        let state = guard.ensure_writable();
        drop(guard);
        assert_eq!(state, Ok(()));
    }
    fn isolated(case: &str, body: impl FnOnce()) {
        const CHILD: &str = "WW5_I2_NORMAL_CHILD";
        if std::env::var(CHILD).as_deref() == Ok(case) {
            body();
            println!("WW5_NORMAL_CHILD_OK:{case}");
            return;
        }
        let temp = Temp::new();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([case, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, case)
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
        assert!(!timeout, "child killed/reaped at 15s\n{out}\n{err}");
        assert!(output.status.success(), "{out}\n{err}");
        assert!(
            out.contains("running 1 test")
                && out.contains("1 passed; 0 failed")
                && out.contains(&format!("WW5_NORMAL_CHILD_OK:{case}"))
        );
    }

    #[test]
    fn n02_raw_reader_returns_exact_unparsed_bytes_and_verified_absence() {
        let f = Fixture::new();
        let malformed = b"{ deliberately undecodable raw bytes";
        fs::write(f.path(), malformed).unwrap();
        assert!(decode_progress(malformed).is_err());
        let before = f.ledger();
        assert_eq!(
            read_progress_bytes(&f.storage.dir, malformed.len() as u64).unwrap(),
            Some(malformed.to_vec())
        );
        assert!(f.ledger() == before);
        assert_eq!(
            read_progress_bytes(&f.storage.dir, malformed.len() as u64 - 1),
            Err(ProgressReadError::LimitExceeded)
        );
        fs::remove_file(f.path()).unwrap();
        assert_eq!(read_progress_bytes(&f.storage.dir, LIMIT).unwrap(), None);
    }
    #[test]
    fn n02_preflight_rejections_preserve_all_bytes_and_do_not_publish_or_block() {
        for case in 0..11 {
            let mut f = Fixture::new();
            let mut candidate = f.candidate();
            let mut limit = LIMIT;
            match case {
                0 => fs::write(f.path(), MIN.replace("\"version\":2", "\"version\":99")).unwrap(),
                1 => fs::write(f.path(), b"broken metadata").unwrap(),
                2 => {
                    fs::remove_file(blob_path(&f.storage.dir, f.a.as_blob())).unwrap();
                    assert!(HistoryReader::new(&f.storage).verify_closure(&f.a).is_err());
                }
                3 => {
                    let (_, trial_ref) = f.with_trial();
                    let raw = fs::read_to_string(blob_path(&f.storage.dir, &trial_ref)).unwrap();
                    let unknown = put(
                        &f.storage.dir,
                        raw.replace("\"schema_version\":1", "\"schema_version\":99")
                            .as_bytes(),
                    );
                    let mut root: HistoryRoot = serde_json::from_slice(
                        &fs::read(blob_path(&f.storage.dir, f.a.as_blob())).unwrap(),
                    )
                    .unwrap();
                    root.trials.values_mut().next().unwrap().document = unknown;
                    f.a = RootRef::new(put(&f.storage.dir, &serde_json::to_vec(&root).unwrap()));
                    let mut p: Progress = serde_json::from_str(MIN).unwrap();
                    p.audio_history = Some(f.a.clone());
                    fs::write(f.path(), serde_json::to_vec(&p).unwrap()).unwrap();
                    assert!(HistoryReader::new(&f.storage)
                        .read_snapshot(Some(&f.a))
                        .root
                        .is_some());
                    assert_eq!(
                        HistoryReader::new(&f.storage)
                            .verify_closure(&f.a)
                            .unwrap_err()
                            .kind,
                        HistoryErrorKind::UnsupportedVersion
                    );
                }
                4 | 5 => {
                    let (original, _) = f.with_trial();
                    if case == 4 {
                        fs::remove_file(blob_path(&f.storage.dir, &original)).unwrap();
                    } else {
                        fs::write(blob_path(&f.storage.dir, &original), b"damaged original")
                            .unwrap();
                    }
                    assert!(HistoryReader::new(&f.storage)
                        .read_snapshot(Some(&f.a))
                        .root
                        .is_some());
                    assert!(HistoryReader::new(&f.storage).verify_closure(&f.a).is_err());
                }
                6 => candidate.settings.minutes = 0,
                7 => fs::write(
                    f.storage.dir.join("pending-material-commit.json"),
                    b"pending must not be recovered",
                )
                .unwrap(),
                8 => fs::create_dir(f.storage.dir.join("pending-material-commit.json")).unwrap(),
                9 => {
                    fs::write(f.path(), format!("{MIN}{}", " ".repeat(8192))).unwrap();
                    limit = 4096;
                }
                _ => {
                    limit = f.next(&candidate).len() as u64 - 1;
                    assert!(fs::metadata(f.path()).unwrap().len() < limit);
                }
            }
            let before = f.ledger();
            let input = serde_json::to_vec(&candidate).unwrap();
            let calls = Cell::new(0);
            let faults = Faults::new(None);
            let result =
                f.storage
                    .save_progress_with_publisher(&candidate, limit, &faults, |_, _| {
                        calls.set(calls.get() + 1);
                        Ok(())
                    });
            assert!(result.is_err(), "preflight case {case}");
            assert_eq!(calls.get(), 0, "case {case}");
            assert!(faults.seen.borrow().is_empty());
            assert!(f.ledger() == before, "case {case}");
            assert_eq!(serde_json::to_vec(&candidate).unwrap(), input);
            writable();
        }
    }
    #[test]
    fn n02_exact_completed_candidate_limit_succeeds_with_root_inheritance() {
        let f = Fixture::new();
        let candidate = f.candidate();
        let expected = f.next(&candidate);
        let faults = Faults::new(None);
        let saved = f
            .storage
            .save_progress_with_limit_and_faults(&candidate, expected.len() as u64, &faults)
            .unwrap();
        assert_eq!(fs::read(f.path()).unwrap(), expected);
        assert_eq!(saved.audio_history, Some(f.a));
        assert!(
            faults.seen.borrow().is_empty(),
            "N must not fire pointer checkpoints or migration for old v2"
        );
    }
    #[test]
    fn n03_v1_exact_hash_daily_backup_and_migration_fault_boundaries() {
        for fail in [
            None,
            Some(ProgressCheckpoint::BeforeMigrationBackup),
            Some(ProgressCheckpoint::AfterMigrationBackup),
        ] {
            let f = Fixture::new();
            fs::write(f.path(), RAW_V1).unwrap();
            let before = f.ledger();
            let candidate: Progress = serde_json::from_str(RAW_V1).unwrap();
            let faults = Faults::new(fail);
            let saved = f
                .storage
                .save_progress_with_limit_and_faults(&candidate, LIMIT, &faults);
            let expected = if fail == Some(ProgressCheckpoint::BeforeMigrationBackup) {
                vec![ProgressCheckpoint::BeforeMigrationBackup]
            } else {
                vec![
                    ProgressCheckpoint::BeforeMigrationBackup,
                    ProgressCheckpoint::AfterMigrationBackup,
                ]
            };
            assert_eq!(*faults.seen.borrow(), expected);
            if fail.is_none() {
                let saved = saved.unwrap();
                assert_eq!(saved.version, 2);
                assert_eq!(saved.audio_history, None);
                assert_eq!(fs::read(f.hash()).unwrap(), RAW_V1.as_bytes());
                assert_eq!(fs::read(f.daily()).unwrap(), RAW_V1.as_bytes());
                assert_eq!(
                    fs::read_dir(f.storage.dir.join("backups")).unwrap().count(),
                    2
                );
                assert_eq!(
                    fs::read(f.path()).unwrap(),
                    serde_json::to_vec_pretty(&saved).unwrap()
                );
            } else {
                assert!(saved.is_err());
                let mut expected = before;
                if fail == Some(ProgressCheckpoint::AfterMigrationBackup) {
                    expected.insert(PathBuf::from("host/backups"), vec![]);
                    expected.insert(
                        PathBuf::from(format!("host/backups/progress-v1-{HASH}.json")),
                        RAW_V1.as_bytes().to_vec(),
                    );
                }
                assert!(f.ledger() == expected);
            }
            writable();
        }
    }
    #[test]
    fn n03b_daily_directory_rejection_retains_only_prior_verified_hash_backup() {
        for mode in 0..3 {
            let f = Fixture::new();
            let old = if mode == 2 { MIN } else { RAW_V1 };
            fs::write(f.path(), old).unwrap();
            fs::create_dir_all(f.daily()).unwrap();
            if mode == 1 {
                fs::write(f.hash(), RAW_V1).unwrap();
            }
            let before = f.ledger();
            let calls = Cell::new(0);
            let faults = Faults::new(None);
            let result =
                f.storage
                    .save_progress_with_publisher(&f.candidate(), LIMIT, &faults, |_, _| {
                        calls.set(calls.get() + 1);
                        Ok(())
                    });
            assert!(result.is_err());
            assert_eq!(calls.get(), 0);
            let mut expected = before;
            if mode == 0 {
                expected.insert(
                    PathBuf::from(format!("host/backups/progress-v1-{HASH}.json")),
                    RAW_V1.as_bytes().to_vec(),
                );
            }
            assert!(
                f.ledger() == expected,
                "mode {mode}: first migration hash is retained, not rolled back"
            );
            writable();
        }
    }
    #[test]
    fn n03b_daily_reparse_is_rejected_and_existing_regular_daily_is_not_replaced() {
        let mut f = Fixture::new();
        fs::create_dir(f.storage.dir.join("backups")).unwrap();
        let target = f.temp.root.join("daily-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"outside daily target").unwrap();
        let daily = f.daily();
        f.temp.junction(&daily, &target);
        let before = f.ledger();
        let calls = Cell::new(0);
        assert!(f
            .storage
            .save_progress_with_publisher(&f.candidate(), LIMIT, &NoProgressFaults, |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            })
            .is_err());
        assert_eq!(calls.get(), 0);
        assert!(f.ledger() == before);
        writable();
        let f = Fixture::new();
        fs::create_dir(f.storage.dir.join("backups")).unwrap();
        fs::write(f.daily(), b"first daily backup retained").unwrap();
        let saved = f.storage.save_progress(&f.candidate()).unwrap();
        assert_eq!(saved.audio_history, Some(f.a.clone()));
        assert_eq!(fs::read(f.daily()).unwrap(), b"first daily backup retained");
    }

    #[test]
    fn n03c_actual_daily_atomic_io_failure_stops_metadata_without_new_block() {
        const SENTINEL: &[u8] = b"fixture-owned locked daily destination";
        const PROBE: &[u8] = b"fixture-owned independent rename probe";
        let f = Fixture::new();
        fs::write(f.path(), RAW_V1).unwrap();
        let before = f.ledger();
        let normal_calls = Cell::new(0);
        let daily_calls = Cell::new(0);
        let atomic_failed = Cell::new(false);
        let rename_failed = Cell::new(false);
        let rename_code = Cell::new(None);
        let read_code = Cell::new(None);
        let held = RefCell::new(None);
        let sent = RefCell::new(None);
        let faults = Faults::new(None);
        let result = f.storage.save_progress_with_publishers(
            &f.candidate(),
            LIMIT,
            &faults,
            |_, _| {
                normal_calls.set(normal_calls.get() + 1);
                Ok(())
            },
            |path, bytes| {
                daily_calls.set(daily_calls.get() + 1);
                *sent.borrow_mut() = Some((path.to_owned(), bytes.to_vec()));
                if path.parent() != Some(f.storage.dir.join("backups").as_path()) {
                    return Err("fixture refuses an out-of-scope daily path".into());
                }
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .read(true)
                    .write(true)
                    .share_mode(0)
                    .open(path)
                    .map_err(|e| e.to_string())?;
                file.write_all(SENTINEL).map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                read_code.set(fs::read(path).err().and_then(|e| e.raw_os_error()));
                let probe = path.parent().unwrap().join("fixture-rename-probe.bin");
                fs::write(&probe, PROBE).map_err(|e| e.to_string())?;
                let rename = fs::rename(&probe, path);
                rename_failed.set(rename.is_err());
                rename_code.set(rename.err().and_then(|e| e.raw_os_error()));
                let actual = atomic_write(path, bytes);
                atomic_failed.set(actual.is_err());
                *held.borrow_mut() = Some(file);
                actual
            },
        );
        drop(held.borrow_mut().take());
        assert_eq!(
            daily_calls.get(),
            1,
            "daily callback/real I/O fixture must be reached"
        );
        assert_eq!(normal_calls.get(), 0);
        assert!(
            atomic_failed.get(),
            "same production atomic_write must itself return Err"
        );
        assert!(result.is_err());
        eprintln!(
            "N03c actual daily atomic Err; independent rename OS={:?}, read OS={:?}",
            rename_code.get(),
            read_code.get()
        );
        assert_eq!(read_code.get(), Some(32));
        assert!(
            rename_failed.get(),
            "independent rename must fail while the destination is held"
        );
        let (daily, bytes) = sent.borrow().clone().unwrap();
        assert_eq!(bytes, RAW_V1.as_bytes());
        assert_eq!(fs::read(&daily).unwrap(), SENTINEL);
        assert_eq!(
            *faults.seen.borrow(),
            [
                ProgressCheckpoint::BeforeMigrationBackup,
                ProgressCheckpoint::AfterMigrationBackup
            ]
        );
        let mut expected = before;
        expected.insert(PathBuf::from("host/backups"), vec![]);
        expected.insert(
            PathBuf::from(format!("host/backups/progress-v1-{HASH}.json")),
            RAW_V1.as_bytes().to_vec(),
        );
        expected.insert(
            daily.strip_prefix(&f.temp.root).unwrap().to_owned(),
            SENTINEL.to_vec(),
        );
        expected.insert(
            PathBuf::from("host/backups/fixture-rename-probe.bin"),
            PROBE.to_vec(),
        );
        assert!(f.ledger()==expected,"only verified migration backup plus explicitly fixture-owned sentinel/probe may remain; atomic temps must be zero");
        writable();

        // All product-result assertions precede these fixture-only controls.
        // Reuse the exact source/destination, changing only the held handle.
        let probe = daily.parent().unwrap().join("fixture-rename-probe.bin");
        fs::rename(&probe, &daily).expect("same rename must succeed after releasing the handle");
        assert_eq!(fs::read(&daily).unwrap(), PROBE);
        expected.remove(&PathBuf::from("host/backups/fixture-rename-probe.bin"));
        let daily_relative = daily.strip_prefix(&f.temp.root).unwrap().to_owned();
        expected.insert(daily_relative.clone(), PROBE.to_vec());
        assert_eq!(f.ledger(), expected);
        atomic_write(&daily, &bytes)
            .expect("same atomic_write must succeed after releasing the handle");
        assert_eq!(fs::read(&daily).unwrap(), RAW_V1.as_bytes());
        expected.insert(daily_relative, RAW_V1.as_bytes().to_vec());
        assert_eq!(f.ledger(), expected);
        eprintln!("N03c unlocked same-path rename and atomic_write succeeded; product ledger/nonblock assertions completed first");
    }
    #[test]
    fn n04_no_write_ok_or_err_confirms_exact_old_without_block() {
        isolated(
            "store::i2_normal_save_tests::n04_no_write_ok_or_err_confirms_exact_old_without_block",
            || {
                for state in 0..3 {
                    for publisher_ok in [true, false] {
                        let f = Fixture::new();
                        if state == 1 {
                            fs::write(f.path(), RAW_V1).unwrap();
                        }
                        if state == 2 {
                            fs::remove_file(f.path()).unwrap();
                        }
                        let old = fs::read(f.path()).ok();
                        let calls = Cell::new(0);
                        let result = f.storage.save_progress_with_publisher(
                            &f.candidate(),
                            LIMIT,
                            &NoProgressFaults,
                            |_, _| {
                                calls.set(calls.get() + 1);
                                if publisher_ok {
                                    Ok(())
                                } else {
                                    Err("same ambiguous publisher message".into())
                                }
                            },
                        );
                        assert_eq!(calls.get(), 1);
                        assert!(result.is_err());
                        assert_eq!(fs::read(f.path()).ok(), old);
                        writable();
                    }
                }
            },
        );
    }
    #[test]
    fn n04_real_atomic_next_is_success_even_when_publisher_returns_err() {
        isolated("store::i2_normal_save_tests::n04_real_atomic_next_is_success_even_when_publisher_returns_err",||{
        for publisher_ok in [true,false]{let f=Fixture::new();let input=f.candidate();let expected=f.next(&input);let calls=Cell::new(0);let sent=RefCell::new(None);let faults=Faults::new(None);let result=f.storage.save_progress_with_publisher(&input,LIMIT,&faults,|path,bytes|{calls.set(calls.get()+1);*sent.borrow_mut()=Some((path.to_owned(),bytes.to_vec()));atomic_write(path,bytes)?;if publisher_ok{Ok(())}else{Err("same ambiguous publisher message".into())}});let saved=result.unwrap();assert_eq!(calls.get(),1);assert_eq!(*sent.borrow(),Some((f.path(),expected.clone())));assert_eq!(fs::read(f.path()).unwrap(),expected);assert_eq!(saved.audio_history,Some(f.a.clone()));assert!(faults.seen.borrow().is_empty());writable();}
    });
    }
    #[test]
    fn n04_equal_old_and_next_prioritizes_success_over_old_failure() {
        isolated("store::i2_normal_save_tests::n04_equal_old_and_next_prioritizes_success_over_old_failure",||{
        let f=Fixture::new();let candidate=f.candidate();let bytes=f.next(&candidate);fs::write(f.path(),&bytes).unwrap();let calls=Cell::new(0);let result=f.storage.save_progress_with_publisher(&candidate,LIMIT,&NoProgressFaults,|_,_|{calls.set(calls.get()+1);Err("same ambiguous publisher message".into())});assert!(result.is_ok());assert_eq!(calls.get(),1);assert_eq!(fs::read(f.path()).unwrap(),bytes);writable();
    });
    }

    fn assert_common_block(f: &mut Fixture, other: &Storage) {
        let gate = acquire_progress_write().map(|guard| drop(guard));
        assert_eq!(gate, Err(ProgressGateError::WriteBlocked));
        let before = f.ledger();
        let candidate = f.candidate();
        assert!(f.storage.save_progress(&candidate).is_err());
        assert!(f.storage.save(&candidate).is_err());
        assert!(f.storage.load().is_err());
        assert!(
            crate::commit::save(&f.storage.dir, crate::model::BUILTIN_DECK, &candidate).is_err()
        );
        assert!(other.save_progress(&candidate).is_err());
        assert!(other.save(&candidate).is_err());
        let original = f.storage.dir.clone();
        f.storage.dir = original.join(".");
        assert!(f.storage.save_progress(&candidate).is_err());
        f.storage.dir = original;
        let request = FrozenHistoryRequest::new(
            HistoryMutation::SetTrash {
                target: TrialTarget {
                    id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
                    expected: f.a.as_blob().clone(),
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
        assert!(
            f.ledger() == before,
            "blocked writers must not add or roll back bytes"
        );
        assert!(
            HistoryReader::new(&f.storage)
                .read_snapshot(Some(&f.a))
                .root
                .is_some(),
            "immutable history remains readable without the writer gate"
        );
    }
    fn third_state(mode: u8) {
        let mut f = Fixture::new();
        let other = Storage::at(f.temp.root.join("other")).unwrap();
        let candidate = f.candidate();
        let calls = Cell::new(0);
        let held = RefCell::new(None);
        let denied = Cell::new(None);
        if mode == 3 {
            let probe = f.temp.root.join("read-probe");
            fs::write(&probe, b"probe").unwrap();
            let lock = OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(&probe)
                .unwrap();
            let error = fs::read(&probe).unwrap_err();
            assert_eq!(error.raw_os_error(), Some(32));
            eprintln!("N read-denial fixture preflight: actual sharing violation 32");
            drop(lock);
            fs::remove_file(probe).unwrap();
        }
        let third = MIN.replace(
            "\"settings\":{}",
            "\"settings\":{\"topic\":\"third-state\"}",
        );
        let result = f.storage.save_progress_with_publisher(
            &candidate,
            LIMIT,
            &NoProgressFaults,
            |path, bytes| {
                calls.set(calls.get() + 1);
                match mode {
                    0 => fs::write(path, third.as_bytes()).map_err(|e| e.to_string())?,
                    1 => fs::write(path, b"broken third state").map_err(|e| e.to_string())?,
                    2 => fs::remove_file(path).map_err(|e| e.to_string())?,
                    _ => {
                        atomic_write(path, bytes)?;
                        let file = OpenOptions::new()
                            .read(true)
                            .share_mode(0)
                            .open(path)
                            .map_err(|e| e.to_string())?;
                        denied.set(fs::read(path).err().and_then(|e| e.raw_os_error()));
                        *held.borrow_mut() = Some(file);
                    }
                }
                Err("same ambiguous publisher message".into())
            },
        );
        drop(held.borrow_mut().take());
        assert_eq!(
            calls.get(),
            1,
            "normal publisher must be reached before classifying third state"
        );
        assert!(result.is_err());
        if mode == 3 {
            assert_eq!(denied.get(), Some(32));
            eprintln!("N actual published-file readback denied: sharing violation 32");
        }
        match mode {
            0 => {
                assert_eq!(fs::read(f.path()).unwrap(), third.as_bytes());
                assert_eq!(
                    f.storage.read_progress().unwrap().settings.topic,
                    "third-state"
                );
            }
            1 => {
                assert_eq!(fs::read(f.path()).unwrap(), b"broken third state");
                assert!(f.storage.read_progress().is_err());
                assert_eq!(
                    ProgressHistoryHead
                        .read_current(&f.storage)
                        .unwrap_err()
                        .kind,
                    HistoryErrorKind::InvalidDocument
                );
            }
            2 => {
                assert!(!f.path().exists());
                assert_eq!(f.storage.read_progress().unwrap().audio_history, None);
            }
            _ => assert_eq!(
                f.storage.read_progress().unwrap().settings.topic,
                candidate.settings.topic
            ),
        }
        assert_common_block(&mut f, &other);
    }
    #[test]
    fn n05_valid_third_bytes_block_all_writers_but_allow_pure_read() {
        isolated("store::i2_normal_save_tests::n05_valid_third_bytes_block_all_writers_but_allow_pure_read",||third_state(0));
    }
    #[test]
    fn n05_corrupt_third_bytes_are_not_rolled_back_or_hidden_by_default() {
        isolated("store::i2_normal_save_tests::n05_corrupt_third_bytes_are_not_rolled_back_or_hidden_by_default",||third_state(1));
    }
    #[test]
    fn n05_disappeared_existing_file_is_unknown_not_known_old_absence() {
        isolated("store::i2_normal_save_tests::n05_disappeared_existing_file_is_unknown_not_known_old_absence",||third_state(2));
    }
    #[test]
    fn n05_real_readback_sharing_denial_sets_common_write_block() {
        isolated(
            "store::i2_normal_save_tests::n05_real_readback_sharing_denial_sets_common_write_block",
            || third_state(3),
        );
    }

    #[test]
    fn n06_callback_reentry_is_rejected_without_deadlock_or_nested_write() {
        isolated("store::i2_normal_save_tests::n06_callback_reentry_is_rejected_without_deadlock_or_nested_write",||{
        let f=Fixture::new();let candidate=f.candidate();let calls=Cell::new(0);let nested=RefCell::new(None);let gate=RefCell::new(None);let before_nested=RefCell::new(None);let after_nested=RefCell::new(None);
        let result=f.storage.save_progress_with_publisher(&candidate,LIMIT,&NoProgressFaults,|path,bytes|{
            calls.set(calls.get()+1);*before_nested.borrow_mut()=Some(f.ledger());*nested.borrow_mut()=Some(f.storage.save_progress(&candidate).map(|_|()));*gate.borrow_mut()=Some(acquire_progress_write().map(|guard|drop(guard)));*after_nested.borrow_mut()=Some(f.ledger());atomic_write(path,bytes)
        });result.unwrap();assert_eq!(calls.get(),1);assert!(nested.borrow().as_ref().unwrap().is_err());assert_eq!(*gate.borrow(),Some(Err(ProgressGateError::Reentrant)));assert_eq!(*before_nested.borrow(),*after_nested.borrow());assert_eq!(fs::read(f.path()).unwrap(),f.next(&candidate));writable();
    });
    }
    #[test]
    fn n06_competing_same_and_distinct_roots_never_overlap_publish_intervals() {
        isolated("store::i2_normal_save_tests::n06_competing_same_and_distinct_roots_never_overlap_publish_intervals",||{
        for same_root in [true,false]{
            let first=Fixture::new();let second=Fixture::new();let expected_root=first.a.clone();let a=Arc::new(first.storage);let b=if same_root{a.clone()}else{Arc::new(second.storage)};let stores=[a.clone(),b.clone()];
            let barrier=Arc::new(Barrier::new(3));let active=Arc::new(AtomicUsize::new(0));let overlap=Arc::new(AtomicBool::new(false));let (events_tx,events_rx)=mpsc::channel();let mut releases=vec![];let mut handles=vec![];
            for (i,storage) in stores.into_iter().enumerate(){let (release_tx,release_rx)=mpsc::channel();releases.push(release_tx);let barrier=barrier.clone();let active=active.clone();let overlap=overlap.clone();let tx=events_tx.clone();
                handles.push(std::thread::spawn(move||{let mut candidate:Progress=serde_json::from_str(MIN).unwrap();candidate.settings.topic=format!("worker-{i}");barrier.wait();let result=storage.save_progress_with_publisher(&candidate,LIMIT,&NoProgressFaults,|path,bytes|{
                    if active.fetch_add(1,Ordering::SeqCst)!=0{overlap.store(true,Ordering::SeqCst);}let _=tx.send(i);let release=release_rx.recv_timeout(Duration::from_secs(3));let result=if release.is_ok(){atomic_write(path,bytes)}else{Err("bounded controller release missing".into())};active.fetch_sub(1,Ordering::SeqCst);result
                });(i,result)}));
            }
            drop(events_tx);barrier.wait();let mut order=vec![];for _ in 0..2 {match events_rx.recv_timeout(Duration::from_secs(4)){Ok(i)=>{order.push(i);let _=releases[i].send(());},Err(_)=>break,}}drop(releases);let results:Vec<_>=handles.into_iter().map(|h|h.join().unwrap()).collect();
            assert_eq!(order.len(),2,"both real publish intervals must be observed, not silence");assert!(!overlap.load(Ordering::SeqCst));assert_eq!(active.load(Ordering::SeqCst),0);
            for (i,result) in results{let saved=result.unwrap();assert_eq!(saved.settings.topic,format!("worker-{i}"));assert_eq!(saved.audio_history,Some(expected_root.clone()));}
            if same_root{assert_eq!(a.read_progress().unwrap().settings.topic,format!("worker-{}",order[1]));}else{assert_eq!(a.read_progress().unwrap().settings.topic,"worker-0");assert_eq!(b.read_progress().unwrap().settings.topic,"worker-1");}assert_eq!(a.read_progress().unwrap().audio_history,Some(expected_root.clone()));assert_eq!(b.read_progress().unwrap().audio_history,Some(expected_root));writable();
        }
    });
    }
}

#[cfg(all(test, windows))]
mod i2_pointer_commit_tests {
    use super::*;
    use crate::store::i2_gate_tests::{tree, Temp};
    use crate::{assets, audio_history::*, commit, model};
    use acoustic_analysis::AnalysisProfile;
    use serde_json::{json, Value};
    use std::{
        cell::RefCell,
        os::windows::fs::OpenOptionsExt,
        process::{Command, Stdio},
        sync::{atomic::AtomicUsize, mpsc, Arc, Barrier},
        time::{Duration, Instant},
    };
    const LIMIT: u64 = 100_000;
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
            let mutation_json =
                exact.unwrap_or_else(|| serde_json::to_string(&create(&t)).unwrap());
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

    struct Probe<F> {
        seen: RefCell<Vec<ProgressCheckpoint>>,
        action: F,
    }
    impl<F: Fn(ProgressCheckpoint) -> Result<(), ProgressFaultError>> Probe<F> {
        fn new(action: F) -> Self {
            Self {
                seen: RefCell::new(vec![]),
                action,
            }
        }
    }
    impl<F: Fn(ProgressCheckpoint) -> Result<(), ProgressFaultError>> ProgressFaults for Probe<F> {
        fn check(&self, p: ProgressCheckpoint) -> Result<(), ProgressFaultError> {
            self.seen.borrow_mut().push(p);
            (self.action)(p)
        }
    }
    fn pointer_seen() -> Vec<ProgressCheckpoint> {
        vec![
            ProgressCheckpoint::BeforePointer,
            ProgressCheckpoint::AfterPointer,
        ]
    }
    fn ready(f: &Fixture) {
        let guard = acquire_progress_write().unwrap();
        HistoryWriter::verify_publication_under_guard(&f.storage, &f.plan.publication(), &guard)
            .expect("independent fixture accepted by completed V");
        drop(guard);
        assert_eq!(
            HistoryWriter::confirm_publication(&f.storage, &f.plan.publication())
                .unwrap_err()
                .kind,
            HistoryErrorKind::Busy
        );
    }
    fn writable() {
        let g = acquire_progress_write().expect("known outcome must not block");
        assert_eq!(g.ensure_writable(), Ok(()));
    }
    fn call<P: FnOnce(&Path, &[u8]) -> Result<(), String>>(
        f: &Fixture,
        cancel: Option<&AtomicBool>,
        faults: &dyn ProgressFaults,
        publish: P,
    ) -> Result<HistoryCommitOutcome, HistoryError> {
        f.storage.commit_history_with_publisher(
            &f.plan.publication(),
            cancel,
            LIMIT,
            faults,
            publish,
        )
    }
    fn backup_ledger(f: &Fixture, mut before: Ledger) -> Ledger {
        before.insert(PathBuf::from("host/backups"), vec![]);
        before.insert(
            PathBuf::from(format!("host/backups/progress-v1-{V1_SHA}.json")),
            RAW_V1.as_bytes().to_vec(),
        );
        assert!(
            !f.storage
                .dir
                .join("backups")
                .join(format!(
                    "progress-{}.json",
                    chrono::Local::now().format("%Y-%m-%d")
                ))
                .exists(),
            "pointer has no daily backup"
        );
        before
    }
    fn install_plan(f: &mut Fixture, p: Plan) {
        for b in &p.objects {
            put_at(&f.storage, b);
        }
        fs::write(
            f.history()
                .join(format!("recovery/{}.json", p.operation.as_str())),
            serde_json::to_vec(&p.intent()).unwrap(),
        )
        .unwrap();
        f.plan = p;
        f.healthy(&f.plan.root_ref());
    }
    #[test]
    fn p01_private_limits_entry_preserves_nonhistory_and_creates_only_pointer() {
        for absent in [false, true] {
            let f = Fixture::new(if absent { 0 } else { 2 }, !absent);
            ready(&f);
            let before = f.ledger();
            let probe = Probe::new(|_| Ok(()));
            assert_eq!(
                f.storage
                    .commit_history_with_limit_and_faults(
                        &f.plan.publication(),
                        None,
                        LIMIT,
                        &probe
                    )
                    .unwrap(),
                f.expected(f.plan.root_ref())
            );
            assert_eq!(*probe.seen.borrow(), pointer_seen());
            assert_eq!(f.ledger(), f.committed_ledger(before));
            writable();
        }
    }
    #[test]
    fn p02_invalid_authority_and_missing_files_never_reach_pointer_or_faults() {
        for mode in 0..7 {
            let f = Fixture::new(2, true);
            ready(&f);
            let mut token = f.plan.publication();
            let kind = match mode {
                0 => {
                    let mut root = f.plan.root.clone();
                    root.trials.remove(&tid(1));
                    root.receipts.remove(&op(90));
                    token.next =
                        RootRef::new(put_at(&f.storage, &serde_json::to_vec(&root).unwrap()));
                    f.healthy(&token.next);
                    HistoryErrorKind::Conflict
                }
                1 => {
                    fs::remove_file(f.intent_path()).unwrap();
                    HistoryErrorKind::MissingObject
                }
                2 => {
                    token.request = digest(b"different exact request");
                    HistoryErrorKind::Conflict
                }
                3 => {
                    token.base = None;
                    HistoryErrorKind::Conflict
                }
                4 => {
                    fs::remove_file(f.asset(&f.plan.refs()[1])).unwrap();
                    HistoryErrorKind::MissingObject
                }
                5 => {
                    let mut intent = f.plan.intent();
                    intent["schema_version"] = json!(99);
                    fs::write(f.intent_path(), serde_json::to_vec(&intent).unwrap()).unwrap();
                    HistoryErrorKind::UnsupportedVersion
                }
                _ => {
                    fs::write(
                        f.storage.dir.join("pending-material-commit.json"),
                        b"synthetic pending",
                    )
                    .unwrap();
                    HistoryErrorKind::PendingMaterial
                }
            };
            let before = f.ledger();
            let count = Cell::new(0);
            let probe = Probe::new(|_| Ok(()));
            let err = f
                .storage
                .commit_history_with_publisher(&token, None, LIMIT, &probe, |_, _| {
                    count.set(count.get() + 1);
                    Ok(())
                })
                .unwrap_err();
            assert_eq!(err.kind, kind, "mode {mode}");
            assert_eq!(count.get(), 0);
            assert!(probe.seen.borrow().is_empty());
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn p02_busy_running_or_sending_trash_plan_is_not_commit_authority() {
        for sending in [false, true] {
            let mut f = Fixture::new(2, true);
            let mut t = trial(1);
            let id = RunId::parse("55555555-5555-4555-8555-000000000001").unwrap();
            if sending {
                let run = AdviceRun {
                    schema_version: 1,
                    id: id.clone(),
                    trial: t.id.clone(),
                    audio: t.input.audio.original.sha256.clone(),
                    reference: t.input.reference.sha256.clone(),
                    confirmed_at: at(),
                    confirmation: digest(b"synthetic visible consent"),
                    destination: "https://fixture.invalid".into(),
                    versions: AdviceVersions {
                        provider: "fixture".into(),
                        prompt: "1".into(),
                        parser: "1".into(),
                    },
                    state: AdviceRunState::Sending,
                    send: SendCertainty::MayHaveBeenSent,
                    started_at: at(),
                    ended_at: None,
                    result: None,
                    failure: None,
                };
                t.advice
                    .insert(id, put_at(&f.storage, &serde_json::to_vec(&run).unwrap()));
            } else {
                let run = AnalysisRun {
                    schema_version: 1,
                    id: id.clone(),
                    trial: t.id.clone(),
                    audio: t.input.audio.original.sha256.clone(),
                    pcm: digest(b"synthetic PCM"),
                    decoder: DecoderIdentity {
                        name: "fixture".into(),
                        version: "1".into(),
                        binary_sha256: None,
                    },
                    profile: AnalysisProfile::v1(),
                    state: AnalysisRunState::Running,
                    started_at: at(),
                    ended_at: None,
                    result: None,
                    failure: None,
                };
                t.analysis
                    .insert(id, put_at(&f.storage, &serde_json::to_vec(&run).unwrap()));
            }
            let mut base = empty_root();
            base.revision = 1;
            base.next_save_sequence = 2;
            base.trials.insert(
                t.id.clone(),
                TrialHead {
                    document: put_at(&f.storage, &serde_json::to_vec(&t).unwrap()),
                    admitted_sequence: 1,
                },
            );
            let base_ref = RootRef::new(put_at(&f.storage, &serde_json::to_vec(&base).unwrap()));
            f.healthy(&base_ref);
            let mutation = HistoryMutation::SetTrash {
                target: target(&t),
                trashed_at: Some(at()),
            };
            let mut next = t.clone();
            next.revision = 2;
            next.previous = Some(target(&t).expected);
            next.trashed_at = Some(at());
            let plan = Plan::update(next, &base, base_ref.clone(), op(1), mutation, vec![]);
            install_plan(&mut f, plan);
            f.set_head(&base_ref);
            f.old = Some(fs::read(f.path()).unwrap());
            let before = f.ledger();
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &f.plan.publication())
                    .unwrap_err()
                    .kind,
                HistoryErrorKind::Busy
            );
            let g = acquire_progress_write().unwrap();
            assert_eq!(
                HistoryWriter::verify_publication_under_guard(
                    &f.storage,
                    &f.plan.publication(),
                    &g
                )
                .unwrap_err()
                .kind,
                HistoryErrorKind::Busy
            );
            drop(g);
            let count = Cell::new(0);
            let probe = Probe::new(|_| Ok(()));
            assert_eq!(
                call(&f, None, &probe, |_, _| {
                    count.set(1);
                    Ok(())
                })
                .unwrap_err()
                .kind,
                HistoryErrorKind::Busy
            );
            assert_eq!(count.get(), 0);
            assert!(probe.seen.borrow().is_empty());
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn p03_existing_next_later_and_cleanup_ignore_cancel_without_fault_or_publish() {
        for later in [false, true] {
            for cleanup in [false, true] {
                for cancel in [false, true] {
                    let mut f = Fixture::new(2, true);
                    let current = if later {
                        f.later().root_ref()
                    } else {
                        f.plan.root_ref()
                    };
                    f.set_head(&current);
                    if cleanup {
                        fs::remove_file(f.intent_path()).unwrap();
                    }
                    let control = f.public_control().unwrap();
                    assert_eq!(control.current, current);
                    let before = f.ledger();
                    let flag = AtomicBool::new(cancel);
                    let count = Cell::new(0);
                    let probe = Probe::new(|_| Err(ProgressFaultError::Io));
                    assert_eq!(
                        call(&f, Some(&flag), &probe, |_, _| {
                            count.set(1);
                            Err("must not run".into())
                        })
                        .unwrap(),
                        f.expected(current)
                    );
                    assert_eq!(count.get(), 0);
                    assert!(probe.seen.borrow().is_empty());
                    assert_eq!(f.ledger(), before);
                }
            }
        }
    }
    #[test]
    fn p03_receipt_and_remaining_intent_mismatch_never_take_idempotent_success() {
        for mode in 0..4 {
            let f = Fixture::new(2, true);
            let mut root = f.plan.root.clone();
            let kind = match mode {
                0 => {
                    root.receipts.get_mut(&op(1)).unwrap().request = digest(b"other request");
                    HistoryErrorKind::Conflict
                }
                1 => {
                    root.receipts.get_mut(&op(1)).unwrap().affected = vec![tid(1)];
                    HistoryErrorKind::Conflict
                }
                2 => {
                    root.receipts.get_mut(&op(1)).unwrap().committed_revision = 1;
                    HistoryErrorKind::Conflict
                }
                _ => {
                    let mut i = f.plan.intent();
                    i["schema_version"] = json!(99);
                    fs::write(f.intent_path(), serde_json::to_vec(&i).unwrap()).unwrap();
                    HistoryErrorKind::UnsupportedVersion
                }
            };
            let current = RootRef::new(put_at(&f.storage, &serde_json::to_vec(&root).unwrap()));
            f.healthy(&current);
            f.set_head(&current);
            let before = f.ledger();
            let count = Cell::new(0);
            let probe = Probe::new(|_| Ok(()));
            assert_eq!(
                HistoryWriter::confirm_publication(&f.storage, &f.plan.publication())
                    .unwrap_err()
                    .kind,
                kind
            );
            assert_eq!(
                call(&f, None, &probe, |_, _| {
                    count.set(1);
                    Ok(())
                })
                .unwrap_err()
                .kind,
                kind
            );
            assert_eq!(count.get(), 0);
            assert!(probe.seen.borrow().is_empty());
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn p04_publisher_ok_or_err_is_classified_by_exact_disk_not_message() {
        for write in [false, true] {
            for publish_err in [false, true] {
                for after_err in [false, true] {
                    let f = Fixture::new(2, true);
                    ready(&f);
                    let before = f.ledger();
                    let count = Cell::new(0);
                    let probe = Probe::new(|p| {
                        if after_err && p == ProgressCheckpoint::AfterPointer {
                            Err(ProgressFaultError::Io)
                        } else {
                            Ok(())
                        }
                    });
                    let expected = f.next();
                    let result = call(&f, None, &probe, |path, bytes| {
                        count.set(count.get() + 1);
                        assert_eq!(path, f.path());
                        assert_eq!(bytes, expected);
                        assert_eq!(
                            acquire_progress_write().unwrap_err(),
                            ProgressGateError::Reentrant
                        );
                        if write {
                            atomic_write(path, bytes)?;
                        }
                        if publish_err {
                            Err("identical synthetic ambiguous message".into())
                        } else {
                            Ok(())
                        }
                    });
                    assert_eq!(count.get(), 1);
                    assert_eq!(*probe.seen.borrow(), pointer_seen());
                    if write {
                        assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
                        assert_eq!(f.ledger(), f.committed_ledger(before));
                    } else {
                        assert_eq!(
                            result.unwrap_err().kind,
                            if publish_err || after_err {
                                HistoryErrorKind::Io
                            } else {
                                HistoryErrorKind::Busy
                            }
                        );
                        assert_eq!(f.ledger(), before);
                    }
                    writable();
                }
            }
        }
    }
    #[test]
    fn p04_before_fault_still_rechecks_exact_old_or_new_without_publisher() {
        for write in [false, true] {
            let f = Fixture::new(2, true);
            ready(&f);
            let before = f.ledger();
            let count = Cell::new(0);
            let probe = Probe::new(|p| {
                assert_eq!(p, ProgressCheckpoint::BeforePointer);
                if write {
                    atomic_write(&f.path(), &f.next()).unwrap();
                }
                Err(ProgressFaultError::Io)
            });
            let result = call(&f, None, &probe, |_, _| {
                count.set(1);
                Ok(())
            });
            assert_eq!(count.get(), 0);
            assert_eq!(
                *probe.seen.borrow(),
                vec![ProgressCheckpoint::BeforePointer]
            );
            if write {
                assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
                assert_eq!(f.ledger(), f.committed_ledger(before));
            } else {
                assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
                assert_eq!(f.ledger(), before);
            }
            writable();
        }
    }
    #[test]
    fn p04_before_ok_snapshot_conflict_refuses_without_new_block() {
        let f = Fixture::new(2, true);
        ready(&f);
        let before = f.ledger();
        let mut third = f.snapshot();
        third.settings.topic = "external update before pointer".into();
        let third = serde_json::to_vec_pretty(&third).unwrap();
        let count = Cell::new(0);
        let probe = Probe::new(|point| {
            assert_eq!(point, ProgressCheckpoint::BeforePointer);
            fs::write(f.path(), &third).unwrap();
            Ok(())
        });
        let result = call(&f, None, &probe, |_, _| {
            count.set(1);
            Ok(())
        });
        assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Conflict);
        assert_eq!(count.get(), 0);
        assert_eq!(
            *probe.seen.borrow(),
            vec![ProgressCheckpoint::BeforePointer]
        );
        let mut expected = before;
        expected.insert(PathBuf::from("host/progress.json"), third);
        assert_eq!(f.ledger(), expected);
        writable();
    }
    #[test]
    fn p04_real_atomic_failure_keeps_exact_old_and_unlocked_control_succeeds() {
        let f = Fixture::new(2, true);
        ready(&f);
        let before = f.ledger();
        let count = Cell::new(0);
        let probe = Probe::new(|_| Ok(()));
        let result = call(&f, None, &probe, |path, bytes| {
            count.set(count.get() + 1);
            let lock = OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(path)
                .unwrap();
            assert_eq!(fs::read(path).unwrap_err().raw_os_error(), Some(32));
            let actual = atomic_write(path, bytes);
            assert!(actual.is_err(), "real locked atomic replace must fail");
            eprintln!("P real atomic failure observed: {actual:?}");
            drop(lock);
            assert_eq!(fs::read(path).unwrap(), f.old.as_ref().unwrap().as_slice());
            actual
        });
        assert_eq!(count.get(), 1);
        assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
        assert_eq!(*probe.seen.borrow(), pointer_seen());
        assert_eq!(
            f.ledger(),
            before,
            "atomic temp removed; exact old; no block"
        );
        writable();
        // Fixture-only causal control, after every production outcome/ledger assertion.
        atomic_write(&f.path(), &f.next()).expect("same path/bytes succeed after unlock");
        assert_eq!(f.ledger(), f.committed_ledger(before));
    }
    #[test]
    fn p06_cancel_three_boundaries_preserve_exact_outcome_and_originals() {
        for mode in 0..3 {
            let f = Fixture::new(2, true);
            ready(&f);
            let before = f.ledger();
            let cancel = AtomicBool::new(mode == 0);
            let count = Cell::new(0);
            let probe = Probe::new(|p| {
                if p == ProgressCheckpoint::BeforePointer && mode == 1 {
                    cancel.store(true, Ordering::Release);
                }
                Ok(())
            });
            let result = call(&f, Some(&cancel), &probe, |path, bytes| {
                count.set(count.get() + 1);
                cancel.store(true, Ordering::Release);
                atomic_write(path, bytes)
            });
            if mode < 2 {
                assert_eq!(result.unwrap(), HistoryCommitOutcome::CancelledBeforeCommit);
                assert_eq!(count.get(), 0);
                assert_eq!(
                    *probe.seen.borrow(),
                    vec![ProgressCheckpoint::BeforePointer]
                );
                assert_eq!(f.ledger(), before);
            } else {
                assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
                assert_eq!(count.get(), 1);
                assert_eq!(*probe.seen.borrow(), pointer_seen());
                assert_eq!(f.ledger(), f.committed_ledger(before));
            }
            writable();
        }
    }
    #[test]
    fn p07_wire1_exact_backup_and_fault_prefixes_preserve_original_bytes() {
        for mode in 0..5 {
            let f = Fixture::new(1, false);
            ready(&f);
            let before = f.ledger();
            let cancel = AtomicBool::new(mode == 3);
            let count = Cell::new(0);
            let stop = match mode {
                1 => Some(ProgressCheckpoint::BeforeMigrationBackup),
                2 => Some(ProgressCheckpoint::AfterMigrationBackup),
                4 => Some(ProgressCheckpoint::BeforePointer),
                _ => None,
            };
            let probe = Probe::new(|p| {
                if Some(p) == stop {
                    Err(ProgressFaultError::Io)
                } else {
                    Ok(())
                }
            });
            let result = call(&f, Some(&cancel), &probe, |path, bytes| {
                count.set(count.get() + 1);
                atomic_write(path, bytes)
            });
            let mut seen = vec![ProgressCheckpoint::BeforeMigrationBackup];
            if mode != 1 {
                seen.push(ProgressCheckpoint::AfterMigrationBackup);
            }
            if !matches!(mode, 1 | 2) {
                seen.push(ProgressCheckpoint::BeforePointer);
            }
            if mode == 0 {
                seen.push(ProgressCheckpoint::AfterPointer);
            }
            assert_eq!(*probe.seen.borrow(), seen);
            if mode == 0 {
                assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
                assert_eq!(count.get(), 1);
                assert_eq!(f.ledger(), f.committed_ledger(backup_ledger(&f, before)));
            } else {
                if mode == 3 {
                    assert_eq!(result.unwrap(), HistoryCommitOutcome::CancelledBeforeCommit);
                } else {
                    assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
                }
                assert_eq!(count.get(), 0);
                assert_eq!(fs::read(f.path()).unwrap(), RAW_V1.as_bytes());
                assert_eq!(
                    f.ledger(),
                    if mode == 1 {
                        before
                    } else {
                        backup_ledger(&f, before)
                    }
                );
            }
            writable();
        }
    }
    #[test]
    fn p07_old_absence_exact_size_and_typed_preflight_are_distinct() {
        for extra in [0, 1] {
            let f = Fixture::new(0, false);
            ready(&f);
            let before = f.ledger();
            let limit = f.next().len() as u64 - extra;
            let count = Cell::new(0);
            let probe = Probe::new(|_| Ok(()));
            let result = f.storage.commit_history_with_publisher(
                &f.plan.publication(),
                None,
                limit,
                &probe,
                |p, b| {
                    count.set(1);
                    atomic_write(p, b)
                },
            );
            if extra == 0 {
                assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
                assert_eq!(count.get(), 1);
                assert_eq!(*probe.seen.borrow(), pointer_seen());
                assert_eq!(f.ledger(), f.committed_ledger(before));
            } else {
                assert_eq!(result.unwrap_err().kind, HistoryErrorKind::LimitExceeded);
                assert_eq!(count.get(), 0);
                assert!(probe.seen.borrow().is_empty());
                assert_eq!(f.ledger(), before);
            }
            writable();
        }
        for err in [false, true] {
            let f = Fixture::new(0, false);
            ready(&f);
            let before = f.ledger();
            let probe = Probe::new(|_| Ok(()));
            let count = Cell::new(0);
            let result = call(&f, None, &probe, |_, _| {
                count.set(1);
                if err {
                    Err("same synthetic message".into())
                } else {
                    Ok(())
                }
            });
            assert_eq!(count.get(), 1);
            assert_eq!(
                result.unwrap_err().kind,
                if err {
                    HistoryErrorKind::Io
                } else {
                    HistoryErrorKind::Busy
                }
            );
            assert_eq!(*probe.seen.borrow(), pointer_seen());
            assert_eq!(f.ledger(), before);
            writable();
        }
    }
    #[test]
    fn p07_invalid_disk_and_migration_backup_refuse_before_pointer_without_mutation() {
        for mode in 0..5 {
            let f = Fixture::new(if mode == 4 { 1 } else { 2 }, mode != 4);
            ready(&f);
            let kind = match mode {
                0 => {
                    let mut v: Value = serde_json::from_slice(f.old.as_ref().unwrap()).unwrap();
                    v["version"] = json!(99);
                    fs::write(f.path(), serde_json::to_vec(&v).unwrap()).unwrap();
                    HistoryErrorKind::UnsupportedVersion
                }
                1 => {
                    fs::write(f.path(), b"{").unwrap();
                    HistoryErrorKind::InvalidDocument
                }
                2 => {
                    let mut raw = f.old.clone().unwrap();
                    raw.resize(LIMIT as usize + 1, b' ');
                    fs::write(f.path(), raw).unwrap();
                    HistoryErrorKind::LimitExceeded
                }
                3 => {
                    fs::remove_file(f.path()).unwrap();
                    fs::create_dir(f.path()).unwrap();
                    HistoryErrorKind::UnsafePath
                }
                _ => {
                    fs::create_dir_all(f.storage.dir.join("backups")).unwrap();
                    fs::write(
                        f.storage
                            .dir
                            .join(format!("backups/progress-v1-{V1_SHA}.json")),
                        b"different bytes",
                    )
                    .unwrap();
                    HistoryErrorKind::Conflict
                }
            };
            let before = f.ledger();
            let probe = Probe::new(|_| Ok(()));
            let count = Cell::new(0);
            assert_eq!(
                call(&f, None, &probe, |_, _| {
                    count.set(1);
                    Ok(())
                })
                .unwrap_err()
                .kind,
                kind
            );
            assert_eq!(count.get(), 0);
            assert!(!probe
                .seen
                .borrow()
                .contains(&ProgressCheckpoint::BeforePointer));
            assert_eq!(f.ledger(), before);
            writable();
        }
    }

    fn isolated(case: &str, mode: u8, body: impl FnOnce()) {
        const ENV: &str = "WW5_I2_P_CHILD";
        const MODE: &str = "WW5_I2_P_MODE";
        if std::env::var(ENV).as_deref() == Ok(case) {
            if std::env::var(MODE).unwrap() == mode.to_string() {
                body();
                println!("WW5_P_CHILD_OK:{mode}");
            }
            return;
        }
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
        let start = Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if start.elapsed() > Duration::from_secs(15) {
                child.kill().unwrap();
                let output = child.wait_with_output().unwrap();
                panic!(
                    "child timeout mode={mode}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "child mode={mode}\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(&format!("WW5_P_CHILD_OK:{mode}")),
            "exact child marker"
        );
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        assert!(tree(&temp.root).is_empty(), "child-owned fixture cleanup");
    }
    fn common_block(f: &mut Fixture, other: &Storage) {
        assert_eq!(
            acquire_progress_write().unwrap_err(),
            ProgressGateError::WriteBlocked
        );
        let before = f.ledger();
        let candidate = f.snapshot();
        let token = f.plan.publication();
        assert_eq!(
            f.storage.commit_history(&token, None).unwrap_err().kind,
            HistoryErrorKind::Io
        );
        assert!(f.storage.save_progress(&candidate).is_err());
        assert!(other.save_progress(&candidate).is_err());
        assert!(commit::save_progress(&f.storage.dir, model::BUILTIN_DECK, &candidate).is_err());
        let t = trial(9);
        let request = FrozenHistoryRequest::new(
            create(&t),
            vec![Arc::from(wav(t.id.as_str().as_bytes()[35]))],
        )
        .unwrap();
        let mut writer =
            HistoryWriter::new(&mut f.storage, &ProgressHistoryHead, HistoryQuota::v1()).unwrap();
        assert_eq!(
            writer.prepare(op(99), None, &request).unwrap_err().kind,
            HistoryErrorKind::Io
        );
        drop(writer);
        assert_eq!(
            f.ledger(),
            before,
            "blocked normal/material/history/pointer have no writes"
        );
        assert!(
            HistoryReader::new(&f.storage)
                .read_snapshot(f.plan.base.as_ref())
                .root
                .is_some(),
            "pure immutable view does not acquire G"
        );
    }
    #[test]
    fn p05_unknown_actual_disk_states_block_all_writers_without_rollback() {
        const CASE:&str="store::i2_pointer_commit_tests::p05_unknown_actual_disk_states_block_all_writers_without_rollback";
        for mode in 0..8 {
            isolated(CASE, mode, || {
                let mut f = Fixture::new(2, true);
                ready(&f);
                let other = Storage::at(f.temp.root.join("other")).unwrap();
                let later = f.later();
                let mut third = f.snapshot();
                third.audio_history = Some(if mode == 0 {
                    later.root_ref()
                } else {
                    f.plan.root_ref()
                });
                third.settings.topic = "external third state".into();
                let third = serde_json::to_vec_pretty(&third).unwrap();
                if mode == 4 {
                    let probe = f.temp.root.join("read-probe");
                    fs::write(&probe, b"synthetic read probe").unwrap();
                    let held = OpenOptions::new()
                        .read(true)
                        .share_mode(0)
                        .open(&probe)
                        .unwrap();
                    assert_eq!(fs::read(&probe).unwrap_err().raw_os_error(), Some(32));
                    drop(held);
                    assert_eq!(fs::read(&probe).unwrap(), b"synthetic read probe");
                    fs::remove_file(probe).unwrap();
                }
                let before = f.ledger();
                let count = Cell::new(0);
                let held = RefCell::new(None);
                let mut expected = f.committed_ledger(before.clone());
                match mode {
                    0 | 1 | 7 => {
                        expected.insert(PathBuf::from("host/progress.json"), third.clone());
                    }
                    2 => {
                        expected.remove(&PathBuf::from("host/progress.json"));
                    }
                    3 => {
                        expected.insert(PathBuf::from("host/progress.json"), vec![]);
                    }
                    5 => {
                        expected.remove(&PathBuf::from(format!(
                            "host/audio-history/assets/{}.bin",
                            f.plan.refs()[1].sha256.as_str()
                        )));
                    }
                    6 => {
                        expected.insert(
                            PathBuf::from("host/pending-material-commit.json"),
                            b"appeared pending".to_vec(),
                        );
                    }
                    _ => {}
                }
                let fault = Probe::new(|p| {
                    if mode == 7 && p == ProgressCheckpoint::BeforePointer {
                        fs::write(f.path(), &third).unwrap();
                        Err(ProgressFaultError::Io)
                    } else {
                        Ok(())
                    }
                });
                let result = call(&f, None, &fault, |path, bytes| {
                    count.set(count.get() + 1);
                    assert_eq!(bytes, f.next());
                    atomic_write(path, bytes)?;
                    match mode {
                        0 | 1 => fs::write(path, &third).unwrap(),
                        2 => fs::remove_file(path).unwrap(),
                        3 => {
                            fs::remove_file(path).unwrap();
                            fs::create_dir(path).unwrap();
                        }
                        4 => {
                            *held.borrow_mut() = Some(
                                OpenOptions::new()
                                    .read(true)
                                    .share_mode(0)
                                    .open(path)
                                    .unwrap(),
                            );
                            assert_eq!(fs::read(path).unwrap_err().raw_os_error(), Some(32));
                        }
                        5 => fs::remove_file(f.asset(&f.plan.refs()[1])).unwrap(),
                        6 => fs::write(
                            f.storage.dir.join("pending-material-commit.json"),
                            b"appeared pending",
                        )
                        .unwrap(),
                        _ => unreachable!(),
                    }
                    Err("same ambiguous return for all actual outcomes".into())
                });
                drop(held.borrow_mut().take());
                assert_eq!(
                    count.get(),
                    if mode == 7 { 0 } else { 1 },
                    "real boundary must run"
                );
                assert_eq!(result.unwrap_err().kind, HistoryErrorKind::Io);
                assert_eq!(
                    *fault.seen.borrow(),
                    if mode == 7 {
                        vec![ProgressCheckpoint::BeforePointer]
                    } else {
                        pointer_seen()
                    }
                );
                drop(fault);
                if mode == 4 {
                    assert_eq!(fs::read(f.path()).unwrap(), f.next());
                    eprintln!("P actual read32 and unlocked exact read succeeded");
                }
                assert_eq!(
                    f.ledger(),
                    expected,
                    "fixture modifications only; no rollback/extra backup"
                );
                if mode == 3 {
                    assert!(f.storage.read_progress().is_err());
                } else {
                    assert!(
                        f.storage.read_progress().is_ok(),
                        "pure read is not blocked"
                    );
                }
                common_block(&mut f, &other);
            });
        }
    }
    #[test]
    fn p08_callback_reentry_is_busy_and_all_nested_writers_preserve_bytes() {
        const CASE:&str="store::i2_pointer_commit_tests::p08_callback_reentry_is_busy_and_all_nested_writers_preserve_bytes";
        isolated(CASE, 0, || {
            let f = Fixture::new(2, true);
            ready(&f);
            let count = Cell::new(0);
            let probe = Probe::new(|_| Ok(()));
            let before = f.ledger();
            let result = call(&f, None, &probe, |path, bytes| {
                count.set(1);
                let inside = f.ledger();
                assert_eq!(
                    acquire_progress_write().unwrap_err(),
                    ProgressGateError::Reentrant
                );
                assert_eq!(
                    f.storage
                        .commit_history(&f.plan.publication(), None)
                        .unwrap_err()
                        .kind,
                    HistoryErrorKind::Busy
                );
                assert!(f.storage.save_progress(&f.snapshot()).is_err());
                assert!(
                    commit::save_progress(&f.storage.dir, model::BUILTIN_DECK, &f.snapshot())
                        .is_err()
                );
                assert_eq!(f.ledger(), inside);
                atomic_write(path, bytes)
            });
            assert_eq!(count.get(), 1);
            assert_eq!(result.unwrap(), f.expected(f.plan.root_ref()));
            assert_eq!(*probe.seen.borrow(), pointer_seen());
            assert_eq!(f.ledger(), f.committed_ledger(before));
            writable();
        });
    }
    #[test]
    fn p08_waiting_pointer_uses_nonhistory_updated_under_normal_save_guard() {
        const CASE:&str="store::i2_pointer_commit_tests::p08_waiting_pointer_uses_nonhistory_updated_under_normal_save_guard";
        isolated(CASE, 0, || {
            let f = Fixture::new(2, true);
            ready(&f);
            let original = f.snapshot();
            let expected_root = f.plan.root_ref();
            let token = f.plan.publication();
            let mut fresh = original.clone();
            fresh.settings.topic = "latest queued normal save".into();
            fresh
                .japanese_drafts
                .insert("late".into(), "must survive pointer".into());
            let mut expected = fresh.clone();
            expected.audio_history = Some(expected_root.clone());
            let expected = serde_json::to_vec_pretty(&expected).unwrap();
            let storage = Arc::new(f.storage);
            let n = storage.clone();
            let p = storage.clone();
            let (entered_tx, entered_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let (started_tx, started_rx) = mpsc::channel();
            let normal = std::thread::spawn(move || {
                n.save_progress_with_publisher(&fresh, LIMIT, &NoProgressFaults, |path, bytes| {
                    assert_eq!(
                        acquire_progress_write().unwrap_err(),
                        ProgressGateError::Reentrant
                    );
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                    atomic_write(path, bytes)
                })
            });
            if let Err(error) = entered_rx.recv_timeout(Duration::from_secs(4)) {
                drop(release_tx);
                let normal_result = normal.join();
                panic!("normal save did not enter publisher: {error:?}; result={normal_result:?}");
            }
            let pointer = std::thread::spawn(move || {
                started_tx.send(()).unwrap();
                p.commit_history(&token, None)
            });
            started_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            release_tx.send(()).unwrap();
            normal.join().unwrap().unwrap();
            match pointer.join().unwrap().unwrap() {
                HistoryCommitOutcome::Committed(c) => assert_eq!(c.current, expected_root),
                _ => panic!("not cancelled"),
            }
            assert_eq!(
                fs::read(storage.dir.join("progress.json")).unwrap(),
                expected
            );
            assert_eq!(
                storage.read_progress().unwrap().settings.topic,
                "latest queued normal save"
            );
            drop(storage);
            writable();
        });
    }
    #[test]
    fn p08_pointer_normal_intervals_are_exclusive_across_same_and_distinct_roots() {
        const CASE:&str="store::i2_pointer_commit_tests::p08_pointer_normal_intervals_are_exclusive_across_same_and_distinct_roots";
        for alias in [false, true] {
            isolated(CASE, u8::from(alias), || {
                let f = Fixture::new(2, true);
                ready(&f);
                let other = Fixture::new(2, true);
                ready(&other);
                let token = f.plan.publication();
                let expected_root = token.next.clone();
                let mut candidate = f.snapshot();
                candidate.settings.topic = "normal interval".into();
                let pstorage = Arc::new(f.storage);
                let nstorage = if alias {
                    pstorage.clone()
                } else {
                    Arc::new(other.storage)
                };
                let active = Arc::new(AtomicUsize::new(0));
                let overlap = Arc::new(AtomicBool::new(false));
                let barrier = Arc::new(Barrier::new(3));
                let (events_tx, events_rx) = mpsc::channel();
                let (pr, px) = mpsc::channel();
                let (nr, nx) = mpsc::channel();
                let pa = active.clone();
                let po = overlap.clone();
                let pb = barrier.clone();
                let pt = events_tx.clone();
                let pointer = std::thread::spawn(move || {
                    pb.wait();
                    let result = pstorage.commit_history_with_publisher(
                        &token,
                        None,
                        LIMIT,
                        &NoProgressFaults,
                        |path, bytes| {
                            if pa.fetch_add(1, Ordering::SeqCst) != 0 {
                                po.store(true, Ordering::SeqCst);
                            }
                            assert_eq!(
                                acquire_progress_write().unwrap_err(),
                                ProgressGateError::Reentrant
                            );
                            pt.send(0).unwrap();
                            px.recv_timeout(Duration::from_secs(3)).unwrap();
                            let r = atomic_write(path, bytes);
                            pa.fetch_sub(1, Ordering::SeqCst);
                            r
                        },
                    );
                    (pstorage, result)
                });
                let na = active.clone();
                let no = overlap.clone();
                let nb = barrier.clone();
                let nt = events_tx.clone();
                let normal = std::thread::spawn(move || {
                    nb.wait();
                    let result = nstorage.save_progress_with_publisher(
                        &candidate,
                        LIMIT,
                        &NoProgressFaults,
                        |path, bytes| {
                            if na.fetch_add(1, Ordering::SeqCst) != 0 {
                                no.store(true, Ordering::SeqCst);
                            }
                            assert_eq!(
                                acquire_progress_write().unwrap_err(),
                                ProgressGateError::Reentrant
                            );
                            nt.send(1).unwrap();
                            nx.recv_timeout(Duration::from_secs(3)).unwrap();
                            let r = atomic_write(path, bytes);
                            na.fetch_sub(1, Ordering::SeqCst);
                            r
                        },
                    );
                    (nstorage, result)
                });
                drop(events_tx);
                barrier.wait();
                let mut seen = vec![];
                for _ in 0..2 {
                    let Ok(id) = events_rx.recv_timeout(Duration::from_secs(4)) else {
                        break;
                    };
                    seen.push(id);
                    if id == 0 {
                        let _ = pr.send(());
                    } else {
                        let _ = nr.send(());
                    }
                }
                drop(pr);
                drop(nr);
                let (ps, pr) = pointer.join().unwrap();
                let (ns, nr) = normal.join().unwrap();
                pr.unwrap();
                nr.unwrap();
                assert_eq!(seen.len(), 2);
                assert!(!overlap.load(Ordering::SeqCst));
                assert_eq!(active.load(Ordering::SeqCst), 0);
                assert_eq!(
                    ps.read_progress().unwrap().audio_history,
                    Some(expected_root.clone())
                );
                assert_eq!(
                    ns.read_progress().unwrap().settings.topic,
                    "normal interval"
                );
                if alias {
                    assert_eq!(
                        ns.read_progress().unwrap().audio_history,
                        Some(expected_root)
                    );
                } else {
                    assert_ne!(
                        ns.read_progress().unwrap().audio_history,
                        Some(expected_root)
                    );
                    assert_eq!(ps.read_progress().unwrap().settings.topic, "合成・日本語");
                }
                drop(ps);
                drop(ns);
                writable();
            });
        }
    }

    #[test]
    fn p08_public_material_alias_and_history_share_pointer_guard_under_competition() {
        const CASE:&str="store::i2_pointer_commit_tests::p08_public_material_alias_and_history_share_pointer_guard_under_competition";
        isolated(CASE, 0, || {
            struct HistoryInterval {
                active: Arc<AtomicUsize>,
                overlap: Arc<AtomicBool>,
                events: mpsc::Sender<u8>,
                release: mpsc::Receiver<()>,
            }
            impl HistoryFaults for HistoryInterval {
                fn check(&self, p: HistoryCheckpoint) -> Result<(), HistoryError> {
                    if p == HistoryCheckpoint::BeforeIntent {
                        if self.active.fetch_add(1, Ordering::SeqCst) != 0 {
                            self.overlap.store(true, Ordering::SeqCst);
                        }
                        assert_eq!(
                            acquire_progress_write().unwrap_err(),
                            ProgressGateError::Reentrant
                        );
                        self.events.send(1).unwrap();
                        self.release.recv_timeout(Duration::from_secs(3)).unwrap();
                    }
                    if p == HistoryCheckpoint::AfterIntent {
                        self.active.fetch_sub(1, Ordering::SeqCst);
                    }
                    Ok(())
                }
            }
            let f = Fixture::new(2, true);
            ready(&f);
            let h = Fixture::new(2, true);
            ready(&h);
            let mut candidate = f.snapshot();
            candidate.settings.topic = "material-latest".into();
            let token = f.plan.publication();
            let expected = token.next.clone();
            let storage = Arc::new(f.storage);
            let pstore = storage.clone();
            let material_path = storage.dir.join(".");
            let active = Arc::new(AtomicUsize::new(0));
            let overlap = Arc::new(AtomicBool::new(false));
            let barrier = Arc::new(Barrier::new(4));
            let (events_tx, events_rx) = mpsc::channel();
            let (prelease, prx) = mpsc::channel();
            let (hrelease, hrx) = mpsc::channel();
            let pa = active.clone();
            let po = overlap.clone();
            let pb = barrier.clone();
            let pt = events_tx.clone();
            let pointer = std::thread::spawn(move || {
                pb.wait();
                pstore.commit_history_with_publisher(
                    &token,
                    None,
                    LIMIT,
                    &NoProgressFaults,
                    |path, bytes| {
                        if pa.fetch_add(1, Ordering::SeqCst) != 0 {
                            po.store(true, Ordering::SeqCst);
                        }
                        assert_eq!(
                            acquire_progress_write().unwrap_err(),
                            ProgressGateError::Reentrant
                        );
                        pt.send(0).unwrap();
                        prx.recv_timeout(Duration::from_secs(3)).unwrap();
                        let r = atomic_write(path, bytes);
                        pa.fetch_sub(1, Ordering::SeqCst);
                        r
                    },
                )
            });
            let mb = barrier.clone();
            let material = std::thread::spawn(move || {
                mb.wait();
                commit::save_progress(&material_path, model::BUILTIN_DECK, &candidate)
            });
            let hb = barrier.clone();
            let hs = HistoryInterval {
                active: active.clone(),
                overlap: overlap.clone(),
                events: events_tx.clone(),
                release: hrx,
            };
            let history = std::thread::spawn(move || {
                let mut storage = h.storage;
                let t = trial(7);
                let request = FrozenHistoryRequest::new(
                    create(&t),
                    vec![Arc::from(wav(t.id.as_str().as_bytes()[35]))],
                )
                .unwrap();
                let mut writer = HistoryWriter::with_faults(
                    &mut storage,
                    &ProgressHistoryHead,
                    HistoryQuota::v1(),
                    &hs,
                )
                .unwrap();
                hb.wait();
                writer.prepare(op(7), h.plan.base.as_ref(), &request)
            });
            drop(events_tx);
            barrier.wait();
            let mut seen = vec![];
            for _ in 0..2 {
                let Ok(id) = events_rx.recv_timeout(Duration::from_secs(4)) else {
                    break;
                };
                seen.push(id);
                if id == 0 {
                    let _ = prelease.send(());
                } else {
                    let _ = hrelease.send(());
                }
            }
            drop(prelease);
            drop(hrelease);
            let pointer_result = pointer.join().unwrap();
            let material_result = material.join().unwrap();
            let history_result = history.join().unwrap();
            pointer_result.unwrap();
            material_result.unwrap();
            history_result.unwrap();
            assert_eq!(seen.len(), 2);
            assert!(!overlap.load(Ordering::SeqCst));
            assert_eq!(active.load(Ordering::SeqCst), 0);
            let disk = storage.read_progress().unwrap();
            assert_eq!(disk.audio_history, Some(expected));
            assert_eq!(disk.settings.topic, "material-latest");
            assert!(!commit::pending(&storage.dir));
            drop(storage);
            writable();
        });
    }
}
