# I5b Windows supervisor 独立テスト / RED handoff

2026-10-07 / ACOUSTIC-HISTORY-001。担当は test author、planner 指定は `gpt-6-astra` / `high`（implementation.md I5）。起動指定と実行値は別であり、runtime model/effort と開始・終了 token counters は未取得である。AGENTS.md、development-team.md、KPI、wordweave-change、agent-skills:test-driven-development を読了した。TDD のテスト/RED 工程だけを担当した。

入力は利用者承認済み仕様 AH-AC01/09 と design-i5.md §4/5/7/8 の監督契約である。AC09 の対象限定取消し・原本保持から、独立 process handle での終了確認、兄弟生存、合成原本不変を期待値とした。コードは未実装 stub であり、既報の実稼働 bug を再現した証拠とはしない。

所有成果物は `tools/acoustic-analysis/tests/support/praat_child.rs`、`tools/acoustic-analysis/tests/support/process_unit.rs`、本書だけである。親が Cargo.toml の bin と lib.rs の private unit module を接続した。本体・仕様・品質 gate・他担当ファイルは変更していない。

## テストと境界

22 tests（fixture self-check 1、supervisor 21）を Windows の実 process / Job / pipe / event で行う。fake は任意の固定挙動をする子実行ファイルだけであり、OS 監督を mock しない。PraatBackend の本番 hash pin は解除しない。

| AC / 契約 | 検査 |
| --- | --- |
| AC01/09 / 正常終了 | 公開 private supervise 窓口と hook 窓口、正常 exit と完全な stdout/stderr、nonzero=37 を adapter classifier へ返す |
| AC09 / 起動前封じ込め | AfterCreate / BeforeAssign / AfterAssign / BeforeResume で40ms待って sentinel 未生成、assign後 Job 所属、全4境界の故障で resumeせず独立handleが終了済み |
| AC09 / Drop | AfterAssign で実際に生存している子を持ったまま panic、catch後に独立handleの終了を検査。hook未到達のstub panicは合格にならない |
| AC01/09 / 出力資源 | stdout 32767/32768/32769、stderr 16383/16384/16385、完全bytesとVec capacity上限、両pipe同時洪水でstderr違反の観測、同時洪水中の取消し |
| AC09 / 取消し・期限 | pre-cancelと期限切れの優先、起動前期限切れ、BeforeResume取消しで実行0、その後正常再試行、実行中timeoutと独立handle終了 |
| AC09 / 所有限定 | child+grandchildを独立handleで保持してから取消し、兄弟fake child生存、合成原本bytes不変、親childの自発終了時にpipeを保持するgrandchildも回収 |
| AC09 / 非継承 | 親がinheritable eventを作り、childからその数値handleでSetEventできず親eventも未signaledであること |
| AC01/09 / メモリ | private 32MiB capで、起動後128MiBの実確保を試みる。通知によるMemoryLimit停止、またはallocator拒否sentinel＋既知OOM診断の安全側分類を要求する。128MiB確保成功sentinelは拒否する |
| AC09 / 引数・path | 空文字、日本語、空白、引用符、末尾backslashの独立length-prefixed byte oracle、Unicode/空白を含むEXE path、存在しないEXE、埋込NUL拒否 |
| AC09 / handle寿命 | warm-up後、成功12回＋assign故障12回のprocess handle count増加が2以下。テスト群を直列化し他のprocess単位検査との競合を避ける |

fixtureは専用tempdirだけへstarted/PID/合成sentinelを書く。原音・ユーザーデータ・API・実Praatは使用しない。各hookで開いた所有handleと孫PIDから独立して開いたhandleを保持する。8秒の試験watchdogが所有handleだけをterminateし、この救済が必要ならテストを不合格にする。通常終了とpanicのDropでも生存所有handleをterminate→wait→closeする。sleepするfake childは15秒で自発終了する二重の上限を持つ。兄弟は個別Child guardでkill+waitする。

このwatchdogはOS pipe/childの停止用であり、壊れたsupervisorの任意の無限CPU loopやOS wait全般の終了保証ではない。実装後runnerはtest実行自体にも別processからの上限を設け、所有test processだけを終了対象とする必要がある。親crashをこのwatchdogで試験したとは扱わない。

## 実行証拠

```powershell
cargo build --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --bin acoustic-test-child --locked
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib process_unit --locked --no-run
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib process_unit --locked -- --test-threads=1
rustfmt --edition 2021 --check tools/acoustic-analysis/tests/support/praat_child.rs tools/acoustic-analysis/tests/support/process_unit.rs
```

unit testでは `CARGO_BIN_EXE` に依存せず、current_exe の deps 親から `acoustic-test-child.exe` を解決する。実行前の明示bin buildは必須である。追加Windows featureは不要だった。

- 初回bin build成功、初回 `--no-run` 成功。logs: `praat-process-fixture-build.log` / `praat-process-compile.log`。
- 初回RED: 21件中1成功、20失敗、ignored 0、4.35秒。log: `praat-process-red.log`。
- 最終bin build成功。最終unit build成功後RED: 22件中1成功、21失敗、ignored 0、4.49秒。logs: `praat-process-fixture-build-final.log` / `praat-process-red-final.log`。終了code 101は意図したREDである。
- fixture self-checkは本体を通さず stdout=`OOOOO`、stderr=`EEE`、exit0、started生成を確認して成功した。他21件は process.rs の `todo!` に到達して失敗した。compile errorやfixture不存在をbehaviorのREDに数えていない。
- 修正したテスト設計は、メモリ通知の合法的競合、Unicode/空白EXE pathの追加、試験安全網での孫handle保有である。stubを合格させるための期待値変更はない。

RED時 SHA-256:

| ファイル | SHA-256 |
| --- | --- |
| src/process.rs（stub） | `72DF5C2E41B1662C86B652E153E719B6B13AC36414B8FDBC6CBEF86BE10A6F61` |
| tests/support/praat_child.rs | `8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228` |
| tests/support/process_unit.rs | `9111C9D4F6D9E3E496D015F4C1C17695041E6C3CAEC5F6F24DD7FBDBD492EE43` |

## implementer / runner への引継ぎ

supervise / supervise_with_hook / FaultPoint の固定signatureを使用し、hookで返すErrはそのkind/reasonを保って通常回収へ進める。OS層のOkは回収完了・bounded bytes取得という意味であり、未知nonzeroの分類は既存classify_outputへ残す。resource違反と取消し・期限をPartialへ変換しない。上記コマンドでGREENを取得した後、featureなしcore回帰と既定integrationをrunnerへ渡す。

未検証は、実装後の全21 behavioral checks、親の異常終了時kill-on-close、Job handle自体の非継承、複数子孫の合計memory cap、memory通知欠落のOS再現、OS wait失敗、BeforeCleanupのerror/cleanup-warning flag、adapter leaseの複数instance競合、parser失敗を含むadapter work ledger、固定公式Praat、I6共通deadline、全app test/release buildである。event非継承だけでJob非継承・親crashを合格にしない。これらはI5c/runner/後続gateへ残す。stop-to-reap 3秒は合成fixtureに対する異常検知予算であり、製品のOS最悪時間保証を新設したものではない。

本担当はここでテスト/fixtureの編集を止め、親を経由して実装者とrunnerへ引き渡す。実装・本番動作の合格は主張しない。
