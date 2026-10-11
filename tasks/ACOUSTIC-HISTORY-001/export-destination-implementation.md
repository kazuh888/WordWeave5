# I2d-H 実装引継ぎ

2026-10-11 / ACOUSTIC-HISTORY-001。指定 `gpt-6-astra/high`。実行model/effort metadata、開始/終了usage counterは未取得である。`wordweave-change` の実装工程を適用した。UI変更はなくUI Skillは非該当である。最終検証は独立runner、受入は利用者に残る。

入力は `ed2cd75fa90a314e19bbe1f6416d46a4b4021805` の固定Err scaffold、承認済みAH-AC04/16・I2-T08/T08b、design-i2.md §5 H1–H5（SHA-256 `50BE63D85B75AF7E7D1F1EABB306698722CAF89AF6E46821FCB34B04E6A06380`）、独立Pre PASS、独立作者の50件とRED記録である。

## 変更範囲

本体所有は `src/store.rs` のみである。同一Mutexの取得をprivate helperへ抽出し、通常writerだけが従来のwrite-block検査を行う。外部guardは通常guardをprivateに所有し、poison・再入拒否・thread排他・Dropを共有する。解除口や変換口は追加していない。root解決を副作用のない関数へ抽出し、`Storage::open` が使う。公開 `atomic_write` の命名・create_new/write/sync/close/rename・失敗cleanup順序をprivate helperへ移し、既存契約を維持する。

Windows宛先検査は元pathを成分順に観測し、reparseと非通常型を拒否する。既存部分はRAII File handle、GUID解決path、volume serial＋128-bit file IDで固定する。予定rootのみWindows CompareStringOrdinalによる成分比較を使い、予定root自身・配下・未作成祖先を両modeで拒否する。File leafの複数hardlinkも拒否する。既存祖先・leaf・不在成分の観測を保持し、最終操作前にidentity・型・link数・存在状態を再確認する。再観測のI/O/identity不明を不在へ変換しない。

tempも同じ宛先検査を通し、既存temp・別親・保護領域・最終fileと同名のtempをI/O前に拒否する。directoryは解決済み絶対pathへcreate_dirを一度行い、後続callerへそのpathを返す。Windows以外の新exportは未対応エラーを返す。consumer・manifest・依存・UI・既存/新規テストを変更していない。

## 固定対象と検証

- 最終source SHA-256：`66287990637663A5A6B0119B9D686AB5AEAA34BB84CDD2D045B24DEA05AF4BD0`。
- integration SHA-256：`9A14DCB790E29AC3407A0D47EB46CACA077BB08DFDCB52D1599C272B7968936E`（入力と不変）。
- `store.rs` の最初のWindows cfg/test以降を入力版と全文比較し、新旧テストおよび後続既存sourceの不変を確認した。rustfmt check・git diff --check成功。
- buildとfixtureを実行するprocessのTEMP/TMPは `artifacts/i2d-h-impl-temp` のみへ限定した。合成fixtureのみで、実データ・実API・OS設定変更・再委譲・commit/pushは行わない。

初回集約buildは `cargo test --locked --lib --test progress_export_destination --test audio_history_gate --test audio_history_migration --no-run`、exec session `50942`、native PID `11984`、開始UTC `12:43:23.5804444`、終了 `12:47:40.3611621`、native/exec exit 0である。初回新50件は公開18 passed/5 failed（fixture error5）、内部26 passed/1 failed（hardlink追加fixture error5）で行動失敗は0だった。public PID31240/private PID33616、双方exit101である。記録は `logs/export-destination-impl-{build,public,private}.{stdout,stderr}.log`。

その後の自主点検でtempと未作成destが同名の場合の保守的拒否を補強した。独立50件にこのexact専用caseはない。既存oracleを改訂せず、追加試験の要否を独立レビューへ渡す。

最終集約buildは同一コマンド、exec session `69167`、native PID `31756`、開始UTC `12:49:03.3926177`、終了 `12:51:34.3220460`、native/exec exit 0である。`logs/export-destination-impl-build-final.{stdout,stderr}.log` に保存した。既存未使用コード警告とincremental cacheのhardlink→copy警告がある。下記EXEはこのbuild完了後にだけ起動し、以後source/buildを固定した。

| target/debug/deps/EXE | SHA-256 | 実在件数 |
| --- | --- | ---: |
| progress_export_destination-b4a366b351b447cc.exe | CC53B50FC19DAEBE1EAB5219C7D7EF9F3389EBEF040F99D7290E25AD79548EC1 | 新公開23 |
| wordweave5-b3388a7bbef44bb4.exe | B38627DC792671E500F1486ED70D925D380956340AE3A45E4E24C992576F7AA0 | 新内部27、旧focused60 |
| audio_history_gate-0c0e7e180bf0149a.exe | CFB058DD74124D4DADDAE0A7599B0CD96C148952428CFCE4DF194EBB98F9E673 | 16 |
| audio_history_migration-33b6a419850044f4.exe | B11F783816B3E99450C41B2EE8C813E868390DF0B6822037F8E4AAE74B4B6D42 | 3 |

各対象を先に同EXEの `--list` で照合した。新公開は全件、新内部は `store::i2_export_destination_tests` filter、いずれも `--nocapture --test-threads=1` で実行した。公開PID34068・内部PID31988はそれぞれ18 passed/5 failed、26 passed/1 failed・exit101であり、6失敗は全てfixture error5である。生ログは `export-destination-impl-{public,private}-final.{stdout,stderr}.log`。未実証marker4件をPASSへ混ぜていない。

既知alias制約はSHA照合した同一EXE・専用TEMPでexactだけ非管理者sandbox外実行した。全6件は各1 passed/0 failed/exit0である。`export-destination-impl-exact-1`〜`-6` のstdout/stderrへ保存した。

| log番号 | exact case | native PID |
| --- | --- | ---: |
| 1 | h02_external_hardlink_to_protected_file_is_rejected | 12684 |
| 2 | h02_external_hardlinks_are_rejected_even_when_both_names_are_outside | 29984 |
| 3 | h02_junction_ancestors_and_junction_dotdot_are_rejected | 7932 |
| 4 | h02_junction_to_safe_outside_is_still_rejected | 9020 |
| 5 | h03_planned_root_with_junction_ancestor_refuses | 18556 |
| 6 | store::i2_export_destination_tests::h06_file_hardlink_added_is_changed | 4964 |

重複除外した新50件は **46行動成功・4環境未実証・行動失敗0** である。未実証はsymlink2（error1314）、別drive alias不在、H06実identity拒否（独立OS probeがshare_mode(0)下でも成功）である。H06のChanged分類/再試行とH07の固定temp保護/実書込みは到達した。8.3は独立alias確認が成立した。内部hardlink追加もexact再実行でChanged/不変oracleへ到達した。親担当の承認済み管理者symlink exact2は別の検証記録であり、本実装担当の46件へ未確認で加算しない。

## 旧focused回帰

旧gate初回は3件中2成功・1失敗（PID6936、exit101）。`block_applies_to_existing_guard_under_guard_and_all_writer_entries` がgate受入到達前の既存HistoryWriter.publishでHistoryError::Ioとなった。同じ最終lib EXEのexact1件を非管理者sandbox外へ出すと成功（PID30768、exit0）した。原ログ `export-destination-impl-regression-1` と対照 `-1-exact` を保持する。残りは親の明示指示で非管理者tokenを確認し、固定EXEと専用TEMPで必要なfilterだけsandbox外実行した。管理者cargo・広いsuite実行へ拡張していない。

| log suffix / lib filter（省略なしの実名はlist log） | 件数 | PID / exit |
| --- | ---: | --- |
| 1 / store::i2_gate_tests | 3 | 6936→失敗exact30768 / 101→0 |
| 2 / store::i2_migration_tests | 19 | 14320 / 0 |
| 3 / store::i2_progress_tests | 11 | 32188 / 0 |
| 4 / store::i2_backup_path_tests | 9 | 25968 / 0 |
| 5 / store::i2_normal_save_tests::n06_ | 2 | 18012 / 0 |
| 6 / n03c_actual_daily_atomic_io_failure_stops_metadata_without_new_block | 1 | 6224 / 0 |
| 7 / n04_real_atomic_next_is_success_even_when_publisher_returns_err | 1 | 7608 / 0 |
| 8 / store::i2_pointer_commit_tests::p08_ | 4 | 26496 / 0 |
| 9 / p04_real_atomic_failure_keeps_exact_old_and_unlocked_control_succeeds | 1 | 4136 / 0 |
| 10 / commit::i2_gate_tests | 5 | 25624 / 0 |
| 11 / commit::i2_material_save_tests::t06_ | 3 | 30812 / 0 |
| 12 / t04_actual_intent_atomic_then_err_recovers_and_returns_actual_progress | 1 | 7704 / 0 |
| gate / audio_history_gate integration | 16 | 28332 / 0 |
| migration / audio_history_migration integration | 3 | 16908 / 0 |

**旧focused79件成功、重複集計なし。** lib残り11群のexec session35019もexit0、integration2群のexec exit0である。`logs/export-destination-impl-regression-<suffix>.{list,stdout,stderr}.log` に件数と結果を保存した。reader・移行・backup bridge・通常保存・pointer・教材保存のoracleは不変である。全suite/releaseはI10・独立runnerへ残し、代行していない。

終了時のsource/public EXE/lib EXEは上記SHAのまま、専用TEMP直下残存0である。手動のrecursive削除は不要だった。本体source所有を親へ返す。親のsymlink実行完了までsource/build固定を継続する。

## 未実証・残余境界

H06実identity拒否は独立OS probeがshare_mode(0)下でも成功する場合、caseが0終了しても受入成功から除外する。symlink2件・別drive・8.3も環境依存markerで区別する。管理者symlink exact2件は親が調整する。既存のalias権限を新しいOS設定や広い管理者実行へ拡張しない。

最後の再検査からOS操作までの敵対的TOCTOU、電源断、GUIDを提供できないnetwork share、UI/実APIは未実証である。d-S/B/Mのconsumer接続とI2-D2閉鎖、I10全体gateはこの実装単位の完了から推定しない。

## Q-H-CWD 本体修正・focused結果

2026-10-11、指定implementer `gpt-6-astra/high`。実行model/effort metadata・開始/終了usage counterは未取得である。入力HEAD `6416f32`、store SHA `F705AE19635F3C95828B36C60D5CBC8E59D9FB43DE3219A14BC02E73CC28022A`、integration SHA `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324`。export-destination-review.md「Q-H-CWD限定Packet」の規範全文、独立Pre cycle2 PASS、作者の公開4行動RED＋正常2PASSを確認して着手した。`wordweave-change` をprivate絶対化修正の範囲で適用した。

変更はstore Windows module内 `absolute_unobserved` の追加と、prepareの `cwd.join` 2箇所の置換のみである。絶対入力は元表現を保持し、rooted-relativeはcwd prefix＋入力、通常相対はcwd＋区切り＋入力をOsStringで構成する。verbatim基点へ通常入力を連結するときだけ `/` を `\` へ置換し、`.`/`..`と成分順序を保持する。既存verbatim絶対入力は変換せず、従来のvalidate→walkへ渡す。Packetの方式変更はない。

新helper範囲を除去し2呼出しを元へ戻した全文SHAが入力store SHAと一致した。FFI・gate・wire・既存/新規testを含む他範囲は不変である。integrationも入力SHAと一致する。rustfmt checkとgit diff --checkはexit0である。他担当のmeasurements変更には触れていない。

| 固定対象 | 最終SHA-256 |
| --- | --- |
| src/store.rs | 53843CC8A8DBDB63BB58C82DB3BB42C7E943F1AE00E8EF613DE61BD9E45F42FC |
| tests/progress_export_destination.rs（不変） | 176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324 |
| target/debug/deps/progress_export_destination-b4a366b351b447cc.exe | 2924D49F774E270CB4C919BCA48710DD3A5DF5C8D96A7E6AAA4802204A9CDE6C |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | FE33CB97384A989E749250D6FB4AC2AA7061E632C151FBB6B85097DEEB2971AB |

buildは `cargo test --locked --lib --test progress_export_destination --no-run` に集約した。exec session77291、cargo PID9332、開始UTC13:26:25.0845349、終了13:28:42.2524173、native/exec exit0。既存unused/incremental hardlink→copy警告があり、compile errorは0である。生ログは `logs/export-destination-impl-cwd-build.{stdout,stderr}.log`。build完了後の固定EXEだけを実行し、以後sourceを変更していない。

`--list` はpublic29・private28（store::i2_export_destination_tests filter）、合計57件と一致した。list logは `export-destination-impl-cwd-{public,private}-list.log`。通常sandbox実行は同EXEの `--nocapture --test-threads=1`、privateのみ同module filterを付けた。public PID24408は20 passed/9 failed/exit101、private PID2364は27 passed/1 failed/exit101。10失敗はいずれも既知junction/hardlink fixture作成error5である。最初のログを `export-destination-impl-cwd-{public,private}.{stdout,stderr}.log` に保持した。

SHA一致・非管理者tokenを確認し、同EXE・合成専用TEMPで失敗exact10件だけをsandbox外実行した。各1 passed/0 failed/exit0。新UAC・管理者実行・OS設定変更はない。`export-destination-impl-cwd-exact-<番号>.{stdout,stderr}.log` に記録した。

| 番号 | exact case（prefixは明記） | PID |
| --- | --- | ---: |
| 1 | h02_external_hardlink_to_protected_file_is_rejected | 20856 |
| 2 | h02_external_hardlinks_are_rejected_even_when_both_names_are_outside | 34692 |
| 3 | h02_junction_ancestors_and_junction_dotdot_are_rejected | 4936 |
| 4 | h02_junction_to_safe_outside_is_still_rejected | 28692 |
| 5 | h03_planned_root_with_junction_ancestor_refuses | 31020 |
| 6 | q_h_cwd_tests::relative_root_junction_dotdot_refuses_file_with_real_verbatim_cwd | 10820 |
| 7 | q_h_cwd_tests::relative_root_junction_dotdot_refuses_directory_with_real_verbatim_cwd | 15256 |
| 8 | q_h_cwd_tests::relative_destination_junction_dotdot_refuses_file_with_real_verbatim_cwd | 7872 |
| 9 | q_h_cwd_tests::relative_destination_junction_dotdot_refuses_directory_with_real_verbatim_cwd | 31148 |
| 10 | store::i2_export_destination_tests::h06_file_hardlink_added_is_changed | 31588 |

Q-H-CWD新6件は **6成功**。拒否4件は実 `Disk(70)` / `VerbatimDisk(70)` の両cwdで `is_err=true / ledger_unchanged=true / outside_entries=0` を記録した。正常File/Directory対照2件は各9表記すべて `is_ok=true`、保護領域不変である。相対入力を一括拒否する修正ではない。成功6件の子exit0・CHILD_OKとcwd復元markerを確認した。

旧境界51件は **47成功・4環境未実証**。合計は **57件中53成功・4未実証・行動失敗0**、exact再試行の重複集計なし。未実証はsymlink2のerror1314、別drive alias不在、H06実identity拒否probeがshare_mode(0)下でも成功の4件である。caseの0終了と受入成功を区別した。H-TEMP-1の独立追加1件も成功した。保存79件はprivate絶対化以外の本体が不変なので、指示どおり反復していない。

build/fixture processのTEMP/TMPは `artifacts/i2d-h-cwd-impl-temp` のみに限定した。終了時残存0、timeout/復元失敗marker0、source/integration/EXE SHA不変を確認した。実データ・外部通信・再委譲・commit/pushなし。Q限定Postと独立runnerは親へ返す。既存4環境未実証、consumer接続・I10・実機受入の残余境界は維持し、本体source所有を返却する。
