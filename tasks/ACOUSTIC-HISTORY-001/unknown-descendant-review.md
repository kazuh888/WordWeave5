# UNKNOWN-DESCENDANT-OPEN01：限定設計Pre受入

2026-10-22。U-A読取監査、U-B最小観測案、U-C独立Preを完了した。原因修正・実測・UNKNOWNのClosedではない。

## 固定入力と独立性

- 再開計画：89919D2D77DFE24180FA70488C44AFD8B9E2D19A18FFDF406A82255C018D74DF（初版E8A2332F…からレビュー担当再選定だけを追補）。
- 証拠監査：8801527FC2B8642ED05EE90F1DE83B24690896B716D957095F1DEA7953EF12BC。
- 最小観測設計：9C45A6DE4E6D46A6F354F7916898C34CBA604F9638877E4D5F1D463B01A0E00A。
- designerはAstra/high。独立reviewerの初回Astra/highはcapacityで作業0、計画者がSol6.1/highへ明示再選定した。親モデル変更0。実provider metadata／子usageは未取得。

## U-C判定

独立reviewerは「観測追加案を次の実装計画へ渡す」範囲で限定Pre PASS、受入阻害findingなしと判定した。指定3文書、関連G4/G5・FF契約、fake/Invoke対象処理、ENV01・FF consumerを読取確認した。unknown保持、actor分離、元Err/status/abort/期限保持、retry0、旧FAIL保全は整合する。7環境変数追加を既存ENV01 consumerが拒否する経路は確認されなかった。

以下は実装前の未解決gateであり、Pre成功によって解除しない。

1. 設計39/72行：診断公開時の上書き禁止保証、又は直接CreateNew方式、局所seamを具体化する。
2. 設計29/60/62行：既存FF testの厳密key集合（Test-FirstFailure.ps1:173–181）に対し、acquisition欠測時の集合維持と追加時の独立oracleを固定する。既存期待を黙って変更しない。
3. 設計66–68行：最終入力hash、完全command、変更／試験許可が揃うまでimplementer/runnerをdispatchしない。

公開方式／seamの実装成立とruntime挙動は未検証である。reviewerの書込・対象script実行・OS照会・試験・再委任は0。親は相対link3文書の欠落0と`git diff --check`診断0を確認した。docs-onlyのため製品Cargo test/buildを実行せず、delivery時の必須条件は維持する。

## 次の承認境界

利用者へ求める次の範囲は、検証用fake/監督scriptの診断追加と、通常の隔離環境内での独立した純合成6対照・局所静的検査である。実装前gateを具体化してから進め、実データ・API送信・admin・OS設定変更・既成功試験の一括反復は含めない。実OS合成ENV01一件一回は最終source/commandを凍結した後の別承認とし、今回の承認へ混在させない。F3/focused10/80/full/build/試用EXEはまだ未実施である。

## 承認受領とU-D差分Pre（2026-10-22）

利用者が上記の診断追加と隔離内純合成6対照を明示承認した。計画追補7D8AFBEB3AFCAD08563354E8690167BAB20EB149BDA2110BA7F1B1D3DB77D746、設計追補B92FE83EB555829AF2EC111E95C9E0109747CB0FEB31CD5B83977760836C27D7へ接合した。designer指定Sol6.1/high、独立reviewer指定Sol6.1/highで、追補37行と計画末尾だけを差分Pre確認した。

U-D限定Pre PASS、作者着手を阻害するfindingなしである。直接CreateNew／writer Result非伝播（設計82–84）、candidate値の初期化／FF完全wire位置／追加9key型制約／欠測旧strict keys（86–90/101）、抽出逆bytes／8stage・13fault／oldRED・baseline PASS・NotApplicable・AdapterFailure区別（92–107）が具体化された。作者と実装者は別担当とし、実装者はU-E期待固定・静的Pre・親受領後に着手する。未検証はadapterと実装の成立であり、完成hash/commandのfreeze、別runner/Postを省略しない。新OS試験・UNKNOWN閉鎖・旧F3治癒の承認ではない。

## U-E前半の限定静的Pre（2026-10-22）

packet ef551043-d22a-44a7-903b-1e9540efc910、expectations A1914AD22D864A30FEFEA90801F349F152EB23177E92E75774863F880047FAD4、manifest F5793B16E37A239FB07F6655F16956499A1C19ED3DE7DACB3D04F2D40B1E36FDに対し、本体実装着手の限定Pre PASSを受領した。6case/51inputsは設計と整合し、before2原本・14span・8成果物のpatch復元bytesが一致した。初回staticの9失敗はPS spanのBOM3bytesのoffset metadata欠陥であり、原文・期待・manifest・旧unit/bindingのSHA不変を独立確認した。修正前結果は保全し、BehaviorREDへ数えない。

実行Preは未達である。manifest旧O1 commandはnative単体で、外側PowerShellの終了値だけでは実exit1と2を区別できない可能性がある。native直後のLASTEXITCODE保存・同値明示exit・stdout/stderr別保存を行う固定receiptを追加し、静的Pre/freeze後に実行する。動的文字列置換で補正しない。新candidate/O2–O6実行unitはPending、compile/run0のままである。本体実装前に旧O1のBehaviorREDを実測するTDD順を維持する。

## 旧O1受入・U-F source静的Pre・O5 seam補正

旧O1はreceipt差分Pre後、親freeze A7924C12…で別runnerがcompile一回exit0→run一回exit1/exactRED3を実測し、16入力保全/保存accepted=trueを受入れた。詳細と完全SHAは成果物索引にある。候補実装のRustcompile/新51入力の合格とは区別する。

候補fake07A47648…／Invoke3B784E9B…とharness.patch083DD2E6…に対する独立限定静的Preは阻害findingなしである。診断Result非伝播、元Err/throw/status/abort保持、厳密型/key射影、candidate初期化を確認し、全9hunkのcontextとbefore→候補全文復元が一致した。

O5-diagnosticfail一件のtest-only構築失敗seamを設計へ明示したが、初版EC62E726…のUTF-8 byte範囲にBOM3bytes漏れを独立Preが検出し保留した。親はEC62原本をP/before/design-EC62E726.mdへ保全し、範囲だけ[92594,92929)とBOM注記へ修正した。現設計45A6F473368B7A06980DE1FB11E7870A2629EA2F55F1C950B7CA095C10D907D5は335bytes/元文SHAと独立一致して差分Pre PASS、期待A1914…／sourceは不変である。元catchを通す一箇所だけの固定置換・逆bytesを作者へ許可し、OS故障やOOMを発生させない。

通常のgit diff --cached --checkはharness.patch8/79/83/154行の末尾空白4件でexit1となった。独立reviewerは全4行が単一空白の正当なunified diff「空行context」であり、他の末尾空白0と確認した。これを診断0へ改稿せず、raw差分データの文法として分類した。trimすると復元性を壊すため削除せず、検査設定・除外・抑制も変更しない。この判断は083DD2E6…の4行だけに限定し、将来の別診断を無条件受入しない。
