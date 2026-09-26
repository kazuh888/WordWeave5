use super::{home_art::*, Page, WordApp};
use crate::app::controls::UiControls as _;
use eframe::egui::*;

const STANDARD_ZOOM: f32 = 0.8;
const ZOOM_PRESETS: [(&str, f32); 3] = [("小", 0.6), ("標準", STANDARD_ZOOM), ("大", 1.0)];
const PAGES: [(Page, &str, Icon); 7] = [
    (Page::Home, "ホーム", Icon::Home),
    (Page::Study, "学習", Icon::Book),
    (Page::Deck, "教材", Icon::File),
    (Page::Words, "語彙を追加", Icon::Plus),
    (Page::Chat, "英語チャット", Icon::Chat),
    (Page::Stats, "記録", Icon::Chart),
    (Page::Settings, "設定", Icon::Gear),
];

fn bounded_zoom(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.5, 1.6)
    } else {
        STANDARD_ZOOM
    }
}

fn requested_zoom(ctx: &Context) -> Option<f32> {
    let (reset, steps) = ctx.input_mut(|input| {
        let reset = input.consume_key(Modifiers::CTRL, Key::Num0);
        let plus = input.consume_key(Modifiers::CTRL, Key::Plus);
        let equals = input.consume_key(Modifiers::CTRL, Key::Equals);
        let minus = input.consume_key(Modifiers::CTRL, Key::Minus);
        let mut wheel = 0.0;
        input.events.retain(|event| {
            if let Event::MouseWheel {
                delta, modifiers, ..
            } = event
            {
                if modifiers.ctrl && delta.y != 0.0 {
                    wheel += delta.y.signum();
                    return false;
                }
            }
            true
        });
        (
            reset,
            u8::from(plus || equals) as f32 - u8::from(minus) as f32 + wheel,
        )
    });
    if reset {
        Some(STANDARD_ZOOM)
    } else if steps != 0.0 {
        Some(bounded_zoom(
            ((ctx.zoom_factor() + steps * 0.1) * 100.0).round() / 100.0,
        ))
    } else {
        None
    }
}

pub(super) fn status_font() -> FontId {
    FontId::new(16.0, FontFamily::Name("status_regular".into()))
}

pub(super) fn status_field(ui: &mut Ui, text: &str, width: f32) -> Response {
    let color = Color32::from_rgb(92, 96, 102);
    let font = status_font();
    let mut job = eframe::egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap.max_width = (width - 16.0).max(1.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let (rect, response) = ui.allocate_exact_size(vec2(width, 30.0), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), text));
    let bounds = if galley.mesh_bounds.is_finite() && galley.mesh_bounds.is_positive() {
        galley.mesh_bounds
    } else {
        galley.rect
    };
    let origin = pos2(
        rect.left() + 8.0 - bounds.left(),
        rect.center().y - bounds.center().y,
    );
    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .galley(origin, galley, color);
    response.on_hover_text(text)
}

impl WordApp {
    pub(super) fn status_summary(&mut self, ui: &mut Ui) {
        // Establish the row height before nested controls calculate alignment.
        ui.spacing_mut().interact_size.y = 30.0;
        let snapshot = wordweave5::execution::snapshot();
        let phase = if snapshot.active { "今回" } else { "前回" };
        let missing = if snapshot.active {
            "確認中"
        } else {
            "未確認"
        };
        let model = snapshot
            .execution
            .as_ref()
            .and_then(|s| s.model.as_deref())
            .unwrap_or(missing);
        let effort = snapshot
            .execution
            .as_ref()
            .and_then(|s| s.effort.as_deref())
            .unwrap_or(missing);
        let fields = [
            (self.connection_label().to_owned(), 240.0),
            (format!("モデル（{phase}）：{model}"), 310.0),
            (format!("effort（{phase}）：{effort}"), 230.0),
            ("ローカル学習 / AI：Codex・ChatGPT認証".to_owned(), 410.0),
        ];
        ui.horizontal(|ui| {
            self.notification_button(ui);
            let available = ui.available_width();
            let total: f32 = fields.iter().map(|(_, width)| width).sum();
            ui.horizontal(|ui| {
                for (index, (text, width)) in fields.iter().enumerate() {
                    if index > 0 {
                        ui.separator();
                    }
                    status_field(
                        ui,
                        text,
                        ((available - 48.0).max(4.0) * width / total).max(1.0),
                    );
                }
            });
        });
    }

    fn apply_chrome_zoom(&mut self, ctx: &Context, value: f32) {
        let value = bounded_zoom(value);
        if self.settings_editor.active {
            self.settings_editor.draft.font_scale = value;
            self.preview_settings_zoom(ctx, value);
            return;
        }
        ctx.set_zoom_factor(value);
        if self.progress.settings.font_scale != value {
            self.progress.settings.font_scale = value;
            self.dirty = true;
        }
    }

    pub(super) fn handle_zoom_input(&mut self, ctx: &Context) {
        if let Some(value) = requested_zoom(ctx) {
            self.apply_chrome_zoom(ctx, value);
        }
    }

    pub(super) fn navigation_chrome(&mut self, ctx: &Context, confirming: bool) {
        TopBottomPanel::top("navigation")
            .frame(
                Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(Margin::symmetric(24, 14)),
            )
            .show(ctx, |ui| {
                if ui.available_width() < 800.0 {
                    ui.add_enabled_ui(!confirming, |ui| self.chrome_zoom_controls(ui));
                    self.chrome_navigation(ui, confirming);
                } else {
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() - 190.0).max(1.0);
                        ui.allocate_ui_with_layout(
                            vec2(width, 42.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                self.chrome_navigation(ui, confirming);
                            },
                        );
                        ui.add_enabled_ui(!confirming, |ui| self.chrome_zoom_controls(ui));
                    });
                }
            });
    }

    fn chrome_zoom_controls(&mut self, ui: &mut Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            // Fixed row height prevents growth from the panel's cached previous height.
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 44.0),
                Layout::right_to_left(Align::Center),
                |ui| {
                    for (label, zoom) in ZOOM_PRESETS {
                        if size_button(ui, label, (ui.ctx().zoom_factor() - zoom).abs() < 0.001)
                            .on_hover_text(format!(
                                "{:.0}%（Ctrl＋ホイール、Ctrl＋＋／－、Ctrl＋0で標準）",
                                zoom * 100.0
                            ))
                            .clicked()
                        {
                            self.apply_chrome_zoom(ui.ctx(), zoom);
                        }
                    }
                },
            );
        });
    }

    fn chrome_navigation(&mut self, ui: &mut Ui, confirming: bool) {
        // Secondary actions remain in the menu even on a wide screen.
        let compact = ui.available_width() < 1100.0;
        ui.horizontal_wrapped(|ui| {
            ui.add(Label::new(
                RichText::new("WordWeave 5")
                    .size(if compact { 22.0 } else { 27.0 })
                    .color(Color32::from_rgb(0, 96, 120))
                    .family(FontFamily::Name("heading".into())),
            ));
            let enabled = self.pending.is_none()
                && self.recorder.is_none()
                && !self.batch_running
                && !confirming;
            ui.add_enabled_ui(enabled, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (page, label, icon) in PAGES {
                        if compact && page != self.page {
                            continue;
                        }
                        if nav_item(ui, label, icon, self.page == page).clicked() {
                            self.page = page;
                        }
                    }
                    ui.ww_menu_button("メニュー", |ui| {
                        if compact {
                            for (page, label, icon) in PAGES {
                                if nav_item(ui, label, icon, self.page == page).clicked() {
                                    self.page = page;
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                        }
                        if ui.ww_button("バージョン情報").clicked() {
                            self.about_open = true;
                            ui.close_menu();
                        }
                        if ui.ww_button("実行記録").clicked() {
                            self.run_history_open = true;
                            self.refresh_runs();
                            ui.close_menu();
                        }
                    });
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn request(ctx: &Context, events: Vec<Event>) -> Option<f32> {
        let mut result = None;
        let _ = ctx.run(
            RawInput {
                events,
                ..Default::default()
            },
            |ctx| {
                result = requested_zoom(ctx);
            },
        );
        result
    }

    #[test]
    fn zoom_shortcuts_use_standard_eighty_percent_and_bounds() {
        let ctx = Context::default();
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        ctx.set_zoom_factor(0.8);
        assert_eq!(
            request(&ctx, vec![key(Key::Plus, Modifiers::CTRL)]),
            Some(0.9)
        );
        assert_eq!(
            request(&ctx, vec![key(Key::Minus, Modifiers::CTRL)]),
            Some(0.7)
        );
        assert_eq!(
            request(&ctx, vec![key(Key::Num0, Modifiers::CTRL)]),
            Some(0.8)
        );
        assert_eq!(request(&ctx, vec![key(Key::Plus, Modifiers::NONE)]), None);
        ctx.set_zoom_factor(1.6);
        assert_eq!(
            request(&ctx, vec![key(Key::Equals, Modifiers::CTRL)]),
            Some(1.6)
        );
        ctx.set_zoom_factor(0.5);
        assert_eq!(
            request(&ctx, vec![key(Key::Minus, Modifiers::CTRL)]),
            Some(0.5)
        );
    }

    #[test]
    fn ctrl_wheel_is_consumed_and_plain_wheel_is_preserved() {
        let ctx = Context::default();
        ctx.set_zoom_factor(0.8);
        let wheel = |modifiers| Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: vec2(0.0, 1.0),
            modifiers,
        };
        let _ = ctx.run(
            RawInput {
                events: vec![wheel(Modifiers::CTRL), wheel(Modifiers::NONE)],
                ..Default::default()
            },
            |ctx| {
                assert_eq!(requested_zoom(ctx), Some(0.9));
                assert_eq!(ctx.input(|input| input.events.len()), 1);
            },
        );
    }

    #[test]
    fn preset_zoom_is_persistable_and_only_changes_dirty_when_needed() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        for (_, value) in ZOOM_PRESETS {
            app.dirty = false;
            app.progress.settings.font_scale = 1.4;
            app.apply_chrome_zoom(&ctx, value);
            assert_eq!(app.progress.settings.font_scale, value);
            assert!(app.dirty);
            app.dirty = false;
            app.apply_chrome_zoom(&ctx, value);
            assert!(!app.dirty);
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
