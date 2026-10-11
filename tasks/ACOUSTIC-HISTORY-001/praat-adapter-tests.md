# I5c-1 pinned Praat公開API独立RED

2026-10-07 / ACOUSTIC-HISTORY-001。planner指定はtest author `gpt-6-astra` / `high`（implementation.md I5）。実行metadata、入力/cache/出力/推論の開始・終了カウンターは未取得である。AGENTS.md、development-team.md、KPI、TDD、wordweave-changeを読了し、テスト/RED工程だけを担当した。

入力は親から承認を引き継いだspec AH-AC01/05/09/10/18とdesign-i5.md、I5a/I5b実装報告である。所有変更は `tools/acoustic-analysis/tests/praat.rs` と本書、担当検証ログだけである。本体・manifest・lib・script・モデル/gate・他者の変更を編集していない。

## ケースと独立oracle

| AC | 公開APIの検査 |
| --- | --- |
| 01 | 相対path、不在、directory、wrong size、同サイズwrong hashをBackendUnavailableで拒否。明示MissingPitchBackend使用時のRust RMS/全長保持。検証済みEXEのwrite/delete拒否とDrop後handle解放 |
| 10 | 固定name/version/EXE hash/script hash identity。PitchProfile全11fieldの各変更とNaNをUnsupportedProfileで拒否 |
| 05 | 直接Monoの空、rate範囲外、60秒超、NaN/±Inf、振幅範囲外をInvalidInputで拒否。59ms直接拒否とcore TooShort、60/61ms有声を8/48kHzで検査 |
| 05/10 | 合成200Hzの2秒8/48kHzと60秒48kHz。有限F0は195–205Hz、有声frameあり、時刻有限/範囲内/厳密増加、1–6001frame。観測値をgoldenにしない |
| 05 | 2秒ゼロは全frame Unvoiced。0.49 PCM16 LSB振幅の合成200Hzは有声（PCM16へ丸めれば全sampleゼロとなることも独立検査）。core反相stereoはDigitalSilence/Unvoiced/Partial |
| 09 | new/extract入口の事前取消し・期限切れ、同時成立時Cancelled優先、次の正常試行成功。最大fixtureでnew/extractへ同一60秒deadlineを渡す |
| 09/18 | Unicode/空白work root、成功・入口拒否・取消し後の専用work残存0、未知の既存workと合成原音sentinel不変。入力slice bit/PCM不変、cleanup warningなし |

20 testはprocess-local mutexで直列化する。これは通常caseが製品の同時1件leaseへ競合しないためであり、並行leaseの証明ではない。test panic後のpoisonだけを回復し、他caseがREDまで到達できる。

## fixtureと実行証拠

実EXEは明示test-only既定値 `tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe` をrepository rootから組み立てる。任意の別位置を指定する場合は絶対pathの `WW_TEST_PRAAT_EXE` を使う。96,780,624 bytes / SHA-256 `22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4` 以外はfixture gateを失敗させる。PATH探索・取得・製品pin緩和はない。scriptは実fileのSHA-256 `3616a00415149e0ef5791f5ea497cbf523468695faa44fb059d911b6bca0876e` とLFを検査する。

一時データは各testのtempfile配下だけである。波形は算式で合成し、未知work/原音の保持対象も合成bytesである。実録音・利用者学習データ・network/API・Praatの追加取得は使用していない。

```powershell
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --test praat --locked --offline --no-run
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --test praat --locked --offline -- --test-threads=1
```

- compile成功: [compile log](artifacts/i5c-adapter-compile.log)。既存未接続本体のdead_code warning 31件で、compile errorは0件である。
- 最終RED: [runtime log](artifacts/i5c-adapter-red-final.log)。**1 passed / 19 failed / 0 ignored / 0 filtered、1.39秒、exit101**。唯一の成功は公式fixtureとscriptの独立pin検査である。
- 19件すべてが `src/praat.rs:23` のconstructor `todo!` panicで失敗した。fixture不在/hash相違・compile失敗ではない。これは未実装APIへ到達するREDであり、既存実装の特定不具合を再現したとの主張ではない。extract内部や実Praat実行まで未到達である。
- 最初のRED後、仕様の「有声frameあり」に合わせてoracleを明記し、半LSB丸めtieによる情報残存を避ける0.49 LSB fixtureへ確定した。そのテスト変更後にのみ再実行した。本体実装/観測値への期待値変更はない。
- 所有testのrustfmt --check、git diff --check成功。I5a/bの既存53/59 testや全app test/release buildは本RED工程で重複実行していない。
- 最終test SHA-256: `493DE1F13F4BD9F6BED64A202E11016F3A9538CCC7726773AF5297D1E5D50FF3`。

Windows＋praat featureのtargetである。feature無効/非Windowsでの0件を実Praatgate成功と扱わない。fixture欠落時は明示FAIL/UNVERIFIEDであり、ignore/skip成功へ変換しない。

## 別境界・所有解放

I5c-2として、全instance lease競合、mid-hash/起動後取消し、起動直前EXE/script差替え、親dir競合、cleanup失敗warningの保持とswap、取消し/失敗後lease回復、wait未確定時のlease保持が残る。公開APIでsleepのみの競合試験を作らない。親がprivate同期hook設計・support test所有・lib接続を別に固定する。提案はLeaseClaimed/ExecutableVerified/BeforeSpawn/BeforeCleanup境界の同期callbackで、cleanup caseには隔離work pathが必要である。今回追加していない。

実Praatのpeak commit≤512MiB・瞬間work≤64MiB・stdout cap・全子孫終了の実測、親異常終了、複数子孫の合計memory、自然音声品質、二host/I6共通deadline接続も未完了である。60秒fixtureのreturn/経過時間だけでこれらを証明しない。通常不正出力・OS故障は既存I5a/b所有とし、本testで模造EXEを製品constructorへ通していない。

本書とtests/praat.rsの所有を解放し、親を介してimplementer/runnerへ引き渡す。実装者は本体praat.rsを担当し、独立期待値は変更しない。最終GREEN・全gate・受入完了を本報告で宣言しない。commit/push/公開は行っていない。
