# I2a-0 Path reader 独立runner

2026-10-08 / ACOUSTIC-HISTORY-001 / `/root/history_recovery_runner`。planner指定 `gpt-6.1-sol/medium` を維持した。runtime model/effort metadata、開始/終了usage counter、親子使用量包含関係は未取得である。AGENTS.md、development-team.md、KPI、wordweave-changeを読了済みであり、同Skillのリスク別検証と報告だけを適用した。

入力は親の承認済みI2a-0 dispatch、[作者handoff](history-path-tests.md)、[実装handoff](history-path-implementation.md)。受入対象はAH-AC04/16に対応するPath reader入口・既存closure維持である。親からcode review PASSとソース固定を受領した。所有は本書と `artifacts/history-path-runner-*` のみであり、既存dirtyを保持し、source/test/期待値/設定を変更していない。

## 固定対象

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` と未コミット差分を対象とする。差分の識別は以下の内容hash、実装handoffの変更範囲（private Path保持・for_root・closureのroot参照2箇所）で固定した。旧scaffoldや旧EXEの結果を今回へ流用していない。

| 対象 | SHA-256 |
| --- | --- |
| src/audio_history.rs | CB47D4078A775CD6FFEC666F306FC4C5B0657FE5B8A7146DD65D76252865AFC0 |
| src/audio_history/closure.rs | 65ACB71BC70CE4D2EC34A0A35242D8496D77D6B2456904AED27D4D503DCA05EE |
| 専用test module、cfg属性からEOFのUTF-8 | 6A2F382ED66A2F661C9D065269D4383F2340F82E3C02E096FE9D3EAA17D40437 |
| tests/audio_history_closure.rs | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | 9D7E8F200475F68ECC7C0964BBFE4EADDE2E6C9F7811DAC7F5A5189CF3690095 |
| target/debug/deps/audio_history_closure-5710f60171e2013d.exe | F08C2300BA3D420663B403514E06FD57C6105F16196308F16D5810E9F6C5A1F4 |

開始時は全指定hash一致。終了時はソース3本・EXE2本の前後不一致0であり、moduleを含むaudio_history.rs全体も不変である。[before](artifacts/history-path-runner-hashes-before.json)、[after](artifacts/history-path-runner-hashes-after.json)。

## 今回のコマンドと結果

リポジトリrootで各実行を一度行った。実行前にWindows tokenのAdministrator=falseを確認した。固定EXEのみscoped sandbox releaseで非管理者実行し、UAC・build・API・実データ・再委任・公開・ユーザーapp停止はない。

| コマンド | exit | passed / failed / ignored / filtered | 実測時間 |
| --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history:: --list` | 0 | 11 tests列挙 | — |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history:: --nocapture --test-threads=1` | 0 | 11 / 0 / 0 / 134 | 0.6334007秒（harness 0.49秒） |
| `target/debug/deps/audio_history_closure-5710f60171e2013d.exe --nocapture` | 0 | 37 / 0 / 1 / 0 | 5.8615258秒（harness 5.81秒） |
| `rustfmt --edition 2021 --config skip_children=true --check src/audio_history.rs src/audio_history/closure.rs` | 0 | check成功 | — |
| `git diff --check` | 0 | check成功 | — |

今回合計48 passed / 0 failed / 1 ignored。134 filteredは指定lib filterによる範囲外であり、成功に算入していない。focused開始/終了 `07:48:54.0061789→07:48:54.6395796`、reader `07:48:54.6926118→07:49:00.5541376`（2026-10-08 +09:00）。

ログは [inventory](artifacts/history-path-runner-list.log)、[focused stdout](artifacts/history-path-runner-focused.stdout.log)／[stderr](artifacts/history-path-runner-focused.stderr.log)、[reader stdout](artifacts/history-path-runner-reader.stdout.log)／[stderr](artifacts/history-path-runner-reader.stderr.log)、[実行metadata・限定残存確認](artifacts/history-path-runner-execution.json)、[fmt](artifacts/history-path-runner-fmt.log)、[diffcheck](artifacts/history-path-runner-diffcheck.log)、[check metadata](artifacts/history-path-runner-checks.json)である。

今回PIDの `ww5-i2-path-35704-*`、`ww5-history-reader-21000-*` をWindows temp直下で限定列挙し、残存はともに0件である。runnerによる追加削除はない。4 junctionケースを含む新11件は末尾assertまで成功した。既存leaf file symlinkの1件は明示ignoredのまま今回未実行であり、I10の未実施項目として引き継ぐ。

## 完了境界

担当検証を完了し所有を親へ返却する。旧writer/recovery EXEは今回実行していない。I2後続の非同期・writer直列化・Progress接合、I10全体 `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/認証、実disk障害・電源断、UI受入は未実施である。本焦点試験はアプリ提供や実機受入の完了を意味しない。
