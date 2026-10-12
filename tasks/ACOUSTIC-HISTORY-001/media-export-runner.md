# I2d-M 独立検証

2026-10-12 / ACOUSTIC-HISTORY-001。**新25件は独立実行で全PASS。関連回帰を含む有効行動114 PASS / 0 FAIL / fixture skip 3**である。libtest表示は117 passed / 0 failed / 0 ignoredだが、宛先integrationの3件はfixture不成立で早期returnするため行動PASSへ数えない。独立Post、最終製品delivery gate、native受入は本報告の完了境界外である。

計画者指定は `gpt-6.1-sol/medium`（[計画](implementation.md)のI2d-M割当と親dispatch）。起動指定とruntime metadataを区別し、後者のmodel/effortは未取得である。thread `01a12e02-2ac8-7332-ab10-3a52bfa6b9c0`、session `01a08550-055c-7aa1-9834-905e4a7ce148`。開始/終了usage counter、親子包含関係はrunner側未取得であり、親の同task観測へ渡す。検証実行は2026-10-12 11:48頃–11:53頃（+09:00）、compile実測3m46sである。時刻からtokenや週枠を推定しない。

AGENTS、開発7役、KPI、wordweave-changeを読了し、runnerのリスク別検証と報告だけを適用した。入力は [独立テスト/実名集合](media-export-tests.md)、[実装引継ぎ](media-export-implementation.md)、[設計](media-export-design.md) §5/6、[Pre PASS](media-export-review.md)。所有は本書、ignored logs/隔離TEMP/build出力だけであり、source/test/oracle/settings/計画は変更していない。再委任、commit/push、ユーザーアプリ停止、実学習データ・実API・資格情報・OS設定変更なしである。

## 固定した対象

着手時はHEAD `24fa10c2300d04bacb871f91e78c08c99610f1b4`＋`src/store.rs`/`src/app/chat_media.rs`未commit本体差分であった。compile中に親が同じ本体をcheckpointし、終了時HEADは **`81aaf978be3cac23b27acefcab85392aa375df39`**、source2本に対するHEADとの差分は0である。対象差分は `24fa10c..81aaf97` の本体2fileであり、同commitに実装報告/current索引も含まれる。親のcommit/pushはrunnerの操作ではない。

| source / oracle | SHA256（実行前後一致） |
| --- | --- |
| src/store.rs | `12023DF049CFD57C034CD95B87FDF8D20A3368A8A2F24B733BA01457A99338D9` |
| src/app/chat_media.rs | `24171BE283A1D856A2752FA9CEE60D3AE4D43EE4386EFAC6C1B1C5C09C47D5F3` |
| bundle独立module | `09FB268E91CBFF4A8D1C7BC80785F5719081D200BCC6FDE97BBBF75C94B44D11` |
| media独立module | `F71AF7A57873928641F8896E58D5801FB0E49F8F70D967E2BEE0A854593D9A5D` |

oracle区間は作者どおりUTF-8/CRLF→LF、`mod i2_export_bundle_tests {` / `mod i2_export_media {` から次の既存 `#[cfg(...)]` 直前まで、間の空行を含む。`--list`で作者固定の9/16実名集合と完全一致した。table反復・子process・再実行をuniqueへ加算していない。

## 新規compileと固定EXE

実コマンドは `cargo test --locked --lib --bin wordweave5 --test progress_export_destination --test audio_history_gate --no-run`。exit **0**、3m46s。今回のログは `logs/media-runner-build.log`。既存warningとincremental cache hardlink失敗→copy fallbackを保持した。既知のsandbox Storage/save/hardlink実行失敗は再試行せず、compileとruntimeを分離した。

以下はいずれも `target/debug/deps/` 配下であり、今回のcompile出力である。list前・各実行直前・全実行後にSHAを照合した。

| ID / EXE | SHA256 | list exit / 実在unique数 |
| --- | --- | --- |
| lib / wordweave5-b3388a7bbef44bb4.exe | `94038D8B706691D01900FF134D4E29C254FEF4821FF7D10BD591776858C72777` | 0 / 333 |
| bin / wordweave5-72e8adffa8f9ef2c.exe | `BBEFEAE390C45F528D18B2708244851076C31F6D5E7DD51F09559B810571BB81` | 0 / 321 |
| progress / progress_export_destination-b4a366b351b447cc.exe | `9647CF78D0D605CFFED1DC0C1D7A10A3C4A23720EC83A83FB58930C64F73ABFD` | 0 / 29 |
| gate / audio_history_gate-0c0e7e180bf0149a.exe | `7C6444C60097CD90707A9077B53C186263DBDD281DE7B25C965E1E069A98A7D5` | 0 / 16 |

## 独立runtime結果

実呼出は各固定EXEへ下表filter（空ならtarget全件）＋`--nocapture --test-threads=1`である。cargo selectorとの対応はlib=`--lib`、bin=`--bin wordweave5`、progress=`--test progress_export_destination`、gate=`--test audio_history_gate`。compile後にcargoを再呼出してEXEを更新することはしていない。

| run / EXE / filter | unique選択 | raw passed / failed / ignored | native exit / timeout | 外側実測秒 |
| --- | --- | --- | --- | --- |
| 1 lib `store::i2_export_bundle_tests` | 9 | 9 / 0 / 0 | 0 / false | 1.775 |
| 2 bin `chat_media::` | 23 | 23 / 0 / 0 | 0 / false | 21.053 |
| 3 lib `store::i2_export_destination_tests` | 28 | 28 / 0 / 0 | 0 / false | 3.535 |
| 4 lib `backup::i2_export_backup` | 7 | 7 / 0 / 0 | 0 / false | 3.391 |
| 5 progress 全件 | 29 | 29 / 0 / 0 | 0 / false | 3.358 |
| 6 gate 全件 | 16 | 16 / 0 / 0 | 0 / false | 7.113 |
| 7 bin `media_export_clears_only_the_successfully_saved_buffer` | 1 | 1 / 0 / 0 | 0 / false | 1.850 |
| 8 bin `unfrozen_annotation_text_survives_fatal_close_and_failed_export` | 1 | 1 / 0 / 0 | 0 / false | 2.749 |
| 9 bin `frozen_background_without_strokes_blocks_close_until_exported` | 1 | 1 / 0 / 0 | 0 / false | 1.339 |
| 10 bin `finishing_recovery_after_an_exit_request_closes_without_discarding_it` | 1 | 1 / 0 / 0 | 0 / false | 0.935 |
| 11 bin `fatal_close_keeps_recovery_controls_visible_without_changing_learning_data` | 1 | 1 / 0 / 0 | 0 / false | 0.425 |

run2の実在集合は `app::chat_media::i2_export_media::` の作者固定16件＋`app::chat_media::attachment_tests::`の7件だけである。新規はrun1の9＋run2の16＝25件、関連既存はraw92件（有効89/skip3）。全選択117名は互いに非重複であり、各runのsummary合計と選択集合数も照合した。

今回の全listは `logs/media-runner-{lib,bin,progress,gate}-list-{stdout,stderr}.log`。各runの全実名と結果は `logs/media-runner-run-<1..11>-{stdout,stderr}.log`、集約は `logs/media-runner-native-summary.log`、固定manifest/guardは `logs/media-runner-manifest.json` / `logs/media-runner-native.ps1`。古い作者/実装ログは今回の結果へ流用していない。PowerShell wrapper自体はshell exit0だが `$LASTEXITCODE`は未設定で空であり、native exitは各Process.ExitCodeと全結果照合による。

## fixture skipと安全確認

run5 stderrの `WW5_H_FIXTURE_UNAVAILABLE` は次の3件である。rawではokだが、本報告ではskipとして行動成功から除く。

- `h02_directory_symlink_is_rejected_when_fixture_is_available`：作成がWindows error1314。権限依存の既存2件の一つであり、指示上のignored/未検証として維持する。
- `h02_leaf_file_symlink_is_rejected_when_fixture_is_available`：同error1314。権限依存の既存2件の一つである。過去の管理者実行は今回の証拠へ数えない。
- `h02_existing_alternate_drive_alias_is_protected_when_available`：所有fixtureに対応する既存same-volume別drive aliasなし。drive設定を変更せず未検証とする。

ほかの今回stderrにfixture unavailable/skip表示はない。short-nameは `WW5_H_ALIAS_CONFIRMED:short-name` を今回確認した。失敗0、compile blocker0、新25件のfixture不成立0である。

require_escalatedによるsandbox外実行前にWindowsIdentity/WindowsPrincipal取得成功かつAdministrator=Falseを確認した。trueまたは取得不能ならEXE起動前拒否するguardである。RunAs/UAC・資格情報・既存管理者承認の流用なし。`Start-Process -WindowStyle Hidden`、list30秒/run120秒の有限WaitForExitを用い、全実行は期限内に終了し、kill分岐は未使用。各childは既存テストの有限待機と終了waitに従い、終了後に4固定EXEの新規process残存 **0** を確認した。

TEMP/TMPはこの検証processだけの `logs/media-runner-synthetic-temp` へ向け、残存子要素 **0** を確認して空directoryを除去した。実録音・学習記録は使用していない。実行前後のsource/oracle/EXE hashは一致した。

## 次工程と未検証

M-AC1/3の実consumer状態/保存先/再試行と独立画像・JSON・hash oracle、M-AC2のlibrary guard観測/途中境界拒否/write-blockを新25件で検証した。binary一式途中Errからclearしないこと、marker結果までguard連続保持、picker取消しのif-let接合は設計どおり独立Postの静的確認へ渡す。observerのdirectory注入は境界拒否であり、実atomic失敗とは呼ばない。関連backupの実I/O故障試験成功も電源断やOS結果不明の耐久性全体を保証しない。

親指定により `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` は後の製品delivery gateへ残す。本runnerは無関係target/releaseを再buildしていない。native picker/IME/ペン/マイク/TTS/実Codex認証、敵対的TOCTOU、Noneから任意の失敗カスタムroot復元、上記fixture skip3件は今回未検証である。製品受入完了やPost PASSを主張しない。
