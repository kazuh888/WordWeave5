# I2a-R 教材復旧の独立レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。担当は既定history_gate_review（ww-reviewer / gpt-6-astra / high、read-only）。親が結果を記録する。実行metadata・usage counterは未取得である。

## 独立試験の先行静的確認：P2補強3点

新R25件のcompile開始後、本体未接続の段階でoracleを独立確認した。対象commit `FDF5BB32…BFF974`、material integration `E6502498…E52589` は前後不変、新R節前prefix `E7492080…5B1AD` は受領scaffoldと一致した。ここで実行成功は判定していない。

1. M callback後のpending改変（commit.rs:1661）：Err・書込0・ledgerだけではprocessが継続可能な誤実装を検出できない。設計§3の全事前検査後のpending第三状態はunknown/blockである。既存隔離childでWriteBlockedと後続writerの無変更拒否を追加する。
2. 重複key（commit.rs:1087）：当初fixtureは必須fieldも欠け、重複を潰す誤実装でも別理由で拒否する。正常V2全文に同値version/audio_history keyだけを重複させ、正常対照成功・重複だけInvalidDocumentを確認する。
3. root bytes権限（tests/audio_history_material.rs:160）：A→Bはhashが異なるためhashだけ比較する誤実装でも通る。旧A(hash同じ・bytes83)→新A(bytes82・closure健全)を各current状態へ追加し、全ledger不変拒否を確認する。旧Progress構造validate成功の対照も必要である。

いずれも既定契約内の試験補強であり、本体の欠陥判定・期待値緩和・追加公開APIではない。親は同独立作者へ限定補強を返した。先行compile/初回ログを保持し、同時buildせず、補強後の最終sourceでREDと旧100件維持を確認する。

その他、実I/O/解除対照、known retry、archive確定、unknown child隔離、全ledgerに追加の静的阻止はない。時刻取得不能・初回archive名衝突は未測定のまま保持する。R本体・独立runner・最終Postは未了である。

## 限定補強の再確認：静的解消、実行未了

初回compileは新R節のmodel名競合とProgressHistoryHead import不足（2原因、7 diagnostics）で失敗し、行動REDへ数えていない。作者はそのimportとP2の3点だけを補強し、再compile前に同reviewerがread-onlyで差分を確認した。

- M callback後のpending改変は既存child内にWriteBlocked/全writer無変更拒否/pure readを追加し、元ledgerも保持した。
- 重複keyは正常V2全文を母体とし、重複だけの違いとした。
- 同hash・旧bytes83→新bytes82について、旧構造validate成功と新closure健全を前提に各current状態の拒否を追加した。

固定commit `4C1976ECD79705DAA2DFABC6D8921ED56F1877CA5D36F84121AEA80BF5C64DE6`、R module `B26C43DB7D68EE8E97EB55A75C14921E722F7E2F3B98E72FAE3EA492C03B1BDB`、material integration `5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683` を現物照合した。新節前prefix E7492080…5B1AD、storeと旧100件は不変、新25件（22＋3）を維持する。

P2の3点は静的解消、新阻止0。importも原因に対応するがcompile成功はまだ未確認である。作者へ再compile/行動REDを返した。各指摘の実行上の閉鎖、R本体、独立runner/Postはこれからである。

## 本実装の先行静的確認：阻止0、Post未了

独立REDは旧100全PASS・新R25の2PASS/23行動REDであり、fixture failure/timeout/ignored/temp残存0であった。本体接続後、同reviewerが固定差分を確認した。

- commit `24414D9610D0970DDE11B09C17F62BB92051BD066245F5F316FD0F466A79C6A6`
- store `38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF`
- material integration `5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683`

strict preflight、root完全一致/closure、intent旧exactによるM退避、M callback後のunknown block、write/checkpoint後の実再読、known retry、finishのexact確定を確認し、静的阻止0と判定した。旧wire1 replay、既存saveからNoProgressFaultsへの経路を維持する。store差分はcodec bridgeだけ、既存save本体・旧commit試験・全store試験は不変、新R moduleもB26C43DB…3B1BDBのままである。

compile/全枝GREEN/独立runnerは未受領であり、Postではない。時刻取得不能・初回archive名衝突は静的確認だけで実測済みとはしない。

本体の初回build中にarchive直前の再確認が補強された。実装者が把握したのは最終hashと依存eframeのcompileログまでであり、本体rustc開始時刻の独立証拠はない。親は初回ログを保持し、終了後に固定source hash→aggregate no-run再確認→125件実行の順へ戻すよう依頼した。曖昧なsource/EXE対応を検証済みと扱わない。

## 最終Post：PASS / Closed

[独立runner](history-material-recovery-runner.md)の新実行125 PASS / 0 FAIL / 0 ignored（R25＋旧100）を同reviewerが確認した。重複key、同hash異bytes、M後pending改変のblockを含む全後段assert、実I/O拒否、解除対照が通過し、先行P2の3点も実行証拠で閉鎖した。

16hashはrunner前後と現物で一致し、patch一致、fmt/diff成功、TEMP残存0を確認した。初回build中編集のログを採用せず、固定source再build後の6EXEと独立新実行を根拠としている。store38D4516D…F312EF、commit24414D96…79C6A6は静的レビュー時から不変である。

R限定Post PASS / Closed、阻止0。時刻取得不能・初回archive名衝突は静的確認のみで実測済みとはしない。T/V/P、I10、native受入・製品提供は未完了のままである。
