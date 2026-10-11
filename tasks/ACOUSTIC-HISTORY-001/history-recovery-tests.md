# I1b-2b 独立回復テスト

2026-10-08 / ACOUSTIC-HISTORY-001 / test author。入力は承認済み `spec.md` AH-AC04/08/14、`design-i1b.md` のwriter/recovery契約と最終I1b-2b節、`writer-design-review.md`、`implementation.md` I1である。計画指定は `gpt-6-astra/high`、実行metadata・開始/終了使用量counterは未取得である。親から割当と承認状態を受領した。

`agent-skills:test-driven-development`を読み、担当をテスト/RED工程に限定した。本体修正・仕様変更・期待値緩和・再委任・公開は行わない。作業開始HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。既存dirty変更を保持する。

所有は `tests/audio_history_recovery.rs`、本記録、`artifacts/history-recovery-tests-*` のみである。既存writer37件・reader・共有DTO・本体は変更しない。終了境界は独立fixture/期待値を固定し、コンパイルと行動REDの結果・実装者/runner向けコマンドを親へ引き継ぐことである。

## 独立oracleと隔離

`ww5-recovery-tests-<pid>-<time>-<counter>` 配下へ `Storage::at` で隔離rootを作る。合成46-byte PCM16 WAV、固定日本語参照文、手製Trial/Run/結果/HistoryRootと正確なintent JSONを直接配置する。writerのprepare/計画結果をfixture oracleにしない。`Plan`は仕様の依存順・revision/sequence/previous/receiptから期待bytesを独立に組み立てる。正常fixture専用テストはcreate、reflection更新、Sending→Assessedの全閉包をreaderで検査する。

再起動ケースはwriterとStorage lockを解放して同じ隔離hostを再openする。全既存object/intentのbytesを比較し、明示した当該intent置換/finish削除だけを差分許容する。headと固定I/O failpointだけをdoubleとし、保存・読取・hash・閉包検査は実ローカル成分を用いる。実学習データ、実録音、外部通信、認証情報は使用しない。

## 受入対応

32 test functions（ループ副ケースを含む）である。

| 受入 | test名の主要部分 | 固定期待値 |
| --- | --- | --- |
| fixture | `handwritten_create_and_update_oracles` | create/reflection/adviceの手製closureがwriter抜きで成立 |
| W02 / AC04 | `missing_original`、`original_only` | 原音不足はOriginalMissing、同hash/len caller bytes必須。原音だけ残存ならAssociationReadyで同TrialId/固定入力を復元 |
| W02 / AC04 | `published_root_is_not_commit` | PublishedUncommitted、confirmはBusy、同object再利用、正本変更なし |
| W02 / AC04 | `real_prepare_publish_faults` | Before/AfterIntent、各Before/AfterObject、Before/AfterClosureの10点で実書込の残存状態と再起動分類を照合 |
| W02 / AC04/08 | `drop_after_resume` | 呼出間の取消し/dropでintent/原音保持、正本は未確定 |
| W03 / AC08 | `missing_result`、`existing_result_blob` | PayloadMissingで結果捏造なし、同結果bytes/既存blobから保存だけ再試行、旧Sending revision保持 |
| W03 / AC04/08 | `exact_mutation_whitespace` | whitespaceを含むexact mutation bytes/hash保持、意味同値だが異なるrequestはConflict |
| W03 / AC04 | `committed_receipt_and_later_root` | 最新・後続root receiptでAlreadyCommitted、revision/sequenceを増やさず対象intentだけcleanup、cleanup再試行可能 |
| W03 / AC04/08 | `finish_faults`、`finish_rechecks_head` | finish前後Ioでもreceipt保持、再起動再試行、最新head/request再照合 |
| W03 / AC04 | `confirm_rejects_wrong_receipt`、`public_tokens` | request/affected/operation/revision/current/nextの偽造、無関係headを拒否、既存bytes保持 |
| W03 / AC04 | `finish_accepts_later_receipt`、`require_readable_complete_latest_closure` | confirm後に進んだ正しい後続rootは許可、Io/閉包欠損は拒否 |
| W06 / AC04/14 | `create_rebases`、`create_existing_id` | 新IDは他Trial変更にrebase、receiptなし同IDは入力一致でもConflict。旧公開object保持 |
| W06 / AC04/14 | `update_rebases_only`、`rebase_validates_old_base` | expected target一致時だけrebase、旧base閉包を検査してから置換 |
| W05/W06 / AC14 | `rebase_quota_counts` | 独立bytes総和、公開済objectの二重加算なし、exact intent差分、上限一致許可/1 byte不足拒否 |
| W06 / AC04/14 | `resume_obeys_fresh_head` | read request後のpending/読み取り失敗を再検査し無書込で拒否 |
| W08 / AC04/08 | `empty_recovery` | 空一覧・不足request読取でdirectoryを作らない |
| W08 / AC04/08 | `forged_old_plan` | 構造と閉包が妥当でもmutationと違う計画は再計算でDamaged、resume禁止 |
| W08 / AC04/08 | `invalid_intents` | 未知版/field、operation不一致、必須base欠落、request不一致、順序/余分ref/len改ざん、未知mutation field、512KiB超過を拒否 |
| W08 / AC04/08 | `invalid_filename`、`corrupt_published_object`、`intent_path_directory` | 不正名/JSON/改ざんobject/unsafe pathを診断し、正常候補を表示、旧bytesを変更しない |
| W08 / AC04/08 | `pagination`、`exactly_32` | OperationId順・32件上限、壊れたintentでもcursor前進、32件ちょうどなら次pageなし |
| W08 / AC04/08 | `unreadable_or_damaged_current_head`、`head_change_during_recovery` | headを読めない/壊れた/snapshot中に変化したとき診断し回復可能と誤分類しない |

## 実行と引継ぎ

初期調査時はRecoveryView/Candidate/Stateと5公開メソッドが未提供であった。親が実装担当へscaffoldを割り当て、固定Io・無副作用の入口と型を追加した後に本targetをコンパイルする。API不足によるcompile errorを行動REDと数えない。

```text
cargo test --locked --offline --test audio_history_recovery --no-run
cargo test --locked --offline --test audio_history_recovery handwritten_create_and_update_oracles_are_valid_without_writer -- --exact
cargo test --locked --offline --test audio_history_recovery
rustfmt --edition 2021 --check tests/audio_history_recovery.rs
git diff --check -- tests/audio_history_recovery.rs tasks/ACOUSTIC-HISTORY-001/history-recovery-tests.md
```

初回no-runは成功（2m11s）、API/型不足0件である。最終32件の版は正常fixture単体のcargo試験でもコンパイル成功（1m58s）し、1 PASS / 0 FAIL / 0 ignored / 31 filtered、0.07sであった。logは [compile](artifacts/history-recovery-tests-compile.log)、[fixture](artifacts/history-recovery-tests-fixture.log) に保存した。

最終版の同じビルド済みEXEをhash固定し、非管理者sandbox外で全件実行した。結果は **32件、3 PASS / 29 FAIL / 0 ignored / 0 filtered、exit101、0.17s** である。[RED log](artifacts/history-recovery-tests-red.log) に保存した。全29失敗はscaffoldの固定Ioまたは空候補に対する仕様assert不一致であり、compile error・fixture破損・OS権限失敗ではない。PASSの内訳は正常fixtureと、旧base閉包欠損/現在head不読の拒否負例である。後者2件は固定Ioでも通るため、回復処理の実装済み・受入成功の証拠とはしない。

```powershell
& target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe
```

親の既存承認に従い、Windows sandboxによる合成asset保存制約を除くため当該EXEだけを非管理者で通常環境実行した。UAC/OS設定の変更はない。繰返しcargo実行では依存crateの再buildとincremental hardlink失敗/copy fallbackが発生したため、ソース変更のない全件REDは同一EXEを使用した。検証対象や基準は変更していない。

scaffoldの固定Ioに対するREDは新機能未実装の証拠であり、既報製品不具合の再現という主張ではない。loop内は最初のREDで停止するため、全副ケースを最後まで実行したとは扱わず、GREEN後のrunnerが32件の全副ケース到達を確認する。

固定対象のSHA-256:

| 対象 | SHA-256 |
| --- | --- |
| 新test | `120C9F31BF66ECD35A9C4E0B2D4FA42907F2D6E5093F80418A519D9CCC08909F` |
| test EXE | `3BD920443AC0F2BC75193ECEEB655CB2DEFAFD49AEFB0039C5D60E315402EF5A` |
| 既存writer37件（未編集） | `7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4` |
| RED時 store.rs | `902BE62827818CB47C789DAE283997E2CBD4CE5E2398273E098048B24C4FA554` |
| RED時 recovery.rs | `2B4CD4322FA3148290A8195145E78DBD97EAE59DC4BAFF75A455219E77CFFD9E` |
| RED時 mutation.rs | `8F55A105C3F7A2BD336F215A6E0233772F13DCDEA89C79FC3A5FEE81AD52B4ED` |
| RED時 audio_history.rs | `742C76361749F312CDB2FEE98538685BC3E1CC449EED9E145369B4E1142B5D5C` |

`rustfmt --edition 2021 --check tests/audio_history_recovery.rs` と担当ファイルの `git diff --check` は成功した。git未追跡ファイルはdiffの検査対象に現れないため、新testはrustfmt、担当文書は別途行末空白/リンク検査で確認する。

GREEN後のrunnerは本target、既存 `audio_history_store` / `audio_history_closure` を実行する。全体Windows delivery gateはI10であり、このテスト作成工程のfocused REDで代替しない。I2 pointer前後・I6解析/POSTの実呼出回数spy、実disk不足/電源断、100,000 entry走査境界、Windows junction/leaf symlink専用検査は本targetだけで追加受入した扱いにしない。今回のunsafe path負例はintent名のdirectory偽装であり、reparse全般の成功証拠ではない。

KPIの開始時刻/開始counterは未取得。作業中に最初に取得したローカル時刻は2026-10-08 06:59:27 +09:00、RED後取得時刻は07:09:48 +09:00であり、作業全体の所要時間として差分を推定しない。子IDは `/root/history_recovery_tests`、runtime model/effort・親子使用量の包含関係・終了counterも未取得である。親には32件固定・29件REDとhashを引き渡した。以降の本体実装は別担当であり、本テストの期待値を実装へ合わせて変更しない。

## 2026-10-08 R1/R2 境界補完

子 `/root/history_recovery_boundary_tests`、planner指定 `gpt-6-astra/high`。承認済み [境界packet](history-recovery-boundary-packet.md) のSHA-256 `87D13C34F7E76C2C6EB66864BE6FF1EA0E24A41A35A9E683DB5276270C9B3E82` と独立Pre PASSを受領して着手した。所有は本test末尾の2件、本記録追補、`artifacts/history-recovery-boundary-*` だけである。元32件を切り出したSHA-256は `120C9F31BF66ECD35A9C4E0B2D4FA42907F2D6E5093F80418A519D9CCC08909F` と一致し、旧期待値は不変である。総数は34 test functionsとなった。

| 指摘 / 受入 | 追加test | 独立oracle・保存条件 |
| --- | --- | --- |
| R1 / W05・W06 / AH-AC04・14 | `w05_resume_rebase_serialized_intent_exact_cap_includes_escapes_and_preserves_on_overflow` | referenceにquote/backslash/newlineを含め、exact mutationの合法末尾whitespaceで手製rebase intentを524,287 / 524,288 / 524,289 bytesへ調整。writer呼出前に長さと意味同一をassert。旧intentは全ケース上限内。前2件はReady、置換全bytes、独立root/Trial/receipt、旧objects不変を照合。超過は2回ともLimitExceeded、旧intentと全tree bytes不変。headは全ケース未変更 |
| R2 / W05・W08 / AH-AC04・08・14 | `w08_recover_scans_100000_entries_but_diagnoses_100001_without_writing` | 実OperationId名100,000件、先頭/末尾32件ずつは手製正常intent、中央99,936件は未選択の `{}`。先頭/末尾pageは32件・順序・state・request・原音不足・cursor・diagnostic空をassert。100,001件はLimitExceeded diagnostic必須。成功/超過各phaseの全tree bytesとasset sentinelを比較。未選択本文の妥当性を検証したという主張ではない |

fixtureは既存専用temp方式を用い、各試験でdrop後のroot不存在もassertする。実データ/API/UAC/OS設定変更はない。新規seamやproduction変更はない。今回の指摘は既知製品バグではなくcoverage不足であり、追加2件は既存本体で初回GREENとなった。行動RED・バグ再現を主張せず、assertを緩める変更もない。

| 自己検証 | 結果・実測 | ログ |
| --- | --- | --- |
| `cargo test --locked --offline --test audio_history_recovery --no-run` | exit 0、cargo 2m03s、監視込み125.28s。15分上限未到達 | [stdout](artifacts/history-recovery-boundary-compile.stdout.log) / [stderr](artifacts/history-recovery-boundary-compile.stderr.log) |
| 固定EXE R1 `--exact --nocapture` | 1 PASS / 0 FAIL / 0 ignored / 33 filtered、exit 0、0.94s（監視込み1.29s） | [stdout](artifacts/history-recovery-boundary-r1.stdout.log) / [stderr](artifacts/history-recovery-boundary-r1.stderr.log) |
| 固定EXE R2 `--exact --nocapture` | 1 PASS / 0 FAIL / 0 ignored / 33 filtered、exit 0、113.67s（監視込み113.81s）。20分上限未到達 | [stdout](artifacts/history-recovery-boundary-r2.stdout.log) / [stderr](artifacts/history-recovery-boundary-r2.stderr.log) |

R2は100k作成32.51s、先頭page全bytes比較完了53.32s、末尾page63.03s、超過診断と全bytes比較80.92s、cleanup完了113.08sであった。これは今回の環境の実測であり性能合格基準ではない。R1の3rootとR2の1rootは全てcleanup assert成功、残存fixtureなしである。EXE実行は親の承認範囲に従い固定test EXEのみ非管理者sandbox外で行った。cargoの依存再build/hardlink-copy fallbackは発生したが、旧writer/reader test EXEはhash不変である。`rustfmt --edition 2021 --check tests/audio_history_recovery.rs` はexit 0。

固定hash:

| 対象 | SHA-256 |
| --- | --- |
| `tests/audio_history_recovery.rs` | `AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567` |
| `audio_history_recovery-b2b70d25a5608ed7.exe` | `B53CFE52631DA229C015498916F168BFDABC1008EE2CAD4492B2FB17363A208F` |

本体4本は `history-recovery-implementation.md` stage2記録のhashと全て一致（store `ACD8C993...`、recovery `7D90C1FB...`、mutation `7D65497A...`、export `742C7636...`）。writer/reader sourceと既存EXEも同記録hashから不変である。

runnerへのコマンドは `target/debug/deps/audio_history_recovery-b2b70d25a5608ed7.exe --nocapture` で新34件を実行する。個別再現は同EXEに上表のtest名と `--exact --nocapture` を渡す。作者は新2件のみを自己実行し、旧32回帰を含む34件全体・既存writer/reader・最終受入は独立runner/親へ引き継ぐ。source/build所有を親へ返却する。

今回の編集着手前に取得した時刻は2026-10-08 07:24:43 +09:00。実行model/effort metadata、開始/終了token counter、親子の包含関係は未取得であり推定しない。
