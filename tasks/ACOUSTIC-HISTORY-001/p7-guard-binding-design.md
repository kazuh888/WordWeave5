# P7-D4-GUARD 限定設計

2026-10-18 / ACOUSTIC-HISTORY-001 / G1。状態：独立Pre提出、実装・runtime未解除。planner指定は `gpt-6-astra/high`、provider返却model/effortは未取得である。親HEADは `fea71cb96053b31c069f3734249b1306bfcdf6f6`。所有は本書だけである。

入力正本は [限定計画](p7-guard-binding-plan.md) SHA `835C7A749F9D501142966C8D75DE30E987406223875459382FB5123E241BCFCA` である。利用者の限定設計・独立レビュー後の修正検証承認と、親から受領した既存仕様P7-S01–05の承認を前提とする。凍結 [main plan](early-trial-gate-plan.md) SHA `FF96F8DE80D3E7602F85F7846AF3C8C4C45AB1FF7A4B3F0806AE6A3DCEF817D2`、[main design](early-trial-gate-design.md) SHA `6F84598858766BA08CFC236B8B847598705BDDD3A3000F4F1AB9BA07938236A7` は変更しない。本書の優先範囲は計画GB-R1–R3の追加guard接合と最大3局所観測だけである。

AGENTS、development-team、KPI、`agent-skills:api-and-interface-design`、`powershell-review`、その基準である `powershell-automation` と版互換・coding/error・checklist/anti-pattern・test coverage資料を読了した。適用範囲は境界、例外伝播、静的同定、証拠保存の設計であり、全PowerShellコードのレビューPASSを主張しない。Targetは既存計画のps7、固定7.6.7である。UI/日本語描画/ボタン配置/縮尺/狭幅、REST、製品永続schemaは変更がなく非該当である。一般skillのmodule/config/Pester新設、BOM一括変更は採用せず、利用者承認済みの単一局所script・既存bytes・既存品質gateを維持する。

## 1. 証拠鎖と追加1literal

以下で `R=F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、`P=R/verification/argv-path01`、`M=P/a1e2d0c5-5d2f-4d6e-97a2-0b6dea11cfea`、`S=P/35186079-b5d6-4e51-b19e-a608b67dadc9` とする。短縮名は表示用であり、照合には完全SHAを使う。

| path | SHA256 / 根拠 |
| --- | --- |
| `P/Test-FirstFailure.ps1`（未編集） | `AD492926E4DC4CA84766710D41CFF9024065A88D79BAF310226B96026DAECD46` |
| `M/Invoke-CargoGate.old.ps1` | `33C21282E728E3F654C727286F6AE462B0BF4A5D48E4A56A009E0D42A22FE38C` |
| `M/Invoke-CargoGate.frozen.ps1` | `318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3A` |
| `R/Invoke-CargoGate.ps1` | `898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71` |
| `M/REPORT.md` | `989572E9EE4B8375DCC52C14B256755929D50C5C73B3192B984CA9477859C2FD`。MI-C実装・静的報告であり単独では承認記録でない |
| `M/inverse-proof.json` | `1DD06FEEAAB5C135119D38984D092FA4F6A09089749D2C578EC22148B00FC12D`。許可区間逆適用後旧33全文一致、prefix/suffix一致 |
| `M/ast-proof.json` | `1DD997D842BB3A34D8A54043ABCE6376C9B48D0BE9AB6328FE049A5B818126FF`。担当外67関数・前後guardの保全証拠 |
| `S/REPORT.md` | `C1C255DEE6D5688691D5F8903C2CE28771323D2A7127A259027EEB2813E0013A`。6literal候補、追加必須validator停止、元FF保全 |
| [承認記録](early-trial-delivery-review.md)（読取時） | `811BFEC050ECA27D06E16497B30500F9E3CB45C7DC4525CFF6E03D7BA0A64128`。173行の「MI-C新Invoke318E…を凍結」段落が独立source static/test接合PASS・許可区間逆適用・親受領を示す。171行MI-D局所Post段落が独立MissingOs2＋MissingPure20を示す |

承認記録は親が更新する索引である。G2作者は上記読取時snapshotと該当2段落を新証拠へ非上書き保存し、親の最終manifestが由来として束縛する。後日の索引更新を理由に未承認現hashへ追従しない。旧/新MI保存sourceと既存承認は再実行で置換しない。

G1の限定read-only再確認では旧/MI後/現行のParser errorは各0、Read-OwnedTreeは各一意、診断callは各2であった。関数のUTF-16 offsetは旧 `[77288,87315)`、MI後/現行 `[77288,93534)`、行は旧1418–1568、MI後/現行1418–1646である。MI後と現行の関数全文はOrdinal一致であり、7.6.7版literalはこの関数にない。MI変更の承認区間は新 `[81049,87671)`、旧 `[81049,81452)`、現行1477–1559行に対応する。

元FF190–242行の `Assert-SourcePreserved` が唯一のconsumerである。Read-OwnedTreeを一意抽出し、Invoke-GateFailureが2call、各CommandElementsが7、追加2名前がCallsite/FailureObservation、分類が順にOwnedTree.IdentityCatch/OwnedTree.RunBoundParentEdge、最後のUnprovableだけを参照することを検査する。各callを逆順に先頭3要素のExtent.Textをspace結合したtextへ置換し、関数全文のUTF-8 SHA256を照合する。これらの算法・型・件数・除外範囲・診断文言は変更0とする。

旧保存本文へ元復元算法を静的適用すると `08600941CB5669B76A9EB938A40B7362203967C2ED167C00D2F2DA6EA5B03E5E`、MI後保存本文と現行本文ではともに `3768A05D1C6B65031492F1C5F8775900B636E5D1AB4AEDDDF5B0F3463AE0F3AF` であった。旧→新値の根拠はこの保存差分・承認・関数一致の鎖であり、現物hash単独ではない。この静的算出をG01動的PASSやFF18のREDと呼ばない。

追加所有はFF230行の `'08600941…'` 一意StringConstantExpressionAstの値64文字だけを上記 `'3768A05D…'` へ置換することに限定する。旧6値はS/REPORTの表を正本として202–203、659–660、717、721、722、723行に適用する。計7literalを逆適用した全bytesが旧FF AD4929と一致し、BOMなし・改行・46,482 bytesを保つことが必要である。helpの旧版記載も今回変更しない。

## 2. 要求と境界の対応

| 受入ID | module / 型・データ所有 | 状態・受入証拠 |
| --- | --- | --- |
| GB-R1、P7-S03/04 | FFの `Assert-SourcePreserved(ScriptBlockAst)`。入力はparse済みAST、成功時出力なし、失敗時元Assert-LocalのInvalidOperationException | 保存承認MI由来の固定期待1値。G01で元consumer全文正常復帰、G02/G03で元本文hash位置の拒否 |
| GB-R2、P7-S03 | test author所有FF7literalと一回用driver。元関数textはimmutable string、fixtureは別copy | 逆7差分全bytes一致、非所有入力前後hash一致、Parser/PSSA品質gate、独立static review |
| GB-R2、P7-S02/04 | runner所有の新UUID内JSON/log。bool sentinel、enum相当status、int件数・exit、string完全SHA | 期待拒否とAdapterFailureを分離、firstUnexpected停止、CreateNew保存、完了markerと実exitを接合 |
| GB-R3、P7-S05 | 親manifest/closure。designerは本書だけ、implementerの本体編集0 | 独立Post後D4限定Closed。Eの58期待＋ValidateOnly2、旧FF18、製品fullへのcredit0 |

新たな非同期処理、job、runspace pool、取消しprotocolは設けない。観測は単一pwsh内でG01→G02→G03の逐次同期処理である。取消し・同期I/O停滞・process異常終了で完了証拠が欠けた場合はIncompleteであり、再開や自動再試行をしない。各5秒未満/全60秒未満の前後monotonic検査を保持し、hard watchdogとは呼ばない。製品側の取消し・cleanup・Run/terminal永続順序は無変更である。

## 3. 最小consumer抽出契約

一回用 `Test-GuardBinding.ps1` は新UUID内にだけ置く。FFやInvokeのscript全体をdot-sourceせず、FFのtop-level FunctionDefinitionAstを名前・型・親scopeで各一意選択する。候補FF全体hashを親freezeへ照合してから、次の8関数のExtent.Textを全文・無改変で取得する。

- consumer：Assert-SourcePreserved。
- consumer依存：Assert-Local、Get-TextHash、Get-FunctionAst、Assert-KeySet。
- adapter既存helper：Read-SourceAst、ConvertTo-BoundBlock、Write-TestEvidence。

前者5関数がguard評価の閉包である。後者は固定path/hashでのparse、抽出blockのparse、CreateNew証拠保存にのみ使う。Write-TestEvidenceの `$script:resultFileWrites` はdriverが0初期化する。bootstrapのFF parse/一意選択はdriver内の局所処理とし、guard predicateを再実装しない。選択した各Extentの開始/終了offset、全文、UTF-8 hashを作者manifestへ保存し、原textのSubstringとのOrdinal一致と再parse error0を要する。未編集Assert-SourcePreservedのhashは `BB5B1C531A249AB6F38BD1FD5804E404E73FDCEF7D825F257DBC48DCF3C9A73A` である。候補の同関数は202–203と230の2literalだけを逆適用するとこれに一致し、他7helperは旧FF全文と一致しなければならない。

Read-SourceAstは候補fixture全体を独立freeze済みfixture hashで検査する。G01のfixtureは現Invoke898Cの全bytes copyで、Read-OwnedTreeが承認MI後318EとOrdinal一致することを別途照合する。G02/G03はG01から一文字だけ変更した独立copyである。fixture hashは作者作成→独立review→親freezeで確定し、runtimeが自動採番して期待に採用してはならない。guardの期待値は候補FF内の3768固定値をそのまま消費する。

抽出blockの新scopeに8関数を定義し、fixture ASTを引数として `Assert-SourcePreserved` 全文を呼ぶ。非所有118文hash、Initialize-Run先頭50文hash、Read-OwnedTreeの構造/本文、Get-CanonicalPath hash、writer禁止依存の順序を保つ。前後validatorを飛ばすtree-only block、独自のhash一致predicate、旧guard文字列をcatchして成功扱いする方式は採らない。G01で別validatorに拒否されたら追加変更せず停止する。

実行する製品関数は0である。Read-OwnedTree、Initialize-Run、Get-CanonicalPath、Write-FirstFailureRecordはASTデータとして読むだけで、それらのGetScriptBlock/呼出しは禁止する。FF main、Invoke main、probe main、Invoke-DiagnosticCase、FF18 loop、process/CIM/cleanup/native/network/製品起動は到達不能な抽出集合とする。runner自身のpwsh起動はこの「合成子process起動0」と区別する。0の主張は抽出text/依存閉包・実行argv/stackのレビューに基づき、ハードコードしたcounterだけでは証明しない。

## 4. 最大3観測と拒否識別

offsetはBOMを除いてParser/ReadAllTextが扱う0-based UTF-16位置、columnは1-basedである。lineだけでは選択せず、固定source SHA・tree offset・一意anchor・旧charの全条件を満たす必要がある。両変異はASCII `K` (U+004B) → `k` (U+006B) の一文字置換である。

| ID | fixture / 正確な変異 | 期待・sentinel |
| --- | --- | --- |
| G01 | 現行898C全bytes copy。treeが承認318Eと一致 | consumer正常復帰、afterGuard=false→true、出力0、例外なし |
| G02 | G01の1497行40列、絶対offset82907/tree内5619。anchor `'Known process registration is incomplete'` の先頭Kのみ。MI新span内 | 本文hash assertionでInvalidOperationException、MessageがOrdinalで `ORACLE: owned-tree body changed`。afterGuard=false |
| G03 | G01の1439行57列、絶対offset78363/tree内1075。anchor `'Known process snapshot row is not unique'` の先頭Kのみ。MI新span外、tree内 | G02と同じ拒否箇所/型/文言、afterGuard=false |

各anchorはtree内1件と確認済みであり、診断call2個の末尾4引数には含まれない。作者は各fixtureのparse error0、UTF-16差分1文字・符号化後差分1byte・長さ/BOM不変、変異逆適用でG01全bytes一致を静的に記録する。変異fixtureをsourceへ書き戻さない。hash対象なので、文字列の実行時意味を評価する必要はない。

sentinelはdriverの局所boolで各観測前false、guardの呼出しが正常復帰した直後にのみtrueにする。guard本文への代入挿入・Assert-Localのmockは0である。G02/G03はcatchしたErrorRecordの型・完全Messageに加え、保存stackで元Assert-LocalからAssert-SourcePreservedの本文hash assertionへ接続したことを確認する。抽出blockの仮想source pathとline/offset対応を作者manifestへ固定し、別assertやfixture hash/parserの例外を本文拒否に換算しない。型/stackが想定に合わなければAdapterFailureで停止し、未知のinner exceptionを無制限にunwrapして合格へ寄せない。

局所結果は `Passed`（G01受理又はG02/G03の期待拒否）、`Failed`（予期せぬ受理/拒否/sentinel/期限）、`AdapterFailure`（parse/固定入力/抽出/保存/環境不成立）、`NotRun` の4分類とする。G01の例外、G02/G03の正常復帰、出力混入、最初のunexpectedで停止し、残りをNotRunにする。旧本文0860の静的再現は観測数へ加算しない。guard変更前の実行や期待失敗を追加しない。

## 5. 保存、有限command、freeze

作者候補rootを `A=P/d03fbe3a-1318-4b39-8ba5-110b588e1640`、runner結果rootを `O=P/ed063a7c-e915-4126-84a7-d882e66fd8f1` と予約する。G1では作成しない。いずれかが既存なら上書きせず親へ返し、親が新UUIDへ再freezeする。P直下の絶対正規path、UUID D形式、祖先reparse無し、ignored境界、非admin、固定pwsh実版7.6.7と既承認runtime hashを開始前に確認する。sourceやfixtureのsymlink/reparseも拒否する。既存Assert-SafeFreshOutputはPSScriptRootを親に要求し新scriptのAと合わないため、そのまま転用して条件を黙って弱めない。driverは本節の固定P境界を確認する短い入口検査を持ち、任意root選択を許さない。

作者はAへFF.before、FF候補copy、Invoke旧/MI後/現行copy、G01/G02/G03 fixture、driver、抽出証拠・逆差分・静的gate結果をCreateNew保存する。承認記録snapshotもAへ保存する。manifestはこれらの入力path/完全SHA、原本path/完全SHA、親承認由来、8関数extent、変異offset、固定3期待、実行順・期限、非到達範囲を持つ。動的結果は含めない。manifestの自己hashを自己埋込みせず、親が外側から完全SHAを固定する。source/Test/Slash/KH/Context/probe等の非所有基準はSのinput-preservationと親の最終組に接合し、作者の現hash採取だけを承認にしない。

G2はFF7literalとAのみ書込み可、runtime観測0である。変更FF/新driverのParser error0、PSSA1.25.0全通常rule診断0・engine0、Stop、診断配列count、完了marker、実exit0が必要である。独立static reviewと親freeze後にのみ別runnerへ次の一回のcommandを渡す。この一回にG01–G03を順序固定で含め、CaseId選択・retryスイッチは設けない。

```powershell
$guardArgs = @{
    ManifestPath = 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/d03fbe3a-1318-4b39-8ba5-110b588e1640/manifest.json'
    ExpectedManifestSha256 = $approvedGuardManifestSha256
    OutputRoot = 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/ed063a7c-e915-4126-84a7-d882e66fd8f1'
}
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/d03fbe3a-1318-4b39-8ba5-110b588e1640/Test-GuardBinding.ps1' @guardArgs
$guardProcessExit = $LASTEXITCODE
```

`$approvedGuardManifestSha256` は未作成manifestの値であり、現在の設計段階では未確定である。親はG2静的review後、これを承認64hex literalへ展開した完全argvとdriver/manifest/fixture/FF/runtime/plan/designの全SHAを新freezeへ保存する。runnerはdriver自身も起動前にその外側freezeと照合する。未展開変数・仮hashでの実行は禁止する。既存E commandは本段階で呼ばない。

runnerはO未存在確認後のみ作成し、先に入力前hash・抽出対応・実runtime/argv・start時刻を保存する。各観測終了時のJSONをCreateNewで保存し、unexpected時も可能な範囲で後hashと部分結果を残す。最後に非所有入力・driver・manifest・fixtureの後hash一致を検査し、件数3/Passed3/NotRun0/Unexpected0とsentinel・5/60秒を確認してresult.jsonを保存した後だけ `GUARD_BINDING_COMPLETE passed=3 unexpected=0 notRun=0` を出す。実exit0を別runnerの外側logへ保存する。exit1は観測不一致、exit2はAdapterFailureであり、guardの期待拒否自体はexit失敗にしない。保存失敗・完了marker欠落・実exit欠測はPASSでない。既存結果のappend上書き、削除、cleanup controller呼出しは0、stream Disposeだけをfinallyで行う。

## 6. closure、停止とhandoff

独立Preの入力は本書の完全SHA、限定計画835C、凍結main plan/design、表1の固定FF/旧MI/新MI/現Invoke・停止REPORT・MI差分証拠・承認記録snapshotである。review対象は追加1literalの由来、全文consumer抽出と前後validator保持、最大3期待・変異位置、非到達/期限/保存、旧6値の範囲保全である。reviewerはread-only `gpt-6-astra/high`、親が指摘を保存し、planner精緻化→親受領の順とする。FF編集はその後の独立test author `gpt-6-astra/high` へ渡す。implementerの本体作業は非該当、runnerは別担当 `gpt-6.1-sol/medium`、Post reviewerは別担当 `gpt-6-astra/high` である。

G3の最大3件・実exit・前後整合・firstUnexpected・未実行を独立Postが受領した時だけ、D4の7literal bindingと追加guard検証を限定Closedにできる。FF18は既存EのContext12→MI MissingOs2→成功時MissingPure20→FF18→EO6→ValidateOnly2へ残す。D1–D3 Closed/C16限定Reuseを維持する。E前提のtoolchain.json/V2Case承認baseline2件は後続runnerのread-only接合責任であり、本G1では探索も解決もしていない。

保存承認由来の欠落、snapshot不一致、元算法の意味変更、7literal以外のFF差分、未知validatorの迂回要求、静的診断、観測unexpected/欠測/hash drift、追加権限・user-visible契約変更で停止する。前段guardが通らないことを新期待値追加で解消しない。旧本文のhashも許す二値集合は未知差分受理へ範囲を広げるため不採用、hashだけを複製した独自predicateは元consumerの退行を見逃すため不採用である。

rollbackは親が候補採用を取り消し、元FF保全copyと差分を照合して所有7literalだけを戻す。既存他者差分・旧証拠は保存し、候補packetも削除しない。戻した状態はD4停止であり旧runtime/F3を再開できない。新UUIDへの再実行は自動復旧にせず、原因の担当差戻しと再freezeを要する。

G1完了証拠は本書保存、参照/完全SHA/差分確認までである。FF編集、runtime観測、test実装、子起動、commit/公開は0。開始時の親current/usage変更と新計画を保持した。子の開始/終了token・週枠・実返却model/effort・親子包含は未取得、KPIは親の既存usage文書へ欠測として返す。残る実証リスクは新runtimeでの抽出consumer/例外stack/証拠I/O成立、同期I/Oのhard期限なし、及び別工程Eの未検証である。
