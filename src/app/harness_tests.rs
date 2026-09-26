use super::*;

#[test]
fn context_base_style_keeps_dialog_text_readable() {
    let (ctx, app, root) = fixture();
    let style = ctx.style();
    for (text_style, expected_size, expected_family) in [
        (egui::TextStyle::Body, 19.0, "home_body"),
        (egui::TextStyle::Small, 16.0, "home_body"),
        (egui::TextStyle::Button, 18.0, "home_body"),
        (egui::TextStyle::Heading, 24.0, "heading"),
    ] {
        let font = &style.text_styles[&text_style];
        assert_eq!(font.size, expected_size);
        assert_eq!(font.family, egui::FontFamily::Name(expected_family.into()));
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn restoring_trash_blocks_retry_attachment_without_losing_audio() {
    let (_, mut app, root) = fixture();
    let id = app.progress.chats[0].id.clone();
    app.progress.chats[0].deleted_at = Some(123);
    app.progress
        .chats
        .push(wordweave5::chat::Conversation::new());
    app.chat_selected = 1;
    let mut wav = Vec::new();
    let mut writer = hound::WavWriter::new(
        std::io::Cursor::new(&mut wav),
        hound::WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for _ in 0..1600 {
        writer.write_sample(0_i16).unwrap();
    }
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
    app.pending_chat_rename = Some(("stale-chat".into(), "stale title".into()));
    app.restore(restored).unwrap();
    assert!(
        !app.chat_context_open
            && !app.chat_material_open
            && !app.chat_media_open
            && app.pending_chat_rename.is_none()
    );
    assert_eq!(app.chat_selected, 1);
    // Even a stale window flag must not expose a deleted conversation's editor.
    app.chat_selected = 0;
    app.chat_context_open = true;
    app.page = Page::Chat;
    frame(&ctx, &mut app, false);
    let output = frame(&ctx, &mut app, false);
    let mut text = String::new();
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
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
    app.effort_catalog = Some((
        app.progress.settings.codex_path.clone(),
        vec![wordweave5::effort::ModelEffort {
            model: "server-model".into(),
            display_name: "Server model".into(),
            supported_efforts: Some(vec![wordweave5::effort::EffortOption {
                effort: "server-effort".into(),
                description: "From server".into(),
            }]),
            default_effort: None,
        }],
    ));
    assert_eq!(
        app.effort_choices()
            .unwrap()
            .supported_efforts
            .as_ref()
            .unwrap()[0]
            .effort,
        "server-effort"
    );
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
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(text.contains("読み取り専用"), "{text}");
    app.set_conversation_deleted(&id, false).unwrap();
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
    assert_eq!(
        serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap(),
        before
    );
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

pub(super) fn fixture() -> (egui::Context, WordApp, PathBuf) {
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

pub(super) fn frame(ctx: &egui::Context, app: &mut WordApp, close: bool) -> egui::FullOutput {
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
    app.settings_zoom_input(ctx, &mut input);
    ctx.run(input, |ctx| app.update_ui(ctx))
}

pub(super) fn shape_text(shape: &egui::epaint::Shape, text: &mut String) {
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
        // Larger dialog text may place recovery actions below the fold. Verify that the
        // same scroll interaction as the UI reaches both export and discard actions.
        let pointer = ctx.screen_rect().center();
        for _ in 0..8 {
            if text.contains("原文をTXTへ退避") && text.contains("未添付の注釈を破棄")
            {
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
            text.contains("原文をTXTへ退避"),
            "Recovery export control unreachable after scrolling: {text}"
        );
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
fn closing_after_hiding_the_media_dialog_explains_why_exit_is_blocked() {
    let (ctx, mut app, root) = fixture();
    app.annotation.freeze_blank(&ctx).unwrap();
    app.annotation.strokes = vec![vec![[0.1, 0.1], [0.2, 0.2]]];
    let original = app.annotation.files().unwrap();
    let audio = b"unsaved recording".to_vec();
    app.unsaved_chat_audio = Some((app.progress.chats[0].id.clone(), audio.clone()));
    app.chat_media_open = false;
    let saved_progress = serde_json::to_value(&app.progress).unwrap();

    let output = frame(&ctx, &mut app, true);
    assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.iter()
        .any(|command| matches!(command, egui::ViewportCommand::CancelClose)));
    assert!(app.chat_media_open && app.exit_media_requested);
    let output = frame(&ctx, &mut app, false);
    let mut labels = String::new();
    for shape in &output.shapes {
        shape_text(&shape.shape, &mut labels);
    }
    assert!(labels.contains("送信するメッセージに添付していない手書き・原文があります。"), "{labels}");
    assert!(labels.contains("破棄して終了"), "{labels}");
    assert!(labels.contains("作業を続ける"), "{labels}");
    assert_eq!(app.annotation.files().unwrap(), original);
    assert_eq!(app.unsaved_chat_audio.as_ref().unwrap().1, audio);
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), saved_progress);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_exit_discard_clears_only_unsaved_media_and_requests_close() {
    let (ctx, mut app, root) = fixture();
    let id = app.progress.chats[0].id.clone();
    let source = root.join("keep.txt");
    std::fs::write(&source, "keep the draft attachment").unwrap();
    app.attach_chat_file(&id, &source).unwrap();
    app.progress.chats[0].draft = "保存した下書き".into();
    app.storage.as_ref().unwrap().save(&app.progress).unwrap();
    app.annotation.freeze_blank(&ctx).unwrap();
    app.annotation.strokes = vec![vec![[0.1, 0.1]]];
    app.unsaved_chat_audio = Some((id, b"unsaved".to_vec()));
    app.exit_media_requested = true;
    let saved_progress = serde_json::to_value(&app.progress).unwrap();

    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.discard_unsaved_media_and_exit(ctx);
    });
    assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.iter()
        .any(|command| matches!(command, egui::ViewportCommand::Close)));
    assert!(!app.exit_media_requested && !app.chat_media_open);
    assert!(!app.annotation.frozen() && app.unsaved_chat_audio.is_none());
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), saved_progress);
    assert_eq!(serde_json::to_value(app.storage.as_ref().unwrap().load().unwrap()).unwrap(), saved_progress);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn clicking_the_exit_warning_discards_unsaved_media_but_keeps_the_saved_draft() {
    let (ctx, mut app, root) = fixture();
    app.page = Page::Chat;
    app.progress.chats[0].draft = "残す下書き".into();
    let saved_progress = serde_json::to_value(&app.progress).unwrap();
    app.annotation.freeze_blank(&ctx).unwrap();
    app.unsaved_chat_audio = Some((app.progress.chats[0].id.clone(), b"unsaved".to_vec()));
    app.exit_media_requested = true;
    app.chat_media_open = true;

    frame(&ctx, &mut app, false);
    let output = frame(&ctx, &mut app, false);
    fn label_center(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(text) if text.galley.text() == label =>
                Some(text.pos + text.galley.size() * 0.5),
            egui::epaint::Shape::Vec(shapes) =>
                shapes.iter().find_map(|shape| label_center(shape, label)),
            _ => None,
        }
    }
    let position = output.shapes.iter()
        .find_map(|shape| label_center(&shape.shape, "破棄して終了"))
        .expect("visible exit choice");
    let click = |app: &mut WordApp, pressed| ctx.run(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
        events: vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position, button: egui::PointerButton::Primary, pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    }, |ctx| app.update_ui(ctx));
    click(&mut app, true);
    let output = click(&mut app, false);
    assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.iter()
        .any(|command| matches!(command, egui::ViewportCommand::Close)));
    assert!(!app.exit_media_requested && !app.chat_media_open);
    assert!(!app.annotation.frozen() && app.unsaved_chat_audio.is_none());
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), saved_progress);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn attachment_table_viewport_reaches_the_compact_resize_boundary() {
    for (zoom, width) in [(0.8, 1150.0), (1.6, 820.0)] {
        let (ctx, mut app, root) = fixture();
        ctx.set_zoom_factor(zoom);
        app.page = Page::Chat;
        app.chat_media_open = true;
        for count in [0, 1, 8] {
            app.progress.chats[0].draft_attachments = (0..count).map(|index| {
                wordweave5::chat::Attachment {
                    original: wordweave5::assets::AssetRef {
                        id: "a".repeat(64), kind: wordweave5::assets::AssetKind::FileBlob, bytes: 16,
                    }, image: None, background: None,
                    file_name: Some(format!("notes-{index}.txt")),
                    source_text: String::new(), transcript: None,
                }
            }).collect();
            for height in [130.0, 210.0, 360.0] {
                app.chat_media_table_height = height;
                for _ in 0..4 {
                    let _ = ctx.run(egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO, egui::vec2(width, 950.0))),
                        ..Default::default()
                    }, |ctx| app.update_ui(ctx));
                }
                let rect = |id| ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new(id))).unwrap();
                let panel = rect("chat-media-table-frame");
                let viewport = rect("chat-media-table-viewport");
                let handle = rect("chat-media-table-resize-handle");
                let gap = panel.bottom() - viewport.bottom();
                assert!((9.0..=12.0).contains(&gap),
                    "only the bottom frame inset should remain, zoom={zoom} count={count} height={height} gap={gap}");
                assert!((panel.height() - height).abs() <= 2.0,
                    "filling the table must not increase its height: {panel:?}");
                assert_eq!(handle.height(), 14.0, "preserve the draggable hit target");
            }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn attached_table_can_resize_and_remove_only_the_selected_draft_attachment() {
    let (ctx, mut app, root) = fixture();
    let make_ref = |id: char, kind| wordweave5::assets::AssetRef {
        id: id.to_string().repeat(64),
        kind,
        bytes: 16,
    };
    let file = make_ref('a', wordweave5::assets::AssetKind::FileBlob);
    let audio = make_ref('b', wordweave5::assets::AssetKind::AudioWav);
    app.progress.chats[0].draft = "保持する下書き".into();
    app.progress.chats[0].draft_attachments = vec![
        wordweave5::chat::Attachment {
            original: file, image: None, background: None,
            file_name: Some("notes.txt".into()), source_text: "notes.txt".into(),
            transcript: None,
        },
        wordweave5::chat::Attachment {
            original: audio, image: None, background: None,
            file_name: Some("voice.wav".into()), source_text: "voice.wav".into(),
            transcript: None,
        },
    ];
    app.page = Page::Chat;
    app.chat_media_open = true;
    frame(&ctx, &mut app, false);
    let output = frame(&ctx, &mut app, false);
    let mut labels = String::new();
    for shape in &output.shapes {
        shape_text(&shape.shape, &mut labels);
    }
    assert!(labels.contains("notes.txt"), "{labels}");
    assert!(labels.contains("確認・削除"), "{labels}");
    assert!(labels.contains("添付を追加"), "{labels}");
    let handle = ctx.data(|data| data.get_temp::<egui::Rect>(
        egui::Id::new("chat-media-table-resize-handle")))
        .expect("table resize handle");
    let start = handle.center();
    let end = start + egui::vec2(0.0, 52.0);
    let initial_height = app.chat_media_table_height;
    let input = |events| egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
        events,
        ..Default::default()
    };
    let _ = ctx.run(input(vec![
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start, button: egui::PointerButton::Primary, pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]), |ctx| app.update_ui(ctx));
    let _ = ctx.run(input(vec![egui::Event::PointerMoved(end)]), |ctx| app.update_ui(ctx));
    let next = end + egui::vec2(0.0, 15.0);
    let _ = ctx.run(input(vec![egui::Event::PointerMoved(next)]), |ctx| app.update_ui(ctx));
    let _ = ctx.run(input(vec![egui::Event::PointerButton {
        pos: next, button: egui::PointerButton::Primary, pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]), |ctx| app.update_ui(ctx));
    assert!((app.chat_media_table_height - (initial_height + 67.0)).abs() < 4.0,
        "divider should follow each pointer movement without a cumulative jump");

    fn label_center(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(text) if text.galley.text() == label =>
                Some(text.pos + text.galley.size() * 0.5),
            egui::epaint::Shape::Vec(shapes) =>
                shapes.iter().find_map(|shape| label_center(shape, label)),
            _ => None,
        }
    }
    let output = frame(&ctx, &mut app, false);
    let delete = output.shapes.iter()
        .find_map(|shape| label_center(&shape.shape, "添付から削除"))
        .expect("visible delete action in first table row");
    let _ = ctx.run(input(vec![
        egui::Event::PointerMoved(delete),
        egui::Event::PointerButton {
            pos: delete, button: egui::PointerButton::Primary, pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]), |ctx| app.update_ui(ctx));
    let _ = ctx.run(input(vec![egui::Event::PointerButton {
        pos: delete, button: egui::PointerButton::Primary, pressed: false,
        modifiers: egui::Modifiers::NONE,
    }]), |ctx| app.update_ui(ctx));
    assert_eq!(app.progress.chats[0].draft, "保持する下書き");
    assert_eq!(app.progress.chats[0].draft_attachments.len(), 1);
    assert_eq!(app.progress.chats[0].draft_attachments[0].file_name.as_deref(),
        Some("voice.wav"));
    assert_eq!(app.storage.as_ref().unwrap().load().unwrap().chats[0]
        .draft_attachments.len(), 1);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn attachment_previews_stay_in_front_of_the_media_dialog() {
    let (ctx, mut app, root) = fixture();
    app.page = Page::Chat;
    app.chat_media_open = true;
    let media_id = egui::Id::new("ファイル・音声・手書きを添付");
    for (preview_id, file) in [
        ("添付ファイル：notes.txt", true),
        ("保存された画像（送信時の固定版）", false),
    ] {
        if file {
            app.file_preview = Some(("notes.txt".into(), "Preview text.".into(), false));
        } else {
            app.file_preview = None;
            app.preview_pixels = Some(("test-image".into(),
                egui::ColorImage::new([8, 8], egui::Color32::LIGHT_BLUE)));
        }
        frame(&ctx, &mut app, false);
        frame(&ctx, &mut app, false);
        let preview_id = egui::Id::new(preview_id);
        let (media, preview) = ctx.memory(|memory| (
            memory.area_rect(media_id).unwrap(), memory.area_rect(preview_id).unwrap()));
        let overlap = media.intersect(preview);
        assert!(overlap.width() > 20.0 && overlap.height() > 20.0,
            "the two windows should overlap for this test: {media:?} {preview:?}");
        assert_eq!(ctx.layer_id_at(overlap.center()),
            Some(egui::LayerId::new(egui::Order::Foreground, preview_id)));
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn attachment_table_does_not_force_a_narrow_dialog_wider() {
    for (zoom, count, exit_warning, solid_scroll) in [
        (0.8, 0, false, false), (1.0, 3, false, false),
        (1.6, 18, false, true), (1.0, 3, true, false),
    ] {
        let (ctx, mut app, root) = fixture();
        ctx.set_zoom_factor(zoom);
        if solid_scroll {
            ctx.style_mut(|style| style.spacing.scroll = egui::style::ScrollStyle::solid());
        }
        let make_ref = |id: char, kind| wordweave5::assets::AssetRef {
            id: id.to_string().repeat(64), kind, bytes: 16,
        };
        let image = make_ref('a', wordweave5::assets::AssetKind::ImagePng);
        let audio = make_ref('b', wordweave5::assets::AssetKind::AudioWav);
        let file = make_ref('c', wordweave5::assets::AssetKind::FileBlob);
        let samples = [
            wordweave5::chat::Attachment { original: image.clone(), image: Some(image),
                background: None, file_name: Some("sample.png".into()),
                source_text: String::new(), transcript: None },
            wordweave5::chat::Attachment { original: audio, image: None,
                background: None, file_name: Some("voice.wav".into()),
                source_text: String::new(), transcript: None },
            wordweave5::chat::Attachment { original: file, image: None,
                background: None, file_name: Some("notes.txt".into()),
                source_text: String::new(), transcript: None },
        ];
        app.progress.chats[0].draft_attachments = samples.iter().cycle().take(count).cloned().collect();
        app.page = Page::Chat;
        app.chat_media_open = true;
        app.exit_media_requested = exit_warning;
        if exit_warning { app.annotation.text = "Unsaved original.".into(); }
        let run = |app: &mut WordApp, events| {
            let _ = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO, egui::vec2(1400.0, 1000.0))),
                events,
                ..Default::default()
            }, |ctx| app.update_ui(ctx));
            ctx.memory(|memory| memory.area_rect(
                egui::Id::new("ファイル・音声・手書きを添付")).unwrap())
        };
        for _ in 0..5 { run(&mut app, vec![]); }
        for distance in [140.0, 160.0, 140.0] {
            let initial = run(&mut app, vec![]);
            let start = initial.right_center() - egui::vec2(1.0, 0.0);
            run(&mut app, vec![egui::Event::PointerMoved(start)]);
            run(&mut app, vec![egui::Event::PointerButton {
                pos: start, button: egui::PointerButton::Primary, pressed: true,
                modifiers: egui::Modifiers::NONE,
            }]);
            let end = start - egui::vec2(distance, 0.0);
            run(&mut app, vec![egui::Event::PointerMoved(end)]);
            let released = run(&mut app, vec![egui::Event::PointerButton {
                pos: end, button: egui::PointerButton::Primary, pressed: false,
                modifiers: egui::Modifiers::NONE,
            }]);
            assert!(released.width() < initial.width() - distance * 0.5,
                "the test must actually shrink the window: {initial:?} -> {released:?}");
            let mut widths = Vec::new();
            for _ in 0..60 {
                widths.push(run(&mut app, vec![]).width());
            }
            assert!(widths.iter().all(|width| (*width - released.width()).abs() <= 1.0),
                "dialog width must stay at the user's resized width while idle: released={} idle={widths:?}",
                released.width());
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn annotation_discard_modal_cancels_without_loss_and_confirms_only_on_ok() {
    let (ctx, mut app, root) = fixture();
    app.page = Page::Chat;
    app.chat_media_open = true;
    app.annotation.text = "Keep this English original.".into();
    app.discard_annotation_confirm = true;
    frame(&ctx, &mut app, false);
    frame(&ctx, &mut app, false);
    fn center(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(text) if text.galley.text() == label =>
                Some(text.pos + text.galley.size() * 0.5),
            egui::epaint::Shape::Vec(shapes) => shapes.iter()
                .find_map(|shape| center(shape, label)),
            _ => None,
        }
    }
    let input = |events| egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
        events,
        ..Default::default()
    };
    let click = |label: &str, app: &mut WordApp| {
        let output = frame(&ctx, app, false);
        let pos = output.shapes.iter().filter_map(|shape| center(&shape.shape, label))
            .last().unwrap_or_else(|| panic!("modal action absent: {label}"));
        let _ = ctx.run(input(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton { pos, button: egui::PointerButton::Primary,
                pressed: true, modifiers: egui::Modifiers::NONE },
        ]), |ctx| app.update_ui(ctx));
        let _ = ctx.run(input(vec![
            egui::Event::PointerButton { pos, button: egui::PointerButton::Primary,
                pressed: false, modifiers: egui::Modifiers::NONE },
        ]), |ctx| app.update_ui(ctx));
    };
    click("キャンセル", &mut app);
    assert_eq!(app.annotation.text, "Keep this English original.");
    assert!(!app.discard_annotation_confirm);
    app.discard_annotation_confirm = true;
    click("OK（破棄する）", &mut app);
    assert!(app.annotation.text.is_empty());
    assert!(!app.discard_annotation_confirm);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn finishing_recovery_after_an_exit_request_closes_without_discarding_it() {
    let (ctx, mut app, root) = fixture();
    app.annotation.text = "Keep this sentence.".into();
    app.annotation.freeze(&ctx).unwrap();
    let original = app.annotation.files().unwrap();
    app.exit_media_requested = true;
    app.chat_media_open = true;
    let dest = root.join("recovered-annotation");
    app.export_chat_annotation(&dest).unwrap();
    let output = frame(&ctx, &mut app, false);
    assert!(output.viewport_output[&egui::ViewportId::ROOT].commands.iter()
        .any(|command| matches!(command, egui::ViewportCommand::Close)));
    assert_eq!(std::fs::read(dest.join("ink.json")).unwrap(), original.0);
    assert!(!app.exit_media_requested && !app.chat_media_open);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
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
