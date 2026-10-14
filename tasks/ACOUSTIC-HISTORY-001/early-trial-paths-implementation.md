# ET1 本体接合・段階分類の再開

2026-10-14。担当 `/root/trial_paths_finish`、planner指定 `gpt-6-astra/high`。実行model/effort metadata・thread UUID・開始/終了usage counterは未取得である。親の測定記録は編集しない。

## 範囲・前提

承認済み契約 SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、probe設計 `40A12BE64F07136DEBE410FB52C22A124D5AC89CDB8D1666E5E19FCF52C05D81` と独立Pre PASSを確認した。AGENTS、開発7役、KPI、wordweave-change全文を読了した。UI変更なし。

開始HEADは親指定 `61d0ef5`。既存dirtyのstore/commit/ai本体、作者のfixture修正、親のusage/current記録を保持した。本再開の本体所有は `src/store.rs` のET1段階分類だけである。commit/ai、作者テスト、manifest、設計には変更なし。前セッションの通常4PASS・試用32PASS/fixture1FAILと、作者段階3の修正済みexact1PASSは旧snapshotの証拠であり、本変更後のGREENとは扱わない。

## 修正

`open_early_trial` が `TrialOpenStage` を保持し、private `TrialPreflight::create_root(&mut self, stage: &mut TrialOpenStage)` に引き継ぐ。create_root内部の読取再検査中は現在の段階を維持し、各実mkdirの直前にのみStartedへ進む。root既存の場合は、create_root後の最終読取再検査もBeforeWritesのままである。app.lock open直前にStartedを確定し、それ以降のすべての失敗はStartedで返す。StartedからBeforeWritesへ戻す代入はない。

reasonと安全文は既存型を維持する。String parse、新公開API、fault seam、検査bool注入、追加guardはない。probeの9点/cleanup一回、OnceLock登録、root検査、load/recover順を変更していない。

## 本体固定・作者への逐次返却

親の追加指示に従い、本体を固定してsource所有を一旦返却する。独立作者の新しい小回帰後にfocused compile/runtimeをまとめて実施する。今回の固定時点ではcompile/runtime未実施であり、GREEN未主張である。

`cargo fmt --all -- --check` と `git diff --check` は成功した。初回fmt checkが示した本変更部分だけをapply_patchで整形し、再check成功。無関係なsourceの自動formatは行っていない。

| 対象 | 固定SHA256 |
| --- | --- |
| src/store.rs（本体修正後） | 767D92D5649D4B1B09E2ED5D2492F3327DA578AB861CD15ADFF767D65FEE5BBD |
| store作者prefix（変更なし） | D2E3225B03A398FC7B2275F49B45010B85F9E5BD5A3F0DD6B4FFBFF3F733C23A |
| src/commit.rs（変更なし） | 7D2720329860CEB3DCE1D2B1B8A5F85DED14EFBDE20FDF556173EF861CDBA95E |
| commit作者prefix（変更なし） | C624AC8332C4B37D1DF033A54A9F5C3E15870F54D57B9DECD3A27F2C63696D71 |
| src/ai.rs（変更なし） | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| ai作者prefix（変更なし） | E609E5392402569B5E08F9170F5CA9C01EE7F5BEF49FAFE48BB3A2013D9977AD |
| tests/early_trial_paths.rs（変更なし） | 0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16 |
| Cargo.toml（変更なし） | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |

prefix境界は作者段階2/3どおり、先頭から本体の `use crate::{`（store/commit）または `use base64::Engine;`（ai）直前までのUTF-8 bytesである。

独立回帰候補は、既存private preflight後の実metadata差替えでcreate_rootを拒否させBeforeWritesを観測するケースと、実mkdirを成功させた同じchecked/stageへ差替えを行い、次の読取拒否でもStartedを保持するケースである。後者は単一production呼出し途中の競争再現とは区別する。テストの具体的な期待値と実装は独立作者が所有する。

残りは作者回帰、固定sourceのfocused GREEN、独立runnerの既存回帰とPostである。全体test/release/native/実API受入はET3等の既定gateに残る。今回の実データ/API/資格情報/プロセス起動/一時fixture作成/削除/commit/pushは0である。

## checkpoint後の感度確認・最終focused GREEN

2026-10-14 10:10:08–10:22:15 JST（最終記録追記前まで）。親のcheckpoint `30fc43c074e21cf5b989722dce4fb48c08cddbcf` と作者段階4の新2回帰を受領し、許可されたprobe条件1行の一時mutationと復元、固定sourceのfocused実行を完了した。上節の未実施記録は本追記前の状態である。再開時のtracked差分はなく、後から加わった親のusage記録とreview記録を保持した。指定model/effortは引き続き `gpt-6-astra/high`、実行metadataは未取得である。

### mutation・復元

`written.is_err() || removed.is_err()` だけをapply_patchで `&&` に変えた。独立oracleの `store::early_trial_probe_tests::et_d01_probe_write_failure_closes_and_cleans_once` をfresh compile後にexact実行した。listは1件、runtimeは0PASS/1FAIL/0ignored、exit101、失敗点は `probe failure must reject startup` である。子の同一失敗を別ケースに加算しない。fixture失敗やcompile不能によるREDではなく、write失敗を見逃した本体変異を検出した結果である。これはレビュー工程の感度証拠であり、成功件数へ加算しない。

mutation中のstore SHA256は `9F2BFEB6B08F04BE635971F3AAE2FAB7A649C33E608F8239829A7856A7F6020B`、lib EXEは `9C81530B56CD8E0B04ED8137215A86B1B29C47ABCBD59273E3ECF99282D0C0A4`。逆apply_patchで元条件へ戻し、以下5fileと3作者prefixが開始snapshotと完全一致することを復元直後と全runtime後に確認した。source5fileの `git diff --exit-code` は0であり、mutationの残存・commit・pushはない。

| 最終source | SHA256 |
| --- | --- |
| Cargo.toml | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |
| src/store.rs | 20C1498B540395164076CECAF9B5ACA56568DE648D0712C3D9F487C53C0AF3F8 |
| src/commit.rs | 7D2720329860CEB3DCE1D2B1B8A5F85DED14EFBDE20FDF556173EF861CDBA95E |
| src/ai.rs | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| tests/early_trial_paths.rs | 0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16 |

store作者prefixは `05DECCA1466F3E77627A7296ECFB782543CC5F91DF0F07F21328949C67CB5660`、commit/ai作者prefixは上節から不変である。storeの上節との差は独立作者の追加2回帰であり、本実行担当がtestを変更したものではない。

### 最終実行

各構成の最終sourceから新たにno-runを行い、そのEXEのSHAと実名listを固定した。通常構成の既存4件も、前回sessionの成功を流用せず新規実行した。targetは両構成とも既定 `target/debug/deps`、CARGO_TARGET_DIR上書きなしである。旧release EXEは変更していない。

```text
cargo test --locked --features early-trial --lib store::early_trial_probe_tests::et_d01_probe_write_failure_closes_and_cleans_once --no-run
cargo test --locked --features early-trial --lib --bin wordweave5 --test early_trial_paths --no-run
cargo test --locked --bin wordweave5 --test early_trial_paths --no-run
cargo fmt --all -- --check
git diff --check
```

compileは上からmutation1m31s、復元後試用3m45s、通常3m34sで、すべてexit0。既存警告とincremental cacheのhardlink不可→copy警告がある。警告解消のためのsource変更は行っていない。fmt/diff検査も成功した。

| 固定EXE（target/debug/deps配下） | SHA256 | filter／最終runtime |
| --- | --- | --- |
| wordweave5-1e58dd7001d62083.exe | 2F965F5A5D2E41D79300C3B5D85620F643FF7D0F61F0A5EB16E5C5ADE5E74285 | `early_trial_`、lib20PASS |
| wordweave5-e0ab9f88db3adadd.exe | 23C10CF18A4C531FFFA4E9819EFCFC5476D8AA068C78A84A3FF4B686C228AA68 | `ai::early_trial_paths_tests`、試用ai1PASS |
| early_trial_paths-573116bfc845b427.exe | 84E294FF85D9ED72A7964D7132DA26E7D7D750236D56258A3AF9CAD4CCE16959 | filterなし、試用integration14PASS |
| wordweave5-72e8adffa8f9ef2c.exe | 3F947F85C6B49465D6386F26F5CCFAF4DEBAE1D89DF14AB9021FBD633CD9C477 | `ai::early_trial_paths_tests`、通常ai1PASS |
| early_trial_paths-2a28ddad8cdc1bbd.exe | 8BAAA82396FE7351D9F9767806FDB32489218ADFDD37407CA1DF06C8729CCB73 | filterなし、通常integration3PASS |

全list/runtimeはexit0、0FAIL/0ignored/timeoutなし。試用35件と通常4件は構成別集合であり、39 uniqueと合算しない。各EXEでlist実名とruntime成功実名の集合一致を確認した。新2件 `et_d01_stage_readonly_recheck_failure_remains_before_writes` と `et_d01_stage_readonly_failure_after_real_mkdir_keeps_started`、および旧fixture修正済み最終recheck1件も同じ最終lib EXEでPASS。後者stage回帰はsticky状態の直接検査であり、single-call途中のrace再現とは扱わない。

### ログ・安全境界・返却

ignored directory `target/early-trial-build/et1-impl/` がログ索引である。

- compile：`finish-mutation-compile.log`、`finish-trial-compile.log`、`finish-default-compile.log`。
- 実名一覧とruntime：`finish-{mutation,trial-lib,trial-ai,trial-integration,default-ai,default-integration}-{list,runtime}.{stdout,stderr}.log`。mutationだけ上記exact1、他は表のfilterを使用した。listは `--list`、runtimeは `--nocapture --test-threads=1`、mutationには両方 `--exact` を追加した。
- 合成環境：`finish-runtime/temp` をTEMP/TMP、`finish-runtime/local` をLOCALAPPDATAへprocess内で設定した。mutation後・試用後・通常後にそれぞれ両directory直下残存0を確認した。実データ/実資格情報/実API/マイクは使用していない。

既知sandbox runtime制約の再試行はせず、sandbox外の通常ユーザー実行を明示要求した。各呼出はWindowsIdentity/WindowsPrincipal取得成功・Administrator=Falseを確認し、取得不能/trueならEXE起動前に拒否する。各起動直前と実行後に固定SHAを照合し、Start-Process Hidden、45秒WaitForExit、timeout時Kill(true)→WaitForExitの有限手順を使用した。timeout/kill実績は0。作者の隔離child内30秒制限と終了waitも維持する。RunAs/UAC/OS設定の変更はない。

本追記以外の恒久変更は0であり、source所有を親へ返却する。既存関連回帰の独立runner、最終Post、ET2、ET3のall-target/release/native gateは残る。今回のfocused GREENはそれらの代替ではない。過去ログは原因・推移の参照だけに用い、今回の35/4PASSへ流用していない。

collaboration taskは `/root/trial_paths_finish`、環境変数から取得したthread UUIDは `01a137e7-ed3a-7bd0-8a38-cfd71e56a966`。exec sessionはmutation compile39693、trial compile31687、default compile40045、trial runtime18171。開始/終了token counterおよびruntime model/effort metadataは未取得であり、親子usageを合算・推定しない。
