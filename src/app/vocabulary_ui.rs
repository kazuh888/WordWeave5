use super::ux;
use super::*;

const TXT_WORD_LIST_EXAMPLE: &str = "take\nlook forward to\nas soon as";
const CSV_HEADWORD_EXAMPLE: &str = "Headword,Meaning\nhappy,うれしい\nlook forward to,楽しみにする";
const TSV_LEMMA_EXAMPLE: &str = "Lemma\tMeaning\ntake\t取る\ngo\t行く";

// Keep the three entry points visually consistent without changing their actions.
fn method_card(
    ui: &mut egui::Ui,
    number: &str,
    title: &str,
    color: Color32,
    label: &str,
    enabled: bool,
    content: impl FnOnce(&mut egui::Ui),
) -> bool {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
        .corner_radius(12)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width((width - 26.0).max(0.0));
            let galley = ui.fonts(|f| {
                f.layout(
                    title.into(),
                    super::home_art::home_font(24.0),
                    color,
                    (ui.available_width() - 52.0).max(1.0),
                )
            });
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), galley.size().y.max(36.0)),
                egui::Sense::hover(),
            );
            response
                .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, title));
            let badge = egui::pos2(rect.left() + 18.0, rect.center().y);
            ui.painter().circle_filled(badge, 18.0, color);
            let digit = ui.fonts(|f| {
                f.layout_no_wrap(
                    number.into(),
                    super::home_art::home_font(22.0),
                    Color32::WHITE,
                )
            });
            ui.painter().galley(
                badge - digit.mesh_bounds.center().to_vec2(),
                digit,
                Color32::WHITE,
            );
            ui.painter().galley(
                egui::pos2(
                    rect.left() + 52.0,
                    rect.center().y - galley.mesh_bounds.center().y,
                ),
                galley,
                color,
            );
            let action = |ui: &mut egui::Ui| {
                let response = ui.add_enabled(
                    enabled,
                    crate::app::controls::Button::new(
                        RichText::new(label)
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(color)
                    .min_size(egui::vec2(ui.available_width().min(310.0), 52.0))
                    .wrap(),
                );
                #[cfg(test)]
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("vocabulary-action", number)),
                        (response.rect, response.enabled()),
                    )
                });
                response.clicked()
            };
            if ui.available_width() >= 850.0 {
                let details_width = ui.available_width() - 334.0;
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(details_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(details_width);
                            content(ui);
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(310.0, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        action,
                    )
                    .inner
                })
                .inner
            } else {
                content(ui);
                ui.add_space(8.0);
                action(ui)
            }
        })
        .inner
}

fn visible_candidates(words: &[String], query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();
    words
        .iter()
        .filter(|word| word.to_lowercase().contains(&query))
        .cloned()
        .collect()
}

fn file_example(ui: &mut egui::Ui, title: &str, description: &str, example: &str) {
    ui.strong(title);
    ui.label(description);
    egui::Frame::new()
        .fill(ux::TINT)
        .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
        .inner_margin(10)
        .corner_radius(6)
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(example).monospace().color(ux::INK))
                    .selectable(true)
                    .wrap(),
            );
        });
}

fn vocabulary_file_help(ui: &mut egui::Ui) {
    ui.strong("列名の選び方");
    ui.label("Word：一般的な単語列。自分で作るファイルはWordでよい。");
    ui.label(
        "Headword：辞書や語彙リストの見出し語の列。追加したい英単語・フレーズをそのまま書く。",
    );
    ui.label("Lemma：活用語をまとめる原形・基本形の列。tookではなくtakeなど、原形を自分で書く。本ツールは活用形を自動変換しない。");
    ui.label("列名はWord・Headword・Lemmaのどれを使ってもよい。公開されている語彙ファイルでは列名が異なるため、列名を変更せず利用しやすいよう3種類に対応している。");
    ui.small("見出しがなければ先頭列、または「順位・単語」の2列を読み取る。");
    ui.add_space(8.0);
    ui.separator();
    file_example(
        ui,
        "TXT（1行に1語・1フレーズ）",
        "見出し行は不要である。文字コードはUTF-8を使う。",
        TXT_WORD_LIST_EXAMPLE,
    );
    ui.add_space(8.0);
    ui.separator();
    file_example(
        ui,
        "CSV（カンマ区切り）の例：Headwordを使う場合",
        "Headword列に、追加したい英単語・フレーズを書く。",
        CSV_HEADWORD_EXAMPLE,
    );
    ui.add_space(8.0);
    ui.separator();
    file_example(
        ui,
        "TSV（タブ区切り）の例：Lemmaを使う場合",
        "列の間をタブで区切り、Lemma列には原形を書く。",
        TSV_LEMMA_EXAMPLE,
    );
    ui.small("Meaningなど他の列は、候補や教材の内容として取り込まない。英語の語の列だけをAI生成の対象にする。");
    ui.add_space(8.0);
    ui.separator();
    ui.strong("完成済み教材TSVについて");
    ui.label("③は列名で自動判定する。Word・Headword・Lemma列は単語リストとしてAIで教材を生成する。id・base・meaningなど15列または17列を持つ完成済み教材TSVは、AI生成せず追加・更新する。列の順序は変更できるが、列名は残す。");
    ui.label("完成済み教材は、追加・変更の差分を確認してから取り込む。同じIDは更新、新しいIDは追加であり、バックアップの復元とは異なる。教材形式に不備がある場合は停止し、単語リストとして生成し直すことはない。");
}

impl WordApp {
    pub(super) fn words_page(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(16.0, 8.0);
            ui.spacing_mut().interact_size.y = 44.0;
            ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
            for (style, size) in [
                (egui::TextStyle::Body, 19.0),
                (egui::TextStyle::Small, 16.0),
                (egui::TextStyle::Button, 19.0),
            ] {
                ui.style_mut()
                    .text_styles
                    .insert(style, super::home_art::home_font(size));
            }
            ui.style_mut().visuals.override_text_color = Some(ux::INK);
            egui::Frame::new()
                .inner_margin(16)
                .show(ui, |ui| self.words_content(ui));
        });
    }

    fn words_content(&mut self, ui: &mut egui::Ui) {
        #[cfg(debug_assertions)]
        if ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::new("preview-vocabulary-file")).unwrap_or(false)) {
            self.vocabulary_file(ui);
            return;
        }
        #[cfg(debug_assertions)]
        if ui.ctx().data(|d| {
            d.get_temp::<bool>(egui::Id::new("preview-vocabulary-candidates"))
                .unwrap_or(false)
        }) {
            self.vocabulary_candidates(ui, true);
            return;
        }
        #[cfg(debug_assertions)]
        if ui.ctx().data(|d| {
            d.get_temp::<bool>(egui::Id::new("preview-vocabulary-file-help"))
                .unwrap_or(false)
        }) {
            ux::panel(ui, false, |ui| {
                super::home_art::title(ui, "ファイルの書き方", 26.0);
                vocabulary_file_help(ui);
            });
            return;
        }
        ux::panel(ui, true, |ui| {
            ui.horizontal_wrapped(|ui| {
                super::home_art::title(ui, "語彙を追加", 28.0);
                if ui.available_width() >= 600.0 {
                    ui.label("学びたい語を、あなたの教材に。4つの方法から選ぶ。");
                }
            });
        });
        ui.add_space(4.0);
        let idle = self.session.is_none()
            && self.pending.is_none()
            && self.recorder.is_none()
            && !self.batch_running
            && self.fatal.is_none();
        self.vocabulary_generation_settings(ui, idle);
        ui.add_space(4.0);
        ui.add_enabled_ui(idle, |ui| {
            self.vocabulary_ngsl(ui);
            ui.add_space(4.0);
            self.vocabulary_provided(ui);
            ui.add_space(4.0);
            self.vocabulary_file(ui);
        });
        ui.add_space(4.0);
        self.vocabulary_candidates(ui, idle);
    }

    fn vocabulary_generation_settings(&mut self, ui: &mut egui::Ui, idle: bool) {
        ux::panel(ui, false, |ui| {
            ui.strong("AIで生成する場合の設定（完成済み教材TSVには適用しない）");
            ui.add_enabled_ui(idle, |ui| {
                let compact = ui.available_width() < 620.0;
                let controls = |ui: &mut egui::Ui| {
                    ui.horizontal(|ui| {
                        ui.label("1回の追加上限");
                        let response = ui.add_sized(
                            egui::vec2(104.0, 40.0),
                            egui::DragValue::new(&mut self.progress.settings.batch_words)
                                .range(1..=3000)
                                .suffix(" 語"),
                        );
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(
                                egui::Id::new(("vocabulary-generation-field", "words")),
                                response.rect,
                            );
                        });
                        if response.changed() {
                            self.dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("1語あたり");
                        let response = ui.add_sized(
                            egui::vec2(104.0, 40.0),
                            egui::DragValue::new(&mut self.progress.settings.examples_per_word)
                                .range(3..=12)
                                .suffix(" 例文"),
                        );
                        #[cfg(test)]
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(
                                egui::Id::new(("vocabulary-generation-field", "examples")),
                                response.rect,
                            );
                        });
                        if response.changed() {
                            self.dirty = true;
                        }
                    });
                    ui.label(format!(
                        "本日の生成試行：{} / {}回",
                        self.progress.ai_calls.get(&today()).copied().unwrap_or(0),
                        self.progress.settings.ai_daily_limit
                    ));
                };
                if compact {
                    ui.vertical(controls);
                } else {
                    ui.horizontal_wrapped(controls);
                }
            });
            ui.small("生成は1語ずつ実行し、形式検査後に自動保存する。内容は教材画面で確認・編集できる。1日の上限は設定で変更できる。");
            if !self.saved_generated_words.is_empty() {
                ui.strong(format!(
                    "この起動中に登録した語：{}語",
                    self.saved_generated_words.len()
                ));
                egui::ScrollArea::vertical()
                    .id_salt("saved-generated-words")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        ui.label(self.saved_generated_words.join(" / "));
                    });
                if ui.ww_button("登録した語をコピー").clicked() {
                    ui.ctx().copy_text(self.saved_generated_words.join("\n"));
                }
                ui.small(
                    "保存に成功した語のみ。再起動するとこの一覧は消えるが、教材は保持される。",
                );
            }
            if !self.batch_queue.is_empty() {
                ui.label(format!(
                    "未処理：{}語（登録済みの教材は保持）",
                    self.batch_queue.len()
                ));
                ui.label(format!(
                    "処理中／次に処理する語：{}",
                    self.batch_queue.front().unwrap()
                ));
            }
            if !self.batch_queue.is_empty()
                && ui
                    .add_enabled(
                        idle,
                        crate::app::controls::Button::new("未処理の語から再開"),
                    )
                    .clicked()
            {
                self.batch_running = true;
            }
            if !idle {
                ui.horizontal_wrapped(|ui| {
                    if self.pending.is_some() || self.batch_running {
                        ui.spinner();
                    }
                    ui.label(if self.batch_running {
                        "生成中。画面下の中断操作で止められる。登録済みの教材は保持する。"
                    } else {
                        "学習・録音・通信の終了後に追加操作ができる。"
                    });
                });
                if !self.message.is_empty() {
                    ui.label(&self.message);
                }
            }
        });
    }

    fn vocabulary_ngsl(&mut self, ui: &mut egui::Ui) {
        let mut fetch = false;
        let generate = method_card(
            ui,
            "1",
            "基本語リストから追加",
            ux::ACCENT,
            "NGSLから生成・登録",
            true,
            |ui| {
                ui.label("NGSLの掲載順に、自動生成済みの語を除き「1回の追加上限」まで追加する。");
                ui.small("未登録の全語を一度に追加する操作ではない。同梱教材にある語も対象となる。未処理の語があれば先に再開する。");
                ui.ww_collapsing("出典・公式リストの再取得", |ui| {
                ui.hyperlink_to("NGSL公式・出典", learning::NGSL_PAGE);
                ui.small("NGSL 1.2 / Browne, Culligan & Phillips / CC BY-SA 4.0。公式の頻度順リストと補足語を同梱。教材解説と例文はCodexが生成する。");
                fetch = ui.ww_button("公式NGSLを再取得して自動登録").clicked();
            });
            },
        );
        if generate {
            self.begin_batch(learning::parse_words(learning::BUNDLED_NGSL).unwrap_or_default());
        }
        if fetch {
            self.fetch_then_generate = true;
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(ai::download_words().map(AiResult::Words));
            });
            self.pending = Some(Pending {
                key: String::new(),
                rx,
                cancel: None,
                kind: Activity::Download,
            });
            self.message = "NGSL公式CSVを取得中…".into();
        }
    }

    fn vocabulary_provided(&mut self, ui: &mut egui::Ui) {
        let enabled = !self.provided_words.trim().is_empty();
        let generate = method_card(
            ui,
            "2",
            "自分で語やフレーズを入力",
            super::home_art::GREEN,
            "入力した語から生成・登録",
            enabled,
            |ui| {
                ui.label("1行に1語または1フレーズを入力する。");
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
                    .inner_margin(10)
                    .corner_radius(8)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.provided_words)
                                .frame(false)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY)
                                .hint_text("take\nlook forward to\nas soon as"),
                        );
                    });
                ui.small("入力した語は、まず④の候補一覧へ保存し、その後に教材を生成・登録する。エラーで教材登録に失敗しても候補は残るため、④から1語を選んで再実行できる。");
                ui.small("中断した未処理語は「未処理の語から再開」でまとめて続行できる。");
            },
        );
        if generate {
            let text = self.provided_words.clone();
            match learning::parse_words(&text).and_then(|words| {
                self.cache_words(&text)?;
                Ok(words)
            }) {
                Ok(words) => self.begin_batch(words),
                Err(error) => self.message = error,
            }
        }
    }

    fn vocabulary_file(&mut self, ui: &mut egui::Ui) {
        let generate = method_card(
            ui,
            "3",
            "CSV・TSV・TXTから追加",
            super::home_art::PURPLE,
            "ファイルを選んで追加",
            true,
            |ui| {
                ui.label("UTF-8のCSV・TSV・TXTを読み込む。列名で単語リストと完成済み教材を自動判定する。");
                ui.label(
                    "単語リスト（2MBまで）：AIが意味・解説・例文を新しく作って登録する。",
                );
                ui.small("単語リストの語は、まず④の候補一覧へ保存し、その後に教材を生成・登録する。エラーで教材登録に失敗しても候補は残るため、④から1語を選んで再実行できる。");
                ui.label("完成済み教材TSV（64MBまで）：AI生成せず、追加・変更の差分を確認してから取り込む。");
                ui.small("単語リストは共通設定の語数まで処理する。中断した未処理語は「未処理の語から再開」で続行できる。");
                ui.ww_collapsing("ファイルの書き方", |ui| {
                    vocabulary_file_help(ui);
                });
            },
        );
        if generate {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("単語リスト・教材（UTF-8）", &["csv", "tsv", "txt"])
                .pick_file()
            {
                if let Err(error) = read_limited(&path, 64_000_000)
                    .and_then(|text| self.prepare_vocabulary_file(&text)) {
                    self.message = error;
                }
            }
        }
    }

    fn prepare_vocabulary_file(&mut self, text: &str) -> Result<(), String> {
        if self.session.is_some() || self.pending.is_some() || self.recorder.is_some()
            || self.batch_running || self.fatal.is_some() || self.pending_import.is_some() {
            return Err("学習・録音・生成・確認を終了してからファイルを追加してください。".into());
        }
        match learning::parse_vocabulary_file(text)? {
            learning::VocabularyFile::Materials(items) => {
                self.message = "完成済み教材TSVとして読み込んだ。まだ登録していない。差分を確認してください。".into();
                self.pending_import = Some(items);
            }
            learning::VocabularyFile::Words(words) => {
                self.cache_words(&words.join("\n"))?;
                self.begin_batch(words);
            }
        }
        Ok(())
    }

    fn vocabulary_candidates(&mut self, ui: &mut egui::Ui, idle: bool) {
        ux::panel(ui, false, |ui| {
            ui.strong(format!(
                "④ 保存された単語から1語を選ぶ（全{}語）",
                self.words.len()
            ));
            let search_changed = egui::Frame::new()
                .fill(ux::TINT)
                .stroke(egui::Stroke::new(1.0_f32, ux::BORDER))
                .inner_margin(8)
                .corner_radius(6)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.word_search)
                            .frame(false)
                            .hint_text("英単語・フレーズを絞り込む（例：take）")
                            .desired_width(f32::INFINITY),
                    )
                })
                .inner
                .changed();
            let visible = visible_candidates(&self.words, &self.word_search);
            ui.label("検索対象は保存された単語リストである。初期状態はNGSLで、②・③で登録開始した語も追加される。教材本文・日本語の意味は検索しない。");
            ui.small(format!("{}語を表示 / 全{}語。入力文字を含む語を検索する（大文字・小文字は区別しない）。空欄なら全語を表示する。", visible.len(), self.words.len()));
            if visible.is_empty() {
                ui.label("該当する候補はない。検索する文字を変えるか、上の入力・ファイル読込から追加する。");
            } else {
                let mut scroll = egui::ScrollArea::vertical()
                    .id_salt("word-candidates")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .min_scrolled_height(280.0)
                    .max_height(320.0)
                    .auto_shrink([false, false]);
                if search_changed {
                    scroll = scroll.vertical_scroll_offset(0.0);
                }
                let output = scroll.show_rows(ui, 44.0, visible.len(), |ui, range| {
                    for word in &visible[range] {
                        if super::controls::list_row(ui, word, self.selected_word == *word)
                            .clicked()
                        {
                            self.selected_word = word.clone();
                        }
                    }
                });
                #[cfg(test)]
                ui.ctx().data_mut(|d| {
                    d.insert_temp(egui::Id::new("candidate-list-rect"), output.inner_rect)
                });
                #[cfg(not(test))]
                let _ = output;
            }
            let selected_is_visible = visible.contains(&self.selected_word);
            if selected_is_visible {
                ui.label(format!("選択：{}", self.selected_word));
            }
            if ui
                .add_enabled(
                    idle && selected_is_visible,
                    crate::app::controls::Button::new("選択した語を生成・登録"),
                )
                .clicked()
            {
                self.begin_batch(vec![self.selected_word.clone()]);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_file_waits_for_confirmation_without_saving_or_generating() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let before = serde_json::to_value(&app.progress).unwrap();
        let deck = model::deck_text(&app.deck);
        let words = app.words.clone();
        let saved = std::fs::read(root.join("data/progress.json")).unwrap();
        let mut changed = app.deck[0].clone(); changed.meaning = "確認用の変更".into();
        let text = model::deck_text(&[changed]);
        app.prepare_vocabulary_file(&text).unwrap();
        assert!(app.pending_import.is_some());
        assert!(app.pending.is_none() && !app.batch_running && app.batch_queue.is_empty());
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        assert_eq!(model::deck_text(&app.deck), deck);
        assert_eq!(app.words, words);
        assert_eq!(std::fs::read(root.join("data/progress.json")).unwrap(), saved);
        for _ in 0..3 { super::super::harness_tests::frame(&ctx, &mut app, false); }
        let cancel = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("material-import-cancel"))).unwrap();
        for pressed in [true, false] {
            let _ = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
                events: vec![egui::Event::PointerMoved(cancel.center()), egui::Event::PointerButton {
                    pos: cancel.center(), button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE }],
                ..Default::default() }, |ctx| app.update_ui(ctx));
        }
        assert!(app.pending_import.is_none());
        assert_eq!(model::deck_text(&app.deck), deck);
        assert!(app.prepare_vocabulary_file("id\tbase\na\thappy").is_err());
        assert_eq!(app.words, words);
        assert!(app.pending.is_none() && app.pending_import.is_none());
        assert_eq!(std::fs::read(root.join("data/progress.json")).unwrap(), saved);
        app.prepare_vocabulary_file(&text).unwrap();
        for _ in 0..3 { super::super::harness_tests::frame(&ctx, &mut app, false); }
        let apply = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new("material-import-apply"))).unwrap();
        for pressed in [true, false] {
            let _ = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
                events: vec![egui::Event::PointerMoved(apply.center()), egui::Event::PointerButton {
                    pos: apply.center(), button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE }],
                ..Default::default() }, |ctx| app.update_ui(ctx));
        }
        assert!(app.pending_import.is_none() && app.pending.is_none());
        assert_eq!(app.deck[0].meaning, "確認用の変更");
        assert_eq!(serde_json::to_value(&app.progress).unwrap()["chats"], before["chats"]);
        assert_eq!(std::fs::read_to_string(root.join("data/custom.tsv")).unwrap(), model::deck_text(&app.deck));
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn material_confirmation_actions_fit_a_small_viewport() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let mut item = app.deck[0].clone(); item.meaning = "確認用".into();
        app.prepare_vocabulary_file(&model::deck_text(&[item])).unwrap();
        for width in [512.5, 360.0] {
            for _ in 0..4 {
                let _ = ctx.run(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO, egui::vec2(width, 406.25))), ..Default::default() },
                    |ctx| app.confirmations(ctx));
            }
            for name in ["material-import-apply", "material-import-cancel"] {
                let rect = ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new(name))).unwrap();
                assert!(ctx.screen_rect().contains_rect(rect), "{name} inaccessible at {width}: {rect:?}");
            }
        }
        drop(app); std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn documented_word_file_examples_are_accepted() {
        assert_eq!(
            learning::parse_words(TXT_WORD_LIST_EXAMPLE).unwrap(),
            vec!["take", "look forward to", "as soon as"]
        );
        assert_eq!(
            learning::parse_words(CSV_HEADWORD_EXAMPLE).unwrap(),
            vec!["happy", "look forward to"]
        );
        assert_eq!(
            learning::parse_words(TSV_LEMMA_EXAMPLE).unwrap(),
            vec!["take", "go"]
        );
        for heading in ["Word", "Headword", "Lemma"] {
            assert_eq!(
                learning::parse_words(&format!(
                    "{heading},Meaning\ntake,取る\nlook forward to,楽しみにする"
                ))
                .unwrap(),
                vec!["take", "look forward to"]
            );
        }
    }

    #[test]
    fn file_help_examples_fit_standard_and_narrow_widths() {
        let ctx = egui::Context::default();
        for width in [1100.0, 480.0] {
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 1200.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            vocabulary_file_help(ui);
                            assert!(ui.min_rect().right() <= ctx.screen_rect().right() + 1.0);
                        });
                    },
                );
            }
        }
    }

    #[test]
    fn candidate_list_keeps_height_even_near_outer_scroll_bottom() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        for width in [1400.0, 480.0] {
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 650.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add_space(550.0);
                                app.vocabulary_candidates(ui, false);
                            });
                        });
                    },
                );
            }
            let rect = ctx
                .data(|d| d.get_temp::<egui::Rect>(egui::Id::new("candidate-list-rect")))
                .unwrap();
            assert!(rect.height() >= 280.0, "{rect:?}");
            assert!(rect.right() <= ctx.screen_rect().right());
            assert!(app.pending.is_none());
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn vocabulary_layout_preserves_drafts_and_disables_generation_when_busy() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let progress = serde_json::to_value(&app.progress).unwrap();
        let deck = serde_json::to_value(&app.deck).unwrap();
        let dirty = app.dirty;
        app.saved_generated_words = (0..100).map(|i| format!("word{i}")).collect();
        app.batch_queue.push_back("in".into());
        for width in [1400.0, 900.0, 480.0] {
            for draft in [
                "",
                "take\nlook forward to",
                "a very long phrase ".repeat(20).as_str(),
            ] {
                for busy in [false, true] {
                    app.provided_words = draft.into();
                    app.batch_running = busy;
                    for _ in 0..3 {
                        let _ = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 1100.0),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        app.words_page(ui);
                                        assert!(
                                            ui.min_rect().right()
                                                <= ctx.screen_rect().right() + 1.0,
                                            "horizontal overflow at {width}: {:?}",
                                            ui.min_rect()
                                        );
                                    });
                                });
                            },
                        );
                    }
                    for number in ["1", "2", "3"] {
                        let (rect, enabled) = ctx
                            .data(|d| {
                                d.get_temp::<(egui::Rect, bool)>(egui::Id::new((
                                    "vocabulary-action",
                                    number,
                                )))
                            })
                            .unwrap();
                        assert!(rect.height() >= 51.0);
                        assert!(rect.right() <= ctx.screen_rect().right() + 1.0);
                        assert_eq!(enabled, !busy && (number != "2" || !draft.is_empty()));
                    }
                    for field in ["words", "examples"] {
                        let rect = ctx
                            .data(|data| {
                                data.get_temp::<egui::Rect>(egui::Id::new((
                                    "vocabulary-generation-field",
                                    field,
                                )))
                            })
                            .unwrap();
                        assert!(rect.width() >= 103.0, "{field}: {rect:?}");
                        assert!(rect.height() >= 39.0, "{field}: {rect:?}");
                        assert!(rect.right() <= ctx.screen_rect().right() + 1.0);
                    }
                    if width < 620.0 {
                        let rect = |field| {
                            ctx.data(|data| {
                                data.get_temp::<egui::Rect>(egui::Id::new((
                                    "vocabulary-generation-field",
                                    field,
                                )))
                            })
                            .unwrap()
                        };
                        assert!(rect("examples").top() >= rect("words").bottom());
                    }
                    assert_eq!(app.provided_words, draft);
                    assert!(app.pending.is_none());
                    assert_eq!(
                        app.batch_queue
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                        vec!["in"]
                    );
                }
            }
        }
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), progress);
        assert_eq!(serde_json::to_value(&app.deck).unwrap(), deck);
        assert_eq!(app.dirty, dirty);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn candidate_search_is_case_insensitive_and_keeps_all_matches() {
        let words = vec!["take".into(), "look forward to".into()];
        assert_eq!(visible_candidates(&words, " TAKE "), vec!["take"]);
        assert!(visible_candidates(&words, "absent").is_empty());
        let many: Vec<String> = (0..150).map(|index| format!("word{index}")).collect();
        assert_eq!(visible_candidates(&many, "").len(), 150);
        assert_eq!(visible_candidates(&many, "word149"), vec!["word149"]);
    }
}
