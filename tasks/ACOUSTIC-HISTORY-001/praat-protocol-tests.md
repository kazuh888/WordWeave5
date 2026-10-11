# I5a 独立 protocol テスト / RED handoff

2026-10-04。担当は `ww-test-author`、計画者の指定は `gpt-6-astra` / `high`（[implementation.md](implementation.md) I5行）。実行モデル/effortの返却metadataと使用量の開始・終了カウンターは未取得であり、指定値から実行値を推測していない。

## 範囲と根拠

承認済み [spec.md](spec.md) のAH-AC01/05/06/09/10と、[design-i5.md](design-i5.md) §3/5/7のI5a契約から期待値を作成した。`agent-skills:test-driven-development` のRED工程のみである。所有したソースは [praat_unit.rs](../../tools/acoustic-analysis/tests/support/praat_unit.rs) だけであり、本体、manifest、lib接続、他テストは変更していない。開始時の既存dirty差分を保持した。

完了境界は独立テスト作成、コンパイル後のstub RED、実装者/runnerへの引継ぎである。修正、OS supervisor、固定Praat実行、全app gateは含まない。今回は新設private関数の未実装stubに対するREDであり、利用者が報告した実音声不具合の再現を主張しない。

## 対応とfixture

| AC / 契約 | 独立テスト群 | 期待値 / fixture |
| --- | --- | --- |
| AH-AC05/10、数値保存 | parser 12件 | 手書きWWPITCH、9桁小数をbit一致、時刻0/終端とHz50/800、CRLF/LF/終端改行任意、完全一致unvoiced |
| AH-AC01/05、protocol失敗 | 同上 | 0Hz、範囲逸脱、NaN/Inf、符号/指数/空白/非ASCII、逆転/重複/範囲外時刻、BOM/UTF-8/裸CR、header/count/field/余剰行をInvalidOutput。後半の不正でも部分trackを返さない |
| AH-AC01、frame/stdout資源境界 | 同上 | 6001行を独立生成し先頭0秒/末尾60秒を照合。6002宣言はResourceLimit。1MiB直前/一致の不正構文はInvalidOutput、1byte超過はResourceLimit |
| AH-AC05/10、WAVと保存 | WAV 11件 | 独立RIFF chunk walkerでfmt18bytes、tag3/mono/64bit、rate/byteRate/blockAlign、fact frame数、data little-endian、58bytes headerを検査。半LSB±1/65536、±1、signed zeroと元sample bit保存。正常fixtureは隔離tempdir内の実ファイル |
| AH-AC01/09、入力・資源境界 | 同上 | 8/48kHzの1sampleはwriter成功。空、rate外、非有限/範囲外PCM、60秒+1sampleはInvalidInputかつ出力なし。48kHz×60秒は23,040,058bytes、個別writeは64KiB以下 |
| AH-AC09、取消し/期限/再試行 | 同上 | 入口の取消し優先、期限切れ時出力なし。Write境界で取消し・絶対deadline到達を発生させ失敗と途中停止を確認。取消し後の新token/新出力による再試行を検査 |
| AH-AC01/06、I/O失敗 | 同上 | Write境界で0/12/62bytes後に失敗。BackendFailed、raw error sentinel非露出。独立出力への再試行成功 |
| AH-AC01/06/09、終了分類 | classifier 8件 | exit0/空stderr成功、parseとの責任分離、exit0を含む既知OOM、8192byte境界を跨ぐOOM、memory observed優先、低peak/通知なしの未知nonzero→ResourceLimit/UnclassifiedAbnormalExit、exit0の非空stderr→BackendFailed/BackendDiagnostic、raw sentinel非露出と再試行 |

fixtureはすべて合成PCM、手書きprotocol、memory内のProcessOutputである。利用者音声、教材、原音、API、外部processを使わない。I/O故障と取消し/期限は制御可能なWrite境界でのみ注入し、通常WAVは実ファイルと実Vecで検証する。正解側のparser/codecとして本体helperを流用しない。

## 実行証拠

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib praat_unit --locked --offline
```

[i5a-protocol-red.log](artifacts/i5a-protocol-red.log)：コンパイル成功、31件実行、0成功、31失敗、0ignore、0filtered、終了code101。全失敗は `src/praat_protocol.rs` の3か所の `todo!("I5 independent tests before implementation")` に到達したpanicである。fixture作成失敗やコンパイルエラーではない。constructor/OS未接続に伴うdead_code警告は残るが、RED判定と区別した。

所有テストへ `rustfmt --edition 2021` を実施。`git diff --check` 成功。新規untrackedのためdiffだけを内容検査済みの根拠にしていない。テストファイルはrustfmt/compileの双方で検査済みである。

## 実装者・runnerへの引継ぎ

実装対象は既存private seamの `parse_pitch` / `write_mono_wav` / `classify_output` である。本テスト31件を変更せず、上記コマンドでGREENを確認する。失敗が仕様曖昧さやfixture不良を示す場合は親へ具体的な差分と根拠を返し、期待値を本体に合わせて緩和しない。全体検証は本体変更後にrunnerが実行する。

次は未検証である。

- stub REDでは、全case内の後続assertやretry成功はまだ到達しない。GREEN時に期待値を実行して確認する必要がある。
- 1MiB丁度で有効なprotocolの受理は未検査であり、今回はサイズ境界と構文エラーの分類を分離した証拠である。
- bounded writeの検査は個別write長を対象とし、内部の最大allocation/全processのmemory使用量を証明しない。
- raw errorが戻り値に含まれないことは確認対象だが、OS層/host全体のlog・永続reportの非露出は未検証である。
- `peak_job_bytes` 単独のcap一致/超過は分類契約の明記がないため新たな期待値を定義していない。memory_limit_observed、既知OOM、未知nonzeroを契約どおり検査する。
- Windows Job、子孫停止、pipe上限、cleanup、固定EXE/hash、実Praat float64読取、自然音声精度、二host、全app test/buildは別gateである。

作成・RED工程を完了し、所有2ファイルを親へ解放する。本体修正や公開は行っていない。
