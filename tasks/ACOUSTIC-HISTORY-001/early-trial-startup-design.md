# ET2 起動接合の限定設計

現行追加Packetは末尾の **ET2-P01-V（通信観測、独立Pre待ち）** である。以下のET2-PR1/Pre-2の状態・6source表は当時の履歴であり、現在のsource凍結や実行解除ではない。ET2-P01-Vの所有追加は末尾に限定し、実行前条件の正本は本書ET2-G0を8source/通信へ同期した一節だけとする。

2026-10-14 / ACOUSTIC-HISTORY-001 / designer指定 `gpt-6-astra/high`。状態は **ET2-PR1設計是正案：Pre-2未判定・計画同期待ち** である。[独立Pre-1](early-trial-startup-review.md) はCHANGES REQUIRED、ET2-PRE-01をAcceptedとした。本書の修正だけでClosedとしない。ET2-Sの限定所有は [計画ET2追補](early-trial-plan.md) SHA256 `38C5486BF336465C85EEC18985EA68CC0890DE02F27A6E215494DEF045E409A9` の採用・親受領済みである。実装着手、seam安全性、動作合格を意味しない。製品契約は [early-trial-contract.md](early-trial-contract.md) SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` のままとする。[仕様・設計cycle 2 PASS](early-trial-review.md)、[ET1/FX1最終Post](early-trial-paths-review.md)も入力とする。正本の元の機能所有5fileをET2-F、追加test専用境界1fileをET2-Sとして区別し、モデル・製品仕様・既定検証gateは変更しない。前版の所有不足によるSTOPは解消済みだが、今回の実行前安全Gate不足はPre-2の独立判定を要する。

完了境界は現sourceに即した関数接合、失敗注入、独立oracle、残る証拠不足の返却である。所有は本書のみ。AGENTS、development-team、KPI、api-and-interface-design、wordweave-egui-uiを全文確認した。型付き境界とUI失敗導線に限定適用し、REST層は追加しない。

## 現物と最小変更

基準HEADは `97d2c72a9adec4f1904c6deb2b18e92c50262923`。開始時source clean。途中に親所有のmeasurements/current/usage文書の変更を観測し、保持した。

| 接合点 | 現物根拠と意味 |
| --- | --- |
| `src/app.rs:334` `new` | Storage open直後にdiagnosticsを開始する。試用factoryからこの関数を呼ぶと、load拒否時のdiagnostics開始0を満たさない |
| `src/app.rs:348` `new_with_storage` | load Errはfatalへ格納し、既定Progressで初期化を続けてSelfを返す。custom.tsv読取/parse、Speaker設定のErrもfatalになる。fatal appを構築しただけでは操作開始ではない |
| `src/store.rs:2260` `Storage::load` | 試用preflight再検査→既存guard下recover→read_progress。回復の実装・順序はここに一つだけ存在する |
| `src/commit.rs:480` `recover_with_io_under_guard` | deck/progress書込・再確認の後にpending archive rename。rename拒否時には書込済みでErrとなり得る |
| `src/commit.rs:2130` `r06_real_archive_rename_denial_is_known_pending_and_unlock_control_succeeds` | `read(true).share_mode(3)` による実rename拒否と解除対照が既存のWindows試験方式である |
| `src/app.rs:2302` `restore` | references検査→現在memoryの退避→diskコピー→save→epoch/UI更新。試用history拒否はこの先頭へ接合する |
| `src/app/qwen_settings.rs:31` `default_store` | testは既存Store、productionはWindowsCredentialStore::for_target。selector検査をtestのdefault_store置換だけで済ませない |
| `src/diagnostics.rs:201` `initialize` | 返値は()で、失敗は診断statusへ記録する既存契約。診断失敗を新たな起動拒否へ変更しない |

## 規範となる内部接合

以下の新helperはprivate、試用専用口はfeature内crate境界までとする。通常版`new`と既存`new_with_storage`の署名・fatal表示・診断順は維持する。

```rust
// app.rs。既存本体を移し、Storage::loadの呼出箇所だけloaderへ置換する。
fn new_with_storage_and_loader<L>(
    ctx: &egui::Context, storage: Result<Storage, String>, loader: L,
) -> Self
where L: FnOnce(&Storage) -> Result<Progress, String>;

// new_with_storageは上記へStorage::loadを渡す薄いwrapperとなる。
#[cfg(feature = "early-trial")]
pub(crate) fn new_early_trial(
    cc: &eframe::CreationContext<'_>, storage: Storage,
) -> Result<Self, TrialStartupFailure>;

// ccの生成/GPU/native windowが不要な、productionと共通のfactory。
#[cfg(feature = "early-trial")]
fn new_early_trial_with<L, D>(
    ctx: &egui::Context, storage: Storage, loader: L, start_diagnostics: D,
) -> Result<Self, TrialStartupFailure>
where
    L: FnOnce(&Storage) -> Result<Progress, String>,
    D: FnOnce(&std::path::Path);
```

`new_early_trial`はctx、実Storage::load、private診断開始関数を渡すだけとする。共通factoryは `new_with_storage_and_loader(ctx, Ok(storage), loader)` を一度呼び、fatalがあれば `RecoveryOrInitializationStarted` を返す。fatalがなければprivate試用boolをtrueにし、保持Storageの `dir/diagnostics` をDへ渡してからOk(app)を返す。全constructorの初期boolはfalseである。factory成功前にtrueを試験から代入して成功を偽装しない。

Dのproduction実体はinitialize→Application診断operationの開始/Startup Completedだけである。Dは診断開始という副作用境界であり、検証成功を注入する口ではない。テストDは呼出回数/pathを記録する。失敗時D=0と実fixtureのdiagnostics不在を両方確認する。diagnosticsの新公開観測API、global reset、既存sinkの変更は不要である。正常系でDが1回・試用pathである対照とproduction wrapperの静的照合を伴わせる。

`TrialStartupFailure`の2variantと本文は正本どおりとし、safeな表示関数（Display/Error実装を含めてよい）へ集約する。Storage::openのErrは `StorageOpenRejected { safe_message }` へ無加工で包む。loaderのErrやfatal内容を分類・引用せず、factoryへ入った後は常に `RecoveryOrInitializationStarted` である。秘密らしい任意Stringを与えても段階と定型本文が変わらない対照を設ける。

mainではfeature分岐でStorage::openをrun_nativeより前に行う。private `early_trial_creator(storage: Result<Storage, String>) -> Result<eframe::AppCreator<'static>, TrialStartupFailure>` は、open ErrならErr、OkならStorageをmove captureするcreation closureを返す。closureは `WordApp::new_early_trial(cc, storage)` のOkだけを `Box<dyn eframe::App>` へ変換する。Errは正本rfd通知後にboxed errorを返し、操作用Appを返さない。open Errの通知はmainがrun_nativeを呼ぶ前に行う。eframe 0.31.1のAppCreatorはFnOnce、Errは`Box<dyn Error + Send + Sync>`であり、Storage lockの所有はmain→closure→WordAppと一方向に移る。失敗時は未公開app/Storageをdropする。native window作成後のcallback拒否が一瞬の空windowも存在しないことまでは要求・保証しない。

titleはfeature別定数をNativeOptionsへ直接渡し、正本literalを独立assertする。通常titleと通常creatorは維持する。debug専用`--ui-check`は合成UI検証口であり、release提供/通常起動の代替証拠にしない。

Qwen selectorは正本の `configured_store_with<F,T>(factory:F)->T where F:FnOnce(&'static str)->T` をprivateのまま用いる。production default_storeだけがこのselectorを介してfor_targetを構築し、cfg(test) default_storeは既存mockを保持する。private selectorの新試験はqwen_settings.rs内の子cfg(test)へ置く。兄弟qwen_settings_tests.rsから呼ぶために公開範囲を増やさない。namespace別spyでeditor/ReadingControllerの読取・保存を観測し、別caseで実for_target構築とtarget()だけを実行する。OS資格情報のload/saveは0である。

restoreはprivate試用boolかつcandidate.audio_history.is_someなら最初に正本文言でErrを返す。その後は既存確認/検査/退避/save順を維持する。既定offと通常constructorのbool=falseにより、既存core履歴試験と通常版動作を保持する。

## ET-D01の3状態を同一factoryへ通すoracle

試用登録/環境は既存ET1と同じ有限timeout子processへ隔離する。synthetic LOCALAPPDATA、通常sentinel、試用rootだけを使用する。private検査boolやfake identity、lib private faultの公開、recover複製は不要である。以下は設計であり新caseの動作実証は作者/runner待ちである。

1. **書込前拒否**：history currentまたはpendingのfixtureで実Storage::openを失敗させる。mainのcreator準備関数はErrでclosureを生成しない。元のstore安全文がそのまま残り、source全bytes/dir集合不変、diagnosticsなし、通常sentinel不変を確認する。open後失敗の安全文もBeforeWrites文へ置換されない対照を置く。
2. **実回復書込後失敗**：有効な非履歴pendingのold/nextに異なるdeckとProgressを用意し、実Storage::openで登録後、pending fileを `OpenOptions::new().read(true).share_mode(3)` で保持する。読み取りとattributes観測を許しdelete共有だけを拒否する。共通factoryへ実Storage::loadを渡す。既存recoverがdeckとprogressをnextへ実書込し、pending archive renameでErrとなることを、返値とledgerで確認する。nextの2file、元bytesのpending、backups directory、その他不変、D=0、AppのErr、安全文の不変断言なしがoracleである。handleを解除して同じデータを再open/loadすればpendingが同一bytesでarchiveされ、正常factoryのD=1となる対照を必須とする。archive名は既存動的名であるため個数と内容を検査し、生成関数から期待名を作らない。
3. **回復成功後の実初期化失敗**：同じ有効な非履歴pendingに対し、Lは最初に実 `storage.load()?` を呼ぶ。pending消滅、archiveの元bytes、next deck/progressをその場でassertしてから、回復済みcustom.tsvを `read(true).share_mode(0)` で保持し、loadしたProgressをそのまま返す。既存constructorのcustom.tsv読取が実Errになりfatalへ入り、共通factoryがErrを返す。テスト所有handleはRefCell等へ保持して読取段階まで生存させる。解除後のledgerは回復成功時と同一、D=0、操作App返却なし、同じ回復後安全文である。handleを持たない対照では同一loader/constructorが成功する。試験のために回復済み教材bytesを書き換えたり、fatal=trueを直に設定したりしない。

各caseはOS fault成立を前提assertで識別し、早すぎるopen/preflight拒否を目的の回復後失敗として数えない。(2)はnext bytes到達、(3)はloader成功と実教材read拒否を必須とする。dir/app.lockの生成はopen段階の既存副作用としてsnapshot境界を明記する。解除後にhandle/Storage/child終了をwaitし、fixture残存を記録する。OS共有拒否が成立しない環境ではskipで受入済みにせず停止する。

既存 `commit::early_trial_paths_tests::et_d01_nonhistory_recovery_fault_after_actual_write_retains_partial_state_then_retries` はAfterDeck checkpointの実書込後Errを独立に証明する補助である。上記(2)はそのprivate seamを呼ぶ試験ではなく、同じproduction recoverのarchive障害を利用したapp接合試験である。両者を同一fault/checkpointの実測とは呼ばない。loader Err stubはString非依存の型検査だけに使用し、実回復の証拠へ代用しない。

## 受入IDと独立作者への引継ぎ

| 正本ID | モジュール/データ所有・新case候補 | 判定と再利用 |
| --- | --- | --- |
| ET-01/04、ET-D01 | main creator、app共通factory、Storage所有移譲 | 上記3状態、loader呼出1、D=0/成功1、normal bytes不変。開けない試用rootを通常rootで補完する経路なし |
| ET-02 | qwen_reading定数、qwen_settings private selector、editor/controllerがArc storeを保持 | 3namespaceの独立literalとspy ledger。通常のみ設定済み→試用未設定、試用save/reopen、変更先+空key拒否、通常/tool load/save0。実for_target.target一致、実key操作0。削除N/A |
| ET-03 | main title、既存UI/使用説明はET3 | 正本title literal、constructor成功後Home/Settings/既存音読導線。native title/拒否dialog/直接起動はET3 |
| ET-04 | app restore入口、現在Progress/draft/epochはWordApp、永続化はStorage | history候補拒否で全tree/current Progress・日本語draft・material draft・epoch・message・dirty不変、退避file増加0。非history JSON成功でdisk/UI head=None |
| ET-04/06 | app cfg(test)から既存import_deck/cache_words、material approval、backup確認、chat添付/drop入口 | 初期None→各操作後save/reloadもNone。承認前/cancelで適用0、完了扱いなし。既存backup::prepareのhistory拒否を再利用し、prepared.progressを試験で編集しない。手動入力元がnormal rootでもQ01を保持 |
| ET-06 | factory/restore/設定、既存Qwen mock transport/probe | 起動/表示/save/restoreでsend/probe0、pending/connection_check/recorderなし、ReadingPhase::Input。認証/CODEX_HOME変更なしの呼出経路確認。下記録音counterの限界を保持 |
| ET-07 | ET3 runner/親 | 本書の文書検査はEXE提供gateを代替しない |

ET2-Fの新testは元の許可5fileの専用cfg(test)とqwen_settings_tests.rsへ、ET2-Sの安全対照はmedia.rsの専用cfg(test)へ置く。既存 `harness_tests::fixture/frame`、backup確認のegui data action rect、Qwen preview Store/Wire、既存editor probe spyを読取再利用する。必要なprivate子module helperへアクセスできない場合、公開化や許可以外のfile編集を先取りせず、許可app.rs内から既存入口と同等の独立fixtureを用いる。合成progress/deckを正本literalから作り、期待値をproduction formatterやresolver返値から生成しない。

録音oracleは元の5fileだけでは不足する。実Recorder::startはsrc/media.rs:61、呼出はapp.rs:1497、app/chat_media.rs:1200、app/qwen_reading_ui.rs:307の3箇所であり、既存mock counterは未発見である。既存 `opening_dialog_does_not_record_or_send` はwire.count=0と入力状態の検査であり、マイク開始counterの検査ではない。正本の「mockで録音開始0回」を静的非到達やrecorder/pending無しだけへ読み替えず、以下のET2-Sを必須依存とする。

### ET2-Sの限定所有（planner採用・親受領済み、Pre未判定・未実装）

追加所有は `src/media.rs` のcfg(test) observerとRecorder::start先頭のcfg(test)分岐だけである。production Recorderと3呼出側の契約・動作は変更しない。元の機能所有5fileに加える6file目であり、根拠は計画ET2追補のET2-S採用である。observer本体と入口接合は実装者、同fileの独立試験/fixtureはtest作者が所有し、作者→実装者→作者の順に所有を返却する。同時編集、作者による本体scaffoldは行わない。designerは本書だけを所有する。chat_media/qwen_reading_ui、既存shared harness、store/commit/diagnosticsは読取再利用のみである。

```rust
// src/media.rs、全てcfg(test)。作者はこの署名と期待値を先に固定する。
pub(crate) fn watch_recording_starts()
    -> Result<RecordingStartWatch, &'static str>;
impl RecordingStartWatch {
    pub(crate) fn calls(&self) -> usize;
}
// Dropで自分が登録したobserverだけを解除する。
// Recorder::start先頭：監視activeなら回数加算後、固定の合成Errを返す。
// cpal::default_host/default_input_deviceよりも前に置く。
```

テスト専用のprocess-global Mutex<Option<Arc<counter>>>でactive監視を一つだけ登録し、重複watchはErrとする。thread-localではworker経由の呼出を見逃すためprocess-globalとする。startはlock内でactive判定と加算を完了し、activeなら必ずErrで戻る。正常lockでwatch未登録の場合だけ既存startへ進む。lock取得異常/poisonを未登録と解釈せず、cpal前の固定Errとfixture異常として扱う。RAII Dropは同じArcの登録だけを解除し、所有外のglobal reset口を設けない。poison時にDropが内側guardを回収する場合も自分の登録の解放に限定し、poisonを解除して試験を継続成功へ変えない。

合成試験は既存の有限timeout子processで実名1caseだけ実行する。録音入口を呼ぶ正対照を含む実行の許可は、次節ET2-G0だけを規範とする。解除後の最初の動的試験では、active中にRecorder::startを直接1回呼んでErr/calls==1、別threadからの1回捕捉とjoin、重複watch拒否後の既存watch有効性、正常終了/捕捉panic時のRAII解除と再登録を確認する。counter未接続の0を成功へ数えず、この対照自体も実micへ到達させない。lock異常の経路もcpal前で拒否することを確認する。

安全対照が成立した後に、watch開始→共通factory起動/設定表示/設定保存/復元/既存音読準備と発生した非同期処理の終了確認→calls==0→RAII解除、の順でET2-Fを試験する。途中panicでも解除し、child wait/timeout/残存を記録する。既存未監視testと非test buildは差分/cfg条件の照合で不変を確認し、無監視の実マイク開始を対照試験にしない。実名/0件除外、source・EXE hash、通常/試用構成は独立runnerへ引き継ぎ、最終sourceを独立Postへ固定する。

これで3箇所の呼出が共通入口を通る限り、録音開始の試行0を動的に観測できる設計となる。実証は未了である。新しい別入口やobserverを迂回するcpal呼出があればSTOPする。設定/復元が返した後に未joinのworkerが残っていればwatch解除しない。既存許可5fileだけでこの監視を偽装する別名Recorder/重複moduleは作らない。

factory/loader/selector/watchの署名と全testを作者が固定した後、API不在のcompile失敗はcompile blockerとして実装者へ渡す。行動REDを作るためのstub、一時公開API、偽成功factoryは設けない。compile成功だけでは実行を許可せず、ET2-G0を経てからET2-S安全対照、ET2-Fの順とする。未取得REDの感度確認もET2-G0と計画の限定手順に従う。実装者だけが一時本体差分を所有し、独立runnerの目的assert FAIL確認と候補hash復元・安全Gate再確認後のfocused GREENを最終Postへ渡す。

### ET2-G0：録音・通信入口を呼ぶ試験の実行前安全Gate（唯一の規範）

1. 実装者が変更を止め、ET2通算8source（main/app/qwen_reading/qwen_settings/qwen_settings_tests/media/ai/qwen_reading_ui）の全bytes hash・dirty識別・構成（通常/early-trial）を固定する。親が実物を読み、録音watch/startの同一global、cpal前のactive判定→加算→固定Errとpoison拒否を確認する。通信を含む実行では、さらに末尾の全8入口が一つのbin側globalを共有し、Codex実lib・journal・認証子process、Qwen inner Transport.send、probeのconnection解決・worker生成の前に加算/拒否することを確認する。登録重複、poison時に未監視へ落ちないこと、default_storeの合成Store選択、guard/保持Transport/workerの寿命と終了手順も読む。symbol存在やcompile成功だけでは解除しない。通信接合が未実装のC1/C2を録音だけの既存限定範囲で扱う場合も、通信試験は解除対象外と明記する。
2. 確認済みsourceから `cargo test --locked --bin wordweave5 --no-run --message-format=json` を既定の通常/試用別target-dirで行う（試用のみ `--features early-trial`）。必要な既存lib検証を併記しても、binの通信接合確認をlib EXEへ代用しない。build前後8source一致、JSON由来のprofile.test・features・EXE絶対path/SHA256、作者が固定した対象実名/予定件数を一組にする。親が対応を照合し、その組と実行順だけを明示解除する。EXEのlistも解除後に行い実名集合を照合する。解除前は全担当とも録音/通信正対照を実行しない。API不在compileはblockerであり行動REDではない。
3. 各構成で、解除された同EXEの録音安全対照→通信8入口と登録/寿命の安全対照→5操作のゼロ観測、の順に進む。同じ固定組で既に通った対照は証拠を接合し、理由なく反復しない。旧録音G0だけで通信正対照を許可しない。通常ユーザー、既存Hidden/有限timeout/child wait・残存検査・合成rootの条件を保持する。安全対照の失敗、0件、timeout、未終了workerなら解除を停止する。静的Gateと動的counterは互いを代替しない。
4. 8source・EXE・構成・対象集合の変更、感度変異・逆復元で解除は失効する。手順1–2を再確認し親が解除する。同じhashへの復元でも旧解除は自動復活しない。感度実験では録音/通信の遮断と実加算を保持し、calls/snapshotの観測返値だけを誤らせる。正対照には誤0、操作0oracleには誤非0を独立に与え、目的assert FAILを確認する。遮断除去、別global化、実資格情報・実API・実録音による対照は禁止する。

このGateは親による現在タスク内の読取確認と記録であり、harness/settingsやproduction APIの変更ではない。実行禁止状態からの解除条件だけを本節に集約し、計画はこの節を参照する。

## 非同期・保存・UIの境界と停止条件

起動は同期処理であり、新worker/retry/cancelを追加しない。成功後だけ既存イベントループがappを所有する。Qwenのgeneration、CancellationToken、送信前確認、GatedTransportは現状維持。load/recover/restoreのguard、archive、metadata commit順を変更しない。回復後失敗にrollbackや自動削除を追加せず、利用者の再起動を維持する。diagnosticsは起動成功の後、通常rootには出さない。

新しいeguiボタン/レイアウトはない。既存の日本語font設定、UiControlsによる両軸の可視glyph中央、focus/keyboard、zoomを共用する。rfd拒否はOS controlであり、日本語折返し、OKの両軸中央とfocus/keyboard、長い安全path、100/150/200%DPI、狭い表示領域の確認をET3へ残す。titleは色に依存しない識別である。headless幾何/文字列成功をnative受入としない。

不採用は、通常newを呼んでfatal appをそのまま返す案（拒否前diagnostics/操作可能）、loadを先に別途呼んで既存constructorで再loadする案（二重回復）、raw Stringで段階を分類する案（誤った不変保証）、production公開fault API/recover複製（所有・回復順の拡大）である。

追加仕様作成/公開/実資格情報/API/マイク/native操作は今回非該当。実装、独立test作成、runner、Pre/Postは後続別役である。本書の承認はそれらの合格を意味しない。録音oracleの安全対照不成立、OS fault不成立、許可6file/各所有範囲超過、新UX制限、規範との矛盾ならSTOPし親/plannerへ返す。planner追補との新たな阻止矛盾は今回の文書照合では未検出である。

## Preへ固定するsnapshot

本書のhashは保存後のhandoffに添付する。次はplannerによるET2-G0への計画同期、その新plan hashを含む独立Pre-2である。Pre対象は本設計全体（load1回、fatal拒否、bool/diagnostics順、creator所有、ET-D01実障害、private selector、録音watch/ET2-G0）、同期後計画追補、正本hash、下記6source、ET1最終Postである。旧cycle 2 PASSは今回の具体接合Preを代替せず、CHANGES REQUIREDとなったPre-1の旧hashを現行判定へ流用しない。Pre PASS前のsource/test着手は行わない。下表plan hashは今回の入力snapshotであり、Pre-2提出時には親が同期後hashを別記して固定する。

| 対象 | SHA256 |
| --- | --- |
| early-trial-plan.md | `38C5486BF336465C85EEC18985EA68CC0890DE02F27A6E215494DEF045E409A9` |
| src/main.rs | `188C3C04365ED587872C049611F5E3F7F763538B69DFB6F5CC93439D9EECC94F` |
| src/app.rs | `C1706E880A27C44BDD205D1BA3FD0E01E8D7D43FF0D3F8543F3D62AE251EDA0A` |
| src/qwen_reading.rs | `D337E8C2308DEA99D776BB5D3A51E40034A997DB5D41F97EC5A048CB8F5FDC75` |
| src/app/qwen_settings.rs | `6E0EDE27082E4AE21FA41C177B93627E9A000D78EAD853EE24DAD73AFE4BD387` |
| src/app/qwen_settings_tests.rs | `B33126247A03016E503F07215FCDD2E6760551DD349882B4700EBC64D6111E17` |
| src/media.rs（ET2-S test専用境界） | `8585149F7ABF9CEACC480DAF2147B263C35C52840D2DF1C75A08010B8455D4B9` |

## ET2-PR1 Packet / Pre-2提出準備

closure matrixとfinding原文の正本は [独立review記録](early-trial-startup-review.md) である。ET2-PRE-01はAccepted、今回の是正計画はPlanned相当でありPre/Post未判定・Closedではない。根本原因は「compile可能なwatch定義」と「cpal前の実遮断接合」の区別を実行前条件へ置かなかったことである。現HEADのmedia.rs:61–62にはwatchがなくstart直後にcpalがあり、3callerも不変である。

| Packet項目 | ET2-PR1の範囲と根拠 |
| --- | --- |
| Objective / traceability | ET2-PRE-01→本書ET2-G0→計画の参照・実行順。録音正対照そのものの未確認実マイク到達を防ぐ |
| Scope / owner / consumers | designerは本書だけ、plannerは計画参照、親はsource/EXE確認と解除、作者/実装者/runnerは解除を消費する。source6file、製品仕様、通常未監視test/非test動作、公開API/OS設定は変更しない |
| Input / output / error | 入力は固定source hash/構成と読取証拠、対応test EXE path/hash/実名。出力は親のその組への解除記録。欠落・不一致・接合漏れ・poison流出は解除なしで停止する。新runtime型/APIは追加しない |
| Contract / state / data boundary | 唯一の規範はET2-G0。禁止→静的source確認→対応EXE固定→親解除→動的試験、変更時は禁止へ戻る。失敗後も実mic/API/資格情報へ進まない。counter成功を静的Gateへ、静的Gateをcounter成功へ代替しない |
| Dependencies / oracle | planner同期→同じdesign/plan/sourceのPre-2 PASS→実装後ET2-G0。watchだけでstart接合なしのsourceはcompile可能でも静的確認で拒否され、positivecontrolは未実行であるべき。別global/poison流出/EXE不一致/変更後旧解除も同様。遮断・加算を保持した観測返値の感度差分だけが動的FAIL確認へ進める |
| Rollback / stop / residual | 本書のET2-PR1差分だけをownerが個別patchで撤回する場合、ET2-PRE-01は未解消・実行禁止へ戻す。source/reset/実データ巻戻しなし。今回残るartifactは設計と親のレビュー記録のみ。追加scope/製品判断/同原因2回連続差戻しはSTOPし正本再構成へ返す。実装・実行・Post・ET3は未完了 |

削除した旧条件は「一括compile後、直ちに最初の安全正対照を実行する」である。置換後はET2-G0だけが実行解除を規定する。execute-review-remediationとprotocol、review-project-conventions、review-reliability-data-integrityを設計のself-checkへ適用した。並行監視のlock異常、source変更、worker残存の境界を確認し、独立Gateの代行はしていない。

Pre-2 requestは計画同期後に親が提出する。Target branch=`codex/development-agent-team`、HEAD=`97d2c72a9adec4f1904c6deb2b18e92c50262923`、target artifactは本書と同期後計画、review classは設計/規約/信頼性、requested GateはPre-implementation cycle 2、dependencyはET1/FX1 ClosedとET2-S受領済み、product decisionなし。out-of-scopeは上表のとおりである。本書hashはhandoff、同期後plan hashは親のrequestに固定する。変更前Pre-1 design hash `D9E044C42C85D1C1914B2E24820AE31D6B4082B7B920EC9F4E5C5CB1DDBBB42A` は履歴である。今回の文書検査はgit diff --check、参照/空白検査、6source hash照合であり、source実行試験は0である。

指定model/effortはplannerのAstra/high、runtime返却値は未取得である。子のcanonical task名は`/root/trial_startup_design`、固有thread ID/usage counterはこの担当では未取得。親が開始/終了測定を担当する。設計者はsource/test編集0、再委譲0、実credential/API/録音0、文書検査のみである。

## ET2-P01-V：操作前からの通信観測の限定接合（2026-10-14）

状態は **設計具体化済み・独立Pre未判定**。ET2-P01/P2はAcceptedのままである。入力は [計画のET2-P01追補](early-trial-plan.md) SHA256 `BCD69DBC7B1EA022598374A93D967DE5DA8239427FF5443C3D55F5E2F5E3A785`、上記Pre-2 PASS版設計 `24B9FEBE6F83528EF1B4D55EF9890B00FC9846E9F50B5BB446F48A2FAE687D7E`、[C1静的指摘とG0記録](early-trial-startup-review.md)、[実装/実行記録](early-trial-startup-implementation.md)、[作者記録とmain fixture是正](early-trial-startup-tests.md)である。外部契約は既承認のET-06、契約hash `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` を保持する。初期plan入力 `FDF9D92BC710C99C40A1F5F5E3EC64EE9B03C28C1A78F8ADEB9FCFC62B103AA5` からの差は、設定済み起動に必要な同fileのtest Store選択をplannerが採用し親が受領したことである。

指定designerは `gpt-6-astra/high`。完了境界は本節の型・接合・安全Gate・独立oracleの固定とPreへの返却であり、source実装/実行/最終受入を含まない。AGENTS/development-team/KPI、`agent-skills:api-and-interface-design`を全文確認した。Skillは既存Rustの内部契約とエラー変換に適用した。UI配置・日本語表示・ボタン・縮尺を変更しないtest専用Packetであるため、今回の`wordweave-egui-ui`適用は非該当である。上記の日本語・両軸中央・100/150/200%DPI・狭幅のET3受入はそのまま残る。

### 所有と実入口

親受領した変更対象は4fileである。`src/ai.rs`のcfg(test)固定監視と6出口直前、`src/app/qwen_reading_ui.rs`のcfg(test)Transport接合/試験終了補助、`src/app/qwen_settings.rs`のcfg(test)default_store選択/start_probe先頭、独立作者の`src/app/qwen_settings_tests.rs`だけを扱う。本節は旧ET2-Sの「qwen_reading_ui読取のみ」をこの限定範囲で更新する。main/app/qwen_reading/mediaの本体、lib/codex/依存/feature、shared harness、control_settings/run_history/chat_mediaは編集しない。実装者と作者は同fileを順番に所有し、相手の本文や期待値を変更しない。

現物で `src/main.rs` がai/app/mediaをbin moduleとして構築することを確認した。libのcfg(test)へ監視を置いてもbinの通常依存libには入らないため、監視実体は **ai.rs内の一つのcfg(test) inline module** に置く。crate内の必要consumerにだけ可視とし、production公開API・別名module・新REST層・汎用DIを作らない。

| 固定Entry（8種類） | 現物の入口と遮断位置 | 戻り型と拒否後の外部境界 |
| --- | --- | --- |
| `CodexCheck` | `Config::check`の`codex::check`直前（ai.rs:149） | `Result<String,String>`。libのobserved_operation/Server::start/initialize/Command::spawnより前 |
| `CodexChat` | `Config::chat`のgenerate_with_effort直前（ai.rs:156） | `Result<ChatReply,String>`。journal作成/実送信/parse前 |
| `CodexMaterial` | `material_detailed`のgenerate_with_effort直前（ai.rs:180） | `Result<Draft,MaterialFailure>`。material wrapperもこの一入口へ合流 |
| `CodexGenerate` | private `generate`のgenerate_with_effort直前（ai.rs:214） | `Result<String,String>`。response/structured経由のfeedback/exercise/assess/draft_word等を含む |
| `CodexReadInk` | `read_ink`のgenerate_with_effort直前（ai.rs:240） | `Result<String,String>`。base64処理は既存どおり、実lib前 |
| `CodexTranscribe` | `transcribe`のgenerate_with_effort直前（ai.rs:250） | `Result<String,String>`。既存サイズ拒否は先行し、正対照は上限内合成WAVを使う |
| `QwenSend` | 実`open_qwen_reading`→`QwenDialog::new`でtransportを接合し、GatedTransport→そのinnerの`send` | `TransportFuture<'a>`。ReqwestTransport.sendへ渡す前。newのclient構築は送信回数に数えない |
| `QwenProbe` | `QwenConnectionEditor::start_probe`の第一処理 | `Result<(),SafeError>`。refresh/invalidate/resolve_connection/ReqwestProbe生成/thread spawnより前 |

Codexの5generate参照とcheckを全件照合した。`list_model_efforts`（control_settings:293）、`recover`（run_history:73）、語彙downloadは別の明示操作であり、今回の起動/表示/save/restore/音読準備には含まれない。対象5操作から新たな到達経路が判明したら範囲を戻す。除外経路の静的非到達は対象8入口の動的観測を代替しない。

### 固定型、登録、拒否と所有寿命

以下は実装前のcompile契約であり実装例ではない。`Entry`の順序は上表で固定する。配列はprivateで、snapshotの`calls(entry)`だけを公開する。非testには型も接合も存在しない。

```rust
// ai.rs 内 #[cfg(test)] pub(crate) mod trial_egress
enum Entry { CodexCheck, CodexChat, CodexMaterial, CodexGenerate,
             CodexReadInk, CodexTranscribe, QwenSend, QwenProbe }
struct Snapshot { calls: [usize; 8] }
enum WatchError { AlreadyActive, Poisoned, Ended }
enum Block { Observed(Entry), Poisoned, Ended }
struct Watch { state: Arc<State> }          // Cloneを実装しない登録owner
struct Lease { state: Arc<State> }          // test Transportだけが保持する
// Stateはprivate: [AtomicUsize; 8] と合成 Arc<dyn CredentialStore>。
fn watch(store: Arc<dyn qwen_audio::CredentialStore>) -> Result<Watch, WatchError>;
fn capture() -> Result<Option<Lease>, WatchError>;
fn reject_if_watched(entry: Entry) -> Result<(), Block>;
fn selected_store() -> Result<Option<Arc<dyn qwen_audio::CredentialStore>>, WatchError>;
impl Watch { fn snapshot(&self) -> Result<Snapshot, WatchError>; }
impl Snapshot { fn calls(&self, entry: Entry) -> usize; }
impl Lease { fn reject(&self, entry: Entry) -> Block; }
```

module/consumer間で必要な型/メソッドだけを`pub(crate)`とする。`State`、global、任意reset/任意callbackは露出しない。`watch`のStoreは既存の合成`preview_data::Store::new(None/Some(connection))`を作者が作って渡す。production Storeを渡さないことをG0で確認し、秘密/host/request/音声をcounterやログへ保存しない。watch登録だけではstore.load/saveしない。

globalは `Mutex<Option<Arc<State>>>` 一つである。正常lockかつNoneの時だけwatchを登録する。重複はAlreadyActiveで先行watchを保持する。`reject_if_watched`は同じlock下でactiveなら該当counterをSeqCst加算してObservedを返し、**正常Noneだけ**Okとする。lock poisonはPoisonedで拒否する。snapshotもpoisonや自分のArcが登録されていない状態をErrとし、異常を全0へ変換しない。capture/selected_storeは同じglobalからArcをcloneし、lockを解放してから構築/credential読取を行う。global lockをawaitやtrait呼出に跨がせない。

Dropは`Arc::ptr_eq`で自分の登録だけを解除する。poison guardを回収して自分の登録を外してもclear_poisonせず、以後の呼出を拒否する。捕捉panic後の通常解除/再登録と、poison後の継続拒否は別oracleである。poison試験は独立childで行い、後続caseへ汚染を持ち越さない。

`QwenDialog::new`では既存引数transportを、capture=Someの時だけprivate `WatchedTransport { lease, inner }` へ包みReadingControllerに渡す。Noneなら元のArcそのものを渡し既存mockを保持する。capture ErrはSafeErrorへ変換してconstructorを拒否する。WatchedTransport.sendは同期部分で保持Leaseのcounterを加算し、innerを呼ばず合成TransportFailureを返すFutureを作る。Futureも同ArcのLeaseを保持する。guardが先にdropされてもこのwrapperは実transportへ復帰しない。Lease.rejectは登録Arc一致ならObserved、poisonならPoisoned、登録なし/別ArcならEndedとして必ず拒否する。後二者でも保持元counterへの呼出は記録し、新しいwatchのcounterへ混入させない。古いTransportを次caseへ使い回すことはfixture誤用であり、Ended対照以外は禁止する。

設定済み起動はfactory後の`app.qwen_store`代入では成立しない。現default_storeはtest時に毎回Store::new(None)であるため、受領済みの限定接合としてcfg(test)分岐の先頭でselected_storeを読む。Someなら登録された同じ合成Arcを返し、Noneは従来のStore::new(None)、Errは固定メッセージのtest fixture panicとする。返値がArcである既存署名を広げない。poisonで既定Storeへ落として合格を続けない。非testのprivate selector/WindowsCredentialStore::for_targetは一切変更しない。app constructor、editor、ReadingControllerの既存所有移譲とstore.load/save契約を保持する。

### 型変換、状態、非同期の終了

Codex6入口ではreject_if_watchedのErrを固定合成Stringへ変換して即returnする。Observed/Poisoned/Endedは異なる固定literalとし、入力本文・path・認証情報を含めない。material_detailedだけは`MaterialFailure::from(String)`を使う。現material.rs:118にFrom<String>があり、`Other(String)`となる。`material()`のlegacy_messageへの変換も既存どおりである。`String.into()`の型推論に依存してMaterialFailureと取り違えない。

probeとconstructorの拒否は`SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent)`である。probe_state、revision、probe_inputs、probe_jobは更新せず、workerを生成しない。SafeErrorに任意本文を詰める口はない。TransportFutureは `Pin<Box<dyn Future<Output=Result<TransportResponse,TransportFailure>> + Send + 'a>>`（tools/qwen-audio/src/provider.rs:71）であり、SafeErrorを直接返せない。wrapperは`Box::pin(async move { /* Leaseをここまで保持 */ Err(TransportFailure::Network) })`を返す。

既存`DisplayResult`という型はsrcとtools/qwen-audio/srcに存在しない。表示へはevaluateのSafeError→ReadingView.error→user_error/Displayが使われる。GatedTransportはinnerを呼ぶ前にstarted=trueにするため、監視Qwen正対照の表示状態は **Failed / Network / MayHaveBeenSent** となる。これは既存保守的表示であり、inner未呼出という試験事実と分ける。NotSent表示を要求するためにcontrollerや製品エラー契約を変えない。

Codex正対照は合成Configを直接構築し、exeには実行不能sentinel、cwdには空の専用fixtureを指定する。check/5generateを実メソッドから呼び、固定拒否と該当1/他7個0、cwd/journal未作成、sentinel不変を確認する。外部lib到達0の根拠はG0で読むlib直前の無条件returnとその固有合成Errであり、未設置のlib counterを測定したとは報告しない。Qwenは既存Wire/拒否spyをinnerへ渡した同じconstructor経路の正対照でinner.count=0を測り、実appのopen_qwen_readingで作ったDialogについてもQwenSend=1を確認する。probeは先頭拒否と既存Probe spy=0、jobなしを確認する。この組で実入口を通らないwatch helper単独の試験を排除する。

Codex別thread正対照はJoinHandleを必ずjoinし、その後にsnapshot/guard解除する。probeは先頭遮断によりworker生成0であり、既存probe workerがjoin可能になったとは主張しない。Qwen正対照は合成audio/connectionでprepare→既存human_sendまたはSend action→pollでFailedを観測する。`pending=None`、receiver受信、controller.close、既存runtime.shutdown_backgroundのいずれもworker終了証拠にしない。

qwen_reading_uiのcfg(test)だけに `pub(super) fn finish_egress_test(&mut self)` を置く。runtimeを先にtakeし、controller.close/既存stop_media/状態解放を行い、取り出したRuntimeを同期dropしてworker終了を待つ。closeのshutdown_backgroundへ渡さない。実装上はruntimeをtake→既存close→runtimeのdropという最小接合でよい。正対照作者は同file子testでruntime/workerを所有し、操作case作者はこの限定補助で実app所有Dialogを終了する。通常close/Dropの契約は変更しない。Runtimeの同期dropが返る前にwatchを外さず、完了しなければ既存child timeout/kill+waitで失敗にする。shutdown_timeout後の残存を成功と扱わない。

試験のcleanupはassert panic時にもDialogの同期終了→audio job registry drain→app/Store所有解放→最後のsnapshot→watch解除の順を確保する（作者の専用RAIIまたはcatch_unwindで行い、shared harnessは変更しない）。5操作自体に新workerはなく、これを各実経路の静的照合で確認する。想定外のpending/connection_check/workerを検出した場合は正常終了扱いにせず、終了を確認できるまで監視を維持してchild失敗を親へ返す。終了不能時はwatchを解除せずchild終了に委ねる。未知workerの終了を証明するための追加file/公開APIが必要ならSTOPする。

保存順序は不変である。設定はstore.save成功後にeditor baseline/key、一般設定はStorage.save成功後にprogress/draft/zoom、restoreは既存検査→現在状態退避→save_progress成功後にmemory/epoch/UIを更新する。watchは全期間の観測所有者であり保存transactionの参加者ではない。回復/rollback/retry/cancelに新たな振舞いを加えない。試験終了後の合成fixture cleanupだけを行い、通常/実学習データには触れない。

### ET-06 oracleと引継ぎ

| 受入/単位 | 実操作・所有・判定 |
| --- | --- |
| ET-06 / P01-1 | 全8Entryで実入口1回→該当1/他0、型に合った合成拒否、外部到達0の上記証拠。重複登録、別thread捕捉/join、通常Drop/unwind後再登録、poison、保持Transportのguard終了後拒否を独立childで確認 |
| ET-06 / P01-3起動 | 録音watchと通信watch/合成Storeを**constructor前**に登録し全0を確認。通常構成は実new_with_storage、試用構成は実new_early_trial_with（既存fresh再利用可）。通常newのnative wrapper/diagnosticsまで動的実行したとは記録しない。両構成×Store設定有無を別caseまたは明示subcaseで区別する |
| ET-06 / 設定表示 | 同じappでgo_to_qwen_settings→実frame/接続panel→既存編集ボタン経由のeditor生成。factory後に別editorへ置換しない。表示/編集を開いた直後に8個全0・録音0、未確認状態、合成Store読取を確認 |
| ET-06 / 設定保存 | 実app内editor.save/finish_qwen_settingsと一般save_settingsを通す。設定なしcaseもここで合成key/hostを明示入力して保存する。各保存直後に全0、合成Store saves/内容、一般設定のdisk反映を確認 |
| ET-04/06 / JSON復元 | 同appの実restoreへ非history候補を渡し、成功後のmemory/diskと全0を確認。試用history拒否は既存exact本文/不変ledgerを併用し全0。通常版へ試用拒否期待値を流用しない |
| ET-06 / 音読準備 | 実open_qwen_readingでapp.qwen_dialogがSomeになったこと、Inputで全0を確認。合成WAV設定→prepareのConfirmingでも全0、Cancel→同期終了後も全0。独立Dialogだけの試験ではこの行を満たさない |
| ET-06 / 終了 | 全5操作後と所有worker終了後に8個全0・録音0、audio_history=None維持、CODEX_HOME値不変を確認。guardを最後まで保持。API/認証ファイルの全面不変やnative成功を環境変数だけで推定しない |

未設定caseの音読prepareは、設定保存で合成設定を作ってから実行する。未設定のまま開く追加対照ではInputとprepareのInvalidKey/NotSentを確認する。未設定起動を設定済み起動の代用にはしない。各操作直後の累積snapshotは0であり、差分だけを比較して先行する送信を見逃さない。旧Wire/Probeの単体証拠はその範囲で保持し、factory後に生成した独立Dialogの0を新表へ流用しない。

不採用はspy生成順だけの移動、Config生成回数、静的非到達だけによる合格、lib内cfg(test)、globalの毎send再検索でguard終了後に実transportへ戻るwrapperである。最後の案はworker生存中の解除と競合し実送信へ流れるため、同Arcを保持する拒否専用wrapperを選んだ。広い通信framework/公開fault/製品エラー型変更はこの受入に不要である。

次は親が本書hash/同期plan/正本/下表を固定して独立Preへ渡す。designer→Pre（reviewer Astra/high）→独立作者Astra/high→実装者Astra/high→runner Sol/medium→独立Post Astra/highの割当を保持する。API不在はcompile blockerとして返し、G0未解除で行動REDを作らない。作者は実名と構成別件数、8入口/設定有無/5操作/寿命の対応を固定する。実装者は4fileだけを接合し、親は唯一のET2-G0を適用する。runnerは解除後の実名/exit/timeout/worker終了/残存と新snapshotを報告する。感度はG0第4項だけを使う。独立Post前にET2-P01をClosedへしない。

最小再検証は新監視全case/新ET06 matrix、qwen_settings/qwen_reading既存bin回帰、同binのET2起動/復元/録音安全集合、aiのET1 config試験、fmt/diffである。変更なしlibの証拠はsource/依存/構成一致を照合した範囲のみ接合する。ET3のall-targets/release/native gateは不変である。合成Errで観測失敗を隠す、未joinをpendingなしで正当化する、設定済み起動をfactory後代入で置換することは禁止する。

### Pre入力snapshot、未検証と測定

HEADは `9a7eb5c94041f0937f8bc3cfac35bdee2f8280e1`。mainはC1実行失敗後の作者fixture是正済みであり、C1の全source/4EXEへ凍結したままとは扱わない。次表は本節作成時の読取snapshotで、将来の実装source凍結とは別である。

| file | SHA256 |
| --- | --- |
| src/main.rs（作者fixture是正後） | `66BDD7137811C5C954A12C8C1E5461D5AF8933DC832D2282F2F37ACDE45C2025` |
| src/app.rs | `2AA13D474D8B8F2F6ED47B1D6E57FC75BA342C6C2822490927E38793860C3C56` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `D847555998E71987E8519D8B8B4811D68231054A043A365F3D040BA0339E11C8` |
| src/app/qwen_settings_tests.rs | `45EF01F0F3DB4A2302AE8DAF9FC10834190D427AB648823139B46DC49DE324F1` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C` |
| src/app/qwen_reading_ui.rs | `0463767500979C625724F065F54D4B1C7F525BC7688F7BD38FC40241CBFCB41D` |

現時点で追加製品判断はない。Store選択の所有不足はplanner/親受領で解消した。残る危険は接合漏れ、poisonをNoneに扱う誤り、保持Transportからの復帰、runtime背景終了、fixture/構成の誤帰属であり、上記Gateと独立oracleへ対応した。これらの実装compile・動的実証・Postは未了である。新しい通信入口/追加所有/終了不能worker/正本矛盾はSTOPして親へ返す。rollbackは各ownerが本Packet差分だけを個別patchで撤回し、ET2-P01を未解消へ戻す。C1本体・作者是正・実データ・既存EXEを巻き戻さない。

担当threadは `01a13975-8bd4-7d32-84b7-1ce794b5d6ca`（`/root/trial_egress_design`）。指定Astra/high、実行model/effort返却metadataは未取得。開始側の取得済みtoken snapshotは2026-10-14T08:09:41.886Z、input88977/cached61440/output363/reasoning13であり、開始直後までの欠測がある。終了値と本書hashはhandoffへ返す。親子包含は未確認、週枠は親測定へ接合し、usage/currentは編集しない。検証は文書diff/参照・実型/consumer照合・入力hashの読取のみ、source/test編集・build/runtime・再委譲・公開は0である。
