use super::home_art::*;
use eframe::egui::*;
#[test]
fn status_glyphs_are_centered_and_clipped_at_each_scale() {
    for scale in [0.6, 0.8, 1.0, 1.6] {
        let (ctx, app, root) = super::harness_tests::fixture();
        ctx.set_pixels_per_point(scale);
        for label in [
            "接続テスト：未確認",
            "effort（前回）：low",
            "長い接続結果の説明文が区画を越えないことを確認する",
        ] {
            let mut rect = Rect::NOTHING;
            let output = ctx.run(RawInput::default(), |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    rect = super::chrome::status_field(ui, label, 240.0).rect;
                });
            });
            let text = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    Shape::Text(t) => Some((s.clip_rect, t)),
                    _ => None,
                })
                .expect("painted status text");
            let ink = text.1.galley.mesh_bounds.translate(text.1.pos.to_vec2());
            assert!((ink.center().y - rect.center().y).abs() <= 1.0 / scale);
            assert!(text.0.right() <= rect.right());
            assert_eq!(text.1.galley.rows.len(), 1);
            assert_eq!(text.1.galley.job.sections[0].format.font_id.size, 16.0);
            assert_eq!(
                text.1.galley.job.sections[0].format.color,
                Color32::from_rgb(92, 96, 102)
            );
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn japanese_button_ink_is_centered() {
    fn ink_bounds(shape: &Shape, label: &str, bounds: &mut Vec<Rect>) {
        match shape {
            Shape::Text(text)
                if text.galley.text().trim() == label
                    && text
                        .galley
                        .job
                        .sections
                        .iter()
                        .any(|s| s.format.color != Color32::TRANSPARENT) =>
            {
                bounds.push(text.galley.mesh_bounds.translate(text.pos.to_vec2()));
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    ink_bounds(shape, label, bounds);
                }
            }
            _ => {}
        }
    }
    for scale in [0.5, 0.6, 0.7, 0.8, 1.0, 1.25, 1.6] {
        let (ctx, app, root) = super::harness_tests::fixture();

        ctx.set_pixels_per_point(scale);
        for (label, is_primary) in [
            ("ホーム", false),
            ("英語チャット", false),
            ("▶  学習を始める", true),
        ] {
            let mut button = Rect::NOTHING;
            let output = ctx.run(RawInput::default(), |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    button = if is_primary {
                        primary(ui, label, 300.0).rect
                    } else {
                        nav_item(ui, label, Icon::Home, true).rect
                    };
                });
            });
            let mut bounds = Vec::new();
            for shape in &output.shapes {
                ink_bounds(&shape.shape, label, &mut bounds);
            }
            assert_eq!(bounds.len(), 1, "{label}");
            let delta = bounds[0].center() - button.center();
            assert!(
                delta.y.abs() <= 1.0 / scale,
                "{label} scale={scale} vertical offset={}",
                delta.y
            );
            if is_primary {
                assert!(
                    delta.x.abs() <= 1.0 / scale,
                    "{label} horizontal offset={}",
                    delta.x
                );
            }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
