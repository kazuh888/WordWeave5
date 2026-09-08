# 0.3.3：終了状態の取得とWindowsバッチテストの診断

## 今回の2件の失敗

1. `early_exit_reports_stage_and_code_without_stderr_secrets`
   stdoutとstderrが閉じた時点で、Windowsのプロセス終了コードが確定していない競合。即座にtry_waitを1回だけ呼ぶ実装を、最大2秒・10ms間隔の終了待ちに変更した。キャンセル時は待機を打ち切る。
2. `windows_batch_wrapper_preserves_piped_rpc_with_unicode_and_spaces`
   原因未確定。診断要約を返却エラーへ添付し、テストでも読めるようにした。コピーした模擬EXEがどこまで進んだかをテスト専用の一時ファイルへ記録する。ユーザーのパス・本文・認証情報は記録しない。

これは1件目への修正と、2件目の切り分け改善を含むソース版である。両方の現象が解消したとはまだ言えない。実CodexとChatGPTはこれらのテストに使用しないため、ログインではテスト失敗を解消できない。

## 実行

新しいフォルダーに展開し、Cargo.tomlのあるwordweave5フォルダーで次を実行する。

```powershell
cargo test --test codex_process --locked -- --test-threads=1
```

失敗した場合はfailuresの内容を共有する。バッチテストにはmock stageと診断要約が追加される。
標準出力・標準エラーの生データは共有不要。10件のテストを減らしたり、失敗を無視する設定はしていない。

上記が合格した後：

```powershell
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
```

通常バイナリ側の0 testsは問題ない。アプリの保存形式と教材は変更していない。

## 制約

作成環境にはRust/Windows実行環境がなく、修正版は未コンパイル・未実行である。
stderrは引き続き分類のみで、未知のエラーやWindowsの文字コードによっては分類できない。
認証状態の常時表示はこの版の変更範囲に含めていない。
