# I5-M2 実装引継ぎ

2026-10-07 / ACOUSTIC-HISTORY-001 / I5-Q1。planner指定implementer=`gpt-6-astra/high`。実行model/effort metadata、開始・終了token countersは未取得である。AGENTS、development-team、KPI、wordweave-changeを読了・適用した。承認済みM2設計§4–10（SHA `B84ACBC6E727898E760B4738F069D3AB033DAC6F06B7B81F09F64C90C4E8BFFB`）、独立15件を入力とし、本体修正とfocused検証までを担当した。UIは非該当である。

所有変更は`tools/acoustic-analysis/src/praat.rs`、本書、`artifacts/m2-impl-*`のログ・baseline・差分のみ。他者のdirty差分、test/fixture、process、protocol、script、manifest、public APIを変更していない。実API・利用者学習データ・UAC・OS設定・実行物配布・再委任・commit/pushは行っていない。

## 実装

- WAV作成時の共有をFILE_SHARE_READへ限定し、DigestWriterで実際に受理されたwriteの先頭n bytesだけをSHA-256へ加える。short/Interrupted/zero/flush errorは元Write契約を維持する。
- `write_mono_wav`のflush成功後に`finish`で所有writer/Fileをdropし、そのstream digestを期待値にする。期待値をWAVの再読から作らない。
- read pinを取得し、期待長`58+8*N`とdigestを既存64KiB streaming hashで照合する。照合済みの同じFileをWork.pinsへ移し、子回収まで保持する。同一bytesの置換は受理する。
- WAV pin/検証由来のBackendUnavailableだけをBackendFailedへ変換する。取消し・期限・既存resource分類、EXE/script分類、cleanup/leaseと上限を維持する。追加読取は同じ絶対deadlineに含む。
- per-callのcfg(test)同期hookを同じextract経路へ接続した。WavPinAttemptedは実open結果直後、checkpoint・`?`より先に成功/失敗とも呼ぶ。競合writer解放後、cancel/deadlineを優先し、元pin Errはhook Errで上書きしない。
- WavHashChunkはWAVの実読取chunk後だけ、WavVerifiedはWorkへのpin移転後、BeforeAnalysisRunは既存self.run直前である。test wrapperにpin/hashを無効化する引数・分岐はない。testなし本番構成もcompile確認済みである。

書込scratchは既存writerの32KiB以下、追加読取scratchは既存hashの64KiBであり、WAV全量の追加Vecは作らない。今回の差分は175行追加/8行削除であり、[M2-only diff](artifacts/m2-impl-source.diff)を保存した。

## RED→GREEN

基準sourceは親の`artifacts/m2-before/praat.rs`（SHA `6BD9D84ADC85E2EBB2E0E546DDCED6AB086405829DBF1B90C3653DAB45C43787`）。これにhookと未接続DigestWriter APIだけを追加した[RED baseline](artifacts/m2-impl-red-praat.rs)を保存した。baselineはWAV digest検証を持たず、最終productionで照合を切るtest bypassはない。

すべてのcargoコマンドは`--manifest-path tools/acoustic-analysis/Cargo.toml --features praat --locked --offline`を指定した。

| commandの残り / 条件 | 結果・証拠 |
| --- | --- |
| `test --lib praat_unit::m2::t1_same_length_in_place_overwrite_is_rejected_before_analysis_then_retry -- --exact --nocapture`、通常sandbox | compile15.21秒成功。0 passed / 1 failed / 68 filtered、5.01秒。同長・実bytes相違確認後、BeforeAnalysisRun=1対期待0で行動RED。[log](artifacts/m2-impl-red-t1.log) |
| `test --lib praat_unit::m2::t2_same_length_replacement_is_rejected_before_analysis_then_retry -- --exact --nocapture`、通常sandbox | compile15.70秒成功。0 passed / 1 failed、2.74秒。rename error5でfixture不成立、行動REDではない。[log](artifacts/m2-impl-red-t2.log) |
| 同T2、承認済み非管理者sandbox外・consoleなし通常cargo | compile11.44秒成功。0 passed / 1 failed / 68 filtered、4.10秒。同長・実bytes相違確認後、BeforeAnalysisRun=1対期待0で行動RED。[log](artifacts/m2-impl-red-t2-unrestricted.log) |
| `test --lib praat_unit::m2:: -- --nocapture`、最終source、非管理者sandbox外・consoleなし通常cargo | compile9.39秒成功。15 passed / 0 failed / 0 ignored / 54 filtered、50.96秒。[GREEN log](artifacts/m2-impl-green.log) |
| `check --lib`、最終source、通常sandbox | test hookなしの本番構成成功、9.66秒。[log](artifacts/m2-impl-production-check.log) |
| `rustfmt --edition 2021 --check tools/acoustic-analysis/src/praat.rs`、`git diff --check` | 成功。[空log](artifacts/m2-impl-static-check.log) |

GREENでは同長改竄2種をBackendFailed・解析入口0で拒否し、同一bytes置換の正常200Hz、保持pinのread許可/write・rename・delete拒否、競合writerの解放と元error保持、取消し/同deadline超過後cleanup・次call成功、独立短write sinkを確認した。54 filteredは既存53件とM1別実行1件であり、今回成功数へ加算しない。

一部sandbox compileのincremental hard-link代替copy警告、既存processのdead_code警告を残した。本番checkは既存peak_job_bytes/Busy/pidの3件であり、所有外を変更しなかった。

## 固定対象

| 対象 | SHA-256 |
| --- | --- |
| praat.rs 最終本体 | `4BBCB967106126A7DAB101EF99E5128C38BA60081A65E102324248404E97EC53` |
| hook-only RED baseline | `C538AFA6F3C8DD0CFA5F7A0AE3DB36085FDCA514C63B8CB25D387B531488D5C3` |
| 独立praat_unit.rs（引継ぎから不変） | `BD1D3ADF3524F11698538F29ED023D960D7B69CA3A7FABD326C6FCE885A899B1` |
| target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe | `DB9BB36BF53C49BD1287DBB2C44B704502EBFC56E62A95302194A247321E2AD6` |
| M2-only diff | `94DEA3842F7E886B040609971C32EF8C0636869AF4F6574C825069D069D93B3A` |

M1非変更の現物hashはprocess.rs=`9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54`、process_windows.rs=`10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C`、process_unit.rs=`CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896`、praat_child.rs=`8EDDB923E88E51258B16267DB60B3967943FABC7BB4E786C95D15C84E8C8A228`である。M1を今回再実行したとは扱わない。

## 残余・所有返却

実装者のfocused 15件と本番compileは成功である。独立runnerによる設計§10の全回帰、公開20件の最大60秒・float64・日本語path/T6、変更後資源/時間実測、M1既存証拠再利用の照合、独立Post-review、利用者受入は残る。同期OS readの厳密な停止時間、I5c-2等の既存未検証を解消したとは扱わない。新しい設計ギャップは検出していない。Q1 Closedは親・独立レビューの判定を待つ。本体所有を親へ返す。
