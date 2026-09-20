use super::*;
use wordweave5::backup;

impl WordApp {
    pub(super) fn backup_controls(&mut self, ui: &mut egui::Ui, idle: bool) {
        ui.separator();
        ui.strong("教材・学習記録・録音・筆跡をまとめて退避");
        ui.small("原本は添付から外したものも含む。実行記録・Codex認証情報・処理待ちの語彙一覧はこのバックアップに含めない。");
        ui.add_enabled_ui(idle,|ui|ui.horizontal_wrapped(|ui|{
            if ui.button("媒体を含めてバックアップ").clicked() {
                if let Some(parent)=rfd::FileDialog::new().pick_folder() {
                    let dest=parent.join(format!("WordWeave5-backup-{}",chrono::Utc::now().timestamp_millis()));
                    let result=self.storage.as_ref().ok_or_else(||"保存先がありません。".to_string())
                        .and_then(|s|backup::export(&s.dir,&dest,&self.deck,&self.progress));
                    self.message=match result {Ok(())=>format!("バックアップを作成した：{}",dest.display()),Err(e)=>format!("バックアップ未完了。完了マーカーのないフォルダーは復元に使用しない：{e}")};
                }
            }
            if ui.button("媒体付きバックアップを復元").clicked() {
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
            ui.label(format!("教材{}件・会話{}件・原本{}ファイルで、現在の教材と学習記録を置き換える。",prepared.deck.len(),prepared.progress.chats.len(),prepared.media_count()));
            ui.label("現在の教材・学習記録はbackupsへ退避する。現在の媒体原本は削除しない。Codexへの通信は発生しない。");
            commit=ui.button("内容を確認して復元する").clicked();cancel=ui.button("キャンセル").clicked();
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
