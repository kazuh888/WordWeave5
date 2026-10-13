# ET1 probe の限定設計

2026-10-13 / ACOUSTIC-HISTORY-001 / designer指定 `gpt-6-astra` / `high`。本書のみ所有する。実装・試験・独立Preは未実施である。

## 入力・承認・範囲

正本は [外部契約](early-trial-contract.md) 78–94、104–108行、SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` である。現物hash一致を確認した。[cycle 2仕様/設計PASS](early-trial-review.md)、[ET1前提Closed](early-trial-dependencies.md)、[計画](early-trial-plan.md) ET1とモデル割当を確認した。本書は88行のprobeと、その故障証拠だけを具体化する。外部契約、2root登録、登録前拒否、回復順、ET1の5file所有、品質gateを変更しない。

HEADは `9fd2dc66d0a51df5263b140c278a3900638d25f9`。着手時 `src/store.rs` と `tests/early_trial_paths.rs` に作者の未コミット変更がある。本書では読取だけを行い、作者の変更を保持した。AGENTS、development-team、KPIと `agent-skills:api-and-interface-design` を全文確認した。新UIはなく、UI skillは非該当である。

## 根拠と最小境界

- 現 `src/store.rs::Storage::at` はmkdir→app.lock open→排他lockの順である。`Storage::load` はguard→recover→readである。probeは存在しない。
- `atomic_temp_path` はpid/time付き候補名を作る既存private helperである。候補生成だけを再利用できる。`atomic_write_with_temp` はrenameを行いcleanupエラーを捨てるため、probeのpublisherとしては使用しない。
- `i2_export_bundle_tests::{Registration, Callback, observe_bundle}` はthread-local登録、二重登録拒否、再帰拒否、Drop解除の実在パターンである。そこで扱うdirectory/bundleのhookをprobeへ流用せず、この小さい登録パターンだけを再利用する。

本体の入口は既定の `open_early_trial(root: PathBuf, normal: PathBuf) -> Result<Storage, TrialOpenFailure>` のままである。追加の本体呼出APIを要求しない。probeの短い処理はこの関数内に置ける。`commit::check_early_trial_recovery(root: &Path) -> Result<(), String>` は既定の読取専用検査だけであり、probe故障seamと結合しない。public API、trait、callback引数、依存、汎用filesystem抽象化は追加しない。

## 順序・所有・エラー

1. 既定のroot/tree/current/pending preflightと登録済み対の整合検査を終える。ここまでの拒否は `BeforeWrites` であり、probe eventは0、試用/通常ledgerは完全不変である。観測handle/identityを保持し、必要な既定recheckを省略しない。
2. 最初のmkdir/app.lock/probe操作前に `StorageInitializationStarted` へ移る。既定初期化で安全な試用rootとapp.lockの排他所有を確立する。通常rootは作らない。lock取得失敗ならprobeへ進まない。
3. 安全確認された試用root直下の候補を `atomic_temp_path(&root.join(".ww5-early-trial-probe"))` で作る。probe作成直前にも保持観測を再確認する。`OpenOptions::new().write(true).create_new(true).open(&path)` が成功したときだけ、そのファイルの削除責任を得る。候補衝突は起動失敗とし、他者の既存ファイルを消したり上書きしたりしない。候補名が一意に見えることと、create_newで排他的に取得できたことを区別する。
4. 固定bytes `b"WordWeave5 early-trial storage probe\n"` を `write_all` し、`sync_all` する。write/syncのいずれが失敗しても、所有Fileを先にdropし、その後で所有probeだけを `remove_file` する。成功経路もclose→removeの順である。RustのFile dropをclose境界とし、観測不能なclose成否を成功証拠へ足さない。
5. create成功後のcleanupは1回である。write/syncの主エラーとcleanupのエラーは別のlocal結果として保持し、どちらかがErrなら起動を拒否する。cleanup失敗を成功に置換せず、その残存を試験traceとdisk oracleで別記する。create失敗時はcleanupを呼ばない。recursive delete、残存probeの起動時一括掃除、既存データの巻戻しはしない。
6. probe成功はremove成功までを要する。その後にだけ既定契約に従って検査成功Storageの試用boolと固定2root登録を確定する。probe失敗は登録を確定せず、返却途中のapp.lockをdropする。前回成功で登録済みの同一対は解除しない。回復は引き続きload側であり、probe中には行わない。

probeのcreate/write/sync/removeのI/O失敗はすべて `TrialOpenFailure { stage: StorageInitializationStarted, reason: StorageIo }` である。既定の2-field型・表示を拡張せず、Stringをparseしない。初期化開始後はmkdir/app.lock/probeが残り得るため、試用全体の不変を一般化しない。通常ledgerは全経路で不変である。root安全性のrecheck失敗は既定の発生箇所による理由分類を維持する。

この処理は同期であり、worker、async、cancel tokenを追加しない。実行中のcrash/killはErr後cleanupとは異なりprobe残存があり得る。将来のdisk-fullや電源断耐性を、起動時のwrite/sync成功から保証しない。敵対的OS操作との競争限界は正本を維持する。

## test-only seam の正確な契約

配置は `src/store.rs` 内の専用 `early_trial_probe_tests` moduleだけである。module、型、thread-local、すべての本体hook呼出を `#[cfg(all(test, windows, feature = "early-trial"))]` で囲む。作者がmoduleを所有し、実装者への逐次引継ぎ後に実装者が本体呼出だけを接合する。integration testからhookを公開せず、通常構成とreleaseには型・状態・分岐・fault設定環境変数を残さない。

以下は作者と実装者が共有する署名であり、未実装である。

```rust
// early_trial_probe_tests 内。型は必要な範囲だけ pub(super)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProbePoint {
    BeforeCreate, Created, BeforeWrite, Written, BeforeSync,
    Synced, Closed, BeforeRemove, Removed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeFault {
    None, CreateCollision, Write, Sync, Cleanup,
    WriteAndCleanup, SyncAndCleanup,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbeEvent {
    point: ProbePoint,
    path: PathBuf,
    injected_error: Option<std::io::ErrorKind>,
}
struct Registration { /* !Send/!Sync marker: PhantomData<Rc<()>> */ }
impl Registration {
    fn new(fault: ProbeFault) -> Self;
    fn events(&self) -> Vec<ProbeEvent>;
}
pub(super) fn observe_probe(point: ProbePoint, path: &Path)
    -> std::io::Result<()>;
```

`Registration::new` は同threadで既登録/observer実行中ならassert失敗とする。thread-local `RefCell<Option<Observer>>` がfaultとevent列を所有する。`events` はcloneを返し、参照やFile/guardを外へ出さない。RegistrationのDropで登録をtakeする。別のcallback実行中フラグとそのDrop guardで再帰を拒否し、unwind時もフラグを解除する。callbackはopen/loadや他hookを呼ばない。新規登録のnested利用や別threadへの持出しを許さない。二重登録/再帰/unwindのfixture不備は試験失敗であり、StorageIo成功証拠には数えない。

未登録時の `observe_probe` はOkのみである。登録時は実引数の絶対pathをcloneして各到達点を記録する。全eventのpathは同じであり、作者の合成trial直下、normal外であることをfixture側で照合する。fake path/identity、検査成功bool、代替Storage/代替publisherは注入しない。

| 呼出位置 | hookの作用と本体の扱い |
| --- | --- |
| BeforeCreate | candidate確定・既定recheck後、実create_new直前。通常Ok。CreateCollisionだけは当該pathへ固定sentinel `b"synthetic collision sentinel\n"` を別のcreate_newで実作成してOkを返す。その後の本体create_newが実OSエラーとなる。fixture作成失敗はpanicし注入成功に数えない |
| Created | 実create_new成功直後、空Fileを保持中。観測のみOk |
| BeforeWrite / Written | 前者は実write_all直前、Write系faultだけ `io::ErrorKind::Other` を返す。後者は実write_all成功直後のみで観測Ok |
| BeforeSync / Synced | 前者は実sync_all直前、Sync系faultだけ同Otherを返す。後者は実sync_all成功直後のみで観測Ok |
| Closed | create成功したFileをdropした直後。write/sync失敗経路でも到達する。観測のみOk |
| BeforeRemove / Removed | 前者はClosed後の実remove_file直前、Cleanup系faultだけ同Otherを返す。後者は実remove_file成功直後のみで観測Ok |

`BeforeWrite`/`BeforeSync` のErrはそれぞれ実操作を実行せず同じ主エラー経路へ入り、必ずclose/cleanupを続ける。`BeforeRemove` のErrは実removeを実行せず同じcleanup失敗経路へ入る。その他のpointは常にOkであり、本体test呼出では観測専用のexpectとして扱う。hookがErrとなったeventだけ `injected_error = Some(Other)` を記録し、それ以外はNoneとする。実create衝突を注入Otherと偽って記録しない。失敗後の到達していないpointをhookが合成記録してはならない。

## 独立oracle と受入対応

全ケースは有限timeoutの隔離子processで合成rootのみを使う。既存作者の子process/Temp/treeを再利用する。解除する登録はthread-local observerのRegistrationだけである。TrialRootsのOnceLockは試験でもresetしない。一度登録したroot対の固定と別対拒否を維持し、その状態分離には有限timeoutの子processを使う。全file bytesとdirectory集合は別々に取得し、空directory消失も検出する。normalの存在/不存在を含む全ledger、既存trial教材/進捗/媒体sentinel不変、fallbackなしを共通に確認する。許される新規app.lock/初期化directoryとprobeの差分を明示し、「Errだから全tree不変」とはしない。ケース終了時のテストfixture cleanupはoracle採取後である。

| ケース / 受入 | 実到達点・期待結果・残存 |
| --- | --- |
| 無故障 / ET-01 | 9点が順番どおり各1回。open成功、probe pathなし、trial内probe残存0。既存/不在trial各fixtureで実I/O成功を確認する |
| CreateCollision / ET-01 | BeforeCreateだけ。StorageInitializationStarted/StorageIoをmatchesで観測。衝突sentinelは完全保持、remove 0回。残存は本体所有probeではなくfixture所有fileである |
| Write / ET-01、ET-D01 | BeforeCreate→Created→BeforeWrite→Closed→BeforeRemove→Removed。StorageIo、所有probe残存0。実write/sync未実施を明示する |
| Sync / ET-01、ET-D01 | 上記にWritten→BeforeSyncが入りSyncedはない。StorageIo、所有probe残存0。実write済み、syncは注入により未実施 |
| Cleanup / ET-01、ET-D01 | BeforeRemoveまでの8点、Removedなし。StorageIo、実write/sync済みのprobeがちょうど1個、bytesは固定bytesと完全一致 |
| WriteAndCleanup / ET-01、ET-D01 | Write失敗traceかつRemovedなし、probeちょうど1個・0 bytes。主失敗とcleanup失敗の注入event両方を確認する |
| SyncAndCleanup / ET-01、ET-D01 | Sync失敗traceかつRemovedなし、probeちょうど1個・固定bytes一致。主失敗とcleanup失敗両方を確認し、sync成功とは記録しない |
| history/current/pending拒否 / ET-04、ET-D01 | 無故障observerを登録して既存拒否fixtureを実行しevent 0とBeforeWrites・全ledger不変を確認。probeに到達せず理由は既定matchesのまま |

分類は発生箇所により `StorageInitializationStarted/StorageIo` に固定し、注入エラーの本文を判定に使わない。実装が返す安全文を型のmatchesに代わる主oracleとしない。各故障caseはoracle採取・Registration解除後、同じ子processで別の安全な合成root対をopenして成功する対照を置き、失敗による初回2root登録の確定がないこととlock/fault解除を確認する。ET2の表示/app開始0/diagnostics0は既定型経由で別担当が検査する。ET-05の2root保護・再open・登録順、ET-06の外部送信なしは既存作者の試験へ引き継ぎ、本書で別policyを追加しない。

実OSエラーの証拠はcreate衝突と既存app.lock競合のfixtureである。write/sync/removeの失敗伝播・cleanup分岐はtest-only決定的注入の証拠であり、実disk-full/ACL拒否の再現成功とはしない。正常のcreate/write/sync/removeは実I/Oである。OS/ACL/quota変更、管理者権限、危険なI/O細工を試験前提にしない。

## 引継ぎ・未解決・測定

作者へは上記署名・point順・残存bytesを渡し、専用cfg(test)のみを追加する。未接続APIのcompile失敗を行動REDとは扱わない。実装者は本体open内の実I/Oとcfg hookを接合し、既定5file内で完結させる。runner/reviewerは本番cfg除外、実I/Oと注入の区別、cleanup失敗oracle、通常ledgerを確認する。試験件数/実在名は作者が確定する。

仕様gap・追加所有の必要はこの限定案では検出していない。残るgateは本書snapshotへの独立Preであり、親が別reviewerへ渡すまで未了である。現物実装、実行、実機保証、電源断/敵対的競争は未検証である。再委任、実装、試験実行、commit/pushは0である。

指定モデル/effortはplannerと親dispatchのAstra/highであり、runtime返却metadataとthread UUIDは未取得である。子識別は `/root/trial_probe_design`。開始/終了usage counterと週枠は未取得であり、合計や節約を推定しない。KPI記録は所有外のため親へ欠測として引き継ぐ。文書確認は入力hash、責任範囲、署名とoracleの整合、参照pathとdiff検査に限定する。

2026-10-13 ET-P01/P2追跡：初回snapshot `CAB517D1385717F523EFB716A7590A657F73EFD7F765DAEB6439AB9C656294B0` の独立PreはCHANGES_REQUIREDである。上記「独立Pre未実施」は初稿時点の記録である。observer解除とTrialRootsの寿命を混同した82行を修正し、OnceLockのresetを試験でも禁止した。指摘は修正済み・独立再Pre未了であり、PASS/Closedとは扱わない。source/testは変更していない。
