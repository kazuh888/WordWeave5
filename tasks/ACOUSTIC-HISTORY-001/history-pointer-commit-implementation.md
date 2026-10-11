# I2a-P 最小scaffold実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。指定 `ww-implementer / gpt-6-astra / high`。実行metadata・usage開始/終了counterは未取得である。AGENTS/development-team/KPIの制約を維持し、wordweave-change全文を再読して実装役へ限定適用した。

親からP最終Pre PASS/阻止0と所有を受領した。入力はP限定設計全文 SHA `13DF70DAD3C247E83AACC384169B507145464C65E40007D347FCD59E4AE1891A`、正本§4/7 SHA `E53C18E88DCDD23F7FC843A3A363F4FE4108E0B8BE2F0689CE8DE30D18F568D7`、implementation.md P行である。文書内の過去の未Pre表記は親の最終Pre受領で解消している。

production変更はsrc/store.rsの加算block1つだけである。

- 941行 `HistoryCommitOutcome`：Committed(既存CommittedHistory) / CancelledBeforeCommit。独立試験で結果比較・診断できる最小deriveはDebug/PartialEq/Eqである。
- 947行 public `Storage::commit_history`：設計のpublished/cancelを受け、100MB/NoProgressFaultsを新private wrapperへ渡す。
- 960行 private `commit_history_with_limit_and_faults`：設計どおり新最内側へatomic_writeを関数値として渡すだけである。
- 970行 private `commit_history_with_publisher<P>`：設計のFnOnce署名を保持し、固定 `HistoryError { kind: Io }` だけを返す。

新入口同士の薄い委譲だけであり、最内側の引数はunderscore付きである。gate/I/O/取消読取/fault/publisher callbackは一切実行しない。atomic_writeは渡すだけで呼ばない。新型参照は完全修飾し既存importを変更せず、既存関数から新口への接続もない。V/P本実装、既commit/取消/保存済み判定、新testは今回未着手である。

`rustfmt --edition 2021 --check src/store.rs` と `git diff --check -- src/store.rs` はexit 0。既存testを整形していない。追加blockをメモリ上で除いた全文はbefore snapshotと完全一致し、UTF-8 hashも一致した。初回メモリ検査は除去範囲に1改行を残したため不一致だったが、検査範囲だけを訂正して一致を確認した。source修正による救済ではない。

| 対象 | SHA-256 |
| --- | --- |
| 開始source / before / 新block除去後 | 38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF |
| scaffold後source | F1E5DF2F27CDCA9F0C3DB7D364B77688CC90DA9D4900189D197C5D9AFEC20A06 |
| 全test suffix before/after | 1FB68B058230B246889833D2D39B308812DD5E20DE83E2BA239AAA1124C9FE97 |

親snapshot `artifacts/history-pointer-commit-before/src__store.rs` は上書きせず保持した。cargo/build/testは単独実行しておらず、compile・行動は未検証である。source/build所有を親へ返却し、作者の試験追加と一括compile/行動REDへ渡す。本体実装へは別指示まで進まない。他者編集を戻さず、再委任・実API/実data・UAC・commit/pushは行っていない。

## 2026-10-11 P本体実装・新22件focused GREEN

親からP最終Pre PASS、診断deltaの独立レビュー阻止0、新22件の独立行動RED、P本体所有を受領した。指定 `ww-implementer / gpt-6-astra / high` を維持し、AGENTS/development-team/wordweave-change/KPIとP限定設計・試験・レビュー記録を読んだ。実行model/effort metadataと開始/終了usage counterは未取得であり、起動指定から推定しない。今回の完了境界は履歴保存確定処理の本体と新22件のfocused検証・引渡しである。後続単位と旧関連回帰は実施していない。

変更は `src/store.rs` の既定P blockだけである。既定enum/public/private wrapperを維持し、`commit_history_with_publisher` の固定Io scaffoldを実装、必要なprivate helperを同blockへ加えた。

- 最内側で共通Gを一度だけ取得し、strict pendingと同一raw snapshotのtyped decodeを行う。既commitは完成Vのpure confirmで先に判定し、実head/currentとsnapshot全bytesの一致を再確認する。取消しより成功を優先し、pointer/M/faultを呼ばない。
- confirmのBusyだけを候補としてbaseを確認し、完成Vの `verify_publication_under_guard` でstrict intent・exact mutation・全objectのauthorityを必ず再検証する。最新snapshotの非履歴fieldを保持しrootだけ置換、validate/pretty sizeを確認して既存Mへ旧exact rawを渡す。
- BeforePointer失敗でも実readbackへ進む。正常時はpending/raw/gateを再確認し、取消しの最後のAcquire load直後にpublisherを一度だけ呼ぶ。publisherのOk/ErrにかかわらずAfterPointerを一度呼び、実new/old/third分類を行う。
- newはV confirm/current==next/最後のraw一致、oldはstrict旧closure/最後のraw一致を要求する。第三状態・確認不能は共通write-blockとIo、old未確定はfault/publish失敗ならIo、効果なしOkならBusyである。rollback・日別backup・finish・writer再構築・再gate・新公開seamは追加していない。

実装後の自己静的点検で、typed errorの直接変換、guardの保持、取消しとpublisherの隣接、既commitと新規authorityの分離、旧rootのretained上限、読取失敗時のblock分類を確認した。独立Postをこの自己点検で代替しない。`rustfmt --edition 2021 --check src/store.rs tests/audio_history_commit.rs` と `git diff --check -- src/store.rs` はexit0である。

### 不変証拠と固定build

P本体blockと作者の新P test節をメモリ上で除いた全文は既存before snapshot `38D4516D…12EF` に一致した。最初の逆除去検査はscaffold区切りLFを一つ残して不一致となったが、比較の除去境界だけを訂正して一致を確認した。sourceを変更して比較へ合わせていない。旧test suffix・新P module・integrationは受領時のままであり、診断分岐も保存した。

| 対象 | SHA-256 |
| --- | --- |
| 固定store全文 | FA9ECA988D4EDF5233E0198F8B2E090F760098D16779001179CBFFF151F38BEB |
| 全test suffix | FF92A2549B2B7CFCB7D8BE058CBC097931E494730CD4952954BB02232DD5B32E |
| 旧test suffix | 1FB68B058230B246889833D2D39B308812DD5E20DE83E2BA239AAA1124C9FE97 |
| 新P module | 8C91F3577B0113C9A7F13DB41D1C836F2C5AF1ED53282522A193449CC80BAF89 |
| 新integration source | 5CA5A171084BDD668D43E2DA85920D0655A9D4DA5D81316B4D349CB48697FFEC |
| lib EXE | ECB98C937815BF2ADC73DB53BD34EDE87A4A365C2A328327D3FB65A4B1AF4FB1 |
| 新integration EXE | F85D9234DC269D65EA4DB41A2DAAC81B27A53B941940A1CD5CDD65565B0F0847 |

親の追加指定により、固定sourceから次を一度だけ集約buildした。exit0、215.173秒、timeoutなしである。以後source編集はない。

```text
cargo test --locked --no-run --lib --test audio_history_commit --test audio_history_gate --test audio_history_save --test audio_history_material --test audio_history_progress --test audio_history_migration --test audio_history_store --test audio_history_recovery --message-format=json
```

9EXEの `--list` でlib279（新P18含む）、integrationはcommit4/gate16/save3/material6/progress4/migration3/store37/recovery34＝107件を確認した。lib全件や旧integrationの成功を意味しない。独立runnerは同じ9EXEを再buildせず利用可能である。固定16source/manifestと9EXEは検証後もすべて同hashであった。

### focused実行と後段到達

lib EXEへ `store::i2_pointer_commit_tests:: --nocapture --test-threads=1`、public EXEへ `--nocapture --test-threads=1` を渡し、新22件だけを実行した。lib18 PASS/0 failed/0 ignored（13.11秒）、public4 PASS/0 failed/0 ignored（0.97秒）、両exit0/外側timeout0である。自己実行の外側上限120秒、既定子process上限15秒を変更していない。

全親caseの完了により、P05の8 mode、P08再入1・待機1・same/distinct2・material/history1、計13子実行でfixture既定の成功status/個別marker/cleanup assertまで通過した。子のstdout自体は既存fixtureが取り込み、成功markerをassertする設計であるため、13件を独立受入件数へ加算せず、別採取した子process完全ログとも称しない。P04の実atomic失敗はOS error5、P05の実read拒否はsharing32であり、解除後同path/bytes成功対照まで到達した。全loop・worker join・ledger assertが完了している。

試験command内だけTEMP/TMP/WW5_I2_GATE_LIB_ROOTを新規workspace専用rootへ設定した。同じshellで祖先とrootのreparseなし・絶対path一致・collision拒否を確認し、前0→lib後0→public後0→終0、追加/消失0、残存対象testprocess0を記録した。空の所有rootだけを非再帰削除し、環境変数はfinallyで戻した。全TEMPの残存0とは主張しない。sandbox TEMPで起きた先行日別atomic拒否の根本OS要因は未解明のままであり、OS/UAC/ACL/harnessは変更していない。

証拠は `artifacts/history-pointer-commit-implementation-*` の `source-proof.log` / `source-fixed.json` / `build-result.log` / `build.jsonl` / `build.stderr.log` / `exes-fixed.json` / `list-*.log` / `list-counts.json` / `focused-results.json` / 各 `stdout.log`・`stderr.log` / `temp.log` / `integrity-end.log` に保持する。

未完は独立runnerの旧関連回帰と独立Post・利用者受入である。native UI、解析/POST開始、微小な取消Acquire直後の窓、V内部head/intent最終再読間置換、pending metadata Io、実closure100k nodes、電源断/kill耐久性、root/祖先消失の全OSパターンは今回の22件で実証していない。source/build所有を親へ返却する。実データ/実API/再委任/commit/push/mergeは行っていない。
