use crate::{
    model::Skill,
    scheduler::{Grade, Memory},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub codex_path: String,
    pub codex_model: String,
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
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            codex_path: "codex".into(),
            codex_model: String::new(),
            examples_per_word: 6,
            batch_words: 5,
            minutes: 5,
            new_per_day: 3,
            skills: vec![Skill::Recall, Skill::Usage],
            topic: "すべて".into(),
            font_scale: 1.0,
            ai_daily_limit: 10,
            voice_id: String::new(),
            slow_speech: false,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            version: 1,
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
        }
    }
}
impl Progress {
    pub fn complete_chat(&mut self, id: &str, question: String, reply: crate::chat_action::ChatReply) -> Result<(), String> {
        let mut next = self.clone();
        let chat = next.chats.iter_mut().find(|c| c.id == id)
            .ok_or("回答の保存先の会話が見つかりません。")?;
        chat.complete(question, reply.answer, reply.execution)?;
        chat.title = reply.title;
        next.chat_action = reply.action.filter(|a| a.operation != crate::chat_action::Operation::Organize)
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
        if let Some(draft)=&self.material_draft { draft.validate_evidence()?; }
        for source in &self.material_sources { source.validate()?; }
        // An editable proposal may be incomplete; validate it only at commit.
        if serde_json::to_vec(&self.material_draft).map_err(|e| e.to_string())?.len() > 4_000_000
            || self.material_sources.len() > 10000 {
            return Err("教材案または会話参照の保存上限を超えました。".into());
        }
        if self.chats.len() > crate::chat::MAX_CHATS {
            return Err("保存できるチャットは50件までです。".into());
        }
        let mut chat_ids = BTreeSet::new();
        for chat in &self.chats {
            chat.validate()?;
            if !chat_ids.insert(&chat.id) { return Err("会話IDが重複しています。".into()); }
        }
        if let Some((id, action)) = &self.chat_action {
            if !chat_ids.contains(id) || action.base.chars().count() > 200
                || action.base.chars().any(char::is_control)
                || action.entry_id.as_ref().is_some_and(|s| s.len() > 1000) {
                return Err("保存されたチャット操作が不正です。".into());
            }
        }
        if serde_json::to_vec(&self.chats).map_err(|e| e.to_string())?.len() > 20_000_000 {
            return Err("チャットの合計が20MBを超えています。".into());
        }
        if self.version != 1 {
            return Err("対応していないデータ形式です。".into());
        }
        if self.settings.minutes == 0
            || self.settings.minutes > 30
            || self.settings.new_per_day > 20
            || !(3..=12).contains(&self.settings.examples_per_word)
            || !(1..=3000).contains(&self.settings.batch_words)
            || self.settings.skills.is_empty()
            || self.settings.ai_daily_limit > 1000
            || !(0.8..=1.6).contains(&self.settings.font_scale)
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
impl Storage {
    pub fn open() -> Result<Self, String> {
        let root = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATAが見つかりません。")?;
        Self::at(Path::new(&root).join("WordWeave5"))
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
        crate::commit::recover(&self.dir)?;
        let p = self.dir.join("progress.json");
        if !p.exists() {
            return Ok(Progress::default());
        }
        if fs::metadata(&p).map_err(|e| e.to_string())?.len() > crate::commit::PROGRESS_LIMIT {
            return Err("学習記録が大きすぎます。".into());
        }
        let text = fs::read_to_string(p).map_err(|e| e.to_string())?;
        let progress: Progress = serde_json::from_str(&text)
            .map_err(|e| format!("学習記録を読めません。元ファイルは変更していません: {e}"))?;
        progress.validate()?;
        Ok(progress)
    }
    pub fn save(&self, p: &Progress) -> Result<(), String> {
        self.save_with_limit(p, crate::commit::PROGRESS_LIMIT)
    }
    fn save_with_limit(&self, p: &Progress, limit: u64) -> Result<(), String> {
        if crate::commit::pending(&self.dir) { return Err("教材の保存途中です。ほかの記録を上書きせず再起動して復旧してください。".into()); }
        p.validate()?;
        let bytes = serde_json::to_vec_pretty(p).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > limit {
            return Err("学習記録が保存上限を超えています。元ファイルは変更していません。".into());
        }
        let path = self.dir.join("progress.json");
        if path.exists() {
            let backup_dir = self.dir.join("backups");
            fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
            let backup = backup_dir.join(format!(
                "progress-{}.json",
                chrono::Local::now().format("%Y-%m-%d")
            ));
            if !backup.exists() {
                fs::copy(&path, backup).map_err(|e| e.to_string())?;
            }
            // Keep backups: user-controlled cleanup avoids destroying recovery points.
        }
        atomic_write(&path, &bytes)
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = path.with_extension(format!("tmp-{}-{stamp}", std::process::id()));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_chat_action_preserves_transcript_and_draft() {
        let mut p = Progress::default();
        let mut chat = crate::chat::Conversation::new();
        chat.draft = "追加して".into();
        let id = chat.id.clone();
        p.chats.push(chat);
        let before = serde_json::to_value(&p).unwrap();
        let reply = crate::chat_action::ChatReply {
            answer: "追加案を確認してください".into(), title: "追加".into(),
            execution: Default::default(), action: Some(crate::chat_action::Action {
                operation: crate::chat_action::Operation::Append, base: "make".into(),
                entry_id: Some("x".repeat(1001)),
            }),
        };
        assert!(p.complete_chat(&id, "追加して".into(), reply).is_err());
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
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
        assert!(migrated.writing_logs.is_empty());
        assert_eq!(migrated.memories["x:recall"].reviews, 2);
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
            .unwrap().map(|entry| entry.unwrap().path()).collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original_bytes);
        drop(storage);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn chat_history_draft_and_reported_settings_survive_storage_reopen() {
        let dir = std::env::temp_dir().join(format!("wordweave-chat-{}-{}",
            std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        let s = Storage::at(dir.clone()).unwrap();
        let mut p = Progress::default();
        let mut chat = crate::chat::Conversation::new();
        chat.memo = "社外メール用".into();
        chat.draft = "最初の質問".into();
        chat.complete("最初の質問".into(), "回答".into(), crate::execution::Execution {
            model: Some("test-model".into()), effort: None, at: 123,
        }).unwrap();
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
        assert_eq!(chat.exchanges[0].execution.model.as_deref(), Some("test-model"));
        assert_eq!(crate::chat::prepare(chat).unwrap().included, 1);
        drop(reopened);
        fs::remove_dir_all(dir).unwrap();
    }
}
