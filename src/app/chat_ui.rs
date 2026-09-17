use super::*;
use wordweave5::chat_action::Operation;

impl WordApp {
    pub(super) fn chat_page(&mut self, ui: &mut egui::Ui) {
        let idle = self.pending.is_none()
            && !self.batch_running
            && self.session.is_none()
            && self.recorder.is_none()
            && self.fatal.is_none();
        let max_width = (ui.available_width() * 0.45).max(160.0);
        egui::SidePanel::left("chat-threads")
            .resizable(true)
            .default_width(240.0)
            .width_range(160.0..=max_width)
            .show_inside(ui, |ui| {
                ui.heading("英語チャット");
                if ui
                    .add_enabled(
                        idle && self.progress.chats.len() < wordweave5::chat::MAX_CHATS,
                        egui::Button::new("＋ 新しい会話"),
                    )
                    .clicked()
                {
                    self.progress
                        .chats
                        .push(wordweave5::chat::Conversation::new());
                    self.chat_selected = self.progress.chats.len() - 1;
                    self.dirty = true;
                    self.persist();
                }
                ui.small("ピン留め優先・最新の回答順");
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("thread-list")
                    .max_height((ui.available_height() - 65.0).max(60.0))
                    .show(ui, |ui| {
                        for index in wordweave5::chat::ordered_indices(&self.progress.chats) {
                            let chat = &mut self.progress.chats[index];
                            ui.push_id(&chat.id, |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_enabled(
                                            idle,
                                            egui::Button::new(if chat.pinned {
                                                "★"
                                            } else {
                                                "☆"
                                            }),
                                        )
                                        .on_hover_text("会話をピン留め／解除")
                                        .clicked()
                                    {
                                        chat.pinned = !chat.pinned;
                                        self.dirty = true;
                                    }
                                    if ui
                                        .add_enabled(
                                            idle,
                                            egui::Button::new(&chat.title)
                                                .selected(self.chat_selected == index)
                                                .wrap(),
                                        )
                                        .clicked()
                                    {
                                        self.chat_selected = index;
                                    }
                                });
                                if let Some(last) = chat.exchanges.last() {
                                    ui.small(
                                        chrono::DateTime::from_timestamp(last.at, 0)
                                            .map(|t| {
                                                t.with_timezone(&chrono::Local)
                                                    .format("%m/%d %H:%M")
                                                    .to_string()
                                            })
                                            .unwrap_or_default(),
                                    );
                                }
                            });
                            ui.separator();
                        }
                    });
                if ui.button("削除済みの教材").clicked() {
                    self.chat_trash_open = true;
                }
                ui.small("境界をドラッグして幅を変更");
                ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(chat) = self.progress.chats.get(self.chat_selected) else {
                ui.heading("英語について話してみよう");
                ui.label("左の「＋ 新しい会話」から始める。");
                ui.label("例：apologize for の使い方を教えて。");
                return;
            };
            let id = chat.id.clone();
            ui.horizontal_wrapped(|ui| {
                ui.heading(&chat.title);
                if ui.button("音声・手書き入力").clicked() { self.chat_media_open = true; }
                if ui.button("読み上げを停止").clicked() { self.speaker.stop(); }
                if ui.button("文脈・送信内容").clicked() { self.chat_context_open = true; }
                if ui.button(if self.progress.material_draft.is_some() { "教材案を確認" } else { "教材に反映" }).clicked() {
                    self.chat_material_open = true;
                }
            });
            let mut send = false;
            let mut speak = None;
            let mut preview = None;
            let mut play = None;
            let mut annotate = None;
            let max_height = (ui.available_height() * 0.55).max(130.0);
            egui::TopBottomPanel::bottom("chat-composer").resizable(false)
                .exact_height(self.chat_composer_height.clamp(130.0, max_height))
                .show_inside(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    let (_, grip) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 8.0), egui::Sense::drag());
                    let grip = grip.on_hover_cursor(egui::CursorIcon::ResizeVertical);
                    if grip.dragged() {
                        self.chat_composer_height = (self.chat_composer_height - grip.drag_delta().y)
                            .clamp(130.0, max_height);
                    }
                    let chat = &mut self.progress.chats[self.chat_selected];
                    let input_id = egui::Id::new(("chat-input", &id));
                    let focused = ui.memory(|m| m.has_focus(input_id));
                    let ime_id = input_id.with("ime");
                    let mut composing = ui.ctx().data_mut(|d| d.get_temp::<bool>(ime_id).unwrap_or(false));
                    let submit_key = ui.input_mut(|i| composer_keys(&mut i.events, focused, &mut composing));
                    ui.ctx().data_mut(|d| d.insert_temp(ime_id, composing));
                    let height = (ui.available_height() - 52.0).max(42.0);
                    egui::ScrollArea::vertical().id_salt(("composer-scroll", &id))
                        .max_height(height).show(ui, |ui| {
                        self.dirty |= ui.add_enabled_ui(idle, |ui| ui.add_sized(
                            [ui.available_width(), height], egui::TextEdit::multiline(&mut chat.draft)
                            .id(input_id)
                            .hint_text("英語の質問、または「この単語を整理して」「○○を新規登録／追加／削除して」")
                            .desired_width(f32::INFINITY).desired_rows(3).char_limit(4000))).inner.changed();
                    });
                    ui.horizontal(|ui| {
                        let full = chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES;
                        let ready = !chat.draft.trim().is_empty() && !full;
                        send = ui.add_enabled(idle && ready, egui::Button::new("送信 (Ctrl+Enter)")).clicked()
                            || (idle && ready && submit_key);
                        ui.small(format!("{} / 4,000文字", chat.draft.chars().count()));
                        if !chat.draft_attachments.is_empty() { ui.small(format!("添付{}件（音声・手書き入力で確認）",chat.draft_attachments.len())); }
                        if !idle { ui.spinner(); ui.label("処理中"); }
                        if full { ui.label("会話上限：新しい会話を作成"); }
                    });
                    ui.allocate_space(egui::vec2(ui.available_width(), 0.0));
                });
            egui::ScrollArea::vertical().id_salt(("chat-transcript", &id))
                .max_height((ui.available_height() - 8.0).max(0.0))
                .auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                let chat = &mut self.progress.chats[self.chat_selected];
                if chat.exchanges.is_empty() {
                    ui.add_space(24.0);
                    ui.heading("何を練習する？");
                    ui.label("下の入力欄へ質問を入力して「送信」を押す。");
                    ui.label("「この会話の単語を整理して」から、教材の登録につなげられる。");
                }
                for (index, exchange) in chat.exchanges.iter_mut().enumerate() {
                    ui.push_id(index, |ui| {
                        bubble(ui, "あなた", &exchange.question, true);
                        bubble(ui, "Codex", &exchange.answer, false);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("回答を読み上げ").clicked() { speak=Some(exchange.answer.clone()); }
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
                if self.pending.is_some() { ui.label("Codexが回答を準備している…"); }
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

    fn chat_windows(&mut self, ctx: &egui::Context, idle: bool) {
        let mut open = self.chat_context_open;
        egui::Window::new("文脈と送信内容").open(&mut open).default_width(640.0)
            .vscroll(true).show(ctx, |ui| {
            let Some(chat) = self.progress.chats.get_mut(self.chat_selected) else { return; };
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
                if self.progress.material_draft.is_none() {
                    ui.label("教材に使う往復だけを選び、対象語と反映方法を確認する。");
                    if let Some(chat) = self.progress.chats.get_mut(self.chat_selected) {
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
                    ui.horizontal(|ui| {
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
        egui::Window::new("チャットからの教材操作").collapsible(false).default_width(560.0)
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
            ui.horizontal(|ui| {
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
    let width = (ui.available_width() * 0.86).max(120.0);
    let layout = if user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };
    ui.with_layout(layout, |ui| {
        egui::Frame::new()
            .fill(if user {
                Color32::from_rgb(215, 237, 235)
            } else {
                Color32::from_rgb(242, 243, 245)
            })
            .inner_margin(12.0)
            .corner_radius(10.0)
            .show(ui, |ui| {
                ui.set_width((width - 24.0).max(80.0));
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.strong(speaker);
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
