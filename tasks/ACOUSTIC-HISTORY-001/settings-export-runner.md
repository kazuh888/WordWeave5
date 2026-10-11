# I2d-S 設定export 独立runner

2026-10-12。完了境界は確定ソースの新25＋既存9＝34 uniqueの独立実行と結果引継ぎである。I2d-S最終Post/Closed、I10全体test/release、製品EXE更新は含まない。

入力は承認済みAH-AC04/16、design-i2.md §5–6 I2-T08/T08b、settings-export-tests.md、settings-export-implementation.md、settings-export-review.md（S-ORACLE-1限定Post PASS、runner-ready）である。計画者割当はimplementation.mdのI2行、`gpt-6.1-sol / medium`。親は固定差分の手順実行として明示dispatchした。実行model/effort metadata、入力/キャッシュ/出力/推論の開始・終了counterは未取得であり推測しない。AGENTS.md、development-team.md、kpi.md、wordweave-changeを全文読み、Skillのリスク別検証と報告のみ適用した。

## 固定対象

開始/終了HEADはともに `476bad7475c1e7375e80c98bdee1bc29b753ae7b`。開始/終了git statusのtracked変更は親所有の `docs/process/improvement/measurements.md` 1件のみ。アプリsource/test/期待値/settingsの差分はなく、親の測定記録を保持した。runnerの所有は本書、専用logs、隔離fixture、build生成物のみである。

| file | 実行前/全実行後SHA-256（同一） |
| --- | --- |
| src/app/settings_ui.rs | C7C7C9D17DD24FC3FDF5762F7CD0E649A1A07E3E7237D31FE6C5A1FB63D1BF9B |
| src/app/control_settings.rs | F7A3FB0777B6E5791F7F4591953ABA0C0F2B16803C224460E81886D09DEF35C5 |

着手時既存test EXEは引継ぎ値 `287E9A9743D5B21E0B0829E715016885099D690941CEC7ECFCA5C618720B0737` と一致した。独立build後の確定EXEは `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe`、SHA-256 `A11ECC2332B3B649FBA6670A6F193F5156DC875EB67E7B656460517560F3F475`。今回list/3filterの前後ともこのhashである。旧EXEの結果を新結果に転用していない。

## 今回の実行

`cargo test --locked --bin wordweave5 --no-run` を1回だけ実行、native exit0、2分14秒でcompile成功。rawは `logs/settings-export-runner-build.log`。依存crate再コンパイル、incremental hardlinkからcopyへの警告、既存unused/dead-code/f32関連警告を記録した。修正は行わない。

確定EXEを `Start-Process -WindowStyle Hidden`、外側60秒deadline、process限定workspace TEMP/TMP、try/finally環境復元で実行した。子processの45秒deadline/exact1件assertは既存testのままである。専用実行手順は `logs/settings-export-runner-execute.ps1` に保存した。

| 引数（確定EXE） | PID / native exit | 新しいraw結果 | timeout |
| --- | --- | --- | --- |
| `--list` | 12972 / 0 | 実在305件、新25・settings_edit5・settings_ui4をassert | false |
| `i2_export_settings --nocapture` | 22156 / 0 | 25 passed / 0 failed / 0 ignored / 280 filtered、2.81秒（外側2.85秒） | false |
| `app::settings_edit::tests:: --nocapture` | 欠測 / 永続化欠測 | 5 passed / 0 failed / 0 ignored / 300 filtered、0.82秒 | 永続化欠測 |
| `app::settings_ui::tests:: --nocapture` | 34700 / 0 | 4 passed / 0 failed / 0 ignored / 301 filtered、0.69秒（外側0.91秒） | false |

各rawは `logs/settings-export-runner-{list,new,edit,ui}.{stdout,stderr}.log`。新25件と旧5/4件はlist上の互いに非重複filterであり、**34 unique passed / 0 failed / 0 ignored**。filtered countsは各filter外でありskip合計として加算しない。対象34件の未実行は0である。

集計スクリプト初回は旧edit5実行後、空stderrをnullのままregexへ渡して例外となった。shell exit1であり、製品compile/fixture/行動失敗ではない。`logs/settings-export-runner-execution.log` にfinallyの初回結果とhash/環境復元/TEMP0を保持する。例外箇所は当時scriptの44行で、editのWaitForExit→Refresh→ExitCode取得→raw summary取得の後、results配列追加の前である。edit native exit変数はその経路で取得されたが永続化されていないため、exit0確認済みとは主張しない。PID/timeoutも欠測として残す。成功summaryの5件は新rawで確認した。

担当logs内の集計だけをnull→空文字列へ限定訂正し、残UI4件だけを実行した（shell exit0）。成功済み新25/旧5とlist/buildは反復していない。続行metadataは `logs/settings-export-runner-remaining-execution.log`。製品source/test/期待値/harness/settings/gatesは変更していない。

新fixtureはcreated25/removed25/retained0。初回専用TEMP `logs/settings-export-runner-temp-0f654a093937423db68aa98ad420fc44` と続行専用TEMP `logs/settings-export-runner-temp-8f3d727b553d4b498b0a1c3124eb657c` は各finallyと全実行後の確認で残存子要素0、空directoryのみ保持した。両実行でTEMP/TMP復元true。実学習データ/API/管理者/OS変更、アプリkill、commit/pushはない。

`rustfmt --edition 2021 --check src/app/settings_ui.rs src/app/control_settings.rs` と `git diff --check` は今回ともにexit0。sourceの実行前後hash一致により試験期待値を含む2file不変を確認した。旧source before snapshot欠測は作者/レビューの既存制約のままであり、今回補完したとは扱わない。

## 親への引継ぎ

固定revision上の独立34件成功を返す。記録スクリプト例外1件とedit runtime metadata欠測を独立Postへ明示する。新25には拒否/外部成功/置換/失敗再試行/保護rootと未保存draftの保持/JSONL eventを含む。共通H136や成功済み他suiteは反復していない。

native picker取消し、UIボタンからの実操作、正本write-block時のconsumer実UIは未測定である。native microphone/pen/TTS/IME/実Codex認証の新しい結果はない。I10の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` は本runner範囲外で未実行。最終Post/I2d-S Closed/製品deliveryの判断は親へ返す。

## edit5 metadata欠測の限定再取得

親は最終Postの実行証拠に必要なnative exit/timeout欠測を未解決リスクと判断し、旧edit5だけの再取得を明示指示した。モデル割当Sol/mediumと所有制約は継続した。成功済み他29件、build/list、format検査は反復していない。上記の初回欠測と集計例外ログは保持する。

開始HEADは同じ `476bad7475c1e7375e80c98bdee1bc29b753ae7b`。開始source2本とEXE hashは上記確定値と一致した。開始git statusは親所有measurements.mdと本runner報告の新規ファイルのみである。

`logs/settings-export-runner-edit-metadata.ps1` は固定EXEの `app::settings_edit::tests:: --nocapture` をHiddenで起動し、PIDを先に出力、60秒wait後のnative exit/timeoutをsummary集計前に出力した。全出力を `logs/settings-export-runner-edit-metadata.execution.log` へ保存した。

結果は **PID29836 / native exit0 / timeout=False / 外側0.971597秒 / 5 passed・0 failed・0 ignored・300 filtered**、shell exit0である。新rawは `logs/settings-export-runner-edit-metadata.stdout.log` と `.stderr.log`。process限定TEMP `logs/settings-export-runner-edit-temp-9354a23b3f0d47d38a4ccb85e73f0f9b` は残存子要素0、finallyの環境復元Trueであった。実行後source2本/EXEのhashは実行前と一致し、source/test/期待値/settingsは変更していない。

これにより現在の採用証拠は新25（PID22156）、旧edit5（PID29836）、旧ui4（PID34700）の **34 unique成功、各native exit0・timeoutなし** で揃った。限定再実行5件はuniqueへ重複加算しない。初回edit5のmetadata欠測そのものを遡って埋めたとは主張せず、新規実行の記録である。残るnative受入/I10/最終Postの範囲は変わらない。
