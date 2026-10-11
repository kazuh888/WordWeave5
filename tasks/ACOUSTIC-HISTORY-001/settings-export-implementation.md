# I2d-S 設定export 本体実装

2026-10-12 / implementer。入力は承認済みAH-AC04/16、design-i2.md §5–6のI2-T08/T08b、implementation.mdのI2d-Sと2026-10-12接合照合、settings-export-tests.mdである。H最終Post Closedと作者の所有返却を親から受領した。計画者指定・起動指定は `gpt-6-astra/high`、実行metadataと開始/終了usage counterは未取得である。wordweave-changeとwordweave-egui-uiを全文読了し、2file本体の保存出口の接合に適用した。

完了境界は5入口・4UI出口の接続、新25件＋既存9件の自己検証と証拠引継ぎである。独立runner/Post、I10全体検証、製品EXE更新、実機受入は含まない。実学習データ・live AI・commit/pushは使用していない。

## 所有・変更

- `src/app/settings_ui.rs`：`write_settings_export` は実Storage.dirを優先し、Storageなしだけ副作用なしresolverを使用する。TSV/メモリProgress pretty JSON/診断reportの生成・既存通知を3つの本体メソッドへ抽出し、各pickerから呼ぶ。JSONのcredit_time→picker→snapshotの順、settings editor draftの非混入を維持する。
- `src/app/control_settings.rs`：`diagnostic_export_result` は共通helperを使用し、Settings Started→Export Completed又はFailed/Ioを維持する。不変借用中はResultのみ返し、通知は従来どおりclosureの後である。
- 両fileのRED-only scaffold implだけを除去した。control_settingsのstub専用 `path::Path` import除去は親の明示許可を得た。期待値・試験本体は変更していない。rustfmtによりJSONの既存UI closureも整形されたが、復元動作の変更はない。

直接atomic_writeは両fileに残っていない。新依存・公開API・新しい保存方針・描画変更はない。所有外の文書更新は他者の変更として保持した。

## 対象固定と検証

基準HEADは `56b346280cbf0d6a36617f3556436068ff7e2fe4`。開始時の2file SHA-256は作者の所有返却値と一致した。確定sourceは以下である。

| file | SHA-256 |
| --- | --- |
| settings_ui.rs | 5575F9E5B5C138380DB273354BF3E12C5D87307F2A4237F254508D8B609871C3 |
| control_settings.rs | F7A3FB0777B6E5791F7F4591953ABA0C0F2B16803C224460E81886D09DEF35C5 |

`cargo test --locked --bin wordweave5 i2_export_settings --no-run` の1回build後、同EXEの実在名一覧と3filterを実行する。作者と同じworkspace専用TEMP/TMP `tasks/ACOUSTIC-HISTORY-001/logs/settings-export-temp` とHidden起動を使用する。通常sandbox TEMPで初期Storage.saveに失敗した作者の環境差を引き継ぎ、検証標準や恒久harnessを変更しない。build rawは `logs/settings-export-impl-build.log`。

buildはexit0、1分56秒。既存unused等5警告とincremental cache hardlinkからcopyへの警告を記録した。fixed EXEは `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe`、実行前後SHA-256はいずれも `8F680C9C10633E7A609A0B77C0702F39954D811FD4E04E9F2E6F31F2F00D118B`。listはPID31276/exit0、305件であり、新25件・既存5件・既存4件の実在数をassertした（`logs/settings-export-impl-list.log`）。

| filter | PID / native exit | 実行結果 | 所要 |
| --- | --- | --- | --- |
| i2_export_settings | 33464 / 101 | 24 passed / 1 failed / 0 ignored / 280 filtered | 2.59秒 |
| app::settings_edit::tests:: | 14528 / 0 | 5 passed / 0 failed / 0 ignored / 300 filtered | 0.93秒 |
| app::settings_ui::tests:: | 29588 / 0 | 4 passed / 0 failed / 0 ignored / 301 filtered | 0.64秒 |

3filterはいずれも `--nocapture`、外側deadline60秒、timeout0である。rawは `logs/settings-export-impl-{new,edit,ui}.{stdout,stderr}.log`。新fixtureはcreated25/removed25/retained0、3filter終了時に専用TEMPの子要素0。環境変数は当該shell内のtry/finallyで元へ戻した。sourceの前後hashも一致し、`rustfmt --edition 2021 --check src/app/settings_ui.rs src/app/control_settings.rs` と `git diff --check` はexit0である。

未達はS04のJSON値比較である。`settings_ui.rs:345` の `assert_eq!(output, state["progress"])` が、wire JSONの `font_scale: Number(0.8)` と `serde_json::json!` でmemoryから直接作った `Number(0.800000011920929)` の違いで失敗した。S02のmemory Progress pretty JSON exact bytes検証は新規/置換とも成功している。既存serializationをValue経由へ変えてoracleへ合わせる実装変更は行わず、期待値を編集せず、親へ作者の独立判断を依頼した。初回の失敗は未達として保持し、33成功を34成功へ読み替えない。

作者最終source全体の開始hash一致は実測したが、変更前source snapshot/test-body hashは保存していない。この欠測を後からbefore証拠として補ったとは主張しない。許可stub/import以外のテスト編集は行っていない。親へ両sourceとCargoの所有を返却し、S04の独立判断後の再build/実行は別記録とする。

native picker取消し/実ボタン操作・正本write-block中のconsumer実UIは未測定である。Hのalias/identity/TOCTOU/poison/write-blockはHの独立証拠を再利用し、ここで再実行しない。closure外通知・picker順序・共通helperへの実配線は静的確認の範囲であり、実画面受入の代用としない。開始/終了usage counterは未取得、検証終了は2026-10-12 08:36 JST頃である。

## 親の統合追補

独立作者がS04のJSON表現経路を限定訂正した。期待値の丸め許容やfield除外はなく、呼出前memoryから期待wire bytesを作り出力exactを確認する。S04外全文/control fileは是正前snapshotと同一である。詳細とexact1件のGREENは [独立テスト追補](settings-export-tests.md)、判定は [レビュー](settings-export-review.md) を参照する。

最終settings_ui SHA256は `C7C7C9D17DD24FC3FDF5762F7CD0E649A1A07E3E7237D31FE6C5A1FB63D1BF9B`、control_settingsは上記から不変。test EXEは `287E9A9743D5B21E0B0829E715016885099D690941CEC7ECFCA5C618720B0737`。自己検証の接合は新25＋旧9の34unique成功であり、初回33/1の履歴を置き換えて隠さず、独立runner成功とも一括実行とも主張しない。次は確定差分の独立実行である。
