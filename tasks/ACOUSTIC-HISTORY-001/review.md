# 独立仕様・設計レビュー

2026-10-04。担当 `/root/acoustic_review`、指定 `ww-reviewer` / `gpt-6-astra/high`。実行metadata・利用量は未取得。担当はread-only、親が報告を記録する。

## 初回

対象はspec/design/tests/evidenceと再利用入口、計画の割当・基準、および既存store/commit/backupの必要な保存境界である。原音固定からcommit、保存再試行と取消し、Qwen同意、比較系列、ごみ箱とバックアップ復元を追跡した。Critical/Highなし、Mediumの受入阻止2件、質問1件であった。

| ID | 指摘と具体的な分岐 | 修正 |
| --- | --- | --- |
| R1 / Medium | spec AC01/tests AC01はengine欠落時に解析利用不可、designはPraat欠落でもRust部分結果を返す。同じ操作から異なる実装になる | Praat不在はF0欠測と理由を表示し、RMS・時間・低エネルギーを部分結果として保存する。全体入力不正/取消し/timeoutは途中結果を公開しない。spec/testsを合わせた |
| R2 / Medium | tests AC15の容量不変と、旧object保持・新revision/root追加の設計が矛盾する | 原音・結果の保持と、使用量が減らず新metadata増分を計上するoracleへ修正した。spec AC15の証拠欄も修正した |
| Q1 / 質問 | close時のgeneration失効とwindow-only閉鎖/controller保持が曖昧で、正常完了を捨てる可能性がある | window非表示ではjobを維持し元試行へ保存する。明示取消し/trash/確認済アプリ終了のみ失効させる。spec/design/testsに閉じて再表示するケースを追加した |

修正は親が行い、同じ独立担当へ変更範囲の再レビューを依頼した。動作試験で確認したという意味ではない。

## 残る検証範囲

音響精度、Windows子processの資源制約、配布ライセンス適合、実機アクセシビリティ、実装のデータ耐久性は後続gateである。今回の文書レビューで実行品質を証明しない。

## 再レビューの判定

独立担当がspec/design/testsの変更箇所と関連状態を再確認し、R1・R2・Q1の解消と、新たな受入阻止指摘がないことを報告した。Praat欠落時のPartial、全体中断、metadata容量増分、window非表示での正常保存とアプリ終了確認の取消しが、仕様・設計・試験計画で一致している。設計文書の指摘解消であり、利用者採否や動作試験の合格ではない。
