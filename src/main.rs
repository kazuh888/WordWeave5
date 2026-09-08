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
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("WordWeave 5 — 使い分ける英語")
            .with_inner_size([1120.0, 850.0])
            .with_min_inner_size([820.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WordWeave 5",
        options,
        Box::new(|cc| Ok(Box::new(app::WordApp::new(cc)))),
    )
}

#[cfg(not(windows))]
fn main() {
    eprintln!("WordWeave 5 is a Windows desktop application. Core tests: cargo test --lib");
}
