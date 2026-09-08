# WordWeave 5 0.3 実装構成

| ファイル | 責務 |
| --- | --- |
| src/app.rs | egui画面・学習セッション・教材操作・生成キュー |
| src/ai.rs | 出題・添削・教材/例文生成・翻訳・認識のプロンプトと応答検証、NGSLのHTTPS取得 |
| src/codex.rs | ネイティブ実行ファイル解決、app-server起動、JSON行通信、認証、ターン完了・中断処理 |
| src/model.rs | 旧15列/新17列TSV、言い換え配列・例文配列、形式検査 |
| src/learning.rs | NGSL・提供語CSVの解析、AI生成教材と問題の検証 |
| src/store.rs | 保存・ロック・バックアップ・旧設定の移行 |
| src/scheduler.rs | 間隔学習、技能別成績、新規上限、段階的ヒント |
| src/ink.rs / src/media.rs | ペン・読み上げ・録音再生 |
| tests/support/mock_codex.rs | Cargoが自動ビルドするRust製の模擬Codex。外部サービス接続なし |
| tests/codex_process.rs | モック子プロセスを使うオフライン通信試験（Windows/Unix共通） |

## Codexプロトコル

`Command`でネイティブCodexを起動する。シェルやユーザー文字列を連結したコマンドは使用しない。npm版のcmdラッパーは実行せず、同梱のネイティブexeを探す。

1. `initialize` → 応答確認 → `initialized`
2. `account/read` → `account.type == chatgpt` の場合だけ継続
3. `thread/start`：OpenAI提供元、read-only、approvalPolicy=never、ephemeral、教材生成の指示
4. 音声入力では `model/list` で選択モデルのaudio対応を確認
5. `turn/start`：テキスト/画像/音声入力、構造化教材には `outputSchema`
6. `item/completed` のagentMessageを収集。`turn/completed` がcompletedのときだけ本文を返す
7. 子プロセスを終了・回収

開始応答より前に届くイベントを一時保持する。別thread/turnの通知を無視する。final_answerがあればそれを優先し、deltaと完了本文を二重に結合しない。失敗・中断ターンの部分出力は登録しない。標準エラーは別スレッドで排出し、パイプ詰まりを防ぐ。応答1行4MB、1回5分、読み取り待ち100ms刻み。キャンセルフラグ、エラー、タイムアウトで子プロセスを終了・waitする。

サーバーからの承認・ツール要求は実行せず、未対応エラーを返して生成を中止する。プロンプトでもコマンド・ファイル読み取り・外部ツールを使わないよう指定する。ローカルのCodex設定や組織の管理ポリシーにより起動が制限された場合、アプリはその制限を解除しない。

`OPENAI_API_KEY` / `CODEX_API_KEY` は子へ引き継がない。アプリはトークンを読み取らず、Codexのログイン状態を利用する。APIキー認証・別提供元は拒否し、有料APIへの自動切り替えを持たない。モデルは設定可能、空欄はCodexの既定値である。

参照：[公式仕様](https://learn.chatgpt.com/docs/app-server)。実装時には環境のCodexが出力するJSON Schemaとも照合した。音声inputとinputModalities=audioは環境のスキーマで確認したが、実サービス・全バージョンでの対応を保証しない。

## 保存と再開

生成キューはUI側で永続化し、先頭語は成功した教材の保存後に除去する。失敗するとキューを残して停止する。教材保存とキュー保存の間で落ちても、再開時に生成済みIDを調べて重複を防ぐ。無限の自動リトライは行わない。1日の試行回数は実行前に保存する。

生成元に関係なく基本語から決定的なweb_ IDを生成し、同じ語の自動教材を重複させない。内蔵教材のIDは変更しない。日本文はprogress.jsonの下書きに先に保存し、翻訳成功後に例文として保存して下書きを除去する。

教材は最大64MB/10,000件。言い換え30件、例文200件/教材。拡張配列はTSV末尾のJSON列で表す。旧15列のデータと復習fingerprintを維持する。FNV-1a fingerprintは元15列を対象とし、追加例文だけでは既存の成績をリセットしない。

NGSL取得のTLSはWindowsの証明書ストアを使うnative-tls（Schannel）。HTTPS限定、タイムアウトとサイズ上限を設ける。AI生成はHTTPクライアントを使わない。

## 検証とビルド

Cargo.lockを同梱。`cargo test --all-targets --locked` は共通ロジックの試験を実行する。Windowsの画面・音声モジュールはWindows向け依存節を使用するため、Linuxでの通常テストだけでは検査されない。0.3.0ではWindowsターゲットの `cargo check --all-targets` を別途実行した。0.3.1の修正版は今回の環境にRustがないため未コンパイル・未実行である。結果・未確認範囲はVALIDATION.mdを参照する。
