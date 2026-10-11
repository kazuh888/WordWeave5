# I2a-R: 教材recover限定接合案

2026-10-08 / ACOUSTIC-HISTORY-001。担当は計画者指定 `ww-designer / gpt-6-astra / high`。API設計Skillを適用し、今回の所有は本書とdesign-i2.md §5/7のR限定統合だけである。source/test/build・計画・他文書は変更せず、実データ/API/UAC・公開・再委任を行わない。実行metadata・使用量カウンターは未取得である。

入力は [implementation.md](implementation.md) I2a-R/T、[design-i2.md](design-i2.md) §5/7（今回統合前SHA-256 `7E93546677EDE42F1B194A277BC222463866D2392D78524C69B033905BCF9024`）、[S限定設計](history-progress-design.md)、[M限定設計](history-migration-design.md)、[旧wire1 pending証拠](history-migration-tests.md) である。本書の予備静的PASS対象SHA-256は `F4891F76D26FEFA648582126BE2457DD5CFF494CCF76BCFE99D1528AC1AAA7A1`であり、契約を再設計せず正本へ限定統合する。

[N最終Post PASS/Closed](history-normal-save-review.md) は独立100 PASS / 0 FAIL / 0 ignored、前後11hash一致、阻止0である。接合現物はstore SHA-256 `93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387`、commit SHA-256 `C1E7D086D12BB872B8B756616030EE12EFF4E150065D72539B3BBAA68CE022AD`である。storeの共通decode/HistoryError変換はprivateのまま存在し、Rは§1の薄いbridgeだけを追加する。現在のrecoverは新Progressだけをdecodeし、old/new exact照合後のatomic結果とrename結果を直接返すため、旧Progress権限・closure・結果不明の接合が必要である。親の `artifacts/history-material-recovery-before/` snapshotは上書きしない。N Postと正本統合は完了したが、R本体は未着手、独立最終Preはこれからである。

## 1. 既存署名と最小private seam

公開recover(root) / save(root, deck, progress)、crate-private recover_under_guard(root, guard)、private recover_with_limits / recover_with_limits_under_guardの署名を維持する。gate取得は現行の外側入口1回であり、内部からStorage::at/openや公開recoverを呼ばない。

store側はprivate codecの型群を公開せず、次のcrate-privateな単一変換口だけを追加する。

```rust
pub(crate) fn decode_progress_for_material(bytes: &[u8])
    -> Result<(u32, Progress), HistoryError>;
```

Sのdecode_progress結果から元wire_versionとruntime Progressを返し、既存ProgressReadError::history_errorで分類を直接変換する薄いbridgeである。通常serdeと別の版判定を作らず、String本文をparseしない。返るProgressは検査済みデータであり、root更新権限ではない。

commit側の新規口は次に限定する。新しい公開fault APIや、callerが手製の「検証済Intent」を渡せる口は追加しない。

```rust
fn recover_with_limits_and_faults_under_guard(
    root: &Path, limits: Limits, guard: &ProgressWriteGuard,
    faults: &dyn crate::store::ProgressFaults,
) -> Result<(), String>;

fn recover_with_io_under_guard<W, A>(
    root: &Path, limits: Limits, guard: &ProgressWriteGuard,
    faults: &dyn crate::store::ProgressFaults,
    write: W, archive: A,
) -> Result<(), String>
where
    W: FnMut(&Path, &[u8]) -> Result<(), String>,
    A: FnOnce(&Path, &Path) -> std::io::Result<()>;

fn read_recovery_bytes(path: &Path, limit: u64)
    -> Result<Option<Vec<u8>>, HistoryError>;
```

既存under_guard wrapperはNoProgressFaultsを、新and_faults wrapperはproduction atomic_writeとfs::renameを渡す。whole recovery本体はwith_ioだけに置き、pendingを自分で読み全検査する。writeはdeck/progressに各1回まで、archiveは選んだpending/退避pathに1回までである。Mのhash退避にwrite seamを流用しない。closure・読取値・decode結果を差し替える能力は渡さない。

read_recovery_bytesは既存check_pending_metadataと同じtype/reparse条件を再利用し、絶対pathの祖先を通常directoryとして確認してからleafをbounded readする。祖先欠損/取得不能はIo、非directory・reparse・symlinkはUnsafePath、検証済み祖先下のleaf欠損だけNoneである。存在確認後のopen/read NotFoundはIo、metadata長とlimit+1読取の実長をともに制限する。元read(path, limit)を無条件に置換してTの新intent作成経路まで変更せず、Rのpending/deck/progress/退避照合だけに使用する。引数はコード内で組み立てたpathであり、Intentから保存先pathを受け取らない。

## 2. 全事前検査とM退避

以下の1–6は書込前の一つの検査段階である。ここでの拒否は両metadata/pending、既存backup、全tempとdirectoryのledgerを変更しない。

1. guard.ensure_writable→strict pending_checked。正常なroot配下にpendingがなければOkで終了する。pendingを存在確認後に読めなくなった場合はIoであり「復旧不要」にしない。pending exact bytesをbounded readして保持し、既存Intentへdecodeする。Intent外側のschema/省略可能field/未知fieldの従来扱いを本単位で勝手に変更しない。
2. 実pending bytesのintent上限350MB、embedded old/new deck各64MB、old/new Progress各100MBを既定Limitsで確認する。private小上限も維持する。新deckはparse_deckで検証する。旧deckはbounded exact比較用であり、旧deckのparse成功を新たな復旧条件にしない。
3. new ProgressとSomeのold Progressをともに共通bridgeでstrict decode/構造validateする。old_progress=Noneの場合だけold_root=None。Someの破損/未知版/上限超過をNoneへ落とさない。new_rootとold_rootはOption<RootRef>のhash/bytesを含む完全一致を要求する。A→None、A→B、old=None→Someは新closureが健全でも拒否する。
4. new_rootがSomeなら`HistoryReader::for_root(root, HistoryLimits::v1())?.verify_closure(...)`で全closureを検証する。Storageを再openせず、partial browsingや自己整合receiptだけで代用しない。old_root==new_rootなので同じclosureを別実装で二重検証しない。
5. 現在deck/progressをstrict bounded readし、それぞれがintentのold bytes（Noneを含む）またはnew exact bytesであることを確認する。各file独立にold/newを許し、両old・deckだけnew・両newに加え、deck old/progress newも整合した中断位置として扱う。第三状態を上書きしない。判定にruntime再serializeを使用しない。
6. 既存backupsと今回のpending退避leafについて、存在する祖先/leafの安全型と取得可能性を無変更で確認し、退避leafは未使用と確認する。backups未作成は許すがここでは作成しない。既存命名方式のprocess ID＋時刻を保持し、時刻取得不能・名前衝突は安全にErrとする。既存退避をrenameで置換しない。これは書込先検査であり、新しいarchive内容/schemaを導入しない。

全事前検査後、new_wire=2かつold_wire=1のときだけMのpreserve_v1_before_v2_under_guardへintent.old_progressのexact bytes、同じguard/limit/faultsを渡す。current progressが既にnewでも省略しない。先行退避が欠損なら作成・readbackし、同名異bytes/破損/未知版の既存退避はConflictで上書きせず停止する。この既存競合拒否も全ledger不変である。old=None、old wire2、new wire1ではR移行hookを呼ばず、new wire1の旧exact replayを維持する。

M検査後の実I/O失敗・AfterMigrationBackup faultでは確認済hash退避や新directory/tempが残り得るが、Rは両metadata/pendingを変更しない。確認済退避を削除しない。M callback後・最初のmetadata書込前にpending exactとcurrent old-or-newを再確認する。ここでの変化検出は初回事前拒否と区別し、既に作られたhash退避まで「変更なし」と主張しない。

## 3. metadata更新・fault・結果不明

退避確認後は既存どおりdeck→progressの順でnew exact bytesを書き、最後に両方を実再読する。new wire1でも元intent文字列のbytesをそのまま使う。現在既にnewであっても書込経路を別の省略実装へ分けず、再recoverでpendingがない場合だけ最初に終了する。

Rが接続するcheckpointはBeforeDeck/AfterDeck、BeforeMaterialProgress/AfterMaterialProgress、BeforeMaterialFinish/AfterMaterialFinishである。Before/AfterMaterialIntentはT、Before/AfterPointerはPに残す。Mの移行2checkpointは必要なR-M呼出内でのみ発火する。既存save_with_limits_and_faultsからのrecover呼出は当面NoProgressFaults wrapperのままであり、T前にfault伝播を広げてM/Sの既存seen順序を変えない。

writeのOk/Errはいずれもgate下の実再読で決着する。各書込後は両metadataとpending identityを再読する。対象fileがnew exact、相方もold/new、pendingが開始時exactならwriteがErrでも次の段階へ進める。対象がまだoldならwriteがOkでもErrとして停止し、pendingを保持する。faultで停止する場合も両metadataを再読し、後続書込を行わない。AfterMaterialProgressで両newを確認できてもpending退避前なら後処理待ちErrである。

| 再読した状態 | 分類 / 動作 |
| --- | --- |
| 両fileが各old/newの範囲、pending exact保持 | 既知の中断状態。失敗ならErr/no new block、後続saveはpendingで拒否、recoverで再試行可能。両newは後処理待ちでありpendingを隠さない |
| metadataに第三bytes、oldがSomeでnewにも一致しない欠損、path/I/O読取不能、pending消失/第三bytes | 結果不明。guard.block_writes()してErr、pending/metadataの自動再構成・rollbackなし。同process再試行で解除しない |

old==newのfileはnew一致を優先する。初回事前検査の不合格はwrite結果不明ではなく、新たなprocess blockを設定しない。全事前検査を通った後の故障境界では実再読で上表を使用し、faultが外側からbytesを変えていても安全を推測しない。純粋read_progress/headはblock中にも利用できるが、recover付きloadはGにより拒否される。

## 4. pending退避の確定

両metadataがnew exactでありpendingも開始時exactと再確認してからBeforeMaterialFinishへ進む。必要なbackups作成と退避先型/非存在の直前再検査後、開始時に選んだleafへrenameする。archive戻り値だけで成功にせず、AfterMaterialFinish faultを含め、両metadata・pending・選んだarchive leafを再読する。archiveは開始時pendingのraw bytesと照合し、Intentを再serializeしない。

| finishの実状態 | 戻り値 |
| --- | --- |
| metadata両new、pendingなし、archive exact | Ok。renameがErr、またはAfterMaterialFinish faultでも確定済みとして扱う |
| metadata両new、pending exact保持、archiveなしまたはexactの複製 | 後処理待ちErr/no new block。既存pendingを残し、次recoverへ委ねる。新しい成功表示を先行させない |
| 上記以外、read不能・archive異bytes・pending不明消失 | process write-block＋Err。archiveを消して再renameしたり、pendingを偽造して補わない |

BeforeMaterialFinish faultはrenameしない。両new/pending保持を確認して後処理待ちErrとする。directory作成等が失敗してrename未到達の場合も、両new/pending exact保持を再確認できれば後処理待ちErr/no new blockである。未作成backups配下のarchiveを読めないことだけで、未実行のrenameを結果不明にしない。rename実行後は表のarchive確認を必須とする。finish段階のdirectory/I/O失敗で残存物が生じ得ることは、初回事前拒否のledger不変とは別に記録する。後処理待ちのpendingがある間は、通常/history writerは既定のstrict pendingで拒否される。

## 5. 独立oracle・所有・未決

| ID / 受入対応 | 手製fixtureと独立判定 |
| --- | --- |
| R01 / T05、AC04/16 | old=Noneまたはwire1旧Progress、wire1 newの3既存中断位置を再現。disk exact wire1・元pending raw archive・媒体保持・再recover無変更・hash退避なし。decode後runtime2は別assert。追加のdeck old/progress newも許可する |
| R02 / T05b、I2-D1 | 手製intentのA→None、A→健全B、old=None→Some、old Some不正/未知版をcurrent各中断位置へ配置。全metadata/pending/backup/temp/directory不変。A→同AとNone→Noneを正例にする |
| R03 / T05、limits | pending/old/new deck/Progressのexact上限と1byte超過、無効new deck、new closureの欠損/未知/破損、第三current、root/ancestor/leaf reparse・I/O不明を個別化する。実際に失敗させた分岐を確認し、事前拒否で追加物0 |
| R04 / R-M | 手製old wire1/new wire2の両old/deck new/両newでhash退避欠損を再現。intent旧exact bytesを先に退避/readbackしてからmetadataへ進む。old=Noneは退避0、既存exactは保持、同名異bytes/破損/未知版は全ledger不変拒否。Before/AfterMigrationBackupと実I/O残存を区別する |
| R05 / T05、fault | 各metadataのwriteなしErr/Ok、実atomic後Err、Before/After checkpointで、known old/newならpending保持・次recover可能。第三bytes/再読不能/pending消失は子processでwrite-blockし、他writer拒否とpure readを確認する |
| R06 / T05、finish | rename前Errは両new/pending exactで後処理待ち。実rename後ErrとAfterFinish faultはarchive/pending/metadata実再読でOk。archive異bytes・不明消失は子processでblock。原pendingの空白を含むraw archive一致を確認する |

private write/rename seamは実filesystemの変更だけを注入する。たとえばrename後Errはfs::rename(from,to)成功後に合成io::Errorを返す。読取結果や検証済Intentを注入するbackdoorは作らない。block/poison fixtureはGと同じ子process隔離を使い、global flagをresetする口を追加しない。正例/拒否をclosure verifierの自己出力だけで判定せず、手製ID/root/metadata/全file ledgerをoracleにする。

所有は計画どおりcommit.rs本体/専用R cfg(test)、store.rsのcodec bridgeだけ、新tests/audio_history_material.rsの最大3fileである。既存G/M/S/N試験の編集は含めない。独立作者が署名/期待値を固定し、必要なら新しい未接続under_guard口だけを固定Errでscaffoldする。既存recover/saveをstubへつながず、行動RED後に接続する。N source所有は返却済みでも、R最終Pre前にstore bridgeを編集しない。

現物から期待値変更が必要な既存ケースは見つかっていない。commit旧`recovery_checks_each_next_file_before_changing_either`、`exact_serialized_intent_limit_can_be_recovered`、`finishes_half_written_commit_and_keeps_recovery_point`、`foreign_changes_are_preserved_and_block_further_saves`は維持する。最後のケースの「block」は既存pendingによる保存拒否であり、事前不合格でprocess flagを必須にするよう変更しない。M/Sの`m05_handwritten_v1_pending_replays_exact_bytes_from_each_partial_state`、`m05_new_v2_material_save_keeps_old_exact_intent_and_hash_backup`、commitの`m05_v2_material_private_entry_matches_public_save_with_exact_migration`も維持し、recoverからMのfault観測を既存save callerへ追加しない。変更必要が判明した場合はケース名・契約差・所有追加を親/M作者へ返し、assert削除・ignoreで救済しない。

Rは既存pendingの検証/復旧だけを閉じる。現在の新教材saveがpending作成前に旧新root/closureを保証すること、Before/AfterMaterialIntent、実保存Progress返却はT所有であり、R成功をその代替にしない。TはRの同じunder_guard本体を再利用し、必要ならfault伝播と重複するM checkpointの期待順序を別途固定する。

最終Preへ返す内部接合は、(1) typed codec bridgeの最小可視性、(2) known中断とunknown blockの境界、(3) rename後Err/AfterFinish faultをexact確認で成功とする既定結果不明契約の適用、(4) archive先の事前/直前無変更検査である。新しい外側Intent schemaやI3 root変更権限は未決事項を推測して追加せず、必要ならspec担当へ返す。正本§5/7へ統合済みであるが、R最終Pre・本体/独立試験は未達である。
