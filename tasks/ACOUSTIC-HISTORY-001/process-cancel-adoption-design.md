# CANCEL-A1: fixture安全補正の原本採用

2026-10-07 / ACOUSTIC-HISTORY-001 / AH-AC09。限定Packetの設計提出であり、Pre-gate PASS前に原本を変更しない。

## 1. Objective / boundary

D1で独立レビュー済みの対象同定・非panic回収・登録sealを原本fixtureへ採用し、tree-exitの孫handle欠測を解消する。原本取消しfailureの原因は未確定のままである。D1の82 PASSから旧failureをClosedへ変更せず、A1の成功は新しいfixtureによる原本回帰と証拠の完全性に限る。本体process実装を推測修正しない。

入力は承認済み[spec](spec.md) AH-AC09、[implementation](implementation.md) I5、[D1設計](process-cancel-diagnostic-design.md) §12追補、[作者R2](process-cancel-d1-tests.md)末尾、[独立runner](process-cancel-d1-runner.md)である。designer=`gpt-6-astra/high`、test author/implementer=`gpt-6-astra/high`、runner=`gpt-6.1-sol/medium`、reviewer=`gpt-6-astra/high`を維持する。AGENTS、development-team、KPI、api-and-interface-designを読了済みであり、UI非該当である。本役は本書のみを所有する。実行metadata/使用量counterは未取得である。

## 2. Evidence / adopted baseline

D1 R2は再安全review PASS後、82 PASS / M1 filter1、52.83秒。元cancelのrootと孫2observerは初回waitすべて0、reap=false、watchdog救済なしであった。ただしtree-exitではroot終了が先行し、watchdogのOpenOwnedProcessFailed、孫未登録、cleanup未確認temp1件を残した。既存testは終了後OpenProcess失敗を許すため、test集計PASSだけで終了確認が揃ったとはいえない。

採用元の固定SHA-256はD1設計=`591DFF53337775E1510C2D28C417F81D5313BB81F4F517F9B632C878894D1457`、隔離process_unit.rs=`4A5BCB21166F1AACAE17F9A447EF0D738FEAAADC8DC0CB3A9EE2654327E91EEC`、隔離praat_child.rs=`A85971AC95A7883239E292FEFB25DF7F7C8BC6461B584946091DC78E4E2A1C9B`である。現在の原本fixture hashesは各`CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896`、`8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228`である。隔離sourceを新たな正本として書き換えず、R2との差分としてA1を記録する。

## 3. Scope / source ownership

| 原本source | 所有と変更 |
| --- | --- |
| tools/acoustic-analysis/tests/support/process_unit.rs | test author：R2診断/同定/seal/非panic cleanup/14自己試験の採用、tree-exit専用ACK発行、保持handleを使う非optional oracle、新しいACK自己試験 |
| tools/acoustic-analysis/tests/support/praat_child.rs | 同じtest author：R2 identity publisher採用、tree-exitだけのACK待機と退出準備通知 |
| tools/acoustic-analysis/src/lib.rs | 統合implementerが逐次所有：既存module宣言順のrustfmt機械整形だけ。module/cfg/path/export/typeを追加削除しない |

3本を上限とする。fmt整形はfixture採用と別hunkとして記録し、他の本体整形を便乗させない。src/process.rs、src/process_windows.rs、M2のpraat.rs/praat_unit.rs、manifest/lock/script、public API、保存schema、network/UIは不変。fixture作者と統合役は同時編集しない。第三のロジックsourceや新依存が必要なら着手前に親へ戻す。

## 4. Adopted contracts

D1 R2のIdentity、OwnedProcessRecord、世代/取得元、16 handles/16世代/256events、registry短mutex、poison回復、初回0ms waitの固定、reap前後値、watchdog/救済の分離、cleanup未確認時のtemp保持を採用する。snapshotと世代sealを同じlockで確定し、後発observerはOpen前/挿入時の二重チェックでclose-onlyにする。元cancelのready2秒/deadline4秒/Cancelled/兄弟生存/保存bytes不変のassert群を保持する。

M1 oracle本体は源泉との改行正規化比較で不変を確認する。共有Fixtureの安全補正によりsource/EXE hashは変わるため、M1 oracle不変は新EXEの外部受入済みを意味しない。tree-exit以外へACK待ちを導入せず、root-only caseへ孫必須を課さない。

## 5. Chosen tree-exit handshake

方式を **watchdogによる同定済み孫handle登録後の、identity付きfile ACK** に固定する。root/孫Identityは既存PID+creation FILETIMEであり、通知WWCHILD1のdecode・実handle照合・世代チェックを再利用する。ACK名は`tree-exit-<root_pid>-<root_created>.ack`、退出準備名は同stemの`.ready`とし、呼出をroot identityで分ける。各fileは最大256bytesである。

| 状態 | 必須処理と所有 |
| --- | --- |
| TreeExitStarted | fixture rootが孫をstdout/stderr継承でspawnし、既存identity→PID通知をpublishする。専用Jobと起動方法は既存のまま |
| DescendantRegistered | 親watchdogが通知と現在世代/rootを照合し、実孫handleをregistryへ挿入完了する。保持はFixture Dropまで。単に通知を読んだだけではACKを発行しない |
| AckPrepared | 同世代/unsealed、root/孫の保持handleの0ms waitがともにWAIT_TIMEOUTであることを確認する。これは退出前のlive確認であり、supervisor後の合否waitとは別に記録する |
| AckPublished | watchdogがACKをcreate_new一時file→write_all→flush→close→非既存final名へrenameでpublishする。registry lock中にfile I/Oしない |
| RootExitReady | rootが一致ACKを期限内に読み、所有Child handleで孫liveを0ms確認する。対応するreadyを同じatomic publish手順で残してexit0する。孫をwait/killせず継承pipeを持たせたまま残す |
| SupervisorReturned | 本体がroot先行終了後の子孫/pipeを回収する。従来どおり直ちにseal＋初回0ms waitを採取する。ACK待ち/join/sleepをここへ追加しない |

ACKは`WWTREEEXITACK1\t<root_pid>\t<root_created>\t<child_pid>\t<child_created>\n`、readyはheaderだけ`WWTREEEXITREADY1`とした同一4値である。単一行・余剰bytesなし・全bytes一致を要求する。rootは自分と所有Childから得た期待値へ照合し、別世代・別孫・途中frame・余剰frameを許さない。既存grandchild.identity/PID通知は変えない。

watchdogはtree-exitとして開始時に記録した世代だけをACK対象とし、登録済みWatchdog孫recordから値を作る。登録→ACKの間にseal/世代相違を観測したら発行を拒否する。検査直後のsealとfile publishの競合が起きても、後発ACKはlatched結果を変えず失敗証拠として扱う。root成功には期限内受理とreadyが必須なので、終了後にACKが現れただけでは合格にできない。ACK発行中のmutex保持やsupervisor側の待機追加でこの競合を消さない。

## 6. Private helpers / consumer boundary

補助は2fixture内に閉じる。test側は`publish_tree_exit_ack(dir, generation, root: Identity, child: Identity) -> Result<(), &'static str>`相当のprivate窓口で、世代/admission・保持handle登録/live確認の後だけpublishする。child側は`wait_tree_exit_ack(dir, root_identity, child_identity, deadline: Instant) -> Result<(), &'static str>`である。型名の局所差は許すがwire bytesと責務境界を変えない。256byte bounded readとcreate_new/flush/renameは既存helperの考え方を再利用する。

CallEvidenceにはtree-exit required、ACK準備/発行結果、対象root/孫identity、退出前live観測、初回waitを区別して残す。global設定/public hook/process telemetryを追加しない。自己試験用のACK欠落/不正注入はfixture helperへのprivate引数・入力bytesで行い、本体supervisorをstub化しない。既存M2 hash検証bypassは追加しない。

## 7. Deadlines / errors / cleanup

ACK待機deadlineはrootのidentity/PID publish完了時から2秒で一度だけ作る。既存test全体3秒（取消しcase4秒）の内側へ収め、時間を足し直さない。通知pollは既存方式同様の短いbounded pollで、ACK欠落・書込失敗・identity不一致・退出前孫終了・期限到達はfixture不成立とする。ACK無しでroot exit0にfallbackしない。不正な完成ACKは即失敗、未作成だけ期限内で待つ。

失敗rootは固定nonzeroで終了し、既存supervisorにJob回収させる。親はACK failureをlatchし、同定不能/孫欠測/cleanup未確認をPASSにしない。初回waitが258なら後続signalや救済で置換しない。失敗時も非panic Drop、同定済みhandle限定救済、未確認temp保持を維持する。既知の自己所有tempだけを明示回収し、D1が保持した既存tempや未知processを探索/削除しない。

0ms oracle、ready2秒、test3–4秒、watchdog8秒、child15秒fallback、救済wait2秒、製品60秒/512MiBを維持する。追加ACK/readyは各256bytes、専用fixture dirにだけ保存する。教材・原音・履歴永続化の順序/回復を変更しない。root先行exitと孫の継承pipe保持という試験目的は、退出前の親の観測可能性を確保した上で維持される。

## 8. Acceptance / independent oracle

| ID / 対応 | 必須証拠 |
| --- | --- |
| A1-01 / AH-AC09 | R2採用diffと14自己試験。poison後非panic、seal前後のclose-only、初回258保持、cleanup未確認temp保持、世代/容量/24callを元期待値で確認 |
| A1-02 / AH-AC09 | 手書きACK/readyでroot/creation/child不一致、途中/余剰/上限超過、欠落、期限到達を拒否。未登録/sealed/旧世代からのACK発行を拒否し、後発ACKで既存失敗が成功にならない |
| A1-03 / AH-AC09 | 実tree-exit：親Watchdog孫登録→双方live→ACK→rootが孫liveのreadyをpublish→root exit0→supervisor完了。同一identityの保持root/孫handle初回wait0、reap=false、救済0、cleanup理由なし、Drop後専用tempなし |
| A1-04 / AH-AC09 | ACK withheld/mismatch fixtureは有限に不成立となる。fake成功・OpenProcess失敗のoptional通過なし。元sourceのtree-exit欠測ログを保持し、欠測条件を新oracleへ渡す決定的fixture自己試験ではFAILになることを示す |
| A1-05 / AH-AC09 | 元cancelおよび全unitを新EXEで実行。兄弟/bytes保護・世代/初回wait・reap値を記録。元原因未確定を維持し、再現時は停止して同定証拠を返す |
| A1-06 / M1・AH-AC09 | oracle不変比較と、同じ最終unit/child EXEを使った外側Job不在のM1別実行。親強制終了後、救済前child終了・未resume・兄弟生存・watchdog非救済を要求 |

tree-exit test末尾の「終了後OpenProcessできたときだけwait」は削除し、登録済み同世代Watchdog孫とrootがsnapshotへ含まれることを必須にする。ACKとreadyは初回wait採取後に読み合わせ、最初のwait前へI/O待ちを入れない。test内でFixtureを明示dropした後にdiagnosticsのcleanup理由とtemp不存在を確認し、test集計PASSとcleanup欠測の併存を認めない。元cancel/root-only/M1の期待値は緩めない。

## 9. Snapshot / staged handoff

親は編集前原本3本を`artifacts/cancel-a1-before/{process_unit.rs,praat_child.rs,lib.rs}`へbyte一致で保存済みであり、再作成/上書きしない。process.rs/process_windows.rsと変更前unit/child EXEは既存cancel-before、D1源泉2本は隔離R2とhash一覧を参照する。Pre-gate PASS後、編集直前に原本と保存hashを照合する。作者はR2採用差分とACK差分を分け、統合役はlib.rs機械整形だけ別差分にする。最終source hashを固定後にbuildし、以後変更があれば関連検証と外部M1対象を再固定する。

test authorは2fixtureだけ、統合implementerはfmtだけを逐次所有する。独立reviewerの安全確認後にrunnerへ渡す。元cancel原因の修正を主張する変更やprocess本体診断が必要ならA1を広げず別Packetへ返す。設計→Pre→実装/独立試験→安全確認→新回帰→外部M1→Postの責任を分ける。

## 10. Fresh verification / M1 order

buildは原本manifestで`--locked --offline`を維持する。authorのfocused自己試験/ACK実例後、安全reviewを受ける。runnerは固定EXEでまずM1を除いた全unit、公開Praat/core/schema、no-default、crate fmt/diffを新規実行する。unit外側180秒、build時間を除外し、内部期限は変更しない。

```text
cargo build --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --bin acoustic-test-child --locked --offline
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --all-targets --locked --offline -- --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --test-threads=1 --nocapture
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline
cargo fmt --manifest-path tools/acoustic-analysis/Cargo.toml --all -- --check
git diff --check
```

unitだけへ外側180秒を適用するため、runnerは必要なら上記と同じ選択をcompile済みunit EXEと公開targetsへ分けて実行し、実件数/同一hashを記録する。build込み全cargoへ180秒を混同しない。lib全unitのM2 15件も実Praatを使うため、起動前に`WW_TEST_PRAAT_EXE`を固定原本`F:/Kazuhiro/GitHub/WordWeave5/tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe`へ明示し、存在・96,780,624bytes・SHA256=`22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4`との一致を確認する。source/default探索/binary copyで代替しない。環境は当該process限定である。

回帰を通過後、同じ最終unit/child EXEをhashで固定してM1 exactを既存の外側Job不在条件で別実行する。filtered1を成功件数へ加算しない。新fixture/source hashが変わるため旧M1 GREENの再利用条件は成立せず、旧Closed履歴を保持しつつ新EXEのM1は未受入として追跡する。外部条件を満たせない場合は未検証で停止し、sandbox内の別Job救済や単独通常caseで代用しない。M1の既存45秒外側監視とoracleを維持し、新たなUAC/環境変更を本書で許可しない。

## 11. Stop / remaining risks

ACK failure、同定済み初回258、reap未確認、cleanup欠測、unexpected残存、新M1不成立、回帰/fmt失敗は報告して停止する。既知元cancelの再現ならD2検討に必要なidentity/初回wait/reapを返し、JobActiveとsignal順序を原因と断定しない。ACKのOS I/Oが準備budgetを満たせない環境でも期限を延長しない。

file ACKはfixtureの観測可能性を確保するもので、過去の原因証明ではない。D1の計測が時間配置を変えた限界も残る。原本安全補正後に全件成功しても、旧failureの原因未確定という記録は保持する。未知tempの事後削除・広範process探索・新依存・本体telemetry・実データ試験はA1対象外である。

## 12. Completion / evidence handoff

A1完了は固定原本差分、安全自己試験/ACK実例、独立安全review、新M2全回帰、同じEXEの外部M1再確認、独立Post判定と残件記録までである。新しい合格件数と旧M1/旧D1結果を区別し、全体アプリ納品gateの代替としない。仕様上の新しい利用者選択はなく、方式未決0。本書のリンク/内容/hashを確認してPre reviewerへ提出する。本役はsource変更・試験実行・公開を行っていない。

## 13. 安全review追補：公開後の証拠記録遅延

2026-10-08。固定初版fixture `37E84E5D54FFA6BA29924ECDF7AE5E48DB08487DFD6391CFAC2348649B649F67` のACK公開後からevidence登録までの間にwatchdogが中断されると、rootは正常にACK受理・ready公開・exit0できるが、先行sealによりLateAckPublication/MissingAckEvidenceとなる。これは真の後発ACKと区別すべき偽失敗であり、安全reviewはCHANGES_REQUIREDである。以下を§5–8に優先する補足契約とし、差分Pre PASS後だけ修正する。3source上限・その他期限/oracleは不変である。

### 13.1 PreparedからCompleteへの記録

watchdogはACK file公開前に、現在世代の登録済みroot/孫とlive観測を含むPrepared slotを短mutex内で確保する。slot上限は既存16件、重複世代は拒否する。確保とadmission確認は公開前に済ませ、確保失敗ならfileを公開しない。file I/Oはlock外で実施する。

公開処理が返ったら同じ世代のslotをCompleteへ更新し、I/Oの成功/失敗を記録して待機者へ通知する。既にseal済みという理由だけで、正常公開の結果を遡及的にLateAckPublicationへ変えない。更新は準備済みslotに限定し、別世代へ付け替えない。公開前の世代/root/sealチェック、未登録/旧世代からの発行拒否は維持する。ACK公開後の時刻と記録完了時刻を同一視しない。

### 13.2 初回観測後だけの完了回収

supervisor返却直後は従来どおりseal+snapshotを行い、全owned handleの初回0ms値、返却結果、reap/ready/watchdog等の返却時状態を不可逆に固定する。この前にACK待機・join・追加I/Oを行わない。

その後、tree-exitでPrepared slotが存在する場合にだけ、そのpublisherのCompleteを回収して最終ACK/ready証拠を評価する。Condvar等のfixture内同期を用い、上限は**既存callの絶対deadlineまでの残時間**とする。新しい3秒等を足し直さず、余時間0なら即失敗とする。spurious wakeupも同じdeadlineで判定する。Prepared欠落、publisher失敗、完了未確認、poisonは成功にしない。watchdogの停止/joinをここへ移動せず、次runにも同じwatchdogを維持する。追加thread・製品hook・global設定は不要である。

最終合格には、Preparedの同一identity/live証拠、公開成功、完全一致するACK/ready、root exit0、同世代登録済みroot/孫の初回wait0、reap/cleanup確認をすべて要求する。readyをrootが公開してexit0した事実はACK受理がroot終了前だった因果証拠である。公開後の記録遅延だけを後発ACKとして拒否しない。一方、root失敗終了後にACKが公開された場合はready/exit条件を満たさず不合格のままである。最初の258/WAIT_FAILEDを後続signalや完了通知で上書きしない。

### 13.3 決定的な回帰と再固定

private fixture seamでACK公開後・Complete更新前を停止し、初回0ms採取を実行してからpublisherを解放する順序を固定する。正常ACK/ready/root exit0・初回全0なら、記録遅延があっても合格することを確認する。sleepや偶然のスケジューリングを利用しない。元実装の偽失敗をこの順序のREDまたは既存コードの同じ境界の期待違反として記録する。

対照として初回258が後で0となっても失敗を保持すること、実際のACK不成立/後発/公開失敗、Complete未達/期限切れ、次世代の独立性を確認する。14採用自己試験・A1関連自己試験・実tree-exit/cancel・共有fixture回帰を再実行し、重複filterを件数へ重ねて足さない。修正は2fixture内、lib.rsは既に完了したfmtから変更しない。新source/EXEhashを固定して再安全reviewへ返し、PASS前に全unit/M1へ進まない。
