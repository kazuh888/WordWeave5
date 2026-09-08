# 0.3.4：Windowsバッチ起動の修正

0.3.3のWindowsテストにより、`cmd.exe`は模擬EXEを起動する前に終了コード1で停止したことが分かった。認証やJSON-RPCの問題ではない。

0.3.4では、引用符付きバッチパスを`/C`へ直接渡す処理を、次の意味になる`CALL`方式へ変更した。

```cmd
cmd.exe /D /V:OFF /S /C "call "<codex.cmdの絶対パス>" app-server"
```

新しいフォルダーへ展開し、Cargo.tomlのあるwordweave5フォルダーで次を実行する。

```powershell
cargo test --test codex_process --locked -- --test-threads=1
```

10件すべて成功した後、全体テストとビルドを行う。

```powershell
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
```

今回の修正版はWindowsで未実行である。再びバッチテストだけが失敗した場合は、failures以下を共有する。診断段階と終了コードを維持している。
