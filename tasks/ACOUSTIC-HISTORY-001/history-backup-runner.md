# I2b-B 独立runner / AH-AC16・I2-T06

2026-10-11。最終固定sourceから4EXEを再生成し、新scope43＋既存回帰33＝**76 unique PASS**を新規採取した。最終行動不合格0、ignored0、timeout0、対象未実行0である。初回readerのjunction setup失敗1件（exit101）は下記に保持し、成功に書き換えていない。限定exact再実行後の到達結果である。

planner指定は `gpt-6.1-sol/medium`、理由は固定差分・固定oracleの手順実行である。`implementation.md` b-B限定delta、`history-backup-review.md` runner-ready、`history-backup-tests.md` と `history-backup-implementation.md` を入力とした。AGENTS.md、development-team.md、KPIを読み、wordweave-changeは保存/復元リスクに対する検証・結果報告だけへ適用した。実行model/effort metadata、開始/終了usage counterは未取得であり推計しない。親が同taskの開始/現状を記録する。

## 固定対象

HEADは前後とも `36f1b7081fc9b570c65d30360463d55cbd3862af`。着手時 `git status --short` は空であった。終了時の既存変更は親所有 `tasks/current.md` と `docs/process/improvement/measurements.md` のみであり、source/tests/Cargo.toml/Cargo.lockのdiffは0。本報告、runnerログ、隔離artifact、build生成物だけを書いた。source/期待値/設定の変更、commit、公開、実データ/API、アプリ停止は行っていない。

前後SHA-256は全7対象で一致した。原記録は `logs/history-backup-runner-before-hashes.json` と `logs/history-backup-runner-after-hashes.json`。

| 対象 | SHA-256 |
| --- | --- |
| src/backup.rs | `12A20A9E5C509574B078A77C47E6B836A112A81EAAD0B8419CDC8E028DC84261` |
| src/store.rs | `516F5DA0E8C6CFDAB76BACAC9578BD477BBE85852A908534C2E5FCCB0F2C2EA9` |
| tests/audio_history_backup.rs | `B907A3F6BCAF195000669CC2B7F954FC4323AC7832B0CDE5A8DE63B3B76CEE0F` |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe (lib) | `0046675468D2B02041DC7521C9C54DFC7050DBD4F2BE3AF99438FA2A46A60187` |
| target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe (bin test) | `F00550DF3B397BE7E4D3AB989F296F862F01848F2B68D0CD34984599ED0C03A3` |
| target/debug/deps/audio_history_backup-bb0141422dff8e3c.exe | `18AA734E38ADA5F78D20E4535D1B31C107AD4396DBF2C2252B2253D3D37CF868` |
| target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe | `92C106849BD0855B6195C90D761BEA9EB9B1E358C4EFEA3FE9D8165BC4EBA808` |

## 実コマンド・結果

`cargo test --locked --lib --test audio_history_backup --test audio_history_progress --bin wordweave5 --no-run` はsession_id **2424**を保持してwrite_stdinで完了まで回収、cargo表示3m51s・native/outer exit0であった。buildログは `logs/history-backup-runner-build.log`。既存dead-code、incremental hard-link fallback、float literal fallback警告があり、compile error0。buildを重ねず、完了前にEXEを実行していない。

実行手順は `artifacts/history-backup-runner.ps1` に保存した。各jobはStart-Process Hidden、stdout/stderr別ログ、60秒WaitForExit、process限定TEMP/TMPを用いる。`-Mode List` (exit0) で8scopeの実在名/件数28・9・6・11・4・16・1・1を照合した。各 `logs/history-backup-runner-list-*.stdout.log` が全名を固定する。

`-Mode Normal` はreader setup失敗でouter exit1になり停止した。既に成功したscopeは反復せず、`-Mode Normal -From progress` (exit0) で後続だけ実行した。`-Mode External` (exit0) は既知媒体/junction制約7件のexactだけを通常sandbox外で実行した。同一4EXE・同一隔離TEMPを用い、UAC・OS設定変更・管理者cargoはない。

| scope | 通常PASS | 限定外PASS | unique | native PID / exit |
| --- | ---: | ---: | ---: | --- |
| audio_history_backup | 26 | 2 | 28 | 24132/0; 1352/0; 33688/0 |
| store::i2_backup_path_tests | 7＋B-WB1 1 | R03 1 | 9 | 6648/0; 3544/0; 18540/0 |
| backup:: | 4 | 2 | 6 | 28644/0; 26272/0; 30568/0 |
| store::i2_progress_tests | 10 | 1 | 11 | 608/101 (setup); 7776/0 |
| audio_history_progress | 4 | 0 | 4 | 32204/0 |
| i2_history_main_consumer | 16 | 0 | 16 | 14232/0 |
| backup_confirmation_ | 0 | 1 | 1 | 32812/0 |
| json_restore_preserves_the_latest_unsaved_chat_draft | 1 | 0 | 1 | 34432/0 |
| 合計 | 69 | 7 | 76 | 全timeout=false |

全対象名と成功名の集合差は0。`logs/history-backup-runner-summary.json` に76実名、全summary、残存数を保存した。対象内で恒久skipしたcaseは0、ignored0。scope外のfiltered件数は個別summaryに保持した。親testの試行は77（76成功＋junction setup失敗1）、B-WB1内部子は同じ1受入の再委譲なのでuniqueを加算しない。

初回reader失敗の原因は、runner artifactで既知除外名を旧m03と誤記し、実在 `store::i2_progress_tests::s03_unsafe_and_missing_paths_never_become_empty_history` を除外できなかったことである。原stderrはjunction fixture DeviceIoControlのerror5を示す。対象は行動境界未到達であり製品不合格とは分類しない。artifactの実行filterだけ訂正し、当該exactの限定外PASSを採取した。成功10件を再実行せず、ソースや期待値には触れていない。

B-WB1は専用exact processで親PID **3544 / exit0 / timeout=false**、1PASS。stderrに **`B-WB1 child status=exit code: 0 timeout=false`**、`running 1 test`、1PASS、**`WW5_BACKUP_WRITE_BLOCK_CHILD_OK`** を確認した。親gate前後正常・子fixture残存0のassertも通過した。親子はunique1件である。

## ログ・残余・handoff

全て新規 `logs/history-backup-runner-*` であり、過去の作者/実装ログを今回の結果へ流用していない。各jobの `.result.json` はEXE/引数/PID/exit/timeout/開始終了/ログpathを保持し、表整形による列欠落を避けた。`.stdout.log` / `.stderr.log` がraw結果である。限定外exact名もresult JSONに保持した。

TEMP/TMPは `F:\Kazuhiro\GitHub\WordWeave5\tasks\ACOUSTIC-HISTORY-001\artifacts\i2b-b-runner-temp` にprocess限定した。終了時直下残存0、cleanup追加削除0。実データを用いていない。`git diff --check` exit0、source/tests/manifests/lockのdiff0を確認した。

本単位の独立自動検証は完了であり、次は固定source/EXE・本報告/新rawログを独立最終Postへ渡す。I10の `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、製品EXE配布、native IME/マイク/ペン/TTS/実Codex認証は未実施である。実path置換競合/metadata I/O不明/電源断耐久性、write-block中の外部export成功（I2d-H/B）も未検証・後続範囲として残す。既存mock/合成fixture成功を新native受入へ読み替えない。
