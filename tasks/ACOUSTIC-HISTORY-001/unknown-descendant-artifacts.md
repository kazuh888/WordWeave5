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

## U-E独立期待と旧O1実測

expectations.json A1914AD22D864A30FEFEA90801F349F152EB23177E92E75774863F880047FAD4は6case/51inputs（3/2/9/14/5/18）。baseline14spanと旧O1だけ完成し、新candidate/O2–O6 unitはPendingである。manifest F5793B16E37A239FB07F6655F16956499A1C19ED3DE7DACB3D04F2D40B1E36FDはこの未完了を保持する。

旧O1 receiptはcompile29635D20F81D54177954FF2A6EEDA92E880138E7922CF294E3E7365FDE87ED2C、run D3B378EF4B71D74879A974D1804EF2ABDB83089A1653D8CCE8F8ED7A5D6C0A80。Parser/PSSA1.25通常診断0/engine0と独立Pre後、親freeze P/OLD-O1-FROZEN.md A7924C12208D5FB0532698FCF83FFFAF16D0F27F6F6BF2C1B68F56204B413323を外側照合し、別runnerが通常sandboxで各一回無加工実行した。

- compile root P/616fa13d-d949-4462-bfc6-0df66c902b6a。tool/native exit0、CompilePASS、execution SHA6D5847D91A5687B4825EE9F19DC13DDF74D6DBDB1AAF2140D39E3DC55511A2BF。
- run root P/d3a00d97-8a53-4c73-a0d9-cc8f9fa3df7f。tool/native exit1、exact BehaviorRED3/3、stderr空、0.0908689秒、execution SHA DE96A4BFB9F2236D852C17F6C4AE518EA11C238BEB6138361725E8B1C95BB70F。
- 両executionはaccepted=true/CreateNew保存、16固定入力前後一致。retry/kill/新case0。report P/OLD-O1-RUNNER-REPORT.md SHA021C234CE8A08DACC5CB38B6D8D2BFBF5D1FA06955AA03AE0027C7E38346870D。report文書だけは上位の編集制約に従うapply_patch新規保存で、実行証跡のCreateNewと区別する。

初回staticはBOM offset9失敗、receipt初回PSSAは未使用宣言2件で停止し、それぞれ元結果を保全した。原文/期待/許可置換は変更せずmetadataと未使用宣言2行だけを補正した。これらをBehaviorREDに算入しない。実行native2と1の識別不足も実行前に固定receiptで解消し、未承認の再試行は0。現在のtests復元patch SHA BF7B5483968C0F05D7F701DEC5D967C2C377F42E8754500B0B8F24BA6B62CB1Bであり、候補束縛後の新差分は別hashで接合する。

## U-F診断本体候補（局所実測前）

実装者指定Sol6.1/high、所有はR/fake.rsとR/Invoke-CargoGate.ps1の診断spanだけである。候補fake07A47648E47FB57F5D0969650BBCEAF138671A4FC151EF6AA1E33020DCF2DCC4（18319bytes）、Invoke3B784E9B7CAE98261D0A496C0E229C4E2CB39120962656BF2186049A08CCC4D3（159294bytes）。Parser0/PSSA1.25通常診断0/engine0、PS static host7.6.7、許可span外0、forward/reverse全bytes一致2file、元target body/13fault/main不変を実装者が確認した。Rustcompile/runtime成立を証明する結果ではない。

P/implementation-static-evidence.json FE4E890062E236176F76C7FD818D8188E9DED8244ED7C4CFC9FFB365A0448D2A、P/implementation-report.md 4225F45349294EF791FB9F8543DAD9979F59E2AD55FF3CBC6D9B338BD8F85B80、tasks/ACOUSTIC-HISTORY-001/unknown-descendant-harness.patch 083DD2E6016FDC0FA319FF9C07FE23FCB90BC91A5A0CD23E76EB9A3CC4F08D0F。初回static session結果の未取得と再採取1回、照合器のList/複数diff境界不備attempt1/2は保全され、本体/期待を変更せず照合器だけ修正した。独立作者が候補束縛中で、最終Pre/freeze/6対照/Postはまだ未完了である。

保存方針の未決：既存.gitignore25–26行はtasks配下のdiff/patchを対象外とする。親の通常git addはpatchで拒否されたためforce-addや設定変更をせず、2つの再現patchだけを個別例外にしてよいか利用者へ非blockingで確認した。返答まではpatchをlocal保持し、Git保存済みと称さない。検証自体は継続する。stagedの文書3件は保持し、無関係編集を戻していない。

利用者がこの2patchだけの例外を明示承認したため、親は.gitignoreへexact pathの否定rule2行だけを追加した。無関係なtasks配下patchは引き続き除外されることをgit check-ignoreで確認した。force-addは0である。source候補の独立局所受入前という状態を保持し、保存点は受入完了と呼ばない。

U-E後半でO5-diagnosticfail一件の純合成seamが不足すると作者が報告した。既存許可adapterだけではacquisition構築失敗→key省略を作れないため、期待を変更せずその一件をdesignerへ戻した。残り50入力の準備と設計文書は所有が異なり独立するため、親はこの間だけ作者＋designerの限定parallel（親を含め3slot）を許可した。未許可assignment置換・新OS障害・意図的メモリ枯渇は0、seam差分の独立Pre後に接合する。
