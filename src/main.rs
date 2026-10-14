#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(all(test, windows))]
mod early_trial_startup_tests {
    #[test]
    fn et03_title_matches_independent_feature_literal() {
        #[cfg(feature = "early-trial")]
        let expected = "WordWeave 5 — 早期試用・専用データ";
        #[cfg(not(feature = "early-trial"))]
        let expected = "WordWeave 5 — 使い分ける英語";
        assert_eq!(super::WINDOW_TITLE, expected);
    }

    #[cfg(feature = "early-trial")]
    #[test]
    fn et_d01_creator_preserves_open_rejection_without_creating_closure() {
        crate::media::early_trial_startup_tests::isolated("early_trial_startup_tests::et_d01_creator_preserves_open_rejection_without_creating_closure", |base| {
            use wordweave5::store::Storage;
            use crate::app::{early_trial_startup_tests::{ledger, seed_roots}, TrialStartupFailure};
            let trial = seed_roots(base);
            let history = r#"{"version":2,"audio_history":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":12}}"#;
            std::fs::write(trial.join("progress.json"), history).unwrap();
            let before = ledger(base);
            let result = super::early_trial_creator(Storage::open());
            let failure = match result { Ok(_) => panic!("creator closure must not exist after open refusal"), Err(e) => e };
            let expected = "音声履歴を含むため、早期試用では開けません。\n起動前の検査で停止しました。元データは変更していません。";
            assert!(matches!(&failure, TrialStartupFailure::StorageOpenRejected { safe_message } if safe_message == expected));
            assert_eq!(failure.to_string(), expected);
            assert_eq!(ledger(base), before);
            assert!(!trial.join("diagnostics").exists());

            let after_open = "試用保存先の初期化を完了できませんでした。試用内に作成済みのフォルダーや確認用ファイルが残っている場合があります。";
            let failure = super::early_trial_creator(Err(after_open.into())).err().expect("Err must not create a closure");
            assert_eq!(failure.to_string(), after_open, "safe Storage String must not be reclassified as BeforeWrites");
        });
    }
}

#[cfg(windows)]
mod ai;
#[cfg(windows)]
mod annotation;
#[cfg(windows)]
mod app;
#[cfg(windows)]
mod ink;
#[cfg(windows)]
mod media;

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
