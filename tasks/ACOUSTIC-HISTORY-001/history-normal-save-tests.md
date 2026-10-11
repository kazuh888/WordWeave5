# I2a-N independent test preparation / RED

指定 ww-test-author / gpt-6-astra / high、runtime metadata・使用量counterは未取得である。TDD skillの作者工程だけを担当する。入力は限定設計SHA `45A7E9C9B08114D9EC00C39B5A3E252F1CB21F0136C62130C82DD1C78651895F`、正本N §4/7、親のS PostClosed / N PrePASSと2file所有である。

所有はstore新 `i2_normal_save_tests` と新 `tests/audio_history_save.rs` のみであり、本体・旧/M/S/G testは変更しない。旧試験は81件のまま保持する。

## 固定oracle

新18件（lib15 / integration3）。N01は全候補非履歴fieldの手製sentinel、最新Aに対するNone/旧B/不正root、入力不変、旧save wrapper、新規missing→Noneを確認する。A/Bは手製82-byte rootであり、独立.NET SHA256はA `ad4eb5ad867a175c2c85b92de9486ce9a68e76af08b36e1a5e44433670f7f7f0`、B `7852cc9a55f3f85fdfba7b2edb9ecdf814cf35787df7fe13fd6ac2b85b73af97` である。

N02はraw helperの無decode、source未知/破損、root欠損、健全root→trial→originalの局所未知schema/欠損/破損、候補非履歴不正、strict pending、旧source/完成候補上限とexact上限。3-object fixtureは製品writerを使わず作り、正常時の具体的closure集合/合計bytesと、局所破損後もpartial rootは読める対照を置く。

N03は既存Mの550-byte raw v1/固定hashを再利用し、新入口の正常/Before/Afterとhash・日別exact bytesを検査する。N03bは日別leaf directoryで先行hashだけ残るledger、既存hash/旧v2対照、日別reparse拒否・正常な既存同日fileの保持である。日別atomicの実I/O失敗は検査拒否と区別し、未承認の追加seamには依存しない。

N04は旧wire2/raw v1/不存在に書込なしOk/Err、実atomic next後Ok/Err、old==next優先。N05は正常第三bytes・破損第三bytes・既存file消失・実共有拒否の各exact子processで、共通blockとnormal/material/history writer、別root/別表記、pure readを確認する。N06はcallback再入と同root/別rootのbarrier競合である。enter/exit counterと実publish event/releaseで非重複を観測し、「一定時間何もない」だけをoracleにしない。

N04–06は同EXE/exact1case/成功marker/15秒kill-reapの隔離で親global状態を保護する。新permission/UAC/ACL/OS変更・global reset・実data/API・再委任は使用しない。

## 作成時点

[prebuild source/prefix/module hash](artifacts/history-normal-save-tests-prebuild.log)。store新節前prefixは受領scaffold SHA `9A67B9D34B313854843BDBDD512F47713B8C9EB634D1F530807D5DD2F63B5A04` に完全一致する。新節/integrationのrustfmt、diff checkは成功した。

親から日別I/O用追加口の文書delta/独立Pre待ちを受領したため、**compile未開始**である。追加口の承認とscaffoldの逐次所有切替を待ち、旧81件と新件数を一回の集約compile・固定EXEで検証する。未コンパイル/未実行を行動REDに数えない。

親から日別delta独立Pre PASS・追加2scaffoldへの所有切替を受領した。新18件・整形済みの区切りで編集を停止し、**source/build所有をいったん親へ返却する。cargoは未実行**である。現store full SHA `E21943DD04D089121AF646750C5EC7DDCE76B312AAB8994AF2A19A30FDD1C3D0`、N専用module SHA `B143EF0A13FF4A19E70445EC4DCC00B7A3E6F20C8957EC0F58712A2E5F14EC2A`。追加口を受領後、承認文書全文→N03c→集約compile/REDへ戻る。

## 日別 delta 受領後の固定・RED・所有返却

承認済み限定設計 SHA `46504FBFE45C01FF6A5140794220A3CF06BAAB91D31B48D103420CA13A32EEDE` と正本 SHA `7E93546677EDE42F1B194A277BC222463866D2392D78524C69B033905BCF9024`、追加 2 private scaffold を受領し、N03c だけを追加した。新 N は **19 件（lib 16 / integration 3）** である。N03c は日別 D 内で所有 leaf を share=0 で作り、独立 read/rename OS probe、同じ `atomic_write` の実 Err、P=0、metadata 不変、非 block、先行 M hash と fixture sentinel/probe を区別した exact ledger を要求する。fake Io や ACL 操作では代用しない。

期待値固定後、`cargo test --locked --lib --test audio_history_save --test audio_history_progress --test audio_history_migration --test audio_history_gate --no-run --message-format=json` を **1 回**実行し、3m05s で成功した。[build JSON](artifacts/history-normal-save-tests-build.jsonl) / [stderr](artifacts/history-normal-save-tests-build.stderr.log)。compile error はなく、再 build も行っていない。

固定 5 EXE を非管理者の通常環境で実行した。引数は lib が `store:: commit:: --nocapture --test-threads=1`、integration は `--nocapture --test-threads=1` である。exact 子 process の内部結果を二重計数しない。

| 対象 | PASS | FAIL | ignored | 証拠 |
| --- | ---: | ---: | ---: | --- |
| lib（旧 58＋新 N 16） | 59 | 15 | 0 | [log](artifacts/history-normal-save-tests-lib.log) |
| 新 save integration | 0 | 3 | 0 | [log](artifacts/history-normal-save-tests-save.log) |
| progress integration | 4 | 0 | 0 | [log](artifacts/history-normal-save-tests-progress.log) |
| migration integration | 3 | 0 | 0 | [log](artifacts/history-normal-save-tests-migration.log) |
| gate integration | 16 | 0 | 0 | [log](artifacts/history-normal-save-tests-gate.log) |
| 合計 | **82** | **18** | **0** | [非管理者・exit・前後 EXE hash](artifacts/history-normal-save-tests-run.log) |

旧 81 件はすべて PASS。新 N は preflight 拒否・不変・非 block 対照 1 PASS、行動 RED 18 件である。fixture failure / timeout はない。legacy wrapper は実保存した root が A でなく null になる RED。他は新口の固定 Err/Io、未発火 M checkpoint、先行 hash 不在、callback 回数 0、publish interval 0 が対応する RED である。preflight の 11 行には正常 3-object closure の具体集合/bytes 検証と、参照先未知/欠損/破損後も root 部分読取可能な対照を含み、全行到達した。ただし固定 Err でも通る拒否対照だけを N 実装完了の証拠にはしない。

未到達境界：N03c は D=0 で停止し、**日別 atomic 実 I/O 失敗・OS probe・metadata P=0・非 block・ledger の全確認は GREEN 時必須**である。N05 は独立 read probe の Win32 共有違反 32 が実成立したが、新 publisher 後の再読拒否・共通 block は未到達である。N01 の後続 B/不正 root、N03 Before/After 分岐、N03b 後続対照、N04 旧状態/戻り値組合せ、N05 全 writer/純読取、N06 再入と実区間非重複（別 root を含む）も、最初の RED 後の assert を成功扱いせず GREEN で全到達を確認する。N03b reparse fixture の作成・拒否対照は通過したが、固定 Err のためその拒否理由の実装検証ではない。

固定 source は [実行前](artifacts/history-normal-save-tests-fixed.log) と [実行後](artifacts/history-normal-save-tests-final.log) に記録した。store 全体 SHA `E5F40F1AA8081CBD087E29B06730C5951D257F6F60D3E736CCC132882A029BF5`、N 節前 prefix SHA `2C10FEC04AEB2FD4696820A1FDB9C94FD0A493D537A13399A863D3A1A448BAF3`、N module SHA `78CA74EF7ABE444FE571AFAD70193CC6751F5348EDF41B54C06BF7173B4748AB`、新 integration SHA `296E07F642FB691A23C846657769434ECBA63D2EE708B286605DA97142766E76`。旧 store test suffix は baseline と完全一致、commit と既存 3 integration hash は変更なし。5 EXE の [path/hash](artifacts/history-normal-save-tests-exes.log) は実行後も一致した。

新 module / integration の rustfmt と diff check は成功、所有 temp prefix 残存 0。hash 採取補助名と PowerShell alias の衝突は採取だけ訂正し、source や期待値に変更はない。使用量の開始/終了 counter と runtime model/effort は未取得であり推計しない。ここで **source/build 所有を親へ返却する**。本体実装・GREEN・独立 Post は次工程である。

prefix 採取範囲の補足：上記 `2C10...BAF3` は新 module 前を `TrimEnd + LF` とした従前方式（106593 chars）であり、実行後も一致した。module 直前の空行まで含めた raw prefix（106594 chars）は `0BF5CA7534207EB782B66199036CA851607D0F5679F5532E2985A0F48154BCB5` である。方式差であり、本体差分ではない。

## N03c OS oracle 限定訂正

本実装の初回は 100 件中 99 PASS / 1 FAIL であった。[保持する初回失敗](artifacts/history-normal-save-implementation-lib.log) の N03c は D=1・同 production atomic_write の実 Err・P=0・通常 Err・独立 read 共有違反 32 まで到達し、rename raw 値が 32 でなく 5 の比較で停止した。後続 ledger/nonblock はその時点では未到達である。

独立作者の再照合と reviewer/親の承認により、設計 §4 は rename raw 値の観測を求めるが 32 固定を規定していないと確認した。raw 5 だけで共有 lock が原因とは断定できず、恒久的な path/権限不備との区別には対照が必要である。訂正は `32 | 5` 許容ではなく、同 lock 下の rename が実 Err であることを別 boolean に保持し、raw 値を記録する。read32・実 atomic Err・D1/P0・全 ledger・sentinel・temp0・nonblock は維持する。それらをすべて assert した後だけ、解除済み同 source/destination の rename 成功＋同 path/bytes の atomic_write 成功を fixture 操作として追加し、各段階で fixture 差分だけの exact ledger を照合する。

所有は N03c 1 case と本記録だけである。[訂正前](artifacts/history-normal-save-tests-n03c-before.log) / [訂正固定](artifacts/history-normal-save-tests-n03c-fixed.log)。store から N03c だけを除いた SHA は前後 `08EBBF10A56BD17010E7EF9FA8F463989252226DC8DE1BB066E3655DF03EC5F7` に一致し、本体・他の case は不変。訂正後 store SHA `93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387`、N03c SHA `19BF098C656373FCBD3229310E9E126BD6C39A79817DD916050223001D582B29`。test 名・件数・API/gate/品質条件は変更していない。新 module fmt と diff check は成功した。

`cargo test --locked --lib --no-run --message-format=json` は 1 回・1m25s・exit 0。[build](artifacts/history-normal-save-tests-n03c-build.jsonl) / [stderr](artifacts/history-normal-save-tests-n03c-build.stderr.log)。固定 lib EXE `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe` SHA `127889DD633931EB4BE66662A16137C4F4EB994550F08921F1D7D050BBF2CE2A` を非管理者環境で実行した。親の指定どおり、[N03c exact](artifacts/history-normal-save-tests-n03c-exact.log) は **1 PASS / 0 FAIL / ignored 0**、続く [lib 74 件](artifacts/history-normal-save-tests-n03c-lib.log) は **74 PASS / 0 FAIL / ignored 0** である。exact は lib 件数に加算しない。[非管理者・exit・EXE hash](artifacts/history-normal-save-tests-n03c-run.log)。

N03c は locked rename=Some(5)、read=Some(32)、同 atomic_write 実 Err を観測し、D1/P0・旧正本/対象物 ledger・M hash・sentinel・temp0・非 block の全 assert を通過した。その後に同 source/destination の rename と同 path/bytes の atomic_write が成功し、各 fixture 差分だけの ledger まで到達した。したがって、本 fixture の恒久的な source/destination/権限不備で失敗していたという仮説には反証が得られた。raw 5 一般の原因を共有 lock と断定するものではない。

[実行後 hash/cleanup](artifacts/history-normal-save-tests-n03c-after.log)。store・commit・全 integration source は固定値から不変、4 integration EXE も初回実装時の hash のままであり、26 件の再 build/再実行はしていない。所有 fixture prefix 残存 0、追加削除なし。**source/build 所有を親へ返却済み**であり、独立 runner による固定 5 EXE / 全 100 件の新実行・Post 判定は次工程である。初回失敗履歴を上書きせず、実装・設計・他 99 case の期待値は変更していない。
