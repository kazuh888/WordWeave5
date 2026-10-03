#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(windows)]
mod ai;
#[cfg(windows)]
mod app;
#[cfg(windows)]
mod ink;
#[cfg(windows)]
mod media;
#[cfg(windows)]
mod annotation;

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    #[cfg(debug_assertions)]
    if std::env::args().any(|arg| arg == "--ui-check") {
        let result = app::visual_check::run();
        finish_audio_jobs();
        return result;
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("WordWeave 5 — 使い分ける英語")
            .with_inner_size([1150.0, 950.0])
            .with_min_inner_size([820.0, 650.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let result = eframe::run_native(
        "WordWeave 5",
        options,
        Box::new(|cc| Ok(Box::new(app::WordApp::new(cc)))),
    );
    finish_audio_jobs();
    result
}

#[cfg(windows)]
fn finish_audio_jobs() {
    qwen_audio::shutdown_audio_jobs();
    if qwen_audio::take_audio_cleanup_warning() {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title("一時音声の削除を確認")
            .set_description(
                "検査用の一時音声コピーを削除できなかった。元の音声ファイルは変更していない。",
            )
            .show();
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("WordWeave 5 is a Windows desktop application. Core tests: cargo test --lib");
}
