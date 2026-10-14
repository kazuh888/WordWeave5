# ET3-G：同一Cargo実行のEXE承認境界

2026-10-14。状態はPre-1のP1二件を反映した設計案・計画同期／独立Pre-2待ちである。[計画](early-trial-gate-plan.md) SHA256 `EDA787512F47F24A360C068ED3C78C38C2BD1B687DA7C37869DC0D5F397C0E1F`末尾の是正方針、[Pre指摘正本](early-trial-delivery-review.md)冒頭、[承認済みET契約](early-trial-contract.md)と同reviewの利用者承認を入力とする。指定designerは `gpt-6-astra/high`。今回process限定runner導入だけを具体化する。製品sourceは親受領のcb882031同体、HEAD6fb6da3。着手時git statusで既存dirty文書8件と未追跡の本書／計画を確認した。所有は本書だけ、実装・テスト編集・runtime・再委譲・公開は0である。旧設計SHA `92E370C131AAF46201BD9D575ADEC1DC0A8E119450587D8DB33966043B07A09C`へのPre不合格は履歴として保持し、本書だけで解除しない。

AGENTS、development-team、KPI、`agent-skills:api-and-interface-design`全文を確認した。入力・承認・実行結果の型を分け、外部境界で検証する。REST層は不要である。UI変更なしのためwordweave-egui-uiは非該当。日本語折返し、ボタン両軸中央、DPI100%、狭幅、focus/keyboard、直接起動・保存・再起動・音読導線は既定native未了として保持する。DPI150/200%だけは利用者承認どおり試用時へ移す。

## 根拠と最小構成

- `src/early_trial_test_support.rs`全文、`src/media.rs`録音3対照、`src/ai.rs`global/lease/poison/対照、既存`target/early-trial-build/et2-egress-runner/Invoke-V2Case.ps1`全文を読んだ。V2Caseはexact/listと65秒監督を担うが、任意argvやCargo全体の代理には使えない。
- [Cargo JSON仕様](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages)はcompiler-artifactのpath/profile/featuresとbuild-finishedを提供し、build-finished後にtest出力が続く。[runner仕様](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)を使い、同じCargoが渡すEXEを停止させる。Cargo終了待ちでJSONを読むとrunner待ちと相互待ちになるため、stdoutを実行中に継続排出する。
- `full2-default-no-run/artifacts-fixed.json`の24artifact中test profileは22である。artifact数を起動数へ読み替えず、旧SHAも再利用しない。source170/G0 12/marker4は既存full2固定入力を親が現物に照合して再使用する。
- full2元inline commandの保存scriptは親・設計者とも未取得である。「保存script再利用」とはせず、既存V2Caseの`Start-Process @launch`（FilePath/ArgumentList/WorkingDirectory/WindowStyle=Hidden/PassThru/RedirectStandardOutput/Error/Environment）と、`et3/full2-fixed-envs.json`、`full2-default-no-run/{start,result,toolchain}.json`の実値を限定接合する。Cargo実pathは`C:/Users/安井　一広/.cargo/bin/cargo.exe`、cwdはrepo rootである。[公式Start-Process仕様](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/start-process?view=powershell-7.6)でEnvironment（7.4以降）とfile redirect/Hidden併用を確認した。

実装rootは解決済みignored `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`（以下R）である。書込前に`git check-ignore`と絶対path境界を確認する。

| G / ET | 所有・モジュール | 契約・受入への対応 |
| --- | --- | --- |
| G1/G2/G5、ET-07 | implementer：R/Invoke-CargoGate.ps1のRunOwner／Supervisor | RunOwnerは有限外側監督、SupervisorはCargo子環境・liveログ・source照合・EXE保持とG0監督を担う。承認発行権はない |
| G2/G3/G4/G5、ET-06/07 | implementer：R/cargo_gate_entry.rs → 同ps1のRunner | native入口はCargoの全argvをUTF-16列として保存し、固定pwshへ渡す。Runnerは親full承認と現在値を照合し1子だけを起動・監督する |
| G2/G5/G6、ET-06 | 同ps1のSafetyWorker | Supervisorの直系子。登録／開始ACK後だけ固定V2Caseを直系子として起動し、aggregate期限・cancel・監督喪失から回収する |
| G2/G6、ET-06 | 親：Run入力・g0/full承認・結果照合 | 新source/JSON/EXEの対応と同新EXEの安全対照結果を確認する。親自身は別treeでV2Caseを起動しない |
| G1–G6 | 独立作者：R/Test-CargoGate.ps1、R/fake.rs | fake起動marker/argv/exit/子寿命を独立oracleとする。製品試験期待値に触れない |
| G1–G6 | runner：R/runs/<runId>の証拠、既存delivery-runner報告 | 固定したscript/test/fake SHAで実行し、wrapper障害と製品結果を分離する |

実装規模はM・4file（作者2、implementer2、生成fixture別）である。native入口とfakeを共有せず、製品Cargo／依存に追加しない。今回の固定field検証・hash・原子的保存・有限監督を同ps1内に置き、別の監督script、汎用schema、常駐heartbeat、REST層は作らない。RunOwnerは同ps1の内部mode追加であり、親は4file／既存責務内として計画同期へ返すことを受領した。Supervisorの異常終了を回収する外側実体が必要なための追加であり、AI会話そのものをprocessと見なさない。

### 固定入口宣言（Pre-1 P1-入口）

`Invoke-CargoGate.ps1`の入口は次の宣言に固定する。`Mode`はparameter setごとの定数照合も本体冒頭で行う。`RunPath`はR/runs/<runId>/run.json、`InvocationPath`は同runのentry payload又はSafetyInvocationに限定する。未知mode／余分な位置引数／modeとsetの不整合／`InvocationPath`を伴うRunOwner・Supervisorは副作用前に拒否する。

```powershell
[CmdletBinding(PositionalBinding = $false, DefaultParameterSetName = 'Control')]
param(
    [Parameter(ParameterSetName = 'Control', Mandatory)]
    [Parameter(ParameterSetName = 'Child', Mandatory)]
    [ValidateSet('RunOwner', 'Supervisor', 'Runner', 'SafetyWorker')]
    [string]$Mode,
    [Parameter(ParameterSetName = 'Control', Mandatory)]
    [Parameter(ParameterSetName = 'Child', Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string]$RunPath,
    [Parameter(ParameterSetName = 'Child', Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string]$InvocationPath
)
```

Mode／RunPathは全set共通、InvocationPathだけChild所属とする。ControlはRunOwner／Supervisor、ChildはRunner／SafetyWorkerだけを許す。CommonParameterはpwshの機能として存在するが、内部呼出の固定argument列には含めない。Cargoの`-Verbose`、`-ErrorAction`、未知option、`--`、空文字をこのbinderへ渡す経路はない。`ValueFromRemainingArguments`、`$args`による任意argv受信、`--runner`、`--supervise`の旧案は削除する。

Cargo runner環境値は次の空白なし絶対path一語である。

```text
F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/cargo-gate-entry.exe
```

nativeは `std::env::args_os().skip(1)` を各 `OsStrExt::encode_wide()`で変換する。先頭要素がCargo対象EXE、残りがtarget argvである。全要素を`argvUtf16: Vec<Vec<u16>>`として整数だけのJSONへ保存し、runId／invocationIdは検証済みASCII UUIDとしてだけ文字列出力する。空引数は`[]`、非BMPはsurrogate pairのまま、正規化・lossy変換・再splitをしない。[Rustのencode_wide契約](https://doc.rust-lang.org/std/os/windows/ffi/trait.OsStrExt.html)は不正UTF-16を含むcode unit列の保持を定義する。PSは各整数を`char`に変換した`char[]`からStringを構築し、Encoding変換やUnicode置換を使わない。

nativeはrun path/idをCargo子限定環境`WW5_CARGO_GATE_RUN_PATH`／`WW5_CARGO_GATE_RUN_ID`から読み、型とR配下の位置だけを検証する。追加JSON crateを使わず、未知fieldを含むRunの承認判定はPSに一元化する。invocationIdは時刻・PID・process内連番から組むUUIDv8形式の識別子であり秘密nonceではない。invocations/<id>を新規作成し、同名があれば既存を読まず新IDで最大16回だけ試み、その後は拒否する。entry payloadはCreateNew／atomic公開し、entryPidを併記する。UUID衝突で別invocationを上書きしない。

nativeが起動するものは固定実path `F:/Program Files/PowerShell/7/pwsh.exe`だけであり、`std::process::Command`の個別argで `-NoLogo -NoProfile -NonInteractive -File <固定ps1絶対path> -Mode Runner -RunPath <run.json> -InvocationPath <payload.json>`を渡す。任意target argvはこのcommandに含めない。CREATE_NO_WINDOW、stdin/out/err継承、cwd/env継承で起動し、PS終了を待って32bit exitを伝達する。capture／固定起動／待機／exit以外のhash・承認・G0判断をnativeへ複製しない。

## データ契約と所有

今回のRun/Manifest/ApprovalはUTF-8 JSONとし、下表の必要fieldだけを明示検証する。runIdはUUID、cfg/phase/decisionは固定集合、固定cargoArgv/featuresは文字列配列、native由来argvUtf16は整数配列の配列、時刻は期限比較可能なUTC、pathは所定の解決済み絶対path、SHA256は保存済みbytesに対する大文字64hexである。argvUtf16は1要素以上、各値はJSON整数0..65535（bool／小数／指数表記／nullは不可）、先頭EXEは非空、NULはWindows argvに存在できないため拒否する。空の後続要素／孤立surrogateを勝手に除去・置換しない。必要field欠落/type違い/重複key/不明状態/重複したphase承認/境界違反は拒否し、未知の追加fieldに権限を持たせない。汎用schema engineやversion移行は作らない。入力に秘密値・環境全文は含めない。

| 型・書込owner | 必須内容 |
| --- | --- |
| Run / 親 | 新規UUID runId、cfg、repo cwd、Cargo実path/version、cargoArgv、固定EnvMap、開始/絶対期限、source170・G0 12・marker4のpath/hash集合と各入力file hash、ps1/native source/native EXE/pwsh/V2Caseのpath/hash、予定artifact identity集合、G0 list/exact順序と件数。related構成は親が既定command/集合を別Runへ固定 |
| Manifest / Supervisor | runId、Run SHA、cfg、Cargo PID＋開始時刻、source集合digest、JSON prefix byte数/SHA、全executable artifactのpackage/manifest/target/kind/profile/features/絶対path/SHA、起動許可候補集合、作成時刻。全source再照合と全EXEの読取lock取得後に発行 |
| EntryPayload / native | runId、invocationId（UUID）、entryPid、argvUtf16。Runとの一致とentry→Runnerの実OS親子／開始時刻をRunnerとSupervisorが照合する。payloadだけでは起動権限を与えない |
| Request / Runner | runId、invocationId、cfg、Cargo→entry→Runnerの各PID/開始時刻、payload SHA、canonical EXE、argvUtf16、現在EXE SHA。CreateNewで作るinvocation固有fileで証拠を対応付ける。同じEXE/argvの正当な再呼出は別invocationとして許可し、毎回Gateを適用する。重複IDだけを拒否し、exe＋argvによる回数制限を追加しない |
| Approval / 親のみ | phase=`g0`又は`full`、decision=`allow`又は`deny`、runId、Run/Manifest SHA、cfg、source/G0/marker digest、絶対expiry、承認記録参照。g0はexact順序・list権限、fullはそのG0成功証拠SHAを含む。承認を別runや同hashへの復元へ再利用しない |
| SafetyInvocation / Supervisor | runId、Run/Manifest/g0承認SHA、caseIdと順序番号、list/exact区別、EXE/SHA、exactName、RequireChildMarker、固定V2Case path/hash、専用OutputDirectory、aggregate/case/Run絶対期限。新caseごとに一度公開する |
| ProcessIntent／Registration／Ready／StartAck / 起動owner、子、起動owner | runId、親子role、invocation/caseId、intent UUIDとSHA、実OS PID/開始時刻/path/command、ready SHA、開始可能期限、親owner/supervisor identity。Intentはspawn前、Registrationはspawn後owner、Readyは子、StartAckはownerだけが一度公開する。ACKは製品承認を代替しない |
| Result / 各監督 | runId/invocationId/caseId、manifest/request/ack hash、開始意図、PID/開始時刻、raw argvUtf16、childExitInt32と同bitsのchildExitU32、gateStatus、timeout/cancel、kill/wait、既知子孫と残存、fixture残存、source前後一致。親は最終的にCargo exitと各Resultを接合 |

保存順は同directory内の新規tempへ全bytesを書込・Flushして閉じる→既存宛先を上書きしないrenameで公開である。公開済みRun/Manifest/Approvalは変更しない。失敗・拒否時は新しいcancel/terminal証拠を残す。承認の部分書込は待機中の完成品と扱わず拒否する。runnerは読んだ承認bytesとhashを記録する。署名・ACLを追加する設計ではなく、親と担当の所有規律を前提とする。同一ユーザーの悪意ある改竄への防御を主張しない。

## 時間順序・状態遷移

1. `Prepared`：親が静的G0、source170/G0 12/marker4、Runのcommand/env/予定集合を照合し、全source writer停止・同target別buildなしを宣言する。独立runnerが固定pwshで同ps1を `-Mode RunOwner -RunPath <run.json>`として一度起動し、その実processが外側監督となる。RunOwnerは通常ユーザー・固定入力・全wrapper hashを確認し、直系SupervisorをHidden起動／登録し開始ACKを出す。SupervisorはACK前Cargo起動0。source170は全期間lockせず、build前後、親承認時、各EXE実行直前と終了時に全bytes hashを照合する。ソース変更を伴わない凍結検証であり、変更検出/所有違反はrun全体を無効化する。
2. `Building`：Cargoだけは固定5commandの既知token（空白・引用符・空文字なし）を単一ArgumentListへ連結し、`Start-Process -FilePath <固定Cargo絶対path> -ArgumentList <固定command文字列> -WorkingDirectory <repo> -Environment <Runの子EnvMap> -WindowStyle Hidden -PassThru -RedirectStandardOutput <run/stdout.log> -RedirectStandardError <run/stderr.log>`で起動する。-Waitを付けず、250msの既存型監督loopでlive stdoutをFileShare.ReadWriteにより読取る。前回offset＋未完行だけを保持し、改行まで揃ったUTF-8行だけJSON解析する。縮小/置換/不正行、Cargo終了前に未完のまま期限切れなら拒否する。最初の`build-finished success=true`までのfile bytesをprefix固定し、その後のtest出力は解析しない。false/予定集合不一致は全解除禁止、fresh=trueも同run artifactとする。stderrは既存redirectのまま保存し、独自pipe排出器を追加しない。
3. `AwaitingManifest`：Cargoが呼ぶ最初のRunnerはRequestを出して停止する。build-finishedが未到着ならSupervisor側収集を待つ。JSONがrunner起動より後に届いても非実行を保ち、期限までに全prefixが取れなければ中断する。stdout収集をrunner待機で塞がない。build-finished前のbuild-script実行は通常Cargo buildの既承認範囲であり、ここで保証する「起動0」は対象test EXEについてである。
4. `AwaitingG0`：Supervisorは全EXEを読取lockし同handleからhashを取り、source再照合後Manifestを公開する。親はraw JSON prefix/source/EXE/profile/features/target・予定集合・first Requestを独立照合し、g0承認を発行する。manifest内に載るだけでは起動許可にならず、full Runnerはここでも待つ。承認前のEXE `--list`も0である。
5. `SafetyChecks`：Supervisorがg0承認に束縛されたSafetyWorkerを一件ずつ直系子として起動する。worker登録／開始ACK後だけ同Manifestの新binとcodex_processを既存V2Caseでlistして実名集合を照合し、その後新binの録音3→P01既定順20→新integration自身A11を実行する。Supervisorとworkerは各V2Case起動直前にもsource170/G0 12/marker4を再照合し、V2CaseのEXE hash確認と組にする。親はこの固定対象／順序をg0発行時に照合し、別tool treeで起動しない。順序の入力は既存`fx1-default-selected.log`先頭23行（試用も同23）と`fx1-integration-selected.log`全11行をsource実名と照合しRunへ固定する。新binの旧G0/PASSを接合しない。V2CaseのTimeoutSeconds=65、内部helperの既定45秒は変更せず、workerの外側case時計はV2CaseのStart直前から65秒、aggregate残りが短ければ外側が先に止める。exact1/PASS1/必要完了marker/AdminFalse/残存0を必要とする。case結果・回収完了前には次workerを開始しない。親が同run全ログとG0結果digestを独立照合するまでfullを発行しない。
6. `RunningFull`：親が同run/Manifest＋新G0結果digestを束縛したfull承認を1枚発行する。待機中Runnerと後続Runnerはこれを共用し、各spawn直前に有効期限・cancel不在・live RunOwner/Supervisor/Cargo/entry identity・Run/Manifest/承認hash・source/G0/marker一致・対象EXE hash・許可集合・当該Runner登録ACKを再照合する。EXE読取lockを終了まで保持し、変更できなかったという推測だけでhash照合を省略しない。構成ごと2枚の親承認であり、24件の人間待ちは設けない。
7. `Finished`：Cargo、全Runner、G0監督、既知子孫の終了/Waitを確認し、source/marker/旧製品の保全とfixture残存0を記録してからlockを解放する。成功判定にはCargo exit0だけでなく全gate Result整合、予定実行集合との対応、実test集計を必要とする。24artifact全部が起動されるとは要求しない。未起動のprofile.test=false補助binと対象testを区別し、件数減を自動容認しない。

任意状態の拒否・変更・取消し・期限切れ・親/supervisor喪失は`Aborting→Stopped`だけへ進む。旧runの再開・自動承認・自動再試行はない。Run開始から3600秒上限、build1800秒、各承認待ち600秒、full各EXE600秒、回収wait10秒を上限とし、残り全体時間の方が短ければそちらを使う。時間延長は親の別Run判断であり待機中の期限を書き換えない。G0中のfull待機は親のG0処理を含む600秒であり、足りなければ失敗として返す。

600秒はaggregateの失敗上限であり、34exact×65秒=2210秒（list2件を含めれば最大2340秒）を全件使っても完了する保証ではない。g0承認の親記録publishedUtcからfull承認受領までの600秒を固定する。親はg0公開時点で`g0DeadlineUtc=min(publishedUtc+600秒, Run期限)`も承認へ含め、Supervisorは将来のpublishedUtcを拒否して同期限を固定する。観測が遅れても時計を後ろへずらさない。期限切れでは実行中G0を所有照合して回収しfull起動0を保つ。G0とfull承認待ちの時計を二重にリセットしない。各監督はUTC残り時間から作るStopwatch上限も持ち、時計の逆行による延長を許さない。fakeはこのaggregate期限切れを独立検証する。全case上限を許す時間枠へ変える必要が生じた場合は親/plannerへ返し、自動延長・case短縮・省略はしない。

## process・argv・回収境界

- `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER`は上記native絶対path一語だけであり、pwsh探索用の追加PATHは不要である。Start-ProcessのEnvironment.PATHが持つPSHOME／Machine/User PATHの実効挙動は固定pwsh7.6.6で記録する。各PS modeは自身の実process path/version/SHAとRunの固定値を照合し、nativeは指定のpwsh絶対pathを直接使う。Cargo runner pathに空白が入る変更は今回対象外として拒否し、aliasやglobal設定へ進まない。
- LOCALAPPDATA/TEMP/TMP、target、JOBS2、repo cwd、WW5_ET2_CHILD/BASE空、既定CARGO_LOGはfull2の構成別EnvMapを使う。runner値とrun path/idだけをCargo子環境へ追加する。親環境を`$env:`で書き換えない。Cargoが実行時に加えるライブラリPATH等はnative→PS→targetへ保持する。環境照合は固定keyとrunner実pathを対象とし、CargoによるPATH追加を誤って違反扱いしない。
- Runnerはpayloadの先頭からcanonical EXEを検証し、残り全argvを順番・空文字・UTF-16 code unit・空白・引用符・末尾backslashを含めProcessStartInfo.ArgumentListへ1要素ずつ渡す。shell、Invoke-Expression、Start-Processの任意argv文字列連結は使わない。cwd/envを継承しUseShellExecute=false/CreateNoWindow=true、stdout/stderr/stdinをOS handleとして継承する。PS pipelineを経由せずbyte列を保ち、wrapperログやnativeエラーはsidecarのみへ出す。filter/skip/ignore/test-thread/quietを付加しない。
- 通常完了時はchild.ExitCodeのInt32をPSの明示`exit`へ渡し、nativeはExitStatus.code()のInt32を`std::process::exit`へ渡す。符号を成功判定へ転用せず、同じ32bit patternを保持する。[pwsh -Fileのexit規則](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_pwsh?view=powershell-7.6)を使い、0/7/101/0x80000001/0xFFFFFFFFを実native→PS→fakeで確認する。通常終了のResultはgateStatus=`Completed`、製品exitはそのままである。gate拒否/timeout/cancel/監督障害は非zero＋`Rejected`/`TimedOut`/`Cancelled`/`OwnerLost`/`CleanupIncomplete`で区別する。製品の同じ数値exitと混同せず、sidecar欠落はunknownであってPASSではない。
- 各ownerは開始意図を先に保存し、spawn直後のProcess handle／PID／開始時刻を保持し登録する。正常exitでも子孫残存を監査する。OSのPIDだけでなく同じ開始時刻・image・command・親子関係を確認し、未知processを停止しない。有限kill/waitの実体と登録raceは次節で固定する。
- 継続lockは承認対象EXEへ限定する。SupervisorがManifest作成前から全対象EXEを`FileAccess.Read + FileShare.Read`で開き、同handleでhashし、G0/full/全回収まで保持する。これが閉じるraceは「hash照合後からspawnまでのEXE差替え」である。sourceは実行imageではなく凍結buildの根拠であり、非並行writer所有＋全件再照合が既定G0を満たすため全170fileの全期間lockは不要と判断した。writerが停止を守れない・一時変更/逆復元が判明した場合はhashが戻っても承認失効である。悪意ある非協調writerへの一般的防御は追加しない。EXEのlock取得/正常実行両立に失敗したらSTOPし、lockを外して続けない。
- 全process終了確認前にTEMPや合成rootを削除しない。fixtureは今回の検証済み絶対pathだけをcleanupし、証拠を保持する。共有full2環境は構成の全作業が終わるまで保持し、内部一時fixture残存0と空の保持directoryを区別する。中断したrunは証拠を保存したまま無効化し、次回は新runId/新承認から始める。

### 所有木・開始ACK・有限回収（Pre-1 P1-回収）

最小所有木は次である。RunOwnerは独立runnerが起動して最後まで残す実pwsh processであり、Codexの会話IDや任意の既存shellを代用しない。OS親processの生存はAI応答継続を意味しない。AIが応答を止めた場合は承認欠落／600秒が終了条件となる。

```text
RunOwner（同ps1、外側の3600秒監督）
└─ Supervisor（同ps1、live JSON／承認／G0 aggregate監督）
   ├─ Cargo → native entry → Runner（同ps1）→ full test → 子孫
   └─ SafetyWorker（同ps1、同時1件）→ 既存V2Case → exact/list test → helper子
```

RunOwnerを外側に置く理由はSupervisor自身の異常終了を回収するためであり、SafetyWorkerを別processに置く理由はV2Case内部の無期限WaitForExit()をSupervisorへ持ち込まないためである。V2Caseの内容・65秒・期待値・引数を変更しない。SafetyWorkerはpwsh -File <固定V2Case>と既知のnamed parameterをProcessStartInfo.ArgumentListで起動する。`-Executable`、`-ExpectedSha256`、`-ExactName`、`-OutputDirectory`、`-TimeoutSeconds 65`、該当時だけ`-List`／`-RequireChildMarker`である。streamはcase専用fileへ保存する。V2Case stdout内のresult JSONは原文保存し、worker証拠と別に照合する。

登録手順は次で固定する。

1. 起動ownerがintentと期限をatomic公開し、固定child processをHiddenでStartする。ownerは返されたhandleからPID＋StartTimeUtcを取得してRegistrationを公開する。子はRun/intentを確認し自身のPID＋開始時刻をReadyへ公開するが、ACKがまだなければ待つ。RunOwner→SupervisorはCargo開始、Supervisor→SafetyWorkerはV2Case開始、Supervisor→Runnerは対象EXE開始をここで禁止する。RunnerはCargo由来なのでSupervisorの直系ではないが、Cargo→entry→Runnerの実OS親子鎖と時刻を登録時に確認する。
2. ownerはRegistration／Readyの同一identity・parent chain・command・Run/manifest/case SHAを実OSと再照合する。workerはg0承認後だけ、Runnerは登録時点で対象起動を許さずfull承認も必須とする。worker ACKにはg0Deadline／caseId／順序番号を含める。RunOwnerはSupervisor登録と各Registryを継続読取し保持する。
3. ownerがStartAckを一度atomic公開する。子はACKのidentity／Ready SHA／期限、cancel不在、RunOwnerとSupervisorの両PID＋開始時刻を再確認する。先に来たACK、別case/旧runのACK、重複ACKは拒否する。ACK後であってもspawn直前のsource/G0/marker/EXE照合が必須である。子は対象EXEの独自FileShare.Read handleを取得し終了／回収まで保持するため、Supervisor異常終了で全体lockが閉じても動作中imageを差し替えられない。
4. spawn直後から各250ms loopで親identity／cancel／phase期限と直接子を監視し、G0ではSupervisorも並行して同じ条件を監視する。SafetyWorkerはV2CaseのWaitForExit(65秒)や引数なしWaitForExit()を呼ばない。毎回の非blocking HasExited又は最大250msのWaitForExit(int)で進み、終了時にも期限/cancelを先に判定する。G0完了はfull待ち時計を再開しない。

ACK前にowner又は子が終了した場合はCargo／V2Case／target起動0である。ACK後、子のStart直後からRegistration公開までの窓は「登録済み」と偽らない。ownerはspawn意図、手元のProcess handle、既知親PID＋開始時刻＋終了時刻、実commandと専用OutputDirectoryを使って生存直系子を照合する。worker喪失時はSupervisor、Supervisor喪失時はRunOwnerが同じ照合を引き取る。死亡親のPIDを再利用した別processからの子は、その親の実生存区間に生成されていないため所有へ取り込まない。既知PIDの再利用時もstart time不一致なら停止対象から除く。

子孫探索にはWindows `Win32_Process`のProcessId／ParentProcessId／CreationDate／ExecutablePath／CommandLineを使い、Process handleのStartTime／ExitTimeと突き合わせる。監督loopで観測済みedgeを保存し、既知親が消えても子の同定を捨てない。V2Case直系は専用OutputDirectoryをcommandに持ち、Runnerは固有InvocationPath、nativeはpayloadのentryPidとCargo→entry→Runnerの実親子鎖、CargoはRun固定commandで識別する。EXE名や時間帯だけの一致を所有根拠にしない。初回から観測不能・権限不足・中間親が消えてedgeを証明不能であれば`CleanupIncomplete`とし、全体PASS／次case／fullを禁止する。親喪失を検出しただけで子孫回収済みとは記録しない。

回収手順のownerは、通常のcaseではSafetyWorker、fullではRunner、全run abortではSupervisor、Supervisor異常時ではRunOwnerである。SafetyWorker／Runner自身もRunOwner又はSupervisor喪失を検出すると自分のtree回収へ入る。entryが強制終了した場合もRunnerは登録済みentry PID＋開始時刻の喪失から回収する。複数ownerが同時にabortを観測してよく、保存先をowner別にして上書きせず、同じOS identityへの停止は既終了を成功として扱う。cancelはrun rootへのCreateNew一度公開、解除はない。

各回収は最初のabort時点から10秒の単一Stopwatch期限とする。まず新規spawnを禁止し、所有identityを再確認して記録済み子孫を葉から順に`Process.Kill()`する。新規子を作り得る既知ownerにも停止要求を送り、250ms以内のWaitForExit(int)で各handleを確認しながら残り時間内だけ再列挙する。Kill要求成功をwait成功に読み替えない。[.NETのKill契約](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.kill?view=net-10.0)でもkillは非同期であり、親のWaitは子孫終了を保証しない。したがって引数なしWaitForExit()、processごとの新しい10秒、Kill(true)だけで完了扱いする実装は禁止する。既知processに対する単体Killと全handleの有限Wait／残存再照合を同ps1の共通helperに固定する。OS列挙の各queryも残り時間以内の有限timeoutにする。

Supervisorのabort/cancelを受けたRunOwnerは同じcleanupDeadlineを共有し、10秒を積み増ししない。Supervisorが通知前に異常終了したときはRunOwnerが検出時点で10秒を開始する。各workerも上位のcleanupDeadlineがあれば自分の期限との早い方を採る。10秒内に対象全handle終了・所有子孫残存0を証明できなければ非zero `CleanupIncomplete`を保存して待機を終え、fixture削除とfullを禁止し、未回収identityを親へ返す。OS停止要求自体が応答不能の場合を実時間10秒で必ず回収できるとは主張しない。

保証対象は少なくとも一つの監督が生存してOS照合／停止APIが応答する障害である。RunOwner／Supervisor／workerを同時に強制終了する、OS自体が停止する、観測前に中間processが消え所有鎖を証明不能となる場合に、Jobを使わず全子孫停止を無条件保証しない。その場合も承認の自動継続／次runへの流用／不明processのkillはない。必要な合成対照で単独監督喪失やACK境界の回収が成立しない場合はPre/Post不合格として親/plannerへ戻す。新Job／別launcher／品質条件の変更を実装者が選択する余地はない。

## 独立テスト・handoff

作者はscriptの実装を期待値にせず、fakeが自身のargs_osから書くargvUtf16/起動marker/stdout/stderr/exit/子PIDをoracleにする。通常Unicodeの表示用UTF-8は補助であり、未対surrogateのoracleへ使わない。`fake.rs`と`cargo_gate_entry.rs`は独立した標準libraryのみのRust sourceであり、各々ignored内へrustcで構築する。test入口のGateScriptから同じdirectoryのcargo_gate_entry.rs／cargo-gate-entry.exeを一意に解決し、実装sourceと生成EXEのSHA／rustc実path/versionを証拠へ含める。nativeのbuildは `rustc --edition=2021 <R/cargo_gate_entry.rs> -o <R/cargo-gate-entry.exe>`だけであり、生成前にそのEXEを使う生存runがないことを確認する。製品Cargo.lock／本体sourceの変更はない。必要なCargo接合fixtureはRの合成directoryに最小Cargo.tomlを生成し、製品Cargo/依存へ触れない。製品への初回適用前に実pwsh7.6.6＋実Cargoによる接合を通す。

| 独立case群 | 必須証拠 |
| --- | --- |
| G1 ENV/PATH | runner値がnative絶対path一語、spacesを含むpwsh installationで正しい実path/hash、子だけのrunner、固定env/cwd/JOBS、CargoのDLL PATH継承、親env/global config前後不変 |
| G2/G3 GATE | 承認なし/deny/欠損・破損/旧run/期限切れ/未知EXE/profile・features・source・marker違いは起動marker0。manifest欠落/partial JSON/build-finished遅延・false/重複ID・phase承認も拒否。新invocationIdによる同EXE/argv再呼出は毎回照合して実行可能。親fullより前のCargo側起動0 |
| G3 RACE | hash取得後・spawn前のEXE差替えはlockで拒否、承認差替え・実行直前のsource変更は再照合でgate非zero・起動0を観測する。FileShare.Read保持下で正常fake起動可能、同path差替え不可を実証する。source全期間lockを期待値にしない |
| G4 TRANSPARENT | 空文字、日本語＋U+3000、非BMP／孤立surrogate、空白、二重引用符、末尾backslash、`--`/`-x`/未知option、`-Verbose`/`-ErrorAction`とその値、複数空文字をnative→PS→fakeのUTF-16 exact配列で比較。payloadの型/範囲/先頭EXE異常は拒否。stdout/stderr bytes、exit0/7/101/0x80000001/0xFFFFFFFFを比較し、pwsh -File境界も通す |
| G5 LIFETIME | 承認待ち/子実行timeout、G0中cancel／aggregate期限、RunOwner／Supervisor／SafetyWorker／native entryの個別強制終了、ACK公開前／後／V2Case Start後登録前を合成する。残ったownerの回収、10秒単一wait予算、full0、同じPIDの別開始時刻拒否、無関係sentinel不変を確認する。所有不明／回収不能対照はCleanupIncomplete＋非zero＋fixture保持でありPASSへ換算しない |
| G6 ORDER | fake G0が未完了/失敗/旧Manifestならfullなし、登録ACK前の対象起動0、同run G0成功後だけ一括解除。g0→全case→親full確認まで同じ600秒であり最終case後も延長0。複数artifactで後続照合とargv保持、full対象不減を実Cargoの小fixtureで確認 |

検証入口は `& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File R/Test-CargoGate.ps1 -GateScript R/Invoke-CargoGate.ps1 -OutputRoot R/verification/<new-id>`（Rは上記絶対pathへ置換）と固定する。作者がcase実名/件数を先に確定し、runnerは同commandを新しい証拠先で実行する。テストseamはRun／EntryPayload／親承認／Ready・ACK・case開始sidecarとsynthetic processであり、製品sourceへのseam追加や実V2Caseの改変をしない。race caseは観測したsidecarで指定ownerを停止し、登録直後の窓は作者fakeの同期点で再現する。caseごとの有限時間対照は合成Runで短い絶対期限を使うが、製品の3600/1800/600/65/45/10秒を変えない。implementer/作者はPowerShell作成skill、Post reviewerはPowerShell review skillを計画どおり読む。本設計はスクリプトを生成しない。

標準静的検査は固定pwsh上、Rをcwdとして両ps1を対象にする。各fileで`$tokens=$null; $parseErrors=$null; [System.Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$parseErrors)`のerror0を確認する。analyzer commandは`Import-Module PSScriptAnalyzer -RequiredVersion 1.25.0; @('./Invoke-CargoGate.ps1','./Test-CargoGate.ps1') | ForEach-Object { Invoke-ScriptAnalyzer -Path $_ }`とし、全指摘を保存・修正又は根拠付き判定する。Pester5.7.1は導入済みだが必須framework追加とはせず、作者の固定case入口を維持する。新規インストール/既存設定変更はない。

製品Cargo argvは既定5種のtest commandへ`--message-format=json`だけを追加する：通常/試用`test --all-targets --locked [--features early-trial]`、acoustic-analysis/qwen-audioそれぞれ`test --manifest-path tools/<crate>/Cargo.toml --all-targets --locked`、qwen-audio `test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked`である。通常→試用→関連を逐次実行し各Runを独立承認する。relatedは新artifactに対する静的安全根拠と該当するG0の有無を親が明示し、wordweave5のG0を別EXEへ流用しない。release3件の取得済み証拠、fmt、native、提供Postは既存計画へ接合する。

不採用は別no-run反復（次の再buildへSHAを持ち越せない）、EXEごとの人間承認（同じ集合の照合を重複）、先にfull起動して後からcounter確認（安全順が逆）、単なるspawn直前hashだけ（hash後差替えraceが残る）である。Pre是正ではPSへの任意argv直接渡し（binderが内容を消費）、nativeとfakeの共有（oracle独立性喪失）、親別treeのV2Case起動（aggregate監督に属さない）、V2Case直接同期呼出（無期限Waitを外側へ持込）を採用しない。Windowsのlock/argv/子孫回収は未実証であり独立fake対照を受入条件とする。満たせない場合は親/plannerへ戻し、追加Job等の機構、権限・gate・対象を黙って変えない。

次はplanner同期と独立Pre-2→独立作者→実装者→独立runner→Postである。同期事項は4file所有、RunOwnerを含む固定mode／parameter宣言、native runner一語とPATH追加撤廃、G0をSupervisor直系SafetyWorkerへ移すこと、登録ACKと単一cleanup期限である。designer／作者／実装者／reviewerはAstra/high、runnerはSol/mediumの指定を維持する。未決の方式選択を実装者へ渡さず、本書方式の独立Preと動的実証を要求する。返却する設計証拠は本書のSHA、上記source/helper/旧証拠への対応、文書diff/link検査である。

本修正ではV2Case全文・共有isolated helper全文・ET2-G0を再確認し、V2Caseの無期限Wait／helperの45秒と単体kill／固定markerを根拠に外側責任を具体化した。製品source全体の静的G0再認証を代行したものではない。native UIの安全な観測手段は別runner所有であり、本書で解決済みにしない。新たな利用者動作の未決は0、独立Pre／Windows実証は未了である。

測定は子 `/root/et3_gate_design_fix`、指定Astra/high。開始時counterは未取得、作業途中22:51:44 JST近傍の共有週枠は70%／10080分／reset1792496608である。他作業並行のため本工程の消費率に換算しない。固有session/provider返却model/effort・token counter・親子包含は未取得であり推定しない。旧design担当の測定欠測も解消したと扱わない。
