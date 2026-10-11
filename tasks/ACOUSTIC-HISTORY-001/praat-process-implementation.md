# I5b Windows supervisor実装

2026-10-07 / ACOUSTIC-HISTORY-001。指定はww-implementer / gpt-6-astra / high。runtime metadataと使用量開始・終了カウンターは未取得である。[設計](design-i5.md)4/5/7/8節と[独立RED](praat-process-tests.md)を入力とし、wordweave-change、TDD、incremental-implementationを担当範囲で使用した。受入条件・資源上限・モデル指定は変更していない。

所有変更は `tools/acoustic-analysis/src/process.rs`、`process_windows.rs` と本書だけである。manifest/lib/test/protocol/adapter/固定scriptは変更していない。Windowsのmemory通知定数を使うため、親が既存windows依存へ `Win32_System_SystemServices` featureだけを追加した。

## 実装範囲

- 非WindowsはBackendUnavailableで終了する。Windowsは非継承Jobとcompletion portを作り、process/job memory capとkill-on-closeを設定・照合する。
- 絶対EXE pathとWindows argv quotingを使って直接、非表示・suspendedで起動する。STARTUPINFOEXのhandle-listはstdin NULとstdout/stderr write端だけである。Job所属の確認後にprimary threadをresumeする。
- 各pipeを1周回最大8KiB、交互に読み、累計byteとVec capacityをprivate limits以内に制限する。取消し・deadline・Job通知を各周回で観測する。ReadToEndやdetached readerを使わない。
- leader自発終了時もJob全体を停止し、leader waitとActiveProcesses=0、両pipe EOFを確認してから結果を返す。nonzero codeはclassifierへ保持する。memory通知を観測した場合はResourceLimit/MemoryLimitで終了する。
- 通常失敗とpanic/unwindでも所有guardがJob（assign前はchildだけ）を停止・waitする。read handleはそのguardが停止確認後まで保持する。停止確認失敗は成功へ変換せず、process内のpoison flagで後続起動を拒否する。raw OS errorやpipe bytesを失敗DTOへ含めない。

APIの根拠はローカルwindows 0.56.0の署名と、Microsoftの[handle-list契約](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)、[Job停止契約](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-terminatejobobject)で確認した。

## 実行結果

すべて合成fixture・専用tempdirだけを使用した。実音声・利用者データ・資格情報・live API・実Praat・commit/pushは対象外である。

| 検査 | 結果 / ログ |
| --- | --- |
| `cargo build --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --bin acoustic-test-child --locked --offline` | 成功。[ログ](artifacts/i5b-fixture-build.log) |
| suspended起動と4故障境界 | 独立1 test内4境界成功。[ログ](artifacts/i5b-suspended-direct.log) |
| 最初の監督試験 | 22成功（supervisor21 + fixture1）。[ログ](artifacts/i5b-first-runtime.log) |
| 最終 `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline --no-run` | compile成功。[ログ](artifacts/i5b-compile-final.log) |
| 上記が出力した `acoustic_analysis-8d1dd6b6ba45c163.exe --test-threads=1` | 53成功（supervisor21 + fixture1 + protocol31）、failed/ignored/filtered 0、3.53秒。[ログ](artifacts/i5b-final-runtime.log) |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline -- --test-threads=1` | 既存59成功（32+27）、unit0。[ログ](artifacts/i5b-default-regression.log) |
| 所有2fileのrustfmt --check、git diff --check | 成功 |
| `cargo clippy --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline` | exit0。未接続dead_code31件と所有外protocolのchunks_exact_to_as_chunks1件が残る。[ログ](artifacts/i5b-clippy-final.log) |

最終runtimeはcompile出力に実在した特定EXEをHiddenで起動し、別processから45秒で監視した。test側8秒watchdogの救済killは不要だった。初期のcompile込み45秒監視はcargo再compile中に到達し、所有cargo treeだけを終了した（[stderr](artifacts/i5b-suspended-stderr.log)）。これをsupervisorの行動失敗へ数えていない。以後compileとruntime監視を分離し、待機上限・期待値・capは緩めていない。

最終本体SHA-256:

- process.rs: `E3EFCDB098C9EC5949D40CE97B69F8319D96798619A0A099C602C39DFDE811B5`
- process_windows.rs: `FAC7022F2E56A8F3EBEB9EEC9AE9BDC332D73A1E02BDE204DF327906990B9FAC`

独立tests/support/process_unit.rsは `9111C9D4F6D9E3E496D015F4C1C17695041E6C3CAEC5F6F24DD7FBDBD492EE43`、praat_child.rsは `8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228` で、RED記録と一致する。期待値を変更していない。

## 残る検証と引渡し

I5cの固定EXE/hash確認・backend接続・全instance lease・work cleanup、親異常終了時kill-on-close、Job handle非継承の直接試験、複数子孫の合計memory使用、OS wait失敗、通知欠落、停止時間のOS最悪値、非Windows実行、実Praatと二host、全アプリgateは未検証である。停止不能時の無期限OS待機を解決済みと主張しない。停止確認のpoison flagはライブラリ内の後続child起動を止めるが、hostの残work/lease表示・回復はI5c/I6で扱う。

今回の完了は限定監督実装と実装者focused検証である。最終検証はtest runner、受入は利用者に残る。本体2fileの所有を解放する。
