# QWEN-SETTINGS-001 独立テスト作成・RED記録

2026-10-03。計画者指定は `gpt-6.1-sol / high`（U1～U4）、取得できた実行model/effortメタデータと開始/終了token counterは未取得である。入力は親がP-S01/P-S02と6地域の順序を採用した `spec.md` と `design.md`。`agent-skills:test-driven-development` のテスト作成/RED工程だけを実施し、本体挙動は変更していない。

## 所有とデータ

- `tools/qwen-audio/tests/qwen_response_contract.rs`：9試験。既存の公開validatorを使用し、別英文の6項目契約、型/欠落/矛盾、文字数/JSON bytes境界、従来2区分を検証する。
- `tools/qwen-audio/tests/qwen_region_contract.rs`：9試験。6地域、選択地域との一致、Workspace境界、URL拒否、Tokyo旧入口、同一正規化hostだけの旧キー保持を検証する。
- `tools/qwen-audio/tests/integration_credentials.rs`：旧host変更の期待値を「別Workspaceのキー非転用」へ更新する。既存namespaceのconstructor/accessor試験は維持する。
- `src/app/qwen_settings_tests.rs`：独立レビュー追補後20試験。親が確定したEditor APIだけを使用し、初回キー、同一/変更host、読取/保存失敗、再試行、取消し、通常設定との独立性、OS終了保留、未設定だけの導線と音声破棄、終了後フォーカス、modal背景保存遮断を検証する。
- `tools/qwen-audio/tests/internal/credential_record.rs`：親の追加所有許可による純粋decoder試験4件を追加した。既存blob安全性4件を維持し計8件である。Windows/windows-credentials featureで内部cfg(test)へ組み込まれる。
- 親の追加所有許可：`tests/qwen_reading.rs` のhost変更/確認失効試験へ新キー明示を追加し、空キーで別hostへの保存を拒否した場合の旧確認保持を1件追加した。`tools/qwen-audio/tests/consent.rs` の保存失敗/取消し保持caseだけ、保存失敗中の編集から送信不可・取消後に旧接続で明示送信を検証した。
- 本ファイルと焦点実行ログだけを所有する。共有Rust production fileは編集していない。

JSON、接続先、キーはすべて合成fixtureである。API送信は0回、Windows資格情報の読込/保存は0回である。純粋なparser/validatorは実装を直接使い、外部境界の資格情報のみ既存 `preview_data::Store` を使用する。本体画面試験の保存データは既存ハーネスのプロセスID/時刻/連番付き一時ディレクトリへ隔離する。

## 受入IDとの対応

| 受入ID | 自動試験の範囲 | 親/runnerへ残す範囲 |
| --- | --- | --- |
| QS-AC-001/002/003 | 新結果の明示区分、聞取文/参照保存、厳格6項目、通常/評価不能保持、境界/不正JSON | 本体/単独版の文言・読み直し導線、実音声での判定精度 |
| QS-AC-007 | 6地域すべて、30組の地域不一致、label 1/63/64、禁止URL/path | UI文字エラー・地域選択到達性 |
| QS-AC-008 | 空キーの初回/変更先拒否、同一正規化先だけ保持 | standaloneの編集操作 |
| QS-AC-009 | TokyoHostの限定性と変換、既存namespace constructor、旧Tokyo v1/6地域v1の純粋復号と入力bytes不変、未知version/effort/host・破損record拒否 | Windows storeの実読込/書戻しとstandalone領域の実操作は行わない |
| QS-AC-010/011 | editor開封でキー非表示/自動保存なし、一般設定save/cancelとの独立性、再open | modal背景操作抑止と見える説明 |
| QS-AC-012 | dirty close要求、draft維持/破棄、変更なしclose、OS CancelClose | Escape/外クリック/ボタンのnative操作 |
| QS-AC-013 | 保存失敗でbaseline/draft保持、明示retry成功、読取失敗/入力検証 | visible errorのUI確認 |
| QS-AC-006/014 | 未設定だけの設定リンク/事前破棄説明、確定移動でaudio/result/preview破棄、接続変更で確認失効、保存失敗で旧確認保持、編集modal送信拒否 | 遅延完了・実native操作・送信時snapshotの他の既存回帰は親/runner |
| QS-AC-016 | 保存/変更なし取消/dirty破棄/OS終了要求後の実編集ボタンwidget IDへのfocus、modal背景save click拒否 | 全操作keyboard到達・実native・拡大/狭幅の追加確認 |
| QS-AC-004/017 | 新規試験の対象外 | 遅延結果・全体検証を親/runnerへ委譲 |

## RED

1. 修正前の既存公開APIに対して実行した：

   `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --locked --test qwen_response_contract`

   結果は9件中6成功/3失敗（exit 101）。`valid_reference_mismatch_preserves_heard_sentence_and_confirmed_reference` が仕様適合JSONを `ResponseInvalid` として拒否した。scalar境界の適合fixture、65,536 bytesの適合JSONも拒否した。ログは `red-response.log`。コンパイル成功後の挙動REDであり、壊れたfixtureやビルド失敗ではない。

2. 将来APIを要求する契約として実行した：

   `cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --locked --test qwen_region_contract --test integration_credentials`

   `ApiHost`/`Region` 未exportと、既存 `with_host` が `Connection` を返すためResult操作が存在しないコンパイルRED（exit 101）。ログは `red-region.log`。新APIの不在を示す証拠であり、実行された挙動試験として数えない。

過去に利用者が観測したResponseInvalidの原API応答は保存されていない。その報告の原因を再現したとは主張せず、今回承認された別英文契約の未対応のみを再現した。実API/利用者音声/利用者キーでの再現は行っていない。

## 実装担当・runnerへのコマンド

焦点core契約：

`cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --locked --test qwen_response_contract --test qwen_region_contract --test integration_credentials`

本体editor：

`cargo test --locked --bin wordweave5 app::qwen_settings_tests`

整形は担当4 Rustファイルのみ `rustfmt --edition 2021` で実施した。修正後は上記をGREEN確認し、計画に定めたpackageのdefault/no-default/windows-credentials/test/releaseと本体全体test/releaseをrunnerが実施する。root focus実行結果は `root-editor-focus.log` と本ファイルへ追記する。GREENに合わせて期待値を緩和しない。

最初のshell実行はsandboxの `apply deny-read ACLs` 構築失敗で開始できなかった。読み取り/ビルド/担当テスト整形だけ `login:false` と `require_escalated` で再実行し、承認レビューの拒否は発生していない。

## 焦点実行結果と引継ぎ

- coreの上記焦点コマンドは実装後19件すべて成功（no-default-featuresでnamespaceのWindows固有3件はコンパイル対象外）。`green-core-focus.log`。
- root初回focusは13件中12件成功/1件失敗。一般設定取消しのfixtureが保存済み `progress.settings` を直接編集していたためであり、本体不具合REDではない。一般設定の入力契約は `settings_editor.draft` であることを確認し、fixtureを修正、取消後の保存済み/draft双方復元とQwen接続保持をassertした。`root-editor-focus.log` はこの初回結果を保存する。
- fixture修正後に未設定導線2件を追加した。最新版15件は親の同時link抑制指示により再実行をrunnerへ引き継ぐ。期待値は変更していない。
- `reading-connection-focus.log` は追加所有の焦点試験結果である。consentは `#![cfg(all(windows, feature = "gui"))]` のため、最初のno-default-features実行は0件であった。`consent-editor-focus.log` のexit 0を成功証拠として数えず、GUI featureを有効にする訂正コマンドを以下に渡す。
- reading接続の2焦点コマンドは各1件成功した。新キー明示の接続変更では旧確認が失効し、別hostへの空キー保存失敗では旧確認/旧接続が保持される。実送信の代わりに架空transportで送信先と合成音声を検証した。

`cargo test --locked --test qwen_reading connection_change_invalidates_preparation_and_does_not_reload_at_send`

`cargo test --locked --test qwen_reading changed_host_without_new_key_is_rejected_and_old_confirmation_is_preserved`

`cargo test --manifest-path tools/qwen-audio/Cargo.toml --locked --test consent cancelling_settings_or_failed_atomic_save_preserves_old_connection`

本体とconsentの期待値更新は元の成功assertを削除せず、明示新キーまたはmodal取消し後の正当操作を追加した。これにより安全契約を維持しながら元の確認失効/接続保持目的を残す。

## 独立レビュー指摘への追補

親の追加割当を受け、資格情報decoder4件と画面5件を追補した。期待値はQS-AC-009/010/011/012/016と承認済み設計から導出した。

資格情報は旧Tokyo v1のliteral JSON、6地域ごとの合成JSONを直接 `decode_record(&[u8])` へ渡す。version/effort/host/key、欠落、未知field、不正JSONを拒否し `CredentialRead / NotSent` を要求する。呼出し側はstoreを渡さずOS load/saveを行わない。入力bytesの不変も確認する。ただし純粋decoder検証をWindows storeの実機読込/非移行の実証へ拡張解釈しない。

画面は親が実装する共通 `finish_qwen_settings()` に接続し、保存・変更なし取消・dirty破棄の完了とOS終了要求で編集ボタンへフォーカスが戻ることをegui frameと実widget IDで検証する。dirty破棄caseは共通完了経路を呼ぶ試験であり、nativeの「破棄して閉じる」ボタン操作を実施したとは扱わない。背景saveのpointer press/releaseを投入し、通常設定draft/保存済み設定/資格情報とmodalの保持を検証する。

親の「Cargo実行はrunnerへ一元化」指示により、追補9件は本担当で未実行である。変更前REDも採取しておらず、compile/挙動の合否は新runner実行が必要である。担当2テストファイルのみrustfmtを実施した。runnerへ渡す焦点コマンドは以下である。

`cargo test --manifest-path tools/qwen-audio/Cargo.toml --locked --lib credentials::credential_record_tests`

`cargo test --locked --bin wordweave5 app::qwen_settings_tests`

no-default-featuresではcredential_record_testsがgateで0件となるため使用しない。計画の全体/release検証を省略する意味ではなく、runnerが新差分に適用する。

## フォーカスfixtureの実操作順序への修正

独立レビューのA判断を親が採用し、保存・変更なし取消・dirty破棄・OS閉じるの4caseを修正した。診断 `runner-root-focus-diagnostic.log` は初回frameでclipが12500 points、後続frameでは1120×850相当へ縮み、初回だけでfocus要求を消費することを示した。変更前fixtureはconstructorの倍率変更直後・画面未描画のままmodal終了を作っていたため、画面上の操作で開いて閉じる利用手順と一致していなかった。

各caseは接続ページを3frame描画し、modalを操作する前にscreenが1120×850、clipとscroll viewportが有限・正サイズかつscreen内であることをassertする。その後modalを1frame以上描画してから保存/取消/破棄/OS終了要求を行う。dirty破棄は破棄確認状態も描画する。終了後の4frame、既存focus ID、focus flag消費、button全体がclip/screen内にあること、最終frameで実文字がpaintされることは維持した。

この修正は表示の期待値を下げず、試験前提となる画面実寸を先に確定するfixture修正である。productionの新sizing guardはそのまま保持し、追加の安定判定stateは編集していない。20件の件数は変わらない。Cargoは本担当で実行せず、runnerが変更後の本体焦点/全体検証を実施する。
