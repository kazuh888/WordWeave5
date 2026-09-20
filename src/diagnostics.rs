//! Bounded diagnostics. Free-form traces stay in memory; only typed events reach disk.
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use std::{fs, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPoint { Application, Codex, Settings, Chat, Recording, Playback, Storage, Backup }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage { Start, Startup, Load, Save, Connect, Initialize, Authenticate, ModelList, ThreadStart, TurnStart, Generate, Recover, Record, Synthesize, Play, Edit, Export }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event { Started, Completed, Failed, Cancelled, Paused, Resumed, Stopped, Seeked, RateChanged, Deleted, Restored, Attached, Detached, Configured }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass { Io, Authentication, Timeout, Cancelled, InvalidData, Unsupported, Permission, Unavailable, Other }

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    at: DateTime<Utc>,
    version: String,
    run_id: String,
    entry_point: EntryPoint,
    stage: Stage,
    event: Event,
    error: Option<ErrorClass>,
    elapsed_ms: u64,
}
impl Record {
    fn validate(&self) -> Result<(), LogError> {
        // Export parses and re-serializes the strict schema, never copies raw lines.
        let numeric_parts = |value: &str, separator: char| {
            let parts: Vec<_> = value.split(separator).collect();
            parts.len() == 3 && parts.iter().all(|part| !part.is_empty() && part.len() <= 20 && part.bytes().all(|b| b.is_ascii_digit()))
        };
        if numeric_parts(&self.version, '.') && numeric_parts(&self.run_id, '-') { Ok(()) }
        else { Err(LogError::InvalidData) }
    }
}

/// One operation owns its correlation ID and elapsed clock; no caller-controlled text.
#[derive(Clone)]
pub struct Operation { id: String, entry: EntryPoint, started: Instant }
static SEQUENCE: AtomicU64 = AtomicU64::new(1);
impl Operation {
    pub fn begin(entry: EntryPoint) -> Self {
        let operation = Self { id: format!("{}-{}-{}", Utc::now().timestamp_millis().max(0), std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)), entry, started: Instant::now() };
        operation.event(Stage::Start, Event::Started);
        operation
    }
    pub fn event(&self, stage: Stage, event: Event) { self.emit(stage, event, None); }
    pub fn fail(&self, stage: Stage, error: ErrorClass) { self.emit(stage, Event::Failed, Some(error)); }
    fn record(&self, stage: Stage, event: Event, error: Option<ErrorClass>) -> Record {
        Record { at: Utc::now(), version: env!("CARGO_PKG_VERSION").into(), run_id: self.id.clone(), entry_point: self.entry, stage, event, error,
            elapsed_ms: self.started.elapsed().as_millis().min(u64::MAX as u128) as u64 }
    }
    fn emit(&self, stage: Stage, event: Event, error: Option<ErrorClass>) {
        let record = self.record(stage, event, error);
        let mut sink = sink().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(store) = &mut sink.store {
            match store.append(&record) {
                Ok(()) => { sink.error = None; sink.has_records = true; }
                Err(error) => sink.error = Some(error),
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LogError { Uninitialized, Io, UnsafePath, InvalidData, Limit }
impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Uninitialized => "永続診断ログは未初期化である。",
            Self::Io => "診断ログの読み書きに失敗した。本処理の結果とは別である。",
            Self::UnsafePath => "診断ログの保存先が通常のフォルダー／ファイルではないため拒否した。",
            Self::InvalidData => "診断ログに未対応・不正な内容があるため出力を中止した。原文は出力しない。",
            Self::Limit => "診断ログの安全な容量上限を超えている。",
        })
    }
}
#[derive(Default)]
struct Sink { store: Option<LogStore>, error: Option<LogError>, has_records: bool }
static SINK: OnceLock<Mutex<Sink>> = OnceLock::new();
fn sink() -> &'static Mutex<Sink> { SINK.get_or_init(|| Mutex::new(Sink::default())) }

/// Call only after acquiring the application's storage lock, never from test constructors.
/// Failure is surfaced through status(), not propagated into application operations.
pub fn initialize(log_dir: &Path) {
    let mut state = sink().lock().unwrap_or_else(|p| p.into_inner());
    match LogStore::open(log_dir, Limits::default(), Utc::now()) {
        Ok(store) => {
            let has_records = store.files().is_ok_and(|files| !files.is_empty());
            *state = Sink { store: Some(store), error: None, has_records };
        }
        Err(error) => *state = Sink { store: None, error: Some(error), has_records: false },
    }
}
pub fn status() -> String {
    let state = sink().lock().unwrap_or_else(|p| p.into_inner());
    if let Some(error) = state.error { return error.to_string(); }
    if state.store.is_none() { return LogError::Uninitialized.to_string(); }
    if !state.has_records { return "永続診断ログは有効。記録はまだない。".into(); }
    "永続診断ログは有効（最大5ファイル・各1MiB・30日）。本文・媒体・認証情報・ユーザーパスは含まない。".into()
}
/// Returns sanitized JSONL from earlier launches too; an empty String means no records.
pub fn export() -> Result<String, String> {
    let mut state = sink().lock().unwrap_or_else(|p| p.into_inner());
    let result = state.store.as_ref().ok_or(LogError::Uninitialized).and_then(|store| store.export(Utc::now()));
    if let Err(error) = result { state.error = Some(error); }
    result.map_err(|e| e.to_string())
}

#[derive(Clone, Copy)]
struct Limits { max_file_bytes: u64, max_files: usize }
impl Default for Limits { fn default() -> Self { Self { max_file_bytes: 1024 * 1024, max_files: 5 } } }
struct LogStore { root: PathBuf, limits: Limits }
struct LogFile { path: PathBuf, date: NaiveDate, sequence: u32, len: u64 }
const PREFIX: &str = "wordweave-diagnostics-";
fn no_link(meta: &fs::Metadata) -> Result<(), LogError> {
    #[cfg(windows)]
    { use std::os::windows::fs::MetadataExt; if meta.file_attributes() & 0x400 != 0 { return Err(LogError::UnsafePath); } }
    if meta.file_type().is_symlink() { Err(LogError::UnsafePath) } else { Ok(()) }
}
fn safe_directory(path: &Path, may_create: bool) -> Result<(), LogError> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, std::path::Component::ParentDir)) { return Err(LogError::UnsafePath); }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => { no_link(&meta)?; if !meta.is_dir() { return Err(LogError::UnsafePath); } }
            Err(e) if may_create && ancestor == path && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(LogError::Io),
        }
    }
    Ok(())
}
fn owned_name(name: &str) -> Option<(NaiveDate, u32)> {
    let body = name.strip_prefix(PREFIX)?.strip_suffix(".jsonl")?;
    let (day, sequence) = body.split_once('-')?;
    if day.len() != 8 || sequence.len() != 10 || !day.bytes().chain(sequence.bytes()).all(|b| b.is_ascii_digit()) { return None; }
    Some((NaiveDate::parse_from_str(day, "%Y%m%d").ok()?, sequence.parse().ok()?))
}
fn safe_file(path: &Path) -> Result<fs::Metadata, LogError> {
    let meta = fs::symlink_metadata(path).map_err(|_| LogError::Io)?;
    no_link(&meta)?;
    if !meta.is_file() { return Err(LogError::UnsafePath); }
    Ok(meta)
}
fn open_log_read(path: &Path) -> Result<fs::File, LogError> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    { use std::os::windows::fs::OpenOptionsExt; options.custom_flags(0x00200000); }
    let file = options.open(path).map_err(|_| LogError::Io)?;
    let meta = file.metadata().map_err(|_| LogError::Io)?;
    no_link(&meta)?;
    if !meta.is_file() { return Err(LogError::UnsafePath); }
    Ok(file)
}
fn ends_with_newline(path: &Path) -> Result<bool, LogError> {
    let mut file = open_log_read(path)?;
    if file.metadata().map_err(|_| LogError::Io)?.len() == 0 { return Ok(true); }
    file.seek(SeekFrom::End(-1)).map_err(|_| LogError::Io)?;
    let mut last = [0];
    file.read_exact(&mut last).map_err(|_| LogError::Io)?;
    Ok(last[0] == b'\n')
}
impl LogStore {
    fn open(root: &Path, limits: Limits, now: DateTime<Utc>) -> Result<Self, LogError> {
        safe_directory(root, true)?;
        match fs::create_dir(root) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(_) => return Err(LogError::Io),
        }
        safe_directory(root, false)?;
        let store = Self { root: root.to_owned(), limits };
        store.prune(now)?;
        Ok(store)
    }
    fn files(&self) -> Result<Vec<LogFile>, LogError> {
        safe_directory(&self.root, false)?;
        let mut files = Vec::new();
        for (index, entry) in fs::read_dir(&self.root).map_err(|_| LogError::Io)?.enumerate() {
            if index >= 10_000 { return Err(LogError::Limit); }
            let entry = entry.map_err(|_| LogError::Io)?;
            let Some((date, sequence)) = entry.file_name().to_str().and_then(owned_name) else { continue; };
            let path = entry.path();
            let meta = safe_file(&path)?;
            if meta.len() > self.limits.max_file_bytes { return Err(LogError::Limit); }
            files.push(LogFile { path, date, sequence, len: meta.len() });
        }
        files.sort_by_key(|file| (file.date, file.sequence));
        Ok(files)
    }
    fn prune(&self, now: DateTime<Utc>) -> Result<(), LogError> {
        let cutoff = (now - chrono::Duration::days(29)).date_naive();
        let files = self.files()?;
        let remaining = files.iter().filter(|file| file.date >= cutoff).count();
        let mut excess = remaining.saturating_sub(self.limits.max_files);
        for file in files {
            if file.date < cutoff || excess > 0 {
                if file.date >= cutoff { excess -= 1; }
                self.remove(&file.path)?;
            }
        }
        Ok(())
    }
    fn remove(&self, path: &Path) -> Result<(), LogError> {
        // Only a single validated direct child with our exact generated name is removed.
        safe_directory(&self.root, false)?;
        if path.parent() != Some(self.root.as_path()) || path.file_name().and_then(|s| s.to_str()).and_then(owned_name).is_none() { return Err(LogError::UnsafePath); }
        safe_file(path)?;
        fs::remove_file(path).map_err(|_| LogError::Io)
    }
    fn append(&mut self, record: &Record) -> Result<(), LogError> {
        record.validate()?;
        let mut bytes = serde_json::to_vec(record).map_err(|_| LogError::InvalidData)?;
        bytes.push(b'\n');
        if bytes.len() as u64 > self.limits.max_file_bytes { return Err(LogError::Limit); }
        self.prune(record.at)?;
        let files = self.files()?;
        let day = record.at.date_naive();
        let mut reusable = files.last().filter(|file| file.date == day && file.len + bytes.len() as u64 <= self.limits.max_file_bytes);
        if let Some(previous) = reusable {
            // A process may have exited during its final write. Keep that file intact
            // and start a new one, rather than appending JSON to the truncated line.
            if !ends_with_newline(&previous.path)? { reusable = None; }
        }
        let mut options = fs::OpenOptions::new();
        options.write(true);
        #[cfg(windows)]
        { use std::os::windows::fs::OpenOptionsExt; options.custom_flags(0x00200000); } // OPEN_REPARSE_POINT
        let mut file = if let Some(previous) = reusable {
            safe_file(&previous.path)?;
            options.append(true).open(&previous.path).map_err(|_| LogError::Io)?
        } else {
            let sequence = files.iter().filter(|file| file.date == day).map(|file| file.sequence).max().unwrap_or(0).checked_add(1).ok_or(LogError::Limit)?;
            if files.len() >= self.limits.max_files { self.remove(&files[0].path)?; }
            let path = self.root.join(format!("{PREFIX}{}-{sequence:010}.jsonl", day.format("%Y%m%d")));
            options.create_new(true).open(path).map_err(|_| LogError::Io)?
        };
        let meta = file.metadata().map_err(|_| LogError::Io)?;
        no_link(&meta)?;
        if !meta.is_file() || meta.len() + bytes.len() as u64 > self.limits.max_file_bytes { return Err(LogError::Limit); }
        file.write_all(&bytes).and_then(|_| file.flush()).map_err(|_| LogError::Io)
    }
    fn export(&self, now: DateTime<Utc>) -> Result<String, LogError> {
        let files = self.files()?;
        if files.len() > self.limits.max_files { return Err(LogError::Limit); }
        let cutoff = now - chrono::Duration::days(30);
        let mut output = String::new();
        for file in files {
            let mut bytes = Vec::new();
            let handle = open_log_read(&file.path)?;
            handle.take(self.limits.max_file_bytes + 1).read_to_end(&mut bytes).map_err(|_| LogError::Io)?;
            if bytes.len() as u64 > self.limits.max_file_bytes { return Err(LogError::Limit); }
            for line in bytes.split_inclusive(|byte| *byte == b'\n') {
                let record: Record = match serde_json::from_slice(line) {
                    Ok(record) => record,
                    // Only an interrupted final line is omitted. Complete invalid
                    // records and malformed interior lines still reject the export.
                    Err(error) if !line.ends_with(b"\n") && error.is_eof() => break,
                    Err(_) => return Err(LogError::InvalidData),
                };
                record.validate()?;
                if record.at >= cutoff {
                    output.push_str(&serde_json::to_string(&record).map_err(|_| LogError::InvalidData)?);
                    output.push('\n');
                }
            }
        }
        Ok(output)
    }
}

#[derive(Clone)]
pub struct Trace(Arc<Mutex<VecDeque<String>>>);
static LAST: OnceLock<Mutex<Option<Trace>>> = OnceLock::new();

impl Trace {
    pub fn begin() -> Self {
        let trace = Self(Arc::new(Mutex::new(VecDeque::new())));
        *LAST.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(trace.clone());
        trace.note(&format!("WordWeave {} / OS {} / {}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH));
        trace
    }
    // Callers must only pass app-owned labels, counts or OS error codes.
    pub fn note(&self, text: &str) {
        let mut lines = self.0.lock().unwrap();
        if lines.len() == 128 { lines.pop_front(); }
        lines.push_back(format!("{} {}", chrono::Utc::now().format("%H:%M:%S%.3fZ"), text.chars().take(300).collect::<String>()));
    }
    pub fn stderr(&self, bytes: &[u8]) {
        self.note(&format!("stderr ({} bytes): {}", bytes.len(), classify(bytes)));
    }
    pub fn snapshot(&self) -> String {
        self.0.lock().unwrap().iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

// Conservative allow-list classification instead of trying to redact every
// possible secret. Unrecognized text is discarded, not copied into a report.
fn classify(bytes: &[u8]) -> &'static str {
    let text = String::from_utf8_lossy(bytes).to_lowercase();
    if text.contains("not logged in") { "未ログイン" }
    else if text.contains("invalid configuration") { "設定ファイルが不正" }
    else if text.contains("not recognized") || text.contains("認識されていません") { "コマンドが見つからない" }
    else if text.contains("cannot find") || text.contains("not found") || text.contains("見つかりません") { "ファイル/パス/対象が見つからない" }
    else if text.contains("access is denied") || text.contains("permission denied") { "アクセス拒否" }
    else if text.contains("syntax") || text.contains("構文") || text.contains("unexpected at this time") || text.contains("このとき予期しない") { "コマンド構文エラー" }
    else if text.contains("unc") { "UNCパス/作業ディレクトリ関連" }
    else if text.contains("certificate") || text.contains("tls") { "証明書/TLS関連" }
    else if text.contains("unauthorized") || text.contains("401") { "認証拒否" }
    else if text.contains("forbidden") || text.contains("403") { "アクセス禁止" }
    else if text.contains("deserialize") || text.contains("invalid json") { "JSON通信の解析失敗" }
    else if text.contains("volta") { "Volta関連メッセージ（本文非保存）" }
    else if text.contains("panic") { "子プロセスのpanic" }
    else { "未分類（機密情報保護のため本文非保存）" }
}

pub fn report() -> String {
    let last = LAST.get_or_init(|| Mutex::new(None)).lock().unwrap().clone();
    let body = last.map(|t| {
        let lines = t.0.lock().unwrap();
        lines.iter().cloned().collect::<Vec<_>>().join("\n")
    })
        .unwrap_or_else(|| "診断情報なし。「接続・ChatGPT認証を確認」を実行してください。".into());
    format!("WordWeaveのCodex接続障害を分析してください。事実と仮説を分け、次の安全な確認手順を提案してください。\n\n注意：これは本文を含まない診断要約です。stderrは分類のみで、原文・認証情報・教材・ユーザーパスは保存していません。未分類の原因はこの情報だけでは特定できません。\n\n{body}\n\n追加情報（必要なら手入力）：codex --version / codex login status の結果。認証URL、ワンタイムコード、auth.jsonは貼らないでください。")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stderr_never_returns_secrets() {
        for input in ["access_token=SECRET", "https://auth.openai.com/?code=SECRET", "volta: SECRET", "401 Bearer SECRET", "日本語SECRET", "C:\\Users\\SECRET", "not logged in SECRET"] {
            assert!(!classify(input.as_bytes()).contains("SECRET"));
        }
    }
    #[test]
    fn ring_is_bounded() {
        let t = Trace(Arc::new(Mutex::new(VecDeque::new())));
        for _ in 0..1000 { t.note("event"); }
        assert_eq!(t.0.lock().unwrap().len(), 128);
    }
    #[test]
    fn captured_stderr_has_categories_but_no_raw_text() {
        let t = Trace(Arc::new(Mutex::new(VecDeque::new())));
        t.stderr(b"volta: not found SECRET");
        let lines = t.0.lock().unwrap();
        assert!(lines[0].contains("見つからない"));
        assert!(!lines[0].contains("SECRET"));
    }

    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("ww-diagnostics-test-{}-{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }
    fn record(at: chrono::DateTime<chrono::Utc>, event: Event) -> Record {
        Record { at, version: env!("CARGO_PKG_VERSION").into(), run_id: "100-200-300".into(),
            entry_point: EntryPoint::Recording, stage: Stage::Record, event, error: None, elapsed_ms: 42 }
    }
    #[test]
    fn persistent_events_survive_restart_with_correlation_and_timing() {
        let temp = Temp::new();
        let at = chrono::Utc::now();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        log.append(&record(at, Event::Started)).unwrap();
        log.append(&record(at, Event::Completed)).unwrap();
        drop(log);
        let log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        let exported = log.export(at).unwrap();
        let records: Vec<Record> = exported.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].run_id, records[1].run_id);
        assert_eq!(records[0].entry_point, EntryPoint::Recording);
        assert_eq!(records[1].event, Event::Completed);
        assert_eq!(records[1].elapsed_ms, 42);
        assert_eq!(records[0].at, at);
    }
    #[test]
    fn rotation_is_bounded_and_does_not_touch_unowned_files() {
        let temp = Temp::new();
        let at = chrono::Utc::now();
        let limits = Limits { max_file_bytes: 600, max_files: 3 };
        let unrelated = temp.0.join("user-notes.jsonl");
        std::fs::write(&unrelated, "KEEP").unwrap();
        let mut log = LogStore::open(&temp.0, limits, at).unwrap();
        for _ in 0..20 { log.append(&record(at, Event::Started)).unwrap(); }
        let files = log.files().unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().all(|file| std::fs::metadata(&file.path).unwrap().len() <= 600));
        assert_eq!(std::fs::read_to_string(unrelated).unwrap(), "KEEP");
        assert!(log.export(at).unwrap().lines().count() < 20);
    }
    #[test]
    fn retention_removes_only_old_owned_files() {
        let temp = Temp::new();
        let at = chrono::Utc::now();
        let old = at - chrono::Duration::days(31);
        let mut log = LogStore::open(&temp.0, Limits::default(), old).unwrap();
        log.append(&record(old, Event::Started)).unwrap();
        let old_path = log.files().unwrap()[0].path.clone();
        let unrelated = temp.0.join("wordweave-diagnostics-not-a-date-0000000000.jsonl");
        std::fs::write(&unrelated, "KEEP").unwrap();
        let mut reopened = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        reopened.append(&record(at, Event::Completed)).unwrap();
        assert!(!old_path.exists());
        assert!(unrelated.exists());
        assert_eq!(reopened.export(at).unwrap().lines().count(), 1);
    }
    #[test]
    fn export_rejects_injected_text_and_never_returns_raw_file_contents() {
        let temp = Temp::new();
        let at = chrono::Utc::now();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        log.append(&record(at, Event::Started)).unwrap();
        let path = log.files().unwrap()[0].path.clone();
        for field in ["version", "run_id", "entry_point", "event", "error"] {
            let mut value = serde_json::to_value(record(at, Event::Started)).unwrap();
            value[field] = serde_json::json!("SECRET C:\\Users\\SECRET access_token=SECRET");
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            let error = log.export(at).unwrap_err().to_string();
            assert!(!error.contains("SECRET"));
        }
    }
    #[test]
    fn empty_log_and_io_failures_are_explicit_and_do_not_panic() {
        let temp = Temp::new();
        let at = chrono::Utc::now();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        assert!(log.export(at).unwrap().is_empty());
        std::fs::remove_dir(&temp.0).unwrap();
        std::fs::write(&temp.0, "not a directory").unwrap();
        assert!(log.append(&record(at, Event::Started)).is_err());
        assert!(log.export(at).is_err());
        std::fs::remove_file(&temp.0).unwrap();
    }

    #[test]
    fn interrupted_last_write_does_not_hide_completed_or_future_events() {
        let temp = Temp::new();
        let at = Utc::now();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        log.append(&record(at, Event::Started)).unwrap();
        let interrupted = log.files().unwrap()[0].path.clone();
        let mut file = fs::OpenOptions::new().append(true).open(&interrupted).unwrap();
        file.write_all(br#"{"version":"SECRET"#).unwrap();
        drop(file);
        let original = fs::read(&interrupted).unwrap();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        log.append(&record(at, Event::Completed)).unwrap();
        let exported = log.export(at).unwrap();
        assert_eq!(exported.lines().count(), 2);
        assert!(exported.contains("completed"));
        assert!(!exported.contains("SECRET"));
        assert_eq!(fs::read(&interrupted).unwrap(), original);
        assert_eq!(log.files().unwrap().len(), 2);
    }

    #[test]
    fn export_rejects_truncated_nonfinal_lines_and_complete_invalid_records() {
        let temp = Temp::new();
        let at = Utc::now();
        let mut log = LogStore::open(&temp.0, Limits::default(), at).unwrap();
        log.append(&record(at, Event::Started)).unwrap();
        let path = log.files().unwrap()[0].path.clone();
        for invalid in ["{\"version\":\"SECRET\n", "{\"version\":\"SECRET\"}", "SECRET"] {
            fs::write(&path, invalid).unwrap();
            assert!(log.export(at).is_err(), "{invalid}");
        }
    }

    #[test]
    fn strict_filename_ownership_does_not_accept_near_matches() {
        assert!(owned_name("wordweave-diagnostics-20260919-0000000001.jsonl").is_some());
        for name in ["wordweave-diagnostics-20269999-0000000001.jsonl", "wordweave-diagnostics-20260919-1.jsonl", "wordweave-diagnostics-20260919-0000000001.jsonl.bak", "user-20260919-0000000001.jsonl", "wordweave-diagnostics-20260919-000000000x.jsonl"] {
            assert!(owned_name(name).is_none(), "{name}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn junction_in_log_path_is_rejected_without_touching_its_target() {
        use std::os::windows::process::CommandExt;
        let temp = Temp::new();
        let target = temp.0.join("target");
        let link = temp.0.join("junction");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep.txt"), "KEEP").unwrap();
        let output = std::process::Command::new("cmd.exe").args(["/D", "/C", "mklink", "/J"])
            .arg(&link).arg(&target).creation_flags(0x08000000).output().unwrap();
        assert!(output.status.success(), "junction setup failed");
        assert!(LogStore::open(&link, Limits::default(), Utc::now()).is_err());
        assert!(LogStore::open(&link.join("logs"), Limits::default(), Utc::now()).is_err());
        assert!(!target.join("logs").exists());
        assert_eq!(std::fs::read_to_string(target.join("keep.txt")).unwrap(), "KEEP");
        std::fs::remove_dir(&link).unwrap();
    }
}
