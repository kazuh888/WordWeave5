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

## KNOWN-HANDLE01 PacketBrief（2026-10-16、暫定計画・設計前）

対応findingは親の既存review matrixの`KH-REOPEN-01`（Accepted、独立P1確認）である。KNOWN-HANDLE01は本局所計画名、KH-A–Dは作業単位であり、findingの複製やClosed判定を作らない。

対象はCXT-Eで観測した、既登録processを再列挙した際に保持handleへ到達する前のPID再openが失敗する局所競合である。完了境界は最小是正の独立回帰・固定snapshotのfocused10件・独立Postを経て標準80件への再開handoffを可能にすること。標準80/87variant、製品full、ET3提供の完了は含めない。本plannerの今回の完了は本追補と文書検査・親返却だけであり、設計/Pre受領後に本節を限定精緻化する。

### 証拠と未検証の境界

- 親受領の独立診断P1は、Invoke `12E7FC95F24883AC89E8464108AC7404D4953006EBFA56DC9B7F8D0613281621` のRead-OwnedTreeで1270のGetProcessById→1272のGet-Identity→1274のKnown.ContainsKeyの順であることを確定した。plannerも当該分岐とAdd-KnownProcessのpinnedHandle保持をread-only確認した。追加OS測定は設計着手条件ではない。
- 保存REDはsuite `R/verification/68cfe879-a4ea-4665-83f0-8bff89cd2b2b`、run `94e5f1c6-43bb-4ca6-98c7-80eb443f76fa`。control2PASS、ENV01FAIL/残9NotRun、sourceMatched=true。RunOwner21556のUnprovable PID23100は再open時not running、rowFilter未到達。同terminalに保持済Owned PID23100（parent14632、start `2026-10-15T20:38:03.9529175Z`、exit `20:38:04.9864708Z`）があり、元CIM creationは`20:38:03.9529170Z`。knownMatch1/snapshotMatch1/path・command存在は補助観測であり、これだけから同世代/所有を断定しない。
- 最初の公開cancelはSupervisor `20:38:05.0399582Z`。RunOwner cancel.tmp `20:38:05.0425184Z` は未公開である。SafetyWorker result.tmpにもPID6472/knownMatch1の同分岐失敗があるが、正式terminal成功へ加算しない。PID23100が全process時系列で最初の原因だったというclaimは置かない。
- CXT-D test94E4928C/driver68EF44DDの12PASS・独立Post PASSとPTH-MONO01 Closedは保持する。今回はPTHのpath-null fallback/UTC精度/mono検証より前の別branchであり、そのclosureを撤回しない。旧Unprovableを後からOwnedへ変更せず、保存RED、tmp、cancel、旧入力/結果/承認は上書きしない。現行CXT-E/ARGV受入は未達のままである。

### 要求・候補・不成立条件

| 要求 | 暫定の受入境界 |
| --- | --- |
| KH1 同世代の既知process | 元snapshotの対象行と台帳identity・有効保持handleが同じ世代であることを再open前に証明できる場合のみ、その既存台帳を利用する。PIDだけのskip、GetProcessById例外の無条件成功化、最新snapshotへの取り直しによる救済は不可 |
| KH2 所有/終了の証拠 | 既存Knownの所有根拠とhandle寿命を保持し、新しい所有権・終了事実を合成しない。既知・同世代でsnapshot後に終了しても、保持handleで既存終了観測へ進める。未知・証明不能は従来のUnprovable/非停止/非PASS、既存fault・kill/wait/terminal条件を維持する |
| KH3 境界の不変 | 新規候補は従来のidentity/parent lifetime検証と登録へ進む。PID再利用、複数/不正CIM行、部分identity、閉鎖/無効handleを安全な既知一致と誤認しない。generic argv exact、承認、UTC/mono期限、未知値nullとfixture保持は不変 |

優先候補はRead-OwnedTree内で既知候補を先に限定照合し、既知同世代の保持handleを借用して既存identity検証へ接続し、新規候補だけをPIDでopenする構成である。ただし単にKnown検索を前へ移すだけでは不十分である。designerは一意な元行、PID、CreationDateの既存UTC6桁比較、保持handleの7桁開始時刻と台帳key、parent/path/commandの照合、pinnedHandleの生存と借用所有、終了後Refresh/StartTime読取の成立性を具体化する。6桁一致を新しい時刻許容差へ変更しない。同PIDに複数世代の台帳候補がある場合の選択/拒否を明示する。

Add-KnownProcessはDirectLaunchHandle由来のpath/parent/commandがnullの暫定identityも保持する。完全identity同士の照合とこの部分identityを混同しない。今回は部分identityの補完・新所有の昇格を追加せず、既存direct-launch証拠処理へ委ねるか既知skip対象外とする。どちらが現行状態遷移を保つかは設計/Preで確定する。既知照合の失敗を別processの再open成功で隠さず、PID再利用の場合は新規世代を既存通常経路で検証するか拒否する既存安全境界を保つ。

代替候補は、既存Get-Identityへ保持Processと元Rowsを渡して完全identityを照合する方式と、既存登録証拠に限定した小private照合をRead-OwnedTreeへ置く方式である。前者で終了済handleの既存consumerが成立するなら優先し、成立しなければ失敗APIと不足証拠を特定して後者をPreで評価する。一般化した新identity framework、全監督の再設計、GetProcessById retry、CIM追加照会、Job/Admin/OS設定による回避は採らない。不成立でも未知を成功にする方向へ縮退しない。

### 作業単位・所有・依存・検証

作業リストの正本は本節である。既存未完了plan/todoは移動・完了扱いにしない。通常1 helperずつ、作者/実装/runner/reviewerは別担当、共有fileは逐次所有とする。

| 単位 | 依存・規模・所有 | 受入と必要検証 |
| --- | --- | --- |
| KH-A 設計/Pre | 本計画の親受領と独立P1。S、designerは既存early-trial-gate-design.mdの局所追補だけ | KH1–3を状態/入出力/失敗/handle所有へ具体化し、独立Pre PASS。designerは部分identity・終了済handle・候補一意性と局所試験の固定件数/時間/commandを確定。plannerが外部契約再利用可否と実装境界を限定確定する。ここまで実装/runtime不可 |
| KH-B 独立回帰→局所実装 | Aとplanner精緻化後。S、作者はR/verification/argv-path01内の局所回帰だけ、別実装者はR/Invoke-CargoGate.ps1のRead-OwnedTreeと必要最小private照合だけ | 保存旧Invokeで競合RED、新Invokeで同一oracle GREEN。実装者は作者期待値を編集しない。Parser::ParseFile/PSSA1.25.0、diff/hash/非所有不変を確認。KH1–3の固定対照がPASSしなければ下流解除不可 |
| KH-C 束縛更新・接合Pre | A/B確定hash後。S、作者は既存Test-ContextPreflight.ps1の入力束縛と同既存handoffの新snapshot説明だけ、その後別実装者はInvoke-ArgvPathFocusedSlash.ps1の固定入力とHANDOFF.mdだけ | 下記依存鎖の旧/new SHA対応、旧bytes保持、12期待/禁止22/10case/17args/firstFail/期限の意味的不変を独立review。入力hash不一致を無視する実行は不可。元context consumerに意味的変更がないことを確認し、単なるplan/hash変更を理由に旧12件を全再実行しない |
| KH-D 独立実測/Post | B/Cの静的接合PASSと親dispatch。S、runnerは新規隔離証拠/log、reviewer read-only、親が既存runner/review/current/usageへ記録 | 最小正常scenarioを早期確認する下記固定順で局所OS証拠→既定focused10件1回、firstFail/残NotRun/retry0。新snapshotの正常ENV01、実terminal/cleanup、AP1–4を独立確認しPost PASS後だけ標準80へ返す。別失敗は別findingとして停止し、本局所是正を理由にfullを解除しない |

- [ ] KH-A：同世代Known再利用の限定設計・Preを受領し、本暫定計画を精緻化する。
- [ ] KH-B：独立作者が期待/REDを固定し、別実装者の局所修正とGREENを照合する。
- [ ] KH-C：新hash組への最小再束縛・旧snapshot保全・独立接合reviewを完了する。
- [ ] KH-D：独立OS対照/既定focused/Postを完了し、未完了標準80へ返す。

局所回帰は既存AST抽出/合成fixtureを利用し、新public fault APIや新frameworkを作らない。必要最小の分類は、(1)正常live Known同世代、(2)元CIM採取後に同じ保持handleのprocessが終了、(3)同PIDだが別世代、(4)閉鎖/無効保持handle、(5)未登録processの再open前消失、(6)行/候補が一意でない・必要identity欠測、(7)通常の新規子登録、である。作者は3–6をpure/局所代替で固定し、実PID再利用を待つ試験を作らない。保存REDは消さず、再現率に依存する再実行をREDの条件にしない。旧実装がどの境界で失敗するかを同じoracleへ結び付ける。

OS-bound最小確認は隔離した自分の合成子の実保持handle・実CIMを使い、liveでの既知照合→snapshot後の終了→同じ保持handleでの終了観測を有限順序で行う。pureのHasExited代用だけで(2)を実証済みにしない。正常liveの最小確認を初めに置き、ここが失敗したら後続を起動しない。未知sentinelを本体に停止させず、必要な合成子回収は作者/runner自身の既存所有手順だけである。固定件数/上限と具体commandはKH-A/作者freezeで決めるが、既存gateの10秒cleanupや期限を延長する権限ではない。

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

未決は(1)終了済保持Processで既存Get-Identityをそのまま使用できる条件、(2)部分identity/複数Known候補の限定分岐、(3)最小回帰の具体件数・時間・commandである。いずれもdesigner/作者と独立Preで解消する技術事項であり、利用者へ選択を丸投げしない。追加権限、既存比較/期限の緩和、別file所有拡大が必要になった場合だけ根拠付きで親へ停止返却する。最小正常確認が別境界で失敗した場合、無限にharness整備へ広げず保存証拠と新findingを返す。

次handoffは親受領後のww-designer `gpt-6-astra/high`、所有は既存designのKNOWN-HANDLE01追補だけである。入力は本節、新独立P1記録、現Invoke12E7と保存RED。外部仕様新規変更は現時点不要、追加OS診断も不要である。設計→独立Pre→planner精緻化まで新実装/試験起動は禁止する。

今回の開始共有週枠は親受領40%、途中の親受領review終了値41%でありplanner単独の終了測定ではない。子`/root/known_handle_race_plan`の開始/終了token、固有session ID、provider返却model/effort、親子包含、終了週枠は未取得。使用差分/改善率は推定しない。本plannerは文書追補・diff/hash/相対link/所有検査のみ、実装/試験起動/再委譲/公開0である。
