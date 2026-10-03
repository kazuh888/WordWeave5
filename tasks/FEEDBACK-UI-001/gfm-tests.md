# FEEDBACK-UI-001 U6 独立テスト引継ぎ

2026-09-29。テスト作成担当は `src/app/notifications/markdown.rs` の既存 `cfg(test)` 追記、`src/app/gfm_tests.rs`、`src/app/visual_check.rs` のU6合成fixtureを所有した。入力は親承認済み [gfm-spec.md](gfm-spec.md) G01–G08と [gfm-design.md](gfm-design.md) である。期待値は原文と外部仕様から定め、本体parserの出力を期待値生成に使わない。

## RED証拠

`cargo test --locked --bin wordweave5 gfm_table_paints_cells_without_separator_syntax -- --nocapture` は新規1件を実行し1件失敗した。GFM区切り行 `| --- | --- |` とescaped pipe `a\|b` が現状の画面描画文字列に残る実際の表示不一致である。fixture/コンパイル失敗ではない。最初の `--exact` 実行は0件であり、RED証拠に算入しない。設計レビューでコード内pipeにescapeを要するGFM構文が指摘され、入力を修正した後に同じ表示不一致を再確認した。`markdown.rs` はその後、実装担当へ解放した。

## 固定した受入テスト

| ID | テスト・観測 | 適用範囲 |
| --- | --- | --- |
| G01/G03 | `gfm_chat_decorates_ai_only_and_copies_the_exact_saved_answer`。同じ `**重要**` をユーザー/保存済みAI回答へ置き、前者だけ記号が残る。既存copy buttonの `CopyText` はCRLF、前後空白、tab、literal `\n`、Unicodeを含む原文と完全一致し、会話は不変 | 実 `update_ui` のチャット経路 |
| G02 | `gfm_table_paints_cells_without_separator_syntax`。GFM表delimiterを文字として表示せず、太字・code・escaped pipeをセル内容として示す | 通知と共用のrenderer |
| G02/G04 | `gfm_table_has_aligned_header_and_body_cells_in_the_rendered_ui`。ヘッダー/本文の2列が別座標で行列として並ぶ | 共用wrapperの実paint。実Windowsでの太字・スクロール合格は別 |
| G02 | `gfm_tight_list_keeps_inline_styles_and_nested_child_paragraph`。tight list項目内の太字/斜体/codeの描画styleと、入れ子項目/子段落の順序・字下げを確認 | 共用wrapperの実paint。独立レビュー指摘を仕様のlist契約へ対応 |
| G02 | `gfm_list_keeps_nested_rule_and_fenced_code_as_distinct_blocks`。list内の区切り線を実line shapeで示し、fenced code内の太字記号を原文のまま表示する | 共用wrapperの実paint |
| G02 | `gfm_unsafe_markup_is_readable_but_does_not_open_external_targets`。fileリンク文字、画像alt+URL、HTML文字が残り、link文字クリック後もOpenUrlなし | 共用wrapper。OS/通信を起こさない隔離入力 |
| G05/G07/G08 | `gfm_registration_receipt_follows_real_approval_and_survives_dialog_close`。実登録button→保存済み教材/案消滅/対象付きreceipt、auto-dialogなし。dialogを閉じてchatに残り、全文copyでも残る。明示receipt閉じるだけで消え、教材/案/AI回数不変 | 隔離Storeと実 `material_panel` click |
| G05/G08 | `gfm_new_and_append_approvals_show_receipt_and_append_keeps_learning_state`。New/Appendも実button→対象付き結果を示し、Appendは既存意味・復習stateを保持 | 隔離Storeと実 `material_panel` click |
| G06/G07 | `gfm_next_accepted_registration_failure_clears_prior_success_and_keeps_proposal`。前回receipt→次の受理button→保存先なし失敗で旧成功消去、対象と原因、案/教材保持 | 隔離Store、保存前失敗 |
| G06 | `gfm_registration_partial_save_never_reports_success_or_erases_recovery_state`。backupsを隔離ファイルとして退避だけ失敗させ、pending/fatal/復旧説明、成功なし、メモリ教材と案保持を確認 | 実commit途中失敗。ディスクを一律未登録と断定しない |
| G02/G04 | `deep_quote_stack_keeps_tail_and_following_table_without_process_crash`。7000段の合成引用末尾と後続表をpaint/clip/dropまで保つ | 別test processで単独実行する。実行時間も記録 |

U5の既存通知テストがG03のRaw切替・診断/両全文コピー、G08の厳密引用・長さ境界/拒否を担う。重複テストを追加せず、最終runnerはその既存回帰も実行する。新規テストだけでG01–G08の全手動受入を代用しない。

## 実行と未検証境界

新module宣言と `harness_tests::material_ui_draft` の `pub(super)` 化は実装担当が統合する。実装後のfocusedコマンドは `cargo test --locked --bin wordweave5 gfm_ -- --nocapture`、深い引用だけは別processで `cargo test --locked --bin wordweave5 deep_quote_stack_ -- --nocapture`、既存通知の関連回帰は `cargo test --locked --bin wordweave5 u5_ -- --nocapture` である。実行数0件を成功としない。全対象テスト・release buildはrunner所有である。

native合成fixtureは `--ui-check PNG_PATH --gfm-notice`、`--gfm-notice-end`、`--gfm-chat`、`--gfm-receipt` と既存 `--small` の組合せである。通知/チャットの表末列を実横wheelで到達して撮る場合は `--gfm-notice --gfm-table-right` または `--gfm-chat --gfm-table-right` とする。標準1150×950/80%、狭幅820×650/160%の画像、表の全列/全行、長いセル、本文末尾、通常文字対日本語太字の実glyph、footer/composer/receipt操作をWindows画面で確認する。これらは表示証拠であり、`--gfm-receipt` は合成stateを作るため実保存成功の証拠ではない。保存成功/失敗は上記clickテストの隔離Storeで判定する。

実AI、実教材、実利用者データは使用しない。KPIの子使用量カウンターと親子包含は取得不能であり推定しない。

## U6再開後のfocused確認（2026-09-30）

登録結果を閉じた後のコピー連続テストだけが失敗した。ボタン矩形はtranscript clip内であったが、直後のクリックは`CopyText`を出さなかった。eguiは閉鎖Windowをfade中もinteractive layerに残すため、テストfixtureでそのlayerがボタン位置から退くまでheadless frameを進め、クリック直前にも退いたことを確認するようにした。期待する原文`合成回答`の完全一致、receipt保持、教材とAI呼出し不変のassertは維持した。製品側のコピー処理は変更していない。

`cargo test --locked --bin wordweave5 gfm_registration_receipt_follows_real_approval_and_survives_dialog_close -- --nocapture` は1件成功、続く `cargo test --locked --bin wordweave5 gfm_ -- --nocapture` は11件成功・失敗0・ignored0（181 filtered）。後者には実装担当の入れ子段落修正後に通過したlist回帰と7000段引用回帰が含まれる。深い引用の別process実行、全targets・release・native表示はrunner担当として未判定である。

## U6 表の折返し行高回帰（2026-09-30）

G02/G04の `gfm_wrapped_table_row_keeps_following_row_below_every_cell` を共用 `notifications::show_markdown` の実paintに追加した。標準1150×950/80%、同サイズ大文字160%、狭幅820×650/160%を各3 frame描画し、長い合成セルが短い隣セルより実際に高く折り返すこと、前行の全セル下端が次行の全セル上端以下であること、左右列の順序と次行の縦整列を判定する。実データは使わない。

実装担当の表修正がこの新テストの初回実行前に入っていたため、このテスト単独のREDは未取得である。修正前のnative画像と独立レビューで確認された重なりは別証拠であり、本テストのRED成功と同一視しない。最初の試行は合成長文が狭幅画面外へ延びて可視rectを取得できないfixture失敗であった。文字量だけを短縮し、折返し量 `long.height > short.height × 1.5` と非重複の期待は維持した。

`cargo test --locked --bin wordweave5 gfm_wrapped_table_row_keeps_following_row_below_every_cell -- --nocapture` は1件成功・0失敗・0 ignored（192 filtered、exit 0）。続く `cargo test --locked --bin wordweave5 gfm_ -- --nocapture` は12件成功・0失敗・0 ignored（181 filtered、exit 0）。ログは `gfm-row-focused.log` と `gfm-row-suite.log` である。Windows実画面の再確認、全targets、release buildはrunnerが別に判定する。

## U7 教材詳細の独立RED（2026-09-30）

親承認の `review-detail-plan.md` D01–D05と `review-detail-design.md` を入力に、`src/app/gfm_tests.rs` 末尾へ `review_detail_` 3件を追加した。合成案と固定会話を既存隔離fixtureで生成し、実AI・実教材は使わない。`review_detail_fixed_ai_answer_uses_markdown_and_preserves_literal_quote_source` は固定会話展開の実click後に、ユーザー原文、AI太字・GFM表、安全な別原文foldと該当引用の色強調、Draft/Progress不変を判定する。`review_detail_list_and_detail_wheels_are_independent_and_selection_resets_detail` は長い合成一覧/詳細の左右wheel、相手pane不動、双方末尾、A→B→Aの右先頭復帰を判定する。`review_detail_standard_and_narrow_dialog_keep_panes_and_actions_visible` は1150×950/80%と820×650/160%の実dialogで両paneとheader/登録・破棄操作の可視・固定を判定する。

本体変更前の `cargo test --locked --bin wordweave5 review_detail_ -- --nocapture` は3件実行・3件失敗・0 ignored・exit 101である。コンパイル成功後、D01はAI本文に `**AI_DECORATED**` とGFM区切り行が残る実paint差、D03は左wheelで一覧が動かない実操作差、D04は狭幅で一覧見出しが不可視になる実clip差として失敗した。fixture/compile failureではない。最初にD01単独も1件実行・同じ実paint差で失敗した。ログは `review-detail-red.log` と `review-detail-red-all.log` である。3件REDの後に追加した「双方末尾」と「右を十分scrollしてからのreset」assertは先行失敗後に位置するため、RED実行での到達は未確認であり、実装後のfocused実行で判定する。U7全対象・release・Windows実画面はrunner担当である。

## U7 狭幅footerの同一frame遷移回帰（2026-09-30）

D04/D05と登録承認・案保持契約に対し、`review_detail_narrow_footer_stays_visible_when_an_edit_invalidates_ready` を追加した。隔離fixtureの新規案は同一基本語の確認欄が選択されている時だけ `Draft::ready` が成功する。820×650/160%の実dialogで確認欄まで上部補助領域をscrollし、実clickで有効→無効へ変える。同じ描画frameで拒否理由、詳細理由操作、無効な登録操作、破棄操作が画面内で分離・順序保持され、案・学習状態・教材が保存されることを期待する。これはfooter予約が編集前のready、実footerが編集後のreadyを参照する経路を対象とする。実TextEditの値変更ではなく、同じready分岐に入る実Checkbox操作である。

D03の初回右wheelだけ35ptから10ptに調整した。高さ23ptの見出しが35pt入力でclip外になり、ラベル位置の比較に使えないためである。30空frame後に右移動が5pt超で左不動という判定、左右末尾到達、A→B→Aの先頭復帰は維持した。上記追加テストと入力調整後のCargo実行は本体担当へ直列に引き継ぎ、compile/fixture/動作REDの判定はその結果を待つ。未実行を合格または不具合再現とは記録しない。

初回の4件focusedはD03 wheel引数の型推論エラーE0282でcompileに失敗した。これはテストコードの不備でありRED証拠ではない。`distance: f32` を明示して再実行へ返した。理由可視の照合は上部checkboxと重なる「同じ基本語」でなく、拒否文固有の「登録済み」にした。理由文と詳細理由buttonは横並びでもよいので、縦順の仮定を除き、両者が登録操作と分離して可視である期待へ修正した。案保存・画面内footerの期待は維持する。

修正後の4件focusedはcompile成功・1件成功/3件失敗である（`review-detail-green-impl5.log`）。D01は成功した。D03では右wheel中の左offsetは35ptで固定だが、その直後にpointerを左へ移してBをclickするframeで左offsetが205.6ptへ飛び、Aがclip外となった。右wheelの平滑化が未完了の入力fixtureと切り分けるため、右paneの連続wheel後に30空frameを加え、B選択前に左位置が保持されることを改めてassertする。末尾到達後の左paneへの切替にも同じ待機を加えた。左を先頭へ戻す操作やA→B→Aの期待変更はしていない。待機後にも左offsetが飛ぶなら本体の独立scroll不具合として扱う。残る狭幅2件は登録操作不可視と補助領域の元会話button不可視であり、本体側の高さ修正前の挙動差である。新遷移テストはclickまで未到達なので、同一frame footer欠陥の再現は未判定である。
