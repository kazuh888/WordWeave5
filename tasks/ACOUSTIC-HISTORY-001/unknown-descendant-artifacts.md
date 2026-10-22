# UNKNOWN-DESCENDANT-OPEN01 診断追加・局所試験の成果物索引

2026-10-22／開始HEADba7f0e4。利用者承認は検証用診断追加と通常隔離内の純合成O1–O6である。新実OS ENV01/F3/native/full/製品buildは対象外である。実装・新試験の成果物は完成後に追記し、予定pathを生成済みとは扱わない。

## 開始baseline

R=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、H=`F:/Program Files/PowerShell/7`。親によるread-only照合9点は全一致した。

| Path | SHA256 |
| --- | --- |
| R/fake.rs | 3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016 |
| R/Invoke-CargoGate.ps1 | 898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71 |
| R/Test-CargoGate.ps1 | F0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919 |
| H/pwsh.exe | FA2C8BB1E35CDD62AD15CD06A1D8F2C5961E9725A9BFA1E7CFD2C322F49E9619 |
| H/pwsh.dll | A583A3721FEEBC4F9552A02358BC81BBDD41E2E18BB3F424226F2B1E0BF07DEB |
| H/System.Management.Automation.dll | 70701F20AFBBF4E763924C2BDD0C0E9A20EEC35A0E4EFF439E0CCF77F20D6B26 |
| H/pwsh.runtimeconfig.json | 8A6363FBA1F89C0688E3594EF4FE7B9036126F04957C5FE56858CBA020B76B33 |
| H/pwsh.deps.json | A01883AEEB2217AEB1D7EC944D649376104E7A874CA4EA71E7F36448C1519DC2 |
| H/ja/System.Management.Automation.resources.dll | F672B902371AD615A7AD211EE387B73C65386E7686610B6DCCA9A91927D8DC94 |

原本before、oracle、patch、最終command、freeze、結果の対応は各工程の完成後にここへ接合する。既存成功ログを新sourceの結果として流用しない。

## U-Eの作成領域

P=`R/verification/argv-path01/ef551043-d22a-44a7-903b-1e9540efc910`。独立作者がP/beforeへ原本2fileをCopy-Itemで保全し、編集0と報告した。親はfake.rs（14443bytes）とInvoke-CargoGate.ps1（154848bytes）の存在を確認した。oracle/新adapterは作成中であり、新candidateの未存在本文位置はPendingのまま、期待値固定後に対応づける。これは期待緩和ではなく、存在しないcandidateのhashを実装前に求める循環gateを避ける順序である。完成物の静的Preと最終freezeを省略しない。
