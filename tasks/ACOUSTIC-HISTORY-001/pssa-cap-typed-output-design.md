# PSSA-CAP：fixture成功出力と観測配列の限定是正

designerは既指定`gpt-6.1-sol/high`、api-and-interface-designの型境界だけを適用する。CAPは既定T/pssa-capture、baselineはCAP/caseobs-31d3adba-c9fc-4674-a8a8-ad64c34d2b3e/Test-PssaCapture.ps1、SHA256 `AB7EAFC5152C53A78D89EBA3933D40E3311BBCFC5F16D45E27F5D544140BBA26`である。本工程は文書だけ、source/fixture/PSSA実行・再委任0。

根拠は今回caseobsのC02 ordinary0/unknown1、raw Pipeline/Unknown/List<Object>・recordRefsも同型・primary=nullの観測と、保存済みCAP/typed-output-comparison-20261024.jsonだけである。比較ではPS7.6.7/PSSA1.25.0の実DiagnosticRecord一個がNoEnumerateでList/別参照、同じadvanced scriptblockのWriteObject(false)で実型/同一参照となった。pins17前後一致・analyzer/CAP対象0であり、これは20受入や旧C02未保存内部・元NRE原因の証明ではない。[Cmdlet.WriteObject(Object,Boolean)公式契約](https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.cmdlet.writeobject?view=powershellsdk-7.4.0)はenumerateCollection=falseの非展開出力を定める。既存advanced producerの同一実行thread内でだけ使い、別runspace/非同期化しない。

| 許可変更 | exact span／不変条件 |
| --- | --- |
| 成功出力11source箇所 | New-CapFixtureの`$producer`内だけで、各`Write-Output -NoEnumerate <expression>`を`$PSCmdlet.WriteObject(<同expression>, $false)`へ置換する。Diagnostic462・DiagnosticThenThrow465・DistinctErrorsSameText471/472・Unknown474・ProjectionFailure476/477・EngineThenPartialSave479・RecordCountOverflow481・VerboseThenWarning484/485の8分岐11statementだけ。expressionのobject生成/参照/回数/順序を変えない。RecordCountOverflowの1..257、C07のevents.Add順とThrowTerminatingError、NonterminatingのWrite-Error -ErrorAction Continue、旧RED producer185行は全bytes不変 |
| root観測配列3箇所 | obsRoot.missingの1172/1194、obsRoot.truncatedの1173に限り、元if式の条件/両枝/Overflow/順序を保持したままRHSを`[string[]]@(if (...) { <元then> } else { <元else> })`へ囲む。空・1件・複数いずれもstring[]で保持し、null/単独stringへ展開しない。個々の元stringを変更せず、elementを追加する修正ではない |

受入は既20期待のままである。型契約の接続はC02～05のDiagnostic、C07の先行Diagnostic→throw、C08の別error参照、C11の未知object、C12の元brokenRecord、C13の元error、C17の257件、C20のVerbose/Warningを各元objectとして既pipelineへ渡すことに限定する。受信側の型正規化/List展開/未知record救済、producer外のNoEnumerate変更、期待後付け、C07型の推測修正0。観測missing/truncatedの型保持は既string[]契約の回復であり、新field/schemaではない。64KiB・Deadline短形・最初のcaptureFailure/元ErrorRecord保持・既5秒/30秒・一回/firstUnexpected後NotRun/retry0を変更しない。

作者はAB7E原本・旧Save修正・旧FAIL/診断Partial/既知state欠測2・B/R/patchをbefore全bytes/hash保全し、新artifactだけを作る。許可diffは**11 emit＋3 array＝14source箇所**、Parser/原文scope/個数を静的照合し、逆適用した全bytes/BOMがAB7Eと一致することを独立Preへ渡す。C39B5/oracle0E20118B/20期待/規則/閾値/保存・停止・元失敗分類は不変である。新Test hashだけを必要なB/R/caller/patchへ再接合し、独立binding/復元・実行Pre→親freezeの後にのみ別runner新slotへ進む。既slot再実行0、本設計は実行gateを解除しない。実provider/usage値は未取得である。
