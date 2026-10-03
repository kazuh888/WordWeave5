# QWEN-UX-002 検証担当記録

2026-10-03。計画者指定・親の起動指定は `gpt-6.1-sol / medium`。実行メタデータ、開始/終了tokenカウンターは未取得である。担当はログ・報告・隔離fixture・ビルド生成物のみであり、ソース・テスト・期待値・設定を変更しない。

最終判定：必須本体testは451件、共通packageのdefault/core-onlyは147件/100件が合格し、両release生成はexit0である。旧焦点の不合格は記録を保持し、独立fixture到達修正後の全体テストで該当2件の合格を確認した。通常アプリの最小820×650では150/160%時のQwen専用保存・キャンセルが完全可視である。480×640・150%の右端切れは通常アプリが到達できないstress条件として保持する。全体fmtの既存不合格と未実施の実機受入は残る。終了時刻は2026-10-03 18:31 JST、使用量カウンターは欠測である。

入力は `plan.md` の確定割当、`spec.md` QU-AC-001～018、`design.md`、`tests.md`。AGENTS.md、development-team.md、KPI、wordweave-changeの全文を読み、同Skillのリスク別検証と報告のみ適用した。新UIの利用者受入・実API受入は未完了である。

## 固定対象

HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。未コミット対象は `runner-before.diff` と `runner-before.sha256` で識別する。後者はsrc/testsと共通packageのsrc/tests、および両Cargo.toml/Cargo.lockを含む。広範な既存dirtyが存在するためHEADのみを検証対象としない。

親開始debug buildはexit0、4分00秒で完了した。その後親が `visual_check.rs` のwhitelistへ既存合成状態4件を追加したため、以後は `runner-fixed.diff` / `runner-fixed.sha256` を正本とする。前後比較は同ファイルのみ変更（81848120…→CA2F35AC…）である。合成表示用のincremental debug rebuildを必要とし、旧EXEを新差分の表示証拠に使用しない。

## 実施中結果

| コマンド | 終了コード | 結果・ログ |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 1 | 不合格。39ファイルの整形差分、`fmt-root.log`。既存広範差分を含み、無関係な整形を実施しない |
| `cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check` | 0 | 合格、`fmt-package.log` |
| `rustfmt --edition 2021 --check src/app/qwen_settings.rs src/app/qwen_settings_tests.rs src/app/qwen_reading_ui.rs src/app/qwen_reading_tests.rs` | 0 | 合格、`fmt-focused.log` |
| `cargo test --locked --bin wordweave5 qwen_settings_tests -j1` | 0 | 中間差分で27 pass / 0 fail / 0 ignored / 216 filtered。`app-settings-focused.log` |
| `cargo test --locked --bin wordweave5 qwen_reading_tests -j1` | 101 | 18 pass / 2 fail / 0 ignored / 225 filtered。`app-reading-focused.log`。送信前/後の教材変更時に既存送信状態文言を検出できず、親へ修正を返した |
| `cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked --target-dir target/qwen-audio -j1` | 0 | 147 pass / 0 fail / 0 ignored / 0 filtered、16 suites、5分00秒。`package-all-targets.log` |
| `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --tests --locked --target-dir target/qwen-audio -j1` | 0 | 100 pass / 0 fail / 0 ignored / 0 filtered、15 suites、2分01秒。`package-core-only.log`。無効なfeature用suiteは0 tests |
| `cargo check --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --features windows-credentials --locked --target-dir target/qwen-audio -j1` | 0 | コンパイル合格、55.81秒。`package-windows-credentials.log`。実資格情報操作ではない |
| `cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio --target-dir target/qwen-audio -j1` | 0 | release生成合格、5分59秒。`package-release.log` |
| `cargo test --locked --bin wordweave5 qwen_invalidated_ -j1` | 101 | 0 pass / 2 fail / 0 ignored / 243 filtered、compile4分00秒。`app-invalidated-final.log` |
| `rustfmt --edition 2021 --check src/app/qwen_reading_tests.rs` | 0 | 独立テストの実wheel到達handoff後の書式合格、`fmt-test-handoff.log` |
| `cargo test --all-targets --locked -j1` | 0 | 451 pass / 0 fail / 0 ignored / 0 filtered、10 suites、5分24秒。`root-all-targets.log` |
| `cargo build --release --locked --bin wordweave5 -j1` | 0 | release生成合格、6分55秒。`root-release.log` |
| `cargo build --locked --bin wordweave5 -j1`（fixture修正後） | 0 | 合成設定fixture再生成合格、3分00秒。`debug-build-fixture-v2.log` |
| `rustfmt --edition 2021 --check src/app/qwen_settings.rs`（debug専用1行修正後） | 0 | 合格、`fmt-native-v3.log` |
| `cargo build --locked --bin wordweave5 -j1`（即時scroll fixture修正後） | 0 | 最終合成fixture生成合格、2分41秒。`debug-build-fixture-v3.log` |
| `cargo build --locked --bin wordweave5 -j1`（実最小サイズfixture修正後） | 0 | 最小幅合成fixture生成合格、3分28秒。`debug-build-fixture-v4.log` |

親から手渡された今回の共通焦点結果は40 pass / 0 fail / 0 ignoredであり、`core-focused.log`。本担当の再実行ではない。親開始のdebugビルドsession 21456は子から参照できず、親の終了コード0の連絡とログで確認した。

読み込み焦点の失敗は `qwen_invalidated_before_send_renders_not_sent_and_calls_zero`（line176、`送信していない。`）と `qwen_invalidated_after_send_renders_remote_and_billing_unknown_and_calls_once`（line210、`送信済みの可能性がある。`）。追加P2B回帰 `qwen_input_errors_and_invalidated_target_do_not_advise_missing_restart_or_audio` は同実行で合格した。親・各所有者の修正以外に本担当はソース・期待値を変更していない。

通知導線と音読案内のP2修正、独立回帰追加後は `runner-freeze.diff` / `runner-freeze.sha256` に保存した。実行中ビルド/テストは開始時点とコンパイル時点でsourceが更新された中間結果であり、最終合格は後続の固定対象全体検証を要する。コマンドごとに依存のcold compileが観測され、キャッシュ/設定変更は行っていない。

親の送信状態説明配置修正とtest-authorの失敗診断追加・未使用import除去後、最終正本は `runner-final.diff`（SHA256 `F5319ADC02FD9A2E3392D969C4ADFA5349DEFF8BDC3562ABDA3D0133D65A0264`）および `runner-final.sha256`（SHA256 `DF040C3719A9B053A21466144021BC1C6C8B35CC777FF6F51A0283ACB7C81D53`）である。共通packageの入力はこの過程で変更なし。コマンド時刻・exitは `package-exits.log` に保存する。

この固定対象のInvalidated再確認2件はline188/225の `対象の教材が変更された。` で不合格となった。描画見出し `qwen_reading_ui.rs:485` は句点なしである。送信状態assertはこの後のため今回その通過を確認できていない。親へ根拠を返し、root全体test/debug/native/root releaseは当該不合格で自動停止した。テストを緩和しない。再修正後は対象を更新し、関連する検証だけ再実行する。

test-author/親の追加handoffは実wheel操作23frameで表示範囲のglyphを集約するInvalidated2件のfixtureである。文言と通信回数の期待値、初期表示の既存3frame検証を維持した。対象は `runner-accepted-input.diff` / `.sha256` に再固定（前対象との差は `qwen_reading_tests.rs` のみ）した。必須全体テストでInvalidated2件とP2A通知導線/P2B案内の回帰を全て個別okで確認した。

全体testの正本は `runner-accepted-input.diff` SHA256 `4710A8567AAA48139A67F14E7BCB37769AC288D604AF792A3C6188BCA5E6A07B`、manifest SHA256 `7F50DD0F67979F030DFD3748F723086EFC845C00FC3DF9B9D6976D2CC23410E5`。その後の変更はreleaseから除外されるdebug-only `visual_check.rs` のfixtureのみで、release本体入力は不変である。native正本は `runner-native-input.diff` SHA256 `24DC644B364A0E83035211A78ABC36EF32124C7A7B88F24C5BD83310466EA23E`、manifest SHA256 `279D3F2E0A5749319F8CDAD8B51030D85A10E435867756FF9D94A993F145CE6F`。native後 `runner-after.sha256` との比較差は0件である。

## native合成画面

`cargo build --locked --bin wordweave5 -j1` は最新productionでexit0、3分33秒（`debug-build-acceptance.log`）。`--ui-check <絶対PNGパス> --qwen-reading <状態>` で8枚を取得し、全owned PIDは通常closeでexit0（`native-exits.log`）。実キー・実API・実音声・学習データは使用していない。`--small --qwen-scale 1.5` は480×640の150%である。

確認画像は `confirm-standard.png`、`jsonfailed-standard.png`、`fieldsfailed-narrow.png`、`connections-standard.png`、`settings-narrow.png`、`probe-success-standard.png`、`probe-auth-narrow.png`、`mismatch-standard.png`。全画像をview_imageで観測した。確認の可視snapshotと明示送信、JSON段階の安全分類と再練習、mismatchの推定英文、narrowの再練習/close/破棄footer、connections保存済みカードは観測できた。

ただしsettings/probeの3枚はCodex上部だけを映し、Qwen editor/結果へ未到達である。PNG保存成功を該当設定受入の合格としない。親がdebug-only fixture初期化をnative viewport確定後へ遅延する修正を行い、該当画像の再取得を予定する。全体テストは変更前の固定対象、後続nativeはfixture差分を別途記録する。

親のfixture初期化遅延後は `settings-standard-v2.png`、`probe-success-standard-v2.png`、`probe-auth-narrow-v2.png` を別名で再取得し、3 owned PID全て正常exit0。全3画像を観測し、settingsはHost/キー入力/接続確認ボタンへ到達した。一方probe-successはCodex中段、probe-authはQwenキー上部までで、結果/専用保存ボタンのnative可視性はまだ証明できていない。親へ証拠不足を返した。旧画像を成功証拠へ置き換えない。

親が `qwen_settings.rs` のcfg(debug_assertions)合成tail処理1行を即時scrollへ変更した後、v3は `probe-success-standard-v3.png`、`probe-auth-standard-v3.png`、`probe-auth-narrow-v3.png` を取得した。3 owned PID全てexit0、全3画像を観測した。標準1150×950では結果文言・接続確認・Qwen専用保存/キャンセルを同時に確認した。成功はモデル一覧だけの確認で、音声評価・推論権限・費用・品質を保証しない説明も表示されている。認証失敗は接続先APIキー確認へ案内している。480×640・150%末尾ではキャンセルの右側とラベルがviewport外へ切れ、同条件での完全可視性は合格にしない。この制約を親へ報告済みであり、runnerは追加修正しない。

v3時点のnative正本は `runner-native-v3-input.sha256` SHA256 `606C92B689CBF7B00976456BC93AA4538CCA0D7B1D492E4AE4E9FB23AA1B776C`。v2からの差は `qwen_settings.rs` のdebug-only行のみ（7ADF77…→6E0EDE…）。v3終了時のafter比較は0件。同ファイルは未追跡のためgit tracked diffのhashはv2と同じであり、正確な対象識別はmanifestも必須である。本体test/両releaseの実行経路は変わらず、debug-only合成処理は追加buildとnativeで検証した。通過した全体test/両releaseを理由なく反復していない。

v3の後、親が `visual_check.rs` の設定/probe `--small` 初期サイズだけを820×650へ変更した。`main.rs:26` の実アプリ `min_inner_size([820.0,650.0])` を本担当も読み取った。readingの480×640stressは保持する。v4の `probe-auth-min-v4.png`（820×650/150%/tail）、`settings-min-v4.png`（同150%/Host focus）、`probe-success-min-v4.png`（同160%/tail）を取得し、3 owned PID全てexit0、全3画像を観測した。認証失敗と成功の末尾ではQwen専用保存/キャンセルが両方完全に表示され、横切れはない。settingsはHost/地域/形式に到達した。tail画像では結果文言が上へ外れるため、最小幅で状態と末尾を同時可視と主張しない。状態文言のnative証拠は標準v3、入力Host/キーの証拠は標準v2、最小幅の専用操作はv4に分ける。

最終nativeの正本は `runner-native-v4-input.diff` SHA256 `4ED8AB0029B03792F0263D265398CFCD7D0E8ADE1E608A16E6D7AEF2BA14AF95` と `.sha256` SHA256 `2A893C2FB1A60CC06911F125C8694BB2213114DED060AC7F84DCF9FF9FCE7A45`。v3からの差は `visual_check.rs` のdebug-onlyサイズだけ（E7B65…→6CAD9…）。最終 `runner-after.sha256` との比較は0差分である。本体production/test・両release実行経路は変更なし。

## 生成物と安全

`build-artifacts.sha256` に生成物ハッシュを記録した。本体releaseは `42E13397C323A34D5A1F77E6456C4B5DE5D764ADECD2E50559018C3B49FE4B49`、共通package releaseは `57A3BE5B10F117E146E1C8AAC77616AA4A0F0873D74D68D858BD8C6E9A64506E`。本体debugはfixture版であり、release native受入の証拠ではない。ユーザーアプリをkillせず、Get-Processでwordweave5不在を確認後にreleaseを生成した。合成fixtureはowned PIDのみ通常CloseMainWindowを使用し、全て終了済みである。

最終debug EXE SHA256は `1E4D4DE8209FA10C611E1777582A8F5F53A408B1733E7D3782EAC15DDCE7E0E0`。終了時cargo/wordweave5該当processは0件である。ログ時刻・exitは `root-exits.log` / `package-exits.log`、native起動PIDとexitは `native-exits.log`。ビルド警告はログへ残し、エラーやnative受入不合格と混同しない。

書式の全体不合格は保持する。共通package・限定Qwen4ファイルと最終独立testの書式は合格。不要なcargo fix/一括rustfmt、source/test/期待値/設定変更、公開を本担当から行っていない。使用量カウンターは開始・終了とも未取得であり、compile経過時間からtoken消費を推定しない。

## 残る受入

実マイク・実音声、IME/ペン/TTS、実Codex認証、実Workspace/API、利用者による新UI受入は未検証である。合成native fixtureとmock通信の結果をこれらの新しい実機結果に置換しない。release native操作、keyboard/スクロール/OS終了は観測できた範囲を別記する。

native画像の観測は標準1150×950・100%、実最小820×650・150/160%、到達不能stress480×640・150%の合成表示に限る。820×650・80/100/125/160%の全操作到達性とkeyboardは自動egui fixtureの証拠であり、同条件のOSキーボード操作を新規実施したとはしない。未対応API/時間切れ/制限等のnative個別画面は取得せず、mock状態回帰の合格と区別する。実APIでResponseInvalidが解消したか、6地域で実接続・音声推論が許可されるかは未確定のままである。
