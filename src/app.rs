use crate::{
    ai,
    ink::Ink,
    media::{self, Recorder, Speaker},
};
use eframe::egui::{self, Color32, RichText};
mod chat_ui;
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
    Chat { id: String, question: String, reply: wordweave5::chat_action::ChatReply },
    Material(wordweave5::material::Draft),
    Played,
}
struct Pending {
    key: String,
    rx: Receiver<Result<AiResult, String>>,
    cancel: Option<Arc<AtomicBool>>,
}

pub struct WordApp {
    storage: Option<Storage>,
    fatal: Option<String>,
    progress: Progress,
    deck: Vec<Entry>,
    page: Page,
    session: Option<Session>,
    current: Option<Task>,
    answer: String,
    revealed: bool,
    matched: Option<bool>,
    hints: usize,
    input: Input,
    ink: Ink,
    speaker: Speaker,
    recorder: Option<Recorder>,
    wav: Option<Vec<u8>>,
    pending: Option<Pending>,
    feedback: String,
    message: String,
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
    chat_composer_height: f32,
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
        let mut font_notice = String::new();
        let mut fonts = egui::FontDefinitions::default();
        let windows = std::env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Windows"));
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
            cc.egui_ctx.set_fonts(fonts);
        } else {
            font_notice =
                "日本語フォントが見つかりません。Windowsの日本語フォントを追加してください。"
                    .into();
        }
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(17.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
        style.visuals.selection.bg_fill = Color32::from_rgb(33, 113, 117);
        cc.egui_ctx.set_style(style);
        let mut fatal = None;
        let storage = match Storage::open() {
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
        cc.egui_ctx.set_zoom_factor(progress.settings.font_scale);
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
        let chat_selected = wordweave5::chat::ordered_indices(&progress.chats).first().copied().unwrap_or(0);
        Self {
            storage,
            fatal,
            progress,
            deck,
            page: Page::Home,
            session: None,
            current: None,
            answer: String::new(),
            revealed: false,
            matched: None,
            hints: 0,
            input: Input::Keyboard,
            ink: Ink::default(),
            speaker: Speaker::new(),
            recorder: None,
            wav: None,
            pending: None,
            feedback: String::new(),
            message: String::new(),
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
            chat_composer_height: 165.0,
        }
    }
    fn persist(&mut self) {
        if self.fatal.is_some() {
            return;
        }
        if let Some(storage) = &self.storage {
            if let Err(e) = storage.save(&self.progress) {
                self.fatal=Some(format!("保存に失敗しました。これ以上の学習記録は変更しません。アプリを閉じる前に、設定画面から現在の記録をエクスポートしてください。原因: {e}"));
            } else {
                self.dirty = false;
                self.last_save = Instant::now();
            }
        }
    }
    fn reset_answer(&mut self) {
        self.answer.clear();
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
        self.speaker.stop();
    }
    fn key(&self) -> String {
        self.current
            .as_ref()
            .map(|t| t.key(&self.deck))
            .unwrap_or_default()
    }
    fn start(&mut self, minutes: u32) {
        self.message.clear();
        let max_new = if minutes <= 2 {
            1
        } else {
            self.progress.settings.new_per_day
        };
        let queue = scheduler::make_queue(&self.deck, &self.progress, now(), &today(), max_new);
        self.session = Some(Session {
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
            self.finish();
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
            self.finish();
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
        self.credit_time();
        if let Some(s) = self.session.take() {
            self.message = format!(
                "今日はここまで。{}項目に回答、学習時間は{}分{}秒。",
                s.completed,
                s.elapsed.as_secs() / 60,
                s.elapsed.as_secs() % 60
            );
        }
        self.current = None;
        self.speaker.stop();
        self.page = Page::Home;
        self.persist();
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
            .is_some_and(|r| r.started.elapsed() >= Duration::from_secs(30))
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
                    if pending.key == self.key() {
                        self.message.clear();
                        match r {
                            Ok(AiResult::Text(t)) => {
                                self.answer = t;
                                self.message =
                                    "認識結果を確認し、誤認識があれば直してから回答してください。"
                                        .into();
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
                            Ok(AiResult::Connection(t)) => self.message = t,
                            Ok(AiResult::Material(draft)) => {
                                self.chat_material_open = true;
                                self.progress.material_draft = Some(draft);
                                self.material_same_base = false;
                                self.dirty = true;
                                self.persist();
                                self.message = "教材案を作成した。「教材案を確認」で差分を確認・編集して登録してください。".into();
                            }
                            Ok(AiResult::Chat { id, question, reply }) => {
                                    match self.progress.complete_chat(&id, question, reply) {
                                        Ok(()) => {
                                            self.chat_target.clear();
                                            self.dirty = true;
                                            self.persist();
                                            self.message = "回答を会話履歴に保存した。続けて質問できる。".into();
                                        }
                                        Err(e) => { self.message = e; }
                                    }
                            }
                            Ok(AiResult::Generated(e)) => {
                                if let Err(err) = self.import_deck(vec![e]) {
                                    self.batch_running = false;
                                    self.message = err;
                                } else {
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
                                self.message = e;
                            }
                        }
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.batch_running = false;
                    self.fetch_then_generate = false;
                    self.message = "処理が終了しましたが結果を取得できませんでした。".into();
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
            rx,
            cancel: Some(cancel),
        });
        self.message = "AIに送信中…（待ち時間は学習タイマーに含めない）".into();
    }
    fn stop_recording(&mut self) {
        if let Some(r) = self.recorder.take() {
            match r.finish() {
                Ok(w) => {
                    self.wav = Some(w);
                    self.message = "録音した（次の問題に進むまでメモリ内で保持）。".into();
                }
                Err(e) => self.message = e,
            }
        }
    }
    fn say(&mut self, text: &str) {
        if let Err(e) = self.speaker.say(
            text,
            &self.progress.settings.voice_id,
            self.progress.settings.slow_speech,
        ) {
            self.message = e;
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
        egui::Grid::new("entry-details")
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, |ui| {
                ui.label("社外メール");
                ui.label(shown(&entry.business));
                ui.end_row();
                ui.label("格調・文体");
                ui.label(shown(&entry.elevated));
                ui.end_row();
                ui.label("語調・意味");
                ui.label(&entry.register);
                ui.end_row();
            });
        ui.add_space(7.0);
        ui.label(&entry.usage);
        ui.separator();
        ui.label(RichText::new(entry.completed()).size(22.0));
        ui.label(&entry.translation);
        if !entry.replacements.is_empty() {
            ui.collapsing("言い換えと使える条件", |ui| {
                for r in &entry.replacements {
                    ui.strong(&r.phrase);
                    ui.label(&r.meaning);
                    ui.label(&r.conditions);
                    ui.separator();
                }
            });
        }
        if !entry.examples.is_empty() {
            ui.collapsing(
                format!("語感を掴む例文（{}件）", entry.examples.len()),
                |ui| {
                    for (i, x) in entry.examples.iter().enumerate() {
                        ui.strong(format!("{}. {}", i + 1, x.english));
                        ui.label(&x.japanese);
                        ui.label(&x.note);
                        if ui
                            .small_button(format!("例文{}を読み上げ", i + 1))
                            .clicked()
                        {
                            self.say(&x.english);
                        }
                        ui.separator();
                    }
                },
            );
        }

        if ui.button("例文を聞く").clicked() {
            self.say(&entry.completed());
        }
    }
    fn home(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.heading("5分で、使える表現を少しずつ。");
        ui.label("基本語から、自然な社外メールと表現の違いを学ぶ。");
        ui.add_space(15.0);
        let due = scheduler::make_queue(&self.deck, &self.progress, now(), &today(), 0).len();
        let new_today = self
            .progress
            .reviews
            .iter()
            .filter(|r| r.date == today() && r.first)
            .count();
        let sec = self
            .progress
            .study_seconds
            .get(&today())
            .copied()
            .unwrap_or(0);
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("今日の学習");
                ui.heading(format!("{}分 {}秒", sec / 60, sec % 60));
            });
            ui.group(|ui| {
                ui.label("今回の復習候補");
                ui.heading(format!("{due}項目"));
            });
            ui.group(|ui| {
                ui.label("今日の新規回答");
                ui.heading(format!("{new_today}項目"));
            });
        });
        ui.add_space(18.0);
        if self.session.is_some() {
            if ui.button("学習の続きから").clicked() {
                self.page = Page::Study;
                if let Some(s) = self.session.as_mut() {
                    s.paused = false;
                }
            }
            if ui.button("現在のセッションを終了").clicked() {
                self.finish();
            }
        } else {
            let minutes = self.progress.settings.minutes;
            if ui
                .add_sized(
                    [230.0, 50.0],
                    egui::Button::new(format!("{minutes}分の学習を始める")),
                )
                .clicked()
            {
                self.start(minutes);
            }
            if ui.button("今日は2分だけ").clicked() {
                self.start(2);
            }
        }
        ui.add_space(15.0);
        ui.label(
            "期限を過ぎた復習は、今後のセッションに分けて出題する。休んでも学習記録は失われない。",
        );
        ui.label("新しいカードは、例を確認してから別の問題を挟んで思い出す。ヒントや直後の再現は、独力の正解と区別する。");
        ui.add_space(10.0);
        let base_count = self
            .deck
            .iter()
            .filter(|e| learning::kind(e) == "単語")
            .map(|e| e.base.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        ui.small(format!("教材 {}項目 / 基本語 {}語。中高水準から選んだ独自教材であり、全教科書の網羅リストではない。",self.deck.len(),base_count));
        ui.small(format!(
            "熟語 {}項目 / 構文 {}項目。設定の対象分野から選んで練習できる。",
            self.deck
                .iter()
                .filter(|e| learning::kind(e) == "熟語")
                .count(),
            self.deck
                .iter()
                .filter(|e| learning::kind(e) == "構文")
                .count()
        ));
        ui.small("既定の出題は「語句・綴り」「使い分け」。聞き取り・作文は設定から追加できる。");
    }
    fn study(&mut self, ui: &mut egui::Ui) {
        let Some(task) = self.current.clone() else {
            ui.label("ホームから学習を始めてください。");
            return;
        };
        let entry = self.deck[task.index].clone();
        let (elapsed, budget, paused) = self
            .session
            .as_ref()
            .map(|s| (s.elapsed.as_secs(), s.budget.as_secs(), s.paused))
            .unwrap_or((0, 300, false));
        ui.horizontal(|ui| {
            ui.heading(task.skill.label());
            let remaining = budget.saturating_sub(elapsed);
            ui.label(format!("残り {}:{:02}", remaining / 60, remaining % 60));
            if ui
                .button(if paused { "再開" } else { "一時停止" })
                .clicked()
            {
                if let Some(s) = self.session.as_mut() {
                    s.paused = !paused;
                }
            }
            if ui
                .add_enabled(
                    self.pending.is_none() && self.recorder.is_none(),
                    egui::Button::new("ここで終える"),
                )
                .clicked()
            {
                self.finish();
            }
        });
        if self.current.is_none() {
            return;
        }
        ui.add(
            egui::ProgressBar::new((elapsed as f32 / budget.max(1) as f32).min(1.0))
                .show_percentage(),
        );
        if paused {
            ui.label("休憩中。再開するまで学習時間は増えない。");
            return;
        }
        if elapsed >= budget {
            ui.colored_label(
                Color32::from_rgb(135, 90, 15),
                "目標時間に到達。この問題を終えたら終了する。",
            );
        }
        ui.separator();
        if task.introduce {
            ui.label("はじめての項目：意味と使用条件を確認する。");
            self.card(ui, &entry);
            ui.add_space(12.0);
            if ui.button("確認した・別の問題のあとで思い出す").clicked() {
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
        match task.skill {
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
                        egui::Button::new("語句を聞く / もう一度"),
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
                            egui::Button::new("新しい日本語の問題を生成（Codexへ送信）"),
                        )
                        .clicked()
                    {
                        self.launch_ai(3);
                    }
                }
                ui.small("意味の一致・文法・自然さと、対象表現を使えたかを分けて確認する。");
            }
        }
        if !self.revealed {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(self.recorder.is_none() && self.pending.is_none(), |ui| {
                    ui.selectable_value(&mut self.input, Input::Keyboard, "キーボード");
                    ui.selectable_value(&mut self.input, Input::Pen, "手書き");
                    ui.selectable_value(&mut self.input, Input::Voice, "音声");
                });
            });
            ui.add_enabled_ui(self.pending.is_none(), |ui| {
                if self.input == Input::Pen {
                    self.ink.ui(ui);
                }
                if self.input == Input::Voice {
                    if let Some(r) = self.recorder.as_ref() {
                        ui.label(format!(
                            "録音中 {}秒 / 最大30秒",
                            r.started.elapsed().as_secs()
                        ));
                        ui.add(egui::ProgressBar::new(r.level()).text("入力音量"));
                        if ui.button("録音を止める").clicked() {
                            self.stop_recording();
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button("録音する（英語）").clicked() {
                                self.speaker.stop();
                                match Recorder::start() {
                                    Ok(r) => {
                                        self.wav = None;
                                        self.recorder = Some(r);
                                    }
                                    Err(e) => self.message = e,
                                }
                            }
                            if ui
                                .add_enabled(
                                    self.wav.is_some(),
                                    egui::Button::new("自分の声を聞く"),
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
                                        rx,
                                        cancel: None,
                                    });
                                }
                            }
                        });
                    }
                }
                ui.add(
                    egui::TextEdit::multiline(&mut self.answer)
                        .desired_rows(if matches!(task.skill, Skill::Usage | Skill::Sentence) {
                            3
                        } else {
                            2
                        })
                        .desired_width(f32::INFINITY)
                        .hint_text("回答 / 認識結果の修正欄"),
                );
                ui.horizontal_wrapped(|ui| {
                    if matches!(task.skill, Skill::Recall | Skill::Listening)
                        && ui.button("文字のヒント").clicked()
                    {
                        self.hints = (self.hints + 1).min(entry.answer().chars().count());
                    }
                    if self.hints > 0 && matches!(task.skill, Skill::Recall | Skill::Listening) {
                        ui.label(format!("ヒント：{}", entry.hint(self.hints)));
                    }
                });
                ui.add_enabled_ui(self.recorder.is_none(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if self.input == Input::Pen
                            && ui
                                .button("手書き英語をAIで文字起こし（Codexへ送信）")
                                .clicked()
                        {
                            self.launch_ai(1);
                        }
                        if self.input == Input::Voice
                            && ui.button("録音をAIで文字起こし（Codexへ送信）").clicked()
                        {
                            self.launch_ai(2);
                        }
                        if ui.button("回答を照合 / わからないので確認").clicked() {
                            self.attempted = !self.answer.trim().is_empty()
                                || (self.input == Input::Pen && !self.ink.empty())
                                || (self.input == Input::Voice && self.wav.is_some());
                            self.matched = if matches!(task.skill, Skill::Recall | Skill::Listening)
                                && !self.answer.trim().is_empty()
                            {
                                Some(entry.accepts(&self.answer))
                            } else {
                                None
                            };
                            self.revealed = true;
                        }
                    });
                });
            });
        } else {
            match self.matched {
                Some(true) => {
                    ui.colored_label(
                        Color32::from_rgb(25, 115, 70),
                        "登録されている解答と一致した。",
                    );
                }
                Some(false) => {
                    ui.colored_label(
                        Color32::from_rgb(155, 92, 25),
                        "登録例とは異なる。別解の可能性を含め、意味・文法・場面を確認する。",
                    );
                }
                None => {
                    ui.label("自分の回答と、以下の解説を照合する。");
                }
            }
            if !self.answer.is_empty() {
                ui.label(format!("自分の回答：{}", self.answer));
            }
            if self.input == Input::Pen {
                ui.add_enabled_ui(false, |ui| self.ink.ui(ui));
            }
            if let Some(problem) = &self.exercise {
                ui.label(format!("模範例：{}", problem.reference));
                ui.small("模範例は一例。ほかの自然な英文も正解になり得る。");
            } else {
                self.card(ui, &entry);
            }
            if matches!(task.skill, Skill::Usage | Skill::Sentence) {
                ui.separator();
                ui.label(&entry.explanation);
            }
            if !self.feedback.is_empty() {
                ui.group(|ui| {
                    ui.label("AIの参考コメント（学習成績は自動変更しない）");
                    ui.label(&self.feedback);
                });
            }
            if ui
                .add_enabled(
                    self.pending.is_none() && !self.answer.trim().is_empty(),
                    egui::Button::new("AIに使い方を確認する（Codexへ送信）"),
                )
                .clicked()
            {
                self.launch_ai(0);
            }
            if task.skill == Skill::Sentence && self.pending.is_none() && !self.feedback.is_empty()
            {
                if ui
                    .button("添削を踏まえて書き直す（ヒントありとして評価）")
                    .clicked()
                {
                    self.revealed = false;
                    self.feedback.clear();
                    self.matched = None;
                    self.revision = true;
                    self.hints = self.hints.max(1);
                }
            }
            ui.separator();
            ui.small("結果を記録：音声・手書きの認識ミスは記憶の失敗として扱わず、自分の元の回答で評価する。");
            ui.add_enabled_ui(self.pending.is_none(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("思い出せなかった").clicked() {
                        self.grade(Grade::Again);
                    }
                    if ui
                        .add_enabled(self.attempted, egui::Button::new("曖昧 / ヒントあり"))
                        .clicked()
                    {
                        self.grade(Grade::Hard);
                    }
                    if ui
                        .add_enabled(
                            self.attempted && self.hints == 0,
                            egui::Button::new("自力でできた"),
                        )
                        .clicked()
                    {
                        self.grade(Grade::Good);
                    }
                    if ui
                        .add_enabled(
                            self.attempted && self.hints == 0,
                            egui::Button::new("すぐ正確にできた"),
                        )
                        .clicked()
                    {
                        self.grade(Grade::Easy);
                    }
                });
            });
            ui.small("学習直後45秒未満の再現は、復習間隔を控えめに設定する。");
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
        if self.pending.is_some() || self.session.is_some() || self.fatal.is_some() {
            return;
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
            cancel: Some(cancel),
        });
        self.message = "Codexで処理中…".into();
    }
    fn launch_chat(&mut self) {
        if self.progress.chat_action.is_some() {
            self.message = "確認中の教材操作を確定またはキャンセルしてから送信してください。".into();
            return;
        }
        if self.pending.is_some() || self.session.is_some() || self.batch_running || self.fatal.is_some() { return; }
        let Some(chat) = self.progress.chats.get(self.chat_selected) else { return; };
        if chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES {
            self.message = "この会話は200往復に達した。引き継ぎメモをコピーして新しい会話を作成してください。".into();
            return;
        }
        let context = match wordweave5::chat::prepare_with_catalog(chat, &self.deck, &self.progress.deleted_entries) {
            Ok(c) => c, Err(e) => { self.message = e; return; }
        };
        let id = chat.id.clone();
        let question = chat.draft.trim().to_string();
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c, Err(e) => { self.message = e; return; }
        };
        // This also saves the draft before sending, so retries and restarts
        // preserve it even when the child fails or the app is closed.
        if !self.reserve_generation() { return; }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = config.chat(context.payload).map(|reply| AiResult::Chat { id, question, reply });
            let _ = tx.send(result);
        });
        self.pending = Some(Pending { key: String::new(), rx, cancel: Some(cancel) });
        self.message = "Codexに質問中…".into();
    }
    fn launch_material(&mut self) {
        if self.pending.is_some() || self.session.is_some() || self.batch_running || self.recorder.is_some()
            || self.fatal.is_some() || self.progress.material_draft.is_some() { return; }
        let Some(chat) = self.progress.chats.get(self.chat_selected) else { return; };
        let baseline = if self.material_mode == wordweave5::material::Mode::New { None }
            else { self.deck.iter().find(|e| e.id == self.material_target
                && !self.progress.deleted_entries.contains(&e.id)).cloned() };
        let request = match wordweave5::material::Request::new(chat, &self.material_base, self.material_mode, baseline) {
            Ok(r) => r, Err(e) => { self.message = e; return; }
        };
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c, Err(e) => { self.message = e; return; }
        };
        if !self.reserve_generation() { return; }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || { let _ = tx.send(config.material(request).map(AiResult::Material)); });
        self.pending = Some(Pending { key:String::new(), rx, cancel:Some(cancel) });
        self.message = "選択したチャットから教材案を作成中…".into();
    }
    fn material_panel(&mut self, ui: &mut egui::Ui) {
        use wordweave5::material::Mode;
        ui.separator();
        ui.heading("チャットを教材に反映");
        let idle = self.pending.is_none() && self.session.is_none() && !self.batch_running && self.recorder.is_none();
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
                        ui.selectable_value(&mut self.material_mode, mode, mode.label());
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
                                ui.selectable_value(&mut self.material_target, e.id.clone(), format!("{} / {} / {} [{}]", e.base, e.meaning, e.context, e.id));
                            }
                        });
                    if matches.is_empty() { ui.label("同じ基本語の教材がない。新規登録を選択するか、対象語を確認してください。"); }
                } else if !matches.is_empty() {
                    ui.label(format!("同じ基本語が{}件ある。既存教材に加える場合は追加・訂正を選択する。", matches.len()));
                }
                let selected = self.progress.chats.get(self.chat_selected)
                    .map(|c| c.exchanges.iter().filter(|e| e.for_material).count()).unwrap_or(0);
                ui.label(format!("教材化の対象：{selected}往復"));
                if ui.add_enabled(selected > 0, egui::Button::new("教材案を作成")).clicked() { self.launch_material(); }
            });
            return;
        }
        let mut draft = self.progress.material_draft.clone().unwrap();
        ui.strong(format!("{}：{}", draft.mode.label(), draft.candidate.base));
        for notice in &draft.notices { ui.label(notice); }
        let source = draft.source.clone();
        if ui.button("元の会話を表示").clicked() {
            self.open_material_source(&source);
        }
        let before = model::deck_text(&[draft.candidate.clone()]);
        ui.add_enabled_ui(idle, |ui| edit_material(ui, &mut draft));
        let changed = before != model::deck_text(&[draft.candidate.clone()]);
        let ready = if draft.baseline.as_ref().is_some_and(|e| self.progress.deleted_entries.contains(&e.id)) {
            Err("対象教材は削除済みである。復元してから登録する。".into())
        } else { draft.ready(&self.deck, self.material_same_base) };
        ui.collapsing("登録される差分", |ui| {
            let current = serde_json::to_value(&draft.candidate).unwrap();
            let old = draft.baseline.as_ref().map(|e| serde_json::to_value(e).unwrap());
            for (key, label) in material_fields() {
                let new_value = &current[key];
                let old_value = old.as_ref().map(|v| &v[key]);
                if old_value == Some(new_value) { continue; }
                ui.strong(label);
                if let Some(value) = old_value { ui.label(format!("変更前：{}", material_value(value))); }
                ui.label(format!("変更後：{}", material_value(new_value)));
            }
        });
        if draft.mode == Mode::New && self.deck.iter().any(|e| model::normalize(&e.base) == model::normalize(&draft.candidate.base)) {
            ui.add_enabled(idle, egui::Checkbox::new(&mut self.material_same_base, "同じ基本語の別用法として新規登録する"));
        }
        ui.label(if draft.resets_learning() { "基本の説明・問題・正解等が変わるため、この教材の復習状態は再学習に戻る。" }
            else if draft.baseline.is_some() { "補足の例文・言い換えの変更であるため、既存の復習成績は維持する。" }
            else { "新しい教材として登録する。" });
        if let Err(e) = &ready { ui.colored_label(Color32::RED, e); }
        let mut commit = false;
        let mut discard = false;
        ui.add_enabled_ui(idle, |ui| ui.horizontal(|ui| {
            commit = ui.add_enabled(ready.is_ok(), egui::Button::new("内容を確認して教材に登録")).clicked();
            discard = ui.button("教材案を破棄").clicked();
        }));
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
                    Ok(()) => self.message = "チャットから教材を登録した。元の会話への参照も保存した。".into(),
                    Err(e) => {
                        // If the deck write succeeded but progress save failed,
                        // preserve matching source metadata for recovery/export.
                        if model::deck_text(&self.deck) == old_deck { self.progress = old_progress; }
                        self.message = e;
                    }
                }
            }
        }
    }
    fn open_material_source(&mut self, source: &wordweave5::material::Source) {
        if let Some(index) = self.progress.chats.iter().position(|c| c.id == source.conversation_id) {
            self.chat_selected = index;
            self.page = Page::Chat;
            self.message = format!("元の会話を表示した。参照したやり取り番号：{}", source.exchange_indices.iter().map(|i| (i+1).to_string()).collect::<Vec<_>>().join(", "));
        } else { self.message = "元の会話は現在の学習記録にありません。".into(); }
    }
    fn words_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("基本語から言い換え・例文を自動登録");
        ui.hyperlink_to("NGSL公式・出典", learning::NGSL_PAGE);
        ui.small("NGSL 1.2 / Browne, Culligan & Phillips / CC BY-SA 4.0。公式の頻度順リストと補足語を同梱。教材解説と例文はCodexが生成する。");
        ui.label(format!(
            "候補{}語 / 未処理{}語 / 本日の生成試行{}回",
            self.words.len(),
            self.batch_queue.len(),
            self.progress.ai_calls.get(&today()).copied().unwrap_or(0)
        ));
        let idle = self.session.is_none()
            && self.pending.is_none()
            && self.recorder.is_none()
            && !self.batch_running;
        if !idle {
            ui.label(
                "学習・通信の終了後に追加操作ができる。自動生成は画面下のボタンで中断できる。",
            );
        }
        ui.add_enabled_ui(idle,|ui|{
            ui.add(egui::Slider::new(&mut self.progress.settings.batch_words,1..=3000).logarithmic(true).text("1回に追加する語数"));
            ui.add(egui::Slider::new(&mut self.progress.settings.examples_per_word,3..=12).text("1語あたりの例文数"));
            ui.small("生成は1語ずつ保存する。生成済みの語は除外し、途中で止まった語は再開時に処理する。1日の上限は設定で変更できる。");
            ui.horizontal_wrapped(|ui|{
                if ui.button("NGSLから自動生成・登録").clicked(){
                    self.begin_batch(learning::parse_words(learning::BUNDLED_NGSL).unwrap_or_default());
                }
                if ui.button("公式NGSLを再取得して自動登録").clicked(){
                    self.fetch_then_generate=true;let(tx,rx)=mpsc::channel();
                    std::thread::spawn(move||{let _=tx.send(ai::download_words().map(AiResult::Words));});
                    self.pending=Some(Pending{key:String::new(),rx,cancel:None});self.message="NGSL公式CSVを取得中…".into();
                }
                if ui.add_enabled(!self.batch_queue.is_empty(),egui::Button::new("未処理の語から再開")).clicked(){self.batch_running=true;}
            });
            ui.separator();ui.label("提供する基本語・熟語（1行に1項目）");
            ui.add(egui::TextEdit::multiline(&mut self.provided_words).desired_rows(4).desired_width(f32::INFINITY));
            if ui.button("入力した語から自動生成・登録").clicked(){
                let text=self.provided_words.clone();
                match learning::parse_words(&text).and_then(|words|{self.cache_words(&text)?;Ok(words)}){Ok(words)=>self.begin_batch(words),Err(e)=>self.message=e}
            }
            if ui.button("CSV・TXTの語から自動生成・登録").clicked(){
                if let Some(path)=rfd::FileDialog::new().add_filter("語彙",&["csv","tsv","txt"]).pick_file(){
                    match read_limited(&path,2_000_000).and_then(|text|{let words=learning::parse_words(&text)?;self.cache_words(&text)?;Ok(words)}){Ok(words)=>self.begin_batch(words),Err(e)=>self.message=e}
                }
            }
            ui.small("UTF-8のCSV・TSV・TXT。先頭列が語、または順位・語の順。Lemma/Word/Headword列にも対応。");
            ui.separator();ui.add(egui::TextEdit::singleline(&mut self.word_search).hint_text("候補を検索"));
            let query=self.word_search.to_lowercase();
            let visible:Vec<String>=self.words.iter().filter(|w|w.contains(&query)).take(100).cloned().collect();
            egui::ScrollArea::vertical().id_salt("word-candidates").max_height(180.0).show(ui,|ui|{
                for word in visible {if ui.selectable_label(self.selected_word==word,&word).clicked(){self.selected_word=word;}}
            });
            if ui.add_enabled(!self.selected_word.is_empty(),egui::Button::new("選択した語を生成・登録")).clicked(){self.begin_batch(vec![self.selected_word.clone()]);}
            ui.small("AI生成の教材は形式検査後に自動保存される。内容は教材画面で確認・編集できる。");
        });
        if self.pending.is_some() {
            ui.spinner();
        }
    }
    fn deck_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("教材を調べる");
        ui.add(
            egui::TextEdit::singleline(&mut self.search)
                .hint_text("基本語・表現・日本語で検索")
                .desired_width(450.0),
        );
        let q = self.search.to_lowercase();
        let matches: Vec<usize> = self
            .deck
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if self.progress.deleted_entries.contains(&e.id) { return false; }
                format!(
                    "{} {} {} {} {}",
                    e.base, e.meaning, e.business, e.elevated, e.usage
                )
                .to_lowercase()
                .contains(&q)
            })
            .map(|(i, _)| i)
            .collect();
        ui.label(format!("{}項目", matches.len()));
        egui::ScrollArea::vertical()
            .id_salt("deck-list")
            .max_height(180.0)
            .show(ui, |ui| {
                for &index in &matches {
                    let e = &self.deck[index];
                    if ui
                        .selectable_label(
                            self.selected == index,
                            format!("{}   {}   [{}]", e.base, e.meaning, e.tag),
                        )
                        .clicked()
                    {
                        self.selected = index;
                    }
                }
            });
        if let Some(e) = self.deck.get(self.selected).filter(|e| !self.progress.deleted_entries.contains(&e.id)).cloned() {
            ui.separator();
            self.card(ui, &e);
            ui.label(&e.question);
            ui.label(&e.explanation);
            let sources: Vec<_> = self.progress.material_sources.iter().filter(|s| s.entry_id == e.id).cloned().collect();
            if !sources.is_empty() {
                ui.collapsing("教材の元になった会話", |ui| {
                    for (i, source) in sources.iter().enumerate() {
                        if ui.add_enabled(self.pending.is_none(), egui::Button::new(format!("{}：元の会話を開く（{}）", i+1, source.mode.label()))).clicked() {
                            self.open_material_source(source);
                        }
                    }
                });
            }
            let idle = self.pending.is_none() && self.session.is_none() && !self.batch_running;
            ui.add_enabled_ui(idle,|ui|{
                if ui.add_enabled(e.examples.len()+self.progress.settings.examples_per_word<=200,egui::Button::new("英文追加：新しい場面の例文を生成・登録")).clicked(){self.launch_content(1,Some(e.clone()));}
                ui.collapsing("日本文を登録して英文を生成",|ui|{
                    let draft=self.progress.japanese_drafts.entry(e.id.clone()).or_default();
                    if ui.add(egui::TextEdit::multiline(draft).desired_rows(3).desired_width(f32::INFINITY)).changed(){self.dirty=true;}
                    if ui.button("日本文を登録し、英文を生成・登録").clicked(){self.persist();self.launch_content(2,Some(e.clone()));}
                    ui.small("日本語原文を先に保存する。生成失敗時も原文は残り、再試行できる。");
                });
                ui.collapsing("言い換えを手動登録",|ui|{
                    ui.label("置き換えの語句");ui.text_edit_singleline(&mut self.replacement_phrase);
                    ui.label("日本語の意味");ui.text_edit_singleline(&mut self.replacement_meaning);
                    ui.label("使える条件・意味の違い");ui.text_edit_multiline(&mut self.replacement_conditions);
                    if ui.button("言い換えを追加").clicked(){let mut changed=e.clone();changed.replacements.push(model::Replacement{phrase:self.replacement_phrase.trim().into(),meaning:self.replacement_meaning.trim().into(),conditions:self.replacement_conditions.trim().into()});
                        match self.import_deck(vec![changed]){Ok(())=>{self.replacement_phrase.clear();self.replacement_meaning.clear();self.replacement_conditions.clear();},Err(err)=>self.message=err}
                    }
                });
                ui.collapsing("教材を編集",|ui|{
                    if ui.button("この教材を編集欄へ読み込む").clicked(){self.draft_text=model::deck_text(&[e.clone()]);}
                    if !self.draft_text.is_empty(){
                        ui.add(egui::TextEdit::multiline(&mut self.draft_text).desired_rows(7).desired_width(f32::INFINITY));
                        ui.small("TSV。末尾2列は言い換えと例文のJSON配列。設定からファイルへの書き出し・取り込みもできる。");
                        if ui.button("編集内容を保存").clicked(){match model::parse_deck(&self.draft_text){Ok(items)=>self.pending_import=Some(items),Err(err)=>self.message=err}}
                    }
                });
            });
            let mut suspended = self.progress.suspended.contains(&e.id);
            if ui
                .checkbox(&mut suspended, "この項目を学習対象から外す（記録は保持）")
                .changed()
            {
                if suspended {
                    self.progress.suspended.insert(e.id.clone());
                } else {
                    self.progress.suspended.remove(&e.id);
                }
                self.dirty = true;
                self.persist();
            }
        }
    }
    fn stats(&mut self, ui: &mut egui::Ui) {
        ui.heading("覚えた感覚と、後日の再現を分けて見る");
        ui.label(format!("記録した回答：{}回", self.progress.reviews.len()));
        egui::Grid::new("retention").striped(true).show(ui, |ui| {
            ui.strong("前回学習からの間隔");
            ui.strong("ヒントなしの自己評価・照合結果");
            ui.end_row();
            for days in [7.0, 30.0] {
                ui.label(format!("{days:.0}日以上"));
                ui.label(match self.progress.observed_retention(days) {
                    Some((ok, total)) => {
                        format!("{ok}/{total}回 ({:.0}%)", 100.0 * ok as f64 / total as f64)
                    }
                    None => "まだ記録がない".into(),
                });
                ui.end_row();
            }
        });
        ui.small("これは通常の復習記録であり、無作為抽出した能力テストではない。自己評価も含む。7日以上には30日以上も含まれる。");
        ui.separator();
        egui::Grid::new("by-skill").striped(true).show(ui, |ui| {
            ui.strong("練習の種類");
            ui.strong("回答回数");
            ui.strong("復習を始めた項目");
            ui.end_row();
            for skill in Skill::ALL {
                let suffix = format!(":{}", skill.code());
                ui.label(skill.label());
                ui.label(
                    self.progress
                        .reviews
                        .iter()
                        .filter(|r| r.key.ends_with(&suffix))
                        .count()
                        .to_string(),
                );
                ui.label(
                    self.progress
                        .memories
                        .keys()
                        .filter(|k| k.ends_with(&suffix))
                        .count()
                        .to_string(),
                );
                ui.end_row();
            }
        });
        ui.separator();
        ui.label("直近7日の学習時間");
        for day in (0..7).rev() {
            let date = (chrono::Local::now().date_naive() - chrono::Duration::days(day))
                .format("%Y-%m-%d")
                .to_string();
            let sec = self.progress.study_seconds.get(&date).copied().unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label(&date);
                ui.add(
                    egui::ProgressBar::new((sec as f32 / 300.0).min(1.0))
                        .desired_width(300.0)
                        .text(format!("{}分{}秒", sec / 60, sec % 60)),
                );
            });
        }
        ui.small("休んだ日は0分として表示する。連続記録が途切れても、習得履歴をリセットしない。");
        ui.separator();
        ui.label("入力方法別の回答記録");
        for (method, label) in [
            ("keyboard", "キーボード"),
            ("pen", "手書き"),
            ("voice", "音声"),
        ] {
            let records: Vec<_> = self
                .progress
                .reviews
                .iter()
                .filter(|r| r.method == method)
                .collect();
            let independent = records
                .iter()
                .filter(|r| !r.assisted && matches!(r.grade, Grade::Good | Grade::Easy))
                .count();
            ui.label(format!(
                "{label}：{}回 / ヒントなしでできた {}回",
                records.len(),
                independent
            ));
        }
        ui.small("方式ごとに問題や難しさが異なるため、この差だけで入力方法の効果は判定できない。");
        ui.separator();
        ui.heading("日本語出題の添削履歴（最新20件）");
        for log in self.progress.writing_logs.iter().rev().take(20) {
            ui.collapsing(
                format!(
                    "{} / {} / {}",
                    log.at,
                    log.problem.target,
                    if log.revision {
                        "書き直し"
                    } else {
                        "初回回答"
                    }
                ),
                |ui| {
                    ui.label(&log.problem.japanese);
                    ui.label(format!("回答：{}", log.answer));
                    ui.label(&log.feedback);
                },
            );
        }
    }
    fn settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("学習と入力の設定");
        let before = serde_json::to_string(&self.progress.settings).unwrap_or_default();
        ui.add(
            egui::Slider::new(&mut self.progress.settings.minutes, 1..=30).text("通常コース（分）"),
        );
        ui.add(
            egui::Slider::new(&mut self.progress.settings.new_per_day, 0..=20)
                .text("新規項目の1日上限"),
        );
        ui.small("1日5分では3項目を初期値とする。復習候補が6項目を超える日は新規を出さない。");
        ui.horizontal_wrapped(|ui| {
            for skill in Skill::ALL {
                let mut enabled = self.progress.settings.skills.contains(&skill);
                if ui.checkbox(&mut enabled, skill.label()).changed() {
                    if enabled {
                        self.progress.settings.skills.push(skill);
                    } else {
                        self.progress.settings.skills.retain(|s| *s != skill);
                    }
                }
            }
        });
        if self.progress.settings.skills.is_empty() {
            self.progress.settings.skills.push(Skill::Recall);
        }
        let mut tags: Vec<String> = self
            .deck
            .iter()
            .map(|e| e.tag.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        tags.insert(0, "すべて".into());
        egui::ComboBox::from_id_salt("topic")
            .selected_text(&self.progress.settings.topic)
            .show_ui(ui, |ui| {
                for tag in tags {
                    ui.selectable_value(&mut self.progress.settings.topic, tag.clone(), tag);
                }
            });
        ui.small("出題の設定は次のセッションから反映する。");
        if ui
            .add(
                egui::Slider::new(&mut self.progress.settings.font_scale, 0.8..=1.6)
                    .text("画面の拡大率"),
            )
            .changed()
        {
            ctx.set_zoom_factor(self.progress.settings.font_scale);
        }
        ui.separator();
        ui.heading("音声");
        let current_voice = self
            .speaker
            .voices
            .iter()
            .find(|v| v.0 == self.progress.settings.voice_id)
            .map(|v| v.1.clone())
            .unwrap_or_else(|| "英語の音声を自動選択".into());
        egui::ComboBox::from_id_salt("voice")
            .width(440.0)
            .selected_text(current_voice)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.progress.settings.voice_id,
                    String::new(),
                    "英語の音声を自動選択",
                );
                for (id, name) in &self.speaker.voices {
                    ui.selectable_value(&mut self.progress.settings.voice_id, id.clone(), name);
                }
            });
        ui.checkbox(
            &mut self.progress.settings.slow_speech,
            "少しゆっくり読み上げる",
        );
        if ui.button("音声を確認する").clicked() {
            self.say("We appreciate your assistance.");
        }
        ui.small(
            "読み上げはWindowsの音声合成。マイクはWindowsで設定した既定の入力デバイスを使う。",
        );
        ui.separator();
        ui.heading("AI接続：codex app-server");
        ui.label(
            "Codex CLIをインストールし、ターミナルで codex login を実行してChatGPTでログインする。",
        );
        ui.label(
            "APIキーは使用しない。生成はChatGPT契約の利用枠を使用する。上限到達時は停止する。",
        );
        ui.horizontal(|ui| {
            ui.label("実行ファイル");
            ui.text_edit_singleline(&mut self.progress.settings.codex_path);
            if ui.button("Codex実行ファイルを選択").clicked() {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Codex", &["exe", "cmd", "bat"])
                    .pick_file()
                {
                    self.progress.settings.codex_path = p.to_string_lossy().into();
                }
            }
        });
        ui.small("既定値codexでPATHとnpmのインストール先を探索する。コマンドの引数は入力しない。");
        ui.horizontal(|ui| {
            ui.label("モデル（空欄はCodexの既定値）");
            ui.text_edit_singleline(&mut self.progress.settings.codex_model);
        });
        if ui
            .add_enabled(
                self.pending.is_none(),
                egui::Button::new("接続・ChatGPT認証を確認"),
            )
            .clicked()
        {
            self.launch_content(0, None);
        }
        ui.add(
            egui::Slider::new(&mut self.progress.settings.ai_daily_limit, 0..=1000)
                .text("生成・添削の1日上限（試行回数）"),
        );
        ui.add(
            egui::Slider::new(&mut self.progress.settings.examples_per_word, 3..=12)
                .text("1回に生成する例文数"),
        );
        ui.small("手書き認識は画像対応モデルが必要。録音の自動文字起こしには音声入力対応モデルが必要。非対応時も録音・再生は利用できる。");
        ui.collapsing("Codex診断情報（コピー・保存・ChatGPTで相談）", |ui| {
            ui.label("接続確認または生成の直近1回を記録する。原文・認証情報は保存せず、stderrは分類のみ。未分類の原因を特定できない場合がある。");
            ui.label("未ログインなら、同じWindowsユーザーのCMDで codex login --device-auth を実行し、認証完了後に codex login status で確認する。");
            let mut report = wordweave5::diagnostics::report();
            egui::ScrollArea::vertical().id_salt("codex_diagnostics").max_height(240.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut report).desired_width(f32::INFINITY).interactive(false));
            });
            ui.small("共有前に表示内容を確認すること。コピーとブラウザー起動は別操作で、自動送信しない。");
            ui.horizontal_wrapped(|ui| {
                if ui.button("確認した診断情報をコピー").clicked() {
                    ui.ctx().copy_text(report.clone());
                }
                if ui.button("診断情報を保存").clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_file_name("wordweave-codex-diagnostics.txt").save_file() {
                        self.message = store::atomic_write(&path, report.as_bytes())
                            .map(|_| "診断情報を保存した。".into()).unwrap_or_else(|e| e);
                    }
                }
                ui.hyperlink_to("ChatGPTを開く（手動貼り付け）", "https://chatgpt.com/");
            });
        });
        ui.separator();
        ui.heading("教材・バックアップ");
        let idle = self.session.is_none() && self.pending.is_none() && self.recorder.is_none();
        ui.horizontal_wrapped(|ui| {
            if ui.button("教材をTSVに書き出す").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("wordweave-deck.tsv")
                    .add_filter("TSV", &["tsv"])
                    .save_file()
                {
                    self.message =
                        store::atomic_write(&path, model::deck_text(&self.deck).as_bytes())
                            .map(|_| "教材を書き出した。編集後は取り込みで反映できる。".into())
                            .unwrap_or_else(|e| e);
                }
            }
            if ui
                .add_enabled(
                    idle && self.fatal.is_none(),
                    egui::Button::new("教材TSVを取り込む"),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TSV", &["tsv"])
                    .pick_file()
                {
                    match read_limited(&path, 64_000_000).and_then(|t| model::parse_deck(&t)) {
                        Ok(d) => self.pending_import = Some(d),
                        Err(e) => self.message = e,
                    }
                }
            }
        });
        ui.small("同じIDは更新、新しいIDは追加。内容を変更した項目の復習状態は再学習から始める。学習中の取り込みはできない。");
        ui.horizontal_wrapped(|ui| {
            if ui.button("学習記録をエクスポート").clicked() {
                self.credit_time();
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("wordweave-progress.json")
                    .add_filter("JSON", &["json"])
                    .save_file()
                {
                    let result = serde_json::to_vec_pretty(&self.progress)
                        .map_err(|e| e.to_string())
                        .and_then(|bytes| store::atomic_write(&path, &bytes));
                    self.message = result
                        .map(|_| "学習記録を書き出した。教材は別途TSVで書き出してください。".into())
                        .unwrap_or_else(|e| e);
                }
            }
            if ui
                .add_enabled(
                    idle && self.storage.is_some(),
                    egui::Button::new("学習記録を復元"),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    match read_limited(&path, 100_000_000)
                        .and_then(|t| {
                            serde_json::from_str::<Progress>(&t).map_err(|e| e.to_string())
                        })
                        .and_then(|p| {
                            p.validate()?;
                            Ok(p)
                        }) {
                        Ok(p) => self.pending_restore = Some(p),
                        Err(e) => self.message = e,
                    }
                }
            }
        });
        if let Some(storage) = &self.storage {
            ui.small(format!("保存先：{}", storage.dir.display()));
        }
        ui.small("日ごとのバックアップは保存先のbackupsフォルダーに残る。復元前の記録も別ファイルに退避する。");
        ui.separator();
        ui.collapsing("学習方式と限界",|ui|{
            ui.label("間隔学習・想起練習・段階的ヒントを採用。復習間隔は透明な独自の計算規則であり、FSRSでも『科学的に最速と証明された方式』でもない。");
            ui.label("正解率・入力方式別の記録を確認しながら、学習量を調整する。自己評価を含むため、数値は能力の厳密な測定ではない。");
            ui.label("詳細な研究根拠・教材の選定基準は同梱のRESEARCH.mdを参照。");
        });
        if before != serde_json::to_string(&self.progress.settings).unwrap_or_default() {
            self.dirty = true;
        }
        if ui
            .add_enabled(self.fatal.is_none(), egui::Button::new("設定を保存"))
            .clicked()
        {
            self.persist();
            if self.fatal.is_none() {
                self.message = "設定を保存した。".into();
            }
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
        let path = storage.dir.join("custom.tsv");
        let backup = storage.dir.join(format!("custom-before-{}.tsv", today()));
        if path.exists() && !backup.exists() {
            std::fs::copy(&path, backup).map_err(|e| e.to_string())?;
        }
        store::atomic_write(&path, text.as_bytes())?;
        let changed = self.progress.reconcile_deck(&merged);
        self.deck = merged;
        self.dirty = true;
        self.persist();
        if let Some(e) = &self.fatal {
            return Err(e.clone());
        }
        self.message = format!("教材を取り込んだ。{changed}項目の復習状態を更新した。");
        Ok(())
    }
    fn restore(&mut self, mut progress: Progress) -> Result<(), String> {
        if self.session.is_some() || self.pending.is_some() || self.recorder.is_some() {
            return Err("学習・録音・通信を終了してから復元してください。".into());
        }
        let storage = self.storage.as_ref().ok_or("保存先がありません。")?;
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
                    ui.label(format!("追加：{added}項目 / 内容の変更：{updated}項目"));
                    ui.label("問題・解答・用法を変更した項目は再学習に戻す。追加例文・言い換えだけの変更では成績を保持する。以前の教材は退避する。");
                    ui.horizontal(|ui| {
                        apply = ui.button("取り込む").clicked();
                        cancel = ui.button("キャンセル").clicked();
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
                ui.label("現在の記録を退避し、選択した記録に戻す。現在の教材と内容が異なる項目は再学習にする。");
                ui.horizontal(|ui|{apply=ui.button("復元する").clicked();cancel=ui.button("キャンセル").clicked();});
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

impl eframe::App for WordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
        let confirming = self.pending_import.is_some() || self.pending_restore.is_some();
        egui::TopBottomPanel::top("navigation").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("WordWeave 5")
                        .strong()
                        .color(Color32::from_rgb(24, 103, 106))
                        .size(24.0),
                );
                let enabled = self.pending.is_none()
                    && self.recorder.is_none()
                    && !self.batch_running
                    && !confirming;
                ui.add_enabled_ui(enabled, |ui| {
                    for (page, title) in [
                        (Page::Home, "ホーム"),
                        (Page::Study, "学習"),
                        (Page::Deck, "教材"),
                        (Page::Words, "語彙を追加"),
                        (Page::Chat, "英語チャット"),
                        (Page::Stats, "記録"),
                        (Page::Settings, "設定"),
                    ] {
                        if ui.selectable_label(self.page == page, title).clicked() {
                            self.page = page;
                        }
                    }
                });
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            let execution = wordweave5::execution::snapshot();
            if let Some(settings) = execution.execution {
                let phase = if execution.active { "今回の実行設定" } else { "前回Codexが返した実行設定" };
                ui.small(format!("{phase}：{}", settings.label()));
            } else {
                ui.small(if execution.active { "Codexの実行設定を確認中…" } else { "モデル：未確認 / effort：未確認" });
            }
            if !self.message.is_empty() {
                ui.label(&self.message);
            }
            if !self.font_notice.is_empty() {
                ui.colored_label(Color32::RED, &self.font_notice);
            }
            if let Some(error) = &self.fatal {
                ui.colored_label(Color32::from_rgb(165, 45, 30), error);
            }
            if self.pending.is_some() || self.batch_running {
                if ui.button("生成・通信を中断").clicked() {
                    self.batch_running = false;
                    self.fetch_then_generate = false;
                    if let Some(c) = self.pending.as_ref().and_then(|p| p.cancel.as_ref()) {
                        c.store(true, Ordering::Relaxed);
                    }
                    self.message = "中断中… 登録済み教材と未処理の語は保持する。".into();
                }
            }
            ui.small("ローカル学習 / AI生成はCodexのChatGPT認証を使用");
        });
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.add_enabled_ui(!confirming,|ui|{
            if self.page == Page::Chat && self.fatal.is_none() {
                self.chat_page(ui);
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
        ctx.request_repaint_after(Duration::from_millis(200));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(p) = self.pending.as_ref() {
            if let Some(c) = p.cancel.as_ref() {
                c.store(true, Ordering::Relaxed);
                let _ = p.rx.recv_timeout(Duration::from_secs(2));
            }
        }
        self.recorder = None;
        self.speaker.stop();
        self.credit_time();
        if self.dirty {
            self.persist();
        }
    }
}

fn material_fields() -> [(&'static str, &'static str); 15] {
    [("meaning","基本語の意味"),("level","学習水準"),("business","社外メールの表現"),
     ("elevated","格調の高い表現"),("register","語調"),("usage","用法・使用条件"),
     ("context","場面"),("example","空欄問題"),("translation","完成英文の訳"),
     ("answers","正解"),("question","用法の質問"),("explanation","質問の解説"),
     ("tag","分類"),("replacements","言い換え"),("examples","完成例文")]
}
fn material_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => {
            if items.is_empty() { "なし".into() }
            else { items.iter().map(material_value).collect::<Vec<_>>().join("\n\n") }
        }
        serde_json::Value::Object(fields) => fields.iter().map(|(key, value)| {
            let label = match key.as_str() { "english"=>"英文", "japanese"=>"訳", "note"=>"説明", "phrase"=>"語句", "meaning"=>"意味", "conditions"=>"使用条件", _=>key.as_str() };
            format!("{label}：{}", material_value(value))
        }).collect::<Vec<_>>().join("\n"),
        _ => "なし".into(),
    }
}
fn edit_material(ui: &mut egui::Ui, draft: &mut wordweave5::material::Draft) {
    let append = draft.mode == wordweave5::material::Mode::Append;
    let baseline_replacements = draft.baseline.as_ref().map(|e| e.replacements.len()).unwrap_or(0);
    let baseline_examples = draft.baseline.as_ref().map(|e| e.examples.len()).unwrap_or(0);
    let entry = &mut draft.candidate;
    ui.collapsing("基本の説明・問題を確認／編集", |ui| {
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
    ui.collapsing(format!("言い換えを確認／編集（{}件）", entry.replacements.len()), |ui| {
        let mut remove = None;
        for (i, r) in entry.replacements.iter_mut().enumerate() {
            ui.push_id(("material-replacement", i), |ui| ui.group(|ui| {
                ui.add_enabled_ui(!append || i >= baseline_replacements, |ui| {
                    for (label, text) in [("語句",&mut r.phrase),("意味",&mut r.meaning),("使用条件・違い",&mut r.conditions)] {
                        ui.label(label);
                        ui.add(egui::TextEdit::multiline(text).desired_rows(2).desired_width(f32::INFINITY).char_limit(1000));
                    }
                    if ui.button("この言い換えを案から除く").clicked() { remove = Some(i); }
                });
            }));
        }
        if let Some(i) = remove { entry.replacements.remove(i); }
        if ui.add_enabled(entry.replacements.len() < 30, egui::Button::new("言い換え欄を追加")).clicked() {
            entry.replacements.push(model::Replacement { phrase:String::new(), meaning:String::new(), conditions:String::new() });
        }
    });
    ui.collapsing(format!("完成例文を確認／編集（{}件）", entry.examples.len()), |ui| {
        let mut remove = None;
        for (i, e) in entry.examples.iter_mut().enumerate() {
            ui.push_id(("material-example", i), |ui| ui.group(|ui| {
                ui.add_enabled_ui(!append || i >= baseline_examples, |ui| {
                    for (label, text) in [("英文",&mut e.english),("日本語訳",&mut e.japanese),("語感・文法・使い方の説明",&mut e.note)] {
                        ui.label(label);
                        ui.add(egui::TextEdit::multiline(text).desired_rows(2).desired_width(f32::INFINITY).char_limit(1000));
                    }
                    if ui.button("この例文を案から除く").clicked() { remove = Some(i); }
                });
            }));
        }
        if let Some(i) = remove { entry.examples.remove(i); }
        if ui.add_enabled(entry.examples.len() < 200, egui::Button::new("例文欄を追加")).clicked() {
            entry.examples.push(model::Example { english:String::new(), japanese:String::new(), note:String::new() });
        }
    });
}

fn read_limited(path: &std::path::Path, max: u64) -> Result<String, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > max {
        return Err("ファイルが大きすぎます。".into());
    }
    std::fs::read_to_string(path).map_err(|e| format!("UTF-8のファイルを読み取れません: {e}"))
}
