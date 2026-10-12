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

## 独立実行後の最終Post

2026-10-12、独立reviewer `backup_destination_post`（指定 `gpt-6-astra/high`）は **PASS／I2d-B Closed、阻止指摘・未解決質問0件** と判定した。対象HEADは `b063591f78a6f3625203f8758b647386c927522f`。source2本の上記hash一致と、対象commitからのsrc/tests/Cargo.toml/Cargo.lock差分なしを確認した。実行metadataとusage counterは未取得である。

[独立runner報告](backup-destination-runner.md)と今回のraw証拠を照合した。cargo JSONの対象2EXEは `fresh=false`、build-finishedはsuccess=true、現物EXE hashはrunner報告と一致する。listと個別okの実名集合はintegration38＋lib13の51件で、重複・欠落0である。run-summaryはadministrator=False、両process exit0・timeout=False・wait済み・fixture残存0であり、lib stderrには隔離子5件のexit0・exact1件成功・完了token・fixture削除がある。sandbox probeのfixture失敗は成功件数へ混入していない。

単一guardは取得から内容検査・解決済みdirectory・全payload・manifest書込結果確定まで連続して生存する。再取得・途中drop・marker後の書込はなく、H4/H5・T08/T08bとの対応を確認した。作者差分原本と現追加部分のbackup末尾482行／integration末尾193行は全文一致した。実I/O失敗・marker不在・source保持・新宛先への再試行・write-block負例6種のoracleは保持されている。

閉鎖範囲はI2d-B consumer接合と独立51件の受入である。I2-D2全体、I10全target/release、製品EXE・実機受入は後工程であり、敵対的OS競争や電源断耐久性を証明したものではない。reviewerは編集・試験反復・再委任を行っていない。
