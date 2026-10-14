# ET2-S/F 本体接合記録

2026-10-14。実装担当 `/root/trial_startup_impl`、thread `01a13962-a5dd-7822-9764-05ea27255dc5`。planner指定 `gpt-6-astra/high`、runtime/provider返却model・effortは未取得である。AGENTS、development-team、KPI、wordweave-change、wordweave-egui-uiを全文確認した。後者はtitle・安全通知の既定表示境界に適用し、native描画の受入は実施していない。

## 最初の返却境界：source固定、ET2-G0親読取前

入力は契約 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、設計 `24B9FEBE6F83528EF1B4D55EF9890B00FC9846E9F50B5BB446F48A2FAE687D7E`、計画 `82E0A3E8A326EBEB2456ACC994731895072F3B714286452598435547ADC31AFD`、独立Pre-2 PASS、作者記録 `A4BE9500375984C9DC60F066C55E29CA7DC51E81A111B40C5E143CF5CBBD3E21` である。入力hashを現物と照合した。branch `codex/development-agent-team`、HEAD `9a7eb5c94041f0937f8bc3cfac35bdee2f8280e1`、製品source基準97d2c72、開始時git statusはcleanであった。

変更は許可された本体5fileだけである。6番目の `src/app/qwen_settings_tests.rs` は全bytes不変。作者の専用module6個は全hashが作者記録と一致し、期待値変更・既存assert変更は0である。再委譲、commit、push、実データ、実credential load/save、API、Recorder runtime、native、OS設定変更は0である。

- `src/main.rs`：feature別titleをviewportへ接合。試用Storage::openをrun_native前に実行し、安全Stringを無加工で型へ包む。open拒否ではclosureを生成しない。factory拒否では正本rfd title/共通追記を表示し、boxed errorだけを返す。
- `src/app.rs`：通常newのdiagnostics/fatal順序を保持し、既存constructorのloadだけをFnOnceへ置換。試用共通factoryは一回constructorを呼び、fatalなら型付き定型Err/D0、成功時だけprivate bool=trueと試用diagnostics/D1を行う。history restoreは全既存検査・退避より先に正本文言で拒否する。
- `src/qwen_reading.rs`、`src/app/qwen_settings.rs`：feature別target、private selectorをproduction for_targetへ接合。test default_storeは既存mockを保持する。資格情報実体のload/saveを新設していない。
- `src/media.rs`：cfg(test) global watch、登録・重複拒否・calls・RAII Dropと、start先頭のcfg(test)拒否だけを追加。非testの録音本体と3callerは変更していない。

## 親によるG0読取箇所

`src/media.rs:193` の `RECORDING_START_WATCH` を登録関数201行とRecorder::startの287行が共用する。startはlock取得→active判定→lock内fetch_add→固定Errを行い、cpal開始は295行である。poisonはlock取得のmap_err/`?`でcpal前に拒否する。未登録の正常lockだけが既存cpalへ進む。Dropは221行で同じArcの場合だけ登録を外し、poison guard回収時にもclear_poisonを呼ばない。任意reset口はない。

作者の安全対照はmedia.rs:91以降、watchの成功必須→直接入口→重複拒否→worker入口→join→calls確認→dropの順である。unwind対照は132行以降、poison対照は158行以降にある。これらの読取自己確認は親のG0判定と動的正対照を代替しない。

他の接合箇所はmain.rs:51/56/73/92（title/creator/dialog/open）、app.rs:615/636/784/803/828/2791（型/診断/通常wrapper/試用factory/共通loader/restore）、qwen_settings.rs:226/233（selector/default_store）である。startの既存callerはapp.rs:1986、chat_media.rs:1200、qwen_reading_ui.rs:307であり、新入口を追加していない。

## 固定sourceと作者oracle

hashはファイル全bytesのSHA256、module hashは作者と同じ最初のcfg属性からmodule閉じ括弧までをUTF-8化しTrimEndした値である。

| source | 固定source SHA256 | 作者module SHA256（不変） |
| --- | --- | --- |
| src/main.rs | A646CB6B4524188DD8DF88F3067A6ACB30ECF8F803C3B6AD6ADBEC6B30B1489D | 73F14DD72E8AE1A0A1B6F42E5C508A54C3C911FE65BC28F73F06581F9316EFD4 |
| src/app.rs | 2AA13D474D8B8F2F6ED47B1D6E57FC75BA342C6C2822490927E38793860C3C56 | 3877EFEAEDBBC7A8BB88A1987070313D7B59AFB632D2DDB1EBF97D6B7C5DC4AA |
| src/qwen_reading.rs | 3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64 | FAE2E01F3926D3AAC196DCCEA603BB7D574711C2FBFDB2402617256A09EC76A0 |
| src/app/qwen_settings.rs | D847555998E71987E8519D8B8B4811D68231054A043A365F3D040BA0339E11C8 | A9881AF09824509E98202B9E673B0E22FC724653FC4AF4255962EB5F970034B6 |
| src/app/qwen_settings_tests.rs | 45EF01F0F3DB4A2302AE8DAF9FC10834190D427AB648823139B46DC49DE324F1 | C6550481C8B381BD44EF0419F9437D4046F7C06BA4A1C189C2EECAD2A10281C0 |
| src/media.rs | 14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207 | DCD6637D4AB50C170E7B6AD22B49F9CC99E0352A499D9B5F37AD1E30A9352FAA |

HEADからのsource差分は5file、207追加/12削除。各fileはapp 90/1、qwen_settings 12/5、main 45/6、media 56/0、qwen_reading 4/0である。本記録だけが追加文書で、current/usageは親所有のままとする。

## 検証・未検証

`cargo fmt --check` と `git diff --check` は最終整形後exit0。最初のfmt checkが指摘した本体4箇所だけを整形した。作者moduleを含むhash不変を整形後に確認した。eframe 0.31.1のローカルsourceでAppCreator FnOnce / AppCreation boxed error型を照合した。コンパイルとruntimeは未確認であり、compile blockerの解消を実測済みとしない。

ET2-G0の順序どおり、ここで変更を停止し親のsource確認へ返す。次段階は確認済みsourceから通常/試用別target-dirの `cargo test --locked --bin wordweave5 --lib --no-run --message-format=json`（試用だけ `--features early-trial`）である。ログ予定は `target/early-trial-build/et2-impl/`、EXE絶対path/SHA/config/実名とbuild前後source hashを固定した後、親の明示解除を要する。今回はno-runも未実行で、旧EXE hashは流用しない。

未完了はコンパイル、ET2-G0親判定・EXE固定、動的安全3件、ET2機能集合/既存回帰、感度確認/逆復元、独立runner/Post、ET3全gate・release・native/DPI/実機受入である。作者reportどおり、起動API送信0はfactoryへtransport注入がないため直接動的観測ではなく、録音watchと起動経路の静的確認を区別する。今の実装確認範囲では追加設計判断・所有拡大は不要であったが、未実行であるためfixture健全性/OS fault成立/動作PASSは未判定である。

## Usage取得範囲

担当sessionのtoken_countだけを読取共有で取得した。開始側の実在snapshotは `2026-10-14T07:48:59.445Z`、input38663/cached23040/output170/reasoning0。終了側の最新観測は `2026-10-14T07:51:27.352Z`、input1052203/cached929536/output5466/reasoning635である。この後の記録作成・最終応答は未反映、開始前区間と親子包含関係は不明であり合算しない。週枠は親測定へ委ねる。本文/秘密は記録へ複製せず、runtime model/effortは未取得のままである。

## C1 no-run build 固定（親source確認後・runtime未解除）

親の `early-trial-startup-review.md` 末尾「候補C1のsource読取確認」を受領してから、次の2構成をビルドした。6sourceはbuild前・両build後とも上表のSHA256と一致し、ソース編集は0。親所有review文書の追記を保持した。`git diff --check` もexit0である。

```text
cargo test --locked --bin wordweave5 --lib --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/default-target
cargo test --locked --bin wordweave5 --lib --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target
```

通常はexit0 / build-finished.success=true / 2m17s、試用はexit0 / build-finished.success=true / 1m55s。コンパイルエラーは双方0である。JSONのwarningは通常7件、試用8件で、通常7件はprogress_wire_version（lib構成別2件）、control_settingsのunused import、store試験fixtureのtrial field、qwen_reading_testsのf32 fallback、ai.material、media.finishである。試用だけ通常constructorのnew unusedが1件増える。これらを本体/作者範囲外の修正へ広げていない。`--no-run` だけであり、test本体・`--list`・native/Recorder・実credential/APIは実行していない。

JSON compiler-artifactから得たprofile.test=trueのEXEを以下へ固定する。通常features=[]、試用features=[early-trial]である。絶対pathはJSONのpathを正規化したものである。

| 構成/target | EXE絶対path | SHA256 |
| --- | --- | --- |
| 通常/lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\default-target\debug\deps\wordweave5-b3388a7bbef44bb4.exe` | E9AC3D17DEE4AB0BA73F3AC6613B3EE4AA83770AF187142C38083985D511D974 |
| 通常/bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\default-target\debug\deps\wordweave5-72e8adffa8f9ef2c.exe` | 3E2F2D987FD73553086C425229E1ED39E4A5B630690E42214ABD6CF62D24EF7A |
| 試用/lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\trial-target\debug\deps\wordweave5-1e58dd7001d62083.exe` | 135273D6F8A1C837ED701C2F0718A2F47E274E6852D843CBF89D719845906415 |
| 試用/bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\trial-target\debug\deps\wordweave5-e0ab9f88db3adadd.exe` | 15B231707BB6FE50E3658688E3EC542FF82AA64D47D0B97A154BF41AE6167687 |

ログは `target/early-trial-build/et2-impl/` 配下に保存した。

| log | SHA256 |
| --- | --- |
| default-build.jsonl | 91CC09519E3B4C9286BD76E9BA56083D9CD0B192D5700E2B1BDD06D3B1DE35EB |
| default-build.stderr.log | 4CF1BA78D5BDD73DF7B472272B0569E57071B567A104BFD4F417BE8B2DD264AA |
| trial-build.jsonl | AA19F01562F855E9372E68C9140AF5679A0D32477B5EBCE437FCAAFE86B6BC9D |
| trial-build.stderr.log | FCE57C40E518E5846B89BD8DFDB92EB3B3A9B5BE3F1243B7B846743F5436C9CC |

### 解除後に照合する予定実名集合（EXE --list 未実行）

作者の固定18case一覧を基にした予定集合である。#1はlib、#2–18はbin。試用は全18件（bin17+lib1）、通常は○の7件（bin6+lib1）を予定する。実EXE一覧との照合・実行件数の証明はまだ行っていない。安全3件は#4→#5→#6の順で、親の固定source/EXE/configへの明示解除を受けてから実行する。0件成功除外やET06不足の解消をこの表の存在から推定しない。

| # | exact name | 通常 |
| --- | --- | --- |
| 1 | `qwen_reading::early_trial_startup_tests::et02_credential_target_matches_independent_feature_literal` | ○ |
| 2 | `early_trial_startup_tests::et03_title_matches_independent_feature_literal` | ○ |
| 3 | `early_trial_startup_tests::et_d01_creator_preserves_open_rejection_without_creating_closure` | — |
| 4 | `media::early_trial_startup_tests::et2s_active_watch_catches_entry_worker_and_rejects_duplicate` | ○ |
| 5 | `media::early_trial_startup_tests::et2s_unwind_releases_only_owned_watch` | ○ |
| 6 | `media::early_trial_startup_tests::et2s_poison_is_fail_closed_and_drop_does_not_clear_poison` | ○ |
| 7 | `app::early_trial_startup_tests::et_d01_real_archive_denial_after_both_writes_then_same_data_retry` | — |
| 8 | `app::early_trial_startup_tests::et_d01_recovered_ledger_survives_actual_constructor_read_denial` | — |
| 9 | `app::early_trial_startup_tests::et_d01_raw_loader_error_cannot_change_failure_stage_or_safe_text` | — |
| 10 | `app::early_trial_startup_tests::et04_history_json_rejected_before_backups_drafts_epoch_or_disk_change` | — |
| 11 | `app::early_trial_startup_tests::et04_nonhistory_json_and_settings_survive_trial_reopen` | — |
| 12 | `app::early_trial_startup_tests::et04_backup_confirmation_cancel_and_approved_restore_keep_history_none` | — |
| 13 | `app::early_trial_startup_tests::et04_material_import_cancel_then_approval_and_word_cache_keep_history_none` | — |
| 14 | `app::early_trial_startup_tests::et04_generated_proposal_requires_visible_approval_and_keeps_history_none` | — |
| 15 | `app::early_trial_startup_tests::et04_explicit_attachment_and_drop_cancel_approval_keep_history_none` | — |
| 16 | `app::qwen_settings::early_trial_startup_tests::et02_production_selector_constructs_exact_windows_target_without_credential_io` | ○ |
| 17 | `app::qwen_settings::early_trial_startup_tests::et02_selector_editor_controller_use_only_selected_namespace_and_no_send` | ○ |
| 18 | `app::qwen_settings_tests::early_trial_startup_tests::et06_factory_settings_restore_and_reading_prepare_do_not_record_probe_or_send` | — |

### 親への未達返却

コンパイル不在API28件のblockerは解消し、固定作者moduleは両構成で型検査を通過した。ただし行動試験の合格ではない。親からET06の動的観測不足P2が返却された。Wire/Probeはfactory後で、Wireは独立dialogにしか入らず、起動・設定save・restoreの送信0を直接測っていない。C1本体の確定欠陥は未検出との親報告であるが、受入oracleは未充足である。追加seam/所有拡張/期待値変更は行わず、親→plannerの限定delta判断へ返す。この制約に関係なくruntimeは未解除のままである。

G0の対応EXE固定まで完了したためここで停止する。親の明示解除なしに `--list` や安全3件を含むEXE実行へ進まない。C1 source/EXE/config変更時は再Gateとなる。最終runner/Post/全体release/ET3/native受入は未完了である。

同sessionの追補終了側snapshotは `2026-10-14T07:58:58.453Z`、input3786092/cached3646464/output12129/reasoning1270。この後の追記・最終応答は未反映である。返却model/effortと親子包含は未取得、週枠は親測定とする。

## C1 限定runtime結果：安全対照PASS、試用#3 FAILで停止

親のreview記録「C1対応EXEの限定実行解除」を受領し、固定4EXEだけでlistと新caseを実行した。各起動の直前にWindowsIdentity/PrincipalからAdministrator=Falseを確認し、SHA256一致を強制した。実行は既知sandbox OS5を反復せず、承認された通常ユーザーのsandbox外で行った。管理者/UAC、実データ、実credential load/save、実API、実マイク、native UIの操作は0である。

実行補助はignored `target/early-trial-build/et2-impl/Invoke-C1Case.ps1`、SHA256 `CD3425E1042A312B897163D0382851214AA3F4E62948A487AD04819B944325E7`。powershell-automationとversion compatibility/coding style/error handling/helpの指示を読んだ。実環境PowerShell7.6.6に合わせtarget ps7（7.4以上）とし、パラメータでEXE/hash/実名/outputを受け取る単発scriptとした。parse error0、Get-Help確認済みである。共通harness/sourceは編集していない。scriptの本体はHidden起動、65秒timeout、必要時process tree killとwait、合成LOCALAPPDATA/TEMP/TMP/cwd、exact1件成功検査、作者child marker、残存検査を実施する。timeoutは発生せずkill経路は未実行。外側の空synthetic directoryとログは証跡として保持し、削除機能をscriptへ設けていない。

### 件数・実名・順序

4EXEの `early_trial_startup_tests --list` は成功し、作者一覧との集合差分0であった。通常bin6/lib1、試用bin17/lib1。listを動作PASSには数えない。

続いて通常#4→#5→#6、試用#4→#5→#6の順で安全対照を行い、6件ともPASS。直接startのcalls1、別workerのcalls2/join、重複拒否、unwind後再登録、poisonのfail-closedとDrop後poison維持が固定assertを通った。synthetic panicは作者が捕捉する目的対照であり、実録音を開始していない。

その後に通常#1/#2/#16/#17、試用#1/#2/#3を順に実行し、試用#3で失敗した時点で全解除を停止した。内訳は次のとおりである。

| 構成 | 実行したcase | PASS | FAIL | 未実行 |
| --- | --- | --- | --- | --- |
| 通常 | #4/5/6/1/2/16/17 | 7 | 0 | 0 |
| 試用 | #4/5/6/1/2/3 | 5 | 1（#3） | #7–18の12件 |
| 合計 | 独立exact13件 | 12 | 1 | 12 |

全17起動（list4＋case13）でAdministrator=False、固定SHA一致、timeout=false、outer wait完了、fixture残存0である。isolated caseの成功6件はinner child marker/1件成功も確認した。試用#3はinnerもexit101で終了し、作者fixtureはwait後cleanupを完了している。補足のread-only列挙でも`runtime-c1`配下の`ww5-et2-*` directory残存0であった。子の失敗を目的行動PASSへ置換していない。

ログは `target/early-trial-build/et2-impl/runtime-c1/{default,trial}-{bin,lib}-list/` と `{default,trial}-case<ID>/` のstdout.log、stderr.log、result.jsonへ保存した。result.jsonは開始/終了、PID、SHA、Admin、timeout、wait、exit、残存を持つ。対象外のcaseをこの結果から合格扱いしない。

### 停止理由と作者へ返したfixture疑義

失敗実名は `early_trial_startup_tests::et_d01_creator_preserves_open_rejection_without_creating_closure`。EXEとinner childはexit101、目的assertは `src/main.rs:27` の `StorageOpenRejected { safe_message }` と期待文一致である。outerは`src/media.rs:85`でchild失敗を報告した。closure未生成のmatch自体は通過したが、expected安全文を確認できなかった。後続のledger/diagnostics assertへは達していない。

実測logにactual safe_messageは出ていない。read-only調査ではmain.rs:22のfixture JSONがversion/audio_historyだけであることを確認した。一方store.rs:1134のProgressWireはsettings/memories/reviews/suspended/ai_calls/study_secondsをserde defaultなしで要求する。decode_progress:1252のErrはcheck_early_trial_contents:1847でInvalidProgressOrPendingへ変換され、HistoryPresentの判定より前に返る。既存ET1のstore.rs:671 fixtureは全必須fieldを含む。したがって不完全fixtureがhistory専用本文を要求した可能性が高いという静的推論を親へ返した。実際の安全文は未観測であり、返却値まで動的確定したとは記録しない。

本体/作者module/期待値変更は0、再実行0。親に作者のfixture確認を依頼し、source修正や再Gateを先取りしていない。変更後source6hashは全てC1表と一致し、`git diff --check` exit0である。以後のruntime解除は停止している。

| 失敗log | SHA256 |
| --- | --- |
| trial-case3/stdout.log | 388BD1DDE376873BF23DC1173B918A1B1EC795242E77D45575B33199C7DBC2FB |
| trial-case3/stderr.log | 6AA7D27DEFEA9A5E248DDEDCC28790CDFB5C1CAEEF4FA0A4E294E61B3CC62BF3 |
| trial-case3/result.json | 4CBA6F943F4AAA0F70401F9DF943B04B78D63613E2BA8529FA129A04DB1301C9 |

ET2-P01（起動/設定save/restoreの動的送信0観測不足）は未解消である。今回の成功範囲でも最終ET2受入、感度確認、既存全回帰、独立runner/Post、ET3/nativeは完了していない。

同session最新snapshotは `2026-10-14T08:06:35.063Z`、input6203193/cached6018048/output23006/reasoning2699。記録追記と最終応答は未反映、runtime返却model/effort・親子包含は未取得である。

## C2 no-run再固定（作者fixture是正後・runtime未解除）

独立作者のmain専用module是正と親のC2 source再確認を受領してから実施した。作者report SHAは `FAA78447E44D4A24376BE0F52CA67DFD4EBE3F2A8E2ACE57511FABF529F443D7`。main全file SHAは `66BDD7137811C5C954A12C8C1E5461D5AF8933DC832D2282F2F37ACDE45C2025`、作者module SHAは `84A5F2A3DF6E645B105A5453FC51B3A6E835D8FEA9F0B5895C6FD4A7B803149C`。他5sourceは上記C1表と一致し、6sourceのbuild前後hashも完全一致した。今回実装担当によるsource変更は0である。親/planner/作者所有の文書追記を保持した。

既存のdefault-target/trial-targetを再利用し、C1 no-run欄と同じ2コマンドを実行した。通常exit0/build-finished.success=true/2m27s、試用exit0/build-finished.success=true/2m22s、コンパイルエラー0。warningは通常13・試用14件。通常では既存7件にincremental cacheのhard-link失敗→copyというcompiler warning6件が加わった。これはコンパイラが選択したcopyであり、実装担当がOS権限/設定を変更して解消したものではない。キャッシュ再利用による時間短縮は実証していない。

旧C1のbuild/runtime logとhash表は保存済みである。同一出力pathのEXEは今回のビルドで置換され、現在有効なのは下表である。C1解除をこれらに流用しない。JSON上は4本ともprofile.test=true、通常features=[]、試用features=[early-trial]である。

| 構成/target | C2 EXE絶対path | C2 SHA256 |
| --- | --- | --- |
| 通常/lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\default-target\debug\deps\wordweave5-b3388a7bbef44bb4.exe` | 742247DBA6FE919F23FA6F927BB153B46EC931B2C517FE88FE577B2455E3FC18 |
| 通常/bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\default-target\debug\deps\wordweave5-72e8adffa8f9ef2c.exe` | 94C2D7F3630F3F801D5E1B739D7997515A602AC97F00F2B3BC78706C30EA63F6 |
| 試用/lib | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\trial-target\debug\deps\wordweave5-1e58dd7001d62083.exe` | 16D61A43A2EC274D90D37A5E1D927FFCF06DAAB624C6AFD446C3DAE465F151E3 |
| 試用/bin | `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\trial-target\debug\deps\wordweave5-e0ab9f88db3adadd.exe` | F89BF3B3B00465B288699941A374D4A93CEDAD01547675FEC1C2B2A8C67EAAD6 |

| C2 log（target/early-trial-build/et2-impl配下） | SHA256 |
| --- | --- |
| c2-default-build.jsonl | 7B3BA95E100FEF5B214A886CD661DFD35CDF4D2FA85AFA8910366EB4163961BC |
| c2-default-build.stderr.log | 34FA55E84DFB532CAA93894083782AA609623770A6EF952CFA2F9F8510D03C7C |
| c2-trial-build.jsonl | 8ED4926A8F4F686C554331ECD9E5095B59C55ABBB749E3A55D746704191D1313 |
| c2-trial-build.stderr.log | 642C6DFDECACAD57287298D68AEF6EAECD1E92C262D801416FEBEB5BFFA5571F |

C2で`--list`/test/native/Recorderは未実行。実名予定集合は作者の18/7件のままであり、C2 EXEからの実在照合は未了である。新EXEの親解除までここで停止する。ET2-P01、C1試用#3是正の動的確認、後続未実行12件、独立runner/Post、感度、ET3は未完了である。`git diff --check` exit0。

同session最新snapshotは `2026-10-14T08:17:33.306Z`、input9382495/cached9183360/output28519/reasoning3479。この追記と最終応答は未反映、runtime返却model/effortと親子包含は未取得である。

## C2 限定runtime結果：是正#3と回復境界PASS、#11 fixture FAILで停止

親の「C2対応EXEの再解除」を読んでから、上表のC2固定4EXEだけを実行した。C1とは別の `target/early-trial-build/et2-impl/runtime-c2/` に全ログを保存した。実行補助はC1と同じSHAのscriptであり、毎起動前Administrator=False・固定SHA一致、Hidden、65秒timeout、wait、合成TEMP/LOCALAPPDATA、exact1件、必要なchild marker/残存検査を維持した。source/期待値/補助scriptの変更0である。

順序は4EXE list→通常#4/5/6→試用#4/5/6→試用#3→試用#7/8/9/10/11である。listの集合差分は0、通常bin6/lib1・試用bin17/lib1の実名を確認した。C2での実行済みcaseは下表の12件であり、C1の成功をC2全件成功へ流用していない。

| 構成 | PASS | FAIL | C2未実行 |
| --- | --- | --- | --- |
| 通常 | #4/5/6（3件） | 0 | #1/2/16/17（4件） |
| 試用 | #3/4/5/6/7/8/9/10（8件） | #11（1件） | #1/2/12/13/14/15/16/17/18（9件） |
| 合計 | 11件 | 1件 | 13件 |

安全3件はC2の両構成で再PASS。是正#3はmalformedの記録不正本文とvalid full-wire historyの元の履歴拒否本文、全tree不変、closureなし、diagnosticsなしを同case内で確認してPASSした。#7の実archive共有拒否/両file回復書込/同データretry、#8の回復成功後実read拒否/ledger維持、#9のraw error非依存、#10のhistory復元先頭拒否も固定assertを通った。

#11 `app::early_trial_startup_tests::et04_nonhistory_json_and_settings_survive_trial_reopen` は `src/app.rs:230:160` のfixture `serde_json::from_str::<Progress>(...).unwrap()` で `Error("missing field memories", line: 1, column: 93)` を返してFAILした（実logはfield名をbacktickで囲む）。次行の`app.restore`には未到達であり、製品restore動作のFAILとは扱わない。inner/outerともexit101、outerはmedia.rs:85でchild失敗を報告した。静的推論だけだったC1 #3と異なり、今回は不足fieldが実診断へ出ている。

最初の失敗で解除を停止し、#12以降および最終集合用の残り通常4件・試用#1/#2は実行しなかった。同種fixture欠落の再発として親へ具体箇所と作者fixture群の点検提案を返した。作者所有moduleや期待値を実装者が修正していない。runtime再試行0、source6hashはC2固定値と一致している。

全16起動（list4＋case12）はAdministrator=False、固定SHA一致、timeout=false、wait完了、fixture残存0である。isolated成功11caseはinner markerも確認し、失敗#11のchildも終了後cleanupされた。read-only列挙で`runtime-c2`以下の`ww5-et2-*` directory残存0を確認した。外側の空synthetic directory/logは保持した。未join workerは観測していない。

ログ命名はC1と同じで、`runtime-c2/<configuration>-case<ID>/` または `<configuration>-<bin|lib>-list/` にstdout.log/stderr.log/result.jsonを持つ。

| 失敗log | SHA256 |
| --- | --- |
| trial-case11/stdout.log | B42031798E7D93733CD0D6466570A9745566888138D475CE2FB2F0093E544447 |
| trial-case11/stderr.log | BCE123969545D5F6986BBB19432700A7BDB05ABB32237FC8C82C80495F62CE50 |
| trial-case11/result.json | 03ED260BEADBDA543FAD8C60EFD2C6CE97766DD51F1A6951E6FCF8B58AD98001 |

ここで実装担当のsource所有を返却して停止する。P01の動的送信観測、#11fixture是正、C2未実行集合、感度、独立runner/Post、ET3/nativeは未完了である。

同session最新snapshotは `2026-10-14T08:22:19.156Z`、input11522707/cached11308544/output34489/reasoning3992。この追記と最終応答は未反映、runtime返却model/effort・親子包含は未取得である。

## C3 試用bin no-run固定（runtime未解除）

作者による#11完全JSON fixture/expectとmain誤コメント訂正を親が再確認したC3を受領した。作者report SHAは `DE5289E551F6D1F638F7D75D84045367333D6A68B67E95C31D12254A22AD4833`。今回は親の限定指示により試用binだけをbuildした。通常/libの旧成功は元snapshotの証拠であり、C3の成功へ置換しない。最終両構成fresh検証/全体gateを削減していない。

```text
cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target
```

exit0、build-finished.success=true、1m52s、JSON warning11/error0。source編集0、`--list`/runtime0、`git diff --check` exit0。8sourceのbuild前後hashは次の値で完全一致した。追加2fileはP01設計snapshotとの照合対象であり、P01監視の実装許可を意味しない。

| source | C3前後SHA256 |
| --- | --- |
| src/main.rs | 0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40 |
| src/app.rs | 4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962 |
| src/qwen_reading.rs | 3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64 |
| src/app/qwen_settings.rs | D847555998E71987E8519D8B8B4811D68231054A043A365F3D040BA0339E11C8 |
| src/app/qwen_settings_tests.rs | 45EF01F0F3DB4A2302AE8DAF9FC10834190D427AB648823139B46DC49DE324F1 |
| src/media.rs | 14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207 |
| src/ai.rs | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| src/app/qwen_reading_ui.rs | 0463767500979C625724F065F54D4B1C7F525BC7688F7BD38FC40241CBFCB41D |

JSON compiler-artifactでtarget.kind=[bin]、profile.test=true、features=[early-trial]を確認した。EXE絶対pathは `F:\Kazuhiro\GitHub\WordWeave5\target\early-trial-build\et2-impl\trial-target\debug\deps\wordweave5-e0ab9f88db3adadd.exe`、SHA256は `3B211C36C9FB10DDE7D04CB01906CE19FA1FBD77AD11AFC5AA03815FB8E201E8`。C2の同pathは置換され、旧解除を流用していない。

ログは `target/early-trial-build/et2-impl/c3-trial-bin-build.jsonl`（SHA `9309090D53F8089BE40B509926C5E5466AFE82940A90EB4D1FF2BEE20574ADBD`）と `c3-trial-bin-build.stderr.log`（SHA `6A06172E68502922D6FEC5575A5C264BB6A8FFFCBC3922C40A7E3FF657786264`）。旧C1/C2記録は保持した。C3の親EXE再解除を待って停止する。#11是正と未実行caseのruntime/P01/感度/独立runner/Post/ET3は未了である。

同session末尾の最新snapshotは `2026-10-14T08:33:39.282Z`、input14030359/cached13800704/output37962/reasoning4278。この追記/最終応答は未反映、runtime model/effortと親子包含は未取得である。

## C3 限定runtime結果：試用bin新17件PASS

親の「C3試用binの限定再解除」を受領してから、上表の試用bin SHA `3B211C36C9FB10DDE7D04CB01906CE19FA1FBD77AD11AFC5AA03815FB8E201E8` だけを実行した。8sourceとEXEは実行前後ともC3固定値に一致し、source/期待値/実行補助scriptの変更0である。通常/libの再実行、通信P01正対照、感度、旧回帰、実データ/資格情報/API/マイク/nativeの操作は0である。

実行順はlist→#4/5/6→是正#11→#2/3/7/8/9/10/12/13/14/15/16/17/18。listの実名17件は作者bin集合と一致し、完了result.jsonの実名集合も同じ17件で差分0であった。**試用bin新17件は全件PASS、FAIL0、skip0**。各caseをexactで1件ずつ実行し、0件成功を除外した。#1はlibなので今回含めていない。

#11は完全な正常JSON fixtureを使用し、restore→設定保存→同root再open→loader1/D1・設定値保持・Home/history=None・通常root不変を元のassertで確認した。#12–15はbackup/教材/生成proposal/添付dropの承認前・取消し・承認後の動作を確認した。#18も現行の固定assertではPASSだが、既報ET2-P01の起動/設定save/restore送信0の直接観測不足はこの結果では解消しない。

全18起動（list1＋case17）はAdministrator=False、毎起動直前SHA一致、Hidden、timeout=false、wait完了、fixture残存0であった。isolated14caseは作者inner markerとchild1件成功も確認し、read-only列挙で`runtime-c3`下の`ww5-et2-*` directory残存0を確認した。workerを持つ安全対照はjoinまで固定assertを通った。timeout kill経路は発動していない。外側の空synthetic directoryとログは保持している。

時刻は17:35:56.9439551–17:36:55.7068452 JST。ログは `target/early-trial-build/et2-impl/runtime-c3/trial-bin-list/` と `runtime-c3/trial-case2/`～`trial-case18/` のstdout.log/stderr.log/result.json。C1/C2ログは上書きしていない。是正#11の証拠hashは次のとおりである。

| log | SHA256 |
| --- | --- |
| trial-case11/stdout.log | 967603831533272D6AD26DE5BF951788CBE18866D6B10BD9C0D4BE2B6F03D72A |
| trial-case11/stderr.log | 93AD152D88DB7D267147EF2B79A2C3C478975001D4FCF03FC192A072C3EB90D4 |
| trial-case11/result.json | 64E95713CC2F03FE250FD6C56944C98A9779805B4E4A5D25646FADA921E5485B |

今回の返却境界はC3試用bin新集合のfocused確認までである。通常/libの成功はC1/C2の各snapshot範囲のまま保持する。P01、感度確認、最終両構成fresh/独立runner/Post、ET3/native受入は未完了であり、ET2全体の受入完了とはしない。`git diff --check` exit0、所有返却して停止する。

同session最新snapshotは `2026-10-14T08:37:16.029Z`、input16047921/cached15808384/output42474/reasoning4645。この追記/最終応答は未反映、runtime model/effortと親子包含は未取得である。
