# Windows Harness（本バージョン）

2026-09-15。前回提案に対する利用者の設計・実装指示に基づく。Android・遠隔接続は対象外。

| モジュールID | 責務 | 依存 |
| --- | --- | --- |
| media-assets | 原録音・筆跡・送信画像の保存、整合性検査、退避 | 既存Storage |
| chat-input | 録音→認識→確認、注釈付き画像送信、履歴再生・読み上げ | media-assets |
| material-review | 項目単位左右比較、変更理由、固定した根拠の並置、承認登録 | chat-input、既存material |
| run-journal | 生成意図・応答の保存、状態と再読込、保存再試行 | 既存Codex接続、Storage |

順序：現行テスト → media-assetsとrun-journalの契約・試験 → chat-input → material-review → 統合回帰。
全面プラグイン化、依存の一括更新、Codex以外のAI接続は行わない。
