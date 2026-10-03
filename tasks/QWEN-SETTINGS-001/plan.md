# QWEN-SETTINGS-001 計画

作成: 2026-10-03。状態: 親が仕様P-S01/P-S02と地域順を採用し、独立仕様・設計レビュー後に実装着手。以下の暫定表記はplanner提出時の記録であり、確定差分はこの冒頭とspec/designを優先する。

確定補足：親Astra/highが設計とroot実装/文書を担当し、helper Astra/highはshared coreとstandaloneを所有する。standalone設定描画がmain.rsにあるため、helper所有へ同package main.rsを追加する（計6ファイル）。root新規qwen_settings.rsとテスト専用qwen_settings_tests.rsを追加する。独立テスト著者Sol/highは新規契約テストと旧with_host契約変更だけを扱う。core実装とテスト著者を一時並行するのは、root設定テストとshared core実装が別所有であり、独立した期待値を先に作成・RED採取済みであるためである。

既存native隔離hookはdebug専用である。debug合成nativeとrelease buildを実施し、下記release-native操作を同じ証拠で合格にしない。release-native/実アカウント操作は未検証として最終受入に残す。既存fmt不適合は対象外のbaselineとして記録し、無関係な全体整形を行わない。packageコマンドには既存専用 `--target-dir target/qwen-audio` を付ける。親/子runtime metadataとtokenカウンターは未取得。追加の利用者承認は不要、課金送信/公開は未承認である。

## 目的・完了境界

音読例文と音声が異なる場合の評価結果を説明可能にし、Qwen接続設定を「設定 → AI接続」へ移す。地域選択、地域別資格情報、保存・取消しの整合を保ち、READMEに導入条件を明記する。完了境界は承認済み仕様の実装、必要な自動検証、WordWeave5と影響するstandalone版のrelease生成・native画面確認、独立レビューまでである。実キー・有料API送信による実機評価は実施せず、未検証として別記する。

根拠: 親から受領したユーザー要求、AGENTS.md、docs/process/development-team.md、planning-and-task-breakdown全文とDefinition of Done。ResponseInvalidを例文不一致と断定できる根拠はまだなく、不正応答と評価不能の分離を仕様で定義する必要がある。

親の公式調査結果を受領済みである。[Qwen3.8-Omni-Flash](https://www.alibabacloud.com/help/en/model-studio/qwen3-8-omni-flash)の6地域対応と、[OpenAI互換API](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions)・[base URL](https://www.alibabacloud.com/help/en/model-studio/base-url)のWorkspace hostを根拠とする。Tokyo、Singapore、Beijing、Hong Kong、Frankfurt、Virginiaの `{WorkspaceId}.{ap-northeast-1,ap-southeast-1,cn-beijing,cn-hongkong,eu-central-1,us-east-1}.maas.aliyuncs.com/compatible-mode/v1` のみを今回の許可対象とする。旧API host全般への拡張は対象外である。既存Tokyo v1レコードは読めることを維持する。

開始基準: HEAD `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 既存の多数の未コミット変更。HEADだけでは作業対象を識別できないため親が開始差分と最終対象差分を保存する。既存tasks/plan.md・tasks/todo.mdは保持する。

## 要求と受入基準

| ID | 要求・観測可能な受入基準 |
| --- | --- |
| R1 | 例文不一致・無音等の評価不能な正当応答は、理由と再試行方法を表示する。不正JSON・欠落必須項目等を成功扱いせず、原因未確定のResponseInvalidを不一致と断定しない。通常評価が退行しない。 |
| R2 | Qwen設定を設定 → AI接続で変更でき、Codexの用途・認証と明確に区別できる。音読dialogにはAlibaba Cloudの表示と送信内容確認を維持し、接続設定へ到達できる。 |
| R3 | 公式根拠で確定したregion/model/hostの組合せだけを選択・送信できる。任意URLを許可せず、別地域へ既存キーを暗黙転用しない。既存Tokyo設定の互換動作を定義・検証する。 |
| R4 | 編集中/保存済みの値を区別し、設定全体の保存・取消しと資格情報の変更の関係を明示する。Qwen別保存方式を採るなら専用保存/取消しと画面移動/閉じる時の未保存guardを定義する。保存失敗で成功表示や部分的な送信先切替を起こさず、取消後の遅延応答が状態を復活させない。 |
| R5 | READMEでNode.js/npmが必要な導入方式、Voltaの位置づけ、FFmpeg/ffprobeが「読んで発音を確認」のどの形式で必要かを明記する。WAVを含む実装上の条件と矛盾しない。 |
| R6 | 関連回帰・本体全体試験・必要なpackage試験・両対象releaseとnative確認の結果を差分単位で記録し、mock成功と実API未検証を区別する。 |

対象外: Codex APIキーfallback、教材登録規則変更、学習記録の変更、任意provider/任意URL対応、実キーの閲覧・移動、実学習データによる試験、課金送信、commit/push/公開、harness・親model・品質gateの変更。非Tokyo選択の追加は今回要求に限る明示的拡張であり、その他の通信先拡張を含まない。

## 工程・依存関係

1. P0: 本計画を親が確認し、外部仕様担当がspec.mdを作る。独立仕様レビューと既存ユーザー承認範囲の照合を行う。
2. P1: 設計担当がdesign.mdを作り、地域と資格情報、設定transaction、送信snapshotと取消し、応答schemaの設計を独立レビューする。本計画を仕様後・設計後の各時点で更新し、正確な所有ファイル・試験名を確定する。
3. U2 → U3 → U4が本体設定経路、U2 → U2bがstandalone経路の依存順である。U1は仕様確定後に独立着手可能だがU2とprovider等を共有する場合は直列とする。U5は仕様確定後に文書の独立単位として並行可能である。
4. 各単位は独立テスト作成 → 本体実装 → 固定差分で試験実施 → 独立レビューで進める。通常補助は1役ずつ、再委任なし。同じRustファイルの本体とcfg(test)は順番に編集する。
5. U2/U3完了時とU1/U4/U5統合時にcheckpointを置く。重大仕様変更は外部仕様へ戻し、影響する計画・期待値・検証を更新する。

## 作業単位と所有候補

ファイル一覧は暫定であり、実装担当への書込み許可の確定は設計後である。各単位は原則5ファイル以内とし、超える場合は起動前に再分割する。planner自身の所有は本plan.mdとtodo.mdのみである。

| 単位 / 難易度・リスク | 成果・受入 | 本体所有候補 / 独立テスト所有候補 | 依存・検証 |
| --- | --- | --- | --- |
| U1 応答の意味と不正応答の分離 / 中・高 | R1。正当な評価不能、通常評価、不正応答の3区分をfixtureで再現する | tools/qwen-audio/src/provider.rs、src/error.rs、src/lib.rs（同package）。テストは新規tests/qwen_response_contract.rs（同package） | P1。package焦点試験、既存応答fixture回帰。音声内容を推測するheuristicを無根拠に追加しない |
| U2 地域ごとの接続と資格情報 / 高・高 | R3。許可表と資格情報の対応、Tokyo互換、非対応組合せの拒否 | tools/qwen-audio/src/credentials.rs、src/provider.rs、src/lib.rs（同package）。テストは新規tests/qwen_region_contract.rs（同package） | P1。地域の全候補、別地域キー非流用、未知値、資格情報失敗をmockで検証。U1と共有ファイルを直列化 |
| U2b standalone消費者への反映 / 中・高 | R3/R4。共通API変更後も地域と資格情報を正しく表示・操作できる | tools/qwen-audio/src/gui.rs。必要な独立ケースは新規tests/qwen_region_gui.rs（同package） | U2。standalone焦点試験、releaseとnative確認。U1/U2を担当する同一helperが順次担当 |
| U3 設定保存と取消し / 高・高 | R4。保存前にactive接続を変えず、保存失敗・取消し・遅延結果を安全に処理する | src/app/settings_edit.rs、src/qwen_reading.rs、src/app.rs。テストは新規tests/qwen_settings_contract.rs（内部アクセスが必要なら設計時に専用test moduleを確定） | U2。設定reopen、cancel、保存失敗、取消後late response、既存接続復元の焦点試験 |
| U4 設定画面移設 / 中・高 | R2。AI接続内で用途・認証・地域が理解でき、音読dialogから誘導できる | src/app/settings_ui.rs、src/app/qwen_reading_ui.rs、src/app/visual_check.rs。独立UIケースはtests.mdと専用新規test moduleへ記録 | U3。新設定導線、Codex非退行、狭幅/標準幅、未設定/設定済/エラーのnative確認 |
| U5 導入条件文書 / 低・低 | R5。必要条件を導入方式と音声形式ごとに説明する | README.md。docs/install-windows.md、tools/qwen-audio/README.mdは矛盾がある箇所だけ親が追加所有を確定 | 仕様確定。既存動作・公式導入情報との照合とリンク確認。専用コードテスト不要 |

省略表記のsrc/とtests/は該当行のtools/qwen-audio配下を指す。U1/U2は共有packageを変更するため本体利用とstandalone双方の検証が必要である。

親指定の担当境界: 独立実装helperがtools/qwen-audio/src/{credentials,provider,error,lib,gui}.rsのみを所有し、U1/U2/U2bを順次実施する。親はrootの設定UI・integration・READMEを所有しU3/U4/U5を担当する。独立テスト著者は新規test/fixtureのみを所有し、本体ファイルは編集しない。接続APIをdesignで固定してから並行し、追加ファイルが必要なら先にplannerへ戻す。

親のUI提案を仕様・設計入力とする: AI接続にCodex/Qwenの別カードを置き、「接続設定を編集」からregion/host/password/保存/キャンセルの専用modalを開く。modal中は背景の通常設定を操作できず、Qwen保存がWindows資格情報への独立保存であることを明示する。通常設定save/cancelと分離し、modal閉じる/外クリックの扱いとapp close時の未保存guardを仕様で固定する。自動保存は行わない。この提案は外部仕様・設計レビュー前のため確定動作とは扱わない。

## 7役のモデル指定

親が提示した呼出可能一覧に基づく指定である。plannerと外部仕様担当はともに `gpt-6-astra` / `high`。親の選択値は `gpt-6-astra` / `high` と受領したが変更しない。各起動時に利用可否を再照合し、返却された実行メタデータとは別記する。本plannerの実行メタデータは未取得である。

| 単位 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- |
| U1 | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U2 | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U2b | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U3 | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U4 | gpt-6-astra / high | gpt-6-astra / high | gpt-6.1-sol / high | gpt-6.1-sol / medium | gpt-6-astra / high |
| U5 | gpt-6.1-sol / medium | gpt-6-astra / high（親が担当） | gpt-6.1-sol / medium | gpt-6.1-sol / medium | gpt-6.1-sol / high |

選定理由: U1の不確実な障害分類、U2/U3の認証・永続化・取消し、U2b/U4の設定transactionとの接続は横断判断を伴うため、設計・本体実装・独立レビューをAstra/highとする。U1–U4（U2b含む）のテスト作成は承認済み仕様から境界/失敗fixtureを独立導出するSol/high、実施は固定手順と証拠採取に限定するSol/mediumである。U5は限定された文書整合なので設計・照合ケース・実施はSol/medium、条件の誤読や旧記述との矛盾のレビューはSol/highとする。U5実装は親の所有であり親の既存Astra/highを変更しない。モデル名から合格を推定せず、独立レビューを省かない。

外部仕様レビューと設計レビュー、最終横断レビューは独立したww-reviewer `gpt-6-astra` / `high`。U5のdesignerは文書構成、test authorは照合ケースを担当し、新規自動テストを不要とする責任分離である。親の調査補助が必要ならww-scout `gpt-5.6-terra` / `medium` に限定する。モデル/skill不在はblockerであり、fallbackしない。

## 必須検証・証拠

- 新規失敗ケースを変更前に再現し、変更後成功と既存正常系を確認する。該当しない場合は理由をtests.mdに明記する。
- 本体: `cargo fmt --all -- --check`、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5`。
- 共有package: `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check`、`cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked`、`cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --tests --locked`、`cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio`。本体利用featureのみのcheckも行う: `cargo check --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --features windows-credentials --locked`。
- Native画面: 隔離した架空設定で本体releaseのAI接続画面・音読dialogを標準幅/狭幅で確認し、standalone releaseの起動・関連設定/評価結果表示を確認する。画像・対象binaryと差分識別をresults.mdへ記録する。認証や課金評価の成否をこの確認から推定しない。
- 独立レビュー: 認証境界、保存失敗、取消しと遅延結果、正当応答/不正応答、Codex分離、互換性、文書条件を確認する。期待値を都合よく弱めず、差戻し後は関連検証だけ再実施する。
- 親がresults.md、tasks/current.md、KPI記録を所有する。ログは本task配下へ保存し、本文/キー/実教材を複製しない。plannerの開始/終了token counters・親子包含関係は未取得であり、推計しない。

## リスク・未決事項・次のhandoff

| 未決事項・リスク | 決定責任・次工程 |
| --- | --- |
| 6地域とWorkspace hostの公式対応は親が確認済み。表示順、Workspace入力の検証、既存Tokyo v1の読み込み後の編集動作を固定する必要がある | specへ6地域の許可表と互換動作を取り込み、designで資格情報keyとの対応を固定 |
| 例文不一致時の実応答は未取得。原因と緩和策を同一視すると不正応答を隠す | 親調査・specで再現可能fixtureと説明文を確定。schema厳格性の維持をdesignレビュー |
| progress.jsonとWindows資格情報は別storeでありatomic保存不可（親調査）。一般設定の保存とQwen設定の保存の関係が未確定 | 外部仕様で専用「Qwen接続を保存」「Qwenの変更をキャンセル」方式を検討し、未保存guardと一般設定保存との関係を明示。designで失敗復元、鍵編集/削除/地域変更/取消しの状態表を確定 |
| dirty状態で既存修正を巻き戻す、別作業の試験結果を混在させる可能性 | 親が開始差分を識別し、各役の所有をdispatch時に明示 |
| 共用部の変更がstandalone UIにも波及する可能性 | design後にU4を分割し、必要なファイルとnativeケースを確定 |

次のhandoff: 親へ本暫定計画を返し、承認後に外部仕様担当Astra/highをspec.mdのみの所有でdispatchする。仕様レビュー・承認を受けた時点でplannerへ再依頼し、design確定後に再度所有/依存/検証を固定する。本役は実装、テスト変更、他agent起動、公開を行わない。
