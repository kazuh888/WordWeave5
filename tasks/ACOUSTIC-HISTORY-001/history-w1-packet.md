# I1b-W1 — writer junction受入証拠の補完

2026-10-07。受入指摘は [history-writer-review.md](history-writer-review.md) のI1b-W1のみ。本PacketはM2と独立であり、本体の欠陥を予断しない。

## 1. Objective
recoveryディレクトリのjunction経由で、履歴writerが別の保存先へ書き込まないことを実際のWindows fixtureで確認する。

## 2. Scope
所有変更は `tests/audio_history_store.rs` の2試験と必要なprivate fixtureのみ。`tests/audio_history_closure.rs` の既存junction fixtureを参考にするが、reader試験は変更しない。本体・公開API・期待値・quota・保存形式・権限設定・実データは変更しない。

## 3. Current evidence
writer35件は成功済みである。しかし通常ファイルによるrecovery妨害と、readerのjunction拒否だけでは、新しいwriter境界の証拠にならない。指摘は未確認の実装バグではなく必須試験の不足である。

## 4. Observable contract
正本は `design-i1b.md` とI1b-W1指摘である。①prepare前からrecoveryがjunction、②正常prepare後にrecoveryをjunctionへ差し替えたpublish、の両方で `UnsafePath`、追加object 0、リンク先・既存原音・intentの全bytes不変を要求する。

## 5. Owner / consumers
本体境界はrecovery.rsのpaths/save_intent/inventory、呼出側はHistoryWriter prepare/publishである。テストは公開writer APIから通す。独立test-authorが試験を作成し、独立reviewerが証拠を確認する。

## 6. Types / errors
既存 `HistoryErrorKind::UnsafePath` との完全一致を要求する。fixture作成失敗・アクセス権不足・別エラーを安全性試験の成功へ読み替えない。

## 7. Transition / failure-after-state
①正常な既存履歴とリンク先sentinelを用意しjunction化→prepare失敗。②正常prepare成功とintent存在を独立確認→recoveryを隔離された退避先へ移動し、その同じディレクトリを指すjunctionを元位置へ作成→同じprepared値でpublish失敗。②ではprepareによる正当なintent作成を、publishの追加書込みに数えない。

## 8. Security / data boundary
すべて合成bytesと試験自身のtemp内に限定する。junctionはnative APIで作成し、OS設定・UAC・管理者権限を要求しない。fixtureのDropは先にjunction entryだけを解除し、解除失敗時はtempを残す。リンク先へ再帰削除しない。publish試験は同じwriterとStorageを保持する（session同一性が必要であり再open不可）。snapshotはhistory配下と隔離リンク先の全bytesを対象とし、Storageが排他保持するhost直下のapp.lockは対象外と明示する。既存原音・全object・intent・リンク先sentinelは対象から除かない。

app.lockは、試験用Storageを一度閉じた準備時にbytesを保存し、再openしたStorageでwriter生成→正常prepare→junction化→同writerでpublish拒否→writer/Storage終了、の後に別途比較する。prepareとpublishの間ではStorage/writerを作り直さない。publishの `UnsafePath` がこの同一sessionから返ることを明示する。

## 9. Dependencies / stop
独立Pre-gate PASS後に試験を実装する。新試験が正しいfixtureで本体欠陥を示した場合は証拠を保存して別の本体修正Packetへ戻す。テスト期待値を緩めない。sandboxの既知junction拒否は低リスクの試験限定sandbox外実行で切り分け、管理者実行と混同しない。

## 10. Deterministic oracle
reparse属性と実際のリンク解決を確認する。失敗直前/直後の全ファイルpathとbytes、既存原音、元intent、リンク先sentinelを比較する。単なる件数やhashだけ、`is_err()`だけでは不足である。リンクを辿るsnapshotは同一temp内の既知targetのみとし、循環を作らない。`cargo test --locked --offline --test audio_history_store` の既存35件と新2件を確認する。reader本体は変更しないため既存37件の反復は不要である。

## 11. Rollback / artifacts
本体と利用者EXEは変更しない。新テストは削除せず失敗を記録する。ログは `artifacts/w1-*`、報告は `history-w1-tests.md` とする。Git操作・公開・他データ削除は行わない。

## 12. Traceability / status
I1b-W1 → W06 → 2件の独立writer回帰 → 独立Post-gate。現在はPre-gate未提出。I1b-2b、I2 pointer commit、全アプリ納品は対象外であり未完了のままである。
