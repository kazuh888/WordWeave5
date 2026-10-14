# ET2-P01-V 実装引継ぎ：G0 source固定

2026-10-14。担当 `ww-implementer / gpt-6-astra/high`（planner指定）、thread `01a139a4-14c3-7402-834c-3e219e5feb55` / `/root/trial_egress_impl`。provider返却model/effortは未取得。AGENTS、development-team、KPI、wordweave-change全文を確認した。表示・操作UIの変更なしにつきegui-uiは非該当である。

今回の完了境界は、承認済みtest専用接合を実装し、作者領域と8sourceを固定して親のET2-G0読取へ返すことに限定する。build/no-run/list/runtime/感度/独立runner/Post/最終受入は未了である。旧C3録音限定解除を通信へ流用しない。

## 入力と所有

- HEAD `fef8bd27dc1fbf5065f568d0d1d7e538f376bbb3`＋作者4file差分。
- 設計 `early-trial-startup-design.md` SHA256 `8AC25D82C1286ED1C4500BC3DFCF232E76DC01673E35758833C7E373C96F82F3` のP01-Vと単一ET2-G0を確認。
- 計画 `early-trial-plan.md` SHA256 `BCD69DBC7B1EA022598374A93D967DE5DA8239427FF5443C3D55F5E2F5E3A785` のP01追補と割当、`early-trial-startup-review.md` のP01設計Pre PASSを確認。
- 作者引継ぎ全文 `early-trial-egress-tests.md` SHA256 `3C1E4041AF0BD1542762DBC0FDC4C9E3AC42DF638B15551EA89F2EB6FB907E65` を確認。予定通常bin20／試用bin20、Entry順序、固定Err literal、private名と署名、全独立期待値を保持した。
- 本担当変更は `src/ai.rs`、`src/app/qwen_reading_ui.rs`、`src/app/qwen_settings.rs` の担当接合と本報告のみ。作者専用 `src/app/qwen_settings_tests.rs` は全bytes不変。並行した親のcurrent/usage編集を保持した。

## 接合と静的確認

| 箇所 | 実装と確認した境界 |
| --- | --- |
| ai.rs:4 `trial_egress` | bin側単一private `GLOBAL: Mutex<Option<Arc<State>>>`。8 Entry固定順、State.callsはAtomicUsize8個、合成Store Arc。正常Noneのみ未監視、登録重複拒否、poisonは拒否、snapshotのpoison/別ownerはErr。登録/参照時にStore.load/saveを呼ばない |
| ai.rs:121 `Watch::drop` | Arc同一性が一致する登録だけ解除。poison guardを回収してもclear_poisonしない |
| ai.rs:140 `Lease::reject` | lock下で旧StateのcounterをSeqCst加算。登録一致はObserved、別owner/終了はEnded、poisonも旧counterへ加算してPoisoned。新ownerへ混入せず、全分岐が拒否を返す |
| ai.rs:607/618/646/686/716/730 | checkと5generateの実lib呼出直前で同globalの加算/固定Errを返す。Materialのみ明示 `MaterialFailure::from(String)`。transcribeサイズ上限判定・既存base64処理は先行する。非testにはblockがない |
| qwen_reading_ui.rs:192/218 | capture=Someだけ同Leaseの拒否専用WatchedTransportへ既存innerを保持。Noneは元Arc、ErrはInvalidState/NotSent。sendの同期部分で加算、futureもLease保持、inner.send呼出は存在せずNetwork合成Err。GatedTransportの既存started=trueによるMayHaveBeenSent表示を変更しない |
| qwen_reading_ui.rs:375 | cfg(test) finish_egress_testはruntime.take→既存close→Runtime同期drop。通常close/Dropのshutdown_background契約は不変 |
| qwen_settings.rs:296 | cfg(test) default_storeがconstructor前から登録済み同Store Arcを選択。正常Noneは従来Store::new(None)、poisonは固定fixture panic。非test selector不変 |
| qwen_settings.rs:445 | start_probe先頭で同globalの加算/拒否→InvalidState/NotSent。refresh/invalidate/credential解決/Reqwest構築/thread生成前に戻り、editor状態更新なし |

global lockをawaitやCredentialStore/Transport trait呼出へ跨がせない。main.rsのai/app/mediaはbin moduleであり、app constructorのdefault_storeと実open_qwen_readingのReqwest→QwenDialog::newが今回の同一observerへ接続する。対象外のlib、main/app/qwen_reading/media、harness、依存は編集していない。

現物型は `material.rs` のFrom<String>、qwen-audioのTransportFuture/TransportFailure/CredentialStore、既存GatedTransportに照合した。local registryのTokio 1.53.1 Runtime::dropとBlockingPool::drop（shutdown(None)）も確認した。これは静的型/寿命確認であり、compile成立やworker終了の実測ではない。

## 固定source（通常/early-trial共通）

| source | SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `A7A202579ABEE7C6F1B56DCC754101BDCAE73A4ABF715686006503534FDBFDF7` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

作者marker hashは作者と同じUTF-8部分文字列規則でfmt前後一致を確認した。

| 作者領域 | SHA256 |
| --- | --- |
| ai.rs | `A95972760840E1A3925A0F296B60A28C7EED008BE4A943FB7FAC7E0FF86A65B5` |
| qwen_reading_ui.rs | `283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3` |
| qwen_settings.rs | `79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3` |
| qwen_settings_tests.rs | `3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6` |

## 検査・次の境界

`rustfmt --check --edition 2021 --config skip_children=true`（上記作者4file）と `git diff --check` はexit0。source読み取り、apply_patch、rustfmtのみであり、cargo/test EXE/list/実API/認証/マイク/native/Admin/UAC/OS変更は0。実行した各ツールcommandはexit済みで、本担当が起動した継続processはない。録音/通信の動的0やworker終了を測定済みとはしない。

ここで本体接合を停止する。親は上記固定sourceのG0読取後にのみ通常／試用のbin no-runを依頼する。既定targetは `target/early-trial-build/et2-impl/{default,trial}-target`、次のログは `target/early-trial-build/et2-egress-impl/`。build前後8source、JSON由来profile.test/features/EXE絶対path・SHAと作者予定20実名を固定し、親の新解除前にはlistも実行しない。後続型診断が作者に属する場合は期待値を変更せず返却する。

新たな設計欠落・追加所有要求は静的確認では検出していない。未達はcompile、各構成の安全対照とmatrix、感度、独立runner/Post、ET3/native及び利用者受入である。ET2-P01 Closedは未主張である。wordweave-changeは変更を担当接合に限定し、静的根拠と未取得の動的証拠を分けるために用いた。

## 使用量観測

同session開始側snapshotは2026-10-14T09:01:03.430Z、input205794/cached163840/output650/reasoning16（着手直後まで欠測）。中間snapshotは09:03:55.046Z、input982103/cached893184/output6096/reasoning489。最終応答前の終了snapshotはhandoffへ添付する。親子包含は未確認で合算せず、週枠/current/usageは親所有である。provider実行metadataを指定Astra/highで埋めない。

## V1 no-run：通常／試用bin固定、runtime未解除

親の [V1 source確認](early-trial-egress-review.md) とno-run限定依頼を受領後、次の2構成を逐次実行した。両方exit0、JSONのbuild-finished.success=true、compiler error0である。作者の不足API71診断は今回出ておらず、今回のsourceで後続型検査を含むbin compileが成立した。動作GREENは未主張である。

```text
cargo test --locked --bin wordweave5 --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/default-target
cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target
```

ログrootは `target/early-trial-build/et2-egress-impl/`。通常 `v1-default-build.jsonl` / `v1-default-build.stderr.log`、試用 `v1-trial-build.jsonl` / `v1-trial-build.stderr.log` を保持した。warningは通常10／試用11。両構成5件ずつのincremental cache hard-link失敗→copy警告と、既存箇所のunused/f32 fallback警告である。環境・警告対象sourceは変更していない。診断表示用PowerShellのJsonSerializer overload呼出1回は失敗したが、CargoとJSON読取・SHA取得の成功とは別であり、featuresは元JSONの配列で確認する。

両binのJSON profileは共通で `opt_level="0", debuginfo=2, debug_assertions=true, overflow_checks=true, test=true`。以下はcompiler-artifactのbin executableであり、lib EXEで代替していない。pathはJSONのseparator表記だけを `/` に統一した。

| 構成 / JSON features | EXE絶対path | SHA256 |
| --- | --- | --- |
| 通常 / `[]` | `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/default-target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `4EA632D9CF0B8F2A818D82ECC4E64E5645A597177E8A7FBA1E7BA1018A891B11` |
| 試用 / `["early-trial"]` | `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `44B5D126EE01C879FEE2E7ABD755922738A8B2571D2550888ED8287C15128959` |

build前後の全8sourceは上記V1表と一致し、build後の作者4markerも同表と一致した。今回source編集0。2つのCargo command sessionはexit0で終了し、継続起動processはない。EXE起動/listは0である。

次の集合は作者の固定表から抽出した**予定**通常20／試用20実名であり、まだEXE listで確認していない。

```text
ai::trial_egress::tests::etp01_codex_check_blocks_before_lib
ai::trial_egress::tests::etp01_codex_chat_blocks_before_lib
ai::trial_egress::tests::etp01_codex_material_blocks_before_lib
ai::trial_egress::tests::etp01_codex_generate_blocks_before_lib
ai::trial_egress::tests::etp01_codex_read_ink_blocks_before_lib
ai::trial_egress::tests::etp01_codex_transcribe_blocks_before_lib
ai::trial_egress::tests::etp01_transcribe_limit_precedes_observer
ai::trial_egress::tests::etp01_duplicate_thread_join_and_drop_reregister
ai::trial_egress::tests::etp01_unwind_drop_releases_registration
ai::trial_egress::tests::etp01_ended_lease_rejects_and_never_counts_into_new_owner
ai::trial_egress::tests::etp01_poison_is_fail_closed_and_drop_keeps_poison
app::qwen_reading_ui::trial_egress_tests::etp01_qwen_inner_send_counts_once_and_never_reaches_wire
app::qwen_reading_ui::trial_egress_tests::etp01_retained_dialog_after_watch_drop_still_blocks_inner_send
app::qwen_reading_ui::trial_egress_tests::etp01_panic_cleanup_waits_for_runtime_before_watch_release
app::qwen_settings::trial_egress_tests::etp01_probe_rejects_before_inputs_resolution_or_worker_creation
app::qwen_settings::trial_egress_tests::etp01_default_store_selects_registered_arc_without_io
app::qwen_settings_tests::trial_egress_tests::etp01_unconfigured_startup_five_operations_remain_silent
app::qwen_settings_tests::trial_egress_tests::etp01_configured_startup_five_operations_remain_silent
app::qwen_settings_tests::trial_egress_tests::etp01_unconfigured_real_dialog_prepare_is_not_sent
app::qwen_settings_tests::trial_egress_tests::etp01_actual_app_dialog_send_is_observed_before_real_transport
```

この固定組を親へ返却し停止する。親の新EXE解除後にのみlist照合→既定録音安全対照→通信入口/寿命対照→matrixへ進む。安全/matrix/感度/独立runner/Post/ET3/利用者受入は未了である。

同session継続工程の取得開始側は09:10:40.214Z input2029240/cached1929728/output10541/reasoning841（no-run着手から同時点まで欠測）。09:15:21.331Zの観測はinput3726552/cached3620224/output12451/reasoning1155。終了snapshotはhandoffへ添付し、工程単独総量としない。

## V1 focused自己検証：通常／試用46case PASS

親のV1 EXE限定解除後に、上記と同じ2EXEでlist照合→各構成の録音3対照→P01のA01–A06/B01/C01→A08–A11/B02–B03/C02→A07/D04→D01–D03を実行した。通常・試用とも作者20実名がlistと完全一致し、**P01各20PASS、録音各3PASS、計46case PASS**。2listを合わせた48起動全てでexit0、Administrator=False、SHA一致、timeout=false、Waited=true、FixtureResiduals=[]であった。各caseはexact1件と作者child完了markerを確認した。0件成功、SKIP、未終了workerによる成功扱いは0である。

実行前の各8source確認と実行後の全8source/両EXE再hashはV1表と一致した。source編集0。全command session終了済みで本担当の継続processなし。独立runner/最終Postを自己検証で代替せず、ET3へ進んでいない。

証拠は `target/early-trial-build/et2-egress-impl/v1-runtime/` の `default-list` / `trial-list`、`{default,trial}-recording{1,2,3}`、`{default,trial}-{A01…A11,B01…B03,C01…C02,D01…D04}` 各ディレクトリのresult.json/stdout.log/stderr.logである。外側の合成local/temp/cwdは証拠として保持し、作者fixtureはchild wait後に削除済みで残存0を確認した。実学習データ削除なし。

実行ラッパーは同ログrootの `Invoke-V1Case.ps1`（SHA `DF1DD08A4CB3338C92C354C8F92BFED0512D20016B86430FCE94DFBCE83B6B35`）。既存Invoke-C1Case.ps1を基に、一覧filterをtrial_egressへ限定し説明のC1をV1へ変更しただけである。既存ファイル/shared harnessは変更していない。PowerShell automationと必要referenceを読了し、既存のps7/7.4以上、param/StrictMode/Stop、Admin/SHA、Hidden、65秒外側timeout、kill+wait、合成環境、exact1件、child marker、残存確認を維持した。PowerShell parser/Get-Help成功。今回専用の単一スクリプトと引数に限定し、汎用module/設定機構は追加していない。既知sandbox OS5を再試行せず、許可されたrequire_escalatedの通常ユーザーで実行した。

A01–A06のlib到達0とD04の実Reqwest到達0は、固有合成Err/counterと親の静的遮断確認を組み合わせた根拠である。未設置のlib/実Reqwest counterを測定したとはしない。B群Wire0/C01Probe0と各operationの累積0、同期Runtime終了対照は今回動的に確認した。既存MayHaveBeenSent表示を実通信ありと解釈しない。実key/API/マイク/nativeは実行していない。

## 感度計画（V2修正を親受領・未実装・実行禁止）

計画265–267/355–356と作者P01感度節に対応し、既定3snapshotにV2のA09を戻す修正を親が受領した。現在のsourceは固定したままとし、独立runner終了後に親が所有・変異snapshot・G0を受領するまで変更しない。以下main/app/mediaは既定ET2の6fileに含まれるが、本担当の現在のP01接合所有には含まれないため、次工程の明示割当が必要である。Qwen namespaceの取得済み同目的REDは反復しない。

| 目的 | 最小の候補変更位置 | exact case / 目的assert |
| --- | --- | --- |
| a title | main.rs:60/62 WINDOW_TITLEの対象構成literalだけを一時誤値へ | `early_trial_startup_tests::et03_title_matches_independent_feature_literal` / 独立feature literalとのassert_eq |
| b history restore拒否欠落 | app.rs:2793のearly_trial/history早期拒否条件だけを一時falseへ。後続保存本体は保持 | `app::early_trial_startup_tests::et04_history_json_rejected_before_backups_drafts_epoch_or_disk_change` / restoreのErr exact RESTORE_ERROR。後続verify_referencesの別Errもこのassertが拒む |
| c fatal App公開 | app.rs:814 fatal分岐のErrをその場のOk(app)へ一時変更、D呼出はしない | `app::early_trial_startup_tests::et_d01_raw_loader_error_cannot_change_failure_stage_or_safe_text` / recovery_errorの `fatal app must not be exposed` |
| c fatalでD開始 | 同fatal分岐で既存start_diagnostics closureを一度呼び、その後元のErrを返す。合成caseのclosureだけを対象に実行 | 同raw_loader_error case / `(loads,diagnostics)==(1,0)`。App公開変異と同時にしない（先行panicでD検査が隠れる） |
| d 目的段階未到達 | app.rs:927 `storage.as_ref().map(loader)` を同位置の合成Err返却へ一時変更しloaderを呼ばない。fixture・実store・期待値は無変更 | `app::early_trial_startup_tests::et_d01_real_archive_denial_after_both_writes_then_same_data_retry` / `FIXTURE_STAGE: actual next deck write must be reached`。別case `app::early_trial_startup_tests::et_d01_recovered_ledger_survives_actual_constructor_read_denial` / `(loads,diagnostics,denied)==(1,0,true)`。早期失敗検出のREDであり本来の回復段階PASSではない |
| e 録音誤0 | media.rs:215 RecordingStartWatch.callsの返値だけ0へ。start遮断・実fetch_add・poison/Dropは保持 | `media::early_trial_startup_tests::et2s_active_watch_catches_entry_worker_and_rejects_duplicate` / 初回入口後の1。初期0は維持され目的assertへ到達する |
| P01誤0 | ai.rs:134 Snapshot.calls返値だけ0へ。実counter/拒否/登録/Leaseは保持 | 上記固定実名表のA01–A06/A07/A08/A09/B01/B03/C01/D04の1 oracleを各exactで確認。A07は上限ちょうどのTranscribe1、D04は実app Send後の1。V2 A09のexact `ai::trial_egress::tests::etp01_unwind_drop_releases_registration` はcounter期待1のassert FAILと、専用IntentionalOwnerUnwindではないpayloadのresume_unwindによるFAIL伝播を取得する |
| P01誤非0 | 同Snapshot.callsで実値0だけ1へ返す。実値/拒否は保持 | A10/B02の新owner全0、D01/D02/D03の累積全0。D群は最初の `before constructor` で止まるため、この一変異だけで後続各stageのassert到達を実証したとはしない |

非隠蔽組合せ案は、S1=(a,b,c-App,e,P01誤0)、S2=(c-D,P01誤非0)、S3=(d早期失敗)の3snapshotである。各snapshotでは表の目的exactだけを実行し、c-App/c-D、誤0/誤非0、d/他factory依存caseを同時にしない。S1のbが影響するD01/D02はS1では選ばない。S3は他変異を復元して2つの段階caseを個別確認する。変異毎と完全逆復元後に全8source/作者marker/EXEを固定して新G0へ戻す。目的FAIL以外の失敗・未終了は停止し、予想された目的FAIL後の次case許可も親の限定解除へ明記する必要がある。遮断撤去・実加算変更・作者oracle変更は全案で0である。

**V1で検出しV2で静的是正した隙間**：V1 A09はcount=1のassertを意図したpanicと同じcatch_unwind(...).is_err()で受け、誤0のassert panicを成功へ吸収できた。V1 PASSは「意図したpanicまで到達」の独立証拠ではない。独立作者はV2で専用IntentionalOwnerUnwind型とpanic_anyを用い、catch外で型を照合して未知payloadをresume_unwindするよう是正した。親と独立reviewerは是正部分の確定欠陥0を確認済みである。ai source SHA `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA`、ai作者marker SHA `E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7` を新正本とし、上記V1の成功/hash記録とは区別する。S1にV2 A09の期待1 FAIL→未知payload再送出を追加し、追加snapshotは作らない。変異動作とV2の最終受入は未検証である。D群の各後続stageを個別に誤非0で検出する要求がある場合は、単純返値変異の最初の0による遮蔽を避ける別設計が必要であり、今回の案を全stage実測としない。

runtime工程の取得開始側snapshotは09:21:56.931Z input4621488/cached4502144/output16499/reasoning1385（再開直後まで欠測）。09:26:51.803Z観測はinput6809898/cached6654976/output22765/reasoning3490。最終応答前snapshotはhandoffへ添付する。指定Astra/high不変、provider返却metadata未取得、親子包含未確認である。

親はA09の専用panic型による作者最小是正を受領し、別ownerへ編集を依頼した。本担当はV1成功範囲と上記証拠を保存して所有返却し、是正後の重複build/自己検証を行わない。新sourceではV1解除が失効し、独立runnerの新build→G0→実行へ渡す。

同様の形は既存 `media::early_trial_startup_tests::et2s_unwind_releases_only_owned_watch` にもある。ただし同caseはcatch外で再登録後の実入口Err/calls=1を再確認するため、今回計画の一律誤0はcatch外で検出される。最初の意図panicと先行assert panicの識別自体はないが、A09と同じ一律誤0の見逃しを断定しない。B03はcatch後のworker completionとcounter=1を外で検査し、早期assert panicをcompletion=falseで拒む。D群はcatchしたpanicをcleanup後にresume_unwindするため成功へ吸収しない。A11はpoison結果をcatch外で検査する。この確認で見つかったP01一律誤0の検出力不足はV1 A09のみであり、上記V2是正後のS1対象にはA09を含む。

## S1 source固定：一時感度変異・build/runtime禁止

2026-10-14。親からV2独立runner 252PASS・全process終了と所有返却を受領した。入力HEADは `ee84f8e326c3163e58ee2ed54782f0f338adc8c9`。編集前にgit statusを確認し、他ownerの `early-trial-egress-runner.md` 編集を保持した。指定Astra/high不変。親の明示追加所有はmain/app/media/aiの下記本体5箇所だけである。wordweave-changeに従い作者oracle/安全遮断/実加算を保持した。

**現在のsourceは一時的な誤り注入S1であり製品修正ではない。commit、製品build、通常利用は禁止する。** 今回はapply_patchによる変更と静的確認のみ、cargo/no-run/list/runtime、S2/S3/逆復元は0。親のsourceG0確認後のtrial bin no-runだけを次の依頼境界とする。

| 位置 | V2からの最小差分 |
| --- | --- |
| main.rs:60 | trial WINDOW_TITLEを `WordWeave 5 — ET2 S1 synthetic wrong title` へ。通常titleは不変 |
| app.rs:815 | fatal時の `return Err(RecoveryOrInitializationStarted)` をその場の `return Ok(app)` へ。Dは開始しない |
| app.rs:2793 | history早期拒否の条件へ `false &&` を加え、この拒否だけを無効化。後続検査/保存は不変 |
| media.rs:216 | RecordingStartWatch.callsは元のatomic loadを行うが返値だけ0。Recorder::startの同global/poison拒否/実fetch_addは不変 |
| ai.rs:133 | Snapshot.callsは元の配列読取を行うが返値だけ0。実8入口/Lease/登録/poison拒否/実fetch_addは不変 |

実差分は4file、7行追加/5行削除。`rustfmt --check --edition 2021 --config skip_children=true src/main.rs src/app.rs src/media.rs src/ai.rs` と `git diff --check` はexit0。diffを読み、作者test領域への変更なし、依存/設定/harness変更なしを確認した。通常title以外を通常構成で検証したとはしない。S1はtrial構成だけを予定する。

### V2完全逆復元の基準とS1 hash

全値はSHA256である。V2列は変更直前の実物であり、元の固定表も削除せず保持する。S1列は変更後の固定値である。

| source | V2（逆復元目標） | S1 |
| --- | --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` | `B6CAE8CD16C5907D6B50ACA0A0B2DB33F7261B7D09351E73BCDE6BF9AA7AEC78` |
| src/app.rs | `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962` | `EA09F3E73FF6C4A75B5A2C54DAEF5B8F8DCAFA1E2F0837C983CF209CEFD8795B` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` | `78ADF5CDD16CF282A0D9030DFDAF811868BEE4D9510DF7740CCAD8E37DB4FE32` |
| src/ai.rs | `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA` | `CAE050CCF36A94FDCAAADD14AD7D0FFF280E778498771ECAE483AA8D9347FFF2` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

P01作者4markerは変更前後一致：ai=`E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7`、qwen_reading_ui=`283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3`、qwen_settings=`79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3`、qwen_settings_tests=`3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6`。

### S1予定exact：trial bin 17件

順にtitle、history拒否、fatal-App、録音誤0、P01誤0の目的FAILを独立runnerが取得する予定である。第4件の録音とP01の各1 oracleでは実入口遮断/加算を残し、初期0を維持して目的assertまで到達させる。A09は期待1のassert panicが専用IntentionalOwnerUnwind型に一致せずresume_unwindされることが目的である。

```text
early_trial_startup_tests::et03_title_matches_independent_feature_literal
app::early_trial_startup_tests::et04_history_json_rejected_before_backups_drafts_epoch_or_disk_change
app::early_trial_startup_tests::et_d01_raw_loader_error_cannot_change_failure_stage_or_safe_text
media::early_trial_startup_tests::et2s_active_watch_catches_entry_worker_and_rejects_duplicate
ai::trial_egress::tests::etp01_codex_check_blocks_before_lib
ai::trial_egress::tests::etp01_codex_chat_blocks_before_lib
ai::trial_egress::tests::etp01_codex_material_blocks_before_lib
ai::trial_egress::tests::etp01_codex_generate_blocks_before_lib
ai::trial_egress::tests::etp01_codex_read_ink_blocks_before_lib
ai::trial_egress::tests::etp01_codex_transcribe_blocks_before_lib
ai::trial_egress::tests::etp01_transcribe_limit_precedes_observer
ai::trial_egress::tests::etp01_duplicate_thread_join_and_drop_reregister
ai::trial_egress::tests::etp01_unwind_drop_releases_registration
app::qwen_reading_ui::trial_egress_tests::etp01_qwen_inner_send_counts_once_and_never_reaches_wire
app::qwen_reading_ui::trial_egress_tests::etp01_panic_cleanup_waits_for_runtime_before_watch_release
app::qwen_settings::trial_egress_tests::etp01_probe_rejects_before_inputs_resolution_or_worker_creation
app::qwen_settings_tests::trial_egress_tests::etp01_actual_app_dialog_send_is_observed_before_real_transport
```

上記は予定集合でありEXE list未確認。目的assertの詳細は既存感度表を維持する。history変異が影響するD01/D02はこの集合に入れない。録音counter返値0は各P01 fixtureの録音0と矛盾せず、P01返値0はtitle/history/fatalの既存caseに観測口を追加しない。相互隠蔽なしは静的見立てであり、runnerの実診断で各目的FAILを確認する必要がある。

現時点ではS1 source読取のみを親へ返却して停止する。source/EXE/構成変更時の旧解除失効を維持し、予定された目的FAIL以外・timeout・未知workerは停止対象である。親の明示指示まではbuild/list/runtime、S2/S3/逆復元を行わない。今段階の継続起動processなし。

S1着手時token snapshotは未取得。同sessionの09:51:05.888Z観測はinput8589330/cached8415104/output31621/reasoning4984であり、同工程の総消費とは扱わない。最終応答前値をhandoffへ添付する。親子包含・provider実行model/effortは未取得である。

## S1 trial bin no-run完了：EXE対応返却

親のS1 sourceG0/no-run限定解除を受領し、既存impl trial-targetだけで次を実行した。

```text
cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target
```

exit0、build-finished.success=true、compiler error0/warning12。ログは `target/early-trial-build/et2-egress-impl/s1-trial-build.jsonl` と `s1-trial-build.stderr.log`。途中のJSON末尾読取では書込み中の行が未完でparseできなかったため、終了後に全行を再読取し、上記成功と構成を確定した。warningには変異により未構築になったRecoveryOrInitializationStartedが含まれる。ソースを警告解消のために変更していない。

| JSON構成 | 値 |
| --- | --- |
| target | wordweave5 / bin |
| features | `["early-trial"]` |
| profile | `opt_level="0", debuginfo=2, debug_assertions=true, overflow_checks=true, test=true` |
| EXE絶対path | `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe` |
| EXE SHA256 | `CD5313E3214F2833B920290C3278D192915980EB66EE4455BE517D653B0B80C6` |

build直前/直後の8sourceはS1表と全一致し、作者4markerもV2基準と一致する。予定exactは上記S1の17件で、文書から17実名を抽出確認した。EXE list/runtimeは未実施であり、目的FAILは未取得である。独立runnerのV2専用target/EXEは編集していない。

Cargo command sessionはexit済み、継続起動processなし。追加source変更、S2/S3、逆復元、commitは行っていない。親がこのS1 source/config/EXE対応を確認して新解除するまで実行しない。runnerへは本表と予定17件を引き渡す。

no-run終了後の同session観測09:57:20.648Zはinput10717531/cached10531840/output36843/reasoning5373。着手専用snapshot・最終応答分は欠測、親子包含とprovider実行model/effortは未取得である。

## S2 source固定：S1逆復元確認後の2変異

親からS1独立runnerの17/17目的assert FAIL、unexpected/timeout/wait不足/admin不一致/残存0、全command終了と所有返却を受領した。親の明示許可に従い、まずS1表の8source一致を確認して5変異だけをapply_patchで逆復元した。**中間状態の全8sourceは上記V2逆復元目標に完全一致**し、4fileのgit diffは空であった。その後だけS2の2変異を適用した。入力HEADは引き続きee84f8e、他ownerのreport/review/runner/current編集を保持した。

現在のsourceは**一時的な感度変異S2**である。S1の誤title/history拒否欠落/fatalApp公開/録音誤0/通信誤0は残っていない。製品修正・commitではなく、trial構成の予定6caseだけが次の対象である。build/no-run/list/runtime、S3、最終逆復元はまだ許可されていない。

| V2との差分 | 実装位置・維持条件 |
| --- | --- |
| fatal時にDを開始 | app.rs:815のfatal分岐内で既存start_diagnosticsをapp.storage.dir.join("diagnostics")へ1回呼び、その後元のRecoveryOrInitializationStarted Errを返す。成功側Dは元のままで、fatal branchからreturnするため同じcallで2回は呼ばない。予定raw-loader caseのDは合成counter closureであり、実診断開始関数を実行するcaseは対象外 |
| 通信観測の誤非0 | ai.rs:133 Snapshot.callsで実値0だけ1へ返し、非0は元値。実State.calls/登録/Lease/拒否/加算に変更なし。録音callsはV2の実観測値へ復元済み |

V2からの差分はapp.rs/ai.rsだけの15行追加/1行削除である。`rustfmt --check --edition 2021 --config skip_children=true src/main.rs src/app.rs src/media.rs src/ai.rs`、`git diff --check` はexit0。作者marker/本体安全接合を変更していない。compile、目的assertへの到達とFAILは未確認であり、S1の結果から推定しない。

### S2の8source SHA256

変更前は上記S1表、中間V2は同表の逆復元目標、変更後は次表である。V2との相違はapp/aiの2つだけで、他6sourceはV2全bytes一致である。

| source | S2 SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `677993F91D842D0F33955CCC0DE1249A3C7F93A6639D00E6A54F0F2995E1E2A6` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `7936322B33982B6CEDE940722F0A2FCF71D32B3E21EC501FBBD90494223DD22C` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

作者4markerはS1/V2と一致した（ai `E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7`、qwen_reading_ui `283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3`、qwen_settings `79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3`、qwen_settings_tests `3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6`）。main/app/mediaの既存testにも差分なし。

### S2予定exact：trial bin 6件

```text
app::early_trial_startup_tests::et_d01_raw_loader_error_cannot_change_failure_stage_or_safe_text
ai::trial_egress::tests::etp01_ended_lease_rejects_and_never_counts_into_new_owner
app::qwen_reading_ui::trial_egress_tests::etp01_retained_dialog_after_watch_drop_still_blocks_inner_send
app::qwen_settings_tests::trial_egress_tests::etp01_unconfigured_startup_five_operations_remain_silent
app::qwen_settings_tests::trial_egress_tests::etp01_configured_startup_five_operations_remain_silent
app::qwen_settings_tests::trial_egress_tests::etp01_unconfigured_real_dialog_prepare_is_not_sent
```

1件目は元の型/安全文Err判定を通過後、`(loads, diagnostics)==(1,0)` に実1回のDを検出させる。A10は旧LeaseのEnded/旧State直接加算1→2を保ち、新owner全0assertを誤1でFAILさせる。B02は保持Dialog送信の拒否/Wire0を保ち、新owner全0をFAILさせる。D01/D02/D03は `before constructor: cumulative counter 0` の期待0でFAILを取るため、後続各stageへの到達・感度の証拠にはしない。各目的は独立exactであり、想定外失敗・timeout・未終了は停止対象である。

全起動commandはexit済み、継続processなし。S2 sourceG0へ返却し停止する。前工程の実行解除を流用せず、親の新指示前にbuild/list/runtimeやS3へ進まない。指定Astra/high不変、provider返却model/effortは未取得。S2着手時token snapshotは未取得で、終了時取得可能値だけをhandoffへ添付する。

## S2 trial bin no-run完了：EXE対応返却

親のS2 sourceG0/no-run限定解除後、既存impl trial-targetで `cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target` を実行した。exit0、build-finished.success=true、compiler error0/warning11である。

ログは `target/early-trial-build/et2-egress-impl/s2-trial-build.jsonl` と `s2-trial-build.stderr.log`。JSONのbin executableは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA256は `F66B82F50088953612AB863A36140514EF10209364581464EC7FA8D7616CE5A9`。features=`["early-trial"]`、profile=`opt_level="0", debuginfo=2, debug_assertions=true, overflow_checks=true, test=true`。

前後8sourceはS2表と全一致、作者4markerもV2基準と一致した。追加ソース変更なし。予定集合は直前のS2 exact6件であり、文書から6実名を抽出確認したが、EXE list/runtimeは0、目的FAILは未取得である。Cargo commandはexit済み、継続起動processなし。親の新EXE解除を待って停止し、S3/復元/commitを行わない。

同session最終取得10:15:38.946Z：input13557466/cached13358336/output44336/reasoning5878。no-run専用開始snapshotと最終応答分は欠測、親子包含・provider実行model/effort未取得。

## S3 source固定：loader前の合成早期失敗のみ

親からS2独立6/6目的assert FAIL、unexpected/timeout/wait不足/admin不一致/残存0、全command終了と所有返却を受領した。S2の8source一致を確認後、appのfatal-D開始とaiの誤非0をapply_patchで逆復元し、中間状態の**全8source V2完全一致**とsource diff空を確認した。その後にapp.rs:927の `storage.as_ref().map(loader)` だけを、実loaderを呼ばず `Err::<Progress, String>("ET2 S3 synthetic early load failure".into())` を返すmapへ一時変更した。明示型は既存matchのResult型に合わせたもので、compile成功を先取りしない。

現在のV2との差分はapp.rs 1箇所（5行追加/1行削除）だけである。S1/S2変異は残っていない。実Store実装、作者fixture/期待値、録音/通信の実加算・拒否・観測値はV2のままである。旧stageに進まないことで既存assertが拒むかを測る候補であり、製品修正ではない。

`rustfmt --check --edition 2021 --config skip_children=true src/app.rs` と `git diff --check` はexit0。変更前は上記S2表、中間はV2逆復元目標、変更後は以下の8sourceで固定した。他7sourceはV2全bytes一致である。

| source | S3 SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `3EE539727A4AD238983F407F6102CD5A201711BA2662692B39B94741B346D89A` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

P01作者4markerはV2値との再計算照合に全て合格（ai=`E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7`、qwen_reading_ui=`283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3`、qwen_settings=`79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3`、qwen_settings_tests=`3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6`）。appの作者testにも差分なし。

### S3予定exact：trial bin 2件

```text
app::early_trial_startup_tests::et_d01_real_archive_denial_after_both_writes_then_same_data_retry
app::early_trial_startup_tests::et_d01_recovered_ledger_survives_actual_constructor_read_denial
```

1件目は回復loadを呼ばないためnext deck書込みへ到達せず、`FIXTURE_STAGE: actual next deck write must be reached` のassertでFAILを取得する予定である。2件目はloaderによる回復成功・実read拒否を行わず、`FIXTURE_STAGE: loader success and real custom read denial required` の `(loads,diagnostics,denied)==(1,0,true)` がFAILする予定である。これらは不適切な早期失敗を検出する感度であり、本来の回復段階に到達した証拠とは数えない。

予定集合のcompile/list/runtime/目的FAILは未取得。新sourceG0前のbuild/list/runtime、最終V2逆復元、commitは行っていない。全command終了、継続processなし。現在の一時S3を固定して親へ返却する。指定Astra/high不変。着手時token snapshotは未取得、終了側10:20:31.993Zはinput14390825/cached14187904/output47175/reasoning5934（最終応答分・親子包含・provider metadata未取得）である。

## S3 trial bin no-run完了：EXE対応返却

親のS3 sourceG0/no-run限定解除後、既存impl trial-targetで `cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target` を実行した。exit0、build-finished.success=true、compiler error0/warning12であり、合成Errの型を含めbin compileが成立した。

ログは `target/early-trial-build/et2-egress-impl/s3-trial-build.jsonl` と `s3-trial-build.stderr.log`。JSONのbin executableは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA256は `42DBBEBC565C55EE9B1B9E3B0639FDC7B7EAB42904D91DC7B648BCA5332583CB`。features=`["early-trial"]`、profile=`opt_level="0", debuginfo=2, debug_assertions=true, overflow_checks=true, test=true`。

前後8sourceはS3表と全一致、作者4markerもV2基準と一致した。追加source変更なし。予定集合はS3節のexact2件で、文書から2実名を抽出確認したが、EXE list/runtimeは0、目的FAILは未取得である。Cargo commandはexit済み、継続processなし。親の新EXE解除を待って停止し、最終復元/commitを行わない。

同session最終取得10:24:33.845Z：input16104536/cached15852672/output50360/reasoning5958。工程開始snapshotと最終応答分は欠測、親子包含・provider実行model/effort未取得である。

## 最終V2完全逆復元：sourceG0返却

親からS3の2/2目的FIXTURE_STAGE FAIL、S1–3計25目的FAIL、全command終了と所有返却を受領した。許可どおりS3のapp loader変異1箇所だけをapply_patchで元の `storage.as_ref().map(loader)` へ戻した。他ownerの文書編集を保持した。

以下8sourceの全bytes SHA256はV2逆復元目標と完全一致する。

| source | 復元後SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

作者4markerを再計算してV2一致を確認した。`git diff --exit-code ee84f8e -- src/main.rs src/app.rs src/media.rs src/ai.rs` は空/exit0、main/app/media/ai内のS1/S2/S3変異文字列・false history条件の残存は0、`git diff --check` はexit0である。テスト変更なし。

**impl trial-targetのEXEはまだS3版**であり、現物SHA `42DBBEBC565C55EE9B1B9E3B0639FDC7B7EAB42904D91DC7B648BCA5332583CB` を確認した。復元sourceに対応するEXEとは扱わず、旧解除を再利用しない。今回build/list/runtime/commitは0、全command終了、継続processなし。親の復元sourceG0→新trial bin no-run→独立runner focused GREENを待つ。

終了時に取得できた同sessionの最新token eventは10:25:22.273Z、input16755488/cached16500736/output52430/reasoning5958であった。復元工程の開始値/末尾反映は未取得であり、この値を復元工程単独量としない。親子包含・provider実行model/effortも未取得である。

## 復元V2 trial bin no-run完了：新EXE対応返却

親の最終復元sourceG0/no-run限定解除後、`cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target` を実行した。exit0、JSON build-finished.success=true、compiler error0/warning11である。ログは `target/early-trial-build/et2-egress-impl/restored-v2-trial-build.jsonl` と `restored-v2-trial-build.stderr.log`。

前後に全8source/作者4markerを再計算し、上記V2完全逆復元表・V2 marker基準と一致、前後出力も完全一致した。source追加変更なし。main/app/media/aiの対ee84f8e差分は空、`git diff --check` はexit0である。

新EXE絶対pathは `F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe`、SHA256は `F3C602DBE2EEA404573E14D1BD3835A908668ECC4A1DB06172CE5C5240447618`。JSON features=`["early-trial"]`、profile=`opt_level="0", debuginfo=2, debug_assertions=true, overflow_checks=true, test=true`。このビルドによって同pathのS3 EXEが復元V2 EXEへ置き換わった。

list/runtime、製品build、ET3、commitは未実施。Cargo commandはexit済み、開始した継続processは残っていない。親の新EXE解除後に独立runnerがtrial focused36件（P01 20＋従来ET2 16、録音安全対照3を含む）のGREENを取得する境界であり、本担当は実行しない。通常/libの同一V2 bytesによる独立252件証拠を再実行していない。

usage session=`01a139a4-14c3-7402-834c-3e219e5feb55`。取得snapshotは10:31:10.901Zのinput17277913/cached17001088/output55141/reasoning6038から、10:34:41.375Zのinput17759993/cached17468416/output56366/reasoning6147までである（最終報告作成・応答分は未反映）。親子包含・provider返却model/effortは未取得、指定Astra/highは変更していない。
