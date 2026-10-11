# I5-M1 利用者環境での修正前再現

2026-10-07受領。利用者がCodex外で起動した管理者PowerShell 5.1と7.6.6の実行出力を根拠とする。親がその場で実行した記録ではない。実行時刻と利用者実行時のhash出力は未取得。21:34:02 JSTのローカル現物照合で凍結EXEのSHAは事前記録と一致した。

対象：`tools/acoustic-analysis/target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe`

SHA256：`AC0CF780F4FF2C6C1FCF732B6A5DB46F38EB03AC6D23D8183D3E7B7AFDA497A9`

引数：`--exact process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --nocapture`

両環境とも0 passed / 1 failed / 0 ignored / 53 filtered、5.06秒。共通の観測結果は以下である。

```text
supervisor_in_job=false
child_in_job_at_create=false
child_wait_before_cleanup=WAIT_EVENT(258)
sentinel_absent=true
sibling_alive_before_cleanup=true
watchdog_rescued=false
child_cleanup=WAIT_EVENT(0)
```

`child survived forced supervisor termination at AfterCreate` のassertで失敗した。外側Job未所属の前提が成立し、親強制終了後も未resumeの子が5秒間残留した。兄弟processは生存し、watchdog救済はなく、試験の明示cleanupで残留子が終了した。これは対象不具合のbehavior REDであり、Codex内での前提失敗とは異なる。PowerShellの版による相違はこの2件では観測されない。

M1の修正前再現待ちは解消した。凍結EXEの置換禁止を解除し、Pre-gate対象hash照合後のM1実装・新EXEでの同一oracle確認へ進む。利用者が管理者実行した事実は、エージェントによる追加UAC実行の承認へ拡張しない。製品EXEの更新・音響機能完成・修正後PASSを意味しない。
