use super::harness_tests::{fixture, shape_text};
use super::*;

#[test]
fn active_study_cards_preserve_answer_and_fit_narrow_width() {
    let (ctx, mut app, root) = fixture();
    ctx.set_zoom_factor(1.0);
    app.start(5);
    app.current.as_mut().unwrap().introduce = false;
    app.answer = "unfinished answer".into();
    let before = serde_json::to_value(&app.progress).unwrap();
    for skill in Skill::ALL {
        app.current.as_mut().unwrap().skill = skill;
        for _ in 0..2 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(512.5, 3000.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let right = ui.max_rect().right();
                        app.study(ui);
                        assert!(
                            ui.min_rect().right() <= right + 1.0,
                            "active study overflow: {:?}",
                            ui.min_rect()
                        );
                    });
                },
            );
        }
        assert_eq!(app.answer, "unfinished answer");
        assert!(!app.revealed);
    }
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_start_cards_show_real_settings_without_mutating_records() {
    let (ctx, mut app, root) = fixture();
    ctx.set_zoom_factor(1.0);
    app.progress.settings.minutes = 7;
    app.deck.clear();
    let before = serde_json::to_value(&app.progress).unwrap();
    for width in [500.0, 1400.0] {
        let mut rendered = String::new();
        for _ in 0..2 {
            let output = ctx.run(
                egui::RawInput {
                    // Include the bottom empty-state card in the painted test viewport.
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 5000.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let right = ui.max_rect().right();
                        app.study_start(ui);
                        assert!(
                            ui.min_rect().right() <= right + 1.0,
                            "horizontal overflow: {:?}",
                            ui.min_rect()
                        );
                    });
                },
            );
            rendered.clear();
            for shape in output.shapes {
                shape_text(&shape.shape, &mut rendered);
            }
        }
        assert!(rendered.contains("7分の学習を始める"));
        assert!(rendered.contains("今取り組める問題はありません"));
        assert!(rendered.contains("今日は2分だけ"));
    }
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    assert!(app.session.is_none());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

fn home_frame(
    ctx: &egui::Context,
    app: &mut WordApp,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1650.0, 1500.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.home(ui));
        },
    )
}

#[test]
fn home_renders_real_counts_without_modifying_learning_records() {
    let (ctx, mut app, root) = fixture();
    app.deck.truncate(1);
    app.progress.study_seconds.insert(today(), 125);
    let before = serde_json::to_value(&app.progress).unwrap();
    home_frame(&ctx, &mut app, vec![]);
    let out = home_frame(&ctx, &mut app, vec![]);
    let mut text = String::new();
    for shape in out.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(
        text.contains("2分5秒") && text.contains("0 / 1 語"),
        "{text}"
    );
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn home_start_button_reaches_existing_session_start() {
    let (ctx, mut app, root) = fixture();
    home_frame(&ctx, &mut app, vec![]);
    let out = home_frame(&ctx, &mut app, vec![]);
    let pos = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t)
                if t.galley.text() == "▶  学習を始める"
                    && t.galley
                        .job
                        .sections
                        .iter()
                        .any(|s| s.format.color != Color32::TRANSPARENT) =>
            {
                Some(t.pos + t.galley.mesh_bounds.center().to_vec2())
            }
            _ => None,
        })
        .expect("visible start button");
    for pressed in [true, false] {
        home_frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    assert!(app.page == Page::Study);
    assert!(app.session.is_some());
    assert!(app.current.is_some());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn home_zoom_preferences_round_trip_but_launch_at_standard() {
    let (ctx, mut app, root) = fixture();
    assert_eq!(Progress::default().settings.font_scale, 0.8);
    for zoom in [0.5, 0.6, 0.8, 1.0, 1.6] {
        app.progress.settings.font_scale = zoom;
        app.storage.as_ref().unwrap().save(&app.progress).unwrap();
        let loaded = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(loaded.settings.font_scale, zoom);
    }
    let restored = WordApp::new_with_storage(&ctx, Ok(app.storage.take().unwrap()));
    assert_eq!(restored.progress.settings.font_scale, 0.8);
    for zoom in [0.49, 1.61, f32::NAN] {
        app.progress.settings.font_scale = zoom;
        assert!(app.progress.validate().is_err());
    }
    drop(restored);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn home_chrome_keeps_content_space_across_frames_at_small_size() {
    let (ctx, mut app, root) = fixture();
    ctx.set_zoom_factor(1.0);
    for _ in 0..8 {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(512.5, 406.25),
                )),
                ..Default::default()
            },
            |ctx| {
                app.update_ui(ctx);
                assert!(
                    ctx.available_rect().height() >= 180.0,
                    "{:?}",
                    ctx.available_rect()
                );
            },
        );
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
