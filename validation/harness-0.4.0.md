# 0.4.0 Windows Harness 検証記録

日付：2026-09-17。作業ツリーの未コミット変更を対象とする。旧validationログは今回の合格証拠に使用しない。

最終結果：Windowsの全119件テストとreleaseビルドに成功した。実マイク・ペン・実Codex媒体入力は未検証である。

## 環境とデータ保護

- Windows、`x86_64-pc-windows-msvc`、rustc 1.98.1、cargo 1.98.1。
- Cargo.lockは依存版を維持し、wordweave5だけ0.3.8→0.4.0へ変更。Cargoによる更新。
- 各試験は専用一時フォルダー・模擬app-serverを使用。実学習データ・実Codex認証・マイクを使用していない。
- 通常sandboxは `helper_unknown_error: apply deny-read ACLs` で開始前に失敗する。承認付き実行でテスト・標準apply_patchを使用した。ACL変更なし。
- 一度、最終テスト／ビルドの起動が承認機構の利用上限で拒否された。処理は開始しておらず、ビルド失敗の証拠ではない。

## 実行結果

| コマンド・時点 | 結果 | 証拠 |
| --- | --- | --- |
| `cargo check --offline --lib`（0.4.0） | 成功 | 当該実行の終了コード0を確認 |
| `cargo test --all-targets --locked`（最終レビュー修正前） | 114件成功：lib83、GUI10、通信21。終了0 | [初回ログ](harness-0.4.0-tests.txt) |
| 固定背景だけで終了する回帰試験 | 修正前に期待どおり失敗（終了101）、修正後の全テストで成功 | [修正前ログ](harness-0.4.0-close-red.txt) |
| レビュー一次修正後 `cargo test --all-targets --locked` | 118件成功（lib85、GUI11、通信22）、終了0 | [一次修正ログ](harness-0.4.0-tests-final.txt) |
| レビュー一次修正後 `cargo build --release --locked --bin wordweave5` | 成功、終了0 | [一次ビルドログ](harness-0.4.0-release.txt) |
| 未固定原文の保護追加直後 | GUI試験1件失敗。スクロール前の初期表示だけで全操作を検査していた | [失敗ログ](harness-0.4.0-verified-tests.txt) |
| **最終ソース `cargo test --all-targets --locked`** | **119件成功：lib85、GUI12、通信22。失敗0・無効化0、終了0** | [最終テスト](harness-0.4.0-acceptance-tests.txt) |
| **最終ソース `cargo build --release --locked --bin wordweave5`** | **成功、終了0** | [最終ビルド](harness-0.4.0-acceptance-release.txt) |

初回114件成功後に、phase未指定メッセージの順序維持、大きな媒体を含むthread/readの受信上限、assets配下への自己バックアップ拒否を修正した。初回ログはこれらの修正の合格証拠ではない。

最後のGUI試験では、描画された退避ボタンの位置へポインターを移し、ホイール入力で下へスクロールして破棄操作への到達も検査した。検査項目を削除・無効化して合格扱いにしていない。`git diff --check` も成功した。

## 成果物の特定

- 実行ファイル：`F:\Kazuhiro\GitHub\WordWeave5\target\release\wordweave5.exe`
- サイズ：9,886,720 bytes。更新日時：2026-09-17 17:58:59（日本時間）。
- SHA-256：`7FD4703ABF3682B77E7A06C3FF7FF18984CC301CD262A607E4E6DA88BD6EC4D4`
- 検証したCargo.toml/Cargo.lock、src、tests、dataの指紋：[source-sha256](harness-0.4.0-source-sha256.txt)。実ユーザーの保存フォルダーの指紋ではない。

## 検査した内容

原本ハッシュ・サイズ・形式・参照パス、媒体の欠落／破損、旧データ読み込み、引用の完全一致、選択外の根拠拒否、追加時の復習状態保護、訂正・競合・二重登録、2ファイル更新の途中復旧・外部編集保護、復元前の未保存下書き退避、保存容量境界、結果不明／失敗／サーバー中断、別turnを使わない再取得、APIキー認証拒否、Volta直接起動とPATH探索、返却モデル／effort未取得時の非推測を含む。

GUI試験はegui Contextによるheadless描画と入力イベントであり、Windowsの物理入力・画面全体の見た目の評価ではない。固定画像はWindows GDIによるオフスクリーン生成を試験する。

## 最終レビュー

保存・通信と画面を別に読み取りレビューした。応答順序、大媒体の回収、自己バックアップ先、並列fixtureの一意性は限定再レビューでも修正を確認し、118件のテストで合格した。固定背景だけの終了保護もRED→GREENを確認した。最後に未固定原文の終了保護・TXT退避・二段階破棄を追加し、失敗時の原文保持と成功後の終了可能性を最終119件の中で確認した。この指摘の限定再レビューもPASSである。未固定原文について修正前REDを実行したとは扱わない。

## 未検証

- CLI 0.153.4による今回の音声・画像生成、媒体付きthread/read、通信切断後の回収。以前のgpt-6-astraテキスト生成成功は利用者報告として区別する。
- 実マイク・原音再生・TTS停止・Windowsペンの操作と画面UX。
- 実教材を使うバックアップ／復元。自動試験は一時データのみ。
- 電源断・ストレージ障害の全条件、NTFS以外の媒体、非Windows動作。Androidは対象外。

手動確認は [UPGRADE-0.4.0.md](../UPGRADE-0.4.0.md) を使用する。
