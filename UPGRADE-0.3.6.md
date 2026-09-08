# 0.3.6：Volta本体が別フォルダーにある環境への対応

0.3.5の実機ログでは、Voltaの直接起動が選ばれず、cmd.exe経由でinitializeの応答前に終了していた。同じフォルダーのvolta.exeだけを探す実装を修正した。

## 起動方法

設定には、これまでの `F:\Tools\Volta\bin\codex.cmd` を指定したままでよい。volta.exe自体をCodex実行ファイルとして選択する必要はない。

1. codex.cmd/codex.batと同じフォルダーのvolta.exeを探す。
2. 見つからず、ファイル内容が利用環境で確認された `@echo off` / `volta run %~n0 %*` の2行である場合、WordWeaveが継承したPATHからvolta.exeを探す。行頭末尾の空白・空行・BOM・英字の大小は許容する。
3. 見つかったEXEを `run`、`codex`、`app-server` の3引数で直接起動する。
4. Volta用ファイルと確認できたのに本体が見つからない場合は、起動前に探索失敗を表示する。

PATHは絶対パスの項目を検索する。空の項目・相対パスは検索しない。他の内容のcmd/batは従来の経路であり、その一般的なバッチ起動の問題まで解消したとは扱わない。

## 更新と確認

旧版を終了し、このZIPを新しいフォルダーへ展開する。Cargo.tomlがあるwordweave5フォルダーで実行する。

```powershell
cargo test --all-targets --locked
```

Windowsではcodex_processの通信テストは11件となる。従来の10件に次を追加した。

```text
windows_volta_from_path_preserves_piped_rpc_with_separate_directories
```

このテストは別プロセスだけのPATHを変更するため、`--test-threads=1`は必須ではない。tests/support/mock_codex.rsの単体テストが0件と表示されることは正常である。模擬EXEは通信テストから起動する。

全体テストに失敗がなければ、次を実行する。

```powershell
cargo build --release --locked --bin wordweave5
.\target\release\wordweave5.exe
```

「設定」→「接続・ChatGPT認証を確認」で接続を確認してから、「未処理の語から再開」を実行する。別フォルダーのVolta本体をPATHから発見した場合、診断情報には次が表示される。

```text
WordWeave 0.3.6
launch: PATH volta.exe run codex app-server
```

これは起動方法の記録であり、ログインや教材生成の成功を意味するものではない。失敗時は、新しい診断情報を共有する。

## Voltaが見つからない場合

通常のcmdで `where volta.exe` を実行する。そのフォルダーがPATHに含まれる状態でWordWeaveを起動し直す。環境変数を変更した場合は、起動元のVisual Studioやターミナルも起動し直す。cmdから起動できる場合は、同じcmdでWordWeaveのEXEを起動すると、そのPATHを引き継げる。

## 検証の限界

この修正版はソース・TOML・配布ZIPの静的確認のみである。作業環境にRustコンパイラとWindows実行環境がないため、コンパイル・テスト実行・実VoltaおよびChatGPT接続は未確認である。模擬Voltaのテストが通っても、実Voltaのツール選択やChatGPT認証を確認したことにはならない。

依存ライブラリの追加はない。教材および学習記録の形式に変更はない。
