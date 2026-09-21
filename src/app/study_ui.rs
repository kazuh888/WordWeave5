use super::*;

impl WordApp {
    pub(super) fn study_answer_input(&mut self, ui: &mut egui::Ui, skill: Skill) -> bool {
        let id = egui::Id::new("study-answer-input");
        let multiline = matches!(skill, Skill::Usage | Skill::Sentence);
        let enabled = ui.is_enabled() && self.fatal.is_none()
            && self.pending.is_none() && self.recorder.is_none()
            && !self.session.as_ref().is_some_and(|s| s.paused);
        if enabled && self.study_focus_pending && self.input == Input::Keyboard {
            ui.memory_mut(|m| m.request_focus(id));
            self.study_focus_pending = false;
        }
        let focused = enabled && ui.memory(|m| m.has_focus(id));
        let ime_id = id.with("ime");
        let mut composing = ui.ctx().data_mut(|d| d.get_temp::<bool>(ime_id).unwrap_or(false));
        let check = ui.input_mut(|i| study_keys(&mut i.events, focused, multiline, &mut composing));
        ui.ctx().data_mut(|d| d.insert_temp(ime_id, composing));
        ui.add(egui::TextEdit::multiline(&mut self.answer)
            .id(id)
            .desired_rows(if multiline { 3 } else { 2 })
            .desired_width(f32::INFINITY)
            .hint_text("回答を入力 / 認識結果を確認・修正"));
        ui.small(if multiline {
            "Enterで改行 / Ctrl+Enterで照合。照合後に自分で評価して記録する。"
        } else {
            "Enterで照合。照合後に自分で評価して記録する。"
        });
        check
    }

    pub(super) fn check_study_answer(&mut self, entry: &Entry, skill: Skill) {
        if self.fatal.is_some() || self.pending.is_some() || self.recorder.is_some()
            || self.session.as_ref().is_some_and(|s| s.paused)
        {
            return;
        }
        self.attempted = !self.answer.trim().is_empty()
            || (self.input == Input::Pen && !self.ink.empty())
            || (self.input == Input::Voice && self.wav.is_some());
        self.matched = if matches!(skill, Skill::Recall | Skill::Listening)
            && !self.answer.trim().is_empty()
        {
            Some(entry.accepts(&self.answer))
        } else {
            None
        };
        self.revealed = true;
        self.study_focus_pending = true;
    }

    pub(super) fn study_feedback(&mut self, ui: &mut egui::Ui, entry: &Entry, skill: Skill) {
        ux::panel(ui, true, |ui| {
            ui.strong("回答を確認");
            match self.matched {
                Some(true) => { ui.colored_label(Color32::from_rgb(25, 115, 70), "登録されている解答と一致した。"); }
                Some(false) => { ui.colored_label(Color32::from_rgb(155, 92, 25), "登録例とは異なる。自然な別解かどうか、意味・文法・場面を確認する。"); }
                None => { ui.label("解答例と比べて、自分の回答を評価する。"); }
            }
            if !self.answer.trim().is_empty() { ui.label(format!("自分の回答：{}", self.answer)); }
            if self.input == Input::Pen { ui.add_enabled_ui(false, |ui| self.ink.ui(ui)); }
            ui.separator();
            if let Some(problem) = &self.exercise {
                ui.label(RichText::new(format!("解答例：{}", problem.reference)).size(22.0));
            } else if skill == Skill::Usage {
                ui.label(&entry.explanation);
            } else {
                ui.label(RichText::new(entry.completed()).size(22.0));
                ui.label(&entry.translation);
            }
            ui.small("解答例は一例。自然な別解もあり得る。照合だけでは学習記録を変更しない。");
        });
        ui.add_space(8.0);
        let guidance = ui.add(egui::Label::new("自分で評価して次へ進む（Tabで評価を選び、Enterで確定）")
            .sense(egui::Sense::focusable_noninteractive()));
        if self.study_focus_pending {
            guidance.request_focus();
            self.study_focus_pending = false;
        }
        ui.add_enabled_ui(self.pending.is_none(), |ui| {
            ui.horizontal_wrapped(|ui| {
                for (grade, label, enabled) in [
                    (Grade::Again, "思い出せなかった → 次へ", true),
                    (Grade::Hard, "曖昧 / ヒントあり → 次へ", self.attempted),
                    (Grade::Good, "自力でできた → 次へ", self.attempted && self.hints == 0),
                    (Grade::Easy, "すぐ正確にできた → 次へ", self.attempted && self.hints == 0),
                ] {
                    if ui.add_enabled(enabled, egui::Button::new(label).wrap()).clicked() {
                        self.grade(grade);
                        break;
                    }
                }
            });
        });
        if !self.revealed { return; }
        ui.small("音声・手書きの認識ミスは、元の回答で評価する。学習直後45秒未満の再現は復習間隔を控えめに設定する。");
        egui::CollapsingHeader::new("詳しい解説・例文を確認")
            .id_salt(("study-details", self.key()))
            .show(ui, |ui| {
                self.card(ui, entry);
                if matches!(skill, Skill::Usage | Skill::Sentence) { ui.label(&entry.explanation); }
            });
        egui::CollapsingHeader::new("AIに使い方を確認する（任意）")
            .id_salt(("study-ai", self.key()))
            .show(ui, |ui| {
                ui.small("Codexへ回答を送信する。AIのコメントは学習成績を自動変更しない。");
                if ui.add_enabled(self.pending.is_none() && !self.answer.trim().is_empty(),
                    egui::Button::new("回答について確認する（Codexへ送信）").wrap()).clicked()
                {
                    self.launch_ai(0);
                }
                if !self.feedback.is_empty() { ui.label(&self.feedback); }
                if skill == Skill::Sentence && self.pending.is_none() && !self.feedback.is_empty()
                    && ui.add(egui::Button::new("添削を踏まえて書き直す（ヒントありとして評価）").wrap()).clicked()
                {
                    self.revealed = false;
                    self.study_focus_pending = true;
                    self.feedback.clear();
                    self.matched = None;
                    self.revision = true;
                    self.hints = self.hints.max(1);
                }
            });
    }
}

// Keep IME events intact; only withhold confirmation/shortcut Enter from TextEdit.
fn study_keys(events: &mut Vec<egui::Event>, focused: bool, multiline: bool, composing: &mut bool) -> bool {
    if !focused { *composing = false; return false; }
    let mut ime_frame = *composing;
    for event in events.iter() {
        if let egui::Event::Ime(event) = event {
            ime_frame = true;
            *composing = matches!(event, egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_));
        }
    }
    let check = !ime_frame && events.iter().any(|event| matches!(event,
        egui::Event::Key { key: egui::Key::Enter, pressed: true, repeat: false, modifiers, .. }
            if (!multiline || modifiers.ctrl) && !modifiers.shift && !modifiers.alt && !modifiers.mac_cmd));
    events.retain(|event| !matches!(event,
        egui::Event::Key { key: egui::Key::Enter, modifiers, .. }
            if ime_frame || !multiline || modifiers.ctrl));
    check
}
