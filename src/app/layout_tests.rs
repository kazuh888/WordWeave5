use super::harness_tests::fixture;
use super::*;

#[test]
fn pages_fit_logical_width_at_minimum_window_and_maximum_zoom() {
    let (ctx, mut app, root) = fixture();
    // egui uses points after zoom; this is 820 x 650 physical pixels at 160%.
    let size = egui::vec2(820.0 / 1.6, 650.0 / 1.6);
    for page in [
        Page::Home,
        Page::Study,
        Page::Deck,
        Page::Words,
        Page::Stats,
        Page::Settings,
    ] {
        for _ in 0..2 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let right = ui.max_rect().right();
                        if page == Page::Settings {
                            app.settings(ui, ctx);
                            assert!(ui.min_rect().right() <= right + 1.0);
                            return;
                        }
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            match page {
                                Page::Home => app.home(ui),
                                Page::Study => app.study(ui),
                                Page::Deck => app.deck_page(ui),
                                Page::Words => app.words_page(ui),
                                Page::Stats => app.stats(ui),
                                Page::Settings => app.settings(ui, ctx),
                                Page::Chat => unreachable!(),
                            }
                            assert!(
                                ui.min_rect().right() <= right + 1.0,
                                "page {} overflow: {:?}, right={right}",
                                page as u8,
                                ui.min_rect()
                            );
                        });
                    });
                },
            );
        }
    }
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn long_status_is_bounded_and_keeps_a_central_work_area() {
    let (ctx, mut app, root) = fixture();
    app.message = "詳しいエラー情報。".repeat(200);
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(512.5, 406.25),
            )),
            ..Default::default()
        },
        |ctx| {
            app.update_ui(ctx);
            assert!(
                ctx.available_rect().height() >= 140.0,
                "{:?}",
                ctx.available_rect()
            );
        },
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
