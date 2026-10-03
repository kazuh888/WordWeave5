use super::*;
use wordweave5::material::{
    DiagnosticStage as MaterialStage, EvidenceCause, MaterialDiagnostic, MaterialFailure,
};
mod markdown;

pub(super) fn show_markdown(ui: &mut egui::Ui, id: egui::Id, source: &str) {
    markdown::show(ui, id, source);
}

fn material_title(stage: MaterialStage) -> &'static str {
    match stage {
        MaterialStage::Generation => "教材案を作成できなかった",
        MaterialStage::SavedDraft => "この案は登録できない",
    }
}

fn material_reason(cause: EvidenceCause) -> &'static str {
    match cause {
        EvidenceCause::SourceReference => "作成時の根拠参照を確認できなかった。",
        EvidenceCause::InvalidPath => "変更する教材項目の指定が正しくない。",
        EvidenceCause::MissingItem => "指定された教材項目が案に存在しない。",
        EvidenceCause::EmptyReason => "変更理由の説明が空白である。",
        EvidenceCause::LongReason => "変更理由の説明が長すぎる。",
        EvidenceCause::MissingQuotes => "変更理由に元発言の引用がない。",
        EvidenceCause::TooManyQuotes => "変更理由の引用が多すぎる。",
        EvidenceCause::MissingSnapshot => "指定された固定元発言を取得できない。",
        EvidenceCause::InvalidRole => "引用の話者指定が正しくない。",
        EvidenceCause::EmptyQuote => "引用が空白である。",
        EvidenceCause::LongQuote => "引用が長すぎる。",
        EvidenceCause::QuoteMismatch => "AIが示した引用を元の発言にそのまま見つけられなかった。",
    }
}

fn excerpt(text: &str) -> String {
    let mut chars = text.chars();
    let start: String = chars.by_ref().take(240).collect();
    if chars.next().is_some() {
        format!("{start}…（省略）")
    } else {
        start
    }
}

pub(super) fn diagnostic_copy_text(diagnostic: &MaterialDiagnostic) -> String {
    let mut text = format!(
        "結果: {}。教材には登録していない。既存教材は変更していない。\n理由: {}\n次にできること: 元の発言と対象項目を確認し、必要なら教材案の作成を明示的に再試行する。\n原因詳細: {}",
        material_title(diagnostic.stage), material_reason(diagnostic.cause), diagnostic.cause_detail,
    );
    if let Some(number) = diagnostic.reason_number {
        text.push_str(&format!("\n変更理由番号: {number}"));
    }
    if let Some(path) = &diagnostic.path {
        text.push_str(&format!("\npath: {path}"));
    }
    if let Some(label) = &diagnostic.target_label {
        text.push_str(&format!("\n対象項目: {label}"));
    }
    if let Some(evidence) = &diagnostic.evidence {
        text.push_str(&format!(
            "\nexchange_index: {}\nrole: {}\nAI引用（先頭240文字の抜粋）: {}",
            evidence.exchange_index,
            evidence.role,
            excerpt(&evidence.quote)
        ));
        if let Some(original) = &evidence.original {
            text.push_str(&format!(
                "\n固定元発言（先頭240文字の抜粋）: {}",
                excerpt(original)
            ));
        } else {
            text.push_str(
                "\n固定元発言: 取得できない（固定版の往復番号または話者を確認してください）",
            );
        }
    } else {
        text.push_str("\nAI引用・固定元発言: 引用照合前に止まったため取得していない");
    }
    text
}

pub(super) struct MaterialNotice {
    diagnostic: MaterialDiagnostic,
    pub(super) comparison_open: bool,
    technical_open: bool,
    raw_mode: bool,
}

pub(super) struct MaterialRegistrationReceipt {
    pub(super) target_base: String,
}

pub(super) struct DailyLimitNotice {
    day: String,
    text: String,
}

fn enlarge_notice(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    let font = super::chrome::status_font();
    for entry in style.text_styles.values_mut() {
        *entry = font.clone();
    }
    style.override_font_id = Some(font);
}

fn missing_codex_path(message: &str) -> bool {
    message.starts_with("指定したCodex実行ファイルがありません。")
        || message.starts_with("Codexが見つかりません。")
        || message == "Codexの実行ファイルを指定してください。"
}

impl WordApp {
    pub(super) fn notify_material_registered(&mut self, target_base: String) {
        self.notify_result(Ok(format!(
            "{target_base} の教材を登録した。元の会話への参照も保存した。"
        )));
        self.material_registration_receipt = Some(MaterialRegistrationReceipt { target_base });
    }

    pub(super) fn show_material_registration_receipt(&mut self, ui: &mut egui::Ui) {
        let Some(target_base) = self
            .material_registration_receipt
            .as_ref()
            .map(|receipt| receipt.target_base.clone())
        else {
            return;
        };
        egui::Frame::new()
            .fill(Color32::from_rgb(235, 249, 239))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().max(1.0));
                ui.label("教材の登録結果");
                egui::ScrollArea::vertical()
                    .id_salt("material-registration-receipt")
                    .max_height(64.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(format!(
                                "{} の教材を登録した。元の会話への参照も保存した。",
                                target_base
                            ))
                            .wrap()
                            .selectable(true),
                        );
                    });
                if ui.ww_button("登録結果を閉じる").clicked() {
                    self.material_registration_receipt = None;
                }
            });
    }

    fn expire_material_notice_if_replaced(&mut self) {
        if self.fatal.is_some()
            || self
                .material_notice
                .as_ref()
                .is_some_and(|notice| self.message != notice.diagnostic.legacy_text)
        {
            self.material_notice = None;
        }
    }

    pub(super) fn notify_error(&mut self, message: String) {
        self.material_notice = None;
        self.daily_limit_notice = None;
        self.message = message;
        self.notification_error = self.message.clone();
        self.notification_attention.clear();
        self.last_notification_alerts[1].clear();
    }

    pub(super) fn notify_warning(&mut self, message: impl Into<String>) {
        self.notify_attention(message);
    }

    pub(super) fn notify_blocked(&mut self, message: impl Into<String>) {
        self.notify_attention(message);
    }

    // Warnings and unfulfilled requests require attention, but are not red errors.
    fn notify_attention(&mut self, message: impl Into<String>) {
        self.material_notice = None;
        self.daily_limit_notice = None;
        self.message = message.into();
        self.notification_attention = self.message.clone();
        self.notification_error.clear();
        // A fresh request may be refused for the same reason as a dismissed one.
        self.last_notification_alerts[1].clear();
    }

    pub(super) fn notify_result(&mut self, result: Result<String, String>) {
        self.material_notice = None;
        self.daily_limit_notice = None;
        match result {
            Ok(message) => {
                self.message = message;
                self.notification_error.clear();
                self.notification_attention.clear();
            }
            Err(error) => self.notify_error(error),
        }
    }

    fn notification_is_error(&self) -> bool {
        self.fatal.is_some()
            || missing_codex_path(&self.message)
            || (!self.message.is_empty() && self.message == self.notification_error)
    }
    pub(super) fn notify_daily_limit_rejected(&mut self, message: impl Into<String>) {
        self.notify_attention(message);
        self.daily_limit_notice = Some(DailyLimitNotice {
            day: today(),
            text: self.message.clone(),
        });
    }

    pub(super) fn notify_material_diagnostic(&mut self, diagnostic: MaterialDiagnostic) {
        self.notify_error(MaterialFailure::Evidence(diagnostic.clone()).legacy_message());
        self.material_notice = Some(MaterialNotice {
            diagnostic,
            comparison_open: false,
            technical_open: false,
            raw_mode: false,
        });
    }

    fn daily_limit_action_applies(&self) -> bool {
        let Some(notice) = &self.daily_limit_notice else {
            return false;
        };
        let day = today();
        let count = self.progress.ai_calls.get(&day).copied().unwrap_or(0);
        let limit = self.progress.settings.ai_daily_limit;
        notice.day == day
            && notice.text == self.message
            && self.notification_attention == self.message
            && self.fatal.is_none()
            && limit < 1000
            && count < 1000
            && count >= limit
    }

    fn open_daily_limit_guidance(&mut self) {
        if self.qwen_settings.is_some() || !self.daily_limit_action_applies() {
            return;
        }
        self.page = Page::Settings;
        self.settings_section = settings_ui::SettingsSection::Connection;
        self.codex_path_guidance = false;
        self.codex_path_focus_pending = false;
        self.daily_limit_guidance = true;
        self.daily_limit_focus_pending = true;
        self.notification_open = false;
    }

    pub(super) fn open_codex_path_guidance(&mut self) {
        if self.qwen_settings.is_some() {
            return;
        }
        self.page = Page::Settings;
        self.daily_limit_guidance = false;
        self.daily_limit_focus_pending = false;
        self.codex_path_guidance = true;
        self.codex_path_focus_pending = true;
        self.notification_open = false;
    }

    pub(super) fn notification_button(&mut self, ui: &mut egui::Ui) {
        self.expire_material_notice_if_replaced();
        ui.scope(|ui| {
            enlarge_notice(ui);
            let detail = self.notification_text();
            let error = self.notification_is_error();
            let label = if !detail.is_empty() {
                "● 通知"
            } else {
                "○ 通知"
            };
            let label = if error {
                RichText::new(label).color(Color32::from_rgb(185, 35, 35))
            } else {
                RichText::new(label)
            };
            let summary = if let Some(notice) = &self.material_notice {
                format!(
                    "{}。{}",
                    material_title(notice.diagnostic.stage),
                    material_reason(notice.diagnostic.cause)
                )
            } else if detail.is_empty() {
                "通知はない".to_owned()
            } else {
                detail
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .take(180)
                    .collect()
            };
            if ui
                .add(crate::app::controls::Button::new(label).min_size(egui::vec2(80.0, 30.0)))
                .on_hover_text(
                    RichText::new(format!("通知：{summary}\nクリックで詳細・コピー"))
                        .font(super::chrome::status_font()),
                )
                .clicked()
            {
                if !self.notification_open {
                    if let Some(notice) = &mut self.material_notice {
                        notice.comparison_open = false;
                        notice.technical_open = false;
                        notice.raw_mode = false;
                    }
                }
                self.notification_open = !self.notification_open;
            }
        });
    }

    fn notification_text(&self) -> String {
        [
            self.fatal.as_deref().unwrap_or_default(),
            self.message.as_str(),
            self.font_notice.as_str(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
    }

    pub(super) fn notification_window(&mut self, ctx: &egui::Context) {
        self.expire_material_notice_if_replaced();
        // Explicit failures, warnings and unfulfilled requests interrupt. Results remain
        // available through the status button, without reopening a dismissed alert.
        let alerts = [
            self.fatal.as_deref().unwrap_or_default(),
            if missing_codex_path(&self.message)
                || (!self.message.is_empty()
                    && (self.message == self.notification_error
                        || self.message == self.notification_attention))
            {
                self.message.as_str()
            } else {
                ""
            },
            self.font_notice.as_str(),
        ];
        if self.pending.is_none() && !self.batch_running {
            // Removing a warning (e.g. by copying text) is not a new alert.
            for (alert, seen) in alerts.into_iter().zip(&mut self.last_notification_alerts) {
                if alert != seen {
                    if !alert.is_empty() {
                        self.notification_open = true;
                    }
                    *seen = alert.to_owned();
                }
            }
        }
        if !self.notification_open {
            return;
        }
        let mut open = true;
        let mut close_material = false;
        let mut fix_path = false;
        let mut fix_daily_limit = false;
        let max_width = (ctx.screen_rect().width() - 40.0).max(240.0);
        let max_height = (ctx.screen_rect().height() - 60.0).max(160.0);
        let material_details = self.material_notice.is_some();
        let shown =
            egui::Window::new(RichText::new("通知の詳細").font(super::chrome::status_font()))
                .id(egui::Id::new(if material_details {
                    "notification-details-material-v1"
                } else {
                    "notification-details-v2"
                }))
                .open(&mut open)
                .resizable([true, true])
                .default_size(egui::vec2(
                    580.0_f32.min(max_width),
                    (if material_details { 540.0_f32 } else { 240.0_f32 }).min(max_height),
                ))
                .min_size(egui::vec2(220.0, 140.0))
                .max_size(egui::vec2(max_width, max_height))
                .show(ctx, |ui| {
                    enlarge_notice(ui);
                    ux::dialog_body(ui);
                    let mut detail = self.notification_text();
                    if detail.is_empty() {
                        detail = "通知はない。".into();
                    }
                    // Lay out the real footer first. Estimating its height caused a
                    // positive feedback loop: content overflow enlarged every frame.
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                        if missing_codex_path(&self.message) {
                            fix_path = ui.add_enabled(self.qwen_settings.is_none(),
                                crate::app::controls::Button::new("設定の実行ファイル欄へ"))
                                .on_disabled_hover_text("先にQwen接続を保存するか、Qwen編集をキャンセルしてください。")
                                .clicked();
                        }
                        if self.daily_limit_action_applies() {
                            fix_daily_limit = ui.add_enabled(self.qwen_settings.is_none(),
                                crate::app::controls::Button::new("生成・添削の上限設定へ"))
                                .on_disabled_hover_text("先にQwen接続を保存するか、Qwen編集をキャンセルしてください。")
                                .clicked();
                        }
                        if let Some(notice) = &self.material_notice {
                            close_material = ui.ww_button("閉じる").clicked();
                            if ui.ww_button("診断情報をコピー").clicked() {
                                ui.ctx().copy_text(diagnostic_copy_text(&notice.diagnostic));
                            }
                        } else if ui.ww_button("詳細をコピー").clicked() {
                            ui.ctx().copy_text(detail.clone());
                        }
                        let body = ui.available_rect_before_wrap();
                        ui.allocate_rect(body, egui::Sense::hover());
                        ui.painter().rect_filled(body, 2.0, Color32::WHITE);
                        ui.painter().rect_stroke(
                            body,
                            2.0,
                            egui::Stroke::new(1.0_f32, Color32::LIGHT_GRAY),
                            egui::StrokeKind::Inside,
                        );
                        let inner = body.shrink(6.0);
                        let mut content = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(inner)
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                        );
                        content.set_clip_rect(inner.intersect(ui.clip_rect()));
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(inner.height().max(1.0))
                            .show(&mut content, |ui| {
                                if let Some(notice) = self.material_notice.as_mut() {
                                    ui.add(egui::Label::new(RichText::new(material_title(notice.diagnostic.stage))
                                        .size(24.0).strong()).wrap().selectable(true));
                                    ui.label("教材には登録していない。既存教材も変更していない。");
                                    ui.add_space(6.0);
                                    ui.label(RichText::new("理由").strong());
                                    ui.label(material_reason(notice.diagnostic.cause));
                                    ui.add_space(6.0);
                                    ui.label(RichText::new("次にできること").strong());
                                    ui.label("元の発言と対象項目を確認し、必要なら教材案の作成を明示的に再試行する。");
                                    if ui.ww_button(if notice.comparison_open { "引用と元の回答を閉じる" } else { "引用と元の回答を確認" }).clicked() {
                                        notice.comparison_open = !notice.comparison_open;
                                    }
                                    if notice.comparison_open {
                                        ui.label("作成を依頼した時点の固定元発言を表示する。読みやすい表示は原文の厳密一致を示さない。");
                                        if ui.ww_button(if notice.raw_mode { "読みやすい表示へ" } else { "原文を表示" }).clicked() {
                                            notice.raw_mode = !notice.raw_mode;
                                        }
                                        ui.label(if notice.raw_mode { "原文表示" } else { "読みやすい表示" });
                                        if let Some(number) = notice.diagnostic.reason_number {
                                            ui.label(format!("AI応答内の{number}番目の変更理由"));
                                        }
                                        if let Some(label) = &notice.diagnostic.target_label {
                                            ui.label(format!("対象項目: {label}"));
                                        } else {
                                            ui.label("対象項目を特定できない。");
                                        }
                                        if let Some(evidence) = &notice.diagnostic.evidence {
                                            if let Some(turn) = evidence.exchange_index.checked_add(1) {
                                                let speaker = match evidence.role.as_str() {
                                                    "user" => "あなたの質問", "assistant" => "Codexの回答",
                                                    _ => "話者を特定できない",
                                                };
                                                ui.label(format!("{turn}組目・{speaker}"));
                                            } else { ui.label("往復番号を特定できない。"); }
                                            ui.separator();
                                            ui.label(RichText::new("AIが示した引用").strong());
                                            if notice.raw_mode {
                                                ui.add(egui::Label::new(&evidence.quote).wrap().selectable(true));
                                            } else { show_markdown(ui,
                                                egui::Id::new("material-quote"), &evidence.quote); }
                                            if ui.ww_button("AI引用の原文をコピー").clicked() {
                                                ui.ctx().copy_text(evidence.quote.clone());
                                            }
                                            ui.add_space(6.0);
                                            ui.separator();
                                            ui.label(RichText::new("作成を依頼した時点の元の発言").strong());
                                            if let Some(original) = &evidence.original {
                                                if notice.raw_mode {
                                                    ui.add(egui::Label::new(original).wrap().selectable(true));
                                                } else { show_markdown(ui,
                                                    egui::Id::new("material-original"), original); }
                                                if ui.ww_button("元の発言の原文をコピー").clicked() {
                                                    ui.ctx().copy_text(original.clone());
                                                }
                                            } else {
                                                ui.label("元の発言を取得できない。");
                                            }
                                        } else {
                                            ui.label("引用の照合前に止まったため、比較する発言はない。");
                                        }
                                    }
                                    if ui.ww_button(if notice.technical_open { "技術情報を閉じる" } else { "技術情報を確認" }).clicked() {
                                        notice.technical_open = !notice.technical_open;
                                    }
                                    if notice.technical_open {
                                        ui.add(egui::Label::new(format!("原因詳細: {}", notice.diagnostic.cause_detail)).wrap().selectable(true));
                                        ui.add(egui::Label::new(diagnostic_copy_text(&notice.diagnostic)).wrap().selectable(true));
                                    }
                                    if !self.font_notice.is_empty() {
                                        ui.separator();
                                        ui.label("表示フォントの注意");
                                        ui.add(egui::Label::new(&self.font_notice).wrap().selectable(true));
                                    }
                                } else {
                                    ui.add(egui::Label::new(&detail).wrap().selectable(true));
                                }
                            });
                    });
                });
        if let Some(shown) = shown {
            ctx.graphics_mut(|graphics| {
                let list = graphics.entry(shown.response.layer_id);
                for index in 0..list.next_idx().0 {
                    list.mutate_shape(egui::layers::ShapeIdx(index), |shape| {
                        if let egui::Shape::Text(text) = &mut shape.shape {
                            if text.galley.text() == "通知の詳細"
                                && text.galley.mesh_bounds.is_positive()
                            {
                                text.pos.y += text.galley.rect.center().y
                                    - text.galley.mesh_bounds.center().y;
                            }
                        }
                    });
                }
            });
        }
        self.notification_open = open && !close_material;
        if fix_path {
            self.open_codex_path_guidance();
        }
        if fix_daily_limit {
            self.open_daily_limit_guidance();
        }
    }

    pub(super) fn operation_notice(&mut self, ctx: &egui::Context) {
        let path_issue = missing_codex_path(&self.message);
        if self.fatal.is_none() && self.pending.is_none() && !self.batch_running && !path_issue {
            return;
        }
        egui::TopBottomPanel::top("operation-notice").show(ctx, |ui| {
            enlarge_notice(ui);
            if let Some(error) = &self.fatal {
                ui.colored_label(
                    Color32::from_rgb(165, 45, 30),
                    "保存・学習を停止している。記録の保全が必要である。",
                );
                egui::ScrollArea::vertical()
                    .id_salt("fatal-notice-detail")
                    .max_height(64.0)
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(error).wrap());
                    });
                if ui
                    .ww_button("設定で記録のエクスポート・復元を確認")
                    .clicked()
                {
                    self.page = Page::Settings;
                }
            }
            if path_issue {
                ui.horizontal_wrapped(|ui| {
                    let font = super::chrome::status_font();
                    let galley = ui.painter().layout_no_wrap(
                        "Codexの実行ファイルを確認する必要がある。".into(),
                        font,
                        ui.visuals().text_color(),
                    );
                    let height = ui
                        .spacing()
                        .interact_size
                        .y
                        .max(galley.size().y + 2.0 * ui.spacing().button_padding.y);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(galley.size().x, height),
                        egui::Sense::hover(),
                    );
                    let pos =
                        egui::pos2(rect.left(), rect.center().y - galley.mesh_bounds.center().y);
                    ui.painter().galley(pos, galley, ui.visuals().text_color());
                    if ui
                        .add_enabled(
                            self.qwen_settings.is_none(),
                            crate::app::controls::Button::new("設定の実行ファイル欄へ"),
                        )
                        .on_disabled_hover_text(
                            "先にQwen接続を保存するか、Qwen編集をキャンセルしてください。",
                        )
                        .clicked()
                    {
                        self.open_codex_path_guidance();
                    }
                });
            }
            if self.pending.is_some() || self.batch_running {
                ui.horizontal_wrapped(|ui| {
                    ui.small(self.activity_label());
                    let cancellable = self.pending.as_ref().is_some_and(|p| p.cancel.is_some());
                    if (cancellable || self.batch_running)
                        && ui.ww_button("生成・通信の中断を要求").clicked()
                    {
                        self.batch_running = false;
                        self.fetch_then_generate = false;
                        if let Some(cancel) = self.pending.as_ref().and_then(|p| p.cancel.as_ref())
                        {
                            cancel.store(true, Ordering::Relaxed);
                        }
                        self.message = "中断中… 登録済み教材と未処理の語は保持する。".into();
                    }
                    if self.fetch_then_generate
                        && !cancellable
                        && ui.ww_button("取得後の自動生成を取り消す").clicked()
                    {
                        self.fetch_then_generate = false;
                        self.message =
                            "語彙一覧の取得は続ける。取得後の自動生成は取り消した。".into();
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordweave5::run_journal::{Outcome, RunRecord};

    fn feedback_ui_draw(
        ctx: &egui::Context,
        app: &mut WordApp,
        size: egui::Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| app.update_ui(ctx),
        )
    }

    fn feedback_ui_visible_text(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
        fn find(shape: &egui::Shape, label: &str) -> Option<egui::Rect> {
            match shape {
                egui::Shape::Text(text) if text.galley.text() == label => {
                    Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
                }
                egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, label)),
                _ => None,
            }
        }
        output.shapes.iter().find_map(|shape| {
            let rect = find(&shape.shape, label)?;
            shape.clip_rect.contains_rect(rect).then_some(rect)
        })
    }

    fn feedback_ui_click(
        ctx: &egui::Context,
        app: &mut WordApp,
        size: egui::Vec2,
        pos: egui::Pos2,
    ) -> egui::FullOutput {
        feedback_ui_draw(
            ctx,
            app,
            size,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        feedback_ui_draw(
            ctx,
            app,
            size,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
    }

    fn feedback_ui_diagnostic_line_visible(output: &egui::FullOutput, marker: &str) -> bool {
        fn found(shape: &egui::Shape, clip: egui::Rect, marker: &str) -> bool {
            match shape {
                egui::Shape::Text(text) if text.galley.text().contains("結果") => {
                    text.galley.rows.iter().any(|row| {
                        let row_text: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                        row_text.contains(marker)
                            && clip.contains_rect(row.rect.translate(text.pos.to_vec2()))
                    })
                }
                egui::Shape::Vec(parts) => parts.iter().any(|part| found(part, clip, marker)),
                _ => false,
            }
        }
        output
            .shapes
            .iter()
            .any(|shape| found(&shape.shape, shape.clip_rect, marker))
    }

    #[test]
    fn feedback_ui_quote_long_notification_scroll_reaches_remedy_with_fixed_copy_footer() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let diagnostic = super::super::visual_check::synthetic_quote_diagnostic(&app.deck[0]);
        assert!(diagnostic.contains("対処") && diagnostic.contains("固定原文"));
        let before_progress = serde_json::to_value(&app.progress).unwrap();
        let before_deck = serde_json::to_value(&app.deck).unwrap();
        app.notify_error(diagnostic.clone());
        app.notification_open = true;
        let size = egui::vec2(820.0 / 1.6, 650.0 / 1.6);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
        let mut output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        for _ in 0..4 {
            output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let window_id = egui::Id::new("notification-details-v2");
        let window = ctx
            .memory(|memory| memory.area_rect(window_id))
            .expect("notification window");
        assert!(
            screen.contains_rect(window),
            "whole quote Window escapes narrow screen: {window:?}"
        );
        let footer = feedback_ui_visible_text(&output, "詳細をコピー")
            .expect("copy footer initially visible");
        assert!(window.contains_rect(footer) && screen.contains_rect(footer));
        let pointer = window.left_top() + egui::vec2(window.width() * 0.55, window.height() * 0.45);
        let mut reached_remedy = feedback_ui_diagnostic_line_visible(&output, "対処");
        for _ in 0..12 {
            output = feedback_ui_draw(
                &ctx,
                &mut app,
                size,
                vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -120.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            reached_remedy |= feedback_ui_diagnostic_line_visible(&output, "対処");
            let current = ctx.memory(|memory| memory.area_rect(window_id)).unwrap();
            assert!(
                screen.contains_rect(current),
                "scroll moved whole quote Window offscreen: {current:?}"
            );
            let footer = feedback_ui_visible_text(&output, "詳細をコピー")
                .expect("copy footer lost during scroll");
            assert!(
                current.contains_rect(footer) && screen.contains_rect(footer),
                "copy footer clipped during scroll"
            );
        }
        assert!(reached_remedy, "real scroll never exposed the remedy line");
        let footer = feedback_ui_visible_text(&output, "詳細をコピー").unwrap();
        let copied = feedback_ui_click(&ctx, &mut app, size, footer.center());
        assert!(copied.platform_output.commands.iter().any(|command|
            matches!(command, egui::OutputCommand::CopyText(text) if text == &diagnostic)),
            "copy action did not emit the exact diagnostic");
        assert_eq!(
            serde_json::to_value(&app.progress).unwrap(),
            before_progress
        );
        assert_eq!(serde_json::to_value(&app.deck).unwrap(), before_deck);
        assert!(app.pending.is_none());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_daily_limit_real_click_reaches_visible_slider_once_at_both_scales() {
        for size in [
            egui::vec2(1120.0, 850.0),
            egui::vec2(820.0 / 1.6, 650.0 / 1.6),
        ] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            app.progress.settings.ai_daily_limit = 1;
            app.progress.ai_calls.insert(today(), 1);
            app.answer = "未送信の学習回答".into();
            app.progress.chats[0].draft = "未送信の会話入力".into();
            app.page = Page::Settings;
            app.begin_settings_edit();
            app.settings_editor.draft.ai_daily_limit = 800;
            let before = serde_json::to_value(&app.progress).unwrap();
            assert!(!app.reserve_generation());
            let mut output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
            for _ in 0..8 {
                output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
            }
            let action = feedback_ui_visible_text(&output, "生成・添削の上限設定へ")
                .expect("eligible refusal action is visible");
            feedback_ui_click(&ctx, &mut app, size, action.center());
            assert!(app.page == Page::Settings);
            assert_eq!(
                app.settings_section,
                settings_ui::SettingsSection::Connection
            );
            assert!(app.daily_limit_guidance && !app.notification_open);
            assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
            assert_eq!(app.answer, "未送信の学習回答");
            assert_eq!(app.progress.chats[0].draft, "未送信の会話入力");
            assert_eq!(app.settings_editor.draft.ai_daily_limit, 800);
            for _ in 0..8 {
                feedback_ui_draw(&ctx, &mut app, size, vec![]);
            }
            let (rect, id, clip, enabled): (egui::Rect, egui::Id, egui::Rect, bool) =
                ctx.data(|data| {
                    data.get_temp(egui::Id::new("feedback-ui-daily-limit-slider"))
                        .expect("limit slider drawn")
                });
            assert!(
                enabled && clip.contains(rect.center()),
                "limit slider is offscreen or disabled: {rect:?} {clip:?}"
            );
            assert!(
                !app.daily_limit_focus_pending,
                "one-time focus remains pending after visible slider"
            );
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(id));
            ctx.memory_mut(|memory| memory.surrender_focus(id));
            feedback_ui_draw(&ctx, &mut app, size, vec![]);
            assert_ne!(
                ctx.memory(|memory| memory.focused()),
                Some(id),
                "guidance stole focus twice"
            );
            assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn feedback_ui_daily_limit_learning_refusal_and_stale_click_preserve_drafts_and_count() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        app.start(5);
        app.answer = "学習解答の下書き".into();
        app.progress.chats[0].draft = "会話下書き".into();
        let before = serde_json::to_value(&app.progress).unwrap();
        app.launch_ai(0);
        assert!(
            app.daily_limit_notice.is_some(),
            "learning refusal did not use daily-limit notice"
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(app.answer, "学習解答の下書き");
        assert_eq!(app.progress.chats[0].draft, "会話下書き");
        assert!(app.pending.is_none());
        let size = egui::vec2(1120.0, 850.0);
        let mut output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        for _ in 0..8 {
            output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let stale_action = feedback_ui_visible_text(&output, "生成・添削の上限設定へ")
            .expect("learning refusal action visible")
            .center();
        app.progress.ai_calls.insert(today(), 0);
        feedback_ui_click(&ctx, &mut app, size, stale_action);
        assert!(
            !app.daily_limit_guidance,
            "stale click navigated after count changed"
        );
        assert!(app.page == Page::Study);
        assert_eq!(app.progress.ai_calls[&today()], 0);
        assert_eq!(app.answer, "学習解答の下書き");
        assert_eq!(app.progress.chats[0].draft, "会話下書き");
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_daily_limit_guidance_waits_for_palette_cancel_before_enabling_settings() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        app.begin_color_editor();
        app.color_editor.as_mut().unwrap().draft.page = wordweave5::store::TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        let before = serde_json::to_value(&app.progress).unwrap();
        assert!(!app.reserve_generation());
        app.open_daily_limit_guidance();
        assert!(app.daily_limit_guidance && app.daily_limit_focus_pending);
        let size = egui::vec2(1120.0, 850.0);
        for _ in 0..8 {
            feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let (_, _, clip, enabled): (egui::Rect, egui::Id, egui::Rect, bool) = ctx.data(|data| {
            data.get_temp(egui::Id::new("feedback-ui-daily-limit-slider"))
                .expect("guided slider drawn")
        });
        assert!(
            !enabled && clip.is_positive(),
            "palette preview must disable the limit editor"
        );
        assert!(
            app.daily_limit_focus_pending,
            "disabled slider consumed the one-time focus"
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        app.cancel_color_editor();
        for _ in 0..8 {
            feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let (rect, id, clip, enabled): (egui::Rect, egui::Id, egui::Rect, bool) =
            ctx.data(|data| {
                data.get_temp(egui::Id::new("feedback-ui-daily-limit-slider"))
                    .unwrap()
            });
        assert!(enabled && clip.contains(rect.center()));
        assert!(!app.daily_limit_focus_pending);
        assert_eq!(ctx.memory(|memory| memory.focused()), Some(id));
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_daily_limit_refusal_shows_settings_action_without_changing_drafts_or_count() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        app.answer = "未送信の学習回答".into();
        app.progress.chats[0].draft = "未送信の会話入力".into();
        app.page = Page::Settings;
        app.begin_settings_edit();
        app.settings_editor.draft.ai_daily_limit = 1000;
        let before = serde_json::to_value(&app.progress).unwrap();
        assert!(!app.reserve_generation());
        super::super::harness_tests::frame(&ctx, &mut app, false);
        let output = super::super::harness_tests::frame(&ctx, &mut app, false);
        let mut drawn = String::new();
        for shape in &output.shapes {
            super::super::harness_tests::shape_text(&shape.shape, &mut drawn);
        }
        assert!(
            drawn.contains("生成・添削の上限設定へ"),
            "eligible daily refusal needs a visible action: {drawn}"
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(app.answer, "未送信の学習回答");
        assert_eq!(app.progress.chats[0].draft, "未送信の会話入力");
        assert_eq!(app.settings_editor.draft.ai_daily_limit, 1000);
        assert!(app.pending.is_none());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_daily_limit_action_disappears_for_stale_or_unrelated_notice() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        assert!(!app.reserve_generation());
        for (count, limit, replace) in [
            (0, 1, false),
            (1, 1000, false),
            (1000, 1, false),
            (1, 1, true),
        ] {
            app.progress.ai_calls.insert(today(), count);
            app.progress.settings.ai_daily_limit = limit;
            if replace {
                app.notify_warning("合成の通信失敗");
            }
            app.notification_open = true;
            super::super::harness_tests::frame(&ctx, &mut app, false);
            let output = super::super::harness_tests::frame(&ctx, &mut app, false);
            let mut drawn = String::new();
            for shape in &output.shapes {
                super::super::harness_tests::shape_text(&shape.shape, &mut drawn);
            }
            assert!(
                !drawn.contains("生成・添削の上限設定へ"),
                "stale notice gained action: {drawn}"
            );
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn deliver_recovery(
        ctx: &egui::Context,
        app: &mut WordApp,
        outcome: Outcome,
        response: Option<&str>,
    ) {
        let (tx, rx) = std::sync::mpsc::channel();
        app.pending = Some(Pending {
            kind: Activity::Recovery,
            key: app.key(),
            rx,
            cancel: None,
        });
        tx.send(Ok(AiResult::Recovered(RunRecord {
            id: "1-1".into(),
            at: 0,
            outcome,
            thread_id: None,
            turn_id: None,
            execution: wordweave5::execution::Execution::default(),
            request: serde_json::json!({}),
            response: response.map(str::to_owned),
            note: String::new(),
        })))
        .unwrap();
        app.tick(ctx);
        assert!(app.pending.is_none(), "recovery result must be consumed");
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
    }

    #[test]
    fn recovered_outcome_and_response_matrix_distinguishes_completion_from_saved_text() {
        let outcomes = [
            Outcome::Prepared,
            Outcome::Submitted,
            Outcome::Completed,
            Outcome::Failed,
            Outcome::Interrupted,
            Outcome::Unknown,
        ];
        let responses = [None, Some(""), Some(" \n\t　"), Some(" 合成の秘密本文 ")];
        for outcome in outcomes {
            for response in responses {
                let (ctx, mut app, root) = super::super::harness_tests::fixture();
                app.answer = "未完了の回答".into();
                app.progress.chats[0].draft = "未送信の会話".into();
                let progress = serde_json::to_value(&app.progress).unwrap();
                let deck = serde_json::to_value(&app.deck).unwrap();
                let has_body = response.is_some_and(|s| !s.trim().is_empty());
                let completed_with_body = outcome == Outcome::Completed && has_body;

                deliver_recovery(&ctx, &mut app, outcome, response);
                let detail = app.notification_text();
                assert!(
                    detail.contains(outcome.label()),
                    "{outcome:?} {response:?}: {detail}"
                );
                assert!(
                    detail.contains("教材・会話には自動反映していない"),
                    "{detail}"
                );
                assert!(
                    !detail.contains("合成の秘密本文"),
                    "response must not be copied into notification: {detail}"
                );
                assert_eq!(
                    app.notification_open, !completed_with_body,
                    "{outcome:?} {response:?}: {detail}"
                );
                assert!(
                    !app.notification_is_error(),
                    "a recovery result is not a red error: {detail}"
                );
                if has_body {
                    assert!(
                        detail.contains("実行記録で確認できる"),
                        "{outcome:?}: {detail}"
                    );
                    if outcome != Outcome::Completed {
                        assert!(
                            detail.contains("完了") && detail.contains("確認できていない"),
                            "{outcome:?}: {detail}"
                        );
                    }
                } else {
                    assert!(
                        detail.contains("本文") && detail.contains("取得できていない"),
                        "{outcome:?}: {detail}"
                    );
                    assert!(
                        !detail.contains("本文は実行記録で確認できる"),
                        "{outcome:?}: {detail}"
                    );
                    if outcome == Outcome::Interrupted {
                        assert!(
                            detail.contains("中断") && detail.contains("完成した本文"),
                            "{detail}"
                        );
                    }
                }
                assert_eq!(
                    serde_json::to_value(&app.progress).unwrap(),
                    progress,
                    "learning and chat data changed"
                );
                assert_eq!(
                    serde_json::to_value(&app.deck).unwrap(),
                    deck,
                    "registered material changed"
                );
                assert_eq!(app.answer, "未完了の回答");
                assert!(
                    !app.batch_running && app.pending.is_none(),
                    "recovery must not start generation"
                );
                drop(app);
                std::fs::remove_dir_all(root).unwrap();
            }
        }
    }

    #[test]
    fn recovered_attention_reopens_after_dismissal_and_success_clears_old_alerts() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        deliver_recovery(&ctx, &mut app, Outcome::Interrupted, None);
        assert!(app.notification_open);
        assert!(!app.notification_attention.is_empty());
        app.notification_open = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(
            !app.notification_open,
            "dismissal must hold without a new recovery"
        );
        deliver_recovery(&ctx, &mut app, Outcome::Interrupted, None);
        assert!(
            app.notification_open,
            "the same interrupted result is a new alert"
        );

        app.notification_open = false;
        app.notify_error("以前の失敗".into());
        deliver_recovery(&ctx, &mut app, Outcome::Completed, Some("保存済み本文"));
        assert!(
            !app.notification_open,
            "successful recovery must not open a new alert"
        );
        assert!(app.notification_error.is_empty());
        assert!(app.notification_attention.is_empty());
        assert!(!app.notification_is_error());
        assert!(!app.notification_text().contains("以前の失敗"));
        app.notify_blocked("以前の注意");
        deliver_recovery(&ctx, &mut app, Outcome::Completed, Some("保存済み本文"));
        assert!(!app.notification_open);
        assert!(app.notification_attention.is_empty());
        assert!(!app.notification_text().contains("以前の注意"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_scan_failure_takes_precedence_over_recovery_notice() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.storage = None;
        deliver_recovery(&ctx, &mut app, Outcome::Completed, Some("保存済み本文"));
        assert!(
            app.notification_is_error(),
            "later scan error must be visible"
        );
        assert!(app.notification_open);
        assert!(app.notification_text().contains("保存先がありません"));
        assert!(!app
            .notification_text()
            .contains("本文は実行記録で確認できる"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_limit_opens_notice_on_each_attempt_without_consuming_quota() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        let before = serde_json::to_value(&app.progress).unwrap();
        for _ in 0..2 {
            assert!(!app.reserve_generation());
            assert!(app.message.contains("本日の生成上限"));
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.notification_window(ctx)
            });
            assert!(app.notification_open, "a rejected request must explain why");
            assert!(
                !app.notification_is_error(),
                "a quota limit is not an error"
            );
            app.notification_open = false;
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.notification_window(ctx)
            });
            assert!(
                !app.notification_open,
                "dismissal lasts until another attempt"
            );
        }
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert!(app.pending.is_none());
        app.message = "発言をコピーした。".into();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(!app.notification_open);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_generation_targets_opens_notice_without_starting_work() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.begin_batch(vec![]);
        assert!(app.message.contains("すべて自動生成・登録済み"));
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(app.notification_open);
        assert!(!app.batch_running && app.pending.is_none());
        assert!(!app.notification_is_error());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_information_does_not_open_but_errors_do() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        for message in [
            "発言をクリップボードへコピーした。",
            "設定を保存した。",
            "接続成功",
        ] {
            app.message = message.into();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.notification_window(ctx)
            });
            assert!(
                !app.notification_open,
                "information must not interrupt: {message}"
            );
        }
        app.message = "接続に失敗した。".into();
        app.notification_error = app.message.clone();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(app.notification_open);
        app.notification_open = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(
            !app.notification_open,
            "dismissed alert must stay dismissed"
        );
        app.notify_error("接続に失敗した。".into());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(
            app.notification_open,
            "a new failure must reopen even with the same text"
        );
        app.notification_open = false;
        app.notify_warning("実行記録の一部を読み込めなかった。");
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(app.notification_open);
        assert!(!app.notification_is_error());
        app.notification_open = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(!app.notification_open);
        // Informational details remain available when explicitly opened.
        app.message = "発言をクリップボードへコピーした。".into();
        app.notification_open = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.notification_window(ctx)
        });
        assert!(app.notification_open);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_failure_warning_and_fatal_are_not_lost_or_reopened_by_copy() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let frame = |app: &mut WordApp| {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.notification_window(ctx)
            });
        };
        app.notify_result(Err("保存に失敗した。".into()));
        app.batch_running = true;
        frame(&mut app);
        assert!(!app.notification_open, "defer alerts while work is active");
        app.batch_running = false;
        frame(&mut app);
        assert!(app.notification_open && app.notification_is_error());
        app.notification_open = false;
        app.notify_result(Ok("保存した。".into()));
        frame(&mut app);
        assert!(!app.notification_open);
        app.notify_warning("録音の一部だけを保持した。");
        frame(&mut app);
        assert!(app.notification_open);
        app.notification_open = false;
        app.fatal = Some("保存先を開けない。".into());
        frame(&mut app);
        assert!(app.notification_open);
        app.notification_open = false;
        app.message = "発言をコピーした。".into();
        frame(&mut app);
        assert!(
            !app.notification_open,
            "a copy must not reopen the same fatal alert"
        );
        app.fatal = None;
        app.font_notice = "フォントを代替した。".into();
        frame(&mut app);
        assert!(app.notification_open);
        app.notification_open = false;
        app.message = "別の発言をコピーした。".into();
        frame(&mut app);
        assert!(!app.notification_open);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_glyphs_align_with_status_row() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        for zoom in [0.8, 1.0, 1.6] {
            ctx.set_zoom_factor(zoom);
            for message in ["", "指定したCodex実行ファイルがありません。"] {
                app.message = message.into();
                for _ in 0..3 {
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(1150.0, 950.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::TopBottomPanel::bottom("status")
                                .show(ctx, |ui| app.status_summary(ui));
                        },
                    );
                    let ink = |prefix: &str| {
                        output
                            .shapes
                            .iter()
                            .find_map(|s| match &s.shape {
                                egui::Shape::Text(t) if t.galley.text().starts_with(prefix) => {
                                    Some(t.galley.mesh_bounds.translate(t.pos.to_vec2()))
                                }
                                _ => None,
                            })
                            .unwrap()
                    };
                    let notice = ink(if message.is_empty() {
                        "○ 通知"
                    } else {
                        "● 通知"
                    });
                    let status = ink("接続");
                    assert!(
                        (notice.center().y - status.center().y).abs() < 1.5,
                        "{notice:?} {status:?}"
                    );
                }
            }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_window_stays_stable_and_resizes_in_both_directions() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(1.0);
        app.message = "指定したCodex実行ファイルがありません。".into();
        app.notification_open = true;
        let frame = |app: &mut WordApp, events| {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 1000.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| app.notification_window(ctx),
            );
            ctx.memory(|m| {
                m.area_rect(egui::Id::new("notification-details-v2"))
                    .unwrap()
            })
        };
        for _ in 0..8 {
            frame(&mut app, vec![]);
        }
        let original = frame(&mut app, vec![]);
        for _ in 0..80 {
            let rect = frame(&mut app, vec![]);
            assert!(
                (rect.height() - original.height()).abs() < 1.0,
                "{original:?} -> {rect:?}"
            );
        }
        for delta in [egui::vec2(80.0, 100.0), egui::vec2(-60.0, -70.0)] {
            let before = frame(&mut app, vec![]);
            let pos = before.max - egui::vec2(3.0, 3.0);
            let event = |pos, pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            frame(&mut app, vec![egui::Event::PointerMoved(pos)]);
            frame(&mut app, vec![event(pos, true)]);
            frame(&mut app, vec![egui::Event::PointerMoved(pos + delta)]);
            frame(&mut app, vec![event(pos + delta, false)]);
            let after = frame(&mut app, vec![]);
            assert!(
                (after.height() - before.height()) * delta.y.signum() > 30.0,
                "{before:?} -> {after:?}"
            );
            for _ in 0..40 {
                let rect = frame(&mut app, vec![]);
                assert!((rect.height() - after.height()).abs() < 1.0);
            }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_font_matches_status_and_is_local() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let before = ui.style().text_styles.clone();
                ui.scope(|ui| {
                    enlarge_notice(ui);
                    for key in before.keys() {
                        assert_eq!(
                            ui.style().text_styles[key],
                            super::super::chrome::status_font()
                        );
                    }
                });
                assert_eq!(ui.style().text_styles, before);
            });
        });
    }

    #[test]
    fn only_known_executable_errors_offer_path_recovery() {
        assert!(missing_codex_path("指定したCodex実行ファイルがありません。設定で実在するファイルを選ぶか、実行ファイル欄をcodexに変更してPATHから自動検出してください。"));
        assert!(missing_codex_path("Codexが見つかりません。Codex CLIをインストールし、codex loginでChatGPTログイン後、必要なら設定でcodex.exeを選択してください。"));
        for error in [
            "認証に失敗しました",
            "ファイルが見つかりません",
            "通信エラー",
            "回答の保存先の会話が見つかりません。",
        ] {
            assert!(!missing_codex_path(error));
        }
    }

    #[test]
    fn recovery_navigation_preserves_user_input() {
        let (_ctx, mut app, root) = super::super::harness_tests::fixture();
        app.start(5);
        assert!(app.session.is_some());
        app.answer = "unfinished answer".into();
        app.ink.strokes = vec![vec![[0.1, 0.2], [0.3, 0.4]]];
        app.message = "指定したCodex実行ファイルがありません。".into();
        app.notification_open = true;
        app.open_codex_path_guidance();
        assert!(app.page == Page::Settings);
        assert!(app.codex_path_guidance && app.codex_path_focus_pending);
        assert!(!app.notification_open);
        assert!(app.session.is_some());
        assert_eq!(app.answer, "unfinished answer");
        assert_eq!(app.ink.strokes, vec![vec![[0.1, 0.2], [0.3, 0.4]]]);
        assert!(app.pending.is_none());
        assert!(app.connection_check.is_none());
        assert_eq!(app.message, "指定したCodex実行ファイルがありません。");
        assert!(app.notification_is_error());
        // Connection results belong to the app, not the preserved study question.
        assert!(!app.key().is_empty());
        for result in [
            Ok(AiResult::Connection("接続成功".into())),
            Err("接続失敗".into()),
        ] {
            let (tx, rx) = std::sync::mpsc::channel();
            let success = result.is_ok();
            app.connection_check = Some(("codex".into(), false));
            app.pending = Some(Pending {
                key: String::new(),
                rx,
                cancel: None,
                kind: Activity::Connection,
            });
            tx.send(result).unwrap();
            app.tick(&_ctx);
            assert!(app.pending.is_none());
            assert_eq!(app.connection_check.as_ref().unwrap().1, success);
            assert_eq!(
                app.message,
                if success {
                    "接続成功"
                } else {
                    "接続失敗"
                }
            );
            assert_eq!(app.notification_is_error(), !success);
            assert!(app.session.is_some());
            assert_eq!(app.answer, "unfinished answer");
            assert_eq!(app.ink.strokes, vec![vec![[0.1, 0.2], [0.3, 0.4]]]);
        }
        app.message = "設定を保存した。".into();
        assert!(!app.notification_is_error());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn material_quote_failure_opens_actual_notice_without_replacing_draft_or_learning_data() {
        use wordweave5::{
            chat::Conversation,
            execution::Execution,
            material::{Mode, Request},
        };
        for retain_existing_draft in [false, true] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            let old = app.deck[0].clone();
            let mut conversation = Conversation::new();
            conversation
                .complete(
                    "Could this be clearer?".into(),
                    "Use **improve clarity** in the answer.".into(),
                    Execution::default(),
                )
                .unwrap();
            conversation.exchanges[0].for_material = true;
            let request =
                Request::new(&conversation, &old.base, Mode::Correct, Some(old.clone())).unwrap();
            let mut candidate = old;
            candidate.usage = "合成の訂正案".into();
            let response = |quote: &str| {
                serde_json::json!({"entry":candidate,"reasons":[{
                "path":"/usage","reason":"合成回答に基づく訂正", "quotes":[{"exchange_index":0,"role":"assistant","quote":quote}]
            }]}).to_string()
            };
            app.progress.material_draft = if retain_existing_draft {
                Some(
                    request
                        .build_response(&response("**improve clarity**"))
                        .unwrap(),
                )
            } else {
                None
            };
            let error = request
                .build_response(&response("Use improve clarity in the answer."))
                .unwrap_err();
            let before_progress = serde_json::to_value(&app.progress).unwrap();
            let before_deck = serde_json::to_value(&app.deck).unwrap();
            let before_open = app.chat_material_open;
            let before_same_base = app.material_same_base;
            let (tx, rx) = std::sync::mpsc::channel();
            app.pending = Some(Pending {
                kind: Activity::Material,
                key: app.key(),
                rx,
                cancel: None,
            });
            tx.send(Err(error.clone())).unwrap();
            app.tick(&ctx);
            assert!(app.pending.is_none());
            assert_eq!(app.message, error);
            assert!(app.notification_is_error());
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.notification_window(ctx)
            });
            assert!(app.notification_open, "教材案の拒否理由が自動表示される");
            for expected in [
                "往復 1",
                "Codex（AIの回答）",
                "AIが返した引用",
                "作成要求時の元発言",
            ] {
                assert!(
                    app.notification_text().contains(expected),
                    "{expected}: {}",
                    app.notification_text()
                );
            }
            assert_eq!(
                serde_json::to_value(&app.progress).unwrap(),
                before_progress,
                "教材案・学習記録・会話の保持"
            );
            assert_eq!(
                serde_json::to_value(&app.deck).unwrap(),
                before_deck,
                "既存教材の保持"
            );
            assert_eq!(
                app.chat_material_open, before_open,
                "案成功の表示状態にしない"
            );
            assert_eq!(
                app.material_same_base, before_same_base,
                "登録選択を変更しない"
            );
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    fn u5_diagnostic(
        baseline: &wordweave5::model::Entry,
        original: &str,
        quote: &str,
    ) -> MaterialDiagnostic {
        use wordweave5::{
            chat::Conversation,
            execution::Execution,
            material::{Mode, Request},
        };
        let mut conversation = Conversation::new();
        conversation
            .complete("合成の質問".into(), original.into(), Execution::default())
            .unwrap();
        conversation.exchanges[0].for_material = true;
        let request = Request::new(
            &conversation,
            &baseline.base,
            Mode::Correct,
            Some(baseline.clone()),
        )
        .unwrap();
        let mut candidate = baseline.clone();
        candidate.usage = "合成の変更".into();
        let response = serde_json::json!({"entry":candidate,"reasons":[{"path":"/usage",
            "reason":"合成理由","quotes":[{"exchange_index":0,"role":"assistant","quote":quote}]}]})
        .to_string();
        match request.build_response_detailed(&response).unwrap_err() {
            MaterialFailure::Evidence(detail) => detail,
            other => panic!("expected evidence failure, got {other:?}"),
        }
    }

    fn u5_draw_notice(
        ctx: &egui::Context,
        app: &mut WordApp,
        size: egui::Vec2,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| app.notification_window(ctx),
        )
    }

    fn u5_notice_text(output: &egui::FullOutput) -> String {
        let mut text = String::new();
        for shape in &output.shapes {
            super::super::harness_tests::shape_text(&shape.shape, &mut text);
        }
        text
    }

    fn u5_click_notice(
        ctx: &egui::Context,
        app: &mut WordApp,
        size: egui::Vec2,
        pos: egui::Pos2,
    ) -> egui::FullOutput {
        let event = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        u5_draw_notice(
            ctx,
            app,
            size,
            vec![egui::Event::PointerMoved(pos), event(true)],
        );
        u5_draw_notice(
            ctx,
            app,
            size,
            vec![egui::Event::PointerMoved(pos), event(false)],
        )
    }

    fn u5_visible_row(output: &egui::FullOutput, marker: &str) -> bool {
        fn found(shape: &egui::Shape, clip: egui::Rect, marker: &str) -> bool {
            match shape {
                egui::Shape::Text(text) => text.galley.rows.iter().any(|row| {
                    let row_text: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                    row_text.contains(marker)
                        && clip.contains(row.rect.translate(text.pos.to_vec2()).center())
                }),
                egui::Shape::Vec(parts) => parts.iter().any(|part| found(part, clip, marker)),
                _ => false,
            }
        }
        output
            .shapes
            .iter()
            .any(|shape| found(&shape.shape, shape.clip_rect, marker))
    }

    fn u5_find_action(
        ctx: &egui::Context,
        app: &mut WordApp,
        size: egui::Vec2,
        label: &str,
    ) -> egui::Rect {
        for _ in 0..30 {
            let output = u5_draw_notice(ctx, app, size, vec![]);
            if feedback_ui_visible_text(&output, label).is_some() {
                // ScrollArea may still animate after the wheel event; click the settled position.
                let mut settled = output;
                for _ in 0..5 {
                    settled = u5_draw_notice(ctx, app, size, vec![]);
                }
                if let Some(rect) = feedback_ui_visible_text(&settled, label) {
                    return rect;
                }
            }
            let window = ctx
                .memory(|memory| {
                    memory.area_rect(egui::Id::new("notification-details-material-v1"))
                })
                .expect("notice window");
            let pointer =
                window.left_top() + egui::vec2(window.width() * 0.55, window.height() * 0.45);
            u5_draw_notice(
                ctx,
                app,
                size,
                vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -120.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        panic!("notice action {label:?} did not become visible");
    }

    #[test]
    fn u5_typed_material_failure_through_pending_tick_preserves_draft_deck_and_ai_count() {
        use wordweave5::{
            chat::Conversation,
            execution::Execution,
            material::{Mode, Request},
        };
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let original = "Use **improve clarity** in the answer.";
        let diagnostic =
            u5_diagnostic(&app.deck[0], original, "Use improve clarity in the answer.");
        let mut conversation = Conversation::new();
        conversation
            .complete("合成の質問".into(), original.into(), Execution::default())
            .unwrap();
        conversation.exchanges[0].for_material = true;
        let baseline = app.deck[0].clone();
        let request = Request::new(
            &conversation,
            &baseline.base,
            Mode::Correct,
            Some(baseline.clone()),
        )
        .unwrap();
        let mut candidate = baseline;
        candidate.usage = "保存済みの合成案".into();
        app.progress.material_draft = Some(request.build(candidate).unwrap());
        let before_progress = serde_json::to_value(&app.progress).unwrap();
        let before_deck = serde_json::to_value(&app.deck).unwrap();
        let before_calls = app.progress.ai_calls.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        app.pending = Some(Pending {
            kind: Activity::Material,
            key: app.key(),
            rx,
            cancel: None,
        });
        tx.send(Ok(AiResult::MaterialFailure(MaterialFailure::Evidence(
            diagnostic,
        ))))
        .unwrap();
        app.tick(&ctx);
        assert!(app.pending.is_none());
        assert!(!app.chat_material_open && !app.material_same_base);
        assert!(app.material_notice.is_some());
        let mut output = u5_draw_notice(&ctx, &mut app, egui::vec2(1120.0, 850.0), vec![]);
        for _ in 0..4 {
            output = u5_draw_notice(&ctx, &mut app, egui::vec2(1120.0, 850.0), vec![]);
        }
        assert!(
            app.notification_open && u5_notice_text(&output).contains("教材案を作成できなかった"),
            "open={}, payload={}, painted={:?}",
            app.notification_open,
            app.material_notice.is_some(),
            u5_notice_text(&output)
        );
        assert_eq!(
            serde_json::to_value(&app.progress).unwrap(),
            before_progress
        );
        assert_eq!(serde_json::to_value(&app.deck).unwrap(), before_deck);
        assert_eq!(app.progress.ai_calls, before_calls);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn u5_stale_material_result_after_selection_change_creates_no_notice() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let diagnostic = u5_diagnostic(&app.deck[0], "fixed **source**", "fixed source");
        let before = serde_json::to_value(&app.progress).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        app.pending = Some(Pending {
            kind: Activity::Material,
            key: "different selection".into(),
            rx,
            cancel: None,
        });
        tx.send(Ok(AiResult::MaterialFailure(MaterialFailure::Evidence(
            diagnostic,
        ))))
        .unwrap();
        app.tick(&ctx);
        assert!(app.pending.is_none() && app.material_notice.is_none());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn u5_direct_message_replacement_new_same_text_fatal_and_font_do_not_revive_stale_comparison() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let diagnostic = u5_diagnostic(&app.deck[0], "fixed **source**", "fixed source");
        let size = egui::vec2(1120.0, 850.0);
        app.notify_material_diagnostic(diagnostic.clone());
        app.notification_open = true;
        app.material_notice.as_mut().unwrap().comparison_open = true;
        app.material_notice.as_mut().unwrap().technical_open = true;
        app.material_notice.as_mut().unwrap().raw_mode = true;
        app.notify_material_diagnostic(diagnostic.clone());
        let fresh = app.material_notice.as_ref().unwrap();
        assert!(
            !fresh.comparison_open && !fresh.technical_open && !fresh.raw_mode,
            "same-text re-notification must start collapsed in readable mode"
        );
        app.font_notice = "合成フォント警告".into();
        let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        let mut font_seen = false;
        for _ in 0..30 {
            output = u5_draw_notice(&ctx, &mut app, size, vec![]);
            font_seen |= u5_notice_text(&output).contains("合成フォント警告");
            if font_seen {
                break;
            }
            let window = ctx
                .memory(|m| m.area_rect(egui::Id::new("notification-details-material-v1")))
                .unwrap();
            let pointer = window.center();
            u5_draw_notice(
                &ctx,
                &mut app,
                size,
                vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -120.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(
            font_seen,
            "font warning hidden by structured notice: {:?}",
            u5_notice_text(&output)
        );
        app.message = "合成操作が完了した。".into();
        let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        for _ in 0..3 {
            output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        }
        assert!(app.material_notice.is_none());
        assert!(u5_notice_text(&output).contains("合成操作が完了した。"));
        assert!(!u5_notice_text(&output).contains("AIが示した引用"));
        app.message = diagnostic.legacy_text.clone();
        u5_draw_notice(&ctx, &mut app, size, vec![]);
        assert!(
            app.material_notice.is_none(),
            "restoring old text must not revive the payload"
        );
        app.notify_material_diagnostic(diagnostic);
        app.fatal = Some("合成の保存停止".into());
        let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        for _ in 0..3 {
            output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        }
        assert!(app.material_notice.is_none());
        assert!(u5_notice_text(&output).contains("合成の保存停止"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn u5_comparison_and_raw_are_one_switch_and_copy_emits_exact_originals() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let original = " # 見出し\\n\r\n- 箇条書き **強調**\n実改行\t🦊 ";
        let quote = " # 見出し\\n\r\n- 箇条書き **強調**\n実改行\t🐺 ";
        let diagnostic = u5_diagnostic(&app.deck[0], original, quote);
        let size = egui::vec2(1120.0, 850.0);
        let expected_copy = diagnostic_copy_text(&diagnostic);
        app.notify_material_diagnostic(diagnostic);
        app.notification_open = true;
        let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        for _ in 0..3 {
            output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        }
        assert!(u5_notice_text(&output).contains("教材案を作成できなかった"));
        assert!(
            !u5_notice_text(&output).contains(" # 見出し"),
            "comparison starts collapsed"
        );
        assert!(
            feedback_ui_visible_text(&output, "引用と元の回答を確認").is_some(),
            "comparison entry must be visible without scrolling at standard size"
        );
        let footer = u5_find_action(&ctx, &mut app, size, "診断情報をコピー");
        let copied = u5_click_notice(&ctx, &mut app, size, footer.center());
        assert!(copied.platform_output.commands.iter().any(|command|
            matches!(command, egui::OutputCommand::CopyText(text) if text == &expected_copy)));
        let comparison = u5_find_action(&ctx, &mut app, size, "引用と元の回答を確認");
        u5_click_notice(&ctx, &mut app, size, comparison.center());
        assert!(app.material_notice.as_ref().unwrap().comparison_open);
        let raw = u5_find_action(&ctx, &mut app, size, "原文を表示");
        output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        assert!(
            feedback_ui_visible_text(&output, "読みやすい表示").is_some(),
            "mode label must be visible beside its switch"
        );
        output = u5_click_notice(&ctx, &mut app, size, raw.center());
        assert!(
            app.material_notice.as_ref().unwrap().raw_mode,
            "raw switch click missed at {raw:?}, painted={:?}",
            u5_notice_text(&output)
        );
        assert!(feedback_ui_visible_text(&output, "原文表示").is_some());
        let quote_copy = u5_find_action(&ctx, &mut app, size, "AI引用の原文をコピー");
        let copied = u5_click_notice(&ctx, &mut app, size, quote_copy.center());
        assert!(copied.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == quote)
        ));
        let source_copy = u5_find_action(&ctx, &mut app, size, "元の発言の原文をコピー");
        let copied = u5_click_notice(&ctx, &mut app, size, source_copy.center());
        assert!(copied.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == original)
        ));
        assert_eq!(
            diagnostic_copy_text(&app.material_notice.as_ref().unwrap().diagnostic),
            expected_copy
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn u5_long_comparison_scroll_reaches_tail_and_keeps_copy_footer_at_both_sizes() {
        for size in [
            egui::vec2(1150.0 / 0.8, 950.0 / 0.8),
            egui::vec2(820.0 / 1.6, 650.0 / 1.6),
        ] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            let original = format!(
                "# 合成回答\n- provide infrastructure\n- run safely\n- power on\n{}\n終端MARKER🦊",
                "固定した長文の説明。\n".repeat(75)
            );
            let quote = format!("{}引用末尾🐺", "合成引用".repeat(90));
            app.notify_material_diagnostic(u5_diagnostic(&app.deck[0], &original, &quote));
            app.notification_open = true;
            app.material_notice.as_mut().unwrap().comparison_open = true;
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
            let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
            for _ in 0..5 {
                output = u5_draw_notice(&ctx, &mut app, size, vec![]);
            }
            let window_id = egui::Id::new("notification-details-material-v1");
            let window = ctx.memory(|m| m.area_rect(window_id)).unwrap();
            assert!(
                screen.contains_rect(window),
                "window left screen: {window:?}"
            );
            let pointer =
                window.left_top() + egui::vec2(window.width() * 0.55, window.height() * 0.45);
            let mut reached = u5_visible_row(&output, "終端MARKER🦊");
            for _ in 0..60 {
                output = u5_draw_notice(
                    &ctx,
                    &mut app,
                    size,
                    vec![
                        egui::Event::PointerMoved(pointer),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, -240.0),
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
                reached |= u5_visible_row(&output, "終端MARKER🦊");
                let footer = feedback_ui_visible_text(&output, "診断情報をコピー")
                    .expect("fixed copy footer");
                assert!(screen.contains_rect(footer), "footer offscreen: {footer:?}");
                if reached {
                    break;
                }
            }
            assert!(
                reached,
                "long fixed answer tail was never visible at {size:?}"
            );
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn u5_saved_draft_detail_click_uses_saved_source_and_keeps_proposal() {
        use wordweave5::{
            chat::Conversation,
            execution::Execution,
            material::{Mode, Request},
        };
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let baseline = app.deck[0].clone();
        let mut conversation = Conversation::new();
        conversation
            .complete(
                "保存時の合成質問".into(),
                "保存時の **固定回答** 🦊".into(),
                Execution::default(),
            )
            .unwrap();
        conversation.exchanges[0].for_material = true;
        let request = Request::new(
            &conversation,
            &baseline.base,
            Mode::Correct,
            Some(baseline.clone()),
        )
        .unwrap();
        let mut candidate = baseline;
        candidate.usage = "保存案の合成変更".into();
        let response = serde_json::json!({"entry":candidate,"reasons":[{"path":"/usage",
            "reason":"合成理由","quotes":[{"exchange_index":0,"role":"assistant","quote":"**固定回答**"}]}]}).to_string();
        let mut draft = request.build_response(&response).unwrap();
        draft.reasons[0].quotes[0].quote = "固定回答 🐺".into();
        app.progress.material_draft = Some(draft);
        app.page = Page::Chat;
        app.chat_material_open = true;
        let before = serde_json::to_value(&app.progress).unwrap();
        let deck = serde_json::to_value(&app.deck).unwrap();
        let size = egui::vec2(1120.0, 850.0);
        let mut output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        for _ in 0..4 {
            output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let detail = feedback_ui_visible_text(&output, "理由を詳しく確認")
            .expect("saved-draft detail action");
        feedback_ui_click(&ctx, &mut app, size, detail.center());
        let notice = app
            .material_notice
            .as_ref()
            .expect("typed saved-draft diagnostic");
        assert_eq!(notice.diagnostic.stage, MaterialStage::SavedDraft);
        assert_eq!(
            notice
                .diagnostic
                .evidence
                .as_ref()
                .unwrap()
                .original
                .as_deref(),
            Some("保存時の **固定回答** 🦊")
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(serde_json::to_value(&app.deck).unwrap(), deck);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn u5_close_button_then_status_reopen_resets_comparison_raw_and_technical_state() {
        fn text_size(output: &egui::FullOutput, label: &str) -> Option<f32> {
            fn find(shape: &egui::Shape, label: &str) -> Option<f32> {
                match shape {
                    egui::Shape::Text(text) if text.galley.text() == label => text
                        .galley
                        .job
                        .sections
                        .first()
                        .map(|section| section.format.font_id.size),
                    egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, label)),
                    _ => None,
                }
            }
            output
                .shapes
                .iter()
                .find_map(|shape| find(&shape.shape, label))
        }
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let diagnostic = u5_diagnostic(
            &app.deck[0],
            "Use **fixed** source 🦊",
            "Use fixed source 🐺",
        );
        app.notify_material_diagnostic(diagnostic);
        app.notification_open = true;
        let size = egui::vec2(1120.0, 850.0);
        let mut output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        for _ in 0..4 {
            output = u5_draw_notice(&ctx, &mut app, size, vec![]);
        }
        let title_size = text_size(&output, "教材案を作成できなかった").expect("painted title");
        let body_size = text_size(
            &output,
            "教材には登録していない。既存教材も変更していない。",
        )
        .expect("painted body");
        assert!(
            title_size > body_size,
            "title {title_size} must exceed body {body_size}"
        );
        let notice = app.material_notice.as_mut().unwrap();
        notice.comparison_open = true;
        notice.raw_mode = true;
        notice.technical_open = true;
        let close = u5_find_action(&ctx, &mut app, size, "閉じる");
        u5_click_notice(&ctx, &mut app, size, close.center());
        assert!(
            !app.notification_open,
            "footer close must dismiss the notice"
        );
        assert!(
            app.material_notice.is_some(),
            "dismissal retains the current payload for status reopen"
        );

        let mut output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        for _ in 0..4 {
            output = feedback_ui_draw(&ctx, &mut app, size, vec![]);
        }
        let status = feedback_ui_visible_text(&output, "● 通知").expect("status reopen button");
        feedback_ui_click(&ctx, &mut app, size, status.center());
        assert!(
            app.notification_open,
            "status button must reopen the notice"
        );
        let notice = app.material_notice.as_ref().unwrap();
        assert!(
            !notice.comparison_open && !notice.raw_mode && !notice.technical_open,
            "reopened notice starts collapsed in readable mode"
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
