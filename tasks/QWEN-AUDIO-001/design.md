# QWEN-AUDIO-001 内部設計

状態：U0独立レビュー合格・親採用済み。2026-10-02。設計担当のモデル割当は [plan.md](plan.md) の `gpt-6.1-sol / high`、実行メタデータは未取得である。親が承認状態を追記し、確定契約として実装へ渡した。

## 入力・設計根拠・完了境界

[spec.md](spec.md) QA-AC-001～023、[sources.md](sources.md)、AGENTS.md、development-team.md、KPI、API設計とwordweave-egui-uiの全文を読了した。親が採用した8～48kHz、2種のassessmentとheard_text、usage、終端GUI子の次start時回収、Tokyo origin正規化を契約に含める。設計完了は公開Rust境界、AC追跡、障害/取消し/永続化/GUI設計と文書検査までである。実装・テスト編集・委任・公開は行わない。

既存コードの証拠は `src/app.rs:319` のWindows日本語フォント候補、`src/app/controls.rs` のnative Button描画後mesh_bounds補正、ルートCargo.tomlのeframe 0.31.1/rfd 0.15.3/windows 0.56.0である。controls.rsのcfg(test)は本体harnessへ依存するためpath includeは使わない。必要な文字ボタン補正だけを独立crate内へ移す設計である。既存Markdown表示も本体依存を持つため、結果は普通のラベル/カードで表示する。

## 配置・責務と依存方向

独立package `tools/qwen-audio/`、package/binary名 `qwen-audio`、library名 `qwen_audio`、edition 2021とする。独立Cargo.lockを持ち、ルートmanifest/lock、WordWeave本体を変更しない。ルートはworkspace宣言を持たないため、独立manifest指定でビルドする。将来の統合はこのlibraryを利用する工程であり、今回はrootへ依存しない。

| ファイル | 所有する責務 | 依存・禁止する結合 |
| --- | --- | --- |
| src/lib.rs / error.rs | 公開re-export、固定安全エラー | GUI/Win32へ非依存 |
| audio.rs | bounded読込み、RIFF解析、確定音声bytes | 外部API、変換、原音書込みなし |
| credentials.rs | TokyoHost/ApiKey/Connection、CredentialStore、Windows adapter | 対象名一つのみ。列挙/平文fallbackなし |
| provider.rs | immutable snapshot、要求構築、async transport、SSE/結果検証 | GUI/ファイルパスへ非依存 |
| session.rs | 1 active、終端確定、16件FIFO cache | HTTP/子プロセス/資格情報へ非依存 |
| ipc.rs / mcp.rs | typed child pipe、子回収、MCP JSON-RPC | MCPからkey/host/audio/path/送信承認を受けない |
| gui.rs / controls.rs / main.rs | 日本語native表示、編集/明示送信、モード起動 | 同期HTTPなし。秘密を描画modelへ入れない |

依存はGUI/MCP→session/provider→validated input/types、provider→Transport、設定controller→CredentialStoreである。Win32とreqwestは末端adapterである。MCP待受用HTTP/REST層、汎用plugin、DBは作らない。実装単位が5ファイルを超える場合は計画者が分割する。本表は所有割当を上書きしない。

## 公開Rust契約（実装前に固定）

以下をcrate rootからre-exportする。省略記号は型のprivate実装のみであり、公開メソッドの名前/引数/戻り値は固定する。外部integration testはprivate moduleやcfg(test)へアクセスしない。型の内部fieldsは下記public field以外privateである。入力型は検証済み、応答型はuntrusted JSONとの境界で検証する。`Debug`はApiKey/Connection/AudioInput/EvaluationSnapshot/ProviderRequestについて内容を伏せる。

```rust
pub const REQUESTED_MODEL: &str = "qwen3.8-omni-flash";
pub const REQUESTED_EFFORT: &str = "medium";
pub const MAX_AUDIO_BYTES: usize = 6 * 1024 * 1024;
pub const MAX_REFERENCE_SCALARS: usize = 10_000;

pub struct ReferenceText { /* private, Clone */ }
impl ReferenceText {
    pub fn new(text: String) -> Result<Self, SafeError>;
    pub fn as_str(&self) -> &str;
}
pub struct AudioInput { /* private immutable bytes, Clone */ }
#[derive(Clone, Debug, PartialEq)]
pub struct AudioInfo {
    pub byte_len: usize,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: u64,
    pub duration_seconds: f64,
}
impl AudioInput {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, SafeError>;
    pub fn info(&self) -> &AudioInfo;
    pub fn bytes(&self) -> &[u8];
}
pub fn read_wav(path: &std::path::Path) -> Result<AudioInput, SafeError>;

pub struct TokyoHost { /* private, Clone, PartialEq */ }
impl TokyoHost {
    pub fn parse(base_url: &str) -> Result<Self, SafeError>;
    pub fn as_str(&self) -> &str;
}
pub struct ApiKey { /* private zeroizing String, Clone */ }
impl ApiKey {
    pub fn new(value: String) -> Result<Self, SafeError>;
}
pub struct Connection { /* private, Clone */ }
impl Connection {
    pub fn new(host: TokyoHost, key: ApiKey) -> Self;
    pub fn host(&self) -> &TokyoHost;
    pub fn effort(&self) -> &'static str;
}
pub trait CredentialStore: Send + Sync {
    fn load(&self) -> Result<Option<Connection>, SafeError>;
    fn save(&self, value: &Connection) -> Result<(), SafeError>;
}
#[cfg(windows)]
pub struct WindowsCredentialStore { /* private */ }
#[cfg(windows)]
impl WindowsCredentialStore {
    pub fn new() -> Self;
}

pub struct EvaluationSnapshot { /* private, NOT Clone */ }
impl EvaluationSnapshot {
    pub fn new(audio: AudioInput, reference: ReferenceText,
               connection: Connection) -> Result<Self, SafeError>;
    pub fn audio_info(&self) -> &AudioInfo;
    pub fn reference(&self) -> &ReferenceText;
    pub fn host(&self) -> &TokyoHost;
}
pub struct ProviderRequest { /* private body/header/endpoint */ }
impl ProviderRequest {
    pub fn endpoint(&self) -> &str;
    pub fn body(&self) -> &[u8];
}
pub type BodyStream = std::pin::Pin<Box<dyn futures_util::Stream<
    Item = Result<Vec<u8>, TransportFailure>> + Send>>;
pub struct TransportResponse {
    pub status: u16,
    pub body: BodyStream,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFailure { Network, Tls }
pub type TransportFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<
    Output = Result<TransportResponse, TransportFailure>> + Send + 'a>>;
pub trait Transport: Send + Sync {
    fn send<'a>(&'a self, request: ProviderRequest) -> TransportFuture<'a>;
}
pub struct ReqwestTransport { /* private */ }
impl ReqwestTransport {
    pub fn new() -> Result<Self, SafeError>;
}
pub async fn evaluate(
    snapshot: EvaluationSnapshot,
    transport: std::sync::Arc<dyn Transport>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<EvaluationResult, SafeError>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assessment { Assessed, Unassessable }
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Improvement {
    pub reference_excerpt: String,
    pub observation: String,
    pub practice: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub assessment: Assessment,
    pub heard_text: Option<String>,
    pub summary: String,
    pub strengths: Vec<String>,
    pub improvements: Vec<Improvement>,
    pub unassessable_reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationResult {
    pub feedback: Feedback,
    pub requested_model: String,
    pub requested_effort: String,
    pub actual_model: Option<String>,
    pub actual_effort: Option<String>,
    pub usage: Option<Usage>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}
pub fn validate_feedback(json: &[u8], reference: &ReferenceText)
    -> Result<Feedback, SafeError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendDisposition { NotSent, MayHaveBeenSent }
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidReference, AudioIo, AudioTooLarge, AudioEmpty, AudioCorrupt,
    AudioUnsupported, AudioTooLong, InvalidHost, InvalidKey,
    CredentialRead, CredentialWrite, Busy, SessionNotFound, InvalidState,
    GuiLaunch, GuiDisconnected, Authentication, RateLimited, Provider,
    Network, Timeout, ResponseTooLarge, ResponseInvalid, ProtocolInvalid,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeError { /* only ErrorCode and SendDisposition */ }
impl SafeError {
    pub fn new(code: ErrorCode, sent: SendDisposition) -> Self;
    pub fn code(&self) -> ErrorCode;
    pub fn disposition(&self) -> SendDisposition;
    pub fn message(&self) -> &'static str;
}
// SafeError Serialize/Deserializeは{code,message,send_disposition}。
// messageは固定のcode対応文を生成し、Deserialize時は一致を検証する。
// Displayも固定文のみ、source()はraw下位errorを返さない。

pub struct SessionId { /* private UUID v4, Clone/Eq/Hash/Serialize */ }
impl SessionId {
    pub fn parse(value: &str) -> Result<Self, SafeError>;
    pub fn as_str(&self) -> &str;
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionState {
    AwaitingUser,
    Running,
    Completed { result: EvaluationResult },
    Failed { error: SafeError },
    Cancelled { send_disposition: SendDisposition },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionView {
    pub session_id: SessionId,
    #[serde(flatten)] pub state: SessionState,
}
pub struct SessionManager { /* private */ }
impl SessionManager {
    pub fn new() -> Self;
    pub fn start(&mut self) -> Result<SessionView, SafeError>;
    pub fn get(&self, id: &SessionId) -> Result<SessionView, SafeError>;
    pub fn mark_running(&mut self, id: &SessionId) -> Result<SessionView, SafeError>;
    pub fn complete(&mut self, id: &SessionId, result: EvaluationResult)
        -> Result<SessionView, SafeError>;
    pub fn fail(&mut self, id: &SessionId, error: SafeError)
        -> Result<SessionView, SafeError>;
    pub fn cancel(&mut self, id: &SessionId) -> Result<SessionView, SafeError>;
}
```

`SessionId`のDeserializeはparseを経る。取消し/complete/failが終端状態に来た場合は同一の終端viewを返し、変更しない。mark_runningはAwaitingUserのみ許可し、Running/終端はinvalid_stateである。startはactiveがあればbusy、なくなるたび新UUIDを生成する。providerのCancelはSafeError(Cancelled)として返り、controllerがcancelへ写す。`SessionManager`自体は通信しない。公開evaluateは信頼されたlibrary利用者用であり、CLI/MCPは直接呼出せずGUIの明示送信controllerだけが呼ぶ。

GUIテストにはWindows限定 `pub mod gui` を提供する。次の描画境界は資格情報adapter/transportを構築せず、純粋な画面入力からactionを返す。

```rust
pub enum GuiPhase { AwaitingUser, Running, Completed(EvaluationResult), Failed(SafeError), Cancelled }
pub struct GuiModel {
    pub reference_text: String,
    pub file_label: String,
    pub audio_info: Option<AudioInfo>,
    pub host_label: String,
    pub credential_present: bool,
    pub mcp_origin: bool,
    pub phase: GuiPhase,
    pub notice: Option<String>,
}
pub enum GuiAction { SelectWav, OpenSettings, Send, Cancel, NewSession }
pub fn render(ui: &mut eframe::egui::Ui, model: &mut GuiModel) -> Option<GuiAction>;

pub struct SettingsView {
    pub host_label: String,
    pub credential_present: bool,
}
pub struct GuiController { /* private draft, validated input, worker, owner */ }
impl GuiController {
    pub fn new(reference: String,
               credentials: std::sync::Arc<dyn CredentialStore>,
               transport: std::sync::Arc<dyn Transport>) -> Result<Self, SafeError>;
    pub fn model(&self) -> &GuiModel;
    pub fn render(&mut self, ui: &mut eframe::egui::Ui) -> Option<GuiAction>;
    pub fn set_reference(&mut self, text: String) -> Result<(), SafeError>;
    pub fn select_audio(&mut self, audio: AudioInput, file_label: String)
        -> Result<(), SafeError>;
    pub fn begin_settings(&mut self) -> Result<SettingsView, SafeError>;
    pub fn set_settings_draft(&mut self, host: String, key: Option<String>)
        -> Result<(), SafeError>;
    pub fn save_settings(&mut self) -> Result<(), SafeError>;
    pub fn cancel_settings(&mut self);
    pub fn send(&mut self, runtime: &tokio::runtime::Handle)
        -> Result<SessionView, SafeError>;
    pub fn poll(&mut self) -> Result<SessionView, SafeError>;
    pub async fn wait_for_terminal(&mut self) -> Result<SessionView, SafeError>;
    pub fn cancel(&mut self) -> Result<SessionView, SafeError>;
    pub fn new_session(&mut self) -> Result<SessionView, SafeError>;
}
```

これらは `qwen_audio::gui` 内へ置く。renderは値更新と描画/action生成のみであり、ファイル/credential/networkへアクセスしない。Send actionは承認済みtokenではない。native GUIもこのGuiControllerを使い、Send action→send→current snapshot再検証→owner claim→evaluateとなる。set_reference/select_audio/settingsはRunning/終端でinvalid_state、sendはAwaitingUserのみで、一回claimしてworkerをspawnする。poll/waitは同じ内部completion処理を使う。cancelは先に終端確定してtoken cancel/JoinHandle abortする。new_sessionは終端のみで新UUIDとAwaitingUserを作る。設定draftのNone keyは旧key保持、cancel_settingsはdraft破棄のみである。fake adapters経由のcontroller試験で保存/取消し/送信0→1/二重Send/入力差替え/遅延cancelを検証する。単独constructorはLocal terminal authority、MCP用private factoryはSupervisor authorityとし、同じsnapshot/save/spawn helperを使いながら親Committed以外から終端を表示しない。

## 入力・確認の同一性

native GUIはGuiController::renderを使い、同じcontroller内のdraft referenceを編集する。純粋render関数はfixture用である。controller新規画面のreference draftは空でも保持でき、set_referenceは編集文字列を保つ。ReferenceText検証はsend/start境界で行う。credential load失敗はcontrollerを破棄せずmodel.noticeへ固定文を置き、設定修復の導線を維持する。SessionManager.completeはRunningのみ許可、failはAwaitingUser/Running、終端に対するcomplete/fail/cancelは同じ既確定viewを返す。

WAV選択はmetadataによる早期拒否後、`take(MAX_AUDIO_BYTES + 1)`相当のbounded読込みを一度だけ行う。bytesをimmutableに保持し、confirm/sendで原ファイルを開き直さない。ファイル名はGUIのbasename表示だけに保持し、library snapshot/MCP viewにフルパスを含めない。最新内容には再選択が必要という仕様文を表示する。

RIFF lengthと実ファイル長を一致させ、checked arithmeticでchunk header/長さ/odd paddingを走査する。fmt/dataを各一つ必要とし、未知chunkは境界内なら読み飛ばす。PCM tag=1、16bit、mono/stereo、8～48kHz、block_align=channels*2、byte_rate=sample_rate*block_align、data長がblock_alignの倍数、frames>0、frames≤sample_rate*60を満たす。浮動小数のdurationで境界合否を決めない。RF64/圧縮/extensible/複数data/切断は拒否する。6MiBはヘッダ/付加chunk込みである。60秒48kHz monoはPCM 5,760,000 bytes、stereoは11,520,000 bytesのため後者はサイズ拒否となる。

ReferenceTextは元の改行/空白を維持し、全体が空白かとUnicode scalar数だけを検証する。Snapshot生成時に確定音声/参照/Connectionを再検証し、Sendがsnapshotの所有権を消費する。設定/参照/音声の変更はAwaitingUserのみであり、変更ごとに確認表示を更新する。送信時点のConnection cloneだけがworkerへ移動し、途中の設定読直しはしない。

## 設定・秘密・保存順序

TokyoHostは原文字列のscheme/authority/pathを厳密に検証する。ASCII小文字のDNS label（1～63文字、英数字、内部hyphenのみ）一つ+固定suffix、httpsである。親採用の入力表現はorigin（path空または/）と `/compatible-mode/v1`（末尾単一/可）であり、canonical baseへ正規化する。userinfo、明示port（443も含む）、percent encoding、query/fragment、他path、余分なslash/label、類似suffixを拒否する。Workspaceは補完しない。正規化は送信先を増やさない。

専用target `WordWeave5.QwenAudio.Connection.v1` のCRED_TYPE_GENERIC、CRED_PERSIST_LOCAL_MACHINEを使う。レコードは `{version:1,host,key,effort:"medium"}` の単一UTF-8 JSON、2560 bytes以下である。keyは空/改行/controlを拒否し、HTTP header化も送信前に検証する。平文serdeはadapter内のzeroizing bufferに限定し、Connection/ApiKeyにSerializeを実装しない。CredReadWは当該target一つ、CredFreeとbuffer消去を必ず行う。

編集draftは既存Connectionと別所有であり、キー欄は既存値を入れず『保存済み』を表示する。未入力は旧キーを保持、新入力は置換とする。保存順序はdraft validate→単一record serialize/上限検証→CredWriteW一回→成功時のみin-memory Connection置換/成功表示である。失敗/取消しは旧Connectionのままであり、hostだけの部分更新はない。設定画面を閉じるとdraftをdropする。履歴/録音/本文のディスク保存や平文fallbackはない。

SafeErrorは固定code/日本語文/dispositionのみ。reqwest/Win32/serdeのraw error、URL、body、header、OS pathをformatしない。ログはモード・code・状態・byte数程度とし、key/音声/英文/評価本文を含めない。actual_model/actual_effortは非空/各128scalar以下、control無しの実応答値のみで、key sentinel混入を検出した場合はresponse_invalidにする。秘密を含むraw応答をGUI/MCPへ返さない。zeroizeは当アプリbufferの管理であり、OS/HTTP stackの全複製消去の保証ではない。

## HTTP・SSE・取消しの境界

Tokio async + reqwest 0.12のasync/native-tls/stream、tokio-util CancellationTokenを使う。blocking clientをthreadに入れる案はfuture dropで転送を止められないため採用しない。ReqwestTransport::newはredirect none、HTTPS only、proxy自動発見なし、connect timeout 10秒、retry無しである。任意endpointコンストラクタ/環境変数/CLI/MCP設定は公開しない。mockはTransport実装を注入し、release factoryは常にReqwestTransportを作る。

POSTは検証済みbase+`/chat/completions`一回だけである。[sources.md](sources.md)の公式証拠に合わせ、Bearer key、model、reasoning_effort medium、modalities [text]、stream true、stream_options include_usage trueを使う。messagesに固定JSON出力指示と、userの参照英文text+`input_audio:{data:"data:;base64,...",format:"wav"}`を置く。未知のresponse_format/enable_thinkingは送らない。reference/応答自然文をシステム命令やshellとして扱わない。

evaluateの最初にcancelを確認し、NotSentで帰す。要求構築/検証後、send futureをpollする直前からMayHaveBeenSentに固定する。180秒のdeadlineはこの時点から全header/body/SSEに適用する。selectはcancel/deadlineを優先し、接続futureとBodyStreamをdropする。worker JoinHandleもabortし、GUI threadを待たせない。既に送った要求についてサーバー処理や課金の中止は保証しない。結果採用はownerの終端commit一回であり、取消し後の遅延応答はdiscardする。

HTTP 401/403→authentication、429→rate_limited、3xx/他4xx/5xx→provider、TLS/切断→networkである。bodyをerror文に使わない。200のみSSEを検証する。累積wire bodyは1MiB、組立delta.contentはUTF-8 64KiB、1 event/line bufferは1MiB以下に制限し、append前に上限確認する。UTF-8/CRLF/event境界がchunkを跨ぐケースを処理する。data JSONのchoices[0].delta.contentのみ連結し、reasoning_content/audioは評価文へ混ぜない。usage-only空choicesは許可する。finish_reason=stopとdata:[DONE]双方が必要であり、length/error/premature EOF/複数choice/不正JSONは失敗する。

JSON Feedbackはcode fence除去や修復をしない。deny_unknown_fields、全必須field（nullを含む）存在、仕様のscalar数/件数/非空白/reference_excerpt部分文字列、assessmentとheard_text/reason/配列の整合を検証する。SerdeのOptionが欠落をnull扱いする点は別のkey存在検査で防ぐ。unassessableは正常resultであり、heard_textはnull、推測補完しない。actual値未提供はNone、要求値をコピーして実値扱いしない。usage fragmentのprompt/completion/total_tokensは提供された非負整数値だけをUsageへ保持し、欠落はNone、全欠落はusage=Noneとする。合算で値を埋めず、金額/課金確定へ変換しない。GUI/MCPはこれを要求の推論effortとは別に返す。

## 状態owner・MCP・子プロセス

親採用の回答上限としてPOST payloadにmax_tokens=4096を明示する。これは旧Chat APIの回答token上限であり、思考tokenを含む総量/課金上限ではない。Omni対応未確認のmax_completion_tokensを足さない。length終端なら正規成功にせず、自動retryしない。GUIは従量課金を明示する。秘密文字列の反射チェックはmetadataだけでなくFeedbackの全自然文fieldにも適用し、API key値を含む結果は安全なresponse_invalidとする。

GUI単独はcontrollerがSessionManagerを持つ。MCPはsupervisor actor一つがSessionManagerとchildを持ち、tool call/child event/timeoutを直列処理する。各イベントにSessionIdを検証し、終端first commit winsを守る。

| 現状態・イベント | 次状態・副作用 |
| --- | --- |
| start成功 | AwaitingUser。GUI起動のみ、POST 0 |
| AwaitingUser・無効入力/編集/設定保存 | 維持、POST 0 |
| AwaitingUser・GUI SendIntent、owner claim | Running、SendGranted後にPOST最大1 |
| AwaitingUser・cancel/GUI close | Cancelled(NotSent)、POST 0 |
| Running・正規検証済みresult | Completed、一度だけcacheへ |
| Running・HTTP/形式/timeout/crash | Failed(MayHaveBeenSent) |
| Running・cancel/GUI close | Cancelled(MayHaveBeenSent)、token cancel/worker abort |
| 任意終端・late result/cancel/fail | 同じ終端を返す。POST/再採用なし |

通常起動はGUI、`--mcp`はSTDIO、内部`--gui-child`はpipe専用GUIである。内部モード自体に自動送信能力はない。startはreferenceを検証してactive枠を予約し、current_exeを新しい子としてspawnする。argvはmodeのみ、stdin/stdout pipeでBoot{session_id,reference_text}を送る。GUI Ready handshakeを5秒以内に受けてawaiting_userを返す。失敗時は子をkill/reapして予約を除きgui_launchを返す。利用者操作/API完了は待たない。

IPCはserdeのdeny_unknown_fieldsを使う改行UTF-8 JSONで、双方512KiB/line、チャネルcapacity 8、書込みtimeout 2秒とする。親→子はBoot/SendGranted/Committed/Cancel/Close、子→親はReady/SendIntent{session_id,reference_text}/Completed{session_id,result}/Failed{session_id,error}/Closedである。key/host/audio/pathは送らない。SendIntentのreferenceはGUIで確認済みの値であり、親はresponse再検証のため終端までだけ保持する。親がRunningを確定してSendGrantedを返すまで子はPOSTしない。キャンセルがclaimに先行するとgrantしない。IPCからresultを受ける親もFeedback/metadata/schemaを再検証する。

子Completed/Failedは終端候補proposalである。子は『結果を確認中』のまま親Committed{view}を待ち、親のfirst commit winsで確定したviewだけを表示する。親cancelが先行すれば、成功proposalが到着してもCommittedはCancelledとなる。Committedがtimeout/EOF/不正で届かない場合は成功表示せず通信中断として閉じる。これによりGUIとMCPに異なる終端状態が表示されない。Cancelを送る場合も親は確定viewのCommittedを送り、子側で独断のCompletedへ戻さない。

MCP protocolは2025-06-18。initialize、notifications/initialized、ping、tools/list、tools/callを実装する。未知versionは対応versionをinitialize応答へ返し、利用者側交渉を妨げない。JSON-RPCは改行一object、UTF-8入力は最大128KiB/line、出力は最大512KiB/lineである。出力は上限64KiBの検証済み結果をstructuredContentとtext JSONへ二重格納するため、入力とは限度を分ける。過大入力行はdrainせずprotocol error後に終了/子回収する。stdoutはprotocol専用、単一writer、bounded channelを使う。unknown method -32601、parse -32700、invalid request -32600、invalid params -32602。tool内busy/not_found等はisError trueのcode/message、正常viewはstructuredContentと同内容のtext JSONを返す。tool argumentsは仕様通り厳格、標準のprotocol metadata（params._meta）は別に検証・許容する。progressTokenを受けても進行通知は送らず、評価状態はgetで取得する。getは副作用無しである。

終端cacheは確定順最新16件、読取りで順序変更/消費しない。child終端後もresult GUIを保持するが送信/入力編集はdisabled、次startで旧終端子へClose→2秒reap→kill/reap→新子を開く。常時最大1 childである。GUIには次のCodex評価で当画面を閉じる旨を表示する。単独GUIの新しい練習は新sessionを作り、新しい明示Sendが必要である。

親stdin EOF/stdout write failure/終了はactiveをcancelし、子へCancel/Closeを送って2秒以内にreap、未終了はkillして必ずwaitする。子は親pipe EOFをcancel/closeとして扱い、OSのkill-on-job-close Job Objectにも所属させる（親の強制crashでも子を残さない）。Job Object割当失敗はgui_launchとして送信前に失敗する。子crash/不正IPCはawaiting時gui_disconnected(NotSent)、running時gui_disconnected(MayHaveBeenSent)。pipe EOFだけでなくprocess.waitも監視する。親はRunning claim時から180秒の期限を持ち、timeout時はFailed(MayHaveBeenSent)を確定してCancel/Committedを送る。AwaitingUserには期限を設けず、既確定の終端はtimeoutでも変更しない。再起動は空cache、新sessionのみであり、自動復元/再送をしない。

## 日本語GUI・幅・倍率

eframe 0.31.1、rfd 0.15.3を固定する。画面は一列の接続状態カード、WAV/参照カード、送信確認、進行/結果カードとする。中央bodyは縦ScrollArea、英文/長いファイル名/結果はwrap、長い単語/hostは横scrollで到達可能にする。カード内は明示vertical layout、幅はavailable_width以下、横ボタン列は狭い場合縦にする。固定高footerを増やさず、cancel/状態はviewport内に保つ。エラーを色/hoverだけに閉じ込めない。

日本語フォントを作成時にWindows FontsのMeiryo→Yu Gothic→MS Gothicから一度読む。欠落は表示警告、OS DPIとegui zoomを区別する。text buttonはnative Buttonを維持し、描画後Galley.mesh_bounds中心をresponse.rect中心へ両軸補正する。非finite/empty meshはgalley.rectの中心へfallbackする。本文は行間を変更しない。選択/保存/取消し/送信はfocus、Enter/Space、disabled、hover、accessible labelを維持し、透明ラベルの二重描画をしない。

非課金fixtureはdebug_assertions限定 `--ui-fixture <input|running|result|error>` とし、合成GuiModel→renderだけを使う。CredentialStore/ReqwestTransportを一切構築しない。releaseは当引数を拒否する。geometry試験は日本語複数ラベル、空/長文、disabled、wrapped、800/360論理幅、倍率1.0/1.5/2.0を扱い、ボタンmesh中心と右端を検証する。native画面はdesktop/narrow+拡大、Windows DPI、IME/貼付け/キーボード/スクリーンリーダーを別証拠とする。

## ACから実装・独立試験へのhandoff

| AC | モジュール/型・state/所有 | 独立した受入証拠 |
| --- | --- | --- |
| 001/017 | package/lib/main、coreはGUI/Win32非依存 | 独立manifest locked build、GUIなし公開API tests、root書込み無し |
| 002/003/014 | Connection/CredentialStore、draft→単一保存→memory更新 | fake store失敗/取消し/旧record維持、secret sentinel監査、Win32手順 |
| 004 | TokyoHost/ReqwestTransport | host表形式拒否、3xx一回で停止、release任意endpoint無し |
| 005/006/008 | AudioInput/ReferenceText/EvaluationSnapshot | 合成RIFF/境界/改変原ファイル/重複Send、0/1 transport回数 |
| 007/016 | gui render/controller/controls | 送信対象表示、MCPorigin、鍵マスク、geometry/native操作証拠 |
| 009/010 | Feedback/EvaluationResult/provider | schema欠落/null/unknown/長さ/不整合、heard_text/評価不能/実値None |
| 011/012/013 | SessionManager/evaluate/actor | cancel先行/完了先行、chunk遅延、paused Tokio時間、timeout/HTTP/late結果 |
| 015/022 | DTO/IPC/ログ、memoryのみ | sentinel/key/audio/fullpath/rawbody非流出、通常ディスク生成無し |
| 018/019/020/021 | mcp/ipc/supervisor/cache | STDIO往復、busy/未知ID/16→17/反復get、GUI失敗/EOF/crash/回収 |
| 023 | supervisor/Job Object/terminal authority | 終端画面再送不可、次start前旧子回収、親終了時回収、Committed cancel優先 |

test authorは `tools/qwen-audio/tests/` と合成fixtureを所有する。Transport fakeはstream chunk/遅延/HTTP statusを制御し、request.endpoint/bodyから送信1回/内容一致を観測する（key accessor不要）。CredentialStore fakeはsingle saveと失敗を制御する。Tokio test paused timeを用い、180秒実待機をしない。process fixtureはtracked test-support binaryを以下の公開supervisor seamへ注入する。production childはcurrent_exe固定、fixture用CLI/envはreleaseへ出さない。型はcrate rootでre-exportし、ipc.rs/mcp.rsが実装する。

```rust
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParentMessage {
    Boot { session_id: SessionId, reference_text: String },
    SendGranted { session_id: SessionId },
    Committed { view: SessionView },
    Cancel { session_id: SessionId },
    Close,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChildMessage {
    Ready { session_id: SessionId },
    SendIntent { session_id: SessionId, reference_text: String },
    Completed { session_id: SessionId, result: EvaluationResult },
    Failed { session_id: SessionId, error: SafeError },
    Closed { session_id: SessionId },
}
pub type IoFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<
    Output = Result<T, SafeError>> + Send + 'a>>;
pub trait GuiProcess: Send {
    fn wait<'a>(&'a mut self) -> IoFuture<'a, Option<i32>>;
    fn kill(&mut self) -> Result<(), SafeError>;
}
pub struct LaunchedGui {
    pub input: Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
    pub output: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
    pub process: Box<dyn GuiProcess>,
}
pub trait GuiLauncher: Send + Sync {
    fn launch<'a>(&'a self) -> IoFuture<'a, LaunchedGui>;
}
#[cfg(windows)]
pub struct NativeGuiLauncher { /* current_exe and Job Object owner */ }
#[cfg(windows)]
impl NativeGuiLauncher {
    pub fn new() -> Result<Self, SafeError>;
}
pub async fn serve_mcp<R, W>(
    reader: R, writer: W, launcher: std::sync::Arc<dyn GuiLauncher>,
) -> Result<(), SafeError>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static;
```

LaunchedGuiはread/write/processを別所有にし、read待機中もcancelを書ける。wait futureをdropしてからkillし、必ず再waitする。NativeGuiLauncherはJob Objectのhandleをparent終了まで保持し、childはspawn直後に送信前割当する。fake launcherはAPI/keyへ一切アクセスせず、pipe/終了イベントを制御する。公開serve_mcpは入力末尾EOFで子回収まで完了して帰る。

実装引継ぎはU1公開input/error types→U2 credential/provider/session/GUI→U3 IPC/MCPである。testsの期待値を実装側都合で変更しない。READMEは親が所有する。testsと本体は同じファイルを同時編集しない。package検証は `cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked`、`cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked`、fmt check。本体納品gateはplanのroot test/releaseを維持する。共通target-dirを `target/qwen-audio` とした場合のbinaryは `target/qwen-audio/release/qwen-audio.exe` である。本設計変更自体はMarkdownのリンク/AC/公開型の整合検査のみである。

## 承認・リスク・測定

Job Objectの割当はspawn直後、Boot送信より前である。子はBoot受信前にGUI/credential/network adapterを起動せず、親pipe EOFで終了する。親承認のmax_tokens/usage/URL入力表現は仕様へ追記済みである。

2026-10-02、U0独立レビュー合格後、親は本書の公開APIと仕様P01～P05・追記を採用した。公開APIを本書で固定した。仕様を弱める挙動を実装者が決めない。API/schema/host/課金取消し/秘密/Job Objectの実装適合性は完成差分の独立レビューで確認する。

実API schema/品質/実値metadata、Tokyo実接続、native資格情報/Job Object/GUI操作は未検証である。モックの成功は実音声評価の正しさを証明しない。Windows同一ユーザー内の資格情報領域は他の同一権限processから読める場合があり、MCP access token化で改善する範囲ではない。file6MiB/body1MiB/結果64KiB/cache16件によりメモリを有界にするが、cloud/Codex側の保存を制御しない。

子ID `/root/qwen_design`。開始/終了token counterと実行model/effortは未取得、親子包含も不明である。KPIへ推測値を渡さず、親の記録へ欠測として渡す。通常shellのACL初期化が失敗したため読み取りは承認レビュー付き実行を使用した。本担当は既存dirtyを保持し、本書以外を変更していない。
