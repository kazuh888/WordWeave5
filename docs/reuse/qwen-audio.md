# 音声評価・Qwenの再利用境界

2026-10-03 / REUSE-DESIGN-001・QWEN-INTEGRATION-001・QWEN-FORMATS-001。再利用設計の後、利用者が共通ライブラリ化と本体組込みを承認した。未コミットの実装を含む。利用者によるWAVの実接続・操作確認は正常動作として受入済みである。発音評価の正確性や追加形式の実API受入は別である。
共通方針は[全体設計](README.md)、文書の受入条件は[仕様](../../tasks/REUSE-DESIGN-001/spec.md)を参照する。
以下の「現行」は未コミット変更を含むソースの確認結果、「proposed」は未実装の目標契約、「未検証」は追加の実行証拠が必要な事項である。

## QWEN-UX-002追補：音声送信とは別の接続確認

`probe_connection(Connection, Arc<dyn ProbeTransport>, CancellationToken)` は同じWorkspaceのモデル一覧をGETで問い合わせる共通APIである。評価のPOST用Transportとは分離し、音声・参照文を要求に含めない。接続10秒・全体30秒・本文256KiB、転送/再送/別hostへの迂回は禁止である。成功は対象モデルの一覧掲載確認に限定し、推論権限・音声品質・費用を保証しない。

ホストUIは確認対象のdraftを固定し、変更・取消し・閉じる・保存で古い確認結果を失効させる。接続確認を保存操作として扱わない。本体ではCodexと同じAI接続ページにQwen編集を展開するが、Windows資格情報の専用保存/取消しと一般設定の保存境界は分ける。単独GUIの設定画面は今回のUX変更対象外である。

評価要求には公式対応のJSON Object指定を追加した。構文不正（ResponseJsonInvalid）、項目の欠落/不整合（ResponseFieldsInvalid）、その他の応答検証失敗（ResponseInvalid）を固定コードで区別し、raw応答は診断へ保存しない。SafeErrorのJSONはcode/message/send_dispositionの3項目を維持する。エラーは発音不良の判定ではなく、適合したreference_mismatchだけを別英文の結果として扱う。

詳細・未検証範囲は[設計](../../tasks/QWEN-UX-002/design.md)と[調査根拠](../../tasks/QWEN-UX-002/evidence.md)を参照する。実API再発原因や6地域の稼働を自動テストだけで確認済みとはしない。

## 1. 用途と対象外

用途は、利用者が選んだ原音と読む予定の英文を照合し、聞こえた内容と日本語の練習助言を得ることである。
音声を聞かず参照英文だけから正解を補うこと、試験の客観点数・検証済み音素正解率を保証することは対象外である。
録音デバイス、音声変換、教材登録、会話保存、アカウント契約、課金管理はホストアプリが担う。
WordWeave5では教材の「主な例文」から専用評価画面を開く。チャットと教材案作成は引き続きCodex app-serverであり、Qwenは明示的に選ぶ音読評価専用である。Codex失敗時のAPIキーfallbackにはしない。

## 実装版の利用境界

共通crateは `qwen-audio 0.1.0`（[manifest](../../tools/qwen-audio/Cargo.toml)）である。両アプリは同じ `AudioInput`、`ReferenceText`、`EvaluationSnapshot`、`evaluate`、応答検査を呼ぶ。教材型や成績を共通crateへ持ち込まない。

| 利用profile | Cargo指定 | 含むadapter |
| --- | --- | --- |
| coreのみ | `default-features = false` | 入力・宛先検査、HTTP/SSE、結果検査。GUI/MCP/OS資格情報なし |
| WordWeave5 | `default-features = false, features = ["windows-credentials"]` | core＋Windows資格情報。画面・録音・承認・寿命は本体 |
| 単独GUI/MCP | default | core＋`gui,mcp,windows-credentials,windows-job` |

他プロジェクトのpath利用例（実際の配置に合わせる）：

```toml
[dependencies]
qwen-audio = { path = "../WordWeave5/tools/qwen-audio", default-features = false }
```

`evaluate(snapshot, transport, cancel)`は信頼された呼出元向けであり、人の同意を証明しない。本体の[ReadingController](../../src/qwen_reading.rs)は入力固定→`prepare()`→確認表示→人のボタン→`human_send(PreparedId, runtime)`を実装する。確認IDは画面所有者・世代・準備番号を持ち、入力/設定変更で失効する。他のhostも同意画面と取消し契約の実装・検証が必要である。

資格情報は単独 `WordWeave5.QwenAudio.Connection.v1` と本体 `WordWeave5.QwenReading.Connection.v1` を分離する。`WindowsCredentialStore::new()`は旧名、`for_target()`は別名を指定できる。キーの暗黙コピー・共有をせず、保存失敗は旧接続を保持する。

本体は専用Recorder/Speakerを使い、英文は教材全fieldの版とともに固定する。教材変更・消失で無効化し、開き直す。本体の音声・助言は画面を閉じるまでのメモリだけに置き、教材/成績/チャットへ保存・送信しない。単独側のsession cacheとは異なる寿命である。

下記proposed節の汎用名・保存拡張は将来候補であり、この実装で公開したAPIではない。正本は[組込み仕様](../../tasks/QWEN-INTEGRATION-001/spec.md)と[内部設計](../../tasks/QWEN-INTEGRATION-001/design.md)である。

## 2. 現行と切り離す責任

### 地域・結果分類・設定UIの拡張（QWEN-SETTINGS-001）

[仕様](../../tasks/QWEN-SETTINGS-001/spec.md)・[設計](../../tasks/QWEN-SETTINGS-001/design.md)を追加契約とする。下記の旧Tokyo限定記述はこの節で拡張する。

- `Region`と`ApiHost`は6地域のWorkspace専用URLを検証する。旧`TokyoHost`は東京限定の意味を保持して`ApiHost`へ変換できる。
- `Connection::with_host`は`Result`となり、保存済みキーを保持できるのは同じ正規化URLだけである。別地域・別Workspaceには新しいキーを明示的に入力する。従来の無条件host差替えは安全のため廃止する。Windowsレコードv1とホストアプリ別namespaceは維持する。
- `Assessment::ReferenceMismatch`を追加する。聞き取った英文を必須、改善/良かった点は空配列とし、例文を読んだと仮定した評価をしない。正常な評価不能・不正応答と区別する。旧消費者は新enumを知らないため、共通crateと両アプリを一緒に更新する。
- 本体接続設定は設定→AI接続のQwen専用modalへ移動する。Progress保存と資格情報保存を同一transactionと装わず、専用保存/キャンセルを表示する。dirty終了要求を確認し、保存失敗でactive接続を切り替えない。
- 判断や助言の正確さはモックの合格から保証しない。地域ごとの実アカウント接続・課金・実音声精度は別受入である。

### 多形式入力の追加契約（QWEN-FORMATS-001）

詳細正本は[形式仕様](../../tasks/QWEN-FORMATS-001/spec.md)と[内部設計](../../tasks/QWEN-FORMATS-001/design.md)である。後続の旧WAVのみの記述はこの差分で拡張する。

- `AudioInput::parse`/`read_wav`は既存のstrict PCM16 WAV入口を維持する。多形式の同期入口は`decode(bytes, cancel)`/`read_audio(path, cancel)`、GUIは非同期の`AudioLoadJob`を使う。
- `bytes()`は検査時に固定した元bytes、`wire_format()`は検査した形式、`playback_wav()`は再生専用PCMである。送信時に再生用bytesを使わず、元pathを再openしない。
- 対応はWAV/MP3/AAC/AMR/3GP/3GPP。codec/profile制限とFFmpeg/ffprobeの配置条件は[利用説明](../../tools/qwen-audio/README.md#音読を評価する)を参照する。依存は自動導入しない。
- 元ファイル6MiB・復号後60秒・8～48kHz・1/2chを検査する。再生用PCMは元ファイルの6MiBとは別上限であり、出力・process時間も制限する。
- `AudioLoadJob`は取消し/Drop後の成功を採用しない。hostは閉じる・取消し・再選択でjobを手放し、旧確認を失効させる。通常終了は`shutdown_audio_jobs()`で全job取消し後に回収する。
- 3GPだけはシーク検査用の短命コピーを作る。通常の成功/失敗/取消し/終了で削除し、`take_audio_cleanup_warning()`を画面更新中とshutdown後に回収して削除失敗を知らせる。強制終了時の残留可能性を利用者へ説明する。これは履歴保存の許可ではない。
- 本体確認欄は音声と英文を表示する。接続情報は設定欄に残す。毎回の人の送信操作と単独ツールの結果受取境界は変更しない。

| 根拠となる入口・型 | 現行の責任 | 再利用時の境界 |
| --- | --- | --- |
| [lib.rs](../../tools/qwen-audio/src/lib.rs)、[Cargo.toml](../../tools/qwen-audio/Cargo.toml) | lib/bin、明示exportとoptional featureでGUI/MCP/Windowsを分離 | 将来の破壊的公開面縮小は別移行。既存Transport試験型は維持する |
| [AudioInput::parse / ReferenceText::new](../../tools/qwen-audio/src/audio.rs) | WAV実体と英文を検証し、所有する | 処理部に残す。ファイル選択・録音・表示名はホストに置く |
| [EvaluationSnapshot / evaluate / Transport](../../tools/qwen-audio/src/provider.rs) | 入力を固定し、HTTP送信・SSE解析・結果検査を行う | 評価契約とQwen接続を分け、Transportを偽応答の試験口として保持する |
| [TokyoHost / Connection / CredentialStore](../../tools/qwen-audio/src/credentials.rs) | 宛先検査と資格情報の読書き | 宛先検査は接続部、キー保存はアプリ別adapterに置く |
| [GuiController::send / prepare_intent / grant](../../tools/qwen-audio/src/gui.rs) | 人の確認から送信を開始、MCP時は親子の許可手順を管理 | UI/controllerの責任であり、評価lib単体の同意保証としない |
| [SessionManager](../../tools/qwen-audio/src/session.rs)、[MCP処理](../../tools/qwen-audio/src/mcp.rs) | 同時1件、終了結果16件のメモリ保持、start/get/cancel | ローカルsession管理とMCP adapter。providerの結果再取得機能ではない |

現行の公開 `evaluate(snapshot, transport, cancel)` は信頼された呼出元向けである。呼出しだけでGUI確認を強制しない。
GUI/MCP/libの存在と、実API受入・他製品への配布完了とは別である。二消費者は同一crateを利用し、同じ偽応答fixtureで結果を検査する。
現行のTokyo宛先、`qwen3.8-omni-flash`、`medium` はソースの制限であり、providerが現在保証する仕様とは主張しない。

## 3. 目標の公開契約（proposed）

論理名 `reading-evaluation` とQwen接続部を想定する。crate名・署名・公開版は未確定であり、下表は実装済みAPIではない。
型の内部fieldは非公開とし、検査済みconstructorと必要な読取専用viewを入口にする。将来の公開面縮小ではTransportのwire型の扱いを検討するが、現行の試験用exportは互換性のため維持する。

| 仮称の操作・型 | 入力 | 出力・不変条件 |
| --- | --- | --- |
| `prepare_evaluation` | 所有WAV bytes、参照英文、Qwen宛先、用途、結果受取先、接続選択 | `PreparedEvaluation`。本文・音声・宛先・受取先を不変snapshotとして所有し、外部送信しない |
| `PreparedEvaluation::preview` | 準備済み入力 | 音声情報、参照全文、宛先、指定model/effort、結果の受取先。秘密キーは出さない |
| `EvaluationApproval` | ホストの人による確認操作と準備ID | その準備だけに対応する一回の実行許可。別入力への流用は拒否 |
| `start_evaluation` | 準備済み入力をmove、一致する許可、接続adapter、取消token | `EvaluationHandle`。同じ準備の二重実行を拒否し、workerの所有者を明確にする |
| `EvaluationOutcome` | workerの終了イベント | 検証済み結果、固定エラー分類、送信確実性。完了とローカル停止を区別する |
| `cancel` / `get_local_result` | 自分のhandleまたはsession ID | 取消要求または既存結果の読取り。新規HTTP要求を生成しない |

承認後に元パスを再読込せず、確認した音声bytesと英文を送る。変更後は再準備・再確認が必要である。
現行`EvaluationSnapshot`は`Connection`も所有する。目標の許可契約ではその接続snapshotまたは接続世代を結合し、確認後の資格情報変更で別アカウントへ黙って送らない。秘密値は確認表示・ログへ出さない。
この承認型は信頼されたホストの誤用防止であり、同一プロセスの悪意あるホストに対する暗号的な同意証明ではない。
独自UIで同意を省いた呼出元まで「同意済み」と保証しない。組込み側の受入試験で人の操作からの連結を確認する。

### 入力・結果・上限

| 項目 | 現行の検証 | 目標契約での扱い |
| --- | --- | --- |
| WAV | PCM16、1/2ch、8–48kHz、60秒以下、全体6MiB以下、RIFF構造検査 | 同じ検査helperを再利用。変換・切詰めで黙って適合させない |
| 参照英文 | 空白のみ不可、Unicode scalarで10,000以下 | 正本と表示を分ける。入力は命令ではなく評価対象データとして扱う |
| 宛先 | `TokyoHost::parse` のhttpsとTokyo hostname/path制限 | 任意URLの汎用HTTP clientにしない。範囲拡大は別の仕様判断 |
| 応答 | wire 1MiB、助言JSON 65,536 bytes、接続10秒・評価180秒 | 現行値を互換profile候補とする。providerの保証値や将来全host共通値にはしない |
| 助言 | `Feedback` の必須項目・上限・参照部分文字列を検査 | `Assessed`と`Unassessable`を区別。形式適合から内容の正しさを推定しない |
| metadata | 指定値と`actual_model/actual_effort/usage`を別保持 | 返却なしは未取得。指定値や文字数から返却値・token数を作らない |

結果は現行 [Feedback / EvaluationResult / validate_feedback](../../tools/qwen-audio/src/provider.rs) の意味を保つ。
60秒/6MiBはQwen入力の現行上限であり、WordWeaveの録音UIにある30秒上限を変更する承認ではない。
`Unassessable`は通信失敗ではなく「安全に評価できない」という検証済み結果であり、理由を表示する。
HTTP本文・秘密・音声をエラー文字列へ載せず、[SafeError / SendDisposition](../../tools/qwen-audio/src/error.rs) の分類方式を継ぐ。
入力、認証、制限、通信、timeout、応答不正、取消し、busyを機械判定し、説明の日本語はadapterが扱えるようにする。

## 4. 所有・非同期・保存

| 対象 | 所有者 | ライフサイクル |
| --- | --- | --- |
| 元録音・英文正本 | ホスト | 選択・取消し・失敗で削除しない。再選択するまでsnapshotへ変更を混入させない |
| 検証済みsnapshot | 評価handle | 1実行に専有。完了後の破棄と永続保存は区別する |
| キー | アプリ別CredentialStoreと接続部 | Windows資格情報のキー名にアプリ名前空間を与える提案。別アプリのキーを暗黙共有しない |
| runtime・取消token | ホストから評価workerへ | UI threadをブロックしない。終了時は自分のworker/子プロセスを回収する |
| ローカルsession・結果 | session adapter | 現行はプロセス中の16件cache。保持延長・永続化は別途仕様化する |
| 利用履歴・教材・結果保存 | ホスト | 保存成功を確認してから保存済みと表示。評価部に固定保存パスを持たせない |

現行は音声・英文・結果の永続化を行わない。資格情報保存と評価実行は別であり、設定保存だけで課金APIを呼ばない。
今回の本体adapterは原音を保持し、準備・同意・開始後に結果をメモリ表示する。以下の保存は将来の別仕様であり、今回実装しない。
結果保存に失敗しても再評価せず、同じ取得済み結果の保存を再試行する。保存前に終了したメモリ結果の回復は保証しない。
実行IDはローカルで関連付けるためであり、provider側の重複課金防止キーとは主張しない。

## 5. sequenceと失敗の意味

通常は「入力検査 → 不変snapshot → 内容/宛先/受取先の提示 → 人の送信操作 → worker開始 → 応答検査 → 表示 → 任意保存」である。
現行 `AwaitingUser → Running → Completed / Failed / Cancelled` に、`SendDisposition`を合わせて読む。
目標でも巨大な共通enumへまとめず、ローカル状態・送信確実性・結果確実性・保存状態を独立に保持する。

| 場面 | 観測する状態・ホストの対応 | 禁止する解釈 |
| --- | --- | --- |
| 確認画面で取消し | 未送信で終了、原音と入力を保持 | 評価開始・課金が必要な取消操作 |
| 送信前に入力/宛先が変化 | 準備の不一致として停止、再確認 | 以前の許可を新入力へ転用 |
| HTTP開始後に取消し/timeout/切断 | `MayHaveBeenSent`、結果不明の可能性を表示 | ローカル待機終了から処理・課金取消しを断定 |
| 確認済みHTTP拒否/不正応答 | 失敗分類を表示。応答不正は相手側実行の不存在を意味しない | 不正結果を助言として採用、自動再送 |
| 正常結果後の保存失敗 | 評価完了・未保存を併記。結果が残る間は保存だけ再試行 | 保存修復のために新たに評価 |
| MCP getで見つからない | cache失効・process終了も含むローカル不在 | API側から回復できた、評価が実行されなかったという推定 |

現行Qwenには遠隔の確認済中断を取得するAPI契約がない。取消完了表示はローカル停止の範囲に限る。
再試行はホストが結果不明・費用の可能性を提示し、人が再送を選び、新しい準備と承認を作る操作である。
MCPへ返す場合は結果がCodex側会話に保存され得ることも確認対象に含め、キーや音声bytesをMCP結果へ返さない。

## 6. UI adapterと互換性

画面部は処理部から分離し、日本語全文・評価不能理由・送信後不明・取消操作へ狭幅でも到達できるようにする。
egui hostは[既存controls](../../tools/qwen-audio/src/controls.rs)の可視字形の両軸中央配置を維持する。日本語の行box中心だけで合格としない。
アイコンを含む群は別に配置検査し、通常のボタンのfocus・keyboard・disabled・hover・accessible labelを保持する。
拡大文字と狭幅で縦横の切れを検査し、bodyをscroll可能にする。Windows DPI・IME・screen readerの実機確認はgeometry試験と別である。

既存package内で処理部のexportを明示し、GUI/MCP/資格情報adapterをfeatureで切り分けた。万能providerやREST layerは作らない。
既存呼出しに互換wrapperを置く移行案だが、trusted `evaluate`を同意付き入口へ置換する差分は明示して呼出元を移す。
公開型の版、SSE/parserのprovider互換、資格情報recordの版、結果保存schemaを別管理する。厳密JSON検査の変更も契約試験で検知する。
同一リリース版を二消費者へ導入してから旧入口を廃止する。失敗時は旧adapterへ戻し、現行の入力/キー形式を保持する。
資格情報の名前空間変更はコピー/移行の確認手順を別途設計し、旧キー削除や別アプリ共有を自動で行わない。

## 7. 二消費者と最小利用例

以下は実装済みhostの概略であり、汎用の同意保証APIではない。

```text
単独ツール: WAV選択 → GuiController確認 → 人の送信 → 同じevaluate → メモリ助言
WordWeave: 新規録音/WAV選択 → ReadingController.prepare → 固定内容を人が確認
          → human_send → 同じevaluate → メモリ助言（閉じると破棄）
```

両消費者のadapterは録音取得・CredentialStore・画面・保存だけを差し替え、評価部に`Entry`や`Progress`を要求しない。
同じ合成WAV/英文で同じ検証結果、正常/評価不能、6MiBと60秒の直前/一致/超過、取消し、切断、不正SSEを観測する。
同意変更時に送信0回、送信後timeout時に自動再送0回、秘密を含む偽応答の拒否、保存失敗時にHTTP追加0回を確認する。
mockの`Transport`、偽`CredentialStore`、時計/timeout制御、session cache境界が試験口である。core-only buildと同じfixtureの両controller試験は[組込み検証記録](../../tasks/QWEN-INTEGRATION-001/results.md)を参照する。mock合格を実API成功・発音判定の正確性とは扱わない。

## Framework観点の補足（2026-10-03）

[教科横断Framework](framework.md)からは、教科の評価adapterが必要な場合だけ本部品を呼ぶ。現在の対象は参照英文を読む練習への助言であり、国語の朗読評価・理科の正誤採点・成績認定へ拡張したとは扱わない。
録音/再生は[音声I/O](audio-io.md)、原本保持は[asset管理](asset-store.md)の境界候補とするが、これらの導入は必須にしない。ホスト提供の検証済みWAVでも評価部品を利用できる。
機器から取れた音声やasset保存済み音声が、本部品のPCM・sample rate・長さ・サイズ条件を満たすとは限らない。送信先adapterが再検査し、変換が必要なら原音とは別の派生物として準備し、送信対象を確認する。
助言は教材/問題/回答の固定版と実行IDへ関連付ける。助言の取得、教科としての評価確定、試行記録の保存は別であり、評価不能を不正解・0点へ変換しない。具体的な評価尺度と品質の受入は教科側に残す。
この補足以外の通信、同意、取消し、キャッシュ、互換性の契約は維持する。今回の文書確認は[追補結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)を参照する。

## 8. 追跡・未決事項・引継ぎ

| AC | この文書の証拠 | 将来の実装/テスト作者への焦点 |
| --- | --- | --- |
| AC01–02 | 1–4節、現行symbol参照 | 公開面の縮小、型とホスト依存の分離、上限境界 |
| AC03–04 | 1・3–5節 | 明示選択・送信同意・不明・保存の独立性 |
| AC05–06 | 6–7節 | wrapper移行、同一版の二adapter、失敗時復帰 |
| AC07 | 本表と[今回の結果](../../tasks/REUSE-DESIGN-001/results.md) | 親によるリンク・文書検査と独立レビュー |

[既存結果](../../tasks/QWEN-AUDIO-001/results.md)の81 tests等は過去証拠であり、今回再実行していない。
実キー、Tokyo接続、助言品質、利用者の音声、実MCP登録は未検証である。[既存設計](../../tasks/QWEN-AUDIO-001/design.md)も現行範囲の補助資料である。
今回の導入方式・公開署名・保存しない初版範囲は組込み仕様で確定した。追加provider、助言の品質基準、公開配布、保存の拡張、キー移行UIは今後の仕様判断である。
設計案だけでこれらを確定せず、実装/試験作者は承認後に既存helperを移植し、実API試験をmock合格と分けて報告する。
