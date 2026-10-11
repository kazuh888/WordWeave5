# I2a-G 独立テスト / RED 引継ぎ

2026-10-08 / ACOUSTIC-HISTORY-001。役割はtest author、planner指定 `gpt-6-astra` / `high`。実行metadata・usage開始/終了カウンターは未取得である。AGENTS.md、development-team.md、test-driven-development SKILLのテスト工程を適用した。

承認入力はimplementation.md I2a-G、design-i2.md §3/§4「I2a-G限定接合」（SHA-256 `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA`、独立Pre PASS）。親からI2a-0 Post PASSとscaffold所有返却を受領して着手した。受入は単一gateによる全metadata/history書込の直列化、再入拒否、poison/block維持、strict pendingと互換boolのfail-closed（I2-T04）である。Default v2、移行、read_progress、結果不明判定、後続UIは対象外である。

## 完了と所有

- 新規 `tests/audio_history_gate.rs` と、`src/store.rs` の `#[cfg(all(test, windows))] pub(crate) mod i2_gate_tests`、`src/commit.rs` の同専用moduleだけを編集した。
- 既存本体と既存testのprefix SHAは受領scaffoldと一致する。`src/audio_history/store.rs` と既存integration testsは変更していない。既存dirtyを保持した。HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。
- 新規24件を固定した。公開targetは16件中4 PASS / 12行動RED、lib追加は8件中8行動RED。compile失敗、fixture失敗、timeout、ignore追加はない。
- REDは新契約未実装と現行公開入口の不足を示す。private scaffold後段の全契約を実行済みとする意味ではない。特にreparse・block・poisonの未到達assertはGREENで確認が必要である。
- 本報告と `artifacts/history-gate-tests-*` を保存し、親へ所有を返却する。実装担当は本体だけ、作者の期待値を変更しない。

## fixture / oracle

一意の `ww5-i2-gate-*` / `ww5-i2-gate-lib-*` 一時directory、v1 `Progress::default()`、合成変更topic、既存builtin教材を使った手製pending JSON、合成原音1件のCreateTrial requestを使用する。履歴writerの正常呼出で有効なprepare/publication/confirmation tokenを用意する箇所は、再入／gate試験の前提作りであり、その生成値を拒否・区間非重複・bytes保持の期待値に流用しない。正本headは既存 `CommittedHistoryHead` 接点の小さいローカルadapterで固定する。実Progress headの新実装はGの範囲外である。

再入8入口はそれぞれ独立rootの正常単独実行が成功してから、別rootのHistoryFaults `BeforeIntent`中で呼ぶ。期待は内側の拒否と内側rootの保存内容不変、外側prepare/publication成功、guard解放後の内側再試行成功である。履歴側の型付き拒否はBusyを要求する。文字列APIは表示本文をparseせずErrを要求する。

bytes snapshotは通常fileのexact bytesとdirectory集合を比較する。実Storageの排他lockが開いている `app.lock` だけは内容を再openできないためfile長で記録する。したがって「全内容file不変」と「live lockの内容bytes未読」を区別する。大きな教材bytesをpanic本文へdumpせず、同じ比較条件で短い失敗文にしている。

危険な再入・競合・poison/block・gate-free出口は同EXEのexact caseだけを子processで実行する。child環境markerは `Command::env` で渡し、親processの環境/global gateを変更しない。親はcase名・running 1 test・成功終了・1 passed/0 failed・固有成功markerを照合する。15秒deadline超過では子をkill/waitして失敗にする。guardはthread間へmoveせず各threadで取得・解放する。reset/poison解除APIはない。

競合caseは開始/入場/解放をchannelで観測し、callback区間の同時数、両操作の正常完了、各rootの3物件closureと原音bytesを確認する。150msの競合機会だけを成功根拠にしない。実測REDでは同時区間の重複数が1であった。有限のスケジュール検証であり、全スケジュールの証明ではないため、入口ごとの決定的な同thread再入試験を併用する。

## 公開target 16件の実測

| case | 独立期待 / 実測 |
| --- | --- |
| reentry_storage_save | 正常対照成功後、再入で内側root bytesが変化。行動RED |
| reentry_storage_load | 手製pendingの正常復旧対照成功後、再入復旧でbytesが変化。行動RED |
| reentry_material_save | 正常対照成功後、再入教材保存でbytesが変化。行動RED |
| reentry_material_recover | 正常対照成功後、再入復旧でbytesが変化。行動RED |
| reentry_history_prepare | 正常対照成功後、再入prepareがintentを書いた。行動RED |
| reentry_history_publish | 正常対照成功後、再入publishが物件を書いた。行動RED |
| reentry_history_resume | 正常対照成功後、再入resumeがBusyではなく成功。行動RED |
| reentry_history_finish | 正常対照成功後、再入finishがintentを削除。行動RED |
| normal_public_operations_complete_without_recursive_gate_deadlock | 8入口の正常完了対照。PASS。GREEN後は内部under_guard接合のdeadlockも検出する |
| head_callback_reentry_rejects_save_without_changing_progress | head callback内のStorage saveが成功してしまう。行動RED |
| public_pending_bool_is_fail_closed_for_missing_or_non_directory_root | root欠損を互換boolがfalseへ落とす。行動RED |
| pending_leaf_directory_blocks_public_writers_and_preserves_bytes | leaf directoryの保存/復旧拒否、内容file保持。PASS |
| normal_parent_missing_pending_leaf_is_false_without_mutation | 正常directory親のleaf欠損はfalse、無変更。PASS |
| panic_inside_writer_poison_stops_metadata_and_history_prepare | 実callbackの意図的panicを捕捉後もStorage saveが成功。行動RED |
| unrelated_roots_have_nonoverlapping_writer_intervals_and_both_finish | 両writerが正常完了したがactual overlap=1。行動RED |
| callback_can_use_gate_free_reader_head_confirmation_and_atomic_write | reader/head/confirmation/隔離先atomic_writeがcallback中に成功し履歴不変。PASS |

## lib追加8件の実測と未到達範囲

prefixはそれぞれ `store::i2_gate_tests::` と `commit::i2_gate_tests::`。

| case | 実測RED / GREENで到達確認する範囲 |
| --- | --- |
| acquisition_reentry_and_drop_leave_thread_ownership_consistent | 最初の正常取得が固定Poisoned。再入Reentrant、drop後再取得、他thread正常取得は未到達 |
| block_applies_to_existing_guard_under_guard_and_all_writer_entries | 有効履歴fixture/token作成・confirmation成功後、gate正常取得が固定Poisoned。既存guardのblock反映、under_guard拒否、全8書込入口拒否、reader/confirmation許可、bytes不変は未到達 |
| poisoned_guard_is_not_cleared_and_blocks_metadata_retry | poison前の正常取得が固定Poisoned。意図的panic後のPoisoned維持・再試行拒否は未到達。公開callback poisonの不足は別caseで再現済み |
| pending_normal_parent_distinguishes_missing_leaf_from_regular_file | 正常親＋leaf欠損が固定Ioで、Ok(false)と不一致。通常fileのOk(true)等は未到達 |
| pending_directory_leaf_and_non_directory_ancestry_are_unsafe | leaf directoryがUnsafePathではなく固定Io。root/祖先の非directory枝、bool/save/recoverの後続assertは未到達 |
| pending_os_io_rejection_fails_closed_without_any_write | `fs::symlink_metadata`へのNUL入りpathの実呼出がInvalidInput、raw_os_error=None。private入口は固定Ioだが公開boolがfalseとなりRED。拒否後のsave/recoverとbytes保持は未到達 |
| pending_missing_root_is_an_error_and_never_created | private固定Ioの後、公開boolがfalseとなりRED。後続保存拒否/未作成assertは未到達 |
| pending_reparse_ancestor_root_and_leaf_are_unsafe_without_mutation | **最初の正常親対照で固定Ioとなり停止。junction生成・ancestor/root/leaf拒否・保存不変は未到達**。GREENで全assert到達が必要 |

NUL負例の証拠はRustのfs APIがNotFound以外のpath入力エラーを返したことである。raw Windows error codeがなく、kernel I/Oの権限拒否を観測したとは主張しない。また固定Io scaffoldではpending_checked内部の分類処理を検証できない。実装後に同fixtureがIoへ分類され、bool trueと書込無変化の両層を通ることを確認する。共有拒否がmetadataへ影響するとの仮定、ACL変更、管理者権限は使わない。

## 実行履歴・再開時の扱い

```text
cargo test --locked --lib --test audio_history_gate --no-run --message-format=json
cargo test --locked --test audio_history_gate --no-run --message-format=json
target/debug/deps/wordweave5-b3388a7bbef44bb4.exe i2_gate_tests --list
target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --list
target/debug/deps/wordweave5-b3388a7bbef44bb4.exe i2_gate_tests --nocapture --test-threads=1
target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1
rustfmt --check --edition 2021 --config skip_children=true tests/audio_history_gate.rs src/store.rs src/commit.rs
git diff --check
```

- 初回集約compile成功、2m20s。続いて統合テストのsnapshot失敗表示を短くしたため、変更targetだけ再compileした。2m03s、成功。これは期待値緩和ではなく、大きなbytes出力で子のpipeを塞がないためのfixture出力修正である。両buildともcompile errorなし。
- 最初のsandbox外実行要求は利用上限により自動承認レビューが完了できず、実行されなかった。安全性の否認ではない。利用者の再開指示後、残存sessionのexit 0・保存済みbuildログ・EXE hashを確認し、**新しいbuildを開始せず**完成EXEを利用した。
- 親が許可した通常非管理者sandbox外実行である。Administrator tokenなら中止する事前検査を通過した。UAC/RunAs/OS設定/ACL変更はない。
- lib inventoryは8 tests/0 benchmarks。実測exit 101、0 PASS / 8 FAIL / 0 ignored / 145 filtered、0.26s。
- integration inventoryは16 tests/0 benchmarks。実測exit 101、4 PASS / 12 FAIL / 0 ignored / 0 filtered、3.90s。
- child timeoutなし、fixture/setup failureなし。全24件はcase名をinventoryで確定した。private未到達部分を実行済みへ数えない。
- 実行前後の両EXE hashは同一。終了時に `ww5-i2-gate-*`（lib用prefixも包含）の残存directoryは0。
- fmt/diff check成功。変更対象prefixは受領値と一致。全体suite/release build、既存reader/store/recovery回帰の新実行は本RED工程に含めない。

証拠は [集約build stderr](artifacts/history-gate-tests-build.stderr.log)、[集約build JSON](artifacts/history-gate-tests-build.jsonl)、[統合再build stderr](artifacts/history-gate-tests-integration-build.stderr.log)、[統合再build JSON](artifacts/history-gate-tests-integration-build.jsonl)、[lib inventory](artifacts/history-gate-tests-lib-list.log)、[integration inventory](artifacts/history-gate-tests-integration-list.log)、[lib RED](artifacts/history-gate-tests-lib-red.log)、[integration RED](artifacts/history-gate-tests-integration-red.log) に保存した。

## 固定SHA-256

| 対象 | SHA-256 |
| --- | --- |
| src/store.rs全文 | AFD717F43ECDDEFB542F4288912A18019B938C3D9E45ABA95A40F7070B91600C |
| store新cfg(test)より前の既存prefix（LF・末尾改行1文字） | F3D8C6546E4122B798F74586ECC51A3BDB6D37F457E86B0899A92F5A8A764B0A |
| store新cfg(test)からEOF、UTF-8 | 2AB39F8D08B1E2265180B9A948A868EC0AE8E279E798F7CDD2418F0AA82BC73C |
| src/commit.rs全文 | C2D8D88E98A5D35DAFFF6BB2ACFA9C81E16891AB648A1525C444F5A89C67F837 |
| commit新cfg(test)より前の既存prefix（LF・末尾改行1文字） | 2AC0132E87F59F913F189C13F7B20895AFC4DDB9204C9223B7F455F448839A15 |
| commit新cfg(test)からEOF、UTF-8 | 9A1E7F3B7497D43A1F9426538990E6E7642ED034C04565F2B7184DA801686205 |
| tests/audio_history_gate.rs | 5A9706DCDB41156119D4CB014692E5FF7748865950A254A32F0C7D1093B86DCC |
| src/audio_history/store.rs（未編集） | ACD8C9938DD08AFB32239BE7BB875A4F1B1A0ADF9E9E5DCF7771A0780C4F9E59 |
| lib EXE before = after | 249ED6880D077FC4FA211C156DEA576A24A4C35027EE7D76E12094EC5E417D85 |
| integration EXE before = after | 6D87E92B76C7D9CC08F51668C36964994DFAE7988AF74B24FEB70AE2CAEFC13B |

## implementer / runnerへ

新規24件の期待値を維持してGの本体を接合する。特に通常親とleaf欠損の区別、UnsafePath/Io/Busyの型、gate取得から副作用終了までの保持、同thread拒否と別thread待機の違い、block後のunder_guard検査を確認する。private未到達assertのGREEN確認が必須である。

GREEN後のfocused対象は `cargo test --locked --test audio_history_gate`、libの `store::i2_gate_tests` / `commit::i2_gate_tests`、既存 `--lib store::` / `commit::` と `audio_history_store` / `audio_history_recovery` である。exact childの件数/marker/timeoutを維持し、0件成功やfixture権限失敗を受入へ算入しない。既存Windows leaf-symlink特定1件の別実行承認を新caseに流用しない。

実データ、実API、default root、録音、認証、公開、再帰委任には接触していない。Gのread-only出口対照は、未実装のread_progressや書込停止後UI閲覧接続の完成を意味しない。
