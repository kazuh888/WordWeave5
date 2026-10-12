# I2d-M 媒体export接合の限定設計

2026-10-12 / ACOUSTIC-HISTORY-001 / 設計担当。状態は **M-D01の計画上の解消を反映済み・独立Pre未了、本体/テスト着手不可** である。親確定の [implementation.md](implementation.md)「I2d-M限定計画delta」に従い、store.rs内の限定bundle façadeとchat_media.rs接合の2file所有へ更新した。M-D01の独立ClosedやPre PASSを本書だけで宣言しない。

## 1. 入力、担当、完了境界

- 入力は [implementation.md](implementation.md) のI2d-M行・I2の7役割当・承認記録、[spec.md](spec.md) AH-AC04/16、[design-i2.md](design-i2.md) §5 H1–H5とI2-T08/T08bである。[early-trial-plan.md](early-trial-plan.md) はd-B/M Closed→ET1という依存だけを参照する。試用の2root機能は先取りしない。
- I2d-Bの独立51件・最終Post PASS/Closedは親の引継ぎと [backup-destination-review.md](backup-destination-review.md) により確認した。現物HEADは `b4f100ca5a8cf997ad1682bbf16f907977e81ebc`、着手時git statusはcleanである。
- 計画者指定はdesigner=`gpt-6-astra/high`。実行model/effort metadataとusage開始・終了counterは未取得であり、指定値を実測値と扱わない。環境取得値は `CODEX_THREAD_ID=01a12dca-d6b3-7581-bb76-143ca9cd099a`、`CODEX_SESSION_ID=01a08550-055c-7aa1-9834-905e4a7ce148`。両値の親子包含関係は未確認、通常開発計測は親へ渡す。
- 読了した規則はAGENTS.md、docs/process/development-team.md、docs/process/improvement/kpi.md、`agent-skills:api-and-interface-design` である。Skillの型契約・境界検査・エラー整合原則を適用する。REST、依存、公開UI、データ形式、非同期処理は追加しない。描画変更がないため `wordweave-egui-ui` は非該当である。
- 所有は本書だけ。目的は3経路の接合、状態保持、最小試験境界を既存コードへ写像し、実装可能性の阻止事項を独立Preへ渡すことである。コード・テスト・計画・gateの変更、再委任、commit/pushを行わない。

現物SHA256：`src/app/chat_media.rs=F8602E5A4BC9D0504B9AB646B871B3C15259DD300735299E3CE7C0D400CB5179`、`src/store.rs=BB35631D2EF3E42EE8B98B9C6BECC3FEC3310C5CD8D6E96E6F6EF47578745EBC`。

## 2. 現物の責務とM-D01の解消方針

`src/app/chat_media.rs:584` の `export_unsaved_chat_audio(&mut self, &Path) -> Result<(), String>` は `unsaved_chat_audio: Option<(会話ID, Vec<u8>)>` を借り、atomic_write成功後だけpayloadとdiscard確認を消す。`:597` の `export_chat_annotation_text` は未固定・非空を検査し、textのUTF-8 exact bytesを書いた後だけAnnotationを初期化する。空白のみの原文も現行どおり退避できる。

`:609` の `export_chat_annotation` は `Annotation::files(&self)` でink/background/compositeのowned bytesを作り、既存version=1、file名・bytes長・sha256を持つmarkerを生成する。新規directoryを作り、3file、`complete.json` の順に直接atomic_writeし、全成功後だけAnnotationを消す。`Annotation::files` は読取専用であり、原文・stroke・背景・textureはWordAppが所有する（`src/annotation.rs:10,135`）。生成物を途中で消費して失敗時に戻す構造へ変えない。

**M-D01 / 計画上解消・独立Pre未了 / crate境界と旧1file所有の不整合**：`src/main.rs:7` はbinary側でappを宣言し、`src/app.rs:80` はlibraryの `wordweave5::store` を参照する。`src/lib.rs:19` のstoreは別crateである。必要な `ExternalExportGuard`、`acquire_external_export`、`write_external_file_under_guard`、`create_external_directory_under_guard` は `src/store.rs:1422–1444` の `pub(crate)` であり、binaryから利用できない。取得エラーの `ProgressGateError` とmessageもlibrary privateである。backup.rsはlibrary内のためd-Bでこの問題が現れなかった。

したがって、既存のpublic file/directory wrapperだけを呼び合わせても「同じguard」「解決済みdirectory返却値」のH4契約を実現できない。各wrapperの都度lock、直接atomic_writeの残置、store moduleのbinaryへの再includeによる別static gateは解決策にならない。最大本体1fileのままの実装を作者へ依頼しない。

親確定の計画deltaにより、同じI2d-Mへstore.rsの限定所有を加え、library内でguardを完結する§6の操作を採用する。新しい作業単位は増やさない。既存guard/under_guard/エラー型の可視性拡張は採用しない。独立Preが署名・名前境界・証拠分担を確認するまで本体/テストへ進まない。

## 3. 3経路の固定契約と接合

| 受入 | consumer / 境界 | 必要な接合・状態遷移 |
| --- | --- | --- |
| T08 / AH-AC04・16 | audio / 単一file | 同じ録音bytesを `write_external_file` へ借用する。Errなら会話ID・bytes・discard_audio_confirmを保持し、Okの後だけNone/falseへ変える。annotationは不変 |
| T08・T08b / AH-AC04・16 | annotation_text / 単一file | 現行のfrozen/空文字検査→外部file helper→Ok後にAnnotation/defaultとdiscard_annotation_confirm=false。Errならtext・stroke・背景・texture・確認flag不変。録音は不変 |
| T08・T08b / AH-AC04・16 | annotation / 複数file | 元のfiles/marker生成→単一external guard→新規directory→返却済みpathを基点にink.json→background.png→composite.png→complete.json最後。各fileはunder_guardで再検査する。全Ok後だけAnnotation/defaultと確認flag=false |
| T08b / AH-AC04・16 | root解決 | 実Storage.dirがあれば必ずそれを使う。Noneの場合だけ既存の副作用なしresolve_storage_rootを使える現行起動契約を適用する。Storage::openやDefault Progressからの推定を行わない |
| T08 / H3・H5 | 失敗・回復 | 事前拒否では全file/dir ledger・既存外部bytes・temp不変。一式の途中失敗では先行新規file/directoryは残り得るが、payload保持、成功表示なし、完了markerなし。自動削除・自動再試行を追加しない |

単一fileは既存 `WordApp::write_settings_export(&self, dest, bytes)`（settings_ui.rs:1246）を再利用する。このメソッドは名前にsettingsを含むが、内容はrootの分岐とbytes書込のみで設定値へ依存しない。新しい同内容のroot選択helperを複製しない。改名・移動は本単位で行わない。一式はrootを1回PathBufとして得て、生成済み3fileとmarkerを§6のfaçadeへ借用し、返却Ok後にだけclearする。

root解決の限界を区別する。現行 `WordApp::new` はStorage::openだけを使用し、失敗時の `new_with_storage(Result<Storage,String>)` は元pathを保持しない。Noneから失敗した任意カスタムrootを復元することはできない。現在の既定起動とd-Sが用いるresolver契約は再利用できるが、任意カスタムroot失敗や起動後LOCALAPPDATA変更から元rootを復元できたとは主張しない。その経路を追加・要求する場合は、試行rootを起動状態に保持する別の所有変更が必要である。失敗したrootを既定rootへ置換するためにresolverを使ってはならない。ET1のroot選択をd-Mへ混ぜない。

Annotationのbytes生成・sha256/marker serializationは既存どおり最初のdirectory作成前に全成功させる。対象はWordAppの同期 `&mut self` 操作であり、生成中にUIから同じAnnotationを書き換えるworkerはない。metadataの保存snapshotを読むbackupとは異なり、生成前から正本gateを保持する必要はない。生成済みimmutable bytesをguard区間へ渡し、directory作成からmarker書込のResult確定までは同じguardを生存させる。追加契約が必要ならPreで指摘し、黙って変更しない。

## 4. 取消し、失敗、通知、UI

3pickerは `chat_media.rs:1107–1155` の `if let Some(path)` 内でのみexportを呼ぶ。picker取消しはメソッド未呼出であり、guard取得・書込・clear・成功通知は0回となる。lockへpickerや確認dialogを含めない。同期exportにworker/token/中断callbackを導入しない。guardをasyncへ移さない。commit途中の新規キャンセルUIも設けない。

結果の分岐は `Result<(), String>` のままである。既存の日本語成功通知、保持を明記する失敗通知、marker欠損の未完了説明をそのまま使う。payload不在、原文の事前条件違反、root取得不能、guard/宛先拒否、I/O失敗はいずれもErrであり、成功前にclearしない。storeの分類・検査・atomic I/O契約はH1–H3を再利用し、consumerで文字列parseや安全判定を再実装しない。

外部export成功はチャット添付・Progress保存・音読履歴登録ではない。Progress/dirty/保存済みhead/学習記録を更新しない。正本write-blockはexternal exportによって解除しない。中途一式は既存契約どおり残し、次回は人が別の新規directoryを選ぶ。OS rename結果不明や敵対的他processによるTOCTOUの完全防止を追加保証しない。

描画・ラベル・buttonレイアウト・font・拡大縮小・scrollを編集しないため、新UI設計は非該当である。日本語文とpathは既存UTF-8/OsStrのまま扱う。両軸中央配置・日本語glyph・狭幅/拡大の確認は既存UI試験を維持し、本単位で改善済みとは主張しない。native picker/IME/ペン実機受入はmockや描画geometryと区別する。

## 5. 独立作者への受入と既存試験

独立Pre PASSの固定hashを親が記録した後、作者は `src/app/chat_media.rs` の `#[cfg(test)] mod i2_export_media` と `src/store.rs` の `#[cfg(all(test, windows))] mod i2_export_bundle_tests` を先に所有し、完了後に本体担当へ逐次移譲する。本体担当は3 exportメソッド、§6のfaçade/private名検査、承認されたtest-only呼出だけを編集する。binary cfg(test)からlibraryのpub(crate) gateへ到達する口は作らない。既存harness/他testはread-onlyで実行する。

| case群 / ID | 独立入力・観測・依存 |
| --- | --- |
| M01 / T08 | 3出口それぞれでStorage.dir配下の既存sentinel、新規leaf/dir、root自身を拒否する。progress/custom.tsv/pending/backups/assets/audio-history/app.lockを含む事前ledger、audio tuple、原文、stroke、frozen、annotation.files、確認flag、Progress serializationを比較する。未保存payloadや空白原文を実教材から採らない |
| M02 / T08b | 外部の日本語file名、新規file、既存1-link file置換、似た名前の兄弟directoryが成功する。単一fileは手製audio/UTF-8 bytes exact、一式は手製text/strokeのink JSONとPNG decode結果を独立oracleで確認する。markerのversion/3件の順・名前・bytes長・sha256を独立期待値と照合し、対象だけclear、他媒体/Progress不変を確認する |
| M03 / T08 | 既存directoryを一式のdestへ渡し、再利用/上書きしない。未固定annotation、空text、frozen text-export、不正strokeで拒否し、payloadと外部ledger不変。内容失敗はdirectory作成前である |
| M04 / T08 | 外部hardlinkや祖先junctionの代表caseを3経路へ接合し、全既存bytes/temp不変を確認する。alias完全集合はH02/H06を再利用し、consumerへWindows identity実装を複製しない。root外へ向くjunctionも拒否する |
| M05 / T08b | LOCALAPPDATAを別rootにした子processでもSome(Storage)を優先する。Noneの既定root解決、LOCALAPPDATA欠落、未作成rootの副作用0、identity不明のErrを確認する。環境変更は有限timeout子processへ隔離する。任意カスタムrootの失敗をNoneへ潰して「同root保護成功」のfixtureにしない |
| M06 / T08・T08b | library façadeで単一guard・解決path・構成file順・marker最後・途中失敗を動的観測する。binaryでは成功/事前拒否のpayload保持を動的確認し、途中Errでもclearへ到達しないことをproductionの`?`接合の独立reviewで確認する。この2種の証拠を混同しない（§6） |
| M07 / T08b | library façadeでwrite-block中の成功とblock維持を確認する。共通H05のgate性質は再利用する。binary試験のapp.fatalだけを実write-blockの証拠に数えない。解除口を追加しない |
| M08 / 既存回帰 | picker取消しのif-let経路、終了要求中の未保存判定、成功後にだけ終了可能になることを既存試験と独立コードreviewで確認する。native picker取消しを自動実機実施済みとは報告しない |
| M09 / T08 / M-AC1 | library façadeへ無効member名、後半memberの不正名、completion不正名、member同士/completionとの完全一致・大小文字衝突を渡す。事前に手製ledgerを固定し、dest/temp/rootの新設0・既存bytes不変を確認する。§6の名前規則をoracleとして列挙し、production validatorで期待値を再計算しない |

既存 `harness_tests::fixture` は隔離rootの下の `data` にStorageを置き、exportは隔離root側を使うため正常系の退避先は保護領域外である。既存期待値を緩める必要はない。以下を新moduleだけの実行から漏らさない。

- `media_export_clears_only_the_successfully_saved_buffer`
- `unfrozen_annotation_text_survives_fatal_close_and_failed_export`
- `frozen_background_without_strokes_blocks_close_until_exported`
- `finishing_recovery_after_an_exit_request_closes_without_discarding_it`
- `fatal_close_keeps_recovery_controls_visible_without_changing_learning_data` と既存 `chat_media::attachment_tests`

予定focused commandは `cargo test --locked --lib store::i2_export_bundle_tests`、`cargo test --locked --bin wordweave5 i2_export_media`、`cargo test --locked --bin wordweave5 chat_media::` と上記harness実名filterである。共通境界の関連回帰は `cargo test --locked --test progress_export_destination`、`cargo test --locked --lib store::i2_export_destination_tests` とし、作者/runnerは実在test名集合をlist出力で固定する。同一testを再実行してもunique件数へ重複計上しない。既定の独立runner/Postと最終 `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5` は変更しない。本設計工程ではsource/test未変更のため実行していない。

## 6. 確定するbundle契約と独立Preへのhandoff

### 6.1 公開する操作とデータ所有

```rust
pub fn write_external_bundle(
    root: &Path,
    dest: &Path,
    files: &[(&str, &[u8])],
    completion: (&str, &[u8]),
) -> Result<(), String>;
```

新規の平坦directoryに、filesの順序でborrowed bytesを書き、別指定のcompletionを最後に書く同期操作である。全入力は呼出中の借用だけであり、保持・spawn・callbackをしない。filesが空でもcompletionだけを書く操作として成立し、空bytesも許す。これはlibrary操作の全域性であり、WordAppは既存の3fileを常に渡す。markerの意味・JSON・hash・版はWordAppが所有し、storeは解釈・再生成しない。戻り値Okは全fileとcompletionの既存atomic操作成功を意味し、Err時の先行出力は自動削除しない。公開型・trait・再帰出力・rollback・retryは追加しない。

WordAppは `files=[("ink.json", ink), ("background.png", background), ("composite.png", composite)]` を保持し、`files`のslice借用と `("complete.json", marker.as_slice())` を渡す。façadeのResultに`?`を適用した後だけ既存clearを行う。実Storage.dirまたは適用範囲内のresolver結果は、この呼出中固定したPathBufとして借用する。

### 6.2 名前と作成前preflight

追加するprivate入口はstore内 `validate_external_bundle_names(files: &[(&str, &[u8])], completion: (&str, &[u8])) -> Result<(), ExternalDestinationError>` とする。bytesを検査・複製しない。Windows module内へ `pub(super) fn validate_bundle_leaf(name: &OsStr) -> Result<(), Error>` を置き、**既存validate_nameを呼び、その前にbundle固有の単一leaf条件を加える**。既存validate_name/validate_path/identityの意味は変えない。

全memberとcompletionについて、空名・`.`・`..`を明示拒否し、元文字列に`/`または`\\`があれば拒否する。Path::componentsがNormal 1個だけであることも確認する。componentsの正規化だけで `a/` や `a/.` を受理しない。NUL/control、ADS/colon、prefix/絶対path、device予約名（拡張子/上付き数字を含む）、末尾dot/space、Windows禁止文字は既存validate_nameへ委譲する。短縮名の衝突候補をmemberへ持ち込まないため、bundle専用の名前では`~`も拒否する。これは既存単一file/directory宛先の規則を変更せず、固定4名を使うUI動作にも影響しない。

全名が有効となった後、member同士および各memberとcompletionを `external_destination_windows::same_planned_name` で比較し、同一ならUnsafePath、比較失敗ならそのエラーを返す。これは同じ新規directoryに作る予定名のCompareStringOrdinal(TRUE)比較であり、ASCII小文字化・locale比較・lossy変換を用いない。case-sensitive directoryでも大小文字だけの別名を許さない。未作成directoryの8.3 aliasを推測生成しない。完了名だけを別の比較方式にしない。小さい借用sliceの二重loopで十分であり、索引・登録機構を追加しない。

検査順を **external guard取得→全名の形式検査→全名の衝突検査→directory under_guardによるroot/dest検査と直前再検査→初回create_dir** と固定する。後半member/completionが不正でも、先行file・directoryを作らない。rootが未作成でdestがその祖先となる場合も既存Directory検査が拒否する。入力destの既存親がない場合にmkdir_allしない。全bytes/marker生成は呼出前に成功している。

「全作成前preflight」はこの全名・内容生成・directory宛先の検査を指す。まだ親が存在しない子fileのfilesystem検査を済ませたとは扱わない。作成後、返却された解決済み絶対directoryを基点に、各member/markerごとに既存under_guardが親/leaf/tempを検査・再検査する。未作成root祖先保護、identity不明拒否、temp保護はH2/H3を再利用する。非WindowsではIdentityUnavailableを返し、dir/fileを作らない。

### 6.3 guardと永続化順序

façade内のlocal `guard` は一度だけ `acquire_external_export().map_err(ProgressGateError::message)?` で取得する。名前preflight後、`create_external_directory_under_guard(root, dest, &guard)?` の返却PathBufを同じlocal `resolved` に固定する。その後は入力destへ戻らず、`resolved.join(name)` と同じroot/guardで各 `write_external_file_under_guard` を順に呼び、最後にcompletionを書いてそのResultを返す。途中drop/再取得、public locking wrapper、直接atomic_writeを入れない。marker Result確定までguardを生存させ、marker後の書込を置かない。

通常writerのwrite-blockを読んで解除・代替する処理はない。external取得口の既存許可範囲のまま出力する。全事前拒否はledger不変、作成後Errは途中directory/先行fileが残り得るというH3契約である。例外的なOS結果不明を新しい保存済み保証へ変えない。

### 6.4 最小test-only seamと証拠分担

storeの作者所有moduleに次だけを置き、production façadeには同じcfgを持つ2呼出を実装者が置く。型/登録/関数/呼出とも非test buildから除外する。

```rust
#[cfg(all(test, windows))]
mod i2_export_bundle_tests {
    // 以下2項目だけが親moduleの本体呼出から見える。
    pub(super) enum BundleStage { AfterDirectory, BeforeCompletion }
    pub(super) fn observe_bundle(
        stage: BundleStage,
        resolved: &Path,
        guard: &ExternalExportGuard,
    );
}
```

AfterDirectoryは作成helper返却直後・最初のmember書込前、BeforeCompletionは全member成功後・completion書込直前である。作者のthread-local登録をRAIIで通常終了/Err/unwind時に解除し、未登録はno-op、再登録・再帰呼出を拒否する。hookは`()`でResultを置換せず、参照を保存せず、fixture内の観測と指定失敗注入だけを行う。失敗注入名は作者の固定scenarioに保持し、production hookの引数やpublic APIへ故障設定を追加しない。gate観測とpanicを含むcaseは有限timeout子processへ隔離し、所有fixtureを回収する。

成功caseは厳密な `AfterDirectory → BeforeCompletion` を観測する。前者で空directory、後者で独立期待bytesの全3member・completion不在を確認する。双方で通常/external取得口がReentrantとなり、返却pathは絶対・両点で同じであることを確認する。API Ok後に完成marker、元root ledger不変、通常gate再取得可能を確認する。guardを点間でdrop/再取得せずmarker Result確定まで生存させることは独立静的reviewで確認する。

途中失敗caseはAfterDirectoryで第二member `background.png` に合成directoryを作る。第一member `ink.json` は成功し、第二memberのunder_guardはdirectory-as-fileを拒否する。観測列はAfterDirectoryだけ、結果Err、残存は新規dest・第一member exact・fixture由来の第二member directoryだけ、第三member/completion/tempは不存在、source ledger不変、終了後gate再取得可能とする。fixtureが作ったdirectoryをproduction出力と数えない。これは**途中の境界拒否**であり実atomic I/O失敗ではない。atomic/cleanupの実I/O回帰は既存H04/H07の範囲で再利用する。元のdestは新規条件で再利用不可であり、別の新規destへの再試行は成功する。

M07では子process内の通常guardでblock_writesしてdropし、同じpublic façadeの適格出力成功後も通常acquireがWriteBlockedになることを確認する。block解除口は作らない。名前検査M09では `ink.json`/`INK.JSON`、member/marker衝突、空名、dot/parent、両separator、末尾separator、absolute/drive-relative、ADS、予約名、NUL、末尾dot/space、短縮名候補を手製負例にし、後半不正でもdest未作成を確認する。空files・空bytesと日本語名はlibraryの正例に含める。

binaryのM01–05/M08は3つの実productionメソッドを呼び、入力・他媒体・Progress・確認flag保持と成功後clearを検証する。binaryにlibrary hookは存在しないため、一式途中失敗の動的観測はlibraryで、consumer保持は事前拒否の動的試験と`?`後clearの独立reviewで担保する。この分担がM-AC3を満たすかをPreが明示判定し、binaryで同じ途中失敗を動的観測したという報告はしない。

### 6.5 Pre判定と作者への受渡し

M-AC1はM01–05/M09と既存harness、M-AC2はlibrary M06/M07と連続guardの静的review、M-AC3はM01–08の状態観測とbinary接合の静的reviewへ追跡する。PreはM-D01の2file所有、公開操作の必要性、名前検査/衝突・予定root祖先保護、別crate試験の到達可能性とこの証拠分担を確認する。未解決の外部仕様変更はなく、実装上の不足が見つかれば本書へ返す。

次は既定Astra/highの独立Preへ本書の固定hashと計画deltaを渡す。PASS後の独立作者もAstra/high、実装者Astra/high、runner Sol/medium、Post Astra/highを維持する。source/test開始とM-D01 Closedは親がPre結果を記録してからである。今回の担当は本書だけを更新し、design-i2.mdへの必要な索引統合は親の所有へ残す。
