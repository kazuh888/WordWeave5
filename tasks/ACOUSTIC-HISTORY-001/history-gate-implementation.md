# I2a-G metadata gate 実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。計画指定 `ww-implementer / gpt-6-astra / high` を維持する。実行metadata、開始/終了使用量counterは未取得である。既読のAGENTS.md、development-team、wordweave-changeに基づき担当範囲だけを扱った。

親からI2a-0 Post PASS・I2a-G Pre PASSを受領した。今回の入力はimplementation.md I2a-Gとdesign-i2.md §3/4のG限定接合であり、設計SHA `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA` と一致した。

段階は独立RED前の最小scaffoldである。変更は以下の新型・新入口だけであり、すべてcrate内可視性、guard fieldはprivateである。

- `src/store.rs`: Debug/PartialEq/EqのProgressGateError、opaque ProgressWriteGuard、固定Err(Poisoned)のacquire_progress_write/ensure_writable、no-opのblock_writes。
- `src/commit.rs`: Debug/PartialEq/EqのPendingCheckError、固定Err(Io)のpending_checked、固定文字列Errのrecover_under_guard。

既存load/save/pending/recover、HistoryWriter、Default/version/migration、codec、テストを変更していない。Mutex取得・block保持・strict pending・既存writerへの接合はまだ未実装である。新口は無副作用であり、既存入口をstubへ置換していない。

`rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs` はexit 0。親指示によりcargo build/testは実行せず、compile・動作は未検証である。

| scaffold source | SHA-256 |
| --- | --- |
| src/store.rs | F3D8C6546E4122B798F74586ECC51A3BDB6D37F457E86B0899A92F5A8A764B0A |
| src/commit.rs | 2AC0132E87F59F913F189C13F7B20895AFC4DDB9204C9223B7F455F448839A15 |

source/build所有を親へ返却する。独立作者のcompile・REDを受領した後、別指示で本実装へ進む。既存dirtyを保持し、実データ・API・公開・UAC・再委任は使用していない。

## 本実装（独立24件RED受領後）

親から新24件の4 PASS / 20行動REDと本実装指示を受領した。指定3本の本体だけを変更した。

- `store.rs`: process単一Mutex、MutexGuardをprivate fieldに持つ!Send guard、同thread held印による待機前Reentrant拒否、guard下だけで操作する共通blockフラグを実装した。panic poisonを解除せず、通常drop/失敗時にthread所有印を解除する。load/save_with_limitは一度取得し、副作用終了まで保持する。
- `commit.rs`: root・全祖先・pending leafの型/reparseを検査し、正常親下のleaf NotFoundだけfalseとする。互換boolはエラーをtrueへ倒す。save/recoverはtyped検査を用いる。material save/loadからの回復は同じguardを引き継ぎ、under_guard入口でもblockを再検査する。
- `audio_history/store.rs`: prepare/publish/resume/finishがそれぞれ一度取得する。型からReentrant→Busy、Poisoned/WriteBlocked→Ioへ変換する。confirm/head/reader/atomic_writeは取得しない。

Default/schema/codec/移行/ProgressFaults/結果不明検出、recovery.rs、UIは変更していない。新規・既存テスト期待値も変更していない。現source3本は親が独立静的レビュー（阻止0）に使用したhashと一致する。

| source | 完了時SHA-256 |
| --- | --- |
| src/store.rs | CC8C39E13EB56591DBFF698FCF2E05D5803B8DB0FD7710E661E8BEE120103154 |
| src/commit.rs | 78C14B81B7801590CDDF082EC06F7BA5F0DD5A06CFCD13AB0BB5E46E3399F78E |
| src/audio_history/store.rs | 7532A07DA9BED7F79AE0B1E28806CFE881BE3B2AC5973E73F39F11F44B51FC67 |

固定されたテストの前後SHAは一致する。store新module `2AB39F8D08B1E2265180B9A948A868EC0AE8E279E798F7CDD2418F0AA82BC73C`、commit新module `9A1E7F3B7497D43A1F9426538990E6E7642ED034C04565F2B7184DA801686205`、gate integration `5A9706DCDB41156119D4CB014692E5FF7748865950A254A32F0C7D1093B86DCC`、writer `7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4`、recovery `AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567` である。

### 実装者の焦点検証

`cargo test --locked --lib --test audio_history_gate --test audio_history_store --test audio_history_recovery --no-run` を1回実行し、exit 0、3分39秒でcompileした。[buildログ](artifacts/history-gate-implementation-build.log)。既知のhardlink/copy警告、既存binのdead-code警告と後続用block_writesの未使用警告を保持した。

以降は固定EXEを直接実行した。実行前にAdministrator tokenでないことを検査し、合成fixtureの必要なEXEだけscoped sandbox releaseで実行した。UAC/RunAs/OS設定変更はない。

| 対象と引数 | 結果 | ログ |
| --- | --- | --- |
| gate integration `--nocapture --test-threads=1` | 16 passed / 0 failed / 0 ignored / 0 filtered、exit 0、8.58秒 | [gate](artifacts/history-gate-implementation-focused.log) |
| lib `store:: commit:: --nocapture --test-threads=1` | 25 passed / 0 failed / 0 ignored / 128 filtered、exit 0、1.13秒（新8＋既存17） | [lib](artifacts/history-gate-implementation-lib.log) |
| writer、引数なし | 37 passed / 0 failed / 0 ignored / 0 filtered、exit 0、6.95秒 | [writer](artifacts/history-gate-implementation-writer.log) |

新24件は全件GREENであり、RED時に未到達だったreparse・block・poisonの末尾assertionも通過した。child timeout/fixture failureはない。targeted rustfmt checkとgit diff --checkもexit 0である。recovery34件は次の完了追記を参照する。

| 固定EXEの絶対path | SHA-256 |
| --- | --- |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | 7BBE1A40659585BCD2D1825014C82D11AA0217A85CB02DF5ADB5380BDC7C8D50 |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe | 21094200A42CDAA7F16A6D5D75A5673D637C34B2D221F1B8CA62625DB7BC7D9C |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe | 41F7ACDA73F715A5062328D46C42B09B211545313B64D78CEC9E2FAF268261E3 |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe | 10F0BA51154BB1D82E2D2A9252170A5C84666721E07675BD693FB24F95CF6779 |

recoveryは **34 passed / 0 failed / 0 ignored / 0 filtered、exit 0、116.73秒**で完了した。[recoveryログ](artifacts/history-gate-implementation-recovery.log)。100,000件列挙/100,001件拒否とresume exact intent上限も含み、既存期待値を維持している。これで依頼された実装者の新24件・既存store/commit/writer/recovery確認が完了した。

source/build所有を親へ返却する。sourceは独立静的review対象から不変である。未完了は独立runnerと最終受入、後続I2接合である。process内gateによる直列化は、別handleの規約外書込や電源断耐久性まで保証しない。I10全体test/release、実学習data・live API、UAC、公開は実行していない。使用量counter・実行model metadataは引き続き未取得である。
