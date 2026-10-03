# MATERIAL-QUOTE-001 独立テスト（RED引継ぎ）

対象は承認済み [spec.md](spec.md) の MQ-AC-01～10。合成会話のみを用い、実行記録・実教材・live AIを使用しない。テスト作成担当の所有は `src/material.rs` と `src/app/notifications.rs` の `cfg(test)` 節であり、本体は編集していない。

| 新規テスト | 受入IDと合成fixture |
| --- | --- |
| `quote_rejection_distinguishes_reference_role_empty_length_and_mismatch` | 01,03。選択外・不正話者・空白・2001文字・Markdown記号を省いた文全体を区別。 |
| `quote_rejection_identifies_original_nonconsecutive_turn_and_both_roles` | 02,05,10。選択 `[0,3]`、user/assistant、往復1/4、固定本文と引用の識別。 |
| `quote_matching_remains_exact_and_accepts_2000_unicode_scalars` | 03,09。Markdownを含む真の連続部分・2000 Unicodeスカラー値は受理し、改変/過長は拒否。強調記号の内側の部分文字列だけなら正常である。 |
| `quote_previews_escape_before_display_and_mark_only_true_truncation` | 04,09。240/241文字、日本語・絵文字・改行・タブ・引用符・バックスラッシュ、個別省略と全文照合。 |
| `saved_draft_rechecks_quotes_without_repairing_or_rebinding_sources` | 08。合成DraftのJSON往復、正常・旧形式の根拠なし案、空/過長/非包含とsnapshot欠落。 |
| `quote_turn_number_handles_usize_max_and_last_valid_conversation_index` | 09。`usize::MAX` 拒否と元index 199→往復200。 |
| `material_quote_failure_opens_actual_notice_without_replacing_draft_or_learning_data` | 05,07,10。実 `Request::build_response` の `Err` を `Activity::Material` の既存 channel/tick に渡し、通知自動表示、案なし/既存案保持、教材・進捗・選択状態不変を検査。 |

RED: `cargo test --locked material::tests` はコンパイル成功、16件中11成功・新規5失敗（終了コード101）。理由分類、往復/話者、プレビュー、保存Draft説明、往復200の不足を現行文字列で再現した。ログは `.context/compound-engineering/material-quote-red-material.log`。初回試行で「強調記号の内側の語」まで不一致としたfixtureを発見し、仕様の連続部分文字列規則に従って文全体の不一致へ訂正した。最終REDは訂正後の結果である。

通知の焦点実行: `cargo test --locked notifications::tests::material_quote_failure_opens_actual_notice_without_replacing_draft_or_learning_data` は1件成功。終了時点で本体担当の `src/material.rs` / `src/ai.rs` 差分が存在したため、実装前REDとは見なさない。ログは `.context/compound-engineering/material-quote-red-notification.log`。

引継ぎコマンドは上記2件。実装後の全対象試験・releaseビルドはrunner担当である。MQ-AC-06 の生成指示は `src/ai.rs::Config::material` の静的差分で確認する。既存の正常案→可視差分→利用者承認と追加/訂正の回帰、標準/狭幅・拡大表示の実機確認もrunnerの検証範囲である。headless通知試験は文字列/状態を確認しており、Windowsフォント・DPIによる実際の可読性は未検証である。

## U3 変更理由形式: 独立RED引継ぎ

2026-09-27。親から承認済みMQ-F-01～04、設計独立レビューpass、planner指定 `gpt-6-sol/high` を受領。実行モデル/effortのmetadataと本子の使用量カウンターは未取得。合成会話・既存組込みdeck・各fixture専用一時ディレクトリのみを使用し、実教材やlive Codexは使用していない。所有は `src/material.rs` の `cfg(test)`、`tests/codex_process.rs`、`tests/support/mock_codex.rs` と本追補に限定した。

| 新規テスト | ID | 独立期待値・fixture |
| --- | --- | --- |
| `format_accepts_all_specified_existing_paths_and_boundary_indices` | MQ-F-01,04 | 14トップレベル（`/answers`を含む）とexamples/replacements各3フィールドの実在index 0/1を合成Entryで受理。 |
| `format_rejects_whole_items_and_bad_indices_with_reason_number_and_path_in_both_entries` | MQ-F-01,03,04 | 有効引用の前置理由4/5/6件から理由5/6/7の全体pathを第一失敗にする。未知項目、空/符号/先頭ゼロ/小数/巨大/末尾改行/余剰セグメント、配列外index、保存済みgeneratedの空配列を生成/保存入口で分類。 |
| `format_distinguishes_explanation_and_quote_count_boundaries_in_both_entries` | MQ-F-03 | 説明空白/2001、引用0/11を理由番号/path/原因付きで拒否し、2000 Unicodeスカラー値と1/10件を受理。 |
| `format_preserves_quote_diagnostics_and_saved_fixed_evidence` | MQ-F-03,04; MQ-AC-02,03,08 | Markdown記号を省いた引用を生成とJSON往復後の保存案で拒否し、固定原文・往復・話者の診断を保持。 |
| `format_path_preview_escapes_controls_and_truncates_only_after_240_scalars` | MQ-F-03; MQ-AC-04 | 生成/保存済み両入口の不正pathで240/241 Unicodeスカラー値、改行・タブ・引用符・バックスラッシュ・絵文字を検査。240ではJSON escapeした全文を表示し省略なし、241では先頭240だけを表示して省略を明記し、検証は全文を拒否する。 |
| `format_schema_constrains_paths_and_reason_sizes_without_changing_structure` | MQ-F-02 | 独立に書いたpath patternと `maxItems:100`、引用`minItems:1/maxItems:10`、添字`minimum:0`、required/未知項目拒否を確認。 |
| `material_config_sends_restricted_path_schema_and_actionable_instructions` | MQ-F-02,04 | Windowsで実 `Config::material`→隔離mock app-serverへ送信。`thread/start.developerInstructions` の許可例/0始まり/要素全体禁止と `turn/start.outputSchema` の独立期待値を検査。mockは合成正常応答を返し、送信patternのみ `MATERIAL_PATH_PATTERN=...` と標準出力に出す。 |

RED結果: `cargo test --locked --lib material::tests` はコンパイル成功、21件中17成功・4失敗、終了コード1。失敗はschema path patternが`Null`、生成/保存のpath・説明・引用条件が一括文言、引用エラーにpath文脈なしという仕様上の不足である。正常pathの新規1件と既存16件は成功した。`cargo test --locked --test codex_process material -- --nocapture` はコンパイル/実送信成功、1件中1失敗、終了コード1。最初のassertは生成指示に `/answers` がないことである。compile error、mock通信エラー、fixture破損ではない。初回integration build/linkは4分15秒を要した。RED出力は親へ共有済みであり、ログ保存のみの重複実行はしない。

RED後にテストfixtureの有効引用を合成原文内の文字列へ合わせ、改行を含むpathの表示期待をJSON escapeへ合わせた。空配列は有効Entry生成の前処理に妨げられない保存済みgeneratedで検査する。これらは期待条件の緩和ではなく、最終小修正後の再実行は本体GREENで行う。テストソースは凍結し、本体担当へ `src/material.rs` を解放した。

本体担当のU3編集凍結後、path表示境界の上記1件を追加した。親の指示によりこの追補の焦点cargoは未実行であり、初回REDの失敗件数へ含めない。runnerはGREENで新規7件と既存回帰を判定する。

GREEN/runner引継ぎ: `cargo test --locked --lib material::tests`、`cargo test --locked --test codex_process material -- --nocapture`。後者の `MATERIAL_PATH_PATTERN=` の値をPowerShellの `.NET Regex` + `RegexOptions.ECMAScript` でコンパイルし、正例 `/answers`, `/examples/0/note`, `/examples/1/english`, `/replacements/0/conditions` と負例 `/id`, `/examples/0`, `/replacements/0`, `/examples/01/note`, `/examples/-1/note`, `/examples/2/unknown` を独立評価する。巨大数と末尾LFは正規表現ではなくruntimeの拒否を確認する。これはschemaの実AI適合を証明しない。既存 `append_reason_is_remapped_to_merged_example_and_dropped_changes_have_no_reason`、`saved_draft_rechecks_quotes_without_repairing_or_rebinding_sources`、通知・承認・学習状態の既存回帰を保持する。全対象test、release build、実画面・実AIの区別はrunner/親の範囲である。

## U4 回復結果通知: 独立RED引継ぎ

2026-09-27。親からMQ-R-01～05、U4仕様・設計の独立PASS、planner指定 `gpt-6-sol/high` を受領した。実行metadataと本子の使用量カウンターは未取得。所有は `src/app/notifications.rs` の `cfg(test)` と本追記のみである。既存 `harness_tests::fixture()` の隔離一時データに合成 `RunRecord` を送り、`Activity::Recovery` のchannel→`tick`→`notification_window` を通した。実記録、実教材、live AIは使っていない。

| 新規テスト | 受入IDと独立期待値 |
| --- | --- |
| `recovered_outcome_and_response_matrix_distinguishes_completion_from_saved_text` | MQ-R-01～03,05。6状態×None/空文字/Unicode空白/非空白本文の24組。Completed＋本文のみ情報通知、残りは非赤色の注意を自動表示。状態・本文取得/未取得を文字で識別し、本文転載・教材/会話/学習状態変更・新規生成を拒否。 |
| `recovered_attention_reopens_after_dismissal_and_success_clears_old_alerts` | MQ-R-01,04。同じInterrupted＋本文なしを閉じて再回復すると再表示。Completed＋本文は旧error/attentionを消し、新たに注意を開かない。 |
| `recovery_scan_failure_takes_precedence_over_recovery_notice` | MQ-R-04。隔離fixtureのstorageを一時的に外して後発 `refresh_runs` エラーを発生させ、回復成功案内よりエラーを優先表示。 |

RED: `cargo test --locked --bin wordweave5 notifications::tests::recover -- --nocapture` はコンパイル成功、4件中2成功・2失敗、終了コード101。失敗はPrepared＋本文なしで注意窓が開かず本文確認可能と誤表示すること、およびInterrupted＋本文なしで注意窓が開かないことである。compile errorやfixture破損ではない。既存ナビゲーションテストと後発scanエラーテストは成功した。ログは `.context/compound-engineering/material-recovery-notice-red.log`。本体担当へREDとsource凍結を通知済みである。GREEN/runnerは同じ焦点コマンドの後、既定の全対象test/release buildを新規実行する。実画面の可読性、実AI、利用者の再試行成功はこの合成試験では未検証である。

## U5 教材比較・詳細・反映方法: 独立RED引継ぎ

2026-09-27。親からMQ-UI-01～08の仕様・設計独立passと承認、planner指定 `gpt-6-sol/high`、U5a→U5bの本体実装順を受領した。実行metadataと本子の使用量カウンターは未取得。所有は `src/app/harness_tests.rs` の追加テスト、`src/app/visual_check.rs` の合成プレビューfixture、本追記のみである。隔離一時store、既存組込みdeckのメモリ上の複製、合成会話のみを使用し、実教材・実会話・live AIを使用していない。

| 新規テスト・fixture | ID | 独立期待値 |
| --- | --- | --- |
| `material_ui_selects_first_change_and_keeps_reasons_quotes_and_context_in_sync` | MQ-UI-01,03,05,08 | 複数行の先頭自動選択、2行目クリック後の理由と引用の同期、固定会話全文の初期折畳み、選択行消滅時の先頭復帰、差分0件時の旧詳細消去、案・学習状態不変。 |
| `material_ui_mode_names_change_only_the_ui_and_preserve_serialized_mode` | MQ-UI-07,08 | 選択前と案表示の新UI名称、復習影響説明。従来の `Mode::label()` とserde値は独立期待値で固定。 |
| `material_detail_shows_all_replacements_without_expanding_a_fold` | MQ-UI-06 | 2件の合成言い換えのphrase/meaning/conditionsを詳細初期状態から読み、語調情報と節を分ける。表示のみでdeckを変更しない。 |
| `material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling` | MQ-UI-02,04,08 | 820×650/zoom1.6相当の論理viewportで長文合成案を描き、見出し・登録・破棄のclip内矩形と本文ホイール後の位置不変、progress/deck不変を確認。 |
| `--ui-check PNG --material-review` / `--material-detail` / `--material-mode`（`--small`併用） | MQ-UI-01～08 | 既存native撮影入口で比較案、言い換え詳細、反映方法選択の合成画面を標準1150×950/zoom0.8と狭幅820×650/zoom1.6で確認する。プレビューの画素・矩形・スクロール到達と展開後の固定会話はrunnerが記録する。 |

RED実測: `cargo test --locked --bin wordweave5 material_ui_ -- --nocapture` はcompile成功、2件中2失敗/exit1。初回差分が未選択で理由未取得、固定会話全文が初期表示され、新UI名称/説明は表示されない。`cargo test --locked --bin wordweave5 material_detail_shows_all_replacements_without_expanding_a_fold -- --nocapture` はcompile成功、1件中1失敗/exit1。詳細には折畳み見出しだけがあり、strengthen等の合成phrase/意味/条件は初期表示されない。いずれも行動上の失敗でありfixture破損・compile errorではない。狭幅テストと新規native fixtureは追加後のRED/撮影を未実施であり、初回RED件数へ含めない。

U5a初回GREEN判定の比較試験は、先頭選択・引用同期・消失/0件の各表示assertを通過し、末尾の不変assertだけが落ちた。原因は新しい `Conversation::new/complete` を再実行して時刻・IDが異なる案と比較したfixture側の誤りである。同一案の描画前snapshotを、意図的に案を編集する直前に比較する位置へ移した。表示期待値・progress/deck不変条件は維持し、この訂正後はcargo未実行である。モード説明と狭幅clipの失敗は本体担当が修正中であり、このfixture訂正と混同しない。

GREEN/runnerの焦点コマンドは `cargo test --locked --bin wordweave5 material_ui_ -- --nocapture` と `cargo test --locked --bin wordweave5 material_detail_shows_all_replacements_without_expanding_a_fold -- --nocapture`。続いて既定の `cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` を行う。撮影入口はdebug専用のため、`cargo build --locked --bin wordweave5` 後に隔離出力先へ `target/debug/wordweave5.exe --ui-check <PNG_PATH> --material-review [--small]`、`--material-detail`、`--material-mode` で標準・狭幅PNGを作成する。撮影だけでは本文スクロール後・展開後・操作の可用性が証明されないため、画面で別途確認する。実AIと実学習データは検証対象外である。

U5レビュー追補（2026-09-27）。`material_ui_legacy_proposal_explains_missing_fixed_source_before_expansion` はMQ-UI-05の旧案境界を、固定snapshotなしの合成案で初期描画し、欠落説明のclip内可視性と既存「元の会話を表示」を確認する。`material_ui_new_proposal_starts_with_fixed_context_collapsed_in_the_same_dialog` はMQ-UI-03,05,08を、同一窓で前案の固定会話を実際に展開した後、同じsource識別子・時刻で候補だけ異なる次案へ切り替え、全文を初期折畳みに戻すことを確認する。狭幅テストには長い不正path診断を持つ登録禁止案を追加し、MQ-UI-04,08の禁止理由表示と見出し・登録/破棄のclip内可視性、案/deck不変を再確認する。いずれも合成案であり、期待値を現行描画に合わせて緩めていない。U5b本体とcargoの競合を避ける親指示により、追補のcompile/REDは未実行である。次回 `cargo test --locked --bin wordweave5 material_ui_ -- --nocapture` の結果は初回GREEN/回帰判定として記録する。

新案fold追補の焦点再判定: 初回 `material_ui_` の4成功/1失敗は、前案を展開した証拠assertで止まっており、epoch引継ぎの失敗を示していなかった。合成チャットの本文・自動タイトルと固定snapshotを分離し、表示中のfold見出しがclip内に入りスクロールが安定してから実クリックするようfixtureを修正した。クリック後の折畳み内容に固有の単独行 `往復 1` を展開の証拠とし、`往復 1・あなた` の引用ラベルとは完全一致で区別する。固定全文は本文スクロール先にあるため、初期viewportのshape出力へ全文を要求する判定は行わない。次案は隔離mpsc channelから `AiResult::Material` として `tick` に渡し、実際のepoch更新を通して同一窓で単独行が消え、foldラベルが残ることを確認した。`cargo test --locked --bin wordweave5 material_ui_new_proposal_starts_with_fixed_context_collapsed_in_the_same_dialog -- --nocapture` はcompile成功、1成功/0失敗、exit0。ログは `tasks/MATERIAL-QUOTE-001/u5-fold-focused.log`。eguiテストctxのアニメーション時間は0に固定した。実時間アニメーションと全文のスクロール到達はnative画面受入に残す。

狭幅画像追補: 隔離native画像 `material-ui-u5-material-review-small.png` では見出し・notice・元会話操作の直後にfooterがあり、比較本文を確認できない。`material-ui-u5-material-mode-small.png` では導入文と見出しが窓を占め、モード選択へ到達した証拠がない。詳細画像も言い換えは初期viewport外である。既存狭幅試験に本文内ホイール後の変更一覧・前後両枠のclip内到達とfooterの維持を追加し、`material_ui_narrow_mode_choices_are_reachable_inside_the_dialog` で追加のみ/内容見直し/作成操作の到達を独立検査する。`--ui-check PNG --material-review|--material-detail|--material-mode --small --material-scroll` は合成画面へ限定のホイールを送り、24frame目にスクロール後PNGを保存する入口である。現時点で新試験のcompile/結果と新PNGは未確認であり、runnerのfocused実行と標準・狭幅画像確認を要する。撮影後の文字・枠の到達はPNGを実見して判定する。

狭幅比較試験の初回postfix判定は他5件成功・当該1件失敗であり、到達集合は「変更前」「登録される内容」の2件だけだった。テストは到達収集を始める前に大きなホイール移動と18frame待機を行い、一覧見出しを通過していた。fixtureの順序を、初期viewportから収集開始→本文上の固定位置で小刻みなホイール→各描画で4項目を収集→footer固定を確認、へ訂正した。4項目の期待は維持する。親のcargo競合回避指示によりこの訂正後の試験は未実行であり、runnerの次回全対象testへ統合する。

後続のnative small-scroll撮影は、回帰試験が狭幅で成功しても初期画像から変化しなかった。撮影fixtureの比較窓ホイール位置が窓高の65%で固定footer付近に当たると親が画像で確認したため、比較・モード窓だけ45%の本文内へ移した。教材詳細の72%位置は維持する。再撮影結果はrunner判定待ちであり、画像変化を先取りして合格としない。

footer配置追補: native `material-ui-u5-footer-material-review-small.png` では登録・破棄の文字は見えるが、窓の左右境界が画面下へ続き下端が650px画像内にない。既存のボタンclip検査だけではMQ-UI-04の窓全体の到達性を判定できないため、狭幅試験に `ctx.memory` の教材窓 `area_rect` が512.5×406.25論理viewportに全て含まれるassertを、通常案と長い禁止理由案の両方へ追加した。期待条件は緩めていない。追加後のcargoは親の調整により未実行である。
