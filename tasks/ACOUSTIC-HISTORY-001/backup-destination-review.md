# I2d-B バックアップ書き出し保護のレビュー

## 追加試験接合の設計レビュー

2026-10-12、独立reviewer `export_destination_final_review`（指定 `gpt-6-astra/high`）は、[design-i2.md](design-i2.md) H4直後の試験接合を **Pre PASS・阻止指摘0** と判定した。対象文書SHA256は `FAC275146D0A509506E4DAB310A5A52FD2EC8C601EE46C4534C64F9979C306C7` である。

3点のobserverは `cfg(all(test, windows))` に限定し、同一export本体のguardを借用する。返却値は `()` であり、製品APIやI/O結果を差し替えない。各点のReentrant観測と、点間からmanifest結果確定までの単一guardの生存を確認する静的レビューを分けた証拠範囲は適切である。

有効な媒体を用意したうえでAfterDirectoryに `dest/assets` を通常fileとして作ると、既存 `AssetStore::put` の `create_dir_all` がtemp生成前に実I/O失敗する。独立ledger、厳密な観測列、fixture sentinelの識別、登録RAII、有限子processと親のcleanupにより、時間競争や結果差替えなしで確認できる。

合格は追加設計だけである。hook実装・本体接続・実行証拠とI2d-B全体の最終判定は未完了。reviewerの編集・試験実行・再委任はなく、実行metadataとusage counterは未取得である。

## コードとテスト設計のレビュー

同日の独立reviewerは **PASS・阻止指摘0** と判定した。対象 `src/backup.rs` SHA256 `CF0355486CB563400485ABE4771176D50C37F13C1A54141D9BF6AF65E77784F7`、`tests/audio_history_backup.rs` SHA256 `1BCA808ED56BE0DB8CCE33F9BE4BCD5899A68017822342DFF71AB32A0750E23B` はレビュー開始・終了とも一致した。

内容preflight前からmanifestのResult確定まで同じexternal guardを保持し、途中のdrop／再取得、locking helperの入れ子がない。解決済みdirectoryを媒体・子fileへ使い、manifestを最後に書く。内容拒否、上限、v1投影、内部restore／preserve、既存テストを維持している。

observerの型・登録・呼出はtest-onlyである。独立期待値、厳密な観測列、実mkdir失敗と再試行を検証できる。子processは20秒期限・exact 1件・完了tokenを確認し、親fixtureを回収する。junctionはledgerで追わずcleanupで先に外す。作者追加482行／193行は実装前後で全文一致し、oracle変更はない。

この判定時点では自己確認51件の結果と独立runnerは未確認である。自己確認GREEN後に独立runnerへ渡せるが、最終Post／I2d-B Closedは独立実行後に判定する。

### 自己確認結果の接合

同日の[実装報告](backup-destination-implementation.md)で新17＋旧34の51件成功を受領した。親はsource2本とEXE2本の実物hash、run-summaryのadministrator=False、native exit0、timeout=False、TEMP残存0を照合した。コードreview対象sourceに変更はない。これは自己確認の結果と既存コードreviewを接合したrunner-ready状態であり、独立runnerの代替ではない。
