# I2a-N: 通常保存の限定接合案

2026-10-08 / ACOUSTIC-HISTORY-001。担当は計画者指定 `ww-designer / gpt-6-astra / high`。API設計Skillの加算的入口・型付き判定・結果不明の分離を適用する。今回の所有は本書の状態/参照更新と正本design-i2.md §4/7のN限定統合だけである。計画・source・testを変更せず、build/test・実データ/API・公開・再委任を行わない。実行metadataと使用量カウンターは未取得である。

入力は [implementation.md](implementation.md) I2a-N、[design-i2.md](design-i2.md) §4/7/8（今回統合前SHA-256 `5E4894D3A2EC7CAFA87115D45AD74B293DFAD01B2C8F436EF4D10D4379E2E714`）、Pre PASS済みの [S限定設計](history-progress-design.md)（`CE1B45349BD7DF4C20DBBC22ADF4C24888DB681EB7533A8244864830E891BF63`）、現store通常save/M hook/G gateとI1 readerである。Sの [最終Post PASS/Closed](history-progress-review.md) は独立81 PASS / 0 FAIL / 0 ignored、store SHA-256 `4C1287B2CDB4DE4621A39E06ED67DE2A181A272B7FB67507F4708C648FD00EEE`不変である。本書の訂正版入力SHA-256 `148C9FC97EAD6B5A93AEC001EE3E333D61A5029B3700B6A411B51C9BFEB354BD`を正本§4/7へ限定統合した。Sのruntime2/common codec/pure readを前提とするが、Nの予備静的PASS・本統合はN最終Preを代替しない。N最終Preはこれからであり、N本体は未着手である。

## 1. 範囲と最小API

既存`Storage::save`はgate付き通常保存の互換入口であり、現物は候補を直接serializeしてM hook→日別backup→atomic_writeのResultを返す。Nは最新disk rootを継承し、exact再読した保存値を返す処理へ置き換える。新rootを提案する権限、教材recover、履歴pointerのreceipt/CAS、UI反映は追加しない。

以下の入口をstore.rs内に固定する。pubは既定の加算APIだけであり、その他はprivateである。日別I/O検証deltaは単publisherの既定署名を維持した加算に限定する。

```rust
impl Storage {
    pub fn save_progress(&self, candidate: &Progress)
        -> Result<Progress, String>;

    fn save_progress_with_limit_and_faults(
        &self, candidate: &Progress, limit: u64,
        faults: &dyn ProgressFaults,
    ) -> Result<Progress, String>;

    fn save_progress_with_publisher<P>(
        &self, candidate: &Progress, limit: u64,
        faults: &dyn ProgressFaults, publish: P,
    ) -> Result<Progress, String>
    where P: FnOnce(&Path, &[u8]) -> Result<(), String>;

    fn save_progress_with_publishers<P, D>(
        &self, candidate: &Progress, limit: u64,
        faults: &dyn ProgressFaults, publish: P, publish_daily: D,
    ) -> Result<Progress, String>
    where
        P: FnOnce(&Path, &[u8]) -> Result<(), String>,
        D: FnOnce(&Path, &[u8]) -> Result<(), String>;
}

fn read_progress_bytes(root: &Path, limit: u64)
    -> Result<Option<Vec<u8>>, ProgressReadError>;

fn preserve_daily_backup_with_publisher<D>(
    root: &Path, old_bytes: &[u8], publish_daily: D,
) -> Result<(), String>
where D: FnOnce(&Path, &[u8]) -> Result<(), String>;
```

save_progressは既定100MB上限/NoProgressFaultsを渡し、with_limit_and_faultsはproductionのatomic_writeをpublishへ渡す薄いwrapperである。単publisher版はdaily用production atomic_writeを渡して最内側with_publishersへ委譲する。既存save、save_with_limit、save_with_limit_and_faultsの引数・Result<(), String>は保ち、この同一本体の返却Progressだけを捨てる。gate取得と通常保存本体は最内側with_publishersの1回だけであり、wrapperやdaily helperで再取得しない。callbackから公開保存へ戻らない。

daily helperはその場で日付/leafを1回決め、root/backups/leafのstrict metadata確認→既存通常fileなら保持してreturn→必要directory作成→新規leafの型/非存在再確認→publish_daily(path, old_bytes)の順に処理する。検査・作成・保存失敗の伝播はproduction/test共通本体1個に置く。daily publisherは新規分だけ最大1回、旧fileなし/既存同日fileなら0回である。Errは通常保存本体へそのまま伝播し、正本publisherへ進まない。日付・読取値・検証結果を差し替える口や公開APIは追加しない。

照合したS現物はread_progress_typed（store.rs:586）がpath検査・bounded read・末尾decodeを一体で行い、raw helperは未分離である。Nでこの関数の絶対path化から実bytes長の上限検査までをread_progress_bytesへ抽出し、末尾だけOk(Some(bytes))へ替える。既存progress_metadata（store.rs:561）はそのまま共有する。祖先のNone→Io、leafだけのNone、metadata.lenとtake(limit.saturating_add(1))による二段上限、open後NotFound→Ioの各分岐を移動時に変更しない。

既存read_progress_typedは`read_progress_bytes(root, limit)?.map(|bytes| decode_progress(&bytes)).transpose()`相当の薄い接続にする。署名・typed error・head/read/load側の呼出形は維持する。raw helper自体はdecodeしないため、publish後の第三bytesも実bytesとして比較でき、正常入力と誤認してはならない。N保存前は同じraw helperの結果を必ずdecodeしてから進む。公開S署名・DecodedProgressへbytesを追加せず、path検査の複製・二重readも作らない。最終S Post差分がこの対象を変えた場合だけ再照合し、別実装へ推測で接続しない。

Nの保存前読取は同じ1回のraw bytesを保持してdecodeし、保存前snapshot `(old_bytes, decoded_old)`をローカルに所有する。read_progress_typedで読んだ後に別readで旧bytesを取る形にしない。保存後はbytes helperから得た実際の値で判定する。旧v1の原bytesとruntime2を再serializeした値は同一ではない。

公開ResultのStringは表示境界だけである。gate/pending/read/codec/closure/Mのtyped errorはそれぞれ固定表示へ変換する。publishのOk/Err本文をparseして結果判定しない。通常保存専用の公開error型・checkpoint・filesystem frameworkは追加しない。

## 2. 保存順序とデータ所有

1. process共通gateを取得し、Gのwrite-block/poison/reentry拒否を維持する。strict pending_checkedを行い、pendingあり/不明なら復旧せず停止する。
2. read_progress_bytesで現在のprogress.jsonを読む。検証済み通常rootのleaf欠損だけold_bytes=Noneとし、既存bytesはSのcodec/構造validateを通す。root/祖先欠損・reparse・I/O・未知版・不正・旧source上限超過を新規扱いしない。
3. 最新decoded_oldのaudio_historyを採用し、Someなら既存`HistoryReader::new(self).verify_closure(root)`で全closureを検証する。partial read成功は代用不可であり、未知object/局所破損でもNoneへ退避しない。Noneなら履歴closureは存在しないため検証不要である。Storageを再openせず、I1既定上限を使用する。
4. candidateをcloneし、audio_historyだけを最新root（旧fileなしならNone）で置換する。その他全fieldはcandidateから保持する。候補の古いSome/None/構造不正rootはroot変更要求ではなく無視するが、置換後の非履歴field・version=2の構造は必ずvalidateする。候補を先に丸ごとvalidateして、使わない古いrootだけを理由に拒否しない。
5. 完成した候補を一度serializeしてnext_bytesを固定し、実byte長をlimitと比較する。ここまでの拒否はmetadata・backup・temp・候補メモリを変更しない。単にrootが同じでもsize/構造検証を省略しない。
6. M helperへ保存前のexact old_bytesとtarget wire2、同じguard/limit/faultsを渡す。旧wire1はhash退避/readbackを保証し、旧wire2はstrict検証後に退避不要とする。Before/AfterMigrationBackupの意味とMの失敗時残存物を変更しない。
7. 旧fileがある場合、§1の共通daily helperで既存の初回日別backup契約を維持する。既存同日fileを置換せず、新規分はold_bytesをそのまま保存する。backups directory/日別leafの型・reparse・取得不明は既存M path/metadata helperを用いて拒否し、existsのfalseへ落とさない。この検査はM退避確認後であるため、日別leafの不正を無書込で検出しても、手順6で作成・確認した旧v1の正しいhash退避は既に残り得る。全save開始前ledger不変とは扱わず、日別検査を前倒しして順序を変えない。新規日別分の実保存はproductionでatomic_writeを渡したdaily publisherを利用し、daily保存失敗なら正本publishへ進まない。日別backupは移行hash退避を代替せず、成功後も自動削除しない。日別検査拒否では日別leaf/正本/deck/pending/objectsを変更せず、Mで確認済みの退避（およびその作成に必要だったdirectory）を保持する。backup段階の実I/O失敗ではさらにdirectory/temp等が残り得るが、正本の旧bytesは変更しない。
8. guard.ensure_writableを再確認し、publishをprogress.jsonとnext_bytesで最大1回呼ぶ。帰り値のOk/Errにかかわらず、gate保持下でread_progress_bytesによるexact再読を必ず行い、§3で決着する。日別/hash backupの書込にこのpublish口を使わない。

候補メモリへ書き戻す操作はしない。成功時はnext_bytesに対応するroot継承後Progressを返し、consumerが後続単位でUI正本へ採用する。diskの他fieldを候補へ逆mergeしたり、失敗時に旧Progress全体をrestoreしない。履歴objectsはimmutableであり同processのwriterは同gateに従うため、closureはpublish前にstrict検証する。任意の外部プロセスが同時にobjectsを破壊する状況の完全防止まで追加しない。

## 3. exact再読と結果不明

publish結果ではなく次表の順で判定する。元bytesはwhitespace/field順も含むexact値であり、再serialize・runtime値の等価比較を代用しない。

| gate下の実再読 | 戻り値 / write-block |
| --- | --- |
| Some(bytes)==next_bytes | root継承後ProgressをOkで返す。publishがErrでも成功。oldとnextが同一ならこの分岐を優先する |
| actual==old_bytes（Some同士exact、または旧fileなしでactualもNone） | 未保存Err、blockしない。publishがOkでも成功扱いしない |
| 上記以外（第三bytes、既存file消失、破損・未知版相当bytes、上限超過、path/I/O読取不能） | guard.block_writes()後に要回復Err。process全writerを停止し、自動rollback・再publishしない |

readbackがnext_bytesと一致する場合、先にvalidateした内容そのものなので別の緩いparse判定へ切り替えない。比較不能なI/Oは旧状態とは断定しない。旧fileなしのNoneは正常なroot/祖先とleaf欠損を再確認できた場合だけであり、open後消失やroot消失はIo→blockである。

publish前の構造/closure/pending/移行/日別backup拒否は通常save正本の結果不明ではなく、Nが新しいblockを設定する理由にはしない。publish後の第三状態だけがNのblock境界である。block中のS read_progress/headとI1部分閲覧はgateを取得せず利用できる。ただし読めないfileをDefaultで隠さず、復旧付きloadはGのwrite-blockで拒否される。

## 4. fault seamと独立oracle

BeforePointer/AfterPointerはPの履歴pointer・保存済み判定用として残し、Nで発火させない。NのProgressFaultsはMの移行2checkpointだけを従来どおり渡す。normal publishの前後・rename結果不明は既存private with_publisherで注入し、日別I/Oだけは§1の追加daily publisherを使う。既存公開署名・enum variantを変えず、Pのcheckpoint順序をNの通常保存で消費しない。

private試験は同じ本体へ、(a) 書込なしErr、(b) 書込なしOk、(c) atomic_write(path, bytes)後Err、(d) 合成の第三bytesを実fileへ書いてErr、を渡す。読取値・候補・検証結果を差し替える口は作らない。file消失/再読不能のcaseも隔離fixtureで実際のfilesystem状態を変え、publisherが返す文字列で結果を誘導しない。

| ID / 正本対応 | 独立fixture・oracle |
| --- | --- |
| N01 / T02、AC04 | 手製健全root Aのdiskに対し候補None/旧B/不正rootを個別化。全非履歴field sentinelを候補へ設定し、返却値とdiskはA＋候補field、入力clone不変。旧fileなしは候補SomeでもNone。既存saveも同じ結果である |
| N02 / T02、AC04/16 | 未知/不正Progress、欠損/未知/破損objectのA、候補非履歴不正、旧source/候補の上限超過、pendingあり/strict不明を手順1–5で拒否するケースだけ、正本/deck/pending/objects/backup/tempの事前ledger不変。M以降の日別検査拒否へこのoracleを拡張しない。sizeちょうどは成功する |
| N03 / T01/T02、S-M | 日本語・空白付き旧v1 raw→runtime2候補の初回保存。hash退避/日別退避exact、next wire2、公開返却runtime2。M Before/After fault順序・残存物を維持し、旧v2に移行faultを追加しない |
| N03b / T01/T02、日別検査境界 | 通常backups配下に日別leafとしてdirectoryを置き、hash退避なしの旧v1を保存する。日別検査でErr、publish呼出0、正本/deck/pending/objects/既存日別leaf不変。差分はMで確認したexact hash退避であり、保持してblockは設定しない。既存一致hash退避あり/旧v2の対照では新たなhash退避を作らない。日別保存の実I/O失敗はこの検査拒否と別に残存物を採取する |
| N03c / T01/T02、実日別write I/O | with_publishers全通常保存経路から、初回日別leafの事前/直前検査を通ってdaily callbackへ到達する。callback内だけでdestを合成sentinelとしてcreate_newし、Windows share_mode(0)で開いたまま同atomic_write(path, oldraw)を実行して実Errを得る。通常保存Err・正本publisher0・正本/deck/pending/objects不変・new blockなしを確認する。既存leaf skipやN03bの検査拒否をI/O証拠にしない |
| N04 / T03通常保存部分 | 同じvalid old/nextにpublisher Ok/Err各々を組み合わせる。actual nextはOk、actual oldはErr/no block。old==nextはOkを優先。旧v1 rawを保持するno-write Errは第三状態と誤認しない。通常faultではPointer checkpointを見ない |
| N05 / T03/T04 | 第三bytes・旧file消失・実readback不能を各子processで発生させる。要回復Err後、normal/material/history writerが同じblockで停止し、別root/別表記でも解除されない。readableな第三Progressはpure readで読め、壊れたProgressはtyped読取エラーを維持する |
| N06 / T04 | 同rootへの競合saveをbarrierで開始し、publish callbackで区間重複なしを観測する。root Aを維持し、最後に確認した保存結果と返却値が各操作の候補fieldに対応する。別rootの内容を混ぜない。公開保存のcallback再入はGのReentrant拒否で停止し、deadlockさせない |

N05は既存Gの子process隔離方式を再利用し、親processのblock/poisonを汚染しない。global flagをresetするtest APIを追加しない。N01/N02のclosure fixtureは既存の手製I1 fixture構造を独立に作り、verify_closure自身の返す値だけをoracleにしない。実power-loss耐久性・UI・解析/POST・取消しはこの試験の受入ではない。

N03cは検査後に外部destが出現する隔離fixtureであり、callbackのsentinel生成をproduction残存物と混同しない。ledgerを(1) 開始時の既存物、(2) productionが確認したM hash退避/作成directory、(3) fixtureのsentinelと下記probe、(4) atomic_write自身のtemp、に分ける。現在のatomic_writeはErr時に自分のtempを除去するため、この共有違反caseではtemp0をassertする。共有lockはErrを取得するまで保持し、終了後もsentinel bytesがoldrawで上書きされていないことを確認する。

atomic_writeのString本文からOSエラー番号をparseしない。共有違反のraw OS根拠が必要な場合は、同lock下の独立した合成probeからdestへのfs::renameでraw_os_errorを観測し、probeをfixture ledgerへ明示する。その観測と、同atomic_write自身の実Err・callback到達・通常保存への失敗伝播を別々にassertし、probeだけでproduction失敗を証明したことにしない。ACL/UAC/OS設定変更や日付境界の時刻raceを利用しない。

この日別I/O検証可能性deltaは独立Pre対象であり、承認前にsource/scaffoldや作者試験を追加署名へ接続しない。既定の通常単publisher署名・2file所有（store本体/専用cfg(test)＋新save integration）・公開API範囲は維持する。検証基準を減らさず、既定N03bと新N03cを別の証拠として返す。

## 5. 所有・依存と未解決

既定N所有は`src/store.rs`本体/専用cfg(test)と新`tests/audio_history_save.rs`の2fileで足りる案である。S readのbytes抽出とM既存private wrapper維持もstore内に収まる。commit本体、I1 reader/writer、S/M外部testへの編集は許可しない。N独立作者の期待値固定→必要な未接続save_progress/private口のscaffold→行動RED→実装の順とし、scaffold時は既存save/M入口をstubへつながない。

S後に回帰対象とするケースは、SのS01–03 pure read/codec、S04/S05とM05通常保存移行（旧名`m05_v1_load_is_read_only_and_save_keeps_daily_exact_backup`）、storeのsource limit拒否（旧名`m05_v1_shrink_does_not_add_migration_read_or_source_validation`）、M Before/After退避・target1 no-opである。Nではこれらの承認済みoracleを変更する理由は現時点でない。S Postで確定した新名/件数を引き継ぎ、既存testのrename・期待値変更が必要ならケースと要求の差をM/S作者・plannerへ返す。独断で第3fileへ広げない。正常通常saveでrootありを候補から新規登録すると期待するfixtureがあれば、Nのroot権限契約との相違を明示し、assert削除や両結果許容で救済しない。

独立Preでは§1のS raw読取抽出が安全分岐を維持することと、日別backupを保存前snapshot bytesから書く接合を確認する。後者は旧日別file維持と新規分のexact内容を変えず、sourceを再serializeせず同じsnapshotへ揃えるための変更であり、保持日数・cleanup・復元機能は追加しない。日別leaf検査拒否でも先行する確認済みhash退避を保持するN03bと、実I/O失敗の残存ledgerを分けてreviewする。S Postと正本への限定統合は完了したが、N最終Pre・N本体は未達である。計画・品質gateの変更権限は本書に含めない。

N成功はNの通常保存だけを閉じる。Rのrecover/Tの教材save、V/Pの履歴commit権限、consumerの返却値採用、I8回復閲覧、I3 backup、製品提供は未達である。文書完成後は署名・既存gateとの整合・exact旧bytes比較・保存成功/不明の三分岐・2file所有を独立レビューへ渡す。
