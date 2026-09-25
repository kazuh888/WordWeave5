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
        ux::heading(
            ui,
            "学習の記録",
            "学習した時間と、取り組んだ表現を振り返る。",
        );
        let total = record_summary(&self.progress, None);
        ux::metric_group(ui, |ui| {
            ux::metric(
                ui,
                "記録した学習時間",
                ux::duration(total.seconds),
                "これまでの合計",
            );
            ux::metric(
                ui,
                "記録した回答",
                format!("{}回", total.answers),
                "同じ項目への再回答を含む",
            );
            ux::metric(
                ui,
                "取り組んだ練習項目",
                format!("{}項目", total.items),
                "同じ教材でも練習形式ごとに数える",
            );
        });
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
            ui.strong("回答の内訳");
            ux::metric_group(ui, |ui| {
                ux::metric(
                    ui,
                    "初めての回答",
                    format!("{}回", total.new_answers),
                    "回答時点で初回だった記録",
                );
                ux::metric(
                    ui,
                    "復習としての回答",
                    format!("{}回", total.review_answers),
                    "2回目以降の記録",
                );
            });
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
            ui.strong("直近7日の学習");
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
                        .desired_height(8.0),
                );
                ui.add_space(4.0);
            }
            ui.small("棒の長さはこの7日間の最長学習時間との比較。休んだ日は0分として表示する。連続記録が途切れても学習履歴はリセットしない。");
        });
    }

    fn record_details(&self, ui: &mut egui::Ui) {
        ux::panel(ui, false, |ui| {
            ui.strong("詳しく振り返る");
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
            ui.strong("日本語出題の添削履歴（最新20件）");
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
