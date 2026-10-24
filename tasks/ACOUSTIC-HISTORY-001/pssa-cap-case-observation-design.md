# PSSA-CAP：共通casecatchの限定失敗観測

担当は既指定designer `gpt-6.1-sol/high`、使用skillはapi-and-interface-designである。CAPは既定T/pssa-capture。本設計は観測契約と許可diffだけであり、source/fixture/PSSA実行・再委任0である。入力はCAP/savehash-770a0e9c-2a45-405e-b7ce-963a86d5814f/Test-PssaCapture.ps1、SHA256 `C357FCEDA8B05B9DFDA1EAB0D944FF205CE6D65F59533F2C49ABBE5379ADC287`。旧Save一行是正を保持する。C39B5候補・oracle `0E20118B8C11714B21DEDE0A1AEC66EAA928FAFF1D845B83DB04847EAB7062C0`・20期待・全規則/severity/閾値・C07型を変更しない。

## 根拠・完了境界

既loop705–740は最初のUnexpectedで停止し、catch736–739がid/status/errorしか残さない。新C01 PASS後のC02 expected ordinary不一致には実値/primaryがなく、原因は未確定である。外側catch808以降の固定scalar・欠測/切詰め分離・一回保存を再利用するが、外側観測をcaseの実値とみなさない。旧C01診断のName引数付きgetterはproperty名に遮蔽されmemory.filesのpathが無効であったため、そのcallback方式を再利用しない。

| 許可span／責任 | 最小契約 |
| --- | --- |
| 新Testコピーのloop | 実行済みC357原本/旧FAIL/Partial/receipt/binding/patchを全bytes/hash保全し、新artifactだけを変更する。各非NotRun caseの既caseStart取得直後に`actual=$null; fixture=$null; actualObtained=$false; fixtureObtained=$false`をcaseローカルで初期化し、New-CapFixtureの正常代入直後だけfixtureObtained=true、既`$actual=@(Invoke-CapFixture $fixture)`の正常代入直後だけactualObtained=trueとする。前case値/関数scope残値を参照しない。元call/代入/期待/成功経路は移動・置換しない |
| 共通casecatchだけ | 冒頭一statementで`$caseFailure=$_`を同じErrorRecord参照のまま保持する。元failed=trueとrows.Addのid/status='Unexpected'/error=元caseFailure.Exception.Messageは維持し、同じrowへfailureObservationだけを加える。元error/primaryを観測errorで置換・再throw0。後続NotRun/accepted=false/requestedExit2/retry0・既時計・制御順は不変、別CLI/個別C02経路/関数signature/global変更0 |
| 取得方式 | 新helper/module/Name付きcallback/汎用property探索器0。catch内の明示した固定field読取statementを各try/catchで囲む。hashtableはContainsKey/index、既知ErrorRecord/Exceptionは型確認後の明示getter、collectionは既知Count/indexだけを使い、任意objectのToString/全property列挙0。実objectは変更・再配置・wrapせず、観測用scalar/新配列だけへコピーする。未取得対象をgetterで生成しない |

## failureObservationの固定shape

root keyは`fixtureObtained:bool,actualObtained:bool,actualCount:Int32又はnull,caseError:object,actual:object又はnull,state:object又はnull,recordRefs:object又はnull,save:object又はnull,caseElapsedSeconds:Double,collectionElapsedSeconds:Double,missing:string[],truncated:string[],captureFailure:object又はnull`だけである。actualはactualObtained=true・Count>0の場合にindex0を要約するが、actualCount!=1も事実として残し、成功扱いしない。stateはfixtureObtained=trueのfixture.stateだけから別欄で採り、actualとの優先置換/混合0である。

| 固定対象 | field/type・上限 |
| --- | --- |
| caseError／primaryError | caseErrorは保存済みcaseFailure、各summaryのprimaryErrorはその元ErrorRecord又はnullから`recordType,exceptionType,fullyQualifiedErrorId,category,message,position,scriptStack:string又はnull`、`hResult:Int32又はnull`を射影する。category=CategoryInfo.Category、message=Exception.Message、position=InvocationInfo.PositionMessage、scriptStack=ScriptStackTrace。nullのprimaryは正当な観測値として`primaryError=null`と記録し、caseErrorで埋めない。WrongType/getter失敗は当該field欠測にする |
| actual/stateの各summary | `type:string,gateStatus,captureStatus:string又はnull,flags:object,counts:object,rawCount:Int32又はnull,firstRaw:object又はnull,primaryError:object又はnull,missing:string[],artifacts:object[]`。flagsはmoduleReady/invokeStarted/invokeReturned/captureComplete/postBindingsMatched/savedWithinDeadlineの6bool又はnull、countsはordinary/engineOccurrences/engineGroups/unknown/signal/auxiliaryの6Int32又はnullだけ。actual.rawRecords、state.recordsのCountとindex0の`kind,origin,type:string又はnull`を採る。stateに元来ないgateStatus/captureStatusはnull＋Missingとし、推算しない。元missingは先頭16件・各256字、artifactは先頭2件の`kind,path,sha256,status:string又はnull,length:Int64又はnull`だけ |
| recordRefs | fixture.state.recordRefsの`count:Int32,firstType:string又はnull`だけ。index0がnullならnullを保持、非nullなら実GetType().FullNameを採り、raw.typeから推定しない。生record/DiagnosticRecord/参照をJSONへ渡さない |
| save | fixture.memoryから`callsCount,filesCount,partialLength:Int32又はnull,calls:object[],files:object[]`だけ。callsは先頭2件の`path:string又はnull,byteLength:Int32又はnull,tick:Int64又はnull`。filesはstate.outputRootに既知raw.json/result.jsonを結合した2pathだけを辞書のContainsKey/indexで照合し`path:string,exists:bool,length:Int32又はnull`を記録する。memory内byte内容/全辞書/新hash計算/ファイル読取・再保存0。callback引数Nameや任意key列挙に依存しない |
| 欠測／切詰め／観測故障 | 全stringは最大8192 UTF-16字、ただしpath/type/status/FQID/category/hash/field名は1024字、missing/truncated項目は256字。超過はprefix保持＋truncatedへ固定field位置を追記する。root missingは最大64件の`field:NotObtained/Null/Missing/WrongType/GetterFailed/ByteLimit/Deadline`、truncatedも64件、超過は最後の枠を固定Overflowとする。NotObtainedと取得したnullを区別し、先頭件だけのsummaryを全件取得と表示しない。captureFailureは観測側最初のerrorの`type,FQID:string(各256字),hResult:Int32又はnull`だけ、元caseErrorとは別欄で保持する |
| 観測全体上限 | scalarだけのrootをConvertTo-Json -Depth 12 -Compress→UTF-8として65536bytes以内にする。超過時はactual/state各primaryErrorのscriptStack→position→message、次にactual/stateのmissing→artifacts、次にsave.calls→save.filesの順にnull/空配列へ落とし、root missingへByteLimitを記録して再判定する。なお超過又は射影全体故障なら、obtained bool/actualCount/経過秒/caseErrorの全field=null/他object=null/固定missing=['Observation:ByteLimit'又は'Observation:GetterFailed']/truncated=[]/固定captureFailureだけの短い同rootへ退避する。再帰的live object JSON/JSON丸dump0 |

観測の前・各上表block後・JSON化後に元caseStart/startTickで5秒/30秒を確認し、境界一致/超過なら部分rootを捨て、既取得fixtureObtained/actualObtainedのbool、既取得actualCount（未取得ならnull）、当該時計確認で得た経過秒と既取得captureFailureのbounded値だけを残す固定短形へ直ちに退避する。rootのkeyと型はByteLimit/GetterFailed短形と同一、caseErrorは全field=null、actual/state/recordRefs/save=null、missing=['Observation:Deadline']、truncated=[]とし、captureFailureは既取得の最初の観測errorのbounded値があればそれを保持し、未取得時だけ{type:'Deadline',FQID:'Observation:Deadline',hResult:null}を設定する。この有限scalar＋上限済み文字列だけの短形は構造上65536bytes未満であり、部分rootの大きさに依存しない。Deadline検出後は追加getter・観測JSON再計測・追加I/Oを一切行わない（既存report一回直列化/CreateNew保存の経路は変更0）。時計reset/新余裕/期限後I/Oで補完0。経過秒は取得時snapshotであり、保存完了やhard停止の証明ではない。catch既Unexpectedは期限超過でもPASSに変わらない。観測は既unit-result.jsonの既存一回CreateNew保存に同梱し、save/catch/exit側の変更0、観測を含む全経路は既5秒/集合30秒の評価対象である。

独立Preは上記初期化/正常代入後flag/catch内観測/row追加だけを許可diffとし、逆置換でC357全bytes/BOM一致、Save一行修正維持、oracle/C/producer/C07/特定assertion/元error参照/停止/保存経路不変を照合する。新実行は別のbinding/実行Pre・親freeze後の新slotだけであり、本設計は解除しない。元C02失敗の再現と観測取得Complete/Partial/Unavailableは別評価とし、診断取得を20PASS/元PSSA NRE解消へ読み替えない。未取得runtime・実provider/usage値は未取得である。
