# WordWeave 5 0.3.2：起動改善と診断情報

## 現在判明していること

`codex login status` の `Not logged in` は、そのCMD環境で未認証であることを示す。ただし、WordWeaveの接続終了の原因も未認証とは限らない。
0.3.1のバッチ起動ではcmd.exe用のコマンド文字列を通常の引数として渡していた。0.3.2ではWindowsのraw_argに変更し、二重の引用符エスケープを避けた。パス内のシェル展開文字は拒否する。この修正が今回の原因を解消するかはWindows実機での確認が必要である。

## 更新

1. 既存アプリを終了し、教材・学習記録をバックアップする。
2. このZIPを新しいフォルダーに展開する。
3. Cargo.tomlのあるwordweave5フォルダーでbuild.cmdを実行する。
   手動の場合は次を順に実行し、成功後だけ次に進む。

```powershell
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
```

4. 新しい `target\release\wordweave5.exe` を起動する。教材・保存形式は変更していない。

## 認証と接続確認

WordWeaveと同じWindowsユーザーでCMDを開く。

```cmd
codex login --device-auth
```

表示されたURLとコードを使いブラウザーで認証する。完了後：

```cmd
codex login status
```

デバイス認証にはChatGPTのセキュリティ設定/組織の許可が必要な場合がある。通常のブラウザー認証が使える環境では `codex login` でもよい。認証URL・ワンタイムコード・auth.jsonを他人へ送らない。

設定画面の実行ファイルを `F:\Tools\Volta\bin\codex.cmd` に設定し、「接続・ChatGPT認証を確認」を押す。ファイル選択もexe/cmd/batに対応した。
接続成功後に「語彙を追加」から再開する。ログイン操作はこのアプリから自動実行しない。

## 再発した場合

設定画面で「Codex診断情報（コピー・保存・ChatGPTで相談）」を開く。

- 発生段階：実行ファイル探索、起動、initialize、account/read、thread/start、turn/startなど。
- 終了コード：起動したプロセスのコード。まだ実行中ならその旨を表示する。
- 標準出力：RPC応答・通知・解析失敗の種別のみ。教材本文や認証データを記録しない。
- 標準エラー：最大4096バイトずつ読み、既知エラーの分類だけを記録する。元の文章は保存しない。

表示を確認して「確認した診断情報をコピー」または「診断情報を保存」を押す。
この会話に貼り付けるか、「ChatGPTを開く（手動貼り付け）」でブラウザーを開いて貼り付ける。
分析依頼文も含まれる。ブラウザーへの自動貼り付け・自動送信はしない。

記録は直近1回・最大128件であり、自動ファイル保存はしない。アプリを終了する前に必要な記録をコピー/保存すること。
未知のエラーは「未分類」となり、診断情報だけでは原因を断定できないことがある。その場合も生の認証情報を共有しない。

## 検証上の制約

今回の修正版は静的確認のみ。Rust環境が利用できないため、コンパイル・テスト実行・Windows/Volta実機・ChatGPT実接続は未確認である。
Windowsの通信テストは10件を用意した（実サービスではなく模擬Codexを使用）。通常バイナリ側の0 tests表示は問題ない。

参考：[OpenAI App Server](https://learn.chatgpt.com/docs/app-server)、[OpenAI認証手順](https://learn.chatgpt.com/docs/auth)。
