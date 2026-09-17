use super::*;
use wordweave5::{material::Draft, material_diff};
impl WordApp {
    pub(super) fn material_comparison(&mut self, ui: &mut egui::Ui, draft: &Draft) {
        let rows = material_diff::rows(draft.baseline.as_ref(), &draft.candidate);
        let key = egui::Id::new(("material-change", &draft.source.entry_id));
        let mut selected = ui
            .ctx()
            .data_mut(|d| d.get_temp::<String>(key))
            .unwrap_or_default();
        let reasons = draft.active_reasons();
        ui.label(
            "変更箇所を選ぶと右側に理由と根拠を表示する。色だけでなく追加・変更・削除を表示する。",
        );
        if draft.generated.is_some() && reasons.is_empty() && !draft.reasons.is_empty() {
            ui.label("手動編集後：AIの理由は現在の差分に適用しない。元の根拠は保持している。");
        }
        let mut image = None;
        let mut audio = None;
        ui.columns(2,|columns| {
            columns[0].set_width(columns[0].available_width());
            columns[0].horizontal(|ui|{ui.strong("変更前");ui.label("｜");ui.strong("登録される内容");});
            egui::ScrollArea::vertical().id_salt("paired-diff").max_height(420.0).show(&mut columns[0],|ui|{
                if rows.is_empty(){ui.label("内容の差分はない。");}
                for row in &rows {
                    let relevant:Vec<_>=reasons.iter().filter(|r|r.path==row.path||r.path.starts_with(&format!("{}/",row.path))).collect();
                    let reason=if relevant.is_empty(){"理由未取得（推測で補完しない）".into()}else{relevant.iter().map(|r|r.reason.as_str()).collect::<Vec<_>>().join("\n")};
                    if ui.selectable_label(selected==row.path,format!("{}：{}",row.kind,row.label)).on_hover_text(&reason).clicked(){selected=row.path.clone();}
                    let(a,b)=material_diff::spans(&row.before,&row.after);
                    ui.columns(2,|sides|{
                        sides[0].add(egui::Label::new(diff_text(&a,Color32::from_rgb(255,217,217))).wrap().selectable(true)).on_hover_text(&reason);
                        sides[1].add(egui::Label::new(diff_text(&b,Color32::from_rgb(207,239,214))).wrap().selectable(true)).on_hover_text(&reason);
                    });ui.separator();
                }
            });
            columns[1].strong("変更理由・根拠のチャット（生成時の固定版）");
            let selected_reasons:Vec<_>=reasons.iter().filter(|r|!selected.is_empty() && (r.path==selected||r.path.starts_with(&format!("{selected}/")))).collect();
            for r in &selected_reasons {columns[1].label(&r.reason);}
            if selected_reasons.is_empty(){columns[1].small("理由未取得、または変更箇所が未選択である。");}
            egui::ScrollArea::vertical().id_salt("source-diff").max_height(420.0).show(&mut columns[1],|ui|{
                if draft.source.snapshots.is_empty(){ui.label("旧版の案には固定原文がない。「元の会話を表示」で現在の履歴を参照する。");}
                for s in &draft.source.snapshots {
                    ui.push_id(s.exchange_index,|ui|{
                        ui.strong(format!("往復 {}",s.exchange_index+1));
                        for (role,label,text) in [("user","あなた",&s.exchange.question),("assistant","Codex",&s.exchange.answer)] {
                            ui.small(label);
                            let quotes:Vec<_>=selected_reasons.iter().flat_map(|r|r.quotes.iter()).filter(|q|q.exchange_index==s.exchange_index&&q.role==role).map(|q|q.quote.as_str()).collect();
                            ui.add(egui::Label::new(quoted_text(text,&quotes)).wrap().selectable(true));
                        }
                        for a in &s.exchange.attachments {
                            ui.small(format!("媒体の根拠：{}",a.source_text));
                            if let Some(r)=&a.image {if ui.button("注釈画像を表示").clicked(){image=Some(r.clone());}}
                            if a.original.kind==wordweave5::assets::AssetKind::AudioWav && ui.add_enabled(self.pending.is_none(),egui::Button::new("原録音を再生")).clicked(){audio=Some(a.original.clone());}
                        }
                        ui.separator();
                    });
                }
            });
        });
        ui.ctx().data_mut(|d| d.insert_temp(key, selected));
        if let Some(r) = image {
            self.preview_asset(&r);
        }
        if let Some(r) = audio {
            self.play_asset(&r);
        }
    }
}
fn diff_text(spans: &[(String, bool)], color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (text, changed) in spans {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(16.0),
                color: Color32::from_gray(30),
                background: if *changed {
                    color
                } else {
                    Color32::TRANSPARENT
                },
                ..Default::default()
            },
        );
    }
    if spans.is_empty() {
        job.append("（なし）", 0.0, egui::TextFormat::default());
    }
    job
}
fn quoted_text(text: &str, quotes: &[&str]) -> egui::text::LayoutJob {
    let mut ranges = Vec::new();
    for quote in quotes {
        if !quote.is_empty() {
            for (start, _) in text.match_indices(quote) {
                ranges.push((start, start + quote.len()));
            }
        }
    }
    let mut groups: Vec<(String, bool)> = Vec::new();
    for (i, c) in text.char_indices() {
        let marked = ranges.iter().any(|(s, e)| *s <= i && i < *e);
        if groups.last().is_none_or(|(_, m)| *m != marked) {
            groups.push((String::new(), marked));
        }
        groups.last_mut().unwrap().0.push(c);
    }
    diff_text(&groups, Color32::from_rgb(255, 237, 168))
}
