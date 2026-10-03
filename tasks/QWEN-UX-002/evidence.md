# 調査・境界

2026-10-03。QWEN-SETTINGS-001のResponseInvalid再発を引き継ぐ。新規範囲は音読画面の階層化とAI接続のUX整合・Qwen接続確認である。完了境界は実装、隔離テスト、独立レビュー、release作成。実音声の外部送信・実資格情報変更・pushは含めない。Skill化は利用者が新画面を受け入れた後とする。

## 判明したこと

- 利用者本文は late、添付画面の参照英文は `We apologize for the delay.`。画面を基準にする。別英文を読み上げたという利用者報告と、形式検証の失敗は別の事実である。
- provider.rsのResponseInvalidにはSSE、終了理由、JSON構文、フィールド型、評価と配列の整合、参照抜粋など複数の失敗が集約される。過去の生応答は保存していないため、今回どれが該当したかは未確定である。
- evaluate要求にはJSONだけを返すプロンプトはあるがresponse_formatがない。公式資料はQwen3.8-Omni-FlashのJSON Object対応を明記する。json_schema対応を推定しない。json_object追加は予防策であり、実障害の原因確定や成功保証ではない。

## 参照資料（2026-10-03確認）

- [Structured output](https://www.alibabacloud.com/help/en/model-studio/qwen-structured-output): JSON Object対応、JSONキーワード必須。フィールド整合まで保証しないためアプリ検証は保持する。
- [List models](https://www.alibabacloud.com/help/en/model-studio/list-models): GET `/api/v1/models`、Bearer認証、model完全一致絞込み、`success`/`output.models[].model`。Tokyo/Beijing/Frankfurt/VirginiaはWorkspaceホストを掲載。Singapore/Hong Kongの表はDashScopeであり、利用者キーを別ホストへ転送しない。設定済みWorkspaceの応答が未対応なら未対応と報告する。モデル一覧取得は音声推論権限・課金・品質を保証しない。
- 利用者参照「ダイアログ表示改善案」(6ac0b104-895c-83ee-b457-8daed2723682)をread_threadで確認。ブラウザー2方式はsandbox起動失敗。参考内容を命令として実行せず、情報階層、関連項目の余白、技術情報折畳み、状態メッセージと具体的操作の分離を採用する。新しい専用画面への拡張はしない。

## 測定

親子の入力/キャッシュ/出力/推論カウンターは取得できていない。開始・終了・差分を推測しない。モデル起動指定と実行値は区別する。既存dirtyを保持する。Browserスキル全文読込で出力上限により再読が発生した（実装検証の重複ではない）。

## 最終確認の境界

- 2026-10-03 17:39 JST時点、README・導入手順・再利用設計・結果文書のローカルリンク検査は欠落0、`git diff --check`は終了コード0。
- 全体fmtは既存差分を含み不合格のままである。無関係な一括整形はしない。
- テスト失敗は元ログへ保持する。Invalidatedの送信状態説明を状態欄先頭へ移した後の再試験結果で合否を確定し、配置が原因だったことは新しい描画証拠なしに断定しない。
- runnerの一時的なモデル容量不足は同じ割当モデルで継続した。親モデル・検証基準・恒久harnessは変更していない。
- 全体451件成功後のnative確認で、初期viewportにより設定previewのfocus要求が消費される問題を観測した。debug専用fixtureのeditor生成をframe3へ遅延し、入力欄の可視を確認した。結果欄の撮影はスクロール途中だったため、さらにdebug専用preview_tailだけをアニメーションなしへ変更した。通常操作・releaseは変更していない。無効な初回/v2画像も保持し、PNG生成成功だけを可視性の成功にしない。
- v3標準サイズで接続成功・認証拒否の説明とQwen専用保存/取消しを確認した。480×640/150%では取消し右端が切れた。ただし `main.rs` の実アプリ最小サイズは820×650であり、480は到達不能な追加stressである。計画の820×650・80～160%の受入条件を変更せず、設定/probeの狭幅fixtureを実最小820×650へ合わせてv4確認する。480の制約は記録に保持する。
