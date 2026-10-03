# MATERIAL-QUOTE-001 差分設計

2026-09-27 / 設計レビュー用。基準HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。所有は本書のみである。

入力は [spec.md](spec.md)、[diagnosis.md](diagnosis.md)、[plan.md](plan.md)。親の今回の引継ぎで仕様レビューpass・仕様全体とD1/D2承認済みと通知されたことを設計開始の根拠とする。仕様先頭の「承認前」と計画の暫定記述は古い状態であり、本担当は更新しない。指定モデルは計画どおり `gpt-6-astra/high`、実行metadataと開始/終了使用量カウンターは未取得である。

完了境界は、既存検証と通知を使う内部契約・所有範囲・独立テスト入口を確定して引き渡すこと。本体・テスト・モデル割当・ゲートは変更しない。`api-and-interface-design` は既存 `Result<_, String>` と入力境界の契約へ、`wordweave-egui-ui` は既存通知の表示・受入条件へ適用した。

## 最小の変更と契約

本体変更は `src/material.rs` の二つの引用検証ループの共通化と、`src/ai.rs::Config::material` の生成指示追記である。公開型、JSON、保存形式、エラーの型を増やさない。

`material.rs` に非公開の `validate_quote(source: &Source, quote: &Quote, missing_source: &str) -> Result<(), String>` と小さなプレビュー関数を置く。`missing_source` は呼出元が渡す既存の固定文言であり、生成入口は「選択していない往復が根拠に指定されました。」、保存済み案入口は「引用元がありません。」である。汎用エラー基盤・新しい永続型・REST層は不要である。

共通検証の順序は次のとおりである。

1. `quote.exchange_index.checked_add(1)` で表示番号を得る。表現不能なら無効参照として拒否し、丸め・別番号表示をしない。
2. `source.snapshots` から元の `exchange_index` で検索する。見つからなければ入口に対応する参照エラーを返し、他の発言を表示しない。
3. `role` が `user` なら `exchange.question` / 「あなた（質問）」、`assistant` なら `exchange.answer` / 「Codex（AIの回答）」とする。他の値は話者不正として拒否し、本文を推測しない。
4. 引用の `trim().is_empty()`、`chars().count() > 2000`、原文の `contains(&quote.quote)` がfalse、の順に判定する。空白判定以外にtrimしない。各失敗は「空白のみ」「上限2000文字を超過」「指定した元発言に連続文字列として存在しない」を区別する。非空白・2000文字以下・全文で包含なら `Ok(())` である。

参照と話者を解決できた失敗の文字列には、理由、`往復 N` と話者、`AIが返した引用`、`作成要求時の元発言`、両プレビュー、元チャットの該当往復・話者を確認する案内を含める。「先頭の抜粋」であり対応箇所の自動抽出ではないこと、現在の元チャットは編集・削除によって固定本文と異なり得ることを明示する。存在しない教材案画面や新規ボタンへ誘導しない。

共通の先頭は「引用の根拠を確認できません」等の検証についての中立表現とする。「作成できなかった」「教材はまだ登録されていない」を保存済み案の検証へ流用しない。通知文から保存・登録の履歴を推定しない。

プレビューは入力を `chars()` で先頭240スカラー値まで取り、残りがある場合だけ省略フラグを立てる。既存 `serde_json::Value::String(preview).to_string()` を使って引用符付きJSON文字列表記を得て、フラグが立つ場合に文字列の外へ `…（省略）` を付ける。これにより改行・タブ・バックスラッシュ・引用符を区別でき、Markdown記号と日本語は文字として残る。上限はエスケープ前であり、エスケープ後の長さで切り直さない。照合はプレビューを使わず全文で行う。

`Request::build_response` と `Draft::validate_evidence` の各引用ループで同じ関数を呼ぶ。JSON・変更理由・`Source::validate` の既存の検証順序は保持するため、保存済み案の不完全なsnapshot集合は従来の「固定版不足」等で先に拒否され得る。空の根拠を持つ旧形式案は、引用ループを通らない既存互換性を維持する。`usize::MAX` の加算拒否以外に新しい参照番号上限を導入しない。

生成指示には「選択された発言の連続する部分を原文どおり引用する。Markdown記号、空白、改行、句読点を削除・追加・置換せず、JSONとして必要なエスケープだけを行う」を明記する。選んだ部分文字列に含まれないMarkdown記号を引用へ足す要件ではない。指示の明確化は失敗率の保証ではなく、検証が最終境界である。

## 所有・状態・保存境界の再利用

`Request::new` は選択往復をpayloadと所有するsnapshotへ複製する。共通関数は `Source` / `Quote` を借用するだけで、会話・教材・案を変更せず、現在の会話やストレージを参照しない。返すエラーだけが新しい所有文字列である。

既存経路は `Config::material` の生成 → `build_response` → `Result<AiResult, String>` のchannel → `WordApp::tick` である。拒否は `Err` → `notify_error` となり、案成功分岐へ入らない。`build_response` 内で候補を組み立てても、そのローカル値は失敗で破棄される。受信時のキー判定、`Pending`、キャンセルの `Arc<AtomicBool>`、取消し・切断処理を変更しない。新しい非同期処理・自動再試行・タイムアウトは追加しない。

正常時は既存の `material_draft` 設定・案の保存 → 差分表示 → 利用者の登録操作 → `Draft::ready` による再検証 → 既存登録処理である。保存済み案は `Progress::validate` からも `validate_evidence` を通る。検証エラーで根拠を削除したり現会話に付け替えたりせず、既存の保存失敗・復旧処理へ返す。教材保存/進捗保存の順序や部分保存復旧は変更しない。拒否による教材・学習状態不変と、生成開始時の既存利用回数予約を区別し、進捗の全フィールドが不変とは主張しない。

## 通知UIと受入対応

`src/app/notifications.rs` は既にplain `Label`、wrap、縦スクロール、選択・コピー、通知クリックを備える。新しい通知本文を渡し、本体は変更しない。本文中の `**` をMarkdownとして解釈しない。通常の通知・コピーボタンは既存 `controls::Button` / `UiControls` を使うため、日本語glyphの両軸センタリングと既存のフォーカス/無効状態を維持する。段落本文をボタン用の補正で動かさない。

画面受入は合成の長い日本語・絵文字・改行引用を使用し、標準幅と狭幅、拡大表示で理由・番号・話者・両ラベルを読めること、縦スクロールとコピーへ到達できることを確認する。既存共有コントロールの両軸中心テストも回帰対象となる。headless形状試験はWindowsフォント/DPIの実機受入とは区別する。実機未確認なら未確認として残す。UI変更が必要と判明した場合は所有範囲と設計を親へ戻す。

| 受入ID | 対象・テスト入口 |
| --- | --- |
| MQ-AC-01～03 | `material.rs` の二入口。参照外/話者不正/空/過長/不一致の分類、選択 `[0,3]` と両話者、記号を含む正常部分引用、Markdown省略・空白/改行/Unicode改変の拒否を合成fixtureで検査する。 |
| MQ-AC-04～05 | プレビューと共通エラー、既存通知。240/241文字、日本語/絵文字/引用符/バックスラッシュ/改行/タブ、省略部分にのみ差があるケースを検査する。固定本文・元チャット案内の意味を検査し、画面でも可読性を確認する。 |
| MQ-AC-06 | `ai.rs::Config::material` の指示差分を静的確認する。live AIへの適合保証は対象外である。 |
| MQ-AC-07 | `notifications.rs` の既存 `Pending` / channel / `tick` fixtureを再利用し、合成 `Request` から得た実際の検証 `Err` を `Activity::Material` として渡す。`pending` 解消、エラー通知が開くこと、案なし、教材・学習状態不変を比較する。正常案/可視差分/承認前非登録と追加・訂正の既存回帰も確認する。 |
| MQ-AC-08 | `material.rs` で合成Draftをserialize/deserializeして検証する。二入口の同じ理由/対象、snapshot欠落、旧形式の根拠なし案、正常案を検査する。 |
| MQ-AC-09 | `checked_add` とchars境界。`usize::MAX` の不正参照でpanic/丸めなし、非連続番号、`chat::MAX_EXCHANGES - 1` の正常元番号から表示200への対応を検査する。 |
| MQ-AC-10 | 合成fixtureのみ。実事象はdiagnosis.mdからの受領証拠とし、実ログ本文は再収集・転載しない。自動試験と実AI/実機を区別して報告する。 |

## 引継ぎ・リスク

| 担当 | 編集する範囲 |
| --- | --- |
| test author | `src/material.rs` の対象 `cfg(test)`、`src/app/notifications.rs` の対象 `cfg(test)`、`tasks/MATERIAL-QUOTE-001/tests.md`。本体を変更しない。 |
| implementer | `src/material.rs` の上記共通関数と二入口、`src/ai.rs::Config::material` の指示だけ。テスト期待値を変更しない。 |
| test runner | タスク用結果・ログ・隔離fixture出力。focused regression、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` と画面受入の実施範囲を記録する。 |

本表は親の依頼に基づく最終所有案である。親/計画者は暫定planの `src/app.rs` / harness候補をこの範囲へ更新してからdispatchする。指定モデル・effortと独立レビューのゲートはplanを維持する。テスト作成のredを先に記録し、その後に本体を編集する。同じ `material.rs` を並行編集しない。

残るリスクは、AIが明確化された指示でも引用を改変することと、不一致が240文字より後なら通知内だけでは差を特定できないことである。いずれも承認済み仕様の制約であり、厳密照合と固定本文の抜粋・元チャット案内で扱う。正規化による救済は根拠の誤受理を招くため採らない。全文通知や新しい保存領域は承認された表示範囲を広げるため採らない。

本設計の証拠は上記のコード経路・型・既存テスト入口とAC対応である。アプリ試験は設計担当では実行せず、後続の独立テスト作成・実装・検証・レビューへ渡す。

## 追補 U3: 変更理由の形式契約と診断

2026-09-27。入力はspec.md追補MQ-F-01～04、plan.md U3、親から受領した公式schema/protocol調査である。親は仕様追補の独立レビューpassと利用者承認範囲内であることを確認済みである。指定は `gpt-6-astra/high`、実行metadataと本子の開始/終了使用量カウンターは未取得。所有は本追補のみである。
`api-and-interface-design` を共有フィールド契約、外部応答/保存済み案の検証境界、既存 `Result<_, String>` の一貫した診断へ適用する。U1の引用・通知・保存設計を再利用し、新しい画面/ボタン設計はない。

### 共通定義と生成契約

`src/material.rs` に非公開のトップレベル14フィールド、examplesの3フィールド、replacementsの3フィールドの定数配列を置く。仕様列挙をそのまま正本とし、`allowed_path`、schemaのpattern生成、公開の `reason_path_instructions() -> String` がこれを共有する。`/answers` を保持し、`/id`、配列そのもの、要素全体を追加しない。永続型や汎用schema基盤は作らない。
`allowed_path` は先頭 `/`、セグメント数、共有フィールド集合、添字のASCII十進表記 `0|[1-9][0-9]*` と `usize` への変換成功を要求する。空・符号・先頭ゼロ・負数・小数・余分なセグメント・末尾改行は拒否する。現行 `parse::<usize>()` 単体より厳しいが、現行の `Value::pointer` を含む受理集合は広げない。続くpointer実在検証を省略しない。
`response_schema(entry: Value) -> Value` の公開契約は維持し、`reasons.items.properties.path` を `type:string` と共有集合から生成した次の形のpatternで制約する。`TOP`、`EXAMPLE`、`REPLACEMENT` は各定数の `|` 結合を意味する。

```text
^/(TOP|examples/(0|[1-9][0-9]*)/(EXAMPLE)|replacements/(0|[1-9][0-9]*)/(REPLACEMENT))$
```

現在の定数はASCII英字だけであり正規表現メタ文字のescape依存は不要である。将来その前提を変える場合はescapeと契約試験を更新する。reasonsの `maxItems:100`、quotesの `minItems:1/maxItems:10`、exchange_indexの `minimum:0` は既存runtime境界を送信制約へ反映する。空reasonsは許可し、既存required/型/enum/additionalProperties:falseを維持する。
親が確認した [Structured Outputs](https://developers.openai.com/api/docs/guides/structured-outputs) の対応keyword内で `pattern`、`minItems/maxItems`、`minimum` を使う。`minLength/maxLength` は対応確認がないため使わず、説明/引用の空白・文字数をruntimeに残す。これは受領した資料根拠であり実AI確認ではない。
schemaは構造制約である。添字の表現可能範囲、配列内の実在、厳密な原文引用はruntimeが判定する。ECMAScriptの `$` は末尾LF直前にも一致し得るため末尾改行もruntimeで拒否する。未確認のlookaroundや `\z` を追加して完全同値を主張しない。この分担は親に確認済みである。
`src/ai.rs::Config::material` は既存指示へ `reason_path_instructions()` を加える。helperは全許可形式、Nが0始まり、配列要素全体は禁止、`/examples/0/note` と `/replacements/0/conditions` 等の正常例、必要な個別フィールドに理由を分けることを説明する。schemaは従来どおり `response_schema(entry_schema())` を渡す。引用の原文保持指示は維持する。
[app-server資料](https://learn.chatgpt.com/docs/app-server) と親が確認した `src/codex.rs:741` の `turn/start.params.outputSchema` を送信境界とする。keyword不受理時の無制約schemaへのfallback、自動再試行、応答の修正は追加しない。

### 二入口の理由検証と状態境界

非公開の `validate_reason(source: &Source, entry: Option<&Value>, reason: &ChangeReason, number: usize, missing_source: &str) -> Result<(), String>` を置く。入力を借用し変更せず、エラー文字列だけを所有する。numberは各入口の元reasonsを `enumerate()` した位置+1であり、追加時の除外後の順位ではない。100件上限確認後なのでこの加算は安全である。
共通検証はpath形式→参照実在→説明空白→説明2000文字超→引用0件→引用10件超→各 `validate_quote` の順とし、最初の失敗を返す。形式不正には許可形式/個別フィールド指定が必要な旨、実在しない項目には参照先が存在しない旨を示す。少なくとも説明の空白/過長、引用件数の不足/過多は別の原因文とする。
全ての理由単位エラーを `変更理由 {number}、path {quote_preview(&reason.path)}: {cause}` 相当で包む。引用の既存原因、往復番号、話者、引用/固定原文プレビューと案内を削らない。引用対象の未選択/欠落を示す既存入口別文言は `missing_source` で保持する。pathの240文字省略とJSON escapeは表示だけに使い、検証は全文で行う。説明本文は通知へ新たに複写しない。
生成入口は返却entryをserializeした既存rawを `Some(&raw)` として渡し、検証後だけ既存の追加/重複除去に伴うpath remapと不採用理由の除外へ進む。保存入口は既存 `generated.as_ref()` のValueを渡し、現在のcandidate/教材へ付け替えない。generatedなし・reasons空の旧案は既存どおり通り、generatedなし・reasonsありは参照不存在で拒否する。
JSON破損/型不正、理由100件超、`Source::validate`、教材自体の不正等の理由検証前エラーには架空の番号/pathを補わない。`build_response` のローカル案構築、`Draft::validate_evidence` のsource検証という既存の前処理順は維持する。
正常状態遷移は生成→理由検証→案保存→可視差分→学習者承認→ready再検証→既存登録のままである。拒否は既存Err/channel/通知へ返り、案成功分岐・登録へ進まない。共有helperは同期処理であり、既存cancel、受信キー照合、非同期所有、保存順序と失敗時復旧を変更しない。保存済み不正案を自動修復/削除しない。新しい永続フィールドはない。
表示は既存plain通知を使う。日本語、制御文字のescape、折返し/縦スクロール/コピー、狭幅・拡大時の可読性、既存ボタンの両軸センタリングは本書U1の受入条件を継承する。UI本体修正が必要なら所有範囲を親へ戻す。

### 受入対応と実装・テストへの引継ぎ

| ID | モジュール・独立した検証入口 |
| --- | --- |
| MQ-F-01 | materialの二入口とschema。14トップレベル（answersを含む）/6配列フィールド、実在0/1と配列外2・空配列、未知項目、全体path、符号/先頭ゼロ/巨大数/末尾改行を別ケースにする。 |
| MQ-F-02 | `response_schema` の型/required/追加項目禁止/keyword/実際のpatternを独立期待値で検査する。`tests/codex_process.rs` からWindows限定 `#[path = "../src/ai.rs"] mod ai` を読み込み、Fixtureのexe/cwdで実際の `ai::Config::material` を呼ぶ。`tests/support/mock_codex.rs` の教材専用modeで実送信outputSchemaとthreadの指示を検査し、正常な合成entry/reasonsを返す。helper同士の等値検査だけにしない。 |
| MQ-F-03 | 生成/保存済み両入口で理由5～7の全体pathを各々最初の失敗とする。有効引用を使い番号/path/原因を確認する。説明空/2000/2001、引用0/1/10/11、引用の各拒否、240/241文字pathと制御文字も検査する。 |
| MQ-F-04 | 正しい個別フィールドへ分割した正常例、serialize/deserializeした案、根拠なし旧案、追加remap、U1の厳密引用、既存の非登録/承認/学習状態保護回帰を保持する。 |

schema検証は定義・実送信・正規表現評価の三点を必要とする。追加依存は使わず、送信したpatternをPowerShellの `.NET Regex`（ECMAScript指定）で構文コンパイルし、仕様から作る正負の表を評価した証拠を残す。全体path/未知項目/不正添字の構造拒否を確認し、巨大数字列と末尾LFはruntime側の拒否責務として別ケースにする。この補助確認をOpenAI側schema受理や実AI成功の証拠と扱わない。
test authorは既存cfg(test)、codex_process/mock、tests.md追補だけを所有する。implementerはmaterialの定数/helper/schema/二入口とaiの指示組立のみを所有し、テスト期待値と通知/保存/承認本体を変更しない。同一material.rsのテストと本体は逐次編集する。公開追加は指示を取得する小さなhelperだけである。
focusedは `cargo test --locked material::tests` と `cargo test --locked --test codex_process material`（新試験名にmaterialを含める）。後続runnerは既定の全対象test/release buildと独立レビューを維持し、新しい証拠を残す。設計担当は本体/テストを編集せずアプリ試験も実行しない。
残るリスクはschemaが実在/引用の正しさを保証しないこと、実AI/実画面が未確認であること、長い不正pathが省略されることである。whole-item受理や自動理由修復は既存契約と根拠対応を変えるため採用しない。レビュー対象は本追補であり、実装・試験の合格を先取りしない。

## 追補 U4: 回復結果通知の内部契約（設計案）

入力はspec.mdのMQ-R-01～05とplan.md U4、対象は `src/app.rs::tick` の `AiResult::Recovered(RunRecord)` 分岐である。仕様追補・本設計は独立レビュー前であり、実装・試験の合格を意味しない。指定は計画者の `gpt-6-sol/high`、実行metadataと本子の使用量カウンターは未取得である。
`api-and-interface-design` に従い、既存の `RunRecord { outcome: Outcome, response: Option<String> }` を入力契約、`notify_result(Ok(String))` / `notify_blocked(String)` を通知契約とする。新しい公開型・REST層・永続フィールドは作らない。

| 入力状態 | 分岐と表示契約 | 対応ID |
| --- | --- | --- |
| `Completed` かつ `response.as_ref().is_some_and(|s| !s.trim().is_empty())` | `notify_result(Ok(...))`。記録上の完了、保存本文を実行記録で確認できる旨、非自動反映を伝える。本文自体は載せない。 | MQ-R-01,05 |
| 上記以外で本文なし | `notify_blocked(...)`。`outcome.label()` による実際の状態と本文未取得、非自動反映を伝える。`Interrupted` は中断と完成本文未取得を明記し、`Completed` も状態と本文を分ける。 | MQ-R-02,04,05 |
| `Completed` 以外で本文あり | `notify_blocked(...)`。実際の状態、保存応答の確認先、完了未確認、非自動反映を伝える。本文を「部分」や「完成」と断定しない。 | MQ-R-03～05 |

本文有無は文字列の存在とUnicode空白を除いた非空性だけで判定し、JSON妥当性や生成成功を推定しない。全6 `Outcome` を同じ分類へ通し、個別原因・認証・model/effortを補わない。文言は日本語の状態・本文有無で区別し、色だけに依存しない。
`tick` は既存 `Pending` のchannel受信、`pending.take()`、キー照合を経てこの分岐へ入る。`recover_run` のworker/cancelと `codex::recover` の記録保存は維持し、受信後は通知分類→既存 `refresh_runs()` の順とする。scan警告/エラーがあれば後者が通知を上書きする。`Err` と切断は既存経路のままである。
成功の `notify_result(Ok)` は古いerror/attention印を消し、新たな注意窓を開かない。未達成の `notify_blocked` は同文言の再回復でも `last_notification_alerts[1]` を消して注意窓を開く。利用者が既に開いた窓を強制的に閉じる契約ではない。
記録はworkerで保存済み、UI側は通知と一覧再走査のみを所有する。教材、会話、学習状態、保存応答、案/承認、生成再試行へ書き込まない。回復通知自体の障害復旧処理・再保存・再試行は追加しない。
UIは既存 `notifications.rs` のplain日本語本文、折返し、縦スクロール、クリック/コピーを再利用する。共有 `controls::Button` / `UiControls` の可視文字の両軸中心とフォーカスを維持し、拡大・狭幅でも状態と本文有無が読めることを画面受入へ渡す。レイアウト変更は本単位に含めない。
test authorは合成 `RunRecord` を隔離 `harness_tests::fixture()` の `Activity::Recovery` channelへ送信し `tick` と通知描画を通す。6状態×None/空/空白/非空白、古いerror/attention消去、閉じた同一注意の再開、`refresh_runs` 後発警告/エラー優先、本文非転載、progress/deck/chat draft不変を独立期待値で検査する。実行記録の実保存やlive AIをfixtureへ要求しない。
実装担当は `src/app.rs` のRecovered分岐のみ、test authorは `src/app/notifications.rs` の対象 `cfg(test)` とtests.mdのみを所有する。runnerは新規RED/GREEN、全対象test、release buildを実施し、独立レビューへ固定差分を渡す。設計上の残余リスクは記録上の `Completed` と本文の有無だけでは本文の完成度・教材としての正しさを保証できないことであり、通知ではその保証をしない。

## 追補 U5: 教材案比較・言い換え・反映方法のUI設計

入力は親が利用者承認の範囲として確認した [spec.md](spec.md) の MQ-UI-01～08 と [plan.md](plan.md) U5 である。指定は統合U5設計に対する計画者選定 `gpt-6-sol/high` であり、実行metadataと使用量カウンターは未取得である。本追補の完了境界はUIの状態・配置・モジュール契約・試験入口の確定までであり、実装・受入結果は含まない。`api-and-interface-design` は既存 `Draft` / `material_diff::Row` / `Mode` の境界へ、`wordweave-egui-ui` は日本語表示、操作文字の両軸中心、拡大・狭幅の配置へ適用する。

### モジュールと入力契約

| 対象 | 所有する表示・状態 | 対応ID |
| --- | --- | --- |
| `src/app/chat_ui.rs` | 教材窓の外側スクロールを廃し、表示可能な窓高さを内部レイアウトへ渡す。未作成時の往復選択も同じ窓内の本文スクロールを使う。 | MQ-UI-02,04,08 |
| `src/app.rs::material_panel` | 案の有無で見出し・本文・固定下部操作を組み立て、既存 `ready`、編集、元会話、同じ基本語チェック、破棄・登録処理を呼ぶ。`ready` 判定と登録ハンドラは変更しない。 | MQ-UI-04,05,07,08 |
| `src/app/material_review.rs` | `material_diff::rows` による閲覧用一覧・一時選択、選択行だけの前後・理由・引用・固定会話を描画する。案・教材・学習状態へ書かない。 | MQ-UI-01～05,08 |
| `src/app/materials_ui.rs` | 既存 `Entry::replacements` を詳細に直接列挙し、語調情報と節を分ける。 | MQ-UI-06 |
| `src/material.rs::Mode` | 必要なら UI 専用の小さな表示名/説明 accessor を追加する。既存 `label()`、variant、serde、AI送信値は保持する。 | MQ-UI-07 |

`material_diff::rows(old, candidate)` は表示順と `Row { path, label, before, after, kind }` を所有する既存入口である。新しい差分計算や公開APIは不要である。`Row.kind` の「追加・変更・削除・並べ替え」を一覧に文字で示し、`Row.before/after` をそのまま枠へ渡す。`material_diff::spans` は前後の強調にだけ使い、登録可否の判断へ使わない。`Draft::active_reasons()` だけを現在有効な理由として扱う。固定版 `source.snapshots` と理由内 `Quote { exchange_index, role, quote }` は読み取り専用であり、元会話の現内容を固定版と混同しない。

### 選択・根拠表示の状態遷移

選択は `egui::Context` の一時データに保持し、`Progress`/`Draft` へ保存しない。既存の `("material-change", source.entry_id)` だけのキーは別案へ漏れるため、`source.conversation_id`、`source.entry_id`、`source.at`、modeを含むキーとする。さらに案を設定する `tick` の `AiResult::Material`、破棄・登録の `material_draft = None`、`restore` でこの一時選択を消去する。同秒・同教材の別案でも設定時の消去が効く。候補を手動編集したときは消去しない。選択値は行の `path` とし、毎描画で行集合に照らす。行が0なら `None` へ消去し、1件以上で未選択または消滅した選択なら先頭行へ復帰し、有効な選択は維持する。選択変更の同じ描画で比較・理由・引用へ同じ行を渡す。選択と展開だけでは `dirty`/保存/生成/登録を発火しない。

理由との対応は `reason.path == row.path` または `reason.path` が `row.path + "/"` で始まる場合だけとする。これで配列行 `/examples/N`、`/replacements/N` に個別field理由をまとめられる。`material_diff` の削除行は `/removed/...` であり、候補側の理由pathから削除理由を安全に逆算できない。削除行は「理由未取得」と明示し、全理由をその行に誤対応させない。並べ替えも候補添字のpath一致だけを使用し、移動元添字から推定しない。`active_reasons()` が空で保存理由がある場合は「手動編集後：生成時の理由は現在の差分には適用しない」と表示し、必要なら元の理由・引用を別の明示展開に全件表示する。そこでは選択行の理由と呼ばない。理由自体が0件なら単に未取得と表示する。差分0件では以前の前後・理由・引用を表示しない。

選択行の理由を複数件、返却順で表示し、各理由の直下にその理由の引用を先に列挙する。引用には `exchange_index.checked_add(1)` で得た元会話の往復番号と `user`/`assistant` の日本語話者、引用文字列を示す。固定snapshotの該当発言を参照できれば引用箇所を既存 `quoted_text` で強調する。引用が複数・別話者でも理由単位の対応を保つ。固定会話の全前後文脈は初期折畳みとし、展開時に既存のsnapshot本文と媒体根拠・注釈画像・原録音操作を残す。固定snapshotがない旧案では欠落を明記し、「元の会話を表示」の既存操作へ案内する。現在の会話が変更され得る説明を維持する。引用の検証・省略・保存契約は変更しない。

### 窓の高さと操作の配置

案がある窓は「固定見出し → 高さを制限した本文スクロール → 固定下部操作」の三領域とする。外側 `Window::vscroll(true)` に依存せず、窓の `ui.available_height()` と実際に配置した見出し・下部領域から本文へ残る高さを毎描画で求める。下部は登録・破棄ボタンと `ready` の禁止理由に絞る。長い禁止理由は最大約64論理ptの独立スクロール/折返し領域で全文に到達できるようにする。同じ基本語チェック（条件付き）、復習影響文、長いnoticeは本文内へ置く。登録操作が本文に押し出されないことを優先し、約360論理ptを窓の高さ下限、本文120論理ptを目標最小値として標準/狭幅で実測調整する。窓の表示可能領域がその高さ未満なら無理に固定を主張せず、結果へ未達範囲を記録する。ボタンは横幅に応じて折返し/縦積みし、disabled状態と日本語の全ラベルを保持する。見出しには案のbase・反映方法を置く。未作成時も往復リスト/対象語/モード説明が本文スクロールで到達可能である。

本文の `ScrollArea::vertical` は可視スクロールバーを常時確保し、候補編集・元会話操作・比較を含める。比較内の利用可能幅が約760論理pt以上なら左一覧へ240pt程度を割り当て、残りを右詳細へ渡す。未満なら一覧を上、詳細を下へ移す。右詳細内が約600論理pt以上なら前後を2列、未満なら前→後の縦積みにする。これは日本語20～21ptの本文に各枠280pt程度と余白を与える初期閾値であり、ズーム後の `ui.available_width()` で判定し、実測で微調整する。前後は見出し付きの独立した `Frame::group` と背景で区別し、色に加え「変更前」「登録される内容」と行の変更種別を文字で示す。空欄は「（なし）」とする。各枠の内側は明示的な縦layout・有限の幅とし、長い日本語/英語を折り返し、列が親幅を押し広げないようにする。入れ子のスクロールを使う箇所もバーを常時表示し、マウスホイール/キーボードで引用・展開内容へ到達できることを確認する。

操作文字は既存 `controls::Button` / `UiControls` を使い、可視glyphの水平・垂直中心、focus/hover/disabledを維持する。変更一覧の行は長文を切る場合も既存 `controls::list_row` とhover全文を使い、選択の意味をテキストで示す。段落は通常の行間を維持し、ボタン用補正を掛けない。`egui` 0.31.1 の実APIに合わせてスクロールバー常時表示を設定し、表示状態を合成画面で検証する。

### 反映方法と詳細

`Mode::New` は既存の「新規登録」を表示する。`Append` はUI上「追加のみ」、`Correct` は「内容を見直す（追加・変更・削除）」とし、選択時と案見出しで同じUI専用名称を使う。説明はAppendで既存内容・復習を維持する条件、Correctで追加も可能であること、基本の説明・問題・正解等を変更した場合に再学習となる条件を区別する。案ごとの実際の復習影響は既存 `Draft::resets_learning()` の表示を優先する。`Mode::label()` は `ai.rs` の要求にも使われるため改名しない。serde値・`Request` payload・生成指示・`ready`/`check_mode`・登録時の復習判定はそのままである。

教材詳細は既存 `material_detail_scroll` の内側で、`replacements` が非空なら「言い換えと使い分け」に `phrase`、`meaning`、`conditions` を元の順で常時列挙する。0件は空節を省くか「言い換えはない」と示し、架空の項目を作らない。「語調・文体」は別節とし、既存 `business`、`elevated`、`register` を保持する。枠内を縦layoutにして長文を折り返し、複数件は既存詳細スクロールで全件へ到達させる。別の要約生成やデータ変換は不要である。

### 受入と引継ぎ

| ID | 独立テスト・描画入口 |
| --- | --- |
| MQ-UI-01,05 | `src/app/harness_tests.rs` の合成 `Draft` と `material_diff::rows`。0/1/複数行、先頭選択・切替・選択行消滅、同一教材の別案、理由なし、生成後編集、旧snapshotなし、削除行を確認する。一時選択以外の `Draft` / `Progress` / `deck` 不変を比較する。 |
| MQ-UI-02,03 | 追加・変更・削除・並べ替え、配列leaf理由と複数引用・話者・非連続往復を合成する。表示文言と選択同期に加え、前後/引用のshape bounds、空欄、展開後の固定原文/媒体操作への到達を確認する。 |
| MQ-UI-04,08 | `chat_ui.rs` の実窓を含むframe試験で長いnotice/禁止理由、編集展開、条件付き同語チェックを合成し、本文スクロール後も見出し・footerの矩形がviewport内か、登録無効/破棄有効と案/教材/学習状態不変を検査する。登録経路自体は既存回帰を維持する。 |
| MQ-UI-06 | 合成 `Entry` で `replacements` 0/1/複数・長文を詳細に描画し、初期状態で項目・意味・条件を確認する。`business/elevated/register` とスクロール到達性も確認する。 |
| MQ-UI-07 | UI文言と説明を検査し、既存 `Mode::label()`、serde、AI要求payloadに変更がないことを独立期待値で回帰する。Appendの基本改変拒否とCorrectの追加/基本変更・復習影響を既存 `ready` 経路で確認する。 |

test author は `tests.md` と対象 `cfg(test)`、`visual_check.rs` の隔離合成fixtureだけを所有し、既存 [harness_tests.rs](../../src/app/harness_tests.rs) の比較fixtureを拡張できる。native隔離 `--ui-check` は標準1150×950/zoom 0.8 と狭幅820×650/zoom 1.6 を既に持つ。比較案/詳細の合成fixtureを追加し、実際に使ったviewport・倍率、展開前後、ボタンと枠の可視範囲を記録する。runner は固定差分に対するfocused、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5`、隔離native描画を実施する。Windowsフォント/DPIの実機確認と実AIは自動検証から推定しない。implementer は計画のU5a/U5b本体所有だけを変更し、テスト期待値を直さない。reviewer は理由と削除行の誤対応、固定footerの高さ、案選択の副作用、AI向けMode値の不変を独立確認する。

残余リスクは `Row` の削除pathと理由pathに安全な対応がないこと、同秒・同内容の別案識別、極端に低い窓での高さ配分である。前者は理由未取得と明示し、後二者は一時状態の初期化と窓下限・狭幅描画で検出する。REST層・新しい永続フィールド・登録処理変更はこの表示契約を満たすために必要ない。
