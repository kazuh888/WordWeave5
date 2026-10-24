# 現在のタスク要約

更新：2026-10-24。これは再開用の索引である。旧経過・完全SHA・コマンド・承認原文は下表の正本を必要時だけ参照する。

## 目標と現在の範囲

ACOUSTIC-HISTORY-001：原音・固定例文・音響解析・Qwen助言を保存し、同条件の履歴をグラフ・表・再生で比較する。WordWeave5と単独音読ツールは共通ライブラリを利用する。現在はET3の安全な試用EXE提供に向けた検証中であり、機能全体・試用提供とも未完了である。

現在の単一工程は、利用者承認済みPSSAエラー採取方式の限定見直しである。規則・失敗判定を維持した設計／独立レビュー／必要な検証手順修正までを対象とする。元33入力・実OS ENV01・compile・製品EXE/buildは今回未解除である。

## 確認済み・未解決

- Save closure内のGet-CapHash解決失敗を診断し、同義BCL SHA式への一行修正後、新C01は固定11期待PASSとなった。
- 最新caseobs一回はC01 PASS／C02 Unexpected／残18 NotRun、実PSSA両roleはNotRun、exit2／retry0である。C02はordinary0／unknown1、PipelineとrecordRefs実型はList<Object>、primaryErrorは取得済みnullだった。観測全体はPartialであり、state固有欠測2とtruncated=null契約不適合を保持する。
- 別の合成API比較では、実DiagnosticRecordがWrite-Output -NoEnumerateでList・別参照、WriteObject(false)で同型・同参照となった。これはCAP対象／analyzer呼出0の補助証拠であり、元20合格や元PSSA NRE解消ではない。
- 元Assertions AFA82E80…のPSSA1.25内部NullReference、旧UNKNOWN-DESCENDANT／旧cancel原因は未解決である。過去の診断成功・局所PASSを別gateへ転用しない。
- 最新pushはc98147d、ブランチはcodex/development-agent-team。原caseobs証拠の復元追補と次の限定候補は作業中である。実際のdirty状態は必ずgit status --shortで確認する。

## 次のアクション

1. caseobs原証拠と合成API比較の復元保存・独立確認を完了する。
2. 独立Pre済み設計7ED8371B…に従い、新Testコピーのproducer正出力11箇所とroot配列保持3箇所だけを是正する。逆適用AB7E全bytes/BOM一致→独立code Pre。作者へ実装依頼済み、対象実行はまだ0である。
3. 新B/R・3出力UUID・caller/argvを限定再束縛→独立binding／復元Pre→親freeze→別runnerの新slot一回。Unit20成功時だけ同hostでCollector-self、固定AssertionsのPSSAへ進む。最初の異常で後続NotRun、retry0を維持する。

## 主要な決定・制約

- C39B5採取本体、oracle0E20118Bの20期待、全規則／severities／閾値、旧RED、C06 WriteError、C07順序・ThrowTerminatingErrorを維持する。未知型の自動展開・期待後付けは禁止である。
- caseobs基準TestはAB7EAFC5…、canonical B797539B2…／RF0DCA8B9…。現在の次候補は未受入であり、古いfreeze／消費済みslotで再実行しない。既原本・FAIL・Partial・無効fieldは保存する。
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
| Pre/Post・失敗・確定範囲・旧経過 | [レビュー](ACOUSTIC-HISTORY-001/unknown-descendant-review.md) |
| 完全path/SHA・receipt・raw・復元patch | [成果物索引](ACOUSTIC-HISTORY-001/unknown-descendant-artifacts.md)、unknown-descendant-tests.patch、unknown-descendant-harness.patch |
| 全体gate／EXE提供の残条件 | [試用runner](ACOUSTIC-HISTORY-001/early-trial-delivery-runner.md)、[試用review](ACOUSTIC-HISTORY-001/early-trial-delivery-review.md) |
| 使用量・欠測・手戻り・改善候補 | [ET3観測](ACOUSTIC-HISTORY-001/usage-observation-20261014-et3.md)、[KPI](../docs/process/improvement/kpi.md) |
| 七役割・担当割当の運用 | [開発体制](../docs/process/development-team.md) |

旧要約の転載はここから除いた。根拠資料・原証拠・Git履歴は削除していない。再開はこの要約と実git状態から始め、次の未完了工程だけを進める。
