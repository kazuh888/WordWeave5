//! Playback-only view: native widgets retain keyboard/focus semantics; icons are vector painted.
use super::audio_controls::SpeechButton;
use super::*;

#[derive(Default)]
pub(super) struct Actions {
    pub back: bool,
    pub toggle: bool,
    pub stop: bool,
    pub forward: bool,
    pub seek: Option<f64>,
    pub rate: Option<f64>,
    pub close: bool,
    pub volume: Option<f64>,
    pub repeat: Option<bool>,
}

#[derive(Clone, Copy)]
enum Icon {
    Back,
    Play,
    Pause,
    Stop,
    Forward,
    Repeat,
}

fn pause_bar_rects(rect: egui::Rect) -> [egui::Rect; 2] {
    let center = rect.center();
    let scale = (rect.width() / 64.0).min(1.0);
    let point = |x, y| center + egui::vec2(x, y) * scale;
    [
        egui::Rect::from_min_max(point(-10.0, -14.0), point(-4.0, 14.0)),
        egui::Rect::from_min_max(point(4.0, -14.0), point(10.0, 14.0)),
    ]
}

fn remember(ui: &egui::Ui, key: &str, response: &egui::Response) {
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(("speech-control", key)), response.rect));
    #[cfg(not(test))]
    let _ = (ui, key, response);
}

fn icon(ui: &egui::Ui, rect: egui::Rect, kind: Icon, color: Color32) {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let c = rect.center();
    let scale = (rect.width() / 64.0).min(1.0);
    let p = |x, y| c + egui::vec2(x, y) * scale;
    let stroke = egui::Stroke::new(2.8_f32 * scale, color);
    match kind {
        Icon::Play => {
            painter.add(egui::Shape::convex_polygon(
                vec![p(-8.0, -14.0), p(14.0, 0.0), p(-8.0, 14.0)],
                color,
                egui::Stroke::NONE,
            ));
        }
        Icon::Pause => {
            for bar in pause_bar_rects(rect) {
                painter.rect_filled(bar, 2, color);
            }
        }
        Icon::Stop => {
            painter.rect_filled(
                egui::Rect::from_center_size(c, egui::vec2(24.0, 24.0)),
                3,
                color,
            );
        }
        Icon::Repeat => {
            for sign in [-1.0_f32, 1.0] {
                painter.add(egui::Shape::line(
                    vec![
                        p(-16.0 * sign, 3.0 * sign),
                        p(-16.0 * sign, -10.0 * sign),
                        p(16.0 * sign, -10.0 * sign),
                    ],
                    stroke,
                ));
                painter.add(egui::Shape::line(
                    vec![
                        p(9.0 * sign, -17.0 * sign),
                        p(16.0 * sign, -10.0 * sign),
                        p(9.0 * sign, -3.0 * sign),
                    ],
                    stroke,
                ));
            }
        }
        Icon::Back | Icon::Forward => {
            let sign = if matches!(kind, Icon::Back) {
                1.0
            } else {
                -1.0
            };
            let points: Vec<_> = (0..=28)
                .map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2
                        + i as f32 * std::f32::consts::TAU * 0.8 / 28.0;
                    p(sign * a.cos() * 19.0, a.sin() * 19.0)
                })
                .collect();
            painter.add(egui::Shape::line(points, stroke));
            painter.add(egui::Shape::line(
                vec![p(7.0 * sign, -25.0), p(0.0, -19.0), p(7.0 * sign, -12.0)],
                stroke,
            ));
            let galley = ui.fonts(|f| {
                f.layout_no_wrap("5".into(), super::home_art::home_font(18.0 * scale), color)
            });
            let origin = c - galley.mesh_bounds.center().to_vec2();
            painter.galley(origin, galley, color);
        }
    }
}

fn round_button(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    kind: Icon,
    selected: bool,
    enabled: bool,
) -> bool {
    ui.push_id(key, |ui| {
        let width = if matches!(kind, Icon::Repeat) {
            70.0
        } else {
            58.0
        };
        ui.allocate_ui_with_layout(
            egui::vec2(width, 64.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                // A real button owns interaction, focus and accessibility; only its icon is custom.
                let response = ui.add_enabled(
                    enabled,
                    egui::Button::new("")
                        .min_size(egui::vec2(42.0, 42.0))
                        .corner_radius(21)
                        .selected(selected)
                        .fill(if selected { ux::ACCENT } else { Color32::WHITE })
                        .stroke(egui::Stroke::new(
                            1.5_f32,
                            if selected { ux::ACCENT } else { ux::BORDER },
                        )),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::WidgetType::Button, enabled, selected, label)
                });
                remember(ui, key, &response);
                icon(
                    ui,
                    response.rect,
                    kind,
                    if !enabled {
                        ux::MUTED
                    } else if selected {
                        Color32::WHITE
                    } else {
                        ux::INK
                    },
                );
                let clicked = response.on_hover_text(label).clicked();
                ui.add(egui::Label::new(RichText::new(label).size(12.0)).truncate());
                clicked
            },
        )
        .inner
    })
    .inner
}

fn volume_control(ui: &mut egui::Ui, snapshot: &media::PlaybackSnapshot, actions: &mut Actions) {
    ui.allocate_ui_with_layout(
        egui::vec2(170.0, 64.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(170.0);
            ui.label(RichText::new("音量").size(13.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().slider_width = 112.0;
                let mut volume = snapshot.volume;
                let response = ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).show_value(false));
                remember(ui, "volume", &response);
                if response
                    .on_hover_text("読み上げの音量（Windows全体の音量は変更しない）")
                    .changed()
                {
                    actions.volume = Some(volume);
                }
                ui.label(RichText::new(format!("{:.0}%", volume * 100.0)).size(13.0));
            });
        },
    );
}

fn rate_control(
    ui: &mut egui::Ui,
    snapshot: &media::PlaybackSnapshot,
    requested_rate: f64,
    can_configure: bool,
    actions: &mut Actions,
) {
    ui.allocate_ui_with_layout(
        egui::vec2(92.0, 64.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_width(92.0);
            ui.label(RichText::new("再生速度").size(13.0));
            ui.add_enabled_ui(can_configure, |ui| {
                let mut rate = if snapshot.loaded && !snapshot.loading {
                    snapshot.rate
                } else {
                    requested_rate
                };
                let response = egui::ComboBox::from_id_salt("speech-rate")
                    .width(76.0)
                    .height(260.0)
                    .selected_text(format!("{rate:.2}×"))
                    .show_ui(ui, |ui| {
                        for value in [
                            0.5, 0.6, 0.7, 0.75, 0.8, 0.9, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0,
                            3.5, 4.0,
                        ] {
                            if ui
                                .ww_selectable_value(&mut rate, value, format!("{value:.2}×"))
                                .changed()
                            {
                                actions.rate = Some(rate);
                            }
                        }
                    })
                    .response;
                remember(ui, "rate", &response);
            });
        },
    );
}

fn close_button(ui: &mut egui::Ui, actions: &mut Actions) {
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size = egui::vec2(32.0, 32.0);
            ui.spacing_mut().button_padding = egui::vec2(6.0, 2.0);
            ui.add(crate::app::controls::Button::new("×").min_size(egui::vec2(32.0, 32.0)))
        })
        .inner;
    remember(ui, "close", &response);
    actions.close = response.on_hover_text("音声を停止して閉じる").clicked();
}

fn transport_controls(
    ui: &mut egui::Ui,
    snapshot: &media::PlaybackSnapshot,
    selected: Option<SpeechButton>,
    actions: &mut Actions,
) {
    actions.back = round_button(
        ui,
        "back",
        "5秒戻る",
        Icon::Back,
        selected == Some(SpeechButton::Back),
        snapshot.can_seek && !snapshot.loading,
    );
    actions.toggle = round_button(
        ui,
        "toggle",
        if snapshot.playing {
            "一時停止"
        } else {
            "再生"
        },
        if snapshot.playing {
            Icon::Pause
        } else {
            Icon::Play
        },
        selected == Some(SpeechButton::Toggle),
        !snapshot.loading,
    );
    actions.stop = round_button(ui, "stop", "停止", Icon::Stop, false, true);
    actions.forward = round_button(
        ui,
        "forward",
        "5秒進む",
        Icon::Forward,
        selected == Some(SpeechButton::Forward),
        snapshot.can_seek && !snapshot.loading,
    );
}

fn time_label(seconds: f64) -> String {
    let total = if seconds.is_finite() {
        seconds.max(0.0).floor() as u64
    } else {
        0
    };
    if total >= 3600 {
        format!("{}:{:02}:{:02}", total / 3600, total / 60 % 60, total % 60)
    } else {
        format!("{}:{:02}", total / 60, total % 60)
    }
}

pub(super) fn show(
    ctx: &egui::Context,
    snapshot: &media::PlaybackSnapshot,
    selected: Option<SpeechButton>,
    requested_rate: f64,
    can_configure: bool,
) -> Actions {
    let mut actions = Actions::default();
    let _panel = egui::TopBottomPanel::bottom("speech-controls")
        .frame(
            egui::Frame::new()
                .fill(ux::TINT)
                .inner_margin(8)
                .corner_radius(14)
                .stroke(egui::Stroke::new(1.0_f32, ux::BORDER)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(super::home_art::home_font(16.0));
            ui.style_mut().visuals.override_text_color = Some(ux::INK);
            ui.style_mut().visuals.selection.bg_fill = ux::ACCENT;
            ui.spacing_mut().interact_size.y = 36.0;
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
            let width = ui.available_width();
            if width >= 760.0 {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(72.0, 64.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.add_space(19.0);
                            ui.strong("読み上げ");
                        },
                    );
                    transport_controls(ui, snapshot, selected, &mut actions);
                    volume_control(ui, snapshot, &mut actions);
                    rate_control(ui, snapshot, requested_rate, can_configure, &mut actions);
                    if round_button(
                        ui,
                        "repeat",
                        if snapshot.repeat {
                            "リピート ON"
                        } else {
                            "リピート"
                        },
                        Icon::Repeat,
                        snapshot.repeat,
                        true,
                    ) {
                        actions.repeat = Some(!snapshot.repeat);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close_button(ui, &mut actions)
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.strong("読み上げ");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close_button(ui, &mut actions)
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    transport_controls(ui, snapshot, selected, &mut actions);
                    volume_control(ui, snapshot, &mut actions);
                    rate_control(ui, snapshot, requested_rate, can_configure, &mut actions);
                    if round_button(
                        ui,
                        "repeat",
                        if snapshot.repeat {
                            "リピート ON"
                        } else {
                            "リピート"
                        },
                        Icon::Repeat,
                        snapshot.repeat,
                        true,
                    ) {
                        actions.repeat = Some(!snapshot.repeat);
                    }
                });
            }
            ui.horizontal(|ui| {
                let time_width = if width >= 520.0 { 62.0 } else { 48.0 };
                ui.add_sized(
                    [time_width, 36.0],
                    egui::Label::new(time_label(snapshot.position_seconds)),
                );
                ui.spacing_mut().slider_width = (ui.available_width() - time_width - 6.0).max(24.0);
                let mut position = snapshot.position_seconds;
                let response = ui
                    .add_enabled(
                        snapshot.can_seek && !snapshot.loading && snapshot.duration_seconds > 0.0,
                        egui::Slider::new(&mut position, 0.0..=snapshot.duration_seconds.max(0.1))
                            .show_value(false),
                    )
                    .on_hover_text("再生位置");
                remember(ui, "seek", &response);
                if response.changed() {
                    actions.seek = Some(position);
                }
                ui.add_sized(
                    [time_width, 36.0],
                    egui::Label::new(time_label(snapshot.duration_seconds)),
                );
            });
        });
    #[cfg(test)]
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new(("speech-control", "panel")),
            _panel.response.rect,
        )
    });
    actions
}

#[cfg(test)]
mod tests {
    #[test]
    fn playback_time_is_bounded_and_readable() {
        assert_eq!(super::time_label(42.9), "0:42");
        assert_eq!(super::time_label(204.0), "3:24");
        assert_eq!(super::time_label(3601.0), "1:00:01");
        assert_eq!(super::time_label(f64::NAN), "0:00");
        assert_eq!(super::time_label(-1.0), "0:00");
    }

    #[test]
    fn pause_visible_ink_is_centered_and_scales_with_button_diameter() {
        let small = eframe::egui::Rect::from_center_size(
            eframe::egui::pos2(80.0, 50.0),
            eframe::egui::vec2(42.0, 42.0),
        );
        let full = eframe::egui::Rect::from_center_size(
            eframe::egui::pos2(80.0, 50.0),
            eframe::egui::vec2(64.0, 64.0),
        );
        let small_bars = super::pause_bar_rects(small);
        let full_bars = super::pause_bar_rects(full);
        let small_ink = small_bars[0].union(small_bars[1]);
        let full_ink = full_bars[0].union(full_bars[1]);
        assert!((small_ink.center().x - small.center().x).abs() < f32::EPSILON);
        assert!((small_ink.center().y - small.center().y).abs() < f32::EPSILON);
        assert!((small_ink.height() / full_ink.height() - 42.0 / 64.0).abs() < 0.001);
        assert!((small_ink.width() / full_ink.width() - 42.0 / 64.0).abs() < 0.001);
    }
}
