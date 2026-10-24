# UNKNOWN-DESCENDANT-OPEN01 診断追加・局所試験の成果物索引

## 最新：CAP-I採取本体の静的受入（2026-10-24）

C=`CAP/Invoke-PssaCapture.ps1` SHA39B5BC40CA83F5E088873B67047243182EF806D96BE8BC5D3372235808187613（43796bytes/816行/Parser0）。初回D862の出力祖先reparse拒否と独立64KiB失敗射影の不足を限定是正し、保存直前時計を接合した。独立code再Pre PASSである。beforeはCAP/before-cap-i-review-3a4bdc57-f7de-4f60-b698-c87d884be268にあり、harness.patch SHA0AC98BD1E95D9D970886A93DB0AC99835F55E3BF35E4309B0E639AE4E1418496（126141bytes）が旧CF775prefix117942bytesを保全し追加deltaから新Cを全bytes復元する。独立保存確認もPASS。

CAP/capture-binding.jsonとCAP/CAP-V-receipt.jsonはauthorが実C/callerへ最終接合中、候補履歴はCAP/capture-prep-c155bc1b-bbbf-4076-8b3d-6a88c0a5a7af。schemaはdesign末尾382～/398～（C6FB5947…）を正本とする。code受入は最終binding/実行Pre/freeze/20runtime/実PSSA合格を意味しない。

## 最新：C16静的受入・CAP-I実装中（2026-10-24）

C16=`CAP/c16-3bfd268a-99a4-4e95-965d-75d689905265`。Test6DD60794BCFDAD8C695D4CDD5274D6886E03452249C64A330E2D29832E3673BBとadapter-contract/c16-binding-pending/source-edits/static-source-proofが、実C Complete一回→driver exit原文→実caller停止分岐の接合を定義する。独立code Pre PASS、固定20期待/40keysは静的受入可能。現物hash/spanとruntime合格はPendingである。

保存patch188937502A943559B4C7A88B3B6FFBD6DE1B452FAAC1BDAE7B7832389EF50B54（3069068bytes）は独立保存確認PASS、旧B7E87prefix2947347bytes不変、新8diff全bytes一致。C16/recovery-index.json49BA03C5C00CBC4A84775B08F36109BB27359781D001360776513B4AC2F021FDが索引である。production bindingの不足はdesign末尾382～/398～（C6FB5947…）へ補足し独立Pre PASS。未完成Cのhashを最終入力へ束縛しない。C/harness.patchは実装者、Test/caller/tests.patchはauthorが別所有し、次はC実装確定と現物接合である。

## 最新：CAP-BF型別是正・6焦点＋旧RED受入（2026-10-24）

BF=`CAP/bf-44ec0b7c-a636-4d7f-996c-bc14cd616f55`、CAP完全pathは次節に記す。sourceDDAD638F…/harness5EFC0D6A…/receiptCE121B8C…を独立Preし、freeze5BC7619B…後に別runnerが固定request一回を実行した。焦点6件6PASSと特定旧RED1件の受入を独立Post PASS、1.170437秒/前後123照合不変/retry0である。元PSSA NRE解消とは区別する。

原結果はBF/out配下のfocused-result.json SHA81E9FD620BF325172E1C47CE86E60577326933FBB42E1C04283DF85EB195F1D0、unit-result.json SHA0AD30E33646031C83F53A258E6A66A0AA2481A796849BC43031D8160D2C620B6。BFのCAP-BF-RUNNER-*原5filesがrequest/toolreturn/実6件/元producer/特定assertion/前後hashを保持する。

結果追記patchはSHA B7E87C5FC6A41C2DBBEBC587DFC0AB0E012416C3E25ADE8DB75DEC65949AC080（2947347bytes）、旧55ACprefix2373431bytesを保全。BF/recovery-post-c2f44717-53ef-4fef-bcf8-1671b52014c8/result-recovery-index.json（09C253ACA0C1C41367B30F86778695A157B3BDCB1E8E7357323E4EAAEB99A1AF）が新原証拠8＋索引1を接合する。過去BD/旧REDの失敗を上書きせず、次C16期待・adapter是正は別artifactで行う。C/C16実動作/20GREEN/新PSSA/33/製品は未受入である。

## 最新：CAP-BD一回診断・独立Post（2026-10-24）

CAP=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d/pssa-capture`、BD=`CAP/bd-176ada9b-8956-467d-95f3-a9c385170a52`。BD新Test99305111…/receipt13840C9E…は設計・source・実行Pre済み、freeze D50BA652…とpath補足ECE3326C…後に一回診断を実施した。元exit2/Unexpected/accepted=false/0.9038672秒/前後84照合不変、診断Partialの独立Post PASSである。元REDや元PSSA NREの解消ではない。

原結果はBD/out/old-red-diag-0ece0ee8-bdd3-437f-9c59-827f1ec319bd/unit-result.json（4CEF4CE13746F9C9039EAF8E05A97460C3F9F85389B939371F3C5797CEC87CA2）、実request/toolreturn・前後pins・評価はBDのrunner PREP/RAWTOOLRETURN/EVIDENCE/ASSESSMENT/REPORTにある。完全hashは各原本とreview正本を参照する。位置194:34/stack195/PropertyNotFoundExceptionを保存し、failedEntryはUnidentifiedを維持した。

復元patchは`unknown-descendant-tests.patch` SHA A3C20E463574EEA4750036B33073C3F6A3F970156EEC730E907A3729BC300371（1949428bytes）。旧A574 prefix1748073bytesを保全し、BD/recovery-post-713abdc3-4427-4063-8725-83448f534586/result-recovery-index.json（BFB3A39CD387EC53ADD483FEEA74EEB6A8D6FA5743BBA4B0E2D1570394178C3C）が原証拠8件＋新索引1件への接合を記す。下記の旧hash/未実行表記は当時の保存点であり、現在の実行許可には使わない。次CAP-BF設計とC16/20/C/33は別の未完工程である。

## PSSA-CAP限定見直し：設計受入（2026-10-24、実行0）

精緻化計画`unknown-descendant-resume-plan.md`のSHA256はA8E1C248CD7DF19ADA805E72D11EBB6141A7E83B50AC521BEA0CCD4C74A686FD、独立是正Pre済み設計`unknown-descendant-design.md`は8295FF7BDCF1AB56CA465A0B6BB1FD86263BD77D9C8B478371D98A7E338CA3A8である。元Stop/全規則/全severity/engine必敗を維持する。初版20244の2阻害とbefore保全はreview正本にあり、実採取経路の原文抽出・最小shape/20照合先を補って受入れた。CAP-TはT/pssa-captureへ固定20期待と実装前の旧方式RED1件を準備する予定であり、生成済み/実行済みとは扱わない。外側中断責任/最終receipt/候補bindingは実行Preで未接合、33候補実行・元NullReference解消・製品提供は未解除である。

## U-N未完状態の保存点（2026-10-24、実行禁止）

T=`R/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d`。これはGitからの復元用であり、実行freeze/全体Pre/最終受入manifestではない。sourceF91C/AssertionsAFA/Controller8FAE/oracleCAA、before/diff・PSSA失敗2回・非gate診断1回を保存する。現receiptは旧Assertions1CAFpinで未接合、現manifest/reportは旧4Cの履歴であり**実行禁止**。最終binding/Pre/freeze/33実行/Postは未完である。

| 成果物 | SHA256 |
| --- | --- |
| tests.patch（1431105bytes） | 31CEF53E63E4425BF320E8BE317F7AAB72511864B8ABFDC4F379835E68922059 |
| T/save-only-state.json | EDC85366D4E856E9424EE25DBAB693ED7BF37BCAF19F5530DFD260BED2AAF88A |
| T/save-only-artifact-index.json | 1622AE050D4CD64D08BE4F839A0EC4C795284700B0D70993D134E434DA76D677 |
| T/save-only-restore-evidence.json | 44477F8340F8BC71EE8071DA2EB554FEEB73FA6E50BA35FFEFF59E8C0D65F83F |
| T/PreparationAssertions.psm1 | AFA82E807179DF852ECCB05D5A8C8F3919B9B1C4C9D981CFA0CD6F368FE824F4 |
| T/PreparationController.psm1 | 8FAECC46B8251D836EE8614655D557F3A4602C114D2D55FD3667876B2EAE134C |
| T/g6-evidence-static-blocker.json | 21B00027783A0F1FE2DE7D247584A5EC5DE24DCA183F4D9BA645A51F7DBE38E1 |
| T/g6-evidence-cause-diagnostic.json | 3D7D833A7C20A78EFDAF7D417327AE2AA1A20C20ABEF8DFCCEA99C66EB794137 |

保存証拠は旧prefix704378bytes/AAE7全bytesと旧66履歴を保持し、42新規/7標準変更を含む最終108fileをmemoryだけで復元、全bytes/BOM/CRLF/LF/NoNewline一致と記録する。FS復元write/追加PSSA/追加診断/runtime0。旧patchコピー4件はprefix長で回収するため、patchを再帰埋込みしない。原本CRLF由来のrawdiff分類は保存証拠を正本とし、検査規則の無効化や改行trimで消さない。

別reviewerのメモリ復元でも108/108 hash・旧66履歴length/hash・旧prefix/4旧patch回収が一致し、主要before/実失敗/診断と実行禁止/Pending表示を独立確認した。保存checkpoint阻害0であり、候補33の実行Pre・GREEN・全体完了へ転用しない。

## U-N合成原因識別の限定候補（2026-10-24、runtime0）

設計B45A320172B6158B39D92F8EB02917A46BB56FCA72B5DC1DD8B634E7CE9B13CEの独立Pre後、L2だけを修正した。最終packetは`R/verification/argv-path01/1c67fb6f-7f72-4394-a719-ab65188838af/static-3150fb42-d2b4-42cc-b48e-19655a502698`。source独立Pre/fixture再bindingは別確認であり、以下を実測GREENとは扱わない。

| 成果物 | SHA256 |
| --- | --- |
| canonical Invoke-FirstFailureEnv01.ps1（59886bytes/BOM） | F91C6B0453942EB13B5840D216A9547DA28394427F9F3F194AB8CD3324053F88 |
| core原文（25748bytes） | 6665470AF42C48015B1E01D85F9C9C706FBD4250ACF285828EE1AF5AA954CE21 |
| corrections-from-rejected.patch | CCA428FCE82F59CBAFF43A1A7E8138882FA67D46F961EBD1C38448DB386FF812 |
| implementation-report.json | F19C7455DC818E9DA886CA4DC2D557BB7A924AAB2FDAA95DE052CE528646B61F |
| inverse-4C-evidence.json | 9E652F756D2D526925D030C24EB973F129A05C295D773FFCD7DF370CE8B91791 |
| inverse-evidence.json | 4E0FB7208A625D937EA39B4CE15127D92B2C665FBF851597F5FEC110DD7A3BE1 |
| restore-evidence.json | 6297784CC71B327005C6F4300D7FB6452E2A1D39D85579F87D1A6A67948E1C1B |
| static-evidence.json | 2AD978D2A0211979FFC4A9893C608A2BDF4C2BBDA8E0AE80013EFDFACE4087F1 |
| harness.patch | CF62D0B20C30BE6F33836BC0C04F060469BACA64DA61475667DB0A6B08F34FC9 |

core offsetはBOM除去後ParseInputで5380～31128。旧4C全55809bytes/元19920bytesの逆一致・非L2差分0・Parser/PSSA通常0engine0・完全候補復元・旧harnessprefix保持は実装者静的証拠であり、独立レビューとruntimeを区別する。

## U-N最終fixture初回Pre差戻し（2026-10-24、実行0）

packetは`R/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d`である。以下は修正前の静的候補であり、実行承認・GREENの証拠ではない。期限確認とprimary原因接続の2blockingにより凍結を保留した。修正版は別hashで記録する。

| 成果物 | SHA256 |
| --- | --- |
| PreparationReceipt.tool.json | 88C19D0677C4651580F34CC9885C443D4E33676FF65A343FBB0F89D6D4DF3AE3 |
| Invoke-PreparationReceipt.ps1 | 19BDF700ADF871C1214D5865C5DD6A3AE2277B618CA363524826A6D3AE2A5A66 |
| manifest.json | 7F1F72B687E5905C6F732DF5D7E747D756785DC7A934E1BB0CA2AD5C3573E187 |
| preservation-bound-final.json | DA7449545264FDB0188CCB9CD9D47F6BAC54DEC9BDFDF8544FBE1C93B867C017 |
| tests.patch（初回最終Pre時点） | AAE7FEE9EED89B178FEDE64755E37E0711375339A7A661EBDC266D22EE582C12 |

core4C1612F8…/21671bytesは原文一致・置換0、18pins一致、66file復元一致、出力UUID57e30fae-cd20-4dcc-babf-a9e8521178c8は未使用であった。`final-static.json`は初回PSSA内部例外を含むため、別hostの最終Parser/PSSA通常0・engine0と区別して参照する。

2026-10-22／開始HEADba7f0e4。利用者承認は検証用診断追加と通常隔離内の純合成O1–O6である。新実OS ENV01/F3/native/full/製品buildは対象外である。実装・新試験の成果物は完成後に追記し、予定pathを生成済みとは扱わない。

## 開始baseline

R=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、H=`F:/Program Files/PowerShell/7`。親によるread-only照合9点は全一致した。

| Path | SHA256 |
| --- | --- |
| R/fake.rs | 3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016 |
| R/Invoke-CargoGate.ps1 | 898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71 |
| R/Test-CargoGate.ps1 | F0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919 |
| H/pwsh.exe | FA2C8BB1E35CDD62AD15CD06A1D8F2C5961E9725A9BFA1E7CFD2C322F49E9619 |
| H/pwsh.dll | A583A3721FEEBC4F9552A02358BC81BBDD41E2E18BB3F424226F2B1E0BF07DEB |
| H/System.Management.Automation.dll | 70701F20AFBBF4E763924C2BDD0C0E9A20EEC35A0E4EFF439E0CCF77F20D6B26 |
| H/pwsh.runtimeconfig.json | 8A6363FBA1F89C0688E3594EF4FE7B9036126F04957C5FE56858CBA020B76B33 |
| H/pwsh.deps.json | A01883AEEB2217AEB1D7EC944D649376104E7A874CA4EA71E7F36448C1519DC2 |
| H/ja/System.Management.Automation.resources.dll | F672B902371AD615A7AD211EE387B73C65386E7686610B6DCCA9A91927D8DC94 |

原本before、oracle、patch、最終command、freeze、結果の対応は各工程の完成後にここへ接合する。既存成功ログを新sourceの結果として流用しない。

## U-Eの作成領域

P=`R/verification/argv-path01/ef551043-d22a-44a7-903b-1e9540efc910`。独立作者がP/beforeへ原本2fileをCopy-Itemで保全し、編集0と報告した。親はfake.rs（14443bytes）とInvoke-CargoGate.ps1（154848bytes）の存在を確認した。oracle/新adapterは作成中であり、新candidateの未存在本文位置はPendingのまま、期待値固定後に対応づける。これは期待緩和ではなく、存在しないcandidateのhashを実装前に求める循環gateを避ける順序である。完成物の静的Preと最終freezeを省略しない。

## U-N準備probe候補・独立静的是正Pre（2026-10-24）

現source R/verification/argv-path01/Invoke-FirstFailureEnv01.ps1 SHA4C1612F8731DA936F32B69455694AE5129527D7143F5BD52797B7F8BEFB978BC/55809bytes/BOM保持。最終packet I=`R/verification/argv-path01/0f64870f-9da0-4c69-95fb-f139f911afe3`、証拠はI/static-4e27a260-717e-42a9-a358-c14ace045986/に限定する。元19920bytesへの許可6span逆一致/許可外diff0/PS7.6.7Parser0/PSSA1.25通常0engine0/完全候補newfilediff復元一致を確認した。

| I最終証拠 | SHA256 |
| --- | --- |
| inverse-evidence.json | E9F6EE1DF6CD59FB376C1596DC337CFA8A266D89583B0DB252BB225FCDD830F7 |
| static-evidence.json | 3B0B678EC43DF0772ADC861FACCE4699B2FE8A2D1715B21888415ED908391A6A |
| restore-evidence.json | 1EAF1DAA694B7844C2585D6605344C6FA88B2950FC64BF92520BA60017E2E6EE |
| candidate-full.patch | A5945CBE49892C6E4D588B6951EA3A9B4854D402C637998BDF19F1510A9AA6E1 |
| implementation-report.json | A27F1086372EEFECF5DFB5C6DCB5B1528AEBA7A53CC18BC583C0F8C816D2BF44 |

tracked harness.patch SHA473C3CC1746D350511C91D20790FC831974BF8EF9B18109AF84D0DADB7DFD3F4、preU-N083DD…prefix15734bytes不変、新candidate完全newfilediff1個だけである。旧811E/55505bytes候補と証拠はI/rejected/beforeへ保全し、3hunk差分corrections-from-rejected.patch SHA5A7884A54C92D33926DDD94874D7780B2420888A93C618AB3A817339DDD12F76を逆変換すると旧811E全bytes一致する。独立code reviewerは3blockingの解消/隣接境界追加指摘0として限定是正Pre PASS、33GREEN/実OS成功とは区別した。

初回実装packet ffcfb3e1-cee2-4d30-8dff-e52a2bd462a0/HANDOFF.mdとstatic-853c5a04…は旧候補811Eの証拠であり現物へ混用しない。Git解決先/不要async出力/サブdir filtering/longpathの静的補助失敗は保全した。復元時のcore.longpaths=true/core.autocrlf=falseは当該Gitコマンド引数だけで、永続Git/OS設定変更0。親は新packetにHANDOFF.mdがあると誤って一度読取失敗したが、正しいstatic dirの証拠を指定し直してhashを取得した。source/runtimeの再実行ではない。

33fixture packet T=`R/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d`。候補受領前の独立fixture固定証拠pre-candidate-static.json SHA990018786281A5B3ED0E80AE538F9729052A12B5C4831E78FD442ECE80B8EF4Bと旧811Ebinding53B7454E…を保全し、新4C16接合中。oracleCAA26281…不変、最終fixture Pre/固定receipt/実行/PostはPendingである。

## 承認済み自動fixture無効化版の旧RED成立（2026-10-24）

S=`R/verification/argv-path01/7f7369e7-0e52-4915-9d92-40cae9c941ab`。unit SHA A35FEB5ECEDDEEDC0B12697FFEB4534319D254927856B8375888A0DD177C86F0、receipt964B800F3F4A445501635E94644EDADE59232EC58882DB27C097A74D8F93335E、toolC1A5296C5E60172B0974C2770D4C35AF955A4657E8C860143FAC9130C21C2FB4、manifestBE39D039478D7BFD044199F9E6EF3FC1ED302D14C5BF7AA8B64C726A7CFA04E2。設定2行以外はpath/outputUUID/hashmetadataのみ、旧33期待CAA26281…とassertion/期限/分類は不変である。

tests.patch SHA4189475ABE0029502CED68260EE40C2556B05243E9531CE62931BC41136F228C、S/preservation-final.json F68C2BC5E34918D601C7565603FA0BA6AE93F4CD60C03000EFABB4814A65D554で全49復元一致/旧prefix525147bytes不変。独立差分Pre PASS、原本CRLF追加124行/不要末尾空白tab0の分類はこのSHAに限定する。

親freeze S/NO-AUTO-FIXTURES-FROZEN.md BF13B9478C40BF9D38F8D40003D0E355FB6D56086375A7831B4E21FC60AB699D。実出力S/e43f88b5-b204-45ee-82fb-a8a1e09f8445で、execution AC612467C8B48FB1614A8EE207E05B67E8D21D34841EF912DAF7180B3E286335、exit3B2FD40166E99A204C90AB52ED6F46966FAF8AFACEEA022D29458C49853EA3C3、stdout352B950A637E820C0A91C69CDE4477334C4D7AFB36C399A262247BE371125AC4、stderr空SHA E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855。report S/NO-AUTO-FIXTURES-RUNNER-REPORT.md SHAE31AA54899473C91F601EE89BE10EED731828DEDFE43F4BAD293B8E290DA635B。

一回/特定deadline assertionのBehaviorMismatch/native-receipt-tool1/accepted=true/18pin不変/正常保存/2.5646826秒/retry0を独立Postで受入れた。旧QのAdapterFailureと全証拠はhash不変で、成功へ読み替えない。旧版合成RED成立だけであり候補33GREEN/実OS/製品受入ではない。ここまでを保存点とし、以後のU-N候補は別証拠へ接合する。

## U-M準備経路の独立期待・旧RED準備（2026-10-24）

新packet Q=`R/verification/argv-path01/6201d237-1221-492b-8858-5c8f4c5f7d23`である。独立期待33入力/6群はQ/expectations.json SHA CAA26281E5EF932156118720997491AE1E3EB55ED9DF0365F32197C6F8979353、元probe0B6C…からAST7spanと逆bytes7/7を固定した。design9106C34A…の8ports exact protocolに接合し、候補private関数/GREEN33実行は未実装Pendingである。

最新：Pester.ps1の実load pin漏れを是正して18pins、独立是正Pre PASS後に旧RED一回を実行したが、registry fixture初期化の権限拒否でAdapterFailure2/accepted=false/RED未成立となった。最新receipt B354B474C91DE6B9536C83751D1EA8A4F232C03338DCC613EBF1DEC505507A31、manifest3DA4F0EA0BE205A9528721FCAF3B7EE868C83C4B843108C946041F75F7759776、patch931FA727BE9E895984403A0F423B2C9E88332810E6A78F89EB61B8CC0D56A093で、下記の旧値は是正前の保全記録である。Q/preservation-pester-pin.json SHA4A9004CDD1FF0D8E70237D13F604EA732BF91F14F1C91DAE9E96BE41226BCA29で39復元一致/旧prefix不変。独立reviewerは新patchもCRLF293行のrawdata/不要末尾空白tab0と分類した。

親freeze Q/OLD-VERSION-FROZEN.md SHA C70B911FEDB6F79D72FE7D85542193E9FA340B397AF655A3EC8598A0B1628DCE、出力root Q/8eac74a4-a484-41df-a1ff-08cfdf25aa28。execution SHA7BD296C96AF1BC2F5666A7E12E7432E769537180012C2FB8A1BD89CF08F2BE5A、exit SHA97C47C5AA7C49843D229599EE9D843E27A00C6F5F9DFC9F6805C019152A2484C、stdout SHA93E0714BBDDAD7B527A03C2F8B1CC0A5A3A84B34EED28C12CBE327B33BF3CFD3、stderr SHA4E05108237F8089CA08FA8557630D62695AEFE6B95DA322C36128180F484C994。Q/OLD-VERSION-RUNNER-REPORT.md SHADA656278287DFDF504B7B7D7D47F1589DA932BA46BF45F044493B59676FA623E、18前後pin一致/保存正常/2.6680518秒/retry0/attempt1である。設定変更・再試行はしていない。

旧version2statementのmemory境界RED固定入口Q/Old-VersionReceipt.tool.json SHA8B132CCC60B5CD4AE5687CF20EEF469B863B905AE0CC648E560AB373ACD3BCEE、receipt SHADD6A42CA9680D6DF089710F9FE8FF39BA53133481C33F74CB9B5B584B944FC5A、manifest SHA2F790E8D94BD21A8A447A00AF2847A62A79C54D12E424E715E1139F65C63EE88。特定assertionだけnative1、adapter/fixture/parse/timeoutは2として分離する設計で、独立Pre/親freeze前は未実行である。PS4fileは作者確認Parser0/PSSA通常0/engine0。

tests.patchは旧8F0DC52F…の先頭445885bytesを保全し、新13fileを追補した。最終SHA986A98662ED2A1E1538A673FF607190EDD06FB1C184C2FD8E70CDFE7E7204E52、Q/preservation-final.json SHA583B180EAE36B72B8AC6EF455AA2E69DAA3EDF7B87D501BF6860520C00099E76で全39復元bytes/SHA/BOM一致・旧26不変を確認した。初回の深い復元先でのlongpath、3JSONのNo-final-newline/CRLF metadata不足、最後の余分CRは元結果保全の上patch形式だけ是正し、原本/期待不変・BehaviorRED0である。親の読取整理でもforeachpipe構文エラー1件が起動前に発生したがファイル変更0であり、その後読取のみ再採取した。新ignore例外/force-add/config変更0。

## 新候補6対照の実行証拠（2026-10-22）

Postでtests.patchのnew-ps-unit.ps1先頭のBOM欠落だけを検出した。実行時patch554F…はP/tests-before-bom-remediation-554F.patchへ保全した。作者はbyte offset191230にEF BB BF3bytesだけを挿入し、新patch SHA8F0DC52FD593EF4B37843CD3C855D00052A278E3BB2EDD2BB5C67E085556B7AE、26/26file復元全bytes一致とした。是正報告P/NEW-PATCH-BOM-REMEDIATION.md SHA2C553ED6F61855A3104C64E9B8DE0AACF4CE4303AF911788621830B39F4E9420。下表554F…は実行時固定値の歴史であり、現在の配布patchとは区別する。独立是正Post待ちで、本体/unit/期待/receipt/実結果変更・再実行0。

Pは冒頭のef551043 packetである。旧Pending記録は旧段階の状態として保持し、本節を最新索引とする。

最終：新patch8F0DC52F…の3bytes限定差分と26/26復元一致を独立確認し、限定Post PASS。下記の「Post待ち」は当時の経緯として保持する。診断追加と局所検証の受入完了であり、UNKNOWN原因/実OS/製品の未確認は維持する。

| 成果物 | SHA256 |
| --- | --- |
| P/new-binding.json | D2211A88897BF595C4F56B2EC8866FCF30347E2588EC186AE6CA40BE40577E81 |
| P/new-manifest.json | CE511B7393FE4B25D1B8E5DB95D82CE0B168D30D99652D25A58A28E0B4A6CA17 |
| P/new-compile.receipt.json | 14AC69BDA4E4AD57A8C26A9D380B957EC6C44D536AEAB378611BE02D275F43A0 |
| P/new-suite.receipt.json | D674693745D18D74729567FA685AC818F63588BD5EB0E171B0791CC76B3760D3 |
| P/new-final-static.json | 563FC2C7E942D4B3486C868019DB42C87B222B7F767DD5D87D2EB70552CE9241 |
| P/NEW-O1-O6-FROZEN.md | AC067D559494E97BFB58EFEDA829A32A2C1AA39B6B2CB8AF3DEE4F2A3B6428D4 |
| P/NEW-O1-O6-RUNNER-REPORT.md | 4D835838B10B10DEA235A9C1FB8C63C93DE436DB5E585E9D0C081A4B0FF2FFEB |
| tests restore patch（新unit/receiptを含む） | 554F2E9291334981224F0B95056F50516E42B115C04B6FE35E724301EF36A75C |

compile root=P/720684ef-1dcc-4261-94b3-c1b18955d06b、execution SHA41C1487A8F312FFD8834871B588F4499B64CB229B012CC5EA8576EAC0D04F0C0。suite root=P/77c19c1c-adef-42af-8c6f-e0ecafefb73c、execution SHA BF35FD8C6D386B6F49D6236643080AB457E84CD86D886113BE572713227ED149、suite-result SHA835917AB2CC91623ACAD2339FBCD786E6124E6D3A3BA3BFB536C712EAE8552BC。binary SHA F5FDC64D318D76C6C166E78594C3C633E5BD7D16438839C4D9D87FC60B78F4DF。完全argvと27pinsは固定receiptとexecution、stream別SHAと実runtimeはrunner reportにある。

別runnerはcompile→suite各一回/tool-native exit0/accepted=true、27pins前後一致、6case/51inputs PASS（3/2/9/14/5/18）、failed/skip/retry0を確認した。suite内部5.1353305秒、外側5.617467秒、各case5秒/集合30秒以内。PS実7.6.7/指定exe-SMA/非admin一致。compile未使用変数warning4は保持し、抑制しない。独立Post待ちであり、実OS UNKNOWN原因確定や製品受入とはしない。実OS/旧F3/full/build/製品変更0。

## U-E独立期待と旧O1実測

expectations.json A1914AD22D864A30FEFEA90801F349F152EB23177E92E75774863F880047FAD4は6case/51inputs（3/2/9/14/5/18）。baseline14spanと旧O1だけ完成し、新candidate/O2–O6 unitはPendingである。manifest F5793B16E37A239FB07F6655F16956499A1C19ED3DE7DACB3D04F2D40B1E36FDはこの未完了を保持する。

旧O1 receiptはcompile29635D20F81D54177954FF2A6EEDA92E880138E7922CF294E3E7365FDE87ED2C、run D3B378EF4B71D74879A974D1804EF2ABDB83089A1653D8CCE8F8ED7A5D6C0A80。Parser/PSSA1.25通常診断0/engine0と独立Pre後、親freeze P/OLD-O1-FROZEN.md A7924C12208D5FB0532698FCF83FFFAF16D0F27F6F6BF2C1B68F56204B413323を外側照合し、別runnerが通常sandboxで各一回無加工実行した。

- compile root P/616fa13d-d949-4462-bfc6-0df66c902b6a。tool/native exit0、CompilePASS、execution SHA6D5847D91A5687B4825EE9F19DC13DDF74D6DBDB1AAF2140D39E3DC55511A2BF。
- run root P/d3a00d97-8a53-4c73-a0d9-cc8f9fa3df7f。tool/native exit1、exact BehaviorRED3/3、stderr空、0.0908689秒、execution SHA DE96A4BFB9F2236D852C17F6C4AE518EA11C238BEB6138361725E8B1C95BB70F。
- 両executionはaccepted=true/CreateNew保存、16固定入力前後一致。retry/kill/新case0。report P/OLD-O1-RUNNER-REPORT.md SHA021C234CE8A08DACC5CB38B6D8D2BFBF5D1FA06955AA03AE0027C7E38346870D。report文書だけは上位の編集制約に従うapply_patch新規保存で、実行証跡のCreateNewと区別する。

初回staticはBOM offset9失敗、receipt初回PSSAは未使用宣言2件で停止し、それぞれ元結果を保全した。原文/期待/許可置換は変更せずmetadataと未使用宣言2行だけを補正した。これらをBehaviorREDに算入しない。実行native2と1の識別不足も実行前に固定receiptで解消し、未承認の再試行は0。現在のtests復元patch SHA BF7B5483968C0F05D7F701DEC5D967C2C377F42E8754500B0B8F24BA6B62CB1Bであり、候補束縛後の新差分は別hashで接合する。

## U-F診断本体候補（局所実測前）

実装者指定Sol6.1/high、所有はR/fake.rsとR/Invoke-CargoGate.ps1の診断spanだけである。候補fake07A47648E47FB57F5D0969650BBCEAF138671A4FC151EF6AA1E33020DCF2DCC4（18319bytes）、Invoke3B784E9B7CAE98261D0A496C0E229C4E2CB39120962656BF2186049A08CCC4D3（159294bytes）。Parser0/PSSA1.25通常診断0/engine0、PS static host7.6.7、許可span外0、forward/reverse全bytes一致2file、元target body/13fault/main不変を実装者が確認した。Rustcompile/runtime成立を証明する結果ではない。

P/implementation-static-evidence.json FE4E890062E236176F76C7FD818D8188E9DED8244ED7C4CFC9FFB365A0448D2A、P/implementation-report.md 4225F45349294EF791FB9F8543DAD9979F59E2AD55FF3CBC6D9B338BD8F85B80、tasks/ACOUSTIC-HISTORY-001/unknown-descendant-harness.patch 083DD2E6016FDC0FA319FF9C07FE23FCB90BC91A5A0CD23E76EB9A3CC4F08D0F。初回static session結果の未取得と再採取1回、照合器のList/複数diff境界不備attempt1/2は保全され、本体/期待を変更せず照合器だけ修正した。独立作者が候補束縛中で、最終Pre/freeze/6対照/Postはまだ未完了である。

保存方針の未決：既存.gitignore25–26行はtasks配下のdiff/patchを対象外とする。親の通常git addはpatchで拒否されたためforce-addや設定変更をせず、2つの再現patchだけを個別例外にしてよいか利用者へ非blockingで確認した。返答まではpatchをlocal保持し、Git保存済みと称さない。検証自体は継続する。stagedの文書3件は保持し、無関係編集を戻していない。

利用者がこの2patchだけの例外を明示承認したため、親は.gitignoreへexact pathの否定rule2行だけを追加した。無関係なtasks配下patchは引き続き除外されることをgit check-ignoreで確認した。force-addは0である。source候補の独立局所受入前という状態を保持し、保存点は受入完了と呼ばない。

U-E後半でO5-diagnosticfail一件の純合成seamが不足すると作者が報告した。既存許可adapterだけではacquisition構築失敗→key省略を作れないため、期待を変更せずその一件をdesignerへ戻した。残り50入力の準備と設計文書は所有が異なり独立するため、親はこの間だけ作者＋designerの限定parallel（親を含め3slot）を許可した。未許可assignment置換・新OS障害・意図的メモリ枯渇は0、seam差分の独立Pre後に接合する。
