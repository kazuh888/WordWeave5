# I2a-S 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`を維持した。返却runtime metadata・開始/終了usage counterは未取得。既読AGENTS.md、development-team.md、KPIとwordweave-changeの検証・報告工程を適用した。入力はhistory-progress-implementation.md本実装固定表、history-progress-tests.md限定oracle訂正、新progress EXE、history-progress-review.md静的阻止0・訂正承認である。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分が対象である。source2・integration3・EXE4・store全test suffixの10 SHAが引継ぎ固定値と一致し、前後不変である。[前hash](artifacts/history-progress-runner-before.json) / [後hash](artifacts/history-progress-runner-after.json)。store `4C1287B2…00EEE`、commit `C1E7D086…022AD`、訂正progress test `81493C07…790A`、新progress EXE `065D050E…524E9`を使用した。[前patch](artifacts/history-progress-runner-before.patch) / [後patch](artifacts/history-progress-runner-after.patch)はともにSHA-256 `6DFA9FF47964AD248ADF121E3D7A9E05F5DE27C0A4776445C0DCB75E35D2D14D`。untracked integrationは全文hashで識別した。

非Administrator tokenを確認し、必要な固定4 EXEだけsandbox外で直接実行した。各EXEへ先に`--list`を渡し、libは`store:: commit::`を併用した。全list exit0、対象件数58/4/3/16が一致した。[lib一覧](artifacts/history-progress-runner-lib-list.log) / [progress一覧](artifacts/history-progress-runner-progress-list.log) / [migration一覧](artifacts/history-progress-runner-migration-list.log) / [gate一覧](artifacts/history-progress-runner-gate-list.log)。

repository rootからの新規実行であり、旧成功ログは使用していない。

| コマンド | exit | passed / failed / ignored / filtered | 外側実測秒 | 新ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store:: commit:: --nocapture --test-threads=1` | 0 | 58 / 0 / 0 / 128 | 1.86 | [lib](artifacts/history-progress-runner-lib.log) |
| `target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe --nocapture --test-threads=1` | 0 | 4 / 0 / 0 / 0 | 0.03 | [progress](artifacts/history-progress-runner-progress.log) |
| `target/debug/deps/audio_history_migration-33b6a419850044f4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.50 | [migration](artifacts/history-progress-runner-migration.log) |
| `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1` | 0 | 16 / 0 / 0 / 0 | 4.81 | [gate](artifacts/history-progress-runner-gate.log) |

計81 passed / 0 failed / 0 ignored（S15＋改訂M25＋G24＋旧17）。lib対象外128 filteredと子process内件数は成功へ加算しない。RED未到達のtyped正例・拒否表末尾、全8 strict path配置、write-block子のpure read/head/load/ledger・15秒deadline、Before/After backup境界、旧pending 3中断状態、progress訂正case末尾runtime1拒否を含む固定assertionが通過した。fixture failure/timeoutなし。libログには既存backup `raw=Some(32)`、新backup `Some((Uncategorized, Some(32)))`、S read `actual read denied with Some(32)`があり、今回の実共有違反を確認した。open後NotFound→Ioは独立静的reviewの証拠であり、実レース再現済みとは扱わない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs tests/audio_history_progress.rs tests/audio_history_migration.rs tests/audio_history_gate.rs`はexit0。[fmt](artifacts/history-progress-runner-fmt.log)。`git diff --check -- src/store.rs src/commit.rs`はexit0。[diff](artifacts/history-progress-runner-diff-check.log)。[summary](artifacts/history-progress-runner-summary.log)に一覧件数・非管理者・exit・時間を保存した。

TEMP直下`ww*` directoryは[前](artifacts/history-progress-runner-temp-before.log)0件・[後](artifacts/history-progress-runner-temp-after.log)0件、新残存0件。fixture自身のcleanup以外の削除は行っていない。[終了status](artifacts/history-progress-runner-status-after.log)を保存した。書込みは本報告、新規担当ログ、一意合成fixtureのみである。source/test/期待値/設定・他者編集は変更せず、UAC/OS/ACL変更・実data/API・公開・再委任は行っていない。

source/build所有を親へ返却する。I2a-S指定独立自動検証は成功である。cargo再build、既成功writer/recovery再実行、N/R、I10全体`cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/authenticationは今回未実行であり、全体の受入完了とはしない。
