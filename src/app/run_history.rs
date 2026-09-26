use super::*;
use wordweave5::{
    codex,
    run_journal::{self, RunRecord},
};

impl WordApp {
    pub(super) fn run_history(&mut self, ctx: &egui::Context) {
        let mut open = self.run_history_open;
        egui::Window::new("Codex実行記録・結果の再取得").open(&mut open).default_width(820.0).show(ctx, |ui| {
            ux::dialog_body(ui);
            ui.label("生成完了と教材への登録は別である。再取得は本文の確認用であり、再生成・再登録は行わない。");
            ui.small("ここにはプロンプトと応答をローカル保存する。秘密の情報を含む会話はバックアップの扱いにも注意する。");
            ui.small("新しい200件まで表示する。古い実行記録のファイルは削除しない。");
            if ui.add_enabled(self.pending.is_none(), crate::app::controls::Button::new("実行記録を更新")).clicked() {
                self.refresh_runs();
            }
            let mut recover = None;
            egui::ScrollArea::vertical().max_height(520.0).show(ui, |ui| {
                for record in &self.run_records {
                    let date=chrono::DateTime::from_timestamp(record.at,0).map(|t|t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(||record.at.to_string());
                    crate::app::controls::CollapsingHeader::new(format!("{} / {}", date, record.outcome.label())).id_salt(&record.id).show(ui, |ui| {
                        ui.label(record.execution.label());
                        ui.small(format!("実行ID：{} / 往復ID：{}", record.id, record.turn_id.as_deref().unwrap_or("未取得")));
                        ui.label(&record.note);
                        if let Some(text) = &record.response {
                            if ui.ww_button("本文をコピー").clicked() { ui.ctx().copy_text(text.clone()); }
                            let mut shown = text.as_str();
                            ui.add(egui::TextEdit::multiline(&mut shown).desired_rows(8).desired_width(f32::INFINITY));
                        } else if ui.add_enabled(self.pending.is_none() && record.turn_id.is_some(), crate::app::controls::Button::new("この往復の結果を再取得（生成しない）")).clicked() {
                            recover = Some(record.id.clone());
                        }
                    });
                }
            });
            if let Some(id) = recover { self.recover_run(id); }
        });
        self.run_history_open = open;
    }
    pub(super) fn refresh_runs(&mut self) {
        let result = self
            .storage
            .as_ref()
            .ok_or_else(|| "保存先がありません。".to_string())
            .and_then(|s| run_journal::scan(&s.dir.join("codex-work")));
        match result {
            Ok(r) => {
                self.run_records = r.records;
                if !r.warnings.is_empty() {
                    self.notify_warning(r.warnings.join("\n"));
                }
            }
            Err(e) => self.notify_error(e),
        }
    }
    fn recover_run(&mut self, id: String) {
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(c) => c,
            Err(e) => {
                self.notify_error(e);
                return;
            }
        };
        let (tx, rx) = mpsc::channel();
        self.pending = Some(Pending {
            kind: Activity::Recovery,
            key: self.key(),
            rx,
            cancel: Some(config.cancel.clone()),
        });
        std::thread::spawn(move || {
            let result: Result<RunRecord, String> =
                codex::recover(&config.exe, &config.cwd, &id, config.cancel);
            let _ = tx.send(result.map(AiResult::Recovered));
        });
    }
}
