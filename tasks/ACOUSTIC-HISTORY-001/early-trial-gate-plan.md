# ET3-G：一時Cargo runnerによる実行前照合（Pre-2提出版）

## LIFE14限定是正計画（2026-10-15再開）

利用者は95%共有週枠の報告後も継続を承認した。plannerのread-only照合では、caseRowのRegistration/Ready検証に対し現刺激はcontrollerIdentityを破損し、bootstrapで拒否され目的validatorへ到達しない。監査contextを緩和せず、正常controllerを維持し検証前Readyの対象identityだけを変異する案を採る。外部仕様担当はRegistration/Ready不一致拒否と実OS世代照合の目的を区別して確認し、designerは有限な検証前同期を具体化する。既存Rejected/非zero/zeroAll/既知cleanup/sentinel生存と80行を維持し、未知非停止/非PASSを変えない。

所有はdesignerの既存設計LIFE14追補、作者のTestPS当該刺激/同期/到達観測、実装のInvokePS最小同期/診断、runnerの新隔離証拠、reviewerのread-onlyである。native/fake/製品source/実data/API/資格情報/OSは変更しない。planner/仕様/設計/作者/実装/reviewerはgpt-6-astra/high、当該単位runnerはplanner指定gpt-6-sol/medium（既定ET3全体runnerとは別割当）。各モデルは利用可能一覧にある。指定と実返却metadataを混同しない。

順序は限定仕様→設計/独立Pre→作者/実装→独立focused実行/正常・影響対照→Postである。目的validator未到達での拒否を成功に数えない。局所Post後も接合レビューと新固定snapshot通常入口80、実Cargo/製品full、native、試用提供のgateは残る。

2026-10-14。親から一時process限定runner導入と、rfd DPI 150/200%だけの利用者試用確認への移管承認を受領した。Pre-1のP1阻止2件は設計修正・本計画同期を完了し、独立Pre-2待ちである。Pre-1 CHANGES_REQUIREDの判定は履歴として保持し、修正だけでClosedとしない。Pre-2 PASS→親受領まで作者/実装/runtimeへ進まない。既存未完了計画は変更しない。計画と作業リストの指定出力先は本書一本である。

## 完了境界・根拠

- 今回の追加単位は、Cargoのbuild後・対象EXE実行前に親の照合を挟む最小wrapperと、その合成対照・独立検証までである。その後に既定ET3全体gateを再開する。wrapper合格をET3完了としない。
- 着手HEADは `6fb6da3b568eb8e4e902f66260ee5df48f9a00a2`、git statusはclean。製品sourceが `cb882031` と同体であることと局所原因は親から受領した。[delivery review](early-trial-delivery-review.md)末尾と[runner](early-trial-delivery-runner.md)のfull2証拠を再利用し、原因調査や成功済みrelease buildを反復しない。
- U+3000を含むdep-infoの不存在prefixによる再buildが再生成後にも残るため、no-run時SHAの解除を次のCargo実行へ持ち越せない。[公式Cargo runner仕様](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)は対象EXEをrunnerへ渡す機構と環境変数を定義している。今回のWindows引数・待機・同定の安全性まで保証する根拠ではなく、合成対照で検証する。
- 外部仕様は既承認[ET契約](early-trial-contract.md)を再利用し、新規工程は非該当である。安全規範は[startup設計のET2-G0](early-trial-startup-design.md)と末尾の新helper込み固定対象である。新UXや契約変更が必要なら外部仕様担当へ戻す。

## 要求・除外

| ID | 必須要件・受入証拠 |
| --- | --- |
| G1 | `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER` は所有Cargo processにだけ設定する。構成別の固定合成LOCALAPPDATA/TEMP/TMP、repo cwd、JOBS2、既定target/features/commandsを維持する。永続config・親環境を変更しない。実値と終了後不変を記録する |
| G2 | Cargoが実際に渡したEXEの解決済み絶対path/SHA、同じ実行のartifact profile/features/target、source170項目・G0固定12項目・marker4の一致を親が照合するまで起動0。承認は今回のrun・構成・EXE・source組へ束縛し、旧runの承認は使えない |
| G3 | 未知EXE、profile/features違い、source/SHA変化、承認欠落・破損・拒否・期限切れでは実行せず非zero終了する。承認後もspawn直前の一致を再確認する。自己承認・自動fallbackなし。構成やEXE変更でG0失効する |
| G4 | Cargo由来の全argvを順序・空文字・Unicode・空白・引用符も含めて保持し、対象のstdout/stderrとexitをそのまま伝達する。skip/filter/ignore/test-thread等を追加しない。wrapper拒否・timeout・cleanup失敗を製品test FAIL又はPASSへ混ぜない |
| G5 | Admin=False、Hidden、有限の承認待ち期限と実行期限、所有treeのkill/wait、終了確認後cleanupとprocess/fixture残存0を維持する。途中失敗も同じ回収規則である。所有不明processを停止しない |
| G6 | G0に従い、新EXEで必要な録音→通信安全対照を先行し、その後にCargoの全対象実行を継続する。旧EXE対照の成功は新EXEの安全証拠にしない。既定full commandの対象集合・引数は変更しない |

変更禁止は製品source/test/期待値、依存、Cargo.lock、fingerprint、OS、global Cargo、認証・実データ、合否/対象test、親モデル/effort、既定品質gateである。実credential/API/mic、再委譲、commit/push/公開は対象外。汎用framework、Cargo parser修正、path alias、7役分の重複文書は作らない。

## 最小作業リスト・所有・依存

### G-D：待機と照合の設計（S、文書1本）

- [x] designerが `early-trial-gate-design.md` へrun識別・request/approval形式・状態遷移・期限・argv/exit・回収を具体化し、D1–D4とPre-1 P1二件を反映した。同一Cargo実行のJSONからartifactを収集し、no-runを別途反復しない。文書作成完了であり設計PASSではない。
- [ ] 独立Pre-2はG1–G6と既存G0の接合、EXE差替え/再build競合、native UTF-16 payload→固定PS parameter→target argv、RunOwner→Supervisor→SafetyWorkerの登録ACKと有限回収を確認する。設計のCmdletBinding/param固定宣言とmode/setの整合、未知option/CommonParameter名/空文字がbinderを通らないことを確認する。実argv対照は静的確認で代替不可。判定は親が既存 `early-trial-delivery-review.md` へ追記する。
- [x] 計画者が修正設計のpath/command/期限/対照集合へ本書を精緻化した。親受領と独立Preは未了。native問題は全体gateの未了項目へ保持する。

依存は本計画の親受領。受入は具体化した方式が独立Pre-2 PASSであること。検証は文書の相互整合・相対link・`git diff --check`。コード/runtimeは0である。

### G-T/I：独立対照作成→wrapper実装（M、ignored内の4file、生成fixture別）

- [ ] 独立test authorは `target/early-trial-build/et3/cargo-gate/` 内の `Test-CargoGate.ps1` と `fake.rs` だけを所有し、G1–G6の期待値を先に固定する。生成するfake EXEと最小Cargo接合fixtureは同root配下に限る。成功/非zero exit、argv境界、未知EXE/SHA/source/構成、旧・欠損・破損承認、承認後差替え、承認待ちtimeout/取消し、子孫残存の拒否を、fakeの起動markerと回収証拠で判定する。製品の実録音/通信を失敗対照へ用いない。
- [ ] 別implementerは同directoryの `Invoke-CargoGate.ps1` と `cargo_gate_entry.rs` だけを所有する。前者はRunOwner/Supervisor/Runner/SafetyWorkerの4内部mode、後者は標準libraryだけのnative入口である。`cargo-gate-entry.exe`も同ignored内へ生成する。既存Start-Process/所有照合パターンと証拠を再利用し、共有harnessを編集しない。full2元inline監督の保存scriptは未取得であり「既存scriptそのもの再利用」と記録しない。本体Cargoへ依存を追加せず、別launcher/Job追加は親/plannerへ戻す。
- [ ] implementerは構文検査・合成focused検証を行い、hash/結果/未解決を親へ返す。作者は実装都合で期待値を変更しない。

依存はG-D/Preと精緻化済み計画。受入は許可なし起動0、許可時argv/exit同一、拒否/timeout時の回収と残存0である。検証commandは下記へ固定し、作者は6群のケース実名/件数/対応を実装前に固定する。PowerShell作成担当は `powershell-automation` を全文読了し、レビュー担当は `powershell-review` を用いる。本計画ではスクリプトを作らないため、その実装skillはまだ適用しない。

### G-V：独立検証・Post→既定ET3へ返却（S、ログと既存報告）

- [ ] independent runnerが固定wrapper/test/fake hashで合成対照を実行し、実argv/exit/拒否時起動0/AdminFalse/Hidden/期限/killwait/残存を記録する。実pwsh 7.6.6とCargoの接合を同ignored配下の合成小fixtureで必ず確認してから製品へ適用する。独立性のためのこの確認と作者のfocused結果は区別する。
- [ ] read-only reviewerがG1–G6、所有diff、合成結果、設定不変を独立Postで判定する。指摘は各ownerへ返し、影響のない成功済み対照を繰り返さない。
- [ ] 親がPost受領後、通常→試用→関連crateを既定commandに観測用`--message-format=json`だけ追加して逐次dispatchする。各runの新artifact照合・G0安全順・明示解除は親責任であり、runnerへ自己承認を委譲しない。

runner所有は上記ignored配下のログ/合成データ、既存 `early-trial-delivery-runner.md` への結果追記である。reviewerはread-only、親が既存reviewと短い現状索引を記録する。全writerは共有workspaceで他者編集を戻さない。ignored pathは書込み前に解決済み絶対pathと `git check-ignore` を確認する。

## モデル・役別handoff

追加作業単位ET3-Gの5役を下表で固定する。既定ET3へ戻った後は[既存ET3割当](early-trial-plan.md)を維持する。全モデル/effortは親から受領した呼出可能一覧内であり、dispatch時にも照合する。指定とprovider返却metadataを別記し、未取得値を推定しない。

| 役 | exact model / effort | 難易度・危険と選定理由 | 所有/責務 |
| --- | --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 中/高。固定役。既存G0と限定承認の整合 | 本書のみ。今回と設計後の精緻化 |
| external-spec author | `gpt-6-astra` / `high` | 今回非該当、起動なし。既存ET契約を再利用 | 新仕様判断が発生した場合のみ親へ戻す |
| designer | `gpt-6-astra` / `high` | 高/高。親承認・process寿命・新artifactの時間順序 | G-D設計1本 |
| implementer | `gpt-6-astra` / `high` | 高/高。native引数保持と複数監督の失敗境界 | G-T/IのPS/native入口2fileのみ |
| test author | `gpt-6-astra` / `high` | 中/高。拒否時非実行・差替え・回収の独立oracle | G-T/I対照/fixtureのみ |
| test runner | `gpt-6.1-sol` / `medium` | 中/高。確定手順・hash・raw exitの収集、未解決判断は親へ | G-V実行/ログ/既存報告 |
| reviewer | `gpt-6-astra` / `high` | 高/高。G0不変、承認再利用と競合、証拠の独立判断 | Pre/Post read-only |

役別既定skillはdevelopment-teamの担当工程内で使う。designerはapi-and-interface-design、作者はtest-driven-development、implementer/runnerはwordweave-change、reviewerは設計時review-software-design・実装時code-review-and-qualityとpowershell-reviewである。skill/model不在はblockerであり無言置換しない。親モデル変更なし、通常補助1役ずつfresh context、作者/実装/runner/reviewerの独立性を保持する。

## リスク・未決事項・次handoff

1. 最重要リスクは親照合後の差替えとG0順序、監督喪失時の回収である。同runのEXE lock/再照合、先行G0、登録ACKとRunOwnerを具体化したが、独立PreとWindows実証は未了である。不成立なら停止する。
2. native runner絶対path一語とUTF-16 payloadにより任意argvのPS binder通過を除いた。Windows argv/exit/OS stream継承は合成対照で実証し、OS/global設定の変更を解法にしない。
3. native 100%/狭幅/日本語折返し/OK中央/focus・keyboard、実releaseの直接起動・合成学習/設定保存・再起動、既存音読導線の確認は未了である。DPI150/200だけは承認どおり利用者試用へ移管する。`--ui-check` の合成store差替えを実release永続化の成功へ代用しない。
4. nativeの安全な実行手段は未解決である。親は実credential/API/micへ到達しない根拠、合成store/実release同定、操作/画面観測手段と終了回収を確定してからnativeを解除する。現手段で満たせない場合はET3提供blockerとして返し、勝手に免除しない。OS変更・新source seam・追加権限が必要なら別判断へ戻す。
5. wrapper完了後も通常/試用all-targets、関連crate既定test/fmt、source/旧EXE保全、native、独立提供Postが残る。取得済み3release buildとFX1 84PASSは対応snapshotの証拠として接合し、全体PASSへ換算しない。

次handoffは親から既存 **ww-reviewer / gpt-6-astra / high** への独立Pre-2である。入力は本計画hash、design SHA `CC72A435CAF840E94F06F5704D95822BDAF57A19C10A91E579CE3ACB062E8313`、G0、親の限定承認、Pre-1指摘正本である。PASS後は独立作者Astra/high→実装Astra/high→runner Sol/medium→Post Astra/high（exact IDは上表）の順である。本plannerはコード/test/build/runtime/再委譲/公開を行わない。

## 設計反映・具体実行境界

現行[設計](early-trial-gate-design.md) SHA `CC72A435CAF840E94F06F5704D95822BDAF57A19C10A91E579CE3ACB062E8313` を全文確認した。Pre-1の旧design SHA `92E370…07A09C` はreview正本の履歴である。live file読取、固定field検証、source凍結/全件再照合、EXE lock、native payload、登録ACK、有限回収を現行方式とする。G-T/IはM・危険高、モデルと独立性は維持する。

Rは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`。親はR/runs/<新UUID>/のRunとg0/full承認だけを作り、wrapperはManifest/Request/Resultを作る。作者の固定期待値・helperは共有編集しない。検証入口は次で固定する（実行時<new-id>だけを新規値へ置換）。

```powershell
& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/Test-CargoGate.ps1 -GateScript F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/Invoke-CargoGate.ps1 -OutputRoot F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/<new-id>
```

標準静的検証は固定pwshで両ps1の`Parser::ParseFile` error0、Rをcwdとした`Import-Module PSScriptAnalyzer -RequiredVersion 1.25.0; @('./Invoke-CargoGate.ps1','./Test-CargoGate.ps1') | ForEach-Object { Invoke-ScriptAnalyzer -Path $_ }`である。全指摘を記録し修正又は根拠付きreview判定を残す。Pester 5.7.1は実在確認済みだが新framework導入は必須にしない。

最小対照は設計6群のENV/PATH、GATE、RACE、TRANSPARENT、LIFETIME、ORDERである。拒否時起動marker0、phase承認/JSON/旧run/未知EXE/構成/source異常、EXElock中の正常実行と差替え拒否、承認後source変更、同EXE/argvの別invocation再呼出、空文字/Unicode/U+3000/quote/末尾backslash/`--`/`-x`のexact argv、stdout/stderr bytesとexit0/7/101/非zero32bit、timeout/cancel/監督喪失/子孫回収/無関係process保全、G0失敗/aggregate期限切れ時full0、複数artifactの実Cargo接合を省かない。作者がこの範囲を実名caseへ落とし、実装不在のcompile/起動失敗を行動REDと混同しない。

製品復帰は下記5commandを固定Cargo実pathで順に実行し、構成別EnvMap・targetと新runを使う。runner環境変数値とPATH変更は設計のCargo子process範囲だけである。

```text
cargo test --all-targets --locked --message-format=json
cargo test --all-targets --locked --features early-trial --message-format=json
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --locked --message-format=json
cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked --message-format=json
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked --message-format=json
```

親が同run source/EXE/固定順序へg0承認を発行した後、Supervisor直系SafetyWorkerがlistと録音3→通信等P01既定20→同構成integration A11を既存V2Caseで先行する。各起動前のsource170/G0 12/marker4とEXEの再照合はSupervisor/worker、結果の独立照合とfull承認は親責任である。relatedのG0適用有無はそのEXEの静的根拠で親が明示する。Run3600秒/build1800秒/各承認待ち600秒/full各EXE600秒/単一回収予算10秒、残り全体時間優先である。G0はg0 publishedUtcから親full受領まで同じ600秒であり、全case上限を消費できる保証ではない。自動延長や省略はしない。

argv/32bit exit保持、source/EXE実行前照合、同run metadata、未知拒否、期限/所有killwait、親承認、安全対照先行、必要な合成失敗対照はnice-to-haveではない。削減対象はこれらを実現する重複機構と余分な呼出制約である。設計のfake6群はその観点で維持し、作者は表の条件をデータ駆動caseへまとめ、各検証のために別監督を作らない。

nativeについて親のPrimaryMonitorScalePercent=100 / HRESULT=0を受領した。親source/SHA照合後のrfd拒否dialogだけが既存runner Sol/mediumにより独立確認中であり、結果未受領である。正常起動・保存再起動・音読導線等の残部は未了であり、本wrapperのPre/Postで免除しない。

## Pre-1 P1二件の是正・Pre-2照合点

Pre-1の入口/回収P1二件は[review正本](early-trial-delivery-review.md)に保持する。以下の具体化を現行設計で確認した。独立Pre-2判定が出るまで設計上Closedとしない。

### P1-入口：native受信→固定PowerShell parameterへ変更

- Cargo runner値は `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/cargo-gate-entry.exe` の一語。pwsh探索用PATH追加なし。nativeは標準libraryのargs_os/encode_wideからEXE＋全argvをCreateNewのUTF-16整数配列JSONへ保存し、固定pwshへ名前付きpathだけを渡す。型/範囲/旧run拒否、孤立surrogate/空文字保持、OS stream継承、32bit exitを実対照で確認する。
- PS入口は設計の`[CmdletBinding(PositionalBinding=$false, DefaultParameterSetName='Control')] param(...)`を採用する。ControlはRunOwner/Supervisor、ChildはRunner/SafetyWorker、全modeにRunPath、ChildだけInvocationPath必須である。mode/setの不一致・余分な位置引数は副作用前拒否。任意`--`/`-x`/CommonParameter名はbinderを通らない。
- 開始は固定pwshで `-File <R/Invoke-CargoGate.ps1> -Mode RunOwner -RunPath <run.json>`。RunOwnerがSupervisorを起動し、Cargo→native→RunnerとG0のSafetyWorkerを監督する。nativeは固定pwshへ `-Mode Runner -RunPath <run.json> -InvocationPath <payload.json>`を渡すだけで、承認・hash・G0判断を複製しない。

作者2fileとimplementer2fileを分離し、nativeとfakeを共有しない。RunOwnerは同ps1内modeであり5file目を追加しない。親から4fileは利用者の一時wrapper承認範囲内と確認済みである。repo source/Cargo/依存・品質gateは変更しない。

### P1-回収：G0実行も同Supervisorの直接所有へ接続

- 所有はRunOwner→Supervisor→SafetyWorker→既存V2Caseである。親はg0/full承認・独立結果照合を担当し別treeでV2Caseを起動しない。各開始はIntent→Registration/Ready→StartAckを経る。ACK前はSupervisorのCargo、workerのV2Case、Runnerの対象EXE起動0。ACKはg0/full承認の代替ではない。
- 全modeは250ms以内の非blocking監督でPID/開始時刻、cancel、絶対期限を確認する。G0はg0公開から親full受領まで同じ600秒、V2Case65秒/内部helper45秒は不変。G0終了後や監督交代で時計を再開しない。UTC逆行でもStopwatch上限は延びない。
- 通常case回収はSafetyWorker、fullはRunner、run abortはSupervisor、Supervisor異常はRunOwnerが担当する。ACK後のStart→登録窓は手元handleとintent/親の生存区間/commandで照合する。10秒の単一cleanupDeadlineを共有し、葉からの単体Kill・有限Wait・残存再照合を行う。Kill(true)だけや無期限Waitを完了証拠にしない。所有不明/回収未了はCleanupIncomplete＋非zero＋fixture保持であり次case/fullを禁止する。
- 保証条件は少なくとも一監督が生存しOS照合/停止APIが応答すること。複数監督同時強制終了・OS停止・観測前の親子鎖喪失で無条件回収を主張しない。AI応答停止は承認欠落/600秒で扱い、常駐heartbeatを追加しない。必要な単独喪失/ACK対照が成立しなければ不合格であり、実装者はJob等へ独断変更しない。

追加対照はnative→pwsh→fakeの全argv/CommonParameter名/空文字/非BMP/孤立surrogate、exit0/7/101/0x80000001/0xFFFFFFFF、G0中期限/cancel、RunOwner/Supervisor/SafetyWorker/native各単独喪失、ACK前後/Start後登録前の競合である。回収可能な異常系はfull0/残存0/無関係sentinel不変、意図した回収不能対照はCleanupIncompleteを検出する。モデル割当変更なし。独立Pre-2で方式、独立runner/Postで実証を判定する。

## 計画作業の測定境界

開始共有週枠は親受領値65% / window10080分 / reset1792496607、観測13:18:04 UTC。子 `/root/et3_gate_plan` のtoken開始/終了counter、固有session ID、provider返却model/effort、親子包含は未取得。指定Astra/highと実行確認を混同しない。終了値は未取得であり差分を推定しない。計画の検証は対象文書のlink/所有/差分検査までであり、実装品質の成功や消費改善を主張しない。

## ARGV-PATH01 PacketBrief（2026-10-15、focused context再構成中）

現在はfocused harnessのcontext前提不整合が2回続いたため、3回目の局所patch/retryを禁止し、末尾「CXT01再構成」を現行の再開条件とする。native2定数修正は保全済み、CXT01は未Closed、AP1–4 GREEN・局所Post・標準80件は未了である。以下の初期snapshotと工程表は履歴として保持し、現在の所有とdispatch制限は末尾を優先する。全argv exact、承認、所有、期限、未知非停止/非PASS、80case oracleは既存基準を維持する。完了は独立focused実測・Postと標準80件への再開handoffまでであり、製品full/試用EXE完了ではない。

### Request snapshot・根拠・修正側の選択

親依頼時HEADは `ac92e29587da3fbffc739e95f17656a7649b6e91`。既存dirtyはreview/runner/usage/current文書でありplannerは触れない。次の現物hashをread-onlyで照合した。

| 入力 | SHA256 |
| --- | --- |
| R/Invoke-CargoGate.ps1 | `12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621` |
| R/Test-CargoGate.ps1 | `D8AD0AD44C2457DC60E64E52A6EDB455974D5F8AD9938B956CEF3E4DDAB0F7F3` |
| R/cargo_gate_entry.rs | `3AEB0B19E2D03EE088EB1BEDA4DBACEEC0406E12ED75D59AB95A7E331F42FD41` |
| R/cargo-gate-entry.exe | `1C04288220C3D98B182F3011FE3D1DE8AE88B050871DF400B21ED8AECCCC5E94` |
| R/fake.rs | `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016` |
| early-trial-gate-design.md | `CE799395A14F1784C05EDDB0BC2D8EE8B53B900EC306B403DB72E225CF5E5756` |

Rは本書既定cargo-gate rootである。[review正本](early-trial-delivery-review.md)冒頭の親受領観測では、Runner内部11引数のindex4/File、8/RunPath、10/InvocationPathだけIntent U+005CとOS U+002Fが異なり、他8引数は一致する。長さ90/120/173、相違8/7/7は当該runの観測値であり、新UUIDを含む後続runの固定期待値ではない。旧source3版の関連4関数同一性とPTH-MONO01のclosureは独立reviewを再利用し、plannerは再実行しない。

現物native14–17のROOT/SCRIPTはforward slash、115/130/167でRun/Invocation pathへ派生し、176–189で内部引数として起動する。Invoke1655–1657は別途内部Intentを生成し、643–647はOS argvとの全code-unit exact比較である。最小候補はnative側ROOT/SCRIPTとそこから派生する内部path表現を既存Intentに揃えることである。Supervisor側にnativeの表現規則を複製する案や比較時のgeneric normalizeは採らない。designerがWindowsのPathBuf派生・実OS引数と既存Intentの同一性を限定確認して方式を確定する。plannerはコード詳細を実装しない。

### 状態・所有者と消費側・型・エラー契約

- nativeは固定内部pathのproducer、SupervisorはIntentのproducer、Runnerの既存validatorはconsumerである。内部3pathは既存絶対pathから構成したOsStr/PathBuf等のOS文字列として渡す。新schema、公開API、source seam、native child modeを足さない。
- Cargo由来の利用者argvは既存args_os→UTF-16整数配列→payload→Runner→targetの別データであり、空文字/Unicode/孤立surrogate/quotes/backslash/順序/個数を含め変換しない。内部path限定修正をpayloadのslash置換へ広げない。PWSH起動実体、他8内部引数、stdout/stderr/exitの扱いは不変である。
- 状態はPrepared→内部path構成→payload非上書き公開→native起動→Request/Intent/Readyの既存検証→既存承認後target開始を維持する。内部path exact不一致は従来どおり拒否し、認可や所有を自動補正しない。native起動/公開失敗は非zero、terminal欠落はunknown、回収証拠欠測は非PASSのままである。
- 本体所有はR/cargo_gate_entry.rsだけを予定する。Invoke/Test/fakeは不変を基本とし、追加所有が必要なら理由付きでplannerへ返す。native source変更後は新EXEをbuildしてsource/EXE SHAを固定し、その組でRun snapshotを新規作成する。旧Run/承認/hashを上書きして再利用しない。

### 7役・依存順・検証チェックリスト

| 役 | exact model / effort | 所有と選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 本書の当該PacketBrief/作業リストだけ。固定役。設計受領後に限定確定する |
| 外部仕様 | `gpt-6-astra` / `high` | 新規作成は非該当。既存G4/G5、設計argv契約、80caseを再利用する。外部挙動・品質基準の変更が必要ならこの役へ戻す |
| designer | `gpt-6-astra` / `high` | 既存early-trial-gate-design.mdの当該限定追補だけ。内部pathとuser payload境界、Windows表現の難度/リスク高 |
| test author | `gpt-6-astra` / `high` | R/verification/下のARGV-PATH01専用回帰だけ。既存framework/helperを利用し本体編集なし。誤境界PASS・期待値追随のリスク高 |
| implementer | `gpt-6-astra` / `high` | R/cargo_gate_entry.rs限定。本体と作者oracleを分離し、argv/起動前所有照合に関わるリスク高 |
| test runner | `gpt-6.1-sol` / `medium` | 新規隔離build/log/報告のみ。固定sourceからnativeをbuildし新hash実測、独立実行。手順は限定的だがOS実観測が必要 |
| reviewer | `gpt-6-astra` / `high` | read-only Pre/Post。親が報告を保存する。全argv exactと独立性の維持を検査するためリスク高 |

上記は現在の呼出可能一覧で対応する指定値である。実行metadata未取得時は未取得と記録し、黙ってモデル/effortを変更しない。通常1 helperずつ親がdispatchし再委譲しない。親モデル・品質gateは変更しない。

- [ ] A 設計/独立Pre：内部3pathだけを揃える限定設計と所有を確定する。plannerが受領して本計画を確定する。既存外部仕様を再利用できなくなれば停止する。
- [ ] B 独立RED（A依存）：旧固定source/EXEの実native→pwsh経路で内部11引数のOS観測を得て、3path不一致を検出する。既存失敗証拠の再利用可否は作者/reviewerが対象とoracleを明記する。compile失敗/入口未到達を行動REDへ数えない。
- [ ] C 本体是正（B依存）：native内部producerだけを修正し、source diffと静的Rust検査を保存する。source/EXEを新固定し、本文比較だけでGREENにしない。
- [ ] D 独立GREEN（C依存）：同じ独立oracleで3pathを含む11引数すべてをOS実観測とIntentのUTF-16 exactで比較する。user payloadの空文字/日本語/非BMP/孤立surrogate/quotes/末尾backslash等の既存TRANS対照とraw stream/32bit exitを新nativeに結び付ける。意図的な内部argv不一致が既存validatorで拒否される負対照も保持する。ENV01の対象境界通過と新たな別原因を区別し、後者を成功にしない。
- [ ] E 局所Post（D依存）：固定hash、RED→GREEN、Invoke/Test/fake/80case不変、内部3引数以外不変を独立照合する。局所Post PASS後に既定標準80件runnerへ返す。fresh80完了・製品full・提供Postは後続gateのままである。

必要検証はnativeの既存build方式とRust format、作者回帰、既存TRANS/ENVの実観測、差分/hash/所有検査である。具体コマンドはdesigner/作者が現行入口に合わせ固定する。公開テストfilterや品質基準を追加変更せず、新framework、Job、権限、依存、global/OS設定、実API/実データは対象外である。製品releaseの成功済みbuildを本単位の比較目的で反復しない。

### リスク・rollback・停止・次handoff

主リスクは内部path修正の利用者argvへの波及、Windows文字列表現の残差、新sourceと旧EXE/Runの混在、別validator拒否をGREENとする誤判定である。独立code-unit観測とsnapshot固定で検査する。rollbackは当該native差分のみを戻し、保存した旧source/EXE hash組を照合する手順を親が担当する。旧snapshotは原因が残る停止状態へ戻るだけであり実行解除しない。旧ログ/Run/承認・既存編集を削除しない。

追加source所有、既存比較/外部契約の緩和、新権限、モデル/skill利用不能、native build失敗、実OS観測不能、回収未証明、別原因で受入不成立なら停止して親へ返す。現段階の次handoffはdesigner `gpt-6-astra/high` の限定設計である。本書を採用したのは既存計画/作業リストの正本が本書一本であり、設計詳細の先取り・重複Packet文書を避けるためである。未完了の既存plan/todoと過去記録は保持した。親はcurrent/review/usageを所有し、当該子のusage/実metadata未取得分を推定しない。

### CXT01再構成：context契約を確定するまで作者・実装・runtime dispatch禁止

第一停止はfocused `113c564c-05eb-487c-a200-d090169238e5` のrepository未初期化であり、RunOwner/native起動前のfixture errorである。第二停止は `5babecf8-0678-4bd2-8bab-ae73c540c952` / run `ab0c29bc-b969-4bea-82ca-1eddf01ab54a`。identity control2 PASS後、RunOwner terminalはfixtureRoot境界違反によるCleanupIncomplete、sourceMatched=false、processes/fixtureResiduals/cleanupDeadlineUtc=null、AP1 evidenceなしである。二つをargv行動RED/GREENへ数えない。第二停止のnative/target marker0は親受領観測であり、null配列から残存0を導かない。runner最終監査は別途受領する。

今回のread-only確認snapshotはnative source `381F5E1DCD70ED8040153FB155C8AABA0881FBFB57A8074B286FB4169D8683DD`、新EXE `FF7C9A0F641F66F7C340AEC94C21EA9A0C745EA5A39C35B252C3A9CBDAFCCB88`、Invoke `12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621`、Test `D8AD0AD44C2457DC60E64E52A6EDB455974D5F8AD9938B956CEF3E4DDAB0F7F3`、focused slash driver `89C6E49B7D2930FE225DA0AB7C4E36B94B55CFBC522CEE35454F0BE17A2350FB`、HANDOFF `FB1F80EB66BD82858821E449605FBB44B76641BCF9D2367158E55DDFEC153695`。受領design `2123BB9E88E616BF8455E7C6A63931D59DFC8CB1C3FBCAB4F134E09FCE9CCAB7`、旧plan `2A463D32…8058CD44`は改訂前を示す。保存RED・native2定数修正・PTH-MONO01 Closedは撤回しない。

#### 原典producer/consumerの再構成と唯一の契約

| 項目 | 初期化・producer → consumer、必要条件 |
| --- | --- |
| source-bound scope | driver54–63/152–156はTestのtop-level try前prefixとmain中boundary/repository代入を抽出する。Test2097/2125はRとrepoを確定し、New-CaseFixture709/738のPSScriptRootは実TestのR、GateScript737/1730は固定Invokeを要求する。関数のFile一致は初期化・precondition充足の証明ではない |
| main前提 | Test2096–2134/2153–2199の絶対固定R、GateScript兄弟実体、OutputRootの絶対/不存在/reparse無し/ignore、Admin=False、pwsh7.6.6、80件uniqueとcaseRows digest、固定native/V2存在・hash・native非稼働、Pester/rustc、controller実identityを列挙する。driverがmainを省く場合も、実行するconsumerに必要な前提を欠落させない。main全体を無差別実行する解法は採らない |
| 暗黙入力 | repositoryはGet-ConfigSnapshot796–800のuser/repo各config/config.toml計4path、USERPROFILEは既存読取、GateScriptはfixtureと起動、fakeContractはConfirm-CaseResult1997–1998のstream oracle、cases/caseRowsは選択集合/80件digest、PSScriptRootはRun/native source位置に使う。caseResults/passed等の標準dispatch専用状態は実行経路外として区別する。config欠測hash=nullを内容無しと推測しない |
| suite/fixture階層 | New-CaseFixture655–657/744はsuiteId=basename(SuiteRoot)、fixtureRoot=SuiteRoot/<case[-variant]>を生成。Invoke789/1011はfixtureRootがR/verification/<suiteId>の厳密な子孫であることを要求する。標準main2103は単にverification配下を許すだけで、後段を含む十分条件ではない。focused driver47のpacketRoot直下要求はこの合成契約と不整合である |
| 正常なfocused配置 | 実行suiteだけを新規R/verification/<suite-UUID>直下に置き、descriptor.suiteIdをそのbasenameに一致させる既存正常形を採る。fixtureはその下の各case/variant。suite名のUUIDは本focused実行の選択であり、既存gateのsuiteId正規表現をUUID限定へ変更しない。R/verification/argv-path01はdriver/oracle/HANDOFF/解析証拠の保存場所であってfixture suiteの親にしない。解析専用oracle OutputRootと実行SuiteRootのconsumerを混同しない |
| Runと拒否sidecar | Test709/1722–1728はR/runs/<run-UUID>/run.jsonと8field rejection-contextを新規公開する。Invoke740–796は正規D形式runId/厳密なRun位置、Run bytes SHA、時刻/期限、実OS launcher親同世代、controller一致、descriptor SHA/root、TEMP/TMP二つを検証し、815で初めてRejectionContext確定。Runのcfg/id/時刻/envMap/fixtureControlとの一致後だけ996でRunへ昇格する |
| marker/fixtureデータ | fixtureRoot直下fixture-owner.txtはUTF-8 no BOMで厳密な既定文言+LF。descriptor/marker immutable、repoはfixtureRoot又は子孫、LOCALAPPDATA/TEMP/TMP/target、source/g0/marker snapshot現物、fake cargo/artifacts/control参照は既定fixture境界内。TEMP/TMPは同じtempでも2fieldを持つ。固定gate/native/pwsh/V2およびRun証拠rootの既存例外を勝手に拡大しない |
| payload/子scope | nativeはR/runs/<run-UUID>だけからinvocations/<新UUID>/payload.jsonを非上書き公開する。Invoke1937–1945はpayload名/UUID/正確なnestingを再検証する。V2は既定R/runs/<runId>/safety/<caseId>/v2-outputでありfixture側へ移さない。suiteの移動でRun/payloadをpacket配下へ移動しない |
| cleanup/観測 | Invoke821–823はcontext未確定ならCleanupIncompleteを維持する。確定後も850–868の実子列挙/実TEMP列挙だけがprocesses=[]/fixtureResiduals=[]の根拠となる。native未到達、control2 PASS、wrapper終了だけでcleanupを既知0にしない。unknown非停止、既定deadline/mono、fixture保持を維持する |

第二失敗のdescriptorはsuiteId=5babecf8-0678-4bd2-8bab-ae73c540c952だがfixtureRootにargv-path01階層が挟まる。Invoke789の期待prefixと一致せず、815のcontext確定に到達しないことを保存descriptor/sidecar/terminalと現物で確認した。suiteIdをargv-path01へ偽装する、boundary比較を緩める、descriptorを消費後に補修する是正は採らない。

designerは既存designのARGV-PATH01節にfocused context契約を追補し、そこを唯一の手順正本とする。特に現在designの「回帰所有folder」と「実行suite配置」を明確化し、HANDOFFのnested実行commandを失効させる。planは依存/所有/受入だけを管理し、別context仕様書・新frameworkを作らない。技術前提の欠落はこの設計工程で解消し、品質基準変更を必要としない限り利用者へ選択を丸投げしない。

#### 新cycleの所有・有限証拠・停止条件

| 役・割当 | 今回の所有と境界 |
| --- | --- |
| planner `gpt-6-astra/high` | 本PacketBriefだけ。現再構成→design受領→限定確定。過去作業リストは未完了のまま保持 |
| 外部仕様 `gpt-6-astra/high` | 新規作成は非該当。G1–G6/AP1–4/既存80を維持し、外部要求の変更が判明した場合だけ戻す |
| designer `gpt-6-astra/high` | early-trial-gate-design.mdのfocused context追補だけ。初期化から全consumer/出力/cleanupまでの前提漏れが高リスクのため限定Pre必須 |
| test author `gpt-6-astra/high` | R/verification/argv-path01/内の新context回帰とその期待値だけ。既存独立argv oracle/10case/17引数slash期待値を保持し、driver本体を同時編集しない |
| implementer `gpt-6-astra/high` | 設計/Pre/作者RED後にfocused slash driverとHANDOFFの実行context部分だけを逐次引継ぎ。今回はharness本体所有であり、native2定数/Invoke/Test/fakeの編集は非該当・凍結。作者の新oracleは変更しない |
| reviewer `gpt-6-astra/high` | read-onlyでcontext Pre、固定harness差分/有限preflight証拠、focused実行後Post。初期化漏れ・誤PASSの独立確認を担う |
| runner `gpt-6.1-sol/medium` | 新規合成証拠/logのみ。許可されたpreflightと既定focusedを段階実行しソース/期待値を変更しない。native source/EXE不変なら再buildを反復しない |

上記は全て利用可能なexact model/対応effortであり、作者・harness実装・reviewerはcontextと所有境界の高リスク判断、runnerは固定手順の独立証拠採取を担当する。親はcurrent/review/runner/usageを所有し、返却metadata未取得分を推定しない。

- [ ] CXT-A：runner最終監査を受領し、上記全consumer mapをdesignerが唯一の契約へ具体化する。独立Preで完全性を確認してplannerへ返す。ここまで作者/実装/runtime dispatch不可。
- [ ] CXT-B（A後）：作者が有限synthetic preflightの固定matrixを作る。旧nested配置とrepository欠落の二失敗をそれぞれ正しい境界で検出し、正常な直下suiteを同じ元fixture/path/schema/marker consumerで受理する。外側guardだけの複製やcallback到達だけを合格条件にしない。固定少数case・一度の実行・上限時間・書込root・呼出禁止境界をdesignerがPre前に明記する。
- [ ] CXT-C（B後）：harness実装者がcontext契約全体に従って是正する。第三の一行patchを先に実行する順序は禁止。旧RED/driver/HANDOFF/失敗rootを保持し、正常/異常のpath、hash、marker、sidecar field/時刻/launcher、Runとfixture整合を対象とする。
- [ ] CXT-D（C後）：独立preflightはnative/target起動前で止める。可能な実path/file consumerを使い、OS親identity等を代用した試験はmock/context検査と明記して実RunOwner/所有回収成功へ換算しない。無効入力はRunOwner/native起動を呼ぶ前に拒否することを別counterで確認する。正常入力もpreflightだけではruntime許可/terminal実測成功にしない。invalid/欠損marker・descriptor/run不一致・nested/別suite/別run/別payload・reparse・期限不正等の代表的consumer分類を固定し、無制限の追加試験やretryをしない。
- [ ] CXT-E（D後）：固定harnessの独立接合reviewと親受領後だけ、新しい合法suiteRootで既定focused10件を一回実行する。正常contextの実OS親照合、実terminal/cleanup、AP1–4はこの段階で判定する。最初の失敗で残りNotRun、sourceMatchedとcleanup観測を別記し、前提不整合なら再び停止して根拠を返す。focused Post後だけ標準80件へ戻る。

撤回/限定するclaimは、(1) source-bound File一致で全context同定済み、(2) boundary/repository2代入とconfig4path確認でruntime前提が網羅済み、(3) ValidateOnly/静的Pre/Postからfocused実行可能、(4) CXT01 Closed、(5) marker0からprocess/TEMP残存0、である。これらの静的検査自体・control2 PASS・保存旧argv REDを消さず、成立した狭い範囲へ限定する。native2定数を原因に戻したり、実行未到達の新nativeをGREEN/FAILと判定しない。

上記再構成/設計/Preが済むまで新patch/再試行は禁止する。80case/87variant/10focused/追加slashの期待値、generic argv exact、OS/権限/依存/Job/API/データ、製品source、既存承認/期限を変えない。必要な新権限または既存consumer境界緩和が判明した場合だけ具体的根拠を親へ返し、利用者判断を求める。現段階は技術契約の再構成で進められ、利用者の追加選択は不要である。

## KNOWN-HANDLE01 PacketBrief（2026-10-16、KH-A設計/Pre受領・限定確定）

対応findingは親の既存review matrixの`KH-REOPEN-01`（Accepted、独立P1確認）である。KNOWN-HANDLE01は本局所計画名、KH-A–Dは作業単位であり、findingの複製やClosed判定を作らない。

対象はCXT-Eで観測した、既登録processを再列挙した際に保持handleへ到達する前のPID再openが失敗する局所競合である。完了境界は最小是正の独立回帰・固定snapshotのfocused10件・独立Postを経て標準80件への再開handoffを可能にすること。標準80/87variant、製品full、ET3提供の完了は含めない。本plannerの今回の完了はKH-A受領による本節の限定精緻化と文書検査・親返却だけである。

受領設計は[既存designのKNOWN-HANDLE01節（477行以降）](early-trial-gate-design.md)、SHA256 `AF67CB6E40B37813226317B68D0698BBD84586732FAE92D88A7982D978E223A2`。親からfresh独立Pre PASS／確定指摘0／closure blocker0を受領した。設計は方式・型・状態・固定22回帰・commandの唯一の正本であり、本節に再定義しない。Preの入力plan956Cは精緻化前の履歴として保持し、設計内の入力hashを新planへ書き換える循環更新は不要である。現段階は設計成立性の確認までで、実装/実OS回帰/Closedではない。

### 証拠と未検証の境界

- 親受領の独立診断P1は、Invoke `12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621` のRead-OwnedTreeで1270のGetProcessById→1272のGet-Identity→1274のKnown.ContainsKeyの順であることを確定した。plannerも当該分岐とAdd-KnownProcessのpinnedHandle保持をread-only確認した。追加OS測定は設計着手条件ではない。
- 保存REDはsuite `R/verification/68cfe879-a4ea-4665-83f0-8bff89cd2b2b`、run `94e5f1c6-43bb-4ca6-98c7-80eb443f76fa`。control2PASS、ENV01FAIL/残9NotRun、sourceMatched=true。RunOwner21556のUnprovable PID23100は再open時not running、rowFilter未到達。同terminalに保持済Owned PID23100（parent14632、start `2026-10-15T20:38:03.9529175Z`、exit `20:38:04.9864708Z`）があり、元CIM creationは`20:38:03.9529170Z`。knownMatch1/snapshotMatch1/path・command存在は補助観測であり、これだけから同世代/所有を断定しない。
- 最初の公開cancelはSupervisor `20:38:05.0399582Z`。RunOwner cancel.tmp `20:38:05.0425184Z` は未公開である。SafetyWorker result.tmpにもPID6472/knownMatch1の同分岐失敗があるが、正式terminal成功へ加算しない。PID23100が全process時系列で最初の原因だったというclaimは置かない。
- CXT-D test94E4928C/driver68EF44DDの12PASS・独立Post PASSとPTH-MONO01 Closedは保持する。今回はPTHのpath-null fallback/UTC精度/mono検証より前の別branchであり、そのclosureを撤回しない。旧Unprovableを後からOwnedへ変更せず、保存RED、tmp、cancel、旧入力/結果/承認は上書きしない。現行CXT-E/ARGV受入は未達のままである。

### 要求・確定方式・不成立条件

| 要求 | 受入境界 |
| --- | --- |
| KH1 同世代の既知process | 元snapshotの対象行と台帳identity・有効保持handleが同じ世代であることを再open前に証明できる場合のみ、その既存台帳を利用する。PIDだけのskip、GetProcessById例外の無条件成功化、最新snapshotへの取り直しによる救済は不可 |
| KH2 所有/終了の証拠 | 既存Knownの所有根拠とhandle寿命を保持し、新しい所有権・終了事実を合成しない。既知・同世代でsnapshot後に終了しても、保持handleで既存終了観測へ進める。未知・証明不能は従来のUnprovable/非停止/非PASS、既存fault・kill/wait/terminal条件を維持する |
| KH3 境界の不変 | 新規候補は従来のidentity/parent lifetime検証と登録へ進む。PID再利用、複数/不正CIM行、部分identity、閉鎖/無効handleを安全な既知一致と誤認しない。generic argv exact、承認、UTC/mono期限、未知値nullとfixture保持は不変 |

確定方式はRead-OwnedTree内で親PIDによる新規候補除外より先に既知候補を照合し、元CIM行一意・UTC6桁候補一意・有効保持handleの同一参照を確認して、未変更Get-Identityの5fieldとUTC7桁keyが完全一致した場合だけcontinueするものである。新private helperを追加せず、本体所有はRead-OwnedTree内だけへ縮小する。終了済保持Processの既存consumer成立性は設計の.NET10.0.12 source根拠と独立Preで確認した。実binaryの成立はKH-O2の未実施受入に残す。

部分DirectLaunchIdentity、同UTC6桁の複数候補、不正/無効handle、全identity/key不一致は既知照合branchで拒否し、新規openへfallbackしない。別UTC6桁の既存世代は保持し、同世代候補0の新規世代だけを元の親寿命・PID open・登録経路へ渡す。新規経路のkeyが既存Knownへ衝突した場合も未照合continueを許さない。詳細な選択順・型・catch/冒頭例外の区分・借用handle非Disposeは受領設計KH1–3に従う。

暫定代替だったprivate identity照合/生成は不採用とする。Get-Identity/Add-KnownProcess/Read-DirectChildIdentity/Get-TerminationObservation/Complete-OwnedTreeと他consumerを変更しない。一般化した新identity framework、全監督の再設計、GetProcessById retry、CIM追加照会、Job/Admin/OS設定による回避は採らない。実OS受入が不成立でも未知を成功にする方向へ縮退せず、新findingとして親へ返す。

### 作業単位・所有・依存・検証

作業リストの正本は本節である。既存未完了plan/todoは移動・完了扱いにしない。通常1 helperずつ、作者/実装/runner/reviewerは別担当、共有fileは逐次所有とする。

| 単位 | 依存・規模・所有 | 受入と必要検証 |
| --- | --- | --- |
| KH-A 設計/Pre | 本計画の親受領と独立P1。S、designerは既存early-trial-gate-design.mdの局所追補だけ | AF67設計・独立Pre PASSを受領し、plannerが外部契約再利用・Read-OwnedTree限定所有・固定Pure20/OS2と順序を確定。KH-A完了でありKH-B以降未実施 |
| KH-B 独立回帰→局所実装 | Aと本精緻化の親受領後。S、作者はR/verification/argv-path01/Test-KnownHandle.ps1だけ、別実装者はR/Invoke-CargoGate.ps1のRead-OwnedTree内だけ | 作者が固定Pure20/旧RED→別実装→作者の新Pure20/OS2を設計commandで確認する。実装者は作者期待値を編集しない。Parser::ParseFile/PSSA1.25.0診断0・実行error0、逆diffで非所有関数不変を確認。固定対照不成立なら下流解除不可 |
| KH-C 束縛更新・接合Pre | A/B確定hash後。S、作者は既存Test-ContextPreflight.ps1の入力束縛と同既存handoffの新snapshot説明だけ、その後別実装者はInvoke-ArgvPathFocusedSlash.ps1の固定入力とHANDOFF.mdだけ | 下記依存鎖の旧/new SHA対応、旧bytes保持、12期待/禁止22/10case/17args/firstFail/期限の意味的不変を独立review。入力hash不一致を無視する実行は不可。元context consumerに意味的変更がないことを確認し、単なるplan/hash変更を理由に旧12件を全再実行しない |
| KH-D 独立実測/Post | B/Cの静的接合PASSと親dispatch。S、runnerは新規隔離証拠/log、reviewer read-only、親が既存runner/review/current/usageへ記録 | 最小正常scenarioを早期確認する設計順序OS2→Pure20→既定focused10件を各1回、firstUnexpected/firstFail・残NotRun・retry0。新snapshotの正常ENV01、実terminal/cleanup、AP1–4を独立確認しPost PASS後だけ標準80へ返す。別失敗は別findingとして停止し、本局所是正を理由にfullを解除しない |

- [x] KH-A：AF67設計・fresh独立Pre PASSを受領し、既存外部仕様再利用と唯一方式・実装/試験境界を限定確定した。
- [ ] KH-B：独立作者が期待/REDを固定し、別実装者の局所修正とGREENを照合する。
- [ ] KH-C：新hash組への最小再束縛・旧snapshot保全・独立接合reviewを完了する。
- [ ] KH-D：独立OS対照/既定focused/Postを完了し、未完了標準80へ返す。

局所回帰は受領設計のKH-P01–20/KH-O1–2へ固定する。型代用とcounter、AST置換箇所/逆置換一致、旧Diagnostics.Process型検査0件・新1件、firstUnexpected、元consumer/失敗stateのoracleは同設計を参照する。Pure各5秒/全60秒、OS2全体finally込み15秒、retry0であり、既存gate期限の変更ではない。保存旧Invoke12E7の決定的P02 REDを保持し、AST/adapter不適合や旧OS O1失敗/O2未実施を競合REDへ数えない。Pureの型代用を実handleの証拠へ加算しない。

OS-boundは受領設計の同一合成子O1→O2、実CIM1回からself/子の元field不変抽出、実保持handleと未変更consumerで確認する。O1失敗でO2を止め、writer/runner自身の回収をgateのkill/wait/terminal成功に数えない。作者の新OS2と独立runnerの新OS2は独立証拠として各1回である。具体command/必須SourcePath・ExpectedSourceSha256・Phase・新UUID rootは同設計で確定済み、未知sentinel停止・PID再利用待ち・再試行・追加権限はない。

新Invokeが変更するのは所有列挙の既知分岐であり、CXTの禁止22中Read-OwnedTreeは実行対象外である。12件は旧固定snapshotでの受入証拠として再利用できる。新hashでの静的抽出・4typedmocks/consumer本文不変・束縛の接合を確認し、影響するconsumerに変更が波及した場合だけその受入を追加実行する。必要な既存LIFE14/16/17/18等の安全検証は後続標準80のgateに残し、局所対照で免除しない。製品source/native/fake/EXE不変ならbuild反復は不要である。

### frozen bindingの再束縛範囲

追補前planは`E209293EC26AA95F0C88C0CFE795B397BEF0B9BF3EB5B94F3FED63F7289E84C4`、designは`D1C38FB3557D067A5E0B414E6DAEA021123C66B64D6FB5C62930B271CC5EFADF`、Invokeは上記12E7である。本追補だけで旧driver68/test94のplan固定値は一致しなくなる。現時点の旧driver実行は許可されず、旧成功記録は旧hash組に対して保存される。

依存鎖は「確定plan/design/Invoke → Test-ContextPreflightの該当3入力hash → 新test hashと既存CONTEXT-TEST-HANDOFF-P1.mdの新snapshot説明 → focused driverのplan/design/Invoke/test/handoff hash → HANDOFF.mdとrunner frozen入力」である。Test-CargoGate.ps1、argv oracle7D74、旧CONTEXT-TEST-HANDOFF.md、native source381F/EXE FF7C、fakeのbytes/期待は不変とする。Test-ContextPreflightの変更は作者所有、focused driver/HANDOFFの変更は別実装者所有であり、今回plannerは変更しない。各段で旧bytesを非上書き保存し、旧testの既定DriverSha256や旧REDの意味を無条件で新候補へ書き換えない。新候補の明示driver引数は最終freeze後にrunnerへ渡す。

設計・planの確定前にhashを追いかけて更新しない。planへ最終hashを追記して自身のhashを再変更する循環を避け、最終hashは親の既存review/handoffへ保存する。Run source snapshot、新suite、新承認は最終固定入力に対して新規作成し、旧Run/承認/frozen-inputsを編集しない。束縛更新はKH-Cの独立reviewを経る限定作業であり、harness判断・期待値の変更許可ではない。

### 各単位の7役割当・リスク・次handoff

下表はKH-A/B/C/Dそれぞれに適用するexact割当である。非該当工程も責務を識別し、起動しない。現在の呼出可能一覧で両モデルとhigh/mediumは対応しており、親はdispatch時にも照合する。指定とprovider返却値は別記し、実metadata不明を推測しない。

| 役 | exact model / effort | 各単位の担当・難易度/リスク・理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | A後の精緻化、B–Dの境界/差戻し判断。本plan該当節のみ。高/高、競合証拠と再束縛範囲の分離が必要 |
| external-spec author | `gpt-6-astra` / `high` | A–D新規執筆は非該当。既存G5の同世代所有/未知非停止、G1–6/AP1–4/80期待を再利用する内部是正であるため。外部判定/承認/期限の変更が必要ならAへ戻し、この固定役で仕様化する |
| designer | `gpt-6-astra` / `high` | Aの局所設計、B/Cで設計から逸脱時の差戻し担当、Dは新設計非該当。高/高、終了後handleと部分identity・PID世代の誤同定リスク |
| implementer | `gpt-6-astra` / `high` | A実装非該当、BのInvoke局所修正、Cのdriver/HANDOFF束縛のみ、D編集非該当。高/高、停止権限と証拠同一性を維持するため |
| test author | `gpt-6-astra` / `high` | Aで独立oracle条件確認、B回帰/RED、Cのcontext test束縛だけ、D期待変更不可。高/高、同PID誤PASS・競合再現の不安定さ・mockと実handle証拠の混同を防ぐため |
| test runner | `gpt-6.1-sol` / `medium` | A/B/Cのruntimeは確定指示前非該当、Dの固定局所/OS/focused実行。中/高、手順確定後の独立採取とfirstFail管理であり未解決の設計判断を委ねない |
| reviewer | `gpt-6-astra` / `high` | A設計Pre、B/C差分・RED/GREEN・束縛の独立接合、D結果Post。高/高、未知への停止/昇格・別原因GREEN・古い証拠接合を独立検査 |

designerはapi-and-interface-design、作者はtest-driven-development、実装/runnerはwordweave-changeの役割内工程、PS作成担当はpowershell-automation、reviewerは設計時review-software-design、実装時code-review-and-quality/powershell-reviewの該当範囲を用いる。未導入skill/未対応modelはblocker、silent fallbackなし。親model・effort・品質gate・再委譲/公開禁止は維持する。

高リスクは既知と同PIDの混同、借用handleのDispose、部分identityの昇格、世代精度の変更、旧REDの消去、再束縛に混ぜた期待値変更である。元snapshot/台帳/保持handleの三者照合、borrowed/ownedの明示、独立負例とdiffで抑止する。rollbackは親が当該局所差分だけを保存旧版へ戻す操作であり、旧競合/旧束縛失敗を再び許可するものではない。他者編集や旧証拠は戻さない。

暫定未決だった(1)既存Get-Identity再利用、(2)部分identity/複数Known候補、(3)回帰件数・時間・commandはAF67設計と独立Preで解消した。新たな利用者仕様判断はない。残る不確実性は未実装・未実行の受入であり、KH-B/Dの証拠で判定する。追加権限、既存比較/期限の緩和、別file所有拡大が必要になった場合だけ根拠付きで親へ停止返却する。最小正常確認が別境界で失敗した場合、無限にharness整備へ広げず保存証拠と新findingを返す。

次handoffは本精緻化を親受領後のww-test-author `gpt-6-astra/high`、所有はTest-KnownHandle.ps1とその新規隔離証拠だけである。入力は本節の返却hash、AF67設計、親受領Pre PASS、現Invoke12E7と保存RED。順序は作者のPure20/旧RED固定→別実装者Read-OwnedTree修正→作者新Pure20/OS2→KH-C再束縛と独立B/C接合→独立runner OS2/Pure20/focused10→独立Postである。各役exact割当は上表を維持し、今回は作者へ渡す境界だけを確定する。作者着手のdispatchは親が行い、実装/後続runtimeの包括解除とはしない。

今回の開始共有週枠は親受領40%、途中の親受領review終了値41%でありplanner単独の終了測定ではない。子`/root/known_handle_race_plan`の開始/終了token、固有session ID、provider返却model/effort、親子包含、終了週枠は未取得。使用差分/改善率は推定しない。本plannerは文書追補・diff/hash/相対link/所有検査のみ、実装/試験起動/再委譲/公開0である。

## FIRST-FAILURE01 PacketBrief（2026-10-16、FF-A設計/Pre受領・限定確定）

対応findingは`OBS-FIRST-01`（監督processの初因診断欠測）とする。親受領の利用者承認は「中断前のエラー専用診断を検証用プログラムへ追加し、合成正常系ENV01を1件1回」である。既存安全判定/時間上限、非管理者、OS設定/実data/API不変を維持する。今回の完了境界は限定診断の実装・局所pure/file検証・独立接合reviewと、承認されたENV01一回の結果/診断を親へ返すところまでである。ENV01が再び失敗した場合は保存証拠を返して止め、原因是正や再実行を自動追加しない。

主目的はKH-D/CXT-E/ARGVを通してET3提供へ進むことであり、本packetはそのための観測不足の是正だけである。診断が得られたこと、probeの正常終了、ENV01の1件PASSのいずれもfocused10/標準80/87variant/製品full/提供完了ではない。probeの10/80/full creditは常に0とする。FIRST-FAILURE01の観測機構受入と主KH-D focused受入を分ける。正常時にfailureが発生しなければ実sidecar公開経路のOS実証は未観測と記録し、局所file検証と混同しない。

受領設計は[既存design FIRST-FAILURE01節（574行以降）](early-trial-gate-design.md)、SHA256 `56F1DB79EEF0F12B8D669DDCF4A927E97ECFFAD5C098A0A13BA1DDEE6FE9280D`である。親からfresh独立Pre PASS／blocker0を受領した。HEAD4cb270f、Invoke35AE・元context/ENV01との接合を確認した同Preに基づき、方式・型・固定18case・commandを同設計の唯一の正本として本節を限定確定する。設計内の入力plan6B84は履歴のまま保持し、新plan hashへの循環更新はしない。実装/試験/ENV01は未実施である。

### 入力証拠・維持する成果

親受領の新suite `a59b49c6-ffca-4f78-8436-6b46b57d1d7c` / run `4de297d1-d99e-4d5e-9a9a-8f74a034bca1` はcontrol2PASS、ENV01FAIL/9NotRun、sourceMatched=true。Supervisor37636がCleanupIncompleteを公開した時刻部分は親共有`21:35:48.8079204Z`であり、正確な日時は保存cancelを正本とする。正式Supervisor terminal/tmpがなく、同観測者のKnown/Unprovable/osObservation/例外が欠測している。Invoke1356のcommoncatchと1397のrun-bound parent edgeを区別できず、RunOwner/Safetyのunknown0からSupervisorの未知候補0を推定しない。

入力Invokeは`35AE6C713ABCEAA28361DA750E7BD06F373E09EF2FB155D8456F79D83ADC66B5`、probe参照元の現focused driverは`72244CD533E6CE3DA1062D05C4A6F9EC5195FE63AAE40C2D7CC01AB7E0AEF3E0`、context testはF46F3B09、Known testは0C82D1C8である。旧plan80B2/designAF67とこれらの結果を保存する。KHの独立OS2/Pure20PASS・static PASS、CXT-D、PTH-MONO、KH-TEST-AST01-P1/P2 Closedを保持し、35AEの所有判定を推測で再修正しない。旧failureBoundaryはReady/Payload/Request等の既存terminal診断であり、新しい初因観測の保存先として流用しない。

### 要求・除外・確定契約

| ID | 観測可能な受入・不変条件 |
| --- | --- |
| FF1 初因保存 | 各actorが最初にInvoke-GateFailureへ到達した際、当該actorの元cleanupがcancel公開を試みる前に、既に確定している合成run/role/actor identity/sourceへ束縛した専用sidecarの公開を一度だけ試みる。固定callsite分類によりcommoncatchとrun-bound-parent-edgeを区別でき、元の該当Unprovable/既存Get-IdentityFailureObservationの最小fieldを結び付ける。他actorの先行cancelを排除する順序保証や、全processの最初の事象・成功terminalとは称さない |
| FF2 非干渉 | 元Status/AbortTick/AbortUtc、元例外・cancel・cleanup・期限・unknownの判定と経路を維持する。保存失敗/衝突/欠測/再入が元failureを置換・握潰し・遅延再試行せず、元cleanupへ進む。試行済み/公開済み/未観測を区別し、tmpを正式公開へ加算しない |
| FF3 安全な最小情報 | 出力先は既に検証済みの合成Run証拠rootと固定filename規則だけ。context/role/identity/source未確定、path不正/境界外/reparseでは書かず元failureを維持する。既存値の明示whitelistだけを保存し、FullRun/全Known一覧/環境変数/資格情報/生commandline/教材/実ユーザー情報/例外全文をdumpしない |
| FF4 実測の限定 | 専用probeは既存正常ENV01のfixture・安全context・case期待を保持し、実case dispatchはENV01だけ1回、成功/失敗とも次case0・retry0。元caseのtimeout/cancel/cleanup/terminal oracleを変えず、probe成功を10/80/full creditへ変換しない |

確定方式はprefixのFirstFailureState、Initialize-Run昇格完了後の合成fixture/source確定context射影、Invoke-GateFailureの一回限定hook、Read-OwnedTree二callerの末尾任意引数、New-FirstFailureRecord/Write-FirstFailureRecordの2private helperである。本体所有をこの境界だけに固定し、元main catch/finally/Initialize-Cleanup/Write-Terminal/所有判定を変更しない。元2引数Status/Messageを維持し、追加Callsite/FailureObservationは診断専用とする。callerは作成済みUnprovableの該当1件を渡し、再観測・全Known列挙をしない。

保存は設計のscalar whitelist/型/長さ制約だけであり、Message/error/例外全文は記録しない。分類はOwnedTree.IdentityCatch、OwnedTree.RunBoundParentEdge、GateFailure.Otherの固定3値とsource hashで接合する。depth5/配列0/UTF-8 no BOM最大8192 bytes、正式名first-failure-<role>-<actorPid>-<actorStartTicks>.jsonと固定.tmp、CreateNew/Flush/非上書きMove各1回に限定する。具体schemaとhelper signatureは受領設計のprivate interfaceを参照し、本planへ重複定義しない。

Invoke-GateFailureは元Status/最初のabort時計設定→一回のattempted印→診断→元Status/AbortTick/AbortUtc復元→元throwの順序とする。state/helperなし抽出scopeではhelper/write0、context未確定ではSkippedを保つ。診断writerからWrite-Json/Assert-Controlを呼ばず、例外を診断block内で処理して元failureを継続する。新clock/予算/待機/retry/workerを加えず、診断時間も元abortからの予算を消費する。同期I/Oの強制中断、canonical確認とI/O間の外部path競合封鎖は保証しない。process消失後のFailed/Skippedは永続証拠なしなら未取得であり、tmp/正式fileなしをattempted=falseへ補完しない。

診断は外部挙動の安全契約を変更しない内部観測であり、新外部仕様書は非該当である。元JSON schema/status/cancel/FailureBoundary/承認形式・期待値は不変、新sidecarは診断専用で本体判定の入力にしない。汎用logging framework、監督再設計、Job、kill権限、global/OS設定、依存package、認証/API、製品source/Cargo/native/fake修正は除外する。副作用を避けるため診断を非同期化する別processの追加も行わない。

### ENV01専用probeと再束縛

現在のfocused driverはENV01成功後に残9caseへ進むため起動しない。新probe `R/verification/argv-path01/Invoke-FirstFailureEnv01.ps1` 1本に固定する。parameterはNativeSourceSha256/NativeExeSha256/OutputRoot/ValidateOnlyだけ、nativeは既存381F/FF7Cの固定組である。元Testのprefix・boundary/repository各1個・前提18文をsource-boundで再利用し、元80unique/caseRows hashを保ったままENV01一意/variantなしを確認する。小private Invoke-FirstFailureEnv01Caseに元Invoke-GateCase一回を限定し、元ConfirmCaseResult/fixture/期限を変更しない。case selector/loop/retry/別variantは設けない。ValidateOnlyは元read-only前提/抽出/hash確認だけでfixture/build/dispatch0とする。

ENV01自身が必要とする合成G0/RunOwner/fixture起動・controller同定・registration/所有検証は元caseの一部として維持する。control2は独立regressionでありENV01 contextへ戻り値を渡さないため、受領設計/Preに従い呼出0とする。旧PASSを新環境へ換算せず、通常の安全前提は省略しない。KH OS2/Pure20、focused残9、標準80、OS障害刺激も追加起動0。ENV01一回で再発しなければ診断目的の再現試行を足さない。

新probe所有は実装者、独立oracle所有は作者で分離する。probeは新規`R/verification/<UUID>`直下suiteと元の`R/runs/<UUID>`を使い、診断資料folderと実行suiteを混同しない。既存fixture/source/marker/sidecar/payload/期限consumerは改変しない。ENV01判定は元ConfirmCaseResultで足りるためargv oracleは追加起動しない。AP1は後続focused10に残す。expectedCount1/dispatchCount0又は1/otherCase0/retry0/credit全0を固定し、case判定と診断Published/AttemptedUnpublished/Unobservedは別記する。

本plan追補と後続design更新で、旧driver722/contextF46の固定plan/design hashは不一致になる。旧frozen-inputs/Run/承認/handoff/結果は旧snapshotの証拠として保存し、hash無視で起動しない。凍結順は確定plan/design→診断Invoke→局所test/新probe入力→probeの固定hashと親の既存review/handoff記録→新suite/Runである。probeが直接消費するものだけを再束縛し、旧focused10入口を使わないためだけの無関係な束縛更新は行わない。

今回のprobe固定入力は最終plan/design/Invoke、元Test D8AD/fake3482/native source381F/EXE FF7C、probe自身、pwsh/V2/rustcと生成fakeだけである。旧focused driver/HANDOFF/context test/P1handoffはruntime前提としてloadせず、再束縛/編集しない。必要な原文対応は新testの静的oracleで確認する。probe自身の承認hashは親の既存review/handoffで固定し、現物の自動hash採番を承認代用にしない。focused10へ戻る時の再束縛は別gateのまま、plan/designの履歴hashは循環更新せず最終値を親へ返す。

### 有限作業リスト・所有・verification

| 単位 | 依存・所有・規模 | 受入と検証 |
| --- | --- | --- |
| FF-A 限定設計/Pre | S、既存designのFIRST-FAILURE01節だけ | design56F1・fresh独立Pre PASSを受領し、plannerが本節を限定確定。方式/型/whitelist/固定18case/commandの未決を解消。FF-A完了でありB–D未実施 |
| FF-B 独立oracle→診断/probe実装 | A/本精緻化の親受領後。M、作者は同packet内Test-FirstFailure.ps1と新規隔離fixture、別実装者はInvokeの確定診断境界＋新probeの2本だけ | 作者18期待/adapter先行凍結→別実装者Invoke/probe→作者旧T01 RED/17NotApplicable→新18GREEN各1回。元failure/state/期限/期待不変、source差分/非所有逆diff、3ps1のParser/PSSA1.25.0診断・実行error0を照合 |
| FF-C 凍結/独立接合 | B後。S、実装者の新probe入力束縛のみ。親が既存報告へ保存。旧context/focusedの編集なし | 確定組の静的接合/ValidateOnlyを1回、旧証拠保全、whitelist/path境界、元error保持・再入/保存失敗非干渉、ENV01限定入口を独立review。旧成功consumerは静的不変/T09互換へ接合し、無関係な再実行なし |
| FF-D 合成ENV01一回/返却 | C PASS・親の明示dispatch。S、runnerは新隔離結果/logだけ、reviewer read-only | 新probeでENV01 1件1回・retry0・元期限のまま実行、case結果とsidecar/欠測・sourceMatched・terminal/cleanupを別記。新診断保存の成否とENV01判定を独立Postで区分し親へ返す。失敗の再試行/推測patch/後続case起動は禁止 |

- [x] FF-A：design56F1・独立Pre PASSを受領し、方式/所有/有限18case/専用入口/束縛を限定確定した。
- [ ] FF-B：独立oracleを固定し、診断と1件専用probeだけを実装・局所検証する。
- [ ] FF-C：最小束縛・旧証拠保全・独立接合PASSを受領する。
- [ ] FF-D：ENV01一回の結果と診断範囲を独立確認して返し、本単位と主gateの残項目を区分する。

局所oracleは設計T01–18の18件、各5秒/全60秒の早いmonotonic上限、各1回/retry0/想定外失敗で残NotRunへ固定する。実CIM/process launch/Kill/Wait/registry/network/admin操作は全0。実fileはT01/02/03/07/08の隔離公開だけで、fixture/結果fileを別counterとする。T04/05/06/11/12–14等のprivate代用、T09のstate/helperなし互換、T15–17の本物probe抽出dispatch、T18の未確定contextを設計どおり分離する。元cleanup自体は起動せず、元throwからtest catch/finally markerへの復帰とmain/cleanup逆diff不変を接合する。markerを実cleanup成功と呼ばない。

Test-FirstFailureの必須interfaceはSourcePath/ExpectedSourceSha256/ProbePath/ExpectedProbeSha256/OutputRoot、Phase/任意case指定なし。局所OutputRootはpacket直下の新UUID、実probeはverification直下の新UUIDである。実行commandは受領設計の「有限18局所case・作者seam・command」に固定し、親が保存旧35AE/新source・承認hash・probe hash/新UUID絶対pathを渡す。作者は旧sourceにT01のみを実行し、元throw到達・sidecar欠測を唯一のRED、他17件をNotApplicableとする。旧helper欠測/抽出不能をREDへ算入しない。新sourceは18/18GREENを必要とする。

作者はprobe未作成の段階で独立18期待/AST adapterを先に凍結し、実probe候補生成後に旧RED→新GREENを各1回行う。mock-only probeで本物入口を代替しない。確定組の静的接合/ValidateOnly後、runnerは18件/KH20/OS2/control2を繰り返さずENV01だけ一回とする。実装差戻し時の影響case再検証はENV01再実行の許可ではない。保存不能を安全に扱えたことを診断保存成功と呼ばず、正常一回で得た新診断を35AEの過去原因へ遡及適用しない。

### 7役のexact割当・リスク・次handoff

以下はFF-A/B/C/D各単位の割当である。現呼出可能一覧にあるexact ID/effortを選定し、親はdispatch時に再照合する。通常1 helperずつ、独立作者/実装/runner/reviewerを分け、再委譲せず、親model/effortと品質gateを変更しない。

| 役 | model / effort | 各単位の責務・難易度/リスクと選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | A後精緻化/B–D境界・差戻し、本plan新節だけ。高/高、診断受入と主gate/承認回数の分離が必要 |
| external-spec author | `gpt-6-astra` / `high` | A–D新規執筆非該当。利用者承認済み内部診断で元外部契約不変。安全判定/期限/承認の変更が必要ならこの固定役へ戻し、現packetでは進めない |
| designer | `gpt-6-astra` / `high` | A設計、B/C逸脱時差戻し、D新設計非該当。高/高、failure再入・保存失敗・actor束縛と専用入口を一意化するため |
| test author | `gpt-6-astra` / `high` | Bの独立18pure/file oracleと旧RED/新GREEN、Cは自己test証拠の凍結のみ、A設計後/D期待変更なし。高/高、診断失敗を本体PASSへ誤換算せず、1回上限と情報非漏洩を独立確認するため |
| implementer | `gpt-6-astra` / `high` | BのInvoke局所診断/新probe、Cその束縛のみ。A/D実装非該当。高/高、abort時計・throw・cleanupを壊さず入力最小化するため |
| test runner | `gpt-6.1-sol` / `medium` | Dの確定probe一回と実測記録。A–CのOS実測非該当。中/高、固定手順で未知値/正式とtmp/creditを分離し独立採取するため |
| reviewer | `gpt-6-astra` / `high` | A設計Pre、B/C差分/有限oracle/束縛/実行数の独立接合、D限定Post。高/高、診断追加によるfailure隠蔽・権限/試験範囲拡大を見抜くため |

適用skillは既存7役の担当工程を維持する。plannerはplanning-and-task-breakdown、designerはapi-and-interface-design、作者はtest-driven-development、実装/runnerはwordweave-changeの担当範囲、PS作成担当はpowershell-automation、reviewerはreview-software-design又はcode-review-and-quality/powershell-reviewの該当範囲である。skill/model不在はblockerでありsilent fallbackしない。返却実metadataは指定値とは分離する。

主要リスクは診断保存の再入/遅延、保存例外による元failure消失、未確定rootへのwrite、情報過剰保存、最初のactor failureを全体初因と誤認すること、ENV01成功時に他caseへ進むことである。FF1–4、独立負例、単一dispatchと旧証拠不変を必須とする。rollbackは今回の診断/probe差分だけを親が保存旧版へ戻す手順であり、旧観測欠測は復活するため実行や合格を解除しない。他者編集/証拠は保持する。

暫定未決だったwhitelist/出力/source束縛、再入を避けるwriter、元contextのENV01専用入口、18case/上限/commandはdesign56F1と独立Preで解消した。未検証なのは実装と受入証拠であり、方式の未決とは区別する。新安全権限・時間変更・追加OS実測が不可欠なら具体的不足を親へ返す。次handoffは本精緻化の親受領後のww-test-author `gpt-6-astra/high`、所有は新Test-FirstFailure.ps1と新隔離証拠だけである。独立18期待凍結→別実装者Invoke/probe→作者旧T01 RED/17NA・新18GREEN→凍結/static/ValidateOnly→独立runner ENV01一回→限定Postの順を維持する。失敗時は証拠を返し、原因patch/retry/後続caseを追加しない。

開始共有週枠は親受領52%。本plannerの開始/終了token・session/実model metadata・終了週枠・親子包含は未取得、推定しない。今回所有は本新節だけ、文書diff/hash/参照確認のみ、他file編集/実装/試験起動/再委譲/公開0である。

## MISSING-IDENTITY01 PacketBrief（2026-10-16、MI-A設計/Pre受領・限定確定）

利用者の最新指示「了解、設計し、以降の作業も実施してください。」を親から受領した。対象は保存されたFIRST-FAILURE01のENV01失敗に対する、既登録・保持handle付きの同世代候補でOS CommandLineが欠測した場合の限定是正である。旧FF-Dの一回承認は履行済みであり、本指示を後続設計・実装・検証の根拠とする。失敗を反復して成功例を得る許可ではない。

本packetの完了境界は、等価な所有権証明の限定設計/Pre、独立回帰、局所実装、独立実測/Post、正常ENV01一回の成立を確認し、既定KH-D/CXT-E focused10→標準80→既定製品fullの段階再開へ返すことである。各下流gateの成功・試用EXE提供は別の既定受入である。本plannerの今回の完了は本追補・文書検査・親への返却まで。計画/task-listの正本は本節のみとし、未完了の旧plan/todo/checklistと旧失敗は保持する。

受領設計は[既存designのMI節（717行以降）](early-trial-gate-design.md)、SHA256 `9679B02C1F3F31A6DE701656A52C4DB0DA3A39A289FC8D498C8898EFDE4AA1AE`。親からfresh独立 `missing_identity_pre`（指定Astra/high）のPre PASS・確定指摘0を受領した。外部仕様変更なし、Invoke33C2不変を確認して本節を限定確定する。方式/型/fixture/20MissingPure・2MissingOs/commandは受領設計を唯一の正本とする。設計の入力planAA92は仮計画時の履歴として保持し、今回hashへ循環更新しない。MI-Aの完了は実装/実測成功を意味しない。

### 保存証拠・外部契約の扱い

- 開始HEADは `a84ad3ae5efbbaa576db212780987665e32d2e87`、git statusはclean。Rは `target/early-trial-build/et3/cargo-gate`、packetは `R/verification/argv-path01` である。
- 現Invokeは `33C21282E728E3F654C727286F6AE462B0BF4A5D48E4A56A009E0D42A22FE38C`、FF probeは `62DA68853F6CD11B86206999C1F892F5C578D670F1C0A248453F149AD7C71B9F`、FF author testは `AD492926E4DC4CA84766710D41CFF9024065A88D79BAF310226B96026DAECD46`。旧plan0CC4/design30F4との固定組を保存する。
- 保存suite `R/verification/6bb4409b-200d-4fa6-a144-9157a37468d2` / run `R/runs/62900800-fee6-4d5d-a590-940ea5939319`、独立報告 `packet/f8918c3a-e070-4e0e-91c7-378426128d40/REPORT.md` と[review冒頭](early-trial-delivery-review.md)が根拠である。RunOwner13520のOwnedTree.IdentityCatchはPID35916/parent23292、snapshot/known/rowFilter各1、command/path存在確認false、source600で拒否した。candidate identityはnull、Owned12終了確認/Unprovable1終了不明、Supervisor/Runner正式terminal欠落、ENV01FAILである。
- plannerのsource確認でも、Read-OwnedTreeは既知候補の有効保持handleを選んだ後にGet-Identityを呼び、Get-IdentityはCommandLine欠測でthrowする。この分岐到達は確認済みである。欠測理由、終了との因果、権限不足、旧runとの原因同一性、保存候補の安全な回収可能性は未確定である。
- 既存外部仕様G5の同世代所有/未知非停止、G1–6/AP1–4、terminal/cleanup/80case期待を再利用し、外部仕様担当の新規執筆は非該当と確定した。KH手順6の元Get-Identity全5field再取得は、受領MI設計の完全登録・元SafeHandle連続保持・元Rows一意/UTC6/UTC7/key一致を証明し、path/command両propertyが存在して一方以上の値だけがnullの場合に限り置換する。外部判定の緩和が必要なら固定外部仕様担当へ戻し、仕様レビュー/必要な利用者確認前に実装しない。
- PTH-MONO、CXT-D12、KH OS2/Pure20、FIRST-FAILURE局所18/独立Postの成功は各旧固定組の証拠として保存する。今回のCommandLine欠測ケースを既存mockの成功で確認済みにしない。

### 要求・除外・確定した証明

| ID | 要求と受入条件 |
| --- | --- |
| MI1 | 元snapshotと既存台帳の完全identity・連続保持handleの同世代を、現行と等価以上の証拠で確認できる場合だけ既知所有を維持する。PID一致、known/snapshot件数、古いregistration、別actorのunknown0、欠測そのものを所有根拠にしない |
| MI2 | 欠測fieldを新しい観測値として捏造せず、登録済みidentity/ownershipEvidence/key/handleを保持する。取得できたfieldの矛盾を無視しない。部分identity、無効/閉鎖/別handle、曖昧行/世代、不正creation、新規候補は従来の拒否または元新規検証経路を維持し、既知照合失敗後のPID reopen救済は不可 |
| MI3 | 終了は既存の実handle観測、kill/wait、正式terminal、cleanup条件で別に判定する。欠測→Exited、row skip→cleanup成功、Unprovable→Ownedへの無根拠変更は不可。FFの最初の失敗保存、未知null、非停止、元例外/abort/期限を保持する |
| MI4 | 独立作者が新しい欠測境界の旧REDと新GREEN、危険な類似入力の拒否、実型/保持handleconsumer成立を有限ケースで確認する。最終固定組で独立検証・reviewを通過してから正常ENV01を新suiteで一回だけ実行し、結果/失敗/残NotRunを保存する |

確定方式はRead-OwnedTreeの既知分岐だけで、完全な5field identityとObservedの登録evidence、元Process/元SafeHandle、CIM UTC6桁と保持開始時刻UTC7桁/key、parentの一致を接合することである。存在するpath/commandは元helperで登録値と比較し、矛盾・不正型・property自体の欠落を拒否する。欠測pathのimage API呼出は0であり、登録path/commandを現在のOS観測として補完しない。両値presentなら元Get-Identity/全5field照合を維持する。前後のhandle guardを通過するまでcontinueせず、getter例外・閉鎖・参照変化は元Unprovable/abortへ渡す。新helper/API/signature、Get-Identity等の別consumer変更は0である。

対象外はactor/root/source/data/credential/API/admin/OS設定、時計/上限/予算、未知判定、JSON schema、native/fake/製品source/EXE、argv exact、別監督方式、一般化framework、追加常駐監視である。元Rowsの書換え・CIM取り直し・GetProcessById retry・台帳への欠測埋め・新規候補への既知専用扱い拡張は不可。必要な所有範囲追加は理由と差分境界を親/plannerへ返し、暗黙に拡張しない。

### 有限作業リスト・依存・所有・verification

通常1 helperずつfresh context、再委任なし。作者/実装者/runner/reviewerを独立させ、同じfileは逐次所有する。下表は受領設計/Preに基づく確定所有である。作者着手まではsource33C2と保存source fixtureを保持し、作者もproductionを編集しない。

| 単位 | 依存・規模・所有 | 受入・必須verification |
| --- | --- | --- |
| MI-A 等価証明の設計/Pre | S、既存designのMI追補のみ | design9679・fresh独立Pre PASSを受領し、本精緻化で完了。外部unknown/cleanup基準不変、B以降は未実施 |
| MI-B 独立回帰の固定 | Aと本精緻化の親受領後。S、作者はpacketの `Test-KnownHandle.ps1` のMissingPure/MissingOs専用差分と新隔離fixture/reportのみ | 設計の20/2 oracle・adapter・型・順序・上限を固定し、保存旧33C2でM01一回BehaviorRed/19NotApplicableを採る。旧Pure/Os本文とP09/P20の履歴oracleは改稿しない。adapter/API不足はREDと区別し、source不変で所有返却 |
| MI-C 局所実装 | B所有返却後。S、別実装者は `R/Invoke-CargoGate.ps1` のRead-OwnedTree既知分岐だけ | 作者期待を編集せず接合し、作者が新MissingPure20GREENを一回取得する。拒否不変・借用Dispose/reopen/追加CIMなし。Parser0、固定PSSA1.25.0診断0/実行error0、非所有逆diff/bytes一致。Get-Identity/新helper/APIへの拡大はAへ差戻す |
| MI-D 凍結/必要な束縛/独立接合 | B/C最終bytes後。S単位に逐次分割、作者は必要なtest束縛、別実装者は必要なdriver/probe/HANDOFF束縛だけ | 独立runnerがMissingOs2→成功時MissingPure20を固定組で各一回、firstUnexpected停止。旧bytes/非所有guard/FF本文不変・最小依存鎖を独立reviewし、接合Pre/局所Post PASSと再束縛ValidateOnly dispatch0/runtime0を得る。旧KHのPure/Osは今回dispatchしない |
| MI-E 正常ENV01一回/Post | D成功・親の固定組dispatch後。S、runnerは新隔離証拠/log/reportのみ | 既存FF probeの意味を保ちENV01だけ一回、新suite/run・retry0/他case0/全体credit0。元terminal/cleanup/固定入力一致を独立Post確認。初回予期外/FAILは保存して停止し、自動patch/retry/残9開始をしない |
| MI-F 既定ET3への接合 | E PASS/Post後。Sの段階dispatch、source/期待編集なし | KH-D/CXT-Eの残るfocused10/AP1の最終固定組・接合reviewを確認して一回firstFailで実行。MI-Dの新局所受入が旧KH-D Pure/Os dispatchを置換するため、Fで旧Pure/Osを再起動しない。focused全PASS/Post後だけ標準80/既定variantへ、全必須gate成功後だけ製品fullへ進む。旧NotRunをPASSへ補わず、別原因は別findingとして停止 |

- [x] MI-0：保存証拠・source分岐・既存契約を読み、仮計画と役割/所有/停止条件を作成した。
- [x] MI-A：既存外部契約再利用、design9679の具体interface・有限20/2ケース・command・独立Pre PASSを受領し、本節を限定確定した。
- [ ] MI-B/C：独立oracle/旧RED→別実装→新GREEN、静的検査と非所有一致を完了する。
- [ ] MI-D：必要な束縛と独立局所/OS検証・接合reviewを完了する。
- [ ] MI-E：正常ENV01一回と独立Postを完了する。失敗時は証拠を返し停止する。
- [ ] MI-F：各既定gateの成功を条件にfocused10→80/variant→製品fullへ順に返す。

固定局所集合は設計MI-M01–20とMI-O1–2である。M01が旧P09のcommand-null拒否、M02が旧P20のpath-null image API失敗拒否を、完全登録・連続保持handle成立時だけ置換する。旧P04/05/06/07/08/13/14/15/16等は分岐前guard全文不変と旧固定証拠で接合する。不変前提が崩れたらPreへ戻す。新MissingPureは各5秒/全60秒、MissingOsは実子1個・finally込み15秒/子自身10秒でO1→O2、retry0である。実型/保持handleに合成欠測rowsを供給するOS2であり、実OS欠測原因の再現とは呼ばない。

fixture型・AST置換/逆照合・Get-Identity/ConvertTo-CodeUnitのcounter wrapper・原文body・出力個数・単一event順序・誤順memory対照は設計で確定済みである。実Get-Identityの旧拒否をthrow mockで再現しない。source/hash/OutputRoot/Phaseの4必須引数と具体commandも設計に従う。実OSで未測定のDispose/Kill/Waitをruntime測定0へ補わず、AST/参照/open0による導出根拠とfixture回収を区別する。旧FF18/PTH/旧KH20/旧Os2は不変consumerのbytes証拠で接合し、一括再実行しない。

変更したInvoke/Test/必要なbinding対象の最終PSSAは、先頭からErrorActionPreference=Stop→固定1.25.0 Import→解析結果保存/件数判定→全対象終了後の `PSSA_COMPLETE count=0` →process exit0を同一logで必須とする。空出力/古いlogでは実行error0を認めない。Parser0、非所有逆bytes、Read内の元GetProcessById2箇所とDispose受信者、FF helper/二caller/元catch不変を併せて確認する。

### frozen binding・下流の解除条件

本追補だけで旧probe/driverのplan bindingは失効する。旧FF probeをそのまま再実行しない。確定plan/design/Invoke→使用するtestの固定入力→新test hash/handoff→実行するprobe又はfocused driverの固定入力→親のfrozen記録の順で更新する。自分のhashを自分へ追記する循環更新を避け、設計中の入力plan hashは履歴として保持する。

MI-E前は実際にFF probeが読む依存だけを再束縛し、FF probeの単一ENV01/前後照合/12入力/dispatch結果契約は変えない。FixedInputsは既定hashtable（絶対path string→大文字64hex string）12件のまま、Test-KnownHandleを追加しない。Ordinal順前12照合→case一回→後12照合と元例外優先順・dispatch全文を保持する。MI-F前にTest-ContextPreflight/CONTEXT-TEST-HANDOFF-P1→focused driver/HANDOFFの必要束縛を独立作者/別実装者の所有に分けて更新する。一度に3–5fileを超える束縛作業にしない。旧bytes・承認・suite/run/frozen-inputsは非上書きで保存し、旧結果の意味を書き換えない。新hashの自動採番だけを承認根拠としない。

ENV01成功はprobeのcredit0を維持する。focused10/標準80を代替しない。製品fullへ到達した場合は既定の通常/試用構成・関連crate・native gateと、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` を既定wrapper/隔離target/承認条件で実施する。既存成功source/buildの証拠はhashと範囲が一致する限り保存し、本PS是正だけを理由に製品buildを反復しない。製品EXE更新は実際の既定提供gate通過後である。

### 各単位の7役exact割当・リスク

以下はMI-A/B/C/D/E/Fそれぞれへの割当である。非該当工程は起動せず責任だけ保持する。現呼出可能一覧にあるモデルと対応effortを選び、親はdispatch時にも照合する。指定値と実返却metadataを分離し、後者がない場合は未取得とする。

| 役 | exact model / effort | 単位の責務・難易度/リスク・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | A精緻化、B–F差戻し/境界、本planのみ。高/高、所有権証明とgate解除の分離が必要 |
| external-spec author | `gpt-6-astra` / `high` | Aで外部契約再利用を確認する責任。B–F新規執筆は非該当。外部変更が必要ならAへ戻して仕様化。高/高、内部是正を安全基準緩和と混同しないため |
| designer | `gpt-6-astra` / `high` | Aの方式/型/fixture/oracle、B–Dの設計逸脱差戻し、E/F新設計非該当。高/高、欠測と世代/handle寿命の厳密な証明が必要 |
| implementer | `gpt-6-astra` / `high` | C本体局所、D及びF前のdriver束縛。A/B/E/F実行中は本体編集なし。高/高、停止権限と保持handleを扱い誤同定の損失が大きい |
| test author | `gpt-6-astra` / `high` | B独立oracle/fixture、D及びF前のtest束縛、Aの試験可能性確認。C/E/F期待編集なし。高/高、旧mockで欠測失敗を隠すことと期待値追随を防ぐ |
| test runner | `gpt-6.1-sol` / `medium` | D固定局所/OS、E一回ENV01、F段階gate。A–C新runtime dispatchなし。中/高、確定手順の独立採取・firstFail/正式証拠を扱い設計判断は委ねない |
| reviewer | `gpt-6-astra` / `high` | A Pre、B–D独立差分/証拠/束縛、E/F Post。高/高、同世代証明・台帳保全・未知非停止と過大creditを独立検査 |

担当skillはplanner=planning-and-task-breakdown、designer=api-and-interface-design、作者=test-driven-development、実装/runner=wordweave-changeの担当工程、PS作成=powershell-automation、reviewer=review-software-design又はcode-review-and-quality/powershell-reviewである。新規skill/別frameworkは作らない。未導入skill/未対応modelはblocker、silent fallback禁止、親model/effortと品質gateは不変である。

主要リスクは欠測を信頼へ変換する誤り、PID世代混同、登録後に矛盾したfieldの無視、borrowed handle解放、旧case期待の無断削除、古いbindingで新sourceを実行することである。元snapshot・完全台帳・連続handleの証明、拒否負例、独立review、旧bytes保全と最小再束縛で対処する。rollbackは親が本packet所有差分だけを保存版へ戻す操作であり、旧失敗を成功にせず、runtime再開は再判定する。

### 未決・次handoff・測定境界

仮計画の未決だった等価証明、path欠測、Read-OwnedTree限定所有、P09/P20追跡、有限ケース/command、影響consumerはdesign9679と独立Preで解消した。残る不確実性は未実装/未実行の受入、OS欠測原因、同期API停滞、実suiteの別失敗である。新たな利用者仕様判断はない。証明が実型で成立しない、又は安全基準/権限/期限の変更が必要な場合は具体的不足を親へ返す。

親の一次資料照合を受け、designerが.NET10.0.12/PowerShell7.6.6の既存consumerへ対応付けた。StartTimeはcacheされ得るため毎回新OS照会と呼ばず、元handleの連続保持を証明に含める。実型の成立はMI-Oの未実施受入である。OS欠測原因の証明、新PInvoke/API追加の指示として扱わない。

次handoffは親が本精緻化を受領後の `ww-test-author` / `gpt-6-astra` / `high` である。入力は本返却hash、design9679/独立Pre PASS、Invoke33C2、既存Test-KnownHandleと保存証拠。所有は同testの新MissingPure/MissingOs phaseと新隔離fixture/reportだけである。順序はMI-B作者freeze/旧M01RED・19NA→MI-C別実装/作者新20GREEN→MI-D独立MissingOs2・MissingPure20/必要束縛review→MI-E正常ENV01一回/Post→条件付きMI-F focused10/80/fullである。旧Pure/Osの再dispatchは含めない。全役exact割当は上表のまま。親が承認・dispatchを担い、plannerは他agent起動/実装/試験編集/公開を行わない。

開始共有週枠は親受領63%、アカウント共通値である。子 `/root/missing_identity_plan` の開始/終了token、session ID、実返却model/effort、終了週枠、親子包含は未取得。今回の測定範囲は計画文書作成から最終応答前までであり、task専有消費や削減率を推定しない。親が既存usage記録へ接合する。本plannerの検証は文書diff/hash/参照/所有範囲のみ、runtime/実装/再委任/公開0である。

## ENV-RUNNER-ORACLE01：固定runner期待値の限定是正（2026-10-16、独立Pre提出）

親から「既定forward slash文字列への厳密比較を維持し、テスト期待値だけを直して続行」の利用者承認を受領した。開始HEADは `7b5607e`、着手時statusはcleanである。仮計画を既存外部契約G1/G4・本書のrunner一語契約・[designの固定runner契約](early-trial-gate-design.md)へ照合して本追補に精緻化した。新しい外部仕様/設計の執筆は非該当である。独立Preは未了であり、既存MIや旧未完了checklistを完了へ書き換えない。本節を今回の計画・task-list正本とする。

Rは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、Pは `R/verification/argv-path01`。source確認ではInvokeの2022行が既定forward文字列を生成し、Testの2043–2044行はUTF-16配列から復元したrunnerをcanonical backslashの `Fixture.run.nativeExe.path` と `-ceq` 比較している。契約に対するoracleの誤拒否であり、観測値を正規化する理由にはならない。保存suite `R/verification/8b8b1d3d-b9d8-428a-90fc-6537a0ee7d02` / run `R/runs/3ef364ec-b7c1-4b9b-9d9b-dda74908bc9e` は失敗証拠のまま保持する。

入力SHA256はdesign `9679B02C1F3F31A6DE701656A52C4DB0DA3A39A289FC8D498C8898EFDE4AA1AE`、Invoke `318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3A`、Test `D8AD0AD44C2457DC60E64E52A6EDB455974D5F8AD9938B956CEF3E4DDAB0F7F3`、既存FF probe `EE5FAA688B19049D74D14ED4B3603E3325ADD3FC068F4F84446DA002C9C51389` であり、plannerが現物一致を確認した。旧bytesとその承認・結果を保存し、新しいhashは親の凍結記録で別に扱う。

### 要求・除外・固定回帰

期待値Eは次の文字列そのものである。fixture/producer/実測値から導出せず、観測側のUTF-16復元と `-ceq` を維持する。

```text
F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/cargo-gate-entry.exe
```

- EO1：Testの当該比較の期待値だけをEへ修正し、Eだけをacceptする。path存在/等価性、大小文字無視、slash置換、trim、quote除去、引数分割による救済は不可である。
- EO2：独立作者が実Testの当該AST比較statementをsource hash・抽出件数1・元本文一致に束縛して実行する。期待条件は作者側で先に固定し、比較を別の模倣predicateへ置換しない。全Confirm関数のmock、大量fixture、新production entry/helper、構造refactorは作らない。
- EO3：通常ユーザー・合成のみ、実API0/admin0/製品変更0を維持する。局所合格・ENV01合格をfocused10/80/fullへ加算しない。予期外の初回結果で停止する。

有限回帰は6件である。R01=E accept、R02=Eの全 `/` を `\` にした値 reject、R03=先頭 `F:` だけを `f:` にした値 reject、R04=Eに ` --extra` を付加 reject、R05=E全体を二重引用符で囲む値 reject、R06=末尾ファイル名だけ `cargo-gate-entry-other.exe` にした値 reject。旧TestではR01の誤拒否1件をBehavior REDとして採り、残5件はNotApplicableとする。新Testは6件すべてGREENを要する。抽出/fixture/起動/構文失敗はREDではない。旧失敗suiteの再実行は不要である。

producer/native/fake、期限/台帳/schema、元ENV01の他の期待、UTF-16復元、dispatch/cleanup/terminal、12入力の集合・型、既存creditと親承認、OS/credential/data、親モデルと品質gateは変更禁止である。回帰は同じ比較statementの利用箇所全体への静的接合を含むが、他ENV caseの実測は今回追加しない。

### 有限作業・依存・所有・検証

| 単位 | 依存・規模・所有 | 受入・検証 |
| --- | --- | --- |
| EO-A 契約再利用/Pre | XS、本plannerは本追補のみ。reviewerはread-only | 親受領→独立PreでE、6ケース、所有分離、既存spec/design再利用、停止条件を照合してPASS。文書diff/参照検査。新仕様判断が出たら固定spec authorへ戻す |
| EO-B 独立回帰/旧RED | A後、S。作者は新 `P/Test-EnvRunnerOracle.ps1` と新UUID隔離fixture/結果だけ | 実statement抽出・最小assert捕捉と固定6期待を先に凍結し、保存旧TestでR01 RED/5NAを一回採取。作者はTest-CargoGateを編集しない。source/hash/commandと実exitを報告し、adapter失敗は停止 |
| EO-C oracle修正/GREEN/Post | B所有返却後、XS。別実装者は `R/Test-CargoGate.ps1` の当該期待値だけ | Testは本単位では是正対象oracle実装として実装者が所有する。作者の独立回帰を変更せず、独立runnerが新6GREENを一回採取。変更ps1のParser error0/PSSA1.25.0診断0・実行error0、非所有bytes一致、独立局所Post PASSを要する |
| EO-D 最小hash再束縛/ValidateOnly | C Post後、S。別実装者は既存 `P/Invoke-FirstFailureEnv01.ps1` の必要hash literalのみ。親は既存frozen/handoff記録 | 最終plan/Test→probe→親frozen記録の順で必要bindingだけ更新する。Invoke/design等不変、probe dispatch全文不変、12入力契約不変を独立確認。変更probeのParser/PSSAを確認し、既存probe commandのValidateOnlyを新OutputRootで一回、dispatch0/runtime0を得る |
| EO-E ENV01一回/Post | D成功と親の固定組dispatch後、S。独立runnerは新隔離log/結果のみ | 既存FF probeの同commandからValidateOnlyだけを外し、新OutputRootでENV01を一回。retry0/他case0/credit全0、正式terminal/cleanup/前後hashを独立Post確認。FAIL/予期外は保存して停止する |
| EO-F 条件付き既定gate再開 | E PASS/Post後。今回新source/期待の所有なし | 既存MI-F/KH-D/CXT-Eのbinding・接合確認後にfocused10→標準80/variant→既定製品fullへ各前段PASS/Postを条件に逐次返す。旧NotRunを補完しない。別失敗は別findingとして停止する |

- [x] EO-0：仮計画を作り、既存外部仕様/設計と現source/hashを照合して本追補へ精緻化した。
- [ ] EO-A：独立Pre PASSと親の受領。
- [ ] EO-B：独立期待凍結→旧R01 RED/5NA。
- [ ] EO-C：別実装者の期待値限定修正→独立6GREEN/静的検査→局所Post。
- [ ] EO-D：必要bindingのみ更新→独立差分確認→ValidateOnly。
- [ ] EO-E：ENV01一回→独立Post。失敗時停止。
- [ ] EO-F：既定下流gateを各成功条件付きで再開。

新回帰commandは作者がSourcePath・承認済みExpectedSourceSha256・新OutputRootを明示して凍結し、旧/新の相違をsource hashで識別する。本文から現在hashを採番して承認の代用にしない。固定pwsh7と既存検証方式を用い、PSSAはErrorActionPreference=Stop、固定1.25.0、診断配列/件数、全対象後の `PSSA_COMPLETE count=0`、process exit0を同一logへ残す。本追補とTest変更の双方で旧probe bindingは失効する。probe自己hashを自己埋込みせず親が最終hashを保持し、未使用のtest/driverへ連鎖更新を広げない。製品buildの反復は本局所是正の受入に含めず、既定提供gateへ到達した場合の必須test/buildは維持する。

### 各単位の役割・exact model/effort・リスク

次表はEO-A〜Fそれぞれに適用し、非該当役は起動しない。利用可能一覧と対応effortを親がdispatch時にも確認し、通常1 helperずつfresh contextで独立作者/実装者/runner/reviewerを分離する。7役の責任を7agent常駐へ置換しない。

| 役 | exact model / effort | 難易度/リスク・選定理由と責務 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 中/中。固定役。本追補と差戻し境界だけを所有 |
| external-spec author | `gpt-6-astra` / `high` | 低/中。固定役。既存契約再利用で新執筆/起動なし。外部契約変更が必要なら戻す |
| designer | `gpt-6.1-sol` / `high` | 低/中。固定文字列/最小AST回帰の既存設計再利用を担当。新設計は非該当、逸脱時だけ判断を戻す |
| implementer | `gpt-6.1-sol` / `medium` | 低/中。Cの期待値一箇所とDの逐次hash literalだけ。明確な所有境界と非所有一致で検証できる |
| test author | `gpt-6.1-sol` / `high` | 中/中。Bの独立6期待と実statement抽出を担当。模倣oracleやfixtureの都合で誤りを隠さない判断を要する |
| test runner | `gpt-6.1-sol` / `medium` | 中/高。Cの6GREEN、D ValidateOnly、E一回実測、F既定手順。確定hash/exit/firstUnexpectedと停止の証拠採取を担当 |
| reviewer | `gpt-6-astra` / `high` | 中/高。A Pre、C局所Post、D束縛、E/F Post。比較緩和や誤credit、古い承認再利用を独立検査 |

担当skillは既存7役の工程内に限定する。PS作者/実装者はpowershell-automation、reviewerは該当するreview-software-design又はcode-review-and-quality/powershell-reviewを読む。model/skill不在はblockerであり無言代替しない。指定と実返却metadataを分離し、未取得値を推定しない。親model/effortは変更しない。

主なリスクは①期待のproducer追随、②回帰が実statementと乖離、③旧bindingでのdispatch、④ENV01成功を全体成功と誤認することである。Eの独立固定、source-bound抽出、最小再束縛、credit0と段階Postで抑える。今回の仕様上の未決はない。Pre/adapter成立/新6GREEN/ENV01結果は未検証であり、追加helper・比較緩和・期限変更が必要ならこの限定計画を越えるため親へ返す。

次handoffは親から `ww-reviewer` / `gpt-6-astra` / `high` への独立Preである。本追補の最終SHA、上記入力hash、既存契約と利用者の限定承認を渡し、read-onlyで判定を受ける。PASS後はEO-B作者へdispatchする。本plannerは本書以外の編集、試験/runtime、再委任、公開を行わない。開始共有週枠は親受領73%。本子の終了週枠・開始/終了token・実model metadata・親子包含は未取得、task専有量は不明である。親が既存usage記録へ接合し、削減率を推定しない。

## EO-STATIC-BASELINE01：既存静的6件の限定是正（2026-10-16、独立Pre提出）

親から既存静的6件の追加是正について利用者の「はい、対応してください」を受領した。開始HEADは`c281448`、statusはcleanである。仮計画を既存G1/G4/G5、EO1–3、design `9679B02C1F3F31A6DE701656A52C4DB0DA3A39A289FC8D498C8898EFDE4AA1AE`、下記6関数と実consumerへ照合し、本追補へ精緻化した。外部動作・内部処理は変更しないため新spec/design文書は非該当、局所属性方式の技術的妥当性は独立Preで確定する。本節を追加単位のplan/task-list正本とし、既存未完了項目は保持する。

R/Pは前節の定義を継承する。入力Testは`96FB6B4B9640BB2055463AE5E53CD70D92E2EE332B4B1EF81B5DEACCC89F1EDF`、独立回帰は`P/Test-EnvRunnerOracle.ps1` / `E752D9DA7E4BC3CF4A14ED1A64C39184327C1BA95E8A789FAB831CEF672D7170`である。親受領ではforward期待修正後の6回帰GREEN済みである。一方、`P/8ff8babe-be07-4593-9346-28db1261c6f2/REPORT.md`と`diagnostic-comparison.json`では旧D8AD/新96FBとも固定PSSA1.25.0の同一6診断、engine error0、exit1、新規0/消失0である。旧診断だから合格とはせず、EO-C総合`CHANGES_REQUIRED`とEO-D/E以降の未解除を維持する。

今回の承認はEO-Cの「当該期待値だけ」という所有制限を、下記6個の関数局所属性追加に限ってsupersedeする。診断0、動作不変、独立review、以後の束縛/ValidateOnly/一回ENV01という品質gateは変更しない。既存6GREENと旧静的失敗は旧snapshotの証拠として保持し、新sourceの受入へ流用しない。

### 固定契約・個別根拠

各関数の既存`param`直前に、次の属性を1個だけ挿入する。`<J>`は表の英文そのものとする。関数内param blockへの局所属性であり、script scope/Target/Scope wildcard、global設定、rule除外、severity引下げ、`CmdletBinding`/`SupportsShouldProcess`/確認promptの追加は許可しない。

```powershell
[System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseShouldProcessForStateChangingFunctions', '', Justification = '<J>')]
```

| 96FB上の関数/行 | 実consumerと省略不可の理由 | 固定Justification（J） |
| --- | --- | --- |
| `Start-FixtureProcess`:593 | identity control、sentinel/controller/replay、real cargo fixtureが戻り値のprocess/streamを消費する。起動だけの省略は後続の観測・所有と整合しない | `Internal synthetic-test launcher; callers require a started process and captured streams. Optional WhatIf or confirmation would invalidate fixture and cleanup contracts.` |
| `New-CaseFixture`:652 | `Invoke-GateCase`が新suite配下fixtureとRun、合成source/hashを直後の刺激・公開に渡す。部分生成のdry-run契約は存在しない | `Internal synthetic-fixture constructor; Invoke-GateCase requires complete isolated files and run metadata. Skipping construction would invalidate the case contract.` |
| `Set-CaseStimulus`:903 | `Invoke-GateCase`がactions/flag/期限等の固定刺激を元case期待と対にして消費する。省略すると別条件の試験になる | `Internal case setup; isolated mutations and flags define the approved test stimulus. Optional skipping would test a different condition against unchanged expectations.` |
| `Set-CaseSynchronization`:1004 | 同callerがdescriptor/hold flagを公開し、`Invoke-CaseSynchronization`が固定hookを消費する。省略すると競合/障害条件を失う | `Internal synchronization setup; descriptors and isolated hold flags establish the approved race and fault conditions. Optional skipping would invalidate those cases.` |
| `Stop-ObservedIdentity`:1040 | 同期刺激、real-fixture失敗回収、`Invoke-GateCase` finally救済から使用。保持handle、世代、image、fixture所有の確認を通った停止だけであり、省略は刺激/回収を損なう | `Internal identity-checked fixture stop used by fault injection and failure cleanup. Existing generation, image, ownership and retained-handle checks must run; optional confirmation could leave fixture processes alive.` |
| `New-RealCargoFixture`:1235 | `ORDER07`だけが隔離Cargo source生成・offline baselineとそのartifactを消費する。途中省略でbaseline/source束縛が壊れる | `Internal ORDER07 fixture builder; isolated Cargo files and offline baseline artifacts are required together. Optional skipping would invalidate baseline and source bindings.` |

これらは副作用がないという主張ではない。副作用は合成fixtureの成立に必要であり、対話的省略契約を持たない内部test helperとして当該推奨ruleだけを局所的に適用除外する理由である。新helper0、関数名/引数/既存属性/本文/consumer/dispatch/cleanup/例外/時計/schema/期待値変更0とする。特にStopの安全判定・実停止を属性の理由で緩和しない。sourceのline移動は許容するが、BOM/encoding/改行と全既存bytesを保持する。

### 有限作業・依存・所有・受入

| 単位 | 依存・規模・所有 | 受入・verification |
| --- | --- | --- |
| ES-A 契約/Pre | XS。本plannerは本追補のみ、reviewer read-only | 既存spec/design再利用、6個の個別理由、局所配置と動作非干渉、有限oracle/所有/停止条件を独立Preで照合しPASS。必要ならplannerが本節だけ精緻化。新しい動作設計が必要ならBへ進まない |
| ES-B 独立静的oracle凍結 | A後、S。作者は`P/<新UUID>`内の一回用静的検査script/期待manifest/結果だけ | 保存96FBをhashで固定し、6関数名・rule/空checkId/J/個数/配置のAST期待、属性挿入逆diffの全bytes一致、非所有hash一致を実装前に固定する。既存EO回帰本体/期待を編集しない。作者自身が実装を変更しない |
| ES-C 属性追加/静的検証 | B後、XS。別実装者は`R/Test-CargoGate.ps1`の上記6属性だけ | 固定oracleで6対象すべてPASS、追加属性を除いた全bytesが96FBと一致、Parser error0。PSSA1.25.0をファイル全体・通常rule集合で実行し診断0/実行error0/実exit0。rule除外で作る0は禁止。予期外の診断は新findingとして停止 |
| ES-D 独立6回帰/Post | C後、S。独立runnerは新UUID結果/logのみ、reviewer read-only | 新source承認hashを引数に既存EO回帰を一回実行し6GREEN/exit0。同時に凍結静的oracleを独立実行し全対象PASS。reviewerが個別理由/AST/逆diff/全PSSA/非所有一致を接合して局所Post PASS、EO-C静的未達の閉鎖を判定する |
| ES-E 既存EO-D以後へ返却 | D PASS後、S。既存probeの必要plan/Test hash literalだけを実装者、既存freeze/handoff記録を親が所有 | 最終plan/Test→probe→親freezeの順で更新。独立束縛review、変更probeのParser/PSSA、ValidateOnly一回/dispatch0→親dispatchでENV01一回/Post→既定条件付きfocused10/80/fullの順を保持。新しい再実行権限は作らない |

独立静的oracleは有限6行の結果と共通のbytes/hash検査に留め、新framework/新常設test suiteを作らない。ASTは各名前のFunctionDefinitionAstが一意、当該ParamBlockに指定属性が一個、引数とJが完全一致、他の追加属性0を確認する。除去対象は認証済み6属性の挿入span（付随indent/改行のみを含む）とし、文字列の全域replace/再format/改行正規化で本文一致を作らない。属性除去後のbyte列が保存96FBと完全一致することに加え、定義数・関数本文・既存paramのAST/署名・caller全文一致を報告する。共通parameter追加0である。入力hash不一致・抽出不能・属性型/配置不適合は静的失敗であり、動作REDと呼ばない。属性追加がコマンドbindingを変えることが判明した場合も停止し、`CmdletBinding`を足して回避しない。

PSSAは既存方式の`ErrorActionPreference=Stop`、module固定1.25.0、`-ErrorAction Stop`、全診断配列/件数、全対象後`PSSA_COMPLETE count=0`、実process exit0を同じlogへ保存する。旧6診断は既存保存証拠を再利用し、旧sourceへのPSSA再実行は不要である。独立oracleはprocess/fixture起動0、実Testのdot-source/entry実行0でAST/bytesのみを読む。作者が承認した一回用commandとscript hashを親がfreezeし、runnerがCの静的結果を再解釈して省略しない。

既存EO回帰はline番号でなくCommandAstのcommand/message/個数1と全文、復元statement全文、親承認ExpectedSourceSha256へ束縛されているため、属性追加によるline移動を許容する（現物93–115行を確認）。固定pwsh `F:/Program Files/PowerShell/7/pwsh.exe`から、`-NoProfile -NonInteractive -File P/Test-EnvRunnerOracle.ps1 -SourcePath R/Test-CargoGate.ps1 -ExpectedSourceSha256 <親が凍結した新SHA> -OutputRoot P/<新UUID>`を用いる。R/Pは絶対pathへ展開し、現物hash自動採番を承認に代用しない。旧D8ADのRED、96FBの6GREEN、KH/MI/FF/control/OSを再実行しない。次のENV01が失敗してもretry0、別case0、既定credit全0のまま証拠を返す。

- [x] ES-0：仮計画を既存spec/design・6関数/consumer・EO回帰へ照合し、本追補へ精緻化した。
- [ ] ES-A：独立Pre PASSを親が受領する。
- [ ] ES-B：独立静的oracle/6期待/元96FBを凍結する。
- [ ] ES-C：別実装者が属性6個を追加し、固定静的oracle/全PSSA診断0を得る。
- [ ] ES-D：独立静的確認＋既存EO6回帰一回GREEN、独立PostでEO-C総合を再判定する。
- [ ] ES-E：必要hash再束縛/ValidateOnly→ENV01一回/Post→条件付き既定gateへ返す。

### exact割当・リスク・次handoff

次の割当はES-A〜E各単位に適用する。非該当工程は起動せず、親が通常helper1・fresh contextで順次dispatchする。利用可能なexact ID/対応effortを使用し、指定とprovider実返却を分け、未取得は未取得とする。固定2役、親model/effort、quality gateは不変、model/skill不在はblockerでsilent fallbackなしである。

| 役 | exact model / effort | 各単位の責任・難易度/リスク・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | A計画/精緻化、B–E逸脱判断。本追補のみ。中/高、旧所有との競合とgate不変を明示する固定役 |
| external-spec author | `gpt-6-astra` / `high` | A既存契約再利用、B–E新執筆非該当。中/高、動作/安全変更が必要なら固定役へ差戻し、現packetで処理しない |
| designer | `gpt-6.1-sol` / `high` | A局所属性方式/既存設計再利用、B/C技術不成立時の差戻し、D/E新設計なし。中/高、属性位置/関数bindingとcleanup非干渉の判断が必要 |
| implementer | `gpt-6.1-sol` / `medium` | Cの6属性、E必要hash literalのみ。A/B/D本文編集なし。低/中、固定した文字列/挿入spanを逆diffで検証できる |
| test author | `gpt-6.1-sol` / `high` | B有限AST/bytes oracle凍結、C/D期待変更なし、E既存回帰再利用。中/高、過大な削除span/文字列正規化による誤PASSを防ぐ独立期待が必要 |
| test runner | `gpt-6.1-sol` / `medium` | D固定静的oracle/既存6回帰、E既定ValidateOnly/ENV01と段階gate。A–C実runtimeなし。中/高、固定source/hashとfirstUnexpectedの忠実な証拠採取 |
| reviewer | `gpt-6-astra` / `high` | A Pre、D独立Post、E束縛/結果Post。B/C read-only。中/高、理由のない抑制・binding変化・未達gateの解禁を独立検査 |

既存7役skillを役内で用いる。plannerはplanning-and-task-breakdown、設計はapi-and-interface-design、作者はtest-driven-developmentの独立期待工程とpowershell-automation、実装/runnerはwordweave-changeの担当工程、PS編集はpowershell-automation、reviewerはreview-software-design又はcode-review-and-quality/powershell-reviewである。新skill/新文書体系を増やさない。

主要リスクは抑制による欠陥の隠蔽、Stopの省略/所有弱化、AST抽出のline依存、逆diffの過大除去、旧hashでのdispatchである。個別consumer根拠＋関数局所scope＋全文byte復元＋独立oracle/Post＋必要hash再束縛で対処する。Invoke318E/design9679は前節の完全SHAを正本とし、native/fake/製品source/実data/API/OS/credential、case/variant/期限/schema/安全品質も変更禁止である。rollbackは親が本単位の6属性だけを保存96FBへ戻し旧静的未達に戻すものであり、他者編集や証拠を消さない。

仕様上の利用者未決はない。技術的未決はPreでの属性配置/適用範囲の妥当性と、未実行のParser/PSSA/独立oracle成立である。これを利用者へ再質問せず担当reviewer/作者の証拠で判定する。次handoffは親から`ww-reviewer` / `gpt-6-astra` / `high`へのread-only Preであり、本plan最終hash・入力96FB/回帰E752・旧6診断・既存design9679を渡す。PASS後ES-B作者へ、別実装者→独立runner/Post→既存EO-Dへ戻す。Pre/本plan受領だけではENV01を起動しない。

開始共有週枠は親受領76%。子`/root/static_baseline_plan`の終了週枠・開始/終了token・session ID・実返却model/effort・親子包含は未取得、専有消費と改善率は不明である。親が既存usage記録へ接合する。本plannerの完了境界は本追補と文書diff/hash/所有確認までであり、コード/試験編集・runtime・再委任・commit/公開0である。

## TOOLCHAIN-PWSH-DRIFT01：7.6.7固定前提の限定改訂（2026-10-16、仕様/設計反映・再Pre提出）

利用者の「7.6.7でよい」という固定環境改訂の承認を親から受領した。開始HEAD `e7389c7`、status clean、本書旧SHA `88F586DD99BFFCFDF13CC675CDAC42CFA625D8E0CF40F97C91F7C912AD897BDE`。本節が今回の計画・task-listと固定環境改訂の唯一の規範追補である。既存plan/designの将来実行に対する固定7.6.6前提は、下記同定・Pre・実装・検証・束縛の完了後に7.6.7へ置き換える。過去runの7.6.6観測・承認・FAIL/PASSと.NET10.0.12根拠は当時の証拠として保持し、全履歴を置換しない。親受領だけで旧hash検査を無効化しない。

仮計画から、既存design末尾のP7-S01–05/V01–08と内部接合設計を受領し本節を精緻化した。独立PreはFF回帰の内部hash/入力binding所有漏れ1件でCHANGES_REQUIREDである。下記D4を追加し、designerのFF追補と独立再Pre PASSを親が受領してから作者/実装へdispatchする。指摘以外の可逆seam、拒否時I/O、循環回避、.NET対象span、既存局所commandsの整合は親から受領済みである。既存未完了checklistを完了へ変更せず、`tasks/plan.md`・`tasks/todo.md`も保持する。

### 完了境界・要求・除外

- P7-R1：指定絶対pathのpwsh 7.6.7のmetadata、実version、実runtimeと重要binaryの安定hashを証拠で同定し、現行のversion producer/validatorを7.6.7 exactへ整合させる。範囲許可、最低版、7.6.x wildcard、旧7.6.6併用への緩和は行わない。
- P7-R2：新runtimeに影響される局所契約の独立互換性確認、必要な現行binding、ValidateOnlyと独立Postまでを完了境界とする。source上の不変根拠と旧runtimeでの実測を区別し、新runtimeで未実行の対象をPASSへ換算しない。
- P7-R3：UNKNOWN-DESCENDANT-OPEN01は次の別packetへ返す。旧F3 suite `b5bb9069-90e5-429b-8c5b-97c7f7f22f75` / run `77a4759e` のcontrol2PASS・focused0PASS/1FAIL/9NotRun、known0/GetProcessById(1569)失敗・target93/完了marker欠落は保持し、原因は未確定である。7.6.7への試験後の更新を旧FAILの原因・治癒とは扱わない。

新ENV01/F3/focused10/残9/標準80/variant/製品full/試用提供の実行・retry解除は本packetの対象外である。未知processのgone/Owned救済、CIM欠測の推定補完、ownership/全argv exact/期限/回収/terminal/quality gateの変更、OSインストール・rollback・pwsh修理・global設定・PATH変更・追加権限・実API/実data/admin・新常設frameworkは0とする。製品source/native/fake/V2/通常EXEを変更しない。

### 入力証拠と同定工程

親受領の旧pwsh exe SHAは `BFB46AF89433268872DDB43D1CA7A3F433452EE91ED356A9786940F90118E285`、更新後exe SHAは `FA2C8BB1E35CDD62AD15CD06A1D8F2C5961E9725A9BFA1E7CFD2C322F49E9619`、exe/dll FileVersion=7.6.7.500・ProductVersion=7.6.7である。前turnに一時DLL解決失敗、最後のWinPS読取でDLL存在true。これらは実7.6.7起動・同梱.NET版・安定性・互換性の証明ではない。plannerはruntimeを起動していない。

計画中の親追加受領：現exe SHAは再読取一致、pwsh.dll SHAは `A583A3721FEEBC4F9552A02358BC81BBDD41E2E18BB3F424226F2B1E0BF07DEB` である。runtimeconfigのincludedFrameworksはMicrosoft.NETCore.App/WindowsDesktop.App 10.0.13である（この受領時点の実ロードruntimeは未取得）。親が取得した[公式PowerShell v7.6.7 release](https://github.com/PowerShell/PowerShell/releases/tag/v7.6.7)にはper-user powershell.config.json読取IO/JSON対応、PATH/MSIX周辺、Microsoft.PowerShell.Native更新、.NET SDK10.0.402が含まれる。patch版だけを互換性根拠にせず、designerが既存config snapshot/PATHと.NET10.0.12依存箇所への非干渉を照合する。既取得の一次資料/metadata読取を反復せず、P7-0では不足項目を補う。

P7-0同定結果受領：`P/870de181-c1f9-4bac-8834-12acde439ec5/report.md` は実PSVersion=7.6.7、実.NET FrameworkDescription/Environment.Version=10.0.13、固定process pathを記録した。exe/pwsh.dll/SMA.dll/runtimeconfig/depsの5fileは前後SHA一致、version-only起動1回361ms/exit0、collector8PASS/0FAIL/0SKIP、timeoutなしである。System.Management.Automation.dll SHAは `70701F20AFBBF4E763924C2BDD0C0E9A20EEC35A0E4EFF439E0CCF77F20D6B26` であり、A583…をSMA.dllとした旧計画の誤記を訂正した。これは採取区間の同定証拠であり、WindowsDesktop個別ロード・全経路互換性・恒久安定は未確認である。一次sourceと実.NET10.0.13のconsumer対応はdesigner待ちであり、P7-0全体をClosedにせず、後続Pre/精緻化/旧F3未解除を維持する。

P7-0では親又はrunnerが、固定 `F:/Program Files/PowerShell/7/` のexe、System.Management.Automation.dll、pwsh.runtimeconfig.json、pwsh.deps.jsonを読取り、存在/size/version/SHAとruntime設定を同じsnapshotへ記録する。最初と最後の2回のhash一致を有限な安定性証拠とし、恒久不変を主張しない。公式PowerShell 7.6.7 release/tagと対応runtime sourceを照合する工程を含む。release未発見・local runtimeとの不整合は未確認/阻害として返し、別versionの資料で代替しない。

承認済みversion-only確認が必要な場合は、親が固定exeからprofile無し・noninteractiveの一回に限定し、PSVersion/PSHOME/process実path/.NET FrameworkDescriptionとEnvironment.Versionを取得する。上限15秒、追加child0、profile/import/network/製品/gate入口0。timeout/起動/DLLエラー/再hash差分が出たら停止し、修理や起動再試行へ進まない。実返却runtimeに対応するProcess.SafeHandle/Refresh/StartTimeの公式sourceをdesignerが既存consumerへ対応付ける。configの要求runtimeだけから実ロード版を推定しない。

### 現行消費側の列挙・所有

R/Pは前節の絶対rootである。以下はplannerの直下現行fileに対するtargeted rg結果であり、保存UUID配下のsnapshotは変更対象に含めない。

| 対象 | 現行前提・依存 | 所有/扱い |
| --- | --- | --- |
| `R/Invoke-CargoGate.ps1` | 20説明、1045実PSVersion、1083 Run.pwshVersionの7.6.6 exact | 別実装者。指定version literal/対応説明のみ。predicate構造・例外伝播不変 |
| `R/Test-CargoGate.ps1` | 26説明、742 Run producer、2130実PSVersionの7.6.6 exact | 別実装者。本packetではharness本体としてversion literalのみ。既存80期待とEO/ES属性は不変 |
| `P/Test-ContextPreflight.ps1` | 15説明、113実PSVersion、37–48 Invoke/Test/plan/design binding | 独立作者。version確認と必要bindingのみ。元context12ケース/adapterを保持 |
| `P/CONTEXT-TEST-HANDOFF-P1.md` | 39旧再現説明、72以降snapshot/binding | 独立作者。今回の新snapshot追補だけ。旧RED再現説明を改稿しない |
| `P/Invoke-ArgvPathFocusedSlash.ps1` | 19説明、43–52 Invoke/Test/context/P1/plan/design binding。実version前提はTest原文から取り込む | 別実装者。必要hashと今回runtime説明のみ。case集合/dispatch全文不変 |
| `P/HANDOFF.md` | 40旧実行前提、135–140入力binding | 別実装者。新snapshot/停止境界を追補、現行入口を新組へ結び旧指示を再利用不可と明示 |
| `P/Invoke-FirstFailureEnv01.ps1` | 21説明、120–126 Invoke/Test/plan/design binding | 別実装者。必要hash/説明のみ。既定12入力契約/単一ENV01 dispatchを変更せず実行禁止を保持 |
| `P/Test-FirstFailure.ps1` | 202–203 Initialize-Run先頭50文hash、659–660 Test preflight18文hash、716–725のTest/Slash/KH/Context全体SHA | 独立作者。D4で2内部hash＋4入力bindingの6literalだけを最終組へ更新。18期待/診断本文/期限/抽出・拒否検査は不変 |
| `early-trial-gate-design.md` | 99/147/396/425/450/638に7.6.6、487/727に.NET10.0.12 source根拠 | spec author→designerが順次当該追補のみ所有。版選択は本節を参照し、同定証拠/互換性設計を記録。全履歴置換なし |

`P/Invoke-ArgvPathFocused.ps1`の70にも7.6.6 exactがあるが、旧source/hash/配置を持つ非現行入口であり更新・実行しない。`context-before-E5FE.ps1`、旧CONTEXT-TEST-HANDOFF、FROZEN/suite/run/log/toolchain.json、UUID下のfrozen/current/before copiesも保全する。`Test-ArgvPathEvidence`、`Test-ContextRejectionPredicate`、`Test-KnownHandle`、`Test-FirstFailure`の7.6.6表記は旧検証対象の説明として保持し、新runtime実行時の適用範囲は今回handoffに記録する。ただしFF本文全体を変更禁止とはせず、Preで判明した上表の6literalだけをD4の例外所有とする。その他の実行必須validatorが見つかれば無断で所有を拡大せずplannerへ返す。

開始入力はInvoke318E6667・Test96BBB420・context034F2D04・P1 B1AE65B9・Slash02A011C2・HANDOFF E2D99548・ENVprobe E8B465F9、design9679B02Cである（完全SHAは親の凍結表/現行handoffを参照）。native source381F5E1D/EXE FF7C9A0F、fake3482075B、argv oracle7D74D443は不変として各作業前後に照合する。短縮SHAだけを実行承認入力に使わない。

### 有限単位・依存・受入・必要検証

| 単位 | 依存/規模/担当file | 受入条件とverification |
| --- | --- | --- |
| P7-0 環境同定 | S、source変更0。runner新UUIDの一回用証拠/logのみ | 固定exe/dll/config/depsの前後hash、実version/runtime、公式release/sourceの対応を採取。不一致/欠測を成功にしない。上記15秒一回を超えない |
| P7-A 外部契約/設計/Pre | 0後、S、spec author→designerが既存design追補、planner本節 | 観測可能なversion一致/拒否/無副作用・既存安全不変をspec authorが確定し、designerが実runtime/API影響/実行到達範囲/正確なcommandsを設計。独立Pre PASS後planner精緻化と親受領。現stageだけでruntime再開不可 |
| P7-B 独立有限oracle | A後、S、作者は`P/<新UUID>/`内の一回用script/manifest/証拠 | 下記V01–V08の入力/期待/抽出位置・件数・元statement全文を実装前にfreeze。旧sourceのV01拒否を期待する旧契約証拠は保存し、新契約に対するREDと区別する。adapter失敗は行動REDへ数えない |
| P7-C version局所接合 | B後、M、別実装者Invoke/Testの2本→返却後作者Contextの1本 | 7.6.7 exact producer/validatorに統一し、版literal/対応説明と後段許可binding以外のbytes不変。独立V01–V08、Parser0/PSSA1.25.0診断0/実行error0/exit0。作者oracleを実装者が変更しない |
| P7-D1 context束縛 | C局所Post後、S、作者Context/P1の2本 | 最終plan/design/Invoke/TestをContextへ、次にContext SHAをP1へ。context12の期待/抽出本文不変を独立確認、親が新hash承認。自己hash自己埋込みなし |
| P7-D2 focused束縛 | D1後、S、別実装者Slash/HANDOFFの2本 | 上流最終hash→Slash→HANDOFF→親frozenの有向順で再束縛。case/variant/dispatch/停止契約不変。旧組混入拒否・全入力一致を独立review。実case0 |
| P7-D3 ENVprobe束縛 | C/最終plan/design後、S、別実装者ENVprobeの1本 | 必要Invoke/Test/plan/design hashのみ更新、既定12入力集合/型/順序/前後照合/元例外優先/dispatch全文不変を独立review。ENV01実行0 |
| P7-D4 FF回帰束縛 | D2の最終Slash確定後、D1/Cの最終Context/Invoke/Testを入力。S、独立作者`Test-FirstFailure.ps1`の1本 | 2内部hash＋4全体SHAだけを更新。元50文/18文のversion差分と許可済み最終Test/Slash/KH/Contextを根拠に親承認manifestへ束縛。6literal逆適用後の全bytes一致、18期待/診断/期限/非所有hash不変、Parser/PSSA0、独立binding review。FF18実行はD3を含む最終組が揃ったEだけ |
| P7-E 新runtime互換性/Post | D1–4後、S、独立runner新UUIDログのみ、reviewer read-only | 下記限定局所集合を各一回、firstUnexpected停止。前後hash/実runtime/command/exit/件数/非実行範囲を独立確認。成功でPWSH改訂だけClosed。UNKNOWN-DESCENDANT/F3/80/fullへcredit0 |

P7-C完了時とP7-E完了時をcheckpointとする。P7-Dは3–5file超の一括作業にせずD1→D2→D4、別枝D3を順次所有返却する。D4のFF全体SHAは親の最終manifestへ加え、上流Slash/Contextへ逆埋込みして循環させない。plan/designを変更すると下流bindingが失効するため、Aの精緻化を終えてfreezeしてからDへ進む。後の計画変更は影響するbindingだけを再reviewする。親のcurrent/review/runner/usage記録は子所有に含めず、既存文書へ接合する。

FF Pre指摘の根拠は、705のAssert-SourcePreserved→725の4旧全体SHA拒否→729のAssert-ProbeStaticが18loop743より前にあることである。旧Test D8AD/Slash72244/KH0C82/ContextF46Fと旧2内部hashでは新組のFF18へ到達しない。D4はこの6期待hashだけを更新し、実行時の現物hash自動採番、hash検査削除、別adapter/framework、18行動期待の追随変更を禁止する。最終SHAはまだ未確定であり、親が実装後の許可差分と逆bytesを照合して承認する。

有限version oracleはV01=実版7.6.7＋Run7.6.7 accept、V02=実版7.6.7＋Run7.6.6 reject、V03=実版7.6.6＋Run7.6.7 reject、V04=実版7.6.8＋Run7.6.7 reject、V05=実版7.6.7＋Run7.6.8 reject、V06=Run版欠落/null reject、V07=Run版`7.6.7 ` reject、V08=producerのpwshVersionが厳密にstring `7.6.7`である、の8安定ID・16観測とする。実版3入口×V01/03/04の9、Run側V01/02/05/06欠落/06null/07の6、producerの1である。各ASTは親scope/型/一意件数/offset/元全文を固定し、V01は原文実行、V03/04だけPSVersion.ToString()の単一ASTをObservedVersion局所引数へ可逆置換する。逆置換全文Ordinal一致を必須とし、自動変数上書き・別predicate再実装は0である。Run側は元Assert-Field→version型foreach→版Ifを順次抽出し、欠落とnullの拒否位置を分ける。各後続sentinelは正常1/throw時0を確認し無関係例外を成功拒否にしない。V08はNew-CaseFixtureの元valueだけを評価し、toolchain観測producerと混同しない。各5秒/全60秒の前後検査、main/CIM/child0、firstUnexpected停止であり、hard watchdogとは呼ばない。詳細はdesignの「独立作者用の有限抽出契約」を正とする。

P7-Eの順序はV01–V08の16観測→既存Context12→MI MissingOs2→成功時MissingPure20→D4後のFF18→EO6→現行Slash/ENVprobeのValidateOnly各1回である。designの「P7-Eの既存commandと到達範囲」の8本を一括実行せず各結果後に次へ進む。version oracleは`P/<新UUID>/Test-PwshVersionContract.ps1`、既存各interface/引数はdesign記載を維持する。最終SHAと変数展開済み完全argvはsource確定後に親manifestへfreezeし、それまで実行承認を作らない。V/Context/MI/FF/EOのOutputRootはP直下の別UUID、ValidateOnly2本はR/verification直下の別UUIDであり作成しない。

MIの実子上限1個・10秒自終了/全15秒finally込み、Pure各5秒/全60秒、Context各15秒/全120秒、FF各5秒/全60秒を保持する。EO6に存在しないhard deadlineを宣言しない。Contextは自身のStartTime/git読取を許し登録/Start/Stop/CIMはtrap、MI Osは実CIM登録1回と別の合成欠測rows、FFはprivate診断/JSON/atomic保存と合成dispatchまででmain/process/CIM/cleanup0である。ValidateOnlyはpreflight/config4path/tool/self読取とgit check-ignoreまで、初回write/import/rustc/fixture/dispatch前にreturnする。Run版拒否前の既存JSON/immutable台帳/CIM/期限読取や拒否後の診断terminal I/Oは保全し、S02をすべてのI/Oゼロとは解釈しない。未解決原因を再現させるOS刺激は追加しない。

designerの.NET10.0.13公式Process.cs/Process.Windows.csの指定spanと10.0.12対応箇所の同形確認を受領した。StartTime cache、Refresh後のhandle継続、SafeHandle再利用、GetProcessTimesのthrowIfExited=false/借用ownsHandle=falseを既存Add-KnownProcess/Read-OwnedTreeへ結ぶsource根拠である。全file/binary完全一致の証明ではなく、実型はEのMIで検証する。HTTP全file比較のDNS失敗/空値は証拠に採用しない。Cargo config4pathとpowershell.config.jsonは異なるconsumerであり、NoProfileからPowerShell設定非読取を推定しない。

既成功のnative argv/opaque UTF-16/期限/unknown非停止等のsource契約は変更span外のbytes/ASTと独立Postの対応が一致する範囲だけ再利用する。旧実OS/Cargo/native→pwsh/Start-Process PATH接合のPASSは7.6.6限定として残す。P7-Eの局所成功だけで7.6.7の全経路互換性を宣言しない。実native/Cargo接合はUNKNOWN-DESCENDANT packet後の既定gate再開時の未了条件である。全体test/releaseは製品提供時の既存条件を維持し、本docs/harness改訂で無関係な製品buildを反復しない。

PSSAは既存ErrorActionPreference=Stop、module1.25.0、全ファイル通常rule、診断配列/件数、最終`PSSA_COMPLETE count=0`、実exit0を記録する。変更ps1/一回用oracleを対象とし、rule除外・mock結果・旧logによる代替なし。独立局所実行は親が承認した最終hashと新OutputRootに束縛し、現物hash採番だけで承認を作らない。

### 各単位のexactモデル/effort・難度/リスク

以下の割当はP7-0/A/B/C/D1/D2/D3/D4/Eの各単位の全7責任へ適用する。表の担当工程以外は非該当として起動せず、役の統合で独立性を省略しない。全値は今回呼出可能一覧のexact ID/対応effortであり、親model/effortを変更しない。

| 役 | model / effort | 工程・理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 全単位の依存/所有/精緻化、固定役。中難度/高risk：環境改訂と旧障害救済の混同を防ぐ |
| external-spec author | `gpt-6-astra` / `high` | Aで限定外部契約、他単位は変更要求時の差戻し。固定役、中/高：利用者の版選択と安全契約の境界を定義 |
| designer | `gpt-6-astra` / `high` | 0/Aのruntime/source照合、B–E設計逸脱判断。高/高：Process handle/APIとAST consumerの成立性が未検証 |
| implementer | `gpt-6.1-sol` / `medium` | Cの2本、D2の2本、D3の1本のみ。低〜中/中：確定literal/spanの限定編集を逆bytes/独立oracleで検査可能 |
| test author | `gpt-6-astra` / `high` | Bの独立8ID/16観測、C/D1のContext/P1、D4のFF6literal。高/高：実predicate抽出と版/束縛不整合を誤PASSにしない独立期待が必要 |
| test runner | `gpt-6.1-sol` / `medium` | 0の同定、C/Dの局所確認、E独立実行。中/高：有限commandとfirstUnexpected/境界を忠実に記録し、原因推論/期待編集をしない |
| reviewer | `gpt-6-astra` / `high` | A Pre、C局所Post、D binding review、E独立Post。高/高：未知所有/旧credit救済、循環binding、未実行の互換性宣言を防ぐ |

0/Aは環境/source対応のため高risk、B/Cは多重version判定の抜けで高risk、D1–4は古い承認と新sourceの混在で中〜高risk、Eは実runtime/保持handleを含み高riskである。D4は作者`gpt-6-astra/high`が独立期待hashを所有し、reviewer`gpt-6-astra/high`が逆bytes/最終bindingを検査し、runner`gpt-6.1-sol/medium`がEでFF18を実行する。D4に実装者のtest編集を割り当てない。通常補助1役・fresh context・親dispatch、再委任なし。起動指定とprovider返却metadataは区別し、未取得を推定しない。利用不能model/effort/skillはblockerであり黙って置換しない。skillは既存7役の担当範囲とPS編集/監査に必要なpowershell-automation/powershell-reviewのみを適用し、新skillや常設仕組みを導入しない。

### task-list・停止・次handoff

- [x] P7-plan0：承認済みversion選択、現行consumer/binding、旧障害との分離を仮計画へ記録した。
- [ ] P7-0：固定環境のread-only/有限version-only同定と一次資料対応を受領する。
- [ ] P7-A：external-spec→design→独立Pre、planner精緻化と親受領を完了する。
- [ ] P7-B/C：作者8ID/16観測凍結→別実装者version接合→独立局所検査/Postを完了する。
- [ ] P7-D1/D2/D3/D4：所有を順次返却し、FF6literalを含む必要bindingの独立reviewを完了する。
- [ ] P7-E：限定新runtime集合/ValidateOnly/独立Postを完了し、未了の実経路互換性を明記する。
- [ ] 次packet：保存F3証拠を入力にUNKNOWN-DESCENDANT-OPEN01の因果診断を別計画化する。本checkboxは原因修正/再試行承認ではない。

一次source/consumer対応、Vの可逆抽出方法、局所command到達範囲はdesignerから受領済みである。残る未決はFF6literalのdesigner追補と独立再Pre、未実装の最終source/hash/完全argv、局所実行の実証である。実runtime版と採取区間の前後hash一致は上記P7-0結果で取得済み、再取得しない。版選択やD4の技術的所有是正の利用者再承認は不要である。別の版・修理・権限・安全判定変更が必要になった場合だけ具体的不足を親へ返す。予期外FAIL/欠測/hash変化は停止し、同じ試験の自動retryや失敗skipを行わない。

次handoffは親→designer `gpt-6-astra/high`の既存design FF箇所限定追補→reviewer `gpt-6-astra/high`の独立再Preである。PASSと本計画の親受領後に作者`gpt-6-astra/high`のBへ進み、B/C/D1–4/Eの順序と独立性を保持する。Preの未解決指摘があればplannerへ戻し実装へ飛ばない。rollbackは今回source差分の取消しと証拠保全だけを親が扱い、OSを7.6.6へ戻さない。旧sourceを戻してもruntime不一致の停止状態であり実行許可ではない。

本plannerの完了境界は本追補の保存・diff/参照/所有/hash確認までである。開始共有週枠85%は親受領値、子`/root/pwsh767_plan`の終了週枠/開始終了token/provider返却model・effort/包含関係は未取得。親が既存usageへ接合し、専有量/改善率を推定しない。plannerのcode/test/runtime/再委任/commit/公開は0である。
