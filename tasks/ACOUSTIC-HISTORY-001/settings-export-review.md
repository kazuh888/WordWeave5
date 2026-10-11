# I2d-S 設定export独立レビュー

2026-10-12。既定reviewer `export_destination_final_review`、指定gpt-6-astra/high。H最終Post PASS後、HEAD `56b3462` からのsettings_ui/control_settings限定diffをread-only確認した。編集・試験実行・再委任なし。実行metadata/usage counterは未取得である。

## 初回コードレビュー

判定は **CHANGES_REQUIRED（試験oracle1件）、本体の確定欠陥0**。対象source SHA256はsettings_ui `5575F9E5B5C138380DB273354BF3E12C5D87307F2A4237F254508D8B609871C3`、control_settings `F7A3FB0777B6E5791F7F4591953ABA0C0F2B16803C224460E81886D09DEF35C5`。

| Finding | 根拠・限定是正 | 状態 |
| --- | --- | --- |
| S-ORACLE-1 / P2 | S04はwire JSON再解析Valueとmemory直接Value化を比較し、font_scaleの0.8対0.800000011920929で失敗する。独立reviewerがrawの唯一の差分と確認。呼出前Progressから期待JSON bytesを作り、出力一致を確認する。root B/draft/editor非混入/後続ledgerと状態保持assertは維持する | Closed、下記限定Post PASS |

4UI出口→共通writer、Storage.dir優先/Noneのみresolver、credit_time→picker→memory JSON、通知、JSONL Start/CompletedまたはFailed(Io)を確認し、直接atomic_writeとstub残存なし。S04以外の25件oracleは仕様と整合し、環境変数と診断sinkは有限timeout子processに隔離される。自己検証は新24成功/S04失敗、旧9成功であり独立runnerの証拠ではない。

作者版source snapshotを保存していないため、stub除去以外のtest bytes不変は再計算できない。開始whole-file hash一致記録と、現行oracleの仕様への独立照合を区別する。欠測を後から埋めたとは主張しない。

S04是正とGREENの確認後にrunner-readyを再判定する。独立runner/最終Post、native picker、d-B/M、I10/製品EXEは未完了である。

## S-ORACLE-1限定Post・コード引渡し

同独立reviewerは **PASS/Closed、追加阻止0、runner-ready** と判定した。是正前snapshot2本のhashは初回対象と一致し、S04外全文とcontrol_settings不変を独立比較した。期待値は出力から逆算せず呼出前Progressのpretty bytesである。全bytes比較を加え、root/draft/editor/disk/memory検査を維持した。

新raw `logs/settings-export-s04-green.stdout.log` のexact1件・1PASS/0FAIL/0ignored、created/removed各1、現在TEMP残存0を確認した。source SHAはsettings_ui `C7C7C9D17DD24FC3FDF5762F7CD0E649A1A07E3E7237D31FE6C5A1FB63D1BF9B`、control `F7A3FB0777B6E5791F7F4591953ABA0C0F2B16803C224460E81886D09DEF35C5`、EXE `287E9A9743D5B21E0B0829E715016885099D690941CEC7ECFCA5C618720B0737` の実物に一致した。build成功・同EXEの出力を確認、PID31472/native exit0/timeout0/前後hashは作者記録による。

I2d-S全体は未Closedである。自己検証34uniqueは異時点接合であり、次は独立runnerの新25＋旧9件。原作者before snapshot欠測は是正前snapshotとは別に維持し、native picker/I10未検証も残す。
