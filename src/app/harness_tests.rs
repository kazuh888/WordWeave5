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
fn feedback_ui_palette_all_seven_pages_and_later_window_use_fixed_bases_without_compounding() {
    let (ctx, mut app, root) = fixture();
    let blue = wordweave5::store::TintChoice { rgb: [0, 0, 255], depth: 100 };
    app.begin_color_editor();
    app.color_editor.as_mut().unwrap().draft.page = blue;
    let page_color = egui::Color32::from_rgb(198, 200, 253);
    let window_color = egui::Color32::from_rgb(204, 204, 255);
    fn has_fill(shape: &egui::epaint::Shape, expected: egui::Color32) -> bool {
        match shape {
            egui::epaint::Shape::Rect(rect) => rect.fill == expected,
            egui::epaint::Shape::Vec(parts) => parts.iter().any(|part| has_fill(part, expected)),
            _ => false,
        }
    }
    fn collect_fills(shape: &egui::epaint::Shape, rgba: &mut std::collections::BTreeSet<[u8; 4]>) {
        match shape {
            egui::epaint::Shape::Rect(rect) => { rgba.insert(rect.fill.to_array()); },
            egui::epaint::Shape::Vec(parts) => for part in parts { collect_fills(part, rgba); },
            _ => {}
        }
    }
    for (page, page_name) in [(Page::Home, "home"), (Page::Study, "study"), (Page::Deck, "deck"),
        (Page::Words, "words"), (Page::Chat, "chat"), (Page::Stats, "stats"), (Page::Settings, "settings")] {
        app.page = page;
        // egui Window's Area fades in for 1/12 s; RawInput advances by one predicted_dt per frame.
        for _ in 0..8 { frame(&ctx, &mut app, false); }
        let output = frame(&ctx, &mut app, false);
        assert_eq!(ctx.style().visuals.panel_fill, page_color, "page style drift");
        assert_eq!(ctx.style().visuals.window_fill, window_color, "window style drift");
        assert!(output.shapes.iter().any(|shape| has_fill(&shape.shape, page_color)), "page {page_name} did not paint preview");
        let mut rgba = std::collections::BTreeSet::new();
        for shape in &output.shapes { collect_fills(&shape.shape, &mut rgba); }
        assert!(output.shapes.iter().any(|shape| has_fill(&shape.shape, window_color)),
            "later Window on {page_name} did not paint preview: rect fills RGBA={rgba:?}");
    }
    app.cancel_color_editor();
    let output = frame(&ctx, &mut app, false);
    assert_eq!(ctx.style().visuals.panel_fill, egui::Color32::from_rgb(247, 250, 253));
    assert_eq!(ctx.style().visuals.window_fill, egui::Color32::WHITE);
    assert!(!output.shapes.iter().any(|shape| has_fill(&shape.shape, page_color)), "cancel left preview pixels");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn feedback_ui_palette_narrow_window_keeps_whole_bounds_footer_and_scrollable_body() {
    let (ctx, mut app, root) = fixture();
    app.page = Page::Settings;
    app.begin_color_editor();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(820.0 / 1.6, 650.0 / 1.6));
    fn draw(ctx: &egui::Context, app: &mut WordApp, screen: egui::Rect, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(egui::RawInput { screen_rect: Some(screen), events, ..Default::default() }, |ctx| app.update_ui(ctx))
    }
    fn visible(output: &egui::FullOutput, needle: &str) -> Option<egui::Rect> {
        fn find(shape: &egui::epaint::Shape, needle: &str) -> Option<egui::Rect> {
            match shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == needle =>
                    Some(egui::Rect::from_min_size(text.pos, text.galley.size())),
                egui::epaint::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, needle)),
                _ => None,
            }
        }
        output.shapes.iter().find_map(|shape| {
            let rect = find(&shape.shape, needle)?;
            shape.clip_rect.contains_rect(rect).then_some(rect)
        })
    }
    let mut output = draw(&ctx, &mut app, screen, vec![]);
    for _ in 0..3 { output = draw(&ctx, &mut app, screen, vec![]); }
    let window = ctx.memory(|memory| memory.area_rect(egui::Id::new("配色を調整"))).expect("palette window rect");
    assert!(screen.contains_rect(window), "whole palette window escapes narrow viewport: {window:?}");
    for label in ["保存", "キャンセル", "標準色に戻す"] {
        let rect = visible(&output, label).unwrap_or_else(|| panic!("footer {label} not visible"));
        assert!(window.contains_rect(rect) && screen.contains_rect(rect), "footer {label} clipped");
    }
    let pointer = window.left_top() + egui::vec2(window.width() * 0.55, window.height() * 0.48);
    let mut reached_second_group = false;
    for _ in 0..8 {
        output = draw(&ctx, &mut app, screen, vec![egui::Event::PointerMoved(pointer), egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point, delta: egui::vec2(0.0, -120.0), modifiers: egui::Modifiers::NONE,
        }]);
        reached_second_group |= visible(&output, "入力欄の背景").is_some();
        let at_scroll = ctx.memory(|memory| memory.area_rect(egui::Id::new("配色を調整"))).unwrap();
        assert!(screen.contains_rect(at_scroll), "palette window escapes viewport while scrolling");
        for label in ["保存", "キャンセル", "標準色に戻す"] {
            let rect = visible(&output, label).unwrap_or_else(|| panic!("footer {label} lost while scrolling"));
            assert!(at_scroll.contains_rect(rect) && screen.contains_rect(rect), "footer {label} clipped while scrolling");
        }
    }
    output = draw(&ctx, &mut app, screen, vec![]);
    assert!(reached_second_group, "scroll did not show the second color group at any point");
    assert!(visible(&output, "見本　#FFFFFF").is_some(), "scroll did not reach the final sample");
    let window_after = ctx.memory(|memory| memory.area_rect(egui::Id::new("配色を調整"))).unwrap();
    assert!(screen.contains_rect(window_after), "scrolled window escapes viewport");
    for label in ["保存", "キャンセル", "標準色に戻す"] {
        let rect = visible(&output, label).unwrap_or_else(|| panic!("footer {label} lost after scroll"));
        assert!(window_after.contains_rect(rect) && screen.contains_rect(rect));
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
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

pub(super) fn material_ui_draft(app: &WordApp) -> wordweave5::material::Draft {
    use wordweave5::material::{ChangeReason, Draft, Mode, Quote, Snapshot, Source};
    let baseline = app.deck[0].clone();
    let mut candidate = baseline.clone();
    candidate.meaning = "合成の新しい意味🐈".into();
    candidate.usage = "合成の新しい使い方".into();
    let mut chat = wordweave5::chat::Conversation::new();
    chat.complete(
        "Please improve this synthetic sentence.".into(),
        "Use it in a synthetic business email.".into(),
        wordweave5::execution::Execution::default(),
    ).unwrap();
    Draft {
        mode: Mode::Correct,
        baseline: Some(baseline),
        candidate: candidate.clone(),
        source: Source {
            entry_id: candidate.id.clone(),
            conversation_id: chat.id,
            exchange_indices: vec![0],
            at: 123,
            mode: Mode::Correct,
            snapshots: vec![Snapshot { exchange_index: 0, exchange: chat.exchanges[0].clone() }],
        },
        notices: vec![],
        reasons: vec![
            ChangeReason { path: "/meaning".into(), reason: "合成の意味変更理由".into(),
                quotes: vec![Quote { exchange_index: 0, role: "user".into(),
                    quote: "improve this synthetic sentence".into() }] },
            ChangeReason { path: "/usage".into(), reason: "合成の用法変更理由".into(),
                quotes: vec![Quote { exchange_index: 0, role: "assistant".into(),
                    quote: "synthetic business email".into() }] },
        ],
        generated: Some(candidate),
    }
}

fn material_ui_text(output: &egui::FullOutput) -> String {
    let mut text = String::new();
    for shape in &output.shapes { shape_text(&shape.shape, &mut text); }
    text
}

#[test]
fn material_ui_selects_first_change_and_keeps_reasons_quotes_and_context_in_sync() {
    let (ctx, mut app, root) = fixture();
    let mut draft = material_ui_draft(&app);
    let before_progress = serde_json::to_vec(&app.progress).unwrap();
    let before_deck = wordweave5::model::deck_text(&app.deck);
    let before_draft = serde_json::to_vec(&draft).unwrap();
    let render = |ctx: &egui::Context, app: &mut WordApp, draft: &wordweave5::material::Draft,
                  events: Vec<egui::Event>| {
        ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO, egui::vec2(1200.0, 850.0))), events,
            ..Default::default() }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.material_comparison(ui, draft));
        })
    };
    let mut output = render(&ctx, &mut app, &draft, vec![]);
    let mut text = material_ui_text(&output);
    assert!(text.contains("変更：意味") && text.contains("変更：説明・使い方"), "{text}");
    assert!(text.contains("合成の意味変更理由"), "first row must be selected: {text}");
    assert!(text.contains("improve this synthetic sentence"), "selected quote: {text}");
    assert!(!text.contains("合成の用法変更理由"), "unselected reason leaked: {text}");
    assert!(!text.contains("Please improve this synthetic sentence."),
        "fixed full context must begin collapsed: {text}");

    fn text_center(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(t) if t.galley.text() == label =>
                Some(t.pos + t.galley.size() * 0.5),
            egui::epaint::Shape::Vec(items) => items.iter().find_map(|x| text_center(x, label)),
            _ => None,
        }
    }
    let pos = output.shapes.iter().find_map(|x| text_center(&x.shape, "変更：説明・使い方"))
        .expect("second change row must be clickable");
    for events in [
        vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed: true,
            modifiers: egui::Modifiers::NONE }],
        vec![egui::Event::PointerButton { pos, button: egui::PointerButton::Primary,
            pressed: false, modifiers: egui::Modifiers::NONE }],
    ] { render(&ctx, &mut app, &draft, events); }
    output = render(&ctx, &mut app, &draft, vec![]);
    text = material_ui_text(&output);
    assert!(text.contains("合成の用法変更理由") && text.contains("synthetic business email"),
        "second row reason and quote must switch together: {text}");
    assert!(!text.contains("合成の意味変更理由"), "previous reason leaked: {text}");
    assert_eq!(serde_json::to_vec(&draft).unwrap(), before_draft,
        "comparison and selection must not mutate the proposal");

    draft.candidate.usage = draft.baseline.as_ref().unwrap().usage.clone();
    draft.generated = Some(draft.candidate.clone());
    draft.reasons.retain(|reason| reason.path == "/meaning");
    text = material_ui_text(&render(&ctx, &mut app, &draft, vec![]));
    assert!(text.contains("合成の意味変更理由"), "vanished selection must choose first: {text}");
    assert!(!text.contains("合成の用法変更理由"), "vanished row leaked: {text}");
    draft.candidate = draft.baseline.as_ref().unwrap().clone();
    draft.generated = Some(draft.candidate.clone());
    text = material_ui_text(&render(&ctx, &mut app, &draft, vec![]));
    assert!(text.contains("内容の差分はない"), "{text}");
    assert!(!text.contains("合成の意味変更理由") && !text.contains("合成の新しい意味🐈"),
        "zero changes must clear prior detail: {text}");
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before_progress);
    assert_eq!(wordweave5::model::deck_text(&app.deck), before_deck);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_ui_legacy_proposal_explains_missing_fixed_source_before_expansion() {
    let (ctx, mut app, root) = fixture();
    let mut draft = material_ui_draft(&app);
    draft.source.snapshots.clear();
    draft.reasons.clear();
    draft.generated = None;
    let before = serde_json::to_vec(&draft).unwrap();
    app.progress.material_draft = Some(draft);
    let output = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO, egui::vec2(1200.0, 850.0))), ..Default::default() }, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.material_panel(ui));
    });
    let text = material_ui_text(&output);
    assert!(text.contains("旧版の案には固定原文がない"),
        "missing fixed source must be explained without opening a fold: {text}");
    fn phrase_rect(shape: &egui::epaint::Shape, phrase: &str) -> Option<egui::Rect> {
        match shape {
            egui::epaint::Shape::Text(t) if t.galley.text().contains(phrase) =>
                Some(egui::Rect::from_min_size(t.pos, t.galley.size())),
            egui::epaint::Shape::Vec(items) => items.iter().find_map(|x| phrase_rect(x, phrase)),
            _ => None,
        }
    }
    assert!(output.shapes.iter().any(|shape| phrase_rect(&shape.shape,
        "旧版の案には固定原文がない").is_some_and(|rect|
            shape.clip_rect.contains_rect(rect))),
        "missing-source explanation must be visible in the initial viewport");
    assert!(text.contains("元の会話を表示"), "existing source navigation must remain: {text}");
    assert!(!text.contains("Please improve this synthetic sentence."),
        "missing source must not be silently substituted: {text}");
    assert_eq!(serde_json::to_vec(app.progress.material_draft.as_ref().unwrap()).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_ui_new_proposal_starts_with_fixed_context_collapsed_in_the_same_dialog() {
    let (ctx, mut app, root) = fixture();
    ctx.style_mut(|style| style.animation_time = 0.0);
    app.page = Page::Chat;
    app.chat_material_open = true;
    app.progress.chats[0].complete(
        "Please improve this synthetic sentence.".into(),
        "Use it in a synthetic business email.".into(),
        wordweave5::execution::Execution::default(),
    ).unwrap();
    app.progress.chats[0].exchanges[0].for_material = true;
    let mut first = material_ui_draft(&app);
    first.source.conversation_id = app.progress.chats[0].id.clone();
    first.source.snapshots[0].exchange = app.progress.chats[0].exchanges[0].clone();
    let first_source = first.source.clone();
    app.progress.chats[0].exchanges[0].question =
        "The current chat changed after the material snapshot.".into();
    app.progress.chats[0].title = "Current synthetic conversation".into();
    app.progress.material_draft = Some(first);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400.0, 1000.0));
    let render = |events: Vec<egui::Event>, app: &mut WordApp| ctx.run(
        egui::RawInput { screen_rect: Some(screen), events, ..Default::default() },
        |ctx| app.update_ui(ctx));
    let mut output = render(vec![], &mut app);
    for _ in 0..2 { output = render(vec![], &mut app); }
    let label = "生成時の固定会話を展開";
    fn label_center(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(t) if t.galley.text() == label =>
                Some(t.pos + t.galley.size() * 0.5),
            egui::epaint::Shape::Vec(items) => items.iter().find_map(|x| label_center(x, label)),
            _ => None,
        }
    }
    let mut pos = None;
    for _ in 0..16 {
        let header = output.shapes.iter().find_map(|clipped| {
            let center = label_center(&clipped.shape, label)?;
            Some((center, clipped.clip_rect))
        });
        if let Some((center, clip)) = header {
            if clip.contains(center) { pos = Some(center); break; }
            render(vec![egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -120.0), modifiers: egui::Modifiers::NONE }],
                &mut app);
        } else {
            let dialog = ctx.memory(|m| m.area_rect(egui::Id::new("教材の根拠と差分を確認")))
                .expect("material dialog must remain open");
            render(vec![egui::Event::PointerMoved(dialog.center()),
                egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -120.0), modifiers: egui::Modifiers::NONE }],
                &mut app);
        }
        output = render(vec![], &mut app);
    }
    pos.expect("fixed context header must be visibly reachable by body scrolling");
    for _ in 0..30 { output = render(vec![], &mut app); }
    let (pos, body_clip) = output.shapes.iter().find_map(|clipped| {
        let center = label_center(&clipped.shape, label)?;
        clipped.clip_rect.contains(center).then_some((center, clipped.clip_rect))
    }).expect("fixed context header must remain visible after scroll settles");
    render(vec![egui::Event::PointerMoved(pos)], &mut app);
    for events in [
        vec![egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed: true,
            modifiers: egui::Modifiers::NONE }],
        vec![egui::Event::PointerButton { pos, button: egui::PointerButton::Primary,
            pressed: false, modifiers: egui::Modifiers::NONE }],
    ] { render(events, &mut app); }
    for _ in 0..3 { output = render(vec![], &mut app); }
    // Opening a fold at the bottom of the scroll viewport does not put its
    // contents on screen. Scroll the same body until the snapshot heading is
    // actually visible, rather than treating an off-screen shape as evidence.
    let turn_visible = |output: &egui::FullOutput| output.shapes.iter().any(|clipped| {
        label_center(&clipped.shape, "往復 1")
            .is_some_and(|center| clipped.clip_rect.contains(center))
    });
    let scroll_point = egui::pos2(pos.x, body_clip.center().y);
    for _ in 0..20 {
        if turn_visible(&output) { break; }
        render(vec![egui::Event::PointerMoved(scroll_point),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -40.0), modifiers: egui::Modifiers::NONE }], &mut app);
        output = render(vec![], &mut app);
    }
    assert!(turn_visible(&output),
        "first proposal must reveal its fixed conversation after clicking and scrolling: header={pos:?}, body_clip={body_clip:?}, scroll_point={scroll_point:?}");

    let mut second = material_ui_draft(&app);
    second.source = first_source;
    second.candidate.meaning = "合成の別案にだけある意味".into();
    second.generated = Some(second.candidate.clone());
    let (tx, rx) = std::sync::mpsc::channel();
    app.pending = Some(Pending { kind: Activity::Material, key: String::new(),
        rx, cancel: None });
    tx.send(Ok(AiResult::Material(second))).unwrap();
    output = render(vec![], &mut app);
    assert!(app.pending.is_none(), "synthetic material result must pass through tick");
    let text = material_ui_text(&output);
    assert!(text.contains("合成の別案にだけある意味"),
        "the same dialog must render the second proposal: {text}");
    assert!(text.contains(label), "fixed context fold must remain available: {text}");
    assert!(!text.contains("\n往復 1\n"),
        "new proposal must not inherit the previous proposal's expanded context: {text}");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_ui_mode_names_change_only_the_ui_and_preserve_serialized_mode() {
    use wordweave5::material::Mode;
    let (ctx, mut app, root) = fixture();
    app.material_base = app.deck[0].base.clone();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.material_panel(ui));
    });
    let text = material_ui_text(&output);
    assert!(text.contains("追加のみ") && text.contains("内容を見直す（追加・変更・削除）"), "{text}");
    assert!(text.contains("復習") && text.contains("再学習"), "mode impact must be explained: {text}");
    app.progress.material_draft = Some(material_ui_draft(&app));
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.material_panel(ui));
    });
    assert!(material_ui_text(&output).contains("内容を見直す（追加・変更・削除）"));
    assert_eq!(Mode::Append.label(), "追加（例文・言い換え）");
    assert_eq!(Mode::Correct.label(), "訂正（既存内容を変更）");
    assert_eq!(serde_json::to_string(&Mode::Append).unwrap(), "\"Append\"");
    assert_eq!(serde_json::to_string(&Mode::Correct).unwrap(), "\"Correct\"");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_detail_shows_all_replacements_without_expanding_a_fold() {
    let (ctx, mut app, root) = fixture();
    app.deck[0].replacements = vec![
        wordweave5::model::Replacement { phrase: "strengthen".into(),
            meaning: "合成の強化する意味".into(), conditions: "合成の業務上の条件".into() },
        wordweave5::model::Replacement { phrase: "reinforce".into(),
            meaning: "合成の補強する意味".into(), conditions: "合成の学習場面の条件".into() },
    ];
    let before = wordweave5::model::deck_text(&app.deck);
    let output = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO, egui::vec2(1400.0, 950.0))), ..Default::default() }, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.deck_page(ui));
    });
    let text = material_ui_text(&output);
    for expected in ["言い換えと使い分け", "strengthen", "合成の強化する意味",
        "合成の業務上の条件", "reinforce", "合成の補強する意味", "合成の学習場面の条件",
        "語調・文体", "社外メール", "格調・文体"] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    assert_eq!(wordweave5::model::deck_text(&app.deck), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling() {
    let (ctx, mut app, root) = fixture();
    ctx.set_zoom_factor(1.6);
    app.page = Page::Chat;
    app.chat_material_open = true;
    let mut draft = material_ui_draft(&app);
    draft.candidate.usage = "Synthetic long explanation for a narrow dialog. ".repeat(30);
    draft.generated = Some(draft.candidate.clone());
    app.progress.material_draft = Some(draft);
    let before = serde_json::to_vec(&app.progress).unwrap();
    let deck_before = wordweave5::model::deck_text(&app.deck);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.5, 406.25));
    let render = |events: Vec<egui::Event>, app: &mut WordApp| ctx.run(
        egui::RawInput { screen_rect: Some(screen), events, ..Default::default() },
        |ctx| app.update_ui(ctx));
    let mut output = render(vec![], &mut app);
    for _ in 0..3 { output = render(vec![], &mut app); }
    let dialog = ctx.memory(|memory| memory.area_rect(
        egui::Id::new("教材の根拠と差分を確認")))
        .expect("material dialog must open");
    assert!(screen.contains_rect(dialog),
        "the entire narrow material dialog must fit inside the viewport: {dialog:?} vs {screen:?}");
    fn visible_text_rect(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
        fn find(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Rect> {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == label =>
                    Some(egui::Rect::from_min_size(t.pos, t.galley.size())),
                egui::epaint::Shape::Vec(items) => items.iter().find_map(|x| find(x, label)),
                _ => None,
            }
        }
        output.shapes.iter().find_map(|clipped| {
            let rect = find(&clipped.shape, label)?;
            clipped.clip_rect.contains_rect(rect).then_some(rect)
        })
    }
    let heading = visible_text_rect(&output, "チャットを教材に反映")
        .expect("material heading must be visible in narrow dialog");
    visible_text_rect(&output, "元の会話を表示")
        .expect("proposal body must have a visible source action");
    let body_viewport = ctx.data(|data| data.get_temp::<egui::Rect>(
        egui::Id::new("material-change-detail-viewport")))
        .expect("comparison detail must expose its actual scroll viewport");
    assert!(body_viewport.is_positive() && screen.contains_rect(body_viewport),
        "comparison viewport must be visible: {body_viewport:?}");
    let body_point = body_viewport.center();
    let register = visible_text_rect(&output, "内容を確認して教材に登録")
        .expect("registration action must be visible in narrow dialog");
    let discard = visible_text_rect(&output, "教材案を破棄")
        .expect("discard action must be visible in narrow dialog");
    for rect in [heading, register, discard] {
        assert!(screen.contains_rect(rect), "action outside narrow viewport: {rect:?}");
    }
    let mut reached = std::collections::BTreeSet::new();
    for _ in 0..48 {
        for label in ["変更箇所", "変更：意味", "変更前", "登録される内容"] {
            if visible_text_rect(&output, label).is_some_and(|rect| screen.contains_rect(rect)) {
                reached.insert(label);
            }
        }
        if reached.len() == 4 { break; }
        render(vec![egui::Event::PointerMoved(body_point),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -40.0), modifiers: egui::Modifiers::NONE }], &mut app);
        output = render(vec![], &mut app);
    }
    assert_eq!(reached.len(), 4,
        "narrow proposal body must scroll to list and both comparison sides; reached {reached:?}");
    for (label, initial) in [
        ("チャットを教材に反映", heading),
        ("内容を確認して教材に登録", register),
        ("教材案を破棄", discard),
    ] {
        let after = visible_text_rect(&output, label)
            .unwrap_or_else(|| panic!("{label} must remain visible after body scroll"));
        assert!(screen.contains_rect(after), "{label} outside narrow viewport: {after:?}");
        assert!((after.top() - initial.top()).abs() <= 2.0,
            "{label} moved with body scroll: {initial:?} -> {after:?}");
    }
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
    assert_eq!(wordweave5::model::deck_text(&app.deck), deck_before);

    let mut invalid = material_ui_draft(&app);
    invalid.reasons[0].path = format!("/{}", "長い合成パス".repeat(45));
    app.progress.material_draft = Some(invalid);
    let invalid_before = serde_json::to_vec(&app.progress).unwrap();
    for _ in 0..3 { output = render(vec![], &mut app); }
    let invalid_dialog = ctx.memory(|memory| memory.area_rect(
        egui::Id::new("教材の根拠と差分を確認")))
        .expect("material dialog must remain open for invalid proposal");
    assert!(screen.contains_rect(invalid_dialog),
        "the narrow dialog must fit even with a long registration prohibition: {invalid_dialog:?} vs {screen:?}");
    let text = material_ui_text(&output);
    assert!(text.contains("変更理由") && text.contains("長い合成パス"),
        "invalid proposal must expose its long registration prohibition: {}",
        text.chars().take(400).collect::<String>());
    for label in ["チャットを教材に反映", "内容を確認して教材に登録", "教材案を破棄"] {
        let rect = visible_text_rect(&output, label)
            .unwrap_or_else(|| panic!("{label} must remain visible beside long prohibition"));
        assert!(screen.contains_rect(rect), "{label} outside narrow viewport: {rect:?}");
    }
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), invalid_before);
    assert_eq!(wordweave5::model::deck_text(&app.deck), deck_before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn material_ui_narrow_mode_choices_are_reachable_inside_the_dialog() {
    let (ctx, mut app, root) = fixture();
    ctx.set_zoom_factor(1.6);
    app.page = Page::Chat;
    app.chat_material_open = true;
    app.material_base = app.deck[0].base.clone();
    let before = serde_json::to_vec(&app.progress).unwrap();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.5, 406.25));
    let render = |events: Vec<egui::Event>, app: &mut WordApp| ctx.run(
        egui::RawInput { screen_rect: Some(screen), events, ..Default::default() },
        |ctx| app.update_ui(ctx));
    fn visible_label(output: &egui::FullOutput, label: &str, screen: egui::Rect) -> bool {
        fn find(shape: &egui::epaint::Shape, label: &str) -> Option<egui::Rect> {
            match shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == label =>
                    Some(egui::Rect::from_min_size(t.pos, t.galley.size())),
                egui::epaint::Shape::Vec(items) => items.iter().find_map(|x| find(x, label)),
                _ => None,
            }
        }
        output.shapes.iter().any(|clipped| find(&clipped.shape, label)
            .is_some_and(|rect| clipped.clip_rect.contains_rect(rect)
                && screen.contains_rect(rect)))
    }
    let mut output = render(vec![], &mut app);
    let mut reached = std::collections::BTreeSet::new();
    for _ in 0..24 {
        for label in ["追加のみ", "内容を見直す（追加・変更・削除）", "教材案を作成"] {
            if visible_label(&output, label, screen) { reached.insert(label); }
        }
        if reached.len() == 3 { break; }
        let dialog = ctx.memory(|m| m.area_rect(egui::Id::new("教材の根拠と差分を確認")))
            .expect("material dialog must remain open");
        let body_point = dialog.left_top() + egui::vec2(dialog.width() * 0.5,
            dialog.height() * 0.7);
        render(vec![egui::Event::PointerMoved(body_point),
            egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -95.0), modifiers: egui::Modifiers::NONE }], &mut app);
        output = render(vec![], &mut app);
    }
    assert_eq!(reached.len(), 3,
        "narrow dialog must scroll to mode choices and create action; reached {reached:?}");
    assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
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
