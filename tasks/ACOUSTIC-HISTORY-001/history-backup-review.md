# I2b-B 独立レビュー

## 設計Pre（2026-10-11）

指定reviewer `backup_consumer_pre`（`gpt-6-astra/high`）の判定は **PASS**。受入阻止・確認事項・任意提案0件である。対象はAH-AC16/I2-T06、[設計のb-B追補](design-i2.md)、[最大3fileの計画delta](implementation.md)。設計SHA-256は `A7A6931A63A53E6842B91DC816ED1430BAF2D3EEB3CC95F753C6226AE26EFBE0`。

- B1：currentの非履歴draftを別memory退避で保持し、退避/commitが各区間の最新rootを採用する。実保存返却値の採用は既存APIと整合する。
- B2：candidate/diskのhistoryは宛先作成前に拒否する。型付け済み値だけのwire1投影と、backup専用不存在許容は通常strict readerの契約を弱めない。
- B3：元bytesのhash確認後にstrict decode/非null root拒否。hashを再計算した不正fixtureでschema拒否を独立に検証する。
- guard再入なし、事前拒否と先行退避/媒体残存の区別、自動rollback/再送なしを確認した。外部write-block対応はI2d-H/Bに残す。
- export長の独立oracle改訂だけを許可し、memory runtime2 oracleと既存reader回帰を維持する。

読取中HEADは`6dd8285`から保存checkpoint `ebac9e0`へ進んだが、対象設計とbackup/store/commitのhashは不変である。適用Skillはreview-software-designと保存失敗境界のreview-reliability-data-integrity。reviewerはread-onlyでbuild/test/編集/送信/公開を行っていない。実装・新規試験・Windows競合の実証は未了であり、実行metadata/開始終了counterは未取得である。次は独立作者→本体→独立runner/Postである。

## コード確認・限定差戻し B-WB1（2026-10-11）

reviewerは本体の確定欠陥0、既存oracle全文/integration hash不変、自己42件成功を確認した。ただし次のP2検証欠落によりrunner前受入は保留である。

| Finding / 原文・位置 | 原因 / 状態 / Packet | Owner / Evidence / Oracle | 残余 |
| --- | --- | --- | --- |
| B-WB1: backup入口のwrite-block拒否が未検証。design-i2のb-B追補は内部退避拒否と子process隔離を要求するが、新42件・既存gate試験はbackup入口の当該境界を呼ばない。確信度高 | gate自身の試験とconsumer接合の試験を区別できていなかった。Accepted / Planned、test-only B-WB1 | 独立作者がstore::i2_backup_path_testsに1件追加。子processでblock→memory/exportを各呼出し→両Err、各時点ledger/current不変、destなし、block継続。有限timeoutで親から隔離 | 外部export拒否はb-B暫定契約。I2d-H/Bでは外部成功へ改訂。独立runner/Post/I10未了 |

### B-WB1 Pre package（唯一の追加契約）

対象branchは`codex/development-agent-team`、HEAD `ebac9e0`。本体hashはbackup `12A20A9E5C509574B078A77C47E6B836A112A81EAAD0B8419CDC8E028DC84261`、store `91DF6C7CC92ABF8DEB2800C08B6B3C47D02F5F6C089ECCE89484FB05CDBB5118`、integration `B907A3F6BCAF195000669CC2B7F954FC4323AC7832B0CDE5A8DE63B3B76CEE0F`。Cycle1、requested gate=限定test追加Pre、依存=上記本体コード確認済み、未決製品判断なし。親self-checkは既定3file所有/既存private guard/API/合成fixtureで収まることの照合であり、独立PASSではない。

目的は保存停止状態でconsumerが書かないことの直接証拠である。正本は既存design-i2、追加契約のownerは独立test作者、変更fileはstoreの新cfg内だけ。consumerはbackup::preserve_current_progressとbackup::export、入力はvalid None-historyのdisk/currentと別dest、出力はResultのErr。子processでguard.block_writes→drop後に両入口を呼び、各失敗後の全ledger/current不変・dest未作成・再取得時WriteBlocked継続を要求する。解除API・既存assert変更・本体/公開API/保存形式/配布変更は対象外。合成専用TEMP、有限timeout、親が子exitと自身の非汚染を確認する。raw記録は同taskログへ保存する。

これは製品挙動の修正ではなく不足test追加であり、修正前は直接証拠なし、修正後は上記入口の実行証拠があることで識別する。test削除で取り消せるが本体や実データのrollbackはしない。失敗時は既存所有fixtureだけを後始末しログを保持する。本体修正/新APIが必要、有限実行できない、既存oracleとの矛盾が出れば停止して親へ戻す。実行後は独立reviewerへ追加差分/ログだけ再提出し、Closedを自己宣言しない。

限定Pre記録：同reviewerが上記packageのSHA `6B3AD46AAB6114E359B58985DC02A26B379E7B1CFFFA93EE709873B526363F81` を確認し、Pre PASS・阻止0とした。親は独立作者へstore cfg内の1件だけを返却した。未検証をClosedにしない。

### B-WB1 Post / runner-ready

独立reviewerが追加case・rawログ・固定hashを確認し、**B-WB1 Closed、コードrunner-ready**とした。残るコード指摘/追加阻止0件。上表のStateはこの判定でClosedへ進む。追加箇所を除いたstore全文hashは以前の`91DF…5118`と一致し、他のoracle/本体は不変である。

現store `516F5DA0E8C6CFDAB76BACAC9578BD477BBE85852A908534C2E5FCCB0F2C2EA9`、backup/integrationは上記固定値、lib EXE `3CC77348194704FE29E1D9969B02683DF6CC70C858CFD9A2E0E57CCA37189CF0`。子exit0・timeout=false・固有marker・親子1PASS、親gate非汚染/fixture残0を確認した。uniqueは1件である。

これは限定指摘の閉鎖であり、I2b-B最終Postではない。自己75件＋追加1件の成功証拠は複数EXE世代/環境に跨り、旧EXE hashとB-WB1親native exit/PIDの欠測は残す。次の独立runnerは最終sourceで4EXE（lib/bin/audio_history_backup/audio_history_progress）を再buildし、`--list`確認後、新scope43＋旧33＝76uniqueを新規実行する。source/EXE前後hash、各process exit/timeout、実名・件数、専用TEMP残存を採取する。build中の同EXE実行を避け、session_id/exit_codeを保持し、PowerShell表整形で列を失わず構造化出力する。既知の媒体/junction制約は同一EXEの該当exactだけ限定sandbox外で確認し、UAC/OS設定変更は行わない。

担当は計画どおりrunner `gpt-6.1-sol/medium`、その後の最終Post `gpt-6-astra/high`。親は利用者の切替前停止指示を守る。I10全体test/release・本体UI・製品EXEは未了。reviewerはread-only、実行metadata/開始終了counter未取得である。
