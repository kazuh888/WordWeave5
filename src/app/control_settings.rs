use super::*;

impl WordApp {
    pub(super) fn effort_choices(&self) -> Option<&wordweave5::effort::ModelEffort> {
        let (path, models) = self.effort_catalog.as_ref()?;
        if path != self.progress.settings.codex_path.trim() {
            return None;
        }
        models
            .iter()
            .find(|m| m.model == self.progress.settings.codex_model.trim())
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
            .filter(|(path, _)| path == self.progress.settings.codex_path.trim())
            .map(|(_, models)| models.clone());
        if let Some(models) = catalog {
            egui::ComboBox::from_id_salt("advertised-models")
                .width(ui.available_width().min(440.0))
                .truncate()
                .selected_text("取得済みモデルから選択")
                .show_ui(ui, |ui| {
                    ui.ww_selectable_value(
                        &mut self.progress.settings.codex_model,
                        String::new(),
                        "Codexの既定モデル",
                    );
                    for model in models {
                        ui.ww_selectable_value(
                            &mut self.progress.settings.codex_model,
                            model.model.clone(),
                            format!("{} ({})", model.display_name, model.model),
                        );
                    }
                });
        }
        let model = self.effort_choices().cloned();
        let choices = model.as_ref().and_then(|m| m.supported_efforts.as_ref());
        let selected = if self.progress.settings.codex_effort.is_empty() {
            "Codexの既定値（指定しない）".to_string()
        } else {
            self.progress.settings.codex_effort.clone()
        };
        ui.label("effort（推論強度）");
        egui::ComboBox::from_id_salt("codex-effort")
            .width(ui.available_width().min(440.0))
            .truncate()
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.ww_selectable_value(
                    &mut self.progress.settings.codex_effort,
                    String::new(),
                    "Codexの既定値（指定しない）",
                );
                if let Some(choices) = choices {
                    for choice in choices {
                        ui.ww_selectable_value(
                            &mut self.progress.settings.codex_effort,
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
        if !self.progress.settings.codex_effort.is_empty()
            && !choices.is_some_and(|c| {
                c.iter()
                    .any(|e| e.effort == self.progress.settings.codex_effort)
            })
        {
            ui.colored_label(Color32::from_rgb(145, 75, 10), "保存済みeffortはこのモデルで未確認。自動変更はしない。実行時に再検査し、非対応なら送信を止める。");
        }
    }

    fn load_effort_catalog(&mut self) {
        if self.pending.is_some() || self.recorder.is_some() || self.fatal.is_some() {
            return;
        }
        let config = match ai::Config::from_settings(&self.progress.settings) {
            Ok(config) => config,
            Err(error) => {
                self.message = error;
                return;
            }
        };
        let path = self.progress.settings.codex_path.trim().to_string();
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

    pub(super) fn persistent_diagnostics_ui(&mut self, ui: &mut egui::Ui) {
        ui.ww_collapsing("診断ログ（再起動後も保持）", |ui| {
            ui.label(wordweave5::diagnostics::status());
            ui.small("操作・失敗分類・所要時間だけを記録する。入力文・回答を含む「実行記録」や、原音・筆跡の保存先とは別である。自動送信はしない。");
            if ui.ww_button("診断ログを読み込む／更新").clicked() {
                match wordweave5::diagnostics::export() {
                    Ok(text) => self.diagnostic_export = Some(text),
                    Err(error) => { self.diagnostic_export = None; self.message = error; }
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
                                let operation = DiagnosticOperation::begin(DiagnosticEntry::Settings);
                                self.message = match store::atomic_write(&path, text.as_bytes()) {
                                    Ok(()) => { operation.event(DiagnosticStage::Export, DiagnosticEvent::Completed); "診断ログを保存した。共有前に内容を確認してください。".into() }
                                    Err(error) => { operation.fail(DiagnosticStage::Export, DiagnosticError::Io); error }
                                };
                            }
                        }
                    });
                }
            }
        });
    }
}
