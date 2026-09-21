# 現在の状態（2026-09-21）

- アプリは0.6.0。今回の開発Harness整備では実装・依存関係・EXE・利用者データを変更していない。
- UX変更は `8c3d7cb` に統合済み。以前の自動検証は198件成功・releaseビルド成功である。[検証範囲](../validation/ux-0.6.0.md)。これは今回の再実行結果ではない。
- 未完了は保全済み実記録の更新/再起動/復元、学習一巡、実IME・マイク・ペン・TTS・実Codex、小画面の実機受入である。[受入手順](../UPGRADE-0.6.0.md)。自動試験の成功で代用しない。
- 開発の入口は [AGENTS.md](../AGENTS.md)。次の新規タスクには目的・範囲・完了条件だけを渡す。[依頼例と設定上の制約](../docs/development-requests.md)。
- 過去の開発経緯が必要な場合だけ [tasks/todo.md](todo.md) や対象バージョンの設計/検証を参照する。
- 次の機能開発では、着手時に対象と完了境界を確定し、[トークン測定手順](../docs/process/improvement/kpi.md) に沿って取得可能範囲を確認・記録する。新機能の内容自体はまだ指定されていない。
- [GitHub比較試行](../docs/process/improvement/github-trial.md) と [改善提案・承認条件](../docs/process/improvement/improvement-actions.md) を文書化済みである。計画の整備と、CI導入・常時自動計測の実装は別であり、後者を完了扱いにしない。
