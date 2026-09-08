# 0.3.5：Voltaを直接起動

0.3.4のテストでもcmd.exeは模擬EXEへ到達しなかった。利用環境の `F:\Tools\Volta\bin\codex.cmd` は、内部で `volta run codex ...` を呼ぶ中継ファイルである。

0.3.5では、選択したファイル名がcodex.cmd/codex.batで、同じフォルダーにvolta.exeが存在する場合、次の構成で起動する。

```text
F:\Tools\Volta\bin\volta.exe run codex app-server
```

cmd.exeとcodex.cmdの引用符・文字コード解釈を通らないため、日本語や空白を含む作業ディレクトリでも標準入出力を直接接続できる。標準エラーの分類、終了コード取得、未認証判定は維持する。

新しいフォルダーへ展開し、Cargo.tomlのあるwordweave5フォルダーで実行する。

```powershell
cargo test --test codex_process --locked -- --test-threads=1
```

成功時は10件すべてがokとなり、最後のテスト名は次になる。

```text
windows_volta_shim_preserves_piped_rpc_with_unicode_and_spaces
```

10件成功後：

```powershell
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
```

この修正版はWindowsで未実行である。失敗した場合はfailures以下を共有する。実CodexやChatGPTログインは通信テストに使用しない。
