# I5-M1 利用者環境での修正後成功

2026-10-07受領。利用者が提示した同一exact試験の実行出力を記録する。親が直接実行した結果ではない。今回のPowerShell版・昇格状態・正確な実行時刻・実行時hash出力は提示されていないため推測しない。親が21:50:02 JSTにsource/test/EXE/childの現物hashを照合し、独立runnerが凍結した版と一致した。

command：

```powershell
& 'F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\deps\acoustic_analysis-8d1dd6b6ba45c163.exe' --exact process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --nocapture
```

結果：1 passed / 0 failed / 0 ignored / 53 filtered、0.08秒。

```text
supervisor_in_job=false
child_in_job_at_create=true
child_wait_before_cleanup=WAIT_EVENT(0)
sentinel_absent=true
sibling_alive_before_cleanup=true
watchdog_rescued=false
child_cleanup=WAIT_EVENT(0)
```

外側Jobによる救済がない前提で、作成直後から子がJobへ所属し、親強制終了後のcleanup前観測で既に終了した。未resume・兄弟生存・watchdog救済なしの各条件も成立した。旧実装の [RED](praat-m1-user-red.md) と同じoracleであり、当該P1に必要なGREENが得られた。他の音響解析・保存・UI・アプリ全体の受入成功を意味しない。

| 対象 | 照合SHA256 |
| --- | --- |
| process_windows.rs | 10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C |
| process_unit.rs | CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896 |
| 新M1 EXE | D79B755753C8335403F1B7F8B7931FE24E87588E7EA9A152EEC7BBCA174B2913 |
| 合成child EXE | 3AD7B416C28C01D13C7FAB7A4F1961D99CD79082A4A341C52C91CBAACBCE204B |

本証拠を既存コード確認・53/20/59回帰証拠と併せて独立reviewerへ返し、M1だけの最終判定を依頼する。コード/期待値/生成物を変更せず、完了済み試験を再実行しない。
