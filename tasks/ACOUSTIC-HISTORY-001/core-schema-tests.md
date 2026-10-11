# I4 strict schema RED引継ぎ

2026-10-04 / ACOUSTIC-HISTORY-001 / 独立テスト作成担当。

- 入力は承認引継ぎを記録した `implementation.md`、`design-i0.md` 3/4節。受入対応はAH-AC10/18（版・共通保存文書境界）、AH-AC05（値／欠測保持）である。
- planner指定はI4の `gpt-6.1-sol` / `high`。実行metadata、開始／終了の入力・キャッシュ・出力・推論カウンターは未取得であり、推定しない。親子包含関係も未取得である。
- `agent-skills:test-driven-development` のテスト／RED工程を適用。所有は `tools/acoustic-analysis/tests/schema.rs` と本書のみ。本体・manifest・既存32テスト・他者差分を変更していない。
- fixtureは手書きJSONの固定v1 profile、16kHz mono 1秒、PCM16値32相当のRMS100窓、全長低エネルギー区間、100/200HzのF0である。正常文書のdecodeとvalidateを負例前に確認する。`analyze` はoracleに使わない。すべてメモリ内隔離値で、未知fieldは無害sentinelのみである。

## コマンドと観測

`cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --test schema --locked --offline`

コンパイル成功、27件実行、8成功／19失敗／ignored 0（実行0.03秒）。通常execはWindows deny-read ACL適用で起動失敗したため、理由付きrequire_escalatedで同じofflineコマンドを実行した。

19件は `serde_json::from_slice` が不正文書を受理したことによる期待assertionのREDであり、fixture不備・compile errorではない。期待失敗は以下である。

- 未知field拒否15件: Report、独立Profile、Report内Profile、EnergyProfile、PitchProfile、EngineIdentity、Measures、PitchMeasure、RMS/Pitch TimedValue、TimeInterval、Quality、SampleValueのFinite/DigitalSilence/Missing。
- 必須Optionキー欠落拒否4件: `ac10_report_requires_pitch_engine_key`、`ac10_engine_requires_binary_hash_key`、`ac10_engine_requires_script_hash_key`、`ac10_quality_requires_fraction_key`。

成功8件は正常Report/Profile/SampleValueのroundtrip保持3件、上記Optionの明示null保持4件、非Optionの代表13キーに対する欠落／null拒否matrix1件である。schema_errorはdeserialize失敗として観測し、後段validateだけの拒否で代替しない。

修正後runnerは同じコマンドで27件成功を確認する。拒否期待値を緩和せず、旧core32件の検証も維持する。retry／取消しはこの純粋decode境界では非該当、正常文書の再読と値保持をroundtripで検証した。実API・実録音・公開は行っていない。テスト作成完了はRED確認と引継ぎまでであり、製品修正の完了ではない。
