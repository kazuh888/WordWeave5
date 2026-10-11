# I1b-2b recovery 実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。計画指定 implementer `gpt-6-astra/high`。起動後の実行model/effort metadata、開始/終了使用量counterは未取得である。AGENTS.md、development-team、KPI、wordweave-changeを読み、承認済みdesign-i1bのI1b-2契約、writer-design-review、先行writer実装記録を継承した。

## 段階1: 独立RED用scaffold

親の指示に従い公開型と署名だけを追加した。RecoveryView/Candidate/Stateは設計どおりである。recoverは固定Io diagnostic、read_recovery_request/resume/confirm_committed/finishは固定Err(Io)を返す。既存prepare/publish/容量/intent処理は不変であり、現時点で回復動作は未実装である。

変更は `src/audio_history/recovery.rs`（公開value型）、`src/audio_history/store.rs`（5メソッドscaffold）、`src/audio_history.rs`（re-export）である。所有範囲の `mutation.rs` は変更していない。reader、shared DTO、manifest、Progress、UI、独立テストには触れていない。UI変更はないためwordweave-egui-uiは適用外である。

`cargo check --locked --offline --lib` は成功した（51.05秒）。[ログ](artifacts/history-recovery-scaffold-check.log)。sandboxのincremental hardlink警告はcopyへのフォールバックであり、compile失敗ではない。4本のtargeted rustfmt checkとgit diff --checkは成功した。行動テストは独立作者のRED工程へ残す。既存の合格済み行動テストを反復していない。

| path | scaffold SHA-256 |
| --- | --- |
| src/audio_history.rs | 742C76361749F312CDB2FEE98538685BC3E1CC449EED9E145369B4E1142B5D5C |
| src/audio_history/store.rs | 902BE62827818CB47C789DAE283997E2CBD4CE5E2398273E098048B24C4FA554 |
| src/audio_history/recovery.rs | 2B4CD4322FA3148290A8195145E78DBD97EAE59DC4BAFF75A455219E77CFFD9E |
| src/audio_history/mutation.rs（不変） | 8F55A105C3F7A2BD336F215A6E0233772F13DCDEA89C79FC3A5FEE81AD52B4ED |

機能実装は親が独立REDを受領して指示するまで待機する。次段階の設計上の焦点は、missing payloadを補完せず旧baseのmetadata計画を再計算すること、一覧の部分診断、exact mutation_json保持、最新headとreceipt、rebase差分quota、confirm/finish冪等である。最終検証はrunner、受入は利用者に残す。実学習データ、live AI、network、UAC、再委任、Git公開は使用していない。

## 段階2: 独立RED受領後の実装

親から新しい独立32件の行動RED（3 passed / 29 failed、失敗はstub不一致）と本実装指示を受領した。テスト本体・期待値は変更していない。以下が現在の実装であり、上記scaffoldの未実装状態を置き換える。

- `mutation.rs`: 同じmutation適用から読取専用RecoveryPlanを組み立てる。欠損外部leafはOptionのNoneだけで保持し、bytesを捏造しない。公開用ObjectPlanへの変換は全bytesがある場合に限る。通常planは欠損を許容せず、既存result関係predicateを維持する。
- `store.rs`: 保存済みmutation_jsonのexact UTF-8 bytesを保持する復元、旧baseのstrict closureと全計画ref順の再計算照合、全既存objectの照合を追加した。recoverは32件の部分診断付き一覧で、head変動時は候補をConflictへ変更する。resumeはfresh head/receipt/targetを照合し、差分quota計算後に明示rebaseする。同じ計画での再試行では旧intentのexact serialized bytesも維持する。
- `recovery.rs`: 最大100,000件の名前列挙、旧intent全bytesを照合したatomic置換と読戻し、当該intentだけの完了削除を追加した。他operation・object・tempの削除はない。
- `store.rs`のconfirm/finish: 公開rootと最新正本のstrict closure/receiptを照合する。public tokenの値だけを信用せず、finishはtoken.current自体のreceiptと最新receiptの両方を確認する。finish直前fault後にも再確認し、終了後faultでも正本を戻さない。削除済みintentへの再試行は成功する。
- `audio_history.rs`: scaffold時の公開型exportを維持する。I2/Progress/UI/reader/shared DTOは変更していない。

## 焦点検証結果

`cargo test --locked --offline --test audio_history_recovery --test audio_history_store --test audio_history_closure --no-run` はexit 0、2分40秒で完了した。[buildログ](artifacts/history-recovery-build.log)。既知のincremental hardlink警告と既存binのdead-code警告がある。生成された下表の3EXEを直接実行し、重複cargo buildを行っていない。

| 固定EXE | 今回の実行結果 | ログ |
| --- | --- | --- |
| audio_history_recovery-b2b70d25a5608ed7.exe | 32 passed / 0 failed / 0 ignored / 0 filtered、exit 0、3.67秒 | [recovery](artifacts/history-recovery-focused-1.log) |
| audio_history_store-e2ea7a2b1774fcfc.exe | 37 passed / 0 failed / 0 ignored / 0 filtered、exit 0、3.82秒 | [writer回帰](artifacts/history-recovery-writer-regression.log) |
| audio_history_closure-5710f60171e2013d.exe | 37 passed / 0 failed / 1 ignored / 0 filtered、exit 0、5.45秒 | [reader回帰](artifacts/history-recovery-reader-regression.log) |

既知のsandbox FileBlob/hardlink/junction制約について親承認済みのscoped sandbox releaseを使い、上記EXEだけを非管理者で実行した。UAC/Windows設定変更はない。readerの `windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target` は今回実行しておらず、過去の成功を今回へ合算しない。4本のtargeted rustfmt checkとgit diff --checkはexit 0である。

| 現在のsource/test | SHA-256 |
| --- | --- |
| src/audio_history.rs | 742C76361749F312CDB2FEE98538685BC3E1CC449EED9E145369B4E1142B5D5C |
| src/audio_history/store.rs | ACD8C9938DD08AFB32239BE7BB875A4F1B1A0ADF9E9E5DCF7771A0780C4F9E59 |
| src/audio_history/recovery.rs | 7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012 |
| src/audio_history/mutation.rs | 7D65497A2455519B21166A485CC98513B1A7425AA41ED472F87041C46242A81E |
| tests/audio_history_recovery.rs（独立作者、本担当未変更） | 120C9F31BF66ECD35A9C4E0B2D4FA42907F2D6E5093F80418A519D9CCC08909F |
| tests/audio_history_store.rs（本担当未変更） | 7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4 |
| tests/audio_history_closure.rs（本担当未変更） | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 |

実行EXEのSHA-256はrecovery `44FFE5D34F46397D2C9620D2498DC1D21DD3D31D364CDDCA9C4703E74901B1F2`、writer `51A497D3500131E79BDC7F534CA382B63F2D813676909B3504C6B0DB24D69E1F`、reader `3AA9B70DABE6C8B5F457677B97E27604C7714B689E4D21834D3A7C2C21A79EBF` である。

runnerへ渡す絶対pathは以下の3本である。

```text
F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe
F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe
F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_closure-5710f60171e2013d.exe
```

本体4本とbuild所有を親へ返却する。未完了は独立レビュー/runner、I2の最新Progress headと全writer直列化/commitpoint接合である。本担当の焦点検証を最終受入とは扱わない。32件に明示ケースのない列挙100,000件上限、resumeの512KiB exact境界、実disk不足/電源断耐久性は今回の成功数から実測済みとしない。開始/終了使用量counterと実行model metadataは引き続き未取得である。
