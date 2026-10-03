// QWEN-INTEGRATION-001: independent UI boundary tests; never starts hardware.
#[path = "../../tools/qwen-audio/tests/fixtures/reuse/mod.rs"]
mod reuse;
use super::qwen_reading_ui::{recording_stop_due, QwenAction, QwenDialog};
use crate::media::CapturedRecording;
use reuse::*;
use std::time::Duration;
use wordweave5::{model::Entry, qwen_reading::ReadingPhase};

fn entry() -> Entry {
    Entry {
        id: "synthetic-qwen-entry".into(),
        base: "bright".into(),
        meaning: "明るい".into(),
        level: "1".into(),
        business: String::new(),
        elevated: String::new(),
        register: String::new(),
        usage: String::new(),
        context: String::new(),
        example: "The sun is ___.".into(),
        translation: "太陽は明るい。".into(),
        answers: vec!["bright".into()],
        question: String::new(),
        explanation: String::new(),
        tag: String::new(),
        replacements: vec![],
        examples: vec![],
    }
}
fn dialog() -> QwenDialog {
    QwenDialog::new(&entry(), Store::configured(), Wire::new(Reply::Pending)).unwrap()
}

// QWEN-STOP-001-AC02: the retained playback object is not evidence of active audio.
#[test]
fn qwen_stop_requires_actual_playback_and_disables_loading_end_and_error_states() {
    use super::qwen_reading_ui::original_audio_playing;
    use crate::media::PlaybackSnapshot;
    assert!(!original_audio_playing(None), "missing/failed snapshot");
    for loaded in [false, true] {
        for (playing, loading, expected) in [
            (false, false, false),
            (false, true, false),
            (true, false, true),
            (true, true, false),
        ] {
            let state = PlaybackSnapshot {
                loaded,
                playing,
                loading,
                ..Default::default()
            };
            assert_eq!(
                original_audio_playing(Some(&state)),
                expected,
                "loaded={loaded}, playing={playing}, loading={loading}"
            );
        }
    }
    // Stop, natural end and retry are separate observations of the same retained object.
    let mut state = PlaybackSnapshot {
        loaded: true,
        duration_seconds: 1.0,
        ..Default::default()
    };
    for (phase, playing, loading, position, expected) in [
        ("opening", false, true, 0.0, false),
        ("playing", true, false, 0.2, true),
        ("stopped", false, false, 0.0, false),
        ("retry", true, false, 0.3, true),
        ("natural end", false, false, 1.0, false),
    ] {
        state.playing = playing;
        state.loading = loading;
        state.position_seconds = position;
        assert_eq!(original_audio_playing(Some(&state)), expected, "{phase}");
    }
    assert!(!original_audio_playing(None), "error after prior playback");
}

// QWEN-STOP-001-AC01/03/05: initial and prepared audio retain a visible inert stop.
#[test]
fn qwen_stop_is_visible_before_playback_and_repeated_disabled_clicks_preserve_input() {
    use eframe::egui::{self, Event, Id, Modifiers, PointerButton, Rect};
    for prepared in [false, true] {
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
        if prepared {
            d.accept_recording(CapturedRecording {
                wav: audio(),
                warning: None,
            });
        }
        let ctx = egui::Context::default();
        let text = settle_reading_frame(&ctx, &mut d);
        assert!(text.contains("停止"), "visible label before playback");
        let raw_before = d.raw_audio.clone();
        let preview_before = d.controller.view().preview.map(|p| p.id);
        let phase_before = d.controller.view().phase;
        for _ in 0..3 {
            let rect = ctx
                .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "qwen-stop"))))
                .expect("stop is drawn even without a playback object");
            let body = ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
                .unwrap();
            assert!(body.contains_rect(rect), "stop must be visibly clickable");
            assert_eq!(
                ctx.data(
                    |data| data.get_temp::<bool>(Id::new(("speech-control-enabled", "qwen-stop")))
                ),
                Some(false)
            );
            for pressed in [true, false] {
                // Clear the observation so the next assertion cannot accept a stale control.
                ctx.data_mut(|data| {
                    data.remove::<Rect>(Id::new(("speech-control", "qwen-stop")));
                });
                reading_frame(
                    &ctx,
                    &mut d,
                    egui::vec2(1000.0, 1400.0),
                    vec![
                        Event::PointerMoved(rect.center()),
                        Event::PointerButton {
                            pos: rect.center(),
                            button: PointerButton::Primary,
                            pressed,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
                assert!(ctx
                    .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "qwen-stop"))))
                    .is_some());
            }
        }
        assert_eq!(d.raw_audio, raw_before);
        assert_eq!(d.controller.view().preview.map(|p| p.id), preview_before);
        assert_eq!(d.controller.view().phase, phase_before);
        assert!(d.controller.view().result.is_none());
        assert!(!d.settings_requested);
        assert_eq!(wire.count(), 0);
    }
}

fn qwen_stop_component_frame(
    ctx: &eframe::egui::Context,
    enabled: bool,
    events: Vec<eframe::egui::Event>,
) -> (bool, eframe::egui::FullOutput) {
    use eframe::egui::{self, Id, Rect};
    ctx.data_mut(|data| {
        data.remove::<Rect>(Id::new(("speech-control", "test-stop")));
    });
    let mut clicked = false;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let before = ui.button("before stop");
                ctx.data_mut(|data| data.insert_temp(Id::new("stop-before-id"), before.id));
                clicked = super::playback_panel::stop_button(ui, "test-stop", enabled);
                let after = ui.button("after stop");
                ctx.data_mut(|data| data.insert_temp(Id::new("stop-after-id"), after.id));
            });
        },
    );
    (clicked, output)
}

fn qwen_stop_key(key: eframe::egui::Key, pressed: bool) -> eframe::egui::Event {
    eframe::egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: eframe::egui::Modifiers::NONE,
    }
}

// QWEN-STOP-001-AC01/04: inspect the painted native button, not only a test hook.
#[test]
fn qwen_stop_shared_control_paints_a_round_button_square_and_stop_label() {
    use eframe::egui::{self, Id, Rect, Shape};
    fn painted_stop(shape: &Shape, target: Rect, round: &mut bool, square: &mut bool) {
        match shape {
            Shape::Rect(painted) => {
                if painted.rect == target {
                    *round |= painted.corner_radius.nw as f32
                        >= target.width().min(target.height()) / 2.0;
                } else if target.contains_rect(painted.rect)
                    && painted.rect.center() == target.center()
                    && painted.rect.width() > 0.0
                    && painted.rect.width() == painted.rect.height()
                    && painted.fill != egui::Color32::TRANSPARENT
                {
                    *square = true;
                }
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    painted_stop(shape, target, round, square);
                }
            }
            _ => {}
        }
    }
    for enabled in [false, true] {
        let ctx = egui::Context::default();
        let (_, output) = qwen_stop_component_frame(&ctx, enabled, vec![]);
        let rect = ctx
            .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "test-stop"))))
            .unwrap();
        let (mut round, mut square) = (false, false);
        let mut text = String::new();
        for shape in output.shapes {
            assert!(shape.clip_rect.contains_rect(rect), "stop is not clipped");
            painted_stop(&shape.shape, rect, &mut round, &mut square);
            super::harness_tests::shape_text(&shape.shape, &mut text);
        }
        assert!(
            round && square,
            "enabled={enabled}: circle and centered square"
        );
        assert!(text.contains("停止"));
    }
}

// QWEN-STOP-001-AC06: judge the actual paint against the approved prior appearance.
#[test]
fn qwen_stop_inactive_square_and_active_outline_increase_visible_contrast_at_zoom() {
    use eframe::egui::{self, Color32, Id, Rect, Shape, Stroke};
    fn luminance(color: Color32) -> f64 {
        let linear = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }
    fn contrast(first: Color32, second: Color32) -> f64 {
        let (first, second) = (luminance(first), luminance(second));
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }
    fn paint_colors(
        shape: &Shape,
        target: Rect,
        outline: &mut Option<(Rect, Stroke, Color32)>,
        square: &mut Option<(Rect, Color32)>,
    ) {
        match shape {
            Shape::Rect(painted)
                if painted.rect.center() == target.center()
                    && painted.rect.contains_rect(target) =>
            {
                // Native hover/focus visuals may expand the button's painted frame.
                *outline = Some((painted.rect, painted.stroke, painted.fill));
            }
            Shape::Rect(painted)
                if target.contains_rect(painted.rect)
                    && painted.rect.center() == target.center()
                    && painted.rect.width() > 0.0
                    && painted.rect.width() == painted.rect.height()
                    && painted.fill != Color32::TRANSPARENT =>
            {
                *square = Some((painted.rect, painted.fill));
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    paint_colors(shape, target, outline, square);
                }
            }
            _ => {}
        }
    }
    fn frame(
        ctx: &egui::Context,
        legacy: bool,
        enabled: bool,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        if !legacy {
            return qwen_stop_component_frame(ctx, enabled, events).1;
        }
        // Frozen U1 Stop rendering. The real egui Button applies its own disabled tint.
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = ui.button("before stop");
                    ui.push_id("test-stop", |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(58.0, 64.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.set_width(58.0);
                                let response = ui.add_enabled(
                                    enabled,
                                    egui::Button::new("")
                                        .min_size(egui::vec2(42.0, 42.0))
                                        .corner_radius(21)
                                        .selected(false)
                                        .fill(Color32::WHITE)
                                        .stroke(Stroke::new(1.5, Color32::from_rgb(211, 226, 239))),
                                );
                                response.widget_info(|| {
                                    egui::WidgetInfo::selected(
                                        egui::WidgetType::Button,
                                        enabled,
                                        false,
                                        "停止",
                                    )
                                });
                                ctx.data_mut(|data| {
                                    data.insert_temp(
                                        Id::new(("speech-control", "test-stop")),
                                        response.rect,
                                    );
                                    data.insert_temp(
                                        Id::new(("speech-control-id", "test-stop")),
                                        response.id,
                                    );
                                });
                                ui.painter()
                                    .with_clip_rect(response.rect.intersect(ui.clip_rect()))
                                    .rect_filled(
                                        Rect::from_center_size(
                                            response.rect.center(),
                                            egui::vec2(24.0, 24.0),
                                        ),
                                        3,
                                        if enabled {
                                            Color32::from_rgb(22, 44, 65)
                                        } else {
                                            Color32::from_rgb(77, 96, 115)
                                        },
                                    );
                                let _ = response.on_hover_text("停止");
                                ui.add(
                                    egui::Label::new(egui::RichText::new("停止").size(12.0))
                                        .truncate(),
                                );
                            },
                        );
                    });
                    let after = ui.button("after stop");
                    ctx.data_mut(|data| data.insert_temp(Id::new("stop-after-id"), after.id));
                });
            },
        )
    }
    struct PaintedStop {
        widget: Rect,
        outline: (Rect, Stroke, Color32),
        square: (Rect, Color32),
    }
    fn observe(legacy: bool, enabled: bool, zoom: f32, interaction: &str) -> PaintedStop {
        let ctx = egui::Context::default();
        // WordApp::new sets Visuals::light (app.rs); Context::default is dark.
        ctx.set_visuals(egui::Visuals::light());
        ctx.set_zoom_factor(zoom);
        frame(&ctx, legacy, enabled, vec![]);
        let mut output = frame(&ctx, legacy, enabled, vec![]);
        let target = ctx
            .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "test-stop"))))
            .unwrap();
        if interaction == "hover" {
            frame(
                &ctx,
                legacy,
                enabled,
                vec![egui::Event::PointerMoved(target.center())],
            );
            output = frame(&ctx, legacy, enabled, vec![]);
            let stop_id = ctx
                .data(|data| data.get_temp::<Id>(Id::new(("speech-control-id", "test-stop"))))
                .unwrap();
            let response = ctx
                .read_response(stop_id)
                .expect("native response for hover");
            assert!(
                response.contains_pointer(),
                "pointer is inside the uncovered real stop button"
            );
            assert_eq!(
                response.hovered(),
                enabled,
                "only an enabled stop can be hovered"
            );
        } else if interaction == "focus" {
            let target_id = ctx
                .data(|data| {
                    data.get_temp::<Id>(if enabled {
                        Id::new(("speech-control-id", "test-stop"))
                    } else {
                        Id::new("stop-after-id")
                    })
                })
                .unwrap();
            for _ in 0..8 {
                if ctx.memory(|memory| memory.focused()) == Some(target_id) {
                    break;
                }
                for pressed in [true, false] {
                    output = frame(
                        &ctx,
                        legacy,
                        enabled,
                        vec![qwen_stop_key(egui::Key::Tab, pressed)],
                    );
                }
            }
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(target_id),
                "real Tab traversal; legacy={legacy}, enabled={enabled}"
            );
        }
        let target = ctx
            .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "test-stop"))))
            .unwrap();
        let (mut outline, mut square) = (None, None);
        for shape in output.shapes {
            paint_colors(&shape.shape, target, &mut outline, &mut square);
        }
        PaintedStop {
            widget: target,
            outline: outline.expect("painted native circle"),
            square: square.expect("painted square"),
        }
    }
    for zoom in [0.8, 1.0, 1.25, 1.6] {
        for interaction in ["normal", "hover", "focus"] {
            let old_inactive = observe(true, false, zoom, interaction);
            let old_active = observe(true, true, zoom, interaction);
            let inactive = observe(false, false, zoom, interaction);
            let active = observe(false, true, zoom, interaction);
            for (before, after) in [(&old_inactive, &inactive), (&old_active, &active)] {
                assert_eq!(
                    before.widget, after.widget,
                    "zoom={zoom}, {interaction}: native geometry preserved"
                );
                assert_eq!(before.outline.0, after.outline.0, "painted circle geometry");
                assert_eq!(before.outline.2, after.outline.2, "circle fill preserved");
                assert_eq!(before.square.0, after.square.0, "square geometry preserved");
            }
            assert_eq!(
                active.square.1, old_active.square.1,
                "active square preserved"
            );
            assert_eq!(active.square.1, Color32::from_rgb(22, 44, 65));
            assert_eq!(
                inactive.outline.1, old_inactive.outline.1,
                "disabled native outline preserved including tint"
            );
            assert!(
                luminance(inactive.square.1) > luminance(old_inactive.square.1),
                "zoom={zoom}, {interaction}: inactive square must become lighter"
            );
            assert!(
                luminance(active.outline.1.color) < luminance(old_active.outline.1.color),
                "zoom={zoom}, {interaction}: active outline must become darker"
            );
            assert!(
                contrast(inactive.square.1, active.square.1)
                    > contrast(old_inactive.square.1, old_active.square.1),
                "zoom={zoom}, {interaction}: square state contrast must increase"
            );
            assert!(
                contrast(active.outline.1.color, inactive.outline.1.color)
                    > contrast(old_active.outline.1.color, old_inactive.outline.1.color),
                "zoom={zoom}, {interaction}: native outline state contrast must increase"
            );
            assert!(
                active.outline.1.width > old_active.outline.1.width,
                "zoom={zoom}, {interaction}: active outline weight must increase"
            );
        }
    }
}

// QWEN-STOP-001-AC03/04: actual Tab traversal, with no forced request_focus.
#[test]
fn qwen_stop_shared_control_accepts_pointer_and_tab_keyboard_only_when_enabled() {
    use eframe::egui::{self, Event, Id, Key, Modifiers, PointerButton, Rect};
    for enabled in [false, true] {
        let ctx = egui::Context::default();
        for _ in 0..2 {
            assert!(!qwen_stop_component_frame(&ctx, enabled, vec![]).0);
        }
        let rect = ctx
            .data(|data| data.get_temp::<Rect>(Id::new(("speech-control", "test-stop"))))
            .unwrap();
        let mut activated = false;
        for pressed in [true, false] {
            activated |= qwen_stop_component_frame(
                &ctx,
                enabled,
                vec![
                    Event::PointerMoved(rect.center()),
                    Event::PointerButton {
                        pos: rect.center(),
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            )
            .0;
        }
        assert_eq!(activated, enabled, "pointer enabled={enabled}");
    }
    for enabled in [false, true] {
        for activation_key in [Key::Enter, Key::Space] {
            let ctx = egui::Context::default();
            qwen_stop_component_frame(&ctx, enabled, vec![]);
            let stop_id = ctx
                .data(|data| data.get_temp::<Id>(Id::new(("speech-control-id", "test-stop"))))
                .unwrap();
            let after_id = ctx
                .data(|data| data.get_temp::<Id>(Id::new("stop-after-id")))
                .unwrap();
            let target = if enabled { stop_id } else { after_id };
            for _ in 0..8 {
                if ctx.memory(|memory| memory.focused()) == Some(target) {
                    break;
                }
                for pressed in [true, false] {
                    assert!(
                        !qwen_stop_component_frame(
                            &ctx,
                            enabled,
                            vec![qwen_stop_key(Key::Tab, pressed)]
                        )
                        .0
                    );
                    if !enabled {
                        assert_ne!(
                            ctx.memory(|memory| memory.focused()),
                            Some(stop_id),
                            "disabled stop must be skipped by Tab"
                        );
                    }
                }
            }
            assert_eq!(
                ctx.memory(|memory| memory.focused()),
                Some(target),
                "Tab must reach the native control"
            );
            let mut activated = false;
            for pressed in [true, false] {
                activated |= qwen_stop_component_frame(
                    &ctx,
                    enabled,
                    vec![qwen_stop_key(activation_key, pressed)],
                )
                .0;
            }
            assert_eq!(activated, enabled, "{activation_key:?}, enabled={enabled}");
            // A subsequent inactive frame still renders the same stop control.
            assert!(!qwen_stop_component_frame(&ctx, false, vec![]).0);
            assert_eq!(
                ctx.data(
                    |data| data.get_temp::<bool>(Id::new(("speech-control-enabled", "test-stop")))
                ),
                Some(false)
            );
        }
    }
}

#[test]
fn recording_stops_at_30_seconds_or_device_failure() {
    assert!(!recording_stop_due(Duration::from_millis(29_999), false));
    assert!(recording_stop_due(Duration::from_secs(30), false));
    assert!(recording_stop_due(Duration::from_secs(31), false));
    assert!(recording_stop_due(Duration::ZERO, true));
}
#[test]
fn opening_dialog_does_not_record_or_send() {
    let wire = Wire::new(Reply::Pending);
    let d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    assert!(d.raw_audio.is_none());
    assert!(d.controller.view().audio_info.is_none());
    assert_eq!(d.controller.view().phase, ReadingPhase::Input);
    assert_eq!(wire.count(), 0);
}
#[test]
fn unsupported_recording_keeps_playback_bytes_and_invalidates_old_confirmation() {
    let mut d = dialog();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    assert!(d.controller.prepare().is_ok());
    let unsupported = wav(1, 192_000, 8);
    d.accept_recording(CapturedRecording {
        wav: unsupported.clone(),
        warning: None,
    });
    assert_eq!(d.raw_audio.as_deref(), Some(unsupported.as_slice()));
    assert!(d.controller.prepare().is_err());
    assert!(d.controller.view().preview.is_none());
    assert!(d.notice.is_some());
}
#[test]
fn valid_partial_recording_preserves_audio_and_safe_warning() {
    let mut d = dialog();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: Some(KEY.into()),
    });
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
    assert!(d.controller.prepare().is_ok());
    assert!(d.notice.is_some());
    assert!(!d.notice.as_ref().unwrap().contains(KEY));
}
#[test]
fn selection_cancel_keeps_input_but_failure_invalidates() {
    let mut d = dialog();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let previous = d.controller.prepare().unwrap();
    d.select_audio(None);
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
    assert!(d.controller.view().preview.unwrap().id == previous.id);
    let missing = std::env::temp_dir().join(format!(
        "ww-qwen-nonexistent-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!missing.exists());
    d.select_audio(Some(missing));
    settle_audio(&mut d);
    assert!(d.raw_audio.is_none());
    assert!(d.controller.prepare().is_err());
    assert!(d.controller.view().preview.is_none());
    assert!(d.notice.is_some());
}

#[test]
fn target_all_fields_and_disappearance_invalidate_confirmation() {
    for supplemental in [false, true] {
        let mut d = dialog();
        d.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        d.controller.prepare().unwrap();
        let mut changed = entry();
        changed.examples.push(wordweave5::model::Example {
            english: "A synthetic extra sentence.".into(),
            japanese: "合成例文".into(),
            note: "synthetic".into(),
        });
        // Supplemental example does not change the primary reference; still changes the target version.
        assert_eq!(changed.completed(), entry().completed());
        d.check_target(if supplemental { Some(&changed) } else { None });
        assert_eq!(d.controller.view().phase, ReadingPhase::Invalidated);
        assert!(d.controller.view().preview.is_none());
        assert!(d.controller.restart().is_err());
    }
}

fn rendered_dialog_text(dialog: &mut QwenDialog) -> String {
    rendered_dialog_content(dialog, false).0
}

fn rendered_dialog_text_and_diagnostics(dialog: &mut QwenDialog) -> (String, String) {
    rendered_dialog_content(dialog, true)
}

fn rendered_dialog_content(
    dialog: &mut QwenDialog,
    scroll_to_read_status: bool,
) -> (String, String) {
    use eframe::egui::{self, pos2, vec2, Event, Modifiers, MouseWheelUnit, Rect, Shape};
    let ctx = egui::Context::default();
    let mut text = String::new();
    let mut diagnostics = String::new();
    let mut seen = std::collections::HashSet::new();
    // Initial disclosure tests inspect the settled initial frame only. State
    // readability tests additionally exercise the real scroll body: QU-AC-017
    // permits reaching content by scrolling, rather than showing it all at once.
    let frames = if scroll_to_read_status { 23 } else { 3 };
    for frame in 0..frames {
        let events = if frame >= 3 {
            let body = ctx
                .data(|data| data.get_temp::<Rect>(egui::Id::new("qwen-body")))
                .expect("real dialog scroll viewport");
            vec![
                Event::PointerMoved(body.center()),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: vec2(0.0, -120.0),
                    modifiers: Modifiers::NONE,
                },
            ]
        } else {
            vec![]
        };
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 1400.0))),
                events,
                ..Default::default()
            },
            |ctx| {
                assert!(!dialog.render(ctx));
            },
        );
        if !scroll_to_read_status {
            text.clear();
        }
        for shape in &output.shapes {
            if scroll_to_read_status && frame >= 2 {
                // Count only complete visible glyph rows. A laid-out but clipped
                // row, or a test-only state hook, is not display evidence.
                fn visible_text(shape: &Shape, clip: Rect, pieces: &mut Vec<String>) {
                    match shape {
                        Shape::Text(text) => {
                            let visible: String = text
                                .galley
                                .rows
                                .iter()
                                .filter(|row| {
                                    clip.contains_rect(row.rect.translate(text.pos.to_vec2()))
                                })
                                .flat_map(|row| row.glyphs.iter().map(|glyph| glyph.chr))
                                .collect();
                            if !visible.is_empty() {
                                pieces.push(visible);
                            }
                        }
                        Shape::Vec(shapes) => {
                            for shape in shapes {
                                visible_text(shape, clip, pieces);
                            }
                        }
                        _ => {}
                    }
                }
                let mut pieces = Vec::new();
                visible_text(
                    &shape.shape,
                    shape.clip_rect.intersect(ctx.screen_rect()),
                    &mut pieces,
                );
                for piece in pieces {
                    if seen.insert(piece.clone()) {
                        text.push_str(&piece);
                        text.push('\n');
                    }
                }
            } else if !scroll_to_read_status {
                super::harness_tests::shape_text(&shape.shape, &mut text);
            }
        }
        let (body, footer, status) = ctx.data(|data| {
            (
                data.get_temp::<Rect>(egui::Id::new("qwen-body")),
                data.get_temp::<Rect>(egui::Id::new("qwen-footer")),
                data.get_temp::<String>(egui::Id::new("qwen-disposition-text")),
            )
        });
        diagnostics.push_str(&format!("frame={frame} screen={:?} body={body:?} footer={footer:?} status_branch={status:?} shapes={} painted_chars={}\n", ctx.screen_rect(), output.shapes.len(), text.chars().count()));
    }
    (text, diagnostics)
}

#[test]
fn qwen_invalidated_before_send_renders_not_sent_and_calls_zero() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    d.controller.prepare().unwrap();
    d.check_target(None);

    let (text, diagnostics) = rendered_dialog_text_and_diagnostics(&mut d);

    assert_eq!(d.controller.view().phase, ReadingPhase::Invalidated);
    assert_eq!(
        d.controller.view().disposition,
        qwen_audio::SendDisposition::NotSent
    );
    assert!(
        text.contains("対象の教材が変更されました。"),
        "{diagnostics}\nvisible_text:\n{text}"
    );
    assert!(
        text.contains("送信していません。"),
        "{diagnostics}\npainted_text:\n{text}"
    );
    assert!(!text.contains("送信済みの可能性があります。"));
    assert!(!text.contains("遠隔処理の完了と課金は不明"));
    assert!(d.controller.view().result.is_none());
    assert!(d.controller.view().preview.is_none());
    assert_eq!(wire.count(), 0);
}

#[tokio::test]
async fn qwen_invalidated_after_send_renders_remote_and_billing_unknown_and_calls_once() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let p = d.controller.prepare().unwrap();
    d.controller
        .human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    wire.started.notified().await;
    assert_eq!(wire.count(), 1);
    let mut changed = entry();
    changed.translation = "合成の教材版変更".into();
    d.check_target(Some(&changed));

    let (text, diagnostics) = rendered_dialog_text_and_diagnostics(&mut d);

    assert_eq!(d.controller.view().phase, ReadingPhase::Invalidated);
    assert_eq!(
        d.controller.view().disposition,
        qwen_audio::SendDisposition::MayHaveBeenSent
    );
    assert!(
        text.contains("対象の教材が変更されました。"),
        "{diagnostics}\nvisible_text:\n{text}"
    );
    assert!(
        text.contains("送信済みの可能性があります。"),
        "{diagnostics}\npainted_text:\n{text}"
    );
    assert!(text.contains("遠隔処理の完了と課金は不明です。"));
    assert!(text.contains("自動再送はしません。"));
    assert!(!text.contains("送信していません。"));
    assert!(d.controller.view().result.is_none());
    assert!(d.controller.view().preview.is_none());
    for _ in 0..5 {
        tokio::task::yield_now().await;
        d.controller.poll();
    }
    assert_eq!(
        wire.count(),
        1,
        "rendering an invalidated target never retries"
    );
}

#[tokio::test]
async fn qwen_cancel_in_same_frame_prevents_send_even_if_send_action_is_first() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let p = d.controller.prepare().unwrap();
    d.dispatch_frame(
        &[QwenAction::Send(p.id), QwenAction::Cancel],
        &eframe::egui::Context::default(),
    );
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert_eq!(d.controller.view().phase, ReadingPhase::Cancelled);
    assert_eq!(
        d.controller.view().disposition,
        qwen_audio::SendDisposition::NotSent
    );
    assert_eq!(wire.count(), 0);
}

#[tokio::test]
async fn qwen_cancel_processed_before_poll_of_ready_completion() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let p = d.controller.prepare().unwrap();
    d.controller
        .human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    let _ = d.dispatch_frame(&[QwenAction::Cancel], &eframe::egui::Context::default());
    assert_eq!(d.controller.view().phase, ReadingPhase::Cancelled);
    assert!(d.controller.view().result.is_none());
    assert_eq!(wire.count(), 1);
    assert_eq!(
        d.controller.view().disposition,
        qwen_audio::SendDisposition::MayHaveBeenSent
    );
}

#[tokio::test]
async fn qwen_close_discards_without_recovery_or_delayed_success() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let p = d.controller.prepare().unwrap();
    d.controller
        .human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    assert!(d.dispatch_frame(
        &[QwenAction::Close, QwenAction::Cancel],
        &eframe::egui::Context::default()
    ));
    assert_eq!(d.controller.view().phase, ReadingPhase::Closed);
    assert!(d.controller.view().result.is_none());
    assert!(d.raw_audio.is_none());
    let fresh = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    assert!(fresh.raw_audio.is_none());
    assert!(fresh.controller.view().result.is_none());
    assert_eq!(wire.count(), 1);
}

#[test]
fn qwen_narrow_scaled_controls_reachable() {
    use eframe::egui::{self, pos2, vec2, Rect};
    for scale in [1.0, 1.5, 2.0] {
        let ctx = egui::Context::default();
        ctx.set_zoom_factor(scale);
        let mut d = dialog();
        d.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        d.controller.prepare().unwrap();
        let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(480.0, 640.0));
        // Multiple frames settle Modal geometry; these are logical points at the selected zoom.
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ctx| {
                    let _ = d.render(ctx);
                },
            );
        }
        // QU-AC-007: a confirmed input has no in-progress operation to cancel.
        assert!(ctx
            .data(|data| data.get_temp::<Rect>(egui::Id::new("qwen-cancel")))
            .is_none());
        for name in ["qwen-close"] {
            let rect = ctx
                .data(|data| data.get_temp::<Rect>(egui::Id::new(name)))
                .expect("recorded footer action");
            assert!(rect.is_finite() && rect.width() > 0.0 && rect.height() > 0.0);
            assert!(
                screen.contains_rect(rect),
                "scale={scale}, {name} is outside viewport: {rect:?}"
            );
        }
    }
}

#[test]
fn qwen_confirmation_has_only_requested_disclosures_in_requested_order() {
    let mut d = dialog();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    d.controller.prepare().unwrap();
    let text = rendered_dialog_text(&mut d);
    let note = "※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しません。";
    assert_eq!(text.matches(note).count(), 1);
    assert!(text.find("音声と例文をAlibaba Cloudへ送り").unwrap() < text.find(note).unwrap());
    assert!(text.find(note).unwrap() < text.find("読む例文").unwrap());
    for removed in [
        "送信先：",
        "要求モデル：",
        "目的：",
        "結果の受取先：",
        "上の音声・英文をAlibaba Cloudへ送る。",
    ] {
        assert!(!text.contains(removed), "{removed}");
    }
    assert!(text.contains("送信する英文："));
    assert!(text.contains("対応形式・音声の詳細"));
    assert!(!text.contains("MP3"), "format constraints start collapsed");
    assert!(!text.contains("3GPP"), "format constraints start collapsed");
    assert!(
        !text.contains("8000Hz"),
        "technical metadata starts collapsed"
    );
}

fn reading_frame(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    size: eframe::egui::Vec2,
    events: Vec<eframe::egui::Event>,
) -> String {
    // Rects describe controls painted in this frame, not controls from an older phase.
    for id in [
        "qwen-send",
        "qwen-prepare",
        "qwen-record",
        "qwen-cancel",
        "qwen-restart",
        "qwen-preparation-summary",
        "qwen-confirmation-summary",
    ] {
        ctx.data_mut(|data| data.remove::<eframe::egui::Rect>(eframe::egui::Id::new(id)));
    }
    let output = ctx.run(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                size,
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            assert!(!dialog.render(ctx));
        },
    );
    let mut text = String::new();
    for shape in output.shapes {
        super::harness_tests::shape_text(&shape.shape, &mut text);
    }
    text
}

// QU-AC-004/005/006: details expand independently of confirmation and identity.
#[test]
fn qwen_details_start_closed_and_expand_without_changing_confirmed_input() {
    use eframe::egui;
    let ctx = egui::Context::default();
    let mut d = dialog();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    let confirmed = d.controller.prepare().unwrap();
    let mut text = String::new();
    for _ in 0..3 {
        text = reading_frame(&ctx, &mut d, egui::vec2(1000.0, 1400.0), vec![]);
    }
    for visible in [
        "読む例文",
        "The sun is bright.",
        "① 音声を準備する",
        "② 送信内容を確認する",
        "③ 評価結果を確認する",
        "送信する英文：",
        "確認した内容を送信して評価",
        "閉じる",
    ] {
        assert!(
            text.contains(visible),
            "initial required disclosure: {visible}"
        );
    }
    assert!(!text.contains("MP3"));
    assert!(!text.contains("状態と次の操作"));
    assert!(text.rfind("① 音声を準備する").unwrap() < text.find("読む例文").unwrap());
    assert!(text.find("読む例文").unwrap() < text.rfind("② 送信内容を確認する").unwrap());
    assert!(text.rfind("② 送信内容を確認する").unwrap() < text.find("送信する英文：").unwrap());
    let reached = reading_text_by_scrolling(&ctx, &mut d, egui::vec2(1000.0, 1400.0));
    for heading in [
        "① 音声を準備する",
        "② 送信内容を確認する",
        "③ 評価結果を確認する",
    ] {
        assert!(
            reached.contains(heading),
            "body heading must be scroll-reachable: {heading}"
        );
    }
    click_reading_control(&ctx, &mut d, "qwen-audio-details");
    text = reading_text_by_scrolling(&ctx, &mut d, egui::vec2(1000.0, 1400.0));
    for format in ["WAV", "MP3", "AAC", "AMR", "3GP", "3GPP"] {
        assert!(text.contains(format), "expanded format: {format}");
    }
    assert!(text.contains("Hz"));
    assert_eq!(d.controller.view().preview.unwrap().id, confirmed.id);
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
}

// QU-AC-008: advice must refer to actions available in the current phase.
#[test]
fn qwen_input_errors_and_invalidated_target_do_not_advise_missing_restart_or_audio() {
    use eframe::egui::{self, Id, Rect};
    for scenario in ["credential-read", "bad-recording", "invalidated"] {
        let store = Store::configured();
        if scenario == "credential-read" {
            store
                .fail_load
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&entry(), store, wire.clone()).unwrap();
        match scenario {
            "credential-read" => assert_eq!(
                d.controller.view().error.unwrap().code(),
                qwen_audio::ErrorCode::CredentialRead
            ),
            "bad-recording" => d.accept_recording(CapturedRecording {
                wav: wav(1, 192_000, 8),
                warning: None,
            }),
            _ => d.check_target(None),
        }
        let ctx = egui::Context::default();
        let mut text = String::new();
        for _ in 0..3 {
            text = reading_frame(&ctx, &mut d, egui::vec2(1000.0, 1400.0), vec![]);
        }
        assert!(ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-restart")))
            .is_none());
        assert!(
            !text.contains("「もう一度練習」"),
            "{scenario}: no nonexistent restart advice"
        );
        let advice: String = text
            .lines()
            .filter(|line| line.starts_with("次の操作："))
            .collect();
        if d.raw_audio.is_none() {
            assert!(!advice.contains("原音"), "{scenario}: {advice}");
        }
        if scenario == "credential-read" {
            assert!(text.contains("設定 → AI接続"));
            assert!(ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-settings")))
                .is_some());
            assert_eq!(d.controller.view().phase, ReadingPhase::Input);
        } else if scenario == "bad-recording" {
            assert!(text.contains("録音") && text.contains("選"));
            assert_eq!(d.controller.view().phase, ReadingPhase::Input);
            assert!(
                d.raw_audio.is_some(),
                "original unsupported recording remains playable"
            );
        } else {
            assert!(text.contains("閉じて教材から開き直"));
            assert!(ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-settings")))
                .is_none());
            assert_eq!(d.controller.view().phase, ReadingPhase::Invalidated);
        }
        assert_eq!(wire.count(), 0);
    }
}

// QU-AC-007/008: cancel belongs exclusively to work in progress.
#[test]
fn qwen_cancel_visibility_and_primary_action_follow_each_phase() {
    use eframe::egui::{self, Id, Rect};
    let ctx = egui::Context::default();
    for (state, cancel, primary) in [
        ("input", false, "qwen-record"),
        ("confirm", false, "qwen-send"),
        ("running", true, "qwen-cancel"),
        ("result", false, "qwen-restart"),
        ("mismatch", false, "qwen-restart"),
        ("unassessable", false, "qwen-restart"),
        ("failed", false, "qwen-restart"),
    ] {
        for id in [
            "qwen-cancel",
            "qwen-record",
            "qwen-prepare",
            "qwen-send",
            "qwen-restart",
        ] {
            ctx.data_mut(|data| data.remove::<Rect>(Id::new(id)));
        }
        let mut d = super::qwen_reading_ui::synthetic_dialog(&entry(), state, false);
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            if matches!(state, "result" | "mismatch" | "unassessable" | "failed")
                && !matches!(
                    d.controller.view().phase,
                    ReadingPhase::Completed | ReadingPhase::Failed
                )
            {
                d.controller.poll();
                assert!(
                    std::time::Instant::now() < deadline,
                    "synthetic {state} completion"
                );
                std::thread::yield_now();
                continue;
            }
            break;
        }
        for _ in 0..3 {
            reading_frame(&ctx, &mut d, egui::vec2(1000.0, 1400.0), vec![]);
        }
        assert_eq!(
            ctx.data(|data| data.get_temp::<Rect>(Id::new("qwen-cancel")))
                .is_some(),
            cancel,
            "{state}"
        );
        assert!(
            ctx.data(|data| data.get_temp::<Rect>(Id::new(primary)))
                .is_some(),
            "{state}: {primary}"
        );
        assert!(ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-close")))
            .is_some());
        assert!(
            ctx.data(|data| data.get_temp::<Rect>(Id::new("qwen-prepare")))
                .is_none(),
            "{state}: obsolete preparation button must be absent"
        );
    }
}

// QU-AC-017: the fixed close action stays visible; scroll makes send reachable.
#[test]
fn qwen_820_by_650_scaled_send_is_reachable_and_close_remains_visible() {
    use eframe::egui::{self, Event, Id, Modifiers, MouseWheelUnit, Rect};
    for scale in [0.8, 1.0, 1.25, 1.6] {
        let ctx = egui::Context::default();
        ctx.set_zoom_factor(scale);
        let mut d = dialog();
        d.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        d.controller.prepare().unwrap();
        let size = egui::vec2(820.0, 650.0);
        for _ in 0..3 {
            reading_frame(&ctx, &mut d, size, vec![]);
        }
        let mut reached = false;
        for _ in 0..20 {
            let body = ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
                .unwrap();
            let send = ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
                .unwrap();
            let close = ctx
                .data(|data| data.get_temp::<Rect>(Id::new("qwen-close")))
                .unwrap();
            assert!(
                ctx.screen_rect().contains_rect(close),
                "scale {scale}: close={close:?}"
            );
            if body.contains_rect(send) {
                reached = true;
                break;
            }
            reading_frame(
                &ctx,
                &mut d,
                size,
                vec![
                    Event::PointerMoved(body.center()),
                    Event::MouseWheel {
                        unit: MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -160.0),
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(reached, "scale {scale}: send must be scroll-reachable");
    }
}

#[test]
fn qwen_modal_blocks_parent_zoom_shortcut_and_preserves_learning_store() {
    use eframe::egui::{self, Event, Key, Modifiers};
    let (ctx, mut app, root) = super::harness_tests::fixture();
    // WordApp queues its initial 80% zoom; egui applies it at the next frame boundary.
    // Measure the initialized screen, not the default Context's pre-frame 100% value.
    super::harness_tests::frame(&ctx, &mut app, false);
    let old_progress = serde_json::to_value(&app.progress).unwrap();
    let old_disk = serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap();
    let zoom = ctx.zoom_factor();
    app.qwen_dialog = Some(dialog());
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1120.0, 850.0),
            )),
            events: vec![Event::Key {
                key: Key::Plus,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::CTRL,
            }],
            modifiers: Modifiers::CTRL,
            ..Default::default()
        },
        |ctx| app.update_ui(ctx),
    );
    // A wrongly accepted shortcut can also queue a zoom for the following frame.
    super::harness_tests::frame(&ctx, &mut app, false);
    assert_eq!(ctx.zoom_factor(), zoom);
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), old_progress);
    assert_eq!(
        serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap(),
        old_disk
    );
    drop(app);
    // This root is owned by harness_tests::fixture(), never a user learning directory.
    std::fs::remove_dir_all(root).unwrap();
}

fn settle_audio(dialog: &mut QwenDialog) {
    let ctx = eframe::egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while dialog.audio_loading() {
        assert!(
            std::time::Instant::now() < deadline,
            "audio load must terminate"
        );
        dialog.dispatch_frame(&[], &ctx);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn settle_reading_frame(ctx: &eframe::egui::Context, dialog: &mut QwenDialog) -> String {
    let mut text = String::new();
    for _ in 0..3 {
        text = reading_frame(ctx, dialog, eframe::egui::vec2(1000.0, 1400.0), vec![]);
    }
    text
}

fn click_reading_control(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    id: &'static str,
) -> String {
    click_reading_control_at_size(ctx, dialog, id, eframe::egui::vec2(1000.0, 1400.0))
}

fn click_reading_control_at_size(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    id: &'static str,
    size: eframe::egui::Vec2,
) -> String {
    use eframe::egui::{Event, Id, Modifiers, PointerButton, Rect};
    let rect = scroll_reading_control_into_view(ctx, dialog, id, size);
    let body = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
        .unwrap();
    assert!(
        body.contains_rect(rect),
        "{id} must be visibly clickable: {rect:?}, body={body:?}"
    );
    for pressed in [true, false] {
        reading_frame(
            ctx,
            dialog,
            size,
            vec![
                Event::PointerMoved(rect.center()),
                Event::PointerButton {
                    pos: rect.center(),
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    let mut text = String::new();
    for _ in 0..3 {
        text = reading_frame(ctx, dialog, size, vec![]);
    }
    text
}

fn scroll_reading_control_into_view(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    id: &'static str,
    size: eframe::egui::Vec2,
) -> eframe::egui::Rect {
    use eframe::egui::{self, Event, Id, Modifiers, MouseWheelUnit, Rect};
    for _ in 0..60 {
        let rect = ctx
            .data(|data| data.get_temp::<Rect>(Id::new(id)))
            .unwrap_or_else(|| panic!("rendered control {id}"));
        let body = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
            .unwrap();
        if body.contains_rect(rect) && ctx.screen_rect().contains_rect(rect) {
            return rect;
        }
        let delta = if rect.bottom() > body.bottom() {
            -80.0
        } else {
            80.0
        };
        reading_frame(
            ctx,
            dialog,
            size,
            vec![
                Event::PointerMoved(body.center()),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, delta),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        for _ in 0..3 {
            reading_frame(ctx, dialog, size, vec![]);
        }
    }
    panic!("real scrolling did not make {id} completely visible");
}

// Collect only painted complete rows while actually visiting the body's top and bottom.
fn reading_text_by_scrolling(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    size: eframe::egui::Vec2,
) -> String {
    use eframe::egui::{self, Event, Id, Modifiers, MouseWheelUnit, Rect, Shape};
    fn visible_piece(shape: &Shape, clip: Rect, pieces: &mut Vec<String>) {
        match shape {
            Shape::Text(painted) => {
                let text: String = painted
                    .galley
                    .rows
                    .iter()
                    .filter(|row| clip.contains_rect(row.rect.translate(painted.pos.to_vec2())))
                    .flat_map(|row| row.glyphs.iter().map(|glyph| glyph.chr))
                    .collect();
                if !text.is_empty() {
                    pieces.push(text);
                }
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    visible_piece(shape, clip, pieces);
                }
            }
            _ => {}
        }
    }
    let body = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
        .unwrap();
    reading_frame(
        ctx,
        dialog,
        size,
        vec![
            Event::PointerMoved(body.center()),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 10_000.0),
                modifiers: Modifiers::NONE,
            },
        ],
    );
    for _ in 0..3 {
        reading_frame(ctx, dialog, size, vec![]);
    }
    let mut text = String::new();
    let mut seen = std::collections::HashSet::new();
    for step in 0..30 {
        let body = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
            .unwrap();
        let events = if step == 0 {
            vec![]
        } else {
            vec![
                Event::PointerMoved(body.center()),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -100.0),
                    modifiers: Modifiers::NONE,
                },
            ]
        };
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| {
                assert!(!dialog.render(ctx));
            },
        );
        for shape in output.shapes {
            let mut pieces = Vec::new();
            visible_piece(
                &shape.shape,
                shape.clip_rect.intersect(ctx.screen_rect()),
                &mut pieces,
            );
            for piece in pieces {
                if seen.insert(piece.clone()) {
                    text.push_str(&piece);
                    text.push('\n');
                }
            }
        }
    }
    text
}

// Hold only the external response boundary so Running can be inspected deterministically.
struct HeldReadingResponse {
    wire: std::sync::Arc<Wire>,
    release: tokio::sync::Notify,
}
impl qwen_audio::Transport for HeldReadingResponse {
    fn send<'a>(&'a self, request: qwen_audio::ProviderRequest) -> qwen_audio::TransportFuture<'a> {
        let response = qwen_audio::Transport::send(&*self.wire, request);
        Box::pin(async move {
            self.release.notified().await;
            response.await
        })
    }
}

fn assert_reading_summaries_closed(text: &str) {
    assert!(
        !text.contains("読む例文（この教材の主な例文）"),
        "stage 1 starts folded: {text}"
    );
    assert!(
        !text.contains("送信する英文："),
        "stage 2 starts folded: {text}"
    );
}

fn focus_reading_control_by_tab(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    widget_hook: &'static str,
) {
    use eframe::egui::{self, Event, Id, Key, Modifiers};
    let target = ctx
        .data(|data| data.get_temp::<Id>(Id::new(widget_hook)))
        .expect("real widget ID");
    for _ in 0..80 {
        if ctx.memory(|memory| memory.focused()) == Some(target) {
            return;
        }
        for pressed in [true, false] {
            reading_frame(
                ctx,
                dialog,
                egui::vec2(1000.0, 1400.0),
                vec![Event::Key {
                    key: Key::Tab,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
    }
    panic!("Tab did not reach {widget_hook}; no request_focus was used");
}

fn activate_focused_reading_control(
    ctx: &eframe::egui::Context,
    dialog: &mut QwenDialog,
    key: eframe::egui::Key,
) {
    use eframe::egui::{self, Event, Modifiers};
    for pressed in [true, false] {
        reading_frame(
            ctx,
            dialog,
            egui::vec2(1000.0, 1400.0),
            vec![Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
    }
}

// Q3-AC-013/008: real Tab navigation reaches send, summaries, and exit without forced focus.
#[tokio::test]
async fn qwen_keyboard_tab_reaches_send_summary_and_close_and_preserves_audio() {
    use eframe::egui::{self, Event, Id, Key, Modifiers, Rect};
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    focus_reading_control_by_tab(&ctx, &mut d, "qwen-send-widget-id");
    let body = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
        .unwrap();
    let send = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
        .unwrap();
    assert!(
        body.contains_rect(send),
        "Tab reached a visible send button"
    );
    assert_eq!(wire.count(), 0, "focus is not send consent");
    activate_focused_reading_control(&ctx, &mut d, Key::Enter);
    tokio::time::timeout(Duration::from_secs(3), wire.started.notified())
        .await
        .unwrap();
    assert_eq!(d.controller.view().phase, ReadingPhase::Running);
    assert_eq!(wire.count(), 1);
    settle_reading_frame(&ctx, &mut d);
    focus_reading_control_by_tab(&ctx, &mut d, "qwen-confirmation-summary-widget-id");
    activate_focused_reading_control(&ctx, &mut d, Key::Space);
    let expanded = settle_reading_frame(&ctx, &mut d);
    assert!(expanded.contains("送信する英文：The sun is bright."));
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
    assert_eq!(wire.count(), 1, "keyboard expansion never sends again");
    focus_reading_control_by_tab(&ctx, &mut d, "qwen-close-widget-id");
    let mut closed = false;
    for pressed in [true, false] {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 1400.0),
                )),
                events: vec![Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ctx| {
                closed = d.render(ctx);
            },
        );
        if closed {
            break;
        }
    }
    assert!(closed, "focused close activates from the keyboard");
    assert_eq!(d.controller.view().phase, ReadingPhase::Closed);
    assert!(d.raw_audio.is_none());
    assert!(d.sent_preview_for_test().is_none());
    assert_eq!(wire.count(), 1);
}

async fn settle_evaluation(dialog: &mut QwenDialog) {
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        dialog.controller.poll();
        if matches!(
            dialog.controller.view().phase,
            ReadingPhase::Completed | ReadingPhase::Failed
        ) {
            // restart also requires the finished worker's cleanup to complete.
            tokio::task::yield_now().await;
            dialog.controller.poll();
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "synthetic evaluation must terminate"
        );
        tokio::task::yield_now().await;
    }
}

// Q3-AC-008/009: preparation is local, stable across frames, and does not authorize a send.
#[test]
fn qwen_auto_confirmation_repeated_render_preserves_snapshot_and_sends_zero() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    assert_eq!(d.controller.view().phase, ReadingPhase::Input);
    assert!(d.controller.view().preview.is_none());
    let text = settle_reading_frame(&ctx, &mut d);
    let first = d
        .controller
        .view()
        .preview
        .expect("valid input automatically displays confirmation");
    assert_eq!(first.reference, REFERENCE);
    assert_eq!(first.audio_label, "練習の録音.wav");
    assert!(text.contains("送信する英文：The sun is bright."));
    for _ in 0..12 {
        reading_frame(&ctx, &mut d, eframe::egui::vec2(1000.0, 1400.0), vec![]);
        assert_eq!(d.controller.view().preview.unwrap().id, first.id);
        assert_eq!(wire.count(), 0);
    }
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
}

// Q3-AC-008: actual pointer activation sends exactly the displayed audio/reference.
#[tokio::test]
async fn qwen_auto_confirmation_requires_visible_send_click_and_sends_once() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let confirmed = d.controller.view().preview.unwrap();
    assert_eq!(wire.count(), 0);
    click_reading_control(&ctx, &mut d, "qwen-send");
    tokio::time::timeout(Duration::from_secs(3), wire.started.notified())
        .await
        .unwrap();
    assert_eq!(d.controller.view().phase, ReadingPhase::Running);
    assert!(d.controller.view().preview.is_none());
    assert_eq!(wire.count(), 1);
    wire.assert_request(0, HOST, &audio());
    // A replay of the same UI action cannot start a second request while running.
    d.dispatch_frame(&[QwenAction::Send(confirmed.id)], &ctx);
    settle_reading_frame(&ctx, &mut d);
    assert_eq!(wire.count(), 1);
    d.dispatch_frame(&[QwenAction::Cancel], &ctx);
}

// Q3-AC-009: replacing sound invalidates the old ID before a new confirmation is drawn.
#[tokio::test]
async fn qwen_auto_confirmation_audio_replacement_rejects_old_id_and_sends_new_bytes() {
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let old = d.controller.view().preview.unwrap();
    let replacement = wav(1, 16_000, 160);
    d.accept_recording(CapturedRecording {
        wav: replacement.clone(),
        warning: None,
    });
    assert!(d.controller.view().preview.is_none());
    assert!(d
        .controller
        .human_send(old.id, &tokio::runtime::Handle::current())
        .is_err());
    settle_reading_frame(&ctx, &mut d);
    let new = d.controller.view().preview.unwrap();
    assert_ne!(new.id, old.id);
    assert_eq!(new.audio_info.sample_rate, 16_000);
    assert!(d
        .controller
        .human_send(old.id, &tokio::runtime::Handle::current())
        .is_err());
    assert_eq!(wire.count(), 0);
    click_reading_control(&ctx, &mut d, "qwen-send");
    tokio::time::timeout(Duration::from_secs(3), wire.started.notified())
        .await
        .unwrap();
    assert_eq!(wire.count(), 1);
    wire.assert_request(0, HOST, &replacement);
    d.dispatch_frame(&[QwenAction::Cancel], &ctx);
}

// Q3-AC-005/008: completed summaries retain the sent content and remain read-only.
#[tokio::test]
async fn qwen_completed_summaries_expand_without_resend_or_discarding_sent_content() {
    use eframe::egui::{self, Id, Rect};
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let sent = d.controller.view().preview.unwrap();
    click_reading_control(&ctx, &mut d, "qwen-send");
    settle_evaluation(&mut d).await;
    let result = d.controller.view().result.unwrap();
    let folded = settle_reading_frame(&ctx, &mut d);
    assert!(!folded.contains("録音を開始（最大30秒）"));
    assert!(!folded.contains("送信する英文："));
    assert!(folded.contains("③ 評価結果を確認する"));
    assert!(
        folded.contains("語尾まで聞き取れる。"),
        "AI response is preserved verbatim"
    );
    for summary in ["qwen-preparation-summary", "qwen-confirmation-summary"] {
        let expanded = click_reading_control(&ctx, &mut d, summary);
        assert!(expanded.contains(&sent.reference));
        assert!(expanded.contains(&sent.audio_label));
        if summary == "qwen-confirmation-summary" {
            assert!(expanded.contains("送信する英文：The sun is bright."));
        }
        assert!(ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
            .is_none());
        assert!(d.controller.view().preview.is_none());
        assert_eq!(d.controller.view().result, Some(result.clone()));
        assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
        assert_eq!(wire.count(), 1);
        let receipt = d.sent_preview_for_test().unwrap();
        assert_eq!(receipt.id, sent.id);
        assert_eq!(receipt.reference, sent.reference);
        assert_eq!(receipt.audio_label, sent.audio_label);
        assert_eq!(receipt.audio_info, sent.audio_info);
        click_reading_control(&ctx, &mut d, summary);
    }
    wire.assert_request(0, HOST, &audio());
}

// Q3-AC-009/010: retry is a new displayed confirmation, never an automatic resend.
#[tokio::test]
async fn qwen_restart_discards_previous_result_and_requires_new_confirmation_id() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let old = d.controller.view().preview.unwrap();
    click_reading_control(&ctx, &mut d, "qwen-send");
    settle_evaluation(&mut d).await;
    d.dispatch_frame(&[QwenAction::Restart], &ctx);
    assert_eq!(d.controller.view().phase, ReadingPhase::Input);
    assert!(d.controller.view().result.is_none());
    assert!(d.controller.view().preview.is_none());
    assert!(d.sent_preview_for_test().is_none());
    assert!(d
        .controller
        .human_send(old.id, &tokio::runtime::Handle::current())
        .is_err());
    let text = settle_reading_frame(&ctx, &mut d);
    let new = d.controller.view().preview.unwrap();
    assert_ne!(new.id, old.id);
    assert!(!text.contains("語尾まで聞き取れる。"));
    assert_eq!(
        wire.count(),
        1,
        "restart and auto preparation never retry transport"
    );
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
}

// Q3-AC-005/010: expanded state belongs to one evaluation, not to the reused Context.
#[tokio::test]
async fn qwen_open_summaries_reset_on_completion_and_restart_resend_in_same_context() {
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let held = std::sync::Arc::new(HeldReadingResponse {
        wire: wire.clone(),
        release: tokio::sync::Notify::new(),
    });
    let mut d = QwenDialog::new(&entry(), Store::configured(), held.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let first = d.controller.view().preview.unwrap();
    for send_index in 0..2 {
        click_reading_control(&ctx, &mut d, "qwen-send");
        assert_eq!(d.controller.view().phase, ReadingPhase::Running);
        assert_reading_summaries_closed(&settle_reading_frame(&ctx, &mut d));
        let preparation = click_reading_control(&ctx, &mut d, "qwen-preparation-summary");
        assert!(preparation.contains("読む例文（この教材の主な例文）"));
        let confirmation = click_reading_control(&ctx, &mut d, "qwen-confirmation-summary");
        assert!(confirmation.contains("送信する英文：The sun is bright."));
        assert_eq!(d.controller.view().phase, ReadingPhase::Running);
        held.release.notify_one();
        settle_evaluation(&mut d).await;
        let completed = settle_reading_frame(&ctx, &mut d);
        assert_eq!(d.controller.view().phase, ReadingPhase::Completed);
        assert_reading_summaries_closed(&completed);
        assert!(completed.contains("語尾まで聞き取れる。"));
        assert_eq!(wire.count(), send_index + 1);
        assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
        if send_index == 0 {
            click_reading_control(&ctx, &mut d, "qwen-preparation-summary");
            let open = click_reading_control(&ctx, &mut d, "qwen-confirmation-summary");
            assert!(
                open.contains("読む例文（この教材の主な例文）") && open.contains("送信する英文：")
            );
            d.dispatch_frame(&[QwenAction::Restart], &ctx);
            assert_eq!(d.controller.view().phase, ReadingPhase::Input);
            settle_reading_frame(&ctx, &mut d);
            assert_ne!(d.controller.view().preview.unwrap().id, first.id);
            assert_eq!(wire.count(), 1, "restart never sends without another click");
        }
    }
}

// Q3-AC-005/007: a failure becomes visible even if both summaries were open while waiting.
#[tokio::test]
async fn qwen_running_open_summaries_reset_on_failure_in_same_context() {
    let wire = Wire::new(Reply::Network);
    let held = std::sync::Arc::new(HeldReadingResponse {
        wire: wire.clone(),
        release: tokio::sync::Notify::new(),
    });
    let mut d = QwenDialog::new(&entry(), Store::configured(), held.clone()).unwrap();
    let ctx = eframe::egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    click_reading_control(&ctx, &mut d, "qwen-send");
    click_reading_control(&ctx, &mut d, "qwen-preparation-summary");
    let open = click_reading_control(&ctx, &mut d, "qwen-confirmation-summary");
    assert!(open.contains("読む例文（この教材の主な例文）") && open.contains("送信する英文："));
    assert_eq!(d.controller.view().phase, ReadingPhase::Running);
    held.release.notify_one();
    settle_evaluation(&mut d).await;
    let failed = settle_reading_frame(&ctx, &mut d);
    assert_eq!(d.controller.view().phase, ReadingPhase::Failed);
    assert_reading_summaries_closed(&failed);
    assert!(failed.contains("もう一度練習"));
    assert!(failed.contains("送信済みの可能性があります。"));
    assert_eq!(wire.count(), 1);
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
}

// Q3-AC-005/013: a long real audio filename cannot widen the summary or fixed footer.
#[tokio::test]
async fn qwen_completed_long_audio_name_wraps_and_keeps_summary_and_close_reachable() {
    use eframe::egui::{self, Event, Id, Modifiers, MouseWheelUnit, Rect, Shape};
    fn assert_label_wrap(shape: &Shape, fullname: &str, body: Rect, found: &mut bool) {
        match shape {
            Shape::Text(painted) if painted.galley.text().contains(fullname) => {
                *found = true;
                assert!(
                    painted.galley.rows.len() > 1,
                    "200-character audio name must wrap"
                );
                assert!(
                    painted.galley.size().x <= body.width() + 1.0,
                    "name width exceeds the body"
                );
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    assert_label_wrap(shape, fullname, body, found);
                }
            }
            _ => {}
        }
    }
    let root = std::env::temp_dir().join(format!(
        "q3-name-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let name = format!("long_{}.wav", "x".repeat(191));
    assert_eq!(name.chars().count(), 200);
    let path = root.join(&name);
    std::fs::write(&path, audio()).unwrap();
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = egui::Context::default();
    d.select_audio(Some(path.clone()));
    settle_audio(&mut d);
    settle_reading_frame(&ctx, &mut d);
    let sent = d.controller.view().preview.unwrap();
    assert_eq!(sent.audio_label, name);
    // Sending still uses the confirmed ID; the geometry under test is the terminal screen.
    d.dispatch_frame(&[QwenAction::Send(sent.id)], &ctx);
    settle_evaluation(&mut d).await;
    ctx.set_zoom_factor(1.6);
    let size = egui::vec2(820.0, 650.0);
    let mut folded = String::new();
    for _ in 0..3 {
        folded = reading_frame(&ctx, &mut d, size, vec![]);
    }
    assert!(folded.contains("① 音声を準備する"));
    assert!(folded.contains(&format!(
        "音声：{}…",
        name.chars().take(32).collect::<String>()
    )));
    assert!(
        !folded.contains(&name),
        "full name starts in the expanded details"
    );
    let summary = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-preparation-summary")))
        .unwrap();
    let close = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-close")))
        .unwrap();
    assert!(
        ctx.screen_rect().contains_rect(summary),
        "summary is outside 820x650/160%: {summary:?}"
    );
    assert!(
        ctx.screen_rect().contains_rect(close),
        "close is outside 820x650/160%: {close:?}"
    );
    // Reach the actual summary viewport before activating it.
    for _ in 0..20 {
        let body = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
            .unwrap();
        let summary = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-preparation-summary")))
            .unwrap();
        if body.contains_rect(summary) {
            break;
        }
        reading_frame(
            &ctx,
            &mut d,
            size,
            vec![
                Event::PointerMoved(body.center()),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -80.0),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    click_reading_control_at_size(&ctx, &mut d, "qwen-preparation-summary", size);
    click_reading_control_at_size(&ctx, &mut d, "qwen-preparation-summary", size);
    // Confirmation retains the full filename; inspect its actual wrapped text shape.
    click_reading_control_at_size(&ctx, &mut d, "qwen-confirmation-summary", size);
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
            ..Default::default()
        },
        |ctx| {
            assert!(!d.render(ctx));
        },
    );
    let body = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-body")))
        .unwrap();
    let mut found = false;
    for shape in output.shapes {
        assert_label_wrap(&shape.shape, &name, body, &mut found);
    }
    assert!(found, "expanded confirmation paints the complete name");
    let close = ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-close")))
        .unwrap();
    assert!(
        ctx.screen_rect().contains_rect(close),
        "expanded full name preserves the fixed exit"
    );
    assert_eq!(d.sent_preview_for_test().unwrap().audio_label, name);
    assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
    assert_eq!(wire.count(), 1);
    assert_eq!(std::fs::read(&path).unwrap(), audio());
    drop(d);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root).unwrap();
}

// Q3-AC-001/002/003/007: reasons appear in their stage and never create a send control.
#[test]
fn qwen_three_stage_guidance_places_input_and_connection_failures_before_results() {
    use eframe::egui::{self, Id, Rect};
    for scenario in ["no-audio", "unconfigured", "bad-audio", "credential-read"] {
        let store = if scenario == "unconfigured" {
            Store::new(None)
        } else {
            Store::configured()
        };
        if scenario == "credential-read" {
            store
                .fail_load
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&entry(), store, wire.clone()).unwrap();
        if scenario != "no-audio" {
            d.accept_recording(CapturedRecording {
                wav: if scenario == "bad-audio" {
                    wav(1, 192_000, 8)
                } else {
                    audio()
                },
                warning: None,
            });
        }
        let ctx = egui::Context::default();
        settle_reading_frame(&ctx, &mut d);
        let text = reading_text_by_scrolling(&ctx, &mut d, egui::vec2(1000.0, 1400.0));
        let first = text.rfind("① 音声を準備する").unwrap();
        let second = text.rfind("② 送信内容を確認する").unwrap();
        let third = text.rfind("③ 評価結果を確認する").unwrap();
        assert!(first < second && second < third, "{scenario}: {text}");
        assert!(!text.contains("状態と次の操作"));
        if matches!(scenario, "unconfigured" | "credential-read") {
            let settings = text.find("設定 → AI接続").unwrap();
            assert!(
                second < settings && settings < third,
                "{scenario}: settings belong to stage 2"
            );
        }
        if scenario == "bad-audio" {
            assert!(
                text[first..second].contains("音声を選んでください"),
                "audio remedy belongs to stage 1: {text}"
            );
            assert_eq!(d.raw_audio.as_deref(), Some(wav(1, 192_000, 8).as_slice()));
        }
        assert!(d.controller.view().preview.is_none(), "{scenario}");
        assert!(
            ctx.data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
                .is_none(),
            "{scenario}"
        );
        assert!(
            ctx.data(|data| data.get_temp::<Rect>(Id::new("qwen-prepare")))
                .is_none(),
            "{scenario}"
        );
        assert_eq!(wire.count(), 0);
    }
}

// Q3-AC-005/007: a remote failure is a stage-3 problem, not inferred poor pronunciation.
#[tokio::test]
async fn qwen_failed_evaluation_folds_previous_steps_and_never_automatically_retries() {
    use eframe::egui::{self, Id, Rect};
    for reply in [Reply::Network, Reply::Feedback("{")] {
        let wire = Wire::new(reply);
        let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
        let ctx = egui::Context::default();
        d.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        settle_reading_frame(&ctx, &mut d);
        click_reading_control(&ctx, &mut d, "qwen-send");
        settle_evaluation(&mut d).await;
        let text = settle_reading_frame(&ctx, &mut d);
        assert_eq!(d.controller.view().phase, ReadingPhase::Failed);
        let third = text.rfind("③ 評価結果を確認する").unwrap();
        assert!(text[third..].contains("もう一度練習"));
        assert!(text[third..].contains("送信済みの可能性があります。"));
        assert!(!text.contains("送信していません。"));
        assert!(ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
            .is_none());
        for id in ["qwen-preparation-summary", "qwen-confirmation-summary"] {
            assert!(ctx
                .data(|data| data.get_temp::<Rect>(Id::new(id)))
                .is_some());
        }
        if d.controller.view().error.unwrap().code() == qwen_audio::ErrorCode::ResponseJsonInvalid {
            assert!(text.contains("発音が悪いという意味ではありません。"));
            assert!(!text.contains("例文と異なる英文が聞き取れました"));
        }
        assert!(d.controller.view().result.is_none());
        assert_eq!(d.raw_audio.as_deref(), Some(audio().as_slice()));
        assert_eq!(wire.count(), 1);
    }
}

// Q3-AC-001/013: long reference text cannot hide the fixed overview or exit action.
#[test]
fn qwen_long_reference_initial_overview_and_close_are_visible_at_supported_scales() {
    use eframe::egui::{self, Id, Rect, Shape};
    fn visible_rows(shape: &Shape, clip: Rect, text: &mut String) {
        match shape {
            Shape::Text(painted) => {
                for row in &painted.galley.rows {
                    if clip.contains_rect(row.rect.translate(painted.pos.to_vec2())) {
                        text.extend(row.glyphs.iter().map(|glyph| glyph.chr));
                    }
                }
                text.push('\n');
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    visible_rows(shape, clip, text);
                }
            }
            _ => {}
        }
    }
    let mut long_entry = entry();
    long_entry.example = "The synthetic passage remains local to this test. ".repeat(90);
    for scale in [0.8, 1.0, 1.25, 1.6] {
        let ctx = egui::Context::default();
        ctx.set_zoom_factor(scale);
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&long_entry, Store::configured(), wire.clone()).unwrap();
        let mut text = String::new();
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(820.0, 650.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    assert!(!d.render(ctx));
                },
            );
            text.clear();
            for shape in output.shapes {
                visible_rows(
                    &shape.shape,
                    shape.clip_rect.intersect(ctx.screen_rect()),
                    &mut text,
                );
            }
        }
        for overview in [
            "① 音声の準備",
            "② 送信内容の確認",
            "③ 評価結果",
            "現在：① 音声を準備する",
        ] {
            assert!(
                text.contains(overview),
                "scale={scale}: visible overview {overview}: {text}"
            );
        }
        let close = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("qwen-close")))
            .unwrap();
        assert!(
            ctx.screen_rect().contains_rect(close),
            "scale={scale}: {close:?}"
        );
        assert_eq!(wire.count(), 0);
    }
}

// Q3-AC-009: starting a replacement import prevents the old confirmed sound from being sent.
#[tokio::test]
async fn qwen_loading_and_failed_replacement_cannot_send_previous_confirmation() {
    use eframe::egui::{self, Id, Rect};
    let root = std::env::temp_dir().join(format!(
        "ww-qwen-ux3-invalid-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("synthetic-invalid.mp3");
    std::fs::write(&source, b"synthetic invalid audio, never user data").unwrap();
    let wire = Wire::new(Reply::Pending);
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    let ctx = egui::Context::default();
    d.accept_recording(CapturedRecording {
        wav: audio(),
        warning: None,
    });
    settle_reading_frame(&ctx, &mut d);
    let previous = d.controller.view().preview.unwrap();
    d.select_audio(Some(source.clone()));
    assert!(d.audio_loading());
    assert!(d.controller.view().preview.is_none());
    d.dispatch_frame(&[QwenAction::Send(previous.id)], &ctx);
    assert_eq!(wire.count(), 0, "loading is not consent to send an old ID");
    settle_audio(&mut d);
    settle_reading_frame(&ctx, &mut d);
    assert!(d.controller.view().preview.is_none());
    assert!(d.controller.view().audio_info.is_none());
    assert!(
        d.raw_audio.is_none(),
        "failed import never falls back to the earlier recording"
    );
    assert!(d
        .controller
        .human_send(previous.id, &tokio::runtime::Handle::current())
        .is_err());
    assert!(ctx
        .data(|data| data.get_temp::<Rect>(Id::new("qwen-send")))
        .is_none());
    assert_eq!(wire.count(), 0);
    assert_eq!(
        std::fs::read(&source).unwrap(),
        b"synthetic invalid audio, never user data"
    );
    drop(d);
    std::fs::remove_file(source).unwrap();
    std::fs::remove_dir(root).unwrap();
}

// Q3-AC-010/012: display-only receipt follows the same target/close lifetime as the session.
#[tokio::test]
async fn qwen_sent_display_receipt_is_discarded_on_target_change_or_close() {
    for close in [false, true] {
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
        let ctx = eframe::egui::Context::default();
        d.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        settle_reading_frame(&ctx, &mut d);
        let p = d.controller.view().preview.unwrap();
        d.dispatch_frame(&[QwenAction::Send(p.id)], &ctx);
        tokio::time::timeout(Duration::from_secs(3), wire.started.notified())
            .await
            .unwrap();
        assert_eq!(d.sent_preview_for_test().unwrap().reference, REFERENCE);
        if close {
            d.dispatch_frame(&[QwenAction::Close], &ctx);
        } else {
            d.check_target(None);
        }
        assert!(d.sent_preview_for_test().is_none());
        assert!(d.controller.view().preview.is_none());
        assert!(d.controller.view().result.is_none());
        assert!(d.raw_audio.is_none());
        assert!(d
            .controller
            .human_send(p.id, &tokio::runtime::Handle::current())
            .is_err());
        assert_eq!(wire.count(), 1);
    }
}

#[test]
fn qwen_audio_load_cannot_reappear_after_cancel_close_or_target_change() {
    let root = std::env::temp_dir().join(format!(
        "ww-audio-load-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("synthetic.mp3");
    std::fs::write(
        &path,
        include_bytes!("../../tools/qwen-audio/tests/fixtures/formats/tone.mp3"),
    )
    .unwrap();
    for terminal in 0..3 {
        let wire = Wire::new(Reply::Pending);
        let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
        d.select_audio(Some(path.clone()));
        assert!(d.audio_loading());
        let ctx = eframe::egui::Context::default();
        match terminal {
            0 => {
                d.dispatch_frame(&[QwenAction::Cancel], &ctx);
            }
            1 => {
                d.dispatch_frame(&[QwenAction::Close], &ctx);
            }
            _ => d.check_target(None),
        }
        for _ in 0..3 {
            d.dispatch_frame(&[], &ctx);
        }
        assert!(!d.audio_loading());
        assert!(d.controller.view().audio_info.is_none());
        assert!(d.raw_audio.is_none());
        assert_eq!(wire.count(), 0);
    }
    // The sole file is synthetic and owned by this test; wait for readers to release it.
    for _ in 0..100 {
        if std::fs::remove_file(&path).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    std::fs::remove_dir(&root).unwrap();
}

#[tokio::test]
async fn qwen_completed_advice_and_close_do_not_mutate_learning_store_or_source_audio() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    app.deck.push(entry());
    let old_deck = serde_json::to_value(&app.deck).unwrap();
    let old_progress = serde_json::to_value(&app.progress).unwrap();
    let old_disk = serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap();
    let source = root.join("synthetic-original.wav");
    std::fs::write(&source, audio()).unwrap();
    let wire = Wire::new(Reply::Feedback(ASSESSED));
    let mut d = QwenDialog::new(&entry(), Store::configured(), wire.clone()).unwrap();
    d.select_audio(Some(source.clone()));
    settle_audio(&mut d);
    let p = d.controller.prepare().unwrap();
    d.controller
        .human_send(p.id, &tokio::runtime::Handle::current())
        .unwrap();
    for _ in 0..5 {
        tokio::task::yield_now().await;
    }
    app.qwen_dialog = Some(d);
    super::harness_tests::frame(&ctx, &mut app, false);
    assert_eq!(
        app.qwen_dialog.as_ref().unwrap().controller.view().result,
        Some(expected(false))
    );
    app.qwen_dialog
        .as_mut()
        .unwrap()
        .dispatch_frame(&[QwenAction::Close], &ctx);
    app.qwen_dialog = None;
    assert_eq!(serde_json::to_value(&app.deck).unwrap(), old_deck);
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), old_progress);
    assert_eq!(
        serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap(),
        old_disk
    );
    assert_eq!(std::fs::read(source).unwrap(), audio());
    assert_eq!(wire.count(), 1);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
