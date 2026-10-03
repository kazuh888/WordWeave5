# MATERIAL-QUOTE-001 検証結果

完了境界は凍結差分の自動検証とWindows release生成物までである。実行model/effort、使用量カウンター、親子包含関係は取得不能であり、推定していない。

## U5 教材比較・詳細・反映方法 — 最終検証完了

- 最終固定差分: base `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 既存U1～U4とU5の未コミット変更。最後の変更はfold回帰試験の表示到達fixtureであり、本体変更後のnative確認結果を再利用した。実データ・実AIは使用していない。
- `cargo test --all-targets --locked`: 315 passed / 0 failed（125+153+2+32+2+1、0件targetが2つ）、exit 0。ログ `.context/compound-engineering/material-ui-u5-closure-alltests.log` の8結果行と終了マーカーを親も確認した。
- `cargo build --release --locked --bin wordweave5`: exit 0、2分41秒、既存/未使用コード警告3件。ログ `.context/compound-engineering/material-ui-u5-closure-release.log`。runnerが更新前にwordweave5未起動を確認し、強制終了なし。
- 更新EXE: `target/release/wordweave5.exe`、11,390,464 bytes、2026-09-27 16:05:33 JST（UTC 07:05:33.2169390）、SHA256 `4644B2AECBF270F28F3AEADB81DB8D374D4A84CB8304193B138B4E60A834FCAE`。親が実ファイル・ハッシュ・release終了マーカーを照合した。
- 最終native比較画像は `.context/compound-engineering/material-ui-u5-boundsfix-review-{standard,small,small-scroll}.png`。親と独立reviewerが実見し、標準の選択比較、狭幅の窓全体/固定操作/ステータス表示、スクロール後の前後枠到達を確認した。詳細/モードは本体不変の `material-ui-u5-postfix2-material-{detail,mode}-{standard,small,small-scroll}.png` 6枚の確認を再利用した。展開/新案折畳みは実クリックと合成受信のheadless回帰で確認し、native展開操作を別途実施したとは扱わない。
- 最終ソースSHA256先頭（親が終了後取得）: app.rs `17D3E891E3B5`、chat_ui.rs `A6E359223E3C`、material_review.rs `ABA9C04EDE89`、materials_ui.rs `7378E0D7880E`、material.rs `18A4014961F2`、harness_tests.rs `A376919C2D95`、visual_check.rs `12AA508A8E87`。最終test開始後はソース/期待値を変更せず、文書のみ更新した。`git diff --check` 成功。
- 残る未確認: 利用者の実教材での操作受入、実AI再生成、OS DPI・IME・マイク・ペン・TTS。本変更の合成描画/回帰はこれらを証明しない。commit/pushなし。

### U5 途中の検証・差戻し履歴（以下は最終結果ではない）

- 計画者指定は test runner=`gpt-5.6-terra` / `medium` である。実行model/effort metadataおよび使用量カウンターは未取得であり、推定していない。
- 初回focusedのexit 101は本体動作の失敗ではない。クリック後に固定会話全文がclip外へ出ることを、global shapesの全文存在で断定した観測不良であった。修正版は実channel→tick切替を維持し、展開内に固有の単独往復見出しが現れることを検証する。修正版1件は親受領ログ `tasks/MATERIAL-QUOTE-001/u5-fold-focused.log` でexit 0、他4件は本体不変の既存GREENを再利用した。
- debug/native開始時のtested revisionは `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 未コミット固定差分。UTF-8全worktree `git diff --binary` SHA-256 は `4B797C3AE27FC170E32D6FBF6AF9696207C5A91AF21CF877BEDA39BE8896FEC0`、`git diff --check` exit 0。runnerはソース・テスト・期待値・設定を編集していない。
- `cargo build --locked --bin wordweave5`: exit 0、debug生成成功。dead-code警告3件（`ww_selectable_label`、`finish`、`say`）は失敗ではない。ログ: `.context/compound-engineering/material-ui-u5-debug-build.log`。
- 隔離`--ui-check`で比較・詳細・反映方法の標準/狭幅PNGを各1枚、計6枚生成し全exit 0。ログ: `.context/compound-engineering/material-ui-u5-native.log`。ただし親の目視で `material-review-small` は固定導入/noticeにより比較一覧・前後本文が初期画面で不可視、`material-mode-small` は段階説明によりモード選択が不可視、さらに右端×が切れることを確認した。PNG生成成功は視覚受入の成功ではないため、U5a本体修正へ差戻しである。
- `cargo test --all-targets --locked`: exit 0、314 passed / 0 failed / 0 ignored / 0 measured / 0 skipped（8 test-result行の合計）。ログ: `.context/compound-engineering/material-ui-u5-all-tests.log`。終了後の全worktree diff SHA-256 は `4C3DAC8250F7CBE6747DA6CC5E1FEFB109B25A932DF471764E9344A9D9F722EF` であったが、これは文書・ログ追記でも変化するためRust本体の同一性根拠ではない。狭幅native FAILおよび後続のU5a本体修正が、新固定差分での再実行根拠である。
- U5bの既存focused 1件成功ログは、`materials_ui.rs` が該当凍結後に不変であるとの親引継ぎにより再実行しなかった。狭幅native FAILによりrelease build、EXE更新/ハッシュ/起動中EXE確認はいずれも未実行である。U5a修正後の新固定差分ではfocused→debug/native→全対象test→releaseを再実行する。
- 後続U5a修正のfocused: `cargo test --locked --bin wordweave5 material_ui_ -- --nocapture` は exit 101、5 passed / 1 failed / 0 ignored / 0 measured / 147 filtered。tested revisionは同じ`d86e7e9d4fe4497d8be9040d30a078582f2887d8`、開始時対象Rust diff SHA-256は`2E6A64D0945AA396201C5AA909377F7B65AAFD5A889F4BD27BBDDF6C3D162BF5`。失敗は `material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling`（`harness_tests.rs:1233`）であり、本文スクロール後に到達した見出しが`変更前`、`登録される内容`の2件だけで、期待4件に満たない。debug/native9、全対象test、releaseは未実行で停止した。ログ: `.context/compound-engineering/material-ui-u5-postfix-focused.log`。
- nativeの再撮影後、`cargo test --all-targets --locked` は exit 101で失敗した。main binary unit tests は152 passed / 1 failed / 0 ignored / 0 measured / 0 filtered（153件）であり、失敗は同じ狭幅回帰の`harness_tests.rs:1243`、`内容を確認して教材に登録 must remain visible beside long prohibition`である。残るtest targetはこの失敗により実行されず、全体成功件数は成立しない。開始時対象Rust diff SHA-256は`13731EF88B92DFB244038ADB1CEF4D67827FA11F97D151C8248486F51CA2BCB3`。native目視合格はこの自動回帰失敗を相殺しないため、release、EXE更新/ハッシュ/起動中EXE確認は未実行で停止した。ログ: `.context/compound-engineering/material-ui-u5-postfix2-all-tests.log`。
- footerの後続修正では、窓全体boundsを追加した狭幅focusedがexit 101で失敗した。`material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling` は`harness_tests.rs:1180`で、dialog `[[4.0, 4.0] - [514.5, 368.6]]` がviewport `[[0.0, 0.0] - [512.5, 406.2]]` の右端を2.0pt超過したことを検出した。開始時対象Rust diff SHA-256は`5CD550F8046E9DBF593EDF4A9EE860E478F32D70AE60318B5EE0AD624AB00423`。debug/native、全対象test、releaseは未実行で停止した。ログ: `.context/compound-engineering/material-ui-u5-final-narrow-focused.log`。

未検証: 修正後のU5標準/狭幅native描画、スクロール・展開後の固定操作、実AI/app-server認証、実教材/学習状態、IME・マイク・ペン・TTS。旧ログ・mock・合成fixtureを新規native結果として扱わない。

- tested revision: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 未コミット差分。検証前後で対象差分は同一であった（UTF-8の`git diff` SHA-256）：`src/material.rs` `198D8BF2C698081AA416647FCB681433BD35C037E12A56B15C45E7BD89324452`（194行）、`src/ai.rs` `9BAC9ABDB80284C2A08A2F489B5D05CCE825BC9F508EA0DE5ADE54F62F6DC0F2`（12行）、`src/app/notifications.rs` `6353D11604046A0DDC087AE0B7C59FA40D91BB1F42E240D296D785A305672E9D`（53行）。
- `cargo test --all-targets --locked`: exit 0、298 passed / 0 failed / 0 ignored / 0 measured / 0 skipped（8 test-result行の合計）。ログ: `.context/compound-engineering/material-quote-all-tests.log`。既存dead-code警告は`finish`と`say`の2件であり、失敗ではない。
- `cargo build --release --locked --bin wordweave5`: exit 0、`Finished release profile [optimized]`（2m55s）。ログ: `.context/compound-engineering/material-quote-release.log`。同じ2件の既存dead-code警告あり。
- 生成物: `target/release/wordweave5.exe`、11,371,520 bytes、UTC `2026-09-27T01:10:12`、SHA-256 `0F470A9C7CFBA5463C4739D56567F6154430C70FA72490F505A96C8EAA71619A`。実行前に観測した旧EXE（UTC 2026-09-26、11,372,032 bytes）は成功証拠に使用していない。
- focused regression は、実装担当報告のmaterial 16/16、既存通知1/1を同一差分で再実行しなかった。source変更なしのため重複を避けた。

実行ラッパーが内部cargo完了前に完了として返り、test/releaseの起動が重なった。強制終了は行わず、両方の新規ログに実cargoのexit 0を確認した。以後はラッパー完了ではなくログの終了マーカーと生成物更新を完了判定とする。

未検証: 実機の標準/狭幅/DPI通知可読性・スクロール/コピー到達性、実AIの引用遵守、実教材/学習状態での受入、IME・マイク・ペン・TTS・認証。mock/合成fixtureおよび過去ログをこれらのnative結果として扱わない。

## U3 変更理由形式 — 追補検証

- tested revision: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 未コミット差分。検証前後で同一（UTF-8の`git diff` SHA-256）：`src/material.rs` `B27CE73F92AA460A88CA7F4495A5FD494275117DA11DD1BD293739205E3A5C1B`（447行）、`src/ai.rs` `FF252D6037AC68F295CBE3F079CC389E9F9AC25B512055676B967E7DC20D97A6`（20行）、`src/app/notifications.rs` `6353D11604046A0DDC087AE0B7C59FA40D91BB1F42E240D296D785A305672E9D`（53行）、`tests/codex_process.rs` `9FABB97B7DA1711A3704A91ABA9C9BDEF4B8578AAD2663D9FDA1E9B0C8D62AC8`（59行）、`tests/support/mock_codex.rs` `6F859A99C161781AB5AD38340DE0D7F5B63A8041985FE36ED98E27456DAC14DA`（25行）。
- 実送信pattern: focused integrationログの`MATERIAL_PATH_PATTERN`を.NET `RegexOptions.ECMAScript`でコンパイルした。正例`/answers`、`/examples/0/note`、`/examples/1/english`、`/replacements/0/conditions`は全てmatch、負例`/id`、要素全体、先頭ゼロ、負数、未知fieldは全てnon-match。巨大indexと末尾LFはECMAScript `$`の境界差を避け、runtime拒否の責務として本補助確認から除外した。exit 0、ログ: `.context/compound-engineering/material-format-pattern.log`。
- `cargo test --all-targets --locked`: 実session終了を確認しexit 0、305 passed / 0 failed / 0 ignored / 0 measured / 0 skipped（8 test-result行の合計）。ログ: `.context/compound-engineering/material-format-all-tests.log`。
- `cargo build --release --locked --bin wordweave5`: 前テストexit 0後に開始し、実session終了でexit 0。ログ: `.context/compound-engineering/material-format-release.log`。releaseの既存dead-code警告は`finish`と`say`の2件。全対象試験にはこれに加え、`tests/codex_process.rs`が読み込む`ai.rs`の`examples`、未使用associated items、`download_words`のdead-code警告が出た。いずれも失敗ではない。
- 生成物: `target/release/wordweave5.exe`、11,375,104 bytes、UTC `2026-09-27T02:05:08`、SHA-256 `CB356D322EA73C576E3EBC2D1B0ACC07623F1C82C245995469F5D409BA24706A`。起動中のEXEはrelease直前の確認で検出されず、強制終了は行っていない。
- focused 22/22およびintegration 1/1は、凍結差分で既に成功した新規ログを入力証拠として受領し、入力不変のため再実行しなかった。

未検証: real Codex/app-server認証・実AIのschema遵守、実機の画面可読性/通知操作、実教材・学習状態、IME・マイク・ペン・TTS。mock送信試験、.NET正規表現補助確認、合成fixtureはこれらのnative結果を証明しない。

## U4 再取得結果に即した通知 — 固定差分検証

- 実行モデル/effort、使用量カウンター、親子包含関係は取得不能であり、推定していない。計画者指定は test runner=`gpt-5.6-terra` / `medium` であるが、これは実行metadataではない。
- tested revision: `d86e7e9d4fe4497d8be9040d30a078582f2887d8` + 未コミット固定差分。runnerはソース・テスト・期待値・設定を編集していない。実session終了後のUTF-8 `git diff` SHA-256 は `src/app.rs`=`0DB3DE7D786A6B5876B70EEE83503E32780DEB3B4654EAC5D7E96E9CC9A45D61`（38行、26追加/1削除）、`src/app/notifications.rs`=`4B26533ECFE6DA0CEBF20623B216669EDA5CC20059CB27BA6E183DC15CFC6AAA`（183行、167追加/0削除）である。`git diff --check` は exit 0 であった。
- focused regression: `.context/compound-engineering/material-recovery-notice-green.log` を受領した。`notifications::tests` 13 passed / 0 failed / 0 ignored / 0 measured / 133 filtered out、exit 0である。新規GREENの入力不変により再実行していない。warning経路は静的レビュー済みであり、直接の実行試験ではない。
- `cargo test --all-targets --locked`: nested session 58202 の終了までpollし exit 0 を確認した。308 passed / 0 failed / 0 ignored / 0 measured / 0 skipped（8 test-result行の合計）。ログ: `.context/compound-engineering/material-recovery-notice-all-tests.log`。依存再コンパイルを含む。dead-code警告は `finish`、`say` と、`tests/codex_process.rs` が取り込む `ai.rs` の `examples`・未使用associated items・`download_words` であり、失敗ではない。
- release直前の `Get-Process wordweave5` は未検出であった。ユーザーのアプリを停止していない。`cargo build --release --locked --bin wordweave5`: nested session 30918 の終了までpollし exit 0 を確認した。`Finished release profile [optimized]`（1m20s）。ログ: `.context/compound-engineering/material-recovery-notice-release.log`。生成先は既定の `target/release/wordweave5.exe` である。
- 生成物: `target/release/wordweave5.exe`、11,376,128 bytes、UTC `2026-09-27T02:43:55`、SHA-256 `DD138102A892139D0892BF61FA6A5F8235F1E6E358E6DBFC2533A79578DD82D3`。

未検証: 実際の再試行成功、実AI/app-server認証、実機の通知可読性・操作、実教材/学習状態、IME・マイク・ペン・TTS。合成fixture・mock・既存focusedログ・release生成物は、これらのnative結果を証明しない。
