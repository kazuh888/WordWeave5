use super::*;

impl WordApp {
    pub(super) fn reset_chat_view_after_restore(&mut self) {
        self.chat_selected = wordweave5::chat::ordered_indices(&self.progress.chats).first().copied().unwrap_or(usize::MAX);
        self.chat_context_open = false;
        self.chat_material_open = false;
        self.chat_media_open = false;
        self.pending_chat_delete = None;
        self.viewed_trash_id = None;
        self.effort_catalog = None;
    }

    fn chat_change_allowed(&self) -> Result<(), String> {
        if self.fatal.is_some() || self.pending.is_some() || self.recorder.is_some()
            || self.session.is_some() || self.batch_running || self.pending_import.is_some()
            || self.pending_restore.is_some() || self.backup_restore.is_some() {
            return Err("処理を終了してから会話を操作してください。".into());
        }
        if self.unsaved_chat_audio.is_some() || self.annotation.frozen() || !self.annotation.text.is_empty() {
            return Err("未保存の録音・手書き注釈を先に添付、退避または破棄してください。".into());
        }
        Ok(())
    }

    pub(super) fn create_conversation(&mut self) -> Result<(), String> {
        self.chat_change_allowed()?;
        let mut next = self.progress.clone();
        next.chats.push(wordweave5::chat::Conversation::new());
        self.storage.as_ref().ok_or("保存先がありません。")?.save(&next)?;
        self.chat_selected = next.chats.len() - 1;
        self.progress = next;
        self.dirty = false;
        Ok(())
    }

    pub(super) fn set_conversation_deleted(&mut self, id: &str, deleted: bool) -> Result<(), String> {
        self.chat_change_allowed()?;
        let operation = DiagnosticOperation::begin(DiagnosticEntry::Chat);
        let mut next = self.progress.clone();
        next.set_chat_deleted(id, deleted).map_err(|error| {
            operation.fail(DiagnosticStage::Edit, DiagnosticError::InvalidData); error
        })?;
        self.storage.as_ref().ok_or("保存先がありません。")?.save(&next).map_err(|error| {
            operation.fail(DiagnosticStage::Save, DiagnosticError::Io); error
        })?;
        operation.event(DiagnosticStage::Save, if deleted { DiagnosticEvent::Deleted } else { DiagnosticEvent::Restored });
        self.progress = next;
        self.dirty = false;
        self.last_save = Instant::now();
        if deleted {
            self.chat_selected = wordweave5::chat::ordered_indices(&self.progress.chats)
                .first().copied().unwrap_or(usize::MAX);
            self.chat_context_open = false;
            self.chat_material_open = false;
            self.chat_media_open = false;
        } else if let Some(index) = self.progress.chats.iter().position(|c| c.id == id) {
            self.chat_selected = index;
            self.viewed_trash_id = None;
        }
        self.message = if deleted { "会話をごみ箱へ移動した。教材・復習記録・媒体原本は保持している。" }
            else { "会話を復元した。" }.into();
        Ok(())
    }

    pub(super) fn conversation_trash_windows(&mut self, ctx: &egui::Context) {
        let allowed = self.chat_change_allowed().is_ok();
        if let Some(id) = self.pending_chat_delete.clone() {
            let title = self.progress.chats.iter().find(|c| c.id == id).map(|c| c.title.as_str()).unwrap_or("対象の会話");
            let (mut apply, mut cancel) = (false, false);
            egui::Window::new("会話の削除を確認").collapsible(false).show(ctx, |ui| {
                ui.label(format!("「{title}」をごみ箱へ移動する。"));
                ui.label("復元可能。教材・復習記録・媒体原本・Codex側の履歴は削除しない。");
                if let Err(reason) = self.chat_change_allowed() { ui.label(reason); }
                ui.horizontal(|ui| {
                    apply = ui.add_enabled(allowed, egui::Button::new("ごみ箱へ移動")).clicked();
                    cancel = ui.button("キャンセル").clicked();
                });
            });
            if apply {
                match self.set_conversation_deleted(&id, true) {
                    Ok(()) => self.pending_chat_delete = None,
                    Err(error) => { self.message = error; self.pending_chat_delete = None; }
                }
            } else if cancel { self.pending_chat_delete = None; }
        }

        let mut open = self.conversation_trash_open;
        let mut restore = None;
        egui::Window::new("チャットのごみ箱").open(&mut open).default_width(580.0).vscroll(true).show(ctx, |ui| {
            ui.label("会話は自動削除しない。復元すると一覧と送信対象に戻る。");
            let mut chats: Vec<_> = self.progress.chats.iter().filter(|c| c.deleted_at.is_some()).collect();
            chats.sort_by_key(|c| std::cmp::Reverse(c.deleted_at));
            if chats.is_empty() { ui.label("ごみ箱は空である。"); }
            for chat in chats {
                ui.push_id(&chat.id, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&chat.title);
                        if ui.button("内容を見る").clicked() { self.viewed_trash_id = Some(chat.id.clone()); }
                        if ui.add_enabled(allowed, egui::Button::new("復元")).clicked() { restore = Some(chat.id.clone()); }
                    });
                    ui.separator();
                });
            }
        });
        self.conversation_trash_open = open;
        if let Some(id) = restore { if let Err(error) = self.set_conversation_deleted(&id, false) { self.message = error; } }

        if let Some(id) = self.viewed_trash_id.clone() {
            let mut open = true;
            let mut preview = None;
            let mut play = None;
            egui::Window::new("削除した会話（読み取り専用）").open(&mut open).default_width(720.0).vscroll(true).show(ctx, |ui| {
                let Some(chat) = self.progress.chats.iter().find(|c| c.id == id) else { ui.label("会話が見つからない。"); return; };
                ui.heading(&chat.title);
                ui.label("教材の根拠と媒体原本は保持している。編集・送信する場合は復元する。");
                if !chat.memo.is_empty() { ui.label(format!("引き継ぎメモ：{}", chat.memo)); }
                if !chat.draft.is_empty() { ui.label(format!("未送信の下書き：{}", chat.draft)); }
                for (index, exchange) in chat.exchanges.iter().enumerate() {
                    ui.separator();
                    ui.strong(format!("往復 {}：あなた", index + 1)); ui.label(&exchange.question);
                    ui.strong("Codex"); ui.label(&exchange.answer); ui.small(exchange.execution.label());
                }
                ui.separator(); ui.strong("保存済みの添付（下書き・履歴）");
                for (index, attachment) in chat.draft_attachments.iter().chain(chat.exchanges.iter().flat_map(|e| &e.attachments)).enumerate() {
                    ui.push_id(index, |ui| {
                        ui.label(&attachment.source_text);
                        if let Some(text) = &attachment.transcript { ui.label(text); }
                        if attachment.original.kind == wordweave5::assets::AssetKind::AudioWav
                            && ui.add_enabled(self.pending.is_none() && self.recorder.is_none(), egui::Button::new("原録音を再生")).clicked() { play = Some(attachment.original.clone()); }
                        if let Some(image) = &attachment.image { if ui.button("注釈画像").clicked() { preview = Some(image.clone()); } }
                    });
                }
            });
            if !open { self.viewed_trash_id = None; }
            if let Some(reference) = preview { self.preview_asset(&reference); }
            if let Some(reference) = play { self.play_asset(&reference); }
        }
    }
}
