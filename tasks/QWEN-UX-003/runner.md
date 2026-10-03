# QWEN-UX-003 検証runner

担当 `/root/qwen_steps_runner`。planner割当 `gpt-6.1-sol/medium`。起動指定と実行値は区別し、実行model/effort metadata、input/cache/output/reasoning開始・終了counterと親子包含は未取得である。

AGENTS.md、docs/process/development-team.md、KPI、wordweave-change全文とspec/design/plan/tests.mdを読了した。検証・結果報告のみを適用する。所有は本書、logs、build生成物のみであり、ソース・期待値・設定は変更しない。

## 対象と境界

親の凍結合図後のHEAD `d86e7e9d4fe4497d8be9040d30a078582f2887d8` と未コミット差分が対象である。実施開始UTC `2026-10-03T10:41:53.9481335Z`。src/tests/Cargo全入力（visual_check.rsを含む）のmanifestは `logs/build-inputs-before.sha256`、manifest SHA256は `003FAD7D60CE1DCDE569E2E0AACBFB0D8CC0AFCDA46A6A6F98E258FF3E2D26E2`。共有path dependencyは `logs/qwen-package-before.sha256` に別記した。dirty全体識別は補助であり、生成物や文書の変化をbuild入力変更と混同しない。

通常shellはdeny-read ACL helperで起動失敗した。以降の必須読取・担当log保存・Cargoはscopeを明示したrequire_escalatedで実施している。これによるソース編集や品質gate変更はない。

## 実行結果（初回、不合格停止）

| コマンド | exit | 判定・件数 | 新規log |
| --- | --- | --- | --- |
| `cargo fmt --all -- --check` | 1 | FAIL、既存39filesの整形差分。修正しない | `logs/fmt-all.log`、`logs/fmt-exit.log` |
| `rustfmt --edition 2021 --check src/app/qwen_reading_ui.rs src/app/qwen_reading_tests.rs` | 0 | PASS、対象2files | `logs/fmt-target.log`、`logs/fmt-exit.log` |
| `cargo test --all-targets --locked -j1` | 101 | FAIL。lib 134成功/0失敗、本体bin 250成功/9失敗、mock bin 0件。実施合計384成功/9失敗/0ignored。本体reading 26成功/8失敗。後続integration targetはCargo停止で未実行（件数未集計） | `logs/cargo-test-all.log`、`logs/cargo-test-all-exit.log` |

test開始UTC `2026-10-03T10:44:12.7445653Z`、終了UTC `2026-10-03T10:52:37.7186335Z`。検証報告確認UTC `2026-10-03T10:53:39.9417523Z`。compilationは成功し、9件はruntime assertion/panicである。fixture timeout・compile errorではない。新規P2回帰も実行したが、そのうち2件は送信buttonの到達assertionで止まったためfold reset自体の合否証拠にはならない。

| 失敗 | 位置・実測 |
| --- | --- |
| `qwen_auto_confirmation_requires_visible_send_click_and_sends_once`、`qwen_completed_summaries_expand_without_resend_or_discarding_sent_content`、`qwen_failed_evaluation_folds_previous_steps_and_never_automatically_retries`、`qwen_restart_discards_previous_result_and_requires_new_confirmation_id`、`qwen_open_summaries_reset_on_completion_and_restart_resend_in_same_context`、`qwen_running_open_summaries_reset_on_failure_in_same_context` | 共通helper `src/app/qwen_reading_tests.rs:847`。send Rect `[[133.0 1080.0]-[387.7 1120.0]]` がbody `[[120.0 489.0]-[880.0 1007.0]]` の完全内側にない |
| `qwen_details_start_closed_and_expand_without_changing_confirmed_input` | `src/app/qwen_reading_tests.rs:535`。初期必須文字 `③ 評価結果を確認する` が取得できない |
| `qwen_three_stage_guidance_places_input_and_connection_failures_before_results` | `src/app/qwen_reading_tests.rs:1474`。`rfind("③ 評価結果を確認する")` がNone |
| `reading_settings_link_is_only_visible_when_unconfigured` | `src/app/qwen_settings_tests.rs:335`。旧案内 `音声と結果は破棄する` のcontains assertion失敗。実表示は丁寧語化された破棄説明である |

検証後 `logs/build-inputs-after-test.sha256` はbeforeと同じSHA256 `003FAD7D60CE1DCDE569E2E0AACBFB0D8CC0AFCDA46A6A6F98E258FF3E2D26E2` であり、行差分0。共有package before/afterの行差分も0。dirty全体補助manifestの初回digestは `5F1F1C682332BC9E951FA0D7841D25E9AB7CA54927BF6C87210B2744E2DE4CCC`（`logs/dirty-before.sha256`）。生成物・旧logの全列挙は反復していない。

toolchainはrustc 1.98.1 / cargo 1.98.1である。全体testにreading焦点を含むため、成功後に同じ焦点を再compileしない。失敗をcompile/fixture/assertionで区別して親へ返し、runnerは修正しない。

## 未検証

全体test不合格によりdebugfixture生成、親の合成native描画確認、プロセス不在確認後のreleaseビルドと生成EXE時刻/hashはSKIP（未開始）である。失敗を親へ報告し、ソース/期待値の修正をrunnerは行わない。実マイク・実API・実音声評価精度・IME・nativekeyboard・pen/TTS・Windows DPI・利用者受入はmock成功では立証しない。既存ログを今回のnative結果として扱わない。

## 修正後v2

親の2回目凍結合図後、入力manifest `logs/build-inputs-v2-before.sha256` のSHA256は `2C05E28FA0782A29686C50D49EC28F43253712884C0D7F8AC6C52F6E8BDB34ED`。初回からの変更は `qwen_reading_ui.rs`、`qwen_reading_tests.rs`、`qwen_settings_tests.rs` の3ファイルのみである。親は本体Area最大heightの修正を、独立test authorは実スクロールhelper/clip確認と旧文言期待の承認済み丁寧語更新を担当した。runnerは変更していない。

| コマンド | exit | 判定・件数 | log |
| --- | --- | --- | --- |
| `rustfmt --edition 2021 --check src/app/qwen_reading_ui.rs src/app/qwen_reading_tests.rs src/app/qwen_settings_tests.rs` | 0 | PASS、変更3files。全体fmtの既存39files以外の入力は不変であり全体dumpは反復しない | `logs/fmt-target-v2.log`、`logs/fmt-exit-v2.log` |
| `cargo test --all-targets --locked -j1` | 0 | PASS、465成功/0失敗/0ignored/0filtered（lib134、bin259、integration72、0件target2）。reading独立34件全成功 | `logs/cargo-test-all-v2.log`、`logs/cargo-test-all-v2-exit.log` |
| `cargo build --locked --bin wordweave5 -j1` | 0 | PASS、debug/nativefixture生成。4 warnings | `logs/cargo-build-debug-v2.log`、`logs/cargo-build-debug-v2-exit.log`、`logs/debug-exe-v2.log` |

v2 test開始UTC `2026-10-03T10:59:37.6326683Z`、終了UTC `2026-10-03T11:06:01.7487509Z`。本体layout/helper/期待文言入力が変わったため全体testを再実施した。成功した全体test/焦点readingは今後の関連入力変更がなければ再実行しない。初回の不合格は上記に保持する。debug/native/releaseの現状は本節を優先する。

debug build開始UTC `2026-10-03T11:06:21.8398130Z`、終了UTC `2026-10-03T11:10:32.6291450Z`。EXEは `target/debug/wordweave5.exe`、23,727,104 bytes、生成UTC `2026-10-03T11:10:30.1209963Z`、SHA256 `0E02775C5E29609D6FA74367FF39FC0152B9F8A4457D527ED59A0110DC33AF5F`。`logs/build-inputs-v2-after-debug.sha256` はv2 beforeと行差分0である。Cargoを停止して親へnativefixtureを引き渡した。親のnative合格合図とEXE process不在確認後にreleaseのみ実行する。

## 親の合成native引継ぎ・release

親は `native/` の14 PNGを実見して合格と報告した。runnerは画像を再レビューせず親の実見結果として区別する。runnerが `native/exits.log` を読んだ結果、14 owned PID全てExited=true、ExitCode=0、ImageExists=trueである。

標準input/confirm/running/result/mismatch/unassessable/jsonfailed/unconfigured、最小820×650の80%input・100/125%confirm末尾・160%result冒頭/末尾・jsonfailed末尾を対象とする。番号段階、折畳み、結果、取消し表示、最小の縦スクロールで送信/再練習/固定Close到達と横切れなしを親が確認した。160%の結果全文が初期一画面に収まるという証拠ではなく、冒頭と末尾は別画像である。操作はheadless pointer/keyboard回帰による証拠でありnativekeyboard未検証である。

release開始前にGet-Process -Name wordweave5でprocess count 0を確認した（`logs/process-before-release.log`）。appを終了していない。`logs/build-inputs-v2-before-release.sha256` はv2 beforeと差分0、共有package `logs/qwen-package-before-release.sha256` は初回beforeと差分0、debug EXE hashは生成時と同一である。

`cargo build --release --locked --bin wordweave5 -j1` はexit 0、PASS（4 warnings）である。担当logは `logs/cargo-build-release-v2.log`、終了時刻/exitは `logs/cargo-build-release-v2-exit.log`。開始UTC `2026-10-03T11:15:25.0349412Z`、終了UTC `2026-10-03T11:21:29.7893740Z`。

release EXEは `target/release/wordweave5.exe`、12,783,616 bytes、生成UTC `2026-10-03T11:21:28.3083271Z`、SHA256 `F6C1960698C86EA809B7AC7D0CE82DDCE236717660AC09513D4EB4E420DCB532`（`logs/release-exe-v2.log`）。最終 `logs/build-inputs-v2-after-release.sha256` はv2 beforeと差分0、`logs/qwen-package-after-release.sha256` は初回beforeと差分0、debug EXE hashも同一である。HEADは初回と同一 `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。検証したdirty入力の正本digestは `2C05E28FA0782A29686C50D49EC28F43253712884C0D7F8AC6C52F6E8BDB34ED`。

## 最終引継ぎ

runner完了境界は凍結差分の新規全体test/debug/release、親の新規合成native結果の明示引継ぎ、入力不変確認、担当報告である。終了観測UTC `2026-10-03T11:22:21.8185985Z`。usage終了counterと実行model/effort metadataは未取得のままであり総消費を推定しない。

v2 test 465成功/0失敗/0ignored、debug/release各exit0、親native14cases合格/exit0である。全体fmtは初回の既存39files差分によりexit1であり、緑と報告しない。修正後3filesのrustfmtcheckはexit0。残るnativekeyboard、IME、実マイク/再生、pen/TTS、実API認証/音読評価精度、Windows DPI、利用者画面受入は未検証である。runnerのソース・test・期待値・設定編集、app強制終了、実API送信、commit/公開は0件である。
