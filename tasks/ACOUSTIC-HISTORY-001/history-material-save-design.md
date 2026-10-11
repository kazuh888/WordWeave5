# I2a-T: 新教材save限定接合案

2026-10-08 / ACOUSTIC-HISTORY-001。担当は計画者指定 `ww-designer / gpt-6-astra / high`。API設計Skillを適用し、今回の所有は本書とdesign-i2.md §5/7のT限定統合だけである。計画/source/test/build・他文書を変更・実施せず、実データ/API/公開/再委任を行わない。実行metadata・使用量カウンターは未取得である。

入力は [implementation.md](implementation.md) I2a-TとT-M限定計画delta（SHA-256 `DDDD0E44D7B6F1A7FA76D0DBAA073EE33ECFAA0F2E2D0776D87D7DB9AD4C7B5A`）、[design-i2.md](design-i2.md) §5/7（今回統合前 `C4CF33CE6C7B69D8106D216497B4897C545C0E46169DEE417B7E27E96903B3D3`）、[R限定設計](history-material-recovery-design.md)（`0C1109DC7B41DBC2E0ACF7B01366FE6622DAB57404B76DB31E5D75A407DBE951`）である。本書の予備静的PASS対象は `9B805A51681B70A3105DA10D5042A1F529EA8A6D605034B6AA4A7D25AD33FD74`。

[R最終Post PASS/Closed](history-material-recovery-review.md) は独立125 PASS / 0 FAIL / 0 ignored、16hash一致、temp0、阻止0である。接合照合はstore `38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF`、commit `24414D9610D0970DDE11B09C17F62BB92051BD066245F5F316FD0F466A79C6A6`で行った。R最終Postと正本へのT限定統合は完了したが、T独立最終Preはこれからであり、T source/testは未着手である。

## 1. 役割と最小API

Tは新しい教材Intentを作る入口であり、metadata更新/復旧はRの同じunder_guard本体へ委譲する。Progressだけを先にNで保存したり、別recover実装を作ったりしない。教材Intentの既存4fieldとexact文字列を保持し、新しいschema/receipt/履歴root更新権限を追加しない。

```rust
pub fn save_progress(root: &Path, deck: &str, candidate: &Progress)
    -> Result<Progress, String>;

fn save_progress_with_limits_and_faults(
    root: &Path, deck: &str, candidate: &Progress,
    limits: Limits, faults: &dyn crate::store::ProgressFaults,
) -> Result<Progress, String>;

fn save_progress_with_intent_publisher<P>(
    root: &Path, deck: &str, candidate: &Progress,
    limits: Limits, faults: &dyn crate::store::ProgressFaults,
    publish_intent: P,
) -> Result<Progress, String>
where P: FnOnce(&Path, &[u8]) -> Result<(), String>;
```

公開save_progressはDEFAULT_LIMITS/NoProgressFaults、and_faultsはproduction atomic_writeを渡す薄いwrapperである。既存save、save_with_limits、save_with_limits_and_faultsは署名・Result<(), String>を保ち、この同一本体のProgress返却値だけを捨てる。最内側intent publisher入口でgateを1回取得し、Rへ同じguardを借用させる。公開save/recoverやStorage::at/openを内部から呼ばない。

publisherは新pendingの既定path/固定bytesへ最大1回だけ呼ぶ。metadata write/archiveの注入口はRの既定private seamを再利用し、Tへ第二のfilesystem注入口群を追加しない。codecはRのdecode_progress_for_material、raw readはRのread_recovery_bytesを同moduleから再利用する。raw readを公開化せず、storeの既存ProgressReadError型群も広げない。

R完成実物の`decode_progress_for_material(&[u8]) -> Result<(u32, Progress), HistoryError>`、`read_recovery_bytes(&Path, u64) -> Result<Option<Vec<u8>>, HistoryError>`、`recover_with_limits_and_faults_under_guard(&Path, Limits, &ProgressWriteGuard, &dyn ProgressFaults) -> Result<(), String>`は本案と一致する。最後の関数は既存recover_with_io_under_guardへ同じfaultsとatomic_write/renameを渡す。Tはこのまま呼び、RecoverySnapshotやRのtyped error分類/known-state/finish処理を別実装へ複製しない。表示変換はcommit内の既存recovery_error_messageを使用でき、store変更は現案では不要である。

## 2. 新intent作成までの順序

1. gate取得/ensure_writable→strict pending_checked。既存pendingあり/読取不明は拒否し、新saveから自動recoverしない。pendingなしは検証済みroot配下のleaf欠損だけである。
2. 現在custom.tsvとprogress.jsonをRのstrict bounded readで一度読み、旧raw bytesを保持する。旧ProgressがSomeなら共通codecでdecode/構造validateし、元wire版と最新rootを得る。旧Progressなしだけroot=None。旧deckは既存IntentのStringへUTF-8として保持し、旧deckのparse成功を新しい条件にしない。不正UTF-8・旧source超過・未知/不正Progressは変更なしで拒否する。
3. 最新rootがSomeならHistoryReader::for_root(root, HistoryLimits::v1())で全closureを検証する。candidateをcloneしaudio_historyだけ最新rootへ置換する。候補のNone/古いB/不正rootはroot変更要求と解釈しないが、置換後の非履歴fieldとruntime2はvalidateする。候補メモリを変更せず、非履歴fieldはすべて候補から保持する。新deckはparse_deckする。
4. root継承後Progressを一度serializeしてnew exact文字列を固定する。deck/new Progressの64MB/100MBと、旧metadataをembeddedしたIntentの実escaped bytes350MBを検証する。private小上限・ちょうど成功を維持する。new root==old rootは構築時にも確認する。ここまでの拒否はpending/metadata/backup/temp/媒体の全ledger不変である。
5. M helperへ旧Progressのexact bytes、target wire2、同じlimit/guard/faultsを渡す。旧wire1のhash退避/readbackが成功するまでpendingを作らない。旧wire2はstrict検証後に退避不要、旧fileなしも不要である。同名異bytes拒否・Before/AfterMigrationBackupの意味を維持する。以後の失敗では確認済hash退避が残り得る。日別backupはTから追加しない。
6. BeforeMaterialIntentを呼ぶ。失敗ならpending/metadataを変更せずErr、確認済hash退避を削除しない。成功後、pendingがなお欠損し、現在両metadataが保持した旧raw bytes/欠損とexact一致することを再確認してからpublish_intentへ進む。途中の外部変化を新しいoldへ取り直してrebaseしない。このpublish前の拒否は新しいprocess blockを設定せず、先行M残存と区別して報告する。

旧raw bytesはIntent作成・移行退避・pending結果確認の比較元を共有し、runtime2へ正規化したProgressを旧文字列へ再serializeしない。新candidateにroot=Aがあっても旧fileなしならNoneへ置き換えるため、手製candidateで新履歴を登録するbackdoorを作らない。

## 3. pending publishとRへの引渡し

publish_intent後はOk/ErrのどちらでもAfterMaterialIntentを呼び、そのfault結果も保持してpendingと両metadataを実再読する。faultやatomicの表示文字列で成功を判定しない。新metadataを一つでも書く前に以下で決着する。

| 実再読 | 分類 / 動作 |
| --- | --- |
| pendingが生成したexact bytes、両metadataが旧exact状態 | intent確定。publisherがErrでも先へ進める。ただしAfterMaterialIntent faultなら既知の中断Errとして停止し、pendingを残す（process blockなし） |
| pendingが欠損、両metadataが旧exact状態 | 未保存Err/no new block。publisherがOkでも成功扱いしない。確認済M退避は残す |
| pending第三bytes/読取不能、metadataがこの段階で旧exactから変化、root/pathの不明 | guard.block_writes()して要回復Err。Rを呼ばず、pending/metadataを巻き戻したり上書きして補修したりしない |

BeforeMaterialIntentの失敗ではpublisherを呼ばず、AfterMaterialIntentはpublisherを呼んだ試行で1回だけである。途中で旧fileが消えた場合は、保持したold=Noneと区別する。read helperのopen後NotFoundはIoであり、未保存の確証へ落とさない。

intent確定かつAfter faultなしの場合、Rの`recover_with_limits_and_faults_under_guard(root, limits, &guard, faults)`へ同じfaultsを渡す。Rはpendingを改めて読み、全旧新root/closure/limits/current preflightと必要なM退避確認を省略しない。T内で検証したIntentをRへ「検証済み」として渡さず、手製pendingにも通用する同じ本体を使う。Rのknown partial/後処理待ち/unknown blockとwire1 replay契約は変更しない。

RがErrならTもErrを返し、両metadataがnewに見えるだけで成功へ格上げしない。RがOkならgate下で両metadataが今回のnew exact文字列、pendingが欠損であることを確認してからroot継承後Progressを返す。これによりRの「pendingなしならOk」を今回の保存完了と無条件に同一視しない。この最終確認が不一致/読取不能ならwrite-block＋Errとし、過去snapshotへrestoreしない。archive exact確認はRの責任であり、Tは別archive名や別cleanupを導入しない。返すProgressは候補cloneそのものではなく、確認された保存bytesに対応するroot継承後値である。

## 4. fault順序と既存oracle差

TはBefore/AfterMaterialIntentだけを新たに接続し、RのBefore/AfterDeck・MaterialProgress・MaterialFinishへ同じfaultsを伝播する。Pointer checkpointは発火しない。R-M再確認を省いたりfaultを隠すadapterを導入したりして、過去のseen長へ合わせない。

旧wire1→新wire2の成功時に観測する順序は次である。

```text
T-M: BeforeMigrationBackup → AfterMigrationBackup
T:   BeforeMaterialIntent → AfterMaterialIntent
R-M: BeforeMigrationBackup → AfterMigrationBackup
R:   BeforeDeck → AfterDeck
     BeforeMaterialProgress → AfterMaterialProgress
     BeforeMaterialFinish → AfterMaterialFinish
```

M checkpointが2回現れるのは、pending前の初回退避と、pending読取後のR独立再検証という異なる保証のためである。既存exact backupでもMが再確認時に両checkpointを発火する契約を維持する。旧wire2/旧fileなしでは両M列がなく、T/Rの8checkpointだけとなる。R内2回目のMigration faultを狙う独立試験は、同variantの出現回数を数えるtest-local Faultsを用い、公開variantを増やさない。

既存commitの`m05_v2_material_private_entry_matches_public_save_with_exact_migration`はseenが移行2checkpointだけであるため、Tではこの成功時assertを上記12checkpointへ改訂する。plannerのT-M限定計画deltaにより独立M作者へ成功seen配列1箇所だけ正式割当済みであり、他assert/fixture/補助型/本体は割当外である。旧case→T fault伝播要求→厳密12件という対応を返し、旧2件の証拠を保持する。assert削除/両順序許容/件数だけのassertで救済しない。`m05_v2_material_migration_faults_never_publish_pending_or_metadata`は最初のT-M faultで止まるため、既存seen/残存ledgerを維持する。`m05_v1_material_preserves_old_source_and_escaped_intent_limits`もfault前拒否を維持する。

M/S integrationの`m05_new_v2_material_save_keeps_old_exact_intent_and_hash_backup`、旧wire1 pending3states、commit旧escaped上限/half-written/foreign-change oracleは変更しない。R単独recoverのNoProgressFaults wrapperと独立fault順序も変更しない。TからRへfaultsを渡す経路だけを新たに検証する。

## 5. 独立oracle・所有・未達

| ID / 正本受入 | 独立fixtureと期待 |
| --- | --- |
| T01 / T02、AC04 | 手製健全root Aのdisk＋候補None/古いB/不正root、新deck/全非履歴sentinel。返却値/diskはA＋候補値、candidate不変。旧fileなしでは候補SomeもNone。旧save wrapperも同じdisk結果 |
| T02 / T02/T04 | pendingあり/strict不明、旧Progress不正/未知、closure欠損/未知/破損、候補非履歴不正、旧/new上限、escaped intent超過をM前に拒否し全ledger不変。上限ちょうどを独立に成功確認 |
| T03 / T01/T05 | 日本語/空白付き旧v1 rawをhash退避し、pending内old_progress/old_deckが原exact文字列であることを確認する。M Before/AfterとBeforeIntent faultの残存を区別し、日別backupを追加しない |
| T04 / T04/T05 | pending publisherの書込なしOk/Err→未保存、実atomic後Err→確定してR成功、AfterIntent fault→metadata旧/pending exactで既知中断。第三pending/実read拒否/metadata変化は子processでblock。復旧は旧snapshotを上書きしない |
| T05 / T05、R接合 | full saveから各R checkpointを止め、R単独と同じpartial/後処理待ち/unknownを確認する。R-M2回目faultでもmetadata未変更・pending exact・既存hash保持。AfterFinish実確定はRのOkに従いactual Progressを返す |
| T06 / T04 | 同一gateのnormal/material/history競合と同root別表記で区間非重複、別root内容不混入。publisher callbackから公開save再入はGのReentrantで拒否しdeadlockなし。block fixtureは子process隔離、pure readは利用可 |

private pending publisherは実pendingのwriteと実filesystem故障だけを注入し、read値/closure結果/Rの検証済状態を差し替えない。T04の実atomic後Errは同atomic_write完了後に合成Errを返す。Rのmetadata/rename I/Oは既存R単体証拠を再利用し、T05ではfull saveからのfault伝播・返却値とledgerを確認する。実通信/解析開始・native UI・電源断受入の代用にはしない。

本体所有はcommit.rsと必要な既定store helper接合、新規file追加なしで既存tests/audio_history_material.rsをT独立作者が追加する最大3fileである。現案ではstore本体変更は不要であり、R bridgeをそのまま利用できる。commit内の専用T cfg(test)にpending publisher/fault試験を置く。上記M成功seen1配列だけは独立M作者の正式限定割当であり、T作者/実装者と同時編集しない。既存M/S外部test、旧100件/R25件のoracleはこの限定配列以外維持し、失敗を許容せず担当へ返す。R最終Post固定差分と所有返却を基点に、同fileの作者/実装者は逐次移譲する。

独立Pre前に公開saveを新stubへ接続しない。必要なcompile scaffoldは新save_progress/private publisher口だけを固定Errで追加し、既存save/limits/R入口は維持する。作者が契約正例の行動REDを固定した後に既存wrapperを同一本体へ接続する。

Rのprivate名/型一致とM成功seen1配列の所有割当は確認済みである。最終Preへ返す論点はpending結果不明のblock境界、R成功後のT最終exact確認、同一faults伝播とT-M/R-M再検証の厳密順序である。新しい外側Intent schema、I3 union root権限、root保持以外の候補merge、品質gate緩和は導入しない。T成功後もconsumerが返却値をUIへ採用する接合、V/P、I3/I8/I10は未達である。正本統合後もT最終PreまではT source/testへ着手しない。
