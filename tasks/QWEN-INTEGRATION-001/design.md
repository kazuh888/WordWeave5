# QWEN-INTEGRATION-001 内部設計

2026-10-03。状態：承認済み仕様を入力とする設計案。独立設計レビュー・計画精緻化前であり、実装済みの主張ではない。
入力は [spec.md](spec.md)（QI-AC-001～021、P1～P5親採用済み、独立仕様レビューP1/P2なし）、[plan.md](plan.md)、[再利用契約](../../docs/reuse/qwen-audio.md) である。
U0の計画者指定は `gpt-6-astra/high`。実行metadata・開始/終了token counter・親子包含は未取得である。所有は本書のみ、基点は計画のcommitと既存dirtyである。
完了境界は既存コード照合、契約・状態・試験口の記述、文書確認と親への引継ぎである。実装・テスト編集・実API・公開・再委任は行わない。
適用Skillは `agent-skills:api-and-interface-design` と `wordweave-egui-ui`。契約を型へ置き、日本語操作部は既存両軸中央配置helperを使う。

## 1. 構成と既存根拠

| 担当境界 | 採用する既存要素／変更先 | 所有するもの／持ち込まないもの |
| --- | --- | --- |
| 共通評価 U1 | `tools/qwen-audio/src/{audio,provider,error,credentials}.rs` | 検証済みWAV/英文、接続、HTTP/SSE検証。Entry、成績、GUI、保存先を要求しない |
| 単独adapter | 同package `gui.rs/session.rs/ipc.rs/mcp.rs` | 現行GUI/MCP、session cache、旧資格情報名を維持する |
| 本体controller U2/U3 | root `src/qwen_reading.rs`、必要なworker分割は `src/qwen_reading/worker.rs` | 準備、接続世代、実行、取消し、メモリ結果。録音機器・widget・教材保存は所有しない |
| 本体UI U4 | `src/app/qwen_reading_ui.rs`、入口 `src/app/materials_ui.rs`、配線 `src/app.rs` | 対象Entryの版、専用Recorder/Speaker、設定draft、runtime、dialog寿命 |
| native証拠 U4 | 既存 `src/app/visual_check.rs` | 隔離store・合成controllerによる実画面。実資格情報を読まない |

`materials_ui.rs::material_detail` の「主な例文」は `Entry::completed()` を表示する。この直下へ入口を加え、同じ全文を固定する。
`EvaluationSnapshot::new` は音声・英文・Connectionを所有し、`evaluate` はTransportとCancellationTokenを受ける。評価ロジックを本体へ複製しない。
`Recorder::start/finish_with_warning/cancel/elapsed/error` は既存30秒sample capを持つ。192kHzまで録音できる事実をQwen適合と混同しない。
`Speaker::play_wav/stop/snapshot` と `SpeechPlayback::drop` はメモリstreamと停止・Closeを持つため再利用する。`media::playback` は一時ファイルを作るので採用しない。
`Entry::fingerprint()` は補助例文変更を除外し、`to_tsv()` は改行等を正規化する。対象版は `serde_json::to_vec(&Entry)` の全field bytes＋idで比較し、参照全文も一致を検査する。

## 2. U1 featureと公開面

package名/versionは既存 `qwen-audio 0.1.0` のまま、rootは同一path依存を使用する。二重実装、新REST層、汎用provider/frameworkは追加しない。

| feature | 依存・module |
| --- | --- |
| 無指定core | audio、credentialsの検証型/trait、error、provider。reqwest/futures/serde/tokio time・sync・runtime/tokio-util/zeroize/base64 |
| `mcp` | session/ipc/mcp、optional uuid、tokio io-util/io-std/process。評価本体はsession/UUID不要 |
| `gui` | `mcp`、optional eframe/rfd/image、controls/gui。現行GuiControllerのsession/IPC依存を保つ最小切出し |
| `windows-credentials` | optional windowsのFoundation/Security_Credentials。WindowsCredentialStoreと関連record/testを `all(windows, feature=...)` で囲む |
| `windows-job` | `mcp` とwindowsのFoundation/Security/JobObjects/Threading。ipcのJob/NativeGuiLauncher/NativeProcessのみを同条件で囲む |
| `default` | `gui,mcp,windows-credentials,windows-job`。既存単独GUI/MCPを維持 |

bin `qwen-audio` の `required-features` は上記4機能。各optional dependencyは `dep:` で指定し、windows crateのfeatureを資格情報とjobで別々に有効化する。
root Windows依存は `qwen-audio = { path = "tools/qwen-audio", default-features = false, features = ["windows-credentials"] }`、tokio runtime/time/sync、tokio-util、zeroizeを追加する。
coreの明示exportは `AudioInput,AudioInfo,ReferenceText,read_wav,TokyoHost,ApiKey,Connection,CredentialStore,SafeError,ErrorCode,SendDisposition,EvaluationSnapshot,evaluate,ReqwestTransport,EvaluationResult,Feedback,Assessment,Improvement,Usage,validate_feedback` と既存定数である。
`Transport,ProviderRequest,TransportResponse,TransportFuture,TransportFailure,BodyStream` は既存試験用契約として明示exportを保つ。wireの秘密key accessorは公開しない。既存公開名の削除はこの差分に含めない。
feature付きmoduleの既存公開名もglobから明示exportへ変更して保持する。GUIなしcore試験にGUI/MCP testを混入させず、既存integration testには対象featureのcfgをテスト作者が付ける。

## 3. U2 接続と永続化順序

`WindowsCredentialStore::new()/Default` は旧 `WordWeave5.QwenAudio.Connection.v1` を維持する。追加 `for_target(target: &'static str) -> Result<Self, SafeError>` は空/NUL/過長を拒否し、private targetを固定する。
本体定数 `WORDWEAVE_QWEN_CREDENTIAL_TARGET = "WordWeave5.QwenReading.Connection.v1"` を用いる。旧recordへのfallback、列挙、コピー、削除を行わず、record schema v1は維持する。
保存済みkey非表示のhost変更を可能にする追加 `Connection::with_host(&self, host: TokyoHost) -> Self` は内部keyをcloneするだけで外へ返さない。
本体の `SettingsDraft` はhostと `Option<Zeroizing<String>>` の入力だけを所有する。Noneは保存済みkey維持、空を明示入力したSomeはInvalidKeyである。UIの空欄はNoneへ変換し「空欄なら保存済みキーを維持」と表示する。
順序はdraft検証 → Connection候補生成 → `CredentialStore::save` 成功 → controllerの接続差替え・世代更新 → 準備破棄 → 保存成功表示。save失敗/取消しでは旧接続と旧世代を保持する。
load失敗は未設定と区別してSafeError表示し、平文ファイルへ代替しない。設定はdialog内の同じ画面で編集し、Running時は開始・保存とも拒否する。
外部アプリによるOS record更新を監視する機能は加えない。dialogは読み込んだConnectionを固定保持し、送信時に資格情報を再読込しない。再open時のloadは新しい接続世代になる。

## 4. U3 公開controller契約

root `src/lib.rs` に `#[cfg(windows)] pub mod qwen_reading;` を追加し、独立integration testから本番と同じ入口を使えるようにする。
以下は署名固定案である。型の状態fieldはprivate、返却viewは秘密・生bytes・接続keyを含めず、UIから状態を代入させない。

```rust
pub struct ReadingTarget { /* id, exact version bytes, validated reference */ }
impl ReadingTarget { pub fn new(id: String, version: Vec<u8>, reference: String) -> Result<Self, SafeError>; }
pub struct PreparedId { /* owner nonce + controller generation + preparation serial; Copy/Eq, private */ }
pub enum ReadingPhase { Input, Confirming, Running, Completed, Failed, Cancelled, Invalidated, Closed }
pub struct ReadingView { /* phase, target label, AudioInfo?, preview?, result?, SafeError?, disposition */ }
pub struct PreparedPreview { /* id, label, AudioInfo, full reference, host, requested model/effort, purpose/recipient */ }
pub struct ReadingController { /* target, input, connection, prepared, generation, worker */ }
impl ReadingController {
    pub fn new(target: ReadingTarget, store: Arc<dyn CredentialStore>, transport: Arc<dyn Transport>) -> Result<Self, SafeError>;
    pub fn view(&self) -> ReadingView;
    pub fn set_audio(&mut self, label: String, bytes: Vec<u8>) -> Result<(), SafeError>;
    pub fn clear_audio(&mut self) -> Result<(), SafeError>;
    pub fn begin_settings(&mut self) -> Result<SettingsView, SafeError>;
    pub fn set_settings_draft(&mut self, host: String, key: Option<String>) -> Result<(), SafeError>;
    pub fn save_settings(&mut self) -> Result<(), SafeError>;
    pub fn cancel_settings(&mut self);
    pub fn prepare(&mut self) -> Result<PreparedPreview, SafeError>;
    pub fn human_send(&mut self, id: PreparedId, runtime: &tokio::runtime::Handle) -> Result<(), SafeError>;
    pub fn poll(&mut self);
    pub fn cancel(&mut self);
    pub fn invalidate_target(&mut self);
    pub fn restart(&mut self) -> Result<(), SafeError>;
    pub fn close(&mut self);
}
```

ReadingViewのpublic fieldは `phase: ReadingPhase, target_id: String, audio_info: Option<AudioInfo>, preview: Option<PreparedPreview>, result: Option<EvaluationResult>, error: Option<SafeError>, disposition: SendDisposition` とする。new時のload失敗はerrorへ保存し設定修復を可能にする。
PreparedPreviewのpublic fieldは `id: PreparedId, audio_label: String, audio_info: AudioInfo, reference: String, host: String, requested_model: &'static str, requested_effort: &'static str, purpose: &'static str, recipient: &'static str`、SettingsViewは `host: String, credential_present: bool`。Viewはowned snapshot、Phase/IdはCopy/Eqである。
set_audioは変更を先に反映して旧確認と旧有効音声を破棄し、`AudioInput::parse` の成否を保持する。失敗後に以前の有効音声を送れる状態へ戻さない。ファイル選択取消しはset_audioを呼ばない。
UIが専有する原音Vecと、controllerの検証済みAudioInputは別所有である。録音が非対応でも原音は再生用に残り、prepareは拒否する。ファイル選択確定後はread前にclear_audioし、読取失敗で旧入力を採用しない。読取は6MiB+1で打切り、元パスを送信時に再読込しない。
prepareは検証済み入力と接続をcloneして既存EvaluationSnapshotへ封入し、private PreparedReadingに一つだけ保持する。Previewはそのsnapshotから生成し、表示と送信の別経路組立てを避ける。
PreparedIdは同一controllerの現在準備だけに有効である。human_sendはid一致・Confirmingを検査し、preparedをtakeしてRunningへ移してからspawnする。二回目はInvalidState/Busy、要求追加0である。
owner nonceはプロセス共通AtomicU64（初期値1）のfetch_updateとchecked_addで一度だけ割当て、再使用しない。枯渇時はnewがInvalidState/NotSentを返す。generation/serialもwrapさせず枯渇時はInvalidatedへ移す。human_sendは三要素すべての一致を要求する。
typed入口は信頼されたhostの誤用防止であり人の操作の証明ではない。UIの送信Button.clicked経路だけがhuman_sendを呼び、poll/timer/復元/keyboardの無関係なEnterからは呼ばない。

## 5. 状態・非同期・取消し

| 入力状態／操作 | 次状態と不変条件 |
| --- | --- |
| Input／prepare成功 | Confirming。要求0、音声/英文/接続・宛先/目的/受取先を固定 |
| Confirming／入力または接続保存成功 | Input。PreparedId失効。save失敗は旧設定を保持 |
| Confirming／human_send | Running。評価snapshotを一回move、要求最大1、入力/設定/第二dialog禁止 |
| Running／正常または評価不能 | Completed。検証済み結果だけ採用。評価不能も正常な終端結果 |
| Running／失敗 | Failed。SafeError分類と送信確実性。自動再試行なし |
| Invalidated/Closed以外／cancel | Cancelled。未送信/送信可能性を表示し、旧成功の採用を禁止 |
| Invalidated/Closed／cancel | 状態維持。残存workerの取消し・回収だけを冪等に行い、prepare/human_send/restartはInvalidState/NotSentで拒否 |
| 対象版不一致／invalidate_target | Invalidated。準備破棄、worker取消し、再open要求。旧参照のまま再準備不可 |
| Completed/Failed/Cancelled／restart | 旧worker回収完了後だけInput。未回収ならBusyで状態維持。結果/確認を破棄、入力は同dialog内で保持し、次のprepare・人の送信が必須 |
| 任意／close | Closed。worker取消し、入力/設定draft/結果破棄。復元・自動送信なし |

runtimeはQwen UI hostが必要時に一つ生成し保持する。生成失敗は明示して送信しない。workerは `evaluate(snapshot, transport, token)` のtokio task、UIはpollのみでblock_onしない。
各実行は単調増加generationとCancellationToken、JoinHandleを所有する。close/invalidate/cancelはまず世代を無効化し、token.cancel、handle.abortを行う。Dropも同じ掃除を保証する。
取消されたhandleは非blockingで回収可能なreaperへ移し、完了/abortを確認して破棄する。アプリ終了時は全token取消し → 全handle.abort → UI所有 `Option<Runtime>::take()` → `shutdown_background()` とし、Runtimeの通常DropでUIを待たせない。workerはRuntimeを所有せずspawn_blockingも使わない。
pollは終端状態でもreaperを進め、回収済み通知を確認する。restartは新generation/tokenを割当て、旧結果を移さない。Invalidated/Closedから入力・設定操作を経由してもInputへ戻れず、新controllerの生成だけが再開入口である。
結果採用条件は「現在Running、同じgeneration、token非取消し、対象版一致」の全条件である。UIはそのframeの閉じる/取消し/対象変更を先に適用してからpollする。
送信確実性はworker用Transport decoratorの共有send gateで記録する。cancelとsend開始を同じmutexで直列化し、取消し先行なら内部Transportを呼ばず、send開始先行ならMayHaveBeenSentを固定する。
このgateは通信の遠隔取消保証ではない。core SafeErrorの分類を保ち、ローカルでNotSentを証明できる場合のみ未送信とする。timeout/切断/送信後取消しで自動再送しない。
本体はSessionManager/UUID/cacheを使わない。別dialogは別controllerとgeneration所有者であり、旧handleの結果を新controllerへ移さない。

## 6. U4 UIと資源の手順

専用dialogはegui 0.31.1の `Modal` を用い、同時一つ。親navigation/material編集は `qwen_dialog.is_some()` で明示無効化し、背景shortcut/drop/既存確認windowからも対象変更を受けない。
入口は他の録音/再生worker、教材変更作業、既存確認dialogが活動中なら理由付きで拒否する。既存Speakerはstop成功を確認し、停止失敗時は開かない。既存未保存録音を破棄しない。
QwenDialogは専用 `Option<Recorder>` と専用 `Speaker` を持つ。rootのrecorderへ入れると既存tick/stop_recordingが学習/チャットへ保存するため共有しない。機器の利用排他はhostが保持する。
録音開始では自分の再生をstopし旧確認を無効化する。30秒またはRecorder.errorで `finish_with_warning` を一度呼び、原音bytesを保持して再検証する。warningは安全な日本語分類で表示する。
開始/finish失敗は旧確認を失効させ無効音声表示とする。有効な部分録音とwarningが返った場合は両方を保持し、音声適合の判定だけは共通parserへ委ねる。
再生は専用Speaker.play_wav(raw_bytes,1.0)。snapshotで遅延MediaFailedを検知し、再生失敗は可視notice、入力は保持する。録音中は再生/WAV選択/準備不可、再生中の準備は停止成功後に行う。
閉じる/Escape/viewport終了/対象無効化は一つのcleanupへ集約する：controller取消し → Recorder.cancel → Speaker.stopとdrop → draft/key/raw bytes/result破棄。stop失敗でもdropのMediaPlayer.Closeを実行する。
対象は各frameのdeck検索でid・正確な版・参照を比較し、消失/変更なら無効化する。既存tickが適用する非同期教材変更も、結果poll前に検出する。
画面は上から対象/全文、録音またはWAV選択、準備内容、課金・送信先の説明、状態/結果。設定は同Modal内のsubviewで、host入力・伏字key・保存済み有無・固定要求値・保存/取消しを置く。
確認では音声名、PCM/ch/rate/秒/bytes、固定英文全文、正規化Host、要求model/effort、目的、結果受取先、Alibaba Cloud送信/従量課金可能性を省略しない。
結果はheard_text推定、日本語summary/strengths/improvements、評価不能理由、実metadata未取得を区別する。送信後取消し/失敗は遠隔完了・課金不明を表示する。「もう一度練習」はrestartだけを呼ぶ。
本体 `controls::Button/UiControls` を使用し、文字meshの両軸中央・focus/keyboard/disabled/accessible labelを維持する。通常本文の行送りや日本語fontを独自補正しない。
横幅はviewport内へclamp、長文はwrap、操作列は狭幅で縦積みとする。本文は高さ制限付きScrollArea、取消し/閉じるは常時到達するfooterとし、極端な低さでは画面全体scrollへ切り替える。
日本語ラベル・Host/key貼付け・IME、拡大1.0/1.5/2.0、狭幅480×640 logical相当で全文末尾・送信・取消し・結果末尾への到達を検査する。Windows DPI実機は別証拠である。

## 7. 独立試験口・AC対応・handoff

| AC（QI-AC-） | module／独立試験名の固定案 | テスト作者の所有 |
| --- | --- | --- |
| 001,009,011 | target/controller: `target_revision_invalidates_pending_and_late_result`, `snapshot_survives_source_rewrite_or_delete` | root `tests/qwen_reading.rs` |
| 002,003,007 | media/UI: `recording_stops_at_30_seconds`, `unsupported_recording_keeps_playback_bytes`, `selection_cancel_keeps_input_but_failure_invalidates` | `src/app/qwen_reading_tests.rs` |
| 004～006 | credentials/controller: `namespaces_never_read_legacy_record`, `failed_save_keeps_connection_and_calls_zero`, `legacy_constructor_keeps_target`, `invalid_host_calls_zero` | root test＋package `tests/integration_credentials.rs` |
| 007～010 | controller: `prepare_calls_zero_human_send_calls_one`, `double_send_calls_once`, `connection_change_invalidates_preparation`, `invalid_inputs_call_zero` | root `tests/qwen_reading.rs` |
| 011,012 | worker: `cancel_before_send_calls_zero`, `cancel_wins_over_ready_completion`, `closed_dialog_drops_late_result`, `failure_never_retries` | root `tests/qwen_reading.rs` |
| 009～012 | controller: `invalidated_cancel_cannot_restart_or_send`, `foreign_controller_prepared_id_calls_zero`, `restart_waits_for_reap_and_requires_new_confirmation`。別ownerでgeneration/serial一致時もHTTP 0を検査 | root `tests/qwen_reading.rs` |
| 013～016,019 | same core: `two_consumers_same_fixture_outcomes`, `malformed_or_secret_response_never_displayed`, `metadata_absent_stays_unknown` | root test＋package `tests/integration_reuse.rs` |
| 017,018 | host: `qwen_does_not_mutate_learning_store`, `qwen_close_discards_without_recovery`, `codex_routes_do_not_consume_qwen_credentials` | root test/UI test、既存回帰 |
| 018,019 | features: coreなしGUI/MCP/Windows依存tree、既存単独GUI/MCP全試験 | runner、期待値変更禁止 |
| 020 | UI: `qwen_modal_blocks_parent`, `qwen_narrow_scaled_controls_reachable`, `qwen_cancel_processed_before_poll` | `src/app/qwen_reading_tests.rs` |
| 021 | 利用者GUI送信・実metadata/助言判定の分離 | 親/利用者、mock合格と別 |

合成共通fixtureは追跡対象 `tools/qwen-audio/tests/fixtures/reuse/` に置き、両testが同一bytes/英文/正常SSE/評価不能/秘密sentinelをincludeする。単独側は既存GuiController、本体側はReadingControllerを通し同じTransport観測結果で比較する。
core-only testは同fixtureをevaluateへ通す。範囲境界、JSON/SSE不正、401/429/500、180秒timeoutは偽Transportとtokio制御時計で検査し、実ネットワークを不要にする。
偽CredentialStoreは呼出回数・保存失敗を制御し、namespace試験は `for_target` の読取専用target accessorで宛先を検査する。実OS資格情報を使う試験は別のnative受入である。
UI renderはactionを返す小関数へ分ける。`cfg(test)`のwidget rect/focus/scroll markerを記録し、合成Recorder/Speakerのstart/finish/stop結果を注入できるhost境界をテスト用に置く。実機成功を偽値で保証しない。
native hookは `--ui-check <PNG> --qwen-reading <input|confirm|running|result|failed|settings> --small` と `--qwen-scale <1|1.5|2>`、必要時 `--qwen-tail` を追加する。Qwen時のsmallは480×640 logicalを使う（既存smallは820×650）。偽store/Transportで実controllerへ状態を作り、起動時の実credential loadを禁止する。
U1所有はmanifest/lib/ipc/credentials（feature cfg）と必要なmain、U2はcredentials（namespace/constructor）＋root controller/lib、U3はroot controller/worker/Cargo.toml、U4はapp.rs/qwen_reading_ui.rs/materials_ui.rs/visual_check.rsとする。credentials.rsのU1→U2は直列とする。lockは親、テストmodule配線は実装者、test本文は独立作者で直列化する。
必須gateは計画の全コマンドを維持する。追加はcore-only `integration_reuse` 試験、root `--test qwen_reading`、UI focal test、feature tree、native画面証拠である。親の着手前fmt既存不適合は別記し新規/変更範囲の書式検査を行う。設計文書だけの本工程ではbuildを行わない。

## 8. 重大リスクと凍結条件

採用しない案は、旧資格情報の暗黙共用（別アカウント送信）、既存root録音slot流用（誤保存）、path再読込（確認差替え）、既存file再生helper（一時保存）、session/framework全面導入（本体に不要な寿命/依存）である。
Skillの型契約は人同意を証明しない。送信Buttonからhuman_sendへの接続、取消し先行のframe順、非同期教材変更の版検査を独立UI試験で確認するまで安全性合格にしない。
Windows nativeの録音/停止/IME/DPI、OS資格情報失敗の実挙動、provider実schema/助言品質は未検証である。実API受入未完は開発gate合格と分けて残す。
親への署名凍結前確認点は、U1 ipc cfg所有追加、host用with_host追加、送信gateとhandle回収、UIメモリSpeaker再利用、全field版比較、test/native hookの所有である。仕様の追加判断が必要になれば外部仕様へ戻す。
引継ぎ順は独立設計レビュー → 計画者の所有/署名反映 → 独立テスト作者 → U1～U4実装である。親だけが承認済みQwen例外の規約反映と成果物統合を行う。
