# I2a-V: publication権限の限定接合案

2026-10-08 / ACOUSTIC-HISTORY-001。担当は計画者指定 `ww-designer / gpt-6-astra / high`。API設計Skillを適用し、今回の所有は本書の状態/参照更新と正本design-i2.md §4/7のV限定統合である。計画/source/testを変更せず、build/test・実データ/API・公開・再委任を行わない。実行metadata・使用量カウンターは未取得である。

入力は [implementation.md](implementation.md) I2a-V/P、[design-i2.md](design-i2.md) §4/7のT03b、現audio_history/store.rs（SHA-256 `7532A07DA9BED7F79AE0B1E28806CFE881BE3B2AC5973E73F39F11F44B51FC67`）とrecovery.rs（`7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012`）である。T最終Post PASS/Closed（独立146 PASS、失敗/ignored 0、15hash一致/temp0）を親から受領した。本書入力SHA `F0E48A1AA5B93BEFAFDAE2E0EA668E5EE620C2249EDC1DFB31EB9693E0A9E833`、P案 `80DD647FFA358F253F9C355F9D485A0975D54773A4DD53B6CB986640AFC045EE` と上記I1現物を照合し、正本の統合前SHA `CBB39AE0518336ADD3C6E8B40F0703525F61287B24946D1362BEF814A556D306` の§4/7へ限定統合した。既存confirm/checked_old_planの接合は案と一致するが、V独立最終Preは未了でありV sourceは未着手である。

## 1. 権限境界と最小API

PublishedHistoryのbase/next/operation/requestはすべて公開であり、callerが構築できる。nextのclosureと自己整合receiptだけでは、無関係なTrial/receipt/sequenceを落としたrootを排除できない。Vはstrict intentに記録された閉じたmutationを最新baseから再計算し、公開tokenをその計画へ束縛する。pointer変更・成功表示・intent cleanupは行わない。

既定のpublication検証に加え、plannerのV-P追補（implementation.md SHA-256 `61980333CCFF2ED1DD294B6013A7A6349F0518B07C7BEF5B68BB01C75AF243FC`）が割り当てたreadonly confirm接合をcrate-privateで追加する。本deltaは今回正本§4/7へ統合したがsourceへは未接続である。割当・統合・予備静的PASSは独立最終Preの代替ではない。

上記計画SHAはV-P割当時の入力証拠である。親によるT行の完了記録後、統合時の計画全文SHAは `2D1AB82BCEB9E6E6A1CCE9DA767864B97EBF6464A5072E880BFD434436F9283D` であり、V行/V-P割当は不変である。本担当は計画を編集していない。

```rust
impl<'a> HistoryWriter<'a> {
    pub(crate) fn verify_publication_under_guard(
        storage: &Storage,
        published: &PublishedHistory,
        guard: &crate::store::ProgressWriteGuard,
    ) -> Result<(), HistoryError>;

    pub(crate) fn confirm_publication(
        storage: &Storage,
        published: &PublishedHistory,
    ) -> Result<CommittedHistory, HistoryError>;
}
```

guardの非公開構造とGのensure_writableを使用する。helper自身はgateを取得しない。検査Okは「同じguardでPがこの候補を検討できる」という時点の結果であり、保存済みreceiptや持ち運べる権限tokenではない。guardを解放した後のpointer書込へ検査結果を再利用しない。

公開HistoryWriter/PublishedHistory/PreparedHistory/CommittedHistoryの署名・field・可視性は維持する。verifyの引数にcaller指定head、未検証HistoryRoot、検証済フラグ、mutation overrideを追加しない。最新headにはstoreのProgressHistoryHeadを用い、純粋readで現在diskを読む。偽Storage、第三head、Storage再openを作らない。

## 2. 既存再計算を共有する内部接合

現HistoryWriterは`&mut Storage`を保持し、checked_old_planはselfを借用する。新helperは`&Storage`だけを受けるため、writerを新しく構築する方法では接続しない。audio_history/store.rs内で次のprivate読取関数へ既存bodyだけを抽出する。

```rust
fn read_history_at(
    reader: &HistoryReader<'_>, reference: Option<RootRef>,
) -> Result<CurrentHistory, HistoryError>;

fn checked_old_plan_with_reader(
    reader: &HistoryReader<'_>, intent: &RecoveryIntent,
    request: &FrozenHistoryRequest,
) -> Result<(mutation::RecoveryPlan, Vec<BlobRef>), HistoryError>;
```

read_history_atは現atのSome(root) strict closure＋root document読取と、Noneのrevision0/next_save_sequence1/空trials/receiptsという既存初期値をそのまま移す。既存atはself.reader(retained)を作って委譲し、既存quota/retainedに基づくreader上限を変えない。

checked_old_plan_with_readerは現checked_old_planのexact request/JSON照合、元baseの検証、mutation::recovery_plan(..., allow_missing=true)、計画root/objectsの順序付き一致、既存object bytes照合、missing列挙、missingなし時のclosure検証を同一bodyとして保持する。既存checked_old_planはself.reader(u64::MAX)で委譲する。新verifyも同じ100,000 nodes/u64::MAX retained読取条件を用い、文書/leafごとの既存size上限を維持する。新quota計算・object再publish・容量予約は行わない。

FrozenHistoryRequest::from_exactとRecoveryIntent/read_intentは既存private/pub(super)のまま使える。recovery.rsに新しい可視性は現時点で不要である。既存I1 resume/classify/confirm/finishのmissing許容をVに合わせて強化せず、Vだけが「すべて既にpublish済み」を追加条件にする。単にchecked_old_planの別コピーを作らない。

### V-P readonly confirm共有delta

PのStorage::commit_historyは`&self`であり、`&mut Storage`を保持するwriterを作って公開confirm_committedを呼ぶ方法は使えない。既存confirm本体を同じaudio_history/store.rsの次のprivate関数へ抽出し、二つの入口で共有する。

```rust
fn confirm_with_reader_and_head(
    storage: &Storage, reader: &HistoryReader<'_>,
    head: &dyn CommittedHistoryHead, published: &PublishedHistory,
) -> Result<CommittedHistory, HistoryError>;
```

- 既存公開confirm_committedはself.reader(u64::MAX)とself.headを渡す。readerの100,000 nodes/quota.limit_bytes.max(retained)、個別document/leaf上限とcustom head契約を維持する。小quotaだから既に保存済みの大きなclosureを確認不能にしない。
- 新associated confirm_publicationは100,000 nodes/u64::MAX retainedのreaderと固定ProgressHistoryHeadを渡す。callerからheadを受け取らず、diskと異なるheadを採用する入口をPへ追加しない。
- 共通bodyは、recovery::guard→operation/request構造→予定next strict closure/receipt/request→存在するintentのstrict読取・request/base/planned_root照合・exact mutation/旧計画再計算・affected照合→最新head strict closure/receipt完全一致→pending/head再確認、という現confirmの順序を維持する。read_history_at/checked_old_plan_with_readerを再利用し、検証を複製しない。読取前のguardを含むerror優先を抽出で変えない。
- 最新headにreceiptがなければcurrent==published.baseのときBusy、それ以外はConflictである。receiptがあればrequestだけでなくexpected receipt全体の一致を要求する。currentが予定nextでも同receiptを保つ後続rootでも成功し、CommittedHistory.currentは実際の最新headである。
- intentが残る場合はstrict検査/再計算を省略しない。intent cleanup済みなら現I1どおり予定nextと現在headのclosure/receiptから確認できる。missing intentを新commitのverifyで許容する意味ではない。confirm側の既存complete/missing条件をverifyの強いpublish完了条件に書き換えない。

両confirmはProgress/intent/object/backup/work/tempやwrite-blockを変更せず、gate取得もensure_writableも行わない。Gのguard保持中にも呼べ、write-block中の純粋readを維持する。新authority型・偽writer/Storage・再open・unsafe借用・cleanupは不要である。Pはこの結果を既commitの確認とpointer後の実証に使うが、未commitのpointer権限は別のverify_publication_under_guardだけから得る。Pの具体順序は [P限定案](history-pointer-commit-design.md) を参照する。

## 3. 新publication検証の順序

1. guard.ensure_writableを既存gate_errorでHistoryErrorへ変換し、recovery::guard(storage)で祖先/pathと教材pendingをstrict検査する。公開operation/request/base/nextの構造も検証する。failはtyped errorであり表示文字列を解析しない。
2. ProgressHistoryHead.read_current(storage)で最新rootを読み、published.baseと完全一致を要求する。履歴rootのhash/bytesを片方だけ比較しない。recovery::read_intent(storage, operation)でstrict intentとそのraw bytesを読む。欠損はMissingObjectであり、新規publicationには不可である。
3. intent.operation==published.operation、intent.request==published.request、intent.base==published.base==最新root、intent.planned_root==published.nextを要求する。read_intentが既に行うexact mutation_json hash==request、schema/未知field/size/operation-filename/planned ref検証も省略しない。from_exact(intent.mutation_json.clone(), vec![])を使い、mutationを再serializeしてhashを作り直さない。
4. 同じreaderで最新baseの全closure/root documentを検証する。baseに今回operationのreceiptが既にある場合はConflictとする。Vは新commitだけの入口であり、同operationのreceiptをmutationで置換する権限を与えない。既commit判定は§4のP別分岐へ返す。
5. checked_old_plan_with_readerで元baseと固定mutationから計画を再構築する。planned_rootと公開next、planned_objectsの順序/全refと再計算計画が完全一致する必要がある。必要payloadは既に存在するhash/len一致のimmutable leafから読む。caller bytesを補給しない。
6. 戻ったmissingが空であることを明示的に要求し、さらにplan.complete()を要求する。completeだけでは生成metadataのメモリ上bytesが揃っていても、その実fileが欠損する場合を排除できないためである。missingありはMissingObjectで拒否し、Vから再生成・publish・resumeしない。
7. nextのstrict closure/root document内に今回operationのreceiptがあり、request・affected（mutation対象1件）・committed_revisionが計画どおりであることを確認する。再計算したrootのexact bytes/ref一致がreceiptを含む全計画への束縛であり、receiptだけの自己整合を代用にしない。既存request、無関係Trial、receipt、保存sequenceの保持は同じ閉じたmutationの再計算で保証する。
8. 同じguardのままrecovery::guard、最新head==base、strict intentの再読raw bytes一致を確認してOkとする。途中でintentが消失/置換された場合はConflictとし、元snapshotへ戻さない。callback・fault注入・worker待ち・外部呼出をこの検査へ追加しない。

全経路はread-onlyであり、Progress/intent/object/backup/work/temp、quota、process write-blockを変更しない。既にblock/poisonなら既存Gエラーを返すが、V検査不合格自体を新しいprocess blockにしない。missing/damagedな新計画をpartial閲覧できることと、pointer候補に採用できることを混同しない。

## 4. エラーと既cleanup再確認の分離

| 不一致 | 分類 / 境界 |
| --- | --- |
| 公開tokenとintent/currentのoperation/request/base/next相違、既存operation receipt、検査中のhead/intent置換 | Conflict。公開fieldを修正して再baseしたことにしない |
| strict intentのschema/構造/size/不正hash/path、base/next/objectの読取障害 | 既存read_intent/readerのUnsupportedVersion/InvalidDocument/LimitExceeded/RelationshipMismatch/UnsafePath/Io等を維持。Stringから再分類しない |
| 再計算root/object列相違、既存objectのexpected bytes相違、Duplicate計画 | checked_old_plan系のRelationshipMismatch等を維持し、権限不一致を成功へ落とさない |
| intentなし、publish済みであるべきleaf/metadata欠損 | MissingObject。completeだけの成功やcaller bytesによる補修を許可しない |

Pはverifyを呼ぶ前にconfirm_publicationで最新正本のreceiptを検証する既commit分岐を持つ。この分岐は、intentがfinishで消えた後や、後続rootが同receiptを保持する場合の保存済み再確認であり、新pointerを書かない。verifyへ「cleanup済みならintent不要」という例外を追加しない。verifyはすでにcommit済みの元Publishedに対しても、新commit用検査として拒否してよい。

既存confirm_committed/checked_committed/finishはI1のreceipt/最新closure/intent cleanup整合を維持する。再計算helper抽出による回帰を検証するが、それらの既cleanup成功条件をVのintent必須へ置き換えない。PはVのOk後も同guard下で最新Progress/候補/取消し/保存順序を処理し、exact再読によりSavedを決める。Vはpointer APIを追加せず、G/M/S/N/R/Tにも新root更新権限を与えない。

## 5. 独立oracleと所有

| ID / T03b、AC04/16 | 手製fixtureと判定 |
| --- | --- |
| V01 正例 | disk Progressがbaseを指し、無関係Trial/receiptを含む手製base・正しいmutation/intent・全publish objectsを置く。helperがOk、全file/dir ledger不変、headはbaseのままである |
| V02 自己整合偽next | closure/receiptとしては有効だが、無関係Trial/receiptを除いたnextやsequenceを変えたnextを用意する。Publishedだけの差替えと、intent.planned_root/planned_objectsも整合させた偽計画を別caseにし、後者は再計算不一致で拒否する |
| V03 束縛 | operation/request/base/nextの各1箇所、planned_objectsの順序/省略/差替え、filename-operation不一致、exact mutationの空白変更を個別化する。fixture自身が狙ったstrict/関係検査前提を満たすことを確認し、無変更拒否 |
| V04 実publish完了 | 原音/result leaf欠損、生成Trial/root metadata欠損、hash/len/bytes破損を独立化する。生成metadata欠損はplan.complete可能でもmissing列が非空になる正例対照を持ち、VがMissingObjectで拒否する |
| V05 guard/head | 教材pending、pending I/O不明、最新headが別root、base/next局所破損、write-block/poisonを検証する。guard保持状態でhelperが再lockしない。block/poisonはGと同じ子process隔離を使い、reset口を追加しない |
| V06 既commit/cleanup境界 | 正本がreceiptを含むnext/後続root、intent cleanup済みの状態を作る。Vは新commit許可を出さず、既存confirm/finishの再確認成功とbytes不変を維持する。baseを最新へ偽装し同operation receiptを再利用する新mutationも拒否する |
| V07 readonly disk confirm / T03 | 新associated口でnext/後続root×intentあり/cleanup済みを確認しcurrent/receiptを固定する。baseでreceiptなしBusyと別headでreceiptなしConflictを区別する。request/affected/revision相違、不正残存intent、next/current破損、strict pending存在/不明の拒否を独立化し、全file/dir ledger不変を確認する |
| V08 confirm互換 / T03 | 同じdisk headを使う公開I1と新口の値/errorを一致させる。diskと異なるcustom headを公開I1へ渡した対照は従来のhead契約を維持し、新口だけがdiskを使う。小quotaを超える保存済closureの確認成功、100,000 nodes/個別document上限・error優先の保持も差分レビューで照合する |
| V09 confirm gate / T04 | guard保持中の新confirmが再lockせず完了すること、write-block中にも両confirmがpure read可能であることを既定子process隔離で確認する。verifyの書込停止拒否と混同せず、reset口やwrite-block変更は追加しない |

oracleはproduction mutation::recovery_planから期待rootを自動採取しない。既存手製I1 fixtureの構造を使い、期待ID/sequence/receipt集合、canonical bytes/hash、拒否前後の全ledgerを独立に固定する。テスト用caller指定headでVのdisk head読取を迂回しない。新公開試験API・新fault/checkpoint・state変更seamは不要である。外部writerによる同時改竄への完全耐性や暗号学的なintent署名を新たに主張しない。

所有は既定のaudio_history/store.rs本体/新専用cfg(test)、必要な場合だけrecovery.rs接合、必要追加時だけ既存tests/audio_history_recovery.rsの最大3fileである。plannerがconfirm_publication/confirm共有bodyもVに割り当て、Pのstore.rs＋新integrationの2fileは維持した。現案ではaudio_history/store.rs内のprivate抽出と新helperで足り、recovery.rs変更は不要である。crate-private helper試験は同file内cfg(test)とし、既存公開writer署名を広げない。第4fileや既存oracle改訂が必要なら親へ返す。

既存`w08_structurally_valid_but_forged_old_plan_is_recomputed_before_resume`、`w03_exact_mutation_whitespace_survives_restart_and_semantic_copy_conflicts`、`w03_committed_receipt_and_later_root_are_idempotent_and_cleanup_is_specific`、`w03_public_tokens_cannot_forge_request_receipt_affected_or_current_for_cleanup`、missing原音/metadataのrecovery分類、rebase・quota・finish後続rootのoracleは変更不要である。抽出がこれらを変えた場合は回帰として担当へ返し、Vのために既存assertを弱めない。作者/実装者は同fileを逐次所有する。

独立作者はV01–09の実在case名/--list件数を固定する。必要なscaffoldは新associated口だけに置き、既存confirmを未実装口へ接続しない。新正例の行動RED→GREENと既存回帰不変を区別し、compile不成立/0件/旧ログを新成功に数えない。runnerは `cargo test --locked --lib audio_history::`、既存audio_history_recovery/store targetで抽出前source・抽出後source/EXE hashを分けて記録する。本設計作業では実行しない。

最終Preの限定論点は、&mut writerを再構築しない単一confirm body、既存custom head/reader/error/cleanup互換、新confirmの固定disk head、completeと実file missingの区別、verifyと既commit確認の分離である。V本体はR/T最終Post・正本統合・V最終Preまで未着手、P本体はV Post・P正本統合・P最終Preを待つ。pointer変更・取消し・結果不明再読はP、保存済UIはconsumer後続であり、V成功だけでそれらをClosedにしない。
