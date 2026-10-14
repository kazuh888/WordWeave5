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

Manifestのtop keyは`runId,runSha256,cfg,cargoIdentity,sourceDigest,g0Digest,markerDigest,jsonPrefixBytes,jsonPrefixSha256,artifacts,createdUtc`、Requestは`runId,invocationId,cfg,cargoIdentity,entryIdentity,runnerIdentity,payloadSha256,executable,argvUtf16`である。Registration／Readyは`runId,role,scopeId,intentSha256,identity,ownerIdentity`、ACKはこれに`readySha256,manifestSha256,caseId,deadlineUtc`を加え、非caseではcaseId=null、Manifest作成前のSupervisor登録ではmanifestSha256=nullとする。開始Intentは`runId,role,scopeId,ownerIdentity,path,argvUtf16,deadlineUtc`を持つ。SafetyInvocationは`runId,runSha256,manifestSha256,g0ApprovalSha256,caseId,order,executable,exactName,list,requireChildMarker,v2Case,outputDirectory,g0DeadlineUtc,deadlineUtc`であり、executable/v2CaseはPathHash。Resultは本書の既定内容を`runId,role,scopeId,identity,manifestSha256,requestSha256,ackSha256,argvUtf16,childExitInt32,childExitU32,gateStatus,startedUtc,endedUtc,timeout,cancelled,cleanupDeadlineUtc,processes,fixtureResiduals,sourceMatched`へ対応させる。未到達のidentity／hash／exitはnull、未取得配列はnullと空配列（取得済み0件）を区別する。processesの行は`{identity,killRequested,waited,hasExited,ownership,error}`、ownershipは`Owned,Unrelated,Unprovable`である。

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
