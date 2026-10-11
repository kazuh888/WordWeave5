# I5-M1 独立runner結果

2026-10-07 / ACOUSTIC-HISTORY-001。plannerの implementation.md I5 指定と親handoffにより test runner `gpt-6.1-sol / medium`。実行metadata・使用量開始/終了counterは未取得である。AGENTS.md、development-team.md、KPI、wordweave-changeの検証・報告規則、M1 Brief/Pre-gate/独立tests/利用者RED/実装引継ぎを読了した。所有は本書と `artifacts/m1-runner-*` ログ、ビルド生成物だけである。source/tests/期待値/manifest/設定を変更していない。

## 固定対象

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` + 既存dirty差分。M1本体は `tools/acoustic-analysis/src/process_windows.rs` のみ。git未追跡の新crateであるため、HEAD差分だけで対象を識別せずファイルSHAを併記する。開始・終了の対象source SHA一致を確認した。関連source/tests/manifest/EXEの正確なpathとSHAは [hashログ](artifacts/m1-runner-hashes.log) に記録した。

| 対象 | SHA-256 |
| --- | --- |
| src/process_windows.rs | `10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C` |
| tests/support/process_unit.rs | `CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896` |
| tests/support/praat_child.rs | `8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228` |
| tests/praat.rs | `493DE1F13F4BD9F6BED64A202E11016F3A9538CCC7726773AF5297D1E5D50FF3` |
| 新M1 unit EXE | `D79B755753C8335403F1B7F8B7931FE24E87588E7EA9A152EEC7BBCA174B2913` |
| 合成child EXE | `3AD7B416C28C01D13C7FAB7A4F1961D99CD79082A4A341C52C91CBAACBCE204B` |

新M1 EXEの絶対pathは `F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\deps\acoustic_analysis-8d1dd6b6ba45c163.exe`、childは `F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\acoustic-test-child.exe` である。実装者報告のEXEではなくrunnerの新compileと公開target build後の現物である。

## 新規実行結果

すべて合成fixtureだけを使用。scoped `require_escalated` は非管理者sandbox外実行であり、UAC/Windows設定変更はない。unit EXEは `Start-Process -WindowStyle Hidden`、45秒 `WaitForExit` と所有process限定監視で実行した。監視による救済killは不要だった。

| command | exit / pass・fail・ignored・filtered | ログ |
| --- | --- | --- |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline --no-run` | 0 / compile成功12.69秒 | [compile](artifacts/m1-runner-compile.log) |
| 上記unit EXE `--exact process_unit::inheritable_parent_event_is_excluded_from_child_handle_list --nocapture` | 0 / 1・0・0・53、0.09秒 | [event](artifacts/m1-runner-event.log)、[stderr](artifacts/m1-runner-event.stderr.log) |
| 上記unit EXE `--test-threads=1 --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling` | 0 / 53・0・0・1、3.61秒 | [unit初回](artifacts/m1-runner-unit.log) |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --test praat --locked --offline -- --test-threads=1 --nocapture` | 0 / 20・0・0・0、94.27秒 | [公開Praat](artifacts/m1-runner-public.log) |
| 同unit EXEの53件、最終childで再確認 | 0 / 53・0・0・1、3.76秒 | [unit最終](artifacts/m1-runner-unit-final.log)、[stderr](artifacts/m1-runner-unit-final.stderr.log) |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline -- --test-threads=1` | 0 / 59・0・0・0（32+27）。unit/publicは0件 | [default](artifacts/m1-runner-default.log) |
| `rustfmt --edition 2021 --check tools/acoustic-analysis/src/process_windows.rs tools/acoustic-analysis/tests/support/process_unit.rs` | 0 | [fmt](artifacts/m1-runner-fmt.log) |
| `git diff --check` | 0 | [diff](artifacts/m1-runner-diff-check.log) |

公開20件は通常cargoのconsole無し条件で実行し、Hidden console条件を使っていない。公開target buildでchild EXEのhashが実装者凍結版から変化したため、同じsourceでも実行入力の変更として最終53件だけ追加確認した。追加成功数を別testの網羅数へ加算しない。

通常sandboxでのevent失敗（STATUS_INVALID_HANDLE）は実装者ログとして受領した既存結果である。runnerの同一source新EXE・sandbox外 exact は成功し、環境差を確認した。Windows内部で異常終了を生じた具体機構や旧実装との帰属比較は未確定である。期待値の緩和・source診断追加は行っていない。

## 未検証と受入handoff

M1固有oracleは **1件未検証** であり、unitの明示filterに対応する。Codex内は外側Job未所属の前提が成立しない既知環境であるため再失敗を繰り返していない。過去の利用者旧EXE REDを新EXE GREENへ読み替えていない。

上記新EXEをCodex外でhash照合し、`--exact process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --nocapture` を実行する利用者確認が残る。supervisor外側Job未所属、生成時child所属、強制終了後child終了、sentinelなし、兄弟生存、watchdog救済なしのoracleを維持する。結果とexitを親へ返す。この新EXEとchildは以後再buildを停止する。

P1のClosed/Post-review PASSは判定しない。Q1/M2、I5c-2、資源実測、全アプリ `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5` は本handoffの対象外であり未実施。これはWindowsアプリ納品完了ではない。実microphone/pen/TTS/IME/auth、自然音声精度、実APIも未検証である。利用者アプリの終了やEXE置換、公開、再委任は行っていない。
