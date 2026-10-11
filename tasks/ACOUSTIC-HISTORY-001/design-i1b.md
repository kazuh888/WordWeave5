# I1b 保存adapterの実装境界

2026-10-04 / ACOUSTIC-HISTORY-001。親の統合追補である。[設計](design.md)5/9節、[I0契約](design-i0.md)5節を具体化する。仕様採否や保存上限を変更しない。I2のProgress commitとI3のbackupをI1bへ混在させない。

## 所有と責任

I1bはWordWeave5の履歴専用object/recovery領域と、その参照検証を担当する。`Storage`のapp.lock取得済みインスタンスを借用し、既定の利用者rootを自分で開かない。`Progress`の読書きはI2の責任である。I1bがroot objectをpublishしただけではSavedではない。

既存AssetStoreはhistory配下のFileBlobとして再利用する。既存chat assetsを移動・変更しない。進行中操作のメモリ原音は移動で消費せず、再試行できるようcallerが保持する。

I1b自体を次の小単位へ分ける。指定モデル・レビューgateはimplementation.mdのI1を維持する。

1. I1b-1: 参照読込・閉包検証・部分閲覧。root依存/exportと型のscaffold、`src/audio_history.rs`、`src/audio_history/closure.rs`、`tests/audio_history_closure.rs`。書込APIはまだ公開しない。
2. I1b-2: 不変publish・容量台帳・intent・回復候補。`src/audio_history/store.rs`、`src/audio_history/recovery.rs`、`src/audio_history.rs`、`tests/audio_history_store.rs`。
3. I2: 上記を既存Storage/Progress/教材commitの全writerへ接続する。

readerを先に固定する理由は、writerが「保存できた」と判断するのに自身の組立て結果だけを使わせないためである。テスト作者は保存fixtureをAssetStoreへ直接配置でき、本体writerの出力をoracleにしない。

## I1b-1 公開窓口

`wordweave5::audio_history`に共有DTOをre-exportする。以下の型名・署名をscaffold/testに用いる。opaqueなpathを返すAPIは作らない。

```rust
pub struct HistoryReader<'a> { /* &'a Storage, limits */ }
impl<'a> HistoryReader<'a> {
    pub fn new(storage: &'a Storage) -> Self;
    pub fn with_limits(storage: &'a Storage, limits: HistoryLimits)
        -> Result<Self, HistoryError>;
    pub fn read_snapshot(&self, root: Option<&RootRef>) -> HistoryView;
    pub fn read_page(&self, root: Option<&RootRef>, after: Option<&TrialId>)
        -> HistoryView;
    pub fn read_analysis(&self, trial: &TrialRevision, run: &RunId)
        -> Result<AnalysisView, HistoryError>;
    pub fn read_advice(&self, trial: &TrialRevision, run: &RunId)
        -> Result<AdviceView, HistoryError>;
    pub fn read_reflection(&self, trial: &TrialRevision, reference: &BlobRef)
        -> Result<Reflection, HistoryError>;
    pub fn verify_closure(&self, root: &RootRef)
        -> Result<VerifiedClosure, HistoryError>;
    pub fn read_original(&self, reference: &BlobRef)
        -> Result<Vec<u8>, HistoryError>;
}
pub struct HistoryLimits {
    pub max_nodes: usize,
    pub max_total_bytes: u64,
}
impl HistoryLimits { pub fn v1() -> Self; }
pub struct VerifiedClosure { pub objects: Vec<BlobRef>, pub total_bytes: u64 }
pub struct HistoryView {
    pub root: Option<HistoryRoot>,
    pub trials: Vec<TrialView>,
    pub next_after: Option<TrialId>,
    pub diagnostics: Vec<HistoryDiagnostic>,
}
pub struct TrialView {
    pub document: TrialRevision,
    pub original_available: bool,
    pub diagnostics: Vec<HistoryDiagnostic>,
}
pub struct AnalysisView { pub run: AnalysisRun, pub result: Option<AnalysisReport> }
pub struct AdviceView { pub run: AdviceRun, pub result: Option<AdviceResult> }
pub struct HistoryDiagnostic { pub trial: Option<TrialId>, pub error: HistoryError }
pub struct HistoryError { pub kind: HistoryErrorKind }
pub enum HistoryErrorKind {
    InvalidDocument, UnsupportedVersion, MissingObject, DamagedObject,
    RelationshipMismatch, LimitExceeded, UnsafePath, Io,
    Conflict, Busy, PendingMaterial, QuotaExceeded,
}
```

Value/View/ErrorはClone/Debug/PartialEq、必要な単純enumはCopy/Eqを持つ。HistoryReaderはCloneを要求しない。全errorは固定codeだけであり、OS本文・原音名・参照全文を含めない。readerはファイルを作成・変更・削除しない。

`HistoryLimits::v1()`は最大100,000個の異なるobject、総参照bytes 2GiBとする。node数は1–100,000、総bytesは1以上である。callerは小さい上限、または利用者が明示変更した保持容量に対応する総bytesを指定できる。node数と各object上限は容量設定によって緩めない。原音6MiB、Report2MiB、Trial256KiB、root8MiBの個別上限は独立して守る。その他共有文書は12MiBを越えず、各DTOの件数制約も適用する。

## 閉包検証

参照はFileBlobへ限定し、サイズ上限→hash/実bytes照合→serde decode→DTO.validate→関係照合の順で扱う。文書未知fieldと未知版を捨てて正規化しない。共有DTOとAnalysisReportはstrict readerである必要がある。

- root自身もobjectsとbytesに含める。objectsはhash順、同じhashは1回だけ計上する。同hash/異なるbytesの参照は拒否する。
- rootの各TrialHeadを辿り、IDがmap keyに一致することを検査する。全previous revisionも含める。旧revisionから到達する結果を省略しない。
- previousは同TrialId、revisionがちょうど1小さい、created_at/inputが不変であることを検査する。claimは同値、またはReferenceから有効な本人・練習日時確認を持つPracticeへの一方向遷移だけを許す。PracticeからReferenceへの逆遷移、保存済みPracticeの日時・確認内容の変更を拒否する。初版はrevision=1/previous=Noneである。参考音声の後日確認は同じTrialIdの新revisionであり、旧Reference revisionを保持する。writerは確認時にも同bytesの既存試行を調べ、該当時は重複を明示した人の確認を要求する。反復・循環・深い連鎖は再帰stackではなくboundedな走査で扱う。
- 原音bytes/hashを照合する。参照英文のhashはtextのUTF-8そのもののSHA-256とする。目標versionはgoal.textのUTF-8のSHA-256、capture.versionはdevice/distance/gain/environmentの4fieldをこの固定順でserde JSON objectへserializeしたbytesのSHA-256とする。confirmed_atは確認出所であり音響条件そのものではないためhashに含めない。同じ条件を再確認しただけで比較系列を分断しない。後述のhash helperをwriterとfixture作成へ公開する。
- Runはmap key/id、trial、audio hashが一致する。AdviceRunはreference hashも一致する。AnalysisReportのprofile/rate/ch/input_framesがRun/Trialと一致し、state Complete/PartialとReport.statusが一致する。
- AdviceResultのassessmentとRun stateを照合する。各improvement.reference_excerptは固定reference.textの連続部分文字列であることを確認する。providerは既存Qwen adapterで再検証するが、閉包検証は別providerに勝手に置換しない。
- ReflectionはTrialIdを照合する。MatchEvidenceのAI runは同TrialのAdviceRunを指し、assessed/mismatch/unassessableの区分が実際の保存結果と一致する。HumanConfirmedはaudio/reference hashを照合する。
- Runは履歴の同じIDでintent→terminalへ進められる。旧TrialRevisionにintentが残ることを許す。終端結果を新しい解析で上書きすることはwriterが拒否する。
- engine/decoder identityは保存された出所であり、それだけで現在のインストール状態や精度を保証しない。原PCM hashの再計算には復号が必要であるため、readerは復号・process・送信を起動しない。解析保存時にhostが検証したhashであることと、format/rate/ch整合の範囲を区別する。

hash helperは`hash_reference(&str) -> Result<Sha256, HistoryError>`、`hash_goal(&str)`、`hash_capture(&CaptureProfile)`とする。captureは未確認をJSON nullとして保持する。version自体を再hashへ含める循環を避ける。元AudioSnapshot.originalはbytesから得る別hashである。host固有のSHA実装は既存assets::sha256を使う。

## 部分閲覧

`None`だけが履歴なしの正常状態である。root読込失敗時はroot=Noneでもdiagnosticsが必須であり、空履歴に保存し直してよい状態を表さない。

rootが読めれば各Trialを独立に読み、壊れた枝のdiagnosticを残して他Trialを返す。Trial本文が正常で原音だけが壊れている場合、original_available=falseとする。Run/result/Reflectionは必要な一件を個別APIで読み、原音欠損だけを理由に読める結果を遮断しない。結果自体が壊れていれば個別APIがErrを返し成功結果と表示しない。過去revisionの破損も当該Trialのdiagnosticとし、新しい本文まで黙って空にしない。

全履歴のReportと音声を一括してメモリへ展開しない。read_snapshotはread_page(root,None)の略であり、ID順で最大32個のTrialHeadを検査する。壊れたTrialもcursor前進には含め、next_afterは未処理headがある時だけ最後の処理IDを返す。cursorが存在しなければ診断付き失敗とする。callerは元rootを固定して次pageを取得し、保存でrootが変わったらcursorを初期化する。UIの練習日時順・比較系列はI7/I8が索引情報から構成し、保存ID順を練習日時順と偽らない。

個別読込はtrial.validateに加え、指定RunId/refがTrialに属することを検証してからbounded readを行う。strict closureは全音声/resultを検証するが一件ずつ解放し、訪問済み参照と現在の走査状態だけを保持する。結果bodyをVerifiedClosureへ蓄積しない。

strict verify_closureは一箇所でも不正ならErrである。read_snapshotは部分閲覧専用であり、そのroot/trialsをそのまま書込用の正本と見なしてはならない。上限に達した場合は残りを黙って成功扱いせずLimitExceededを残す。

## 独立受入

I1b-1は合成objectを手作りし、root→原音/初版/旧revision/複数Run/resultの期待hash集合・総bytesと照合する。固定参照/目標/capture hash、map ID、Run入力、結果state、不一致根拠、欠損/改ざん、未知版/未知field、同hash不整合、個別/総容量・node上限を別々に試す。Reference→Practice許可、逆遷移/Practice日時変更/固定input変更拒否、旧Reference版保持を独立ケースにする。正常な別Trialが局所破損に巻き込まれないこと、reader操作で全ファイルのbytesが変わらないことを確認する。

原音再解析・Qwen・Progress移行・復元はここでは未実施である。I1b-2の書込API詳細はこのreader完成後に同じ文書へ追記し、I2のcommitpointを別のmutable indexで代替しない。

## I1b-2 writer / recovery 契約（2026-10-07）

本節は[実装計画](implementation.md)のI1、designer `gpt-6-astra` / `high` の割当による追補である。承認入力は同計画の承認記録、[仕様](spec.md) AH-AC02–04/08/14と[既存設計](design.md)5/6/9節である。readerの公開契約・共有DTOは変更しない。完了境界は書込・回復API、独立fixture、I2への接合条件の確定であり、実装・実行試験・UI・Progress移行・backupは含まない。開始/終了の使用量カウンターと実行model/effort metadataは未取得である。

根拠となる現行APIは `Storage::at` のapp.lock、`AssetStore::{put,read}` のFileBlob不変publish/hash照合、`store::atomic_write` のtemp→sync→rename、`commit::pending` とreaderのstrict closureである。`Storage::load` は教材recoverを実行するので、下記のread-only head窓口として直接呼んではならない。`AssetStore::inventory` は残存tempを除外するため、これだけをquota台帳にしてはならない。rootの祖先を含むreparse検査はreaderと同じ強度をwriterの作成前・読書き前にも要求する。

### 所有と正本の境界

| 所有者 | 所有する状態・責任 | 次の境界へ渡すもの |
| --- | --- | --- |
| caller / I6 | 一意図につき一回発行したOperationId、TrialId/RunId、固定mutation bytes、原音/結果の `Arc<[u8]>`、未保存表示と再試行 | 借用可能なFrozenHistoryRequest。元path・資格情報・送信権限を渡さない |
| `audio_history/store.rs` | 検証済み基底、mutation適用、新objectのexact bytes、容量台帳、不変publish | PublishedHistory。Saved判定/Progress更新をしない |
| `audio_history/recovery.rs` | operation別intentのstrict読取・保存、回復分類、receipt照合、確認済みintentの完了処理 | 回復候補と検証済みrequest。解析/送信を呼ばない |
| I2 / Storage | app.lockの寿命、全Progress writerの直列化、最新headの再読、pending教材guard、pointer commit/読戻し | CommittedHistoryHead実装、commit確認後のSaved |

`HistoryWriter` は既にlock済みの `&mut Storage` を借り、既定rootを開かない。I2は全保存を同じ直列writer経路へ載せ、各呼出し中に教材commitを割り込ませない。borrowだけで、公開 `Storage::dir` や別handleからの違反アクセスまで排除できるとは説明しない。I1b単体の本番head実装は作らず、I2接合までは合成headで試す。history内にhead.json等の第三のmutable正本を作らない。

### 公開APIと入力型

以下はhost内Rust契約でありREST/汎用永続化frameworkではない。DTOはsharedの既存型を使う。新規host DTOのserde境界は版付き・未知field拒否・必須Optionの欠落拒否とする。Value/ViewはClone/Debug/PartialEq、opaque tokenはDebugだけでよい。

```rust
pub trait CommittedHistoryHead {
    // I2: pending recoverを起動せず、disk上の最新Progressをstrictに読む。
    // Err/未知版をNoneへ変換しない。Noneは検証済みの履歴なしだけ。
    fn read_current(&self, storage: &Storage)
        -> Result<Option<RootRef>, HistoryError>;
}
pub struct HistoryWriter<'a> { /* &mut Storage, &dyn CommittedHistoryHead, quota */ }
pub struct HistoryQuota { pub limit_bytes: u64 } // v1() = 2GiB、0は拒否
pub struct FrozenHistoryRequest { /* private mutation_json, hash, Arc payloads */ }
pub struct PreparedHistory { /* private session/op/base/plan/intent binding */ }
pub struct PublishedHistory {
    pub base: Option<RootRef>, pub next: RootRef,
    pub operation: OperationId, pub request: Sha256,
}
pub struct CommittedHistory {
    pub current: RootRef, pub operation: OperationId,
    pub receipt: OperationReceipt,
}
pub enum PrepareOutcome {
    Ready(PreparedHistory), AlreadyCommitted(CommittedHistory),
    DuplicateConfirmationRequired(Vec<TrialId>),
}
pub enum PublishOutcome {
    Published(PublishedHistory), AlreadyCommitted(CommittedHistory),
}
impl FrozenHistoryRequest {
    pub fn new(mutation: HistoryMutation, payloads: Vec<std::sync::Arc<[u8]>>)
        -> Result<Self, HistoryError>;
    pub fn request_hash(&self) -> &Sha256;
    pub fn mutation_json(&self) -> &str;
    pub fn mutation(&self) -> &HistoryMutation;
}
impl<'a> HistoryWriter<'a> {
    pub fn new(storage: &'a mut Storage, head: &'a dyn CommittedHistoryHead,
        quota: HistoryQuota) -> Result<Self, HistoryError>;
    pub fn prepare(&mut self, operation: OperationId, base: Option<&RootRef>,
        request: &FrozenHistoryRequest) -> Result<PrepareOutcome, HistoryError>;
    pub fn publish(&mut self, prepared: &PreparedHistory)
        -> Result<PublishOutcome, HistoryError>;
    pub fn quota(&self) -> Result<QuotaLedger, HistoryError>;
    pub fn recover(&self, after: Option<&OperationId>) -> RecoveryView;
    pub fn read_recovery_request(&self, operation: &OperationId,
        payloads: Vec<std::sync::Arc<[u8]>>)
        -> Result<FrozenHistoryRequest, HistoryError>;
    pub fn resume(&mut self, operation: &OperationId,
        request: &FrozenHistoryRequest) -> Result<PrepareOutcome, HistoryError>;
    pub fn confirm_committed(&self, published: &PublishedHistory)
        -> Result<CommittedHistory, HistoryError>;
    pub fn finish(&mut self, committed: &CommittedHistory)
        -> Result<(), HistoryError>;
}
```

I0の概念署名のrequest hash引数は、ここではopaqueなFrozenHistoryRequestへ結び、callerが別payloadのhashを渡す余地をなくす。request hashは `mutation_json()` のUTF-8 exact bytesのSHA-256である。serializeは初回一度だけ行い、再試行では同じbytesを使う。原音/結果本体はmutationのBlobRefに結び、payload表はhash→Arcとして重複排除する。同hashで異なるlen/bytes、余分な未参照payload、指定refと不一致のpayloadは拒否する。既存検証済みBlobRefだけで足りる場合はpayload不要である。callerのArc/入力をtake/dropしない。

constructorとread系はdirectoryを作らず、準備の全事前検証後に必要な安全なdirectoryだけを作る。read_recovery_requestは再起動後の入口であり、intentをstrict検証して保存されていたmutation_jsonをそのまま復元する。callerはmutation()から固定入力/対象を確認できる。原音だけ残るAssociationReadyは追加payloadなしで復元でき、OriginalMissing/PayloadMissingは不足する同bytesをcallerが渡すまでMissingObjectである。復元したrequestもresumeの最新head検証を省略できない。

`HistoryMutation` の正確なvariant/fieldsは次の表とする。全更新の `target: TrialTarget` は `{id: TrialId, expected: BlobRef}` で、現在のTrialHead.documentとの一致を要求する。新しいTrial本文全体や任意JSON patchをcallerから受けない。

| variant | fields / 適用 |
| --- | --- |
| `CreateTrial` | `id:TrialId, created_at:RecordedAt, input:FrozenInput, claim:PracticeClaim, duplicate:Option<DuplicateConfirmation>`。初版1、previous=None、結果map/根拠/振り返りは空、Activeで生成 |
| `ConfirmPractice` | `target:TrialTarget, claim:PracticeClaim, duplicate:Option<DuplicateConfirmation>`。Reference→Practiceだけ。I0 enumに欠けていたreader既存遷移の入口であり、新しい利用者動作ではない |
| `AddAnalysisRun` | `target:TrialTarget, run:AnalysisRun`。新Running、または同RunIdのRunning→終端。result refの本文はpayload/既存blob |
| `AddAdviceRun` | `target:TrialTarget, run:AdviceRun`。新Sending、または同RunIdのSending→終端。result refの本文はpayload/既存blob |
| `AddReflection` | `target:TrialTarget, reflection:Reflection`。新blob refを末尾追加 |
| `AddMatchEvidence` | `target:TrialTarget, evidence:MatchEvidence`。既存根拠を保持して追加。参照先Runの実結果を照合 |
| `SetTrash` | `target:TrialTarget, trashed_at:Option<RecordedAt>`。Someでごみ箱、Noneで復帰。最新状態と異なる変更だけ |

I0の `ImportSnapshot` はI3へ保留する。I1bの公開enumに未検証の復元入口やunimplemented分岐を先置きしない。今後の追加時にもTrialId衝突の全体拒否契約を維持する。

`DuplicateConfirmation { original:Sha256, matches:Vec<TrialId>, confirmed_at:RecordedAt }` は人の確認をhostが渡す値である。CreateTrial/ConfirmPracticeでは現在rootの全TrialHead（ごみ箱/参考音声を含む、確認対象自身は除く）の原音hashを検査し、同bytesのIDをソートして重複表示用に返す。重複があるときはhashと全ID集合が一致した確認が必要である。未確認ならintentもobjectも作らない。集合が変化した古い確認は再確認対象であり、boolだけで通さない。Practiceの `duplicate_confirmed` はその確認の有無と整合させる。仕様2節とAH-AC12は「明示確認なしで別試行や回数を増やさない」とするため、同bytesを**別TrialIdで新規登録する場合**はReferenceも対象である。重複のないReference保存に追加確認を要求せず、本人/練習日時の確認やPracticeへの昇格も要求しない。既存Trialを選ぶだけならCreateTrialを呼ばない。確認文言/画面はI8の担当である。

### mutation、receipt、Runの状態遷移

1. 各prepare/resume/publish/confirm/finishはhead seamで最新正本を再読する。既存rootがあればstrict closureを通す。部分HistoryViewやcallerのbaseを正本にしない。receiptが同operation・同request・期待affectedならAlreadyCommittedを返す。異なるrequest/affectedはConflictであり、最初の成功結果へ誤って接続しない。
2. receipt照合はbase比較より先である。commit後に別保存が進んでも最新rootがreceiptを保持していれば再試行成功を認識できる。receiptは全rootで維持し、期限削除しない。root revision、Trial revision、sequenceのchecked加算失敗をLimitExceededとし、wrap/resetしない。
3. receiptなしでbaseが最新headと違えば通常prepareはConflictであり無書込で返す。callerは最新headで同requestを再検証する。変更対象が既存Trialならexpected refも一致が必要である。他Trialだけが進んだ場合はrebaseできるが、対象Trialが進んだ場合にinput一致だけで上書きしてはならない。進んだ対象を使う別操作はcallerが結果/意図を再検証して新OperationIdで開始する。同RunIdの結果は保持でき、再解析/再POSTの理由にはならない。
4. 現rootをcloneし、対象Trialを新revisionへ更新する。previousは直前head、created_at/input/admitted_sequenceは不変、旧map/ref/根拠を省略しない。新Trialだけnext_save_sequenceを一回消費する。root revisionは一回増え、receiptのcommitted_revisionをその値、affectedを対象ID一件とする。新rootには旧receipt全部を残す。まだpointer未確定であるため、receiptの存在だけを成功判定にしない。
5. 同IDの未確定intentがあれば、request不一致はConflict、一致は通常prepareでBusyとする。明示resumeだけが同intentを引き継ぐ。publish再試行は同Prepared tokenで可能だが、別writer sessionのtokenは拒否する。中断したことを時刻から推測して別操作へ解放しない。

| Run操作 | 許可する遷移 / 保持項目 |
| --- | --- |
| 新規analysis | Active/Saved Trialへ未使用RunIdのRunningだけ。別Trial/別種Runの既存IDも再利用不可 |
| analysis終端 | Running→Complete/Partial/Failed/Cancelled/TimedOut/Interrupted。id/trial/audio/pcm/decoder/profile/started_atを固定し、ended_atはstarted_at以降。result/failureはshared DTOとreaderの規則を満たす |
| 新規advice | Active/Saved Trialへ未使用RunIdのSending/NotSentだけ。確認hash・日時・destination・versions・入力を固定し、durable intentを送信許可に転用しない |
| advice終端 | Sending→Assessed/Unassessable/ReferenceMismatch/Failed/Interrupted。id/trial/audio/reference/confirmation/confirmed_at/destination/versions/started_atを固定。正常3状態はResponseReceived。Failedはhostの観測に応じNotSent/MayHaveBeenSent/ResponseReceived、InterruptedはMayHaveBeenSentを保持する。状態は送信済みの確証を捏造しない |
| 終端済みRun | 別OperationIdで同Runを再書込/上書きしない。Conflict。同operation再試行はreceiptでのみ冪等成功。再解析/再送信は新RunId |
| Trash交差 | AddRun/結果/確認/注記/根拠はActiveのみ。SetTrash(Some)は最新TrialにRunning/Sendingが残ればBusy。I6が取消・停止を確定しRun終端を保存してから新targetでtrash。遅延結果がtrash後に到着すればConflict、旧revisionを再Activeにしない |

Running/Sendingを保存した操作と、その結果を保存する操作は**異なるOperationId、同じRunId**である。同じOperationIdのpayloadをintentから結果へ変更してはならない。結果保存の再試行だけが同OperationIdと同result bytesを使う。再起動時のInterrupted表示はreader/Viewの意味であり、回復列挙がRunを書き換えることはない。Interruptedを永続化する場合も明示した別mutationを使う。

### exact quota と書込順

`QuotaLedger { object_bytes:u64, recovery_bytes:u64, residual_bytes:u64, retained_bytes:u64, limit_bytes:u64, warning:bool }` と、Preparedの読み取りaccessor `quota_plan() -> &QuotaPlan` を公開する。`QuotaPlan { before:QuotaLedger, added_object_bytes:u64, intent_delta_bytes:i64, projected_bytes:u64 }`。減算/加算/変換はcheckedで行う。warningは整数演算で `retained_bytes >= ceil(limit_bytes * 4 / 5)`、planの予告ではprojectedにも同じ式を使う。乗算overflowを避け、bytesを浮動小数へ丸めない。

- 台帳は `<root>/audio-history/` の保持ファイルを走査する。`assets/<hash>.bin` の不変objectは到達可能性によらずhash一回、回復intentは保存したJSON bytes、残存temp/作業ファイルはファイル長で数える。原音、全旧revision/root、全結果、ごみ箱、未参照公開objectが含まれる。通常の同hash参照の重複を加算しない。残存tempはblob参照ではないため別の残存bytesとして数える。これは保持内容の論理bytesであり、NTFSクラスタ/圧縮/一時hardlinkの物理使用量ではない。
- chat assets、Progress本体/日別backupは履歴quotaから外し、root全体disk量と混同しない。workが他workerにより変化中ならI2/I6が停止点で再走査するかBusyとする。走査中に検証不能/未知file/危険pathがあれば台帳を小さく見積もって成功させない。未知通常ファイルも長さを残存量へ算入した上で診断し、書込は停止する。symlink/reparse/directory偽装はUnsafePath、I/O失敗はIoである。
- writerは最大100,000エントリのbounded走査を行い、超過時はLimitExceeded。AssetStoreのtemp除外inventoryだけに依存せず、stream列挙＋bounded readで台帳を作る。blob名/hash/len/実bytes、intentのschema/operationを検証する。既存の壊れたobjectを正常な同hash入力で置換してはならない。
- readerのnode/個別上限は維持する。容量設定を2GiBより増減したhostでは、旧closure確認の総bytes上限を `max(limit_bytes, retained_bytes)`、新closureをlimit_bytesとする。設定を現在量未満へ下げた場合に、既存データを不正扱いして空へ戻さず、台帳からQuotaExceededとして新規保存を止めるためである。
- mutation→新leaf/run→新trial→receipt込み新rootを**一度serializeしたexact bytes**から計画する。追加object量は計画の異なるhashのうち既存照合済み集合にないものだけ。intentは実際に書くserialized長（新規なら全量、明示resumeによる置換なら旧長との差）を使う。`projected = before.retained + added_object_bytes + intent_delta`。同operationの現在intentを二回数えない。既存台帳が上限超過なら書込を止める。projected==limitは許可、>limitはRecoveryIntent作成前にQuotaExceeded。
- publish再試行の事前確認は最新台帳とまだ存在しない計画objectから再計算する。初回のadded_object_bytesを、部分publish後のretainedへもう一度足さない。新しい無関係ファイル/残存tempによる超過は未公開段階で拒否し、既に公開したobjectを取り消さない。
- この上限は保持量でありatomic_write/AssetStoreの一時stagingで必要な追加disk余裕とは別である。write失敗で残ったtempは次の台帳へ入り、今後も自動GCしない。意図したintent完了処理以外に旧object/残存tempを削除してquotaへ合わせない。I/O成功や電源断耐久性をquota計算だけから保証しない。

保存順は次の通りである。

APIへの割当はI0の署名を維持する。`prepare` は下記1–2を担当し、全事前検証後にintentを保存・読戻し確認してからReadyを返す（BeforeIntent/AfterIntentもここで呼ぶ）。`publish` は3–4を担当し、原音/resultを含むobjectを公開する。したがってprepare全体が無書込なのではなく、事前検証による拒否時が無書込である。Readyのdropは保存済みintentを消さない。

1. pending-material-commit.jsonをmetadata/try_existsで検査する。存在はPendingMaterial、読取不能はIo、reparseはUnsafePath。`commit::pending` のboolだけでアクセス失敗を「なし」にしない。latest head/receipt、target、重複確認、schema、payload、旧closure、quotaを検証する。
2. `RecoveryIntent` を安全なrecovery directoryへatomic_writeし、exact bytesで読戻す。失敗/結果不明ならdiskを再読し、予定intentと一致する場合だけ次へ進む。**intentを確認できる前に原音/resultをpublishしない**。
3. original/result/reflection leaf→Run文書→TrialRevision→HistoryRootの依存順で、`AssetStore::new(storage.dir.join("audio-history"))` のFileBlobとしてput/readする。同refが既にあれば照合のみ。各段階の前にpath/pending、初回publishと再試行時にlatest head/receipt/台帳を再確認し、準備後にbaseが動けばConflictで止める。途中objectは保持する。
4. 新rootのstrict `HistoryReader::verify_closure` を通す。全previousから全旧結果まで含める。成功でPublishedHistoryを返す。ここでUIをSavedにせず、解析/POSTも起動しない。
5. I2はwriter借用を終了し、同じ直列経路内で最新Progressを再読、base一致・pendingなし・新closureを再確認する。最新Progressの他fieldを保持してpointerを保存/再読する。旧Progress全体のsnapshotを戻さない。
6. 新writerのconfirm_committedは再読した最新正本のreceipt/request/affected/closureを確認する。予定nextそのもの、またはそのreceiptを維持する後続rootだけが成功である。旧baseならBusy（未確定）、別rootかつreceiptなしはConflict、I/O不明はIoである。I2は成功後だけSavedを表示する。finishは再度同条件とintentのoperation/requestを照合し、当該intentだけを完了削除する。削除失敗はSavedと分けて残し、次回もreceiptで識別する。

publish APIは同期で、I/O途中の強制的な取消しは行わない。I6はUI thread外の直列writerへ依頼する。取消しはprepare/publish呼出し前とI2 pointer commit前に判定する。既に開始したpublish呼出しは終了まで待ち、実施済み書込を戻さず、pointer結果を照合してSaved/Unsaved/Recoverableを判断する。I1bに解析token・generation・送信APIを追加しない。Preparedをdropしてもcallerの原音Arcとintentは消えない。

### RecoveryIntent と回復候補

`RecoveryIntent` はhost専用serde DTOで `schema_version:1, operation:OperationId, request:Sha256, mutation_json:String, base:Option<RootRef>, planned_root:RootRef, planned_objects:Vec<BlobRef>` を必須とする。mutation_jsonは上記HistoryMutationのexact UTF-8 JSON、requestはそのhashである。planned_objectsは依存順、hash重複なしで、原音/result refsと生成されるRun/Trial/rootを列挙する。root自身のhashをroot本文へ含めない。名前は `<operation-id>.json` に固定し、文書内IDと一致させる。path/raw応答/秘密/原音やresult本体の二重コピーは持たない。固定入力はCreateTrialのmutation内にあるため、原音だけpublish済みでもTrialを再構成できる。

intent上限は既存設計8節の**512KiB（524,288 bytes）**を維持する。原音/result/Trial/root本文は埋め込まず参照だけを保持し、固定入力を含むmutation_jsonもboundedに扱う。mutation_json単体のUTF-8長が512KiBを超えた時点で拒否し、さらにJSON文字列のescapeと全fieldを含むRecoveryIntentの**serialize後exact bytes**を検査する。512KiBちょうどは他条件適合時に許可、1 byteでも超えればLimitExceededとし、directory/intent/objectを一切書かない。resumeでintentを再構築する場合も旧intent置換前に同じ検査を行い、超過時は旧intentと全objectを保持する。読取側もfile長→最大512KiB+1のbounded read→実長再確認→decodeの順で、超過を同じLimitExceededにする。AssetStoreの12MiB上限や履歴quotaの引上げによってintent上限を緩めない。

mutation解析後は各DTO/Trial256KiB/原音6MiB/report2MiB/root8MiBの上限を別に守る。planned_objectsの件数上限は一mutationあたり8件（現在のvariantは最大4件）とし、本文と計画を再計算して一致させる。unknown field/schema、不正hash/ID/順序/余分なrefは拒否する。deserializeした値の再serializeを、保存されていたmutation_jsonのhash照合に使わない。

```rust
pub struct RecoveryView {
    pub candidates: Vec<RecoveryCandidate>, // OperationId順、最大32件
    pub next_after: Option<OperationId>,
    pub diagnostics: Vec<HistoryError>,
}
pub struct RecoveryCandidate {
    pub operation: OperationId, pub request: Sha256,
    pub state: RecoveryState, pub missing: Vec<BlobRef>,
}
pub enum RecoveryState {
    AlreadyCommitted, OriginalMissing, AssociationReady,
    PayloadMissing, PublishedUncommitted, Conflict, Damaged,
}
```

recoverは無書込の一覧である。最新headが読めない/壊れている場合、diagnosticを付け、commit/resume可能と判定しない。有効なoperation名を順に扱い、壊れたintentもcursorを前へ進める。無効名はdiagnostic、100,000を超える列挙はLimitExceededであり、空成功へ落とさない。snapshotの取得中にheadが変わればConflictで一覧を再取得させる。

| disk/正本の状態 | 候補 / 明示resume |
| --- | --- |
| 最新正本に同operation/request receiptあり | AlreadyCommitted。finishだけ可能。原音/resultを再publishしない |
| Create intentだけ、原音なし | OriginalMissing。callerが同hash/lenの原音を保持していれば同意図でresume。元pathの再読・捏造はしない |
| 原音照合済み、Trial/root未公開 | AssociationReady。intentの固定入力から同TrialIdで関連付けを再構築できる |
| 結果/注記の必要leafが不足 | PayloadMissing。callerの同bytesまたは検証済み既存blobが必要。メモリ消失した結果の再生成/再POSTを行わない |
| 全planned object公開済み、正本receiptなし | PublishedUncommitted。全閉包と最新targetを再検証し、I2へのPublishedまで戻せる。公開rootのreceipt単独では成功ではない |
| 同operation別request、既存TrialId別内容、更新targetが変更済み | Conflict。全既存情報を保持し、自動union/reset/新ID化を行わない |
| intent/object未知版・破損・危険path | Damagedとdiagnostic。読めた別候補は表示できるが当該resumeは禁止 |

resumeは同request hashだけでなくmutation_jsonのexact bytesもintentと照合する。baseが進んだ場合、CreateのIDが現在rootに存在しなければ重複確認を含め再検証して最新rootへrebaseできる。IDが既にあれば入力一致だけで新規receiptを作らずConflictとする。更新はtarget.expectedが現在headと同一の場合だけrebaseできる。元baseのclosureと元計画の照合を済ませ、変更後のplanned_root/object refsを新しいintentとしてatomic置換・再読してから不足objectをpublishする。旧計画の公開objectは保持してquotaへ算入する。I3の取込unionとは別である。

### 独立試験seamと受入対応

fault seamは `HistoryFaults::check(&self, point:HistoryCheckpoint) -> Result<(),HistoryError>`、productionは常にOkの `NoHistoryFaults` とする。テスト向け `HistoryWriter::with_faults(storage, head, quota, faults)` を通常constructorと同じ検証で提供する。Checkpointは `BeforeIntent, AfterIntent, BeforeObject{index:usize}, AfterObject{index:usize}, BeforeClosure, AfterClosure, BeforeFinish, AfterFinish`。順序indexはPreparedのobject順であり、操作ID/本文を診断へ漏らさない。seamは固定Ioを返すfailpointで、検証の無効化/値の差替えはできない。I2のpointer前後faultは別単位で追加する。

| ケースID / AC | 独立fixture・観測点 |
| --- | --- |
| W01 / AH-AC02–03 | 手製の短いPCM16 WAV、固定reference/capture hash、既存rootを直接AssetStoreへ置く。caller Arcと全旧bytesを前後比較。選択元pathの変更を保存adapterが読まない。codec/復号/60秒判定の受入はI6であり、I1b単体の成功へ含めない |
| W02 / AH-AC04 | 各faultで再open、intent→原音→Trial→rootの存在とhashを独立列挙。原音publish前のintent確認、原音だけから同TrialId回復、Publishedを正本にしない。既存root/payload/過去revisionを不変比較 |
| W03 / AH-AC04/08 | 同operation同request、同operation別payload、未確定Busy、明示resume、後続正本receipt、cleanup失敗を別ケースにする。root/Trial/sequence増分が一度だけ。保存再試行時の解析/POSTは後続I6 spyで0増分 |
| W04 / AH-AC08 | 手製Running/Sending→正常/失敗/Interrupted、終端から別結果拒否、Run immutable field改変拒否、old revisionの旧Run保持。intent/resultは別OperationId。report/result本文不足、caller保持bytesからの保存のみ再試行 |
| W05 / AH-AC14 | writerのquota_planをoracleにせず、手製ファイルの長さと独立serialize bytesを加算。旧root/旧result/trash/orphan/temp/intentを含め、hash重複一回。80%直前/一致、上限直前/一致/超過、残存intent置換差分、checked overflow、上限低下、走査上限。intentはescapeを含むserialized長524,287/524,288/524,289 bytesを独立に作り、超過はLimitExceeded/無書込、resume超過は旧intent不変、読取超過はbounded拒否を検証 |
| W06 / AH-AC04/14 | head seamを段階間で変更、同対象変更/別対象更新、pending作成/読取不能、2個目Storage::atのlock失敗、アクセス拒否/ディスク書込失敗、root祖先とassets/recoveryのreparse・dangling link。拒否時の全既存bytes不変 |
| W07 / AH-AC12/14–15交差 | 同bytesのActive/Trash/Referenceを手製rootに置き、未確認・古いID集合・Reference→Practiceを検査。Running/Sendingのtrash拒否、停止終端後trash、trash後の遅延結果拒否、復帰で同ID/原音/全結果保持 |
| W08 / AH-AC04/08 | 未知版/未知field/不正operation名/欠損/改ざんintent、root破損、receipt不一致、32件pagination、部分診断。回復一覧が全ファイルを変更せず、自動処理しない |

fixtureの期待JSON/hash集合・総bytesはtest authorが手作りする。writerで作ったfixtureをwriterで読めたことだけで正当化しない。正常closure readerを併用する場合も、期待Trial数/ID/sequence/previous/receipt/resultを独立assertする。fault試験は通常エラー復帰とプロセス終了による残存状態を区別し、電源断/媒体故障の実機耐久性は未検証として残す。

### 実装・テスト担当への引継ぎ

I1b-2は二つの逐次小単位へ分ける。役割/model/effortと独立レビューgateは実装計画I1をそのまま継承する（designer/implementer/test author/reviewer = Astra/high、runner = Sol/medium）。計画者の再選定やgate変更を行わない。

1. I1b-2a: exact request、全mutation純粋適用、fresh-head/receipt/pending guard、quota、intent-before-publish、Prepared/Published、fault seam。実装所有は `src/audio_history/{store,recovery,mutation}.rs` とexport用の `src/audio_history.rs`、独立作者所有は `tests/audio_history_store.rs` とする。2026-10-07の接合で、純粋な変更計画/Run遷移/結果payload照合をmutation.rsへ分離する親承認を追加した。本体4＋独立test1の計5本であり、実装計画の5本超の再分割条件内である。reader本体の複製や汎用Framework追加はしない。recovery.rsはstrict intent codec/保存まで。restart resume/finish公開は次単位まで提供しない。W01–W07の該当範囲を作者が先に固定する。
2. I1b-2b: read-only recovery列挙、明示resume/rebase、confirm/finish、再起動fault fixture。同じ本体4本と独立testを逐次移譲し、W02/W03/W06/W08を拡充する。I1b-2aの合格期待値を変更して通さない。

段階間の制約：2aの既存intent読取はstrict codec・exact request hash・参照/長さの照合と容量計上までであり、そのintentからPreparedを再発行しない。同operationはBusy、異requestはConflictとする。公開されていないpayloadの欠損を空データで補わない。自sessionのpublishは自身の確定planとintent全bytesを照合する。既存intentの本文から計画を再計算して照合し回復へ使う実装は2bの必須gateであり、2aだけで回復の全受入を満たしたとはしない。

親によるreader接合は `results.rs` の既存result関係predicateを `check_analysis_result` / `check_advice_result` として抽出するだけである。readerと未公開payload検査で共用し、DTO検証・Run identity/membershipは各callerが保持する。これはwriter本体所有へreaderを移管する意味ではなく、接合後の既存reader回帰を必須とする。

test authorとimplementerは同一ファイルを同時編集しない。reader本体 `closure.rs` / `results.rs`、共有DTO、manifest、Progress、UI、backupはこの所有外である。reader helperの公開範囲変更が必要なら別所有接合を親へ返し、reader契約を黙って変えない。writer側は既存の親moduleのhash helperとpub(super)検証を利用できる。安全path処理の共通化が所有外へ広がるなら先に分割を明示し、所有ファイルへ無理に埋め込まない。

focused検証は `cargo test --locked --test audio_history_store`、既存reader回帰は `cargo test --locked --test audio_history_closure`、差分は `git diff --check` とする。具体case数・コマンド成功はrunnerの新しい証拠を要する。全体Windows delivery gateはI10に残り、本設計文書の検査を代替証拠にしない。

I2への必須handoffは、read-only最新Progress head実装、全writerの直列化、pointer前後のfault、commit不明時の再読、Savedと処理開始の制御である。I6にはArc保持、別OperationIdのRun intent/result、世代/Active照合、取消し終端→trash順を渡す。I3には全旧revisionと未確定回復候補の分離を渡す。UIの日本語/両軸中央寄せ/scale/狭幅はI8で `wordweave-egui-ui` を適用する範囲であり、このheadless保存追補に画面設計を追加しない。

主な残存リスクはI2の最新head実装の誤り、同一process内の別writer経路、atomic rename後の成否不明、保持object増加による容量/走査上限、Windows電源断耐久性である。SQLiteや第三headはI2との二重commitを増やすため採らず、input一致だけの自動mergeは新しいRun/注記/trashを失うため採らない。利用者に見える新しい動作・仕様値の変更は追加していない。未承認の完全削除/自動GC/結果再送へ回復処理を拡張してはならない。

文書検査（2026-10-07）: 相対リンク5件/欠落0、行末空白0、コードfence6本/対3組、`git diff --check`成功。追補の独立設計レビュー、ソース実装、上記W01–W08の実行は未実施であり、文書検査から合格を推定しない。設計担当の変更は本書のI1b-2追補だけである。
