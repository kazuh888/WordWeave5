# I5-M2 独立テスト引継ぎ

2026-10-07 / ACOUSTIC-HISTORY-001 / test author。完了境界は独立テストsourceと期待値の固定、焦点コンパイル診断の引継ぎまでである。本体修正、GREEN確認、全体gate判定は含まない。

## 入力・所有・状態

計画者割当は [implementation.md](implementation.md) I5 の `gpt-6-astra/high`。親dispatchと一致する。実行metadata、開始/終了token counters、親子包含は未取得であり推定しない。AGENTS、development-team、KPI、`agent-skills:test-driven-development`全文を読み、同Skillのtest/RED工程を使用した。

承認入力は [spec.md](spec.md) AH-AC05/09/10、および親から独立PrePASSとして渡された [praat-m2-design.md](praat-m2-design.md) §4–10。設計SHA-256は `B84ACBC6E727898E760B4738F069D3AB033DAC6F06B7B81F09F64C90C4E8BFFB` と現物一致した。

所有は `tools/acoustic-analysis/tests/support/praat_unit.rs` の末尾 `#[cfg(windows)] mod m2` と本書のみ。既存protocolテスト31件のprefixはbefore snapshotと、末尾空白を除き完全一致する。本体、manifest、process、既存公開テスト、spec、モデル、gateは変更していない。既存dirtyは保持した。

テストsource SHA-256: `BD1D3ADF3524F11698538F29ED023D960D7B69CA3A7FABD326C6FCE885A899B1`。元source SHA-256: `FF7CDAC10481E33405B4BDBF22A61487063C7C4DBCBCA0BCD21CC86402250189`。

## 15件の独立期待値

接頭辞は `praat_unit::m2::`。全件通常testでありignoreはない。

| case / AC | test名（接頭辞省略） | 期待値 |
| --- | --- | --- |
| T1 / AC05/10 | `t1_same_length_in_place_overwrite_is_rejected_before_analysis_then_retry` | WriterClosedで同長無音へ上書き、独立readで内容相違を確認。解析入口0、BackendFailed、PitchTrackなし、cleanupと次call成功 |
| T2 / AC05/10 | `t2_same_length_replacement_is_rejected_before_analysis_then_retry` | WriterClosedで元fileをwork内別名へ退避し、手書き同長WAVで置換。同じ拒否・回復条件 |
| T2 boundary / AC05/10 | `t2_identical_byte_replacement_is_allowed` | fileを置換してもbytesが同一なら正常200Hzを返す。file identity一致を過剰要求しない |
| T3a / AC05/09 | `t3a_verified_pin_allows_read_but_denies_write_rename_and_delete` | Verifiedで独立Fileからread成功し、そのhandleを閉じてからwrite/rename/delete拒否。正常200Hz、全hook到達、cleanup |
| T3b / AC05/09 | `t3b_existing_writer_blocks_pin_and_is_released_before_cleanup_then_retry` | WriterClosedからPinAttemptedまでwrite handle保持、acquired=falseで解放。hash/verified/analysis0、BackendFailed、cleanup、次call成功 |
| T3b error / AC05/09 | `t3b_original_pin_error_survives_release_hook_error_then_retry` | 同じ競合を解放後hookがInvalidOutputを返しても元pinのBackendFailedを保持 |
| T4 / AC09 | `t4_cancel_at_writer_closed_cleans_and_retries` | 指定hook到達後cancel、Cancelled、analysis0、cleanup、次call成功 |
| T4 / AC09 | `t4_cancel_at_hash_chunk_cleans_and_retries` | 初回WAV chunk後cancel、chunk計1、Cancelled、analysis0、cleanup、次call成功 |
| T4 / AC09 | `t4_timeout_at_writer_closed_cleans_and_retries` | hookで同じ絶対deadline到達まで待機、TimedOut、analysis0、cleanup、次call成功 |
| T4 / AC09 | `t4_timeout_at_hash_chunk_cleans_and_retries` | 初回WAV chunk後同deadline到達、chunk計1、TimedOut、analysis0、cleanup、次call成功 |
| T4 priority / AC09 | `t4_cancel_precedes_timeout_at_hash_chunk` | 同期点でdeadline到達とcancel両方成立、Cancelled優先 |
| T5 / AC05/10 | `t5_short_writes_hash_only_accepted_prefix_and_finish_drops_inner` | 1/3/17/74/128 byte制限sinkで全74 bytes一致、固定SHA一致、finish帰還時inner drop済み、追加flushなし |
| T5 / AC05/10 | `t5_interrupted_write_is_retried_without_hashing_unaccepted_bytes` | Interruptedを1回注入、write_all再試行後bytes/SHA一致 |
| T5 / AC05/10 | `t5_zero_write_is_write_zero_without_adopting_a_digest` | zero writeはWriteZero、受理bytesなし、finish採用せずdrop |
| T5 / AC05/10 | `t5_flush_error_is_propagated_without_adopting_a_digest` | 書込み済みでもflush errorを伝播しfinish採用せずdrop |

`Seen`は各hookが同じWAV pathを受け取ること、成功時WriterClosed/PinAttempted/Verified/BeforeAnalysisRun各1回、pin取得成功、複数chunk到達を検査する。合成PCMは8kHz・2秒200Hz、WAVは128,058 bytesである。成功時chunk数>=2は64KiB上限による境界であり、正確なchunk分割数は固定しない。借用PCMの全sample bitsを毎call前後比較する。

## Fixture・oracle

毎caseは専用TempDirと日本語・空白を含むworkを使い、未知の既存workおよび原音相当のsynthetic sentinelを保存する。cleanupでowned WAV/dir消滅、未知work/sentinel保持、cleanup warningなしを確認する。失敗caseの次callは同じbackendと新token/deadlineで実Praatの200Hzを確認する。T4はcallとretryを合わせたmsを出力する。

実EXEは既存公開fixtureと同じ絶対path `tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe`（または明示 `WW_TEST_PRAAT_EXE`）を使う。サイズ96,780,624 bytesを検査し、実constructorによる固定SHA/版検証を通す。偽EXE、hash bypass、ネットワーク、実音声、実API、UACは使わない。constructor失敗はfixture不成立として停止し、WAV挙動REDへ数えない。

改竄WAVは手書きRIFF/IEEE-float64 headerと+0.0 payloadであり、本番writerをoracleにしない。T5のGOLDENは8kHz mono・2sample(+0.25,-0.5)の手書き74 bytes、独立.NET `SHA256.HashData`計算による期待digestは `5b2bf4be3b68fdeb2064b271d1fadc33ebdaa6ae4ee8b87e40be2eedb3d9a4e0` である。実装側hash関数を期待値計算に使わない。sinkは安全なI/O障害注入境界だけの代替であり、Windows共有試験は実Fileを使う。

期限試験はconstructor後に2秒の絶対deadlineを1回作り、指定同期点に期限前に到達したことをassertしてから同deadline到達までparkする。待機で改竄窓を推測しない。期限到達を確認後hookから戻す。hook未到達・期限前中断・0件実行をPASSにしない。

## 実施済み・次の実行

`rustfmt --check --edition 2021 tools/acoustic-analysis/tests/support/praat_unit.rs` は成功した。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --locked --lib praat_unit::m2:: --no-run
```

上記を一度実行しexit101。現時点は設計済み `DigestWriter`/`WavTestPoint` import未存在のE0432、および `extract_with_hook` 未存在のE0599である。ログは [m2-tests-compile-before.log](artifacts/m2-tests-compile-before.log)。**コンパイル失敗であり、行動RED未確認、実行成功0件である。** API未接合のまま同じcompileを反復しない。

実装者は§8の固定APIを接合する。callback enumにCopy/Clone/Eq/Debugは要求しない。最初は元snapshotに同期hookと未使用DigestWriter型を加えたbaselineを親が保存し、T1/T2を以下で実行する。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --locked --lib praat_unit::m2::t1_same_length_in_place_overwrite_is_rejected_before_analysis_then_retry -- --exact --nocapture
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --locked --lib praat_unit::m2::t2_same_length_replacement_is_rejected_before_analysis_then_retry -- --exact --nocapture
```

両caseの行動RED期待は「改竄fixtureの同長・実bytes相違を確認した後、BeforeAnalysisRun計数が1になり0とのassertが失敗する」である。このassertをchunk/Verified到達検査より前に置き、未接続hashのhook不足をバグ再現へ読み替えない。baselineでconstructorやfixture作成が失敗した場合は行動REDではない。

本体修正後の焦点確認は次である。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --locked --lib praat_unit::m2:: -- --nocapture
```

期待15 passed/0 failed。その後runnerが設計§10の既存公開20件・protocol/process・default回帰とM1明示別実行証拠の適用条件を確認する。T6の60秒上限・float64半LSB・既存原音保持は公開回帰を再利用し、新規重複fixtureを増やしていない。本体差分で発見された接合問題は親経由でテスト作者へ返し、期待値を緩和しない。

本書は独立Postレビュー、M2 Closed、本体修正成功、実API/実機受入の証拠ではない。
