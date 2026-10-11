# I2d-H 独立テスト作者 handoff

現状（2026-10-11追記）：Q-H-CWD公開回帰6件を追加し、4件の行動REDと2件の正常対照PASSを採取した。source所有を親へ返却する。過去の記録は保持し、今回の測定結果は末尾「Q-H-CWD公開回帰6件」を正とする。

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

環境依存の4件（symlink2、short-name、別drive）は `WW5_H_FIXTURE_UNAVAILABLE:<case>:<理由>` を出しreturnする。cargoのPASSでも受入成功から必ず除外し、case名/理由を別集計する。H06実identity拒否は以下の必須fixtureであり、この早期returnの対象ではない。

- h02_leaf_file_symlink / h02_directory_symlink：新しい権限昇格を求めず通常権限で試みる。不可なら親へexact case/EXE/所有fixtureを返す。
- h02_short_name：GetShortPathNameWの実在別名だけを使う。返却が同じ名なら未実証。
- h02_alternate_drive：既存driveをread-only照会し所有fixtureの同identityが見える別driveだけを採用する。drive mapping/OS設定を追加しない。部分rootへの既存subst等を全探索するfixtureではない。

H06実identity拒否は、通常のFILE_READ_ATTRIBUTES＋FileIdInfo成功とprepare後、合成destのDELETE/share7 handleでFileDispositionInfo TRUEを設定する。独立属性openがPermissionDenied/Win32 5となることを必須assertし、製品のIo/IdentityUnavailable拒否・publish失敗を評価する。FALSE解除とhandle close後に同一identity・全bytes/ledger・temp不変を確認し、通常writeの再試行を成功させる。fixture不成立や復元失敗はtest failureとし、UNAVAILABLE早期returnで成功扱いしない。panic/unwind時もRAIIでclose前のFALSE解除を試みる。fake identityやOS設定変更を用いない。

junction作成失敗、通常identity照会失敗、case-sensitive fixture不成立は明示fixture failureであり行動REDではない。初回計画時点では動的fixtureは未実行であり、実施結果は後続の各自己検証記録を参照する。最後のrecheckからOS作成までの敵対的競争、network GUID非対応、実機電源断、UI/実APIは検証したと扱わない。

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

## H-TEMP-1限定追加

2026-10-11。指定test-author `gpt-6-astra/high`、TDD作者工程。入力HEADは `ed2cd75fa90a314e19bbe1f6416d46a4b4021805`、Packetはexport-destination-review.md「H-TEMP-1限定Pre」（SHA-256 `4BF8543BE6DD95A07EA154427F5E5743D3401FE94236CBB805E780F3081306CC`）であり、親から独立Pre PASS受領を確認して着手した。実行model/effort metadataとusage counterは未取得である。

追加はstore専用moduleの `h07_absent_destination_cannot_be_its_own_temp` 1件だけである。安全な不存在destを独立のsymlink_metadata/NotFoundで確認し、prepare成功→temp=destでpublish→Err→dest不存在・全ledger不変・出力directory entry数0を検証する。既存temp sentinelを使わず、同名拒否の分岐へ到達させる。初回からPASSであり、新たな修正前REDの採取や本体修正は行っていない。旧50件に本件を追加したが、旧成功46/79の再実行はしていない。Q-H-CWDは扱っていない。

| 検証 | 証拠 |
| --- | --- |
| libのみbuild | `cargo test --locked --lib --no-run`、exec session83484、cargo PID30204、native/exec exit0。UTC13:06:22.548–13:07:58.826 |
| 新exact | `wordweave5-b3388a7bbef44bb4.exe store::i2_export_destination_tests::h07_absent_destination_cannot_be_its_own_temp --exact --nocapture --test-threads=1`。native PID26124、exit0、1 passed/0 failed/0 ignored/316 filtered |
| 子process | `WW5_H_TEMP1_PREPARED_ABSENT`、`WW5_H_TEMP1_REJECTED_UNCHANGED`、exact CHILD_OKを確認。子exit0、timeout=false。親30秒/子20秒上限、実行timeout0 |
| 限定format/diff | 新関数を抽出・共通indent除去後のrustfmt check exit0、git diff --check exit0。最初のstdin整形確認はPowerShellが付加した余分な終端改行だけを表示したため、入力終端を1改行へ直して無差分を確認した。sourceの再修正はない |
| source境界 | 新test部分を除去したbytesのSHA-256が着手時 `66287990637663A5A6B0119B9D686AB5AEAA34BB84CDD2D045B24DEA05AF4BD0` と一致。本体・旧testは不変 |

終了store SHA-256は `F705AE19635F3C95828B36C60D5CBC8E59D9FB43DE3219A14BC02E73CC28022A`。lib EXE SHA-256は `76EAA8D2CEEDBA99B0894B90611CF7C054D00276104D14E7D3AB6028EDE469F3` である。integration sourceは `9A14DCB790E29AC3407A0D47EB46CACA077BB08DFDCB52D1599C272B7968936E` のまま、public EXEは `CC53B50FC19DAEBE1EAB5219C7D7EF9F3389EBEF040F99D7290E25AD79548EC1` のままであり、public targetをrebuild/実行していない。

TEMP/TMPはprocess限定の `artifacts/i2d-h-temp1-author-temp`。通常sandboxの合成fixtureで実行でき、権限昇格・OS設定変更はない。Drop後の直下残存0、cleanup失敗0を確認した。生ログは `logs/export-destination-author-temp1-build.{stdout,stderr}.log` と `logs/export-destination-author-temp1-exact.{stdout,stderr}.log` に保持した。独立Post判定は親へ渡し、本作者はClosedへ変更しない。

**H-TEMP-1作者検証完了、source所有返却。** 本体失敗・fixture不成立・compile失敗は0であり、追加assertionの緩和や他者差分の変更は行っていない。

## Q-H-CWD公開回帰6件

2026-10-11、test-author指定 `gpt-6-astra/high`、TDD作者工程。親から限定Pre cycle2 PASSを受領したPacket（export-destination-review.md SHA-256 `0E432A55F1DAB3F30B7003AFCE70C8E3CBD82CB112AC4F61040A771E1B603B31`）とHEAD `62f932403a232cfad41ffbb11acf3b3206f8ab49` を入力とした。実行model/effort metadata・usage counterは未取得である。

変更はintegration末尾 `q_h_cwd_tests` の6件と本記録だけである。既存isolated/Fixture/ledger/junction/独立OS identityを再利用し、cwd復元RAII・実prefix観測・case内の区切り表記生成を追加した。旧test部分のbytes SHA-256は `9A14DCB790E29AC3407A0D47EB46CACA077BB08DFDCB52D1599C272B7968936E` と一致した。本体storeは着手/終了とも `F705AE19635F3C95828B36C60D5CBC8E59D9FB43DE3219A14BC02E73CC28022A` で不変である。

buildは `cargo test --locked --test progress_export_destination --no-run` だけを実施した。exec session47565、cargo native PID33096、native/exec exit0、UTC13:19:20.665–13:21:14.974。compile失敗0である。終了を確認後、同一の新public EXE `target/debug/deps/progress_export_destination-b4a366b351b447cc.exe`（SHA-256 `59399F1DB2322F47AF04BE983D062FEB7C8CF7CCF6014E048FECA9993F8023CD`）で各caseを `--exact --nocapture --test-threads=1` により個別実行した。

下表のcase名には `q_h_cwd_tests::` を付ける。生ログは `logs/export-destination-author-cwd-<log名>.{stdout,stderr}.log` に保存した。

| case | log名 / native PID / exit | 結果 |
| --- | --- | --- |
| relative_root_junction_dotdot_refuses_file_with_real_verbatim_cwd | q1-outside / 14668 / 101 | 行動RED |
| relative_root_junction_dotdot_refuses_directory_with_real_verbatim_cwd | q2-outside / 30796 / 101 | 行動RED |
| relative_destination_junction_dotdot_refuses_file_with_real_verbatim_cwd | q3-outside / 27608 / 101 | 行動RED |
| relative_destination_junction_dotdot_refuses_directory_with_real_verbatim_cwd | q4-outside / 23940 / 101 | 行動RED |
| normal_file_path_forms_and_separators_work_with_real_verbatim_cwd | q5-sandbox / 21028 / 0 | 1 PASS、9表記成功 |
| normal_directory_path_forms_and_separators_work_with_real_verbatim_cwd | q6-sandbox / 29896 / 0 | 1 PASS、9表記成功 |

拒否4件は、所有fixture内のjunction作成・独立identity照合を通過後、実 `set_current_dir` → `current_dir` を使った。すべて通常cwdではprefix `Disk(70)`、Err=true、ledger_unchanged=true、outside_entries=0である。同じfixtureをcanonicalizeしたpathへcwdを移すと、実prefixは `VerbatimDisk(70)` となり、4件すべてErr=false、ledger_unchanged=false、outside_entries=1になった。公開APIが元のjunction/..を見落としてfile/directoryを作成した実動作のREDであり、private注入やcompile失敗ではない。

正常対照2件は同じ実VerbatimDisk cwdで、通常相対/rooted-relative/絶対の3形×backslash/forward slash/混在の3表記、各9行を実行した。Fileは期待した外部pathのexact PAYLOAD、Directoryは期待した新規directory・空内容、両方とも保護rootのledger不変と外部9宛先以外のtempなしを確認した。したがって相対入力の一括拒否を修正完了とするoracleではない。

最初のq1通常sandbox試行はnative PID27992/exit101でjunction作成error5となった。これはfixture失敗であり行動REDへ数えていない。生ログ `q1-sandbox` を保持し、既定の許可とツール審査を経て同EXEの拒否4件だけを非管理者sandbox外で確認した。正常対照2件は通常sandboxで実行した。新UAC・管理者実行・OS設定変更はない。

全7試行で `WW5_Q_CWD_RESTORED` を確認した。4行動REDのpanic時もcwd復元がFixture Dropに先行する。TEMP/TMPはprocess限定の `artifacts/i2d-h-cwd-author-temp`、最終残存0、復元/cleanup失敗0、親30秒/子20秒のtimeout0である。各親/子はexact1件を実行し、拒否4件は子exit101、正常2件は子exit0とCHILD_OKを記録した。期待値またはfixtureの想定外結果はなかった。

終了integration SHA-256は `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324`。rustfmt checkとgit diff --checkはexit0である。旧成功suiteは反復していない。build生ログは `logs/export-destination-author-cwd-build.{stdout,stderr}.log` に保持した。

**Q-H-CWD作者RED採取完了、source所有返却。** 本体の修正は実装者へ渡す。新6件の期待値を維持し、修正後のGREEN・既定51件・独立Post/runnerは後続責任とする。本作者は既存の未実証環境条件や全体gateを完了へ変更していない。

## H-IDENTITY-EVIDENCE 必須fixtureへの置換

2026-10-12。test-author指定 `gpt-6-astra/high`、TDD作者工程。入力は承認済みH06/I2-T08・AH-AC04/16と、export-destination-review.md末尾の唯一規範Packetである。親から独立Pre PASS・阻止0（提出時文書SHA `4A516C7F169B8820695CF11B4DF9249E25A5C019354E6D354657F26B8EB55AD8`）を受領して編集した。開始/終了ともbranch `codex/development-agent-team`、HEAD `5d4ab1072158d3c9a3a7875b5f4b4bf1997146ee` である。実行model/effort metadataとusage countersは未取得である。

変更は既存 `src/store.rs::i2_export_destination_tests::h06_real_identity_denial_is_not_absence_when_fixture_is_available` 1関数内のみである。FileDispositionInfoのFFI、1-byte BOOLEAN、DELETE/share7 handle、明示解除とDrop復元のhelperはすべて関数内に置いた。`DELETE_ON_CLOSE`は使用していない。既存prepare後に削除保留を設定し、独立属性openのPermissionDenied/Win32 5を必須assertする。recheck/publish結果を保存し、FALSE解除・handle close完了後に、recheckのIo/IdentityUnavailable・publishのErr、元identity/bytes/全ledger一致、fixed-temp不存在、guard解放後の通常write再試行を確認する。UNAVAILABLE早期returnは除去した。復元失敗はログとtest failureに残し、unwind時の二重panicを避ける。既存Temp/isolated cleanupは変更しない。本記録の現行fixture規範も環境依存4件とH06必須fixtureへ更新し、旧実行記録は歴史的証拠として保持した。

旧share_mode(0) fixtureが属性取得を止められないことは前回rawと独立probeで確認済みである。今回のexactは初回からPASSであり、新しいproduction不具合のRED→GREENではなく、未到達だった既存拒否契約の実証である。期待値緩和、本体修正、他case変更は行っていない。

| 検証 | 今回の結果・証拠 |
| --- | --- |
| lib-only build | `cargo test --locked --lib --no-run`、cargo PID33892、native/exec exit0、UTC21:45:10.362–21:47:08.290。既存unused/incremental hardlink警告、compile error0。`artifacts/export-identity-author-build.json` と同prefixのstdout/stderr log |
| exact1件 | `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe store::i2_export_destination_tests::h06_real_identity_denial_is_not_absence_when_fixture_is_available --exact --nocapture --test-threads=1`。PID10832、exit0、1 passed/0 failed/0 ignored/316 filtered、UTC21:47:36.770–21:47:37.491 |
| 到達oracle | 独立属性open Win32 5/PermissionDenied、`recheck=Err(Io)`、publish Err、FALSE before close、同一identity/bytes/ledger・fixed-temp不在、通常retry exact PAYLOAD/outside entry1。rawは `artifacts/export-identity-author-exact-sandbox.stdout.log` / `.stderr.log`、PID/引数/token/timeoutは `.json` |
| 隔離/cleanup | 通常sandbox・非管理者（WindowsPrincipal Administrator=False）。親30秒/既存子20秒上限、timeout=false、子exit0/CHILD_OK。TEMP/TMPはprocess限定 `artifacts/export-identity-author-temp`、最終残存0、cleanup/復元失敗0。sandbox外・管理者比較は不要であった |
| 名前集合 | 固定公開EXE `--list` の29名＋新libの専用module `--list` 28名、旧runner集合との差0、計57名を維持。実名・差集合・exitは `artifacts/export-identity-author-names.json` |
| source範囲 | `artifacts/export-identity-source-boundary-exact.json`。HEAD原文bytesのhashが着手時hashと一致。変更前/後から該当関数だけを1箇所ずつ除去したhashは両方 `FAAB4CA1A0CDEFE6B41260CC548799A4960DE8AC774AE4A96A2FFE87AD3E7126`。関数外の本体・他testはbytes一致 |
| format/diff | `rustfmt --edition 2021 --check src/store.rs`、`git diff --check` ともexit0。最初のformat指摘は追加eprintln2箇所の改行だけで修正済み。初回logを保持した |

source範囲確認の初回補助スクリプトはHEAD原文を不要にCRLFへ変換して比較し、不一致となった（`export-identity-source-boundary.json`）。これはsource変更ではなく検査入力の誤りである。次のexact記録ではGitのUTF-8 stdoutを改行変換なしで読み、原文hashを開始時hashへ照合したうえで関数外bytes一致を確認した。失敗した補助記録を上書きしていない。

| 対象 | 終了SHA-256 |
| --- | --- |
| src/store.rs | `BB35631D2EF3E42EE8B98B9C6BECC3FEC3310C5CD8D6E96E6F6EF47578745EBC` |
| lib EXE | `68E8DC0504F714EEED8B547228396812089AAE9BFF5310C4BC6BC321454B1314` |
| 公開integration source（不変） | `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324` |
| 公開EXE（不変） | `95E01D39B0ECB7C018C3D78FFE5D3C1B8FF96A395EC3748644F83345954C36AD` |

開始記録は `artifacts/export-identity-source-before.json`、終了branch/HEAD/source/EXE/cleanupは `artifacts/export-identity-source-after.json`。独立fixture probeと通常/unwind復元の先行証拠は `artifacts/export-identity-feasibility.md` / `export-identity-run.log` にある。probe提案の採否は上記唯一規範Packetが優先する。

**作者の自己exact検証完了、source所有返却。** 新しい本体欠陥・fixture不成立・compile失敗は0。既存成功suite・公開symlink2・製品releaseは反復していない。後続の独立runner（指定Sol/medium）と独立Post（指定Astra/high）が未了であり、本作者はH-IDENTITY-EVIDENCEをClosedにしない。symlink2の利用者手動確認・別drive・consumer/I10の残余も維持する。

