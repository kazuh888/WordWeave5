# I1b-W1 独立test author引継ぎ

2026-10-07 / ACOUSTIC-HISTORY-001。計画 `implementation.md` I1 の指定は `gpt-6-astra` / `high`、役割はtest authorである。実行model/effort metadata、開始・終了の使用量counterは未取得。着手後最初の取得時刻は22:09:50 JST、検証後の取得時刻は22:17:51 JSTである（最終応答前まで）。読了はAGENTS.md、development-team.md、KPI、agent-skills:test-driven-development全文である。完了境界は試験追加・compile・新2件focused検証と所有返却であり、本体修正・全37件の独立runner実行・Post-gateは含まない。

## 承認入力と所有

- `history-w1-packet.md` SHA256 `1613BE0A2308583F20495AD37A3E5425004EEDB5A2A2F886330157F9985E093C`。`m2-w1-gates.md` の独立Pre PASSを確認した。
- 正本は `design-i1b.md` W06 / AH-AC04/14 と `history-writer-review.md` I1b-W1。指摘は受入証拠不足であり、本体欠陥の確定ではない。
- 所有変更は `tests/audio_history_store.rs` 末尾の `recovery_junction` module（2試験とprivate fixture）と本書だけである。本体・manifest・権限・期待値の変更はない。
- 旧test SHA256 `5D097873DF335BA6D152CF7522879D5870A43C5E44E5B443A748A9545C89098A`。保存snapshot先頭58,832 bytesが新testのprefixと完全一致することを比較した。既存35件は不変である。

## ケースとoracle

| 受入ID | 新test（module `recovery_junction`） | 期待結果 |
| --- | --- | --- |
| I1b-W1 / W06 / AH-AC04/14 | `w06_prepare_rejects_recovery_junction_without_changing_any_bytes` | 正常constructor後、prepare前にrecovery junctionを作成。prepareを2回試行し各回UnsafePath。既存object集合/bytes、原音、手製の既存intent、リンク先全bytes、head不変 |
| I1b-W1 / W06 / AH-AC04/14 | `w06_same_writer_publish_rejects_recovery_junction_and_preserves_prepared_intent` | 正常prepareがReadyと文書仕様どおりのintentを生成。recoveryをprivate siblingへ移しjunction化。同writer/Storage/Preparedでpublishを2回試行し各回UnsafePath。追加object 0、intent全bytes・原音・全履歴・リンク先・head不変 |

既存履歴はwriterを使わず手製Trial/Root/PCM16 WAVで構築し、reader closureで正常性を確認する。intent期待値はreflection→revision+1/previous→receipt込みrootの仕様から独立計算する。case①の既存intentは別operation、case②は生成intent JSONをこの独立oracleと完全比較した後に元bytesを固定する。未知内容を拒否するだけのfixtureにしない。

比較は `BTreeMap<relative path, Vec<u8>>` の完全一致であり、件数/hashだけに依存しない。history配下とprivate target配下を各API呼出し前後で比較し、assetsの全path/bytesも独立比較する。app.lockはStorageを閉じた準備時に読み、writer/Storage終了後に同bytesを比較する。prepare/publish間のStorage再openはない。取消しは同期APIのため本単位の対象外であり、拒否後の同操作再試行を含む。

fixtureは既存reader試験と同じnative DeviceIoControl mount-point方式である。全pathは試験専用temp内、reparse属性・canonical target一致・リンク経由全bytes読取を確認する。Dropはjunction entryを非再帰解除した後だけTempを解放する。解除に失敗すればTempをforgetして残し、リンク経由の再帰削除を避ける。実データ・外部通信・UAC・OS設定変更はない。

## 検証

compileは成功した（`artifacts/w1-compile.log`、2m18s）。続くcargo focusedは依存を再buildし（1m47s）、2件を実行した。sandbox内ではcase①がnative junction作成時error 5、case②が正常prepareの段階でIoとなり、期待する正常fixtureへ到達しなかった（`artifacts/w1-focused-sandbox.log`）。いずれも行動REDには数えない。依存cacheの調査は範囲外である。

その後は同じ生成済みEXEを新2件のfilter付きでsandbox外・非管理者実行した。**2 passed / 0 failed / 0 ignored / 35 filtered、0.11s**（`artifacts/w1-focused-unsandboxed.log`）。正常fixtureで仕様期待値を満たしたため、本体欠陥は再現していない。本体修正は行わず、既存実装の受入証拠を補完した。本単位の指摘は試験不足であり、通すために本体を壊すような人工的REDは作成していない。

`rustfmt --edition 2021 --check tests/audio_history_store.rs` とsnapshotとの差分whitespace検査は成功した。差分は末尾291行の追加だけである。全37件の独立runner実行とPost-gateは未実施であり、親へ引き継ぐ。

固定するsource SHA256は `7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4`。EXEは `target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe`、SHA256は `F1A00AE86F10C0045FD59CFEDE8FECC3BC9B18182F651A6E8A76321A1BCAA2F2` である。runnerはsource/EXE不変を確認し、このEXEで全37件を実行できる。reader suiteの反復は本単位では要求しない。

```text
cargo test --locked --offline --test audio_history_store --no-run
cargo test --locked --offline --test audio_history_store recovery_junction:: -- --nocapture
# 同じ生成済みEXEでsandbox制約を切り分けた実コマンド
./target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe recovery_junction:: --nocapture
# 独立runnerへ渡す全37件
cargo test --locked --offline --test audio_history_store
# source/EXE不変なら再buildを要さない同等の全37件実行
./target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe
```

ログは `artifacts/w1-*`。独立reviewerは同一writer寿命、oracle独立性、全bytes比較、unlink失敗時保存を確認する。本体に行動REDが出た場合は期待値を変更せず別Packetへ返す。
