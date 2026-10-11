# I1b-2a 独立実装レビュー

## 現在の判定（親による最終状態の追記）

2026-10-07、I1b-W1はClosed。独立作者がrecovery junctionの2件を追加し、本体は変更していない。独立runnerの全37件成功、独立reviewer `praat_m1_review`（指定Astra/high）のPost-gate PASS・残阻止0を受領した。同sessionのpublish拒否、全bytes・原音・intent・app.lock保持、fixtureの非再帰unlinkを確認した。[追加試験](history-w1-tests.md)、[独立実行](history-w1-runner.md)、[gate](m2-w1-gates.md)。I1b-2b/I2/全機能は未完了である。

## 初回レビュー時点の証拠

2026-10-07、history_writer_code_review、計画指定Astra/high、read-only。実行metadata・使用量counterは未取得。対象はhistory-writer-implementation.md記載の本体5本と独立writer testであり、開始/終了hash一致を確認した。再実行・編集は行っていない。

確定したコード欠陥0件、受入証拠の不足1件である。writer35成功・reader37成功/1明示ignoredは保存ログで確認した。古い昇格leaf成功を今回の37件へ加えていない。

| Finding | 原因・対象 | 状態 / 必須oracle |
| --- | --- | --- |
| I1b-W1 / P2 / 受入阻止 | recovery.rsのpaths/save_intent/inventoryは新しいwriter安全境界だが、現在のwriter試験はrecoveryを通常ファイルにするケースだけである。reader junction試験はこの書込境界を通らない。W06要求の証拠が不足 | Accepted、未修正。①recoveryがjunctionのprepare、②正常prepare後にrecoveryをjunctionへ差替えたpublishを独立合成fixtureで検査する。UnsafePath、追加object0、リンク先/既存原音/intent全bytes不変を要求する。実装突破を確認した指摘ではない |

静的に確認した範囲はexact request、全mutation、旧版/receipt保持、Run固定項目/終端遷移、重複集合、quota/部分publish再試行、intent先行、freshhead/pending、公開後閉包、opaque Debugである。この範囲に原音消失・誤quota・壊れたpublish・重複成功・原音Debug漏れの確定欠陥はなかった。

2bのresume/recover/confirm/finish、I2 pointer commit、実disk障害/電源断、全体deliveryはレビュー対象外である。I1b-W1の限定的補完と独立確認まではI1b-2a受入を保留する。
