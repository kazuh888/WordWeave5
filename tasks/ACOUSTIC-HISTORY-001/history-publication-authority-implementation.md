# I2a-V 未接続scaffold実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。指定 `ww-implementer / gpt-6-astra / high`、実行metadata・usage開始/終了counterは未取得である。AGENTS/development-team/KPIの制約を維持し、wordweave-change全文を再読して実装役の範囲へ適用した。

親からT最終Post146 Closed、V最終Pre PASS、作者PREPの2口確定とsource所有返却を受領した。V限定設計全文 SHA `A4F5199EA1BB7D7A4D4ADECC4FC76BC9BBFC6A3C05E4E3716E742D4A8A347215`、正本V接合 SHA `70749C50224F8051949487ABBFE436AEFE642637B98CA9B488BFE6484E6E92AF`、既定V/V-P所有を入力とした。文書内の過去の未Pre状態は親の最終Pre受領で解消している。

production変更は `src/audio_history/store.rs` の `impl<'a> HistoryWriter<'a>` へ加算した次のassociated関数2口だけである。

- crate-private `verify_publication_under_guard(&Storage, &PublishedHistory, &crate::store::ProgressWriteGuard) -> Result<(), HistoryError>`。
- crate-private `confirm_publication(&Storage, &PublishedHistory) -> Result<CommittedHistory, HistoryError>`。

全unused引数はunderscore付きとし、既存の `error(HistoryErrorKind::Io)` を使う固定Errだけを返す。I/O・gate取得・callback・他関数への委譲はない。既存confirm/checked_old_plan/at/その他入口へは接続していない。recoveryの公開化や変更、既存helperの抽出、本体実装は今回行わない。

新blockだけをメモリ上で除去した全文は親snapshotと完全一致し、そのUTF-8 SHAもbaselineと一致した。storeには既存cfg(test)節が存在しない。既存recovery本体と2本のintegration testはsnapshotの全文SHAと一致し、旧testを整形していない。`rustfmt --edition 2021 --check src/audio_history/store.rs` と `git diff --check -- src/audio_history/store.rs` はexit 0である。親snapshot `artifacts/history-publication-authority-before` は上書きせず保持した。

| 対象 | SHA-256 |
| --- | --- |
| store baseline / 新block除去後 | 7532A07DA9BED7F79AE0B1E28806CFE881BE3B2AC5973E73F39F11F44B51FC67 |
| store scaffold後 | 8D73115CE0EE0CBCA86F933F75B69C47E4F0ED6E24F6D3D9960752A20716D16C |
| recovery不変 | 7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012 |
| recovery integration不変 | AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567 |
| store integration不変 | 7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4 |

cargo/build/testは今回実施していない。compile・行動は未検証であり、固定Errを製品実装完了とは扱わない。作者が独立期待値を追加し、一括compile/行動REDを確認した後に本体実装へ進む。source/build所有を親へ返却する。他者編集を戻さず、実data/API/UAC/公開/再委任は行っていない。

## 独立RED受領後の本実装・初回自己確認

同日、同じ指定役/model/effortで本実装へ進んだ。入力設計SHAと最終Preは上記のまま、作者の新V19行動RED・旧82 PASS（lib11/recovery34/store37）を受領した。受領store全文は `19492438206F79FF2E0C3B556E5F66C16B322C846C7DD06D220771550FD2E195`、新V節は `2610E3AF318BC791F8051ED60AE1137D97033CC3060D5CAD7314EC0F6DA7B19B`。作者の初回旧recovery外側5分timeoutと限定再実行記録は保持し、今回へ成功を流用しない。

### production変更と固定

変更した本体はaudio_history/store.rsだけである。wordweave-changeに従い、独立期待値を固定したまま既存読取本体を次のprivate関数へ抽出した。

- `read_history_at`：既存atのstrict closure/root documentとNone初期値をそのまま共有する。
- `checked_old_plan_with_reader`：exact request/mutation、元base、順序付き全planned refs、既存bytes照合、missing列挙、missing空時closure検証を共有する。既存I1はreader(u64::MAX)へ委譲し、missing許容/complete条件を変えない。
- `confirm_with_reader_and_head`：既存confirmのguard→operation/request→next/receipt→存在intent再計算→最新head/receipt完全一致→guard/head再確認という順序とBusy/Conflictを維持する。公開I1は従来self.head、新associated口だけ固定ProgressHistoryHeadを渡す。両者はgate取得/ensure/書込/cleanup/block変更をしない。

新verifyは渡されたguardのensure、strict recovery guard、公開field構造、固定disk head==base、strict intentと公開tokenの全束縛、exact mutation、base内の既operation拒否、共有計画再計算、**missing空かつcomplete**、next receipt、最後のguard/head/intent raw一致を要求する。pointer・metadata・object・intent・quotaは変更せず、検査失敗で新しいwrite-blockを設定しない。新confirmにはこの新commit向け強化条件を混ぜず、cleanup済み確認とcustom head互換を保持した。

reader構築は有効な固定100,000 nodes/u64::MAX（既存atでは従来quota.max(retained)）の純粋検査だけであり、I/Oやエラー優先をguard前へ動かさない。個別document/leaf上限を変更していない。この点を含む独立静的reviewは親経由で阻止0を受領した。

15:52:10 +09:00に整形/source hashを固定して親へ通知した。[固定証拠](artifacts/history-publication-authority-implementation-fixed.log)。その後のbuild/test中source変更は0、recovery/全cfg(test)/integration期待値は不変である。

### Compile・固定EXE・初回結果

```text
cargo test --locked --lib --test audio_history_recovery --test audio_history_store --no-run
```

[集約compile](artifacts/history-publication-authority-implementation-build.log) は1回、exit 0 / 2m56sで成功した。[実行前3EXE path/hash](artifacts/history-publication-authority-implementation-exes-before.log)を保存し、非管理者tokenを検査して固定EXEを直接実行した。libはaudio_history::に限定し新V節を旧11件から分離、全実行は `--nocapture --test-threads=1` である。

| 群 | PASS | FAIL | ignored | 秒 | 証拠 |
| --- | ---: | ---: | ---: | ---: | --- |
| 旧audio_history lib | 11 | 0 | 0 | 0.68 | [old-lib](artifacts/history-publication-authority-implementation-old-lib.log) |
| 新V | 18 | 1 | 0 | 4.66 | [new-lib](artifacts/history-publication-authority-implementation-new-lib.log) |
| 旧store | 37 | 0 | 0 | 12.48 | [store](artifacts/history-publication-authority-implementation-store.log) |
| 旧recovery | 34 | 0 | 0 | 318.51 | [recovery](artifacts/history-publication-authority-implementation-recovery.log) |
| 合計 | **100** | **1** | **0** | — | 旧82全成功、新18成功/1 fixture失敗 |

旧recoveryは最初から外側15分監視を設定し、短いyieldで状態を報告した。製品60秒/512MiB基準、品質assert、OS設定は変更しない。[外側結果](artifacts/history-publication-authority-implementation-recovery-result.log)はexit 0 / timeout=False / 318.84255秒である。[phase/cleanup](artifacts/history-publication-authority-implementation-recovery.stderr.log)では100,000件作成96.594秒、100,001超過診断225.711秒、cleanup完了311.145秒。今回はtimeout・未完了・再実行なしで全34件を通過した。inventory境界成功をclosure readerの100,000 nodes実測へ転用しない。

### V08の初回失敗と未到達

`v08_retained_reader_quota_and_exact_root_document_cap_are_preserved` が `Fixture::head` 内（当該固定sourceの1112行）のProgress deserialize unwrapで失敗した。8MiBちょうどのextra0は旧/新confirm成功と末尾ledgerまで通過した。その次のextra1では8MiB+1のRootRefをdisk Progressへ入れようとしたため、既存 `RootRef::validate`（practice-history ids.rs、8MiB上限）とProgress codecに拒否された。旧/新confirmに到達する前のfixture前提不成立であり、製品戻り値が期待LimitExceededと異なった結果ではない。

extra1の旧/新confirm呼出・LimitExceeded assert・末尾ledgerは未到達である。初回logと失敗件数を保持し、全101 GREENとは扱わない。親経由の独立reviewerもP2 fixture前提不成立/本体阻止なしと判定した。限定修正は作者へ返され、合法disk headを保って公開token.nextだけを超過させる方法で厳密LimitExceededを維持する方針である。実装者はfixture/期待値を変更せず、新build/再試験もしていない。

V08以外の新18件は全loop/末尾assertを通過した。正当計画・偽next/refs/exact request拒否、生成metadata欠損とcompleteの区別、原音/result欠損、intent/原音実read32と解除後正常対照、junction、既commit/cleanup/finish再試行、custom headとdisk headの区別、held guard下の非再lock、block/poison後のpure confirmまで到達した。V child timeoutは0。poisonの意図的catch対象panicは別の失敗に加算しない。

### 固定値・残存確認・所有返却

| 対象 | SHA-256 |
| --- | --- |
| 自己確認store全文 | 2C0423C2C115BC5024881E39E5F5816A4EE2BAAFE20452D0FC2DF1A74D0A1589 |
| V節受領/最終 | 2610E3AF318BC791F8051ED60AE1137D97033CC3060D5CAD7314EC0F6DA7B19B |
| recovery不変 | 7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012 |
| recovery integration不変 | AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567 |
| store integration不変 | 7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4 |

3EXEは `F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/` 配下であり、実行前後hash一致を確認した。

| EXE | SHA-256 |
| --- | --- |
| wordweave5-b3388a7bbef44bb4.exe | A3B6AD40E5D0E8AE5F1CAD55730D6D939E8AB8BA707417C52DA19ADA1F55D9D4 |
| audio_history_recovery-b2b70d25a5608ed7.exe | 8E74C911E510B3CA49A3284080112A07FDB2FF3659C2CB67D1355A09BC1E7E5A |
| audio_history_store-e2ea7a2b1774fcfc.exe | FD354E5505A14C097E2646DE7A05E2D2899BD92CDDE2B6FE5F447A99FCC437EA |

[最終確認](artifacts/history-publication-authority-implementation-final.log)は16:02:36 +09:00。通常非管理者環境の実temp root `F:\TempCodex` を確認し、recovery/writer/i2-gateの所有prefix残存0、fmt/diff exit 0、追加削除なしである。source/build所有をこの確認後に親へ返却し、以後は本記録だけを追記した。作者のV08限定修正と独立再確認、runner/Postは未達である。

最終head/intent再読中の置換、pending metadata Io、新規100,000実object closureは引き続き未測定である。既存上限設定/文書境界/回帰の証拠とは分ける。pointer/P/UI、実データ/API/native/電源断、製品全体releaseは今回対象外である。実行model/effort metadata・usage開始/終了counterは未取得、推計なしである。
