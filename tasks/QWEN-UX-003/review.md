# QWEN-UX-003 独立レビュー

2026-10-03。reviewer起動指定はgpt-6-astra/high、実行metadata・usage counterは未取得である。read-onlyで仕様・設計・本体・独立テストを確認し、本書は親が報告を保存した。

## 静的確認

仕様/設計に未解決のblockerはない。自動prepareは通信を伴わず、明示SendがcontrollerのPreparedId照合を通る。読込中拒否、Cancel/Close優先、閲覧用receiptの非権限性と寿命、原音保持を追跡した。根拠はqwen_reading_ui.rsのrender/dispatch_frameとReadingControllerのprepare/human_send/cancel/restartである。

- P2：同じegui Contextで①②を開いた状態が次の評価へ残る。receipt.id＋phaseのscopeで原因を修正し、新送信とRunning→Completed/Failed時に初期closedとなる独立回帰を追加した。
- P2：長い音声ファイル名をCollapsingHeaderへ渡すとExtendで横幅が広がる。固定短見出し＋wrapする短名概要、展開内の全文表示へ変更し、200文字名・820×650/160%描画の独立回帰を追加した。

reviewerは両対策と回帰の原因対応を確認した。Tab/Enter/Spaceによる独立回帰はrequest_focusを強制せず、実Response.idへのTab到達とactivationを検査する。headless証拠をnative操作の証拠へ拡張しない。

## 残る確認

着手前baselineコピー/hashは未取得であり、対象が既存untrackedのためHEADだけを今回差分として扱わない。前タスクの成功ログは今回の合格証拠へ流用していない。

実API・実マイク・DPI/IME・nativeキーボード・取消し終端のnative画像・利用者による最終受入は別の未検証である。

## 最終照合

同じ独立reviewerがv2全465件（音読34件を含む）、native14画像を実見し、release exit 0とEXE現物12,783,616 bytes／SHA256 `F6C1960698C86EA809B7AC7D0CE82DDCE236717660AC09513D4EB4E420DCB532`を照合した。test/debug/release間の入力manifestは一致し、共有package変更もない。既報P2二件は実装と独立回帰で解消し、新規P1/P2指摘はない。

実装・自動検証・合成表示・release生成の証拠は揃った。ただしplanの全体fmt gateは既存39ファイル差分でFAILである。対象3ファイルの整形成功と既存編集保持を確認したが、全gate合格・最終受入済みとは判定しない。
