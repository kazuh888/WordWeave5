# I5-M1 実装引継ぎ

2026-10-07 / ACOUSTIC-HISTORY-001。planner指定implementerは`gpt-6-astra/high`。実行model/effort metadataと開始・終了token countersは未取得である。AGENTS.md、development-team.md、KPI、wordweave-changeを読了した。完了境界はM1限定本体修正とfocused検証であり、独立最終検証・Post-review・利用者受入を含まない。UIは非該当である。

開始時にdirty差分を確認し、Pre-gateのBrief/design-i5/design-i0の3文書と修正前sourceのSHA-256一致を確認した。親から外側Jobのない利用者環境での旧実装behavior REDとbuild停止解除を受領した。本体所有は`tools/acoustic-analysis/src/process_windows.rs`のみであり、独立test・fixture・manifest・他本体・正本文書は変更していない。

## 変更

- HANDLE_LISTとJOB_LISTの2属性でCreateProcessWを呼ぶ。既存stdin/stdout/stderrだけの継承を維持し、非継承の専用JobはJOB_LISTへ渡す。
- `Attributes<'a>`が両値配列を借用し、DeleteProcThreadAttributeListまで配列の寿命を型で保持する。
- 作成成功直後のChildGuardは`assigned=true`である。後続AssignProcessToJobObjectを削除し、既存BeforeAssign/AfterAssignラベル間で専用Job所属を確認する。
- 所属なしを明示観測した場合だけ`assigned=false`とし、既存の所有processへのTerminateProcess→waitを使う。照会API失敗は作成時Job所属を維持して停止する。resume前の失敗分類・取消し・期限・512MiB等の契約は変更していない。

## focused結果

通常sandbox、非管理者、通常cargoで実行した。過去の53成功はHidden凍結EXE＋sandbox外・非管理者であり、今回と実行条件が異なる。

| command | 結果 |
| --- | --- |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline -- --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling` | compile成功21.59秒。52 passed / 1 failed / 0 ignored / 1 filtered、runtime 2.93秒。M1 exactは既知の外側Job制約により明示filterし、未検証である |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline process_unit::inheritable_parent_event_is_excluded_from_child_handle_list -- --exact --nocapture` | compile成功13.64秒。0 passed / 1 failed / 53 filtered、runtime 0.96秒。初回と同じstdout空対`blocked`の失敗 |
| 同exact、一時cfg(test)数値診断付き | compile成功11.56秒。0 passed / 1 failed、runtime 0.99秒。`exit=0xc0000008 stdout=0 stderr=0`。一時診断は除去済み |
| `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --features praat --lib --locked --offline --no-run` | 診断除去後compile成功11.88秒。下記EXEを確定 |
| `rustfmt --edition 2021 --check tools/acoustic-analysis/src/process_windows.rs`、`git diff --check` | 成功 |

失敗は`process_unit::inheritable_parent_event_is_excluded_from_child_handle_list`の出力比較だけである。superviseのunwrapは成功し、子の終了codeはローカルwindows 0.56 Foundation定義の`STATUS_INVALID_HANDLE`と一致した。collectは所有processの終了と回収を確認後、stdout/stderr EOF・completion queue空を確認してそのcodeを返している。未回収/pipe読み出しErrとは異なる。

合成childのevent分岐は親のraw handle数値を子のSetEventへ渡し、Result判定後に`blocked`を出力する。異常終了がその呼出から生じたか、M1変更へ帰属するかは未確定である。raw handle値はprocess間で同一objectを意味しない。試験期待値を変更せず、親へ独立runnerによる以前と同条件のexact比較を返した。旧source比較は未実施である。compileの既存dead_code 2件とincremental hard-link代替copy警告は未修正である。

## 確定対象・残余

| 対象 | SHA-256 |
| --- | --- |
| process_windows.rs | `10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C` |
| 独立process_unit.rs（不変） | `CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896` |
| target/debug/deps/acoustic_analysis-8d1dd6b6ba45c163.exe（診断除去後） | `146617DB1AC55BD72478E63CC27C910989AEE4B74F7C8551E0FEA314CAC61F8A` |
| target/debug/acoustic-test-child.exe（不変） | `6F53E36CC76FAD677E9707E4A1F559F6ED09352D330593E7C35BA5F4D78A8D65` |

パスはcrate `tools/acoustic-analysis`からの相対である。M1 exactの修正後GREEN、event回帰失敗の帰属、実Praat公開20/default59、独立Post-reviewは未完了である。Q1/M2、I5c-2、資源実測、全app検証、実機受入は本修正に混在させていない。実データ・実API・OS設定・UAC・再委任・commit/pushを行っていない。本体所有を親へ返す。
