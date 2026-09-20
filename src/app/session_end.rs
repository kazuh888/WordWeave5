use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum EndReason { Time, Manual, NoTasks }

pub(super) struct SessionSummary {
    pub reason: EndReason,
    pub seconds: u64,
    pub answers: usize,
    pub independent: usize,
    pub reviewed: usize,
    pub expressions: Vec<Entry>,
    pub next_due: Option<i64>,
}

impl SessionSummary {
    pub(super) fn collect(session: &Session, progress: &Progress, deck: &[Entry], reason: EndReason) -> Self {
        let records = &progress.reviews[session.review_start.min(progress.reviews.len())..];
        let keys: std::collections::BTreeSet<_> = records.iter().map(|r| r.key.as_str()).collect();
        Self {
            reason,
            seconds: session.elapsed.as_secs(),
            answers: records.len(),
            independent: records.iter().filter(|r| !r.assisted && matches!(r.grade, Grade::Good | Grade::Easy)).count(),
            reviewed: records.iter().filter(|r| !r.first).map(|r| &r.key).collect::<std::collections::BTreeSet<_>>().len(),
            expressions: deck.iter().filter(|e| Skill::ALL.iter().any(|s| keys.contains(s.key(&e.id).as_str()))).cloned().collect(),
            next_due: keys.iter().filter_map(|key| progress.memories.get(*key).map(|m| m.due)).min(),
        }
    }
}

impl WordApp {
    pub(super) fn session_result(&mut self, ui: &mut egui::Ui) {
        let Some(summary) = self.session_summary.as_ref() else { return; };
        ux::heading(ui, "今回の学習", match summary.reason {
            EndReason::Time => "設定した時間に達したため、回答の区切りで終了した。",
            EndReason::Manual => "ここで一区切り。記録した回答は保存されている。",
            EndReason::NoTasks => "今取り組める問題は終了した。復習期限と新規項目の上限を守って出題する。",
        });
        ux::panel(ui, true, |ui| {
            ux::metric_group(ui, |ui| {
                ux::metric(ui, "学習時間", ux::duration(summary.seconds), "休止・AI待機の時間を除く");
                ux::metric(ui, "記録した回答", format!("{}回", summary.answers), "同じ問題への再回答を含む");
                ux::metric(ui, "復習した練習項目", format!("{}項目", summary.reviewed), "初回の回答を除く・重複なし");
            });
        });
        ui.add_space(16.0);
        ux::panel(ui, false, |ui| {
            ui.heading("今回できたこと");
            if summary.answers == 0 {
                ui.label("まだ回答は記録していない。教材を確認した時間は学習時間に残る。");
            } else {
                ui.label(format!("{}件の教材に取り組んだ。", summary.expressions.len()));
                ui.label(format!("ヒントなしでできたと記録：{}回（自己評価を含む）。", summary.independent));
            }
            if let Some(due) = summary.next_due.and_then(|at| chrono::DateTime::from_timestamp(at, 0)) {
                ui.small(format!("今回の項目で最も近い復習予定：{}", due.with_timezone(&chrono::Local).format("%m/%d %H:%M")));
            }
        });
        ui.add_space(16.0);
        if !summary.expressions.is_empty() {
            ui.collapsing("今回取り組んだ表現を振り返る", |ui| {
                for entry in &summary.expressions {
                    ui.strong(format!("{} — {}", entry.base, entry.meaning));
                    ui.label(entry.completed());
                    ui.small(&entry.translation);
                    ui.separator();
                }
            });
        }
        ui.add_space(16.0);
        let can_continue = !self.study_queue(self.progress.settings.minutes).is_empty();
        ui.horizontal_wrapped(|ui| {
            if ui.add(egui::Button::new("今日はここで終了").min_size(egui::vec2(220.0, 48.0))).clicked() { self.page = Page::Home; }
            if ux::primary(ui, "もう少し学習する", can_continue).clicked() { self.start(self.progress.settings.minutes); }
        });
        if !can_continue { ui.label("今出題できる対象はない。次の復習を待つか、教材を読む・英語チャットで質問することができる。"); }
        ui.small("続けなくても記録は失われない。自分に合った区切りで終了できる。");
    }

    pub(super) fn request_study_end(&mut self, ui: &mut egui::Ui) {
        if !self.study_end_confirm { return; }
        ux::panel(ui, true, |ui| {
            ui.strong("この問題の回答はまだ記録していない");
            ui.label("ここで終了すると、この問題の入力・筆跡・録音は学習記録に登録されない。回答済みの記録は保持する。");
            ui.horizontal_wrapped(|ui| {
                if ui.button("問題に戻る").clicked() { self.study_end_confirm = false; }
                if ui.button("未記録の回答を破棄して終了").clicked() {
                    self.study_end_confirm = false;
                    self.finish();
                }
            });
        });
    }
}
