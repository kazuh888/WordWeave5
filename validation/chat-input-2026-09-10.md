# チャット入力修正とCodex互換性調査

2026-09-10。今回の入力修正後のソースで検証した。以前のUI検証を今回の実機合格証拠にはしない。

## 変更

- チャット入力のEnterによる改行を無効化。
- 入力欄にフォーカスがあるときのCtrl+Enter送信を追加。IME変換中・確定イベントと同一フレーム・長押しリピートでは送信しない。処理中や空の入力は従来と同じ条件で送信不可。
- 入力欄上辺の説明文を削除。8px高の文字なしドラッグ領域と細線を残して高さ変更を維持。
- `CHAT_UI.md` のキー操作説明を更新。

## 実行した検証

- `cargo test --all-targets --locked`：終了コード0、57＋3＋15＝75件合格。
- 新規3テスト：Enterで日本語の文字列へ改行を挿入しない、フォーカス付きCtrl+Enterのみ送信、IME確定と送信を分離。
- `cargo build --release --locked --bin wordweave5`：終了コード0。
- `git diff --check`：終了コード0。

実際のWindows日本語IMEを使った手動操作と、新しい細線のドラッグは今回未検証。IMEイベントの自動テストとeguiの入力テストを実行した範囲に限る。

## エラーの切り分け

利用者提供の400エラーは、gpt-6-astraがより新しいCodexを要求しているというサーバーの拒否である。「処理の2.」という質問形式が原因だという証拠はない。

ローカルの実測：

- WordWeaveの設定は `codex_path=codex`。モデル設定は空欄（Codex側へ委譲）。
- PATHの先頭候補は `F:/Tools/Volta/bin/codex.cmd`。`codex --version` は `codex-cli 0.144.4`。
- 別の既存実行ファイル `C:/Users/安井　一広/AppData/Local/OpenAI/Codex/bin/7ac07f4ce733f89a/codex.exe` は `codex-cli 0.153.4`。

WordWeaveの実行ファイル設定だけを後者へ切り替える案について承認を求めた。現時点で設定ファイル、PATH、Volta、モデル設定、認証情報は変更していない。0.153.4で実際の生成が通るかは未検証であり、問題解消済みとはしていない。

参照した公式資料：[Codex CLI](https://learn.chatgpt.com/docs/codex/cli)。インストール・更新とChatGPT認証の案内を確認した。今回のモデルに必要な最小CLI版は、このページからは確定できない。
