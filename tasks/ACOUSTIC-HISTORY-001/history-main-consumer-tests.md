# I2b-A 主要consumer 独立テスト・RED引継ぎ

2026-10-11 / ACOUSTIC-HISTORY-001。担当は `main_consumer_tests`、計画者指定 `gpt-6-astra / high` である。実行model/effort metadata、入力・キャッシュ・出力・推論の開始終了counter、親子包含関係は未取得である。終了記録時刻は19:43:45 JST、開始絶対時刻は未採取であり補完しない。

入力は承認済みspecのAH-AC04、[design-i2.md](design-i2.md) I2-T02・§6 consumer表、[implementation.md](implementation.md) I2b-A・b-A準備追補・7役割当である。AH-AC16の旧復元保全も既存契約として維持するが、本試験だけでバックアップ機能全体の受入を完了とはしない。`agent-skills:test-driven-development`を全文読了し、独立期待値→compile→動作REDの作者工程だけに適用した。

所有変更は `src/app.rs` 末尾の新規 `#[cfg(test)] mod i2_history_main_consumer` 433行と本書だけである。本体修正、既存テスト変更、仕様変更、model/gate変更、再委任、実データ・外部API、commit/pushは行っていない。共有app.rsの作者編集を終了し、親を通じて実装者へ所有を返す。

## 対象と固定証拠

基点HEADは `21c304afd86ae5c4a2af96be881e772dbdca86c8`。着手前git statusはcleanであり、その後の親によるdesign/current/measurements変更を保存した。

| 対象 | SHA-256 |
| --- | --- |
| 着手前src/app.rs | `347C62479A485C549ED4986C6278438C082960F988EF5F73EF28C59E593D8D94` |
| 最終src/app.rs（433行追加） | `44961B13D8F98D317CFF35D17C1414E14804610B8F75DAD3709312D46D79E401` |
| 最終test EXE `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `495D940A8B63D5896C742F53D87DE39434F7D03E8F684D69EC9D44033ACD855D` |

最終実行前後のsource/EXE hashは一致した。追加モジュール前の本文を元のLF終端で切り出したSHA-256は着手前値と一致する。git diffは末尾への追加1hunkだけであり、本体bytes/動作は未変更である。`rustfmt --edition 2021 --config skip_children=true src/app.rs` と `git diff --check -- src/app.rs` を実施した。

## 16件の受入対応と実測

全件の主受入IDはI2-T02 / AH-AC04である。名前のprefixはすべて `app::i2_history_main_consumer::` であり、独立test関数16件、loop内subcaseの未到達による水増しはない。

| test名（prefix省略） | 修正前結果 |
| --- | --- |
| `t02_persist_stale_none_adopts_a_and_preserves_drafts` | 行動RED |
| `t02_persist_stale_b_adopts_a_and_preserves_drafts` | 行動RED |
| `t02_import_stale_none_adopts_a_and_preserves_drafts` | 行動RED |
| `t02_import_stale_b_adopts_a_and_preserves_drafts` | 行動RED |
| `t02_restore_stale_none_adopts_a_and_snapshots_unsaved_drafts` | 行動RED |
| `t02_restore_stale_b_adopts_a_and_snapshots_unsaved_drafts` | 行動RED |
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

最終結果は**16実行、10PASS、6FAIL、0ignored、264filtered、exit101**（2.40秒、PID26384）である。6件の失敗箇所は最終UI全Progress比較（作者版app.rs:2955）。disk全ProgressはAと候補非履歴fieldに一致し、dirty解除・pendingなし・history bytes不変、およびimport/restore固有の後段確認を通過した後、UIだけがNone/Bを保持して失敗する。既存保存層の成功を確認した上で今回の未接合を再現しており、compile errorやfixture失敗をREDに数えていない。

成功期待値は候補全Progressを基礎に、履歴は固定A、importの新教材fingerprint1件、restoreの当日ai_calls最大値7だけを独立に設定する。設定・会話下書き・日本語下書き・合成material_draftを全比較に含む。importは既存教材の複製に専用idを付けた合成追加であり、既存復習状態を変更しない。

restoreではUIを復元候補の下書きへ置換する。旧UIの未保存下書き・設定・日本語下書きは先行memory backupへ残ることを別に確認する。旧UI下書きを復元候補へmergeする期待は置かない。backup内部root正規化はb-B所有のため本試験のoracleにはしない。

拒否9件は事前の全file/dir ledgerとUI全Progress・deck・dirty・last_save・epoch・messageを確認する。restoreだけは既存の先行memory backupとdisk copyの追加を認める。既存fileの変更、その他artifact追加は認めない。pendingは同じAを持つ有効な手製intentであり、recoverすれば別topicと追加教材になる状態を置いて、consumerの保存が暗黙recoverしないことを検出する。

最後の1件は実filesystemの`backups`通常fileを置く。新intent作成後、`recovery_archive_destination`の事前検査で拒否され、metadataはまだ変更されない。pending内容のA・draft・旧progress exact bytes・追加教材を確認し、fatal後のpersist2回でもUI/metadata/pendingが不変であることを確認する。**finish後半の障害を再現した試験ではない**。その境界は既存R/T保存層の証拠に委ね、新seamは追加していない。同期3consumerには新しい取消し入口がないため、I6のcancel/遅延結果は本単位対象外である。

## Fixtureと実行環境

既存`harness_tests::fixture`の実WordApp/Storageを使う。Aは手製82bytesの空root、SHA-256は`ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0`。開始時にclosureを検証する。stale Bはsynthetic RootRefであり、Bのobjectを候補から取り込む権限を与えない。未知rootはschema_version=999の別手製bytesをhash整合付きで置く。破損rootはAのobjectを壊す。候補は既存の合成`material_ui_draft`を含めvalidate成功を確認する。ネットワーク・実学習記録を使わない。

初回の`cargo test --locked --bin wordweave5 i2_history_main_consumer`はcompile成功後、sandbox package TEMP下の既存harness初期`storage.save`（harness_tests.rs:202）で16件失敗した。consumer本体に未到達であり、**環境下のfixture失敗16件**として保持する。失敗syscall/OS値は未取得で、hard_link等の原因を断定しない。cleanup guardより前の失敗であり、初回16rootの個別path/掃除完了は未採取である。無関係な一時領域は削除していない。

次のworkspace TEMPへのnative直接呼出しはexit0・0byte logで、後段出力もなく実行数を取得できなかった。テスト成功とは数えない。`Start-Process -WindowStyle Hidden -Wait -PassThru`にすると同workspace内で一覧16件を取得できた。親の追加保全依頼でmaterial_draft sentinelとcandidate validateをまとめて追加し、最終compileを1回実施した。

最終実行は**sandbox内・process限定TEMP/TMP**を次へ設定し、Hidden/Waitで起動した。恒久環境設定、OS権限、UAC、sandbox外実行は変更/使用していない。

`F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\i2b-a-author-temp`

最終実行のfixture-created/removed markerは各16、retained markerは0、実行後この専用rootの子要素は0である。並行testのstderrは一部行が交錯するため、markerは行頭件数ではなく出現回数を数えた。RAII DropでStorage lockを先に解放し、RED unwindを含め今回のfixtureだけを掃除した。空の専用親directoryは残している。

## 再現コマンドと後続担当

通常の焦点コマンドは計画どおり `cargo test --locked --bin wordweave5 i2_history_main_consumer`。今回環境ではbuildと実行を分けた。

```powershell
cargo test --locked --bin wordweave5 i2_history_main_consumer --no-run
# 表示されたEXEを固定し、source/EXEの前後hashを記録する。
# 各担当専用の新規ignored artifacts directoryを作り、このprocessだけで設定する。
$env:TEMP = $testTempRoot
$env:TMP = $testTempRoot
$testProcess = Start-Process -FilePath $testExe -ArgumentList 'i2_history_main_consumer','--nocapture' -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog
$testProcess.ExitCode
```

`$testTempRoot`等は担当者が明示的な所有pathへ割当する変数であり、未設定のまま実行しない。実装者は本体persist/import_deck/restoreだけを修正し、保存成功返却値を採用した後に既存dirty等を更新する。期待値の変更、app.rs:2229付近の既存memory-only rollbackの撤去、importへの新epoch更新は行わない。6件をGREENへし、防御PASS10件を維持する。

最終EXEの`--list`で全280件、新16件、以下の既存4件の実在を確認した。親指示により作者は4件を実行せず、実装selfcheck/独立runnerへ引き継ぐ。

- `app::harness_tests::json_restore_preserves_the_latest_unsaved_chat_draft`
- `app::harness_tests::restoring_trash_closes_editors_and_selects_an_active_conversation`
- `app::gfm_tests::gfm_registration_partial_save_never_reports_success_or_erases_recovery_state`
- `app::ux_flow_tests::failed_finish_keeps_session_for_export_instead_of_claiming_success`

全suite/release build、I10、実録音・native入力・実API受入は本作者工程では未実施であり、後続gateを代替しない。

## ログ

- `logs/history-main-consumer-red.log`：初回compile成功、fixture失敗16件。初版source hash `97EC4D0B7D923E95ED694EE87A3B63F0EEBCC03E9DDB3A200CEE7BF7D85CF900`、初版EXE hash `E44734F558295529721172CA008D1774368E4FBE0BD7772F3B69364526D0EFC3`。
- `logs/history-main-consumer-workspace-red.log`：native直接起動の0byte記録。実行数不明。
- `logs/history-main-consumer-workspace-list.log` / `.stderr.log`：Hidden起動で16件一覧、exit0。
- `logs/history-main-consumer-build-final.log`：material_draft追加後の最終compile成功。
- `logs/history-main-consumer-final-red.stdout.log` / `.stderr.log`：最終16件の10PASS/6FAIL、行動RED理由、fixture marker。
- `logs/history-main-consumer-final-list.log`：最終280件の一覧と既存4回帰の実在確認。

改善機会は、sandboxの既定TEMPを避けるprocess限定の所有一時領域とHidden/Wait実行を後続担当へ渡すことである。恒久harness/settings変更は提案・実施していない。独立runner/Postの合格は未了である。
