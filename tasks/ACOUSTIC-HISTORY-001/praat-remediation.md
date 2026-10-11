# I5レビュー指摘と修正単位

2026-10-07。execute-review-remediationを適用する。HEADはb077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d、branchはcodex/development-agent-team。実装中dirty差分を保持する。モデル/effortは既存I5の指定を維持し、親は変更しない。

## Closure matrix

| ID | 原文・位置と原因 | 状態 / Packet | 正本 / oracle / 残余 |
| --- | --- | --- | --- |
| I5-P1 | 修正前process_windows.rs:397/421/588。CreateProcessWのsuspended作成後からAssignProcessToJobObjectまでに親が強制終了すると、Dropが動かず専用Job未所属のchildが残る。原因は作成と所属が別操作であること | Closed / I5-M1、Pre-gate・Post-gate PASS | design-i5.md §4の作成時JOB_LISTを実装。利用者RED→GREEN、53/20/59回帰、[独立最終判定](praat-m1-post-review.md)。P1以外のI5未検証を完了扱いしない |
| I5-Q1 | 修正前praat.rs:510–521。WAV writer close→read pinの間に同長別波形へ置換でき、再open後digest照合がない。workへ書ける別processを境界に含むかが質問 | Closed / I5-M2、Pre-gate・Post-gate PASS（2026-10-08） | [M2 Brief](praat-m2-design.md)どおりstream SHA-256→writer close→read pin同handleの長さ/digest照合→子回収までpin保持を実装。T1/T2 RED→独立15件GREEN、新回帰・外部M1・[独立最終判定](m2-w1-gates.md)により閉鎖。旧cancel原因未確定、I5c-2資源/障害検証等は別残件である |

独立reviewerはpraat_adapter_review、指定Astra/high、read-only。P1確信度高、Q1は独立blockingへ数えていない。parentはP1とQ1対策を技術的な整合性要件として受け入れた。両者はowner/failure境界が異なるため別Packetとし、M2はM1のPost-review PASS後に進める。

レビュー時hash：praat.rs `6BD9D84ADC85E2EBB2E0E546DDCED6AB086405829DBF1B90C3653DAB45C43787`、praat_protocol.rs `EF354A74EA2498D8BEC36F68A826BF031282000357EDD4094A9D80ED3AC8AB48`、process.rs `9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54`、process_windows.rs `F73399B120FD07E024182092F0E1D1A183E8659960B1ABB8AF740FD50ED3208B`、script `3616A00415149E0EF5791F5EA497CBF523468695FAA44FB059D911B6BCA0876E`。開始/終了不変をreviewerが確認した。レビューは公開20/unit53/default59の既存ログと整合を確認したが、再試験はしていない。

## I5-M1 Brief（規範）

### 1. Objective

Praat supervisorの子作成直後に親が強制終了しても、停止中の子を専用Jobのkill-on-close対象から漏らさない。

### 2. Scope

本体変更はprocess_windows.rsの属性list・spawn・所属確認、必要なprivateコメントのみ。独立試験はtests/support/process_unit.rsとその既存合成fixtureへの最小追加に限定する。新規依存、public API、Report/保存形式、memory/時間上限、UI、音声送信は変更しない。Q1、lease/cleanup全般、合計メモリ実測、I6は対象外である。

### 3. Current evidence

現spawnはHANDLE_LIST一属性、CreateProcess成功後のguardはassigned=falseである。runが後からAssignProcessToJobObjectを呼ぶ。通常Errとunwindは回収できるが親強制終了ではDropは動かない。Windowsは親終了だけでは子を自動終了しない。[公式根拠](https://learn.microsoft.com/en-us/windows/win32/procthread/terminating-a-process)。

### 4. Observable contract

唯一の正本はdesign-i5.md §4の訂正後起動順である。制約・kill-on-close設定済JobをPROC_THREAD_ATTRIBUTE_JOB_LISTへ入れ、suspended childを作成時から所属させる。HANDLE_LISTも維持する。作成成功後に所属確認してからresumeし、失敗時は無制約fallbackしない。属性に渡す値配列の寿命をDeleteProcThreadAttributeListまで保持する。[公式API](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)でJOB_LISTはWindows 10以降/Server 2016以降対応と確認した。

### 5. Owner / consumers

OS process生成・回収ownerはprocess_windows.rsのspawn/run/ChildGuardである。production consumerはprocess::supervise経由のPraatBackend::new（version）とPitchBackend::extract（analysis）の二箇所。hostの直接プロセス生成、他のPraat instance、WordWeave5データへ触れない。

### 6. Types / errors

ProcessSpec/Limits/Output/Failureとpublic AnalysisErrorは変更しない。属性設定・所属確認の失敗は既存ResourceLimit/SetupまたはAssignを返す。CreateProcessW自体が失敗した場合は従来のBackendUnavailable/Setupを維持し、作成された子があると仮定しない。成功してから所属確認が失敗した場合はresumeせず所有childをkill+waitする。取消し/期限の優先順は維持する。

### 7. Transition / failure-after-state

JobConfigured→両属性作成→SuspendedInJob→MembershipVerified→Running。CreateProcess失敗前はchildなし、成功後は専用Job対象である。専用Job handleは非継承で、最後の所有親が強制終了してもkill-on-closeが働く。run内の後続AssignProcessToJobObjectは削除する。既存のprivate BeforeAssign/AfterAssignラベルは所属確認の前後を示す。通常cancel/Err/panicでは明示kill+wait/RAIIを維持する。所属なしを明示観測した異常は所有processへTerminateProcess→waitする。

### 8. Security / data boundary

既存のstdin NUL/stdout/stderrだけのHANDLE_LISTを保持し、Jobやhost秘密handleを渡さない。512MiB/同時1/共通60秒を維持する。テストは合成childと隔離tempだけ、PID探索ではなく自分が作ったprocessのhandleで終了を確認する。原音/教材/実APIを使わない。

### 9. Dependencies / stop

独立Pre-gate PASS後に実装する。属性値寿命・nested Job・対象OSで安全性が成立しない場合、fallbackせず修正単位を停止して再設計する。新public型/依存/保存影響が必要な場合はscope外として止める。親が外部Jobに囲まれた試験だけを、専用Jobの安全性の証明にしない。

### 10. Deterministic oracle

既存AfterCreate hookで生成直後のbarrierを作る。独立fixtureは別processのsupervisorを起動し、child PIDを受信した第三者が独立handleを開いてからsupervisorを強制終了する。supervisor自身はJob未所属と確認し、その外側Jobによる救済を除外する。childのwaitが5秒以内に終了、未resumeのsentinelが存在しない、無関係の兄弟processは生存することを確認する。失敗時も試験が所有するchild/兄弟をhandleで回収し、回収をPASSへ読み替えない。fixtureは8秒自己watchdog、外側runner45秒で有限化する。既存AfterCreate境界の所属確認も追加し、修正前に失敗、修正後に成功することを要求する。fixture環境がJob未所属を成立させられない場合は未検証とし、skip成功へしない。

回帰は既存process/protocol53件、実Praat公開20件、default59件、所有fmt/diffである。実Praat20件はconsole無し条件を明記し、診断規則を緩めない。

### 11. Rollback / artifacts

未配布のdirty実装であり、既存EXE/データへ配置しない。修正が成立しなければ本Packetの対象差分だけを保存した上でI5提供を未完了に戻す。git resetや無関係変更のrevertはしない。合成試験の自分のtemp/logとSHA証拠を残し、他process/実録音を削除しない。

### 12. Traceability / status

I5-P1→AH-AC09の生成直後親異常終了部分・design-i5.md §4/8・M1 oracle。Pre-gate PASS→実装→限定回帰と利用者GREEN→独立Post-gate PASSによりM1をClosedとする。AH-AC09全体/I5全体の完了ではない。[gate記録](praat-m1-gate.md)、[独立最終判定](praat-m1-post-review.md)。

## Pre-gate提出（cycle1当時の履歴・非規範）

以下とgate記録の旧文書hashは提出時点のsnapshotを示す。完了時に状態欄だけ更新しており、Briefの技術契約§1–11は変更していない。

cycle1、requested Gate=Pre-implementation、review class=Windows process lifetime/reliability。branch/HEADは冒頭の通り。対象snapshotは本Brief、design-i5.md、design-i0.mdとレビュー時production hashで固定し、提出時の文書hashを別request記録へ保存する。製品判断の未決0、技術方針は上記一案である。変更は設計正本と本書だけ、productionはレビュー時hashのままである。wordweave-change、review-project-conventions、review-reliability-data-integrity、review-release-operabilityの観点をself-checkへ用いた。明示Pre-gate PASSまで本体を変更しない。
