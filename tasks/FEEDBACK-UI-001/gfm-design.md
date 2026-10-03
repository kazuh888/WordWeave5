# FEEDBACK-UI-001 U6 Markdown表示・教材登録結果の内部設計

2026-09-29。設計担当の所有は本書のみである。入力は [gfm-spec.md](gfm-spec.md) G01–G08、[gfm-plan.md](gfm-plan.md)、既存 [notice-design.md](notice-design.md) である。親は仕様の対応構文と「次の明示登録操作が受理された時点で結果を消す」を、既承認利用者要求の具体化として承認済みである。計画者指定は U6a/b/c 共通 `gpt-6-astra / high`、実行metadataは未取得である。本書は独立仕様・設計レビューと計画最終確定への入力であり、自身でそのゲートを承認しない。

## 1. 根拠と採用境界

- `src/app/notifications/markdown.rs` は記号の対を独自検索して単一 `LayoutJob` を作る。表・入れ子をこのparserへ追加しない。
- `src/app/chat_ui.rs` の `bubble_with_attachments` はユーザー/AI共通の `Label(text).wrap().selectable(true)` を持ち、コピーは借用原文 `text.to_owned()` を直接渡している。bubbleの垂直child、clip内幅、添付/話者/操作は再利用する。
- `src/app.rs` は `heading` font familyの先頭へ `YuGothB.ttc` を登録済みである。太字は `RichText::strong()` の色だけに依存せず、この字体を使う。
- `material_panel` の承認clickは `ready` → source追加/案除去 → progress検証 → `import_deck` の順である。`import_deck` は `commit::save` 成功後だけ `self.progress/self.deck` を更新して `Ok` を返す。
- `src/commit.rs` は pending intent発行 → `custom.tsv` → `progress.json` → pendingをbackupsへ退避の順である。最後の退避失敗も `Err` となる。`import_deck` はpendingが残ればfatalを設定する。メモリのdeckが元のままでも、ディスク不変の証明にはならない。

`pulldown-cmark = { version = "=0.13.0", default-features = false }` をWindows target依存へ直接追加し、lockを更新する。0.13.0は今回取得済みの公式配布ソースでAPI/featuresを確認した固定基準であり、最新版という意味ではない（調査時0.13.4の存在を確認）。選択する構文は `Parser::new_ext` と `ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS` である。`ENABLE_GFM` は全GFMを一括有効化する指定ではなく、この版では引用alert拡張であるため使わない。smart punctuation、footnotes、math、metadata、heading attributes等も有効化しない。構文の意味は標準parserに任せ、ローカルrendererだけを作る。

確認資料は配布物 `pulldown-cmark-0.13.0/Cargo.toml.orig` と `src/lib.rs`（Tag/Event/Options）、[公式repository](https://github.com/raphlinus/pulldown-cmark)、[GFM仕様](https://github.github.com/gfm/) である。defaultのCLI/HTML出力featuresは不要であり切る。HTML出力を無効化しても入力HTMLイベントは発生するため、下記の文字描画が必要である。依存取得・lock更新・build適合の最終証拠は実装/runnerが取る。

`egui_commonmark` の全面viewerは採用しない。親の0.20.0ソース調査でStrongの実太字不足、任意schemeのリンク、既存loaderへつながる画像経路が確認されており、G02/G04のために別途制限が必要になるからである。REST、永続schema、全アプリのMarkdown化、汎用通知frameworkは追加しない。

## 2. 共通描画の契約・所有

配置を変えず、`notifications/markdown.rs` をprivate実装とする。通知moduleに次の薄い共有入口だけを置く。

```rust
// src/app/notifications.rs、appの兄弟moduleから呼べる範囲だけ公開
pub(super) fn show_markdown(ui: &mut egui::Ui, id: egui::Id, source: &str);
// src/app/notifications/markdown.rs、上記wrapperから呼ぶ
pub(super) fn show(ui: &mut egui::Ui, id: egui::Id, source: &str);
```

入力は借用原文である。戻り値による変更、I/O、clipboard、URL操作、AI、保存、app stateへの参照を持たない。`id` は表/コード内のスクロールとテキスト選択を安定させるUI identityであり、内容照合・保存IDではない。通知は引用/固定元発言で異なるID、チャットは会話ID・往復番号・AI回答の既存copy ID由来を使い、ブロック番号を子IDにする。

通知の読みやすい二本文だけをwrapperへ接続する。Raw表示、`QuoteEvidence`、`diagnostic_copy_text`、各原文コピーは現状の原文から作り続ける。chatは `user == false` の本文だけwrapperへ渡し、`user == true` のLabelは維持する。既存コピー、読み上げ、注釈、根拠選択は `exchange.answer` 原文を使う。表示済み文字やHTMLを `Progress`、案、snapshot、clipboardへ逆流させない。

renderer内部はparserイベントをflat arenaへ格納し、描画を分離する。node間はindex/範囲で参照し、子を再帰的な `Vec<Node>` として所有しない。構築、block描画、inline/style走査、画像alt/コードの文字抽出はヒープ上の明示stackと反復で行い、dropにも深さ依存の再帰を残さない。eguiのFrame/indent closureを深さの分だけ再帰呼出しする描画も使わない。Paragraph/Heading/Quote/List/Code/Rule/Tableの構造、inlineのstrong/emphasis/strike/code、表のalignment/header/rowsを保持する。名前・細かな格納方法は実装私有であり、外部testがAST内部値を契約にしない。手製の記号再解析や正規表現によるtable判定を加えず、段落のないtight listのTextもitemとして保持する。表セルはinlineの列であり、行全体を一つの文字列へ潰さない。

スタイルはstackのpush/popで合成する。Strong+Emphasis、取り消し線+Code等を排他的分岐で欠落させない。通常本文は呼出元のbody/override fontを使い、Strongと見出しは同じサイズ基準の `FontFamily::Name("heading")` を使う。コードは等幅familyと背景で区別し、コード内容を再解析しない。全テキストLabelは選択可能にする。各ブロックを選択・コピーでき、既存の全文copyボタンは全文原文を保証する。

| 標準イベント | ローカル描画契約 |
| --- | --- |
| Paragraph/Heading/Quote/List/Item/Rule | 段落間隔、見出し、引用の字下げ/境界、リストのmarkerと入れ子、区切り線。順序付き開始番号と子段落を保持する |
| Text/Code/CodeBlock/SoftBreak/HardBreak | Textはparserのescape解釈結果、inline codeはliteral、code blockは空白と改行を保持。SoftBreakは空白、HardBreakは改行として表示する |
| Table/TableHead/TableRow/TableCell | header/各セル/列境界が分かる表。alignmentを列へ反映し、headerを太字にする。区切り行なしの本文を表に推測しない |
| Strikethrough/TaskListMarker | 線による取り消し。taskは `[x]` / `[ ]` と本文を非操作文字で示し、Checkbox操作は作らない |
| Link | 子の表示文字だけを通常の選択可能文字にする。リンクwidget、クリックhandler、scheme判定からの起動経路を作らない |
| Image | 子イベントからalt文字を集め、解決済みdestinationとともに「画像: alt (destination)」という文字で表示。altなしでも参照先を失わない。Image/loader/textureを呼ばない |
| Html/InlineHtml | 受けた文字をそのまま選択可能Labelへ渡す。HTMLとして解釈せず、空文字化しない |

未知/非対応部分はparserのliteral出力を保持する。防御用fallbackが必要なときは対応原文範囲を文字で表示し、黙って消去しない。成立するCommonMarkの深い入れ子を非対応とみなし、深さ閾値で本文全体をRaw表示へ退避することはしない。今回有効化しない拡張を実装都合でONにしない。描画失敗を教材検査エラーへ変換しない。

parserは同期の純粋な表示処理である。最初はframe間のAST/cacheを追加せず、古い回答や通知へ前の構造が残る状態をなくす。新しい文字数・列数・深さの閲覧上限、切捨て、バックグラウンドjobを追加しない。長い既存fixtureでframe時間/末尾到達が問題になる場合は、原文一致をkeyにした限定cache等を別途設計へ戻す。無制限の永続cacheを先行導入しない。

## 3. 表と画面配置

表はtable単位の横ScrollAreaに閉じ込める。viewport幅を親の `available_width` 以下に固定し、`auto_shrink([false, true])`、最小scroll幅0、横方向だけを持たせる。内部の列幅は有限値とし、各セルはLayoutJobをその幅でwrapする。少列は親幅へ収め、多列は可読幅を保って横移動で全列へ到達させる。長い空白なし英単語もセル内で折り返す。行の高さは全セルの最大値へ合わせ、headerと各行の列位置を共通化する。列数や内容の自然幅で親Uiの幅を広げない。表の縦方向は既存本文ScrollAreaを使い、縦scrollを表の中へ重ねない。

コードは空白を保つ選択可能Labelと、親幅に限定した横ScrollAreaで全文へ到達させる。本文段落は親幅でwrapする。リスト/引用の字下げ量はviewport残幅に応じて頭打ちにし、本文を表示できる幅を確保する。論理的な入れ子深度は保持し、字下げが同じ位置になってもmarker/引用境界や深度表示で構造を区別できるようにする。狭いchild内でwrapまたは必要なローカル横scrollに閉じ込め、親viewportを拡張しない。これは字下げの幾何制限であり、構文深さの受付上限や本文省略ではない。本文を固定高さで切って末尾を失わせない。

通知は既存bottom-upの固定footerと本文縦ScrollArea、チャットは既存composer固定領域・header・transcript縦ScrollAreaを保持する。bubbleの明示的な垂直childを維持する。成功receiptを足す分だけ本文の残高を測り直し、表の幅や長い対象語で送信・閉じるを押し出さない。

文字ボタンは既存 `controls::Button` / `UiControls::ww_button` を使い、日本語Galleyのmesh boundsに基づく両軸中心、focus、disabled、キーボード操作を再利用する。既存アイコンcopy buttonは独自の実Buttonとアクセシブルlabelを維持する。task markerはボタンではない。日本語の段落行間へbutton用の中心補正を流用しない。

1150×950/80%と820×650/160%で表の全列/全行・長いセル・コード末尾、通知footer、chat composer/根拠選択、receiptの閉じるを確認する。通常文字と日本語Strongの実glyphを同画面で比較する。`heading`指定だけではYuGothB未導入環境の実太字を保証しないため、フォントがない場合はG04を合格扱いせず親へ返す。Windows DPIとIME等は今回のzoom/geometry試験と区別して報告する。

## 4. 教材登録receiptと保存順序

`notifications.rs` に次を置き、`WordApp` に `material_registration_receipt: Option<notifications::MaterialRegistrationReceipt>` を加える。初期値はNone、serde/Progressへ加えない。

```rust
pub(super) struct MaterialRegistrationReceipt {
    pub(super) target_base: String,
}
impl WordApp {
    pub(super) fn notify_material_registered(&mut self, target_base: String);
    pub(super) fn show_material_registration_receipt(&mut self, ui: &mut egui::Ui);
}
```

成功helperは `notify_result(Ok(format!("{target_base} の教材を登録した。元の会話への参照も保存した。")))` で既存messageを更新し、旧 `notification_error` / `notification_attention` / material/daily-limit payloadを消す。その後receiptをSomeへする。成功のために `notification_open=true` を設定せず、fatalを消さない。呼出元は教材案の承認処理にある `import_deck(…).Ok(())` 節一箇所だけとする。汎用importや案生成はreceiptを作らない。

receipt描画は登録dialogのclosure内で `ux::dialog_body` の直後・draft有無の分岐前（本文ScrollArea外）、およびchatの `chat_header` 直後・transcriptの前から呼ぶ。同じ一個のOptionを表示するため、どちらの「登録結果を閉じる」でも両表示が消える。対象語付きtextはwrapする。長い対象語はreceipt本文に有限高の縦scrollを設け、閉じるボタンをその外に保つ。狭幅ではtextとボタンを縦配置し、chat本文/入力やdialog操作の残高を確保する。dialog自体のcloseはreceiptを消さない。

| 入力/現在状態 | 状態遷移と副作用 |
| --- | --- |
| 起動、生成成功、案破棄 | 起動はNone。生成/破棄は新receiptを作らず、既存receiptの寿命に介入しない |
| 次の登録clickがenabledかつ受理された | 保存/検証前にreceipt=None。disabledボタンのhover/click、次の案生成開始では消さない |
| ready/Progress検証が拒否 | 成功は設定しない。既存disabled理由と詳細経路を維持。今回受理した登録内の失敗はnotify_errorで自動注意を促す |
| import_deck Ok | 保存成功後に対象語を渡してSome。新規/追加/訂正で同じ成功判定 |
| import_deck Err | Noneのまま。対象語と原因をnotify_errorへ渡す。pendingによるfatalがある場合は既存fatalの復旧説明/停止を優先する |
| コピー、通知閲覧、dialog閉鎖、画面再描画 | receiptを保持する。一般notify_*やmessage変更にも連動させない |
| 「登録結果を閉じる」 | Noneのみ。保存、取消し、案破棄、AI呼出しを行わない |

登録前に `entry.base.clone()` を成功/失敗の対象表示用へ退避する。案内容や保存結果を文字列から再構成しない。既存 `old_progress` / `old_deck` 保持、material_sources追加、案除去、selection epoch、検証、import、失敗時メモリ復元の順序は変えない。成功文言の目的で保存を追加したり、先に成功表示を予約したりしない。

`import_deck` / `commit` の戻り型は変更せず、UIは成功とErrをそのまま扱う。確定した保存前失敗でも最小の表示は「対象の登録処理に失敗した。原因」であり、不必要にディスク不変を断定しない。pendingがある失敗は `import_deck` の既存fatalにより復旧待ちとして識別される。教材本体だけ保存済み、両方保存済みでpending退避失敗、復旧先の外部変更検出のいずれも、成功扱いや先行再登録案内にしない。復旧は既存起動時処理に任せ、receiptを再構成しない。

新規非同期境界はない。既存AI worker・`Pending`・cancel/key/epochによる破棄を変更しない。登録はidle時の同期処理であり、表示closeを保存取消しへ意味付けしない。復旧待ちの `Storage::save` 停止とfatalによる編集停止を維持する。

## 5. 受入ID、テスト口、実装引継ぎ

| AC | module / 状態・データ所有 | 独立した受入証拠 |
| --- | --- | --- |
| G01 | markdown、notifications比較、chat AI分岐。原文は既存diagnostic/Exchange所有 | 同一合成入力を通知2本文/保存済みAI回答へ投入し構造を確認。ユーザー本文だけ記号を含む原文のまま |
| G02 | parser Options、block/inline、非操作のLink/Image/Html/Task | 入れ子・表escape・区切りなし行・未閉鎖、alt/destination、任意scheme。描画内容とOpenUrl/画像loader呼出なしを別判定 |
| G03 | Raw、diagnostic_copy_text、bubble copy。renderer出力は保存しない | CRLF/空白/tab/Unicode/240–241のclipboard完全一致、Raw切替前後と保存前後の原文一致 |
| G04 | 横scrollとセルwrap、heading family、固定footer/composer | headless geometry + Windows2サイズで全列/行/末尾に移動。日本語の実太字は画像証拠で判定 |
| G05 | material_panel Ok節、receipt、dialog/chat固定位置 | 実承認button経路で新規/追加/訂正→保存ファイルとSome、案消滅後の両画面表示、auto-dialogなし |
| G06 | ready、import_deck Err、pending/fatal。メモリ復元とdiskは別 | 引用拒否/削除対象/保存先なし/保存前失敗/途中失敗。成功なしとauto通知、pending復旧説明と停止、隔離diskと案を照合 |
| G07 | receiptのSome→None、一般通知から独立 | 成功→コピー→dialog閉鎖→保持、結果close→消去、次の受理click→失敗→旧成功なし。生成開始では維持 |
| G08 | material validator/保存/worker/controlsは既存契約 | strict引用例/2000–2001、append既存内容・復習状態、AI呼出数不変、keyboard/copy/disabled操作 |

test authorは新規 `src/app/gfm_tests.rs` と `visual_check.rs` のU6合成fixtureを所有する。app.rsのtest module宣言や必要な既存harness helperの `pub(super)` 化は実装担当が統合する。テストは共有wrapper/WordApp UIを使い、AST型や実装と同じparserから期待値を生成しない。`show_markdown` は描画専用なので `FullOutput` の文字/shape、tableのscroll移動、既存bubble copy座標、クリック前後のapp state/diskがテスト口となる。表scroll IDは入力id.with(block index)から追跡できるようにする。必要ならcfg(test)だけでtable viewport/内容rectを記録し、描画期待値を返す本体APIは増やさない。

途中保存失敗は隔離fixtureで `backups` をファイルにする等、commitの最後の退避段階を失敗させれば、教材/記録が書かれていても成功と断定できない経路を実際に通せる。保存前失敗は保存先なし等で別に確認する。既存 `commit.rs` の復旧検査も再利用対象だが、helperだけの成功試験をmaterial_panelの実click受入の代用にしない。実利用者データ・実AIは使わない。

実装所有案は U6a: Cargo.toml/Cargo.lock/notifications.rs/notifications/markdown.rs、U6b: chat_ui.rs（共有入口に変更が要ればnotifications.rs）、U6c: app.rs/notifications.rs/chat_ui.rs である。U6cのchat_ui追加と新規test moduleを計画最終確定へ反映する。共有ファイルは実装者一名がa→b→cの順に扱い、同居testの既存期待値を勝手に書き換えない。U5の限定parser期待値のうち今回仕様が更新する箇所だけtest authorへ返す。

残るリスクは、大量セル/深い入れ子の描画負荷、フォントfallbackによる見かけの太字不足、狭幅receiptの高さ、保存途中失敗と未反映の混同である。いずれも受入削減で解消しない。標準parserの導入だけを「全GFM準拠」の証拠にしない。深い入れ子への設計精緻化は親採用のflat arena/明示stack方式に限定した。`"> ".repeat(7000) + "末尾"` と前後の通常の太字/表を合成し、構築・描画・drop完了、装飾保持、末尾到達、原文copy一致を焦点回帰で確認する。深い入力を含むstack overflowの実測は本設計更新時点では未取得であり、コード/試験結果はreviewerへ返す。仕様に戻す未決事項は現時点でない。

設計時検証は本書の差分/参照/AC対応確認だけであり、本体test/build/native受入は未実施である。AGENTS、development-team、KPI、指定2 Skillを読了した。KPIの子開始/終了カウンターと親子包含は未取得であり推定しない。通常shellはACL初期化エラー、read-only昇格で調査した。担当外文書・本体・テストを編集していない。
