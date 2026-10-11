# I5-CANCEL-D1 fixture自己試験引継ぎ

最新の固定対象は末尾「安全review cycle 1追補の修正・再検証」のR2である。先行ログ・SHA・結果は履歴として維持する。

2026-10-07 / ACOUSTIC-HISTORY-001 / test author。完了境界は隔離fixture作成・compile・D1-A～E自己試験までである。全unit再現、製品取消し修正、M2 Post判定、変更後EXEのM1受入は未実施である。

## 入力・所有

親指定はI5 `gpt-6-astra/high`（implementation.mdのI5割当）。実行model/effort metadata、開始/終了token counterは未取得。AGENTS、development-team、KPI、agent-skills:test-driven-developmentを読了し、test/RED責務に適用した。承認条件はAH-AC09とprocess-cancel-diagnostic-design.mdのD1-A～E、cycle2 Pre PASS、設計SHA256 `4ADA492BCC842DF786E314943DF65E2B5DDCF3F8C23C4C77ABF865A73CA17B99`である。

原本のsrc/tests/scripts/Cargo.toml/Cargo.lock計17ファイルを新規 `artifacts/cancel-d1-crate/` へbytecopyした。元targetはコピーせず、copy内targetへビルドした。開始inventoryは `artifacts/cancel-d1-source-inventory.json`、終了照合は `artifacts/cancel-d1-final-source-inventory.json`。原本17ファイルの変更0、copyの変更は割当2fixtureのみである。copy内src/manifest/lock/scriptsは原本と一致する。他者の編集・親の文書・原本snapshotは変更していない。

| 最終成果物 | SHA256 |
| --- | --- |
| copy tests/support/process_unit.rs | `2AC50B94BC76B92FE965161A6EA44CA42DFEB0EF2B2FCDF69F2D6742BC68E338` |
| copy tests/support/praat_child.rs | `A85971AC95A7883239E292FEFB25DF7F7C8BC6461B584946091DC78E4E2A1C9B` |
| copy target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe | `50D22D67F2F980AF9954EB4F8A28D79CF2940E1B9A69082E31B68765C34B8ABA` |
| copy target/debug/acoustic-test-child.exe | `FC2C80721E1E5B5917650868E8521AB2273B29118482B4C74FBEC6EC2A43F684` |

バイナリhashの機械可読記録は `artifacts/cancel-d1-exe-hashes.json`。M1の `// Invoked only by the exact regression test` 以降は改行コード正規化後に原本と完全一致する。既存M1のoracle不変と、変更後copy EXEのM1未受入を区別する。

## 診断fixtureの内容

所有handleには世代、role/origin、通知PID、実PID/creation FILETIME、取得sequence、不可逆first_waitを保持する。両observerは共通の256bytes decoderで完全identityとraw PID全bytesを照合してからOpenProcessし、handleのPID/creationを再照合する。不一致handleはcloseのみ。publisherは所有Child実handle由来のidentityをcreate_new/write_all/flush/close/renameし、その後に従来のPID fs::writeを行う。

run返却直後の0ms waitを採取してからログ・assertを行う。Result(kind/reason)/panic、reap前後、ready、poison、watchdogを別のcall evidenceとして保持する。救済前後のwaitとTerminateProcess結果は別recordであり、初回値を置き換えない。registry poisonは回復して失敗証拠へ固定し、Dropのlock/回収/closeは再panicしない。既存2/4/8/15秒とDrop救済2秒を維持した。root-only caseへ孫必須を追加していない。

## REDと環境失敗

旧 `artifacts/m2-runner-praat.log` の取消しcase初回WAIT_TIMEOUT=258とmutex保持assert→Drop PoisonError→abortを既存RED証拠とする。危険な旧fixtureは再実行していない。D1-Dは合成live childのWAIT_TIMEOUTを実観測して救済後も失敗が残ることの自己試験であり、製品取消しfailureの再現ではない。

初回sandbox内自己試験は8 PASS/1 FAIL（`artifacts/cancel-d1-selftests.log`）。追加した実publisher自己試験が通知未達となった。child stderr採取を追加したexact診断（`artifacts/cancel-d1-publisher-diagnose.log`）で、identity publishのfs::renameがWindows code5 PermissionDeniedを返し、child exit101となったことを確認した。独立したsandbox一時空fileのrenameも同じ拒否であった。これはfixture環境の失敗であり、製品cancel REDではない。

親の明示許可後、同じrename方式・期限・期待値で非管理者sandbox外の9自己試験を実行し全PASSした。UAC/OS設定/恒久環境は変更していない。追加したstderrは合成publisher自己試験だけの診断であり、製品stdout/stderrを記録しない。sandboxの失敗ログを保持し、単純再試行による成功として扱わない。

## 固定した自己試験9件 / AH-AC09

全名のprefixは `process_unit::` である。

| ID | 名前 | 独立期待値 |
| --- | --- | --- |
| D1-A | d1_a_complete_notification_requires_all_bytes_and_current_root | 一桁prefix/空/余剰/途中frame/root・creation不一致/上限超過を拒否、OpenProcess試行0・所有登録0・terminate0。手書き完全既知frameだけ受理 |
| D1-A | d1_a_atomic_publisher_registers_owned_tree_with_matching_identity | 実合成root/childのpublishを読み、通知と実handle identityの一致、世代/role、所有回収を確認 |
| D1-B | d1_b_creation_mismatch_closes_unknown_handle_without_termination | 自作childのcreationだけ不一致として拒否、handle-count不増、child生存・terminate0 |
| D1-C | d1_c_registry_poison_drop_is_nonpanicking_and_reaps_in_outer_process | 小process内で意図的poisonをcatchしDrop再panicなし、child終了、close1、poison失敗保持、外側exit0とcleanup marker |
| D1-D | d1_d_first_live_timeout_remains_failure_after_successful_rescue | 実live child初回258→救済後0、初回258と不合格保持、close1 |
| D1-E | d1_e_two_runs_duplicate_tracks_preserve_first_observation_and_close_once | 同root5独立handle＋次run1、世代1/2・過去first_wait維持・root-only受理・旧root通知拒否・close6 |
| D1-E | d1_e_record_capacity_fails_without_evicting_owned_handles | 16収容・17個目失敗、所有16を保持し回収、新handleもcloseしてclose17 |
| D1-E | d1_e_generation_capacity_fails_and_still_reaps_owned_child | 世代16収容・17拒否、所有child回収・close1 |
| D1-E | d1_e_event_capacity_fails_without_discarding_first_evidence | event256収容・257拒否、先頭証拠維持、所有child回収・close1 |

非sandbox `artifacts/cancel-d1-selftests-unsandboxed.log`: 9 PASS、0 FAIL、69 filtered、0.27秒。D1-Cの内側小processは同じcaseの構成要素であり、追加の合格件数に数えない。

固定前にD1-A自己試験2件だけを調整した。手書き不正identityを試すcaseはwatchdogを開始直後に停止・joinして不正bytesのOpenProcess試行0を決定的にし、実publisherのcaseは登録完了後・snapshot前にwatchdogを停止・joinしてclose件数の採取競合を除いた。本番取消しcaseのwatchdog仕様は不変である。最終EXEの `artifacts/cancel-d1-selftests-frozen.log` は9 PASS、0 FAIL、69 filtered、0.32秒。これを最終自己試験結果とする。

D1-E追加の既存caseは期待値無変更で以下を確認した。

- `repeated_success_and_injected_assignment_failure_do_not_leak_handles`: 1 PASS、0.53秒。warmup後12反復/24 calls、handle許容+2を維持。`artifacts/cancel-d1-repeat.log`。
- `missing_executable_and_embedded_nul_fail_without_running_child`: 1 PASS、0.01秒。起動前拒否2回、子起動なし。`artifacts/cancel-d1-prereject.log`。

既存2caseは自己試験2件だけの最終調整前EXE（SHA `3060C345F3A3F657AB9FB21266F1B3D094F706D004086E9E5091FA79109A7A6D`）での実行であり、そのcase・共有Fixture・本体は後の調整で変えていない。無関係な再実行は行っていない。

compile/build成功。compile最終ログは `artifacts/cancel-d1-compile-freeze.log`、child最終buildは `artifacts/cancel-d1-build-final.log`。警告は本体の既存dead_code、診断用構造のDebug経由読出しに対するdead_code、sandbox内incremental hardlink不可によるcopy fallbackであり、compile errorはない。

## コマンドとrunner境界

repo rootをcwdとし、最終compileは以下である。

```text
cargo build --manifest-path tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate/Cargo.toml --features praat --bin acoustic-test-child --locked --offline
cargo test --manifest-path tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate/Cargo.toml --features praat --lib --no-run --locked --offline
```

上記unit EXEに次の引数を個別に渡した（非管理者sandbox外、stderr/stdoutを各専用logへredirect）。

```text
d1_ --test-threads=1 --nocapture
--exact process_unit::repeated_success_and_injected_assignment_failure_do_not_leak_handles --test-threads=1 --nocapture
--exact process_unit::missing_executable_and_embedded_nul_fail_without_running_child --test-threads=1 --nocapture
--list
```

一覧は `artifacts/cancel-d1-test-list.log`: 78 tests、0 benchmarks。次の全unitは**独立安全review PASS後のrunner専任**である。M1 configured skip1なら実行予定77件（M2 15件を含む）、このauthorでは実行していない。外側180秒打切り、内部期限不変、1回のみという設計を維持する。`WW_TEST_PRAAT_EXE`は設計指定の絶対path/size/SHAを確認して当該processへ設定する。自己試験ではM2を実行せず、実Praatも使用していない。

## cleanup / 残件

自己試験の意図的なlive child救済は期待された自己試験動作であり、製品caseの成功へ転用しない。D1-C/D/Eのclose数、D1-Aの全rescue wait0、D1-Dの初回258保持、既存24callの漏洩上限を確認した。8秒watchdog作動は観測していない（自己試験全体0.27秒、既存試験0.53秒/0.01秒）。

`artifacts/cancel-d1-cleanup.log`でadministrator=False、非sandbox temp root=`F:\TempCodex\`を記録した。同prefixの残存dir5件は最終更新21:28–22:29であり、本authorのcopy作成・試験開始23時台より前のものである。今回時刻の同prefix残存は観測されず、未知の5件は削除していない。sandbox rename probeの一意pathは後の確認で既に不存在であった。失敗したsandbox publisherが通知前に生成した孫はhandle未取得であり、15秒fallbackを維持したが、その個体の終了を独立handleで確認できていない。この欠測を今回の所有handle回収PASSで埋めない。

残件は独立安全review、指定runnerによる全unit1回、同定済み失敗時のD2要否判断である。D1未再現でも過去のfailureを否定しない。原本へのfixture採用、M1再受入、M2 Post、fmt既存課題は親へ残す。再委任・実データ・API・公開・Git変更操作は行っていない。

## 安全review cycle 1追補の修正・再検証（R2、最新固定）

親から§12追補の差分Pre PASSと実装許可を受けた。正本SHA256は `591DFF53337775E1510C2D28C417F81D5313BB81F4F517F9B632C878894D1457`、割当は引き続き `gpt-6-astra/high`、取得不能な実行metadata/使用量counterは未取得である。変更は隔離process_unit.rsだけであり、前回praat_child.rs/child EXEは不変である。

修正前REDを独立した2件で記録した。

- `artifacts/cancel-d1-r2-seal-red.log`: snapshot採取後のobserver登録が `Ok(HANDLE)` になり、期待`RegistrationSealed`に反して1 FAIL。compile成功後の実behavior failureである。
- `artifacts/cancel-d1-r2-cleanup-red.log`: 意図的watchdog thread panicをjoinしたDropが、保持すべき合成tempを削除して1 FAIL。thread panic自体ではなく、残存path/bytesの期待違反がREDである。

修正内容は以下である。

- first_waitsの短mutex内で世代sealとsnapshotを同時に確定する。解放後に従来の0ms waitを行い、前にjoin/sleep/追加waitを挟まない。observerはOpen前と挿入lock内で世代/root/sealを確認し、後発取得は`RegistrationSealed`でclose-onlyとする。次世代のbeginはchecked incrementと同じlock内でsealを開く。run外の明示trackは維持した。
- cleanup確証をpoison/容量超過から分離したbit集合に保持する。救済wait非signal、watchdog panic、reap_unconfirmed、tree/tree-exit作成観測後の孫未同定があればTempDir自動削除を止め、絶対pathと固定reasonをfallibleログへ残す。tree作成から実行までの不確実な区間も保守的に保持対象とする。poisonだけで所有範囲・終了確認が揃う場合は削除でき、既存D1-Cがその対照となる。
- cleanupの失敗注入はfixture内の結果判定境界に限定し、実OSのterminate失敗・未知process・権限変更を用いていない。保持を確認した後、当該自己試験自身の一意tempだけを通常削除する。

新規5件（prefix `process_unit::`、AH-AC09 / D1 §12）：

| 名前 | 独立期待値 |
| --- | --- |
| d1_r_sealed_observer_makes_no_new_open_and_preserves_snapshot | seal後Open試行不増、record数・初回観測不変、terminate0 |
| d1_r_open_inflight_at_seal_is_closed_once_without_register_or_terminate | 実自作child handleをOpen済み→seal→挿入の決定的境界へ通し、未登録handle close1、既存record保持、child生存、terminate0 |
| d1_r_new_generation_rejects_old_inflight_and_accepts_new_root | 次世代root受付、旧世代の取得中handle拒否、初回値保持、全取得handleのclose3 |
| d1_r_watchdog_panic_retains_owned_temp_and_bytes | 実thread panicとjoin後に専用temp・合成bytesを保持、確認後に自己所有tempを回収 |
| d1_r_cleanup_failure_observations_retain_temp_at_policy_boundary | WAIT_TIMEOUT/WAIT_FAILEDの救済結果境界、reap未確認境界、tree孫未同定の4条件で保持・bytes不変、Open/terminate0 |

最終自己試験は**14件（既存9＋新規5）全PASS、0 FAIL、69 filtered、0.36秒**。ログは `artifacts/cancel-d1-r2-selftests.log`。新規seal試験はsleepで競合確率を上げる方法を用いていない。自己試験の期待された救済・保持を、製品caseの回収成功へ転用しない。

共有Fixtureを変更したため、同一最終EXEで既存24callを1 PASS（0.55秒、`artifacts/cancel-d1-r2-repeat.log`）、起動前拒否2回を1 PASS（0.01秒、`artifacts/cancel-d1-r2-prereject.log`）として再確認した。実行は親許可済み非管理者sandbox外であり、administrator=Falseを記録した。compileは `artifacts/cancel-d1-r2-compile.log` に成功記録、割当fixture2本のrustfmt --checkも成功。既存本体のfmt gateを閉じた意味ではない。

最新hashは以下で固定する。前回JSON/log/hashは上書きしていない。

| R2成果物 | SHA256 |
| --- | --- |
| copy tests/support/process_unit.rs | `4A5BCB21166F1AACAE17F9A447EF0D738FEAAADC8DC0CB3A9EE2654327E91EEC` |
| copy tests/support/praat_child.rs | `A85971AC95A7883239E292FEFB25DF7F7C8BC6461B584946091DC78E4E2A1C9B` |
| unit EXE（上記同path） | `7F37ABED714DE2F81A535494024D733AABE9FA8ED7DCCE5DF9BAFAE7995C66BC` |
| child EXE（上記同path、不変） | `FC2C80721E1E5B5917650868E8521AB2273B29118482B4C74FBEC6EC2A43F684` |

原本17件再照合は `artifacts/cancel-d1-r2-source-inventory.json`、EXEは `artifacts/cancel-d1-r2-exe-hashes.json`。原本変更0、copy差分2fixtureのみ。M2入りsrc/praat.rsも原本一致であり、既存M2 15件を変更していない。元cancelのCancelled/descendant_ready/兄弟生存/保存bytesの末尾assert群は原本と完全一致する（改行正規化）。ready2秒・deadline4秒も不変。cancel observer取得方式はD1既承認の完全identity照合であり、R2で元cancel case本文は変更していない。M1 oracle suffixも原本一致である。

最新一覧は `artifacts/cancel-d1-r2-test-list.log` の**83 tests、0 benchmarks**。次の全unitはM1 configured skip1で82件予定、既存M2 15件と元cancelを含む。compile済み当該EXEのhashを確認し、設計どおり固定原本Praatの絶対path/size/SHAを検証して環境変数を明示する。全unit・製品cancel exactはR2でも未実行であり、**再安全review PASS後にrunnerへ渡す**。

cleanup再確認は `artifacts/cancel-d1-r2-cleanup.log`。既存5ディレクトリの名前・時刻は前回と同一、増分0であり、未知dirは削除していない。自己試験で保持確認したtempはその試験内で回収した。元のsandbox publisher孫の独立handle欠測という前回限界は変わらない。R2の成功を元source/変更後M1の受入・製品取消し原因解決・M2 Post PASSとは呼ばない。
