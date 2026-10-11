# I2a-P: 履歴pointer確定の限定接合案

2026-10-08 / ACOUSTIC-HISTORY-001。計画者指定は `ww-designer / gpt-6-astra / high`。API設計Skillを適用し、今回の所有は本書の状態/実helper照合と正本design-i2.md §4/7のP限定統合である。計画/source/testを編集せず、build/test・実データ/API・公開・再委任を行わない。実行metadata・開始/終了使用量カウンターは未取得である。

今回の入力は本書SHA `80DD647FFA358F253F9C355F9D485A0975D54773A4DD53B6CB986640AFC045EE`、[正本](design-i2.md) §4/7 SHA `70749C50224F8051949487ABBFE436AEFE642637B98CA9B488BFE6484E6E92AF`、[計画](implementation.md) P行、[V限定設計](history-publication-authority-design.md)、[I1契約](design-i1b.md) である。V最終Post PASS/Closedを親から受領し、[独立runner](history-publication-authority-runner.md) の101 PASS（旧82/新19）、8hash一致/temp集合差0、V08訂正後証拠を参照した。現store `38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF` と完成V audio_history/store `224F2E6CE2BADBD729205EE0BFDC371C2CB01D93BB4EEB5B50D1427EE92B278B` を実helper照合した。P独立最終Preは未了、P sourceは未着手である。

plannerのV-P追補（割当時implementation.md SHA-256 `61980333CCFF2ED1DD294B6013A7A6349F0518B07C7BEF5B68BB01C75AF243FC`）により、readonly confirm抽出はVで完了した。今回P契約を正本へ限定統合したが、P独立最終Preを代替しない。計画全文は親の完了記録で更新されるため、上記割当時SHAを現計画全文hashとは扱わない。

## 1. 入口・所有と読取接合

既定の公開型/署名を維持する。Progress全体、head adapter、検証済フラグをcallerから受け取らない。

```rust
pub enum HistoryCommitOutcome {
    Committed(CommittedHistory),
    CancelledBeforeCommit,
}
impl Storage {
    pub fn commit_history(&self, published: &PublishedHistory,
        cancel: Option<&AtomicBool>) -> Result<HistoryCommitOutcome, HistoryError>;
    fn commit_history_with_limit_and_faults(&self, published: &PublishedHistory,
        cancel: Option<&AtomicBool>, limit: u64, faults: &dyn ProgressFaults)
        -> Result<HistoryCommitOutcome, HistoryError>;
    fn commit_history_with_publisher<P>(&self, published: &PublishedHistory,
        cancel: Option<&AtomicBool>, limit: u64, faults: &dyn ProgressFaults,
        publish: P) -> Result<HistoryCommitOutcome, HistoryError>
    where P: FnOnce(&Path, &[u8]) -> Result<(), String>;
}
```

公開入口は100MB/NoProgressFaults、private wrapperはatomic_writeを渡す。最内側だけがGのgateを1回取得し、副作用・最終再読まで保持する。private publisherは実diskへの1回の書込結果だけを差し替え、読取値・receipt・V検証の代替値を注入しない。通常saveへroot変更能力を追加せず、save_progressやloadを内部から呼ばない。

**V完成版との接合照合**：HistoryWriterの `&mut Storage` 借用は変更されていない。Pの `&self` からwriterを構築せず、実装済みの次のcrate-private接合を使う。verify_publication_under_guard(&Storage, &PublishedHistory, &ProgressWriteGuard)とconfirm_publicationの実署名は本案/V契約に一致する。verifyは同guardのensure_writable/strict intent再読/complete＋missing空を行い、confirmはpublication_readerと固定ProgressHistoryHeadから共有bodyへ委譲する。再gateもsource所有追加も不要である。

```rust
impl<'a> HistoryWriter<'a> {
    pub(crate) fn confirm_publication(storage: &Storage,
        published: &PublishedHistory) -> Result<CommittedHistory, HistoryError>;
}
// audio_history/store.rs private: existing confirm body, not a second algorithm.
fn confirm_with_reader_and_head(storage: &Storage, reader: &HistoryReader<'_>,
    head: &dyn CommittedHistoryHead, published: &PublishedHistory)
    -> Result<CommittedHistory, HistoryError>;
```

既存confirm_committedはself.reader(u64::MAX)とself.headを渡す薄いwrapperとする。追加associated helperは100,000 nodes/u64::MAX retainedのreaderと**固定ProgressHistoryHead**を渡す。Vのread_history_at/checked_old_plan_with_reader抽出を再利用する。head引数はaudio_history内部privateだけであり、P/public callerへ公開しない。recovery::guard→公開operation/request検証→予定nextのstrict closure/receipt→存在するintentのstrict検査と元計画再計算/affected照合→最新headのstrict closure/receipt完全一致→head/pending再確認、という現confirmの順序・error・cleanup済み成功条件を変えない。型追加によるauthorityの持出し、偽Storage、Storage再open、unsafeな共有borrow化は不要である。

このhelperはgateを取得せずwrite-blockも変更しない純粋readである。公開I1 confirmのwrite-block中の読取可能性を維持する。一方commit_history自体はwriter入口であり、gate拒否時に勝手にread-only成功へ迂回しない。store内の既存read_progress_bytes/decode_progress/M helperとcrate-private pending_checkedも利用でき、typed分類をString経由に戻す必要はない。実接合blockerは検出していない。

## 2. 同一guard内の順序とデータ所有

1. gate取得/ensure_writable、pending_checkedのstrict拒否、read_progress_bytes→decode_progressにより旧rawとruntime2を同じsnapshotから得る。leafだけの安全なNotFoundは旧None/default2であり、root/祖先欠損・reparse・I/O不明をNoneへ落とさない。未知版/構造/上限errorはtypedのまま返す。
2. confirm_publicationを固定disk headで呼ぶ。成功した場合は最新rawが手順1とexact一致し、返却current==snapshot.rootであることを再確認し、そのCommittedを返す。取消済みでも成功、pointer/M/publisher/fault呼出0である。nextそのものでも、同receiptを保持する後続rootでもよい。intent cleanup済みも現I1条件で成功する。operation一致/request相違、receiptのaffected/revision相違、壊れたnext/current、存在する不正intentは拒否する。単にreceiptのrequestだけを比較して成功としない。
3. confirmのBusyだけをV必須再検証へ進む候補とする。Busyは未commitの証明ではない。base上receiptなしのほか、checked_old_plan経由のSetTrashに対するRunning/Sending拒否もBusyを返し得る。Conflictを「未commit」と読み替えない。最新snapshot.root==published.baseを改めて要求し、同じguardでVのverify_publication_under_guardを呼ぶ。Vのstrict intent/exact mutation/全計画/実publish完了検証は必須であり、自己整合next/receiptやBusyだけで代用しない。Vで同じmutation拒否が再現されればpointer前で停止する。V完了後にも最新Progress raw==旧snapshotを確認し、検査中の非履歴変更を見落とさない。
4. snapshot.runtime Progressをcloneしaudio_historyだけpublished.nextへ変更する。設定/会話/教材/学習記録の全fieldをsnapshotから保持し、validate→pretty serialized exact size検査を行う。旧wire1はruntime2再serializeではなく手順1のrawをMへ渡し、preserve_v1_before_v2_under_guard(..., 2, limit, guard, faults)を通す。None/旧wire2は退避不要、旧wire1はcontenthash exact退避/readback必須である。pointer経路に日別backupを新設しない（正本§4のpointerは保存上限/移行退避契約である）。
5. BeforePointerを1回呼ぶ。Errでも旧/新/不明を実readで判定し、callback結果だけで未保存にしない。Okならstrict pendingと旧raw不変を再確認し、guard.ensure_writable後、cancelをAcquireで最後に読む。trueならCancelledBeforeCommit、falseなら直ちにpublish(progress.json, nextbytes)を1回試行する。この取消判定とpublishの間にworker待ち・callback・解析・外部送信を挟まない。BeforePointerの試験fault実装はAtomicBoolを変更してよいが、production callbackやgate再入は設けない。
6. publishのOk/ErrいずれでもAfterPointerを1回呼び、faultがErrでも必ず§3の実再読へ進む。publish試行なしのBeforePointer失敗/取消しではAfterPointerを呼ばない。取消flagが最後の読取後にtrueとなった場合も再読結果が優先し、new確定を取消しへ降格しない。

手順1–4の検証失敗はpointer変更0である。M以前の拒否は全ledger不変。M後のBeforePointer失敗/取消し等では確認済みhash退避が残り得る。原音/object/intentは回復候補として保持し、削除・rollback・旧Progress全体の復元は行わない。同期処理だけであり、cancel token/worker/UIの寿命はI6所有、Pは借用だけである。

## 3. exact再読・保存済みauthority・error

結果境界ではread_progress_bytesで実diskを再読し、typed decodeとpending strict状態も確認する。旧wire1とruntime2を同値と扱わず、全文raw bytesで分類する。

| 実状態 | 戻り値と副作用 |
| --- | --- |
| intended nextbytesとexact一致、strict closure/confirm_publication成功、返却current==next、最後のraw再読もnext一致 | Committed。atomic Err/AfterPointer Err/遅い取消しより実証が優先する。write-blockしない |
| 旧rawとexact一致（旧Noneなら検証済rootのleaf欠損だけ）、旧rootのstrict closureが維持されpendingなし | 未確定Err、blockしない。注入faultまたはpublisherがErrならIo、エラーなしで書込効果なしならBusy。String本文を解析せず成功/失敗の別だけを使う |
| 第三raw、既存file消失、root/祖先消失、読取不能/破損、同じrootでも非履歴bytesだけ変更、newなのにclosure/receipt確認不能、pendingが出現/確認不能 | guard.block_writes、Io（要回復）。Saved/Cancelledを返さず、旧snapshotへ復元しない |

新pointerを試行した後の「後続rootにreceiptがある」は第三rawであり、この呼出しの成功へ広げない。同process writerはgateで排他されているためである。後日の新しい呼出しの入口では、§2手順2の後続root再確認を許す。既commit/read-only分岐やpublish前のsnapshot不一致は自分がpointerを試行していないためConflictで拒否し、新しいblockは立てない。ただしBeforePointer fault後に結果を確定する境界は上表に従う。

typed変換はProgressReadError::history_error、PendingCheckErrorのUnsafePath/Ioの直接match、MigrationErrorの同名分類（Gate Reentrant→Busy、Poisoned/WriteBlocked→Io）、ProgressFaultError::Ioを用いる。serde serialize/validate不正はInvalidDocument、実size超過はLimitExceeded、pending存在はPendingMaterialである。MのConflict等をStringにして再parseしない。新公開error enumや機密path/本文入りerrorは追加しない。

CommittedHistoryは既存の公開field型であり、token自体を偽造不能authorityと説明しない。保存済み表示の根拠は**この呼出しが実head/closure/receiptを確認して返した結果**である。callerはcurrent rootだけを最新UI Progressへ採用し未保存の非履歴fieldを保持する。I1 confirm/finishは必要時に自身で再検証する。finish失敗は保存済み＋後処理未完でありCommittedを未保存へ変更しない。Pからfinishを呼ばず、同operationの再試行で新ID/sequence/receiptを作らない。

## 4. 独立oracle・影響・handoff

| ID / 対応 | 独立fixtureと判定 |
| --- | --- |
| P01 / T03、AC04/08 | 手製base/next/intent/immutable objectsと非履歴sentinelを置く。public commitがrootだけ変更し、保存値・再起動read・closure/receipt一致。publish済みnextだけで正本がbaseの状態はSavedではない |
| P02 / T03b、AC04/16 | V01正例とV02–04偽Published/別intent/request/base/欠落objectsの代表case。全closureが有効な無関係Trial/receipt脱落nextも拒否。confirmの旧計画再計算でSetTrash対象のRunning/SendingがBusyとなるintentを別対照にし、Busyを成功権限にせずV再検証でも拒否、pointerpublisher0を確認する。全拒否でledger不変。V専用網羅を複製せずPの接続を確認する |
| P03 / T03 | 既commit next/後続root、intentあり/cleanup済み、cancel trueを組合せ、Committed.currentは実最新root、pointer/M/fault0。同operation別request・偽affected/revision・不正残存intentは拒否。finish失敗後再試行でもID/sequence/receipt/原音不増不減 |
| P04 / T03 | publisherがno-write Ok/Err、実atomic成功後Err、通常Ok、AfterPointer Errを固定する。exact old/new分類とstrict error kind、fault seen順序を独立固定する。BeforePointer Errにも実再読があることをprivate fixtureで証明する |
| P05 / T03/T04 | 第三root、同root別非履歴bytes、file消失、unsafe path、再読I/O不明、newbytesだがclosure破損を個別fixtureにする。Io＋全writer停止、pure read可能を子processで確認しblock reset口は作らない。fixtureの改竄物とproduction残存を分離する |
| P06 / T04、AC09 | 呼出前cancel true、BeforePointerでtrue、publisher開始後trueの3境界。前2者はpointer0/Cancelled、後者new確定はCommitted。原音/intent保持。cancel trueでも既commitはP03のCommitted優先 |
| P07 / T03/T04 | 旧wire1 exact原bytesのhash退避、旧None、旧v2、上限内/超過、M前後faultを検証。正常oldwire1 seenはMigration2→Pointer2、old2/NoneはPointer2。M後取消/Before faultでは正しい退避残存を許すが正本不変 |
| P08 / T04 | gate待機中に通常saveが非履歴fieldを更新した後Pが進み、その最新fieldを保持する。material pendingでは拒否。barrierを用い同root別表記/別rootのnormal/material/history区間不重複、データ非混入とdeadlockなしを既存G oracleに接合する |

公開integrationは新tests/audio_history_commit.rs、private publisher/fault/小上限・process隔離はsrc/store.rs内の専用cfg(test)とする。正本/planどおりPは2file所有である。Vのconfirm抽出は別の先行所有/レビューdeltaであり、Pに黙ってaudio_history/store.rsを追加しない。既存G/M/S/N/R/T試験とI1公開confirm/finish/cleanup/rebase/quotaの期待値変更は不要である。V抽出で既存confirmのerror優先・retained読取・custom headが変わる場合は回帰であり、assert緩和ではない。

test作者は新enum/3入口の未接続scaffoldだけを先に固定し、既存入口をstubへ置換しない。V helperはV本実装・Postに依存するため、その不在compile失敗をPの行動REDに数えない。独立fixtureの期待bytes/hash/receiptをproduction mutation planから採取しない。publisherはproduction atomic_writeを呼んだ後Errを返す最小口であり、電源断/kill耐久性の証拠ではない。OS I/O拒否は合成一時dirのhandle/path fixtureで確認し、単なるfault Ioと区別する。

V Post・実helper照合・P正本統合は完了し、残るgateはP独立最終Preである。Pの2file所有と既定公開型/署名を維持する。実機UIの保存済み表示、解析/POST開始0、取消遅延、電源断耐久性、外部writerによる連続改竄への完全耐性は未測定でありP文書で保証しない。V runnerが未測定とした最終head/intent再読中置換・pending metadata Io・実closure100k nodesを、Pの新成功証拠へ読み替えない。最終Pre後に独立test作者/実装者へ引き渡す。
