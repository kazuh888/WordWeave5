# QWEN-INTEGRATION-001 実施記録

2026-10-03。実装・自動検証は完了、実接続と助言の品質は未受入である。既存dirtyを保持する。中間記録の進行中表記は、その観測時点を表す。

## 着手・測定

- 基点：HEAD `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。src/tests/tools/設定の98paths SHA256集計 `E7399454CD509E05AFEDA5000EBA1634E5C1FECA675F8F5BCB9F05571D1C1625`。
- 親の開始観測 2026-10-03 00:43:06 JST。input 920310970 / cached 895251200 / output 2139804 / reasoning 669752 / total 922450774。初期読取から観測までと子別counterは欠測。親のmodel/effortはユーザーが選択、実行metadataは未取得。
- planner/spec-author/spec-reviewerの起動指定 `gpt-6-astra/high`。実行metadataは未取得。使用役割/所有はplanに記載する。

## 実接続準備（実送信前）

- 単独ツールの専用Windows資格情報record存在を確認した。内容は表示・記録していない。有効なHost/キー、利用権限の証明ではない。
- 本体は確認時に起動0件。既存releaseはまだ更新していない。
- 合成英文 `Please reply within three business days.`、Microsoft Zira Desktop、mono PCM16/16kHz。約3秒のfixture [reading-synthetic.wav](fixtures/reading-synthetic.wav)、95,726 bytes、SHA256 `F28939CD9FCA9DC6A9059FCB0BC779818E61CC5D93E60E99091FB89F042B59FD`。個人の録音・学習資料を使っていない。
- [live-check.mjs](live-check.mjs) は公開MCP start/getだけを用い、人がGUIで音声を選択・有料送信する。harnessからgrant/SendIntentを偽造しない。初回harnessの状態項目誤読（stateでなくstatus）を修正した。初回harnessは20分でEOF終了し、結果証拠を取得できなかったため成功と扱わない。次回は状態が終端になるまで画面を保持し、未操作の画面を時間だけで閉じない。再送は行っていない。
- 認証済み実API、実返却metadata、助言の品質は未取得。合成音声で接続成功しても、人の発音判定の正確性は別受入である。

## 実装・中間検証（2026-10-03）

- U1～U4 productionは親（利用者選択Sol/high）が担当した。計画の実装helper/runner新規dispatchはagent thread capで不成立。モデルを別の子へ黙って継承せず、同指定モデルの親が実装・実行を担当した。独立Astra/highテスト作者とread-only reviewerは維持した。seven-roleの子7名が全て実行したとは扱わない。
- 独立テストはcontroller24、package9、UI14（表示指摘2回帰を含む）。未実装APIのコンパイルREDは挙動REDと区別して記録。UI初回11/12でzoom試験の初期化境界が不適切と判明、作者が通常frameで初期zoom反映＋入力後neutralframeを追加し、期待値を緩めずPASSした。
- `cargo test --all-targets --locked` 415件PASS/exit0 [root-all-tests.log](root-all-tests.log)。`cargo test --manifest-path tools/qwen-audio/Cargo.toml --offline --all-targets` 90件PASS/exit0 [package-build.log](package-build.log)。後者はlock解決込みの中間実行であり、最終locked gateと分ける。
- core-only check exit0、integration_reuse3件PASS/exit0。normal dependency treeのGUI/MCP用eframe/egui/rfd/uuid/imageとwindows crateは0、本体のqwen featureはwindows-credentialsだけ。reqwestのOS TLS用platform依存まで不存在という意味ではない。
- 独立reviewのP2（Invalidated時の遠隔処理/課金不明説明欠落）を修正、作者による実描画＋HTTP0/1の2回帰PASS、独立再レビューでblocking0。
- 既存のpreview hookによるnative合成確認初回4画像を取得したが、tailへ過大な固定offsetを毎frame指定したためbodyが空になった。またModal継承のExtendで横幅拡大とeframe後段themeで文字が小さくなることを観測した。debug tailをstick/scroll_to_cursorへ改め、dialog scopeのWrap/文字sizeを指定した。初回画像を最終受入証拠としない。修正後focused/nativeと本体最終releaseは再実行中。
- 2アプリの中間release buildはexit0（本体4m19s、単独2m52s）。本体は上記UI修正後の最終build未完。本体/単独のプロセス0を事前に確認し、稼働中ならbuildを中止するguardを付けた。強制終了なし。
- root/package Cargo commandが同一testを再実行しても依存全体を再compileする観測がある。原因と改善効果は未判定、harness/toolchain/settingsは変更していない。結果待ちだけのagentを置かず、必要再検証をまとめる。

## 最終検証・引渡し（2026-10-03 02:28 JST）

- 本体・単独ツールは同じ`qwen-audio` coreのsnapshot/evaluateを使う。本体はGUI/MCPを依存に含めず、音声評価専用`WordWeave5.QwenReading.Connection.v1`へ設定を保存する。旧単独recordは変更・コピーせず、チャット/教材生成はCodex app-serverのままである。共通設計と利用手順は [再利用設計](../../docs/reuse/qwen-audio.md)、[単独README](../../tools/qwen-audio/README.md)へ反映した。
- 固定ソースの本体`cargo test --all-targets --locked`は415件成功/失敗0 [root-final-tests.log](root-final-tests.log)。単独`--all-targets --locked`は90件成功/失敗0 [package-final-tests.log](package-final-tests.log)。controller24件・UI14件は本体集計に含む。core-only checkとintegration_reuse3件も成功し、normal依存のGUI/MCP部品混入なしを確認した。root既存全体fmtの771差分は未解消であり、変更範囲のみの書式検査とpackage fmtは成功した。
- 本体release/debugと単独releaseのビルドが成功した。既存プロセスが起動中ならビルドを中止するguardを使い、強制終了はしなかった。最終単独EXEの公開MCP initialize/tools/listは [release-smoke-final.log](release-smoke-final.log) で成功。初回smokeの誤った期待ツール名を公開仕様の`get_reading_result`/`cancel_reading_session`へ訂正したもので、製品の名称変更ではない。実評価・送信はsmokeに含まない。

| 対象 | 最終SHA256 |
| --- | --- |
| 本体release EXE（12,408,832 bytes、02:23:00 JST） | `A01C4C624E0C184F1E417F12E9B487FA2AC7BD3C1D9213BE2D14D39BD7EF2CCE` |
| 本体debug EXE（23,230,976 bytes、02:23:47 JST） | `EDA87554822B456CBAD964DE0FD587EB3DC26060E835AF827A4F93FCAC73FD82` |
| 単独release EXE | `C18AAB012E3064D8A99F9F7231B2DBF7C807D65E6A176B85C728E754773EDE29` |
| 本体controller | `0EFF76F8D6DE4DD9F5DAE75E187D20766D80239C6EFB6556C2FE3844E30E9093` |
| 本体UI | `3B93B8A5662148CB6CEB7E38161D16DA45BA23EE818627EA67577BF0F99BBE1D` |

- 最終nativeは [標準確認](artifacts/delivery-confirm-standard.png)、[狭幅確認](artifacts/delivery-confirm-small.png)、[標準結果](artifacts/delivery-result-standard.png)、[狭幅設定](artifacts/delivery-settings-small.png)、[狭幅結果](artifacts/delivery-result-small.png) の5画像。標準1150×950/zoom1、狭幅480×640/zoom1.5または2の隔離合成画面である。送信・保存/取消し・再練習・固定footerへの到達を親と独立reviewerが確認した。実録音・資格情報・HTTPは使っていない。
- 限定変更と最終5画像の独立Astra/highレビューは合格、新規P1/P2なし。取消し/無効化後の不確実性表示の2回帰も成功した。現行UIのwidth固定/Wrap/ローカル文字sizeとtail検証を対象とし、後続production変更なし。
- 文書検査は終了記録後の7文書/60 local linksに破損0、限定diff checkも成功した。使用量・欠測と手戻りは [測定記録](../../docs/process/improvement/measurements.md) に記録する。

### 未完の受入と次の操作

- QI-AC-021（実接続・実返却metadata・聞取/助言の妥当性）は未受入。キーrecord存在や合成画面/構造検査だけで接続成功・発音評価精度を主張しない。APIの自動送信・再試行は行わなかった。
- 利用者が本体の「教材→主な例文→読んで発音を確認→音声評価の接続設定」でTokyo workspace Host/キーを明示入力する。録音または短いWAVを選び、「送信内容を確認」で固定英文/原音/送信先/有料処理を確認後、自身で送信する。キーをこのチャットへ貼り付けない。単独設定からの自動移行はない。
- 実マイク、IME、実DPI、screen reader、OS資格情報の本体専用実保存/削除は別受入。助言の妥当性は利用者の実例で判断し、客観的な発音採点や診断を保証しない。
- 既存教材/成績/録音/ink、未関連dirty、旧tasks/plan.md・todo.mdを保持した。commit/push/PR/MCP設定変更は今回行っていない。step2の開発区切りは完了したが、step2全体の完了宣言は実接続受入後に行う。

## 公式根拠（一覧）

- [指定モデル](https://help.aliyun.com/en/model-studio/qwen3-8-omni-flash)：Tokyo提供と音声入力/テキスト出力。発音採点の精度を保証する記載とは扱わない。
- [OpenAI互換接続先](https://www.alibabacloud.com/help/en/model-studio/compatibility-of-openai-with-dashscope)：Tokyo workspace固有Host、同regionキーを使用する。
- [Qwen-Omni](https://help.aliyun.com/en/model-studio/qwen-omni)：検索でreasoning_effort契約を確認。全文取得はtimeout/サイズ上限で不成立であり、取得不能をwire全契約の確認済みと扱わない。

## 仕様・設計の採否

- 独立仕様レビュー `/root/qwen_integration_spec_review`（指定 Astra/high）：P1/P2なし、仕様承認可能。主例文固定、専用設定、人の送信、取消し、二消費者契約を確認。実API/native/新規実装はレビュー対象外。
- 親はP1～P5を限定した初版として採用、QI-AC-001～021を受入条件とする。内部設計中、実装着手前である。
- 独立設計レビューで無効化対象のcancel→restart迂回とPreparedId所有者の識別不足を指摘。設計を不可逆Invalidated/Closed、owner nonceと枯渇時拒否、worker回収待ちへ修正し、独立再確認で解消済み。凍結候補SHA256 `2BA651CD6F1546446130B34D34DF6DF4AA6262C6B56987F98B3145B1AF24A3A9`。
- 本体コード変更前のroot fmt checkは既存771 diff blocksで不合格 [baseline-fmt.log](baseline-fmt.log)。無関係な全体整形は混ぜず、新規/変更範囲の書式結果と区別する。root全体fmtを合格とは扱わない。本体全targetsテストとreleaseビルドは省略しない。
