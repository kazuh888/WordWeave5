# I2a-G 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001 / history_gate_runner。計画指定 `gpt-6.1-sol / medium`（implementation.md I2割当・I2a再分割）である。実行model/effort metadata、開始/終了usage counterは未取得であり、推定しない。AGENTS.md、development-team.md、KPI、wordweave-changeの検証・報告工程を読了した。

## 対象固定と受入

親からsource/build所有返却、独立静的review阻止0、独立24件REDの引継ぎを受領した。受入対象はI2a-G / I2-T04のmetadata/history直列化、再入拒否、block/poison維持、strict pendingと互換boolのfail-closed、および既存store/commit/writer/recovery回帰である。実装者の実行結果を独立結果に流用していない。

HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。未commit差分を含むためHEADだけで対象を識別しない。source3本・新test suffix2件・integration3本・EXE4本の全12 SHAを実行前後に照合し、history-gate-implementation.mdの固定値とすべて一致した。[前](artifacts/history-gate-runner-before.json) / [後](artifacts/history-gate-runner-after.json)。固定対象のtracked diffを保存した[前patch](artifacts/history-gate-runner-diff.patch) / [後patch](artifacts/history-gate-runner-diff-after.patch)はともにSHA-256 `32996A1BF843E68F4D384970EC1B59DA4B16D458AF56AFCED499ADD335A79D9E` である。untracked integration/sourceの識別は前後JSONの全文SHAによる。

## 新規実行

作業directoryはrepository root、以下は固定EXEを直接実行したコマンドである。全実行でAdministrator tokenを事前拒否し、非管理者sandbox外で実行した。UAC/RunAs/ACL/OS設定変更はない。

| コマンド | exit | passed / failed / ignored / filtered | 実測秒 | 新ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store:: commit:: --nocapture --test-threads=1` | 0 | 25 / 0 / 0 / 128 | 1.03 | [lib](artifacts/history-gate-runner-lib.log) |
| `target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe --nocapture --test-threads=1` | 0 | 16 / 0 / 0 / 0 | 5.02 | [gate](artifacts/history-gate-runner-gate.log) |
| `target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe --nocapture --test-threads=1` | 0 | 37 / 0 / 0 / 0 | 6.49 | [writer](artifacts/history-gate-runner-writer.log) |
| `target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe --nocapture --test-threads=1` | 0 | 34 / 0 / 0 / 0 | 106.97 | [recovery](artifacts/history-gate-runner-recovery.log) |

集計112 passed / 0 failed / 0 ignored、lib対象外128 filteredである。filteredを成功件数へ含めない。新24件（lib8＋gate16）はすべてGREENであり、RED時に未到達だったblock後のunder_guard・8入口拒否/bytes保持、poison後の再試行拒否、reparse ancestor/root/leaf拒否の後段assertionも含む。子processのexact1件・成功marker・15秒deadlineは固定期待値のまま通過した。fixture failure/timeoutはない。recoveryは実際の100,000件境界、100,001件拒否を含む。時間は外側Stopwatch値、libtest内部時間は各新ログにある。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs src/audio_history/store.rs tests/audio_history_gate.rs tests/audio_history_store.rs tests/audio_history_recovery.rs` はexit0、0.49秒。[fmt](artifacts/history-gate-runner-fmt.log)。`git diff --check` はexit0。[diff check](artifacts/history-gate-runner-diff-check.log)。再build、全体cargo test、release buildは実行していない。本工程は親の指定どおり固定済み対象のI2a-G検証であり、Windowsアプリ納品のI10ではない。

## 清掃・境界・返却

TEMP直下の`ww*` directoryを[実行前](artifacts/history-gate-runner-temp-before.log)と[実行後](artifacts/history-gate-runner-temp-after.log)で確認した。ともに0件、新規残存0件である。各試験の生成物は試験自身のcleanupで回収された。既存・不明所有のTEMPは削除していない。[終了status](artifacts/history-gate-runner-status-after.log)を保存した。

書込みは本報告と新規`artifacts/history-gate-runner-*`ログ、およびテストの一意な合成fixtureのみである。source、test、期待値、設定、設計、既存成果物を変更していない。公開、実API、実学習data、録音、秘密情報には接触していない。

I2a-Gの指定自動検証は成功である。source/build所有を親へ返却する。最終Post判定は親へ委ねる。I10 `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、後続I2接合、native microphone/pen/TTS/IME/authenticationは未実行であり、本結果で受入済みとしない。process内gate・有限schedule・合成fixtureの結果は、規約外handle・電源断耐久性・全scheduleの証明ではない。
