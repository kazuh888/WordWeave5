use crate::app::controls::UiControls as _;
use crate::{
    ai,
    ink::Ink,
    media::{self, Recorder, Speaker},
};
use eframe::egui::{self, Color32, RichText};
mod activity;
mod chat_ui;
use activity::Activity;
mod audio_controls;
mod backup_ui;
mod chat_media;
mod chrome;
mod control_settings;
pub(crate) mod controls;
mod conversation_trash;
mod dashboard;
#[cfg(test)]
mod harness_tests;
mod home_art;
#[cfg(test)]
mod home_render_tests;
#[cfg(test)]
mod home_tests;
mod home_view;
#[cfg(test)]
mod layout_tests;
mod material_review;
mod materials_ui;
mod notifications;
mod playback_panel;
mod run_history;
mod session_end;
mod settings_ui;
mod stats_ui;
#[cfg(test)]
mod study_tests;
mod study_ui;
mod ux;
#[cfg(test)]
mod ux_flow_tests;
#[cfg(debug_assertions)]
pub(crate) mod visual_check;
mod vocabulary_ui;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant},
};
use wordweave5::diagnostics::{
    EntryPoint as DiagnosticEntry, ErrorClass as DiagnosticError, Event as DiagnosticEvent,
    Operation as DiagnosticOperation, Stage as DiagnosticStage,
};
use wordweave5::learning::{self, Exercise};
use wordweave5::{
    model::{self, Entry, Skill},
    scheduler::{self, Grade, Task},
    store::{self, Progress, Storage},
};

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Study,
    Deck,
    Words,
    Chat,
    Stats,
    Settings,
}
#[derive(Clone, Copy, PartialEq)]
enum Input {
    Keyboard,
    Pen,
    Voice,
}
impl Input {
    fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Pen => "pen",
            Self::Voice => "voice",
        }
    }
}
struct Session {
    review_start: usize,
    queue: VecDeque<Task>,
    elapsed: Duration,
    budget: Duration,
    paused: bool,
    completed: usize,
    credited: u32,
    introduced: std::collections::BTreeMap<String, Instant>,
}
enum AiResult {
    Text(String),
    Feedback(String),
    Exercise(Exercise),
    Words(String),
    Generated(Entry),
    Updated(Entry, bool),
    Connection(String),
    ModelChoices {
        path: String,
        models: Vec<wordweave5::effort::ModelEffort>,
    },
    Chat {
        id: String,
        question: String,
        reply: wordweave5::chat_action::ChatReply,
    },
    Material(wordweave5::material::Draft),
    Played,
    Recovered(wordweave5::run_journal::RunRecord),
    ChatRecognized {
        id: String,
        expected: String,
        asset_id: String,
        text: String,
    },
}
struct Pending {
    kind: Activity,
    key: String,
    rx: Receiver<Result<AiResult, String>>,
    cancel: Option<Arc<AtomicBool>>,
}

struct ChatFileConsent {
    chat_id: String,
    question: String,
    attachment_ids: Vec<String>,
    attachment_names: Vec<Option<String>>,
    labels: Vec<String>,
    text_names: Vec<String>,
}
struct ChatDropProposal {
    chat_id: String,
    files: Vec<std::path::PathBuf>,
    unsupported: usize,
}
impl ChatFileConsent {
    fn matches(&self, chat: &wordweave5::chat::Conversation) -> bool {
        chat.deleted_at.is_none()
            && chat.id == self.chat_id
            && chat.draft.trim() == self.question
            && chat.draft_attachments.iter().map(|a| a.original.id.as_str()).collect::<Vec<_>>()
                == self.attachment_ids.iter().map(String::as_str).collect::<Vec<_>>()
            && chat.draft_attachments.iter().map(|a| a.file_name.as_ref()).collect::<Vec<_>>()
                == self.attachment_names.iter().map(Option::as_ref).collect::<Vec<_>>()
    }
}

fn add_chat_text_files(payload: &mut serde_json::Value, texts: &[(String, String)]) -> Result<(), String> {
    if texts.is_empty() { return Ok(()); }
    payload["current_text_files"] = serde_json::json!(texts.iter()
        .map(|(name, content)| serde_json::json!({"name": name, "content": content}))
        .collect::<Vec<_>>());
    if serde_json::to_vec(payload).map_or(true, |payload| payload.len() > wordweave5::chat::CONTEXT_BYTES) {
        return Err("添付テキストと会話の文脈が送信上限を超えました。添付または文脈の選択を減らしてください。".into());
    }
    Ok(())
}

pub struct WordApp {
    storage: Option<Storage>,
    fatal: Option<String>,
    progress: Progress,
    deck: Vec<Entry>,
    page: Page,
    session: Option<Session>,
    session_summary: Option<session_end::SessionSummary>,
    study_end_confirm: bool,
    current: Option<Task>,
    answer: String,
    study_focus_pending: bool,
    revealed: bool,
    matched: Option<bool>,
    hints: usize,
    input: Input,
    ink: Ink,
    speaker: Speaker,
    speech_visible: bool,
    speech_selected: Option<audio_controls::SpeechButton>,
    speech_operation: Option<DiagnosticOperation>,
    recorder: Option<Recorder>,
    recording_operation: Option<DiagnosticOperation>,
    recording_cancel_confirm: bool,
    wav: Option<Vec<u8>>,
    pending: Option<Pending>,
    connection_check: Option<(String, bool)>,
    effort_catalog: Option<(String, Vec<wordweave5::effort::ModelEffort>)>,
    diagnostic_export: Option<String>,
    feedback: String,
    message: String,
    notification_open: bool,
    last_notification_message: String,
    notification_error: String,
    codex_path_guidance: bool,
    codex_path_focus_pending: bool,
    search: String,
    selected: usize,
    last_frame: Instant,
    last_save: Instant,
    card_start: Instant,
    font_notice: String,
    dirty: bool,
    attempted: bool,
    pending_import: Option<Vec<Entry>>,
    pending_restore: Option<Progress>,
    exercise: Option<Exercise>,
    revision: bool,
    words: Vec<String>,
    word_search: String,
    selected_word: String,
    draft_text: String,
    batch_queue: VecDeque<String>,
    batch_running: bool,
    saved_generated_words: Vec<String>,
    fetch_then_generate: bool,
    provided_words: String,
    replacement_phrase: String,
    replacement_meaning: String,
    replacement_conditions: String,
    chat_selected: usize,
    material_base: String,
    material_target: String,
    material_mode: wordweave5::material::Mode,
    material_same_base: bool,
    chat_target: String,
    chat_material_open: bool,
    chat_context_open: bool,
    chat_trash_open: bool,
    conversation_trash_open: bool,
    pending_chat_delete: Option<String>,
    pending_chat_rename: Option<(String, String)>,
    viewed_trash_id: Option<String>,
    chat_composer_height: f32,
    about_open: bool,
    chat_media_open: bool,
    chat_media_table_height: f32,
    chat_media_focus_ink: bool,
    exit_media_requested: bool,
    exit_media_scroll_to_warning: bool,
    pending_chat_file_send: Option<ChatFileConsent>,
    pending_chat_drop: Option<ChatDropProposal>,
    chat_recording_id: Option<String>,
    annotation: crate::annotation::Annotation,
    media_preview: Option<(String, egui::TextureHandle)>,
    preview_pixels: Option<(String, egui::ColorImage)>,
    file_preview: Option<(String, String, bool)>,
    run_history_open: bool,
    run_records: Vec<wordweave5::run_journal::RunRecord>,
    backup_restore: Option<wordweave5::backup::Prepared>,
    unsaved_chat_audio: Option<(String, Vec<u8>)>,
    discard_audio_confirm: bool,
    discard_annotation_confirm: bool,
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn shown(s: &str) -> &str {
    if s == "-" {
        "－"
    } else {
        s
    }
}

impl WordApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage = Storage::open();
        if let Ok(storage) = &storage {
            wordweave5::diagnostics::initialize(&storage.dir.join("diagnostics"));
        }
        let operation = DiagnosticOperation::begin(DiagnosticEntry::Application);
        let app = Self::new_with_storage(&cc.egui_ctx, storage);
        if app.fatal.is_some() {
            operation.fail(DiagnosticStage::Startup, DiagnosticError::Io);
        } else {
            operation.event(DiagnosticStage::Startup, DiagnosticEvent::Completed);
        }
        app
    }
    fn new_with_storage(ctx: &egui::Context, storage: Result<Storage, String>) -> Self {
        let mut font_notice = String::new();
        let mut fonts = egui::FontDefinitions::default();
        let windows = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Windows"));
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        let candidates = ["meiryo.ttc", "YuGothR.ttc", "msgothic.ttc"];
        if let Some(bytes) = candidates
            .iter()
            .find_map(|name| std::fs::read(windows.join("Fonts").join(name)).ok())
        {
            fonts
                .font_data
                .insert("japanese".into(), egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .push("japanese".into());
            }
        } else {
            font_notice =
                "日本語フォントが見つかりません。Windowsの日本語フォントを追加してください。"
                    .into();
        }
        let mut home_body = fonts.families[&egui::FontFamily::Proportional].clone();
        if let Ok(bytes) = std::fs::read(windows.join("Fonts").join("YuGothM.ttc")) {
            fonts
                .font_data
                .insert("home_body".into(), egui::FontData::from_owned(bytes).into());
            home_body.insert(0, "home_body".into());
        }
        fonts
            .families
            .insert(egui::FontFamily::Name("home_body".into()), home_body);
        let mut heading = fonts.families[&egui::FontFamily::Proportional].clone();
        if let Ok(bytes) = std::fs::read(windows.join("Fonts").join("YuGothB.ttc")) {
            fonts
                .font_data
                .insert("heading".into(), egui::FontData::from_owned(bytes).into());
            heading.insert(0, "heading".into());
        }
        fonts
            .families
            .insert(egui::FontFamily::Name("heading".into()), heading);
        let mut status_font = fonts.families[&egui::FontFamily::Proportional].clone();
        if let Ok(bytes) = std::fs::read(windows.join("Fonts").join("YuGothR.ttc")) {
            fonts.font_data.insert(
                "status_regular".into(),
                egui::FontData::from_owned(bytes).into(),
            );
            status_font.insert(0, "status_regular".into());
        }
        fonts
            .families
            .insert(egui::FontFamily::Name("status_regular".into()), status_font);
        ctx.set_fonts(fonts);
        ctx.set_visuals(egui::Visuals::light());
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, home_art::home_font(19.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, home_art::home_font(16.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, home_art::home_font(18.0));
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(24.0, egui::FontFamily::Name("heading".into())),
        );
        style.visuals.selection.bg_fill = ux::ACCENT;
        style.visuals.selection.stroke.color = Color32::WHITE;
        style.visuals.panel_fill = Color32::from_rgb(247, 250, 253);
        style.visuals.window_fill = Color32::WHITE;
        style.visuals.override_text_color = Some(ux::INK);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, ux::ACCENT);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, ux::ACCENT);
        ctx.set_style(style);
        let mut fatal = None;
        let storage = match storage {
            Ok(s) => Some(s),
            Err(e) => {
                fatal = Some(e);
                None
            }
        };
        let mut progress = match storage.as_ref().map(Storage::load) {
            Some(Ok(p)) => p,
            Some(Err(e)) => {
                fatal = Some(e);
                Progress::default()
            }
            None => Progress::default(),
        };
        let builtin = format!("{}\n{}", model::BUILTIN_DECK, learning::EXTRA_DECK);
        let mut deck = match model::parse_deck(&builtin) {
            Ok(d) => d,
            Err(e) => {
                fatal = Some(e);
                Vec::new()
            }
        };
        if let Some(s) = &storage {
            let path = s.dir.join("custom.tsv");
            if path.exists() {
                match std::fs::read_to_string(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|t| model::parse_deck(&t))
                {
                    Ok(additional) => {
                        for e in additional {
                            if let Some(old) = deck.iter_mut().find(|x| x.id == e.id) {
                                *old = e;
                            } else {
                                deck.push(e);
                            }
                        }
                    }
                    Err(e) => {
                        fatal = Some(format!(
                            "追加教材を読み取れません（元ファイルは変更していません）: {e}"
                        ))
                    }
                }
            }
        }
        // Each launch starts at the user-requested standard display size.
        progress.settings.font_scale = 0.8;
        ctx.set_zoom_factor(0.8);
        progress.reconcile_deck(&deck);
        let words = storage
            .as_ref()
            .and_then(|s| read_limited(&s.dir.join("words.csv"), 2_000_000).ok())
            .and_then(|s| learning::parse_words(&s).ok())
            .unwrap_or_else(|| learning::parse_words(learning::BUNDLED_NGSL).unwrap_or_default());
        let batch_queue = storage
            .as_ref()
            .and_then(|s| read_limited(&s.dir.join("generation-queue.json"), 2_000_000).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let chat_selected = wordweave5::chat::ordered_indices(&progress.chats)
            .first()
            .copied()
            .unwrap_or(0);
        let mut speaker = Speaker::new();
        if let Err(error) = speaker
            .set_volume(progress.settings.speech_volume)
            .and_then(|_| speaker.set_repeat(progress.settings.speech_repeat))
        {
            fatal = Some(error);
        }
        Self {
            storage,
            fatal,
            progress,
            deck,
            page: Page::Home,
            session: None,
            session_summary: None,
            study_end_confirm: false,
            current: None,
            answer: String::new(),
            study_focus_pending: true,
            revealed: false,
            matched: None,
            hints: 0,
            input: Input::Keyboard,
            ink: Ink::default(),
            speaker,
            speech_visible: false,
            speech_selected: None,
            speech_operation: None,
            recorder: None,
            recording_operation: None,
            recording_cancel_confirm: false,
            wav: None,
            pending: None,
            connection_check: None,
            effort_catalog: None,
            diagnostic_export: None,
            feedback: String::new(),
            message: String::new(),
            notification_open: false,
            last_notification_message: String::new(),
            notification_error: String::new(),
            codex_path_guidance: false,
            codex_path_focus_pending: false,
            search: String::new(),
            selected: 0,
            last_frame: Instant::now(),
            last_save: Instant::now(),
            card_start: Instant::now(),
            font_notice,
            dirty: true,
            attempted: false,
            pending_import: None,
            pending_restore: None,
            exercise: None,
            revision: false,
            words,
            word_search: String::new(),
            selected_word: String::new(),
            draft_text: String::new(),
            batch_queue,
            batch_running: false,
            saved_generated_words: Vec::new(),
            fetch_then_generate: false,
            provided_words: String::new(),
            replacement_phrase: String::new(),
            replacement_meaning: String::new(),
            replacement_conditions: String::new(),
            chat_selected,
            material_base: String::new(),
            material_target: String::new(),
            material_mode: wordweave5::material::Mode::New,
            material_same_base: false,
            chat_target: String::new(),
            chat_material_open: false,
            chat_context_open: false,
            chat_trash_open: false,
            conversation_trash_open: false,
            pending_chat_delete: None,
            pending_chat_rename: None,
            viewed_trash_id: None,
            chat_composer_height: 165.0,
            about_open: false,
            chat_media_open: false,
            chat_media_table_height: 210.0,
            chat_media_focus_ink: false,
            exit_media_requested: false,
            exit_media_scroll_to_warning: false,
            pending_chat_file_send: None,
            pending_chat_drop: None,
            chat_recording_id: None,
            annotation: Default::default(),
            media_preview: None,
            preview_pixels: None,
            file_preview: None,
            run_history_open: false,
            run_records: Vec::new(),
            backup_restore: None,
            unsaved_chat_audio: None,
            discard_audio_confirm: false,
            discard_annotation_confirm: false,
        }
    }
    fn persist(&mut self) {
        if self.fatal.is_some() {
            return;
        }
        if let Some(storage) = &self.storage {
            let operation = DiagnosticOperation::begin(DiagnosticEntry::Storage);
            if let Err(e) = storage.save(&self.progress) {
                operation.fail(DiagnosticStage::Save, DiagnosticError::Io);
                self.fatal=Some(format!("保存に失敗しました。これ以上の学習記録は変更しません。アプリを閉じる前に、設定画面から現在の記録をエクスポートしてください。原因: {e}"));
            } else {
                operation.event(DiagnosticStage::Save, DiagnosticEvent::Completed);
                self.dirty = false;
                self.last_save = Instant::now();
            }
        }
    }
    fn reset_answer(&mut self) {
        self.answer.clear();
        self.study_focus_pending = true;
        self.revealed = false;
        self.matched = None;
        self.hints = 0;
        self.attempted = false;
        self.ink.clear();
        self.wav = None;
        self.feedback.clear();
        self.card_start = Instant::now();
        self.exercise = None;
        self.revision = false;
        self.stop_speech();
    }
    fn key(&self) -> String {
        self.current
            .as_ref()
            .map(|t| t.key(&self.deck))
            .unwrap_or_default()
    }
    fn study_queue(&self, minutes: u32) -> VecDeque<Task> {
        let max_new = if minutes <= 2 {
            self.progress.settings.new_per_day.min(1)
        } else {
            self.progress.settings.new_per_day
        };
        scheduler::make_queue(&self.deck, &self.progress, now(), &today(), max_new)
    }
    fn start(&mut self, minutes: u32) {
        if self.fatal.is_some()
            || self.pending.is_some()
            || self.recorder.is_some()
            || self.batch_running
        {
            return;
        }
        if self.session.is_some() {
            self.page = Page::Study;
            return;
        }
        self.session_summary = None;
        self.study_end_confirm = false;
        self.message.clear();
        let queue = self.study_queue(minutes);
        self.session = Some(Session {
            review_start: self.progress.reviews.len(),
            queue,
            elapsed: Duration::ZERO,
            budget: Duration::from_secs(minutes as u64 * 60),
            paused: false,
            completed: 0,
            credited: 0,
            introduced: Default::default(),
        });
        self.page = Page::Study;
        self.next();
    }
    fn next(&mut self) {
        self.reset_answer();
        let timed_out = self.session.as_ref().is_some_and(|s| s.elapsed >= s.budget);
        if timed_out {
            self.finish_session(session_end::EndReason::Time);
            return;
        }
        let stamp = now();
        let eligible = self.session.as_ref().and_then(|s| {
            s.queue.iter().position(|task| {
                task.introduce
                    || self
                        .progress
                        .memories
                        .get(&task.key(&self.deck))
                        .map_or(true, |m| m.due <= stamp)
            })
        });
        self.current =
            eligible.and_then(|pos| self.session.as_mut().and_then(|s| s.queue.remove(pos)));
        if let Some(task) = &self.current {
            if !task.introduce && matches!(task.skill, Skill::Recall | Skill::Listening) {
                self.hints = self
                    .progress
                    .memories
                    .get(&task.key(&self.deck))
                    .map(|m| m.cue_chars as usize)
                    .unwrap_or(0);
            }
        }
        if self.current.is_none() {
            self.finish_session(session_end::EndReason::NoTasks);
        }
    }
    fn credit_time(&mut self) {
        if let Some(s) = self.session.as_mut() {
            let whole = s.elapsed.as_secs().min(u32::MAX as u64) as u32;
            let delta = whole.saturating_sub(s.credited);
            if delta > 0 {
                *self.progress.study_seconds.entry(today()).or_default() += delta;
                s.credited = whole;
                self.dirty = true;
            }
        }
    }
    fn finish(&mut self) {
        self.finish_session(session_end::EndReason::Manual);
    }
    fn finish_session(&mut self, reason: session_end::EndReason) {
        if self.session.is_none() || self.pending.is_some() || self.recorder.is_some() {
            return;
        }
        self.credit_time();
        self.persist();
        if self.fatal.is_some() {
            return;
        }
        if let Some(s) = self.session.take() {
            self.session_summary = Some(session_end::SessionSummary::collect(
                &s,
                &self.progress,
                &self.deck,
                reason,
            ));
            self.message = format!(
                "今日はここまで。{}項目に回答、学習時間は{}。",
                s.completed,
                ux::duration(s.elapsed.as_secs())
            );
            // The completion screen already presents this result; keep the
            // notification accessible without opening a duplicate popup.
            self.last_notification_message = self.message.clone();
        }
        self.current = None;
        self.stop_speech();
        self.study_end_confirm = false;
        self.page = Page::Study;
    }
    fn grade(&mut self, grade: Grade) {
        let Some(task) = self.current.clone() else {
            return;
        };
        let key = task.key(&self.deck);
        let recently_seen = self
            .session
            .as_ref()
            .and_then(|s| s.introduced.get(&key))
            .is_some_and(|t| t.elapsed() < Duration::from_secs(45));
        let assisted = self.hints > 0 || recently_seen;
        let self_assessed = matches!(task.skill, Skill::Usage | Skill::Sentence)
            || self.input != Input::Keyboard
            || self.matched != Some(true);
        let seconds = self.card_start.elapsed().as_secs().min(600) as u32;
        self.progress.record(
            key,
            grade,
            assisted,
            self.input.name(),
            now(),
            &today(),
            seconds,
            self_assessed,
        );
        if let Some(s) = self.session.as_mut() {
            s.completed += 1;
            if grade == Grade::Again
                && s.budget.saturating_sub(s.elapsed) >= Duration::from_secs(120)
            {
                s.queue.push_back(task);
            }
        }
        self.credit_time();
        self.dirty = true;
        self.persist();
        if self.fatal.is_none() {
            self.next();
        }
    }
    fn tick(&mut self, ctx: &egui::Context) {
        let delta = self.last_frame.elapsed().min(Duration::from_secs(1));
        self.last_frame = Instant::now();
        if self.fatal.is_none()
            && self.page == Page::Study
            && !self.study_end_confirm
            && self.pending.is_none()
            && ctx.input(|i| i.focused)
        {
            if let Some(s) = self.session.as_mut() {
                if !s.paused {
                    s.elapsed += delta;
                }
            }
        }
        if self
            .recorder
            .as_ref()
            .is_some_and(|r| r.elapsed() >= Duration::from_secs(30) || r.error().is_some())
        {
            self.stop_recording();
        }
        if self.last_save.elapsed() > Duration::from_secs(20) {
            self.credit_time();
            if self.dirty {
                self.persist();
            }
        }
        let outcome = self.pending.as_ref().map(|p| p.rx.try_recv());
        if let Some(result) = outcome {
            match result {
                Ok(r) => {
                    let pending = self.pending.take().unwrap();
                    if matches!(pending.kind, Activity::Connection) || pending.key == self.key() {
                        self.message.clear();
                        match r {
                            Ok(AiResult::Text(t)) => {
                                self.answer = t;
                                self.message =
                                    "認識結果を確認し、誤認識があれば直してから回答してください。"
                                        .into();
                            }
                            Ok(AiResult::ChatRecognized {
                                id,
                                expected,
                                asset_id,
                                text,
                            }) => {
                                if let Some(chat) =
                                    self.progress.chats.iter_mut().find(|c| c.id == id)
                                {
                                    if let Some(a) = chat
                                        .draft_attachments
                                        .iter_mut()
                                        .find(|a| a.original.id == asset_id)
                                    {
                                        a.transcript = Some(text.clone());
                                    }
                                    self.message = match chat.apply_recognition(&expected, &text) {
                                        Ok(()) => "認識結果を入力欄に追加した。確認・訂正してから送信する。原録音も保存している。".into(),
                                        Err(e) => format!("{e}\n認識結果：{text}"),
                                    };
                                    self.dirty = true;
                                    self.persist();
                                }
                            }
                            Ok(AiResult::Feedback(t)) => {
                                if let Some(problem) = self.exercise.clone() {
                                    self.progress.writing_logs.push(store::WritingLog {
                                        key: pending.key.clone(),
                                        at: now(),
                                        problem,
                                        answer: self.answer.clone(),
                                        feedback: t.clone(),
                                        revision: self.revision,
                                    });
                                    self.dirty = true;
                                    self.persist();
                                }
                                self.feedback = t;
                            }
                            Ok(AiResult::Exercise(e)) => {
                                self.exercise = Some(e);
                                self.answer.clear();
                                self.study_focus_pending = true;
                                self.ink.clear();
                                self.wav = None;
                                self.feedback.clear();
                                self.revision = false;
                                self.revealed = false;
                                self.matched = None;
                                self.attempted = false;
                                self.hints = 0;
                                self.card_start = Instant::now();
                            }
                            Ok(AiResult::Words(text)) => match self.cache_words(&text) {
                                Ok(()) => {
                                    if self.fetch_then_generate {
                                        self.fetch_then_generate = false;
                                        self.begin_batch(
                                            learning::parse_words(&text).unwrap_or_default(),
                                        );
                                    }
                                }
                                Err(e) => {
                                    self.fetch_then_generate = false;
                                    self.message = e;
                                }
                            },
                            Ok(AiResult::Connection(t)) => {
                                if let Some((_, success)) = self.connection_check.as_mut() {
                                    *success = true;
                                }
                                self.message = t;
                            }
                            Ok(AiResult::ModelChoices { path, models }) => {
                                if path == self.progress.settings.codex_path.trim() {
                                    self.message = format!("Codexから{}件のモデル候補を取得した。モデルを選びeffortを指定できる。", models.len());
                                    self.effort_catalog = Some((path, models));
                                } else {
                                    self.message = "実行ファイルの設定が変わったため、候補を再取得してください。".into();
                                }
                            }
                            Ok(AiResult::Recovered(record)) => {
                                self.message = format!("{}。本文は実行記録で確認できる。教材・会話には自動反映していない。",record.outcome.label());
                                self.refresh_runs();
                            }
                            Ok(AiResult::Material(draft)) => {
                                self.chat_material_open = true;
                                self.progress.material_draft = Some(draft);
                                self.material_same_base = false;
                                self.dirty = true;
                                self.persist();
                                if self.fatal.is_none() {
                                    self.message = "教材案を作成した。「教材案を確認」で差分を確認・編集して登録してください。".into();
                                }
                            }
                            Ok(AiResult::Chat {
                                id,
                                question,
                                reply,
                            }) => match self.progress.complete_chat(&id, question, reply) {
                                Ok(()) => {
                                    self.chat_target.clear();
                                    self.dirty = true;
                                    self.persist();
                                    if self.fatal.is_none() {
                                        self.message =
                                            "回答を会話履歴に保存した。続けて質問できる。".into();
                                    }
                                }
                                Err(e) => {
                                    self.message = e;
                                }
                            },
                            Ok(AiResult::Generated(e)) => {
                                let saved_word = e.base.clone();
                                if let Err(err) = self.import_deck(vec![e]) {
                                    self.batch_running = false;
                                    self.message = err;
                                } else {
                                    if !self.saved_generated_words.contains(&saved_word) {
                                        self.saved_generated_words.push(saved_word);
                                    }
                                    self.batch_queue.pop_front();
                                    if let Err(err) = self.save_queue() {
                                        self.batch_running = false;
                                        self.message = err;
                                    } else {
                                        self.message = format!(
                                            "生成・登録した。残り{}語。",
                                            self.batch_queue.len()
                                        );
                                    }
                                }
                            }
                            Ok(AiResult::Updated(e, translated)) => {
                                let id = e.id.clone();
                                if let Err(err) = self.import_deck(vec![e]) {
                                    self.message = err;
                                } else if translated {
                                    self.progress.japanese_drafts.remove(&id);
                                    self.dirty = true;
                                    self.persist();
                                    self.message = "日本語原文と英文・解説を登録した。".into();
                                } else {
                                    self.message = "新しい例文を追加した。".into();
                                }
                            }
                            Ok(AiResult::Played) => {}
                            Err(e) => {
                                self.batch_running = false;
                                self.fetch_then_generate = false;
                                self.notification_error = e.clone();
                                self.message = e;
                            }
                        }
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.batch_running = false;
                    self.fetch_then_generate = false;
                    self.message = "処理が終了したが結果を取得できなかった。入力は保持している。Codexの処理は実行記録から状態と再取得可否を確認してから再試行する。".into();
                    self.notification_error = self.message.clone();
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if self.batch_running && self.pending.is_none() && self.fatal.is_none() {
            self.launch_next_word();
        }
    }
    fn launch_ai(&mut self, action: u8) {
        if self.pending.is_some() || self.recorder.is_some() || self.fatal.is_some() {
            return;
        }
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let count = self.progress.ai_calls.get(&today()).copied().unwrap_or(0);
        if count >= self.progress.settings.ai_daily_limit {
            self.message =
                "本日のAI送信回数の上限に達しました。無料の自己評価は続けられます。".into();
            return;
        }
        let Some(task) = self.current.clone() else {
            return;
        };
        let entry = self.deck[task.index].clone();
        let answer = self.answer.clone();
        if task.skill == Skill::Sentence && action == 0 && self.exercise.is_none() {
            self.exercise = Some(Exercise {
                japanese: entry.translation.clone(),
                situation: entry.context.clone(),
                target: learning::target(&entry).into(),
                reference: entry.completed(),
            });
        }
        let exercise = self.exercise.clone();
        let png = if action == 1 {
            match self.ink.png() {
                Ok(p) => Some(p),
                Err(e) => {
                    self.message = e;
                    return;
                }
            }
        } else {
            None
        };
        let wav = if action == 2 {
            match self.wav.clone() {
                Some(w) => Some(w),
                None => {
                    self.message = "先に録音してください。".into();
                    return;
                }
            }
        } else {
            None
        };
        if action == 0 && answer.trim().is_empty() {
            self.message = "回答または自作の英文を入力してください。".into();
            return;
        }
        *self.progress.ai_calls.entry(today()).or_default() += 1;
        self.dirty = true;
        self.persist();
        if self.fatal.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let key = self.key();
        let cancel = config.cancel.clone();
        std::thread::spawn(move || {
            let result = match action {
                1 => config.read_ink(png.unwrap()).map(AiResult::Text),
                2 => config.transcribe(wav.unwrap()).map(AiResult::Text),
                3 => config.exercise(&entry).map(AiResult::Exercise),
                _ => match exercise {
                    Some(e) => config.assess_exercise(&entry, &e, &answer),
                    None => config.feedback(&entry, task.skill.label(), &answer),
                }
                .map(AiResult::Feedback),
            };
            let _ = tx.send(result);
        });
        self.pending = Some(Pending {
            key,
            kind: Activity::Study,
            rx,
            cancel: Some(cancel),
        });
        self.message = "AIに送信中…（待ち時間は学習タイマーに含めない）".into();
    }
    fn stop_recording(&mut self) {
        self.recording_cancel_confirm = false;
        if let Some(r) = self.recorder.take() {
            let operation = self.recording_operation.take();
            match r.finish_with_warning() {
                Ok(captured) => {
                    if let Some(operation) = &operation {
                        if captured.warning.is_some() {
                            operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable);
                        } else {
                            operation.event(DiagnosticStage::Record, DiagnosticEvent::Completed);
                        }
                    }
                    if let Some(id) = self.chat_recording_id.take() {
                        self.save_chat_recording(&id, captured.wav);
                    } else {
                        self.wav = Some(captured.wav);
                        self.message = "録音した（次の問題に進むまでメモリ内で保持）。".into();
                    }
                    if let Some(warning) = captured.warning {
                        self.message.push_str(&format!(
                            "\n録音障害が発生したため取得できた音声だけを保持した：{warning}"
                        ));
                    }
                }
                Err(e) => {
                    if let Some(operation) = operation {
                        operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable);
                    }
                    self.chat_recording_id = None;
                    self.message = e;
                }
            }
        }
    }
    fn say(&mut self, text: &str) {
        if let Err(error) = self.check_speech_start() {
            self.message = error;
            return;
        }
        let operation = DiagnosticOperation::begin(DiagnosticEntry::Playback);
        if let Err(e) =
            self.speaker
                .say_at_rate(text, &self.progress.settings.voice_id, self.speech_rate())
        {
            operation.fail(DiagnosticStage::Synthesize, DiagnosticError::Unavailable);
            self.message = e;
        } else {
            if let Some(previous) = self.speech_operation.replace(operation) {
                previous.event(DiagnosticStage::Play, DiagnosticEvent::Stopped);
            }
            self.speech_visible = true;
            self.speech_selected = None;
        }
    }
    fn card(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        ui.heading(format!("{}  ·  {}", entry.base, entry.meaning));
        ui.small(format!(
            "{} / {}水準（目安） / {}",
            learning::kind(entry),
            entry.level,
            entry.tag
        ));
        for (label, value) in [
            ("社外メール", shown(&entry.business)),
            ("格調・文体", shown(&entry.elevated)),
            ("語調・意味", &entry.register),
        ] {
            ui.label(RichText::new(label).small().color(ux::MUTED));
            ui.add(egui::Label::new(value).wrap());
        }
        ui.add_space(7.0);
        ui.label(&entry.usage);
        ui.separator();
        ui.label(RichText::new(entry.completed()).size(22.0));
        ui.label(&entry.translation);
        if !entry.replacements.is_empty() {
            ui.ww_collapsing("言い換えと使える条件", |ui| {
                for r in &entry.replacements {
                    ui.strong(&r.phrase);
                    ui.label(&r.meaning);
                    ui.label(&r.conditions);
                    ui.separator();
                }
            });
        }
        if !entry.examples.is_empty() {
            ui.ww_collapsing(
                format!("語感を掴む例文（{}件）", entry.examples.len()),
                |ui| {
                    for (i, x) in entry.examples.iter().enumerate() {
                        ui.strong(format!("{}. {}", i + 1, x.english));
                        ui.label(&x.japanese);
                        ui.label(&x.note);
                        if ui
                            .ww_small_button(format!("例文{}を読み上げ", i + 1))
                            .clicked()
                        {
                            self.say(&x.english);
                        }
                        ui.separator();
                    }
                },
            );
        }

        if ui.ww_button("例文を聞く").clicked() {
            self.say(&entry.completed());
        }
    }
    fn home(&mut self, ui: &mut egui::Ui) {
        self.home_dashboard(ui);
    }
    fn study(&mut self, ui: &mut egui::Ui) {
        if self.current.is_none() {
            self.study_content(ui);
            return;
        }
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
            ui.spacing_mut().button_padding = egui::vec2(16.0, 12.0);
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 16.0),
                (egui::TextStyle::Button, 18.0),
            ] {
                ui.style_mut()
                    .text_styles
                    .insert(style, home_art::home_font(size));
            }
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(30.0, egui::FontFamily::Name("heading".into())),
            );
            ui.style_mut().visuals.override_text_color = Some(home_art::INK);
            egui::Frame::new()
                .inner_margin(16)
                .show(ui, |ui| self.study_content(ui));
        });
    }
    fn study_content(&mut self, ui: &mut egui::Ui) {
        let Some(task) = self.current.clone() else {
            if self.session_summary.is_some() {
                self.session_result(ui);
            } else {
                self.study_start(ui);
            }
            return;
        };
        let entry = self.deck[task.index].clone();
        let (elapsed, budget, paused) = self
            .session
            .as_ref()
            .map(|s| (s.elapsed.as_secs(), s.budget.as_secs(), s.paused))
            .unwrap_or((0, 300, false));
        ux::panel(ui, false, |ui| {
            ui.horizontal_wrapped(|ui| {
                home_art::badge(ui, home_art::Icon::Book, home_art::BLUE, 48.0);
                home_art::title(ui, task.skill.label(), 26.0);
                let remaining = budget.saturating_sub(elapsed);
                ui.label(format!(
                    "残り {}:{:02}  /  今回の回答 {}回",
                    remaining / 60,
                    remaining % 60,
                    self.session.as_ref().map_or(0, |s| s.completed)
                ));
                if ui
                    .ww_button(if paused { "再開" } else { "一時停止" })
                    .clicked()
                {
                    if let Some(s) = self.session.as_mut() {
                        s.paused = !paused;
                    }
                }
                if ui
                    .add_enabled(
                        self.pending.is_none() && self.recorder.is_none(),
                        crate::app::controls::Button::new("ここで終える"),
                    )
                    .clicked()
                {
                    if !self.answer.trim().is_empty()
                        || !self.ink.empty()
                        || self.wav.is_some()
                        || self.revealed
                    {
                        self.study_end_confirm = true;
                    } else {
                        self.finish();
                    }
                }
            });
        });
        if self.current.is_none() {
            return;
        }
        if self.study_end_confirm {
            self.request_study_end(ui);
            return;
        }
        ui.label(format!(
            "学習時間 {} / 目安 {}",
            ux::duration(elapsed),
            ux::duration(budget)
        ));
        ui.add(
            egui::ProgressBar::new((elapsed as f32 / budget.max(1) as f32).min(1.0))
                .desired_height(10.0)
                .fill(home_art::BLUE),
        );
        if self.session.as_ref().is_some_and(|s| s.paused) {
            ui.label("休憩中。再開するまで学習時間は増えない。");
            if self.recorder.is_some() {
                ui.label("学習の一時停止と録音の一時停止は別である。録音は以下から操作できる。");
                self.recording_controls(ui, "録音を終了");
            }
            return;
        }
        if elapsed >= budget {
            ui.colored_label(
                Color32::from_rgb(135, 90, 15),
                "目標時間に到達。この問題を終えたら終了する。",
            );
        }
        ui.add_space(12.0);
        if task.introduce {
            ux::panel(ui, true, |ui| {
                ui.heading("はじめての項目");
                ui.label("意味と使用条件を確認しましょう。");
                self.card(ui, &entry);
            });
            ui.add_space(12.0);
            if ui.ww_button("確認した・別の問題のあとで思い出す").clicked() {
                if let Some(s) = self.session.as_mut() {
                    s.introduced.insert(task.key(&self.deck), Instant::now());
                    let mut retry = task;
                    retry.introduce = false;
                    let pos = s.queue.len().min(2);
                    s.queue.insert(pos, retry);
                }
                self.next();
            }
            return;
        }
        ux::panel(ui, true, |ui| match task.skill {
            Skill::Recall => {
                if entry.accepts(&entry.base) {
                    ui.heading(format!("{} · 場面に合う表現を思い出す", entry.meaning));
                } else {
                    ui.heading(format!("{} · {}", entry.base, entry.meaning));
                }
                ui.label(format!("場面：{}", entry.context));
                ui.add_space(8.0);
                ui.label(RichText::new(&entry.example).size(23.0));
                ui.label(&entry.translation);
                ui.small("空欄に入る語句を回答する。自然な別解は、照合後に自己評価できる。");
            }
            Skill::Usage => {
                ui.heading(format!("{} の使い分け", entry.base));
                ui.label(&entry.question);
                ui.small("日本語の短い説明でもよい。違いを自分の言葉で思い出す。");
            }
            Skill::Listening => {
                ui.heading("聞こえた語句を英語で回答する");
                ui.small("表示から答えを推測せず、音を聞いて綴る練習。");
                if ui
                    .add_enabled(
                        self.recorder.is_none(),
                        crate::app::controls::Button::new("語句を聞く / もう一度"),
                    )
                    .clicked()
                {
                    self.say(entry.answer());
                }
            }
            Skill::Sentence => {
                ui.heading("日本語から英作文する");
                ui.label(format!("対象表現：{}", learning::target(&entry)));
                if let Some(problem) = &self.exercise {
                    ui.label(format!("場面：{}", problem.situation));
                    ui.label(RichText::new(&problem.japanese).size(22.0));
                } else {
                    ui.label(format!("場面：{}", entry.context));
                    ui.label(RichText::new(&entry.translation).size(22.0));
                    ui.small(
                        "現在は教材の日本語訳を出題中。AIで別の場面の問題を作ることもできる。",
                    );
                    if ui
                        .add_enabled(
                            !self.revealed
                                && self.answer.trim().is_empty()
                                && self.ink.empty()
                                && self.wav.is_none()
                                && self.pending.is_none()
                                && self.recorder.is_none(),
                            crate::app::controls::Button::new(
                                "新しい日本語の問題を生成（Codexへ送信）",
                            ),
                        )
                        .clicked()
                    {
                        self.launch_ai(3);
                    }
                }
                ui.small("意味の一致・文法・自然さと、対象表現を使えたかを分けて確認する。");
            }
        });
        ui.add_space(12.0);
        if !self.revealed {
            ux::panel(ui, false, |ui| {
                let previous_input = self.input;
                ui.horizontal_wrapped(|ui| {
                    ui.add_enabled_ui(self.recorder.is_none() && self.pending.is_none(), |ui| {
                        ui.ww_selectable_value(&mut self.input, Input::Keyboard, "キーボード入力");
                        ui.ww_selectable_value(&mut self.input, Input::Pen, "手書き入力");
                        ui.ww_selectable_value(&mut self.input, Input::Voice, "音声入力");
                    });
                });
                if previous_input != self.input && self.input == Input::Keyboard {
                    self.study_focus_pending = true;
                }
                ui.add_enabled_ui(self.pending.is_none(), |ui| {
                    if self.input == Input::Pen {
                        self.ink.ui(ui);
                    }
                    if self.input == Input::Voice {
                        if self.recorder.is_some() {
                            self.recording_controls(ui, "録音を終了");
                        } else {
                            ui.horizontal(|ui| {
                                if ui.ww_button("録音する（英語）").clicked() {
                                    if !self.stop_speech() {
                                        return;
                                    }
                                    let operation =
                                        DiagnosticOperation::begin(DiagnosticEntry::Recording);
                                    match Recorder::start() {
                                        Ok(r) => {
                                            operation.event(
                                                DiagnosticStage::Record,
                                                DiagnosticEvent::Started,
                                            );
                                            self.recording_operation = Some(operation);
                                            self.recording_cancel_confirm = false;
                                            self.recorder = Some(r);
                                        }
                                        Err(e) => {
                                            operation.fail(
                                                DiagnosticStage::Record,
                                                DiagnosticError::Unavailable,
                                            );
                                            self.message = e;
                                        }
                                    }
                                }
                                if ui
                                    .add_enabled(
                                        self.wav.is_some(),
                                        crate::app::controls::Button::new("自分の声を聞く"),
                                    )
                                    .clicked()
                                {
                                    if let (Some(wav), Some(storage)) =
                                        (self.wav.clone(), self.storage.as_ref())
                                    {
                                        let dir = storage.dir.clone();
                                        let (tx, rx) = mpsc::channel();
                                        let key = self.key();
                                        std::thread::spawn(move || {
                                            let _ = tx.send(
                                                media::playback(wav, dir).map(|_| AiResult::Played),
                                            );
                                        });
                                        self.pending = Some(Pending {
                                            key,
                                            kind: Activity::Playback,
                                            rx,
                                            cancel: None,
                                        });
                                    }
                                }
                            });
                        }
                    }
                    let check_with_key = self.study_answer_input(ui, task.skill);
                    egui::Frame::new()
                        .fill(Color32::from_rgb(255, 251, 240))
                        .corner_radius(8)
                        .inner_margin(12)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if matches!(task.skill, Skill::Recall | Skill::Listening)
                                    && ui.ww_button("文字のヒント").clicked()
                                {
                                    self.hints =
                                        (self.hints + 1).min(entry.answer().chars().count());
                                }
                                if self.hints > 0
                                    && matches!(task.skill, Skill::Recall | Skill::Listening)
                                {
                                    ui.label(format!("ヒント：{}", entry.hint(self.hints)));
                                }
                            });
                        });
                    ui.add_enabled_ui(self.recorder.is_none(), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if self.input == Input::Pen
                                && ui
                                    .ww_button("手書き英語をAIで文字起こし（Codexへ送信）")
                                    .clicked()
                            {
                                self.launch_ai(1);
                            }
                            if self.input == Input::Voice
                                && ui
                                    .ww_button("録音をAIで文字起こし（Codexへ送信）")
                                    .clicked()
                            {
                                self.launch_ai(2);
                            }
                            let can_answer = !self.answer.trim().is_empty()
                                || (self.input == Input::Pen && !self.ink.empty())
                                || (self.input == Input::Voice && self.wav.is_some());
                            if ux::primary(ui, "回答を確認", can_answer).clicked()
                                || (check_with_key && !self.answer.trim().is_empty())
                                || ui
                                    .add(
                                        crate::app::controls::Button::new(
                                            "わからないので答えを見る",
                                        )
                                        .wrap(),
                                    )
                                    .clicked()
                            {
                                self.check_study_answer(&entry, task.skill);
                            }
                        });
                    });
                });
            });
        } else {
            ux::panel(ui, false, |ui| self.study_feedback(ui, &entry, task.skill));
        }
        if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("処理中…");
            });
        }
        ui.small(format!(
            "Codex（ChatGPT認証） / 本日の送信試行 {} / {}回",
            self.progress.ai_calls.get(&today()).unwrap_or(&0),
            self.progress.settings.ai_daily_limit
        ));
    }
    fn cache_words(&mut self, text: &str) -> Result<(), String> {
        let incoming = learning::parse_words(text)?;
        let mut combined = self.words.clone();
        combined.extend(incoming);
        let words = learning::parse_words(&combined.join("\n"))?;
        let storage = self.storage.as_ref().ok_or("保存先がありません。")?;
        store::atomic_write(&storage.dir.join("words.csv"), words.join("\n").as_bytes())?;
        self.words = words;
        self.message = format!("候補{}語を保存した。", self.words.len());
        Ok(())
    }
    fn save_queue(&self) -> Result<(), String> {
        let storage = self.storage.as_ref().ok_or("保存先がありません。")?;
        store::atomic_write(
            &storage.dir.join("generation-queue.json"),
            &serde_json::to_vec(&self.batch_queue).map_err(|e| e.to_string())?,
        )
    }
    fn begin_batch(&mut self, words: Vec<String>) {
        if self.pending.is_some() || self.session.is_some() || self.fatal.is_some() {
            return;
        }
        let deck = &self.deck;
        let existing = |w: &str| {
            deck.iter().any(|e| {
                e.id.starts_with("web_")
                    && model::normalize(&e.base) == model::normalize(w)
                    && !e.examples.is_empty()
            })
        };
        let mut selected: VecDeque<String> = words
            .into_iter()
            .filter(|w| !existing(w))
            .take(self.progress.settings.batch_words)
            .collect();
        let mut seen = std::collections::BTreeSet::new();
        selected.retain(|w| seen.insert(model::normalize(w)));
        // Keep an interrupted queue; new requests are appended without losing prior work.
        for word in selected {
            if !self.batch_queue.contains(&word) {
                self.batch_queue.push_back(word);
            }
        }
        self.batch_queue.retain(|w| !existing(w));
        if self.batch_queue.is_empty() {
            self.message =
                "対象語はすべて自動生成・登録済みである。例文追加は教材画面から実行できる。".into();
            return;
        }
        if let Err(e) = self.save_queue() {
            self.message = e;
            return;
        }
        self.batch_running = true;
        self.message = format!("{}語の生成・登録を開始する。", self.batch_queue.len());
    }
    fn launch_next_word(&mut self) {
        let Some(word) = self.batch_queue.front().cloned() else {
            self.batch_running = false;
            self.message = "選択した語の生成・登録が完了した。".into();
            return;
        };
        // Recovery after a crash between deck save and queue save.
        if self.deck.iter().any(|e| {
            e.id.starts_with("web_")
                && model::normalize(&e.base) == model::normalize(&word)
                && !e.examples.is_empty()
        }) {
            self.batch_queue.pop_front();
            if let Err(e) = self.save_queue() {
                self.batch_running = false;
                self.message = e;
            }
            return;
        }
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.batch_running = false;
                self.message = e;
                return;
            }
        };
        if !self.reserve_generation() {
            self.batch_running = false;
            return;
        }
        let source = if learning::BUNDLED_NGSL.lines().any(|w| w == word) {
            "NGSL 1.2 / AI生成"
        } else {
            "提供語 / AI生成"
        }
        .to_string();
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.message = format!(
            "{word}：言い換えと{}例文を生成中（残り{}語）",
            config.examples,
            self.batch_queue.len()
        );
        std::thread::spawn(move || {
            let _ = tx.send(config.draft_word(&word).map(|mut e| {
                e.tag = source;
                AiResult::Generated(e)
            }));
        });
        self.pending = Some(Pending {
            key: String::new(),
            kind: Activity::Generate,
            rx,
            cancel: Some(cancel),
        });
    }
    fn reserve_generation(&mut self) -> bool {
        if self.progress.ai_calls.get(&today()).copied().unwrap_or(0)
            >= self.progress.settings.ai_daily_limit
        {
            self.message="本日の生成上限に達した。設定の上限を変更するか、翌日「未処理の語から再開」を押してください。".into();
            return false;
        }
        *self.progress.ai_calls.entry(today()).or_default() += 1;
        self.dirty = true;
        self.persist();
        self.fatal.is_none()
    }
    fn launch_content(&mut self, action: u8, entry: Option<Entry>) {
        if self.pending.is_some()
            || self.recorder.is_some()
            || (action != 0 && self.session.is_some())
            || self.fatal.is_some()
        {
            return;
        }
        if action == 0 {
            self.connection_check =
                Some((self.progress.settings.codex_path.trim().to_string(), false));
        }
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let japanese = entry
            .as_ref()
            .and_then(|e| self.progress.japanese_drafts.get(&e.id))
            .cloned()
            .unwrap_or_default();
        if action == 2 && (japanese.trim().is_empty() || japanese.chars().count() > 2000) {
            self.message = "日本語原文を1〜2,000文字で登録してください。".into();
            return;
        }
        if action != 0 && !self.reserve_generation() {
            return;
        }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = if action == 0 {
                config.check().map(AiResult::Connection)
            } else if let Some(mut e) = entry {
                if action == 1 {
                    config.more_examples(&e).map(|examples| {
                        e.examples.extend(examples);
                        AiResult::Updated(e, false)
                    })
                } else {
                    config.translate(&e, &japanese).map(|x| {
                        e.examples.push(x);
                        AiResult::Updated(e, true)
                    })
                }
            } else {
                Err("教材を選択してください。".into())
            };
            let _ = tx.send(result);
        });
        self.pending = Some(Pending {
            key: String::new(),
            rx,
            kind: if action == 0 {
                Activity::Connection
            } else {
                Activity::Generate
            },
            cancel: Some(cancel),
        });
        self.message = "Codexで処理中…".into();
    }
    fn launch_chat(&mut self) {
        self.launch_chat_with_file_consent(false);
    }
    fn launch_chat_with_file_consent(&mut self, consented: bool) {
        if self.progress.chat_action.is_some() {
            self.message =
                "確認中の教材操作を確定またはキャンセルしてから送信してください。".into();
            return;
        }
        if self.pending.is_some()
            || self.session.is_some()
            || self.batch_running
            || self.fatal.is_some()
        {
            return;
        }
        let Some(chat) = self.progress.chats.get(self.chat_selected) else {
            return;
        };
        if chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES {
            self.message =
                "この会話は200往復に達した。引き継ぎメモをコピーして新しい会話を作成してください。"
                    .into();
            return;
        }
        let mut context = match wordweave5::chat::prepare_with_catalog(
            chat,
            &self.deck,
            &self.progress.deleted_entries,
        ) {
            Ok(c) => c,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let id = chat.id.clone();
        let question = chat.draft.trim().to_string();
        let attachments = chat.draft_attachments.clone();
        let images = match self.attachment_images(&attachments) {
            Ok(images) => images,
            Err(error) => {
                self.message = error;
                return;
            }
        };
        let texts = match self.attachment_texts(&attachments) {
            Ok(texts) => texts,
            Err(error) => { self.message = error; return; }
        };
        if !texts.is_empty() && !consented {
            let labels = attachments.iter().map(|attachment| {
                let kind = if attachment.image.is_some() { "画像" }
                    else if attachment.original.kind == wordweave5::assets::AssetKind::AudioWav { "音声" }
                    else { "ファイル" };
                format!("{kind}：{}", attachment.file_name.as_deref().unwrap_or(&attachment.source_text))
            }).collect();
            self.pending_chat_file_send = Some(ChatFileConsent {
                chat_id: id,
                question,
                attachment_ids: attachments.iter().map(|a| a.original.id.clone()).collect(),
                attachment_names: attachments.iter().map(|a| a.file_name.clone()).collect(),
                labels,
                text_names: texts.iter().map(|(name, _)| name.clone()).collect(),
            });
            return;
        }
        if let Err(error) = add_chat_text_files(&mut context.payload, &texts) {
            self.message = error;
            return;
        }
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        // This also saves the draft before sending, so retries and restarts
        // preserve it even when the child fails or the app is closed.
        if !self.reserve_generation() {
            return;
        }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = config
                .chat(context.payload, images)
                .map(|reply| AiResult::Chat {
                    id,
                    question,
                    reply,
                });
            let _ = tx.send(result);
        });
        self.pending = Some(Pending {
            key: String::new(),
            rx,
            cancel: Some(cancel),
            kind: Activity::Chat,
        });
        self.message = "Codexに質問中…".into();
    }
    fn launch_material(&mut self) {
        if self.pending.is_some()
            || self.session.is_some()
            || self.batch_running
            || self.recorder.is_some()
            || self.fatal.is_some()
            || self.progress.material_draft.is_some()
        {
            return;
        }
        let Some(chat) = self.progress.chats.get(self.chat_selected) else {
            return;
        };
        let baseline = if self.material_mode == wordweave5::material::Mode::New {
            None
        } else {
            self.deck
                .iter()
                .find(|e| {
                    e.id == self.material_target && !self.progress.deleted_entries.contains(&e.id)
                })
                .cloned()
        };
        let request = match wordweave5::material::Request::new(
            chat,
            &self.material_base,
            self.material_mode,
            baseline,
        ) {
            Ok(r) => r,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let attachments: Vec<_> = request
            .source
            .snapshots
            .iter()
            .flat_map(|s| s.exchange.attachments.clone())
            .collect();
        let images = match self.attachment_images(&attachments) {
            Ok(images) => images,
            Err(error) => {
                self.message = error;
                return;
            }
        };
        if !self.reserve_generation() {
            return;
        }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(config.material(request, images).map(AiResult::Material));
        });
        self.pending = Some(Pending {
            key: String::new(),
            rx,
            cancel: Some(cancel),
            kind: Activity::Material,
        });
        self.message = "選択したチャットから教材案を作成中…".into();
    }
    fn material_panel(&mut self, ui: &mut egui::Ui) {
        use wordweave5::material::Mode;
        ui.separator();
        ui.heading("チャットを教材に反映");
        let idle = self.pending.is_none()
            && self.session.is_none()
            && !self.batch_running
            && self.recorder.is_none();
        ui.small("上の履歴で反映するやり取りを選ぶ。選択した質問・回答と反映先の教材をCodexへ送り、登録前に確認する。");
        if self.progress.material_draft.is_none() {
            ui.add_enabled_ui(idle, |ui| {
                ui.horizontal(|ui| {
                    ui.label("対象の基本語・熟語");
                    if ui.add(egui::TextEdit::singleline(&mut self.material_base).char_limit(100)).changed() {
                        self.material_target.clear();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    for mode in [Mode::New, Mode::Append, Mode::Correct] {
                        ui.ww_selectable_value(&mut self.material_mode, mode, mode.label());
                    }
                });
                let matches: Vec<_> = self.deck.iter().filter(|e| !self.progress.deleted_entries.contains(&e.id)
                    && model::normalize(&e.base) == model::normalize(&self.material_base)).collect();
                if self.material_mode != Mode::New {
                    if matches.len() == 1 && self.material_target.is_empty() { self.material_target = matches[0].id.clone(); }
                    egui::ComboBox::from_id_salt("material-target")
                        .selected_text(matches.iter().find(|e| e.id == self.material_target)
                            .map(|e| format!("{} / {}", e.base, e.meaning)).unwrap_or_else(|| "反映先の意味・用法を選択".into()))
                        .show_ui(ui, |ui| {
                            for e in &matches {
                                ui.ww_selectable_value(&mut self.material_target, e.id.clone(), format!("{} / {} / {} [{}]", e.base, e.meaning, e.context, e.id));
                            }
                        });
                    if matches.is_empty() { ui.label("同じ基本語の教材がない。新規登録を選択するか、対象語を確認してください。"); }
                } else if !matches.is_empty() {
                    ui.label(format!("同じ基本語が{}件ある。既存教材に加える場合は追加・訂正を選択する。", matches.len()));
                }
                let selected = self.progress.chats.get(self.chat_selected)
                    .map(|c| c.exchanges.iter().filter(|e| e.for_material).count()).unwrap_or(0);
                ui.label(format!("教材化の対象：{selected}往復"));
                if ui.add_enabled(selected > 0, crate::app::controls::Button::new("教材案を作成")).clicked() { self.launch_material(); }
            });
            return;
        }
        let mut draft = self.progress.material_draft.clone().unwrap();
        ui.strong(format!("{}：{}", draft.mode.label(), draft.candidate.base));
        for notice in &draft.notices {
            ui.label(notice);
        }
        let source = draft.source.clone();
        if ui.ww_button("元の会話を表示").clicked() {
            self.open_material_source(&source);
        }
        let before = model::deck_text(&[draft.candidate.clone()]);
        ui.ww_collapsing("教材案を編集", |ui| {
            ui.add_enabled_ui(idle, |ui| edit_material(ui, &mut draft));
        });
        let changed = before != model::deck_text(&[draft.candidate.clone()]);
        let ready = if draft
            .baseline
            .as_ref()
            .is_some_and(|e| self.progress.deleted_entries.contains(&e.id))
        {
            Err("対象教材は削除済みである。復元してから登録する。".into())
        } else {
            draft.ready(&self.deck, self.material_same_base)
        };
        self.material_comparison(ui, &draft);
        if draft.mode == Mode::New
            && self
                .deck
                .iter()
                .any(|e| model::normalize(&e.base) == model::normalize(&draft.candidate.base))
        {
            ui.add_enabled(
                idle,
                egui::Checkbox::new(
                    &mut self.material_same_base,
                    "同じ基本語の別用法として新規登録する",
                ),
            );
        }
        ui.label(if draft.resets_learning() {
            "基本の説明・問題・正解等が変わるため、この教材の復習状態は再学習に戻る。"
        } else if draft.baseline.is_some() {
            "補足の例文・言い換えの変更であるため、既存の復習成績は維持する。"
        } else {
            "新しい教材として登録する。"
        });
        if let Err(e) = &ready {
            ui.colored_label(Color32::RED, e);
        }
        let mut commit = false;
        let mut discard = false;
        ui.add_enabled_ui(idle, |ui| {
            ui.horizontal(|ui| {
                commit = ui
                    .add_enabled(
                        ready.is_ok(),
                        crate::app::controls::Button::new("内容を確認して教材に登録"),
                    )
                    .clicked();
                discard = ui.ww_button("教材案を破棄").clicked();
            })
        });
        if changed {
            self.progress.material_draft = Some(draft.clone());
            self.dirty = true;
        }
        if discard {
            self.progress.material_draft = None;
            self.dirty = true;
            self.persist();
        } else if commit {
            if let Ok(entry) = ready {
                let old_progress = self.progress.clone();
                let old_deck = model::deck_text(&self.deck);
                self.progress.material_sources.push(source);
                self.progress.material_draft = None;
                if let Err(e) = self.progress.validate() {
                    self.progress = old_progress;
                    self.message = e;
                    return;
                }
                match self.import_deck(vec![entry]) {
                    Ok(()) => {
                        self.message =
                            "チャットから教材を登録した。元の会話への参照も保存した。".into()
                    }
                    Err(e) => {
                        // If the deck write succeeded but progress save failed,
                        // preserve matching source metadata for recovery/export.
                        if model::deck_text(&self.deck) == old_deck {
                            self.progress = old_progress;
                        }
                        self.message = e;
                    }
                }
            }
        }
    }
    fn open_material_source(&mut self, source: &wordweave5::material::Source) {
        if let Some(index) = self
            .progress
            .chats
            .iter()
            .position(|c| c.id == source.conversation_id)
        {
            if self.progress.chats[index].deleted_at.is_some() {
                self.viewed_trash_id = Some(source.conversation_id.clone());
                self.message = "根拠の会話はごみ箱にある。読み取り専用で表示する。".into();
                return;
            }
            self.chat_selected = index;
            self.page = Page::Chat;
            self.message = format!(
                "元の会話を表示した。参照したやり取り番号：{}",
                source
                    .exchange_indices
                    .iter()
                    .map(|i| (i + 1).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            self.message = "元の会話は現在の学習記録にありません。".into();
        }
    }
    fn import_deck(&mut self, items: Vec<Entry>) -> Result<(), String> {
        if self.session.is_some() || self.pending.is_some() || self.recorder.is_some() {
            return Err("学習・録音・通信を終了してから取り込んでください。".into());
        }
        let storage = self.storage.as_ref().ok_or("保存先がありません。")?;
        let mut merged = self.deck.clone();
        for entry in items {
            if let Some(old) = merged.iter_mut().find(|x| x.id == entry.id) {
                *old = entry;
            } else {
                merged.push(entry);
            }
        }
        let text = model::deck_text(&merged);
        model::parse_deck(&text)?;
        let mut next = self.progress.clone();
        let changed = next.reconcile_deck(&merged);
        if let Err(e) = wordweave5::commit::save(&storage.dir, &text, &next) {
            if wordweave5::commit::pending(&storage.dir) {
                self.fatal = Some(format!(
                    "教材保存が途中で停止した。再起動時に復旧する。原因：{e}"
                ));
            }
            return Err(e);
        }
        self.progress = next;
        self.deck = merged;
        self.dirty = false;
        self.message = format!("教材を取り込んだ。{changed}項目の復習状態を更新した。");
        Ok(())
    }
    fn restore(&mut self, mut progress: Progress) -> Result<(), String> {
        if self.session.is_some() || self.pending.is_some() || self.recorder.is_some() {
            return Err("学習・録音・通信を終了してから復元してください。".into());
        }
        let storage = self.storage.as_ref().ok_or("保存先がありません。")?;
        wordweave5::backup::verify_references(&storage.dir, &progress)?;
        wordweave5::backup::preserve_current_progress(storage, &self.progress)?;
        let current = storage.dir.join("progress.json");
        if current.exists() {
            std::fs::copy(
                &current,
                storage.dir.join(format!(
                    "progress-before-restore-{}.json",
                    chrono::Utc::now().timestamp_millis()
                )),
            )
            .map_err(|e| e.to_string())?;
        }
        // Restoring an older backup must not reset today's paid-request counter.
        let used = self.progress.ai_calls.get(&today()).copied().unwrap_or(0);
        let restored = progress.ai_calls.entry(today()).or_default();
        *restored = (*restored).max(used);
        progress.reconcile_deck(&self.deck);
        storage.save(&progress)?;
        self.progress = progress;
        self.reset_chat_view_after_restore();
        self.fatal = None;
        self.dirty = false;
        self.message =
            "学習記録を復元した。必要に応じて設定画面でCodex接続を確認してください。".into();
        Ok(())
    }
    fn confirmations(&mut self, ctx: &egui::Context) {
        if let Some(items) = self.pending_import.as_ref() {
            let added = items
                .iter()
                .filter(|x| !self.deck.iter().any(|e| e.id == x.id))
                .count();
            let updated = items
                .iter()
                .filter(|x| {
                    self.deck
                        .iter()
                        .any(|e| e.id == x.id && e.to_tsv() != x.to_tsv())
                })
                .count();
            let (mut apply, mut cancel) = (false, false);
            egui::Window::new("教材の取り込みを確認")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ux::dialog_body(ui);
                    ui.label(format!("追加：{added}項目 / 内容の変更：{updated}項目"));
                    ui.label("問題・解答・用法を変更した項目は再学習に戻す。追加例文・言い換えだけの変更では成績を保持する。以前の教材は退避する。");
                    ui.horizontal(|ui| {
                        apply = ui.ww_button("取り込む").clicked();
                        cancel = ui.ww_button("キャンセル").clicked();
                    });
                });
            if apply {
                let items = self.pending_import.take().unwrap();
                if let Err(e) = self.import_deck(items) {
                    self.message = e;
                }
            } else if cancel {
                self.pending_import = None;
            }
        }
        if self.pending_restore.is_some() {
            let (mut apply, mut cancel) = (false, false);
            egui::Window::new("学習記録の復元を確認").collapsible(false).resizable(false).show(ctx,|ui|{
                ux::dialog_body(ui);
                ui.label("現在の記録を退避し、選択した記録に戻す。現在の教材と内容が異なる項目は再学習にする。");
                ui.horizontal(|ui|{apply=ui.ww_button("復元する").clicked();cancel=ui.ww_button("キャンセル").clicked();});
            });
            if apply {
                let p = self.pending_restore.take().unwrap();
                if let Err(e) = self.restore(p) {
                    self.message = e;
                }
            } else if cancel {
                self.pending_restore = None;
            }
        }
    }
}

impl WordApp {
    fn update_ui(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.chat_recording_id.is_some() {
                self.stop_recording();
            }
            if self.unsaved_chat_audio.is_some()
                || self.annotation.frozen()
                || !self.annotation.text.is_empty()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.page = Page::Chat;
                self.chat_media_open = true;
                self.exit_media_requested = true;
                self.exit_media_scroll_to_warning = true;
                self.message="終了前に未保存の録音・手書き注釈を確認してください。".into();
            }
        }
        self.tick(ctx);
        let confirming = self.pending_import.is_some()
            || self.pending_restore.is_some()
            || self.backup_restore.is_some();
        self.handle_zoom_input(ctx);
        self.navigation_chrome(ctx, confirming);
        self.operation_notice(ctx);
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            self.status_summary(ui);
        });
        self.speech_controls(ctx);
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.add_enabled_ui(!confirming,|ui|{
            if self.page == Page::Chat && self.fatal.is_none() {
                self.chat_page(ui);
                return;
            }
            if self.page == Page::Deck && self.fatal.is_none() {
                self.deck_page(ui);
                return;
            }
            egui::ScrollArea::vertical().id_salt(format!("page-{}",self.page as u8)).show(ui,|ui|{
                if self.fatal.is_some()&&self.page!=Page::Settings {
                    ui.heading("学習を停止している");ui.label("設定画面で記録のエクスポート・復元を確認してください。元の保存ファイルは自動で初期化しない。");return;
                }
                match self.page{Page::Home=>self.home(ui),Page::Study=>self.study(ui),Page::Deck=>self.deck_page(ui),Page::Words=>self.words_page(ui),Page::Chat=>{},Page::Stats=>self.stats(ui),Page::Settings=>self.settings(ui,ctx)}
            });
            });
        });
        self.confirmations(ctx);
        self.notification_window(ctx);
        self.version_dialog(ctx);
        self.run_history(ctx);
        self.backup_confirmation(ctx);
        self.conversation_trash_windows(ctx);
        self.recording_cancel_dialog(ctx);
        let media_idle = self.pending.is_none()
            && self.recorder.is_none()
            && self.session.is_none()
            && !self.batch_running
            && !confirming;
        self.chat_media_windows(ctx, media_idle);
        self.chat_file_send_confirmation(ctx);
        self.media_preview_window(ctx);
        self.file_preview_window(ctx);
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}
impl eframe::App for WordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_ui(ctx);
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(p) = self.pending.as_ref() {
            if let Some(c) = p.cancel.as_ref() {
                c.store(true, Ordering::Relaxed);
                let _ = p.rx.recv_timeout(Duration::from_secs(2));
            }
        }
        if self.chat_recording_id.is_some() {
            self.stop_recording();
        } else {
            self.recorder = None;
        }
        self.stop_speech();
        self.credit_time();
        if self.dirty {
            self.persist();
        }
    }
}

fn edit_material(ui: &mut egui::Ui, draft: &mut wordweave5::material::Draft) {
    let append = draft.mode == wordweave5::material::Mode::Append;
    let baseline_replacements = draft
        .baseline
        .as_ref()
        .map(|e| e.replacements.len())
        .unwrap_or(0);
    let baseline_examples = draft
        .baseline
        .as_ref()
        .map(|e| e.examples.len())
        .unwrap_or(0);
    let entry = &mut draft.candidate;
    ui.ww_collapsing("基本の説明・問題を確認／編集", |ui| {
        if append { ui.small("追加モードでは既存の基本項目は変更しない。語感・文法の補足は追加例文の説明に記入する。"); }
        ui.add_enabled_ui(!append, |ui| {
            for (label, text) in [
                ("基本語の意味", &mut entry.meaning), ("学習水準", &mut entry.level),
                ("社外メールの表現", &mut entry.business), ("格調の高い表現", &mut entry.elevated),
                ("語調", &mut entry.register), ("用法・使用条件", &mut entry.usage),
                ("場面", &mut entry.context), ("空欄問題（___を1個）", &mut entry.example),
                ("完成英文の訳", &mut entry.translation), ("用法の質問", &mut entry.question),
                ("質問の解説", &mut entry.explanation), ("分類", &mut entry.tag),
            ] {
                ui.label(label);
                ui.add(egui::TextEdit::multiline(text).desired_width(f32::INFINITY).desired_rows(2).char_limit(1000));
            }
            let mut answers = entry.answers.join("|");
            ui.label("空欄の正解（別解は | で区切る）");
            if ui.add(egui::TextEdit::singleline(&mut answers).desired_width(f32::INFINITY).char_limit(1000)).changed() {
                entry.answers = answers.split('|').map(|s| s.trim().to_string()).collect();
            }
        });
    });
    ui.ww_collapsing(
        format!("言い換えを確認／編集（{}件）", entry.replacements.len()),
        |ui| {
            let mut remove = None;
            for (i, r) in entry.replacements.iter_mut().enumerate() {
                ui.push_id(("material-replacement", i), |ui| {
                    ui.group(|ui| {
                        ui.add_enabled_ui(!append || i >= baseline_replacements, |ui| {
                            for (label, text) in [
                                ("語句", &mut r.phrase),
                                ("意味", &mut r.meaning),
                                ("使用条件・違い", &mut r.conditions),
                            ] {
                                ui.label(label);
                                ui.add(
                                    egui::TextEdit::multiline(text)
                                        .desired_rows(2)
                                        .desired_width(f32::INFINITY)
                                        .char_limit(1000),
                                );
                            }
                            if ui.ww_button("この言い換えを案から除く").clicked() {
                                remove = Some(i);
                            }
                        });
                    })
                });
            }
            if let Some(i) = remove {
                entry.replacements.remove(i);
            }
            if ui
                .add_enabled(
                    entry.replacements.len() < 30,
                    crate::app::controls::Button::new("言い換え欄を追加"),
                )
                .clicked()
            {
                entry.replacements.push(model::Replacement {
                    phrase: String::new(),
                    meaning: String::new(),
                    conditions: String::new(),
                });
            }
        },
    );
    ui.ww_collapsing(
        format!("完成例文を確認／編集（{}件）", entry.examples.len()),
        |ui| {
            let mut remove = None;
            for (i, e) in entry.examples.iter_mut().enumerate() {
                ui.push_id(("material-example", i), |ui| {
                    ui.group(|ui| {
                        ui.add_enabled_ui(!append || i >= baseline_examples, |ui| {
                            for (label, text) in [
                                ("英文", &mut e.english),
                                ("日本語訳", &mut e.japanese),
                                ("語感・文法・使い方の説明", &mut e.note),
                            ] {
                                ui.label(label);
                                ui.add(
                                    egui::TextEdit::multiline(text)
                                        .desired_rows(2)
                                        .desired_width(f32::INFINITY)
                                        .char_limit(1000),
                                );
                            }
                            if ui.ww_button("この例文を案から除く").clicked() {
                                remove = Some(i);
                            }
                        });
                    })
                });
            }
            if let Some(i) = remove {
                entry.examples.remove(i);
            }
            if ui
                .add_enabled(
                    entry.examples.len() < 200,
                    crate::app::controls::Button::new("例文欄を追加"),
                )
                .clicked()
            {
                entry.examples.push(model::Example {
                    english: String::new(),
                    japanese: String::new(),
                    note: String::new(),
                });
            }
        },
    );
}

fn read_limited(path: &std::path::Path, max: u64) -> Result<String, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > max {
        return Err("ファイルが大きすぎます。".into());
    }
    std::fs::read_to_string(path).map_err(|e| format!("UTF-8のファイルを読み取れません: {e}"))
}
