use super::*;

fn fixture() -> (egui::Context, WordApp, PathBuf) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "ww-ui-{}-{}-{sequence}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let storage = Storage::at(root.join("data")).unwrap();
    let mut progress = Progress::default();
    progress.chats.push(wordweave5::chat::Conversation::new());
    storage.save(&progress).unwrap();
    let ctx = egui::Context::default();
    let app = WordApp::new_with_storage(&ctx, Ok(storage));
    (ctx, app, root)
}

fn frame(ctx: &egui::Context, app: &mut WordApp, close: bool) -> egui::FullOutput {
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1120.0, 850.0),
        )),
        ..Default::default()
    };
    if close {
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .events
            .push(egui::ViewportEvent::Close);
    }
    ctx.run(input, |ctx| app.update_ui(ctx))
}

fn shape_text(shape: &egui::epaint::Shape, text: &mut String) {
    match shape {
        egui::epaint::Shape::Text(t) => {
            text.push_str(&t.galley.text());
            text.push('\n');
        }
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                shape_text(shape, text);
            }
        }
        _ => {}
    }
}

#[test]
fn fatal_close_keeps_recovery_controls_visible_without_changing_learning_data() {
    let (ctx, mut app, root) = fixture();
    app.fatal = Some("injected save failure".into());
    app.unsaved_chat_audio = Some((app.progress.chats[0].id.clone(), b"recorded audio".to_vec()));
    let before = serde_json::to_vec(&app.progress).unwrap();
    frame(&ctx, &mut app, true);
    let output = frame(&ctx, &mut app, true);
    let mut text = String::new();
    for shape in &output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(
        text.contains("WAVへ退避"),
        "Recovery controls were not drawn: {text}"
    );
    assert!(output.viewport_output[&egui::ViewportId::ROOT]
        .commands
        .iter()
        .any(|c| matches!(c, egui::ViewportCommand::CancelClose)));
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
    assert!(app.pending.is_none());
    assert!(app.recorder.is_none());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn media_export_clears_only_the_successfully_saved_buffer() {
    let (ctx, mut app, root) = fixture();
    app.fatal = Some("injected save failure".into());
    let audio = b"original audio".to_vec();
    app.unsaved_chat_audio = Some((app.progress.chats[0].id.clone(), audio.clone()));
    app.annotation.freeze_blank(&ctx).unwrap();
    app.annotation.strokes = vec![vec![[0.1, 0.1], [0.2, 0.2]]];
    let original_files = app.annotation.files().unwrap();
    let before = serde_json::to_vec(&app.progress).unwrap();
    assert!(app.export_unsaved_chat_audio(&root).is_err());
    assert_eq!(app.unsaved_chat_audio.as_ref().unwrap().1, audio);
    assert!(app.export_chat_annotation(&root).is_err());
    assert_eq!(app.annotation.files().unwrap(), original_files);
    let wav = root.join("recovered.wav");
    app.export_unsaved_chat_audio(&wav).unwrap();
    assert_eq!(std::fs::read(wav).unwrap(), audio);
    assert!(app.unsaved_chat_audio.is_none());
    assert!(!app.annotation.strokes.is_empty());
    let dest = root.join("recovered-annotation");
    app.export_chat_annotation(&dest).unwrap();
    assert_eq!(
        std::fs::read(dest.join("ink.json")).unwrap(),
        original_files.0
    );
    assert_eq!(
        std::fs::read(dest.join("background.png")).unwrap(),
        original_files.1
    );
    assert_eq!(
        std::fs::read(dest.join("composite.png")).unwrap(),
        original_files.2
    );
    assert!(dest.join("complete.json").is_file());
    assert!(!app.annotation.frozen());
    let output = frame(&ctx, &mut app, true);
    assert!(!output.viewport_output[&egui::ViewportId::ROOT]
        .commands
        .iter()
        .any(|c| matches!(c, egui::ViewportCommand::CancelClose)));
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unfrozen_annotation_text_survives_fatal_close_and_failed_export() {
    for original in ["Please review this sentence.\n日本語の注釈も残す。", " \n "] {
        let (ctx, mut app, root) = fixture();
        app.annotation.text = original.into();
        app.fatal = Some("injected save failure".into());
        let before = serde_json::to_vec(&app.progress).unwrap();
        frame(&ctx, &mut app, true);
        let output = frame(&ctx, &mut app, true);
        assert!(output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::CancelClose)));
        let mut text = String::new();
        for shape in &output.shapes {
            shape_text(&shape.shape, &mut text);
        }
        assert!(
            text.contains("原文をTXTへ退避"),
            "Recovery controls missing: {text}"
        );
        // Reach controls below the fold through the same scroll input as the UI.
        let pointer = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(t) if t.galley.text().contains("原文をTXTへ退避") => {
                    Some(t.pos + egui::vec2(5.0, 5.0))
                }
                _ => None,
            })
            .expect("visible recovery control position");
        for _ in 0..5 {
            if text.contains("未添付の注釈を破棄") {
                break;
            }
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1120.0, 850.0),
                )),
                events: vec![
                    egui::Event::PointerMoved(pointer),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -300.0),
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            };
            let scrolled = ctx.run(input, |ctx| app.update_ui(ctx));
            for shape in &scrolled.shapes {
                shape_text(&shape.shape, &mut text);
            }
        }
        assert!(
            text.contains("未添付の注釈を破棄"),
            "Discard controls unreachable after scrolling: {text}"
        );
        assert!(app.export_chat_annotation_text(&root).is_err());
        assert_eq!(app.annotation.text, original);
        let dest = root.join("annotation-original.txt");
        app.export_chat_annotation_text(&dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest).unwrap(), original);
        assert!(app.annotation.text.is_empty());
        let output = frame(&ctx, &mut app, true);
        assert!(!output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::CancelClose)));
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn frozen_background_without_strokes_blocks_close_until_exported() {
    for blank in [false, true] {
        let (ctx, mut app, root) = fixture();
        if blank {
            app.annotation.freeze_blank(&ctx).unwrap();
        } else {
            app.annotation.text = "Please review this sentence.".into();
            app.annotation.freeze(&ctx).unwrap();
        }
        assert!(app.annotation.strokes.is_empty());
        let output = frame(&ctx, &mut app, true);
        assert!(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::CancelClose)),
            "A fixed but unexported background must remain recoverable"
        );
        app.export_chat_annotation(&root.join("exported")).unwrap();
        let output = frame(&ctx, &mut app, true);
        assert!(!output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::CancelClose)));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn material_comparison_draws_without_modifying_the_proposal_or_progress() {
    use wordweave5::material::{Draft, Mode, Source};
    let (ctx, mut app, root) = fixture();
    let baseline = app.deck[0].clone();
    let mut candidate = baseline.clone();
    candidate.meaning.push_str("（補足）");
    let draft = Draft {
        mode: Mode::Correct,
        baseline: Some(baseline),
        candidate: candidate.clone(),
        source: Source {
            entry_id: candidate.id.clone(),
            conversation_id: app.progress.chats[0].id.clone(),
            exchange_indices: vec![],
            at: 0,
            mode: Mode::Correct,
            snapshots: vec![],
        },
        notices: vec![],
        reasons: vec![],
        generated: Some(candidate),
    };
    let before = serde_json::to_vec(&app.progress).unwrap();
    let proposal = serde_json::to_vec(&draft).unwrap();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.material_comparison(ui, &draft));
    });
    assert!(!output.shapes.is_empty());
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
    assert_eq!(serde_json::to_vec(&draft).unwrap(), proposal);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn json_restore_preserves_the_latest_unsaved_chat_draft() {
    let (_ctx, mut app, root) = fixture();
    let old = app.progress.clone();
    app.progress.chats[0].draft = "保存前の手直しした日本語の下書き".into();
    app.dirty = true;
    app.restore(old.clone()).unwrap();
    assert_eq!(app.progress.chats[0].draft, old.chats[0].draft);
    let snapshots = std::fs::read_dir(root.join("data/backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("progress-memory-before-restore-")
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 1);
    let saved: Progress = serde_json::from_slice(&std::fs::read(&snapshots[0]).unwrap()).unwrap();
    assert_eq!(saved.chats[0].draft, "保存前の手直しした日本語の下書き");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
