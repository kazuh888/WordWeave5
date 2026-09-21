use super::*;
use wordweave5::{
    assets::{AssetKind, AssetRef, AssetStore},
    chat::Attachment,
};

impl WordApp {
    fn asset_store(&self) -> Result<AssetStore, String> {
        Ok(AssetStore::new(
            self.storage
                .as_ref()
                .ok_or("保存先がありません。")?
                .dir
                .clone(),
        ))
    }
    pub(super) fn attachment_images(
        &self,
        attachments: &[Attachment],
    ) -> Result<Vec<Vec<u8>>, String> {
        let store = self.asset_store()?;
        let mut images = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        let mut total = 0;
        for a in attachments {
            a.validate()?;
            store.read(&a.original)?;
            if let Some(background) = &a.background {
                store.read(background)?;
            }
            if let Some(reference) = &a.image {
                if seen.insert(reference.id.clone()) {
                    let bytes = store.read(reference)?;
                    total += bytes.len();
                    if total > 12_000_000 || images.len() >= 8 {
                        return Err(
                            "送信画像は8件・合計12MBまでです。根拠の選択を減らしてください。"
                                .into(),
                        );
                    }
                    images.push(bytes);
                }
            }
        }
        Ok(images)
    }
    fn add_chat_attachment(&mut self, id: &str, attachment: Attachment) -> Result<(), String> {
        let mut next = self.progress.clone();
        let chat = next
            .chats
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or("保存先の会話がありません。")?;
        if chat.deleted_at.is_some() { return Err("ごみ箱の会話には添付できない。先に復元するか、原録音を退避してください。".into()); }
        chat.draft_attachments.push(attachment);
        self.storage
            .as_ref()
            .ok_or("保存先がありません。")?
            .save(&next)?;
        self.progress = next;
        self.dirty = false;
        self.last_save = Instant::now();
        Ok(())
    }
    pub(super) fn save_chat_recording(&mut self, id: &str, wav: Vec<u8>) {
        let operation = DiagnosticOperation::begin(DiagnosticEntry::Storage);
        let result = (|| {
            let original = self.asset_store()?.put(AssetKind::AudioWav, &wav)?;
            self.add_chat_attachment(
                id,
                Attachment {
                    original,
                    image: None,
                    background: None,
                    source_text: "録音（音声本体は文字起こしを選んだ時だけCodexへ送信）".into(),
                    transcript: None,
                },
            )
        })();
        self.message = match result {
            Ok(()) => {
                operation.event(DiagnosticStage::Save, DiagnosticEvent::Attached);
                self.unsaved_chat_audio = None;
                "原録音を保存した。「文字起こし」で認識し、確認・訂正して送信できる。".into()
            }
            Err(error) => {
                operation.fail(DiagnosticStage::Save, DiagnosticError::Io);
                self.unsaved_chat_audio = Some((id.into(), wav));
                format!("録音の保存に失敗した。アプリを閉じず、音声入力画面で再保存またはWAV退避を選んでください：{error}")
            }
        };
    }
    pub(super) fn preview_asset(&mut self, reference: &AssetRef) {
        let result = (|| {
            let bytes = self.asset_store()?.read(reference)?;
            let (w, h) = image::ImageReader::with_format(
                std::io::Cursor::new(&bytes),
                image::ImageFormat::Png,
            )
            .into_dimensions()
            .map_err(|e| e.to_string())?;
            if w == 0 || h == 0 || w as u64 * h as u64 > 4_000_000 {
                return Err("画像の寸法が上限を超えています。".into());
            }
            let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?
                .to_rgba8();
            Ok(egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                image.as_raw(),
            ))
        })();
        match result {
            Ok(image) => {
                // The context is supplied by the current viewport in chat_media_windows.
                self.preview_pixels = Some((reference.id.clone(), image));
            }
            Err(e) => self.message = e,
        }
    }
    pub(super) fn play_asset(&mut self, reference: &AssetRef) {
        if self.pending.is_some() {
            return;
        }
        let result = self.asset_store().and_then(|s| s.read(reference));
        match result {
            Ok(bytes) => {
                let dir = self.storage.as_ref().unwrap().dir.clone();
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(media::playback(bytes, dir).map(|_| AiResult::Played));
                });
                self.pending = Some(Pending {
                    kind: Activity::Playback,
                    key: String::new(),
                    rx,
                    cancel: None,
                });
            }
            Err(e) => self.message = e,
        }
    }
    fn recognize_chat(&mut self, id: String, reference: AssetRef, original_id: String) {
        if self.pending.is_some() || self.recorder.is_some() || self.fatal.is_some() {
            return;
        }
        let Some(chat) = self.progress.chats.iter().find(|c| c.id == id) else {
            return;
        };
        let expected = chat.draft.clone();
        let prepared = (|| {
            Ok((
                self.asset_store()?.read(&reference)?,
                ai::Config::from_settings(&self.progress.settings)?,
            ))
        })();
        let (bytes, config) = match prepared {
            Ok(v) => v,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        if !self.reserve_generation() {
            return;
        }
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let recognized = if reference.kind == AssetKind::ImagePng {
                config.read_ink(bytes)
            } else {
                config.transcribe(bytes)
            };
            let result = recognized.map(|text| AiResult::ChatRecognized {
                id,
                expected,
                asset_id: original_id,
                text,
            });
            let _ = tx.send(result);
        });
        self.pending = Some(Pending {
            kind: Activity::Recognition,
            key: String::new(),
            rx,
            cancel: Some(cancel),
        });
        self.message =
            "選択した録音または画像をCodexへ送って文字起こし中。自動でチャット送信はしない。"
                .into();
    }
    pub(super) fn media_preview_window(&mut self, ctx: &egui::Context) {
        if let Some((id, pixels)) = self.preview_pixels.take() {
            self.media_preview = Some((
                id.clone(),
                ctx.load_texture(id, pixels, egui::TextureOptions::LINEAR),
            ));
        }
        if let Some((_, texture)) = &self.media_preview {
            let mut open = true;
            egui::Window::new("保存された画像（送信時の固定版）")
                .open(&mut open)
                .resizable(true)
                .default_width(850.0)
                .show(ctx, |ui| {
                    ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(
                        ui.available_width(),
                        ui.available_width() * texture.size()[1] as f32 / texture.size()[0] as f32,
                    )));
                });
            if !open {
                self.media_preview = None;
            }
        }
    }
    pub(super) fn export_unsaved_chat_audio(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), String> {
        let (_, bytes) = self
            .unsaved_chat_audio
            .as_ref()
            .ok_or("未保存の録音はありません。")?;
        store::atomic_write(path, bytes)?;
        self.unsaved_chat_audio = None;
        self.discard_audio_confirm = false;
        Ok(())
    }
    pub(super) fn export_chat_annotation_text(
        &mut self,
        dest: &std::path::Path,
    ) -> Result<(), String> {
        if self.annotation.frozen() || self.annotation.text.is_empty() {
            return Err("退避する未固定の原文はありません。固定後は筆跡・背景・送信画像をまとめて退避してください。".into());
        }
        store::atomic_write(dest, self.annotation.text.as_bytes())?;
        self.annotation = Default::default();
        self.discard_annotation_confirm = false;
        Ok(())
    }
    pub(super) fn export_chat_annotation(&mut self, dest: &std::path::Path) -> Result<(), String> {
        let (ink, background, composite) = self.annotation.files()?;
        let files = [
            ("ink.json", ink),
            ("background.png", background),
            ("composite.png", composite),
        ];
        let mut manifest = Vec::new();
        for (name, bytes) in &files {
            manifest.push(serde_json::json!({"file":name,"bytes":bytes.len(),"sha256":wordweave5::assets::sha256(bytes)?}));
        }
        let marker = serde_json::to_vec_pretty(&serde_json::json!({"version":1,"files":manifest}))
            .map_err(|e| e.to_string())?;
        // Never merge with an existing directory. The marker is published only after every original.
        std::fs::create_dir(dest)
            .map_err(|e| format!("新しい退避先フォルダーを作成できません：{e}"))?;
        for (name, bytes) in &files {
            store::atomic_write(&dest.join(name), bytes)?;
        }
        store::atomic_write(&dest.join("complete.json"), &marker)?;
        self.annotation = Default::default();
        self.discard_annotation_confirm = false;
        Ok(())
    }
    pub(super) fn chat_media_windows(&mut self, ctx: &egui::Context, idle: bool) {
        let can_edit =
            idle && self.fatal.is_none() && self.progress.chats.get(self.chat_selected).is_some_and(|c| c.deleted_at.is_none());
        let mut open = self.chat_media_open;
        let mut record = false;
        let mut recognize = None;
        let mut play = None;
        let mut preview = None;
        let mut attach = false;
        let mut detach = None;
        let mut retry = false;
        let mut export_audio = false;
        let mut export_annotation = false;
        let mut export_annotation_text = false;
        egui::Window::new("音声入力・英文への手書き注釈").open(&mut open).default_width(900.0).vscroll(true).show(ctx,|ui| {
            if self.fatal.is_some() { ui.colored_label(Color32::RED,"保存障害のため通常の入力・添付登録は停止中。未保存の録音・筆跡は、この画面から別ファイルへ退避できる。"); }
            ui.heading("音声入力");
            ui.label("録音はPCに保存する。文字起こしを選ぶと原音をCodexに送る。認識結果を確認してからチャット送信する。");
            if self.recorder.is_some() {
                self.recording_controls(ui, "録音を終了して保存");
            } else {record=ui.add_enabled(can_edit && self.unsaved_chat_audio.is_none(),egui::Button::new("録音を開始（最大30秒）")).clicked();}
            if self.unsaved_chat_audio.is_some() {
                ui.colored_label(Color32::RED,"未保存の録音がある。閉じると失われるため、再保存またはWAV退避する。");
                retry=ui.add_enabled(can_edit,egui::Button::new("録音の保存を再試行")).clicked();
                export_audio=ui.button("WAVへ退避（添付登録とは別）").clicked();
                if !self.discard_audio_confirm {
                    if ui.button("未保存の録音を破棄…").clicked(){self.discard_audio_confirm=true;}
                } else {
                    ui.label("この録音を保存せずに破棄する。この操作は取り消せない。");
                    ui.horizontal(|ui| {
                        if ui.button("録音を保存せず破棄する").clicked(){self.unsaved_chat_audio=None;self.discard_audio_confirm=false;}
                        if ui.button("録音を保持する").clicked(){self.discard_audio_confirm=false;}
                    });
                }
            }
            if let Some(chat)=self.progress.chats.get(self.chat_selected) {
                for (i,a) in chat.draft_attachments.iter().enumerate() {ui.push_id(i,|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("添付{}：{}",i+1,a.source_text));
                        if a.original.kind==AssetKind::AudioWav {
                            if ui.add_enabled(idle,egui::Button::new("原音を再生")).clicked(){play=Some(a.original.clone());}
                            if ui.add_enabled(can_edit,egui::Button::new("Codexで文字起こし")).clicked(){recognize=Some((chat.id.clone(),a.original.clone(),a.original.id.clone()));}
                        }
                        if let Some(image)=&a.image {
                            if ui.button("送信画像を確認").clicked(){preview=Some(image.clone());}
                            if ui.add_enabled(can_edit,egui::Button::new("画像の文字を入力欄へ認識")).clicked(){recognize=Some((chat.id.clone(),image.clone(),a.original.id.clone()));}
                        }
                        if ui.add_enabled(can_edit,egui::Button::new("添付を外す"))
                            .on_hover_text("このメッセージへの添付を解除する。保存済みの録音・筆跡と、入力欄へ反映済みの文章は削除しない。")
                            .clicked(){detach=Some((chat.id.clone(),i));}
                    });
                    if let Some(t)=&a.transcript {ui.collapsing("元の認識結果（訂正前）",|ui|{ui.label(t);});}
                });}
            }
            ui.separator();ui.heading("英文への手書き注釈");
            ui.label("英文を貼り付けて固定し、その上に注釈を書く。登録後は元の英文・筆跡・画像が残る。");
            ui.add_enabled(!self.annotation.frozen() && can_edit,egui::TextEdit::multiline(&mut self.annotation.text).desired_rows(3).desired_width(f32::INFINITY).char_limit(2000));
            if !self.annotation.frozen() {
                if ui.add_enabled(can_edit && self.annotation.text.is_empty(),egui::Button::new("空白面に手書きする")).on_disabled_hover_text("原文がある場合は、先にTXT退避または明示的に破棄する。").clicked() {
                    if let Err(e)=self.annotation.freeze_blank(ctx){self.message=e;}
                }
                if ui.add_enabled(can_edit,egui::Button::new("英文を固定して注釈を書く")).clicked() {
                    if let Err(e)=self.annotation.freeze(ctx){self.message=e;}
                }
                if !self.annotation.text.is_empty() {
                    export_annotation_text=ui.button("原文をTXTへ退避（添付登録とは別）").clicked();
                }
            } else {
                ui.add_enabled_ui(can_edit,|ui|self.annotation.ui(ui));
                attach=ui.add_enabled(can_edit,egui::Button::new("この画像と筆跡を入力欄に添付・保存")).clicked();
                export_annotation=ui.button("筆跡・背景・送信画像を新しいフォルダーへ退避").clicked();
            }
            if self.annotation.frozen() || !self.annotation.text.is_empty() {
                if !self.discard_annotation_confirm {
                    if ui.button("未添付の注釈を破棄…").clicked(){self.discard_annotation_confirm=true;}
                } else {
                    ui.label("この原文・筆跡・背景を保存せずに破棄する。この操作は取り消せない。");
                    ui.horizontal(|ui| {
                        if ui.button("注釈を保存せず破棄する").clicked(){self.annotation=Default::default();self.discard_annotation_confirm=false;}
                        if ui.button("注釈を保持する").clicked(){self.discard_annotation_confirm=false;}
                    });
                }
            }
        });
        self.chat_media_open = open;
        if export_audio {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("unsaved-recording.wav")
                .add_filter("WAV", &["wav"])
                .save_file()
            {
                self.message = self
                    .export_unsaved_chat_audio(&path)
                    .map(|_| {
                        format!(
                            "原録音を退避した：{}。チャットへの添付登録は行っていない。",
                            path.display()
                        )
                    })
                    .unwrap_or_else(|e| {
                        format!("WAV退避に失敗した。未保存の録音は保持している：{e}")
                    });
            }
        }
        if export_annotation_text {
            if let Some(dest) = rfd::FileDialog::new()
                .set_file_name("WordWeave5-annotation-original.txt")
                .add_filter("Text", &["txt"])
                .save_file()
            {
                self.message = self
                    .export_chat_annotation_text(&dest)
                    .map(|_| {
                        format!(
                            "原文をTXTへ退避した：{}。チャットへの添付登録は行っていない。",
                            dest.display()
                        )
                    })
                    .unwrap_or_else(|e| {
                        format!("原文のTXT退避に失敗した。入力した原文は保持している：{e}")
                    });
            }
        }
        if export_annotation {
            if let Some(dest) = rfd::FileDialog::new()
                .set_title("新しい退避先フォルダー名を指定")
                .set_file_name("WordWeave5-annotation")
                .save_file()
            {
                self.message=self.export_chat_annotation(&dest).map(|_|format!("筆跡・背景・送信画像を退避した：{}。チャットへの添付登録は行っていない。",dest.display())).unwrap_or_else(|e|format!("注釈の退避は未完了。筆跡は保持している。complete.jsonのないフォルダーは未完了である：{e}"));
            }
        }
        if retry {
            if let Some((id, bytes)) = self.unsaved_chat_audio.take() {
                self.save_chat_recording(&id, bytes);
            }
        }
        if let Some((id, index)) = detach {
            let operation = DiagnosticOperation::begin(DiagnosticEntry::Chat);
            let mut next = self.progress.clone();
            if let Some(chat) = next.chats.iter_mut().find(|c| c.id == id) {
                chat.draft_attachments.remove(index);
            }
            let result = self
                .storage
                .as_ref()
                .ok_or_else(|| "保存先がありません。".to_string())
                .and_then(|s| s.save(&next));
            match result {
                Ok(()) => {
                    operation.event(DiagnosticStage::Save, DiagnosticEvent::Detached);
                    self.progress = next;
                    self.dirty = false;
                    self.message = "このメッセージへの添付を解除した。保存済みの録音・筆跡は削除していない。".into();
                }
                Err(e) => { operation.fail(DiagnosticStage::Save, DiagnosticError::Io); self.message = e; }
            }
        }
        if record {
            if !self.stop_speech() { return; }
            self.recording_cancel_confirm = false;
            let operation = DiagnosticOperation::begin(DiagnosticEntry::Recording);
            match Recorder::start() {
                Ok(r) => {
                    operation.event(DiagnosticStage::Record, DiagnosticEvent::Started);
                    self.recording_operation = Some(operation);
                    self.chat_recording_id = self
                        .progress
                        .chats
                        .get(self.chat_selected)
                        .map(|c| c.id.clone());
                    self.recorder = Some(r);
                }
                Err(e) => { operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable); self.message = e; }
            }
        }
        if let Some((id, reference, original_id)) = recognize {
            self.recognize_chat(id, reference, original_id);
        }
        if let Some(reference) = play {
            self.play_asset(&reference);
        }
        if let Some(reference) = preview {
            self.preview_asset(&reference);
        }
        if attach {
            let result = (|| {
                let id = self
                    .progress
                    .chats
                    .get(self.chat_selected)
                    .ok_or("会話がありません。")?
                    .id
                    .clone();
                let (ink, bg, png) = self.annotation.files()?;
                let store = self.asset_store()?;
                let original = store.put(AssetKind::InkJson, &ink)?;
                let background = Some(store.put(AssetKind::ImagePng, &bg)?);
                let image = Some(store.put(AssetKind::ImagePng, &png)?);
                self.add_chat_attachment(
                    &id,
                    Attachment {
                        original,
                        image,
                        background,
                        source_text: self.annotation.text.clone(),
                        transcript: None,
                    },
                )
            })();
            self.message = match result {
                Ok(()) => {
                    self.annotation = Default::default();
                    "注釈画像と原本を保存した。質問を入力し、添付を確認してから送信する。".into()
                }
                Err(e) => e,
            };
        }
    }
}
