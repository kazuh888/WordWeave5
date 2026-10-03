# QWEN-INTEGRATION-001 実装計画

2026-10-03。状態: U0後の精緻化版。P1～P5親採用・独立仕様レビュー重大指摘なし、設計の2指摘は独立再レビューで解消し、親から実装開始可の引継ぎを受領した。
対象基点: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` と着手時の既存未コミット差分。既存変更・旧未完計画を保持する。
計画者の所有は本書と [todo.md](todo.md) のみ。承認対象設計SHA256は `2BA651CD6F1546446130B34D34DF6DF4AA6262C6B56987F98B3145B1AF24A3A9` である。

## 目的・要求・完了境界

- R1: 単独QwenツールとWordWeave5が同じ評価ライブラリを使い、本体の英文音読から明示的なQwen評価へ到達する。
- R2: 固定した音声bytes・参照英文・接続・目的・結果受取先を表示し、人の送信操作だけで一回実行する。変更時は再準備・再確認する。
- R3: 原音、教材、学習記録、Codex認証を保持する。チャット/教材生成は既存Codex経路を維持する。
- R4: 本体専用資格情報を使い、単独ツールの既存キーを暗黙共有・移行・削除しない。設定保存ではAPIを呼ばない。
- R5: 取消し、timeout、切断後は自動再送しない。送信後は遠隔処理・課金状態不明を表示し、遅延結果を採用しない。
- R6: 正常助言/評価不能/通信失敗を分け、要求model/effortと返却実値を区別する。返却なしは未取得である。
- R7: 実API接続と助言品質を本物の入力で別途確認する。mock合格を実接続合格に読み替えない。
- 開発境界: 共通化、本体組込み、独立回帰、固定差分の納品gate、native画面確認、独立レビューまで。
- 全体完了: 上記に加え、利用者による専用設定・選択音声・送信操作を経た実接続と助言の受入証拠が揃うこと。入力欠落時は開発境界まで進め、全体未完と明記する。

## 除外・前提

Framework全面導入、教科共通型、汎用provider、客観点数/音素正解率、教材自動登録、学習記録schema変更、MCP機能追加、公開/commit/pushは対象外である。
AGENTSの承認済み例外は明示的Qwen音読評価に限定する。本体結果はCodexへ送らず、チャット/教材生成の認証fallbackを加えない。
P1～P5採用済み: 教材詳細の主例文全文を固定した単一dialog、新規専用録音または一件のWAV選択、入力/助言はdialog内メモリのみ、閉じると今回分を破棄する。自由英文編集・保存機能は含めない。
PCM16・1/2ch・8–48kHz・60秒・6MiBを再検査する。本体録音30秒上限は維持する。機器側最大192kHzとの差を無視しない。
自動変換/切詰めは除外する。非対応時は原音の再生bytesを保持し理由と再録音/対応WAV選択を示す。教材編集/切替をdialog中阻止し、別経路の版変更は不可逆に無効化する。
単独ツールの資格情報名 `WordWeave5.QwenAudio.Connection.v1` は維持し、本体は `WordWeave5.QwenReading.Connection.v1` を使う。

## 根拠・仕様設計ゲート

根拠は [再利用契約](../../docs/reuse/qwen-audio.md)、[既存仕様](../QWEN-AUDIO-001/spec.md)、[設計](../QWEN-AUDIO-001/design.md)、[結果](../QWEN-AUDIO-001/results.md)、[利用手順](../../tools/qwen-audio/README.md) である。過去の試験結果は今回の証拠にしない。
U0: 外部仕様担当 `gpt-6-astra/high` が `spec.md`、設計担当 `gpt-6-astra/high` が `design.md` を所有する。仕様/設計レビューは別担当 `gpt-6-astra/high`、read-onlyである。
外部仕様Skillは `agent-skills:spec-driven-development`、設計Skillは `agent-skills:api-and-interface-design` とUI部分の `wordweave-egui-ui`、レビューは `review-external-specification` / `review-software-design` とする。
U0受入済み: [spec.md](spec.md) のQI-AC-001～021を受入正本、[design.md](design.md) の第2～7節を公開契約/試験口の正本とする。旧AC1～6は以下の作業内略号であり、仕様IDを置換しない。
設計本文の「案/レビュー前」表記は残るが、上記hashについて親の再レビュー解消・実装可報告を採用根拠とする。親は採否記録を統合する。重大な契約変更は仕様/設計へ戻す。

## 凍結した実装契約

- featureはcore、`mcp`、`gui`（mcp依存）、`windows-credentials`、`windows-job`。default/bin必須は後者4つ。coreはGUI/MCP/UUID/Windows保存不要、既存公開名は明示exportで維持する。
- root依存は同一pathの `qwen-audio` を `default-features = false, features = ["windows-credentials"]` とする。検証型/Transport/evaluateを再利用し、評価ロジックを複製しない。
- `WindowsCredentialStore::for_target(&'static str) -> Result<Self, SafeError>` と検査用target accessor、`Connection::with_host(&self, TokyoHost) -> Self` を追加する。new/Defaultの旧宛先は不変、空/NUL/過長targetを拒否する。
- `ReadingController::new(ReadingTarget, Arc<dyn CredentialStore>, Arc<dyn Transport>) -> Result<Self, SafeError>`。load失敗はview errorへ残し設定修復を可能にする。constructor自体はowner nonce枯渇等で失敗できる。
- `ReadingTarget::new`、`view/set_audio/clear_audio/begin_settings/set_settings_draft/save_settings/cancel_settings/prepare/human_send/poll/cancel/invalidate_target/restart/close` とview fieldは設計第4節の署名通りである。
- `PreparedId` はowner nonce＋generation＋serial、全要素照合・非wrap。別controllerの同じ局所番号を拒否する。Invalidated/Closedは取消し/設定変更経由でも復活せず、新controllerだけで再開する。
- 取消しとTransport開始を同じgateで順序付ける。worker abort後はreaperを進め、restartは回収完了までBusy。UIは取消し/閉じる/対象版変更をpollより先に適用する。
- UIはEntry全fieldのserde JSON bytes＋idと固定参照を照合し、専用RecorderとメモリSpeakerを所有する。root recorder slot/一時ファイル再生を使わず、runtime終了はshutdown_backgroundでUIを待たせない。

## 作業単位・依存・所有

順序は `U0完了 → 独立テスト作成 → U1 → U2 → U3 → 統合U4 → U5`。テスト作者から実装者への引継ぎは直列とする。U4の親UI作業は凍結APIでU1～U3 helperと所有別に並行可能、統合合格はU3完了後である。
通常helper一名。U1～U3は同一 `ww-implementer`、U4は親が担当する。実API準備は親が並行し、秘密は会話/ログへ収集しない。以下のproduction所有を超える場合は親へ分割を戻す。

| 単位・規模 | 実装責任とproduction所有 | 受入条件（AC） | 検証・依存 |
| --- | --- | --- | --- |
| U1 共通部の機能選択 / M | package `Cargo.toml`, `src/lib.rs`, `src/ipc.rs`, `src/credentials.rs`（feature cfgのみ）, 必要な `src/main.rs`。lockは親 | AC1 / QI-018,019: 評価libがGUI/MCP/Windows保存なしでbuildできる。単独GUI/MCPの既定機能は維持する | GUIなしlib check/test、feature依存tree、単独package全試験。独立テスト後 |
| U2 本体専用接続 / M | package `src/credentials.rs`（namespace/constructor/with_host）、root `src/qwen_reading.rs`, `src/lib.rs` | AC2 / QI-004～006,016: 両hostの資格情報分離、保存取消し/失敗で旧設定維持、保存だけのHTTP 0、秘密非流出 | fake store契約、旧名互換、キーsentinel、Tokyo宛先拒否。U1 cfg編集後に直列 |
| U3 本体からの固定評価 / M | root `src/qwen_reading.rs`, 必要時 `src/qwen_reading/worker.rs`, root `Cargo.toml`。lockは親 | AC3 / QI-007～019: snapshot一回送信、owner識別、Invalidated不可逆、取消し優先、回収後restartと送信確実性を保持する | 二消費者fixture、HTTP 0/1、対象版/接続変更、cancel/reap競合。U2後 |
| U4 本体UI入口 / M | 親所有: `src/app.rs`, `src/app/qwen_reading_ui.rs`, `src/app/materials_ui.rs`, `src/app/visual_check.rs` | AC4 / QI-001～003,008～018,020: 主例文→録音/選択→準備→人の送信→結果/取消へ到達する。原音保持、親操作遮断、狭幅到達 | 独立UI/state回帰、合成native fixture、keyboard/IMEと倍率。API凍結後並行、統合はU3後 |
| U5 統合受入 / M | 実装者の新規production所有なし。修正は該当Uへ戻す。親が利用手順・限定規約・結果/現状索引を統合 | AC5: 固定差分でroot/独立package gate、独立レビューを通る。AC6: 別記の実API証拠を揃えるか欠落を明示する | 下記必須検証と実接続チェック。U4後 |

U1～U4の独立テスト作成は一名 `ww-test-author` にまとめる。所有はpackage `tests/integration_credentials.rs`, `tests/integration_reuse.rs`, `tests/fixtures/reuse/`、既存integration testsのfeature cfg、root `tests/qwen_reading.rs`, `src/app/qwen_reading_tests.rs` とtask `tests.md` である。
同じfixture bytes/英文/SSE/評価不能/sentinelを単独GuiControllerと本体ReadingControllerへ通す。core-onlyではevaluateを直接試験する。test本文は作者、module配線と試験口は各実装者が直列に編集する。
重点testは `foreign_controller_prepared_id_calls_zero`, `invalidated_cancel_cannot_restart_or_send`, `restart_waits_for_reap_and_requires_new_confirmation`, `cancel_wins_over_ready_completion`, `two_consumers_same_fixture_outcomes`。追加の固定試験名は設計第7節を適用する。
親のnative hookは `--ui-check <PNG> --qwen-reading <input|confirm|running|result|failed|settings> --small --qwen-scale <1|1.5|2>` と必要時 `--qwen-tail`。Qwen smallは480×640 logical、偽store/Transportのみで実資格情報loadを禁止する。
`cfg(test)` の追加がproduction同居になる場合は節単位を指定し、実装者と直列に編集する。既存期待値を実装に合わせて弱めない。
親のみ `AGENTS.md`、両lock、`docs/reuse/qwen-audio.md`、`tools/qwen-audio/README.md`、`tasks/current.md` と測定記録を統合する。他者dirtyを上書きしない。
ランナーはtask内 `results.md` / logs / 隔離生成物のみ書く。レビューはread-only、親が指摘・判定を保存する。

## モデル割当・難易度

以下は親が提示した呼出可能一覧のexact ID/対応effortであり、実行値の証拠ではない。親の選択済み `gpt-6.1-sol/high` は変更しない。
計画者は `gpt-6-astra/high`、外部仕様作者も `gpt-6-astra/high`。親はdispatch時に各引数を明示し、起動指定と返却metadataを別記する。未取得を推測しない。

| 単位 | 設計 | 実装 | テスト作成 | テスト実施 | 独立レビュー | 難易度/リスクと選定理由 |
| --- | --- | --- | --- | --- | --- | --- |
| U1 | `gpt-6.1-sol/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6-luna/medium` | `gpt-6.1-sol/high` | 中: feature条件と既存bin互換。テストは他unitと共通fixture/credential oracleを跨ぐためAstraへ変更、実行は固定コマンド中心 |
| U2 | `gpt-6-astra/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6.1-sol/medium` | `gpt-6-astra/high` | 高: 資格情報の衝突/保存失敗/秘密流出。設計・独立oracle・レビューへ高い検討幅を置く |
| U3 | `gpt-6-astra/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6.1-sol/medium` | `gpt-6-astra/high` | 高: snapshot同意、非同期取消し、二重課金。独立した競合期待値と設計レビューが必要 |
| U4 | `gpt-6.1-sol/high` | 親 `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | 高: UIから原音/課金への結合。testはcontrollerとUI取消順を跨ぐためAstraへ変更、runnerも実画面解釈が必要 |
| U5 | `gpt-6.1-sol/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | 高: 横断受入と実API/模擬の境界。前3役は不足時のみ該当U修正、追加テストも同じ独立作者。通常runner/reviewerが担当 |

U0の全体設計はAstra/highで実施済みであり、表の設計割当は以降のunit内補足/差戻し担当を指す。U1/U4のテスト作者を旧SolからAstra/highへ変更した理由は共通fixture・資格情報・controller・UIの横断検証である。モデル名を合格や性能保証の根拠にしない。
利用不可のモデル/Skillはblockerとして親へ返し、黙って継承/代替しない。親のモデル変更やquality gate変更は本計画に含めない。
各役はdevelopment-teamのSkillを全文確認し担当工程だけ実施する。通常helper一名、fresh context、子の再委任なし。独立テスト/レビューの担当は実装者と分ける。

## 必須検証・チェックポイント

- CP1（U1/U2後）: GUIなしlibと単独packageの両経路、専用資格情報の隔離が合格。設計変更は親が確認する。
- CP2（U3/U4後）: 二消費者で同一fixtureと同じ評価処理を確認。未承認/設定保存/取消後の追加HTTPは0、二重送信は1回。単なる同じcrate名では受入にしない。
- CP3（U5）: 固定差分識別とnew logsを残し、`cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` をrootで実行する。
- 独立package: `cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked`、`cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked`。
- 共通lib: `cargo check --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked` と `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked`。
- focused: `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --test integration_reuse --locked`、root `cargo test --test qwen_reading --locked` と `cargo test --bin wordweave5 qwen_reading --locked`。feature treeと仕様リンク/AC追跡を検査する。
- 両manifestのfmt checkを記録する。root着手前は既存771 diff blocksで不合格（[baseline-fmt.log](baseline-fmt.log)、親報告）。全体fmt PASSと偽らず、無関係な全面整形を禁止し、新規/変更範囲の書式適合を別記する。既存不適合を理由にtest/build gateを弱めない。
- 合成fixtureで境界（音声時間/size/形式/英文）、無効host/redirect、SSE不正、評価不能、返却metadata欠落、timeout/取消競合、secret sentinelを検証する。
- 本体録音を含むnative GUIの狭幅/拡大/日本語/keyboard/IMEを確認する。自動geometryとnative結果は別記し、未実施は合格扱いにしない。
- 実API: 親と利用者がTokyo Host/専用キーをGUI設定し、同意した音声/英文の固定対象を確認して送信する。接続成功、聞取内容/助言の妥当性、実metadata/欠落を別証拠に残す。秘密/音声本文をログへ複製しない。
- 実APIの失敗で自動連打しない。設定/契約の不一致を診断し、再送ごとに新しい人の確認を要する。保存/成績確定や課金取消しを成功条件へ混同しない。
- 不変の既存合格を無理由に再実行しない。指摘修正は影響unitの検証と必要gateを再実行し、未解決重大指摘では完了しない。

## リスク・未決・次handoff

| 論点 | 対応/決定者 |
| --- | --- |
| Qwen例外の一般化、結果のCodex流出 | 更新済みAGENTSとQI-018を追跡し、fallback/本体結果の自動送信を阻止する |
| 原音保存混入・対象版競合 | 主例文固定、専用録音slot、全field版比較、Invalidated不可逆を独立確認する |
| 192kHz等の非対応録音 | P4通り明示拒否し原音再生を保持する。変換要求は追加仕様へ戻す |
| 別controller許可流用・worker再起動競合 | owner nonceとreaper/restart待機を新しい独立回帰で検証する |
| 既存key record名の衝突、接続変更 | 固定専用名と接続snapshotで分離し、旧recordを暗黙読取/コピーしない |
| 実接続証拠不足 | 初回harnessはflatten status誤読で結果未取得、次起動分修正済みという親報告。実送信・結果を推定せずQI-021未受入を保持する |
| provider schema/助言の実際は未検証 | 実APIで確認。公式契約の確認が必要ならspec/設計担当が根拠を取得し、過去実装から保証を推定しない |

実行時追記：独立test-authorはAstra/highで完了した。implementerとtest-runnerの新規dispatchはagent thread capで不成立だったため、親（利用者選択・同指定Sol/high）がU1～U4実装と検証実行を担当する。別modelへの黙示fallbackは行わない。独立test-authorとAstra/high reviewerの責務は維持する。七名の子が全て実行したとは扱わず、gateは省略しない。
未決は実provider応答/助言品質とnative受入結果であり、P1～P5の範囲判断は再質問しない。実API送信は人のGUI操作だけ、harnessは許可/SendIntentを偽造しない。
計画Skillの既定 `tasks/plan.md` / `todo.md` は、明示されたtask専用所有指示に従い使用しない。既存未完計画は移動/変更しない。
計画担当KPI: 子 `/root/qwen_integration_planner`。開始/終了token counter・実行model/effort metadata・親子包含は未取得であり推計しない。親が測定記録へ欠測として集約する。
読み取りはWindows sandbox ACL初期化障害のため承認レビュー付き実行を使用した。計画担当は実装・テスト・実API・キー・公開を操作していない。
