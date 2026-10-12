# I2d-M 独立テスト作成

2026-10-12 / ACOUSTIC-HISTORY-001。作者指定 `gpt-6-astra/high`。**作者RED工程完了：25件中15行動RED、10 PASS、fixture不成立0、compile blocker0**。consumerは9 RED/7 PASS、libraryは6 RED/3 PASSである。本体は実装担当が追加した副作用なし固定Err scaffoldだけであり、作者は本体を変更していない。2fileを親へ返却し、実装担当の本体接合へ渡す。

入力はHEAD `a64ede0`、[限定設計](media-export-design.md) SHA256 `F6A1D38ABEFEC370FAF14835BFDF48D40B4F76A6E7321F889E1AD028B1B1F49F`、[Pre PASS](media-export-review.md)、[計画delta](implementation.md) M-AC1–3/M01–09である。AGENTS、開発7役、KPI、`agent-skills:test-driven-development` を読了し、テスト/RED工程だけを適用した。着手時tree clean。途中の親によるusage文書変更は保持した。

所有は `src/store.rs` の新 `cfg(all(test, windows)) mod i2_export_bundle_tests`、`src/app/chat_media.rs` の新 `cfg(test) mod i2_export_media`、本書、ignored logsだけである。既存oracle、production、仕様、モデル/gateの変更、再委任、commit/pushなし。

## 独立期待値と範囲

| AC / case | 証拠 |
| --- | --- |
| M-AC1/3・M01 | 音声、原文、一式の3実メソッド。実root自身、既存progress/custom/pending/app.lock、新規leaf、backups/assets/audio-history配下の拒否。全directory/file/temp ledger、payload tuple、text/strokes/frozen/files、両確認flag、Progress/dirty/messageの不変 |
| M-AC1/3・M02 | 手製audio/日本語UTF-8 exact bytes、既存単一link置換、空白原文、似た名前のroot兄弟。一式は手製JSON、1200×700白背景と左上3×3の赤点という全画素oracle、3member順・長さ・独立CryptoAPI SHA256・version=1 marker。対象だけclear |
| M-AC1/3・M03 | 未固定・原文なし・録音なし・frozen原文・不正座標、既存directory、一式内容生成失敗、親directory不存在の全不変 |
| M-AC1/3・M04 | 外部hardlink、実root向き/外部向きjunction祖先を3実メソッドへ接合。OS aliasの完全集合とatomic故障は既存H試験へ追跡 |
| M-AC1/3・M05 | finite child内でSome(Storage)優先、別LOCALAPPDATA/missing環境、None既定root・未作成root副作用0、rootがfileというidentity不明、実既定rootのlock取得失敗後の保護。任意カスタムroot失敗や起動後環境変更から元rootを復元できるとは主張しない |
| M-AC2/3・M06 | library public façadeの2点observer、同一guard pointer/絶対resolved path、Reentrant両入口、member順とmarker最後、終了後gate再取得。第二member directory注入で先行file exact/markerなし/同先拒否/別新規先再試行。completion directory注入でもErr。panic時registration解除と既存gate poison保持 |
| M-AC2・M07 | child内block_writes後にpublic façade成功、その後も通常writer WriteBlocked、external再取得可。app.fatalをblockの証拠にしない |
| M-AC3・M08 | 実consumerの失敗後payload保持→人が選ぶ別の外部先へ成功。picker取消しは既存if-let分岐/終了harnessと独立reviewへ残す |
| M-AC1・M09 | 手製無効名34種を先頭/後半/completionの各位置へ投入。完全一致/ASCII・非ASCII大小文字衝突、member/completion衝突、空member列・空bytes・異なる日本語名、未作成root祖先の拒否 |

合計25 test（consumer 16、library 9）。table入力反復やchild内部をunique testへ重複加算しない。fixtureは既存の合成settings/harness Fixture、library Tempをread-only再利用する。childは親fixture配下へ隔離し20/45秒の上限でkill/reap、RAIIで回収する。junctionはfixture内作成後に個別unlinkする。実データ、API、音声装置、symlink管理者権限なし。

observer公開範囲は設計署名の `BundleStage` と `observe_bundle(stage, resolved, guard)` のみ。登録はthread-local/RAII、未登録no-op、再登録/再帰拒否、戻り値はunitでproduction Resultを書換えない。点間からcompletionのResultまでguardを連続保持した証拠は独立静的reviewである。第二member/completionに作ったdirectoryはfixture由来であり、実atomic I/O失敗ではない。binaryの途中Err保持は事前拒否状態試験と本体の借用/`?`後clearの独立reviewで補う。

## 初回結果とfixture修正

1. `cargo test --locked --bin wordweave5 i2_export_media -- --nocapture` はcompile成功。sandbox実行は16件すべてが既存harnessのStorage::saveで失敗し、exportへ到達しなかった。行動REDは0である。`logs/media-author-consumer-red.log`。
2. 同じ固定EXEをsandbox外で実行する前にWindowsIdentityを取得し、Administrator=Falseを確認した。true/取得不能なら起動しないguardを使用した。`Start-Process -WindowStyle Hidden` と45秒WaitForExit、合成TEMPのみで実行し、exit101、5 PASS/11 FAIL。8件が行動RED、3件は旧PowerShell hash oracleがstdout空であったfixture失敗である。`logs/media-author-consumer-normal-red-{stdout,stderr}.log`。
3. 行動REDはM01の3件、M04の2件、M05のSome優先・missing環境・実既定root失敗の3件。audio/textは実rootのprogress.jsonを上書き、一式は実root内の新規directoryを受理した。hardlink/junction fixtureは作成成功後に不正受理を観測した。
4. hash fixture不成立はM02 bundle、M05 None、M08 retryの3件である。テスト内CryptoAPI（永続key containerなし、RAII handle解放）へ置換し、SHA-256のabc固定ベクトルを前提検査する。production sha256 helperやmarkerの値を期待値に再利用しない。修正後のcompile/実行はstub追加後へまとめる。
5. library `cargo test --locked --lib store::i2_export_bundle_tests --no-run` はAPI不在14箇所とbytes slice推論1箇所でcompile失敗。推論は明示as_sliceへ修正済み。compile失敗は行動REDへ数えない。`logs/media-author-bundle-compile.log`。

実行したconsumer EXEは `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe`、SHA256 `33AA006BA6EC01A1A68C60762FDB1DE5CFF6A62AA08F82BA22C279A6BE32E3B2`。これはhash fixture修正前であり最終sourceとの一致は主張しない。初回のPowerShell直接GUI EXE呼出は終了待機/終了codeが得られず空logとなったため、有効な結果へ含めない。Start-Processによる完了取得後、所有合成TEMPが空であることを確認し空directoryだけ除去した。

## scaffold後の最終RED（初回結果を更新）

実装担当からstore SHA256 `F36EFE8C343998A1A1C7B4AF923E528DA5067CD2A752CB1F814098D522B2CE51`、chat_mediaは下記返却時hashのままで受領した。scaffoldが副作用なし固定Errだけであることを確認し、`cargo test --locked --lib --bin wordweave5 --no-run` はexit0、2m20sで成功した。型推論とCryptoAPIのcompile blockerは解消した。logは `logs/media-author-scaffold-build.log`。未使用observer警告は本体未接合のためであり、既存警告も含めてgateの変更や抑制をしない。

| 対象・固定EXE | SHA256 | 実行結果 |
| --- | --- | --- |
| library `wordweave5-b3388a7bbef44bb4.exe`（unwind強化前） | `1F4371B53786BD24BB51B37A05AE662C0F0AECE975FCA70FECD53C4936092F59` | --list 9件、3 PASS/6 FAIL、0.63s |
| consumer `wordweave5-72e8adffa8f9ef2c.exe` | `B222F8F48E6318D690370D7BA4DB716BAB4D9C059BB9769D92587604AB2E544E` | --list 16件、7 PASS/9 RED、10.32s |
| library同名EXE（最終unwind強化後） | `1589424C60D319D3E44238D014A35139FB3EDA5D0EC03645440DE49B1E2C2070` | --list 9件、対象1件のみ再実行し0 PASS/1 RED、0.38s |

すべて `target/debug/deps/` 配下である。実行前にhashを照合し、WindowsIdentity取得成功かつAdministrator=Falseを確認した。true/取得不能時はEXEを起動しない。Hidden/PassThru/Redirect/WaitForExitの有限待機とtask logs内の合成TEMPだけを用い、RunAs/UAC・資格情報・実データ・管理者symlink承認を使用していない。run完了後は合成TEMPが空であることを確認して空directoryを除去した。

consumerの9 REDはM01の3件、M04の2件、M05の4件であり、すべて不適切なOkを明示assertionで検出した。初回hash fixture不成立3件のうちM02 bundle/M08 retryはPASSへ、M05 Noneはhash検証を通過した後に既定rootへの不適切OkでREDへ進んだ。既知abcベクトル、全画素、ink JSON、marker hashは実行してPASSした。REDの各table testは最初の不適切Okで停止するため、後続の反復入力をすべて実行済みとは扱わない。

libraryのPASS3件は宛先事前拒否、全無効名、衝突名である。固定Errは拒否入力にもErrを返すため、この段階のPASSは名前validator実装完了を意味しない。REDは成功、第二member途中失敗、completion途中失敗、unwind、write-block中の成功、空member/日本語正例の6件である。成功系は固定Err、境界試験はobserver列が空であることを検出した。

unwind試験は初回にscaffoldのErr.unwrap panicを捕捉した後、未作成出力への読取で失敗していたため、期待するpanic本文 `synthetic observer unwind` を直接比較するassertionへ強化した。これは期待値緩和ではなく誤ったpanicの受理を防ぐ修正である。`cargo test --locked --lib --no-run` はexit0、1m28s、最終EXEの対象1件は実際のpanicがscaffold未実装Errであることを示す明示比較でREDとなった。logは `logs/media-author-unwind-{build,red-stdout,red-stderr}.log`。他24件は影響しないため再実行していない。

一覧/初回25件logは `logs/media-author-scaffold-{bundle,consumer}-{list,red}-{stdout,stderr}.log`、最終library一覧は `logs/media-author-unwind-list-stdout.log`。library 9実名はassertion強化前後で不変であり、25 unique件へ再実行1件を重複加算しない。残存fixture failure/compile blockerは0である。既存harness/attachment/H回帰、全体build/test、独立Postはrunnerへ渡す。

最終返却source SHA256：store.rs `24EC8BDF162DDDC0AB23FCDFE23222295CB7709CDE9743EFCF0782408E6F784A`、chat_media.rs `30E37BEAD696E057C4E0306D5462B85136401E97037170FC04EF0EAD76C10B26`。最終新test moduleの固定hashは次である。UTF-8、CRLFをLFへ正規化し、`mod i2_export_bundle_tests {` / `mod i2_export_media {` から次の既存test moduleの `#[cfg(...)]` 直前まで（間の空行を含む）をSHA256対象とした。

- bundle module：`09FB268E91CBFF4A8D1C7BC80785F5719081D200BCC6FDE97BBBF75C94B44D11`
- media module：`F71AF7A57873928641F8896E58D5801FB0E49F8F70D967E2BEE0A854593D9A5D`

## 最小scaffoldの引渡し履歴（追加済み）

親がstore.rsの所有を本体担当へ渡し、次の設計済み署名へ**副作用なし固定Errのみ**を追加する。作者は本体stubを書かない。

```rust
pub fn write_external_bundle(
    root: &Path,
    dest: &Path,
    files: &[(&str, &[u8])],
    completion: (&str, &[u8]),
) -> Result<(), String>;
```

未使用引数の警告回避は許されるが、guard/observer/file操作はscaffoldに不要である。既存3consumerもこの段階では変更しない。所有を作者へ返した後、fixture修正のcompileとlibraryの行動REDを取得する。その後の本体実装では設計どおりAfterDirectory/BeforeCompletionを同じcfgで呼び出す。

返却時source SHA256：store.rs `677B0BC17F128B7ABDCA034D3FD33F1FB837B7BA5FDA057D3920B163817778D1`、chat_media.rs `30E37BEAD696E057C4E0306D5462B85136401E97037170FC04EF0EAD76C10B26`。`rustfmt --edition 2021`、`git diff --check` PASS。source差分は新cfg test moduleだけである。

## 固定した実名集合とrunnerコマンド

consumer prefix `app::chat_media::i2_export_media::`（初回EXE --listで16件確認）：

```text
m01_audio_protects_root_and_preserves_state
m01_text_protects_root_and_preserves_state
m01_bundle_protects_root_and_preserves_state
m02_audio_exact_bytes_and_other_state_preserved
m02_text_exact_utf8_and_other_state_preserved
m02_bundle_independent_pixels_ink_hashes_and_clear
m02_existing_single_link_files_are_replaced_and_whitespace_text_survives
m03_existing_bundle_and_missing_parent_refuse_without_mutation
m03_invalid_or_absent_payloads_refuse_before_directory_creation
m04_external_hardlinks_refuse_and_preserve_both_names
m04_junction_ancestors_to_protected_and_external_directories_refuse
m05_actual_storage_precedes_different_or_missing_environment_root
m05_none_resolves_default_without_creating_root_then_protects_it
m05_missing_environment_and_unknown_root_identity_refuse
m05_failed_default_storage_open_keeps_the_attempted_default_root_protected
m08_failed_export_retains_payload_for_explicit_external_retry
```

library prefix `store::i2_export_bundle_tests::`（source上9件、compile後--list要確認）：

```text
m01_bundle_destination_preflight_preserves_root_existing_directory_and_missing_parent
m06_single_guard_resolved_path_members_then_completion_and_release
m06_second_member_boundary_failure_retains_prefix_and_allows_new_destination_retry
m06_completion_boundary_failure_is_err_with_all_originals_preserved
m06_observer_unwind_releases_registration_without_clearing_gate_poison
m07_write_block_allows_bundle_but_remains_after_completion
m09_all_invalid_names_are_preflighted_in_first_last_and_completion_positions
m09_member_and_completion_collisions_use_windows_ordinal_case_rules
m09_empty_members_empty_bytes_and_distinct_japanese_names_are_valid
```

```text
cargo test --locked --lib store::i2_export_bundle_tests
cargo test --locked --bin wordweave5 i2_export_media
cargo test --locked --bin wordweave5 chat_media::attachment_tests
cargo test --locked --bin wordweave5 media_export_clears_only_the_successfully_saved_buffer
cargo test --locked --bin wordweave5 unfrozen_annotation_text_survives_fatal_close_and_failed_export
cargo test --locked --bin wordweave5 frozen_background_without_strokes_blocks_close_until_exported
cargo test --locked --bin wordweave5 finishing_recovery_after_an_exit_request_closes_without_discarding_it
cargo test --locked --bin wordweave5 fatal_close_keeps_recovery_controls_visible_without_changing_learning_data
cargo test --locked --test progress_export_destination
cargo test --locked --lib store::i2_export_destination_tests
```

sandbox fixture失敗時はcargoのcompileと実行を分け、固定hashのEXEを上記非管理者guard・有限待機・合成TEMPに限定して実行する。既存harness/attachment/H回帰は作者段階では未実行、runnerへ残る。全体gate `cargo test --all-targets --locked` / `cargo build --release --locked --bin wordweave5`、独立Postは変更しない。native picker/IME/pen、実atomic故障、guard連続生存の静的review、binary途中Err接合の静的reviewは本結果だけで検証済みとしない。

usageは親が区切りで採取する。thread `01a12de4-4b0b-7561-a325-9b8bca07594c`、session `01a08550-055c-7aa1-9834-905e4a7ce148`。指定model/effortとruntime値は別であり後者は未取得。親子包含関係と開始/終了counterは作者側未取得。
