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
            let malformed = r#"{"version":2,"audio_history":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":12}}"#;
            let history = r#"{"version":2,"audio_history":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","bytes":12},"settings":{},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
            // Progress's Deserialize also validates through ProgressWire and normalization.
            // The real Storage::open path must distinguish malformed input from
            // a structurally valid v2 record that reaches the history check.
            for (case, input, expected) in [
                ("missing required wire fields", malformed, "試用の学習記録または保存途中の記録を確認できません。音声履歴を含む記録にも対応していません。\n起動前の検査で停止しました。元データは変更していません。"),
                ("valid history-bearing v2", history, "音声履歴を含むため、早期試用では開けません。\n起動前の検査で停止しました。元データは変更していません。"),
            ] {
                // Each explicit fixture replacement precedes its preservation snapshot.
                std::fs::write(trial.join("progress.json"), input).unwrap();
                let before = ledger(base);
                let result = super::early_trial_creator(Storage::open());
                let failure = match result { Ok(_) => panic!("{case}: creator closure must not exist after open refusal"), Err(e) => e };
                assert!(matches!(&failure, TrialStartupFailure::StorageOpenRejected { safe_message } if safe_message == expected), "{case}: expected safe text={expected:?}; actual failure={failure}");
                assert_eq!(failure.to_string(), expected, "{case}: strict-wire refusal must reach the intended reason and stage");
                assert_eq!(ledger(base), before, "{case}: refusal must preserve all source bytes and directories");
                assert!(!trial.join("diagnostics").exists(), "{case}: refusal must not start diagnostics");
            }

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

#[cfg(all(windows, feature = "early-trial"))]
const WINDOW_TITLE: &str = "WordWeave 5 — 早期試用・専用データ";
#[cfg(all(windows, not(feature = "early-trial")))]
const WINDOW_TITLE: &str = "WordWeave 5 — 使い分ける英語";

#[cfg(all(windows, feature = "early-trial"))]
fn early_trial_creator(
    storage: Result<wordweave5::store::Storage, String>,
) -> Result<eframe::AppCreator<'static>, app::TrialStartupFailure> {
    let storage = storage
        .map_err(|safe_message| app::TrialStartupFailure::StorageOpenRejected { safe_message })?;
    Ok(Box::new(move |cc| {
        match app::WordApp::new_early_trial(cc, storage) {
            Ok(app) => Ok(Box::new(app)),
            Err(failure) => {
                show_early_trial_failure(&failure);
                Err(Box::new(failure))
            }
        }
    }))
}

#[cfg(all(windows, feature = "early-trial"))]
fn show_early_trial_failure(failure: &app::TrialStartupFailure) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("早期試用を開始できない")
        .set_description(format!(
            "{failure}\n\n通常版の保存先へは切り替えません。表示された原因を確認し、データを削除せずに再起動してください。"
        ))
        .show();
}

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    #[cfg(debug_assertions)]
    if std::env::args().any(|arg| arg == "--ui-check") {
        let result = app::visual_check::run();
        finish_audio_jobs();
        return result;
    }
    #[cfg(feature = "early-trial")]
    let creator = match early_trial_creator(wordweave5::store::Storage::open()) {
        Ok(creator) => creator,
        Err(failure) => {
            show_early_trial_failure(&failure);
            return Err(eframe::Error::AppCreation(Box::new(failure)));
        }
    };
    #[cfg(not(feature = "early-trial"))]
    let creator: eframe::AppCreator<'static> = Box::new(|cc| Ok(Box::new(app::WordApp::new(cc))));
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([1150.0, 950.0])
            .with_min_inner_size([820.0, 650.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let result = eframe::run_native("WordWeave 5", options, creator);
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
