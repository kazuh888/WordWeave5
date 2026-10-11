# I2a-R 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`。実行metadata・開始/終了usage counterは未取得。AGENTS.md、development-team.md、KPIとwordweave-changeの検証・報告工程を適用した。入力はhistory-material-recovery-{design,tests,review,implementation}.md、親のsource/build返却、implementation-final.logとexes-before.logの固定対象である。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分。source2/test5/EXE6の13全文SHAとstore/commit全test suffix・R moduleの計16 hashは引継ぎ値と一致し、前後不変。[前hash](artifacts/history-material-recovery-runner-before.json) / [後hash](artifacts/history-material-recovery-runner-after.json)。store `38D4516D…F312EF`、commit `24414D96…79C6A6`、R module `B26C43DB…3B1BDB`、material test `5AC9BBC6…3A0683`を使用した。[前patch](artifacts/history-material-recovery-runner-before.patch) / [後patch](artifacts/history-material-recovery-runner-after.patch)はともにSHA-256 `0CEE330769DC32C50235BA010FBAC6217D003CCE2EC2514A47AB2A5FD1B6DA53`。untracked testsは全文hashで識別した。

非Administrator tokenを確認し、固定6 EXEだけsandbox外で直接実行した。最初の`--list`は全exit0、lib filter込み96/material3/save3/progress4/migration3/gate16件が一致した。新`history-material-recovery-runner-*-list.log`と[summary](artifacts/history-material-recovery-runner-summary.log)に保存した。

repository rootからの新規独立実行であり、実装者や旧ログの結果を代用していない。

| コマンド | exit | passed / failed / ignored / filtered | 外側実測秒 | 新ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store:: commit:: --nocapture --test-threads=1` | 0 | 96 / 0 / 0 / 128 | 11.01 | [lib](artifacts/history-material-recovery-runner-lib.log) |
| `target/debug/deps/audio_history_material-d89f83041366a9a0.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 1.58 | [material](artifacts/history-material-recovery-runner-material.log) |
| `target/debug/deps/audio_history_save-3ab33269b9010ef4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.39 | [save](artifacts/history-material-recovery-runner-save.log) |
| `target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe --nocapture --test-threads=1` | 0 | 4 / 0 / 0 / 0 | 0.03 | [progress](artifacts/history-material-recovery-runner-progress.log) |
| `target/debug/deps/audio_history_migration-33b6a419850044f4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.70 | [migration](artifacts/history-material-recovery-runner-migration.log) |
| `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1` | 0 | 16 / 0 / 0 / 0 | 4.83 | [gate](artifacts/history-material-recovery-runner-gate.log) |

計125 passed / 0 failed / 0 ignored（新R25＋旧100）。lib対象外128 filtered・exact child内部件数は成功へ加算しない。fixture failure/15秒child timeoutなし。新RのRED未到達・末尾assertionは全通過した。duplicate補強、同hash異bytes拒否、M後pending unknown/block、R-M書込前hash・current既new、実read32と解除、known retry全行、unknown children全行、rename Err後実状態判定・解除対照、finish exactを含む。lib新ログに`R archive real rename denial raw=Some(32)`と`R archive postrename read denied 32; same file readable after release`を記録した。旧N03c・M/S/Gを含む100件も維持した。

時刻取得不能・初回archive選定名衝突は独立静的reviewだけであり、今回も実測未実施である。有限scheduleや合成fixtureを全schedule・電源断耐久性の証明とはしない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs tests/audio_history_material.rs tests/audio_history_save.rs tests/audio_history_progress.rs tests/audio_history_migration.rs tests/audio_history_gate.rs`はexit0。[fmt](artifacts/history-material-recovery-runner-fmt.log)。`git diff --check -- src/store.rs src/commit.rs`はexit0。[diff](artifacts/history-material-recovery-runner-diff-check.log)。

TEMP直下`ww*` directoryは[前](artifacts/history-material-recovery-runner-temp-before.log)0件・[後](artifacts/history-material-recovery-runner-temp-after.log)0件、新残存0件。試験自身のcleanup以外の削除なし。[終了status](artifacts/history-material-recovery-runner-status-after.log)を保存した。担当報告・新ログ・一意合成fixtureのみ書き込み、source/test/期待値/build環境・他者編集は変更していない。実data/API/UAC/OS/ACL変更・公開・再委任なし。

source/build所有を親へ返却する。I2a-R指定独立自動検証は成功。追加cargo、既成功writer/recovery再実行、T/V/P・consumer、I10全体`cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/authenticationは今回未実行であり、全体受入完了とはしない。
