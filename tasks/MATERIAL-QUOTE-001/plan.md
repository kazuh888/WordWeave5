# MATERIAL-QUOTE-001 実装計画

状態: 実装・自動検証・release配置完了 / 2026-09-27（親による最終結果統合）。仕様/設計・最終独立レビューpass、material 16/16・通知対象1/1・全対象298件成功、releaseビルド成功をresults.md/reviews.mdで確認した。実機UI・実AIは未実施の別受入であり、これらを含む全体受入完了とは扱わない。commit/pushなし。

## 完了境界と根拠

「チャット→教材案を作成」の引用拒否について、コード上の拒否条件を説明し、拒否対象の元発言と引用文を利用者が識別できるようにする。範囲は局所診断、表示改善、独立回帰、全対象テスト、Windows releaseビルド、独立レビューまでである。実際のAI応答の原因特定と実機受入は、証拠が得られない場合に未確認として残す。

- 基準HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 未コミット差分。各引継ぎで親が差分対象を固定する。
- 確認済み: `src/material.rs` の `Request::build_response` は、選択snapshotと話者の存在確認後、空白のみ・2000文字超・元発言への非包含を同じ「元の発言に存在しない引用です。」で拒否する。発言そのものの欠落とは限らない。
- 確定診断: 親が最新の実行記録2件を限定読み取りし、対象語 improve・訂正・選択4往復に一致する記録を確認した。4往復目assistantの返却引用が原文のMarkdown強調記号を省略し、8件の不一致を起こしていた。原発言は存在する。根拠は [diagnosis.md](diagnosis.md) に集約し、本担当は実ログを再収集しない。実ログ本文をrepository/fixtureへ複写しない。
- 既存 `tasks/plan.md` と `tasks/todo.md` には他作業・実機受入の未完了項目があるため保持する。本タスクの正本は本書と [todo.md](todo.md) である。

## 要求と除外

| ID | 要求 / 受入基準 |
| --- | --- |
| R1 / AC1 | 引用不一致と、選択していない往復・不正な話者・空引用・過長引用を区別できる説明にする。元発言が消失したと誤認させない。 |
| R2 / AC2 | 引用不一致に、元会話に対応する往復番号、話者、拒否された引用断片を表示する。番号は選択リスト内順位と混同しない。元発言の確認手段を示す。省略範囲と長文表示は仕様で確定する。 |
| R3 / AC3 | 厳密な照合を維持する。正しい引用は受理し、別往復・別話者・改変/非包含・空・過長引用は拒否する。正規化、類似照合、自動引用修復による受理拡張をしない。 |
| R4 / AC4 | 拒否時に教材を登録しない。正常時も案→可視差分→学習者承認の境界を保持する。追加/訂正の意味、既存教材・学習状態を保持する。 |
| R5 / AC5 | 合成fixtureで同種失敗と改善表示を再現する。今回の利用者payloadに対する確定診断と、コードから分かる拒否条件を報告で分離する。 |

受入の詳細は [spec.md](spec.md) のMQ-AC-01～10を正本とする。R1→01/08、R2→02/04/05/09、R3→03/06/08、R4→07/08、R5→10として追跡する。生成指示はMarkdown・改行・空白等を保つ連続部分引用を明確化する。保存済みDraftにも同じ拒否理由と表示契約を適用する。

対象外: 実学習データの編集/試験、live AI 呼出、認証/PATH/返却model表示の変更、永続形式変更、新規logging/固定原文全文viewer、機能全体の再設計、harness/設定/品質ゲート変更、commit/push/公開。

## 作業単位と依存

1. U1「引用拒否の説明」: 生成応答と保存済みDraftの拒否理由・元発言/引用表示、および生成指示を改善する。想定S～M、`src/material.rs` と `src/ai.rs`、対応する独立テストのみ。既存エラー通知を再利用する。難易度中、リスク中～高。出典の誤認と誤受理が学習内容の根拠に直結する。
2. U2「教材化経路の受入」: U1の合成失敗と正常経路を教材案画面で確認し、非登録・可視性・全対象/release検証を行う。実装変更は原則なし。画面修正が必要なら設計へ戻し、最大5ファイルの境界と所有を更新する。難易度中、リスク中。画面可視性と非登録の判定は単体文字列検査だけでは不十分である。

依存順は、親の限定診断 → 外部仕様 → 独立仕様レビュー/利用者確認状況の記録 → 計画更新 → 内部設計 → 独立設計レビュー → 計画確定/親のdispatch判断 → 独立テスト作成(red) → 本体実装(green) → U2検証 → 独立最終レビューである。承認の充足は親が既存の利用者指示も含め判断し、子が推定しない。重大な仕様不明点は実装で補完せず仕様へ戻す。

並列化は親の読み取り診断と計画のみとする。通常は子1役ずつを新規文脈で起動する。同一Rustファイルの本体と `cfg(test)` は別役が順番に編集する。

## モデル割当

呼出可能一覧は親から受領した exact ID とeffortに基づく。以下は起動指定であり実行確認値ではない。実行metadataは親が別記録し、取得不能なら「未取得」とする。親のモデル/effortは変更しない。起動前に一覧・役定義・必要Skillを再照合し、不在はブロッカーとして戻す。

固定役: planner = `gpt-6-astra` / `high`、external-spec author = `gpt-6-astra` / `high`。誤認を避ける観測可能な要求と工程境界の確定を担う。

| 単位 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- |
| U1 | `gpt-6-astra` / `high` | `gpt-6-sol` / `high` | `gpt-6-sol` / `high` | `gpt-5.6-terra` / `medium` | `gpt-6-astra` / `high` |
| U2 | `gpt-6-astra` / `high` | `gpt-6-sol` / `high` | `gpt-6-sol` / `high` | `gpt-5.6-terra` / `medium` | `gpt-6-astra` / `high` |

- Designer: 難易度中/根拠喪失リスク高。生成応答・snapshot・画面番号・表示制限の契約を横断するためAstra/highを選定する。U2では既存設計の受入経路への適合を担当する。
- Implementer: 難易度中/リスク中。確定契約に沿う局所Rust変更でありSol/highを選定する。U2で修正不要なら役を識別したまま実装非該当と記録する。
- Test author: 難易度中/誤受理リスク高。正常・異常と正規化非許可、番号・話者の取り違えを独立期待値で検証するためSol/highを選定する。
- Test runner: 難易度低～中/判定漏れリスク中。固定差分への既定コマンドと合成データでの証拠収集に限定しTerra/mediumを選定する。テストや期待値を修正しない。
- Reviewer: 難易度中～高/安全境界リスク高。仕様/設計/最終差分の独立確認にAstra/highを選定し、検証緩和・誤参照・非登録の証拠不足を確認する。

モデル名は性能実測や合格の根拠ではない。役の責任とリスクに応じた選定であり、同一モデルのauthor/implementerも別文脈で期待値の独立性を守る。

## 所有権と引継ぎ

| 担当 | 編集を許可する成果物 |
| --- | --- |
| planner | `tasks/MATERIAL-QUOTE-001/plan.md`, `todo.md` のみ |
| external-spec author | 同ディレクトリ `spec.md` のみ |
| designer | 同ディレクトリ `design.md` のみ |
| test author | 同ディレクトリ `tests.md`、`src/material.rs` と `src/app/notifications.rs` の対象 `cfg(test)` 節のみ。本体を変更しない。 |
| implementer | `src/material.rs` の非公開引用検証/preview helperと `Request::build_response` / `Draft::validate_evidence` の二入口、`src/ai.rs::Config::material` の生成指示のみ。テスト期待値・既存保存/承認/通知本体を変更しない。 |
| test runner | 同ディレクトリ `results.md`、タスク用検証ログ、隔離fixture出力/ビルド生成物のみ |
| reviewer | 読み取りのみ。指摘は親へ返す。自分の指摘を自己修正/承認しない。 |
| parent | 起動・統合・レビュー記録・`tasks/current.md` の短い状態・KPI測定記録。既存未コミット7役設定等を保存する。 |

手渡す情報は作業ID、要求/AC、承認状態、対象差分識別、所有ファイル/節、モデル/effort、Skill、検証と除外、未解決事項である。各役は `docs/process/development-team.md` の該当Skillを利用可能一覧で解決し全文を読む。UI変更時はUI担当Skillも適用する。

## 検証とチェックポイント

- CP1完了: 外部仕様/設計レビューpassと親承認を確認し、番号・話者・表示・最終所有を確定した。各役の起動直前のモデル/effort・Skill可用性照合は引き続き親が行い、不在なら停止する。
- U1: focused regression のred/greenを記録する。`cargo test --locked material::tests` と `cargo test --locked notifications::tests` を対象とし、追加テストの正確な名称とAC対応をtest authorが `tests.md` に記録する。非連続番号、両話者、Unicode、240/241文字preview、2000/2001文字引用、記号/改行/空白、保存済みDraft、無効参照・overflowを合成データで扱う。対象入力を勝手に正規化して受理しない。生成指示は静的差分で確認する。
- U2/CP2: 合成教材案でエラーが読めること、引用元を特定できること、拒否時の教材・学習状態不変、正常時の案/差分/承認境界を確認する。標準/狭画面・拡大表示で通知の可読性・スクロール/コピー到達性を確認し、既存共有コントロールの回帰を確認する。生成開始時の既存利用回数予約まで不変とは要求しない。headless試験とWindowsフォント/DPIの実機確認、実AI検証は区別する。
- 固定最終差分で `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を実行する。整形/静的確認は既存規約に従い対象差分を確認し、無関係な一括整形をしない。結果は新規ログで記録する。
- CP3: 独立レビューがACと変更範囲/回帰証拠を確認し、指摘は担当へ戻す。関連変更後だけ影響する検証を再実行する。実機受入が残る場合は自動検証完了と分離して報告する。

## リスクと未決事項

| リスク | 対策 |
| --- | --- |
| 確認した1事象を全拒否原因へ一般化する | 今回は原文記号省略と確認済みである。空/過長/参照不正等は別条件として扱い、生成指示強化で再発が消えるとは保証しない。 |
| 選択4件を1～4に振り直し原会話と誤対応する | 元exchange_indexに基づく表示契約を仕様で固定し、非連続選択で試験する。 |
| 改善表示に合わせ検証を緩和する | 既存拒否境界を固定し、正常受理と拒否保持の独立テストを用意する。 |
| 長文引用/元発言がエラー欄を占有し引用確認不能になる | 明示的省略・Unicode境界・改行とUI可視性を仕様/設計で扱う。全文の無制限ログ複製はしない。 |
| 保存や承認境界が変わる | 既存フローを保ち、拒否時不変と承認前未登録を検証する。 |

仕様で解決済み: 元会話位置+1の往復番号と「あなた（質問）/Codex（AIの回答）」、引用/作成要求時の元発言を各先頭240 Unicodeスカラー値のエスケープ表記とし、超過時は省略を明示する。元チャットの該当往復/話者へ案内し、編集後は固定本文と異なり得ることを説明する。固定原文全文viewerは追加しない。省略された不一致箇所が通知内で常に見える保証はない。[design.md](design.md) で非公開helperの二入口共有と最終所有を確定済みであり、実装を阻む未決事項はない。

KPI: 本子の開始/終了使用量カウンターと実行model/effortは未取得。親子包含関係も未確認。推定や合算はせず親が取得可能範囲を `docs/process/improvement/measurements.md` に記録する。計画作成のみではapplication rebuildを実行しない。

最終引継ぎ: 親がrunnerの `results.md`（全対象件数・release成果物/hash・ログ）を確認して全体検証と最終受入状態を統合する。追加の計画往復は不要である。REDの範囲は [tests.md](tests.md) に従い、materialの新規5失敗→greenと、実装差分存在後に成功した通知試験を区別する。実機UI・実AIの未実施を自動試験から合格と推定せず、commit/pushはしない。

## 追補 U3: 変更理由の出力形式を揃える

状態: 実装・自動検証・release更新完了 / 2026-09-27（親による結果統合）。利用者が承認したformat-first方針を実装した。仕様/設計/最終独立レビューpass、focused22/22・送信1/1・pattern表・全305件とrelease成功を新規証拠で確認した。実AI/実機受入は別途、commit/pushなし。既存U1/U2の履歴・残る実機受入を保持する。

親の確定診断: 最新improve/訂正/4往復の応答は全引用が有効だが、理由5の `/replacements/0`、理由6の `/examples/0`、理由7の `/examples/1` が既存 `allowed_path` に拒否された。現在のschemaはpathを任意stringとしており、promptは例示にとどまる。原因は厳密な実行時契約と生成側への制約の不整合である。実応答本文は複写せず、親の限定診断を受領証拠とする。

完了境界は、既存の許可パス集合を広げず、prompt・送信JSON schema・生成応答/保存済みDraftのruntime検証を整合させ、不正理由の番号とpathを利用者が識別できることを合成回帰・送信経路・全対象/release・独立レビューで確認するところである。難易度中～高、リスク中～高、規模M（本体2ファイルと独立テスト）である。

| 追補AC | 要件・証拠 |
| --- | --- |
| MQ-F-01 | 許可集合は既存 `allowed_path` のトップレベル列挙項目と `/examples/{index}/{english,japanese,note}`、`/replacements/{index}/{phrase,meaning,conditions}` を基準とする。`/answers` 等の既存トップレベル項目を「leaf」の語から誤って排除しない。要素全体 `/examples/0` 等、未知項目、構造不正は拒否し続ける。添字表現と整数境界の扱いは仕様追補で確定する。 |
| MQ-F-02 | promptは許可形式・要素全体禁止・正しい個別フィールド例を明示し、outbound schemaがpathの構造/列挙を制約する。実送信payloadのoutputSchemaをmock/protocolで検査し、schema定義だけの試験にしない。構造制約とruntimeの実在要素/整数境界検証の責務は設計で区別する。 |
| MQ-F-03 | 生成応答と保存済みDraftで、不正理由を1始まりの理由番号、返却path、具体的原因により識別できる。少なくとも最初の失敗を示し、引用不一致と項目path不正を混同しない。長文/制御文字の表示は既存安全なpreviewを再利用する。正常path・不正path・存在しない項目・説明/引用条件の識別範囲を仕様追補で固定する。 |
| MQ-F-04 | 有効な引用を持つ理由5～7の要素全体パスを合成して拒否と対象表示を再現し、許可末端パスへ分けた正常応答を受理する。保存済み案も同じ契約、旧形式互換性・追加時の既存path対応付け・U1の引用厳密性・案/差分/承認境界を維持する。 |

除外: whole-itemパスの受理、引用正規化、応答の自動補正/理由削除/再試行/登録、auth変更、実データ変更、新規logging、live AI、公開。schema keywordが使えない場合も無制約stringやruntime緩和へ黙って戻さず、サポートされる同等制約を設計で確認できなければブロッカーとして親へ返す。

### 狭い追補と依存・所有

既存spec/designの引用・preview・通知・保存/承認・安全境界はそのまま再利用する。spec authorは `spec.md` にMQ-F-01～04の観測可能契約・番号/path/理由種別と互換性だけを追補する。designerは `design.md` に許可パス定義とschema/promptの同期方法、生成/保存済み二入口の診断共有、送信schema keyword適合と試験入口だけを追補する。外部仕様→限定独立レビュー→設計追補（親のschema/protocol調査を入力）→限定独立レビュー→独立red→実装green→runner→最終独立レビューの順とする。承認済み要求の再確認は求めず、実際の範囲拡大のみ親へ戻す。

| 役 | U3指定 model / effort | 所有・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | plan/todoの本追補のみ。固定役、既存契約・依存・所有の整合。 |
| external-spec author | `gpt-6-astra` / `high` | spec.mdの追補のみ。固定役、leaf用語と既存受理集合/互換性の曖昧さを解消。 |
| designer | `gpt-6-astra` / `high` | design.mdの追補のみ。schema/protocol/runtime横断の不確実性と範囲拡大リスクを扱う。 |
| implementer | `gpt-6-sol` / `high` | `src/material.rs` のpath/schema/二入口の理由診断本体、`src/ai.rs::Config::material` のprompt/当該schema呼出のみ。確定契約に沿う局所実装。 |
| test author | `gpt-6-sol` / `high` | tests.md追補、`src/material.rs` のcfg(test)、`tests/codex_process.rs` と `tests/support/mock_codex.rs` の教材schema送信試験のみ。runtime/schema乖離と番号取り違えを独立期待値で検出。最終試験入口は設計で絞る。 |
| test runner | `gpt-5.6-terra` / `medium` | results.md追補/タスクログ/隔離生成物のみ。固定差分と既定コマンドの実行・証拠収集。 |
| reviewer | `gpt-6-astra` / `high` | 読み取りのみ。契約不整合、受理拡張、mockを実AI成功と誤認するリスクを独立確認。 |

全指定は親からの呼出可能一覧に適合する。実行metadataは未取得、起動指定と区別する。モデル/effort・Skillの起動前確認、未利用時の停止、親モデル/品質ゲート不変更は既存規則のままである。逐次dispatchし、同一material.rsのテスト節と本体は同時編集しない。既存通知/保存/承認本体は変更しない。

検証は新規focused red/green（`cargo test --locked material::tests` と設計で確定するcodex_process教材schema試験）、MQ-Fとの対応、送信schema検査、prompt静的確認を行う。変更後の固定差分で `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を新規実行する。旧U1/U2ログをU3の検証へ流用しない。schemaが適合出力を必ず生成する保証はせず、実AI/実画面未検証は別記する。

親の設計入力: `src/codex.rs:741` が `turn/start.params.outputSchema` へschemaを渡し、`tests/support/mock_codex.rs:162` のstructured_chat検査と `tests/codex_process.rs` のFixtureを送信試験へ利用できる。親が確認した公式app-server/Structured Outputs資料ではstring `pattern`、array `minItems/maxItems`、number `minimum`、`required/additionalProperties:false` を確認済みである。`minLength/maxLength` の利用可否はこの調査で確証がなく、意味上の長さ検証はruntimeに残す。これは親から受領した証拠であり、本担当による実AI確認ではない。

次のhandoff: spec author `gpt-6-astra/high` は親が起動済みであり、追補仕様レビュー後にdesigner `gpt-6-astra/high` へ渡す。未確定は添字構文/境界とschema機能の最終適合判断・独立送信試験入口であり、設計追補で解決する。必要な使用量カウンターは本追補でも未取得であり、親のKPI記録へ欠測として返す。

## 追補 U4: 再取得結果に即した通知（確定計画）

状態: 仕様/設計の限定独立レビュー両方pass・親承認済み / 2026-09-27。親から `recovery_notice_review`（指定 `gpt-6-sol/high`）の判定を受領しCP6まで完了した。完了境界は通知修正・合成回帰・全対象/release・独立レビューまでとし、利用者の再試行成功は未確認として残す。U1～U3と未完了の実機受入を維持する。

根拠: `src/app.rs` の `AiResult::Recovered` はoutcome/responseによらず本文確認可能と表示する。`run_history.rs` はresponseがSomeの場合だけ本文を表示する。親の追加確認では `notify_result(Ok)` は古いerror/attentionを消去し、warning/blockedは注意通知を開く。`refresh_runs` による後続の読込警告/エラー優先順序を維持する。

単位U4はS（本体1ファイル・既存テスト1ファイル）、難易度低～中、誤認リスク中である。新しい復旧処理や画面は作らず、既存通知・実行記録・保存/承認設計を再利用する。

| AC | 要求・受入証拠 |
| --- | --- |
| MQ-R-01 | [spec.md](spec.md) U4の安定IDを正本とする。Completedかつtrim後非空の本文がある場合だけ情報通知とし、完了/本文の確認先/非自動反映を示す。本文は通知へ転載せず、旧error/attentionを消去し、新たな注意窓を開かない。 |
| MQ-R-02～03 | 全6状態×None/空/空白/非空白を合成確認する。本文なしは未取得、Completed以外で本文ありは保存応答の確認先と完了未確認を注意表示する。Interruptedかつ本文なしは中断と完成本文未取得を示し、本文の完成度を推定しない。 |
| MQ-R-04～05 | 同一未達成結果の再回復でも注意を再表示し、後続refresh_runsの警告/エラーを優先する。全分岐で非自動反映を示し、教材/会話/学習状態/保存応答を変更せず生成・再適用・登録を開始しない。合成channel→tickと通知描画、状態比較で確認する。 |

除外: timeout/process/recovery実行ロジック、model/effort/auth/PATH、保存形式、実行記録UI、引用/path検証、実データ変更、自動AI呼出/登録、公開・commit/push。通知分類の修正はU4の範囲であり、通知機構自体の再設計は含まない。

| 役 | U4指定 model / effort | 所有・難易度/リスク・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | plan/todoのU4追補のみ。固定役、範囲・依存を限定する。 |
| external-spec author | `gpt-6-astra` / `high` | spec.mdのU4短い追補のみ。固定役、状態/本文/成功の意味を分離する。 |
| designer | `gpt-6-sol` / `high` | design.mdのU4短い追補のみ。難易度中/リスク中、通知マーカーとrefresh順序を確認する。 |
| implementer | `gpt-6-sol` / `medium` | `src/app.rs` のRecovered分岐本体のみ。難易度低/リスク中、確定した分類を既存通知APIへ接続する。 |
| test author | `gpt-6-sol` / `high` | `src/app/notifications.rs` の対象cfg(test)とtests.md追補のみ。難易度中/リスク中、状態×本文と古い通知・非反映を独立に検査する。 |
| test runner | `gpt-5.6-terra` / `medium` | results.md追補・タスクログ・隔離/ビルド生成物のみ。難易度低/判定漏れリスク中、固定差分の既定検証を実行する。 |
| reviewer | `gpt-6-sol` / `high` | read-only。難易度中/リスク中、仕様/設計の限定追補・最終差分・検証証拠と通知の誤認を独立確認する。 |

依存/CP6: 外部仕様の分類表→既存設計へ接続する短い設計追補→両追補の限定独立確認→計画者のAC/所有確定→親dispatch→独立RED→本体GREEN→固定差分のrunner→独立最終レビュー/CP7。仕様と設計の独立確認は1回のread-onlyレビューで別判定として記録する。既存U1～U3の再レビューは不要であり、仕様適合と設計適合の品質ゲートは両方維持する。承認済みの修正要求を再質問せず、親が承認充足と範囲を確認する。通常1子ずつ、再委任なし。

必須検証: `cargo test --locked notifications::tests` の新規RED/GREENとMQ-R対応、静的差分/所有確認、固定最終差分で `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`。成功済みU3ログをU4の検証に流用しない。実AI/実機未実施を明記する。起動中EXEの扱いは親が利用者応答に従い、ビルド都合で強制終了しない。

リスク対策: 本文存在を生成完了と取り違えない分類表、None/空/空白回帰、古い通知マーカーの回帰、refresh読込失敗の優先順序の確認、非反映の状態比較で制御する。全割当は親の呼出可能一覧のhigh/mediumに適合する。起動前の役/Skill/モデル照合を維持し、不在時は停止、親モデル/品質ゲートを変更しない。

未決事項/次handoff: 仕様/設計の動作選択と所有は確定し、モデル割当は維持する。親がtest authorの独立REDを受領後、implementerへ本体編集を渡す。全役の責任と最終検証/レビューは未完了のまま維持する。本子の開始/終了使用量と実行model/effortは未取得であり、親KPI記録へ欠測として返す。

## 追補 U5: 教材案比較・言い換えの可視性（確定計画）

状態: 仕様MQ-UI-01～08・設計U5の限定独立レビューpass、親承認済み / 2026-09-27。親から `material_ui_review`（指定 `gpt-6-sol/high`）の両判定を受領し、AC/試験入口/所有を確定してCP8まで完了した。完了境界はUI変更、合成回帰、標準・狭幅・拡大の隔離描画、全対象/release、独立レビューである。実機で未確認の項目は別記し、U1～U4・既存未完了項目を保持する。

根拠: `material_review.rs` は初期選択が空で、全差分の前後表示と固定会話全文を左右に並べる。`materials_ui.rs` はreplacementsを「追加の言い換えと条件」に折り畳む。strengthenが既存replacementsにある点は親の診断であり、実データを再読・複写しない。親調査では比較親画面は `src/app.rs::material_panel`、窓スクロールは `chat_ui.rs`、既存隔離native描画入口は `src/app/visual_check.rs` である。`Mode::label` は要求payloadにも使われるため、UI表示を別入口へ分離し既存AI向けlabelを保つ。

| 単位 / 規模・難易度・リスク | 確定要求・受入条件（spec.md U5を正本とする） |
| --- | --- |
| U5a 比較・モード説明 / M・中～高・中 | MQ-UI-01～03,05: 一覧の先頭自動選択/切替と枠付き前後・理由・引用が同期する。引用優先、固定文脈は明示展開。空側、0件、消滅行、理由なし、手動編集後、旧snapshotなしを区別し、削除行へ理由を推定対応しない。 |
| U5a | MQ-UI-02,04: 標準幅は前後左右、狭幅は前→後の縦配置。見出し/下部登録・破棄を固定し比較スクロールバーを常時表示する。実窓を含む標準/狭幅/拡大描画で全文と操作へ到達できる。 |
| U5a | MQ-UI-07～08: 「追加のみ」「内容を見直す（追加・変更・削除）」と条件説明を選択/案に表示する。Correctで補足追加も可能、基本変更時だけ復習へ影響する既存判定を説明する。編集・元会話・破棄・別用法確認と全登録禁止条件、Mode/AI/保存/承認契約を保持する。 |
| U5b 詳細の言い換え / S・中・中 | MQ-UI-06: replacementsの表現・意味・条件を「言い換えと使い分け」で折り畳まず一覧表示する。「語調・文体」とbusiness/elevated/registerを保持する。空/複数/長文と合成strengthenで確認し、全件へスクロール到達できる。 |

除外: 登録/永続化/Mode serde値/AI要求・prompt/schema/timeout/auth/引用検証/復習判定の変更、実データ試験、live AI、設定・品質ゲート変更、公開・commit/push。変更箇所の選択は閲覧対象の選択であり、一部だけを登録する新機能ではない。

| 役 | U5a model / effort | U5b model / effort | 所有・選定理由 |
| --- | --- | --- | --- |
| planner | `gpt-6-astra` / `high` | `gpt-6-astra` / `high` | plan/todoのU5のみ。固定役、既存契約と所有境界を整理する。 |
| external-spec author | `gpt-6-astra` / `high` | `gpt-6-astra` / `high` | spec.mdのU5のみ。固定役、選択・欠落理由・狭幅の観測可能条件を定義する。 |
| designer | `gpt-6-sol` / `high` | `gpt-6-sol` / `medium` | design.mdのU5のみ。U5aは親窓/比較/一時選択を横断し難易度中～高・誤表示リスク中。U5bは局所配置で難易度中・情報欠落リスク中である。 |
| implementer | `gpt-6-sol` / `high` | `gpt-6-sol` / `medium` | U5aはmaterial_review.rs・chat_ui.rs・app.rsの比較窓/配置本体と一時選択消去、material.rsのUI専用表示入口のみ。app.rsのMaterial受領/登録/破棄/restoreでは表示一時状態消去だけを許可しdirty/保存/既存ハンドラを変えない。U5bはmaterials_ui.rs詳細本体のみ。難易度/リスクはdesignerと同じ。 |
| test author | `gpt-6-sol` / `high` | `gpt-6-sol` / `high` | tests.md追補、harness_tests.rs、visual_check.rsの隔離合成fixtureを中心とし、必要な対象cfg(test)のみ。両単位の独立REDを本体より先に一括作成する。選択同期・clip・承認不変・表示欠落は文字列一致だけでは判定困難なため、難易度中・見逃しリスク中として独立期待値を作る。 |
| test runner | `gpt-5.6-terra` / `medium` | `gpt-5.6-terra` / `medium` | results.md追補・ログ・隔離描画/ビルド出力のみ。難易度低～中・判定漏れリスク中、確定ケースを実行しソース/期待値を変えない。 |
| reviewer | `gpt-6-sol` / `high` | `gpt-6-sol` / `high` | read-only。難易度中・誤対応/情報欠落/承認境界リスク中、仕様・設計・差分と描画証拠を独立確認する。 |

割当は呼出可能一覧のexact IDと対応effortに基づく起動指定であり、実行metadataではない。親が起動前に役・Skill・モデル/effortを照合し、未利用なら停止する。親モデル・品質ゲートは変更しない。通常1子ずつ、再委任なし。同一ファイルのテスト節と本体は逐次編集する。本体所有は最大4ファイル/単位であり、試験は各単位の必要入口に限定する。U5a/U5bの設計を1回の追補へまとめる場合は、U5aの難易度に合わせdesignerを `gpt-6-sol/high` と明示指定する。

依存: 外部仕様追補→設計追補→独立レビュー（仕様/設計を別判定）→計画確定/CP8までは完了。次は両単位一括の独立RED→U5a本体GREEN（Sol/high）→U5b本体GREEN（Sol/medium）→CP9固定差分の統合描画/全体検証→最終独立レビューとする。機能上独立したU5bも共有fixtureと編集競合を避けて逐次実施する。親の隔離preview調査を設計入力に反映済みである。

必須検証: `cargo test --locked --bin wordweave5 harness_tests` を中心に、既存Mode/ready契約は `cargo test --locked material::tests` で確認する。追加した対象cfg(test)の正確なfilter/ケース名とAC対応はtest authorがtests.mdへ固定する。選択初期化/切替/消滅/同教材の別案、理由なし・編集後・旧案、削除/並べ替えとleaf理由、非連続往復の複数引用、元会話/媒体展開、serde/AI向けlabel、全登録禁止条件、空/複数/長文replacements、dirty/保存不変を検証する。実窓を含むframeと既存隔離native previewは標準1150×950/zoom 0.8、狭幅820×650/zoom 1.6で実施し、長いnotice/禁止理由・同語チェック・編集/会話展開と本文スクロール後にもheader/footerがviewport内に残ることを必須とする。固定最終差分で `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を新規実行する。U4ログを流用せず、未実施のWindows実機/DPI・実AIを推定合格にしない。対象差分の整形/静的確認を行う。

リスク対策/残余: 一時選択は案識別キーと設定/解除/restore時の一時消去、0件消去/無効行の先頭復帰で制御し、dirty/保存に触れない。削除行に対応を推定せず、理由なしと表示する。幅760/右詳細600論理pt、高さ下限360/本文目標120/禁止理由最大約64論理ptは設計初期値であり実測調整可能である。特に狭幅820×650/zoom 1.6でheader/footerが隠れる場合、360下限を根拠に受入を免除せず調整する。契約上の未決はない。詳細ケース名と視覚調整値は後続の測定事項であり、範囲・受入基準を拡大/緩和する場合のみ親へ戻す。

次handoff: 親がtest author `gpt-6-sol/high` へ両単位の独立REDをdispatchし、受領後にU5a implementer `gpt-6-sol/high`、U5b implementer `gpt-6-sol/medium` を逐次dispatchする。最終検証/レビューは未完了である。今回plannerの実行model/effort・開始/終了使用量カウンターは未取得であり、親KPIへ欠測として返す。計画だけの作業ではapplication rebuildを実行しない。
