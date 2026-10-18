# P7-GUARD-LOCALE 限定設計

2026-10-18 / ACOUSTIC-HISTORY-001 / L1。独立Pre提出版であり、新script作成・新3観測は未解除である。planner指定designer `gpt-6-astra/high`、provider返却値は未取得。所有は本書のみ。入力は[locale計画](p7-guard-locale-plan.md) SHA `5B5600354C9E3A8F160E5997F7770322ED9796223ECFC6470503ED7737F28BB6`、親checkpoint `7d6709f` である。

旧[限定design](p7-guard-binding-design.md) SHA `184289A35BF03FD4101789E39120F4F9E159B4947BB30AB195A7314F7E879A4A`、旧[限定plan](p7-guard-binding-plan.md) SHA `01AD21496978F7D4D03191FACEB0C6A5F8679D52C2C3FE2BBF787E60D78EE32C`とmain文書を参照契約として維持する。既読AGENTS/development-team/KPI、api-and-interface-design、PowerShell review/automation基準を境界・例外・保存設計に適用した。UI/REST/製品の動作・永続schema変更は非該当である。

`P=F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01`、`A=P/d03fbe3a-1318-4b39-8ba5-110b588e1640`、`O=P/ed063a7c-e915-4126-84a7-d882e66fd8f1` とする。旧G3-FROZEN `9C3FD5B84C935DE4A770955C93BC302C0AA93656AA6B33E07132F9B8D9E171A5`、runner REPORT `9F0D81C22D7A74CCDAAE1F23FFED3AB2E33862CEEE8F0DF40C7AA707099E145A`、O/result `F989EC04A95075396EA1E3DB942F92AA359E0567EF025263A6467945A41E366C` が失敗証拠である。旧G02はAdapterFailure、旧G03はNotRun、実exit2のまま保存する。

## 1. 一次source未取得とローカル資源の証拠

公式PowerShell v7.6.7のraw source/resource取得はwebのcache missとshellのDNS未解決で成立しなかった。公式source読了やtagと配布binaryの完全同一性は主張しない。代わりに、計画が許す「同定済みruntimeの表示資源形式」を採用する。次は既インストールfileのread-only metadata/IL/resource読取であり、G01–03や元guardの実行ではない。

| 証拠 | 固定値・確認範囲 |
| --- | --- |
| `F:/Program Files/PowerShell/7/System.Management.Automation.dll` | 既承認SHA `70701F20AFBBF4E763924C2BDD0C0E9A20EEC35A0E4EFF439E0CCF77F20D6B26`。Assembly identity `System.Management.Automation, Version=7.6.0.500, Culture=neutral, PublicKeyToken=31bf3856ad364e35` |
| `F:/Program Files/PowerShell/7/ja/System.Management.Automation.resources.dll` | L1読取SHA `F672B902371AD615A7AD211EE387B73C65386E7686610B6DCCA9A91927D8DC94`。既存インストールの衛星、identity `System.Management.Automation.resources, Version=7.6.0.500, Culture=ja, PublicKeyToken=31bf3856ad364e35`。新freezeで追加承認する入力である |
| 両fileの版情報 | FileVersion `7.6.7.500`、ProductVersion `7.6.7 SHA: 3f20d575fe08ecb01e260bc19969bb8632bf103f+3f20d575fe08ecb01e260bc19969bb8632bf103f`。版表示と上記file SHAを分けて保存する |
| neutral埋込み資源 | manifest resource `System.Management.Automation.resources.DebuggerStrings.resources`、key `StackTraceFormat`、値 `at {0}, {1}: line {2}` |
| ja衛星資源 | manifest resource `System.Management.Automation.resources.DebuggerStrings.ja.resources`、同key、値 `{0}、{1}: 行 {2} で`。引用符を含まない完全format、末尾space/newlineなし |
| fallback前提 | `F:/Program Files/PowerShell/7/ja-JP/System.Management.Automation.resources.dll` はL1時点で不存在。public ResourceManagerのexact ja ResourceSet値と、ja-JP指定GetString値が上記ja値に一致した |

L1検査processは7.6.7/.NET10.0.13、CurrentUICulture.NameとCurrentCulture.Nameがともに`ja-JP`であった。これは新runの前提選定のための現在の独立観測であり、旧G3の欠測cultureを補う値ではない。OS/user/global/threadのculture設定変更は行っていない。

表示と固定fieldの対応は既承認SMAの公開 `CallStackFrame.ToString()` のMethodBodyを読み、moduleのmetadata tokenを解決して確認した。同メソッドは `DebuggerStrings.get_StackTraceFormat`、FunctionName、ScriptName（null時だけNoFile）、ScriptLineNumberを順に取得し、StringUtil.Formatへ渡す。FormatはCurrentCultureを取得してString.Formatを呼ぶ。keyはget_StackTraceFormatの文字列tokenで`StackTraceFormat`、resource managerのbase nameもIL文字列から `System.Management.Automation.resources.DebuggerStrings` と確認した。

再確認可能なtoken対応は `0600010E`=get_StackTraceFormat、`0600147D`=get_FunctionName、`06001479`=get_ScriptName、`0600010D`=get_NoFile、`0600147A`=get_ScriptLineNumber、`060058E3`=StringUtil.Format、`0A0000FF`=CultureInfo.get_CurrentCulture、`0A0001B1`=String.Formatである。公開ToStringのIL bytesは次であり、この固定DLLだけに帰属する。

```text
28-0E-01-00-06-02-28-7D-14-00-06-02-28-79-14-00-06-25-2D-06-26-28-0D-01-00-06-02-28-7A-14-00-06-8C-B6-02-00-01-28-E3-58-00-06-2A
```

ErrorRecord.ScriptStackTrace getterは保存済みfieldを返す。LockScriptStackTraceのIL読取ではDebugger.GetCallStackを列挙し、最初以外にEnvironment.NewLineを挟み、各frameのToStringをStringBuilderへAppendして同fieldへ保存することを確認した。これにより「先頭2行を各frame表示へ完全照合する」既存方式と表示formatterが接続する。これらの内部metadata読取はL1の証拠調査だけである。新driverはprivate getter/field/LockScriptStackTrace、内部StringUtilをreflectionで呼ばず、public ResourceManager/CultureInfo/String.Formatだけを使う。

## 2. L-R1–3の最小変更

| ID | 対象・型・変更 | 保持する条件 |
| --- | --- | --- |
| L-R1 | 新driverのReject分岐。先頭2frame期待をpublic資源と固定fieldから事前生成したstring2個に置換する | 例外型・Ordinal全文message・virtual file・throw44・guard230・frame名と順序・afterGuard=false・output0は全てAND条件のまま |
| L-R2 | 新manifestのlocale契約、同一run内のculture/resource preflightと前後観測 | 読取のみ、支持cultureはja-JPに限定、未知資源/別衛星/culture変化はAdapterFailure |
| L-R3 | 新root/manifest接合、追加入力の前後同定と結果へlocale証拠追加 | 元42入力・FF AF179・8関数・virtual text・3fixture・3期待・5/60秒・firstUnexpected・AdapterFailure優先exit2は不変 |

支持環境はCurrentUICulture.NameとCurrentCulture.NameがともにOrdinalで`ja-JP`の新runだけである。この一回用packetで多言語対応範囲を広げない。名前が異なる場合、又はja-JP専用衛星が存在する場合は観測開始前にAdapterFailureとする。cultureを変更して前提を成立させない。ja-JP→jaという既存ResourceManagerの親culture探索だけに依存し、neutralへのfallbackを成功として許さない。

driver preflightは既存42入力検査に加え、上記ja衛星SHA/identity/version、SMAの実loaded Assembly.Location/identityと既承認path、ja-JP専用衛星不存在を確認する。`[Resources.ResourceManager]::new('System.Management.Automation.resources.DebuggerStrings', [Management.Automation.CallStackFrame].Assembly)` を作り、`GetResourceSet(GetCultureInfo('ja'), $true, $false)` がnullでないこと、その `GetString('StackTraceFormat', $false)` が固定ja formatとOrdinal一致することを要求する。次に同じmanagerの `GetString('StackTraceFormat', CurrentUICulture)` が同値であることを要求する。`ignoreCase`、未知key、別base名、別culture、未固定formatを許さない。catchしてneutralへ切り替える実装はない。

固定formatは新manifestに独立出典付きで保存する。placeholderはこの全文一致で `{0}`、`{1}`、`{2}` 各1個に固定される。ResourceManagerが返した値をそのまま無条件に期待へ採用せず、まず上記固定formatへ一致させる。その後、CurrentCultureを明示したpublic String.Formatにobject配列を渡して次の2期待を生成する。

| index | `{0}` string | `{1}` string | `{2}` Int32 |
| --- | --- | --- | --- |
| 0 | `Assert-Local` | 旧manifest.virtualSource.pathそのもの | 44 |
| 1 | `Assert-SourcePreserved` | 同じpath | 230 |

旧manifestのassertLocalThrowLine=44、guardAssertionLine=230もpreflightで固定値へ一致させる。旧stackFirst/stackSecond英語literalは旧証拠として読み込むが、新recognizerで使うのは新しい局所expectedFramesだけであり、旧manifestを書き換えない。新driverの行番号が変わっても、virtual sourceは旧Aのpath、元FFの44/230を保持する。

既存 `ScriptStackTrace -split '\r?\n'`、frames.Count>=2、frames[0]/[1]のcase-sensitive全文比較を維持する。抽出やmatch用の新regex、言語別error catch、末尾/部分一致、path正規化、広い例外unwrapは追加しない。stack実値は証拠として保存するが、期待生成・format選択の入力にしない。InvocationInfo.ScriptName/ScriptLineNumberも従来どおり独立に検査する。未知型・異なるmessage/位置/順序はAdapterFailure、既知拒否でもsentinel/output不適合はFailedである。期限超過が重なった場合もAdapterFailureが優先する旧cycle2条件を保持する。

culture証拠は同一driver processでpreflight時、各実施caseのguard直前/直後、finallyの結果保存前に取得する。2つのNameが開始値ja-JPから変化したらその時点をAdapterFailureとし、残りNotRun・exit2を維持する。guard直後の差異をcase catchでFailedへ再分類しない。postでは資源解決値とja-JP衛星不存在も再確認し、DLL hashは追加入力の前後検査に含める。取得不能もAdapterFailureである。旧runのculture欄を補記しない。

## 3. 所有、input42、修正span

新候補を `N=P/4e5f72fc-b5bc-45e1-9052-dc4d47e3e03f`、新結果を `Q=P/b5517847-0aff-42cf-85f2-80ac79f064a1` と予約する。L1では作成しない。既存なら停止し親の新freezeを待つ。旧A/Oは全体を保全し、元helper/fixture/FFをNへ複製しない。

N/manifest.jsonは旧manifest `A/manifest.json` SHA `F41DB193AC1FA6B79D46180B83DC68EFF863CEA0FAB3BC78D0E6AD2C3E201CA8` をbaselineManifestとして固定参照する小さな追補とする。旧driverは `6163E24572B51BA389ECD70FA8E2BA27C838DCE0D712BD36F2D64BE0513E1DA7` である。元42件は旧manifestから順序・path・SHA・originを変更せず読む。旧driverも42件の一つとして保持する。

新manifestの追加情報は、baseline参照、localeの固定fields/format/資源identity/version、N/Q、L-R1–3、追補plan/design、追加input4件（新driver、ja衛星、新plan、新design）だけとする。新driverと新manifest自身のSHAは外側freezeで固定する。新manifestは新driverのSHAを持てるが、自己SHAを自己埋込みしない。元42＋追加4=46入力、baseline manifestと新manifestを加えた内部post48入力を正本とし、外側freeze自身はrunnerが別に照合する。新plan/designの最終SHAはplanner精緻化後の親freezeで確定し、現在値へ自動追従しない。

新driverは旧driver6163からの局所差分で作る。許可spanは次の5群に限定する。

1. `$candidateRoot` / `$reservedOutput` のN/Qへの接合、旧AをbaselineRootとして明示するpath宣言。
2. 自身manifest読取後の追補manifest検査、固定旧manifestのhash確認と読取、元42＋追加4のboundInputs構成。既存before/post入力loopの対象だけをboundInputsへ変更し、requiredInput確認では新driverを追加集合でexact1件、FF/fixtureを旧集合で各exact1件とする。旧/新manifestの固定前後照合を接合する。
3. virtualSource.path検査のbaseを旧Aへ固定する。virtual text/sha/length/8関数の抽出・評価は不変。新候補rootへ仮想pathを移さない。
4. preflightのpublic資源/版/culture検査とexpectedFrames2個の生成、Reject分岐の旧2期待参照だけをexpectedFrames[0]/[1]へ変更する。既存型/message/file/line条件には手を加えない。
5. before/case/最終resultへの同一run locale証拠と、上記前後不一致のAdapterFailure伝播。保存順・CreateNew・出力marker・実exit・既存優先分類は保持する。

作者はこれらのold/new span・AST位置・逆差分を証拠にし、逆適用後の旧driver全bytes/SHA一致を示す。変更scriptのhelpは同一run locale記録と固定ja-JP前提の説明だけを追加可とする。新manifestの旧参照部分が旧42値を改変しないこと、ff/helpers/cases/virtualSourceが旧値へ直結することを独立static reviewerが確認する。無変更FF/fixture/8helperへParser/PSSAを反復せず、変更driverだけParser0/PSSA1.25.0全通常rule診断0・engine0・Stop・明示marker・実exit0を要する。必要なresource/IL出典証拠はN内の作者報告へ保存し、内部APIを実行経路へ持ち込まない。

## 4. 有限command、保存と受入

次が新観測の一回用commandである。script/manifestは未作成であり、`$approvedLocaleManifestSha256`は独立static review後に親が決定する承認64hex値へ必ず展開する。他引数は以下の固定値である。展開済み完全argv、46入力・baseline/new manifest・driver/runtimeの全SHA、支持culture/資源条件を親の新freezeへ保存するまで実行禁止である。

```powershell
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/4e5f72fc-b5bc-45e1-9052-dc4d47e3e03f/Test-GuardBinding.ps1' -ManifestPath 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/4e5f72fc-b5bc-45e1-9052-dc4d47e3e03f/manifest.json' -ExpectedManifestSha256 $approvedLocaleManifestSha256 -OutputRoot 'F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01/b5517847-0aff-42cf-85f2-80ac79f064a1'
$localeProcessExit = $LASTEXITCODE
```

runnerは新candidateの自己同定を外側freezeへ照合してからこの一commandだけを実行する。stdout/stderr/実exitは親freezeが指定する新証拠保存先へ非上書き保存する。Qの事前不存在条件を保つため、外側logでQを先に作らない。旧G3で使った外側保存と同じく、親の新freeze保存dirへrunner log/reportを置く。Nには作者証拠以外の実行結果を上書きしない。

新driverの保存は旧契約どおりbefore→実施case→after→result→完了markerの順、CreateNewである。locale資源の出典/完全format、実Name、生成expectedFrames、guardのraw stackを区分して保存する。主結果のexit2優先、保存エラー/marker欠落/実exit欠測は未受入とする。culture観測追加自体を新caseやPASS件数にしない。G01→G02→G03各一回、最大3、各5秒/全60秒の前後検査、firstUnexpected/retry0、main/process/CIM/cleanup実行0は旧設計のままである。新async/cancellation境界、hard watchdog、再起動は設けない。

L1独立Preは①DLL/resourceの独立出典と版・public identity、②ja-JP→ja限定fallback/未固定衛星拒否、③L-R1のAND条件とunknown拒否、④旧42不変・修正span・新freeze、⑤同一run culture/保存/exit2優先を確認する。Pre後planner精緻化・親受領を経て独立test author `gpt-6-astra/high` がL2を所有する。source implementerは非起動。L3は別runner `gpt-6.1-sol/medium`、各reviewは別担当 `gpt-6-astra/high`、再帰委任0である。

3Passed/Unexpected0/NotRun0、同一run culture/resource前後一致、48内部入力保全、保存成功、完了marker、実exit0を独立Postが受領した範囲でだけlocale接合を閉じる。旧G02の再集計、FF18/E/旧F3/製品へのcreditは0。未知culture/資源、入力drift、期待不一致、必要な所有外変更、品質未達、追加観測要求は停止し親へ返す。rollbackはN候補採用の取消しだけであり、旧A/O・現FF・旧証拠を逆編集しない。

L1完了は本書・参照/hash/所有確認までである。旧FF/script・test・親docsの変更、新観測、子起動、commit/公開は0。既インストール資源のmetadata読取は行ったが、旧runのcultureは未取得のままである。子usage開始/終了・週枠・provider返却model/effort・親子包含は未取得、親KPIへ欠測として引き継ぐ。
