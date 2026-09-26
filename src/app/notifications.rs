use super::*;

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
    pub(super) fn notify_error(&mut self, message: String) {
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
        self.message = message.into();
        self.notification_attention = self.message.clone();
        self.notification_error.clear();
        // A fresh request may be refused for the same reason as a dismissed one.
        self.last_notification_alerts[1].clear();
    }

    pub(super) fn notify_result(&mut self, result: Result<String, String>) {
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
    pub(super) fn open_codex_path_guidance(&mut self) {
        self.page = Page::Settings;
        self.codex_path_guidance = true;
        self.codex_path_focus_pending = true;
        self.notification_open = false;
    }

    pub(super) fn notification_button(&mut self, ui: &mut egui::Ui) {
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
            let summary = if detail.is_empty() {
                "通知はない".to_owned()
            } else {
                detail.chars().take(180).collect()
            };
            if ui
                .add(crate::app::controls::Button::new(label).min_size(egui::vec2(80.0, 30.0)))
                .on_hover_text(
                    RichText::new(format!("通知：{summary}\nクリックで詳細・コピー"))
                        .font(super::chrome::status_font()),
                )
                .clicked()
            {
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
        // Explicit failures, warnings and unfulfilled requests interrupt. Results remain
        // available through the status button, without reopening a dismissed alert.
        let alerts = [
            self.fatal.as_deref().unwrap_or_default(),
            if missing_codex_path(&self.message)
                || (!self.message.is_empty() && (self.message == self.notification_error
                    || self.message == self.notification_attention)) {
                self.message.as_str()
            } else { "" },
            self.font_notice.as_str(),
        ];
        if self.pending.is_none() && !self.batch_running {
            // Removing a warning (e.g. by copying text) is not a new alert.
            for (alert, seen) in alerts.into_iter().zip(&mut self.last_notification_alerts) {
                if alert != seen {
                    if !alert.is_empty() { self.notification_open = true; }
                    *seen = alert.to_owned();
                }
            }
        }
        if !self.notification_open {
            return;
        }
        let mut open = true;
        let mut fix_path = false;
        let max_width = (ctx.screen_rect().width() - 40.0).max(240.0);
        let max_height = (ctx.screen_rect().height() - 60.0).max(160.0);
        let shown =
            egui::Window::new(RichText::new("通知の詳細").font(super::chrome::status_font()))
                .id(egui::Id::new("notification-details-v2"))
                .open(&mut open)
                .resizable([true, true])
                .default_size(egui::vec2(
                    580.0_f32.min(max_width),
                    240.0_f32.min(max_height),
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
                            fix_path = ui.ww_button("設定の実行ファイル欄へ").clicked();
                        }
                        if ui.ww_button("詳細をコピー").clicked() {
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
                                ui.add(egui::Label::new(&detail).wrap().selectable(true));
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
        self.notification_open = open;
        if fix_path {
            self.open_codex_path_guidance();
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
                    if ui.ww_button("設定の実行ファイル欄へ").clicked() {
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

    #[test]
    fn generation_limit_opens_notice_on_each_attempt_without_consuming_quota() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.settings.ai_daily_limit = 1;
        app.progress.ai_calls.insert(today(), 1);
        let before = serde_json::to_value(&app.progress).unwrap();
        for _ in 0..2 {
            assert!(!app.reserve_generation());
            assert!(app.message.contains("本日の生成上限"));
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
            assert!(app.notification_open, "a rejected request must explain why");
            assert!(!app.notification_is_error(), "a quota limit is not an error");
            app.notification_open = false;
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
            assert!(!app.notification_open, "dismissal lasts until another attempt");
        }
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert!(app.pending.is_none());
        app.message = "発言をコピーした。".into();
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(!app.notification_open);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_generation_targets_opens_notice_without_starting_work() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.begin_batch(vec![]);
        assert!(app.message.contains("すべて自動生成・登録済み"));
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(app.notification_open);
        assert!(!app.batch_running && app.pending.is_none());
        assert!(!app.notification_is_error());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_information_does_not_open_but_errors_do() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        for message in ["発言をクリップボードへコピーした。", "設定を保存した。", "接続成功"] {
            app.message = message.into();
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
            assert!(!app.notification_open, "information must not interrupt: {message}");
        }
        app.message = "接続に失敗した。".into();
        app.notification_error = app.message.clone();
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(app.notification_open);
        app.notification_open = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(!app.notification_open, "dismissed alert must stay dismissed");
        app.notify_error("接続に失敗した。".into());
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(app.notification_open, "a new failure must reopen even with the same text");
        app.notification_open = false;
        app.notify_warning("実行記録の一部を読み込めなかった。");
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(app.notification_open);
        assert!(!app.notification_is_error());
        app.notification_open = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(!app.notification_open);
        // Informational details remain available when explicitly opened.
        app.message = "発言をクリップボードへコピーした。".into();
        app.notification_open = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
        assert!(app.notification_open);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn notification_failure_warning_and_fatal_are_not_lost_or_reopened_by_copy() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let frame = |app: &mut WordApp| {
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.notification_window(ctx));
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
        assert!(!app.notification_open, "a copy must not reopen the same fatal alert");
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
}
