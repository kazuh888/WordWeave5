//! Debug-only native capture of the real app using a new, synthetic store.
use super::*;

pub(super) fn synthetic_quote_diagnostic(baseline: &wordweave5::model::Entry) -> String {
    use wordweave5::material::{ChangeReason, Mode, Quote, Request};
    let mut conversation = wordweave5::chat::Conversation::new();
    let original = format!(
        "固定原文：利用者が選択した回答です。**強調**、改行と絵文字🦊を含む。\n{}",
        "根拠を一字ずつ比較するための合成回答。".repeat(16)
    );
    conversation
        .complete(
            "長文診断用の合成質問".into(),
            original,
            wordweave5::execution::Execution::default(),
        )
        .expect("synthetic quote exchange");
    conversation.exchanges[0].for_material = true;
    let request = Request::new(
        &conversation,
        &baseline.base,
        Mode::Correct,
        Some(baseline.clone()),
    )
    .expect("synthetic quote request");
    let mut candidate = baseline.clone();
    candidate.usage = "合成会話に沿った訂正案".into();
    let reason = ChangeReason {
        path: "/usage".into(),
        reason: "合成の説明".into(),
        quotes: vec![Quote {
            exchange_index: 0,
            role: "assistant".into(),
            quote: "原文に存在しない強調抜粋".into(),
        }],
    };
    let error = request
        .build_response(&serde_json::json!({"entry":candidate,"reasons":[reason]}).to_string())
        .expect_err("synthetic quote must fail verification");
    assert!(
        error.contains("対処"),
        "fixture did not produce a user diagnostic"
    );
    error
}

// The native preview uses only a constructed conversation and the real evidence validator.
fn synthetic_structured_notice(
    baseline: &wordweave5::model::Entry,
) -> wordweave5::material::MaterialDiagnostic {
    use wordweave5::material::{MaterialFailure, Mode, Request};
    let mut conversation = wordweave5::chat::Conversation::new();
    let original = format!("# provide infrastructure\n- run the synthetic setup\n- check power safely\n実改行とliteral \\nを区別する。**強調** 🦊\n{}\n固定元発言の終端MARKER🦊",
        "合成の手順を確認する。\n".repeat(48));
    conversation
        .complete(
            "合成の確認質問".into(),
            original,
            wordweave5::execution::Execution::default(),
        )
        .expect("synthetic notice exchange");
    conversation.exchanges[0].for_material = true;
    let request = Request::new(
        &conversation,
        &baseline.base,
        Mode::Correct,
        Some(baseline.clone()),
    )
    .expect("synthetic notice request");
    let mut candidate = baseline.clone();
    candidate.usage = "合成の説明更新".into();
    let quote = format!("# provide infrastructure\n- run the synthetic setup\n- check power safely\n{}\n引用の終端MARKER🐺",
        "合成の誤引用。\n".repeat(24));
    let response = serde_json::json!({"entry":candidate,"reasons":[{"path":"/usage",
        "reason":"合成の変更理由","quotes":[{"exchange_index":0,"role":"assistant","quote":quote}]}]}).to_string();
    match request
        .build_response_detailed(&response)
        .expect_err("synthetic quote mismatch")
    {
        MaterialFailure::Evidence(detail) => detail,
        other => panic!("expected synthetic evidence, got {other:?}"),
    }
}

// U6: fixed, synthetic GFM in both evidence strings; strict quote mismatch opens comparison.
fn synthetic_gfm_notice(
    baseline: &wordweave5::model::Entry,
) -> wordweave5::material::MaterialDiagnostic {
    use wordweave5::material::{MaterialFailure, Mode, Request};
    let mut conversation = wordweave5::chat::Conversation::new();
    let original = "通常の重要と **重要**。\n\n| 番号 | 日本語 | English | コード | 補足 | 最終列 |\n| --- | --- | --- | --- | --- | --- |\n| 一 | 長い日本語の合成セルを繰り返して折返しを確認する | supercalifragilisticexpialidocious | `a\\|b` | 合成の補足説明を折り返す | 横スクロール終端① |\n| 二 | 末尾まで到達するための合成行 | synthetic ending | `tail` | 二行目の補足 | 横スクロール終端② |\n\n原文末尾GFM🦊";
    conversation
        .complete(
            "合成の表を確認する質問".into(),
            original.into(),
            wordweave5::execution::Execution::default(),
        )
        .expect("synthetic GFM exchange");
    conversation.exchanges[0].for_material = true;
    let request = Request::new(
        &conversation,
        &baseline.base,
        Mode::Correct,
        Some(baseline.clone()),
    )
    .expect("synthetic GFM request");
    let mut candidate = baseline.clone();
    candidate.usage = "GFMの合成訂正案".into();
    let quote = original.replace("原文末尾GFM🦊", "誤引用末尾GFM🐺");
    let response = serde_json::json!({"entry":candidate,"reasons":[{"path":"/usage",
        "reason":"合成の変更理由","quotes":[{"exchange_index":0,"role":"assistant","quote":quote}]}]}).to_string();
    match request
        .build_response_detailed(&response)
        .expect_err("synthetic GFM quote mismatch")
    {
        MaterialFailure::Evidence(detail) => detail,
        other => panic!("expected GFM evidence mismatch, got {other:?}"),
    }
}

fn visible_notice_button(ctx: &egui::Context, label: &str, full: bool) -> Option<egui::Pos2> {
    fn find(shape: &egui::Shape, clip: egui::Rect, label: &str, full: bool) -> Option<egui::Pos2> {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == label => {
                let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                (if full {
                    clip.contains_rect(rect)
                } else {
                    clip.contains(rect.center())
                })
                .then_some(rect.center())
            }
            egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, clip, label, full)),
            _ => None,
        }
    }
    let layers: Vec<_> = if full {
        vec![egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("教材の根拠と差分を確認"),
        )]
    } else {
        ctx.memory(|memory| memory.layer_ids().collect())
    };
    ctx.graphics(|graphics| {
        layers.into_iter().find_map(|layer| {
            graphics
                .get(layer)?
                .all_entries()
                .find_map(|entry| find(&entry.shape, entry.clip_rect, label, full))
        })
    })
}

fn gfm_visible_area(ctx: &egui::Context, app: &WordApp) -> egui::Rect {
    let screen = ctx.screen_rect();
    if app.page != Page::Chat {
        return screen;
    }
    let id = &app.progress.chats[app.chat_selected].id;
    let answer_id = egui::Id::new(("chat-copy-answer", id, 0));
    ctx.data(|data| data.get_temp::<egui::Rect>(answer_id.with("clip")))
        .unwrap_or(egui::Rect::NOTHING)
        .intersect(screen)
}

fn visible_gfm_header(ctx: &egui::Context, viewport: egui::Rect) -> Option<egui::Pos2> {
    fn find(shape: &egui::Shape, clip: egui::Rect) -> Option<egui::Pos2> {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == "番号" => {
                let mesh = text.galley.mesh_bounds.translate(text.pos.to_vec2());
                clip.contains_rect(mesh).then_some(mesh.center())
            }
            egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, clip)),
            _ => None,
        }
    }
    let layers: Vec<_> = ctx.memory(|memory| memory.layer_ids().collect());
    ctx.graphics(|graphics| {
        layers.into_iter().find_map(|layer| {
            graphics
                .get(layer)?
                .all_entries()
                .find_map(|entry| find(&entry.shape, entry.clip_rect.intersect(viewport)))
        })
    })
}

fn visible_notice_glyphs(ctx: &egui::Context, marker: &str, viewport: egui::Rect) -> Vec<bool> {
    visible_notice_glyphs_in_layer(ctx, marker, viewport, None)
}

fn visible_notice_glyphs_in_layer(
    ctx: &egui::Context,
    marker: &str,
    viewport: egui::Rect,
    layer: Option<egui::LayerId>,
) -> Vec<bool> {
    fn found(
        shape: &egui::Shape,
        clip: egui::Rect,
        marker: &str,
        seen: &mut [bool],
        pixels_per_point: f32,
    ) {
        match shape {
            egui::Shape::Text(text) => {
                let mut content = String::new();
                let mut visible = Vec::new();
                for row in &text.galley.rows {
                    for glyph in &row.glyphs {
                        content.push(glyph.chr);
                        let local_left_top = glyph.pos.to_vec2() + glyph.uv_rect.offset;
                        let left_top = text.pos
                            + egui::vec2(
                                (local_left_top.x * pixels_per_point).round() / pixels_per_point,
                                (local_left_top.y * pixels_per_point).round() / pixels_per_point,
                            );
                        let glyph_rect = egui::Rect::from_min_size(left_top, glyph.uv_rect.size);
                        visible.push(!glyph.uv_rect.is_nothing() && clip.contains_rect(glyph_rect));
                    }
                }
                for (byte_index, _) in content.match_indices(marker) {
                    let start = content[..byte_index].chars().count();
                    let count = seen.len();
                    for (stored, visible_now) in seen.iter_mut().zip(&visible[start..start + count])
                    {
                        *stored |= *visible_now;
                    }
                }
            }
            egui::Shape::Vec(parts) => {
                for part in parts {
                    found(part, clip, marker, seen, pixels_per_point);
                }
            }
            _ => {}
        }
    }
    let mut seen = vec![false; marker.chars().count()];
    let layers: Vec<_> = if let Some(layer) = layer {
        vec![layer]
    } else {
        ctx.memory(|memory| memory.layer_ids().collect())
    };
    let pixels_per_point = ctx.pixels_per_point();
    ctx.graphics(|graphics| {
        for layer in layers {
            if let Some(list) = graphics.get(layer) {
                for entry in list.all_entries() {
                    found(
                        &entry.shape,
                        entry.clip_rect.intersect(viewport),
                        marker,
                        &mut seen,
                        pixels_per_point,
                    );
                }
            }
        }
    });
    seen
}

fn visible_notice_row(ctx: &egui::Context, marker: &str) -> bool {
    visible_notice_glyphs(ctx, marker, ctx.screen_rect())
        .into_iter()
        .all(|seen| seen)
}

pub(crate) fn run() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .iter()
        .position(|s| s == "--ui-check")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .expect("--ui-check PNG_PATH");
    let small = args.iter().any(|s| s == "--small");
    let qwen_state = args
        .iter()
        .position(|s| s == "--qwen-reading")
        .and_then(|i| args.get(i + 1))
        .cloned();
    if let Some(state) = &qwen_state {
        assert!(
            [
                "input",
                "confirm",
                "running",
                "result",
                "failed",
                "settings",
                "connections",
                "mismatch",
                "unassessable",
                "jsonfailed",
                "fieldsfailed",
                "responsefailed",
                "unconfigured",
                "probe-running",
                "probe-success",
                "probe-auth",
                "probe-permission",
                "probe-unsupported",
                "probe-missing",
                "probe-cancelled",
                "probe-timeout",
                "probe-invalid",
                "probe-network",
                "probe-rate-limit"
            ]
            .contains(&state.as_str()),
            "unknown Qwen preview state"
        );
    }
    let qwen_scale = args
        .iter()
        .position(|s| s == "--qwen-scale")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);
    assert!(
        [0.8, 1.0, 1.25, 1.5, 1.6, 2.0].contains(&qwen_scale),
        "unsupported Qwen preview scale"
    );
    let empty = args.iter().any(|s| s == "--empty");
    let study = args.iter().any(|s| s == "--study");
    let active = args.iter().any(|s| s == "--active");
    let page = args
        .iter()
        .position(|s| s == "--page")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let path_error = args.iter().any(|s| s == "--path-error");
    let path_guidance = args.iter().any(|s| s == "--path-guidance");
    let feedback = args.iter().any(|s| s == "--feedback");
    let voice = args.iter().any(|s| s == "--voice");
    let chat_empty = args.iter().any(|s| s == "--chat-empty");
    let chat_filled = args.iter().any(|s| s == "--chat-filled");
    let chat_rename = args.iter().any(|s| s == "--chat-rename");
    let chat_attachments = args.iter().any(|s| s == "--chat-attachments");
    let chat_consent = args.iter().any(|s| s == "--chat-consent");
    let chat_drop_confirm = args.iter().any(|s| s == "--chat-drop-confirm");
    let chat_media_entry = args.iter().any(|s| s == "--chat-media-entry");
    let chat_media_ink = args.iter().any(|s| s == "--chat-media-ink");
    let chat_media_exit = args.iter().any(|s| s == "--chat-media-exit");
    let chat_media_table_tall = args.iter().any(|s| s == "--chat-media-table-tall");
    let chat_media_preview_image = args.iter().any(|s| s == "--chat-media-preview-image");
    let chat_media_preview_file = args.iter().any(|s| s == "--chat-media-preview-file");
    let chat_media_resize = args.iter().any(|s| s == "--chat-media-resize");
    let chat_media_table = chat_media_table_tall
        || chat_media_preview_image
        || chat_media_preview_file
        || chat_media_resize
        || args.iter().any(|s| s == "--chat-media-table");
    let material_review = args.iter().any(|s| s == "--material-review");
    let material_context = args.iter().any(|s| s == "--material-context");
    let material_context_tail = args.iter().any(|s| s == "--material-context-tail");
    assert!(
        !material_context_tail || material_context,
        "--material-context-tail requires --material-context"
    );
    assert!(
        !material_context || material_review,
        "--material-context requires --material-review"
    );
    let material_detail = args.iter().any(|s| s == "--material-detail");
    let material_mode = args.iter().any(|s| s == "--material-mode");
    let material_scroll = args.iter().any(|s| s == "--material-scroll");
    let palette_preview = args.iter().any(|s| s == "--palette-preview");
    let palette_saved = args.iter().any(|s| s == "--palette-saved");
    let daily_limit = args
        .iter()
        .any(|s| s == "--daily-limit" || s == "--daily-limit-after-jump");
    let daily_limit_after_jump = args.iter().any(|s| s == "--daily-limit-after-jump");
    let quote_long = args
        .iter()
        .any(|s| s == "--quote-long" || s == "--quote-long-scroll");
    let quote_long_scroll = args.iter().any(|s| s == "--quote-long-scroll");
    let notice_structured = args.iter().any(|s| s == "--notice-structured");
    let notice_compare = args
        .iter()
        .any(|s| s == "--notice-compare" || s == "--notice-raw" || s == "--notice-end");
    let notice_raw = args
        .iter()
        .any(|s| s == "--notice-raw" || s == "--notice-end");
    let notice_end = args.iter().any(|s| s == "--notice-end");
    let gfm_notice = args
        .iter()
        .any(|s| s == "--gfm-notice" || s == "--gfm-notice-end");
    let gfm_notice_end = args.iter().any(|s| s == "--gfm-notice-end");
    let gfm_chat = args.iter().any(|s| s == "--gfm-chat");
    let gfm_receipt = args.iter().any(|s| s == "--gfm-receipt");
    let gfm_table_right = args.iter().any(|s| s == "--gfm-table-right");
    assert!(
        !gfm_table_right || gfm_notice || gfm_chat,
        "--gfm-table-right requires --gfm-notice or --gfm-chat"
    );
    // create_dir (not create_dir_all) refuses an existing destination.
    let root = std::env::temp_dir().join(format!(
        "ww-visual-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    std::fs::create_dir(&root).expect("create isolated visual-check directory");
    let store = Storage::at(root.join("data")).expect("isolated store");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WordWeave 5 — UI確認（実データ不使用）")
            .with_inner_size(if args.iter().any(|s| s == "--qwen-minimum") {
                // Reading preview at the real minimum, retaining --small's 480px stress fixture.
                [820.0, 650.0]
            } else if small
                && qwen_state.as_deref().is_some_and(|state| {
                    state == "settings" || state == "connections" || state.starts_with("probe-")
                })
            {
                // Match main.rs: the actual application cannot shrink below this.
                [820.0, 650.0]
            } else if small && qwen_state.is_some() {
                [480.0, 640.0]
            } else if small {
                [820.0, 650.0]
            } else {
                [1150.0, 950.0]
            }),
        ..Default::default()
    };
    eframe::run_native(
        "WordWeave UI check",
        options,
        Box::new(move |cc| {
            let mut app = WordApp::new_with_storage(&cc.egui_ctx, Ok(store));
            // All native previews use fake credentials, including the AI settings page.
            app.qwen_store = super::qwen_reading_ui::preview_data::Store::configured();
            app.qwen_probe_transport = Some(super::qwen_settings::synthetic_transport());
            if empty {
                app.deck.clear();
            }
            if study {
                app.page = Page::Study;
            }
            if active {
                app.start(5);
                if let Some(task) = app.current.as_mut() {
                    task.introduce = false;
                }
            }
            if let Some(page) = page.as_deref() {
                app.page = match page {
                    "study" => Page::Study,
                    "materials" => Page::Deck,
                    "words" => Page::Words,
                    "chat" => Page::Chat,
                    "records" => Page::Stats,
                    "settings" => Page::Settings,
                    _ => Page::Home,
                };
            }
            if let Some(section) = args
                .iter()
                .position(|s| s == "--settings-section")
                .and_then(|index| args.get(index + 1))
            {
                app.page = Page::Settings;
                app.settings_section = match section.as_str() {
                    "voice" => settings_ui::SettingsSection::Voice,
                    "connection" => settings_ui::SettingsSection::Connection,
                    "data" => settings_ui::SettingsSection::Data,
                    "help" => settings_ui::SettingsSection::Help,
                    _ => settings_ui::SettingsSection::Learning,
                };
            }
            if args.iter().any(|s| s == "--vocabulary-file") {
                app.page = Page::Words;
                cc.egui_ctx
                    .data_mut(|d| d.insert_temp(egui::Id::new("preview-vocabulary-file"), true));
            }
            if args.iter().any(|s| s == "--settings-zoom") {
                app.page = Page::Settings;
                app.begin_settings_edit();
                app.settings_editor.zoom_open = true;
            }
            if args.iter().any(|s| s == "--material-import") {
                let mut changed = app.deck[0].clone();
                changed.meaning = "確認用の変更された意味".into();
                let mut added = changed.clone();
                added.id = "preview_new_material".into();
                added.base = "preview phrase".into();
                app.pending_import = Some(vec![changed, added]);
            }
            if chat_empty
                || chat_filled
                || chat_rename
                || chat_attachments
                || chat_consent
                || chat_drop_confirm
                || chat_media_entry
                || chat_media_ink
                || chat_media_exit
                || chat_media_table
            {
                if app.progress.chats.is_empty() {
                    app.progress
                        .chats
                        .push(wordweave5::chat::Conversation::new());
                }
                app.chat_selected = 0;
            }
            if args.iter().any(|s| s == "--records-filled") {
                let today = chrono::Local::now().date_naive();
                for (offset, seconds) in [360, 420, 0, 120, 480, 300, 0].into_iter().enumerate() {
                    let date = (today - chrono::Duration::days(offset as i64)).to_string();
                    app.progress.study_seconds.insert(date.clone(), seconds);
                    for index in 0..seconds / 60 {
                        app.progress.reviews.push(wordweave5::store::Review {
                            key: format!("sample-{}:recall", index % 4),
                            at: 0,
                            date: date.clone(),
                            grade: Grade::Good,
                            assisted: false,
                            method: "keyboard".into(),
                            first: offset == 5,
                            elapsed_days: 1.0,
                            seconds: 60,
                            self_assessed: true,
                        });
                    }
                }
            }
            if chat_filled {
                let chat = &mut app.progress.chats[0];
                chat.complete(
                    "海外出張で使う自己紹介を練習したいです。".into(),
                    "Of course. Let's start with your name, role, and the purpose of your visit."
                        .into(),
                    wordweave5::execution::Execution::default(),
                )
                .expect("synthetic chat exchange");
                chat.complete(
                    "I work as a software engineer. は自然ですか？".into(),
                    "Yes, it is natural. You can also say, “I'm a software engineer,” in conversation.".into(),
                    wordweave5::execution::Execution::default(),
                ).expect("second synthetic chat exchange");
                let mut previous = wordweave5::chat::Conversation::new();
                previous.title = "空港での会話練習".into();
                previous
                    .complete(
                        "Could you tell me where the baggage claim is?".into(),
                        "Certainly. Follow the signs to the first floor.".into(),
                        wordweave5::execution::Execution::default(),
                    )
                    .expect("synthetic previous conversation");
                app.progress.chats.push(previous);
            }
            if chat_attachments || chat_consent || chat_media_table {
                let store = wordweave5::assets::AssetStore::new(
                    app.storage.as_ref().expect("isolated store").dir.clone(),
                );
                let mut png = std::io::Cursor::new(Vec::new());
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                    16,
                    12,
                    image::Rgba([130, 193, 238, 255]),
                ))
                .write_to(&mut png, image::ImageFormat::Png)
                .unwrap();
                let image = store
                    .put(wordweave5::assets::AssetKind::ImagePng, &png.into_inner())
                    .unwrap();
                let mut wav = Vec::new();
                wav.extend_from_slice(b"RIFF");
                wav.extend_from_slice(&38u32.to_le_bytes());
                wav.extend_from_slice(b"WAVEfmt ");
                wav.extend_from_slice(&16u32.to_le_bytes());
                wav.extend_from_slice(&1u16.to_le_bytes());
                wav.extend_from_slice(&1u16.to_le_bytes());
                wav.extend_from_slice(&8000u32.to_le_bytes());
                wav.extend_from_slice(&16000u32.to_le_bytes());
                wav.extend_from_slice(&2u16.to_le_bytes());
                wav.extend_from_slice(&16u16.to_le_bytes());
                wav.extend_from_slice(b"data");
                wav.extend_from_slice(&2u32.to_le_bytes());
                wav.extend_from_slice(&0i16.to_le_bytes());
                let audio = store
                    .put(wordweave5::assets::AssetKind::AudioWav, &wav)
                    .unwrap();
                let file = store
                    .put(
                        wordweave5::assets::AssetKind::FileBlob,
                        "旅先での挨拶を練習する。\nPlease help me practice.".as_bytes(),
                    )
                    .unwrap();
                if chat_media_preview_image {
                    app.preview_asset(&image);
                }
                if chat_media_preview_file {
                    app.preview_file_asset(&file, "trip-notes.txt");
                }
                let items = vec![
                    wordweave5::chat::Attachment {
                        original: image.clone(),
                        image: Some(image),
                        background: None,
                        file_name: Some("airport.png".into()),
                        source_text: "airport.png".into(),
                        transcript: None,
                    },
                    wordweave5::chat::Attachment {
                        original: audio,
                        image: None,
                        background: None,
                        file_name: Some("greeting.wav".into()),
                        source_text: "greeting.wav".into(),
                        transcript: None,
                    },
                    wordweave5::chat::Attachment {
                        original: file,
                        image: None,
                        background: None,
                        file_name: Some("trip-notes.txt".into()),
                        source_text: "trip-notes.txt".into(),
                        transcript: None,
                    },
                ];
                let chat = &mut app.progress.chats[0];
                chat.draft = "添付の内容を使って旅行英語を練習したい。".into();
                chat.draft_attachments = items;
                if chat_attachments {
                    chat.complete(
                        chat.draft.clone(),
                        "空港での挨拶から始めましょう。".into(),
                        wordweave5::execution::Execution::default(),
                    )
                    .unwrap();
                } else if chat_consent {
                    app.pending_chat_file_send = Some(ChatFileConsent {
                        chat_id: chat.id.clone(),
                        question: chat.draft.clone(),
                        attachment_ids: chat
                            .draft_attachments
                            .iter()
                            .map(|a| a.original.id.clone())
                            .collect(),
                        attachment_names: chat
                            .draft_attachments
                            .iter()
                            .map(|a| a.file_name.clone())
                            .collect(),
                        labels: vec![
                            "画像：airport.png".into(),
                            "音声：greeting.wav".into(),
                            "ファイル：trip-notes.txt".into(),
                        ],
                        text_names: vec!["trip-notes.txt".into()],
                    });
                }
            }
            if chat_rename {
                let chat = &app.progress.chats[app.chat_selected];
                app.pending_chat_rename = Some((chat.id.clone(), chat.title.clone()));
            }
            if chat_drop_confirm {
                let drop_dir = app.storage.as_ref().expect("isolated store").dir.clone();
                let text_path = drop_dir.join("travel-note.txt");
                let image_path = drop_dir.join("airport-photo.png");
                std::fs::write(&text_path, "Practice a greeting at the airport.").unwrap();
                image::RgbImage::new(1, 1).save(&image_path).unwrap();
                let chat = &mut app.progress.chats[app.chat_selected];
                chat.draft = "空港での挨拶を練習したい。".into();
                app.pending_chat_drop = Some(ChatDropProposal {
                    chat_id: chat.id.clone(),
                    files: vec![text_path, image_path],
                    unsupported: 0,
                });
            }
            if chat_media_entry || chat_media_ink || chat_media_exit || chat_media_table {
                app.page = Page::Chat;
                app.chat_media_open = true;
                if chat_media_table_tall {
                    app.chat_media_table_height = 360.0;
                }
                if chat_media_entry || chat_media_table {
                    app.annotation.text = "Please review the attached sentence.".into();
                } else if chat_media_ink {
                    app.annotation.text = "Please review the attached sentence.".into();
                    app.annotation.freeze(&cc.egui_ctx).unwrap();
                } else {
                    app.annotation.freeze_blank(&cc.egui_ctx).unwrap();
                    app.exit_media_requested = true;
                }
            }
            if material_review || material_mode {
                use wordweave5::material::{ChangeReason, Draft, Mode, Quote, Snapshot, Source};
                app.page = Page::Chat;
                app.chat_material_open = true;
                if app.progress.chats.is_empty() {
                    app.progress
                        .chats
                        .push(wordweave5::chat::Conversation::new());
                }
                app.chat_selected = 0;
                app.material_base = app.deck[0].base.clone();
                app.material_target = app.deck[0].id.clone();
                app.material_mode = Mode::Correct;
                if material_review {
                    let chat = &mut app.progress.chats[0];
                    chat.complete(
                        "Please improve this synthetic paragraph for a client email.".into(),
                        "Use a **clear phrase** in the client email.\n\n| 語 | 意味 |\n| --- | --- |\n| **clear phrase** | **明確な表現** |\n| together | 一緒に |\n\n固定回答の末尾。".into(),
                        wordweave5::execution::Execution::default(),
                    ).expect("synthetic material conversation");
                    let baseline = app.deck[0].clone();
                    let mut candidate = baseline.clone();
                    candidate.meaning = "合成教材の更新後の意味。長い日本語とEnglish wordsを交え、比較画面の折返しを確認する。".repeat(3);
                    candidate.usage =
                        "In a client email, explain the next action precisely. ".repeat(7);
                    candidate.replacements.push(wordweave5::model::Replacement {
                        phrase: "strengthen".into(),
                        meaning: "合成の強化する意味".into(),
                        conditions: "顧客への改善提案を明確にする場合".into(),
                    });
                    app.progress.material_draft = Some(Draft {
                        mode: Mode::Correct,
                        baseline: Some(baseline),
                        candidate: candidate.clone(),
                        source: Source {
                            entry_id: candidate.id.clone(),
                            conversation_id: chat.id.clone(),
                            exchange_indices: vec![0],
                            at: 123,
                            mode: Mode::Correct,
                            snapshots: vec![Snapshot {
                                exchange_index: 0,
                                exchange: chat.exchanges[0].clone(),
                            }],
                        },
                        notices: vec!["合成案：実教材・実会話は使用していない。".into()],
                        reasons: vec![
                            ChangeReason {
                                path: "/meaning".into(),
                                reason: "顧客向けに意味を詳しくした合成理由".into(),
                                quotes: vec![Quote {
                                    exchange_index: 0,
                                    role: "user".into(),
                                    quote: "client email".into(),
                                }],
                            },
                            ChangeReason {
                                path: "/usage".into(),
                                reason: "使い方を明確にした合成理由".into(),
                                quotes: vec![Quote {
                                    exchange_index: 0,
                                    role: "assistant".into(),
                                    quote: "clear phrase".into(),
                                }],
                            },
                        ],
                        generated: Some(candidate),
                    });
                }
            }
            if material_detail {
                app.page = Page::Deck;
                app.selected = 0;
                app.deck[0].replacements = vec![
                    wordweave5::model::Replacement {
                        phrase: "strengthen".into(),
                        meaning: "合成の強化する意味".into(),
                        conditions: "合成の顧客向け業務メールで使う場合".into(),
                    },
                    wordweave5::model::Replacement {
                        phrase: "reinforce".into(),
                        meaning: "合成の補強する意味".into(),
                        conditions: "合成の長い文脈で意味の違いを確認する場合。".repeat(5),
                    },
                ];
            }
            if voice {
                app.input = Input::Voice;
            }
            if args.iter().any(|s| s == "--candidates") {
                cc.egui_ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("preview-vocabulary-candidates"), true)
                });
            }
            if args.iter().any(|s| s == "--word-file-help") {
                cc.egui_ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("preview-vocabulary-file-help"), true)
                });
            }
            if args.iter().any(|s| s == "--words-filled") {
                app.provided_words = "take\nlook forward to\nas soon as".into();
            }
            if args.iter().any(|s| s == "--speech-selected") {
                app.speech_selected = Some(super::audio_controls::SpeechButton::Toggle);
            }
            if args.iter().any(|s| s == "--finished") {
                app.start(5);
                if let Some(session) = app.session.as_mut() {
                    session.elapsed = Duration::from_secs(300);
                }
                if !empty {
                    app.grade(Grade::Good);
                }
                app.finish();
            }
            if feedback {
                app.revealed = true;
            }
            if path_error || path_guidance {
                app.message = "指定したCodex実行ファイルがありません。設定で実在するファイルを選ぶか、実行ファイル欄をcodexに変更してPATHから自動検出してください。".into();
                app.last_notification_alerts[1] = app.message.clone();
            }
            if path_guidance {
                app.open_codex_path_guidance();
            }
            if palette_preview || palette_saved {
                if page.is_none() {
                    app.page = Page::Settings;
                }
                app.begin_color_editor();
                let editor = app.color_editor.as_mut().expect("synthetic palette editor");
                editor.draft.page = wordweave5::store::TintChoice {
                    rgb: [0, 0, 255],
                    depth: 100,
                };
                editor.draft.input = wordweave5::store::TintChoice {
                    rgb: [255, 0, 0],
                    depth: 100,
                };
                editor.page_hex = "#0000FF".into();
                editor.input_hex = "#FF0000".into();
                editor.page_depth_text = "100".into();
                editor.input_depth_text = "100".into();
                if palette_saved {
                    app.save_color_editor()
                        .expect("save only synthetic palette");
                    app.notification_open = true;
                    app.message = "合成データの配色を保存した。".into();
                }
            }
            if daily_limit {
                app.progress.settings.ai_daily_limit = 1;
                app.progress.ai_calls.insert(today(), 1);
                assert!(
                    !app.reserve_generation(),
                    "synthetic daily limit must reject"
                );
                if daily_limit_after_jump {
                    // The actual click/recheck path is exercised in the headless UI test.
                    // Capture only its resulting view with isolated synthetic state.
                    app.page = Page::Settings;
                    app.settings_section = settings_ui::SettingsSection::Connection;
                    app.daily_limit_guidance = true;
                    app.daily_limit_focus_pending = true;
                    app.notification_open = false;
                }
            }
            if quote_long {
                app.notify_error(synthetic_quote_diagnostic(&app.deck[0]));
                app.notification_open = true;
            }
            if notice_structured {
                app.notify_material_diagnostic(synthetic_structured_notice(&app.deck[0]));
                app.notification_open = true;
            }
            if gfm_notice {
                app.notify_material_diagnostic(synthetic_gfm_notice(&app.deck[0]));
                app.material_notice.as_mut().unwrap().comparison_open = true;
                app.notification_open = true;
            }
            if gfm_chat {
                app.page = Page::Chat;
                if app.progress.chats.is_empty() {
                    app.progress
                        .chats
                        .push(wordweave5::chat::Conversation::new());
                }
                app.chat_selected = 0;
                app.progress.chats[0].complete(
                    "あなたの原文 **重要** は記号のまま。".into(),
                    "通常の重要と **重要**。\n\n| 番号 | 日本語 | English | コード | 補足 | 最終列 |\n| --- | --- | --- | --- | --- | --- |\n| 一 | 長い日本語の合成セルを繰り返して折返しを確認する | supercalifragilisticexpialidocious | `a\\|b` | 合成の補足説明を折り返す | 横スクロール終端① |\n| 二 | 末尾まで到達するための合成行 | synthetic ending | `tail` | 二行目の補足 | 横スクロール終端② |\n\n回答末尾GFM🦊".into(),
                    wordweave5::execution::Execution::default(),
                ).expect("synthetic GFM chat");
            }
            if gfm_receipt {
                app.page = Page::Chat;
                if app.progress.chats.is_empty() {
                    app.progress
                        .chats
                        .push(wordweave5::chat::Conversation::new());
                }
                app.chat_selected = 0;
                app.notify_material_registered("clarity".into());
            }
            if args.iter().any(|s| s == "--notice") {
                app.notification_open = true;
            }
            if let Some(state) = &qwen_state {
                app.page = Page::Deck;
                app.selected = 0;
                app.deck[0].example = super::qwen_reading_ui::preview_data::REFERENCE.into();
                app.deck[0].base = "within".into();
                if state == "settings" || state == "connections" || state.starts_with("probe-") {
                    app.page = Page::Settings;
                    app.settings_section = super::settings_ui::SettingsSection::Connection;
                    app.qwen_settings_focus = state == "settings";
                } else {
                    app.qwen_dialog = Some(super::qwen_reading_ui::synthetic_dialog(
                        &app.deck[0],
                        state,
                        args.iter().any(|s| s == "--qwen-tail"),
                    ));
                }
                cc.egui_ctx.set_zoom_factor(qwen_scale);
            } else {
                cc.egui_ctx.set_zoom_factor(if small { 1.6 } else { 0.8 });
            }
            Ok(Box::new(Capture {
                app,
                output,
                frames: 0,
                qwen_zoom: qwen_state.as_ref().map(|_| qwen_scale),
                qwen_navigate: qwen_state.as_deref() == Some("connections"),
                qwen_editor_preview: qwen_state
                    .as_ref()
                    .filter(|state| state.as_str() == "settings" || state.starts_with("probe-"))
                    .map(|state| (state.clone(), args.iter().any(|s| s == "--qwen-tail"))),
                speech: args.iter().any(|s| s == "--speech"),
                speech_playing: args.iter().any(|s| s == "--speech-selected"),
                word_file_help: args.iter().any(|s| s == "--word-file-help"),
                media_check: chat_media_entry
                    || chat_media_ink
                    || chat_media_exit
                    || chat_media_table,
                material_scroll: material_scroll
                    && (material_review || material_detail || material_mode),
                material_context,
                material_context_tail,
                notice_scroll: quote_long_scroll,
                notice_buttons: if material_context {
                    vec!["生成時の固定会話を展開"]
                } else if notice_structured && notice_raw {
                    vec!["引用と元の回答を確認", "原文を表示"]
                } else if notice_structured && notice_compare {
                    vec!["引用と元の回答を確認"]
                } else {
                    vec![]
                },
                notice_button_index: 0,
                notice_button_position: None,
                notice_button_pressed: false,
                notice_finished_frame: None,
                notice_body_scroll: notice_structured && notice_compare,
                notice_end_scroll: notice_structured && notice_end || gfm_notice_end,
                notice_tail_visible: false,
                notice_tail_marker: if gfm_notice_end {
                    "原文末尾GFM🦊"
                } else {
                    "固定元発言の終端MARKER🦊"
                },
                gfm_table_right,
                gfm_table_reached: false,
                gfm_table_pointer: None,
                gfm_horizontal_hover_frame: None,
                gfm_horizontal_sent: false,
                gfm_table_seen: vec![false; "横スクロール終端②".chars().count()],
                gfm_marker_since: None,
                gfm_capture_mask: None,
                gfm_capture_target: None,
                palette_preview,
                media_resize: chat_media_resize.then_some(MediaResizeCheck {
                    drag_distance: if small { 80.0 } else { 300.0 },
                    start: egui::Pos2::ZERO,
                    initial_width: 0.0,
                    released_width: None,
                }),
            }))
        }),
    )
}

struct Capture {
    app: WordApp,
    output: PathBuf,
    frames: usize,
    qwen_zoom: Option<f32>,
    qwen_navigate: bool,
    qwen_editor_preview: Option<(String, bool)>,
    speech: bool,
    speech_playing: bool,
    word_file_help: bool,
    media_check: bool,
    material_scroll: bool,
    material_context: bool,
    material_context_tail: bool,
    notice_scroll: bool,
    notice_buttons: Vec<&'static str>,
    notice_button_index: usize,
    notice_button_position: Option<egui::Pos2>,
    notice_button_pressed: bool,
    notice_finished_frame: Option<usize>,
    notice_body_scroll: bool,
    notice_end_scroll: bool,
    notice_tail_visible: bool,
    notice_tail_marker: &'static str,
    gfm_table_right: bool,
    gfm_table_reached: bool,
    gfm_table_pointer: Option<egui::Pos2>,
    gfm_horizontal_hover_frame: Option<usize>,
    gfm_horizontal_sent: bool,
    gfm_table_seen: Vec<bool>,
    gfm_marker_since: Option<usize>,
    gfm_capture_mask: Option<Vec<bool>>,
    gfm_capture_target: Option<PathBuf>,
    palette_preview: bool,
    media_resize: Option<MediaResizeCheck>,
}

struct MediaResizeCheck {
    drag_distance: f32,
    start: egui::Pos2,
    initial_width: f32,
    released_width: Option<f32>,
}

impl eframe::App for Capture {
    fn raw_input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        self.app.settings_zoom_input(ctx, input);
        if self.material_scroll && (self.frames == 8 || self.frames == 12) {
            let rect = if self.app.page == Page::Deck {
                ctx.screen_rect()
            } else {
                ctx.memory(|memory| memory.area_rect(egui::Id::new("教材の根拠と差分を確認")))
                    .unwrap_or_else(|| ctx.screen_rect())
            };
            let fallback = rect.left_top()
                + egui::vec2(
                    rect.width()
                        * if self.app.page == Page::Deck {
                            0.72
                        } else {
                            0.5
                        },
                    rect.height()
                        * if self.app.page == Page::Deck {
                            0.72
                        } else {
                            0.45
                        },
                );
            let pointer = ctx
                .data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new("material-change-list-viewport"))
                })
                .filter(|_| self.app.page == Page::Chat)
                .filter(|pane| pane.is_positive() && ctx.screen_rect().contains(pane.center()))
                .map(|pane| pane.center())
                .unwrap_or(fallback);
            input.events.push(egui::Event::PointerMoved(pointer));
            input.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.0,
                    if self.app.page == Page::Deck {
                        -180.0
                    } else {
                        -220.0
                    },
                ),
                modifiers: egui::Modifiers::NONE,
            });
        }
        if self.notice_scroll && (self.frames == 8 || self.frames == 12) {
            let rect = ctx
                .memory(|memory| memory.area_rect(egui::Id::new("notification-details-v2")))
                .expect("synthetic quote notification window");
            let pointer = rect.left_top() + egui::vec2(rect.width() * 0.55, rect.height() * 0.45);
            input.events.push(egui::Event::PointerMoved(pointer));
            input.events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -220.0),
                modifiers: egui::Modifiers::NONE,
            });
        }
        if self.notice_button_index < self.notice_buttons.len() {
            if let Some(pos) = self.notice_button_position {
                input.events.push(egui::Event::PointerMoved(pos));
                input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: !self.notice_button_pressed,
                    modifiers: egui::Modifiers::NONE,
                });
                if self.notice_button_pressed {
                    self.notice_button_index += 1;
                    self.notice_button_position = None;
                    self.notice_button_pressed = false;
                    if self.notice_button_index == self.notice_buttons.len() {
                        self.notice_finished_frame = Some(self.frames);
                    }
                } else {
                    self.notice_button_pressed = true;
                }
            } else if self.frames > 4 {
                let area = if self.material_context {
                    ctx.data(|data| {
                        data.get_temp::<egui::Rect>(egui::Id::new(
                            "material-change-detail-viewport",
                        ))
                    })
                } else {
                    ctx.memory(|memory| {
                        memory.area_rect(egui::Id::new("notification-details-material-v1"))
                    })
                };
                if let Some(rect) = area {
                    let pointer = rect.center();
                    input.events.push(egui::Event::PointerMoved(pointer));
                    if !self.material_context || self.frames % 4 == 0 {
                        input.events.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(
                                0.0,
                                if self.material_context { -12.0 } else { -120.0 },
                            ),
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
            }
        } else if self.notice_end_scroll
            || self.notice_body_scroll
                && self
                    .notice_finished_frame
                    .is_some_and(|finished| self.frames <= finished + 4)
        {
            if let Some(rect) = ctx.memory(|memory| {
                memory.area_rect(egui::Id::new("notification-details-material-v1"))
            }) {
                let pointer = rect.center();
                input.events.push(egui::Event::PointerMoved(pointer));
                input.events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.0,
                        if self.notice_end_scroll {
                            -1800.0
                        } else {
                            -150.0
                        },
                    ),
                    modifiers: egui::Modifiers::NONE,
                });
            }
        }
        if self.material_context
            && self.notice_finished_frame.is_some()
            && self.gfm_marker_since.is_none()
            && self.gfm_capture_target.is_none()
        {
            if let Some(rect) = ctx.data(|data| {
                data.get_temp::<egui::Rect>(egui::Id::new("material-change-detail-viewport"))
            }) {
                input.events.push(egui::Event::PointerMoved(rect.center()));
                if self.frames % 4 == 0 {
                    input.events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -12.0),
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        if self.gfm_table_right
            && !self.gfm_table_reached
            && self.gfm_capture_target.is_none()
            && (8..=500).contains(&self.frames)
            && self.gfm_marker_since.is_none()
        {
            let viewport = gfm_visible_area(ctx, &self.app);
            if self
                .gfm_table_pointer
                .is_some_and(|pointer| !viewport.contains(pointer))
            {
                self.gfm_table_pointer = None;
                self.gfm_horizontal_hover_frame = None;
            }
            if let Some(pointer) = self.gfm_table_pointer.filter(|_| !self.gfm_horizontal_sent) {
                input.events.push(egui::Event::PointerMoved(pointer));
                if self
                    .gfm_horizontal_hover_frame
                    .is_some_and(|frame| self.frames > frame)
                {
                    input.events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(-500.0, 0.0),
                        modifiers: egui::Modifiers::NONE,
                    });
                    self.gfm_horizontal_sent = true;
                } else {
                    self.gfm_horizontal_hover_frame = Some(self.frames);
                }
            } else if self.frames % 4 == 0 {
                let pointer = if self.app.page == Page::Chat {
                    (viewport.width() > 1.0 && viewport.height() > 1.0).then_some(viewport.center())
                } else {
                    Some(
                        ctx.memory(|memory| {
                            memory.area_rect(egui::Id::new("notification-details-material-v1"))
                        })
                        .expect("synthetic GFM notification window")
                        .center(),
                    )
                };
                if let Some(pointer) = pointer {
                    input.events.push(egui::Event::PointerMoved(pointer));
                    input.events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -10.0),
                        modifiers: egui::Modifiers::NONE,
                    });
                }
            }
        }
        let Some(check) = &mut self.media_resize else {
            return;
        };
        // Exercise egui's actual edge-drag path in the isolated native capture.
        input.events.retain(|event| {
            !matches!(
                event,
                egui::Event::PointerMoved(_)
                    | egui::Event::PointerButton { .. }
                    | egui::Event::PointerGone
            )
        });
        if self.frames == 8 {
            let rect = ctx
                .memory(|memory| memory.area_rect(egui::Id::new("ファイル・音声・手書きを添付")))
                .expect("media dialog");
            check.initial_width = rect.width();
            check.start = rect.right_center() - egui::vec2(1.0, 0.0);
            input.events.push(egui::Event::PointerMoved(check.start));
        } else if self.frames == 9 || self.frames == 11 {
            let pressed = self.frames == 9;
            let pos =
                check.start - egui::vec2(if pressed { 0.0 } else { check.drag_distance }, 0.0);
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
        } else if self.frames == 10 {
            input.events.push(egui::Event::PointerMoved(
                check.start - egui::vec2(check.drag_distance, 0.0),
            ));
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(zoom) = self.qwen_zoom {
            if ctx.zoom_factor() != zoom {
                ctx.set_zoom_factor(zoom);
                ctx.request_repaint();
            }
        }
        // Exercise the real navigation after startup zoom/font layout settles.
        if self.qwen_navigate && self.frames == 3 {
            self.app.go_to_qwen_settings();
        }
        // Do not consume editor focus/scroll requests in the initial native
        // sizing viewport. The first real layout starts after zoom settles.
        if let Some((state, tail)) = &self.qwen_editor_preview {
            if self.frames == 3 {
                self.app.qwen_settings = Some(super::qwen_settings::synthetic_editor(
                    self.app.qwen_store.clone(),
                    state,
                ));
                self.app.qwen_settings_focus = false;
            }
            if *tail && (3..=8).contains(&self.frames) {
                if let Some(editor) = self.app.qwen_settings.as_mut() {
                    editor.preview_tail = true;
                }
            }
        }
        if self.speech {
            egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.app.status_summary(ui));
            self.app.speech_panel(
                ctx,
                &media::PlaybackSnapshot {
                    loaded: true,
                    can_seek: true,
                    position_seconds: 2.8,
                    duration_seconds: 12.0,
                    rate: 1.0,
                    playing: self.speech_playing,
                    ..Default::default()
                },
            );
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("読み上げ操作の表示確認（模擬状態・音声再生なし）");
            });
        } else if self.word_file_help {
            egui::CentralPanel::default().show(ctx, |ui| self.app.words_page(ui));
        } else {
            self.app.update_ui(ctx);
        }
        if self.notice_button_index < self.notice_buttons.len()
            && self.notice_button_position.is_none()
            && !self.notice_button_pressed
        {
            self.notice_button_position = visible_notice_button(
                ctx,
                self.notice_buttons[self.notice_button_index],
                self.material_context,
            );
        }
        if self.material_context
            && self.notice_finished_frame.is_some()
            && self.gfm_capture_target.is_none()
        {
            let marker = if self.material_context_tail {
                "固定回答の末尾。"
            } else {
                "明確な表現"
            };
            let visible = ctx
                .data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new("material-change-detail-viewport"))
                })
                .is_some_and(|viewport| {
                    visible_notice_glyphs_in_layer(
                        ctx,
                        marker,
                        viewport,
                        Some(egui::LayerId::new(
                            egui::Order::Middle,
                            egui::Id::new("教材の根拠と差分を確認"),
                        )),
                    )
                    .into_iter()
                    .all(|seen| seen)
                });
            if visible {
                self.gfm_marker_since.get_or_insert(self.frames);
            } else {
                self.gfm_marker_since = None;
            }
        }
        if self.notice_end_scroll {
            self.notice_tail_visible |= visible_notice_row(ctx, self.notice_tail_marker);
        }
        if self.gfm_table_right {
            let viewport = gfm_visible_area(ctx, &self.app);
            if self.frames >= 8 && self.gfm_table_pointer.is_none() {
                self.gfm_table_pointer = visible_gfm_header(ctx, viewport);
            }
            let visible_now = visible_notice_glyphs(ctx, "横スクロール終端②", viewport);
            let new_glyph = visible_now
                .iter()
                .zip(&self.gfm_table_seen)
                .any(|(visible, seen)| *visible && !*seen);
            if new_glyph {
                let since = *self.gfm_marker_since.get_or_insert(self.frames);
                if self.frames >= since + 3 && self.gfm_capture_target.is_none() {
                    let complete = visible_now
                        .iter()
                        .zip(&self.gfm_table_seen)
                        .all(|(visible, seen)| *visible || *seen);
                    let target = if complete {
                        self.output.clone()
                    } else {
                        self.output.with_file_name(format!(
                            "{}-part{}.png",
                            self.output.file_stem().unwrap().to_string_lossy(),
                            self.gfm_table_seen.iter().filter(|seen| **seen).count() + 1
                        ))
                    };
                    self.gfm_capture_mask = Some(visible_now);
                    self.gfm_capture_target = Some(target);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                }
            } else {
                self.gfm_marker_since = None;
            }
        }
        if let Some(check) = &mut self.media_resize {
            if self.frames >= 11 {
                let width = ctx
                    .memory(|memory| {
                        memory.area_rect(egui::Id::new("ファイル・音声・手書きを添付"))
                    })
                    .unwrap()
                    .width();
                let released = *check.released_width.get_or_insert(width);
                assert!(
                    released < check.initial_width - check.drag_distance * 0.5,
                    "native capture must actually shrink the dialog: {} -> {released}",
                    check.initial_width
                );
                assert!(
                    (width - released).abs() <= 1.0,
                    "dialog grew while idle: released={released}, frame={}, width={width}",
                    self.frames
                );
                if self.frames == 71 {
                    eprintln!(
                        "native resize verified: {} -> {released}, idle 60 frames, final {width}",
                        check.initial_width
                    );
                }
            }
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                let target = if self.gfm_table_right {
                    self.gfm_capture_target
                        .take()
                        .unwrap_or_else(|| self.output.clone())
                } else {
                    self.output.clone()
                };
                image::save_buffer(
                    &target,
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("save screenshot");
                if self.gfm_table_right {
                    if let Some(mask) = self.gfm_capture_mask.take() {
                        for (seen, visible) in self.gfm_table_seen.iter_mut().zip(mask) {
                            *seen |= visible;
                        }
                        self.gfm_table_reached = self.gfm_table_seen.iter().all(|seen| *seen);
                        self.gfm_marker_since = None;
                        if !self.gfm_table_reached {
                            continue;
                        }
                    }
                    assert!(
                        self.gfm_table_reached,
                        "native GFM preview did not reach the table's last column"
                    );
                }
                if self.palette_preview {
                    // Screenshot is already saved; discard only this isolated fixture's draft
                    // so the normal unsaved-palette close guard does not block the capture exit.
                    self.app.cancel_color_editor();
                }
                if self.media_check {
                    self.app.annotation = Default::default();
                    self.app.exit_media_requested = false;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.frames += 1;
        if self.material_context
            && self.gfm_capture_target.is_none()
            && self
                .gfm_marker_since
                .is_some_and(|since| self.frames >= since + 8)
        {
            self.gfm_capture_target = Some(self.output.clone());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        if self.frames
            == if self.material_context {
                512
            } else if self.media_resize.is_some() {
                72
            } else if self.gfm_table_right {
                512
            } else if self.material_scroll
                || self.notice_scroll
                || !self.notice_buttons.is_empty()
                || self.notice_end_scroll
            {
                24
            } else {
                8
            }
        {
            assert_eq!(
                self.notice_button_index,
                self.notice_buttons.len(),
                "native notice preview did not complete comparison/raw click"
            );
            if self.notice_end_scroll {
                assert!(
                    self.notice_tail_visible,
                    "native notice preview did not visibly reach fixed-source tail marker"
                );
            }
            if self.material_context {
                assert!(
                    self.gfm_capture_target.is_some(),
                    "native fixed conversation did not visibly reach its Markdown marker"
                );
            } else if !self.gfm_table_right {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
        }
        if self.gfm_table_right
            && self.frames >= 512
            && !self.gfm_table_reached
            && self.gfm_capture_target.is_none()
        {
            self.gfm_capture_target = Some(self.output.clone());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint();
    }
}
