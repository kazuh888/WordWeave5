//! Debug-only native capture of the real app using a new, synthetic store.
use super::*;

pub(crate) fn run() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let output = args
        .iter()
        .position(|s| s == "--ui-check")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .expect("--ui-check PNG_PATH");
    let small = args.iter().any(|s| s == "--small");
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
        || chat_media_preview_image || chat_media_preview_file || chat_media_resize
        || args.iter().any(|s| s == "--chat-media-table");
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
            .with_inner_size(if small {
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
            if chat_empty || chat_filled || chat_rename || chat_attachments || chat_consent
                || chat_drop_confirm || chat_media_entry || chat_media_ink || chat_media_exit
                || chat_media_table {
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
                            at: 0, date: date.clone(), grade: Grade::Good,
                            assisted: false, method: "keyboard".into(), first: offset == 5,
                            elapsed_days: 1.0, seconds: 60, self_assessed: true,
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
                    16, 12, image::Rgba([130, 193, 238, 255]),
                )).write_to(&mut png, image::ImageFormat::Png).unwrap();
                let image = store.put(wordweave5::assets::AssetKind::ImagePng, &png.into_inner()).unwrap();
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
                let audio = store.put(wordweave5::assets::AssetKind::AudioWav, &wav).unwrap();
                let file = store.put(wordweave5::assets::AssetKind::FileBlob,
                    "旅先での挨拶を練習する。\nPlease help me practice.".as_bytes()).unwrap();
                if chat_media_preview_image { app.preview_asset(&image); }
                if chat_media_preview_file { app.preview_file_asset(&file, "trip-notes.txt"); }
                let items = vec![
                    wordweave5::chat::Attachment { original: image.clone(), image: Some(image),
                        background: None, file_name: Some("airport.png".into()),
                        source_text: "airport.png".into(), transcript: None },
                    wordweave5::chat::Attachment { original: audio, image: None,
                        background: None, file_name: Some("greeting.wav".into()),
                        source_text: "greeting.wav".into(), transcript: None },
                    wordweave5::chat::Attachment { original: file, image: None,
                        background: None, file_name: Some("trip-notes.txt".into()),
                        source_text: "trip-notes.txt".into(), transcript: None },
                ];
                let chat = &mut app.progress.chats[0];
                chat.draft = "添付の内容を使って旅行英語を練習したい。".into();
                chat.draft_attachments = items;
                if chat_attachments {
                    chat.complete(chat.draft.clone(), "空港での挨拶から始めましょう。".into(),
                        wordweave5::execution::Execution::default()).unwrap();
                } else if chat_consent {
                    app.pending_chat_file_send = Some(ChatFileConsent {
                        chat_id: chat.id.clone(), question: chat.draft.clone(),
                        attachment_ids: chat.draft_attachments.iter().map(|a| a.original.id.clone()).collect(),
                        attachment_names: chat.draft_attachments.iter().map(|a| a.file_name.clone()).collect(),
                        labels: vec!["画像：airport.png".into(), "音声：greeting.wav".into(),
                            "ファイル：trip-notes.txt".into()],
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
                app.last_notification_message = app.message.clone();
            }
            if path_guidance {
                app.open_codex_path_guidance();
            }
            if args.iter().any(|s| s == "--notice") {
                app.notification_open = true;
            }
            cc.egui_ctx.set_zoom_factor(if small { 1.6 } else { 0.8 });
            Ok(Box::new(Capture {
                app,
                output,
                frames: 0,
                speech: args.iter().any(|s| s == "--speech"),
                speech_playing: args.iter().any(|s| s == "--speech-selected"),
                word_file_help: args.iter().any(|s| s == "--word-file-help"),
                media_check: chat_media_entry || chat_media_ink || chat_media_exit || chat_media_table,
                media_resize: chat_media_resize.then_some(MediaResizeCheck {
                    drag_distance: if small { 80.0 } else { 300.0 },
                    start: egui::Pos2::ZERO, initial_width: 0.0, released_width: None,
                }),
            }))
        }),
    )
}

struct Capture {
    app: WordApp,
    output: PathBuf,
    frames: usize,
    speech: bool,
    speech_playing: bool,
    word_file_help: bool,
    media_check: bool,
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
        let Some(check) = &mut self.media_resize else { return };
        // Exercise egui's actual edge-drag path in the isolated native capture.
        input.events.retain(|event| !matches!(event, egui::Event::PointerMoved(_)
            | egui::Event::PointerButton { .. } | egui::Event::PointerGone));
        if self.frames == 8 {
            let rect = ctx.memory(|memory| memory.area_rect(
                egui::Id::new("ファイル・音声・手書きを添付"))).expect("media dialog");
            check.initial_width = rect.width();
            check.start = rect.right_center() - egui::vec2(1.0, 0.0);
            input.events.push(egui::Event::PointerMoved(check.start));
        } else if self.frames == 9 || self.frames == 11 {
            let pressed = self.frames == 9;
            let pos = check.start - egui::vec2(if pressed { 0.0 } else { check.drag_distance }, 0.0);
            input.events.push(egui::Event::PointerButton {
                pos, button: egui::PointerButton::Primary, pressed,
                modifiers: egui::Modifiers::NONE,
            });
        } else if self.frames == 10 {
            input.events.push(egui::Event::PointerMoved(
                check.start - egui::vec2(check.drag_distance, 0.0)));
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        if let Some(check) = &mut self.media_resize {
            if self.frames >= 11 {
                let width = ctx.memory(|memory| memory.area_rect(
                    egui::Id::new("ファイル・音声・手書きを添付"))).unwrap().width();
                let released = *check.released_width.get_or_insert(width);
                assert!(released < check.initial_width - check.drag_distance * 0.5,
                    "native capture must actually shrink the dialog: {} -> {released}", check.initial_width);
                assert!((width - released).abs() <= 1.0,
                    "dialog grew while idle: released={released}, frame={}, width={width}", self.frames);
                if self.frames == 71 {
                    eprintln!("native resize verified: {} -> {released}, idle 60 frames, final {width}",
                        check.initial_width);
                }
            }
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    &self.output,
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("save screenshot");
                if self.media_check {
                    self.app.annotation = Default::default();
                    self.app.exit_media_requested = false;
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.frames += 1;
        if self.frames == if self.media_resize.is_some() { 72 } else { 8 } {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        ctx.request_repaint();
    }
}
