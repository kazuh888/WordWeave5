use super::*;
use wordweave5::backup;

impl WordApp {
    pub(super) fn backup_controls(&mut self, ui: &mut egui::Ui, idle: bool) {
        ui.separator();
        ui.strong("教材・学習記録・録音・筆跡をまとめて退避");
        ui.small("原本は添付から外したものも含む。実行記録・Codex認証情報・処理待ちの語彙一覧はこのバックアップに含めない。");
        ui.add_enabled_ui(idle,|ui|ui.horizontal_wrapped(|ui|{
            if ui.ww_button("媒体を含めてバックアップ").clicked() {
                if let Some(parent)=rfd::FileDialog::new().pick_folder() {
                    let dest=parent.join(format!("WordWeave5-backup-{}",chrono::Utc::now().timestamp_millis()));
                    let result=self.storage.as_ref().ok_or_else(||"保存先がありません。".to_string())
                        .and_then(|s|backup::export(&s.dir,&dest,&self.deck,&self.progress));
                    self.message=match result {Ok(())=>format!("バックアップを作成した：{}",dest.display()),Err(e)=>format!("バックアップ未完了。完了マーカーのないフォルダーは復元に使用しない：{e}")};
                }
            }
            if ui.ww_button("媒体付きバックアップを復元").clicked() {
                if let Some(path)=rfd::FileDialog::new().pick_folder() {
                    match backup::prepare(path) {Ok(prepared)=>self.backup_restore=Some(prepared),Err(e)=>self.message=e}
                }
            }
        }));
    }
    pub(super) fn backup_confirmation(&mut self, ctx: &egui::Context) {
        let Some(prepared) = &self.backup_restore else {
            return;
        };
        let mut commit = false;
        let mut cancel = false;
        egui::Window::new("媒体付きバックアップの復元を確認").collapsible(false).show(ctx,|ui|{
            ux::dialog_body(ui);
            if self.settings_changed() { ui.label("未保存の設定変更も破棄し、バックアップの設定へ戻す。"); }
            ui.label(format!("教材{}件・会話{}件・原本{}ファイルで、現在の教材と学習記録を置き換える。",prepared.deck.len(),prepared.progress.chats.len(),prepared.media_count()));
            ui.label("現在の教材・学習記録はbackupsへ退避する。現在の媒体原本は削除しない。Codexへの通信は発生しない。");
            let restore = ui.ww_button("内容を確認して復元する");
            let dismiss = ui.ww_button("キャンセル");
            #[cfg(test)]
            ctx.data_mut(|data| data.insert_temp(egui::Id::new("backup-confirm-actions"), (restore.rect, dismiss.rect)));
            commit = restore.clicked();
            cancel = dismiss.clicked();
        });
        if cancel {
            self.backup_restore = None;
        }
        if commit {
            if let (Some(prepared), Some(storage)) =
                (self.backup_restore.take(), self.storage.as_ref())
            {
                match prepared.restore(storage, &self.progress) {
                    Ok((deck, progress)) => {
                        self.deck = deck;
                        self.progress = progress;
                        self.dirty = false;
                        self.fatal = None;
                        self.reset_chat_view_after_restore();
                        self.message = "媒体を含む教材・学習記録を復元した。".into();
                    }
                    Err(e) => {
                        if wordweave5::commit::pending(&storage.dir) {
                            self.fatal =
                                Some("復元の保存が途中で停止した。再起動で復旧する。".into());
                        }
                        self.message = e;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::harness_tests::fixture;

    fn confirm(ctx: &egui::Context, app: &mut WordApp, accept: bool) {
        let mut draw = |events| {
            let _ = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1150.0, 950.0))),
                events, ..Default::default()
            }, |ctx| app.backup_confirmation(ctx));
        };
        for _ in 0..3 { draw(vec![]); }
        let (yes, no) = ctx.data(|data| data.get_temp::<(egui::Rect, egui::Rect)>(egui::Id::new("backup-confirm-actions"))).unwrap();
        let pos = if accept { yes.center() } else { no.center() };
        for pressed in [true, false] {
            draw(vec![egui::Event::PointerMoved(pos), egui::Event::PointerButton {
                pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE
            }]);
        }
    }

    #[test]
    fn backup_confirmation_cancel_restore_restart_and_corruption_preserve_boundaries() {
        let (ctx, mut app, root) = fixture();
        let source = app.storage.as_ref().unwrap().dir.clone();
        let destination = root.join("roundtrip-backup");
        let assets = wordweave5::assets::AssetStore::new(source.clone());
        let original = assets.put(wordweave5::assets::AssetKind::AudioWav, b"RIFF\x04\0\0\0WAVE").unwrap();
        app.progress.settings.minutes = 7;
        app.progress.chats[0].draft = "バックアップ時の会話".into();
        app.progress.study_seconds.insert("2026-09-26".into(), 321);
        app.storage.as_ref().unwrap().save(&app.progress).unwrap();
        backup::export(&source, &destination, &app.deck, &app.progress).unwrap();
        let saved = app.progress.clone();
        let deck = model::deck_text(&app.deck);
        app.progress.chats[0].draft = "復元直前の未保存会話".into();
        app.progress.settings.minutes = 11;
        app.begin_settings_edit();
        app.settings_editor.draft.minutes = 23;
        let before = serde_json::to_value(&app.progress).unwrap();
        app.backup_restore = Some(backup::prepare(destination.clone()).unwrap());
        confirm(&ctx, &mut app, false);
        assert!(app.backup_restore.is_none());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(app.settings_editor.draft.minutes, 23);
        assert_eq!(app.storage.as_ref().unwrap().load().unwrap().settings.minutes, 7);

        app.backup_restore = Some(backup::prepare(destination.clone()).unwrap());
        confirm(&ctx, &mut app, true);
        assert!(app.backup_restore.is_none());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), serde_json::to_value(&saved).unwrap());
        assert_eq!(model::deck_text(&app.deck), deck);
        app.begin_settings_edit();
        assert_eq!(app.settings_editor.draft.minutes, 7);
        assert!(!app.settings_changed());
        assert_eq!(assets.read(&original).unwrap(), b"RIFF\x04\0\0\0WAVE");
        let snapshots: Vec<_> = std::fs::read_dir(source.join("backups")).unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap().to_string_lossy().starts_with("progress-memory-before-restore-")).collect();
        assert_eq!(snapshots.len(), 1);
        let snapshot: Progress = serde_json::from_slice(&std::fs::read(&snapshots[0]).unwrap()).unwrap();
        assert_eq!(snapshot.chats[0].draft, "復元直前の未保存会話");
        assert_eq!(snapshot.settings.minutes, 11); // never the uncommitted settings draft

        std::fs::write(destination.join("progress.json"), b"corrupt").unwrap();
        assert!(backup::prepare(destination).is_err());
        drop(app);
        let reopened = WordApp::new_with_storage(&egui::Context::default(), Storage::at(source));
        assert_eq!(reopened.progress.settings.minutes, 7);
        assert_eq!(reopened.progress.chats[0].draft, saved.chats[0].draft);
        assert_eq!(reopened.progress.study_seconds, saved.study_seconds);
        assert_eq!(model::deck_text(&reopened.deck), deck);
        drop(reopened);
        std::fs::remove_dir_all(root).unwrap();
    }
}
