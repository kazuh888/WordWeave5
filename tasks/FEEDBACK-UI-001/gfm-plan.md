# FEEDBACK-UI-001 U6 Markdown描画・教材登録結果の実装計画

2026-09-29。状態: **親承認済み仕様と設計を反映して計画・所有を確定。親伝達で独立仕様/設計レビューは阻害指摘なし。test authorのRED fixture修正完了後、親が実装をdispatchする**。仕様の親承認は [gfm-results.md](gfm-results.md) に記録済みで、仕様冒頭の旧「承認待ち」と区別する。計画者は本書だけを所有し、作業リストも本書に置く。既存の未完了 `tasks/plan.md` / `tasks/todo.md`、本タスクの既存文書、dirtyな本体・テストを保持する。実装・テスト編集・再委任・公開は行わない。

## 完了境界と根拠

通知の引用比較と英語チャットのAI回答を、仕様で定めたCommonMark/GFM構文に従って描画し、教材案の登録成功・失敗を保存結果と一致させる。独立テスト、隔離Windows表示・操作、全対象テスト、releaseビルド、独立レビューまでを開発完了境界とする。実AI・実教材による受入とは区別する。

- 親からの確定要求: 適用対象は通知の引用・固定元発言と英語チャット回答の両方である。通知だけに限定する案は採らない。ユーザー発言の通常チャット表示は対象外である。
- 親のソース調査: `notifications/markdown.rs` は限定手製parser、`chat_ui.rs` の回答はLabel、`app.rs` の教材案登録成功は `self.message` のみである。`import_deck` はatomic commitの保存成功後にOkとなる。これらは親提供の根拠であり、本計画者による再調査ではない。
- 入力契約は [gfm-spec.md](gfm-spec.md) G01–G08と [gfm-design.md](gfm-design.md) 第1–5節である。既存 [notice-spec.md](notice-spec.md) / [notice-design.md](notice-design.md) の厳密引用検査、原文コピー、通知状態、可視差分・承認を再利用し、旧「限定Markdown・通知専用・新依存なし」だけを今回の仕様で更新する。
- 設計確定: Windows targetに `pulldown-cmark = { version = "=0.13.0", default-features = false }` を追加する。`ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS` のみ追加有効化する。private rendererを保ち、`notifications::show_markdown(ui, id, source)` を兄弟module向けwrapperとする。Strongは既存heading字体、表/コードは親幅内の横scroll、本文は原文と分離する。`egui_commonmark` viewerは採用しない。
- receiptはメモリ上の対象語付きOptionであり、保存Ok後だけ設定する。dialog先頭とchat header直後に表示し、結果closeまたは次のenabled登録click受理時だけ消去する。生成開始・コピー・dialog閉鎖では保持する。pendingを伴うErrは既存fatal/復旧停止を優先する。
- 比較基準は親保存の `.context/compound-engineering/gfm-before.patch` と [gfm-results.md](gfm-results.md) の調査境界である。HEADとの差分全体をU6成果とみなさない。

## 要求・受入基準

| ID | 受入基準 |
| --- | --- |
| G01 | 通知2本文と保存済みAI回答へ共通描画を適用し、通常チャットのユーザー本文は原文のままとする。 |
| G02 | 仕様第2.1節のCommonMark構造・表・打消し・読取専用taskを描画する。リンクは非操作文字、画像はalt/参照先の文字、HTMLは文字とし、外部取得/起動をしない。 |
| G03 | Raw、通知/チャット原文コピー、診断コピー、保存原文を既存契約通り保持する。CRLF/空白/Unicode等を文字列比較で検証する。 |
| G04 | 1150×950/80%と820×650/160%で実際の日本語太字、表の全列/全行/長いセル、コード・本文末尾、固定操作領域を確認する。Windows画像とスクロール操作証拠を必須とする。 |
| G05 | 新規/追加/訂正の承認・保存成功後だけ対象語付き結果をdialog先頭と閉鎖後のchatに表示し、自動成功dialogを開かない。案生成・破棄・未承認を成功としない。 |
| G06 | 登録拒否、保存前失敗、pendingを残す途中失敗を区別する。成功を出さず、既存拒否理由経路/自動通知/fatal復旧説明を保持する。未登録・ディスク不変を一律に断定しない。 |
| G07 | receiptはコピー・通知閲覧・dialog閉鎖で保持し、結果closeか次の明示登録操作受理で消える。表示操作から保存・AI・取消しを起こさない。 |
| G08 | strict引用照合/上限、保存形式、AIプロトコル、可視差分と登録承認、append時の既存内容/復習状態、既存選択/コピー/keyboard操作を保持する。 |

本表は外部仕様G01–G08の索引であり、詳細例と期待値は同仕様を正とする。暫定計画のR01–R03はG05–G08へ統合し、旧IDを新しい受入証拠に使わない。

対象外: 全アプリへのMarkdown化、Markdown編集、HTML実行・画像取得、新しい自動リンク起動、引用正規化/自動修正、自動登録、永続schema変更、認証/モデル/Volta/PATH変更、実データ/実AI試験、無関係なrefactor、harness設定/品質ゲート変更、commit/push/公開である。

## 工程・タスク・所有

暫定計画、親承認済み外部仕様、U6全体設計を経て、本書の計画確定まで完了した。独立仕様・設計一括レビューは親伝達でpass、仕様/設計blockerはない。残工程は独立RED fixtureの表内コードpipe escape修正/期待値固定 → 親の着手判断 → U6a → U6b → U6c → 最終runner → 独立reviewerである。レビューで契約・所有が変わった場合だけ該当箇所を修正する。親は通常補助1役ずつfresh contextで起動し、共有ファイルと同居 `cfg(test)` は直列所有とする。

3単位は受入と所有を識別する区分であり、共通実装者・テスト作成者を再利用する。計画Skillの約5ファイル指針は実装の各単位を小さくするために適用する。共通test/visual fixtureを全単位で再利用するため、工程全体のユニークファイル数を新たな品質ゲートにはしない。

テスト所有は全単位共通で、test authorが `src/app/gfm_tests.rs`、`src/app/visual_check.rs` のU6合成fixture、`gfm-tests.md` を持つ。既存 `notifications/markdown.rs` の同居testは、仕様が更新する限定parser期待値だけtest authorが順番に変更する。実装者は `src/app.rs` の `#[cfg(test)] mod gfm_tests;` 宣言と、必要な `src/app/harness_tests.rs` の既存helperの最小 `pub(super)` 化を統合してよいが、テスト本体・fixture・期待値は変更しない。private ASTを期待値にせず、共有wrapper/実WordApp UI、出力イベント、app stateと隔離diskを検証境界とする。

### U6a: 通知比較を標準構文で描画する

規模M、難易度/リスク高。依存はG01–G04/G08、設計第1–3節、独立レビュー解除と独立期待値である。

- 本体所有確定: `Cargo.toml` / `Cargo.lock` のWindows parser依存、`src/app/notifications/markdown.rs` のprivate描画、`src/app/notifications.rs` の共有wrapperと通知接続の4ファイル。テストは上記共通所有を使う。
- 受入: G01/G02の通知2本文・安全な構文描画、G03/G08の原文/コピー/厳密検査不変、G04の実太字・表/長文到達を満たす。既存通知状態を維持する。
- 検証: 仕様固定例の描画構造/可視文字と原文一致を別判定する独立回帰、通知状態回帰、隔離描画。focusedコマンドはtest authorが実在名で固定し、0件実行を合格にしない。

### U6b: 英語チャットAI回答へ同じ描画を適用する

規模M、難易度/リスク中～高。依存はU6aの描画契約・focused成功である。通知限定からの追加差分は回答側描画入口、選択/コピー、可変長回答のレイアウト検証である。

- 本体所有確定: `src/app/chat_ui.rs` のAI回答本文分岐、必要時だけ `src/app/notifications.rs` の共有入口接続。テストは共通所有を使う。
- 受入: G01/G02の両画面共通構文、G03/G08の保存原文/既存操作保持、G04の回答長・表・コード・狭幅操作を満たす。通常のユーザー発言表示は変えない。
- 検証: 同一の固定Markdownを両画面へ投入した差異検査、回答更新時の古い描画状態の残留検査、標準/狭幅の隔離Windows描画・選択・コピー。応答生成は合成データを用いる。

### チェックポイント: 両画面の描画契約

- [ ] G01–G04/G08の新証拠を親が確認し、原文保持と見た目の正しさを別々に判定した。
- [ ] 依存/version/featuresとローカル表示境界、共有ファイル所有を確定した。全GFMを満たさない場合の製品説明も仕様と一致する。

### U6c: 教材登録の結果を保存結果に連動させる

規模M、難易度中・失敗時リスク高。仕様/設計上はU6a/bと独立するが、共有する通知本体を直列に変更するためU6b後に実施する。

- 本体所有確定: `src/app.rs` のmaterial_panel登録結果・receipt field/初期化/dialog表示、`src/app/notifications.rs` のreceipt型・設定/描画、`src/app/chat_ui.rs` のheader直後receipt表示の3ファイル。テストは共通所有を使う。`store.rs` / `material.rs` / `commit.rs` の保存・検査本体は所有外である。
- 受入: G05の両画面成功表示・自動dialogなし、G06の拒否/保存失敗/復旧区分、G07/G08のreceipt寿命・承認・データ保護を満たす。
- 検証: 実material_panel経路の成功、登録前検証失敗、隔離先への保存前失敗とpending intentを残す途中失敗、直前エラー→成功、成功→次の失敗、契約上可能な再試行。UI通知状態と保存ファイル/案/復習状態の前後比較を独立期待値とする。内部helperだけの成功試験で画面経路を代用しない。

### 最終チェックポイント

- [ ] 独立test authorのG01–G08期待値とfocused回帰が成功した。parser出力やprivate ASTを使って期待値を自己生成していない。
- [ ] runnerが最終未コミット差分の識別情報と新規U6ログを固定し、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` を成功させた。旧U5ログを流用しない。
- [ ] `git diff --check` と対象の整形確認、標準/狭幅の隔離Windows描画・操作を確認した。画像だけでclipboard一致を主張しない。
- [ ] 独立レビューが構文適合、入力由来の外部取得、原文/コピー、保存失敗・通知state、期待値の独立性を確認した。指摘修正は所有役へ戻し、影響する検証だけ再実行した。
- [ ] 親が短いcurrent更新、使用量の範囲/欠測、残る受入を記録した。実データ・実AI・公開は未実施と区別した。

## 7役のモデル・effort・選定理由

計画者と外部仕様担当は全単位 `gpt-6-astra / high` 固定である。以下は親が示した利用可能exact IDと対応effortによる起動指定であり、実行値の証拠ではない。親はdispatch時に可用性/対応effortを照合し、実行メタデータを別記録する。未取得は未取得とし、モデル/Skill不在はブロッカーとする。親モデルと品質ゲートは変更しない。

| 単位 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- |
| U6a | `gpt-6-astra / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-astra / high` |
| U6b | `gpt-6-astra / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-astra / high` |
| U6c | `gpt-6-astra / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-astra / high` |

- U6a: designerは標準仕様・新依存・UI機能制限の未確定な組合せ、reviewerはその適合主張と原文保持の独立判定を担うためAstra/high。implementerは設計確定後の局所接続、test authorは既知構文例と境界例の独立期待値を担うためSol/highである。
- U6b: designer/reviewerはU6aと同一のAstra/highを再利用し、両画面の共通描画契約と原文保持を一度の設計/レビューで扱う。implementerは既存回答描画の接続、test authorは回答更新/コピー/レイアウトの回帰に限定するためSol/highである。
- U6c: designerはU6全体と同一のAstra/highで保存完了・部分保存・通知状態を一貫して設計する。implementerは既存保存APIの結果配線、test authorは失敗時の保持/復旧契約に基づく独立期待値を担うためSol/high。reviewerは保存失敗を成功扱いする危険と再試行データ保持を独立評価するため同一Astra/highを再利用する。
- 全runnerは固定fixture・コマンド・隔離Windows証拠の実行/報告に限定するためTerra/medium。判断困難な差異は設計/親へ戻し、期待値やソースを直さない。仕様/設計は同一の独立reviewer Astra/highによる一括確認、最終差分は同reviewerの別ターンとし、実装者との独立性を保つ。最終runnerはTerra/mediumとする。モデル名を実測品質の保証とみなさない。

## 文書と役割の所有

| 役 | 所有・出力 | 必要Skill |
| --- | --- | --- |
| planner | 本書のみ、仕様後と設計後の更新 | `agent-skills:planning-and-task-breakdown` |
| external-spec author | `gfm-spec.md` の利用者動作/旧仕様との差分 | `agent-skills:spec-driven-development` の仕様工程 |
| designer | `gfm-design.md` の依存・共有描画・状態・保存結果契約 | `agent-skills:api-and-interface-design`、`wordweave-egui-ui` |
| implementer | 各単位の本体節のみ | `wordweave-change`、`wordweave-egui-ui` |
| test author | `gfm-tests.md`、`src/app/gfm_tests.rs`、visual U6 fixture、明示した既存同居testの仕様差分のみ | `agent-skills:test-driven-development` の作成工程 |
| test runner | `gfm-results.md`、U6 logs/artifacts、隔離データ、build生成物のみ | `wordweave-change` の検証工程 |
| reviewer | read-only、親が既存 `reviews.md` のU6節へ結果を保存 | `review-external-specification` / `review-software-design` / `agent-skills:code-review-and-quality` を担当対象に応じ使用 |

## リスク・未決事項・次の引継ぎ

1. 構文/改行/画像/HTML/リンクの契約は確定したが、rendererがそれを満たすかは未検証である。parser導入だけで全GFM準拠としない。
2. 大量セル/深い入れ子の描画負荷、狭幅receipt、font fallbackによる太字不足は実装時リスクである。G04の実描画と末尾操作で判定し、受入削減で解消しない。
3. 保存途中失敗はメモリ復元だけでは未反映を証明できない。pending/fatalを残す隔離失敗fixtureと保存ファイルを照合し、成功や先行再登録案内を出さないことを確認する。
4. dirty差分の比較基準は親が保存済みである。所有外の保存責務変更や新たな契約変更が必要なら親へ返す。現時点で未決の利用者質問はない。

次の引継ぎは親によるtest authorのRED fixture修正確認と、`ww-implementer` の `gpt-6-sol / high` によるU6a→b→cのdispatchである。独立レビューが返した表内コードpipe escapeのP2はtest authorが所有し、実装者は期待値を都合よく修正しない。契約変更がなければ計画の再読・再確定工程を繰り返さない。本書の確定は実装着手の親判断を代行しない。

計測: KPIと計画Skill/参照DoD、AGENTS、development-teamを読了。子の実行model/effort、開始/終了tokenカウンター、親子包含は未取得。親が保持する開始カウンターは親範囲の測定として扱い、本子へ割り当てて推定しない。初期shellはsandbox初期化エラー、read-only再試行で必須文書を取得した。計画者の完了境界は本書の静的確認と親への引渡しである。
