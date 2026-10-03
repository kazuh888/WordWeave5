#![cfg(all(windows, feature = "gui"))]

#[path = "support/provider.rs"]
mod fixture;
mod support;

use eframe::egui::{self, epaint::Shape, pos2, vec2, Rect};
use qwen_audio::gui::{render, GuiModel, GuiPhase};

fn model(phase: GuiPhase, long: bool) -> GuiModel {
    GuiModel {
        reference_text: if long {
            "The synthetic sentence is deliberately long.\n".repeat(200)
        } else {
            fixture::REFERENCE.to_owned()
        },
        file_label: if long {
            format!("{}.wav", "合成音声の長い名前".repeat(30))
        } else {
            "合成音声.wav".to_owned()
        },
        audio_info: Some(
            qwen_audio::AudioInput::parse(support::wav(1, 8_000, 8))
                .unwrap()
                .info()
                .clone(),
        ),
        audio_format: Some("wav"),
        host_label: fixture::BASE.to_owned(),
        credential_present: true,
        mcp_origin: true,
        phase,
        notice: None,
    }
}

struct Geometry {
    text: Vec<(String, Rect, Rect)>,
    rects: Vec<Rect>,
}
fn collect(shape: &Shape, clip: Rect, geometry: &mut Geometry) {
    match shape {
        Shape::Vec(shapes) => {
            for shape in shapes {
                collect(shape, clip, geometry);
            }
        }
        Shape::Text(text) => {
            let mesh = text.galley.mesh_bounds.translate(text.pos.to_vec2());
            geometry
                .text
                .push((text.galley.job.text.clone(), mesh, clip));
        }
        Shape::Rect(shape) => geometry.rects.push(shape.rect),
        _ => {}
    }
}

fn draw(width: f32, zoom: f32, model: &mut GuiModel) -> Geometry {
    let context = egui::Context::default();
    context.set_zoom_factor(zoom);
    let output = context.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 900.0))),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = render(ui, model);
            });
        },
    );
    let mut geometry = Geometry {
        text: Vec::new(),
        rects: Vec::new(),
    };
    for clipped in &output.shapes {
        collect(&clipped.shape, clipped.clip_rect, &mut geometry);
    }
    geometry
}

#[test]
fn japanese_send_button_mesh_is_centered_and_accessible_at_narrow_width_and_zoom() {
    for width in [800.0, 360.0] {
        for zoom in [1.0, 1.5, 2.0] {
            let geometry = draw(width, zoom, &mut model(GuiPhase::AwaitingUser, false));
            let (_, mesh, clip) = geometry
                .text
                .iter()
                .find(|(text, _, _)| text == "評価を送信")
                .expect("visible send action label");
            assert!(mesh.is_finite());
            let button = geometry
                .rects
                .iter()
                .filter(|rect| rect.contains(mesh.center()))
                .min_by(|a, b| a.area().total_cmp(&b.area()))
                .expect("native button rectangle");
            assert!(
                (mesh.center().x - button.center().x).abs() <= 1.0,
                "width={width}, zoom={zoom}, horizontal glyph center"
            );
            assert!(
                (mesh.center().y - button.center().y).abs() <= 1.0,
                "width={width}, zoom={zoom}, vertical glyph center"
            );
            assert!(
                mesh.right() <= clip.right() + 1.0,
                "send label cannot be clipped on the right"
            );
        }
    }
}

#[test]
fn long_input_running_error_and_results_render_with_finite_geometry_without_credentials() {
    for width in [800.0, 360.0] {
        for zoom in [1.0, 1.5, 2.0] {
            for phase in [
                GuiPhase::AwaitingUser,
                GuiPhase::Running,
                GuiPhase::Failed(qwen_audio::SafeError::new(
                    qwen_audio::ErrorCode::Network,
                    qwen_audio::SendDisposition::MayHaveBeenSent,
                )),
                GuiPhase::Cancelled,
            ] {
                let geometry = draw(width, zoom, &mut model(phase, true));
                assert!(!geometry.text.is_empty());
                for (text, mesh, _) in geometry.text {
                    assert!(mesh.is_finite(), "nonfinite mesh at {width}/{zoom}");
                    assert!(!text.contains(fixture::KEY));
                }
            }
            let mut feedback: qwen_audio::Feedback =
                serde_json::from_value(fixture::feedback()).unwrap();
            feedback.summary = "日本語の長い結果表示。".repeat(150);
            let result = qwen_audio::EvaluationResult {
                feedback,
                requested_model: "qwen3.8-omni-flash".to_owned(),
                requested_effort: "medium".to_owned(),
                actual_model: None,
                actual_effort: None,
                usage: None,
            };
            let geometry = draw(width, zoom, &mut model(GuiPhase::Completed(result), true));
            assert!(!geometry.text.is_empty());
            for (_, mesh, _) in geometry.text {
                assert!(mesh.is_finite());
            }
        }
    }
}
