# I2d-B 独立runner結果

2026-10-12。担当`backup_destination_runner`、計画・起動指定は`gpt-6.1-sol/medium`である。理由は固定対象51件の手順実行と独立証拠採取であり、`implementation.md`のd-H/B割当と一致する。実行model/effort metadata、開始/終了usage counterは未取得であり、推定しない。AGENTS.md、development-team、KPI、wordweave-change全文を読み、skillはrisk-based検証・報告だけへ適用した。

入力は[作者引継ぎ](backup-destination-tests.md)、[実装引継ぎ](backup-destination-implementation.md)、[review](backup-destination-review.md)である。受入はAH-AC04/16・I2-T08/T08b・H4/H5。完了境界は同じsourceからの独立build、integration全38＋lib backup::全13の有限実行、native exit/子wait/fixture残存の確認である。ソース、テスト、期待値、設定は変更していない。再委任・commit/push・publish・利用者appの停止はない。

## 固定対象

開始・終了HEADは`b063591f78a6f3625203f8758b647386c927522f`。開始・終了とも対象source2本のhashは以下に一致した。終了時の`git diff b063591f78a6f3625203f8758b647386c927522f -- src tests Cargo.toml Cargo.lock`は空である。並行した親の`tasks/current.md`と`docs/process/improvement/measurements.md`編集は保持した。

| 対象 | SHA256 |
| --- | --- |
| src/backup.rs | CF0355486CB563400485ABE4771176D50C37F13C1A54141D9BF6AF65E77784F7 |
| tests/audio_history_backup.rs | 1BCA808ED56BE0DB8CCE33F9BE4BCD5899A68017822342DFF71AB32A0750E23B |
| target/debug/deps/audio_history_backup-bb0141422dff8e3c.exe | EAA3EB02FDF97A59A2D96432941C3FF260A08394C027C373641D65CC34B5E066 |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | 5F05B4942877C974E83533D3B323F6925CD3E1B60A6855E9F2169C47F9FB24E4 |

EXEは今回のcargo JSON compiler-artifactのexecutableから同定し、実行直前にhashを固定照合した。実装者の旧EXE/hashを今回のbuild成果と読み替えていない。

## 今回の実行

開始buildは09:48:59 JST、最終確認09:52:12 JSTである。TEMP/TMPは当該processだけ合成fixture用directoryへ指定した。永続環境設定・RUSTFLAGS・target-dir・依存・OS権限は変更していない。

| コマンド | native結果 | 証拠 |
| --- | --- | --- |
| `cargo test --locked --lib --test audio_history_backup --no-run --message-format=json` | exit0、136.20秒。build成功、test実行なし | `logs/backup-destination-runner-build.jsonl` / `-build.stderr.log` |
| integration EXE `--list`、lib EXE `backup:: --list` | 各exit0、38件/13件を固定 | `logs/backup-destination-runner-integration-list.log` / `-lib-list.log` |
| integration EXE `i2_export_backup::d_b_external_and_similar_name_preserve_wire1_deck_media_and_source --exact --nocapture --test-threads=1`、sandbox内 | PID24656、exit101、timeout=False、wait済み、fixture失敗1（os error5）、行動REDに数えない | `logs/backup-destination-runner-probe.{stdout,stderr}.log` / `-probe-summary.log` |
| integration EXE `--nocapture --test-threads=1`、通常user sandbox外 | PID33304、exit0、timeout=False、wait済み、38 PASS/0 FAIL/0 ignored、4.91秒 | `logs/backup-destination-runner-integration.{stdout,stderr}.log` |
| lib EXE `backup:: --nocapture --test-threads=1`、通常user sandbox外 | PID7092、exit0、timeout=False、wait済み、13 PASS/0 FAIL/0 ignored、311 filtered、4.81秒 | `logs/backup-destination-runner-lib.{stdout,stderr}.log` |

sandbox内の1件は媒体fixtureのAssetStore::putにおけるhard-link公開がアクセス拒否された。fixture回収済みであり、製品の宛先拒否行動の失敗とは区別する。この環境制約を解消するため管理者token拒否guard付きの固定EXE実行をrequire_escalatedで申請し、auto-reviewは許可した。今回の拒否はない。実行時`WindowsIdentity/WindowsPrincipal.IsInRole(Administrator)`は`False`であった。trueまたは検査不能ならテスト起動前に停止する。RunAs/UAC/別資格情報は使っていない。scriptとコマンドは`logs/backup-destination-runner-execute.ps1`、全体記録は`logs/backup-destination-runner-run-summary.log`である。上位各processのdeadlineは90秒である。

最終結果は新17＋既存34＝**51 PASS、0 FAIL、0 SKIP/ignored**である。sandbox probeを加算して52件成功とはしない。実在する各51ケースの名前と個別`ok`は今回生成したintegration/lib stdoutに全件記録した。`--list`と件数は一致する。

| lib隔離子のケース（prefix `backup::i2_export_backup::`） | PID | 結果 |
| --- | --- | --- |
| d_b_guard_observations_cover_snapshot_directory_and_marker_last | 4804 | exit0、timeout=false、wait済み |
| d_b_normal_and_external_guard_reentry_refuse_without_deadlock_or_writes | 24180 | exit0、timeout=false、wait済み |
| d_b_real_media_io_failure_keeps_source_and_leaves_no_completion_marker | 21356 | exit0、timeout=false、wait済み |
| d_b_write_block_allows_eligible_external_backup_without_unblocking | 32700 | exit0、timeout=false、wait済み |
| d_b_write_block_still_checks_pending_history_and_corrupt_content | 11216 | exit0、timeout=false、wait済み |

子は作者実装の20秒deadlineとexact 1件・完了tokenを満たした。各fixture削除ログがあり、probe・integration・lib後の合成TEMP配下残存はいずれも0である。junctionも合成fixtureだけである。実媒体I/O失敗、write-block負例の全副ケース、再入、3点観測とmarker最後のassertは全て実行成功した。mockによるI/O結果差替えはない。

## Handoffと未確認

独立runnerの固定対象51件はPASSであり、親・独立Post reviewerへ渡す。guardの点間連続保持は動的3点観測だけから証明せず、既存の静的reviewとPostで扱う。本runnerは本体reviewを代替しない。

I10の`cargo test --all-targets --locked`と`cargo build --release --locked --bin wordweave5`は今回の親指定範囲外で未実行である。製品delivery完了・I2d-B Closedは主張しない。native microphone/pen/TTS/IME/authentication、実録音・実API・UI受入は未実行であり、合成fixtureの成功をその受入へ読み替えない。今回の出力は本報告・ignored logs/合成fixture・build生成物だけである。誤ってroot logsへ開始した今回のbuildログ2本と合成TEMPは、終了後に自分の明示pathだけを本taskのignored logsへ移し、他者fileは変更しなかった。
