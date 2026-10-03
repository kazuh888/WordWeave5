# 独立レビュー

2026-10-03、ww-reviewer、指定gpt-6-astra/high（実行metadata未取得）。静的レビュー時点の判定は要修正。

1. P2：Qwen編集中に通知のCodex/上限設定誘導から編集欄が隠れ、別設定を操作できる。QU-AC-011/017。入口guard、関連actionの無効化、描画分岐の保護をsettings担当へ戻した。独立test-authorへdirty draftと実操作の回帰を依頼した。
2. P2：音読errorの共通案内が、Input/Invalidatedでも存在しない「もう一度練習」を指示し、原音がない場合も再生を案内する。QU-AC-008。phase/原音有無で実在する操作へ案内する修正をreading担当へ戻した。独立test-authorへ資格情報読込失敗・入力音声不正・教材変更の回帰を依頼した。

同一authority GET、Bearer秘匿、no_redirect/no_proxy、30秒/256KiB、厳密モデル一致、固定エラー、独立保存、遅延結果破棄は追跡済み。共通40件の新ログを確認済み。原応答未保存のため実不具合の原因確定・実API改善を保証しない。

修正後再レビューと全体/native/release証拠は待ちであり、この記録は合格報告ではない。

## 修正後の限定静的確認

同じ独立reviewerが2件とも静的に解消したと確認した。P2Aの回帰は通知3経路の実clickと直接入口、dirty保持を検査する。P2Bの回帰は資格情報読込失敗・不正録音・教材変更時の実在操作を検査する。

最終固定後の5関連ファイルは `runner-final.sha256` と一致する。送信状態を状態欄先頭へ移した変更は条件・文言を維持し、action/snapshotへ影響しない。Invalidated2件の期待値は描画文字・送信0/1・旧結果無効を維持し、追加は失敗診断のみである。新規P1/P2はない。全体/native/releaseの最終実行証拠については引き続き保留である。

## スクロール到達テストの追加確認

その後のInvalidated helper変更を同reviewerが限定確認した。QU-AC-017がスクロール到達を許容し、2件だけ実MouseWheel入力でclipとscreenに完全包含されたglyph行を集めるため、不当な期待値緩和ではない。初期開示3frameと、状態・課金不明・再送なし・送信0/1・旧result/preview破棄のassertは維持する。状態hookは診断専用である。静的findingなし。変更後の最終hash更新と新規全体テストは別途必要である。

## 実行証拠の確認

reviewerが本体451件、package147件/core100件、credentials check/package releaseのログを確認した。P2A/P2Bの個別成功とInvalidated2件の成功により初回指摘は閉鎖可能である。

acceptedからnative入力への差は `visual_check.rs` 一件で、native manifestと現物のhash不一致0。`app.rs` / `main.rs` のdebug条件によりreleaseから除外される。production/testが不変のため全体テストを無効化せず、追加debug/nativeで確認することが妥当である。

音読の確認・JSON不正・別英文・狭幅末尾と接続概要の5画像を親とreviewerが実見し、新規findingなし。最新fixtureのdebug・settings/probe再画像と本体releaseの成功のみ待ちである。

## 最終判定

独立reviewerが保留2項目を確認し、実装・オフライン検証・release引渡し境界を満たすと判定した。未解決P1/P2なし。root-release/debug-v4の成功、release実物SHA256、v4 manifestと現物の不一致0を確認した。全体テスト後の差分はdebug専用tail1行とvisual_checkのみで、releaseへの影響はない。

標準v3の接続成功/認証失敗、settings-v2の入力、最小v4全3画像を親とreviewerが実見した。最小サイズの末尾画像に結果文は含まれず、標準の結果表示と最小の操作到達証拠を区別する。main.rsの既存最小820×650に基づき480stressの切れを適用外と記録することは、基準緩和ではない。

実API・実音声・利用者受入とroot全体fmtの既存不合格は未解消のままである。新UIのSkill化は利用者受入後とする。
