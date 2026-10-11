# I2a-N 通常保存の独立レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。reviewerは計画指定 `ww-reviewer / gpt-6-astra / high` のhistory_gate_review。レビューはread-only、親が結果を保存する。実行metadata・usage counterは未取得である。

## 初回静的確認：本体阻止0、試験阻止1、Post未了

対象は限定設計46504FBF…32EEDE、正本7E935466…CF9024と、固定store `1282F59E989FB3301816E763DB7A8A2728432681835A20483CE6A91B41CFA595`。新N module `78CA74EF7ABE444FE571AFAD70193CC6751F5348EDF41B54C06BF7173B4748AB`、save integration `296E07F642FB691A23C846657769434ECBA63D2EE708B286605DA97142766E76`。前後hashは一致した。

本体は単一gate、最新root置換後validate、同raw snapshotの共有、next優先の成功/旧状態/不明判定、日別保存失敗時の正本publisher未到達、S readerの安全分岐維持を満たす。旧store試験はbaselineと一致し、commit/既存integrationも不変である。

初回自己検証は100件中99 PASS/1 FAIL。N03cだけが実read raw32に対してrename raw5を観測し、renameにもraw32を固定したassertで停止した。D=1、同production atomic_write実Err、P=0、通常保存Errまでは到達したが、後続ledger/nonblockは未到達である。初回失敗は[実装記録](history-normal-save-implementation.md)と同artifactsログに保持する。

P2（試験）：CreateFileの共有競合の観測を別操作のrenameへそのまま適用する根拠はない。[CreateFileW公式仕様](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)は共有modeと後続openの競合を説明しているが、今回のrenameが必ずraw32になることを規定していない。raw5だけから原因を共有lockと断定することもできない。

独立作者へ返す最小訂正は、locked renameのErrとraw観測記録、read=32、同production atomicの実Errを維持することである。既存ledger/sentinel/temp0/nonblockの全assert後に、lock解除済みの同pathでrename成功と同atomic_write成功を確認する対照を加える。対照の変更はfixture操作として分離し、通常保存のretryとはしない。単なる32/5の許容列挙にはしない。限定設計はraw観測を求めるがrenameのraw32固定は規定せず、API/品質gateの変更は不要である。

作者の独立診断・限定訂正、N03c末尾までのGREEN、固定5EXEの独立runner、最終Postが残る。本体の静的判定だけでNをClosedにしない。

## 限定訂正の静的確認・作者GREEN

独立作者はN03cのみを訂正し、storeは `93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387`、同case除外storeは前後とも `08EBBF10A56BD17010E7EF9FA8F463989252226DC8DE1BB066E3655DF03EC5F7` である。reviewerは現物とhashを確認し、P2の静的修正確認PASS、新たな静的阻止0を返した。

locked rename Err/raw記録、read32/実atomic Err/D1/P0と元の全ledger/nonblockを維持し、その後にfixture専用の同path解除後rename/atomic成功と各ledgerを確認する構成である。通常保存のretryではない。

作者のlibのみ1回compile（1m25s）後、N03c exactは末尾までGREEN、lib74/74 PASS・ignored0。locked rename5/read32、同atomic実Err、元のledger/temp0/nonblock、解除後の両対照まで到達した。新lib EXE SHAは `127889DD633931EB4BE66662A16137C4F4EB994550F08921F1D7D050BBF2CE2A`、4integration EXEは初回から不変。初回失敗を削除せず、固定5EXEを独立runnerへ渡した。全体Postは独立実行後とする。

## 最終Post：PASS / Closed

独立reviewerは[新しい独立実行](history-normal-save-runner.md)の100 PASS / 0 FAIL / 0 ignored（N19＋旧81）、N03cの全ledger/temp0/nonblockと解除後同path対照、N05/N06の後段assert到達を確認した。source/test/EXEの11hashはrunner前後と現物で全一致し、patch一致、fmt/diff成功、TEMP残存0も確認した。

固定store `93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387`、lib EXE `127889DD633931EB4BE66662A16137C4F4EB994550F08921F1D7D050BBF2CE2A`。静的判定後の本体変更はない。N限定Post PASS / Closed、阻止0、P2閉鎖とする。初回失敗と訂正根拠は保持する。

閉鎖範囲はNの通常保存だけである。R/T・後続consumer・I10・native受入・製品提供は未完了である。
