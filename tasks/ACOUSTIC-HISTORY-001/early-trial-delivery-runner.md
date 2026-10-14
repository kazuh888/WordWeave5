# ET3 試用EXE検証 runner

2026-10-14。担当 `ww-test-runner`、planner割当 `gpt-6.1-sol / medium`（early-trial-plan.md ET3）。実行providerの返却model/effortは未取得である。親 `/root`、子 `/root/trial_delivery_runner`。AGENTS、development-team、KPI、wordweave-change全文を読み、skillは検証・結果報告だけへ適用した。

## 今回の境界

親の最初の許可は通常/試用all-targetsと関連crateの **no-runのみ** である。ET-01–07、ET2-G0、最終V2/ET2 Closedを入力とする。新artifact対応を親へ返すまではEXE/list/runtimeを実行しない。旧ET2解除を流用しない。release/native/実API/実資格情報load-save/実マイク/管理者/OS設定変更/再委譲/公開は0である。source、tests、期待値、設定を編集しない。

## 固定対象と開始

HEAD `d446c5dd32177815ee322a39cf51ab84a99ee458`、開始git statusはclean。通常targetは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/default`、試用targetは同 `/trial`、ログは同 `/et3`、deliveryは `target/early-trial-delivery`。4pathは `git check-ignore` 成功である。CARGO_TARGET_DIRは各build process内でのみ指定する。

旧 `target/release/wordweave5.exe` SHA256 `9B888E9FC923823C59B4C3B0AB1A4DAD2051CA6139DE153B6DE6CA213DD387D3` を着手時に確認した。上書き対象にしない。

開始20:10頃（JST）。cargo `1.98.1 (797e8a9bc 2026-08-05)`、rustc `1.98.1 (48a229cea 2026-09-01)`。repo `.cargo` は不存在。user Cargo configの有無/hashは `target/early-trial-build/et3/cargo-config-before.log`。tracked src/tests/tools/Cargoの全hashは `source-before.json`、8sourceは最終V2固定表に一致、作者4markerは `markers-before.json` でV2に一致した。examplesは途中追加snapshotで識別し、着手時source一覧と区別する。

週枠開始58%、window10080分、reset1792496607。子sessionのtoken counter/log session ID、親子包含、返却model/effortは未取得。週枠共有値から本工程消費率を推定しない。

## コマンドと状態

逐次compileであり、テスト実行数PASS0/FAIL0/SKIP0（全未実行）である。no-run成功は行動PASSに加算しない。

| 構成 | コマンド | 終了/状態 | 正本ログ（et3配下） |
| --- | --- | --- | --- |
| 通常 | `cargo test --all-targets --locked --no-run --message-format=json` | **exit101 / FAIL**、build-finished.success=false | `default.jsonl` / `default.stderr.log` / `default.exit.log` |
| 試用 | 同コマンド＋`--features early-trial` | 未実施（通常compile blockerで停止） | なし |
| acoustic-analysis | 計画のall-targets＋no-run | 未実施（同停止） | なし |
| qwen-audio全target/lib no-default | 計画の各コマンド＋no-run | 未実施（同停止） | なし |

途中診断のPowerShell brace形式pathはparser errorでコマンド実行前に停止、修正して再読した。書込中JSON末尾を解析した診断は未完行で失敗したためbuild結果へ数えず、終了後の全JSONを使う。CIM process照会はsandboxアクセス拒否であり、終了確認の証拠に使わない。これらをbuild/test FAILへ混同しない。

## 次handoff

親の新G0解除前はlist/runtime0を維持する。通常/試用と関連crate no-runの終了code、前後source/Cargo/config不変、JSON由来全testartifactの絶対path/SHA/profile/featuresを返す。解除後の全体test/release/fmt/diff/関連gate、合成native直接起動・学習/設定保存・再起動・既存音読導線、独立Postとlocal提供は未了である。実IME/pen/TTS/mic/Codex/Qwen認証等は別のnative受入であり、旧ログやmock成功で代替しない。

## 停止結果：ET3 compile blocker

通常no-runのcompiler errorは1件（E0433）、warningは19件である。`tests/codex_process.rs:16–18` はWindowsで `../src/ai.rs` をintegration crateの `mod ai` として読む。一方 `src/ai.rs:163` の作者test moduleが `crate::media::early_trial_startup_tests::isolated` を参照するため、mediaを持たないintegration crateでcompile不能となった。これはcompiler診断と現物のimportから確認した原因である。bin-onlyの旧ET2成功をall-targets成功へ拡張できない。

no-run command集計はPASS0/FAIL1、行動testはPASS0/FAIL0/SKIP0（全未実行）。FAILED command後の部分artifactは23件（profile.test=true 21件）、全path/SHA/profile/featuresは `target/early-trial-build/et3/default-artifacts.json`。部分成果物をGate解除候補・合格提供物にしない。通常wordweave5 test-binは `default/debug/deps/wordweave5-72e8adffa8f9ef2c.exe`、SHA `CC752E3D2C17265AB6784CF5AB3F9760920E85F5A291C186808BAEAB9F9A93DF`、features=[]、profile.test=true。診断正本は `default-errors.json` と上表rawログである。

`source-after-default.json` は開始snapshotと全path/hash一致（差0）、したがって8source/4marker不変である。user Cargo configの前後log差0、source/tests/tools/examples/Cargo設定のgit diffは空。途中examples snapshot追加の元一覧との差も0。旧release EXEの終了側SHAも保全値一致。親所有current/usageの文書dirtyは保持した。全Cargo commandはexit済み、runnerによるEXE/list/runtime/release起動0、source/test/設定変更0である。

終了観測20:15:29 JST、週枠終了59%（window10080、reset1792496608）。開始58%→終了59%は共有accountの観測であり本工程の消費率ではない。子token counter/log session ID/親子包含/返却model・effortは未取得を維持する。最終応答前までの観測である。

親へcompile blockerを報告して所有を返却する。作者/実装者の是正と新source G0後に、入力が変わった範囲を再compileする。runnerは修正・assert変更・skip追加・再試行・品質gate変更を行わない。

## ET3-FX1 no-run 再開

親の限定sourceG0（early-trial-delivery-review.md末尾）を受領し、ET3-FX1計画/設計と作者handoffを確認した。指定Sol/mediumは維持、返却metadataは未取得である。旧FAILログを保持し、新ログは `et3/fx1-*` のみとする。許可は通常/試用all-targets no-runだけであり、関連crate/release/list/runtime/nativeはまだ行わない。

HEADは同じ `d446c5dd32177815ee322a39cf51ab84a99ee458`＋作者のai/media/newhelper差分である。`fx1-tested-tracked.diff` と新helper raw hash/logで識別する。作者12項目snapshotを現物照合し差0、全tracked src/tests/tools/examples/Cargoと新helperは `fx1-source-before.json`、12項目は `fx1-g0-before.json`、4markerは `fx1-markers-before.json` に固定した。ai markerは新 `190D951624F470071F5E013591A59B476A23F451FF6C984BCF5D2F6DE390F407`、他3markerはV2一致である。helperの同体性と実遮断/global/guard寿命不変は親のG0と独立静的reviewを受入入力とし、runnerがsourceを書き換えない。

開始週枠62%（10080分/reset1792496608）、start時刻/statusは `fx1-start.json`。user Cargo config前snapshotは `fx1-cargo-config-before.log`。counter/session/返却model/effortの欠測は前節どおりである。通常・試用は既定の別targetを逐次利用する。

### FX1 通常 no-run 成功

`cargo test --all-targets --locked --no-run --message-format=json` はexit0、JSON build-finished.success=true。`fx1-default.{jsonl,stderr.log,exit.log}` を正本とする。全tracked+newhelperの前後hash差0（`fx1-source-after-default.json`）。全executable artifactのpath/SHA/profile/featuresは `fx1-default-artifacts.json`。通常test-bin `default/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` SHA `CDE160AEB81AE335379CE624E9EC942A28FDD21766823685F528DF5A6FE508F5`、integration `default/debug/deps/codex_process-b80f5fbdc29c53b8.exe` SHA `54075C6EEDA79877A75687EEE6BEDDF7BF5D819CE261A1C0C4A0D12C746B7E14`、両features=[]/profile.test=trueである。

旧E0433のcompile回帰は通常構成で解消した。試用はprocess限定 `CARGO_BUILD_JOBS=2`（資源抑制の親許可）と既定trial targetで逐次開始した。永続設定・source・検証範囲は不変、EXE/list/runtimeは0のままである。

### FX1 両構成 no-run 完了・新artifact対応返却

試用の `cargo test --all-targets --locked --no-run --message-format=json --features early-trial` もexit0、build-finished.success=trueである。通常20:44:23／試用20:48:25 JSTに終了。通常/試用とも全executable artifact24件、うちprofile.test=true 22件。compiler errorは各0、warningは通常49/試用20（JSON compiler-message数）であった。build PASS2/FAIL0、行動test PASS0/FAIL0/SKIP0（未実行）である。

全artifact一覧は `fx1-default-artifacts.json` / `fx1-trial-artifacts.json`、集計は `fx1-no-run-summary.json`。以下は親の構成別G0照合用であり、list/runtime解除を自動的に意味しない。EXE pathは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/` からの相対pathである。

| 構成/target | EXE | SHA256 |
| --- | --- | --- |
| 通常bin | `default/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `CDE160AEB81AE335379CE624E9EC942A28FDD21766823685F528DF5A6FE508F5` |
| 通常codex_process | `default/debug/deps/codex_process-b80f5fbdc29c53b8.exe` | `54075C6EEDA79877A75687EEE6BEDDF7BF5D819CE261A1C0C4A0D12C746B7E14` |
| 試用bin | `trial/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `3014F3091436122D86DDC3543D5F842D853AD5CC779B651A4D9A145EACCBD107` |
| 試用codex_process | `trial/debug/deps/codex_process-4e6a5997dab868a1.exe` | `C70CE963CAB94990CD48599366CFAA3F71B0C41E3F1EE7642DED190579B0F625` |

上表全profileはopt_level=0/debuginfo=2/debug_assertions=true/overflow_checks=true/test=true、featuresは通常[]、試用[early-trial]。両libの対応も全artifact JSONへ記録した。全JSONは終了後に全行解析し、書込中断片を成功判定へ使っていない。

`fx1-source-after-default.json` / `fx1-source-after-trial.json` は全tracked source/Cargo/examples＋新helperの開始snapshotと差0、12項目G0と作者4markerも不変。`fx1-markers-after.json` と `fx1-cargo-config-after.log` は各beforeと差0。旧release SHAは終了側も保全値一致。HEAD/dirtyの最終正本は `fx1-end.json`、対象はd446c5d＋FX1の限定差分のままである。親所有の文書dirtyを保持した。

予定集合は通常bin共有helper23／試用bin33、integration各11（作者handoffのsource集合）であり、EXE listはまだ取得していない。親の新対応解除後にlist→構成別bin録音3/P01既定安全順20→integration自身A01–A11→従来ET2/関連回帰へ進む。全体test/release/関連crate/native/独立Post/提供は未了である。

今回の全Cargo command/session44109・29269はexit済み、EXE/list/runtime0、source/期待値/設定変更0。終了20:49:36 JST（最終応答前）、週枠62→63%/window10080/reset1792496608は共有観測であり、工程単独消費へ換算しない。token counter/session log ID/provider返却model・effort/親子包含は未取得のままである。

### FX1 新EXE限定runtime：84件 PASS、独立Post待ち

親が `early-trial-delivery-review.md` の新EXE Gateで上表4EXEだけのlist/限定回帰を解除した。旧解除やbin解除をintegrationへ流用していない。開始週枠63%。予定実名は実行前に `fx1-default-selected.log`（26）、`fx1-trial-selected.log`（36）、`fx1-integration-selected.log`（各11）へ固定した。旧final-selectedは変更していない名前と安全順の入力だけに使い、旧integrationのlist成功baselineとは扱わない。

4listは全exit0。新bin listは通常348/試用358、integrationは各12（うちP01 A群11）。予定実名欠落0であり、新listから抽出した共有helper consumer集合と予定集合は完全一致した。正本は `fx1-list-summary.json`、`fx1-{構成}-{target}-listed.log` と `fx1-runtime/*-list/{stdout.log,stderr.log,result.json}`。

| 新EXE | 選択/成功実行数 | 共有helper利用/marker適用 | 非isolated |
| --- | --- | --- | --- |
| 通常bin | 26/26 | 録音3＋P01 20＝23 | title1＋namespace2 |
| 通常codex_process | 11/11 | A01–A11＝11 | 0 |
| 試用bin | 36/36 | 上記23＋main creator1/app9＝33 | title1＋namespace2 |
| 試用codex_process | 11/11 | A01–A11＝11 | 0 |

**計84/84実行PASS、FAIL0/SKIP0**。同名を構成/EXE別に検証した実行数であり84 unique test名とは主張しない。4listを合わせ88起動、全exit0/Admin=False/SHA一致/Hidden/有限timeout/Waited=true/fixture残存0。raw stdoutのrunning1/1passedを全84で再確認し、共有helper利用78件だけ原stderrの `ET2_CASE_COMPLETE:<exact>:Administrator=False` を確認した。非isolated6件へ内部markerを要求しない。0件やfixture SKIPをPASSへ算入していない。

構成ごとにbin録音3→P01 A01–A06/B01/C01→A08–A11/B02–B03/C02→A07/D04→D01–D03→既存ET2残りの順であり、各integration自身のA群11も自身の解除済みEXEで検証した。helper consumer23/33の成功はこの実行結果を接合したものであり、別枠で同じcaseを再実行していない。未変更の関連94件や旧感度25件を無用に反復していない。

実行は既存 `target/early-trial-build/et2-egress-runner/Invoke-V2Case.ps1`（SHA `572E56BE23FDD59B949A678BAEACAFE4E035434D064B9B4D28425D781AFF1168`）を無変更で再利用した。commandは `pwsh -NoProfile -File <wrapper> -Executable <固定絶対path> -ExpectedSha256 <固定SHA> -ExactName <exact> -OutputDirectory <新logdir>`、helper利用caseだけRequireChildMarker=true（非利用false）。listはExactName `::`/List=true。wrapperは毎起動非管理者/SHA確認、合成LOCALAPPDATA/TEMP/TMP/cwd、Hidden、外側65秒/内部45秒、必要時kill+wait、子回収後cleanupを保持する。既知sandbox OS5を繰返さず、親が許可したrequire_escalatedの通常ユーザーで実行した。管理者・UAC・OS設定変更は行っていない。各EXE起動前に全source/Cargo/helper hashを照合した。

集計正本は `fx1-runtime-summary.json`、全実名/command対象/時刻/exit/hash/権限は `fx1-runtime-results.json` / `.csv`、個別rawは `fx1-runtime/{default,trial}-{wordweave5,codex_process}-caseN/`。外側合成treeは証拠として保持し、ユーザーデータ削除0。終了20:56:25 JSTの `fx1-source-after-runtime.json` は開始全source/Cargo/helperと差0（したがって4marker不変）、`fx1-exe-after-runtime.json` は4EXE SHA不変。旧releaseも保全値不変。read-only process照会は `fx1-process-after-runtime.json` に対象4EXE同名process残存0を保存した。プロセス停止0。runtime sessions11298/41416と全commandはexit0で終了した。

`git diff --check` はexit0（runtime中の文書追加を含む確認）。同じsource snapshotに対する作者の `cargo fmt --all -- --check` exit0証拠を保持し、入力不変のため反復していない。source/tests/期待値/settings/追加harness編集0、変異0。終了週枠63%（10080分/reset1792496607、最終応答前）。counter/log session ID/返却model・effort/親子包含は未取得であり、指定Sol/mediumと混同しない。

FX1の新動的証拠を親へ返し、独立Post判定を待つ。本84実行はET3 all-targets runtime、関連crate、release、native/実音声/実認証、提供完了を意味しない。これらはまだ未解除・未実施である。

## 既定ET3 full gate：環境境界で再compileを検出し停止

FX1独立Post PASS/Closedと親の全24artifact照合後、保全commit `cb882031385796eb35c3049856371cebe5be084f`（source bytesはFX1と同じ、着手clean）で通常 `cargo test --all-targets --locked` を開始した。`full-start.json` と `full-source-before.json` が入力正本。旧main SHAは既定保全値、旧standalone `target/qwen-audio/release/qwen-audio.exe` SHAは `57A3BE5B10F117E146E1C8AAC77616AA4A0F0873D74D68D858BD8C6E9A64506E` と固定した。

外側の既存Start-Process型監督を使い、非管理者/Hidden、合成 `full-default-runtime/{local,temp}`、process限定CARGO_TARGET_DIR=既定default、JOBS=2、600秒上限/必要時kill+waitで開始した。Cargo自身のcwdは親が了承したrepo rootであり、Cargo既定package cwdとexact gateを維持した。個別isolatedの合成child cwdや後段の任意cwd native受入とは別である。

Cargoが依存を含む再compileに入り、新EXE対応前にruntimeへ進む可能性を親へ報告した。親の明示停止指示後、今回wrapper PID22180のcommandに `full-default-runtime` が含まれ、Cargo PID13248の実command/開始時刻/親PIDが一致することをCIMで再確認した。PID13248→実Cargo28588→今回のrustc/link等の所有treeだけをKill(true)/Waitした。`stop-owned-tree-before.json` / `stop-result.json` に照合と回収を記録し、所有tree残存0、ユーザーprocess停止0。新たなbuildを自動開始しなかった。

| 観測 | 結果 |
| --- | --- |
| Cargo実行期間 | 21:04:00.783→21:08:03.881 JST |
| 終了 | Cargo exit=-1、外側command exit1（親指示による中断。test FAILへの換算なし） |
| 外側安全条件 | Admin=False/Hidden=true/Timeout=false/Waited=true/合成TEMP残存0 |
| raw stdout | 0 byte |
| raw stderr | compile進行あり、Cargo Running/Finished行0 |
| test実行の観測 | running/test-result marker0、PASS0/FAIL0/SKIP0。実行未観測であり、全生涯process追跡による未実行証明とは区別する |
| source | 170項目前後差0、HEAD cb882031、source dirty0 |
| 旧製品2EXE | 両SHA不変 |
| 旧default artifact | 24中23はbytes変更、1（test-bin）は不変。欠落0 |

正本は `full-default-runtime/{stdout.log,stderr.log,result.json,stop-result.json,stopped-execution-audit.json}`、`full-source-after-stop.json`、`full-default-old-artifacts-after-stop.json`、`full-old-products-after-stop.json`。session56929は終了済み。空stdoutをregex集計した最初の診断でnull-input例外が出たため、raw byte長と空文字を明示したauditへ修正した。これはテスト失敗ではない。旧default artifact Gateを新bytesへ流用しない。全体gateはPASS0/中断1で未完了、試用full/関連no-run/release/nativeは未開始である。

### 限定read-only環境/fingerprint診断

秘密を含むenv全文を取得していない。cargo/rustc path/version、RUSTUP_TOOLCHAIN/RUSTUP_HOME/CARGO_HOME/RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS/RUSTC_WRAPPER/RUSTC/CARGO_TARGET_DIRとLOCALAPPDATA/TEMP/TMPだけを調べた。`full-sandbox-environment-audit.json` / `full-ordinary-environment-audit.json`、さらに停止時と同じStart-Process/合成環境で-Vvだけを実行した `full-default-runtime/environment-probe-versions.json` に記録した。version probeはbuild/testではない。

| 軸 | 確認済みの比較と限界 |
| --- | --- |
| source/HEAD | FX1 no-runはd446c5d＋限定差分、fullはcb882031。全170bytesは一致。commit hook再書込の根拠なしは親の独立診断を接合し再読しない |
| command | no-runは`test --all-targets --locked --no-run --message-format=json`、fullは既定`test --all-targets --locked`。default target pathは同じ。JSON/no-runの差だけで原因を断定しない |
| toolchain | 着手と今回のsandbox/ordinary/同合成Start-Processでcargo1.98.1 commit797e8a9bc、rustc1.98.1 commit48a229cea/LLVM22.1.8が一致。RUSTUP_TOOLCHAIN等とRustFlags/Wrapperはnull |
| config | user Cargo config前後log差0。repo/親directoryのCargo config8候補は現時点全不存在（`full-ancestor-config-audit.json`）。祖先の変更前snapshotは未取得 |
| process環境 | no-runはsandbox、fullはordinary-userで合成LOCAL/TEMP/TMPを明示。現在のsandbox auditではAppContainer配下、ordinary親はユーザーLOCALとF:/TEMPCO~1、full子は指定target配下。旧no-run開始時のLOCAL/TEMP/TMPの完全snapshotは未取得なので、現在値を過去実測値へ読み替えない。通常no-run JOBS未指定→full JOBS2も差として残す |
| fingerprint | 現root25項目はrustc数値15335473461698482004/config9396254390672932401が一致、test profileは1722584277633009122/RustFlags=[]。残る旧test-bin fingerprint mtime20:44:14、再compile済みlib21:06:26/int21:06:49で実更新を確認（`full-fingerprint-after-stop.json`）。proc-macro2のbuildscript/libも21:04:08–09へ更新し、現在RerunIfEnvChangedはRUSTC_BOOTSTRAP=null。旧fingerprint全体snapshotとdirty理由logがなく、個別の初発原因は未確定 |

`.rustc_info.json` はmtime20:14:44、内容を `full-rustc-info-after-stop.json` に保存した。再compileとEXE bytes変更は観測事実であるが、ENVpath変更だけを確定原因とはしない。Cargo fingerprintの上書き前証拠不足を残す。設定変更、OS変更、source/harness編集による是正は行っていない。

### 親へ返す再開案（未実行）

構成ごとに固定した合成環境rootを使い、ordinary-user/同cwd/同LOCALAPPDATA/TEMP/TMP/同target/JOBS2/同toolchainでno-run→親の新全artifact対応Gate→exact fullを行う。環境treeのcleanupは両command終了後だけとし、ログは別prefixへ保存してこの中断証拠を保持する。固定環境のmapと実command/versionをログへ残し、no-run/full間に変えない。

既存Invoke-V2Case.ps1はRust exact1/List専用であり、Cargo全体commandをそのまま受け付けない。今回のStart-Process型監督はEnvironment hashとcwd/targetを固定して再利用でき、script file/shared harnessの編集は不要である。親の次解除後のno-runにprocess限定 `CARGO_LOG=cargo::core::compiler::fingerprint=info` を追加してdirty理由を取得する案を提案する。同一性を優先するならそのlog設定もfullへ同じ値を渡す。現在は提案までで、診断no-run/全体再開/related/releaseを実行していない。

終了週枠64→64%（window10080/reset1792496607、最終応答前）。token counter/log session ID/provider返却model・effort/親子包含は未取得。指定Sol/mediumを維持し、停止結果と原因未確定を親へ返す。

## full2：固定ordinary環境 no-run と局所dep-info診断

親が構成別の同一ordinary-user/合成環境/cwd/target/JOBS2/toolchainとprocess限定fingerprint logを許可した。`full2-fixed-envs.json` は再開用のログであり永続Cargo設定ではない。環境IDは `ET3-full2-{default,trial}-20261014`、LOCALAPPDATA/TEMP/TMPは `target/early-trial-build/et3/full2-env/{default,trial}/{local,temp}`、cwdはrepo root、targetは既定default/trial、JOBS=2、CARGO_LOG=`cargo::core::compiler::fingerprint=info`、WW5_ET2_CHILD/BASEは空と固定した。cleanupはno-runと親解除後fullの両終了・所有process終了後だけとし、待機中のprocessは残さない。

通常 `cargo test --all-targets --locked --no-run --message-format=json` はexit0/build-finished=true、全24executable artifact/うちtest profile22、compiler error0/warning22。正本は `full2-default-no-run/{start.json,result.json,artifacts.jsonl,stderr.log,artifacts-fixed.json,summary.json,toolchain.json}`。Hidden/AdminFalse/期限内/Waitedtrue/TEMP残存0、session33504終了、list/runtime0である。source170/12G0/4markerは前後差0（`full2-source-after-default-no-run.json`、G0/markerの同prefix）。before markerはFX1同raw sourceのreference由来で、そのprovenanceを別記した。

新通常test-bin SHA `7C5F5B089E4AC9A63770D5D4959879454B8982C1219358919C198E370586A0E6`、codex_process SHA `F3C72D1743083C5C0A1436CF90A3CC4A169DE3C85C63A84CFED418098FE1CEF2`、lib SHA `7B3319BF9CAC496B3530F4395B131F45D093FC353B4C59F7B4AB83020FD26E9B`。全path/profile/featuresは上記JSONへ固定し、まだruntime解除されていない。

### 初発原因の限定：Cargoが不存在のprefixを依存fileへ保持

新INFO logはserde/serde_core/proc-macro2/bytes等へ `dirty: FsStatusOutdated(StaleItem(MissingFile { path: "C:\\Users\\安井" }))`、その依存先へStaleDepFingerprintを明示した（`full2-default-fingerprint-reasons.log`）。この時点で単にENVpathだけを原因とする説明は不足する。

対応proc-macro2のRustc `.d`ではユーザーpathの空白がU+3000（UTF-8 `E38080`）、直前は`井`でbackslash escapeがない。Cargo binary dep-infoには正常なfull profile path11件と、**正確な `C:\Users\安井`＋NUL終端のentry1件** が共存する。このprefix pathは不存在である。新固定ENVの通常compileで21:17に再生成されたdep-infoにも同entryが残り、既存試用dep-infoにも同じentry/無escape空白を確認した。実bytesの限定証拠は `full2-dep-info-path-audit.json`、`full2-dep-info-bytes-audit.json`、`full2-both-dep-info-prefix-audit.json`。

確認できたのは、Cargoの依存file解釈で不存在prefixを持ちMissingFileとなる局所機構と、その再生成である。上流parserのコード位置は調査していない。最初の中断fullにはdirty理由logがなかったので、その初発を同logで直接観測したとは扱わない。同一ENVだけで次のexact fullの再compileを防げるとは主張しない。

親は同原因の試用no-run反復を不要と判断し、未開始を維持した（試用PASSへ換算しない）。通常no-run以外のfull/related runtime/runner導入/nativeは実行していない。process限定Cargo target runnerの機構に関する公式根拠は親が取得し、導入はユーザー回答待ちである。source/依存/fingerprint/OS/global設定の修正やpath alias作成は0。親の公式調査を重複していない。

### release buildだけの追加解除

親が通常no-run成功・証拠固定後の未影響作業として試用→通常→別related targetの単独qwen release **build only** を許可した。必須release/locked/bin/featuresを保持し、出力形式だけ `--message-format=json` の追加を親が明示許可した。native/全体runtime/newrunner/提供先copyは未許可のままである。各buildで同sourceHEAD cb882031/170項目を固定し、JOBS2/ordinary/Hidden/固定合成環境/有限期限/回収/旧mainとstandalone保全を維持する。

試用commandは `cargo build --release --locked --bin wordweave5 --features early-trial --message-format=json`、targetは既定trial、logは `full2-trial-release/`。環境IDは上記固定trialを再利用し、1800秒上限で開始した。build成果物の存在を全体検証PASSや提供完了へ代用しない。

試用releaseは21:24:11→21:28:00 JST、exit0/build-finished=true、compiler error0/warning6。EXEは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/trial/release/wordweave5.exe`、SHA `E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2`。JSONのfeatures=[early-trial]、profileはopt3/debuginfo0/debug_assertions=false/overflow_checks=false/test=false。AdminFalse/Hidden/期限内/Waitedtrue/TEMP残存0、source170/G0差0、旧製品2SHA不変、session39083終了済み。正本は同dirの `artifacts-fixed.json` / `start.json` / `result.json` / `summary.json` / `source-after.json` / toolchainログ。EXE起動/copy0。

次の通常 `cargo build --release --locked --bin wordweave5 --message-format=json` は固定default環境、別default target、log `full2-default-release/` で逐次開始した。試用と競合するbuildはない。

通常releaseは21:29:23→21:32:58 JST、exit0/build-finished=true、compiler error0/warning5。EXEは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/default/release/wordweave5.exe`、SHA `661156D57067404BE8A5C232CED6A19442BF5D3F6487EF4B905AF4848AE4F2C2`。JSON features=[]、profileは試用と同じopt3/debuginfo0/assertionsfalse/overflowfalse/testfalse。source170/G0差0、旧製品2SHA不変、AdminFalse/Hidden/期限内/Waitedtrue/TEMP残存0、session35827終了済み。正本は同dirのartifact/start/result/summary/source-after各JSON。EXE起動/copy0。

単独qwenの `cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio --message-format=json` を逐次開始した。出力は解決済みignored `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/related`、logは `full2-qwen-release/`、環境ID `ET3-full2-related-20261014` は `full2-related-fixed-env.json` に固定した。default/trialの固定EnvMapを変更せず、relatedの合成LOCAL/TEMP、JOBS2/同CARGO_LOG/cwd/ordinary/Hidden/1800秒上限を別記した。旧standalone出力先を使わない。

単独qwen releaseは21:34:27→21:37:16 JST、exit0/build-finished=true、compiler error0/warning6。EXEは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/related/release/qwen-audio.exe`、SHA `DD93326B164661A74CDF498852DF19DF471C7A3AFBED65980D5AAA45A30526AA`。features=[default,gui,mcp,windows-credentials,windows-job]、profileはopt3/debuginfo0/assertionsfalse/overflowfalse/testfalse。AdminFalse/Hidden/期限内/Waitedtrue/TEMP残存0、source170/G0差0、旧製品2SHA不変、session77215終了済み。正本は同dirのartifact/start/result/summary/source-after/toolchain各JSON。native/API/資格情報/mic/copy0。

### build-only最終返却

| 今回許可されたrelease build | exit / 結果 | EXE SHA256 |
| --- | --- | --- |
| 試用WordWeave5 | 0 / build PASS | `E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2` |
| 通常WordWeave5 | 0 / build PASS | `661156D57067404BE8A5C232CED6A19442BF5D3F6487EF4B905AF4848AE4F2C2` |
| 単独qwen-audio | 0 / build PASS | `DD93326B164661A74CDF498852DF19DF471C7A3AFBED65980D5AAA45A30526AA` |

3build PASS/FAIL0、各実command/env/profile/features/絶対path/時刻/exitの統合正本は `full2-release-summary.json`。これはtest PASS数ではない。full2通常no-runはcompile PASS1、試用no-runは親の同原因反復不要指示で未実施、全体runtimeは中断のまま未完了である。

最終 `full2-final-preservation.json` はHEAD cb882031、source170/G0 12/marker4差0、full2通常debug24artifact SHA変化0、旧main `9B888E9FC923823C59B4C3B0AB1A4DAD2051CA6139DE153B6DE6CA213DD387D3` と旧standalone `57A3BE5B10F117E146E1C8AAC77616AA4A0F0873D74D68D858BD8C6E9A64506E` 不変を記録した。各新release EXEの最終現物SHAも統合JSONと一致した。

21:40:09 JSTのread-only `full2-final-process-audit.json` は記録済みno-run/release4root PIDと同時刻の直接子、対象targetのcompiler候補、新release3EXEのlive候補が各0である。4commandは全Waitedtrue、sessions33504/39083/35827/77215は全exit0で終了した。ユーザーprocess停止0、無期限background process0。固定合成環境treeとraw logはignored配下に保持し、後続full/relatedの親解除待ちのためcleanup境界を早めていない。

native DPI条件と新Cargo runner導入のユーザー回答は親で未受領である。新runner/harness、OS/global設定、source/依存/fingerprintの修正、関連test/no-run反復、全体runtime再開、native、提供folderへのcopyは開始していない。実IME/pen/TTS/mic/Codex/Qwen等の受入も未検証のままであり、mock/旧ログ/今回buildで解消したとは扱わない。

今回区間の週枠64→65%（window10080/reset1792496607、終了21:39頃取得、最終応答前）は共有account観測であり本工程消費へ換算しない。子token開始/終了counter/log session ID/provider返却model・effort/親子包含は未取得。planner指定Sol/mediumは維持した。今回の許可作業は3release buildまで終了し、残るgateと未回答事項を親へ返して所有を返却する。

## 22:18再開・RootUnavailable native限定観測（部分取得、中断）

親からユーザーの一時Cargo runner/DPI150・200試用時移管の承認と、delivery-review冒頭のRootUnavailableだけのnative Gateを受領した。正常起動・資格情報・実API/mic・音読・OS設定・他app/desktop全体capture・提供copyを解除したものではない。指定runner Sol/mediumを維持した。HEADは `6fb6da3b568eb8e4e902f66260ee5df48f9a00a2`、共有の文書dirtyは保持した。

PowerShell automationとversion-compatibility/coding-style/error-handlingを全文読了した。既存PowerShell7監督とStart-Process Environmentの証拠に合わせTarget=ps7（実機7.6.6）、単発のoperational commandだけとし、repo source/期待値/shared script/汎用native harnessを作成しない。skillは明示encoding/StrictMode/Stop/期限/try-finally/所有子回収へ適用した。UIAutomationClient/System.Drawingのassembly読込み能力は起動前に確認した。

storeのresolve→Storage::open、mainのearly_trial_creator→show_early_trial_failureを現物読取した。相対LOCALAPPDATAはBeforeWrites/RootUnavailableでrfdへ行き、WordApp factory/diagnosticsの前に戻る。170sourceとrelease SHA `E3A0A87861D835C1E142072A14F3EBF7CF9DA0A8B95D404F4BF99402D4B1BBD2` を非管理者tokenと共に実行直前に照合した。通常/試用dataを読ませる正常rootを渡していない。

出力正本は `target/early-trial-build/et3/native-refusal/ru-20261014-224121/`。最初のobserver EncodedCommandはWindows command-line長制限でprocess起動前に拒否された（アプリ起動0）。同じoperational commandをメモリ内GZip/EncodedCommandへ圧縮し、source/harness fileの追加なしでobserverを起動した。commandそのものは `operational-command.log`、転送長/hash/起動前failureは `command-transport.json` に記録した。

実アプリ起動は今回 **1回**、22:42:26.929→22:42:28.655 JST。observer PID19940、native PID12880、所有HWND854890、class=#32770。Start-ProcessはHidden、cwd/TEMP/TMPは新合成tree、LOCALAPPDATAは相対literal `ww-et3-relative-native-refusal`。outer observer40秒期限、未知要素で停止して所有windowへWM_CLOSE→wait、強制kill不要。observer exit1（観測不成立）、native exit1（StorageOpenRejectedの期待拒否）、timeoutfalse/Waitedtrue/cwd・TEMP残存0。global input/他app操作/desktop capture/user process停止0である。

| 観測対象 | 新nativeで取得した事実 |
| --- | --- |
| caption | `早期試用を開始できない` 完全一致 |
| body | RootUnavailable/BeforeWrites/通常版fallbackしない日本語3文をUIA Nameで全文一致確認 |
| native stderr | `AppCreation(StorageOpenRejected { safe_message: ... })` の同じ拒否reason/stageを確認 |
| OK | 所有HWND44171330、Name=`OK`、Win32 ClassName=`Button`、rect1080,603,75,23、offscreen=false |
| dialog rect | UIA rect758,452,420,192 |
| 中断理由 | UIA ControlTypeがOK/Staticとも`ControlType.Pane`、HasKeyboardFocus/IsKeyboardFocusable=false。observerのButton型条件と一致せず停止した。製品ボタン欠陥とは確定しない |
| 未取得 | PNG/実window monitor scale/focus/keyboard。処理はこれらへ進んでいない |

`uia-elements.json` / `native.stdout.log` / `native.stderr.log` / `result.json` / `observer-{start,result}.json` がraw正本。この部分取得をnative全受入PASSへ数えない。nativeケースPASS0/未完了1、製品FAILは未確定である。実credential/API/micのcounterをこのreleaseへ新設しておらず、その非到達は固定sourceのRootUnavailable分岐と観測した拒否状態の根拠へ限定する。

source後170差0、試用release/旧通常/旧standaloneの各SHA不変を `source-after.json` / `exe-after.json` に保存した。22:44:17 JSTのread-only `process-after.json` は記録PIDと時刻範囲の所有process/直接子live0。アプリ/observerは終了し無期限processはない。

親へUIA provider分類の観測制約を報告した。同source/EXE/RootUnavailableに限定し、UIA分類へ依存せずWin32 Class=Button/IDOK/GetGUIThreadInfoでfocus、PrintWindow、所有HWNDへのkeyboard messageを補充する案を返した。新しい指示前に再起動しない。PNGのためにdesktop captureやglobal inputへ切り替えない。

開始週枠は親の65%/13:18UTC引継ぎ、終了取得69%（window10080/reset1792496608、22:44JST・最終応答前）。共有・並行工程を含むため単独消費量ではない。子token counter/log session ID/provider返却model・effort/親子包含は未取得である。

### 親許可のWin32補充と最後のread/capture補充

親が同source/EXE/RootUnavailableだけのWin32 one-off補充を許可した。`ru-supp-20261014-224728/` はnative PID6152/observer17188、22:47:29.186→22:47:29.868 JST、caption/body/所有Buttonの一致後、GetDlgCtrlID=1という未確認のobserver条件で中断した。ID実測値をassert前へ保存しなかったoperational loggingの欠落も報告した。PNG/DPI/focus/keyboardへ進まず、所有WM_CLOSE/wait、native exit1/observer exit1、timeoutfalse/cwd・TEMP残存0である。製品の失敗と数えない。

局所rfd0.15.3 backend sourceではOk→TDCBF_OK_BUTTONとTaskDialogIndirect戻値IDOKの扱いを確認したが、子HWNDのcontrol IDを1と保証する根拠は確認できなかった。観測commandの前提を製品契約と取り違えないため、親へ値を先に保存するread/capture補充を提案し、最後1回だけ明示許可を受領した。root source/期待値/汎用harnessやrfd依存を変更したものではない。

最終正本は `native-refusal/ru-final-20261014-225110/`。native PID20092/observer39072、22:51:11.447→22:51:12.320 JST。caption/body/owner一致で撮影対象を同定し、未確認のUIA型/control ID/focusを撮影前gateへ追加せず、PrintWindow/DPI/全actual ID/focusを取得した。操作は所有WM_CLOSE cleanupだけで、key messageも標準keyboardも実行していない。

| 最終新native実測 | 結果 |
| --- | --- |
| caption/body | 前記日本語全文一致、Win32 Static全文も一致 |
| 所有window | HWND1183230/#32770、rect左758/上452/幅420/高192 |
| 所有window monitor | GetScaleFactorForMonitor HRESULT0、100% |
| PrintWindow | true、所有windowだけの420×192 PNG |
| OK | HWND44302402、Win32 Class=Button、Text=OK、**actual control ID=2**。public semantic IDOK=1との同一視はしない |
| body/icon control | Static ID65535（日本語全文）、Static ID20（icon）、root ID0 |
| Win32 focus | GetGUIThreadInfo=true、focus HWND44302402、owner一致、Button/OKである |
| UIA providerの限界 | 同じOKをPane/HasKeyboardFocusfalseとして返す。Win32実測と別記する |
| keyboard | 標準操作未確認。WM_KEYDOWN補助操作も今回実行0 |
| 終了 | native exit1（拒否Err）、observer exit0、timeoutfalse/Waitedtrue/TEMP・cwd残存0。kill不要 |

PNGは `owned-refusal.png`、SHA256 `79A18A944717CE06AA63EE975DD0EE22CAEE42F3033A5EABC18DA9FDD119B40B`。このPNGを直接表示して、日本語title/body、折返し、error icon、右下OKとfocus点線が420px幅内で重なりや見える範囲の欠けなく描画されていることを確認した。desktop全体や他appを撮影した画像ではない。RootUnavailable以外の拒否原因・長いpath表示や通常WordAppの狭幅はこの一枚から判定しない。

最終 `result.json` のPassed=trueは `ScopeOfPassed` に明記した所有拒否windowの画像/metadata取得だけの成功であり、native全gate PASSではない。観測commandは成功1/中断2、実アプリ起動3回、標準keyboard未確認、製品native全面PASS未判定である。以前の中断ログを消去・成功へ書き換えていない。

最終source170差0/HEAD6fb6da3と試用E3A0…BBD2/旧通常9B888…387D3/旧standalone57A3…4506E不変を `source-after.json` / `exe-after.json` / `final-audit.json` に保存した。22:53:17 JSTの `native-refusal-process-summary.json` は今回3組のnative/observerと所有子のlive0、全Waitedtrue/timeoutfalse/残存0。ユーザーprocess停止/global input/他app操作/desktop capture/実資格情報/API/mic/提供copy/再委譲/commitは0。

最後の許可補充を終了し、再起動しない。RootUnavailableの新PNG/DPI100/本文/OK/focusと、残る標準keyboard/他native gateを親へ返して所有返却する。終了週枠70%（window10080/reset1792496607、22:53JST・最終応答前）、開始65%との差は共有並行区間の観測であり本工程単独量ではない。子token/log session/provider返却model・effort/親子包含の欠測は継続する。

## ET3-G初回独立合成試験：不合格・所有返却（2026-10-15 00:42 JST）

planner `early-trial-gate-plan.md` のrunner指定は `gpt-6.1-sol/medium`、provider返却model/effortは未取得である。AGENTS/development-team/KPIとwordweave-change全文を読み、検証・報告だけを適用した。親から静的P1二件closure PASSと初回合成試験だけの解除を受領した。実行対象HEADは `2cec3c08686dae9c6e74131b4369f03366c38d1f`、開始dirtyはdelivery-review、gate-design、usage-observation-20261014-et3、currentの文書4件である。製品source/test/期待値/設定を変更していない。

固定入力SHA256はInvoke-CargoGate.ps1 `F39A95D99618EBD35576C64F8144C710110EF0D77FA91BE66AB28B9F91A1A892`、Test-CargoGate.ps1 `763CB0B1D4308C6FFF029CCBE7FBA3E5667E4DE2EB1CEE963222B554534CD0CC`、cargo_gate_entry.rs `3AEB0B19E2D03EE088EB1BEDA4DBACEEC0406E12ED75D59AB95A7E331F42FD41`、cargo-gate-entry.exe `1C04288220C3D98B182F3011FE3D1DE8AE88B050871DF400B21ED8AECCCC5E94`、fake.rs `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016`。設計SHAは `2DE1F344FEC43468AA1F40716A7A912BDCD943318B5D39FDC5F47CBEEC970937`。前後現物一致、driverもsourceMatched=trueである。

R=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、suite=`8b960cec-97f2-43fb-9851-50b4044d17be`。git check-ignore成功後、通常ユーザーAdmin=false、pwsh7.6.6を確認した。CIMに必要なsandbox外実行は管理者昇格ではない。実commandは `& 'F:/Program Files/PowerShell/7/pwsh.exe' -NoLogo -NoProfile -NonInteractive -File R/Test-CargoGate.ps1 -GateScript R/Invoke-CargoGate.ps1 -OutputRoot R/verification/8b960cec-97f2-43fb-9851-50b4044d17be` （Rは上記絶対pathへ展開）。一回だけ実行しexit1、firstFail停止、自動再試行0である。新fixture rustc1.98.1 compile成功、stderr0byte、fake.exe SHA `1AF001913CECEA34F4B69B14E5E093A2A8AAF933416D20200F88054B6BAB8965`。入口controller PID2620開始15:40:24.5891281UTC、GATE21 RunOwner PID37284開始15:40:31.8137400UTCである。

controlはPASS2/FAIL0/SKIP0/NotRun0。80caseのraw resultはPassed26/Failed1/NotRun53/Skipped0、gatePass=falseである。ただしPassed26（ENV01–06/GATE01–20）は全variants=[]、対応case directory0であり、静的契約検査や実process動作の成功としても扱えない。driverのL1995付近 `$variants = if (...) { ... } else { @($null) }` → `foreach ($variant in $variants)` により通常case呼出が消失した疑いが強い。実証済みgate behavior PASSは0、raw26件を合格へ接合しない。テスト作者へ返すべき観測不備でありrunnerは修正しない。

最初の実caseはGATE21/run（runId `f8ed9578-d246-40ed-be83-a02aa797c96f`）。失敗は `BEHAVIOR_ASSERTION: Terminal gate evidence missing; unknown is not PASS`、Confirm-CaseResult L1688、driver L2000。run rootにはrun.json/cancel.jsonだけ、terminal result0、controller stdout/stderr各0byte、target started marker0。classificationはUnverifiedBehaviorOrFixtureFailureであり、製品拒否動作のPASSにも製品FAIL確定にも読み替えない。GATE21/approval variant未到達である。

15:41:54.7818683UTCのread-only CIM監査（exit0）は固定controller/RunOwner PIDからの生存子孫とsuite内実image候補0、cargo-gate-entry.exe全体0。初回広いcommand文字列検索は観測用pwsh自身を候補へ拾ったため、固定PID鎖/実image境界へ限定して確認した。ユーザーprocess停止0である。gate自身のkill/wait/cleanup terminal証拠は欠落しているためcleanup PASSは未成立。suiteのtemp/localと合成証拠fileは保持し、現在生存0をfixture残存0へ換算しない。

正本は `R/verification/<suite>/result.json`、`frozen-inputs.json`、`toolchain.json`、`identity-control/`、`GATE21-run/author-failure.json`、同controller stdout/stderr、`R/runs/<runId>/{run,cancel}.json`、追加read-only結果 `independent-runner-audit.json` である。親へ不合格とraw合格件数の不成立を返し、所有を返却する。製品full/real Cargo接合ORDER07/実API/資格情報/native microphone・pen・TTS・IME・authentication/提供copyは本試験では未実施であり、旧ログや合成controlで解除しない。app終了/EXE置換/再委譲/公開0である。

週枠開始91→終了92%（window10080分/reset1792496607、最終応答前）の共有account観測は並行工程を含むため単独消費へ換算しない。子token開始/終了counter、固有session ID、provider返却model/effort、親子包含は未取得である。
