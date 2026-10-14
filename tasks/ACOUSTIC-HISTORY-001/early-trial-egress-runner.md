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
