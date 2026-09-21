# Windows CI

設定: [rust.yml](../../../.github/workflows/rust.yml)。GitHubが認識するディレクトリ名は `.github/workflows` であり、`.githob` ではない。

## 検証内容と起動

- mainへのpush、mainを宛先とするPR、Actions画面の手動実行で起動する。
- Windows runnerでstable Rustを用い、`cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を順番に実行する。テストに失敗した場合はreleaseビルドへ進まない。
- Cargoの取得済み依存だけをキャッシュする。認証情報は投入せず、Codex接続はモックで検証する。実教材・学習記録も投入しない。
- 同じブランチへの新しい実行は古い実行を中断する。40分でタイムアウトし、GitHub Actionsの各stepログから失敗箇所を確認する。
- トークン計測・実Codex認証・IME・音声・手書きの受入をCI成功で代用しない。Git除外されたホーム画面試作もCIの対象外である。

既存Ubuntu設定ではWindows固有コードやテストを確認できなかったため、Windowsへ変更した。全体整形は既存コードの広範囲な変更になることを確認し、2026-09-21の利用者の選択により別作業とした。現時点の必須チェックに `cargo fmt` は含めない。

## 初回の確認手順

1. `codex/windows-ci` ブランチからmain宛てのPRを作成し、初回CIを確認する。2026-09-21に利用者がこのcommit・push・PR作成を承認した。手動実行の一覧に出すには、workflowをデフォルトブランチへ反映する必要がある。
2. GitHubのActionsで **Windows Rust CI** を開き、対象commit、テスト、releaseビルドの結果を確認する。ローカル成功とは別の検証結果として記録する。
3. PRの必須チェックにする場合は、初回実行後にリポジトリのruleset/branch protectionへ **Windows tests and release build** を登録する。workflow追加だけではマージが必ず禁止されるわけではない。この設定は今回変更していない。

2026-09-21、commit `cc91129` の本体ソースを変更せず、ローカルWindowsで上記全対象テスト198件成功・releaseビルド成功を確認した。未使用メソッド `finish` / `say` の警告2件は残る。ログは今回の試作フォルダーに `ci-tests.log` / `ci-build.log` として保存した。

GitHub上の合否は初回PRのChecksで対象commitと併せて確認する。この文書のローカル成功をリモート成功の証拠として扱わない。ローカルのRustとGitHubのstable Rustは同一バージョンとは限らない。Issue運用とGitHub試行の削減効果は別途評価する。

公式仕様: [workflow構文とイベント](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)、[手動実行](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)。
