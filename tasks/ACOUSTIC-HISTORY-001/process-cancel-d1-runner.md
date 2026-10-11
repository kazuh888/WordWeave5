# I5-CANCEL-D1 R2 独立runner結果

2026-10-07 / ACOUSTIC-HISTORY-001 / AH-AC09。**元cancelはD1条件で未再現である。全unitの集計は82 PASS / 0 FAILだが、別のtree-exit caseで孫回収未確認・専用temp残存を観測した。元の全unit failure、M2 Post-gate、変更後M1を閉じない。**

## 範囲と引継ぎ

plannerのI5割当は `gpt-6.1-sol/medium`（implementation.md割当表）。親からPre PASS・R2再安全review PASS・全unit一回実行許可を受領した。実行model/effort metadata、開始/終了の使用量counterは未取得である。AGENTS、development-team、KPI、wordweave-changeを読み、runnerの検証・結果報告範囲だけ適用した。

入力はprocess-cancel-diagnostic-design.md（SHA256 `591DFF53337775E1510C2D28C417F81D5313BB81F4F517F9B632C878894D1457`）とprocess-cancel-d1-tests.md末尾R2である。再build・追加filter・再試験はしていない。原本/隔離source・期待値・設定を編集せず、生成ログと本報告のみを書いた。親のGit禁止を優先しGit操作は行っていない。commit値の新規取得に代えて、試験入力全17件の開始/終了hashを固定した。

## 固定対象と実行条件

原本 `tools/acoustic-analysis` と隔離 `artifacts/cancel-d1-crate` の17件は、R2 inventoryと開始/終了とも一致した。不一致0、隔離差分は指定された2fixtureのみである。全件は [開始inventory](artifacts/cancel-d1-runner-pre-inventory.json) / [終了inventory](artifacts/cancel-d1-runner-post-inventory.json) に記録した。

| 試験入力 | SHA256 |
| --- | --- |
| 隔離process_unit.rs | `4A5BCB21166F1AACAE17F9A447EF0D738FEAAADC8DC0CB3A9EE2654327E91EEC` |
| 隔離praat_child.rs | `A85971AC95A7883239E292FEFB25DF7F7C8BC6461B584946091DC78E4E2A1C9B` |
| unit acoustic_analysis-8d1dd6b6ba45c163.exe | `7F37ABED714DE2F81A535494024D733AABE9FA8ED7DCCE5DF9BAFAE7995C66BC` |
| child acoustic-test-child.exe | `FC2C80721E1E5B5917650868E8521AB2273B29118482B4C74FBEC6EC2A43F684` |
| 固定原本Praat.exe | `22C8994FD9767A2A5C1F03F21B35E41BB5AC716C628965E5D437033EEDA8FFF4` |

Praatは起動前に絶対pathの存在、size=96,780,624、hash一致を確認した。`WW_TEST_PRAAT_EXE=F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\praat-probe\praat7002-x64v1\Praat.exe` を当該processの環境へ明示した。永続環境変更・copy・downloadなし。EXEのbytes/pathは [hash記録](artifacts/cancel-d1-runner-exe-hashes.json) にある。

非管理者（administrator=false）、sandbox外の要求で起動した。UAC/OS設定変更なし。.NET ProcessStartInfoのUseShellExecute=false / CreateNoWindow=trueを用いた。子console handleと外側Job所属は独立取得していない。ここを確認済みとは扱わない。

## コマンド・実結果

cwdはrepo root。既compile unit EXEへ以下を一回だけ渡した。

```text
tasks/ACOUSTIC-HISTORY-001/artifacts/cancel-d1-crate/target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --test-threads=1 --nocapture
```

外側監視は当該所有ProcessにWaitForExit(180000)を適用した。期限到達時だけ当該Process.Kill()を行う構成であり、今回は到達せずkillなし。未知PID/名前一致kill・利用者アプリ操作なし。内部期限は変更していない。

| 実行 | exit / 結果 |
| --- | --- |
| 起動前inventory/Praat/EXE検証 | 0 / 17件一致、Praat/EXE一致 |
| 全unit | 0 / 82 PASS、0 FAIL、0 ignored、0 measured、M1 configured filtered 1、未到達0 |
| 終了inventory/ledger採取 | 0 / 17件一致、専用temp増分1 |

M2 15件、D1自己試験14件、元cancelを含む。作者の83件一覧は入力資料として使用し、新しい一覧試験・新しいPASSとして数えていない。実測unit時間52.83秒、外側52.8550613秒。UTC開始2026-10-07T14:34:19.1950559Z、終了14:35:12.0851636Z。所有unit PID30696、creation UTC14:34:19.1976879Zである。

新規証拠は [stdout](artifacts/cancel-d1-runner-stdout.log)、[stderr](artifacts/cancel-d1-runner-stderr.log)、[実行metadata](artifacts/cancel-d1-runner-metadata.json) にある。既存ログを今回の実行結果として流用していない。

## 元cancelの同定・初回結果

stderr 12–13行が元cancelの同一callである。supervisor=`ProcessFailure { kind: Cancelled, reason: Wait }`、descendant_ready=true、reap_before=false、reap_after=false、poison_seen=false、watchdog_rescued=false。世代1の3独立handleを記録した。

| role / origin / sequence | PID / creation FILETIME | 初回0ms wait |
| --- | --- | --- |
| Root / AfterCreate / 1 | 2572 / 134358573098751331 | 0 |
| Descendant / CancelObserver / 2 | 33180 / 134358573098987050 | 0 |
| Descendant / Watchdog / 3 | 33180 / 134358573098987050 | 0 |

初回採取は単調時刻4612059706.820213903–4612059706.820215003sに完了した。Drop後のreportはrecords=[]であり、このcallの救済ログはない。兄弟生存・保存bytes不変を含む元cancelのtestはPASSである。観測配置が異なる隔離fixtureでの一回未再現であり、原本で観測済みWAIT_TIMEOUTの原因否定・解決ではない。

## 別caseのcleanup未確認と残存ledger

`child_exit_with_descendant_holding_pipes_does_not_hang_eof_collection` はtest集計上PASSだが、stderr 21–23行でroot PID34468 / creation134358573102582092、supervisor Ok、root初回wait0、watchdog `OpenOwnedProcessFailed`、孫handle未登録を観測した。`cleanup=unconfirmed` / reason=`TreeDescendant` として以下を保持した。

```text
F:\TempCodex\ww-praat-process-日本語 Z32XRK
```

ログ上の短縮pathは `F:\TEMPCO~1\ww-praat-process-日本語 Z32XRK` である。保持pathの通知fileをread-onlyで確認した。`grandchild.identity` はroot34468/creation134358573102582092、child11712/creation134358573102809610、`grandchild.pid` は11712であった。この通知は当該孫handleの取得・終了確認を代替しない。孫の生存/終了・Job所属は未確認であり、探索、再Open、kill、temp削除をしていない。原因は確定していない。

開始ledger5件→終了6件、既存5件は同じ名前/更新時刻、増分はZ32XRKのみである。[開始ledger](artifacts/cancel-d1-runner-pre-ledger.json) / [終了ledger](artifacts/cancel-d1-runner-post-ledger.json) を残した。既存の未知5件も削除していない。

全ログのwatchdog_rescued=trueおよびcallのreap_before/reap_after=trueは0件である。ただし自己試験が意図的に扱う救済・poison・WatchdogPanic・cleanup結果境界の保持は別である。D1-A実publisherはlive root/孫をDropで救済し、D1-Dは初回258→救済0を保持する。これらとD1-E/Rの意図的救済は元cancel failureの再現ではない。D1-Rの保持自己試験pathは自身の試験内で回収され、終了ledgerへ追加されていない。既存unwind caseのcatch対象panicもtest PASSの構成要素である。

## 親・独立reviewerへの残件

完了境界はR2隔離EXEの一回実行と証拠報告までである。元cancel未再現、別tree-exit孫回収未確認、原本failure原因未確定として返す。残存とcleanup判定の扱いを独立reviewerへ戻す。M2 Post-gate、元fixtureへの採用、D2要否、変更後診断EXEのM1受入は親側に残る。

今回の限定診断はWindowsアプリ納品ではなく、cargo test --all-targets --locked / cargo build --release --locked --bin wordweave5は再実行していない。native microphone/pen/TTS/IME/Codex実認証の新規受入も実施していない。使用量counterの欠測を件数・bytes・経過時間から推定していない。
