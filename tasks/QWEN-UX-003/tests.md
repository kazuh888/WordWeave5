# QWEN-UX-003 独立テスト作成

2026-10-03。所有：`src/app/qwen_reading_tests.rs` と本書のみ。計画者指定は `gpt-6.1-sol/high`、実行メタデータと開始/終了usage countersは未取得である。AGENTS.md、開発7役、KPI、test-driven-development全文、計画・仕様・親承認済み設計を照合した。

## 受入対応

既存20件を保持し、10件を追加した。以前の独立した状態欄、手動準備ボタン、断定調の案内に関する期待を承認済みQ3仕様に合わせて更新した。取消し優先、送信済み可能性、遠隔処理/課金不明、初期技術詳細の折畳み、原音と学習記録の保存に関する既存assertionは維持した。

| 新規テスト名の末尾 | 受入ID | 検証する振る舞い |
| --- | --- | --- |
| auto_confirmation_repeated_render_preserves_snapshot_and_sends_zero | 008、009 | 有効録音後の自動確認、12描画で同じID、通信0、原音不変 |
| auto_confirmation_requires_visible_send_click_and_sends_once | 008、009 | 実描画内の送信ボタンをpointerで押して初めて1回、実payloadの英文/bytes一致、重複送信拒否 |
| auto_confirmation_audio_replacement_rejects_old_id_and_sends_new_bytes | 009 | 音声変更直後と新確認後の旧ID拒否、新IDで新bytesを1回送信 |
| completed_summaries_expand_without_resend_or_discarding_sent_content | 005、008、012 | 完了後①②が初期折畳み、実pointer開閉で送信時の名前/英文/音声情報を閲覧、再送0、結果と原音不変 |
| restart_discards_previous_result_and_requires_new_confirmation_id | 009、010 | 再練習で結果/閲覧用確認を破棄、新確認ID、旧ID拒否、自動再送0 |
| three_stage_guidance_places_input_and_connection_failures_before_results | 001～003、007、009 | 音声なし、未設定、資格情報読取失敗、不正音声。段階順と原因配置、送信/旧prepare部品不在、通信0 |
| failed_evaluation_folds_previous_steps_and_never_automatically_retries | 005、007、010 | 通信失敗/不正JSON後③に再練習と送信済み可能性、①②概要、発音不良と推定しない、再送0 |
| long_reference_initial_overview_and_close_are_visible_at_supported_scales | 001、013 | 820×650、80/100/125/160%、長文でも実paintのclip内に初期全段階概要/現在段階/固定終了 |
| loading_and_failed_replacement_cannot_send_previous_confirmation | 009、010、012 | 新音声読込中の旧ID送信拒否、取込失敗後の旧音声復帰禁止、通信0、入力原本不変 |
| sent_display_receipt_is_discarded_on_target_change_or_close | 010、012 | 対象変更/終了で送信時の閲覧用確認/原音/結果を破棄、旧ID拒否、再送0 |

すべて名前の先頭は `qwen_` である。合成WAVと既存合成JSONを用い、実controller/egui/音声形式検査/ローカル読込を動かす。資格情報と外部transportだけを既存Store/Wire fixtureで隔離した。不正MP3用にテストが一意なtemporary directoryを作成し、その自分のファイルだけを終了時に除去する。実キー・API・マイク・利用者音声は使わない。

## 実施と引継ぎ

所有テストへ `rustfmt --edition 2021 src/app/qwen_reading_tests.rs` を実施しexit 0を得た。構文整形成功はコンパイル/テスト成功ではない。通常shellはdeny-read ACL初期化で起動できず、対象読取と所有ファイルの整形を許可環境で実施した。

Cargoは親指示によりrunnerへ集約するため、本担当は実行していない。変更前REDは未取得である。本体が並行改修済みのため、現ソースでの失敗を変更前再現と扱わない。変更前では自動確認preview、3段階配置、閲覧用確認保持のassertion失敗を予測するが、予測と実測を区別する。getterがない旧ソースでcompile errorとなる場合はRED証拠にならない。

runnerへ渡す焦点コマンド：`cargo test --locked --bin wordweave5 qwen_reading_tests`。新規回帰だけの焦点には `cargo test --locked --bin wordweave5 qwen_auto_confirmation`、`qwen_completed_summaries`、`qwen_restart_discards` 等のfilterを使える。30件全体のexit codeと件数/失敗を記録し、本体差分固定後に規定のfmt/all-targets/release gateを親が実施する。

compile error/fixture timeout/実動作assertion失敗を区別し、独立期待値を実装に合わせて弱めない。実pointer試験は描画されたRectがbody内に完全にあることを要求する。旧phaseのRectを証拠にしないようテスト描画helperで毎frameのcontrol hookをクリアする。長文可視試験は実paintの完全clip内glyph行のみを数える。

nativeでのkeyboard実操作、実マイク/再生、IME、Windows DPI、実API、利用者のnative最終受入は未検証である。句点改行・案内全体の丁寧語は対象文字列と既存描画試験を更新したが、全画面の文章/行位置を網羅して自動保証していない。追加テストと既存viewport回帰の成功をnative操作合格へ拡張しない。

## 独立レビュー後の追加回帰

親からレビューP2二項目の対策を受領し、以下4件を追加した。現時点の所有テストは34件（既存20、初回追加10、レビュー後追加4）である。所有テストのみrustfmtを再実施してexit 0を得た。追加4件のCargo/変更前REDは本担当で未実施であり、runnerへ引き渡した。

| テスト名の末尾 | 受入ID | 追加の観測 |
| --- | --- | --- |
| open_summaries_reset_on_completion_and_restart_resend_in_same_context | 005、010 | 同じContextでRunningの①②を開く→Completedで閉じる。Completedの①②を開く→再練習・手動再送で新Runningの①②を閉じる→開いた後の新Completedでも閉じる。通信は手動2回のみ、原音不変 |
| running_open_summaries_reset_on_failure_in_same_context | 005、007 | Runningで①②を開いたまま通信失敗→Failedで閉じて再練習と送信済み可能性を表示、再送0 |
| completed_long_audio_name_wraps_and_keeps_summary_and_close_reachable | 005、013 | 一意temp内の200文字WAV名を実ローカル読込・確認・送信。Completedの820×650/160%で概要とCloseが画面内、短名を表示し、展開した全文は実galleyで折り返す。展開後もCloseが画面内、原音/原本不変 |
| keyboard_tab_reaches_send_summary_and_close_and_preserves_audio | 008、013 | headless eguiで実Tabキーから実Response.idへのfocus到達を確認し、Enterで送信1回、Tab/Spaceで②概要展開、Tab/Enterで終了。request_focusは使わず、focusだけでは送信0回、概要展開でも再送0 |

Runningの確認を決定的に行うため、追加fixture HeldReadingResponseは外部transportの応答だけをNotifyで保留する。controller/描画/操作は実部品を使用する。終了状態へ移行させるときだけ応答を開放し、同じegui Contextを保つ。初期折畳みの判定は詳細本文の不在を観測し、実装のpush_id文字列をassertionへ写していない。

200文字名はWindows temporary filesystemへの適合も必要なfixtureである。ファイル作成/読込の失敗は表示動作のREDとは別に報告する。keyboard回帰の成功はheadlessの操作証拠であり、nativekeyboard/DPI/IMEの受入を代替しない。

## 初回全体実行の失敗と入力操作修正

runnerの `logs/cargo-test-all.log` はcompile成功後、250 passed / 9 failedを記録した。失敗内訳はreading8件、settings1件である。readingの6件は送信ボタンがbody clip外（ボタンy1080～1120、body y489～1007）で、実クリック前の可視assertionが失敗した。残り2件は③正文見出しを初期paintから取得できなかった。settingsは旧断定調の破棄説明を期待していた。これを変更前の動作RED証拠とは扱わない。

親はAreaのavailable heightと前frameのstateによるbody制約を特定し、標準画面の余った高さを使う本体修正を担当した。本担当は標準1000×1400の初期③正文見出し期待を維持した。本文の段階別配置・詳細全文の試験では、同じContextを実MouseWheelで先頭から末尾へ移動し、clip内の完全可視glyph行だけを収集して各正文見出しへの到達と原因配置を確認する。

click helperは実MouseWheelを上下80ptずつ送り、bodyとscreenの両方がcontrol Rect全体を含むまで最大60回の限定反復を行ってからpointerを押す。完全可視assertionを削除していない。原音、通信数、旧ID拒否、初期折畳み、取消し優先の期待は変更しない。

親が追加所有を明示した `src/app/qwen_settings_tests.rs:335` の期待文字列だけを「音声と結果を破棄します」へ更新した。他のsettings変更はない。readingテストのrustfmt exit 0、Cargo再実行はrunnerへ引き渡し、本担当では未実施である。テスト件数34件は変わらない。
