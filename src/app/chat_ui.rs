use super::*;
use wordweave5::chat_action::Operation;

impl WordApp {
    pub(super) fn chat_page(&mut self, ui: &mut egui::Ui) {
        let idle = self.pending.is_none()
            && !self.batch_running
            && self.session.is_none()
            && self.recorder.is_none()
            && self.fatal.is_none();
        let activity = self.activity_label();
        if ui.available_width() < 700.0 {
            ui.horizontal_wrapped(|ui| {
                ui.strong("英語チャット");
                ui.menu_button("会話を選ぶ", |ui| {
                    ui.set_width(280.0);
                    self.chat_thread_picker(ui, idle, true);
                });
            });
        } else {
            let max_width = ui.available_width() * 0.40;
            egui::SidePanel::left("chat-threads")
                .resizable(true)
                .default_width(240.0)
                .width_range(176.0..=max_width)
                .show_inside(ui, |ui| {
                    ui.heading("英語チャット");
                    self.chat_thread_picker(ui, idle, false);
                });
        }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(chat) = self.progress.chats.get(self.chat_selected).filter(|c| c.deleted_at.is_none()) else {
                ui.heading("英語について話してみよう");
                ui.label("会話一覧の「＋ 新しい会話」から始める。");
                ui.label("例：apologize for の使い方を教えて。");
                return;
            };
            let id = chat.id.clone();
            let title = chat.title.clone();
            ui.add(egui::Label::new(RichText::new(&title).size(24.0).strong().color(ux::INK))
                .truncate()).on_hover_text(&title);
            ui.horizontal_wrapped(|ui| {
                if ui.button(if self.progress.material_draft.is_some() { "教材案を確認" } else { "教材案を作る" })
                    .on_hover_text("根拠の往復を選択 → 教材案を作成 → 差分を確認して登録。確認前に教材は変更しない。")
                    .clicked() {
                    self.chat_material_open = true;
                }
                ui.menu_button("会話の操作", |ui| {
                    if ui.button("文脈・送信内容を確認").clicked() {
                        self.chat_context_open = true;
                        ui.close_menu();
                    }
                    if ui.button("実行記録・結果の再取得").clicked() {
                        self.run_history_open = true;
                        self.refresh_runs();
                        ui.close_menu();
                    }
                    if ui.button("読み上げを停止").clicked() {
                        self.stop_speech();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.add_enabled(idle, egui::Button::new("会話を削除…")).clicked() {
                        self.pending_chat_delete = Some(id.clone());
                        ui.close_menu();
                    }
                });
            });
            let mut send = false;
            let mut speak = None;
            let mut preview = None;
            let mut play = None;
            let mut annotate = None;
            let max_height = (ui.available_height() * 0.60).max(144.0);
            egui::TopBottomPanel::bottom("chat-composer").resizable(false).show_separator_line(false)
                .exact_height(self.chat_composer_height.clamp(144.0, max_height))
                .show_inside(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    let (_, grip) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 8.0), egui::Sense::drag());
                    let grip = grip.on_hover_cursor(egui::CursorIcon::ResizeVertical);
                    if grip.dragged() {
                        self.chat_composer_height = (self.chat_composer_height - grip.drag_delta().y)
                            .clamp(144.0, max_height);
                    }
                    let chat = &mut self.progress.chats[self.chat_selected];
                    let input_id = egui::Id::new(("chat-input", &id));
                    let focused = ui.memory(|m| m.has_focus(input_id));
                    let ime_id = input_id.with("ime");
                    let mut composing = ui.ctx().data_mut(|d| d.get_temp::<bool>(ime_id).unwrap_or(false));
                    let submit_key = ui.input_mut(|i| composer_keys(&mut i.events, focused, &mut composing));
                    ui.ctx().data_mut(|d| d.insert_temp(ime_id, composing));
                    let height = (ui.available_height() - 76.0).max(36.0);
                    egui::ScrollArea::vertical().id_salt(("composer-scroll", &id))
                        .min_scrolled_height(0.0).max_height(height).show(ui, |ui| {
                        self.dirty |= ui.add_enabled_ui(idle, |ui| ui.add_sized(
                            [ui.available_width(), height], egui::TextEdit::multiline(&mut chat.draft)
                            .id(input_id)
                            .hint_text("英語の質問、または「この単語を整理して」「○○を新規登録／追加／削除して」")
                            .desired_width(f32::INFINITY).desired_rows(1).char_limit(4000))).inner.changed();
                    });
                    ui.horizontal_wrapped(|ui| {
                        let full = chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES;
                        let ready = !chat.draft.trim().is_empty() && !full;
                        send = ui.add_enabled(idle && ready, egui::Button::new(
                            RichText::new("送信 (Ctrl+Enter)").strong().color(Color32::WHITE))
                            .fill(ux::ACCENT).min_size(egui::vec2(170.0, 36.0)).wrap()).clicked()
                            || (idle && ready && submit_key);
                        if ui.button("音声・手書き")
                            .on_hover_text("音声入力、手書き入力、保存した添付を確認する。")
                            .clicked() {
                            self.chat_media_open = true;
                        }
                    });
                    ui.small(format!("{} / 4,000文字 · Enterで改行 · 添付{}件",
                        chat.draft.chars().count(), chat.draft_attachments.len()));
                    ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                });
            egui::ScrollArea::vertical().id_salt(("chat-transcript", &id))
                .min_scrolled_height(0.0)
                .max_height((ui.available_height() - 8.0).max(0.0))
                .auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                let chat = &mut self.progress.chats[self.chat_selected];
                if chat.exchanges.is_empty() {
                    ui.add_space(16.0);
                    ui.heading("英語について話してみよう");
                    ui.label("質問や練習したいことを下の入力欄へ。日本語でも質問できる。");
                    ui.add_space(8.0);
                    for example in ["sorry と apologize の使い分けを教えて。",
                        "自己紹介を練習したい。英語で質問してください。"] {
                        if ui.add_enabled(idle && chat.draft.is_empty(),
                            egui::Button::new(example).wrap())
                            .on_hover_text("入力欄に入れる。送信前に編集できる。")
                            .clicked() {
                            chat.draft = example.into();
                            self.dirty = true;
                            ui.memory_mut(|m| m.request_focus(egui::Id::new(("chat-input", &id))));
                        }
                    }
                    ui.add_space(8.0);
                    ui.small("会話の単語は、根拠を選んで教材案にできる。差分を確認してから登録する。");
                }
                for (index, exchange) in chat.exchanges.iter_mut().enumerate() {
                    ui.push_id(index, |ui| {
                        bubble(ui, "あなた", &exchange.question, true);
                        bubble(ui, "Codex", &exchange.answer, false);
                        ui.horizontal_wrapped(|ui| {
                            if ui.add_enabled(self.recorder.is_none(), egui::Button::new("回答を読み上げ")).clicked() { speak=Some(exchange.answer.clone()); }
                            if ui.button("英文に注釈を書く").clicked() { annotate=Some(exchange.answer.clone()); }
                            for a in &exchange.attachments {
                                if a.original.kind==wordweave5::assets::AssetKind::AudioWav && ui.add_enabled(idle,egui::Button::new("原録音を再生")).clicked() {play=Some(a.original.clone());}
                                if let Some(image)=&a.image {if ui.button("注釈画像").clicked(){preview=Some(image.clone());}}
                            }
                        });
                        ui.small(exchange.execution.label());
                        ui.horizontal_wrapped(|ui| {
                            self.dirty |= ui.add_enabled(idle, egui::Checkbox::new(&mut exchange.pinned,
                                "文脈に必ず含める")).changed();
                            self.dirty |= ui.add_enabled(idle, egui::Checkbox::new(&mut exchange.for_material,
                                "教材の根拠にする")).changed();
                        });
                        ui.add_space(16.0);
                    });
                }
                if !idle {
                    ui.horizontal_wrapped(|ui| {
                        if self.pending.is_some() || self.batch_running { ui.spinner(); }
                        ui.label(activity);
                    });
                }
                if chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES {
                    ui.label("この会話は保存上限に達した。引き継ぎメモを確認し、新しい会話を作成する。");
                }
            });
            if send { self.launch_chat(); }
            if let Some(text)=speak {self.say(&text);}
            if let Some(reference)=preview {self.preview_asset(&reference);}
            if let Some(reference)=play {self.play_asset(&reference);}
            if let Some(text)=annotate {
                if self.annotation.frozen() {self.message="作成中の注釈がある。先に添付または破棄する。".into();}
                else {self.annotation.text=text;}
                self.chat_media_open=true;
            }
        });
        self.chat_windows(ui.ctx(), idle);
    }

    fn chat_thread_picker(&mut self, ui: &mut egui::Ui, idle: bool, popup: bool) {
        let mut close = false;
        let can_create = self.progress.chats.iter().filter(|c| c.deleted_at.is_none()).count()
            < wordweave5::chat::MAX_CHATS;
        if ui.add_enabled(idle && can_create, egui::Button::new("＋ 新しい会話").wrap()).clicked() {
            match self.create_conversation() {
                Ok(()) => close = true,
                Err(error) => self.message = error,
            }
        }
        ui.small("ピン留め優先・最新の回答順");
        ui.separator();
        let height = if popup { 240.0 } else { (ui.available_height() - 100.0).max(60.0) };
        egui::ScrollArea::vertical().id_salt("thread-list").max_height(height).show(ui, |ui| {
            for index in wordweave5::chat::ordered_indices(&self.progress.chats) {
                let chat = &mut self.progress.chats[index];
                ui.push_id(&chat.id, |ui| {
                    ui.horizontal(|ui| {
                        if ui.add_enabled(idle, egui::Button::new(if chat.pinned { "★" } else { "☆" }))
                            .on_hover_text("会話をピン留め／解除").clicked() {
                            chat.pinned = !chat.pinned;
                            self.dirty = true;
                        }
                        if ui.add_enabled(idle, egui::Button::new(&chat.title)
                            .selected(self.chat_selected == index).wrap()).clicked() {
                            self.chat_selected = index;
                            close = true;
                        }
                    });
                    if let Some(last) = chat.exchanges.last() {
                        ui.small(chrono::DateTime::from_timestamp(last.at, 0).map(|t|
                            t.with_timezone(&chrono::Local).format("%m/%d %H:%M").to_string())
                            .unwrap_or_default());
                    } else { ui.small("まだメッセージはない"); }
                });
                ui.separator();
            }
        });
        if ui.button("削除済みの教材").clicked() {
            self.chat_trash_open = true;
            close = true;
        }
        if ui.button("チャットのごみ箱").clicked() {
            self.conversation_trash_open = true;
            close = true;
        }
        if !popup { ui.small("境界をドラッグして幅を変更"); }
        if popup && close { ui.close_menu(); }
    }

    fn chat_windows(&mut self, ctx: &egui::Context, idle: bool) {
        let mut open = self.chat_context_open;
        egui::Window::new("文脈と送信内容").open(&mut open).default_width(640.0)
            .vscroll(true).show(ctx, |ui| {
            let Some(chat) = self.progress.chats.get_mut(self.chat_selected).filter(|c| c.deleted_at.is_none()) else { return; };
            ui.label("引き継ぎメモ：学習目的・重要な訂正（毎回送信）");
            self.dirty |= ui.add_enabled(idle, egui::TextEdit::multiline(&mut chat.memo)
                .desired_width(f32::INFINITY).char_limit(2000)).changed();
            ui.label("古い発言も保存される。「文脈に必ず含める」は会話のピン留めとは別の指定である。");
            match wordweave5::chat::prepare_with_catalog(chat, &self.deck, &self.progress.deleted_entries) {
                Ok(context) => {
                    ui.label(format!("送信 {} 往復 / 対象外 {} 往復 / {} bytes",
                        context.included, context.omitted, context.bytes));
                    let mut preview = context.preview();
                    ui.add(egui::TextEdit::multiline(&mut preview).desired_width(f32::INFINITY)
                        .desired_rows(15).interactive(false));
                }
                Err(error) => { ui.label(error); }
            }
        });
        self.chat_context_open = open;
        let mut open = self.chat_material_open;
        egui::Window::new("教材の根拠と差分を確認")
            .open(&mut open)
            .default_width(1100.0)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label(RichText::new("1 根拠を選ぶ → 2 教材案を作る → 3 差分を確認して登録")
                    .strong().color(ux::ACCENT));
                ui.small("登録を確認するまで教材は変更しない。追加と訂正は区別して確認する。");
                ui.separator();
                if self.progress.material_draft.is_none() {
                    ui.label("教材に使う往復だけを選び、対象語と反映方法を確認する。");
                    if let Some(chat) = self.progress.chats.get_mut(self.chat_selected).filter(|c| c.deleted_at.is_none()) {
                        for (i, e) in chat.exchanges.iter_mut().enumerate() {
                            self.dirty |= ui
                                .add_enabled(
                                    idle,
                                    egui::Checkbox::new(
                                        &mut e.for_material,
                                        format!(
                                            "{}：{}",
                                            i + 1,
                                            e.question.chars().take(60).collect::<String>()
                                        ),
                                    ),
                                )
                                .changed();
                        }
                    }
                }
                self.material_panel(ui);
            });
        self.chat_material_open = open;
        self.chat_action_window(ctx, idle);
        let mut open = self.chat_trash_open;
        egui::Window::new("削除済みの教材")
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label("削除した教材と復習記録を保持している。復元すると再び一覧と出題に戻る。");
                let entries: Vec<_> = self
                    .deck
                    .iter()
                    .filter(|e| self.progress.deleted_entries.contains(&e.id))
                    .map(|e| (e.id.clone(), e.base.clone(), e.meaning.clone()))
                    .collect();
                if entries.is_empty() {
                    ui.label("削除済みの教材はない。");
                }
                for (id, base, meaning) in entries {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{base} / {meaning} [{id}]"));
                        if ui.add_enabled(idle, egui::Button::new("復元")).clicked() {
                            self.set_chat_entry_deleted(&id, false);
                        }
                    });
                }
            });
        self.chat_trash_open = open;
    }

    fn chat_action_window(&mut self, ctx: &egui::Context, idle: bool) {
        let Some((conversation_id, action)) = self.progress.chat_action.clone() else {
            return;
        };
        let mut dismiss = false;
        let mut proceed = false;
        egui::Window::new("チャットからの教材操作").collapsible(false).default_width(560.0).vscroll(true)
            .show(ctx, |ui| {
            let label = match action.operation { Operation::New => "新規登録", Operation::Append => "追加",
                Operation::Delete => "削除", Operation::Organize => "整理" };
            ui.heading(format!("{} を{}", action.base, label));
            ui.label("まだ教材は変更していない。対象を確認して進める。");
            if action.operation != Operation::New {
                let matches: Vec<_> = self.deck.iter().filter(|e|
                    !self.progress.deleted_entries.contains(&e.id)
                    && model::normalize(&e.base) == model::normalize(&action.base)).collect();
                if matches.len() == 1 && self.chat_target.is_empty() { self.chat_target = matches[0].id.clone(); }
                for e in &matches {
                    ui.radio_value(&mut self.chat_target, e.id.clone(), format!("{} / {} [{}]", e.base, e.meaning, e.id));
                }
                if matches.is_empty() { ui.label("登録済み教材が見つからない。対象語を指定して質問し直す。"); }
            }
            if action.operation == Operation::Delete {
                ui.label("一覧・学習対象から削除する。教材と復習記録は「削除済みの教材」から復元できる。");
            } else {
                ui.label("次の画面で根拠の往復を選び、教材案を生成する。生成後の差分を確認して登録する。");
                if self.progress.material_draft.is_some() {
                    ui.label("未登録の教材案がある。先に「教材案を確認」で登録または破棄する。");
                }
            }
            ui.horizontal_wrapped(|ui| {
                proceed = ui.add_enabled(idle && (action.operation == Operation::Delete || self.progress.material_draft.is_none())
                    && (action.operation == Operation::New || !self.chat_target.is_empty()),
                    egui::Button::new(if action.operation == Operation::Delete { "対象を確認して削除" } else { "根拠と教材案を確認する" })).clicked();
                dismiss = ui.button("キャンセル").clicked();
            });
        });
        if proceed {
            if action.operation == Operation::Delete {
                // Revalidate after selection; never use a model-supplied identifier directly.
                if self.deck.iter().any(|e| {
                    e.id == self.chat_target
                        && model::normalize(&e.base) == model::normalize(&action.base)
                }) {
                    let target = self.chat_target.clone();
                    if !self.set_chat_entry_deleted(&target, true) {
                        return;
                    }
                }
            } else if let Some(index) = self
                .progress
                .chats
                .iter()
                .position(|c| c.id == conversation_id)
            {
                self.chat_selected = index;
                self.material_base = action.base;
                self.material_target = self.chat_target.clone();
                self.material_mode = if action.operation == Operation::New {
                    wordweave5::material::Mode::New
                } else {
                    wordweave5::material::Mode::Append
                };
                self.chat_material_open = true;
            }
            dismiss = true;
        }
        if dismiss {
            self.progress.chat_action = None;
            self.chat_target.clear();
            self.dirty = true;
            self.persist();
        }
    }

    fn set_chat_entry_deleted(&mut self, id: &str, deleted: bool) -> bool {
        let mut next = self.progress.clone();
        if deleted {
            next.deleted_entries.insert(id.to_owned());
        } else {
            next.deleted_entries.remove(id);
        }
        let result = self
            .storage
            .as_ref()
            .ok_or_else(|| "保存先がない。".to_owned())
            .and_then(|storage| storage.save(&next));
        match result {
            Ok(()) => {
                self.progress = next;
                self.dirty = false;
                self.last_save = Instant::now();
                self.message = if deleted {
                    "教材を削除済みに移動した。復習記録は保持している。"
                } else {
                    "教材を復元した。復習記録は保持している。"
                }
                .into();
                true
            }
            Err(error) => {
                self.message = format!("保存に失敗したため削除・復元を反映していない：{error}");
                false
            }
        }
    }
}

fn bubble(ui: &mut egui::Ui, speaker: &str, text: &str, user: bool) {
    let width = (ui.available_width() * 0.90).min(820.0);
    let layout = if user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };
    ui.with_layout(layout, |ui| {
        egui::Frame::new()
            .fill(if user {
                ux::TINT
            } else {
                Color32::from_rgb(247, 249, 251)
            })
            .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
            .inner_margin(12.0)
            .corner_radius(10.0)
            .show(ui, |ui| {
                ui.set_width((width - 24.0).max(0.0));
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new(speaker).strong().color(ux::INK));
                    ui.add(egui::Label::new(text).wrap().selectable(true));
                });
            });
    });
}

// A conversion-confirmation frame must not also submit the draft.
fn composer_keys(events: &mut Vec<egui::Event>, focused: bool, composing: &mut bool) -> bool {
    if !focused { *composing = false; return false; }
    let mut ime_frame = *composing;
    for event in events.iter() {
        if let egui::Event::Ime(event) = event {
            ime_frame = true;
            *composing = matches!(event, egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_));
        }
    }
    let submit = !ime_frame && events.iter().any(|event| matches!(event,
        egui::Event::Key { key: egui::Key::Enter, pressed: true, repeat: false, modifiers, .. }
            if modifiers.ctrl && !modifiers.shift && !modifiers.alt && !modifiers.mac_cmd));
    // Leave IME commit/preedit events intact. Only the confirmation/shortcut key
    // is withheld from TextEdit; a normal Enter still inserts a newline.
    events.retain(|event| !matches!(event,
        egui::Event::Key { key: egui::Key::Enter, modifiers, .. } if ime_frame || modifiers.ctrl));
    submit
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn narrow_chat_keeps_composer_and_approval_workflow_without_changing_draft() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.chats[0].draft = "保存する下書き".into();
        let before = serde_json::to_vec(&app.progress).unwrap();
        let mut text = String::new();
        for _ in 0..2 {
            let output = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.0, 420.0))),
                ..Default::default()
            }, |ctx| { egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui)); });
            text.clear();
            for shape in output.shapes { super::super::harness_tests::shape_text(&shape.shape, &mut text); }
        }
        assert!(text.contains("会話を選ぶ"), "{text}");
        assert!(text.contains("送信 (Ctrl+Enter)"), "{text}");
        assert!(text.contains("教材案を作る"), "{text}");
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn audio_playback_is_not_presented_as_a_codex_response() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let (_tx, rx) = std::sync::mpsc::channel();
        app.pending = Some(Pending { key: String::new(), rx, cancel: None,
            kind: activity::Activity::Playback });
        let output = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            ..Default::default()
        }, |ctx| { egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui)); });
        let mut text = String::new();
        for shape in output.shapes { super::super::harness_tests::shape_text(&shape.shape, &mut text); }
        assert!(text.contains("保存した音声を再生中"), "{text}");
        assert!(!text.contains("Codexが回答"), "{text}");
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn short_chat_keeps_send_button_inside_its_clip_rect() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.chats[0].draft = "送信前の下書き\n".repeat(20);
        app.chat_composer_height = 144.0;
        let mut send_visible = false;
        for _ in 0..3 {
            let output = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.0, 300.0))),
                ..Default::default()
            }, |ctx| { egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui)); });
            send_visible = output.shapes.iter().any(|shape| {
                if let egui::epaint::Shape::Text(text) = &shape.shape {
                    text.galley.text() == "送信 (Ctrl+Enter)"
                        && shape.clip_rect.contains(text.pos)
                        && shape.clip_rect.contains(text.pos + text.galley.size())
                } else { false }
            });
        }
        assert!(send_visible, "Send must remain fully visible with a long draft and a short composer");
        assert_eq!(app.progress.chats[0].draft, "送信前の下書き\n".repeat(20));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn enter(ctrl: bool) -> egui::Event {
        egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true,
            repeat: false, modifiers: if ctrl { egui::Modifiers::CTRL } else { egui::Modifiers::NONE } }
    }
    #[test]
    fn only_focused_ctrl_enter_submits() {
        assert!(!composer_keys(&mut vec![enter(false)], true, &mut false));
        assert!(!composer_keys(&mut vec![enter(true)], false, &mut false));
        let mut events = vec![enter(true)];
        assert!(composer_keys(&mut events, true, &mut false));
        assert!(events.is_empty());
    }
    #[test]
    fn ime_confirmation_does_not_submit() {
        let mut composing = false;
        assert!(!composer_keys(&mut vec![egui::Event::Ime(egui::ImeEvent::Preedit("日本語".into()))], true, &mut composing));
        assert!(!composer_keys(&mut vec![enter(true)], true, &mut composing));
        let mut events = vec![egui::Event::Ime(egui::ImeEvent::Commit("日本語".into())), enter(false)];
        assert!(!composer_keys(&mut events, true, &mut composing));
        assert!(matches!(events.as_slice(), [egui::Event::Ime(egui::ImeEvent::Commit(_))]));
        assert!(composer_keys(&mut vec![enter(true)], true, &mut composing));
    }
    #[test]
    fn plain_enter_inserts_newline_but_ime_confirmation_does_not() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("input-test");
        let mut text = "日本語".to_owned();
        let mut composing = false;
        for (events, expected_newlines) in [(vec![], 0),
            (vec![egui::Event::Ime(egui::ImeEvent::Commit("漢字".into())), enter(false)], 0),
            (vec![enter(false)], 1)] {
            let _ = ctx.run(egui::RawInput { events, ..Default::default() }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.memory_mut(|m| m.request_focus(id));
                    ui.input_mut(|i| { composer_keys(&mut i.events, true, &mut composing); });
                    ui.add(egui::TextEdit::multiline(&mut text).id(id));
                });
            });
            assert_eq!(text.matches('\n').count(), expected_newlines);
        }
    }
}
