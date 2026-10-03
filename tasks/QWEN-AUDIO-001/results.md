# QWEN-AUDIO-001 検証・納品記録

## 既存WordWeave5の回帰

2026-10-02、親がリポジトリルートで実行。今回rootソース/manifestには変更していない。既存利用者差分を保持した作業ツリーの結果である。

- `cargo test --all-targets --locked`: exit 0、377 passed、0 failed、0 ignored（9 test groups）。`root-test.log`。
- `cargo build --release --locked --bin wordweave5`: exit 0。`root-build.log`。既存unused警告あり。
- 実行前にwordweave5プロセスが起動していないことを確認し、強制終了はしていない。

## 独立音声評価ツール

`tools/qwen-audio/` に独立したCargo packageを実装した。配布EXEは `target/qwen-audio/release/qwen-audio.exe`。本体のCodex認証・教材・学習記録は変更していない。通常起動は日本語GUI、`--mcp` はローカルSTDIOサーバーである。設定・使用手順は[README](../../tools/qwen-audio/README.md)にある。

- 独立作者の全81 tests成功（lib4、integration77）。資格情報blob4件とMCP追加5件を含む。詳細は[tests.md](tests.md)。最終runnerの固定差分結果は[verification.md](verification.md)へ別記する。
- 配布EXEの最終SHA-256: `9B36D56936D39852F0A24C66B153D5BBA475514F8FF65C5DF17F4041DEBB64CC`。
- 秘密は専用Windows資格情報へ単一保存、音声/英文/結果はメモリのみ。Tokyo以外の宛先・redirect・自動再送は不可。毎回GUIで確認した音声/英文への明示送信が必要である。
- 親統合修正: テストincludeの相対pathを1階層訂正、native起動で文字サイズが初期値へ戻る現象を両テーマへのstyle適用で修正、provider iteratorの整形。試験期待値は変更していない。

## Windows画面・実プロセス

- debug専用の合成fixtureでinput/result/running/errorを撮影し、全プロセスexit 0。800px/倍率1と360px/倍率1.5を目視確認した。Body/Button16、Heading22、結果/失敗原因の先頭表示、狭幅の取消し可視、日本語の折返しを確認した。証拠: `ui-input.png`、`ui-result.png`、`ui-running-narrow.png`、`ui-error-narrow.png`。入力全文と送信操作は縦スクロールで到達する。
- 最初の画面では結果が下へ隠れ、次の確認ではnativeの文字サイズが初期値のままであった。最終撮影で両修正の反映を確認した。撮影成功だけを表示合格とは扱っていない。
- releaseをNodeの明示stdin/stdout pipeで起動し、initialize→標準metadata付きstart→実GUI子1件→cancel→親EOFと子回収を確認した。別起動でstart→親強制終了→子回収も確認した。証拠 `native-process.log`。合成英文のみ、WAV選択・送信操作なし。自分が起動したプロセス以外を終了していない。
- 上記実プロセス確認後のrelease差分はテーマstyle適用1行、整形、test専用include訂正であり、MCP/Job Objectの実行経路は不変である。最終EXEのMCP JSON-only/EOF/fixture拒否はrunnerが別確認する。
- PowerShellの通常パイプはGUI subsystemのEXE終了・標準出力を待機/捕捉しない場合がある。これは初回smoke起動手法の不成立であり、明示した標準handleで成功したMCP連携と区別する。console subsystemへの仕様変更はしていない。

## レビュー

U0の仕様/設計レビュー合格後に実装した。最終実装レビューの2件（標準MCP metadata拒否、不正blobの無条件メモリ書込み）は修正し、独立回帰を追加した。最終判定は[review.md](review.md)へ記録する。

## 実機受入の境界

実キーの設定、利用者の録音、課金APIは使用していない。Tokyo実接続、発音/強弱/リズムの評価品質、実際の資格情報保存・OS保護、IME/キーボード/スクリーンリーダーの全操作、Codexへの実MCP登録は未確認である。音声評価の成功は構造検証の成功だけから推定しない。MCP設定例のみ納品し、利用者のCodex設定は変更していない。commit/pushはしていない。
