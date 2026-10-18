# P7-C9 junction一回診断・限定計画

2026-10-18 / ACOUSTIC-HISTORY-001 / 仮計画。親受領→既存仕様・設計との独立Pre照合→必要なら本書精緻化→親freezeを経て実行する。planner所有は本書（task-listを兼ねる）のみ。HEAD `d81dc52`、開始時tree clean。未完了の既存plan/todo、旧証拠を保全する。

**目的・根拠。** 承認は「隔離環境外・通常ユーザー権限で、検証用リンクを1回だけ作成する診断。管理者権限・OS設定変更・実データは使用しない」である。既存[独立Post](early-trial-delivery-review.md)冒頭のEはContext9Passed/C9Unexpected/C10–11NotRun/実exit2、後続0/retry0、34入力保全、FAIL・停止適合のままである。C9は`Test-ContextPreflight.ps1`340–342のjunction作成でconsumer未到達。旧accepted-bと隔離条件の差は確定、時点・版・ACLも異なるため隔離が単独原因との推論は未確定である。

**再利用・除外。** 上記承認の観測契約と既存CreateNew保存・入力freeze・独立Pre/Post設計を再利用する。新外部仕様書・内部設計書・本体実装は起動しない。診断は製品動作を変更せず、曖昧点はPreで本書へ返す。新framework/Pester/module/config/log基盤、E再実行、consumer実行、期待変更、管理者化、OS/ACL変更、実データ/API、削除/cleanup、commit/publishは対象外。旧F3/UNKNOWN/ENV01/focused10/80/full/build/製品提供の解除は0である。

## 要件・受入境界

- J-R1：専用の新UUID実行rootに空targetと固定名junctionを一つだけ作る。root既存なら中止し、正規化した固定親と全祖先のreparseを拒否する。対象pathは親freezeで完全展開し、任意path入力・代替作成法・再試行を設けない。junctionの作成attemptは最大1、許可条件成立時の実測1を報告する。空targetとリンクはそのまま保存する。
- J-R2：固定PS7.6.7/実ロードruntime hashを前後照合し、同processの公開`WindowsIdentity`/`WindowsPrincipal`情報（User SID/AuthenticationType/ImpersonationLevel/groups/IsAdmin）で通常ユーザー・admin=falseを実行内検査する。判定不能/admin=trueは作成前停止。PInvoke/外部whoami/native/CIM/子process起動は禁止。runnerの固定PS起動一回とscript内子起動0を区別する。
- J-R3：作成直前/直後の同process token、親/root/targetのACL・属性、リンクの不存在/事後状態、時刻・attempt数を採取する。ACL/属性取得不能は欠測と元ErrorRecordを保存し、成功値で補完しない。祖先reparse判定不能は安全条件未成立として停止する。作成例外はcatch時の完全ErrorRecordを保持・保存し、例外chain/HResult/FullyQualifiedErrorId/CategoryInfo/TargetObject/InvocationInfo/ScriptStackTraceを落とさない。rawのシリアライズと可読記録を用い、保存失敗は別欄に残して主結果を覆わない。
- J-R4：全出力はCreateNew、上書き/消去0。事前情報を作成前に保存できない場合はattempt0で停止する。旧34入力＋親指定の今回入力を前後照合し、保存artifact・実exit・consoleを別runnerが返す。入力drift/保存失敗/欠測を隠した成功を禁止する。
- J-R5：`creation-success`（実junction/固定targetを事後確認）、`diagnostics-complete`（要求証拠・保存・入力保全が揃う）、`productPASS`（本診断では常に未評価）を別項目で報告する。作成失敗でも完全な失敗証拠は診断完了になり得る。欠測時は診断不完全、単なるNew-Item無例外を成功扱いしない。診断exit0は証拠充足、exit2は安全停止/欠測/保存・整合不備とし、作成成否は独立値である。

## 作業順・所有・必須検証

`P = target/early-trial-build/et3/cargo-gate/verification/argv-path01`。親予約の候補rootは`P/289725bd-b80a-4ba5-9d3b-673b7eba04ba`、実行rootは`P/7ef4d4a9-a4bd-45bd-9194-46970e71afe0`（ともに未作成）。外側freeze/log/reportは候補root内に非上書き保存し、第三rootは作らない。候補rootは作者が初回不存在を確認して作成、実行rootはrunnerが初回不存在を確認して作成する。所有は各新root内の下記成果物だけであり、同時編集しない。

| task-list | 依存・担当所有 | 受入・検証（各単位S） |
| --- | --- | --- |
| [ ] J1 固定診断候補 | 本書の親受領。test author：候補rootの単一`Test-JunctionDiagnostic.ps1`と短い静的報告のみ | ①J-R1–5を満たす固定script、200行未満目標（gateの省略理由にしない）。②固定PSでParserエラー0、PSSA1.25通常診断0/engine error0、除外規則追加0。③runtime0・旧入力保全、候補hash/行数/完全静的command/実exitを返す。 |
| [ ] J2 独立Pre・親freeze | J1。reviewer read-only、親がPre報告/freezeを新規保存。plannerは指摘時に本書だけ精緻化 | ①reviewerが契約再利用、path/祖先・admin guard、attempt1/retry0、ErrorRecord/欠測/保存・分類を静的確認。②Pre PASSと親受領、実行root不存在・固定exe/runtime/input hash/完全command/require_escalated一回をfreeze。③各ownerと最終hashを確認し、未確定値・循環hash・所有外差分なし。 |
| [ ] J3 一回診断・独立Post | J2。別runner：実行rootの合成データ/結果/consoleと新規runner報告のみ。reviewer read-only、親がPost保存 | ①`require_escalated`で固定PSを一回起動、Windows管理者化なし、実行内admin=false確認後junction最大1試行、再実行0。②前後証拠/入力一致/完全ErrorRecord/保存成否/実exitと未観測事項を返す。③独立PostでJ-R1–5・禁止操作0を確認し、旧EのFAILと未解除範囲を維持して終了。 |

J2が実行前checkpointである。static失敗は作者へ戻す。実行開始後の失敗・不明・通信断は自動再実行せず、取得済み証拠を保全して親へ返す。cleanup/rollbackは実施しない。親はfreezeで本書・候補のhashを外側から記録し、本書への自己hash埋込みや上流への逆bindingをしない。

## 7役・モデル割当

以下をJ1–J3すべてに適用する。非起動役も責任を識別し、親が起動時にexact model/effortを明示する。model/skill利用不能はblocker、親modelと品質gateは不変である。

| 役 | exact model / effort | 難易度・リスクと選定理由／今回の担当 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 中/高。権限と因果推論の境界を計画し本書だけ所有する。 |
| external-spec author | `gpt-6-astra` / `high` | 中/高。既承認診断契約を再利用し非起動。観測仕様変更が必要なら親へ差戻す。 |
| designer | `gpt-6-astra` / `high` | 中/高。既存保存・freeze設計を再利用し非起動。追加設計が必要なら先に計画精緻化する。 |
| implementer | `gpt-6.1-sol` / `medium` | 低/低。本体変更なし・非起動。本体問題の推定で実装を開始しない。 |
| test author | `gpt-6-astra` / `high` | 中/高。既存`p7_d1_author`を再利用、有限scriptと失敗保存境界を担当する。 |
| test runner | `gpt-6.1-sol` / `medium` | 低/中。既存`locale_l3_runner`を再利用、凍結command一回と証拠採取のみ。 |
| reviewer | `gpt-6-astra` / `high` | 中/高。既存`p7_binding_review`を再利用、作者・runnerから独立して安全境界と欠測を判定する。 |

通常補助1役ずつ、子再委任0。各役は必要な既存skillを全文読む。作者は`powershell-automation`、reviewerは`powershell-review`と役の既定review skillを担当範囲で適用する。起動指定と実行metadataは別記、未取得は未取得のままとする。

**リスク・未確定事項・次handoff。** 隔離外成功から旧失敗の単独原因は証明できず、同時点のACL・token証拠でも過去を補完できない。公開WindowsIdentity/Principal情報は完全なeffective permission解析ではない。安全guardと観測欠測を区別し、追加権限/試行/実装変更が必要なら本診断を停止する。親が確定すべき値は固定runtime/input集合・最終hash・完全commandである。次は親が本仮計画を受領し、上記作者へJ1をdispatchする。J2で仕様/設計再利用を確認し、指摘がある場合は実行前にplannerへ返す。新たな外部仕様/設計が必要なら、その承認後に本書を精緻化しJ1へ戻す。

planner完了境界は本書の保存・diff/link/所有確認と親への返却までである。script作成/runtime/子起動/commit/publishは0。KPI開始週枠45%/10080分/reset1792882580、アカウント共有値で専有量不明。親子token開始/終了・実行metadata・包含関係は未取得で推計しない。終了値と本書確認結果は親へ返し、既存usageへの接合は親の所有とする。
