# I2a-M 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`を維持した。実行metadata・開始/終了usage counterは未取得である。既読AGENTS.md、development-team.md、KPI、wordweave-changeの検証・報告工程を適用した。入力はhistory-migration-implementation.md最新固定表、history-migration-tests.mdのM21件RED、history-migration-review.mdの静的阻止0である。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分を対象とした。source2・G/M test節4・integration2・EXE3の計11 SHAは引継ぎ値と一致し、実行前後も一致した。[前hash](artifacts/history-migration-runner-before.json) / [後hash](artifacts/history-migration-runner-after.json)。[前patch](artifacts/history-migration-runner-before.patch) / [後patch](artifacts/history-migration-runner-after.patch)のSHA-256はともに `70029A02D03D13EC8AF9D56390C74B1E4D291069DD53B6047FD8ABB91A11A55F`。untracked integrationは全文hashで識別した。無関係な文書変更は固定コード照合に含めていない。

非Administrator tokenを確認し、repository rootから固定EXEを直接、新規ログへ実行した。実装者の結果は流用していない。

| コマンド | exit | passed / failed / ignored / filtered | 外側実測秒 | ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store:: commit:: --nocapture --test-threads=1` | 0 | 43 / 0 / 0 / 128 | 1.37 | [lib](artifacts/history-migration-runner-lib.log) |
| `target/debug/deps/audio_history_migration-33b6a419850044f4.exe --nocapture --test-threads=1` | 0 | 3 / 0 / 0 / 0 | 0.55 | [migration](artifacts/history-migration-runner-migration.log) |
| `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1` | 0 | 16 / 0 / 0 / 0 | 4.82 | [gate](artifacts/history-migration-runner-gate.log) |

計62 passed / 0 failed / 0 ignored。lib対象外128 filteredは成功へ算入しない。M21（lib18＋integration3）、G24、旧17が成功した。publisher実atomic後Err/readback、After fault後retry、全6 path配置、版拒否表末尾、M05末尾も固定assertionを通過した。lib新ログには既存backup `raw=Some(32)` と新backup `Some((Uncategorized, Some(32)))` があり、今回の実Windows共有違反を確認した。fixture failure/child timeoutなし。これを公開v2受付・S/R完成の証拠とはしない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs tests/audio_history_migration.rs tests/audio_history_gate.rs` はexit0。[fmt](artifacts/history-migration-runner-fmt.log)。`git diff --check -- src/store.rs src/commit.rs` はexit0。[diff check](artifacts/history-migration-runner-diff-check.log)。[summary](artifacts/history-migration-runner-summary.log)に非管理者・exit・時間を記録した。

TEMP直下`ww*` directoryは[前](artifacts/history-migration-runner-temp-before.log)0件・[後](artifacts/history-migration-runner-temp-after.log)0件、新残存0件である。試験自身のcleanup以外の削除は行っていない。[終了status](artifacts/history-migration-runner-status-after.log)を保存した。source/test/期待値/設定/他文書は編集せず、担当報告・新ログ・一意合成fixtureのみ書き込んだ。UAC/OS設定/ACL変更、実data/API、公開、再委任はない。

source/build所有を親へ返却する。今回の指定独立自動検証は成功である。writer37/recovery34は変更影響と独立review判定に従い再実行していない。新build、I10全体`cargo test --all-targets --locked`・`cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/authenticationは未実行であり、後続受入に残る。
