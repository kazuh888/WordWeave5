# QWEN-UX-002 計画

## 親による実装着手時の確定（2026-10-03）

外部仕様の正本は spec.md の QU-AC-001～018、内部設計は design.md。以下の暫定UX-ACは元の追跡として保持する。親は3能力の範囲・P-U01を依頼済み要求と照合して採用した。独立レビューは実装と統合して実施し、利用者による新UI受入は残す。

未決事項の確定：表示中draftの手動probe、同じAI接続ページのQwen inline編集、SafeError wire3fieldsを保持した固定2code追加、30秒/256KiBの同一host GET。文字寸法は日本語nativeで確認する。地域別の実接続は未検証のままである。

実装所有はU1/U2共通libをAstra/high、U3/U4設定UIをAstra/high、U5音読UIをSol/highへ明示dispatch。共通lib・設定UI・音読UIは別ファイルなので独立並列とし、共有箇所は親が統合する。独立test-authorはSol/high、runnerはSol/medium、最終reviewはAstra/highを維持する。実行metadataは未取得。

2026-10-03。状態：外部仕様・設計前、親への承認・dispatch待ち。作業一覧は [todo.md](todo.md)。本計画は [QWEN-SETTINGS-001](../QWEN-SETTINGS-001/results.md) の利用者受入で見つかったResponseInvalid再発とUI改善の追跡であり、前タスクの実API未検証を解消済みにしない。

提出直前の親決定：AI接続は2カードの「用途→認証→設定→操作→状態」に限定して統一し、Codexの既存保存/取消し/誘導経路を維持する。Codex編集modal新設は不要である。公式調査は親所有の [evidence.md](evidence.md) を参照する。親からspec P-U01と設計契約承認の連絡を受領したが、本担当は完成spec/designを未読のため、本書の残りは暫定として区別する。

## 完了境界と既存変更

音読評価の失敗を利用者が理解して次の操作を選べるようにし、参照画面の助言をnative音読画面へ適用し、Codex/Qwenの接続設定の操作体系を揃え、Qwenの明示的な接続試験を実装する。今回の境界は外部仕様・設計・独立レビュー、実装、隔離した自動検証、native合成画面確認、本体と影響する共通packageのrelease生成までである。利用者による改善UIの確認、実APIによる再現解消の受入は別途記録する。

計画担当の所有は本書とtodo.mdのみである。コード、テスト、他文書、実資格情報、利用者音声を変更しない。開始時HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。追跡/未追跡の広範な既存変更をgit statusで確認した。未完のtasks/plan.mdとtasks/todo.mdを保持し、前タスク成果物も書き換えない。HEAD単独は対象差分を表さないため、実施担当が対象ファイルmanifest/hashと追跡差分を記録する。

## 入力と証拠の区別

- 利用者要求（親から受領）：別英文MP3でなおResponseInvalidとなる。理解できるエラー、ブラウザ参照を基にした音読画面再設計、Codex/Qwen設定UX統一、Qwen接続試験が必要である。Skill作成は改善UIを利用者が承認した後である。
- ソース確認：`tools/qwen-audio/src/provider.rs` はSSE、finish_reason、メタデータ、JSON/schema、反射キー検知等をResponseInvalidへ集約している。`error.rs` のSafeErrorは安全な3項目の厳格なwire形式を持つ。別英文の適合結果は既にReferenceMismatchであり、不正応答とは異なる。
- ソース確認：本体Qwen接続は専用modal・Windows資格情報への独立保存、Codexは一般設定のdraftを編集する。音読画面は全文説明・形式詳細・結果メタデータを常時表示し、非実行時も取消しを出す。Qwen接続試験は存在しない。
- 親の参照読取：目的→例文→操作→状態→終了の順、見出し20/本文14程度、節間16～24/節内8程度、音声名・長さを前面へ、形式・メタデータを折りたたむ、理由と次の操作を一つの状態欄へ、非実行時の重複取消しを除く、footerに閉じる・破棄説明を置く。数字はデザイン入力でありnativeの拡大時可読性によって確定する。
- 親の公式調査：[Structured output](https://www.alibabacloud.com/help/en/model-studio/qwen-structured-output) はQwen3.8-OmniのJSON Object対応を示す。現要求にはresponse_formatがない。json_object指定は妥当な改善候補だが、実例の原因/解消の証明ではない。
- 親の公式調査：モデル一覧は `/api/v1/models?model=qwen3.8-omni-flash`。`compatible-mode/v1/models` と推測しない。公式資料の地域host差を理由に登録済みWorkspace authorityから別hostへキーを送らない。正確な出典と応答契約は親が仕様入力へ追記する。
- 未検証：失敗した実応答は保存されていないため今回の原因は未確定である。音声が別英文であることから不正応答原因を断定しない。

## 要求・暫定受入条件

| ID | 要求と観測可能な受入 |
| --- | --- |
| UX-AC-01 | 不正応答は「受信/形式/内容のどこを確認できなかったか」と次の操作を、取得できた安全な分類の範囲で表示する。判明していない音声の良否・英文不一致を断定せず、正常なReferenceMismatch/Unassessableとは区別する。未知原因も明示する |
| UX-AC-02 | JSON Object指定と必要な互換修正は公式対応・再現fixtureに基づく。欠落、矛盾、不正JSON、過大応答、途中終了等を成功へ丸めず、無断の修復/自動再送をしない。診断にキー・原応答・英文・音声を保存しない |
| UX-AC-03 | 音読画面で目的、対象例文、音声名/長さ、主操作、現在状態、終了が順に識別できる。形式制約・返却model/effort/usageは必要時に展開できる。上部の利用者指定の送信後取消し注記は原文のまま維持する |
| UX-AC-04 | 正常/別英文/評価不能/失敗/処理中の主操作と説明が状態に対応する。送信前確認、明示送信、取消し、閉じる、再練習の安全な状態遷移と旧結果の無効化を保持する。非実行時に意味が重なる取消しを出さない |
| UX-AC-05 | AI接続でCodex/Qwenの用途、認証方式、設定状態、編集、接続試験の位置・語彙・表示順を揃える。CodexはChatGPT認証、Qwenは専用APIキーであり、保存/取消し境界は明示する。見た目の統一から認証/保存の統合を推論しない |
| UX-AC-06 | Qwen試験は明示クリックした検証済み接続snapshotだけに対し、音声/教材/参照文を含まないモデル一覧要求を最大1回送る。保存操作だけでは送らず、接続先変更時に旧キーを再利用せず、redirect先へキーを転送しない |
| UX-AC-07 | Qwen試験は未試験/実行中/成功/認証失敗/制限/通信失敗/未対応/不正応答/取消しを区別する。成功は接続・認証・一覧での対象モデル確認に限定し、音声入力対応・推論権限・課金/評価品質を保証しない。404は単にキー不正と表示しない |
| UX-AC-08 | 試験中の編集/保存/閉じる・試験後の接続変更で、別接続へ古い成功が残らない。二重送信、自動再試験、遅延結果による状態復活を防ぐ。失敗時も入力/保存済み接続を守り、修正・再試験・終了に到達できる |
| UX-AC-09 | 標準/820×650/80～160%拡大のnative表示、keyboard操作、文字による状態識別を確認する。例文・音声・主ボタン・状態・終了の表示またはスクロール到達を確認し、modalの背景抑止・終了後focusを保持する |
| UX-AC-10 | 固定差分に対する焦点回帰、既存回帰、required build、独立レビュー証拠を残す。実API/実マイク/実Codexと合成表示の証拠を区別する。UI利用者承認までは新規Skillを作らない |

外部仕様担当は上記IDを引き継ぎ、正確な文言・試験対象draft/saved・状態表・原因分類を確定する。上部注記の現ソースは「※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しない。」である。親が持つ利用者指定原文と照合して固定する。

## 対象外・維持する境界

実キーでの送信、実音声/学習データ試験、API権限や正確さの保証、モデル/effort変更、音声形式/上限の拡張、採点、結果履歴/教材登録、Codexへの評価返送、汎用provider基盤化、設定profile管理、資格情報移行、認証方式統合、公開/commit/pushを含めない。ChatGPT認証、direct Volta shim、PATH探索、6地域の許可host、アプリ別資格情報、安全な送信確認は維持する。standaloneのUI全面改修は含めず、共通error/provider変更の互換性だけ検証する。新規Skillは本タスク成果物ではなく、利用者UI承認後の別handoffである。

## 工程と依存

P0 暫定計画を親へ返却 → P1 外部仕様＋独立仕様レビュー＋利用者指示との照合 → 計画改訂1 → P2 内部設計＋必要な独立設計レビュー → 計画改訂2 → 各単位の独立テスト作成、実装、固定差分の試験実施、独立レビュー → 統合検証 → 改善UIの利用者受入。

親は公式調査/参照入力をP1と並行して整える。通常補助は一役ずつ、再委任なし。U1/U2後、U3/U4後にcheckpointを設ける。共有Rust本体とcfg(test)は必ず順番に編集する。同じqwen_settings.rsを所有するU3/U4は直列である。

## 作業単位・所有候補

各単位はS/M（原則5ファイル以内）である。外部仕様/設計後に正確な節とテスト名を固定し、超える場合は実装前に再分割する。候補は新規ファイルを作る確定指示ではない。

| 単位 | 内容・難易度/リスク | 依存 | 実装所有候補 | 独立テスト所有候補・受入 |
| --- | --- | --- | --- | --- |
| U1 | 応答の説明とJSON Object要求。高：実原因不明、SSE/schema/IPC互換 | P2 | `tools/qwen-audio/src/provider.rs`, `error.rs` | `tools/qwen-audio/tests/provider.rs`, `qwen_response_contract.rs`。AC01–02、正常/別英文/評価不能、失敗段階、安全なserialization/非漏洩 |
| U2 | Qwen接続試験の安全な通信契約。高：キー送信先、取消し、成功の意味 | P2、U1のerror契約 | `tools/qwen-audio/src/connection_probe.rs`（候補）, `lib.rs`。error/provider変更が必要なら所有をU1から逐次移管 | `tools/qwen-audio/tests/connection_probe.rs`（候補）。AC06–08、URL/method/body、redirect、期限、status/response上限、model有無、漏洩/取消し |
| U3 | Codex/Qwen接続領域の共通UX。高：Codex一般設定draftとQwen独立保存、focus | P2 | `src/app/settings_ui.rs`, `qwen_settings.rs` | `src/app/qwen_settings_tests.rs`, `harness_tests.rs`。AC05/09、一般保存/取消し、専用保存失敗、既存Codex診断/PATHの回帰 |
| U4 | Qwen接続試験を設定編集へ接続。高：draft/saved snapshot、遅延成功、秘匿 | U2、U3 | `src/app/qwen_settings.rs`, 必要時`src/app.rs`の限定接続節 | `src/app/qwen_settings_tests.rs`。AC06–09、編集/保存/終了での結果無効化、連打・取消し・再試験、資格情報mock |
| U5 | 音読画面の情報順・操作整理。中～高：幅/拡大、状態分岐、確認の可視性 | U1、P2。U3/U4とはファイル独立 | `src/app/qwen_reading_ui.rs`、必要時`visual_check.rs`の合成表示節 | `src/app/qwen_reading_tests.rs`。AC03–04/09、全phaseの主操作、狭幅/拡大geometry、確認音声/英文、注記、旧結果排除 |

controllerの動作変更が必要なら `src/qwen_reading.rs` と `tests/qwen_reading.rs` をU5に無条件追加せず、別の小単位へ分けてモデル/所有を再選定する。`settings_edit.rs` の保存契約変更もU3の範囲を超える場合は同様である。README/current/結果保存は親の責任として必要な差分のみ行う。

## 7役のモデル・effort指定

呼出可能一覧（親提供および現在のtool schema）で `gpt-6-astra` と `gpt-6.1-sol` の `high` / `medium` は対応している。固定2役も含めeffortを明示し、inheritへ黙って置換しない。

| 対象単位 | 計画 | 外部仕様 | 設計 | 実装 | テスト作成 | テスト実施 | レビュー |
| --- | --- | --- | --- | --- | --- | --- | --- |
| U1 | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U2 | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U3 | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U4 | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U5 | gpt-6-astra / high | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6.1-sol / high |

理由：U1–U4は未知原因/通信と認証/保存と取消しの境界判断を伴い、設計・実装・reviewをAstra/highとする。U5は既存状態遷移の保持を前提とした限定UI変更なので設計をAstra/high、実装/独立テスト/局所reviewをSol/highとする。全単位のtest-authorは承認済み期待値から障害fixtureを独立導出するSol/high、runnerは固定手順と証拠記録に限定してSol/mediumとする。モデル名から成功を推定せず品質ゲートは共通である。

仕様review、設計review、最終の認証・取消し・結果分類を跨ぐreviewは独立 `ww-reviewer` の `gpt-6-astra/high`。必要Skillは役割表の通り、計画=`agent-skills:planning-and-task-breakdown`、仕様=`agent-skills:spec-driven-development`の仕様部分、設計=`agent-skills:api-and-interface-design`とUI時`wordweave-egui-ui`、実装=`wordweave-change`とUI時`wordweave-egui-ui`、test-author=`agent-skills:test-driven-development`のテスト部分、runner=`wordweave-change`の検証部分、review=対象に応じ `review-external-specification` / `review-software-design` / `agent-skills:code-review-and-quality`。必要Skill/モデルの未導入・未検出はblockerでありfallbackしない。

親が明示モデル/effortでfresh-context dispatchし、要求値と返却runtime metadataを別記する。plannerはAstra固定役で起動されたが本担当にruntime metadataは返却されていないため「未取得」である。親のmodel/effort・harness・検証水準は変更しない。reviewerはread-only、親が結果を保存し、指摘を実装/testの所有者へ戻す。

## 必須検証

計画のみの今回：リンク、要求/単位/依存/全7役、所有重複、未完計画保存、非実装差分を確認する。後工程は新しい固定差分で以下を実施する。前タスクログで代替しない。

- 各単位の独立RED→GREEN、正常・境界・障害の焦点回帰。テスト名は設計後に確定し、実装担当が期待値を変更して合格にしない。
- 本体 `cargo fmt --all -- --check`、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5`。
- 共通package `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check`、`cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked --target-dir target/qwen-audio`、`cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --tests --locked --target-dir target/qwen-audio`、`cargo check --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --features windows-credentials --locked --target-dir target/qwen-audio`、`cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio --target-dir target/qwen-audio`。
- native合成画面は入力/確認/処理/通常/不一致/評価不能/失敗/未設定、両接続設定、probe各状態、標準/820×650/80～160%、キーボード/スクロール/背景抑止/focusを対象とする。debug用hookでrelease実機確認を代替したと主張しない。release-native・実マイク・実Codex・実APIで未実施のものは受入残として明示する。
- 共有error変更のstandalone/MCP serialization回帰、Codex実行ファイル/ChatGPT認証/Volta/PATH探索、設定保存失敗、取消し・遅延結果、キー反射/redirect/誤hostを独立reviewする。
- 既存の全体fmt不合格は新規に結果を取得してbaselineと新規差分を区別し、不合格を合格と書かない。無関係な全体整形で解消しない。ログをtasks/QWEN-UX-002/へ保存し、合格後の繰り返しは関連差分/未解決リスクがある場合だけとする。

## リスクと未決事項

| リスク | 対策・確定する工程 |
| --- | --- |
| JSON Object指定だけで再発が直ったと誤認 | U1は安全分類と合成回帰で証明できる範囲に限定。実APIの原因/改善は別の利用者受入 |
| 分かりやすいエラーが原応答・キーを漏らす | 閉じた分類/固定文言のみ、wire互換と反射キー回帰。debugでも原応答ログ禁止 |
| list-models成功を音声評価可能と誤表示 | 外部仕様で成功の意味を固定。model不在/未対応/404を独立分類。別地域hostへrerouteしない |
| 統一UXで保存の二重境界が見えなくなる | draft/saved/試験snapshot、独立保存、一般取消しを状態表にし、文言と挙動を同時検証 |
| 書換えた入力に古い試験成功が付着 | generation/snapshot失効、期限/取消し、旧完了無視を設計しmock遅延試験 |
| 参照画面の寸法をそのままnativeへ適用 | 20/14等は暫定。日本語・拡大・長文・狭幅のnative証拠で設計確定 |

親へ返す未決事項は、(1) probeは編集中draftを検証して未保存表示するか保存済みのみか、(2) Codex側も編集modalに揃えるか共通カード/行だけに揃えるか、(3) 各応答失敗分類の粒度とwire互換方式、(4) 参照から採用する文字寸法、(5) 公式モデル一覧の地域別到達性である。いずれも仕様/設計の提案として具体化して親が既存利用者指示と照合する。利用者へ不要な再承認を要求する根拠にはしない。

次のhandoff：親の暫定計画確認後、`ww-spec-author` `gpt-6-astra/high` へ `tasks/QWEN-UX-002/spec.md` のみを割り当て、上記要求、参照助言、公式ソースを渡す。仕様review後plannerへ改訂依頼、設計review後に再度plannerへ所有/試験名の確定を依頼する。本担当からagentを起動しない。

KPI：本担当の開始/終了tokenカウンター・親子包含関係・runtime model/effortは未取得。推定しない。sandbox通常execはACL初期化で起動できず、承認付き読み取りで必須文書/ソース確認を行った。計画使用Skillは全文とDefinition of Doneを読了し、指定されたタスク別保存先を使用した。親が同一不具合追跡としてQWEN-SETTINGS-001との関連をmeasurementsへ記録する。
