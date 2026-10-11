# I2a-P 独立試験 — source固定・静的review待ち

2026-10-08 / ACOUSTIC-HISTORY-001。`ww-test-author / gpt-6-astra / high` の計画者割当を受領した。実行model/effort metadataと開始/終了usage counterは未取得。AGENTS/development-team/KPI、test-driven-development skillの作者責務を適用する。実装修正・再委任・実データ/API・UAC/ACL/OS/harness変更・公開は対象外である。

## 入力・所有・完了境界

P finalPre PASS/阻止0を [独立設計review](history-commit-design-review.md) から受領した。入力は [P限定設計](history-pointer-commit-design.md) SHA `13DF70DAD3C247E83AACC384169B507145464C65E40007D347FCD59E4AE1891A`、[正本](design-i2.md) SHA `E53C18E88DCDD23F7FC843A3A363F4FE4108E0B8BE2F0689CE8DE30D18F568D7` §4/7、[計画](implementation.md) P/V-P、[scaffold記録](history-pointer-commit-implementation.md) である。V Closedの101 PASSと実helper完成を前提とするが、Pの実行成功へ転用しない。

所有は `src/store.rs::i2_pointer_commit_tests` 新cfg(test)節と新 `tests/audio_history_commit.rs` の2か所だけである。最小enum/3口以外のAPI・可視性・seamは追加しない。既存本体/旧oracleを変更せず、parent before snapshotも上書きしていない。新lib18件＋公開integration4件＝22件を作成・整形した。**cargo/compile/testは未実施**。source/build所有を親へ返却し、独立静的確認待ちである。以後は本記録だけを追記する。

## fixtureとoracle

V/I1の手製Plan/trial/WAV/receipt構造とSの全field sentinelを限定再利用した。期待rootは元rootをcloneして明示的にrevision/trial/sequence/receiptを変更し、元の無関係Trial/receiptを保持する。production mutation planやpointer writerから期待値を採取しない。原音・Trial・rootの3object基本closure、strict intentのexact mutationと予定refsを事前配置する。lib正常fixtureには完成V verify成功とconfirm Busyの先行対照を置き、stub失敗と壊れたfixtureを区別する。

旧Progressは手製raw wire1、全legacy top-level fieldを含むwire2、正常root配下のleaf欠損を使い分ける。pretty next期待bytesはfixtureの旧snapshot DTOをcloneしてaudio_historyだけ変更した全体serializeであり、production encode helperを呼ばない。f32をValue往復比較で別丸めしない。raw v1はUTF-8 550 bytes、独立.NET SHA-256 `a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989` を固定し、runtime2再serializeを旧退避期待値にしない。

ledgerはhost/neighbor、Progress/deck、assets、intent、backup/temp、directoryを含む。成功ではProgressだけを期待newへ置換、wire1では所定hash退避とbackups dirだけ追加する。pointerの日別backupなしも確認する。結果不明ではfixture自身の第三bytes/削除/dir化/pending/破損を期待ledgerへ明示的に反映し、productionによるrollback/追加残存と混同しない。

## case構成

全case名は [一覧](artifacts/history-pointer-commit-tests-cases.log) に固定した。lib prefixは `store::i2_pointer_commit_tests::` である。

| 群 | 件数（lib＋integration） | 固定した判定 |
| --- | --- | --- |
| P01 | 1＋1 | public/private limits成功、None/Some base、最新全非履歴field保持、rootだけ変更、exact保存bytes・再open・receipt・全ledger |
| P02 | 2＋1 | closure健全な無関係Trial/receipt脱落next、request/base/intent差替え、実Trial欠損、未知intent、pending。Running/Sending SetTrashは既存confirmとV verifyのBusy対照後、新入口もBusy/publisher0/ledger不変 |
| P03 | 2＋1 | next/後続root×intent有無×取消済み、既commitはM/fault/publisher0。receipt request/affected/revisionと残存未知intentの拒否。公開finish故障後の同operation再試行、cleanup後の後続root再試行でID/receipt/媒体不増不減 |
| P04 | 4＋0 | no-write Ok→Busy/Err→Io、実atomic newのOk/Err＋After fault→Committed。Before faultでもold/new実再読。Before Ok後のsnapshot競合→Conflict/no block。実share0中atomic失敗→old/Io、全assert後に同path/同bytesのunlock成功対照 |
| P05 | 1＋0 | 8子fixture：後続rootの第三raw、同root非履歴だけ第三raw、既存file消失、leaf directory、実read拒否、new closure欠損、pending出現、Before Errで第三raw。Io＋normal/material/I1/history pointer全writer停止、別rootも停止、pure readとledger、rollbackなし |
| P06 | 1＋1 | 事前取消/Beforeで取消はBeforeだけ・publisher0・Cancelled、publisher開始後の取消は実new優先Committed。同operation再試行で媒体/intent保持 |
| P07 | 3＋0 | wire1 exact hash退避・Migration2→Pointer2、M前後fault、M後取消/Before faultの退避残存。旧Noneのno-write Ok/Err対照、next exact size境界、未知/破損/旧size超過/unsafe leaf/既存不一致退避のtyped拒否 |
| P08 | 4＋0 | 同thread再入Busyとnested normal/material拒否、通常保存guard待機後の最新非履歴field保持、same/distinct rootの実pointer/normal区間、material公開alias＋I1 historyとの共通G競合とデータ非混入 |

block/再入/競合はsameEXE exact1case＋mode環境値＋成功marker、15秒有限childで隔離し、親globalを汚染しない。guardをthread間moveしない。channel/barrierで実区間の開始・解除・終了を観測し、単に「何ms何も起きない」を証拠にしない。競合worker結果をjoinしてから判定し、stub Ioによるpublisher未到達をchannel切断だけのfixture失敗に読み替えない。reset/poison解除口なし。

## 未検証と決定的観測限界

- 今は静的作成段階であり、正常fixture・共有拒否・RED・cleanupの実成立を主張しない。固定Ioで後段が未到達なら、その後段の全ledger/解除後対照/loopをGREENで最後まで確認する。
- 実read拒否はread-probeのsharing32とunlock成功を先に確認し、publisher後も実read32を確認する。実atomic失敗はその実戻り値を観測し、OS rename raw番号を32に固定せず、同path/同bytes解除後成功の対照を全製品assertの後へ分離する。
- materialのprivate callbackはstore testから公開せず、公開material(alias path)完了・最終field/root保持・pendingなしを、新P/I1の実区間と既存G/Tのmaterial gate証拠へ接合する。material内部区間を新seamで直接注入したとは主張しない。
- 最後の取消Acquire直後の微小窓は決定注入しない。Beforeでの取消とpublisher開始後の取消は別の観測境界である。
- V内部のhead/intent最終再読間置換、pending metadata Io、closure reader実100k nodes、電源断/kill耐久性、UI/解析/POSTは未測定である。read32、recovery inventory100k entries、同期pointer試験で代替しない。
- app.lockを保持するStorageのroot/祖先消失の全OSパターンは未注入であり、旧Noneの安全なleaf欠損とleaf dir/実read不能を混同しない。OS/ACL変更やraceで到達を作らない。

## 固定hash・静的結果

[hash/件数/fmt](artifacts/history-pointer-commit-tests-fixed.log)、[source一覧](artifacts/history-pointer-commit-tests-source-fixed.json) を保存した。

- store全文: `097C0C3356A3E0C3049E65C52F16B57EF7D0AF9CE6B8B86310ECDE0C54DFFF11`
- 新P module: `4C6340F5209A2546FADCA66B815965EDEA3CAC972F7D6B54CB2B7E975BA3F911`
- 新integration: `5CA5A171084BDD668D43E2DA85920D0655A9D4DA5D81316B4D349CB48697FFEC`
- 新節前prefix: `F1E5DF2F27CDCA9F0C3DB7D364B77688CC90DA9D4900189D197C5D9AFEC20A06`（受領scaffold全文と一致、節間末尾空行のみTrimEnd＋LF）
- 旧test suffix: `1FB68B058230B246889833D2D39B308812DD5E20DE83E2BA239AAA1124C9FE97`（受領時と一致）

`rustfmt --edition 2021 --check src/store.rs tests/audio_history_commit.rs` と `git diff --check` は成功。新case以外の既存期待値は改訂していない。

## 静的PASS後の実行対象案

親が静的PASSと実行packetを確定後、lib＋新audio_history_commit＋既存gate/save/material/I1store/recoveryを必要targetとして1回集約no-runする。固定EXEの `--list` で新P22、既存各prefix/targetの実件数と0件でないことを確認し、実行結果を分離する。compile error・fixture failureは行動REDに数えない。

影響根拠は、Pの新writerがG/通常save/教材save/I1 writerと排他を共有し、S codec/M退避/V verify・confirmへ接続することである。現scaffoldは未接続fixedIo、新節前全文が同一であり、RED段階では旧source/期待値不変とcompile証拠を本体後の関連回帰成功へ読み替えない。既存受入gateを作者判断で削除せず、実行対象は親へ返す。

recovery inventoryのR2はVの直前独立成功があり、同じ件数を理由なく反復しない。P本体接合後の影響と指定回帰gateによりR2を実行する場合、当初から外側900秒（既存20分以内）を用い、自動延長しない。製品解析子60秒/512MiBと別である。旧ログを今回の新実行と称しない。

## compile前の独立静的指摘（未修正）

親からP2を受領した。P08競合caseの `writer.prepare(op(7), h.plan.base.clone(), &request)` は公開引数 `Option<&RootRef>` に所有型 `Option<RootRef>` を渡している。`h.plan.base.as_ref()` が最小訂正である。現在はreview継続中であり、指摘集約と限定訂正所有を待つ。上記source/hashを変更せず、cargoも実施していない。これは静的型不一致であって行動REDではない。

### P2限定訂正・再確認待ち

独立静的最終結果は当P2だけのCHANGES_REQUIREDで、他の22件に追加阻止なしを受領した。限定所有を受け、当1式だけを `h.plan.base.as_ref()` に訂正した。逆変換した全文SHAは訂正前 `097C0C3356A3E0C3049E65C52F16B57EF7D0AF9CE6B8B86310ECDE0C54DFFF11` と完全一致し、期待値・旧本体/旧test・その他新caseの不変を確認した。

- 訂正後store: `2660B8ECFB244339AD895E64A73252BBE23502A9ED174FB9BC3D94270287B5CE`
- 訂正後P module: `BA8E7018F8170A66400A0C81D75E9E6A9E0A99FCB026774062D9BD43C1953089`
- integration不変: `5CA5A171084BDD668D43E2DA85920D0655A9D4DA5D81316B4D349CB48697FFEC`

[限定訂正証拠](artifacts/history-pointer-commit-tests-p2-fix.log) のfmt/diff checkは成功した。cargo未実施、compile/行動RED未到達である。source/build所有を親へ返却し、reviewerの限定再確認後にcompileへ進む。

## 初回aggregate compile不成立（行動REDではない）

P2静的解消PASS後、親の限定packetに従い `cargo test --locked --no-run --lib --test audio_history_commit --message-format=json` を1回実行した。今回は新P22だけをRED実行対象とし、旧行動回帰は実施しない承認である。本体後には指定関連回帰を独立runnerが実施し、inventory34の外側900秒を含む。旧回帰gateを削除したものではない。

初回buildはexit101、timeoutなし、161.832秒で終了した。[build結果](artifacts/history-pointer-commit-tests-build-result.log)、[JSON](artifacts/history-pointer-commit-tests-build.jsonl)、[stderr](artifacts/history-pointer-commit-tests-build.stderr.log)、[error抽出](artifacts/history-pointer-commit-tests-build-errors.log) を保持する。唯一のerrorは `E0433 src/store.rs:4403 AnalysisProfile::v1()` の未importである。型は `acoustic_analysis::AnalysisProfile` であり、再利用元のclosure testでは明示importされていた。作者と先行静的reviewで見逃した型名解決不成立であって、製品の行動REDやfixture失敗ではない。

最小修正案は新P cfg(test)節内の `use acoustic_analysis::AnalysisProfile;` 1行である。現時点では修正していない。固定source全対象のhashは開始前と一致し、store `2660B8EC…287B5CE`・integration `5CA5A171…697FFEC` を維持する。build processは終了し、source/build所有を親へ返却した。試験実行は0件であり、RED/cleanup実証は未到達である。成功して生成されたpartial test EXEがあれば [partial EXE一覧](artifacts/history-pointer-commit-tests-build-partial-exes.json) にhashを保持し、後の不要再build/再実行を避ける対象判断へ渡す。

### import1行の限定訂正（再build前）

親から限定所有を受け、新P moduleへ `use acoustic_analysis::AnalysisProfile;` の1行だけ追加した。[訂正証拠](artifacts/history-pointer-commit-tests-import-fix.log) で当行を逆除去した全文SHAが旧 `2660B8ECFB244339AD895E64A73252BBE23502A9ED174FB9BC3D94270287B5CE` と完全一致し、他部/期待値不変を確認した。

- 訂正後store: `E9F281659E9EFF97892B611A2C384C56DEC1C251C6E09C1613CE8C4F991F6FD9`
- 訂正後P module: `ED2429AF0C23F351A1C088B11517830138BCDB025BC570A4F444CD9990A47808`
- integration source不変: `5CA5A171084BDD668D43E2DA85920D0655A9D4DA5D81316B4D349CB48697FFEC`
- 初回に生成されたintegration EXE `audio_history_commit-9eef11e266791940.exe`: `F03DDA59041D763598832D4F2FCA5B2458BB68C855F9D8E88F61158308C185B0`

fmt/diff成功、source/build所有を親へ返却した。修正後の再build/testはまだ実施していない。独立解消確認後は、cfg(test)限定importだけでproductionとintegrationが不変であることを根拠に、lib-only再compileと生成済integration EXEの利用が可能である。初回失敗・静的見逃し・未実行を隠さない。

## 2026-10-11 再開: lib compile成功、fixture不成立で停止

親からimport1行の独立静的解消とP限定再開を受領した。指定 `ww-test-author / gpt-6-astra / high` を維持し、source編集なしで実行した。今回の完了境界は新P22のRED分類であり、本体修正・旧回帰・後続単位は所有しない。実行model/effort metadataと開始/終了usage counterは未取得である。

固定10 source、P module、既生成integration EXEのSHAを開始時に照合した。`cargo test --locked --no-run --lib --message-format=json` は1回、exit0、154.403秒で成功した。incremental cacheのhard linkからcopyへのwarningを含むがcompile errorではない。新lib EXEは `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe`、SHA `223EE259FE60AD46295836FFDE10B4DD6B27E92157D23F491569D18A66264EFC`。integration EXEのSHAは前記 `F03DDA59…85B0` のままである。両EXEの `--list` はlib prefix18件＋integration4件、benchmark0を確認した。

実行は `& target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store::i2_pointer_commit_tests:: --nocapture --test-threads=1` の1回である。exit101、1.680秒、18 failed / 0 passed / 0 ignored / 261 filtered out。**行動RED17件、fixture不成立1件、integration4件未実行**に分類し、指定の停止条件に従って追加実行を止めた。子processの同一case実行を別の受入件数へ加算しない。

- 行動RED17件はP01–P08の固定Io返却、要求したtyped errorとIoの相違、またはpointer/fault callback到達0による失敗である。P05/P08の子processも該当行の失敗を確認した。P04の実atomic拒否/解除対照、P05のmode1以降・実read拒否・全writer block、各loopの後続modeと成功後ledgerは未到達であり、RED17をこれら全境界の実証へ読み替えない。
- fixture不成立は `p08_waiting_pointer_uses_nonhistory_updated_under_normal_save_guard`。子processは `src/store.rs:5229` の `entered_rx.recv_timeout(...).unwrap()` で `Disconnected` となった。pointer thread生成はこの行の後であり、PのfixedIoがこの停止を起こしたとはいえない。既存normal saveがpublisher前に終了したことまでは判明したが、normal workerの `Result<Progress, String>` はjoin前panicで出力されず、内部の失敗原因は未取得である。fixtureの正常V/confirm対照はこの行より前に通過している。
- 最小の診断修正案は、この1caseの最初の受信失敗分岐でrelease senderを解放してnormal workerをjoinし、その実戻り値をpanicへ含めることである。正常経路・期待値・productionは変えない。診断結果を得ずにfixtureのbase、保存limit、通常保存APIやassertを変更しない。現時点で診断修正・再build・再実行は行っていない。競合caseの他のworker結果もGREEN時に最後まで確認する必要がある。

新ログは `artifacts/history-pointer-commit-tests-resume-20261011-*` に保存した。`build-result.log` / `build.jsonl` / `build.stderr.log`、`lib-list.log` / `public-list.log`、`lib-red-result.log` / `lib-red.stdout.log` / `lib-red.stderr.log` が今回の実行証拠である。終了時は固定source10件とEXE2件のSHAが開始/生成時と一致した。source/build所有を親へ返却する。

**TEMP計測の欠測訂正**: 開始時は当該commandのsandbox TEMPで `ww5-i2-gate-lib-*` と `ww5-i2-pointer-public-*` の集合0を記録した。しかしcommandごとにsandbox TEMPのUUIDが異なり、終了確認時には開始rootが存在しなかった。終了確認commandが非terminating error後に空配列を集計したため、`temp-end.json` と `integrity-result.log` の `end=0 / added=0 / removed=0` は有効な集合差証拠ではない。同じrunの前後TEMP集合は未取得であり、cleanup成功や全TEMP0を主張しない。stderrにcleanup失敗の明示行はないが、それだけでは残存0を証明できない。次回は試験と同じcommand内で前後集合を採取し、列挙失敗をterminating errorにする。記録訂正は `resume-20261011-temp-correction.log` にも保持した。

### P08先行normal saveの限定診断

親の追加packetにより、当caseの最初の `entered_rx` 受信失敗分岐だけの所有を受領した。失敗時にrelease senderをdropしnormal workerをjoinして実Resultをpanicへ含めた。成功経路、timeout値、期待値、本体、他caseは不変である。逆置換した全文SHAが `E9F28165…6FD9` と一致し、rustfmt checkとdiff checkも成功した。診断変更はレビュー対象として残す。原因修正やassert変更は行っていない。

- 診断後store SHA: `BA93F520BFE8508D9AC440EB356773945F1F0F1623887D5FFD5A0CFD6C63A9BC`。
- lib-only buildはexit0、74.341秒。生成lib EXE SHA: `42E45CC9DD9383835862503B8B60CDB7CB65B33AA2CA63F4CFBFA0E51AFCD706`。初回の `223EE259…64EFC` EXEは同pathでこの診断版へ更新された。前回ログは保持した。
- 当EXEの `--exact store::i2_pointer_commit_tests::p08_waiting_pointer_uses_nonhistory_updated_under_normal_save_guard --nocapture --test-threads=1` だけを実行し、exit101、0.689秒、1 failed / 0 passed / 0 ignored / 278 filtered out。実Resultは `Ok(Err("アクセスが拒否されました。 (os error 5)"))` であった。pointerは依然未起動であり、この再実行を追加の行動REDに数えない。

現sourceでpublisher前にこのraw OS errorを返す経路は、`save_progress_with_publishers` が先行する日別退避を `preserve_daily_backup_with_publisher` → 既存 `atomic_write` へ渡した箇所である。typed read/closure/M/size拒否は別の固定文言となるため、この経路に絞られる。ただしatomic内部のopen/write/sync/renameのどのsyscallが拒否されたか、およびOS拒否の根本原因は未識別である。今回のsandbox TEMPと過去N独立runnerのsandbox外成功環境の相違だけで原因を断定しない。

親へ返す最小比較案は、sourceを変更せず既存fixture環境変数 `WW5_I2_GATE_LIB_ROOT` を今回所有のworkspace内一時dirへ向け、同EXE exact1件で日別atomicが成立するかを確認することである。未実施であり、原因修正の許可は受けていない。日別backup事前作成による先行atomicのskip、fake publisherによる成功への置換、assert緩和は今回の失敗を隠すため採用しない。

診断ログは `artifacts/history-pointer-commit-tests-resume-20261011-diagnostic-*`。source-proof、build-result/jsonl/stderr、exe-start、red-result/stdout/stderr、hashes-endを保存した。診断実行と同じshell内で、継承TEMP/TMPが指すsandbox rootと対象prefix集合を採取し、当runに限り0→0・追加0/消失0であった。全TEMPや初回18件の残存0を意味しない。終了EXE hashは開始と一致、終了storeも上記診断版と一致した。17件の初回失敗とsource位置の対応は [分類](artifacts/history-pointer-commit-tests-resume-20261011-red-classification.log) に保存した。integration4件・旧回帰・P本体は引き続き未実行/未実装である。

### workspace fixture比較後: 新P22の行動RED到達

親の限定許可で既存 `WW5_I2_GATE_LIB_ROOT` を新規専用workspace directoryへcommand内だけ設定し、診断版の同EXE・同exact1件をbuildせず比較した。`p08_waiting_pointer_uses_nonhistory_updated_under_normal_save_guard` は先行normal saveのcallback・保存完了を通過し、`src/store.rs:5241` のpointer Resultに対するunwrapで固定 `HistoryError { kind: Io }` を観測した。exit101、0.214秒である。このcaseは当比較で初めて行動REDに到達した。先行日別atomic拒否はfixture配置環境に依存することを確認したが、具体syscall/OS要因の根本原因までは確定していない。fixtureの本体・期待値、OS設定、UAC/ACLは変更していない。

続いて既生成integration EXE（SHA `F03DDA59…85B0` 不変）へ `--nocapture --test-threads=1` を渡し、公開4件を初めて実行した。public fixtureはWW5変数を読まないため、そのcommand内のTEMP/TMPだけを別の新規専用workspace directoryへ設定し終了時に戻した。exit101、0.146秒、4 failed / 0 passed / 0 ignored / 0 filtered out。P01（451行）/P03（475行）はCommittedを要求する入口で固定Io、P06（536行）はCancelledを要求する入口で固定Io、P02（568行）はConflictを要求してIoを受けた。全て行動REDでありfixture失敗ではない。

今回のユニーク受入件数は **lib18＋public4＝22行動RED、0 PASS、0 ignored** となった。初回libの環境fixture failure1件と診断再現1件は履歴として保持し、成功や別の受入件数へ加算しない。初回にREDを確認した17件は再実行していない。旧回帰は本体後の独立runnerへ残し、P本体GREEN・後段mode/ledger/実I/O対照は未完了である。

workspace比較とpublic実行の各commandで、専用directoryの新規collision拒否、祖先とrootのreparseなし、明示絶対path一致を確認した。各run後に所有directory内が空であることを列挙し、同じ個別rootだけをnative PowerShellの非再帰Remove-Itemで削除した。両runとも所有残存0、全TEMP0の主張ではない。ログは `resume-20261011-workspace-probe-*` / `resume-20261011-public-*` に保持した。

最終固定source10件は当cfg(test)診断分岐以外が開始時と一致し、storeは `BA93F520…63A9BC`、lib EXEは `42E45CC9…FCD706`、integration source/EXEは元hashを維持する。`resume-20261011-final-source.json` / `final-exes.json` / `final-result.log` に保存し、diff checkも成功した。source/build所有を親へ返却する。実装者へはP01–08の既定fixture/期待値、当22件のコマンドと行動RED分類、workspace内の専用fixture配置、未到達境界を引き渡す。追加した診断分岐は最終独立レビュー対象である。
