# I2d-H 独立runner結果

2026-10-12 JST / ACOUSTIC-HISTORY-001。計画者指定は `gpt-6.1-sol/medium`（implementation.md「全単位の7役・起動値」）、起動指示も同値である。実行model/effort metadata、開始/終了のusage countersは未取得であり推定しない。AGENTS.md、development-team.md、KPI、および wordweave-change 全文を読了し、後者は検証・結果報告部分だけを適用した。

完了境界は、承認済み AH-AC04/16・I2-T08/T08b、H-TEMP-1/Q-H-CWD限定Post PASSを入力とする固定sourceの独立build、新57件と旧focused79件の実行・証拠採取である。製品配布・consumer接続・I10・実機受入の完了ではない。入力handoffは export-destination-{implementation,tests,review}.md である。管理者symlink exact2件は親所有であり、本報告へ未確認で加算していない。

## 固定対象

branch `codex/development-agent-team`、開始HEAD `334fc46f8665514f7328ae6d328f3e46a1c377ea`、開始tree cleanである。試験対象の未コミットsource差分は0である。終了時の他者変更は measurements.md/current.md のみであり保持した。本runnerの書込みは本報告、専用logs、合成TEMP、build生成物だけである。

| 対象 | SHA-256（前後一致） |
| --- | --- |
| src/store.rs | `53843CC8A8DBDB63BB58C82DB3BB42C7E943F1AE00E8EF613DE61BD9E45F42FC` |
| tests/progress_export_destination.rs | `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324` |
| target/debug/deps/progress_export_destination-b4a366b351b447cc.exe | `95E01D39B0ECB7C018C3D78FFE5D3C1B8FF96A395EC3748644F83345954C36AD` |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | `681ABCFAE7CE0AAE679799A1D962DB1E31822040E0A2FC8C03AB31286D918799` |
| target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe | `7A11B4BBA64565EEF1E5B60C141F80852FF4D2A5D537A80595F1DD07B58743CB` |
| target/debug/deps/audio_history_migration-33b6a419850044f4.exe | `99A308EA631D5446BEE54C992A9898111D0B4357E509D0485568C6B40B804FDE` |

Cargo.toml/Cargo.lock/src/commit.rs/旧integration2fileも前後hash差0である。詳細は [source-before](logs/export-destination-runner-source-before.json) / [source-after](logs/export-destination-runner-source-after.json)。EXEはbuild完了後にだけ固定し、以後source/buildを停止した。

## コマンドと結果

独立集約buildは `cargo test --locked --lib --test progress_export_destination --test audio_history_gate --test audio_history_migration --no-run` を一回だけ実施した。exec session34894、cargo native PID9244、UTC `2026-10-11T18:47:57.7237861Z`–`18:51:51.4828488Z`、native/exec exit0である。既存unused警告とincremental hardlink→copy警告があり、compile error0。生ログは [build stdout](logs/export-destination-runner-build.stdout.log) / [build stderr](logs/export-destination-runner-build.stderr.log)。

4EXEに各必要filter＋`--list` を先行した。全16照合はexit0、実名136・unique136・重複0、新57件は作者handoff実名集合との差0である。実名を省略しない集合・filter・期待数・PID・exit・EXE hashは [lists.json](logs/export-destination-runner-lists.json) と各 `export-destination-runner-<群>.list.{stdout,stderr}.log` に保存した。

実行は同EXEにfilter（空欄ならtarget全件）＋`--nocapture --test-threads=1` を付けた。旧filterの完全名はlists.jsonに保持した。初回exec session61639は全processをWaitForExitし、最後のwrapper exit0は各試験の成功を意味しない。各native exitを次表へ別記する。

| 群 / filter | 件数 | 初回pass/fail | native PID / exit | 重複除外した最終成功/未実証 |
| --- | ---: | --- | --- | --- |
| public / 全件 | 29 | 20/9 | 7868 / 101 | 26/3 |
| private / store::i2_export_destination_tests | 28 | 27/1 | 28844 / 101 | 27/1 |
| r01 / store::i2_gate_tests | 3 | 2/1 | 9564 / 101 | 3/0 |
| r02 / store::i2_migration_tests | 19 | 18/1 | 7564 / 101 | 19/0 |
| r03 / store::i2_progress_tests | 11 | 10/1 | 31372 / 101 | 11/0 |
| r04 / store::i2_backup_path_tests | 9 | 8/1 | 24660 / 101 | 9/0 |
| r05 / store::i2_normal_save_tests::n06_ | 2 | 2/0 | 29444 / 0 | 2/0 |
| r06 / store::i2_normal_save_tests::n03c_actual_daily_atomic_io_failure_stops_metadata_without_new_block | 1 | 1/0 | 32760 / 0 | 1/0 |
| r07 / store::i2_normal_save_tests::n04_real_atomic_next_is_success_even_when_publisher_returns_err | 1 | 1/0 | 8704 / 0 | 1/0 |
| r08 / store::i2_pointer_commit_tests::p08_ | 4 | 4/0 | 8848 / 0 | 4/0 |
| r09 / store::i2_pointer_commit_tests::p04_real_atomic_failure_keeps_exact_old_and_unlocked_control_succeeds | 1 | 1/0 | 33960 / 0 | 1/0 |
| r10 / commit::i2_gate_tests | 5 | 4/1 | 3400 / 101 | 5/0 |
| r11 / commit::i2_material_save_tests::t06_ | 3 | 3/0 | 32008 / 0 | 3/0 |
| r12 / commit::i2_material_save_tests::t04_actual_intent_atomic_then_err_recovers_and_returns_actual_progress | 1 | 1/0 | 34692 / 0 | 1/0 |
| gate / audio_history_gate全件 | 16 | 5/11 | 24084 / 101 | 16/0 |
| migration / audio_history_migration全件 | 3 | 3/0 | 17996 / 0 | 3/0 |

初回は110 libtest通過・26失敗である。新境界10失敗はhardlink/junction fixture error5、旧libのjunction4と既知HistoryWriter.publish Io1、旧gate11もHistoryWriter.publish Ioで受入oracle到達前だった。原ログを保持した。既定のツール審査を経て、固定同EXEの失敗exact26件だけを非管理者sandbox外で比較した。WindowsPrincipalのAdministrator判定 `False`、process限定TEMP/TMP、source/build固定である。成功済みcaseは反復していない。

exactコマンドは `<固定EXE> <完全case名> --exact --nocapture --test-threads=1`、exec session80280/exit0、native26件はすべて各1 passed/0 failed/0 ignored/exit0である。完全case名・PID・開始終了UTC・引数・hash・token条件は [exacts.json](logs/export-destination-runner-exacts.json)、生ログは `export-destination-runner-exact-1`～`-26.{stdout,stderr}.log`。新10件はindex1–10、旧16件はindex11–26である。この対照結果から今回の初回失敗を環境制約に分類する。期待値・本体を修正して緑へ変えてはいない。

最終集計は **136 unique中132行動成功・4環境未実証・行動失敗0・ignored0**。新57は53成功/4未実証、旧79は79成功である。[cases.json](logs/export-destination-runner-cases.json) は136完全実名の初回状態・exact接合・最終状態を一件ずつ保持し、[summary.json](logs/export-destination-runner-summary.json) と [runs.json](logs/export-destination-runner-runs.json) へ集計とnative実行情報を保存した。古い担当ログを今回の実行結果へ代用していない。

## 到達証拠と残余

- private28 unique/public7 uniqueの子成功marker、exact親子の `running 1 test`・exit0・timeout=falseを確認した。意図的poison panicは成功oracle内であり試験失敗へ読み替えない。timeout=true0、cleanup失敗0、cwd復元失敗0。
- H-TEMP-1のprepare成功/拒否後不変markerを確認した。Q-H-CWD拒否4件は実Disk/VerbatimDisk両cwd、Err/ledger不変/outside_entries0、正常2件は各9表記すべてis_ok=trueを確認した。cwd復元marker計10は初回失敗4も含み、成功case数ではない。
- short-nameは独立alias確認markerへ到達した。未実証はsymlink2（error1314）、既存別drive alias不在1、share_mode(0)下でも独立FILE_READ_ATTRIBUTES/FileIdInfo probe成功1である。libtestの0終了でもPASSに数えない。親の管理者exact2件は別証拠として接合する。
- build/fixtureのTEMP/TMPはprocess限定 `artifacts/i2d-h-runner-temp`。終了直下残存0、手動recursive削除なし。実データ・外部API・OS設定変更・再委譲・commit/push/publish・ユーザーアプリ停止は行っていない。
- 最後の再検査後の敵対的TOCTOU、電源断、GUID非対応network share、native IME/マイク/ペン/TTS/実認証/UIは未検証である。consumer接続、I2-D2閉鎖、I10全体gate、`cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` は本限定runnerでは実施しておらず、Windows製品配布を合格としない。

補助静的確認の `git diff --check` はexit0。本体・テスト・期待値・設定の変更0で所有を親へ返す。親のsymlink実行完了までsource/build固定を維持する。KPI開始/終了usage欠測と対象範囲を親へ引き継ぎ、measurements/currentの更新は親担当とする。

## 親による承認済み管理者2件の起動確認

同じ公開EXE/hashを指定し、既存 `artifacts/Invoke-ExportSymlinkTests.ps1` のRunAsをUTC `2026-10-11T18:52:25.7266576Z` に開始した（session36012、待機wrapper PID3172）。10分経過後も起動結果・`export-destination-elevated-*` artifactがなく、試験実行を確認できなかった。Windows確認画面が利用者に届いたか、起動しなかった原因は未確認である。

利用者の10分待機方針に従い、CIMで名前・作成時刻・script名・指定hash・Start-Processを照合した自分のwrapper3172だけをUTC `19:02:50.0100360Z` に終了した。session exit-1、artifact0。製品・他のprocess・OS設定には変更していない。管理者2件は未実行であり、上記132成功/4未実証の集計を変更しない。起動待ちは残していない。次工程の独立最終Postへ、この未実証を含む証拠を引き渡す。
