use super::*;

fn vocabulary_counts(deck: &[Entry], progress: &Progress) -> (usize, usize) {
    let mut bases = std::collections::BTreeMap::<String, bool>::new();
    for entry in deck
        .iter()
        .filter(|e| !progress.deleted_entries.contains(&e.id))
    {
        let answered = Skill::ALL.iter().any(|skill| {
            progress
                .memories
                .get(&skill.key(&entry.id))
                .is_some_and(|m| m.reviews > 0)
        });
        let learned = bases.entry(entry.base.trim().to_lowercase()).or_default();
        *learned |= answered;
    }
    (
        bases.len(),
        bases.values().filter(|&&learned| learned).count(),
    )
}

impl WordApp {
    pub(super) fn home_dashboard(&mut self, ui: &mut egui::Ui) {
        let (registered, answered) = vocabulary_counts(&self.deck, &self.progress);
        let date = today();
        let mut view = super::home_view::HomeView {
            registered,
            answered,
            due: scheduler::make_queue(&self.deck, &self.progress, now(), &date, 0).len(),
            first_today: self
                .progress
                .reviews
                .iter()
                .filter(|r| r.date == date && r.first)
                .count(),
            seconds: self.progress.study_seconds.get(&date).copied().unwrap_or(0) as u64,
            minutes: self.progress.settings.minutes,
            resuming: self.session.is_some(),
            enabled: self.fatal.is_none()
                && self.pending.is_none()
                && self.recorder.is_none()
                && !self.batch_running,
            start_requested: false,
            tips_requested: false,
        };
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 13.0),
                (egui::TextStyle::Button, 16.0),
            ] {
                ui.style_mut()
                    .text_styles
                    .insert(style, super::home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(super::home_art::INK);
            egui::Frame::new()
                .inner_margin(16)
                .show(ui, |ui| view.show(ui));
        });
        if view.start_requested {
            self.start(view.minutes);
        }
        if view.tips_requested {
            self.message = "例文で意味を確認し、別の問題を挟んでから思い出す。回答した経験と、独力で使える状態は区別する。".into();
        }
    }

    pub(super) fn study_start(&mut self, ui: &mut egui::Ui) {
        use super::home_art::*;
        use egui::{Color32, Frame, RichText, TextStyle};
        let minutes = self.progress.settings.minutes;
        let queue = self.study_queue(minutes);
        let due = queue.iter().filter(|task| !task.introduce).count();
        let new = queue.iter().filter(|task| task.introduce).count();
        let enabled = self.fatal.is_none()
            && self.pending.is_none()
            && self.recorder.is_none()
            && !self.batch_running;
        let mut requested = None;
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
            for (style, size) in [(TextStyle::Body, 19.0), (TextStyle::Small, 15.0), (TextStyle::Button, 17.0)] {
                ui.style_mut().text_styles.insert(style, home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(INK);
            Frame::new().inner_margin(16).show(ui, |ui| {
                let width = ui.available_width();
                card(ui, width, 320.0, Color32::from_rgb(233, 246, 255), |ui| {
                    if width > 950.0 {
                        let r = egui::Rect::from_min_size(ui.cursor().min + egui::vec2(width * 0.68, 0.0), egui::vec2(width * 0.25, 155.0));
                        scenery(ui.painter(), r);
                    }
                    title(ui, "学習を始める", 42.0);
                    ui.label("期限を迎えた復習を優先し、新しい表現を少しずつ学びましょう。");
                    ui.label(format!("まずは{minutes}分。回答の区切りで終了でき、そのまま続けることもできます。"));
                    ui.add_space(16.0);
                    let start_label = format!("▶  {minutes}分の学習を始める");
                    ui.horizontal(|ui| {
                        badge(ui, Icon::Clock, BLUE, 64.0);
                        title(ui, &format!("約 {minutes} 分"), 35.0);
                        if width >= 900.0 && ui.add_enabled_ui(enabled, |ui| primary(ui, &start_label, 390.0)).inner.clicked() {
                            requested = Some(minutes);
                        }
                    });
                    if width < 900.0 && ui.add_enabled_ui(enabled, |ui| primary(ui, &start_label, 390.0)).inner.clicked() {
                        requested = Some(minutes);
                    }
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.ww_button("今日は2分だけ").clicked() { requested = Some(2); }
                    });
                });
                ui.add_space(12.0);
                let metrics = [
                    ("今回の復習候補", format!("{due} 項目"), "期限を迎えた項目を優先", Icon::Refresh, ROSE),
                    ("新しく学ぶ候補", format!("{new} 項目"), "設定した1日の上限内で出題", Icon::File, GREEN),
                    ("今回の学習の目安", format!("{minutes} 分"), "短い時間から、無理なく続ける", Icon::Clock, BLUE),
                ];
                let columns = if width >= 900.0 { 3 } else { 1 };
                let card_width = (width - 16.0 * (columns - 1) as f32) / columns as f32;
                for row in metrics.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        for (label, value, note, icon, color) in row {
                            card(ui, card_width, 165.0, Color32::WHITE, |ui| {
                                ui.horizontal(|ui| {
                                    badge(ui, *icon, *color, 50.0);
                                    ui.vertical(|ui| {
                                        ui.label(*label);
                                        title(ui, value, 32.0);
                                    });
                                });
                                ui.label(RichText::new(*note).size(16.0).color(MUTED));
                            });
                        }
                    });
                }
                ui.add_space(12.0);
                let content_width = if width >= 950.0 { (width - 16.0) / 2.0 } else { width };
                let draw_skills = |ui: &mut egui::Ui| {
                    card(ui, content_width, 270.0, Color32::WHITE, |ui| {
                        ui.horizontal(|ui| { badge(ui, Icon::Book, BLUE, 48.0); title(ui, "練習する内容", 25.0); });
                        ui.label("現在の設定で選ばれている練習です。");
                        for skill in &self.progress.settings.skills { ui.label(format!("・{}", skill.label())); }
                        ui.label(RichText::new("学習時間・対象分野・技能は「設定」で変更できます。").size(16.0).color(MUTED));
                    });
                };
                let draw_tips = |ui: &mut egui::Ui| {
                    card(ui, content_width, 270.0, Color32::from_rgb(255, 251, 240), |ui| {
                        ui.horizontal(|ui| { badge(ui, Icon::Bulb, Color32::from_rgb(159, 111, 0), 48.0); title(ui, "上手に続けるコツ", 25.0); });
                        ui.label("・完璧を目指さず、まずは短い時間から。");
                        ui.label("・間違えた問題は、次の復習につなげましょう。");
                        ui.label("・疲れたら回答の区切りで終了できます。");
                        ui.label("・続けたくなったら、終了画面からもう少し学習できます。");
                    });
                };
                if width >= 950.0 { ui.horizontal_top(|ui| { draw_skills(ui); draw_tips(ui); }); }
                else { draw_skills(ui); ui.add_space(12.0); draw_tips(ui); }
                ui.add_space(12.0);
                card(ui, width, 66.0, Color32::from_rgb(237, 247, 254), |ui| {
                    ui.label(if queue.is_empty() {
                        "今取り組める問題はありません。復習予定を待つか、設定の対象分野・教材を確認してください。".to_owned()
                    } else { format!("今回の候補は復習 {due} 項目・新規 {new} 項目です。復習時期はWordWeaveの規則で計算します。") });
                });
            });
        });
        if let Some(minutes) = requested {
            self.start(minutes);
        }
    }

    pub(super) fn version_dialog(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Window::new("バージョン情報")
            .open(&mut self.about_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.heading("WordWeave5");
                ui.label(format!("バージョン {}", env!("CARGO_PKG_VERSION")));
                ui.label("Windows向け英語語彙学習アプリ");
                ui.label(format!("ライセンス：{}", env!("CARGO_PKG_LICENSE")));
                ui.separator();
                ui.small("この番号はWordWeave5本体のバージョンである。");
                ui.small("Codex CLIのバージョンとは異なる。");
                close = ui.ww_button("閉じる").clicked();
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
        p.memories
            .insert(Skill::ALL[0].key(&deck[0].id), scheduler::Memory::default());
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
