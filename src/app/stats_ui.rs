use super::*;

#[derive(Debug, Default, PartialEq, Eq)]
struct RecordSummary {
    answers: usize,
    new_answers: usize,
    review_answers: usize,
    items: usize,
    seconds: u64,
}

fn record_summary(progress: &Progress, range: Option<(&str, &str)>) -> RecordSummary {
    let includes = |date: &str| range.is_none_or(|(from, through)| from <= date && date <= through);
    let mut summary = RecordSummary::default();
    let mut items = std::collections::BTreeSet::new();
    for review in progress
        .reviews
        .iter()
        .filter(|review| includes(&review.date))
    {
        summary.answers += 1;
        if review.first {
            summary.new_answers += 1;
        } else {
            summary.review_answers += 1;
        }
        items.insert(review.key.as_str());
    }
    summary.items = items.len();
    summary.seconds = progress
        .study_seconds
        .iter()
        .filter(|(date, _)| includes(date))
        .map(|(_, seconds)| u64::from(*seconds))
        .sum();
    summary
}

impl WordApp {
    pub(super) fn stats(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 10.0);
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 16.0),
                (egui::TextStyle::Button, 18.0),
            ] {
                ui.style_mut().text_styles.insert(style, home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(home_art::INK);
            egui::Frame::new().inner_margin(16).show(ui, |ui| self.records_content(ui));
        });
    }

    fn records_content(&self, ui: &mut egui::Ui) {
        ux::panel(ui, true, |ui| {
            ui.horizontal(|ui| {
                home_art::badge(ui, home_art::Icon::Chart, home_art::BLUE, 44.0);
                home_art::title(ui, "学習の記録", 28.0);
            });
            ui.label("学習した時間と、取り組んだ表現を振り返る。");
        });
        ui.add_space(6.0);
        let total = record_summary(&self.progress, None);
        record_metrics(ui, &[
            ("記録した学習時間", ux::duration(total.seconds), "これまでの合計"),
            ("記録した回答", format!("{}回", total.answers), "同じ項目への再回答を含む"),
            ("取り組んだ練習項目", format!("{}項目", total.items), "同じ教材でも練習形式ごとに数える"),
        ]);
        ui.add_space(12.0);
        if total.answers == 0 {
            ux::panel(ui, true, |ui| {
                ui.strong("回答の記録はまだない。");
                ui.label("学習画面で回答して評価すると、ここに記録が残る。休んだ日があっても履歴は失われない。");
            });
            ui.add_space(12.0);
        }
        self.recent_activity(ui);
        ui.add_space(12.0);
        ux::panel(ui, false, |ui| {
            home_art::title(ui, "回答の内訳", 22.0);
            record_metrics(ui, &[
                ("初めての回答", format!("{}回", total.new_answers), "回答時点で初回だった記録"),
                ("復習としての回答", format!("{}回", total.review_answers), "2回目以降の記録"),
            ]);
            ui.small("項目数や回答数は学習経験を表す。正解数・習得済み語彙数ではない。削除・編集前の回答履歴も含む。");
        });
        ui.add_space(12.0);
        self.record_details(ui);
        ui.add_space(12.0);
        self.writing_record_details(ui);
    }

    fn recent_activity(&self, ui: &mut egui::Ui) {
        let today = chrono::Local::now().date_naive();
        let days: Vec<_> = (0..7)
            .rev()
            .map(|days| {
                let date = (today - chrono::Duration::days(days))
                    .format("%Y-%m-%d")
                    .to_string();
                let summary = record_summary(&self.progress, Some((&date, &date)));
                (date, summary)
            })
            .collect();
        let first = &days[0].0;
        let last = &days[6].0;
        let week = record_summary(&self.progress, Some((first, last)));
        let longest = days
            .iter()
            .map(|(_, day)| day.seconds)
            .max()
            .unwrap_or(0)
            .max(1);
        ux::panel(ui, false, |ui| {
            home_art::title(ui, "直近7日の学習", 22.0);
            ui.label(format!(
                "{} / {}回答 / {}練習項目",
                ux::duration(week.seconds),
                week.answers,
                week.items
            ));
            for (date, day) in &days {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(date);
                    ui.label(format!(
                        "{} · {}回答 · 新規{} / 復習{}",
                        ux::duration(day.seconds),
                        day.answers,
                        day.new_answers,
                        day.review_answers
                    ));
                });
                ui.add(
                    egui::ProgressBar::new(day.seconds as f32 / longest as f32)
                        .desired_width(ui.available_width())
                        .fill(home_art::BLUE)
                        .desired_height(8.0),
                );
                ui.add_space(4.0);
            }
            ui.small("棒の長さはこの7日間の最長学習時間との比較。休んだ日は0分として表示する。連続記録が途切れても学習履歴はリセットしない。");
        });
    }

    fn record_details(&self, ui: &mut egui::Ui) {
        ux::panel(ui, false, |ui| {
            home_art::title(ui, "詳しく振り返る", 22.0);
            ui.ww_collapsing("間隔を空けた復習の記録", |ui| {
                for days in [7.0, 30.0] {
                    let result = match self.progress.observed_retention(days) {
                        Some((ok, total)) => format!("{ok}/{total}回 ({:.0}%)", 100.0 * ok as f64 / total as f64),
                        None => "まだ記録がない".into(),
                    };
                    ui.label(format!("前回から{days:.0}日以上：{result}"));
                }
                ui.small("ヒントなしでGood/Easyと記録した割合。自己評価と照合結果を含む通常の復習記録であり、能力テストの正解率ではない。7日以上には30日以上も含まれる。");
            });
            ui.ww_collapsing("練習の種類別の記録", |ui| {
                for skill in Skill::ALL {
                    let suffix = format!(":{}", skill.code());
                    let answers = self
                        .progress
                        .reviews
                        .iter()
                        .filter(|r| r.key.ends_with(&suffix))
                        .count();
                    let items = self
                        .progress
                        .memories
                        .keys()
                        .filter(|key| key.ends_with(&suffix))
                        .count();
                    ui.label(format!(
                        "{}：{answers}回答 / 復習状態のある項目 {items}件",
                        skill.label()
                    ));
                }
                ui.small(
                    "回答数は過去の履歴、復習状態のある項目数は現在保持している状態から集計する。",
                );
            });
            ui.ww_collapsing("入力方法別の回答記録", |ui| {
                for (method, label) in [("keyboard", "キーボード"), ("pen", "手書き"), ("voice", "音声")] {
                    let records: Vec<_> = self.progress.reviews.iter().filter(|r| r.method == method).collect();
                    let independent = records.iter().filter(|r| !r.assisted && matches!(r.grade, Grade::Good | Grade::Easy)).count();
                    ui.label(format!("{label}：{}回 / ヒントなし・Good/Easyの記録 {independent}回", records.len()));
                }
                ui.small("自己評価を含む。方式ごとに問題や難しさが異なるため、この差だけで入力方法の効果は判定できない。");
            });
        });
    }

    fn writing_record_details(&self, ui: &mut egui::Ui) {
        ux::panel(ui, false, |ui| {
            home_art::title(ui, "日本語出題の添削履歴（最新20件）", 22.0);
            if self.progress.writing_logs.is_empty() {
                ui.label("添削履歴はまだない。日本語から英文を作る練習の記録をここに表示する。");
            }
            for (index, log) in self.progress.writing_logs.iter().enumerate().rev().take(20) {
                let time = chrono::DateTime::from_timestamp(log.at, 0)
                    .map(|time| {
                        time.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
                    .unwrap_or_else(|| log.at.to_string());
                crate::app::controls::CollapsingHeader::new(format!(
                    "{time} / {}",
                    if log.revision {
                        "書き直し"
                    } else {
                        "初回回答"
                    }
                ))
                .id_salt(("writing-record", index))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(format!("対象：{}", log.problem.target)).wrap());
                    ui.add(egui::Label::new(&log.problem.japanese).wrap());
                    ui.add(egui::Label::new(format!("回答：{}", log.answer)).wrap());
                    ui.add(egui::Label::new(&log.feedback).wrap());
                });
            }
        });
    }
}

/// Cards always have a vertical inner layout, even in a horizontal metric row.
fn record_metrics(ui: &mut egui::Ui, metrics: &[(&str, String, &str)]) {
    let width = ui.available_width();
    let columns = if width >= 1000.0 { 3 } else if width >= 640.0 { 2 } else { 1 };
    let columns = columns.min(metrics.len().max(1));
    let card_width = (width - 16.0 * (columns - 1) as f32) / columns as f32;
    for row in metrics.chunks(columns) {
        ui.horizontal_top(|ui| {
            for (label, value, note) in row {
                home_art::card(ui, card_width, 156.0, Color32::WHITE, |ui| {
                    ui.label(RichText::new(*label).size(17.0).color(home_art::MUTED));
                    home_art::title(ui, value, 30.0);
                    ui.small(*note);
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordweave5::store::Review;

    fn review(key: &str, date: &str, first: bool) -> Review {
        Review {
            key: key.into(),
            at: 0,
            date: date.into(),
            grade: Grade::Good,
            assisted: false,
            method: "keyboard".into(),
            first,
            elapsed_days: 0.0,
            seconds: 999,
            self_assessed: true,
        }
    }

    #[test]
    fn empty_records_have_zero_activity() {
        assert_eq!(
            record_summary(&Progress::default(), None),
            RecordSummary::default()
        );
    }

    #[test]
    fn record_screen_is_readable_bounded_and_does_not_change_history() {
        fn text_bounds(shape: &egui::epaint::Shape, label: &str) -> Option<(egui::Rect, f32)> {
            match shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => Some((
                    egui::Rect::from_min_size(text.pos, text.galley.size()),
                    text.galley.job.sections[0].format.font_id.size,
                )),
                egui::epaint::Shape::Vec(shapes) => shapes.iter()
                    .find_map(|shape| text_bounds(shape, label)),
                _ => None,
            }
        }
        for filled in [false, true] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            ctx.set_zoom_factor(1.0);
            if filled {
                app.progress.reviews = vec![
                    review("sample:recall", &today(), true),
                    review("sample:recall", &today(), false),
                ];
                app.progress.study_seconds.insert(today(), 300);
            }
            let history = serde_json::to_value(&app.progress).unwrap();
            for width in [1400.0, 760.0, 420.0] {
                for _ in 0..3 {
                    let output = ctx.run(egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO, egui::vec2(width, 5000.0))),
                        ..Default::default()
                    }, |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let right = ui.max_rect().right();
                            let styles = ui.style().text_styles.clone();
                            app.stats(ui);
                            assert!(ui.min_rect().right() <= right + 1.0,
                                "record screen overflow at {width}: {:?}", ui.min_rect());
                            assert_eq!(ui.style().text_styles, styles, "page styles must stay local");
                        });
                    });
                    let find = |label| output.shapes.iter()
                        .find_map(|shape| text_bounds(&shape.shape, label))
                        .unwrap_or_else(|| panic!("missing visible text: {label}"));
                    let (label, font) = find("記録した学習時間");
                    assert!(font >= 17.0);
                    let (value, font) = find(if filled { "5分0秒" } else { "0分0秒" });
                    assert_eq!(font, 30.0);
                    let (note, font) = find("これまでの合計");
                    assert!(font >= 16.0);
                    assert!(label.bottom() <= value.top() + 1.0 && value.bottom() <= note.top() + 1.0,
                        "card text must stack vertically: {label:?} {value:?} {note:?}");
                    assert_eq!(find("学習した時間と、取り組んだ表現を振り返る。").1, 19.0);
                }
            }
            assert_eq!(serde_json::to_value(&app.progress).unwrap(), history);
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn record_cards_wrap_large_totals_without_expanding_the_page() {
        let (ctx, app, root) = super::super::harness_tests::fixture();
        for width in [1400.0, 760.0, 420.0] {
            let _ = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO, egui::vec2(width, 2000.0))),
                ..Default::default()
            }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    let right = ui.max_rect().right();
                    record_metrics(ui, &[
                        ("記録した学習時間", ux::duration(u64::MAX), "大きな累積値も折り返す"),
                        ("取り組んだ練習項目", "999999999999項目".into(), "練習形式ごとの項目"),
                    ]);
                    assert!(ui.min_rect().right() <= right + 1.0);
                });
            });
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn date_range_is_inclusive_and_uses_recorded_study_time() {
        let mut progress = Progress::default();
        for date in ["2026-09-13", "2026-09-14", "2026-09-20", "2026-09-21"] {
            progress.reviews.push(review("word:recall", date, false));
            progress.study_seconds.insert(date.into(), 10);
        }
        let summary = record_summary(&progress, Some(("2026-09-14", "2026-09-20")));
        assert_eq!(
            summary,
            RecordSummary {
                answers: 2,
                new_answers: 0,
                review_answers: 2,
                items: 1,
                seconds: 20,
            }
        );
    }

    #[test]
    fn repeated_answers_count_once_per_practice_item_without_inferring_mastery() {
        let mut progress = Progress {
            reviews: vec![
                review("word:recall", "2026-09-20", true),
                review("word:recall", "2026-09-20", false),
                review("word:usage", "2026-09-20", true),
            ],
            ..Default::default()
        };
        progress.reviews[1].assisted = true;
        progress.reviews[1].grade = Grade::Again;
        progress.reviews[2].self_assessed = false;
        assert_eq!(
            record_summary(&progress, None),
            RecordSummary {
                answers: 3,
                new_answers: 2,
                review_answers: 1,
                items: 2,
                seconds: 0,
            }
        );
    }

    #[test]
    fn time_without_answers_is_kept_and_all_time_sum_does_not_overflow_u32() {
        let mut progress = Progress::default();
        progress.study_seconds.insert("2026-09-19".into(), u32::MAX);
        progress.study_seconds.insert("2026-09-20".into(), u32::MAX);
        let summary = record_summary(&progress, None);
        assert_eq!(summary.seconds, u64::from(u32::MAX) * 2);
        assert_eq!(summary.answers, 0);
    }
}
