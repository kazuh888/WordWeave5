# QWEN-SETTINGS-001 独立検証実施

2026-10-03。役割はtest runner、計画者指定 `gpt-6.1-sol / medium`。取得した実行model/effortと開始/終了token countersは未取得であり、推定しない。AGENTS.md、development-team.md、wordweave-change全文、計画/仕様/設計/独立テストhandoffを確認した。所有は本報告、担当ログとビルド生成物のみである。

最終状態（15:26:57 JST）：本体全体439件、設定焦点20件、package default135件/core88件が成功し、両release生成成功。終了時の最終Rust/Cargo入力比較は差分0である。全体root fmtは既存広域差分で失敗したままであり、全検証無条件GREENとは報告しない。native実接続/機器/release操作は別の受入残である。

## 対象と受入境界

HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`、多数の既存未コミット差分を含む。最初の入力は `runner-input-manifest.txt` の124ファイルSHA256、tracked差分は `runner-tracked-diff.patch`（SHA256 `5CECEE8D0359D2AA1EE207B86FFAFEE950A990149D16FC5650DB47A03B3E4DF1`）、git状態は `runner-git-status.log` に保存した。HEADだけで変更済みソースを識別した扱いにしない。

新結果の厳格validator、6地域とhost/キー非転用、設定保存/取消し/遅延/独立保存、本体15件とGUI consentを含む全体回帰、両releaseの生成を検証する。native画面は親担当、実キー/Windows資格情報操作/課金API/利用者音声による精度とrelease-nativeは別の受入残である。mock成功から認証、microphone/pen/TTS/IME成功を推定しない。

## 試行と対象変化

- 通常shellは `apply deny-read ACLs` のprocess生成失敗で開始できず、読み取りと担当ビルド/ログ保存のみ `require_escalated`, `login:false` で実行した。承認レビュー拒否はない。
- 親のREADY後 `cargo test --all-targets --locked -j1` を開始、`runner-root-tests.log` に保存中。依存コンパイル中に親から独立review対応でroot editor/app/core credentialsと追加テストが変わる通知を受けた。親指示に従い試行ログを保存し、release/packageは再READY待機する。この試行を最終差分の固定合格証拠と扱わない。
- 既存focusログを新実施として再利用しない。最新版15件とdefault GUI consentを全体試験で確認する。core/no-defaultのGUI/MCP gateによる0件は成功したケースとして数えない。

初回root全体試行はexit 101。テスト実行前に `target/debug/wordweave5.exe` の削除が `Access denied (os error 5)` で失敗した。親のdebug合成画面確認プロセスが同EXEを使用していたことは読み取りで検出した。実行試験は0、成功/失敗ケースも0、ビルド環境競合1として親へ報告した。ユーザーapp停止は行わない。再READYは親の合成fixture正常終了とsource安定後に受け取る。

検証コマンド、終了コード、成功/失敗/skip集計と最終入力一致は完了時追記する。passed再実行は入力変化か具体的な未解決リスクがある場合に限る。失敗時は親へ報告し、source/期待値/gateを変更しない。

## 新規実行の中間結果

- 再READYで `runner-final-input-manifest.txt` と `runner-final-tracked-diff.patch` を採取しroot全体試行2を実行した。途中でapp.rsのOS終了分岐に独立review修正が入り、このsnapshotも最終root対象ではない。結果はexit101、lib134成功、bin232成功/1失敗（各ignored0）。失敗は `reading_settings_link_is_only_visible_when_unconfigured` のclip外button検査だった。親/独立著者がfixtureを切り分け、期待文字assertを保持して実wheel到達とconfigured時の未生成検査を追加した。`runner-final-root-tests.log` は失敗試行のまま保存し、root最終試験を別ログで行う。
- `cargo fmt --all -- --check` exit1、40ファイルの広域整形差分。`runner-root-fmt.log`。今回touch範囲にもapp.rs/settings_ui.rs/visual_check.rs/qwen_reading.rs/qwen_reading_ui.rsの差分があるため、全体を無条件の既存baseline合格とは扱わない。source整形はrunnerが行わない。
- `rustfmt --edition 2021 --check --config skip_children=true src/app/qwen_settings.rs` exit0。`runner-editor-fmt.log`。
- package実装6ファイル、新契約2ファイル、integration_credentials/consent/root qwen_settings_testsの単独 `rustfmt --edition 2021 --check --config skip_children=true ...` exit0。`runner-owned-fmt.log`。root fixture追補前のチェックであるため、最終root testファイルの整形は親が追加確認する。
- `cargo test --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --all-targets --locked -j1` exit0、135成功/0失敗/0ignored。`runner-package-tests.log`。default GUIのconsentとWindows資格情報試験が実行されている。
- `cargo test --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --no-default-features --lib --tests --locked -j1` exit0、88成功/0失敗/0ignored。`runner-package-core-tests.log`。consent/gui_geometry/mcp_stdio/sessionの4targetはfeature gateで0件であり成功caseには計上しない。
- `cargo check --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --no-default-features --features windows-credentials --locked -j1` exit0。`runner-package-windows-check.log`。実OS資格情報の読取/保存ではない。
- package release buildはEXE使用プロセス未検出を確認して開始した。packageのsourceは親が固定済みと通知し、root fixture修正とは独立して進めている。
- `cargo build --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --release --locked --bin qwen-audio -j1` exit0、4m30s。`runner-package-release.log`。
- `cargo build --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --locked --bin qwen-audio -j1` exit0、2m35s。`runner-package-debug.log`。親へdebug完成を通知し、親の合成native確認を別担当へ残した。
- `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check` exit0。`runner-package-fmt.log`。

## 最終root焦点の失敗引継ぎ

親の再READY後、Rust/Cargo入力だけを `runner-accepted-input-manifest.txt`（114ファイル＋末尾改行）と `runner-accepted-tracked-diff.patch` に固定した。EXE未使用を確認し `cargo test --locked --bin wordweave5 app::qwen_settings_tests -j1` を実行、exit101、16成功/4失敗/0ignored/213filtered。`runner-root-settings-focus.log`。

4失敗は `confirmed_discard_returns_keyboard_focus_and_preserves_saved_summary`、`dedicated_save_returns_keyboard_focus_to_edit_button`、`os_close_of_unchanged_editor_holds_exit_and_returns_keyboard_focus`、`unchanged_cancel_returns_keyboard_focus_without_saving`。共通の `qwen_settings_tests.rs:402` の可視assertで、編集button `[[25.0 1363.1]-[231.2 1407.9]]` がclip `[[0.0 276.0]-[1120.0 736.0]]` 外であった。focus値だけの旧検査と、画面内に実在することの新検査を混同しない。

親へ4失敗を報告し、root full/release/debugを停止した。runnerはsource/test/期待値を変更していない。package検証はそのsourceへの影響がない限り再実行しない。rootの最終全体合格・release生成は未完了であり、親の修正と再固定handoff後に再開する。

SHA256: accepted tracked diff `91185BF12A4434938F2761B7C3E8B55739434E8DDA7C543111E55F14C7F655E8`、accepted input manifest `F06158A6DFE8F39C1E5B80C98013CE9FB7CFB3D7570CE29AB9113335728063D7`。package Rust/Cargo入力は前後比較の差分0を確認した。新package release EXE `968A24822EB3CB166F6E50F04236A2455EBCB2B3C19AB02339C50FDA38AAEF09`、debug EXE `4CB3770FBBB0D4D2F1515370EA16F8B6FDD740D9A4DBB6D1707F25B4A6AD5158`。EXEは生成物の識別でありnative受入成功を意味しない。

親がsettings_ui.rs/root設定テストへ観測だけを追加後、診断1件を実行した：`cargo test --locked --bin wordweave5 app::qwen_settings_tests::unchanged_cancel_returns_keyboard_focus_without_saving -j1 -- --nocapture`。exit101、0成功/1失敗/0ignored/232filtered、`runner-root-focus-diagnostic.log`。入力差分は `runner-diagnostic-tracked-diff.patch`、settings_ui.rs SHA256 `E45C79CE8B1DDDF8432FD2E3FEDAD7DEB7565E42DD448855FA309DBC6DFB9C87`、qwen_settings_tests.rs SHA256 `2C7D5A796D4668A85A3E4A2734B2350510AE8814BBBEE6BEFE518B873AB63AEF`。

frame0: pending=false、button `[25,1343.4]-[231.2,1388.2]`、clip `[0,282.6]-[12500,12421]`、scroll ID CC7F/offset `[0,0]`。frame1/2: pending=false、button `[25,1363.1]-[231.2,1407.9]`、clip `[0,276]-[1120,736]`、同scroll ID/offset `[0,0]`。仮想大画面から通常viewportへ変わる前に復帰フラグが消費された可能性を親へ返した。これは診断の推論でありproduction修正/最終合格ではない。

親がsizing pass/disabled除外、clipとscreen全体可視までpending保持、Qwen復帰時のみscroll animation停止を修正し再READY。Rust/Cargo入力を `runner-sizing-fixed-input-manifest.txt`、tracked差分を `runner-sizing-fixed-tracked-diff.patch` へ再固定し `cargo test --locked --bin wordweave5 app::qwen_settings_tests -j1` を再実行した。exit101、16成功/4失敗/0ignored/213filtered、`runner-root-settings-fixed-focus.log`。4件とbutton/clip座標は前回と同一であり、修正後の可視性合格は得られていない。親へ返し全体/release/debug停止を継続した。

## 最終GREENとrelease生成

その後、親と独立著者が「初回描画前にmodalを終了しContext screenが12500のまま」のfixture条件を切り分けた。期待値を維持して実寸screen/clipをassertし、modal描画安定後の終了操作へ修正した。親は新debugの合成native標準/狭幅で編集button到達を別途確認した。runnerはroot process未検出を確認してRust/Cargo入力114ファイルを `runner-stable-input-manifest.txt`、tracked差分を `runner-stable-tracked-diff.patch` に固定した。

- 親指定診断native用 `cargo build --locked --bin wordweave5 -j1` exit0、3m26s。`runner-root-diagnostic-debug.log`。途中fixture変更だけでproduction不変と親通知。親へ完成通知し、native確認終了まで本体Cargo停止を維持した。production不変なら同debugを繰り返し生成しない。
- `rustfmt --edition 2021 --check --config skip_children=true src/app/qwen_settings.rs src/app/qwen_settings_tests.rs` exit0。`runner-final-editor-fmt.log`。
- `cargo test --locked --bin wordweave5 app::qwen_settings_tests -j1` exit0、20成功/0失敗/0ignored/213filtered。`runner-root-settings-stable-focus.log`。可視性4件のassertを含む。
- `cargo test --all-targets --locked -j1` exit0、439成功/0失敗/0ignored。`runner-root-stable-tests.log`。0件target2は成功caseへ計上していない。
- `cargo build --release --locked --bin wordweave5 -j1` exit0、5m32s、`runner-root-release.log`。EXE使用プロセス未検出を確認して開始した。ユーザーapp停止は一度も実施していない。

試行2/診断/途中focus失敗はすべて旧対象ごとのログを保持した。最終GREENで上書きしていない。root全体試験を再実行した理由はsource/fixtureが変わったためであり、package入力は変わらず成功検証を再実行しない。

## 最終対象・生成物・受入残

HEADは冒頭の `d86e7e9d4fe4497d8be9040d30a078582f2887d8` である。最終コンパイル入力114ファイルのmanifest SHA256は `D32F7159B80B2940B2F72830186FF96B1E583642DD9E6716A8C19C5A3D48DB26`。tracked差分patch SHA256は `FB10C11BCD069E74C5636E4795068518D410E63058E79F8583F2BF38EEC96062`。未追跡Rust/Cargoもmanifestへ含め、HEAD＋tracked patchだけで対象を識別した扱いにしない。最終focus/full/release前後でmanifest各ファイルSHA256を再比較し差分0を確認した。報告文書や親のnative画像はコンパイル入力と分けた。

生成物SHA256：

- `target/release/wordweave5.exe`: `F9F708A48E8EFC3ECDC4CD50C19DB211158BE1C3BBEA5D6A319FB7F1EB78FF24`
- `target/debug/wordweave5.exe`: `37CD48D98A1969CE9BE1EE7E307B1B9D650177CF40D80622A5DBC1F98002C411`
- `target/qwen-audio/release/qwen-audio.exe`: `968A24822EB3CB166F6E50F04236A2455EBCB2B3C19AB02339C50FDA38AAEF09`
- `target/qwen-audio/debug/qwen-audio.exe`: `4CB3770FBBB0D4D2F1515370EA16F8B6FDD740D9A4DBB6D1707F25B4A6AD5158`

最終試験のfail/ignoredはいずれも0。focusの213filtered、no-defaultの4 gated 0件target、root全体の2件の0試験targetは実行合格caseへ計上していない。defaultとcore-onlyの重複caseは単純合算して独立したcoverage件数にしない。root全体439と設定focus20も重複を含む。

親が別途行ったdebug合成native画像と操作は親のresultsへ記録する。runner自身はmicrophone/pen/TTS/IME、実Codex認証、実Windows資格情報操作、実Qwenキー/API/利用者音声/他地域利用権、release-native操作を実施していない。mockや新buildをその成功証拠へ読み替えない。実API送信とOS資格情報操作はrunnerの試験では0である。

本検証担当の完了境界は最終自動検証、両release生成、固定差分/ログ/未検証引継ぎまでである。起動model/effortと実行メタデータ、開始/終了usage counterは引き続き未取得であり推計しない。ソース/テスト/期待値/settings変更、モデル/gate変更、再委任、commit/push/公開は実施していない。
