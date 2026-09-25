//! Shared visual vocabulary. Width decisions use logical points after zoom.
use super::*;

pub(super) const ACCENT: Color32 = Color32::from_rgb(0, 100, 190);
pub(super) const INK: Color32 = Color32::from_rgb(22, 44, 65);
pub(super) const MUTED: Color32 = Color32::from_rgb(77, 96, 115);
pub(super) const BORDER: Color32 = Color32::from_rgb(211, 226, 239);
pub(super) const TINT: Color32 = Color32::from_rgb(235, 246, 255);

/// Make dialog content readable without changing the window title or the
/// typography of the underlying page.
pub(super) fn dialog_body(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    style
        .text_styles
        .insert(egui::TextStyle::Body, home_art::home_font(21.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, home_art::home_font(18.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, home_art::home_font(20.0));
    style.override_font_id = None;
}

pub(super) fn heading(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).size(28.0).strong().color(INK));
    if !description.is_empty() {
        ui.label(RichText::new(description).color(MUTED));
    }
    ui.add_space(16.0);
}

pub(super) fn panel<R>(
    ui: &mut egui::Ui,
    tint: bool,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let width = ui.available_width();
    egui::Frame::group(ui.style())
        .fill(if tint { TINT } else { Color32::WHITE })
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .inner_margin(16)
        .corner_radius(10)
        .show(ui, |ui| {
            ui.set_width((width - 34.0).max(0.0));
            content(ui)
        })
        .inner
}

pub(super) fn primary(ui: &mut egui::Ui, text: impl Into<String>, enabled: bool) -> egui::Response {
    let width = ui.available_width().min(320.0);
    ui.add_enabled(
        enabled,
        crate::app::controls::Button::new(
            RichText::new(text.into())
                .size(20.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(ACCENT)
        .min_size(egui::vec2(width, 48.0))
        .wrap(),
    )
}

pub(super) fn metric(ui: &mut egui::Ui, label: &str, value: impl Into<String>, note: &str) {
    ui.group(|ui| {
        ui.set_max_width(ui.available_width().min(240.0));
        ui.label(RichText::new(label).color(MUTED));
        ui.label(RichText::new(value.into()).size(26.0).strong().color(INK));
        if !note.is_empty() {
            ui.small(note);
        }
    });
}

pub(super) fn duration(seconds: u64) -> String {
    format!("{}分{}秒", seconds / 60, seconds % 60)
}

pub(super) fn metric_group(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    if ui.available_width() < 700.0 {
        ui.vertical(content);
    } else {
        ui.horizontal_wrapped(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_body_enlarges_content_without_changing_the_window_title() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let heading = ui.style().text_styles[&egui::TextStyle::Heading].clone();
                dialog_body(ui);
                assert_eq!(ui.style().text_styles[&egui::TextStyle::Body].size, 21.0);
                assert_eq!(ui.style().text_styles[&egui::TextStyle::Small].size, 18.0);
                assert_eq!(ui.style().text_styles[&egui::TextStyle::Button].size, 20.0);
                assert_eq!(ui.style().text_styles[&egui::TextStyle::Heading], heading);
                assert!(ui.style().override_font_id.is_none());
            });
        });
    }
}
