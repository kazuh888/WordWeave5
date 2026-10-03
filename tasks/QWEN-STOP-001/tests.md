# QWEN-STOP-001 独立テスト引継ぎ

担当はテスト作成のみ。計画者指定は `gpt-6.1-sol/high`、実行モデル/effort metadata・開始終了usage counterは未取得である。AGENTS.md、development-team.md、KPI、`agent-skills:test-driven-development`全文を読んだ。親からAC01–05と設計の承認、開始可を受領した。所有は `src/app/qwen_reading_tests.rs` の今回追加と本書のみであり、本体・共有cfg(test)・Cargo・公開は扱わない。

基準HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`、対象は既存dirty作業ツリーの今回追加である。既存変更を保持し、他者の編集は戻していない。

| テスト名 | 受入条件 | 判定 |
| --- | --- | --- |
| `qwen_stop_requires_actual_playback_and_disables_loading_end_and_error_states` | AC02 | snapshotなしは無効。loadedの有無に依存せず、playing/loadingの4組を検証。読込→再生→停止→再試行→自然終了と失敗相当Noneで再判定する。実Speakerのイベント試験ではない。 |
| `qwen_stop_is_visible_before_playback_and_repeated_disabled_clicks_preserve_input` | AC01/03/05 | 未選択／合成録音準備後に実dialogを描画。停止ラベル・viewport内のnativeボタン・disabledを確認し、3回のpointer連打で音声bytes、確認ID、phase、結果、設定移動、送信件数0を維持する。各イベント前にrectを消し、過去frameの存在情報で合格させない。 |
| `qwen_stop_shared_control_paints_a_round_button_square_and_stop_label` | AC01/04 | 実共有コンポーネントを描画し、有効／無効の両方で円形native Button、中心の塗りつぶし四角、可視「停止」をpaint shapeで確認する。 |
| `qwen_stop_shared_control_accepts_pointer_and_tab_keyboard_only_when_enabled` | AC03/04 | 実共有コンポーネントのpointer・Tab到達→Enter/Space実行を確認。request_focusは使わない。disabledのpointerは不発、Tabは停止を飛ばす。続く無効frameでもボタンが残る。 |

fixtureは既存 `reuse::audio()` と `entry()` の合成データ、各テスト専用egui Context、外部境界だけの既存Store/Wireである。実資格情報・HTTP・マイク・Speaker生成・実録音・実教材・一時ファイルは使用しない。既存Rust/Cargo環境のテストであり、browser試験は非該当である。

## RunnerへのコマンドとREDの限界

焦点コマンドは `cargo test --locked qwen_stop_`。既存保全は `cargo test --locked qwen_reading`。全体ゲートはplan記載の `cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5`。テスト作者には親からCargo実行禁止があり、実行とログは親runnerが担当する。作者が実行済みとは扱わない。

作者への引継ぎ時点で本体修正が済んでいるため、修正前の実動REDは未取得である。仕様・旧挙動の根拠から期待される挙動失敗は、未再生dialogで「停止」が描かれず、初期表示回帰の可視ラベル／rect要求が失敗することである。新API `original_audio_playing` / `stop_button` を持たない旧ソースに全テストをそのまま適用するとコンパイル失敗になるが、それを不具合再現とは扱わない。修正前保存ソースを使う場合は初期表示回帰だけの独立適用が必要であり、共有作業ツリーを巻き戻してはならない。fixture障害・コンパイル障害と挙動失敗を分けてrunner結果へ記録する。

## 未検証と既存回帰

実Speakerの有効停止→native音停止、実機の自然終了、録音中の実device状態、支援技術による読み上げ、tooltip表示は未検証である。状態表・共有button成功を実音声成功とは扱わない。AC01–05全体の受入には本体接続先の独立レビューと既存取消し／閉じる／送信確認回帰の実行結果を併せる。既存 `qwen_audio_load_cannot_reappear_after_cancel_close_or_target_change`、`qwen_loading_and_failed_replacement_cannot_send_previous_confirmation`、`qwen_completed_advice_and_close_do_not_mutate_learning_store_or_source_audio` などは変更せず親runnerへ引き継ぐ。

通常shellとnode読取はWindows sandbox helperの `apply deny-read ACLs` 障害で起動失敗した。対象限定read-only昇格で規約・仕様・fixtureを読めた。これはテストの挙動失敗ではない。品質ゲートやモデルは変更していない。

## 実施結果・凍結

親runnerから、全体testは2026-10-03 12:38:13Z開始、12:43:24Z終了、exit 0、新規 `qwen_stop_` 4件すべて成功と受領した。作者によるCargo実行ではない。実行ログと全体集計は親 `results.md` を正とする。

親依頼により担当テスト1ファイルへ `rustfmt --edition 2021 src/app/qwen_reading_tests.rs` を実行した。整形前保存コピー `logs/tests-before-format.rs` と比較し、差分は空白・改行・末尾カンマに限定されることを確認した。厳密なwhitespace-onlyではなくformat-onlyである。期待値・fixture・テストの意味は変更していない。`rustfmt --edition 2021 --check src/app/qwen_reading_tests.rs` はexit 0。sourceは再凍結済みである。

## U2 / AC06 独立描画コントラスト回帰

計画者の再選定 `gpt-6.1-sol/high` と親のAC06・設計承認を受領し、全文を確認した。担当範囲は今回追加1テストと本書末尾のみである。起動指定とruntime metadataは別であり、metadata・usage counterは引き続き未取得である。

追加テストは `qwen_stop_inactive_square_and_active_outline_increase_visible_contrast_at_zoom`。既存の実共有ボタンfixtureをzoom 0.8 / 1.0 / 1.25 / 1.6で描画し、中央■のfillと外円のstrokeをpaint shapeから取得する。固定した旧色#4D6073/#D3E2EFに対し、無効■の明度上昇、有効円枠の明度低下と、両状態間の相対輝度コントラスト比の拡大を要求する。新RGB値は固定しない。変更しない約束の有効■#162C41、無効円枠の旧色/幅1.5、両状態の同一寸法も確認し、承認設計の有効枠幅増を測る。他icons不変は本体差分の独立レビューへ引き継ぐ。

旧描画値はU1の既存状態を基準とする。親の `logs/contrast/playback-before.rs` は実コピーではなくU2差分を戻した再構成であり、前回検証済みhashとの一致確認は親の証拠である。変更前に本テストを実行したREDは作者未取得である。旧描画では新テストの「無効■が従来より明るい」「有効枠が従来より濃い」が同値となり失敗する期待であり、コンパイル／fixture障害とは区別する。native画面比較は親担当で、paint shapeの成功だけで受入完了とはしない。

`rustfmt --edition 2021 src/app/qwen_reading_tests.rs` と担当ファイルの `--check` は成功した。実コピー `logs/contrast/tests-before.rs` と、新AC06 blockを除いたソースを空白除去比較し、既存テスト全体のtoken不変を確認した。2026-10-03 22:07:50 JSTにsourceを凍結し、親runnerへ `cargo test --locked qwen_stop_` または全体テスト内で新規1件を識別する検証を引き渡した。作者はCargo禁止を守り未実行であり、U2実行結果は親results.mdへ記録する。

### U2初回失敗と比較fixture是正

親runnerの初回U2実行はexit 101、lib134件成功、bin263件成功/1件失敗、integrationは未到達であった。`logs/contrast/tests.log` の唯一の失敗は追加色差テストで、zoom0.8の無効円枠に実描画#767E84と未変換RGB#D3E2EFの同値を要求した箇所である。これは本体の円枠変更を検出した証拠ではなく、実描画値と入力値を混同したテストoracleの誤りである。

一次確認：ローカル公式egui0.31.1の `src/ui.rs` ではadd_enabled(false)がdisableを呼びpainterへfadeを設定し、button描画後に元painterを復元する。`src/widgets/button.rs` のButton外枠はその無効painterを使い、外枠がhover/focus時に拡張される。■は共有描画の復元後painterで描かれる。さらに実アプリの `src/app.rs:383` は `ctx.set_visuals(egui::Visuals::light())` を指定するが、初版fixtureは `Context::default()` のdarkを使用していた。初版のraw円枠同値と本番と異なるthemeは期待する保全／色差を正しく評価できない。

是正はAC06の期待値を緩めず、同theme・zoom・layout・pointer/focus条件で旧native Buttonを別に描く比較fixtureを追加するものである。旧42px/角丸21/白fill/枠幅1.5/#D3E2EFと、外側painterの旧■#4D6073/有効■#162C41を保存ソースから再現し、現行共有停止は実本体を呼ぶ。双方とも実アプリのlight visualsとする。無効の旧・新実描画strokeの同値、円fill・外円/■/native widget形状の保全、有効■保全を要求し、無効■の明度上昇・有効枠の明度低下・状態間コントラスト比拡大・枠幅増という改善条件は維持する。

比較条件はzoom0.8/1/1.25/1.6と通常/hover/focusである。hoverはstop widget IDの実Response.contains_pointerとhovered==enabledを確認し、イベントを渡しただけで成立としない。focusは強制request_focusを使わずTabで実IDへの到達を確認する（無効時は停止を飛ばして次のbuttonへ到達）。Paint shape抽出はnative hover/focusの外枠拡張に対応する。既存4件の意味は変更していない。

2026-10-03 22:22:38 JSTにsourceを再凍結した。担当rustfmt/checkはexit 0、U2追加前実コピーとの比較で追加AC06 block以外のtoken不変を確認した。親へ全体再実行を引き渡し、作者はCargoを実行していない。今回の是正を実動REDや本体の修正として数えず、再実行結果とnative受入は親results.mdへ追記する。
