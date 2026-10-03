# FEEDBACK-UI-001 U5 通知可読性の追加計画

2026-09-28。状態: **仕様・設計に基づく計画/所有/API境界を確定。設計の独立レビュー完了は親が着手前に確認する**。利用者の構造化通知案への実装承認は親から伝達済みである。再承認は要求拡張が生じた場合に限る。親が本計画を確認して各役を起動する。plannerの編集所有は本書だけであり、作業リストも本書に置く。既存 `plan.md` / `todo.md` とルート `tasks/plan.md` / `tasks/todo.md` の未完了項目を上書きしない。

## 仕様・設計後の確定事項

入力は [notice-spec.md](notice-spec.md) と [notice-design.md](notice-design.md) である。親の引継ぎによれば仕様は親承認済み・独立レビュー指摘0、設計は完成・独立レビュー中である。仕様文書の旧「レビュー待ち」表示とこの引継ぎを区別する。以下を暫定節の候補・未決事項より優先する。モデル割当、既存品質ゲート、他の未完了項目は変更しない。

- 仕様: 追加仕様N01–N08を正とし、旧Q03/Q05等との優先関係は同仕様第1節による。比較/技術情報は初期閉、一括Raw切替、同一通知内の再展開はモード保持、新規/同文再通知/再表示は初期化する。診断コピーは240 Unicode scalar抜粋、各原文コピーは全文の完全一致とする。新外部依存を追加しない。
- API: 設計第1・2節の `MaterialFailure::{Other, Evidence}`、`MaterialDiagnostic`、`DiagnosticStage`、`EvidenceCause`、`QuoteEvidence` の正確な型/field契約を採用する。`Request::build_response_detailed(&str)`、`Draft::validate_evidence_detailed()`、`Config::material_detailed(Request, Vec<Vec<u8>>)` を追加し、既存String APIは `legacy_message()` へ変換する薄いwrapperで維持する。検査を一本化し、平文を再parseしない。
- 非同期: 共通 `Pending.rx` の `Result<AiResult, String>` は維持する。教材専用の `AiResult::MaterialFailure` で型付き失敗を渡し、既存失敗整理・pending識別/取消し条件を保持する。`Other` は一般通知、`Evidence` だけを構造化通知へ渡す。
- 保存案: 毎frameの既存 `Draft::ready` と登録disabledを維持する。明示「理由を詳しく確認」で同じ案の `validate_evidence_detailed()` を実行し、一致する根拠失敗だけ構造化する。一般失敗とStorage読込はString互換を保ち、`src/store.rs` を変更しない。
- 通知: 設計第2・3節の一時 `MaterialNotice`、`notify_material_diagnostic`、状態に依存しない `diagnostic_copy_text` を採用する。別通知/fatalへの切替では古いpayloadを無効化し、本文表示/コピーで原文や教材を変更しない。
- 所有: U5aは `material.rs` / `ai.rs` / `app.rs` / `app/notifications.rs` の4本体、U5bは `material.rs` / `app.rs` / `app/notifications.rs` の3本体、U5cは `app/notifications.rs` / 新規 `app/notifications/markdown.rs` の2本体に確定する（すべて `src/` 配下）。独立testは各単位に既記載の同居test/H/Vのみ。ユニーク上限はa=5、b=4、c=4であり共有ファイルは直列所有である。
- 設計の限定Markdown（見出し/段落/実改行/太字/斜体/箇条書き/コード）と文字を失わないfallbackを採用する。rendererの内部構造は設計裁量であり、新依存や汎用Markdown導入へ広げない。

次の着手条件は親による設計独立レビュー結果の確認・阻害指摘解消と、独立test authorによる期待値固定である。設計レビューで契約変更が生じた場合は影響する所有/APIだけ本書へ反映してからdispatchする。現時点で新たな利用者判断は不要である。詳細契約は設計を参照し、本書に重複転記しない。

## 完了境界・根拠

教材引用検査失敗の通知を、初期要約、展開式の比較、Markdown表示と原文切替、折り畳んだ技術情報に分ける。全文を確認・コピーでき、既存一般通知と設定誘導、厳密引用判定、教材登録安全性が回帰しない状態を対象とする。独立テスト、隔離Windows描画・操作、最終全対象テストとreleaseビルド、独立レビューまでが開発完了境界である。実データ・実AIによる利用者受入は模擬証拠と区別する。

- 規約/手順: `AGENTS.md`、`docs/process/development-team.md`、`docs/process/improvement/kpi.md`、`agent-skills:planning-and-task-breakdown` と参照Definition of Doneを読了。
- 基準HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。多数の既存未コミット変更を含む。親はU5開始差分/hashを保存し、HEADとの差分全体をU5成果とみなさない。
- 既存仕様 [spec.md](spec.md) Q01/Q02/Q04/Q06、設計 [design.md](design.md) 第5節の検査順序・固定snapshotを再利用する。配色C01–C09と上限L01–L05は変更しない。
- **仕様差分が必要**: 旧Q03は240 Unicode scalar previewと全文機能非提供、旧Q05はMarkdown非解釈を要求している。今回の利用者承認を根拠として表示契約を更新し、厳密検査の期待値は保持する。旧仕様に合格したU4を今回の受入証拠へ流用しない。
- ソース確認: `material.rs::quote_preview` はJSON escapeした240文字をflat診断へ埋め込む。`Request::build_response` と `Draft::validate_evidence` は `Result<_, String>`、`ai.rs::Config::material` とAI結果通知でも文字列経路を使う。`Draft::ready` は登録直前にも根拠を再検査する。`notifications.rs` は選択可能Labelと本文スクロール・固定footerを持つ。
- 親の読取調査: 既存Markdown renderer/依存はない。現在の日本語診断の再parseでは失われた全文を復元できない。検査時に構造化診断を保持し、対象経路だけに追加する設計が必要である。

## 要求・受入条件

IDは `FEEDBACK-UI-001-N01` 等であり、Q01–Q06との対応を外部仕様担当が明記する。

| ID | 要求と観測可能な受入条件 |
| --- | --- |
| N01 | 通知を開いた初期状態に、明確なタイトル、失敗した段階/登録していない結果、教材が変更されていないこと、平易な理由、次の操作を示す。内部path・理由番号・roleを主見出しにせず、生成失敗と保存案の再検査失敗を混同しない。 |
| N02 | 「引用の比較」相当の明示操作でAI引用と作成要求時の固定元発言を別領域に展開する。最初の失敗理由/対象/話者/往復との対応を保持し、未選択往復・不正話者・欠落snapshotは取得不能と示す。別の現在チャットで補わない。 |
| N03 | 引用と元発言の両方を、太字・段落・箇条書きが読めるMarkdown表示にする。原文表示へ切り替えると元の文字列を正確に取得でき、`**`、改行、空白、Unicode等を勝手に修正しない。読みやすい表示が厳密一致の証拠ではないことを明示する。 |
| N04 | 固定元発言の全文へ到達できる。240文字以後と末尾も表示/原文コピーで確認でき、抜粋だけを全文と称しない。全文取得のための実AI再送信や現在会話の再取得は行わない。 |
| N05 | 技術識別情報は初期状態で折り畳み、必要時に読める。診断の選択と「詳細をコピー」を維持し、コピー内容は折り畳みや表示モードに依存せず仕様で固定する。原文専用コピーは引用/元発言の元文字列と一致する。 |
| N06 | 一般通知、fatal/font情報、Codexパス誘導、日次上限誘導を保持する。別通知への置換・同文の再通知・閉じる/再表示で、以前の構造化診断や開閉状態を別通知へ混入させない。 |
| N07 | 空白のみ、2000文字超、厳密contains、大文字小文字/空白/改行/Markdown記号、理由/引用件数とpath実在検査、その順序を維持する。表示・切替・コピーだけで再試行、教材登録、既存教材/復習状態変更を起こさない。成功時も案・可視差分・学習者承認を必要とする。 |
| N08 | 標準1150×950/80%と狭幅820×650/160%で要約・展開・切替・末尾スクロール・コピーを操作できる。長文でfooterが画面外へ押し出されず、重なりや読めない日本語がない。表示はローカルな読取専用の範囲であり、本文由来の外部資源取得や実行を追加しない。 |

## 対象外

全アプリ/チャットへのMarkdown導入、一般通知の全面型変更、永続schema刷新、認証/AIモデル/effort/Volta/PATH変更、生成枠の緩和、引用の正規化・自動修復、教材の自動登録、実データ/実AI試験、無関係差分整理、harness設定/quality gate変更、commit/push/公開は対象外である。新しいMarkdown依存やネットワーク機能を暫定計画だけで導入しない。依存が必要なら設計で具体的な最小案と所有ファイル数・検証を示して親へ返す。

## 工程・依存・チェックポイント

暫定計画の親確認 → Astra外部仕様差分 → 独立仕様レビュー → 計画精緻化 → 内部設計差分/独立設計レビュー → 所有/境界を確定した計画 → 独立テスト作成 → U5a → U5b → U5c → 最終runner → 独立reviewer。

親は通常補助1役ずつ、fresh contextで起動する。子の再委任は禁止する。共有ファイル本体と `cfg(test)` も直列所有である。親は補助稼働中に未重複のソース確認、差分基準保存、証拠整理を行う。既存仕様・設計の非変更節は再作成しない。各単位の実装者はfocused検証まで行い、独立runner/reviewerと兼任しない。

### U5a: 生成失敗から構造化比較へ到達する

規模: 中、難易度/リスク: 高。文字列化前の診断と非同期結果境界を扱う。依存: 更新仕様、承認済み設計、独立した期待値。

- 本体所有候補: `src/material.rs` の診断生成/生成response境界、`src/ai.rs` のmaterial結果境界、`src/app.rs` の教材AI結果と一時通知state、`src/app/notifications.rs` の構造化通知設定/基本表示。独立testは同ファイルの `cfg(test)` と `src/app/harness_tests.rs`。ユニーク計5ファイル上限。
- 受入: N01/N02/N04の生成経路が実際に通知へ届き、全文を保持する。N06の一般通知へ切り替えた時に構造化内容が残らない。N07の生成検査順序・安全性を保持する（3まとまり）。
- 検証: 合成Requestの最初の失敗/存在しない参照/長文/Unicode回帰、AI結果→実WordApp通知経路、一般通知への置換、読み取り/コピーでのデータ不変。focused `cargo test --locked <固定したU5フィルター>` と関連ターゲットのコンパイルを確認する。フィルターはtest authorが実在名で固定し、0件成功を合格としない。
- 設計制約: 日本語flat診断の再parseでpayloadを作らない。全Result型を連鎖変更せず、旧文字列呼出元と型付き診断の共存/変換境界を明記する。両者の検査ロジックを二重実装しない。UIはこの時点でも動作可能な要約/原文比較を持ち、MarkdownはU5cで完成させる。

### U5b: 保存案の再検査失敗にも同じ診断を接続する

規模: 中、難易度/リスク: 高。登録直前の安全境界と保存エラーの分類を扱う。依存: U5aの診断契約とfocused成功。

- 本体所有候補: `src/material.rs` の保存案診断/ready境界、`src/app.rs` の登録前検査・失敗通知、`src/app/notifications.rs` の段階別表示のみ。独立testは同ファイル `cfg(test)` と `src/app/harness_tests.rs`。ユニーク計4ファイル上限。
- 受入: 保存案でもN01/N02/N04を満たし、保存済み固定sourceから全文を得る。N07の再検査/登録防止と既存案・教材・復習状態を保持する。一般storage失敗等を引用失敗へ誤分類しない（3まとまり）。
- 検証: 合成保存案の復元→根拠/ready失敗→通知、現在会話だけ変更したケース、不正sourceが理由検査より先に拒否されるケース、登録前後/隔離ディスクの不変検査。既存保存/登録回帰とU5 focusedを実行する。
- storage読込時のStringエラーは既存互換経路を維持するのを暫定案とし、構造化通知の適用入口を仕様/設計で列挙する。読込時まで必要と確定した場合は `src/store.rs` を無断追加せず所有と単位を再計画する。

### チェックポイント: 構造化診断の契約

- [ ] 生成時/保存案時の2入口が固定全文と正しい段階を伝え、一般通知とのstate分離、厳密検査、登録安全性のfocused証拠を親が確認した。
- [ ] 本体/testの差分が所有内であり、後段UIが日本語の文字列構造に依存していない。

### U5c: 比較のMarkdown・原文切替と段階的な表示

規模: 中、難易度/リスク: 中～高。見た目の同一性と原文の同一性を分離し、狭幅/長文/選択コピーを扱う。依存: U5a/bの固定payload契約。

- 本体所有候補: `src/app/notifications.rs` と、設計で必要と確定した時だけ通知専用renderer `src/app/notifications/markdown.rs`（既存module内に限定する）。独立testは同ファイル `cfg(test)`、`src/app/harness_tests.rs`、`src/app/visual_check.rs` のU5合成シナリオ。ユニーク計4ファイル上限。追加依存のmanifest/lock編集は現所有に含めない。
- 受入: N03/N04/N05の両比較対象Markdown・正確な原文・全文・独立コピーが成立する。N01/N02/N05の初期要約と折り畳み比較/技術情報が成立する。N06/N08の既存通知/設定誘導と標準/狭幅操作が成立する（3まとまり）。
- 検証: 太字、複数段落、箇条書き、未閉鎖/混在記号、改行/CRLF/タブ/前後空白/絵文字、240文字以後・末尾の固有語、引用なし/元文なし、同文再通知、閉じる/再表示。レンダリングの文字とRawの完全一致を別々に検査する。一般通知/日次上限/パス誘導のfocused回帰を実行する。
- 実描画: 1150×950/80%・820×650/160%で初期状態、比較展開、Markdown、原文、技術展開、末尾到達、コピー操作の隔離Windows証拠を残す。単なるスクリーンショットだけでクリップボード内容一致を主張しない。

### 最終チェックポイント

- [ ] 全N01–N08と影響するQ01–Q06/L01–L05を、独立テスト/Windows証拠/結果へ対応付けた。C01–C09は配色互換を維持し、通知の白い読取専用面等の既存境界を確認した。
- [ ] 最終差分を固定し、新規専用ログで `cargo test --all-targets --locked` が成功した。
- [ ] 同じ最終差分で `cargo build --release --locked --bin wordweave5` が成功した。
- [ ] `git diff --check` と対象整形確認、標準/狭幅描画・操作が成功した。既存無関係な整形差分を混ぜていない。
- [ ] 独立レビューで型付きpayloadの伝搬、snapshotの所有、厳密検査、登録安全性、通知stateの陳腐化、原文コピーを確認した。指摘の修正は所有役へ戻し、影響する検証を再実行した。
- [ ] 親が短いcurrent更新、使用量の取得範囲/欠測、実機/実AIの残る受入、未公開を記録した。旧343件成功や旧releaseログをU5の検証として転用していない。

## 7役のモデルとeffort

以下はホストの呼出可能一覧に存在する正確なmodel IDと対応effortによる**起動指定**である。実行メタデータではない。起動担当が各回に一覧/effort対応を照合し、取得できた実行値を別記録する。未取得値は未取得とし推測しない。モデル/Skill不在はブロッカーでありsilent fallbackしない。親のmodel/effortと品質ゲートは変更しない。

計画は全単位 `gpt-6-astra / high`、外部仕様は全単位 `gpt-6-astra / high` 固定。前者は境界/所有/依存調整、後者は旧Q03/Q05との矛盾解消と観測可能な表示契約の確定を担う。

| 単位 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- |
| U5a | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-sol / high` |
| U5b | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-sol / high` |
| U5c | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-6-sol / high` | `gpt-5.6-terra / medium` | `gpt-6-sol / high` |

- U5a: designerは既存String境界と非同期payloadの限定設計、implementerは既存汎用結果を保つ配線、test authorは生成失敗/状態更新の独立期待値、reviewerは検査と表示の分離を分析するためSol/highである。runnerは固定合成ケースと明示コマンド/ログ取得に限定するためTerra/mediumである。
- U5b: designer/implementerは登録直前・保存案の失敗順序と副作用、test author/reviewerは不正sourceや永続状態の不変証拠を扱うためSol/highである。runnerは既定fixture・不変比較・コマンド実行のためTerra/mediumである。
- U5c: designer/implementerは新しい限定Markdown表示と選択・raw・コピー・狭幅の相互作用、test author/reviewerは読める表示とバイト列保持の独立判定を扱うためSol/highである。runnerは固定視覚シナリオを実行し差異を報告するTerra/mediumであり、設計解釈が要る差異は親/設計役へ返す。
- 仕様レビュー/設計レビュー/最終統合レビューは独立した `ww-reviewer` の `gpt-6-sol / high`。最終runnerは `ww-test-runner` の `gpt-5.6-terra / medium`。役は実装と独立させ、同時常駐させない。Lunaへの単純化は原文保持と通知stateの判定が複合するため今回は選ばない。

## ファイル/工程所有と必要Skill

| 役 | 所有・出力 | 必要Skill |
| --- | --- | --- |
| planner | 本書だけ。外部仕様・設計後の再割当/チェックリスト更新 | `agent-skills:planning-and-task-breakdown` |
| external-spec author | `tasks/FEEDBACK-UI-001/notice-spec.md` のU5差分/AC対応だけ。既存非変更要求を保持 | `agent-skills:spec-driven-development` の仕様工程 |
| designer | `tasks/FEEDBACK-UI-001/notice-design.md` のU5境界/状態/表示契約だけ | `agent-skills:api-and-interface-design`、`wordweave-egui-ui` |
| implementer | 上記単位の本体節だけ。独立test/期待値を変更しない | `wordweave-change`、UI単位では `wordweave-egui-ui` |
| test author | `tests.md` のU5対応、指定 `cfg(test)` / H/VのU5合成fixtureだけ | `agent-skills:test-driven-development` のテスト作成工程 |
| test runner | `results.md` U5節、U5専用logs/artifacts、隔離データ、build生成物だけ | `wordweave-change` の検証工程 |
| reviewer | read-only。親が `reviews.md` U5節へ報告保存 | コードは `agent-skills:code-review-and-quality`、仕様は `review-external-specification`、設計は `review-software-design` |

スキルは利用可能一覧から名前で解決し全文を読む。未検出は親へ戻す。全書込み担当に共有作業ツリー・他者差分を戻さない制約を渡す。追加ファイルや5ファイル超過が判明したら根拠と再分割をplannerへ返す。

独立test authorは旧動作で実行できる利用者観測テストのREDを先に残す。存在しないAPIによるcompile-failをRED件数に含めない。新境界が必要なテストは期待値を先に固定し、実装後の独立追補と事前REDを区別する。旧240文字/非Markdown表示テストは承認済み仕様差分に対応する表示期待値だけを更新し、厳密照合テストを都合よく緩めない。

## リスクと未決事項

| リスク | 対応・精緻化の担当 |
| --- | --- |
| flat文字列から構造を復元して全文欠落や誤対応を残す | designerが診断発生時の型とString互換境界を固定。U5a独立回帰で全文/理由対応を検査する。 |
| Markdown上は一致して見えるが原文は不一致 | 外部仕様でRaw/原文コピーと短い説明を定義し、validatorは元文字列だけで判定する。 |
| 失敗通知が一般通知へ漏れる、同文再通知の開閉stateを誤る | designerが通知識別・置換・初期化・優先順位を定義し、別通知/fatal/font/再表示の回帰を作る。 |
| 保存案エラーを表示するための再検査が検査順序や永続化を変える | U5b設計で呼出位置と失敗時無変更を固定する。store全体の型変更は避け、必要性を再判断する。 |
| 限定Markdownが暗黙の汎用parserへ拡大する | 必須の太字/段落/箇条書きと未対応構文のfallbackを仕様化する。HTML/画像/リンク取得/実行は追加しない。大きい文の描画は全文到達を維持した上で設計する。 |
| 長文・狭幅が操作とコピーを隠す | 既存固定footerと本文scrollを保ち、初期/展開/末尾の実描画・コピー内容を検証する。 |

仕様担当が確定する事項: 比較の初期開閉、Raw切替の単位、詳細コピーの正確な内容、取得不能の表示、サポートするMarkdownの範囲と未対応構文の表示。要求の範囲内で決定でき、利用者への追加質問は現時点で不要である。

設計担当が確定する事項: 型付き診断を流す最小API/非同期結果境界、旧String互換、保存案に適用する入口一覧、通知の識別とstate破棄、限定rendererの配置/依存要否。これらは現在の実装承認を取り直す理由ではないが、所有/単位を確定する前に必要である。

## 作業リストと次の引継ぎ

- [x] U5暫定計画・要求・対象外・候補所有・モデル割当を記載する。
- [x] 外部仕様役 `gpt-6-astra / high` が旧Q03/Q05の更新とN01–N08を具体化し、親承認・独立レビュー指摘0を確認した（親引継ぎ）。
- [x] plannerが追加仕様・完成設計を読んで、仕様AC・単位・正確なAPI・本体/testの所有を確定した。
- [x] designer `gpt-6-sol / high` が最小payload境界と表示/状態を設計した。
- [ ] 親が進行中の独立設計レビューの終了/阻害指摘解消を確認し、必要な契約変更だけplannerへ返す。
- [ ] 独立test author → U5a → U5b → 構造化契約チェックポイント → U5cを直列実施する。
- [ ] 固定最終差分のrunner、独立reviewer、必要な指摘修正/関連再検証、親の最終報告を完了する。

次のdispatchは設計レビュー確認後の独立test author `gpt-6-sol / high` である。本書、`notice-spec.md`、`notice-design.md`、U5aの限定所有と既存String互換/厳密検査の期待値を渡す。plannerは実装/テスト編集・子起動・公開を行わない。今回の文書変更検証は本書のdiff/必須節/リンク先と所有境界の確認に限定し、アプリのbuildは行わない。

計測: 本計画役の開始/終了tokenカウンター、実行model/effortメタデータ、親子カウンター包含は未取得であり推定しない。親がタスク計測へ欠測を記録する。通常shell起動はACL適用エラーで失敗し、同じ読取専用確認の昇格実行は成功した。計画文書だけを変更した。
