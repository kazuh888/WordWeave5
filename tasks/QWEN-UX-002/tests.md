# QWEN-UX-002 独立テスト

2026-10-03。test-author の計画者指定・起動指定は `gpt-6.1-sol / high`。返却実行metadata、開始/終了tokenカウンターは未取得である。AGENTS、development-team、KPI、`agent-skills:test-driven-development` を全文読了した。承認入力は親のQU-AC-001～018と通信・分類契約であり、完成UIの利用者受入とは区別する。

## 共通テストの所有・入力

所有は `tools/qwen-audio/tests/provider.rs`、新規 `connection_probe.rs`。親の追加割当で `feedback.rs` と `qwen_response_contract.rs` の分類期待値のみ更新した。production、実資格情報、実API、利用者音声を変更/使用しない。既存dirtyを保持した。

| 受入ID | コマンドの対象 | 独立ケース・fixture |
| --- | --- | --- |
| QU-AC-001/003 | `--test provider` | 適合mismatchの推定英文保持・発音助言空、正常既存SSE、JSON Object要求、json_schema要求なし |
| QU-AC-002/003/018 | `--test provider` | 正しいSSE内の壊れたJSON/コードフェンスはResponseJsonInvalid、欠落/区分矛盾はResponseFieldsInvalid。SSE/途中終了/キー反射は既存ResponseInvalid。SafeError厳格3項目往復、message改ざん/原文field追加拒否、再送なし |
| QU-AC-003 | `--test feedback --test qwen_response_contract` | 6必須項目・型・不整合・文字/配列/JSON上限の既存期待値を保持し、検出段階に応じたコードへ分類更新。JSON nullは構文適合だがshape不適合 |
| QU-AC-012/013/016 | `--test connection_probe` | 6地域の元Workspace authority、公式path/完全一致model/page上限、1回のみ、request Debug秘密非露出、入力Connection保持 |
| QU-AC-014 | 同上 | 対象あり/空/別名/対象なし、success false、必要構造欠落・型不正・不正JSON、matching entryと不正entry混在、401/403/404/405/501/302/307/429/500分離、raw body非露出 |
| QU-AC-013/015 | 同上 | 256KiBちょうど/1byte超、chunk蓄積、通信/TLS/body切断、送信前取消し通信0、header/body待機後取消し、30秒deadline、全ケース自動再送なし |

全fixtureは架空キー/架空Workspace/メモリ内JSONと合成WAVである。POST評価は既存 `tests/support/provider.rs` を再利用、probeは同テストファイル内の制御可能な境界fakeである。実HTTP verb・Bearer・bodyなし・no_redirect/no_proxyは公開fake APIに観測点がないため、ReqwestProbeTransport静的確認と自動endpoint試験を区別する。TLS/実Workspace到達性の証明ではない。

## RED・実行の引継ぎ

過去に失敗した原応答は保存されていないため、報告された実MP3のResponseInvalidを直接再現できない。今回のfixtureは段階別の合成再現であり、実障害の原因/JSON Objectによる解消を証明しない。

親の指示でCargo同時実行を避け、コマンド実施は親/runnerへ集約する。test-authorは担当4ファイルにrustfmtを実行済み、焦点Cargoは未実行である。共通productionの同時編集により新error型は既に到着したため、pre-fix RED観測済みとは記録しない。

焦点コマンド:

```text
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --locked --target-dir target/qwen-audio --test provider --test feedback --test qwen_response_contract --test connection_probe
```

作成直後のprobeソースと承認契約の差は、success:false（期待Provider、現ResponseInvalid）とredirect302/307（期待Unsupported、現Provider）である。これらはfixture/compile失敗ではなく挙動差のRED候補であり、親の焦点実行で確定する。期待値は修正コードに合わせて緩和しない。

親の新規焦点実行で共通4test対象40件（probe9、feedback7、provider15、contract9）は合格した。ログは `core-focused.log`。pre-fix RED実測はなく、core/API到着後の独立fixture回帰である。

## appテストの所有・引継ぎ

親の追加割当は `src/app/qwen_settings_tests.rs`、`qwen_reading_tests.rs` のみである。両production所有者とAPI/観測IDを調整し、同居production節は編集していない。担当ファイルにrustfmtを実行済み。

| 受入ID | ケース・fixture |
| --- | --- |
| QU-AC-010/012 | 実editor検証＋架空store：変えたhostと空キー/地域不一致/header不正/未設定キーは通信0、旧保存接続を保持 |
| QU-AC-010/013/015 | 未保存draftのprobe成功はstore保存0・baseline保持、明示保存でのみ切替。保存時に再probeせず確認状態を保存しない |
| QU-AC-015 | 連打は1送信、fakeを完了可能にして取消しても成功採用なし、host/key/region編集で完了状態を失効、失敗後の再試験は明示時だけ2送信目 |
| QU-AC-010/011/015 | 保存失敗/dirty closeで進行probeを失効し、遅延成功不採用・draft/旧保存保持。既存破棄guard/focus回帰を維持 |
| QU-AC-009/011/017 | 実AI接続page内のinline入力へscroll＋keyboard文字入力、input enabled、一般transaction保持、通信/保存0。既存一般saveクリック抑止はinline説明へ更新 |
| QU-AC-004/005/006 | 上下注記順と1回表示、初期詳細閉、実ヘッダclickで6形式/Hzを展開、確認preview ID/音声保持、例文→準備→状態の順 |
| QU-AC-007/008 | Input/Confirming/Running/Completed/mismatch/unassessable/Failedの主操作。CancelはRunningのみ、Closeは全状態。録音破棄/再生停止は別操作 |
| QU-AC-017 | 820×650の80/100/125/160%でCloseの画面内可視、実wheelでSendへ到達。既存480×640の確認phaseは重複Cancelなし・Close可視へ更新 |

設定probeのfixtureはメモリ内statusキューとSemaphoreによる手動完了、real editor/store/URL検証である。3秒のローカルworker終了期限を設け、ネットワークや固定sleepによる成否判定は使わない。native focus/OS入力そのものの証明ではない。

app焦点コマンドを親へ渡し、test-authorからCargoは起動していない:

```text
cargo test --locked --bin wordweave5 qwen_settings_tests
cargo test --locked --bin wordweave5 qwen_reading_tests
```

app新規ケースはproduction/API到着後のため、pre-fix REDを観測したと主張しない。親runnerの新規実行結果と不具合/fixture/compile失敗の分類を待つ。全体試験・release・native画面は親/runnerの責任であり、このテスト作成記録だけでQU-AC-017/018の最終受入を宣言しない。

## 独立レビューP2回帰

親からレビュー指摘2点と修正所有者を受領し、担当appテストだけに2ケースを追加した。productionと親のCargo進行へ介入しない。

- QU-AC-011/017：Codexの上部operation notice、Codex通知window、日次上限通知windowの3経路で、画面・clip内の実actionへpointer press/releaseを送る。Qwen編集のkey、一般draft、保存接続、progress、通知openを保持し、direct `open_codex_path_guidance` もguardされる。以前のguidance flagがtrueでもQwen入力がenabledでscreen/scroll viewport完全内に残ることを要求する。
- QU-AC-008：CredentialRead/Input（原音なし）、不適合録音/Input（原音bytes保持）、Invalidatedの3状態で、存在しないRestartへの案内を拒否する。原音がない場合は次操作へ原音確認を出さず、実際の設定/録音・選択/閉じるへ案内する。すべて送信0である。

初期REDの未取得を保持する。新規回帰は親集中runnerへ引き渡し済み、test-authorによるCargo実行はない。既存Invalidatedの句点assertは現在notice文に原文が残るため変更していない。rootが新しい対象差分で焦点結果と必要全体検証を確定する。

## 描画失敗の切り分け

親の `app-reading-focused.log` は20件中18合格、既存Invalidated2ケースの送信状態文言がpainted textにないため失敗した。compile/fixture構築エラーではなく、render観測の失敗である。期待文言やviewのNotSent/MayHaveBeenSentは変更していない。

独立読取で、`invalidate_target` はInvalidState error、`check_target` は対象変更noticeを保持し、旧UIは状態→error group→notice→送信状態の順に描いていたことを確認した。後方labelがscroll clip外でText shapeを出さない可能性があるが、失敗ログには座標がなく、clip原因は未確定である。rootは送信状態を状態欄冒頭へ移動した。

test-authorは helper に各frameのscreen/body/footer rect、送信状態branch hook、shape数、painted文字数を追加した。2assert失敗時にはその診断と合成painted全文を出す。hookあり/paintなしはbranch未実行と抽出対象外を切り分ける補助であり、hookだけで表示合格としない。担当2ファイルを整形・freezeし、runnerへ最新ソースの実行を引き渡した。

再試験 `app-invalidated-final.log` は先行する対象変更noticeのassertで失敗した。送信状態assertへ未到達であり、その通過を主張しない。test-authorが保存した後にモデル容量不足で終了したため、親が保存済み差分を確認・引き継いだ。

QU-AC-017はスクロールによる到達を許容するため、Invalidated2件のみ実wheelでbodyを読み進め、screen/clip内に完全に収まる描画glyph行の文字を集約する。初期開示の試験は従来3frameのままである。期待する状態文言、禁止文言、送信0/1、旧結果無効は維持する。first assertにも診断を追加した。実行結果と独立レビューで妥当性を確定する。
