# CANCEL-A1 R2 最終独立runner

2026-10-08 / ACOUSTIC-HISTORY-001 / AH-AC09 / A1-03・05・06。**指定回帰は成功した。feature praatは172 PASS / 0 FAIL / M1 filter1、no-defaultは59 PASS / 0 FAILである。cancel・tree-exitの保持handle初回wait、ACK/ready、cleanupを確認し、残存ledgerは6→6で増分0であった。最終2EXEを固定した。外部M1と独立Postは未実施である。** 原本cancelの過去failure原因は未確定のままである。

## 入力・責任・固定差分

planner I5のrunner指定は `gpt-6.1-sol/medium`。親からA1 §13差分Pre PASS、新独立安全review PASS、全回帰実行許可を受領した。起動指定と実行metadataを区別し、実行model/effort、開始/終了の入力・cache・出力・推論counterは未取得である。AGENTS、development-team、wordweave-changeを読み、検証・報告責務だけ適用した。KPIの欠測規則を適用し、数値を推定していない。

正本はprocess-cancel-adoption-design.md §10/13、SHA256 `4E951D34056360F84221BF8DDA6EB1D46E12D07AFD06D0B756CF30C07337B4FB`、作者handoffはprocess-cancel-a1-tests.md末尾R2、統合handoffはprocess-cancel-a1-integration.mdである。原本tools/acoustic-analysisの作者inventory17件は開始/終了すべて一致した。[開始source](artifacts/cancel-a1-runner-pre-source.json) / [終了source](artifacts/cancel-a1-runner-post-source.json) に全件を保存した。commitへの新規変更はなく、未コミット対象はこのhash集合で固定する。親の指定source3本は以下である。

| source | SHA256 |
| --- | --- |
| tests/support/process_unit.rs | `2009A2B7EED04A5C07E95EFFF11A12B773B8BDA533796475B1FBF61380B38425` |
| tests/support/praat_child.rs | `14EAF8EE8A55F040DD63989E82DD2E9F96A900ED4AEAECB8489082F1361ED356` |
| src/lib.rs | `832864EFFB903CCC97D0E7BB5B42DCCC92F7D4B7EF5584D3A3F4BEC87F33452B` |

製品process2本、M2のpraat.rs/praat_unit.rs、manifest/lock/scriptを含む残り14件も固定inventoryに一致した。元cancel末尾assert・M1 oracle不変の作者確認を入力として受領した。runnerはsource・tests・期待値・設定を編集していない。build生成物、専用ログ、本報告、許可されたM1 byte snapshotだけを書いた。既存dirty worktreeをgit status --shortで確認し保持した。公開、実API、実学習データ、UAC、未知process探索/kill、既存temp削除なし。

## 環境・監視

非管理者administrator=false、親の許可どおりsandbox外の実行を要求した。require_escalatedはUACではない。.NET ProcessStartInfoでUseShellExecute=false、CreateNoWindow=true、stdout/stderrを専用logへ採取した。子console handleと外側Job所属は独立取得していない。[環境](artifacts/cancel-a1-runner-env.json) に確認範囲を記録した。外側Job不在をこの回帰で証明したとは扱わない。

feature試験前に固定原本Praatの絶対path存在、size=96,780,624、SHA256=`22C8994FD9767A2A5C1F03F21B35E41BB5AC716C628965E5D437033EEDA8FFF4`を検証した。当該feature試験processへ次を明示した。

```text
WW_TEST_PRAAT_EXE=F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\praat-probe\praat7002-x64v1\Praat.exe
```

永続環境変更、Praat copy/download、default探索への代替なし。unitはbuild時間を含まない180秒のWaitForExit監視とし、timeout時だけ自ら起動したProcess.Kill()を使う構成であった。実際はtimeout=false、killなし。内部deadline・終了oracleを変更していない。

## コマンド・target別結果

cwdはrepo root。all-targetsはcompileとEXE実行に分け、Cargoのcompiler-artifact JSONからtest targetを列挙した。対象一覧は [targets](artifacts/cancel-a1-runner-targets.json) に保存した。test=falseの通常childをtestとして二重実行していないが、all-targetsが生成したchild test harnessも0件targetとして実行した。

```text
cargo build --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --bin acoustic-test-child --locked --offline
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --all-targets --no-run --locked --offline --message-format=json
<当該unit EXE> --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --test-threads=1 --nocapture
<当該child test harness / praat / schema / core EXE> --test-threads=1 --nocapture
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline
cargo fmt --manifest-path tools/acoustic-analysis/Cargo.toml --all -- --check
git diff --check
git diff --no-index --check -- <cancel-a1-beforeの各source> <対応する原本source>
```

| 対象 | exit / PASS・FAIL・skip | test時間 / 外側時間 | stdout log |
| --- | --- | --- | --- |
| child build | 0 / build成功 | — / 13.025秒 | [build](artifacts/cancel-a1-runner-child-build-stdout.log) |
| feature all-targets compile | 0 / compile成功 | — / 21.111秒 | [compile JSON](artifacts/cancel-a1-runner-praat-compile-stdout.log) |
| feature lib unit | 0 / 93 PASS・0 FAIL・1 M1 filtered | 54.99 / 55.183秒 | [unit](artifacts/cancel-a1-runner-unit-stdout.log) |
| feature child test harness | 0 / 0 PASS・0 FAIL | 0.00 / 0.101秒 | [child harness](artifacts/cancel-a1-runner-feature-acoustic-test-child-stdout.log) |
| feature public Praat | 0 / 20 PASS・0 FAIL | 61.79 / 61.950秒 | [Praat](artifacts/cancel-a1-runner-feature-praat-stdout.log) |
| feature core | 0 / 32 PASS・0 FAIL | 0.12 / 0.297秒 | [core](artifacts/cancel-a1-runner-feature-core-stdout.log) |
| feature schema | 0 / 27 PASS・0 FAIL | 0.05 / 0.239秒 | [schema](artifacts/cancel-a1-runner-feature-schema-stdout.log) |
| no-default all-targets | 0 / lib0・core32・Praat0・schema27 PASS、全0 FAIL | 各0.00・0.04・0.00・0.03 / compile込み12.660秒 | [no-default](artifacts/cancel-a1-runner-no-default-stdout.log) |
| crate fmt | 0 / PASS | — / 0.307秒 | [fmt](artifacts/cancel-a1-runner-fmt-stdout.log) |

ignored/measuredはすべて0。unitの94一覧は作者入力であり、新しい一覧試験として反復していない。実行unit93件にM2 15件、D1 14件、A1 11件、元cancel/tree-exitを含む。内側poison childを追加caseとして数えず、名前重複filterも使っていない。feature172件とno-default59件は別configurationの実行数であり、231のユニークcaseという意味ではない。未到達0、M1だけ明示filter1である。

git diff --checkはexit0・出力なし（実行記録はrunner tool transcript）。対象crateがuntrackedのため、これだけで差分を確認済みとせず、編集前snapshotとの3本のno-index whitespace checkを追加した。各exit1は「差分あり」、全stdout/stderr 0bytesでwhitespace問題なし。[差分metadata](artifacts/cancel-a1-runner-diff-metadata.json) に各pathとexitを保存した。source差分の内容は作者R2 patchと統合fmt証拠を入力として固定している。

各コマンドの絶対EXEpath、引数、PID/creation、UTC時刻、exit/外側時間、process環境、timeoutは同prefixの`-metadata.json`に保存した。stderrも同prefixの`-stderr.log`にある。[結果集計](artifacts/cancel-a1-runner-results-summary.json) は新規ログだけから生成した。

## 重要2caseの新規生ログ

[unit stderr](artifacts/cancel-a1-runner-unit-stderr.log) 38–39行が元cancel、47–48行が通常tree-exitである。A1 R2遅延自己試験の別tree-exit callと区別した。

| case / role・origin | PID / creation FILETIME | supervisor返却直後の初回0ms |
| --- | --- | --- |
| cancel Root / AfterCreate | 26752 / 134358611355021747 | 0 |
| cancel Descendant / Watchdog | 23508 / 134358611355335327 | 0 |
| cancel Descendant / CancelObserver | 23508 / 134358611355335327 | 0 |
| tree-exit Root / AfterCreate | 3428 / 134358611358811168 | 0 |
| tree-exit Descendant / Watchdog | 34540 / 134358611358998083 | 0 |

cancelは`ProcessFailure { kind: Cancelled, reason: Wait }`、descendant_ready=true、reap_before/after=false、poison=false、watchdog_rescued=false。初回採取時刻4612063532.458180303–4612063532.458181403sである。兄弟生存・合成bytes不変の元assertがPASSした。ACKはtree-exit専用なのでcancelはtree_exit=Noneで正常である。

tree-exitはsupervisor Ok / exit_code=Some(0)、required=true、ACK/ready notices=Ok、verdict=Ok、reap_before/after=false、poison/watchdog=falseである。ACK準備時root_live=258・child_live=258は**退出前live確認**であり、終了oracle失敗ではない。同一identityのACK prepared_at=4612063532.826409103s、published=Ok、completed_at=4612063532.826774203s、終了後初回採取4612063532.845341303–4612063532.845342103sのroot/孫は全0であった。test末尾は明示Drop後のcleanup理由空・terminate数0・専用temp不存在をassertしPASSした。

この2callに救済ログはなく、Drop後reportはrecords=[]である。全unitのcallにwatchdog_rescued=true / reap_beforeまたはafter=trueは0件。意図的なD1/A1自己試験の初回258、救済、poison、publication失敗、catch対象panicは各拒否/回収oracleの確認であり、元cancel failureの再現ではない。

## ledger・cleanup

[開始ledger](artifacts/cancel-a1-runner-pre-ledger.json) → [unit後ledger](artifacts/cancel-a1-runner-unit-post-ledger.json) → [終了ledger](artifacts/cancel-a1-runner-post-ledger.json) は6→6→6、名前/更新時刻の差分0である。既存D1 Z32XRKを含む未知6dirをそのまま残した。未知process探索・名前kill・旧残存の後追い削除なし。

stderrにcleanup未確認保持は5行あるが、既知のD1-R判定境界4条件（OwnedWait2、SupervisorReap、TreeDescendant）とWatchdogPanic自己試験1件である。各case自身が保持を確認後に自己所有tempを回収し、終了ledgerに追加されていない。通常cancel/tree-exitの未確認保持は0件。自己試験の故意の保持を通常caseのcleanup欠測と混同していない。

## 最終EXE・外部M1 handoff

runnerの最終build実測hashは作者focused EXEと異なる。source一致だけからEXEhashを仮定せず、unit実行前→全回帰後の実hashを照合し同一であった。[実行前](artifacts/cancel-a1-runner-pre-exe-hashes.json) / [終了後](artifacts/cancel-a1-runner-post-exe-hashes.json) を保存した。

| 対象 | bytes / 最終SHA256 |
| --- | --- |
| tools/acoustic-analysis/target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe | 1,988,608 / `D923AFE3CE225EC3A48AC09E39B8D9B4C798E298481022F07B8DFC7AD4206A59` |
| tools/acoustic-analysis/target/debug/acoustic-test-child.exe | 379,904 / `8438841AF3D2853AB55C5DC89B82D135DF8BA254890EE0867076EC601565BAF2` |

新規 `artifacts/cancel-a1-m1-final/` に両EXEのbyte snapshotを保存し源泉hash一致を確認した。既存pathがあれば拒否する手順であり上書きなし。[snapshot記録](artifacts/cancel-a1-runner-m1-final-hashes.json) に源泉/保存先を記録した。以後このunit/childを再buildしない。no-defaultは別hash名のlib/test targetsであり、最終2EXEを変更していない。

M1 exactは本runnerでは実行していない。親が利用者へ既存の外側Job不在条件で、**この最終unit/child組**による1件の外部確認を依頼する。snapshotを任意配置してchild探索条件を変えた別試験で代用せず、実際の実行path/使用child/hashも外部結果に残す必要がある。既存45秒監視と救済前child終了・未resume・兄弟生存・watchdog非救済のoracleを維持する。外部M1・独立PostまではA1全体/M2 Post未完である。

今回の完了境界は限定fixture採用の指定回帰と最終EXE固定である。Windowsアプリ納品全体ではなく、root cargo test --all-targets --locked / cargo build --release --locked --bin wordweave5を代替したとは主張しない。native microphone/pen/TTS/IME/Codex実認証・原音精度の新規受入は未実施である。使用量counterの欠測、旧cancel原因未確定を維持する。時刻coverageは新build開始2026-10-08 00:36:56.298+09から最終照合後00:41:50.916+09までの取得範囲であり、最初の読取・報告作成・最終応答を含む全作業時間ではない。
