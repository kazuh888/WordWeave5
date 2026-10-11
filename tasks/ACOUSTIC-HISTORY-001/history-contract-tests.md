# I1a共有履歴契約の独立テスト／RED記録

2026-10-04 / ACOUSTIC-HISTORY-001 / I1a。担当は `ww-test-author`、planner指定は `gpt-6-astra` / `high` である。実行model/effortと利用量開始・終了カウンターは未取得である。

## 入力・承認・所有

親から実装許可と[実装契約4節](design-i0.md)の引継ぎを受領した。[実装計画](implementation.md)はAH-AC01–18の実装承認を記録している。今回の直接対象はAH-AC02/07/10–13/18の共有DTO境界であり、保存transaction・比較・UIの全受入を意味しない。

親が追加確定した境界は、offset ±1439分、UTC millis -62135596800000～253402300799999、非空Blob 12MiB／Root 8MiB、Trial revision／保存順1開始、空rootのみrevision0、receipt revision 1～root revision、terminal失敗のresultなし／failureとended必須、AdviceResultの評価別構造・文字数／件数制限である。これらを実装出力から推定していない。

所有変更は [contract.rs](../../tools/practice-history/tests/contract.rs) と本記録だけである。指定された実行ログは [history-contract-red.log](artifacts/history-contract-red.log) に保存した。本体、manifest、既存テスト、既存編集は変更していない。`agent-skills:test-driven-development` のRED担当として実施し、修正・GREEN実装は担当しない。

## fixtureと判定範囲

- 固定UUID、別々の64桁hash、RecordedAt、原音metadata、固定参照文、目標／振り返り、解析／助言run、root／receiptをメモリ内で手書き生成する。ID fixtureはserde経由で構築し、未実装parseが全ケースを巻き込まないようにした。
- 原音・参照のhashは構造テスト用の既知値である。実bytesからのhash照合、Blob内容の閉包検証、秘密allowlist、保存root隔離はhost adapter工程である。DTOに秘密フィールドを追加しても拒否されることは今回検証する。
- 実データ、filesystem fixture、実credentials、実API、Praatを使わない。AnalysisReportは作らず、I4の測定品質／Report.validateを今回の失敗原因へ混在させない。AnalysisRunのprofileは確定v1である。
- 無効入力は拒否し、切詰め・正規化・Noneからの値の捏造を認めない。Optionはキー必須／null許容であり、キー欠落をNone補完しない。確認競合は保存可能な事実として保持し、比較除外はI7で検証する。

## 35ケースの追跡

| 契約／AC | ケース群・期待 |
| --- | --- |
| 共通ID／AC02・18 | `canonical_uuid_*`、`uuid_parsers_*`、`sha256_*`、`deserialized_identifiers_*`。正規表記保持、不正表記拒否、decode後もvalidate必須 |
| serde／AC02・07・18 | `required_optional_keys_*` 2件、`unknown_and_secret_fields_*`、`required_nonoptional_keys_*`、`unknown_document_versions_*`。全documentと主要nested DTOで欠落／未知field／未知版拒否 |
| 固定入力／AC02・13 | `reference_is_*`、`goals_and_reflections_*`。Unicode scalar境界、空白参照拒否、固定値不変。`audio_accepts_*`／`audio_rejects_*` は6MiB・60秒・rate/ch境界と整数極値 |
| 本人・時刻／AC11・12 | `imported_unconfirmed_*`、`practice_claim_*`、`unknown_capture_values_*`、`recorded_at_*`。参考音声維持、確認欠落拒否、不明はNone、offset／UTC境界 |
| 確認出所・版／AC10・11 | `human_match_evidence_*`、`ai_evidence_*`、`conflicting_match_evidence_*`、`trial_revision_zero_*`。固定hash不一致と未所属run拒否、競合と旧結果参照を保持 |
| 処理状態／AC07・10 | `analysis_running_*`、`analysis_success_*`、`advice_sending_*`、`advice_three_response_*`、`terminal_failures_*`。result／ended／failureの状態整合、取消し・timeout・interruptedを成功化しない |
| 助言・安全／AC07・13・18 | `failure_codes_*`、`advice_missing_returned_*`、`advice_usage_*`、`contract_error_field_*`、`advice_assessment_*`、`advice_text_*`。固定code、未取得None、型付きエラーへのraw本文混入防止、3評価構造と上限 |
| 参照・順序／AC02・12・18 | `blob_and_root_*`、`root_sequences_*`、`receipts_*`。参照bytes上限、空root、保存順の正値／一意性／next境界、receipt確定revision |

## 実行とRED判定

```powershell
rustfmt --edition 2021 tools/practice-history/tests/contract.rs
cargo test --manifest-path tools/practice-history/Cargo.toml --test contract --locked --offline
```

2026-10-04実行結果は **compile成功、35件実行、5 passed／30 failed／0 ignored／0 filtered、exit 101** である。ログは標準出力・標準エラーを含む。

初回実行後、親がserde_jsonの `float_roundtrip` featureをmanifestへ追加したため同commandを再実行した。[変更後ログ](artifacts/history-contract-red-float-roundtrip.log) でもcompile成功、5 passed／30 failed／0 ignored／0 filtered、exit 101である。テスト期待値は変更していない。

成功5件は、Option必須キー2件、未知／秘密field拒否、非Option必須キー／schema欠落拒否、PracticeClaimの本人／日時確認欠落拒否である。これらはstub段階で既に満たされたserde契約の維持確認であり、修正前REDとは数えない。

失敗30件は `src/ids.rs` のparse/validateまたは `src/model.rs` の各DTO validateの `todo!` へ到達した。新機能が未実装であることのREDであり、既存の実不具合を再現したとの主張ではない。fixture生成失敗、compile error、実行0件、I4 Reportの失敗はない。実装後の特定assertionまで全て到達したことをこの段階で保証するものではない。

実行対象のSHA-256（未コミットscaffoldと新規テスト）:

| ファイル | SHA-256 |
| --- | --- |
| `src/ids.rs` | `60fbd331db1bfa4eeb592bd0b937017883f1fcc3c282d02ba0c58a0fea0d29f7` |
| `src/model.rs` | `f27aa2ba8c6d9f15cd91a0468d0fe487160986a0ec901a235dade12d8ae27a9a` |
| `src/lib.rs` | `f0281f6137e4ce1dfff1e811eeee013c400dfd480c7ea00e675b916e6bb9ef1f` |
| `tests/contract.rs` | `328879ac726d98323adda771608c29e44cd23f2f6a3122dc50275eb3ee8cef91` |

## 親経由の実装／実施担当への引継ぎ

上記commandをGREEN確認に用いる。特定ケースは末尾へケース名と `-- --exact` を加える。failure分類／上限／assertionを変更する場合は仕様根拠を親へ戻し、通過させるために期待値を弱めない。

AnalysisRun.validateがI4のAnalysisProfile.validateへ委譲した後、その未完了による失敗が出ればI1aの失敗と分離する。Report内容検証、hash実体照合、host別保存・原音保持、no-POST再試行、cohort選別、backups、native操作は後続担当の範囲である。

今回の完了境界は独立契約テストとRED引継ぎである。アプリ全体テスト・release buildは未実行であり、Windows提供gateを変更しない。`git diff --check` は成功。通常execのACL起動不具合に対して指定どおりrequire_escalatedを使用した。
