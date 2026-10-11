# I2d-H 独立テスト作者 handoff

現状（2026-10-11追記）：fixedErr scaffold後の50件を実行し、31件の行動REDを採取した。本体実装へsource所有を返却する。測定結果は末尾「scaffold後のRED結果」を正とする。

2026-10-11 / ACOUSTIC-HISTORY-001。指定 `gpt-6-astra/high`、実行model/effort metadata・開始/終了usage counterは未取得である。TDD Skillの作者工程を適用した。今回の完了境界は期待値固定とscaffold要求までである。

入力は承認済みAH-AC04/16・I2-T08/T08b、design-i2.md §5「I2d-H限定接合」（SHA-256 `50BE63D85B75AF7E7D1F1EABB306698722CAF89AF6E46821FCB34B04E6A06380`、親から独立Pre PASS/阻止0を受領）、implementation.mdのplanner割当である。開始HEAD `946aa0868051e0a4a091f96bda54104279727c81`、文書commit後HEAD `97469190f315dcbaf4fd5f869e02cb9a35dac0c7`。本体はその間不変である。

## 所有と現状

変更は新 `tests/progress_export_destination.rs` と `src/store.rs::i2_export_destination_tests` の2 source/testfileだけである。既存fixture `i2_gate_tests::{Temp, tree}` をread-only再利用した。store新moduleを除去した全文がHEADのstoreと一致することを改行正規化比較で確認した。本体・既存テスト・oracle・manifest・設定は変更していない。

| ファイル | 固定SHA-256 |
| --- | --- |
| src/store.rs | C6D3B1475C13D7D3F30C107335CC22A2B279484A15FD0A89857858A55E842B73 |
| tests/progress_export_destination.rs | 9A14DCB790E29AC3407A0D47EB46CACA077BB08DFDCB52D1599C272B7968936E |

このhashは作者scaffold前版である。後続本体追加後は再採取する。staticに公開23件＋内部27件＝50件を固定した。`cargo --list`は未実行であり、実在実行件数はscaffold後に照合する。新targetのrustfmt check、新moduleのrustfmt parse/整形、git diff --checkは成功した。新API不在のため、依頼どおりbuild/testは実行していない。compile成功・行動RED・行動PASS・bug再現をまだ主張しない。

## 必要scaffold

新しい未接続の型・口だけを実装役が追加する。既存入口をstubへ接続しない。型のprivate fieldをテストから構築・差替えする要求はない。`ExternalDestinationError`にDebugが必要である。

```rust
pub fn resolve_storage_root() -> Result<PathBuf, String>;
pub fn write_external_file(root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), String>;
pub fn create_external_directory(root: &Path, dest: &Path) -> Result<(), String>;
pub(crate) struct ExternalExportGuard { /* private; clone/変換口なし */ }
pub(crate) fn acquire_external_export() -> Result<ExternalExportGuard, ProgressGateError>;
pub(crate) fn write_external_file_under_guard(
    root: &Path, dest: &Path, bytes: &[u8], guard: &ExternalExportGuard,
) -> Result<(), String>;
pub(crate) fn create_external_directory_under_guard(
    root: &Path, dest: &Path, guard: &ExternalExportGuard,
) -> Result<PathBuf, String>;

enum ExternalDestinationKind { File, Directory }
#[derive(Debug)]
enum ExternalDestinationError {
    UnsafePath, ProtectedRoot, IdentityUnavailable, Io, Changed, AlreadyExists,
}
struct CheckedExternalDestination<'g> { /* private &'g ExternalExportGuard */ }
fn prepare_external_destination<'g>(
    root: &Path, dest: &Path, kind: ExternalDestinationKind,
    guard: &'g ExternalExportGuard,
) -> Result<CheckedExternalDestination<'g>, ExternalDestinationError>;
fn recheck_external_destination(
    checked: &CheckedExternalDestination<'_>,
) -> Result<(), ExternalDestinationError>;
fn publish_external_file(
    checked: &CheckedExternalDestination<'_>, bytes: &[u8], temp: &Path,
) -> Result<(), String>;
fn create_checked_external_directory(
    checked: &CheckedExternalDestination<'_>,
) -> Result<PathBuf, String>;
```

以上の新10関数は固定Errでよい。例としてacquireは既存Reentrant、typed helperはIo、String口は固定scaffoldメッセージとする。新型はその戻り型に必要な最小限であり、FileIdentity/ProtectedRootの実装や既存atomic/gate抽出をRED前へ先行させる必要はない。既存acquire_progress_write/Storage::open/atomic_write・既存入口の接続は維持する。

固定Err scaffoldの予測では公開成功契約4件（H03 sibling/resolver、H04 file/directory）と内部27件がRED候補である。公開拒否18件は固定Errでも通り得るため製品の拒否実装の証拠にはならない。公開legacy atomic 1件は既存動作PASS対照である。これは予測分類であり測定件数ではない。scaffold後は新target/新moduleで型検査→実行して、compile/fixture失敗とbehavior failureを分ける。内部の初期REDがacquire固定Errで止まる場合、H06/H07深部の再現済みとは扱わない。consumer未接続であるため、元のsettings/媒体/backup迂回のend-to-end再現はd-S/B/Mへ残る。

## 受入対応とfixture

| 群 | cases | 固定oracle |
| --- | ---: | --- |
| H01 / T08 | 2 | 手製progress/deck/pending/lock/backup/audio/日本語sentinel。保護file/新規file/root/配下directory拒否、全bytes/dir/temp集合不変 |
| H02 / T08 | 12 | root/dest casing・dot/dotdot・ADS/device/曖昧名・型/親欠損・内外hardlink・junction・symlink・8.3・別drive。aliasは独立OS identity/属性で確認 |
| H03 / T08・08b | 6 | 予定root一段/多段、FileとDirectoryの未作成祖先拒否、ASCII以外の大小文字、祖先junction、兄弟成功、resolver、custom rootでStorage取得失敗 |
| H04 / T08b | 4 | 空/日本語bytes・通常1-link置換・新規/似た名前directory、旧atomic新規/置換/I/O失敗・guard中利用 |
| H05 / T08b | 7 | write-block維持中の外部成功、両方向poison/再入、under-guard複数出力、両方向別thread排他 |
| H06 / T08 | 14 | prepare無副作用/typed分類、File交換9種、Directory交換3種、実identity拒否probe。変化後ledgerを基準にChanged/無変更拒否。新呼出しでの再試行対照 |
| H07 / T08 | 5 | 固定temp=予定root・既存sentinel・通常成功、tempの親/保護領域、mode不一致。実I/O前拒否とexact置換を対照 |

公開fixtureは専用一時directory、内部は既存Tempを使い、すべて合成bytesである。公開清掃はreparseを辿らずentryだけを解除する。内部fixtureは新linkを作らない。hardlink先も所有fixture内である。

内部27件すべてとLOCALAPPDATA試験は20秒上限の子processである。意図的write-block/poisonだけでなく、RED assertionのpanicで保持中mutexがpoisonした場合も親へ伝播させない。親はexit・timeout・stdout/stderrとexact case成功markerを記録し、timeout時はkill/reapする。thread順序はchannel開始確認→取得未完了→guard drop→3秒以内完了で確認する。取消token/非同期cancelは設計で非該当であり、新設していない。

環境依存の5件は `WW5_H_FIXTURE_UNAVAILABLE:<case>:<理由>` を出しreturnする。cargoのPASSでも受入成功から必ず除外し、case名/理由を別集計する。

- h02_leaf_file_symlink / h02_directory_symlink：新しい権限昇格を求めず通常権限で試みる。不可なら親へexact case/EXE/所有fixtureを返す。
- h02_short_name：GetShortPathNameWの実在別名だけを使う。返却が同じ名なら未実証。
- h02_alternate_drive：既存driveをread-only照会し所有fixtureの同identityが見える別driveだけを採用する。drive mapping/OS設定を追加しない。部分rootへの既存subst等を全探索するfixtureではない。
- h06_real_identity_denial：通常のFILE_READ_ATTRIBUTES＋FileIdInfo成功を先に確認し、share_mode(0)保持中の同じ独立OS probeが実際に失敗した場合だけ製品のIo/IdentityUnavailable拒否を評価する。probe成功や独占handle不成立は未実証であり、fake identityや設定変更で補わない。

junction作成失敗、通常identity照会失敗、case-sensitive fixture不成立は明示fixture failureであり行動REDではない。5件を含むすべての動的fixtureは今回未実行である。最後のrecheckからOS作成までの敵対的競争、network GUID非対応、実機電源断、UI/実APIは検証したと扱わない。

## 固定case名

公開target `progress_export_destination`（23）：
```text
h01_every_protected_file_and_new_descendant_refuses_without_mutation
h01_root_and_descendant_directory_refuse_without_mutation
h02_root_and_destination_case_aliases_are_protected
h02_existing_dot_and_dotdot_components_do_not_bypass_protection
h02_ads_device_and_ambiguous_names_refuse_without_writes
h02_wrong_types_and_missing_parent_refuse_without_mkdir
h02_external_hardlink_to_protected_file_is_rejected
h02_external_hardlinks_are_rejected_even_when_both_names_are_outside
h02_junction_ancestors_and_junction_dotdot_are_rejected
h02_junction_to_safe_outside_is_still_rejected
h02_leaf_file_symlink_is_rejected_when_fixture_is_available
h02_directory_symlink_is_rejected_when_fixture_is_available
h02_short_name_alias_is_protected_when_os_provides_one
h02_existing_alternate_drive_alias_is_protected_when_available
h03_planned_root_itself_is_protected_in_both_modes_and_case_spellings
h03_uncreated_ancestor_of_multicomponent_root_is_rejected_in_both_modes
h03_planned_root_with_junction_ancestor_refuses
h03_sibling_export_does_not_create_planned_root_or_parents
h03_resolver_is_side_effect_free_and_tracks_environment_only_in_child
h03_custom_locked_storage_root_remains_protected_without_opening_storage
h04_external_new_replace_empty_and_japanese_bytes_succeed_exactly
h04_new_directory_and_similarly_named_sibling_succeed
h04_legacy_atomic_write_keeps_new_replace_and_io_failure_contract
```

内部prefix `store::i2_export_destination_tests::`（27）：
```text
h05_write_block_allows_external_file_directory_and_keeps_normal_block
h05_poison_refuses_both_gate_entries_and_public_export_without_mutation
h05_external_guard_panic_poison_is_shared_with_normal_writer
h05_reentry_both_directions_is_bounded_and_drop_restores_ownership
h05_one_external_guard_supports_directory_then_repeated_files
h05_external_guard_excludes_other_thread_normal_writer_until_drop
h05_normal_guard_excludes_other_thread_external_writer_until_drop
h04_legacy_atomic_write_remains_gate_free_under_both_guards
h06_prepare_is_read_only_and_typed_initial_refusals_are_distinct
h06_file_parent_replaced_is_changed_and_retry_is_fresh
h06_file_leaf_replaced_is_changed_and_retry_is_fresh
h06_file_leaf_disappeared_is_changed_and_retry_is_fresh
h06_file_absent_leaf_created_is_changed_and_retry_is_fresh
h06_file_hardlink_added_is_changed
h06_file_leaf_became_directory_is_changed
h06_planned_root_appeared_is_changed_and_retry_is_fresh
h06_planned_ancestor_appeared_is_changed_and_retry_is_fresh
h06_existing_root_replaced_is_changed_and_retry_is_fresh
h06_directory_parent_replaced_is_changed
h06_directory_absent_leaf_created_is_changed
h06_directory_planned_root_appeared_is_changed
h06_real_identity_denial_is_not_absence_when_fixture_is_available
h07_fixed_temp_equal_to_planned_root_is_rejected_before_io
h07_existing_temp_sentinel_is_not_deleted_or_replaced
h07_normal_fixed_temp_publishes_exact_bytes_and_disappears
h07_wrong_parent_or_protected_temp_is_rejected_before_io
h07_final_operation_requires_matching_file_or_directory_mode
```

## 次工程のfocused command

scaffold後の作者RED確認：
```powershell
cargo test --locked --test progress_export_destination -- --nocapture
cargo test --locked --lib store::i2_export_destination_tests -- --nocapture
```

旧動作対照と本体変更後に必要な回帰を以下へ限定して選定する。実行者は--list/実行ログから実在件数とEXE/source hashを採取する。本一覧は今回は未実行である。

```powershell
cargo test --locked --test audio_history_gate
cargo test --locked --lib store::i2_gate_tests
cargo test --locked --test audio_history_migration
cargo test --locked --lib store::i2_migration_tests
cargo test --locked --lib store::i2_progress_tests
cargo test --locked --lib store::i2_backup_path_tests
cargo test --locked --lib store::i2_normal_save_tests::n06_
cargo test --locked --lib store::i2_normal_save_tests::n03c_actual_daily_atomic_io_failure_stops_metadata_without_new_block
cargo test --locked --lib store::i2_normal_save_tests::n04_real_atomic_next_is_success_even_when_publisher_returns_err
cargo test --locked --lib store::i2_pointer_commit_tests::p08_
cargo test --locked --lib store::i2_pointer_commit_tests::p04_real_atomic_failure_keeps_exact_old_and_unlocked_control_succeeds
cargo test --locked --lib commit::i2_gate_tests
cargo test --locked --lib commit::i2_material_save_tests::t06_
cargo test --locked --lib commit::i2_material_save_tests::t04_actual_intent_atomic_then_err_recovers_and_returns_actual_progress
```

gate/reader/移行・atomic失敗cleanup/通常保存・pointer・教材saveの再入/競合・backup bridgeを対象とする。d-Hではbackup本体を接続しないため、b-B暫定write-block拒否の既存oracleはそのまま保持する。全suite/releaseは親の既定I10 gateであり、本作者のscaffold前境界では反復しない。

**scaffold必要、source所有返却。** 親から実装役へ新固定Err口だけを渡し、追加中は作者がstoreを編集しない。scaffold完了後に同じ固定期待値でREDを採取する。再委譲・commit/publish・API通信・実データ・OS設定変更は行っていない。

## scaffold後のRED結果

親から新10関数・4型の未接続fixedErr版を受領し、上記50件の期待値を変更せず実行した。型不一致・テスト自身の修正は発生せず、今回のsource変更は0である。store入力/終了SHA-256は `46D03F4479312B978A68D4E8CD80A272950500655167449A60E09CB5452D0C40`、integrationは前記 `9A14…8936E` のままである。実行model/effort metadata・usage counterは引き続き未取得である。

`cargo test --locked --lib --test progress_export_destination --no-run` はexec session `10680`、cargo PID `7704`、native/exec exitともに0で完了した。UTC開始12:29:25.706、終了12:32:33.885。完了前のEXE起動はない。cacheのhardlink→copy警告と未使用コード警告があり、compile errorは0である。build中も実行中もTEMP/TMPはprocess限定で `artifacts/i2d-h-author-temp` とした。

| EXE（target/debug/deps/） | SHA-256 | --list |
| --- | --- | ---: |
| progress_export_destination-b4a366b351b447cc.exe | BE8E23B4CFF5F4C0F27138BFE2C10588DE759EF9262DC6EBAEC84755A4DD8254 | 23 |
| wordweave5-b3388a7bbef44bb4.exe | D5C3124E790C5C4D1484B1A83DFC89645C91A8E4FCD5B2CC30CF9AAD0BA26215 | 27（専用module filter） |

listのPIDは公開32588・内部5904、exitは双方0であり、上記固定50実名と一致した。初回実行は公開PID29784、14 passed/9 failed・exit101、内部PID31056、0 passed/27 failed・exit101である。両方 `--nocapture --test-threads=1`、内部は `store::i2_export_destination_tests` filterである。watchdog timeoutは0であった。

初回公開の9失敗は行動RED4件とfixture error5の5件に分かれる。hardlink2/junction3のfixture失敗だけを、同じhashの公開EXE・exact1件ずつ・通常権限sandbox外で再確認した。親の限定指示と承認によるもので、管理者/UAC要求・OS設定変更はない。

| exact再確認名 | native PID | exit |
| --- | ---: | ---: |
| h02_external_hardlink_to_protected_file_is_rejected | 14688 | 0 |
| h02_external_hardlinks_are_rejected_even_when_both_names_are_outside | 23236 | 0 |
| h02_junction_ancestors_and_junction_dotdot_are_rejected | 33856 | 0 |
| h02_junction_to_safe_outside_is_still_rejected | 3200 | 0 |
| h03_planned_root_with_junction_ancestor_refuses | 31560 | 0 |

再確認5件は独立OS alias照合とledger比較へ到達し、各1 passed/0 failed/0 ignoredである。ただし製品がfixedErrのため、完成した宛先保護の証拠へ読み替えない。全体を再実行した数値ではなく、重複を除いた50件の分類は次である。

| 分類 | 件数 | 根拠/限界 |
| --- | ---: | --- |
| 行動RED | 31 | 公開H03 sibling/resolver・H04 file/directoryの4件と内部27件。固定Errが成功要求または型付きgate要求に反する |
| fixedErr拒否通過 | 15 | 利用可能fixtureでErrと無変更を確認。製品の安全検査が実装済みとは判定しない |
| 旧atomic動作PASS | 1 | h04_legacy_atomic_write_keeps_new_replace_and_io_failure_contract。新規/置換/実I/O失敗時cleanupの対照 |
| fixture未実証 | 3 | leaf/directory symlinkはerror1314、別drive aliasは不在。cargo PASSから除外 |

内部27件はexact子processで実際に1件ずつ走り、公開resolverを合わせた28件の親marker・各child exit101・timeout=false・`running 1 test`を確認した。成功markerは0であり、失敗に整合する。H06/H07の内部検査はscaffoldの外部guard取得Errで止まっている。したがってprepare後交換・recheck・publish/create・固定temp保護そのものをRED再現済みとは扱わず、本体後のGREENで同じ期待値へ到達する必要がある。H05も深い同期/保持状態の完成証拠ではない。

5つの環境依存境界は個別に記録した。symlink2件・別drive1件は上記の未実証、8.3は実別名の独立OS identity照合が成立した。H06実identity拒否は通常FILE_READ_ATTRIBUTES/FileIdInfo成功後にacquireの固定Errで中断し、share_mode(0)による拒否fixture・再検査へ未到達である。この件はgate取得の行動REDであり、I/O拒否の成立/不成立やPASSを主張しない。新symlink権限は本体後に親が必要なら扱う。

全fixtureは自動Dropで清掃され、終了時の専用temp直下残存0、cleanup失敗0を確認した。手動のrecursive削除は行っていない。既存の広い回帰は指示どおり本体後へ集約し、今回は50件とその中の旧atomic対照だけである。UI/実API/実データ/電源断/敵対的最終TOCTOUは未検証のままである。

構造化記録は [export-destination-author-red.json](logs/export-destination-author-red.json) にbuild session/native PID/exit、EXE/source hash、50実名の初回/最終分類、exact再確認5件、28子marker、欠測とcleanupを格納した。生ログは同じlogs内の `export-destination-author-build.{stdout,stderr}.log`、`export-destination-author-{public,private}-{list,red}.{stdout,stderr}.log`、`export-destination-author-exact-<case>.{stdout,stderr}.log` である。元ログを集約値で上書きしていない。

**RED採取完了、source所有返却。** 次は本体実装役である。既定scaffoldの固定Errを解消し、固定50件の到達済み/未到達境界を区別したGREENと上記focused回帰を行う。作者は本体を修正していない。

