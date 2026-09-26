use super::ux;
use super::*;

fn selected_material(selected: usize, matches: &[usize]) -> Option<usize> {
    matches
        .iter()
        .copied()
        .find(|index| *index == selected)
        .or_else(|| matches.first().copied())
}

impl WordApp {
    pub(super) fn deck_page(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 10.0);
            ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 15.0),
                (egui::TextStyle::Button, 18.0),
            ] {
                ui.style_mut()
                    .text_styles
                    .insert(style, super::home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(super::home_art::INK);
            egui::Frame::new()
                .inner_margin(16)
                .show(ui, |ui| self.materials_content(ui));
        });
    }

    fn materials_content(&mut self, ui: &mut egui::Ui) {
        if ui.available_height() >= 420.0 {
            ux::heading(
                ui,
                "教材を調べる",
                "意味と使い分けを確認し、必要なときに例文や言い換えを追加する。",
            );
        }
        egui::Frame::new()
            .fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
            .inner_margin(10)
            .corner_radius(8)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .frame(false)
                        .hint_text("基本語・表現・日本語で検索")
                        .desired_width(f32::INFINITY),
                );
            });
        let query = self.search.trim().to_lowercase();
        let matches: Vec<usize> = self
            .deck
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                !self.progress.deleted_entries.contains(&entry.id)
                    && (query.is_empty()
                        || format!(
                            "{} {} {} {} {}",
                            entry.base, entry.meaning, entry.business, entry.elevated, entry.usage
                        )
                        .to_lowercase()
                        .contains(&query))
            })
            .map(|(index, _)| index)
            .collect();
        ui.small(format!("{}項目 / 選択した教材の詳細を表示", matches.len()));
        ui.add_space(8.0);
        let Some(selected) = selected_material(self.selected, &matches) else {
            ux::panel(ui, false, |ui| {
                ui.strong("該当する教材はない");
                ui.label("検索する文字を変えるか、検索欄を空にすると一覧に戻る。");
                if self.search.is_empty() {
                    ui.label("教材を増やす場合は「語彙を追加」を開く。");
                } else if ui.ww_button("検索をクリア").clicked() {
                    self.search.clear();
                }
            });
            return;
        };
        self.selected = selected;
        let width = ui.available_width();
        if width >= 380.0 {
            let pane = egui::SidePanel::left("materials-list-pane")
                .resizable(true)
                .default_width((width * 0.32).clamp(160.0, 420.0))
                .width_range(140.0..=(width - 220.0).max(140.0))
                .show_inside(ui, |ui| {
                    self.material_list(ui, &matches, ui.available_height());
                });
            #[cfg(test)]
            ui.ctx().data_mut(|d| {
                d.insert_temp(egui::Id::new("test-material-split"), pane.response.rect)
            });
            #[cfg(not(test))]
            let _ = pane;
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show_inside(ui, |ui| {
                    self.material_detail_scroll(ui);
                });
        } else {
            let list_height = (ui.available_height() * 0.3).clamp(60.0, 160.0);
            self.material_list(ui, &matches, list_height);
            ui.add_space(12.0);
            self.material_detail_scroll(ui);
        }
    }

    fn material_detail_scroll(&mut self, ui: &mut egui::Ui) {
        let output = egui::ScrollArea::vertical()
            .id_salt("material-detail-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| self.material_detail(ui));
        #[cfg(test)]
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                egui::Id::new("test-material-detail"),
                (output.inner_rect, output.state.offset),
            )
        });
        #[cfg(not(test))]
        let _ = output;
    }

    fn material_detail(&mut self, ui: &mut egui::Ui) {
        let Some(entry) = self.deck.get(self.selected).cloned() else {
            return;
        };
        ui.push_id(&entry.id, |ui| {
            ux::panel(ui, false, |ui| {
                ui.small(format!(
                    "{} / {}水準（目安） / {}",
                    learning::kind(&entry),
                    entry.level,
                    entry.tag
                ));
                super::home_art::title(ui, &entry.base, 36.0);
                ui.label(RichText::new(&entry.meaning).size(22.0));
                ui.add_space(12.0);
                ui.strong("言い換え・語調");
                ui.label(format!("社外メール：{}", shown(&entry.business)));
                ui.label(format!("格調・文体：{}", shown(&entry.elevated)));
                ui.label(format!("語調・意味：{}", entry.register));
                if !entry.replacements.is_empty() {
                    ui.ww_collapsing(
                        format!("追加の言い換えと条件（{}件）", entry.replacements.len()),
                        |ui| {
                            for replacement in &entry.replacements {
                                ui.strong(&replacement.phrase);
                                ui.label(&replacement.meaning);
                                ui.label(&replacement.conditions);
                                ui.separator();
                            }
                        },
                    );
                }
                ui.separator();
                ui.strong("使い分けのポイント");
                ui.label(&entry.usage);
                ui.label(format!("場面：{}", entry.context));
                ui.add_space(12.0);
                ux::panel(ui, true, |ui| {
                    ui.strong("主な例文");
                    ui.label(RichText::new(entry.completed()).size(22.0).color(ux::INK));
                    ui.label(&entry.translation);
                    if ui
                        .add(
                            crate::app::controls::Button::new("例文を聞く")
                                .min_size(egui::vec2(150.0, 44.0)),
                        )
                        .clicked()
                    {
                        self.say(&entry.completed());
                    }
                });
                self.material_examples(ui, &entry);
                ui.ww_collapsing("学習問題と解説", |ui| {
                    ui.label(&entry.question);
                    ui.label(&entry.explanation);
                });
                self.material_sources(ui, &entry);
            });
            ui.add_space(12.0);
            self.material_management(ui, &entry);
        });
    }

    fn material_examples(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        if entry.examples.is_empty() {
            return;
        }
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 44.0;
            ui.spacing_mut().button_padding.x = 22.0;
            let _response = ui.ww_collapsing(
                format!("例文を開く（{}件）", entry.examples.len()),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("material-examples")
                        .max_height(480.0)
                        .min_scrolled_height(180.0)
                        .show(ui, |ui| {
                            for (index, example) in entry.examples.iter().enumerate() {
                                ui.strong(format!("{}. {}", index + 1, example.english));
                                ui.label(&example.japanese);
                                ui.label(&example.note);
                                if ui
                                    .ww_small_button(format!("例文{}を読み上げ", index + 1))
                                    .clicked()
                                {
                                    self.say(&example.english);
                                }
                                ui.separator();
                            }
                        });
                },
            );
            #[cfg(test)]
            ui.ctx().data_mut(|data| {
                data.insert_temp(
                    egui::Id::new("material-examples-header-rect"),
                    _response.header_response.rect,
                );
            });
        });
    }

    fn material_sources(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        let sources: Vec<_> = self
            .progress
            .material_sources
            .iter()
            .filter(|source| source.entry_id == entry.id)
            .cloned()
            .collect();
        if sources.is_empty() {
            return;
        }
        ui.ww_collapsing("教材の元になった会話", |ui| {
            for (index, source) in sources.iter().enumerate() {
                if ui
                    .add_enabled(
                        self.pending.is_none(),
                        crate::app::controls::Button::new(format!(
                            "{}：元の会話を開く（{}）",
                            index + 1,
                            source.mode.label()
                        ))
                        .wrap(),
                    )
                    .clicked()
                {
                    self.open_material_source(source);
                }
            }
        });
    }

    fn material_management(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        let idle = self.pending.is_none()
            && self.session.is_none()
            && self.recorder.is_none()
            && !self.batch_running
            && self.fatal.is_none();
        ux::panel(ui, false, |ui| {
            super::home_art::title(ui, "この教材でできること", 24.0);
            ui.small("例文・補足の言い換えの追加では復習状態を保持する。基本の説明や解答の訂正は再学習になる。");
            if !idle {
                ui.label("学習・録音・通信の終了後に編集できる。");
            }
            ui.add_enabled_ui(idle, |ui| {
                if ui.add_enabled(entry.examples.len() + self.progress.settings.examples_per_word <= 200,
                    crate::app::controls::Button::new("新しい場面の例文を生成・登録").wrap()
                        .fill(ux::TINT).min_size(egui::vec2(ui.available_width(), 44.0))).clicked() {
                    self.launch_content(1, Some(entry.clone()));
                }
                ui.ww_collapsing("日本文を登録して英文を生成", |ui| {
                    let draft = self.progress.japanese_drafts.entry(entry.id.clone()).or_default();
                    if ui.add(egui::TextEdit::multiline(draft).desired_rows(3)
                        .desired_width(f32::INFINITY).hint_text("この表現で伝えたい日本語を入力")).changed() {
                        self.dirty = true;
                    }
                    if ui.ww_button("日本文を登録し、英文を生成・登録").clicked() {
                        self.persist();
                        self.launch_content(2, Some(entry.clone()));
                    }
                    ui.small("日本語原文を先に保存する。生成失敗時も原文は残り、再試行できる。");
                });
                self.material_replacement_editor(ui, entry);
                ui.ww_collapsing("教材を編集（TSV）", |ui| {
                    if ui.ww_button("この教材を編集欄へ読み込む").clicked() {
                        self.draft_text = model::deck_text(std::slice::from_ref(entry));
                    }
                    if !self.draft_text.is_empty() {
                        ui.add(egui::TextEdit::multiline(&mut self.draft_text).desired_rows(7)
                            .desired_width(f32::INFINITY));
                        ui.small("保存する対象は編集欄のTSVである。末尾2列は言い換えと例文のJSON配列。設定からファイルの書き出し・取り込みもできる。");
                        if ui.ww_button("編集内容を確認して保存へ").clicked() {
                            match model::parse_deck(&self.draft_text) {
                                Ok(items) => self.pending_import = Some(items),
                                Err(error) => self.notify_error(error),
                            }
                        }
                    }
                });
                ui.separator();
                let mut suspended = self.progress.suspended.contains(&entry.id);
                if ui.checkbox(&mut suspended, "この教材を学習対象から外す（記録は保持）").changed() {
                    if suspended { self.progress.suspended.insert(entry.id.clone()); }
                    else { self.progress.suspended.remove(&entry.id); }
                    self.dirty = true;
                    self.persist();
                }
                ui.small("教材の削除ではない。チェックを外すと学習対象に戻る。");
            });
        });
    }

    fn material_replacement_editor(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        ui.ww_collapsing("言い換えを手動登録", |ui| {
            ui.label("置き換えの語句");
            ui.add(
                egui::TextEdit::singleline(&mut self.replacement_phrase)
                    .desired_width(f32::INFINITY),
            );
            ui.label("日本語の意味");
            ui.add(
                egui::TextEdit::singleline(&mut self.replacement_meaning)
                    .desired_width(f32::INFINITY),
            );
            ui.label("使える条件・意味の違い");
            ui.add(
                egui::TextEdit::multiline(&mut self.replacement_conditions)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            if ui.ww_button("言い換えを追加").clicked() {
                let mut changed = entry.clone();
                changed.replacements.push(model::Replacement {
                    phrase: self.replacement_phrase.trim().into(),
                    meaning: self.replacement_meaning.trim().into(),
                    conditions: self.replacement_conditions.trim().into(),
                });
                match self.import_deck(vec![changed]) {
                    Ok(()) => {
                        self.replacement_phrase.clear();
                        self.replacement_meaning.clear();
                        self.replacement_conditions.clear();
                    }
                    Err(error) => self.notify_error(error),
                }
            }
        });
    }

    fn material_list(&mut self, ui: &mut egui::Ui, matches: &[usize], height: f32) {
        ux::panel(ui, false, |ui| {
            if height >= 300.0 {
                super::home_art::title(ui, "教材一覧", 24.0);
                ui.small("選択した教材の詳細を確認");
            }
            let height = height.min(ui.available_height()).max(32.0);
            let output = egui::ScrollArea::vertical()
                .id_salt("deck-list")
                .max_height(height)
                .min_scrolled_height(0.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for &index in matches {
                        let entry = &self.deck[index];
                        let excluded = if self.progress.suspended.contains(&entry.id) {
                            " · 学習対象外"
                        } else {
                            ""
                        };
                        let text = format!(
                            "{} — {}{}",
                            entry.base.replace(['\r', '\n'], " "),
                            entry.meaning.replace(['\r', '\n'], " "),
                            excluded
                        );
                        if super::controls::list_row(ui, &text, self.selected == index).clicked() {
                            self.selected = index;
                        }
                    }
                });
            #[cfg(test)]
            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("test-material-list"),
                    (output.inner_rect, output.state.offset),
                )
            });
            #[cfg(not(test))]
            let _ = output;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::selected_material;

    #[test]
    fn panes_scroll_independently_and_splitter_resizes() {
        use super::*;
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(1.0);
        app.deck[0].usage = "Long detail paragraph. ".repeat(400);
        let mut frame = |events: Vec<egui::Event>| {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1150.0, 850.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.deck_page(ui));
                },
            );
        };
        let state = |key: &str| {
            ctx.data(|d| {
                d.get_temp::<(egui::Rect, egui::Vec2)>(egui::Id::new(key))
                    .unwrap()
            })
        };
        for _ in 0..3 {
            frame(vec![]);
        }
        let list = state("test-material-list");
        let detail = state("test-material-detail");
        let wheel = |pos| {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -180.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        frame(wheel(list.0.center()));
        for _ in 0..30 {
            frame(vec![]);
        }
        let list_scrolled = state("test-material-list").1.y;
        assert!(list_scrolled > 0.0);
        assert_eq!(state("test-material-detail").1.y, detail.1.y);
        frame(wheel(detail.0.center()));
        for _ in 0..30 {
            frame(vec![]);
        }
        assert!(state("test-material-detail").1.y > 0.0);
        assert!((state("test-material-list").1.y - list_scrolled).abs() < 1.0);
        let split = || {
            ctx.data(|d| {
                d.get_temp::<egui::Rect>(egui::Id::new("test-material-split"))
                    .unwrap()
            })
        };
        let before = split();
        let from = egui::pos2(before.right(), before.center().y);
        let to = from + egui::vec2(80.0, 0.0);
        frame(vec![egui::Event::PointerMoved(from)]);
        frame(vec![egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }]);
        frame(vec![egui::Event::PointerMoved(to)]);
        frame(vec![egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(split().width() > before.width() + 40.0);
        drop(frame);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn material_layout_fits_and_does_not_change_learning_data() {
        use super::*;
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(1.0);
        let before = serde_json::to_value(&app.progress).unwrap();
        let deck_before = model::deck_text(&app.deck);
        for query in ["", "sorry", "___no_material_matches___"] {
            app.search = query.into();
            for width in [1400.0, 760.0, 440.0] {
                for _ in 0..3 {
                    let _ = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 950.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                let right = ui.max_rect().right();
                                app.deck_page(ui);
                                assert!(
                                    ui.min_rect().right() <= right + 1.0,
                                    "width={width}: {:?}",
                                    ui.min_rect()
                                );
                            });
                        },
                    );
                }
                if let Some(rect) = ctx.data(|data| {
                    data.get_temp::<egui::Rect>(egui::Id::new("material-examples-header-rect"))
                }) {
                    assert!(rect.width() >= 150.0, "width={width}: {rect:?}");
                    assert!(rect.height() >= 43.0, "width={width}: {rect:?}");
                    assert!(rect.right() <= ctx.screen_rect().right() + 1.0);
                }
            }
        }
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(model::deck_text(&app.deck), deck_before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_search_results_never_keep_an_unrelated_detail() {
        assert_eq!(selected_material(3, &[]), None);
    }

    #[test]
    fn search_keeps_visible_selection_or_selects_first_match() {
        assert_eq!(selected_material(3, &[1, 3, 7]), Some(3));
        assert_eq!(selected_material(3, &[1, 7]), Some(1));
    }
}
