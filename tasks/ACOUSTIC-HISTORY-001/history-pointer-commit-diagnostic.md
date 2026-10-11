# I2a-P 独立回帰4失敗の環境比較

2026-10-11。利用者のAstra/high切替連絡を受け、親が限定診断を実施した。対象は初回独立runnerの4失敗だけである。製品source・テスト・期待値・EXEは変更していない。最終Postおよび残190件の独立実行は未了である。

## 仮説と比較

初回79件中75PASS/4FAILを保持する。3件はDeviceIoControlでjunction fixtureを設定する際のOS5であり、製品の拒否検査には到達していない。Gの1件はpublishが返すIoであり、raw OS値・失敗syscallは取得できていない。Gは別processでprepare→publishを呼び、Pのcommit_historyを呼ばない。P本体変更で既存GやAssetStoreを変更していないことは独立reviewerが確認した。

同じ16source/9EXEを固定し、4件それぞれを `--exact <case> --nocapture --test-threads=1` で実行した。通常sandbox内、続いてexecのsandbox外実行を用いた。両側でWindowsPrincipalのAdministrator tokenを拒否し、UAC/RunAsは使用していない。OS/ACL/恒久的harness設定は変更していない。

各側で同一artifacts親の新規専用directoryを作り、TEMP/TMP/既存WW5_I2_GATE_LIB_ROOTをそのcommand内だけ設定した。祖先reparse拒否とcollision拒否を実施し、終了時に環境変数を復元した。fixture rootは兄弟pathであり、同一pathの上書き再利用ではない。親子directoryのACL全文を別採取した証拠はない。

| case（store::以下） | sandbox内 | 非管理者sandbox外 |
| --- | --- | --- |
| i2_gate_tests::block_applies_to_existing_guard_under_guard_and_all_writer_entries | exit101 / 失敗 | exit0 / 1PASS |
| i2_migration_tests::m03_wrong_types_and_real_junctions_are_rejected_before_backup | exit101 / 失敗 | exit0 / 1PASS |
| i2_normal_save_tests::n03b_daily_reparse_is_rejected_and_existing_regular_daily_is_not_replaced | exit101 / 失敗 | exit0 / 1PASS |
| i2_progress_tests::s03_unsafe_and_missing_paths_never_become_empty_history | exit101 / 失敗 | exit0 / 1PASS |

各実行の外側上限60秒、timeout0、ignored0。4件のsandbox内失敗は同じ失敗位置を再現し、外では後段assertまで通過した。Gの子成功markerは既存fixtureが内部でassertしており、成功stdoutへ個別出力していない。source/EXE25件は両側とも前後hash不一致0である。各case後と終了時の専用root残存0、終了時対象process0。空の今回所有directory2個だけを非再帰削除した。旧失敗ログや既存tempは保持している。

## 結論と限界

この固定EXE・fixture条件では、4件の結果がsandbox内外に依存することを確認した。製品コードや期待値を修正する根拠は得られていない。GのIoをhard_link失敗と断定する証拠はない。AssetStore::putのhard_linkは候補だが、他の書込・同期・読取もIoへ変換されるためである。

次の独立runnerは従来Gと同じ非管理者sandbox外で、失敗4件と未実行190件を実行する案とする。成功済み75件はsource/EXE固定を確認して維持する。今回の親による4件の診断成功を独立runnerの合格に置き換えず、初回失敗と別に記録する。残りを含めた受入・最終Postは保留する。

## 証拠

- [初回独立実行](history-pointer-commit-runner.md)
- [sandbox内結果](artifacts/history-pointer-commit-diagnostic-sandbox-results.json)、[環境・清掃](artifacts/history-pointer-commit-diagnostic-sandbox-environment.json)
- [sandbox外結果](artifacts/history-pointer-commit-diagnostic-outside-results.json)、[環境・清掃](artifacts/history-pointer-commit-diagnostic-outside-environment.json)
- `artifacts/history-pointer-commit-diagnostic-{sandbox,outside}-integrity-{before,after}.json`：16source/9EXEの現物照合。
- 同prefixの `0..3.stdout.log` / `0..3.stderr.log`：各exact実行の原ログ。
- [実行wrapper](artifacts/history-pointer-commit-diagnostic-wrapper.txt)：実引数・60秒上限・環境設定・token検査・清掃を含む。外側比較ではtaskModeだけsandboxからoutsideへ置換し、execのrequire_escalatedでsandbox外へ移した。
- 同prefixの結果JSONに個別所要秒を記録。開始終了の絶対時刻は別採取していないため補完しない。

指定reviewerはpointer_failure_review（gpt-6-astra/high）。実行model/effort metadata、開始終了usage counter、親子包含は未取得である。親はデバッグskillに従い、失敗再現→source局所化→固定EXE環境比較まで進めた。実データ・外部API・commit/push・製品EXE更新は行っていない。

## 独立比較レビュー

pointer_failure_reviewは4件の同位置失敗/外側成功、前後両側100hash記録の25path不変、非管理者・残存0を照合し、指定Sol/mediumで失敗4＋未実行190を実行する方針に阻止0と判断した。製品source/期待値修正は要求しない。G syscall特定は再開の前提ではなく、再び失敗した場合に限定診断する。

初回75PASSと再検証194件を集計する際、269件を単一環境・連続実行したとは書かない。既存jobsの対象名を照合し、recovery900秒を含む元上限を維持する。親の診断4件を独立合格として数えない。現在は利用者指示に従い、独立runnerへのモデル切替前で停止している。
