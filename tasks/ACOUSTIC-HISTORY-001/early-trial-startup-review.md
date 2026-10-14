# ET2 起動接合・独立レビュー

## Pre-1：CHANGES REQUIRED

2026-10-14。担当 `/root/trial_startup_review`、指定gpt-6-astra/high、thread `01a1393b-2c35-74d3-a853-57441be994a3`。read-only担当の返却を親が保存した。HEAD `97d2c72a9adec4f1904c6deb2b18e92c50262923`、design SHA `D9E044C42C85D1C1914B2E24820AE31D6B4082B7B920EC9F4E5C5CB1DDBBB42A`、plan SHA `38C5486BF336465C85EEC18985EA68CC0890DE02F27A6E215494DEF045E409A9`、正本契約SHA `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`。6source基準hash一致、source差分0である。

| Finding | 原因・現物根拠 | State / packet / canonical owner | Oracle・残余 |
| --- | --- | --- | --- |
| ET2-PRE-01 / P2 / 受入阻止 / 確信度高 | 初回録音正対照の実行前に、watchとRecorder::start先頭の遮断接合を確認する条件がない。監視定義だけあり入口接合を漏らすとcompile可能であり、media.rs:62のcpalへ到達後にcounter assertが失敗しても遅い | Accepted / ET2-PR1 / 設計者が設計の唯一の規範条件を修正、plannerは実行計画へ参照・順序を反映 | source固定→親が共有global/active判定/加算/合成Err/poison拒否のcpal前完結を読取確認→そのsourceから作ったEXE hash固定後にだけ実行解除。後続動的counter検査の代替ではない。実装/runner/Postの証拠は未取得 |

最小是正は上記実行前安全Gateを設計・計画に明記し、新hashで再Pre判定することである。未確認なら正対照を実行しない。未監視test・非test本体・限定所有を維持し、感度実験でも遮断自体は撤去しない。新製品判断・公開APIは不要である。

その他はload1回/fatal時App非公開/成功時bool・diagnostics/creator所有移譲/実回復後rename拒否/回復後read拒否/JSON先頭拒否/private credential selector/通常構成を現物照合し、確定阻止0であった。OS共有拒否と全動作は未実証、ET3全体/release/native/DPI/製品EXEは未完了である。編集・実行試験・再委譲・実資格情報/API/録音は0。

指定モデルとprovider返却値は別であり後者未取得。担当が返したcounter観測は07:06:07.683–07:09:57.372 UTCまでで、その後の検討と最終応答は含まない。詳細は親のusage記録へ接合する。

## Pre-2：ET2設計PASS / ET2-PRE-01設計上Closed

同じread-only reviewer（Astra/high、同thread）の独立再判定はPASS、阻止0、要回答質問0、追加提案0。親が返却結果を保存した。対象HEADは上記97d2c72、design SHA `24B9FEBE6F83528EF1B4D55EF9890B00FC9846E9F50B5BB446F48A2FAE687D7E`、plan SHA `82E0A3E8A326EBEB2456ACC994731895072F3B714286452598435547ADC31AFD`、正本契約と6sourceの基準hashは不変。

ET2-G0でsource固定→親の共通global/cpal前拒否/poison/watch寿命の読取確認→対応EXE固定→親明示解除→動的正対照の順が確定した。接合漏れ・別global・EXE不一致は実行前に停止する。source/EXE/構成変更、感度実験・逆復元後も再確認する。planの旧即時実行条件と加算を崩す変異は除去され、録音の遮断/加算を保持したcalls観測返値だけの変異へ限定した。動的counter検査、通常未監視/non-test動作、所有6file、既定gateを維持する。

設計全体・plan追補・是正記録・実差分を照合し、規範競合なし、参照欠落0、diff検査成功。既存factory/実回復障害/復元拒否/credential selectorの判断も維持される。ET2-PRE-01の現在状態は**設計上Closed**である。

これは設計欠落の解消であり、実行解除ではない。observer実装、動的安全証拠、実ET2-G0、ET2機能/最終Postは未了。実装後の同指摘のclosureも要求する。ET3全体test/release/native/DPI/製品EXEは未完了、reviewerの編集・実試験・再委譲・実資格情報/API/録音は0。

## 親ET2-G0：候補C1のsource読取確認（EXE未固定・実行禁止）

HEAD `9a7eb5c`＋実装担当の本体5file差分を親が読取確認した。対象6sourceの全SHA256は[実装記録の固定source表](early-trial-startup-implementation.md)と現物が一致した（main A646CB6B…B1489D／app 2AA13D47…0C3C56／qwen_reading 3910E004…8CBB64／qwen_settings D8475559…9E11C8／qwen_settings_tests 45EF01F0…E324F1／media 14914D66…9B3207）。構成は通常とearly-trialであり、双方ともまだEXEを固定していない。

親はmedia.rsの専用試験全文とglobal/登録/Drop/startを読んだ。登録とstartは同じ `RECORDING_START_WATCH` を参照し、start先頭の同一lock内でactive判定→fetch_add→固定Errを完了する。poisonはmap_errと`?`でcpal前に返す。cpal::default_hostはその分岐の後にある。Dropは同じArcの登録だけを解除し、poisonをclearしない。正対照は登録成功を必須とし、直接入口・worker入口とjoin・回数確認までwatchを保持する。unwind後はDropして再登録し、poison対照もworker join後に検査する。既存3callerは同じ入口、追加の入力stream入口なしである。

この確認はno-run buildへの移行だけを許可する。Recorder正対照・新bin caseの実行解除ではない。確認済みsourceから通常/試用別target-dirで実行せずにbuildし、前後の6source一致、JSON由来EXE絶対path/SHA/構成/実名を固定して再提出する。source/EXE/構成変更で本確認も再判定する。動的安全対照・機能試験・最終Postは未了である。

## C1独立静的Post準備：ET2-P01 / P2 / 検証不足（Accepted）

同じ独立reviewer（Astra/high、read-only）はC1 source6hashを前後照合し、確定本体機能欠陥0、最終受入を阻止する必須検証不足1件を返した。最終Postは未判定である。

`src/app/qwen_settings_tests.rs:31`のfactory実行後32行で通信spyを生成している。Wireは47行の独立QwenDialogだけに接続され、64行のwire.count()==0は先行する起動・設定保存・復元の送信を観測しない。Probeもfactory後のeditorだけで起動中は観測外である。契約ET-06、設計の受入表、計画の必須集合5の動的送信/probeゼロ証拠を満たしていない。Wireを前で生成するだけでは未接続なので解消しない。

親はET2-P01をAcceptedとする。現起動・保存・復元経路に自動送信は静的には確認されていないが、これを必須mock観測へ代替しない。planner/designerへ操作別の実通信境界と不足接合だけを返し、独立作者→実装者→runnerで動的証拠を取得する。追加所有/新APIを親や実装者が先取りしない。これは最終受入証拠の不足であり、録音G0のsource安全性とは別である。既存C1のno-runは続行可、runtimeは依然未解除である。

その他の静的阻止0：load一回/fatalApp非公開/D順/通常constructor、main安全表示とtitle、実回復失敗oracle、restore先頭、private selectorのproduction使用、watch共有global/cpal前拒否/poison/同ArcDrop/workerjoinと非test不変を確認した。独立実行、EXE対応、感度、最終Post、native/API受入は未了。編集・runtime・実data/key/マイク/管理者操作0。reviewer snapshot07:57:35.085Z input3343566/cached3164928/output12173/reasoning3949は同session累積、C1単独消費ではない。

## 親ET2-G0：C1対応EXEの限定実行解除

親は実装者の通常/試用no-run JSONのcompiler-artifactを読み、profile.test=true、通常features=[]/試用features=[early-trial]、lib/binの絶対pathを照合した。そのpathの実EXE SHAは[実装記録のC1 no-run表](early-trial-startup-implementation.md)の4件と一致した。6sourceもbuild後の現物がC1表の全hashと一致した。したがって**その表のC1 source/config/4EXEの固定組だけを、下記限定順序で実行解除する**。

- 通常bin `…/default-target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe`：`3E2F2D987FD73553086C425229E1ED39E4A5B630690E42214ABD6CF62D24EF7A`。
- 試用bin `…/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`：`15B231707BB6FE50E3658688E3EC542FF82AA64D47D0B97A154BF41AE6167687`。
- 通常lib `…/default-target/debug/deps/wordweave5-b3388a7bbef44bb4.exe`：`E9AC3D17DEE4AB0BA73F3AC6613B3EE4AA83770AF187142C38083985D511D974`。
- 試用lib `…/trial-target/debug/deps/wordweave5-1e58dd7001d62083.exe`：`135273D6F8A1C837ED701C2F0718A2F47E274E6852D843CBF89D719845906415`。

接頭絶対pathは全て `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/` であり、JSONの区切り表記差以外を変更しない。まず実名listを作者集合と照合し、各構成で安全#4→#5→#6のexact3件を成功させてから残る新caseへ進む。新caseは試用bin17+lib1、通常bin6+lib1である。毎EXE起動の前にWindowsIdentity/PrincipalでAdministrator=FalseとSHA一致を強制し、true/不明なら実行しない。通常ユーザーのsandbox外実行だけを必要時に使い、Hidden/有限timeout/kill+wait/合成TEMP・LOCAL/fixture残存確認を保持する。管理者/UAC、実API/実key/実マイク/nativeは解除しない。

失敗/timeout/worker未終了なら解除を停止して親へ返す。source/EXE/config変更で解除は失効し、感度変異/逆復元も再確認する。ET2-P01は未解消のままであり、既存caseが成功しても起動/設定/復元の送信観測完備やET2最終受入PASSとはしない。今回は不足是正計画と並行してC1 fixture/機能の健全性を確認する限定実行である。

## C1動的結果とfixture是正C2のsource再確認

実装者のC1実行で実名集合は通常7/試用18と一致。録音安全3件は両構成PASS、通常新7PASS、試用5PASS/#3FAIL/後続12未実行で停止した。全起動Administrator=False/SHA一致/timeoutなし/wait完了/合成fixture残存0。失敗はmainのHistoryPresent安全文assertであり、旧actual本文はログ非出力なので分類を実測済みとはしない。

独立作者がstrict ProgressWireの必須6field欠落を確認し、main専用cfg(test)だけを是正した。旧malformedの記録不正/BeforeWrites拒否と全tree不変を対照として残し、全必須fieldを含む履歴入力で元のHistoryPresent exact文/AppCreatorなし/Dなし/全tree不変を維持する。期待値弱化・本体修正なし。新mainSHA `66BDD7137811C5C954A12C8C1E5461D5AF8933DC832D2282F2F37ACDE45C2025`、作者moduleSHA `84A5F2A3DF6E645B105A5453FC51B3A6E835D8FEA9F0B5895C6FD4A7B803149C`、他5sourceはC1表と一致する。

親は新main試験と現物6hash、mediaのglobal/登録/Drop/startを再読取した。同global・cpal前lock内active加算/Err・poison拒否・同ArcDrop/poison維持は不変、正対照のwatch寿命/workerjoinも元moduleと不変である。C1解除は失効している。C2はno-run再buildへの移行だけを許可し、JSON/EXE対応が未固定のためruntime禁止を維持する。ET2-P01は別の限定是正設計待ちである。

### C2対応EXEの再解除

親がc2-default/trial-build.jsonlのcompiler-artifact/profile.test/features/lib/bin pathを読み、現EXEの全SHAと再build後6sourceを照合した。全て[実装記録のC2 no-run表](early-trial-startup-implementation.md)と一致。**C2の固定6source/config/4EXEだけを再解除する**。pathはC1と同一だがbinaryは置換済み、C1解除は再利用しない。

- 通常lib：`742247DBA6FE919F23FA6F927BB153B46EC931B2C517FE88FE577B2455E3FC18`
- 通常bin：`94C2D7F3630F3F801D5E1B739D7997515A602AC97F00F2B3BC78706C30EA63F6`
- 試用lib：`16D61A43A2EC274D90D37A5E1D927FFCF06DAAB624C6AFD446C3DAE465F151E3`
- 試用bin：`F89BF3B3B00465B288699941A374D4A93CEDAD01547675FEC1C2B2A8C67EAAD6`

list照合→各構成の録音安全#4→#5→#6→試用の是正exact#3→C1後続未実行12件の順で進める。通常の他新4件/試用#1#2は必要な最終集合へ接合できるよう現EXEでも確認可。毎EXE前Administrator=FalseとSHAを強制、Hidden/有限timeout/kill+wait/合成TEMPLOCAL/0件除外/残存確認をC1と同じ条件で保持する。失敗時即停止し親へ返す。source/EXE/config変更で失効、通信P01の新安全対照をこの解除で開始しない。P01/感度/独立最終Post/ET3は未完了である。

## ET2-P01-V設計Pre：PASS（指摘0・実装後closure未了）

独立reviewer Astra/highは設計SHA `8AC25D82C1286ED1C4500BC3DFCF232E76DC01673E35758833C7E373C96F82F3`、plan SHA `BCD69DBC7B1EA022598374A93D967DE5DA8239427FF5443C3D55F5E2F5E3A785`、契約不変、HEAD9a7eb5c＋設計末尾8source表を前後照合しPASSと判定した。指摘0、リンク/diff成功、source編集/runtime0である。

4file限定のCodex6出口/実appDialog/probe先頭/factory前合成Store選択、同global・poison・保持Lease・guard終了後拒否が現型と整合する。受信とworker終了の区別、同期Runtime drop/panic cleanup、両構成×Store有無×5操作の累積全0を要求し、旧観測不足を設計上是正できる。単一G0は初回正対照/変異/逆復元に適用し、旧録音限定解除は通信へ流用しない。

ET2-P01はAcceptedのまま、Closedではない。実装/compile/新安全Gate解除/動的安全対照/matrix/最終Postは未了。C2#11のmemories不足はrestore前のfixture受入未了であり本体欠陥と判定しない。reviewer同session最終観測08:25:51.820Zはinput4574818/cached4332800/output15702/reasoning4553、本Packet単独量/最終応答ではない。

## C2#11停止・一括fixture是正C3のsource確認

C2は録音安全3件両構成と是正#3、試用#7–10がPASS後、#11で`missing field memories`により停止した。restore呼出前のfixture構築失敗である。C2実行は11PASS/1FAIL/13未実行、全起動Administrator=False/SHA一致/timeoutなし/wait完了/fixture残存0。成功集合/失敗ログは実装記録へ保持した。

独立作者の全18case一括点検は、正常JSONの構造不備を#11の5field欠落1箇所へ限定した。他17件の追加構造不備は静的には見つからず、未実行をPASSとはしていない。作者が#11の必須field補完とfixture名付きexpect、mainのDeserializeに関する誤コメントだけを修正した。ProgressのDeserializeもProgressWire/normalizeを使うため、以前の寛容という作者説明は訂正された。本体・期待値の弱化は0。

親は実差分/8source現物SHA、録音global/登録/Drop/startを再読取した。新main `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40`、新app `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962`、他6sourceはP01設計表と不変。録音の同global/cpal前加算拒否/poison/同ArcDrop/正対照寿命は不変である。C2解除は失効、C3はno-runのみ許可する。

修正した動作caseは試用専用cfg(test)内であり、mainの他差分はコメントだけである。直後の焦点検査は既存trial-targetのbin no-runだけへ限定し、通常/libの成功は元snapshotの範囲に保持する。最終P01/独立runnerの通常・試用fresh検証やET3全体gateは削減しない。通信sourceは未実装で、通信正対照は引き続き解除対象外である。

### C3試用binの限定再解除

親が`c3-trial-bin-build.jsonl`のprofile.test/features=[early-trial]/bin絶対pathと現EXE SHAを照合し、8sourceもC3表と一致した。**試用bin `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA `3B211C36C9FB10DDE7D04CB01906CE19FA1FBD77AD11AFC5AA03815FB8E201E8` のC3固定組だけを再解除する**。

listでbin17件照合→録音安全#4/#5/#6→是正#11→残るbin新caseの順で、試用bin新17件の現snapshot確認を行う。通常/libは今回再実行しない。毎起動前Admin=False/SHA強制、Hidden/timeout/kill+wait/合成fixture/残存確認は従来と同じ。失敗/未終了なら解除停止、source/EXE/config変更で失効する。通信P01正対照・感度・最終独立runner/Post・ET3は許可対象外である。
