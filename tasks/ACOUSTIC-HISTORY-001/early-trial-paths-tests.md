# ET1 独立テスト作者・段階1引継ぎ

2026-10-13。役 `ww-test-author`、planner指定 `gpt-6-astra/high`、fresh context。AGENTS、development-team、KPI、`agent-skills:test-driven-development` 全文を読了した。担当は独立oracle/fixtureとREDまでであり、本体・既存期待値・仕様・gateを変更しない。

入力は `early-trial-plan.md` ET1、契約SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`（実測一致）、`early-trial-review.md` cycle 2仕様/設計PASS・ET-D01設計Closed、親受領のd-S/B/M Closed・依存集約PASSである。開始HEADは `186b46433801b0bc953eeaebf15b297bf07cd577`。途中の親docs-only checkpoint `9fd2dc6`は本体変更でないという親報告を受領した。通常版の10/03 EXE・実データ・鍵・マイクには触れていない。

## 完了境界と現状

新 `tests/early_trial_paths.rs` と `src/store.rs`・`src/commit.rs`・`src/ai.rs` の新 `early_trial_paths_tests`、およびstoreの `early_trial_probe_tests` moduleだけを編集した。本体の追加stubも作者は書かない。既存テストは変更していない。公開Storage/open/load/resolver/external file/directoryを中心とし、private APIは設計にある型/関数と既存prepare/recheck/publish・recover faultを使用する。

最終段階1sourceには34個の `#[test]` 宣言が実在する。構成による有効数は通常4（integration 3、bin ai 1）、試用33（integration 14、lib store 5、lib commit 4、lib probe 9、bin ai 1）である。これはsource上の集合であり、取得済みlibtest listは通常integrationの3件だけである。宣言/予定有効数/list/runtimeを合算しない。probe追加前の中間報告25宣言・通常4/試用24は、この最終集合とは分ける。

`cargo test --locked --features early-trial --test early_trial_paths --no-run` はexit 101、`package ... does not contain this feature: early-trial`。ログ `target/early-trial-build/et1-author/feature-absence.log`。**compile blockerであり行動REDではない**。同じfeature欠落確認は繰り返さない。通常構成compile/runtimeの結果は下記に固定する。現時点の試用行動REDと受入PASSは未取得である。

通常integration no-runは成功した。初回sandbox runtimeは2pass/1fail、正常external publishのOS5でありruntime制約として分類した。固定EXEをAdministrator=Falseでsandbox外実行したところ、同external publish成功後にoracleが生存中app.lockを読んでOS33となった。これはfixture欠陥でありREDではない。lockだけmetadata長を観測するよう修正し、再compile exit0後、固定SHA `384CF397447A4638F302C4658D80C856FF6C601BCD9579AE9C7A8D8991F26ED3` の `target/debug/deps/early_trial_paths-2a28ddad8cdc1bbd.exe` を通常権限で再実行し、**3実在/3pass/0fail/0ignored**を確認した。Administrator=False、Hidden、45秒上限、専用合成TEMPである。ログは同author directoryの `default-compile-fixture-repair.log`、`default-runtime-fixed.stdout.log`、`default-runtime-fixed.stderr.log`。元の失敗ログも保持した。

## 実在集合と独立期待値

integration test名は `tests/early_trial_paths.rs` 内の次である。`et01_default...`だけ通常構成専用、resolver/genericは両構成、他は試用専用である。

| test名 | AC | 独立oracle |
| --- | --- | --- |
| et01_resolver_is_pure_and_selects_build_namespace | ET-01 | 固定namespace、異cwd、全ledger不変 |
| et01_default_open_still_uses_normal_namespace | ET-01 | 通常openは従来root、試用root不作成、既存sentinel不変 |
| et05_unregistered_generic_and_storage_at_keep_caller_contract | ET-05 | LOCALAPPDATAなしでも未登録Storage::atのcaller契約を保持 |
| et01_trial_open_save_reopen_and_environment_change_keep_registered_root | ET-01/04/06 | 新規保存・再open・環境変更後も登録root固定、通常sentinelとCODEX_HOME不変 |
| et01_missing_or_relative_localappdata_refuses_without_fallback | ET-01 | root欠落/相対rootはBeforeWritesの安全文、全bytes/dir不変 |
| et01_normal_content_is_not_read_and_normal_absence_is_not_created | ET-01 | malformed通常progressのOS読取共有拒否でも試用open/load成功。通常root不在を補充しない |
| et04_current_history_unknown_and_corrupt_progress_refuse_unchanged | ET-04 | history/未知version/破損/duplicateはpreflight拒否、修正後retry成功 |
| et04_pending_old_and_new_history_or_invalid_documents_refuse_before_recovery | ET-04 | pending旧/新のhistory又は不正を回復前拒否、ledger不変 |
| et04_nonhistory_pending_recovers_exact_bytes_and_keeps_orphan_history | ET-04 | 非履歴pendingは既存回復、旧pending全bytesをarchive。未参照objectだけでは拒否/削除しない |
| et04_load_rechecks_current_and_pending_after_open_without_recovery_writes | ET-04 | open後にcurrent/pendingを変えた合成入力でもloadは回復前に拒否 |
| et01_trial_hardlinked_file_refuses_and_preserves_both_names | ET-01 | 試用内複数linkを拒否、link解除後retry成功 |
| et01_trial_identity_unavailable_is_not_treated_as_absent | ET-01 | 実Windows共有拒否32を独立確認、identity不能でrootを不在扱いしない |
| et05_registered_caller_normal_and_trial_roots_reject_files_directories_and_retry_outside | ET-05 | caller+固定2rootについてfile/dir拒否、不変、外部宛先選び直し成功 |
| et01_trial_root_and_nested_junction_refuse_without_following_normal_tree | ET-01 | 通常rootへのjunctionと試用内assets junctionを追わず拒否 |
| et05_registered_root_aliases_and_hardlinks_reject_unchanged | ET-05 | 通常/試用へのjunction・hardlink出力を拒否、全bytes/dir/payload不変 |

libの各名には `store::early_trial_paths_tests::` を付ける。

| test名 | AC | 独立oracle |
| --- | --- | --- |
| et01_typed_preflight_rejects_equal_contained_and_relative_roots_before_writes | ET-01 | 同一/双方の包含/相対pathはtyped BeforeWrites/UnsafeRoot、mkdir前に拒否 |
| et01_lock_failure_is_initialization_started_and_retry_preserves_progress | ET-01/ET-D01 | 実app.lock競合はStarted/StorageIo、解放後retry、通常data不変 |
| et01_registration_rejects_a_different_root_pair_without_creating_it | ET-01 | 別pairへの再登録は拒否、別rootを作らず同pair再open可能 |
| et05_fixed_temp_cannot_create_the_registered_absent_normal_root | ET-05 | destの同parentにある不在normal rootをtempとしても作らない。通常tempは成功・消去 |
| et05_final_recheck_observes_caller_and_both_registered_root_identities | ET-05 | caller/trial/normalの各identityをprepare後に通常renameで変更し、最終recheck/publish拒否 |

libの次には `commit::early_trial_paths_tests::` を付ける。

| test名 | AC | 独立oracle |
| --- | --- | --- |
| et04_pending_preflight_is_read_only_for_absent_and_nonhistory_intents | ET-04 | pending不在/非履歴旧null/非履歴旧有は読取検査成功、回復/退避書込なし |
| et04_pending_preflight_rejects_history_on_either_side_and_strict_decode_errors | ET-04 | 旧新それぞれhistory/未知版/duplicate/破損、intent破損を副作用なし拒否 |
| et04_pending_preflight_enforces_existing_intent_byte_limit_before_read | ET-04 | 既存上限350,000,000 bytesの+1をファイル長で拒否、既存file集合/長さ保持 |
| et_d01_nonhistory_recovery_fault_after_actual_write_retains_partial_state_then_retries | ET-D01 | 既存AfterDeck fault時に実deck書込だけが適用されpending保持。retryで回復完了。不変と誤分類しない |

binの `ai::early_trial_paths_tests::et06_config_uses_storage_namespace_without_launch_or_environment_rewrite` はET-01/06である。合成の非実行EXEをresolveするだけで、Config.cwdを保存先namespaceと照合する。試用登録後はLOCALAPPDATA/cwdを変えても同じcodex-workを返す。Config作成からdata/codex-workを作らず、model/effort設定・CODEX_HOMEを変更しない。app-server起動、実API、認証読取は呼ばない。

追加入力は `early-trial-probe-design.md` SHA256 `40A12BE64F07136DEBE410FB52C22A124D5AC89CDB8D1666E5E19FCF52C05D81`（実測一致）、親受領の再Pre PASS・ET-P01設計Closedである。次の実在9名は `store::early_trial_probe_tests::` を付ける。いずれも未compile/未実行である。

| test名 | AC | 独立oracle |
| --- | --- | --- |
| et01_probe_new_root_completes_real_io_and_removes_file | ET-01 | 不在trialの9点・実create/write/sync/close/remove、probe不在 |
| et01_probe_existing_root_preserves_data_and_removes_file | ET-01 | 既存trialの9点、通常/既存data sentinel保持 |
| et01_probe_create_collision_preserves_unowned_sentinel | ET-01 | 実create_new衝突、BeforeCreateだけ、非所有sentinel保持 |
| et_d01_probe_write_failure_closes_and_cleans_once | ET-D01 | Write注入、close/cleanup各1、残存0 |
| et_d01_probe_sync_failure_closes_and_cleans_once | ET-D01 | Sync注入、実write後close/cleanup各1、残存0 |
| et_d01_probe_cleanup_failure_retains_exact_synced_bytes | ET-D01 | cleanup注入、実write/sync済み固定bytes1file残存 |
| et_d01_probe_write_and_cleanup_failure_retains_empty_owned_file | ET-D01 | 主/cleanup両注入を別event、空probe1file残存 |
| et_d01_probe_sync_and_cleanup_failure_retains_exact_written_bytes | ET-D01 | 主/cleanup両注入、write済み固定bytes1file残存、sync成功としない |
| et04_history_current_and_pending_refuse_before_any_probe_event | ET-04/ET-D01 | current/pending拒否はBeforeWrites、event0、lock未作成 |

専用observerは設計指定のProbePoint/Fault/Event/Registrationとobserve_probeを実装し、callerから受け取ったpathだけを観測する。collision sentinelの作成先は子の合成root配下とassertする。ObserverだけをDrop解除し、TrialRootsにreset口を追加しない。各故障後、別の安全な合成pairで再openする対照を置き、失敗時に初回登録を確定しないこととlock解除を検査する。

## fixtureと安全境界

各環境変更/登録を30秒上限・終了時kill/reap付きexact childへ隔離する。childだけに合成LOCALAPPDATA/TEMP/TMPを渡す。Windows `CREATE_NO_WINDOW`、固定test名、各rootは一意の合成tempである。通常treeはprogress/deck/audio/ink/journal/diagnostics/backup/pendingに独立sentinelを作る。ledgerはreparseを追わずdata file bytesとdir集合を比較する。**生存中app.lockだけはOS byte-range lockのためmetadata長の観測**であり、未読を全bytes不変に一般化しない。通常open controlではdrop後の空lock bytesも確認する。junctionはWin32の通常権限fixture、hardlinkは通常 `fs::hard_link`、symlink/RunAs/UAC/OS設定変更はない。

Progress/Intent fixtureは仕様形式のliteralと文字列escapeだけを使う。productのProgress serializer/Intent writerから期待値を作らない。history wireは拒否試験より先に構造decodeでSomeを確認し、fixture不正をhistory拒否成功としない。非履歴のarchive内容・deck・progressは独立literalと厳密比較する。app.lock競合、identity拒否、history入力は実local componentであり、fake identity/validation success flagはない。

正常ledger内の通常sentinelの読取はテストoracleによる読取である。product側の通常tree非読取については、通常progressの実共有拒否を置く対照と後続source reviewを併用する。このケースだけで全treeへのすべての読取不存在を動的証明したとは扱わない。

known sandbox hardlink制約は再確認せず、compileとruntimeを分離する。sandbox外runtimeが必要な場合は親/runnerが固定SHA EXEを対象にAdministrator=falseを実測し、true/取得不能なら起動前拒否、通常権限・hidden・有限timeout・合成TEMPを維持する。新管理者symlink試験の許可を既存証拠から流用しない。

## 最小scaffoldの返却

feature/API不在のため、親から実装者に次だけを依頼する。副作用なし固定Errと型の存在が目的でありGREEN実装ではない。

1. `Cargo.toml`: 既定off `early-trial = []`（依存/Cargo.lock変更なし）。
2. `src/store.rs`: feature内の設計明記private `TrialOpenStage::{BeforeWrites, StorageInitializationStarted}`、`TrialOpenReason::{RootUnavailable, UnsafeRoot, HistoryPresent, InvalidProgressOrPending, StorageIo}`、`TrialOpenFailure { stage, reason }`、free private `open_early_trial(root: PathBuf, normal: PathBuf) -> Result<Storage, TrialOpenFailure>`。関数は副作用なし固定Err。既存 `Storage::open` をstubへ接続して振る舞いを書き換えることはこのscaffoldでは要求しない。
3. `src/commit.rs`: feature内crate-private `check_early_trial_recovery(root: &Path) -> Result<(), String>` の副作用なし固定Err。

aiには新署名が要らない。登録型/登録関数の私的命名を作者から追加要求しない。実装者のstub後、作者へ所有を戻してcompile修正と公開経路の行動REDを取得する。固定Err stubの成功ケース失敗はscaffold由来であり、現在の公開経路の通常root選択を再現したREDと区別する。

probeは独立Pre済み限定設計に基づく専用observerを追加済みである。本体hook呼出の接合はGREEN工程の実装者が担当する。scaffold工程には追加接合を要求しない。write/sync/removeのfailureは決定的注入、create collisionは実OS拒否、無故障は実I/Oと区別する。安全検査を飛ばす注入、検査bool、偽identity、新公開APIはない。

## 検証コマンドと残る受入

stub後のfocused compileは次である。

```text
cargo test --locked --features early-trial --test early_trial_paths --no-run
cargo test --locked --features early-trial --lib early_trial_paths_tests --no-run
cargo test --locked --features early-trial --lib early_trial_probe_tests --no-run
cargo test --locked --features early-trial --bin wordweave5 ai::early_trial_paths_tests --no-run
cargo test --locked --test early_trial_paths --no-run
cargo test --locked --bin wordweave5 ai::early_trial_paths_tests --no-run
```

各実行EXEのSHA/listを固定した後、同じfilterの有限runtimeを実施する。既存store/export/backup/media regressionと全体gateは親→実装者→独立runnerが既定計画どおり担当し、この作者のcompile/REDは全体PASSの代用でない。

ET2の資格情報/JSON適用/起動factory・表示、回復成功後app初期化失敗、ET3 native直接起動/改名/再起動/拒否dialog/DPI/実機受入は対象外である。通常root非列挙、bounded read/strict decodeの接合、publisher/fault順不変は独立source reviewでも確認する。取消しボタン/自動retryは本契約に新設されず、ET1は拒否後に原因・宛先を変えた明示retryを検査する。

## 記録・所有の追記欄

開始/終了token counter・runtime model/effort metadataは未取得。指定値から実行値を推定しない。環境が返したthread IDは `01a13517-5497-7ca0-80c6-7b1b3997bde4`、collaboration taskは `/root/trial_paths_tests` である。親のusage記録を作者は編集しない。

段階1最終source hash（probe追加・lock fixture修正後、未compile試用）:

- `tests/early_trial_paths.rs`: `25C16F6BFE54C7293FF22433F5F11620189D8497725EAB7E2FEF13E3BCB3B9DB`
- `src/store.rs`: `300F368F9B05D69FDEAFD1CBAAAD4FF6C3C490A6950021B38197B3C58435B4BD`
- `src/commit.rs`: `F1D7815CB3A866456833260B04119DA9F09E193CAD5A6AB82EAF369713516307`
- `src/ai.rs`: `D53FE44EA6391412E26EEE44B84AF05475E2C9674626D6DD2F57EB92E638B3E8`

所有返却は親への明示message時点で行う。それまでは新test/専用cfg(test)を作者が保持する。

## 段階2：最小stub後のcompile/RED（2026-10-13）

親からsource所有を再受領した。実装者はfeature、設計private型と固定Err関数だけを追加し、Storage::open/load/resolver/外部出口/probe hookをまだ接合していない。作者は本体を編集していない。

最初の試用lib/bin/integrationまとめno-runはexit0で型エラーなし。ただしbuild中の作者self-reviewで前提assertを強化したため、そのbuildを最終snapshotの実行証拠にはしない。root未接合のまま通常側の壊れたpendingによるErrを受けてload再検査が通る偽陽性を避け、trial dir一致と変更前load成功を先に確認した。alias fixtureもtrial dir一致を先に確認し、root未接合をjunction作成失敗と混同しない。

ET-D01の実recover AfterDeck faultは、新preflight固定Errで遮断されないよう既存recoverへ直接到達させた。新preflightの読取成功/副作用なしは別テストのままであり、期待値は変更していない。これらをまとめて修正後、以下のsourceを固定した。

| file | SHA256 |
| --- | --- |
| Cargo.toml（実装者所有） | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |
| src/store.rs | 7E8AF45169AC2DA858CFC29190143C1C6DE28331680CF3C4D00899D3938699A0 |
| src/commit.rs | 996AFDABEEE47E70686F3546AB57C93FBEB5724E66B32EB00B9EB3E04D7851E8 |
| src/ai.rs | D53FE44EA6391412E26EEE44B84AF05475E2C9674626D6DD2F57EB92E638B3E8 |
| tests/early_trial_paths.rs | 3D7E7A45CCBE7E3944E2F091FAFB54EC0384B54005AD76F05A6A0AEAE6AC3F3B |

作者module境界は各fileの先頭から本体先頭の`use crate::{`（store/commit）又は`use base64::Engine;`（ai）の直前までである。先頭doc commentを含むUTF-8 bytesのhashはstore `442A2837BED7E83745C78487D721026A4B052452DD74A8FB8298528E6344E8D7`、commit `C624AC8332C4B37D1DF033A54A9F5C3E15870F54D57B9DECD3A27F2C63696D71`、ai `E609E5392402569B5E08F9170F5CA9C01EE7F5BEF49FAFE48BB3A2013D9977AD` である。本体stubのhashと独立oracleのhashを区別する。

固定sourceのcompileログは `trial-compile-fixed.log` と `default-ai-compile.log`。通常integration3成功のテスト本体は変更しておらず反復しない。追加したtrial専用assertは通常cfgでは除外される。runtime/listの結果は以下に追記する。

### 最終実測と分類

試用lib/bin/integrationまとめfreshcompileはexit0、通常bin ai focused compileもexit0である。追加の型エラー/fixture compile errorはなかった。試用lib list18件、integration list14件、ai list1件を取得した。通常は既取得integration list3件と今回ai list1件である。source宣言34、試用構成33、通常構成4の対応を確認した。共通のresolver/generic/aiは構成別に実行される同名3件であり、37 unique testと数えない。

| 構成・実測集合 | 件数 | runtime | 判定 |
| --- | --- | --- | --- |
| 試用lib `early_trial_` | 18 | 4pass/14fail/0ignored | 固定Err stubに由来する14fail、stubだけで成立する拒否3pass、既存recover実挙動1pass |
| 試用integration | 14 | 1pass/13fail/0ignored | 未登録generic契約1pass、公開root選択/起動拒否未接合13fail。junction1件は下記修正版で分類を補完 |
| 試用bin ai | 1 | 0pass/1fail/0ignored | 公開Config.cwdが通常namespaceを返す具体RED |
| 通常integration | 3 | 既取得3pass/0fail/0ignored | 変更していない通常caseを反復しない。段階1のsnapshot |
| 通常bin ai | 1 | 今回1pass/0fail/0ignored | 通常cwdと設定/環境/無起動のbaseline維持 |

試用の33構成caseは合計5pass/28failである。失敗28件は公開経路の具体RED14（integration13＋ai1）と未実装stubの失敗14（lib）に分ける。libの拒否3passは固定Errでも成立するため、bounded read/strict decode/history拒否の実装成功とは扱わない。実挙動を確認した試用側の2passは未登録generic外部保存と、既存recoverのAfterDeck実書込後失敗・pending保持・retry完了である。

具体REDの代表は `et01_resolver_is_pure_and_selects_build_namespace` で、leftが合成LOCALAPPDATA/WordWeave5、rightが同基点/WordWeave5-EarlyTrialである。aiも同じ差がcodex-workに現れる。history/alias等のprivate機能はまだ接合されておらず、多くのケースはroot/入場前提又はstubで止まる。各細分条件の深部まで実行済みとは主張しない。GREEN後の独立runnerが最終sourceで全集合をfresh実行する。

試用integration初回のjunction caseは「unsafe trial opened」の行動RED後、fixture cleanupがjunctionに`is_dir()`だけを使いremove_fileを選んでOS5になった。判定を混同しないよう、reparse属性0x400/ディレクトリ属性0x10を使ってjunction自体だけをremove_dirする修正を加えた。これは期待値変更でなくcleanup修正である。新EXEから該当1件をexact実行し、0pass/1fail、失敗点はunsafe openと親のchild失敗assertだけ、cleanup失敗なしを確認した。

**最終証拠はmixed snapshot**である。lib/aiとintegrationの残り13件は最初の固定snapshot、junction1件はcleanup修正後snapshotである。独立runnerへ全件fresh実行を引き継ぐ。最終integration source SHA256は `0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16`。他4fileと作者module hashは上表から不変である。追加compileは `trial-integration-cleanup-compile.log` exit0、その間はsourceを固定した。

| 実行EXE（target/debug/deps） | SHA256 | 使用 |
| --- | --- | --- |
| wordweave5-1e58dd7001d62083.exe | 75E8D0731C3366623FA8045C3A880BFFCD6CE426EC54584E541200A5E8848A86 | 試用lib18 |
| wordweave5-e0ab9f88db3adadd.exe | 12D2137B4816BCD1D0259696FB587ACBDE6F6CCDBF0EC600E2691D280BECFE43 | 試用ai1 |
| early_trial_paths-573116bfc845b427.exe（最初の固定snapshot） | 5020E20F923ECB851818F1B43DFE046AB8F9425830FE5D8930162C3D2952856E | 試用integration14 |
| early_trial_paths-573116bfc845b427.exe（cleanup修正後） | 6164D55302C9E75E75D4E1A0887029AD6E96FCEED04DC013F8ABCCD895655149 | junction exact1、現在のEXE |
| wordweave5-72e8adffa8f9ef2c.exe | D7157D69C92D6CBEEB5E0EAE523EEF55E7DE62C11A7290E34DA763A1A08C2A3B | 通常ai1 |

ログはすべて `target/early-trial-build/et1-author/` に置く。listは `trial-lib-list.log`、`trial-integration-list.log`、`trial-ai-list-fixed.stdout.log`、`default-ai-list.stdout.log`。Windows GUI subsystemのbin testではPowerShell通常起動のlist captureが空だったため、binはStart-Processの終了待機/標準出力redirectで取り直した。空の `trial-ai-list.log` を0件又は成功の証拠には使わない。

runtimeは `trial-lib-red.{stdout,stderr}.log`、`trial-ai-red.{stdout,stderr}.log`、`trial-integration-red.{stdout,stderr}.log`、`trial-junction-fixed.{stdout,stderr}.log`、`default-ai-runtime.{stdout,stderr}.log`。sandbox外の各呼出はWindowsIdentity取得成功・Administrator=False、固定SHA照合、Hidden、45秒上限とkill/reap、合成LOCALAPPDATA/TEMP/TMPに限定した。初回sandbox OS5制約の再試行は行っていない。

修正版junction runtimeの新TEMP残存0を確認した。初回cleanup不備の合成fixture残存1個は、正確な絶対path/targetを検証してjunction自体と空dirだけを非再帰で除去し、旧TEMP残存0を確認した。ログは保持した。実data、鍵、通常版EXE、API/マイクへの接触はない。

### 実装者・runnerへの返却

RED工程の完了境界は独立oracle、最小fixture、compile/list、上記分類と引継ぎである。試用実装/受入は未完了であり、固定Errによるpassやmixed snapshotをgate PASSへ昇格しない。作者は本体/feature/既存期待値を変更していない。今回のsource変更はload/aliasの前提assert、既存recover faultの独立化、reparse cleanupの修正だけである。

実装者は確定契約とprobe限定設計どおりroot/preflight/登録/外部保護/ai cwd/読取専用pending検査を接合する。新testを変更せず、BeforeWrites/Started、probe pathと9points、retry/保持の期待値を使う。本体不足を解消後は上記focusedコマンドと既定関連回帰を行う。独立runnerは最終source/EXEを固定して通常4・試用33の新集合と計画gateをfresh検証する。新たな仕様/API判断が必要なら親へ返す。

source所有は段階2の親宛明示messageで再度返却する。計画済みGREEN/runner/レビューを作者は担当しない。runtime model/effort metadataと開始/終了usage counterは引き続き未取得である。

## 段階3：最終recheck fixtureの作者再開確認（2026-10-14）

親から残る `store::early_trial_paths_tests::et05_final_recheck_observes_caller_and_both_registered_root_identities` 1ケースと本節だけを受領した。planner指定は `gpt-6-astra/high`、役は `ww-test-author`、受入は承認済みET-05である。AGENTS、開発7役、KPI、TDD skill全文を確認した。開始HEADは `61d0ef54a0d021db915773f038f8973673d96e79`、既存dirtyのstore/commit/ai本体と親usage記録を保持した。本再開ではRust sourceを追加編集していない。

### 旧証拠と今回の差

親受領の前回実装担当結果は試用32PASS/fixture1FAIL・通常4PASSであり、今回の新規実測とは区別する。旧 `recheck-diagnostic.stderr.log` はcaller/normalの後、trial root renameで `FIXTURE_FAILURE` / OS5になっていた。trialのrecheck/publish oracleへは未到達であり、製品挙動のREDとは数えない。原因はStorageの生存中app.lockであり、前回作者はStorageだけをdropするfixture修正とroot名付き失敗表示を適用した。登録OnceLockの解除や本体観測handleの変更、期待値の緩和はしていない。旧 `recheck-fixed-compile.log` は完了記録がなく、成功証拠に使用しない。

今回、修正済みsourceからfresh no-runを実施しexit0、2m25sで完了した。既存の警告はあるがcompile errorなしである。固定EXEのexact listは1 test/0 benchmarks、exact runtimeは **1PASS/0FAIL/0ignored**、exit0である。外側wrapperと隔離childの各1PASSは同じ1ケースであり、2件に合算しない。caller/normal/trialの各rootについてprepare後に実rename・別identityのdirectory作成を行い、recheck拒否、publish拒否、変更後ledger不変をすべて通過した。旧app.lock保持状態の再実行はせず、旧失敗ログを根拠に保持する。

```text
cargo test --locked --features early-trial --lib store::early_trial_paths_tests::et05_final_recheck_observes_caller_and_both_registered_root_identities --no-run
target/debug/deps/wordweave5-1e58dd7001d62083.exe store::early_trial_paths_tests::et05_final_recheck_observes_caller_and_both_registered_root_identities --exact --list
target/debug/deps/wordweave5-1e58dd7001d62083.exe store::early_trial_paths_tests::et05_final_recheck_observes_caller_and_both_registered_root_identities --exact --nocapture --test-threads=1
```

runtimeはWindowsIdentity/WindowsPrincipal取得成功、`Administrator=False`を起動前に確認し、true/取得不能を拒否するguard付きでsandbox外の通常権限にて実行した。list/runtimeそれぞれ固定SHA照合、Hidden、45秒上限とtimeout時kill/reapを使用し、実測timeoutなしである。隔離child側は既存30秒上限を維持する。`target/early-trial-build/et1-author/recheck-resume-runtime/{temp,local}` の合成環境だけを使用し、終了時temp/local直下とも残存0。実データ・鍵・API・RunAs/UAC・OS設定変更はない。

ログは `target/early-trial-build/et1-author/` 配下の `recheck-resume-compile.log`、`recheck-resume-list.{stdout,stderr}.log`、`recheck-resume-runtime.{stdout,stderr}.log`。同directoryがignoredであることを確認した。失敗の期待値は「root差し替え後にrecheckとpublishがErr」であり、fixture自体の失敗は今回0である。

### 固定snapshot・返却

以下は今回のsource/EXE識別である。store source/moduleはcompile開始前からruntime後まで不変、他sourceはcompile中とruntime後で不変、EXEはcompile完了後からruntime後まで不変を確認した。moduleは `src/store.rs` 先頭から本体 `use crate::{` 直前までのUTF-8 bytes（既存改行を保持）であり、先頭のpaths/probe両moduleを含む。

| 対象 | SHA256 |
| --- | --- |
| Cargo.toml | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |
| src/store.rs | 0B7F5D2B295FF1A1BC26F8D70586452E7762C9F492E4FAB5A5B5E582BFE9649B |
| store作者module | D2E3225B03A398FC7B2275F49B45010B85F9E5BD5A3F0DD6B4FFBFF3F733C23A |
| src/commit.rs | 7D2720329860CEB3DCE1D2B1B8A5F85DED14EFBDE20FDF556173EF861CDBA95E |
| src/ai.rs | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| tests/early_trial_paths.rs | 0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16 |
| target/debug/deps/wordweave5-1e58dd7001d62083.exe | B95E14BE4EFAC1A3BBF0282B1C51F0AC993F0A45082ED041912AA4CDA3AACAEE |

本再開の完了はこの1件のfixture成立と受入oracle到達の確認までである。親へ所有を返却し、本体担当によるStarted遷移のsticky化、最終sourceでの独立runner/Postへ進める。今回の1PASSを試用33件の同一snapshotでの全集合成功や全体gate成功に置き換えない。

collaboration taskは `/root/trial_recheck_resume`、環境で取得したthread IDは `01a137e2-5701-7692-a8e2-78b0a6f5871a`、compile exec sessionは `73903` である。指定model/effortと実行metadataを区別し、実行model/effort metadata、開始/終了token counterは未取得。runtime後のhash照合時刻は2026-10-14 09:53:20 JST、compile所要はCargo実測2m25s、開始時計は未記録である。usage集計は親へ欠測を引き継ぎ、作者から親記録を変更しない。

## 段階4：Started保持の独立小回帰（2026-10-14）

同じ作者・planner指定Astra/highで、親からstore paths test moduleの新2ケースと本節だけを追加受領した。本体は[実装記録](early-trial-paths-implementation.md)のsticky修正後、SHA256 `767D92D5649D4B1B09E2ED5D2492F3327DA578AB861CD15ADFF767D65FEE5BBD` で固定されていた。根拠は承認契約ET-01/ET-D01の「最初のmkdir/app.lock/probe直前にStartedへ移り、以後戻さない」である。

新private署名 `TrialPreflight::create_root(&mut self, stage: &mut TrialOpenStage)` と既存trial_preflight、実Windows identity、Temp/tree/有限childだけを使った。新seam、fake identity、検査bool、OnceLock resetはない。新2ケース以外のsourceと本体は変更していない。

| 実在test名（prefixは `store::early_trial_paths_tests::`） | AC | oracle |
| --- | --- | --- |
| et_d01_stage_readonly_recheck_failure_remains_before_writes | ET-01/ET-D01 | 既存の空trialをpreflight後に実rename/再作成し、create_root拒否でもBeforeWrites・ledger不変・app.lock未作成 |
| et_d01_stage_readonly_failure_after_real_mkdir_keeps_started | ET-01/ET-D01 | 不在trialへcreate_rootの実mkdir成功・Started・空dir・通常sentinel不変を確認。同じchecked/stageを保持してtrialのidentityを差し替え、2回目のcreate_root拒否でもStarted・ledger不変・app.lock未作成 |

後者はsticky状態の直接検査であり、single-call途中の競争を再現したものではない。旧版には新署名がなく、本作者は旧版のcompile不能をREDに数えず、旧不具合の動的再現済みとも主張しない。Startedの期待値は現実装から導出せず承認契約を使用した。

### focused実測とfixture修正

初回 `cargo test --locked --features early-trial --lib store::early_trial_paths_tests::et_d01_stage_readonly --no-run` はexit0（1m54s）。固定EXEでlist2件を確認し、runtimeはStartedケース1PASS、BeforeWritesケース1fixtureFAILであった。BeforeWrites fixtureはtrial内sentinelもpreflightで観測した状態でroot renameがOS5となり、段階oracleへ到達しなかった。失敗はcompile errorでも行動REDでもない。

試験用trial sentinel作成だけを除き、空の既存trialを差し替えるfixtureへ限定した。通常root sentinel、stage、ledger、app.lockのassertはすべて維持した。修正後の該当1件だけをfresh no-run（exit0、1m30s）→固定EXE/list1→exact runtimeで確認し、**1PASS/0FAIL/0ignored**となった。成功済みStartedケースは変更も反復もしていない。したがって新2ケースはそれぞれPASSの証拠を持つが、**作者の最終証拠はmixed snapshot**であり、同一最終EXEの2PASSではない。最終sourceの試用全件/通常GREENと独立runnerは後続担当へ引き継ぐ。

最終fixtureの再compileコマンドは以下である。list/runtimeは固定したEXEに同じ完全名と `--exact --list`、または `--exact --nocapture --test-threads=1` を渡した。初回だけは上記2件共通prefixを使用した。

```text
cargo test --locked --features early-trial --lib store::early_trial_paths_tests::et_d01_stage_readonly_recheck_failure_remains_before_writes --no-run
```

両runtimeはWindowsIdentity/WindowsPrincipal取得成功・Administrator=Falseを起動前に確認し、true/取得不能で拒否するguard、固定SHA照合、Hidden、45秒上限（child30秒）、終了wait付きである。timeoutなし、各回の合成TEMP/LOCALAPPDATA直下残存0。通常ユーザーのsandbox外実行であり、実データ、鍵、API、UAC/RunAs、OS設定変更は使用していない。今回の既存sentinel保持時rename拒否のOS内部原因までは特定していない。

ログはignoredの `target/early-trial-build/et1-author/` 内で、初回が `sticky-author-compile.log`、`sticky-author-list.{stdout,stderr}.log`、`sticky-author-runtime.{stdout,stderr}.log`、fixture修正後が `sticky-author-fixture-compile.log`、`sticky-author-fixture-list.{stdout,stderr}.log`、`sticky-author-fixture-runtime.{stdout,stderr}.log` である。元の失敗ログを保持した。

### 固定sourceと所有返却

最終sourceから新2テストの連続範囲だけを除いたUTF-8 bytesのSHA256は、受領source `767D92D5649D4B1B09E2ED5D2492F3327DA578AB861CD15ADFF767D65FEE5BBD` と一致した。これにより本体・既存test・共有helperの不変を照合した。`cargo fmt --all -- --check`、`git diff --check` は成功である。

| snapshot | store source SHA256 | store作者prefix SHA256 | libtest EXE SHA256 |
| --- | --- | --- | --- |
| 初回2件（Started PASS、BeforeWrites fixtureFAIL） | 8C114A1DE1CCD01E7C4EE727038100A25C1987AA581667BFF5A6EB747E077083 | 9A7B7DDA5034B028AC55006784C5CF6A17C4350AAE797F82BA4CC79F0A6E5E1E | C53A4C977C1BC273823C6B337D74FCE23A913783E062815E35B6DA71C76DD4B8 |
| 最終fixture（BeforeWrites exact PASS） | 20C1498B540395164076CECAF9B5ACA56568DE648D0712C3D9F487C53C0AF3F8 | 05DECCA1466F3E77627A7296ECFB782543CC5F91DF0F07F21328949C67CB5660 | E4D86950F31F61F6E6539BB3F80DA3FFDE925419C86391F0648FDF73CCD37867 |

EXE pathは両snapshotとも `target/debug/deps/wordweave5-1e58dd7001d62083.exe`、prefix境界は段階3と同じである。Cargo/integration/commit/aiの全file hashは段階3から不変。新しいsource宣言は合計36、試用構成は35（lib20、integration14、ai1）、通常は4のままである。この構成別予定集合を今回の実行件数に加算しない。

追加2テストと本節の所有を親へ返却する。実装役が最終sourceの試用全件/通常GREEN、次に独立runner/Postを担当する。collaboration task/thread IDは段階3と同じ、compile exec sessionは初回26194・fixture修正版50426である。指定model/effortはAstra/high、実行model/effort metadataと開始/終了usage counterは引き続き未取得である。
