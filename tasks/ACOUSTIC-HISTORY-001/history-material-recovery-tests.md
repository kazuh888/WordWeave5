# I2a-R independent tests / RED

2026-10-08。計画者指定 ww-test-author / gpt-6-astra / high。TDD skill の作者工程を使用し、本体修正は行わない。runtime metadata・usage 開始/終了 counter は未取得であり推計しない。

入力は限定設計 SHA `0C1109DC7B41DBC2E0ACF7B01366FE6622DAB57404B76DB31E5D75A407DBE951` 全文、正本 §5/7 SHA `C4CF33CE6C7B69D8106D216497B4897C545C0E46169DEE417B7E27E96903B3D3`、最終 Pre PASS、N 最終 PostClosed/独立 100 PASS である。scaffold は store `3F6FEA29456D3C9C002B84C0D2D144574BD343DD7549D33D18D628F5FF6BBF2B`、commit `E7492080DFB8FBE53BE5AD09C4421F95EE96BF94898C06BCE73331BE9C15B1AD`。新 4 口は固定 Err/Io、旧 recover/save へ未接続である。

所有は commit の新 `i2_material_recovery_tests`、新 `tests/audio_history_material.rs`、本記録/log のみ。store、本体、旧 G/M/S/N test は変更しない。親の before snapshot と既存ログは保持する。

## 固定 oracle

新 **25 件（lib 22 / integration 3）**。行列内の枝は個別件数へ加算しない。

| ID | 独立内容 |
| --- | --- |
| R01 | public 手製 wire1 / old=None / 4 中断位置、invalid 旧 deck を許す exact replay と raw archive、再 recover 不変。bridge 元 wire と runtime v2 を分離、raw duplicate/未知/欠落拒否、pending 欠損 no-op |
| R02 | public A→None/B、old=None→Some、旧不正/未知を各 current 状態へ置き全 ledger 不変。private 同 A / None 正例 |
| R03 | old/new deck/Progress/pending の個別 exact / 1 byte 超過、文書/第三 current/既存 backups 型拒否、具体 3-object closure→参照先欠損/破損/未知の public 拒否、typed raw 読取の正常/上限/祖先/leaf/reparse/実 sharing 拒否と解除対照 |
| R04 | pending 旧 raw v1 由来 hash 退避を各 current old/new 状態で書込前確認、既存 exact の非置換と競合拒否、M Before/After ledger と retry、既存 hash 実 read 拒否、M callback 後の pending identity 再確認 |
| R05 | 各 write の no-write Ok/Err、実 atomic 後 Err、old==new 優先、6 material checkpoint と retry。unknown 子 process 6 行（第三 deck、欠損、実 atomic/read 拒否、pending 消失/第三/読取拒否）で全 writer block・純読取・rollback なし |
| R06 | archive no-op Ok/Err、exact 複製、実 rename 後 Err/Ok、実 rename 拒否と同 path 解除後対照、finish 前 directory 作成障害。unknown 子 process 5 行（第三 archive、pending 不明消失、archive 実 read 拒否、第三 progress、pending 保持＋異 archive） |

既存 Temp/tree、独立 M raw550-byte fixture/固定 hash を再利用する。A/B は手製 82-byte root と固定 SHA、3-object は手製 DTO と具体 object 集合/bytes。製品 history/material writer 出力で pending や期待値を作らない。Intent 文字列の escaping だけ serde を利用し、Progress runtime 再 serialize を exact oracle にしない。

各 callback 到達回数・実 filesystem 効果を区別し、固定 Err だけで通る拒否を製品実装完了と扱わない。実 I/O の read raw32、rename raw 値観測、同 atomic 自身の Err、解除後成功を別証拠にする。読み値の差替え、OS code 許容リスト、String parse はしない。unknown ケースは同 EXE/exact/marker/15s kill-reap 子 process とし、reset/poison 解除口は作らない。M callback 後 identity ケースも global 状態の判定を追加せず子 process に隔離する。

時刻取得不能・初回 archive 選定名衝突は現 seam で決定注入できず未測定である。実データ/API/UAC/ACL/OS 設定・公開・再委任なし。native/電源断受入を代替しない。

## Compile 前固定

新 25 件の期待値を固定し、lib＋material/save/progress/migration/gate の aggregate no-run 1 回、以後固定 6 EXE で旧 100 / 新 R を分離する。compile error、fixture failure、scaffold 行動 RED、後続未到達は別に記録する。

## 初回 compile と限定静的再確認への返却

初回 aggregate は exit 101。[build JSON](artifacts/history-material-recovery-tests-build.jsonl) / [stderr](artifacts/history-material-recovery-tests-build.stderr.log) を保持する。原因は新 R 節内の glob import による `model` 競合（3 箇所に重複 diagnostic）と `ProgressHistoryHead` の import 不足、計 7 diagnostics である。署名不足・本体欠陥ではない。compile failure を行動 RED に数えず、この段階で EXE 試験は行っていない。

親の独立静的 P2 指摘 3 件により、次だけを補強した。

1. M callback 後 pending identity 改変は、既存 exact 子 process 内で `WriteBlocked` と全 writer 無変更拒否・pure read を既存 helper で追加検証する。Err/P0/M hash/外部改変を含む exact ledger も保持する。
2. 同値 version / audio_history の重複は正常 V2 全文へその key だけを追加する。正常 V2 の bridge 成功と InvalidDocument を対照にし、必須 field 欠落との交絡を除いた。raw JSON であり Value 経由で重複を消さない。
3. public 権限行列へ同 hash・旧 bytes83→新 bytes82 を追加した。旧 Progress の構造 validate 成功・同 hash/83 bytes を先に確認し、各 current 状態で拒否＋ledger 不変を要求する。新 A の closure 正常対照は既存 fixture を保持する。

明示 import 2 件を含め、変更は新 2 test 所有範囲だけ。件数は 25 のままである。[再確認用固定 hash](artifacts/history-material-recovery-tests-p2-fixed.log)：commit `4C1976ECD79705DAA2DFABC6D8921ED56F1877CA5D36F84121AEA80BF5C64DE6`、R module `B26C43DB7D68EE8E97EB55A75C14921E722F7E2F3B98E72FAE3EA492C03B1BDB`、integration `5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683`。store は受領 hash 不変、commit 新節前 prefix は scaffold `E749...B1AD` と一致する。新節/新 integration 整形・diff check 成功。

親指定に従い、**2 回目 compile は未開始**。source 所有は作者が維持し、reviewer の限定 read-only 確認後に aggregate compile/RED へ進む。本体実装担当への所有返却ではない。

## 限定静的解消後の compile / 行動 RED / 所有返却

親から P2 3 件・import 原因対応の独立静的解消、新阻止 0 を受領した。補強固定 source のまま aggregate を再実行し、**2 回目 compile は exit 0 / 2m55s** で成功した。[build2 JSON](artifacts/history-material-recovery-tests-build2.jsonl) / [stderr](artifacts/history-material-recovery-tests-build2.stderr.log)。初回 compile failure は保持し、行動 RED に加算しない。追加署名や期待値の修正はない。

生成した [固定 6 EXE/path/hash](artifacts/history-material-recovery-tests-exes.log) を `--list` で確認後、非管理者 token の通常環境で直接実行した。lib の filter は `store:: commit:: --nocapture --test-threads=1`、integration は `--nocapture --test-threads=1` である。[前後 EXE hash・exit・token](artifacts/history-material-recovery-tests-run.log)。子 process 内部結果は親 case に含め、二重計数しない。

| 対象 | PASS | FAIL | ignored | 証拠 |
| --- | ---: | ---: | ---: | --- |
| lib（旧 74＋新 R22） | 75 | 21 | 0 | [log](artifacts/history-material-recovery-tests-lib.log) |
| 新 material integration | 1 | 2 | 0 | [log](artifacts/history-material-recovery-tests-material.log) |
| save integration | 3 | 0 | 0 | [log](artifacts/history-material-recovery-tests-save.log) |
| progress integration | 4 | 0 | 0 | [log](artifacts/history-material-recovery-tests-progress.log) |
| migration integration | 3 | 0 | 0 | [log](artifacts/history-material-recovery-tests-migration.log) |
| gate integration | 16 | 0 | 0 | [log](artifacts/history-material-recovery-tests-gate.log) |
| 合計 | **102** | **23** | **0** | 旧100全 PASS、新 R25 = 2 PASS＋23行動 RED |

新 PASS は public wire1 の old=None/Some×4 中断位置・exact archive・媒体保持・再 recover 不変と、private preflight 9 行の全 ledger/非 block 対照である。後者は固定 Err でも通るため制約の実装完了証拠ではない。公開 root 権限拒否、公開 3-object 局所欠損拒否は旧 recover が誤って成功する RED、公開 current=両new の R-M は intent 旧 hash 退避欠損が RED である。それ以外は新口の固定 Io/Err、checkpoint 空、callback 0、未実行の書込状態に対する ledger 差が RED となった。実行した範囲で fixture failure / child timeout は **0**。未到達の I/O fixture が成立済みという意味ではない。

### 到達済みと GREEN 時に残る境界

- R01 の旧 wire1 replay は全枝 PASS。bridge は最初の正常 RAW_V1 に固定 Io で停止し、正常 V2 と補強 duplicate/未知/欠落は未到達。
- R02 は手製 A/B closure と bytes83 旧 Progress の構造 validate 対照が成立した。public 行列は最初の A→None で停止し、同 hash/83→82 を含む後続各 current 状態は GREEN 時に全到達が必要。private A 正例も固定 Err で停止する。
- R03 の具体 3-object closure/合計 bytes、局所原音欠損後に root 部分読取だけは可能な対照が成立した。旧 recover はその欠損を拒否せず成功した。破損/未知の後続枝、上限行列の最初の exact 成功以降、reparse の祖先/leaf/archive-parent 後続枝は未到達。最初の junction fixture 自体は成立し、typed reader の Io 対 UnsafePath で停止した。
- raw reader の実 read 拒否は独立 probe で Win32 32 を観測したが、解除後の正常読取は固定 Io のため RED。既存 M hash の独立 read32 assertion・解除後 ledger 不変・非 block までは通り、再試行の固定 Err で停止した。stub が M の実 read を実行した証拠にはしない。
- R04 の移行 checkpoint、書込前 exact hash、callback 後 identity/block 補強、競合後続枝は未到達。R05 の実 atomic/known retry、全 checkpoint 後続枝、unknown block/純読取/全 writer 拒否も未到達。子 process の最初の write scenario は callback 0 で停止し、実 atomic/read I/O 拒否枝は未実行。
- R06 は archive callback 0 または finish fault 未発火で停止した。実 rename 拒否・解除後同 path 対照、archive 再読/結果不明・全 ledger・nonblock/retry の成立は **GREEN 時の必須検証**である。時刻取得不能・初回 archive 名衝突は引き続き未測定。

最終 [source hash/fmt/cleanup](artifacts/history-material-recovery-tests-final.log) は補強固定値と一致した。store `3F6FEA29456D3C9C002B84C0D2D144574BD343DD7549D33D18D628F5FF6BBF2B`、commit `4C1976ECD79705DAA2DFABC6D8921ED56F1877CA5D36F84121AEA80BF5C64DE6`、R module `B26C43DB7D68EE8E97EB55A75C14921E722F7E2F3B98E72FAE3EA492C03B1BDB`、新 integration `5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683`。commit 新 R 節前全文は受領 scaffold `E749...B1AD` に一致し、旧 test/本体を保持した。既存 4 integration source hash も不変、6 EXE hash は実行前後一致。fmt/diff check 成功、所有 fixture prefix 残存 0、追加削除なし。

ここで **source/build 所有を親へ返却した**。次工程は本体実装・全枝 GREEN・独立 Post である。役/モデル指定・計画 gate は変更せず、runtime metadata・使用量 counter は未取得のままである。
