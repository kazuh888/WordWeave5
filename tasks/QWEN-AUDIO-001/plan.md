# QWEN-AUDIO-001 計画

状態: オフライン開発完了、独立レビューPASS。2026-10-02。実API・実キー・音声評価品質・実Codex登録は別受入である。本状態欄と末尾の統合記録は親が更新する。
計画担当の所有範囲はこのファイルのみである。作業チェックリストも本書に集約し、既存の `tasks/plan.md`、`tasks/todo.md`、他タスク、利用者差分を保持する。親のU0合格・採用記録は[review.md](review.md)である。

## 目的・完了境界

Windows上で英語音読WAVと参照英文を選択し、`qwen3.8-omni-flash` による練習用フィードバックを得る独立ツールを作る。GUI単独利用とCodexからのローカルstdio MCP利用を支え、評価処理は後のWordWeave統合に再利用できるライブラリへ分離する。

今回の完了境界は実装、独立したオフライン回帰、Windowsビルド、MCPプロセス結合試験、GUIの非課金操作確認、独立レビューまでである。実API接続・課金・音声評価品質の確認は利用者がGUIで資格情報を入力した後の別の受入項目であり、mock成功を実モデル成功と扱わない。計画作成担当はコード・テストを変更せず、他エージェントを起動せず、公開しない。

## 根拠と依存関係

- 読了: `AGENTS.md`、`docs/process/development-team.md`、`docs/process/improvement/kpi.md`、指定 `agent-skills:planning-and-task-breakdown` 全文と参照Definition of Done。
- 調査時HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。ルート `Cargo.toml` / `Cargo.lock` / アプリ / テスト / 作業記録に既存差分あり。本タスク文書の作成前に `git status --short` を確認済み。
- 親から受領した公式確認は同ディレクトリ `sources.md` にある。Tokyoで対象モデルを利用でき、API baseは `https://{WorkspaceId}.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1`。Base64は10MB未満、stream/include_usage対応。モデルのreasoning_effortはnone/low/medium/xhigh、既定xhighであり、旧enable_thinkingを流用しない。公式schema等は外部仕様担当が親の証拠から確定する。計画担当が公式サイトを直接再検証した事項ではない。
- ユーザーは独立したAPIキー利用ツールの設計・実装を明示承認済みである。既存WordWeave本体のChatGPT認証/Codex専用方針には手を加えない。
- 依存順: 公式API確認 → 外部仕様/受入ID・内部設計 → U0独立複合レビュー（仕様・設計をそれぞれ判定）と既存承認範囲照合 → 計画・所有範囲確定 → 独立テスト作成と限定本体実装 → 固定差分の試験 → 独立レビュー → 親による統合報告。仕様上の重大な未確定事項があれば設計を確定しない。

## 要件・対象外

| ID | 要件 / 観測可能な受入条件 |
| --- | --- |
| R1 | 起動するとGUIが開く。API keyとTokyo workspace/API hostを利用者がGUIから設定できる。keyはWindows Credential Managerに保存し、ソース・設定JSON・引数・環境変数・stdout・診断へ出さない。保存失敗時に平文へ退避しない。 |
| R2 | 利用者がローカルWAVと参照英文を確認し、今回の送信をGUIで明示承認するまで通信しない。MCP引数や過去の承認でこの確認を迂回できない。承認対象は確定した音声bytes・参照英文・宛先・モデルと結び付ける。 |
| R3 | サイズ・時間・WAV構造・対応PCM形式・参照文長を読込み前/処理中に上限検証する。採用値は60秒以下、PCM16 mono/stereo、sample rate 8–48 kHz、参照英文10,000 Unicode scalar以下である。検証済み音声snapshotを確認・送信に使い、承認後に元ファイルを再読込みしない。byte上限と空白だけの参照文等の細則は仕様で確定し、境界値テストを持つ。空、破損、偽拡張子、上限超過は通信せず説明する。 |
| R4 | 対象は厳密に `qwen3.8-omni-flash` とTokyoの検証済みHTTPS hostである。userinfo、任意port、query/fragment、不正workspace、類似hostを拒否し、redirectを追わない。任意URLをMCP入力から受けない。 |
| R5 | 音声認識内容と参照英文を踏まえた練習用フィードバックを表示する。検証済みの発音点数・客観的診断として表示しない。返却model/usage等が無ければ未取得とし、要求値と実応答値を区別する。 |
| R6 | `--mcp` はstdio JSON-RPCで `start_reading_session(reference_text)`、`get_reading_result(session_id)`、`cancel_reading_session(session_id)` を提供する。startはopaqueなin-process IDを返し、同時に1件だけ扱う。GUI子プロセスで利用者がファイルを選び送信を承認する。結果をCodexへ返すことをGUIで明示する。getは生成/再送を行わない。送信前の取消し/閉じる操作は通信せず終わる。stdoutはプロトコルだけとし、key・音声・不要なローカルパスを返さない。 |
| R7 | 認証失敗、制限、timeout、接続断、不正応答を区別し、自動再送しない。送信開始後の取消し/切断は課金取消しを保証しないことを正しく扱う。処理重複・親終了・子終了時の状態遷移と回収を設計する。 |

対象外: 録音機能、リアルタイム音声、数値採点/音素アラインメント、教材登録、WordWeave本体統合、DB/履歴保存、HTTP待受サーバー、任意のモデル/リージョン選択、キーの探索/チャット受領、実教材・録音のテスト流用、課金API呼出し、自動retry、MCP/harness設定の自動編集、依存の無関係な更新、commit/push/公開。

## 暫定構造と比較

推奨は `tools/qwen-audio/` の独立Cargo packageである。ライブラリに検証・要求構築・応答処理を置き、同じ実行ファイルを通常GUI/`--mcp`/内部GUI子モードに分ける。独立lockfileを持ち、ルートworkspace/依存/WordWeave binaryを変更しない。内部子モードを呼ぶだけでは送信承認を省略できない。

ルートbinary追加は既存依存再利用が容易である一方、既存dirtyなmanifest/lockと認証境界へ結合する。独立crateは依存lockが増えるが、本タスクの隔離と将来のライブラリ利用に適する。別repo、汎用plugin基盤、複数常駐プロセスは不要である。この決定は設計でCargoの入れ子package動作を確認して確定する。

テスト用transport/credential storeは内部注入点に限定する。mock用HTTP宛先や資格情報を本番MCPの設定として開放しない。fixtureは合成WAVと固定の架空英文を使う。

親の最新設計候補: 保存値はkey/host/effortを専用Windows汎用資格情報の単一JSONレコードにまとめ、1回のCredWriteで保存する。新機能のeffort既定はmediumを明示表示する。入力は60秒以下のPCM16 mono/stereo WAV、さらにBase64制約を満たすbyte上限を設ける。出力JSONをローカル検証して評価可否/聞こえた内容/良かった点/改善点/次の練習をカード表示する。既存Markdown moduleはtest依存があるため無理なpath再利用をしない。これらは外部仕様・設計で確定する候補である。

## 作業単位とチェックリスト

暫定のファイル名であり、5ファイル程度を工程分割の目安とする。固定上限のためにコードを分割せず、公開type/interfaceと責務に沿って設計後に確定する。sharedな `lib.rs` / `main.rs` は順次編集し、実装者とテスト作者は同じファイルを同時に変更しない。テストは原則integration testに置く。各単位のテスト作成は本体変更とは別の所有範囲である。

### U0 仕様と設計を固定する（中規模、高リスク）

- [x] 外部仕様でR1–R7、API証拠、数値上限、エラー/取消し、承認境界を確定する。内部設計でsecret所有権、データ保持時間、GUI/MCPプロセス、応答schemaを決定する。
- [x] 独立仕様/設計レビューで重要な未解決事項を解消し、本計画を改訂する。既存ユーザー承認との相違がある場合だけ親へ判断を戻す。
- 所有: 外部仕様担当 `tasks/QWEN-AUDIO-001/spec.md`、設計担当 `design.md`、計画担当本書。レビューはread-only、記録保存は親。
- 検証: R1–R7とacceptance caseの対応、公式API出典、モデル名/endpoint/schema、全分岐の通信可否を文書で照合する。依存なし。

### U1 隔離packageと有界入力の足場（小〜中規模、中リスク）

- [x] 独立packageのライブラリとWindows実行ファイルがbuildでき、root manifest/lockを変更しない。
- [x] 合成WAV/参照英文の正常・異常・境界ケースが独立テストで再現され、入力不正時にproviderを呼ばない。
- 所有: 実装 `tools/qwen-audio/{Cargo.toml,Cargo.lock,src/lib.rs,src/main.rs,src/audio.rs}`。テスト作者 `tests/audio.rs` と必要な合成fixture生成。変更が広がる場合は責務と検証境界に沿って小作業へ分割する。
- 検証: package focused input tests、`cargo fmt --manifest-path tools/qwen-audio/Cargo.toml -- --check`、package locked build。依存U0。

### U2 GUIから承認して評価する（中規模、高リスク）

- [x] GUIでkey/host設定・WAV選択・参照英文・送信先/モデル確認・承認・結果表示までをmockで通せる。未設定/不正入力/取消し時は送信0回である。
- [x] Credential Manager失敗時に平文保存せず、providerは許可hostだけへ一度だけ送る。mockで認証失敗/429/timeout/不正応答/redirectを確認する。
- [x] UI処理を阻害せず進行状態を示し、二重評価を防ぐ。再試行は利用者の新しい明示操作となる。
- 所有: 実装 `src/credentials.rs`, `src/provider.rs`, `src/gui.rs` と順次 `src/lib.rs`, `src/main.rs`。テスト作者 `tests/provider.rs`, `tests/consent.rs`、隔離fixture。すべて `tools/qwen-audio/` 以下。
- 検証: fake credentialsとmock HTTPによる承認→単一送信→結果の結合試験、host/secret redactionの拒否試験、非課金GUI実機チェック。依存U1。必要なら「設定」「評価」の2小作業へ分割する。

### Checkpoint A

- [x] U1/U2の焦点テストとpackage buildが成功し、R1–R5/R7の証拠を確認した。実装担当はU1～U3のコードを一連で配置したため、当初予定したMCP実装前のチェックではなく、最終統合時の確認となった。最終検証・独立レビュー完了前に実設定の登録・課金送信・公開は行っていない。

### U3 CodexからGUI承認付きで呼び出す（中規模、高リスク）

- [x] initialize/tools/list/tools/call、ID対応、改行/UTF-8、エラー応答がstdioプロセス試験を通る。対応protocol version・入力/出力上限を仕様に明記する。
- [x] GUI子の取消し/異常終了/親切断/並行呼出しを有界に処理し、承認を迂回できない。隔離試験でstdout汚染・key/音声漏えい・残存子プロセスがないことを確認した。
- [x] READMEに手動MCP設定例とGUI設定/練習用という限界を記す。自動登録はしない。
- 所有: 実装 `src/mcp.rs`, `src/main.rs`, `src/lib.rs`, `README.md`。テスト作者 `tests/mcp_stdio.rs`, `tests/support/gui_fixture.rs`。すべて `tools/qwen-audio/` 以下。
- 検証: fixture子を使ったオフラインstdioプロセス試験、同じ承認モデルへの接続を検証するGUI結合試験、READMEとCLIの照合。依存U2/Checkpoint A。

### U4 固定差分の検証と独立レビュー（小〜中規模、高リスク）

- [x] test runnerが固定差分でrequired verificationを実行し、件数・未検証事項を `verification.md` と専用ログへ保存した。親の統合証拠は `results.md`。ソース/期待値は変更していない。
- [x] reviewerが仕様適合、資格情報、host制限、承認binding、MCP protocol/ライフサイクル、エラー表示、テスト独立性をread-onlyで確認し、修正後PASSと判定した。
- [x] 親がR1–R7の到達点と実API/実音声品質の未検証を分けて記録した。依存U3。

## 7役・モデル/effortの明示割当

利用可能な明示override一覧は親の再確認による `gpt-6-astra`, `gpt-6.1-sol`, `gpt-6-sol`, `gpt-6-luna`, `gpt-5.6-sol` とhostの対応effortを根拠とする。旧runner指定の `gpt-5.6-terra` は固定ww-scoutには存在するが、runnerへのoverride可否を確認できないため採用を撤回し、計画者が `gpt-6-luna / medium` へ明示再選定した。無言のfallbackではない。モデル名を性能の実測証拠とは扱わない。親model/effortと品質gateは変更しない。

計画者は `gpt-6-astra / high`、外部仕様担当は `gpt-6-astra / high` を全単位で指定する。前者は順序・所有権・リスク・モデル選択、後者はR1–R7の観測可能な仕様と変更管理を所有する。

| 単位 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- |
| U0 | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6-luna / medium | gpt-6-astra / high |
| U1 | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6-luna / medium | gpt-6.1-sol / high |
| U2 | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6-luna / medium | gpt-6-astra / high |
| U3 | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6-luna / medium | gpt-6-astra / high |
| U4 | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6.1-sol / high | gpt-6-luna / medium | gpt-6-astra / high |

- 選定理由: 設計/実装/テスト作成はprotocol、secret、状態遷移にまたがるためSol high。U1レビューは有界入力とpackage隔離に限定しSol high。U0/U2/U3/U4レビューは課金・流出・取消しの誤認を独立に検証するためAstra high。runnerは固定差分への定義済みコマンド実行と証拠採取という低リスク責務に限定し、呼出確認済みのLuna mediumとする。原因修正・合否基準変更・高リスク解釈を担わず、疑義は親/レビュアーへ戻す。
- U0の実装/テスト役は受入可能性の照合だけでコードを書かない。U4のdesigner/implementer/test authorは差戻し時だけ起動し、合格時の形式的な追加起動はしない。
- 通常は補助1役ずつ、fresh contextで親が明示dispatchする。U0複合レビュー通過後は、本タスクの例外として独立テスト作者と限定本体実装者の2役を並行可能とする。根拠は、確定仕様から独立に期待値を作成し、tests/fixtureとproductionの所有ファイルが重ならず、相互の待機を減らせるためである。公開API契約を先に固定し、実装者は期待値を変更せず、作者は実装に合わせて期待値を追従変更しない。未実装によるred/compile失敗と期待値の失敗を区別して採取する。shared manifest/lock/lib/mainは実装者のみ所有する。契約変更が必要なら並行編集を止めU0へ戻す。
- 7役は責任であり、単位ごとに全役を形式的に再起動しない。同じ役は承認済み範囲を継続して担当できる。Checkpoint Aは証拠確認であり、新しい役の起動自体を要求しない。最終レビューは実装担当から独立する。子から再委任しない。
- 起動直前にexact ID/effortとcallable一覧を照合する。未導入skill/モデル不在はブロッカーであり、別モデルやskillへ黙って切替えない。起動指定と返却実行metadataは分け、未返却値は「未取得」とする。

## 必須検証と停止条件

- unit/integration: WAV/文長境界、host拒否、credential error/redaction、consent binding、成功/認証/429/timeout/不正応答/redirect、二重送信なし、MCP lifecycleをオフラインで実行する。
- 独立package: `cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked --target-dir target/qwen-audio` と `cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --target-dir target/qwen-audio`。repo rootで実行し、既存exeとの衝突を避ける。最初の依存lock生成は実装所有者が行い、以後lockedで検証する。
- 既存Windows delivery gate: repo rootで `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を維持する。本タスクはrootコード/manifestを変更しないためrunnerがU4の固定差分で最終1回実行し、各単位では独立packageの焦点検証を使う。通過後は関係する変更・失敗・未解決リスクがある場合だけ必要範囲を再実行する。既存差分による失敗は証拠で分離し、無関係な修正・成功扱いをしない。
- formatting: 独立packageのfmt check。GUIは日本語表示、keyマスク、選択/取消し/送信前表示、応答とエラー表示、MCP経由の結果返却の明示を実機で確認する。利用可能手段で検証できない項目は未確認として残す。
- mockは実providerの受入・精度・課金動作の証拠にならない。実資格情報・ユーザー録音・有料リクエストを使わない。実MCP登録は設定例の納品までであり、harness変更は別権限とする。
- 重要レビュー指摘、公式API不明、使用モデル/skill不在、実行不能な必須検証は親へ報告し、基準を弱めず停止/差戻しする。

## リスクと対処

| リスク | 影響 | 対処・証拠 |
| --- | --- | --- |
| GUI表示後の入力差替え/承認の再利用 | 意図しない送信・課金 | 確定bytesと要求へ承認を結び付け、差替え・重複操作テストで送信回数を検証する。 |
| host偽装/redirect/secret出力 | key・音声流出 | 固定のTokyo host検証、redirect禁止、拒否ケースとredactionの独立レビューを行う。 |
| GUI/親/子の終了順とtimeout | 多重要求・残存プロセス・誤った取消し表示 | 1 session制限、状態遷移、強制終了、送信後状態不明をprocess試験で確認する。 |
| schema変動・音声理解の誤り | パース失敗・誤った学習助言 | 公式schemaと厳格なローカル応答検証を使い、練習用表示と実品質未検証を残す。 |
| 別タスクのdirty差分との混在 | 既存作業破壊・誤った合否 | 独立crate/lockと所有範囲を維持し、root gateの失敗原因を分離する。 |

## 未確定事項・次のhandoff

1. 親の公式証拠からrequest/response schema、対応音声形式、上限、timeout方針、modelの返却metadataを仕様へ固定する。
2. Tokyo workspace入力の表現と正式なhost検証規則、Credential Managerのtarget名・上書き/削除動作を仕様/設計で確定する。
3. 非同期MCP sessionのキャンセル・結果保持期間・応答文字数上限・protocol version・終了後のメモリ/pipe処理を設計で確定する。同時実行は1件であり、get/cancelによる再送は禁止する。
4. 設計後、ファイル配置とU2分割の要否を計画担当が更新する。未確定仕様を実装者の裁量で追加しない。

2026-10-02 統合記録：上記1～4はspec.md/design.mdで確定し、U0独立レビューを通過した。親は `ww-test-author / gpt-6.1-sol / high` と `ww-implementer / gpt-6.1-sol / high` をdispatch済み。実装者の所有は `tools/qwen-audio/Cargo.toml`、`Cargo.lock`、`src/**`、テスト作者は `tests/**` と `tests.md`、READMEと統合記録は親が所有する。公開APIはdesign.mdを正とする。次は固定実装の検証・独立レビューである。

## 使用量・作業証拠

親の開始counterは `sources.md` に記録済みであり、初動読込は欠測である。子の開始/終了counter、親子の包含関係、実行model/effort metadataは未取得である。起動要求はplanner `gpt-6-astra / high`（親dispatchで要照合）。推測・合算しない。子IDは `/root/qwen_planner`。本担当の終了境界は暫定計画と文書検査であり、実装/テストは未着手。親がKPI記録へ取り込むための材料であり、本担当はmeasurements.mdを編集しない。

通常shellはACL初期化エラーで開始できず、承認レビュー付きの読取り専用実行で規約/skill/manifest/git状態を取得した。ソース変更・設定変更はしていない。
