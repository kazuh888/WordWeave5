# I5a protocol実装

2026-10-07 / ACOUSTIC-HISTORY-001。指定はww-implementer / gpt-6-astra / high。実行metadataと使用量開始・終了カウンターは未取得である。[設計](design-i5.md)3/5/7節と[独立RED](praat-protocol-tests.md)を入力とし、wordweave-change、TDD、incremental-implementationを担当範囲で適用した。

所有は `tools/acoustic-analysis/src/praat_protocol.rs` と本書である。2026-10-04にparser、float64 WAV writer、終了分類を順に実装してprotocol31件を成功させた。その後、承認レビューの利用量制限でログ集計と本書作成が実行されず中断した。2026-10-07の利用者継続指示後に検証を再実行した。保存済み本体を変更する必要はなかった。manifest/lib/process/固定script/独立テストは変更していない。

- parserはUTF-8、LF/CRLF、exact header/count、ASCII十進数、有限値、時刻の範囲/厳密増加、Hz範囲を検証する。符号・指数・空白・部分結果を受理せず、stdout 1MiBと6001frameを上限とする。
- WAVは全入力の検査後に58byte headerを出力し、32KiB bufferでf64 little-endianを書き込む。半LSBとsigned zeroを保持し、PCMを変更しない。partial write/Interruptedを含めI/O境界で同一deadlineと取消しを検査し、raw I/O errorを返さない。
- classifierはbounded stderr全体の固定OOM署名またはmemory通知をResourceLimit/MemoryLimit、未知nonzeroをResourceLimit/UnclassifiedAbnormalExit、exit0の非空stderrをBackendFailed/BackendDiagnosticとする。低いpeakや通知欠落から資源内完了を推定しない。

## 2026-10-07の検証

合成fixtureのみを使用し、実録音・利用者データ・live API・外部processは使用していない。

| コマンド | 結果 |
| --- | --- |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib praat_unit --locked --offline -- --test-threads=1` | 31 passed / 0 failed / 0 ignored / 0 filtered。[ログ](artifacts/i5a-protocol-green-20261007.log) |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline -- --test-threads=1` | 既存59件（32+27）成功、unit 0件。[ログ](artifacts/i5a-default-regression-20261007.log) |
| `rustfmt --edition 2021 --check tools/acoustic-analysis/src/praat_protocol.rs` | 成功 |
| `git diff --check` | 成功 |

本体SHA-256: `EF354A74EA2498D8BEC36F68A826BF031282000357EDD4094A9D80ED3AC8AB48`。
独立テスト `praat_unit.rs` のSHA-256: `FF7CDAC10481E33405B4BDBF22A61487063C7C4DBCBCA0BCD21CC86402250189`。2026-10-04に取得した値と一致し、期待値を変更していない。

未接続process側のdead_code警告は7件である（ProcessSpec、ProcessLimits、peak_job_bytes、未構築variants、supervise、FaultPoint、supervise_with_hook）。不要なallowで抑止していない。

残るgateはOS監督、子孫process停止、固定EXE、実Praat float64読取り、資源使用実測、cleanup、二host、全アプリ検証である。今回の成功はこれらの証拠ではない。最終検証はtest runner、受入は利用者に残る。本体単一fileの所有を解放する。commit/pushは行っていない。
