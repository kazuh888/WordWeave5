# U7 review-detail runner 結果

担当: test runner。計画指定は `gpt-5.6-terra / medium`、実行メタデータは未取得である。

## 対象

- HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。
- 親の最終source freeze後の共有未コミット作業木である。U7 focusedは実装担当報告で4件pass / exit 0（`impl7.log`）である。
- `git diff --check`: exit 0。ログ: `.context/compound-engineering/u7-diff-check.log`。

## 必須全target結果

`cargo test --all-targets --locked` は exit 101、196 passed / 1 failed / 0 ignored である。ログは `.context/compound-engineering/u7-all-targets.log`。

失敗条件は次である。

```text
app::harness_tests::material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling
src/app/harness_tests.rs:1368
dialog [[4.0 4.0] - [5699.3 348.3]]
viewport [[0.0 0.0] - [512.5 406.2]]
```

長い登録禁止文で狭幅dialogが幅外へ拡大している。runnerはソース・期待値・設定を変更しないため、本体/テスト所有担当へ返した。

## 未実行

全targetが失敗したため、debug build、release build、native `--ui-check` 4 captureは未実行である。実AI、実教材/学習データ、push/commitも未実行である。

## longreason幅修正後の再実行

- `git diff --check`: exit 0（`.context/compound-engineering/u7-diff-check-rerun.log`）。
- `cargo test --all-targets --locked`: exit 0、377 passed / 0 failed / 0 ignored（`.context/compound-engineering/u7-all-targets-rerun.log`）。
- `cargo build --locked --bin wordweave5`: exit 0（`.context/compound-engineering/u7-debug-build-rerun.log`）。
- `cargo build --release --locked --bin wordweave5`: exit 0。起動前の`wordweave5`は0件であり、強制終了はしていない（`.context/compound-engineering/u7-release-build-rerun.log`）。
- 隔離debug fixtureのreview/context/scrollを標準・smallで各1枚、計6枚生成し全exit 0（`.context/compound-engineering/u7-native-rerun.log`）。

親目視ではsmall contextがAI/tableを含まず、standardでは日本語table cellがclip下にある。exit 0をnative受入とは扱わず、fixture調整後の再captureを親へ残した。実AI・実データ・認証・IME/音声/ペンは未検証である。

## 最終native記録（実行者報告に基づく親の統合追記）

debug-only fixtureを対象Window layerと実右viewport内markerの8連続可視へ変更した後、runnerがdebugビルドとnativeを再実施した。production/testは変更せず、全targets/releaseの再反復はしていない。

- 最終画像は `.context/compound-engineering/u7-native/context-layer-{marker,tail}-{standard,small}.png` の4枚、すべて実exit0。実行ログはu7-native-layer.log、各画像と同じディレクトリにstdout/stderrを保存した。
- tail初回exit101はrunnerの`--material-context`引数漏れであり、stderrは `--material-context-tail requires --material-context`。訂正後は標準/狭幅ともexit0。本体/fixture不具合とは扱わない。
- 親と独立reviewerが日本語太字、表2列2行、固定回答末尾、左右paneと固定操作を実画像で確認した。表示受入を妨げる指摘はない。
- 更新release EXEは2026-09-30 21:31:46 JST、11,962,880 bytes、SHA256 `C91B34B1900D0D31DEC09E314153AEB7F0326055189588D2F9803BE2A8868F23`。実AI/実データ/IME/touch/pen/音声受入、commit/pushは未実施である。
