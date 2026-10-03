// QWEN-SETTINGS-001: synthetic credentials only, never accesses Windows credentials.
use super::qwen_reading_ui::preview_data::Store;
use super::qwen_settings::QwenConnectionEditor;
use qwen_audio::{ApiHost, ApiKey, Connection, ErrorCode, Region};
use std::sync::atomic::Ordering;

const TOKYO: &str = "https://old.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1";
const SINGAPORE: &str = "https://new.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1";

fn configured_store() -> std::sync::Arc<Store> {
    Store::new(Some(Connection::new(
        ApiHost::parse(TOKYO).unwrap(),
        ApiKey::new("synthetic-editor-key".into()).unwrap(),
    )))
}

fn stored_host(store: &Store) -> Option<String> {
    store
        .value
        .lock()
        .unwrap()
        .as_ref()
        .map(|c| c.host().as_str().to_owned())
}

// QS-AC-010/011: opening is read-only, and the opaque old key is never redisplayed.
#[test]
fn opening_editor_shows_saved_destination_without_key_or_automatic_save() {
    let store = configured_store();
    let editor = QwenConnectionEditor::new(store.clone()).unwrap();
    assert_eq!(editor.region, Region::Tokyo);
    assert_eq!(editor.host, TOKYO);
    assert!(editor.key.is_empty());
    assert!(!editor.dirty());
    assert!(!editor.saved);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(
        editor.summary().unwrap(),
        Some((Region::Tokyo, TOKYO.into()))
    );
}

// QS-AC-008/011: a normalized identical URL with an empty key keeps the connection.
#[test]
fn normalized_same_destination_accepts_empty_key_and_reopens_saved_connection() {
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = format!("{TOKYO}/");
    editor.save().unwrap();
    assert!(editor.saved);
    assert!(!editor.dirty());
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    let reopened = QwenConnectionEditor::new(store.clone()).unwrap();
    assert_eq!(reopened.host, TOKYO);
    assert!(reopened.key.is_empty());
    assert_eq!(store.saves.load(Ordering::SeqCst), 1);
}

#[test]
fn first_connection_requires_explicit_valid_key() {
    let store = Store::new(None);
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = TOKYO.into();
    assert_eq!(editor.save().unwrap_err().code(), ErrorCode::InvalidKey);
    assert!(!editor.saved);
    assert_eq!(stored_host(&store), None);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    *editor.key = "synthetic-first-key".into();
    editor.save().unwrap();
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
}

#[test]
fn changed_workspace_or_region_cannot_reuse_blank_old_key() {
    for (region, host) in [
        (
            Region::Tokyo,
            "https://new.ap-northeast-1.maas.aliyuncs.com",
        ),
        (Region::Singapore, SINGAPORE),
    ] {
        let store = configured_store();
        let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
        editor.select_region(region);
        editor.host = host.into();
        assert!(editor.save().is_err());
        assert!(!editor.saved);
        assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
        assert_eq!(
            editor.summary().unwrap(),
            Some((Region::Tokyo, TOKYO.into()))
        );
        assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    }
}

// QS-AC-007/013: invalid selection/URL must fail before any persistence.
#[test]
fn region_url_mismatch_keeps_existing_connection_and_draft_available() {
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = SINGAPORE.into();
    *editor.key = "synthetic-replacement-key".into();
    assert_eq!(editor.save().unwrap_err().code(), ErrorCode::InvalidHost);
    assert_eq!(editor.host, SINGAPORE);
    assert_eq!(editor.key.as_str(), "synthetic-replacement-key");
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
}

#[test]
fn selecting_new_region_clears_old_destination_and_unsaved_key_without_saving() {
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    *editor.key = "synthetic-draft-key".into();
    editor.select_region(Region::Singapore);
    assert_eq!(editor.region, Region::Singapore);
    assert!(editor.host.is_empty());
    assert!(editor.key.is_empty());
    assert!(editor.dirty());
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
}

// QS-AC-011/013: failed persistence keeps both active baseline and editable draft.
#[test]
fn save_failure_preserves_saved_connection_then_explicit_retry_commits_new_one() {
    let store = configured_store();
    store.fail_save.store(true, Ordering::SeqCst);
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.select_region(Region::Singapore);
    editor.host = SINGAPORE.into();
    *editor.key = "synthetic-retry-key".into();
    assert_eq!(
        editor.save().unwrap_err().code(),
        ErrorCode::CredentialWrite
    );
    assert!(!editor.saved);
    assert!(editor.dirty());
    assert_eq!(editor.host, SINGAPORE);
    assert_eq!(editor.key.as_str(), "synthetic-retry-key");
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(
        editor.summary().unwrap(),
        Some((Region::Tokyo, TOKYO.into()))
    );
    store.fail_save.store(false, Ordering::SeqCst);
    editor.save().unwrap();
    assert!(editor.saved);
    assert!(editor.key.is_empty());
    assert!(!editor.dirty());
    assert_eq!(stored_host(&store).as_deref(), Some(SINGAPORE));
    assert_eq!(
        editor.summary().unwrap(),
        Some((Region::Singapore, SINGAPORE.into()))
    );
    assert_eq!(store.saves.load(Ordering::SeqCst), 2);
}

#[test]
fn read_failure_is_reported_and_never_treated_as_unconfigured_success() {
    let store = configured_store();
    store.fail_load.store(true, Ordering::SeqCst);
    let error = QwenConnectionEditor::new(store.clone())
        .err()
        .expect("read must fail");
    assert_eq!(error.code(), ErrorCode::CredentialRead);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
}

// QS-AC-012: all close requests use the same dirty guard; return preserves draft.
#[test]
fn cancel_request_keeps_dirty_draft_until_explicit_discard() {
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = "https://new.ap-northeast-1.maas.aliyuncs.com".into();
    *editor.key = "synthetic-cancel-key".into();
    assert!(!editor.request_close());
    assert!(editor.discard_requested);
    assert_eq!(editor.key.as_str(), "synthetic-cancel-key");
    assert!(editor.host.contains("//new."));
    // Choosing to return to editing only dismisses the close decision.
    editor.discard_requested = false;
    assert!(editor.dirty());
    assert_eq!(editor.key.as_str(), "synthetic-cancel-key");
    assert!(!editor.request_close());
    drop(editor); // Explicit discard drops the draft, never the saved credential.
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
}

#[test]
fn unchanged_editor_can_close_without_discard_confirmation() {
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    assert!(editor.request_close());
    assert!(!editor.discard_requested);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
}

// QS-AC-011: dedicated credential persistence is independent of general settings.
#[test]
fn general_settings_cancel_does_not_undo_dedicated_qwen_save() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    app.qwen_store = store.clone();
    app.begin_settings_edit();
    let original = app.progress.settings.codex_path.clone();
    app.settings_editor.draft.codex_path = "synthetic-cli-draft".into();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.select_region(Region::Singapore);
    editor.host = SINGAPORE.into();
    *editor.key = "synthetic-independent-key".into();
    editor.save().unwrap();
    app.cancel_settings(&ctx);
    assert_eq!(app.progress.settings.codex_path, original);
    assert_eq!(app.settings_editor.draft.codex_path, original);
    assert_eq!(stored_host(&store).as_deref(), Some(SINGAPORE));
    assert_eq!(QwenConnectionEditor::new(store).unwrap().host, SINGAPORE);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn general_settings_save_never_commits_unsaved_qwen_draft() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    app.qwen_store = store.clone();
    app.begin_settings_edit();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = "https://new.ap-northeast-1.maas.aliyuncs.com".into();
    *editor.key = "synthetic-unsaved-key".into();
    app.qwen_settings = Some(editor);
    app.save_settings(&ctx).unwrap();
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert!(app.qwen_settings.as_ref().unwrap().dirty());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

// QS-AC-012: a normal OS close request is held while the draft needs a decision.
#[test]
fn os_close_request_cancels_exit_and_preserves_dirty_editor_and_saved_connection() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = "https://new.ap-northeast-1.maas.aliyuncs.com".into();
    *editor.key = "synthetic-os-close-key".into();
    app.qwen_settings = Some(editor);
    let output = super::harness_tests::frame(&ctx, &mut app, true);
    assert!(output.viewport_output.values().any(|viewport| {
        viewport
            .commands
            .iter()
            .any(|command| matches!(command, eframe::egui::ViewportCommand::CancelClose))
    }));
    let editor = app
        .qwen_settings
        .as_ref()
        .expect("close must preserve dirty editor");
    assert!(editor.discard_requested);
    assert_eq!(editor.key.as_str(), "synthetic-os-close-key");
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

// QS-AC-006: show the destructive navigation explanation only before configuration.
#[test]
fn reading_settings_link_is_only_visible_when_unconfigured() {
    fn draw(
        ctx: &eframe::egui::Context,
        dialog: &mut super::qwen_reading_ui::QwenDialog,
        events: Vec<eframe::egui::Event>,
    ) -> String {
        let output = ctx.run(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(1120.0, 850.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                dialog.render(ctx);
            },
        );
        let mut text = String::new();
        for shape in output.shapes {
            super::harness_tests::shape_text(&shape.shape, &mut text);
        }
        text
    }

    let (ctx, app, root) = super::harness_tests::fixture();
    for (configured, state) in [(false, "unconfigured"), (true, "input")] {
        let link_id = eframe::egui::Id::new("qwen-settings");
        // The same Context is reused, so discard the previous case's widget metadata.
        ctx.data_mut(|data| data.remove::<eframe::egui::Rect>(link_id));
        let mut dialog = super::qwen_reading_ui::synthetic_dialog(&app.deck[0], state, false);
        let mut text = String::new();
        for _ in 0..3 {
            text = draw(&ctx, &mut dialog, vec![]);
        }
        let link = ctx.data(|data| data.get_temp::<eframe::egui::Rect>(link_id));
        assert_eq!(link.is_some(), !configured, "settings link creation");
        if !configured {
            // QS-AC-016 permits scrolling: a widget below the viewport has no painted text.
            // Exercise the actual scroll input rather than treating absent paint as absent UI.
            for _ in 0..8 {
                if text.contains("設定のAI接続を開く") {
                    break;
                }
                let pointer = ctx.screen_rect().center();
                text = draw(
                    &ctx,
                    &mut dialog,
                    vec![
                        eframe::egui::Event::PointerMoved(pointer),
                        eframe::egui::Event::MouseWheel {
                            unit: eframe::egui::MouseWheelUnit::Point,
                            delta: eframe::egui::vec2(0.0, -120.0),
                            modifiers: eframe::egui::Modifiers::NONE,
                        },
                    ],
                );
            }
        }
        assert_eq!(text.contains("設定のAI接続を開く"), !configured, "{text}");
        if !configured {
            assert!(text.contains("音声と結果を破棄します"), "{text}");
        }
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unconfigured_navigation_closes_audio_and_opens_connection_settings() {
    use super::qwen_reading_ui::{preview_data, QwenAction};
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let mut dialog = super::qwen_reading_ui::synthetic_dialog(&app.deck[0], "unconfigured", false);
    dialog.accept_recording(crate::media::CapturedRecording {
        wav: preview_data::audio(),
        warning: None,
    });
    assert!(dialog.raw_audio.is_some());
    assert!(dialog.dispatch_frame(&[QwenAction::GoToSettings], &ctx));
    assert!(dialog.settings_requested);
    assert!(dialog.raw_audio.is_none());
    assert!(dialog.controller.view().result.is_none());
    assert!(dialog.controller.view().preview.is_none());
    assert_eq!(
        dialog.controller.view().phase,
        wordweave5::qwen_reading::ReadingPhase::Closed
    );
    app.qwen_dialog = Some(dialog);
    app.codex_path_guidance = true;
    app.daily_limit_guidance = true;
    app.go_to_qwen_settings();
    assert!(app.qwen_dialog.is_none());
    assert!(app.page == super::Page::Settings);
    assert!(app.settings_section == super::settings_ui::SettingsSection::Connection);
    assert!(!app.codex_path_guidance);
    assert!(!app.daily_limit_guidance);
    assert!(app.settings_editor.active);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

fn prepare_connection_page(app: &mut super::WordApp, store: std::sync::Arc<Store>) {
    app.page = super::Page::Settings;
    app.settings_section = super::settings_ui::SettingsSection::Connection;
    app.qwen_store = store;
    app.begin_settings_edit();
}

fn settle_connection_page_before_editor(ctx: &eframe::egui::Context, app: &mut super::WordApp) {
    assert!(app.qwen_settings.is_none());
    for _ in 0..3 {
        super::harness_tests::frame(ctx, app, false);
    }
    // A real user expands the editor after the page has a native-sized viewport.
    // The constructor's pending zoom can otherwise leave an initial 12500-point clip.
    let screen = ctx.screen_rect();
    assert_eq!(screen.size(), eframe::egui::vec2(1120.0, 850.0));
    let (clip, viewport) = ctx.data(|data| {
        (
            data.get_temp::<eframe::egui::Rect>(eframe::egui::Id::new("qwen-settings-button-clip"))
                .expect("Qwen panel clip after page layout"),
            data.get_temp::<eframe::egui::Rect>(eframe::egui::Id::new("settings-content-viewport"))
                .expect("settings scroll viewport after page layout"),
        )
    });
    assert!(clip.is_finite() && clip.is_positive());
    assert!(viewport.is_finite() && viewport.is_positive());
    assert!(
        screen.contains_rect(clip),
        "page clip must be screen-sized: {clip:?}, screen={screen:?}"
    );
    assert!(
        screen.contains_rect(viewport),
        "scroll viewport must be screen-sized: {viewport:?}, screen={screen:?}"
    );
}

fn assert_focus_returns_to_qwen_edit_button(ctx: &eframe::egui::Context, app: &mut super::WordApp) {
    assert!(app.qwen_settings.is_none());
    for _ in 0..3 {
        super::harness_tests::frame(ctx, app, false);
    }
    let output = super::harness_tests::frame(ctx, app, false);
    let button = ctx.data(|data| {
        data.get_temp::<eframe::egui::Id>(eframe::egui::Id::new("qwen-settings-button-id"))
            .expect("Qwen edit button must be rendered")
    });
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(button));
    assert!(!app.qwen_settings_focus);
    let (rect, clip) = ctx.data(|data| {
        (
            data.get_temp::<eframe::egui::Rect>(eframe::egui::Id::new("qwen-open-settings"))
                .expect("Qwen edit button rectangle"),
            data.get_temp::<eframe::egui::Rect>(eframe::egui::Id::new("qwen-settings-button-clip"))
                .expect("Qwen edit button clip rectangle"),
        )
    });
    assert!(
        clip.contains_rect(rect),
        "button must be fully inside clip: {rect:?}, clip={clip:?}"
    );
    assert!(
        ctx.screen_rect().contains_rect(rect),
        "button must be fully inside screen: {rect:?}"
    );

    fn visible_button_text(shape: &eframe::egui::epaint::Shape, clip: eframe::egui::Rect) -> bool {
        match shape {
            eframe::egui::epaint::Shape::Text(text)
                if text.galley.text() == "Qwen接続設定を編集" =>
            {
                clip.contains_rect(eframe::egui::Rect::from_min_size(
                    text.pos,
                    text.galley.size(),
                ))
            }
            eframe::egui::epaint::Shape::Vec(shapes) => {
                shapes.iter().any(|shape| visible_button_text(shape, clip))
            }
            _ => false,
        }
    }
    assert!(
        output
            .shapes
            .iter()
            .any(|shape| visible_button_text(&shape.shape, shape.clip_rect)),
        "Qwen edit button text must be painted inside its final frame clip"
    );
}

// QS-AC-011/016: successful commit closes through the same focus restoration path.
#[test]
fn dedicated_save_returns_keyboard_focus_to_edit_button() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    prepare_connection_page(&mut app, store.clone());
    settle_connection_page_before_editor(&ctx, &mut app);
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.select_region(Region::Singapore);
    editor.host = SINGAPORE.into();
    *editor.key = "synthetic-focus-save-key".into();
    app.qwen_settings = Some(editor);
    super::harness_tests::frame(&ctx, &mut app, false);
    app.qwen_settings
        .as_mut()
        .expect("rendered editor")
        .save()
        .unwrap();
    app.finish_qwen_settings();
    assert_eq!(
        app.qwen_summary,
        Some(Ok(Some((Region::Singapore, SINGAPORE.into()))))
    );
    assert!(app.message.contains("保存した"));
    assert_focus_returns_to_qwen_edit_button(&ctx, &mut app);
    assert_eq!(stored_host(&store).as_deref(), Some(SINGAPORE));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unchanged_cancel_returns_keyboard_focus_without_saving() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    prepare_connection_page(&mut app, store.clone());
    settle_connection_page_before_editor(&ctx, &mut app);
    app.qwen_settings = Some(QwenConnectionEditor::new(store.clone()).unwrap());
    super::harness_tests::frame(&ctx, &mut app, false);
    assert!(app
        .qwen_settings
        .as_mut()
        .expect("rendered editor")
        .request_close());
    app.finish_qwen_settings();
    assert_focus_returns_to_qwen_edit_button(&ctx, &mut app);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn confirmed_discard_returns_keyboard_focus_and_preserves_saved_summary() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    prepare_connection_page(&mut app, store.clone());
    settle_connection_page_before_editor(&ctx, &mut app);
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    editor.host = "https://discarded.ap-northeast-1.maas.aliyuncs.com".into();
    *editor.key = "synthetic-focus-discard-key".into();
    app.qwen_settings = Some(editor);
    super::harness_tests::frame(&ctx, &mut app, false);
    assert!(!app
        .qwen_settings
        .as_mut()
        .expect("rendered editor")
        .request_close());
    assert!(app.qwen_settings.as_ref().unwrap().discard_requested);
    super::harness_tests::frame(&ctx, &mut app, false);
    // The confirmed discard action invokes the shared editor completion path.
    app.finish_qwen_settings();
    assert_eq!(
        app.qwen_summary,
        Some(Ok(Some((Region::Tokyo, TOKYO.into()))))
    );
    assert_focus_returns_to_qwen_edit_button(&ctx, &mut app);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn os_close_of_unchanged_editor_holds_exit_and_returns_keyboard_focus() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    prepare_connection_page(&mut app, store.clone());
    settle_connection_page_before_editor(&ctx, &mut app);
    app.qwen_settings = Some(QwenConnectionEditor::new(store.clone()).unwrap());
    super::harness_tests::frame(&ctx, &mut app, false);
    assert!(app.qwen_settings.is_some());
    let output = super::harness_tests::frame(&ctx, &mut app, true);
    assert!(output.viewport_output.values().any(|viewport| {
        viewport
            .commands
            .iter()
            .any(|command| matches!(command, eframe::egui::ViewportCommand::CancelClose))
    }));
    assert_focus_returns_to_qwen_edit_button(&ctx, &mut app);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

fn settings_input_frame(
    ctx: &eframe::egui::Context,
    app: &mut super::WordApp,
    events: Vec<eframe::egui::Event>,
) {
    let mut input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(1120.0, 850.0),
        )),
        events,
        ..Default::default()
    };
    app.settings_zoom_input(ctx, &mut input);
    let _ = ctx.run(input, |ctx| app.update_ui(ctx));
}

// QU-AC-011/017: the inline editor remains operable while general save is blocked.
#[test]
fn background_settings_save_click_is_blocked_while_qwen_editor_is_open() {
    use eframe::egui::{Event, Id, Modifiers, PointerButton, Rect};
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    prepare_connection_page(&mut app, store.clone());
    let original = app.progress.settings.codex_path.clone();
    app.settings_editor.draft.codex_path = "synthetic-background-draft".into();
    settle_connection_page_before_editor(&ctx, &mut app);
    let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
    *editor.key = "synthetic-open-inline-key".into();
    app.qwen_settings = Some(editor);
    for _ in 0..3 {
        settings_input_frame(&ctx, &mut app, vec![]);
    }
    let rect = ctx.data(|data| {
        data.get_temp::<Rect>(Id::new("settings-save-button"))
            .expect("background settings save button must be rendered")
    });
    for pressed in [true, false] {
        settings_input_frame(
            &ctx,
            &mut app,
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
    assert_eq!(app.progress.settings.codex_path, original);
    assert_eq!(
        app.settings_editor.draft.codex_path,
        "synthetic-background-draft"
    );
    assert_eq!(
        app.storage
            .as_ref()
            .unwrap()
            .load()
            .unwrap()
            .settings
            .codex_path,
        original
    );
    assert!(app.qwen_settings.is_some());
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

// The only fake boundary is external HTTP; editor/store/URL validation are real.
struct EditorProbe {
    replies: std::sync::Mutex<std::collections::VecDeque<u16>>,
    endpoints: std::sync::Mutex<Vec<String>>,
    release: tokio::sync::Semaphore,
}

impl EditorProbe {
    fn new(statuses: &[u16]) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            replies: std::sync::Mutex::new(statuses.iter().copied().collect()),
            endpoints: std::sync::Mutex::new(Vec::new()),
            release: tokio::sync::Semaphore::new(0),
        })
    }
    fn count(&self) -> usize {
        self.endpoints.lock().unwrap().len()
    }
    fn complete_one(&self) {
        self.release.add_permits(1);
    }
}

impl qwen_audio::ProbeTransport for EditorProbe {
    fn send<'a>(&'a self, request: qwen_audio::ProbeRequest) -> qwen_audio::TransportFuture<'a> {
        Box::pin(async move {
            self.endpoints
                .lock()
                .unwrap()
                .push(request.endpoint().to_owned());
            let status = self
                .replies
                .lock()
                .unwrap()
                .pop_front()
                .expect("unexpected automatic request");
            self.release.acquire().await.unwrap().forget();
            let body = if status == 200 {
                br#"{"success":true,"output":{"models":[{"model":"qwen3.8-omni-flash"}]}}"#.to_vec()
            } else {
                b"synthetic-private-provider-response".to_vec()
            };
            Ok(qwen_audio::TransportResponse {
                status,
                body: Box::pin(futures_util::stream::iter(vec![Ok(body)])),
            })
        })
    }
}

fn wait_for_probe_start(transport: &EditorProbe, count: usize) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while transport.count() != count {
        assert!(
            std::time::Instant::now() < deadline,
            "local probe worker start"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn settle_probe(editor: &mut QwenConnectionEditor) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while editor.probe_state == super::qwen_settings::ProbeState::Running {
        editor.poll_probe();
        assert!(
            std::time::Instant::now() < deadline,
            "local probe worker completion"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

// QU-AC-010/012: invalid drafts never use the baseline key for another host.
#[test]
fn probe_rejects_invalid_or_retargeted_drafts_before_communication_or_save() {
    let ctx = eframe::egui::Context::default();
    for (region, host, key, expected) in [
        (
            Region::Tokyo,
            "https://new.ap-northeast-1.maas.aliyuncs.com",
            "",
            ErrorCode::InvalidKey,
        ),
        (
            Region::Tokyo,
            SINGAPORE,
            "synthetic-valid-key",
            ErrorCode::InvalidHost,
        ),
        (
            Region::Tokyo,
            TOKYO,
            "synthetic\nkey",
            ErrorCode::InvalidKey,
        ),
    ] {
        let store = configured_store();
        let transport = EditorProbe::new(&[200]);
        let mut editor =
            QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
        editor.region = region;
        editor.host = host.into();
        *editor.key = key.into();
        assert_eq!(editor.start_probe(&ctx).unwrap_err().code(), expected);
        assert_eq!(transport.count(), 0);
        assert_eq!(store.saves.load(Ordering::SeqCst), 0);
        assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
        assert!(!editor.saved);
    }
    let store = Store::new(None);
    let transport = EditorProbe::new(&[200]);
    let mut editor =
        QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
    editor.host = TOKYO.into();
    assert_eq!(
        editor.start_probe(&ctx).unwrap_err().code(),
        ErrorCode::InvalidKey
    );
    assert_eq!(transport.count(), 0);
}

#[test]
fn unsaved_draft_probe_success_does_not_save_or_replace_baseline_until_explicit_commit() {
    use super::qwen_settings::ProbeState;
    let ctx = eframe::egui::Context::default();
    let store = configured_store();
    let transport = EditorProbe::new(&[200]);
    let mut editor =
        QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
    editor.select_region(Region::Singapore);
    editor.host = SINGAPORE.into();
    *editor.key = "synthetic-probe-draft-key".into();
    editor.start_probe(&ctx).unwrap();
    wait_for_probe_start(&transport, 1);
    assert!(transport.endpoints.lock().unwrap()[0]
        .starts_with("https://new.ap-southeast-1.maas.aliyuncs.com/api/v1/models?"));
    transport.complete_one();
    settle_probe(&mut editor);
    assert_eq!(
        editor.probe_state,
        ProbeState::Completed(Ok(qwen_audio::ProbeSuccess))
    );
    assert!(editor.dirty());
    assert!(!editor.saved);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    editor.save().unwrap();
    assert_eq!(stored_host(&store).as_deref(), Some(SINGAPORE));
    assert_eq!(transport.count(), 1, "saving must not send a new probe");
    assert_eq!(
        editor.probe_state,
        ProbeState::Untested,
        "probe guarantee is not persisted"
    );
}

// QU-AC-015: cancelling wins even when the transport was released to finish.
#[test]
fn probe_double_click_is_single_request_and_cancel_does_not_adopt_late_success() {
    use super::qwen_settings::ProbeState;
    let ctx = eframe::egui::Context::default();
    let store = configured_store();
    let transport = EditorProbe::new(&[200]);
    let mut editor =
        QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
    editor.start_probe(&ctx).unwrap();
    editor.start_probe(&ctx).unwrap();
    wait_for_probe_start(&transport, 1);
    transport.complete_one();
    editor.cancel_probe();
    for _ in 0..3 {
        editor.poll_probe();
    }
    assert_eq!(
        editor.probe_state,
        ProbeState::Completed(Err(qwen_audio::ProbeError::Cancelled))
    );
    assert_eq!(transport.count(), 1);
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
}

#[test]
fn changing_host_key_or_region_invalidates_completed_probe_without_resending() {
    use super::qwen_settings::ProbeState;
    let ctx = eframe::egui::Context::default();
    for changed in 0..3 {
        let store = configured_store();
        let transport = EditorProbe::new(&[200]);
        let mut editor =
            QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
        editor.start_probe(&ctx).unwrap();
        wait_for_probe_start(&transport, 1);
        transport.complete_one();
        settle_probe(&mut editor);
        assert_eq!(
            editor.probe_state,
            ProbeState::Completed(Ok(qwen_audio::ProbeSuccess))
        );
        match changed {
            0 => editor.host = "https://new.ap-northeast-1.maas.aliyuncs.com".into(),
            1 => *editor.key = "synthetic-replaced-key".into(),
            _ => editor.select_region(Region::Singapore),
        }
        editor.poll_probe();
        assert_eq!(editor.probe_state, ProbeState::Untested);
        assert_eq!(transport.count(), 1);
        assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn failed_probe_preserves_draft_and_manual_retry_is_the_only_second_request() {
    use super::qwen_settings::ProbeState;
    let ctx = eframe::egui::Context::default();
    let store = configured_store();
    let transport = EditorProbe::new(&[401, 200]);
    let mut editor =
        QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
    *editor.key = "synthetic-manual-retry-key".into();
    editor.start_probe(&ctx).unwrap();
    wait_for_probe_start(&transport, 1);
    transport.complete_one();
    settle_probe(&mut editor);
    assert_eq!(
        editor.probe_state,
        ProbeState::Completed(Err(qwen_audio::ProbeError::Authentication))
    );
    assert_eq!(editor.key.as_str(), "synthetic-manual-retry-key");
    for _ in 0..3 {
        editor.poll_probe();
    }
    assert_eq!(transport.count(), 1);
    editor.start_probe(&ctx).unwrap();
    wait_for_probe_start(&transport, 2);
    transport.complete_one();
    settle_probe(&mut editor);
    assert_eq!(
        editor.probe_state,
        ProbeState::Completed(Ok(qwen_audio::ProbeSuccess))
    );
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
}

#[test]
fn failed_save_or_close_discards_pending_probe_completion_and_preserves_connection() {
    use super::qwen_settings::ProbeState;
    let ctx = eframe::egui::Context::default();
    for close in [false, true] {
        let store = configured_store();
        store.fail_save.store(true, Ordering::SeqCst);
        let transport = EditorProbe::new(&[200]);
        let mut editor =
            QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap();
        *editor.key = "synthetic-preserved-draft-key".into();
        editor.start_probe(&ctx).unwrap();
        wait_for_probe_start(&transport, 1);
        transport.complete_one();
        if close {
            assert!(!editor.request_close());
            assert!(editor.discard_requested);
        } else {
            assert_eq!(
                editor.save().unwrap_err().code(),
                ErrorCode::CredentialWrite
            );
        }
        for _ in 0..3 {
            editor.poll_probe();
        }
        assert_eq!(editor.probe_state, ProbeState::Untested);
        assert_eq!(editor.key.as_str(), "synthetic-preserved-draft-key");
        assert!(editor.dirty());
        assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
        assert_eq!(transport.count(), 1);
    }
}

// QU-AC-009/011/017: render the actual AI connection page, not an obsolete modal.
#[test]
fn inline_qwen_editor_accepts_keyboard_input_and_keeps_general_transaction_blocked() {
    use eframe::egui::{self, Event, Id, Modifiers, MouseWheelUnit, Rect};
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let store = configured_store();
    let transport = EditorProbe::new(&[200]);
    prepare_connection_page(&mut app, store.clone());
    let original = app.progress.settings.codex_path.clone();
    app.settings_editor.draft.codex_path = "synthetic-general-draft".into();
    settle_connection_page_before_editor(&ctx, &mut app);
    app.qwen_settings =
        Some(QwenConnectionEditor::new_with_transport(store.clone(), transport.clone()).unwrap());
    let mut key_control = None;
    for _ in 0..20 {
        settings_input_frame(&ctx, &mut app, vec![]);
        let control = ctx
            .data(|data| data.get_temp::<(Rect, Id, bool)>(Id::new("qwen-key-input")))
            .unwrap();
        assert!(control.2, "inline field must stay enabled");
        let viewport = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("settings-content-viewport")))
            .unwrap();
        if ctx.screen_rect().contains_rect(control.0) && viewport.contains_rect(control.0) {
            key_control = Some(control);
            break;
        }
        settings_input_frame(
            &ctx,
            &mut app,
            vec![
                Event::PointerMoved(viewport.center()),
                Event::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -120.0),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    let (_, key_id, _) = key_control.expect("key input must be scroll-reachable");
    ctx.memory_mut(|memory| memory.request_focus(key_id));
    settings_input_frame(
        &ctx,
        &mut app,
        vec![Event::Text("synthetic-typed-inline-key".into())],
    );
    assert_eq!(
        app.qwen_settings.as_ref().unwrap().key.as_str(),
        "synthetic-typed-inline-key"
    );
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(key_id));
    assert!(app.page == super::Page::Settings);
    assert!(app.settings_section == super::settings_ui::SettingsSection::Connection);
    assert_eq!(app.progress.settings.codex_path, original);
    assert_eq!(
        app.settings_editor.draft.codex_path,
        "synthetic-general-draft"
    );
    assert_eq!(store.saves.load(Ordering::SeqCst), 0);
    assert_eq!(transport.count(), 0, "opening and typing do not probe");
    assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

fn guidance_notice_frame(
    ctx: &eframe::egui::Context,
    app: &mut super::WordApp,
    operation: bool,
    events: Vec<eframe::egui::Event>,
) -> eframe::egui::FullOutput {
    ctx.run(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(1120.0, 850.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            if operation {
                app.operation_notice(ctx);
            } else {
                app.notification_window(ctx);
            }
        },
    )
}

fn visible_notice_action(
    output: &eframe::egui::FullOutput,
    label: &str,
) -> Option<eframe::egui::Rect> {
    use eframe::egui::{Rect, Shape};
    fn find(shape: &Shape, label: &str) -> Option<Rect> {
        match shape {
            Shape::Text(text) if text.galley.text() == label => {
                Some(Rect::from_min_size(text.pos, text.galley.size()))
            }
            Shape::Vec(parts) => parts.iter().find_map(|part| find(part, label)),
            _ => None,
        }
    }
    output
        .shapes
        .iter()
        .filter_map(|shape| {
            find(&shape.shape, label).filter(|rect| shape.clip_rect.contains_rect(*rect))
        })
        .next()
}

// QU-AC-011: notification shortcuts must not bypass the Qwen edit transaction.
#[test]
fn codex_and_daily_limit_notice_clicks_preserve_dirty_qwen_editor_and_general_draft() {
    use eframe::egui::{Event, Id, Modifiers, PointerButton, Rect};
    for mode in ["codex-operation", "codex-window", "daily-window"] {
        let (ctx, mut app, root) = super::harness_tests::fixture();
        let store = configured_store();
        prepare_connection_page(&mut app, store.clone());
        settle_connection_page_before_editor(&ctx, &mut app);
        app.settings_editor.draft.codex_path = "synthetic-preserved-general-path".into();
        app.settings_editor.draft.ai_daily_limit = 800;
        let mut editor = QwenConnectionEditor::new(store.clone()).unwrap();
        *editor.key = "synthetic-preserved-qwen-key".into();
        app.qwen_settings = Some(editor);
        if mode == "daily-window" {
            app.progress.settings.ai_daily_limit = 1;
            app.progress.ai_calls.insert(super::today(), 1);
            assert!(!app.reserve_generation());
        } else {
            app.notify_error("Codexの実行ファイルを指定してください。".into());
        }
        let before = serde_json::to_value(&app.progress).unwrap();
        app.notification_open = mode != "codex-operation";
        let operation = mode == "codex-operation";
        let label = if mode == "daily-window" {
            "生成・添削の上限設定へ"
        } else {
            "設定の実行ファイル欄へ"
        };
        let mut output = guidance_notice_frame(&ctx, &mut app, operation, vec![]);
        for _ in 0..4 {
            output = guidance_notice_frame(&ctx, &mut app, operation, vec![]);
        }
        let action =
            visible_notice_action(&output, label).expect("real notice action must be visible");
        assert!(ctx.screen_rect().contains_rect(action));
        for pressed in [true, false] {
            guidance_notice_frame(
                &ctx,
                &mut app,
                operation,
                vec![
                    Event::PointerMoved(action.center()),
                    Event::PointerButton {
                        pos: action.center(),
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        // Direct entry must preserve the same guard, independently of disabled UI.
        app.open_codex_path_guidance();
        assert!(
            !app.codex_path_guidance && !app.daily_limit_guidance,
            "{mode}: navigation bypassed editor guard"
        );
        assert_eq!(
            app.notification_open, !operation,
            "{mode}: blocked action must not dismiss its notice"
        );
        assert_eq!(
            app.qwen_settings.as_ref().unwrap().key.as_str(),
            "synthetic-preserved-qwen-key"
        );
        assert_eq!(
            app.settings_editor.draft.codex_path,
            "synthetic-preserved-general-path"
        );
        assert_eq!(app.settings_editor.draft.ai_daily_limit, 800);
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(stored_host(&store).as_deref(), Some(TOKYO));
        assert_eq!(store.saves.load(Ordering::SeqCst), 0);
        // Even an earlier guidance flag must not replace the inline card's body.
        app.codex_path_guidance = true;
        app.notification_open = false;
        for _ in 0..4 {
            super::harness_tests::frame(&ctx, &mut app, false);
        }
        let (host_rect, _, enabled) = ctx
            .data(|data| data.get_temp::<(Rect, Id, bool)>(Id::new("qwen-host-input")))
            .expect("Qwen editor remains drawn in actual settings page");
        assert!(enabled);
        let viewport = ctx
            .data(|data| data.get_temp::<Rect>(Id::new("settings-content-viewport")))
            .unwrap();
        assert!(
            ctx.screen_rect().contains_rect(host_rect) && viewport.contains_rect(host_rect),
            "{mode}: inline editor must remain visible: {host_rect:?} in {viewport:?}"
        );
        assert!(app.qwen_settings.is_some());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
