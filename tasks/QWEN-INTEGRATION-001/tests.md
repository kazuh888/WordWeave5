# QWEN-INTEGRATION-001 独立テスト

2026-10-03。test-author `/root/qwen_integration_tests`、計画者指定 `gpt-6-astra/high`。実行metadata、token開始/終了counter、親子包含は未取得である。
入力は親採用済みspec QI-AC-001～021、凍結design、精緻化planである。TDD Skillを全文確認し、テスト/fixtureと既存testのfeature cfgのみを所有する。production、期待値緩和、実API、実資格情報、公開操作は扱わない。

## テストとoracle

共有fixtureは `tools/qwen-audio/tests/fixtures/reuse/`。WAVは決定的な合成PCM16 8kHz mono 8frames、参照は `The sun is bright.`、助言/評価不能JSONは固定ファイル、キーは架空sentinelである。
同じfixture/helperをcore evaluate、単独GuiController、本体ReadingControllerへ通す。expected()は明示したFeedback/metadataであり、production parserの出力から期待値を生成しない。request観測は同一bytes、全文、host、固定要求値を検査する。Transportを呼んだ瞬間に数え、future未pollを送信0と誤認しない。
外部APIとOS資格情報の境界のみfakeを使う。音声parser、評価器、controller、settings状態は本番部品を使う。OS namespace試験はconstructor/accessorだけでload/saveしない。

| AC | 所有テスト | 期待 |
| --- | --- | --- |
| 001,007 | root qwen_reading: invalid_reference_boundaries, invalid_inputs, audio_sixty_second_boundary | 10,000scalar/60秒まで、超過・192kHz・3ch・破損等はHTTP0、旧音声再利用なし |
| 004～006 | integration_credentials＋root namespace/settings/read_failure | 専用target、旧constructor維持、取消し/保存失敗は旧接続と既準備ID保持、保存HTTP0 |
| 008～010 | root prepare/double_send/connection_change/foreign_controller | preview一致、最大1送信、別owner ID拒否、設定変更後再確認、送信時store再読なし |
| 009,011,012 | root source_rewrite/cancel/close/invalidate/restart/drop/timeout | 固定bytes、取消し先行0、送信先行1かつ不明、ready成功を不採用、reap前Busy、永久無効化 |
| 013～016,019 | integration_reuse＋root same_fixture/failure | 共通の正常/評価不能/metadata欠落/分類、secret反射拒否、自動retryなし |
| 002,003,007 | app qwen_reading_tests | 30秒境界、非対応原音保持、警告付き部分録音保持、選択取消し保持/読取失敗失効 |
| 011,017,020 | app qwen_reading_tests | Send+Cancel同frame取消先行0、ready成功取消/Close、補助例文版変更/消失、親zoom遮断、cancel/close狭幅rect、評価完了後のdeck/progress/disk/原音不変 |
| 018 | 既存Codex回帰/独立レビュー | 認証経路と結果非転送は親の統合検証対象。新testだけで静的経路保護を証明しない |
| 021 | 親/利用者 | 実APIと助言品質は別受入、mockから合格を推定しない |

## REDと引継ぎ

- `cargo test --manifest-path tools/qwen-audio/Cargo.toml --test integration_credentials --locked` → `red-credentials.log`。`Connection::with_host`未実装(E0599)。feature未導入のためwindows-credentials条件も未認識である。挙動不良の再現ではなく設計API不足のコンパイルREDである。
- `cargo test --test qwen_reading --locked` → `red-controller.log`。root qwen_reading module/qwen-audio/tokio未配線。API未実装のコンパイルREDであり、fixture壊れや動作バグの再現とは扱わない。
- `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --test integration_reuse --locked` → `red-reuse.log`。3 passed / 0 failed。既存評価器でfixtureの妥当性を確認したが、feature導入前のためcore依存分離合格と扱わない。gui条件は未認識であり単独GUI2ケースは実施されていない。

実装者への必要口：凍結design署名通りのcontroller、`WindowsCredentialStore::target()` accessor、root tokio macros/test-util、futures-util/dev dependency（BodyStream）、既存base64。test module配線は実装者の所有である。
UIは親との合意口 QwenDialog::new/accept_recording/select_wav、recording_stop_due、controller/raw_audio/notice可視fieldを使う。`dispatch_frame(&[QwenAction], &egui::Context)->bool`、`check_target(Option<&Entry>)` は親が確定した。`render(&egui::Context)->bool` とctx tempの `qwen-cancel/qwen-close` Rectを使用する。

初版引継ぎ時点の件数はroot controller 24、package credentials 4/reuse 5、UI 12である。core-only実行済みはreuse内3ケースだけ、残りは未実装API/未配線のため実行未完である。root/package新規ファイルとshared helperのrustfmt --checkは合格した。
UIの未検証項目は全文/結果末尾までのscroll、Sendへの到達、keyboard focus/IME、実機30秒停止・再生とWindows DPIである。footer geometryと録音停止条件の単体試験を実機成功と読み替えない。UIはmodule配線後にfocused実行する。親は並行でproduction全Uを担当し、所有を分離している。

## 実施担当へ

focused: 上記3コマンドに加え package default `--test integration_reuse --test integration_credentials`、root `--bin wordweave5 qwen_reading`。U4とU1～U3統合後に新しいログで実行する。
必須gateはplan通りroot全target test/release bin、単独package全target/release、core-only lib check/test、feature tree、native画面である。本稿のコンパイルREDを納品検証や全試験合格へ読み替えない。
新規test/helperのみrustfmtを行い、既存dirtyや全体fmt不適合を直さない。合成原ファイル変更テストは専用temp directoryを作り、自分のファイルと空directoryだけを削除する。

## 独立レビューP2後の表示回帰

QI-AC-011/012に対し、対象教材無効化後も送信確実性を表示する2回帰を追加した。所有は `src/app/qwen_reading_tests.rs` と本書のみ、計画者指定Astra/highを維持する。

- `qwen_invalidated_before_send_renders_not_sent_and_calls_zero`：準備後・未送信の対象消失でInvalidatedへ遷移し、実Modalに「送信していない。」が描画される。送信可能性/課金不明の誤表示なし、Transport呼出0。
- `qwen_invalidated_after_send_renders_remote_and_billing_unknown_and_calls_once`：Transport開始を待って教材版を変え、実Modalに送信済み可能性・遠隔処理完了/課金不明・自動再送なしを描画する。未送信の誤表示なし、描画/poll後もTransport呼出1。

固定期待値はQI-AC-012に由来する。実描画が生成したegui Text shapeを読み、状態判定helperの出力だけを試験しない。音声/資格情報/応答待ちは既存の合成fixtureを使い、実API/機器/OS credentialは操作しない。
着手時点で親のproduction修正（表示条件へInvalidated追加）は適用済みであり、修正前の動作REDは未観測である。REDを得るために所有外productionを戻さない。
`cargo test --bin wordweave5 qwen_invalidated --locked` は2 passed / 0 failed / 209 filtered outで合格した。`rustfmt --check --edition 2021 src/app/qwen_reading_tests.rs` も合格した。UI回帰は追加後14件である。

親から渡された `ui-tests.log` のズーム試験失敗（11 passed / 1 failed、actual 0.8 / expected 1.0）は、fixtureの初期倍率適用前に基準を取得した問題である。`WordApp::new_with_storage` が初期0.8を予約するコードを独立確認し、通常frameを一回描画してから基準値を取得するよう修正した。誤って受理されたshortcutの遅延反映も検出するため、入力後にも通常frameを一回進める。倍率・保存不変のassertionは維持する。
`cargo test --bin wordweave5 qwen_modal_blocks_parent_zoom_shortcut_and_preserves_learning_store --locked` は1 passed / 0 failed / 210 filtered outで合格した。両focused実行とも返却exit code 0である。全体gateの新しい結果は親/runnerが記録する。
