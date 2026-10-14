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
