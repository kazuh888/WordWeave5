use super::*;
use super::ux;

fn visible_candidates(words: &[String], query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    words.iter().filter(|word| word.to_lowercase().contains(&query))
        .take(100).cloned().collect()
}

impl WordApp {
    pub(super) fn words_page(&mut self, ui: &mut egui::Ui) {
        ux::heading(ui, "語彙を追加", "学びたい語やフレーズを選び、教材解説と言い換え・例文を生成する。");
        let idle = self.session.is_none() && self.pending.is_none()
            && self.recorder.is_none() && !self.batch_running && self.fatal.is_none();
        self.vocabulary_generation_settings(ui, idle);
        ui.add_space(12.0);
        ui.add_enabled_ui(idle, |ui| {
            self.vocabulary_ngsl(ui);
            ui.add_space(12.0);
            self.vocabulary_provided(ui);
            ui.add_space(12.0);
            self.vocabulary_file(ui);
        });
        ui.add_space(12.0);
        self.vocabulary_candidates(ui, idle);
    }

    fn vocabulary_generation_settings(&mut self, ui: &mut egui::Ui, idle: bool) {
        ux::panel(ui, false, |ui| {
            ui.strong("生成・登録の設定（すべての追加方法に共通）");
            ui.add_enabled_ui(idle, |ui| {
                if ui.add(egui::Slider::new(&mut self.progress.settings.batch_words, 1..=3000)
                    .logarithmic(true).text("1回に追加する語数")).changed() {
                    self.dirty = true;
                }
                if ui.add(egui::Slider::new(&mut self.progress.settings.examples_per_word, 3..=12)
                    .text("1語あたりの例文数")).changed() {
                    self.dirty = true;
                }
            });
            ui.label(format!("本日の生成試行：{} / {}回　未処理：{}語",
                self.progress.ai_calls.get(&today()).copied().unwrap_or(0),
                self.progress.settings.ai_daily_limit, self.batch_queue.len()));
            ui.small("生成は1語ずつ実行し、形式検査後に自動保存する。内容は教材画面で確認・編集できる。1日の上限は設定で変更できる。");
            if !self.batch_queue.is_empty()
                && ui.add_enabled(idle, egui::Button::new("未処理の語から再開")).clicked() {
                self.batch_running = true;
            }
            if !idle {
                ui.horizontal_wrapped(|ui| {
                    if self.pending.is_some() || self.batch_running { ui.spinner(); }
                    ui.label(if self.batch_running { "生成中。画面下の中断操作で止められる。登録済みの教材は保持する。" }
                        else { "学習・録音・通信の終了後に追加操作ができる。" });
                });
                if !self.message.is_empty() { ui.label(&self.message); }
            }
        });
    }

    fn vocabulary_ngsl(&mut self, ui: &mut egui::Ui) {
        ux::panel(ui, true, |ui| {
            ui.label(RichText::new("1　NGSLから自動生成・登録").size(22.0).strong());
            ui.label("同梱の基本語リストから、生成済みの語を除いて追加する。");
            if ux::primary(ui, "NGSLから生成・登録", true).clicked() {
                self.begin_batch(learning::parse_words(learning::BUNDLED_NGSL).unwrap_or_default());
            }
            ui.collapsing("出典・公式リストの再取得", |ui| {
                ui.hyperlink_to("NGSL公式・出典", learning::NGSL_PAGE);
                ui.small("NGSL 1.2 / Browne, Culligan & Phillips / CC BY-SA 4.0。公式の頻度順リストと補足語を同梱。教材解説と例文はCodexが生成する。");
                if ui.button("公式NGSLを再取得して自動登録").clicked() {
                    self.fetch_then_generate = true;
                    let (tx, rx) = mpsc::channel();
                    std::thread::spawn(move || {
                        let _ = tx.send(ai::download_words().map(AiResult::Words));
                    });
                    self.pending = Some(Pending { key: String::new(), rx, cancel: None, kind: Activity::Download });
                    self.message = "NGSL公式CSVを取得中…".into();
                }
            });
        });
    }

    fn vocabulary_provided(&mut self, ui: &mut egui::Ui) {
        ux::panel(ui, false, |ui| {
            ui.label(RichText::new("2　入力した語から自動生成・登録").size(22.0).strong());
            ui.label("1行に1語または1フレーズを入力する。");
            ui.add(egui::TextEdit::multiline(&mut self.provided_words).desired_rows(4)
                .desired_width(f32::INFINITY).hint_text("take\nlook forward to\nas soon as"));
            if ux::primary(ui, "入力した語から生成・登録", !self.provided_words.trim().is_empty()).clicked() {
                let text = self.provided_words.clone();
                match learning::parse_words(&text).and_then(|words| {
                    self.cache_words(&text)?;
                    Ok(words)
                }) {
                    Ok(words) => self.begin_batch(words),
                    Err(error) => self.message = error,
                }
            }
            ui.small("入力した語は登録開始時に候補として保存する。途中で止まった語は上の「未処理の語から再開」で続けられる。");
        });
    }

    fn vocabulary_file(&mut self, ui: &mut egui::Ui) {
        ux::panel(ui, false, |ui| {
            ui.label(RichText::new("3　ファイルの語から自動生成・登録").size(22.0).strong());
            ui.label("UTF-8のCSV・TSV・TXTを読み込む（2MBまで）。");
            ui.small("先頭列が語、または順位・語の順。Lemma / Word / Headword列にも対応する。");
            if ux::primary(ui, "ファイルを選んで生成・登録", true).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("語彙（UTF-8）", &["csv", "tsv", "txt"]).pick_file() {
                    match read_limited(&path, 2_000_000).and_then(|text| {
                        let words = learning::parse_words(&text)?;
                        self.cache_words(&text)?;
                        Ok(words)
                    }) {
                        Ok(words) => self.begin_batch(words),
                        Err(error) => self.message = error,
                    }
                }
            }
            ui.small("ファイル選択後は共通設定の語数まで生成・登録を開始する。教材TSVそのものの取り込みは「設定」で行う。");
        });
    }

    fn vocabulary_candidates(&mut self, ui: &mut egui::Ui, idle: bool) {
        ux::panel(ui, false, |ui| {
            ui.strong(format!("保存された候補から1語を選ぶ（全{}語）", self.words.len()));
            ui.add(egui::TextEdit::singleline(&mut self.word_search)
                .hint_text("候補を検索").desired_width(f32::INFINITY));
            let visible = visible_candidates(&self.words, &self.word_search);
            if visible.is_empty() {
                ui.label("該当する候補はない。検索する文字を変えるか、上の入力・ファイル読込から追加する。");
            } else {
                ui.small("一致する候補を最大100語表示する。");
                egui::ScrollArea::vertical().id_salt("word-candidates").max_height(180.0)
                    .show(ui, |ui| {
                        for word in &visible {
                            if ui.add(egui::Button::new(word)
                                .selected(self.selected_word == *word).wrap()).clicked() {
                                self.selected_word = word.clone();
                            }
                        }
                    });
            }
            let selected_is_visible = visible.contains(&self.selected_word);
            if selected_is_visible { ui.label(format!("選択：{}", self.selected_word)); }
            if ui.add_enabled(idle && selected_is_visible,
                egui::Button::new("選択した語を生成・登録")).clicked() {
                self.begin_batch(vec![self.selected_word.clone()]);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::visible_candidates;

    #[test]
    fn candidate_search_is_case_insensitive_and_bounded() {
        let words = vec!["take".into(), "look forward to".into()];
        assert_eq!(visible_candidates(&words, " TAKE "), vec!["take"]);
        assert!(visible_candidates(&words, "absent").is_empty());
        let many: Vec<String> = (0..150).map(|index| format!("word{index}")).collect();
        assert_eq!(visible_candidates(&many, "").len(), 100);
    }
}
