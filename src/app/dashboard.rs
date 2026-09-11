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
    pub(super) fn vocabulary_dashboard(&self, ui: &mut egui::Ui) {
        let (registered, learned) = vocabulary_counts(&self.deck, &self.progress);
        ui.add_space(15.0);
        ui.horizontal_wrapped(|ui| {
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
        ui.heading("学習を始める");
        ui.label("期限を迎えた復習を優先し、新しい表現を少しずつ学ぶ。");
        ui.add_space(15.0);
        let minutes = self.progress.settings.minutes;
        if ui.add_sized([230.0, 50.0],
            egui::Button::new(format!("{minutes}分の学習を始める"))).clicked() {
            self.start(minutes);
        }
        if ui.button("今日は2分だけ").clicked() { self.start(2); }
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
