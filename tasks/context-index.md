# コンテキスト参照インデックス

2026-10-07。入口は [最新要約](current.md)。以下は必要箇所だけ読む。一括ロードせず、ページ内の提案を承認済み・実装済みへ読み替えない。

## 現行タスクの正本・証拠

| 調べること | 参照先 |
| --- | --- |
| 外部仕様・18受入条件 / 内部契約 | [spec.md](ACOUSTIC-HISTORY-001/spec.md)、[design.md](ACOUSTIC-HISTORY-001/design.md) |
| 実装順・所有・役割別モデル/effort・完了条件 | [implementation.md](ACOUSTIC-HISTORY-001/implementation.md) |
| 検証結果・未完了範囲 | [implementation-results.md](ACOUSTIC-HISTORY-001/implementation-results.md) |
| 固定Praat・資源・共有境界 | [design-i0.md](ACOUSTIC-HISTORY-001/design-i0.md) |
| 保存reader/writer・回復契約 | [design-i1b.md](ACOUSTIC-HISTORY-001/design-i1b.md) |
| writer独立指摘 / 試験結果 | [history-writer-review.md](ACOUSTIC-HISTORY-001/history-writer-review.md)、[history-writer-tests.md](ACOUSTIC-HISTORY-001/history-writer-tests.md) |
| Praatプロセス設計 / 修正単位 | [design-i5.md](ACOUSTIC-HISTORY-001/design-i5.md)、[praat-remediation.md](ACOUSTIC-HISTORY-001/praat-remediation.md) |
| M1レビュー対象hash / 凍結EXE・exact引数・環境制約 | [praat-m1-gate.md](ACOUSTIC-HISTORY-001/praat-m1-gate.md)、[praat-m1-tests.md](ACOUSTIC-HISTORY-001/praat-m1-tests.md) |
| M1修正前の利用者RED / 修正内容 / 新凍結EXEと回帰結果 | [praat-m1-user-red.md](ACOUSTIC-HISTORY-001/praat-m1-user-red.md)、[praat-m1-implementation.md](ACOUSTIC-HISTORY-001/praat-m1-implementation.md)、[praat-m1-runner.md](ACOUSTIC-HISTORY-001/praat-m1-runner.md) |
| M1利用者GREEN・独立最終判定 | [praat-m1-user-green.md](ACOUSTIC-HISTORY-001/praat-m1-user-green.md)、[praat-m1-post-review.md](ACOUSTIC-HISTORY-001/praat-m1-post-review.md) |
| M2生成WAV整合性 / W1 writer安全性の補完 | [praat-m2-design.md](ACOUSTIC-HISTORY-001/praat-m2-design.md)、[history-w1-packet.md](ACOUSTIC-HISTORY-001/history-w1-packet.md)、[独立gate記録](ACOUSTIC-HISTORY-001/m2-w1-gates.md) |
| M2独立試験・差分とRED/GREEN / W1最終実行 | [praat-m2-tests.md](ACOUSTIC-HISTORY-001/praat-m2-tests.md)、[praat-m2-implementation.md](ACOUSTIC-HISTORY-001/praat-m2-implementation.md)、[history-w1-tests.md](ACOUSTIC-HISTORY-001/history-w1-tests.md)、[history-w1-runner.md](ACOUSTIC-HISTORY-001/history-w1-runner.md) |
| M2全回帰の未解決失敗 / 隔離診断の規範 | [praat-m2-runner.md](ACOUSTIC-HISTORY-001/praat-m2-runner.md)、[process-cancel-diagnostic-design.md](ACOUSTIC-HISTORY-001/process-cancel-diagnostic-design.md) |
| 隔離診断fixtureの変更・自己試験・環境制約 | [process-cancel-d1-tests.md](ACOUSTIC-HISTORY-001/process-cancel-d1-tests.md) |
| 隔離診断の一回実行・元cancel未再現・残存ledger | [process-cancel-d1-runner.md](ACOUSTIC-HISTORY-001/process-cancel-d1-runner.md) |
| 原本fixture安全補正・tree-exit同定・新M1受入の契約 | [process-cancel-adoption-design.md](ACOUSTIC-HISTORY-001/process-cancel-adoption-design.md) |
| 原本fixture採用のfocused証拠 / module順fmt | [process-cancel-a1-tests.md](ACOUSTIC-HISTORY-001/process-cancel-a1-tests.md)、[process-cancel-a1-integration.md](ACOUSTIC-HISTORY-001/process-cancel-a1-integration.md) |
| 原本fixture最終回帰 / 新M1固定EXE・hash・外部確認条件 | [process-cancel-a1-runner.md](ACOUSTIC-HISTORY-001/process-cancel-a1-runner.md) |
| 新M1の利用者GREEN・受領後hash / A1・M2最終判定 | [process-cancel-a1-user-green.md](ACOUSTIC-HISTORY-001/process-cancel-a1-user-green.md)、[独立gate記録](ACOUSTIC-HISTORY-001/m2-w1-gates.md) |
| 保存回復I1b-2bの実装段階・API scaffold | [history-recovery-implementation.md](ACOUSTIC-HISTORY-001/history-recovery-implementation.md) |
| 保存回復の独立32件・行動RED / I2準備設計 | [history-recovery-tests.md](ACOUSTIC-HISTORY-001/history-recovery-tests.md)、[design-i2.md](ACOUSTIC-HISTORY-001/design-i2.md) |
| I2設計の独立指摘・再判定 | [history-commit-design-review.md](ACOUSTIC-HISTORY-001/history-commit-design-review.md) |
| 保存回復の独立レビュー・受入不足 | [history-recovery-review.md](ACOUSTIC-HISTORY-001/history-recovery-review.md) |
| 保存回復34件・既存writer/readerの独立実行 | [history-recovery-runner.md](ACOUSTIC-HISTORY-001/history-recovery-runner.md) |
| I2a-0 Path読取接合の実装・scaffold | [history-path-implementation.md](ACOUSTIC-HISTORY-001/history-path-implementation.md) |
| I2a-0 独立11件とコード判定 | [history-path-tests.md](ACOUSTIC-HISTORY-001/history-path-tests.md)、[history-path-review.md](ACOUSTIC-HISTORY-001/history-path-review.md) |
| I2a-0 新11/既存37の独立実行 | [history-path-runner.md](ACOUSTIC-HISTORY-001/history-path-runner.md) |
| I2a-G 保存排他の実装段階・scaffold | [history-gate-implementation.md](ACOUSTIC-HISTORY-001/history-gate-implementation.md) |
| I2a-G 独立24件・RED証拠・固定期待値 | [history-gate-tests.md](ACOUSTIC-HISTORY-001/history-gate-tests.md) |
| I2a-G 独立コード・oracle判定 | [history-gate-review.md](ACOUSTIC-HISTORY-001/history-gate-review.md) |
| I2a-G 新24/既存88の独立実行・最終固定 | [history-gate-runner.md](ACOUSTIC-HISTORY-001/history-gate-runner.md) |
| I2a-M/R 移行退避の限定設計・正本Pre | [history-migration-design.md](ACOUSTIC-HISTORY-001/history-migration-design.md)、[history-commit-design-review.md](ACOUSTIC-HISTORY-001/history-commit-design-review.md) |
| I2a-M 移行退避の実装段階・scaffold | [history-migration-implementation.md](ACOUSTIC-HISTORY-001/history-migration-implementation.md) |
| I2a-M 独立21件・RED・固定fixture/期待値 | [history-migration-tests.md](ACOUSTIC-HISTORY-001/history-migration-tests.md) |
| I2a-M 独立コード・oracle判定 | [history-migration-review.md](ACOUSTIC-HISTORY-001/history-migration-review.md) |
| I2a-M 独立62件・固定入力・清掃 | [history-migration-runner.md](ACOUSTIC-HISTORY-001/history-migration-runner.md) |
| I2a-S codec/read/headの限定設計（Pre PASS、実装中） | [history-progress-design.md](ACOUSTIC-HISTORY-001/history-progress-design.md) |
| I2a-S未接続scaffold・旧62件維持 | [history-progress-implementation.md](ACOUSTIC-HISTORY-001/history-progress-implementation.md) |
| I2a-S独立期待値・M改訂対応・行動RED | [history-progress-tests.md](ACOUSTIC-HISTORY-001/history-progress-tests.md) |
| I2a-Sコード・oracle独立レビュー | [history-progress-review.md](ACOUSTIC-HISTORY-001/history-progress-review.md) |
| I2a-S固定4EXEの独立81件実行 | [history-progress-runner.md](ACOUSTIC-HISTORY-001/history-progress-runner.md) |
| I2a-N通常保存の限定設計（最終Pre PASS） | [history-normal-save-design.md](ACOUSTIC-HISTORY-001/history-normal-save-design.md) |
| I2a-N本実装・初回99/1履歴 | [history-normal-save-implementation.md](ACOUSTIC-HISTORY-001/history-normal-save-implementation.md) |
| I2a-N独立試験・daily I/O delta | [history-normal-save-tests.md](ACOUSTIC-HISTORY-001/history-normal-save-tests.md) |
| I2a-N最終Post PASS・OS oracle訂正根拠 | [history-normal-save-review.md](ACOUSTIC-HISTORY-001/history-normal-save-review.md) |
| I2a-N独立100件成功 | [history-normal-save-runner.md](ACOUSTIC-HISTORY-001/history-normal-save-runner.md) |
| I2a-R教材復旧の限定設計（正本統合・最終Pre PASS） | [history-material-recovery-design.md](ACOUSTIC-HISTORY-001/history-material-recovery-design.md) |
| I2a-R本実装・固定source再build・自己125件 | [history-material-recovery-implementation.md](ACOUSTIC-HISTORY-001/history-material-recovery-implementation.md) |
| I2a-R独立25件・旧100件の検証境界 | [history-material-recovery-tests.md](ACOUSTIC-HISTORY-001/history-material-recovery-tests.md) |
| I2a-R最終Post PASS・3指摘の閉鎖証拠 | [history-material-recovery-review.md](ACOUSTIC-HISTORY-001/history-material-recovery-review.md) |
| I2a-R独立125件成功 | [history-material-recovery-runner.md](ACOUSTIC-HISTORY-001/history-material-recovery-runner.md) |
| I2a-T教材保存の限定設計（正本統合・最終Pre PASS） | [history-material-save-design.md](ACOUSTIC-HISTORY-001/history-material-save-design.md) |
| I2a-T本実装・固定source build・自己146件 | [history-material-save-implementation.md](ACOUSTIC-HISTORY-001/history-material-save-implementation.md) |
| I2a-T独立21件・正式M配列改訂・静的指摘訂正 | [history-material-save-tests.md](ACOUSTIC-HISTORY-001/history-material-save-tests.md) |
| I2a-T最終Post PASS・独立146件 | [history-material-save-review.md](ACOUSTIC-HISTORY-001/history-material-save-review.md)、[history-material-save-runner.md](ACOUSTIC-HISTORY-001/history-material-save-runner.md) |
| I2a-V履歴publication権限の限定設計（正本統合・最終Pre PASS） | [history-publication-authority-design.md](ACOUSTIC-HISTORY-001/history-publication-authority-design.md) |
| I2a-V本実装・初回100/101・旧I1互換保持 | [history-publication-authority-implementation.md](ACOUSTIC-HISTORY-001/history-publication-authority-implementation.md) |
| I2a-V独立19件・V08限定訂正・最終Post PASS | [history-publication-authority-tests.md](ACOUSTIC-HISTORY-001/history-publication-authority-tests.md)、[history-publication-authority-review.md](ACOUSTIC-HISTORY-001/history-publication-authority-review.md) |
| I2a-V独立101件成功・temp集計境界 | [history-publication-authority-runner.md](ACOUSTIC-HISTORY-001/history-publication-authority-runner.md) |
| I2a-P履歴pointer確定の限定設計（正本統合・最終Pre PASS） | [history-pointer-commit-design.md](ACOUSTIC-HISTORY-001/history-pointer-commit-design.md)、[最終Pre記録](ACOUSTIC-HISTORY-001/history-commit-design-review.md) |
| I2a-P保存確定処理（本体・新22件GREEN、最終Post PASS/Closed） | [history-pointer-commit-implementation.md](ACOUSTIC-HISTORY-001/history-pointer-commit-implementation.md)、[最終判定](ACOUSTIC-HISTORY-001/history-pointer-commit-review.md) |
| I2a-P独立22件の行動RED・限定診断・環境比較 | [history-pointer-commit-tests.md](ACOUSTIC-HISTORY-001/history-pointer-commit-tests.md)、[review](ACOUSTIC-HISTORY-001/history-pointer-commit-review.md) |
| I2a-P独立関連回帰（初回75成功＋再開194成功＝269、元4失敗保持） | [history-pointer-commit-runner.md](ACOUSTIC-HISTORY-001/history-pointer-commit-runner.md) |
| I2a-P失敗4件のsandbox内外比較・独立再開条件 | [history-pointer-commit-diagnostic.md](ACOUSTIC-HISTORY-001/history-pointer-commit-diagnostic.md) |
| I2b-Aアプリ主要3保存経路・独立16件の行動REDと環境条件 | [設計接合確認](ACOUSTIC-HISTORY-001/design-i2.md)、[独立テスト](ACOUSTIC-HISTORY-001/history-main-consumer-tests.md) |
| I2b-A本体3関数・自己20件・固定EXEと後続実行条件 | [実装記録](ACOUSTIC-HISTORY-001/history-main-consumer-implementation.md) |
| I2b-A独立20件を受領した最終Post PASS/Closed | [レビュー記録](ACOUSTIC-HISTORY-001/history-main-consumer-review.md) |
| I2b-A新16/旧4の独立20件成功・最終Postへの固定証拠 | [独立実行記録](ACOUSTIC-HISTORY-001/history-main-consumer-runner.md) |
| I2b-B backup接合・Pre PASS・B-WB1 Closed・runner-readyと次の独立76件 | [設計追補](ACOUSTIC-HISTORY-001/design-i2.md)、[限定計画](ACOUSTIC-HISTORY-001/implementation.md)、[レビュー](ACOUSTIC-HISTORY-001/history-backup-review.md) |
| I2b-B独立42件・25行動RED/17PASS・fixture訂正と環境比較 | [独立テスト](ACOUSTIC-HISTORY-001/history-backup-tests.md) |
| I2b-B本体接合・自己75件・世代/環境の証拠境界 | [実装記録](ACOUSTIC-HISTORY-001/history-backup-implementation.md) |
| I2b-B最終固定4EXE・独立76件成功・初回setup失敗と限定再実行 | [独立runner](ACOUSTIC-HISTORY-001/history-backup-runner.md) |

ソース入口：保存は `src/audio_history/`、共有履歴型は `tools/practice-history/`、音響解析は `tools/acoustic-analysis/`。生ログは各試験記録から辿る。要約へログ本文を複製しない。

## docs以下

| 内容 | 参照先 |
| --- | --- |
| 再利用設計の総合入口・音響機能 | [reuse/README.md](../docs/reuse/README.md)、[acoustic-analysis.md](../docs/reuse/acoustic-analysis.md) |
| Framework・ライブラリ化候補 | [framework.md](../docs/reuse/framework.md)、[library-candidates.md](../docs/reuse/library-candidates.md) |
| 音声I/O・原本管理・Qwen | [audio-io.md](../docs/reuse/audio-io.md)、[asset-store.md](../docs/reuse/asset-store.md)、[qwen-audio.md](../docs/reuse/qwen-audio.md) |
| Markdown・Codex・チャット・共通UI | [markdown.md](../docs/reuse/markdown.md)、[codex-app-server.md](../docs/reuse/codex-app-server.md)、[chat.md](../docs/reuse/chat.md)、[egui-controls.md](../docs/reuse/egui-controls.md) |
| UI操作と完了表示 | [interaction-controls.md](../docs/design/interaction-controls.md)、[ux-completion.md](../docs/design/ux-completion.md) |
| 設計判断の理由 | [Windows harness](../docs/decisions/0001-windows-harness.md)、[再利用境界](../docs/decisions/0002-reusable-library-boundaries.md)、[学習Framework](../docs/decisions/0003-learning-framework-boundaries.md) |
| 開発7役・利用量記録 | [development-team.md](../docs/process/development-team.md)、[kpi.md](../docs/process/improvement/kpi.md)、[measurements.md](../docs/process/improvement/measurements.md) |
| CI・改善試行 | [windows-ci.md](../docs/process/improvement/windows-ci.md)、[github-trial.md](../docs/process/improvement/github-trial.md) |
| 原因分析・振り返り・改善項目 | [root-cause-analysis.md](../docs/process/improvement/root-cause-analysis.md)、[retrospective.md](../docs/process/improvement/retrospective.md)、[improvement-actions.md](../docs/process/improvement/improvement-actions.md) |
| 別PC導入・開発依頼 | [install-windows.md](../docs/install-windows.md)、[development-requests.md](../docs/development-requests.md) |

## WordWeave5ワークスペースのページ

検索で確認した索引であり、網羅性は保証しない。ページは編集していない。下位ページはタイトル・プレビューで分類したものであり、適用前に必要箇所を読む。

| ページ | 内容の所在 |
| --- | --- |
| [WordWeave5](https://chatgpt.com/space/page_6ac1f63830b48191ba5552d9794fa9f4) | Space入口。本文は紹介用テンプレートであり実装状況の正本ではない |
| [Harnessの分析](https://chatgpt.com/space/page_6ac21748dce08191bad523b7b97854d7) | 会話・教材作成を扱うHarness構造 |
| [「Harnessの分析」追記内容](https://chatgpt.com/space/page_c9ca0bc6774c819199240a40ba88b8ef) | Harness研究との照合・追加検討 |
| [Codex開発運用の適用方針](https://chatgpt.com/space/page_6ac37404370481918c3232c8c9202ffa) | モード・規約・Skill等の運用提案 |
| [GithubのCIの使い方](https://chatgpt.com/space/page_6ac2190e789c8191b6f7458f17ca9f7f) | CIとGitHub Actionsの方針 |
| [設定画面UX](https://chatgpt.com/space/page_6ac21d2949d88191b34b46ccf54cb196) | 設定編集・学習目標・Learning Planner |
| [ダイアログUX](https://chatgpt.com/space/page_d1911b3262b48191afd3eed618da74e1) | 発音確認ダイアログのUI/UX方針 |

## 通常は読み込まない履歴

- [整理前のcurrent.md](current-history-20261007.md)：過去時点の証拠が必要な場合だけ参照する。復元可能な退避であり削除ではない。
- 過去タスク・重複した実行ログ・探索途中の案は元の場所に保持する。古い未承認案や失敗ログを現在の実行計画へ混ぜない。
