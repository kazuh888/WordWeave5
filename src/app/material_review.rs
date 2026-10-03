use super::*;
use wordweave5::{
    assets::AssetRef,
    material::{ChangeReason, Draft},
    material_diff,
};
impl WordApp {
    pub(super) fn material_comparison(&mut self, ui: &mut egui::Ui, draft: &Draft) {
        let rows = material_diff::rows(draft.baseline.as_ref(), &draft.candidate);
        let key = egui::Id::new((
            "material-change",
            &draft.source.conversation_id,
            &draft.source.entry_id,
            draft.source.at,
            draft.mode.label(),
            self.material_selection_epoch,
        ));
        let previous = ui.ctx().data_mut(|d| d.get_temp::<String>(key));
        let mut selected = previous
            .as_ref()
            .cloned()
            .filter(|path| rows.iter().any(|row| row.path == *path))
            .or_else(|| rows.first().map(|row| row.path.clone()));
        let reasons = draft.active_reasons();
        if rows.is_empty() {
            ui.label("内容の差分はない。");
            ui.ctx().data_mut(|d| d.remove::<String>(key));
            return;
        }
        let mut image = None;
        let mut audio = None;
        let mut selected = selected.take().unwrap();
        let can_play = self.pending.is_none();
        let pane_height = ui.available_height().max(1.0);
        let left_width = (ui.available_width() * 0.34).min(240.0).max(1.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, pane_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let output = egui::ScrollArea::vertical()
                        .id_salt(key.with("change-list-scroll"))
                        .auto_shrink([false, false])
                        .min_scrolled_height(0.0)
                        .drag_to_scroll(false)
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                        .show(ui, |ui| comparison_list(ui, &rows, &mut selected));
                    #[cfg(any(test, debug_assertions))]
                    ui.ctx().data_mut(|data| data.insert_temp(
                        egui::Id::new("material-change-list-viewport"), output.inner_rect));
                    #[cfg(not(any(test, debug_assertions)))]
                    let _ = output;
                },
            );
            ui.separator();
            let right_width = ui.available_width().max(1.0);
            ui.allocate_ui_with_layout(
                egui::vec2(right_width, pane_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let mut scroll = egui::ScrollArea::vertical()
                        .id_salt(key.with(("change-detail-scroll", selected.as_str())))
                        .auto_shrink([false, false])
                        .min_scrolled_height(0.0)
                        .drag_to_scroll(false)
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
                    if previous.as_deref() != Some(selected.as_str()) {
                        scroll = scroll.vertical_scroll_offset(0.0);
                    }
                    let output = scroll.show(ui, |ui| {
                        (image, audio) = comparison_detail(
                            ui, draft, &rows, &selected, reasons, can_play, key,
                            pane_height < 180.0);
                        ui.label("変更箇所を選ぶと、変更前後と対応する理由・引用を表示する。");
                        if draft.generated.is_some() && reasons.is_empty() && !draft.reasons.is_empty() {
                            ui.label("手動編集後：生成時の理由は現在の差分には適用しない。元の根拠は保持している。");
                        }
                        if draft.generated.is_some() && reasons.is_empty() && !draft.reasons.is_empty() {
                            ui.push_id(key, |ui| {
                                ui.ww_collapsing("生成時の理由・引用を確認（現在の差分には未適用）", |ui| {
                                    for reason in &draft.reasons {
                                        ui.label(&reason.reason);
                                        for quote in &reason.quotes { ui.label(&quote.quote); }
                                    }
                                });
                            });
                        }
                    });
                    #[cfg(any(test, debug_assertions))]
                    ui.ctx().data_mut(|data| data.insert_temp(
                        egui::Id::new("material-change-detail-viewport"), output.inner_rect));
                    #[cfg(not(any(test, debug_assertions)))]
                    let _ = output;
                },
            );
        });
        ui.ctx().data_mut(|d| d.insert_temp(key, selected));
        if let Some(r) = image {
            self.preview_asset(&r);
        }
        if let Some(r) = audio {
            self.play_asset(&r);
        }
    }
}

fn comparison_list(ui: &mut egui::Ui, rows: &[material_diff::Row], selected: &mut String) {
    ui.strong("変更箇所");
    for row in rows {
        let label = format!("{}：{}", row.kind, row.label);
        if crate::app::controls::list_row(
            ui,
            &label,
            *selected == row.path,
        )
        .on_hover_text(&label)
        .clicked()
        {
            *selected = row.path.clone();
        }
    }
}

fn comparison_side(
    ui: &mut egui::Ui,
    title: &str,
    spans: &[(String, bool)],
    fill: Color32,
    highlight: Color32,
) {
    let width = ui.available_width().max(32.0);
    egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
        ui.set_width((width - 16.0).max(16.0));
        ui.vertical(|ui| {
            ui.strong(title);
            ui.add(
                egui::Label::new(diff_text(spans, highlight))
                    .wrap()
                    .selectable(true),
            );
        });
    });
}

fn comparison_detail(
    ui: &mut egui::Ui,
    draft: &Draft,
    rows: &[material_diff::Row],
    selected: &str,
    reasons: &[ChangeReason],
    can_play: bool,
    scope_key: egui::Id,
    compact: bool,
) -> (Option<AssetRef>, Option<AssetRef>) {
    let mut image = None;
    let mut audio = None;
    let Some(row) = rows.iter().find(|row| row.path == selected) else {
        return (image, audio);
    };
    if compact { ui.strong("変更理由・該当引用"); }
    ui.strong(format!("{}：{}", row.kind, row.label));
    let (before, after) = material_diff::spans(&row.before, &row.after);
    if ui.available_width() >= 600.0 {
        ui.columns(2, |columns| {
            comparison_side(
                &mut columns[0],
                "変更前",
                &before,
                Color32::from_rgb(255, 247, 247),
                Color32::from_rgb(255, 217, 217),
            );
            comparison_side(
                &mut columns[1],
                "登録される内容",
                &after,
                Color32::from_rgb(246, 252, 247),
                Color32::from_rgb(207, 239, 214),
            );
        });
    } else {
        comparison_side(
            ui,
            "変更前",
            &before,
            Color32::from_rgb(255, 247, 247),
            Color32::from_rgb(255, 217, 217),
        );
        comparison_side(
            ui,
            "登録される内容",
            &after,
            Color32::from_rgb(246, 252, 247),
            Color32::from_rgb(207, 239, 214),
        );
    }
    ui.separator();
    if !compact { ui.strong("変更理由・該当引用"); }
    let selected_reasons: Vec<_> = reasons
        .iter()
        .filter(|reason| {
            reason.path == row.path || reason.path.starts_with(&format!("{}/", row.path))
        })
        .collect();
    if selected_reasons.is_empty() {
        ui.label("理由未取得（推測で補完しない）。");
    }
    for reason in &selected_reasons {
        ui.label(&reason.reason);
        for quote in &reason.quotes {
            let turn = quote
                .exchange_index
                .checked_add(1)
                .map(|index| format!("往復 {index}"))
                .unwrap_or_else(|| "往復番号が不正".into());
            let role = match quote.role.as_str() {
                "user" => "あなた（質問）",
                "assistant" => "Codex（AIの回答）",
                _ => "話者不明",
            };
            ui.strong(format!("{turn}・{role} の引用"));
            ui.add(egui::Label::new(&quote.quote).wrap().selectable(true));
        }
    }
    if draft.source.snapshots.is_empty() {
        ui.label("旧版の案には固定原文がない。「元の会話を表示」で現在の履歴を参照する。現在の会話は生成時と異なる可能性がある。");
    } else {
        ui.push_id((scope_key, &row.path), |ui| {
            ui.ww_collapsing("生成時の固定会話を展開", |ui| {
                for snapshot in &draft.source.snapshots {
                    ui.push_id(snapshot.exchange_index, |ui| {
                        let turn = snapshot
                            .exchange_index
                            .checked_add(1)
                            .map(|index| format!("往復 {index}"))
                            .unwrap_or_else(|| "往復番号が不正".into());
                        ui.strong(turn);
                        for (role, label, text) in [
                            ("user", "あなた（質問）", &snapshot.exchange.question),
                            ("assistant", "Codex（AIの回答）", &snapshot.exchange.answer),
                        ] {
                            ui.small(label);
                            let quotes: Vec<_> = selected_reasons
                                .iter()
                                .flat_map(|reason| reason.quotes.iter())
                                .filter(|quote| {
                                    quote.exchange_index == snapshot.exchange_index
                                        && quote.role == role
                                })
                                .map(|quote| quote.quote.as_str())
                                .collect();
                            if role == "assistant" {
                                let markdown_id = ui.id().with(("fixed-answer", role));
                                ui.ww_collapsing("原文と該当引用を確認", |ui| {
                                    ui.add(
                                        egui::Label::new(quoted_text(text, &quotes))
                                            .wrap()
                                            .selectable(true),
                                    );
                                });
                                super::notifications::show_markdown(
                                    ui, markdown_id, text);
                            } else {
                                ui.add(
                                    egui::Label::new(quoted_text(text, &quotes))
                                        .wrap()
                                        .selectable(true),
                                );
                            }
                        }
                        for attachment in &snapshot.exchange.attachments {
                            ui.small(format!("媒体の根拠：{}", attachment.source_text));
                            if let Some(reference) = &attachment.image {
                                if ui.ww_button("注釈画像を表示").clicked() {
                                    image = Some(reference.clone());
                                }
                            }
                            if attachment.original.kind == wordweave5::assets::AssetKind::AudioWav
                                && ui
                                    .add_enabled(
                                        can_play,
                                        crate::app::controls::Button::new("原録音を再生"),
                                    )
                                    .clicked()
                            {
                                audio = Some(attachment.original.clone());
                            }
                        }
                        ui.separator();
                    });
                }
            });
        });
    }
    (image, audio)
}
fn diff_text(spans: &[(String, bool)], color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (text, changed) in spans {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(16.0),
                color: Color32::from_gray(30),
                background: if *changed {
                    color
                } else {
                    Color32::TRANSPARENT
                },
                ..Default::default()
            },
        );
    }
    if spans.is_empty() {
        job.append("（なし）", 0.0, egui::TextFormat::default());
    }
    job
}
fn quoted_text(text: &str, quotes: &[&str]) -> egui::text::LayoutJob {
    let mut ranges = Vec::new();
    for quote in quotes {
        if !quote.is_empty() {
            for (start, _) in text.match_indices(quote) {
                ranges.push((start, start + quote.len()));
            }
        }
    }
    let mut groups: Vec<(String, bool)> = Vec::new();
    for (i, c) in text.char_indices() {
        let marked = ranges.iter().any(|(s, e)| *s <= i && i < *e);
        if groups.last().is_none_or(|(_, m)| *m != marked) {
            groups.push((String::new(), marked));
        }
        groups.last_mut().unwrap().0.push(c);
    }
    diff_text(&groups, Color32::from_rgb(255, 237, 168))
}
