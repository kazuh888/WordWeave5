# ET3 試用EXE提供・独立レビュー記録

## 観測Post・通常入口再実行と限定path読取設計（2026-10-15）

独立Preはdesign CE799395の322–367行と現物Invoke333C37FF／TestD8AD0AD4を照合しPASS、blockingなしと判定した。公式QueryFullProcessImageNameW／SafeHandle契約、nullと欠落の区別、先行世代照合、非null非上書き、最大8回、Aborting時期限、元OS例外、unknown非停止を確認した。PTH1–8の動的確認／同期API中断／80case成功は未承認・未証明のままである。plannerへ局所計画の確定を依頼した。

designer返却SHA `CE799395A14F1784C05EDDB0BC2D8EE8B53B900EC306B403DB72E225CF5E5756`、追補PTH1–8は同一保持SafeHandleの限定補完と先行世代照合、非null値の非上書き、有限buffer、既存期限、失敗時非PASSを規定した。文書diff-check／相対4link実在／Invoke・Test不変を確認し、独立Preへ引き継いだ。実API成功・OS同期応答時間・負例回収は未実測。既存通常releaseと別試用のEXE SHAも保護値と一致した。

Invoke333C37FF／TestD8AD0AD4／design51DBDBD7の観測追加は独立Post静的PASS。Data添付失敗の動的分岐は未取得であり、15回帰成功へ含めない。suite e48f86cd-4032-4598-a904-a4d0a3679be6、run2e9884b9-0768-41ed-a3a4-261b8ecc8889はcontrol2PASS、ENV01実呼出1FAIL、残79NotRun。RunOwner33040はCleanupIncomplete、unknownPID15748はsnapshot/rowFilter各1、known0、pathPresent=false、commandPresent=true、RuntimeException行360。親PID22496は登録Cargoと一致するが、世代・所有証明へ昇格しない。所有4件は終了wait済み、target起動0、再試行0、固定6SHA不変。旧unknownPID24592へこの原因を遡及しない。

plannerのread-only判定ではG5はOS所有証明を要求しCIM専用には限定しないため、既に保持した同一SafeHandleによる画像path補完は、設計追補・独立Preを条件に同等以上の証明手段として成立し得る。CIM path=nullの場合だけを対象とし、既存CIM値の上書き、PID再Open、自己申告、未知非PASSの緩和は認めない。designer gpt-6-astra/highへ既存設計1fileの限定追補を割当した。現時点では実装修正・新runtime・製品fullを解除していない。

## 入力状態是正後：terminal到達、別の所有同定失敗へ移行

限定観測の設計SHA51DBDBD7は独立Pre PASS。作者TestPS D8AD0AD44C2457DC60E64E52A6EDB455974D5F8AD9938B956CEF3E4DDAB0F7F3はschemaCandidate optional型一覧13行だけ（逆除去旧一致/80・87・validator不変）、回帰Test-OsObservation.ps1 SHA DF93D8FDC5E46C64EFC60D527F917655E9B2A78A0EBD43A3C2BD2A57209572C0のred-fixedは観測不足15RED/元unknown維持15。OBS9の旧getter故障刺激は伝播しないためproducer失敗証明から除外し、testonly producer境界throwへ訂正した。

Invokeの観測追加はSHA `333C37FF523542AD52004B76D17CE43369896555CEA2F326AE749A3C55D42245`、純追加58行（12field型/null/producer失敗の明示のため30行級見積りから親許可）、小private1＋局所Data＋最初catchだけ。green-333c37ffは15PASS/unknown維持15、parser/PSSA/error0、同定body/所有tree/対象外は逆抽出一致。実Gate/terminal出力とData添付失敗の動的分岐は未検証、独立reviewerへ安全・欠測・元例外保持の限定確認を依頼した。旧PID24592の正体は未解決のままである。

観測設計中の現物訂正：Get-Identity345/353は別のOS再照会をせず、Read-OwnedTree入口1117の同じRowsからProcess.Id一致行を抽出する。したがって元snapshotと呼出内の照合結果を『fresh OS query』と説明しない。既存Rowsのpid/parent/creation/field有無/一致件数/例外型行だけを許可listで残す設計とし、新OS照会や所有判定の緩和は加えない。

元-File通常入口の新suite `85cf0746-bd8a-4670-b321-d76d860ef0da`はcontrol2PASS/ENV01実呼出1FAIL/残79NotRun。run `99b70df0-dc08-4344-93a7-b1b75522e851`のRunOwner24188 terminalは初めて保存され、CleanupIncomplete/fixtureResiduals=[]/sourceMatched=true、所有4rowはwaited/exitedtrue、同PID24592のUnprovable2rowがOS process identity is unavailableを示した。入力変数の開始障害/terminal欠落とは区別する。

実装担当へPID24592の役割・保持handle・登録ACK・開始終了順のread-only照合を依頼した。既知signaled直系handleと終了後CIM欠測の取り違えか、本当に所有未証明かを根拠で区別し、未知非停止/非PASSや保持handle規範を緩めない。追加runtime/編集はまだ解除しない。LIFE14/全体/提供の成功へ換算しない。

## ENV01専用診断：最初の例外の実測位置

実装InvokePS SHA `0E28AAA12343AB331FD4D26837324B403ECBFC0C0980092237B4A5AC721035C2`を返却、5行逆置換は旧E1F5C520一致。固定回帰は10PASS/0FAIL（R/verification/invocation-state-contract-20261015-a/green-0e28aaa1）、parser/PSSA/error0。独立reviewerは内部4読取/外param・属性・argv・入口/日時・所有・期限不変と証拠SHAを確認し限定PASSを返した。親は診断probeではなく元-File通常入口を新UUIDで1回だけrunnerへ解除し、実動作/cleanup/80の未実証を動的に確認する。

独立回帰Test-InvocationState.ps1（R/verification/invocation-state-contract-20261015-a、SHA `589D6B844810699CF1A3D293E9412BC7504384D9FFD8C73D950D04481C3B338E`）は実ParamBlock/先頭代入/読取ASTを使い、旧E1F5C520で7PASS/3REDを取得した。Control2modeのValidationMetadataExceptionと内部状態未分離がRED、Child正常2/明示空null拒否4/外部属性・入口不変1はPASS。132行、parser/PSSA/error0、子process/Gate0。実装者へ5行だけの状態名分離と同一10回帰のGREEN確認を解除した。実-File ENV01は独立review後のrunner待ちである。

実装担当の現物ParamBlock純粋対照で原因を確定した。Control省略でもInvocationPath変数にValidateNotNullOrEmpty等の属性が残り、同変数へ空を再代入するとRunOwner/SupervisorともValidationMetadataException、内部別変数への代入なら成功する。Child正常値保持/明示空拒否も確認し、このscript-scope属性検証は-File形態にも適用される。最小fixは750の代入と1793/1839/1861/1925の内部読取をInvocationPayloadPathへ分離する5行で、外部param39/属性/argv/入口2215は不変である。

作者へ現物ParamBlock・先頭代入・内部読取ASTによる独立小回帰を先行割当した。Control2mode省略成功/Child2mode正常値保持/明示空null拒否/新状態読取を要求する。実Gate再実行は未解除、日時是正と今回の最初のthrowの因果を区別したまま進める。

専用診断一回はexit1、run `461eee50-80a7-4408-803a-5da7de828d34`、追加probeはValidationMetadataException/元Invoke750を回収した。外側は既存Test758のRuntimeException（判定失敗）、terminal/streams0である。元Invoke750はInitialize-Runの`$script:InvocationPath = $InvocationPathValue`で、RejectionRoot確定/JSON日時読取より前にある。入力paramのvalidation属性を持つ同名script変数へControl時の空値を再代入して失敗する仮説を、実装担当へ原形属性と純粋対照で確認依頼した。source編集/Gate再実行はまだ解除しない。

因果を訂正する：JSON日時の暗黙変換は実関数の独立REDで確定した別不具合であるが、実ENV01の最初のthrowが日時検査だったとは言えない。今回実測の750はそれ以前であり、前述の純粋式から実行順を推測した部分と区別する。probeの呼出形態が違うため、元-Fileの同属性代入でも再現することを確認し、診断形態だけの拒否を元原因へ混同しない。

## 日時是正後の独立再runtimeもENV01停止

診断3helperの限定Preは最終entry証拠不足一件を指摘した。親が現物0DC17ECCを直接parser/PSSA検査（各0）・ValidateOnly実行exit0し、tool `ac5b7c`の31関数/引数保持/他role・二重起動拒否/fixtureCreated=false/processStarted=false/acceptanceCredit0をSHAへ紐付けた。reviewerが未確認点closed/Pre PASSを返し、親はrunnerへENV01専用診断一回だけを解除した。原本source無変更、診断の80case受入加算0、commandline由来拒否と元障害を区別する。

診断entry Invoke-Env01Diagnostic.ps1は同専用root内188行、SHA `0DC17ECCE2DCE63E818D9B450C43B6935384141E8790D443BEFE941BEFDD2A33`で返却。ValidateOnlyで原本結合31関数・起動引数写像・他role/二度目起動拒否・非引数入力保持を合成確認し、fixtureCreated/processStarted=false/acceptanceCredit0である（結果は作者tool stdout、別証拠file未作成）。parser/PSSA/error0、固定2helper/原本/80行不変。既存fake.exe SHA `CFACB5A55109B5F7DA3327EC5B28EE37473C9C65C6F3DCF1DBA49FCDBC252435`を再使用する。独立reviewerへ3helperの固定境界/例外回収/exit/streams/認可と所有/期限/cleanup不変を限定Pre依頼した。実mode未解除、OS rawstream/commandline同定両立は未検証である。

診断helperの小対照は2PASS。固定R/verification/env01-error-probe-20261015-a/内のInvoke-ErrorProbe.ps1 SHA `CB7DCAE09936E434FDB4E38B1400AD034A00B8DCB26D4B2C04158AE6D8CECFDD`／Test-ErrorProbe.ps1 SHA `E2546A91D5E5E771182F0FF38C779F8175337460E72951BE4522F3225E619AEE`を返却した。selftest-finalでcaught型/元行、exit7/101、同pwshのstream records、CreateNew非上書きを確認、static-finalで合成fixture含む3fileのparser/PSSA診断0。別process/実Gate/ENV01開始0、OS rawstreamの実証ではない。正常ENV01への最小AST接合entryは作成前の必要範囲確認中である。

限定診断案はsource無変更の外script scope呼出を採る。Invokeは$Errorを消去しておらず、外側から`&`で固定Invoke/RunOwnerを呼んだ後、新規例外の型/ScriptLineNumberだけを固定R配下の既検証新診断dirへCreateNew保存できる。dot-sourceせず、LASTEXITCODE/streamsを保持する。まず合成例外で捕捉/exit/非上書きの小対照を作り、独立確認後に新正常ENV01一件だけを診断する。呼出形態が変わる診断なので80case合格には換算しない。作者所有は専用verification helperだけで、原本Invoke/Test/native/fakeの編集を解除しない。

日時差分限定review PASS後の新suite `4bfe2cb0-11d8-4cc5-a9ce-bd267926c77e`／run `b3237e4b-6338-48ca-bdb6-1f08ad66ebd6`はcontrol2PASS、ENV01実呼出1FAIL/残79NotRun、LIFE14未到達となった。terminal/target marker/streamsは0、Admin=false/固定6SHA一致/終了後CIM所有候補0、gatecleanup証拠は欠落、temp/local保持、自動retry0。日時是正の9+2系統PASSと、正常開始/terminalの別の未解決を区別する。

親は全suiteの再反復を解除せず、実装担当へ最初の例外とterminal例外の型/行番号を安全に取得する最小診断案をread-onlyで依頼した。source無変更のcaller-scope診断が可能なら優先し、source追補なら固定既検証R境界/専用CreateNew証拠だけ、raw内容/env/資格情報を記録せず、targetstream/認可/所有/期限を変えない案とする。

## ENV01・保存入力と純粋式による原因一件確定

TestPSの23 option追加はSHA `0BB2510E928C9C3BFAD34D8190F007C97881047B195194A586A319837FE484DE`で返却。逆置換SHAは旧BA73802F一致、独立2系統旧2RED→新2GREEN（approval7桁保持/明示日時計算、identity同時刻一致/1tick差拒否/残存3件正検出）。専用回帰SHA `C781FFB9C51D3178B1E6B218A5E72ED004896D409EC1EE381C62B8269AE9F7B5`、証拠はR/verification/testps-datetime-20261015。parser/analyzer実行error/新診断0、既存ShouldProcess6保持。実process/Gate/Cargo0、独立reviewerへ日時option差分だけを確認依頼した。

InvokePSは188への-DateKind String一箇所追加だけでSHA `E1F5C520199DA8362A486CB164DDC6389B093E71CB612215769C9C470D6E8062`を返却。同option逆置換bytesは旧00436AD9と一致、固定9回帰は9PASS/0FAIL、parser/PSSA診断・実行error0、実Gate/process/Cargo0。TestPSは日時を含む証拠読取23箇所（approval/identity/Ready/receipt等）のoption追加だけを別所有へ解除した。非日時14箇所と既指定5箇所は保持、明示datetime計算/80oracle/87呼出/期待値は変えない。独立小対照2系統を先に固定し旧RED→新GREENと逆置換一致を確認する。

独立作者の実Read-Json/Get-Utc回帰9件を固定した（R/verification/json-datetime-contract-20261015/Test-JsonDateContract.ps1 SHA `BA60AF8BCBB2A5DBFF9FA64B7F11773A20A8DDDC8DF237443057E6F1D66A6891`）。旧sourceは正常日時保持/65秒期間の2RED、異常日時・root/nested重複key・trailing comma・UTF8不正・immutable変化の7PASS、parser/PSSA/error0。実装者へInvoke読取一行と同一9件のfocused再検証を解除した。

作者はTestPSにも、読取identity日時とOS文字列の比較、approval再発行時の末尾ゼロ消失（7桁→6桁）の同種リスクを確認した。別所有でraw日時契約に該当する読取箇所と最小対照の限定案だけを先行確認する。実processを起動した結果ではなく、terminal欠落の全因果は引き続き未確定である。

Invoke188のConvertFrom-Json既定日時変換により、contextのstartedUtc/deadlineUtcがSystem.DateTimeとなり、632→Get-Utc109のstring必須検査で拒否される。固定pwshの純粋対照では-DateKind Stringにより7桁Z文字列を保持し既存検査が成功、期間65秒/offset・3桁negative拒否を確認した。UUID/path/8key/RunSHA/launcher-descriptor一致/Control parameter-setは成功。最小修正は日時検査の型許容緩和ではなく、読取時の表現保持一箇所である。

RejectionRootIdは当該throw前に確定するため、これだけでterminal欠落の全因果を断定しない。terminal22field組立/直列化/pathの純粋検査は成功、当時のwriter二次失敗は例外無記録により未確定である。実source/実Gate/process変更0。作者へ実Read-Json/Get-Utcの独立小回帰（正常文字列保持/異常拒否/duplicate厳格性）を先行依頼した。旧launcher終了後の現在不一致は原因へ読み替えない。

## 新固定snapshot・通常入口再runtime：ENV01停止

suite `aa5b5366-33bd-477c-af8e-cc4aad2ded11`、run `a495d5a7-0438-4ace-88f7-a5af12a8c572`。control2 PASS、実呼出1のENV01がterminal欠落でFAIL、80case PASS0/FAIL1/NotRun79/SKIP0。LIFE14未到達。Admin=false、固定6入力SHA一致、target marker0、streams0byte、終了後CIM所有候補0。gate自身のcleanup証拠は欠落、temp/local保持、自動retry/ソース編集/実API0。raw26偽PASS問題は再発せず呼出数判定が機能したが、正常初期化の失敗原因は未確定である。

実装担当へsource/保存入力の限定read-only切分けと、型/path/parameter-set等の純粋式診断だけを解除した。終了済み旧launcherとの現在の不一致を実行時原因と誤認しない。既存例外がFailureへ保存されるだけでterminal自体が書けないため、実行時の最初のthrowの復元には不足がある。新RunOwnerや製品fullは未解除である。

## LIFE14再開・既存要求の限定解釈

Invoke00436AD9/TestBA73802F/design0B7802CCを固定し、独立reviewerはLIFE14接合・拒否context・dispatchの再runtime前限定静的PASSとした。動的OS/cleanup/Postを代用しない。親は既定ET3-G全体runner gpt-6.1-sol/mediumへ新UUIDの通常入口80/87呼出しを1回だけ解除した。LIFE14だけのfocused割当gpt-6-sol/mediumとは工程を区別し、今回はfull80で正常/影響対照も含めて接合確認する。旧raw26偽PASSは再利用しない。

実装返却InvokePS SHA `00436AD987A4BD1D2FDF70BDDA683D60DA8983CC9AEDCB4BE9F8BC57304C3402`：3箇所15行追加/2削除、読取前hook閉範囲と実比較分岐診断だけを追加、parser/PSSA診断・実行error0、他入力不変/runtime0。独立reviewerへLIFE14接合と前checkpointの拒否context/dispatch/sidecarの未確認実装境界を限定静的確認として返した。PASS前に再runtimeを解除しない。

作者TestPS SHA `BA73802F7082F6266E419CD0EABC2ACC37961B9AF7C99BBF9A9879D06D2F2605`を返却。旧Confirm-CaseResult実関数の目的未到達/診断なし/他role診断は3RED、新oracle8PASS、合成同期対照はidentity限定変異・Registration保全・release結合を確認した。controller破損刺激を撤去、正常bootstrap不変とRunOwner実actorに拒否診断を束縛した。80行SHA不変、87呼出し/dispatch/P1/native/fake/V2も不変、parser0/analyzer実行error0/新診断0/既存ShouldProcess6保持。これは合成関数対照であり実hook/実cleanup成功ではない。実装担当の限定hook/診断接合を解除した。

designer追補SHA `0B7802CC6347EB177E074E677E7FC028C6283ABCA7C98C6ABB42B9C7B0554A69`（末尾280–307）は独立局所Pre PASS。RunOwner/現在runId/固定Supervisor Ready・初回読取前hook、正常bootstrap保持、Readyidentityだけの変異、同actorの実比較診断を定義した。immutable既読hashを削除しない、偽identityをOwnedへ登録しない、既存期限/回収を維持する。実装・動的実証はこのPASSに含まない。作者のTestPS限定是正へ進めた。

plannerと外部仕様担当（各Astra/high、read-only）は、caseRow110のRegistration/Ready境界と現刺激のcontroller破損の取り違えを確認した。真controller/contextを保持し、検証前Ready identityだけを実sentinel PID＋誤StartTimeへ変異させる。1271のRegistration不一致拒否への実測到達、Rejected/nonzero/zeroAll/既知cleanup/sentinel実identity生存が既存LIFE14受入である。1272 Assert-IdentityのOS世代比較そのものへ到達した証拠ではなく、既存OS再照合の約束を検証済みにしない。designerへ既存有限同期の最小追補だけを割り当てた。80行や未知非停止/非PASSの基準を緩和しない。

## 限定是正返却・未完了checkpoint（2026-10-15）

InvokePS SHA `3421604615881BAE6451A988245049CD67E5476551FDCE0700FC825F5BFB9037`、TestPS SHA `EEAF6957CBBCB7B76A14ADEE5CBA52D46D547AE44A67A995DBCA189DAC760DD6`、design SHA `0EC0336FA63EEC9DDB81ADD19A10444511A5EA72EC1B4184344C8312DC9F869D`で各所有返却。Invoke parser/PSSA診断・実行error0。Test parser/analyzer実行error0/新診断0/既存ShouldProcess6保持、sidecar準備の合成2対照PASS（正常/duplicate Run実bytesSHA・8field・非上書き）。80行oracle/87呼出し制御/P1/fake/nativeは不変である。

**LIFE14未解決**：既存刺激はdescriptor.controllerIdentityをsentinel＋誤StartTimeへ変える。真driverを示す独立launcherとの照合が不成立となり、実装はsafe runDirにCleanupIncomplete/配列nullを保存するが、既存oracleはRejected＋既知cleanupを要求する。sentinelをlauncherと認定したり、監査境界を緩めたり、空配列を捏造したりして解消しない。局所契約是正→接合後独立レビュー→新固定snapshot合成80→実Cargo/製品fullの順が残る。今回修正後のgate runtimeは0であり、旧局所設計PASSを実装Postへ読み替えない。

shared週枠95%を受け、親は保存checkpointを作り次工程の範囲回答を求める。DPI移管以外のnative残部移管は未承認のままであり、試用EXEは未提供である。

## 初回合成runtime停止：driver偽PASSとbootstrap証拠欠落

dispatch限定是正の返却：TestPS `C9BA3EF8…`、80行oracle不変。通常1回／variant契約数と順序、呼出ごとの一致する結果1件、ケースと最終集計の予定数・呼出数・結果数を保証した。独立合成再現は旧4RED/1PASS→修正後5PASS（正常87呼出し、空結果、複数結果、途中throw、空結果偽装）。parser0/analyzer実行error0/新診断0、既存ShouldProcess6は保持。gate全体の再実行ではない。次phaseのsidecar準備接合前snapshotであり、最終hashは後続返却を正とする。

bootstrap局所設計は独立PASS：新しい拒否専用contextはRunOwner限定、sidecarは実行cfg/認可に流用しない。固定UUID path・実bytesSHA・保持launcher同一性・fixture/TEMP境界・UTC/Stopwatchと既定上限/回収10秒予約を独立検証する。実測0のみ空配列、不能はCleanupIncomplete/null、context自身不正は書込なしunknown。childは自己scope前unknown、確定後だけ既存出力。関連3cfgのsynthetic TEMP/TMPは親指定の`target/early-trial-build/et3/full2-env/related/temp`、localは隣接local、targetは既存et3/related、逐次実行のみである。default/trial固定環境は変更しない。実装と作者準備の接合・独立確認は未了である。

固定snapshotの通常入口はexit1。control2 PASS後、resultのENV01–GATE20は26Passedだがvariants=[]で対応fixtureなしであった。TestPS1995のif出力`@($null)`が消失してforeach0回となり、failure=nullだけで成功化した。26件を行動PASSから撤回し、未実行として扱う。最初の実動作GATE21（sourceInput欠落）はterminal証拠なしでFAIL、残53はNotRun。ログ正本はignored `target/early-trial-build/et3/cargo-gate/verification/8b960cec-97f2-43fb-9851-50b4044d17be`。

InvokePS613–627で完全field検証が拒否証拠context設定より先にthrowし、finally2036の条件が偽となることをread-only切分けで確認した。作者はTestPSの実行数保証／空実行非成功だけを限定是正する。bootstrapは安全なpath/UUID/reparseと拒否専用contextの局所設計を先に確定する。既存80刺激・期待値・terminal必須・未知非PASS・起動0を緩和しない。製品fullと提供は未解除である。

## P1限定是正・独立再静的closure PASS（2026-10-15）

InvokePS `F39A95D9…`／TestPS `763CB0B1…`を固定し、独立reviewer Astra/highが旧P1二件のclosureをPASSと判定した。CIM世代照合、属性取得前のSafeHandle固定、同handle保持・有限停止・Dispose、未知identity非停止を確認した。作者のdriver共通関数回帰は修正前2RED→修正後2PASS、mockはIt内限定である。通常入口の動的mock除去、実PID再利用race、合成80case、wrapper全体／製品fullはこのPASSに含まない。TestPSのparser/analyzer実行error0、新規診断0、既存ShouldProcess診断6件を保持した。

親は通常入口のcontrol2＋合成80caseだけをrunner Sol/mediumへ解除した。初回失敗停止・NotRun非成功・所有processだけの有限回収を維持し、製品fullは未解除である。native/fake/設計の固定SHAは変更していない。

P1限定修正の実装返却：Invoke PS `F39A95D99618EBD35576C64F8144C710110EF0D77FA91BE66AB28B9F91A1A892`、parser/PSSA診断・実行error0、native不変/runtime0。取得直後SafeHandle固定、CIMとhandleのUTC6桁観測精度照合、不一致/欠測Unprovable、Knownの同handle保持/kill前確認へ限定した。作者はTestPS同種修正と独立2回帰を作成中であり、独立再確認/P1 Closedではない。親の通常権限・sandbox外・自processだけの読取診断はAdministrator=false/CIMDateKind=Local/handle valid/UTC6桁一致true/TargetStarted=falseであった。CIM能力/時区精度の確認だけで、全process世代raceや80caseの成功を代用しない。

## 初回runtime前・独立静的最終：CHANGES_REQUIRED（P1二件）

fresh reviewer Astra/highは固定4source全文・V2・自己oracle結果を確認し、初回80caseの受入阻止2件を確定した。第一は下記CIM行/実handle世代混在（Invoke PS 339/925–935/1898–1902）、Test PS 303/954–965の子孫収集にも横展開が必要である。CIM CreationDateと同保持handleの開始時刻を観測精度を明示して照合し、不一致/取得不能をUnprovableとして非停止にする。

第二は列挙GetProcessByIdのProcessオブジェクトにOS SafeHandle固定がなく、時刻確認後のKillで別世代を開き得る点（Invoke PS 925–935/1894–1902、Test PS 764/766–776）。reviewerは.NET10.0.12の自processだけの読取で、StartTime/Path/HasExited/Refresh後も_haveProcessHandle=false、SafeHandle取得後のみtrueを観測した。列挙対象取得直後・開始時刻等を読む前にhandleを固定し、同handleで同世代確認→停止→有限Waitする。固定不能は未知/非停止。cached StartTimeだけや『handleがPID再利用を防ぐ』未検証仮定では代替しない。

静的根拠は[Win32_Process](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-process)と[.NET Process実装](https://raw.githubusercontent.com/dotnet/runtime/v10.0.0/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Windows.cs)。これ以外のfixture範囲/reparse/cleanup/製品cfghook拒否/unknown非停止/有限回収/MoveFileW/API・資格情報・mic非利用の確定blockingは検出0。悪意ある並行writer防御は対象外。reviewerの対象runtime/compile/編集0、全6入力SHAは終了時一致、usage counter等欠測である。親は作者/実装へ各所有PSだけの限定修正を返し、初回runtime未解除を維持する。

独立初回runtime前静的確認の中間指摘：`et3_gate_static_review` Astra/highはP1（Invoke-CargoGate.ps1 925–935／342–351）を確認した。CIM snapshotの子Aが終了し同PIDのBへ再利用された後、GetProcessByIdのB.StartTimeとPID-onlyで対応するAのparent/path/commandを結合してOwnedへ入る。cleanupのStartTime比較だけではこの混成identityを防げない。直系Start保持handleとは別の取得前raceである。親はruntimeを未解除のまま、固定入力の全境界レビュー完了までwriterを停止させた。最終指摘受領後に所有PSだけを限定是正する。中間報告を最終Postや製品不具合へ換算しない。

## 2026-10-15 初回runtime前：4file固定、独立静的確認中

作者/実装の各2file所有返却を親が受領し、5現物SHAを照合した。R=ignored `target/early-trial-build/et3/cargo-gate`。wrapper/実Cargo/runtime80caseは0であり、製品full/提供は未解除。最終設計SHA `2DE1F344FEC43468AA1F40716A7A912BDCD943318B5D39FDC5F47CBEEC970937`の回収予約とMoveFileW非上書き公開も独立局所PASSを受領した。

| 固定現物 | SHA256 |
| --- | --- |
| Invoke-CargoGate.ps1 | `E4FE39822CA1C75CA2EC34530BCE18DBC6CCEF4559547C901A730951C52F8BB9` |
| cargo_gate_entry.rs | `3AEB0B19E2D03EE088EB1BEDA4DBACEEC0406E12ED75D59AB95A7E331F42FD41` |
| cargo-gate-entry.exe | `1C04288220C3D98B182F3011FE3D1DE8AE88B050871DF400B21ED8AECCCC5E94` |
| Test-CargoGate.ps1 | `E492FB11C443D08857AE5CBF93AA25165E59F2444ABA02A97F75D61F139612F0` |
| fake.rs | `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016` |

作者の現snapshot自己検証は9/9PASS、80行oracle SHA `CEF0406933398F433D2726F7EC4F1C4292A0CDF555B39AEA725ABB40920FF905`不変。通常Run65/短期30/ORDER01 35/real85秒は回収10秒を含め事前固定し、動作中延長なし。作者PSはparser0/PSSA error0・ShouldProcess警告6を根拠付き保持、実装PSはparser/PSSA0、native rustc1.98.1/edition2021 compile exit0。既知残存は保持handle StartTimeとCIM同PID metadata間のCreationDate未照合であり、親は未知を無条件承認せずfresh独立reviewer Astra/highへ初回runtime前の所有/境界/停止/cleanup確認を依頼した。新caseの自動retryはなく、ORDER09だけ固定刺激として取消Run拒否後に別Runを一度実行する。

受領証拠/短命processの局所追補も独立PASS：Supervisorの期限内full受領をimmutable `full-received.json`へ束縛し、後続Runnerは同receipt/full bytes/identityと既存expiryを照合する。G0期限の自動延長/全fullへの再適用はしない。最終設計SHA `BC763229524B5AF64E236BD773341E63AB94A7074E62D564272FEE9D11049068`のDirectLaunchHandleは、同ownerのatomicIntent/directStart/返却保持handleに限り直系子の所有終了を証明する。未観測OS値はnullでIntent値と別、未知子孫/引継ぎ/子孫監査/negativefaultの迂回には使わない。同reviewer Astra/highは両追補に確定矛盾なしと判定した。設計のみで動的成功ではない。

実装者の初回静的確認はPS parser error0/native compile成功、runtime0。PSは複数監督/fixture hookを含む約1700行となり当初の小さなscript像より大きいが、製品source/依存/OS/通常Cargoを変更せず、固定80case/独立実行/Postの基準も維持する。作者/実装が最終設計へ接合・静的確認中、両者返却前の混在runtimeは引き続き禁止する。CIM sandbox拒否は通常ユーザーのsandbox外能力確認と別途扱い、管理者実行やDirectLaunchHandleによる全列挙代替を許可していない。

局所接合追補も独立PASS：最終設計SHA `91650B87DD9BDC8752658DCEA2646A9FED1B02BD8655E4F6A28E3A0C116523F9`の役別scope束縛/参照欠落時有限停止、V2出力の固定例外、payload読取前の正常OSchain hookと目的拒否段階の識別を同reviewerが確認し、確定矛盾なしと判定した。GATE18はRequest非上書き対照でnativeUUID衝突実証とは別である。編集/build/runtime/再委譲0、動的成功の判定ではない。親は作者/実装へ追補の接合を解除した。native残部移管質問は10分以上無応答のため承認とせず、利用者既定どおり依存しないwrapper作業だけを継続する。

並行接合でscopeId事前指定、fixtureのcase出力境界、正常OS親子での不正payload/重複ID対照の局所不足が判明した。designerを既存設計だけの短い具体化へ戻し、この接合点を解除する間だけ作者/実装/設計の3補助とする。親は指定model/effortを変えず、各所有fileは非重複。未知のscope wildcardやfixture例外を実装者が独断導入せず、該当部のみ保留し普通のdriver/実装作成は継続する。混在runtimeは解除しない。

## ET3-G 局所追加レビュー：PASS、作者/実装並行

同独立reviewer Astra/highは新設計SHA `6DBF6D8C6961F4E89BEEFF8ECD33B73A892B47317076918507A1FE54B973C04A`と作者2fileの現物を照合し、局所追加に確定blockingなしと判定した。製品cfg5値/既定command一致、既存marker4/4の同算法再計算一致、G0固定12項目差0、fixture閉境界/hold後再照合/同期限/negative観測と独立OS oracleを確認した。元Pre-2と別の判定であり、行動試験0/80case/動的Post未了を保持する。

親は固定された80caseと具体interfaceに対し、独立作者Astra/highのTest-CargoGate.ps1/fake.rsと実装者Astra/highのInvoke-CargoGate.ps1/cargo_gate_entry.rsの並行を限定許可した。各2fileの所有が重ならず、期待値を実装者が変更できず、同じ確定schemaを用いるためである。混在snapshotのruntimeは解除せず、両者の所有返却・現物固定後にfocused/独立実行する。生成fakeは作者だけが扱い、製品source/共有helper/full/native/実API等は触れない。通常の補助1役既定をこの固定interface区間だけ2役へ拡張した。

native残部の扱いを利用者へ追加確認した：正常起動、学習/設定保存・再起動、音読画面の導線、標準keyboardを未確認と明記して試用時確認へ移すかを質問した。DPI150/200承認だけではこれらを免除しない。回答待ちの間は独立した一時wrapperの作業を継続できるが、提供解除はしない。実資格情報IOを避ける制約と現操作手段の限界を、製品の動作不良とは区別する。

## ET3-G 独立作者：契約固定、driver未実装

`et3_gate_tests` Astra/highがignored R配下のTest-CargoGate.ps1（SHA `424207EC8058DE0719384A5700611B0E4DEBEF39F973B44B18383712B9B46F5A`）とfake.rs（SHA `46FA412A04F6151AFB2C23B0259C72E2549C48686A70AAB7CB744568825A90DE`）を作成し所有返却した。ENV6/GATE26/RACE5/TRANSPARENT15/LIFETIME19/ORDER9の80caseを固定。通常入口は契約保存後BlockedPrerequisite/exit2であり、wrapper行動試験0である。

作者fakeの自己検証は7/7PASS・skip0、UTF-16/孤立surrogate、raw stream、32bit exit各値、FileShare.Read下の実行を確認した。Pester初回の自動Registry/CIM初期化拒否はfixture未実行で行動REDではない。テスト局所で未使用自動fixtureを無効化して再検証し、global/registry変更0。parser0、analyzer未使用注入parameter警告3件はPost判定へ保持する。

Run等の具体key/cfg/digest/sidecar名、承認read後/ACK境界、V2Case Start後登録前、unknown/cleanup不能の合成観測境界の4局所事項をdesignerへ返した。caseを弱化せず、確定runへfixture bypassを作らない。製品full/native解除なし。自己検証7を80caseや製品PASSへ換算せず、usage counter等欠測・共有週枠73%を保持する。

ET3書式検査：親が `cargo fmt --all -- --check`、音響解析/Qwen各manifestの `cargo fmt --manifest-path tools/<crate>/Cargo.toml --all -- --check` を実行し、3件ともexit0・診断出力0であった。書式の自動修正は実施していない。release固定source-afterの170項目を現物へ再照合し不一致0。全体test/native/提供Postは別の未了項目である。

## ET3-G 独立Pre-2：PASS

同reviewer Astra/highは計画 `06E880E7…1463C7F2`／設計 `CC72A435…62E8313`の現物一致を確認し、Pre-1の入口parse/G0回収所有のP1二件を設計上解消と判定した。新たな確定矛盾/受入阻止/要回答質問0。任意argvのbinder非通過、UTF-16保持、同ps1所有木と登録ACK、aggregate600秒と単一10秒回収、証明不能時CleanupIncomplete/full禁止が具体化された。

PASSは設計Preのみであり、独立作者→実装→runner→Postでargv/stream/exit、EXE lock、ACK競合/単独監督喪失/有限回収を実証する必要がある。全source G0再認証、native残件、製品full/提供解除ではない。編集/build/runtime/再委譲0、usage/provider返却metadata等は欠測である。

独立Pre-2へ返却：計画 `06E880E7BD1C387278DE20ECD5573F6F308C01A2417B1313E676305E1463C7F2`／設計 `CC72A435CAF840E94F06F5704D95822BDAF57A19C10A91E579CE3ACB062E8313`。native入口が任意argvをUTF-16整数JSONへ保持し、PSには固定parameterだけ渡す。G0はRunOwner→Supervisor→SafetyWorker所有木・登録ACK・単一期限の回収へ接続した。作者2file/実装2file、製品source/既存helper/合否は不変。親が現物SHA一致を確認し、同reviewer Astra/highへ返した。未Closed・実装/runtime未解除である。

## ET3 native RootUnavailable：DPI100%外観・focus確認、keyboard未確認

独立runner Sol/mediumの最終所有窓PNG `target/early-trial-build/et3/native-refusal/ru-final-20261014-225110/owned-refusal.png`（SHA `79A18A944717CE06AA63EE975DD0EE22CAEE42F3033A5EABC18DA9FDD119B40B`）を親も直接確認した。420×192、actual monitor100%/HRESULT0、日本語title/bodyの折返しと全文、error icon、OKの文字配置、focus点線を確認し、見える欠け/重なりなしである。Win32実測OK control ID=2・所有一致・focus一致であり、UIAのPane/HasFocusfalseとの違いは観測providerの制約として保持する。

先行2回はobserver側の未確認ControlType/IDOK前提で停止したものであり、製品不具合とは判定しない。標準keyboardは未確認、keymessage送信0である。最終補充後は再起動停止、3組の所有process残存0、source170差0、旧2EXEと試用E3のhash不変をrunnerが報告した。通常起動/保存再起動/音読導線等を含む全native gateのPASSではない。詳細は既存delivery-runner記録を正とする。

## ET3-G 独立Pre：CHANGES_REQUIRED

独立reviewer `et3_gate_review`（指定Astra/high）は計画SHA `6E368E648D919AC40C56B8606BAAB8ADFE8A45B2C0EDE971DBC9E792E5BB4A9E`、設計SHA `92E370C131AAF46201BD9D575ADEC1DC0A8E119450587D8DB33966043B07A09C`を照合し、確定した設計不足2件を報告した。実装/runtimeの欠陥を実測した判定ではない。

1. P1：必須のCmdletBinding/paramとCargo runnerの任意argv保持が両立する入口宣言・境界が未確定である。未知option、空文字、Common Parameter名が入口で拒否/消費されない具体契約が必要である。
2. P1：親が別起動するG0/V2Caseに、aggregate600秒・cancel・監督喪失からの有限回収が接続されていない。実processの所有・登録前起動禁止・開始許可・残り時間・kill/wait責任を固定する必要がある。

計画者へ方式判断を返した。追加の利用者承認は不要であるが、再Preが通るまでwrapper実装・製品full実行を解除しない。旧承認の再利用、対象縮小、待機期限の自動延長をしない。reviewerの編集/runtime/再委譲0、usage/provider返却metadataは欠測である。

## 2026-10-14 22:18 JST 再開：利用者承認を受領

利用者は(1)今回のコマンドだけの一時的な検証用Cargo runner追加、(2)rfdのWindows DPI150/200%確認だけを利用者の試用時確認へ移すことを明示承認した。通常設定/OS/製品source/対象test/合否は変更しない。以下の以前の「未回答」は当時の履歴となる。DPI100%・日本語/狭幅/focus/keyboard・直接起動/保存/再起動/既存音読導線・通常/試用fullと関連crate・最終独立Postは提供前に残す。提供時は「DPI確認未完了」を明記する。

現HEAD6fb6da3、tree cleanから再開し、親の完了境界は一時runnerの安全設計/対照検証→新EXE照合付き全体検証→既定native→最終提供Post→既存製品を置き換えない試用EXE提供である。planner Astra/highが限定手順と役割を確定する。承認だけで未知EXE実行や未検証wrapperを解除せず、再build後のsource/構成/実EXEを照合する。

### 独立したnative限定確認の解除

親はrelease source-afterの170項目を全現物照合し不一致0、試用EXE SHA E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2と旧通常EXE不変を確認した。store.rsのresolve→Storage::openとmainの失敗分岐を読取確認した。processのLOCALAPPDATAを相対値にした新processではBeforeWrites/RootUnavailableからrfdへ進み、WordApp/diagnostics/資格情報/録音へ到達しない。GetScaleFactorForMonitorの読取はHRESULT0/primary100%、OS変更/アプリ起動0である（[API資料](https://learn.microsoft.com/en-us/windows/win32/api/shellscalingapi/nf-shellscalingapi-getscalefactorformonitor)）。

この固定EXEでRootUnavailable拒否dialogだけのordinary-user/Hidden/有限期限/所有PID回収確認を解除する。観測/PNGは所有dialogだけに限定し、他app/デスクトップ全体の撮影・操作やユーザーprocess停止をしない。未知のtoken/EXE/source/画面では中断する。正常起動・音読・実API等はこの解除に含めない。wrapper設計とnative拒否確認はsource不変・別出力の独立作業であり、この区間だけ2補助を並行する。

## ET3-FX1 Pre：PASS（2026-10-14）

独立reviewer `trial_delivery_review`（指定 `gpt-6-astra/high`、read-only）は、all-targets compile blockerを是正する限定設計をPASSと判定した。確定欠陥/受入阻止/要回答質問/任意提案0。実装・compile・runtime・G0解除・提供の成功を意味しない。

入力HEAD `d446c5dd32177815ee322a39cf51ab84a99ee458`、計画SHA `4DACB561260B32D15570C07D7F202E82B0D6719149D1EDE194CB0057DA61100F`、設計SHA `B2FEFA5C9E1B10F509939F5235C2B45DBFA58F6455B83C58F19D6298F8422969`、契約SHA `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` を現物照合した。source-beforeの168項目/作者4markerは不変、新helper不存在、dirtyは文書のみ、diff検査成功。

確認したのは、ai/mediaのファイル直下private Windows/test限定support moduleによるsibling path解決、旧media再公開、stateless helperの同体抽出と明示imports、非Windows/非testでの除外である。既存7consumerと通常bin23/試用bin33/integration各11の予定集合、A08/A09/B/Dのguard・worker終了契約、import逆置換によるai oracle保持と他3marker一致要求を照合した。隔離のAdminFalse/1exact1child/合成環境/Hidden/45秒/killwait/子回収後cleanup/完了markerは変更しない。

修正は作者がsrc/ai.rs・src/media.rs・新src/early_trial_test_support.rsのtest/fixtureだけを担当する。独立runnerの両構成all-targets no-run、新helper込み9source＋integration/Cargoの固定、構成EXE別の親G0、集合不減と独立Postを必須とする。既定ET3の全体/関連crate/release/native/提供gateは維持する。source本文不一致や所有不足では停止する。

未検証は実差分の同体性、Rust compile、新EXE listと動的結果である。reviewerの編集/build/runtime/再委譲/公開0。週枠61→61%（共有）、終了20:30:43 JST・最終応答前、counter/session/provider返却値/親子包含は未取得。

## FX1親sourceG0・両構成no-run限定解除

作者は3sourceだけを修正し所有返却した。親がai/mediaの実diff全体と新helper全文を読み、module宣言/ai import/旧media再公開/同体抽出だけであることを確認した。通信GLOBAL・録音global・実入口遮断/加算/poison/Dropとworker・guard寿命は変更していない。新helperは明示importsと元doc/signature/bodyだけで、状態やtest caseを追加しない。

親がGitの修正前mediaから関数を再抽出し、設計どおり行頭4space/CRLF→LFだけを正規化して新関数全文と比較した。一致true、正規化SHA `2C5A41C49D78717C41A8118738F5F8ACFEF73FB1121D90C95C62F9E357A17B0F`。作者evidence SHA `D5357C2602B17962C6A94520B13B36D37EF80687E1155BE722AB38E37E18C563`、afterSnapshot全12項目（新helper込み9source＋integration＋Cargo2）も親現物hashに全一致した。作者証拠のai import逆置換marker一致/他3marker一致を照合、旧release SHA不変も再確認した。

| 変更source | 固定SHA256 |
| --- | --- |
| src/ai.rs | `94B85460696455D8D9D48593B0C299B5E849B10A45C2039E43F18D057E062A99` |
| src/media.rs | `A3EC60D795C63052E6E3EEAC45EE7903CAAD97752242C005DBFE801EE6DF2673` |
| src/early_trial_test_support.rs | `40A43200CE106B555E4688716179BA88C972D104AF401F059A86A15FF33FE34C` |

この固定組に対して独立runnerの通常/試用all-targets no-runだけを解除する。各構成bin/codex_processを含む全artifact JSONと前後source一致、profile/features/絶対EXE/SHAを返すまでlist/runtime禁止。旧失敗時部分artifact、旧ET2、bin→integrationの解除を流用しない。source/EXE変更・errorでは停止、作者以外のsource修正不可。関連crate/release/native/提供は後続のままとする。

## FX1独立静的確認：指摘0、Post保留

同reviewerは実差分を独立確認し、承認済み3file境界、helper同体SHA、ai markerのimport逆置換による旧hash一致、他3marker不変、修正後12項目と他166項目の現物一致を確認した。確定指摘/要回答質問0である。compile・新EXE list・動的結果は未確認であり、最終Post/Closedではない。編集/build/runtime/再委譲0。共有週枠62→62%、終了20:39:14 JST・最終応答前。指定Astra/high、返却metadata/token/session/親子包含は未取得である。

## ET3 native提供条件の限定確認

同reviewerの限定読取判断では、契約ET-03/設計/計画が要求する合成native導線とrfd起動拒否dialogの確認は提供前gateである。特にOS dialogのWindows DPI 100/150/200%をegui zoomや正常起動だけで代用しない。実credential/API/micを利用した検証を要求するものでもない。mock画面の直開きだけでは既存音読導線の到達証拠にならない。IME・実Codex/Qwen・mic・TTSは既存どおり利用者実機受入へ残せるが、native gateの暗黙免除にはならない。

親はDPI 150/200%の確認だけを利用者試用時確認へ移す承認を非同期で求めた。回答待ちであり、既定gateは未変更である。他の検証は継続する。限定判断の編集/実行0、共有週枠62→63%、終了20:44:17 JST、返却metadata等の欠測は継続する。

## FX1新EXE Gate：両構成のlist・限定回帰を解除

独立runnerの通常/試用all-targets no-runは各exit0/build-finished=true、各24artifact、compiler error0。親が固定12項目の現物hash差0と旧release不変を再確認し、次の4EXEをartifact JSONおよび現物hashへ照合した。binとintegrationを区別し、通常features=[]、試用features=[early-trial]、いずれもprofile.test=trueである。

| 構成/target | 絶対EXE（repo rootからの固定相対表記） | SHA256 |
| --- | --- | --- |
| 通常/bin | `target/early-trial-build/default/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `CDE160AEB81AE335379CE624E9EC942A28FDD21766823685F528DF5A6FE508F5` |
| 通常/codex_process | `target/early-trial-build/default/debug/deps/codex_process-b80f5fbdc29c53b8.exe` | `54075C6EEDA79877A75687EEE6BEDDF7BF5D819CE261A1C0C4A0D12C746B7E14` |
| 試用/bin | `target/early-trial-build/trial/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `3014F3091436122D86DDC3543D5F842D853AD5CC779B651A4D9A145EACCBD107` |
| 試用/codex_process | `target/early-trial-build/trial/debug/deps/codex_process-4e6a5997dab868a1.exe` | `C70CE963CAB94990CD48599366CFAA3F71B0C41E3F1EE7642DED190579B0F625` |

絶対基点は `F:/Kazuhiro/GitHub/WordWeave5/`。この4EXEだけlistとFX1限定回帰を解除する。録音安全3件を先行し、P01安全順序、helper consumer集合不減、integration自身のA群11件を確認する。各起動で非管理者確認・EXE hash・合成環境・Hidden・期限・回収・該当child完了markerを維持する。source/EXE不一致や未知の失敗で停止する。他artifact、全体実行、related/release/native/提供は未解除である。

## FX1最終Post：PASS / Closed

独立reviewerは両no-run raw JSON、4EXE現物、全88個別result/stdout/stderr、集計JSON/CSVを照合した。通常bin26/integration11、試用bin36/integration11の84/84が実名付きexact1件PASS、ignored0。helper78件の実名child marker/AdminFalse/exit0、非helper6件の外側結果、4listの集合不減、実行順、期限/Hidden/wait/fixture残存0、終了後対象process残存0を確認した。

build前後/runtime後170項目と現物差0、G0の12項目/4marker/Cargo設定/旧release不変、wrapper hash一致。既確認の同体抽出とoracle/global/worker寿命保持の静的判定、同snapshotのfmt/diff成功を接合した。FX-R1–3の限定是正をClosedとし、確定指摘/要回答質問0。ET3全体/related/release/native/提供は閉じない。reviewer編集/実行0、終了20:59:39 JST・最終応答前、共有週枠63→64%、返却metadata/token等は未取得である。

## 既定ET3全体gateへ復帰

親は通常/試用それぞれ全24artifactの現在SHAを保存JSONへ全照合した（各不一致0、test profile各22、features通常[]/試用[early-trial]）。固定source不変とFX1独立Postを受領し、この組で既定の通常/試用all-targets runtimeを解除する。各構成の既定Cargoコマンドを省略せず、合成環境・非管理者・有限timeout・終了/残存確認を維持する。full run中の非test artifactを起動する許可ではない。新source/EXE変化、未説明の失敗では停止する。

関連crateの既定no-runと通常/試用/単独qwen-audioの別target release buildも許可する（buildのみ）。関連test EXEの対応照合までruntime不可、releaseのnative起動/最終提供も未解除である。native条件の未回答承認を推定で埋めず、旧製品releaseを上書きしない。

### 通常full開始時の再compileによる解除失効

既定Cargo full commandは固定sourceのまま再compileを開始した。原因を環境差と断定せず、親は所有する検証Cargoだけの停止/回収を指示した。runnerは所有treeを親子/時刻/commandで照合し、21:08:03 JSTにkill/wait、対象残存0・ユーザーprocess停止0を確認した。wrapper exit1/Cargo exit-1は意図した停止であり、製品テストFAILとして数えない。raw test result/Running/Finished行の観測0、実行成功証拠へ流用しない。旧通常24artifactは23件が変更又は不存在となったため、旧通常EXEの実行解除は失効する。source170項目差0、旧製品/単独tool EXE不変である。

次は再compileの限定診断と、fullと同一process環境・同一合成rootでのno-run固定→親の新artifact照合である。現時点、再build/runtime/related/releaseはrunnerへ停止返却し、native/提供も未解除のままとする。FX1の既取得・独立確認済み84件は当時のsnapshot証拠として保持し、新EXEの成功へ読み替えない。

### 再compileの局所原因と許可待ち

runnerのfingerprint診断は複数依存crateについて `MissingFile { path: "C:\\Users\\安井" }` を明示した。対応Rustc dep-infoではプロフィール名のU+3000が無escape、Cargo binary dep-infoには切れた不存在prefix1件と正しいfull path11件が共存する。固定環境の通常no-runで再生成したdep-infoにも同entryが残り、既存試用側にも同じ問題を確認した（`full2-both-dep-info-prefix-audit.json`）。少なくとも同一環境だけでは次の再buildを防げないことが分かった。Cargo全般/全日本語pathへの一般化はしない。source、主要fingerprint、toolchain/configの不一致やGit hookは検出していない。

親はprocess限定Cargo target runnerによる「build後・実行前の新EXE照合」の一時ラッパー導入許可を利用者へ質問した。公式[Cargo runner仕様](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)で機構の存在は確認したが、適用設計・実装・承認は別であり未着手。source/依存/fingerprint/OS/global Cargo設定の書換えや、テスト/合否基準の緩和はしない。原因既知の試用no-runの無用な反復は行わず、進行中の通常no-runだけ安全に終了して返却する。

承認待ちの間も未影響の既定release build（試用→通常→別target単独qwen）だけは追加解除した。新EXE/構成/hash/source保全を返すまで起動不可、全体runtime/native/提供は依然未解除である。build成功を全体テスト成功の代用にしない。

親の代替案read-only確認では、既存`.cargo`/`.rustup`の8.3短縮pathにもユーザー名部分のU+3000が残った。短縮名へのprocess環境切替だけではこの空白を回避できないため採用していない。alias作成/移動/環境変更0である。

### 配布用buildの進捗（提供判定ではない）

試用releaseはexit0/build-finished=true、features=[early-trial]/opt3/debuginfo0/debug_assertions=false/overflow_checks=false/test=falseである。`full2-trial-release/artifacts-fixed.json` のEXE SHA `E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2` を親が現物へ照合した。旧通常製品SHAは保全値一致。source170/G0不変・非管理者/Hidden/有限期限/wait/TEMP残存0はrunner記録を参照する。試用EXEのnative起動・提供用copyは0、全体gate成功を意味しない。

通常releaseもrunnerからbuild成功が返った（EXE SHA `661156D57067404BE8A5C232CED6A19442BF5D3F6487EF4B905AF4848AE4F2C2`、features=[]）。単独qwenの別target buildを継続している。全EXEの最終親照合と提供Postは未了である。

### 今回の区切り：3build成功・承認待ち（ET3未完了）

単独qwenもexit0で完了した。親は通常/単独qwenのartifact JSONと現物SHAを照合し、単独SHA `DD93326B164661A74CDF498852DF19DF471C7A3AFBED65980D5AAA45A30526AA`、features=[default,gui,mcp,windows-credentials,windows-job]を確認した。試用と合わせ3release build成功であり、製品test成功件数へ数えない。親がG0全12項目差0、旧通常EXE保全SHA、旧単独 `target/qwen-audio/release/qwen-audio.exe` SHA `57A3BE5B10F117E146E1C8AAC77616AA4A0F0873D74D68D858BD8C6E9A64506E` 不変も再確認した。

runnerの最終source170/marker4/新通常debug24artifact差0、全所有process残存0/wait済み、実API/credential/mic/native/提供copy0は `full2-final-preservation.json` / `full2-final-process-audit.json` と報告末尾へ固定した。runnerの許可作業は終了して所有返却済み、背景で続行するprocessはない。

一時Cargo runner導入とDPI150/200確認の移管は未回答であるため、ここで承認待ちにする。残りは承認範囲の計画/設計と検証手順整備→通常/試用full/関連crate gate→native確認→最終提供Post→ignored提供folderへEXE/使用説明/manifestのcopyである。release buildとFX1の84PASSだけでET3をClosed又は提供済みとしない。実API/実マイク等の利用者受入は引き続き別項目である。
