# ET3-G：一時Cargo runnerによる実行前照合（Pre-2提出版）

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
