# ET1 独立runner

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
