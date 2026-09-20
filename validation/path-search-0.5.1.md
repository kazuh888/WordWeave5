# 0.5.1 PATH探索の検証（2026-09-20）

## 対象

未コミットの既存変更を保持し、Codex/VoltaのPATH探索だけを補完する。起動時PATH→保存システムPATH→保存実行ユーザーPATHの順序と、明示パス欠損時の暗黙代替禁止を契約とする。

Windows APIの文字列取得・REG_EXPAND_SZ展開は [MicrosoftのRegGetValueW仕様](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-reggetvaluew) と既存windows 0.56.0の定義を確認した。既存依存のRegistry機能を有効化し、依存バージョンは変更していない。Cargo.lockはCargoが本体0.5.1へ更新した。

## 検証経過

- 再現テスト: システムPATH未探索、ユーザーPATH未探索、補完順序、空・相対PATH除外の4件が修正前に失敗した。[REDログ](path-search-red.txt)。大文字小文字の重複テストはこの段階で成功していたため、REDの証拠には数えない。
- `cargo check --offline --lib`: 終了0。[ログ](path-search-0.5.1-check.txt)。
- `cargo test --all-targets --locked`: **167件成功、失敗0・無視0、終了0**。[ログ](path-search-0.5.1-tests.txt)。lib104、Windowsアプリ27、会話ごみ箱2、模擬通信31、設定互換2、診断1。補助bin/example自体は0件である。
- `cargo build --release --locked --bin wordweave5 --example codex_path_probe`: **成功、終了0**。[ログ](path-search-0.5.1-release.txt)。既存の未使用互換メソッド`Recorder::finish`と`Speaker::say`の警告2件は継続する。
- `git diff --check`: 終了0。
- 独立担当がPATH探索・レジストリAPI・Volta直接起動・認証境界・テストを読み取りレビューし、重大な指摘なし。

## 実環境での探索確認

保存されたユーザーPATHを読み取り、利用者が提示した2ファイルが探索対象に含まれることを確認した。システムPATHにはこの確認時点でCodex/Voltaの一致候補はなかった（システムPATHの探索処理自体は独立したfixtureテストで確認）。

新ソースでビルドした`examples/codex_path_probe.rs`を使用した。これは探索のみを行い、発見したCodexを起動しない。[実測出力](path-search-0.5.1-probe.json)。

- 検証用プロセスのPATHを空にしても、保存ユーザーPATHから`C:\Users\安井　一広\AppData\Local\Programs\OpenAI\Codex\bin\codex.exe`を検出、終了0。
- 上記EXEの明示指定も終了0。canonicalize先は`.codex/packages/standalone/releases/0.135.0-x86_64-pc-windows-msvc/bin/codex.exe`であった。これは実行時の版照会ではない。
- `F:\Tools\Volta\bin\codex.cmd`の明示指定も終了0。Voltaの実AI起動は行わず、別ディレクトリ・日本語パスでのパイプ通信は模擬サーバー試験で確認した。

実験で空にしたのは検証シェルのプロセス環境だけで、finallyで元に戻した。Windowsの保存環境変数・利用者設定は変更していない。

## 成果物

- `F:\Kazuhiro\GitHub\WordWeave5\target\release\wordweave5.exe`、0.5.1。
- サイズ: 10,107,392 bytes。更新時刻: 2026-09-20 23:54:40（日本時間）。
- SHA-256: `EF578BF0F25074404E8178CE5D4F2D4395BCB614B528DA1FBD84FE54438C9539`。
- [検証対象50ファイルの指紋](path-search-0.5.1-source-sha256.txt)をビルド後に再照合し、全件一致した。
- 通常sandboxのACL初期化エラーがあるため承認付き実行を使用した。ACL変更、コミット、pushはしていない。

## 保護範囲・未検証

実教材・学習記録・認証情報の読み書き、レジストリ変更、永続環境変数変更、CLIのインストール・アップグレード、実AI生成は行わない。旧0.5.0の成功を本変更の合格証拠として流用しない。

GUIの物理操作、実Codexでの認証・effort指定生成、実マイク・ペン・TTS、非Windows実行は本変更では未検証である。
