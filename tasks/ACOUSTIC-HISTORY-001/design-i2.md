# I2 Progress移行と履歴commitの接合

2026-10-08 / ACOUSTIC-HISTORY-001 / 準備設計。担当は [implementation.md](implementation.md) I2指定の `gpt-6-astra` / `high` である。起動指定と実行metadataは別であり、後者・使用量の開始/終了カウンターは未取得である。入力は承認済み [spec.md](spec.md) AH-AC04/08/09/16、[design.md](design.md) 5/9節、[design-i0.md](design-i0.md) 5節、[design-i1b.md](design-i1b.md) のI2 handoffである。

完了境界は保存契約・所有分割・独立oracleの設計追補である。I1b-2bの独立Post完了前にI2本体へ着手しない。本書以外は編集しない。API設計Skillの型付き境界、加算的API、結果不明と失敗の区別を適用した。画面構成は対象外であり、UI SkillはI8の画面設計で適用する。既存consumerの保存呼出し変更は描画変更を含まない。

## 1. 現物根拠と差分の要点

調査基準HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋共有dirty差分である。I1b-2bは並行準備中のため、下記は完成を主張しない。

| 現物 | 確認した事実 / I2で必要な接合 |
| --- | --- |
| `src/store.rs` Progress/default/validate | 現在version=1のみ。既存fieldと旧fieldのserde defaultを維持してv2 rootを追加する |
| 同 Storage::load/save | loadはcommit::recoverを起動する。saveは`&self`、日別旧Progress退避後atomic_write。loadを最新head readerへ転用しない |
| `src/commit.rs` save/recover | 公開引数は`&Path`。pendingはexists()。intentのold/new exact文字列を照合後deck→progress更新。全履歴閉包の検証と保存ゲートが不足する |
| `src/audio_history.rs` / closure.rs | HistoryReaderは`&Storage`に依存するが、closureのStorage使用はdir参照だけである。既存公開署名を保って内部root読取口を追加できる |
| `src/audio_history/store.rs` | CommittedHistoryHead、Prepared/Published、confirm/finishの接続口が存在する。正本pointerはまだ提供しない |
| `src/backup.rs` | manifest v1、Progress直接decode、restoreは取込Progressをcommitしてそのcloneを返す。現root保持と実際の保存値の返却が必要である |
| `src/app.rs` と設定/配色/会話/媒体consumer | 保存後にcallerの`next`をUI正本へ代入する。diskだけrootを保持しても、このcloneは古いrootを再表示できる |
| `src/app/settings_ui.rs` 772/800/846、control_settings.rs 152、chat_media.rs 592/604/626 | 利用者選択pathへatomic_writeする別経路がある。JSONだけでなくTSV・診断・未保存媒体も稼働rootへの直接書込を迂回できる |
| `backup.rs::reject_asset_destination` | 既存はassetsだけをcanonical親＋末尾名のstarts_withで拒否する。新規directory作成なので既存rootそのものはcreate_dirで失敗するが、root以下の新規directory・alias全般の保護は不足する。単一file exportへそのまま転用できない |

原音/resultをProgressへinlineしない。唯一のmutable履歴pointerは`progress.json.audio_history`であり、third head、別DB、worker開始時Progress全体の復元を導入しない。

## 2. v1/v2の型・読取・移行

runtime `Progress`へ`audio_history: Option<RootRef>`を追加し、Defaultはversion=2/Noneとする。RootRefはpractice-historyの既存型を使う。`Progress::validate`はversion=2と既存field/RootRefの構造だけを検査し、filesystem、復旧、送信を呼ばない。

serde読取は単一の版dispatchを通す。`Progress`のDeserialize実装からも同じwire→runtime変換を使用し、backup/pending/UIの直接serde decodeが別の移行規則を作らない。private wire型とTryFrom等で実現し、公開Progressの既存field/メソッドを変更しない。

Sのprivate型・署名は [限定設計§1](history-progress-design.md) に固定する。全既存fieldを持つProgressWireと、欠落/nullableを区別するHistoryFieldを使用する。共通normalize/decodeはDecodedProgressへ元wire版を保持し、公開Deserializeはruntimeだけを返す。公開Progressへ由来版fieldを加えず、v2のNoneも明示nullとしてserializeする。未知top/重複/不正はInvalidDocument、正常形の未知版はUnsupportedVersionとし、複数不正の優先順位を追加仕様にしない。

| wire | 読取結果 |
| --- | --- |
| version=1、audio_history欠落またはnull | 既存fieldの意味を保持したruntime v2/None。既存の省略可能field/defaultを維持する |
| version=1、非null root | 不整合として拒否する。v1をroot付きv2として推測しない |
| version=2、audio_history必須、nullまたは有効RootRef | runtime v2。field欠落をNoneへ補わない |
| version欠落/未知/不正、root不正、未知top-level field | 拒否し原bytesを保持する。未知fieldを捨てて再保存しない |

必須nullable fieldの「欠落」と「null」を区別するwire実装を使用する。既存Settings等の後方互換defaultを一括でdeny_unknown_fieldsへ変える作業は含まない。既存schemaで認識する全fieldの値・学習記録・教材根拠・会話/添付を独立fixtureで比較する。履歴DTOのstrict未知field/未知版規則は既存どおりである。

読取だけではv1を書き換えない。旧v1からv2を書き出す経路は、gate下で取得した現diskのexact bytes、または全事前検査を通過した教材intent.old_progressのexact bytesを、`backups/progress-v1-<旧bytesのsha256>.json`へatomic保存・再読照合してから次へ進む。runtimeへの正規化後のversionではなく元bytesのwire版で移行要否を判断し、旧bytesを再serializeしない。旧fileなしは退避不要であり、旧v2もstrict検証後に退避不要である。既存同名異bytesは上書きせず停止し、日別backupで代替しない。退避を成功後も自動削除せず、v2→v1の自動rollbackや旧Progressだけのrestoreは行わない。

移行helperはgate確認後、target wire版=1なら直ちにno-opとする。通常saveはこの場合に移行目的の旧disk読取も追加しない。新教材saveの既存旧metadata読取・上限検査はtransaction保護として維持する。旧sourceの長さ・必須版・構造検査はtarget=2の移行契約であり、未知targetはtyped UnsupportedVersionとして拒否する。これにより、旧v1がprivate limitを超えていても新v1候補がlimit内へ縮小する既存通常saveを拒否しない。target=2の同じ旧sourceはLimitExceededで無変更拒否する。

旧pendingのnew wire版=1は元のexact文字列を比較・書込に使用し、検査用decodeでv2へ変換した文字列に差し替えない。new wire版=2、old wire版=1のrecoverは、§5の全事前検査後にintentの旧exact bytesを退避してから両metadataへ進む。事前検査での拒否はbackup/tempも変更しないが、検査後の実I/O失敗では新規directory・退避・残存tempが生じ得る。退避確認前に両metadata/pendingを書き換えず、確認済み退避を自動削除しない。

最小API・エラー分類・独立oracleは [M限定設計](history-migration-design.md)（独立レビュー入力SHA-256 `27D5233049D7F7B370381A77196335FBB649B9CB2FAED060CF188E3257F040AA`）を参照する。Mはv1のDefault/validateを保ったhelperとnormal/new material hookの準備、Sはwire版保持付きcodecと実v2候補によるS-M検証、Rはrecover hookとR-M検証を所有する。Mのprivate target=2試験を公開v2保存の証拠にしない。SからRまでの中間状態では全first-v2-write保護は未完成である。M本体は統合後の最終Preまで未着手とする。

SではM旧source検査と既存progress_wire_version署名をstore内の同じcodecへ接続する。旧v2もstrict検証後にのみ退避不要とし、source上限超過/破損を無変更拒否する。private target1 no-opは維持するが、Sの通常公開候補はruntime2であるためM時点の公開v1縮小成功を継続する契約ではない。元wire1 pendingのexact replayとdecode後runtime2は別々に検証する。S-Mの具体的oracle改訂と最大4file境界は [S限定設計§3](history-progress-design.md) とplannerのS限定計画deltaを正とする。

## 3. 読取窓口と書込ゲート

以下は追加するhost内契約名である。既存`Storage::{load,save}`、`commit::{save,recover}`とI1の公開署名は維持する。

```rust
pub struct ProgressHistoryHead;
pub enum HistoryCommitOutcome {
    Committed(CommittedHistory), CancelledBeforeCommit,
}
impl CommittedHistoryHead for ProgressHistoryHead {
    fn read_current(&self, storage: &Storage)
        -> Result<Option<RootRef>, HistoryError>;
}
impl Storage {
    pub fn read_progress(&self) -> Result<Progress, String>;
    pub fn save_progress(&self, candidate: &Progress) -> Result<Progress, String>;
    pub fn commit_history(&self, published: &PublishedHistory,
        cancel: Option<&std::sync::atomic::AtomicBool>)
        -> Result<HistoryCommitOutcome, HistoryError>;
}
// commit module: callerがUIへ反映すべき実保存値を返す加算的入口
pub fn save_progress(root: &Path, deck: &str, candidate: &Progress)
    -> Result<Progress, String>;
```

`read_progress`はroot/祖先→leaf型安全確認→size上限付きopen/read→版dispatch→構造validateだけである。recover、移行保存、mkdir、backup、intent変更をしない。検証済み通常root配下のprogress.json leaf NotFoundだけを新規v2/Noneとする。root/祖先欠損・取得不能はIo、非directory・symlink/reparseやleafの非通常fileはUnsafePathであり、存在確認後のopen/readでNotFoundとなってもIoとする。未知版もNoneへ落とさない。`ProgressHistoryHead`は同じtyped読取で得たrootを返す。strictとはProgressのschema/構造のstrict読取を指し、closure検証はI1のwriter/confirmが別途必ず行う。pending時はheadをcommit済みとして使用しない。callerはstrict pending guardを先に通すが、read/head自身のpath検査を省略する根拠にはしない。

内部codec/読取errorはUnsupportedVersion/InvalidDocument/LimitExceeded/UnsafePath/Ioの固定分類を持つ。private read_progress_typed(root, limit)はOption<DecodedProgress>を返す。公開Stringは既存consumer向けの日本語への変換だけであり、head adapterは同じtyped helperの内部分類からHistoryErrorへ直接変換する。表示文字列やOS本文をparseして分岐しない。最小署名と非副作用境界は [S限定設計§1/2](history-progress-design.md) による。

`Storage::load`はGのgate取得→recover_under_guard→read_progressの順である。read/headはgateを再取得しない。通常起動のloadは履歴closureを成功条件にしない。局所欠損でもProgress/rootを保持し、HistoryReaderの部分閲覧に診断を渡せる。未知Progress自体は読取不能でありDefaultをdiskへ保存しない。

同一process内はstore moduleの単一private `Mutex<()>` と非公開 `ProgressWriteGuard`で全metadata/history変更を直列化する。root別registryを作らず、同じrootの大小文字/相対・絶対path表現差による別lockを避ける。別rootも同process内では直列であるが、データは各rootに固定し共有しない。別processは既存app.lockが担当する。Pathだけのcommit APIはapp.lock取得済みcaller用であり、勝手にStorage::atを再取得しない。公開path APIが単独でOS排他を証明するとは扱わない。

- public Storage save/load、commit save/recover、HistoryWriterのprepare/publish/resume/finish各操作が入口で一度取得し、検証から副作用終了まで保持する。同期の短い保存操作単位であり、writerオブジェクトの寿命全体や解析/通信/人の確認中は保持しない。
- 内部呼出しは`*_under_guard(..., &ProgressWriteGuard)`を使う。load→recover、commit save→recover、resume→prepare/publish、finish→confirm等で公開locking入口を再帰呼出ししない。lock付きcallbackへアプリ処理を呼び戻さない。
- read_progress/head/HistoryReader/atomic_writeはgateを取得しない。履歴readerはimmutable objectを読む。履歴の復旧一覧はI1のhead再確認契約を維持する。poisonは書込停止エラーとし、黙って解除しない。
- mutexの待機は内部同期だけであり、ネットワーク/worker完了待ちは含めない。別rootの同時testは順番に完了し、try_lock由来の偶発Busyへ変えない。Busyの操作重複契約はI1のintentで判定する。
- pending検査は`Result<bool, ...>`のprivate strict helperを使い、NotFound以外のI/O失敗とreparse/directoryを拒否する。既存`pending(root)->bool`はUI用互換入口として不明をtrueへ倒す。falseへ落として保存を進めない。

commitのPath入口から同一closure実装を呼べるよう、HistoryReader内部保持を`&Path`へ置換し、`new(&Storage)`/with_limitsは既存署名のままdirを借用する。crate-private `for_root(&Path, limits)`を追加し、closure.rsの2箇所のdir取得を接続する。ロックを持たない偽Storageを構築しない。root/祖先/assets/leafのreparse検査とbounded readを既存readerのまま維持する。

## 4. 通常保存と履歴pointer commit

### I2a-G限定接合（2026-10-08 designer追補）

以下は§3を現I1b-2b Post版へ具体化するprivate APIである。designer `history_commit_design`（指定Astra/high）のread-only提案を親が統合した。Gはschema/default/migration/ProgressFaultsを変更せず、結果不明の検出はN/Pに残す。実装前にこのdeltaを独立レビューする。

```rust
// store.rs。型と入口はpub(crate)、fieldsはprivate。
enum ProgressGateError { Poisoned, WriteBlocked, Reentrant }
struct ProgressWriteGuard { /* MutexGuard<'static, ()> */ }
fn acquire_progress_write() -> Result<ProgressWriteGuard, ProgressGateError>;
impl ProgressWriteGuard {
    fn ensure_writable(&self) -> Result<(), ProgressGateError>;
    fn block_writes(&self); // 解除APIを提供しない
}
// commit.rs。Limits付きhelperはprivateのままである。
fn recover_under_guard(root: &Path, guard: &ProgressWriteGuard) -> Result<(), String>;
enum PendingCheckError { Io, UnsafePath }
fn pending_checked(root: &Path) -> Result<bool, PendingCheckError>;
```

単一Mutexとgate下でしか更新しないprocess共通blockフラグを保持する。thread-localの保持中印で同thread再入を取得前に拒否する。別threadは通常待機し、try_lock由来Busyへ変更しない。Poisoned/WriteBlockedはHistoryErrorKind::Io、ReentrantはBusyへ型で変換し、既存String APIは固定日本語を返す。表示文字列から分類しない。guardの解放と失敗時にも所有中印を整合させ、panicによるmutex poisonを解除しない。head/fault callbackが保存入口を呼んでもdeadlockさせない。

取得位置はStorage::loadとsave_with_limit、commitのsave_with_limits/recover_with_limits、HistoryWriterのprepare/publish/resume/finishで各1回である。loadはrecover_under_guardへ、教材saveはprivate recover_with_limits_under_guardへ接続する。under_guard入口でもensure_writableを行い、既存guardの保持を証拠に書込停止を無視しない。現履歴resumeは公開prepare/publishを呼ばず、finishもchecked_committedへ直接行くため不要な再分割をしない。confirm_committed/head/reader/atomic_writeは取得しない。

pending_checkedはrootと祖先・leafを検査し、正常な親の下のpending leaf NotFoundだけfalseとする。directory/reparse/symlinkはUnsafePath、その他I/O不明はIo。公開pending boolはErrをtrueへ倒し、save/recoverは型付き結果を使う。履歴側recovery::guardは既にpaths＋strict metadata検査を行うためGでは変更しない。

G所有はstore.rs/commit.rs/audio_history/store.rsと独立tests/audio_history_gate.rsの4本を維持する。private gate試験は同file cfg(test)を作者→実装者へ逐次移譲する。poison/blockは同じtest executableのexact caseを別processで実行し、親のglobal状態を汚染しない。child分岐・終了コード・実行case・成功markerを照合し、timeoutは未完了/失敗とする。reset/poison解除用APIを足さない。手製pending/履歴fixture・channelで保存区間の非重複、再入拒否、正常完了とbytes保持を確認する。新しいlink昇格承認は含まない。

G時点のStorage::loadは復旧を伴うためblock/poison時に拒否される。純粋read_progressはSで追加するので、G単位で「書込停止後の閲覧接続完了」を主張しない。

### 完成I2の通常保存・pointer契約

通常`save_progress(candidate)`は候補の非履歴fieldsを採用し、gate下でSのpure readと同じstrict契約で読んだ最新diskのrootを明示継承する。候補の古いSome/Noneをroot更新要求と解釈しない。旧fileなしならNoneのみである。最新版の構造と継承rootの全closureを検証し、候補をcloneしてrootを継承した後に候補の構造・exact serialized sizeを検証する。移行退避/日別退避→atomic replace→exact再読照合を行い、実際に保存したProgressを返す。候補メモリを変更せず、既存`save(&Progress)->Result<(),String>`はこの処理の返却値だけ捨てる互換wrapperである。

Nの最小署名とsnapshot境界は [限定設計§1–3](history-normal-save-design.md) に従う。Sのread_progress_typedからpath検査/bounded readだけをprivate read_progress_bytesへ抽出し、typed readerも同じhelper＋decode_progressへ委譲する。Nは1回の読取で得た旧exact bytesとそのdecode結果を保持し、root継承・M退避・日別backup・結果判定に同じsnapshotを使う。旧v1 bytesをruntime2の再serializeで代用しない。既存save_with_limit/with_faultsの署名とGのgate1回/strict pending拒否を維持する。

移行退避の確認後に日別backupを検査し、既存同日fileを置換せず、新規分へ旧snapshot bytesを保存する。M前の検査拒否だけを全ledger不変とし、日別leaf検査拒否でも先行する確認済みhash退避は保持する。日別検査を前倒しせず、実I/O失敗による追加残存物とも区別する。正本/deck/pending/objectsを日別失敗で変更せず、N03bの差分ledgerを用いる。

新root変更は`commit_history`だけが行う。順序は以下である。

1. I1 prepare→publishでPublishedを取得する。I1 writerのborrowを解放する。publishはSavedではない。
2. gateを取得し、strict pending guard、最新disk Progress、最新rootの全closureを読む。同operation/requestのreceiptが最新正本にあれば検証してCommittedを返す（後続rootでもよい）。既commitは取消要求があってもCommittedである。同operation別requestはConflictである。
3. receiptなしなら最新root==published.baseを要求する。nextの全closure、receiptのoperation/request/committed_revisionに加え、次段落のintent/計画束縛を検証する。public Publishedのfieldを無条件に信頼しない。base不一致はConflictであり旧snapshotを書き戻さない。
4. 最新disk Progressをcloneしてaudio_historyだけnextへ変更する。保存上限/移行退避を満たし、pointer直前にcancelをAcquire読取する。要求済みならCancelledBeforeCommitを返しpointerは変えない。要求なしならatomic replaceする。候補UIやworkerのProgress全体を引数に取らないため、既存の新しい設定/会話/教材fieldを巻き戻さない。
5. pointer成功/エラーのどちらでも、結果不明の可能性がある境界ではgateを保持してexact diskを再読する。予定new全文bytes一致＋closure/receipt一致ならCommitted、旧全文raw bytes一致なら未確定、第三bytes/読取不能/破損なら書込停止・要回復である。同じrootでも非履歴bytesが違えば第三状態であり、root参照だけの一致で成功/旧状態としない。atomic_writeエラーを単純な未保存へ変換しない。
6. callerは確認されたrootだけを現在UI Progressへ反映する。UIに未保存の設定/会話があれば保持する。別途I1 confirm_committed/finishを呼ぶ。finish失敗は保存済み＋後処理未完であり未保存へ降格しない。

Publishedの公開fieldsから新root変更権限を得ることはできない。I1側へcrate-private associated helper `HistoryWriter::verify_publication_under_guard(storage: &Storage, published: &PublishedHistory, guard: &ProgressWriteGuard) -> Result<(), HistoryError>`を追加する。既存の公開署名/型の生成を壊さず、追加helper自体はgateを再取得しない。このhelperは該当operationのstrict intentを読み、operation/request/exact mutation hash、intent.baseと最新base、planned_rootとnext、planned_objectsを照合する。さらにI1b-2bの既存resume用計画再計算を再利用し、検証済みbaseとintentの固定mutationから生成したroot/object refsが計画と一致することを要求する。必要leafは既にpublishされた同hash/len bytesから取得し、無い場合は拒否する。変更前root内の無関係Trial/receipt/sequenceが消えていないことは、同じ閉じたmutationの再計算で保証する。closure＋自己整合receiptだけの検証では不十分である。

intentなし/別request/next差替え/別base計画は新commitを拒否する。既に正本に同operation/requestが確定している場合だけ、finishでintentが消えていても前記2の冪等なCommittedを許す。これを新pointer書込の許可へ転用しない。

Vの最小API/再計算接合は [限定設計§1–3](history-publication-authority-design.md) に従う。既存at/checked_old_planをprivate read_history_at/checked_old_plan_with_readerへ抽出し、元baseとexact mutationから計画root・順序付き全object refs/bytesを再計算する。verifyはplan.complete()だけでなくmissing列が空であることを要求する。生成metadataがメモリ上で揃っていても実file欠損なら拒否し、再生成/publishはしない。最新baseに同operation receiptがある場合は新commit権限を与えず、検査の最後にも同guard下でstrict pending/最新head==base/strict intent raw不変を確認する。検査はread-onlyであり、拒否自体でwrite-blockを変更しない。公開Published、自己整合closure/receipt、検証Okを持ち運べるauthorityへ転用しない。

既commit確認は追加crate-private associated `HistoryWriter::confirm_publication(storage: &Storage, published: &PublishedHistory) -> Result<CommittedHistory, HistoryError>`へ接合する。現confirm_committed本体をprivate confirm_with_reader_and_headへ一度だけ抽出し、既存公開I1はself.reader(u64::MAX)/self.head、新接合は100,000 nodes/u64::MAX retainedのreaderと固定ProgressHistoryHeadを渡す。custom headは従来I1とprivate内部だけに残し、Pへcaller headを追加しない。strict guard→operation/request→next closure/receipt→存在intentのstrict検査/旧計画再計算/affected→最新head closure/receipt完全一致→pending/head再確認という既存順序/error・読取上限・cleanup後成功条件を維持する。新confirmもgateを取得せず、書込/cleanup/write-block変更を行わないためguard保持中とwrite-block中の純粋readが可能である。偽writer/Storage・再open・検証複製は行わない。Busyには旧計画再計算由来もあり、それだけを未commitの証明や新commit許可とせずV検証を必須とする。詳細は [V限定設計§2/4](history-publication-authority-design.md) とPの後続接合による。

通常保存はatomic publishのOk/Errにかかわらずgate下でexact再読する。next bytes一致ならOk（old==nextもこの判定を優先）、旧raw bytes一致ならErr/no block、第三bytes・既存file消失・再読不能なら要回復として次の上書きを止める。旧fileなしで正常なroot配下のleafがなお欠損する場合だけ旧状態一致とし、root消失/open後NotFoundを未保存の確証にしない。要回復状態はgate保護下のprocess共通write-blockフラグで保持し、StorageとPathの全writerが同じblockを確認する。単一rootを扱う製品ではroot識別registryより小さく、別表記によるblock迂回もない。結果不明を扱う隔離試験はprocessを分ける。解除は自動的な次saveでは行わず再起動後の読取/回復確認に委ねる。block中もread_progressと部分閲覧を許す。

I1への再試行は同OperationId/request/payloadでresumeする。publish済みnextのreceiptだけを成功証拠にせず、正本head内のreceiptを使う。復旧時の再解析/POSTは0回である。未知root/trial/run・局所破損は閲覧diagnosticへ残すが、通常保存を含む新metadataの書込はstrict closureで拒否する。rootをNoneへ戻す回避策は認めない。

Pの最小API/同一snapshot/保存済み判定は [限定設計§1–3](history-pointer-commit-design.md) に従う。既定HistoryCommitOutcome/commit_history署名を維持し、最内側private commit_history_with_publisherだけがgateを1回取得する。strict pending→旧raw/typed decode→完成Vのconfirm_publicationで既commitを先に判定し、後続root/cleanup済みも厳密receipt再確認後だけ成功とする。この経路は取消済みでもCommittedであり、M/pointer/faultを呼ばない。confirm Busyは未commitの証明ではなくV必須再検証へ進む候補である。SetTrashのRunning/Sending拒否Busyも伝播し得るため、最新base一致とverify_publication_under_guardを省略しない。V後にも旧raw不変を確認し、最新Progressのaudio_historyだけnextへ変更する。

Pは構造/serialized size検査後、同じ旧snapshotのexact bytesをMへ渡す。wire1をruntime2再serializeして退避せず、pointerに日別backupを新設しない。BeforePointer→strict pending/旧raw不変/ensure_writable→Acquire取消読取→atomic試行→AfterPointer→実再読の順とする。Before faultでも結果は実再読し、取消し/Before失敗でpublishを試行しなければAfterを呼ばない。atomic/AfterのErrや遅い取消しでも実new＋strict confirm＋最後のexact再読が成功すればCommittedである。旧raw一致はfault/publisher ErrならIo、書込効果なしOkならBusy、いずれもblockしない。第三bytes・再読不能・newのclosure/receipt確認不能・pending不明は共通write-block/Ioである。旧Noneの一致は検証済root配下のleaf欠損だけとする。pointer試行後の後続rootは第三状態であり、次回入口の冪等な後続root確認とは区別する。

PのM前拒否だけが全ledger不変であり、M後取消し/Before失敗には正しいhash退避が残り得る。原音/object/intentを削除せず旧Progress全体を復元しない。CommittedHistoryの公開field自体をauthorityとせず、この呼出しの実disk確認を保存済みの根拠とする。UIは返却current rootだけ採用し非履歴未保存値を保持、finishは別途再検証し、その失敗でCommittedを未保存へ降格しない。

## 5. 教材transactionと旧backup接合

`commit::save_progress(root, deck, candidate)`は同じgate下で最新rootを継承し、そのclosureとdeck/Progressを検証する。既存どおり旧metadataをbounded readし、intentのescapedサイズまで検査した後、target wire版=2なら必要な旧v1退避を確認してからpending intentを保存する。target=1の移行helperはno-opだが、既存旧metadata読取・上限検査は省かない。intentは既存old/new exact文字列を保持する。新しいmetadataを一つでも書く前にpendingを読戻し照合する。既存saveは返却値を捨てるwrapperとする。

Tの最小API/具体順序は [限定設計§1–3](history-material-save-design.md) に固定する。R完成版のread_recovery_bytesとdecode_progress_for_materialで同じ旧raw snapshotと最新rootを得て、candidateのaudio_historyだけを置換し非履歴fieldを保持する。M退避前に構造/全closure/旧新64MB・100MB/実escaped intent350MBを検査し、旧wire1は原bytesを退避する。旧fileなしの候補SomeもNoneへ置換する。公開save/既存limits/fault入口は同一新本体の返却Progressだけを捨てる互換wrapperとし、gateは最内側で1回だけ取得する。新たなIntent schema・root更新権限・日別backupは追加しない。

Tのpending publisherはOk/ErrどちらでもAfterMaterialIntentと実再読で決着する。pending exactかつ両metadata旧exactならintent確定、pending欠損かつ両旧なら未保存Err/no new block、第三pending/metadata変化/再読不能ならwrite-blockでRを呼ばない。AfterMaterialIntent faultで確定pendingを保持して止まる場合は既知の中断であり、先行M退避を削除しない。正常確定時だけRのrecover_with_limits_and_faults_under_guardへ同じguard/faultsを渡し、Rの全preflightとM再確認を省略しない。RのErrを両newらしいという理由で成功へ格上げせず、RのOk後にも今回の両new exactとpending欠損を確認して実保存Progressを返す。不一致/読取不能はblockし、旧snapshotへrollbackしない。

recoverはpendingの型/サイズ、deck、新Progressの構造と全history closure、下記の旧新root同一性、現在deck/progressが各oldまたはnew bytesであることを**すべて先に**検証する。どれか失敗なら両metadataとpendingのexact bytesを保持し、backup作成・pending退避も行わない。進行中intent内の新rootを最新rootでmerge/再baseしない。外部編集があれば停止する。

Rの具体的preflight・署名は [限定設計§1/2](history-material-recovery-design.md) に固定する。storeへcrate-private `decode_progress_for_material(bytes: &[u8]) -> Result<(u32, Progress), HistoryError>`という共通codecの薄いbridgeだけを追加し、元wire版とruntimeを分離する。Pathのrecoverは既存HistoryReader::for_rootで新rootの全closureを検証し、Storageを再openしない。pending/各metadataはroot/祖先/leafの安全型とbounded readを通し、存在確認後の消失をNoneへ落とさない。実pending上限・embedded旧新上限、両metadataそれぞれのold/new exact一致、archive先の安全型/未使用確認をすべてM退避前に終える。旧deckのparse成功や外側Intent schema変更は新しい復旧条件にしない。

I2教材intentの権限は非履歴fieldsの更新に限る。`old_progress: Some(bytes)`は必ず同じstrict版dispatch/構造validateで読み、取得したaudio_historyをold_rootとする。破損・未知版・読取失敗をNoneやDefaultへ変換しない。`old_progress: None`のときだけold_root=Noneとする。new_rootとold_rootのOption<RootRef>がhash/bytesまで完全一致することをsaveのintent構築時とrecoverの事前検査時の両方で要求する。A→None、A→別B、旧Progressなし→Someは、newのclosureが健全でも拒否する。旧v1と新v2で両方Noneなら許可する。I3のunion root更新には後続の明示intent/CAS契約が必要であり、I2の無印教材intentへ例外を追加しない。

全事前検査後、new wire版=2かつintent.old_progressのwire版=1なら、Rのrecover hookで旧exact bytesのcontenthash退避・再読を保証する。正常save由来の退避が残っているとは仮定しない。current progressが既にnewでも旧bytesはintentから取得する。先行退避が欠損なら作成し、既存同名が異bytes/破損/未知版なら上書きせず両metadataとpendingを保持して停止する。old_progress=Noneなら旧fileの退避は不要であり、new wire版=1のexact replayへ移行処理を追加しない。

必要な退避の確認後はdeck→progress→両exact bytes再読→pendingを既存backupsへ移動、の順である。事前検査不合格や既存退避不一致ではbackup/tempも不変である。検査成功後の退避I/O失敗では残存物が生じ得るが、両metadataとpendingは書き換えず、正しい退避を削除しない。M後にpending/currentの再確認で変化を検出した場合も、先行した確認済退避まで不変とは主張しない。deckだけnew等の中断状態は物理的にはあり得るが、pendingがある間は製品の新状態として公開せず全writerを止める。再起動recoverが両方を揃える。単にprogressだけoldへ戻すrollbackはしない。

Rの書込Ok/Errは両metadataと開始時pendingの実再読で判断する。各fileがold/newの範囲内でpending exactが残る場合は既知の中断であり、失敗時はErr/no new process block、pendingにより他writerを拒否する。書込対象がnewならatomicがErrでも次へ進み、oldならOkでも停止する。第三bytes・許されない欠損・再読不能・pending不明消失は共通write-blockを設定する。wire1 pendingは元exact文字列でreplayし、runtime2でserializeし直さない。詳細のfault停止位置・pure read境界は [R限定設計§3](history-material-recovery-design.md) に従う。

finishは両metadata new exactの確認後、pendingを選定済みarchiveへ移し、その元raw bytesを再読照合する。両new・pendingなし・archive exactならrename後Err/AfterMaterialFinish faultでもOkである。pending exactが残る場合は両newでも「教材復旧後処理待ち」Errとし、履歴書込を引き続き拒否する。archive異bytes・不明消失・再読不能はwrite-blockとする。rename未到達のdirectory作成失敗では、両new/pending exactを確認できれば後処理待ちとし、未作成archiveを読めないだけで結果不明にしない。初回事前不変とfinish段階の残存物を区別し、archiveの上書き/自動削除/偽pending再構成は行わない（[R限定設計§4](history-material-recovery-design.md)）。

pending復旧拒否時でもread_progressと部分HistoryReaderは利用できる。I2はこのread-only出口とwrite-blockを提供し、UIは読める保存済みデータをDefaultへ置換しない。回復専用画面の描画はI8であり、現在app.rsのfatal時に設定以外を隠す挙動のまま「回復閲覧を提供済み」と判定しない。画面接続の未完了をhandoffに残す。

I3のbackup v2/union実装は本書の対象外である。ただしI2で次を満たす。

- manifest v1と履歴なしの旧Progressは同じ版dispatchで読め、既存deck/媒体の検査を通る。restoreはgate下の最新現rootを保持し、未保存draftの退避とai_calls最大値を維持し、commit::save_progressの返却ProgressをUIへ返す。
- `preserve_current_progress`も最新rootを反映したメモリsnapshotを退避する。これは正本commitではなく、root以外の未保存draftをdisk版で置換してはならない。
- backup v1 exportは最新diskまたはcandidateのいずれかにhistoryがある場合、宛先作成前に未対応を返す。v1 manifestにrootだけを入れて履歴保存成功を装わない。両方Noneの経路は既存機能を継続する。v1 export wireはversion=1/履歴fieldなしへ明示投影し、旧readerへの後方互換を保つ。
- v1 manifestに非null historyが混入していればprepareで拒否する。未知manifest版は既存どおり拒否する。I3のv2をI2で空historyとして扱わない。
- 製品提供gateは元設計どおりI3後である。I2中のhistory付きexport拒否は開発中の保護であり、最終製品のbackup仕様変更ではない。

### 利用者選択の書出し先から正本を保護する境界

設定JSONはメモリ上Progressの外部退避であり、Storageのcommitではない。`progress.json`等を保存先に選べても、この経路にroot変更権限を与えない。稼働rootそのものと全配下（progress/custom.tsv/pending、assets/audio-history、backups、app.lockを含む）を外部書出し先から除外する。内部の移行/日別backupはこの外部export APIを通らず、前節までのgate付き保存契約を使う。

store.rsへ狭い共通宛先境界を置く。単一file用`write_external_file(root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), String>`はgate取得→宛先検査→既存atomic_writeを一つの同期操作とする。directory用`create_external_directory(root: &Path, dest: &Path) -> Result<(), String>`は同じprivate検査のDirectoryモードを用い、gate下で作成直前の検査→create_dirを行う。backup.rsとchat_media.rsからこの加算的入口を呼び、保存済みrootを更新する能力は持たせない。directory exportの完了marker最後/既存宛先上書き禁止は維持する。共通検査を公開atomic_writeの内部へ入れて内部commitを禁止する設計にはしない。

宛先検査は以下を満たす。

1. callerは実際のStorage.dirを渡す。Storage取得に失敗した画面でも、Storage::openと同じ副作用のないroot解決関数から試行対象rootを得る。既定rootを開き直さない。rootを特定・検査できなければ安全と推定せずエラーとし、別rootを保護したことにしない。
2. fileは既存親directoryと任意の既存leaf、directoryは新規作成予定pathと既存親directoryを検査する。絶対化と既存部分のcanonical解決だけで文字列比較を終えず、OSが報告するdirectory identity（Windowsのvolume/file identity等）で親から祖先を辿り、保護rootと同一なら拒否する。大小文字、`.`/`..`、別drive表記・短縮名が同じdirectoryを指す場合も拒否する。似た名前の外部directoryは許可する。root未作成の場合は既存祖先と未作成成分を正規化してその予定領域を保護し、既存親のないdestを自動作成しない。
3. 入力pathの既存祖先/leafのsymlink/reparseは、履歴readerと同じsymlink_metadata＋Windows reparse判定を使って拒否する。directoryをfileとして扱わない。既存fileの複数hardlinkも拒否し、保護fileへの外部hardlink aliasを許さない。ADS/device等、通常fileとして境界を確認できない指定は拒否する。Windowsの大小文字同一性をRustの単純Path文字比較やASCII lowercaseで代替しない。
4. 検査した解決済み親へ固定して書き、書込直前にも親/leafのidentityとlink状態を再確認する。拒否・検査不明時はtemp/宛先を作らず、既存の保護bytesと既存外部宛先を変更しない。既存atomic_writeによる外部file置換の成功/失敗は元の契約を維持する。

これは通常操作での誤選択、安定した別表記/alias、同processの保存経路迂回を防ぐ境界である。他processが検査後にdirectory/linkを敵対的に交換するTOCTOUの完全防止や、権限を持つ外部processの直接書込防止までは主張しない。identityを取得できない環境を文字比較へ弱めず未対応として返す。UIは既存のエラー表示で別の外部保存先を求め、成功表示や未保存payloadのclearを行わない。

共通化するのは宛先の領域/alias検査だけである。backup v1の内容/closure検査や新規directory条件はbackup.rsに残す。現行reject_asset_destinationはこの共通Directory検査へ置き換え、assets限定の古い判定を並行正本にしない。Settings JSON/TSV/診断と未保存媒体の通常の外部出力は維持する。processの正本write-block中も、安全な外部宛先へメモリを退避する操作は許可する。この限定入口は正本を書けないため、write-blockの解除にはならない。JSON退避を音声原本込みの完全backupや新しい保存済み判定へ昇格しない。

### I2d-H限定接合・実装handoff（2026-10-11）

入力は承認済みAH-AC04/16、I2-T08/T08b、implementation.mdのd-H/B行とb-B Path bridge deltaである。b-B独立76件・最終Post PASS/Closedは親から受領した。本節はd-Hの内部契約だけを具体化し、独立Preの判定を代行しない。指定は`gpt-6-astra/high`、実行metadata・開始/終了usage counterは未取得である。現物照合HEADは`a0e97bb9b49b6875452805ab99a388497c0bd789`、SHA-256は`src/store.rs=516F5DA0E8C6CFDAB76BACAC9578BD477BBE85852A908534C2E5FCCB0F2C2EA9`、`src/backup.rs=12A20A9E5C509574B078A77C47E6B836A112A81EAAD0B8419CDC8E028DC84261`である。

現物の不足は、通常gate取得が内部でwrite-blockを拒否すること、Storage::openのroot解決がopenから独立していないこと、atomic_writeが検査前に一時名を内部生成することの3点である。b-Bの`read_progress_for_backup`は再利用できる。d-Hの所有は本体`src/store.rs`と新`tests/progress_export_destination.rs`の最大2fileを維持する。private試験はstore内の新`cfg(test) mod i2_export_destination_tests`を作者→実装者へ逐次移譲する。backup/consumer/manifest/依存fileはd-Hで編集しない。

#### H1. 取得口と型の境界

次の加算的署名を固定する。`ExternalExportGuard`のfield/constructorはstore privateとし、Clone/Deref/内部guardの返却を設けない。MutexGuardにより同じthread内に寿命を限定する。

```rust
pub fn resolve_storage_root() -> Result<PathBuf, String>;
pub fn write_external_file(root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), String>;
pub fn create_external_directory(root: &Path, dest: &Path) -> Result<(), String>;

pub(crate) struct ExternalExportGuard { /* private ProgressWriteGuardを所有 */ }
pub(crate) fn acquire_external_export() -> Result<ExternalExportGuard, ProgressGateError>;
pub(crate) fn write_external_file_under_guard(
    root: &Path, dest: &Path, bytes: &[u8], guard: &ExternalExportGuard,
) -> Result<(), String>;
pub(crate) fn create_external_directory_under_guard(
    root: &Path, dest: &Path, guard: &ExternalExportGuard,
) -> Result<PathBuf, String>;
```

`resolve_storage_root`は現在の`LOCALAPPDATA`のOsString取得→`WordWeave5`結合だけを抽出する。環境変数欠落時の既存日本語エラーを保持し、canonicalize/mkdir/app.lock/recover/gateは行わない。Storage::openは`Self::at(resolve_storage_root()?)`へ委譲するだけであり、Storage::atの契約は変更しない。実Storageがあるcallerは必ずそのdirを使う。Storage取得失敗時のcallerは同じ試行rootを保持するか同resolverを使用し、カスタムroot失敗を既定rootへ置換しない。環境が変更された場合も試行rootの保持を優先する。

現在の取得処理から、再入検査→同じPROGRESS_WRITE.lock→同じthread-local設定→ProgressWriteGuard生成だけをprivate `acquire_progress_exclusion() -> Result<ProgressWriteGuard, ProgressGateError>`へ抽出する。既存`acquire_progress_write`はその後ensure_writableを従来どおり呼ぶ。新external取得口だけはensure_writableを呼ばずwrapperへ包む。poison/reentrant拒否、他threadの待機、Drop時のthread-local解除/poison保持は共有する。external tokenから正本writer用tokenへ変換する口、別Mutex、flag解除、lock中のアプリcallbackは追加しない。

公開file/directory入口はexternal guardを1回取得して対応under_guardへ委譲する。under_guardは再lock/ensure_writableを行わず、全宛先検査を必ず実施する。directory版の返却値は検査・作成した解決済み絶対pathであり、公開版だけが`()`へ捨てる。複数file exportのcallerはunder_guardの返却pathを後続の基点に使い、元の相対/別drive表記へ戻さない。

#### H2. Windowsの同一性と予定root

store内private型は、`ExternalDestinationKind::{File, Directory}`、`FileIdentity { volume_serial: u64, file_id: [u8; 16] }`、`ProtectedRoot::{Existing, Planned}`、`CheckedExternalDestination`に限定する。Existingはroot identity/handle、Plannedは最寄り既存祖先identity/handleと未作成成分列を所有する。Checkedは元の絶対root/dest、解決済み親とleaf、祖先/親のidentity・handle、leafの`Absent | ExistingFile(identity, links)`、modeを保持する。型の値をcallerに構築させず、成功boolだけを後で別pathへ転用しない。bytesはcallerから借り、Progress/dirty/保存済み判定は所有しない。

1. root/destを同じ取得済みcurrent_dir基準で絶対化する前に、入力のnamespace/成分を検査する。通常drive絶対path・UNC・相対pathと通常filesystemを示すverbatim drive/UNC/volume GUIDを扱い、drive-relative、device namespace、GLOBALROOT、NUL文字、drive区切り以外のcolon（ADS）、予約device名を拒否する。予約名は拡張子付きCON/NUL/COM等とWindowsの上付き数字も含む。末尾dot/space等で通常名と別解釈になり得る成分もUnsafePathとする。日本語はOsStr/UTF-16のまま扱い、lossy変換しない。
2. `.`/`..`は既存祖先を順に確認してから整理する。先にcanonicalize/lexical collapseして`junction/..`を消し、入力にあったreparseを見落とさない。root/dest双方のすべての既存祖先とleafを`symlink_metadata`＋FILE_ATTRIBUTE_REPARSE_POINTで検査する。既存祖先はdirectory、file leafは通常fileでなければ拒否する。既存directory宛先は再利用しない。初回の確認済みNotFoundだけを不在とし、取得不明/観測後消失を不在へ変換しない。
3. Windows handleをOPEN_EXISTING、FILE_READ_ATTRIBUTES、FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE、FILE_FLAG_OPEN_REPARSE_POINT（directoryはBACKUP_SEMANTICSも付加）で開く。特権を有効化しない。handleの属性/typeを再検査し、GetFileInformationByHandleEx(FileIdInfo)のvolume serial＋128-bit file IDを同一性とする。GetFileInformationByHandleのNumberOfLinksを使い、既存fileは1以外を拒否する。64-bit file indexへの縮小、metadata取得失敗時の文字比較へのfallbackはしない。handleはRAIIのFile/OwnedHandleで保有し、全終了経路で閉じる。
4. GetFinalPathNameByHandleW(FILE_NAME_NORMALIZED | VOLUME_NAME_GUID)から解決済み親pathを得る。返却長を確認し必要容量で再取得する。元drive表記の祖先だけで止まらず、解決済み親からvolume rootまで各directoryを開いてidentity列を得る。既存rootのidentityがこの列にあれば保護配下として拒否する。これで大小文字・短縮名・同volumeの別drive表記を文字のprefix判定に依存せず処理する。GUID解決またはidentityが得られないfilesystem/共有/driverはIdentityUnavailableで拒否する。UNCへ表記を変えてこの失敗を迂回しない。GUIDのないnetwork shareの対応を完了とは主張しない。
5. rootが不在の場合は、既存部分を同じ手順で開き、その最深祖先identityに未作成成分列を固定する。destの解決済み既存祖先列に同identityを照合してから、未作成名だけをWindows CompareStringOrdinal(TRUE)で成分比較する。これは存在しない名の予定領域比較であり、既存directory identity失敗のfallbackではない。API失敗は拒否、ASCII lowercase/文化依存比較は不可である。case-sensitive directoryでも予定名の大小文字だけの差は保護側へ倒す。短縮名が実在すれば既存handleで判定し、存在しない8.3名を推測生成しない。
6. 両modeとも予定root自身/配下と、予定rootへ続く未作成祖先そのものへの出力を拒否する。例：root=`X:\safe\new\assets`が未作成、dest=`X:\safe\new`をDirectoryで許すとbackupのassets生成が予定rootへ侵入し、Fileで許すと予定directory成分がfileで塞がるためである。既存rootの場合はその祖先が既存directoryなので元の新規条件/型検査で拒否される。root=`X:\safe\new\assets`に対するdest=`X:\safe\other`や、root=`X:\safe\WordWeave5`に対する`WordWeave5-export`は許可する。dest親が不在ならmkdir_allせず拒否する。

metadataの型/reparse判定は既存progress_metadataと同じ規則を再利用する。ただしreaderのStrict/AllowAbsent policyへexport用途を混ぜず、identity/予定root/再検査はexportのprivate helperへ置く。Windows以外は新export入口を未対応エラーにして、既存Storage/atomic_writeの他platform挙動は変更しない。Windows依存の機能追加が本単位の範囲である。

既存Cargo.tomlのwindows 0.56.0はStorage/FileSystem featureを有効化していないが、新依存/feature変更は不要である。`src/assets.rs::sha256`やstore既存fixtureと同じ`#[link(name = "kernel32")] extern "system"`方式をstore内private Windows moduleに置ける。CreateFileW/GetFileInformationByHandleEx/GetFileInformationByHandle/GetFinalPathNameByHandleW/GetFileType/CompareStringOrdinalと必要なrepr(C)型だけを宣言し、構造体size/ABIとunsafeのbuffer・handle寿命を局所化する。FFIが実環境で未対応なら安全判定を緩めず親へ返す。

#### H3. 検査・永続化の順序とエラー

共通順序は、guard取得→prepare_external_destination（副作用なし）→内容はcallerで確定→recheck_external_destination→最初の作成、である。再検査は元pathと固定解決先の双方の祖先/親/leaf、rootまたは予定root既存祖先・不在成分を再観測する。identity/型/link数の差、AbsentからPresent、PresentからAbsent、root出現/移動をChangedとし、同じ操作内で新しい宛先へ再baseしない。再試行は新しい呼出しである。再検査終了後の敵対的TOCTOU完全防止、OS renameの電源断耐久性を追加保証しない。

fileの書込先は固定解決親＋leafである。既存atomic_writeの順序（隣接tempのcreate_new→write_all→sync_all→close→rename、失敗時temp cleanup）を共有するが、tempも保護領域検査の対象にする。到達可能な衝突例は、カスタムroot=`X:\safe\note.tmp-<pid>-<stamp>`が未作成で、dest=`X:\safe\note.txt`の場合である。現在のwith_extensionによるtemp名が予定rootそのものになり得る。日時の一致が稀であることを安全性の根拠にしない。

抽出はstore内private `atomic_temp_path(path: &Path) -> PathBuf`（現行命名をそのまま使用）と`atomic_write_with_temp(path: &Path, bytes: &[u8], temp: &Path) -> Result<(), String>`（現行I/O/cleanup本体）だけとする。公開atomic_writeは両者への委譲であり、gate/安全検査/新エラー分類を追加しない。外部file入口は同じ命名helperでtempを一度選び、同じ解決親であること、temp自身も保護領域外であること、既存tempなしを確認し、root/親/宛先leafの最終再検査を終えてから同じI/O本体へ渡す。直前拒否ではI/O本体/cleanupへ入らず、tempや既存destを作成・削除しない。既存tempの競合を自動削除してやり直さない。外部操作の実I/O開始後の失敗は元atomic契約であり、全bytes不変を事前拒否と同じ範囲で約束しない。

directoryは最終再検査→固定解決pathにcreate_dirを一度行い、成功時そのPathBufを返す。再lockやcreate_dir_allは行わない。作成失敗を成功へ補正しない。後続file/markerの失敗で作成済みexport directoryや先行fileを自動削除せず、完了markerがない途中成果物として残す。sourceのProgress/pending/rootは変更しない。

private `ExternalDestinationError`は`UnsafePath / ProtectedRoot / IdentityUnavailable / Io / Changed / AlreadyExists`を区別する。gateのPoisoned/Reentrantは既存messageを使う。境界で「安全な保存先を確認できない」「保存先が変更された」「別の外部保存先を選ぶ」等の日本語Stringへ変換し、path/本文/原音を含めない。OSエラー文字列のparseで分類しない。最初の固定順序の不合格を返し、分類はprivate試験、公開試験はErrとledgerで確認する。実atomic/create_dirエラーの従来String契約は維持する。どのexternalエラーも正本write-blockを新設/解除せず、pending回復・移行退避・保存済み判定を起動しない。

#### H4. d-B/Mへの接合と同期区間

d-Bはbackup.rs::export_with_limitsの通常gateを`acquire_external_export`へ置換し、その1つのguardを内容snapshot取得前から完了manifest書込まで保持する。`legacy_export_bytes_under_guard`のguard型をExternalExportGuardへ替え、ensure_writableだけを除く。candidate/disk history拒否、read_progress_for_backupによるstrict pending/版/構造/不存在の検査、v1 wire投影、deck/媒体/inventory/manifest上限はそのままである。preserve_current_progress/restore等の内部正本系のProgressWriteGuard/ensure_writableは変更しない。

順序はexternal guard→全内容preflight→create_external_directory_under_guard→返却pathを媒体出力基点に設定→媒体→write_external_file_under_guard(custom.tsv)→同(progress.json)→同(learning-backup.json)最後、である。reject_asset_destinationを除き、public locking directory/file helperを入れ子に呼ばない。媒体AssetStoreの元検査/putと順序は維持し、外側directoryが予定rootの祖先でもないことをH2で保証する。d-Mのannotation一式も同じguard、返却path、各fileのunder_guard、完了marker最後を使用する。d-S/単一媒体fileは公開file wrapperで足りる。

write-block中も安全な外部file退避、およびb-Bの内容条件を満たすbackup exportは成功できるが、pendingやhistory混入の内容拒否を迂回しない。guardは同期snapshot/出力区間だけに所有し、file picker/確認画面、解析、ネットワーク、worker完了待ちを含めない。guardをasync taskへ移動せず、取消callbackやtokenを新設しない。既存callerの取消し/遅延結果契約は維持する。d-H時点ではbackup/媒体のconsumer接続とI2-D2 Closedを主張しない。

#### H5. 独立oracle・受入と引継ぎ

| 受入 / 割当 | 独立作者が固定する入力と観測 |
| --- | --- |
| H01 / T08 / AC04・16 / store境界 | 手製root配下のprogress/custom.tsv/pending/app.lock/backups/assets/audio-historyと日本語sentinelの全file/dir ledgerを先に作る。既存file・新規file・既存root・配下新規directoryを各公開入口へ渡し、Err、既存bytes/dir集合不変、temp増分0を確認する。Progressの候補None/Bを安全判定の入力にしない |
| H02 / T08 / Windows identity | root/dest両方の大小文字、既存成分を含む`.`/`..`、利用可能な8.3/別drive、祖先junction・leaf symlink、外部へのjunctionも拒否、外部hardlinkを独立fixtureにする。links>1は保護root外同士でも拒否する。ADS、device（拡張子/上付き数字）、曖昧末尾、directory-as-file、非既存親を拒否し、通常の日本語名を許す。alias生成成功を別OS handle/fixture情報で確認し、guard判定をoracleにしない |
| H03 / T08・T08b / 予定root | rootの未作成1成分/複数成分、大小文字別表記、既存祖先alias、rootを内包するdirectory宛先拒否と兄弟directory成功を別caseにする。root不在を維持し、parent自動作成なし。解決rootが既定以外でも保護対象が変わらないこと、resolver単独のmkdir/lockなしを確認する。LOCALAPPDATA欠落/変更は子process内だけで扱う |
| H04 / T08b / 通常成功・互換 | 外部新規file、通常1-link file置換、新規directory、似た名前の兄弟、空bytes/日本語bytesを公開入口で成功させexact bytesを確認する。公開atomic_writeの新規/置換/I/O失敗を抽出前契約と比較し、gate保持中の従来atomic_write成功も既存gate回帰で確認する。旧reader/pending/移行/backup bridgeのoracleを変更しない |
| H05 / T08b / gate | store専用試験を有限timeoutの子processで実施する。通常guardのblock_writes後drop→external file/directory成功→通常acquireは引き続きWriteBlocked、poison時は両取得口拒否、通常→externalとexternal→通常/外部の同thread再入は即Reentrant、under_guard連続fileは成功とする。外部guardを保持中の別thread通常writerが開始せず、drop後に完了することをchannelで確認する。解除口/別Mutex/無期限waitを置かない |
| H06 / T08 / 直前拒否 | private prepareでCheckedを得て、fixture側が親directory交換・leaf置換/消失・Absent leaf新設・hardlink追加・root出現を行った後、productionと同じ最終recheck/publishを呼ぶ。変更後ledgerを比較基準とし、export由来temp/bytes変更0を確認する。metadata/identity取得不能は実handle共有制限等で作り、安易なNone化を検出する。最後の再検査からOS作成までの敵対的競争を完全防止したとは扱わない |
| H07 / T08 / temp保護 | private final-publishが受ける固定temp pathを使い、予定rootと同名temp、既存temp sentinel、通常tempの3対照を決定的に作る。禁止tempではI/O本体を呼ばず、root/既存temp/dest不変、通常tempではexact置換後tempなしを確認する。時刻の偶然一致や同一guard実装による期待値再計算へ依存しない |
| d-B/M後続 / T08・T08b / AC16 | 同じexternal guardによるsnapshot→directory→子file→markerの順序、write-block中の適格backup成功、history/pending拒否、途中失敗のmarker欠損とsource不変をconsumer作者が確認する。d-Hのhelper成功をこの証拠で代用しない |

private試験seamの署名は以下とする。Checkedのprivate fieldが`&'g ExternalExportGuard`を保持し、最終操作まで同じguardの生存を要求する。publish/createは対応modeを要求し、不一致はUnsafePathで拒否する。publishのtempは検査前の名前だけを受け、検証済みとして信用しない。

```rust
fn prepare_external_destination<'g>(
    root: &Path, dest: &Path, kind: ExternalDestinationKind,
    guard: &'g ExternalExportGuard,
) -> Result<CheckedExternalDestination<'g>, ExternalDestinationError>;
fn recheck_external_destination(
    checked: &CheckedExternalDestination<'_>,
) -> Result<(), ExternalDestinationError>;
fn publish_external_file(
    checked: &CheckedExternalDestination<'_>, bytes: &[u8], temp: &Path,
) -> Result<(), String>;
fn create_checked_external_directory(
    checked: &CheckedExternalDestination<'_>,
) -> Result<PathBuf, String>;
```

productionとtestでprepare/recheck/最終作成の安全判定を複製しない。testがfake identityや合格boolを注入する口は不要である。H03はFileで予定rootの未作成祖先を塞ぐ拒否も含む。上表のcase数/名称は独立作者が固定する。昇格が必要なaliasは親へ返し、作成不能/未実行を成功件数へ混ぜない。既存の特定symlink試験への承認を新fixtureへ転用しない。

設計Pre後、作者は新integration＋store専用節で期待値を先に固定する。実装役は本体だけを変更し、runnerは`cargo test --locked --test progress_export_destination`、`cargo test --locked --lib store::i2_export_destination_tests`、`cargo test --locked --test audio_history_gate`およびgate取得/atomic抽出の影響する既存store・migration・save・material・backup bridge回帰を固定source/EXE hashと実在件数で記録する。通過済み全体suiteを無条件に反復せず、影響caseは作者/親が既存受入から選ぶ。format/diff・独立Post・I10全体gateは維持する。

判断根拠は[FILE_ID_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info)（volumeと128-bit identity）、[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)（既存handle/共有/reparse flags）、[GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)（GUID/normalized pathと非対応）、[CompareStringOrdinal](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-comparestringordinal)（予定名だけのOS比較）である。型付き内部境界と加算的APIはapi-and-interface-design Skillに基づく。REST/新dependencyは不要である。

未実測はOS/FSのidentity取得可否、alias作成権限、実I/O競争であり、実装/独立試験で証拠を得る。APIの不足や新file/依存が判明すれば親へ戻す。現段階で承認仕様の変更は不要と判断するが、GUID解決不能環境の拒否、予定rootを内包するdirectory拒否、atomic private抽出は独立Preで明示確認する。UI描画の変更はなく、日本語配置・両軸button中央・縮尺・狭幅設計とUI Skillは本単位では非該当である。本追補の完了は文書差分/参照検査とhandoffであり、build/test/実データ/API/公開は実施しない。

## 6. consumer・非同期・取消し

| consumer | 必要な反映 |
| --- | --- |
| app.rs persist | save_progress返却値をUI正本へ反映してからdirty解除。未保存fieldは渡した候補のまま、rootは実保存値となる |
| app.rs import_deck / restore | 教材はcommit::save_progress、Progress復元はStorage::save_progressの返却値を採用する。pending時に旧全体snapshotを保存して巻き戻さない |
| settings_edit / color_theme | 現在のfield mergeを維持し、返却Progressを採用する。draft/previewを勝手に確定しない |
| settings_ui（JSON/TSV/診断）、control_settings（診断JSONL） | 利用者選択fileへの直接atomic_writeを共通write_external_fileへ接続する。外部JSONは既存メモリsnapshot/未保存draftを保持し、正本rootの更新として扱わない |
| chat_ui / chat_media / conversation_trash | 同じ返却値採用。録音/筆跡の失敗時payload保持、削除guard、元媒体は変更しない |
| chat_mediaの未保存音声/注釈text/注釈一式export | fileと新規directory各モードの宛先検査を接続する。拒否時はpayload/annotation状態を保持する |
| backup.rs / backup_ui | Prepared::restoreが実保存Progressを返すのでbackup_uiの代入形は維持できる。backup::exportは共通Directory検査を必須とし、既存root/保護配下/aliasを拒否する。backup UIの文言/画面はI3/I8で確認する |
| I6 history controller | workerはFrozenHistoryRequest/結果だけを返す。最新UI非履歴fieldsは保持し、commit確認後rootだけを採用。画面を閉じてもhost寿命を維持する |

取消しはI6が所有するAtomicBoolを借り、gate取得後のpointer直前まで照合する。commit前取消しはpointerを変えず、原音/object/intentを回復候補として残す。最後の照合とatomic replaceの競合は再読で決着するため、その区間では取消成功を先に表示しない。commit済みなら原音と結果を残す。Storageはworker/tokenの寿命を所有せず、同期区間へcancel callbackを呼び戻さない。取消しをIoエラーや未確定Conflictに混同しない。遅延結果のgeneration/Active照合と停止終端→trash順はI1/I6契約を維持する。本文/原音/path/秘密を新しいerrorに含めない。

## 7. fault seamと独立受入

I1のHistoryCheckpointは変更しない。I2は固定Ioだけを返す`ProgressFaults::check(ProgressCheckpoint)`と通常NoProgressFaultsを追加し、検証内容や返すrootを差し替える能力を持たせない。利用箇所は実装所有のstore/commitに限定する。test用with_faults入口は既存production入口と同じvalidate/gateを通り、通常APIは変更しない。

CheckpointはBeforeMigrationBackup/AfterMigrationBackup、BeforePointer/AfterPointer、BeforeMaterialIntent/AfterMaterialIntent、BeforeDeck/AfterDeck、BeforeMaterialProgress/AfterMaterialProgress、BeforeMaterialFinish/AfterMaterialFinishとする。AfterPointerはatomic replace後でcaller確認前のエラー、material finishはpending退避の前後である。fault注入後にもpointer/両metadataの実再読を行う。再読不能は別の隔離filesystem fixtureで検証する。kill残存状態fixtureと通常Err注入を分け、電源断耐久性と同一視しない。

Mは [限定設計§2/3/5](history-migration-design.md) のguard付きhelper、typed MigrationError/ProgressFaultErrorと移行2checkpointだけを接続する。target=1では移行faultも呼ばない。AfterMigrationBackupは退避のexact再読確認後・次のmetadata操作前であり、そのErrでは正しい退避だけが残る。rename結果不明は別のprivate試験口で、実際に同path/同bytesをatomic_writeした後にErrを返し、helper自身の再読で判定する。read結果や検証値を差し替える口にしない。M05のv1縮小成功/target=2上限拒否対照、S-Mの両公開保存入口、R-Mの手製pending・backup欠損・current=old/new・既存退避不一致を各担当へ引き継ぐ。

Mの試験作者向け追加deltaとして、store.rsのprivate試験口を次に固定する。限定設計のcrate-private `preserve_v1_before_v2_under_guard` は薄いwrapperとし、productionの`atomic_write`をpublishへ渡す。検証・path確認・exact再読の本体はこのprivate helperだけに置き、production/testで分岐複製しない。publishは新規退避が必要な場合に同じ検証済みpath/bytesで最大1回呼び、既存一致やtarget=1では呼ばない。

```rust
fn preserve_v1_before_v2_with_publisher<P>(
    root: &Path,
    old_bytes: Option<&[u8]>,
    next_wire_version: u32,
    limit: u64,
    guard: &ProgressWriteGuard,
    faults: &dyn ProgressFaults,
    publish: P,
) -> Result<(), MigrationError>
where
    P: FnOnce(&Path, &[u8]) -> Result<(), String>;

pub enum ProgressFaultError { Io }
pub struct NoProgressFaults;
impl ProgressFaults for NoProgressFaults {
    fn check(&self, _: ProgressCheckpoint) -> Result<(), ProgressFaultError> {
        Ok(())
    }
}
```

ProgressFaultsとProgressCheckpointはpubであり、Checkpoint名は上記既定の12variantを保持する。private unit試験だけが `|path, bytes| { atomic_write(path, bytes)?; Err("synthetic post-rename failure".into()) }` 等を渡す。Err本文を分類に使用せず、helper自身の再読結果から保護済みか判断する。公開のfilesystem差替えAPIは追加しない。

既存`Storage::save_with_limit(&self, p: &Progress, limit: u64) -> Result<(), String>`は署名を保ち、新private `save_with_limit_and_faults(&self, p: &Progress, limit: u64, faults: &dyn ProgressFaults) -> Result<(), String>`へNoProgressFaultsを渡す。commit側も既存save_with_limitsの署名を保ち、同じroot/deck/progress/limits引数の末尾にfaultsを加えたprivate `save_with_limits_and_faults`へ委譲する。gate取得は各and_faults入口の1回だけとし、元wrapperで二重取得しない。Gのunder_guard接続・ensure_writableを維持する。Mでは移行2checkpoint以外を実装せず、公開v2受付やrecover fault接合はS/Rへ残す。この追加deltaも統合後最終Preの対象である。

SのRED前scaffoldは [限定設計§4](history-progress-design.md) に限定する。新audio_history fieldには一時的なserde(skip)を付け、Default1・既存derive/validate・M/G入口を維持したまま、新private型/固定Err helperとhead/read口だけを追加する。改訂前M/Gの全試験不変と成功を確認した後、独立作者が指定oracleを改訂して行動REDを取る。RED後にskip除去・実codec・manual Deserialize・Default2/read/head/M接合を完成し、既存公開Deserializeをstubへつなぐ中間状態を作らない。

S-Mは通常/新教材の公開save成功によるwire2・旧v1 exact退避を確認し、既存private faults口ではBefore→After→次metadata操作、Before失敗で全ledger不変、After失敗で確認済退避だけ残存を確認する。M段階の空seen/退避なし成功を維持せず、wire1 pending exact replayは維持する。Sの最大4fileはstore本体・新progress integration・既存migration integration・commitのM専用test節だけであり、commit本体/旧test/G節は不変である。改訂するMケースと不変のG/旧testの証拠を分け、S設計Pre前に本体へ着手しない。

NはBeforePointer/AfterPointerを発火せず、Pの履歴pointer専用として残す。通常保存は既存Mの移行2faultだけを引き継ぎ、private save_progress_with_publisherへproduction atomic_writeを渡す。独立試験では同じpath/bytesの実書込後Err、書込なしOk/Err、実第三bytesを注入し、読取値や検証結果は差し替えない。詳細署名・N01–06/N03b oracleは [N限定設計§4/5](history-normal-save-design.md) を参照する。特にN03bは日別leaf拒否時の先行hash退避残存を検証する。Nはstore本体/専用cfg(test)と新audio_history_saveの2file案であり、S/M既存oracleを無断改訂しない。S最終Post後の統合であるがN最終Preは未了、本体未着手である。

Nの日別I/O検証deltaでは、通常単publisher署名を保って最内側private with_publishersへ委譲し、daily helperのmetadata検査→必要directory作成→直前確認→日別publisherと通常保存本体をproduction/testで共用する。gate取得は最内側1回だけであり、公開API/checkpoint/所有fileを増やさない。N03bの検査拒否とは別に、N03cは検査後のdaily callback内で合成destをshare0保持し、同atomic_writeの実I/O失敗が通常保存へ伝播して正本publisher0となることを確認する。M確認済退避/作成directory、fixture sentinel/probe、atomic tempをledgerで区別し、表示StringからOS分類を復元しない。最小追加署名・probeの独立証拠・失敗時残存は [N限定設計§1/2/4](history-normal-save-design.md) に固定する。本deltaの独立Pre前にsource/scaffold/作者試験を新署名へ接続しない。

Rは既存recover/under_guard署名を保ち、新private recover_with_limits_and_faults_under_guard→recover_with_io_under_guardへ接続する。後者は自分でpendingを読み全事前検査を行い、production atomic_write/renameを渡す同一本体とする。具体的なFnMut write/FnOnce archive署名は [R限定設計§1](history-material-recovery-design.md) を正とし、読取値や手製「検証済Intent」を注入する口を作らない。gateは既存外側入口の1回、再入なしである。RはBefore/AfterDeck・MaterialProgress・MaterialFinishを接続し、MaterialIntentはT、PointerはPに残す。既存save側からのrecoverはNoProgressFaults wrapperを維持し、T前にM/Sのfault観測順序を広げない。

R01–06の独立oracleは [限定設計§5](history-material-recovery-design.md) による。旧wire1の3中断位置とexact archive、fake intentのA→None/B・旧不正、current既newでのintent旧v1退避、同名backup競合、各書込/rename後Err、known中断とunknownの子process隔離を検証する。既存M/S/N/G oracleを無断改訂せず、最大3fileはcommit本体/R専用cfg(test)、storeのcodec bridge、新audio_history_materialだけである。N最終Post完了後の限定統合であり、R本体は未着手、独立最終Preは別に必要である。

TはBefore/AfterMaterialIntentを追加し、Rへ同じfaultsを伝播する。旧wire1→新wire2の成功seenはT-M2件→MaterialIntent2件→R-M2件→RのDeck/MaterialProgress/MaterialFinish各2件の厳密12件、旧wire2/旧fileなしはMを除く8件である。Pointerは含めず、R独立再検証のMを省略/隠蔽しない。private save_progress_with_intent_publisherの最小署名、実pending write後Err/第三状態、R成功後のT最終確認、T01–06 oracleは [T限定設計§1/3–5](history-material-save-design.md) による。Rのmetadata/archive本体や試験口は複製しない。

plannerのT-M限定割当はcommitの `m05_v2_material_private_entry_matches_public_save_with_exact_migration` 成功seen配列1箇所だけを独立M作者が2→12へ改訂するものである。他assert/fixture・旧M fault/limits・旧100件/R25件のoracleは維持し、旧2件の証拠と新配列の行動RED/全後段到達を分ける。最大3file・同file逐次所有を保ち、R最終Post固定実物のcodec/read/under_guard署名との一致を確認済みである。本統合後のT独立最終Preは未了、T source/testは未着手である。

V01–06はstrict intent/公開token束縛、自己整合偽next、計画object順序・exact mutation、completeでも実object欠損、guard/head、既commitとの権限分離を独立fixtureで確認する。V07–09は新confirmのdisk head next/後続root×intentあり/cleanup済み、base receiptなしBusyと別head Conflict、不正receipt/残存intent/局所破損/pending、既存公開I1との値/error同値、異なるcustom head・小quota超の既存closure・個別上限/error順序の互換を確認する。guard保持中の再lockなしとwrite-block下の両confirm純粋readは既定子process隔離で検証する。全file/dir ledger不変、reset口なしであり、詳しいoracleは [V限定設計§5](history-publication-authority-design.md) を正とする。

plannerのV-P追補によりconfirm共有抽出はVのaudio_history/store.rs本体/専用cfg(test)に割当済みである。必要時だけ既定recovery.rs接合と既存audio_history_recovery testを使う最大3fileを維持し、Pの2file所有を広げない。現案はrecovery.rs変更不要であり、既存confirm/custom head/finish/rebase/quota/w03/w08等の期待値は変更しない。新口のscaffoldが必要でも既存confirmを未実装口へ接続せず、新caseの行動REDと既存回帰を分ける。T最終Post完了後の限定統合であるがV独立最終Preは未了、V sourceは未着手である。V Post後にPが実helperを照合して正本統合・独立最終Preへ進み、V成功をP/UI完了へ拡張しない。

P01–08のoracle/private署名は [限定設計§1/4](history-pointer-commit-design.md) に固定する。P02はBusyを生むRunning/Sending対象SetTrash intentがV再検証でも拒否されpublisher0となる対照を含む。旧wire1成功seenはMigration2→Pointer2、旧wire2/NoneはPointer2、既commitは0である。private publisherは実atomic後Err/no-write OkErr/第三bytesを注入し、read結果やV権限を差し替えない。Before/After fault、取消前後、cleanup/後続root再確認、M退避残存、gate待機中に更新された非履歴field保持を検証する。結果不明/write-blockは子processで隔離しpure read継続と全writer停止を確認する。合成fixtureの改竄物とproduction残存を分け、電源断耐久性と同一視しない。

P所有はstore.rs本体/専用cfg(test)と新tests/audio_history_commit.rsの2fileである。完成Vのverify/confirm実helperとの署名一致と追加gate不要を確認済みであり、I1/V/T/G/M/S/N/Rの本体・既存oracleを変更しない。新enum/3入口だけの未接続scaffoldと行動REDを分け、既存入口をstub化しない。V最終Post PASS後の限定統合であるがP独立最終Preは未了、P sourceは未着手である。Vの未測定head/intent再読中置換・pending metadata Io・実closure100k nodesやUI/I10をP統合だけで完了と扱わない。

| ID / 受入 | 独立fixture・oracle |
| --- | --- |
| I2-T01 / AC04/16 | 手製v1に全既存field sentinel、既存媒体・sourceを置く。loadでbytes不変、初回保存でv2、元v1退避exact一致。日本語/省略legacy field/未知版/未知field/v2 root欠落を個別化する |
| I2-T02 / AC04 | disk root=A、caller clone root=None/旧root、設定変更。通常saveと全consumerでdisk/UI root=A、非履歴変更だけ反映。未知/壊れたAは変更0。read_currentがpending recoverを起動しない |
| I2-T03 / AC04/08 | 手製closure/receiptとPublishedでpointer前後fault。旧root→未確定、next→Committed、第三状態/再読不能→write-block。cleanup失敗後もSaved、同operation再試行でID/sequence/receipt数不増 |
| I2-T03b / AC04/16 | 公開Publishedを手組みし、既存Trial/receiptを除いた自己整合next、別intent/request/base、計画object差替えを与える。closure自体が有効でもintent再計算不一致でpointer変更0。既commit/intent cleanup済みの再確認だけ成功する |
| I2-T04 / AC04/09 | 同process複数threadのnormal/material/history writerをbarrierで競合させる。同root表現違いでも区間不重複、別root内容不混入、再入停止なし。pending I/O不明は拒否。キャンセル前/commit後は各々未保存/保存済みで原音不変 |
| I2-T05 / AC04/16 | pendingなし/intentのみ/deckだけnew/両new/finish後を手製bytesで再構成する。全closure検証前のmetadata変更0、旧pending v1 recover→次保存移行、外部編集と局所欠損拒否、pending保持、再度recoverの冪等性 |
| I2-T05b / AC04/16 | 手製教材intentで旧root=A→new None、A→健全B、old_progress=None→Some、旧Progress破損/未知版を個別化する。現在fileをold/new各中断位置へ配置しても、拒否時はdeck/progress/pendingのexact bytesが全て不変、追加backup/tempなし。正例はA→同A、None→None、v1 None→v2 None |
| I2-T06 / AC16 | historyなしv1 export/prepare/restoreの既存回帰、history有り現rootへv1 restoreで保持、draft/ai_calls保護。history有りv1 exportは宛先作成0。root混入v1/未知版は拒否 |
| I2-T07 / AC08/09 | I6接合spyでは保存だけ再試行で解析/POST増分0、再起動から自動実行0。I2単体はprocess/transport依存なしを確認し、UI表示や実通信受入を代替しない |
| I2-T08 / AC04/16 | disk root=A・UI古いNone/BのJSON書出し先をprogress.json、custom.tsv、pending、backups、assets/audio-history配下へ指定し拒否する。settings TSV/診断・control_settings診断・chat媒体・backup exportにも同じ保護を適用する。大小文字/`.`/`..`、利用可能な別drive/短縮名、親junction/leaf symlink、外部hardlinkを別fixtureにし、拒否で保護bytes/既存宛先/未保存payload不変、tempなし。directory dest=既存rootと保護配下新規dirの両方を確認する |
| I2-T08b / AC04/16 | 通常外部JSON/TSV/診断/媒体と媒体backupが成功し既存内容契約を維持する。似た名前の外部dir、通常の外部file置換、write-block中の外部JSON退避、Storage取得失敗時のroot解決、検査I/O不明の拒否を検証する。手製の事前全file ledgerで比較し、path guard自身の判定をoracleにしない |

root/Trial/旧revision/原音/resultの期待hash集合はtest作者が手製で作る。writerの出力やquota_planを期待値に転用しない。全既存bytesとmetadata ledgerをfault前後に比較する。通常保存の100MB/教材64MB/intent350MBと既存のexact serialized境界を維持する。小さい上限seamがある既存unit testを大幅改変せず、独立integration testを追加する。

path alias試験の権限不足/非対応は未実行と記録し、通常suite成功へ混ぜない。symlink等の作成に新たな昇格が必要なら親へ戻し、既存I1の特定1件だけの昇格承認を流用しない。本設計修正はOS設定変更や昇格を許可しない。

### I2b-A接合確認（2026-10-11）

P最終Post後、`main_consumer_design`（指定Astra/high、read-only）が既存consumerと完成した保存APIを照合した。新API・新workerは不要である。`src/app.rs` の `persist`、`import_deck`、`restore` だけを対象とし、それぞれ `Storage::save_progress`、`commit::save_progress`、`Storage::save_progress` の成功返却値をUIの `Progress` として採用する。rootだけの先行読取・手動mergeは行わない。成功フラグ・dirty解除は返却値採用後とし、失敗時に古い画面snapshotをdiskへ保存し直さない。

I2-T02の「未知/壊れたAは変更0」は、b-Aでは正本 `progress.json`、教材、pending、履歴objectsの不変とUIへのcandidate未採用を指す。復元前に現在の未保存draftを別snapshotへ退避する既存保護は維持し、その補助退避の追加を許容する。これは復元後UIに古いdraftをmergeする仕様ではない。退避helper内の最新root反映は予定b-Bの所有であり、b-Aに含めない。教材登録失敗時の既存メモリ復元をdisk rollbackと誤認して削除しない。

検証は3経路×stale None/Bの成功、壊れた/未知rootとpendingの拒否、教材pending作成後の保存拒否での旧snapshot非書戻しを対象とする。`backups`をfileにするfixtureは現recoverの退避先事前確認で停止するため、両metadata書込後のfinish失敗と称さない。後半障害注入用seamは追加せず、保存層R/Tの既存証拠とconsumerの実到達境界を区別する。restoreは復元candidate採用、未保存draft別退避、当日生成回数最大値、既存chat resetを維持する。独立作者は `src/app.rs` 内の専用 `i2_history_main_consumer` のみを追加し、作者のRED確認後に3関数本体の所有を実装者へ移す。既存restore/partial-save回帰と新focused試験を実行する。独立runnerは既定Sol/mediumの別工程であり、I10全体gate・製品EXE提供とは区別する。

### I2b-B接合確認・実装handoff（2026-10-11）

入力は承認済みAH-AC16／I2-T06、implementation.mdのb-B行と7役割当、b-A最終Post PASS/Closedである。担当指定は`gpt-6-astra/high`、実行metadata・usage counterは未取得である。照合HEADは`6dd82855f0b42690b5c20315b2335a7fabe2e3cb`。本節だけを設計担当が所有し、本体・test・gateの変更を許可しない。受入単位は次の3条件であり、公開署名`export(&Path, &Path, &[Entry], &Progress)`と`Prepared::restore`を維持する。

| 条件／受入 | 所有・データ契約 | 独立受入 |
| --- | --- | --- |
| B1／T06・AC16 | `preserve_current_progress`は最新disk rootだけをcurrent cloneへ反映し、非履歴draft・ai_calls等はcurrentから保持する。`Prepared::restore`はbackup candidateへ日別ai_callsのcurrentとの最大値を反映し、`commit::save_progress`の実保存返却値を返す | 手製disk A＋current None/B、異なるbackup draft/current未保存日本語draft、複数日の大小逆転ai_calls。復元後はbackup draft、別memory退避はcurrent draft、両方のrootは各保存区間のdisk A。返却値と正本を比較する |
| B2／T06・AC16 | v1 exportはcandidate／最新diskのどちらかがSomeなら宛先作成前に拒否する。両方Noneだけ明示wire1・`audio_history`キーなしとして出力する | candidateだけSome／diskだけSome／両方Noneを別caseとする。拒否時destなし、source全ledger不変。成功時は手製legacy oracleと全非履歴field・manifest hashを照合しprepare/restoreまで往復する |
| B3／T06・AC16 | prepareは既存のhash／媒体照合とstrict `Progress` Deserializeを維持し、manifest1＋非null rootと未知manifestを拒否する | 正しい再計算manifest hash付きの混入root、未知版、重複version/rootキー、未知field、壊れたJSONを各別fixtureにする。単なるhash不一致による拒否をschema拒否の証拠にしない |

現物SHA-256は`src/backup.rs=193D347A4D1A8C97612D4F283BFDD0F78492AAFE1DDA3EE31E3BD694392E0F16`、`src/store.rs=FA9ECA988D4EDF5233E0198F8B2E090F760098D16779001179CBFFF151F38BEB`、`src/commit.rs=14952768353B2111D7AF9C96541B8F321AC79BDD30FE26CF9C41A3EA188E0154`である。初案2fileではPath読取を共有できなかったため、plannerがimplementation.mdのb-B限定計画deltaで本体`src/backup.rs`、`src/store.rs`の限定bridge／共有読取接合、新`tests/audio_history_backup.rs`の最大3fileへ確定し、親から承認を受領した。両本体の専用cfg(test)は独立作者が先に所有する。本節の反映後も独立Preは未了である。

#### 既存APIの再利用と不足

`Storage::read_progress(&self)`は安全な祖先／leaf検査、100MB bounded read、strict版dispatchを共有し、復旧もgate取得も行わない。内部memory退避にはこれをそのまま使える。`HistoryReader::new(storage).verify_closure(root)`も読取専用であり、guard保持中に呼べる。`commit::pending_checked`はmissing leafだけをfalseとし、I/O不明・不正pathをErrとする。`commit::save_progress`は自分でgateを取り最新rootを再読し、実保存値を返す。

一方、外部exportの入力は`&Path`だけである。`store::read_progress_bytes/read_progress_typed`と`commit::read_recovery_bytes`はprivateであり、既存のcrate-private `decode_progress_for_material(bytes)`は安全なfile読取を提供しない。`ProgressHistoryHead`も`&Storage`が必要である。`Storage::at(root)`による再取得はapp.lock競合とmkdirを起こすため使わない。backupの既存`read`はmanifest等のbounded読取であり、稼働rootの祖先/reparse安全検査を備えない。ここへ別のProgress readerを複製する案も不採用とする。

確定するcrate-private入口は`pub(crate) fn read_progress_for_backup(root: &Path) -> Result<Option<Progress>, String>`である。store内にprivate `enum ProgressRootPolicy { RequireExisting, AllowAbsent }`と`fn progress_root_exists(absolute_root: &Path, policy: ProgressRootPolicy) -> Result<bool, ProgressReadError>`を置き、現`read_progress_bytes`の祖先走査だけを共有helperへ抽出する。helperの入力はcallerが`std::path::absolute`で得たpathに限り、ルートから最終rootまで祖先を上から順に既存`progress_metadata(path, true)`で検査する。全て通常directoryならtrue、最初のNotFoundはAllowAbsentだけfalse、RequireExistingなら従来どおりIoである。途中のfile／symlink／reparseはUnsafePath、NotFound以外のmetadata失敗はIoで停止し、下流が存在しないことを理由に先行異常を無視しない。`exists()`やcanonicalize失敗を不存在へ読み替えない。

bridgeの順序はabsolute解決→上記AllowAbsent→falseなら`Ok(None)`→既存rootなら`pending_checked`のErr/trueを拒否→既存`read_progress_typed(root, PROGRESS_LIMIT)`→runtime Progressへmapである。pendingを解消せず、gate／app.lock取得・mkdir／backup／recover／書込を一切行わない。存在しないrootへpending_checkedを呼ばない。通常`read_progress_bytes`はRequireExistingで同じ祖先helperを呼び、その後のleaf metadata／長さ確認／File::open／limit+1読取をそのまま保つ。typed decodeと既存ProgressReadErrorの型・分類は変更しない。bridge後半のstrict readによる祖先再検査は、初回確認後にrootが消失した場合をNoneへ落とさないために維持する。検証済み既存rootのprogress leafなしはNone、観測済みleafがopen時に消えた場合はIoであり、この区別も維持する。Storage／head／通常writerは全て既存の厳格reader入口を継続し、AllowAbsentを流用しない。

さらに`acquire_progress_write()`は取得内部で`ensure_writable()`を呼ぶため、write-block中に外部退避だけを直列化する取得口は現物にない。内部memory退避は稼働rootのbackupsへの書込みなのでwrite-blockで拒否する。外部exportは§5の外部退避契約に従い、write-blockを解除せず利用できる必要がある。これは既定I2d-Hの外部宛先/gate接合へ渡す不足であり、b-Bでglobal flagを読んだり別Mutexを作ったりしない。最終的には外部exportの内容snapshot取得と宛先作成が同じ外部guard区間を通る必要がある。新たなguardを入れ子に取るdirectory helperの呼出しは不可であり、I2d-H/Bの所有でunder-guard接合を確定する。b-B時点の通常write gate利用を完成したwrite-block時外部exportとして報告しない。

#### backup内の最小helperと順序

1. `fn current_snapshot_under_guard(storage: &Storage, current: &Progress, guard: &ProgressWriteGuard) -> Result<Progress, String>`はguardのwrite可否→pending_checked→read_progress→Some rootのclosure検証→current.cloneのroot置換→validateを行う。root leafなしはNoneを採用し、古いcurrent rootを復活させない。pending、未知版、不正root、局所欠損、read不明は退避directory作成前に拒否する。callerのcurrentは変更しない。
2. `preserve_current_progress`だけがこの区間のgateを1回取得し、上記helper→既存`progress_bytes`→memory退避作成／atomic_writeまで保持する。`progress_bytes`はruntime2用のままとし、`audio_history`を消さない。guardを解放してから返る。commit/recover/load/exportをこのguard内から呼ばない。
3. `fn legacy_progress_bytes(progress: &Progress, limit: u64) -> Result<Vec<u8>, String>`をexport専用に置く。runtime validateとhistory None確認後、既に型付けされたProgressをSerializeしたobjectだけにversion=1／historyキー削除を施し、pretty bytesの実長で上限を判定する。raw inputを`Value`へ先行decodeして重複キーを潰す経路にはしない。全非履歴fieldを保持し、型を丸ごと複製したlegacy structは作らない。manifestのhashはこの最終bytesを対象とする。
4. 外部exportの内容検査helperは`fn legacy_export_bytes_under_guard(root: &Path, candidate: &Progress, limit: u64, guard: &ProgressWriteGuard) -> Result<Vec<u8>, String>`程度に限定する。b-Bの暫定接合ではexport_with_limitsが通常gateを1回取得し完了markerまで保持する。候補Someは拒否し、上記bridgeでpendingなし・安全な最新disk読取を確認してdisk Someも拒否する。helperはgateを再取得しない。disk Someはclosureを走査してもv1 exportできないため、rootを消して続行せず未対応で停止する。両方Noneならlegacy_progress_bytesへ進む。deck／参照／inventory／manifest上限を全て検査した後にdestを新規作成し、媒体→deck→progress→完了manifest最後の順を維持する。exportからrecoverは起動しない。外部guardへの切替は前述I2d-H/Bの未解決接合である。
5. prepareはmanifest版→元bytesのhash→strict `serde_json::from_slice::<Progress>`→history Noneの明示条件→既存媒体検証の順である。manifest1にruntime wire2・null rootが入った既存互換入力を新たに拒否しないが、非null rootはwire版にかかわらず拒否する。exportが新しく生成するwireは常に1とする。
6. restoreはsource媒体の再検証→current memory退避→不変媒体のtargetへの追加→参照検証→`commit::save_progress`→実保存Progress返却を維持する。restore自身は外側gateを保持しない。退避とcommitは別のgate区間であり、間に履歴が更新された場合は各時点の最新rootを使う。退避で読んだrootを最終commitへ強制しない。旧current draftを復元後UIへmergeせず、別退避で保護する。ai_callsは既存どおりbackup/currentの各日最大値であり、diskの非履歴field全体を混ぜる変更はしない。

新しい非同期処理、取消token、公開fault APIは不要である。確認dialogの取消しはrestore呼出し前の既存UIで完結する。restore開始後の中断・結果不明は完成済みcommitのpending／write-block／実再読契約に委ね、旧snapshotの書戻しや自動再送を追加しない。source媒体再検証／memory事前検査での拒否は正本と媒体を不変にする。memory write後や媒体追加後、commit失敗では先行memory退避・不変媒体・既定commitの移行退避／pendingが残り得るため全ledger不変を要求しない。残存を「復元成功」とせず、既存UIへErrを返す。原音削除・既存backupの削除は行わない。

#### 独立作者／実装者への引継ぎ

作者はB1–3に加え、progress leafなしとroot自体なし、pendingあり／metadata不明、未知版・破損disk、closure欠損、memory退避先がfile、媒体再検証失敗を合成データだけで用意する。事前拒否は全既存bytesとdest未作成、後半失敗は正本不変または既定pending状態と許される先行残存を別oracleとする。返却値／disk／memory退避の3者と、caller currentの不変を検証する。guard再入停止なしは有限timeoutの子processで確認し、write-block試験は解除口を追加せず子process隔離とする。外部write-block成功はI2d-H/B未実装なら未検証として残す。

storeの新`cfg(test) mod i2_backup_path_tests`には最小で次の5群を置く。(R1) 手製v1／v2 None／v2 Someの全field・root保持と全ledger不変、(R2) root直下／複数段の最初の不存在はbridge Noneかつstrict reader Io、既存rootでprogress leafなしは両方None、(R3) 不存在leafより上のfile・junction/reparse、progress leafのdirectory等を拒否し不在扱いしない、(R4) pendingあり／pending不正pathはbridgeだけ拒否しstrict pure readは既存どおり読める、(R5) 未知版・不正JSON・重複キー・上限超過・実open/read不明をbridgeも拒否する。小上限exact/exact-1は既存`s03_typed_read_has_exact_limit_and_only_verified_missing_leaf_is_none`、実read拒否は`s03_real_read_denial_is_io_for_typed_read_public_read_and_head`のfixtureを参照し、新backup用filesystem差替えAPIを増やさない。観測済みleafのopen前消失など実fixtureで起こせない競合／metadata I/Oは静的確認と未測定を分ける。全群でroot/dest/temp新設なしを確認し、既存Storage/headのroot不存在拒否も回帰する。

`backup::tests::rejects_unrestorable_sizes_before_creating_destination`は現状runtime2のSerialize長をv1 export上限に使っている。独立作者がこの1箇所のoracleを明示wire1・historyキーなしの独立投影へ改訂し、`exact-1`拒否／`exact`成功を保つ。production helperや先行exportの実測長を唯一の期待値にしない。`memory_snapshot_rejects_invalid_or_oversized_progress`のruntime2 oracleは変更しない。default100MB／deck64MB／manifest8MB、JSON escapingを含む最終bytes長の判定を維持する。

実在回帰は`--lib backup::`全5件、`--lib store::i2_progress_tests`、`--test audio_history_progress`、`--bin wordweave5 backup_confirmation_`、`--bin wordweave5 json_restore_preserves_the_latest_unsaved_chat_draft`、b-Aの`i2_history_main_consumer`である。新target `audio_history_backup`と新module `store::i2_backup_path_tests`は作者作成後に`--list`で固定しRED→実装→GREENを独立記録する。構造／署名追加のscaffoldと行動REDを区別し、所有は作者のcfg(test)/integration→実装者のbackup本体と限定store接合へ逐次移す。親が独立Pre/Post、runner、I10全体test/releaseを扱う。本設計追記だけで各gateを通過済みとしない。

Path読取bridgeとroot不存在互換の配置・最大3file所有は本追補で確定し、独立Preへ渡す。後続未解決はI2d-H/Bの外部guard接合である。UI描画変更はなく、UI skill／日本語配置・両軸中央揃え・縮尺・狭幅の新設計は非該当である。日本語draftの保存往復はB1のデータ受入に含む。I3のhistory backup v2/union、I8の利用可能UI、全体test/release／製品EXE提供のplanner確定gateを維持する。

## 8. 分割提案とhandoff

以下は親/plannerへ返す所有提案であり、dispatch許可ではない。各行最大5file（独立testを含む）、すべて逐次である。I2担当model/effortと独立Pre/Post、I10全体gateを変更しない。同file内cfg(test)の編集は作者→実装者の所有切替後に行う。

| 単位 | 本体所有候補 | 独立test所有 / 終了条件 |
| --- | --- | --- |
| I2a-0 reader接合 | audio_history.rs、audio_history/closure.rs | tests/audio_history_closure.rs。内部root入口と公開API互換、reader回帰。歴史writer/復旧は触らない |
| I2a-1 schema・head・ゲート | store.rs、commit.rs、audio_history/store.rs、必要なprivate再計算接合のみaudio_history/recovery.rs | tests/audio_history_commit.rs。v1/v2 codec、head、通常保存/教材commit/recover、pointer/intent束縛とfault。計5本以内でも責務が重いため独立レビューでさらに分割可能 |
| I2b-1 主要consumer・v1 backup | app.rs、backup.rs | tests/audio_history_commit.rs と必要なら新tests/audio_history_backup.rs。実保存値反映、旧backup保持、migration/backup合成受入 |
| I2b-1e 外部宛先保護 | store.rs、backup.rs、app/settings_ui.rs、app/control_settings.rs | 新tests/progress_export_destination.rs。共通file/directory宛先検査、root解決、settingsの3出口/診断JSONL/backupの接続。計5本。UI click seamが必要なら同file cfg(test)を作者→実装者へ逐次移譲し、6本目は追加しない |
| I2b-2 設定・配色・会話 | app/settings_edit.rs、app/color_theme.rs、app/conversation_trash.rs | app配下の既存cfg(test)を逐次作者へ移譲。clone stale root回帰。追加test module/exportが必要なら所有確定を親へ返す |
| I2b-3 chat結果・媒体 | app/chat_ui.rs、app/chat_media.rs | 同fileの既存cfg(test)を逐次移譲。保存結果反映、未保存audio/ink保持、I2b-1e共通helperへ3媒体exportを接続。最大2本 |

実装前の独立設計レビュー依頼は、(1) 加算APIでroot更新権限が分離されるか、(2) gate再入と結果不明block、(3) 教材pending旧新root同一性と移行bytes、(4) v1 backup互換と外部export全consumerの取りこぼし/alias境界、(5) I1完成版との整合、を対象とする。特にI1b-2bのresume内部呼出しが変わり得るので、Post後の差分へgate配置を照合してから実装する。

保存/復旧のfocused commandは新test作成後 `cargo test --locked --test audio_history_commit`、reader/storeは既存 `audio_history_closure` / `audio_history_store`、旧backupとconsumerは変更箇所の既存caseを実在名で固定する。Windows全体test/releaseは既存I10のままである。

残る明示事項は、結果不明によるprocess全保存停止の復帰を再起動/読取確認へ限定すること、pending拒否・局所破損時の回復閲覧画面はI8未完了であること、I3前にhistory付きbackupを製品提供しないことである。現appのpersist失敗→fatal→設定以外遮断は局所欠損後のautosaveでも起き得るため、I8で閲覧可能状態と書込停止を分離することが必要である。I2のreader成功だけでこの表示受入を満たしたとしない。I3のunion rootを教材commitへ渡す変更権限は通常save_progressにはなく、別の明示intent/CAS接合をI3で設計する。元仕様の保存済み/不明/破損時閲覧の区別を変える必要が生じたらspec担当へ返す。I2内で独自の成功表示や自動rollback/再送を決めない。Windowsの電源断/媒体障害、native UI/実APIは未検証である。

本書は設計のみであり、実装・テスト実行・公開を行っていない。使用量欠測は親のタスク測定へ引き継ぐ。

独立設計レビュー初回はCHANGES_REQUIREDである。2026-10-08再提出では、レビュー対象SHA-256 `73219261b073da6e8301373adcee57b9df4fa98b1bf4640cdbd6c122b4a657c5`からP1の2件を修正した。教材pendingのroot権限は§5/I2-T05b/I2a-1、外部exportの正本迂回は§5–6/I2-T08・08b/I2b-1e・b-3へ追跡する。親がレビュー記録を所有し、本書の修正だけで合格へ変更しない。文書検査と再レビューの結果は提出時に別記する。
