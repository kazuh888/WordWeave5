# I5-CANCEL-D1: 取消し試験の対象同定・安全回収診断

2026-10-07 / ACOUSTIC-HISTORY-001。限定診断PacketのPre-gate提出である。取消し不具合の原因確定、M2 Post-gate PASS、M1再受入を意味しない。

## 1. Objective / approved input

AH-AC09の「所有child/子孫を回収し、兄弟process・原音を保持する」期待値を変えず、全unitで一度観測したWAIT_TIMEOUTと、その後のfixture二重panicを区別して記録する。次の再現試験に先立ち、所有handle同定・非panic回収・失敗の固定を成立させる。承認済み[spec](spec.md)、[implementation](implementation.md) I5、親の限定診断依頼に従う。

planner指定designer=`gpt-6-astra/high`、親モデル/limits/gatesは変更しない。AGENTS、development-team、KPI、api-and-interface-designを読了済みであり、型・所有・失敗境界へ適用した。UI非該当である。本役の所有は本書のみ、本体・試験・既存M2/M1文書を編集しない。開始/終了counter、実行model/effort metadataは未取得である。

## 2. Evidence / unresolved hypotheses

[runner報告](praat-m2-runner.md)と[全unitログ](artifacts/m2-runner-praat.log)によれば、`process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data`の172行でWaitForSingleObject(handle,0)=258、期待0。そのassertはhandles mutex guardの生存中に実行され、201行Dropのlock().unwrap()がPoisonErrorで再panicしてabortした。**この二重panic経路はコードとログで確認済み**である。

対象handleはVec<isize>だけでroot/孫/取得元の記録がなく特定できない。watchdogと取消しthreadはそれぞれ`grandchild.pid`を読みOpenProcessする。childは通常fileへfs::writeでPID文字列を通知する。短いprefixが整数として読める可能性は仮説であり、今回の誤同定が起きた証拠ではない。Hidden/consoleなしの単独試験各1PASSも全unit失敗の原因を確定しない。

本体stop_innerはTerminateJobObject→root handle無期限wait→ActiveProcesses=0待ちである。[TerminateJobObject](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-terminatejobobject)の成功だけと、[WaitForSingleObject](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject)のsignal観測を同一視しない。JobActive=0と各独立handleのsignalの厳密な観測順序は未確認であり、「Windowsの遅延が原因」と断定しない。

## 3. Chosen scope / owners

**D1は隔離crateコピーのfixture 2本だけを変える**。先に本体へtelemetryを追加する案は選ばない。コピー先は `tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate`。Pre-gate PASS後に指定実装者が元crateのsrc/tests/scripts/Cargo.toml/Cargo.lockだけを複製し、元targetをコピーせず独立targetを生成する。親は相対path/hash一覧と原本不変を確認する。copy先が既存なら上書きせず、その由来を親が確認する。

| copy内source / owner | 変更責務 |
| --- | --- |
| tests/support/process_unit.rs / test author Astra/high | 取得元付きhandle管理、identity/通知検証、最初のwait結果記録、lock外assert、poison耐性回収、独立fixture回帰 |
| tests/support/praat_child.rs / 同じtest author | 所有Child handle由来のidentity通知を追加。既存tree動作・PID通知・15秒fallbackを保持 |

prep役からtest authorへ逐次引継ぎ、fixture作者は1役だけである。独立runner=`gpt-6.1-sol/medium`、reviewer=`gpt-6-astra/high`を維持する。src/process.rs、src/process_windows.rs、M2入りpraat.rs、manifest/lock/scriptはcopy内でも不変。M1 test本体・oracle・watchdog仕様は編集しない。再委任・実データ・API・UAC・公開なし。

## 4. Typed identity / notification contract

fixture内部に`OwnedProcessRecord`を置く。値はowned HANDLE、call世代、role（Root/Descendant/ExplicitOwned）、origin（AfterCreate/CancelObserver/Watchdog/ExplicitTrack）、通知PID、GetProcessId結果、creation FILETIMEの64bit値、採取順序番号である。handleは取得からcloseまで一つのrecordが所有し、同じ孫を別observerが開いた場合もorigin別recordとして区別する。既存`track(pid)`は互換wrapperとして維持し、現在のrootとPID/作成時刻が一致すればRoot/ExplicitTrack、run外の既存M1 observerのようなcaller所有processはExplicitOwned/ExplicitTrackとして記録する。M1の呼出/期待値を変更しない。serial guardとregistry guardを混同しない。

rootは既存AfterCreateのPIDを使い、suspended中にOpenProcessし、PID・作成時刻を取得して登録する。孫についてはroot fixtureが保持する`std::process::Child`の実handleからPID・作成時刻を取得する。取得根拠は[GetProcessId](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocessid)と[GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)である。時刻の表示変換はせず64bit FILETIMEを比較する。

孫通知は既存`grandchild.pid`へのfs::writeを観測対象として残す。その**前**に新しい`grandchild.identity`をcreate_new一時file→write_all→flush→close→非既存final名へのrenameでpublishする。内容は固定ASCII `WWCHILD1\t<root_pid>\t<root_created_u64>\t<child_pid>\t<child_created_u64>\n`、上限256bytes、一行・固定5fields・正の10進値・余剰bytesなし。publisherはroot自身のhandleと所有Child handleから構成する。専用temp dirが通知の呼出境界であり、実データのID/秘密を入れない。旧PID通知の書込み順や分割を改変して仮説を隠さない。

両observerは共通のbounded decoderを使う。identityのroot PID/作成時刻がAfterCreateで登録したrootと一致し、`grandchild.pid`の**全bytes**がidentity.child_pidの10進表記と一致した場合だけ孫OpenProcessを試す。部分通知・不正frame・root不一致を記録し、prefixのparse成功を完全通知としない。取得したhandleのPID/作成時刻がidentityと一致した場合だけ所有registryへ登録する。不一致/照会失敗ならそのhandleはcloseのみ、TerminateProcessしない。誤通知のPIDをprocess全探索で補完しない。

この同定は許諾済み合成fixture内の所有関係を確認するもので、悪意あるprocessが通知やmemoryを書き換える環境の認証方式ではない。registry登録後はPIDから再取得せず、保持handleでwait/救済する。

## 5. State / synchronization / data ownership

`Created → RootTracked → IdentityPublished → DescendantTracked → CancelRequested → SupervisorReturned → FirstWaitLatched → Reported → RescueIfNeeded → Closed`を区別する。取消しthreadは孫handleを登録してからdescendant_readyを立てcancelする。既存2秒ready budgetを超えた場合もcancelして終了させ、ready不足はfixture不成立として保持する。

registryは短時間mutexでrecordの追加とsnapshot取得だけを行う。OpenProcess/照会失敗のassert、wait結果のassert、ログI/O、TerminateProcess、bounded waitをmutex保持中に行わない。snapshotのhandleはfixture終了まで閉じず、watchdogはclose権を持たない。Dropはwatchdogを停止・joinした後にregistryをtakeし、一度だけcloseする。

`Fixture::run`入口でchecked incrementした世代1,2,…を設定する。run外の明示trackは世代0とする。AfterCreateで現在世代のroot identityを確定し、取消しobserver/watchdogの通知はそのrootと一致する世代にだけ登録する。旧世代の通知を新しい孫として採用しない。重複trackをdedupして呼出を消さず、独立して取得したhandleを別recordとしてすべて保持する。終了確認済みrecordも次run開始時には解放せず、過去の初回観測を保存してFixture Dropで一度だけcloseする。次runのsnapshotは旧recordも0msで照合するが、旧世代の初回値を上書きしない。今回の既存tree caseは1 Fixtureにつき1 tree起動であり、既存の複数runはecho成功→hang失敗と起動前拒否2回である。

正常/失敗の全経路でlock取得はpoisonをrecoverし、`poison_seen`を失敗証拠として記録する。poisonを回復したことを成功へ読み替えない。Drop内はunwrap/expect/assertを使わず、stderr出力もfallible writeの結果を捨てる形とし二重panicを起こさない。採取失敗・外側thread panic・回収失敗は固定reasonとして残す。

## 6. First observation / error contract

supervisorから戻った直後、registry snapshotの各handleへ**一回のWaitForSingleObject(handle,0)**を行い、role/origin/IDと採取前後の単調時刻、戻り値（WAIT_FAILEDなら直後のOS error）を保存する。全初回waitの採取まではログ出力・追加identity照会・救済を挟まない。短いsnapshot取得以外の待ちを追加せず、0ms oracleは維持する。取消しtree case（descendant_readyを要求するcase）のsnapshotにはcancel前に登録したrootと孫が必須である。共有Fixtureのargv/output/event等のroot-only caseには孫必須を課さない。

first_wait、supervisor Result(kind/reason)、`process::reap_unconfirmed()`の呼出前/返却直後値、descendant_ready、poison_seen、watchdog_rescuedを不可逆にlatchし、その後にログ・assertへ進む。既存crate-private accessorを読むだけで、flagをresetしない。supervisorはchild.stop()のErr後もcancel checkpointを返し得るため、Cancelledという返却値と回収未確認flagを別項目として扱う。raw stdout/stderrや秘密は記録しない。WAIT_TIMEOUT/WAIT_FAILED/未同定/未通知をPASSへ変えない。救済前後のwaitとTerminateProcess結果は別fieldsに保存し、後のsignalで初回値を上書きしない。ログには期待値との比較と失敗recordをすべて出し、root/孫/重複observerのどれが失敗したかを識別可能にする。

既存のCancelled、兄弟生存、保存済み合成bytes不変というtest期待値は変更しない。fixture回収のためのpanic捕捉をsupervisor成功へ変換しない。観測コードによる時間配置の変化が再現率を変え得るため、D1で未再現でも旧failureを否定しない。

## 7. Cleanup / cancellation / bounded resources

8秒watchdog・15秒child自己終了・既存Dropの2秒救済wait、testの4秒deadlineとreadyの2秒を保持する。固定sleepやwait延長で期待値を吸収しない。watchdogの既存poll間隔は通知探索の待機であり、supervisor戻り後の合否waitへ持ち込まない。

watchdog/Dropは同定済みowned handlesだけを対象にする。救済を開始する前にrescue flagを立て、terminate結果と救済waitを別記録する。外部process探索・名前一致kill・未同定PIDのkillは禁止する。取得前の孫については既存supervisor Jobとfixtureの15秒自己終了を安全網とし、期限内に所有同定できなかった場合を成功としない。

追加診断の上限は**1 Fixtureあたり16 owned-handle records、16 call世代、256 event records**、通知buffer256bytesとする。これは製品上限の変更ではなく診断fixtureの保持量である。上限到達を成功/切捨てにせず固定`DiagnosticCapacityExceeded`としてlock外で失敗させ、既に所有するhandlesを通常回収する。登録競合で上限に入らなかった新handleはcloseし、過去recordを追い出さない。通知poll全件を蓄積せず、最初の不完全/不一致、完全通知、登録、初回wait、救済という遷移だけを記録する。

収容根拠は原本process_unit.rsの304行付近（1 run内の自動AfterCreate track1回＋明示root track4回=5 records）、803行付近（12反復それぞれ新Fixtureで2 run=2 root records）、761行付近（同Fixtureの起動前拒否2回、created handle0）、取消しtree（root1＋孫observer2=3）である。M1外側observerの明示track2回も16内である。各Fixtureは最大2 run、実例最大5 recordsであり、保守的に2 run×（root5＋孫2）=14も収容する。12反復を同Fixtureへ統合したり、反復数/handle-count許容+2を緩和しない。原音・履歴・schemaは触らず、診断logsと合成tempのみを保存する。cleanup未確認のtempは残存として報告し、未知dirを削除しない。

## 8. Test seams / independent checks before reproduction

本体のcfg(test) hookは既存AfterCreateを利用し、D1では追加しない。fixture内部の通知decoderとrecord照合、初回waitの評価/救済記録の境界だけをテスト可能にする。独立authorは次を本番取消し再実行より先に確認する。

| D1 ID / AC | 独立期待値 |
| --- | --- |
| D1-A / AH-AC09 | PIDの1桁prefix・空・余剰・途中identity・root/creation不一致を手書きbytesで拒否し、所有登録/TerminateProcess=0。完全通知の既知値だけ受理 |
| D1-B / AH-AC09 | 自分で作った合成childのhandleと異なるcreation値を渡して照合拒否。unknown handleはcloseのみ。fixture外processを試験対象にしない |
| D1-C / AH-AC09 | 意図的なregistry poisonをcatchしたfixtureでDropが再panicせず、既に登録済みの自分のchildを回収する。外側小processでexit/回収ログを確認しabortなし |
| D1-D / AH-AC09 | 自分の合成live childで初回WAIT_TIMEOUTを実観測し、記録後に救済する。初回失敗は残り、救済成功が元caseのPASSを作らない。これはfixture自己試験であり製品取消しfailureの再現とは呼ばない |
| D1-E / AH-AC09 | 同Fixture2 runと同root5 trackを安全確認で収容し、世代・独立handle数・過去初回観測保持・最終一回closeを照合する。既存12反復/24 callの漏洩試験と起動前拒否2回を元の期待値で実行する。root-onlyに孫必須を課さず、旧root identityの通知を新世代へ登録しない。16 records/16世代/256events超過は診断失敗となり、所有回収されることをfixture自己試験で確認する |

旧fixtureのpoison→abortは既存ログをRED証拠とし、危険な旧fixtureを再実行して証拠を作り直さない。D1-C/D1-Dの合格は「失敗を安全に失敗として返せる」ことの検証である。既存取消しoracleの期待値を調整してGREENにするものではない。

## 9. Reproduction / evidence gates

Pre-gate PASS→copy原本照合→fixture自己試験→独立reviewの安全確認→runnerがconsoleなし・非管理者で同じ全unit順を一回実行、という順序とする。失敗した条件を含めるため本体M2の15件も含むlib全体を使用し、単独成功で代替しない。compile・EXE hashを固定してから実行し、全unitの外側診断打切り監視は初期設計値180秒とする。根拠は既測M2 15件約51秒（50.96秒）と既存process約4秒に診断実行の余裕を加えたもので、build時間は含めない。M1 exact用の45秒を全unitへ適用しない。これはPre-gate前の設計訂正であり、既存失敗を成功へ変える事後延長ではない。試験内部の2/4/8/15秒、既存Dropの2秒救済wait、製品共通60秒は変更しない。180秒到達または救済作動時は失敗/未検証として固定し、時間を延ばして成功を取りに行かない。

```text
cargo build --manifest-path tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate/Cargo.toml --features praat --bin acoustic-test-child --locked --offline
cargo test --manifest-path tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate/Cargo.toml --features praat --lib --no-run --locked --offline
```

runnerは出力された**当該copyのunit EXE**に`--skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --test-threads=1 --nocapture`を渡す。M1 configured filter1をPASS件数へ足さない。self-test名・実件数はauthorが提出時に固定する。**必須lib全unit自体が実Praatを使うM2 15件を含むため、起動前に必ず`WW_TEST_PRAAT_EXE`を既存固定原本EXEの絶対pathへ設定する**。使用pathは `F:/Kazuhiro/GitHub/WordWeave5/tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe` とし、絶対path/存在/size=96,780,624/SHA-256=`22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4`を起動前に確認する。M2 caseの既存固定constructor検証も通過させる。不在/不一致はfixture不成立で停止し、copyのCARGO_MANIFEST_DIRに基づくdefault相対探索へ落とさない。source書換え、binaryのcopy先への複製、downloadで代替しない。環境設定は当該診断processへの明示値として記録し、利用者の永続環境設定を変更しない。

記録はcopy/原本source hashes、unit/child EXE hash、環境（console/外側Job/非管理者）、実ケース/結果/未到達数、初回と救済の時系列、観測失敗のreason、watchdog作動、残存ledgerである。未再現時は「D1条件で未再現」とし、M2全gateは未解決のまま親へ返す。自動反復、追加filter、原因が定まらないままの期待値修正はしない。

## 10. M1 / original gate isolation

M1のClosedは元source/元oracle/既存利用者GREENの履歴として保持する。D1はcopyのprocess_unit.rsとpraat_child.rsのhashが変わるため、**変更後診断EXEのM1合格へ旧GREENを転用しない**。M1 oracle本体を変更しないことと、新しいsource/EXEが未受入であることを分ける。D1から元fixtureへ修正を採用する際は別handoffとPre-gateを設け、M1の既存外部条件での再確認が必要になることを明記する。[M2設計](praat-m2-design.md)の「関連source/fixture不変時だけ外部GREEN再利用」を緩めない。

元sourceの開始hashはprocess.rs=`9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54`、process_windows.rs=`10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C`、process_unit.rs=`CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896`、praat_child.rs=`8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228`で一致を確認した。その他原本はrunner記録と親snapshotで固定し、終了時に再照合する。

D1成功だけで元の全unit failure、fmt failure、M2 Post-gateをClosedへ移さない。fixture改修を元へ採用する判断と、本体取消し修正の必要性は診断結果に基づき独立reviewへ返す。

## 11. Escalation boundary / rejected expansion

同定済みのroot/孫handleで初回WAIT_TIMEOUTが再現した場合、D1証拠だけでは専用Jobの状態が取得できない。`IsProcessInJob(handle,NULL)`は「どれかのJob」を示すだけで、専用Job所属の代替証拠にしない。そこで止め、**別D2**としてprocess.rs/process_windows.rsのper-call cfg(test)診断を計画者へ返す。必要情報は専用Job所属、TerminateJobObjectの結果、root waitの値、ActiveProcesses観測値と各時刻であり、既存終了順/期待waitを変えない。

D2は本体source hashが変わりM1の再確認範囲にも及ぶため、本書で実装を許可しない。親のscope割当・具体設計・独立Pre-gateが必要である。この追加統制は未確定OS仮説を根拠に本体を変更しないためであり、新しい利用者仕様の判断や上限緩和を求めるものではない。D1でprocess source変更・新依存・上限変更が必要と分かった場合も同様に停止する。

## 12. Safety-review cycle 1 追補 — 登録sealとcleanup確証

2026-10-07。初回の独立安全reviewはCHANGES_REQUIREDであり、全unit再現は未許可である。§4–7の不足を以下で具体化する。D2・製品process変更・期限変更は含めない。追補の差分Pre PASS後に隔離fixtureだけを修正し、関連自己試験後に再安全reviewを受ける。

### 12.1 初回snapshotの固定

`first_waits`はregistryの短mutex区間で現在世代をsealし、同時にその時点の全owned recordsのsnapshotを取得する。lock解放直後に全0ms waitを採取する。join・追加wait・ログ・identity照会を初回採取前に挟まない。過去世代の初回値は維持する。

observerのtrackはOpenProcess前と、取得handleをregistryへ挿入するlock内の双方で世代/root/sealを確認する。seal後は新規OpenProcessをせず、既に取得中だったhandleが挿入時にsealを検出した場合はclose-onlyとする。この未登録handleは§4の「登録した独立所有handleをDropまで保持」の対象外である。既存recordを除去せず、TerminateProcessを行わない。固定reason `RegistrationSealed` を記録するが、seal前に必要なrootとCancelObserver孫が登録されている正常caseをlate observerだけで不合格にしない。必要record不足は従来どおり不合格である。

次のrunはchecked generation更新と同じlock内で新世代の登録を開く。旧世代の途中取得は新世代へ挿入できない。run外の明示trackと既存M1 oracleは変更しない。

自己試験は、seal前に取得した合成owned handleをseal後の挿入境界へ渡しclose-only・terminate0・records不変を確認する。seal後の新規observerはOpenProcess試行0とする。snapshot cohortの全first_wait保持、旧世代拒否、次世代root受付も確認する。タイミング依存のsleepではなくfixture内の決定的境界で検証する。

### 12.2 cleanup確証とtemp保持

Dropは既存のwatchdog停止/join後、同定済みowned handlesを救済・確認してcloseする。次のいずれかがあればcleanup未確認としてtemp自動削除を無効化し、専用tempの絶対pathと固定reasonをfallibleログへ残す。

- 救済後にもWAIT_TIMEOUT/WAIT_FAILED等で終了を確認できない。
- watchdogがpanicした。
- `process::reap_unconfirmed()`がtrueである。
- tree/tree-exitの起動を観測したが孫を同定できず、全所有子孫の回収を確認できない。

poison/容量超過そのものとcleanup確証は別状態とする。poisonがあっても全owned handle終了・子孫範囲の確認が揃えばtemp削除は可能だが、元のtest failureは消さない。未同定processの探索・kill・tempの後追い削除は行わない。TempDir::disable_cleanupを使用できることはローカル依存tempfile 3.27.0で作者が確認した。

自己試験は副作用を起こす実terminate失敗を捏造せず、fixture内のcleanup判定境界に未確認reasonを与えて保持path/bytesを確認する。既知の自己所有tempだけを試験終了時に通常cleanupしてよい。全回収確認済みpoison caseで不要な保持をしない対照も確認する。実製品の回収失敗を再現した証拠とは扱わない。

### 12.3 固定と再検証

修正前の成功ログ・hashを残し、新しいsource/unit/child hash、変更関連自己試験の実件数・結果を別ログに記録する。既存24callと起動前拒否のfixture共有部分にも関連するため、その2caseは再実行する。原本17件不変・copy変更2本・M1 oracle不変を再照合し、再安全review PASSまで全unit診断を実行しない。

## 13. Completion / handoff

設計完了は本書・参照・原本hashの確認まで、D1完了は安全自己試験と一回の限定再現、その同定証拠/未再現範囲、独立診断判定までである。成果物は `artifacts/cancel-d1-*` 配下のcopy差分・logs・hash・短い結果に限定する。大量ログは本書へ複製しない。既存失敗ログと元snapshotは保持する。

Pre-gate reviewerへ本書を提出し、PASS後にprep役→fixture test author→独立安全確認→runner→reviewerの順で引き渡す。親は原本process関連4sourceとunit/child EXEを`artifacts/cancel-before`へ保存済みである。未決の方式選択0。未再現/同定失敗/回収失敗を観測可能な終端として返し、原因解決済みとしない。使用量counterの欠測は親へ返す。追加source実装・試験実行・公開は本設計役では行っていない。
