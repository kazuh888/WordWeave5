# QWEN-FORMATS-001 最終検証 runner

2026-10-03。担当はテスト実施のみ。計画 `plan.md` U5 の起動指定 `gpt-6.1-sol/medium` を親から明示受領した。実行model/effort metadataと開始/終了token counterは未取得であり推定しない。AGENTS.md、development-team.md、wordweave-change全文とKPI、plan/tests/resultsを読了した。Skillは検証・報告工程だけを適用した。ソース・テスト・期待値・設定の編集、再委任、API送信、公開は行っていない。

## 対象と引継ぎ

- 基点: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` と既存未コミット差分。QF-01～07 / QF-AC-001～009をplan/spec/design/testsから引継いだ。
- `revision-final.log` は開始時のsrc/tests/tools/qwen-audio（target除外）とroot Cargo.toml/lockのSHA256台帳である。未追跡ソース/fixtureも含む。途中照合で差異は親による `src/main.rs` の警告文字列折返しのみであった。挙動変更なしとの親報告に基づきルート全テストは再実行せず、releaseは更新する。
- 最終入力台帳 `revision-end-final.log` SHA256 = `24DCD5698F5C0A91968646F4BB31223A2A956606DC919E358ED8A6946FCD5333`。tracked差分 `tracked-diff-final.log` SHA256 = `07F4F2232BF00AEF39ACEFB38F00E8B74BF25E95113DA6B80F48BF9C274AF781`。差分ログ単独では未追跡ファイルを識別できないため、台帳と組み合わせる。
- 親の今回ログ `root-all-targets.log` は10 suites、418 passed / 0 failed / 0 ignored、`root-release.log` はrelease Finishedを照合した。exit0は親の実行handoffで確認した。root focused25、UI focused16の成功ログも照合した。以前の作業のログを新規実機成功として転用していない。
- 親から独立レビューP1/P2=0を受領。native画面は親が合成fixtureの本体通常/狭幅3枚・単独2枚を観察済みとの引継ぎであり、本runnerが実機観察した証拠ではない。

## 新規実行結果

全Cargo実行は `--locked`。単独Cargoは `--manifest-path tools/qwen-audio/Cargo.toml --target-dir target/qwen-audio` を使用した。通常execはACL helper初期化で失敗し、以後承認付き実行を用いた。失敗を回避するためのテスト/期待値/設定変更はない。

| 検査 | exit / 件数 | 新規ログ |
| --- | --- | --- |
| 単独 `cargo test --all-targets --locked` | 101。provider / integration_credentialsリンクがLNK1102メモリ不足。テスト成功扱いせず | package-tests-final.log |
| 同gate `-j 1` 再試行 | 0。13 suites、113 passed / 0 failed / 0 ignored。形式13件に60秒境界/AMR-WB container含む | package-tests-serial-final.log |
| 単独 release `cargo build --release --locked -j 1` | 0 | package-release-final.log |
| core-only `cargo check --no-default-features --lib --locked -j 1` | 0 | core-check-final.log |
| core-only `cargo test --no-default-features --lib --locked -j 1` | 0。8 passed / 0 failed / 0 ignored | core-lib-final.log |
| core-only `cargo test --no-default-features --test integration_reuse --locked -j 1` | 0。3 passed / 0 failed / 0 ignored | core-reuse-final.log |
| core-only `cargo test --no-default-features --test audio_formats --locked -j 1` | 0。12 passed / 0 failed / 0 ignored | core-formats-final.log |
| 本体 `cargo build --release --locked --bin wordweave5 -j 1` | 0。main.rs整形後成果物一致用 | root-release-final.log |
| core-only `cargo tree --no-default-features --locked` | 0。eframe/egui/rfd/image/uuid/windows 0.56なし。native TLSに由来するWindows依存は残る | core-tree-final.log |
| `git diff --check` | 0 | diff-check-final.log |
| 変更Rust16ファイル `rustfmt --check --edition 2021 --config skip_children=true` | 1。main.rs 3/9/43とvisual_check.rsの既存広範囲整形差分。他14ファイル差分なし | rustfmt-final.log |
| 親のmain.rs新規警告行折返し後、同checkをmainのみ再実行 | 1。3/9の既存module順序だけ残り、新規警告行差分は解消 | main-rustfmt-final.log |

整形検査の失敗は親へ報告済みである。既存差分を一括整形して緑にはしていない。単独release前に `Get-Process qwen-audio` 0件を確認した。ユーザーアプリのkillは行っていない。

本runner新規テスト成功は構成別合計136件（113+8+3+12）、failed=0、ignored=0である。同じケースのfeature別実行を含むため136種類の独立ケースとはしない。初回リンク失敗は実行失敗1件として別記した。成功gateの無理由再実行はない。

単独release成果物: `target/qwen-audio/release/qwen-audio.exe`、7,901,696 bytes、2026-10-03T12:40:32.2023032+09:00、SHA256 `560D65F14BC8AE1486EBA4BED2B3BEC505A98B6F6E588C23EFEAC9ABA7D7D063`。

本体release成果物: `target/release/wordweave5.exe`、12,567,552 bytes、2026-10-03T12:52:27.5220986+09:00、SHA256 `D859AACB60B0B5060E5F2579ED56DFD99015EDC37C86DA2BC8DB155834AD9C56`。本体更新開始12:46:24、直列compile/linkを経て12:52:28終了、Cargo所要6m03s。開始前 `Get-Process wordweave5` 0件を確認した。最後の台帳照合12:52:44で変更0件である。

親の追加handoff: 最終文書8件/ローカルリンク44件missing0、git diff --check exit0。これは親の実行証拠であり、本runnerが繰り返した結果ではない。

## 未検証と完了境界

自動検証は元bytes/wire一致、拒否入力、取消し/再選択、上限、backendのprivate試験等の証拠であり、今回の実API受理・認証成功・原音出力・マイク・ペン・TTS・IME成功を意味しない。親handoffの残る実機受入は原音再生/停止/閉じる、IME/keyboard、実ディスク削除失敗時native警告、実API全codec受理、配布先FFmpeg/ffprobeである。過去実録音受入と今回の多形式受入を区別する。

12:52:44（最終応答前）で実装・自動検証の委任範囲は完了した。全必須Cargo gateは成功、整形検査は既存差分によりexit1のままであり完全な整形合格とはしない。残るnative受入は上記のとおりである。開始/終了token counter・親子包含関係は未取得。初回通常execのACL helper障害とリンクメモリ不足を含む実行失敗を親へ返し、検証基準を下げていない。
