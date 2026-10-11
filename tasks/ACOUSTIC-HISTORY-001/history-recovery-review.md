# I1b-2b 保存回復の独立レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。branch `codex/development-agent-team`、HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋既存dirty差分。独立reviewer `history_commit_review`、指定 `gpt-6-astra/high`。実行metadata/usageは未取得。

## Cycle 1 — CHANGES_REQUIRED（必須試験不足）

対象は[実装記録](history-recovery-implementation.md) stage 2の本体4本・新32件・既存writer37件/readerの固定hash。reviewerが一致を確認した。編集/build/実行は行っていない。

| ID | 指摘原文・現物根拠 | 共通原因 / oracle | 状態 |
| --- | --- | --- | --- |
| R1 / P2 | resume時のintent上限境界が未検証。design-i1b.md W05はescape後524,287/524,288/524,289 bytesとresume超過時の旧intent保持を要求する。store.rs resumeの再構築encode経路に新32件の境界試験がなく、既存37件のprepare試験は代替不可 | 回復受入を正常/fault中心に分解し、quotaと別のserialized上限を落とした。rebase後exact長を手製oracleで合わせ、成功/LimitExceededと旧intent・全object不変を検証 | Accepted、追加試験準備 |
| R2 / P2 | 復旧一覧の100,000件境界が未検証。design-i1b.md回復列挙/W05、recovery.rs operationsのENTRY_CAPに新試験がない。32件paginationでは走査上限を通らない | page件数と全走査件数を別oracleで追跡しなかった。100,000許可/100,001 LimitExceeded、空成功でないこと、無書込を検証 | Accepted、追加試験準備 |

確定したコード欠陥は見つからなかった。旧base閉包/計画再計算、欠損payload非捏造、exact mutation、rebase target制約、intent差分quota、public token再照合、finish対象限定、fault後再試行を確認した。手製fixtureはwriter出力を期待値に転用していない。

実装者の新32 PASS/writer37 PASS/reader37 PASS・1 ignoredは整合するが、独立runnerの新実行ではない。R1/R2追加後のrunner検証・変更入力の再レビューが必要である。I2実Progress/I6実行回数spy/Windows実障害は対象外として残る。

## 補完の境界

追加試験の所有は独立作者へ渡す。既存32件とwriter/readerの期待値は緩めない。本体変更は予定せず、失敗が実装欠陥を示した場合は別の修正packetへ戻す。既存実装で追加試験がGREENでも検証不足の補完として正当であり、人工的なREDを作らない。仕様上限・昇格範囲・並行数・全体gateを変更しない。

## R1/R2 補完Pre — PASS

同reviewerが[packet](history-recovery-boundary-packet.md) SHA `87D13C34F7E76C2C6EB66864BE6FF1EA0E24A41A35A9E683DB5276270C9B3E82`を独立確認した。R1のescape後exact長と失敗後不変、R2の実100k名走査/正常page/LimitExceededの別判定、有限打切り、隔離cleanup、本体欠陥時差戻しが成立すると判定した。中央本文`{}`は名前走査境界の受入を弱めない。編集/build/実行なし。

R1/R2はPre-gate PASSへ更新し、独立作者 `history_recovery_boundary_tests`（指定Astra/high）へtest/build所有を渡した。実行成功・Post Closedはまだ主張しない。

## 追加2試験のコード判定 — PASS、独立実行待ち

reviewerが新test hash `AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567`の末尾2試験をread-only確認した。R1の独立wire/exact長・失敗後tree不変・再試行、R2の実100k/100001走査・正常pageとdiagnostic・sentinel不変・限定cleanupはPre契約に適合し、阻止事項なしである。既存32件の本文は不変、本体4本もstage2 hash不変である。

作者の新2件は各自己実行PASS（R1 0.94秒、R2 113.67秒）。runnerの新34件＋既存writer/reader実行を待つ。コードPASSだけでR1/R2をClosedにしない。

## 最終Post — PASS、R1/R2 Closed

同独立reviewerが[runner記録](history-recovery-runner.md)・raw log・前後hashを照合した。回復34 PASS、writer37 PASS、reader37 PASS/1 ignored、全exit0/watchdog未発火。R1の全3長境界、R2の実100k両page/100001超過/無書込/cleanupと4root不存在が記録に一致した。source7本・EXE3本の前後hashは不変である。

これによりR1/R2はClosed、I1b-2b保存回復のPostはPASSとなった。追加testは上記AA97C2F…版、本体はstage2版である。権限依存1件は今回未実行のままであり、I2/I6・全体提供・実disk故障/電源断の合格を含まない。レビュー中の編集/build/試験なし。I2a-0へ進むためのI1依存gateのみ解消した。
