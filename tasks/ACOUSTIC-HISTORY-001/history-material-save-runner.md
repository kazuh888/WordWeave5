# I2a-T 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`。実行metadata・開始/終了usage counterは未取得。既読AGENTS.md、development-team.md、KPI、wordweave-changeの検証・報告工程を適用した。入力はhistory-material-save-{tests,review,implementation}.mdと親の所有返却、固定implementation-final/exes-before.logである。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分。source2/test5/EXE6の13全文SHAとcommit全test suffix/T moduleの計15 hashは固定値と一致し、前後不変。[前hash](artifacts/history-material-save-runner-before.json) / [後hash](artifacts/history-material-save-runner-after.json)。commit `14952768…188E0154`、store `38D4516D…F312EF`、test suffix `DAD7F863…0B92307A`、T module `54F55EDE…E12CFE`を使用した。[前patch](artifacts/history-material-save-runner-before.patch) / [後patch](artifacts/history-material-save-runner-after.patch)はともにSHA-256 `33BCC4E07F931799EF08AB62A888088F30F244DCC109759B6A387EA410DF20CE`。untracked testsは全文hashで識別した。

非Administrator tokenを確認し、固定6 EXEだけsandbox外で直接実行した。先に全9群の同filter＋`--list`を実行し、exit0・件数一致を確認した。`history-material-save-runner-*-list.log`と[summary](artifacts/history-material-save-runner-summary.log)に記録した。新規実行区間は2026-10-08 15:01:02.7682255–15:01:39.8699622 JSTである。旧成功logは代用していない。

repository rootから実行した。下表の全行へ共通引数 `--nocapture --test-threads=1` を追加した。`lib`は`target/debug/deps/wordweave5-b3388a7bbef44bb4.exe`、`material`は`target/debug/deps/audio_history_material-d89f83041366a9a0.exe`である。M1のexact名は `commit::i2_migration_tests::m05_v2_material_private_entry_matches_public_save_with_exact_migration`。

| 群 / EXE・filter | exit | passed / failed / ignored / filtered | 外側実測秒 | 新ログ |
| --- | --- | --- | --- | --- |
| old-lib / lib `store:: commit:: --skip i2_material_save_tests --skip commit::i2_migration_tests::m05_v2_material_private_entry_matches_public_save_with_exact_migration` | 0 | 95 / 0 / 0 / 147 | 16.21 | [old-lib](artifacts/history-material-save-runner-old-lib.log) |
| m1 / lib 上記M1名 `--exact` | 0 | 1 / 0 / 0 / 241 | 0.48 | [m1](artifacts/history-material-save-runner-m1.log) |
| t-lib / lib `commit::i2_material_save_tests::` | 0 | 18 / 0 / 0 / 224 | 8.36 | [t-lib](artifacts/history-material-save-runner-t-lib.log) |
| old-material / material `--skip t_material_save` | 0 | 3 / 0 / 0 / 3 | 2.12 | [old-material](artifacts/history-material-save-runner-old-material.log) |
| t-material / material `t_material_save` | 0 | 3 / 0 / 0 / 3 | 1.19 | [t-material](artifacts/history-material-save-runner-t-material.log) |
| save / `target/debug/deps/audio_history_save-3ab33269b9010ef4.exe` | 0 | 3 / 0 / 0 / 0 | 0.66 | [save](artifacts/history-material-save-runner-save.log) |
| progress / `target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe` | 0 | 4 / 0 / 0 / 0 | 0.06 | [progress](artifacts/history-material-save-runner-progress.log) |
| migration / `target/debug/deps/audio_history_migration-33b6a419850044f4.exe` | 0 | 3 / 0 / 0 / 0 | 0.79 | [migration](artifacts/history-material-save-runner-migration.log) |
| gate / `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe` | 0 | 16 / 0 / 0 / 0 | 6.48 | [gate](artifacts/history-material-save-runner-gate.log) |

計146 passed / 0 failed / 0 ignored（旧124＋正式M1＋新T21）、重複加算なし。filteredとexact child内部件数は成功へ含めない。fixture failure/15秒child timeoutなし。M1の厳密12checkpointと末尾backup/媒体、T01全候補・全field/ledger、T02全上限/closure/path、T03全12/8checkpoint・外部変更、T04全publish結果/unknown child、T05全R checkpoint/retry・R-M2回目・unknown block、T06再入/実区間非重複/保存先非混入の後段assertionが通過した。

t-lib新ログにはold source progress.json/custom.tsvの実read32、pending read32 mode1/5、`T pending atomic failure and unlocked same-path success verified after block/ledger assertions`を記録した。実atomic失敗・block/ledger完了後の解除対照を、合成Errや旧R/N結果で代替していない。R Ok→T最終readback間は決定fault未注入、normal競合は既定の合成証拠であり、新normal publisher区間観測を実施したとはしない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs tests/audio_history_material.rs tests/audio_history_save.rs tests/audio_history_progress.rs tests/audio_history_migration.rs tests/audio_history_gate.rs`はexit0。[fmt](artifacts/history-material-save-runner-fmt.log)。`git diff --check -- src/store.rs src/commit.rs`はexit0。[diff](artifacts/history-material-save-runner-diff-check.log)。TEMP直下`ww*` directoryは[前](artifacts/history-material-save-runner-temp-before.log)0件・[後](artifacts/history-material-save-runner-temp-after.log)0件、新残存0件。試験自身のcleanup以外の削除なし。[終了status](artifacts/history-material-save-runner-status-after.log)を保存した。

担当報告・新ログ・一意合成fixtureのみ書込み、source/test/期待値/設定・他者編集は変更していない。実data/API/UAC/OS/ACL変更・公開・再委任なし。source/build所有を親へ返却する。指定独立自動検証は成功である。追加build、V/P・consumer、I10全体`cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、native microphone/pen/TTS/IME/authentication・電源断受入は今回未実行であり、全体受入完了とはしない。
