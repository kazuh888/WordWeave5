use super::*;

#[cfg(test)]
mod i2_export_settings {
    use super::*;
    use crate::app::settings_ui::i2_export_settings::{isolated, ledger, memory, Fixture};
    use std::fs;

    const JSONL: &str = "{\"at\":\"2026-10-12T00:00:00Z\",\"version\":\"0.6.0\",\"run_id\":\"1-2-3\",\"entry_point\":\"settings\",\"stage\":\"export\",\"event\":\"completed\",\"error\":null,\"elapsed_ms\":0}\n";

    fn reject(existing: bool) {
        let mut f = Fixture::new();
        f.app.diagnostic_export = Some(JSONL.into());
        let before = ledger(&f.root);
        let state = memory(&f.app);
        let targets = if existing {
            vec![
                "progress.json",
                "custom.tsv",
                "pending-material-commit.json",
                "app.lock",
                "backups/kept.json",
                "assets/audio-history/kept.bin",
                "audio-history/assets/kept.bin",
            ]
        } else {
            vec![
                "new.jsonl",
                "backups/new.jsonl",
                "assets/audio-history/new.jsonl",
                "audio-history/assets/new.jsonl",
            ]
        };
        for target in targets {
            let old_message = f.app.message.clone();
            let result = f.app.diagnostic_export_result(&f.dir().join(target), JSONL);
            assert!(
                result.is_err(),
                "{target}: must refuse protected destination"
            );
            assert_eq!(
                f.app.message, old_message,
                "notification belongs after the borrowed UI closure"
            );
            f.app.notify_result(result);
            assert!(!f.app.notification_error.is_empty());
            assert_eq!(f.app.message, f.app.notification_error);
            assert_eq!(ledger(&f.root), before);
            assert_eq!(memory(&f.app), state);
        }
    }

    #[test]
    fn s08_jsonl_rejects_protected_existing() {
        reject(true);
    }

    #[test]
    fn s08_jsonl_rejects_protected_new() {
        reject(false);
    }

    fn succeeds(replace: bool) {
        let mut f = Fixture::new();
        f.app.diagnostic_export = Some(JSONL.into());
        let external = f.root.join("data-similar-name");
        fs::create_dir(&external).unwrap();
        let dest = external.join("診断.jsonl");
        if replace {
            fs::write(&dest, b"external old").unwrap();
        }
        let before = ledger(&f.dir());
        let state = memory(&f.app);
        let result = f.app.diagnostic_export_result(&dest, JSONL);
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(f.app.message, "operation not completed");
        f.app.notify_result(result);
        assert!(f.app.notification_error.is_empty());
        assert_eq!(fs::read(&dest).unwrap(), JSONL.as_bytes());
        assert_eq!(fs::read_dir(external).unwrap().count(), 1);
        assert_eq!(ledger(&f.dir()), before);
        assert_eq!(memory(&f.app), state);
    }

    #[test]
    fn s09_jsonl_creates_exact_external() {
        succeeds(false);
    }

    #[test]
    fn s09_jsonl_replaces_exact_external() {
        succeeds(true);
    }

    #[test]
    fn s10_jsonl_failure_preserves_display_and_retry_succeeds() {
        let mut f = Fixture::new();
        f.app.diagnostic_export = Some(JSONL.into());
        let before = ledger(&f.root);
        let state = memory(&f.app);
        let result = f
            .app
            .diagnostic_export_result(&f.root.join("missing/export.jsonl"), JSONL);
        assert!(result.is_err());
        f.app.notify_result(result);
        assert_eq!(f.app.message, f.app.notification_error);
        assert_eq!(ledger(&f.root), before);
        assert_eq!(memory(&f.app), state);
        let dest = f.root.join("retry.jsonl");
        let result = f.app.diagnostic_export_result(&dest, JSONL);
        assert!(result.is_ok(), "{result:?}");
        f.app.notify_result(result);
        assert!(f.app.notification_error.is_empty());
        assert_eq!(fs::read(dest).unwrap(), JSONL.as_bytes());
        assert_eq!(memory(&f.app), state);
    }

    #[test]
    fn s11_jsonl_emits_failed_then_completed_events_in_isolated_process() {
        isolated(
            concat!(
                module_path!(),
                "::s11_jsonl_emits_failed_then_completed_events_in_isolated_process"
            ),
            || {
                let f = Fixture::new();
                let log_dir = f.dir().join("diagnostics");
                wordweave5::diagnostics::initialize(&log_dir);
                assert!(
                    wordweave5::diagnostics::export().unwrap().is_empty(),
                    "fresh isolated sink"
                );
                let progress = fs::read(f.dir().join("progress.json")).unwrap();
                let old_message = f.app.message.clone();
                assert!(f
                    .app
                    .diagnostic_export_result(&f.dir().join("progress.json"), JSONL)
                    .is_err());
                assert_eq!(fs::read(f.dir().join("progress.json")).unwrap(), progress);
                let failed: Vec<serde_json::Value> = wordweave5::diagnostics::export()
                    .unwrap()
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(
                    failed.len(),
                    2,
                    "start and failed export records, no success"
                );
                assert_eq!(failed[0]["entry_point"], "settings");
                assert_eq!(failed[0]["stage"], "start");
                assert_eq!(failed[0]["event"], "started");
                assert_eq!(failed[1]["stage"], "export");
                assert_eq!(failed[1]["event"], "failed");
                assert_eq!(failed[1]["error"], "io");
                assert_eq!(failed[0]["run_id"], failed[1]["run_id"]);
                let dest = f.root.join("diagnostic.jsonl");
                assert!(f.app.diagnostic_export_result(&dest, JSONL).is_ok());
                assert_eq!(fs::read(dest).unwrap(), JSONL.as_bytes());
                let records: Vec<serde_json::Value> = wordweave5::diagnostics::export()
                    .unwrap()
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                assert_eq!(records.len(), 4);
                assert_eq!(records[2]["event"], "started");
                assert_eq!(records[3]["stage"], "export");
                assert_eq!(records[3]["event"], "completed");
                assert_eq!(records[3]["error"], serde_json::Value::Null);
                assert_eq!(records[2]["run_id"], records[3]["run_id"]);
                assert_ne!(records[0]["run_id"], records[2]["run_id"]);
                assert_eq!(f.app.message, old_message, "result helper must not notify");
                // The typed diagnostics above are expected additions, not data corruption.
                assert_eq!(fs::read(f.dir().join("progress.json")).unwrap(), progress);
            },
        );
    }
}

impl WordApp {
    pub(super) fn effort_choices(&self) -> Option<&wordweave5::effort::ModelEffort> {
        let (path, models) = self.effort_catalog.as_ref()?;
        if path != self.settings_for_ui().codex_path.trim() {
            return None;
        }
        models
            .iter()
            .find(|m| m.model == self.settings_for_ui().codex_model.trim())
    }

    pub(super) fn effort_settings(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.pending.is_none() && self.recorder.is_none() && self.fatal.is_none(),
                crate::app::controls::Button::new("モデル・effort候補を取得"),
            )
            .clicked()
        {
            self.load_effort_catalog();
        }
        let catalog = self
            .effort_catalog
            .as_ref()
            .filter(|(path, _)| path == self.settings_for_ui().codex_path.trim())
            .map(|(_, models)| models.clone());
        if let Some(models) = catalog {
            egui::ComboBox::from_id_salt("advertised-models")
                .width(ui.available_width().min(440.0))
                .truncate()
                .selected_text("取得済みモデルから選択")
                .show_ui(ui, |ui| {
                    ui.ww_selectable_value(
                        &mut self.settings_editor.draft.codex_model,
                        String::new(),
                        "Codexの既定モデル",
                    );
                    for model in models {
                        ui.ww_selectable_value(
                            &mut self.settings_editor.draft.codex_model,
                            model.model.clone(),
                            format!("{} ({})", model.display_name, model.model),
                        );
                    }
                });
        }
        let model = self.effort_choices().cloned();
        let choices = model.as_ref().and_then(|m| m.supported_efforts.as_ref());
        let selected = if self.settings_for_ui().codex_effort.is_empty() {
            "Codexの既定値（指定しない）".to_string()
        } else {
            self.settings_for_ui().codex_effort.clone()
        };
        ui.label("effort（推論強度）");
        egui::ComboBox::from_id_salt("codex-effort")
            .width(ui.available_width().min(440.0))
            .truncate()
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.ww_selectable_value(
                    &mut self.settings_editor.draft.codex_effort,
                    String::new(),
                    "Codexの既定値（指定しない）",
                );
                if let Some(choices) = choices {
                    for choice in choices {
                        ui.ww_selectable_value(
                            &mut self.settings_editor.draft.codex_effort,
                            choice.effort.clone(),
                            &choice.effort,
                        )
                        .on_hover_text(&choice.description);
                    }
                }
            });
        ui.small("effortは要求する推論強度。実際の実行表示はCodexの返却値であり、この指定から推測しない。");
        if let Some(default) = model.as_ref().and_then(|m| m.default_effort.as_ref()) {
            ui.small(format!(
                "サーバーが示すこのモデルの既定effort：{default}（今回の実行値ではない）"
            ));
        }
        if choices.is_none() {
            ui.label(
                "このモデルのeffort候補は未取得。候補を取得し、モデルを選択すると指定できる。",
            );
        }
        if !self.settings_for_ui().codex_effort.is_empty()
            && !choices.is_some_and(|c| {
                c.iter()
                    .any(|e| e.effort == self.settings_for_ui().codex_effort)
            })
        {
            ui.colored_label(Color32::from_rgb(145, 75, 10), "保存済みeffortはこのモデルで未確認。自動変更はしない。実行時に再検査し、非対応なら送信を止める。");
        }
    }

    fn load_effort_catalog(&mut self) {
        if self.pending.is_some() || self.recorder.is_some() || self.fatal.is_some() {
            return;
        }
        let config = match ai::Config::from_settings(&self.settings_for_ui()) {
            Ok(config) => config,
            Err(error) => {
                self.notify_error(error);
                return;
            }
        };
        let path = self.settings_for_ui().codex_path.trim().to_string();
        self.effort_catalog = None;
        let cancel = config.cancel.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result =
                wordweave5::codex::list_model_efforts(&config.exe, &config.cwd, config.cancel)
                    .map(|models| AiResult::ModelChoices { path, models });
            let _ = tx.send(result);
        });
        self.pending = Some(Pending {
            key: self.key(),
            rx,
            cancel: Some(cancel),
            kind: Activity::Models,
        });
        self.message = "ChatGPT認証を確認し、Codexからモデル・effort候補を取得中…".into();
    }

    fn diagnostic_export_result(
        &self,
        dest: &std::path::Path,
        text: &str,
    ) -> Result<String, String> {
        let operation = DiagnosticOperation::begin(DiagnosticEntry::Settings);
        match self.write_settings_export(dest, text.as_bytes()) {
            Ok(()) => {
                operation.event(DiagnosticStage::Export, DiagnosticEvent::Completed);
                Ok("診断ログを保存した。共有前に内容を確認してください。".into())
            }
            Err(error) => {
                operation.fail(DiagnosticStage::Export, DiagnosticError::Io);
                Err(error)
            }
        }
    }

    pub(super) fn persistent_diagnostics_ui(&mut self, ui: &mut egui::Ui) {
        let mut export_result = None;
        ui.ww_collapsing("診断ログ（再起動後も保持）", |ui| {
            ui.label(wordweave5::diagnostics::status());
            ui.small("操作・失敗分類・所要時間だけを記録する。入力文・回答を含む「実行記録」や、原音・筆跡の保存先とは別である。自動送信はしない。");
            if ui.ww_button("診断ログを読み込む／更新").clicked() {
                match wordweave5::diagnostics::export() {
                    Ok(text) => self.diagnostic_export = Some(text),
                    Err(error) => { self.diagnostic_export = None; self.notify_error(error); }
                }
            }
            if let Some(text) = &self.diagnostic_export {
                if text.is_empty() { ui.label("診断ログはまだない。"); }
                else {
                    let mut display = text.as_str();
                    egui::ScrollArea::vertical().id_salt("persistent-diagnostics").max_height(200.0).show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut display).desired_width(f32::INFINITY).interactive(false));
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.ww_button("表示中のログをコピー").clicked() { ui.ctx().copy_text(text.clone()); }
                        if ui.ww_button("表示中のログをファイルに保存").clicked() {
                            if let Some(path) = rfd::FileDialog::new().set_file_name("wordweave-diagnostics.jsonl").save_file() {
                                export_result = Some(self.diagnostic_export_result(&path, text));
                            }
                        }
                    });
                }
            }
        });
        if let Some(result) = export_result {
            self.notify_result(result);
        }
    }
}
