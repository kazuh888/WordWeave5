# FEEDBACK-UI-001 U7 教材登録詳細の局所設計

2026-09-30。入力は確定済み [review-detail-plan.md](review-detail-plan.md) の D01–D05 と既存 [gfm-spec.md](gfm-spec.md) / [gfm-design.md](gfm-design.md) の U6 契約である。計画者指定は designer `gpt-6-sol / high`。本書は `src/app.rs::material_panel` の本文配置と `src/app/material_review.rs::material_comparison` / `comparison_detail` の設計差分だけを定める。本体・テスト・保存形式は変更しない。

## 境界と受入対応

| ID | module / 入力・出力契約 | 状態と確認点 |
| --- | --- | --- |
| D01 | `material_review.rs` の固定会話表示は `Draft.source.snapshots[*].exchange.answer` を借用して `notifications::show_markdown(ui, stable_id, answer)` へ渡す。質問は従来の原文Labelである | 見出し・日本語太字・表等はU6描画契約を再利用する。snapshotの話者、往復、添付と既存操作を保持する |
| D02 | rendererは表示専用であり、`Draft`、`Progress`、clipboard、`ready`、strict引用照合へ描画結果を戻さない | AI回答の原文引用強調は従来の `quoted_text(answer, quotes)` を別の折畳み「原文と該当引用を確認」で残す。引用は同じ選択理由/往復/roleから導き、Markdown表示と並存する。承認は既存の可視差分と登録buttonだけから進む |
| D03 | `app.rs` は単一の `material-proposal-body` scrollを廃し、有限高さのbodyを上部補助領域と比較領域へ配分する。`material_review.rs` は左一覧・右詳細に別々の縦ScrollAreaを置く | 左wheelは左、右wheelは右だけを動かす。両末尾へ到達する。比較領域の外に無制限高さの説明を置かない |
| D04 | header（既存receiptを含む）とbottom-up footerの予約は維持する。比較領域は狭幅でも有限幅の2paneである | 1150×950/80%と820×650/160%で両paneと登録・破棄・拒否理由・閉じるを操作できる。長い表/コードの横scrollはU6 renderer内に閉じる |
| D05 | 選択は既存の案単位 `key` と安定した `row.path` で識別する | path変更フレームで右詳細scrollを明示的に0へ戻す。以前選んだpathへ戻る場合も先頭である。左一覧のscroll IDは選択pathに依存させず、位置を保持する |

## 配置と状態

`material_panel` は既存のtitle・対象語、receipt、footerの順序と実測に基づくfooter予約を保つ。残る `body_height` を有限に割り当て、notice、元会話button、編集fold、同一基本語checkbox、再学習説明を上部の高さ制限付き縦ScrollAreaへまとめる。上部の最大高はbodyの一部に抑え、長いnoticeや展開編集はその中で到達可能にする。残高を比較領域へ与え、比較の導入文・適用されない旧理由foldも同じ有限領域内へ収める。body全体を包む親縦ScrollAreaは置かない。極小残高でどちらかのpaneが無操作になる実測が出れば、footer予約・上部配分を再調整し、受入を縮めない。

`material_comparison` は既存 `material_diff::rows`、`active_reasons()` と `selected` の妥当path確認を維持する。左右の幅はその時点の `available_width` から計算し、左を比率で割り当てつつ240ptを上限にして右へ残幅を渡す。760pt未満で縦積みに切り替えない。各paneは比較領域と同じ有限高の縦ScrollArea、別の安定IDを持つ。右内部の変更前/後は既存通り十分な幅で2列、狭幅では右pane内で縦積みとし、親幅を拡張しない。日本語の説明・見出しは右幅で折り返す。文字buttonと一覧選択は既存 `controls` を用い、可視字形の両軸中央寄せ、focus/keyboard/disabledを保つ。折畳み見出しは既存の左寄せ/縦中央を保つ。

左scroll IDは案の `key.with("change-list-scroll")`、右scroll IDは `key.with(("change-detail-scroll", selected.as_str()))` 相当とし、表/コードのU6子IDはその内側へ分離する。`key` の案識別・`material_selection_epoch` は既存のままとする。行click前後のpathを比較し、変更時だけ右ScrollAreaへ先頭offsetを指定する。path別IDだけでは以前のpathの旧offsetを再利用し得るため、明示resetを要する。選択不能pathは既存通り先頭行へ戻し、空一覧は選択状態を消す。scroll offsetはeguiの一時UI状態だけに置き、`Progress`へ保存しない。

固定会話では現行のsnapshot・role・引用集合をそのまま使う。AI本文の閲覧表示だけ `show_markdown` を呼び、別foldの原文 `quoted_text` により引用箇所の色強調と厳密な文字列確認を残す。ユーザー発言のMarkdown化はしない。表示中のリンク/画像/HTML/表/コードの安全性と局所横scrollはU6共有rendererの責務である。`show_markdown` のIDには案 `key`、選択path、往復番号、AI roleを含め、会話展開や選択変更で他paneと衝突させない。引用強調は視覚補助であり、strict判定値ではない。

## 非同期、永続化、回復とテスト口

### 実測による高さ配置の補正

狭幅の実測で、親UIの無有限 `bottom_up` footerは予約枠より下へ描かれ、作業領域のclipを超えた。また編集前readyによる予約では同frameのready変更に追従しない。したがって上部補助領域を有限高で先に描き、編集/checkbox変更後のreadyを確定してから、そのframeのfooter必要高と比較pane残高を計算する。footerも有限矩形のtop-downで理由行・操作行を描く。表示配分だけの変更であり、登録承認・差分・保存・原文契約は変更しない。helperは操作可能な高さを確保し、長文/展開内容は内部scrollで到達する。親がこの局所補正を承認した。最終合成/Windows検証は未了である。

新しい非同期処理・取消し境界はない。画像preview/録音再生の既存clickは右paneから同じactionを返し、AI worker・pending・cancel/key/epochには触れない。選択/scroll/foldは閲覧用一時状態、`Draft`編集と同一基本語checkboxは現行の所有・`changed`/`ready`経路を保持する。登録のsource追加、検証、`import_deck`、receipt生成の順序と、失敗時のメモリ復元・pending復旧条件はU6設計から変更しない。scroll/Markdown表示によって保存や登録を誘発しない。

test authorへはD01/D02の固定会話AI/ユーザー差、原文・引用強調・strict/clipboard不変、D03の左右wheelと各末尾、D05のA→B→A選択時の右先頭/左位置、D04の標準・狭幅/倍率・日本語太字/ボタン中央・固定header/footerと表/コード横到達を渡す。既存の合成fixtureと実panel入口を使い、renderer単体合格を固定会話入口の証拠にしない。runnerのWindows画面/操作で実字形と高さ/幅を判定し、音声実機・IME・実Codexの受入は推定しない。

設計時の静的根拠は `app.rs` の単一 `material-proposal-body` とfooter予約、`material_review.rs` の760pt縦積み・一時選択path・`quoted_text`、U6の `show_markdown`/原文/strict/receipt契約である。実装・テスト・実機描画は未実施。最大リスクは狭幅高倍率で上部・receipt・footerの残高が比較paneを圧迫する点と、nested scrollのwheel/ID干渉である。新しい利用者可視動作の未決事項はない。これらの実測がD03–D05を満たさなければ、設計へ戻して配分を調整する。
