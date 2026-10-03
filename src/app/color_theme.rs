use super::*;
use wordweave5::store::{Settings, TintChoice};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ColorChoices {
    pub page: TintChoice,
    pub input: TintChoice,
}

impl ColorChoices {
    fn from_settings(settings: &Settings) -> Self {
        Self {
            page: settings.page_tint,
            input: settings.input_tint,
        }
    }
}

pub(super) struct ColorEditor {
    pub baseline: ColorChoices,
    pub draft: ColorChoices,
    pub page_hex: String,
    pub input_hex: String,
    pub page_depth_text: String,
    pub input_depth_text: String,
    pub page_error: Option<String>,
    pub input_error: Option<String>,
    pub page_depth_error: Option<String>,
    pub input_depth_error: Option<String>,
    pub save_error: Option<String>,
    pub leave_close: bool,
    focused: bool,
}

impl ColorEditor {
    fn new(settings: &Settings) -> Self {
        let choices = ColorChoices::from_settings(settings);
        Self {
            baseline: choices,
            draft: choices,
            page_hex: format_hex(choices.page.rgb),
            input_hex: format_hex(choices.input.rgb),
            page_depth_text: choices.page.depth.to_string(),
            input_depth_text: choices.input.depth.to_string(),
            page_error: None,
            input_error: None,
            page_depth_error: None,
            input_depth_error: None,
            save_error: None,
            leave_close: false,
            focused: true,
        }
    }

    fn invalid(&self) -> bool {
        self.page_error.is_some()
            || self.input_error.is_some()
            || self.page_depth_error.is_some()
            || self.input_depth_error.is_some()
    }

    pub(super) fn dirty(&self) -> bool {
        self.draft != self.baseline || self.invalid()
    }

    fn reset(&mut self) {
        self.draft = ColorChoices {
            page: TintChoice::default(),
            input: TintChoice::default(),
        };
        self.page_hex = format_hex(self.draft.page.rgb);
        self.input_hex = format_hex(self.draft.input.rgb);
        self.page_depth_text = self.draft.page.depth.to_string();
        self.input_depth_text = self.draft.input.depth.to_string();
        self.page_error = None;
        self.input_error = None;
        self.page_depth_error = None;
        self.input_depth_error = None;
        self.save_error = None;
    }
}

pub(super) fn blend(base: Color32, choice: TintChoice) -> Color32 {
    debug_assert!(choice.depth <= 100);
    let depth = u32::from(choice.depth);
    let channel = |base: u8, selected: u8| {
        ((u32::from(base) * (500 - depth) + u32::from(selected) * depth + 250) / 500) as u8
    };
    Color32::from_rgb(
        channel(base.r(), choice.rgb[0]),
        channel(base.g(), choice.rgb[1]),
        channel(base.b(), choice.rgb[2]),
    )
}

pub(super) fn parse_hex(value: &str) -> Result<[u8; 3], String> {
    let Some(hex) = value.strip_prefix('#') else {
        return Err("色は#に続く6桁の16進数で指定してください。".into());
    };
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("色は#に続く6桁の16進数で指定してください。".into());
    }
    Ok([
        u8::from_str_radix(&hex[0..2], 16).expect("validated hex"),
        u8::from_str_radix(&hex[2..4], 16).expect("validated hex"),
        u8::from_str_radix(&hex[4..6], 16).expect("validated hex"),
    ])
}

pub(super) fn format_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

pub(super) fn parse_depth(value: &str) -> Result<u8, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("濃さは0から100までの整数で指定してください。".into());
    }
    value
        .parse::<u8>()
        .ok()
        .filter(|depth| *depth <= 100)
        .ok_or_else(|| "濃さは0から100までの整数で指定してください。".into())
}

pub(super) fn contrast_ratio(left: Color32, right: Color32) -> f64 {
    fn linear(channel: u8) -> f64 {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(color: Color32) -> f64 {
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }
    let (light, dark) = (luminance(left), luminance(right));
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}

const INPUT_BORDER: Color32 = Color32::from_rgb(73, 90, 109);
const INPUT_FOCUS: Color32 = Color32::from_rgb(20, 80, 150);

/// Apply input visuals to exactly one widget on the same Ui, preserving its auto ID.
pub(super) fn editable_input<R>(
    ui: &mut egui::Ui,
    choice: TintChoice,
    base: Color32,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let previous = ui.style().clone();
    let fill = blend(base, choice);
    let enabled = ui.is_enabled();
    {
        let visuals = &mut ui.style_mut().visuals;
        visuals.extreme_bg_color = fill;
        if enabled {
            let border = egui::Stroke::new(1.0_f32, INPUT_BORDER);
            visuals.widgets.inactive.bg_stroke = border;
            visuals.widgets.hovered.bg_stroke = border;
            visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0_f32, INPUT_FOCUS);
            // egui 0.31.1 paints a focused TextEdit frame with selection.stroke.
            visuals.selection.stroke = egui::Stroke::new(2.0_f32, INPUT_FOCUS);
        }
        let text = visuals.override_text_color.unwrap_or(ux::INK);
        if contrast_ratio(text, fill) < 4.5 {
            visuals.override_text_color = Some(ux::INK);
        }
    }
    let response = add(ui);
    ui.set_style(previous);
    response
}

pub(super) fn input_frame(base: Color32, choice: TintChoice, enabled: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(blend(base, choice))
        .stroke(egui::Stroke::new(
            1.0_f32,
            if enabled { INPUT_BORDER } else { ux::BORDER },
        ))
}

pub(super) fn input_focus_outline(ui: &egui::Ui, rect: egui::Rect, focused: bool) {
    if focused && ui.is_enabled() {
        ui.painter().rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(2.0_f32, INPUT_FOCUS),
            egui::StrokeKind::Outside,
        );
    }
}

impl WordApp {
    pub(super) fn begin_color_editor(&mut self) {
        if let Some(editor) = &mut self.color_editor {
            editor.focused = true;
            return;
        }
        if self.pending_restore.is_some() || self.backup_restore.is_some() {
            self.message = "復元の確認を終えてから配色を調整してください。".into();
            return;
        }
        if self.settings_changed() {
            self.settings_editor.leave = Some(settings_edit::Destination::ColorEditor);
            return;
        }
        self.color_editor = Some(ColorEditor::new(&self.progress.settings));
    }

    pub(super) fn effective_tint(&self) -> ColorChoices {
        self.color_editor.as_ref().map_or_else(
            || ColorChoices::from_settings(&self.progress.settings),
            |editor| editor.draft,
        )
    }

    pub(super) fn cancel_color_editor(&mut self) {
        self.color_editor = None;
    }

    pub(super) fn save_color_editor(&mut self) -> Result<(), String> {
        let editor = self
            .color_editor
            .as_ref()
            .ok_or("配色の編集は開いていません。")?;
        if editor.invalid() {
            return Err("色と濃さの入力を修正してから保存してください。".into());
        }
        let draft = editor.draft;
        if let Some(error) = &self.fatal {
            return Err(error.clone());
        }
        self.credit_time();
        let mut next = self.progress.clone();
        next.settings.page_tint = draft.page;
        next.settings.input_tint = draft.input;
        next.validate()?;
        self.storage
            .as_ref()
            .ok_or("保存先がありません。")?
            .save(&next)?;
        self.progress = next;
        self.dirty = false;
        self.last_save = Instant::now();
        self.color_editor = None;
        self.message = "配色を保存した。".into();
        Ok(())
    }

    pub(super) fn color_editor_window(&mut self, ctx: &egui::Context) {
        if self.color_editor.is_none() {
            return;
        }
        let escape = ctx.input(|input| input.key_pressed(egui::Key::Escape));
        // The color picker consumes Escape itself. Remember its state before it closes.
        let popup_was_open = ctx.memory(|memory| memory.any_popup_open());
        let width = (ctx.screen_rect().width() - 24.0).min(460.0).max(1.0);
        // Keep room for the native title, frame margins, separator and wrapped footer.
        // The scroll body alone may shrink; the footer must stay outside it.
        let height = (ctx.screen_rect().height() - 40.0).max(1.0);
        let body_height = (height - 112.0).clamp(1.0, 430.0);
        let mut open = true;
        let mut action = 0;
        let mut widget_focused = false;
        let window = egui::Window::new("配色を調整")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(width)
            .max_width(width)
            .max_height(height)
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.set_width(ui.available_width().min((width - 16.0).max(1.0)));
                let editor = self.color_editor.as_mut().expect("open editor");
                ui.add_enabled_ui(!editor.leave_close, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("color-editor-body")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                        .max_height(body_height)
                        .show(ui, |ui| {
                            ui.add(egui::Label::new("選んだ色を既存の明るい背景へ最大20%混ぜる。濃さ0では元の色、100で最も濃い明色になる。").wrap());
                            ui.add_space(8.0);
                            let input_tint = editor.draft.input;
                            widget_focused |= color_controls(
                                ui,
                                "画面・ダイアログの背景",
                                Color32::from_rgb(247, 250, 253),
                                input_tint,
                                &mut editor.draft.page,
                                &mut editor.page_hex,
                                &mut editor.page_depth_text,
                                &mut editor.page_error,
                                &mut editor.page_depth_error,
                            );
                            ui.separator();
                            widget_focused |= color_controls(
                                ui,
                                "入力欄の背景",
                                Color32::WHITE,
                                input_tint,
                                &mut editor.draft.input,
                                &mut editor.input_hex,
                                &mut editor.input_depth_text,
                                &mut editor.input_error,
                                &mut editor.input_depth_error,
                            );
                            if let Some(error) = &editor.save_error {
                                ui.colored_label(Color32::DARK_RED, format!("配色を保存できなかった：{error}"));
                            }
                        });
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let save = ui.add_enabled(
                            !editor.invalid() && self.fatal.is_none(),
                            crate::app::controls::Button::new("保存"),
                        );
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| data.insert_temp(egui::Id::new("feedback-ui-palette-save"), (save.rect, save.enabled())));
                        widget_focused |= save.has_focus();
                        if save.clicked() {
                            action = 1;
                        }
                        let cancel = ui.ww_button("キャンセル");
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| data.insert_temp(egui::Id::new("feedback-ui-palette-cancel"), cancel.rect));
                        widget_focused |= cancel.has_focus();
                        if cancel.clicked() {
                            action = 2;
                        }
                        let reset = ui.ww_button("標準色に戻す");
                        widget_focused |= reset.has_focus();
                        if reset.clicked() {
                            action = 3;
                        }
                    });
                });
            });
        if let Some(window) = window {
            if let Some(editor) = &mut self.color_editor {
                let pointer = ctx.input(|input| {
                    input
                        .pointer
                        .any_pressed()
                        .then(|| input.pointer.interact_pos())
                        .flatten()
                });
                if let Some(pointer) = pointer {
                    if window.response.rect.contains(pointer) {
                        editor.focused = true;
                    } else if !popup_was_open {
                        editor.focused = false;
                    }
                }
                if widget_focused {
                    editor.focused = true;
                } else if pointer.is_none()
                    && ctx.memory(|memory| memory.focused().is_some())
                    && !popup_was_open
                {
                    editor.focused = false;
                }
            }
        }
        if !open || action == 2 {
            self.cancel_color_editor();
            return;
        }
        if let Some(editor) = &self.color_editor {
            if escape && editor.focused && !popup_was_open && !editor.leave_close {
                self.cancel_color_editor();
                return;
            }
        }
        match action {
            1 => {
                if let Err(error) = self.save_color_editor() {
                    if let Some(editor) = &mut self.color_editor {
                        editor.save_error = Some(error);
                    }
                }
            }
            3 => {
                if let Some(editor) = &mut self.color_editor {
                    editor.reset();
                }
            }
            _ => {}
        }
        self.color_close_dialog(ctx);
    }

    fn color_close_dialog(&mut self, ctx: &egui::Context) {
        if !self
            .color_editor
            .as_ref()
            .is_some_and(|editor| editor.leave_close)
        {
            return;
        }
        let mut action = 0;
        let escape = ctx.input(|input| input.key_pressed(egui::Key::Escape));
        let popup_was_open = ctx.memory(|memory| memory.any_popup_open());
        egui::Window::new("未保存の配色変更")
            .collapsible(false)
            .resizable(false)
            .max_width((ctx.screen_rect().width() - 24.0).max(1.0))
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.label("配色はまだ保存されていない。終了前に変更をどう扱うか選択してください。");
                if let Some(error) = self
                    .color_editor
                    .as_ref()
                    .and_then(|editor| editor.save_error.as_ref())
                {
                    ui.colored_label(
                        Color32::DARK_RED,
                        format!("配色を保存できなかった：{error}"),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    let valid = self
                        .color_editor
                        .as_ref()
                        .is_some_and(|editor| !editor.invalid());
                    if ui
                        .add_enabled(valid, crate::app::controls::Button::new("保存して終了"))
                        .clicked()
                    {
                        action = 1;
                    }
                    if ui.ww_button("破棄して終了").clicked() {
                        action = 2;
                    }
                    if ui.ww_button("終了を中止").clicked() {
                        action = 3;
                    }
                });
            });
        if escape && !popup_was_open {
            action = 3;
        }
        match action {
            1 => match self.save_color_editor() {
                Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                Err(error) => {
                    if let Some(editor) = &mut self.color_editor {
                        editor.save_error = Some(error);
                    }
                }
            },
            2 => {
                self.cancel_color_editor();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            3 => {
                if let Some(editor) = &mut self.color_editor {
                    editor.leave_close = false;
                }
            }
            _ => {}
        }
    }
}

fn color_controls(
    ui: &mut egui::Ui,
    title: &str,
    base: Color32,
    input_tint: TintChoice,
    choice: &mut TintChoice,
    hex: &mut String,
    depth_text: &mut String,
    color_error: &mut Option<String>,
    depth_error: &mut Option<String>,
) -> bool {
    ui.strong(title);
    ui.label("色（#RRGGBB）");
    let mut focused = false;
    ui.horizontal_wrapped(|ui| {
        let picker = ui.color_edit_button_srgb(&mut choice.rgb);
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new(("feedback-ui-palette-picker", title)),
                picker.rect,
            )
        });
        focused |= picker.has_focus();
        if picker.changed() {
            *hex = format_hex(choice.rgb);
            *color_error = None;
        }
        let response = editable_input(ui, input_tint, Color32::WHITE, |ui| {
            ui.add(egui::TextEdit::singleline(hex).desired_width(150.0))
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new(("feedback-ui-palette-hex", title)),
                (response.rect, response.id),
            )
        });
        focused |= response.has_focus();
        if response.changed() {
            match parse_hex(hex) {
                Ok(rgb) => {
                    choice.rgb = rgb;
                    *color_error = None;
                }
                Err(error) => *color_error = Some(error),
            }
        }
        if response.lost_focus() && color_error.is_none() {
            *hex = format_hex(choice.rgb);
        }
    });
    if let Some(error) = color_error.as_ref() {
        ui.colored_label(Color32::DARK_RED, error);
    }
    ui.label("濃さ（0～100）");
    ui.horizontal_wrapped(|ui| {
        let slider = ui.add(
            egui::Slider::new(&mut choice.depth, 0..=100)
                .step_by(1.0)
                .show_value(false),
        );
        focused |= slider.has_focus();
        if slider.changed() {
            *depth_text = choice.depth.to_string();
            *depth_error = None;
        }
        let response = editable_input(ui, input_tint, Color32::WHITE, |ui| {
            ui.add(egui::TextEdit::singleline(depth_text).desired_width(64.0))
        });
        #[cfg(test)]
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                egui::Id::new(("feedback-ui-palette-depth", title)),
                (response.rect, response.id),
            )
        });
        focused |= response.has_focus();
        if response.changed() {
            match parse_depth(depth_text) {
                Ok(depth) => {
                    choice.depth = depth;
                    *depth_error = None;
                }
                Err(error) => *depth_error = Some(error),
            }
        }
        if response.lost_focus() && depth_error.is_none() {
            *depth_text = choice.depth.to_string();
        }
    });
    if let Some(error) = depth_error.as_ref() {
        ui.colored_label(Color32::DARK_RED, error);
    }
    let effective = blend(base, *choice);
    egui::Frame::new()
        .fill(effective)
        .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
        .inner_margin(8)
        .show(ui, |ui| {
            ui.label(format!(
                "見本　{}",
                format_hex([effective.r(), effective.g(), effective.b()])
            ));
        });
    focused
}

#[cfg(test)]
mod feedback_ui_tests {
    use super::*;
    use crate::app::harness_tests::{fixture, frame};

    fn draw(ctx: &egui::Context, app: &mut WordApp, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1120.0, 850.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.update_ui(ctx),
        )
    }

    fn visible_label(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
        fn find(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Rect> {
            match shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                    Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
                }
                egui::epaint::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, label)),
                _ => None,
            }
        }
        output.shapes.iter().find_map(|shape| {
            let rect = find(&shape.shape, label)?;
            shape.clip_rect.contains_rect(rect).then_some(rect)
        })
    }

    fn click_label(ctx: &egui::Context, app: &mut WordApp, label: &str) -> egui::FullOutput {
        let mut output = draw(ctx, app, vec![]);
        for _ in 0..2 {
            output = draw(ctx, app, vec![]);
        }
        let pos = visible_label(&output, label)
            .unwrap_or_else(|| panic!("visible action {label:?} was not drawn"))
            .center();
        draw(
            ctx,
            app,
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
        draw(
            ctx,
            app,
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

    fn replace_input(ctx: &egui::Context, app: &mut WordApp, field: (&str, &str), value: &str) {
        draw(ctx, app, vec![]);
        let (rect, id): (egui::Rect, egui::Id) = ctx.data(|data| {
            data.get_temp(egui::Id::new(field))
                .expect("palette field rect")
        });
        let pos = rect.center();
        draw(
            ctx,
            app,
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
        draw(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(id),
            "palette input did not receive focus: {field:?}"
        );
        draw(
            ctx,
            app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                egui::Event::Text(value.into()),
            ],
        );
    }

    #[test]
    fn feedback_ui_palette_real_hex_and_depth_edit_reject_invalid_without_losing_preview() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        app.begin_color_editor();
        for _ in 0..4 {
            draw(&ctx, &mut app, vec![]);
        }
        replace_input(
            &ctx,
            &mut app,
            ("feedback-ui-palette-hex", "画面・ダイアログの背景"),
            "#0000FF",
        );
        replace_input(
            &ctx,
            &mut app,
            ("feedback-ui-palette-depth", "画面・ダイアログの背景"),
            "100",
        );
        let blue = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        assert_eq!(app.effective_tint().page, blue);
        replace_input(
            &ctx,
            &mut app,
            ("feedback-ui-palette-hex", "画面・ダイアログの背景"),
            "#12",
        );
        assert_eq!(
            app.effective_tint().page,
            blue,
            "invalid HEX changed last valid preview"
        );
        assert!(app.color_editor.as_ref().unwrap().page_error.is_some());
        let (_, enabled): (egui::Rect, bool) = ctx.data(|data| {
            data.get_temp(egui::Id::new("feedback-ui-palette-save"))
                .unwrap()
        });
        assert!(!enabled, "invalid HEX must disable save");
        replace_input(
            &ctx,
            &mut app,
            ("feedback-ui-palette-hex", "画面・ダイアログの背景"),
            "#0000FF",
        );
        replace_input(
            &ctx,
            &mut app,
            ("feedback-ui-palette-depth", "画面・ダイアログの背景"),
            "101",
        );
        assert_eq!(
            app.effective_tint().page,
            blue,
            "out-of-range depth changed preview"
        );
        assert!(app
            .color_editor
            .as_ref()
            .unwrap()
            .page_depth_error
            .is_some());
        let (_, enabled): (egui::Rect, bool) = ctx.data(|data| {
            data.get_temp(egui::Id::new("feedback-ui-palette-save"))
                .unwrap()
        });
        assert!(!enabled, "invalid depth must disable save");
        draw(&ctx, &mut app, vec![]);
        let rect: egui::Rect = ctx.data(|data| {
            data.get_temp(egui::Id::new("feedback-ui-palette-cancel"))
                .unwrap()
        });
        let pos = rect.center();
        draw(
            &ctx,
            &mut app,
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
        draw(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert!(app.color_editor.is_none());
        assert_eq!(app.progress.settings.page_tint, TintChoice::default());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_palette_focused_escape_cancels_but_picker_popup_escape_does_not() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        app.begin_color_editor();
        app.color_editor.as_mut().unwrap().draft.page = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        for _ in 0..8 {
            draw(&ctx, &mut app, vec![]);
        }
        let picker: egui::Rect = ctx.data(|data| {
            data.get_temp(egui::Id::new((
                "feedback-ui-palette-picker",
                "画面・ダイアログの背景",
            )))
            .expect("picker rect")
        });
        let pos = picker.center();
        draw(
            &ctx,
            &mut app,
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
        draw(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert!(
            ctx.memory(|memory| memory.any_popup_open()),
            "real color picker did not open"
        );
        let escape = egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        draw(&ctx, &mut app, vec![escape.clone()]);
        assert!(
            app.color_editor.is_some(),
            "picker Escape cancelled the whole editor"
        );
        draw(&ctx, &mut app, vec![]);
        assert!(
            !ctx.memory(|memory| memory.any_popup_open()),
            "picker popup stayed open after Escape"
        );
        draw(&ctx, &mut app, vec![escape]);
        assert!(
            app.color_editor.is_none(),
            "focused editor Escape did not cancel"
        );
        assert_eq!(app.progress.settings.page_tint, TintChoice::default());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_palette_input_frame_paint_and_contrast_cover_color_extremes_and_disabled() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(180.0, 36.0));
        for rgb in [
            [0, 0, 0],
            [255, 255, 255],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
        ] {
            for depth in [0, 100] {
                let choice = TintChoice { rgb, depth };
                let fill = blend(Color32::WHITE, choice);
                let painted = input_frame(Color32::WHITE, choice, true).paint(rect);
                let egui::Shape::Rect(painted) = painted else {
                    panic!("enabled input frame is not a rect")
                };
                assert_eq!(
                    painted.fill, fill,
                    "actual frame paint lost the selected input tint: {choice:?}"
                );
                assert_eq!(painted.stroke.color, INPUT_BORDER);
                assert!(
                    contrast_ratio(ux::INK, fill) >= 4.5,
                    "input text contrast below 4.5: {choice:?}"
                );
                assert!(
                    contrast_ratio(INPUT_BORDER, Color32::WHITE) >= 3.0,
                    "input border below 3:1"
                );
                assert!(
                    contrast_ratio(INPUT_FOCUS, Color32::WHITE) >= 3.0,
                    "focus outline below 3:1"
                );
                let disabled = input_frame(Color32::WHITE, choice, false).paint(rect);
                let egui::Shape::Rect(disabled) = disabled else {
                    panic!("disabled input frame is not a rect")
                };
                assert_eq!(disabled.fill, fill);
                assert_ne!(
                    disabled.stroke.color, INPUT_BORDER,
                    "disabled frame looks enabled"
                );
            }
        }
    }

    #[test]
    fn feedback_ui_palette_editable_text_paints_only_the_input_and_preserves_other_widget_style() {
        let ctx = egui::Context::default();
        let choice = TintChoice {
            rgb: [255, 0, 0],
            depth: 100,
        };
        let expected = Color32::from_rgb(255, 204, 204);
        let mut value = String::new();
        let mut disabled_value = String::new();
        let mut input_rect = None;
        let mut button_rect = None;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500.0, 280.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let before = ui.style().clone();
                    let input = editable_input(ui, choice, Color32::WHITE, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut value).desired_width(180.0))
                    });
                    assert!(input.enabled());
                    input_rect = Some(input.rect);
                    assert_eq!(
                        ui.style().visuals.extreme_bg_color,
                        before.visuals.extreme_bg_color
                    );
                    assert_eq!(
                        ui.style().visuals.widgets.inactive.bg_stroke,
                        before.visuals.widgets.inactive.bg_stroke
                    );
                    ui.label("読み取り専用");
                    let disabled = ui
                        .add_enabled_ui(false, |ui| {
                            editable_input(ui, choice, Color32::WHITE, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut disabled_value)
                                        .desired_width(180.0),
                                )
                            })
                        })
                        .inner;
                    assert!(!disabled.enabled(), "disabled TextEdit became operable");
                    let button = ui.button("通常ボタン");
                    button_rect = Some(button.rect);
                    assert_eq!(
                        ui.style().visuals.extreme_bg_color,
                        before.visuals.extreme_bg_color
                    );
                    assert_eq!(
                        ui.style().visuals.widgets.inactive.bg_stroke,
                        before.visuals.widgets.inactive.bg_stroke
                    );
                });
            },
        );
        let input_rect = input_rect.unwrap();
        let button_rect = button_rect.unwrap();
        fn tinted(shape: &egui::Shape, expected: Color32, target: egui::Rect) -> bool {
            match shape {
                egui::Shape::Rect(rect) => rect.fill == expected && rect.rect.contains_rect(target),
                egui::Shape::Vec(parts) => parts.iter().any(|part| tinted(part, expected, target)),
                _ => false,
            }
        }
        assert!(
            output
                .shapes
                .iter()
                .any(|shape| tinted(&shape.shape, expected, input_rect)),
            "empty editable TextEdit did not paint the chosen input color"
        );
        assert!(
            !output
                .shapes
                .iter()
                .any(|shape| tinted(&shape.shape, expected, button_rect)),
            "button inherited the input tint"
        );
    }

    #[test]
    fn feedback_ui_palette_blend_parse_and_contrast_match_specified_boundaries() {
        let white = Color32::WHITE;
        let page = Color32::from_rgb(247, 250, 253);
        let blue = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        let none = TintChoice {
            rgb: [255, 0, 0],
            depth: 0,
        };
        assert_eq!(blend(white, none), white);
        assert_eq!(blend(page, none), page);
        assert_eq!(blend(white, blue), Color32::from_rgb(204, 204, 255));
        assert_eq!(blend(page, blue), Color32::from_rgb(198, 200, 253));
        for rgb in [
            [0, 0, 0],
            [255, 255, 255],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
        ] {
            assert_eq!(blend(page, TintChoice { rgb, depth: 0 }), page);
        }
        assert_eq!(parse_hex("#aBcD09").unwrap(), [0xAB, 0xCD, 0x09]);
        assert_eq!(format_hex([0xAB, 0xCD, 0x09]), "#ABCD09");
        for invalid in ["", "123456", "#12", "#GG0000", "#00000000", "#１２３４５６"] {
            assert!(
                parse_hex(invalid).is_err(),
                "invalid HEX accepted: {invalid:?}"
            );
        }
        for valid in ["0", "1", "99", "100"] {
            assert!(parse_depth(valid).is_ok());
        }
        for invalid in ["", "-1", "101", "1.5", "4a", " 40", "４０"] {
            assert!(
                parse_depth(invalid).is_err(),
                "invalid depth accepted: {invalid:?}"
            );
        }
        assert!((contrast_ratio(Color32::BLACK, white) - 21.0).abs() < 0.001);
        assert!((contrast_ratio(white, white) - 1.0).abs() < 0.001);
    }

    #[test]
    fn feedback_ui_palette_save_preserves_latest_progress_and_autosave_excludes_preview() {
        let (ctx, mut app, root) = fixture();
        let original = app.progress.settings.clone();
        app.begin_color_editor();
        let blue = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        let red = TintChoice {
            rgb: [255, 0, 0],
            depth: 40,
        };
        let editor = app.color_editor.as_mut().unwrap();
        editor.draft.page = blue;
        editor.draft.input = red;
        assert_eq!(app.effective_tint().page, blue);
        assert_eq!(app.progress.settings, original);
        app.progress.settings.speech_volume = 0.3;
        app.progress.chats[0].draft = "配色中の未送信会話".into();
        app.progress.study_seconds.insert("2026-09-27".into(), 42);
        app.dirty = true;
        app.last_save = Instant::now() - Duration::from_secs(30);
        frame(&ctx, &mut app, false);
        let autosaved = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(autosaved.settings.page_tint, original.page_tint);
        assert_eq!(autosaved.settings.input_tint, original.input_tint);
        assert_eq!(autosaved.chats[0].draft, "配色中の未送信会話");
        app.save_color_editor().unwrap();
        assert!(app.color_editor.is_none());
        let saved = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(saved.settings.page_tint, blue);
        assert_eq!(saved.settings.input_tint, red);
        assert_eq!(saved.settings.speech_volume, 0.3);
        assert_eq!(saved.chats[0].draft, "配色中の未送信会話");
        assert_eq!(saved.study_seconds["2026-09-27"], 42);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_palette_invalid_edit_and_save_failure_keep_retry_or_cancel() {
        let (_, mut app, root) = fixture();
        app.begin_color_editor();
        let blue = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        let editor = app.color_editor.as_mut().unwrap();
        editor.draft.page = blue;
        editor.page_hex = "#12".into();
        editor.page_error = parse_hex(&editor.page_hex).err();
        assert!(app.save_color_editor().is_err());
        assert_eq!(
            app.effective_tint().page,
            blue,
            "invalid text must retain last valid preview"
        );
        let editor = app.color_editor.as_mut().unwrap();
        editor.page_hex = "#0000FF".into();
        editor.page_error = None;
        let data = app.storage.as_ref().unwrap().dir.clone();
        let destination = data.join("progress.json");
        let original = data.join("palette-before-failed-save.json");
        std::fs::rename(&destination, &original).unwrap();
        let disk_before = std::fs::read(&original).unwrap();
        std::fs::create_dir(&destination).unwrap();
        assert!(app.save_color_editor().is_err());
        assert_eq!(app.progress.settings.page_tint, TintChoice::default());
        assert_eq!(app.effective_tint().page, blue);
        assert_eq!(std::fs::read(&original).unwrap(), disk_before);
        std::fs::remove_dir(&destination).unwrap();
        std::fs::rename(&original, &destination).unwrap();
        app.save_color_editor().unwrap();
        assert_eq!(
            app.storage
                .as_ref()
                .unwrap()
                .load()
                .unwrap()
                .settings
                .page_tint,
            blue
        );
        app.begin_color_editor();
        app.color_editor.as_mut().unwrap().draft.page = TintChoice {
            rgb: [255, 0, 0],
            depth: 100,
        };
        app.progress.chats[0].draft = "取消しでも保持".into();
        app.cancel_color_editor();
        assert_eq!(app.effective_tint().page, blue);
        assert_eq!(app.progress.chats[0].draft, "取消しでも保持");
        assert_eq!(
            app.storage
                .as_ref()
                .unwrap()
                .load()
                .unwrap()
                .settings
                .page_tint,
            blue
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_palette_dirty_settings_requires_explicit_continue_discard_or_save() {
        for action in ["編集を続ける", "破棄して進む", "保存して進む"] {
            let (ctx, mut app, root) = fixture();
            app.page = Page::Settings;
            app.begin_settings_edit();
            app.settings_editor.draft.minutes = 23;
            app.begin_color_editor();
            assert!(app.color_editor.is_none());
            assert!(matches!(
                app.settings_editor.leave,
                Some(settings_edit::Destination::ColorEditor)
            ));
            assert_eq!(app.progress.settings.minutes, 5);
            click_label(&ctx, &mut app, action);
            match action {
                "編集を続ける" => {
                    assert!(app.color_editor.is_none());
                    assert_eq!(app.settings_editor.draft.minutes, 23);
                    assert_eq!(app.progress.settings.minutes, 5);
                }
                "破棄して進む" => {
                    assert!(app.color_editor.is_some());
                    assert_eq!(app.progress.settings.minutes, 5);
                }
                _ => {
                    assert!(app.color_editor.is_some());
                    assert_eq!(app.progress.settings.minutes, 23);
                    assert_eq!(
                        app.storage
                            .as_ref()
                            .unwrap()
                            .load()
                            .unwrap()
                            .settings
                            .minutes,
                        23
                    );
                }
            }
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn feedback_ui_palette_blocks_settings_and_json_restore_and_guards_close() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        app.begin_settings_edit();
        app.begin_color_editor();
        app.color_editor.as_mut().unwrap().draft.page = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        let before = serde_json::to_value(&app.progress).unwrap();
        assert!(app.save_settings(&ctx).is_err());
        let mut restored = app.progress.clone();
        restored.settings.minutes = 44;
        assert!(app.restore(restored).is_err());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        let output = frame(&ctx, &mut app, true);
        assert!(output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&egui::ViewportCommand::CancelClose));
        assert!(app.color_editor.as_ref().unwrap().leave_close);
        click_label(&ctx, &mut app, "終了を中止");
        assert!(!app.color_editor.as_ref().unwrap().leave_close);
        let output = frame(&ctx, &mut app, true);
        assert!(output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&egui::ViewportCommand::CancelClose));
        click_label(&ctx, &mut app, "破棄して終了");
        assert!(app.color_editor.is_none());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(
            app.storage
                .as_ref()
                .unwrap()
                .load()
                .unwrap()
                .settings
                .page_tint,
            TintChoice::default()
        );
        app.pending_restore = Some(app.progress.clone());
        app.begin_color_editor();
        assert!(
            app.color_editor.is_none(),
            "pending JSON restore requires an explicit decision first"
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn feedback_ui_palette_close_save_commits_only_on_success_and_failed_save_keeps_window() {
        let (ctx, mut app, root) = fixture();
        app.begin_color_editor();
        let blue = TintChoice {
            rgb: [0, 0, 255],
            depth: 100,
        };
        app.color_editor.as_mut().unwrap().draft.page = blue;
        frame(&ctx, &mut app, true);
        let output = click_label(&ctx, &mut app, "保存して終了");
        assert!(app.color_editor.is_none());
        assert_eq!(
            app.storage
                .as_ref()
                .unwrap()
                .load()
                .unwrap()
                .settings
                .page_tint,
            blue
        );
        assert!(output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&egui::ViewportCommand::Close));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();

        let (ctx, mut app, root) = fixture();
        app.begin_color_editor();
        app.color_editor.as_mut().unwrap().draft.page = blue;
        let storage = app.storage.take();
        frame(&ctx, &mut app, true);
        let output = click_label(&ctx, &mut app, "保存して終了");
        assert!(app.color_editor.as_ref().unwrap().leave_close);
        assert!(app.color_editor.as_ref().unwrap().save_error.is_some());
        assert_eq!(app.progress.settings.page_tint, TintChoice::default());
        assert!(!output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .contains(&egui::ViewportCommand::Close));
        app.storage = storage;
        assert_eq!(
            app.storage
                .as_ref()
                .unwrap()
                .load()
                .unwrap()
                .settings
                .page_tint,
            TintChoice::default()
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
