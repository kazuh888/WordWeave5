# UNKNOWN-DESCENDANT-OPEN01：保存証拠監査

2026-10-22／U-A。限定scout（指定gpt-5.6-terra/medium）の読取報告を親が保存した。実provider model/effortは未取得。実試験・OSプロセス照会・停止・実データ・API・設定変更は0である。初回の誤モデル起動は中断し、調査credit0としてusageに別記した。

## 対象と証拠索引

略記R=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、S=`b5bb9069-90e5-429b-8c5b-97c7f7f22f75`、U=`77a4759e-0508-4cdd-966b-140c3ef89a12`である。表の相対pathはRからの位置であり、実行commandではない。

| 保存証拠 | SHA256 | 受入範囲 |
| --- | --- | --- |
| verification/S.F3-REPORT.md | A739994AE600B451687BBDDAA5BF29931440ED12C46A0B3BB9996ECD78AB3608 | 旧失敗の報告。FAILは維持 |
| verification/S.f3-run-hashes.json | A526579ECEAA582DE35FC461A1BE8FA34C2FEB3E9527D5BA7F48DBED7A0CCF76 | scoutが列挙72/72一致を確認。これは生成run資料の照合であり旧入力本文の存在証明ではない |
| verification/S/frozen-inputs.json | B04B16855B1D337681F311C46F55637F408D6B99AA0E165C2F2D747CA1A922C2 | 当時のInvoke指定318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3A |
| runs/U/first-failure-Supervisor-8048-639277209899781912.json | 1D1E07D9AC168E45FDAC818BFA55B9C93055FD39A452067DBA394ABB5E068B93 | gateSourceSha256、failure.osObservationのPID／known／snapshot／exceptionLine |
| verification/S.f3-evidence-summary.json | 8BFD8745541725F42DBADFA74F8C67EB974B408E95C1E485C1E09DDBB27E6F03 | actor別観測の直接索引。source本文は含まない |
| runs/U/invocations/00000000-3204-8000-98de-e7755fa38cbc/result.json | 708E51B3AB7C63C9F27F4EBF22BEB1EA8AED4F1E9809DB763285155CCCE575FC | Runnerの直接handle／終了観測 |

Runnerの直接根拠は`R/runs/U/invocations/00000000-3204-8000-98de-e7755fa38cbc/result.json`の`childExitInt32`と`processes[0].ownershipEvidence`である。親による索引接合時に最初の候補を誤ってR/invocationsへ置いたが不存在であり、保存hash一覧のrun配下pathへ訂正した。不存在候補を証拠として使用していない。

## actorを分離した時系列

以下は保存UTC時刻である。現在のPID状態は調べていない。

| actor | 保存観測 | 言えること／言えないこと |
| --- | --- | --- |
| Runner PID22796 | 04:16:55.722 target PID20688起動、55.810 DirectLaunchHandleでexit93／NotFoundAfterExit | Runnerが保持した直接handleの終了観測。Supervisorの所有・同世代判定を代替しない |
| Supervisor PID8048 | 04:16:56.013、knownMatchCount=0、snapshotMatchCount=1、GetProcessById例外line1569、CleanupIncomplete | Get-Identityより前で止まった。snapshotの対象が同じ世代だったことや正確な取得競合は未証明 |
| RunOwner PID26088 | 04:16:56.159 cancel | 前段失敗後の停止記録であり元target異常の原因ではない |

`target-20688.started/environment`は保存されているが、ENV01.zipの目録に`target-20688.completed.json`はない。`cargo-invocation-0.exit.json`は93、controller stdout/stderrとrun stderrは空である。元targetのI/O／fixture error本文、ET3_FAKE継承値、snapshot採取区間・同世代の根拠は欠測である。

## source対応と限界

当時hashと一致する`R/fake.rs`（3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016）の191–193が完了marker、291–297がerror時exit93のproducerである。ただし実継承設定が欠測であり、exit93単独からこのerror経路を確定しない。

現在のInvokeは898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71であり、当時指定318Eとは異なる。run zipとENV01.zipには当時のInvoke本文がなく、現行line1569/1571を当時の本文として流用しない。FF内の例外位置と現行source位置の解釈を分ける。

U-B designerの追加静的確認により、現898CのUTF-8 bytesをメモリ上で7.6.7→7.6.6へ4 occurrence（line20、1045内2、1083）だけ逆適用したSHAが旧318E6667C21B02D54DFBD8A539163365868D25F76866C9200DB8884EB1DB7A3Aに完全一致した。保存・実行は0である。これにより当時本文の1569 GetProcessById／1571 Get-Identity対応は同定できた。保存archiveに本文がない事実は変わらず、メモリ上の再構成を旧原本発見とは呼ばない。元targetエラー・継承値・snapshot世代の欠測は解消していない。

## 結論と次工程

U-Aの資料照合と欠測の特定を受領した。原因は未確定であり、UNKNOWNはOpenのままである。終了前後の競合、fixture内部異常、設定によるexit指定を区別できない。次は最小観測の設計と、target正常性oracleに外部仕様差分があるかの判定である。既合格P7-E／MissingOs／Context12の再実行は不要であり、旧F3への合格creditも0である。
