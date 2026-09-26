use super::*;

#[cfg(test)]
mod result_tests {
    use super::*;

    #[test]
    fn completion_layout_preserves_records_at_wide_and_narrow_widths() {
        for answered in [false, true] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            ctx.set_zoom_factor(1.0);
            app.start(5);
            if answered {
                app.grade(Grade::Good);
            }
            app.finish();
            assert!(app.session.is_none());
            assert!(app.session_summary.is_some());
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
            assert!(!app.notification_open, "学習終了の情報だけでは通知を自動表示しない");
            let records = serde_json::to_value(&app.progress).unwrap();
            for reason in [EndReason::Time, EndReason::Manual, EndReason::NoTasks] {
                app.session_summary.as_mut().unwrap().reason = reason;
                for width in [1400.0, 760.0, 420.0] {
                    for _ in 0..3 {
                        let _ = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 950.0),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    let right = ui.max_rect().right();
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        app.session_result(ui);
                                        assert!(
                                            ui.min_rect().right() <= right + 1.0,
                                            "width={width}, right={right}, {:?}",
                                            ui.min_rect()
                                        );
                                    });
                                });
                            },
                        );
                    }
                }
            }
            assert_eq!(serde_json::to_value(&app.progress).unwrap(), records);
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum EndReason {
    Time,
    Manual,
    NoTasks,
}

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
    pub(super) fn collect(
        session: &Session,
        progress: &Progress,
        deck: &[Entry],
        reason: EndReason,
    ) -> Self {
        let records = &progress.reviews[session.review_start.min(progress.reviews.len())..];
        let keys: std::collections::BTreeSet<_> = records.iter().map(|r| r.key.as_str()).collect();
        Self {
            reason,
            seconds: session.elapsed.as_secs(),
            answers: records.len(),
            independent: records
                .iter()
                .filter(|r| !r.assisted && matches!(r.grade, Grade::Good | Grade::Easy))
                .count(),
            reviewed: records
                .iter()
                .filter(|r| !r.first)
                .map(|r| &r.key)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            expressions: deck
                .iter()
                .filter(|e| {
                    Skill::ALL
                        .iter()
                        .any(|s| keys.contains(s.key(&e.id).as_str()))
                })
                .cloned()
                .collect(),
            next_due: keys
                .iter()
                .filter_map(|key| progress.memories.get(*key).map(|m| m.due))
                .min(),
        }
    }
}

impl WordApp {
    pub(super) fn session_result(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 12.0);
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 15.0),
                (egui::TextStyle::Button, 18.0),
            ] {
                ui.style_mut()
                    .text_styles
                    .insert(style, super::home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(super::home_art::INK);
            egui::Frame::new()
                .inner_margin(16)
                .show(ui, |ui| self.session_result_content(ui));
        });
    }

    fn session_result_content(&mut self, ui: &mut egui::Ui) {
        use super::home_art::{badge, card, scenery, title, Icon, BLUE, GREEN, PURPLE, ROSE};
        let Some(summary) = self.session_summary.as_ref() else {
            return;
        };
        let width = ui.available_width();
        card(ui, width, 160.0, Color32::from_rgb(233, 246, 255), |ui| {
            if width > 950.0 {
                let rect = egui::Rect::from_min_size(
                    ui.cursor().min + egui::vec2(width * 0.74, 0.0),
                    egui::vec2(width * 0.20, 110.0),
                );
                scenery(ui.painter(), rect);
            }
            title(
                ui,
                "今回の学習を終えました",
                if width > 800.0 { 36.0 } else { 28.0 },
            );
            ui.label(match summary.reason {
                EndReason::Time => "設定した時間に達したため、回答の区切りで終了した。",
                EndReason::Manual => "ここで一区切り。記録した回答は保存されている。",
                EndReason::NoTasks => {
                    "今取り組める問題は終了した。復習期限と新規項目の上限を守って出題する。"
                }
            });
            ui.label("今日の取り組みを振り返り、自分に合った区切りで終えましょう。");
        });
        ui.add_space(16.0);
        let metrics = [
            (
                Icon::Clock,
                BLUE,
                "学習時間",
                ux::duration(summary.seconds),
                "休止・AI待機を除く",
            ),
            (
                Icon::File,
                GREEN,
                "記録した回答",
                format!("{}回", summary.answers),
                "同じ問題への再回答を含む",
            ),
            (
                Icon::Check,
                ROSE,
                "ヒントなしでできた",
                format!("{}回", summary.independent),
                "自己評価を含む",
            ),
            (
                Icon::Refresh,
                PURPLE,
                "復習した練習項目",
                format!("{}項目", summary.reviewed),
                "初回を除く・重複なし",
            ),
        ];
        let columns = if width >= 1000.0 {
            4
        } else if width >= 600.0 {
            2
        } else {
            1
        };
        for row in metrics.chunks(columns) {
            ui.horizontal(|ui| {
                let metric_width = (width - 16.0 * (columns - 1) as f32) / columns as f32;
                for (icon, color, label, value, note) in row {
                    card(ui, metric_width, 160.0, Color32::WHITE, |ui| {
                        ui.horizontal(|ui| {
                            badge(ui, *icon, *color, 38.0);
                            ui.label(*label);
                        });
                        title(ui, value, 30.0);
                        ui.small(*note);
                    });
                }
            });
        }
        ui.add_space(12.0);
        let achievements = |ui: &mut egui::Ui, w| {
            card(ui, w, 225.0, Color32::WHITE, |ui| {
                title(ui, "今回の積み重ね", 25.0);
                if summary.answers == 0 {
                    ui.label("まだ回答は記録していない。教材を確認した時間は学習時間に残る。");
                } else {
                    ui.label(format!(
                        "{}件の教材に取り組んだ。",
                        summary.expressions.len()
                    ));
                    ui.label(format!(
                        "ヒントなしでできたと記録：{}回（自己評価を含む）。",
                        summary.independent
                    ));
                }
                if let Some(due) = summary
                    .next_due
                    .and_then(|at| chrono::DateTime::from_timestamp(at, 0))
                {
                    ui.small(format!(
                        "今回の項目で最も近い復習予定：{}",
                        due.with_timezone(&chrono::Local).format("%m/%d %H:%M")
                    ));
                }
                ui.add_space(8.0);
                ui.label("続けなくても記録は失われません。次の学習につなげましょう。");
            });
        };
        let expression = |ui: &mut egui::Ui, w| {
            card(ui, w, 225.0, Color32::from_rgb(240, 249, 255), |ui| {
                title(ui, "取り組んだ表現", 25.0);
                if let Some(entry) = summary.expressions.first() {
                    title(ui, &entry.base, 28.0);
                    ui.label(&entry.meaning);
                    ui.label(entry.completed());
                    ui.small(&entry.translation);
                } else {
                    ui.label("今回はまだ回答を記録していません。");
                    ui.small("回答を記録すると、ここで表現を振り返れます。");
                }
            });
        };
        if width >= 850.0 {
            ui.horizontal(|ui| {
                achievements(ui, (width - 16.0) / 2.0);
                expression(ui, (width - 16.0) / 2.0);
            });
        } else {
            achievements(ui, width);
            expression(ui, width);
        }
        if summary.expressions.len() > 1 {
            ui.add_space(12.0);
            ui.ww_collapsing("今回取り組んだ表現を振り返る", |ui| {
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
            if ui
                .add(
                    crate::app::controls::Button::new("今日はここで終了")
                        .min_size(egui::vec2(220.0, 48.0)),
                )
                .clicked()
            {
                self.page = Page::Home;
            }
            if ux::primary(ui, "もう少し学習する", can_continue).clicked() {
                self.start(self.progress.settings.minutes);
            }
        });
        if !can_continue {
            ui.label("今出題できる対象はない。次の復習を待つか、教材を読む・英語チャットで質問することができる。");
        }
    }

    pub(super) fn request_study_end(&mut self, ui: &mut egui::Ui) {
        if !self.study_end_confirm {
            return;
        }
        ux::panel(ui, true, |ui| {
            ui.strong("この問題の回答はまだ記録していない");
            ui.label("ここで終了すると、この問題の入力・筆跡・録音は学習記録に登録されない。回答済みの記録は保持する。");
            ui.horizontal_wrapped(|ui| {
                if ui.ww_button("問題に戻る").clicked() {
                    self.study_end_confirm = false;
                }
                if ui.ww_button("未記録の回答を破棄して終了").clicked() {
                    self.study_end_confirm = false;
                    self.finish();
                }
            });
        });
    }
}
