# WordWeave 5 0.3 実装構成

| ファイル | 責務 |
| --- | --- |
| src/app.rs | egui画面・学習セッション・教材操作・生成キュー |
| src/ai.rs | 出題・添削・教材/例文生成・翻訳・認識のプロンプトと応答検証、NGSLのHTTPS取得 |
| src/codex.rs | ネイティブ実行ファイル解決、app-server起動、JSON行通信、認証、ターン完了・中断処理 |
| src/execution.rs | Codexが返したモデル・effortの取得、生成中/前回の状態表示 |
| src/chat.rs | 会話履歴・下書き・メモ、送信する直近履歴と指定した過去発言の選択 |
| src/material.rs | 選択チャットからの教材化要求、追加/訂正の分離、重複除去、比較元の競合検査、会話参照 |
| src/model.rs | 旧15列/新17列TSV、言い換え配列・例文配列、形式検査 |
| src/learning.rs | NGSL・提供語CSVの解析、AI生成教材と問題の検証 |
| src/store.rs | 保存・ロック・バックアップ・旧設定の移行 |
| src/scheduler.rs | 間隔学習、技能別成績、新規上限、段階的ヒント |
| src/ink.rs / src/media.rs | ペン・読み上げ・録音再生 |
| tests/support/mock_codex.rs | Cargoが自動ビルドするRust製の模擬Codex。外部サービス接続なし |
| tests/codex_process.rs | モック子プロセスを使うオフライン通信試験（Windows/Unix共通） |

## Codexプロトコル

`Command`で指定された実行ファイルを起動する。既知のVolta shimでは同一フォルダーまたはPATHのvolta.exeを使い、run/codex/app-serverの3引数を渡す。他のcmd/bat用の制約付きCMD経路も残る（一般的なバッチ起動の問題が解消したとは扱わない）。ネイティブEXEはapp-serverの1引数で起動する。

## 英語チャットと実行設定（0.3.7）

0.3.8では各Exchangeに教材化用の選択フラグを追加する。通常のチャットの参照指定とは別である。material::Requestが選択した往復と対象のEntryだけを教材生成へ渡す。返却IDは使用せず、既存IDまたはホスト側の新規IDを割り当てる。material::Draftをprogress.jsonに保存し、画面で編集後に検証して既存のimport_deck経路で登録する。

追加モードは基本項目をホスト側で元に戻し、既存の配列要素も上書きしない。訂正モードは全内容を比較・編集できる。登録前に比較元のTSV全体を照合するため、fingerprintに含まれない追加例文の競合も検出する。元の会話IDと選択した往復の添字をSourceに記録する。既存の復習判定はEntry::fingerprintに従う。

会話の正本はWordWeaveのprogress.json内のchatsである。成功した質問/回答の組、回答時にCodexが返したモデルとeffort、引き継ぎメモ、次回参照フラグ、未送信/失敗時の下書きを保存する。既存の原子的保存・バックアップ・エクスポートに含める。旧版データにchatsがなければ空配列として読む。

各質問では新しいephemeral threadを作る。Codexの永続thread IDには依存しない。質問・メモ・参照指定した過去発言を先に選び、残りの容量に直近の完了済み会話を新しい順から追加し、時系列順で送信する。JSON payloadは64,000 UTF-8 bytes以下。これはアプリ側の容量制限であり、モデルのトークン上限そのものではない。必須部分が上限を超えた場合は送信を止め、参照指定を勝手に落とさない。保存履歴の削除や自動要約はしない。検索は画面内の検索であり、検索結果が自動送信されるわけではない。

thread/startの応答のmodelとreasoningEffortをExecutionに変換する。欠落/null/空文字を既定値で補わない。UI表示用の最新状態はメモリ内に持ち、RunのDropで処理中を解除する。接続確認はaccount/readまでなので、新しい実行設定は取得しない。チャットの完了結果には同じリクエストのExecutionを添えて返し、グローバルな最新値を回答履歴へ混入させない。turn/startではモデル・effortを上書きしない。

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
