use super::*;

fn vocabulary_counts(deck: &[Entry], progress: &Progress) -> (usize, usize) {
    let mut bases = std::collections::BTreeMap::<String, bool>::new();
    for entry in deck.iter().filter(|e| !progress.deleted_entries.contains(&e.id)) {
        let answered = Skill::ALL.iter().any(|skill| progress.memories
            .get(&skill.key(&entry.id)).is_some_and(|m| m.reviews > 0));
        let learned = bases.entry(entry.base.trim().to_lowercase()).or_default();
        *learned |= answered;
    }
    (bases.len(), bases.values().filter(|&&learned| learned).count())
}

impl WordApp {
    pub(super) fn home_dashboard(&mut self, ui: &mut egui::Ui) {
        ux::heading(ui, "今日の学習", "少しずつ思い出す練習を重ねる。まずは自分に合う時間から。");
        let minutes = self.progress.settings.minutes;
        ux::panel(ui, true, |ui| {
            ui.label(format!("目安時間：約{minutes}分。回答の途中で時間になっても、その問題を終えてから区切る。"));
            ui.add_space(12.0);
            let title = if self.session.is_some() { "学習を再開する" } else { "今日の学習を始める" };
            if ux::primary(ui, title, self.pending.is_none() && self.recorder.is_none() && !self.batch_running).clicked() { self.start(minutes); }
            if self.session.is_some() { ui.small("入力中の回答と学習の進み具合は保持されている。"); }
        });
        ui.add_space(16.0);
        let date = today();
        let answers = self.progress.reviews.iter().filter(|r| r.date == date).count();
        let new_today = self.progress.reviews.iter().filter(|r| r.date == date && r.first).count();
        let seconds = self.progress.study_seconds.get(&date).copied().unwrap_or(0);
        let due = scheduler::make_queue(&self.deck, &self.progress, now(), &date, 0).len();
        ux::metric_group(ui, |ui| {
            ux::metric(ui, "今日の学習時間", ux::duration(seconds as u64), "休止・AI待機を除く保存済み時間");
            ux::metric(ui, "今日の回答", format!("{answers}回"), &format!("うち初回回答 {new_today}回"));
            ux::metric(ui, "今回の復習候補", format!("{due}項目"), "現在の出題設定・1回最大18項目");
        });
        ui.add_space(16.0);
        ux::panel(ui, false, |ui| {
            ui.heading("学習の積み重ね");
            self.vocabulary_dashboard(ui);
        });
        ui.add_space(16.0);
        ui.collapsing("学習のヒントと教材について", |ui| {
            ui.label("期限を過ぎた復習は、今後のセッションに分けて出題する。休んでも記録は失われない。");
            ui.label("新しい教材は、例を確認してから別の問題を挟んで思い出す。ヒントや直後の再現は独力の回答と区別する。");
            let count = self.deck.iter().filter(|e| !self.progress.deleted_entries.contains(&e.id)).count();
            ui.small(format!("教材{count}項目。中高水準から選んだ独自教材であり、全教科書の網羅リストではない。"));
            ui.small("復習時期はWordWeaveの規則で計算する。AIは任意の生成・添削に使用する。");
        });
    }
    pub(super) fn vocabulary_dashboard(&self, ui: &mut egui::Ui) {
        let (registered, learned) = vocabulary_counts(&self.deck, &self.progress);
        ui.add_space(15.0);
        ux::metric_group(ui, |ui| {
            for (label, count) in [("登録されている語彙", registered),
                ("学習済みの語彙", learned), ("未学習の語彙", registered - learned)] {
                ui.group(|ui| { ui.label(label); ui.heading(format!("{count}語")); });
            }
        });
        ui.small("語彙数は教材の基本語・表現の重複を除いた数（削除済みを除く）。");
        ui.small("学習済み＝現在の教材で1回以上回答した語彙。正解・習得完了を意味しない。");
        if registered > 0 {
            ui.add(egui::ProgressBar::new(learned as f32 / registered as f32)
                .text(format!("学習経験あり {learned} / {registered}語")));
        } else {
            ui.label("登録された教材がない。「語彙を追加」タブから追加できる。");
        }
    }

    pub(super) fn study_start(&mut self, ui: &mut egui::Ui) {
        ux::heading(ui, "学習を始める", "期限を迎えた復習を優先し、新しい表現を少しずつ学ぶ。");
        let minutes = self.progress.settings.minutes;
        ux::panel(ui, true, |ui| {
            ui.label(format!("まずは{minutes}分。時間になったら回答の区切りで終了でき、続けたければそのまま継続できる。"));
            ui.add_space(12.0);
            if ux::primary(ui, format!("{minutes}分の学習を始める"), true).clicked() { self.start(minutes); }
            ui.add_space(8.0);
            if ui.button("今日は2分だけ").clicked() { self.start(2); }
        });
        ui.add_space(16.0);
        let queue = self.study_queue(minutes);
        ux::metric_group(ui, |ui| {
            ux::metric(ui, "今回の復習候補", format!("{}項目", queue.iter().filter(|t| !t.introduce).count()), "期限を迎えた項目を優先");
            ux::metric(ui, "新しく学ぶ候補", format!("{}項目", queue.iter().filter(|t| t.introduce).count()), "設定した1日の上限内で出題");
        });
        ui.add_space(16.0);
        ui.heading("練習する内容");
        for skill in &self.progress.settings.skills { ui.label(format!("・{}", skill.label())); }
        if queue.is_empty() { ui.label("今取り組める問題はない。復習予定を待つか、設定の対象分野・教材を確認できる。"); }
        ui.small("学習時間・対象分野・技能は「設定」タブから変更できる。");
    }

    pub(super) fn version_dialog(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Window::new("バージョン情報")
            .open(&mut self.about_open).collapsible(false).resizable(false)
            .show(ctx, |ui| {
                ui.heading("WordWeave5");
                ui.label(format!("バージョン {}", env!("CARGO_PKG_VERSION")));
                ui.label("Windows向け英語語彙学習アプリ");
                ui.label(format!("ライセンス：{}", env!("CARGO_PKG_LICENSE")));
                ui.separator();
                ui.small("この番号はWordWeave5本体のバージョンである。");
                ui.small("Codex CLIのバージョンとは異なる。");
                close = ui.button("閉じる").clicked();
            });
        if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.about_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_counts_unique_bases_and_answered_not_mastered() {
        let mut deck = model::parse_deck(model::BUILTIN_DECK).unwrap();
        deck.truncate(1);
        let mut sibling = deck[0].clone();
        sibling.id = "sibling".into();
        deck.push(sibling);
        let mut other = deck[0].clone();
        other.id = "other".into();
        other.base = "another word".into();
        deck.push(other);
        let mut p = Progress::default();
        assert_eq!(vocabulary_counts(&deck, &p), (2, 0));
        p.memories.insert(Skill::ALL[0].key(&deck[0].id), scheduler::Memory::default());
        assert_eq!(vocabulary_counts(&deck, &p), (2, 0));
        let mut memory = scheduler::Memory::default();
        memory.reviews = 1;
        for skill in Skill::ALL {
            p.memories.insert(skill.key(&deck[0].id), memory.clone());
        }
        assert_eq!(vocabulary_counts(&deck, &p), (2, 1));
        p.suspended.insert(deck[0].id.clone());
        assert_eq!(vocabulary_counts(&deck, &p), (2, 1));
        p.deleted_entries.insert(deck[0].id.clone());
        assert_eq!(vocabulary_counts(&deck, &p), (2, 0));
        p.deleted_entries.insert("sibling".into());
        assert_eq!(vocabulary_counts(&deck, &p), (1, 0));
        assert_eq!(vocabulary_counts(&[], &p), (0, 0));
    }
}
