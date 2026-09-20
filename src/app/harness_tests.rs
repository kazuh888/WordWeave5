use super::*;

#[test]
fn restoring_trash_blocks_retry_attachment_without_losing_audio() {
    let (_, mut app, root) = fixture();
    let id = app.progress.chats[0].id.clone();
    app.progress.chats[0].deleted_at = Some(123);
    app.progress.chats.push(wordweave5::chat::Conversation::new());
    app.chat_selected = 1;
    let mut wav = Vec::new();
    let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut wav), hound::WavSpec {
        channels: 1, sample_rate: 16_000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int,
    }).unwrap();
    for _ in 0..1600 { writer.write_sample(0_i16).unwrap(); }
    writer.finalize().unwrap();
    let before = serde_json::to_value(&app.progress).unwrap();
    app.save_chat_recording(&id, wav.clone());
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    assert_eq!(app.unsaved_chat_audio, Some((id, wav)));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn restoring_trash_closes_editors_and_selects_an_active_conversation() {
    let (ctx, mut app, root) = fixture();
    let mut restored = app.progress.clone();
    restored.chats[0].deleted_at = Some(123);
    restored.chats[0].memo = "keep deleted context".into();
    restored.chats.push(wordweave5::chat::Conversation::new());
    app.chat_context_open = true;
    app.chat_material_open = true;
    app.chat_media_open = true;
    app.restore(restored).unwrap();
    assert!(!app.chat_context_open && !app.chat_material_open && !app.chat_media_open);
    assert_eq!(app.chat_selected, 1);
    // Even a stale window flag must not expose a deleted conversation's editor.
    app.chat_selected = 0;
    app.chat_context_open = true;
    app.page = Page::Chat;
    frame(&ctx, &mut app, false);
    let output = frame(&ctx, &mut app, false);
    let mut text = String::new();
    for shape in output.shapes { shape_text(&shape.shape, &mut text); }
    assert!(!text.contains("引き継ぎメモ：学習目的"), "{text}");
    assert_eq!(app.progress.chats[0].memo, "keep deleted context");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn speech_rate_preserves_legacy_settings_and_rejects_invalid_changes() {
    let (_, mut app, root) = fixture();
    app.progress.settings.slow_speech = true;
    assert_eq!(app.speech_rate(), 0.8);
    app.change_speech_rate(2.5).unwrap();
    assert_eq!(app.speech_rate(), 2.5);
    assert!(app.change_speech_rate(4.1).is_err());
    assert_eq!(app.speech_rate(), 2.5);
    let original = vec![1, 2, 3];
    app.wav = Some(original.clone());
    app.cancel_recording();
    assert_eq!(app.wav, Some(original));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn effort_choices_are_only_from_the_current_cli_and_model() {
    let (_, mut app, root) = fixture();
    app.progress.settings.codex_model = "server-model".into();
    app.progress.settings.codex_effort = "server-effort".into();
    assert!(app.effort_choices().is_none());
    app.effort_catalog = Some((app.progress.settings.codex_path.clone(), vec![wordweave5::effort::ModelEffort {
        model: "server-model".into(), display_name: "Server model".into(),
        supported_efforts: Some(vec![wordweave5::effort::EffortOption { effort: "server-effort".into(), description: "From server".into() }]),
        default_effort: None,
    }]));
    assert_eq!(app.effort_choices().unwrap().supported_efforts.as_ref().unwrap()[0].effort, "server-effort");
    app.progress.settings.codex_model = "another-model".into();
    assert!(app.effort_choices().is_none());
    app.progress.settings.codex_model = "server-model".into();
    app.progress.settings.codex_path = "another-cli.exe".into();
    assert!(app.effort_choices().is_none());
    assert_eq!(app.progress.settings.codex_effort, "server-effort");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn conversation_trash_roundtrip_preserves_drafts_and_learning_data() {
    let (ctx, mut app, root) = fixture();
    let id = app.progress.chats[0].id.clone();
    app.progress.chats[0].draft = "unsent text".into();
    app.progress.chats[0].memo = "learning context".into();
    app.progress.study_seconds.insert("2026-09-19".into(), 45);
    let before = serde_json::to_value(&app.progress).unwrap();
    app.set_conversation_deleted(&id, true).unwrap();
    assert!(wordweave5::chat::ordered_indices(&app.progress.chats).is_empty());
    assert_eq!(app.progress.chats[0].draft, "unsent text");
    app.viewed_trash_id = Some(id.clone());
    app.page = Page::Chat;
    frame(&ctx, &mut app, false);
    let output = frame(&ctx, &mut app, false);
    let mut text = String::new();
    for shape in output.shapes { shape_text(&shape.shape, &mut text); }
    assert!(text.contains("読み取り専用"), "{text}");
    app.set_conversation_deleted(&id, false).unwrap();
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    assert_eq!(serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn conversation_trash_does_not_apply_when_saving_fails_or_media_is_unsaved() {
    let (_, mut app, root) = fixture();
    let id = app.progress.chats[0].id.clone();
    app.annotation.text = "keep this annotation draft".into();
    let before = serde_json::to_value(&app.progress).unwrap();
    assert!(app.set_conversation_deleted(&id, true).is_err());
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    app.annotation.text.clear();
    let path = app.storage.as_ref().unwrap().dir.join("progress.json");
    std::fs::rename(&path, path.with_extension("saved")).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(app.set_conversation_deleted(&id, true).is_err());
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

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
