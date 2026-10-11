# I5-M2 独立runner結果

2026-10-07 / ACOUSTIC-HISTORY-001。**全回帰gateは未達である。** consoleなし全targetsはprocess取消しcaseでabortし、crate全体fmtも既存lib.rsの並び順で失敗した。M2独立15件、公開実Praat20件、core32件、schema27件、no-default構成59件は今回成功した。後述のHidden console比較1件の成功で最初の失敗を撤回しない。

## 割当・対象・測定境界

計画 `implementation.md` I5 runner=`gpt-6.1-sol/medium`、親dispatchも同指定。実行model/effort metadata、使用量開始/終了counter、親子包含は未取得である。子IDは `/root/m2_w1_runner`。W1完了後のfollowupとしてM2のみ実施し、開始hash採取22:28:18 JST、終了hash採取22:32:50 JST（最終応答前まで）である。

AGENTS、development-team、KPI、ww-test-runner、wordweave-changeは前段W1で全文読了済みであり、今回はその検証・報告規則を継続適用した。承認入力は `praat-m2-design.md` §10、`praat-m2-tests.md`、`praat-m2-implementation.md` と親の本体所有返却である。AH-AC05/09/10のWAV同一性、取消し・期限・cleanup、T6公開回帰が対象。所有は本書、`artifacts/m2-runner-*`、build生成物だけであり、source/test/期待値/settings/Git/UAC/API/実データは変更していない。

基準HEADは計画書記載 `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`（今回Gitで再取得した値ではない）。対象diffは親保存 `artifacts/m2-impl-source.diff` SHA256 `94DEA3842F7E886B040609971C32EF8C0636869AF4F6574C825069D069D93B3A`。開始・終了の次の現物SHAは一致した。詳細終了ログは `artifacts/m2-runner-end-hashes.log` である。

| 対象（tools/acoustic-analysis/以下） | 開始・終了SHA256 |
| --- | --- |
| src/praat.rs | 4BBCB967106126A7DAB101EF99E5128C38BA60081A65E102324248404E97EC53 |
| tests/support/praat_unit.rs | BD1D3ADF3524F11698538F29ED023D960D7B69CA3A7FABD326C6FCE885A899B1 |
| src/process.rs | 9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54 |
| src/process_windows.rs | 10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C |
| tests/support/process_unit.rs | CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896 |
| tests/support/praat_child.rs | 8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228 |
| Cargo.toml | 08860AAEC9993B8886206F384771640170312CC05A72D0714AEA0DA3DD6789EC |
| Cargo.lock | 567E766C5E3BB1B508E66A4D32588B2C1CC61CBDE530D462F2B045540415D73F |

lib.rsはsnapshot `artifacts/m2-before/lib.rs` と現物SHAがともに `9238732D20C1DF3AA266FB9CCFABEE53FE3B2550FEFA1AD05D1CEEAA965DB0B5` である。M1関連4ファイルは既存M1判定/実装引継ぎの現物hashと一致するため、`praat-m1-post-review.md` と `praat-m1-user-green.md` の旧M1成功証拠を再利用可能とした。今回M1 exactを実行したとは扱わず、configured filter 1件を成功件数へ加算しない。

## 新規実行とログ

全cargo試験はmanifest `tools/acoustic-analysis/Cargo.toml`、`--locked --offline` を指定した。長い出力は各ログへTeeし、直後の `$LASTEXITCODE` を `runner_exit_code` として保存した。features praatの試験は親指定のscoped require_escalatedで非管理者sandbox外、通常cargoのconsoleなし条件。no-defaultは通常sandboxである。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --features praat --locked --offline -- --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --test-threads=1 --nocapture
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --test praat --test core --test schema --locked --offline -- --test-threads=1 --nocapture
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline -- --test-threads=1 --nocapture
cargo fmt --manifest-path tools/acoustic-analysis/Cargo.toml --all -- --check
rustfmt --edition 2021 --check tools/acoustic-analysis/src/praat.rs tools/acoustic-analysis/tests/support/praat_unit.rs
```

| 実行 / ログ（artifacts/以下） | 結果 |
| --- | --- |
| 全targets features praat / m2-runner-praat.log | build17.86秒。selected68件のうち48件okを観測、1case失敗・二重panicでabort、残19件未到達。M1 configured filter1。exit -1073740791（0xc0000409）。suite成功summaryなし |
| 未到達target補完 / m2-runner-remaining-targets.log | build10.40秒。core32/0/0（0.13秒）、公開Praat20/0/0（69.67秒）、schema27/0/0（0.06秒）、全target filtered0、exit0 |
| no-default全targets / m2-runner-no-default.log | build13.76秒。core32/0/0（0.13秒）＋schema27/0/0（0.07秒）=59 passed、unit/Praat targetはcfgで0件、filtered0、exit0 |
| crate全体fmt / m2-runner-fmt.log | lib.rsのmodule並び順だけdiff、exit1。snapshotから不変の既存状態である |
| 変更2ファイルfmt / m2-runner-changed-fmt.log | exit0 |
| 保存M2-only diff追加行の末尾空白検査 / m2-runner-diff.log | trailing-whitespace0、exit0。Git禁止のためgit diff --checkは未実行であり、その代替を同一チェックと称しない |

48件okの内訳はM2新規15、既存protocol31、process先頭2である。未到達target補完は最初のunit abortで実行されなかった範囲だけを実行し、失敗unitのfilter追加や再実行は行わなかった。no-defaultは別構成として指定された回帰であり、core/schemaの同名ケースを異なる新規ケースへ加算しない。既存dead_codeとincremental hard-link代替copy警告を保持した。

主要新ログSHA256：全targets `B83C866D92F0A901EB777FC6550597055DCBCA71BB05CAB246622B026DA4C2A9`、補完target `749CD84B70D60E400CD26554C93C244A7AD0AF06D27E412F36B4D5F583DCE9A7`、no-default `183F344CD0EFFBCD5B2A438F5844F4E57E84B64F6B70743B0EABB6DB934579DA`。

## 失敗と追加診断

失敗caseは `process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data`。`tests/support/process_unit.rs:172` のowned handle即時waitが `WAIT_EVENT(258)`、期待 `WAIT_EVENT(0)` となった。後続Dropの同mutex `PoisonError` により二重panic、abortとなった。取消し時点で所有processが生存とのoracle違反を観測したが、対象handle種別とOS内部原因はrunnerで確定していない。親へ即時返却し、test無効化・期待値緩和・本体修正を行っていない。

親の追加指示により、他の未到達試験完了後、現unit EXEの同caseだけを既存M1-runnerと同じHidden console条件で一度比較した。

```powershell
$m2Proc = Start-Process -FilePath 'F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\deps\acoustic_analysis-8d1dd6b6ba45c163.exe' -ArgumentList @('--exact','process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data','--nocapture') -WorkingDirectory 'F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis' -WindowStyle Hidden -RedirectStandardOutput 'F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\m2-runner-cancel-hidden-stdout.log' -RedirectStandardError 'F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\m2-runner-cancel-hidden-stderr.log' -PassThru
$m2Done = $m2Proc.WaitForExit(45000)
```

結果は **1 passed / 0 failed / 0 ignored / 68 filtered、0.21秒、exit0、watchdog_rescued=false**。非管理者sandbox外、有限45秒監視であり、監視によるkillは不要であった。EXE SHAは実行前・終了時とも `D17625844EEB5A2034B5B29B225B8B70881112A79D56DE467763872A7B801553`。compile後の現物でありauthorの旧EXE hashとは異なる。stdout/stderrを専用新ログに保存した。この条件差比較は原因確定でも全unit成功でもない。

失敗後の残存合成child照会は通常sandboxのCIMアクセス拒否で取得不能だった。ユーザーアプリ・任意processを停止していない。親の独立診断に引き継ぐ。

親の追加許可による最小A/Bとして、同じPowerShell exec shell・非管理者sandbox外・crate cwdで現EXEをconsoleなし直接呼出し、一度だけ同exactを実行した。直前SHAは同じ `D17625844EEB5A2034B5B29B225B8B70881112A79D56DE467763872A7B801553`。再compileなし。結果は **1 passed / 0 failed / 0 ignored / 68 filtered、0.06秒、exit0**、新ログ `artifacts/m2-runner-cancel-direct.log`。追加反復は行っていない。両exact条件で成功したため、console有無だけを一括時失敗の原因と断定できない。一括failureと未到達19件は未解決のままである。

```powershell
# cwd: F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis
& './target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe' --exact process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data --nocapture 2>&1 | Tee-Object -FilePath '../../tasks/ACOUSTIC-HISTORY-001/artifacts/m2-runner-cancel-direct.log'
$m2Exit = $LASTEXITCODE
"runner_exit_code=$m2Exit" | Tee-Object -FilePath '../../tasks/ACOUSTIC-HISTORY-001/artifacts/m2-runner-cancel-direct.log' -Append
exit $m2Exit
```

## 上限・native受入・停止

公開T6の60秒48kHz fixtureは成功し、constructor/extract/cleanupを含む共通deadline内の実測 **5,517ms**。WAV bytesは仕様式 `58+8×48,000×60=23,040,058` による計算値であり、この公開caseの実ファイルbytes/一時work peakを測定した値ではない。物理メモリpeak/child peak commitも未測定であり、512MiB上限を実測保証済みとは記さない。公開test自身もprivate telemetryが必要との未検証を明示する。

実マイク・pen・TTS・IME・実Codex認証・実Qwen送信・一般話者精度は未実施である。今回は固定実Praatと合成PCMの証拠である。root全体test/release buildは親指示により対象外であり、Windows納品時の `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5` を完了したとは扱わない。

全unitの残19件、consoleなし取消し失敗の診断、crate fmt failureの扱い、独立Post-gateは親へ返す。scope拡大・model/gate変更・公開は行わず、ここで停止する。
