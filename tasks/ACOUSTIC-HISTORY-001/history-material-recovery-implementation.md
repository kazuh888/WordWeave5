# I2a-R 未接続scaffold実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。指定 `ww-implementer / gpt-6-astra / high`。実行metadata・usage開始/終了counterは未取得である。wordweave-change全文を再読し、実装役の所有範囲へ適用した。

親からR最終Pre PASS、作者の4口署名確定とsource/build所有を受領した。限定設計全文 SHA `0C1109DC7B41DBC2E0ACF7B01366FE6622DAB57404B76DB31E5D75A407DBE951`、正本§5/7 SHA `C4CF33CE6C7B69D8106D216497B4897C545C0E46169DEE417B7E27E96903B3D3`、implementation.mdのR最大3file所有を確認した。

今回追加した新口は以下の4つだけである。

- storeのcrate-private `decode_progress_for_material`：設計どおり `Result<(u32, Progress), HistoryError>`、固定Ioを返す。
- commitのprivate `recover_with_limits_and_faults_under_guard`、`recover_with_io_under_guard<W,A>`：設計のguard/faults、FnMut write/FnOnce archive署名を保ち、固定の表示用Errを返す。
- commitのprivate `read_recovery_bytes`：設計どおり `Result<Option<Vec<u8>>, HistoryError>`、固定Ioを返す。

全unused引数はunderscoreである。新口は相互にも既存入口にも接続せず、callback実行・I/O・gate取得をしない。旧recover/save/read/codec/G/M/Nや既存testへ変更を加えていない。既存scaffoldではなく本体へ接続する作業は独立行動RED後の別指示を待つ。

親snapshot `artifacts/history-material-recovery-before` は上書きせず保持した。今回の追加blockだけをメモリ上で除くと、store/commitともsnapshot全文と完全一致した。最初のcfg(test)からEOFまでの全test suffixもそれぞれ完全一致した。

| 対象 | SHA-256 |
| --- | --- |
| store baseline | 93E76640B73966C331B4135B2FC1114933B8EAEEA65814226DD3C4740EFA3387 |
| store R scaffold | 3F6FEA29456D3C9C002B84C0D2D144574BD343DD7549D33D18D628F5FF6BBF2B |
| commit baseline | C1E7D086D12BB872B8B756616030EE12EFF4E150065D72539B3BBAA68CE022AD |
| commit R scaffold | E7492080DFB8FBE53BE5AD09C4421F95EE96BF94898C06BCE73331BE9C15B1AD |
| store全test suffix before/after | 1FB68B058230B246889833D2D39B308812DD5E20DE83E2BA239AAA1124C9FE97 |
| commit全test suffix before/after | F4CE4B49499FF933EBAD38E50C2532838268FC8884D055F690015DF885366A7E |

targeted rustfmt/diff check成功。cargo/build/testは親指示どおり実施せず、compile・行動は未検証である。source/build所有を親へ返却し、作者の試験追加・集約compile/REDへ渡す。他者変更を戻さず、実data/API/UAC/公開/再委任は行っていない。Rの復旧事前検査・結果不明/finish判定・R-M接合はまだ未実装である。

## 独立RED受領後の本実装・自己確認

同日、同じ指定役/model/effortでR本実装を実施した。入力設計・最終Pre PASSは上記のまま、独立作者の新25件=2 PASS/23行動RED、旧100全PASSを受領した。受領sourceはstore `3F6FEA29456D3C9C002B84C0D2D144574BD343DD7549D33D18D628F5FF6BBF2B`、commit `4C1976ECD79705DAA2DFABC6D8921ED56F1877CA5D36F84121AEA80BF5C64DE6` である。wordweave-changeを実装役に限定して適用し、固定した独立期待値と一時合成dataで停止・保存・再試行を検証した。最終独立検証・受入は別工程である。

### 変更範囲

- `src/store.rs` は `decode_progress_for_material` のstubを既存の単一codecへ接続しただけである。元wire versionと正規化runtime Progressを分離して返し、typed error分類を保つ。
- `src/commit.rs` productionにstrict raw reader、recovery状態照合、archive事前検査を実装し、既存underguard recoveryを共通本体へ接続した。pending原文・各field上限・新deck・旧新Progress・旧新root完全一致・whole closure・実metadata旧/新一致・archive先を、副作用前に検証する。
- R-Mはnew wire2/old wire1のときだけintent内のexact旧rawを渡す。現在のProgressが既にnewでも元intent由来のhash退避を先行する。M成功/失敗後もpending identityと両metadataを再確認する。
- 各writeは最大1回、返り値Ok/Errではなく実ファイルを共通再読して判定する。既知oldはpending保持の再試行可能停止、第三状態・検証不能はguard下のprocess-wide blockとし、rollbackしない。archive直前にも両newを再確認する。
- rename/AfterFinishの結果も実metadata・pending・選定archiveのexact再読で判定する。pending欠損＋exact archive＋両newだけが完了、pending exact保持とarchive無/同一複製は未完了である。
- G、N、他production、全test期待値は変更していない。material save側fault伝播はTの所有であり、今回も旧NoProgressFaults wrapperを維持した。original・history objects・他intentを削除しない。

### Build順序の補正を含む記録

集約コマンドは次である。

```text
cargo test --locked --lib --test audio_history_material --test audio_history_save --test audio_history_progress --test audio_history_migration --test audio_history_gate --no-run
```

1. [初回build](artifacts/history-material-recovery-implementation-build.log) はexit 0、3m14s。本体compile開始前と思われる依存build中に、archive準備後の両new再確認を追加した。しかし確認根拠は最終source hashとlog末尾のeframe行だけであり、rustc開始時刻との前後関係を独立に取得していない。このbuildからテストは実行せず、logを保持した。
2. その不確実性を残さないため、最終sourceの下記2hashをコマンド内で取得した後、同じ集約no-runを開始した。[固定source再build](artifacts/history-material-recovery-implementation-fixed-build.log) はexit 0、3m13s。以後production/testは変更していない。この6EXEだけを自己確認の根拠にした。
3. source変更なしでも再buildに3m13sを要した事実を記録するが、原因調査やharness/環境/品質gate変更は今回の範囲に含めない。次工程では「本体固定→hash→cargo起動」を維持する。今回の再buildは順序証拠不足の是正であり、成功済み試験の反復ではない。

### 固定EXE初回実行結果

非管理者tokenを明示確認した通常環境で固定EXEを直接実行した。libは `store:: commit:: --nocapture --test-threads=1`、integrationは `--nocapture --test-threads=1`。新R子processの結果は親caseに含め、二重加算しない。

| 対象 | PASS | FAIL | ignored | 秒 | log |
| --- | ---: | ---: | ---: | ---: | --- |
| lib（旧74＋R22） | 96 | 0 | 0 | 14.92 | [lib](artifacts/history-material-recovery-implementation-lib.log) |
| material integration | 3 | 0 | 0 | 1.71 | [material](artifacts/history-material-recovery-implementation-material.log) |
| save integration | 3 | 0 | 0 | 0.47 | [save](artifacts/history-material-recovery-implementation-save.log) |
| progress integration | 4 | 0 | 0 | 0.02 | [progress](artifacts/history-material-recovery-implementation-progress.log) |
| migration integration | 3 | 0 | 0 | 0.78 | [migration](artifacts/history-material-recovery-implementation-migration.log) |
| gate integration | 16 | 0 | 0 | 5.59 | [gate](artifacts/history-material-recovery-implementation-gate.log) |
| 合計 | **125** | **0** | **0** | — | 新R25＋旧100、全成功 |

本実装の初回行動試験に失敗はない。独立作者のREDや初回compile failureの記録は保持しており、上書き・成功への読み替えはしない。実施範囲でfixture failure/child timeoutは0である。

新R25の全caseが成功したため、作者記録のRED時未到達だった正常wire2/duplicate拒否、同hash異bytesを含むroot権限行列、whole closure、exact/1byte上限、reparse、R-M全中断位置・fault retry・callback後pending改変、各metadata/finish checkpoint以降のassertへ到達した。read共有拒否Win32 32、実atomic Errとread32、同path解除後atomic成功、実rename拒否32と解除後成功、rename後archive read32と解除後読取成功をlib logで観測した。unknown write 6行/finish 5行およびM callback改変の子processで、全writer停止・純読取・exact ledger・rollbackなしの独立assertを通過した。既存N03c実atomic失敗と解除対照も同じlib実行で成功している。

### 固定値・不変証拠・返却

| 対象 | SHA-256 |
| --- | --- |
| 最終store | 38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF |
| 最終commit | 24414D9610D0970DDE11B09C17F62BB92051BD066245F5F316FD0F466A79C6A6 |
| store全test suffix受領/最終 | 1FB68B058230B246889833D2D39B308812DD5E20DE83E2BA239AAA1124C9FE97 |
| commit全test suffix受領/最終（新Rを含む） | 7CDDD3A3381FDD3CB1604C6032651DDF12619C9EB0FC06524BB2744DAB290DBE |
| R module受領/最終 | B26C43DB7D68EE8E97EB55A75C14921E722F7E2F3B98E72FAE3EA492C03B1BDB |
| material integration受領/最終 | 5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683 |

[source確認](artifacts/history-material-recovery-implementation-source.log)、[実行前6EXE](artifacts/history-material-recovery-implementation-exes-before.log)、[最終source/test/6EXE/cleanup](artifacts/history-material-recovery-implementation-final.log)に完全path/hashを保存した。最終時刻は2026-10-08 14:08:08 +09:00である。既存4integrationも作者固定hash不変、6EXEの実行前後hash一致、targeted rustfmt/diff check exit 0、所有fixture prefix残存0である。追加削除は行わない。

独立runnerへの固定EXEはいずれも `F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/` 配下である。

| EXE | SHA-256 |
| --- | --- |
| wordweave5-b3388a7bbef44bb4.exe | 6F4E7C89873D51E3CDCC8E7EC92112A8CE8998F472C9715BC92024D7991B8C64 |
| audio_history_material-d89f83041366a9a0.exe | 58C3958F5EB04A46F27A5B324F9141B71781F93875F7FF81CCAB60133AE57364 |
| audio_history_save-3ab33269b9010ef4.exe | D23905230F70DF0FE434648A3CC38F71F3E37A79A1F8E07A7098F46DF7BD1449 |
| audio_history_progress-892b5d70a3c847c8.exe | EA5FCEC8A14F36289F460A30A8D68A53102227D5318784725605542E71AD80A3 |
| audio_history_migration-33b6a419850044f4.exe | 6A777674202D601D28AF7910474795AFF5B79732C55F5B7D9257629B2C066EB5 |
| audio_history_gate-0c0e7e180bf0149a.exe | DF06330C7966452FC452F36E3209C04B35EE031B1AA83B2748635D0B417BEA35 |

時刻取得不能・初回archive名衝突はseamで決定注入できず未測定のままである。native/電源断、実利用data、release全体確認は実施していない。runtime model/effort metadataとusage開始/終了counterは未取得、推計しない。

source/build所有は最終固定値確認後に親へ返却した。以後は本記録のみ追記し、独立runner/reviewerへ渡す。実装自己GREENは独立Postや利用者受入を意味しない。
