# QWEN-FORMATS-001 検証設計

親（割当Astra/high）が仕様からintegration期待値を作成した。backend実装は別agent、root GUI実装は親のため、その期待値は独立reviewerへ確認を依頼する。実音声・資格情報・教材は使用しない。

| 対象 | 期待値/証拠 |
| --- | --- |
| 6形式 | 合成440Hz各形式をdecodeし、形式/長さ/PCM再生情報を照合。AMR-WB SIDも構成する |
| 元bytes/確認 | 模擬HTTPのBase64を独立復号し元bytesと完全一致。core・単独・本体controller、明示送信前0回/後1回 |
| 不正入力 | 末尾切断、途中rate変更、動画、外部dref、複数stsd、空、超過を拒否 |
| 独立上限 | 圧縮40秒stereo48kを受理（PCMのみ6MiB超過）、60.1秒を拒否。既存WAV上限回帰も維持 |
| snapshot/取消し | 拡張子に依存せず検査、元file削除後も固定。選択cancel保持、別入力開始時旧確認無効、読込cancel/close/対象変更で遅延結果を採用しない |
| UI | 指定取消し注記1回・例文前、削除3群と旧段落の不在を描画textで検査。通常・狭幅/拡大のnative合成画像 |
| backend | 担当によるprivate試験口で探索・timeout・出力上限・cancel/reap・一時copy削除/失敗通知を検査。独立reviewerが期待値と経路を照合 |

最初のREDは新API未定義によるcompile失敗である。初回ログにはテストimportの誤りも含み、修正した。behavior失敗の証拠とは区別する。非同期既存テストは同期選択後すぐprepareから、期限付きload完了待ちへ変更した。API送信・実マイク・実音声出力の品質は本自動検証で証明しない。

実行結果、件数、未確認はresults.mdへ記録する。FFmpeg不足をskipして対応合格とはしない。
