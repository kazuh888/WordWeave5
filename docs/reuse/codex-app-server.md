# Codex app-server接続の再利用境界

2026-10-02 / REUSE-DESIGN-001。文書のみの設計であり、以下の`proposed` APIは仮称・未実装である。
[全体設計](README.md)と[文書受入仕様](../../tasks/REUSE-DESIGN-001/spec.md)に従う。現行は未コミット変更を含むソースから確認した。
外部protocolの最新保証を調査した文書ではなく、現在のclientが依存するprotocol面と移行時の検証責任を記す。

## 1. 用途と境界

用途は、ChatGPT認証のCodex app-serverを起動し、明示入力から生成結果を得て、記録したthread/turnの結果を再取得することである。
教材prompt、教材JSON検査、登録承認、学習記録、画面、日次利用制限はアプリ責任である。
汎用のCodex操作・任意tool実行・ブラウザー操作・APIキー認証は対象外とする。Qwenへのfallbackを持たせない。
ホストからの許可の範囲でのみ動作し、文字列コマンドやREST gatewayへ一般化する必要はない。

## 2. 現行構造と残すhelper

| ソースの入口・型 | 現行の責任・coupling | 目標の配置 |
| --- | --- | --- |
| [resolve_executable / resolve_volta_launcher](../../src/codex.rs)、[search_path::directories](../../src/codex/search_path.rs) | 設定パス、system/user PATH、既知Volta shimを探索する | clientのprocess adapter。直接Volta起動と探索優先順位を契約試験で維持 |
| [Server::start / initialize / rpc / receive](../../src/codex.rs) | process、JSON-lines、認証、timeout。diagnosticsへ直接依存 | protocolとprocessに分離。診断は固定分類のobserverへ通知 |
| [generate_with_effort / generate_recorded](../../src/codex.rs) | generation、run_journal保存、グローバルexecution、effort検査、言語教師命令 | 通信手順はclient。prompt・journal保存実体・画面用状態はadapter |
| [Output::event / recover](../../src/codex.rs) | exact IDの最終応答組立てと再取得 | clientの結果解釈へ残す。run IDからthread/turnを引くのはホスト |
| [RunRecord / Outcome](../../src/run_journal.rs) | 要求・thread/turn・結果の永続記録、再起動後のUnknown | WordWeave journal adapter。保存前提を必須callback契約で保つ |
| [Execution / Run](../../src/execution.rs)、[ModelEffort](../../src/effort.rs) | 返却metadata、画面用global、候補検査 | 返却metadataと能力型はclient契約へ。globalと表示文言はホストへ |
| [ai::Config](../../src/ai.rs) | 教材・会話の用途別promptとschema、媒体入力組立て | WordWeave adapterに残す |

単に`codex.rs`をcrateへ移すだけでは、本体の診断型・台帳・学習promptへの依存が残る。
特に現行`generate_recorded`の言語教師用命令はtransport仕様ではない。ホストの用途別promptへ移してWordWeaveの振る舞いを保持する。
ツールなし・read-only・web無効の実行profileは既定の安全境界として維持し、任意権限を注入できる入口へ広げない。

## 3. 目標の公開契約（proposed）

論理名は`codex-client`である。公開型は検査済みconstructorとaccessorを持ち、serde_jsonの任意wire objectやprocess handleを公開しない。
以下の署名は概念表であり、利用可能なRust API、ABI、Cargo packageを宣言するものではない。

| 操作・型（仮称） | 入出力 | 所有・検証 |
| --- | --- | --- |
| `resolve(LaunchConfig)` | 設定実行ファイル、作業dir、client表示名 → `ResolvedLaunch` | cwd・探索結果を起動前に解決。ホストは任意shell引数を渡さない |
| `check_connection` | 起動設定、取消handle → 認証状態または分類エラー | initialize成功だけで認証成功にしない。`account.type == chatgpt`を検査 |
| `list_capabilities` | 起動設定 → model/effort/入力能力の検証済み候補 | 不明と非対応を保持。取得失敗から候補を推測しない |
| `prepare_generation` | instructions、typed input、任意output schema、指定model/effort、用途、受取先 | `PreparedGeneration`が入力・起動/接続選択・宛先を固定し、確認用viewを返す |
| `start_generation` | 準備と一致する人の許可、`RunCheckpoint`、取消handle、observer | 一回の`GenerationHandle`。必要な保存ack前に次の送信へ進めない |
| `GenerationOutcome` | 本文、返却metadata、`ThreadId`/`TurnId`、結果確実性、保存状態 | 成功・相手側失敗・確認済中断・不明を区別。完了本文をerrorだけで失わない |
| `recover_exact` | `ThreadId`と`TurnId`の組、起動設定、取消handle | exact対象の結果。`turn/start`・`thread/resume`を呼ばず、最新turnで代用しない |
| `request_cancel` | 自分の実行handle | ローカル取消要求。遠隔中断確認の有無を別に返す |

`ThreadId`、`TurnId`、ホストの`RunId`は別型にし、文字列の取り違えを防ぐ。公開型追加のRust互換性も版管理する。
schemaの用途妥当性はホスト、wire構造・ID・message順序・返却能力の妥当性はclientが検査する。
資格情報はCodex自身が管理し、clientはAPIキーを受け取らない。確認後にアカウント/起動設定が変わる場合は停止して再確認する。
アカウント同一性の確認に利用できるprotocol情報は切出し時に検証する。確認できない状態を「同じアカウント」と推定しない。

### 現行上限とエラー契約

現行 `Server`は全体300秒、通常JSON 1行4MiB、再取得は1行32MiB、開始前通知2000件を上限とする。
`read_model_efforts`は1頁100件、20頁/計2000件、cursor長4096以下と重複cursor拒否を持つ。これは現行clientの上限である。
台帳の8,000,000 bytesは[RunRecord::save/load](../../src/run_journal.rs)の保存上限であり、protocolや会話文脈の上限と混ぜない。
切出しはこれらを現行互換profileとして保持する。総入力・累積出力・イベントqueueの公開上限は切出し仕様で確定し、無制限とは約束しない。
将来エラーは`Launch / Authentication / Unsupported / InvalidInput / Protocol / Timeout / Cancelled / Checkpoint / UnknownOutcome`等の固定分類と送信段階を返す提案である。
保存失敗と生成失敗を別に表し、再取得可否・既知IDを安全なfieldで返す。本文やerror stringの文言を制御分岐に使わせない。
現行 `Server::rpc`、`turn/start`、`Output::event`には生のprovider errorをStringへ含める経路がある。既に全経路が秘匿済みとは主張しない。
目標では生本文・prompt・資格情報を共通ログへ流さず、診断observerに段階/固定分類だけを渡す。原応答の保存先はホストの明示判断である。

## 4. 実行と保存の順序

最重要の分離条件は、台帳をホストへ移しても、保存に成功していない実行を黙って送信しないことである。
`RunCheckpoint`はclientからホストへ要求する同期的なack契約の仮称であり、通知を投げるだけの任意event sinkではない。
clientのworkerがawait/待機する間もUI threadは進む。保存adapterのtimeout/例外は停止要因とし、ackを推定しない。

| 順序 | clientの動作 | ホストの保存責任 / 失敗時 |
| --- | --- | --- |
| 1 | 準備と人の許可を照合 | 不変要求とRunIdの`Prepared`を保存。失敗なら送信前停止 |
| 2 | 起動 → initialize → account/read → thread/start | 返却thread IDとmodel/effortを保存。失敗ならturn/startへ進まない |
| 3 | 送信予定のcheckpointを要求 | `Submitted`を永続化してack。失敗ならturn/start送信0回 |
| 4 | turn/start送信、早着通知を一時保持 | 返却turn IDを保存。応答前断絶ならturn ID不明を保持し、自動再送しない |
| 5 | thread/turn一致イベントだけで最終結果を判定 | exact IDと結果を`Completed`等として保存。成功応答の適用はこの保存とは別 |
| 6 | 結果と保存状態をホストへ返す | 会話/教材の追加はホストが必要な承認・保存を別途行う |

`Submitted`は送信直前の保守的な記録であり、必ず相手に届いたという意味ではない。crash窓を未送信に決めつけない。
現行は生成完了後のjournal保存失敗を「生成は完了したが応答保存に失敗」として返す。切出しでは本文と既知IDを回復可能な失敗結果に保持する提案である。
これは新しい結果型の提案であり、現行`Result<Generated, String>`が失敗時も本文を返すという説明ではない。
再起動時は現行同様、終了確認できないPrepared/SubmittedをUnknownとして扱い、回復はID照会に限定する。

## 5. 状態・取消し・回復

| 観測 | clientの意味 | ホストの操作 |
| --- | --- | --- |
| 未送信で取消し | ローカル取消し、生成要求なし | 入力を保持、再実行時は新しい確認 |
| 送信後の取消し/EOF/timeout | 遠隔処理は結果不明になり得る | 自動再生成を禁止し、既知IDでの再取得を提示 |
| exact turnの`interrupted` | 相手側中断を確認 | 確認済中断として表示。利用量や費用の取消しは保証しない |
| exact turnの`failed` | 相手側失敗を確認 | 失敗として保持。再生成は利用者が別操作として選ぶ |
| exact turnの`completed`と最終本文 | 取得済み結果 | journal保存後、未適用の結果としてホストへ渡す |
| ID不足/対象不在 | 回復不能または不明 | 最新turnへの置換・推測・再生成をしない |
| 完了後保存失敗 | 生成完了かつ保存失敗 | 生存中の結果を保持し保存のみ再試行。可能ならexact照会 |

現行取消しはAtomicBoolを監視して受信を終了する経路であり、`turn/interrupt`の確認応答を取得する契約ではない。
新APIも取消要求を遠隔中断確認と同一視しない。遠隔interrupt機能追加はprotocol検証と別の仕様承認を要する。
所有するprocessとpipe/workerだけを終了・回収する。他のCodex processを名前で一括終了しない。
結果再取得の成功から会話への重複追加を起こさないよう、適用済RunIdはホストが保持する。

## 6. ホストとUIの責任

WordWeave adapterは[ai.rs](../../src/ai.rs)の用途別prompt、[run_journal.rs](../../src/run_journal.rs)の保存、[diagnostics](../../src/diagnostics.rs)の表示をつなぐ。
教材の案・可視差分・学習者承認、追加時の既存内容とreview保持、訂正の別操作はclientに持ち込まない。
独立ツールは自分の保存root・client表示名・prompt・結果用途を渡す。`LOCALAPPDATA/WordWeave5`や`Progress`を要求されない。
確認画面は送信対象、用途、接続選択、結果受取先を示す。人の操作がないMCP/AI要求を人の許可へ変換しない。
返却model/effortは実測fieldだけを表示し、未取得を指定値で補わない。接続失敗をすべて未ログインと案内しない。
native UIの日本語、両軸中央ボタン、拡大/狭幅での取消し可視性はホスト受入であり、clientがeguiを依存しない条件でもある。

## 7. 互換性・段階移行・復帰

protocol method/field、公開Rust型、ホストのjournal形式は別の互換面である。CLI更新をjournal schema更新へ直結させない。
現行[process試験](../../tests/codex_process.rs)と[模擬Codex](../../tests/support/mock_codex.rs)を基に、実行順・exact ID・未知結果・保存失敗を回帰条件にする。
公式protocol/利用CLI版の確認は切出し実装時に行う。未知のauth種別、能力、必須field欠落を推測で受理しない。
まず既存`generate_with_effort`等を薄い互換adapterにし、戻り値・保存場所・既存journalの読取りを保持したままclientを接続する。
次に独立最小hostへ同一版を接続する。旧版journalを読めない変更は明示migrationを要し、原本は保持する。
復帰はadapterの切替で行い、旧入口が既存journalを読めることを条件とする。Unknownを再実行して復帰したことにしない。
global executionとdiagnosticsをそのまま公開する案は、別host/同時利用の状態混線を生むため採用しない。instance単位のobserverで足りる。

## 8. 二消費者の最小例と受入

以下はproposedの順序例であり、実行可能なcargo/Rust利用例ではない。

```text
WordWeave: 用途promptを構築 → prepare_generation → 内容確認 → start_generation
           → WordWeave journalがcheckpointを保存 → 取得結果を教材案として表示
最小host: 独自promptを構築 → 同じ版のprepare_generation → 内容確認
          → 同じ版のstart_generation → 独自rootへcheckpoint保存 → 本文を表示
不明時: 記録したThreadId + TurnId → recover_exact → 未適用の結果として表示
```

第二候補は教材を持たないRustの文章支援hostである。最初の最小hostは契約試験用であり、実製品導入実績とは数えない。
両方でChatGPT認証以外を拒否、Volta/PATH探索、指定と返却値の不一致、無metadata、能力不明、異ID通知無視を同一fixtureで観測する。
Prepared/Submitted保存失敗時にturn/start 0回、完了後保存失敗時に再生成0回、再取得時に生成0回を検査する。
取消し・EOF・timeoutでUnknownを保持し、exact ID不在時に別turnを返さない。ホスト固有の型・保存pathを交換できることも合格条件である。
試験口は偽process/JSON-lines、偽checkpoint store、時計/deadline、observerである。実認証と実CLI接続はmockと別に受け入れる。

## Framework観点の補足（2026-10-03）

[Framework](framework.md)や教科adapterに対しても、本部品は通信能力だけを提供する。出題、教材案、会話、評価のどの用途かに応じたprompt/schemaと意味検査は呼出し側に残す。
英語教師という固定指示を汎用clientの既定にせず、安全な実行profileを維持したまま教科の依頼を明示する。clientが返す構造上の成功は、正しい採点・教材登録・記録保存を意味しない。
用途を増やしても新しい万能LLMインターフェースを作る根拠にはしない。教科/試行IDとの対応はhostがRunIdを通じて持ち、clientへEntry/Progressや復習方針を渡さない。
認証・Volta/PATH・checkpoint・exact回復・秘密の扱いは変更不要である。今回の点検と補足の証拠は[追補結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)に記録する。

## 9. 追跡と引継ぎ

| AC | 文書内の対応 | 次工程の焦点 |
| --- | --- | --- |
| AC01–02 | 1–4節、現行symbol参照 | 型・process・prompt・保存の依存分離 |
| AC03–04 | 3–6節 | 認証、送信前ack、取消し、不明、exact回復 |
| AC05–06 | 7–8節 | journal互換、同一版二host、復帰と故障注入 |
| AC07 | 本表、[今回の記録](../../tasks/REUSE-DESIGN-001/results.md) | 親の静的検査・独立設計レビュー |

未決は公開署名、累積入出力上限、対応CLI/protocol範囲、account同一性の確認方式、journal互換期間である。実装仕様承認者へ引き継ぐ。
今回clientの抽出、認証、mock実行、既存test再実行は行っていない。実装/試験作者は既存helperの移動を中心にし、保存失敗と結果不明を独立に検証する。
