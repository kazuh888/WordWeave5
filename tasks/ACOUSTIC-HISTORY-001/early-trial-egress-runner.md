# ET2-P01-V 独立runner記録

2026-10-14。planner指定 `gpt-6.1-sol/medium`、担当 `/root/trial_egress_runner`。返却model/effort metadata未取得。AGENTS.md、development-team、KPI、wordweave-change全文を読み、同Skillのリスク別検証と結果報告のみ適用した。source・test・期待値・settings編集、再委譲、公開は0である。

今回の完了境界は独立no-run buildのsource/EXE対応を親へ返すところである。唯一の設計ET2-G0に従い、親の新解除まではEXE起動/list/runtimeを禁止する。実装者EXEへの解除をrunnerへ流用しない。ET2独立Post、ET3、native受入完了は主張しない。

## 固定入力

HEAD `fef8bd27dc1fbf5065f568d0d1d7e538f376bbb3` ＋V1の4source未コミット差分である。2026-10-14T18:24:31+09:00の読取で8source SHAは `early-trial-egress-review.md` V1表に一致、作者4marker SHAは `early-trial-egress-tests.md` 表に一致した。marker部分はBEGINの最初の `/` からEND末尾 `S` まで、内部改行を保持してUTF-8 SHA256とした。

source/Cargo snapshot正本は `target/early-trial-build/et2-egress-runner/source-before.log`。通常/試用専用targetを逐次使用し、実装者targetを上書きしない。作者予定集合は通常/試用bin各20件（A01–A11/B01–B03/C01–C02/D01–D04）である。従来ET2集合は作者startup表を参照し、旧ET06の1件は新D群に置換されたことを保持する。現段階では予定集合であり、list・実行確認件数ではない。

## 検証記録

通常bin no-runはexit0、`build-finished.success=true`、2m11sで終了した（18:27:00頃）。起動済command/session58700は終了確認済みである。commandは以下である。

```text
cargo test --locked --bin wordweave5 --no-run --message-format=json --target-dir target/early-trial-build/et2-egress-runner/default-target
```

JSON由来features=`[]`、profile=`{opt_level:"0",debuginfo:2,debug_assertions:true,overflow_checks:true,test:true}`。EXE絶対pathは `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-egress-runner\default-target\debug\deps\wordweave5-72e8adffa8f9ef2c.exe`、SHA256は `33B8338B0BDCB3254AA60BBFB1A5F95D0D981F90486637BBADC3BA160CDDA248` である。実装者EXEとは別生成物であり、その解除を流用しない。

logsは専用dirの `default-bin.jsonl`、`default-bin.stderr.log`、`default-bin.exit.log`。18:27:35の `source-after-default.log` と `source-before.log` のCompare-Object差分0であり、8sourceとCargo.toml/Cargo.lockは前後不変であった。したがって作者4markerもビルド前確認から不変である。ビルドPASS1/FAIL0、runtime PASS0/FAIL0/SKIP0、runtime未実施は予定20件である（SKIP実行結果へ換算しない）。

親よりA09の感度不足（catch_unwindがcount assert panicも意図panicと混同する）を理由に次build停止指示を受領した。通常V1のcompile証拠を保持し、試用bin・通常/試用libの追加no-run、list/runtimeはいずれも未開始で停止した。作者修正でsource凍結が更新されるため、このEXEを修正後sourceの証拠として扱わない。新source/EXE/G0の親確認が必要である。テスト修正や感度判定はrunnerで行っていない。

## 使用量・未検証

開始側の取得は2026-10-14T18:23頃の共通週枠usedPercent49/windowDurationMins10080/resetsAt1792496607である。親・他担当と並行するため当工程消費率へ換算しない。tokenの開始実在snapshot、session ID、親子包含は未取得、終了snapshotは後述する。返却metadataからモデルを確認できていないため指定値と実行値を同一視しない。

実API、credential load/save、mic、pen、TTS、IME、実Codex認証、native UI、Admin/UAC/OS変更は未実施である。no-run成功はこれらの新native受入を意味しない。

終了側の取得は2026-10-14T18:27:35+09:00の共通週枠usedPercent50/windowDurationMins10080/resetsAt1792496608（最終応答前まで）である。開始側49との表示差を当工程単独消費率としない。token開始/終了snapshot・session ID・包含関係は未取得、返却model/effort metadata未取得である。使用量欠測と最終応答未反映を親へ返す。

## V2 A09是正後の独立no-run

親のV2 sourceG0読取確認とno-run限定解除を受領し再開した。ai sourceは `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA`、ai作者markerは `E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7` と現物一致、他7source/3markerはV1一致であった。A09の専用型panic/catch外payload検査と既存oracle保持を対象としており、V1のcatchによる検出力不足を成功扱いにしない。

source/Cargo固定logは `v2-source-before.log`、通常後は `v2-source-after-default.log`。比較差0である。作者表からP01実名20件を `v2-planned-p01.log`、旧ET06を除くstartup17候補（bin通常6/試用16＋lib1）を `v2-planned-startup.log` に固定した。ai ET1 config実名 `ai::early_trial_paths_tests::et06_config_uses_storage_namespace_without_launch_or_environment_rewrite` もsource現物で確認して対象候補とする（listは後続照合）。既存qwen_settings/qwen_reading/media/fatal-close/restore/取込等の関連回帰は親新解除後の実EXE一覧から選択集合/件数を固定する。

```text
cargo test --locked --bin wordweave5 --lib --no-run --message-format=json --target-dir target/early-trial-build/et2-egress-runner/default-target
cargo test --locked --bin wordweave5 --lib --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-egress-runner/trial-target
```

binとlibは構成ごと同一commandでno-runし、通常→試用を逐次処理する。通常commandはexit0、build-finished.success=true、2m07s。JSON features=[]、bin/lib両profile.test=true、opt_level0/debuginfo2/debug_assertions true/overflow_checks trueである。通常のJSON/診断/exitは `v2-default.jsonl` / `v2-default.stderr.log` / `v2-default.exit.log`。V1と同じ通常bin pathはV2生成物に置換されたため、V1記載SHAを現EXEへ適用しない。

| V2通常target | EXE絶対path | SHA256 |
| --- | --- | --- |
| bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-egress-runner\default-target\debug\deps\wordweave5-72e8adffa8f9ef2c.exe` | `8F72838BA93B186D6F980A5157E440304AF18CCEA5B9BF1D81D58CBF8E52B969` |
| lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-egress-runner\default-target\debug\deps\wordweave5-b3388a7bbef44bb4.exe` | `1C174022959E71BDEB6EAE2DC7C9DA26121637C045469B7880DCEE0EC2816409` |

試用commandもexit0、build-finished.success=true、1m28sで終了した。JSON features=["early-trial"]、bin/lib両profile.test=true、その他profileは通常と同じである。診断正本は `v2-trial.jsonl` / `v2-trial.stderr.log` / `v2-trial.exit.log`。`v2-source-after-trial.log` の10fileとbefore比較差0、18:35:13+09:00の4marker再計算も全一致であった。通常session20162・試用session5178・JSON診断session85361は全てexit0で終了確認済みである。

| V2試用target | EXE絶対path | SHA256 |
| --- | --- | --- |
| bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-egress-runner\trial-target\debug\deps\wordweave5-e0ab9f88db3adadd.exe` | `D6DA249FF9AF7755F774CD9002D750E57BA325193B733D835F2F3167347659C0` |
| lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-egress-runner\trial-target\debug\deps\wordweave5-1e58dd7001d62083.exe` | `9920760395F7EF4BE4A50A569B9A91C9C93B62BB7766FB0E33953D73594DFBF1` |

V2 Cargo no-run command PASS2/FAIL0、生成test EXE4。追加の以下2検査もexit0であり、`v2-rustfmt.log` / `v2-diff-check.log` と各 `.exit.log` に保存した。失敗を修正するためのsource/test/期待値/settings編集は0である。

```text
rustfmt --check --edition 2021 --config skip_children=true src/ai.rs src/app/qwen_reading_ui.rs src/app/qwen_settings.rs src/app/qwen_settings_tests.rs
git diff --check
```

V2 runtime PASS0/FAIL0/SKIP0、list未実施である。予定bin各20件と従来ET2関連集合は未実行であり、EXE list未確認を成功へ数えない。新EXE組を親へ返しG0新解除を待つ。感度未実施、独立Post未了、ET3 all-targets/release未実施、native受入は上記のまま未検証である。

V2開始/終了側共通週枠は51→51%/10080分/reset1792496607（終了18:35頃・最終応答前）であり、無消費や当工程単独消費を意味しない。token開始/終了snapshot・session ID・親子包含・返却model/effort metadataは未取得である。補助hash診断commandにPowerShell foreach直結pipeのParserError exit1が1件あり、ビルド開始前に通常pipelineへ修正して診断を取得した。source/設定等への書込はなく、Cargo失敗には数えない。

## V2限定runtime（252件PASS・所有返却）

親の `early-trial-egress-review.md` 末尾「V2独立EXE対応・runtime限定解除」を現物確認し、上記4EXEだけ実行する。追加build・source/期待値編集・実API/key/mic/native/管理者・感度変異は未許可で実施しない。親はruntime中にsource無変更の未完了checkpoint `ee84f8e` を保存したが、試験対象は上記V2 source/EXEのままである。

list4起動は全てexit0/Admin=False/SHA一致/Hidden/timeout=false/Waited=true/FixtureResiduals=[]。通常bin一覧348/試用bin358、P01は両構成とも作者20実名と完全一致した。lib定数実名も両一覧に存在した。一覧logは `v2-runtime/{default,trial}-{bin,lib}-list/` のstdout/stderr/result.json。

固定集合は `v2-default-selected.log` 120件、`v2-trial-selected.log` 130件とlib定数各1件。内訳はP01各20、従来ET2通常6/試用16、関連各94である。関連集合の正本は `v2-{default,trial}-related.log`。Qwen設定/音読境界の既存mock回帰、backup・fatal-close・保存/復元/取込状態保持、media controls6、ai config1に限定し、P01重複を除いた。全ET3/all-targetsを実行したとはしない。

実行順は録音3→P01 A01–A06/B01/C01→A08–A11/B02–B03/C02→A07/D04→D01–D03→従来ET2残り→関連である。各caseは `[exact] --exact --nocapture --test-threads=1`、内部isolated利用caseだけ子完了Admin=False markerも必須、全caseは外側安全条件とexact1件成功を必須とする。title literal、namespace selector2件、lib定数は内部isolatedを使わず、子markerを要求しない。毎起動source/Cargo snapshot、EXE SHA、Admin=Falseを確認する。

実行wrapperは `target/early-trial-build/et2-egress-runner/Invoke-V2Case.ps1`、SHA256 `572E56BE23FDD59B949A678BAEACAFE4E035434D064B9B4D28425D781AFF1168`。既存実装者wrapperをコピーし、V2表記とlist filterを全実名に含まれる `::` に変更しただけである。PowerShell automation全文とversion/coding/error-handling referenceを読み、既存ps7/7.4以上・単一script・引数構成・JSON result・Hidden/65秒timeout/kill+wait/合成LOCAL TEMP cwd/残存検査を保持した。parser/Get-Helpはexit0。既知sandbox OS5を反復せず、require_escalatedの通常ユーザーで実行している（管理者権限を要求/使用していない）。外側の合成rootは証拠として保持し、ユーザーデータ削除0である。

最終結果は通常bin120/120、試用bin130/130、lib定数各1/1、合計252/252 PASS、FAIL0/SKIP0である。4listを加えた全256起動のexitは0、timeout0/wait不足0/Admin不一致0/fixture残存0。全logにfixture SKIP/skip(ped)/0 passed表示は検出されなかった。SKIPをPASSへ読み替えた結果ではない。runtime期間は2026-10-14 18:38:30→18:48:02+09:00、source/EXE最終確認は18:48:35+09:00である。

集計正本は `v2-runtime-summary.json`、全実名/起動日時/exit/権限/SHAの一覧は `v2-runtime-results.csv`、case単位の正本は `v2-runtime/{default-case1…120,trial-case1…130,default-lib-case1,trial-lib-case1}/result.json,stdout.log,stderr.log` である。各caseの実行引数は上記exact command、各listは `:: --list` である。source試験集合とEXE集合は固定logに記録され、PASS済みcase再実行0である。

最終 `v2-source-after-runtime.log` は8source/Cargo2fileについてbeforeと差0、したがって4作者markerも不変である。`v2-exe-after-runtime.log` の4EXE hashは親解除表と一致、`v2-process-after-runtime.log` は対象4EXEのlive process0であった。通常runtime session23436/試用11702とlib/list起動commandは全て終了確認済み。ET2/P01は作者の子完了marker・45秒有限timeout・kill/wait・合成tree cleanupを通し、B03の同期Runtime/worker終了oracleもPASSした。既存回帰のprocess終了を、実APIやnative workerの受入確認と混同しない。

build時の試験版はfef8bd27＋V2固定4source差分、runtime中のcheckpoint後のHEADは `ee84f8e326c3163e58ee2ed54782f0f338adc8c9` であり、同じsource bytesである。親が保存したこのcommitとfef8bd27との差を8source限定 `v2-tested-source.diff` に記録した。runnerがcommit/push/source修正を行った結果ではない。

本書と生成log/fixtureの所有を親へ返却する。V2独立focused GREENは取得済みだが、感度S1/S2/S3は未実施、独立Post未了、ET2/P01 Closedを主張しない。ET3 all-targets/release/提供gate、実mic/pen/TTS/IME/Codex認証/native受入は別の未検証項目として保持する。実API/key/実資格情報loadsave/native/admin操作0である。

runtime終了側共通週枠は52%/10080分/reset1792496608（18:48:35、最終応答前）である。V2開始51%からの表示差は他工程を含み、当runner単独消費としない。token開始/終了累積・session ID・親子包含は未取得、返却model/effort metadata未取得。指定Sol/mediumを実行値と断定しない。

## S1感度：17件の目的FAIL確認・source所有返却

親のS1 sourceG0/build/EXE照合と、予定された目的FAIL後だけ次caseへ進める限定解除を受領した。S1は `ee84f8e326c3163e58ee2ed54782f0f338adc8c9` からの一時変異であり、製品版ではない。main trial title/history拒否/fatalApp/録音返値0/通信返値0の本体5箇所だけで、runnerのsource編集/追加build/期待値変更/commitは0である。

8sourceを実装S1表へ現物照合し、4作者markerはV2と一致。固定値は `s1-source-before.log`、差分は `s1-tested-source.diff`。EXEは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA `CD5313E3214F2833B920290C3278D192915980EB66EE4455BE517D653B0B80C6`、構成はearly-trial/profile.test=true（親/実装者no-run JSONを入力）。本runnerは同EXEでlist exit0/一覧358、予定17実名全存在を確認し `s1-selected.log` に固定した。

caseの起動commandはV2 wrapperを変更せず以下の引数を用いた。各wrapperは予定どおりexit1/Passed=falseであり、これをテストPASSへ変更していない。test EXEと作者childのexit101/exact1/所定目的assertを独立判定し、認められた目的FAILだけ次へ進めた。

```text
pwsh -NoProfile -File target/early-trial-build/et2-egress-runner/Invoke-V2Case.ps1 -Executable <上記S1絶対path> -ExpectedSha256 CD5313E3214F2833B920290C3278D192915980EB66EE4455BE517D653B0B80C6 -ExactName <s1-selected.logの実名1件> -OutputDirectory <runner専用/s1-runtime/caseN>
```

| case / oracle | 実測目的FAIL |
| --- | --- |
| 1 title | main.rs:11。誤titleと独立試用literalの不一致 |
| 2 history拒否 | app.rs:398。実restoreがOk(())となりunwrap_errが拒否 |
| 3 fatalApp | app.rs:95。`fatal app must not be exposed` |
| 4 録音誤0 | media.rs:106。`actual entry must increment, never a disconnected zero`、left0/right1 |
| 5–10 A01–A06 | ai.rs:228。counter index0–5それぞれleft0/right1 |
| 11 A07 | 同count helperのindex5、上限ちょうどのTranscribe期待1がleft0/right1 |
| 12 A08 | 同count helperのindex0、join後期待1がleft0/right1 |
| 13 A09 | 同count helperのindex0がleft0/right1。専用IntentionalOwnerUnwind前のassert panicが未知payloadとしてresume_unwindされ、子exit101へ伝播 |
| 14 B01 | qwen_reading_ui.rs:26。QwenSend期待1がleft0/right1 |
| 15 B03 | 同:117の意図unwind→同期worker completion確認→同:26の期待1がleft0/right1。completion=falseや目的外panicではない |
| 16 C01 | qwen_settings.rs:253。QwenProbe期待1がleft0/right1 |
| 17 D04 | qwen_settings_tests.rs:356。実app Send後期待1がleft0/right1 |

S1の実テスト結果はPASS0/FAIL17/SKIP0であり、感度の目的FAIL確認17/17、予期しない結果0である。全17がexit101/exact1、timeout0/wait不足0/Admin不一致0/fixture残存0。listも含む18起動は毎回SHA/Admin/Hidden/有限timeout/合成LOCAL TEMP cwdを保持し、内部isolated利用16件で作者child終了とfixture cleanupを確認した。title1件は内部子fixtureを使わない。同型P01群は所定source位置・index・left0/right1と安全条件の一致を毎case判定する逐次小batchとした。並列runtime/汎用harness/基準緩和0。

正本は `s1-runtime-summary.json` / `s1-runtime-results.csv` と `s1-runtime/{list,case1…17}/{result.json,stdout.log,stderr.log}`。case5–17には個別 `expected-fail-verdict.log` もある。終了時 `s1-source-after.log` はbefore差0、4markerも不変、`s1-exe-process-after.log` はEXE SHA一致/対象live process0。runtime command sessions66254/36899/73534/97659/63739及び同期起動commandは全終了済みである。目的以外のFAILへ進んだ結果やcompile FAILを感度成功へ代用していない。

S1 source/EXEは変異状態のまま親へ返却する。S2/S3/逆復元はrunnerで行わず次の新G0を待つ。候補最終GREEN/独立Post/ET2 Closed、ET3、native/API/key/credential/mic受入を主張しない。

S1週枠は開始53→終了54%/10080分/reset1792496608（終了19:08頃・最終応答前、共通枠で単独帰属不可）。token開始/終了累積・session ID・親子包含・provider返却model/effortは未取得である。

## S2感度：6件の目的FAIL確認・所有返却

親のS2 source/no-run/EXE照合と限定runtime解除を受領した。実装報告S2の8source表を現物照合し、作者4markerもV2完全一致であった。`s2-source-before.log` と `s2-selected.log` を固定し、同EXEのlistで予定6実名全存在を確認した。追加build/source編集/作者oracle変更は0。S1からS2への逆復元・変異は実装者の担当でありrunnerは行っていない。

対象EXE絶対pathはS1と同じimpl trial-targetの `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、S2 SHAは `F66B82F50088953612AB863A36140514EF10209364581464EC7FA8D7616CE5A9`。親確認済みJSONのearly-trial/profile.test=trueを入力とし、V2/S1 EXEの解除は流用しない。commandはS1と同じ既存wrapper/各exact引数、S2 SHAと `s2-runtime/caseN` を使用した。各wrapper exit1/Passed=falseとEXE exit101を保持し、所定目的FAILを確認した場合のみ次へ進んだ。

| S2 case | 目的FAILの実測 |
| --- | --- |
| 1 fatal-D | app.rs:378。型/安全文Err通過後、(loads,diagnostics)のleft(1,1)/right(1,0) |
| 2 A10 | ai.rs:233。旧Lease/旧State1→2を通過後、新owner counter index0のleft1/right0 |
| 3 B02 | qwen_reading_ui.rs:26。保持Dialog拒否/終了/Wire0を通過後、新owner期待0のleft1/right0 |
| 4 D01 / 5 D02 / 6 D03 | qwen_settings_tests.rs:42。`before constructor: cumulative counter 0` のleft1/right0 |

実テスト結果PASS0/FAIL6/SKIP0、感度目的FAIL確認6/6、予期しない結果0。全6がexit101/exact1、timeout0/wait不足0/Admin不一致0/fixture残存0。list1もexit0/Admin=False/SHA一致/Hidden/有限timeout/合成環境/終了wait/残存0であった。D群の後続constructor/save/restore/prepare各stageのassert到達・検出力まで実証したとはしない。

正本は `s2-runtime-summary.json` / `s2-runtime-results.csv` / `s2-runtime/{list,case1…6}/{result.json,stdout.log,stderr.log}`。case2–6は所定assert位置・left1/right0・必要stage文字列と安全条件を毎case照合する逐次小batchであり、個別 `expected-fail-verdict.log` を保持した。新汎用harness/並列runtime/品質基準緩和0である。

最終19:19:15+09:00の `s2-source-after.log` はbeforeと全8source差0、4作者markerも不変。`s2-exe-process-after.log` はS2 SHA一致/対象live process0。runtime sessions6659/91658と全起動commandは終了確認済み。試験差分はee84f8e→S2の `s2-tested-source.diff`、SHA `81B5903919D41C8E4010DA234EC74E2DFC77AF65F39A5DF94EFEE83F1A6F12F5`。

S2変異sourceをそのまま親へ返す。S3/最終逆復元/追加build/commitは未実施。最終GREEN/独立Post/ET2 Closed/ET3/native受入を主張しない。実API/key/mic/実credential/native/管理者操作0。週枠は開始55→終了55%/10080分（reset開始1792496607→終了1792496608、最終応答前の共通枠）で単独消費は不明。token開始/終了累積・session ID・包含関係・返却model/effortは未取得である。

## S3感度：2件の目的FIXTURE_STAGE FAIL確認・所有返却

親のS3 source/no-run/EXE新Gateとruntime限定解除を読取確認した。source8件を実装S3表と現物一致、4作者markerはV2値と全一致。S3はapp共通loader前の合成早期Errだけであり、runnerのsource編集/追加build/最終逆復元は0である。source/実名正本は `s3-source-before.log` / `s3-selected.log`。同EXEのlistはexit0、予定2実名全存在であった。

EXE絶対pathは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA `42DBBEBC565C55EE9B1B9E3B0639FDC7B7EAB42904D91DC7B648BCA5332583CB`、親確認済みJSONのearly-trial/profile.test=trueを入力とする。commandはS1/S2と同じ既存wrapper/各exact1名引数、S3 SHAと `s3-runtime/caseN` を使用。所定目的FAIL後のみ次caseへ進む解除に従った。

| S3 case | 実測目的FAIL |
| --- | --- |
| 1 archive到達 | app.rs:325、`FIXTURE_STAGE: actual next deck write must be reached`。実bytesはold deck（et_old）のまま、期待はnext deck（et_next）。診断配列をUTF-8復号して確認 |
| 2 read拒否到達 | app.rs:359、`FIXTURE_STAGE: loader success and real custom read denial required`。left(0,0,false)/right(1,0,true) |

実テストPASS0/FAIL2/SKIP0、感度目的FAIL確認2/2、予期しない結果0。両wrapper exit1/Passed=false、EXE/child exit101/exact1、timeout0/wait不足0/Admin不一致0/fixture残存0。list1も安全条件一致で成功した。これは必要な段階へ進まない不適切な早期失敗をfixture前提assertが拒む証拠であり、実回復成功・archive拒否・回復後read拒否まで到達した証拠ではない。

正本は `s3-runtime-summary.json` / `s3-runtime-results.csv` / `s3-runtime/{list,case1,case2}/{result.json,stdout.log,stderr.log,expected-fail-verdict.log}`（verdictはcaseのみ）。毎起動のSHA/Admin=False/Hidden/有限timeout/合成LOCAL TEMP cwd、作者child終了waitとfixture cleanupを保持した。

最終19:27:21+09:00の `s3-source-after.log` は8source before差0、作者4marker不変。`s3-exe-process-after.log` はS3 SHA一致/対象live process0。全起動commandは終了確認済みである。ee84f8e→S3差分 `s3-tested-source.diff` SHAは `1D91CF89285C930FD698EA4D409ACF97DF6FFFE6AE5D526E0AB48A043AA3FF7A`。

S1/S2/S3を合計した感度目的FAIL確認は25件である。製品のPASSへ加算しない。現在のsourceはS3変異状態のまま親へ返却し、最終V2逆復元→新G0→候補GREEN→独立Postを待つ。ET2/P01 Closed/ET3/native受入は未了である。実API/key/mic/credential/native/管理者操作0、再委譲/公開0。週枠はS3開始56→終了56%/10080分/reset1792496607（最終応答前・共通枠）、token開始/終了累積・session ID・親子包含・provider返却model/effortは未取得である。

## 最終V2復元後focused GREEN：36件PASS・Postへ所有返却

親の最終復元sourceG0/no-run/新EXE対応とfocused36の解除を受領した。全8source/Cargo2fileをV2 beforeへ現物照合し差0、作者4markerも再計算でV2一致。対ee84f8e source差分0である。S1/S2/S3変異は残っていない。復元/source編集/buildは実装者所有でありrunner実施0、旧解除を自動復活させていない。

対象は同impl trial EXE絶対path `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、新SHA `F3C602DBE2EEA404573E14D1BD3835A908668ECC4A1DB06172CE5C5240447618`、親確認済みJSON early-trial/profile.test=trueである。list exit0、P01作者20実名完全一致、V2 trial-selected先頭36件（P01 20＋従来ET2 16）が全て存在した。選択正本は `final-selected.log`、順は録音3→P01既定安全順→従来ET2残り。

既存wrapperを変更せず、各exact1件を逐次実行した（commandはS1節と同形式で新SHA/`final-runtime/caseN`）。**36/36 PASS、FAIL0/SKIP0**。listを加えた37起動は全exit0/Admin=False/SHA一致/Hidden/有限timeout/合成LOCAL TEMP cwd、timeout0/wait不足0/fixture残存0である。原stdoutで全caseのrunning1/1passedを再照合し、内部isolated利用33件の原stderrに完了Admin=False markerを確認した。内部isolatedを使わないtitle literal1件とnamespace selector2件の3件は外側安全/exact1成功のみを確認した。この区別はV2報告の総称marker必須も上記へ修正したもので、検査基準の省略ではない。

正本は `final-runtime-summary.json` / `final-runtime-results.csv` / `final-runtime/{list,case1…36}/{result.json,stdout.log,stderr.log}`。最終19:38:38+09:00の `final-source-after.log` はbefore全10file差0（4marker不変）、新EXE SHA一致/対象live process0を `final-exe-process-after.log` に記録した。runtime session68326と全診断commandはexit0で終了確認済み。SKIP/0 passed表示は原logから検出されなかった。

復元後focused36のGREENとsource/EXE所有を親/独立Postへ返す。前の同一V2 bytesによる独立252PASS、S1–3計25目的FAIL、新Gate群と接合する。通常/libの理由のない再実行0、追加広域build/ET3/commit/公開0。最終Post/P01 Closedはreviewer/親の判断待ち、製品提供とnative mic/pen/TTS/IME/auth受入は未実施である。

週枠は最終GREEN開始58→終了58%/10080分/reset1792496607（最終応答前・共通枠、工程単独消費不明）。token開始/終了累積・session ID・包含関係・provider返却model/effortは未取得である。
