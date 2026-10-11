# I1b-1 閉包readerの独立テスト記録

2026-10-04 / ACOUSTIC-HISTORY-001 / I1b-1。親から確定契約と実装許可を受領した。指定は `ww-test-author` / `gpt-6-astra` / `high`、実行metadata・利用量開始終了カウンターは未取得である。

## 所有・正本・fixture

正本は [design-i1b.md](design-i1b.md) である。Reference→Practiceの一方向遷移を許す追補を含む。TDDのテスト／RED工程を担当し、writer・reader本体・manifest・既存35件の共有契約テストは変更しない。

所有は [audio_history_closure.rs](../../tests/audio_history_closure.rs) と本記録だけである。実行ログは指定に従いtask artifactsへ保存する。中断前のテストを保持して再開し、作り直していない。

- ケースごとに固有名で新規作成した一時rootへ `Storage::at` を開き、履歴専用 `AssetStore::new(storage.dir.join("audio-history"))` のFileBlobへ手書きJSON／原音bytesをpublishする。終了時はStorage lock解放後、自身の一時rootのみ片付ける。
- 期待hashは既存 `assets::sha256` と固定bytesから作る。readerのhash helper、履歴writer、analyzeの出力をfixtureやoracleへ流用しない。captureのhash入力はdevice/distance/gain/environmentの固定順JSONであり、確認日時とversion自身を除く。
- 有効Reportは60ms未満の既知長、−6dBFS、F0 TooShort、低エネルギー区間なしを手書きで構成する。別ケースでDTO.validateを実行しfixtureの成立を確認する。真の音声形式復号・PCM hash再計算・F0精度は対象外である。
- 大きい総容量は生成しない。総bytes／node制限は小さな閉包で境界を検証する。個別cap超過は未公開object参照のmetadataで検査順を検証し、原音6MiBちょうどだけ実bytesを生成する。

## ケース追跡

| 正本の契約／対応AC | 代表ケース群と独立期待 |
| --- | --- |
| 固定内容／AC02・18 | `hash_helpers_*`、`original_reads_*`、`original_length_*`、`frozen_reference_goal_and_capture_*`。exact UTF-8／固定順hash、原音bytes保持、改ざん拒否 |
| 閉包／AC02・10・16 | `strict_closure_*`、`shared_original_*`、`same_hash_*`。root・旧revision・旧結果を含む手書き集合と総bytes、hash順／重複排除、同hash異size拒否 |
| 部分閲覧／AC02・08 | `missing_root_*`、`missing_original_*`、`one_broken_trial_*`、`damaged_previous_*`。空履歴との区別、他Trial／読める結果保持、局所診断とstrict失敗 |
| bounded read／AC14 | `node_and_total_*`、`invalid_limits_*`、`individual_caps_*`、`original_cap_*`、`bounded_previous_chain_*`。正確な境界、上限到達時に成功を捏造しない |
| strict schema／AC02・18 | `unknown_root_and_trial_schema_*`、`unknown_fields_*`。未知版・unknown fieldを捨てず拒否。Reportのnested unknown fieldも含む |
| 所属と結果／AC07・10・11 | `root_map_key_*`、`analysis_run_map_*`、`individually_valid_report_*`、`advice_run_input_*`、`advice_assessment_*`、`ai_match_evidence_*`、`individual_reads_*`、`reflection_must_*`。map ID／Trial／入力hash／状態／引用／出所の整合 |
| 過去保持／AC02・10・12 | `reference_to_confirmed_practice_*`、`previous_chain_rejects_*`、`previous_chain_requires_*`、`old_running_intent_*`。合法確認、逆遷移・過去日時／input変更拒否、intent→terminalの同RunIdを旧版と共存 |
| ページング／AC12・18 | `pagination_counts_*`。33head中32番目が破損してもcursorは32番目、次pageは33番目。存在しないcursorは診断付き失敗 |
| 無副作用／AC02・18 | `no_root_*`、`all_reader_operations_*`。全file bytes・directory集合と隣接host sentinelが不変、原音・結果・索引の自動生成なし |

エラー期待は親確認済みの区分に従う。未知fieldはInvalidDocument、未知schemaはUnsupportedVersion、関連ID/hash/state不整合はRelationshipMismatch、実体hash/長さ改ざんはDamagedObjectである。同hash異sizeは単一kindが契約で固定されていないためErrを要求する。

## 実行

```powershell
rustfmt --edition 2021 tests/audio_history_closure.rs
cargo test --test audio_history_closure --locked --offline
```

初回ログ: [history-reader-red.log](artifacts/history-reader-red.log)。compile成功、34件実行、1 passed／33 failedである。成功は手書きfixtureの妥当性確認である。失敗の内訳はreader/hashのstub到達31件と、Windowsのapp.lock保持中に全file bytesを読もうとしたfixture障害2件であり、後者をreaderのREDには数えない。

fixture障害はlock解放後の全file bytes取得→再lockしてreader操作→解放後の全file bytes比較へ修正した。対象fileやassertionを除外していない。未使用importも除去し、同commandを再実行した。再実行ログは [history-reader-red-fixture-fixed.log](artifacts/history-reader-red-fixture-fixed.log) である。

最終結果は **compile成功、34件実行、1 passed／33 failed／0 ignored／0 filtered、exit 101** である。成功は手書きfixture妥当性確認、33失敗は全てreader/hashの `todo!` による未実装REDである。fixture側panic・compile error・テスト側warningは各0件である。本体側の既存dead-code warning4件は変更していない。これは新機能未実装を確認したREDであり、既存の報告済み実不具合の再現とは扱わない。

実行対象SHA-256はテスト `309c8f92c1d49cff294e14b8ba1402d898451d50bfd04460c2bd0182936079ca`、reader scaffold `4d5a389cfc5cf4625d72e41f91557d250c894ebbb936a456403ddd7bd2c4d57a` である。compile error、fixture生成失敗、実行0件をreaderの行動REDに数えない。

## 引継ぎ境界

実装担当は上記focused commandを使用する。失敗を通すためにassertionを緩めず、仕様の曖昧さが見つかった場合は親へ戻す。個別caseは末尾へ名前と `-- --exact` を加える。

本テストで確認する無副作用はreader操作のfile bytes／directory集合である。真の二host機能適合、writer／intent recovery、Progress commit、backup復元、解析／送信の再実行禁止は後続工程であり、本テストの合格だけで全AC完了としない。実音声・実API・既定rootを使用していない。再委任・commit・pushは行っていない。

静的確認はrustfmt check、相対リンク欠落0、行末空白0、`git diff --check`成功である。今回の完了は独立テストとRED引継ぎまでであり、アプリ全体test／release buildは未実行である。
