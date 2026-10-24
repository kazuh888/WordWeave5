# 現在のタスク要約

更新：2026-10-24。これは再開用の索引である。旧経過・完全SHA・コマンド・承認原文は下表の正本を必要時だけ参照する。

## 目標と現在の範囲

ACOUSTIC-HISTORY-001：原音・固定例文・音響解析・Qwen助言を保存し、同条件の履歴をグラフ・表・再生で比較する。WordWeave5と単独音読ツールは共通ライブラリを利用する。現在はET3の安全な試用EXE提供に向けた検証中であり、機能全体・試用提供とも未完了である。

現在の単一工程は、利用者承認済みPSSAエラー採取方式の限定見直しである。規則・失敗判定を維持した設計／独立レビュー／必要な検証手順修正までを対象とする。元33入力・実OS ENV01・compile・製品EXE/buildは今回未解除である。

## 確認済み・未解決

- Save closure内のGet-CapHash解決失敗を診断し、同義BCL SHA式への一行修正後、新C01は固定11期待PASSとなった。
- 最新caseobs一回はC01 PASS／C02 Unexpected／残18 NotRun、実PSSA両roleはNotRun、exit2／retry0である。C02はordinary0／unknown1、PipelineとrecordRefs実型はList<Object>、primaryErrorは取得済みnullだった。観測全体はPartialであり、state固有欠測2とtruncated=null契約不適合を保持する。
- 別の合成API比較では、実DiagnosticRecordがWrite-Output -NoEnumerateでList・別参照、WriteObject(false)で同型・同参照となった。これはCAP対象／analyzer呼出0の補助証拠であり、元20合格や元PSSA NRE解消ではない。
- 新typed-output一回はC01–C05 PASS、C06 primaryIdentity不一致、残14NotRun、実PSSA0/retry0で停止した。exit2/tool2.1436989秒/unitouter1.7469127秒。C06の実受信型はErrorRecord、ordinary0/engineOccurrences2/engineGroups1/unknown0、primary取得済み。前後core17/pins17/settings23/freeze一致。独立Postは全体FAIL／観測Partialを確認した。入力↔初保持↔primaryの参照比較不足であり、原因は未確定である。
- 元Assertions AFA82E80…のPSSA1.25内部NullReference、旧UNKNOWN-DESCENDANT／旧cancel原因は未解決である。過去の診断成功・局所PASSを別gateへ転用しない。
- 候補保存点da1f701までGitHubへ反映済み、ブランチはcodex/development-agent-team。最新identity原証拠の復元patchB44BCDC9…は独立保存Pre PASS、後続解除不可を維持する。実際のHEADとdirty状態は必ずgitで確認する。

## 次のアクション

1. 原証拠保存は完了。設計7ED8371B…の14箇所だけを是正した新Test（CAP/typed-output-7c98c831-f8e8-42d3-91b9-60b54c1d11a2、688FED51…/97363bytes）は独立code Pre PASS、逆適用AB7E全bytes/BOM一致。3配列式の空/1/65入力の別9回帰もPASS、CAP対象実行0である。
2. canonical新B727443AE…／R259BAEA7…（cmd1BDBAFED…）は独立binding Pre PASS。復元patch02C62D34…は新9節と旧alias3件の全bytes/SHA一致、独立復元Pre PASS。親freeze60043B77…を作成済みである。
3. identity新slotも一回exit2/PASS5/C06FAIL/14NotRun/analyzer0/retry0で停止・消費済み。参照4値はmemory=context true/context=初保持 false/初保持=statePrimary true/初保持=resultPrimary true、追加欠測0。独立Post・原証拠復元Pre確認済み、全観測は既欠測2でPartial。固定oracleはfixture入力同一性を要求し未達、受信後保持へ読み替える期待修正は未承認で後続解除不可。比較基準見直しの設計・レビューについて利用者判断を求める。新実行・期待変更は行わない。

## 主要な決定・制約

- C39B5採取本体、oracle0E20118Bの20期待、全規則／severities／閾値、旧RED、C06 WriteError、C07順序・ThrowTerminatingErrorを維持する。未知型の自動展開・期待後付けは禁止である。
- 実行済みcaseobs基準はTest AB7EAFC5…／B797539B2…／RF0DCA8B9…。新688FED/B7274/R259Bのruntimeは未受入であり、古いfreeze／消費済みslotで再実行しない。既原本・FAIL・Partial・無効fieldは保存する。
- 通常sandbox／通常ユーザー／合成データのみ。新しい昇格・隔離外実行・OS設定・依存更新・実データ・API送信は行わない。過去の個別試験承認を流用しない。hard cancellationはUnavailableであり、yield／事後時間測定を強制停止保証にしない。
- 資源契約は子メモリ512MiB・同時1件・60秒。音響値を発音の優劣・総合点へ直結させない。原音・教材・学習記録を保全し、完全削除は今回含めない。
- 親モデルは利用者選択のまま。既担当はdesigner／test-author／reviewer=gpt-6.1-sol/high、runner=gpt-6.1-sol/medium。子の指定と実provider返却値を混同しない。
- 通常開発中の使用量を区切りで記録する。アカウント共通週枠と当工程消費を同一視せず、親子包含未確認なら合算しない。比較だけの再試行はしない。
- 適時checkpoint commit/pushは既存開発ブランチへの保存として承認済み。main merge／releaseとは別である。製品EXEは全体test/release・独立gate後の別配置milestoneで提供し、現在は未更新。DPI150/200%だけは試用時確認への移管承認済みである。

## 正本索引

| 内容 | 所在 |
| --- | --- |
| 全体依存・残工程・試用milestone | [実装計画](ACOUSTIC-HISTORY-001/implementation.md)、[早期試用計画](ACOUSTIC-HISTORY-001/early-trial-plan.md)、[試用契約](ACOUSTIC-HISTORY-001/early-trial-contract.md) |
| UNKNOWN／PSSA限定工程の順序と承認境界 | [再開計画](ACOUSTIC-HISTORY-001/unknown-descendant-resume-plan.md)、[設計](ACOUSTIC-HISTORY-001/unknown-descendant-design.md) |
| 共通case観測／今回14箇所是正 | [観測設計](ACOUSTIC-HISTORY-001/pssa-cap-case-observation-design.md)、[型出力設計](ACOUSTIC-HISTORY-001/pssa-cap-typed-output-design.md) |
| C06の入力・初保持・primaryの観測境界 | [参照境界設計案](ACOUSTIC-HISTORY-001/pssa-cap-identity-boundary-design.md) |
| Pre/Post・失敗・確定範囲・旧経過 | [レビュー](ACOUSTIC-HISTORY-001/unknown-descendant-review.md) |
| 完全path/SHA・receipt・raw・復元patch | [成果物索引](ACOUSTIC-HISTORY-001/unknown-descendant-artifacts.md)、unknown-descendant-tests.patch、unknown-descendant-harness.patch |
| 全体gate／EXE提供の残条件 | [試用runner](ACOUSTIC-HISTORY-001/early-trial-delivery-runner.md)、[試用review](ACOUSTIC-HISTORY-001/early-trial-delivery-review.md) |
| 使用量・欠測・手戻り・改善候補 | [ET3観測](ACOUSTIC-HISTORY-001/usage-observation-20261014-et3.md)、[KPI](../docs/process/improvement/kpi.md) |
| 七役割・担当割当の運用 | [開発体制](../docs/process/development-team.md) |

旧要約の転載はここから除いた。根拠資料・原証拠・Git履歴は削除していない。再開はこの要約と実git状態から始め、次の未完了工程だけを進める。
