# FEEDBACK-UI-001 U5 最終runner結果

2026-09-28。担当: test runner。計画者指定は `gpt-5.6-terra / medium`、実行model/effort metadataは未取得である。source/test/expectation/settingsは変更していない。

## 対象

- HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` と最終凍結済み未コミット差分。初期差分証跡は `logs/notice-runner-tested-revision.txt`、status/diffは同接頭辞のファイルである。
- 初回native PNGは本体欠陥発見前の工程記録であり、受入には用いない。最終PNGは `artifacts/notice-native-rerun2/` である。

## 実施済み

- `cargo test --all-targets --locked`: PID 6580、実exit 0、終了 2026-09-28T23:10:21+09:00。9 target、361 passed / 0 failed / 0 ignored / 0 measured / 0 filtered。stdout/stderr/marker: `logs/notice-runner-rerun2-all-targets.*`。
- `cargo build --locked --bin wordweave5`: PID 17616、実exit 0、終了 2026-09-28T23:11:47+09:00。stdout/stderr/marker: `logs/notice-runner-rerun2-debug.*`。
- 隔離debug native: summary/compare/raw/end × 標準1150×950・80% / 狭幅820×650・160% の8枚すべて実exit 0、待機後process不存在、PNG存在。個別ログは `logs/notice-native-rerun2/`。
- 目視: 標準summaryで本文より大きいtitleとfooterの「閉じる」を確認。標準compareでAI引用本文、狭幅endで `固定元発言の終端MARKER`、copy・技術情報・閉じるを確認。実IME、マイク、pen、TTS、実認証/AIは未検証である。

## PNG SHA-256

| file | SHA-256 |
| --- | --- |
| compare 1150 | `9A52B919D73E39005EF88F0E40B38FC5940F8CFA850116FABF076F81864843B9` |
| compare 820 | `36319904C869B50F2BC43786243324758C4EABC6BEB0A3B6F1BF92AB1C5846D3` |
| raw 1150 | `2FEE4DD8B57A57FEBF73FB5D8251A6E12AB62A69F6EB902F0EC8D8EB34E889B2` |
| raw 820 | `8EA8BCF415AE0BD34C74B1236E72C4614530A82B06B95BB2BD0C1FA7FAE186FC` |
| end 1150 | `46495ACBC9B3C601A3BA4669B222ABDB953DB7BD878775CC9868F413D23133DB` |
| end 820 | `F0770C526B884C841817850C5E9277C741C18E912E18C3C7C94155A523D0AEF8` |
| summary 1150 | `ED5FC71E786B83F2F68DE663E106487C3BA62D1D302B14C2A56362F5DB0172FE` |
| summary 820 | `9B1A6BA8237DFC0FCDAFBA23470DD2BE124B91CAB01088ECA48719D420F81607` |

## release

- ユーザーの終了連絡後、`wordweave5_processes=0` を確認した。PID 22060を停止していない。アプリ再起動もしていない。
- `cargo build --release --locked --bin wordweave5`: PID 24612、実exit 0、終了 2026-09-28T23:16:09+09:00。stdout/stderr/marker: `logs/notice-runner-rerun2-release.*`。
- `target/release/wordweave5.exe`: 2026-09-28T23:16:07+09:00、11,598,336 bytes、SHA-256 `3674873D6CE550127CFE0FE5DDD107781D7965E3BB557A42E8E7E2664EFDDB76`。

## 残る受入

- 実IME、マイク、pen、TTS、実認証/AI、および利用者によるnative操作受入は未実施である。合成PNG・mockをこれらの結果として扱わない。
