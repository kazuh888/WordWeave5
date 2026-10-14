# ET2-P01-V 独立テスト作者引継ぎ

2026-10-14。指定 `ww-test-author / gpt-6-astra/high`、thread `01a13995-0994-7fc2-b085-cca36d93e390` (`/root/trial_egress_tests`)。実行model/effort返却metadataは未取得。AGENTS/development-team/KPIと `agent-skills:test-driven-development` 全文を読了した。担当は独立期待値・fixture・テスト作成のみであり、本体接合、動的実行、再委譲、commit/pushは0である。

入力HEADは `fef8bd27dc1fbf5065f568d0d1d7e538f376bbb3`、開始treeはclean。設計 `early-trial-startup-design.md` SHA256 `8AC25D82C1286ED1C4500BC3DFCF232E76DC01673E35758833C7E373C96F82F3`、plan `early-trial-plan.md` SHA256 `BCD69DBC7B1EA022598374A93D967DE5DA8239427FF5443C3D55F5E2F5E3A785` を照合した。Pre PASS（指摘0）の正本は `early-trial-startup-review.md` のET2-P01-V節であり、短縮依頼の `review.md` ではない。正本ET-06は不変である。

## 固定した集合と独立oracle

新集合は **通常bin20件、early-trial bin20件** である。macro展開6件を含むsource上の予定集合であり、EXE listやruntimeで確認した件数ではない。旧 `early_trial_startup_tests::et06_factory_settings_restore_and_reading_prepare_do_not_record_probe_or_send` 1件は未接続spyのため、このPacketの明示許可に従い下記D群へ置換した。他の既存case・期待値は変更していない。

実名のprefixは A=`ai::trial_egress::tests::`、B=`app::qwen_reading_ui::trial_egress_tests::`、C=`app::qwen_settings::trial_egress_tests::`、D=`app::qwen_settings_tests::trial_egress_tests::` である。下表はprefix＋suffixが一意の完全実名となる。

| ID / ET-06対応 | prefix / suffix | 固定期待値 |
| --- | --- | --- |
| A01 / P01-1 CodexCheck | A / `etp01_codex_check_blocks_before_lib` | 実check→Observed literal、該当1/他7個0、cwd未作成、exe sentinel不変 |
| A02 / P01-1 CodexChat | A / `etp01_codex_chat_blocks_before_lib` | 実chat→同上 |
| A03 / P01-1 CodexMaterial | A / `etp01_codex_material_blocks_before_lib` | 実material_detailed→MaterialFailure::Other(Observed literal)、同上 |
| A04 / P01-1 CodexGenerate | A / `etp01_codex_generate_blocks_before_lib` | 実private generate→同上 |
| A05 / P01-1 CodexReadInk | A / `etp01_codex_read_ink_blocks_before_lib` | 実read_ink→同上 |
| A06 / P01-1 CodexTranscribe | A / `etp01_codex_transcribe_blocks_before_lib` | 実transcribe、上限内合成PCM WAV→同上 |
| A07 / 境界 | A / `etp01_transcribe_limit_precedes_observer` | 12,000,001 bytes→既存サイズErr/全0、12,000,000 bytes→Observed/Transcribe1 |
| A08 / 登録・thread・再登録 | A / `etp01_duplicate_thread_join_and_drop_reregister` | duplicate=AlreadyActive、先行Store Arc保持、別thread実check→join→1、Drop→capture/selected None→再登録0 |
| A09 / unwind・再試行 | A / `etp01_unwind_drop_releases_registration` | 実check1後の捕捉panicでowner解除、再登録0 |
| A10 / Ended・identity | A / `etp01_ended_lease_rejects_and_never_counts_into_new_owner` | 旧LeaseはEndedで旧callsを1/2へ、新owner全0、旧snapshot=Ended、旧owner Dropで新登録を外さない |
| A11 / poison | A / `etp01_poison_is_fail_closed_and_drop_keeps_poison` | snapshot/capture/selected/watchがPoisoned、6実Codex入口固定Poisoned literal、LeaseはPoisonedでも旧counter加算、Drop後poison維持/登録None |
| B01 / QwenSend | B / `etp01_qwen_inner_send_counts_once_and_never_reaches_wire` | 実Dialog new→prepare→Send→Failed/Network/MayHaveBeenSent、QwenSend1/他0、inner Wire0、同期終了 |
| B02 / 保持Transport | B / `etp01_retained_dialog_after_watch_drop_still_blocks_inner_send` | 旧watch終了後の実Dialog SendもFailed/Network/MayHaveBeenSent、Wire0、新watch全0 |
| B03 / panic終了 | B / `etp01_panic_cleanup_waits_for_runtime_before_watch_release` | 実Send1後にblocking worker＋panic、RAII同期Runtime drop後completion=true、Wire0、watch保持下snapshot1→解除→再登録 |
| C01 / QwenProbe | C / `etp01_probe_rejects_before_inputs_resolution_or_worker_creation` | 実start_probe先頭でInvalidState/NotSent、Probe1/他0、Probe spy0、jobなし、revision41/probe_state/inputs保持、追加Store load0 |
| C02 / factory前Store選択 | C / `etp01_default_store_selects_registered_arc_without_io` | default_storeは登録Arcとptr_eq、load/save0、解除後の既定Storeは未設定 |
| D01 / P01-3・ET-04 | D / `etp01_unconfigured_startup_five_operations_remain_silent` | 未設定でconstructorから同Store、下記5操作の各累積通信8個0/録音0 |
| D02 / P01-3・ET-04 | D / `etp01_configured_startup_five_operations_remain_silent` | 設定済みでconstructorから同Store、同じ5操作と保持oracle |
| D03 / 未設定失敗・取消し | D / `etp01_unconfigured_real_dialog_prepare_is_not_sent` | 実app Dialog Input→合成音声prepare=InvalidKey/NotSent→Cancel、通信/録音全0 |
| D04 / 実app QwenSend接合 | D / `etp01_actual_app_dialog_send_is_observed_before_real_transport` | 実open_qwen_reading所有Dialog→prepare→Send→Failed/Network/MayHaveBeenSent、QwenSend1/他0、同期終了後も1 |

D01/D02の共通5操作は、(1)通常 `new_with_storage` 又は試用 `new_early_trial_with`、(2)go_to_qwen_settings→実frame→実編集ボタンのEnter→editor生成、(3)実editor.save/finish_qwen_settings＋一般save_settings、(4)完全JSONをDeserializeして実restore、(5)実open_qwen_readingのDialog Input（3frame）→prepare Confirming→Cancel→同期終了である。2構成×2設定×5操作=20操作subcaseである。設定保存のStore saves=1/host=Tokyo、一般保存disk minutes=7、restore memory/disk minutes=11/topic保持を独立literalで判定する。起動後のqwen_store/editor代入で代用しない。

試用D01/D02だけにhistory JSONのexact拒否文・progress不変・storage ledger全bytes/構造不変を追加した。通常構成へ試用専用拒否を要求しない。旧のpending/connection_check/recorderなし、memory/disk audio_history=None、CODEX_HOME値不変、保存成功・Input/Confirming/Cancelledの期待を保持した。既存 `save_failure_preserves_saved_connection_then_explicit_retry_commits_new_one` 等の保存失敗・再試行試験も無変更である。新Packet内の再試行はA08/A09/B03再登録、失敗はA07/A11/C01/D03、取消しはD01–D03で扱う。

## fixture・終了・観測限界

- 全20caseは既存 `media::early_trial_startup_tests::isolated` を読取再利用する。1実名1子process、合成LOCAL/TEMP/cwd、Admin=False、Hidden、45秒deadline、kill/wait、子終了後のみ固有tree cleanupである。trial registryとpoisonはcaseごとの子へ隔離する。shared harnessは無変更。
- D群のINITIAL/RESTOREDはversion2/audio_history:nullと必須settings/memories/reviews/suspended/ai_calls/study_secondsを全て持つ。settingsは有効な5/11分と短い合成topic、map/object、arrayの型を点検した。両定数は各constructor前にstrict Progress Deserialize/normalizeを通す。history候補もRESTOREDから作り、64桁hex sha256/bytes12を加えるため必須fieldを失わない。意図的malformed JSONは追加していない。fixture自己検査もruntime未実行であり、実行成功は未主張である。
- D群はnormal/trialの両rootへ有効な合成progressを用意する。他方rootのprogress bytesと両rootの合成assets sentinelを最後まで保存する。既存実学習データ・音声・API keyは使わない。A群の無効exe sentinel、未作成cwd、合成Store、60-byte PCM WAVも実データを使わない。
- D群はassertをcatch_unwind内に置き、実app Dialogの `finish_egress_test`→audio job drain→app/Store解放→最後のsnapshot→watch解除を保証する。未知pending/connection_check/recorderは成功扱いにせず、両watchをforgetして登録を子終了まで保持しFAILする。B群はOwnedDialog Dropで同期終了/drainを保証し、B02は後から作ったnext watchより先にDialogを終了する。B03は受信後も残るblocking workerのcompletionを同期drop後に要求する。A08のthreadはjoinする。pending=Noneやreceiverだけをworker終了証拠にしない。
- 動的に測る予定の外部境界はB群のinner Wire0とC01のProbe spy0である。A群のlib到達0及びD04の実Reqwest到達0は、固有合成拒否/counterとG0の実lib/inner直前returnの静的確認を組み合わせる。未設置のlib/HTTP counterを測定済みとはしない。Qwenの既存保守的表示MayHaveBeenSentをNotSentへ変更しない。
- 通常constructorのnative wrapper/diagnostics、実API/認証/マイク/IME/音声再生を試験済みとはしない。自動送信の実在を再現したのではなく、既存ET06の観測接続不足を是正するテストである。接合なしで実入口を呼ぶ行動REDは禁止されており、今回は未取得である。

## 不足APIと固定署名

実装者は設計の `#[cfg(test)] pub(crate) mod trial_egress` 容器へ本体を追加する。作者の `BEGIN/END ET2-P01 AUTHOR TESTS` 内は変更しない。Entry8種/順序、Snapshot.calls、WatchError::{AlreadyActive,Poisoned,Ended}、Block::{Observed(Entry),Poisoned,Ended}、Watch/Lease及び下記は承認済み設計のままである。

```rust
fn watch(store: Arc<dyn CredentialStore>) -> Result<Watch, WatchError>;
fn capture() -> Result<Option<Lease>, WatchError>;
fn reject_if_watched(entry: Entry) -> Result<(), Block>;
fn selected_store() -> Result<Option<Arc<dyn CredentialStore>>, WatchError>;
impl Watch { fn snapshot(&self) -> Result<Snapshot, WatchError>; }
impl Snapshot { fn calls(&self, entry: Entry) -> usize; }
impl Lease { fn reject(&self, entry: Entry) -> Block; }
// qwen_reading_ui::QwenDialog の cfg(test) 限定
pub(super) fn finish_egress_test(&mut self);
```

同module子testでだけ参照するprivate名は `GLOBAL: Mutex<Option<Arc<State>>>`、`Watch.state: Arc<State>`、`State.calls: [AtomicUsize;8]` に固定した。GLOBAL/Stateをpub化する要求ではない。終了後Leaseの旧counterとpoison/Dropの結果を測るためである。型/関数のstub、observer本体、default_store接合、入口blocker、Transport wrapper、finish_egress_test実装は一切追加していない。

Codex合成Err literalは独立期待値として `ET2 synthetic egress blocked: observed` / `ET2 synthetic egress blocked: poisoned` / `ET2 synthetic egress blocked: ended` に固定する。入力/host/path/秘密を含めない。Endedには直接Config経路がないためA10/B02はBlock::Ended/保持Transport拒否で測定し、Ended文字列の動的到達を主張しない。

## 検証・安全順・感度

作者が実行したのは対象4fileのrustfmt、`git diff --check`、下記no-run **1回** だけである。Cargo exit101、compiler error71件は上記の未実装型/関数/GLOBAL/finish_egress_testに関する診断であった。未実装API診断以外のerrorは未検出だが、**不足APIに隠れる後続型検査は未了** であり、テスト自体のcompile成立は未確認である。ログは `target/early-trial-build/et2-egress-author/no-run.jsonl` と `no-run.stderr.log`。Cargo processはexit済み、診断読取processも終了した。EXE起動/list、通信/録音正対照、fixture runtimeは0。同原因の別構成buildは反復していない。

```text
cargo test --locked --bin wordweave5 --features early-trial --no-run --message-format=json --target-dir target/early-trial-build/et2-impl/trial-target
rustfmt --check --edition 2021 --config skip_children=true src/ai.rs src/app/qwen_reading_ui.rs src/app/qwen_settings.rs src/app/qwen_settings_tests.rs
git diff --check
```

実装者→親は唯一の設計ET2-G0に従い、8source全hashと前後一致、通常/試用それぞれのno-run JSON/profile/features/EXE絶対path・SHA、上記予定20実名を凍結する。旧C3録音限定解除は使わない。親の新解除後だけ同EXEのlist照合→既存録音3正対照→A01–A06/B01/C01及びA08–A11/B02–B03/C02の安全・寿命対照→A07/D04→D01–D03の順に1実名ずつ `--exact --nocapture --test-threads=1` で進む。0件/FAIL/timeout/未終了なら停止する。全体ET3/all-targets/release/native gateは不変である。

P01-4感度は遮断と実counter加算を保持する。snapshot/calls返値だけの誤0はA01–A06/A07上限ちょうど/A08/A09/B01/B03/C01/D04の「1」oracleへ、誤非0はD01–D03の各段階累積0及びA10/B02の新owner全0へ対応する。A11のpoison型判定、C02のArc同一性、A10/A11の旧State直接loadはこの返値変異の対象ではなく、別の保持oracleである。作者assert・登録・拒否・実加算・安全動作を変える変異は行わない。相互隠蔽しない変異のまとめ方は実装者と親が凍結時に確定し、各目的assert FAILを個別記録する。変異/逆復元とも再G0必須で、現時点の感度確認は未実施である。

## 作者source snapshot・所有返却

以下はno-run診断対象と同じ最終source bytesである。8sourceのうち変更は作者所有4fileだけで、他4fileはHEADのままである。

| source | SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `C671AB68E1FBB2214AC6B8FB5971CAB99A7E7A678213ADE90D7833CEFCAB18A1` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `1B29C84B01216307A123B4458222C89E362E5247AB4152FA3C56E268773D006C` |
| src/app/qwen_reading_ui.rs | `D032B1AEE66FE233A8C2A44689D797F218C3AD1FABBBFD77B3E37659FC133F89` |

作者module hashは、UTF-8で `// BEGIN ET2-P01 AUTHOR TESTS` の最初の `/` から `// END ET2-P01 AUTHOR TESTS` の末尾 `S` までを含む部分文字列で計算した。marker前indentと最終改行は含めず、内部改行はそのままである。実装者が隣接本体を追加した後もこの4hashを維持する。

| 作者領域 | SHA256 |
| --- | --- |
| ai.rs | `A95972760840E1A3925A0F296B60A28C7EED008BE4A943FB7FAC7E0FF86A65B5` |
| qwen_reading_ui.rs | `283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3` |
| qwen_settings.rs | `79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3` |
| qwen_settings_tests.rs | `3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6` |

計測は親へ返す。取得開始側snapshotは同sessionの2026-10-14T08:45:34.235Z、input569416/cached506496/output1719/reasoning199（着手直後まで欠測）である。終了snapshotは最終応答前の値をhandoffに添付する。親子包含未確認、週枠/current/usageは親所有であり編集していない。本報告と4fileの作者領域を固定して所有返却する。ET2-P01 Closed、行動GREEN、最終受入はいずれも未了である。

## A09判定欠落の最小是正 V2（2026-10-14）

親からV1自己検証は通常/試用各20件＋録音各3件、計46PASSと報告された。これはV1正常観測値での実行結果であり、偽の実行結果へ読み替えない。ただしA09の `catch_unwind(...).is_err()` は、意図したowner unwindだけでなく、先行する固有Err/counter=1 assertの失敗も受理する欠落があった。特にsnapshot返値を誤0にする感度変異では、本来のFAILがcatch内で吸収され得た。V1 PASSはこの判定感度を証明していない。作者はこの指摘を現物で確認した。

runnerの起動build終了・他source編集停止を親が確認し、親の明示許可後にA09だけをapply_patchで是正した。A09関数内の専用型 `IntentionalOwnerUnwind` と `panic_any` を使い、catch外で `payload.is::<IntentionalOwnerUnwind>()` を判定する。未知payloadは新しいassert panicで置き換えず、`resume_unwind(payload)` で元の失敗を再送出する。これにより、実checkの固定Observed Errとcounter=1を確認した後に意図したunwindが起きた場合だけ、RAII解除後の再登録/全0確認へ進む。既存のwatch寿命、counter1、再登録全0は維持した。実名/予定件数は変わらない。

修正範囲は `src/ai.rs` 作者marker内A09と本追補のみである。本体・他case・fixture・shared harnessは変更していない。`rustfmt --check --edition 2021 --config skip_children=true src/ai.rs` と `git diff --check` は成功し、整形による追加編集は不要であった。no-run/runtime/感度実行/再委譲/commitは0である。修正V2のcompile・正常動作・誤0変異FAILは未実行であり、親の再G0とrunner検証を必要とする。最終Post未了を維持する。

V1→V2でai sourceは `A7A202579ABEE7C6F1B56DCC754101BDCAE73A4ABF715686006503534FDBFDF7` → 下表へ、ai作者markerは `A95972760840E1A3925A0F296B60A28C7EED008BE4A943FB7FAC7E0FF86A65B5` → 下表へ変わった。残る7source/3作者markerは着手前後一致である。初回作者版のsource表は履歴として保持し、以後の接合対象はこのV2表とする。

| V2 source | SHA256 |
| --- | --- |
| src/main.rs | `0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40` |
| src/app.rs | `4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962` |
| src/qwen_reading.rs | `3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64` |
| src/app/qwen_settings.rs | `649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC` |
| src/app/qwen_settings_tests.rs | `CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD` |
| src/media.rs | `14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207` |
| src/ai.rs | `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA` |
| src/app/qwen_reading_ui.rs | `4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7` |

| V2作者marker | SHA256（前述と同一計算法） |
| --- | --- |
| ai.rs | `E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7` |
| qwen_reading_ui.rs | `283D5244702AD35FD83869D6CA1C77B5EDF3B8447C9AD164B7E22131602311D3` |
| qwen_settings.rs | `79758D2D5347C06BCC5A4AE038B7FB4114EC8D9DA0553C4AF71B0AED34BA32A3` |
| qwen_settings_tests.rs | `3099EF8852606374FF55206D15A9AB94677CF31CD92F5260CFFDFD36D778F9A6` |

V2 source所有を親へ返却する。本是正工程の開始token snapshotは未取得であり、同sessionの先行工程と混ぜて単独使用量を推定しない。
