use super::*;

impl WordApp {
    pub(super) fn settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.codex_path_guidance {
            // Recovery is a focused settings view, not a jump into a long form.
            let before = serde_json::to_string(&self.progress.settings).unwrap_or_default();
            ux::panel(ui, true, |ui| self.connection_settings_panel(ui));
            if before != serde_json::to_string(&self.progress.settings).unwrap_or_default() {
                self.dirty = true;
            }
            if ux::primary(ui, "設定を保存", self.fatal.is_none()).clicked() {
                self.persist();
            }
            return;
        }
        ux::heading(
            ui,
            "設定",
            "学習しやすい量・入力方法・音声を選ぶ。接続やバックアップの設定は下にまとめてある。",
        );
        let before = serde_json::to_string(&self.progress.settings).unwrap_or_default();
        ux::panel(ui, false, |ui| self.learning_settings_panel(ui, ctx));
        ui.add_space(12.0);
        ux::panel(ui, false, |ui| self.voice_settings_panel(ui));
        ui.add_space(16.0);
        ui.label(RichText::new("接続とデータ管理").size(22.0).strong());
        ux::panel(ui, false, |ui| self.connection_settings_panel(ui));
        ui.add_space(12.0);
        ux::panel(ui, false, |ui| {
            ui.strong("生成・添削の上限");
            ui.label("生成・添削の1日上限（試行回数）");
            ui.add(egui::Slider::new(
                &mut self.progress.settings.ai_daily_limit,
                0..=1000,
            ));
            ui.label("1回に生成する例文数");
            ui.add(egui::Slider::new(
                &mut self.progress.settings.examples_per_word,
                3..=12,
            ));
            ui.small(
                "生成・添削の上限は語数ではなく試行回数である。ChatGPT契約側の利用枠とは別の設定。",
            );
        });
        ui.add_space(12.0);
        ux::panel(ui, false, |ui| self.backup_settings_panel(ui));
        ui.add_space(12.0);
        ux::panel(ui, false, |ui| self.diagnostic_settings_panel(ui));
        ui.add_space(12.0);
        ui.ww_collapsing("学習方式と限界", |ui| {
            ui.label("間隔学習・想起練習・段階的ヒントを採用。復習間隔は透明な独自の計算規則であり、FSRSでも『科学的に最速と証明された方式』でもない。");
            ui.label("復習結果・入力方式別の記録を確認しながら、学習量を調整する。自己評価を含むため、数値は能力の厳密な測定ではない。");
            ui.label("詳細な研究根拠・教材の選定基準は同梱のRESEARCH.mdを参照。");
        });
        if before != serde_json::to_string(&self.progress.settings).unwrap_or_default() {
            self.dirty = true;
        }
        ui.add_space(12.0);
        if ux::primary(ui, "設定を保存", self.fatal.is_none()).clicked() {
            self.persist();
            if self.fatal.is_none() {
                self.message = "設定を保存した。".into();
            }
        }
    }

    fn learning_settings_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.strong("学習と入力");
        ui.label("通常コースの目安時間（分）");
        ui.add(egui::Slider::new(
            &mut self.progress.settings.minutes,
            1..=30,
        ));
        ui.label("新規項目の1日上限");
        ui.add(egui::Slider::new(
            &mut self.progress.settings.new_per_day,
            0..=20,
        ));
        ui.small("1日5分では3項目を初期値とする。復習候補が6項目を超える日は新規を出さない。");
        ui.add_space(8.0);
        ui.label("練習で出題する形式");
        ui.horizontal_wrapped(|ui| {
            for skill in Skill::ALL {
                let mut enabled = self.progress.settings.skills.contains(&skill);
                if ui.checkbox(&mut enabled, skill.label()).changed() {
                    if enabled {
                        self.progress.settings.skills.push(skill);
                    } else {
                        self.progress.settings.skills.retain(|s| *s != skill);
                    }
                }
            }
        });
        if self.progress.settings.skills.is_empty() {
            self.progress.settings.skills.push(Skill::Recall);
        }
        ui.small("少なくとも1種類を使用する。すべて外した場合は語句・綴りを有効にする。");
        let mut tags: Vec<String> = self
            .deck
            .iter()
            .map(|e| e.tag.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        tags.insert(0, "すべて".into());
        ui.label("出題する場面・分類");
        egui::ComboBox::from_id_salt("topic")
            .width(ui.available_width().min(440.0))
            .truncate()
            .selected_text(&self.progress.settings.topic)
            .show_ui(ui, |ui| {
                for tag in tags {
                    ui.ww_selectable_value(&mut self.progress.settings.topic, tag.clone(), tag);
                }
            });
        ui.small("出題の設定は次のセッションから反映する。");
        ui.add_space(8.0);
        ui.label("画面の拡大率");
        if ui
            .add(egui::Slider::new(
                &mut self.progress.settings.font_scale,
                0.5..=1.6,
            ))
            .changed()
        {
            ctx.set_zoom_factor(self.progress.settings.font_scale);
        }
    }

    fn voice_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("音声");
        ui.label("読み上げに使用する音声");
        let current_voice = self
            .speaker
            .voices
            .iter()
            .find(|v| v.0 == self.progress.settings.voice_id)
            .map(|v| v.1.clone())
            .unwrap_or_else(|| "英語の音声を自動選択".into());
        egui::ComboBox::from_id_salt("voice")
            .width(ui.available_width().min(440.0))
            .truncate()
            .selected_text(current_voice)
            .show_ui(ui, |ui| {
                ui.ww_selectable_value(
                    &mut self.progress.settings.voice_id,
                    String::new(),
                    "英語の音声を自動選択",
                );
                for (id, name) in &self.speaker.voices {
                    ui.ww_selectable_value(&mut self.progress.settings.voice_id, id.clone(), name);
                }
            });
        ui.label("読み上げ速度（倍）");
        let mut rate = self.speech_rate();
        if ui
            .add(egui::Slider::new(&mut rate, 0.5..=4.0).step_by(0.1))
            .changed()
        {
            if let Err(error) = self.change_speech_rate(rate) {
                self.message = error;
            }
        }
        if ui.ww_button("音声を確認する").clicked() {
            self.say("We appreciate your assistance.");
        }
        ui.small(
            "読み上げはWindowsの音声合成。マイクはWindowsで設定した既定の入力デバイスを使う。",
        );
    }

    fn connection_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("AI接続・モデル・effort");
        ui.label(
            "Codex CLIをインストールし、ターミナルで codex login を実行してChatGPTでログインする。",
        );
        ui.small("APIキーは使用しない。生成はChatGPT契約の利用枠を使用し、上限到達時は停止する。");
        ui.add_space(8.0);
        ui.label("Codex実行ファイル");
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.progress.settings.codex_path)
                .id_salt("codex-executable-input")
                .desired_width(ui.available_width().min(720.0)),
        );
        if self.codex_path_guidance {
            ui.painter().rect_stroke(
                response.rect.expand(3.0),
                3.0,
                egui::Stroke::new(2.0_f32, Color32::from_rgb(190, 100, 20)),
                egui::StrokeKind::Outside,
            );
            ui.colored_label(Color32::from_rgb(165, 75, 15), "修正対象は上の実行ファイル欄である。① 実在するファイルを選択、または自動検出を選ぶ。②「接続・ChatGPT認証を確認」を実行する。");
            ui.small(
                "入力中の回答・学習セッションは保持している。案内を閉じると設定の全項目へ戻る。",
            );
        }
        if self.codex_path_focus_pending {
            response.scroll_to_me_animation(
                Some(egui::Align::Center),
                egui::style::ScrollAnimation::none(),
            );
            // Initial sizing passes may discard the scroll request. Complete
            // guidance only after the actual input is inside the viewport.
            if ui.clip_rect().contains(response.rect.center()) && !ui.ctx().will_discard() {
                response.request_focus();
                self.codex_path_focus_pending = false;
            }
        }
        if self.codex_path_guidance && ui.ww_button("codexで自動検出する").clicked() {
            self.progress.settings.codex_path = "codex".into();
        }
        if ui.ww_button("Codex実行ファイルを選択").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Codex", &["exe", "cmd", "bat"])
                .pick_file()
            {
                self.progress.settings.codex_path = path.to_string_lossy().into();
            }
        }
        ui.small("実行ファイル欄をcodexにすると、起動時PATH → システムPATH → ユーザーPATH → npmの順に探索する。特定のCLIを使う場合はファイルを選択する。引数は入力しない。");
        ui.add_space(8.0);
        ui.label("要求するモデル（空欄はCodexの既定値）");
        ui.add(
            egui::TextEdit::singleline(&mut self.progress.settings.codex_model)
                .desired_width(ui.available_width().min(440.0)),
        );
        self.effort_settings(ui);
        ui.small(
            "ここは次回の要求設定。実際のモデル・effortは、Codexの返却後に画面下部へ表示する。",
        );
        ui.label(self.connection_label());
        if ui
            .add_enabled(
                self.pending.is_none(),
                crate::app::controls::Button::new("接続・ChatGPT認証を確認"),
            )
            .clicked()
        {
            self.launch_content(0, None);
        }
        if self.codex_path_guidance && ui.ww_button("修正案内を閉じる").clicked() {
            self.codex_path_guidance = false;
        }
        ui.small("手書き認識は画像対応モデルが必要。録音の自動文字起こしには音声入力対応モデルが必要。非対応時も録音・再生は利用できる。");
    }

    fn backup_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("教材・バックアップ");
        let idle = self.session.is_none() && self.pending.is_none() && self.recorder.is_none();
        ui.label("教材の書き出し・取り込み");
        ui.horizontal_wrapped(|ui| {
            if ui.ww_button("教材をTSVに書き出す").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("wordweave-deck.tsv")
                    .add_filter("TSV", &["tsv"])
                    .save_file()
                {
                    self.message =
                        store::atomic_write(&path, model::deck_text(&self.deck).as_bytes())
                            .map(|_| "教材を書き出した。編集後は取り込みで反映できる。".into())
                            .unwrap_or_else(|e| e);
                }
            }
            if ui
                .add_enabled(
                    idle && self.fatal.is_none(),
                    crate::app::controls::Button::new("教材TSVを取り込む"),
                )
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("TSV", &["tsv"])
                    .pick_file()
                {
                    match read_limited(&path, 64_000_000).and_then(|t| model::parse_deck(&t)) {
                        Ok(deck) => self.pending_import = Some(deck),
                        Err(error) => self.message = error,
                    }
                }
            }
        });
        ui.small("同じIDは更新、新しいIDは追加。内容を変更した項目の復習状態は再学習から始める。学習中の取り込みはできない。");
        ui.add_space(8.0);
        ui.label("学習記録の書き出し・復元");
        ui.horizontal_wrapped(|ui| {
            if ui.ww_button("学習記録をエクスポート").clicked() {
                self.credit_time();
                if let Some(path) = rfd::FileDialog::new().set_file_name("wordweave-progress.json")
                    .add_filter("JSON", &["json"]).save_file() {
                    let result = serde_json::to_vec_pretty(&self.progress).map_err(|e| e.to_string())
                        .and_then(|bytes| store::atomic_write(&path, &bytes));
                    self.message = result.map(|_| "学習記録を書き出した。このJSONに教材TSV・録音・筆跡原本は含まれない。媒体付きバックアップも使用してください。".into()).unwrap_or_else(|e| e);
                }
            }
            if ui.add_enabled(idle && self.storage.is_some(), crate::app::controls::Button::new("学習記録を復元")).clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
                    match read_limited(&path, 100_000_000)
                        .and_then(|text| serde_json::from_str::<Progress>(&text).map_err(|e| e.to_string()))
                        .and_then(|progress| { progress.validate()?; Ok(progress) }) {
                        Ok(progress) => self.pending_restore = Some(progress),
                        Err(error) => self.message = error,
                    }
                }
            }
        });
        if let Some(storage) = &self.storage {
            ui.add(
                egui::Label::new(
                    RichText::new(format!("保存先：{}", storage.dir.display())).small(),
                )
                .wrap(),
            );
        }
        ui.small("日ごとのバックアップは保存先のbackupsフォルダーに残る。復元前の記録も別ファイルに退避する。");
        self.backup_controls(ui, idle);
    }

    fn diagnostic_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.strong("診断");
        ui.small("接続の問題を調べるための情報。確認・コピー・共有はそれぞれ別の操作である。");
        ui.ww_collapsing("Codex診断情報（コピー・保存・ChatGPTで相談）", |ui| {
            ui.label("接続確認または生成の直近1回を記録する。原文・認証情報は保存せず、stderrは分類のみ。未分類の原因を特定できない場合がある。");
            ui.label("未ログインなら、同じWindowsユーザーのCMDで codex login --device-auth を実行し、認証完了後に codex login status で確認する。");
            let mut report = wordweave5::diagnostics::report();
            egui::ScrollArea::vertical().id_salt("codex_diagnostics").max_height(240.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut report).desired_width(f32::INFINITY).interactive(false));
            });
            ui.small("共有前に表示内容を確認すること。コピーとブラウザー起動は別操作で、自動送信しない。");
            ui.horizontal_wrapped(|ui| {
                if ui.ww_button("確認した診断情報をコピー").clicked() {
                    ui.ctx().copy_text(report.clone());
                }
                if ui.ww_button("診断情報を保存").clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_file_name("wordweave-codex-diagnostics.txt").save_file() {
                        self.message = store::atomic_write(&path, report.as_bytes())
                            .map(|_| "診断情報を保存した。".into()).unwrap_or_else(|e| e);
                    }
                }
                ui.hyperlink_to("ChatGPTを開く（手動貼り付け）", "https://chatgpt.com/");
            });
        });
        self.persistent_diagnostics_ui(ui);
    }
}
