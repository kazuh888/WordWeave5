# I2a-0 Path reader: independent test / RED

2026-10-08 / ACOUSTIC-HISTORY-001。test authorのplanner指定は `gpt-6-astra` / `high`、実行metadataとusage countersは未取得である。AGENTS.md、development-team.md、test-driven-development SKILLを読了し、承認済みimplementation.md I2a-0およびdesign-i2.md §3/8の範囲でテスト作成工程を実施した。I1b-2b最終Post PASS後の親の解禁を受領した。

## 対象と所有

- HEAD: `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` と既存dirty。既存編集を保持した。
- 変更は `src/audio_history.rs` の `#[cfg(all(test, windows))] mod i2_path_adapter_tests` だけである。本報告と `artifacts/history-path-tests-*` は作者の証拠成果物である。
- module前の本体をLF/末尾改行1文字で照合したSHA-256は `3CFB86BCF790360B2EF9C3222839AD21BDEDED0565B7235BA7DDBAD452CD4969` で、受領したscaffold fileのSHAと一致する。既存new/with_limits/closureの本体は変更していない。
- `tests/audio_history_closure.rs` は変更していない。既存37/ignored1の期待値や実行条件も変更していない。本工程で既存suiteは再実行せず、GREEN後のrunner回帰へ引き継ぐ。

## 独立oracle / 受入対応

受入はimplementation.md I2a-0の3条件（同じclosure、公開署名・部分閲覧・上限/reparse維持、全bytes不変・Storage再取得/回復/書込なし）である。外部条件AH-AC04/16のreader接合部分に対応し、AH-AC08/09の保存再試行/controller受入やI8表示受入を代替しない。

既存closure fixtureの小さい部分を専用unit moduleに移した形式で用いる。root→trial→originalの3物件を手製DTO/合成bytesから通常fileへ書き、writer/analyzerを呼ばない。DTOのvalidate、物件の明示集合、SHA順序、合計bytes、trial本文、原音bytesをoracleとする。Storage/Path入口の単なる相互一致だけを成功根拠にしない。private入口への到達のためunit moduleが必要であり、公開APIや依存の追加はない。

全ケースのprefixは `audio_history::i2_path_adapter_tests::` である。

| case | 期待と範囲 | scaffold実測 |
| --- | --- | --- |
| handwritten_three_object_fixture_passes_existing_storage_entry | 手製3物件が既存Storage入口で正確に読めるfixture対照 | PASS |
| path_and_storage_entries_match_independent_closure_while_lock_is_held | new/with_limits/Path、正確なclosure・原音・snapshot/page、実Storage lock保持、全bytes保持 | 行動RED: Storage対照通過後for_rootがInvalidDocument |
| path_limits_accept_exact_closure_and_reject_one_less | nodes/bytes丁度成功、1不足LimitExceeded、許容される大bytes枠 | 行動RED: Storage境界対照通過後for_rootがInvalidDocument |
| invalid_path_limits_fail_without_creating_the_absent_root | nodes 0/100001、bytes 0をInvalidDocument、欠損directory未作成 | PASS（固定Err scaffoldでも通る対照） |
| missing_root_is_repeatable_diagnostic_failure_without_repair | strictと部分閲覧にMissingObject、2回反復、修復・bytes変更なし | 行動RED: for_rootがInvalidDocument |
| missing_original_preserves_trial_and_repeated_partial_view | strictはMissingObject、trial本文保持、original_available=false、診断と反復結果・bytes保持 | 行動RED: for_rootがInvalidDocument |
| path_only_read_never_creates_storage_lock_or_missing_directories | Storage未生成fixture読取成功、app.lock未作成、欠損root未作成 | 行動RED: for_rootがInvalidDocument |
| path_ancestor_junction_rejected_without_target_mutation | 通常pathの正例後、ancestor junction経由はUnsafePath、2回反復、全target bytes保持 | 非管理者sandbox外で行動RED: for_rootがInvalidDocument |
| path_root_junction_rejected_without_target_mutation | root junctionの同条件 | 非管理者sandbox外で行動RED: for_rootがInvalidDocument |
| path_history_junction_rejected_without_target_mutation | audio-history junctionの同条件 | 非管理者sandbox外で行動RED: for_rootがInvalidDocument |
| path_assets_junction_rejected_without_target_mutation | assets junctionの同条件 | 非管理者sandbox外で行動RED: for_rootがInvalidDocument |

有効入口scaffoldは常にInvalidDocumentであるため、非管理者sandbox外での4件再試験を含む9件の失敗は新API未実装に対する行動REDであり、既存製品の報告bugを再現したとの主張ではない。正常比較case内の既存Storage対照通過をログに出している。compile失敗やfixture不成立は行動REDに数えない。Path constructorより後の期待値はGREENで到達・確認する必要がある。取消しは同期read-only APIに該当契約がなく、この単位では非該当である。

## 実行証拠

```text
cargo test --locked --lib audio_history::i2_path_adapter_tests --no-run --message-format=json
target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history::i2_path_adapter_tests --list
target/debug/deps/wordweave5-b3388a7bbef44bb4.exe audio_history::i2_path_adapter_tests --nocapture --test-threads=1
rustfmt --check --edition 2021 --config skip_children=true src/audio_history.rs
git diff --check
```

- `--no-run`は1回のみ、exit 0、1m24s。compile成功。依存incremental cacheのhard-link失敗によるcopy fallback警告はあるが、テストのcompile失敗ではない。
- 固定EXE inventoryは11 tests / 0 benchmarks。実行はexit 101、2 PASS / 9 FAIL / 0 ignored / 134 filtered、0.15s。
- sandbox内の9 FAILの内訳は行動RED 5、fixture環境失敗4（ancestor/rootはjunction生成、history/assetsは準備renameのPermissionDenied(5)）。この時点では4件をREDに数えなかった。
- 親の明示許可後、同一EXEのjunction4件だけを非管理者sandbox外で各 `--exact --nocapture` 実行した。Administrator tokenでないことを事前検査し、管理者なら試験を中止する条件を置いた。4件とも通常Storage対照・junction生成・リンク経由の原音bytes確認・全bytes不変を通過し、for_rootのInvalidDocumentで行動RED。各exit 101、0 PASS / 1 FAIL / 0 ignored / 144 filtered、順に0.03s/0.01s/0.01s/0.01sである。
- 同一source/EXEに対する最終分類は対照PASS 2件・行動RED 9件。1つの全suite実行で2/9が得られたとの意味ではない。追加build・期待値変更・ignore追加はない。junctionの読取UnsafePath期待値自体はGREENで到達確認する。
- fmt check / diff check成功。
- 実行後、Windows一時directory中の `ww5-i2-path-*` 残存は0。panic経路でもfixture cleanupを確認した。

ログ: [build stderr](artifacts/history-path-tests-build.stderr.log)、[build JSON](artifacts/history-path-tests-build.jsonl)、[inventory](artifacts/history-path-tests-list.log)、[RED](artifacts/history-path-tests-red.log)。

非管理者sandbox外4件のraw logは `artifacts/history-path-tests-unsandboxed-path_{ancestor,root,history,assets}_junction_rejected_without_target_mutation.log`。前後hash/非管理者/cleanup確認は [scoped execution記録](artifacts/history-path-tests-scoped-execution.md) に保存した。

## 固定SHA-256

| 対象 | SHA-256 |
| --- | --- |
| src/audio_history.rs（本体scaffold＋test） | CC43B6D4D4781C540BCA649931DD73A15B8E7238035F7CD20791B4E23ED03FC2 |
| test module（cfg属性からEOF、UTF-8） | 6A2F382ED66A2F661C9D065269D4383F2340F82E3C02E096FE9D3EAA17D40437 |
| src/audio_history/closure.rs（未編集） | 44D8F94AE6D8D56C9743803D51F00DFEFADF11F577B8866ABFC001463192432D |
| tests/audio_history_closure.rs（未編集） | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 |
| fixed lib EXE | EC4B2ABF7E915205DD46B21BDA89D93D54CD9E4C1448173DF0F51901A8744783 |

## implementer / runner handoffと安全境界

作者から親へ本体fileの所有を返す。implementerはfor_root/root保持/closure2参照の本体だけを担当し、専用moduleの期待値を変更しない。正常Storage対照と型署名を維持する。runnerはGREEN後に `cargo test --locked --lib audio_history::` と `cargo test --locked --test audio_history_closure` を実施し、0件成功を受入にしない。

junctionは既存fixtureと同じWindows mount-point機構で、各targetを同じ隔離directory内へ限定し、リンクentryを非再帰unlink後にfixtureを削除する。bytes snapshotはreparseをたどらず、実targetの別entryで全bytesを捕捉する。unlink失敗時は再帰削除を中止して残存を報告する。親の明示許可で4件だけを非管理者sandbox外実行した。新しいfile symlink、管理者実行、OS設定変更はなく、既存leaf symlink特定1件の昇格承認を流用していない。

実学習データ、default root、認証、通信、録音、writer/recovery、公開、再帰委任には接触していない。全体test/releaseと実機受入は本工程の完了主張に含めない。
