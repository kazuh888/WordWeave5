# ET1 独立runner

最新は末尾ET1-FX1である。影響83件を両構成で独立再実行し、各行動80PASS/0FAIL/fixtureSKIP3。旧FAIL5件は解消し、未変更証拠の接合後は通常250/試用281行動PASSである。独立PostとET2/ET3/nativeは未完了。以下の初回CHANGES_REQUIREDと旧FAILログは履歴として保持する。

2026-10-14。ACOUSTIC-HISTORY-001 / ET1。planner指定は `gpt-6.1-sol/medium`（early-trial-plan.mdと親dispatch）。実行model/effort metadataと開始usage counterは未取得である。thread `01a13801-2d5c-7f51-86ef-860c79cb964b`、session `01a08550-055c-7aa1-9834-905e4a7ce148`。

AGENTS、開発7役、KPI、wordweave-change全文を確認し、skillのrisk-based検証と報告だけを適用した。受入はET-01/04/05/06とET-D01、入力はearly-trial-plan.md ET1、early-trial-paths-tests.md段階1–4、early-trial-paths-implementation.md、既定backup-destination-runner.md/media-export-runner.mdの関連回帰集合である。

所有は本書とignored `target/early-trial-build/et1-runner/` 配下のログ・隔離fixture、cargo build生成物だけである。source/test/oracle/settings/他者編集を変更しない。再委譲、commit/push/publish、製品EXE変更、利用者app停止は行わない。

完了境界は同じ最終sourceからの独立no-run、固定EXE/list/runtimeによる通常4・試用35と既存関連回帰、hash/件数/exit/有限wait/fixture残存の確認である。ET2、ET3のall-targets/release/関連crate/native受入は親へ残す。

## 固定対象

開始HEAD `30fc43c074e21cf5b989722dce4fb48c08cddbcf`。対象sourceに対するHEAD差分は空である。開始dirtyは親所有usage記録とreview記録だけであり保持した。

| 対象 | SHA256 |
| --- | --- |
| Cargo.toml | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |
| src/store.rs | 20C1498B540395164076CECAF9B5ACA56568DE648D0712C3D9F487C53C0AF3F8 |
| src/commit.rs | 7D2720329860CEB3DCE1D2B1B8A5F85DED14EFBDE20FDF556173EF861CDBA95E |
| src/ai.rs | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| tests/early_trial_paths.rs | 0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16 |

作者prefixは先頭から本体 `use crate::{`（store/commit）、`use base64::Engine;`（ai）直前までのUTF-8 bytes。store `05DECCA1466F3E77627A7296ECFB782543CC5F91DF0F07F21328949C67CB5660`、commit `C624AC8332C4B37D1DF033A54A9F5C3E15870F54D57B9DECD3A27F2C63696D71`、ai `E609E5392402569B5E08F9170F5CA9C01EE7F5BEF49FAFE48BB3A2013D9977AD` を確認した。

全tracked source/test/manifest/lockの開始hashは `source-before.log` に記録した。実行は親の明示所有解除後に開始した。既知sandbox hardlink/atomic OS5は再試行せず、compileと通常ユーザーsandbox外runtimeを分離する。

## 独立buildとruntime

両構成のcompileコマンドは次であり、試用構成だけ `--features early-trial` を加える。target-dir/RUSTFLAGS/設定は変更していない。cargoの確定JSON `compiler-artifact` のprofile.test/executableから8固定EXEを同定する。

```text
cargo test --locked --lib --bin wordweave5 --test early_trial_paths --test progress_export_destination --test audio_history_gate --test audio_history_backup --test audio_history_progress --test audio_history_material --no-run --message-format=json
```

通常buildはexit0、Cargo wrapper実測4m23.816s。`default-build.jsonl` / `default-build.stderr.log` / `default-build-summary.log` に保存した。build途中の不完全な末尾JSON読取による診断エラーはbuild/test FAILではなく、確定JSONのparseは成功した。旧作者・実装者のPASS/EXEは今回の証拠へ流用していない。

各固定EXEの全 `--list` を取得後、下表filter＋`--nocapture --test-threads=1` で実行する。commitだけ `--skip early_trial_` を付け、新集合との重複実行を避けた。各選択実名とstdoutの成功実名集合、最終summaryの件数を照合した。

| target/filter | 通常 raw PASS | 試用 |
| --- | ---: | --- |
| lib / store::i2_export_destination_tests | 28 | 28 PASS |
| lib / store::i2_export_bundle_tests | 9 | 9 PASS |
| lib / backup:: | 13 | 13 PASS |
| lib / commit::（新test除外） | 53 | 53 PASS |
| bin / ai::early_trial_paths_tests | 1 | 1 PASS |
| bin / i2_export_settings | 25 | 24 PASS / 1 FAIL |
| bin / chat_media:: | 23 | 20 PASS / 3 FAIL |
| bin / media-export-tests.mdのfatal-close 5filter | 5 | 5 PASS |
| integration / early_trial_paths | 3 | 14 PASS |
| integration / progress_export_destination | 29 | 28 PASS / 1 FAIL |
| integration / audio_history_gate | 16 | 16 PASS |
| integration / audio_history_backup | 38 | 38 PASS |
| integration / audio_history_progress | 4 | 4 PASS |
| integration / audio_history_material | 6 | 6 PASS |
| lib / early_trial_ | 構成非該当 | 20 PASS |

通常構成は新4＋関連249＝raw **253 PASS / 0 FAIL / 0 ignored**。宛先integrationのsymlink2件はOS1314、alternate-drive alias1件はfixture不在で早期returnしたため、行動結果は **250 PASS / 0 FAIL / fixture SKIP 3**である。short-name alias確認は今回のstderrに記録されている。旧昇格symlink証拠は使用しない。

初回wrapperは関数のsummary出力と配列returnが混在し、list件数表示と実名比較が誤った。実store宛先28件はnative exit0/28PASSのままである。生ログから直接読む収集へ修正し、既取得8listと28PASSは再起動せず照合、残りだけ実行した。期待値/テスト/source変更は0。初回の誤表示は `default-selection.log`、訂正は `default-selection-corrected.log` に保持した。wrapper停止exit1を製品FAILに数えない。

通常8list＋18runtimeのnative exitはすべて0、timeoutはすべてfalse、終了wait済みである。shell wrapper再開後はexit0。WindowsIdentity/WindowsPrincipal取得成功・Administrator=Falseを各EXE起動前に強制し、true/不明なら拒否する。Hidden、上限120秒、timeout時Kill(true)→WaitForExitを設定した。kill実績0。合成TEMP/TMP/LOCALAPPDATAだけを当該process環境へ指定し、終了時TEMP/LOCAL直下残存0、全tracked source/test/manifest/lock hash不変を確認した。

実行記録は各構成 `*-native-summary.log`（コマンド引数/PID/nativeexit/timeout/wait/Admin/秒）、`*-manifest.json`（EXE path/SHA）、`*-<target>-list.{stdout,stderr}.log`、`*-<group>-runtime.{stdout,stderr}.log`、`*-<group>-selected.log`、`*-results.log`、`*-completion.log`。子processの出力・終了assertを含む生ログを保存し、子の同じcaseやtable副ケースはuniqueへ加算しない。

## 試用構成で検出したFAIL

試用buildはexit0、3m23.821s、8targetのlistも各exit0である。設定export25件のうち `app::settings_ui::i2_export_settings::s06_storage_none_uses_resolved_root_without_opening_or_creating_it` がFAILとなった。raw24PASS/1FAIL/0ignored、nativeexit101、timeoutFalse、AdminFalse、終了wait済み。親へ直ちに報告し、成功済み試験を再実行せず残りの独立集合だけを継続した。

根拠は `trial-settings-runtime.{stdout,stderr}.log`。src/app/settings_ui.rs:453の `write_settings_export(&root.join(name), b"no").is_err()` がfalseであった。fixtureは実際に作成され、cleanup成功、合成TEMP/LOCAL直下残存0。OS5や権限fixture不成立とは分類しない。子と外側の同一失敗は1件だけである。通常構成の同名は今回PASSである。

fixtureはStorageをtake/drop後、合成LOCALAPPDATAをfallback-localに変更し、固定名 `WordWeave5` のrootにsentinelを作る。製品None分岐（同file:1253）は `resolve_storage_root()` を外部guardのcaller rootとする。試用resolverは `WordWeave5-EarlyTrial` を返す。従って「未登録・Storage=Noneで通常rootへのexportが拒否されなかった」が今回の観測事実であり、一般の試用main起動後にも同じ状態へ到達するか、既存ケースの契約をどう維持するかは本runnerの判断範囲外である。仕様/設計/作者/本体の担当へ返す。期待値変更でgreenにせず、ET1最終判定はCHANGES_REQUIREDである。

FAIL後の収集wrapper調整で構文エラーexit1が2回発生したが、いずれもEXE起動前である。構文を修正して既存ログを再照合し、未実行集合だけ再開した。生FAILログは保持した。これらも製品FAIL件数へ加算しない。

### 残る4件と契約判断の分離

| 試用でFAIL・通常でPASSの実名（mediaは `app::chat_media::i2_export_media::` prefix） | 今回の最初の失敗・根拠 |
| --- | --- |
| m05_failed_default_storage_open_keeps_the_attempted_default_root_protected | chat_media.rs:1801 `fixture must establish real failed open`。fixtureはlocal/WordWeave5をStorage::atでlockし、Storage::open()のErrを要求したが成功した。試用namespaceにはlockを置いていない。failed-openの前提不成立であり、後続の拒否oracle未到達 |
| m05_missing_environment_and_unknown_root_identity_refuse | chat_media.rs:1331 `Audio accepted forbidden destination`、destは合成fixture/new。missing環境と通常名rootがfileである2副ケースのうちどちらで最初に失敗したかは生ログだけでは特定しない |
| m05_none_resolves_default_without_creating_root_then_protects_it | chat_media.rs:1331 `Audio accepted forbidden destination`、destは合成local/WordWeave5/new。通常名rootを保護対象とするassertの失敗 |
| h03_resolver_is_side_effect_free_and_tracks_environment_only_in_child | progress_export_destination.rs:580、実値はLocalAppDataMissing/WordWeave5-EarlyTrial、期待値は同/WordWeave5。namespace固定assertで失敗し、後続環境変更/欠落副ケースへ未到達 |

設定s06を含めて5件の原FAILを保持し、fixture修正や期待値変更、無効化、再実行は行っていない。mediaのfailed-open前提不成立1件を行動REDやPASSへ変換しない。media/destinationのnativeexitも101、timeoutfalse、終了wait済みである。各groupの全失敗実名・子outputは `trial-media-runtime.{stdout,stderr}.log` / `trial-destination-integration-runtime.{stdout,stderr}.log` にある。

親から独立reviewerの契約判定を受領した：「contract:92『登録前generic APIはcaller契約維持』、Storage::at/OnceLock未登録のs06 fixtureを照合し、製品漏れでなく通常namespace固定fixture非対応である」。これはrunner自身の動的証明ではなく受領したreview判断である。runnerの実測はwrite成功と拒否assert失敗までであり、他4件も本表の観測と前提を親へ返した。試用productionで登録前画面へ非到達であることを今回のFAIL/新35PASSから新たに証明しない。追加所有・作者修正・Pre/Post判断は親/planner/reviewerが担当する。

## 最終結果と返却

最終判定は **CHANGES_REQUIRED**。通常はraw253PASS/0FAIL/0ignored → 行動250PASS/0FAIL/fixtureSKIP3。試用は新35全PASS＋既存関連244PASS/5FAILでraw279PASS/5FAIL/0ignored → 行動276PASS/5FAIL/fixtureSKIP3である。試用の選択実名は284件、通常253件、各構成内で互いに重複なし。新36宣言・共通3名・通常4/試用35という構成差を39uniqueや構成横断uniqueへ合算しない。

試用のfixtureSKIP3は通常と同じsymlink2件（1314）と別drive alias1件（不在）。各構成のstderrで今回の発生を確認した。試用short-name aliasも今回確認した。旧昇格log/mocksをnative結果に再利用していない。

通常8list/18runtimeは各nativeexit0。試用8listは各0、19runtime中settings/media/destinationの3groupだけnativeexit101、残り16groupは0。すべてAdminFalse、timeoutfalse、終了wait済み、kill0。失敗も含めて結果実名と選択実名の集合一致を照合した。試用最終shell収集はexit0だが、これは全試験成功を意味しない。試用確定集計は `trial-results-complete.log`、初回停止前は `trial-results.log`、通常は `default-results.log` に保存した。試用settingsのexit101は `trial-native-summary.log` にあり、再開時にその済みログを読んだため `trial-failures.log` には後続2groupだけがある。

2026-10-14 10:41:45.688 JSTの最終read-only確認で、通常/試用16固定EXEのhash一致・同path process残存0、tracked source/test/manifest/lock **91hash一致**を確認した（`final-process-exe-check.log` / `source-after-summary.log`）。両構成とも合成TEMP/LOCAL直下残存0。HEADは開始時と同じ `30fc43c074e21cf5b989722dce4fb48c08cddbcf`、`git diff HEAD -- Cargo.toml Cargo.lock src tests` は空であり、5source/3作者prefixも固定一致である。親が途中追記した実装報告/current/usage/reviewを保持した。今回の恒久編集は本書だけである。

EXE path/SHA256は今回の `default-manifest.json` / `trial-manifest.json` を正本とする。build実行sessionは89385/55534、通常runtime1347（収集停止）/36683、試用runtime13002（設定FAILで停止）/40048（残り完了）。再開で成功済みEXEを再実行していない。実行開始は通常build log作成の10:23:57頃、終了境界は上記最終hash確認までである。終了usage counter・実行model/effort metadata・親子包含関係は未取得であり、使用量や週枠への換算は行わない。

新35/通常4のfocused受入は同じ最終sourceで独立PASSしたが、既存回帰5FAILによりET1全体の受入完了・Post PASSを主張しない。親へ各FAILを返し、source/hash固定の所有を返却する。保存/回復/書込失敗/再試行/保持は合成データの局所証拠であり、電源断、敵対的TOCTOU、single-call途中のstage競争全体の保証ではない。

未検証はET2の実起動/資格情報namespace/画面非到達、ET3の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`、試用全体test/releaseと関連crate gate、native直接起動/IME/マイク/ペン/TTS/実Codex・Qwen認証/API/拒否dialog/DPIである。今回のtest EXEによるOS fixture検査をそれらの実機受入へ読み替えない。実data、実credential、実API、OS設定変更、ユーザーapp停止、製品EXE置換、publishは0である。

## ET1-FX1 独立再検証（最新、2026-10-14）

同じrunner `gpt-6.1-sol/medium`、同thread/sessionで継続した。wordweave-changeは既読の検証・報告規約だけを継続する。実行model/effort metadataと追加工程の開始usage counterは未取得である。入力はplan末尾ET1-FX1、Pre-1 PASS、作者tests.md段階5であり、plan SHA256 `3E698E4B6CC2A0778AC12B4B5C2DC012A47381FCC1365134CFE9A7B46A591312` の一致を確認した。親から作者のsource所有返却・全実行終了を受領してから開始した。

対象はHEAD `bb01b5869c7e9b22af07a4a4593fae708b38ca60`＋未commitの次3test-file差分である。HEADは文書checkpointであり、本体/依存/manifest/lockは旧runner基準から不変。旧91hashのうち88件一致、更新3件は以下に一致した。差分は46追加/13削除、所有5symbolに限り、全文diffを `fixture-fix-source.diff.log` に保存した。

| 変更test file | 固定SHA256 |
| --- | --- |
| src/app/settings_ui.rs | 021FB7F28501512AC2083AFA371260E5572031C491CB4FF0E057C0E685BBC32C |
| src/app/chat_media.rs | 1D50424433A8F37A1962E439A34F34B8851C99F5CDA45EEAA3B74D1BB8A6BF3B |
| tests/progress_export_destination.rs | B77396A23A1523437E8C7865799AE9740426F12ECA4FAD8017597065CDA7F9CF |

独立のread-only比較で、settings S06、media M05A/B/C、integration H03の関数範囲を現sourceと `git show 30fc43c:<path>` の双方から除外し、残り全文をLF正規化して比較した。3fileとも厳密一致した。shared helper/production/他testを含み、期待rootのfeature別literal、resolver照合、M05B名前付き前提以外の変更はない。新開始snapshotは `fixture-fix-source-before.log`、88/3の確認は `fixture-fix-source-check.log`。従来store/commit/ai prefixとET1新36宣言は不変である。

### 新規影響集合のcompile/runtime

両構成を直列に次でfresh no-runする。試用だけ `--features early-trial` を加える。既知sandbox OS5は再試行せず、確定JSONのtest executableから各2EXEを同定し、新SHAを固定する。

```text
cargo test --locked --bin wordweave5 --test progress_export_destination --no-run --message-format=json
```

通常compileはexit0、2m21.943s。新bin `--list` は322件、integrationは29件、各nativeexit0。旧5実名の存在を確認後、settings25、media23、既定fatal-close5filter各1、ai1、destination integration29を実行した。各filterに `--nocapture --test-threads=1`、integrationは空filterを渡した。選択実名/成功実名/summaryの一致を確認し、重複なしraw83PASS/0FAIL/0ignored。symlink2件OS1314とalternate-drive alias不在1件を今回stderrで確認し、行動80PASS/fixtureSKIP3である。short-name aliasも今回確認した。旧logsからskip発生を推測していない。

通常2list/9runtimeはnativeexit0、AdminFalse、timeoutFalse、終了wait済み、kill0。各起動前のWindowsIdentity/WindowsPrincipal取得成功・Administrator=False、固定SHA、Hidden、120秒上限、timeout時Kill(true)/WaitForExitを維持した。合成TEMP/TMP/LOCALAPPDATAだけを使用し、終了時TEMP/LOCAL/テストEXE process残存0、91source hash一致を確認した。

全新証拠はignored `target/early-trial-build/et1-runner/fixture-fix-*`。構成別build.jsonl/build.stderr.log/build-summary.log、manifest.json、2target list、group selected/runtime stdout/stderr、native-summary.log、results.log、completion.logに保存した。旧5FAILログは上書きしない。試用構成の独立確認と最終接合は次の追記で確定する。

### FX1 最終結果・証拠接合

試用fresh compileはexit0、2m28.151s。実コマンドは上記通常コマンドへ `--features early-trial` を追加した次である。

```text
cargo test --locked --features early-trial --bin wordweave5 --test progress_export_destination --no-run --message-format=json
```

今回の試用listもbin322/integration29、各exit0。同じ9runtimeのsettings25/media23/fatal-close5/ai1/destination29はすべてPASSで、選択と成功の83実名集合は一致した。raw **83PASS/0FAIL/0ignored**、行動 **80PASS/0FAIL/fixtureSKIP3**である。skip理由は今回もsymlink2件OS1314とalternate-drive alias不在1件。short-name alias成功は今回stderrにある。各構成で同じ83名を実行したものであり、166uniqueとは数えない。

| 変更対象・旧実名（省略prefixは上節） | 旧通常 / 旧試用 | 今回通常 / 今回試用 |
| --- | --- | --- |
| S06 storage_none_uses_resolved_root_without_opening_or_creating_it | PASS / FAIL | PASS / PASS |
| M05A none_resolves_default_without_creating_root_then_protects_it | PASS / FAIL | PASS / PASS |
| M05B missing_environment_and_unknown_root_identity_refuse | PASS / FAIL | PASS / PASS |
| M05C failed_default_storage_open_keeps_the_attempted_default_root_protected | PASS / 前提FAIL | PASS / PASS |
| H03 resolver_is_side_effect_free_and_tracks_environment_only_in_child | PASS / FAIL | PASS / PASS |

旧5名は新bin/integrationの全list内にあり、設定・媒体・integrationのgroup選択と成功実名でも照合した。M05Bのmissing環境resolver Err、file分岐のresolver==独立literal rootとroot.is_fileという名前付き前提assertを含み、両分岐の3媒体拒否・ledger/memory不変まで通過した。これらの個別成功markerを追加出力したわけではなく、無条件の直列assert/loopを保持した同caseの完了PASSが到達根拠である。

M05Cでは各構成literalとresolverの一致後、その同rootを実Storage::atで保持し、Storage::open()の実Err、Storage=None、3媒体拒否/不変までPASSした。mockによるErr差替えはない。旧通常root lockで試用rootをopenしていた前提不成立は解消した。ErrのOScodeを今回ログに追加取得しておらず、特定のWin32 codeまで実測したとは主張しない。S06/M05Aの外部成功・root未作成・内部拒否・保持、H03の2環境literal/全Err文言/snapshot不変も全保持している。

| 今回runner固定EXE（target/debug/deps/） | SHA256 |
| --- | --- |
| 通常 wordweave5-72e8adffa8f9ef2c.exe | 2B94BAC1889E553CEC61F839C6B529C2E568477C991D667B2C0D08547ADB3F57 |
| 通常 progress_export_destination-b4a366b351b447cc.exe | 68B3E5BD3F8E07CD98E251B497ECAAE2B01E2FDC359356E356525B840F29C543 |
| 試用 wordweave5-e0ab9f88db3adadd.exe | 2DCE5268F78BD57776BE39EB27CFB5ADEBFD8C57C05712996266A4C20A7FF50D |
| 試用 progress_export_destination-8431c9241084d920.exe | 127232B7E3F82E9AD0843BE586B98D7004EA14CFA4974E030BB1A45BB664DCE9 |

両構成それぞれ2list/9runtimeのnativeexitはすべて0。全起動AdminFalse・timeoutFalse・wait済み、kill0。両構成ともTEMP/LOCAL/テストEXE残存0、91source hash一致。2026-10-14 **11:11:19.250 JST** の最終read-only確認でも91source/4固定EXE SHA一致・同path process残存0（`fixture-fix-final-check.log`）。HEADはbb01b58のまま、source差分は上記未commit3file/5symbolだけである。親/作者のdocs dirtyは保持した。

最新の必要範囲は以下の接合で評価する。単一の新全件実行と表現しない。

| 構成 | 今回の影響集合 | source/依存不変で継承した旧集合 | 接合後raw / 行動 |
| --- | --- | --- | --- |
| 通常 | raw83PASS、行動80PASS/skip3 | 170PASS | raw253PASS/0FAIL、行動250PASS/0FAIL/skip3 |
| 試用 | raw83PASS、行動80PASS/skip3 | 201PASS | raw284PASS/0FAIL、行動281PASS/0FAIL/skip3 |

旧通常253・旧試用284の実名集合から影響83をそれぞれ除いた170/201件を接合した。本体/依存/manifest/lockは不変、変更3fileの5symbol外は独立全文一致、残る88対象も一致する。計画FX1-2の明示境界に従い、未変更lib/他integrationを理由なく反復していない。新通常4/試用35は、今回ai1と旧不変integration3またはlib20/integration14の証拠を接合した結果である。旧trial5FAILを残数へ加算したり、旧PASSを今回fresh全件PASSへ言い換えたりしない。

FX1のrunner範囲は **PASS** であり、独立Postへ返却する。findingのClosed、ET1全体のPost PASSはrunnerが決定しない。旧FAIL・作者E0433・前回収集wrapper不備のログは保持し、今回FX1 runnerではcompile/runtime/収集失敗0、source/期待値編集0、再委譲/commit/push/publish0、実data/鍵/API/release/ユーザーapp停止0である。

build sessionは通常21409/試用39463、runtimeは通常6183/試用74210。thread/sessionは初回と同じ、追加工程の開始/終了usage counter・実行model/effort metadata・親子包含関係は未取得。推定や合算をしない。実行時間は各build summaryと各native-summaryに保持した。ET2、ET3の通常/試用all-targets・release・関連crate gate、native起動/表示/IME/マイク/ペン/TTS/実認証/API受入は依然未実施であり、今回の局所合成検証を代替証拠にしない。
