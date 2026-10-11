# I4 独立coreテスト引継ぎ

2026-10-04 / ACOUSTIC-HISTORY-001 / test author。planner指定 `gpt-6.1-sol` / `high`、実行metadataと開始・終了使用量カウンターは未取得である。AGENTS.md、development-team.md、KPI、`agent-skills:test-driven-development` を読了し、テスト/REDだけを担当した。所有は `tools/acoustic-analysis/tests/core.rs` と本書である。manifest・本体・他者差分へ変更していない。

入力は承認済み spec/design 冒頭、implementation.md I4割当、design-i0.md 3節exact APIとdesign.md 7節である。60ms境界は親の明示handoffとprobe.mdの `3/floor=0.06` に従う。512MiB子process上限はcoreへ持ち込まず、64MiB/6001/60秒を固定している。

## 結果と再現コマンド

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --test core --locked --offline
```

最終実行は32件、0 passed / 32 failed / 0 ignored、cargo exit 101である。[core-red.log](artifacts/core-red.log) に保存した。全caseはコンパイル成立後に実行され、PcmView::new、AnalysisProfile::validate、AnalysisReport::validateの明示todo panicで停止した。初回30件の後、親指定59/60/61msの1件を追加し、Report入力metadataの契約追補後にさらに1件を加えて再実行した。単なるcompile error/0件実行をREDとしていない。新規capabilityの未実装REDであり、既存の報告バグを再現したという証拠ではない。手書きvalid reportの実際の受理もstubにより未確認であり、実装後にこの正例を先に確認する必要がある。

## AC・fixture・期待

| AC | テスト名prefix/主要case | 独立oracle・期待 |
| --- | --- | --- |
| AH-AC01 | `ac01_*`、`ac05_zero_*` | Missing backendはRMS/区間を残してPartial。Unavailable/Failed/InvalidOutputだけ欠測へ変換、InvalidInput/UnsupportedProfile/TimedOut/Cancelled/ResourceLimitはreportなし |
| AH-AC03 | `ac03_*` | 非空/ch割切り/8–48kHz/1–2ch/60秒一致と直後。borrowed PCM不変。RMS frame上限100一致/99超過、pitch101>100、buffer0拒否。6MiB原音制限をPCMへ適用しない |
| AH-AC03 | `ac03_saved_report_*` | sample_rate/channels/input_framesを保存しdurationとの不整合、rate/ch範囲、ゼロ/過大framesを拒否。rateがない旧案ではsample丸め時刻を正確に検証できないことを親へ返し、design-i0の明示追補後にfixtureを更新 |
| AH-AC05 | `ac05_*` | sample列を正本として `10*log10(sum(x²)/N)`、DC除去なし。mono arithmetic mean、25ms/10ms整数sample丸め・中心時刻・末尾sampleのみ。量子化正弦、22050Hz、低区間の和集合 |
| AH-AC05 | `ac05_exact_minus_forty_*` | 25kHz/625sample窓に8192を1個、他0でRMS=0.01の厳密−40dBFS。8191/8192は低値、8193は非低値。端末zero hopが200ms未満なら区間不採用 |
| AH-AC05 | `ac05_low_*` | 1599/1600/1601 frames @8kHz。200msちょうどを採用。silence/loud/silenceは[0,.28]/[.60,1]、合計.68秒 |
| AH-AC05 | pitch quantile/unvoiced/invalid cases | fake外部boundaryのみ。値800/50/300/100、無声1frameの中央値200、P10=65、P90=650、割合4/5。rangeは50/800を含む。時刻保持、重複/逆順/範囲外/NaN/Infinity/DigitalSilenceはInvalidOutput欠測 |
| AH-AC06 | `ac06_*` | stereo反相はDigitalSilenceとStereoMixed。原PCM端値2/16000がClipCandidate割合。1sample/59msはF0 TooShort、60/61msは取得可能。59msはfatal backendへ入らずlocal値を返す |
| AH-AC09 | `ac09_*` | 事前token取消し、backendが取消し後に成功してもreportなし。zero deadlineと実deadline経過後の成功をTimedOutにする。再試行は新tokenで成功、元PCM不変 |
| AH-AC10/18 | `ac10_*` | 全v1設定固定とprofile差替え拒否、同入力のReport一致、serde snake_case/tag、欠測保持、版保存。host別保存/秘密は本scope外 |
| AH-AC05/10 | `*saved_report*` | analyzeから作らず手書きの1秒/.5DC/100 RMS frames/100・200Hzを正例にする。serde decode後validateを要求し未知schema/profile、不正unit/range/time/hop/interval/ratio/quantileを拒否 |

全fixtureはcase内Vecと手書きDTOだけであり、実学習データ・録音・ファイル・資格情報・ネットワークを使わない。数値許容差は絶対1e-10、hop連結/min200ms判定の参照式ではbinary丸め誤差だけ1e-12を許す。期待を実装結果へ寄せていない。JSONで表現不能なNaN/Infinityはメモリ上validateを別に検査する。

## 実装・runnerへ

公開APIに不足はなく、`serde_json` dev-dependencyを統合担当が用意した。実装担当は本体だけを変更し、テストの調整が必要なら理由と仕様根拠を親へ返す。runnerは上記targetを同じlockで実行し、手書きvalid fixtureの成功と負例拒否を区別して報告する。compile成功はfixture/振る舞いの全妥当性を保証しない。

Praat実process、精度、OS child回収、license、二host永続化、外部送信、native UIは未検証であり、本32件でそれらのgateを代替しない。全体Cargo/release gateは親の後続runnerに残る。
