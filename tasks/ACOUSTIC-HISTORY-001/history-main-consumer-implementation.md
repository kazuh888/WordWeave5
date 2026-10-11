# I2b-A 主要consumer実装・focused確認

2026-10-11 / ACOUSTIC-HISTORY-001。担当 `main_consumer_impl`、計画者指定 `gpt-6-astra / high`。実行model/effort metadata、使用量の開始/終了counterと親子包含関係は未取得である。開始絶対時刻は未採取、検証終了は19:48:20 JSTであり、欠測から所要時間・トークンを推定しない。親の同一task測定へ引き継ぐ。

承認入力は [implementation.md](implementation.md) I2b-A・7役指定、[design-i2.md](design-i2.md) §6/I2-T02/I2b-A接合確認、[独立作者のRED証拠](history-main-consumer-tests.md) である。要求はAH-AC04/I2-T02、復元の既存保全はAH-AC16である。P最終Post依存解消と本体着手許可を親から受領した。完了境界は3関数の実装と指定20件のselfcheckであり、独立runner/Post・I10・利用者受入は別工程である。

AGENTS、開発7役、KPI、wordweave-change、git-workflow-and-versioning、incremental-implementationと参照Definition of Doneを読了した。Skillの最小変更・失敗時保全・差分確認を適用し、commitや全体検証は本担当の許可範囲に含めない。描画変更がないためwordweave-egui-uiは非該当である。

## 変更と所有

- `src/app.rs::persist`：`Storage::save_progress` の `Ok(saved)` を `self.progress` へ採用後、Completed・dirty解除・last_save更新を行う。Err時のdiagnostic/fatal、旧メモリ状態は保持する。
- `src/app.rs::import_deck`：`commit::save_progress` 成功返却値を採用後、deck・dirty・成功messageを更新する。pending時のfatal/Err、既存候補保持を維持する。
- `src/app.rs::restore`：参照検証、現在未保存状態の別backup、現diskのcopy、当日ai_calls最大値、reconcileの後に `self.progress = storage.save_progress(&progress)?` を実行する。成功後のepoch/chat reset等は既存順である。
- 本書とignoredのtaskログ・専用一時領域。

本体差分は3hunk、23追加/17削除である。同file末尾の独立作者433追加行は保持した。rootの先行読取・手動merge・新API・新worker・importのepoch追加・disk rollbackを導入していない。既存教材承認/登録フロー、failure時memory復元の撤去も行っていない。親のdesign/current/measurements編集および作者報告を保存した。再委任、commit/push、OS/恒久設定変更、実データ・live AI/APIは使用していない。

## 固定対象と不変証拠

基点HEADは `21c304afd86ae5c4a2af96be881e772dbdca86c8`。下記source/EXEはテスト前後一致である。

| 対象 | SHA-256 |
| --- | --- |
| 編集前app.rs（作者の433行込み） | `44961B13D8F98D317CFF35D17C1414E14804610B8F75DAD3709312D46D79E401` |
| 編集後app.rs | `C1706E880A27C44BDD205D1BA3FD0E01E8D7D43FF0D3F8543F3D62AE251EDA0A` |
| 編集後の新moduleより前の本文（LF・末尾改行1個） | `F11D5E8D549D6D114F432B56D5D3D2A7695E71C46C2DB77B1CD9215479F78FF3` |
| 独立作者の新cfg(test) module（LF） | `469B87EA860C07783EA3C446F1132EE08A8F8464EE6713515E5B4EF130303CEB` |
| `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `C6143EA4833832FE66B0F0F0B938A7EBE69EA8D2C6E2ADAF474AA1772CD932BA` |

HEADのapp.rs本文（末尾改行を1個へ正規化）、空行、現在の新moduleをメモリ内で結合したSHA-256が編集前作者値 `44961B13…` と完全一致した。したがって作者が固定した433行のoracleは不変である。HEADからのapp.rs差分は本体3hunkと作者追加1hunkだけであり、他の既存テストfileは変更していない。

## コマンドと結果

`cargo fmt --all -- --check` はexit0（書換えなし）、`git diff --check` はexit0。`cargo test --locked --bin wordweave5 --no-run` は1回だけ実行しexit0、1分48秒、上記EXEを生成した。buildログはlib警告2件・bin test警告4件を含むがcompile失敗はない。対象外警告の修正は行っていない。

この環境で作者が確立したHidden子process方式を使用した。専用TEMP/TMPはprocess内だけで `F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\i2b-a-impl-temp` に設定した。同じEXEの `--list` はexit0、280 tests/0 benchmarks、新16件と指定旧4件の実在を確認した。実行は各 `Start-Process -WindowStyle Hidden -PassThru -RedirectStandardOutput ... -RedirectStandardError ...` を `WaitForExit(50000)` で上限付き待機し、完了後 `WaitForExit()` とExitCodeを取得した。timeout・管理者権限・sandbox外実行は使用していない。

新moduleは引数 `i2_history_main_consumer --nocapture` で1回実行した。**16 passed / 0 failed / 0 ignored / 264 filtered、exit0、2.35秒**。作者の成功6件REDをすべてGREENにし、防御10件PASSを維持した。下記prefixは全件 `app::i2_history_main_consumer::` である。

| test名 | 結果 |
| --- | --- |
| `t02_persist_stale_none_adopts_a_and_preserves_drafts` | PASS |
| `t02_persist_stale_b_adopts_a_and_preserves_drafts` | PASS |
| `t02_import_stale_none_adopts_a_and_preserves_drafts` | PASS |
| `t02_import_stale_b_adopts_a_and_preserves_drafts` | PASS |
| `t02_restore_stale_none_adopts_a_and_snapshots_unsaved_drafts` | PASS |
| `t02_restore_stale_b_adopts_a_and_snapshots_unsaved_drafts` | PASS |
| `t02_persist_corrupt_root_preserves_state` | PASS |
| `t02_import_corrupt_root_preserves_state` | PASS |
| `t02_restore_corrupt_root_preserves_state` | PASS |
| `t02_persist_unknown_root_preserves_state` | PASS |
| `t02_import_unknown_root_preserves_state` | PASS |
| `t02_restore_unknown_root_preserves_state` | PASS |
| `t02_persist_pending_refuses_without_recovery` | PASS |
| `t02_import_pending_refuses_without_recovery` | PASS |
| `t02_restore_pending_refuses_without_recovery` | PASS |
| `t02_import_pending_creation_failure_and_later_persist_never_rollback` | PASS |

既存4件は同EXEへ `--exact <完全名> --nocapture` を渡し、各1回実行した。全件 **1 passed / 0 failed / 0 ignored / 279 filtered、exit0** である。

| 番号 | 完全名 | 秒 |
| --- | --- | --- |
| 1 | `app::harness_tests::json_restore_preserves_the_latest_unsaved_chat_draft` | 0.32 |
| 2 | `app::harness_tests::restoring_trash_closes_editors_and_selects_an_active_conversation` | 0.29 |
| 3 | `app::gfm_tests::gfm_registration_partial_save_never_reports_success_or_erases_recovery_state` | 0.30 |
| 4 | `app::ux_flow_tests::failed_finish_keeps_session_for_export_instead_of_claiming_success` | 0.16 |

新16件のstderr markerはcreated=16、removed=16、retained=0。新16件後と旧4件後の専用TEMP子要素はともに0である。テスト自身が所有fixtureだけを削除し、空の専用親directoryは残した。作者の一時領域や他taskのデータには触れていない。

## ログと残り

`logs/history-main-consumer-impl-` をprefixとして、`fmt.log`、`build.log`、`list.stdout.log`/`list.stderr.log`、`green.stdout.log`/`green.stderr.log`、`legacy-1`～`legacy-4`の各 `.stdout.log`/`.stderr.log` に保存した。20件とも新EXEの新規実行証拠であり、旧REDログを成功証拠へ転用していない。

割当本体・focused selfcheck内の未達条件や設計gapは検出していない。`backups`通常fileの失敗試験はpending作成後・metadata書込前の拒否であり、finish後半障害の証拠ではない。後半のR/T保存層証拠、b-Bのbackup内root正規化、他consumer、独立runner/Post、I10全体test/release build、native/実録音/実API受入は本担当の完了主張に含めない。最終受入は利用者に残す。

親へ本体所有を返す。後続独立runnerは計画者指定 `gpt-6.1-sol / medium` の別工程である。process限定workspace TEMPとHidden起動の既知手順を再利用したことで、作者が特定した環境由来fixture失敗の再試行を避けた。恒久harness/settings変更は提案・実施していない。
