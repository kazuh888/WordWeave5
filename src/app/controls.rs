//! Native egui interaction, with visible Japanese glyphs aligned inside controls.
use eframe::egui::*;

fn start(ui: &Ui) -> usize {
    ui.ctx()
        .graphics_mut(|g| g.entry(ui.layer_id()).next_idx().0)
}

fn center(ui: &Ui, first: usize, rect: Rect, horizontal: bool) {
    ui.ctx().graphics_mut(|graphics| {
        let list = graphics.entry(ui.layer_id());
        let end = list.next_idx().0;
        for index in first..end {
            list.mutate_shape(layers::ShapeIdx(index), |shape| {
                if let Shape::Text(text) = &mut shape.shape {
                    // Collapsing/menu contents must not be moved with their header.
                    if !rect.contains(text.pos + text.galley.rect.center().to_vec2()) {
                        return;
                    }
                    let ink = text.galley.mesh_bounds;
                    if !ink.is_finite() || !ink.is_positive() {
                        return;
                    }
                    text.pos.y = rect.center().y - ink.center().y;
                    if horizontal {
                        text.pos.x = rect.center().x - ink.center().x;
                    }
                }
            });
        }
    });
}

#[must_use]
pub(crate) struct Button<'a>(eframe::egui::Button<'a>);

// List rows have left-aligned text, unlike centered action buttons.
pub(crate) fn list_row(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let first = start(ui);
    let response = ui.add_sized(
        vec2(ui.available_width(), 44.0),
        eframe::egui::Button::new(text)
            .truncate()
            .selected(selected)
            .fill(if selected {
                Color32::from_rgb(226, 241, 255)
            } else {
                Color32::WHITE
            })
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(216, 232, 244)))
            .corner_radius(6),
    );
    center(ui, first, response.rect, false);
    ui.ctx().graphics_mut(|graphics| {
        let list = graphics.entry(ui.layer_id());
        for index in first..list.next_idx().0 {
            list.mutate_shape(layers::ShapeIdx(index), |shape| {
                if let Shape::Text(t) = &mut shape.shape {
                    t.pos.x = response.rect.left() + ui.spacing().button_padding.x;
                }
            });
        }
    });
    response.on_hover_text(text)
}
impl<'a> Button<'a> {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self(eframe::egui::Button::new(text))
    }
    pub fn fill(mut self, value: impl Into<Color32>) -> Self {
        self.0 = self.0.fill(value);
        self
    }
    pub fn stroke(mut self, value: impl Into<Stroke>) -> Self {
        self.0 = self.0.stroke(value);
        self
    }
    pub fn min_size(mut self, value: Vec2) -> Self {
        self.0 = self.0.min_size(value);
        self
    }
    pub fn corner_radius(mut self, value: impl Into<CornerRadius>) -> Self {
        self.0 = self.0.corner_radius(value);
        self
    }
    pub fn wrap(mut self) -> Self {
        self.0 = self.0.wrap();
        self
    }
    pub fn frame(mut self, value: bool) -> Self {
        self.0 = self.0.frame(value);
        self
    }
    pub fn small(mut self) -> Self {
        self.0 = self.0.small();
        self
    }
}
impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let first = start(ui);
        let response = self.0.ui(ui);
        center(ui, first, response.rect, true);
        response
    }
}

pub(crate) trait UiControls {
    fn ww_button(&mut self, text: impl Into<WidgetText>) -> Response;
    fn ww_small_button(&mut self, text: impl Into<WidgetText>) -> Response;
    fn ww_selectable_label(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response;
    fn ww_selectable_value<T: PartialEq>(
        &mut self,
        current: &mut T,
        value: T,
        text: impl Into<WidgetText>,
    ) -> Response;
    fn ww_collapsing<R>(
        &mut self,
        text: impl Into<WidgetText>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;
    fn ww_menu_button<R>(
        &mut self,
        text: impl Into<WidgetText>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>>;
}
impl UiControls for Ui {
    fn ww_button(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add(Button::new(text))
    }
    fn ww_small_button(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add(Button::new(text).small())
    }
    fn ww_selectable_label(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response {
        let first = start(self);
        let response = self.selectable_label(selected, text);
        center(self, first, response.rect, true);
        response
    }
    fn ww_selectable_value<T: PartialEq>(
        &mut self,
        current: &mut T,
        value: T,
        text: impl Into<WidgetText>,
    ) -> Response {
        let first = start(self);
        let response = self.selectable_value(current, value, text);
        center(self, first, response.rect, true);
        response
    }
    fn ww_collapsing<R>(
        &mut self,
        text: impl Into<WidgetText>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        let first = start(self);
        let response = self.collapsing(text, content);
        center(self, first, response.header_response.rect, false);
        response
    }
    fn ww_menu_button<R>(
        &mut self,
        text: impl Into<WidgetText>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let first = start(self);
        let response = self.menu_button(text, content);
        center(self, first, response.response.rect, true);
        response
    }
}

pub(crate) struct CollapsingHeader(eframe::egui::CollapsingHeader);
impl CollapsingHeader {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self(eframe::egui::CollapsingHeader::new(text))
    }
    pub fn id_salt(mut self, id: impl std::hash::Hash) -> Self {
        self.0 = self.0.id_salt(id);
        self
    }
    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R> {
        let first = start(ui);
        let response = self.0.show(ui, body);
        center(ui, first, response.header_response.rect, false);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn button_keeps_keyboard_activation_and_disabled_semantics() {
        let (ctx, app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(1.0);
        for enabled in [true, false] {
            let mut clicked = false;
            for frame in 0..3 {
                let events = if frame == 1 {
                    vec![Event::Key {
                        key: Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Modifiers::NONE,
                    }]
                } else {
                    vec![]
                };
                let _ = ctx.run(
                    RawInput {
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        CentralPanel::default().show(ctx, |ui| {
                            let response = ui.add_enabled(enabled, Button::new("回答を確認"));
                            if frame == 0 {
                                response.request_focus();
                            }
                            clicked |= response.clicked();
                            assert_eq!(response.enabled(), enabled);
                        });
                    },
                );
            }
            assert_eq!(clicked, enabled);
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_controls_center_visible_ink_in_both_axes() {
        for scale in [0.6, 0.8, 1.0, 1.6] {
            let (ctx, app, root) = super::super::harness_tests::fixture();
            ctx.set_pixels_per_point(scale);
            for enabled in [true, false] {
                for kind in 0..4 {
                    let mut rect = Rect::NOTHING;
                    let out = ctx.run(RawInput::default(), |ctx| {
                        CentralPanel::default().show(ctx, |ui| {
                            ui.add_enabled_ui(enabled, |ui| {
                                rect = match kind {
                                    0 => {
                                        ui.add_sized([300.0, 60.0], Button::new("回答を確認")).rect
                                    }
                                    1 => {
                                        ui.add_sized(
                                            [180.0, 100.0],
                                            Button::new("長い日本語のボタンを複数行で表示する")
                                                .wrap(),
                                        )
                                        .rect
                                    }
                                    2 => ui.ww_selectable_label(true, "手書き入力").rect,
                                    _ => {
                                        ui.ww_collapsing("詳しい解説・例文を確認", |_| {})
                                            .header_response
                                            .rect
                                    }
                                };
                            });
                        });
                    });
                    let texts: Vec<_> = out
                        .shapes
                        .iter()
                        .filter_map(|s| match &s.shape {
                            Shape::Text(t) => Some(t),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(texts.len(), 1);
                    let ink = texts[0]
                        .galley
                        .mesh_bounds
                        .translate(texts[0].pos.to_vec2());
                    assert!(
                        (ink.center().y - rect.center().y).abs() <= 1.0 / scale,
                        "kind={kind} {ink:?} {rect:?}"
                    );
                    if kind != 3 {
                        assert!((ink.center().x - rect.center().x).abs() <= 1.0 / scale);
                    }
                }
            }
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
