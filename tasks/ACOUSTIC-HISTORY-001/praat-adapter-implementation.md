# I5c-1 固定Praat公開adapter実装

2026-10-07 / ACOUSTIC-HISTORY-001。planner指定 implementer `gpt-6-astra` / `high`。実行model/effort metadataと開始・終了token countersは未取得である。AGENTS.md、development-team.md、KPI、wordweave-change、TDD、incremental-implementation、障害切分け時のdebugging-and-error-recoveryを適用した。受入済みspecとdesign-i5.md、独立praat-adapter-tests.mdを入力とした。

所有変更は `tools/acoustic-analysis/src/praat.rs` と本書、担当検証ログのみである。manifest、exports、protocol、script、独立testは変更していない。親は別所有でprivate `process::reap_unconfirmed()` を追加した。これは元のCancelled等を保ちながら、adapterが未回収work/leaseを保持するための照会である。再委任・commit/push・実録音・利用者データ・live AI/APIは使用していない。

## 実装境界

- 明示絶対path、非reparseのEXE/親dirを確認する。EXEはwrite/delete共有なしの読取handleをbackend寿命中保持する。親dirのdelete拒否handleは各呼出内だけ保持し、backend待機中にworkspaceのrenameを阻害しない。
- 固定96,780,624 bytes/SHA-256を64KiB streamingでconstructor時と各spawn前に検証する。現pathのvolume/file IDをpinと照合し、固定scriptの内蔵bytesと実fileのhashも検証する。version成功後にのみ固定identityを公開する。
- 全instance共通AtomicBool leaseを即時claimし、競合はResourceLimitとする。固定512MiB process/Job、stdout1MiB、stderr64KiBの既存supervisorを利用する。backend fallbackは追加していない。
- 直接Mono/profileを検証し、60ms未満はInvalidInput。checked work見積り64MiBと実書込byte cap、既存float64 WAV writerを使う。一呼出一専用tempdirのみ作成し、versionのcwdもpinする。原音や既存未知workには書かない。
- child/pipe終了後にscript/WAV/専用dirのpinを閉じて自分のdirだけ削除し、その後leaseを解放する。停止未確認ではworkとfile/dir pins、leaseを保持する。cleanup失敗はAtomicBool警告へ保存し、takeでswapする。取消し・期限・資源エラーをPartialへ緩和しない。
- 非Windowsでは無制約processを起動せずBackendUnavailableを返す。非Windowsでのcompile/runtimeは本工程で未検証である。

## 再現したrunner差と原因

最初のHidden launcherでは公開20件中8成功/12失敗（59.29秒）となり、全正常extractがBackendFailedになった。[初回runtime](artifacts/i5c-adapter-impl-run.log)、[stderr](artifacts/i5c-adapter-impl-run.stderr.log)。終了0・正常stdout・stderr116 bytes・cleanup成功へ局所化した。同一build/合成200Hz1件を直接起動するとstderr0で成功した。

[固定v7.0.02公式source](https://raw.githubusercontent.com/praat/praat/v7.0.02/sys/praat.cpp)では、Praatが親consoleへAttachConsoleを試み、GetConsoleWindowが非nullの場合にscript開始bannerをstderrへ書く。Hiddenはconsoleを隠すだけで、console無しではない。合成実行内で原文やpathを出さない一時boolean照合を行い、[公式banner prefix一致](artifacts/i5c-adapter-banner-one.stderr.log)、[Hidden親consoleあり](artifacts/i5c-adapter-hidden-console.stderr.log)、[通常cargo親consoleなし＋200Hz成功](artifacts/i5c-adapter-no-console.log)を確認した。公式起動処理でquiet/silent抑止optionは確認できなかった。

非空stderrを無視する変更、classifier緩和、固定script/pin変更、hidden起動廃止は行っていない。一時計測コードは除去した。親確認のGUI hostはwindows subsystemでconsole無しであり、この条件の公開20件を通常の要求cargo commandで検証する。console付きdebug/CLI hostは開始bannerによりBackendFailedとなる制約を残す。GUI本体からのI6実行をここで合格扱いしない。

## 検証

すべて固定済みEXEと合成波形/合成子だけを用いた。独立期待値を変更していない。

| 検査 | 結果 / 証拠 |
| --- | --- |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --test praat --locked --offline -- --test-threads=1 --nocapture` | 20成功、failed/ignored/filtered 0、63.40秒。[最終公開log](artifacts/i5c-adapter-final.log)。60秒48kHz最大fixtureはnew/extract同一deadlineで4651ms |
| 合成child build、`--features praat --lib --locked --offline --no-run` | 成功。[fixture build](artifacts/i5c-fixture-build.log)、[unit compile](artifacts/i5c-unit-compile.log) |
| compile出力の実在unit EXE `acoustic_analysis-8d1dd6b6ba45c163.exe --test-threads=1` | 53成功（process21＋fixture1＋protocol31）、failed/ignored/filtered 0、3.64秒。[最終unit log](artifacts/i5c-unit-final.log)。Hidden launcherに45秒所有限定watchdog、救済kill不要 |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked --offline -- --test-threads=1` | default既存59成功（core32＋report27）。feature付きunit/publicが0件なのはdefault境界であり実Praat成功数へ加算しない。[default log](artifacts/i5c-default-final.log) |
| 所有file rustfmt --check、git diff --check | 成功。一時計測eprintln/GetConsoleWindow/todoの残存0 |
| `cargo clippy --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline` | exit0。所有adapterの警告0。所有外dead_code 3件（peak_job_bytes/Busy/pid）と既存protocol chunks_exact_to_as_chunks 1件を未修正のまま明示。[clippy log](artifacts/i5c-clippy-final.log) |

0件filterは成功件数へ含めない。途中の誤exact filterで0件、先行runtime中の再linkでLNK1104が各1回発生したが、対象終了後に正しく再実行した。これらを製品の行動失敗や合格へ数えていない。通常sandboxは既知ACL初期化問題のため、承認レビュー付きscoped実行を用いた。OS管理者権限の追加やセキュリティ設定変更はない。

最終SHA-256:

- 本体praat.rs: `6BD9D84ADC85E2EBB2E0E546DDCED6AB086405829DBF1B90C3653DAB45C43787`
- 独立tests/praat.rs: `493DE1F13F4BD9F6BED64A202E11016F3A9538CCC7726773AF5297D1E5D50FF3`（REDから不変）
- 独立process_unit.rs: `9111C9D4F6D9E3E496D015F4C1C17695041E6C3CAEC5F6F24DD7FBDBD492EE43`、praat_child.rs: `8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228`（ともにREDから不変）

## 残るgate

I5c-2のprivate fault/複数instance lease競合、mid-hash/起動後取消し、EXE/script差替え・親dir競合、cleanup失敗warning保持/取得、未回収時のlease保持は独立試験待ちである。現実装の存在を検証成功の代わりにしない。実Praatのpeak commit、瞬間work/合計memory ledger、親crash、複数子孫合計cap、OS待機最悪値、I6共通deadline/二host、全app test/release build、自然音声精度も未完了である。

最終verificationはtest runner、受入は利用者に残る。本体praat.rsと本書の所有を解放する。
