# I5-M1 親強制終了の独立回帰テスト

2026-10-07 / ACOUSTIC-HISTORY-001。test authorは既存 `ww-test-author / gpt-6-astra / high` を維持した。AGENTS、開発7役規約、TDD、wordweave-change、[I5-M1 Brief](praat-remediation.md)、[Pre-gate cycle1](praat-m1-gate.md)を参照した。親から独立Pre-gate PASSを受領しており、テスト/REDのみを担当する。model/effortの実行metadataと利用量counterは未取得である。

所有は `tools/acoustic-analysis/tests/support/process_unit.rs`、本書、担当logだけである。本体、manifest、別test、OS設定、UAC、実音声/学習データ、外部APIは変更・使用していない。I5-P1 / AH-AC09の作成直後親異常終了に限り、I5-Q1は対象外である。

## Oracleとfixture

追加した `process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling` は同じテストEXEをexact指定で別process起動し、限定env変数でsupervisor役へ分岐する。追加のtest-only本体seamや合成child変更は不要であり、既存AfterCreate hookと `acoustic-test-child.exe hang` を再利用する。

supervisor自身の `IsProcessInJob(GetCurrentProcess(), NULL)` がfalseであることを子作成前に要求する。AfterCreateではchild PIDを隔離temp内へatomic通知し、未resumeのままbarrierで待機する。第三者testが独立した所有child handleを開き、supervisor自身のJob未所属も再確認した後にsupervisorだけを強制終了する。

期待値は、cleanup前のchild waitが5秒以内に終了、未resumeのstarted sentinelなし、無関係の所有兄弟process生存、8秒watchdog救済なし、作成直後childのJob所属である。wait結果と副作用oracleを先に保存し、その後に残存child/兄弟を所有handleだけで回収する。救済後にwaitを取り直してPASSにする処理はない。既存fixtureの8秒watchdog、helper側7秒barrier失敗復帰、実行側45秒上限で有限化した。

既存 `child_does_not_execute_until_assignment_and_resume_are_complete` にもAfterCreate/BeforeAssignのJob所属assertを追加した。BeforeAssign/AfterAssignラベルと既存順序・sentinel・その他53件の期待値を変更していない。`IsProcessInJob(..., NULL)`単独は外側Jobを区別できないため、強制終了testのsupervisor未所属確認と組み合わせる。外側Job所属環境の成功だけを専用Jobの安全証拠へ読み替えない。

## 実行・現在の制約

`cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --no-run --locked --offline` はexit 0、compile 14.00秒である。既存の合成child EXEを使用した。単体targetは既存53件＋新1件の54件となる。本体修正前のコンパイルは成功したが、以下の理由で行動REDには未到達である。

凍結EXEへ `--exact process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling --nocapture` を渡し、`Start-Process -WindowStyle Hidden` と所有processの45秒WaitForExitを使用した。

- 通常sandbox: 0 passed / 1 failed / 0 ignored / 53 filtered、exit 101、0.13秒。supervisorの `CREATE_BREAKAWAY_FROM_JOB` 起動がWindows error 5で拒否された。[stdout](artifacts/praat-m1-parent-death-red.stdout.log)、[stderr](artifacts/praat-m1-parent-death-red.stderr.log)。
- 承認レビュー後のsandbox外・非管理者: 0 passed / 1 failed / 0 ignored / 53 filtered、exit 101、0.45秒。supervisor起動は成功したが、supervisor自身のJob所属がtrueで前提assertに停止した。子作成前である。[stdout](artifacts/praat-m1-parent-death-red-unrestricted.stdout.log)、[stderr](artifacts/praat-m1-parent-death-red-unrestricted.stderr.log)。

後者の失敗本文は `fixture prerequisite: supervisor must not be rescued by an outer Job` である。外側Jobの救済を除外できないfixtureを成功や行動REDに数えず、親へ外側Jobを持たない実行環境を照会した。二実行ともAfterCreateへ到達せず、残存音響childは作成されていない。所有supervisor/兄弟のhandle cleanupは終了した。skipやassert緩和は行っていない。

## 固定対象と引継ぎ

| 対象 | SHA-256 |
| --- | --- |
| process_unit.rs（変更前） | 9111C9D4F6D9E3E496D015F4C1C17695041E6C3CAEC5F6F24DD7FBDBD492EE43 |
| process_unit.rs（変更後） | CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896 |
| acoustic_analysis-8d1dd6b6ba45c163.exe | AC0CF780F4FF2C6C1FCF732B6A5DB46F38EB03AC6D23D8183D3E7B7AFDA497A9 |
| acoustic-test-child.exe | 6F53E36CC76FAD677E9707E4A1F559F6ED09352D330593E7C35BA5F4D78A8D65 |
| src/process_windows.rs（未変更） | F73399B120FD07E024182092F0E1D1A183E8659960B1ABB8AF740FD50ED3208B |
| src/process.rs（未変更） | 9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54 |
| src/praat.rs（未変更） | 6BD9D84ADC85E2EBB2E0E546DDCED6AB086405829DBF1B90C3653DAB45C43787 |
| src/praat_protocol.rs（未変更） | EF354A74EA2498D8BEC36F68A826BF031282000357EDD4094A9D80ED3AC8AB48 |

EXE絶対pathは `F:\Kazuhiro\GitHub\WordWeave5\tools\acoustic-analysis\target\debug\deps\acoustic_analysis-8d1dd6b6ba45c163.exe`、合成childはその親の `..\acoustic-test-child.exe` である。以後のbuildは停止し、親へ同一EXEの実行判断を返した。`rustfmt --edition 2021 --check tools/acoustic-analysis/tests/support/process_unit.rs` と `git diff --check` は成功。本体4本はreview時hashと一致している。

旧コードで期待する行動REDは、AfterCreate通知・親強制終了後、5秒waitがWAIT_TIMEOUTとなり、救済前の観測値で失敗することである。その際はsentinelなし、兄弟生存、watchdog救済なしを別々に確認し、leaked childを所有handleで回収して終了を記録する。修正後は同じoracleでWAIT_OBJECT_0を要求する。外側Job未所属を成立させられない間はこの受入を未検証として保持する。
