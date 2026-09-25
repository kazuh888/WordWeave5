use super::*;
use wordweave5::{
    assets::{AssetKind, AssetRef, AssetStore, MAX_ASSET_BYTES},
    chat::Attachment,
};

const MAX_PREVIEW_CHARS: usize = 100_000;
const MAX_CHAT_TEXT_BYTES: usize = 32_000;

fn image_display_png(kind: AssetKind, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let format = match kind {
        AssetKind::ImageBmp => image::ImageFormat::Bmp,
        AssetKind::ImageGif => image::ImageFormat::Gif,
        AssetKind::ImageJpeg => image::ImageFormat::Jpeg,
        _ => return Err("表示用画像に変換できない形式です。".into()),
    };
    let (width, height) = image::ImageReader::with_format(
        std::io::Cursor::new(bytes), format,
    ).into_dimensions().map_err(|e| format!("画像の寸法を読めません：{e}"))?;
    if width == 0 || height == 0 || width as u64 * height as u64 > 4_000_000 {
        return Err("画像の寸法が上限を超えています（最大400万画素）。".into());
    }
    let frame = image::load_from_memory_with_format(bytes, format)
        .map_err(|e| format!("画像を読み取れません：{e}"))?;
    let mut png = std::io::Cursor::new(Vec::new());
    frame.write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| format!("表示用画像を作れません：{e}"))?;
    Ok(png.into_inner())
}

fn chat_media_dialog_style(ui: &mut egui::Ui) {
    ux::dialog_body(ui);
    let style = ui.style_mut();
    for (text_style, size) in [
        (egui::TextStyle::Body, 19.0),
        (egui::TextStyle::Small, 16.0),
        (egui::TextStyle::Button, 18.0),
    ] {
        style.text_styles.insert(text_style, super::home_art::home_font(size));
    }
    if let Some(heading) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        heading.size = (heading.size - 2.0).max(1.0);
    }
}

fn text_preview(bytes: &[u8]) -> Option<(String, bool)> {
    if [b"%PDF-".as_slice(), b"PK\x03\x04", b"MZ", b"GIF8", b"\x89PNG\r\n\x1a\n",
        b"\xFF\xD8\xFF", b"BM", b"RIFF"].iter().any(|magic| bytes.starts_with(magic)) {
        return None;
    }
    let decoded = if let Some(raw) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let words = raw.chunks_exact(2);
        if !words.remainder().is_empty() { return None; }
        String::from_utf16(&words.map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect::<Vec<_>>()).ok()?
    } else if let Some(raw) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let words = raw.chunks_exact(2);
        if !words.remainder().is_empty() { return None; }
        String::from_utf16(&words.map(|pair| u16::from_be_bytes([pair[0], pair[1]])).collect::<Vec<_>>()).ok()?
    } else {
        std::str::from_utf8(bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)).ok()?.to_owned()
    };
    if decoded.chars().any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t')) {
        return None;
    }
    let mut chars = decoded.chars();
    let preview: String = chars.by_ref().take(MAX_PREVIEW_CHARS).collect();
    Some((preview, chars.next().is_some()))
}

pub(super) enum AttachmentOpen {
    Image(AssetRef),
    Audio(AssetRef),
    File(AssetRef, String),
}

pub(super) fn attachment_button(ui: &mut egui::Ui, attachment: &Attachment, index: usize) -> Option<AttachmentOpen> {
    let (kind, action) = if let Some(image) = &attachment.image {
        ("画像", AttachmentOpen::Image(image.clone()))
    } else if attachment.original.kind == AssetKind::AudioWav {
        ("音声", AttachmentOpen::Audio(attachment.original.clone()))
    } else if attachment.original.kind == AssetKind::FileBlob {
        ("ファイル", AttachmentOpen::File(
            attachment.original.clone(),
            attachment.file_name.clone().unwrap_or_else(|| "ファイル".into()),
        ))
    } else {
        return None;
    };
    let name = attachment.file_name.as_deref().unwrap_or("");
    let short_name: String = name.chars().take(18).collect();
    let short_name = if name.chars().count() > 18 { format!("{short_name}…") } else { short_name };
    let label = if short_name.is_empty() {
        format!("{kind} {}", index + 1)
    } else {
        format!("{kind} {} · {short_name}", index + 1)
    };
    let response = ui.add(crate::app::controls::Button::new(label)
        .min_size(egui::vec2(112.0, 38.0)))
        .on_hover_text(if name.is_empty() { format!("{kind}を開く") } else { format!("{kind}：{name}") });
    #[cfg(test)]
    ui.ctx().data_mut(|data| data.insert_temp(
        egui::Id::new(("chat-attachment-button", kind, index)), response.rect));
    response.clicked().then_some(action)
}

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
    pub(super) fn attachment_texts(&self, attachments: &[Attachment]) -> Result<Vec<(String, String)>, String> {
        let store = self.asset_store()?;
        let mut texts = Vec::new();
        let mut total = 0;
        for attachment in attachments {
            if attachment.original.kind != AssetKind::FileBlob { continue; }
            attachment.validate()?;
            let bytes = store.read(&attachment.original)?;
            let Some((text, truncated)) = text_preview(&bytes) else { continue; };
            total += text.as_bytes().len();
            if truncated || total > MAX_CHAT_TEXT_BYTES {
                return Err("Codexへ送る添付テキストは合計32KB以下にしてください。大きいファイルはアプリ内で表示できます。".into());
            }
            texts.push((attachment.file_name.clone().unwrap_or_else(|| "ファイル".into()), text));
        }
        Ok(texts)
    }
    fn add_chat_attachment(&mut self, id: &str, attachment: Attachment) -> Result<(), String> {
        let mut next = self.progress.clone();
        let chat = next
            .chats
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or("保存先の会話がありません。")?;
        if chat.deleted_at.is_some() {
            return Err(
                "ごみ箱の会話には添付できない。先に復元するか、原録音を退避してください。".into(),
            );
        }
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
                    file_name: None,
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
    pub(super) fn preview_file_asset(&mut self, reference: &AssetRef, name: &str) {
        self.file_preview = None;
        if reference.kind != AssetKind::FileBlob {
            self.message = "選択した添付はファイルではありません。".into();
            return;
        }
        match self.asset_store().and_then(|store| store.read(reference)) {
            Ok(bytes) => match text_preview(&bytes) {
                Some((text, truncated)) => self.file_preview = Some((name.to_owned(), text, truncated)),
                None => self.message = format!("{name}はテキストファイルではないため、内容を表示しません。"),
            },
            Err(error) => self.message = error,
        }
    }
    pub(super) fn play_asset(&mut self, reference: &AssetRef) {
        if self.pending.is_some() {
            return;
        }
        if let Err(error) = self.check_speech_start() {
            self.message = error;
            return;
        }
        if reference.kind != AssetKind::AudioWav {
            self.message = "選択した添付はWAV音声ではありません。".into();
            return;
        }
        let rate = self.speech_rate();
        let result = self.asset_store().and_then(|s| s.read(reference))
            .and_then(|bytes| self.speaker.play_wav(&bytes, rate));
        match result {
            Ok(()) => {
                if let Some(previous) = self.speech_operation.replace(
                    DiagnosticOperation::begin(DiagnosticEntry::Playback)) {
                    previous.event(DiagnosticStage::Play, DiagnosticEvent::Stopped);
                }
                self.speech_visible = true;
                self.speech_selected = None;
                self.message = "保存した音声を再生中。読み上げパネルで一時停止・再生位置を操作できる。".into();
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
                .order(egui::Order::Foreground)
                .resizable(true)
                .default_width(850.0)
                .show(ctx, |ui| {
                    ux::dialog_body(ui);
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
    pub(super) fn file_preview_window(&mut self, ctx: &egui::Context) {
        if let Some((name, text, truncated)) = &self.file_preview {
            let mut open = true;
            egui::Window::new(format!("添付ファイル：{name}"))
                .open(&mut open)
                .order(egui::Order::Foreground)
                .resizable(true)
                .default_width(760.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    ux::dialog_body(ui);
                    if *truncated {
                        ui.label(format!("表示は先頭{MAX_PREVIEW_CHARS}文字まで。保存した原本は変更していません。"));
                    }
                    egui::ScrollArea::both().max_height(ui.available_height().max(200.0))
                        .show(ui, |ui| { ui.add(egui::Label::new(text.as_str()).selectable(true)); });
                });
            if !open { self.file_preview = None; }
        }
    }

    pub(super) fn chat_file_send_confirmation(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending_chat_file_send.as_ref() else { return; };
        let mut open = true;
        let mut approve = false;
        let mut cancel = false;
        egui::Window::new("添付テキストの送信を確認")
            .open(&mut open).collapsible(false).resizable(false).default_width(580.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.spacing_mut().button_padding.x = 18.0;
                ui.label("次のテキストファイルの本文をCodexへ送信する。よい場合だけ送信を許可してください。");
                for name in &pending.text_names { ui.label(format!("・{name}")); }
                ui.separator();
                ui.label("この発言の添付ファイル名：");
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for label in &pending.labels { ui.label(format!("・{label}")); }
                });
                ui.label("画像は従来どおり送信される。音声と非テキストファイルの本体は送信しない。");
                ui.horizontal_wrapped(|ui| {
                    approve = ui.ww_button("本文をCodexへ送信して質問").clicked();
                    cancel = ui.ww_button("キャンセル").clicked();
                });
            });
        if !open || cancel { self.pending_chat_file_send = None; return; }
        if approve {
            let pending = self.pending_chat_file_send.take().unwrap();
            let unchanged = self.progress.chats.get(self.chat_selected)
                .is_some_and(|chat| pending.matches(chat));
            if unchanged {
                self.launch_chat_with_file_consent(true);
            } else {
                self.message = "確認中に質問または添付が変更された。内容を確認し、もう一度送信してください。".into();
            }
        }
    }

    pub(super) fn attach_chat_file(&mut self, id: &str, path: &std::path::Path) -> Result<(), String> {
        use std::io::Read;
        let name = path.file_name().and_then(|s| s.to_str())
            .ok_or("ファイル名を読み取れません。")?.to_owned();
        Attachment::validate_file_name(&name)?;
        let chat = self.progress.chats.iter().find(|chat| chat.id == id)
            .ok_or("保存先の会話がありません。")?;
        if chat.deleted_at.is_some() { return Err("ごみ箱の会話には添付できません。".into()); }
        if chat.draft_attachments.len() >= 8 { return Err("一つの発言の添付は8件までです。".into()); }
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > MAX_ASSET_BYTES as u64 {
            return Err("添付ファイルは12MiB以下にしてください。".into());
        }
        let mut bytes = Vec::new();
        file.take(MAX_ASSET_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ASSET_BYTES { return Err("添付ファイルは12MiB以下にしてください。".into()); }
        let kind = match path.extension().and_then(|s| s.to_str()).map(str::to_ascii_lowercase).as_deref() {
            Some("png") => AssetKind::ImagePng,
            Some("bmp") => AssetKind::ImageBmp,
            Some("gif") => AssetKind::ImageGif,
            Some("jpg" | "jpeg") => AssetKind::ImageJpeg,
            Some("wav") => AssetKind::AudioWav,
            _ => AssetKind::FileBlob,
        };
        // Keep the selected bytes as the original; Codex and the preview use a
        // bounded PNG of the first frame for formats other than PNG.
        let display_png = match kind {
            AssetKind::ImageBmp | AssetKind::ImageGif | AssetKind::ImageJpeg =>
                Some(image_display_png(kind, &bytes)?),
            _ => None,
        };
        let store = self.asset_store()?;
        let original = store.put(kind, &bytes)?;
        let image = if let Some(png) = display_png {
            Some(store.put(AssetKind::ImagePng, &png)?)
        } else {
            (kind == AssetKind::ImagePng).then(|| original.clone())
        };
        self.add_chat_attachment(id, Attachment {
            original, image, background: None, source_text: name.clone(), transcript: None,
            file_name: Some(name),
        })
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
    pub(super) fn discard_unsaved_media_and_exit(&mut self, ctx: &egui::Context) {
        self.unsaved_chat_audio = None;
        self.annotation = Default::default();
        self.discard_audio_confirm = false;
        self.discard_annotation_confirm = false;
        self.exit_media_requested = false;
        self.exit_media_scroll_to_warning = false;
        self.chat_media_open = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    fn chat_annotation_discard_controls(&mut self, ui: &mut egui::Ui) {
        if !self.annotation.frozen() && self.annotation.text.is_empty() {
            return;
        }
        if !self.discard_annotation_confirm {
            if ui.ww_button("未添付の注釈を破棄…").clicked() {
                self.discard_annotation_confirm = true;
            }
        }
    }

    fn chat_annotation_discard_dialog(&mut self, ctx: &egui::Context) {
        if !self.discard_annotation_confirm {
            return;
        }
        let mut confirmed = false;
        let mut cancelled = false;
        let response = egui::Modal::new(egui::Id::new("chat-annotation-discard-confirm"))
            .show(ctx, |ui| {
                chat_media_dialog_style(ui);
                ui.set_max_width(ctx.available_rect().width().min(500.0) - 24.0);
                ui.heading("未添付の注釈を破棄");
                ui.label("入力した原文・筆跡・背景を保存せずに削除します。この操作は取り消せません。");
                ui.horizontal_wrapped(|ui| {
                    confirmed = ui.add(crate::app::controls::Button::new(
                        RichText::new("OK（破棄する）").color(Color32::WHITE))
                        .fill(Color32::from_rgb(164, 49, 49))).clicked();
                    cancelled = ui.ww_button("キャンセル").clicked();
                });
            });
        if confirmed {
            self.annotation = Default::default();
            self.discard_annotation_confirm = false;
        } else if cancelled || response.should_close() {
            self.discard_annotation_confirm = false;
        }
    }

    fn chat_annotation_controls(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        can_edit: bool,
    ) -> (bool, bool, bool) {
        let (mut attach, mut export_annotation, mut export_text) = (false, false, false);
        let frozen = self.annotation.frozen();
        if !frozen {
            ui.heading("③ 手書きの下地を選ぶ");
            ui.label("③で下地の有無を選んで開始し、④で手書きします。");
            ui.strong("英文を下地にする場合");
            ui.label("下の欄に英文を入力・貼り付け、その英文を固定してから上に書き込みます。");
            ui.add_enabled(can_edit, egui::TextEdit::multiline(&mut self.annotation.text)
                .hint_text("手書きの下地にする英文（任意）")
                .desired_rows(3).desired_width(f32::INFINITY).char_limit(2000));
            if !self.annotation.text.is_empty() {
                export_text = ui.ww_button("原文をTXTへ退避（添付登録とは別）").clicked();
            }
            if ui.add_enabled(can_edit && !self.annotation.text.trim().is_empty(),
                crate::app::controls::Button::new("英文を下地にして手書きを始める")).clicked()
            {
                match self.annotation.freeze(ctx) {
                    Ok(()) => self.chat_media_focus_ink = true,
                    Err(error) => self.message = error,
                }
            }
            ui.separator();
            ui.strong("下地なし（白紙）で書く場合");
            ui.label("英文を入力せず、白紙の上に直接書き込みます。");
            if ui.add_enabled(can_edit && self.annotation.text.is_empty(),
                crate::app::controls::Button::new("白紙で手書きを始める"))
                .on_disabled_hover_text("英文がある場合は、先にTXTへ退避するか、「未添付の注釈を破棄…」で確認してください。").clicked()
            {
                match self.annotation.freeze_blank(ctx) {
                    Ok(()) => self.chat_media_focus_ink = true,
                    Err(error) => self.message = error,
                }
            }
            ui.separator();
            ui.heading("④ 手書きして添付");
            ui.label("③で手書きを始めると、ここに手書き面が表示されます。");
        } else {
            ui.heading("③ 選択した下地");
            ui.label(if self.annotation.text.is_empty() {
                "下地なし（白紙）を選択中。入力欄を重ねず、白紙の手書き面を表示しています。"
            } else {
                "英文を下地として固定済み。入力欄を重ねず、下地の上に書き込みます。"
            });
            let heading = ui.heading("④ 手書きして添付");
            if self.chat_media_focus_ink {
                heading.scroll_to_me(Some(egui::Align::Min));
                self.chat_media_focus_ink = false;
            }
            ui.label("下の画像をドラッグして書き、確認後にメッセージへ添付してください。");
            attach = ui.add_enabled(can_edit,
                crate::app::controls::Button::new("手書き画像をメッセージに添付")).clicked();
            export_annotation = ui.ww_button("筆跡・背景・送信画像を新しいフォルダーへ退避").clicked();
        }
        if !self.exit_media_requested {
            self.chat_annotation_discard_controls(ui);
        }
        if self.annotation.frozen() {
            ui.add_enabled_ui(can_edit, |ui| self.annotation.ui(ui));
        }
        (attach, export_annotation, export_text)
    }
    pub(super) fn chat_media_windows(&mut self, ctx: &egui::Context, idle: bool) {
        let can_edit = idle
            && self.fatal.is_none()
            && self
                .progress
                .chats
                .get(self.chat_selected)
                .is_some_and(|c| c.deleted_at.is_none());
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
        let mut import_file = None;
        let mut preview_file = None;
        let mut keep_exit = false;
        let mut discard_exit = false;
        let screen_rect = ctx.screen_rect();
        let screen = screen_rect.size();
        let dialog_width = (screen.x - 40.0).min(900.0).max(300.0);
        let dialog_height = (screen.y - 80.0).min(1080.0).max(300.0);
        egui::Window::new(RichText::new("ファイル・音声・手書きを添付")
            .font(egui::FontId::new(22.0, egui::FontFamily::Name("heading".into()))))
            .open(&mut open)
            .movable(true)
            .order(egui::Order::Foreground)
            .constrain_to(screen_rect)
            .default_size(egui::vec2(dialog_width, dialog_height))
            .max_width(dialog_width)
            .max_height(dialog_height)
            .default_pos(screen_rect.center() - egui::vec2(dialog_width, dialog_height) * 0.5)
            .vscroll(true).show(ctx,|ui| {
            chat_media_dialog_style(ui);
            if self.exit_media_requested {
                egui::Frame::new()
                    .fill(Color32::from_rgb(255, 244, 239))
                    .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(205, 91, 70)))
                    .corner_radius(8).inner_margin(12).show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let heading = ui.heading("終了前に未保存の作業があります");
                        if self.exit_media_scroll_to_warning {
                            heading.scroll_to_me(Some(egui::Align::Min));
                            self.exit_media_scroll_to_warning = false;
                        }
                        ui.horizontal_wrapped(|ui| {
                            keep_exit = ui.add(crate::app::controls::Button::new("作業を続ける")
                                .min_size(egui::vec2(135.0, 42.0))).clicked();
                            discard_exit = ui.add(crate::app::controls::Button::new(
                                RichText::new("破棄して終了").color(Color32::WHITE))
                                .fill(Color32::from_rgb(164, 49, 49))
                                .min_size(egui::vec2(145.0, 42.0)))
                                .on_hover_text("未保存の録音・原文・筆跡だけを破棄する。保存済みの下書きと添付は残る。")
                                .clicked();
                        });
                        if self.unsaved_chat_audio.is_some() {
                            ui.label("保存できていない録音があります。");
                        }
                        if self.annotation.frozen() || !self.annotation.text.is_empty() {
                            ui.label("送信するメッセージに添付していない手書き・原文があります。");
                        }
                        ui.label("添付・退避して残すか、破棄して終了してください。");
                        ui.horizontal_wrapped(|ui| {
                            if self.unsaved_chat_audio.is_some() {
                                export_audio |= ui.ww_button("録音をWAVへ退避").clicked();
                            }
                            if self.annotation.frozen() {
                                export_annotation |= ui.ww_button("手書き・背景を退避").clicked();
                            } else if !self.annotation.text.is_empty() {
                                export_annotation_text |= ui.ww_button("原文をTXTへ退避").clicked();
                            }
                        });
                        self.chat_annotation_discard_controls(ui);
                    });
                ui.add_space(10.0);
            }
            let table_max_height = (dialog_height - 60.0).max(160.0);
            let table_height = self.chat_media_table_height.clamp(130.0, table_max_height);
            egui::Frame::new()
                .fill(Color32::from_rgb(236, 247, 255))
                .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(177, 207, 233)))
                .inner_margin(10)
                .show(ui, |ui| {
                    // Frame's inner UI already excludes padding AND stroke. Using
                    // the parent width minus padding alone grows the window 2pt/frame.
                    ui.set_width(ui.available_width());
                    ui.set_min_height((table_height - 22.0).max(0.0));
                    let count = self.progress.chats.get(self.chat_selected)
                        .map_or(0, |chat| chat.draft_attachments.len());
                    ui.horizontal_wrapped(|ui| {
                        ui.heading(format!("添付済み（{count}件）"));
                        ui.small("境界をドラッグで高さ変更");
                    });
                    egui::ScrollArea::vertical()
                        .id_salt("chat-media-attached-scroll")
                        .max_height((table_height - 95.0).max(45.0))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Use the scroll content width, after reserving the bar.
                            // columns propagates the widest child's minimum to both
                            // columns; stack actions when that space is unavailable.
                            let compact_table = ui.available_width() < 620.0;
                            egui::Frame::new().fill(Color32::from_rgb(220, 237, 250))
                                .inner_margin(4).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    if compact_table {
                                        ui.strong("種類・名前／確認・削除");
                                    } else {
                                        ui.columns(2, |columns| {
                                            columns[0].strong("種類・名前");
                                            columns[1].strong("確認・削除");
                                        });
                                    }
                                });
                            if let Some(chat) = self.progress.chats.get(self.chat_selected) {
                                if chat.draft_attachments.is_empty() {
                                    egui::Frame::new().fill(Color32::WHITE).inner_margin(4)
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            if compact_table {
                                                ui.label("まだ添付はありません。下から追加できます。");
                                            } else {
                                                ui.columns(2, |columns| {
                                                    columns[0].label("まだ添付はありません。");
                                                    columns[1].label("下から追加できます。");
                                                });
                                            }
                                        });
                                }
                                for (i, a) in chat.draft_attachments.iter().enumerate() {
                                    ui.push_id(i, |ui| {
                                        egui::Frame::new()
                                            .fill(if i % 2 == 0 { Color32::WHITE }
                                                else { Color32::from_rgb(245, 250, 255) })
                                            .inner_margin(4)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                let kind = match a.original.kind {
                                                    AssetKind::AudioWav => "音声",
                                                    AssetKind::ImagePng | AssetKind::ImageBmp |
                                                    AssetKind::ImageGif | AssetKind::ImageJpeg => "画像",
                                                    AssetKind::InkJson => "手書き画像",
                                                    AssetKind::FileBlob => "ファイル",
                                                };
                                                let name = a.file_name.clone().unwrap_or_else(|| {
                                                    if a.original.kind == AssetKind::InkJson
                                                        && !a.source_text.trim().is_empty()
                                                    {
                                                        format!("原文：{}", a.source_text.chars().take(40).collect::<String>())
                                                    } else {
                                                        kind.to_owned()
                                                    }
                                                });
                                                let mut actions = |ui: &mut egui::Ui| {
                                                    if a.original.kind == AssetKind::AudioWav
                                                        && ui.add_enabled(idle, crate::app::controls::Button::new("再生")).clicked()
                                                    {
                                                        play = Some(a.original.clone());
                                                    }
                                                    if let Some(image) = &a.image {
                                                        if ui.ww_button("画像を見る").clicked() {
                                                            preview = Some(image.clone());
                                                        }
                                                    }
                                                    if a.original.kind == AssetKind::FileBlob
                                                        && ui.ww_button("内容を見る").clicked()
                                                    {
                                                        preview_file = Some((a.original.clone(),
                                                            a.file_name.clone().unwrap_or_else(|| "ファイル".into())));
                                                    }
                                                    if ui.add_enabled(can_edit, crate::app::controls::Button::new("添付から削除"))
                                                        .on_hover_text("このメッセージへの添付を解除します。原本の録音・筆跡と、入力欄へ反映済みの文章は削除しません。")
                                                        .clicked()
                                                    {
                                                        detach = Some((chat.id.clone(), i));
                                                    }
                                                    if a.original.kind == AssetKind::AudioWav
                                                        && ui.add_enabled(can_edit, crate::app::controls::Button::new("Codexで文字起こし")).clicked()
                                                    {
                                                        recognize = Some((chat.id.clone(), a.original.clone(), a.original.id.clone()));
                                                    }
                                                    if let Some(image) = &a.image {
                                                        if ui.add_enabled(can_edit, crate::app::controls::Button::new("Codexで文字認識"))
                                                            .on_hover_text("画像をCodexへ送って文字を読み取り、入力欄へ反映します。")
                                                            .clicked()
                                                        {
                                                            recognize = Some((chat.id.clone(), image.clone(), a.original.id.clone()));
                                                        }
                                                    }
                                                };
                                                if compact_table {
                                                    ui.strong(format!("{}．{kind}", i + 1));
                                                    ui.add(egui::Label::new(name.as_str()).wrap())
                                                        .on_hover_text(name.as_str());
                                                    ui.horizontal_wrapped(|ui| actions(ui));
                                                    if let Some(transcript) = &a.transcript {
                                                        ui.ww_collapsing("元の認識結果（訂正前）", |ui| {
                                                            ui.label(transcript);
                                                        });
                                                    }
                                                } else {
                                                    ui.columns(2, |columns| {
                                                        columns[0].strong(format!("{}．{kind}", i + 1));
                                                        columns[0].add(egui::Label::new(name.as_str()).wrap())
                                                            .on_hover_text(name.as_str());
                                                        columns[1].horizontal_wrapped(|ui| actions(ui));
                                                        if let Some(transcript) = &a.transcript {
                                                            columns[1].ww_collapsing("元の認識結果（訂正前）", |ui| {
                                                                ui.label(transcript);
                                                            });
                                                        }
                                                    });
                                                }
                                            });
                                    });
                                }
                            }
                        });
                });
            let (handle_rect, handle) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 14.0), egui::Sense::drag());
            #[cfg(test)]
            ui.ctx().data_mut(|data| data.insert_temp(
                egui::Id::new("chat-media-table-resize-handle"), handle_rect));
            if handle.dragged() {
                self.chat_media_table_height = (table_height + handle.drag_delta().y)
                    .clamp(130.0, table_max_height);
            }
            handle.on_hover_cursor(egui::CursorIcon::ResizeVertical);
            ui.painter().rect_filled(handle_rect, 3.0, Color32::from_rgb(218, 230, 242));
            ui.painter().rect_filled(
                egui::Rect::from_center_size(handle_rect.center(), egui::vec2(72.0, 4.0)),
                2.0, Color32::from_rgb(98, 139, 175));
            let add_height = (dialog_height - table_height - 85.0).max(115.0);
            egui::Frame::new()
                .fill(Color32::WHITE)
                .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(207, 222, 236)))
                .inner_margin(10)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::vertical()
                        .id_salt("chat-media-add-scroll")
                        .max_height(add_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
            ui.heading("添付を追加");
            ui.label("送信するメッセージにファイル・録音・手書き画像を追加します。ここで追加しても送信されず、チャット画面の「送信」で送ります。");
            if self.fatal.is_some() { ui.colored_label(Color32::RED,"保存障害のため通常の入力・添付登録は停止中。未保存の録音・筆跡は、この画面から別ファイルへ退避できる。"); }
            if self.fatal.is_none() {
                ui.heading("① ファイルを選ぶ");
                ui.label("画像（PNG・BMP・GIF・JPEG）・音声・文書を追加。GIFは最初の静止画を表示・送信し、元のGIFは保存します。テキスト本文をCodexへ送る際は、送信前にファイル名を示して許可を求めます。");
                if ui.add_enabled(can_edit, crate::app::controls::Button::new("ファイルを選んで添付")).clicked() {
                    import_file = self.progress.chats.get(self.chat_selected).map(|chat| chat.id.clone());
                }
            }
            ui.separator();
            ui.heading("② 声で伝える");
            ui.label("録音を添付。文字起こしは原音をCodexへ送るため、認識結果を確認してください。");
            if self.recorder.is_some() {
                self.recording_controls(ui, "録音を終了して保存");
            } else {record=ui.add_enabled(can_edit && self.unsaved_chat_audio.is_none(),crate::app::controls::Button::new("録音を開始（最大30秒）")).clicked();}
            if self.unsaved_chat_audio.is_some() {
                ui.colored_label(Color32::RED,"未保存の録音がある。閉じると失われるため、再保存またはWAV退避する。");
                retry=ui.add_enabled(can_edit,crate::app::controls::Button::new("録音の保存を再試行")).clicked();
                export_audio=ui.ww_button("WAVへ退避（添付登録とは別）").clicked();
                if !self.discard_audio_confirm {
                    if ui.ww_button("未保存の録音を破棄…").clicked(){self.discard_audio_confirm=true;}
                } else {
                    ui.label("この録音を保存せずに破棄する。この操作は取り消せない。");
                    ui.horizontal(|ui| {
                        if ui.ww_button("録音を保存せず破棄する").clicked(){self.unsaved_chat_audio=None;self.discard_audio_confirm=false;}
                        if ui.ww_button("録音を保持する").clicked(){self.discard_audio_confirm=false;}
                    });
                }
            }
            ui.separator();
            let (a, e, t) = self.chat_annotation_controls(ui, ctx, can_edit);
            attach |= a;
            export_annotation |= e;
            export_annotation_text |= t;
                        });
                });
        });
        self.chat_media_open = open;
        self.chat_annotation_discard_dialog(ctx);
        if keep_exit || !open {
            self.exit_media_requested = false;
            self.exit_media_scroll_to_warning = false;
        }
        if discard_exit {
            self.discard_unsaved_media_and_exit(ctx);
            return;
        }
        if let Some(id) = import_file {
            if let Some(path) = rfd::FileDialog::new().set_title("チャットに添付するファイルを選択").pick_file() {
                self.message = self.attach_chat_file(&id, &path)
                    .map(|_| format!("{}を添付した。送信前に内容を確認できる。", path.file_name().and_then(|s| s.to_str()).unwrap_or("ファイル")))
                    .unwrap_or_else(|e| format!("ファイルを添付できなかった：{e}"));
            }
        }
        if let Some((reference, name)) = preview_file {
            self.preview_file_asset(&reference, &name);
        }
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
                    self.message =
                        "このメッセージへの添付を解除した。保存済みの録音・筆跡は削除していない。"
                            .into();
                }
                Err(e) => {
                    operation.fail(DiagnosticStage::Save, DiagnosticError::Io);
                    self.message = e;
                }
            }
        }
        if record {
            if !self.stop_speech() {
                return;
            }
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
                Err(e) => {
                    operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable);
                    self.message = e;
                }
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
                        file_name: None,
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
        if self.exit_media_requested
            && self.unsaved_chat_audio.is_none()
            && !self.annotation.frozen()
            && self.annotation.text.is_empty()
        {
            self.exit_media_requested = false;
            self.exit_media_scroll_to_warning = false;
            self.chat_media_open = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

#[cfg(test)]
mod attachment_tests {
    use super::*;

    #[test]
    fn media_dialog_text_is_two_points_smaller_than_other_dialogs() {
        let (ctx, app, root) = super::super::harness_tests::fixture();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                chat_media_dialog_style(ui);
                for (style, size) in [
                    (egui::TextStyle::Body, 19.0),
                    (egui::TextStyle::Small, 16.0),
                    (egui::TextStyle::Button, 18.0),
                    (egui::TextStyle::Heading, 22.0),
                ] {
                    assert_eq!(ui.style().text_styles[&style].size, size);
                }
            });
        });
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn text_preview_accepts_utf8_and_utf16_but_rejects_binary() {
        assert_eq!(text_preview(b"hello\nworld"), Some(("hello\nworld".into(), false)));
        assert_eq!(text_preview(&[0xFF, 0xFE, b'A', 0, b'B', 0]), Some(("AB".into(), false)));
        assert!(text_preview(b"PDF\0binary").is_none());
        assert!(text_preview(b"%PDF-1.4\n1 0 obj\nendobj").is_none());
        assert!(text_preview(&[0xFF, 0xFE, b'A']).is_none());
    }

    #[test]
    fn selected_file_is_saved_with_the_draft_and_binary_is_not_previewed() {
        let (_ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        let text_path = root.join("sample.txt");
        std::fs::write(&text_path, "添付テキスト\nsecond line").unwrap();
        app.attach_chat_file(&id, &text_path).unwrap();
        let attachment = app.progress.chats[0].draft_attachments[0].clone();
        assert_eq!(attachment.original.kind, AssetKind::FileBlob);
        assert_eq!(attachment.file_name.as_deref(), Some("sample.txt"));
        app.preview_file_asset(&attachment.original, "sample.txt");
        assert_eq!(app.file_preview.as_ref().map(|(_, text, _)| text.as_str()), Some("添付テキスト\nsecond line"));

        let binary_path = root.join("sample.pdf");
        std::fs::write(&binary_path, b"PDF\0binary").unwrap();
        app.attach_chat_file(&id, &binary_path).unwrap();
        let binary = app.progress.chats[0].draft_attachments[1].clone();
        app.preview_file_asset(&binary.original, "sample.pdf");
        assert!(app.file_preview.is_none());
        assert!(app.message.contains("表示しません"));
        let reloaded = app.storage.as_ref().unwrap().load().unwrap();
        assert_eq!(reloaded.chats[0].draft_attachments.len(), 2);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bmp_gif_and_jpeg_keep_originals_and_open_as_images() {
        let (_ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        let source = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            2, 2, image::Rgb([22, 88, 190])));
        let store = app.asset_store().unwrap();
        for (extension, kind, format) in [
            ("bmp", AssetKind::ImageBmp, image::ImageFormat::Bmp),
            ("gif", AssetKind::ImageGif, image::ImageFormat::Gif),
            ("jpg", AssetKind::ImageJpeg, image::ImageFormat::Jpeg),
            ("jpeg", AssetKind::ImageJpeg, image::ImageFormat::Jpeg),
        ] {
            let mut encoded = std::io::Cursor::new(Vec::new());
            source.write_to(&mut encoded, format).unwrap();
            let bytes = encoded.into_inner();
            let path = root.join(format!("picture.{extension}"));
            std::fs::write(&path, &bytes).unwrap();
            app.attach_chat_file(&id, &path).unwrap();
            let attachment = app.progress.chats[0].draft_attachments.last().unwrap().clone();
            assert_eq!(attachment.original.kind, kind);
            assert_eq!(attachment.file_name.as_deref(), Some(path.file_name().unwrap().to_str().unwrap()));
            assert_eq!(store.read(&attachment.original).unwrap(), bytes);
            let preview = attachment.image.as_ref().expect("display PNG");
            assert_eq!(preview.kind, AssetKind::ImagePng);
            assert!(store.read(preview).unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
            app.preview_pixels = None;
            app.preview_asset(preview);
            assert_eq!(app.preview_pixels.as_ref().map(|(name, _)| name.as_str()), Some(preview.id.as_str()));
            assert!(app.attachment_texts(&[attachment.clone()]).unwrap().is_empty());
            assert_eq!(app.attachment_images(&[attachment]).unwrap().len(), 1);
        }
        let prior = serde_json::to_value(&app.progress).unwrap();
        let assets_before = store.inventory().unwrap().len();
        let corrupt = root.join("broken.gif");
        std::fs::write(&corrupt, b"GIF89aBROKEN").unwrap();
        assert!(app.attach_chat_file(&id, &corrupt).is_err());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), prior);
        assert_eq!(store.inventory().unwrap().len(), assets_before);
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().chats[0].draft_attachments.len(), 4);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn text_attachment_waits_for_named_user_consent_before_chat_request() {
        let (_ctx, mut app, root) = super::super::harness_tests::fixture();
        let id = app.progress.chats[0].id.clone();
        let path = root.join("reading-note.txt");
        std::fs::write(&path, "Please explain this phrase.").unwrap();
        app.attach_chat_file(&id, &path).unwrap();
        let audio = app.asset_store().unwrap().put(AssetKind::AudioWav,
            b"RIFF\x04\x00\x00\x00WAVE").unwrap();
        app.add_chat_attachment(&id, Attachment { original: audio, image: None,
            background: None, file_name: None, source_text: "録音".into(), transcript: None }).unwrap();
        app.progress.chats[0].draft = "この文章を解説して".into();
        let progress_before = serde_json::to_value(&app.progress).unwrap();
        app.launch_chat();
        let consent = app.pending_chat_file_send.as_ref().expect("send requires consent");
        assert_eq!(consent.text_names, ["reading-note.txt"]);
        assert!(consent.labels.iter().any(|line| line.contains("reading-note.txt")));
        assert!(consent.matches(&app.progress.chats[0]), "mixed named/unnamed attachments remain approvable");
        let mut changed = app.progress.chats[0].clone();
        changed.draft.push('!');
        assert!(!consent.matches(&changed));
        let mut changed = app.progress.chats[0].clone();
        changed.draft_attachments[0].file_name = Some("other.txt".into());
        assert!(!consent.matches(&changed));
        let mut changed = app.progress.chats[0].clone();
        changed.draft_attachments[0].original.id = "a".repeat(64);
        assert!(!consent.matches(&changed));
        let mut changed = app.progress.chats[0].clone();
        changed.id = "different-chat".into();
        assert!(!consent.matches(&changed));
        assert!(app.pending.is_none(), "Codex must not start before consent");
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), progress_before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn approved_text_payload_has_named_content_and_respects_context_limit() {
        let mut payload = serde_json::json!({"current_question": "Explain this."});
        assert!(payload.get("current_text_files").is_none());
        super::super::add_chat_text_files(&mut payload,
            &[("reading-note.txt".into(), "a phrase in context".into())]).unwrap();
        assert_eq!(payload["current_text_files"][0]["name"], "reading-note.txt");
        assert_eq!(payload["current_text_files"][0]["content"], "a phrase in context");
        let mut oversized = serde_json::json!({"current_question": "x".repeat(wordweave5::chat::CONTEXT_BYTES)});
        assert!(super::super::add_chat_text_files(&mut oversized,
            &[("reading-note.txt".into(), "text".into())]).is_err());
    }

    #[test]
    fn attachment_button_identifies_each_kind_and_file_click_opens_the_file_action() {
        let make_ref = |kind| AssetRef { id: "a".repeat(64), kind, bytes: 16 };
        let image = make_ref(AssetKind::ImagePng);
        let items = [
            Attachment { original: image.clone(), image: Some(image), background: None,
                source_text: "画像".into(), transcript: None, file_name: Some("photo.png".into()) },
            Attachment { original: make_ref(AssetKind::AudioWav), image: None, background: None,
                source_text: "録音".into(), transcript: None, file_name: Some("voice.wav".into()) },
            Attachment { original: make_ref(AssetKind::FileBlob), image: None, background: None,
                source_text: "資料".into(), transcript: None, file_name: Some("notes.txt".into()) },
        ];
        let ctx = egui::Context::default();
        let frame = |events| {
            let mut opened = None;
            let output = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 300.0))),
                events, ..Default::default()
            }, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (index, attachment) in items.iter().enumerate() {
                            if let Some(action) = attachment_button(ui, attachment, index) { opened = Some(action); }
                        }
                    });
                });
            });
            (output, opened)
        };
        for _ in 0..2 { frame(vec![]); }
        let pos = ctx.data(|data| data.get_temp::<egui::Rect>(
            egui::Id::new(("chat-attachment-button", "ファイル", 2)))).unwrap().center();
        frame(vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed: true,
            modifiers: egui::Modifiers::NONE,
        }]);
        let (output, opened) = frame(vec![egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(matches!(opened, Some(AttachmentOpen::File(_, name)) if name == "notes.txt"));
        let mut text = String::new();
        for shape in output.shapes {
            super::super::harness_tests::shape_text(&shape.shape, &mut text);
        }
        assert!(text.contains("画像 1") && text.contains("音声 2") && text.contains("ファイル 3"), "{text}");
    }
}
