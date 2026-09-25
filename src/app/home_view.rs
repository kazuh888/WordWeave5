//! Home presentation: receives computed data and returns explicit UI intents.
use super::home_art::*;
use eframe::egui::*;

pub(super) struct HomeView {
    pub registered: usize,
    pub answered: usize,
    pub due: usize,
    pub first_today: usize,
    pub seconds: u64,
    pub minutes: u32,
    pub resuming: bool,
    pub enabled: bool,
    pub start_requested: bool,
    pub tips_requested: bool,
}
impl HomeView {
    fn values(&self) -> (usize, usize, usize, usize, String) {
        (
            self.registered,
            self.answered,
            self.due,
            self.first_today,
            super::ux::duration(self.seconds),
        )
    }
    fn start_label(&self) -> &'static str {
        if self.resuming {
            "▶  学習を再開する"
        } else {
            "▶  学習を始める"
        }
    }
    fn hero(&mut self, ui: &mut Ui) {
        let width = ui.available_width();
        let wide = width >= 900.0;
        Frame::new()
            .fill(Color32::from_rgb(233, 246, 255))
            .stroke(Stroke::new(1.0_f32, BORDER))
            .inner_margin(28)
            .corner_radius(14)
            .show(ui, |ui| {
                ui.set_width((width - 58.0).max(1.0));
                ui.set_min_height(if wide { 230.0 } else { 245.0 });
                let r = Rect::from_min_size(
                    ui.cursor().min,
                    vec2(ui.available_width(), ui.min_size().y),
                );
                if wide {
                    scenery(
                        ui.painter(),
                        Rect::from_min_max(pos2(r.left() + r.width() * 0.49, r.top()), r.max),
                    );
                }
                let text_width = if wide {
                    ui.available_width() * 0.64
                } else {
                    ui.available_width()
                };
                ui.scope(|ui| {
                    ui.set_max_width(text_width);
                    title(ui, "今日の学習", if wide { 42.0 } else { 34.0 });
                    ui.add_space(6.0);
                    ui.label(format!("まずは{}分、英語にふれる時間を。", self.minutes));
                    ui.label("今日の問題から、少しずつ使える表現を増やしましょう。");
                    ui.add_space(24.0);
                    let minutes = self.minutes;
                    let clock = |ui: &mut Ui| {
                        ui.horizontal(|ui| {
                            badge(ui, Icon::Clock, BLUE, 64.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("目安時間").size(15.0).color(MUTED));
                                title(ui, &format!("約 {minutes} 分"), 35.0);
                            });
                        });
                    };
                    if text_width > 600.0 {
                        ui.horizontal(|ui| {
                            clock(ui);
                            ui.add_space(16.0);
                            if ui
                                .add_enabled_ui(self.enabled, |ui| {
                                    primary(ui, self.start_label(), 300.0)
                                })
                                .inner
                                .clicked()
                            {
                                self.start_requested = true;
                            }
                        });
                    } else {
                        clock(ui);
                        ui.add_space(8.0);
                        if ui
                            .add_enabled_ui(self.enabled, |ui| {
                                primary(ui, self.start_label(), 360.0)
                            })
                            .inner
                            .clicked()
                        {
                            self.start_requested = true;
                        }
                    }
                });
                if wide {
                    ui.painter().text(
                        pos2(r.right() - 32.0, r.top() + 24.0),
                        Align2::RIGHT_TOP,
                        format!("今日の{}分を、\n明日の自分へ。", self.minutes),
                        FontId::proportional(20.0),
                        Color32::from_rgb(47, 103, 151),
                    );
                }
            });
    }

    fn metrics(&self, ui: &mut Ui) {
        let (registered, _, due, new, time) = self.values();
        let data = [
            (
                "今回の復習候補",
                format!("{due} 項目"),
                "期限を迎えた項目から",
                Icon::Refresh,
                ROSE,
            ),
            (
                "今日の初回回答",
                format!("{new} 回"),
                "初めて回答した問題",
                Icon::File,
                GREEN,
            ),
            (
                "今日の学習時間",
                time.to_string(),
                "休止・AI待機を除く",
                Icon::Clock,
                BLUE,
            ),
            (
                "登録されている語彙",
                format!("{registered} 語"),
                "重複を除いた基本語・表現",
                Icon::Book,
                PURPLE,
            ),
        ];
        let width = ui.available_width();
        let cols = columns(width);
        let card_width = (width - 16.0 * (cols - 1) as f32) / cols as f32;
        for chunk in data.chunks(cols) {
            ui.horizontal(|ui| {
                for (label, value, note, kind, color) in chunk {
                    card(ui, card_width, 142.0, Color32::WHITE, |ui| {
                        ui.horizontal(|ui| {
                            badge(ui, *kind, *color, 50.0);
                            ui.vertical(|ui| {
                                ui.set_width((card_width - 110.0).max(1.0));
                                ui.label(RichText::new(*label).size(14.0).color(MUTED));
                                title(
                                    ui,
                                    value,
                                    if value.chars().count() > 7 {
                                        27.0
                                    } else {
                                        31.0
                                    },
                                );
                            });
                        });
                        ui.add_space(8.0);
                        ui.label(RichText::new(*note).size(14.0).color(MUTED));
                    });
                }
            });
            ui.add_space(4.0);
        }
    }

    fn progress(&self, ui: &mut Ui, width: f32) {
        let (registered, learned, _, _, _) = self.values();
        card(ui, width, 244.0, Color32::WHITE, |ui| {
            ui.horizontal(|ui| {
                badge(ui, Icon::Book, BLUE, 48.0);
                title(ui, "学習の積み重ね", 23.0);
            });
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("学習経験あり").color(MUTED));
                title(ui, &format!("{learned} / {registered} 語"), 21.0);
            });
            ui.add(
                ProgressBar::new(ratio(learned, registered))
                    .fill(BLUE)
                    .desired_height(12.0),
            );
            ui.add_space(14.0);
            let stats = [
                ("回答した語彙", learned, Icon::Check, GREEN),
                (
                    "まだ回答していない語彙",
                    registered - learned,
                    Icon::Book,
                    MUTED,
                ),
            ];
            let draw = |ui: &mut Ui, index: usize| {
                let (label, value, kind, color) = stats[index];
                ui.horizontal_top(|ui| {
                    badge(ui, kind, color, 42.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(label).size(15.0).color(MUTED));
                        title(ui, &format!("{value} 語"), 26.0);
                    });
                });
            };
            if width >= 560.0 {
                ui.columns(2, |columns| {
                    draw(&mut columns[0], 0);
                    draw(&mut columns[1], 1);
                });
            } else {
                draw(ui, 0);
                ui.add_space(8.0);
                draw(ui, 1);
            }
            ui.add_space(10.0);
            ui.label(
                RichText::new(if registered == 0 {
                    "まだ教材がない状態。「語彙を追加」から登録できる。"
                } else {
                    "1回以上回答した語彙の数であり、習得完了を意味しない。"
                })
                .size(14.0)
                .color(MUTED),
            );
        });
    }

    fn tips(&mut self, ui: &mut Ui, width: f32) {
        card(ui, width, 244.0, Color32::from_rgb(255, 251, 240), |ui| {
            ui.horizontal(|ui| {
                badge(ui, Icon::Bulb, Color32::from_rgb(159, 111, 0), 48.0);
                title(ui, "続けるためのヒント", 22.0);
            });
            ui.add_space(12.0);
            ui.label("間違えても大丈夫。次の復習で、もう一度思い出す練習ができます。");
            ui.add_space(8.0);
            ui.label("今日は5分だけでも十分。休んだ日があっても、記録は失われません。");
            ui.add_space(14.0);
            if ui
                .add(
                    crate::app::controls::Button::new(
                        RichText::new("学習のヒントを見る  ›").color(MUTED),
                    )
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .corner_radius(8),
                )
                .clicked()
            {
                self.tips_requested = true;
            }
        });
    }

    pub(super) fn show(&mut self, ui: &mut Ui) {
        ui.set_width(ui.available_width());
        self.hero(ui);
        ui.add_space(16.0);
        self.metrics(ui);
        ui.add_space(12.0);
        let width = ui.available_width();
        if width >= 950.0 {
            let left = (width - 16.0) * 0.62;
            ui.horizontal_top(|ui| {
                self.progress(ui, left);
                self.tips(ui, width - left - 16.0);
            });
        } else {
            self.progress(ui, width);
            ui.add_space(16.0);
            self.tips(ui, width);
        }
        ui.add_space(16.0);
        Self::encouragement(ui, width);
    }

    fn encouragement(ui: &mut Ui, width: f32) -> Rect {
        Frame::new()
            .fill(Color32::from_rgb(237, 247, 254))
            .corner_radius(10)
            .inner_margin(16)
            .show(ui, |ui| {
                ui.set_width((width - 32.0).max(1.0));
                ui.horizontal_wrapped(|ui| {
                    badge(ui, Icon::Leaf, GREEN, 36.0);
                    centered_label(ui, "少しずつの積み重ねを、確かな力に。", 19.0, 36.0);
                    centered_label(ui, "復習時期はWordWeaveの規則で計算する。", 15.0, 36.0);
                });
            })
            .response
            .rect
    }
}
fn columns(width: f32) -> usize {
    if width >= 1100.0 {
        4
    } else if width >= 620.0 {
        2
    } else {
        1
    }
}
fn ratio(answered: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (answered as f32 / total as f32).clamp(0.0, 1.0)
    }
}
