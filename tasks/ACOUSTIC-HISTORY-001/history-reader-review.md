# I1b-1 reader 独立レビュー

2026-10-07 / ACOUSTIC-HISTORY-001。指定reviewerはgpt-6-astra/high、実行metadata・使用量counterは未取得である。親がread-only reviewerの返却結果を保存した。

対象は`src/audio_history.rs`、`src/audio_history/{closure,results}.rs`と`tests/audio_history_closure.rs`である。対象hashは[実装記録](history-reader-implementation.md)と一致した。保存ログの34成功を確認し、fixture作成段階の初回アクセス拒否と区別した。レビュー中のファイル変更・再テストは行っていない。

## 判定と指摘

受入保留。確認済みコード欠陥はないが、P2の検証不足が1件ある。

- Storage祖先から履歴assets/leafまでのリンク・reparse拒否に専用試験がない。正常なhash・長さを持つ対象でも、Storage祖先junction、audio-history/assets junction、leafリンクからの読込が`UnsafePath`となり、対象sentinelと全bytesが変わらないことを独立Windows fixtureで確認する必要がある。作成権限がなければ未検証であり、skipを成功へ数えない。これは侵害の再現ではなく安全境界の証拠不足である。

旧revisionの閉包、容量の重複排除、上限、schema/hash/関連ID、Reference→Practice遷移、遅延結果、局所破損・ページ送り・無副作用の経路には追加の確定指摘はなかった。writer、Progress、UI、実音声/API、敵対的な並行ファイル差替えの安全証明は対象外である。

## 対応

独立テスト作者へ追加ケースを割り当てた。本体の期待値への合わせ込みは行わず、結果を取得してから指摘の閉鎖を判定する。

追加ケースの[結果](history-reader-path-tests.md)は通常権限で37件成功、leaf symlink作成だけerror 1314であった。利用者の単一ケース昇格実行の明示承認後、親がビルド済みEXEのSHA-256 `A83A8250779B0C3943CD50BF0ECBF1AF187763418AEEDEDB6D9D7B905A12B037`を照合し、`--exact windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target`だけをRunAs実行した。終了code 0、[単一ケースlog](artifacts/history-leaf-elevated-20261007.log)にokを確認した。管理者権限でビルド・全ケース実行・OS設定変更は行っていない。通常37件＋昇格1件の別実行であり、一括38件が通常権限で成功したとは扱わない。

この追加証拠を独立reviewerへ戻している。通常の一括テストへ権限要求を持ち込まないための明示別実行への整理は利用者へ確認中であり、まだテストの除外・gate変更は行っていない。

再確認結果：reviewerは本体3ファイル・テスト・凍結EXEのhash一致、実リンクと対象bytesの確認、拒否と無副作用、非再帰unlinkと失敗時保全を照合し、P2を閉鎖可能と判定した。追加指摘なし。今回のreader安全境界の検証不足は解消した。通常suiteの権限依存と今後の別実行化は別事項として残す。

利用者は権限依存1件の別実行化を承認した。独立作者がleaf1件へ理由付きignore属性だけを追加し、他37件・fixture・assertion・本体を保持した。通常focusedは37成功・1明示ignored・失敗0。新EXE SHA-256 `1A8D0ABE8DB8F08207449E9505093909A9DD23BA44BA0722B8E4408C22925613`を親が照合し、承認されたleafだけを`--ignored --exact`で再度昇格実行、終了code 0と[単一okログ](artifacts/history-leaf-explicit-elevated-20261007.log)を確認した。これで新harnessでも通常37件＋別実行1件が成功した。以降も必要な検証時にこの追加gateを実行し、ignoredを成功には数えない。
