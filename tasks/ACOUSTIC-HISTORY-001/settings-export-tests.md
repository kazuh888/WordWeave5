# I2d-S 設定export 独立テスト

2026-10-12。担当はtest author。入力は承認済みAH-AC04/16、[design-i2.md](design-i2.md) §5–6のI2-T08/T08b、[implementation.md](implementation.md) I2d-S、I2d-H独立Post Closed（親提示HEAD `56b3462`）である。計画者指定と親確認の起動値は `gpt-6-astra/high`、fresh context。実行metadata、開始/終了token counterは未取得である。

完了境界は独立期待値、隔離fixture、focused REDと実在回帰名を実装者へ渡すところまでである。本体接続・GREEN・独立runner/Post・全体test/release・実画面のpicker操作は含まない。`agent-skills:test-driven-development` を全文読み、RED担当として使用した。

## 所有・入口

所有は `src/app/settings_ui.rs` と `src/app/control_settings.rs` の新 `cfg(test) mod i2_export_settings`、本書、専用logのみである。他者のimplementation.md追記は変更しない。両Rust fileは本体担当と同時編集しない。

親・designerが確定した入口は `write_settings_export(&self, &Path, &[u8]) -> Result<(), String>`、`export_deck_to(&mut self, &Path)`、`export_progress_to(&mut self, &Path)`、`export_diagnostic_report_to(&mut self, &Path, &str)`、`diagnostic_export_result(&self, &Path, &str) -> Result<String, String>` である。未接続の固定Err scaffoldだけを専用cfg(test)内へ置いた。本体の既存画面・atomic_writeは未変更である。

**実装者は本体の同名メソッドを追加する際、各module冒頭のRED-only scaffold implを除去する必要がある。** その除去は期待値変更ではない。その他のテスト・oracleを本体に合わせて緩めない。settings内の共有fixtureは2個のtest moduleだけが使うapp内部可視性であり、production APIではない。

## 受入とoracle

| ケース | 条件と独立期待値 | 追跡 |
| --- | --- | --- |
| S01 × TSV/JSON/report、S08 × JSONL | 実Storage.dir内のprogress/custom/pending/app.lock/backups/assets/audio-history/audio-history、既存と未作成leafを拒否。file bytes・directory/temp一覧とdraft/dirty状態不変、成功通知なし | T08、AC04/16 |
| S02 ×3、S09 | 通常外部・似た名前のdirectoryで新規/既存置換が成功。手書きTSV・診断text/JSONL、メモリProgressのpretty JSONとexact bytes一致。保護root不変 | T08b |
| S03 ×3、S10 | 不在の親directoryは自動作成せず拒否、入力とdraftを維持。同じ内容で通常外部へ再試行でき、前回error表示を解消 | T08/08b |
| S04 | disk root AとUI root BのJSON退避はUI Progressを出力する。diskのAを変えず、未保存会話/回答を保ち、別のsettings editor draftを混入しない | §5のmemory export契約、T08/08b |
| S05 | LOCALAPPDATAがない場合でも実Storage.dirを優先し、外部成功・実root配下拒否 | T08b |
| S06 | StorageなしはLOCALAPPDATA/WordWeave5を解決。不在rootへmkdir/lock/recoverなしで外部出力し、存在rootの既存/未作成leafを拒否 | T08b |
| S07 | Storageなし・root解決不能では既存/新規外部destとも拒否、全ledger不変 | T08b |
| S11 | JSONL拒否でSettings Started→Export Failed/Ioのみ、再試行成功で新operation Started→Completed。通知は戻り値を受け取った後。保護Progress bytes保持 | 既存diagnostic event保持、T08/08b |

S01/08のdisk rootは手製A、UIはNoneである。S04は手製参照Bを使用する。履歴の完全backupやroot commitとしての成功を要求しない。TSV oracleは手書き17列header＋拡張データなし15列rowという既存互換形式、reportは日本語・CRLF/LF・末尾空白を含む固定文字列である。JSON oracleはexport関数やpath guardを呼ばず、呼出し前のメモリProgressをSerializeする。settings editorは別snapshotとして比較する。

fixtureは既存 `harness_tests::fixture()` の実Storage（temp root/data）を利用する。未保存日本語chat/answer、編集設定、診断表示、注釈、dirty状態を追跡する。RAII cleanupは失敗のunwind時にもStorage lock解放後に実行し、作成/削除/残存をlogへ表示する。実学習データ、実送信、OS設定変更はない。

LOCALAPPDATA操作とdiagnostics::initializeは `current_exe --exact <case> --test-threads=1` の子processだけで行う。45秒でkill/waitする有限deadlineを持つ。exit0だけでなくstdoutの `running 1 test` と `1 passed; 0 failed` をassertし、0件を成功にしない。診断event試験では期待されるlog増分を許し、全ledger不変と誤認しない。他のケースでは診断sinkを初期化しない。

## 実行・引継ぎ

```text
cargo test --locked --bin wordweave5 i2_export_settings
cargo test --locked --bin wordweave5 -- --list
rustfmt --edition 2021 --check src/app/settings_ui.rs src/app/control_settings.rs
git diff --check
```

初回focused logは `logs/settings-export-red.log` である。compileは成功したが21件は `harness_tests.rs:202` の初期Storage::saveで対象処理へ未到達、4件は子filterのcrate prefix差により0件起動だった。初回の「4PASS/21FAIL」は受入成功/行動REDに数えない。prefixを取り除きexact1件をassertする限定訂正をした。fixture/本体の保存処理と期待値は変更していない。初回fixture失敗は所有RAII取得前で、個別path/cleanupは未取得である。無関係なTEMPを探索・削除していない。

compile失敗、fixture失敗、scaffoldによる行動REDを区別する。固定Errだけで拒否ケースがGREENになっても保護の実装証拠ではなく、実装後に同じoracleで再実行が必要である。最終結果は後段へ追記する。

workspace専用TEMP `logs/settings-export-temp` とHidden/Wait起動では初期saveが成功した。限定環境差であり、初回の失敗syscallは未取得である。sandbox外・管理者・OS設定変更は行っていない。最初のworkspace実行（PID29620、1PASS/24FAIL、`settings-export-final-red.stdout/stderr.log`）では、3件がscaffold行動RED、21件は作者ledgerが保持中のapp.lockを読みWindows error33に遭遇したfixture欠陥だった。app.lockはmetadata長0をassertして空bytesとして表現する限定訂正を行った。存在確認・通常fileのexact読取は維持する。初回workspaceの25個のfixtureはRAIIで削除され、専用TEMPの残存子要素は0だった。

最終source変更はこのledger訂正のみである。子filter補正後のlistは `logs/settings-export-list.log` に305件（新25件＋旧280件）の実在名を保持する。既存回帰は次の9件を実装者/runnerへ渡し、作者は追加実行していない。

```text
app::settings_edit::tests::settings_draft_survives_autosave_without_leaking_and_cancel_preserves_chat
app::settings_edit::tests::settings_model_catalog_uses_unsaved_request_path_without_saving_it
app::settings_edit::tests::settings_navigation_close_and_restore_never_silently_commit_a_draft
app::settings_edit::tests::settings_save_is_explicit_preserves_external_preferences_and_failure_is_retryable
app::settings_edit::tests::settings_zoom_drag_tracks_physical_pointer_without_jumping
app::settings_ui::tests::settings_categories_preserve_values_and_keep_save_visible
app::settings_ui::tests::settings_fit_between_native_navigation_and_status_at_maximum_zoom
app::settings_ui::tests::settings_recovery_focuses_the_visible_path_in_the_scrolling_form
app::settings_ui::tests::settings_zoom_control_keeps_native_position_and_size_at_all_scales
```

上記9件はfilter `app::settings_edit::tests::`（5件）、`app::settings_ui::tests::`（4件）で実行できる。native pickerと本体の接合は独立reviewで確認する。

### 最終RED（所有返却時点）

`cargo test --locked --bin wordweave5 i2_export_settings --no-run` は1分47秒、exit0でcompile成功（`logs/settings-export-final-build.log`）。最終source SHA-256はsettings_ui=`B72040DC2480CF40F8A8B7E1BD595C6DA0950CE58EE80FC5231F9D53E80B5CA7`、control_settings=`5F14ECB45E32E5818A8C417EC9273F7D55738BF8E1192B2560D58BD6FF9F6506`。

固定EXE `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` の実行前後SHA-256はともに `11A92FE756AC4E6C13B448E07E3B4065AF105C9E6D21A1E9B25AD30200ABE536`。sandbox内でprocess限定TEMP/TMPを前記専用pathに設定し、`Start-Process -WindowStyle Hidden`、filter `i2_export_settings --nocapture`、外側60秒deadlineで起動した。PID32296、native exit101、25件実行・**9PASS/16行動RED/0ignored/280filtered**、1.66秒、timeout0である。shellはログ読取後exit0だがnative exit101を合格へ読み替えない。

最終rawは `logs/settings-export-final2-red.stdout.log` と `.stderr.log`。16失敗はS02×6/S03×3/S04/S05/S06/S09×2/S10/S11である。S02/03/04/05/06/09/10は固定Errに対して成功を期待する箇所、S11はSettings Start/Failedの2件を期待したところ0件の箇所で失敗した。新入口の未接続を検出する行動REDであり、現画面のpickerを通じた既存bug再現とは区別する。子4件は各exact1件実行をrawで確認し、S07成功についてもwrapperが1件成功summaryをassertした。compile/fixture/OSエラーは最終実行にない。

created25/removed25/retained0、専用TEMP残存子要素0。`rustfmt --edition 2021 --check`（所有2file）と `git diff --check` は成功した。全体suite・releaseは実行していない。

両Rust fileの作者所有を返却する。本体担当は冒頭scaffold implを除去して承認5入口を接続し、同25件をGREENへする。9件の防御PASSも再実行する。通常Cargo実行の既定TEMPで再びfixture失敗が出る場合は、上記no-run→固定EXE→process専用TEMP/Hidden方式を再利用する。恒久harness/settings・検証基準の変更ではない。開始/終了usage counterは未取得のままであり、親の測定記録へ欠測を引き継ぐ。

### S04 oracle限定訂正（本体接合後・2026-10-12）

実装者のPID33464による新25件は24PASS/1FAILであり、`logs/settings-export-impl-new.stderr.log` のS04で出力wire Valueの `font_scale=0.8` と、`serde_json::json!(Progress)` がf32をValue数値へ拡張した `0.800000011920929` を比較していた。left/rightのこの数値表現だけを揃えると全文一致することを作者が読取比較で確認した。独立reviewerからもS04 P2受入阻止の報告を親経由で受領した。本件は製品のsnapshot内容欠陥ではなく、作者oracleの比較経路の不一致である。

訂正前に**実装後・oracle訂正前**の両sourceを次へ保存し、保存先hashが元sourceと完全一致することを確認した。これは未保存だった原作者RED sourceの遡及snapshotではない。原作者sourceについては全file hashとlogのみ保存していた事実を維持する。

| 対象 | 訂正前snapshot | 訂正前SHA-256 | 訂正後SHA-256 |
| --- | --- | --- | --- |
| settings_ui.rs | `logs/settings-export-s04-before-settings_ui.rs.txt` | `5575F9E5B5C138380DB273354BF3E12C5D87307F2A4237F254508D8B609871C3` | `C7C7C9D17DD24FC3FDF5762F7CD0E649A1A07E3E7237D31FE6C5A1FB63D1BF9B` |
| control_settings.rs | `logs/settings-export-s04-before-control_settings.rs.txt` | `F7A3FB0777B6E5791F7F4591953ABA0C0F2B16803C224460E81886D09DEF35C5` | 同じ（無変更） |

`git diff --no-index` でsnapshotと現settings_uiを比較し、差分がS04内の次だけであることを確認した。他testと本体は不変である。

```diff
 let state = memory(&f.app);
+// Compare wire snapshots: json!(Progress) widens f32 to a Value f64,
+// whereas JSON serialization preserves the shortest f32 representation.
+let expected_bytes = serde_json::to_vec_pretty(&f.app.progress).unwrap();
+let expected: serde_json::Value = serde_json::from_slice(&expected_bytes).unwrap();
 // export呼出しと成功assertは不変
-let output: serde_json::Value = serde_json::from_slice(&fs::read(dest).unwrap()).unwrap();
-assert_eq!(output, state["progress"]);
+let actual_bytes = fs::read(dest).unwrap();
+assert_eq!(actual_bytes, expected_bytes, "exact pre-export memory snapshot");
+let output: serde_json::Value = serde_json::from_slice(&actual_bytes).unwrap();
+assert_eq!(output, expected);
```

許容誤差やfield除外は追加していない。期待値はexport前のProgressから独立生成し、export本体・helper・出力結果から作らない。root B、未保存会話/回答、settings editorの非混入、disk root Aとmemory全体の不変assertを維持し、元のValue比較に加えてexact wire bytes契約を明示した。

親の所有返却後、作者はS04 exact1件だけを再build/実行する。成功済み新24件と旧9件はこの訂正で変わらないため反復しない。最終独立runnerの対象は新25＋旧9＝34 uniqueであり、作者の今回exact実行を別受入として加算しない。

S04再検証は完了した。`cargo test --locked --bin wordweave5 app::settings_ui::i2_export_settings::s04_json_exports_memory_root_and_drafts_without_committing_editor --no-run` は1分42秒・exit0（`logs/settings-export-s04-build.log`）。表示された同じtest EXEをprocess専用TEMP/Hidden起動し、`--exact app::settings_ui::i2_export_settings::s04_json_exports_memory_root_and_drafts_without_committing_editor --nocapture` で **1PASS/0FAIL/0ignored/304filtered**、PID31472・exit0・0.38秒・timeout0だった。rawは `logs/settings-export-s04-green.stdout.log` と `.stderr.log`。created1/removed1/retained0、TEMP残存子要素0である。

実行前後EXE SHA-256はともに `287E9A9743D5B21E0B0829E715016885099D690941CEC7ECFCA5C618720B0737`。訂正後のrustfmt checkとdiff checkは成功した。compile/fixture失敗はない。親へ当該testと報告の所有を返却し、独立runner/Postへ渡す。今回の開始/終了usage counterも未取得である。

共通HのWindows alias/identity/link/TOCTOU/poison・write-blockは再試験せず [H独立runner](export-destination-runner.md) と [H独立review](export-destination-review.md) に追跡する。consumerから同じhelperへ到達することは実装diffの独立reviewが必要である。write-block設定用の公開seamは増設しない。

作者RED時点のUIの4出口はatomic_writeを直接呼ぶことを静的確認したが、native file pickerを自動選択して実報告bugを再現してはいない。新入口のscaffold REDはその代替証拠ではない。picker取消しは入口を呼ばない既存の `if let Some(path)` であり、credit_timeはJSON picker前にある既存動作を維持する。関数テストはpickerとcredit_timeを含まない。native picker取消し、UIボタンからの実配線、実write-block中のconsumer UIは独立した未測定項目である。
