# UNKNOWN-DESCENDANT-OPEN01：最小観測設計

2026-10-22／U-B。planner指定 `gpt-6-astra/high`、実provider metadata未取得。所有は本書だけである。[再開計画](unknown-descendant-resume-plan.md) SHA256 `E8A2332F45A15864B1D31D493FDC5DA08365E28B994ACF56386C0B0014FAE314`、親受領済み[保存証拠監査](unknown-descendant-evidence-audit.md)を入力とする。AGENTS、development-team、KPI、`agent-skills:api-and-interface-design`を全文読了した。同Skillの内部契約・境界検証・unknown保持を適用する。UI変更はなく、日本語描画・両軸中央・拡大率・狭幅およびUI Skillは非該当である。REST、新module、常設frameworkは作らない。

完了境界は観測案と独立Preへの提出であり、実装・実試験・UNKNOWN閉鎖ではない。現許可は文書と静的メモリ計算のみである。以下の変更案は未実装・未承認であり、既承認の観測仕様への追加差分としてPreと親の受領を要する。

## 既存契約と静的根拠

略記R=`target/early-trial-build/et3/cargo-gate`。既承認[ET契約](early-trial-contract.md) ET-07、[G1–G6](early-trial-gate-plan.md)、[FF1–FF4とKH/MI](early-trial-gate-design.md)を再利用する。G4のargv/stream/exit透過、G5の未知非停止・有限回収、FFの診断失敗による元障害の置換禁止を保つ。ENV01 case row・成功期待・expectedExit比較範囲は変更しない。診断の採取成否は新観測単位の成否であり、ENV01/旧F3のPASSへ変換しない。この範囲では新しい外部仕様は不要である。target exit0・completed marker必須等をENV01の新成功条件にする必要が生じた場合は、仕様担当へ戻し、本書から追加しない。

| 根拠 | 確認範囲 |
| --- | --- |
| 旧runの72/72 hash一致 | 生成run資料の保全だけであり、凍結入力本文の所在証明ではない。U-Aを再調査していない |
| 現Invoke `898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71` | P7許可差分に対応する`7.6.7`4出現（line20、1045内2個、1083）をUTF-8文字列上で`7.6.6`へ戻し、UTF-8 bytesをSHA256計算した。結果は旧`318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3A`と完全一致。旧本文のファイル保存・展開・実行0 |
| 旧本文との対応確定 | Invoke1569がGetProcessById、1570がSafeHandle、1571がGet-Identityである。FFの1569はGet-Identity未到達に対応する。本文一致は当時のOS状態を証明しない |
| fake `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016` | target151–194でobserve→wait→stdout/stderr→flush→exit設定→completed marker。main291–297でErrをstderrへ表示しexit93。指定exit93と内部Err93をexitだけで区別できない |
| 現Test `F0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919` | 593–636に既存Start/Complete-FixtureProcessと独立stream採取。1987–1996のexpectedExit照合はTRANS03–07だけ。2030以降のENV01判定は環境等を扱う。現Testを旧本文と断定しない |

原初因は未確定である。Runnerの直接handle exit93、Supervisorのknown0/snapshot1、RunOwnerのcancelを別actor証拠として保持する。当時のsnapshot採取区間・元エラー・ET3_FAKE継承値、および10/18以降の対象環境変化は未確認である。

## 受入対応・変更候補

| 計画ID | 予定span／所有役 | 新しい観測と判定限界 |
| --- | --- | --- |
| U-R1、U-R3 | test author：R/fake.rsのobserve keys（113–120）、target（151–194）と小private writer | 既存environment.jsonに下記7変数を追加。target内の元Errを独立sidecarへ保存し、stderr自身の失敗でも取得可能にする。argv/stream/exitとcompleted markerの既存本文・順序を保つ |
| U-R2、U-R3 | implementer：R/Invoke-CargoGate.ps1のRead-OwnedTree snapshot取得（1422）、未登録candidate取得（1569–1571）、同catch（1584以降）、Get-IdentityFailureObservation、既存FF projector | 既存1回のsnapshotと取得の時刻区間・到達段階・構造化例外を任意追加記録する。Known、所有判定、取得順、登録条件、kill/wait、catchの元failureは変更0 |
| U-R4、G4/G5、FF1–3 | independent test author：新UUID内の限定oracle、既存FF抽出方式の必要部分のみ | 下記6対照と非所有逆diffを固定する。既存80行・ENV01期待・新OS障害注入口を変更しない。ソースをdot-sourceせず、source hash/一意AST抽出/逆照合を使う |

実装者と作者は共有fileを同時編集しない。最終spanは上記hashに対するAST/関数境界で固定し、行番号だけで採番しない。新fieldは診断だけで、Run/registration/approval/terminalの必須schemaとauthorityへの入力にはしない。既存FFの「12field＋任意producerFailureのみ」という射影契約には、以下の任意`acquisition`追加を明示差分として扱う。既存consumer/抽出testが余分なfieldを拒否する場合は期待を黙って変更せずPreへ戻す。

## データ契約と所有

**fakeの既存environment記録。** 追加keyは厳密に`ET3_FAKE_ROOT/MODE/EXIT_U32/RELEASE/WAIT_MS/BUILD_RELEASE/INVOCATIONS`の7個である。既存`var_os`→UTF-16整数配列／未設定nullを再利用し、未設定と空配列を区別する。任意prefix列挙・全環境採取0。observe時点はtarget自身がhold flagによりRELEASEを書き換える前であり、継承値である。run.envMapと同値と推定しない。既存書込回数は増やさない。field追加による記録hash変化を新fixture入力へ束縛する。

**fakeの新sidecar。** `target`の元bodyを一つの`io::Result<i32>`局所closureとして保ち、その外側だけでErrを捕捉する。stack-localなstageを各既存fallible操作直前に更新する。stageは`Observe/WaitRelease/StdoutWrite/StderrWrite/StdoutFlush/StderrFlush/ExitSetting/CompletedWrite`の固定列挙である。treeへの既存早期returnは`TreeDelegated`として区別し、その内部原因をtargetのI/O段階と偽らない。Errの場合だけ検証済みfixture root直下`target-<pid>.failure.json`へ一回記録し、元Errをそのままmainへ返す。Ok(code)には新完了判定を加えない。mainのstderr prefixとexit93は保持する。

sidecarは`schemaVersion=1, pid:u32, stage:string, errorKind:string, rawOsError:i32|null, messageUtf16:u16[]|null, messageStatus:Captured|NotAllowlisted|TooLong`だけの小recordである。kindはRust ErrorKindの診断表示でありgate分岐に使用しない。元エラー本文を無制限保存しない。messageはfake source内の固定エラーliteralとの完全一致、又はrawOsErrorから再生成したOS error表示との完全一致の場合だけ最大2048 UTF-16 unitsを保存する。他はnullと理由を記録し、完全な本文を取得したと称さない。path、argv、環境全文、ErrorRecord全文、任意inner-dataは保存0。総UTF-8上限8192 bytesを超えた場合はmessage=null/TooLongに落として再serialize一回までとする。id/contextを文字列化して別pathを作らない。

writerは既存`write_new`と同じcreate_new/write/sync_all規則で固定`.tmp`へ一度書き、同directoryの未存在finalへ上書きなしで公開する。tmp/final衝突、serialize/write/flush/公開失敗は観測失敗として保持し、元Errを置換しない。renameの上書き禁止を選択APIで保証できない場合は、既存write_newでfinalへ直接書く最小方式に戻し、complete JSON parse＋hash成功だけをPublishedとすることをPreで固定する。削除・retry・別名fallback0。root検証前のrun()失敗、panic、強制終了はこのsidecarの採取範囲外である。

**Supervisor等の任意acquisition。** actor-local Read-OwnedTree呼出ごとにsnapshot取得直前/直後UTCをscalar保存し、未登録candidateのGetProcessById直前から`LookupHandle→PinHandle→ReadIdentity`をlocal stageで更新する。既存catchで失敗UTC、例外type/HResult、存在すれば直接inner例外のtype/HResult/NativeErrorCodeだけを1階層取得する。文字列Message/StackTrace/command/pathは追加しない。`acquisition={snapshotStartedUtc,snapshotCompletedUtc,attemptUtc,failedUtc,stage,exceptionHResult,innerType,innerHResult,nativeErrorCode}`は任意で、時刻はUTC7桁string又はnull、codeはint又はnull、typeは既存FFのASCII256字制約である。snapshot取得自体が失敗した場合は既存経路のままで、このcatchの診断があると約束しない。

FF projectorはこの固定fieldのみを追加射影し、必須context・8192 bytes上限・一回性・元status/AbortTick/AbortUtc保持・元throwを維持する。時刻取得/追加射影自体の失敗は診断だけnull/省略とし、元sourceの成功/失敗へ影響させない。snapshot内容・GetProcessById・Get-Identity・Known再照会は増やさない。actor間のUTC比較は参考であり、同世代の根拠はrow CreationDate、target startedのstartFileTimeUtcと各source hashを別に照合する。欠測・不一致なら競合仮説を確定しない。PID一致だけによる所有昇格は0である。

## 順序・回復・合成対照

targetの観測順は既存environment保存→元操作列→Err保持→sidecar試行→元Err返却→元stderr表示/exit93である。sidecar失敗で元エラーを失わず、stdout/stderrを再open/redirectしない。監視側はsnapshot取得区間の記録→元取得列→元catch/Unprovable→既存FF→元cleanupである。新非同期worker、cancel通知、追加wait/retryは0。追加I/O時間は既存期限を消費し、期限延長をしない。同期I/Oの強制中断保証はなく、返却後超過も失敗とする。

両actorは互いの記録を変更しない。sidecar/FFは診断であり停止権限を与えない。実試験に進む場合も既存ownerだけが既知handleを回収し、unknownは停止しない。途中失敗・tmpのみ・破損JSON・記録欠落をPublishedへ変換せず、旧/new証拠を新UUIDごと保存する。原本、失敗fixture、stdout/stderr、残留/tmpを削除しない。

独立作者の最小6対照は以下であり、今回は作成も実行も0である。各case5秒、集合30秒を候補上限とし、開始/終了検査、firstUnexpected停止、後続NotRun、retry0とする。同期APIのhard timeoutの証明ではない。

| 対照 | 純合成／静的受入 |
| --- | --- |
| O1 | environment 7keyのnull/空/値を独立oracleで比較。既存field不変、追加任意変数採取0。プロセス環境を変更するfixtureは作らず、独立test seamへの値入力で確認 |
| O2 | targetの成功0と指定93を各1入力として元Ok/marker/stream列の不変を確認。failure writer呼出0。成功93を内部Errと分類しない |
| O3 | 8stageそれぞれの合成Errを入力し、元kind/raw codeとstageが保持され、元Err返却・main exit93本文不変。OS停止やpipe破壊でエラーを作らない |
| O4 | 診断writerの衝突/失敗/上限/非allowlist本文を合成入力し、元Err不変、試行1、retry0、過長・未許可本文保存0。実装コード自身から期待値を生成しない |
| O5 | snapshot/candidate/例外の合成scalarからFF追加射影を確認。stage LookupHandle失敗時、後続Get-Identity/登録/killが0であることを元関数の局所adapter/counterで確認。実OS API呼出0 |
| O6 | acquisition欠測・診断失敗・異actor/世代不一致を含む記録組を独立判定し、unknown保持、元status/abort/throw保持、8192上限、追加OS照会/retry0。既存FF通常12fieldへの余分な必須key追加0 |

case内の複数入力は入力数を別記し、6caseを6呼出と偽らない。adapterは最小関数/statementの一意抽出と元bytes逆照合が成立する場合のみ使用する。正しいseamを作れない場合はAdapterFailureで止め、新frameworkへ広げない。fakeのstage・診断writer導入に必要な局所seam以外に製品/ENV01のfault switchを作らない。Parser/PSSA対象とRust局所compileの完全commandは作者成果物が存在してからplannerが固定する。未存在commandを実行可能と称さない。

## 実行許可へ渡す条件と未解決

順序は本書独立Pre→plannerが変更span/最小対照/入力hashを固定→親が観測追加の変更許可を確認→作者と実装者を順次dispatch→新hashで局所6対照と独立Post、である。本書はこの許可を与えない。bindingは最終設計/Invoke/fake/Test→局所oracle→必要な既存ENVprobe/hash固定→親manifestの順であり、旧run/旧承認は不変である。依存先が変わらない既成功P7/MissingOs/Context12を一括反復しない。

局所Post後の実OS観測候補は、新UUIDの合成ENV01を**1件1回**、既存`Invoke-FirstFailureEnv01.ps1`の元dispatchのままで観測する一回枠である。通常ユーザーAdmin=False・Hidden・現承認runtime/source/hash・新fixture/output絶対root・全argv・既存期限/回収・固定toolchainを親が事前に凍結する必要がある。完全command/具体UUID/最終SHAが未確定なのでrunner dispatch不可である。新native build、F3/focused10/標準80、実data/API/admin/設定変更は含めない。source/hash/入力不整合・unexpected failure・観測欠測・超過・回収不明で停止し、次case/再試行/fixture削除0。実際にこの一回枠を使うかは新たな限定承認で決める。

結果は「元fixture Errと監視取得失敗が同runに共存」「設定exit」「採取できたが因果不明」「診断欠測」を区別するだけである。新試験は当時の環境を復元せず、観測追加自体も時機へ影響する。sidecar取得に成功してもSupervisor unknownを救済せず、新ENV01成功でも旧F3はFAILのままである。未知候補の取得retry、他actorのhandleによる救済、target正常性oracle強化は本案に含めない。前二者はG5境界を変え、後者は外部期待判断を要するためである。

実装handoffは上記2fileの観測spanだけ、テスト作者handoffはO1–O6と既存消費者互換性である。残る判断は独立Pre、必要な局所seamの成立、診断公開APIの上書き禁止確認、最終hash/command、実OS一回枠の承認である。開始statusの他者変更2fileと計画untrackedを保持した。子usage開始/終了counter・親子包含は未取得で、親が取得可能範囲を記録する。文書以外の編集、実試験、OS照会/停止、リンク作成、API、再委任、commit/pushは0である。

## U-D追補：承認済み診断の内部契約（2026-10-22）

planner指定`gpt-6.1-sol/high`、実provider metadata・子usage開始/終了は未取得である。入力計画SHA256は`7D8AFBEB3AFCAD08563354E8690167BAB20EB149BDA2110BA7F1B1D3DB77D746`、[限定Pre](unknown-descendant-review.md)の3gateと親受領の利用者承認を適用する。本節は上記の未確定writer/接続/seamだけを更新する。既承認ET-07/G4/G5/FF1–3を再利用し、外部期待変更0である。診断追加と隔離内O1–O6は承認済み、新OS/ENV01/native/fullは解除0である。

**Rustのprivate契約と保存。** `TargetFailureStage`は既記8値＋`TreeDelegated`、文字列は上記表記へ固定する。`target`の局所closureは元`io::Result<i32>`を返し、stage更新以外の元操作順を保つ。

- `encode_environment(keys:&[&str], lookup:impl FnMut(&str)->Option<OsString>)->String`は既存12key＋承認7keyだけを順番にJSON化する。productionは`env::var_os`を渡し、observeの既存保存回数を保つ。null/空/UTF-16は従前どおりである。
- `encode_target_failure(pid:u32, stage:TargetFailureStage, error:&io::Error)->Vec<u8>`は既定7fieldだけを返す純関数とする。内部のmessage選別と8192bytes判定は別statement境界とし、上限超過時のnull/TooLong化は一度だけである。新crate/汎用serializerは導入しない。
- `try_write_target_failure(root:&Path, pid:u32, stage:TargetFailureStage, error:&io::Error, write_once:impl FnOnce(&Path,&[u8])->io::Result<()>)->io::Result<()>`は固定final pathと上記bytesを一度だけ渡す。productionの`write_once`は既存`write_new`であり、直接CreateNew→write_all→sync_allを行う。tmp/rename方式を採用しない。
- writerの`Ok(())`はWriteSynced、`Err`は診断失敗である。Published判定は外側の完全JSON parse＋SHA成功に限る。targetのErr分岐はwriter結果を局所値に保持して捨て、`?`伝播せず、借用元の同じ`io::Error`をmainへ返す。Ok(code)分岐はwriter呼出0である。writer結果は新sidecar fieldや新stderrへ出さず、O3/O4の返却値・呼出counterで測る。
- O4のfile境界fakeはin-memory CreateNew/WriteAll/SyncAll状態と各失敗を持つ。実`write_new`の抽出片をこの境界へ結び、衝突時はwrite0、partial/write/sync失敗時は残byte保持、open1/retry0を確認する。FSアクセス・削除・別名救済0である。同期I/Oの期限・cancel限界は既設計のままである。

**PSのlocal接続とwire。** `Read-OwnedTree`内だけにsnapshot開始/完了UTC、candidate試行UTC、stageを置く。各candidate直前に試行値をnullへ戻し、既存未登録取得列だけで`LookupHandle→PinHandle→ReadIdentity`を更新する。既知枝やparentなしの枝へ前候補の値を持ち越さない。

snapshot/失敗UTC取得とrecord作成はbest-effortの局所try/catchとする。元`$caught`を先に保持し、既存Get-IdentityFailureObservation/producerFailure fallbackの後で、candidate試行があった場合だけ`$observation.acquisition`を追加する。作成失敗はkeyなし、単一値の欠測/型不正はnullであり、元exceptionへData追加や再throwをしない。Get-IdentityFailureObservation、New-FirstFailureRecord、Invoke-GateFailureのsignatureは変更0である。

入力位置は`FailureObservation.osObservation.acquisition`、出力位置は`record.failure.osObservation.acquisition`である。acquisitionがIDictionaryの場合だけ新9keyを射影し、それ以外はkeyごと省略する。内側は厳密に`snapshotStartedUtc,snapshotCompletedUtc,attemptUtc,failedUtc,stage,exceptionHResult,innerType,innerHResult,nativeErrorCode`、時刻は既存Get-Utcで検証するUTC7string又はnull、stageは上記3値又はnull、codeはInt32範囲整数又はnull、innerTypeは既存ASCII1–256文字又はnullである。例外type本体は従前のosObservation.exceptionType、直接inner一階層だけを採取する。入力の未知keyは出力しない。acquisitionなしでは旧12key＋既存任意producerFailureのみを維持し、envelope/failureのkey集合とFF byte上限は変更0である。

**単一processの局所seamと独立期待。** 作者は新UUID内に抽出unitを作り、凍結前に生成を完了する。PSは既存AST一意抽出方式、Rustは一意fn境界とoffset/length/元byte hashを固定する。compile対象は抽出済みRust test unitだけであり、元fakeのmain/run/cargo/tree/externを含めず実行しない。新module/framework/OS注入口は作らない。

| 対照／要求 | 実コードとの接点・独立観測 |
| --- | --- |
| O1／U-R1,3 | 新旧observeのkeys/records構築片を抽出し、var_os取得だけを合成lookupへ置換する。19keyの厳密集合と各null/空/値を独立期待で照合し、環境変更0である |
| O2／G4,U-R4 | 新旧targetの元bodyを抽出する。args/env/flag/observe/wait/stream/number/completed書込/tree境界だけをin-memory入力・sinkへ結び、成功0/指定93のOk・出力byte・操作列・markerとwriter0を比較する |
| O3／U-R1,3 | 同targetの各8操作境界へ一度ずつ合成Errを返し、stage、元errorのkind/raw値と独立payload同一性、mainの元Err93分岐の静的不変、sidecar一回を確認する。TreeDelegatedはnative treeを呼ばず合成delegationで識別する |
| O4／FF3,U-R4 | 実encoder/writerとwrite_new抽出片を使用する。message選別と上限判定statementへ独立の非許可文字列/過長UTF-16/過大encoded値を入力し、null化・8192以内・書込一回・診断Err時の元Err保持を測る。新エラーliteralを本体へ足して刺激を作らない |
| O5／U-R2,4 | `$parents`算出以降の未登録candidate statementsと同じcatch/finallyを抽出し、合成tryへ置く。GetProcessById呼出だけをthrow可能なlocal lookupへ、SafeHandle getter/Get-Identity/Add-KnownProcess/FF呼出をcounter付きfakeへ結ぶ。lookup失敗でidentity/登録/kill0、元Unprovableと追加scalarを実catchから取得する |
| O6／G5,FF1–3,U-R2,4 | 実New-FirstFailureRecord/Get-UtcとInvoke-GateFailureを抽出し、writerだけをin-memory失敗へ結ぶ。旧形/追加形の厳密集合、invalid→null、8192上限、元status/AbortTick/AbortUtc/throwと一回性を照合する。異actor/世代不一致の記録組は独立oracleでunknownとし、既存所有判定が不変である静的証拠と分ける |

置換は上表境界だけである。作者は元statementの位置・原文・置換個数・置換後byte hashをmanifestへ固定し、逆置換で原文byte一致を必須とする。PSの他枝を含めたRead-OwnedTree全体やRust mainを実行してcounter0を主張しない。lookup/identity/登録/kill等の実OS到達は抽出unitの静的禁止検査とfake counterを併記する。実装の非所有逆diffは別に、許可spanをbeforeへ戻した全fileのbyte/SHA一致を検査し、adapter逆照合だけで代用しない。

Rust targetのfault差込点は既存fallible呼出13箇所だけである：Observe1、WaitRelease1、StdoutWrite5、StderrWrite2、StdoutFlush1、StderrFlush1、ExitSetting(number)1、CompletedWrite1である。O3はfull target経路の各stageを1入力ずつ選び、同じstageの未到達別枝へ追加faultを配らない。境界fake化の補助置換はargs_os1、flag is_file6、set_var2、tree1、既存completed内process::id1、新sidecar呼出用process::id1を上限とし、encoder/writer新helperの関数内byteは抽出のまま保持する。導入stage更新とclosure以外の呼出増減はPreへ返す。stageは最後に着手した取得段階であり、ReadIdentity後の既存検証失敗をOS取得失敗と断定しない。

oldREDはbaselineにも存在する上記keys片/target body/candidate catch/projectorを動かし、O1の追加7key欠測、O3のErr時sidecar呼出欠測、O5のacquisition欠測を独立のbehavior assertionで検出する。baseline向けadapterは存在しない新helperを要求せず、診断呼出counterの初期0を保持するだけである。O2と旧FF互換はbaseline PASSを期待する。新writer専用O4/追加形O6を旧版で無理に動かさずNotApplicableと別計上する。抽出一意性/逆照合/compile失敗や未実装helper不在はAdapterFailureでありREDではない。入力数をcase数と別記し、5秒/30秒・firstUnexpected停止・残NotRun・retry0を維持する。

handoffはU-D差分Pre→独立作者のoracle/完全command固定→実装者の上記2file局所変更→最終hash/親freeze→別runner/Postである。作者へ追加設計判断を残さない。上限statement抽出等の一意性が成立しなければAdapterFailureで親へ返し、新frameworkやOS試験へ広げない。本追補は文書静的確認だけであり、runtime成立・UNKNOWN閉鎖・旧F3治癒は未検証、他者編集保持・実装/テスト編集/対象実行/再委任/公開0である。

## U-E限定seam追補：O5-diagnosticfail（2026-10-22）

designer指定Sol6.1/highを継続する。追補前設計B92FE83E…をpacket `ef551043-d22a-44a7-903b-1e9540efc910/before/design-B92FE83E.md`へCopy-Item保全し、完全SHA256 `B92FE83EB555829AF2EC111E95C9E0109747CB0FEB31CD5B83977760836C27D7`一致を確認した。
固定期待`expectations.json` SHA256 `A1914AD22D864A30FEFEA90801F349F152EB23177E92E75774863F880047FAD4`のO5-diagnosticfail（U-R2/U-R4）は「元caught保持後に診断構築がthrow→acquisition省略、元Unprovable/FF保持」のままとする。6case/51入力、旧版の本入力NotApplicableは変更0である。
現Invoke SHA256 `3B784E9B7CAE98261D0A496C0E229C4E2CB39120962656BF2186049A08CCC4D3`のRead-OwnedTree→candidate catch→`if ($null -ne $acquisitionStage)`→内側tryの先頭`$acquisition = @{ ... }`一文だけを、当該入力の抽出unit内で置換可とする。他入力の追加置換は0である。
正確なspanはline1636–1638、ASTのUTF-16 offset `[92581,92916)`（335文字）、元file UTF-8 byte offset `[92594,92929)`（BOM3bytesを含む335bytes）であり、元文UTF-8 SHA256は`7985FF6954A18BE9BE2CDE39B7C8D3E633CFC0B53395748890941177A8209493`である。offsetだけで選ばず、上記親AST列・左辺・hashが一致する一意AssignmentStatementAstを必須とする。
置換文字列は厳密に`throw [InvalidOperationException]::new('SYNTHETIC_ACQUISITION_CONSTRUCTION_FAILURE')`、個数は1である。assignment以外の内側catch（Remove）、既存observation/fallback、元caught保持、Unprovable追加、FF呼出、finallyは元抽出bytesのまま通す。
作者manifestへsource hash・元文全文・offset単位・length・元文hash・置換全文・適用順・生成unit hashを保存する。既存許可置換と本置換を逆適用し、全抽出原文のbyte一致を必須とする。曖昧一致・個数違反・逆不一致はAdapterFailureであり、behavior RED/PASSへ変換しない。
lookupで元の合成取得Errを一回発生させ、上記throw後は実catchがkeyを除去すること、元exception型/本文由来のUnprovable・元FF引数が保持されること、lookup1/pin0/identity0/registration0/kill0を独立期待で確認する。診断用throwをlookupの元Errと取り違えない。
この入力が証明するのは診断構築tryの任意失敗からの非干渉回復だけである。実scalar getterの失敗可能性、OS取得故障、OOM、実行時タイミングや旧UNKNOWN原因を証明しない。通常構築とgetter欠測→nullは他の既存入力に任せる。
canonical source/期待の編集、新public signature/runtime fault flag/抽象化framework/OS操作は追加0である。純合成6対照の承認範囲内にあるtest-only adapterの明示化であり、新たな利用者判断は不要と判定する。差分Pre受領後だけ作者が当該1入力へ反映し、完全command/hashの再freezeは親が行う。

## U-H追補：実OS ENV01入口の静的設計と実行阻害条件（2026-10-24）

planner指定`gpt-6.1-sol/high`、実provider metadata・子usage開始/終了counter・親子包含は未取得である。入力は[再開計画](unknown-descendant-resume-plan.md)150行以降（読取時SHA256 `B066305F62220E506A017F24035BCCCD3D317CDC9A1438B7F693EE9CE7D18079`）、本書既存契約、親受領済み6case/51inputs・独立Post PASSである。既承認ET-07/G4/G5/FF・ENV01 oracleを再利用する。api-and-interface-designを全文読み、入力境界と生成物境界、success/failure/unknownの分離に適用した。UI・RESTは非該当である。所有は本追補だけ、完了境界は独立Preへ渡す静的差分であり、source/期待/receipt/ログ変更、対象実行・ValidateOnly・compile・OS照会・再委任は0である。

**結論。既存probeの2literal更新だけでは、compile込みの有限実行枠を満たさない。** 現probeは同期`rustc --version`とcompileを無期限に待ち、Testの90秒watchdogはその後に始まる。下記commandは実行不可draftであり、U-Iが入口・診断保存を確認できても、有限停止の不足が解消するまでU-J実装範囲のfreeze／U-K dispatchへ進めない。本追補は実OS実行承認を与えない。

### 最小bindingと生成後freeze

略記Rは従前どおりである。現probe `R/verification/argv-path01/Invoke-FirstFailureEnv01.ps1` の読取SHA256は`0B6C78196FD5DB0CB5AE99364BC26720C99E2214467A71FC32156E2A12E91951`である。元dispatch・case行・oracle・回収を保つbinding候補は、同fileの`$fixed`内の次の各1literalだけである。

| E-R1／入力 | before → after（完全SHA256） |
| --- | --- |
| Invoke-CargoGate.ps1 | `898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71` → `3B784E9B7CAE98261D0A496C0E229C4E2CB39120962656BF2186049A08CCC4D3` |
| fake.rs | `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016` → `07A47648E47FB57F5D0969650BBCEAF138671A4FC151EF6AA1E33020DCF2DCC4` |

両afterとTest `F0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919`は本工程のread-only hashで一致した。Test/native source/native exe/主plan/designの他literalは変更0、各pinはU-Jで独立照合する。実装者はbefore bytesを新packetへ保全し、2literalを逆置換した全bytes一致を証明する。将来必要な有限停止変更はこの許可spanへ混入させず、plannerが別spanとして設計・独立Pre・承認を確定する。最終probe SHAは変更後に採取し、旧自己hashを承認値へ流用しない。

E-R2のfake.exeは未生成であり、SHAは`PendingGeneration`である。抽出unit binary・旧fake.exe・source SHAを代入しない。必要な順序は、事前にsource/toolchain/command/生成先と「生成値を採用する規則」を親が承認→probe内でcompile一回→成功exit・期限内返却→生成fake.exe hashを一回取得→`$fixed`へ追加→toolchain.json/frozen-inputs.jsonを保存→既存case前12入力照合→ENV01 dispatch一回以下、である。probe220–240にはhash追加とこの保存・照合が既に存在する。生成後・実行前freezeは必要であり、この規則による同一probe内freezeを採用する案である。生成SHAを親が対話的に受領してから再開する停止点は現probeにない。その形式が必須なら2literal案では成立せず、別compileや二回起動を追加せず設計へ戻す。

E-R3の親manifestは入力／生成物を分ける。事前入力は最終probe、Invoke/fake/Test、native source/exe、probeがpinする主plan/design、現計画/設計/承認packet、固定pwsh.exe/pwsh.dll/System.Management.Automation.dllとそのruntime依存、V2Case、Pester5.7.1の解決先manifestと実load対象である。compilerはPATH上のrustc.exeだけで閉じず、shimならrustupと選択toolchainのrustc、必要DLL/sysroot標準library、実際のlinkerと必要SDK/library、解決に用いるtoolchain/PATH等の限定設定をpath/hashで束縛する。実体を特定できない依存は未確定として止め、全環境dumpや現在値へのpin一括更新で埋めない。git check-ignoreに使うgit実体とignore規則、Cargo config4pathの存在/hashも記録対象である。これらはread-only準備の候補であり、本工程ではruntime/toolchain queryを起動していない。実行時自己採取は事前承認の代用ではない。

### データ境界と診断保存

| 受入ID／既存module・型 | 所有・遷移・保存／test seam |
| --- | --- |
| E-R1,3／probe `$fixed:hashtable<absolute path,SHA256>`、`Invoke-FirstFailureEnv01Case` | 事前入力は親freeze、実装者は限定binding、runnerは生成物だけを所有する。`Unbound→PreFrozen→GeneratedFrozen→Dispatched→Stopped`を記録し、drift/compile失敗/超過はDispatchedへ進めない。既存before/after hash passと12入力数は不変であり、新依存pinは外側manifestへ置く。静的逆bytes、Parser/PSSA、一意dispatch位置を独立Preで照合する |
| E-R2,5／Test `New-CaseFixture`、descriptor/run hashtable | suiteは`F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et3\cargo-gate\verification\<new-suite-UUID>`、fixtureはその直下`ENV01`、runは`R/runs/<dynamic-runId>`である。UUID値はU-Jで固定し、runIdはfixtureが一回生成する。descriptorのsuiteId/root/runId、run.jsonのrunId/fixtureControlのpath/hashを相互照合する。単に最新runを選ばない。生成fake.exeとfixtureのfake.exe/fake-cargo.exe、およびrun.cargo/成果物manifestの参照hashを結び、コピーの不一致を保存して停止する |
| E-R4／fake targetの7field sidecar、started/environment/completed | 原本はfixture root直下、run側FFとは別所有である。target startedのpid・startFileTimeUtc・exe・argvからrequest/registration/resultの同一実行と世代を結ぶ。PIDだけ・file時刻だけで同世代としない。environmentは既存12＋承認7keyのnull/空/UTF-16をそのまま保存する。mainの元stderr/exit93とcompleted.exitU32、既存target stream/exit証拠を結び、controller.stdout/stderrをtarget直結streamと取り違えない |
| E-R4／Supervisor等のFF `failure.osObservation.acquisition` | run.json hash、gate source hash、role、actor pid/startTime、filenameを既存契約で照合する。外側12keyは不変、acquisitionは内側の任意9keyである。異actor/世代の値を足して一つの成功取得へ変換しない。stageは最後の着手点であり失敗APIや因果を断定しない |
| G4/G5/FF／既存RunOwner・Test回収 | 新worker/cancel/retry/kill権限は追加0。各actorの元Err/status/abort、所有判定と既知handleだけの回収は不変。sidecar追加I/Oも元期限を消費する。採取・parse・hash失敗を元ENV01期待の変更で救済しない。合成6case/51inputsと独立Postを再利用し、反復しない |

E-R4の補完はtest authorが新packetへ固定する事後read-only採取表である。probeの既存`result.json.diagnostic`はrun側FFだけを集め、fixture sidecarを集めず、exit0も診断充足を保証しないため、既存reportの書換え・key数変更は行わない。採取表にはdescriptor/run/source/生成exeのhash、targetごとのstarted/environment/completed、`target-<pid>.failure.json`の有無・raw bytes/hash/parse状態、関連stream/exit/result、actor別FFのraw bytes/hashとacquisitionを記録する。対象は当該suiteとdescriptorで結ばれたrunだけ、再帰的な全run収集は0である。原本を移動・上書き・削除せず、copyする場合は新packet内へCreateNewで保存し前後hashを結ぶ。

| 診断観測 | 独立判定（元ENV01判定とは別欄） |
| --- | --- |
| 同世代のcompleted/結果が元Okと整合しsidecarなし | `NoSidecarExpected`。指定93のOkを内部Errとしない。環境・marker等の他欠測は別記する |
| sidecarが8192bytes以内、固定7field/type・完全JSON parse・hash・pid/世代対応成立 | `Published`。stage/kind/raw codeを観測値として記し、元ErrとFF失敗の共存と因果を区別する |
| sidecarありだが部分write/破損/過大/型不正 | `AttemptedUnpublished/Invalid`。rawとhashが得られた範囲を保存する。再書込・修復・別名救済0 |
| Err/異常終了が示唆されるがsidecarなし、startedなし、世代不一致、必要stream/exitなし | `Missing/Unbound/Unknown`。sidecar捕捉前のroot失敗・panic・強制終了等を原因確定しない。sidecarなしだけで正常としない |

### 期限・起動境界と残る最大3判断

E-R5の既存期限はrun65秒（実行55＋回収reserve10）、run内build25秒、fixture watchdog90秒、終了後観測11秒であり延長0である。Test1743の90秒loopは同期OS観測を含むためhard timeoutではなく、finallyには別の最大11秒待機等もある。run内build25秒は事前fake compileの期限ではない。外側総枠を90秒や65秒と同一視しない。

最小追加設計の境界は次の3点である。いずれも未確定であり、親は実行承認に先立ち、この不足する安全手順の設計／限定実装範囲をplannerへ返す。

1. **compile込み外側停止。** probeの同期version/compile spanを有限待機へ結ぶ追加設計が必要である。既存Start-FixtureProcess/Complete-FixtureProcessのCreateNoWindow・ArgumentList・非同期stream採取・直接handle保持を再利用候補とし、新frameworkを作らない。version/compile各枠、入口から報告/回収までの外側総枠、観測・保存reserve、停止責任者を具体秒数とmonotonic時刻で事前固定する。まだ値を承認済みとはしない。遅れて返ったcompileも期限超過ならdispatch0とする。outer toolのyieldは停止期限ではない。外側期限でfixture回収を途中killして成功扱いにせず、controller/直接compiler等の証明済みhandleだけを停止対象とし、未登録linker等はUNKNOWNで保全する。同期OS/FS API自体のhard中断保証は追加waitだけでは得られず、hostが提供する上限・停止後の証拠保存能力まで確定しなければ実行不可である。
2. **実行環境・依存・権限。** 新suite/runの書込先はworkspace内であるが、rustcのshim/選択toolchain/linker/SDKとTEMP解決先、固定pwsh/Pester依存、CIM/Win32_Process/token観測を通常sandboxが許すかは静的文面だけでは未確認である。通常ユーザーAdmin=False/Hiddenは必要条件であり、sandbox外が必要との証明でもない。限定read/hashで依存と書込範囲を特定し、hostの権限仕様で判断する。確認のためのValidateOnly/試走や、拒否後の無承認sandbox外切替は0である。隔離外が必要なら同一command・root・期限・未知非停止の範囲を具体化して別承認へ含める。
3. **二段階freezeの承認形式。** 上記の事前承認済み生成規則により、compile直後のfake SHAを同一probeが実行前に固定する形式を親が採るかを確定する。採らない場合に必要な中間停止点は追加設計であり、生成前SHA捏造、probe二回起動、事前別buildを代案として自動追加しない。診断欠測は欠測で保持し、target正常性の新oracleが必要ならexternal-spec authorへ戻す。

元入口に渡すargvの実行不可draftは以下である。`<new-suite-UUID>`、最終probe/pwsh/toolchain/pin、上記有限停止を提供する起動主体・全引数・起動環境が未確定なので、そのまま実行しない。NoProfile/NonInteractiveだけではHiddenにならず、外側起動主体にもCreateNoWindow/Hiddenが必要である。

```text
Executable: F:\Program Files\PowerShell\7\pwsh.exe
WorkingDirectory: F:\Kazuhiro\GitHub\WordWeave5
argv (各要素をArgumentListへ渡す):
  -NoLogo -NoProfile -NonInteractive -File
  F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et3\cargo-gate\verification\argv-path01\Invoke-FirstFailureEnv01.ps1
  -NativeSourceSha256 381F5E1DCD70ED8040153FB155C8AABA0881FBFB57A8074B286FB4169D8683DD
  -NativeExeSha256 FF7C9A0F641F66F7C340AEC94C21EA9A0C745EA5A39C35B252C3A9CBDAFCCB88
  -OutputRoot F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et3\cargo-gate\verification\<new-suite-UUID>
```

この一入口に内在する`rustc --version`一回、診断fake compile一回、ENV01一件一回以下、元dispatchのOS照会/既知回収と指定rootへの保存だけを将来承認対象とする。ValidateOnly、別compile、native entry/製品再build、F3/focused10/80/full、51入力再試験、admin/実data/API/設定変更/公開を含めない。drift/firstUnexpected/compile失敗/超過/回収不明で停止しretry0、後続case0とする。保存の遅延・失敗も記録欠測として保持し、無期限待機や削除へ逃がさない。

独立Preへのhandoffは本追補と元probe/Test/fakeの指定spanだけである。reviewerはE-R1～5、元oracle/unknown非停止、生成freeze位置、actor/世代、外側停止未成立を確認する。実装者へ渡せる確定差分は2literal案だけであり、有限停止の追加spanはplanner・承認・差分Preを経てからである。test authorは新packetの事前pin／生成hash規則／事後採取表を所有し、期待を実結果から作らない。本工程の確認は限定参照・hashと文書diffだけであり、OS成功、compile成功、UNKNOWN原因、旧F3治癒は未検証である。
