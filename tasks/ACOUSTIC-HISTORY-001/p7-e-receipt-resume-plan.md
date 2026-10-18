# P7-E receiptによる限定再開計画

2026-10-18 / ACOUSTIC-HISTORY-001 / HEAD `27cab19`。暫定案から既承認外部仕様・設計の再利用を確認し、親承認・独立Pre待ちへ精緻化した。**本書は実行解除ではない。** 完了境界は未完了6入口の新46行動期待＋ValidateOnly2の独立Postまでであり、製品提供ではない。

## 根拠・要求・除外

正本は [main plan](early-trial-gate-plan.md) SHA `FF96F8DE80D3E7602F85F7846AF3C8C4C45AB1FF7A4B3F0806AE6A3DCEF817D2` と [design](early-trial-gate-design.md) SHA `6F84598858766BA08CFC236B8B847598705BDDD3A3000F4F1AB9BA07938236A7` のP7-S01–05/P7-Eである。R=`F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、P=`R/verification/argv-path01`、O=`P/41b01c28-5fe3-43a3-b510-14bafdd179c2` とする。

Oの`P7-E-OUTSIDE-FROZEN.md`（D00D8049…）、`P7-E-OUTSIDE-DRAFT.md`（1F9C6F2A…）、`E-OUTSIDE-INPUT-INDEX.json`（02964CB1…）を変更しない。`P7-E-OUTSIDE-RUNNER-REPORT.md` SHA `D9CC5FBD84C35878174D29D836B9B85789A97D31C17F9E2CC625B339B8D34BEE`は、root置換後にcommand全文置換が不成立となり、MissingOs予定slotでContextを反復した操作ミスを記録している。旧全体Post FAIL・停止適合、誤O02の追加Context12はcredit0、旧C9原因未確定を保持する。

親から受領した独立reviewは正O01だけの12件Reuse可である。対象は`P/9244f7ae-d0c6-4f8d-88d4-3a228343fa88/result.json` SHA `C11F70C6E20D1C2391499FF8EAEB3327358DF898E71AFCB4D60A0929634A20B8`。全42入力・固定runtime・境界・前後保全と12Passed/実exit0/期限内という同一実行の証拠に限定し、単一source SHAや旧J成功だけを再利用根拠にしない。

- R1：試験source、58期待、schema、期限、安全判定、firstUnexpected、retry0を維持し、完全argvの入口取り違えを実行前に排除する。
- R2：新候補UUIDの完成済みreceipt6本を固定する。各JSONの`cmd`は直接`&`固定pwsh `-File`完全argv、起動元非admin前後guard、前証拠CreateNew保存、直後の実exit保持を含むtool本文である。実行時は固定SHA確認済みJSONの`cmd`を無加工で`tools.exec_command`へ渡す。文字列置換、テンプレート合成、slot番号だけの選択、Invoke-Expression、追加PSwrapperの`-File`起動を禁止する。
- R3：旧Outside42＋正O01 result＋旧runner report＋本計画＝45、receipt6＋小index＋親freeze＝53を各入口の前後・最終に外側で照合・非上書き保存する。小indexは本計画を含む45の承認path/SHAと6receiptの対応を持ち、親freezeが本計画を含む自己以外の52入力を固定し、その自己SHAは親dispatchで外側固定する。本計画内に自己SHAを記載せず、receipt/index内部へ自己又は後続freezeのhashを循環埋込みしない。53入力guardは固定された外側手順であり、receipt本文を加工しない。
- R4：実行は隔離外`require_escalated`・通常ユーザーのみ。前guard未知/true/null/保存失敗は起動0、後guard又は保存不適合は次入口0。child tokenや実runtimeの欠測を起動元/別runの測定で補完しない。

対象外はContext再実行、新診断、旧E/旧C9の再分類、旧F3/UNKNOWN/ENV01/focused10/80/full/build/製品提供、admin/OS・ACL・PATH設定、実data/API、製品/test source・期待変更、新module/Pester/framework、公開である。旧誤rootも旧未使用予約rootも再使用せず、新6rootを固定する。最初4個はP直下、最後2個はR/verification直下、各直前に不存在・祖先reparseなし・ignoreを確認し、最後2個は終了後も不存在を要する。

`execute-review-remediation`のclosure matrixを本限定Packetへ適用する。

| Finding / 原文・位置 | Root cause / State / Packet | Owner / Canonical / Evidence・consumer | Oracle / Reviewer record / Residual |
| --- | --- | --- | --- |
| R-ARGV-01 / OのOutsideRunnerReport「誤O02と原因証拠」：root先置換によりcommand全文置換が不成立となり、MissingOs予定slotでContextを反復 | 実行時文字列置換・slot依存 / Planned / 本P7-E receipt再開 | test author / 完成receipt6本・小index / HEAD27cab19、同reportの構成式とactual argv、consumerはfresh runnerのtool呼出 | 各入口・Phase・root・schema・full argvの完全照合で旧誤O02を不一致として識別し、新完成receiptを照合 / Pre・Post未実施 / 旧F3・製品本体は対象外 |

## 順序・所有・受入・検証

本書を計画とtask-listの唯一の追加先とする。planner所有は本書のみ、既存`tasks/plan.md`・`tasks/todo.md`と親dirty `tasks/current.md`を保全する。候補UUIDと6rootの完全値は作者が提案し、独立Pre後に親が確定する。

| 単位 / 依存 / 規模・リスク | 担当所有と受入条件 | 必須検証・checkpoint |
| --- | --- | --- |
| U1 / 親計画受領 / 3file・中/高 | 独立作者：新候補のMissingOs/MissingPure/FFのreceipt3本だけ。各入口・Phase・OutputRoot・完全argv・元schema/期限・guard/保存/exitを明記 | JSON解析、`cmd`のParser error0・PSSA1.25.0通常rule診断0/実行error0・完了marker/exit0。独立reviewは旧承認argvのOutputRoot以外同一と本文の直接起動を静的確認。runtime0 |
| U2 / U1 / 4file・中/高 | 同作者：EO/Slash ValidateOnly/ENVprobe ValidateOnlyのreceipt3本＋小indexのみ。U1既存fileを変更する場合は再レビュー対象 | 同じ静的確認。全6で入口file/Phase/OutputRoot/expected schema/完全argvの相互対応、root重複0、53保全の非循環、正O01のReuse根拠を独立Pre。親が完全hash/argv/6root/外側guardをfreezeするまで実行0 |
| U3a–f / U2 Pre PASS＋親新freeze・各直前段受入 / 各1入口・中/高 | fresh runner：新candidate内の非上書きlog/reportと各固定rootの生成物のみ。順序はMissingOs2→MissingPure20→FF18→EO6→Slash ValidateOnly1→ENVprobe ValidateOnly1、各一回 | 各回53前後・実cmd・stdout/stderr・開始終了・実exit・元schema/件数/期限/保存を確認して次へ進む。MissingOs全15秒finally込み/Pure・FF各5秒全60秒、EOは既定の有限同期6件で新hard deadline保証なし。ValidateOnlyはdispatch/runtime0・root不存在。初不適合で残NotRun/retry0 |
| U4 / U3完了又は停止 / 報告1file＋証拠・中/高 | read-only reviewerが独立Post、親が報告/current/usageを保存。作者/runnerは自分の証拠を自己承認しない | 6本planned/actualの一致、53保全、各期待・期限・guard・保存・新rootを確認。合格時もO01 Reused12＋今回46、ValidateOnly2、C16 Reused16を別記し、全58freshとはしない。失敗時は失敗維持と停止適合を分離する |

- [ ] U1：receipt3本の静的適合、所有返却。
- [ ] U2：残3本/index、独立Pre PASS、親新freeze/dispatch承認。
- [ ] U3a–f：6本を順次一回、各checkpoint合格後だけ次へ。
- [ ] U4：独立Postと未実施・再利用範囲の報告。

各receiptのexpected schema/件数は旧草案「期待結果・境界」をそのまま引き継ぐ。MissingOs phase=MissingOs/accepted=true/results2PASS、Pure phase=MissingPure/results20PASS、FF Green/18Passed、EO SixGreen/green6、Slash validated/caseCount10/variantGuard/runtime=false、ENVprobe validated/expectedCount1/dispatch0/各credit0を照合し、存在しないfieldを0に補完しない。全実exit0が必要であり、exit0単独やslot番号は入口受入証拠ではない。変更していないtest sourceの再lint・製品cargo再buildは本復旧に追加しない。receipt内PowerShellの静的検査は実行互換性の代替ではない。

## 7役とモデル割当

呼出可能一覧は親提示の`gpt-6-astra`と`gpt-6.1-sol`（ともにlow/medium/high/xhigh/max/ultra）を根拠とする。以下の5役割当は**U1、U2、U3a–f各単位、U4のすべて**へ適用し、非起動役も責任を識別する。親モデル/品質gateは変更0、未対応・不足skillはblocker、代替/継承への黙示fallbackは禁止する。

| 役 | exact model / effort | 責任・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 固定役。本書の依存・証拠再利用・停止境界を扱う |
| external-spec author | `gpt-6-astra` / `high` | 固定役・今回は非起動。既承認P7-S01–05と元58＋ValidateOnly2を再利用し、外部契約変更0 |
| designer | `gpt-6-astra` / `high` | 今回は非起動。既設計・外側guard境界再利用。receiptが追加wrapperやschema変更を要求した時のみ差戻し、状態/証拠の循環を扱う能力を優先 |
| implementer | `gpt-6.1-sol` / `medium` | 今回は非起動・所有sourceなし。本体変更0という限定責任。新本体変更が必要なら本割当で着手せず再計画 |
| test author | `gpt-6-astra` / `high` | `p7_d1_author`候補。U1/U2の試験手順receipt/indexのみ、誤argvとhash循環の再発リスクを判断する |
| test runner | `gpt-6.1-sol` / `medium` | 誤実行担当から切り離したfresh `ww-test-runner`。U3の固定本文一回実行・証拠採取は明確な逐次手順である |
| reviewer | `gpt-6-astra` / `high` | 既存独立`p7_binding_review`候補。U1/U2 Pre/U4 Postで実行入口と再利用証拠を独立評価する |

起動は親だけがfresh context・明示model/effortで順次行い、通常補助1役・再帰0である。作者は既存test-author skillとPowerShell生成に必要な`powershell-automation`、reviewerは担当範囲のレビューskillと`powershell-review`、runnerは`wordweave-change`の検証責任を起動時に全文確認する。plannerは`agent-skills:planning-and-task-breakdown`全文と参照Definition of Doneを確認済み。起動指定と実返却metadataを区別し、未取得を推定しない。

## 未決・リスク・次handoff

未決は候補UUID/新6root、receipt完全本文・最終SHA、外側53保全手順の完全本文、独立Pre結果、親freezeである。これらは承認範囲内の具体化であり、追加ユーザー権限を要する未決ではない。新source/schema/期限変更が必要になった場合は本計画を越えるので止めて親へ返す。argv取り違えは全6の入口別静的照合と無加工実行で防ぐ。hash一致は未知の環境差や同期I/O停止を排除せず、旧C9原因やchild tokenを解決したとはしない。

作成時の機械生成は許容するが、完成した各`cmd`全文を独立照合し、実行時の文字列置換・合成は行わない。U1/U2の静的確認は同じ完成snapshotの独立Preへ接合し、機械生成方法だけを理由に追加frameworkや別runtime試験を要求しない。

次は親が本書を受領し、独立作者へU1→U2をdispatchする。作者はruntime0で返却し、独立Pre→親freeze→fresh runner→独立Postへ進む。本plannerはコード/テスト/実行/再委任/公開0で返却する。

KPI：本子の開始共有週枠49%/10080分/reset1792882581→最終応答前49%/10080分/reset1792882580、ordinaryUsageAllowed=true。reset値差を保持する。子token/session・親子包含・実返却model/effortは未取得、専有消費や削減を推計しない。親が既存usageへ接合し、本書外の測定文書をplannerは編集しない。
