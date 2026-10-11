# I2a-V 独立実行結果

2026-10-08 / ACOUSTIC-HISTORY-001。planner指定 `ww-test-runner / gpt-6.1-sol / medium`を維持した。実行metadata・開始/終了usage counterは未取得。AGENTS/development-team/KPIとwordweave-changeの検証・報告責務を適用した。入力はhistory-publication-authority-{tests,implementation,review}.md、V08限定訂正後固定source/lib EXE、implementation-final/exes-before.logの無変更integration EXEである。

HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`＋未commit差分。source2/test2/EXE3の7全文SHAとV moduleの計8 hashは引継ぎ固定値と一致し、前後不変。[前hash](artifacts/history-publication-authority-runner-before.json) / [後hash](artifacts/history-publication-authority-runner-after.json)。store `224F2E6C…92B278B`、V module `9C4FBC33…830D019`、lib EXE `9CDB2D4D…32BD3C8`を使用した。untracked storeはbefore snapshotとの差分を保存した。[前patch](artifacts/history-publication-authority-runner-before.patch) / [後patch](artifacts/history-publication-authority-runner-after.patch)はともにSHA-256 `AE4592FE14E7D35652FBF98CFED8A0F0570E842B7627EF0908CB353355B104FA`。

非Administrator tokenを確認し、固定3 EXEのみsandbox外で実行した。全4群の同filter＋`--list`はexit0、旧lib11/新V19/store37/recovery34件と一致した。`history-publication-authority-runner-*-list.log`と[summary](artifacts/history-publication-authority-runner-summary.log)へ保存した。実行区間は2026-10-08 16:14:22.6475900–16:20:01.8655570 JST。外側900秒は開始時から固定し、自動延長なし。製品60秒/512MiB、品質assert、V child15秒は不変である。

repository rootから以下を新規独立実行した。旧成功・作者V08 exactログを代用せず、V08 exactの重複実行も行っていない。

| コマンド | exit | passed / failed / ignored / filtered | outer timeout | 外側実測秒 | 新stdout / stderr |
| --- | --- | --- | --- | --- | --- |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history:: --skip i2_publication_authority_tests --nocapture --test-threads=1` | 0 | 11 / 0 / 0 / 250 | False | 0.57 | [旧lib](artifacts/history-publication-authority-runner-old-lib.log) / [stderr](artifacts/history-publication-authority-runner-old-lib.stderr.log) |
| `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history::store::i2_publication_authority_tests:: --nocapture --test-threads=1` | 0 | 19 / 0 / 0 / 242 | False | 3.09 | [新V](artifacts/history-publication-authority-runner-new-lib.log) / [stderr](artifacts/history-publication-authority-runner-new-lib.stderr.log) |
| `target/debug/deps/audio_history_store-e2ea7a2b1774fcfc.exe --nocapture --test-threads=1` | 0 | 37 / 0 / 0 / 0 | False | 10.35 | [store](artifacts/history-publication-authority-runner-store.log) / [stderr](artifacts/history-publication-authority-runner-store.stderr.log) |
| `target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe --nocapture --test-threads=1` | 0 | 34 / 0 / 0 / 0 | False | 324.76 | [recovery](artifacts/history-publication-authority-runner-recovery.log) / [phase/cleanup](artifacts/history-publication-authority-runner-recovery.stderr.log) |

計101 passed / 0 failed / 0 ignored（旧82＋新V19）、fixture failure/child timeout/outer timeout0。filtered・exact child内部件数は成功へ加算しない。新V全loop/後段assertionが通過した。V08はextra0の8MiB正常対照とextra1の合法disk base先行確認、両confirmの厳密LimitExceeded、各末尾ledgerを含む。V04の実read32＋解除後成功mode2/3は新stderrへ記録した。junction・cleanup retry・held guard・block/poison後pure confirmも成功した。意図的catch対象panicをfixture failureへ加算しない。

R2は100,000件生成104.288秒、先頭/末尾page成功158.545/184.176秒、100,001超過診断235.837秒、cleanup完了316.881秒まで通過した。これはinventory entries境界であり、未測定V closure reader 100,000実nodesの証拠ではない。初回fixture失敗・旧outer timeoutの既存ログは保持し、本結果へ混ぜていない。

`rustfmt --edition 2021 --config skip_children=true --check src/audio_history/store.rs src/audio_history/recovery.rs tests/audio_history_recovery.rs tests/audio_history_store.rs`はexit0。[fmt](artifacts/history-publication-authority-runner-fmt.log)。`git diff --no-index --check -- tasks/ACOUSTIC-HISTORY-001/artifacts/history-publication-authority-before/src__audio_history__store.rs src/audio_history/store.rs`はexit1、[診断log](artifacts/history-publication-authority-runner-diff-check.log)は0 bytesである。no-indexの差分存在exit1であり、空白不備診断はない。source/EXEを変更していないため成功済みチェックは反復していない。

通常実行のactualTempは`F:\TempCodex\`。[前](artifacts/history-publication-authority-runner-temp-before.log)/[後](artifacts/history-publication-authority-runner-temp-after.log)の広い`ww*`既存directory集合は558→558、新OwnedTemp0。今回関連prefix `ww5-recovery-tests-*` / `ww5-writer-tests-*` / `ww5-i2-gate-*`はそれぞれ前0/後0。試験自身のcleanup以外は削除せず、既存558件を保持した。[終了status](artifacts/history-publication-authority-runner-status-after.log)を保存した。

担当報告・新ログ・一意合成fixtureのみ書込み、source/test/期待値/script/OS設定・他者編集は変更していない。実data/API/UAC/ACL変更・公開・再委任なし。source/build所有を親へ返却する。指定独立自動検証は成功である。最終head/intent再読中置換・pending metadata Io・新規実closure100k nodesは未測定のまま。P/UI/I10全体cargo test/release、native microphone/pen/TTS/IME/authentication・電源断受入は未実行であり、全体受入完了とはしない。
