# I2b-B 独立テスト作成 / AH-AC16・I2-T06

2026-10-11。担当はtest author、planner指定は `gpt-6-astra/high`。実行metadataと開始/終了usage counterは未取得であり、推計しない。入力は承認済み `spec.md` AH-AC16、`design-i2.md` I2b-B B1–3/R1–5（SHA-256 `A7A6931A63A53E6842B91DC816ED1430BAF2D3EEB3CC95F753C6226AE26EFBE0`）、`implementation.md` b-B限定delta、親受領の独立Pre PASSである。HEADは `ebac9e0`。本工程の完了境界は独立oracle・focused RED確認・実装者へのhandoffである。

所有は新 `tests/audio_history_backup.rs`、`src/store.rs::i2_backup_path_tests`、`src/backup.rs` 新規境界testと既存 `rejects_unrestorable_sizes_before_creating_destination` のwire1長さoracleだけである。本体変更は行わない。storeの署名不在を親へ報告し、8件の期待値固定後に同fileを返却した。実装者が未接続 `read_progress_for_backup(&Path) -> Result<Option<Progress>, String>` 固定Err scaffoldを置いた後、所有を受け戻した。compileエラーを行動REDとして扱わない。

使用Skillは `agent-skills:test-driven-development` のRED工程である。仕様から期待値を固定し、実Storage・実filesystem・手製JSONを用いた。ネットワーク・実学習記録・公開操作はない。

## 固定した受入

| 範囲 | 件数 | 期待値 |
| --- | ---: | --- |
| integration B1 | 14 | stale None/Bでもmemoryはcurrentの全非履歴field＋disk A、restore返却/diskはbackupの全非履歴field＋currentとの日別ai_calls最大値＋disk A。caller/source/履歴bytes不変。leafなしでcurrent rootを復活させない。pending/不正disk/closure欠損/退避先file拒否、媒体再検証前後の拒否、memory・媒体後commit拒否の残存を区別 |
| integration B2 | 5 | candidate Some/disk Some/両Someの拒否はdest作成0、全ledger不変。両None成功は明示wire1・historyキーなし、全fieldとmanifest hashを比較しprepare/restore往復。pending/不正disk拒否も確認 |
| integration B3 | 9 | 正しいmanifest hash付きroot混入v1/v2、未知progress/manifest版、重複version/root、未知field、壊れJSONの拒否。既存wire2 null入力は維持 |
| store R1–5 | 8 | v1/v2 None/v2 Someのfield保持、root不存在vsleafなし、通常strict reader/Storage/head対照、file/junction祖先とleaf、pending、strict decode、exact境界/100MB超過、実sharing-denialと再試行。全ledger不変 |
| backup上限 | 新1＋既存改訂1 | 独立wire1投影のpretty最終bytes exact-1拒否/exact成功、日本語・改行・quote/backslash escaping。memory用runtime2長さ契約は不変 |

B1の事前拒否では既存bytesとdirectory ledgerを全比較する。後半のcommit拒否は `custom.tsv` をdirectoryにし、memory保存・媒体追加が済んでからcommit旧deck読取で拒否される実境界である。旧正本/既存媒体/sourceは不変、memory1件と合成不変媒体の追加を認める。pending作成後/finish失敗ではなく、既存R/Tの中断契約を代替しない。

同期backup APIに取消tokenはない。UI確認取消はrestore呼出し前の既存 `backup_confirmation_` 回帰へ引き継ぐ。検査中のpath置換・metadata I/O不明・電源断耐久性・write-block中の外部exportは未測定であり、最後はI2d-H/Bの未完接合である。OS権限変更・追加symlink昇格は行わない。junctionは既存の非昇格合成fixtureを利用する。

## 実行手順

```powershell
cargo test --locked --lib --test audio_history_backup --no-run
# buildに表示されたEXEを固定し、process限定TEMP/TMPを担当専用pathへ設定。
# Start-Process -WindowStyle Hidden とstdout/stderr別ログ、有限WaitForExitを使用。
```

専用TEMP/TMPは `tasks/ACOUSTIC-HISTORY-001/artifacts/i2b-b-author-temp`。環境設定は実行process限定である。新integrationはRAIIでStorageを先に解放し、RED unwindでも所有fixtureを掃除する。store/新backupは既存TempのRAIIを用いる。

## RED結果と環境の分離

`--list`でintegration 28件、store新module 8件、backup 6件（新1＋既存5）を確認した。**重複を除く42件の到達結果は25期待RED・17PASS、ignored0、timeout0**である。複数の実行環境とbuildを組み合わせた作者証拠であり、同一最終EXEの全件成功結果ではない。

| 実行 | 実行数・結果 | 失敗理由・到達境界 |
| --- | --- | --- |
| 初回integration、sandbox、PID34468 | 28: 9PASS/19FAIL | 17件は期待する行動RED、2件は媒体fixture `AssetStore::put` がerror5で未到達 |
| 同一integration EXE、限定sandbox外、PID30064/30892 | 2: 1PASS/1RED | source媒体再検証はPASS。late commit拒否は旧正本全保全・媒体追加・memory1件を通過し、memory root B≠AでRED |
| 初回store、sandbox、PID1964 | 8: 1PASS/7FAIL | 4件scaffold RED、2件は手製RAWの必須memo欠落、1件はjunction作成error5。fixture失敗をREDへ含めない |
| 同一初版lib EXE、限定sandbox外、PID30412 | 1PASS | R03はfile/junction祖先とleafの全配置を通過。junction作成は非UAC・既存fixture、OS設定変更なし |
| memo追加後store、sandbox、PID6840 | 7: 2PASS/5RED | R03だけ実行filterで除外。5件は固定Err scaffoldが正常読取/不存在/読取retryを拒否するためRED。invalid-document等のfixture失敗は0 |
| backup6件、sandbox、PID7744 | 6: 2PASS/4FAIL | wire1 exact長さの新1＋改訂1は行動RED、旧媒体2件はfixture error5 |
| 同一初版lib EXE、限定sandbox外、PID1932/33280 | 2PASS | 旧媒体2件がPASSとなり環境依存を確認 |

integrationの期待REDはmemory latest rootの2件、restoreの2件、leafなし1件、事前拒否6件、後半memory root1件、candidate/disk rootのexport拒否3件、wire1 export1件、pending等export拒否1件、wire2非null root prepare拒否1件である。restore2件ではdiskが期待するA＋backup draft＋ai_calls最大値であることを先に確認済みだが、memoryと返却値は古いrootのため両方falseである。

storeの固定Errによる5REDは正の契約を実証するためのREDであり、拒否条件が完成済みという意味ではない。R1の後続wireケース、R2のstrict対照、R5の後続100MB境界などは先行assertで止まるため、全分岐の到達はGREEN後に独立runnerが確認する。R03はmemo訂正の影響を受けない同一testソースで成功済みであり、修正後EXEで再実行していない。

環境比較は親の限定許可を受け、`require_escalated` の通常processとして同一EXE・exact caseだけを実行した。新しい専用TEMPは `tasks/ACOUSTIC-HISTORY-001/artifacts/i2b-b-author-external-temp`。管理者cargo/build、UAC、実データ、設定変更は使用していない。junctionはfixtureのDeviceIoControl失敗とerror5がログで分かる。媒体のerror5は `AssetStore::put` 内で文字列化され、原syscallの識別は未取得である。hard_linkを原因と断定しない。sandbox外で媒体追加まで成功した事実と区別する。

初回integrationのcreated/removed markerは28/28、retained0。限定external integrationも各1/1・retained0。既存backupテストがpanic後に残した合成directory3件は、絶対pathが作者TEMP内であることを確認して削除した。名前は `ww-backup-1791717801758866500`、`ww-backup-limits-7744-1791717802083502500`、`ww-backup-self-7744-1791717802000254900`。最終時点でsandbox/external専用TEMPの子要素はともに0。空の専用親directoryだけ残している。

## 固定hashとhandoff

初回buildは2分11秒で成功、memoだけのfixture訂正後lib buildは1分22秒で成功した。既存のdead-code、incremental cacheのhard-link fallback警告はあるがcompile errorは0。作者の期待値は失敗を通すために変更していない。初回RAWの必須memo補完だけがfixture訂正である。

| 対象 | SHA-256 |
| --- | --- |
| 最終 `src/backup.rs`（本体未変更） | `13E92122AD30CA917096F0C789F7283CC5683F83D7C10545E396AE25B9B2114B` |
| 最終 `src/store.rs`（実装者のfixedErr scaffold含む） | `AEC126A40429C16B3F545EDEE5B95F75B76633945E5C442624B2168228FCED19` |
| `tests/audio_history_backup.rs` | `B907A3F6BCAF195000669CC2B7F954FC4323AC7832B0CDE5A8DE63B3B76CEE0F` |
| 初版lib EXE `wordweave5-b3388a7bbef44bb4.exe` | `6C93E46E4EADEB7CA16214D45C9C85F1B0B2DBACC0E28740626D2DE436421118` |
| memo訂正後lib EXE（同path） | `7E5B525322E1B9F76C1CAC72D4A15D02699A792EBD9601990E137DA75A760DBE` |
| integration EXE `audio_history_backup-bb0141422dff8e3c.exe` | `CF66496A110DDE7965BB8BAF825EDE6FF4526CCD75FCB82F09DD3D23EF573BA9` |

`git diff --check` 成功。rustfmtは所有3ソースを整形し、backupの本体差分なし、storeは実装者のscaffold＋専用moduleだけである。親のtask文書編集は保持した。

作者時点のstore新moduleとbackup test module全文の固定snapshotは `logs/history-backup-author-oracles.txt`、SHA-256は `5A93B942A445AC7A7E693C10AFB9F160A38136DB897F6CFB2B333B857D3F3A40` である。実装後のoracle不変確認に使える。integration全fileは上表のhashで固定する。

source所有を親経由で実装者へ返却済み。実装者はbackup本体と既定store橋渡しだけを変更し、新37件＋旧改訂1件の期待値を維持する。後続focused commandは以下である。

```powershell
cargo test --locked --test audio_history_backup
cargo test --locked --lib store::i2_backup_path_tests
cargo test --locked --lib backup::
cargo test --locked --lib store::i2_progress_tests
cargo test --locked --test audio_history_progress
cargo test --locked --bin wordweave5 i2_history_main_consumer
cargo test --locked --bin wordweave5 backup_confirmation_
cargo test --locked --bin wordweave5 json_restore_preserves_the_latest_unsaved_chat_draft
```

新scopeのexact実在名は下記listログで固定した。既存reader/bin回帰は作者が反復実行せず、独立runnerへ渡す。sandbox内では媒体publish/junction fixtureが拒否されるため、その失敗を製品不良/成功と数えず限定実行の環境条件を明示する。I10全suite/release buildと独立runner/Postは未実施である。

## ログ

全て `tasks/ACOUSTIC-HISTORY-001/logs/` の作者所有ログである。

- `history-backup-author-build.log` / `history-backup-author-build-memo.log`: 初回・memo補完後build。
- `history-backup-author-integration-list.stdout.log` / `store-list.stdout.log` / `backup-list.stdout.log`: 初版実在一覧（後二つも `history-backup-author-` prefix）。
- `history-backup-author-integration-red.stdout.log` / `.stderr.log`: 初回28件、環境失敗を含む。
- `history-backup-author-store-red.stdout.log` / `.stderr.log`: 初回8件、memo/junction fixture失敗を含む。
- `history-backup-author-backup-red.stdout.log` / `.stderr.log`: backup6件、環境失敗を含む。
- `history-backup-author-external-{media-recheck,late-refusal,junction,legacy-media,legacy-assets}.stdout.log` / `.stderr.log`: 同一EXEの限定5ケース比較。
- `history-backup-author-store-final-red.stdout.log` / `.stderr.log`: memo補完後7件。
- `history-backup-author-store-final-list.stdout.log` / `.stderr.log`: 最終module8件の実在一覧。

## B-WB1 追加test-only（2026-10-11）

独立reviewerのP2「保存停止時のbackup入口直接証拠不足」に対応した。入力は `history-backup-review.md` 末尾の唯一の追加契約、package SHA-256 `6B3AD46AAB6114E359B58985DC02A26B379E7B1CFFFA93EE709873B526363F81`、親受領の限定Pre PASS/阻止0である。担当/Skill/model指定は前節の独立作者を継続し、実行metadata/usageは未取得である。

追加は `store::i2_backup_path_tests::b_wb1_write_block_refuses_both_backup_entries_without_parent_contamination` の1件だけである。本体・既存8case・backupのcfg(test)・integrationに変更はない。追加区間を除いたstore全文のSHA-256が着手前と一致したことを確認し、他の差分を混ぜていない。

- before store: `91DF6C7CC92ABF8DEB2800C08B6B3C47D02F5F6C089ECCE89484FB05CDBB5118`
- after store: `516F5DA0E8C6CFDAB76BACAC9578BD477BBE85852A908534C2E5FCCB0F2C2EA9`
- backup不変: `12A20A9E5C509574B078A77C47E6B836A112A81EAAD0B8419CDC8E028DC84261`
- integration不変: `B907A3F6BCAF195000669CC2B7F954FC4323AC7832B0CDE5A8DE63B3B76CEE0F`
- 追加oracle snapshot: `logs/history-backup-b-wb1-oracle.txt`、SHA-256 `C3892613E87980E0F39AF9D03D3DA101EC8B880388D7E85410815EE450D73A2F`

子processはvalid None-historyの手製v1 diskと別の未保存日本語draftを持つcurrent、未作成の別destを用意する。guardを取得してblockを設定し、guard解放後にmemory/exportを順に直接呼ぶ。各Errの直後に全file/directory ledger・全current値不変、destなし、guard再取得がWriteBlockedであることを確認する。外部exportの拒否はb-Bの暫定契約であり、後続I2d-H/Bの外部export契約を制約しない。

親は実行前後のgate正常を確認する。既存store試験の方式で同一EXEのexact caseだけを子へ渡し、子timeout15秒でkill/reapする。子exit成功に加え、`running 1 test`、`1 passed; 0 failed`、固有 `WW5_BACKUP_WRITE_BLOCK_CHILD_OK` を必須とした。子の合成fixtureを解放した後にmarkerを出し、親も子fixtureの残存0をassertする。解除APIは追加していない。

`cargo test --locked --lib --no-run` のみbuildし、session_id `6919`、exit0、1分20秒で終了した。追加exactの `--list` は1件、list exit0（PID9156）。実行はsandbox内・`Start-Process -WindowStyle Hidden`、外側のwaitは30秒である。親test **1PASS/0FAIL/0ignored、288filtered、0.24秒**、内部子test **1PASS/0FAIL/0ignored、288filtered、0.01秒、child exit0、timeout=false**。これは1個の受入testが子へ自己委譲した結果であり、独立した2ケースと数えない。outer shell exit0、親native exit/PIDの表出力はPowerShellの異なる列の整形で保持できず未取得である。親の1件成功summaryと子のexit/markerはrawログで確認した。

実行前後のlib EXE `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe` SHA-256は `3CC77348194704FE29E1D9969B02683DF6CC70C858CFD9A2E0E57CCA37189CF0` で一致し、store hashも一致した。担当専用TEMP/TMPは `artifacts/i2b-b-author-wb1-temp`、最終子要素0。`git diff --check -- src/store.rs` 成功。build/testによる既存warning以外にcompile errorやfixture失敗はない。

本追加は実装後の検証欠落を埋めるtest-only対応であり、初回PASSである。製品の不具合再現REDとは扱わない。本体を壊すmutation、期待値の緩和、既存成功suiteの反復は行っていない。追加case成功を親へ報告しsource所有を返却済み、独立reviewerのClosed判定は未了である。

ログは `history-backup-b-wb1-build.log`、`history-backup-b-wb1-list.stdout.log` / `.stderr.log`、`history-backup-b-wb1-test.stdout.log` / `.stderr.log`。子のraw stdout/stderrは親testのstderrログに含む。後続の再現は `cargo test --locked --lib store::i2_backup_path_tests::b_wb1_write_block_refuses_both_backup_entries_without_parent_contamination -- --exact --nocapture --test-threads=1` である。
