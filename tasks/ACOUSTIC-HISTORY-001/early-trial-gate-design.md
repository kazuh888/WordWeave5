# ET3-G：同一Cargo実行のEXE承認境界

2026-10-14。Pre-1のP1二件を反映した本書SHA `CC72A435CAF840E94F06F5704D95822BDAF57A19C10A91E579CE3ACB062E8313`は独立Pre-2 PASSとなった。現在の追加は末尾「作者handoff後の局所具体化」だけであり、キー名確定と合成fixtureの同期／観測契約を区別する。初回入力は[計画](early-trial-gate-plan.md) SHA256 `EDA787512F47F24A360C068ED3C78C38C2BD1B687DA7C37869DC0D5F397C0E1F`末尾の是正方針、[Pre指摘正本](early-trial-delivery-review.md)冒頭、[承認済みET契約](early-trial-contract.md)と同reviewの利用者承認である。指定designerは `gpt-6-astra/high`。今回process限定runner導入だけを具体化する。製品sourceは親受領のcb882031同体、HEAD6fb6da3。初回着手時git statusで既存dirty文書8件と未追跡の本書／計画を確認した。所有は本書だけ、実装・テスト編集・runtime・再委譲・公開は0である。旧設計SHA `92E370C131AAF46201BD9D575ADEC1DC0A8E119450587D8DB33966043B07A09C`へのPre不合格は履歴として保持し、製品runtimeを本書だけで解除しない。

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

nativeのこの非上書き公開はRust `std::fs::rename`ではなく、Kernel32の`MoveFileW` FFI（`extern "system"`、戻りBOOL）に固定する。所有する同directoryのCreateNew tempへ全bytesを書き`sync_all`後に閉じ、検証済み絶対pathを内部NULなしのUTF-16＋末尾NULへ変換して呼ぶ。temp／宛先の解決済み親directory一致を必須とし、事前の不存在確認だけで競合を防いだことにしない。[MoveFileW公式契約](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefilew)の「新nameは既存不可」を公開境界に使い、BOOL=0では直ちにGetLastErrorを保存して非zero終了する。失敗時に処理できるのは自分のtempだけであり、既存payload／宛先は変更・削除しない。hard_linkによる複数link状態、置換許可付き別API、copy fallbackは追加しない。標準Windows APIへのFFIだけで外部Cargo crateは0、既定の完成file・非上書き公開を実現するprimitiveの具体化であり新承認はない。

## 時間順序・状態遷移

1. `Prepared`：親が静的G0、source170/G0 12/marker4、Runのcommand/env/予定集合を照合し、全source writer停止・同target別buildなしを宣言する。独立runnerが固定pwshで同ps1を `-Mode RunOwner -RunPath <run.json>`として一度起動し、その実processが外側監督となる。RunOwnerは通常ユーザー・固定入力・全wrapper hashを確認し、直系SupervisorをHidden起動／登録し開始ACKを出す。SupervisorはACK前Cargo起動0。source170は全期間lockせず、build前後、親承認時、各EXE実行直前と終了時に全bytes hashを照合する。ソース変更を伴わない凍結検証であり、変更検出/所有違反はrun全体を無効化する。
2. `Building`：Cargoだけは固定5commandの既知token（空白・引用符・空文字なし）を単一ArgumentListへ連結し、`Start-Process -FilePath <固定Cargo絶対path> -ArgumentList <固定command文字列> -WorkingDirectory <repo> -Environment <Runの子EnvMap> -WindowStyle Hidden -PassThru -RedirectStandardOutput <run/stdout.log> -RedirectStandardError <run/stderr.log>`で起動する。-Waitを付けず、250msの既存型監督loopでlive stdoutをFileShare.ReadWriteにより読取る。前回offset＋未完行だけを保持し、改行まで揃ったUTF-8行だけJSON解析する。縮小/置換/不正行、Cargo終了前に未完のまま期限切れなら拒否する。最初の`build-finished success=true`までのfile bytesをprefix固定し、その後のtest出力は解析しない。false/予定集合不一致は全解除禁止、fresh=trueも同run artifactとする。stderrは既存redirectのまま保存し、独自pipe排出器を追加しない。
3. `AwaitingManifest`：Cargoが呼ぶ最初のRunnerはRequestを出して停止する。build-finishedが未到着ならSupervisor側収集を待つ。JSONがrunner起動より後に届いても非実行を保ち、期限までに全prefixが取れなければ中断する。stdout収集をrunner待機で塞がない。build-finished前のbuild-script実行は通常Cargo buildの既承認範囲であり、ここで保証する「起動0」は対象test EXEについてである。
4. `AwaitingG0`：Supervisorは全EXEを読取lockし同handleからhashを取り、source再照合後Manifestを公開する。親はraw JSON prefix/source/EXE/profile/features/target・予定集合・first Requestを独立照合し、g0承認を発行する。manifest内に載るだけでは起動許可にならず、full Runnerはここでも待つ。承認前のEXE `--list`も0である。
5. `SafetyChecks`：Supervisorがg0承認に束縛されたSafetyWorkerを一件ずつ直系子として起動する。worker登録／開始ACK後だけ同Manifestの新binとcodex_processを既存V2Caseでlistして実名集合を照合し、その後新binの録音3→P01既定順20→新integration自身A11を実行する。Supervisorとworkerは各V2Case起動直前にもsource170/G0 12/marker4を再照合し、V2CaseのEXE hash確認と組にする。親はこの固定対象／順序をg0発行時に照合し、別tool treeで起動しない。順序の入力は既存`fx1-default-selected.log`先頭23行（試用も同23）と`fx1-integration-selected.log`全11行をsource実名と照合しRunへ固定する。新binの旧G0/PASSを接合しない。V2CaseのTimeoutSeconds=65、内部helperの既定45秒は変更せず、workerの外側case時計はV2CaseのStart直前から65秒、aggregate残りが短ければ外側が先に止める。exact1/PASS1/必要完了marker/AdminFalse/残存0を必要とする。case結果・回収完了前には次workerを開始しない。親が同run全ログとG0結果digestを独立照合するまでfullを発行しない。
6. `RunningFull`：親が同run/Manifest＋新G0結果digestを束縛したfull承認を1枚発行する。Supervisorが期限内に検証・受領した証拠を下記full-received.jsonへ公開してから、待機中Runnerと後続Runnerが同full承認を共用する。先行Runnerもreceipt前target0である。各spawn直前にreceiptと承認の対応、有効期限・cancel不在・live RunOwner/Supervisor/Cargo/entry identity・Run/Manifest/承認hash・source/G0/marker一致・対象EXE hash・許可集合・当該Runner登録ACKを再照合する。EXE読取lockを終了まで保持し、変更できなかったという推測だけでhash照合を省略しない。構成ごと2枚の親承認であり、24件の人間待ちは設けない。
7. `Finished`：Cargo、全Runner、G0監督、既知子孫の終了/Waitを確認し、source/marker/旧製品の保全とfixture残存0を記録してからlockを解放する。成功判定にはCargo exit0だけでなく全gate Result整合、予定実行集合との対応、実test集計を必要とする。24artifact全部が起動されるとは要求しない。未起動のprofile.test=false補助binと対象testを区別し、件数減を自動容認しない。

任意状態の拒否・変更・取消し・期限切れ・親/supervisor喪失は`Aborting→Stopped`だけへ進む。旧runの再開・自動承認・自動再試行はない。Run.deadlineUtcは**回収を含む全体停止上限**であり、Run.startedUtcから最大3600秒のままとする。全mode共通の実行停止境界`executionDeadlineUtc = Run.deadlineUtc - 10秒`を導出して末尾10秒を回収へ予約する（Runへ新しい入力fieldは足さない）。build1800秒、各承認待ち600秒、G0 aggregate600秒、full各EXE600秒、V2Case65秒の各実行／待機期限は、このexecutionDeadlineUtcとの早い方を採る。各phase自体からさらに10秒を差し引く意味ではない。期限到達で新規spawn／承認採用／待機を止め、回収へ移る。時間延長は親の別Run判断であり待機中の期限を書き換えない。

ACK、Intent、SafetyInvocation、fixture停止点のdeadlineは実行／待機側の期限、Result.cleanupDeadlineUtcは回収側、Run.deadlineUtcだけが両者を含む全体上限である。全modeはUTC比較と初回に残り時間から設定したStopwatch上限の早い方で停止し、時計逆行や監督交代で予約を失わない。Run検証／child起動前にexecutionDeadlineUtcへ既に達していれば対象起動0で返す。短いfixtureRunも同じ10秒予約を持ち、作者が必要な起動・hook・phase対照時間に回収10秒を加えた絶対期限を明示する。残り10秒以下のRunを通常対照として実行可能に自動延長せず、短いphase期限によるtimeout対照は十分な全体Run内に置く。

600秒はaggregateの失敗上限であり、34exact×65秒=2210秒（list2件を含めれば最大2340秒）を全件使っても完了する保証ではない。g0承認の親記録publishedUtcからfull承認受領までの600秒を固定する。親はg0公開時点で`g0DeadlineUtc=min(publishedUtc+600秒, executionDeadlineUtc)`も承認へ含め、Supervisorは将来のpublishedUtcを拒否して同期限を固定する。観測が遅れても時計を後ろへずらさない。期限切れでは実行中G0を所有照合して回収しfull起動0を保つ。G0とfull承認待ちの時計を二重にリセットしない。各監督はUTC残り時間から作るStopwatch上限も持ち、時計の逆行による延長を許さない。fakeはこのaggregate期限切れを独立検証する。全case上限を許す時間枠へ変える必要が生じた場合は親/plannerへ返し、自動延長・case短縮・省略はしない。

## process・argv・回収境界

- `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER`は上記native絶対path一語だけであり、pwsh探索用の追加PATHは不要である。Start-ProcessのEnvironment.PATHが持つPSHOME／Machine/User PATHの実効挙動は固定pwsh7.6.6で記録する。各PS modeは自身の実process path/version/SHAとRunの固定値を照合し、nativeは指定のpwsh絶対pathを直接使う。Cargo runner pathに空白が入る変更は今回対象外として拒否し、aliasやglobal設定へ進まない。
- LOCALAPPDATA/TEMP/TMP、target、JOBS2、repo cwd、WW5_ET2_CHILD/BASE空、既定CARGO_LOGはfull2の構成別EnvMapを使う。runner値とrun path/idだけをCargo子環境へ追加する。親環境を`$env:`で書き換えない。Cargoが実行時に加えるライブラリPATH等はnative→PS→targetへ保持する。環境照合は固定keyとrunner実pathを対象とし、CargoによるPATH追加を誤って違反扱いしない。
- Runnerはpayloadの先頭からcanonical EXEを検証し、残り全argvを順番・空文字・UTF-16 code unit・空白・引用符・末尾backslashを含めProcessStartInfo.ArgumentListへ1要素ずつ渡す。shell、Invoke-Expression、Start-Processの任意argv文字列連結は使わない。cwd/envを継承しUseShellExecute=false/CreateNoWindow=true、stdout/stderr/stdinをOS handleとして継承する。PS pipelineを経由せずbyte列を保ち、wrapperログやnativeエラーはsidecarのみへ出す。filter/skip/ignore/test-thread/quietを付加しない。
- 通常完了時はchild.ExitCodeのInt32をPSの明示`exit`へ渡し、nativeはExitStatus.code()のInt32を`std::process::exit`へ渡す。符号を成功判定へ転用せず、同じ32bit patternを保持する。[pwsh -Fileのexit規則](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_pwsh?view=powershell-7.6)を使い、0/7/101/0x80000001/0xFFFFFFFFを実native→PS→fakeで確認する。通常終了のResultはgateStatus=`Completed`、製品exitはそのままである。gate拒否/timeout/cancel/監督障害は非zero＋`Rejected`/`TimedOut`/`Cancelled`/`OwnerLost`/`CleanupIncomplete`で区別する。製品の同じ数値exitと混同せず、sidecar欠落はunknownであってPASSではない。
- 各ownerは開始意図を先に保存し、spawn直後のProcess handle／PID／開始時刻を保持し登録する。正常exitでも子孫残存を監査する。OSのPIDだけでなく同じ開始時刻・image・command・親子関係を確認する。自身がdirect Startした子の所有根拠は下記DirectLaunchHandleでも証明できるが、起動入力をOS照会値へ転記せず、未知processを停止しない。有限kill/waitの実体と登録raceは次節で固定する。
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

直系子については`DirectLaunchHandle`を独立した所有根拠として認める。条件は同じownerが(1)atomic Intentを保存、(2)承認済み絶対path／image lock／正確なArgumentListを使いUseShellExecute=falseで直接Start、(3)その呼出が返したProcess／SafeHandleをPID照会から作り直さず保持、(4)そのhandleのPID／StartTimeと、終了時のExitTime／ExitCode／有限wait結果を記録する、の全成立である。即exitしてCIM行が消えても、この証拠で当該直系子のOwnedと終了を判定できる。handle時刻／waitが取得不能なら推定せずUnprovableであり、Intentだけでは所有を証明しない。通常のlive childでもOS照会が取得できたfieldは記録し、実測値とIntentが矛盾すれば拒否する。

最小追加fieldはRegistrationとResult.processes行の`ownershipEvidence:{kind,intentSha256,launcherIdentity,exitTimeUtc,osQueryStatus}`である。kindは`OsQuery`又は`DirectLaunchHandle`、osQueryStatusは`Observed,NotFoundAfterExit,Unavailable`。DirectLaunchHandle時のidentity.pid/startTimeUtcは保持handle由来、未観測のidentity.path/parentPid/commandLineUtf16はnullとする。実際にAPIへ渡したpath／argv／親はintentSha256の指すIntentとlauncherIdentityから読み、OS実測値として埋めない。OsQuery時はintentSha256をnull可、exit前のexitTimeUtcもnullとする。保持handleを持つ起動ownerだけがその同一handleへ停止／waitでき、PID再利用やsidecar読取を理由に他processをDirectLaunchHandleへ昇格しない。他監督の引継ぎと未知descendantは既定のOS identity／親子edge照合を必要とする。

この規則は直系子の短寿命を扱うものであり、子孫残存0・ACK・親livenessを代替しない。合成LIFE16/17のnegative観測境界も通し、保持handleを理由にその欠測を成功へ書き換えない。CIMのsandbox access拒否はUnavailableとして別途記録し、未知子孫の監査不能を短寿命扱いでPASSにしない。ordinary環境での照会能力確認は後続runnerが行い、本変更でquery機構を置換しない。一次根拠は[Process.SafeHandle](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.safehandle?view=net-10.0)のnative handle、[CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)の作成process情報の返却、[GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)のhandleに対応する作成／終了時刻である。別APIで即exit後のcommand lineを必ず復元できるとは仮定せず、TRANSPARENTの実証は保持handleによる終了と独立fakeのargv／stream／exit証拠を組み合わせる。

回収手順のownerは、通常のcaseではSafetyWorker、fullではRunner、全run abortではSupervisor、Supervisor異常時ではRunOwnerである。SafetyWorker／Runner自身もRunOwner又はSupervisor喪失を検出すると自分のtree回収へ入る。entryが強制終了した場合もRunnerは登録済みentry PID＋開始時刻の喪失から回収する。複数ownerが同時にabortを観測してよく、保存先をowner別にして上書きせず、同じOS identityへの停止は既終了を成功として扱う。cancelはrun rootへのCreateNew一度公開、解除はない。

各回収の共有絶対期限は`cleanupDeadlineUtc = min(firstAbortUtc + 10秒, Run.deadlineUtc)`である。firstAbortUtcは最初の既知abort（cancelが先ならcancel.publishedUtc）とし、各owner自身のabort観測からのmonotonic10秒上限も併用する。まず新規spawnを禁止し、所有identityを再確認して記録済み子孫を葉から順に`Process.Kill()`する。新規子を作り得る既知ownerにも停止要求を送り、250ms以内のWaitForExit(int)で各handleを確認しながら残り時間内だけ再列挙する。Kill要求成功をwait成功に読み替えない。[.NETのKill契約](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.kill?view=net-10.0)でもkillは非同期であり、親のWaitは子孫終了を保証しない。したがって引数なしWaitForExit()、processごとの新しい10秒、Kill(true)だけで完了扱いする実装は禁止する。既知processに対する単体Killと全handleの有限Wait／残存再照合を同ps1の共通helperに固定する。OS列挙の各queryも残り時間以内の有限timeoutにする。

Supervisorのabort/cancelを受けたRunOwnerは同じcleanupDeadlineを共有し、10秒を積み増ししない。Supervisorが通知前に異常終了し既知の先行abortがなければRunOwnerの検出時点をfirstAbortUtcとするが、Run.deadlineUtcの上限は維持する。各workerも上位のcleanupDeadlineがあれば自分の期限との早い方を採る。通常のRun timeoutはdeadlineそのものではなくexecutionDeadlineUtcで発火するので回収予約を使える。スケジューラ停止等で検出が遅れ残りが0なら、保持する既知所有handleへの未送信Killを直ちに要求し、待機を追加せず未確認分をCleanupIncompleteとして返す。「0秒だから停止要求も省く」は禁止する。期限内に対象全handle終了・所有子孫残存0を証明できなければ非zero `CleanupIncomplete`を保存して待機を終え、fixture削除とfullを禁止し、未回収identityを親へ返す。OS停止要求自体が応答不能の場合を全体上限までに必ず回収できるとは主張しない。

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

検証入口は `& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File R/Test-CargoGate.ps1 -GateScript R/Invoke-CargoGate.ps1 -OutputRoot R/verification/<new-id>`（Rは上記絶対pathへ置換）と固定する。作者がcase実名/件数を先に確定し、runnerは同commandを新しい証拠先で実行する。テストseamはRun／EntryPayload／親承認／Ready・ACK・case開始sidecarとsynthetic processであり、製品sourceへのseam追加や実V2Caseの改変をしない。作者fakeだけではspawn前／登録前を停止できないため、その窓は末尾の限定同期契約で再現する。caseごとの有限時間対照は合成Runで短い絶対期限を使うが、製品の3600/1800/600/65/45/10秒を変えない。implementer/作者はPowerShell作成skill、Post reviewerはPowerShell review skillを計画どおり読む。本設計はスクリプトを生成しない。

標準静的検査は固定pwsh上、Rをcwdとして両ps1を対象にする。各fileで`$tokens=$null; $parseErrors=$null; [System.Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$parseErrors)`のerror0を確認する。analyzer commandは`Import-Module PSScriptAnalyzer -RequiredVersion 1.25.0; @('./Invoke-CargoGate.ps1','./Test-CargoGate.ps1') | ForEach-Object { Invoke-ScriptAnalyzer -Path $_ }`とし、全指摘を保存・修正又は根拠付き判定する。Pester5.7.1は導入済みだが必須framework追加とはせず、作者の固定case入口を維持する。新規インストール/既存設定変更はない。

製品Cargo argvは既定5種のtest commandへ`--message-format=json`だけを追加する：通常/試用`test --all-targets --locked [--features early-trial]`、acoustic-analysis/qwen-audioそれぞれ`test --manifest-path tools/<crate>/Cargo.toml --all-targets --locked`、qwen-audio `test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked`である。通常→試用→関連を逐次実行し各Runを独立承認する。relatedは新artifactに対する静的安全根拠と該当するG0の有無を親が明示し、wordweave5のG0を別EXEへ流用しない。release3件の取得済み証拠、fmt、native、提供Postは既存計画へ接合する。

不採用は別no-run反復（次の再buildへSHAを持ち越せない）、EXEごとの人間承認（同じ集合の照合を重複）、先にfull起動して後からcounter確認（安全順が逆）、単なるspawn直前hashだけ（hash後差替えraceが残る）である。Pre是正ではPSへの任意argv直接渡し（binderが内容を消費）、nativeとfakeの共有（oracle独立性喪失）、親別treeのV2Case起動（aggregate監督に属さない）、V2Case直接同期呼出（無期限Waitを外側へ持込）を採用しない。Windowsのlock/argv/子孫回収は未実証であり独立fake対照を受入条件とする。満たせない場合は親/plannerへ戻し、追加Job等の機構、権限・gate・対象を黙って変えない。

次はplanner同期と独立Pre-2→独立作者→実装者→独立runner→Postである。同期事項は4file所有、RunOwnerを含む固定mode／parameter宣言、native runner一語とPATH追加撤廃、G0をSupervisor直系SafetyWorkerへ移すこと、登録ACKと単一cleanup期限である。designer／作者／実装者／reviewerはAstra/high、runnerはSol/mediumの指定を維持する。未決の方式選択を実装者へ渡さず、本書方式の独立Preと動的実証を要求する。返却する設計証拠は本書のSHA、上記source/helper/旧証拠への対応、文書diff/link検査である。

本修正ではV2Case全文・共有isolated helper全文・ET2-G0を再確認し、V2Caseの無期限Wait／helperの45秒と単体kill／固定markerを根拠に外側責任を具体化した。製品source全体の静的G0再認証を代行したものではない。native UIの安全な観測手段は別runner所有であり、本書で解決済みにしない。新たな利用者動作の未決は0、独立Pre／Windows実証は未了である。

測定は子 `/root/et3_gate_design_fix`、指定Astra/high。開始時counterは未取得、作業途中22:51:44 JST近傍の共有週枠は70%／10080分／reset1792496608である。他作業並行のため本工程の消費率に換算しない。固有session/provider返却model/effort・token counter・親子包含は未取得であり推定しない。旧design担当の測定欠測も解消したと扱わない。

## 作者handoff後の局所具体化（Pre-2 PASS後）

入力は作者`Test-CargoGate.ps1` SHA `424207EC8058DE0719384A5700611B0E4DEBEF39F973B44B18383712B9B46F5A`の80case／schemaCandidateと、`fake.rs` SHA `46FA412A04F6151AFB2C23B0259C72E2549C48686A70AAB7CB744568825A90DE`全文である。fake自己検証7/7は親受領証拠であり、本担当は再実行していない。以下の名前／型は既存semanticの具体化、fixtureControlによる停止／観測障害は合成試験専用の新しい内部semanticである。Pre-2 PASSをこの追加への判定として遡及適用しない。80caseの期待値・件数・期限・4file所有を維持する。

### 1. JSON実キーと証拠名

作者候補のRunキーを次の型で確定する。大文字小文字を区別し、旧別名を受け付けない。`runId,cfg,repo,cargoVersion,pwshVersion,startedUtc,deadlineUtc,buildDeadlineUtc`は文字列、`cargoArgv`は文字列配列、`envMap`は固定key→文字列map、`cargo,sourceInput,g0Input,markerInput,gateScript,nativeSource,nativeExe,pwsh,v2Case`は`{path,sha256}`、`sourceFiles,g0Files,markerFiles`は同shapeの配列、`expectedArtifacts,safetyCases`は下記配列である。時刻はUTCの`yyyy-MM-ddTHH:mm:ss.fffffffZ`、identityの開始時刻は実Process handleのStartTimeを基準とし、CIMの低精度表示から捏造しない。

| 対象 | 確定するfield／照合 |
| --- | --- |
| cfg | 製品は`default`,`early-trial`,`acoustic-analysis`,`qwen-audio`,`qwen-audio-no-default`の5値。順に本書の既定5Cargo commandへ一対一対応。合成試験だけ`fixture`を追加し、下記閉じたfixture境界を必須とする。任意の新cfgは拒否 |
| source/g0/marker digest | sourceInput等はそのrun用に保存するUTF-8 JSON配列のPathHash snapshot。各fileの**保存済み全bytes SHA**を`sourceDigest/g0Digest/markerDigest`とし、再serializeしない。decodeした順序・path/hash集合はRunの対応配列と一致必須。source/g0は全file bytes、markerは一意の`// BEGIN ET2-P01 AUTHOR TESTS`先頭`/`から`// END ET2-P01 AUTHOR TESTS`末尾`S`までの現行UTF-8 bytesをhashする。旧marker snapshotの値を復活させない |
| expectedArtifacts | `{packageId,manifestPath,targetName,targetKind,profile,features,allowExecution}`。targetKind/featuresは重複なしordinal順配列、profileはCargo JSONの実field名と型を保持し構造比較する。Manifestのartifact行はこれに`executable:{path,sha256}`を加える。allowExecution=trueだけが候補であり親承認は別途必要 |
| safetyCases | `{caseId,order,artifactKey,list,exactName,requireChildMarker,expectedListNames}`。orderは0開始の連続整数、artifactKeyは`{packageId,manifestPath,targetName,targetKind}`、caseIdはrun内一意のASCII英数／`-`／`_`。list=trueのexactNameは`::`、expectedListNamesは作者固定実名集合。exactはexpectedListNames=[]。製品のlist2＋録音3→P01 20→integration11を変更しない |
| process identity | `{pid,startTimeUtc,path,parentPid,commandLineUtf16}`。前3fieldは作者候補を採用し、OS親子照合に後2fieldを加える。commandLineUtf16はOSのcommand line全体のcode unit配列であり、target argvとは区別する。identityの等価性はPID＋StartTime、path/command/parentは追加の所有根拠である。DirectLaunchHandleの未観測fieldだけは上記規則でnullとし、ownershipEvidenceとIntentに起動入力の由来を残す |
| Approval | 作者候補の`phase,decision,runId,runSha256,manifestSha256,cfg,sourceDigest,g0Digest,markerDigest,publishedUtc,expiryUtc,g0DeadlineUtc,recordReference,safetyEvidenceSha256`を採用。recordReferenceは非空文字列。g0のsafetyEvidenceSha256だけnull、fullでは同runの成功証拠file SHA必須。fullの受領はg0Deadline以前、実行有効期限expiryUtcはRun期限以下であり、G0期限経過後に全full実行を直ちに失効させる意味ではない |
| FullReceipt／Supervisorのみ | run rootの`full-received.json`へ`{runId,runSha256,manifestSha256,fullApprovalSha256,supervisorIdentity,receivedUtc}`をimmutable公開する。fullApprovalSha256は検証したapproval-full.jsonの全bytes SHA、supervisorIdentityは登録済み実identity。receivedUtcは全G0結果／full承認検証が成功した実受領時刻であり、full.publishedUtcの転記ではない |
| G0成功証拠 | Supervisorが`safety-evidence.json`へ`{runId,manifestSha256,cfg,cases}`を保存。casesはRun順に`{caseId,order,invocationSha256,workerIdentity,v2Identity,targetIdentities,result,v2Result,stdout,stderr,listNames,passed,gateStatus}`。result/v2Result/stdout/stderrはPathHash、listNamesはlist時だけ実名集合、exact時[]。全行passed=true／gateStatus=Completed、V2のPassed／exact1・marker・wait／残存0と原文証拠が一致して初めて親full承認へ進む。SHAは保存済み全bytesから取り、順序や件数を再構成して省略しない |

sidecarは作者候補を採用し、run rootの`run.json,manifest.json,approval-g0.json,approval-full.json,cancel.json,safety-evidence.json`、`invocations/<invocationId>/{payload,request,intent,registration,ready,ack,result}.json`、`safety/<caseId>/{invocation,intent,registration,ready,ack,result}.json`に固定する。RunOwner／Supervisor／Cargoの起動情報は`processes/<role>/{intent,registration,ready,ack}.json`、V2CaseのStart意図／登録は`safety/<caseId>/v2-{intent,registration}.json`とする。V2Case専用OutputDirectoryは`safety/<caseId>/v2-output`であり、既存V2の`result.json/stdout.log/stderr.log`はここに保持する。root監督のterminalは`result-<role>-<pid>.json`、worker／Runnerのterminalは各固有directoryの`result.json`である。roleは`RunOwner,Supervisor,Cargo,Entry,Runner,SafetyWorker,V2Case,Target`。同名はCreateNew、既存内容の差替え／第二承認は拒否し、同じimmutable fileの再読だけは重複発行としない。

同run共通sidecarに上記`full-received.json`を加える。Supervisorはfull検証終了時にUTCと既定G0 Stopwatchの両方が期限未到達、cancelなし、G0成功証拠完了、承認expiry未到達を確認してreceivedUtcを取り、直ちに既定atomic保存でreceiptを一度公開する。受領時刻がg0DeadlineUtc以上なら発行禁止でTimedOut、過去のpublishedUtcで救済しない。保存失敗／receipt欠落は実行許可にならず、先行Runnerの既定待機期限を延長しない。再起動してreceiptを復元・再発行する経路もない。

全Runnerはreceiptの全bytes／SHA、run・manifest・full承認SHA、登録Supervisorの同一identityを検証し、`full.publishedUtc <= receivedUtc < g0DeadlineUtc`と受領時の承認有効範囲を確認する。その後は現在時刻に対するfull.expiryUtc／executionDeadlineUtc／各EXE600秒と既定のlive identity・cancel等を適用する。**後続Runner自身の読取時刻をg0Deadlineと比較して共有full承認を失効させない**。欠落／破損／別fullへのreceiptはtarget0、同じfull bytesだけのreceiptを共用する。Runner Resultには`fullReceiptSha256`を追加し、full実行前の拒否では未読ならnullとする。この証拠の追加は期限内受領の可視化だけであり、新しいdeadlineや自動延長、第三の親承認を導入しない。

Manifestのtop keyは`runId,runSha256,cfg,cargoIdentity,sourceDigest,g0Digest,markerDigest,jsonPrefixBytes,jsonPrefixSha256,artifacts,createdUtc`、Requestは`runId,invocationId,cfg,cargoIdentity,entryIdentity,runnerIdentity,payloadSha256,executable,argvUtf16`である。Registration／Readyは`runId,role,scopeId,intentSha256,identity,ownerIdentity`、ACKはこれに`readySha256,manifestSha256,caseId,deadlineUtc`を加え、非caseではcaseId=null、Manifest作成前のSupervisor登録ではmanifestSha256=nullとする。開始Intentは`runId,role,scopeId,ownerIdentity,path,argvUtf16,deadlineUtc`を持つ。SafetyInvocationは`runId,runSha256,manifestSha256,g0ApprovalSha256,caseId,order,executable,exactName,list,requireChildMarker,v2Case,outputDirectory,g0DeadlineUtc,deadlineUtc`であり、executable/v2CaseはPathHash。Resultは本書の既定内容を`runId,role,scopeId,identity,manifestSha256,requestSha256,ackSha256,argvUtf16,childExitInt32,childExitU32,gateStatus,startedUtc,endedUtc,timeout,cancelled,cleanupDeadlineUtc,processes,fixtureResiduals,sourceMatched`へ対応させる。未到達のidentity／hash／exitはnull、未取得配列はnullと空配列（取得済み0件）を区別する。processesの行は`{identity,killRequested,waited,hasExited,ownership,error}`、ownershipは`Owned,Unrelated,Unprovable`である。Unprovable行だけに任意の`osObservation`を追加でき、型・欠測・生成失敗は末尾「所有同定失敗の限定観測」に従う。既存ownershipEvidenceと所有／終了判定を変更しない。

scopeIdはRunOwner／Supervisor／CargoでrunId、SafetyWorker／V2CaseでcaseId、Entry／RunnerでinvocationIdとする。Targetは所有branchを継承し、fullではinvocationId、G0ではcaseIdである（G0に存在しないinvocationIdを新設しない）。processの識別は`runId＋role＋scopeId＋PID/開始時刻`、ACKはさらに既定のintent／Ready SHAへ束縛する。以下のfixture参照は同期対象の後決めだけであり、このlifecycle scopeを変更しない。

### 2. 合成fixtureの閉じた境界

cfg=fixtureではRunへ必須`fixtureControl:{path,sha256}`を加える。それ以外のcfgではfixtureControl、`ET3_FAKE_*`環境、下記hook／fault指定を副作用前に拒否する。合成RunでもRunPathはR/runs/<runId>/run.jsonのままである。descriptorは`{runId,suiteId,fixtureRoot,controllerIdentity,cargoKind,cargoFixture,syncPoints,observationFaults}`、fixtureRootはR/verification/<suiteId>配下の専用directory、cargoKindは`fake`又は`real`である。cargoFixtureは生成Cargo.toml等のPathHash配列（fake時[]）。controllerIdentityは独立作者driverの実processであり、gateの承認権限を製品Runへ移すものではない。

RunOwnerと各childはdescriptor SHA／runId、同rootの`fixture-owner.txt`内容、解決済みpath／reparse境界を検証する。repo、source/g0/markerの現物とsnapshot、LOCALAPPDATA/TEMP/TMP/CARGO_TARGET_DIR、全artifact EXE/manifest、fake cargo実体はこのfixtureRoot内だけを許す。共通Run証拠root、固定gate source/native EXE/pwshと**変更しない既存V2Case**は既存固定path/hashの例外である。case出力の例外も共通Run証拠rootに限定し、既定`R/runs/<runId>/safety/<caseId>/v2-output`とV2Case自身がその下に作る`local,temp,cwd`を許す。V2Caseへ渡すOutputDirectory／子環境はこの固定case所有pathであり、fixtureRootへ再配置しない。その他のfixture作業pathはfixtureRoot内に保ち、別run／case／reparse先への拡張は拒否する。cargoKind=realでは固定実Cargoだけを例外とし、repoと生成workspace／manifest／targetはfixtureRoot内、commandは作者の`test --all-targets --locked --offline --message-format=json`だけとする。fakeは標準library fixture、realは依存なしの作者固定crateを使う。製品repo／source／EXEをfixtureへ指す、junction等で境界を迂回する、fakeを製品cfgで起動する入力は拒否する。

fixtureでもsource/hash／artifact／g0/full承認／登録ACK／期限／OS identity／image lock／回収の同じ本体経路を通す。hookは許可・hash・exit・test結果を生成せず、faultは観測を良好に変えない。製品Runのkey省略や短い時計をfixtureだから自動補完する経路を作らない。合成の対象集合と短い絶対期限は作者がRunへ明示し、既存fakeのmarker／独立OS handleがoracleとなる。

### 3. 決定的同期点（RACE03/04、LIFE09/10/11）

既存Runner／SafetyWorker／Supervisorに同一のprivate helperを置き、descriptor.syncPointsの`{pointId,role,scopeId,hook,deadlineUtc}`に完全一致する一回だけ停止する。hookは次表の閉じた集合である。同期点の到達は`fixtureRoot/control/<pointId>/arrived.json`へ`{runId,runSha256,pointId,role,scopeId,hook,actorIdentity,scopeReferenceSha256}`をatomic公開し、作者driverだけが同directoryの`release.json`へ`{runId,pointId,role,scopeId,hook,arrivedSha256}`をCreateNewする。scopeReferenceSha256は直接scope指定時null、下記参照時は読んだ全bytesのSHAである。到達を見てから変異／指定OS identity停止を行いreleaseするので、sleepの長短や確率的kill反復を使わない。

Runner同期点だけは、nativeが生成するinvocationIdを事前には指定できないため、descriptorのscopeIdを`scopeIdReference`へ置き換えてよい（両方／両方なしは禁止）。参照先は事前固定の`fixtureRoot/control/<pointId>/scope.json`である。controllerは対象Request観測後・対応phase承認前に`{runId,pointId,role:"Runner",scopeId:<invocationId>,requestSha256}`を一度atomic CreateNew公開する。readerは同runの`invocations/<scopeId>/request.json`の実bytes SHA／invocationId／runnerIdentityと照合し、参照bytesとSHAを保持する。下記payload読取前hookだけはRequestがまだ存在しないため、requestSha256を`captureReadySha256`へ置き換え、同scopeのcapture-ready.jsonへ同様に束縛する（両SHAの併記／両方欠落は禁止）。descriptorは更新しない。解決したrole＋scopeId＋hookが現在のactorと完全一致する場合だけ当該点へ到達し、別の有効invocationならその点は対象外である。未知ID／参照差替え／破損／重複keyはRejectedであり、最初に来たRunnerへの自動割当やwildcard照合はない。

参照の公開が遅れ、同role/hookへ先に到達した場合は、arrived／spawnへ進まず既定250ms loopで参照を待つ。owner liveness／cancel／最初から固定されたmin(point期限,phase期限,Run期限)は継続監視し、欠落のまま期限ならTimedOutで当該target0、期限後公開では復帰しない。解決後のarrived／release／hook検証には具体scopeIdを必ず使う。releaseはarrived SHAと全identity fieldが一致した時だけ有効であり、参照pathそのものをscopeの代わりにしない。これは既定の有限停止点の入力具体化であり、承認・ACK・実行期限を追加解除しない。

| case／hook | 固定する停止位置と検証順 |
| --- | --- |
| TRANS08–15／GATE18補助／`payload-read-before-validation` | Runnerが固定Run／fixture descriptor／InvocationPath境界と実Cargo→固定native entry→自身のOS親子・開始時刻を確認した後、payload bytesを読む前。invocationIdは検証済みinvocations directory名から一旦取り、payload内IDとの一致はその後必須とする。fixture時だけ同directoryへ`capture-ready.json:{runId,invocationId,cargoIdentity,entryIdentity,runnerIdentity,payloadPath}`を公開し、scope参照解決後に既定arrived/releaseで有限停止する。driverはpayloadを変異するか下記Request衝突を準備し、release後の本物のpayload validator／CreateNewへ進ませる |
| RACE03/04／`approval-read-before-recheck` | Runnerがfull承認bytesを読み、そのSHAを内部確定した直後、spawn前の全再照合より前。作者がapproval又はsynthetic sourceを変更してreleaseする。再照合から開始し、不一致ならtarget0 |
| LIFE09／`ready-validated-before-ack` | 起動ownerがReadyとOS identityを検証した後、ACK公開前。作者がそのownerを停止。子はACKを受けずtarget0。Supervisor自身の子ならRunOwner、workerならSupervisorを止める |
| LIFE10／`ack-read-before-recheck` | 子が対応ACKを読みSHAを固定した後、owner liveness／cancel／期限／source等の最終再照合前。作者がownerを停止。release又は喪失検出から拒否へ進みtarget0 |
| LIFE11／`v2-started-before-registration` | SafetyWorkerのV2Case Process.Start成功後、handleを手元に保持したままv2-registration.jsonを書く前。v2-intent.jsonは既に公開済み。arrivedにはV2のPID／handle／commandを入れず、回収者へ欠けた登録情報を供給しない。作者がworkerだけを停止し、Supervisorは既定のOS親子鎖／生存区間から回収する。証明不能なら既定CleanupIncomplete |
| RACE05補助／`image-lock-acquired` | workerが独自EXE read handleを取得した直後。arrivedは取得時点の観測だけであり、Supervisor喪失後もlockが生存するとの推定には使わない。作者は別handleの書込／差替え拒否と生存identityを併記する |

停止中も250ms以内でowner liveness／cancel／min(point期限,phase期限,Run期限)を監視する。期限をリセットせず、owner喪失／cancelならreleaseを待たず既定abortへ進む。release欠落も有限TimedOutである。再開時には**必ず既存の最終再照合**を行い、停止前の判定でspawnしない。V2 Start済みhookだけはabort時に手元handleを回収する。hookのenv単独有効化、任意script callback、fakeによる承認代理は実装しない。

TRANS08–15はcapture-readyにより正常bootstrap identityを実OSと確認し、各不正payloadを公開してからreleaseする。Resultへ限定診断`failureBoundary`を加え、実payload validatorで拒否した時だけ`PayloadValidation`、RequestのCreateNew衝突だけ`RequestCreateNew`、それ以外はnullとする。driverはRejected／target0に加えてこの値を照合し、外側の親子拒否やhook timeoutを目的validatorの成功に数えない。早期hook自身のscopeはcapture-readyへ束縛済みであり、破損payloadから得たIDで同期先や所有者を差し替えない。

GATE18は同じ早期hookで**valid payloadを保持**し、controllerが現在のinvocationId／正常identityを持つrequest.jsonを最初のCreateNewで占有してbytes/SHAを保存する。release後、正常Runnerが同pathへ二度目のCreateNewを行い、`RequestCreateNew`で拒否、既存request／承認bytes不変、target0を確認する。重複invocationの永続Request非上書き対照であり、nativeのUUID衝突再生成を強制した実証とは記録しない。scopeId・payload・正常OSchainを共用し、別Runnerの直起動やnativeの再呼出bypassを追加しない。作者のGATE18／TRANS08–15行の期待値は維持し、同期／観測方法だけをこの形へ接合する。

### 4. 観測不能の合成（LIFE16/17）とhandoff

同ps1の既存所有照合／終了観測を、それぞれ`ObserveOwnership(identity)`／`ObserveTermination(identity)`のprivate境界へ集約する。通常は実OS証拠だけを返す。fixture descriptor.observationFaultsは`{faultId,kind,targetIdentity,activeUntilUtc}`で、kindは`ownership-unavailable`と`termination-unavailable`だけである。対象はcontrollerが実際に起動したfixtureRoot内のfake／その子の正確なPID＋開始時刻に限定し、OS照合で確認できない指定はfixture自体の失敗とする。候補processが生まれた後にdriverがdescriptorを変える必要を避けるため、descriptorには先に`targetIdentity`の代わりに`targetReference`（fixtureRoot/control/<faultId>/target.jsonの新規path）を固定してよい。driverがそこへ実identityを一度公開し、各監督は読み込んだbytes/SHAを保存し差替えを拒否する。faultが適用された事実は`fault-<faultId>-<role>-<pid>.json`へ記録する。

LIFE16は独立controller所有sentinelを同じ合成root／fake imageで動かし、gateがabort時に監査する候補へ加える。`ownership-unavailable`はそのidentityの親子edge観測を`Unprovable`として返すだけで、Ownedへの昇格や停止権限を作らない。gateはunknown candidateを記録してCleanupIncomplete／full0／fixture保持とし、sentinelをkillしない。driverの別OS handleで同PID＋開始時刻の生存を確かめ、gate終了後にdriverが自分のsentinelだけを回収する。

LIFE17は実owned fake targetの終了観測を`Unavailable`とし、kill自体は通常経路で行う。driverは既存fakeのrelease待ちでtargetを生存させ、targetReference公開とfault適用証拠を確認してからcancelを出すため、終了後にfaultを後付けしない。これは`HasExited=false`の偽造でもwait成功の偽造でもなく、「終了を証明する観測が得られない」という欠測である。全ownerが同じfaultを適用し、別監督の正常な列挙結果で当該欠測を成功へ書き換えない。activeUntilUtcはcleanupDeadline以後、Run期限以下に置き、単一10秒内に終了証拠が成立せずCleanupIncomplete／fixture保持となることを確認する。driverは独立handleで実終了を確認し、未終了なら自分が所有するfixture treeを回収する。これらは意図したnegative対照であり、gateのCleanupIncompleteを製品PASSに数えない。

実装者は固定キー／sidecarと同ps1の限定hook／negative observerだけを実装し、既存V2Case・作者oracle・native captureの責務を変更しない。作者は80caseを維持したままdriverを確定キーへ接合し、対象hook到達→変異／停止→release→外側markerとOS handleで結果を判定する。未実装を行動REDにせず、欠けたcaseをskip/PASSにしない。この新fixture契約は親が局所レビュー対象として識別し、その後の独立実行／Postへ渡す。追加の製品semanticは0、合成試験専用の新semanticは有限停止とnegative観測の二つである。

局所作業の測定は23:19:16 JST近傍の共有週枠73%／10080分／reset1792496607。開始token／provider返却metadataは未取得である。現物hashは上記入力2fileと一致、作者fileの編集・fixture実行・再委譲・公開は0である。

## RunOwner bootstrap拒否context（初回合成実行後の局所是正）

GATE21-runの必須field欠落、GATE22の重複Run JSON、GATE23の型／未知cfgは、実行認可用Runの成立前に拒否する。一方、既定oracleはRejected／非zero／target0に加えてterminalと実測cleanup証拠を必要とする。この監査情報を壊れたRunから補完せず、RunOwnerだけが固定`R/runs/<UUID>/rejection-context.json`をstrict Run解析前に検証する。これは実行許可／g0／fullの代替ではない。childはこのlauncherを自身の親と比較せず、既存の自己scope確定前はunknown／書込なし、確定後だけ既存scopeへterminalを保存する。

sidecarは次の8fieldだけを持ち、別名／欠損／重複key／未知keyを拒否する。

| field | 完全な型・値 |
| --- | --- |
| runId | canonical directory名とOrdinal一致する小文字D形式UUID |
| runSha256 | 刺激による変異を全て完了した後のrun.json全bytes SHA256、uppercase64hex |
| startedUtc / deadlineUtc | 既定7桁UTC文字列。未来開始不可、正の期間、開始から最大3600秒。実行停止はdeadline-10秒 |
| launcherIdentity | 既存5fieldの実OS identity。RunOwnerの直接OS親と同世代であり、開始時刻照合前にSafeHandleを固定して終了まで保持 |
| auditKind | `fixture`,`default`,`early-trial`,`acoustic-analysis`,`qwen-audio`,`qwen-audio-no-default`の閉集合。監査境界だけを選び、Run.cfgの代替にしない |
| fixtureControl | fixture時だけ既存descriptorのPathHash、製品5値ではnull |
| tempPaths | 正確に`{TEMP:<absolute string>,TMP:<absolute string>}`。fixtureでは検証済みfixtureRoot内。製品は下表の固定値 |

準備順は、独立親／作者がfixtureとdescriptorを確定→Runの刺激／raw書込を完了→raw SHAを計算→sidecarを既定atomic CreateNew公開→RunOwner開始である。GATE22の重複keyを削除・正規化してSHAを取らない。動作中のsidecar更新／第二発行／旧run流用はない。新public parameter、native変更、fixture bypassは追加しない。

readerは固定mode／parameter set／実pwsh／非Adminの確認後、RunPathが`R/runs/<UUID>/run.json`そのもの、UUID小文字D形式、絶対path、既存directory、全祖先reparse不在であることを確認する。この安全なpath由来runIdと証拠rootだけを先に保持する。境界未確定なら外へ書かず非zero／unknown。sidecar欠落・不正なら、安全rootが確定している時だけCleanupIncomplete／未取得配列nullのterminalを保存する。

fixture contextはdescriptorのSHA／runId／suiteId／root、固定fixture-owner.txtの内容、controllerIdentityとlauncherIdentityの一致、実OS親identity、TEMP／TMPのroot内境界を検証する。descriptorやmarkerのbytesもimmutable照合へ含める。製品contextはfixtureControlを禁止し、固定TEMP／TMPを確認する。Runは別のValidationInputとして通常の全検証を通し、その完了まで実行stateのRunはnullとする。成功時にRunのcfg／runId／時刻／TEMP／TMP／fixtureControlがcontextと一致することを確認して初めてRunへ昇格する。不一致をcontext値で補修しない。

| auditKind | TEMP / TMP | 実行時の関連固定値 |
| --- | --- | --- |
| default | 既存full2-fixed-envs.jsonのConfiguration=defaultの値 | 既存値不変 |
| early-trial | 同Configuration=trialの値 | 既存値不変 |
| acoustic-analysis / qwen-audio / qwen-audio-no-default | `Repo/target/early-trial-build/et3/full2-env/related/temp`共通 | LOCALAPPDATA=`.../full2-env/related/local`、CARGO_TARGET_DIR=`Repo/target/early-trial-build/et3/related`、JOBS2と他既定envキーを維持。3cfgは逐次実行のみ |

関連3cfgの値は本局所是正で親が指定した新しい合成境界であり、default／trialと共有しない。Assert-ProductionRunもこの固定値以外を拒否する。新しい実Cargo起動権限は与えない。

bootstrap拒否ではsidecar検証後の残時間から得たUTC／Stopwatch上限を引き継ぎ、単一回収期限をmin(最初abort+10秒,context.deadlineUtc,既存cancel共有期限)とする。時計逆行・後続owner・初期化の再実行で予算を増やさない。Runを一度も実行stateへ昇格していないためStartは未到達だが、これだけで子0とせずOSの自身の子を実際に観測する。未知の子があればUnprovableとして記録し停止しない。所有集合0かつ観測子0だけでprocesses=[]、検証済みTEMP／TMPの実列挙0だけでfixtureResiduals=[]とする。列挙不能／残存／期限超過はCleanupIncompleteであり、未取得値はnullのまま、fixtureを削除しない。通常の入力拒否はRejectedを保持し、既に検出したTimedOut／Cancelled等を成功へ置換しない。

terminalは既定`result-RunOwner-<pid>.json`へatomic CreateNewする。path由来runId、実測できた自己identity、拒否理由を記録し、未到達のmanifest／request／ACK／argv／child exitはnull、sourceMatched=falseとする。開始前監査を他branchやrun全体の終了証明へ拡張しない。GATE21-approvalは通常Run成立後の既存拒否・回収経路を維持する。80case刺激／期待値、未知非PASS、terminal必須、negative oracleは変更しない。

未解決の受入阻止：LIFE14の既存刺激はdescriptor.controllerIdentityをsentinelのPIDと誤StartTimeへ変異するが、独立sidecar.launcherIdentityは真driverである。上記contextのcontroller／launcher一致検証は成立せず、監査context完成前のCleanupIncomplete／未取得配列nullとなる。既存oracleのRejected＋既知cleanupとは整合しない。sentinelをlauncherと認定する、descriptor境界を緩和する、空監査結果を捏造する是正は行っていない。本局所実装のparser／analyzer成功をLIFE14や80caseのPASSへ換算せず、追加設計と独立検証を未了として親へ返す。

## LIFE14：Supervisor Ready初回読取前の限定同期（2026-10-15）

本追補は直前の未解決事項に対する設計であり、実装・受入完了ではない。[計画](early-trial-gate-plan.md)の指定designerは`gpt-6-astra/high`、[仕様担当の限定解釈](early-trial-delivery-review.md)を入力とする。利用者継続承認は親引継ぎによる。LIFE14 caseRow110はRegistration/Ready不一致拒否への実到達を要求し、後続OS実世代比較への到達証明は要求と混同しない。80行・既存合否・期限・所有基準を変更しない。UI変更なしのためwordweave-egui-uiは非該当である。

現物基準はHEAD `25b744ea26237703acddb2125d04c008241bede8`、R=`target/early-trial-build/et3/cargo-gate`、Invoke-CargoGate.ps1 SHA256 `3421604615881BAE6451A988245049CD67E5476551FDCE0700FC825F5BFB9037`、Test-CargoGate.ps1 SHA256 `EEAF6957CBBCB7B76A14ADEE5CBA52D46D547AE44A67A995DBCA189DAC760DD6`である。Invokeの174/191行のRead-Json/Write-Jsonは初回読取／発行時にimmutable SHAを登録し、1263–1273行のConfirm-ChildReadyはReady存在待ち→初回読取→Registration全体比較→Assert-Identity→既存hookの順である。Testの1589行のcontroller変異はこの比較前のbootstrapで拒否されるため、目的validatorを試験していない。

| LIFE14対応／担当境界 | 確定インターフェース |
| --- | --- |
| 実装者／InvokeのAssert-Fixture・Confirm-ChildReady | 閉集合へhook=`supervisor-ready-read-before-validation`を1個追加する。cfg=fixture、role=`RunOwner`、scopeId=現在runIdの直接指定のみ許し、scopeIdReferenceは禁止する。このhook指定で別role／scopeは拒否する。呼出位置は正常に生成済みのChild.registration.role=`Supervisor`、scopeId=runId、directory=`R/runs/<runId>/processes/Supervisor`を持つRunOwnerのConfirm-ChildReadyだけである |
| 作者／Set-CaseSynchronization | LIFE14だけ`{pointId:"author-point",role:"RunOwner",scopeId:<runId>,hook:"supervisor-ready-read-before-validation",deadlineUtc:<既存run.deadlineUtc-10秒>}`をdescriptor.syncPointsへ設定する。既存author-pointの単一処理を再利用し、新同期framework／callbackは作らない |
| 同期入力・出力／既存Invoke-SyncPoint | Readyの存在を待ち終えた後、Ready bytesの初回Read-Jsonより前に呼ぶ。arrived/releaseの既存field・SHA結合・一回性を保持し、scopeReferenceSha256=null。arrived.actorIdentityはRunOwnerでありSupervisorではない。待機は既存Assert-Controlと25ms loop、固定point/phase/run上限を使用する |
| 対象参照／作者だけが刺激 | 対象は`R/runs/<runId>/processes/Supervisor/ready.json`、対照は同directoryのregistration.jsonで一意に導出する。任意path入力・新targetReference・Request／Runner scope参照は追加しない。arrivedは対象bytesを読む許可やprocess停止権限ではなく、ownerが対象をまだ読んでいない位置の観測である |
| 変異field／作者のInvoke-CaseSynchronization | Readyの`identity`だけを保持中sentinelの実5field identityのコピーに置換し、その`startTimeUtc`だけ実時刻+1 tickの既存UTC形式へ変える。pidは生存sentinelの実PID。他のReady field、正常Registration、controllerIdentity、rejection-context.launcherIdentity、Run/descriptorを保持する |
| 到達診断／Invokeの実比較分岐 | 初回Read-Json成功後、既存`Test-EqualData ready Child.registration`がfalseとなる分岐だけで既存Result.failureBoundary=`ReadyRegistrationMismatch`を設定して既存拒否へ進む。hook到達／Ready読取／schema拒否／timeout／bootstrap失敗だけでは設定しない。実比較をfixture専用の失敗判定で置換せず、診断は所有・回収を変更しない |

保存・状態の順序は、(1) 作者が真controllerのdescriptor／Run／rejection-contextを既存順に固定してRunOwner開始、(2) RunOwnerがSupervisorを実Startしhandle／Registrationを保持、(3) Supervisorが正常Registrationと同内容のReadyを既存Write-Jsonで一度発行してACK待ち、(4) RunOwnerがReady存在確認後の新hookでarrivedを公開して停止、(5) 作者が正常Ready／Registrationの一致と実Supervisor identityを確認し、元Ready bytes/SHA・Registration SHA・sentinel実identityを作者証拠へ保持、(6) 作者がReady.identityだけを変異して書込を閉じ、変異bytes/SHAを保存し、既存Write-NewEvidenceでreleaseを一度公開、(7) RunOwnerがreleaseとimmutable証拠を検証して初回Read-Json→実不一致拒否→既存abort/cleanup/Result、である。ACKは公開せずCargo/V2Case/対象EXEは0である。

Ready変異は合成入力破損という作者の刺激であり、本体Write-Jsonの非上書き規則を変えない。RunOwnerは新hook前に対象ReadyをRead-Json／Assert-PathHashへ渡さず、対象SHAをImmutableへ先登録しない。登録済みhashを削除／更新する回避も禁止する。そうしないとrelease末尾Assert-Immutableで先に落ち、目的比較へ届かない。RunOwnerの既存Read-OwnedTreeはOS列挙でありReadyを読まない。Supervisorは自分が発行した正常Ready SHAを保持するが、ACK待ちはAssert-Controlだけであり、この刺激のための子側検証緩和は不要である。正常経路・他hook・製品cfgの順序は維持する。

作者は到達時にarrivedのrunId/runSha256/pointId/role/scopeId/hookとactorIdentityを照合し、RunOwnerの保持handle実identityと一致させる。SupervisorのRegistration.identityは独立OS handleで実PID/開始時刻/pathと直接親RunOwnerを照合する。Readyの変異前後の構造差がidentityだけ、変異identityがsentinel実コピー＋誤StartTime、Registrationと正常bootstrapのbytesが不変である証拠を保持する。fixture.observationsの局所名は`readyTargetPath, originalReadySha256, mutatedReadySha256, registrationSha256`、実identityは既存`sentinelIdentity`、同期は既存`arrived, arrivedSha256, hookApplied`を再利用する。これらは作者証拠でありgate所有台帳の入力にしない。

LIFE14の判定は既存Rejected／非zero／zeroAll／terminal必須／既知cleanup／sentinel同実identity生存に、同run・role=RunOwner・scopeId=runId・actorと同じ実identityのResultの`failureBoundary=ReadyRegistrationMismatch`と上記刺激証拠を結び付ける。他roleの同名診断、エラー文の部分一致、hook到達だけでは合格しない。1271相当の不一致分岐で停止するため、1272相当のAssert-IdentityがOS開始時刻を照合して拒否した証明にはならない。OS再照合の既存約束は残し、別の実測なしに検証済みへ変更しない。

cancel／controller喪失／期限／release欠落・不正は既存abortへ進み、目的診断がなければLIFE14不成立である。RunOwnerは手元の実Supervisor handleと既存所有台帳を使って単一cleanup期限内で回収・waitし、偽Ready.identityからsentinelをOwnedへ登録しない。unknownは停止せずCleanupIncomplete／非PASS、未取得配列はnullを保持する。診断があっても回収未証明をRejectedの成功対照へ変換しない。作者はgate終了後もsentinelの保持handleで実PID＋実StartTimeの同一生存を確認し、その後だけ自身のsentinelを既存手順で回収する。刺激に使った誤StartTimeを終了確認や停止指定へ流用しない。

既存`ready-validated-before-ack`流用は比較済みのため却下する。controller変異・context緩和は目的境界を変えるため却下する。Registrationも同じ偽identityに変える案は実OS比較という別の試験目的と所有証拠を変えるため採用しない。固定対象を新targetReferenceや汎用fault APIへ抽象化する必要はない。

引継ぎは独立Pre→作者のLIFE14刺激／同期／診断oracleのみ→実装者の限定hook／診断のみ→新固定snapshotの独立実行／Postである。作者のcaseRows80行と既存期待値、native/fake/V2Case、製品sourceを変更しない。作者のschemaCandidate.failureBoundaryには新診断値を追加して現物契約と一致させる。正常Readyの無刺激対照と有限待機・誤release・目的診断不在時の非PASSを既存試験枠で確認し、残る80件／全体gateを省略しない。今回の完了境界は本節とdiff/link/hash確認までで、source実装・test編集・runtime・再委譲・公開は0。仕様上の新たな未決事項はなく、独立Pre・動的到達／cleanup実証は未了である。子の開始／終了token counterとprovider実行metadataは未取得であり、親のKPI記録へ欠測として引き継ぐ。

## 所有同定失敗の限定観測（2026-10-15）

親からの観測限定割当、designer=`gpt-6-astra/high`。基準Invoke SHA256=`0E28AAA12343AB331FD4D26837324B403ECBFC0C0980092237B4A5AC721035C2`、Test=`0BB2510E928C9C3BFAD34D8190F007C97881047B195194A586A319837FE484DE`、追補前本書=`0B7802CC6347EB177E074E677E7FC028C6283ABCA7C98C6ABB42B9C7B0554A69`。ENV01の未知PID24592は元CIM行未保存で役割／欠測理由不明であり、CleanupIncompleteを修正しない。本追補はG5の所有証拠・LIFE16/17の未知非PASSに対する診断追加だけで、新たな所有根拠ではない。
観測先は既存terminal.processesの該当Unprovable行の任意object `osObservation`であり、親terminalのrunId/role/scopeId/identityとfile SHAへ結合する。同PIDの複数行を集約／同世代扱いしない。追加sidecar・OS照会・trace・新frameworkは0、既存terminalのatomic公開順とcleanup期限を維持する。
objectは`pid,parentPid,creationUtc,pathPresent,commandPresent,snapshotMatchCount,knownMatchCount,exceptionType,exceptionLine,rowFilterMatchCount,rowFilterPathPresent,rowFilterCommandPresent`を持ち、生成失敗時だけ`producerFailure:{exceptionType,exceptionLine}`を加える。全fieldは未取得null可。pidは正の整数、parentPidは0以上整数、件数は0以上整数、Presentはbool、exceptionTypeは実例外の型FullName文字列、exceptionLineは正のScriptLineNumber整数である。bool／文字列を整数へ強制変換せず、取得不能を0/falseにしない。
snapshot側の読取元はRead-OwnedTreeの既存`$row/$rows`とcatch時の既存`$script:Known.Values`だけである。pid/parentPidはCIM ProcessId/ParentProcessIdの範囲内整数、creationUtcはCreationDateがDateTimeかつKindがUnspecified以外の時だけUTC7桁へ変換し、その他null。pathPresent/commandPresentは元行の当該propertyが存在し非nullかどうか（空文字はpresent）である。snapshotMatchCountは同snapshot内の同PID行数、knownMatchCountは台帳の同PID行数であり、いずれも同世代／所有の証明ではない。
rowFilter側は失敗したGet-Identityで実際に渡されたRowsをProcess.Idで抽出した結果だけを表す。現物345–353行は新OS照会を行わず、Read-OwnedTree1136行では入口1117行のsnapshotを再使用するため、「fresh OS再照会」と記録しない。rowFilterMatchCountはその実抽出件数、rowFilterPathPresent/rowFilterCommandPresentは件数1の時だけ同じ非null判定、件数0/複数なら両bool=nullである。抽出前のhandle取得／Refresh失敗等では3fieldともnull、snapshot側から補完しない。
実装者はGet-Identity内でこの3値を最小局所変数として保持し、失敗時だけ元例外のDataキー`ET3.IdentityRowFilterObservation`へ同名3fieldを渡して元例外をrethrowする。取得した値だけを格納し、全metadata未取得もnullで表す。Data添付自体の失敗は元例外を置換せず、outer側はData欠落をrowFilter全nullとして扱う。共有script変数への最後の観測保持は別呼出の混入を招くため用いない。
Read-OwnedTreeの最初のdescendant catch（現物1149行近傍）が元ErrorRecordを先に保持し、既存Unprovable rowへ上記objectだけを添える。exceptionType/exceptionLineはこの元ErrorRecord由来であり、message/stack trace/path/command/environment/資格情報を新objectへ複製しない。観測生成は囲ったtry/catch内で行い、変換等が失敗したfieldはnull、producerFailureは生成側例外の型と行だけとする。producer失敗でも元row・元拒否・identity=null・killRequested=false・waited=false・hasExited=nullを保存する。既存errorの意味を変えず、別のUnprovable生成経路への機械的横展開は今回不要である。
実装所有はInvokeのGet-Identity局所metadataと上記catchの観測生成のみ（必要なら同fileの小private変換helper1個）である。Get-OsProcess、所有判定、停止、回収、期限、例外の成功化、native/fake/V2/製品sourceは変更しない。TestのschemaCandidateは144/207/2131行の出力用契約一覧で、process行の未知fieldを拒否するvalidatorではない。作者は任意型の許可一覧`processRowOptionalFields.osObservation`へ上記field/typeだけを記述し、必須化や80case oracle変更をしない。
独立作者の小対照はproperty欠落、snapshot/rowFilter件数0/1/複数、Known同PID候補あり、CreationDate欠落／Unspecified、観測producer例外、rowFilter metadata欠落と実測0の区別を扱う。合成行と局所境界で検証し追加OS照会0、全対照で元拒否／unknown非停止／非PASS不変を確認する。実terminalへ載る証拠は後続の新固定snapshot実行で取得し、旧PID24592の原因へ遡って適用しない。
本担当の完了境界はschema文1箇所と本追補・diff/link/hash確認である。独立Preと実装／小対照／動的実証は未了、未知processの正体は未解決のまま親へ返す。使用skillは既読api-and-interface-design、UI非該当。runtime/source/test編集・再委譲・公開0、子token counter／provider実行metadataは未取得である。

## CIM path欠測時の同一保持handleによる限定補完（2026-10-15）

入力は[承認済みET契約](early-trial-contract.md)、[G5とモデル割当](early-trial-gate-plan.md)、[plannerの限定解釈・親割当](early-trial-delivery-review.md)である。designerは`gpt-6-astra/high`、本書だけを所有する。api-and-interface-designを読了し、内部型・失敗境界へ適用した。UIは非該当。基準HEAD=`9963c52c8c277ea744e9e204e74af96e9b451ce7`、Invoke SHA256=`333C37FF523542AD52004B76D17CE43369896555CEA2F326AE749A3C55D42245`、Test=`D8AD0AD44C2457DC60E64E52A6EDB455974D5F8AD9938B956CEF3E4DDAB0F7F3`、追補前本書=`51DBDBD7B8AE8E31AAA2BF5BB74D9C6D51EF39A8455C2EEFB2C38A1250A2A8EB`である。既存dirtyを保持して末尾だけ追記する。

[直前実測](early-trial-delivery-runner.md)のsuite `e48f86cd-4032-4598-a904-a4d0a3679be6`ではENV01がCleanupIncomplete、control2PASS／80caseは0PASS・1FAIL・79NotRunである。unknown15748のCIMは1行、path=null、commandあり、CreationDateあり、known0で、parent22496は登録Cargoと一致する。しかしGet-Identityの360行で世代照合前に拒否されており、path欠測以外が成立した証拠でも、このprocessを所有できる証拠でもない。本追補は今後のnull-pathだけを扱い、旧runの判定・所有記録を変更しない。

### 内部境界・順序

実装所有はR/Invoke-CargoGate.ps1のGet-Identityと同fileの小private helper／相互運用宣言だけである。現物にはAdd-Type／DllImportがなく既存interop再利用はできないため、親受領済みの最小Add-Type宣言1個を追加する。追加package・外部DLL配置・新process／runspace・新public parameter・native入口／V2Case／製品source変更はない。型ロードは最初の必要時に一度だけ、失敗を既存拒否へ返す。目安は宣言／1回読取20–35行、有限loop／control確認20–35行、Get-Identity接合5–15行である。これは品質上限ではなく、汎用frameworkへ膨張した場合に親へ返す目安である。

| 境界 | 入出力・所有・不変条件 |
| --- | --- |
| Get-Identity(Process, Rows) | 現在のProcess.SafeHandleをRefresh／StartTimeより前に固定する。Rowsは既存snapshotのままで新CIM照会を追加しない。既存5field identityとrowFilter例外metadataの型を維持する |
| helper呼出前の検証 | 同PID行数が厳密に1、CommandLineが存在かつ非null、CreationDateがDateTimeかつKind≠Unspecifiedであることを先に検証する。保持Process.StartTimeとのUTC6桁・InvariantCultureの世代一致を既存方式で確認し、返す開始時刻は7桁のままとする。いずれか不成立ならpath helperへ到達しない。欠損propertyをnullとして補修しない |
| pathの二分岐 | ExecutablePath propertyが存在し、値が厳密にnullの時だけhelperへ渡す。非null値は従来どおりGet-CanonicalPathへ渡し、空／相対／不正／不存在／reparse等の拒否を維持する。非nullの矛盾をOS値で上書きしない。helper返却値も同じGet-CanonicalPathで検証し、prefix除去・短縮名展開・検索・alias許可を追加しない |
| Get-PinnedProcessImagePath(SafeProcessHandle) | 呼出元が既に保持する同じオブジェクトを借用し、canonical化前の非空Win32絶対path文字列を返すか例外を返す。PIDを引数にせず、OpenProcess／GetProcessById／MainModule／自己申告／Intent値で再取得しない。handleを作成・複製・Dispose・Closeしない |
| 小C#相互運用型のRead(SafeProcessHandle, int capacity) | Kernel32のQueryFullProcessImageNameWを1回だけ呼ぶ。P/Invokeは明示W、CharSet.Unicode、ExactSpelling=true、SetLastError=true、BOOLのbool marshal、引数SafeProcessHandle／uint flags=0／StringBuilder／ref uint sizeである。公開されるのは同script用static methodだけで、gate外部契約ではない |

Readは入口でnull／IsInvalid／IsClosedを拒否する。P/InvokeへSafeProcessHandleを直接渡し、CLRの呼出中参照増減を使う。裸IntPtr、DangerousGetHandle、独自CloseHandle、手動DangerousAddRef/Releaseは不要である。呼出元がProcessと固定handleの強参照を保持し、終了・台帳登録／既存finallyまで所有を変えない。各試行前と成功返却前にも同じ固定handleのclosed/invalidを確認し、再Openによる回復をしない。並行Disposeを許可する新workerは作らない。これはPID再利用を防ぐhandle寿命であり、親edge・command・世代検証の代替ではない。[SafeHandleの公式参照数契約](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.safehandle?view=net-10.0)を根拠とする。

Readのnative失敗直後に、別のP/InvokeやPowerShell処理より先にC#内でMarshal.GetLastWin32Error()を保存し、そのNativeErrorCodeを持つWin32Exceptionを投げる。PSのstatic呼出wrapper越しでもその元例外を保持し、文字列比較で再試行を選ばない。成功ではsizeが1以上capacity未満、StringBuilder長とsizeが一致、内部NULなしを確認し、全文字列を返す。切詰め・失敗buffer・不正sizeを成功として返さない。[QueryFullProcessImageNameW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew)はflags=0をWin32形式、sizeを文字数、成功sizeを終端NULを含まない長さと定める。必要権限は既存handleのquery権限だけで、権限不足でも再Open／Admin昇格しない。[Marshal.GetLastWin32Error](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.marshal.getlastwin32error?view=net-10.0)のキャッシュされたエラーを保持する。

PS helperの容量は256→512→1024→2048→4096→8192→16384→32768文字、最大8回とする。再試行は元Win32Exception.NativeErrorCode=122（[ERROR_INSUFFICIENT_BUFFER](https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-)）だけで、失敗時sizeを必要容量として信用しない。他エラー、最終容量でも122、型ロード失敗、無効／閉鎖handle、不正返却値はそのまま非成功である。32768はこのhelperの保守的資源上限であり、全Windows pathの対応保証ではない。上限到達でも最後のOSエラーを失わず、追加容量・別API・PID検索へfallbackしない。

型ロードと各API試行の前後で既存Assert-Controlと既存cleanup UTC／monotonic上限を確認する。Aborting時はAssert-Controlが早期returnするためcleanupの2上限を別途必ず確認する。新Stopwatch、phase再初期化、新10秒、sleep／backoffは設けず、残予算0なら呼ばない。cancel／期限がAPI中に生じたら復帰直後に拒否し、遅い成功をidentity／Ownedへ渡さない。API失敗と期限超過が重なる場合も元OS例外は保持し、既存の非成功状態を成功に変えない。同期OS呼出と型ロード自体を強制中断する保証はない。既存の「少なくとも一監督が生存しOS APIが応答する」保証条件を維持し、8回という回数上限を壁時計上限の実証へ換算しない。

状態順序は`Pinned → CIM行/command/date/同世代確認 → 既存path又は同handle読取 → canonical化 → 5field identity → 既存parent生存区間／Intent・Ready照合 → Add-KnownProcess`である。pathを取得しただけではOwnedにならない。親PID一致のみ、複数親候補、親生存区間外、LIFE16の観測不能faultは既存拒否を通す。helper失敗はGet-Identityの元例外／rowFilter metadata経由で既存Read-OwnedTree catchへ戻り、unknownはUnprovable・killRequested=false・waited=false・hasExited=null、CleanupIncomplete／非PASSである。既存DirectLaunchHandleの独立した直接起動証拠は変更せず、helper失敗を新たな直接起動証拠へ変換しない。

新永続field・sidecarは不要である。osObservationのpathPresent／rowFilterPathPresentは元CIM値の観測なので、補完成功でもfalseをtrueへ変えない。identity.pathだけが実OS値となり、ownershipEvidence.kind=OsQueryと既存全identity比較を維持する。Intent→Registration→Ready→ACKとterminalのatomic CreateNew順、回収後のfixture削除条件、未知値nullを保持する。失敗後の新しい同定試行を旧Unprovable行の除去や旧runのPASS化に使わない。現物の非null pathも常に照会して上書きする案は矛盾を隠すため、MainModule／PID再Openは同handle保証を外すため、別監督／Jobは権限・寿命範囲を広げるため採用しない。

### 受入対応・独立引継ぎ

以下のPTH番号は局所回帰名であり、既存80case／87variantの受入ID・刺激・期待値を置換しない。作者は既存のAST抽出による隔離対照を再利用し、必要な置換はテスト内のprivate境界だけへ限定する。製品modeやdescriptorにfault APIを追加しない。

| 局所証拠／受入ID | 作者・runnerの必須対照と期待値 |
| --- | --- |
| PTH1／G5、ENV01 | 合成processの同じ保持SafeHandleと実OS StartTime／実CIM1行を使い、行コピーのpathだけnullへ変更する。実QueryFullProcessImageNameWを通ったpathが独立確認した合成EXEと一致し、他4fieldが通常CIM対照と同一。helper前後のReferenceEqualsとhandle開存を確認する。成功値をmockしただけでは本項未検証 |
| PTH2／G3/G5 | 通常の非null CIM対照でhelper呼出0。空・相対・不正pathを従来どおり拒否しhelper0。非nullの別EXE pathはAssert-Identity又は既存Intent比較で拒否し、実pathへの上書きで成功しない |
| PTH3／G5 | 同PID0行／複数行、CommandLine欠落／null、CreationDate欠落／非DateTime／Unspecified、UTC6桁世代不一致はすべてhelper0・非成功。property欠落と明示null pathを区別する |
| PTH4／G5 | 実合成handleのclosed／invalid、OS API失敗、型ロード失敗を区別し、元例外／error code保持・非成功を確認する。未知descendant経路では非停止・非PASSであり、部分identityをKnownへ登録しない |
| PTH5／G5 | テスト内の1回Read境界で122→成功、122以外、8回連続122、成功size0／capacity以上／長さ不一致／内部NULを独立刺激する。容量列は上記8値、上限後の9回目0、元エラー保持、失敗buffer非採用である。mockと実API証拠を分ける |
| PTH6／G5 | 既存cancel／run・phase期限、Aborting中cleanup UTC／monotonic期限を各試行前後に刺激し、残0でAPI0、復帰後期限超過でOwned0、時計再開始0を確認する。遅延を与えた境界対照はOS全体の中断保証ではない |
| PTH7／G5、LIFE14/16/17/18 | 既存Registration/Ready拒否・unknown非停止／非PASS・終了未証明非PASS・独立sentinel同PID/実StartTime生存を維持する。正常path補完をmissing-edge／termination faultの回避に使わない。sentinel停止は作者の既存所有手順だけである |
| PTH8／G1–G6、ET-07 | 新固定Invoke/Test/design SHAで独立Pre→作者のRED→実装→局所GREEN／静的検査→独立Post／通常入口80caseを引き継ぐ。ENV01成功や局所PASSだけを80件・実Cargo接合・製品fullの成功へ換算しない |

設計上の未決仕様はないが、null-path時の実API成功、同handle保持、同期呼出時間、負例回収は未実測である。独立Preで同等のG5証明と判断できなければ実装前に親へ返す。APIが失敗する実環境でもunknown非PASSを維持し、OS権限・gate・80case oracle変更が必要なら本追補の範囲を終了する。実装／test作成はAstra/high、独立runnerはSol/medium、reviewerはAstra/highの既定割当である。本担当は文書diff／相対link／hash確認だけで、runtime・source/test編集・再委譲・公開0。最初の時刻観測は2026-10-15T22:00:46.9921021+09:00（着手そのものの時刻ではない）、子`/root/et3_image_path_design`の開始／終了token counter・provider返却model/effort・週枠は未取得。親のusage記録へ欠測として引き継ぎ、指定値や作業量から推計しない。

## ARGV-PATH01：native内部3pathの生成表現（2026-10-15）

[PacketBrief](early-trial-gate-plan.md)140行以降を全文確認した。指定designerは`gpt-6-astra/high`、所有は本追補のみ、api-and-interface-designを内部producer/consumer契約に適用し、UIは非該当である。入力HEAD=`ac92e29587da3fbffc739e95f17656a7649b6e91`、計画SHA256=`2A463D3260EC322D53B0D4815EB8EB1C0E103FCB75B29AA074455A7F8058CD44`、追補前本書=`CE799395A14F1784C05EDDB0BC2D8EE8B53B900EC306B403DB72E225CF5E5756`。Invoke/Test/native source/旧native EXE/fakeの正確な固定hashは同PacketBriefの表を参照する。native source=`3AEB0B19…331F42FD41`、Invoke=`12E7FC95…3281621`は現物一致を確認した。前packetのClosedは[review正本](early-trial-delivery-review.md)の独立証拠を再利用し、本節で再判定しない。

既承認G1–G6、特にG4の全argv exactとG5の所有・有限回収を再利用する。利用者のargvを変えず、既存consumerが要求する内部argvをproducerが正しく生成する是正であり、新しい外部仕様は不要である。旧節の固定path表示は同じファイル位置を指す説明として維持し、native内部引数の文字列表現に限って本節を優先する。旧PTH節の「native変更なし」はそのpacketの範囲であり、本packetのnative2定数変更を禁止する規範ではない。

原因証拠は標準suite `7f2c941c-d1b3-4b4e-a3cc-a71c627a6e2a`、run `7da565fe-32ab-42fb-b4c5-4eaf2d59a356`の保存OS commandとIntentの照合である。11引数中index4/8/10だけがOS側U+002F、Intent側U+005C、残8引数は一致し、target0のまま既存Assert-ObservedCommandが拒否した。数値indexは実行ファイル自身を除く0起算である。この1runの長さや差分数を後続UUIDの固定期待値にしない。

### 最小変更と型・境界

実装者はR/cargo_gate_entry.rsの`ROOT`を`r"F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et3\cargo-gate"`、`SCRIPT`を`r"F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et3\cargo-gate\Invoke-CargoGate.ps1"`という同じ絶対場所のraw stringへ変更する。定数2個だけの差分とし、helper追加0、PWSH定数・capture本体・Command構築・既存FFIは変更しない。動的なslash置換、UTF-8化、lossy変換、大小文字変換、prefix除去、quote付加は行わない。既存`#![cfg(windows)]`の固定Windows nativeであり、非Windows動作やportable path抽象化は対象外である。

| G4/G5のproducer→consumer | 生成値・所有・照合 |
| --- | --- |
| native SCRIPT → 内部index4 `-File` | `&'static str`の上記固定値を既存Command.argsへ渡す。SupervisorのRun.gateScript.pathとUTF-16 exact一致する。Run側の既存Assert-PathHash／Assert-SamePathで固定script実体を照合する |
| native ROOT → expected → 内部index8 `-RunPath` | `Path::new(ROOT).join("runs").join(run_id).join("run.json")`の既存PathBufを渡す。run_idは既存UUID検証済みでseparatorを含まない。環境WW5_CARGO_GATE_RUN_PATHは比較入力suppliedとしてのみ使い、起動文字列へ転記しない。Supervisorの既存RunPath文字列とexact一致する |
| expected.parent → invocations → directory → payload → 内部index10 `-InvocationPath` | 既存joinのASCII単一componentと生成invocation UUIDだけでPathBufを派生する。生成・atomic公開した同じpayload PathBufをCommand.argへ渡す。SupervisorのscopeDirectory/payload.jsonとexact一致する |
| Cargo argv → payload.argvUtf16 → Runner → target | native所有の`Vec<Vec<u16>>`を既存args_os/encode_wideから生成し、JSONのfield/header/bodyとコード単位を保持する。利用者のEXE pathを含む全入力は変換対象外。利用者argvのslash、空文字、日本語、非BMP、孤立surrogate、quote、末尾backslash、個数、順序を維持する |

[Rust PathBuf/push・join](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push)と[WindowsのMAIN_SEPARATOR](https://doc.rust-lang.org/std/path/constant.MAIN_SEPARATOR.html)に基づき、backslashの固定rootへseparatorのないcomponentをjoinする今回の構成からbackslash表現が得られると判断する。任意入力をjoinすれば正規化できるという一般化はしない。[encode_wide](https://doc.rust-lang.org/std/os/windows/ffi/trait.OsStrExt.html#tymethod.encode_wide)は不正対を含み得るUTF-16をlosslessに扱う既存根拠であり、本変更でもその処理を触れない。実装toolchainでの実OS11引数一致は後続の独立動的証拠を必要とする。

pathの同じ実体であることと、内部引数のexact一致を分ける。nativeの既存`reject_reparse`、supplied/expectedの`fs::canonicalize`比較、公開元／先のcanonical親一致を維持する。PS側もInitialize-Runの固定Run/script検証、Initialize-RunnerChainのinvocations境界・payload名・UUID・nesting、Supervisorのpayload SHA/Request比較を維持する。この3pathのcanonical一致が不成立・不明なら既存の該当段階で拒否し、文字列を書き換えて通過させない。canonical一致しても11引数exactが異なれば従来どおり拒否する。nativeがPSの検証を複製したりRun JSON parserを追加したりする必要はない。[fs::canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)はWindowsでextended-length形式を返し得るため、その返却PathBufをCommandへ渡す案は採らない。canonical値は従来の実体照合だけに使用する。

保存・起動順は、run ID/absolute/reparse/canonical一致確認→args_os捕捉→invocation directory確保→payload.tmp CreateNew/write/sync_all/close→MoveFileWで非上書き公開→固定pwshのRunner開始→既存Request/Intent/Registration/Ready/ACK→親承認・spawn直前照合→target開始、で不変である。nativeは同期status待ちを維持し、新async／cancel境界を作らない。RunOwner/Supervisorの既存期限と所有回収がnative/Runnerを監督する。nativeで入力／公開／spawnが失敗すれば既存Err→exit1、Runner以前ならtarget0。Runnerのpath/argv拒否もtarget0で既存回収へ進む。terminal欠落・回収未証明はunknown／非PASS、payload/fixtureを保持し、未知processを停止しない。既存CommandのHidden、stream継承、32bit exit伝達を変更しない。

### 独立証拠とhandoff

新機構は不要であり、独立作者はR/verification/argv-path01/配下の専用回帰だけを所有する。保存REDの実OS command/Intent/payloadとそのrun・PID/開始時刻・source/EXE SHAを結び、既存parser/validatorのAST抽出を用いる。旧証拠に必要fieldがなければ行動RED未成立として親へ返し、compile失敗やmock不一致で代替しない。新nativeのGREENは実native→実pwsh7.6.6→既存gate経路から観測する。Command組立だけの単体試験、payload JSONだけ、別scriptへの代替起動は実OS到達証拠にならない。

| 証拠／受入 | 必須判定とtest seam |
| --- | --- |
| AP1／G4、ENV01 | REDのindex4/8/10不一致と残8一致を保存入力から再現する。GREENは同じ既存parserと独立oracleで実OS11引数すべてをIntent.argvUtf16へcode-unit exact照合する。path3個をnormalizeして比較しない。実行file自身は既存Assert-SamePathで別に照合する |
| AP2／G2/G3/G5 | 3pathの正しい固定実体／run／invocationへのcanonical一致とSHAを検証する。別Run、別payload、別script又は意図的な内部引数1code-unit不一致は既存validatorで拒否しtarget0。AST単体の負例と実gate拒否・terminal・cleanup証拠は区別して記録する |
| AP3／G4、既存TRANS | 新nativeに束縛した既存argv境界対照で、空文字、日本語、非BMP、孤立surrogate、quotes、末尾backslash、slashを含む利用者引数のexact、raw stdout/stderr、exit0/7/101/0x80000001/0xFFFFFFFFを独立fake観測と照合する。旧nativeのPASSを新nativeの証拠へ流用しない |
| AP4／G1–G6、ET-07 | native差分が2定数だけ、Invoke/Test/fake/80case・87variant oracleが不変、新source/EXE/runのhash結合、許可前target0、所有cleanup、unknown非停止/非PASSを確認する。ENV01のargv境界通過後に別失敗が出れば局所到達と全体失敗を併記し、80件PASSにしない |

build/formatは既存方式を使い、RをPacketBriefの絶対rootへ展開して`rustfmt --edition 2021 --check <R/cargo_gate_entry.rs>`と`rustc --edition=2021 <R/cargo_gate_entry.rs> -o <R/cargo-gate-entry.exe>`を実行する。build担当は開始前に旧source/EXEの証拠を保持し、同EXEを使う生存runなし、使用するrustc/rustfmt実path・versionを確認する。新source/EXE SHA固定後に新UUIDのRunを作りnativeSource/nativeExeへ束縛する。旧Run／承認／hashの上書き更新はしない。作者は回帰command・固定入力・出力rootを実装前に記録し、同じoracleでRED→GREENを実行する。局所Post後の標準入口は既存`& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File <R/Test-CargoGate.ps1> -GateScript <R/Invoke-CargoGate.ps1> -OutputRoot <R/verification/new-UUID>`の80caseを維持する。製品release再buildは本packetの検証ではない。

引継ぎは計画者の設計受領→本書SHAとPacketBrief snapshotによる独立Pre→作者RED→native2定数実装→独立GREEN/Post→標準80件である。役・model/effortはPacketBriefのまま。本設計に未決技術選択はなく、実OS exact・TRANS・負例回収が未実測である。2定数を越える修正、実OS観測不能、canonical不一致や新たな拒否原因、権限／依存／品質gate変更が必要なら親へ返す。着手観測2026-10-15T23:09:44.4237729+09:00、token開始/終了counter・週枠・provider実行metadataは未取得。本担当の検証は文書diff/link/hashだけ、code/test/runtime/再委譲/公開0である。

## CXT01：focused実行contextの唯一の現行契約（2026-10-16）

本節は[PacketBriefのCXT01再構成](early-trial-gate-plan.md)を受けた限定追補である。計画SHA256=`E209293EC26AA95F0C88C0CFE795B397BEF0B9BF3EB5B94F3FED63F7289E84C4`、追補前設計=`2123BB9E88E616BF8455E7C6A63931D59DFC8CB1C3FBCAB4F134E09FCE9CCAB7`、入力driver=`89C6E49B7D2930FE225DA0AB7C4E36B94B55CFBC522CEE35454F0BE17A2350FB`、HANDOFF=`FB1F80EB66BD82858821E449605FBB44B76641BCF9D2367158E55DDFEC153695`を実照合した。native381F5E1D/EXEFF7C9A0F/Invoke12E7FC95/TestD8AD0AD4/fake3482075Bの全hashは同計画表へ束縛する。designer指定Astra/high、api-and-interface-design既読、UI非該当。本書だけを編集し他者のdirtyを保持する。

前節のnative2定数・G1–G6/AP1–4は保全する。ただし前節の「回帰はargv-path01配下」と旧HANDOFFの実行OutputRoot例は以下の用途分離で置き換え、旧nested実行commandとValidateOnlyからruntime可能とする解釈を失効させる。`R/verification/argv-path01`はdriver/oracle/HANDOFFと解析結果の場所、**実行OutputRootは新規`R/verification/<suite-UUID>`直下**、fixtureRootはその`<case[-variant]>`子である。Runは従来の`R/runs/<run-UUID>`、payloadは`invocations/<invocation-UUID>/payload.json`、V2は`safety/<caseId>/v2-output`のままとする。suiteIdは実行OutputRootのbasenameでありargv-path01へ偽装しない。UUID命名はfocused producerの選択で、既存consumerのsuiteId正規表現は変更しない。解析oracleだけのOutputRootは従来のpacket配下でよい。

初回repository未定義と二回目nested root拒否はcontext構築の失敗であり、argvのRED/GREENではない。二回目のcontext確定前terminalにあるprocesses/fixtureResiduals/cleanupDeadlineUtc=null、sourceMatched=falseを0/PASSへ変換しない。以下のPre以前の第三patch／再実行は不可であり、CXT01は未Closedである。

### 初期化・消費・保存順

driverは実TestのAST prefixと、同main内の一意なboundary→repository代入を元Fileにsource-bindして実行する既存方式を保つ。main丸ごと実行・同内容のconsumer再実装はしない。callbackのFile一致だけでは初期化済みと判定しない。driverの`ValidateOnly`は次表の起動前検査まで、file書込／build／identity-control／RunOwner起動0とし、出力へ`runtime=false`と未検証のfile/schema/OS境界を残す。通常実行でも同じ起動前検査を通ってから、既存fake build→identity-control2件→controller取得→既存10focusedへ進む。

| 境界／実consumer | 起動前の入力・所有・必要な実確認 |
| --- | --- |
| source-bound scope／Test New-CaseFixture・Invoke-GateCase | boundary=固定R、repository=Rの4階層上のrepo、PSScriptRoot=実TestのR、GateScript=固定兄弟Invoke。fakeContractのstream keys、caseRows SHA=`CEF0406933398F433D2726F7EC4F1C4292A0CDF555B39AEA725ABB40920FF905`、80件unique、選択9件＋既承認slash variantの10件と16→17引数guardを保持する。standard dispatch専用caseResults/passedは使用しない |
| 暗黙config入力／実Get-ConfigSnapshot | USERPROFILEとrepositoryを設定済みの同scopeで呼び、user `.cargo/config`、user `.cargo/config.toml`、repoの同2pathという順序・4件を確認する。pathと取得hashだけを記録する。hash=nullはsnapshotの欠測/不存在表現であり内容無しや無変更成功と推測しない。環境・configへ書かない |
| 実行path／driverとTest main前提 | 絶対OutputRootの親が解決済みR/verificationそのもの、basenameが新UUID、未存在、全祖先reparseなし、git check-ignore成功を確認する。GateScript実体、Windows/pwsh7.6.6/AdminFalse、Pester5.7.1の利用可否、rustc実path、V2/native/source存在、native非稼働を確認する。未取得なら起動せず停止する。部分的なStartsWithだけで階層を許可しない |
| frozen inputs／driver・Run Assert-Source | 計画表のInvoke/Test/native source/EXE/fake、独立argv oracle7D74D443、今回driver/HANDOFF/新context-testと設計を実hashで固定する。V2/pwsh/fake EXEの実path/hashとtoolchain証拠も保存する。各case前後の既存固定入力照合を維持し、別snapshotを混ぜない。nativeはsource/EXE不変なので再buildしない |
| fixture／元New-CaseFixture→Publish-FixtureRun→Initialize-Fixture | fixtureRoot以下のrepo、local/temp/target/control、fake cargo/artifacts、source/g0/markerと各snapshot、descriptorと参照fileを既存producerで生成する。`fixture-owner.txt`はUTF-8 no BOMの`ET3-G independent synthetic fixture`＋LF。descriptorのrunId/suiteId/root/controller/cargoKind/cargoFixture/syncPoints/observationFaultsと各境界を元consumerへ通す。固定gate/native/pwsh/V2、Run/safetyの既存例外を増やさない |
| sidecar／Invoke Initialize-RejectionContext→Initialize-Run | 刺激完了→descriptor/Run bytes確定→raw Run SHA取得→8field `runId,runSha256,startedUtc,deadlineUtc,launcherIdentity,auditKind,fixtureControl,tempPaths`の非上書き公開→RunOwner開始、の既存順。tempPathsはTEMP/TMP両fieldが必須で同temp可。真の起動driver identity=descriptor.controllerIdentity=sidecar.launcherIdentityであり、実RunOwnerのOS直接親・同世代照合を必要とする。時刻は既存UTC7桁とRunの65秒/build25秒、実行55秒＋回収10秒。context確定前にRunへ昇格せず、Run/cfg/時刻/fixtureControl/TEMP/TMP一致後だけ昇格する |
| 子scope／Initialize-RunnerChain、所有・cleanup | 正確なpayload名/UUID/nesting確認の後に既存registration/OS親鎖を照合する。新suite配置でRun/payload位置を移さない。失敗は既存有限cleanupへ返し、unknown非停止／非PASSを維持する。実子列挙・実TEMP列挙以外を空配列の根拠にせず、未確定ならnullとfixtureを保持する |

このmapの前半をdriverの起動前検査へ、後半を次の独立consumer preflightへ結合する。後半をdriver内へ複製した汎用validatorは追加しない。通常focusedでは後半を未変更のInvoke自身が実OS条件で再検証する。HANDOFFは現行実行commandを1個へ更新し、`$out = Join-Path "$r/verification" ([guid]::NewGuid().ToString('D'))`と固定slash driver／native source381F5E1D…/EXEFF7C9A0F…を指定する。旧nested例は「履歴・実行禁止」と明示するか現行commandへの参照に置換し、旧失敗rootとRED証拠は保存する。

### 有限preflightと証拠の区分

独立作者はpacket配下の新context-testだけを所有する。元TestのNew-CaseFixture/Publish-FixtureRun/Get-ConfigSnapshot、Invokeのpath/hash/schema/fixture/初期化関数をASTでそのまま取り出す既存手段を使い、source bytes/SHAと抽出元を記録する。新framework／gate mode／production test seamは追加しない。正常fixtureはENV01の無刺激形を元producerで生成し、sidecarはInvoke-GateCase内の既存8field発行statementを抽出して使用する。各caseに新しいscope・Immutable・Run/ValidationInput/RejectionContext状態を与え、変異は読取前に1箇所だけ行い、整合を検証するcaseでは関連hashを変異後bytesへ正しく結合する。旧runを補修しない。

このpreflightで**実RunOwner起動は0回**とする。元Invokeの初期化prefixと関数を別の隔離scopeへ読み、既存param契約どおりMode=RunOwner、ParameterSet=Control、RunPath=当該新Runの絶対path、InvocationPath未指定を与えて元Initialize-Runを呼ぶ。Test側の初期化scopeや前caseのscript状態を流用せず、main/dispatchは呼ばない。CXT-PRE-01への修正として、入力設計SHA `82A4774AA899628BCBC4E821386E1F8FD7DB1950473C495390B3B673019A3D36`の「4関数を合成5field identityへ置換」という一括記述を次の型付き契約へ置き換える。選択は**Ownersを空に保ち、ownerのOS同定・登録・liveness監督を省くfile/context試験**である。mock handleは作らない。元Assert-Control、Get-Identity本体、file/hash/schema consumerを編集しない。

test入力の自己Sは`pid:int`を現在preflight pwshの実PID、`startTimeUtc:string`を保持した実現在processの開始UTC7桁、`path:string`を実MainModule.FileNameの既存canonical値、`parentPid:int`を後述C.pid、`commandLineUtf16:int[]`を固定の合成非NUL文字列から元ConvertTo-CodeUnitで作る5field ordered dictionaryとする。controller Cもこの**同じ実現在process**のPID/開始/pathを用い、parentPid=0、commandLineUtf16は別の固定合成文字列とする。descriptorとsidecarのcontroller/launcherはCの同内容コピーである。S.parentPid=C.pidという自己参照は2processのOS関係を表さない明示的なtest代用であり、実親・command・所有証拠へ保存しない。存在しないPIDを開く前提はない。合成OS行R0はSに対応する`ProcessId:int,ParentProcessId:int,CreationDate:DateTime(Utc),ExecutablePath:string,CommandLine:string`を持つPSCustomObject1個とし、CIM実測とは記録しない。

| 隔離scopeの置換関数 | 正確な引数・成功出力・状態更新 |
| --- | --- |
| Get-OsProcess | 引数なし。成功streamに`return ,([object[]]@($R0))`相当の**OS行配列1個**だけを出す。identity dictionaryを返さない。呼出をtest専用logへ記録し、gate状態更新・CIM照会なし |
| Get-Identity | 元どおり`([Diagnostics.Process] Process, [object[]] Rows)`。Processは元consumerが渡す実GetCurrentProcess結果であり、IdがS.pid、保持SafeHandle有効、開始時刻がSと一致、Rowsが上記R0配列と同じ型/全field値であることを確認してSの5field ordered dictionary1個だけを返す。ProcessをDisposeしない。別Process/Rowsを受理せずthrowする |
| Assert-Identity | 元どおり`([object] Identity, [object[]] Rows)`。元Get-IdentityKeyのschemaを確認し、IdentityがC全5fieldと一致、RowsがR0配列と一致する時だけ成功する。戻り値・success stream出力なし、gate状態更新なし。GetProcessByIdを呼ばず、実OS再同定を行ったとしない |
| Add-Owner | 元どおり`([object] Identity)`。置換Assert-Identityを`Identity, (Get-OsProcess)`で呼び、Cとの一致を確認する。戻り値・success stream出力なし。test logへ引数と`registrationOmitted=true`を記録し、**Ownersへ追加しない**。GetProcessById／handle確保／Kill／Waitを呼ばない |

元prefixが作るOwners（List）、Known、Unprovableは各case開始から終了までCount=0を必須とする。logやassertion結果をsuccess streamへ漏らさず、元関数の消費側へ追加objectを混ぜない。C0ではGet-Identityの2回、Add-Ownerの1回とInitialize-FixtureからのAssert-Identity到達を確認し、各出力型/個数・引数・Owners空を既存試行内で検査する。負例では到達した呼出だけを検査し、拒否後の未到達を成功扱いしない。元Assert-Controlのcancel/UTC/monotonic検査は動くが、Ownersのforeachは0回であり、Refresh/HasExited/StartTimeによるowner監督は**未実行・未検証**と記録する。OS親世代・所有登録・監督喪失・terminal/cleanupのPASSを一切付与しない。

残る実OSアクセスは現在process自身だけである。Initialize-RejectionContextのGetCurrentProcess/SafeHandle/finally Dispose、Initialize-RunのWindowsIdentity/Admin照合、実GetCurrentProcess.MainModule.FileNameと固定pwshの比較、終端Get-Identityへ渡す実GetCurrentProcessは変更しない。したがってtestは固定pwsh7.6.6の通常ユーザーprocessで実行し、取得不能ならcontext試験失敗とする。fixtureはcargoKind=fake、syncPoints/observationFaults空の無刺激形を維持する。payload切出しは最初のcargoRegistration読取前で止めるため後段のGetProcessById/owner登録へ到達しない。Start/cleanup/Complete-BootstrapAudit等も呼ばない。この到達範囲をASTで確認し、型を保つために合成PIDのProcessを生成する回避をしない。

payload検査だけは元Initialize-RunnerChainの入口から最初のcargoRegistration読取直前までの同じstatementを抽出して実行し、payload名/UUID/nestingの受理／拒否を確認する。後続登録・親鎖への到達を主張しない。Start-FixtureProcess、Invoke-GateCase全体、Invoke-IdentityControlRegression、native/target/監督開始・停止・wait経路はpreflightの呼出対象から外し、隔離test内に呼出禁止trapと個別attempt counterを置く。正常・負例ともRunOwner/native/target実起動0、attempt0を必要とし、trap到達を期待拒否のPASSにしない。invalid入力の本番gate拒否や実回収は後続focusedの証拠と分ける。

書込は新しい`R/verification/<preflight-UUID>`とそのcase子、元producerが新規生成する`R/runs/<run-UUID>`、packet配下の新しい結果rootだけに限定する。旧nested負例だけは新しい`R/verification/argv-path01/<negative-UUID>`を使い、存在済み失敗rootへ書かない。fakeのcopy元は保存suite `argv-path01/5babecf8-0678-4bd2-8bab-ae73c540c952/fake.exe`、SHA=`286E1EFB55A9EAD69D097F6EAEF76C85795145AEE3CC4829CA9D429D41FF5461`と同toolchain.jsonのsource3482075Bを照合して使う。copyするだけで実行・再buildしない。不一致なら停止する。junction負例は新合成root内のdirectoryだけを指す通常権限のjunction1個までとし、作成不能時に権限を追加せず未検証として停止する。

下表は固定12試行で、正常1＋負例11を各1回、予期しない結果で残りNotRun、retry0である。各試行15秒・全体120秒の早いmonotonic上限を入口／consumer前後で確認し、超過は失敗として残りを実行しない。同期file/OS呼出を強制中断できる保証はなく、その応答条件を明記する。この上限はtestの起動抑止予算であり、Runの65/25/55/10秒やcleanupを延長・置換しない。正常consumer確認に実RunOwnerを追加する必要はなく、実OS親／cleanupは既定focused10件の段階へ留める。

| CXT試行／G5・AP4 | 元consumerで必要な結果 |
| --- | --- |
| C0 正常direct suite | 元Get-ConfigSnapshot4path、Initialize-Runのcontext確定とRun昇格、元payload入口の正確なnesting受理。模擬identityを除くfile/hash/source/fixtureの通過を記録し、runtime/cleanup PASSとしない |
| C1 旧nested suite | fixtureRoot=`R/verification/argv-path01/<UUID>/ENV01`、suiteId=`<UUID>`の元fixtureをInitialize-RejectionContextの厳密prefixで拒否する。修正driverの起動前guard拒否も別記する。suiteIdを改変しない |
| C2 repository未初期化 | 隔離scopeからrepositoryだけを欠かし、元Get-ConfigSnapshotが実際に失敗する。通常scopeの4path成功と組にする。driverのcallback到達だけを検出証拠にしない |
| C3/C4 marker欠落／文言違い | 別fixture各1回で元path/file/marker検査が拒否する。既存fixtureを壊さない |
| C5/C6 descriptorの別runId／別suiteId | 各1回、descriptorとそのhashを整合させた上でrun同一性／root境界の実validatorが拒否する。単なるSHA不一致で代替しない |
| C7 別Run位置 | 有効D形式でもrunIdとcanonical Run位置が異なる入力を元Initialize-RejectionContextが拒否する |
| C8 別payload nesting | 新Run内の別階層に合成payloadを用意し、元Initialize-RunnerChain入口のname/UUID/nestingが拒否する。nativeを起動しない |
| C9 reparse | 合成junction配下のfixture pathを元Get-CanonicalPathで拒否する。文字列guardだけを試さない |
| C10 期限不正 | sidecar deadlineUtc≤startedUtcの1変異を元context時刻validatorで拒否する。OS時計変更なし |
| C11 TEMP不一致 | sidecarには有効なfixture内TEMP/TMP、Run.envMap.TEMPだけ別の有効fixture内directoryを指し、元Initialize-Runのcontext/Run一致比較で拒否する |

初期化schemaやmarkerをconsumer通過済みと扱うには上記の実path/file結果が必要である。source-bound origin、ValidateOnly、mock identity、native未到達だけからCXT01 ClosedやAP1 GREENを宣言しない。新独立testの期待値を先に固定して旧driver89C6に対するnested契約のREDを示し、別実装者がdriver/HANDOFFのcontext部分だけを是正する。新context-testと既存argv oracle7D74/10case/slash17引数は実装者が変更しない。

Preは本書の返却SHA、計画E209…、driver89C6…/HANDOFF FB1…と固定native/Invoke/Test/fakeを対象とする。順序は設計受領・Pre→独立作者12試行固定→harness実装→独立preflight→接合review/親受領→合法な新suiteでfocused10件1回→Post→標準80件である。最初のfocused失敗で残りNotRun、実OS同世代・実terminal・実cleanup・argv証拠を別記する。新前提不整合、元consumer緩和、権限追加、想定外のscope変更が必要なら停止して親へ返す。rollbackは保存driver89C6/HANDOFF FB1への当該差分だけで、復帰後も実行不可の既知不整合状態である。native2定数／旧証拠／他者編集を戻さない。

完了境界は本追補と文書diff/link/hash確認のみである。役・model/effortは計画のCXT表を維持し、次は独立Preへ返す。開始時計観測2026-10-16T00:02:05.1877712+09:00、token開始/終了・週枠・provider返却model/effortは未取得。code/tests/runtime/新文書/再委譲/公開0、CXT01受入は未完了である。

## KNOWN-HANDLE01：既登録の同世代handleを元snapshotで再照合する（KH-A、設計Pre提出）

入力は[計画244行以降](early-trial-gate-plan.md) SHA256 `956C0CDA0802654176C95378A5E4FA92A9FACE18EB75670E9F731D9A13977AA1` 全節、[review冒頭KH-REOPEN-01](early-trial-delivery-review.md)、Invoke `12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621`、追補前本書 `D1C38FB3557D067A5E0B414E6DAEA021123C66B64D6FB5C62930B271CC5EFADF` である。承認済みET契約と既存G5（同世代所有・未知非停止）の内部是正であり、新外部仕様・UI・RESTは非該当。計画のdesigner指定 `gpt-6-astra/high` を維持する。AGENTS/development-team/KPI、api-and-interface-design、powershell-review、同automationのps7/例外/型・所有規約を読了した。汎用module/config/log追加は本packetに適用しない。

保存suite `68cfe879-a4ea-4665-83f0-8bff89cd2b2b`／run `94e5f1c6-43bb-4ca6-98c7-80eb443f76fa` の `result-RunOwner-21556.json` ではPID23100はOsQuery/Observed、start=`2026-10-15T20:38:03.9529175Z`、parent14632、path=pwsh、command590 code unitsの完全identityである。`safety/list/result.json.b7e3c0ac-7b89-4cfb-9e64-49e5b98863fd.tmp` のPID6472もOsQuery/Observed、start=`2026-10-15T20:38:03.9605517Z`、parent23100、path=conhost、command39 code unitsの完全identityである。同tmpのPID23100はDirectLaunchHandle/Observedだが5fieldが揃っており部分identityではない。両failureの1270行、snapshotMatch=knownMatch=1、path/command存在と別Owned行の保存を確認した。この情報は修正対象の分類根拠であり、旧UnprovableをOwnedへ読み替える証拠ではない。最初の公開cancelはSupervisor、正式Supervisor/Safety terminal欠落で全時系列の初因は断定しない。

### 唯一の方式と既存consumerの成立性

方式は **Read-OwnedTreeが元Rows・台帳keyで候補を一意化し、その既存Processを借用して未変更Get-Identityを呼び、全5fieldをTest-EqualDataで照合する** ものに固定する。新private照合helperは不要である。変更所有はRead-OwnedTree内だけであり、Get-Identity/Add-KnownProcess/Read-DirectChildIdentity/Get-TerminationObservation/Complete-OwnedTreeとその別consumerを変更しない。

Invoke429–475のGet-Identityは、SafeHandleを確保→Refresh→元Rows一意性→CreationDateのUTC6桁とStartTime比較→5field生成の順である。MainModuleを読まず、pathが非nullならimage APIも呼ばない。終了だけを理由に拒否する分岐はない。固定pwshのruntimeconfigはnet10.0/10.0.12であり、公式同tagの[Process.cs](https://github.com/dotnet/runtime/blob/v10.0.12/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.cs) のSafeHandle/GetOrOpenProcessHandle/Refreshと[Process.Windows.cs](https://github.com/dotnet/runtime/blob/v10.0.12/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Windows.cs) のStartTimeCore/GetProcessTimes/GetProcessHandleを参照した。Refreshは保持handleを解除せず、既存handle時のSafeHandleは再openしない。StartTimeはthrowIfExited=falseの保持handleからGetProcessTimesへ進む。したがって有効な同一保持handleと完全な元行があれば既存Get-Identityを再利用できる。これはsource上の成立性であり、同梱binaryの実終了対照はKH-O2で別に検証する。追加OS診断は実施していない。

新しいprivate identity生成・新Win32 APIは採らない。既存のUTC6/7桁比較、path canonical化、UTF16変換を二重実装すると負例とpath-null/PTH-MONOの接続が分岐するためである。PID一致skipや例外握潰しは世代の証明にならず採らない。snapshotのExecutablePath=nullは既存Get-Identityの同一handle path取得へそのまま渡す。そのAPIが失敗すれば元catchへ戻し、台帳pathで補完しない。command/null CreationDate等も既存拒否を維持する。

### KH1–3の契約・状態・handle所有

| ID／既存G | モジュール・入力型 | 成功／失敗・所有 |
| --- | --- | --- |
| KH1/G5 | Read-OwnedTree、`Rows:object[]` は当該呼出のGet-OsProcess結果1個。各rowはCIMのProcessId/ParentProcessId/CreationDate/ExecutablePath/CommandLine。`Known:Dictionary<string,object>` のkeyは既存PID/UTC7桁 | snapshotを取り直さず、同PIDの元行1件・同UTC6桁候補1件・handle/全identity一致だけで既知継続。候補/行の曖昧さはthrow |
| KH2/G5 | `known.handle:Diagnostics.Process`、`known.pinnedHandle:SafeProcessHandle`、identityの既存5field、role/ownershipEvidence/exitUtc | 既存台帳の借用であり、Add/Dispose/Kill/Waitは0。identity/role/evidence/keyを更新しない。終了は冒頭の実Refresh/HasExited/ExitTimeと既存cleanup consumerだけで観測 |
| KH3/G5 | 新規世代は元GetProcessById→Get-Identity→親寿命→Add-KnownProcess | new handleはlocal所有、登録成功時だけ台帳へ移譲しlocalをnull。失敗時だけlocalをDispose。既知照合失敗は新規openへfallbackせず既存Unprovable/cancel/cleanupへ返す |

Read-OwnedTreeの同期処理を次の順序へ局所変更する。非同期worker、追加CIM、retry、新時計は追加しない。

1. 元のmanifest確認、1回のGet-OsProcess、既存KnownのRefresh/終了時刻更新は維持する。ここで実disposed Process等がthrowした場合は元callerへ伝播し、cleanupは既存のincompleteとなる。これをrow内Unprovableへ合成しない。
2. `foreach row` のself PID除外後、local `$handle=$null` と既存try/catch/finallyの内側で、**親PIDによる新規候補除外より前**に同PID Knownのkey/valueを列挙する。known rowの親fieldが改変されて親候補0になっても既知検証をすり抜けさせない。同PID Knownが0かつ親候補0の無関係rowは従来どおり無視する。
3. 同PID Knownがある時は、元Rows内の当該PID行が厳密に1件、CreationDateがtimezone付きDateTimeであることを確認する。全候補のdictionary keyとGet-IdentityKey(identity)のOrdinal一致を確認する。UTC6桁はGet-Identityと同じinvariant書式 `yyyy-MM-ddTHH:mm:ss.ffffffZ` で比較し、差分許容・丸め・double変換を加えない。別6桁世代のKnownは残したまま候補外とする。
4. 同6桁候補が2件以上なら曖昧として拒否する（7桁だけ異なる候補も同じ扱い）。1件なら完全identityだけを対象にする。path非null文字列、parentPid非null整数、commandLineUtf16非null配列であることを確認する。部分DirectLaunchIdentityはこのbranchでthrowし、元direct-launch証拠と台帳を保持する。ここで補完・再登録・別handleへの置換を行わない。同6桁候補0なら新規世代の既存経路へ進む。
5. 候補のpinnedHandleが非null、IsClosed/IsInvalidがともにfalseであることを **known.handle.SafeHandleの取得前** に確認する。handle型はDiagnostics.Process、Idはrow PID、取得したSafeHandleはpinnedHandleとReferenceEqualsでなければ拒否する。借用Processをlocal `$handle` に代入しない。保持対象をClose/Dispose/Start等で再関連付けするconsumerは追加しない。
6. `Get-Identity known.handle rows` の戻り5fieldと台帳identityを元Test-EqualDataで完全照合する。戻りkeyもdictionary keyとOrdinal一致を必須とする。これによりrowのparent/path/command、CIM6桁、保持handleのUTC7桁が接合する。照合後もpinnedHandleの有効性/参照同一性を確認し、成功時だけcontinueする。途中例外や不一致は元catchへ渡す。既存FaultTargetsは末尾Get-FaultTargetおよびcleanupのGet-OwnershipObservation/Get-TerminationObservationで引き続き適用される。
7. 候補0のnew経路は従来の親候補抽出、PID open、identity、親寿命一意性、登録を維持する。新経路のkeyが予想に反してKnownに存在したら、未照合のままcontinueせずthrowする。既知照合失敗やpartialをopen成功で覆わない。新登録によるwhile再走時は新台帳も手順2–6で照合する。

失敗時のrow catchは元Get-IdentityFailureObservationとUnprovable追加を維持する。`identity=null, ownership=Unprovable, killRequested=false, waited=false, hasExited=null`、既存Knownのkey/identity/evidence/handleは保持する。Aborting=falseなら元Invoke-GateFailure(CleanupIncomplete)で停止し、trueなら既存cleanup観測としてUnprovableを保持する。観測値を0/trueへ補わない。Read自身の成功はterminal成功ではなく、Ownedに対するkill/wait/終了確認、fault拒否、fixture残留、10秒cleanupの判定を省略しない。

新永続状態・新approvalはない。元台帳登録→同handle保持→終了観測→cancel公開→cleanup→terminalという既存順序と、非上書きJSON公開・tmp未公開の区分を維持する。既知照合中のcancel/期限は元callerとGet-Identityのpath fallback consumerが担い、待機や予算を追加しない。PTH-MONO local clock/budget、UTC精度、元fault・unknown非停止条件は不変である。

### 固定22回帰と独立作者のseam（実行はKH-A Pre・planner確定後）

作者は `R/verification/argv-path01/Test-KnownHandle.ps1` 1本を新規所有する。既存Test-PinnedImagePath/Test-ReadBoundariesのsource SHA検査・AST抽出・局所置換・case結果の方式だけを再利用し、新frameworkやproduction fault APIを作らない。`SourcePath:string, ExpectedSourceSha256:64hex, OutputRoot:string, Phase:{Pure,Os}` を必須引数とする。新OutputRootは `R/verification/argv-path01/<UUID>` 直下、旧証拠への上書きなし。caseは下表の20pure＋2OS、retry0、同一oracleを旧Invoke12E7と新Invokeへ適用する。

pureは実Read-OwnedTree ASTを抽出し、同関数内のGetProcessById static呼出2箇所だけを、同じ引数を受ける `Get-TestProcess([int] ProcessId)` へ置換する（1箇所はrun-wide末尾、SafetyWorker modeなので未到達）。個数/位置/引数/逆置換本文一致を検査し、適合失敗はbehavior REDに数えない。各caseをfresh scopeでMode=SafetyWorker、Fixture=null、Manifest=null、Owners空、Known/Unprovable新規にする。

pureのprocessは `PSCustomObject` の `Id:int, StartTime:DateTime, HasExited:bool, ExitTime:DateTime, SafeHandle:PSCustomObject` を持ち、SafeHandleには `IsClosed:bool, IsInvalid:bool` を持たせる。Refresh/Disposeは引数なし・出力なしのScriptMethod、呼出回数だけをtest台帳へ記録する。実SafeProcessHandle/Diagnostics.Processと偽装せず、型検査を含む実機契約はOS phaseで検証する。**Read内のDiagnostics.Process型検査だけ**はpure AST上でPSCustomObject型へ置換し、OS phaseでは原文を使う。この型検査ASTは旧12E7に0件、新sourceに1件を固定し、他の型やstatementを置換しない。public/本体param型は変更しない。

Get-OsProcess代用は`return ,([object[]] rows)`の1出力で、CIM0。Get-TestProcessは登録済みtest process1個だけを返すか固定例外をthrowする。Get-Identity代用は`([object] Process,[object[]] Rows)`を受け、参照同一性を検査してcase所定の5field ordered dictionary1個を返すか固定例外をthrowする。これはselection/error伝播の試験であり、元Get-IdentityのOS同定を試験したとは記録しない。Add-KnownProcess代用は`([object] Handle,[object] Identity,[string] Role,[object] OwnershipEvidence=$null)`、void、元形式の台帳1件を追加し、new移譲を検査する。Get-IdentityKey/Test-EqualData/Get-Utc/Get-IdentityFailureObservation/Invoke-GateFailure/Get-FaultTargetは元定義を使う。script Clock、AbortTick/Utc、Status等は元prefixと同型で初期化する。

counterは `osQuery, pidOpen, getIdentity, addKnown, borrowedDispose, newDispose, kill, wait, terminalWrite:int` と順序log、Knownの前後key/identity/evidenceおよびUnprovable全fieldを保存する。PureのosQuery/kill/wait/terminalWriteは全0、Get-OsProcess代用呼出は各Read1回。borrowedDisposeは全0を必須とし、型代用や未到達を実OS安全性のPASSにしない。実行経路外のStart/Complete/terminal/承認/file書込はtrapし、trap到達は期待拒否ではなく失敗である。

| pure ID | 入力・固定oracle（各1回） |
| --- | --- |
| KH-P01 | live完全Known。pidOpen0/getIdentity1/addKnown0、Known同一、unknown0 |
| KH-P02 | snapshot後終了を代用した完全Known。P01と同じ、exitUtcだけ元冒頭で取得。旧12E7はopen1でnot-running固定例外→unknown1のRED、新はopen0のGREEN |
| KH-P03 | 同PID別UTC6桁、親寿命内の新規世代。open1/add1、旧key保持＋new key追加。while再走の既知確認を含めgetIdentity2 |
| KH-P04 | 同PID同UTC6桁でUTC7桁だけ異なるKnown2件。open0/getIdentity0、曖昧拒否 |
| KH-P05 | 同PID別UTC6桁のKnown2件中1件だけrowに一致。open0/getIdentity1、選択した参照だけ使用、両key保持 |
| KH-P06 | 元Rowsに同PID2行。open0/getIdentity0で拒否、追加CIM0 |
| KH-P07/P08 | CreationDate=null／Kind=Unspecified。各open0/getIdentity0で拒否 |
| KH-P09 | command欠測でGet-Identity代用が元失敗相当をthrow。open0/getIdentity1、unknown保持 |
| KH-P10/P11/P12 | 戻りidentityのpath／parent（親候補0）／UTF16 code unit1個が台帳と相違。各open0/getIdentity1で拒否 |
| KH-P13/P14/P15 | pinned IsClosed／IsInvalid／別SafeHandle参照。各open0/getIdentity0で拒否。ここではRefreshは成功する代用でrow拒否を検証。実disposedによる冒頭例外は別の未検証経路として明記 |
| KH-P16 | path/parent/command=nullの部分DirectLaunchHandle identity。open0/getIdentity0で拒否、partial bytes/evidence不変 |
| KH-P17 | 未登録候補がopen前消失。open1/getIdentity0/add0、unknown1/newDispose0。例外は成功化しない |
| KH-P18 | 通常の新規子、親寿命内。open1/add1、Known+1、unknown0、getIdentity2（while再走を含む）、newDispose0 |
| KH-P19 | 同UTC6桁候補だが戻りUTC7桁/keyが異なる。open0/getIdentity1で拒否 |
| KH-P20 | 元row path=null、Get-Identity代用でpath取得失敗。open0/getIdentity1で拒否。台帳pathへのfallback禁止。path API自体の既存47＋5受入とは区別 |

P04/06–17/19/20の拒否はAborting=falseを使い、Unprovable1、Status=CleanupIncomplete、元Known不変、kill/wait/terminal0、以後case依存処理未実行を必須とする。新経路でopenに成功後失敗した時だけnewDispose1となるが本表P17はopen自体がthrowなので0である。各pure case5秒、Pure全体60秒の早いmonotonic上限を入口/consumer前後で確認する。既知のRED（P02等）は期待差分として保存して他の固定caseへ進み、adapter/fixture/期限など想定外の失敗はfirstUnexpectedで残NotRunとする。新sourceは20/20 PASS以外を受理しない。

OS phaseは元Get-Identity/Add-KnownProcess/Get-TerminationObservationと実Read-OwnedTreeを使用する。Get-OsProcessだけを採取済み実CIM rowsを返す1出力関数へ限定置換し、ReadのGetProcessById2箇所はcounter経由で元APIへそのまま渡す（既知経路で0を証明する）。型代用は0である。新隔離pwsh合成子1個を通常ユーザー・hidden、stdin読取で待機させ、親はStartが返したProcessとSafeHandleを保持する。引数は固定文字列、子はstdin EOFで正常終了し、独自10秒上限でも終了する。開始からの同じ15秒monotonic上限内でO1→O2を行い、O1失敗時はO2を起動しない。作者自身のfinally回収もこの上限内、保持launch handleのみに限定する。取得/回収不能は非PASSとして証拠を残し、PID停止・retry・権限追加はしない。

| OS ID | 同一合成子による固定oracle |
| --- | --- |
| KH-O1 | live中に実CIMを1回採取し、selfと合成子の実row各1件だけを抽出したobject[]を固定する（field改変なし）。完全path/commandを要求。元Get-Identity→元Add-KnownProcessで子を登録する（親edge用Knownは実selfの保持Process/実row identity、self除外は元処理）。登録用counterとRead用counterを分離し、Readに同snapshotを渡してopen0・unknown0・登録後identity/evidence/handle不変、実getIdentity1を確認する。typed consumer成立の最小正常確認 |
| KH-O2 | O1成功後stdinを閉じ、同launch handleの有限WaitForExitで終了を確認する。**CIM再取得せず**O1のrowsのまま元Readを呼び、open0・unknown0・同UTC7桁の完全identityを確認し、元Get-TerminationObservationでExitedを観測する。Readによる借用Dispose/Kill/Waitは0。runner自身のwaitと終了確認を別counterへ記録し、gate cleanup/terminal成功には数えない |

OS phaseは2件・全15秒、うち各caseも15秒以下、finallyを含む。同期CIM/OS APIの強制中断を保証せず、返却後の超過は失敗として後続抑止する。子自身の10秒上限はfixture側の上限でありgateの10秒cleanupを変更しない。旧sourceのOS phaseはO1がpidOpen>0でoracle不一致ならO2はNotRun、旧競合REDは決定的なP02で担保する。新sourceのOS 2PASSは作者と独立runnerがそれぞれ固定1回取得する。pure20は制御論理、OS2は実handle/元identity/終了観測、focused10は実監督全体と証拠を分ける。

command契約は下記とする。`$frozenOld` は作者が12E7をbyte保存した絶対path、`$frozenNew/$newSha` はKH-B確定入力であり、存在/hash一致がない限り起動しない。Pure/Osそれぞれ新UUID outputを使う。expected hashを現物から無条件に採番して承認済みと扱わない。

```powershell
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File "$r/verification/argv-path01/Test-KnownHandle.ps1" -SourcePath $frozenOld -ExpectedSourceSha256 '12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621' -OutputRoot $freshOldPure -Phase Pure
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File "$r/verification/argv-path01/Test-KnownHandle.ps1" -SourcePath $frozenNew -ExpectedSourceSha256 $newSha -OutputRoot $freshNewPure -Phase Pure
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File "$r/verification/argv-path01/Test-KnownHandle.ps1" -SourcePath $frozenNew -ExpectedSourceSha256 $newSha -OutputRoot $freshNewOs -Phase Os
```

runnerは新sourceのOS2を最初に1回、成功時だけPure20を1回、KH-C静的接合PASS後の既定focused10を1回実行する。focused commandは既存slash driverのNativeSourceSha256=`381F5E1DCD70ED8040153FB155C8AABA0881FBFB57A8074B286FB4169D8683DD`、NativeExeSha256=`FF7C9A0F641F66F7C340AEC94C21EA9A0C745EA5A39C35B252C3A9CBDAFCCB88`、OutputRoot=`R/verification/<新UUID>`を保持する。既存case/Runの65秒（build25、実行55＋cleanup10）、firstFail/残NotRun/retry0を変えない。focusedの全体wall上限を新たに設定・延長せず既存driver/元fixtureの有限契約を使う。OS15秒/Pure60秒は局所試験だけの上限である。

### 凍結・受渡し・停止条件

KH-A完了はこの追補と静的self-check・独立Preへの提出までである。未決だった(1)既存Get-Identity再利用、(2)partial/複数世代/無効handle、(3)局所22件・型/counter/時間/commandを上記一方式へ確定した。実装・試験起動は行っていない。追加の仕様判断が必要なら外部仕様担当へ返し、比較・所有・期限・権限の緩和で解消しない。

Pre→planner精緻化→独立作者の20pure/旧RED固定→別実装者のRead-OwnedTree局所修正→作者新GREEN/OS2→独立B/C接合→独立runner OS2/Pure20/focused10→独立Postの順である。新sourceで最小正常OSが失敗したら新findingとして停止し、harnessを一般化する連続patchへ進まない。parser/PSSA1.25.0は診断0かつ実行error0、逆diffで非所有関数本文不変を必要とする。実装者は期待値を編集しない。

frozen bindingは計画どおり、最終plan/design/Invoke→Test-ContextPreflightの3入力hash→新testとCONTEXT-TEST-HANDOFF-P1→focused driverの5入力hash→HANDOFF/runner frozenの順でのみ更新する。今は更新しない。旧bytesを保存し、旧結果・承認・Run source/frozen-inputを上書きしない。CXT-D旧12PASS/独立PostとPTH-MONO Closedは維持し、関係consumer本文不変の静的証拠でCXT12の不要な再実行を避ける。LIFE14/16/17/18・標準80/87variant・製品full・提供のgateは残る。

rollbackはRead-OwnedTreeの今回差分だけを12E7保存版へ戻すことであり、旧競合や旧binding不一致を解除しない。新回帰/RED/旧source・tmp・fixtureは保持する。他者編集、native/fake/EXE、製品source、認証/API/OS設定は変更しない。usageは子`/root/known_handle_race_design`の開始/終了counter・provider実metadata・週枠・親子包含を未取得とし推定しない。指定model/effortと返却実値を混同しない。
