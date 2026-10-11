# CANCEL-A1 fixture採用・focused検証

最新固定は末尾「§13追補（A1-R2）」である。初版のsource/EXE・件数・公開後seal判定の記述は履歴として保持し、最新結果へ転用しない。

2026-10-08 / ACOUSTIC-HISTORY-001 / AH-AC09。test authorの完了境界は原本fixture2本の採用・ACK/ready・自己試験・tree-exit/cancel focusedまでである。sourceを固定して親へ返した。全unit、外側Job不在でのM1 exact、独立Postは未実施である。

## 入力・責務・固定

承認正本は `process-cancel-adoption-design.md`、Pre PASS、SHA256=`61500C17AC214258EA1B79F9D8AF98724FB1AEA7A1EE5B6CC036F470B3DB4122`。I5指定model/effort=`gpt-6-astra/high`、実行metadata/開始終了usage counterは未取得である。AGENTS、development-team、KPI、test-driven-developmentのテスト/RED責務を適用した。

所有は `tools/acoustic-analysis/tests/support/{process_unit.rs,praat_child.rs}` と本書、新規cancel-a1-*証拠だけである。親のlib.rs fmtとは逐次所有し、作者はlib.rsを編集していない。production process/M2/manifest/lock/script、D1隔離source、旧ログ・snapshot、未知tempは変更していない。再委任・実データ・API・UAC・公開なし。

R2採用前に原本2本、親snapshot、D1源泉のSHAを設計値と照合した。R2源泉を原本2fixtureへbyte採用し、採用後SHAも源泉と一致した。採用は機械的rewriteであり、その後のACK追加はapply_patchで行った。

| 段階 | 証拠 |
| --- | --- |
| 原本snapshot→R2採用 | `artifacts/cancel-a1-adoption-process.patch` / `cancel-a1-adoption-child.patch` |
| R2源泉→ACK/ready追加 | `artifacts/cancel-a1-ack-process.patch` / `cancel-a1-ack-child.patch` |
| 採用時source一覧 | `artifacts/cancel-a1-adopted-source-inventory.json` |
| 作者固定時の17ファイル照合 | `artifacts/cancel-a1-final-source-inventory.json` |

作者固定時の17件は、変更2fixtureだけである。M2を含むsrc/praat.rs、praat_unit.rs、本体process2本、manifest/lock/scriptは元hashと一致した。親による後続lib.rs fmtは別工程である。

| 作者固定対象 | SHA256 |
| --- | --- |
| process_unit.rs | `37E84E5D54FFA6BA29924ECDF7AE5E48DB08487DFD6391CFAC2348649B649F67` |
| praat_child.rs | `14EAF8EE8A55F040DD63989E82DD2E9F96A900ED4AEAECB8489082F1361ED356` |
| tools/acoustic-analysis/target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe | `19A66F49568731C0360F61261C13CF2D4C8375C755D775E2105EF71BB0A04C4E` |
| tools/acoustic-analysis/target/debug/acoustic-test-child.exe | `3AA26103A2A9BE1211900596647296C8C2C5DAFEECE81AF793DF779389FC5060` |

EXE記録は `artifacts/cancel-a1-author-exe-hashes.json`。これは**lib.rs fmt統合前の作者focused確認EXE**であり、最終M1対象EXEではない。runnerは統合後に再build・再固定する。

## 実装とoracle

R2の所有handle/creation同定、世代、seal、初回wait保持、poison非panic回収、cleanup未確認temp保持、14自己試験を採用した。tree-exit世代ではWatchdog孫をregistryへ挿入した後だけACKを準備する。保持root/孫を0msでlive確認し、世代/root/sealを再確認してACKをatomic publishする。publish後にseal/世代変化があれば`LateAckPublication`を固定し、後発ACKで成功へ戻さない。

childは自分と所有Childの実handle由来identityを用い、PID通知publish後に固定した2秒deadline内で完全一致ACKを読む。完成不正frameは即失敗、欠落だけを短くpollする。ACK受理後に孫liveを0ms確認してreadyをatomic publishし、孫をkill/waitせずrootがexit0する。欠落・不一致等はexit241である。tree以外/root-only/M1へACK待機は導入していない。

supervisor返却直後はseal＋全初回0ms waitを先に採取する。その後にACK/readyを読む。CallEvidenceはtree-exit必須性、exit code、ACKのidentity/live値/発行結果、notice検証、verdictを保存する。tree-exitのoptional OpenProcess oracleを除去し、保持済み同世代rootとWatchdog孫、初回0、一致ACK/ready、root exit0を必須とした。明示Drop後にはcleanup理由なし・terminate0・temp不存在も要求する。

既存の元cancel末尾assert群（ready、Cancelled、兄弟生存、合成bytes不変）は改行正規化後に原本と完全一致した。ready2秒/deadline4秒、一般test3秒、watchdog8秒、child15秒fallback、救済2秒を維持した。M1 oracle suffixも元snapshotと一致する。M1 oracle不変を変更後EXEの受入済みとは扱わない。

## REDの証拠と限界

既存D1 runnerのtree-exit欠測実測は `process-cancel-d1-runner.md` と既存stderrに保持されている。root初回0、孫未登録、OpenOwnedProcessFailed、cleanup TreeDescendantにもかかわらず旧oracleがoptional通過した証拠である。

最初に新しいready必須assertを追加してcompileしたが、親指示に従いACK未実装の旧実tree-exitは実行していない。`cancel-a1-red-child-build.log` / `cancel-a1-red-compile.log` は**compile記録のみであり、behavior RED実行ログではない**。未同定孫と残存を増やす再現は行っていない。

代わりにA1-04の決定的fixtureへ、D1実測のrootだけのsnapshot形を手書きで渡した。新oracleが`MissingRetainedDescendant`を返すこと、完全な後発ACK/readyを与えても同じFAILであることを確認した。識別済み孫の初回258も`OwnedFirstWaitFailed`として拒否し、既存ACK失敗も後の完全noticeで上書きしない。この自己試験自体は「拒否が正しい」という期待値でPASSしている。新しい製品cancel failureの再現とは呼ばない。

## 新規6自己試験と受入対応

全名のprefixは `process_unit::`。

| ID | test名 | 確認内容 |
| --- | --- | --- |
| A1-02 | a1_handwritten_ack_and_ready_require_exact_identity_and_complete_bytes | 手書きACK/ready正常値、root PID/creation・child PID/creation不一致、途中/余剰/257bytes拒否。実child consumerも不正完成fileをdeadline待ちせず拒否 |
| A1-02 | a1_missing_expired_and_late_ack_cannot_restart_the_deadline | 欠落＋期限到達を拒否し、同deadline後の正しいACKでも成功へ戻らない |
| A1-02 | a1_ack_publisher_requires_registered_live_current_unsealed_handles | 実自作process2個を用い、未登録孫拒否、登録済みlive pairの正規ACK、seal後拒否、旧世代拒否、既存bytes/初回値保持 |
| A1-02 | a1_child_cannot_publish_ready_after_its_descendant_exits | 実終了済み自作childで`DescendantNotLiveAtReady`、ready不作成 |
| A1-04 | a1_missing_d1_descendant_evidence_is_rejected_even_with_late_complete_notices | 上記欠測oracle。純粋な証拠形の入力でありraw=0をOS呼出しに渡さない |
| A1-04 | a1_withheld_and_mismatched_ack_fail_without_ready_or_cleanup_loss | 実supervisorを使ったACK withheld/mismatch各1case。有限にexit241、fixture拒否、readyなし、全保持handle初回0、reap/watchdog false、救済0、temp削除 |

`artifacts/cancel-a1-selftests.log`: **6 PASS、0 FAIL、83 filtered、2.27秒**。negative case内の意図的assert捕捉は成功の条件であり、fake supervisorは使っていない。

## 採用回帰・実例

| 範囲 / log | 結果 |
| --- | --- |
| A1-01 / cancel-a1-adopted-selftests.log | 15 PASS、0 FAIL、0.30秒。`d1_` filterによりR2 14件に加え名前にd1_を含むA1欠測case1件も選択された。独立新規件数へ二重加算しない |
| A1-03 / cancel-a1-tree-exit.log | 実tree-exit 1 PASS、0.06秒 |
| A1-05 focused部分 / cancel-a1-cancel.log | 元cancel 1 PASS、0.05秒。全unit順の再現試験ではない |
| A1-01 / cancel-a1-repeat.log | 12反復/24call漏洩 1 PASS、0.49秒、許容+2不変 |
| A1-01 / cancel-a1-prereject.log | 起動前拒否2回 1 PASS、0.01秒 |

各logは `artifacts/` 配下。ユニークに確認したcaseは24件（採用14＋新規6＋既存focused4）、実行件数は重複1件を含め25件である。

正常tree-exitのidentityはroot PID27896 / creation134358589239672548、孫PID20756 / creation134358589239875179。ACK準備時は双方live、ACK/ready一致、保持root/孫の初回wait0、reap前後false、watchdog false、救済/cleanup未確認ログ0。test自身がDrop後の専用temp不存在を確認した。

元cancelのroot PID6012 / creation134358589721876641、孫PID7820 / creation134358589722078868。root AfterCreate、孫CancelObserver、孫Watchdogの3独立handleは初回waitすべて0。reap前後false、watchdog false、救済/cleanup未確認ログ0。兄弟と合成bytesの既存assertも通過した。旧cancelの原因未確定を維持する。

## 実行条件とコマンド

`artifacts/cancel-a1-execution-env.log`でadministrator=False。既知のsandbox rename制約に対し、親承認済みの非管理者sandbox外で合成試験だけを実行した。UAC/OS設定/永続環境は変更していない。実Praat/M2は今回のfocused対象外であり、WW_TEST_PRAAT_EXEを用いた全unitは未実行である。

```text
cargo build --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --bin acoustic-test-child --locked --offline
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --no-run --locked --offline
rustfmt --check --edition 2021 tools/acoustic-analysis/tests/support/process_unit.rs tools/acoustic-analysis/tests/support/praat_child.rs
```

最終build/compileは `cancel-a1-child-build.log` / `cancel-a1-compile.log`。成功し、警告は既存・診断Debug用のdead_code等である。全crate fmtは親のlib.rs統合後のrunnerへ残した。

作者EXEに次の引数を個別に渡し、stdout/stderrを上記専用logへ保存した。

```text
a1_ --test-threads=1 --nocapture
d1_ --test-threads=1 --nocapture
--exact process_unit::child_exit_with_descendant_holding_pipes_does_not_hang_eof_collection --test-threads=1 --nocapture
--exact process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data --test-threads=1 --nocapture
--exact process_unit::repeated_success_and_injected_assignment_failure_do_not_leak_handles --test-threads=1 --nocapture
--exact process_unit::missing_executable_and_embedded_nul_fail_without_running_child --test-threads=1 --nocapture
--list
```

将来R2の14件だけを選ぶならfilterは `process_unit::d1_` とする。今回の重複分は隠しておらず、同じ対象を再実行して件数を作り直していない。

`artifacts/cancel-a1-test-list.log` は **89 tests、0 benchmarks**。統合後のM1 configured skip1なら88件予定（既存M2 15件を含む）。作者はM2ソースもtestも変更していない。

## cleanupとhandoff

`cancel-a1-pre-ledger.json` → `cancel-a1-post-ledger.json` は同名・同時刻の6件→6件、増分0。D1のZ32XRKを含む既存未知tempは変更・削除していない。自己試験の意図的保持は自身の一意tempだけを回収した。正常tree-exit、cancel、withheld/mismatchのcleanupはtestで確認済みである。

親へfixture2source固定を通知し、lib.rs module順だけの逐次fmt所有へ引き渡した。次は独立安全review、統合後の固定EXEで全unit/公開targets/no-default/fmt、同一最終EXEの外部M1再受入である。A1-05全unitとA1-06実行部分は未検証。作者EXEの成功をM1受入、M2 Post、旧cancel原因解決へ転用しない。

## §13追補（A1-R2、最新固定）

2026-10-08。正本追補SHA=`4E951D34056360F84221BF8DDA6EB1D46E12D07AFD06D0B756CF30C07337B4FB`、独立差分Pre PASS後、同じAstra/high割当で修正した。追加変更は原本process_unit.rsだけである。praat_child.rsは初版から不変、親のlib.rs機械整形は開始時のSHA `832864EFFB903CCC97D0E7BB5B42DCCC92F7D4B7EF5584D3A3F4BEC87F33452B` を維持した。実行model metadata/usage counterは未取得。

初版source3本と実在unit/child EXEを新規 `artifacts/cancel-a1-r2-before/` へ保存し、`cancel-a1-r2-before-hashes.json`へhashを記録した。初版ログ・hash・D1 copy・旧snapshotは上書きしていない。

### 決定的RED

private有限seamを旧処理のatomic_notice成功後・AckEvidence登録前へ置き、初回snapshot/全0ms採取の後にpublisherを解放した。実rootはACK受理→ready→exit0、保持root/孫は初回0であるが、旧処理は `MissingAckEvidence` でFAILした。

`artifacts/cancel-a1-r2-red.log`: 新規 `a1_r2_published_ack_finalize_delay_succeeds_after_first_snapshot` 1 FAIL、0.09秒。compileは `cancel-a1-r2-red-compile.log` で成功している。これは設計§13の偽失敗を同じ境界で再現したbehavior REDであり、製品取消しfailureでもcompile failureでもない。旧経路を繰り返していない。

### 修正した順序

1. root/Watchdog孫の同定・live確認後、admission確認と同じ短mutex内でPrepared slotを確保する。上限16、世代重複拒否、確保失敗時のfile公開なしを維持する。
2. lock外でfileを公開し、成功/失敗を同世代・同identityのPrepared slotだけへCompleteとして記録する。Completeの二重上書きも拒否する。後からsealが成立した事実だけで正常公開をLateへ変更しない。
3. supervisor返却直後の全初回0msと返却結果・reap/ready/poison/watchdog flagsを先に不可逆固定する。その後だけ、Prepared publisherのCompleteをCondvarで回収する。待機は既存callの同一絶対deadline残時間内であり、spurious wakeupでも期限を足し直さない。Prepared欠落・poison・余時間0・Complete未達は失敗である。
4. 完全ACK/ready、root exit0、同一identity/live証拠、保持root/孫の初回0を最終判定する。root失敗後の真の後発ACK、公開失敗、初回258を後のCompleteやsignalで成功にしない。

watchdogを返却直後に停止/joinしていない。決定的GREENでは同じFixtureでtree-exit→echoの次runも実行し、watchdog継続・世代2を確認した。private seamは同じ絶対deadlineで有限に解放され、Fixture unwindのDropでもjoinより先に解放される。新しい本体hook/thread/global設定・期限延長はない。

### 新規5件と対照

prefixは `process_unit::`。A1-02/03/04、AH-AC09、設計§13に対応する。

| test名 | 独立期待値 |
| --- | --- |
| a1_r2_published_ack_finalize_delay_succeeds_after_first_snapshot | 上記REDと同順序でGREEN。Prepared時刻≤全初回採取、Complete時刻≥全初回採取をassertし、ready/exit0/初回全0/次run/救済0/temp削除を確認 |
| a1_r2_completion_absence_deadline_and_poison_are_failures | Prepared欠落、即時期限切れ、未Completeの有限20ms deadline、同deadline後Complete、registry poisonを拒否。20msは同期helperの入力期限の自己試験であり製品/既存test期限の変更ではない |
| a1_r2_old_completion_cannot_fill_or_overwrite_another_generation | 旧世代Completeが次世代slotを完了させない、未準備世代拒否、記録済み失敗をOkで上書きしない |
| a1_r2_publication_io_failure_completes_as_failure_and_keeps_first_timeout | 自己所有ACK宛先を占有して実publish失敗を得る。Completeは失敗、合成live child初回258は救済後0でも固定、oracleは不合格 |
| a1_r2_publisher_gate_releases_on_deadline_and_fixture_unwind | publisher側/採取側双方のseam期限切れとFixture panic時の解放を確認し、永久待機を作らない |

既存A1 publisher自己試験に世代重複拒否を追加した。withheld対照ではroot exit241の観測後に正しいACKを実際に公開し、ready不在と既存`AckWithheld`失敗が維持されることを追加確認した。単なる偽のlate文字列を与えるだけの対照ではない。

### 最終focused結果

全試験は同じ最終unit/child EXE、非管理者sandbox外で実行した。`cancel-a1-r2-execution-env.log` はadministrator=Falseである。全unit/M1は実行していない。

| log（artifacts配下） | 結果 |
| --- | --- |
| cancel-a1-r2-selftests.log | 新規5 PASS、0 FAIL、0.15秒 |
| cancel-a1-r2-a1-regression.log | 既存A1 6 PASS、0 FAIL、2.21秒 |
| cancel-a1-r2-adopted-regression.log | 採用R2 14 PASS、0 FAIL、0.36秒 |
| cancel-a1-r2-tree-exit.log | 1 PASS、0.06秒 |
| cancel-a1-r2-cancel.log | 1 PASS、0.05秒 |
| cancel-a1-r2-repeat.log | 24call/12反復 1 PASS、0.53秒 |
| cancel-a1-r2-prereject.log | 起動前拒否2回 1 PASS、0.01秒 |

合計**29ユニークcaseがPASS、重複実行なし**。compileは `cancel-a1-r2-compile-final.log` で成功し、2fixtureのrustfmt --checkも成功した。child source/EXEは初版から不変であり、同じ固定child EXEを使った。初回258の意図的自己試験と実tree-exit/cancelの終了観測を混同しない。

使用したfilterは次であり、各exact caseは前節と同名である。

```text
process_unit::a1_r2_ --test-threads=1 --nocapture
process_unit::a1_ --skip process_unit::a1_r2_ --test-threads=1 --nocapture
process_unit::d1_ --test-threads=1 --nocapture
```

最新一覧 `cancel-a1-r2-test-list.log` は**94 tests、0 benchmarks**。M1 configured skip1の全unit予定は93件であり、旧件数89/88を用いない。既存M2 15件のsource/testは不変である。

### 最新hash・保存・残件

| 対象 | SHA256 |
| --- | --- |
| process_unit.rs | `2009A2B7EED04A5C07E95EFFF11A12B773B8BDA533796475B1FBF61380B38425` |
| praat_child.rs（不変） | `14EAF8EE8A55F040DD63989E82DD2E9F96A900ED4AEAECB8489082F1361ED356` |
| lib.rs（親fmt完了時から不変） | `832864EFFB903CCC97D0E7BB5B42DCCC92F7D4B7EF5584D3A3F4BEC87F33452B` |
| unit EXE | `868DB9158B1D0705D4E7E098AE2409F3AE665FBDFCFB33C47CC01C55080481E9` |
| child EXE（不変） | `3AA26103A2A9BE1211900596647296C8C2C5DAFEECE81AF793DF779389FC5060` |

EXEは上記原本target/debugの同path。`cancel-a1-r2-exe-hashes.json`、`cancel-a1-r2-source-inventory.json`、`cancel-a1-r2-process.patch`を保存した。M1 oracle suffixと元cancel末尾assert群は元snapshotと改行正規化一致である。production process/M2/manifest/lock/scriptは不変、lib.rsは作者未編集である。

ledgerは `cancel-a1-r2-pre-ledger.json` → `cancel-a1-r2-post-ledger.json` の6→6、名前/更新時刻差分0。今回の意図的保持は自己所有tempだけを回収し、既存D1残存/未知dirを削除していない。

sourceとhashを親へ固定通知した。残件は新しい独立安全review、同じ固定対象によるrunnerの全unit/公開targets/no-default/fmt、外部M1、Postである。旧cancel原因未確定と新EXEのM1未受入を維持し、今回の29 PASSで置き換えない。
