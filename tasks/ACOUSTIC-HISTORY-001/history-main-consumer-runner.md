# I2b-A 主要consumer 独立実行結果

2026-10-11 / ACOUSTIC-HISTORY-001。担当 `main_consumer_runner`、計画者指定 `gpt-6.1-sol / medium`。割当根拠は [implementation.md](implementation.md) I2b-A・7役表である。実行model/effort metadata、入力・キャッシュ・出力・推論の開始終了counter、親子包含関係は未取得であり推定しない。開始絶対時刻は未採取、最初の時刻採取は19:55:00 JST、実行後確認は19:55:50 JSTである。

AGENTS.md、docs/process/development-team.md、KPI、wordweave-changeを全文読了し、Skillのリスク別検証・報告工程だけを適用した。[作者の独立期待値とRED](history-main-consumer-tests.md)、[実装selfcheck](history-main-consumer-implementation.md)、[runner前独立確認](history-main-consumer-review.md)を全文読了した。要求・受入はAH-AC04/I2-T02、旧復元保全はAH-AC16である。所有は本書、新規runnerログ、専用合成fixture領域だけである。

## 固定対象

今回の対象HEADは `6839e0adb23f3515b4d8a238911e6eec3b2cec67`。開始git statusはcleanである。元基点 `21c304afd86ae5c4a2af96be881e772dbdca86c8` からのapp.rs差分は456追加/17削除（作者433行＋本体23追加/17削除）であり、今回HEADからapp.rsの未コミット差分はない（`git diff --exit-code HEAD -- src/app.rs` exit0）。終端の親所有measurements/current変更は保持した。

| 対象 | 実行前後の一致SHA-256 |
| --- | --- |
| `src/app.rs` | `C1706E880A27C44BDD205D1BA3FD0E01E8D7D43FF0D3F8543F3D62AE251EDA0A` |
| `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `C6143EA4833832FE66B0F0F0B938A7EBE69EA8D2C6E2ADAF474AA1772CD932BA` |

両hashが指定値と一致したため再buildは行わず、同一EXEを今回新規に実行した。旧成功ログは今回の実行証拠へ流用していない。

## コマンドと新規結果

実行共通条件はsandbox内、process限定TEMP/TMPを `F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\i2b-a-runner-temp` へ設定、`Start-Process -WindowStyle Hidden -PassThru -RedirectStandardOutput ... -RedirectStandardError ...` で起動、各processを `WaitForExit(50000)` で待ち、完了後に `WaitForExit()` でflushしExitCodeを取得する方式である。恒久設定やOS権限は変更していない。

| EXE引数 | exit | passed / failed / ignored / filtered | 実測test秒 / process wall ms | PID |
| --- | --- | --- | --- | --- |
| `--list` | 0 | 280 tests / 0 benchmarks、新16件＋指定既存4件の実在確認 | — | — |
| `i2_history_main_consumer --nocapture` | 0 | 16 / 0 / 0 / 264 | 2.36 / 2781 | 5564 |
| `--exact app::harness_tests::json_restore_preserves_the_latest_unsaved_chat_draft --nocapture` | 0 | 1 / 0 / 0 / 279 | 0.29 / 672 | 14360 |
| `--exact app::harness_tests::restoring_trash_closes_editors_and_selects_an_active_conversation --nocapture` | 0 | 1 / 0 / 0 / 279 | 0.32 / 495 | 33828 |
| `--exact app::gfm_tests::gfm_registration_partial_save_never_reports_success_or_erases_recovery_state --nocapture` | 0 | 1 / 0 / 0 / 279 | 0.30 / 472 | 33088 |
| `--exact app::ux_flow_tests::failed_finish_keeps_session_for_export_instead_of_claiming_success --nocapture` | 0 | 1 / 0 / 0 / 279 | 0.19 / 370 | 34476 |

**合計20実行 / 20PASS / 0FAIL / 0ignored / 0timeout / 0未実行**。filteredは各jobの対象外であり、skipや未実行の指定ケースとして合算しない。新16件は以下の実名すべてが今回PASSである。prefixは `app::i2_history_main_consumer::`。

- `t02_persist_stale_none_adopts_a_and_preserves_drafts`
- `t02_persist_stale_b_adopts_a_and_preserves_drafts`
- `t02_import_stale_none_adopts_a_and_preserves_drafts`
- `t02_import_stale_b_adopts_a_and_preserves_drafts`
- `t02_restore_stale_none_adopts_a_and_snapshots_unsaved_drafts`
- `t02_restore_stale_b_adopts_a_and_snapshots_unsaved_drafts`
- `t02_persist_corrupt_root_preserves_state`
- `t02_import_corrupt_root_preserves_state`
- `t02_restore_corrupt_root_preserves_state`
- `t02_persist_unknown_root_preserves_state`
- `t02_import_unknown_root_preserves_state`
- `t02_restore_unknown_root_preserves_state`
- `t02_persist_pending_refuses_without_recovery`
- `t02_import_pending_refuses_without_recovery`
- `t02_restore_pending_refuses_without_recovery`
- `t02_import_pending_creation_failure_and_later_persist_never_rollback`

新suite stderrのmarkerはcreated16/removed16/retained0。専用TEMPは新規作成直後、list後、新16件後、既存各1件後、終了時のすべてで子要素0である。fixture自身による所有合成データの削除だけが行われ、runnerの追加削除は不要であった。空の専用親directoryを保持した。

補助集計では、既存4件の空stderrを `Get-Content -Raw` で読むとnullとなり、regex集計で非終端例外が各1件発生した。これはtest processの失敗ではなく、各ExitCode0とPASSを別途取得済みである。ログの再読を `[IO.File]::ReadAllText` で行い、4件ともstderr 0bytes・marker各0を確認した。testは再実行していない。集計例外を隠して全面無エラーとは報告しない。

## 証拠と引継ぎ

新規ログは `logs/history-main-consumer-runner-` prefixに `list`、`new`、`legacy-1`～`legacy-4` の各 `.stdout.log` / `.stderr.log` として保存した。前後hash・各PID/exit・専用TEMP検査は今回の実行出力と本書へ記録した。作者・実装者の旧ログと失敗証拠は保存したままである。

本単位の20件独立実行は合格であり、ソース・期待値・設定の変更や弱化は行っていない。`backups`通常file試験はpending作成後・metadata書込前の拒否であり、finish後半障害の新規証明ではない。backup内容root正規化（b-B）、他consumer、I10全体検証、製品release buildは今回対象外である。Windows納品時の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を代替しない。

最終Postは別担当 `gpt-6-astra / high` へ戻す。native microphone/pen/TTS/IME、実録音・実Codex認証/APIの新規受入は実施しておらず、今回の合成データテストで完了とは扱わない。製品EXE生成・置換・ユーザーアプリ終了、再委任、公開は行っていない。測定記録とcurrent更新は親所有へ返す。
