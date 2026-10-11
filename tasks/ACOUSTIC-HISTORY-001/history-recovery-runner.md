# I1b-2b 独立runner結果

2026-10-08 / ACOUSTIC-HISTORY-001 / `/root/history_recovery_runner`。計画指定 `gpt-6.1-sol/medium`。返却runtime model/effort metadata、開始/終了usage counter、親子usage包含関係は未取得である。所有は本書と `artifacts/history-recovery-runner-*` のみであり、本体・テスト・期待値・設定を変更していない。

AGENTS.md、development-team.md、KPI、wordweave-changeを読み、同Skillの保存/回復に対する隔離データ検証・独立検証・実行証拠報告を適用した。入力は親の承認済みI1b-2b handoff、history-recovery-implementation.md stage2、history-recovery-tests.md、history-recovery-boundary-packet.mdである。受入範囲はAH-AC04/08/14に対応するwriter/recoveryの焦点検証であり、Windowsアプリ提供ではない。

## 対象固定

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`、branch `codex/development-agent-team`。開始時dirtyは保持した。追跡済み差分のスナップショットは [tracked-diff](artifacts/history-recovery-runner-tracked-diff.patch)、SHA-256 `CCEC01A915C75B4BA2697ED20453B1D803A5626C8851508912D2BBE0766A21B4`。untracked本体/テストは下表の内容hashで識別する。リポジトリ全体の既存変更をこのrunnerによる変更と扱わない。

| 対象 | SHA-256 |
| --- | --- |
| src/audio_history.rs | 742C76361749F312CDB2FEE98538685BC3E1CC449EED9E145369B4E1142B5D5C |
| src/audio_history/store.rs | ACD8C9938DD08AFB32239BE7BB875A4F1B1A0ADF9E9E5DCF7771A0780C4F9E59 |
| src/audio_history/recovery.rs | 7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012 |
| src/audio_history/mutation.rs | 7D65497A2455519B21166A485CC98513B1A7425AA41ED472F87041C46242A81E |
| tests/audio_history_recovery.rs | AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567 |
| tests/audio_history_store.rs | 7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4 |
| tests/audio_history_closure.rs | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 |
| recovery EXE | B53CFE52631DA229C015498916F168BFDABC1008EE2CAD4492B2FB17363A208F |
| writer EXE | 51A497D3500131E79BDC7F534CA382B63F2D813676909B3504C6B0DB24D69E1F |
| reader EXE | 3AA9B70DABE6C8B5F457677B97E27604C7714B689E4D21834D3A7C2C21A79EBF |

各EXEは `target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe`、`audio_history_store-e2ea7a2b1774fcfc.exe`、`audio_history_closure-5710f60171e2013d.exe` である。初回照合は全一致、recovery開始直前のsource/EXE再照合も一致した。[before hash](artifacts/history-recovery-runner-hashes-before.json)。`--list` は3EXEともexit 0、recovery34/writer37/reader38（1 ignoredを含む）を確認した。各listは同prefixの `recovery-list.log`、`writer-list.log`、`reader-list.log` に保存した。

## 今回の実行

各コマンドはリポジトリrootで実行した。既知のsandbox filesystem制約に対し、親が承認した固定EXEのみscoped `require_escalated` で非管理者実行した。UAC、OS設定変更、build、実学習データ、実API、再委任、公開、ユーザーアプリ停止は行っていない。

| コマンド | exit | passed / failed / ignored | 実測時間 | 今回ログ |
| --- | --- | --- | --- | --- |
| `target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe --nocapture` | 0 | 37 / 0 / 0 | 4.2393103秒 | [writer](artifacts/history-recovery-runner-writer.log)、[metadata](artifacts/history-recovery-runner-writer-meta.json) |
| `target/debug/deps/audio_history_closure-5710f60171e2013d.exe --nocapture` | 0 | 37 / 0 / 1 | 5.3054588秒 | [reader](artifacts/history-recovery-runner-reader.log)、[metadata](artifacts/history-recovery-runner-reader-meta.json) |
| `target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe --nocapture` | 0 | 34 / 0 / 0 | 102.1191682秒 | [stdout](artifacts/history-recovery-runner-recovery.stdout.log)、[stderr](artifacts/history-recovery-runner-recovery.stderr.log)、[metadata](artifacts/history-recovery-runner-recovery-meta.json) |
| `rustfmt --edition 2021 --check tests/audio_history_recovery.rs` | 0 | check成功 | — | [fmt](artifacts/history-recovery-runner-fmt.log) |
| `git diff --check` | 0 | check成功 | — | [diffcheck](artifacts/history-recovery-runner-diffcheck.log)、[static metadata](artifacts/history-recovery-runner-static-meta.json) |

writer/readerは今回一度だけ実行した。recoveryは親の作者実行終了・所有返却通知を受けて実行開始した。20分watchdogは外部打切り条件であり、性能の合格assertではない。権限依存の `windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target` は今回未実行であり、過去の結果を合算していない。

開始/終了時刻はwriter `07:31:30.2867956→07:31:34.5261059`、reader `07:31:44.3508313→07:31:49.6562901`、recovery `07:32:22.4738617→07:34:04.5930299`（2026-10-08 +09:00）である。全3run合計108 passed / 0 failed / 1 ignored / 0 filtered。watchdogは発火していない。ソース上の `#[test]` も34個である。

R1は手製wire oracleの524,287/524,288/524,289 bytes境界と許可/拒否後の保存状態を検証した。R2は実100,000件を34.3786375秒で生成し、先頭pageを54.3830435秒、末尾pageを63.2506075秒、100,001件のLimitExceededと無書込を80.4274572秒で確認した。100.4524602秒で `R2 cleanup complete` が出力された。値はこの実行の経過観測であり、性能基準ではない。

cleanupはtest内の `drop(f)` 後の不在assert成功に加え、raw stderrに出たR1の3root・R2の1rootすべてへ `Test-Path -LiteralPath` を実行し、4件ともfalseを確認した。runnerによる追加削除はない。[最終チェック](artifacts/history-recovery-runner-final-checks.json)。終了時のソース7本・EXE3本はすべて開始hashと一致し、差分検出0である。[after hash](artifacts/history-recovery-runner-hashes-after.json)。検証を通すための変更・期待値緩和はなく、担当成果物の所有を親へ返却する。

## 残る受入

本結果でI2の最新Progress head接合・全writer直列化、実disk不足/電源断耐久性、Windows native microphone/pen/TTS/IME、実認証/実Qwen、音響解析品質、UI受入を合格とはしない。Windows提供時の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` は今回の親指定「build禁止」の焦点runner範囲外であり、後続提供gateに残る。旧logとmockを今回の実機結果へ転用していない。
