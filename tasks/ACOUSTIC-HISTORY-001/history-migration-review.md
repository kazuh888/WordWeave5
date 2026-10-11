# I2a-M 独立コード・受入レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。reviewer history_gate_review、計画指定gpt-6-astra/high。read-only回答を親が保存した。実行metadata・usage counterは未取得である。

## 静的コード・oracle — 阻止0、Post前

入力は[正本設計](design-i2.md) SHA1375E2A1…1090EAF、[限定設計](history-migration-design.md) SHA27D52330…7F040AA、[独立21件](history-migration-tests.md)とG Post版snapshot `artifacts/history-migration-before`との差分である。

- store.rs:178：target=1即時no-op、target=2の必須・重複wire版、現v1構造・サイズ検査。
- store.rs:213–247：安全path→既存exact→Before→再確認→新規時publish最大1回→Ok/Err共通bounded exact readback→After。callback後の既存退避消失も再書込せずIo。
- store.rs:250–310：root祖先/backups/leafの種類・reparse・I/O不明拒否。
- store.rs:709 / commit.rs:134：and_faultsでgate1回、通常v1の移行旧read省略、教材旧metadata/escaped intent検査維持。
- M21件：固定手製bytes/hash、拒否ledger、実atomic後Err、実sharing violation、After失敗後retry、v1縮小/target=2拒否対照。

G/旧test不変、M test節は作者固定hashに一致した。writer本体とG integrationも不変であり、現差分で既成功writer/recoveryの100,000件境界等を再実行する具体的必要はないと判定した。品質gateの免除ではなく、変更影響に基づく今回の検証範囲である。

| 前後一致の対象 | SHA-256 |
| --- | --- |
| src/store.rs | 364613CB7E769A05D63BEAB7D735A94DE68570CF1213AA344EE77E7E01DBB9EF |
| src/commit.rs | D70D1CC065409B526789A6991ACDB6876F9782B4D94C67396AC445D55B088A1D |
| tests/audio_history_migration.rs | 862FE8A4DD0A63E4254F77315E1AE12AA596AF2D3A3F4C2ACE7FCA0225BB84EF |

編集・build/test・再委任は実施していない。受入残はM21＋旧store/commit17＋G24の独立62件GREEN、固定hashと清掃確認である。RED未到達publisher/readback/After retry/path全配置/版拒否後段を含める。codec・公開v2受付・recover hookはS/R未達であり、Mだけで全移行保護完了とはしない。

## 最終Post — PASS / Closed

同reviewerが[独立runner](history-migration-runner.md)の新ログ・現物hashを照合し、M限定Post PASS、阻止0と判定した。M21＋G24＋旧17の62件成功、失敗0・ignored0は実装者自己試験と別の独立実行である。

RED未到達のreadback/After失敗後retry/path各配置/版拒否後段がGREENであり、既存・新規backupの実sharing violation 32も記録済み。source2・integration2・EXE3の現物hashはrunner前後と一致し、G/M test節4も静的レビュー時の固定値と一致した。TEMP前後0、fmt/diff成功を確認した。

この判定はMの退避helperと既存保存への接合だけを閉じる。S/Rのcodec・公開v2受付・recover hook、I10、native・実データ受入は未完了である。
