# QWEN-AUDIO-001 確認済み資料と開始境界

2026-10-02。利用者は小規模な独立音声評価アプリをCodexから利用し、品質検証後にWordWeave5へ組み込める設計・実装を承認した。Model Studio有効化/APIキー作成済み、Japan (Tokyo)。キー値・Workspace IDは未取得であり、ローカル画面で入力する。

## 確認した公式資料

- [Qwen-Omni](https://www.alibabacloud.com/help/en/model-studio/qwen-omni)：qwen3.8-omni-flashはTokyoで提供。音声入力・テキスト出力、Chat Completions対応。ローカル音声のBase64は10MB未満。stream/include_usageを使用。reasoning_effortはnone/low/medium/xhighなど、既定xhigh。旧qwen3-omniのenable_thinkingを混同しない。
- [接続先](https://www.alibabacloud.com/help/en/model-studio/compatibility-of-openai-with-dashscope)：Tokyoは `https://{WorkspaceId}.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1`。API Hostを利用者が管理画面から取得する。ワークスペース値は推測しない。
- [APIキー](https://www.alibabacloud.com/help/en/model-studio/get-api-key)：リージョンに対応したキーが必要。キー値を会話・ソース・ログへ出さない。
- [Codex MCP](https://learn.chatgpt.com/docs/extend/mcp)：ローカルSTDIOサーバーを利用可能。登録はcommand/args。APIキーをMCP引数や設定断片に含めない。
- [MCP STDIO](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)、[Tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)：JSON-RPC、改行区切り。stdoutはプロトコルのみ。
- [Windows CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)：専用の汎用資格情報を利用する案。既存の利用者資格情報を列挙/変更しない。

## 実装・検証の境界

### 実装用API契約の確認（同日追記）

Qwen-Omni資料の「Input Base64-encoded local file / Audio」のqwen3.8例は、user messageのcontent要素 `{"type":"input_audio","input_audio":{"data":"data:;base64,<符号化音声>","format":"mp3"}}` を使用する。今回のWAVでは対応形式名`wav`を指定し、同じdata URI形式を使う。`modalities:["text"]`、`stream:true`、`stream_options:{"include_usage":true}`。本文上位に`reasoning_effort:"medium"`を明示する。要件にない検索・ツールは要求しない。JSON出力schemaのサーバー強制対応は未確認のため、promptとローカル構造検証を使い、未確認パラメーターを足さない。

[Streaming output](https://www.alibabacloud.com/help/en/model-studio/stream) のOpenAI互換curl応答例で、SSE `data:` JSONの `choices[0].delta.content` を連結、`finish_reason:"stop"`、次に空choicesのusage、最後に`data: [DONE]`を確認した。[Chat API](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions) も確認済みである。`length`は途中打切りなので成功扱いしない。要求effortと返却model/usageは別に管理し、返されないeffortを実値として補完しない。

- 独立アプリ、原音と英文を1件ずつ扱う。発音・強弱・リズムについて日本語の改善フィードバック。客観的な点数/音素正解率は未検証であり標榜しない。
- 今回の有料送信は利用者がローカル画面で明示して開始する。オフライン試験は合成WAV/モックのみ。利用者の録音・教材・APIキーを開発用試験へ流用しない。
- MCP経由では送信前に対象/送信先/結果がCodexへ戻ることを表示する。自動再送・代替モデル呼出しなし。取消し/timeout後の課金・完了状態は不明になり得る。
- コード/独立アプリのビルド/自動検証/独立レビューまでを今回の開発完了境界とする。実キーによる接続と音声評価品質、利用者操作は別の受入項目として残す。
- 既存本体のCodex認証、教材登録、保存データへ変更を加えない。MCPの利用者設定変更・公開・pushはこの開発に含めない。

## 開始時の証拠

- HEAD `d86e7e9d4fe4497d8be9040d30a078582f2887d8` / branch `codex/development-agent-team`。既存dirtyを確認して保護。
- 親ログ末尾1MiBから本文を出さず取得。観測2026-10-02T12:04:19.317Z（21:04:19 JST）。初動読込は観測前のため欠測。
- input 872401075 / cached_input 848583296 / output 1961425 / reasoning_output 610324 / total 874362500。内数を再加算しない。子の包含関係と実行model metadataは未取得。
