use super::*;
use wordweave5::chat_action::Operation;

impl WordApp {
    pub(super) fn chat_page(&mut self, ui: &mut egui::Ui) {
        let previous_style = (*ui.style()).clone();
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 10.0);
        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
        for (style, size) in [
            (egui::TextStyle::Body, 19.0),
            (egui::TextStyle::Small, 15.0),
            (egui::TextStyle::Button, 18.0),
        ] {
            ui.style_mut()
                .text_styles
                .insert(style, home_art::home_font(size));
        }
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(28.0, egui::FontFamily::Name("heading".into())),
        );
        ui.style_mut().visuals.override_text_color = Some(home_art::INK);
        let idle = self.pending.is_none()
            && !self.batch_running
            && self.session.is_none()
            && self.recorder.is_none()
            && self.fatal.is_none();
        let activity = self.activity_label();
        let short_page = ui.available_height() < 430.0;
        if ui.available_width() < 820.0 {
            if !short_page {
            egui::Frame::new()
                .fill(Color32::WHITE)
                .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                .corner_radius(10)
                .inner_margin(if short_page { 6 } else { 10 })
                .show(ui, |ui| {
                    if short_page {
                        ui.spacing_mut().button_padding = egui::vec2(9.0, 5.0);
                    }
                    ui.horizontal(|ui| {
                        home_art::badge(
                            ui,
                            home_art::Icon::Chat,
                            home_art::BLUE,
                            if short_page { 30.0 } else { 40.0 },
                        );
                        ui.vertical(|ui| {
                            home_art::title(
                                ui,
                                "英語チャット",
                                if short_page { 20.0 } else { 24.0 },
                            );
                            if !short_page {
                                ui.label(
                                    RichText::new("質問・会話練習・教材づくり")
                                        .size(15.0)
                                        .color(home_art::MUTED),
                                );
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.ww_menu_button("会話を選ぶ", |ui| {
                                ui.set_width(340.0);
                                self.chat_thread_picker(ui, idle, true);
                            });
                        });
                    });
                });
            ui.add_space(10.0);
            }
        } else {
            let max_width = (ui.available_width() * 0.42).max(280.0);
            egui::SidePanel::left("chat-threads")
                .resizable(true)
                .default_width(292.0)
                .width_range(248.0..=max_width)
                .frame(
                    egui::Frame::new()
                        .fill(Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                        .corner_radius(12)
                        .inner_margin(14),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        home_art::badge(ui, home_art::Icon::Chat, home_art::BLUE, 44.0);
                        ui.vertical(|ui| {
                            home_art::title(ui, "英語チャット", 25.0);
                            ui.label(
                                RichText::new("英語で話して、学んで、表現を広げる。")
                                    .size(14.0)
                                    .color(home_art::MUTED),
                            );
                        });
                    });
                    ui.add_space(8.0);
                    self.chat_thread_picker(ui, idle, false);
                });
        }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(chat) = self.progress.chats.get(self.chat_selected).filter(|c| c.deleted_at.is_none()) else {
                egui::Frame::new().fill(Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                    .corner_radius(12).inner_margin(24).show(ui, |ui| {
                    ui.set_min_height((ui.available_height() - 48.0).max(260.0));
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        home_art::badge(ui, home_art::Icon::Chat, home_art::BLUE, 82.0);
                        ui.add_space(12.0);
                        home_art::title(ui, "英語で話してみましょう", 32.0);
                        ui.label("質問、会話練習、表現の確認から始められます。");
                        ui.label(RichText::new("教材案は、差分を確認して承認するまで教材を変更しません。")
                            .size(15.0).color(home_art::MUTED));
                        ui.add_space(18.0);
                        if ui.add_enabled_ui(idle, |ui| home_art::primary(ui, "＋  新しい会話", 300.0))
                            .inner.clicked() {
                            if let Err(error) = self.create_conversation() { self.notify_error(error); }
                        }
                    });
                });
                return;
            };
            let id = chat.id.clone();
            let title = chat.title.clone();
            let mut send = false;
            let mut speak = None;
            let mut opened_attachment = None;
            let mut annotate = None;
            let mut copied = None;
            let compact_vertical = ui.available_height() < 430.0;
            let composer_min = if compact_vertical { 112.0 } else { 158.0 };
            let max_height = (ui.available_height() * 0.60).max(composer_min);
            let composer_height = if compact_vertical { composer_min }
                else { self.chat_composer_height.clamp(composer_min, max_height) };
            let mut drop_rect = None;
            egui::TopBottomPanel::bottom("chat-composer").resizable(false).show_separator_line(false)
                .exact_height(composer_height)
                .show_inside(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    let (_, grip) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), if compact_vertical { 4.0 } else { 8.0 }),
                        if compact_vertical { egui::Sense::hover() } else { egui::Sense::drag() });
                    let grip = grip.on_hover_cursor(egui::CursorIcon::ResizeVertical);
                    if !compact_vertical && grip.dragged() {
                        self.chat_composer_height = (self.chat_composer_height - grip.drag_delta().y)
                            .clamp(composer_min, max_height);
                    }
                    egui::Frame::new().fill(Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                        .corner_radius(12).inner_margin(if compact_vertical { 6 } else { 10 }).show(ui, |ui| {
                        if compact_vertical { ui.spacing_mut().item_spacing.y = 6.0; }
                        let chat = &mut self.progress.chats[self.chat_selected];
                        let input_id = egui::Id::new(("chat-input", &id));
                        let focused = ui.memory(|m| m.has_focus(input_id));
                        let ime_id = input_id.with("ime");
                        let mut composing = ui.ctx().data_mut(|d| d.get_temp::<bool>(ime_id).unwrap_or(false));
                        let submit_key = ui.input_mut(|i| composer_keys(&mut i.events, focused, &mut composing));
                        ui.ctx().data_mut(|d| d.insert_temp(ime_id, composing));
                        let height = (ui.available_height() - if compact_vertical { 44.0 } else { 72.0 }).max(34.0);
                        egui::ScrollArea::vertical().id_salt(("composer-scroll", &id))
                            .min_scrolled_height(0.0).max_height(height).show(ui, |ui| {
                            let response = ui.add_enabled_ui(idle, |ui| ui.add_sized(
                                [ui.available_width(), height], egui::TextEdit::multiline(&mut chat.draft)
                                .id(input_id)
                                .hint_text("英語の質問、会話練習、または教材にしたい表現を入力")
                                .desired_width(f32::INFINITY).desired_rows(1).char_limit(4000))).inner;
                            let visible_rect = response.rect.intersect(ui.clip_rect());
                            drop_rect = Some(visible_rect);
                            self.dirty |= response.changed();
                        });
                        let full = chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES;
                        let ready = !chat.draft.trim().is_empty() && !full;
                        ui.horizontal(|ui| {
                            let button_width: f32 = if compact_vertical { 164.0 } else { 188.0 };
                            let button_height: f32 = if compact_vertical { 36.0 } else { 42.0 };
                            if ui.add_sized([button_width.min(ui.available_width()), button_height],
                                crate::app::controls::Button::new("音声・手書き・添付"))
                                .on_hover_text("音声入力、手書き入力、保存した添付を確認する。")
                                .clicked() { self.chat_media_open = true; }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                send = ui.add_enabled(idle && ready, crate::app::controls::Button::new(
                                    RichText::new("送信 (Ctrl+Enter)").strong().color(Color32::WHITE))
                                    .fill(home_art::BLUE).corner_radius(8)
                                    .min_size(egui::vec2(button_width, button_height)).wrap()).clicked()
                                    || (idle && ready && submit_key);
                            });
                        });
                        if !compact_vertical {
                            ui.label(RichText::new(format!("{} / 4,000文字  ·  Enterで改行  ·  添付{}件  ·  ファイルをドロップして添付",
                                chat.draft.chars().count(), chat.draft_attachments.len()))
                                .size(14.0).color(home_art::MUTED));
                        }
                    });
                });
            let (hovering_files, dropped_files) = ui.ctx().input(|input| {
                (!input.raw.hovered_files.is_empty(), input.raw.dropped_files.clone())
            });
            if let Some(rect) = drop_rect.filter(|_| hovering_files) {
                let rect = rect.shrink(2.0);
                ui.painter().rect_stroke(rect, 11.0,
                    egui::Stroke::new(2.0_f32, home_art::BLUE), egui::StrokeKind::Inside);
                let banner = egui::Rect::from_min_size(
                    rect.min + egui::vec2(12.0, 12.0),
                    egui::vec2((rect.width() - 24.0).max(0.0), 32.0),
                );
                ui.painter().rect_filled(banner, 6.0, Color32::from_rgb(226, 241, 255));
                ui.painter().text(banner.center(), egui::Align2::CENTER_CENTER,
                    "ファイルをドロップしてメッセージに添付",
                    home_art::home_font(17.0), home_art::BLUE);
            }
            if !dropped_files.is_empty() {
                // Native Windows drops do not carry a position. Ask before saving to this draft.
                send = false;
                self.offer_dropped_chat_files(&id, &dropped_files, idle);
                ui.ctx().request_repaint();
            }
            self.chat_header(ui, idle, activity, compact_vertical, &id, &title);
            ui.add_space(10.0);
            let transcript_empty = self.progress.chats[self.chat_selected].exchanges.is_empty();
            egui::ScrollArea::vertical().id_salt(("chat-transcript", &id))
                .min_scrolled_height(0.0)
                .max_height((ui.available_height() - 8.0).max(if compact_vertical { 72.0 } else { 0.0 }))
                .auto_shrink([false, false]).stick_to_bottom(!transcript_empty && (!compact_vertical || self.pending.is_some())).show(ui, |ui| {
                let chat = &mut self.progress.chats[self.chat_selected];
                if chat.exchanges.is_empty() {
                    let welcome_width = ui.available_width();
                    if let Some(example) = chat_welcome(ui, idle && chat.draft.is_empty(), welcome_width) {
                        chat.draft = example.into();
                        self.dirty = true;
                        ui.memory_mut(|m| m.request_focus(egui::Id::new(("chat-input", &id))));
                    }
                }
                for (index, exchange) in chat.exchanges.iter_mut().enumerate() {
                    ui.push_id(index, |ui| {
                        if bubble_with_attachments(ui, egui::Id::new(("chat-copy-question", &id, index)),
                            "あなた", &exchange.question, true, &exchange.attachments,
                            &mut opened_attachment) {
                            copied = Some("あなたの発言");
                        }
                        if bubble(ui, egui::Id::new(("chat-copy-answer", &id, index)),
                            "Codex", &exchange.answer, false) {
                            copied = Some("Codexの発言");
                        }
                        ui.horizontal_wrapped(|ui| {
                            if ui.add_enabled(self.recorder.is_none(), crate::app::controls::Button::new("回答を読み上げ")).clicked() { speak=Some(exchange.answer.clone()); }
                            if ui.ww_button("英文に注釈を書く").clicked() { annotate=Some(exchange.answer.clone()); }
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
                    egui::Frame::new().fill(Color32::from_rgb(238, 247, 255))
                        .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                        .corner_radius(8).inner_margin(10).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if self.pending.is_some() || self.batch_running { ui.spinner(); }
                            ui.label(activity);
                        });
                    });
                }
                if chat.exchanges.len() >= wordweave5::chat::MAX_EXCHANGES {
                    ui.label("この会話は保存上限に達した。引き継ぎメモを確認し、新しい会話を作成する。");
                }
            });
            if let Some(label) = copied {
                self.message = format!("{label}をクリップボードへコピーした。");
            }
            if send { self.launch_chat(); }
            if let Some(text)=speak {self.say(&text);}
            if let Some(action) = opened_attachment {
                match action {
                    super::chat_media::AttachmentOpen::Image(reference) => self.preview_asset(&reference),
                    super::chat_media::AttachmentOpen::Audio(reference) => self.play_asset(&reference),
                    super::chat_media::AttachmentOpen::File(reference, name) => self.preview_file_asset(&reference, &name),
                }
            }
            if let Some(text)=annotate {
                if self.annotation.frozen() {self.notify_blocked("作成中の注釈がある。先に添付または破棄する。");}
                else {self.annotation.text=text;}
                self.chat_media_open=true;
            }
        });
        ui.set_style(previous_style);
        self.chat_windows(ui.ctx(), idle);
        self.chat_drop_confirmation(ui.ctx());
    }

    fn offer_dropped_chat_files(&mut self, id: &str, files: &[egui::DroppedFile], idle: bool) {
        if !idle || self.pending_chat_file_send.is_some() {
            self.notify_warning("処理中は添付できません。処理完了後、もう一度ドロップしてください。");
            return;
        }
        if self.pending_chat_drop.is_some() {
            self.notify_warning("先のファイル添付を確認またはキャンセルしてから、もう一度ドロップしてください。");
            return;
        }
        let paths: Vec<_> = files.iter().filter_map(|file| file.path.clone()).collect();
        let unsupported = files.len() - paths.len();
        if paths.is_empty() {
            self.notify_warning("ファイルのパスを取得できません。ファイル選択から添付してください。");
            return;
        }
        let existing = self.progress.chats.iter().find(|chat| chat.id == id)
            .map_or(0, |chat| chat.draft_attachments.len());
        if existing + paths.len() > 8 {
            self.notify_warning(format!("添付は一つの発言につき8件までです。現在{existing}件あるため、{}件をまとめて追加できません。今回のドロップは保存していません。", paths.len()));
            return;
        }
        self.pending_chat_drop = Some(ChatDropProposal {
            chat_id: id.to_owned(), files: paths, unsupported,
        });
    }

    fn chat_drop_confirmation(&mut self, ctx: &egui::Context) {
        let Some(proposal) = self.pending_chat_drop.as_ref() else { return; };
        let Some(chat) = self.progress.chats.get(self.chat_selected)
            .filter(|chat| chat.deleted_at.is_none() && chat.id == proposal.chat_id) else {
            self.pending_chat_drop = None;
            self.notify_warning("会話が切り替わったため、ドロップしたファイルは添付していません。");
            return;
        };
        let mut open = true;
        let mut approve = false;
        let mut cancel = false;
        egui::Window::new("ファイル添付を確認")
            .open(&mut open).collapsible(false).resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(460.0).show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.label(format!("会話「{}」で送信するメッセージに添付する。", chat.title));
                ui.label(format!("既存{}件＋今回{}件＝{}件 / 上限8件",
                    chat.draft_attachments.len(), proposal.files.len(),
                    chat.draft_attachments.len() + proposal.files.len()));
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for path in proposal.files.iter().take(8) {
                        ui.label(path.file_name().and_then(|name| name.to_str()).unwrap_or("ファイル"));
                    }
                });
                if proposal.unsupported > 0 {
                    ui.label(format!("{}件はパスを取得できず添付できない。", proposal.unsupported));
                }
                ui.label("確認するまで保存しない。テキスト本文をCodexへ送る場合は、送信時に別途許可を求める。");
                ui.horizontal(|ui| {
                    approve = ui.add(crate::app::controls::Button::new(
                        egui::RichText::new("メッセージに添付").color(egui::Color32::WHITE))
                        .fill(home_art::BLUE).corner_radius(8)
                        .min_size(egui::vec2(156.0, 40.0))).clicked();
                    cancel = ui.add(crate::app::controls::Button::new("キャンセル")
                        .min_size(egui::vec2(108.0, 40.0))).clicked();
                });
            });
        if !open || cancel {
            self.pending_chat_drop = None;
        } else if approve {
            self.confirm_chat_drop();
        }
    }

    fn confirm_chat_drop(&mut self) {
        let Some(proposal) = self.pending_chat_drop.take() else { return; };
        let can_edit = self.pending.is_none() && self.pending_chat_file_send.is_none()
            && !self.batch_running
            && self.session.is_none() && self.recorder.is_none() && self.fatal.is_none()
            && self.progress.chats.get(self.chat_selected)
                .is_some_and(|chat| chat.deleted_at.is_none() && chat.id == proposal.chat_id);
        if !can_edit {
            self.notify_warning("会話が切り替わったか処理中のため、ファイルは添付していません。");
            return;
        }
        let existing = self.progress.chats[self.chat_selected].draft_attachments.len();
        if existing + proposal.files.len() > 8 {
            self.notify_warning(format!("確認中に添付数が変わった。上限は8件で、現在{existing}件あるため今回は保存していません。"));
            return;
        }
        self.attach_dropped_chat_files(&proposal.chat_id, &proposal.files, proposal.unsupported);
    }

    fn attach_dropped_chat_files(&mut self, id: &str, files: &[std::path::PathBuf], unsupported: usize) {
        let mut attached = 0;
        let mut failed = unsupported;
        let mut first_error = None;
        let mut first_name = None;
        for path in files {
            match self.attach_chat_file(id, path) {
                Ok(()) => {
                    attached += 1;
                    first_name.get_or_insert_with(|| path.file_name()
                        .and_then(|name| name.to_str()).unwrap_or("ファイル").to_owned());
                }
                Err(error) => {
                    failed += 1;
                    first_error.get_or_insert(error);
                }
            }
        }
        if failed > 0 && first_error.is_none() {
            first_error = Some("このファイルのパスを取得できません。ファイル選択から添付してください。".into());
        }
        self.message = match (attached, failed) {
            (1, 0) => format!("{}をメッセージに添付した。送信前に内容を確認できる。",
                first_name.as_deref().unwrap_or("ファイル")),
            (count, 0) => format!("{count}件をメッセージに添付した。送信前に内容を確認できる。"),
            (0, count) => format!("{count}件を添付できなかった：{}",
                first_error.as_deref().unwrap_or("不明なエラー")),
            (ok, ng) => format!("{ok}件をメッセージに添付した。{ng}件は添付できなかった：{}",
                first_error.as_deref().unwrap_or("不明なエラー")),
        };
        if failed > 0 { self.notify_warning(self.message.clone()); }
    }

    fn chat_header(
        &mut self,
        ui: &mut egui::Ui,
        idle: bool,
        activity: &str,
        compact_vertical: bool,
        id: &str,
        title: &str,
    ) {
        let compact = ui.available_width() < 650.0;
        let inner_margin = if compact_vertical {
            6
        } else if compact {
            10
        } else {
            14
        };
        let header_width = ui.available_width();
        egui::Frame::new().fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
            .corner_radius(12).inner_margin(inner_margin).show(ui, |ui| {
            ui.set_min_width((header_width - inner_margin as f32 * 2.0 - 2.0).max(1.0));
            ui.spacing_mut().button_padding.x = if compact_vertical { 8.0 } else if compact { 12.0 } else { 16.0 };
            if compact_vertical {
                ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
                ui.horizontal(|ui| {
                    home_art::badge(ui, home_art::Icon::Chat, home_art::BLUE, 26.0);
                    let action_width = 340.0_f32.min((ui.available_width() * 0.82).max(260.0));
                    let title_width = (ui.available_width() - action_width).max(72.0);
                    let dense_title = if idle { title.to_owned() }
                        else { format!("{title} — {activity}") };
                    ui.add_sized([title_width, 32.0], egui::Label::new(
                        RichText::new(dense_title)
                            .family(egui::FontFamily::Name("heading".into()))
                            .size(19.0).color(home_art::INK)).truncate())
                        .on_hover_text(title);
                    ui.ww_menu_button("会話を選ぶ", |ui| {
                        ui.set_width(340.0);
                        self.chat_thread_picker(ui, idle, true);
                    });
                    if ui.ww_button(if self.progress.material_draft.is_some() {
                        "教材案を確認"
                    } else {
                        "教材案を作る"
                    }).on_hover_text("根拠を選び、差分を確認してから登録する。").clicked() {
                        self.chat_material_open = true;
                    }
                    ui.ww_menu_button("操作", |ui| {
                        ui.style_mut().text_styles.insert(egui::TextStyle::Button, home_art::home_font(18.0));
                        if ui.ww_button("タイトルを変更…").clicked() {
                            self.pending_chat_rename = Some((id.to_owned(), title.to_owned()));
                            ui.close_menu();
                        }
                        if ui.ww_button("文脈・送信内容を確認").clicked() {
                            self.chat_context_open = true;
                            ui.close_menu();
                        }
                        if ui.ww_button("音声・手書き・添付").clicked() {
                            self.chat_media_open = true;
                            ui.close_menu();
                        }
                        if ui.ww_button("実行記録・結果の再取得").clicked() {
                            self.run_history_open = true;
                            self.refresh_runs();
                            ui.close_menu();
                        }
                        if ui.ww_button("読み上げを停止").clicked() {
                            self.stop_speech();
                            ui.close_menu();
                        }
                        if ui.add_enabled(idle, crate::app::controls::Button::new("会話の削除を確認")).clicked() {
                            self.pending_chat_delete = Some(id.to_owned());
                            ui.close_menu();
                        }
                    });
                });
                return;
            }
            ui.horizontal(|ui| {
                home_art::badge(ui, home_art::Icon::Chat, home_art::BLUE,
                    if compact_vertical { 30.0 } else if compact { 36.0 } else { 44.0 });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let edit_width = if compact { 0.0 } else { 144.0 };
                        let title_width = (ui.available_width() - edit_width - ui.spacing().item_spacing.x).max(92.0);
                        ui.add_sized([title_width, 32.0], egui::Label::new(RichText::new(title)
                            .family(egui::FontFamily::Name("heading".into()))
                            .size(if compact_vertical { 20.0 } else if compact { 22.0 } else { 25.0 })
                            .color(home_art::INK)).truncate())
                            .on_hover_text(title);
                        if !compact && ui.add(crate::app::controls::Button::new("タイトル変更")
                            .min_size(egui::vec2(edit_width, 36.0))).clicked() {
                            self.pending_chat_rename = Some((id.to_owned(), title.to_owned()));
                        }
                    });
                    if !idle {
                        ui.label(RichText::new(activity)
                            .size(15.0).strong().color(home_art::BLUE));
                    } else if !compact {
                        ui.label(RichText::new("英語で質問したり、会話を練習したりできます。")
                            .size(15.0).color(home_art::MUTED));
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                if !compact && ui.ww_button("文脈を確認").clicked() { self.chat_context_open = true; }
                if !compact && ui.ww_button("音声・手書き・添付")
                    .on_hover_text("音声入力、手書き入力、保存した添付を確認する。")
                    .clicked() { self.chat_media_open = true; }
                if ui.ww_button(if self.progress.material_draft.is_some() { "教材案を確認" } else { "教材案を作る" })
                    .on_hover_text("根拠の往復を選択 → 教材案を作成 → 差分を確認して登録。確認前に教材は変更しない。")
                    .clicked() { self.chat_material_open = true; }
                if !compact && ui.add_enabled(idle, crate::app::controls::Button::new("会話の削除を確認")).clicked() {
                    self.pending_chat_delete = Some(id.to_owned());
                }
                ui.ww_menu_button(if compact { "会話の操作" } else { "その他" }, |ui| {
                    ui.style_mut().text_styles.insert(egui::TextStyle::Button, home_art::home_font(18.0));
                    if compact && ui.ww_button("タイトルを変更…").clicked() {
                        self.pending_chat_rename = Some((id.to_owned(), title.to_owned()));
                        ui.close_menu();
                    }
                    if ui.ww_button("文脈・送信内容を確認").clicked() {
                        self.chat_context_open = true;
                        ui.close_menu();
                    }
                    if compact && ui.ww_button("音声・手書き・添付").clicked() {
                        self.chat_media_open = true;
                        ui.close_menu();
                    }
                    if ui.ww_button("実行記録・結果の再取得").clicked() {
                        self.run_history_open = true;
                        self.refresh_runs();
                        ui.close_menu();
                    }
                    if ui.ww_button("読み上げを停止").clicked() {
                        self.stop_speech();
                        ui.close_menu();
                    }
                    if compact && ui.add_enabled(idle, crate::app::controls::Button::new("会話の削除を確認")).clicked() {
                        self.pending_chat_delete = Some(id.to_owned());
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn chat_thread_picker(&mut self, ui: &mut egui::Ui, idle: bool, popup: bool) {
        let mut close = false;
        if !popup {
            let footer = egui::TopBottomPanel::bottom("chat-thread-footer")
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(Color32::WHITE))
                .show_inside(ui, |ui| {
                    ui.separator();
                    self.chat_trash_controls(ui, &mut close);
                });
            #[cfg(test)]
            ui.ctx().data_mut(|data| {
                data.insert_temp(egui::Id::new("chat-thread-footer-rect"), footer.response.rect)
            });
            #[cfg(not(test))]
            let _ = footer;
        }
        let can_create = self
            .progress
            .chats
            .iter()
            .filter(|c| c.deleted_at.is_none())
            .count()
            < wordweave5::chat::MAX_CHATS;
        if ui
            .add_enabled(
                idle && can_create,
                crate::app::controls::Button::new(
                    RichText::new("＋  新しい会話")
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(home_art::BLUE)
                .corner_radius(8)
                .min_size(egui::vec2(ui.available_width(), 48.0))
                .wrap(),
            )
            .clicked()
        {
            match self.create_conversation() {
                Ok(()) => close = true,
                Err(error) => self.notify_error(error),
            }
        }
        ui.label(
            RichText::new("ピン留めを先頭に、最近使った順で表示")
                .size(14.0)
                .color(home_art::MUTED),
        );
        let height = if popup {
            340.0
        } else {
            ui.available_height().max(0.0)
        };
        egui::ScrollArea::vertical()
            .id_salt("thread-list")
            .max_height(height)
            .show(ui, |ui| {
                for index in wordweave5::chat::ordered_indices(&self.progress.chats) {
                    let chat = &self.progress.chats[index];
                    let id = chat.id.clone();
                    let title = chat.title.clone();
                    let pinned = chat.pinned;
                    let preview = chat
                        .exchanges
                        .last()
                        .map(|exchange| compact_text(&exchange.question, 34))
                        .unwrap_or_else(|| "まだメッセージはありません".into());
                    let timestamp = chat
                        .exchanges
                        .last()
                        .map_or(chat.created_at, |exchange| exchange.at);
                    let date = chrono::DateTime::from_timestamp(timestamp, 0)
                        .map(|time| {
                            time.with_timezone(&chrono::Local)
                                .format("%m/%d")
                                .to_string()
                        })
                        .unwrap_or_default();
                    let mut toggle_pin = false;
                    let mut select = false;
                    ui.push_id(&id, |ui| {
                        ui.horizontal(|ui| {
                            toggle_pin = ui
                                .add_enabled(
                                    idle,
                                    crate::app::controls::Button::new(if pinned {
                                        "★"
                                    } else {
                                        "☆"
                                    })
                                    .frame(false)
                                    .min_size(egui::vec2(34.0, 74.0)),
                                )
                                .on_hover_text("会話をピン留め／解除")
                                .clicked();
                            select = ui
                                .add_enabled_ui(idle, |ui| {
                                    chat_thread_row(
                                        ui,
                                        &title,
                                        &preview,
                                        &date,
                                        self.chat_selected == index,
                                    )
                                })
                                .inner
                                .clicked();
                        });
                    });
                    if toggle_pin {
                        self.progress.chats[index].pinned = !pinned;
                        self.dirty = true;
                    }
                    if select {
                        self.chat_selected = index;
                        close = true;
                    }
                    ui.add_space(4.0);
                }
            });
        if popup {
            ui.separator();
            self.chat_trash_controls(ui, &mut close);
        }
        if popup && close {
            ui.close_menu();
        }
    }

    fn chat_trash_controls(&mut self, ui: &mut egui::Ui, close: &mut bool) {
        ui.horizontal_wrapped(|ui| {
            if ui.ww_button("チャットのごみ箱").clicked() {
                self.conversation_trash_open = true;
                *close = true;
            }
            if ui.ww_button("削除済みの教材").clicked() {
                self.chat_trash_open = true;
                *close = true;
            }
        });
    }

    fn chat_windows(&mut self, ctx: &egui::Context, idle: bool) {
        if let Some((id, mut draft)) = self.pending_chat_rename.clone() {
            let mut open = true;
            let mut save = false;
            let mut cancel = false;
            egui::Window::new("チャットのタイトルを変更")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(480.0)
                .show(ctx, |ui| {
                    ux::dialog_body(ui);
                    ui.spacing_mut().button_padding = egui::vec2(18.0, 8.0);
                    ui.label("会話一覧とチャット上部に表示するタイトルを入力する。");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut draft)
                            .desired_width(f32::INFINITY)
                            .char_limit(100),
                    );
                    response.on_hover_text("1〜100文字。改行は入力できない。");
                    let count = draft.chars().count();
                    ui.label(
                        RichText::new(format!("{count} / 100文字・改行不可"))
                            .size(18.0)
                            .color(home_art::MUTED),
                    );
                    ui.horizontal(|ui| {
                        let save_response = ui.add_enabled(
                            !draft.trim().is_empty()
                                && count <= 100
                                && !draft.contains('\r')
                                && !draft.contains('\n'),
                            crate::app::controls::Button::new("変更を保存"),
                        );
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(egui::Id::new("chat-rename-save"), save_response.rect)
                        });
                        save = save_response.clicked();
                        cancel = ui.ww_button("キャンセル").clicked();
                    });
                });
            if save {
                match self
                    .progress
                    .chats
                    .iter_mut()
                    .find(|chat| chat.id == id)
                    .ok_or_else(|| "対象の会話が見つかりません。".to_owned())
                    .and_then(|chat| chat.rename(&draft))
                {
                    Ok(()) => {
                        self.dirty = true;
                        self.message = "チャットのタイトルを変更した。".into();
                        self.pending_chat_rename = None;
                    }
                    Err(error) => self.notify_error(error),
                }
            } else if cancel || !open {
                self.pending_chat_rename = None;
            } else {
                self.pending_chat_rename = Some((id, draft));
            }
        }
        let mut open = self.chat_context_open;
        egui::Window::new("文脈と送信内容").open(&mut open).default_width(640.0)
            .vscroll(true).show(ctx, |ui| {
            ux::dialog_body(ui);
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
                ux::dialog_body(ui);
                ui.label(
                    RichText::new("1 根拠を選ぶ → 2 教材案を作る → 3 差分を確認して登録")
                        .strong()
                        .color(ux::ACCENT),
                );
                ui.small("登録を確認するまで教材は変更しない。追加と訂正は区別して確認する。");
                ui.separator();
                if self.progress.material_draft.is_none() {
                    ui.label("教材に使う往復だけを選び、対象語と反映方法を確認する。");
                    if let Some(chat) = self
                        .progress
                        .chats
                        .get_mut(self.chat_selected)
                        .filter(|c| c.deleted_at.is_none())
                    {
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
                ux::dialog_body(ui);
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
                        if ui
                            .add_enabled(idle, crate::app::controls::Button::new("復元"))
                            .clicked()
                        {
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
            ux::dialog_body(ui);
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
                    crate::app::controls::Button::new(if action.operation == Operation::Delete { "対象を確認して削除" } else { "根拠と教材案を確認する" })).clicked();
                dismiss = ui.ww_button("キャンセル").clicked();
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
                self.notify_error(format!("保存に失敗したため削除・復元を反映していない：{error}"));
                false
            }
        }
    }
}

fn compact_text(text: &str, limit: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= limit {
        return normalized;
    }
    let mut result = normalized
        .chars()
        .take(limit.saturating_sub(1))
        .collect::<String>();
    result.push('…');
    result
}

fn fit_thread_line(ui: &egui::Ui, text: &str, font: egui::FontId, max_width: f32) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let width = |value: &str| {
        ui.painter().layout_no_wrap(value.to_owned(), font.clone(), Color32::WHITE).size().x
    };
    if width(&normalized) <= max_width {
        return normalized;
    }
    let mut result = "…".to_owned();
    for (end, _) in normalized.char_indices().skip(1) {
        let candidate = format!("{}…", &normalized[..end]);
        if width(&candidate) > max_width {
            break;
        }
        result = candidate;
    }
    result
}

fn chat_thread_row(
    ui: &mut egui::Ui,
    title: &str,
    preview: &str,
    date: &str,
    selected: bool,
) -> egui::Response {
    let desired = egui::vec2(ui.available_width().max(80.0), 74.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), title)
    });
    let fill = if selected {
        Color32::from_rgb(226, 241, 255)
    } else if response.hovered() {
        Color32::from_rgb(244, 249, 253)
    } else {
        Color32::WHITE
    };
    ui.painter().rect_filled(rect, 8.0, fill);
    ui.painter().rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(
            if selected { 1.5_f32 } else { 1.0_f32 },
            if selected {
                home_art::BLUE
            } else {
                home_art::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    let left = rect.left() + 12.0;
    let title_font = home_art::home_font(17.0);
    let preview_font = home_art::home_font(14.0);
    let date_width = ui.painter().layout_no_wrap(date.to_owned(), home_art::home_font(13.0),
        home_art::MUTED).size().x;
    let display_title = fit_thread_line(ui, title, title_font.clone(),
        (rect.width() - 22.0 - date_width - 8.0).max(0.0));
    let preview = fit_thread_line(ui, preview, preview_font.clone(),
        (rect.width() - 24.0).max(0.0));
    let painter = ui.painter().with_clip_rect(rect.shrink(2.0));
    painter.text(
        egui::pos2(left, rect.top() + 10.0),
        egui::Align2::LEFT_TOP,
        display_title,
        title_font,
        home_art::INK,
    );
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 11.0),
        egui::Align2::RIGHT_TOP,
        date,
        home_art::home_font(13.0),
        home_art::MUTED,
    );
    painter.text(
        egui::pos2(left, rect.top() + 42.0),
        egui::Align2::LEFT_TOP,
        preview,
        preview_font,
        home_art::MUTED,
    );
    response.on_hover_text(title)
}

fn chat_welcome(ui: &mut egui::Ui, enabled: bool, width: f32) -> Option<&'static str> {
    let suggestions = [
        (
            "質問する",
            "「この表現は自然ですか？」",
            "この表現は自然ですか？",
        ),
        (
            "会話の練習",
            "「自己紹介を練習したい」",
            "自己紹介を練習したいです。英語で質問してください。",
        ),
        (
            "表現を確認",
            "「sorry と apologize の違いは？」",
            "sorry と apologize の使い分けを教えて。",
        ),
        (
            "単語を教材に",
            "会話後に根拠と差分を確認",
            "この会話で出た重要な表現を教材として整理してください。",
        ),
    ];
    let mut selected = None;
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
        .corner_radius(12)
        .inner_margin(20)
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                home_art::badge(ui, home_art::Icon::Chat, home_art::BLUE, 70.0);
                ui.add_space(8.0);
                home_art::title(ui, "英語で話してみましょう", 30.0);
                ui.label("わからないことを質問したり、会話を練習したりできます。");
                ui.label(
                    RichText::new("例を選ぶと入力欄へ入り、送信前に編集できます。")
                        .size(15.0)
                        .color(home_art::MUTED),
                );
            });
            ui.add_space(14.0);
            let columns = if width >= 650.0 { 2 } else { 1 };
            for row in suggestions.chunks(columns) {
                if columns == 1 {
                    for (title, detail, prompt) in row {
                        if chat_suggestion(ui, title, detail, enabled).clicked() {
                            selected = Some(*prompt);
                        }
                    }
                } else {
                    ui.columns(2, |uis| {
                        for (column, (title, detail, prompt)) in row.iter().enumerate() {
                            if chat_suggestion(&mut uis[column], title, detail, enabled).clicked() {
                                selected = Some(*prompt);
                            }
                        }
                    });
                }
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "教材に反映するときは、会話の往復を根拠に選び、差分を確認してから登録します。",
                )
                .size(14.0)
                .color(home_art::MUTED),
            );
        });
    selected
}

fn chat_suggestion(ui: &mut egui::Ui, title: &str, detail: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        crate::app::controls::Button::new(
            RichText::new(format!("{title}\n{detail}"))
                .size(17.0)
                .color(home_art::INK),
        )
        .fill(Color32::from_rgb(247, 251, 255))
        .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
        .corner_radius(9)
        .min_size(egui::vec2(ui.available_width(), 76.0))
        .wrap(),
    )
    .on_hover_text("入力欄に入れる。送信前に編集できる。")
}

fn bubble(ui: &mut egui::Ui, _copy_id: egui::Id, speaker: &str, text: &str, user: bool) -> bool {
    bubble_with_attachments(ui, _copy_id, speaker, text, user, &[], &mut None)
}

fn bubble_with_attachments(
    ui: &mut egui::Ui,
    _copy_id: egui::Id,
    speaker: &str,
    text: &str,
    user: bool,
    attachments: &[wordweave5::chat::Attachment],
    opened: &mut Option<super::chat_media::AttachmentOpen>,
) -> bool {
    let available = ui.available_rect_before_wrap();
    let clip = ui.clip_rect();
    let visible_left = available.left().max(clip.left());
    let visible_right = available.right().min(clip.right());
    let width = ((visible_right - visible_left).max(0.0) * 0.90).min(820.0);
    let clipped_edge = if user {
        (available.right() - visible_right).max(0.0)
    } else {
        (visible_left - available.left()).max(0.0)
    };
    let mut copied = false;
    let layout = if user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };
    ui.with_layout(layout, |ui| {
        if clipped_edge > 0.0 {
            ui.add_space(clipped_edge);
        }
        // A Frame inherits its parent's layout. Put it in an explicit vertical child so
        // the right-to-left alignment cannot stretch the bubble to the transcript height.
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_width(width);
                let _frame = egui::Frame::new()
                    .fill(if user {
                        Color32::from_rgb(231, 244, 255)
                    } else {
                        Color32::WHITE
                    })
                    .stroke(egui::Stroke::new(1.0_f32, home_art::BORDER))
                    .inner_margin(14.0)
                    .corner_radius(12.0)
                    .show(ui, |ui| {
                        ui.set_width((width - 28.0).max(0.0));
                        ui.label(RichText::new(speaker).strong().color(if user {
                            home_art::BLUE
                        } else {
                            home_art::GREEN
                        }));
                        ui.add(egui::Label::new(text).wrap().selectable(true));
                        if !attachments.is_empty() {
                            ui.add_space(8.0);
                            ui.separator();
                            ui.label(RichText::new(format!("添付 {}件", attachments.len()))
                                .size(15.0).color(home_art::MUTED));
                            ui.horizontal_wrapped(|ui| {
                                for (index, attachment) in attachments.iter().enumerate() {
                                    if let Some(action) = super::chat_media::attachment_button(ui, attachment, index) {
                                        *opened = Some(action);
                                    }
                                }
                            });
                        }
                        ui.with_layout(
                            if user {
                                egui::Layout::right_to_left(egui::Align::Center)
                            } else {
                                egui::Layout::left_to_right(egui::Align::Center)
                            },
                            |ui| {
                                let response = ui.add(
                                    egui::Button::new("")
                                        .min_size(egui::vec2(34.0, 34.0))
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE),
                                );
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        ui.is_enabled(),
                                        "この発言をコピー",
                                    )
                                });
                                home_art::icon(
                                    ui.painter(),
                                    response.rect.shrink(8.0),
                                    home_art::Icon::Copy,
                                    home_art::MUTED,
                                );
                                #[cfg(test)]
                                ui.ctx().data_mut(|data| {
                                    data.insert_temp(_copy_id, response.rect);
                                    data.insert_temp(_copy_id.with("clip"), ui.clip_rect());
                                });
                                if response.on_hover_text("この発言をコピー").clicked() {
                                    ui.ctx().copy_text(text.to_owned());
                                    copied = true;
                                }
                            },
                        );
                    });
                #[cfg(test)]
                ui.ctx().data_mut(|data| {
                    data.insert_temp(_copy_id.with("bubble"), _frame.response.rect)
                });
            },
        );
    });
    copied
}

// A conversion-confirmation frame must not also submit the draft.
fn composer_keys(events: &mut Vec<egui::Event>, focused: bool, composing: &mut bool) -> bool {
    if !focused {
        *composing = false;
        return false;
    }
    let mut ime_frame = *composing;
    for event in events.iter() {
        if let egui::Event::Ime(event) = event {
            ime_frame = true;
            *composing = matches!(event, egui::ImeEvent::Enabled | egui::ImeEvent::Preedit(_));
        }
    }
    let submit = !ime_frame
        && events.iter().any(|event| {
            matches!(event,
        egui::Event::Key { key: egui::Key::Enter, pressed: true, repeat: false, modifiers, .. }
            if modifiers.ctrl && !modifiers.shift && !modifiers.alt && !modifiers.mac_cmd)
        });
    // Leave IME commit/preedit events intact. Only the confirmation/shortcut key
    // is withheld from TextEdit; a normal Enter still inserts a newline.
    events.retain(|event| {
        !matches!(event,
        egui::Event::Key { key: egui::Key::Enter, modifiers, .. } if ime_frame || modifiers.ctrl)
    });
    submit
}

#[cfg(test)]
mod input_tests {
    use super::*;

    #[test]
    fn dropped_files_wait_for_explicit_confirmation_before_attaching_to_the_unsent_draft() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let text_path = root.join("reading-note.txt");
        let image_path = root.join("picture.png");
        std::fs::write(&text_path, "An example in context.").unwrap();
        image::RgbImage::new(1, 1).save(&image_path).unwrap();
        app.progress.chats[0].draft = "Explain this phrase.".into();
        let _ = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            dropped_files: vec![text_path, image_path].into_iter()
                .map(|path| egui::DroppedFile { path: Some(path), ..Default::default() })
                .collect(),
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
        });
        let output = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
        });
        let mut dialog_text = String::new();
        for shape in output.shapes {
            super::super::harness_tests::shape_text(&shape.shape, &mut dialog_text);
        }
        assert!(dialog_text.contains("ファイル添付を確認")
            && dialog_text.contains("reading-note.txt") && dialog_text.contains("picture.png"),
            "{dialog_text}");
        assert_eq!(app.pending_chat_drop.as_ref().unwrap().files.len(), 2);
        assert!(app.progress.chats[0].draft_attachments.is_empty());
        assert!(app.storage.as_ref().unwrap().load().unwrap().chats[0].draft_attachments.is_empty());
        app.confirm_chat_drop();
        let chat = &app.progress.chats[0];
        assert_eq!(chat.draft, "Explain this phrase.");
        assert!(chat.exchanges.is_empty());
        assert_eq!(chat.draft_attachments.len(), 2);
        assert_eq!(chat.draft_attachments[0].file_name.as_deref(), Some("reading-note.txt"));
        assert_eq!(chat.draft_attachments[1].file_name.as_deref(), Some("picture.png"));
        assert_eq!(chat.draft_attachments[0].original.kind, wordweave5::assets::AssetKind::FileBlob);
        assert_eq!(chat.draft_attachments[1].original.kind, wordweave5::assets::AssetKind::ImagePng);
        assert!(app.pending.is_none() && app.pending_chat_file_send.is_none());
        assert!(app.pending_chat_drop.is_none());
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().chats[0].draft_attachments.len(), 2);
        assert!(app.message.contains("2件をメッセージに添付した"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejected_drops_preserve_the_draft_and_report_partial_batch_results() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let valid = root.join("valid.txt");
        let oversized = root.join("oversized.txt");
        std::fs::write(&valid, "keep this").unwrap();
        std::fs::File::create(&oversized).unwrap()
            .set_len(wordweave5::assets::MAX_ASSET_BYTES as u64 + 1).unwrap();
        app.progress.chats[0].draft = "Original draft".into();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            dropped_files: vec![
                egui::DroppedFile { path: Some(oversized), ..Default::default() },
                egui::DroppedFile { path: None, name: "no-path.txt".into(), ..Default::default() },
                egui::DroppedFile { path: Some(valid.clone()), ..Default::default() },
            ],
            ..Default::default()
        };
        let _ = ctx.run(raw, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
        });
        assert_eq!(app.progress.chats[0].draft, "Original draft");
        assert!(app.progress.chats[0].draft_attachments.is_empty());
        assert_eq!(app.pending_chat_drop.as_ref().unwrap().unsupported, 1);
        app.confirm_chat_drop();
        assert_eq!(app.progress.chats[0].draft_attachments.len(), 1);
        assert_eq!(app.progress.chats[0].draft_attachments[0].file_name.as_deref(), Some("valid.txt"));
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().chats[0].draft_attachments.len(), 1);
        assert!(app.message.contains("1件をメッセージに添付した") && app.message.contains("2件は添付できなかった"));
        assert!(app.message.contains("12MiB"));
        let before = serde_json::to_vec(&app.progress).unwrap();
        let (_tx, rx) = std::sync::mpsc::channel();
        app.pending = Some(Pending {
            key: String::new(), rx, cancel: None, kind: activity::Activity::Chat,
        });
        let _ = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            dropped_files: vec![egui::DroppedFile { path: Some(valid), ..Default::default() }],
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
        });
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        assert!(app.message.contains("処理中は添付できません"));
        assert!(app.pending_chat_drop.is_none());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn positionless_drop_can_be_cancelled_without_saving() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let path = root.join("private.txt");
        std::fs::write(&path, "not attached").unwrap();
        let before = serde_json::to_vec(&app.progress).unwrap();
        let _ = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
            dropped_files: vec![egui::DroppedFile {
                path: Some(path), ..Default::default()
            }],
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
        });
        assert!(app.pending_chat_drop.is_some());
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        app.pending_chat_drop = None;
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        assert!(app.progress.chats[0].draft_attachments.is_empty());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn drop_confirmation_never_approves_more_files_than_the_remaining_slots() {
        let (_ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        let path = root.join("small.txt");
        std::fs::write(&path, "small").unwrap();
        for _ in 0..7 {
            app.attach_chat_file(&id, &path).unwrap();
        }
        let dropped = egui::DroppedFile { path: Some(path.clone()), ..Default::default() };
        app.offer_dropped_chat_files(&id, &[dropped.clone(), dropped.clone()], true);
        assert!(app.pending_chat_drop.is_none());
        assert_eq!(app.progress.chats[0].draft_attachments.len(), 7);
        assert!(app.message.contains("8件まで") && app.message.contains("保存していません"));

        app.offer_dropped_chat_files(&id, &[dropped], true);
        assert!(app.pending_chat_drop.is_some());
        app.attach_chat_file(&id, &path).unwrap();
        app.confirm_chat_drop();
        assert!(app.pending_chat_drop.is_none());
        assert_eq!(app.progress.chats[0].draft_attachments.len(), 8);
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().chats[0].draft_attachments.len(), 8);
        assert!(app.message.contains("確認中に添付数が変わった"));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dragging_files_highlights_the_composer_without_attaching_them() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let before = serde_json::to_vec(&app.progress).unwrap();
        for size in [egui::vec2(1120.0, 850.0), egui::vec2(512.0, 420.0)] {
            let output = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                hovered_files: vec![egui::HoveredFile {
                    path: Some(root.join("hovered.txt")), ..Default::default()
                }],
                ..Default::default()
            }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
            });
            let mut text = String::new();
            for shape in output.shapes {
                super::super::harness_tests::shape_text(&shape.shape, &mut text);
            }
            assert!(text.contains("ファイルをドロップしてメッセージに添付"),
                "size={size:?}\n{text}");
        }
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn compact_chat_keeps_attachment_bubble_in_the_visible_transcript() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        let make_ref = |kind| wordweave5::assets::AssetRef {
            id: "a".repeat(64), kind, bytes: 12,
        };
        let chat = &mut app.progress.chats[0];
        chat.title = "テスト会話".into();
        chat.title_manual = true;
        chat.draft_attachments = vec![
            wordweave5::chat::Attachment { original: make_ref(wordweave5::assets::AssetKind::ImagePng),
                image: Some(make_ref(wordweave5::assets::AssetKind::ImagePng)), background: None,
                file_name: Some("photo.png".into()), source_text: "photo.png".into(), transcript: None },
            wordweave5::chat::Attachment { original: make_ref(wordweave5::assets::AssetKind::AudioWav),
                image: None, background: None, file_name: Some("voice.wav".into()),
                source_text: "voice.wav".into(), transcript: None },
            wordweave5::chat::Attachment { original: make_ref(wordweave5::assets::AssetKind::FileBlob),
                image: None, background: None, file_name: Some("note.txt".into()),
                source_text: "note.txt".into(), transcript: None },
        ];
        chat.complete("添付を確認".into(), "会話を続けます。".into(),
            wordweave5::execution::Execution::default()).unwrap();
        let mut visible_text = String::new();
        for _ in 0..3 {
            let output = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.0, 420.0))),
                ..Default::default()
            }, |ctx| { egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui)); });
            visible_text.clear();
            for shape in output.shapes { super::super::harness_tests::shape_text(&shape.shape, &mut visible_text); }
        }
        let copy_id = egui::Id::new(("chat-copy-question", &id, 0));
        let bubble = ctx.data(|data| data.get_temp::<egui::Rect>(copy_id.with("bubble")));
        assert!(visible_text.contains("あなた") && bubble.is_some_and(|rect| rect.top() < 420.0 && rect.bottom() > 0.0),
            "bubble={bubble:?}\n{visible_text}");
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn narrow_chat_keeps_composer_and_approval_workflow_without_changing_draft() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        app.progress.chats[0].draft = "保存する下書き".into();
        let before = serde_json::to_vec(&app.progress).unwrap();
        let mut text = String::new();
        for _ in 0..2 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(512.0, 420.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                },
            );
            text.clear();
            for shape in output.shapes {
                super::super::harness_tests::shape_text(&shape.shape, &mut text);
            }
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
        app.pending = Some(Pending {
            key: String::new(),
            rx,
            cancel: None,
            kind: activity::Activity::Playback,
        });
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1120.0, 850.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
            },
        );
        let mut text = String::new();
        for shape in output.shapes {
            super::super::harness_tests::shape_text(&shape.shape, &mut text);
        }
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
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(512.0, 300.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                },
            );
            send_visible = output.shapes.iter().any(|shape| {
                if let egui::epaint::Shape::Text(text) = &shape.shape {
                    text.galley.text() == "送信 (Ctrl+Enter)"
                        && shape.clip_rect.contains(text.pos)
                        && shape.clip_rect.contains(text.pos + text.galley.size())
                } else {
                    false
                }
            });
        }
        assert!(
            send_visible,
            "Send must remain fully visible with a long draft and a short composer"
        );
        assert_eq!(app.progress.chats[0].draft, "送信前の下書き\n".repeat(20));
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn chat_states_and_responsive_layout_preserve_drafts_media_and_learning_data() {
        for (size, populated, busy) in [
            (egui::vec2(1400.0, 900.0), false, false),
            (egui::vec2(900.0, 700.0), true, false),
            (egui::vec2(512.0, 420.0), true, true),
        ] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            if populated {
                app.progress.chats[0]
                    .complete(
                        "Could you help me practice a self-introduction?".into(),
                        "Of course. What would you like to say first?".into(),
                        wordweave5::execution::Execution::default(),
                    )
                    .unwrap();
            }
            app.progress.chats[0].draft = "送信前に残す下書き".into();
            app.annotation.text = "消してはいけない注釈".into();
            let chat_id = app.progress.chats[0].id.clone();
            app.unsaved_chat_audio = Some((chat_id, b"unsaved audio".to_vec()));
            let mut pending_sender = None;
            if busy {
                let (tx, rx) = std::sync::mpsc::channel();
                app.pending = Some(Pending {
                    key: String::new(),
                    rx,
                    cancel: None,
                    kind: activity::Activity::Chat,
                });
                pending_sender = Some(tx);
            }
            let progress_before = serde_json::to_vec(&app.progress).unwrap();
            let deck_before = serde_json::to_vec(&app.deck).unwrap();
            let annotation_before = app.annotation.text.clone();
            let audio_before = app.unsaved_chat_audio.clone();
            let mut text = String::new();
            for _ in 0..3 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                    },
                );
                text.clear();
                for shape in output.shapes {
                    super::super::harness_tests::shape_text(&shape.shape, &mut text);
                }
            }
            assert!(text.contains("送信 (Ctrl+Enter)"), "size={size:?}\n{text}");
            assert!(text.contains("教材案を作る"), "size={size:?}\n{text}");
            if populated {
                assert!(
                    text.contains("会話の削除を確認")
                        || text
                            .lines()
                            .any(|line| { matches!(line.trim(), "会話の操作" | "操作") }),
                    "size={size:?}\n{text}"
                );
            } else {
                assert!(text.contains("質問する"), "size={size:?}\n{text}");
            }
            if busy {
                assert!(
                    text.contains("Codexが回答を準備中"),
                    "size={size:?}\n{text}"
                );
            }
            assert_eq!(serde_json::to_vec(&app.progress).unwrap(), progress_before);
            assert_eq!(serde_json::to_vec(&app.deck).unwrap(), deck_before);
            assert_eq!(app.annotation.text, annotation_before);
            assert_eq!(app.unsaved_chat_audio, audio_before);
            drop(pending_sender);
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn no_conversation_state_is_actionable_without_creating_or_modifying_data() {
        for size in [egui::vec2(1400.0, 900.0), egui::vec2(512.0, 420.0)] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            app.progress.chats.clear();
            let before = serde_json::to_vec(&app.progress).unwrap();
            let mut text = String::new();
            for _ in 0..2 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                    },
                );
                text.clear();
                for shape in output.shapes {
                    super::super::harness_tests::shape_text(&shape.shape, &mut text);
                }
            }
            assert!(
                text.contains("英語で話してみましょう"),
                "size={size:?}\n{text}"
            );
            assert!(text.contains("新しい会話"), "size={size:?}\n{text}");
            assert_eq!(serde_json::to_vec(&app.progress).unwrap(), before);
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    fn enter(ctrl: bool) -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: if ctrl {
                egui::Modifiers::CTRL
            } else {
                egui::Modifiers::NONE
            },
        }
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
        assert!(!composer_keys(
            &mut vec![egui::Event::Ime(egui::ImeEvent::Preedit("日本語".into()))],
            true,
            &mut composing
        ));
        assert!(!composer_keys(&mut vec![enter(true)], true, &mut composing));
        let mut events = vec![
            egui::Event::Ime(egui::ImeEvent::Commit("日本語".into())),
            enter(false),
        ];
        assert!(!composer_keys(&mut events, true, &mut composing));
        assert!(matches!(
            events.as_slice(),
            [egui::Event::Ime(egui::ImeEvent::Commit(_))]
        ));
        assert!(composer_keys(&mut vec![enter(true)], true, &mut composing));
    }
    #[test]
    fn plain_enter_inserts_newline_but_ime_confirmation_does_not() {
        let ctx = egui::Context::default();
        let id = egui::Id::new("input-test");
        let mut text = "日本語".to_owned();
        let mut composing = false;
        for (events, expected_newlines) in [
            (vec![], 0),
            (
                vec![
                    egui::Event::Ime(egui::ImeEvent::Commit("漢字".into())),
                    enter(false),
                ],
                0,
            ),
            (vec![enter(false)], 1),
        ] {
            let _ = ctx.run(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.memory_mut(|m| m.request_focus(id));
                        ui.input_mut(|i| {
                            composer_keys(&mut i.events, true, &mut composing);
                        });
                        ui.add(egui::TextEdit::multiline(&mut text).id(id));
                    });
                },
            );
            assert_eq!(text.matches('\n').count(), expected_newlines);
        }
    }

    #[test]
    fn copy_icon_copies_exactly_one_complete_utterance() {
        let ctx = egui::Context::default();
        let copy_id = egui::Id::new("copy-one-utterance");
        let utterance = "I need this line.\nこの行だけをコピーする。";
        let frame = |events| {
            let mut clicked = false;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(720.0, 360.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        clicked = bubble(ui, copy_id, "Codex", utterance, false);
                    });
                },
            );
            (output, clicked)
        };
        for _ in 0..2 {
            frame(vec![]);
        }
        let rect = ctx
            .data(|data| data.get_temp::<egui::Rect>(copy_id))
            .unwrap();
        let pos = rect.center();
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        let (output, clicked) = frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(clicked);
        assert!(output.platform_output.commands.iter().any(|command| {
            matches!(command, egui::OutputCommand::CopyText(text) if text == utterance)
        }));
    }

    #[test]
    fn chat_bubble_height_follows_content_instead_of_transcript_height() {
        let ctx = egui::Context::default();
        for (width, user) in [(1150.0, true), (620.0, false)] {
            let copy_id = egui::Id::new(("bounded-chat-bubble", width as i32, user));
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 850.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.set_min_height(760.0);
                            bubble(
                                ui,
                                copy_id,
                                if user { "あなた" } else { "Codex" },
                                "This is one message.\n矩形は本文の高さだけを使う。",
                                user,
                            );
                        });
                    },
                );
            }
            let rect = ctx
                .data(|data| data.get_temp::<egui::Rect>(copy_id.with("bubble")))
                .unwrap();
            assert!(rect.height() >= 70.0, "bubble is too short: {rect:?}");
            assert!(
                rect.height() <= 220.0,
                "bubble filled the transcript: {rect:?}"
            );
            assert!(rect.right() <= ctx.screen_rect().right() + 1.0, "{rect:?}");
        }
    }

    #[test]
    fn chat_bubble_and_copy_control_stay_inside_the_transcript_clip() {
        let ctx = egui::Context::default();
        for (width, user) in [(360.0, true), (620.0, false), (1100.0, true)] {
            let copy_id = egui::Id::new(("clipped-chat-bubble", width as i32, user));
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 400.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                bubble(ui, copy_id, if user { "あなた" } else { "Codex" },
                                    &"Long English sentence and 長い日本語の発言。".repeat(12), user);
                            });
                        });
                    },
                );
            }
            let (bubble_rect, copy_rect, clip_rect) = ctx.data(|data| {
                (
                    data.get_temp::<egui::Rect>(copy_id.with("bubble")).unwrap(),
                    data.get_temp::<egui::Rect>(copy_id).unwrap(),
                    data.get_temp::<egui::Rect>(copy_id.with("clip")).unwrap(),
                )
            });
            assert!(bubble_rect.right() <= clip_rect.right() + 1.0,
                "bubble={bubble_rect:?}, clip={clip_rect:?}, width={width}");
            assert!(copy_rect.right() <= clip_rect.right() + 1.0,
                "copy={copy_rect:?}, clip={clip_rect:?}, width={width}");
            assert!(bubble_rect.contains(copy_rect.min) && bubble_rect.contains(copy_rect.max),
                "bubble={bubble_rect:?}, copy={copy_rect:?}, width={width}");
        }
    }

    #[test]
    fn sidebar_trash_controls_remain_at_the_bottom_as_threads_grow() {
        let mut footer_tops = Vec::new();
        for chat_count in [1, 5] {
            let (ctx, mut app, root) = super::super::harness_tests::fixture();
            let first_chat = app.progress.chats[0].clone();
            for index in 1..chat_count {
                let mut chat = first_chat.clone();
                chat.id = format!("sidebar-test-chat-{index}");
                app.progress.chats.push(chat);
            }
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1200.0, 700.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                    },
                );
            }
            let footer = ctx.data(|data| {
                data.get_temp::<egui::Rect>(egui::Id::new("chat-thread-footer-rect"))
                    .unwrap()
            });
            assert!(footer.bottom() >= 660.0, "footer={footer:?}");
            footer_tops.push(footer.top());
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
        assert!((footer_tops[0] - footer_tops[1]).abs() <= 1.0,
            "footer moved with thread count: {footer_tops:?}");
    }

    #[test]
    fn rename_dialog_changes_only_the_selected_chat_title() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        app.progress.chats[0].draft = "送信前の下書き".into();
        let draft_before = app.progress.chats[0].draft.clone();
        app.pending_chat_rename = Some((id, "  海外旅行の練習  ".into()));
        let mut frame = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1150.0, 950.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.chat_page(ui));
                },
            )
        };
        for _ in 0..2 {
            frame(vec![]);
        }
        let rect = ctx
            .data(|data| data.get_temp::<egui::Rect>(egui::Id::new("chat-rename-save")))
            .unwrap();
        let pos = rect.center();
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        drop(frame);
        assert_eq!(app.progress.chats[0].title, "海外旅行の練習");
        assert!(app.progress.chats[0].title_manual);
        assert_eq!(app.progress.chats[0].draft, draft_before);
        assert!(app.dirty);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}
