# I2a-G 保存排他の独立レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。reviewer `history_gate_review`、計画指定gpt-6-astra/high。起動指定と実行metadataは別であり、後者とusage counterは未取得。親がread-onlyの回答を保存した。

## コード・oracle判定 — 阻止0件、Post前

固定[設計](design-i2.md) SHA `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA` §3/4、編集前snapshot `artifacts/history-gate-before` と現差分、[独立24件](history-gate-tests.md)を対象とした。

- store.rsの単一mutex、private非Send guard、同thread再入拒否、別thread待機、drop時の所有印解除、poison/block維持を確認した。
- metadata/historyの8入口で取得から副作用終了まで保持する。load/save→recoverのunder_guard接合、under_guardでのblock再確認、HistoryError Io/Busyの型変換と整合する。
- commit.rsのroot・祖先・leaf検査、正常親のleaf欠損だけfalse、UnsafePath/Io分類、互換boolのfail-closedを確認した。既存上限・復旧照合の緩和はない。
- 新24件の正常対照→再入拒否→解放後再試行、callback poison、並行区間と正常完了、gate-free出口、15秒child回収・exact 1件・固有markerを確認した。有限scheduleの検証であり全schedule証明ではない。通常fileはexact bytes、live app.lockは長さのみである。

code-review-and-quality、review-reliability-data-integrity、security-reviewを適用した。編集・build/test・mutation試験・再委任は実施していない。

| レビュー前後一致の対象 | SHA-256 |
| --- | --- |
| src/store.rs | CC8C39E13EB56591DBFF698FCF2E05D5803B8DB0FD7710E661E8BEE120103154 |
| src/commit.rs | 78C14B81B7801590CDDF082EC06F7BA5F0DD5A06CFCD13AB0BB5E46E3399F78E |
| src/audio_history/store.rs | 7532A07DA9BED7F79AE0B1E28806CFE881BE3B2AC5973E73F39F11F44B51FC67 |
| tests/audio_history_gate.rs | 5A9706DCDB41156119D4CB014692E5FF7748865950A254A32F0C7D1093B86DCC |

store/commitの新test suffixも作者固定hashに一致した。期待値変更なし。

残る受入証拠は、新24件のGREEN（RED未到達のblock/poison/reparse後段を含む）、既存store/commit・writer/recovery回帰、独立runnerの対象source/EXE固定と清掃確認である。現時点は最終Post PASSではない。schema/migration/結果不明判定/read_progress/UI完成は本判定の範囲外である。

## 最終Post — PASS / Closed

同じ独立reviewerが[runner結果](history-gate-runner.md)と新ログを追加照合し、I2a-G限定の最終Post PASS、阻止0件と判定した。lib25（新8＋旧17）/gate16/writer37/recovery34の112件成功・失敗0・ignored0は実装者自己試験と別の独立実行である。128 filteredは対象外で成功に数えない。

RED未到達のblock/poison/reparse後段は新24件GREENでClosed。source3・gate test・EXE4の現物hashも、runner前後と静的レビュー値に一致した。清掃前後0、新規残存0、fmt/diff成功は報告・対応ログと整合する。コード・期待値は修正していない。

この判定はGの保存排他境界だけを閉じる。I10全体検証、後続schema/移行/結果不明判定、純粋read_progressとUI接続、実データ/実機受入は含めない。
