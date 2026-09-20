use super::*;
use super::ux;

fn selected_material(selected: usize, matches: &[usize]) -> Option<usize> {
    matches.iter().copied().find(|index| *index == selected)
        .or_else(|| matches.first().copied())
}

impl WordApp {
    pub(super) fn deck_page(&mut self, ui: &mut egui::Ui) {
        ux::heading(ui, "教材を調べる", "意味と使い分けを確認し、必要なときに例文や言い換えを追加する。");
        ui.add(egui::TextEdit::singleline(&mut self.search)
            .hint_text("基本語・表現・日本語で検索").desired_width(f32::INFINITY));
        let query = self.search.trim().to_lowercase();
        let matches: Vec<usize> = self.deck.iter().enumerate().filter(|(_, entry)| {
            !self.progress.deleted_entries.contains(&entry.id)
                && (query.is_empty() || format!("{} {} {} {} {}", entry.base, entry.meaning,
                    entry.business, entry.elevated, entry.usage).to_lowercase().contains(&query))
        }).map(|(index, _)| index).collect();
        ui.small(format!("{}項目 / 選択した教材の詳細を表示", matches.len()));
        ui.add_space(8.0);
        let Some(selected) = selected_material(self.selected, &matches) else {
            ux::panel(ui, false, |ui| {
                ui.strong("該当する教材はない");
                ui.label("検索する文字を変えるか、検索欄を空にすると一覧に戻る。");
                if self.search.is_empty() {
                    ui.label("教材を増やす場合は「語彙を追加」を開く。");
                } else if ui.button("検索をクリア").clicked() {
                    self.search.clear();
                }
            });
            return;
        };
        self.selected = selected;
        if ui.available_width() >= 760.0 {
            let width = ui.available_width();
            let list_width = 240.0;
            let detail_width = width - list_width - ui.spacing().item_spacing.x;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(egui::vec2(list_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(list_width);
                        self.material_list(ui, &matches, 540.0);
                    });
                ui.allocate_ui_with_layout(egui::vec2(detail_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(detail_width);
                        self.material_detail(ui);
                    });
            });
        } else {
            self.material_list(ui, &matches, 190.0);
            ui.add_space(12.0);
            self.material_detail(ui);
        }
    }

    fn material_detail(&mut self, ui: &mut egui::Ui) {
        let Some(entry) = self.deck.get(self.selected).cloned() else { return; };
        ui.push_id(&entry.id, |ui| {
            ux::panel(ui, false, |ui| {
                ui.small(format!("{} / {}水準（目安） / {}", learning::kind(&entry), entry.level, entry.tag));
                ui.label(RichText::new(&entry.base).size(32.0).strong().color(ux::INK));
                ui.label(RichText::new(&entry.meaning).size(22.0));
                ui.add_space(12.0);
                ui.strong("言い換え・語調");
                ui.label(format!("社外メール：{}", shown(&entry.business)));
                ui.label(format!("格調・文体：{}", shown(&entry.elevated)));
                ui.label(format!("語調・意味：{}", entry.register));
                if !entry.replacements.is_empty() {
                    ui.collapsing(format!("追加の言い換えと条件（{}件）", entry.replacements.len()), |ui| {
                        for replacement in &entry.replacements {
                            ui.strong(&replacement.phrase);
                            ui.label(&replacement.meaning);
                            ui.label(&replacement.conditions);
                            ui.separator();
                        }
                    });
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
                    if ui.button("例文を聞く").clicked() { self.say(&entry.completed()); }
                });
                self.material_examples(ui, &entry);
                ui.collapsing("学習問題と解説", |ui| {
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
        if entry.examples.is_empty() { return; }
        ui.collapsing(format!("例文を開く（{}件）", entry.examples.len()), |ui| {
            egui::ScrollArea::vertical().id_salt("material-examples").max_height(320.0).show(ui, |ui| {
                for (index, example) in entry.examples.iter().enumerate() {
                    ui.strong(format!("{}. {}", index + 1, example.english));
                    ui.label(&example.japanese);
                    ui.label(&example.note);
                    if ui.small_button(format!("例文{}を読み上げ", index + 1)).clicked() {
                        self.say(&example.english);
                    }
                    ui.separator();
                }
            });
        });
    }

    fn material_sources(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        let sources: Vec<_> = self.progress.material_sources.iter()
            .filter(|source| source.entry_id == entry.id).cloned().collect();
        if sources.is_empty() { return; }
        ui.collapsing("教材の元になった会話", |ui| {
            for (index, source) in sources.iter().enumerate() {
                if ui.add_enabled(self.pending.is_none(), egui::Button::new(format!(
                    "{}：元の会話を開く（{}）", index + 1, source.mode.label())).wrap()).clicked() {
                    self.open_material_source(source);
                }
            }
        });
    }

    fn material_management(&mut self, ui: &mut egui::Ui, entry: &Entry) {
        let idle = self.pending.is_none() && self.session.is_none()
            && self.recorder.is_none() && !self.batch_running && self.fatal.is_none();
        ux::panel(ui, false, |ui| {
            ui.strong("この教材を追加・編集する");
            ui.small("例文・補足の言い換えの追加では復習状態を保持する。基本の説明や解答の訂正は再学習になる。");
            if !idle { ui.label("学習・録音・通信の終了後に編集できる。"); }
            ui.add_enabled_ui(idle, |ui| {
                if ui.add_enabled(entry.examples.len() + self.progress.settings.examples_per_word <= 200,
                    egui::Button::new("新しい場面の例文を生成・登録").wrap()).clicked() {
                    self.launch_content(1, Some(entry.clone()));
                }
                ui.collapsing("日本文を登録して英文を生成", |ui| {
                    let draft = self.progress.japanese_drafts.entry(entry.id.clone()).or_default();
                    if ui.add(egui::TextEdit::multiline(draft).desired_rows(3)
                        .desired_width(f32::INFINITY).hint_text("この表現で伝えたい日本語を入力")).changed() {
                        self.dirty = true;
                    }
                    if ui.button("日本文を登録し、英文を生成・登録").clicked() {
                        self.persist();
                        self.launch_content(2, Some(entry.clone()));
                    }
                    ui.small("日本語原文を先に保存する。生成失敗時も原文は残り、再試行できる。");
                });
                self.material_replacement_editor(ui, entry);
                ui.collapsing("教材を編集（TSV）", |ui| {
                    if ui.button("この教材を編集欄へ読み込む").clicked() {
                        self.draft_text = model::deck_text(std::slice::from_ref(entry));
                    }
                    if !self.draft_text.is_empty() {
                        ui.add(egui::TextEdit::multiline(&mut self.draft_text).desired_rows(7)
                            .desired_width(f32::INFINITY));
                        ui.small("保存する対象は編集欄のTSVである。末尾2列は言い換えと例文のJSON配列。設定からファイルの書き出し・取り込みもできる。");
                        if ui.button("編集内容を確認して保存へ").clicked() {
                            match model::parse_deck(&self.draft_text) {
                                Ok(items) => self.pending_import = Some(items),
                                Err(error) => self.message = error,
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
        ui.collapsing("言い換えを手動登録", |ui| {
            ui.label("置き換えの語句");
            ui.add(egui::TextEdit::singleline(&mut self.replacement_phrase).desired_width(f32::INFINITY));
            ui.label("日本語の意味");
            ui.add(egui::TextEdit::singleline(&mut self.replacement_meaning).desired_width(f32::INFINITY));
            ui.label("使える条件・意味の違い");
            ui.add(egui::TextEdit::multiline(&mut self.replacement_conditions)
                .desired_rows(3).desired_width(f32::INFINITY));
            if ui.button("言い換えを追加").clicked() {
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
                    Err(error) => self.message = error,
                }
            }
        });
    }

    fn material_list(&mut self, ui: &mut egui::Ui, matches: &[usize], height: f32) {
        ux::panel(ui, false, |ui| {
            ui.strong("教材一覧");
            egui::ScrollArea::vertical().id_salt("deck-list").max_height(height)
                .show(ui, |ui| {
                    for &index in matches {
                        let entry = &self.deck[index];
                        let excluded = if self.progress.suspended.contains(&entry.id) {
                            " · 学習対象外"
                        } else { "" };
                        let text = format!("{}\n{}\n{}{}", entry.base, entry.meaning, entry.tag, excluded);
                        if ui.add(egui::Button::new(text).selected(self.selected == index)
                            .wrap().min_size(egui::vec2(ui.available_width(), 64.0))).clicked() {
                            self.selected = index;
                        }
                    }
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::selected_material;

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
