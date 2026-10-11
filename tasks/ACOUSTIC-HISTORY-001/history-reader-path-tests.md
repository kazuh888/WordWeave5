# I1b-1 Windows path安全境界の独立テスト

2026-10-07 / ACOUSTIC-HISTORY-001。担当はtest author、計画者指定は `gpt-6-astra` / `high`。実行model/effortと利用量の開始・終了カウンターは未取得であり、推計しない。入力は[承認・I1割当](implementation.md)、[仕様](spec.md)、[reader契約](design-i1b.md)、[P2レビュー指摘](history-reader-review.md)である。TDDとwordweave-changeを担当範囲で使用した。

所有は `tests/audio_history_closure.rs` の追記と本書だけである。本体、manifest、既存assertion、仕様、gateを変更していない。目的は既存の正しい可能性がある防御処理への不足受入証拠追加であり、不具合修正ではない。初回PASSは既存処理の独立確認として扱い、人工的なREDを作らない。作成権限不足やcompile errorをreaderの行動REDへ数えない。

## ケースと安全性

AH-AC02/03の原音・既存履歴保持とAH-AC18のhost境界を具体化したP2に対し、以下を追加した。

| test名（`windows_path_safety::`配下） | 実在fixture |
| --- | --- |
| `storage_ancestor_junction_rejects_valid_original_and_preserves_target` | Storageの一段上の親directoryを経由するNTFS junction |
| `audio_history_junction_rejects_valid_original_and_preserves_target` | audio-history directoryのNTFS junction |
| `assets_junction_rejects_valid_original_and_preserves_target` | assets directoryのNTFS junction |
| `leaf_file_symlink_rejects_valid_original_and_preserves_target` | 正しいhash名を持つ原音leafのWindows file symlink |

各caseは独立temp rootで手書き合成RAWを通常pathから先にreaderで受理する。リンク設置後にもリンク経由の実bytesを読み、期待SHA-256と長さが正しいことを検査する。そのうえで同じ参照の読込を2回行い、毎回 `UnsafePath` を要求する。Storageのapp.lockを解放してから前後の全tree bytesを比較する。neighbor sentinelも保持する。取消しは同期reader APIに存在せず、この追加範囲では非該当である。

junctionはRustから `DeviceIoControl(FSCTL_SET_REPARSE_POINT)` を呼び、`IO_REPARSE_TAG_MOUNT_POINT` の実reparse pointを作る。leafは `std::os::windows::fs::symlink_file` を使う。モック、実教材、実音声、network/API、Windows設定変更、Developer Mode変更は使わない。

cleanupは登録済みリンク自身に対する非再帰 `remove_dir` / `remove_file` を先に行い、リンク解消後にtarget全bytes不変を検査する。その後だけ既存TempRootの通常cleanupを許す。panic時もリンクを先に外し、unlinkが失敗した場合はtemp rootを残して再帰cleanupを行わない。

## 実行記録

新4件のcommandは `cargo test --test audio_history_closure --locked --offline windows_path_safety -- --nocapture` である。

- 通常sandbox: compile成功、0 passed / 4 failed / 0 ignored / 34 filtered、exit 1。junction作成または元fixtureのrenameに対するWindows error 5であり、危険pathをreaderへ渡す前の環境失敗である。
- 承認レビュー後のsandbox外: compile成功、3 passed / 1 failed / 0 ignored / 34 filtered、exit 1、test実行0.03秒。3箇所の実junctionと保持検証はPASS。leaf file symlinkだけは作成時のWindows error 1314（要求特権なし）であり、reader未到達・未検証である。承認レビュー拒否ではない。OS設定やユーザー特権を変更していない。

sandbox外の実行ログ抜粋（tool session 32400 / completion chunk 577de5）:

```text
running 4 tests
create real Windows file symlink; privilege/setup failure is not a reader RED:
Os { code: 1314, kind: Uncategorized, message: "クライアントは要求された特権を保有していません。" }
test windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target ... FAILED
test windows_path_safety::assets_junction_rejects_valid_original_and_preserves_target ... ok
test windows_path_safety::audio_history_junction_rejects_valid_original_and_preserves_target ... ok
test windows_path_safety::storage_ancestor_junction_rejects_valid_original_and_preserves_target ... ok
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.03s
```

既存34件を含む全focused command `cargo test --test audio_history_closure --locked --offline` は承認レビュー後のsandbox外で **37 passed / 1 failed / 0 ignored / 0 filtered、exit 1**、test実行7.98秒であった。既存34件と追加junction3件はすべてPASS、唯一の失敗は同じleaf作成時error 1314である。P2全件完了とは扱わず、leafは既にsymlink作成能力のあるWindows runnerでの実行を要する。全体アプリtest/release buildはこの不足AC作成の担当外であり、親のgateを維持する。

全focused実行ログ抜粋（tool session 21997 / completion chunk 274c38）:

```text
running 38 tests
test windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target ... FAILED
test windows_path_safety::assets_junction_rejects_valid_original_and_preserves_target ... ok
test windows_path_safety::audio_history_junction_rejects_valid_original_and_preserves_target ... ok
test windows_path_safety::storage_ancestor_junction_rejects_valid_original_and_preserves_target ... ok
create real Windows file symlink; privilege/setup failure is not a reader RED:
Os { code: 1314, kind: Uncategorized, message: "クライアントは要求された特権を保有していません。" }
test result: FAILED. 37 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.98s
```

既存binのdead-code warning4件は担当外として保持した。証拠は本書の関連ログ抜粋とtool実行結果であり、過去ログを新実行へ流用していない。新しい独立logファイルは作らず指定所有の2ファイルだけを変更した。

`rustfmt --edition 2021 --check tests/audio_history_closure.rs` と `git diff --check` はexit 0である。対象testは既存untrackedファイルの末尾moduleだけを追記した。以下のreader本体3本は実装引継ぎ時hashと同一である。

| 対象 | SHA-256 |
| --- | --- |
| tests/audio_history_closure.rs | E0295610746C4C27E27F6C516C145563A73A70ABEF53FD98D1EA7B0788E2E466 |
| src/audio_history.rs | C30A7EA136CE94AA3B423F09A030D9EEC37A8C7B81BC54A4BA044A6106E0CCD0 |
| src/audio_history/closure.rs | 44D8F94AE6D8D56C9743803D51F00DFEFADF11F577B8866ABFC001463192432D |
| src/audio_history/results.rs | 7BC153FF0B9288E5FD258B948E1686A7E1657FBB8104E473AB78044CC3516714 |

runnerへの引継ぎは上記command・同じfixture・同じ期待値であり、leafの唯一許されるPASSは実file symlinkの作成と全assertion通過である。環境が不足したままなら `UnsafePath` の成否は未判定である。今回、本体の変更要求は見つかっていない。

親から「このテストだけ管理者権限で実行してよい」という利用者の追加承認を受領した。昇格起動は親が所有する。担当は全38実行後の下記EXEを固定し、以後cargo/buildを実行していない。昇格実行結果は親が追加記録する。

- EXE: `F:\Kazuhiro\GitHub\WordWeave5\target\debug\deps\audio_history_closure-5710f60171e2013d.exe`
- SHA-256: `A83A8250779B0C3943CD50BF0ECBF1AF187763418AEEDEDB6D9D7B905A12B037`
- size: 3,347,456 bytes / LastWriteTimeUtc: 2026-10-07 04:00:39。
- 1件選択引数: `--exact windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target`。

## 利用者承認による特権ケースの明示実行分離

親は上記の凍結EXE（SHA-256 `A83A8250779B0C3943CD50BF0ECBF1AF187763418AEEDEDB6D9D7B905A12B037`）のleaf1件を昇格実行し、exit 0を返却した。[昇格実行ログ](artifacts/history-leaf-elevated-20261007.log)には次の成功が記録されている。

```text
ok windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target
```

これにより当該EXEのleaf fixture作成、正常hash/長さの確認、UnsafePath再試行、全bytes・sentinel保持は確認済みとなった。通常tokenのerror 1314は過去の環境制約の記録として残す。

続いて利用者が「権限が必要な1件を別実行に分ける」を承認したため、leaf1件に限り `#[ignore = "Requires Windows file-symlink privilege; run this exact test elevated"]` を追加した。fixture・assertion・他37件は変更しない。通常focusedは37 passed / 1 explicit ignoredを期待するが、ignoredをPASSには数えない。

受入に必要な追加実行は、通常ユーザーでbuildした凍結EXEへ `--ignored --exact windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target` を渡す1件である。起動・権限確認は親が所有し、自動昇格、管理者cargo/build、Windows設定変更は行わない。新EXEでの追加実行が未実施なら未実施と記録する。

明示実行分離後の `cargo test --test audio_history_closure --locked --offline` は承認レビュー後のsandbox外・非管理者で **37 passed / 0 failed / 1 ignored / 0 filtered、exit 0**、test実行8.68秒となった。tool session 49017 / completion chunk 25d36aのログ抜粋は以下である。

```text
running 38 tests
test windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target ... ignored, Requires Windows file-symlink privilege; run this exact test elevated
test result: ok. 37 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 8.68s
```

新EXEのleaf明示実行は本担当では未実施であり、旧EXEの昇格成功を新EXEの成功とは数えない。次の凍結済み成果物を親へ返し、以後buildを行わず所有を解放する。

| 対象 | SHA-256 |
| --- | --- |
| tests/audio_history_closure.rs | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 |
| target/debug/deps/audio_history_closure-5710f60171e2013d.exe | 1A8D0ABE8DB8F08207449E9505093909A9DD23BA44BA0722B8E4408C22925613 |

EXE絶対pathは `F:\Kazuhiro\GitHub\WordWeave5\target\debug\deps\audio_history_closure-5710f60171e2013d.exe`、sizeは3,347,456 bytes、LastWriteTimeUtcは2026-10-07 04:07:44である。必要な起動引数は `--ignored --exact windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target` である。

追加したignore属性1行だけを除いたUTF-8内容のSHA-256は旧testの `E0295610746C4C27E27F6C516C145563A73A70ABEF53FD98D1EA7B0788E2E466` と完全一致した。fixture・assertion・他37件は不変である。本体3本のSHA-256も上表の既存値と再一致した。整形検査は成功、担当文書のリンク・差分検査も実施した。model/effort実行値と使用量カウンターは追加対応でも未取得である。

親追記：引渡し後、上記新EXEのSHA-256を照合し、利用者が承認したleaf1件だけを`--ignored --exact`で昇格実行した。終了code 0、[ログ](artifacts/history-leaf-explicit-elevated-20261007.log)の単一okを確認した。管理者でのbuild・全suite実行・OS設定変更はない。通常37件と別実行1件は両方成功し、明示別実行の運用も確認済みである。
