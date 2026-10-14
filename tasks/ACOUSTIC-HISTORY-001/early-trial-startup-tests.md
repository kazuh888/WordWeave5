# ET2 独立テスト作者の固定 oracle

2026-10-14。作者 `/root/trial_startup_tests`、thread `01a1394d-02e8-7493-84ae-371e9ade04bc`。planner指定 `gpt-6-astra/high`、provider/runtime返却値は未取得である。AGENTS、development-team、KPI、test-driven-development、wordweave-changeを確認し、テスト/RED工程だけを担当した。実装者へ本体所有を返却する。新試験の受入PASS、ET2-G0解除、ET2完了を主張しない。

入力は契約 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、限定設計 `24B9FEBE6F83528EF1B4D55EF9890B00FC9846E9F50B5BB446F48A2FAE687D7E`、計画 `82E0A3E8A326EBEB2456ACC994731895072F3B714286452598435547ADC31AFD` と同snapshotのPre-2 PASSである。実source基準は97d2c72、docs checkpointは親申告abad297である。ET1/FX1 Closedは再レビューしない。

## 所有・固定署名

変更は許可6sourceに追加した専用cfg(test) module/fixtureのみ、合計900行追加・既存行削除0である。既存shared harness、通常constructor、cfg(test) default_store、production behavior、sourceの公開範囲は変更していない。既存fixtureは読取再利用した。親のtasks/current.md差分を保持した。再委譲/commit/pushは0である。

以下は実装者が所有する不足接合である。作者はstub、偽factory、仮公開APIを設けていない。

| 所有file | 固定接合 |
| --- | --- |
| main.rs | feature別private `WINDOW_TITLE: &str`。`early_trial_creator(Result<wordweave5::store::Storage,String>) -> Result<eframe::AppCreator<'static>, app::TrialStartupFailure>`。binのStorageは `wordweave5::store` であり `crate::store` ではない |
| app.rs | 設計どおり `WordApp::new_with_storage_and_loader<L>`、`new_early_trial`、private `new_early_trial_with<L,D>`。後者のLは `FnOnce(&Storage)->Result<Progress,String>`、Dは `FnOnce(&Path)`。独立試験は実factoryと実Storageを使う |
| app.rs | `TrialStartupFailure::{StorageOpenRejected { safe_message:String }, RecoveryOrInitializationStarted}`。Displayの返値は安全な本文そのもの。mainのdialog title/共通追記はproduction接合の読取とET3 nativeで検査する |
| qwen_settings.rs | private `configured_store_with<F,T>(factory:F)->T where F:FnOnce(&'static str)->T`。private子moduleから検査し、兄弟module向けに公開しない |
| media.rs | 設計の `watch_recording_starts()->Result<RecordingStartWatch,&'static str>`、`RecordingStartWatch::calls()->usize`、RAII Drop。作者がpoison検査に使うprivate名は `RECORDING_START_WATCH: Mutex<Option<Arc<AtomicUsize>>>` |
| media.rs | active時の固定Errは `ET2: recording start intercepted`、poison時は `ET2: recording watch poisoned`。重複登録はErr（本文の固定assertなし）。watch/startは同一globalを共有し、判定・加算・Errはcpalより前であることを親がET2-G0で確認する |

試用boolの名前はテストで参照しない。成功factory経由でだけ試用動作へ入り、bool/fatalの直接操作で結果を作らない。定数と表示の期待値は正本literal、回復fixtureのold/next JSONとTSVは独立literalである。

## 実在test名・件数・受入

filter `early_trial_startup_tests` の新規集合は **試用bin17+lib1=18件、通常bin6+lib1=7件** である。下表のdefault欄○だけが通常構成にも存在する。runnerはbuildしたEXEの `--list` と照合し、0件成功を除外する。全新caseは現時点でlib定数以外未実行である。

| # | 実名 | 受入・目的assert | default |
| --- | --- | --- | --- |
| 1 | `qwen_reading::early_trial_startup_tests::et02_credential_target_matches_independent_feature_literal` | ET-02 feature選択の独立literal。試用のactual RED取得 | ○ |
| 2 | `early_trial_startup_tests::et03_title_matches_independent_feature_literal` | ET-03 通常/試用title literal。mainがこの値を消費するかはPostの接合確認も必要 | ○ |
| 3 | `early_trial_startup_tests::et_d01_creator_preserves_open_rejection_without_creating_closure` | ET-D01 実history rootの書込前open拒否、creator closureなし、全tree不変、Dなし。open後安全文は書込前文へ分類し直さない | — |
| 4 | `media::early_trial_startup_tests::et2s_active_watch_catches_entry_worker_and_rejects_duplicate` | ET2-S 実入口1回Err/calls1、重複拒否、worker1回/join/calls2、正常Drop再登録0 | ○ |
| 5 | `media::early_trial_startup_tests::et2s_unwind_releases_only_owned_watch` | ET2-S panic捕捉後RAII解除、再登録・実入口1回捕捉 | ○ |
| 6 | `media::early_trial_startup_tests::et2s_poison_is_fail_closed_and_drop_does_not_clear_poison` | ET2-S lock poison時Err、登録拒否、Drop後もpoison維持・自登録のみ解放、cpal未到達はG0と接合 | ○ |
| 7 | `app::early_trial_startup_tests::et_d01_real_archive_denial_after_both_writes_then_same_data_retry` | ET-D01 実pending回復deck/progress次bytes到達、archive拒否、pending元bytes、D0、AppErr、解除同データretry成功/D1 | — |
| 8 | `app::early_trial_startup_tests::et_d01_recovered_ledger_survives_actual_constructor_read_denial` | ET-D01 loaderの実回復成功→実custom.tsv read拒否→AppErr/D0、回復ledger維持、解除同データconstructor成功/D1 | — |
| 9 | `app::early_trial_startup_tests::et_d01_raw_loader_error_cannot_change_failure_stage_or_safe_text` | ET-D01 秘密風/書込前文風/空の3raw Stringでも同じ型/定型文、L1/D0。不変を断言しない | — |
| 10 | `app::early_trial_startup_tests::et04_history_json_rejected_before_backups_drafts_epoch_or_disk_change` | ET-04 exact拒否理由、Progress/日本語・会話・教材draft/epoch/message/dirty/全tree不変・退避増加0 | — |
| 11 | `app::early_trial_startup_tests::et04_nonhistory_json_and_settings_survive_trial_reopen` | ET-01/04 非履歴JSON適用、設定保存、同root再open、L1/D1と診断path、Home、disk/UI head None | — |
| 12 | `app::early_trial_startup_tests::et04_backup_confirmation_cancel_and_approved_restore_keep_history_none` | ET-04 実backup export/prepareと既存確認UI、承認前/cancel無適用、承認後復元/diskUI None。prepared.progress編集0 | — |
| 13 | `app::early_trial_startup_tests::et04_material_import_cancel_then_approval_and_word_cache_keep_history_none` | ET-04 TSV確認UI→cancel無適用→承認後実import、実cache_words、保存/loadでNone | — |
| 14 | `app::early_trial_startup_tests::et04_generated_proposal_requires_visible_approval_and_keeps_history_none` | ET-04 既存合成Draftを実material UIで表示、承認前教材不変、破棄で登録0、明示クリック後内容/source反映・None | — |
| 15 | `app::early_trial_startup_tests::et04_explicit_attachment_and_drop_cancel_approval_keep_history_none` | ET-Q01/04 明示通常root合成入力→attach、実drop→確認/cancel無保存→承認後添付、元bytes/未送信draft保持・None | — |
| 16 | `app::qwen_settings::early_trial_startup_tests::et02_production_selector_constructs_exact_windows_target_without_credential_io` | ET-02 同じprivate selector→実for_target構築/targetのみ、通常/試用/toolの3literal | ○ |
| 17 | `app::qwen_settings::early_trial_startup_tests::et02_selector_editor_controller_use_only_selected_namespace_and_no_send` | ET-02 namespace別spy、通常/toolだけ設定→試用未設定、save/reopen/controller read-save先、宛先変更空key拒否、明示key成功、他namespace read/save0、send/probe0 | ○ |
| 18 | `app::qwen_settings_tests::early_trial_startup_tests::et06_factory_settings_restore_and_reading_prepare_do_not_record_probe_or_send` | ET-06 watch→factory→設定表示/save/restore→既存QwenDialog/prepare/cancel/close→calls0、接続spy送信/probe0、pending/connection_check/recorderなし、CODEX_HOME値不変 | — |

既存回帰は `app::qwen_settings_tests`、`app::qwen_reading_tests`、`app::i2_history_main_consumer`（通常constructor下のrestore含む）、`app::backup_ui::tests`、媒体録音buffer/終了、fatal-close、`store::early_trial_paths_tests`、`commit::early_trial_paths_tests`、`tests/audio_history_backup.rs` を再利用する。history backup拒否のoracleを新fixtureで偽造しない。実行集合の詳細選択と各既存filter件数はrunnerが実EXE一覧で固定する。既存全体/release/関連crate/ET3 gateは削減しない。

## fixture・段階の識別

新共有fixtureはmedia.rsの専用cfg(test)内だけにある。実名1caseの子process、独立LOCALAPPDATA/TEMP/TMP/cwd、CREATE_NO_WINDOW、45秒deadline、timeout時kill→wait、child marker/1件成功、Administrator=Falseの実assert、終了後一意合成tree cleanupと残存検査を行う。実データ、実資格情報load/save、API、マイク、製品EXEを使用しない。子が異常終了した場合はstage/child失敗であり行動PASSにしない。fixture作成/cleanup失敗も隠さない。

ledgerはfile bytes、directory集合、directory属性を比較し、reparse点をfixture異常とする。fs2 lock中のapp.lockだけbytesが読めないため長さを記録する。open後の回復境界snapshotには既存openのdir/app.lock副作用を含める。通常rootに教材/Progress/音声/ink/journal/診断の合成sentinelを置く。

archive拒否は実pendingをread/share_mode(3)で保持する。事前renameがOS32となりreadは成功することを確認し、実factory後にdeck/progress両方が独立next bytesへ達したことを必須assertする。pending元bytesと空backups以外の差分を許さない。解除後同一dataの再open/loadでarchive内容/個数と成功D1を検査する。既存AfterDeck faultの実証とは別のarchive障害である。

初期化拒否はloader内で実Storage::load成功とpending消滅/archive元bytes/next2fileを先にassertし、そのledger保存後にcustom.tsvをshare_mode(0)で保持する。実readのOS32を確認してProgressを返す。constructor後にAppErr/D0、handle解除後ledger不変を検査し、解除constructor成功対照を置く。fake Errや直接fatal代入は使わない。OS共有拒否/次bytes到達不成立はfixture/stage FAILでありskip成功にしない。

3raw Stringを返すloaderだけは安全表示分類の試験用stubである。これを実回復・実IO障害の証拠として数えない。

## 実行した検証・未実行

ログはignored `target/early-trial-build/et2-author/` に保存した。

1. `cargo test --locked --lib --features early-trial qwen_reading::early_trial_startup_tests::et02_credential_target_matches_independent_feature_literal -- --exact --nocapture`：compile成功、1実行/1目的assert FAIL、exit101。左が通常target、右が正本試用targetである。`target-red.log`。初回EXE hashは351D3B69C6B7E468BE8FD6A189200FB8FAB71C83BAD3AD8A5AFC739C19D69669であったが後続no-run buildで同pathが置換された。
2. `cargo test --locked --bin wordweave5 --lib --features early-trial --no-run --message-format=json`：初回は不足API28件と作者の既存Wire.count誤参照2件、exit101。`compile-blocker.jsonl`。後者を既存count()へ修正した。
3. 同no-runの修正確認：不足API28件のみ、E0425=17/E0432=3/E0433=1/E0599=7、exit101。`compile-blocker-final.jsonl`。E0433は不足TrialStartupFailure型でありcrate::store誤参照ではない。production接合前なのでコンパイル成功、試用binの実行/行動REDを主張しない。通常binも未build/未実行である。
4. no-run後に置換されたpure-lib EXEで同じ定数exact caseだけを再実行し、1実行/1目的assert FAIL・exit101を確認した。これは最終EXE識別との接合のためである。`target-red-final.log`。EXEを `target/early-trial-build/et2-author/red-lib-8676593D.exe` に保持した。SHA256=`8676593D8CBE6DFE34C249F29E6C9362A736A345063E48D1D2959675DCF960D5`。元pathは `F:\Kazuhiro\GitHub\WordWeave5\target\debug\deps\wordweave5-1e58dd7001d62083.exe`。pure定数比較だけであり、合成root/OS障害/native/Recorderの証拠ではない。このpure-lib実行自体はcargo/terminalで行い、新bin隔離fixtureのAdmin/TEMP/Hidden/timeout検証はまだ実行していない。
5. 追加moduleをrustfmtで整形し `cargo fmt --check` exit0（`fmt-final.log`）、`git diff --check` exit0。全6sourceの差分は追加のみである。bin runtime、Recorder runtime、実native/実credential/API/マイク試験は0。

未実装APIの存在検索/コンパイル失敗と、動作の失敗を区別した。必要なAPIが実装されるまで18caseの全コンパイル/実行は確認できない。不足APIは後続の型検査を抑止し得るため、追加fixture全体のコンパイル健全性も未確認である。実装者は作者期待値を修正せず、不足/fixture不具合を作者へ返す。

## 実装者・runnerへの順序

1. 上記不足接合を実装者が本体所有内で実装する。6sourceと作者moduleを固定し、設計ET2-G0の親source確認を受ける。
2. 通常/試用を別target-dirで `cargo test --locked --bin wordweave5 --lib --no-run --message-format=json`（試用は `--features early-trial`）する。JSONのbin/lib EXE path/hash、前後source hashを固定する。
3. 親が固定source/EXE/構成へのET2-G0解除を記録するまで、新bin caseは一件も実行しない。解除後に#4→#5→#6の安全対照、続いて#3/#7–18へ進む。filterだけで安全対照と機能caseを無順序一括開始しない。
4. `EXE exact_name --exact --nocapture --test-threads=1` を用い、上表の実名と期待件数を照合する。子process/workerの終了とfixture残存を記録する。失敗/timeoutなら解除を停止する。
5. 新全caseと関連回帰の両構成結果を独立Postへ渡す。source変更/感度差分/逆復元後は再Gateする。EXE提供・native受入はET3である。

感度対応は(a)誤namespace/title→#1/#2/#16/#17、(b)restore拒否欠落→#10のexact理由と全ledger、(c)fatalApp公開/D順→#7/#8/#9、(d)目的回復段階未到達→#7 next2file、#8 loader成功/OS32/read後ledger、(e)誤0→#4の実入口後calls1/worker後calls2と#5である。(a)の定数namespaceだけはactual RED済み、他の変異実験は未実施。作者は本体を変異しない。録音変異は実装者によるcalls()返値のみで、cpal前遮断/active加算を維持する。

## 観測限界

ET06の録音watchはfactory前からclose/音声job registry drain後まで保持する。Qwen send/probe spyはfactory後の設定・復元周辺と実QwenDialog constructor/prepare/cancelへ接合する。factoryはtransport注入引数を持たず、起動中API送信0を動的に直接観測していない。pending/connection_checkなしやCODEX_HOME値不変から共有認証ファイル不変更を全面推定しない。起動経路・production default_storeのselector使用・title実使用・main通知の静的接合確認がPostに残る。親へこの限界を返し、許可外global通信seamは追加しないとの回答を受領した。

対象操作は評価/probe/audio-load workerを開始しない設計である。QwenDialogはcloseしaudio job registryをdrainするが、存在しないworkerをjoin済みと表現しない。media正対照のworkerは明示joinする。将来実行で新workerが発生する場合はwatch解除前に停止し、完了を観測できる所有へ返す。実音声、Codex/Qwen認証/API、native title/拒否dialog、DPI/IMEの成功は未検証である。

## 最終固定hash

module hashは最初の追加cfg属性からmodule閉じ括弧までをUTF-8化し、末尾区切り空白だけTrimEndしてSHA256した。実装者はmodule本文を変更しない。

| source | source SHA256 | 作者module SHA256 |
| --- | --- | --- |
| src/main.rs | EFD8D6F143CA49FD2E516002ABA6EE85F6F58065C5F0D4B466BAC06632373BE2 | 73F14DD72E8AE1A0A1B6F42E5C508A54C3C911FE65BC28F73F06581F9316EFD4 |
| src/app.rs | 4ADFF925155D2834ECA22C619EADF6748639EF6C5B4502634F7DA2C4B19DD013 | 3877EFEAEDBBC7A8BB88A1987070313D7B59AFB632D2DDB1EBF97D6B7C5DC4AA |
| src/qwen_reading.rs | E8E30C24BBEAC6F093939C18E087CC263DD4E71854EAB6F036ECA0ACE5FF8C58 | FAE2E01F3926D3AAC196DCCEA603BB7D574711C2FBFDB2402617256A09EC76A0 |
| src/app/qwen_settings.rs | 757D9DE74F1779F6E1A28239DA089D4EB89D42925A4B96AB04946712AD89D953 | A9881AF09824509E98202B9E673B0E22FC724653FC4AF4255962EB5F970034B6 |
| src/app/qwen_settings_tests.rs | 45EF01F0F3DB4A2302AE8DAF9FC10834190D427AB648823139B46DC49DE324F1 | C6550481C8B381BD44EF0419F9437D4046F7C06BA4A1C189C2EECAD2A10281C0 |
| src/media.rs | 775F1B67CFA6DF7186ED12866C8080F787CB4FE1D2B046F849DC8050CF45A0B6 | DCD6637D4AB50C170E7B6AD22B49F9CC99E0352A499D9B5F37AD1E30A9352FAA |

usageは対象session token_count取得可。最初の観測07:25:24.180Zはinput38639/cached23040/output223/reasoning10、途中07:39:49.553Zはinput4107026/cached3974272/output22534/reasoning3844である。開始前と最終応答は含まず、親子包含不明、週枠は親測定である。終了付近snapshotは親handoffに返す。runtime model/effortを指定値で埋めず、未取得とする。
