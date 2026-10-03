# FEEDBACK-UI-001 U5 独立テスト引継ぎ

2026-09-28。担当: test author。計画者指定の起動値は `gpt-6-sol / high`。実行model/effortメタデータおよびtokenカウンターは未取得であり推定しない。入力は親が承認し、独立仕様レビューで阻害指摘0とした `notice-spec.md` N01–N08、および独立設計レビューで阻害指摘0の `notice-design.md` である。完了境界はテスト期待値の固定と実装者への引継ぎであり、本体修正、Windows実描画、全体gate、実AI/実データ試験は含まない。

## 所有とfixture

所有ファイルは新規 `tests/notice_diagnostics.rs` と本書のみ。既存の `src/`、既存テスト、仕様、計画、設定は変更していない。テストは `Conversation::new`、`Execution::default`、同梱の `BUILTIN_DECK` から合成したメモリ内データを用い、実学習記録・録音・外部AI・ファイルを使用しない。既存未コミット変更は保持した。

| テスト名の先頭 | AC | 固定した期待値 |
| --- | --- | --- |
| `n06_legacy_generation` | N06、Q03互換 | 旧String診断の240文字抜粋と旧文言を維持 |
| `n02_n04_seventh_reason` | N02、N04、N07 | 7番目の最初の失敗、4組目の固定回答、引用と元発言の全文、後編集を混入しない、旧String wrapper同値 |
| `n03_raw_evidence` | N03、N05 | literal `\n`、実CRLF/LF、Markdown記号、tab、前後空白、Unicodeを元文字列のまま保持 |
| `n07_markdown_display` | N07 | 原文の連続部分文字列だけを受理し、見た目のMarkdown正規化で誤受理しない |
| `n02_n07_invalid_path` | N02、N07 | path形式、実在項目が説明空白/引用0件より先。存在しない項目へ表示名や引用を捏造しない |
| `n02_missing_turn` | N02 | 不存在往復と不正roleは別causeで、別往復・別話者の元発言を代用しない |
| `n04_quote_limit` | N04、N07 | 2000 Unicode scalar値は受理、2001は拒否、拒否引用と元発言の全文は診断へ保持 |
| `n01_n07_saved_draft` | N01、N02、N07 | 保存案の再検査、段階、旧String同値、案の直列化内容不変 |
| `n02_saved_draft_source` | N02、N07 | Source失敗は理由/引用より先、未検査の引用を捏造せず保存案を変更しない |

## RED実測と実装後コマンド

旧String経路の診断実験では、251文字以上の固定元発言の末尾を全文期待する単一テストを `cargo test --locked --test notice_diagnostics n04_legacy_generation_failure_must_retain_full_source_for_comparison -- --exact` で実行した。テストはcompile成功、1件実行、`…（省略）` と先頭240文字のみを返してアサーション失敗した。これは旧表示の限界の実測である。ただし旧String文言は互換維持が仕様であるため、その全文アサーションは恒久テストに残していない。新しい型付きAPIは実装前には存在せず、存在しないAPIによるcompile失敗を行動REDとは数えない。実装開始と並走した二度目のcompile確認は依存コンパイル中に停止し、結果を判定に使わない。

実装後の焦点コマンドは `cargo test --locked --test notice_diagnostics` である。9件の実行を確認し、0件成功を合格としない。実装者は型付きAPIを設計の署名どおりに追加し、旧String wrapperの同値と厳密validation順序を維持する。runnerはU5a/bの本体と同一差分でこのコマンドを再実行し、既存教材回帰、Windows UI、全体gateを別途記録する。compileエラー時は不足APIかfixtureの誤りかを分けて報告する。後段N01/N03/N05/N06/N08の通知状態・Markdown描画・コピー・狭幅操作は本テストだけでは証明できず、U5cの独立UIテストと隔離Windows検証を要する。

## U5c 合成UIテストとnative引継ぎ

親の後段GOで `src/app/notifications.rs` と新規 `src/app/notifications/markdown.rs` の `cfg(test)` 節だけに9件、`src/app/visual_check.rs` の隔離preview hookだけに合成状態を追加した。通知本体・validator・保存/登録処理は変更していない。両テスト群は専用fixtureの一時ディレクトリを各件終了時に削除する。実教材・実学習記録は読み込まない。

| 焦点テスト | AC・判定 |
| --- | --- |
| `u5_typed_material_failure_through_pending_tick`、`u5_stale_material_result` | N01/N02/N06/N07: 型付き `AiResult::MaterialFailure` を実 `Pending` channel→`tick` に渡す。案・deck・進捗・AI回数を保ち、選択変更後の結果は通知にしない |
| `u5_direct_message_replacement` | N05/N06: 同文の新通知は比較/技術/Rawを初期化。直接 `message` 差替え後は一般本文になり、元文へ戻しても復活しない。fatal優先とfont警告表示を確認 |
| `u5_comparison_and_raw` | N01/N03/N05/N08: 標準初期の比較入口可視、実クリックによる展開・一括Raw切替、診断抜粋と両原文の正確な `CopyText` を別々に照合 |
| `u5_long_comparison_scroll` | N04/N08: 合成長文で標準/狭幅の実wheel末尾到達とコピーfooterの画面内維持 |
| `u5_saved_draft_detail_click` | N01/N02/N07: 保存案の実「理由を詳しく確認」クリックで固定版の `SavedDraft` 診断へ入り、案・deckを変更しない |
| `u5_markdown_paint_*` 2件 | N03/N08: 見出し/箇条書き/強調/斜体/コードの描画と、未閉鎖記号・literal `\n`・画像/HTML/URL文字の保持 |
| `u5_close_button_then_status_reopen` | N01/N05/N06: 実paint title fontが本文より大きい、footer「閉じる」実clickで閉じる、status通知の実click再表示で比較/Raw/技術情報を初期化 |

焦点コマンドは `cargo test --locked --bin wordweave5 u5_`（最終9件必須）。初期8件の最初の実行はcompile成功・4成功/4失敗。単独`Context`の未登録フォントと、Window初回paint・スクロール外ボタン・wheel後の座標移動というfixture条件を修正した。専用window ID・標準初期高さを含む次回実行は7成功/1失敗で、残るRawボタンの座標を安定後に取得するよう修正した。当該1件は `cargo test --locked --bin wordweave5 u5_comparison_and_raw_are_one_switch_and_copy_emits_exact_originals` で1件実行・PASS。最後に追加した閉じる/再表示テストはrunner検証待ちである。最終9件aggregateと全体gateはrunnerが固定差分で行う。0件の`--exact`誤フィルター試行は成功証拠に含めない。

native capture用の合成fixtureは `--ui-check <隔離PNGパス> --notice-structured` に、必要な操作を `--notice-compare`、`--notice-raw`、`--notice-end` のいずれかで加える。`--notice-raw` は比較クリックを先行し、`--notice-end` はRaw後に実wheelする。`--small` は820×650/160%、省略時は1150×950/80%である。描画済みボタンの可視位置へpointer press/releaseを送り、比較/Raw後に本文まで実wheelし、24frame後に撮影する。`--notice-end` は末尾markerの実paint可視を撮影前にassertする。合成本文はprovide infrastructure/run/power、太字、箇条書き、実改行とliteral `\n`、固有の末尾markerを持つ。native撮影の成功・フォント・日本語判読・末尾実視認は未検証であり、runnerの別証拠を要する。headless fixtureのフォントはアプリ起動時と異なり得るため、画像と操作で最終判定する。
