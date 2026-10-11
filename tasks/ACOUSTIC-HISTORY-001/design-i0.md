# I0実装契約追補

2026-10-04 / ACOUSTIC-HISTORY-001 / I0。実装契約を固定する追補であり、I0の全gate合格報告ではない。

## 1. 権限と根拠

入力は [implementation.md](implementation.md)、[spec.md](spec.md)、[design.md](design.md)、[tests.md](tests.md) である。親から利用者の実装許可を受領した。元設計の「利用者採否待ち」は設計時点の記録であり、本追補が実装許可の引継ぎを記録する。observableな契約、完全削除保留、検証gateを変更しない。

planner指定は designer `gpt-6-astra` / `high`。実行metadata・使用量開始終了カウンターは未取得。所有は本書だけであり、本体・テスト・他文書は変更していない。API Skillを適用し、型・error・保存の冪等性を固定する。UI構成は承認済み設計を維持し、本追補では変更しない。

実コードでrootの `Cargo.toml` が `qwen-audio` を参照し、独立crateがrootを参照していないことを確認した。rootはCargo workspaceではない。`AudioInput::bytes()` と `playback_wav()`、Storage/app.lock、commit/backupの接合根拠は元設計1節の通りである。

## 2. crate配置と依存方向の確定

```text
wordweave5 ──────────┬─ acoustic-analysis（PCMと測定契約）
                    ├─ practice-history（共有履歴DTO・検証・純比較）
                    └─ qwen-audio（音声入口と明示Qwen送信）
qwen-audio GUI ──────┬─ acoustic-analysis
                    └─ practice-history
practice-history ───── acoustic-analysis（serde Report/profile型のみ利用）
```

- `tools/acoustic-analysis`: 新crate、version `0.1.0`。`serde` と既存版系列の `tokio-util::sync::CancellationToken` を使う。coreはegui/HTTP/root/Qwenに依存しない。`praat` featureの既定はoffとし、process adapterだけを隔離する。core API/serde型はfeature有無で変えない。
- `tools/practice-history`: 新crate、version `0.1.0`。責任はserde DTO、境界検証、純比較である。依存はserde/serde_jsonと同一path版のacoustic-analysis（default-features=false）まで。filesystem、hash実装、時計/ID発行、credentials、UI、runtime起動、Entry/Progress、qwen-audioへの依存を持たない。
- 二hostで型を複製する案は、保存版・欠測・比較条件の不一致を生むため採らない。acoustic-analysisへ履歴/目標/AI助言を詰める案も責任が異なるため採らない。追加crate一つで共有する範囲はDTOと純関数だけであり、共通永続化frameworkではない。
- `src/audio_history/{store,recovery,jobs}.rs` はWordWeave5 adapterである。既存AssetStoreをそのまま履歴専用rootに使う。独立ツールは `tools/qwen-audio/src/history/` のhost adapterを持ち、rootのAssetStore/Progressを取り込まない。
- rootは共有DTOをre-exportできるが正本定義を持たない。qwen-audioのcore-only buildへ履歴UIを強制しない。GUI用practice-history/acoustic-analysis依存のfeature接続とrootからの直接利用は統合担当がmanifestで明示する。

## 3. I4で先に固定する公開解析API

以下は公開名・field・variant・signatureの契約であり実装ではない。`pub` fieldsを持つDTO、private fieldsを持つ `PcmView`、公開traitを区別する。値型は `Clone, Debug, PartialEq, Serialize, Deserialize`、コピー可能な単純enumは `Copy, Eq` も持つ。floatを持つ型にEqを要求しない。enumのJSONは `snake_case`、SampleValueは `#[serde(tag="kind", content="value")]` とする。

```rust
pub struct PcmView<'a> { /* private: borrowed interleaved PCM16 */ }
impl<'a> PcmView<'a> {
    pub fn new(samples: &'a [i16], sample_rate: u32, channels: u16)
        -> Result<Self, AnalysisError>;
    pub fn samples(&self) -> &'a [i16];
    pub fn sample_rate(&self) -> u32;
    pub fn channels(&self) -> u16;
    pub fn frames(&self) -> usize;
}
pub struct AnalysisRequest<'a> {
    pub pcm: PcmView<'a>, pub profile: &'a AnalysisProfile,
    pub limits: &'a AnalysisLimits,
}
pub struct AnalysisLimits {
    pub wall_time: std::time::Duration, pub max_frames: usize,
    pub max_buffer_bytes: usize,
}
impl AnalysisLimits { pub fn v1() -> Self; }
pub fn analyze(request: AnalysisRequest<'_>, pitch: &dyn PitchBackend,
    cancel: &CancellationToken) -> Result<AnalysisReport, AnalysisError>;
pub struct MonoPcm<'a> { pub samples: &'a [f64], pub sample_rate: u32 }
pub trait PitchBackend: Send + Sync {
    fn identity(&self) -> Option<EngineIdentity>;
    fn extract(&self, pcm: MonoPcm<'_>, profile: &PitchProfile,
        deadline: std::time::Instant, cancel: &CancellationToken)
        -> Result<PitchTrack, AnalysisError>;
}
pub struct MissingPitchBackend;
pub struct PitchTrack { pub frames: Vec<TimedValue> }
```

`PcmView` は非空、1/2ch、sample数のch割切り、8–48kHz、60秒以下を検証する。原音の6MiB制限はこのPCM型へ掛けない。新しいrate/chへの変換や切詰めはしない。`AnalysisLimits::v1()` はwall_time=60秒、max_frames=6001（各時系列）、max_buffer_bytes=64MiB（解析coreが所有するbuffer合計）である。child processのmemory capはこの型に含めない。

`AnalysisRequest` と `PcmView/MonoPcm/AnalysisLimits` はruntime型でありserde対象外。`analyze` は開始時に共通deadlineを作り、Rust段階とPitchBackendへ同じdeadlineを渡す。token取消し・deadline超過・buffer/frame上限は全体Errorとし、途中reportを返さない。backendを任意のuser scriptへ差し替えるUIを作らない。

```rust
pub struct AnalysisProfile {
    pub schema_version: u32, pub profile_id: String,
    pub preprocessing: PreprocessProfile, pub energy: EnergyProfile,
    pub pitch: PitchProfile,
}
pub enum PreprocessProfile { Pcm16MeanChannelsV1 }
pub struct EnergyProfile {
    pub window_ms: u32, pub hop_ms: u32, pub low_threshold_dbfs: f64,
    pub min_low_interval_ms: u32,
}
pub struct PitchProfile {
    pub floor_hz: f64, pub top_hz: f64, pub step_seconds: f64,
    pub max_candidates: u16, pub very_accurate: bool,
    pub attenuation_at_top: f64, pub silence_threshold: f64,
    pub voicing_threshold: f64, pub octave_cost: f64,
    pub octave_jump_cost: f64, pub voiced_unvoiced_cost: f64,
}
impl AnalysisProfile {
    pub fn v1() -> Self;
    pub fn validate(&self) -> Result<(), AnalysisError>;
}
pub struct EngineIdentity {
    pub name: String, pub version: String,
    pub binary_sha256: Option<String>, pub script_sha256: Option<String>,
}
pub enum SampleValue { Finite(f64), DigitalSilence, Missing(MissingReason) }
pub enum MissingReason {
    Unvoiced, TooShort, BackendUnavailable, BackendFailed, InvalidOutput,
}
pub struct TimedValue { pub time_seconds: f64, pub value: SampleValue }
pub struct TimeInterval { pub start_seconds: f64, pub end_seconds: f64 }
pub struct PitchMeasure {
    pub frames: Vec<TimedValue>, pub median_hz: SampleValue,
    pub p10_hz: SampleValue, pub p90_hz: SampleValue,
    pub valid_frame_fraction: SampleValue,
}
pub struct Measures {
    pub duration_seconds: f64, pub rms_dbfs: SampleValue,
    pub rms_frames: Vec<TimedValue>, pub low_intervals: Vec<TimeInterval>,
    pub low_total_seconds: f64, pub low_fraction: f64,
    pub pitch: PitchMeasure,
}
pub enum AnalysisStatus { Complete, Partial }
pub enum QualityKind {
    StereoMixed, ClipCandidate, NoVoicedFrames, ShortInput, BackendWarning,
}
pub struct Quality { pub kind: QualityKind, pub fraction: Option<f64> }
pub struct AnalysisReport {
    pub schema_version: u32, pub metrics_version: String,
    pub library_version: String, pub profile: AnalysisProfile,
    pub sample_rate: u32, pub channels: u16, pub input_frames: u64,
    pub pitch_engine: Option<EngineIdentity>, pub status: AnalysisStatus,
    pub measures: Measures, pub quality: Vec<Quality>,
}
impl AnalysisReport { pub fn validate(&self) -> Result<(), AnalysisError>; }
pub enum AnalysisErrorKind {
    InvalidInput, UnsupportedProfile, BackendUnavailable, BackendFailed,
    TimedOut, Cancelled, ResourceLimit, InvalidOutput,
}
pub struct AnalysisError { pub kind: AnalysisErrorKind }
```

v1はschema_version=1、profile_id=`reading-v1`、metrics_version=`acoustic-metrics-v1`、library_versionはcrateの実版である。energyの25/10/-40/200と全pitch初期値は元設計7節に一致させる。初版はv1の正確な組を受理し、profile_idだけ維持した設定差替えはUnsupportedProfileとする。設定編集UIは今回のAPI確定に含めない。後続新版の追加は既存v1を再解釈しない。

`MissingPitchBackend` はidentity=None、extract=BackendUnavailableを返す。PitchBackendだけのUnavailable/Failed/InvalidOutputはF0全体を該当理由のMissingとするPartial reportになり、Rust測定を残す。backendからのInvalidInput/UnsupportedProfile/TimedOut/Cancelled/ResourceLimitは全体Errorである。外部出力の範囲/非有限/時刻不正はInvalidOutput、過大出力・frame超過はResourceLimitと区別する。

PitchBackendはframe値だけ返し、中央値/P10/P90/有効割合はcoreが算出する。backend無しではframe列は空、4つの集計値はMissing(BackendUnavailable)。frameがあるが全て無声なら集計HzはMissing(Unvoiced)、割合はFinite(0)、status=Partial。自然な無声frameを含んでも有効F0集計が得られればCompleteである。短い入力はF0だけTooShortでPartial、全ゼロRMSはDigitalSilenceでありエラーではない。

固定floor50Hzの最短入力は`3 / floor_hz = 0.06秒`である（probe.mdの59ms拒否/60ms成功による）。60ms未満はbackendを呼ばずTooShortとし、RMS・全長・低エネルギー区間を残す。floorや入力長を黙って変更しない。

Report検証はschema/profile、有限値、各fieldのunit/range、時刻範囲と増加、hop由来のRMS時刻、区間の整列/非重複、合計/割合整合、F0にDigitalSilenceがないことまで行う。2026-10-04の統合点検で、sample丸め時刻を単独検証するために不足していた入力rate/ch/frame数をReportへ追加した。8–48kHz・1/2ch・非空60秒以下と、`duration=input_frames/sample_rate`を照合する。低エネルギー/RMS/F0の算式・分位数定義は元設計7節から変更しない。ratioは保存時0–1、UIで100倍する。日時や元原音hashをReportへ混入させず、次節のAnalysisRunが結ぶ。

## 4. 共有履歴DTOとserde境界

公開場所は `practice_history::{ids, model, compare}`。JSONの全documentは `schema_version: 1` を必須とする。unknown schemaはfail closed、必須fieldの欠落はdefault補完しない。Optionだけがnullを許す。未知fieldを黙って落として再保存しないようdocument DTOは `deny_unknown_fields` とする。メモリ上のView/Job tokenはserializeしない。

`TrialId/RunId/OperationId` はcanonical小文字UUID文字列のnewtype、`Sha256` は64桁小文字hexのnewtype。`BlobRef { sha256: Sha256, bytes: u64 }` と `RootRef(BlobRef)` はserde透過の参照型でありpathを持たない。IDの発行はhostが一意図につき一回行う。型は `parse(&str) -> Result<Self, ContractError>` と `as_str()`、参照は `validate()` を公開する。hash算出はhostの検証済み実装を使う。

| 共有document/値型 | 必須fieldsと具体型（`?`はOption） |
| --- | --- |
| `RecordedAt` | `utc_millis:i64, offset_minutes:i16`。表示offsetを保持し整列はUTCを使う |
| `Origin` | `CapturedWav` / `Imported`。元pathは保存しない |
| `AudioSnapshot` | `original:BlobRef, format:AudioFormat, rate:u32, channels:u16, frames:u64, origin:Origin, warnings:Vec<CaptureWarning>`。AudioFormatはWav/Mp3/Aac/Amr/ThreeGp/ThreeGpp |
| `ReferenceSnapshot` | `text:String, sha256:Sha256, source:Option<SourceTag>`。`SourceTag{host:String,id:String,version:String}` はopaqueであり教材型を要求しない |
| `GoalSnapshot` | `text:String, version:Sha256`。未設定はOption::None |
| `CaptureProfile` | `device:Option<String>, distance:Option<String>, gain:Option<String>, environment:Option<String>, confirmed_at:Option<RecordedAt>, version:Sha256`。不明を空の既知文字列にしない |
| `FrozenInput` | `audio:AudioSnapshot, reference:ReferenceSnapshot, goal:Option<GoalSnapshot>, capture:CaptureProfile` |
| `PracticeClaim` | `Reference` / `Practice{performed_at:RecordedAt, self_speaker_confirmed_at:RecordedAt, time_confirmed_at:RecordedAt, duplicate_confirmed:bool}` |
| `MatchEvidence` | `HumanConfirmed{at:RecordedAt,audio:Sha256,reference:Sha256}` / `AiAssessed{run:RunId}` / `AiMismatch{run:RunId}` / `AiUnassessable{run:RunId}`。Vecで全出所を保持 |
| `TrialRevision` | `schema_version, id:TrialId, revision:u64, previous:Option<BlobRef>, created_at:RecordedAt, input:FrozenInput, claim:PracticeClaim, match_evidence:Vec<MatchEvidence>, analysis:BTreeMap<RunId,BlobRef>, advice:BTreeMap<RunId,BlobRef>, reflections:Vec<BlobRef>, trashed_at:Option<RecordedAt>` |
| `AnalysisRun` | `schema_version, id:RunId, trial:TrialId, audio:Sha256, pcm:Sha256, decoder:DecoderIdentity, profile:AnalysisProfile, state:AnalysisRunState, started_at:RecordedAt, ended_at:Option<RecordedAt>, result:Option<BlobRef>` |
| `AdviceRun` | `schema_version, id:RunId, trial:TrialId, audio:Sha256, reference:Sha256, confirmed_at:RecordedAt, confirmation:Sha256, destination:String, versions:AdviceVersions, state:AdviceRunState, send:SendCertainty, started_at:RecordedAt, ended_at:Option<RecordedAt>, result:Option<BlobRef>` |
| `Reflection` | `schema_version, trial:TrialId, at:RecordedAt, text:String`。改訂は追加であり過去注記を上書きしない |
| `HistoryRoot` | `schema_version, revision:u64, next_save_sequence:u64, trials:BTreeMap<TrialId,TrialHead>, receipts:BTreeMap<OperationId,OperationReceipt>` |
| `TrialHead` | `document:BlobRef, admitted_sequence:u64`。初回保存/取込時のhost内順序。更新時不変 |
| `OperationReceipt` | `request:Sha256, committed_revision:u64, affected:Vec<TrialId>`。root自身のhashは含めず循環を避ける |

`CaptureWarning` はPartialRecording（必要なら固定安全codeを伴う）でありraw機器エラー文字列を保存しない。`DecoderIdentity {name:String,version:String,binary_sha256:Option<Sha256>}`。PCM hashはrate/chをLE u32/u16、frame数をLE u64、その後interleaved PCM16LEを連結したbytesに対しSHA-256を取る。原音hashとは別型fieldであり、両者が一致する前提を持たない。

AnalysisRunStateはRunning/Complete/Partial/Failed/Cancelled/TimedOut/Interrupted、AdviceRunStateはSending/Assessed/Unassessable/ReferenceMismatch/Failed/Interrupted、SendCertaintyはNotSent/MayHaveBeenSent/ResponseReceived。失敗stateには固定codeを持つ別field `failure:Option<FailureCode>` を両Runへ置く。Complete/Partial/評価3状態はresult必須、Running/Sendingはended_at/resultなし、terminalはended_at必須をvalidateする。起動後に見つけたRunning/SendingはView上Interruptedとし、自動実行しない。永続化する場合は新revisionだけを発行する。

`FailureCode {domain:FailureDomain, code:String}`、FailureDomainはStorage/Analysis/Adviceである。codeは1–64文字の小文字ASCII英数字とunderscoreだけの固定識別子とし、各host adapterの明示allowlistから生成する。messageや外部応答をcodeへ変換して保存しない。未知codeは「詳細未対応」と表示し、成功へ読み替えない。

`AdviceResult` はprovider非依存のserde型とし、`assessment:AdviceAssessment, heard_text:Option<String>, summary:String, strengths:Vec<String>, improvements:Vec<AdviceImprovement>, unassessable_reason:Option<String>, requested_model:String, requested_effort:String, actual_model:Option<String>, actual_effort:Option<String>, usage:Option<AdviceUsage>` を持つ。AdviceAssessmentはAssessed/ReferenceMismatch/Unassessable、AdviceImprovementはreference_excerpt/observation/practiceの3 String、AdviceUsageはprompt_tokens/completion_tokens/total_tokensの3 Option<u64>である。

`AdviceVersions {provider:String, prompt:String, parser:String}` と結果はhostが既存 `EvaluationResult` から明示変換する。practice-historyからqwen-audioへ参照しない。保存readerは助言構造を検証し、qwen adapter側は既存validate_feedback相当の制約を再適用する。raw応答、ApiKey、CredentialStore、PreparedId、transport bodyはDTOに持たせない。

各documentに `validate() -> Result<(), ContractError>` を置く。`ContractError{code:ContractErrorCode, field:&'static str}`、codeはUnknownVersion/InvalidId/InvalidHash/InvalidValue/LimitExceeded/ReferenceMismatch/Conflictであり、fieldに本文/path/秘密を含めない。外部bytesはhostがサイズ制限→serde decode→validate→参照内容照合の順で読む。serde parse成功だけではSavedにしない。

### I1a境界の確定（2026-10-04）

- `RecordedAt`はoffsetを±1439分、UTC millisを`-62135596800000..=253402300799999`（西暦0001年から9999年末）に制限する。日時の表示不能やoffset加算時のoverflowを成功値へ変えない。
- 一般`BlobRef`は非空12MiB以下、`RootRef`は非空8MiB以下である。音声原音の参照には別途6MiBを適用する。
- Trialのrevisionとadmitted_sequenceは1開始。空HistoryRootだけrevision=0を許し、その場合trials/receiptsは空である。next_save_sequenceの初期値は1、常に登録済み最大sequenceより大きい。receiptのcommitted_revisionは1以上、所属rootのrevision以下である。
- Running/Sendingはresult/ended_at/failureがすべてNone。正常な終端（Partialを含む）はresult/ended_atがSome、failureがNone。Failed/Cancelled/TimedOut/InterruptedはresultがNone、ended_at/failureがSomeである。Praatのみの不足理由はPartial report内へ保存する。
- AdviceResultの共有検証は既存Qwenの文字数・評価区分を維持する。summary非空2000文字以下、strengths/improvements各8件以下、各本文field非空1000文字以下、heard_text非空10000文字以下。Assessedはheard_textあり・reasonなし、ReferenceMismatchは同じ条件かつ両配列が空、Unassessableはheard_textなし・非空2000文字以下のreasonあり・両配列が空である。改善引用と固定参照の部分文字列照合はhostの閉包検証で追加する。
- requested_model/effortは非空128文字以下で制御文字なし。actual_model/effortはNoneを許し、Someなら同じ文字列制約とする。shared DTOで特定provider/model名へ固定せず、Qwen adapterは既存の指定値検証も維持する。

## 5. host保存APIとtransaction接合

共有crateは `HistoryStore` traitやfilesystem callbackを公開しない。host adapter内の公開窓口を以下へ揃え、二host適合テストは同じ操作シナリオを各adapterへ適用する。独立ツールへroot依存を作ってまで同じ実装を共有しない。

| host API（signatureの型名） | 契約 |
| --- | --- |
| `read_snapshot(root:Option<&RootRef>) -> HistoryView` | 部分破損をTrial別diagnosticとして返し、読める結果を残す。完全な新rootとして返さない |
| `prepare(op:OperationId, request:Sha256, base:Option<&RootRef>, mutation:HistoryMutation) -> Result<PreparedHistory, HistoryError>` | 最新snapshot/receiptをlock下で照合、quota見積、RecoveryIntentを原音publish前に保存。先に正本pointerを変えない |
| `publish(prepared:PreparedHistory) -> Result<PublishedHistory, HistoryError>` | original/result→trial revision→rootの順に不変publishし全閉包を厳格検証。`PublishedHistory{base:Option<RootRef>, next:RootRef, operation:OperationId}` を返す |
| `verify_closure(root:&RootRef) -> Result<(),HistoryError>` | hash/bytes/schema/対象ID、RunとTrialの入力hash、previous連鎖、全結果を検証。visited hashと総bytes/node上限を持ち循環/巨大入力を拒否 |
| `recover(current:Option<&RootRef>) -> RecoveryView` | root receiptsで既commitを識別。未commitは候補表示、勝手な解析/送信/競合解決なし |

HistoryMutationはCreateTrial/AddAnalysisRun/AddAdviceRun/AddReflection/AddMatchEvidence/SetTrash/ImportSnapshotの閉じたenumとし、自由なJSON patchは受けない。RuntimeのComputedUnsaved/ReceivedUnsavedは完成した結果を持ち、同じRunId/OperationIdでAdd...を再試行する。更新対象Runの旧resultを上書きしない（再解析は別RunId）。

rootへ同一OperationId+同一requestのreceiptがあればAlreadyCommitted、異なるrequestならConflict、進行中の同IDならBusy。request hashは開始時に一度serializeしたmutationのexact bytesから作り、原音/result bytesはhash参照にする。receiptを保存する前の再試行は同じintentを使う。複数frameのUI操作ごとにOperationIdを発行し直さない。

WordWeave5ではPublishedHistoryを受けたwriterが最新Progressをcloneし、base一致を再確認して `audio_history:Option<RootRef>` を設定、`Storage::save` 成功/読戻しでcommitを確定する。別の保存が進んでいたら同じmutationを最新rootへ再検証し、古いProgress全体を戻さない。Pending教材commit中は履歴writeを止める。`Progress::validate` はRootRefの構造検証だけでありfilesystemを呼ばない。

`Storage::load` はv1→v2を既存fields保持で読む。`commit::save/recover` とbackup適用はrootが指す全閉包をmetadataの変更前に検証する。通常読取の局所破損を空historyへ変換しない。`Progress` が保持するroot以外のmutable history indexを作らない。

独立ツールは独自rootで排他lockを保持し、host所有 `HistoryHead{schema_version:1, root:Option<RootRef>}` をatomic replaceする。この一ファイルがpointer commit点であり、root側Progressを読み書きしない。immutable publish/receipt/recoveryの順序は同じ受入契約で検証する。既存credentials namespaceは変更しない。

Backup v2のhistory sectionはRootRefとclosure参照列、別欄の未確定recovery候補を持つ。v1復元では現在のRootRef保持、v2ではTrialId衝突を事前検証してunion rootをpublishし、既存のdeck/progress transactionへ接ぐ。同じTrialIdで現在revisionのbytes/hashが異なる場合は全history取込拒否とし、root revision数やlocal admitted_sequenceだけの差をTrial内容差にしない。復元から再実行しない。

取込時、既存TrialHeadのadmitted_sequenceを保持し、新Trialは取込元の順→TrialId順で現在rootのnext_save_sequenceから割り当てる。created_at/PracticeClaimの元日時は変えない。通常一覧の安定キーはperformed_at UTC→host admitted_sequence→TrialId、参考一覧はcreated_at UTC→admitted_sequence→TrialIdとする。headの順序と内容hashを分けることで別host由来の同一試行を重複と誤判定しない。

## 6. 原音と解析PCMの受渡し

原音保存/Qwen送信は必ず `AudioInput::bytes()` を使用する。解析は `playback_wav()` の復号済みPCMから行う。選択した元pathの再読、playback PCMを「原音」として保存、解析用mono化でQwen送信bytesを置換することは禁止する。

二hostの重複RIFF処理を避けるため、I4と独立したI6接続でqwen-audioへ `AudioInput::decoded_pcm16() -> Result<DecodedPcm16, SafeError>` を追加する。`DecodedPcm16 {samples:Vec<i16>, sample_rate:u32, channels:u16}` はprovider/解析非依存の公開音声型である。既存parse/decodeが検証済みのdata chunkを復号し、rate/ch/frame数をAudioInfoと照合する。既存private RIFF走査helperを再利用し、rootのhoundだけに依存しない。

復号WAVを `AudioInput::parse(playback_wav.to_vec())` に再投入しない。合法な60秒48kHz stereo PCMは11,520,000bytesになり、元bytes用6MiB制限に誤拒否されるためである。PCM構築後は同じrate/chのPcmViewへ借用する。解析libraryのmono化はf64で行い、原PCM配列を変更しない。PCM hashとDecoderIdentityはhostがAnalysisRunへ記録する。

## 7. 所有と実装/テストへのhandoff

| 作業単位 | 本体所有を固定する範囲 | 独立テスト所有／依存 |
| --- | --- | --- |
| I0統合 | rootと2新crate/qwen-audioのCargo.toml/Cargo.lock、export接続 | 統合実装役だけが逐次編集。workspace化しない |
| I1a共有契約 | `tools/practice-history/src/{lib,ids,model}.rs` | `tools/practice-history/tests/contract.rs`。core-onlyでserde/制約/秘密排除を検証 |
| I1b本体保存 | `src/audio_history.rs`、`src/audio_history/{store,recovery}.rs`、`src/lib.rs` | `tests/audio_history_store.rs`。I1a後。assets.rsの汎用化は所有外 |
| I2 | `src/{store,commit}.rs`、I1保存窓口の必要接合 | `tests/audio_history_commit.rs`。I1b後、共有source直列 |
| I3 | `src/backup.rs`、backup UI、履歴backup adapterの最大1file | `tests/audio_history_backup.rs`。I2後 |
| I4 | `tools/acoustic-analysis/src/{lib,profile,measure}.rs`（必要ならerror.rsを4本目） | `tools/acoustic-analysis/tests/core.rs`。本書3節確定でPraat gateから独立して開始可 |
| I5 | 同crateの`{praat,process}.rs`と固定script | 同crate`tests/praat.rs`。512MiBは利用者承認済み、製品supervisorの資源制御gateは未完了。I4のoracle変更禁止 |
| I6接続 | 既存`qwen-audio/src/{audio,lib}.rs`のPCM出口を先に限定編集、その後root jobs/controller | 音声PCM互換testとroot job testを別targetにする。5file超過は親が分割 |
| I7 | `tools/practice-history/src/compare.rs`と `src/audio_history/compare.rs` の薄いadapter | 共有crate`tests/compare.rs`。純比較正本を2hostで複製しない |
| I8 | 実装計画のroot UI所有を維持 | UI作者とstate test作者の共有fileは逐次 |
| I9a/I9b | 単独host `history/` の保存/復旧/backup→別単位でGUI | 二host ledger/独立root/秘密namespace test。共有DTOへの独断変更禁止 |

この表は責任分割でありdispatch権限の変更ではない。planner/親がI1a/I1b、I9a/I9bへ既存単位を分けて所有を渡す。指定model/effortはimplementation.mdを維持する。test authorは独立ファイルで期待値を先に作り、本体担当へ失敗assertionと確定case名を渡す。

I4の必須RED seamはMissingPitchBackend/固定fake PitchBackend、既知PCM、cancel事前設定、frame/resource上限、profile相違である。全ゼロ・PCM量子化後RMS・stereo反相・端数窓/閾値境界を独立式で照合する。backendの集計値をoracleに使わない。I1a/I2/I3はserde roundtrip/未知版/入力hash不一致/各publish fault/receipt retry/backup v1-v2を先行する。

受入対応はI4→AH-AC01/03/05/06/09/10/18、共有DTO→02/07/10–13/18、保存接合→04/08/09/14–16である。AH-AC17は元設計UI契約を引き継ぐ。各期待値と副ケースはtests.mdが正本であり、本書はテストの合格条件を緩めない。

## 8. 未完了gateとI4へ渡せる境界

親からのprobe引継ぎではPraat 7.0.02 x64v1、合成200Hz/2秒が成功した。60秒48kHz monoは256MiB child capでメモリ不足、512MiB診断では577ms、peak 469,676,032bytes、出力166,879bytesで成功した。2秒toneのpeakは261,804,032bytes、silenceは262,156,288bytesである。20ms失敗はfloor50Hzに対して短すぎる入力であり、memory失敗と混同しない。これは親/別probe担当の実測引継ぎであり、本設計担当が再実行した値ではない。

512MiBへの製品上限変更は2026-10-04に利用者が承認した。同時1件・処理上限60秒・上限超過時停止は維持する。取消しprobeは200ms要求→220ms回収との引継ぎがあるが、assign前実行raceが残る。製品は制約設定済みJobをPROC_THREAD_ATTRIBUTE_JOB_LISTへ渡し、Job所属と同時にchildをsuspended作成→所属確認→resumeする。失敗時resumeせず回収を必須とし、作成後assignだけで親強制終了時の回収を満たしたと扱わない。2026-10-07の独立レビューP1対応で作成時所属へ訂正した（詳細はdesign-i5.md）。

GPL-3.0-or-later、製品bundle/自動取得未承認を維持する。現段階は明示指定された別exe運用だけである。I0全体/I5のrelease gateは未完了、I4のRust算式・DTO・fake backend試験、I1の保存はこの未解決と独立に進められる。F0失敗時に別engineへ黙って切替えず、上限や期待値を実装担当が緩和しない。

今回の完了境界は本書の契約と文書検査であり、production/testコードは作っていない。アプリbuild・測定精度・二host動作の合格を主張しない。

文書検査は相対リンク欠落0、行末空白0、code fence対応、`git diff --check` 成功である。新規ファイルのため本書も直接検査した。
