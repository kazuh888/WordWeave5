# CANCEL-A1 最終EXEの利用者M1結果

2026-10-08受領。利用者が前回依頼したCodex外PowerShellのexact実行結果として、以下を提示した。実行開始時刻、PowerShell版、管理者状態、実行時hashは提示されていないため補完しない。

```text
> & 'F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\deps\acoustic_analysis-8d1dd6b6ba45c163.exe' --exact process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --nocapture

running 1 test
M1 evidence: supervisor_in_job=false child_in_job_at_create=true child_wait_before_cleanup=WAIT_EVENT(0) sentinel_absent=true sibling_alive_before_cleanup=true watchdog_rescued=false child_cleanup=WAIT_EVENT(0)
D1 events=["Registered ExplicitTrack pid=9424", "Registered ExplicitTrack pid=3296"] calls=[] records=[] poison=false capacity=false invalid=false expected_wait=0
test process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 0.11s
```

## 固定版との照合

親が受領後に現物を読取照合し、[最終runner](process-cancel-a1-runner.md)の実行前後固定値と一致した。このターンではbuild・test再実行・実行物変更を行っていない。

| 対象 | 受領後SHA256 |
| --- | --- |
| unit EXE（上記path） | `D923AFE3CE225EC3A48AC09E39B8D9B4C798E298481022F07B8DFC7AD4206A59` |
| tools/acoustic-analysis/target/debug/acoustic-test-child.exe | `8438841AF3D2853AB55C5DC89B82D135DF8BA254890EE0867076EC601565BAF2` |

runner終了時source inventory17件も現在hashと全件一致した。branchは`codex/development-agent-team`、HEADは`b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。dirty変更は保持する。利用者実行時のhashを別取得したとは主張しない。

## 読み取れる結果と限界

外側Jobなし、作成時の子Job所属あり、救済前の子終了、未resume、兄弟生存、watchdog非救済が確認できる。後始末記録は空、poison/capacity/invalidはfalse。これは更新版M1の1件成功であり、前回のunit93件と同一バッチで94件成功したという意味ではない。

旧cancel失敗の根本原因を特定した証拠ではない。I5全体・履歴機能・製品EXEの完成とも区別する。独立Post判定は[gate記録](m2-w1-gates.md)へ集約する。
