use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::harness_tests::fixture;

    fn render(ctx: &egui::Context, app: &mut WordApp, size: egui::Vec2, events: Vec<egui::Event>) {
        let _ = ctx.run(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events, ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let right = ui.max_rect().right();
                app.settings(ui, ctx);
                assert!(ui.min_rect().right() <= right + 1.0, "settings overflow: {:?}", ui.min_rect());
            });
        });
    }

    #[test]
    fn settings_zoom_control_keeps_native_position_and_size_at_all_scales() {
        let (ctx, mut app, root) = fixture();
        app.begin_settings_edit();
        app.settings_editor.zoom_open = true;
        let mut baseline: Option<egui::Rect> = None;
        for zoom in [0.5, 0.8, 1.0, 1.6, 0.8] {
            ctx.set_zoom_factor(zoom);
            for _ in 0..3 {
                let _ = ctx.run(egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO,
                        egui::vec2(1150.0, 950.0) / zoom)), ..Default::default()
                }, |ctx| app.settings_zoom_panel(ctx));
            }
            let logical = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-zoom"))).unwrap();
            let physical = egui::Rect::from_min_max(logical.min * zoom, logical.max * zoom);
            if let Some(first) = baseline {
                assert!((physical.min - first.min).length() < 2.0, "{physical:?} vs {first:?}");
                assert!((physical.size() - first.size()).length() < 2.0, "{physical:?} vs {first:?}");
            } else { baseline = Some(physical); }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_categories_preserve_values_and_keep_save_visible() {
        for size in [egui::vec2(1437.5, 1187.5), egui::vec2(512.5, 406.25)] {
            let (ctx, mut app, root) = fixture();
            app.progress.settings.minutes = 12;
            app.progress.settings.codex_model = "keep-user-model".into();
            app.progress.chats[0].draft = "unfinished message".into();
            let before = serde_json::to_value(&app.progress).unwrap();
            app.dirty = false;
            for _ in 0..2 { render(&ctx, &mut app, size, vec![]); }
            for section in SettingsSection::ALL {
                render(&ctx, &mut app, size, vec![]);
                if size.x < 650.0 {
                    let menu = ctx.data(|data| data.get_temp::<egui::Rect>(
                        egui::Id::new("settings-category-menu"))).unwrap();
                    for pressed in [true, false] {
                        render(&ctx, &mut app, size, vec![egui::Event::PointerMoved(menu.center()),
                            egui::Event::PointerButton { pos: menu.center(), button: egui::PointerButton::Primary,
                                pressed, modifiers: egui::Modifiers::NONE }]);
                    }
                    render(&ctx, &mut app, size, vec![]);
                }
                let tab = ctx.data(|data| data.get_temp::<egui::Rect>(
                    egui::Id::new(("settings-tab", section as u8)))).unwrap();
                assert!(tab.bottom() < size.y, "category must remain accessible");
                for pressed in [true, false] {
                    render(&ctx, &mut app, size, vec![egui::Event::PointerMoved(tab.center()),
                        egui::Event::PointerButton { pos: tab.center(), button: egui::PointerButton::Primary,
                            pressed, modifiers: egui::Modifiers::NONE }]);
                }
                assert_eq!(app.settings_section, section);
                render(&ctx, &mut app, size, vec![]);
                let save = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-save-button"))).unwrap();
                let viewport = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-content-viewport"))).unwrap();
                assert!(save.bottom() <= size.y
                    && (save.top() >= viewport.bottom() || save.bottom() <= viewport.top()),
                    "save={save:?}, viewport={viewport:?}");
                assert!(viewport.height() >= 90.0, "retain usable form height: {viewport:?}");
                render(&ctx, &mut app, size, vec![
                    egui::Event::PointerMoved(viewport.center()),
                    egui::Event::MouseWheel { unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -180.0), modifiers: egui::Modifiers::NONE },
                ]);
                let after_scroll = ctx.data(|data| data.get_temp::<egui::Rect>(
                    egui::Id::new("settings-save-button"))).unwrap();
                assert_eq!(after_scroll, save, "scrolling must not move the save action");
                assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
                assert!(!app.dirty, "navigation must not change saved settings");
                assert!(app.pending.is_none() && app.pending_import.is_none() && app.pending_restore.is_none());
            }
            drop(app);
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn settings_fit_between_native_navigation_and_status_at_maximum_zoom() {
        let (ctx, mut app, root) = fixture();
        app.page = Page::Settings;
        ctx.set_zoom_factor(1.6);
        for section in SettingsSection::ALL {
            app.settings_section = section;
            for _ in 0..4 {
                let _ = ctx.run(egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO, egui::vec2(512.5, 406.25))),
                    ..Default::default()
                }, |ctx| app.update_ui(ctx));
            }
            let save = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-save-button"))).unwrap();
            let viewport = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-content-viewport"))).unwrap();
            let menu = ctx.data(|data| data.get_temp::<egui::Rect>(egui::Id::new("settings-category-menu"))).unwrap();
            assert!(viewport.height() >= 80.0, "retain form space with app chrome: {viewport:?}");
            assert!(menu.bottom() < viewport.top() && save.bottom() <= viewport.top(),
                "controls must not overlap: {menu:?}, {viewport:?}, {save:?}");
            assert!(save.bottom() < 406.25);
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_recovery_focuses_the_visible_path_in_the_scrolling_form() {
        let (ctx, mut app, root) = fixture();
        app.settings_section = SettingsSection::Data;
        // Startup applies the configured zoom using egui's previous screen size.
        // Recovery is opened by a user action after the real viewport is established.
        for _ in 0..2 {
            render(&ctx, &mut app, egui::vec2(512.5, 406.25), vec![]);
        }
        app.open_codex_path_guidance();
        for _ in 0..8 {
            render(&ctx, &mut app, egui::vec2(512.5, 406.25), vec![]);
        }
        let (id, rect, clip) = ctx.data(|data| data.get_temp::<(egui::Id, egui::Rect, egui::Rect)>(
            egui::Id::new("settings-path-input"))).unwrap();
        assert!(clip.contains(rect.center()), "path={rect:?}, clip={clip:?}");
        assert!(ctx.memory(|memory| memory.has_focus(id)));
        assert!(!app.codex_path_focus_pending);
        assert_eq!(app.settings_section, SettingsSection::Data, "recovery must not reset category state");
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsSection {
    Learning,
    Voice,
    Connection,
    Data,
    Help,
}

impl SettingsSection {
    pub(super) const ALL: [Self; 5] = [
        Self::Learning, Self::Voice, Self::Connection, Self::Data, Self::Help,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Learning => "学習・表示",
            Self::Voice => "音声",
            Self::Connection => "AI接続",
            Self::Data => "教材・データ",
            Self::Help => "診断・ヘルプ",
        }
    }
}

impl WordApp {
    pub(super) fn settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.begin_settings_edit();
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(12.0, 10.0);
            ui.spacing_mut().button_padding = egui::vec2(14.0, 8.0);
            ui.spacing_mut().interact_size.y = 36.0;
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 16.0),
                (egui::TextStyle::Button, 18.0),
            ] {
                ui.style_mut().text_styles.insert(style, home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(home_art::INK);
            let compact = ui.available_width() < 650.0;
            let show_save_hint = ui.available_width() >= 800.0;
            // Keep saving and category navigation outside the scrolling form.
            if !compact || self.codex_path_guidance {
            egui::TopBottomPanel::bottom("settings-save-bar")
                .resizable(false)
                .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(8))
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.settings_save_button(ui);
                        if show_save_hint {
                            ui.small("設定は保存するまで確定しない。出題設定は次の学習から反映。");
                        }
                    });
                });
            }
            egui::Frame::new().fill(ux::TINT)
                .stroke(egui::Stroke::new(1.0_f32, ux::BORDER)).corner_radius(10)
                .inner_margin(if compact { 6 } else { 16 })
                .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if compact && !self.codex_path_guidance {
                    ui.horizontal(|ui| {
                        home_art::title(ui, "設定", 24.0);
                        let menu = egui::ComboBox::from_id_salt("settings-category")
                            .width(ui.available_width().min(150.0))
                            .selected_text(self.settings_section.label())
                            .show_ui(ui, |ui| {
                                // Keep all five categories reachable in the compact popup.
                                ui.spacing_mut().item_spacing.y = 4.0;
                                ui.spacing_mut().button_padding.y = 4.0;
                                ui.spacing_mut().interact_size.y = 28.0;
                                for section in SettingsSection::ALL {
                                    let response = ui.ww_selectable_value(
                                        &mut self.settings_section, section, section.label());
                                    #[cfg(test)]
                                    ui.ctx().data_mut(|data| data.insert_temp(
                                        egui::Id::new(("settings-tab", section as u8)), response.rect));
                                    #[cfg(not(test))]
                                    let _ = response;
                                }
                            });
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| data.insert_temp(
                            egui::Id::new("settings-category-menu"), menu.response.rect));
                        #[cfg(not(test))]
                        let _ = menu;
                        self.settings_save_button(ui);
                    });
                    return;
                }
                ui.horizontal(|ui| {
                    home_art::badge(ui, home_art::Icon::Gear, home_art::BLUE, 36.0);
                    home_art::title(ui, if self.codex_path_guidance { "AI接続の修正" } else { "設定" }, 28.0);
                });
                if !self.codex_path_guidance {
                    if ui.available_width() >= 650.0 {
                        ui.label("変更したい項目を選ぶ。設定値は分類を切り替えても保持される。");
                    }
                    ui.horizontal_wrapped(|ui| {
                        for section in SettingsSection::ALL {
                            let selected = self.settings_section == section;
                            let response = ui.add(crate::app::controls::Button::new(
                                RichText::new(section.label()).color(if selected { home_art::BLUE } else { home_art::INK }))
                                .fill(if selected { Color32::from_rgb(210, 233, 255) } else { Color32::WHITE })
                                .stroke(egui::Stroke::new(if selected { 2.0_f32 } else { 1.0_f32 }, ux::BORDER))
                                .corner_radius(6).min_size(egui::vec2(0.0, 40.0)));
                            #[cfg(test)]
                            ui.ctx().data_mut(|data| data.insert_temp(
                                egui::Id::new(("settings-tab", section as u8)), response.rect));
                            if response.clicked() {
                                self.settings_section = section;
                            }
                        }
                    });
                }
            });
            ui.add_space(4.0);
            let content = egui::ScrollArea::vertical()
                .id_salt(("settings-content", self.settings_section as u8, self.codex_path_guidance))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if self.codex_path_guidance {
                        // Preserve the focused recovery view and its one-shot field focus.
                        ux::panel(ui, true, |ui| self.connection_settings_panel(ui));
                    } else {
                        match self.settings_section {
                            SettingsSection::Learning => ux::panel(ui, false, |ui| self.learning_settings_panel(ui, ctx)),
                            SettingsSection::Voice => ux::panel(ui, false, |ui| self.voice_settings_panel(ui)),
                            SettingsSection::Connection => {
                                ux::panel(ui, false, |ui| self.connection_settings_panel(ui));
            ui.add_space(4.0);
            if let Some(error) = &self.settings_editor.error {
                ui.colored_label(Color32::DARK_RED, format!("設定を保存できなかった：{error}"));
            }
                                ux::panel(ui, false, |ui| self.generation_settings_panel(ui));
                            }
                            SettingsSection::Data => ux::panel(ui, false, |ui| self.backup_settings_panel(ui)),
                            SettingsSection::Help => {
                                ux::panel(ui, false, |ui| self.diagnostic_settings_panel(ui));
                                ui.add_space(4.0);
                                ux::panel(ui, false, |ui| {
                                    ui.ww_collapsing("学習方式と限界", |ui| {
                                        ui.label("間隔学習・想起練習・段階的ヒントを採用。復習間隔は透明な独自の計算規則であり、FSRSでも『科学的に最速と証明された方式』でもない。");
                                        ui.label("復習結果・入力方式別の記録を確認しながら、学習量を調整する。自己評価を含むため、数値は能力の厳密な測定ではない。");
                                        ui.label("詳細な研究根拠・教材の選定基準は同梱のRESEARCH.mdを参照。");
                                    });
                                });
                            }
                        }
                    }
                });
            #[cfg(test)]
            ui.ctx().data_mut(|data| data.insert_temp(
                egui::Id::new("settings-content-viewport"), content.inner_rect));
            #[cfg(not(test))]
            let _ = content;
        });
    }

    fn settings_save_button(&mut self, ui: &mut egui::Ui) {
        let save = ui.add_enabled(self.fatal.is_none() && self.settings_changed(),
            crate::app::controls::Button::new(
                RichText::new("保存").color(Color32::WHITE))
                .fill(home_art::BLUE).min_size(egui::vec2(72.0, 44.0)));
        #[cfg(test)]
        ui.ctx().data_mut(|data| data.insert_temp(
            egui::Id::new("settings-save-button"), save.rect));
        if save.clicked() {
            if let Err(error) = self.save_settings(ui.ctx()) {
                self.message = error.clone();
                self.settings_editor.error = Some(error);
            }
        }
        if ui.ww_button("キャンセル").clicked() { self.cancel_settings(ui.ctx()); }
        if let Some(error) = &self.settings_editor.error {
            save.on_hover_text(error);
        }
    }

    fn generation_settings_panel(&mut self, ui: &mut egui::Ui) {
        home_art::title(ui, "生成・添削の上限", 22.0);
        ui.label("生成・添削の1日上限（試行回数）");
        ui.add(egui::Slider::new(
            &mut self.settings_editor.draft.ai_daily_limit,
            0..=1000,
        ));
        ui.label("1回に生成する例文数");
        ui.add(egui::Slider::new(
            &mut self.settings_editor.draft.examples_per_word,
            3..=12,
        ));
        ui.small(
            "生成・添削の上限は語数ではなく試行回数である。ChatGPT契約側の利用枠とは別の設定。",
        );
    }

    fn learning_settings_panel(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        home_art::title(ui, "学習と表示", 22.0);
        ui.label("通常コースの目安時間（分）");
        ui.add(egui::Slider::new(
            &mut self.settings_editor.draft.minutes,
            1..=30,
        ));
        ui.label("新規項目の1日上限");
        ui.add(egui::Slider::new(
            &mut self.settings_editor.draft.new_per_day,
            0..=20,
        ));
        ui.small("1日5分では3項目を初期値とする。復習候補が6項目を超える日は新規を出さない。");
        ui.add_space(8.0);
        ui.label("練習で出題する形式");
        ui.horizontal_wrapped(|ui| {
            for skill in Skill::ALL {
                let mut enabled = self.settings_editor.draft.skills.contains(&skill);
                if ui.checkbox(&mut enabled, skill.label()).changed() {
                    if enabled {
                        self.settings_editor.draft.skills.push(skill);
                    } else {
                        self.settings_editor.draft.skills.retain(|s| *s != skill);
                    }
                }
            }
        });
        if self.settings_editor.draft.skills.is_empty() {
            self.settings_editor.draft.skills.push(Skill::Recall);
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
        let topic = egui::ComboBox::from_id_salt("topic")
            .width(ui.available_width().min(240.0))
            .truncate()
            .selected_text(&self.settings_editor.draft.topic)
            .show_ui(ui, |ui| {
                for tag in tags {
                    ui.ww_selectable_value(&mut self.settings_editor.draft.topic, tag.clone(), tag);
                }
            });
        topic.response.clone().on_hover_text(&self.settings_editor.draft.topic);
        #[cfg(test)]
        ui.ctx().data_mut(|data| data.insert_temp(
            egui::Id::new("settings-topic"), topic.response.rect));
        ui.small("出題の設定は次のセッションから反映する。");
        ui.add_space(8.0);
        ui.label("画面の拡大率");
        if ui.ww_button(format!("{:.0}% — 倍率を調整", self.settings_editor.draft.font_scale * 100.0)).clicked() {
            self.settings_editor.zoom_open = true;
        }
        ui.small("固定位置の操作パネルで連続プレビューする。保存で確定、キャンセルで元に戻る。起動時は標準の80%。");
    }

    fn voice_settings_panel(&mut self, ui: &mut egui::Ui) {
        home_art::title(ui, "読み上げと音声入力", 22.0);
        ui.label("読み上げに使用する音声");
        let current_voice = self
            .speaker
            .voices
            .iter()
            .find(|v| v.0 == self.settings_editor.draft.voice_id)
            .map(|v| v.1.clone())
            .unwrap_or_else(|| "英語の音声を自動選択".into());
        egui::ComboBox::from_id_salt("voice")
            .width(ui.available_width().min(440.0))
            .truncate()
            .selected_text(current_voice)
            .show_ui(ui, |ui| {
                ui.ww_selectable_value(
                    &mut self.settings_editor.draft.voice_id,
                    String::new(),
                    "英語の音声を自動選択",
                );
                for (id, name) in &self.speaker.voices {
                    ui.ww_selectable_value(&mut self.settings_editor.draft.voice_id, id.clone(), name);
                }
            });
        ui.label("読み上げ速度（倍）");
        let mut rate = self.settings_editor.draft.speech_rate.unwrap_or(
            if self.settings_editor.draft.slow_speech { 0.8 } else { 1.0 });
        if ui
            .add(egui::Slider::new(&mut rate, 0.5..=4.0).step_by(0.1))
            .changed()
        {
            self.settings_editor.draft.speech_rate = Some(rate);
        }
        if ui.ww_button("音声を確認する").clicked() {
            let settings = self.settings_editor.draft.clone();
            self.say_with_settings("We appreciate your assistance.", &settings, true);
        }
        ui.small(
            "読み上げはWindowsの音声合成。マイクはWindowsで設定した既定の入力デバイスを使う。",
        );
    }

    fn connection_settings_panel(&mut self, ui: &mut egui::Ui) {
        home_art::title(ui, "AI接続・モデル・effort", 22.0);
        ui.label(
            "Codex CLIをインストールし、ターミナルで codex login を実行してChatGPTでログインする。",
        );
        ui.small("APIキーは使用しない。生成はChatGPT契約の利用枠を使用し、上限到達時は停止する。");
        ui.add_space(8.0);
        ui.label("Codex実行ファイル");
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.settings_editor.draft.codex_path)
                .id_salt("codex-executable-input")
                .background_color(ux::TINT)
                .desired_width(ui.available_width().min(720.0)),
        );
        #[cfg(test)]
        ui.ctx().data_mut(|data| data.insert_temp(
            egui::Id::new("settings-path-input"), (response.id, response.rect, ui.clip_rect())));
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
            self.settings_editor.draft.codex_path = "codex".into();
        }
        if ui.ww_button("Codex実行ファイルを選択").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Codex", &["exe", "cmd", "bat"])
                .pick_file()
            {
                self.settings_editor.draft.codex_path = path.to_string_lossy().into();
            }
        }
        ui.small("実行ファイル欄をcodexにすると、起動時PATH → システムPATH → ユーザーPATH → npmの順に探索する。特定のCLIを使う場合はファイルを選択する。引数は入力しない。");
        ui.add_space(8.0);
        ui.label("要求するモデル（空欄はCodexの既定値）");
        ui.add(
            egui::TextEdit::singleline(&mut self.settings_editor.draft.codex_model)
                .background_color(ux::TINT)
                .desired_width(ui.available_width().min(440.0)),
        );
        self.effort_settings(ui);
        ui.small(
            "ここは次回の要求設定。実際のモデル・effortは、Codexの返却後に画面下部へ表示する。",
        );
        ui.label(self.connection_label_for(&self.settings_editor.draft.codex_path));
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
        home_art::title(ui, "教材・バックアップ", 22.0);
        let idle = self.session.is_none() && self.pending.is_none() && self.recorder.is_none();
        ui.label("教材の共有・編集（バックアップとは別の操作）");
        ui.horizontal_wrapped(|ui| {
            if ui.ww_button("教材をTSVに書き出す").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("wordweave-deck.tsv")
                    .add_filter("TSV", &["tsv"])
                    .save_file()
                {
                    self.message =
                        store::atomic_write(&path, model::deck_text(&self.deck).as_bytes())
                            .map(|_| "教材TSVを書き出した。「語彙を追加」の③から追加・更新できる。".into())
                            .unwrap_or_else(|e| e);
                }
            }
            if ui
                .add_enabled(
                    idle && self.fatal.is_none(),
                    crate::app::controls::Button::new("語彙・教材の追加へ"),
                )
                .clicked()
            {
                self.page = Page::Words;
            }
        });
        ui.small("教材TSVは意味・解説・例文を共有・編集するためのファイル。「語彙を追加」の③で読み込む。学習記録・設定・添付原本は含まない。");
        ui.add_space(8.0);
        ui.label("学習記録のバックアップ・復元（JSON）");
        ui.horizontal_wrapped(|ui| {
            if ui.ww_button("学習記録をバックアップ").clicked() {
                self.credit_time();
                if let Some(path) = rfd::FileDialog::new().set_file_name("wordweave-progress.json")
                    .add_filter("JSON", &["json"]).save_file() {
                    let result = serde_json::to_vec_pretty(&self.progress).map_err(|e| e.to_string())
                        .and_then(|bytes| store::atomic_write(&path, &bytes));
                    self.message = result.map(|_| "学習記録をバックアップした。このJSONに教材TSV・録音・筆跡原本は含まれない。媒体付きバックアップも使用してください。".into()).unwrap_or_else(|e| e);
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
        ui.small("復元は追加取り込みではない。現在の学習記録・設定・チャットを退避したうえで、JSONに保存された内容へ置き換える。教材・録音・筆跡原本はこのJSONに含まれない。");
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
        home_art::title(ui, "接続の診断", 22.0);
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
