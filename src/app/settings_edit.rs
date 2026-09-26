//! Settings editing is deliberately separate from the autosaved Progress.
use super::*;
use wordweave5::store::Settings;

#[derive(Clone, Copy)]
pub(super) enum Destination { Page(Page), Close }

#[derive(Default)]
pub(super) struct SettingsEditor {
    pub draft: Settings,
    baseline: Settings,
    pub active: bool,
    pub leave: Option<Destination>,
    pub error: Option<String>,
    pub preview_speech: bool,
    pub reset_view: bool,
    pub zoom_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::harness_tests::{fixture, frame};

    #[test]
    fn settings_zoom_drag_tracks_physical_pointer_without_jumping() {
        let (ctx, mut app, root) = fixture();
        app.begin_settings_edit();
        app.settings_editor.zoom_open = true;
        let mut draw = |events: Vec<egui::Event>| {
            let mut raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO,
                    egui::vec2(1150.0, 950.0) / ctx.zoom_factor())),
                events, ..Default::default()
            };
            app.settings_zoom_input(&ctx, &mut raw);
            let _ = ctx.run(raw, |ctx| app.settings_zoom_panel(ctx));
        };
        for _ in 0..3 { draw(vec![]); }
        let rect = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-zoom"))).unwrap();
        let start = (rect.left_center() + egui::vec2(60.0 / ctx.zoom_factor(), 0.0)) * ctx.zoom_factor();
        draw(vec![egui::Event::PointerMoved(start / ctx.zoom_factor()), egui::Event::PointerButton {
            pos: start / ctx.zoom_factor(), button: egui::PointerButton::Primary,
            pressed: true, modifiers: egui::Modifiers::NONE }]);
        let mut previous = 0.0;
        for offset in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0] {
            let pointer = (start + egui::vec2(offset, 0.0)) / ctx.zoom_factor();
            draw(vec![egui::Event::PointerMoved(pointer)]);
            let value = ctx.zoom_factor();
            assert!(value >= previous && value < 1.5, "continuous pointer produced jump: {previous} -> {value}");
            previous = value;
        }
        drop(draw);
        assert!(app.settings_editor.draft.font_scale > 0.9);
        assert_eq!(app.progress.settings.font_scale, 0.8);
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_draft_survives_autosave_without_leaking_and_cancel_preserves_chat() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        app.begin_settings_edit();
        let saved = app.progress.settings.clone();
        app.settings_editor.draft.minutes = 23;
        app.settings_editor.draft.codex_model = "draft-only".into();
        app.progress.chats[0].title = "独立した会話の更新".into();
        app.dirty = true;
        app.last_save = Instant::now() - Duration::from_secs(30);
        frame(&ctx, &mut app, false);
        let stored = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(stored.settings, saved);
        assert_eq!(stored.chats[0].title, "独立した会話の更新");
        assert_eq!(app.settings_editor.draft.minutes, 23);
        app.cancel_settings(&ctx);
        assert_eq!(app.settings_editor.draft, saved);
        assert_eq!(app.progress.chats[0].title, "独立した会話の更新");
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_save_is_explicit_preserves_external_preferences_and_failure_is_retryable() {
        let (ctx, mut app, root) = fixture();
        app.begin_settings_edit();
        app.settings_editor.draft.minutes = 18;
        app.progress.settings.speech_volume = 0.3;
        app.progress.settings.speech_repeat = true;
        app.begin_settings_edit();
        assert_eq!(app.settings_editor.draft.minutes, 18);
        assert_eq!(app.settings_editor.draft.speech_volume, 0.3);
        let storage = app.storage.take();
        assert!(app.save_settings(&ctx).is_err());
        assert_eq!(app.progress.settings.minutes, 5);
        assert_eq!(app.settings_editor.draft.minutes, 18);
        app.storage = storage;
        // Force an actual filesystem failure without touching real user data.
        let data = app.storage.as_ref().unwrap().dir.clone();
        let destination = data.join("progress.json");
        let original = data.join("original-settings-test.json");
        std::fs::rename(&destination, &original).unwrap();
        std::fs::create_dir(&destination).unwrap();
        assert!(app.save_settings(&ctx).is_err());
        assert_eq!(app.progress.settings.minutes, 5);
        assert_eq!(app.settings_editor.draft.minutes, 18);
        std::fs::remove_dir(&destination).unwrap();
        std::fs::rename(&original, &destination).unwrap();
        app.save_settings(&ctx).unwrap();
        let stored = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(stored.settings.minutes, 18);
        assert_eq!(stored.settings.speech_volume, 0.3);
        assert!(stored.settings.speech_repeat);
        assert!(!app.settings_changed());
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_model_catalog_uses_unsaved_request_path_without_saving_it() {
        let (ctx, mut app, root) = fixture();
        app.begin_settings_edit();
        let original = app.progress.settings.codex_path.clone();
        app.settings_editor.draft.codex_path = "draft-codex".into();
        let (tx, rx) = mpsc::channel();
        app.pending = Some(Pending { kind: Activity::Models, key: app.key(), rx, cancel: None });
        tx.send(Ok(AiResult::ModelChoices { path: "draft-codex".into(), models: vec![] })).unwrap();
        app.tick(&ctx);
        assert_eq!(app.effort_catalog.as_ref().unwrap().0, "draft-codex");
        assert_eq!(app.progress.settings.codex_path, original);
        app.cancel_settings(&ctx);
        assert!(app.effort_choices().is_none());
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_navigation_close_and_restore_never_silently_commit_a_draft() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        app.begin_settings_edit();
        app.settings_editor.draft.minutes = 19;
        app.page = Page::Home;
        app.guard_settings_navigation(&ctx);
        assert!(app.page == Page::Settings);
        assert!(matches!(app.settings_editor.leave, Some(Destination::Page(Page::Home))));
        let output = frame(&ctx, &mut app, true);
        assert!(output.viewport_output.values().any(|v| v.commands.contains(&egui::ViewportCommand::CancelClose)));
        assert!(matches!(app.settings_editor.leave, Some(Destination::Close)));
        app.dirty = true;
        eframe::App::on_exit(&mut app, None);
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().settings.minutes, 5);
        let mut restored = app.progress.clone();
        restored.settings.minutes = 9;
        app.restore(restored).unwrap();
        app.begin_settings_edit();
        assert_eq!(app.settings_editor.draft.minutes, 9);
        assert!(!app.settings_changed());
        assert!(app.settings_editor.leave.is_none());
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }
}

fn merge_settings(base: &Settings, draft: &Settings, current: &Settings) -> Settings {
    let baseline = serde_json::to_value(base).expect("serializable settings");
    let edited = serde_json::to_value(draft).expect("serializable settings");
    let mut merged = serde_json::to_value(current).expect("serializable settings");
    for (key, value) in edited.as_object().expect("settings object") {
        if baseline[key] != *value { merged[key] = value.clone(); }
    }
    serde_json::from_value(merged).expect("merge of valid settings fields")
}

impl WordApp {
    pub(super) fn begin_settings_edit(&mut self) {
        if !self.settings_editor.active {
            self.settings_editor.draft = self.progress.settings.clone();
            self.settings_editor.baseline = self.progress.settings.clone();
            self.settings_editor.active = true;
        } else if self.settings_editor.baseline != self.progress.settings {
            // External playback preferences may change while the form is open.
            self.settings_editor.draft = merge_settings(&self.settings_editor.baseline,
                &self.settings_editor.draft, &self.progress.settings);
            self.settings_editor.baseline = self.progress.settings.clone();
        }
    }

    pub(super) fn settings_for_ui(&self) -> &Settings {
        if self.settings_editor.active { &self.settings_editor.draft } else { &self.progress.settings }
    }

    pub(super) fn settings_changed(&self) -> bool {
        self.settings_editor.active && self.settings_editor.draft != self.settings_editor.baseline
    }

    pub(super) fn save_settings(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if let Some(error) = &self.fatal { return Err(error.clone()); }
        self.credit_time();
        let mut next = self.progress.clone();
        next.settings = merge_settings(&self.settings_editor.baseline,
            &self.settings_editor.draft, &self.progress.settings);
        next.validate()?;
        self.storage.as_ref().ok_or("保存先がありません。")?.save(&next)?;
        // Commit only after persistence succeeds; retry/cancel retain the draft on failure.
        self.progress = next;
        self.dirty = false;
        self.last_save = Instant::now();
        self.settings_editor.baseline = self.progress.settings.clone();
        self.settings_editor.draft = self.progress.settings.clone();
        self.settings_editor.error = None;
        self.preview_settings_zoom(ctx, self.progress.settings.font_scale);
        self.message = "設定を保存した。".into();
        Ok(())
    }

    pub(super) fn cancel_settings(&mut self, ctx: &egui::Context) {
        if self.settings_editor.preview_speech { self.stop_speech(); }
        self.settings_editor.preview_speech = false;
        self.settings_editor.draft = self.progress.settings.clone();
        self.settings_editor.baseline = self.progress.settings.clone();
        self.settings_editor.error = None;
        self.preview_settings_zoom(ctx, self.progress.settings.font_scale);
        self.message = "未保存の設定変更を取り消した。".into();
    }

    pub(super) fn guard_settings_navigation(&mut self, ctx: &egui::Context) {
        if self.settings_editor.active && self.page != Page::Settings {
            if self.settings_changed() {
                self.settings_editor.leave = Some(Destination::Page(self.page));
                self.page = Page::Settings;
            } else {
                if self.settings_editor.preview_speech { self.stop_speech(); }
                self.settings_editor = SettingsEditor::default();
            }
        }
        if self.settings_editor.reset_view {
            self.preview_settings_zoom(ctx, self.progress.settings.font_scale);
            self.settings_editor.reset_view = false;
        }
    }

    pub(super) fn reset_settings_after_restore(&mut self) {
        if self.settings_editor.preview_speech { self.stop_speech(); }
        self.settings_editor = SettingsEditor { reset_view: true, ..Default::default() };
    }

    pub(super) fn settings_leave_dialog(&mut self, ctx: &egui::Context) {
        let Some(destination) = self.settings_editor.leave else { return; };
        let mut action = 0;
        egui::Window::new("未保存の設定変更")
            .collapsible(false).resizable(false)
            .max_width((ctx.screen_rect().width() - 32.0).max(240.0))
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.label("設定はまだ保存されていない。変更をどう扱うか選択してください。");
                if let Some(error) = &self.settings_editor.error {
                    ui.colored_label(Color32::DARK_RED, error);
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.ww_button("保存して進む").clicked() { action = 1; }
                    if ui.ww_button("破棄して進む").clicked() { action = 2; }
                    if ui.ww_button("編集を続ける").clicked() { action = 3; }
                });
            });
        if action == 3 { self.settings_editor.leave = None; }
        if action == 1 {
            if let Err(error) = self.save_settings(ctx) {
                self.settings_editor.error = Some(error);
                return;
            }
        }
        if action == 2 { self.cancel_settings(ctx); }
        if action == 1 || action == 2 {
            if self.settings_editor.preview_speech { self.stop_speech(); }
            self.settings_editor = SettingsEditor::default();
            match destination {
                Destination::Page(page) => self.page = page,
                Destination::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }
    }

    /// This one persistent Area uses native-pixel geometry, including its real
    /// Slider hit target. Never move a dragged Slider between parents/IDs.
    pub(super) fn settings_zoom_panel(&mut self, ctx: &egui::Context) {
        if !self.settings_editor.active || !self.settings_editor.zoom_open
            || self.settings_editor.leave.is_some() { return; }
        let scale = 1.0 / ctx.zoom_factor();
        let screen = ctx.screen_rect();
        let position = egui::pos2(screen.right() - 360.0 * scale,
            screen.bottom() - 180.0 * scale);
        let mut close = false;
        egui::Area::new(egui::Id::new("fixed-settings-zoom"))
            .order(egui::Order::Foreground).fixed_pos(position)
            .constrain(false).movable(false).show(ctx, |ui| {
                ui.set_width(340.0 * scale);
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0) * scale;
                ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0) * scale;
                ui.spacing_mut().interact_size = egui::vec2(65.0, 28.0) * scale;
                ui.spacing_mut().slider_width = 240.0 * scale;
                for style in [egui::TextStyle::Body, egui::TextStyle::Button, egui::TextStyle::Small, egui::TextStyle::Monospace] {
                    ui.style_mut().text_styles.insert(style, home_art::home_font(15.0 * scale));
                }
                egui::Frame::new().fill(Color32::WHITE)
                    .stroke(egui::Stroke::new(scale, ux::BORDER))
                    .inner_margin(egui::Margin::same((8.0 * scale).round() as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("倍率プレビュー");
                            close = ui.ww_button("×").clicked();
                        });
                        let slider = ui.add(egui::Slider::new(&mut self.settings_editor.draft.font_scale, 0.5..=1.6)
                            .step_by(0.01)
                            .custom_formatter(|value, _| format!("{:.0}%", value * 100.0))
                            .custom_parser(|text| text.trim().trim_end_matches('%').trim().parse::<f64>().ok().map(|value| value / 100.0)));
                        #[cfg(test)]
                        ctx.data_mut(|data| data.insert_temp(egui::Id::new("settings-zoom"), slider.rect));
                        if slider.changed() { self.preview_settings_zoom(ctx, self.settings_editor.draft.font_scale); }
                        ui.label("50% ～ 160%　標準：80%");
                    });
            });
        if close { self.settings_editor.zoom_open = false; }
    }

    pub(super) fn preview_settings_zoom(&mut self, ctx: &egui::Context, value: f32) {
        self.settings_zoom_input_target = Some(value);
        ctx.request_repaint();
    }

    /// egui 0.31 rescales screen_rect when a pending zoom is applied, but not
    /// pointer events. egui-winit collected those using the previous zoom.
    /// Convert once, before begin_pass, so a native-fixed slider cannot feed
    /// its own coordinate jump back into the next requested zoom.
    pub(super) fn settings_zoom_input(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        let Some(target) = self.settings_zoom_input_target.take() else { return; };
        let ratio = ctx.zoom_factor() / target;
        if (ratio - 1.0).abs() < f32::EPSILON { return; }
        ctx.set_zoom_factor(target);
        let moved = input.events.iter().any(|event| matches!(event, egui::Event::PointerMoved(_)));
        for event in &mut input.events {
            match event {
                egui::Event::PointerMoved(pos) | egui::Event::PointerButton { pos, .. }
                | egui::Event::Touch { pos, .. } => *pos = *pos * ratio,
                _ => {}
            }
        }
        if !moved {
            if let Some(pos) = ctx.input(|input| input.pointer.latest_pos()) {
                input.events.insert(0, egui::Event::PointerMoved(pos * ratio));
            }
        }
    }
}
