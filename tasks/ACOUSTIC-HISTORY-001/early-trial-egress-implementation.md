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
