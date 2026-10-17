# ET3 試用EXE提供・独立レビュー記録

## P7-D4 guard限定接合（2026-10-18、進行中）

G1独立Pre PASS/阻害0（reviewer p7_binding_review、指定Astra/high）を受領した。対象design184289A35BF03FD4101789E39120F4F9E159B4947BB30AB195A7314F7E879A4A/仮plan835C、凍結main2文書・承認snapshot811BF一致。MI帰属の旧証拠を再利用し、元consumer BB5B1C53…/7helper閉包、変異offset/MI内外/parse、前後validator/復元算法不変、sentinelとexact拒否stack、5/60秒、CreateNew保存/marker/実exit/最大3観測/親freezeを独立確認した。実候補script/stack成立・G01–03/FF18/Eは未実証。

plannerは01AD21496978F7D4D03191FACEB0C6A5F8679D52C2C3FE2BBF787E60D78EE32Cへ限定精緻化し、親が受領してG2作者p7_d1_author=Astra/highへFF7literalと候補rootだけを解除した。候補root P/d03fbe3a-1318-4b39-8ba5-110b588e1640 に承認記録approval-review.snapshot.md（811BFEC050ECA27D06E16497B30500F9E3CB45C7DC4525CFF6E03D7BA0A64128）を非上書き保存したとの通知を受領し、親の索引更新を再開した。design/最終planは以後freeze入力として保持、上流へ逆bindingしない。作者runtime0、別静的review/親freeze前のG3実行は禁止する。

利用者が限定設計→独立レビュー→修正・検証を承認した。planner新追補p7-guard-binding-plan.md（仮SHA835C7A749F9D501142966C8D75DE30E987406223875459382FB5123E241BCFCA）、designer Astra/highへ限定design所有で引継いだ。既存凍結main plan/designは変更せず、旧6literalと追加guard1literal以外のFF本文不変、承認MI出典と未知変異拒否を要求する。実装/行動実行はPreまで0。

親のE前提read-only確認：独立受入済みCXT-D `P/cxt-runner-20261016-approved-b` のresult.jsonは既存REPORT記載SHA6C32037803AA5C694A193BEB11327E8D3732166957EB17898462250A553197E9と一致した。REPORTの15入力/bytes前後一致・toolchain279bytes/V2 6362bytesとinputs.json（0C31523A8A07F9DFC2FD1BE2A46A6E5D0EDC9DC397E92EAF57E5C642E4BBF11E）の完全値を対応付けた。toolchain.json A16668FDD9C5F3A684FD6C7A43CBBC7CEC34F9F3A8FA36575C1798466BA6785A、V2Case572E56BE23FDD59B949A678BAEACAFE4E035434D064B9B4D28425D781AFF1168は現物と一致した。現物hashだけから新規承認したものではなく、過去独立受入の固定15入力へ由来を結んだ。後続runner/reviewerへこの索引を渡しE freezeへ接合する。旧CXT-Dのruntime環境や成功を新E実行結果へ転記しない。

## D1–4／E再開

### D4追加validatorによる停止

独立read-only確認（p7_binding_review、指定Astra/high）で受入阻害1件を確定した。保存MI packet a1e2d0c5-5d2f-4d6e-97a2-0b6dea11cfeaの旧Invoke33C2は復元SHA08600941…、新318Eは3768A05D…である。現898Cと318EのRead-OwnedTree全体がOrdinal一致し、両本文の7.6.6/7.6.7 literalは0個、差分は既知process欠測の証明分岐1477–1559である。従って版literalの影響ではなく先行MI本文変更とFFの旧全本文guardとの接合漏れである。2call/各7要素/固定分類/最後のUnprovable参照/逆順置換/UTF-8 hashの元算法も独立一致した。D4は未編集でplannerへ返す停止checkpoint、追加scope設計・修正・FF18/runtime0。現SHAへの機械置換を解決策として承認しない。

D4作者は編集前にFF230の追加必須guardを検出した。Read-OwnedTreeの2診断callから末尾4引数だけを元算法で除去した復元本文について、固定期待08600941CB5669B76A9EB938A40B7362203967C2ED167C00D2F2DA6EA5B03E5Eに対して、承認Invoke898C…からの実算3768A05D1C6B65031492F1C5F8775900B636E5D1AB4AEDDDF5B0F3463AE0F3AFである。main705のこのguardは18caseより前であるが、実runtime拒否は未観測。plan683の停止条件を適用し、FF6literalも未適用、FF AD492926…/46482bytes全一致、上流11入力一致を保持した。作者証拠P/35186079-b5d6-4e51-b19e-a608b67dadc9/REPORT.md SHA C1C255DEE6D5688691D5F8903C2CE28771323D2A7127A259027EEB2813E0013A。MI変更との関係は候補であり差分全体の帰属は未確認、現値への無根拠な置換をしない。独立read-only確認へ提出した。

runnerのE草案はP/f3f85d34-7dbf-49ff-81b7-82d1d756c7d4/P7-E-DRAFT.md SHA84D46E7C9F68939B5CED3C354D1EF129AB28F21E23069A350514D9455593C7E5。7command/別UUIDを準備したが最終FF未承認、親freezeなし、全実行0。新58期待＋ValidateOnly2はNotRunであり、C16の限定Reuseを新実行成功へ拡張しない。Context117–126のtoolchain.jsonとV2Caseの完全承認baselineも未確定。argv oracle7D74…/fake source3482…/copy fake286E…は既存設計426/454とplan685に指定があることまで親が確認した。旧F3/full/製品提供は未解除である。

D1独立binding Post PASS/阻害0を親が受領し、Context464A33E5…/P1914C56FD…を承認、D1 Closedとした。逆全bytes/旧履歴非規範化/新組と非循環を独立確認。oracleのContext consumerを大小文字含む関連29参照まで追跡し、動作へ渡るAssert定義とRuntime assertion、9fragment/型/親/位置/3adapterがC保存と同一と確認した。C16のE再利用は局所版判定/拒否位置/sentinel/producerの8ID16観測だけに限定して承認する。Cの旧Context01E0…before/afterを新Contextの実行結果/sourcePreservedへ表示しない。Eの最終新組freeze/前後整合/環境観測は新規採取が必要である。

D2/D3は既定実装者Sol6.1/mediumへ、D2 Slash/HANDOFFの2file→別枝D3 ENVprobeの1fileの順で、最大2ChangeSet/同じbinding失敗モデルとして渡した。6/4hashと対応説明以外不変、before保全/逆bytes、Handoff旧prefix保持、Parser/PSSAを要求する。実case/ENV01/ValidateOnlyはEまで実行せず、D4はD2/D3の独立binding review後に別作者へ渡す。

D2/D3返却組はSlash80ED5491EA802FFF0D1000934B92AE68C248ED4A7073F6C0E182204BAE5C81AE、HANDOFF59CAEF6F8BFC49D81172D8BCECBE6F49687A8F65FF47EF89D1EB0341A8188E66、ENVprobe0B6C78196FD5DB0CB5AE99364BC26720C99E2214467A71FC32156E2A12E91951。証拠P/4e26af9d-a68e-46a0-93ab-2c335a924f57/REPORT.md、D3直前copy P/8c3990de-b224-42f2-bd91-70a202aeadce。6/4hash＋各runtime説明1以外の逆全bytes/BOM一致、HANDOFF旧25279bytes厳密prefix、非所有10入力不変、Parser/PSSA1.25通常診断0/engine0/marker/exit0を実装者が報告した。独立binding Postへ提出中であり、実ケース・ValidateOnlyはまだ実行していない。

D2/D3独立Post PASS/阻害0を親が受領し、新3SHAを承認した。Slash54/ENVprobe128–130の完全hash比較が抽出/ValidateOnly/書込み前に旧組を拒否すること、case10・16値guard→17番目a/b・firstFail/NotRun/期限/cleanup不変、probe12inputの型/順序/元例外優先不変を確認した。独立Parser/PSSAも通常診断0/engine0/marker/exit0、非所有10不変である。D2→D4依存を解除し、別作者Astra/highへFF6literalだけを渡した。旧FF AD492926E4DC4CA84766710D41CFF9024065A88D79BAF310226B96026DAECD46は過去FF-TEST-ORDER01独立Postの完全承認値と現物が一致した。

D4作者と、独立runnerSol6.1/mediumのE実行準備だけを並行する。FFの変更所有と新UUID内の手順草案が非重複であり、runnerは可変FFを読まず最終FF SHAを未承認欄として残す。親が最終組・完全command・新OutputRootを凍結するまで実行0。新frameworkや実行基準の変更0、7役全再起動0である。

既存planFF96F8DE/design6F845988・C16独立Postを再利用し、D1作者を既定Astra/highへdispatchした。P7-C-FROZENの14入力とP1/C Context保存snapshotが現物一致し、環境driftを検出しなかった。新plan/framework/既成功16再実行0。D1 REPORTはP/da92a6e9-4299-44d6-b7ff-4d904907e586/REPORT.md（FE8D2024A5335421ECF67FD5B51AF6CCC4C349099B8B269BD5B5A48BB22A06A2）。Context464A33E5012416F0844B8F381769CC7926D694765742F0C65B46C7F8BBF2325E、P1 914C56FD696523D58AC40820DB4A3EEAC4CE2CF8CB2510C78744060418837714、4literal逆全bytes一致/27793bytes・BOMなし、P1旧11234bytes厳密prefix＋3861bytes追補。9fragmentとoracle全consumerの静的不変、Parser/PSSA1.25通常診断0/engine0/marker/実exit0、非所有13・追加保全6不変を作者が返した。独立binding reviewとC16のE限定再利用可否を依頼中であり、実Context12/互換性完了は未受入である。

## P7-C版接合（2026-10-16、限定完了）

既存planFF96F8DE/design6F845988のPreとP7-B復旧/目的REDのPostを再利用した。plannerの実行handoffはP/170dad30-7c4c-41b1-a9c7-6ccb5401098f/P7-CDE-HANDOFF.md（B251D49B…）。親が仮説として挙げた新manifest必要性は、現oracle408–409の旧text又は許可版置換受理と現sourceのthrow literal認識を根拠に撤回した。oracleBCF9/manifest5FC4は不変である。

実装者Sol6.1/mediumはInvoke20/1045/1083、Test26/742/2130の各4literalだけを7.6.7へ変更した。Invoke898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71、TestF0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919、各4byte差分/逆全bytes一致、BOM/長さ保持。作者Astra/highはContext15/113だけを変更し01E0CCB3B675724FB5C9D1A64E5C312558E76508E191AE8176B7C1354A66E376、27793bytes逆一致/BOMなし保持、binding4/12期待不変。全3fileのParser/PSSA1.25通常診断/engine0・marker・実exit0を確認した。

独立reviewer Astra/highは3file静的Post＋既oracle適用Pre PASSを返した。9fragmentが各一意で型/親scope/位置と許可置換一致、runtime seam各1逆一致、現在throw由来classifier/schema→型→版/producer/16期待が整合し非関連例外の誤PASS変更0と確認した。親はP/ea0d7135-f168-407c-a71a-891d49f2687f/P7-C-FROZEN.md（4FC41BE8A1A7EABAA35E49CE4F5B8C2F94A00EE494C992D0ED27743553013A2A）の14入力・完全argvを承認し、runnerSol6.1/mediumへ新16観測一回を渡した。出力予定P/9bcbb13b-5482-40dd-8260-a930efb8ad61。現時点で16GREEN/Context12/D1–4/Eは未受入、旧F3等も未解除である。

実行結果は8ID/16観測すべて独立期待一致、16Passed/0Failed/0NotRun/実exit0/起動一回/retry0であった。REPORT同freeze dirのP7-C-RUNNER-REPORT.md（711324BFC3FA2068B206FE7D7627DA0B6C8BF09571997D972089C903DA01B3FD）、result（549D8501BED961B9A168CD5FC527598B04D28751D5532BA3C934E473EACB40C0）、125照合（351FB5230C314420FAE8811B4FE50AAFAE45301907DDAA71B712C83AB83876D0）。14入力＋freeze前後一致、11hash全Matched/postsave0/source3beforeafter一致、after/result各一回/最終正常保存、consoleの日本語Ordinal/ASCII JSON/marker整合を確認した。

独立動的PostはPASS／P7-C Closed。拒否は元sourceの型/message/ID/file/行範囲と一致し、欠落Schemaのsentinel0,0,0、null Typeの1,0,0、版拒否Versionの1,1,0、producer単一string7.6.7を実証した。非関連例外のPASSは認められない。全source消費不変のD1後証明まではE16再利用未承認、Context12/D1–4/E/旧F3等は未解除、局所抽出main非到達は静的根拠でOS実監視ではない。raw bytes/hard watchdog保証なし。共有週枠表示100%につき既定C完了checkpointで区切り、後続を新たに開始せず未了として保存する。通常EXE・製品source・学習データ/APIは変更していない。

## P7-B保存境界の復旧（2026-10-16）

利用者が修正・独立review後の旧V01一回再実行を明示承認した。復旧briefはP/f6e31ed9-5f00-460f-9db2-f0c94082464b/P7-B-RECOVERY.md（B1304A985C1AA45137B61EC7F33930D96E15960C1D352887751BF6D77EA0277C）。独立R-A Pre PASS/新規阻害0であり、親が受領した。旧oracle446以後だけを別UUIDで修正しprefix1–445、16期待/9抽出、凍結main plan/design/manifestと旧証拠は保持する。

主例外保持は439–443で既に保存されるclassification/message/stack/positionと主観測配列の保持であり、未保存ErrorRecord情報を復元しない。保存境界T1–4は11照合（source3/documents2/environment5/manifest1）、主状態・副次エラー分離、after/result各一回、console/marker/明示exitを確認する。T4注入はcreated=true、created=falseは未初期化参照がない制御フロー確認と区別する。作者Astra/highが修正/局所検査、独立reviewer Astra/highが確認後、親の最終SHA/完全command/新UUID凍結を経てrunner Sol6.1/mediumへ旧V01一回を渡す。現時点で修正版/旧REDの受入は未了である。

R-B候補はP/5de4073f-f48f-4295-8e75-c2b93918663f、oracleFD49BCD9BD9CC37F02347877CD10A95E891350490EE93916C9D1AEE102E32B15、Test-TailCase6219B4F8850CB1E52DE6F7AD15477B43F07010EEEAD63C367ADB7E33D25E208A、manifest5FC4…不変。prefix23318bytes一致を独立確認した。static初回にfixtureのPSAvoidOverwritingBuiltInCmdlets1診断があり、作者が理由付き関数局所Suppression後static02を実行したが親はruntime前に採用を留保した。独立相談はPASS：既存PowerShellレビュー基準の明示許容、Test-FirstFailure309–311の同rule承認seam先例、usualrule/対象不変、ExcludeRules/dynamic隠蔽なしにより既存品質契約へ適合する。元警告を消去せず「局所抑制1・残診断0/Parser0/engine0」と記録する。tailの主状態保持・11照合・各保存一回・console/exit2・created=falseにも静的阻害0。親はこの組をT1–4用に承認した。局所実結果/独立Post/旧V01は未了である。

R-B作者の4tail結果は各一回4PASS/実exit1,2,2,2/driver0/retry0であったが、独立PostはUnicode保持のP1一件CHANGES_REQUIRED。T2/T4のstdout内postHashErrors.stack/positionにU+FFFDがあり、UTF8 result.jsonと不一致、保存値をCP932 encode→UTF8 decodeした値と一致した。stdoutは復号後文字列の保存であり元raw bytesではない。原因はoracle553の非ASCII JSON直接出力とdriver67–88のencoding未固定、fixture115–127の主seed ASCIIのみで検出できない。console-onlyとなるT3にも波及するため旧V01へ進めない。

既実証の主状態/件数/11hash/各保存一回/noRetry/実exit/T3の2保存失敗/候補5・保護25一致は保持する。最小是正はconsole JSONのEscapeNonAsciiと主seedの非ASCII化、副次例外も保存/捕捉値とconsole全文一致を確認することである。旧候補と4証拠は保存し、plannerへ新候補/newUUID/独立Pre/該当4再検証/Postを返した。16期待/9抽出/prefix/manifest/期限や旧V01一回枠は変更しない。sourcePreserved=false/created=falseは静的範囲であり実証へ水増ししない。

Unicode brief初稿50C107…はfixture終了後stdout観測の責任が未定義でPre CHANGES_REQUIRED一件。plannerがfixture独立期待保存→作者同4実行/self-check→別runner終了後Ordinal全文/JSON7bit読取→Post→旧V01一回へ責任を分離し、最終briefFEA2E6A2E7F4703E97E02FB719C6244A338D22A6D4C5253663C80F5E5DBDA7EFで独立限定Pre PASS/追加指摘0を受領した。親がこの組を承認し、新作者Sol6.1/highへdispatchした。対象2file/driver不変、原証拠と元raw bytes未取得を保持する。

R-B最終候補P/2dd54494-5095-4ea1-b363-85c23024cbebはoracleBCF9F4B823158DEBB05D0671F07414DA0542C69BF7382EA3E8F167A0B9708DE9、fixtureBD653FBCE975A8DF45869DC2A759DDF2C343C619CA9D0AAD53B9993B84F030A3。Parser/PSSA残診断/engine0・既存局所抑制1、prefix23318bytes/manifest/driver/16期待/9抽出/期限保持、候補5/保護121一致。新tail4組各一回4PASS/実exit1,2,2,2、別runnerP/17bc606c-88b8-4241-a2d9-55a367c31ac2の923機械チェック/4組PASS（result24030B06…）で主日本語・独立捕捉副次例外全文Ordinal一致/ASCII JSON/保存とexit整合を実証した。独立R-B Post PASS、Unicode P1閉鎖を受領した。923は全oracleケース数ではない。

R-C親freeze P/6a702737-9b36-4df9-a247-3eaaf5f74270/R-C-FROZEN.md（C84A7EB57DBE23A012A7E89743010AA784FCAED8B178548ABD17FA6E05B61028）の完全commandを一回実行した。oracle出力P/8bff74d6-5fcf-4da3-8804-4e9c12d162ec、runner報告/validationはfreeze同dir。実exit1/1.2909336秒/retry0、V01.Invoke.Runtimeの型・message・ID・元file/行範囲で認識された`Fixed Windows pwsh 7.6.6 required`拒否、sentinel0/BehaviorMismatch/executed1failed1passed0notRun15、14入力＋freeze前後一致、11hash全Matched/postsaveerror0/afterresult各一回/主日本語値とconsole一致/ASCII JSON/markerを確認した。runner29チェックPASS（validation6802161A91D58AB5AD5D31022E5A87A1FA1071EC464C5E3643863AC17FC85BE3）、独立R-D Post PASSで目的REDと保存境界の復旧を限定Closedとした。

| Finding / 原位置 | 共通原因・Packet | 最終状態 / 受入証拠 | 残余 |
| --- | --- | --- | --- |
| P7-B-KEY01 / 旧finally454–455 | schema別keyの消費不整合 / P7-B-RECOVERY | Closed / 分離loop、11照合、tail4、R-B/R-D Post PASS | 元manifest改稿0 |
| P7-B-PRIMARY01 / 旧finally446–475 | 事後例外が主結果/exitを覆う / 同Packet | Closed / 主状態保持、事後エラー別記録、保存不能時console/exit2、tail4/Post PASS | created=falseとhash不一致時sourcePreserved=falseは静的範囲、OS破断保証なし |
| P7-B-UNICODE01 / 候補553・fixture115–127 | transport encoding依存とASCII seedの検出漏れ / 同Packet | Closed / EscapeNonAscii・日本語seed/独立捕捉、同4結果Ordinal/7bit照合、独立Post PASS | 元raw bytes未取得、旧証拠保全 |

Closedはこの三指摘と目的REDだけであり、承認済み旧V01一回枠は消費済みである。新16GREEN/P7-C以後/旧F3・ENV01/80/full/提供の解除は0。元失敗を事後REDへ変えず、固定childCountをOS実監視にしない。hard watchdog保証なし。次は既存P7-C以後へ別handoffし、通常EXE/source/data/APIは変更しない。

## TOOLCHAIN-PWSH-DRIFT01：7.6.7採用のPre（2026-10-16）

利用者が7.6.7採用を承認。P7-0同定は実7.6.7/.NET10.0.13、5file前後SHA一致、起動1回361ms/exit0、有限同定8PASSである。pwsh.dll A583…とSMA.dll70701F20…を区別する。互換性/P7全体のPASSではない。

独立Pre（pwsh767_pre、指定Astra/high）はplanFF5DFF7F/designD29184B5にCHANGES_REQUIRED、確定指摘1件を返した。P7-EのTest-FirstFailure.ps1は202–203/659–660の内部AST hashと716–725のTest/Slash/KH/Context旧全体hashで18case loopより前に停止するため、追加binding所有が必要である。plannerへ2内部hash＋4入力bindingだけの限定更新・逆照合・独立reviewを返した。18期待/診断本文/期限・検査自身は維持する。影響確認の仮hashを承認値に使わない。

他の確認範囲ではV8ID/16観測の可逆seam、schema→型→版、拒否時既存診断I/O、ContextへのDriverSha引数、MI実型/保持handle、EO/ValidateOnlyのhard deadline非保証は整合した。実装/runtime検証は未実施であり、FF binding是正のPre再確認後に進む。旧F3FAIL/UNKNOWN-DESCENDANT、ENV/F3/80/full/試用提供の未解除は維持する。

再PreはplanFF96F8DE/design6F845988にPASS、新規指摘0。D4のFF6literal限定更新とD2→D4、D3を含む最終組確認、逆bytes/18期待/期限保持/独立review/親manifestを受入れた。P7-Aの設計上のP1のみ閉鎖し、FF実装やFF18成功には換算しない。親はこの規範組をP7-B以降の入力として承認し、文書checkpoint後に独立作者へ進む。

P7-B作者の新oracle37CD96A8/manifest5FC4DE6Fは16観測/9抽出を凍結しParser0/PSSA0を確認したが、旧sourceへの固定pwsh一回はAdapterFailure/exit1で停止した。原証拠P/a1a298af-29f3-4302-9681-419905f49616/REPORT.md、実行root P/82f079c0-6d6c-47d7-a895-074feed3959d。finally454–455でdocumentsの小文字path/sha256とenvironmentの大文字Path/SHA256を同じ小文字で参照し、OrderedHashtable/StrictModeのproperty欠落となる説明を作者から受領した。実stderrにstackはなく、位置は静的照合である。before/extractionsのみ保存、result/完了marker/元観測結果・例外は未保存。旧V01 RED/PASSと認定せず、再実行0、元3source/規範2/environment5は事後別読取で一致した。独立原因確認を依頼し、修正・再実行条件を分離する。P7-C以後未到達、製品変更0である。

独立確認はCHANGES_REQUIRED/P1二件。①OrderedHashtableのschema混在を現物型/key確認で確定、開始時378–383同様にdocuments/environment loopを分離し正しいkeyを明示する。②finally446–475の事後hash/CreateNew例外が主結果と明示exitを覆い、459が既存firstUnexpectedを上書きする。主観測/主例外を保持し、事後hash/保存失敗を別項目へ残し、保存失敗でも構造化console証拠とadapter exit2へ到達させる最小修正が必要である。manifest改稿/fallback/hash省略/別root退避/retry追加は0。raw stderr fileなし、位置一致は静的根拠であり元結果未確認を維持する。

次は作者の限定修正→16期待/抽出/期限不変・Parser/PSSA→独立review→親の新SHA/完全command/別未存在UUID承認である。利用者へ、この条件下の旧V01一回再実行可否を照会する。期待は元拒否/sentinel0/BehaviorMismatch/残15NotRun/保存・marker・exit1であり、AdapterFailureなら停止する。gate main/child/CIM/network非到達は静的範囲であり固定childCount値を動的監視証拠にしない。旧oracle/manifest/rootは保全し、今回追加runtime/修正は0。

## MI-F／EO-Fの再開（2026-10-16）

F3初回はcontrol2PASS、focused0PASS/1FAIL/9NotRun/実exit1/34.578秒で停止した。suite b5bb9069-90e5-429b-8c5b-97c7f7f22f75／run77a4759e-0508-4cdd-966b-140c3ef89a12、ENV01の元errorはBEHAVIOR_ASSERTION: Unexpected gate failure、AP1未到達/argvEvidence0、事前15→fake後16全入力一致/sourceMatched=true/caseBodiesUnmodified=true。REPORTはR/verification/b5bb9069-90e5-429b-8c5b-97c7f7f22f75.F3-REPORT.md（A739994A）。再試行/修正/残9/80/full/製品起動0。

独立PostはFAIL維持、初回停止・証拠保全は適合と判定した。UNKNOWN-DESCENDANT-OPEN01：最初の正式FFはSupervisor OwnedTree.IdentityCatch/CleanupIncomplete、knownMatchCount0/snapshotMatchCount1のPID20688でInvoke318E:1569 GetProcessByIdが失敗、1571 Get-Identity未到達である。identity=null/waited=false/hasExited=null/killRequested=false、MIの既知保持handle分岐ではない。Runnerの同PID開始04:16:55.7220098Z/終了55.8101757Zに対しSupervisor FF56.0136209Z、RunOwner cancel56.1593637Z。取得と終了の競合は候補だが、snapshot採取時刻や原初因は一意未確定。別actorのDirectLaunchHandle/NotFoundAfterExitをSupervisor当時のownershipへ補完しない。Supervisor Owned18/Unprovable1、RunOwner20/Runner1の終了確認・残留配列空・期限内終了をunknown0/全cleanup成功へ拡張しない。正式FF2/現在tmp0、旧MI/EO局所Closedを否定する証拠はない。

診断補足：今回のtargetはexit93/完了marker欠落でありfixture内部error経路も候補であるが、元I/O errorは未取得。保存Runはmode=target/当該ENV01 root/exit指定なし、target markerはrole=target/argv0/同rootを示す。実target環境記録はET3_FAKE系を採取せず、実継承exit指定の不存在までは証明しない。fake正常fullは既定0で完了marker後に返るが、93単独でErrへ一意確定しない。独立補足reviewにより、先行ENV01 run e0d4d4fdの「childExit93は正常期待値」という説明は撤回する。旧ENV01は既存oracleでPassedのまま保存するが、ENV01共通判定はtarget exit0/完了markerを保証しない（expectedExit比較はTRANS03–07のみ）。正常期待値一致やtarget正常終了を実証済みとはしない。これをfixture失敗/監視失敗の因果調査へ接合し、過去FAIL/PASSを書き換えない。

別の実行前阻害TOOLCHAIN-PWSH-DRIFT01：F3実行時/runner事後は16全一致だが、Post読取時はpwsh.exeだけ不一致/他15一致となり、一時的なpwsh.exe/pwsh.dll解決失敗2件を観測した。F3凍結旧SHA BFB46AF89433268872DDB43D1CA7A3F433452EE91ED356A9786940F90118E285に対し、親のWinPS読取で現SHA FA2C8BB1E35CDD62AD15CD06A1D8F2C5961E9725A9BFA1E7CFD2C322F49E9619、exe/dll FileVersion7.6.7.500/ProductVersion7.6.7を確認した（現DLL存在true）。F3時点の7.6.6/sourceMatchedを遡及否定せず、更新主体/発生時刻やF3失敗原因と同一視しない。追加PS7実行/修理/OS設定変更は0、Windows PowerShell5.1は既存証拠・file metadata読取だけでPS7検証の代替ではない。利用者へ7.6.7前提への固定環境改訂可否を照会済み、許可なしに再採番・実行しない。製品EXE9B888E9F不変。

| Finding ID | 原文・位置 | Root cause | State | Packet | Canonical artifact | Evidence | Oracle | Reviewer record | Residual / out-of-scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| UNKNOWN-DESCENDANT-OPEN01 | F3 ENV01/Supervisor Invoke1569 | 未登録候補の取得失敗。先行target終了/fixture errorとの因果は未確定 | Open | F3初回停止 | Invoke318E/既存所有契約 | FF正式/terminal/target記録、known0、GetIdentity前で停止 | 未登録・取得前終了/元fixture errorの有限切分けが次。未知をgone/Ownedへ救済しない | 独立Post FAIL/停止適合 | 既知MI分岐と分離、追加runtime/修正なし |
| TOOLCHAIN-PWSH-DRIFT01 | F3事後pwsh hash/読取tool起動 | 実行環境7.6.6→7.6.7、主体/時刻未確定 | Decision pending | 次検証の環境固定 | 固定pwsh前提/利用者判断 | 旧BFB46→現FA2C8、file metadata7.6.7/一時DLL解決失敗 | 新前提の承認後、互換性と固定入力を限定再設計 | 独立Post追加観測＋親WinPS読取 | 旧結果へ遡及混同しない、修理/自動adoptなし |

F2後ValidateOnly正常一回＋既定配置guard負例一回は2PASS/Unexpected0である（report R/verification/6cd34167-f41a-4737-a182-5472259a4d73.F2-REPORT.md、SHA24E65BCE）。正常exit0/validatedtrue/caseCount10/variantGuardtrue/runtimefalse、元Test両originと4config順序照合、負例exit1/exact Fresh direct-child UUID output requiredで拒否、両root前後未存在。生成前15入力は正常後/負例後とも一致、fake未生成を16件確認済みとしない。4config hashのnullを空設定の証明へ換算しない。build/control/caseDispatch0/retry0でありfocused10の成功ではない。親は新固定組と未存在新suiteで既定control2を含むfocused10一回へ進む前条件を受領した。最初の予期外で停止し、10PASS/独立Post前に標準80以降は解除しない。

F2 driver02A011C2/HANDOFF E2D99548を親が現物事前固定し、独立接合Pre PASS／確定指摘0を受領した（impl packet94c64483）。6literal逆bytes旧15709一致/BOM、HANDOFF旧21842厳密prefix/非所有10一致、10case/slash16→17/firstFail・NotRun/control2/期限schema/cleanup不変、Parser0/PSSA1.25.0通常rules Stop診断0/engine0/marker/実exit0を照合した。F1限定再利用を現在のruntime合格へ拡張しない。runner focused_runner（指定Sol6.1/medium）へ現組のValidateOnly正常一回→成功時既定配置guard負例一回だけを依頼した。正常R/verification直下UUID、負例P直下UUID、いずれも未存在/未作成、build/control/caseDispatch0を要する。focused10はその成功受領後まで未解除である。

F1独立reviewは阻害指摘0、旧C0–C11のfile/context受入を現context034F/Invoke318E/Test96BBBへ限定再利用可能と判定した。旧結果/保存copy/現hash、4literalのみ/12期待/4mocks/禁止22/P1/期限/89C6保持、Initialize-Run元50文後のC0/C8だけが診断contextメモリ代入/Get-Utcへ到達し、payload入口からwriter未到達、負例の元直接throw・未呼出Read/Confirm・6属性metadataを照合した。旧12PASSは旧組の実行証拠のまま保持する。新組runtime/旧guard2から新driverの受入/FF writer/MI実OS/cleanup/AP1/focused以降へ拡張しない。F2接合Preと新caseの親dispatchは未了である。

F1作者はcontext034F2D04/P1handoffB1AE65B9を返却した（packet3f4e3031）。4literal逆bytes旧F46F一致/BOM/12期待/4typed mocks/禁止22/P1述語/期限/旧89C6不変、Parser0/PSSA1.25.0通常rules診断0/engine0/marker/実exit0。Invoke35AE→318Eは63関数一致/変更3追加2とprefix、Initialize-Run元50statement＋末尾context/Get-Utc、TestD8AD→96BBBは6局所属性＋未呼出Confirm-CaseResult比較1箇所としてconsumer接合を保存した。旧12PASSは独立review待ちの再利用候補である。別実装者focused_driver_bindings（指定Astra/high）へdriver/HANDOFFの6literalだけを引き継ぎ、immutable F1の独立reviewをfocused_bindings_review（指定Astra/high）へ並行提出した。新runtime/無条件旧12再実行は0である。

planner focused_resume_plan（指定Astra/high）は既存plan88F5/design9679の適用で再開可能、新plan改訂不要と確認した。現contextF46F/P1handoff4A74は作者がInvoke/Test/plan/designの必要4literal、別実装者がdriver72244/HANDOFF331Fの必要6literal（同4＋新context/P1）を逐次更新する。過去KH-Cの3/5という当時の差分数を新制約へ誤適用せず、既存の「必要束縛」に従う。旧89C6等RED既定値と履歴は保持する。作者/実装/設計/reviewerは既存割当Astra/high、runner Sol6.1/medium、固定spec/planner Astra/high、親不変である。

F1作者focused_context_bindingsへcontext/P1と新隔離証拠だけを所有して依頼した。旧CXT12保存consumer証拠はInvoke12E7→35AEまでであり、FFのprefix/Invoke-GateFailure/Initialize-Run末尾追加とMI差分、TestのEO/局所属性を新318E/96BBBへ限定静的接合する。元schema/拒否/出力個数/4typed mocks/cleanup未観測の非干渉を独立reviewが認めた範囲だけ旧12PASSを再利用し、不明なら影響受入を差し戻す。旧12やfocusedのruntimeはまだ解除していない。F2の新固定組接合Pre→focused10一回/Post→標準80/87variant→製品fullの依存を維持する。

## EO-STATIC-BASELINE01（2026-10-16、利用者承認後）

ENV01の独立Post PASS／確定指摘0を受領した。suite3d864e43-91b2-4d59-b10c-53c651ab9cf4／run e0d4d4fd-ff06-4da1-a693-79ca4ade08d0はPassed/実exit0/32.19632秒/dispatch1/other0/retry0、12入力前後一致/sourceMatched=true。正式6terminalのhash参照一致、Completed/error=null/所有行waited・hasExited true/unknown0/residual0/killRequested0を現物接合し、重複台帳を合算しなかった。独立remainingは実配列[]、環境/config不変assert後のauthor-result到達もsourceへ対応付けた。fixture救済回数は未計装、残存確認はfinally前、終了後の別OS監査は未実施である。この限界を保持して正常1件だけを認定する。

ENV-RUNNER-ORACLE01とMI-VOLATILE-01は、保存局所証拠＋今回の正常ENV01/Postを満たした範囲でClosedとする。OS欠測そのものの発生理由、旧失敗runの根本原因全体を解明済みとは主張しない。旧FAILは保持する。次はMI-F/EO-F所定の未使用focused/context入力の必要再束縛と接合review→focused10→標準80/variant→製品fullであり、現在の直接dispatch許可や全体成功creditへ拡大しない。製品EXEと実データは未変更、試用提供は未完了である。

| Finding ID | 原文・位置 | Root cause | State | Packet | Canonical artifact | Evidence | Oracle | Reviewer record | Residual / out-of-scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ENV-RUNNER-ORACLE01 | 旧Test2044のexact path比較 | Cargo環境文字列とファイル同定canonical pathを混同 | Closed | EO/ES | plan88F5/Test96BBB/probeE8B465 | 旧R01 RED→新6GREEN、正常ENV01一回成功 | 固定forward literalのみaccept/誤表現拒否、元remaining/config事後確認到達 | EO Pre/test/Post、ES限定Post/binding、ENV01 Post PASS | focused10/80/full/試用未了、旧FAIL保全 |
| MI-VOLATILE-01 | 保存run62900800でCommandLine欠測拒否 | 既知保持handleでも完全OS再取得を必須化 | Closed | MI/EO/ES | design9679/Invoke318E/Test96BBB/probeE8B465 | 旧RED→局所20/実型2独立PASS＋今回正常ENV01/Post | 完全登録/元handle/世代/現存矛盾拒否、未知非停止維持 | 既保存MI局所Post＋今回ENV01 Post PASS | 欠測発生理由と旧run全原因は未確定、finally後別OS監査なし |

ValidateOnly一回はexit0/1.321575秒/validated=true/dispatch0/runtime=false/retry0/suite前後不存在で成功した（packet e91cf4ec）。この時点のfrozenInputsは11件で全一致。probe216のreturn後、fake.exe生成と225の追加を経て実case前に12件となるため、生成前のfakeを確認済みとはしない。親の依頼にあった「ValidateOnly12一致」は集計説明の誤りとして訂正し、コード/集合/基準は変更しない。親は同一probeE8B465/固定組のENV01一回だけを新UUID・通常ユーザーsandbox外で解除した。実成功/cleanup/残9/80/fullはまだ未判定である。

ES-E/EO-D新probe E8B465F9を親が現物で事前固定し、独立binding review PASS／指摘0を受領した（packet13dd15a7）。旧TestD8AD→96BBB、plan1133→88F5の2literalのみ、逆bytes旧EE5F一致/BOM保持、他10入力構築・固定12集合・dispatch/cleanup/期限/期待本文不変、Parser0/PSSA1.25.0通常rules Stop診断[]/engine0/marker/実exit0。親はrunnerへR/verification直下の新UUIDでValidateOnly一回だけを解除した。自己hashをapproval代替にせず、ENV01はその成功受領後まで未解除である。

ES-D独立runnerは固定source96BBBで静的oracle6PASS/exit0/1.2566201秒→EO6GREEN/RED0/NA0/Unexpected=null/exit0/.9649447秒、各一回/retry0、9入力前後一致を報告した（packet fce140bd）。独立Postは同sourceのresult/binding/command/process/logと既確認の全PSSA0を接合しPASS／確定指摘0。EO-STATIC-BASELINE01はClosed、EO-C静的未達は解消した。読取時のファイル名誤記は試験失敗や再実行に混入していない。親は実装者へprobeのTest/plan必要2hash literal更新だけを依頼し、旧EE5Fは実行しない。ENV01/10/80/full/製品提供の成功は未判定である。

| Finding ID | 原文・位置 | Root cause | State | Packet | Canonical artifact | Evidence | Oracle | Reviewer record | Residual / out-of-scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| EO-STATIC-BASELINE01 | 旧Testの6関数でShouldProcess推奨、plan診断0未達 | 内部fixture/回収helperの必須副作用と対話的省略推奨が不一致 | Closed | ES-A〜D | plan88F5→Test96BBB | 旧6同一診断保存→6属性のみ/全PSSA0、独立静的6PASS/EO6GREEN | 認証済み属性行だけの逆bytes96FB、param/body/caller不変、個別理由/局所scope/通常rules全体解析 | Pre/作者static/源static/最終限定Post PASS | ENV-RUNNER全体/MI/80/full/提供は未了。global除外/安全判定変更なし |

ES-B immutable oracleの独立静的review PASS／指摘0を受領した。ES-C実装者は新Test96BBB420を返却し、親が現物hashを固定した。指定6属性各1行だけ、固定oracle一回6PASS/逆bytes一致/BOM維持/非所有一致、Parser0・PSSA1.25.0通常全rules診断0/engine0/完了marker/実exit0を保存した（impl packet9b3d0f70、oracle結果ea1ae69f）。独立runner static_baseline_runner（指定Sol6.1/medium）へ新sourceの固定静的oracle一回→成功時EO6回帰一回を依頼し、静的差分/証拠reviewをAstra/highへ並行した。独立Post前でありENV01/下流は未解除である。

ES-B作者はpacket23598468の有限oracle59B1170B/manifest89B99314とReadOnly96FB（162096bytes）を凍結して所有返却した。commands.txtに3引数・ES-C/D別UUID・期待を固定し、対象へのdot-source/runtime/旧PSSA再解析は0。作者scriptのParser0/PSSA1.25.0 Stop診断0/engine0/完了marker/実exit0を保存した。CLI scriptblock引数binding、不正LiteralPath、自動変数matches命名の3手戻りは別logに保全し、対象動作REDに数えない。別実装者static_baseline_impl（指定Sol6.1/medium）へ6属性挿入だけ、immutable oracleの独立reviewをAstra/highへ並行依頼した。新source/6回帰/ENV01はまだ未受入である。

ES-A独立Pre PASS／確定指摘0を受領した。plan88F586DD/Test96FB/回帰E752/design9679の現物hash、個別consumer、メモリ内属性挿入のParser0/各ParamBlock一個/位置引数2・名前付き1を確認した。SuppressMessageAttributeのみでCmdletBinding/Parameterを追加せず、共通parameter追加を伴わない方式である。公式PSSA局所抑制/PowerShell advanced parameter資料にも照合した。実sourceの全PSSA0と新oracle/6回帰は未検証である。ES-B作者static_baseline_tests（指定Sol6.1/high）へ新UUID内の有限AST/bytes oracle・readonly96FB固定だけを依頼し、既存sourceの変更を許可していない。

利用者は既存静的6件の追加是正を承認した。planner static_baseline_plan（指定Astra/high）のplan88F586DDを現物受領し、独立Preをstatic_baseline_review（指定Astra/high）へ提出した。対応候補は対象6関数のParamBlock直前への関数局所SuppressMessageAttribute各1個・個別Justificationだけである。副作用がないとは主張せず、内部fixture/同期刺激/所有確認済み回収の省略が契約違反となるため、対話的ShouldProcess制御を追加しない。属性以外の全bytes/本文/param/consumer不変、通常rule集合の全PSSA診断0、独立AST/既存6回帰/Postを受入とする。実装/新検証/ENV01はまだ0である。

## ENV-RUNNER-ORACLE01（2026-10-16、利用者承認後）

EO-C限定差分は適合し、独立6回帰は6PASS/FAIL0/skip0/Unexpected=null/実exit0/983ms、一回のみである（runner packet 9eb35a66、source96FB6B4B/testE752D9DA、6入力前後一致）。独立reviewerはresult/binding/green.log/frozenの整合を確認し、追加finding0を返却した。ただし総合PostはCHANGES_REQUIREDである。

阻害EO-STATIC-BASELINE01：plan534のPSSA診断0条件に対し、新旧TestはいずれもPSUseShouldProcessForStateChangingFunctionsが6件、engine error0/実exit1である。593 Start-FixtureProcess、652 New-CaseFixture、903 Set-CaseStimulus、1004 Set-CaseSynchronization、1040 Stop-ObservedIdentity、1235 New-RealCargoFixture。Rule/Line/Column/Message/Severityが新旧完全一致し、新規0/消失0を独立確認した（impl packet 8ff8babe）。期待operandだけを旧値へ逆置換すると旧D8AD全文bytes/BOM一致、非所有10hash一致、Parser0であり、今回の機能回帰ではない。しかし既存であることを診断0達成へ換算しない。元scope外の属性追加/他関数修正/除外/基準変更は未実施である。利用者へ6件の限定是正を追加してよいか照会した。binding/ValidateOnly/ENV01/80/full/提供は保留し、旧失敗を事後PASSにしない。

| Finding | 状態 | 確認済み | 残る条件 |
| --- | --- | --- | --- |
| ENV-RUNNER-ORACLE01 | Implemented・局所行動Verified／未Closed | 旧R01 RED→新6GREEN、限定差分適合 | EO-C静的gateとENV01/Post |
| EO-STATIC-BASELINE01 | Accepted・追加範囲承認待ち | 新旧同一6診断、engine error0、新規0 | 対応範囲/根拠/影響検証の確定、是正と独立Post |

EO-B作者はtest E752D9DAを凍結し、ReadOnly旧D8ADを入力として旧R01一回を採取した。実exit1／ExpectedOldRed／BehaviorRed1／5NA／Unexpected0であり、実AST比較2044・UTF16復元2043の各一意抽出／元本文／source hashを記録した。packet ea23f78eのREPORT/FROZEN-before-red/static-02/old-redとresult a94f9f9bを親が接合した。Parser0/PSSA1.25.0 Stop/完了marker/実exit0、非所有4hash不変。初回静的driverの未対応-LiteralPathエラーは修正前logに保持し、行動REDへ数えない。別実装者Sol6.1/mediumへTest期待operand一箇所だけを依頼し、immutable作者成果物の独立reviewを並行した。新GREEN/ENV/下流はまだ未実行である。

利用者は既定forward slash文字列への厳密一致を維持した期待値修正・続行を承認した。plannerの末尾追補plan D7D6E063を親が現物照合し、独立reviewer `runner_oracle_review`（指定gpt-6-astra/high）がEO-A Pre PASS／阻害0を返却した。既存design53、producer Invoke2022、実Test2044の一意AST statement、適用ENV01/02/03/04/06を照合した。新adapter・旧RED・新6GREEN・PSSA・binding・ENV01は未検証であり、Preを実行受入へ拡大しない。

独立作者 `runner_oracle_tests`（指定gpt-6.1-sol/high）へ新Test-EnvRunnerOracleと旧R01の一回REDだけを依頼した。元Test/Invoke/製品は未変更。旧MI-E失敗と下記MI/FFの限定結果は保持する。ENV-RUNNER-ORACLE01はPlanned／PrePASSであり未Closed、MI-VOLATILEはVerified維持、80/full/試用提供は未解除である。

## MISSING-IDENTITY01（2026-10-16、局所検証済み・ENV oracle矛盾で停止）

MI-E一回はsuite8b8b1d3d-b9d8-428a-90fc-6537a0ee7d02／run3ef364ec-b7c1-4b9b-9d9b-dda74908bc9e、exit1/31.9297878秒/dispatch1/other0/retry0/sourceMatchedtrue/全credit0でFAIL。独立PostはCHANGES_REQUIRED（別root ENV-RUNNER-ORACLE01/P2）。RunOwner/Supervisor/nested Runner/SafetyWorker3の正式terminal全6件はCompleted/sourceMatchedtrue、記録対象はOwned/waited/hasExitedでfixtureResiduals=[]、failure/error全null。重複台帳の20/18/1/2/2/2は合算しない。FF正式/tmp0/Unobservedはgate内部failureなしと整合し、診断不具合へ換算しない。独立remaining確認Test2089とconfig事後確認1810はoracle throwで未到達、rescue counter未取得でありsuite全cleanup成功には拡大しない。MI-VOLATILEはVerified維持／Closed不可、MI-F/80/full/提供は未解除。

ENV-RUNNER-ORACLE01の唯一の契約は既存plan121/design53のforward slash絶対path一語で、Invoke2022が同値をCargo子環境に設定する。Test2044だけがTest739→Get-PathHash297のGetFullPath由来canonical backslash pathを期待していた。保存cargo28052/fulltarget28648の有効90要素UTF16配列は既定文字列と完全一致し、誤期待との差は位置2/11/18/29/36/54/58/69のU+002F対U+005Cだけ。別sampleのnullをchar0へcoerceした初期読取結果は証拠から除外し、有効arrayだけで照合した。native内部argvのbackslash追補design373/379はCargo runner環境値を変更しない。是正の推奨は、観測値を任意正規化せず、独立期待値を既承認文字列へ修正して誤表現の負例を維持することである。

元失敗を事後PASSへ変更せず、runner packet/3e18765c-f3df-43f9-90cd-1e37760f8110/REPORTと独立Postを保全した。execute-review-remediationの仕様/test矛盾時停止条件に従い、ENV oracle期待値修正と続行の利用者確認待ちとする。新Test/仕様変更、再実行、MI-F開始0。通常EXE9B888E9Fも終了時再照合で不変。既存規約への適合、永続data/認証/API変更なし、ローカルcheckpointと製品配布の区別を親self-checkでも確認したが独立PASSの代替ではない。

MI-D probe3literalの独立static PASS／指摘0を受領し、親のValidateOnly一回はexit0/validatedtrue/dispatch0/runtimefalse/suite作成false（packet/976645fb-947f-474f-9724-9e059121e681.mi-validate-only.log）。親が新EE5FAAを事前固定し、MI-E独立runnerへ新suiteでENV01一回だけを解除した。case前後12入力・sourceMatched・正式診断とtmp・元terminal/cleanup・unknownを分離する。成功前の残9/80/full起動はせず、失敗時は既存診断を保存して停止する。自己hashを承認にせず、ローカル22件/FF18は反復しない。

MI-Dのprobe束縛は新EE5FAA688B19049D74D14ED4B3603E3325ADD3FC068F4F84446DA002C9C51389で凍結した（packet/b10753dcのREPORT）。source33→318E/plan0CC4→1133/design30F4→9679の3literalだけ、逆適用全文/BOM旧62DA一致、非所有9入力不変、Parser/PSSA Stop/1.25.0/明示完了/実exit0。親はこのhashを事前固定し独立binding reviewへ提出した。selfhash取得は承認の代用ではない。ValidateOnly/ENV01はまだ未実行、未使用の旧FF test/context/focusedの再束縛は追加していない。

MI-D局所Post独立PASS／指摘0。固定source318E/test8C4D/plan1133/design9679で独立MissingOs2PASS/exit0/0.854196秒→MissingPure20PASS/exit0/0.7871883秒、retry/NotRun/差分0、cleanupFailure=null（packet/6f15bcf2統合REPORTと42318d7d Pure）。元CIM2行と別合成rows、元Process/SafeHandle継続、read open/GetIdentity/add0、O1 null→O2元終了consumer Exited、lifecycle導出0とfixture実counterを区分した。作者20PASSも別証拠として接合し、同担当の再試行なし。MI-VOLATILE ClosedはENV01/Post後まで保留。別実装者へ既存FF probeのsource/plan/design3literal束縛更新だけを依頼、旧62DAを保存し12入力/dispatch/期待/期限本文不変を要求した。新runtimeはまだ解除しない。

MI-C新Invoke318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3Aを凍結（packet/a1e2d0c5のREPORT、旧33/新frozen/inverse/AST/staticを保全）。独立source static/test接合PASS／指摘0、許可1区間逆適用で旧33全文一致・他67関数と前後guard/catch不変、Parser/PSSA Stop/1.25.0配列count0/明示完了/実exit0、抑制追加なし。作者test8C4Dの新MissingPure一回は20PASS/Unexpected0/NotRun0、exit0/0.8090172秒/retry0・source/test前後不変（packet/4c344f66のREPORT）。独立runner Sol6.1/mediumへMissingOs2→成功時MissingPure20各一回だけを解除した。ENV01/ValidateOnly/旧phases/下流は未解除、旧probe62DAは束縛失効のため実行しない。

MI-B test-only独立static PASS／阻害指摘0。新test8C4Dの追加逆適用と旧0C82全文一致、20oracle/元関数抽出/単一event順/型/Rows台帳保全、実型2のCIM1回＋別合成rows/EOF/15秒/同launch handle回収、保存旧REDを確認した。最終static元commandは内側Stop＋Import1.25.0/Invoke-ScriptAnalyzer各ErrorActionStop＋完了marker/exit0、外側code取得/STATIC_PROCESS_EXIT0（chunk3a5f2a/7.1655203秒）。static同logSHAは未取得だが以後無編集とbaseline前後8C4D/finalGetFileHash6245deを接合した。新source接合/20GREEN/実OS2/ENV01は未確認。helpのPhase説明補足は任意改善であり現在のfreezeを再変更しない。

MI-B作者はTest-KnownHandle8C4DFC69A8485C93D90F61739C628822BAEA78EDC397AC05537FDCDBFBA356DBを凍結した（packet/fba04021のREPORT/TEST-EXPECTATIONS、旧0C82保存/追加逆適用全文一致）。旧33C2でM01一回は実Get-IdentityのCommandLine欠測拒否→BehaviorRed1/19NA/Unexpected0、exit1/全0.6115847秒、source/test前後不変。Parser/PSSA1.25.0完了0/exit0、production変更/新20/実型2は0。別実装者Astra/highへRead既知分岐だけを引き継ぎ、immutable test-only独立reviewを別fileの実装と並行して依頼した。旧拒否期待を実装に合わせて変更しない。

MI-Aの計画精緻化を受領し最終plan11332260BC8ADC291BAEF3F4A4DF88A477D2E5256DA584645154A80C9F5F4DB2とdesign9679を固定した。旧KH Pure/Osの再dispatchはせず、新MissingOs2→MissingPure20を現局所受入とする。fresh作者 `missing_identity_tests`（指定Astra/high）へTest-KnownHandleのMI phaseだけを所有して期待固定・静的確認・旧M01一回の合成REDまでを依頼した。新20/実型2/ENV01はまだ解除していない。旧case期待/bytes/証拠を保持する。

MI-A独立Pre PASS／確定指摘0を受領した（planAA92/design9679/source33C2開始終了一致）。元登録producer/Read/失敗catch/終了consumer、公式.NET10.0.12のStartTime cacheと元handle保持、property/null/空/型の区別、親/path/command矛盾拒否、局所20＋実型2のAST/配列/順序/時間/拘束を確認した。旧Pure/Osを再dispatchせずMI-D MissingOs2→MissingPure20を現局所受入とし、MI-Fはfocused10へ接合する対応をplanner精緻化へ返した。実装bytes/局所実測/ENV01/下流gateは未検証、元guardの旧証拠再利用は全文不変が条件である。新実装/runtime0。

MI-A設計は9679B02C1F3F31A6DE701656A52C4DB0DA3A39A289FC8D498C8898EFDE4AA1AE（既存design717〜115行追補）で凍結し、fresh reviewer `missing_identity_pre`（指定Astra/high）へ独立Preを提出した。変更候補はRead-OwnedTreeの既知分岐だけで、new helper/APIなし。propertyの存在＋値nullを欠測条件とし、完全登録/元SafeHandle継続/UTC6/7/現存field照合・前後guardを接合する。局所MissingPure20/実型MissingOs2、旧P09/P20の置換条件、元OS値を復元した観測ではないことを固定。Pre未了のため実装/実測は0である。

利用者がOS情報欠測時の安全な扱いの設計と以降の作業を承認した。現HEAD a84ad3a、source33C2と保存run62900800の証拠を入力に、planner仮計画AA92（既存plan412〜）を受領しdesigner Astra/highへ限定設計を依頼した。現Get-Identityは既知保持handleを選んだ後もCommandLine欠測でsource600に到達する。等価以上の同世代証明なしに欠測を無視せず、完全登録/連続handle/世代/現存field矛盾を接合する方式をPreで確認する。欠測原因は未確定。設計・Pre・計画精緻化前の実装/runtime0。既存FF診断Closed/旧局所PASS/失敗証拠を維持し、主KH/ARGV/80/full/提供は未了である。

| Finding ID | 原文・位置 | Root cause | State | Packet | Canonical artifact | Evidence | Oracle | Reviewer record | Residual / out-of-scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| MI-VOLATILE-01 | run62900800のCommandLine欠測/source600 | 既知保持handleでも完全なOS情報再取得を必須とする | Verified | MISSING-IDENTITY01 | plan1133/design9679→Invoke318E/Test8C4D | 旧M01RED1/19NA→作者20PASS/独立OS2+Pure20PASS、許可区間のみ/他67関数不変 | 完全登録/元handle/UTC6・7/現存矛盾/危険類似入力拒否、元未知非停止 | 設計Pre/test/source/localPost PASS、MI-E Postは別rootでCHANGES_REQUIRED | ENV01全PASS前にClosed不可。欠測原因/10/80/full/提供未了 |
| ENV-RUNNER-ORACLE01 | Test-CargoGate2044のexact native path比較 | 子環境文字列とファイル同定canonical pathを同じ表現と誤認 | Decision pending | 次packet未着手 | 既存plan121/design53を再利用しTest2044期待値だけが修正候補 | run3ef364ec/有効配列90要素/8slash差/Invoke2022、旧FAIL保全 | 既定forward文字列厳密一致とbackslash等誤表現拒否を事前固定する必要 | MI-E独立Post CHANGES_REQUIRED/P2、旧固定12一致 | skill仕様/test矛盾停止条件で利用者確認待ち。観測側正規化/判定緩和/再試行なし |

## ARGV-PATH01（CXT01再構成に依存、動的検証未完了）

### CXT-E focused実動作：ENV01停止・保存証拠の診断中

FIRST-FAILURE01の限定Post PASS／OBS-FIRST-01 Closed（初回failure診断保存契約のみ）。独立runnerのENV01一回はsuite6bb4409b-200d-4fa6-a144-9157a37468d2／run62900800-fee6-4d5d-a590-940ea5939319、exit1/20.4470182秒、dispatch1/retry0/other0/sourceMatchedtrue、Failed/CaseInvocationFailed、credit全0。正式sidecar3/tmp0はactor登録のPID/開始時刻・run hash9A51…ADA8/source33C2・固定名/保存hashと一致した。RunOwner13520の診断file時刻22:33:21.9140827Zはcancel作成21.9526478Zより前で、cancel publishedUtcはfirstAbortUtcと同値であり物理保存時刻とは区別する。RunOwnerのOwnedTree.IdentityCatchでosObservation PID35916/snapshot1/known1/rowFilter1、CommandLine確認false、source600のOS process identity is unavailableへ到達した直接根拠を得た。path確認もfalseだがcandidate=nullであり確定identityではない。OS情報欠測の理由・過去runの同一原因は未確定である。

元判定は維持され、RunOwner正式terminal CleanupIncomplete／Owned12全waited/hasExitedtrue／Unprovable1 hasExitednull／fixtureResiduals=[]、Supervisor/Runner正式terminalは欠測しENV01はinvocation terminal evidence missingで失敗した。残留0・別actor unknown0・診断成功で補完しない。最終reportはpacket/f8918c3a-e070-4e0e-91c7-378426128d40/REPORT.md（SHA5046419529E1653E02AD4E8348F666C8142644E7A2B64F91EFF48DF3B6A0BC2C）。前後凍結入力・HEAD/diff一致、追加runtime/原因patch/OS照会0を独立Postで確認。次工程は保存証拠に基づくOS情報欠測時の安全な扱いの設計であり、このpacketでは実施しない。主KH/ARGV/80/full/試用提供は未了。通常EXE9B888E9Fも終了時再照合で不変。

FF-C最終独立接合PASSを受領した。固定組plan0CC4/design30F4/testAD49/Invoke33C2/probe62DAを維持し、元PSSA Stop/完了根拠接合で証拠不足は解消、再実行0。親のValidateOnlyはpacket/7e3c4862-3e6c-42e6-a003-70a8219ab03a.validate-only.log、exit0/validatedtrue/dispatch0/runtimefalse/suite作成false。独立runner Sol6.1/mediumへENV01正常1件1回だけを解除した。case前後hash・sourceMatched・正式/tmp診断・元terminal/cleanupを分離して返す。未知の強制終了・retry・残9/80/fullは解除しない。

FF-C初回判定はコード欠陥なし／PSSA Execution errors=0のprovenance未接合でCHANGES_REQUIRED。再実行せず元commandを担当から確認した。実装最終33C2/62DAのPSSAは先頭`ErrorActionPreference='Stop'`→Import1.25.0→各Invoke-ScriptAnalyzer→診断保存→完了marker→exit0（session41459開始932f2b/完了faa073、PSSA_COMPLETE count=0）。作者AD49も同じ先頭Stop条件でpredicate正負対照後に解析、chunk891f01/6.8708348秒/exit0/marker count0/最終hashAD49。個別ErrorVariableやError.Countの数値測定ではなく、Stop下の明示完了を根拠とする。既存最終bytesとの対応を独立reviewへ返し、再検査/編集/ENV01実行0。

FF-BはInvoke33C21282E728E3F654C727286F6AE462B0BF4A5D48E4A56A009E0D42A22FE38C／probe62DA68853F6CD11B86206999C1F892F5C578D670F1C0A248453F149AD7C71B9Fで凍結（packet/a5376e00のIMPLEMENTER-FREEZE）。元35AEとのBOM-aware逆bytes完全一致、非所有7不変、最終3Parser0/担当2PSSA完了0。作者testAD49の旧T01一回はBehaviorRed1/17NA・exit1/0.1931709秒、新18一回は18Passed/0NotRun・exit0/全1.0888672秒/max0.3517502秒（packet/9027e098のAUTHOR-LOCAL-RESULTS）。4入力前後不変、retry0、実fileの正式3/tmp1/sentinel2を区分、local markerは実cleanupではない。独立FF-Cへ接合を依頼し、ValidateOnly/ENV01は未実行。probeのselfhashは変化検出用であり、本記録で親が事前固定したhashと別に扱う。

FF-TEST-ORDER01は新testAD492926E4DC4CA84766710D41CFF9024065A88D79BAF310226B96026DAECD46で独立Post PASS／Closed。差分はInvoke-ProbeCase内だけで、共通event列のOrdinal完全一致により12hash→case→12hash／T17hash1を確認する。作者の実predicate正順true/同counts誤順false対照と整合し、他本文/18件/期限/期待不変。旧097は保存版のみで次実行入力にしない。Postは当該oracle是正の受入だけであり、18GREEN/実装接合/ENV01成功ではない。

固定test097の独立read-only reviewはCHANGES_REQUIRED（FF-TEST-ORDER01/P2）。T15/T16はhash24回の順とcase1回を別確認しており、誤った前12hash→後12hash→caseでも通るため、design653のcase後照合を証明できない。作者へ既存T15–17内の共通event順（12hash→case→12hash／T17はhash1のみ）の検証だけを依頼した。18件/期限/interface/実OS0不変、旧test保全、元期待を弱めない。その他baseline RED/17NA・root逆照合・file/seam分離・局所mock抑制2箇所の限定性は支持されたが、最終接合/実行PASSではない。

FF-B独立作者のTest-FirstFailureは0975973DF1C81FD40E1E4D34F135115BFCBBAD88622FFE9583EB9FC897545B04、引継ぎはpacket/54a0a3af-cafe-48f1-9660-726f27406dd0/TEST-AUTHOR-FREEZE.md。Parser0/PSSA1.25.0診断0を明示完了marker/exit0で受領し、旧/新18/probe/ENV01実行0を保持した。T05/T18の局所mockに理由付きrule抑制2箇所があり、FF-Cで期待どおりの限定seamか確認する。未完成probeの原文context抽出/12path集合/ValidateOnly接合もFF-Cの残確認である。別実装者へInvoke許可境界＋新probeだけを引き継ぎ、作者期待変更は禁止した。

FF-B作者がprobeの具体入出力3点とproducerFailure型位置の不足を検出したため、設計だけを限定追補した。design最終30F4AACAF4BD6385887E67DECE3DA398D5FFADEA4CAAEF9400FC3D40370AE45C、独立追補Pre PASS／指摘0。実Get-PathHash295〜298の12入力照合seam、12key結果・前後照合/例外優先順、T15–17 hash24/24/1・dispatch1/1/0、元1343〜1347に合うproducerFailure位置を確認した。旧design56F1参照は履歴として保持し、18件/安全前提/期限/承認回数は不変。作者に期待の固定継続を返した。実装/実測は0。

FIRST-FAILURE01 FF-A設計は56F1DB79EEF0F12B8D669DDCF4A927E97ECFFAD5C098A0A13BA1DDEE6FE9280D（既存design574〜126行追加）。独立`first_failure_pre`（指定Astra/high）はPre PASS／確定指摘0。実物Invoke35AEの初期化/元throw/二caller/cleanupと、元focused/Testの前提・ENV01・oracleを照合し、fixture昇格後context、一回保存、abort先行、whitelist8192bytes、欠測state互換、ENV01だけの入口を支持した。同期I/O停止とpath検査後競合は保証外と明記。設計Preは局所18件/実file保存/実装/ENV01の成功を意味しない。planner精緻化plan0CC4B319BE7F48DC78BBB79EDE857C11C0A7C4321C0450C850AB3DC3348BFB9Fを受領し、独立作者へ18期待の先行固定だけを依頼した。旧context/focusedは今回再束縛しない。新実装/実測はまだ0。

新runの独立診断ではSupervisorのCleanupIncomplete初因を1356 commoncatch／1397 run-bound parent edgeの候補まで絞れたが、保存例外/Unprovable/osObservationがなく区別不能。別時点registrationや他観測者のunknown0から推定しない。既存failureBoundary/terminalでは元初因を保存できず、cancel前firstFailureの専用sidecarが必要と判断した。利用者はこの診断追加と合成正常系1件1回を明示承認した。新Packet FIRST-FAILURE01の計画を既存plannerへ依頼し、元schema/安全判定/期限/unknown・実data/API/OS設定不変、診断保存失敗が元failure/cleanupを置換しない契約をPreで固定する。現focused10をそのまま正常1件承認へ流用せず、診断専用入口に10/80/full creditを与えない。

KH-D独立OS2は2PASS/exit0/0.5952944秒、Pure20は20PASS/exit0/0.4744072秒。その後focused新suite a59b49c6-ffca-4f78-8436-6b46b57d1d7cはcontrol2PASS／ENV01FAIL／9NotRun、child/shell exit1／15.7376213秒／retry0、sourceMatched=true、argvEvidence空。run4de297d1-d99e-4d5e-9a9a-8f74a034bca1ではSupervisor37636がCleanupIncomplete cancelを21:35:48.8079204Zに公開したが、Supervisor正式terminal/tmpがなく詳細errorは未取得。RunOwner12488はCancelled、Owned12全waited/hasExitedtrue・unknown0・fixtureResiduals[]、Runner/Safety正式もCancelled（Safety unknown0）。target23060はG0list用局所結果でありfull/AP1成功とはしない。独立read-only診断へ返し、過去KH再open競合の除去と新cancel初因を推測で同一視しない。

遅着の「修正版12件を許可する」回答は既に承認・実行済みapproved-bに対応する。重複12実行や、新しいOS/focused追試の許可へ転用しない。

KH-C最終独立static接合PASS／指摘0を親受領。F46/4A74/72244/331Fと旧保存4ファイル、3/5literal逆BOMbyte一致、非所有13不一致0、12期待/4mocks/22trap/10case/17argv/期限/cleanup不変を確認した。Read-OwnedTreeはcontext禁止trapで実行対象外、旧12PASSを対象consumer本文不変へ対応付けた。最新HANDOFFの唯一順序を確認し、KH-D独立runner Sol6.1/mediumへ新OS2→成功時Pure20→成功時focused10/AP1を各1回・firstUnexpected停止でdispatchした。標準80/fullは未解除、旧結果合算なし。

KH-C driver最終freezeは72244CD533E6CE3DA1062D05C4A6F9EC5195FE63AAE40C2D7CC01AB7E0AEF3E0／HANDOFF331F2EFD6F9A3222B0B08DBCBF0A0DD38C7EE33B1B99679271A0F5374643C75A（packet/kh-c-driver-bindings-20261016）。旧68EF/A32を非上書き保存、5literalだけの逆byte一致、非所有13不一致0、Parser/PSSA/whitespace0、runtime0。旧gate文は履歴へ区分し、現gateはKH-C接合PASS・親受領後の独立OS2→Pure20→focused10である。context F46/4A74と合わせて独立staticへ提出し、結果待ちである。

KH-C作者のcontext束縛更新はfinal testF46F3B09DEADB8F4A2D1E1F52D09E62F0C746967F824381ED97C6CEBE5C6B4EC／P1handoff4A74FACE49B6314A8DE837E044462DE0B94A587BB7EDC6786C25D0D4672146FF。Invoke/plan/designの3literalだけの逆byte一致、旧handoff全文保存、凍結32件不変、Parser/PSSA0、元context consumerのRead以外全文不変／Read trapにより旧approved-b12PASSを静的対応付けした。証拠はpacket/kh-c-context-bindings-20261016、今回runtime0。別実装者へdriverの5inputhash/HANDOFFだけを逐次所有で引き渡した。

作者OS2はpacket/8e9595fc-e238-4ff6-bf45-00d8f565af54で2PASS、child/shell exit0、finally込み0.6519982秒。source35AE/test0C82一致、実CIMself/子各1行を同一snapshotのままO1live→O2Exitedへ渡し、各ReadのIdentity1/Add0/PID-open0、作者wait1/kill0、cleanupFailureなし。独立証拠接合PASS／指摘0、D12Pure20PASSの同体再利用と旧結果保全も確認した。これは作者局所20+2の受領であり、独立runner/focused/主KHClosedではない。KH-Cのcontext3binding＋P1handoff追記だけを別作者へ渡した（runtime0）。

KH-TEST-AST01-P1/P2の最小是正は新test0C82D1C838F1EC59D85CE24667FADDB4C5EAD1A0E96C077B764E6BA0DCF5A092。AST4対照旧2RED2PASS→新4PASS／report3PASS、独立逆diffで旧D12全文一致（OS2箇所だけ）、Pure/accepted/時間/回収不変、parser/PSSA0を独立Postで確認し閉鎖可。report意味はHANDOFF記載でありJSONへの新meaning field追加ではない。Pure20はD12での結果を同体範囲へ対応付けるが、新testで実行済みとはしない。親は新OS2だけを新root・通常非adminで1回dispatchし、動的結果待ちである。主KH-REOPENとgate全体は未了。

KH-TEST-AST01の独立reviewはCHANGES_REQUIRED（初回）。P1はzeroarg ASTのArguments=null未考慮、P2はphase選択をosEvidence=trueと記録する分類であり、accepted=falseを成功へ読み替えない。規範契約/plan/design/sourceは変更不要と判断した。作者へOS条件のnull-safe化とPASS case≥1だけosEvidence=trueという2箇所の最小是正、実adapter抽出のAST4対照（旧2RED2PASS→新4PASS）と合成report3対照を依頼。15/10秒・回収・Pure本文/期待・新source35AEは不変、OS/Pure20の追試はまだ行わない。修正後独立差分確認・新SHA freeze・別実行dispatchを必要とし、同根因再差戻し時はtwo-strikeで局所追記を止める。

作者newPure20は20/20PASS、child/shell exit0（packet/3863c4bf-6104-45a2-be2c-95ea7fc17785）。OS1回は起動前AST adapterでO1FAIL/O2NotRun、exit1・0.1678467秒（packet/b7544ab3-3bf5-4055-99d9-f319fc4f0211）。TestD12:232の引数なしDispose ASTはArguments=nullであり、StrictModeの `.Count` 読取が失敗した。launch/CIM/registration/wait/kill0、cleanupFailure=null、source/test不変。実handleの元consumer成立は未検証、OS reportのosEvidence=trueはphase由来であり実測成功へ流用しない。新finding KH-TEST-AST01として独立read-only原因照合/最小是正Preへ返し、再試行・harness拡大は行わない。

OS追加testの最終SHA D12A5DB59689BEA912BFB6E93EE3739DA01B46D21483519CAD8790FF8834AB43、静的handoffはpacket/e7f4fdb2-efa1-4028-9cf7-59c16f69ebf5/HANDOFF-OS-STATIC.md。追加OS/旧Pure不変/Source35AE接合の独立staticはPASS／指摘0。実型consumer/単CIMself子snapshot/EOF終了/15秒同予算/finally同launchhandle回収、lifecycle0の間接証明と未達nullを確認した。親が受領し、作者newPure20→成功時のみOS2各1回を明示dispatchした。動的結果はまだ未取得、source/test変更・再試行・focused/80は未解除。

KH-B sourceの独立static reviewはPASS／確定指摘0（plan80B2/designAF67/新35AE/旧12E7/Pure840の現物一致）。Read前後の本文不変、既知完全照合・借用非Dispose・新規所有移譲、元Unprovable/fault/cancel/期限を確認した。新Pure20/作者OS2/OS freeze接合/独立runner/Postはまだ未取得であり、source実装のVerified/Closedを宣言しない。

KH-B実装候補はInvoke SHA35AE6C713ABCEAA28361DA750E7BD06F373E09EF2FB155D8456F79D83ADC66B5、Read-OwnedTree内57追加3削除だけで凍結した。parser/PSSA1.25診断・実行error0、旧Read本文への逆置換でBOM込み旧12E7全bytes一致、他consumer/fixed期待不変。証拠はpacket/6a6ee790-fa4a-496c-b8a4-205acca75e2f。新Pure/OS/runtimeは未実行、sourceのみの独立static reviewへ渡した。作者は別fileのOSphaseを準備中で未freeze、その新testはこのsource reviewの対象外である。

KH-Aはplan80B2FE76095C74711D7AA0E5DB1D469096F25C84B3CC5DE2B798FEC9C8444035へ限定確定し、designAF67のPreを維持した。独立作者の旧Pure20は保存旧copy（packet/360dac41-af21-4400-91fc-aacfff8f6d2c/Invoke-CargoGate.old.ps1）に対し1PASS／19RED／Unexpected0、exit1。P02はopen1→not-running／getIdentity0／Unprovable1／CleanupIncompleteで決定的RED、保持Knownは保存された。結果rootはpacket/4edbfada-b1f0-45d8-b48e-93d59e776a18、全caseのosQuery/kill/wait/terminal/borrowedDispose/runWideOpen/trap0。これは新source GREENではない。作者はPure期待/旧REDの凍結中、OS2はguardのみ未実装・未実行であり、全22件の完成ではない。

KH-A fresh独立設計PreはPASS／確定指摘0。requestはHEAD ed1ce4a・plan956C0CDA・designAF67CB6E・Invoke12E7、class設計/規約/reliability、Gate Pre。元Rows一意→UTC6桁候補一意→有効保持handle参照→未変更Get-Identityの全5field/UTC7key一致、借用非Dispose、既知失敗fallback禁止、新規旧経路、失敗後stateを支持した。Pure20とOS2、旧型AST0/新1、OS snapshotのself/合成子だけのfield不変抽出を分離して成立性確認済み。実コード・回帰・OS・focusedは未検証でありKH-REOPEN-01 Closedではない。plannerへKH-Aの限定精緻化を返した。

KH-A設計は既存design477〜、SHA AF67CB6E40B37813226317B68D0698BBD84586732FAE92D88A7982D978E223A2で凍結し、fresh独立Preへ提出した。唯一方式はRead-OwnedTree内で元Rows/台帳/保持Processを未変更Get-Identityへ渡す完全照合、newだけ元PID open。保存2PIDは完全identity、部分identityの補完・昇格はしない。Pure20＋実handle2の固定受入を分離し、実装・OS対照はまだ未実施。Pre後のplanner精緻化・束縛再結合前に旧driverを起動しない。

| Finding ID | 原文・位置 | Root cause | State | Packet | Canonical artifact | Evidence | Oracle | Reviewer record | Residual / out-of-scope |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FF-TEST-ORDER01 | test097:528〜542/563〜565のhash/case別counter | 回数検査だけでは前照合→case→後照合という交差順を証明できない | Closed | FIRST-FAILURE01 | Test-FirstFailure T15–17のみ | design653・旧097保全・新AD49・他本文不変 | 共通event順＋実predicate正順true/同counts誤順false | 独立限定Post PASS／指摘0 | 18件/OS実測/期限不変、18GREEN・最終接合は別判定 |
| OBS-FIRST-01 | run4de297d1のSupervisorがCleanupIncomplete cancel後に正式terminal/tmp欠落 | 初回failure観測がmemory/terminal依存で中断前に永続化されない | Closed（診断契約のみ） | FIRST-FAILURE01 | plan0CC4/design30F4→Invoke33C2/probe62DA/testAD49 | 旧証拠保全、旧T01RED/新18GREEN、suite6bb4409b/run62900800正式診断3/tmp0 | actor/run/source束縛・一回性・元error保持・whitelist・cancel前保存／dispatch1 | 独立FF-C＋限定Post PASS | ENV01はFAIL、OS欠測理由未確定、10/80/full credit0・主KH/ARGV/提供未了 |
| KH-TEST-AST01-P1 | zeroarg Dispose ASTのArguments.Count（TestD12:232） | 引数なしをnullで表すAST契約の未考慮 | Closed | KH-TEST-AST01 | Test-KnownHandle OS adapterのみ | 新0C82／旧D12・OS b7544ab3保全 | AST4旧2RED2PASS→新4PASS、作者OS2PASSも後続確認 | 独立PostPASS・指摘0、親受領済み | source35AE/Pure20不変、主KH/全体は別判定 |
| KH-TEST-AST01-P2 | 起動前失敗でもosEvidence=true（TestD12:456） | phase選択とPASS済みOS case証拠を混同 | Closed | KH-TEST-AST01 | 同report分類のみ | 新0C82・accepted=falseとcounter未観測を保持する旧失敗結果 | 合成0/1/2PASSのevidence/accepted3PASS | 独立PostPASS・指摘0、親受領済み | accepted/時間/回収条件不変、旧結果書換え不可 |
| KH-REOPEN-01 | 既知候補の確認より先にPIDを再openする（旧Invoke Read-OwnedTree:1270→1274） | snapshot後の終了で、既存保持handleを確認する前にGetProcessByIdが失敗する | Verified（局所作者/独立20+2、主gate未完了） | KNOWN-HANDLE01 | 既存gate-design477〜→Invoke Read-OwnedTree | 新35AE／旧12E7、保存run94e5f1c6・Owned/Unprovable/cancel/tmp。所有を計画指定で分離 | Pure20旧19RED1PASS→新20PASS、作者/独立OS2PASS、focusedは別の初因欠測でENV01停止 | KH-A Pre・source static・作者証拠接合・KH-C独立接合PASS | CXT-D/PTH-MONOの限定PASS維持、主KH/CXT-E/ARGV/80/full/提供は未了 |

独立診断を受領して既存plan244〜へKNOWN-HANDLE01の暫定4単位を追補した（SHA956C0CDA）。新planner/局所designerはAstra/high、既定作者/実装/reviewerもAstra/high、runner Sol6.1/medium。外部仕様の新規変更は非該当。設計・Pre・planner確定前に実装/runtimeを行わない。plan変更により旧test94/driver68の固定plan hashは不一致となり、再束縛までは実行不可。旧受入証拠は旧snapshotのまま保持する。

利用者「続けてください」とCXT-D独立受入を受領後、固定driver68EF44DD／HANDOFFA32C8F6Dでfocused10件を1回実行した。新suite `R/verification/68cfe879-a4ea-4665-83f0-8bff89cd2b2b`、run `94e5f1c6-43bb-4ca6-98c7-80eb443f76fa`。control2PASS、ENV01 FAIL「Unexpected gate failure」／残9NotRun、child/shell exit1、26.6723846秒、retry0、frozen16/16前後一致、sourceMatched=true。AP1 oracleのcasePASS後分岐は未到達、argvEvidence空を保持する。

RunOwner21556はCleanupIncomplete「Descendant generation or retained handle could not be proved」。processes13行はOwned12（全hasExited=true）＋Unprovable1、fixtureResiduals=[]、cleanupDeadlineUtc=2026-10-15T20:38:15.0399582Z。unknown PID23100はGetProcessById時not running、identity=null／killRequested=false／waited=false／hasExited=null、osObservationのsnapshotMatch1／knownMatch1／pathPresent/commandPresenttrue／rowFilter3値null。未知の終了・同世代・回収成功を推定しない。Supervisor cancel.reason=CleanupIncomplete→Runner Cancelledであり、Supervisor正式terminalとSafety list正式terminalは欠落（tmpのみ）である。target23492の完了はargv['::','--list']のG0列挙用fake、V2 List=true/Passed=true/ExitCode0であり、通常full test完了ではない。独立read-only診断へ渡し、後続・追試・修正は未実施。

### CXT01 context再構成（cycle 2、CXT-D受入PASS／後続は上記別項）

独立接合reviewはCXT-Dの固定file/context受入PASS／指摘0。C0のcontext/Run/payload、C9実junction拒否、C10 deadline=startedと元期限consumer、C11 Run.TEMPだけの変異と元一致consumer、意味的な12期待の不変、15現物hash/P1引継ぎ一致、禁止22/null/旧結果保全を確認した。親が受領し、CXT-Eへ進む技術的前提成立と判定する。次は現行driver68EF44DD／HANDOFFA32C8F6D・新規R/verification/<UUID>直下suiteで既定focused10件1回、firstFail停止である。今回の承認済み12件は終了しており、後続を実行済みとはしない。実OS所有/terminal/cleanup/native、CXT01/ARGV全体Closed、標準80/full/試用提供は未了。

修正版独立実行の新結果 `argv-path01/cxt-runner-20261016-approved-b` は12PASS／0Unexpected／0NotRun、配置guard2PASS／RED0、child/shell exit0、retry0、6.5264904秒（最大case0.8797314秒）。15入力hash/bytes前後一致、P1引継ぎ別途前後一致、禁止22counter全0。C9合成junction→元reparse拒否、C10期限拒否、C11context/Run TEMP不一致拒否まで到達した。旧approved-a結果を合算・改変しない。親も集計値を確認し、独立reviewへ接合を渡した。実OS親関係・所有監督・terminal/cleanup/native argvはこの12件の対象外であり、focused／80／製品full／試用提供は未実施である。

利用者が「再実行してよい」と回答した。修正版test94E4928C／driver68EF44DDを固定し、新しい合成結果rootで12件をsandbox外・非管理者通常ユーザーとして1回実行する許可を受領した。既存runner gpt-6.1-sol/mediumへ引き渡し、firstunexpected停止／retry0／各15秒・全120秒／禁止22を維持する。旧結果を上書きせず、後続focused／80／製品fullの解除とは区別する。

最終P1是正・束縛接合static reviewはPASS／新規指摘0。test94E4928C／P1引継ぎ43151F84、driver68EF44DD／HANDOFFA32C8F6Dを独立照合した。test1行／driver固定入力2行／HANDOFF追記だけ、期待/consumer/期限/firstFail/禁止22は不変。作者の凍結30件は現在28件一致＋承認済みdriver/HANDOFF更新2件で、旧FED3/822E copyも一致した。保存pure21の旧6FAIL15PASS→新21PASSを確認し、判定式のP1を是正済みとするが、実CXT-D受入は未達である。新12件のsandbox外通常ユーザー実行を非同期照会済み・未回答、旧許可を再試行へ流用せず、focused/80/full/提供は止める。

P1作者是正はtest述語1行だけ、新test SHA `94E4928C9454FA5BB08E64EC9A50069FE4A247B84131BA50EA60C378676E5494`／新 `CONTEXT-TEST-HANDOFF-P1.md` SHA `43151F84629CA518FF3003AEB0593B87451FA0F131D6FA603A88BA42015EE1AD`。専用prefixだけを `\A(?:TEST_ADAPTER:|FORBIDDEN:|TEST_BUDGET_EXCEEDED\z)` で区別する。基本演算の純粋21件は旧6FAIL/15PASS→新21PASS、parser/PSSA診断・実行error0、30凍結ファイル不変。無効Pester準備2回（構文/XML内部CIM、XML無し内部registry拒否）は回帰結果に算入しない。registry操作の実変更有無は不明、追加権限やOS変更は行わない。旧test/結果/HANDOFFは保存、新実12は未実施。別実装者へdriverのtest/handoff束縛だけを渡し、独立reviewと新実行承認を待つ。

承認済み独立実行は `argv-path01/cxt-runner-20261016-approved-a` で停止した。C0–8=9PASS／C9 UnexpectedFailure／C10,11NotRun、driver配置guard2PASS／RED0、retry0、14.5262183秒。C9はjunction作成と元Get-CanonicalPathの `Reparse paths are forbidden` 拒否へ到達したが、testの大小無視 `-notmatch 'TEST_ADAPTER|FORBIDDEN|TEST_BUDGET'` が通常文中のforbiddenにも一致し誤判定した。独立reviewでテスト作者責務のP1を確定、元consumer変更不要。15入力hash/bytes前後一致・禁止22counter全0・null保持。shell実測exit1とscript分岐exit2は区別し、独立取得していないchild exitを補わない。C9をPASSへ再集計せずCXT-D未達、後続0を維持する。最小是正と純粋文字列回帰だけを作者へ渡し、修正した12件の再実行には新たな許可が必要である。

2026-10-16、利用者が「実行してよい」と回答した。固定12試行だけのsandbox外・非管理者通常ユーザー実行を承認として受領した。対象testB779／driverFED3を変更せず、独立runner gpt-6.1-sol/mediumへ1回・firstunexpected停止・retry0で引き渡した。管理者実行、OS設定変更、実データ／資格情報／API、focused10／標準80／製品fullの実行許可へ拡張しない。

最終CXT-C独立Post静的PASS／blocking0。fresh reviewerはdriverFED3／HANDOFF822Eと固定13入力を照合し、直下UUID guardが最初のtop-level tokens代入より前にあること、元main書込前18文の限定reuse、ValidateOnlyのfixture/build/control/RunOwner前return、10case／literal17／既存oracle／firstFail／cleanup不変を確認した。git check-ignoreはread-only子プロセスを起動するため「全process起動0」とは表現しない。作者の旧9PASSは新候補のGREENではなく、dynamic Post／Closed／試用提供は未了である。回答前の追加runtimeを行わず、承認待ちで区切る。

固定context-testの独立静的Pre-for-rerun PASS：testB779/HANDOFF451451と15入力/保存証拠hash、4typedmocks/12matrix/禁止22/firstFail/null保持を確認しcodeblocking0、C9の自己作成directory衝突経路なし。ただし9PASS/C9環境阻害/2NotRunは作者結果、sandbox外成功保証・実行解除・Closedではない。

ユーザー回答前に進められるCの着手条件をplannerが設計471/473から確認し、作者12期待固定＋旧guardRED成立が条件で全12PASSは必須でないと判断した。D独立12/Efocusedは承認/C9実証/接合reviewまで保留する。別fresh実装者Astra/highがdriver/HANDOFFだけを逐次所有し、driverFED3C837A3B6F4B252AE6D4BE6604ED904817E26BA3872230687D64AB61655EE（208行net49）／HANDOFF822E1E8D257C8FAC2A997F5802F4C1A593AA36BB2DBF7498A7C58D2C3279BDDAをfinalfreezeした。元main書込前18文原文reuse、OutputRoot直下・sourcebinding・起動前前提を接合し、param/10case/17arg/firstFail/cleanupを保持、旧89C6/FB1byte保存、parser/PSSA1.25診断0/実行error0/whitespace0/13入力差分0。ValidateOnly/新12/focused/build/native実行0。独立静的Postへ提出し、動的GREEN/Closedは未判定である。

独立作者fixed Test-ContextPreflight.ps1 SHA B77954FCDE5CBA9FB182C259BF5792D461BE8D556E1E38F9917CD0E23B0250DE／CONTEXT-TEST-HANDOFF.md SHA4514510360DCD2DD94A09217BC1D27ABE102AB68736BF70C557DE2995495E037。最終12試行はC0–8=9PASS、C9NewItem junction AccessDeniedでconsumer未到達、C10/11NotRun、retry0。旧driverguard RED2（正常direct拒否／nested受理）は別集計。parser/PSSA errorwarning0、固定15入力不変、禁止22項目attempt0。初aのC2英語regexに依存したlocale adapter阻害は分離後、同failure種別/consumerをErrorId＋repository名＋到達で確認し、aを最終件数へ足さない。

C9はuse_default managed sandbox内AdminFalseで失敗した。F NTFSFixedReady、suite/target存在ReadOnlyFalse/ReparseFalse、親junctionEntries0、ACL継承保護False/7rule/Deny0はread-only確認、effectivePermission未評価。ErrorId NewItemCreateIOError、exceptiontype/HResultは未保存のためnull、直接原因がsandbox/ACL/FSのいずれか未確定。junctionは新suite内の初作成試行だが直前単体不存在観測はなく、停止後entry0のみ確認。設計stopに従い権限追加/代替/追試しない。ユーザーへ12件だけsandbox外・非admin通常ユーザー実行の許可を質問し、回答前runtimeは解除しない。固定testのfresh独立静的reviewを依頼した。12件受入／CXTClosedは未達。

再構成Pre最終PASS：designD1C38FB3557D067A5E0B414E6DAEA021123C66B64D6FB5C62930B271CC5EFADFの437～で4mocksの型/引数/出力数/副作用を確定し、Owners空・監督未実行を明示した。fresh reviewerは実currentprocessを渡すGetIdentity2箇所／AddOwner／fixtureAssertIdentityへ型維持到達可、元AssertControlのcancel/UTC/monoとfilehashschema維持、freshcase状態/出力stream保全を確認しPre PASS。CXT-PRE-01の設計不足は解消、CXT01本体のClosed/runtime成功ではない。plannerがplanE209を変更不要で確定し、fresh作者Astra/highへ新context回帰だけを割当した。次の別実装者はdriver/HANDOFFだけを逐次所有する。contenthash不変のcheckpointでPreを反復しない。

two-strike再構成：plannerのconsumer/rootmap再確認をplan E209293EC26AA95F0C88C0CFE795B397BEF0B9BF3EB5B94F3FED63F7289E84C4へ反映し、design82A4774AA899628BCBC4E821386E1F8FD7DB1950473C495390B3B673019A3D36の409～へ唯一context契約を作成した。実行suiteR/verification/<UUID>直下／資料argv-path01を分離、旧nestedcommand失効、12試行各15秒全120秒（同期I/O強制中断保証なし）／実RunOwner-native-target起動0とした。

fresh-context reviewer Astra/highのPreはCXT-PRE-01（P2阻止1）でCHANGES_REQUIRED。GetOsProcessはOSrow配列、GetIdentityは5field、AssertIdentityはvoid、AddOwnerはvoid＋Owners/handle登録であり、登録後AssertControlがRefresh/liveness/startを確認する。設計437の一括identity代用では型とOwners副作用が未決だった。designerへ4関数の型/出力/引数/状態更新と省略する監督境界の一意化を返した。試行追加は不要、元AssertControl/filehashschemaを変更しない。12試行は代表context負例として妥当、既存80・実OS回収の代替にしない区分、配置/sidecar/config/null/rollbackに追加blockingなし。Pre以前に作者/実装/runtimeを解除しない。

CXT01 Post静的PASS後の新独立focusedroot5babecf8-0678-4bd2-8bab-ae73c540c952はcontrol2PASS／ENV01FAIL／9NotRun。runab0c29bc-b969-4bea-82ca-1eddf01ab54aは実RunOwner起動後、verification/argv-path01/<UUID>/ENV01のdedicatedboundary外でCleanupIncomplete／sourceMatchedfalse、fixtureResidualsnull/processes未取得。known/unknown/heldhandle/cleanup証拠を0へ補わない。native/target/AP1argv0、現在CIM候補0はcleanupPASSではない。固定9入力不変、build反復/retry0。repository不足は解消したが、抽出contextの別前提が再び不足したため、同契約の連続阻害としてthird局所patchを禁止し正本context再構成へ戻した。plannerへ全consumer初期化/OutputRoot許可階層/fixtureowner/RunOwner拒否sidecar/cleanup契約のread-only再確認とnewcycle計画を依頼した。CXT01の最終Closedは未達、ARGV実動作も未達。

独立Pre PASS後、作者はdriver89C6E49B7D2930FE225DA0AB7C4E36B94B55CFBC522CEE35454F0BE17A2350FB（159行、net11）をfinalfreeze、旧E5FEをbytepreservedした。元main2文各1件/順序/sourcebindingと実consumer4path/hashをfixture前確認し、ValidateOnlyexit0/10case/variantguard/runtimefalse/outputdir未作成、parser/PSSA/whitespace0。HANDOFF FB1F80EB66BD82858821E449605FBB44B76641BCF9D2367158E55DDFEC153695に差分/依存map/検証logを固定した。production/期待/cleanup/既存80不変、新native/targetruntime0。Post静的へ提出し、実動作とClosedはまだ未了。

findingはdriverE5FEのsourcebound prefixから通常main repository初期化が漏れ、GetConfigSnapshot798で失敗したことである。native起動前・InvokeGateCase try1639前のfixture阻害であり、ARGVの実動作結果ではない。canonical ownerは作者のInvoke-ArgvPathFocusedSlash.ps1だけ、consumerはGetConfigSnapshot／NewCaseFixture／InvokeGateCase／ConfirmCaseResult。必要自由変数はGateScript／repository／fakeContract、PSScriptRootは実Testへのsourcebinding、自動変数とread-onlyUSERPROFILEは既存方式のままとする。

唯一の是正案は既存Testmain2097boundaryと2125repositoryの代入ASTを名前・個数で固定抽出し、prefix後／callback前に原文2文をsourcebound実行すること。main全体・newparam・host環境・期待値・case10・cleanup・nativeを変えない。入力は固定TestD8AD／driverE5FE／Invoke12E7／native381F・EXEFF7C、出力は実consumerへ渡せる同一context、抽出/値不一致はruntime前拒否・非PASSである。ValidateOnlyでもboundary/root/repository値、GateScriptSHA/fakeContractと実GetConfigSnapshot4pathをread-only確認し、originだけでは十分性としない。新永続状態/外部送信はない。

oracleは保存初回fixture阻害をRED代替せず初期化欠陥の証拠として保持し、旧snapshotで実consumer未到達、新snapshotでValidateOnly実consumer到達と後続focused実経路を分けて検証する。rollbackは保存E5FEへ当該driver差分だけ戻し旧不合格を解除しない。旧log/EXE/sourceを保持する。依存は独立Pre→作者限定修正・静的/実consumer確認→独立Post静的→runner新focusedであり、次も同根因差戻しなら3回目局所修正を止めcontext正本を再構成する。scope拡大／未知停止／環境変更は停止条件、製品判断pendingなし。

request HEADac92e29／branchcodex/development-agent-team／driverE5FE、class規約・reliability・operability、requestedPre。計画割当作者/reviewer Astrahigh／runner Sol6.1mediumは維持する。本finding/ARGV全体のClosedは動的実測と独立Post後だけとする。

cycle3解除後の独立build/rustfmtはexit0、新nativeEXE FF7C9A0F641F66F7C340AEC94C21EA9A0C745EA5A39C35B252C3A9CBDAFCCB88。focusedroot113c564c-05eb-487c-a200-d090169238e5はcontrol2PASS後、ENV01構成中repository未設定でexit1/raw0PASS1FAIL9NotRun。fixture失敗でありbehaviorRED／新nativeGREENに数えない。実native入口/target/AP1argv/AST検査0、現在CIM候補0だがgatecleanup未成立、固定入力不変/retry0。作者へsourceboundprefix/main初期化と同consumer自由変数のread-only診断・最小是正案を返し、再実行を止めた。ValidateOnlyのorigin到達はruntime文脈の十分性を証明していなかった。次のfixture是正は既存期待値を変えずPre判断後だけ行う。

runtime解除cycle2は作者freeze前の親dispatchでdriver4B253と現物E5FEが不一致となり失効、AP3同根因再差戻しには数えない。差は使用例コメント1行訂正のみ。作者最終freeze後、cycle3はdriverE5FEF838D266F37E50AD61ED51DC5B1919FECBE299D81C62774F9D54AF403387／HANDOFF97680F0337BD6F039B9DCE62E621DE7B9839E1FF42BD8C3FFC552D32863C3698の現物一致、parser/PSSA/whitespace0、ValidateOnly実10case/variantguard/origin/runtimefalseを確認し静的PASS。slash variantのみliteral a/bを追加し元16codeunits/orderと既存全比較/cleanup/firstFail/6負例を維持する。AP3は静的不足解消までで動的充足/Closedは未了。runner Sol6.1/mediumへ既存toolchain nativebuild→新EXEhash固定→focused10＋AP1oracleを解除した。標準80は局所Post後の別解除である。

runtime解除前の局所静的確認はAP3 explicit U+002F userarg coverage不足1件でCHANGES_REQUIRED。既存TRANS01の951–954には明示slashがなく、driver9をAP3完了へ拡大しない。他のnative2定数／固定9ID／firstFail／cleanup／証拠oracle結合にはblockingなし。作者へ元driverF026を固定保持した別versionでTRANS01 variant ARGV-PATH01-SLASHのFixture.argv末尾literal a/bを1個追加する独立対照を割当した。元16argsのcount/order/codeunits、既存ConfirmのUTF16exact／rawstreams／exit／cleanupを不変とし、既存80／TRANS01sourceは変更しない。findingはPlanned、追加test/hash/独立再レビュー後だけruntime解除する。

本体nativeは新381F5E1DCD70ED8040153FB155C8AABA0881FBFB57A8074B286FB4169D8683DD、14/17 ROOT/SCRIPTの2定数だけ、逆置換旧全文一致/rustfmtbeforeafterPASS。新EXEbuildはまだ未実施。保存REDoracle7D74D443968A951220404612D24E44D077A48B005248FC6C04EF12FA221533C9は11args8exact3mismatchと6ASTnegative拒否、run/scope/native/EXE/Intent/PIDstart結合。固定driverF026C65A95E20C0C4CA4863173546BD1A84113FCD0A687B1BAA2919745C41556はValidateOnly実source/case9到達、runtimefalse、parser/PSSA0。初回object表示不足はJSON化/実境界guardで是正し、合成PASSにしない。作者HANDOFF SHA CEB406D27BFE983083036A3EAA5C7EFB42491A39CD011BBC148AEAC841717CC0にcommand/証拠/不足を固定した。

| Finding／原因 | State／正本・owner | Oracle／Gate／残余 |
| --- | --- | --- |
| ENV01 Runner内部3pathの区切りがIntentとexact不一致。native ROOT/SCRIPTがforwardslash、既存join派生とPS GetFullPathの表現が違う | Pre-gate PASS、plan2A463D32の140～／design2123BB9Eの369～。native2定数producerだけ実装者、回帰は作者専用folder、validator/Intent/userpayload不変 | 保存REDの同run/scope/Intenthash/OScommand/PIDstart/nativehash結合を独立確認。AP1実11args exact／AP2拒否／AP3新nativeTRANS／AP4sourceEXEbindを独立GREEN後Postで確認。80／製品full／提供は残る |

HEADac92e29、plan SHA2A463D3260EC322D53B0D4815EB8EB1C0E103FCB75B29AA074455A7F8058CD44、design SHA2123BB9E88E616BF8455E7C6A63931D59DFC8CB1C3FBCAB4F134E09FCE9CCAB7。planner受領確定後、独立Preは2定数＋既存joinで内部3pathが揃うこと、利用者UTF16／genericexact／canonical拒否／unknown非停止不変を支持した。既存Invoke-GateCase＋sourcebound抽出でfocusedENV/TRANS可、80先行PASSの循環なし。作者へ新専用回帰と保存RED再利用を割当、code/runtimeはまだ解除しない。新native実測／command固定／独立Postは未了である。

## 観測Post・通常入口再実行と限定path読取設計（2026-10-15）

### PTH-MONO-01 是正packet（cycle 1、最終Post PASS／Closed）

最終独立Post PASSを受領し、PTH-MONO-01をClosedへ更新した。Invoke12E7への独立47/47＋5/5、専用child同handle回収、遅延成功／retry拒否／再進入非延長を確認済み。path補完も実OS同一handle取得を含む47件の範囲で限定Post支持である。argv拒否と関係4関数の333C/54AE/12E7本文一致は独立照合し期限接合と分離した。index4/8/10区切り詳細は親受領診断として区別する。標準80はENV01FAIL／79NotRun、実Cargo／製品full／OS同期中断保証は残る。下記Pre時点・自己検証時点の記録は履歴であり現状は本段落を正とする。

標準ENV01の別原因は保存dataの実parser ASTによるread-only照合で確定した。Runnerの11引数中index4(-File)/8(-RunPath)/10(-InvocationPath)だけIntent U+005C対OS U+002F（長さ90/120/173、差8/7/7）で、他8一致・引数数11一致。native176の固定forwardslashパスとPathBuf派生、Supervisor1655のGetFullPath backslashが食い違い、AssertObserved646のexact比較が拒否する。比較／Intent／Runner登録／Ready前検証は333C/54AE/12E7で本文同一、今回path/mono是正とは別の既存不整合である。全argv比較の緩和は不要であり、内部path構築表現を揃える別packet候補とする。現在packetの最終独立closureを依頼し、その後だけ新packet計画へ進む。

独立runnerの新root4161f2ea-445b-4f1c-8265-5bf48f322fdaは47/47、4fd51a22-4385-473b-b781-bc9a781c8254は5/5各exit0、専用child17160同handle回収。段階条件成立後の標準suite7f2c941c-d1b3-4b4e-a3cc-a71c627a6e2aはcontrol2PASS／ENV01実呼出1FAIL／残79NotRunで停止した。run7da565fe-32ab-42fb-b4c5-4eaf2d59a356のunknown0／Owned10全wait/exited／fixtureResiduals[]、Runner Rejected error『Observed command argv contradicts intent』、manifestConfirmed=true/g0Published、Runner取消後RunOwner/SafetyWorker取消、target0/Cargo1、LIFE14未到達。9SHA不変／現在CIM候補0を全体PASSに読み替えない。実装担当へ保存情報のみのargv/intent差分原因照合を返し、再実行を止めた。PTH-MONOの最終Closedは独立最終判定を残す。

cycle1 Pre-gate PASS：packet SHA414F3610739FBD8B19D2FF7EDDC46B49EDD98C4305C0F64E16DDC0C80D6EBFB9を固定した後、座標変換案はdouble丸め非延長が未証明で不採用。既存localclock/確定budgetのfunctionlocal window直接参照案は独立Pre範囲内許可を得た（非Private/noshadow、未存在だけ許容、他error伝播）。Invoke新SHA12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621の11追加だけで接合した。

自己検証の新5PASSと旧47PASSを別rootで取得、late/retry各Owned0/unknown1/API1、再進入API追加0、専用child13760同handle回収済み。parser/PSSA1.25診断0・実行エラー0、対象外逆復元一致。独立Post静的PASSは同一clock/budget/local/noshadow/missing-only/再進入非延長・80期待不変を確認し、固定47＋5→双方成功時のみ標準80firstFailを解除した。findingはVerified（自己検証＋独立静的支持）であり、最終Closedは独立runner／追跡更新後に留保する。runner Sol6.1/mediumへ新UUIDの段階実行を引き継いだ。製品full／同期API中断保証／提供は非解除。

Pre提出固定追加test：`9E9B0444E5E5FB4449D2B89417E2F09E8F7AFBE3AC7847FDF86AA7B0B7837404`、141行、証拠R/verification/image-path-contract-20261015-a/2189dce6-4de1-4171-81f6-06235b2af3ce。実Complete初期化prefix＋最初tryを抽出しactual Read-OwnedTree／Get-Identity／helper経路到達、同localclock/budget10を観測。5件3PASS2FAIL、UTC余裕・単調超過後successでOwned1/unknown0/API2、122後API8（期待Owned0/unknown>=1/API1）。normalと再進入残0API0×2はPASS。専用child40380同launchhandle保持・有限finally回収/exitedtrue、parser/PSSA0、旧47／Invoke／TestCargo不変。後段Kill/Waitは未実行。author HANDOFFにowner/consumer/state/rollbackを保持し、独立Preへ提出した。

| Finding／原文位置 | Root cause／State | Packet／正本／owner・consumer | Oracle／review／残余 |
| --- | --- | --- | --- |
| P1 PTH-MONO-01：通常回収の単調時計期限が新helperへ未接合（Invoke54AE 349、2225–2228） | 通常cleanupのlocal clock/budgetとhelperのscript上限が別系統。Closed | PTH-MONO-01、設計CE799395の346/PTH6を唯一の基準に維持。InvokeのComplete-OwnedTree→Read-OwnedTree→Get-Identity→Get-PinnedProcessImagePath。作者は新回帰だけ、実装はInvokeだけ | 実初期化5件3PASS2FAIL→新5PASS／独立5PASS、旧局所47独立PASS、最終Post PASS（Invoke12E7）。標準80の別argv拒否は分離、OS同期中断保証／80／製品fullは対象外 |

目的・範囲：同一の既存通常cleanup clock/budgetをhelperへ延長なしで接合する1Change Setである。新時計・新予算・署名変更・oracle変更・未知停止・製品data/資格情報/OS変更は除外する。入力は保持handleと既存期限、成功出力は期限内だけの完全path、失敗は元例外と既存Unprovable/CleanupIncomplete/非停止/非PASSであり、遅いpathはOwnedにしない。新永続状態はなく、途中失敗・再進入でも上限を再付与しない。

依存・停止：plannerは現物確認後、既存設計への不適合是正でdesigner追補不要と確定した。作者RED／独立Preの対象hash固定後だけ実装へ進む。scope拡張・同根因2回連続差戻しは局所追記を止め再構成する。rollbackは今回の局所差分を保存snapshotへ個別復元し、旧不合格snapshotをruntime解除しないこと。既存証拠・fixtureを保持し自動削除しない。

request：branch codex/development-agent-team、HEAD b360e9f、対象Invoke54AE／designCE799395、class reliability＋規約・運用、requested Pre、製品判断pendingなし。新回帰hash／self-checkは作者返却後に固定する。モデルは作者／実装／reviewer Astra/high、runner Sol6.1/medium。製品full・EXE提供・main mergeはこのpacketで完了しない。

独立局所PostはCHANGES_REQUIRED（P1阻止1）。Invoke54AE行349のCleanupMonotonicLimitは通常Complete-OwnedTree2225–2228のlocal clock/budgetに接続されず、初期Infinityが残る。有限値設定816はbootstrapだけ、Aborting時Assert-Controlもreturnするため、UTC後退等で単調上限後のRead再試行／遅い成功を拒否できない。設計346／PTH6違反であり、47件の期限変数直接設定では未接続を検出しなかった。他の同一handle／世代／null限定／8回／元例外／unknown保持／adapter不変にはblockingなし。現snapshotのrunner／80解除は不可。plannerへ既存通常budgetを延長なしでhelperに渡す最小是正と実初期化経路の独立REDを返した。

実装者はInvoke `54AE0BD10E437AC4AAFD46F72DB74659A2FFFF5AB284F0C82C0B68337B3BBD28`（79追加2削除、345–417/433–434/451–452/456行のみ）を返却、parser/PSSA1.25診断0・実行エラー0、対象外本文逆diff一致。初GREENはテストnative stubの別Add-Type assembly参照CS0400でresult未生成、動作REDに数えない。作者がstubと元Readを同一Add-Typeでcompileする3追加1削除だけを修正し、47期待値／元Read bodyの逆diff不変を確認した。新companion SHA13F27CEB2AF10F26A0B71F26D073F0DDCB22F6D75960B0114465E349431D0113。新UUIDで旧333C sourceは15PASS2FAIL30NotRun維持、新54AEは47PASS0FAIL0NotRun。実OS同一handleとmock retry/期限/bufferを区別する。独立局所Post静的を依頼し、runner再実測／標準80／LIFE14等／実Cargo／製品fullはまだ未実施。

独立作者はR/verification/image-path-contract-20261015-aの2script計498行を固定し、47件15PASS／2FAIL／30NotRunMissingBoundary（red-fixed/result SHA BF080B0C58270F1C31A68DAD34523E2FEAE604880FC3030BF8B4BFC90609D5C2）を返却した。Test-PinnedImagePath SHA A8B646E941C84F139B04BA338A62532A8D42A62974352081BEA6876C93D0B226、Test-ReadBoundaries SHA CE8742B4B75C1BFD902453283F10D8EC3B72B66FB057704288D74080B239BDCA。PTH1実CIM自身の同一handle＋null化は早期拒否を再現し、PTH4はunknown保全自体は成功だがhelpererror未到達でFAIL。helper未実装30件はREDに数えない。初回sandboxCIM拒否はfixture阻害、通常ユーザーsandbox外で最終RED取得、管理者ではない。parser/PSSA0、C#fixture compile成功。既存Invoke/Test/design不変、子process起動停止0。planner確定後の実装担当Astra/highへInvokeのみの限定修正を渡した。将来helper adapterの動的適合もGREEN前に必要であり、期待値を適合のために緩和しない。

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
