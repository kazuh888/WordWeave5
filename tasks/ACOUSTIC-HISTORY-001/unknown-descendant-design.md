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

## U-L追補：生成後停止する限定準備経路（2026-10-24）

planner指定`gpt-6.1-sol/high`、入力は再開計画U-L～Pである。利用者承認は検証用手順の限定修正・純合成検証だけであり、本追補はU-Hの同一入口生成後freeze案を「保存後に必ずAwaitingApproval停止」へ更新する。ET-07/G4/G5/FF・元ENV01期待を再利用し、新公開param/function signature・helper file・module・再開入口は追加しない。api-and-interface-designの境界分離を継続適用する。実provider値・子usageは未取得である。
対象P=`R/verification/argv-path01/Invoke-FirstFailureEnv01.ps1`、baseline SHA256 `0B6C78196FD5DB0CB5AE99364BC26720C99E2214467A71FC32156E2A12E91951`、19920bytes、先頭BOM `EF-BB-BF`である。所有は本追補だけ、対象実行・compile・OS照会・再委任0である。

| 許可span／要求 | 実装へ渡す固定境界（baseline行、両端statementで一意照合） |
| --- | --- |
| L1／E-R1 | U-H記載のInvoke/fake SHA各1literalだけを更新する。他の既存pinは不変 |
| L2／E-R2,5 | line106の`$packetRoot = $PSScriptRoot`直前に新private関数`Invoke-Env01Preparation`一つだけを追加する。line36のErrorActionPreference直後にStopwatch開始値の初期化一文を追加する。既存関数の移動・signature変更0 |
| L3／E-R3 | line194の`$rustc = (Get-Command rustc.exe -CommandType Application).Source`からline201のforeach終端までを、下記manifest照合と同じrustc/pwsh/v2解決・既存fixed追加へ置換する。line202の`$self = ...`以降の元runtime/token/process preflightは保持する |
| L4／E-R2,5 | line219の`Import-Module Pester -RequiredVersion 5.7.1`からline230のtoolchain Write-NewEvidence終端までを、固定ports構築→private関数一回呼出し→生成後停止の処理へ置換する。生成準備にはPester import不要なのでこのimportだけ除く。line231の`$controller = Get-PinnedProcess ...`以降は全bytesを保持し、無条件returnにより今回経路から到達不能にする |

上記以外はbytes不変である。実装者は新packetへbefore全bytesを保全し、挿入を除去・L1/L3/L4を元bytesへ逆適用したP全体SHA/BOM一致を証明する。作者adapterの逆照合とは別検査である。lineだけで置換せず、元SHA・親AST・両端原文の一意性と元span bytes/hashをU-M manifestで固定する。不一致はAdapterFailure、別spanが不可欠なら当該箇所だけ親へ戻す。

**private契約。** `Invoke-Env01Preparation([hashtable]$Inputs,[hashtable]$Ports)`は単一resultを返し、`Inputs={startedTick,frequency,suiteRoot,sourcePath,fakePath,fixed,dependencies,manifestSha256}`とする。固定budgetは関数内定数であり呼出側で延長できない。`Ports`は`Now/ReadState/Hash/Start/Poll/Stop/Wait/Save`の8scriptblockだけとし、任意コマンド名の実行やdispatch callbackは受け取らない。既存公開関数は変更しない。
Nowはmonotonic tick、Waitは最大50msの待機である。ReadStateはcanonical path・leaf/dir・ancestor reparse・存在/長さ・限定環境・解決toolを返す境界、Hashは既存Get-PathHash相当、Startは固定exe/ArgumentList/cwd/env/log先だけから`{started,handle,startedTick,outTask,errTask,startError}`を返す境界、Pollは同じhandleの終了/exit・stream完了だけを返す。Stopは同じStart返却handleだけを受け取り、PID再open・名前kill・Kill(true)・子列挙0である。Saveは指定pathへCreateNewで一回だけ保存する。production portsはL4内に置き、合成では関数本文をそのまま抽出し全8portsをmemory実装へ差替える。
process構築は既存Start-FixtureProcessのUseShellExecute=False/CreateNoWindow/ArgumentList/非同期stream採取方式だけを再利用し、起動後例外時にDisposeしてthrowするcatchは再利用しない。Startは子起動成功直後から同一Process handleを保持し、stream設定/StartTime等の後続取得が失敗しても`started=true,handle=同一handle,startError=元error`を返す。未設定taskはnullである。呼出側は元errorをprimaryに保持し、そのhandleを通常の異常時回収へ必ず渡す。回収判定前のProcess.Dispose・handle紛失・元errorだけのthrowは禁止する。起動前失敗は`started=false,handle=null,startError=元error`としてStop0、起動後設定失敗はStop最大1・unknown Stop0、終了を証明できなければRecoveryUnknownである。既存helperの変更・移動・同期Complete-FixtureProcessの無期限呼出しは0である。必要なstream完了待機もPollと共通期限で確認し、stdout/stderrの個別Task.Waitを積み重ねて枠を延ばさない。Start自体・FS/OS APIが同期blockする限界は実host gateへ残す。

**時間と回収。** 開始tickはprobe line36直後で一回取得し、絶対上限40秒、作業終了35秒、残5秒は直接handle回収最大3秒＋保存最大2秒である。versionは最大2秒、compileは最大25秒、いずれも作業終了tickとの早い方、境界一致は期限切れとする。全portの前後にNowを照合し、遅延Start/Hash/Save返却でも次段へ進まない。既存run65/build25/watchdog90/終了後11秒を変更せず、今回それらの経路を起動しない。これら新40/35/2/25/3/2秒は準備経路だけの上限である。
version/compileは各Start一回以下、非zero/開始失敗/超過で後続NotRun・retry0とする。compile成功にはexit0と両stream閉鎖、生成exe存在/hash、事前依存の再hash一致が必要である。異常時のStopは未終了の直接handleだけに一回、3秒内の終了が示せなければ`RecoveryUnknown`とする。unknown子は照会・停止せず`NotObserved`、全子回収済みとは記録しない。first error/例外をprimaryとして保持し、停止/保存/後続timeoutはsecondaryへ分離する。保存reserve枯渇では新I/Oを開始せず欠測を返し、既存partial/log/生成exeを削除しない。

**dependencyと保存。** 事前manifest pathは`Pの親/<suite UUID>/prepare-inputs.json`、suiteは従来の`R/verification/<suite UUID>`である。作者packetと将来suiteは別pathとなり、既存OutputRoot新規必須を維持する。manifestは64KiB以下、`schemaVersion=1,scope=PrepareOnly,suiteRoot,files:[{path,sha256}],tools:{rustc,pwsh,v2},environment:{TEMP,TMP,PATH,RUSTUP_HOME,RUSTUP_TOOLCHAIN,CARGO_HOME}`の厳密shapeである。未設定と空を区別し、全環境は収集しない。実toolchain/DLL/sysroot/linker/SDK/Pester等の必要file集合は実OS承認前に親が固定し、欠測なら起動不可とする。
各fileは正規絶対leaf、大小文字を含む期待pathとhash一致、path重複0、全ancestor reparse0、tools解決先一致を要求する。manifest自体も親が外側でhash固定し、内部は読取時hashを保持して前後driftを拒否する。自己採取だけで承認済みとは扱わない。書込先は新suite内だけ、manifestのTEMP/TMPはversion/compile子専用のsuite/temp指定値であり、親TEMP/TMPとの同値は要求しない。他の限定環境値は親からの継承値と照合する。親環境・PATH・OS設定は変更0。suite/tempは作業時間内に一回新規作成する。パス前方一致だけで許可せずsegment境界を照合する。manifest親・source/toolchain・出力先の実ACL/TOCTOU/host制限は未確認であり、合成PASSを実path保護の証明としない。
成功順序はdependency前照合→version→compile→dependency後照合→生成hash→toolchain.json→frozen-inputs.json→生成exe再hash→preparation.json最終保存である。既存context-preflight.jsonは保持する。生成SHAは保存前までPendingGenerationであり旧値/source hashを代用しない。toolchainの既存fieldを保持し、manifest digest・期限・exit/stream/直接回収状態・primary/secondaryはpreparation.jsonへ分ける。各evidenceは指定新pathへ一回、失敗後の同path再保存0である。最後のmarker保存失敗ならAwaitingApproval完了とはせずEvidenceFailedとする。
resultは`{state,dispatchCount=0,retryCount=0,fakeSha256,primaryError,secondaryErrors,directRecovery,unknownDescendants,evidence}`だけとし、stateは`Failed/RecoveryUnknown/EvidenceFailed/AwaitingApproval`である。L4は成功時も`$script:env01Outcome`へ元shapeのNotRun・errorCode=AwaitingApprovalを設定し、dispatch0の準備resultを表示して無条件returnする。元末尾exit1は保持し、exit0/ENV01 PASSを生成しない。失敗も元Errをprimaryへ保存してからthrowし、元outer catchは不変とする。
承認token・生成済みrootの再開・compile skip・元ENV01への接続は今回実装しない。将来の再開には別の明示承認を示すtokenとsuite/source/dependency/fake/保存manifest全hashの一致・drift拒否が必要であるが、現OutputRoot既存拒否との接合は未承認である。未使用のtoken検証関数も追加しない。このscope差分をplannerへ接合する。

| 独立期待群／入力数（計33、各群5秒・集合30秒、firstUnexpected停止／残NotRun／retry0） | memory oracle（U-Mで固定し、実装から生成しない） |
| --- | --- |
| G1／5 | version+compile成功、version非zero、compile非zero、version起動前Start失敗、compile起動前Start失敗。固定argv・各Start回数・後続0・元error保持。起動前失敗はhandle=null/Stop0であり、G3の起動後設定失敗と区別する |
| G2／6 | version2秒一致、compile25秒一致、Start遅延返却、Poll遅延返却、35秒reserve入口、40秒保存枠終了。作業期限後の新Start/作業Hash禁止、回収・Saveは各reserve期限内だけ、dispatch0 |
| G3／7 | 既存5入力はunknown子を伴う直接handle停止、Stop失敗、停止後終了不明、既終了handle、同PID別handle。追加2入力は起動成功後stream設定失敗→同一handleの回収成功と、起動成功後StartTime取得失敗→同一handleの回収不能である。両追加入力は元設定errorのprimary同一性・後続Start0・直接handle Stop最大1・unknown Stop0を確認し、後者はRecoveryUnknownを必須とする。全入力で既知handle以外Stop0・各Stop最大1・unknown保持 |
| G4／6 | dependency欠測、hash drift、prefix衝突path、ancestor reparse、TEMP境界外、manifest shape不正。process Start0・親環境変更0 |
| G5／4 | 保存完了でもAwaitingApproval/dispatch0、外部承認らしいfileがあっても今回経路は読まずdispatch0、生成exe再hash drift、compile後source drift。元ENV01接続/再開を実証したと称さない |
| G6／5 | version元Err＋保存失敗、compile元Err＋保存失敗、成功後最終marker保存失敗、partial write、reserve枯渇。primary同一・secondary分離・再保存0・partial保全 |

U-Mは8ports以外を置換せず、production portsと元ENV01/top-levelは実行せず静的禁止検査で到達0を確認する。U-Nは許可span逆bytes/BOM、PS7.6.7 Parser0/PSSA1.25通常診断0・engine0、native/Test/fake/Invoke不変を証明する。U-O/Pは固定33入力だけを独立実行/レビューし、実OS停止・実toolchain依存・sandbox/ACL・本物compile・ENV01は未検証のgateへ残す。公開signature変更や新再開経路が必要ならその箇所だけ停止して親へ返す。

### U-M着手用：8portsのexact protocol

本表はU-Lのprivate形だけを具体化する。8ports・33入力・許可span・公開signatureは不変である。FB0全文は`R/verification/argv-path01/caa885f1-3d13-44ab-ba1b-7eecc540377f/before/design-FB0.md`へ保存しSHA一致済みである。共通`R<T>={value:T|null,error:ErrorRecord|null}`、errorはcatchした同一ErrorRecord参照を包み直さず保持する。primary/secondaryにも同じ参照を渡し、保存時だけ`{type,fullyQualifiedErrorId,hResult}`へ射影する。合成validation/timeoutは最初に一回生成したErrorRecordを同様に保持する。scriptblock呼出しは下記引数順で一回、不要なpipeline出力0、Now/Start以外はRを返し、throwした場合も元ErrorRecordを同じ失敗として扱う。

| 境界 | 正確な引数 → 返却値・責務 |
| --- | --- |
| Inputs | 既定8keyのみ。startedTick:Int64/frequency:Int64>0、suiteRoot/sourcePath/fakePath:string正規絶対path、fixed:hashtable<path,SHA256>、dependencies:既定manifestをConvertFrom-Json -AsHashtable -DateKind Stringで解析したhashtable、manifestSha256:大文字64hex。fakePathはsuiteRoot/fake.exe、sourcePathはR/fake.rs。manifestPathは`DirectoryName(suiteRoot)/argv-path01/<suite UUID>/prepare-inputs.json`に一意導出する。L3で読取・解析したmanifestのbytes hashを渡し、私関数が同じpathを再読取して一致確認する |
| Now | `()` → Int64 tick、Stopwatch.GetTimestamp相当の単調時計である。frequencyはInputsの同一時計周波数。逆行/型不正はAdapterFailureとし新作業を開始しない。全deadlineはstartedTickからの絶対tick、回収終了はmin(回収開始+3秒,総枠終了-2秒)、保存終了はmin(保存開始+2秒,総枠終了)でありreserveを再起算して延長しない |
| ReadState | `(action:string,arg:hashtable)` → R。`Path,{path}`のvalueは`{fullPath:string,exists:bool,kind:Missing\|File\|Directory,length:Int64\|null,ancestorReparse:bool}`。`Manifest,{path}`は最大65537bytesまで読んだbyte[]（65536超は拒否、読取失敗はerror）。`Context,{}`は`{tools:{rustc,pwsh,v2},environment:{PATH,RUSTUP_HOME,RUSTUP_TOOLCHAIN,CARGO_HOME}}`で値は絶対path又はstring/null、tool欠測はnullである。全actionは読取専用、temp作成は行わない |
| Hash | `(path:string)` → Rのvalue `{path:string,sha256:大文字64hex}`。leaf読取だけである。manifest raw bytesのdigestは私関数内でSHA256計算し、Hash(manifestPath)と事前/事後照合する。fixedとdependencies.filesの和集合（同一pathの期待hash不一致は拒否）、sourcePath、manifestを前後照合し、未知file探索0 |
| Start | `(spec:hashtable)` → `{started:bool,handle:object\|null,startedTick:Int64\|null,outTask:object\|null,errTask:object\|null,startError:ErrorRecord\|null}`。specは`{phase:Version\|Compile,exe:string,argv:string[],cwd:string,environment:hashtable,stdoutPath:string,stderrPath:string}`だけ。Version argv=`--version`、Compile argv=`--edition=2021,sourcePath,-o,fakePath`、exe=dependencies.tools.rustc/cwd=suiteRoot、environmentは既定6keyでTEMP/TMPだけsuiteRoot/tempへ上書きする。nullは子environmentからRemove、空stringは空値設定、他keyは元の継承値を保持する。logはVersion=`rustc-version.stdout/.stderr`、Compile=`fake-build.stdout/.stderr`をsuite直下へCreateNewする。startedTickは成功直後の同一時計、起動前失敗はhandle=null/startedTick=null、起動後設定失敗でもhandle保持・startError返却・throw/Dispose0。返却handleからstream等の資源を引く対応表はproduction port closureが所有する |
| Poll | `(handle:object)` → Rのvalue `{exited:bool,exitCode:Int32\|null,outClosed:bool,errClosed:bool,stdoutText:string\|null,released:bool}`。未終了exitCode=null、stdoutTextは終了＋両stream閉鎖時だけ（VersionはUTF-8 decode結果、Compileはnull）。Startで生成できなかったtaskは閉鎖扱いとするがstartErrorは消さない。終了/両stream閉鎖を確認した最後のPollは対応資源とProcessをDisposeしてreleased=trueを返す。以後同handleのPoll/Stop0。Task faultや観測/解放失敗はerrorへ返しhandleを保持する。これは実process回収証明とは別である |
| Stop／Wait | `Stop(handle:object)` → Rのvalue=null。同一Start成功handleへKill(false)一回だけであり、終了/stream待機はPollで行う。Stop成功自体を回収成功としない。`Wait(milliseconds:Int32)` → Rのvalue=null、値は1～50かつ残りdeadline以下、sleepだけである |
| Save | `(action:string,arg:hashtable)` → Rのvalue `{path:string,status:Created\|Saved\|Partial\|Missing,sha256:string\|null,length:Int64\|null}`。`CreateTemp,{path}`は前照合成功後・version前にsuiteRoot/tempを一回新Directory作成するだけ（既存はerror、Created/sha256=null/length=null）。`Record,{path,bytes:byte[]}`はtoolchain/frozen-inputs/preparation各.jsonへCreateNew→write_all→flushだけでSaved時は渡したbytesのhash/長さを返す。失敗も判明範囲のPartial/Missing valueと同じerrorを返し、再open/修復/再試行0。ReadStateには書込を追加しない |
| evidence／保存payload | evidenceは`[{kind:Temp\|Toolchain\|FrozenInputs\|Preparation,path,status,sha256,length}]`、値はSave返却snapshotだけである。Record bytesはUTF-8 BOMなしConvertTo-Json -Depth 20 -Compressの出力。toolchainは既定旧fieldだけ、versionはVersion Poll.stdoutText、frozen-inputsは生成hash追加済みfixed。preparationは`{schemaVersion:1,manifestSha256,startedTick,frequency,limitsMs:{total:40000,work:35000,version:2000,compile:25000,recovery:3000,save:2000},phases:[{phase,startedTick,exitCode,outClosed,errClosed}],result}`。このresult.evidenceは自己保存前のsnapshotとし、自己hash循環を作らない。logsの生bytesはStart側の原本に保持し、この表へ重複収集しない |
| result／優先順 | 既定9keyのみ、state:string、dispatchCount/retryCount:Int32=0、fakeSha256:string\|null（未取得null）、primaryError:ErrorRecord\|null、secondaryErrors:ErrorRecord[]、directRecovery:NotStarted\|Exited\|Recovered\|Unknown、unknownDescendants:string=NotObserved、evidence:上記配列。version/compileのいずれか未回収ならdirectRecovery=Unknown、stateはRecoveryUnknown優先、次に必要Record保存未完了EvidenceFailed（入力拒否で保存未許可の場合は除く）、その他primaryありFailed、全成功AwaitingApproval。出力JSONはerror参照だけ上記scalarへ射影し、生参照は返却result内へ保持する |

ReadState/Hash/Saveの型不正・未知action・path逸脱は、起動前拒否なら累積Start0、起動後拒否なら以降の追加Start0とし、既存の同じhandle回収を優先する。Save/CreateTempの責務追加は既存8ports内の指定path新規保存であり9portや汎用FS APIを導入しない。manifest/実toolchain/実path保護の成立、実再開、実process停止は本追補でも未検証である。

### U-M最終Pre差戻し：合成primaryの原因識別

既定原因を観測可能にするL2内のprivate契約だけを追加する。固定oracle SHA256 `CAA26281E5EF932156118720997491AE1E3EB55ED9DF0365F32197C6F8979353`の33入力・expect.primary16件は変更しない。設計beforeは`R/verification/argv-path01/b80899de-9c23-47b3-99dc-865e91730957/before/design-9106.md`へ保存しSHA一致済みである。8ports/公開param/result9key・期限・元error同一参照・primary/secondary順序は不変である。
L2自身がvalidation/期限/非zeroexitを検出した場合だけ、最初に一回`ErrorRecord::new(exception,id,category,$null)`を生成し、その同一参照をprimary/secondaryへ保持する。idは厳密に`WW5.ENV01.Preparation.`＋下表suffixであり、FullyQualifiedErrorIdを直接照合する。Messageは自由な診断文で判定に使用しない。非zeroexitはInt32値のInvariantCulture十進表記をsuffixへ含める。deadlineはTimeoutException/OperationTimeout、exitはInvalidOperationException/InvalidResult、Dependency.MissingはInvalidOperationException/ResourceUnavailable、その他validationはInvalidOperationException/InvalidDataとする。元port ErrorRecordの再生成・Exception/Data/FQID変更・合成id付替えは禁止し、既存のprimaryIdentity/secondaryIdentity照合を維持する。

| 既存expect.primary／全16入力 | 固定suffixと識別する既存事実 |
| --- | --- |
| version exit 17／G1-02 | `NonzeroExit.Version.17`：Versionの終了観測exitCode=17。一般形は`NonzeroExit.Version.<Int32>` |
| compile exit 23／G1-03 | `NonzeroExit.Compile.23`：Compileの終了観測exitCode=23。一般形は`NonzeroExit.Compile.<Int32>` |
| version deadline／G2-01,G2-03,G3-02 | `Deadline.Version`：Versionのphase期限到達。遅延Start/Poll返却も含む |
| compile deadline／G2-02,G2-04 | `Deadline.Compile`：Compileのphase期限到達。遅延Poll返却も含む |
| work deadline／G2-05 | `Deadline.Work`：次の作業前に全作業期限到達。phase上限をwork期限が切り詰めた場合もWorkとする |
| missing dependency／G4-01 | `Dependency.Missing`：必要fileのexists=false又は必要pathが依存集合に存在しない |
| dependency drift／G4-02,G5-04 | `Dependency.Drift`：既定dependency/manifest/sourceの期待hashと観測hash不一致。前後どちらのpassでも同じ識別子 |
| path boundary／G4-03 | `Boundary.Path`：正規絶対path・期待path・許可segment境界の違反 |
| reparse boundary／G4-04 | `Boundary.Reparse`：ancestorReparse=true。単なるpath不一致へ合算しない |
| child TEMP boundary／G4-05 | `Boundary.ChildTemp`：子TEMP/TMPがsuiteRoot/tempへ一致しない |
| manifest shape／G4-06 | `Manifest.Shape`：manifest自身のparse/厳密key集合/field型/値の契約違反。port応答shape不正のAdapterFailureとは区別する |
| generated drift／G5-03 | `Generated.Drift`：取得済みfakeSha256と保存後生成exe再hashが不一致。dependency用hash不一致へ合算しない |

判定位置は原因を検出した元分岐であり、catch後のMessage文字列分類は行わない。生成元ErrorRecord参照を保持してから制御を短絡し、throw/catchで得たwrapperをcanonical errorへ取り直したり同じ原因のsecondaryへ重複追加したりしない。path観測はport元errorを先に保持し、正常応答に対するexists=false／reparse／path違反を個別に識別する。時刻境界一致・先行primary保持は従前どおり、先行port元errorを後続timeoutで置換しない。AdapterFailureを上表idに変換しない。上表外の障害のoracleや汎用error体系を増やさない。
作者handoffは独立assertion内で固定expect.primaryの各値を上表suffixへ明示対応させ、actual.primaryErrorがErrorRecordかつFullyQualifiedErrorIdが完全一致することを確認することである。16件すべてにこのassertionが適用された数を静的に示し、期待本文・33入力は不変とする。idは候補Message/実結果/ソース抽出から生成しない。既存保存射影のfullyQualifiedErrorIdにも同じidが残るため、resultやwireへ新keyは不要である。実装者はL2の原因検出分岐だけを修正し、source/adapterの候補hash再束縛と独立Preは親が接合する。本工程は設計差分・read/hash確認だけでruntime/OS/source編集0である。

## PSSA-CAP：Stopを維持する逐次採取の限定設計

CAP-Dはplanner指定`gpt-6.1-sol/high`である。before全bytesはT/`before/cap-design-47827255-2459-453a-b1cb-7ef577fcd5f5/design-B45A.md`へ保全しSHA256 `B45A320172B6158B39D92F8EB02917A46BB56FCA72B5DC1DD8B634E7CE9B13CE`一致済み。Tは計画PSSA-CAPの既存packetである。所有は本追補だけ、コード/設定/新診断/新PSSA実行/再委任0である。旧2回のengine2は同一errorの複数採取であり独立2障害と扱わず、Continue/Verbose診断の未再現理由は未確定とする。save-only-state/indexは非freezeの停止状態を維持する。
根拠は保存request/toolreturn2JSONと、ローカルPSSA1.25.0のmanifest/psm1/Help.xmlである。Helpの既定全規則・Severity未指定時全診断・DiagnosticRecord返却、PSv7 binaryのloadを読取確認した。[CommonParameters](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_commonparameters?view=powershell-7.6)はStopによる非終端errorの昇格とErrorVariableの追加採取を規定し、[PSSA原典](https://learn.microsoft.com/en-us/powershell/utility-modules/psscriptanalyzer/using-scriptanalyzer?view=ps-modules)はParseErrorも診断へ出す。[Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection?view=powershell-7.6)に従いstream1～6を扱うが、Progressの完全採取は約束しない。これらは採取意味の根拠であり、今回PSSA内部故障の解消証明ではない。

| CAP要求／境界 | 固定契約（採取器C=`T/pssa-capture/Invoke-PssaCapture.ps1`一つだけ） |
| --- | --- |
| R1／呼出し | PS7.6.7、Import-Module PSScriptAnalyzer -RequiredVersion 1.25.0 -ErrorAction Stop。production producerは`[CmdletBinding()]param([hashtable]$Context)`の本体で`Invoke-ScriptAnalyzer -Path $Context.targetPath -ErrorAction Stop`を一回だけ呼ぶ。下記private pipelineがproducer呼出しにErrorVariable/stream統合を付けて逐次sinkへ渡す。完了前の`$diagnostics=@(...)`代入は使わない。Include/Exclude/Severity/Settings/CustomRulePath/Fix/SuppressedOnly/IncludeSuppressed/EnableExit/Recurseは追加0、Verbose/Debugの有効化・preference変更0。既存default規則・既存抑制状態・全severityを保つ。全元規則の完走は正常return契約に依存し、Verbose件数やordinary0から推定しない |
| R1／typed判定 | CLRの`Microsoft.Windows.PowerShell.ScriptAnalyzer.Generic.DiagnosticRecord`はSeverityを選別せずordinaryへ入れる（Error/Warning/Information/ParseErrorを含む）。ErrorRecordは経路に関係なくengine。VerboseRecord/DebugRecord/InformationRecordだけをauxiliaryとし、WarningRecordは保守的に非PASSのsignal、その他object/null/型不正はunknownで非PASS。文字列MessageやPSObject.TypeNamesだけでDiagnosticRecord/errorへ擬装しない。PSSA型の解決失敗はmodule failureである |
| R1／完了と判定 | flagsは`moduleReady,invokeStarted,invokeReturned,captureComplete,postBindingsMatched,savedWithinDeadline`のboolのみ。invokeReturnedは対象pipelineが例外なしに返った直後だけtrue、catch/finallyだけでtrueにしない。captureCompleteは全観測recordの射影・末尾ErrorVariable/catch回収が上限内で完了した場合だけtrue。PASSは全flag=true、ordinary/engine/unknown/signal各0、overflow/truncation各false、元期限内、raw/result保存成功、実host exit0の全条件。採取完了とgate PASSを別欄に置く。部分出力後の例外・正常return欠測・module/採取/保存errorは必敗である |
| R2／原本と順序 | 各観測を`{seq,origin,type,kind,fields}`へ順番に記録し、originはPipeline/ErrorVariable/Catch/Import/Capture/Saveである。逐次pipelineを先に保持し、finallyでErrorVariableを元順に追加、その後caught recordを追加する。originを跨ぐseqは採取順であってengine発生順を偽らない。in-memoryでは全ErrorRecordとExceptionの元参照を保持し、first engine errorを保存errorで置換しない。保存errorはsecondaryへ分ける。逐次sink自体の失敗もcapture errorとして別保持し、既得recordを捨てない |
| R2／重複関連付け | raw occurrenceは重複でも一件も間引かない。`links=[{fromSeq,toSeq,relation}]`は同一ErrorRecord参照、同一Exception参照、ActionPreferenceStopException.ErrorRecordの直接参照一致だけで結ぶ。Message/FQID/stackの文字列一致だけで統合0。raw engine occurrence数と参照で関連したgroup数を別記し、未関連を独立原因と断定しない。捕捉wrapperの直接InnerExceptionも型/HResultだけ保存し、再帰探索0。旧保存JSONの文字列だけから失われた参照関係を復元したとは称さない |
| R2／有限保存 | rawは生objectの無制限serializeではなく、型名と固定scalarの観測snapshotである。Diagnostic fields=`RuleName,Severity,Message,ScriptPath,StartLine,StartColumn,EndLine,EndColumn`、Error fields=`exceptionType,hResult,fullyQualifiedErrorId,category,message,exceptionStack,scriptStack,position,innerType,innerHResult`、auxiliary/unknownは型とToString結果。未存在propertyはnull、getter/ToString例外はcapture failureとして元record参照と共に残す。最大256 occurrences、各string8192 UTF-16、raw本体UTF-8 2MiB以内＋失敗summary用64KiBを別reserveとする。上限を越える前に採取を停止し、取得済みprefix/越境record型/不足範囲/overflow又はtruncationを明示して必敗とする。無限rawやOOM回避の保証ではなく、記録量の上限である |
| R3／bindingと時刻 | authorの`T/pssa-capture/capture-binding.json`にcwd=`F:/Kazuhiro/GitHub/WordWeave5`、C/Test/期待表・target2本の完全hash、pwsh.exe/pwsh.dll/SMA.dll、PSSA psd1/psm1/types/format/PSv7依存DLLの固定path/hash、default規則/Severity未指定・元command・preference、対象pathのsettings有無/hashを束縛する。module load実体/version・実PS・全argvを照合し、同じ集合を前後hashする。未知依存・取得不能・driftは欠測/非PASS。AssertionsはAFA82E807179DF852ECCB05D5A8C8F3919B9B1C4C9D981CFA0CD6F368FE824F4、F91C/33期待/旧receiptに変更0。targetをdot-source/importしない |
| R3／期限と保存 | monotonic開始は各入力のimport/前hash前、work9秒＋保存reserve1秒を既存入力10秒内に収め、PSSA2入力の集合30秒を共通startTickから測る。境界一致は超過、全record受信・正常return・posthash・保存前後に確認し、遅延returnも非PASS。同期PSSA/FS自体のhard中断は保証せず、tool yieldを上限としない。hostが中断した場合は不完了/保存欠測でありPASS0。外側の監視・可能な取消要求・後続停止責任者は実行Preで親が記録し、無断の監督process/timeout延長0。保存は新OutputRootへraw.json→result.json各CreateNew一回、失敗時partial/prefixと元errorを保持、再保存/削除0。result.jsonは`gateCandidate`とraw hash/flags/counts/links/欠測を記録するだけで、保存後clock確認と実exit0を伴わなければ受入不可。自己hash循環やdisk上の仮PASSによる救済0 |
| 合成seam／所有 | C内private関数`Add-PssaCaptureRecord(state,origin,record)`、`Invoke-PssaCapturePipeline(state,producer,context)`、`Complete-PssaCapture(state,flags,now,save)`と初期state statementを一意AST抽出し、元bytes/hash/逆一致を固定する。pipelineの実際のproducer呼出し/逐次sink/EV/catch/finallyは抽出原文のまま通し、producerだけmemory scriptblockへ束縛する。now/saveも下記型のmemory実装とする。PSSA1.25の実DiagnosticRecord/error/stream型をmemory生成し、module型読込以外のInvoke-ScriptAnalyzer・C top-levelはunit内0。pipelineを手動のrecord投入/完了flag設定で代用0である |

是正P1/P2のbefore全bytesはT/`before/cap-design-fix-2a3689d8-1d9b-4dd9-a6ae-54a0261f8e42/design-20244.md`へ保全し、SHA256 `20244CD9928264BAF527572FB0551D37D05C504189B8FF958124B181E6DB0B8E`一致済みである。以下は上記seamと既存20期待の接合だけを具体化し、規則/severity/Stop/期限/外側中断gateを変更しない。

| P1/P2の最小shape／責任 | 固定契約 |
| --- | --- |
| state／初期値 | 必須keyは`targetPath,outputRoot`(string)、`frequency,inputStartTick,collectionStartTick,rawBytes`(Int64、rawBytes初期0)、`now`(無引数scriptblock→Int64 tick)、`records,recordRefs,links,secondaryErrors,missing,artifacts`(それぞれList[hashtable]/List[object]/List[hashtable]/List[ErrorRecord]/List[string]/List[hashtable]、初期空)、`flags,counts`(下記hashtable)、`primaryError`(ErrorRecord又はnull、初期null)、`overflow,truncation,captureBroken,finalized`(bool、初期false)だけ。public ABIへは出さない |
| flags/counts／所有 | flagsは既定6keyすべてbool/初期false。moduleReady/postBindingsMatchedはdriverのimport/前後照合だけが更新、invokeStarted/invokeReturned/captureCompleteはprivate pipelineだけが更新、savedWithinDeadlineはCompleteだけが更新する。countsは`ordinary,engineOccurrences,engineGroups,unknown,signal,auxiliary`のInt32/初期0で、Addだけが更新する。20期待はこの名前へ照合し、初期値をsuccessにしない |
| Add／返却とraw | Addはpipeline出力0、stateをin-place更新する。recordsの各要素は既定`{seq:Int32(1始まり),origin:string,type:string,kind:Diagnostic/Engine/Auxiliary/Signal/Unknown,fields:hashtable}`だけ、fieldsのkeyは上記型別固定集合、auxiliary/unknownは`{text:string又はnull}`。recordRefs[seq-1]へ同じ生object参照を置きserializeしない。linksは`{fromSeq:Int32,toSeq:Int32,relation:SameRecord/SameException/StopWrapperRecord}`だけ。engineGroupsはこの参照linksの連結群数、文字列一致は使わない。最初のErrorRecord参照をprimaryErrorへ保持し、以後の別参照errorをsecondaryErrorsへ保持する。射影失敗/上限はcaptureBroken=true・missingへ不足箇所、overflow/truncationへ事実を残して短絡し、元errorを置換しない |
| pipeline／PS scope | 返却pipeline出力0。関数ローカルに`$engineRaw=@(); $caught=$null`を宣言し、GetNewClosure/別runspace/module/script scopeへ移さず、try内でinvokeStarted=trueの後、厳密に`& $producer -Context $context -ErrorAction Stop -ErrorVariable +engineRaw *>&1`をForEach-Objectの逐次`$null=Add-PssaCaptureRecord ... Pipeline $_`へpipeする。そのpipelineが正常に戻った次statementだけinvokeReturned=true。catchは同じErrorRecordを$caughtへ保持し、finallyがローカルengineRawを元順にAdd(ErrorVariable)、次にcaughtがあればAdd(Catch)する。CommonParameterのErrorVariableはこの呼出元関数のローカルengineRawを採取先とし、producer本体に同名変数や別EVを作らない。finallyの末尾でcaptureBroken/overflow/truncationが全falseの場合だけcaptureComplete=true。finally/Addの失敗はcaptureBrokenとsecondaryへ残す |
| producer／C07の接合 | producerは必ず上記advanced scriptblockでContext一引数、production contextは`{targetPath:string}`だけ。C07のmemory contextはこれに`diagnostic:実DiagnosticRecord,error:同一ErrorRecord`を加え、producer本体は`Write-Output -NoEnumerate $Context.diagnostic`の後に`$PSCmdlet.ThrowTerminatingError($Context.error)`を実行する。捕捉側は実際のthrowを経由し、先行ordinary1・EV originが1件以上・Catch origin1・invokeReturned=falseを確認する。wrapperリンクは実参照から判定する。testがC07のinvokeReturnedやcaptureCompleteを直接代入すること、EV配列を手で埋めることは禁止する。これでproductionと同じ逐次採取/EV scope/catch/finallyの故障窓を検証する |
| Complete／引数と返却 | flagsはstate.flagsと同じhashtable参照、nowはstate.nowと同じscriptblock参照、saveは`(absolutePath:string,bytes:byte[])->{path:string,sha256:大文字64hex,length:Int64}`又は元ErrorRecordをthrowするCreateNew sinkである。Completeは一回だけ呼びfinalized=trueとし、既定時刻/保存/非PASS判定を行い、単一hashtable `{captureStatus:string,gateStatus:string,flags:hashtable,counts:hashtable,rawRecords:object[],links:object[],primaryError:ErrorRecord又はnull,secondaryErrors:ErrorRecord[],overflow:bool,truncation:bool,missing:string[],artifacts:object[]}`を返す。その他出力0。flags/counts/配列は返却時snapshot、errorは元参照である。captureStatus=CompleteはmoduleReady/invokeStarted/invokeReturned/captureComplete全true、それ以外Incomplete。gateStatusは既定の全PASS条件を満たす場合だけPASS、それ以外FAIL。保存完了後のclock確認を終えるまでsavedWithinDeadline=falseを維持する |
| 保存shape／20期待の照合先 | artifacts各要素は`{kind:Raw/Result,path:string,sha256:string又はnull,length:Int64又はnull,status:Saved/Incomplete}`。raw.jsonは`{schemaVersion:1,targetPath,records,links,overflow,truncation,missing}`、result.jsonは`{schemaVersion:1,targetPath,gateCandidate,flags,counts,rawFile:{path,sha256,length},links,primaryError,secondaryErrors,missing}`だけ。errorは既定固定scalarへ射影し、生参照は返却内だけに置く。disk gateCandidateは保存前条件だけの候補、flagsはその時点snapshotであり最終savedWithinDeadlineの証拠にしない。20期待のordinary/raw/group/unknown/signalは返却counts/rawRecords/linksへ、元error保持は返却参照へ、完了/期限は返却flags/statusとmemory clockへ、partial/新保存0はmemory save呼出履歴とbytesへ照合する。save失敗後にsealed rawを書換えず、不足を返却missingへ記す |

独立期待は以下20入力を各一回、各5秒・純合成集合30秒で固定する（PSSA入力10秒/集合30秒とは別の時計）。expected FAILを正しく検出した対照はunit PASSであり、firstUnexpectedは期待不一致/adapter failure/保存不全/超過のことである。以後NotRun、retry0である。

| 入力ID | 独立観測／期待 |
| --- | --- |
| C01～C05 | C01=全flag正常・record0→採取Complete/gate PASS候補。C02=Error診断、C03=Warning診断、C04=Information診断、C05=ParseError診断→各ordinary1・gate FAIL。全severityの見落しを検出する |
| C06～C08 | C06=同じ非終端ErrorRecordをPipeline/EVへ各一回→raw2/group1・FAIL。C07=上記advanced memory producerが本当にordinaryを出力してからthrowし、原文抽出pipeline自身がEV/catch/完了flagを更新する→ordinary保持/raw双方保持/invokeReturned=false/FAIL。C08=同じ本文/FQIDの別error参照2個→raw2/group2、文字列dedup0・FAIL |
| C09～C12 | C09=import元error→invokeStarted=false/元参照保持。C10=PS/module/path/hash不一致→開始0/FAIL。C11=未知object→unknown1/FAIL。C12=snapshot getter失敗→既得record/元参照保持・captureComplete=false/FAIL |
| C13～C16 | C13=engine元error後にsaveがpartialを書いて失敗→元error primary同一/保存error secondary/partial保持/FAIL。C14=posthash drift→postBindingsMatched=false/FAIL。C15=入力10秒一致→FAIL・新保存0。C16=集合30秒一致→FAIL・後続NotRun |
| C17～C20 | C17=257件目又は文字/byte上限の最初の超過→prefixと欠測保持/overflow又はtruncation/FAIL（authorはrecord数超過をこの一入力に選ぶ）。C18=record0だが正常return flagなし→Incomplete/FAIL。C19=保存開始時は期限内だが返却時に10秒到達→diskにgateCandidateがあってもFAIL。C20=VerboseRecordの後にWarningRecord→双方raw保持・signal1/FAIL |

CAP-Tは20入力・既定集計・command/入力hashを先に凍結し、CAP-IはC一つだけを実装する。CとTestのParser0をread-onlyで確認してから独立Pre/親freezeへ進む。新PSSAの実施順はunit20→C自身一回→固定Assertions一回、いずれかの未達で後続NotRunである。self PSSAの結果を捨ててAssertionsだけ実施0。下記はファイル完成/hash接合/Pre前の実行不可command案であり、新しい実行許可ではない。shellは固定`F:/Program Files/PowerShell/7/pwsh.exe`、login=false、cwdは上記repoである。OutputRootの親`pssa-capture/out`はCAP-Tが用意し、各leafは実行前未存在を確認する。

```powershell
$capRoot='F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d/pssa-capture'
& "$capRoot/Test-PssaCapture.ps1" -BindingPath "$capRoot/capture-binding.json" -OutputRoot "$capRoot/out/unit-6df5fb22-633a-41a3-b36a-e2b7c89cf1eb"
if ($LASTEXITCODE -ne 0) { exit 2 }
$capPssaStart=[Diagnostics.Stopwatch]::GetTimestamp()
& "$capRoot/Invoke-PssaCapture.ps1" -BindingPath "$capRoot/capture-binding.json" -TargetRole Collector -CollectionStartTick $capPssaStart -OutputRoot "$capRoot/out/self-f2200615-9a3d-4339-a41d-9cd78330b694"
if ($LASTEXITCODE -ne 0) { exit 2 }
& "$capRoot/Invoke-PssaCapture.ps1" -BindingPath "$capRoot/capture-binding.json" -TargetRole Assertions -CollectionStartTick $capPssaStart -OutputRoot "$capRoot/out/assertions-30f73aec-dd59-445a-9b9c-aec3a15df450"
exit $LASTEXITCODE
```

Cのprivate CLIは上記BindingPath/TargetRole(Collector又はAssertions)/CollectionStartTick/OutputRootだけとし、targetと期待SHAをmanifestのroleから解決する。Test CLIはBindingPath/OutputRootだけ、unitの自己startTick/frequencyは同一hostで採る。C/Testは正常受入0、その他2で返し、runnerは実exitと全保存hashを別記する。単純スクリプト呼出しだけで新module/監督process/公開ABIを作らない。CAP-V Postは「採取契約の局所対照」と「今回Assertionsのgate」を分離し、新方式が正常でも旧2FAIL・原NullReference原因未確定・33候補未解除を保持する。本工程で前後hashを再実測したのは設計と現Assertionsだけであり、runtime/module load/新採取成立は未検証である。

### CAP実行Preの期限責任の限定確定

今回必要なのは**親の監視・期限超過非PASS・後続停止責任の接合**であり、10秒以内の実処理切断保証ではない。根拠は、旧T/Invoke-PreparationReceipt.ps1:116–130がnative直接呼出しの返却後にelapsed>=30を拒否し、`no hard OS cancellation claim`と明記すること、CAP計画が元10/30を維持し新監督processを対象外とすること、上記R3が同期hard中断を明示的に非保証とすることである。旧receiptの終了後elapsed・yield・session IDから同一handle保持や強制停止能力を推定しない。この解釈はCAPの受入deadlineを具体化するもので、実OS ENV01/compile向けU-Hの未解決な有限停止条件を解除しない。
実行Preでは親/runnerが、(1)監視担当のtask/役と固定command/出力先、(2)入力/集合の共通startTick・frequency・限界値・境界一致拒否、(3)使用hostの自動hard上限と取消機能について証拠のある能力又は「自動hard停止なし／未提供・未確認」をそのまま記録し、(4)超過/応答停止の観測時点から新dispatch/retry0・残NotRun・非PASS・原ログ/partial保全とする責任を記録する。これが計画の未接合な外側責任/host上限/欠測扱いへ渡す最小接合である。能力欄へ保証値を創作せず、担当者名だけでhard停止保証済みとも扱わない。
実行中はhostが実際に返す進捗/session/終了情報で監視し、超過観測又は中断では不完了とする。既存hostの取消機能が実在し許可範囲内ならその機能だけで取消要求し、要求/応答/実終了の確認可否を分ける。取消手段がない・終了未確認ならその事実をblockerとして報告し、後続はNotRunのまま、起動済み処理が停止済みとは称さない。最終正常返却でも入力10秒/集合30秒未満、保存・hash・完了条件を満たさなければ非PASSである。永久blockからの自動回復は今回の保証範囲に含めず、追加待機を受入期限の延長へ使わない。
10秒以内の実切断までを新たに要求する場合は、専用pwsh子のHidden/非admin起動・同一handle保持・非同期stream保存・期限Stop/終了確認・回収reserve/不明残留を管理する新harness authorityとなるため、別設計と利用者判断へ戻す。今回その実装・子起動/停止・設定変更は0である。本補足は期限値・20期待・非PASS条件を変更せず、独立Pre/親freezeの他条件や新実行許可を代替しない。

### C16限定：実exit判定からcaller後続停止への接続

固定C16の「集合30秒一致→FAIL・後続NotRun」は変更しない。初期followup=NotRunの比較だけでは充足しないため、実Complete→Cの実exit tail→実CAP-V caller分岐を同一host内の限定adapterで通す。既存3関数/C07/20期待/期限は不変、新native process/module/framework/公開ABIは追加0である。

| 原文と所有境界 | C16専用の固定adapter契約 |
| --- | --- |
| Cの実driver tail／implementer | Cのmain末尾は`$captureResult = Complete-PssaCapture -State $state -Flags $state.flags -Now $now -Save $save`→`if ($captureResult.gateStatus -ceq 'PASS') { exit 0 }`→`exit 2`の一意連続statementとする。Complete自体は既定本文であり、追加のexit判定関数は作らない。C16はmemory nowで集合30秒一致を与え、原文Completeが返した同じcaptureResultをこの判定へ渡す。Test側でgateStatusや要求exit値を生成/差替え0 |
| 実caller／author | 抽出元は将来runnerが実際に使うCAP-V receiptのcommand本文にあるCollector呼出し→直後`if ($LASTEXITCODE -ne 0) { exit 2 }`→Assertions呼出し→最終`exit $LASTEXITCODE`の連続spanである。設計code blockの手写しifで代用0。CollectorのTargetRole/argvを固定照合し、この実caller spanをそのまま実行する。今回まだC/実callerが未生成ならsource/hash/spanはPendingBindingであり、期待Preではこの選択規則/許可置換を固定するだけ、実行前に現物の一意ASTと全bytes/hashへ接合する |
| 生exitの観測／test-only | C tail/caller span内のExitStatementAstだけを一意抽出し、各`exit <expr>`を`& $exitProbe (<exprの元bytes>)`へ置換可とする。exitProbeは要求code:Int32とstage=Collector/Callerをledgerへ追記し、Collector時だけGet-Variable -Name captureResult -Scope 1 -ValueOnlyで呼出元の実Complete返却参照もledgerへ保持する（取得不成立はAdapterFailure）。その後、adapterが一回生成して保持する専用Exception参照をthrowする。直近adapterだけが同一Exception参照（PS wrapperなら直接InnerExceptionの同一参照）を捕捉し、そのstageの観測exitを返す。Message一致catch0、他の例外はAdapterFailure。if条件/Complete呼出し/元exit式は変更0、生exitをunit hostへ到達させない |
| Collector呼出しの置換／test-only | callerのCollector command一箇所だけを「CollectorCallsを追記→上記原文C tailをmemory state/now/saveで実行し専用exitを捕捉→観測した実要求codeだけを同じcaller scopeのLASTEXITCODEへ渡す」statementへ置換可とする。codeを2等の固定値で代入0、Completeを再実装0。これはnative返却境界のmemory代替であり、実exit判定はC原文である。Testは独立した呼出scopeで実行し、呼出前のLASTEXITCODEをfinallyで復元する。別runspace/process/moduleは不要である |
| Assertions呼出しの置換／test-only | callerのAssertions command一箇所だけを、AssertionsCallsへ実呼出を追記するmemory ledger statementへ置換する。事前に呼ばれた扱いにせず、呼出時だけ追記し、LASTEXITCODE/分岐を変更しない。CAP-V callerのif/exit以外の制御構造置換0。期待はCollectorCalls=1、原文CompleteのgateStatus=FAIL、観測Collector exit=2、原文callerの停止exit=2、AssertionsCalls=0である。followupはこの観測を照合した後にだけNotRunとし、初期値/手書き停止ifによる証明を禁止する |
| binding／停止 | 作者manifestへC/callerそれぞれのsource SHA、親AST・元span全文・offset単位/length/byte SHA、全置換文と個数、生成adapter SHAを固定する。C tailのexit2個、callerのexit2個、Collector/Assertions command各1個だけを上記境界で置換し、逆適用で両元span全bytes一致を必須とする。差異/余分なexit/曖昧AST/未束縛はAdapterFailure又はPendingBindingであり、C16 PASSへ変換0。C16の固定期待や候補値からの期待生成は追加0 |

実装前期待Preへ渡せるのは上記adapter選択/置換/観測規則であり、C16意味充足は実C/caller接合後の一回観測までPendingである。旧REDだけを先行する親の独立Pre/一回freezeを妨げず、旧REDの18pinやsource/test本文は本工程で変更0である。

### CAP-B-R1-DIAG：outer catchだけの限定観測

BD-Dはplanner指定`gpt-6.1-sol/high`、入力は計画CAP-BDと旧RUN1 report/原unit-resultである。旧Test SHA256 `4DF81C43AF2085AC9E21FE346410C51C4DADECFFD224B80E3F905C8C3239B144`の196–198には無型entryへのException参照があるが、旧結果はerror本文だけで実位置/entry型を欠く。候補箇所を原因確定せず、旧exit2/Unexpected/accepted=false/slot消費1/elapsed1.1109889を保持する。Test外側catchだけでは既にunwindしたInvoke-CapOldRed/Where-Object内のentry参照を取得できず、標準ErrorRecord.TargetObjectがそのentryである保証もない。今回のfailureObservationでentry同一性を推定しない。

| 契約 | 最小観測・不変条件 |
| --- | --- |
| 許可span | 作者は新UUID artifactへTest全bytesをコピーし、baseline outer main catch（585以降、`$report = @{ accepted = $false; classification = 'Unexpected'; error = $_.Exception.Message; requestedExit = 2 }`と`$requestedExit = 2`を持つ一意catch）内だけを追加変更する。最初に`$outerFailure = $_`を保持し、元2statementをそのまま実行した後、局所try/catchで`$report.failureObservation`を追加する。他のcatch/Where-Object/engineErrors/Exception参照/元Try/producer/prefix比較/期待RED/exit/保存statementは全bytes不変。cast/skip/dedup/bookmark、新helper/module/汎用採取器は追加0 |
| 固定scalar | failureObservationは`{errorRecordType,exceptionType,hResult,fullyQualifiedErrorId,category,scriptStack,position,scriptName,line,column,targetObjectType,failedEntryType,failedEntryStatus,missing,truncated,captureFailure}`だけ。前11fieldは保持したouter ErrorRecordのCLR型、ExceptionのCLR型/HResult、FullyQualifiedErrorId、CategoryInfo.ToString、ScriptStackTrace、InvocationInfo.PositionMessage/ScriptName/ScriptLineNumber/OffsetInLine、TargetObjectのCLR型を対応させる。line/columnはInt32又はnull、hResultはInt32又はnull、他はstring又はnull。TargetObjectはGetType().FullNameだけで、値/ToString/entry本文/任意propertyを列挙0 |
| 型の判定限界 | failedEntryTypeはnull、failedEntryStatusは厳密にUnidentifiedFromOuterCatchとする。targetObjectTypeが取得できてもfailedEntryTypeへ転記しない。scriptStack/position/line/columnは記録した元位置として保存し、196–198と一致するときだけその箇所まで到達した観測とする。外側呼出し行しか示さない場合は位置不明を保持する。元の欠測を推測で補わず、entry型が必要ならこの一回の結果を根拠に別の限定設計へ戻す |
| 欠測・上限・非干渉 | CAP R2の固定scalar取得を局所statementで再利用し、各fieldのnull/未存在/getter失敗をmissingへ`field:Null/Missing/GetterFailed`として区別する。各stringは8192 UTF-16以内、超過はprefixとtruncated field名を保持する。failureObservationのUTF-8 JSONは64KiB以内とし、超過時はposition→scriptStackの順でnull化してmissingへByteLimitを追記、なお超過なら固定fieldをnullにした短いcaptureFailureだけへ退避して欠測を明示する。missing/truncatedは固定field名だけ、各16件以内。採取自体の失敗はcaptureFailureの`{type,hResult,fullyQualifiedErrorId}`各string256以内へ記録し、元outerFailure/report.error/classification/accepted/requestedExitを置換・再throwしない。任意object/全環境/Exception全文dump0 |
| 保存と時計 | 追加観測は元unit-result.jsonへの既存一回CreateNew保存に同梱し、新sidecar/再保存/別名救済0。既存SaveFailure catchはそのままoriginal reportを保持する。診断を保存失敗で置換しないため、runnerは元reportを内包する範囲と欠測を分ける。旧一件5秒/集合30秒・境界一致拒否を維持し、採取/JSON化/保存を含む時間もrunnerの外側時計で照合する。保存余裕がなければ欠測/超過で停止し、期限後に追加I/Oで埋めない。自動hard停止保証は既存CAP責任契約のとおり追加しない |
| before／新binding | BD-Tは旧実行時Test4DF、receipt/binding、当時patchとRUN1原request/toolreturn/report/unit-resultを既存18pin/保存hashへ照合して別beforeへ全bytes保全し、現patchが異なる場合は旧実行版と現prefixを別物として保持する。新Test/path/hashと新artifact/output UUIDだけを新receipt/bindingへ接合する。outer catch追加を逆適用してbaseline全bytes/BOM一致、元entry参照等の非対象span不変を独立Preで証明する。C16是正/20期待/33期待/sourceF91C/PSSA規則変更0 |
| 一回受入 | 独立Pre/親freeze後、新artifactのOldRed経路を純合成一件一回だけ実施し、前後pin・実PS/argv/exit/時計・保存hashを独立Postへ渡す。rawの元分類/accepted/exitと診断採取成否を別欄で報告する。期待のException故障が観測されなければNotReproduced、固定fieldが取れなければPartial/Unavailableとして保存し、繰返し0。fieldが取れても失敗entry型の同定・原因一意化・特定RED受入・元PSSA NRE解消・C実装を成立させない。unexpected/drift/超過/採取・保存失敗で停止・後続NotRun/retry0 |

BD-Dの変更は本節だけ、コード/対象実行/新PSSA/OS/設定/再委任は0である。before設計はT/pssa-captureの新before-design-bd配下へ保全し、追補前SHA `87AD4D3037FFA1A310166E5DF94DA0397C4A02F3FBAF0D3A885B0BBE27F28AC5`一致を確認した。次はこの許可span/観測field/欠測判定を独立Preへ渡し、作者が新artifactを固定する。

### CAP-BF：entryのCLR型に基づく参照照合だけの是正

BF-Dはplanner指定`gpt-6.1-sol/high`である。BDの元Unexpected/exit2と位置194:34/stack195、entry型未特定を保持し、静的な無型Exception仮定だけを是正する。親からBD独立Post PASS/前後84一致/診断Partialを受領後に本追補を保存した。before全bytesはT/pssa-capture/`before-design-bf-257da649-3f21-4dcc-b3f4-b656b23da85f/design-19BF.md`へ保全し、追補前SHA256 `19BF4554BF241D8F2573A73D14E5B077F8BBCFEAE09D4F66F505C08AFC885FCA`一致を確認した。

| 範囲／seam | 固定契約 |
| --- | --- |
| 新artifactの許可span | BD実行Test99305111版をbaselineとしてコピーし、完全SHAを既存BD freezeから固定する。Invoke-CapOldRed内の一意`$related = @($engineErrors ... Where-Object ...)` assignment一つだけを、同位置の局所scriptblock `$isRelatedEntry`定義と`$related = @(foreach ($entry in $engineErrors) { if (& $isRelatedEntry $entry $originalError) { $entry } })`へ置換する。foreachはnullを入力から消さず検査するためであり、collectionのfilter/skipではない。前後のengineErrors.Count/related.Count assertion、元diagnostics代入/try/producer出力→throw、特定assertion/prefix/exit分類・既存BD診断は全bytes不変。公開function/signature/new helper/module追加0 |
| 局所predicate | 引数entry/originalはobjectとして受け、originalがErrorRecordかつoriginal.ExceptionがExceptionであることを確認する。entryがErrorRecordなら同じrecord参照一致でtrue、それ以外は型保証されたentry.Exceptionをcandidateとする。entryがExceptionならentryそのものをcandidateとする。null又はこの2型以外は直ちに`AdapterFailure: unsupported engineErrors entry`で失敗する。candidateがoriginal.Exceptionと同一参照ならtrue、candidateがActionPreferenceStopExceptionならその直接ErrorRecordがoriginalと同一参照の場合だけtrue、それ以外false。比較はReferenceEqualsだけ、duck typing/文字列一致/InnerException再帰/元参照の包直し/重複排除0 |
| 6焦点／独立期待 | F1=同一original ErrorRecord→true、F2=同一original.Exception→true、F3=originalを直接ErrorRecordとして持つ実ActionPreferenceStopException→true、F4=同じ本文/FQIDだが別recordかつ別Exception参照→false、F5=null entry→明示AdapterFailure、F6=Exceptionという名のpropertyだけを持つ未知CLR型→明示AdapterFailure。各入力は元entry/Exception参照の不変も確認する。F4を成功一致へ変えず、F5/F6をempty/nonmatchとして救済しない |
| 原文接続 | 作者の焦点試験は新Testの局所predicate＋実foreach assignmentを一意AST抽出し、そのまま6入力へ通す。engineErrorsは`[object[]]::new(1)`のindex0へ各entryの元参照を設定したarrayとし、nullもforeachへ到達することを観測する。trueではrelated.Count=1かつrelated[0]が元entry同一参照、falseではCount=0、F5/F6ではassignmentが正常完了せずAdapterFailureとなることを照合する。手書きの別predicateをoracle代わりに実行0。source SHA/元span/offset単位/length/hash/置換文/個数をmanifestへ固定し、是正spanをbeforeへ戻した全bytes/BOM一致を必須とする |
| 手順・受入境界 | 作者は旧4DF/99305111版・BD原本/失敗/診断/旧patchを保存し、新artifactだけで期待Pre→是正/Parser/限定差分・復元一致→独立実行Pre/親freezeへ進む。6焦点各一回が全PASS後だけ旧REDを新枠で一回、各入力5秒/共通合成集合30秒・firstUnexpected後NotRun/retry0。旧REDは同じ実producer/元代入/特定assertionだけがexit1/accepted=trueを成立させ、unknown/null/adapterは非REDのexit2、予想外成功も停止する。型契約6件と旧RED1件は別計上し、BD消費slotを再利用0 |

この変更は実entry型を同定した証拠でも原NullReferenceの原因解消でもない。sourceF91C/C16/20/33/全PSSA規則・severity/権限は不変、C未作成のままである。実装開始はBD Postと本設計Pre後、本工程は文書だけでコード/対象実行/再委任0である。

#### CAP-BF F3公開API fixtureの生成接合

F3だけは`$wrapper = [System.Management.Automation.ActionPreferenceStopException]::new('CAP-BF-F3')`、`$originalError = $wrapper.ErrorRecord`、`$originalException = $originalError.Exception`の順で公開APIからpairを生成し、entryへ同じwrapper参照を渡す。既存originalをctorへ渡す要件はなく、手製wrapper/private ctor/reflection invoke/setter・field書換え/元recordの包直し0である。[公開ctor](https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.actionpreferencestopexception.-ctor?view=powershellsdk-7.6.0)と[直接getter](https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.actionpreferencestopexception.errorrecord?view=powershellsdk-7.6.0)を用い、[一次source](https://github.com/PowerShell/PowerShell/blob/master/src/System.Management.Automation/utils/RuntimeException.cs)のbase getterはParentContainsErrorRecordExceptionを持つrecordをcacheする。ただし、この読取根拠を固定host PS7.6.7でのfixture成功とみなさない。

F3は実抽出spanへ渡す前にwrapperが実ActionPreferenceStopException、originalErrorが非null ErrorRecord、originalExceptionが非null Exceptionであり、`ReferenceEquals(wrapper,originalException)=false`かつ`ReferenceEquals(wrapper.ErrorRecord,originalError)=true`を必須前提とする。既存局所predicateでは前者によりSameException分岐を除外し、後者の直接record分岐でtrueとなることを元span＋固定入力から照合する。正常完了後もrelated.Count=1・related[0]がwrapper同一参照、wrapper.ErrorRecord/ originalError.Exceptionの再読が保存済み各参照と同一であることを確認する。前提不成立/getter失敗は明示FixtureFailureで停止・後続NotRunとし、別fixtureへ自動救済0・期待trueの変更0である。F1/F2/F4～F6・旧RED1・5秒/30秒・一回順/逆bytes契約は不変、本追補でfixture実行0。旧BB66全bytesはT/pssa-capture/`before-design-bf-f3-3792c0bb-b571-4f68-af5c-62874304749b/design-BB66.md`へ保全した（SHA256 `BB664B4F2E357A0E70434B9F0D6533E3C1C38757E433380D3D1BFED426FC6C78`）。
