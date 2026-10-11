# I2a-N 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`を維持した。実行metadata・開始/終了usage counterは未取得。既読AGENTS.md、development-team.md、KPIとwordweave-changeの検証・報告工程を適用した。入力はhistory-normal-save-implementation.md固定表、history-normal-save-tests.md N03c限定訂正、history-normal-save-review.mdと親からの訂正後static PASS・source/build返却である。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分を対象とした。source2・integration4・EXE5の11 SHAは引継ぎ固定値と一致し、実行前後不変。[前hash](artifacts/history-normal-save-runner-before.json) / [後hash](artifacts/history-normal-save-runner-after.json)。store `93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387`、lib EXE `127889DD633931EB4BE66662A16137C4F4EB994550F08921F1D7D050BBF2CE2A`を使用した。[前patch](artifacts/history-normal-save-runner-before.patch) / [後patch](artifacts/history-normal-save-runner-after.patch)のSHA-256はともに `1CD104BC3EBD09EF889177493E6171F7C208482A8E6E674316EC1FED5BBA8DA9`。untracked integrationは全文hashで識別した。

非Administrator tokenを確認し、固定5 EXEだけsandbox外で直接実行した。先に各EXEの`--list`（libは`store:: commit::`も指定）を保存し、exit0・74/3/4/3/16件一致を確認した。[summary](artifacts/history-normal-save-runner-summary.log)と`history-normal-save-runner-*-list.log`を参照する。

repository rootから以下を新規実行した。初回99/1や作者GREENのログを代用していない。

| コマンド | exit | passed / failed / ignored / filtered | 外側実測秒 | 新ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store:: commit:: --nocapture --test-threads=1` | 0 | 74 / 0 / 0 / 128 | 4.13 | [lib](artifacts/history-normal-save-runner-lib.log) |
| `target/debug/deps/audio_history_save-3ab33269b9010ef4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.38 | [save](artifacts/history-normal-save-runner-save.log) |
| `target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe --nocapture --test-threads=1` | 0 | 4 / 0 / 0 / 0 | 0.03 | [progress](artifacts/history-normal-save-runner-progress.log) |
| `target/debug/deps/audio_history_migration-33b6a419850044f4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.68 | [migration](artifacts/history-normal-save-runner-migration.log) |
| `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1` | 0 | 16 / 0 / 0 / 0 | 4.99 | [gate](artifacts/history-normal-save-runner-gate.log) |

計100 passed / 0 failed / 0 ignored（N19＋旧81）。lib対象外128 filteredとexact child内件数は成功へ加算しない。fixture failure/15秒child timeoutなし。N03c新ログは実daily atomic Err、独立locked rename `Some(5)`、read `Some(32)`、product ledger/nonblockの全assert後の同pathでの解除後rename/atomic成功markerとcase成功を記録した。N05全第三状態・postpublish共有拒否・全writer共通block/pure read、N06再入・同root/別rootの実publish区間、M/S/G回帰の固定後段assertionも通過した。旧81には実backup/read共有違反32、8 path、Before/After backup、旧pending3状態を含む。有限scheduleの成功を全scheduleの証明とはしない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs tests/audio_history_save.rs tests/audio_history_progress.rs tests/audio_history_migration.rs tests/audio_history_gate.rs`はexit0。[fmt](artifacts/history-normal-save-runner-fmt.log)。`git diff --check -- src/store.rs src/commit.rs`はexit0。[diff](artifacts/history-normal-save-runner-diff-check.log)。

TEMP直下`ww*` directoryは[前](artifacts/history-normal-save-runner-temp-before.log)0件・[後](artifacts/history-normal-save-runner-temp-after.log)0件、新残存0件。試験自身のcleanup以外の削除はない。[終了status](artifacts/history-normal-save-runner-status-after.log)を保存した。担当報告・新ログ・一意合成fixtureのみ書き込み、source/test/期待値/設定・他者編集は変更していない。UAC/OS/ACL変更・実data/API・公開・再委任なし。

source/build所有を親へ返却する。I2a-N指定独立自動検証は成功。cargo再build、既成功writer/recovery再実行、R/T/V/P・consumer、I10全体`cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/authenticationは今回未実行であり、全体の受入完了とはしない。
