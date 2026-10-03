# Qwen 音読レビュー

録音した英文と、読む予定の英文を比較し、日本語の練習用フィードバックを得るWindowsアプリである。Alibaba Cloud Model Studioの `qwen3.8-omni-flash` を使用する。地域は東京・シンガポール・北京・香港・フランクフルト・バージニアから選ぶ。WordWeave5本体の設定や学習記録は変更しない。

発音・強弱・リズムへの助言を目的とする。試験の採点や、検証済みの音素正解率ではない。聞き取れない場合は「評価できない」と表示する。実モデルによる助言の妥当性は、利用者の録音で別途確認する必要がある。

## 初回の設定

本体でも「教材」→「主な例文」→「読んで発音を確認」から利用できる。評価ライブラリは共通、接続設定は別々である。本体は「設定 → AI接続 → 音読評価：Qwen」の「Qwen接続設定を編集」で地域・Host・キーを保存する。Qwen専用の保存・キャンセルは通常設定とは独立している。単独ツールのキーを自動コピーしない。本体の録音は最大30秒、原音と結果は画面を閉じるまでのメモリに限り、成績やチャットへ保存しない。

1. `qwen-audio.exe` を起動する。
2. 接続設定で契約した地域を選び、同じ地域のWorkspace専用API HostとAPIキーを入力する。
3. 「保存」を押す。保存だけではAPIを呼び出さない。

API Hostは次の形式である。`{WorkspaceId}`には自分のワークスペースIDが入る。東京の例：

```text
https://{WorkspaceId}.ap-northeast-1.maas.aliyuncs.com/compatible-mode/v1
```

他の地域は `ap-southeast-1`（シンガポール）、`cn-beijing`（北京）、`cn-hongkong`（香港）、`eu-central-1`（フランクフルト）、`us-east-1`（バージニア）である。アプリは選択地域とURLの一致を確認する。任意の互換APIや旧DashScope共用URLは今回の入力対象ではない。地域とWorkspaceで対象モデルの利用権・提供範囲を確認する。地域選択だけで処理全体の所在地を保証しない。[公式の地域・エンドポイント](https://www.alibabacloud.com/help/en/model-studio/regions)、[Chat Completionsの接続先](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions)。

キーはWindowsの、このアプリ専用の資格情報に保存する。Codexの会話、MCP設定、ソースコードに貼り付けない。設定を取り消した場合や保存に失敗した場合は、以前の設定を保つ。

同じ接続先ならキーを空欄にして保存済みのキーを維持できる。地域またはWorkspaceを変える際には、新しい接続先のキーを入力し直す。以前のキーを別の接続先へ自動転用しない。既存の東京設定は再登録なしに読み込める。

## 音読を評価する

1. 読む予定の英文を入力する。
2. 録音済みの音声ファイルを選ぶ。
3. ファイル名、長さ、英文、送信先を確認して「評価を送信」を押す。
4. 「聞き取れた内容」「良かった点」「改善点」を確認し、次の練習に使う。

例文と異なる英文が聞き取れた場合は、その英文と読み直し案内を表示し、例文の発音改善点は作らない。無音等の評価不能、AIの応答形式が不正な場合とは区別する。音声の一致判定もAIの推定であり、実音声での正確さを保証しない。形式エラーだけから「別の英文を読んだ」と推定しない。

対応形式はWAV（PCM16）、MP3（Layer III）、AAC（ADTS）、AMR（NB/WB）、3GP・3GPP（AAC/AMRの音声一つ）である。モノラルまたはステレオ、8～48kHz、復号後60秒以内、元ファイル全体6MiB以内の両上限を満たす必要がある。動画入り・複数音声・外部参照付き3GP、MP3 free-format、AAC PCE、M4A/MP4は受け付けない。これらはアプリの検査範囲であり、API全体の制限とは異なる。選択後に元ファイルを録り直した場合は再選択する。

WAV以外の検査・再生用復号にはFFmpegとffprobeが必要である。アプリ隣の`media-tools`フォルダー、次にPATHの絶対ディレクトリから両実行ファイルを探索する。不足時は理由を表示し、自動インストールや自動送信はしない。別PCへ配布する場合は信頼できる配布元・使用codec・ライセンスを確認して両ツールを用意する。

AMR-WBには`libopencore_amrwb` decoderを持つFFmpegが必要である。3GPのfragmented（moof）構造は扱わない。MP3タグは先頭ID3v2と末尾ID3v1に対応する。形式名が一致してもこの検査profile外のファイルは理由を表示して拒否する。

送信するのは選択時に固定した元ファイルのbytesであり、再生用PCMへ置き換えない。3GP系のみシーク検査のため短命の一時コピーを作り、通常完了・失敗・取消し・アプリ終了時に削除する。削除失敗は通知する。アプリ強制終了やOS停止時には一時音声が残る可能性がある。元ファイルを変更・削除せず、録音・結果の永続履歴は追加しない。

「評価を送信」で録音と英文をAlibaba Cloudへ送信する。従量課金が発生し得る。取消しや通信切断によって、サーバー側の処理や課金が取り消されるとは限らない。失敗後も自動で再送しない。

## Codexから利用する

CodexのMCP設定へ、以下のローカルサーバーを登録する。アプリの配置場所を変更した場合はcommandの絶対パスも変更する。APIキーをこの設定に含める必要はない。

```toml
[mcp_servers.qwen_audio]
command = 'F:\Kazuhiro\GitHub\WordWeave5\target\qwen-audio\release\qwen-audio.exe'
args = ['--mcp']
startup_timeout_sec = 15
tool_timeout_sec = 30
```

このリポジトリには設定例だけを含める。既存のCodex設定は自動変更しない。

Codexへの依頼例：

> 音読レビューを開いてください。読む英文は「The sun is bright.」です。

専用画面で録音を選び、送信を実行する。MCP経由で開いた画面では、評価結果がCodexへ返ることも表示する。APIキーと録音データはMCPの結果に含めない。

| 操作 | 動作 |
| --- | --- |
| `start_reading_session` | 英文を入力した画面を開く。これだけではAPIを呼ばない |
| `get_reading_result` | 指定した評価の状態・結果を取得する。生成や再送はしない |
| `cancel_reading_session` | 指定した評価を取り消す。送信済みの場合、サーバー側の状態は確約しない |

同時に進められる評価は1件である。次の評価を開始すると前の結果画面を閉じる。最新16件の結果はMCPプロセスが動いている間だけ取得でき、Codexの再起動などで失われる。Codexへ渡った結果は、Codex側の会話保存の対象になり得る。

## 開発・検証

本体から独立したCargo packageである。リポジトリのルートから実行する。

```text
cargo test --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --all-targets --locked
cargo build --manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio --release --locked
```

自動試験では合成WAV・架空の英文・偽キー・模擬応答を使用する。モック成功は、利用者のキーで接続できることや、実モデルが正しく発音を評価できることを保証しない。開発仕様・設計・実行証拠は [QWEN-AUDIO-001](../../tasks/QWEN-AUDIO-001/) に保存する。

参考：[Qwen-Omni](https://www.alibabacloud.com/help/en/model-studio/qwen-omni)、[Tokyo接続先](https://www.alibabacloud.com/help/en/model-studio/compatibility-of-openai-with-dashscope)、[Codex MCP](https://learn.chatgpt.com/docs/extend/mcp)。
