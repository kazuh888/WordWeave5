# I2a-M 初回v2退避の限定設計

2026-10-08 / ACOUSTIC-HISTORY-001 / designer指定 `gpt-6-astra` / `high`。実行metadataと使用量開始/終了カウンターは未取得である。承認済み仕様のAH-AC04/16、[実装計画](implementation.md) I2a-M/S/R、[固定設計](design-i2.md) §2/5を入力とする。固定設計の確認SHA-256は `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA` である。

本書だけを所有する。Gのsource/test/buildと固定設計を変更しない。M実装はG Post、親による正本への統合、独立Preの後である。本書はその準備であり、実装・試験・gate通過を表さない。既読のAPI/interface-design Skillを型付きerror、exact入力の所有、結果不明の再読判定に適用する。

## 1. Mが確定する範囲

初回v2書込前に、旧v1の元bytesを `backups/progress-v1-<sha256>.json` へ保護し、exact再読確認する。既存同名異bytesは拒否する。日別backup・教材intentの存在だけでは、この確認の代わりにならない。退避の内容をruntime Progressから再serializeしない。

現物はstore.rsのsave_with_limitがvalidate/serialize/上限検査→日別copy→atomic_write、commit.rsのsave_with_limitsが旧metadata取得→intentサイズ検査→pending書込→recoverという順序である。Gのunder_guard最終形は未確定なので、symbolの接合はG Post差分へ照合する。MはDefault/Progress::validateをv1のままに保つ。v2有効化はS、recoverの全事前検査と移行hookはRが所有する。

「通常saveと新教材saveの2箇所だけで全first-v2-writeを保護できる」という案は採用しない。手製またはコピーされた妥当なpendingは、退避が無くてもrecoverへ到達できる。new wire版=2、intent.old_progressのwire版=1なら、recoverでも両metadata書込前に退避を保証する必要がある。

## 2. 最小内部APIと入力の出所

store.rsに次のcrate-private helperを置く。公開Progress/Storage/commitの既存署名は変更しない。汎用migration registry、複数schema engine、公開のraw JSON保存口は作らない。

```rust
pub(crate) fn preserve_v1_before_v2_under_guard(
    root: &Path,
    old_bytes: Option<&[u8]>,
    next_wire_version: u32,
    limit: u64,
    guard: &ProgressWriteGuard,
    faults: &dyn ProgressFaults,
) -> Result<(), MigrationError>;

pub(crate) enum MigrationError {
    Gate(ProgressGateError), InvalidDocument, UnsupportedVersion,
    LimitExceeded, UnsafePath, Conflict, Io,
}
```

old_bytesは借用であり、callerが寿命を保持する。許す出所は同じgate下で対象progress.jsonからbounded readしたexact bytes、または全事前検査を通過した教材intent.old_progressのexact UTF-8 bytesだけである。前者はnormal/new material、後者はrecoverで使う。raw bytesを外部callerからこのhelperへ直接公開しない。rootは処理中のStorage.dir/教材transaction rootであり、別rootを開かない。

helperは最初にguard.ensure_writableを行う。target=1ならその直後にOkを返し、移行目的のsource長/版/構造/path検査・読取・fault呼出し・書込を行わない。通常saveのcallerもtarget=1では移行用の旧disk読取を行わず、helperにはNoneを渡せる。このNoneは旧file不存在の判定ではなく、移行処理がno-opであるため入力を取得しないことを表す。新教材saveの既存旧metadata読取と上限検査は別のtransaction保護であり、省略しない。targetが1/2以外ならsourceに触れずtyped UnsupportedVersionを返す。

source長・必須wire版・既存構造の検証はtarget=2の契約であり、caller側のサイズ判定だけに依存しない。この場合は旧fileなしだけがNone、旧v1だけが退避対象である。旧v2はSのstrict codecで検証してから退避不要とする。M段階では旧v2の正当性を保証できないため拒否し、Sでこの分岐を接続する。

wire版は元bytesの必須整数versionである。Sでv1をruntime v2へ正規化しても、その前のsource wire版を保持して判定する。欠落/型違い/重複version、壊れたJSON、未知版をNoneやv1へ推測しない。Mは現行v1 deserialize/validateを利用し、Sは同じ検証箇所を正本の単一版codecへ接続する。Mのために完成版v2 codecを複製しない。未知top-level field等を含む完成版strict schemaの受入はSに残る。

各callerは候補Progressのvalidate後、その実際のserialize wire版をnext_wire_versionとして渡す。private fixtureだけがtarget=2で退避helperを直接検査する。この試験は、公開saveがv2を受理する証拠にはならない。limitは既存PROGRESS_LIMITまたは既存private小上限seamであり、新しい製品上限を設けない。errorの分類はenumで渡し、既存String入口で固定日本語へ変換する。OS本文・error文字列のparseで分岐しない。

## 3. 退避の順序と障害結果

1. gateを再取得せずensure_writableする。target=1は直ちにOk、1/2以外はUnsupportedVersionで終了する。以下はtarget=2だけに適用し、exact source bytes/長さ/必須wire版/構造を検査して退避要否を判定する。
2. 既存assets::sha256で元bytesをhashし、固定prefixとhashだけから退避名を作る。root祖先・backups・既存leafのsymlink/reparse/種類を検査する。既存backupsが通常directoryでない場合、leafが通常fileでない場合は拒否する。読取不明をNotFoundへ落とさない。
3. 既存同名fileがあれば上限付きで読み、長さと全bytesを照合する。一致なら既に保護済み、異bytesならConflictであり上書きしない。未知版/壊れたbackupも、検証済み旧v1 sourceとexact一致しなければ同じ拒否となる。別backupや日別snapshotの中身で代替しない。
4. 全検査後にBeforeMigrationBackupを呼ぶ。新規なら必要な安全なbackups directoryを作り、書込直前のpath再検査後、既存atomic_writeで退避する。既存一致の場合は再書込しない。
5. 書込がOk/Errのどちらでも退避先を再読する。sourceとexact一致する場合だけ保護済みと認める。欠落、異bytes、上限超過、再読不能はエラーであり、正本/pending更新へ進まない。
6. 再読確認後にAfterMigrationBackupを呼ぶ。このcheckpointのErrは「旧v1退避は確認済み、次のmetadata操作は未実施」である。正しい退避を消さず、callerは保存を停止する。

退避だけの結果不明は、正本pointerの結果不明ではない。退避を確認できなければcallerの次の書込を拒否するが、これだけで正本が変わったと判定しない。新しいprocess write-block基準は追加しない。source/pendingの強制rollbackも行わない。

| 状態/失敗 | 結果 |
| --- | --- |
| target=1 | gate確認後、移行処理はno-op。旧sourceの移行用read/検査/hash退避は追加しない。既存通常保存/教材transactionの検査・日別backupは維持 |
| 正常な親の下で旧progress不存在 | None。退避なし。root自体の読取不明/危険pathと混同しない |
| target=2の旧bytes不正・未知版・読取上限超過・I/O | 退避を開始せず、正本/pending/既存backup不変 |
| 同名退避が異bytes/壊れた内容、危険path | 上書き・temp作成なし。正本/pending/既存退避不変 |
| backup作成前fault | 書込なし |
| backup書込/renameがErr、再読がexact一致 | 退避確認済みとして扱い、AfterMigrationBackupへ進む |
| backup書込/renameがErr、再読が欠落/不明/異bytes | 保存停止。両metadataとpendingは未変更。既存退避を修復・削除しない |
| AfterMigrationBackup fault | 正しい退避だけが残り、次の正本/pending書込なし。同じsourceで再試行できる |
| 後続metadata保存が失敗 | 退避を保持する。通常保存/教材pendingの回復契約がその後の状態を決める |

「拒否時backup/temp不変」は事前検査での拒否について保証する。検査後の実I/O失敗・process終了ではdirectoryや新しい退避、残存tempが生じ得る。既存atomic_writeの自分のtempに対するcleanupを超えてGCしない。全filesystemが不変と説明しない。gate/app.lockによる協調writer排他を前提とし、外部processの敵対的なpath交換や電源断耐久性の保証は追加しない。

## 4. normal/new material/recoverの接合

| 経路/所有 | 必須順序とsource |
| --- | --- |
| 通常save / Mでhook準備、Sでv2到達 | 候補validate→exact serialize/上限検査→target版dispatch。target=1は移行用旧disk読取なしでhelper no-op→既存日別backup/atomic書込。target=2だけ旧disk bounded read→退避helper→既存日別backup→progress atomic書込。target=2では日別backupが既存でも退避helperを省かない |
| 新教材save / Mでhook準備、Sでv2到達 | deck/候補validate→既存どおり旧deck/Progress exact取得・上限検査→intent組立て→escaped intentを含む全サイズ検査→退避helper（target=1はno-op、target=2は旧bytes検査/必要退避）→pending publish→under_guard recover。hookより前にpendingを作らず、target=1を理由に既存旧metadata検査を省かない |
| 教材recover / Rでhook接続 | pendingのbounded decode→旧新Progress/版/旧新root同一性→新closure→現在両metadataのexact old-or-new照合→既存退避の検査→必要なら退避publish/readback→deck/progress更新→既存の再読/完了処理 |

recoverのsourceはintent.old_progressであり、current progressから取り直さない。既にcurrent=newでも旧bytesを保持しているintentから同じhash名へ保護できる。全事前検査に落ちたintentではbackup/tempを作成しない。helperのsource検証や既存同名退避検査も副作用前に終わる。旧新root不一致などを、退避作成後に判明させる順序は禁止する。

recoverでnew_wire=1なら移行退避は不要であり、正本設計どおり元intentのexact文字列を再生する。new_wire=2かつold_progress=Noneなら旧fileの退避は不要である。new_wire=2かつold_wire=1は、先行backupが欠損していても必ず作成/再読確認する。旧v2→新v2はstrict検査後に退避不要である。壊れた既存退避は上書きしないので、currentが既にnewでもその場で自動修復せず停止する。

M時点でRのfull validation/recover hookを実装しない。S→Rの中間状態では全first-v2-write保護は未完成であり、I2提供完了とは扱わない。既存履歴controllerの先行接続や、手製pending回復の受入済み判定へ用いない。

## 5. fault口と独立oracle

Mはstore.rsに公開`ProgressFaults::check(ProgressCheckpoint) -> Result<(), ProgressFaultError>`、固定`ProgressFaultError::Io`、NoProgressFaultsを置く。Checkpointの名前は正本§7を維持する。将来variantを型として宣言しても、Mで接続するのはBeforeMigrationBackup/AfterMigrationBackupだけである。新しい公開raw保存・filesystem差替えtraitは不要である。productionはNoProgressFaults、private limit付き入口へfaultsを渡し、public既存署名はそのwrapperとして保つ。後続の公開test用with_faults入口を作る場合も、元validate/gateを通す。

rename結果不明はAfterMigrationBackupでは再現できない。store.rs内部に、退避のatomic_write呼出しだけを置換できるprivate試験口を設ける。productionは既存atomic_write、cfg(test)では「実際に同path/同bytesをatomic_writeしてからErrを返す」publisherを渡す。helperの事前検査と書込後再読を迂回しない。検証結果や読戻しbytesを直接注入する口は作らない。無書込Errとの対比、書込後に読戻せない隔離fixtureで成功判定を独立に検査する。callbackからgate入口へ再入せず、Gの拒否契約も維持する。

| ケース / AC | 独立した期待値・範囲 |
| --- | --- |
| M01 / AC04/16 | 日本語・空白・field順を含む手製v1 exact bytesを使い、事前に固定したSHA-256名と全bytesを照合する。writerによる再serializeを期待値にしない。旧学習/会話/source/媒体file ledger不変 |
| M02 / AC04/16 | 日別backupありでもhash退避必須。同名一致は再書込なし、同名異bytes/壊れ/未知版は拒否し既存bytes不変 |
| M03 / AC04 | target=2でsource欠落・malformed・未知版・version欠落/型違い/重複、exact limit/超過、unsafe path、read失敗を個別化する。事前拒否でtemp/backup追加なし。未知targetはtyped UnsupportedVersionでsource処理なし |
| M04 / AC04 | Before/AfterMigrationBackup、無書込Err、実rename成功後Err、再読不能を分け、backup ledgerとmetadata/pending不変を検査する。Afterはbackupだけ残る |
| M05 / AC04/16 | 現行v1通常save/新教材save/旧pending replayの回帰。通常saveの旧v1 bytesがprivate limit超過でも、新v1候補がlimit内へ縮小する場合は成功し、hash移行退避を追加しない。既存日別backupは維持する。同じ旧sourceをtarget=2へ与えた対照はLimitExceededで正本/pending/backup/temp不変。private target=2 helper成功と公開v2受付成功を混同しない |
| S-M / AC04/16 | S後に実v2候補を通常saveと新教材saveへ通し、hash退避→正本/pendingの順を実観測する。new sourceなし・旧v1省略field保持・既存v2再保存も検査する |
| R-M / AC04/16 | 手製妥当pending old=v1/new=v2、backup欠損、current=old/deckのみnew/両new各状態から旧exact退避を保証する。old=None/new=v2、new=v1 replay、旧不正/未知版/root不一致/既存退避異bytes拒否も独立化する |

Mのprivate helper試験はstore.rs/commit.rs内cfg(test)へ独立作者が作成し、作者→実装者へ逐次所有を切り替える。`tests/audio_history_migration.rs`はM時点の公開v1回帰と、S後に追加する両公開入口の統合を担当する。v2 compile失敗やvalidate拒否を移行の行動RED/GREENと呼ばない。reader/hash helperの自己一致だけでなく、手製bytes/既定名/metadata ledgerを照合する。実行コマンドと件数は作者/runnerが対象作成後に固定する。実データ・実API・昇格試験を行わない。

## 6. 所有と正本への統合差分

Mの本体所有は`src/store.rs`、`src/commit.rs`、独立testは`tests/audio_history_migration.rs`の計3fileである。本書はこれらの編集許可ではない。G確定後のguard/private入口へ接合し、共有codecの完成はSへ、recover hookはRへ渡す。新module/manifest/依存追加は不要であり、必要になれば所有分割を親へ返す。

親が正本へ統合する差分は、(1) 初回退避の入力を現diskと検証済intentの両方にすること、(2) recoverのnew wire v2/old wire v1に限る退避保証をRへ追加すること、(3) 検査拒否時の不変と検査後I/O失敗の残存物を区別すること、(4) Mのraw bytes試験とS/Rの公開経路保証を分けること、(5) AfterMigrationBackupとrename結果不明の試験口を区別することである。

未達条件はG Post、正本統合/独立Pre、Sのwire版保持付き単一codec、Rの全事前検査後hookである。外部仕様の新しい製品値/UX判断は提案していない。Windows電源断・敵対的外部writer・実データ耐久性は未検証であり、本書の検査で保証しない。

独立レビューM-D1/P2への訂正：target=1にも移行目的の旧source上限検査を加えると、既存の通常saveで可能な「大きい旧v1から上限内の新v1への縮小」を拒否する。この回帰を防ぐため、target=1の移行処理と旧disk読取をno-opへ限定した。target=2の保護条件、新教材saveの既存検査、schema/gateは緩めない。M本体は未着手であり、G Post・正本統合・最終Pre待ちを維持する。

文書検査は相対リンク欠落0、行末空白0、Rust fence 1対、git diff --check成功である。固定設計のhashは変更前後で一致した。M-D1訂正後の再レビュー待ちであり、実装、試験は未実施である。
