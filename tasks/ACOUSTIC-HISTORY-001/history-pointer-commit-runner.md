# I2a-P 独立runner結果

2026-10-11 / ACOUSTIC-HISTORY-001。planner指定 `gpt-6.1-sol / medium`（implementation.md I2/Pとモデル割当表）に従う独立runnerである。起動指定と実行値を区別し、実行metadata・開始終了usage counterは未取得である。AGENTS.md、docs/process/development-team.md、KPI、wordweave-change全文を読み、保存/障害/保全の検証・結果報告だけに適用した。入力はP最終Pre、新22行動RED、P本体静的阻止0と自己22 GREEN、実装・試験・レビュー最新節、親の固定EXE handoffである。

完了境界は固定source/EXE上の関連回帰の独立実行と失敗報告であり、最終Post/利用者受入は判定しない。**初回79件で4失敗となったため停止した。75 PASS / 4 FAIL / 0 ignored / timeout 0、残190件未実行である。** 再試行、ソース・テスト・期待値・設定変更は行っていない。

## 対象固定

HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。dirty/untracked差分が存在するためHEAD単独を試験版とは称しない。正確な16 source/manifestと9 EXEは実装者の固定manifestに従い、開始・終了の現物SHA-256一致25/25、不一致0であった。

- [source固定manifest](artifacts/history-pointer-commit-implementation-source-fixed.json)、[EXE固定manifest](artifacts/history-pointer-commit-implementation-exes-fixed.json)
- [開始現物照合](artifacts/history-pointer-commit-runner-integrity-start.json)、[終了現物照合](artifacts/history-pointer-commit-runner-integrity-end.json)
- [HEAD](artifacts/history-pointer-commit-runner-revision.log)、[開始status](artifacts/history-pointer-commit-runner-status-start.log)、[終了status](artifacts/history-pointer-commit-runner-status-end.log)、[tracked diff概要](artifacts/history-pointer-commit-runner-diff-stat.log)

store全文SHA-256は `FA9ECA988D4EDF5233E0198F8B2E090F760098D16779001179CBFFF151F38BEB`、lib EXEは `ECB98C937815BF2ADC73DB53BD34EDE87A4A365C2A328327D3FB65A4B1AF4FB1`。source/buildは常時読取であり所有を取得していない。新buildは行っていない。

## コマンドと実数

9固定EXEそれぞれに `--list` を渡し、全てexit0・0件なしを確認した。lib一覧は279件、そのうち先頭prefixを限定した重複なし関連162件である。対象名全件・実行引数・外側上限は [jobs](artifacts/history-pointer-commit-runner-jobs.json)、一覧原logは `artifacts/history-pointer-commit-runner-list-*.log` に保存した。

| 対象 | 固定件数 | 今回結果 | 外側上限 |
| --- | ---: | --- | ---: |
| lib store:: | 79 | 75 PASS / 4 FAIL、exit101 | 300秒 |
| lib commit:: | 53 | 未実行 | 300秒 |
| lib audio_history:: | 30 | 未実行 | 300秒 |
| audio_history_commit | 4 | 未実行 | 300秒 |
| audio_history_gate | 16 | 未実行 | 300秒 |
| audio_history_save | 3 | 未実行 | 300秒 |
| audio_history_material | 6 | 未実行 | 300秒 |
| audio_history_progress | 4 | 未実行 | 300秒 |
| audio_history_migration | 3 | 未実行 | 300秒 |
| audio_history_store | 37 | 未実行 | 300秒 |
| audio_history_recovery | 34 | 未実行 | 900秒 |

実行した行動commandは次の1回だけである。

```text
F:\Kazuhiro\GitHub\WordWeave5\target\debug\deps\wordweave5-b3388a7bbef44bb4.exe store:: --nocapture --test-threads=1 --skip audio_history:: --skip commit::
```

2026-10-11 16:38:31–16:38:52 JST、wrapper実測21.018秒、libtest実測20.89秒。75 passed / 4 failed / 0 ignored / 0 measured / 200 filtered out、exit101、外側timeoutなし。新P18件は全PASS、旧store群61件は57 PASS/4 FAILである。子process内部件数は加算していない。269件中79件実行、残190件はskip/ignoredへ加算せず未実行とする。新22件全成功を今回の独立結果として主張しない。

[結果JSON](artifacts/history-pointer-commit-runner-results.json)、[stdout原log](artifacts/history-pointer-commit-runner-lib-store.stdout.log)、[stderr原log](artifacts/history-pointer-commit-runner-lib-store.stderr.log)、[wrapper全文](artifacts/history-pointer-commit-runner-wrapper.log) を保持した。wrapper呼出command自体は例外によるexit1で停止した。製品60秒/512MiB上限とrunner外側上限は別であり、自動延長していない。

## 失敗の分類と返却

| case（store::以下） | 今回の証拠 | 判定限界 |
| --- | --- | --- |
| i2_gate_tests::block_applies_to_existing_guard_under_guard_and_all_writer_entries | 子process exit101、src/store.rs:1985のHistoryWriter::publishでHistoryError Io。親側1885で子失敗を報告 | write-block導入前のpublishで停止。Io根本原因・P回帰とは断定できない |
| i2_migration_tests::m03_wrong_types_and_real_junctions_are_rejected_before_backup | src/store.rs:1794、junction fixture setup failed、OS error5 | DeviceIoControlのjunction設定が0を返したfixture不成立。該当junction拒否の行動REDではない |
| i2_normal_save_tests::n03b_daily_reparse_is_rejected_and_existing_regular_daily_is_not_replaced | 同1794、同OS error5 | 同上 |
| i2_progress_tests::s03_unsafe_and_missing_paths_never_become_empty_history | 同1794、同OS error5 | 同上 |

3件は実source上DeviceIoControlによるjunction設定の失敗位置まで特定した。GのIoにはraw OS番号がなく、同一原因へまとめない。今回はcommand内の既存WW5_I2_GATE_LIB_ROOTとTEMP/TMPを同じworkspace所有rootへ向けた。過去のsandbox TEMP先行日別atomic OS5と今回のjunction OS5は操作が異なる。過去P自己22 GREENはjunction/G旧caseを含まず、今回旧回帰の環境成功対照とはならない。OS/ACL/UAC/harnessを変更せず、原因確定のための再試行もしていない。

親へ失敗位置・原log・未実行190件を返した。追加診断や修正は既定の別担当判断を要する。

## fixtureと残存

新規絶対path `F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\history-pointer-commit-runner-fixture-20261011` のcollision拒否・祖先/root reparseなしを検査した。同shellのtry/finally内で環境を限定し、終了後は元の環境へ戻した。

専用rootは開始0→store後0→終了0、追加/消失0、対象9 EXEの残存process0であった（temp JSONのItemsのnull要素は空列挙のPowerShell直列化であり、Countは0である）。空の自分rootだけを非再帰Remove-Itemで削除し、その後root不存在も確認した。全TEMP残存0を意味しない。[開始計測](artifacts/history-pointer-commit-runner-temp-start.json)、[store後](artifacts/history-pointer-commit-runner-temp-lib-store.json)、[終了計測](artifacts/history-pointer-commit-runner-temp-end.json)、[cleanup](artifacts/history-pointer-commit-runner-cleanup.log)。ユーザーのアプリやEXEを停止・置換していない。

## 未検証・handoff

残190件、独立最終Post、native UI/IME/マイク/ペン/TTS/実Codex認証、実API、微小な取消Acquire直後の窓、V内部最終再読間置換、pending metadata Io、実closure100k nodes、電源断/kill耐久性、root/祖先消失の全OSパターンは未検証である。recovery inventory100kも今回未実行であり、旧ログやP同期fixtureでは代替しない。今回は固定EXEの限定独立回帰でありWindowsアプリdeliveryではない。全target cargo test/release buildは後続I10のgateのまま残る。

開始/終了usage counterと実行model/effort metadataは取得不能であり推定なし。親の同一task KPIへ欠測として引き継ぐ。所有する報告・runnerログだけを残して返却し、再委任・実データ・実API・commit/push/publishは行っていない。

## 2026-10-11 独立runner再開194件

利用者のSol/medium切替連絡と親の再開handoffを受領し、planner指定 `gpt-6.1-sol / medium` を維持した。実行model/effort metadata・開始終了usage counterは未取得であり推計しない。AGENTS.md、development-team.md、KPI、wordweave-change全文、implementation.mdのP行・モデル表、上記初回記録とdiagnostic.mdを確認した。skillは保存/障害/保全の検証・結果報告に限定して適用した。完了境界は失敗4exact＋未実行190件の独立実行であり、次のAstra/high最終Post前で停止する。

**再開は194 PASS / 0 FAIL / 0 ignored / timeout 0 / 未実行0である。** 初回79件の75 PASS / 4 FAILは上記のまま保持し、親診断4 PASSを独立合格へ加算しない。初回成功75件と今回194件で当初269対象の各caseに成功証拠が揃ったが、269全件の同一環境・連続runではない。旧4 FAILの根本OS原因、Gの失敗syscallは未確定である。

### 固定対象と実行環境

HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`、dirty/untracked差分あり。試験版はHEAD単独ではなく、既存のsource16件・EXE9件の固定manifestを正とする。開始・終了の現物SHA-256は各25/25一致、不一致0である。新build、source・test・期待値・設定変更は行っていない。

- [開始hash](artifacts/history-pointer-commit-runner-resume-integrity-start.json)、[終了hash](artifacts/history-pointer-commit-runner-resume-integrity-end.json)、[revision](artifacts/history-pointer-commit-runner-resume-revision.log)、[開始status](artifacts/history-pointer-commit-runner-resume-status-start.log)、[終了status](artifacts/history-pointer-commit-runner-resume-status-end.log)、[diff概要](artifacts/history-pointer-commit-runner-resume-diff-stat.log)。正確なsource/EXE版は上記の実装者固定manifestを参照する。
- [194件の固定jobs](artifacts/history-pointer-commit-runner-resume-jobs.json)：元jobsのlib-storeを失敗4exactへ置き換え、その他190件のArgs/Names/Expected/Secondsは維持した。重複なし194件であり、全14jobに実引数＋`--list`を渡しexit0・0件なし・期待名集合一致を事前確認した。原list logは同prefixの`-list-*.log`である。
- exec `require_escalated`によるsandbox外実行である。wrapperがWindowsPrincipalのAdministrator tokenを拒否してから開始した。UAC/RunAs、ACL/OS/harness変更はない。TEMP/TMP/既存WW5_I2_GATE_LIB_ROOTだけを同shell内で新しいworkspace所有rootへ向け、finallyで復元した。ancestor reparse・root collision拒否を実施した。

### command・件数・exit

実行wrapperは [全文](artifacts/history-pointer-commit-runner-resume-wrapper.txt)、[原log](artifacts/history-pointer-commit-runner-resume-wrapper.log) に保存した。呼出commandは次である（exit0）。

```powershell
& ([scriptblock]::Create((Get-Content -LiteralPath 'tasks/ACOUSTIC-HISTORY-001/artifacts/history-pointer-commit-runner-resume-wrapper.txt' -Raw))) *> 'tasks/ACOUSTIC-HISTORY-001/artifacts/history-pointer-commit-runner-resume-wrapper.log'
```

各行動commandの実EXE絶対pathと引数は [results](artifacts/history-pointer-commit-runner-resume-results.json) のCommandに保存した。全行 `--nocapture --test-threads=1` である。4exactは `--exact <固定case名>`、commit/audio_historyは元prefix/skip、integrationは元全件Argsを維持した。外側上限は事前固定の4exact各60秒、通常各300秒、recovery900秒であり、自動延長・再試行なし。stdout最終summaryと実行名集合の両方を各jobのNamesへ照合し、14/14一致、全exit0である。

| job | PASS | FAIL / ignored / timeout | 外側上限秒 | 実測秒 |
| --- | ---: | --- | ---: | ---: |
| failed-exact-0（G） | 1 | 0 / 0 / 0 | 60 | 0.469 |
| failed-exact-1（M junction） | 1 | 0 / 0 / 0 | 60 | 0.261 |
| failed-exact-2（N junction） | 1 | 0 / 0 / 0 | 60 | 0.270 |
| failed-exact-3（S junction） | 1 | 0 / 0 / 0 | 60 | 0.256 |
| lib-commit | 53 | 0 / 0 / 0 | 300 | 18.203 |
| lib-audio_history | 30 | 0 / 0 / 0 | 300 | 4.763 |
| audio_history_commit | 4 | 0 / 0 / 0 | 300 | 0.964 |
| audio_history_gate | 16 | 0 / 0 / 0 | 300 | 5.285 |
| audio_history_save | 3 | 0 / 0 / 0 | 300 | 0.797 |
| audio_history_material | 6 | 0 / 0 / 0 | 300 | 2.476 |
| audio_history_progress | 4 | 0 / 0 / 0 | 300 | 0.448 |
| audio_history_migration | 3 | 0 / 0 / 0 | 300 | 1.021 |
| audio_history_store | 37 | 0 / 0 / 0 | 300 | 10.250 |
| audio_history_recovery | 34 | 0 / 0 / 0 | 900 | 426.754 |

行動実行と終了監査は18:57:42.994–19:05:38.210 JST。recovery内の100000/100001 inventory境界も後段assertまでPASSである。各jobの開始終了時刻・秒・filtered数とstdout/stderr原log pathはresultsに記録した。filteredは別caseの選択除外であり、失敗・ignored・未実行に加算しない。子process内部件数も加算しない。recovery中のtool waitは30秒以下、親へ短い継続報告を返した。

jobs SHA-256: `7C307549D6984A984D98E4A047B476BF7CD8965253643005054031AEE1B3DD32`。results SHA-256: `50BFE0666978259CD01E1A6C5C9AF2C1AA5BA6509248C09664BB6B25213A895C`。wrapper全文 SHA-256: `2C32457CBF701CFF6388F150A5FB268763EAAEC4FEE6DBF43C23C9F576FC4081`。

### 隔離・清掃・残受入

専用rootは `F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\history-pointer-commit-runner-resume-fixture-20261011`。同shell開始・各suite後・終了で専用rootの直下残存0、固定9EXEの対象process残存0を記録した（同prefix `-environment-*.json`）。空の今回所有rootだけを非再帰削除し、[final](artifacts/history-pointer-commit-runner-resume-final.json) で不存在・hash不一致0・194/194完了を確認した。旧失敗fixture/log、学習・録音・ink、ユーザーアプリを削除・停止・置換していない。

独立最終Post、native UI/IME/マイク/ペン/TTS/実Codex認証、実API、微小取消窓・V内部最終再読間置換・pending metadata Io・実closure100k nodes・電源断/kill耐久・全OS消失パターンの未検証は継続する。inventory100k成功とclosure100k未検証は別である。この限定回帰はWindowsアプリdeliveryではないため再buildせず、I10の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` は後続gateに残す。P完了・製品完成は主張しない。次はAstra/high最終Postへ渡すため利用者のモデル切替前で停止する。
