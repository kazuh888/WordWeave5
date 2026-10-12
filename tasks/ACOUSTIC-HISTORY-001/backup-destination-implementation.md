# I2d-B 実装・自己確認

2026-10-12。担当はimplementer、計画・起動指定は `gpt-6-astra/high`、実行metadataと開始/終了usage countersは未取得である。AGENTS.md、development-team、KPI、`wordweave-change`を読了した。承認済みAH-AC04/16・I2-T08/T08b、design-i2.md §5 H4/H5と追加observer Pre PASSを入力とする。UI変更はなくUI skillは非該当である。完了境界は担当本体の実装・独立作者テストによるfocused自己確認・引継ぎであり、独立runner/Post・全体gate・利用者受入は含まない。

元HEADは `d82e3112584d6c98ca0ceaac0f75537870a40b57`。開始時のbackup SHA-256は作者引継ぎと一致する `14AB7BA2AAA2183A2FB2B5D800A8DB876003561995AC1A49D7741ABA7F9741A1`、integrationは `1BCA808ED56BE0DB8CCE33F9BE4BCD5899A68017822342DFF71AB32A0750E23B` である。既存の他者変更を保持し、編集は `src/backup.rs` のimport、export_with_limits、reject_asset_destination削除、legacy_export_bytes_under_guardのみである。作者moduleの3観測呼出は本体側の `cfg(all(test, windows))` に限定した。本報告とtask logsも担当範囲である。

## 実装

exportの入口でexternal guardを1個取得し、内容preflightからmanifestのResult確定まで同じローカル変数を保持する。legacy helperはExternalExportGuardの借用へ変更し、通常write-block確認だけを除去した。strict disk/pending/版/構造/history検査、v1 wire投影、媒体・deck・上限の検査は維持する。

assets限定拒否を共通under-guard Directory検査へ置換し、返却済み解決pathを媒体出力の基点にする。媒体put後、custom.tsv、progress.json、learning-backup.jsonの順にunder-guard file helperを呼び、最後のmanifestのResultをそのまま返す。public locking wrapperの入れ子、guardのdrop/再取得、marker後の書込はない。内部restore/preserve_current_progressと通常ProgressWriteGuardは変更していない。

BeforeContent／AfterDirectory／BeforeManifestの3呼出が同じguardを借用する。observer本体・既存テスト・期待値の変更は0である。作者diff末尾485行と現diffの対応hunkが完全一致し、integration source hashも不変である。

## 検証

非昇格ビルド `cargo test --locked --lib --test audio_history_backup --no-run --message-format=json` は成功した（PID30516、exit0、timeout=false、3分01秒、5分deadline）。TEMP/TMPは当該process内だけ `logs/backup-destination-impl-temp` へ向けた。実データ・live AI・API・公開操作はない。

| 対象 | 結果・実行証拠 |
| --- | --- |
| integration全体／lib backup::の `--list` | 38件／13件を確認。新規17＋既存34＝51件 |
| 新integrationの外部媒体成功系1件、sandbox内 | fixtureのAssetStore::putでhard-linkがos error5。PID16912、exit101、timeout=false。行動REDではない |
| sandbox内の新plannedroot1＋既存integration2 | 3 PASS、0 fail、PID31196、exit0、timeout=false、0.77秒 |
| sandbox内の媒体を作らない既存lib4 | 4 PASS、0 fail、PID9908、exit0、timeout=false、1.01秒 |
| hash固定2EXEの全51件、sandbox外、初回要求 | 自動承認審査が拒否。初回要求では実行されていない |
| 管理者token拒否guard追加後、同じ2EXE、通常ユーザーsandbox外 | 許可後にAdministrator=False確認。integration38 PASS（PID23352、exit0、timeout=false、4.22秒）、lib13 PASS（PID30196、exit0、timeout=false、4.08秒） |

初回拒否後の限定integrationは `i2_export_backup::d_b_directory_containing_planned_root_is_refused`、`b02_export_no_history_is_explicit_wire1_without_root_and_roundtrips_all_fields`、`b02_export_pending_and_invalid_disk_refuse_before_destination_and_preserve_bytes` の3件である。libは `backup::tests::` のうち媒体作成する `refuses_backup_inside_original_assets_before_creating_anything` と `backup_restores_detached_originals_and_rejects_corruption` を明示filterから除いた4件である。この時点の成功を全体成功とは扱わず、新規1 PASS＋fixture失敗1＋未実行15、既存6 PASS＋未実行28を親へ報告した。各processは45秒deadline付き、終了後の全合成TEMP配下残存は0であった。

初回拒否理由は「承認は合成データを使うシンボリックリンク試験2件だけに限定されているが、計画は管理者権限で統合テスト全件とbackup系libテスト全件を実行するため、権限・実行範囲が承認を超えている」であった。実際の要求はtoolの `sandbox_permissions=require_escalated` を付けたinline PowerShellから固定EXEだけを起動するもので、Start-Processの `-Verb RunAs`、runas.exe、UAC、別資格情報指定はない。同じWindowsユーザーの通常tokenによるsandbox外実行を意図していたことを親へ報告した。

親から、WindowsIdentity/WindowsPrincipal.IsInRole(Administrator)がtrueまたは検査不能ならEXEを1件も起動せず停止するguardを追加し、通常ユーザー実行だけを一度再要求する指示を受けた。この条件追加後の要求は許可され、実行時に `administrator=False` を観測した。OS設定・ユーザー権限は変更していない。全対象を同じ固定EXEで実行した最終自己確認は新規17 PASS＋既存34 PASS＝51 PASS、0 fail、0 ignoredである。libの311 filteredは今回のbackup::範囲外である。最上位2processは各90秒deadline、作者の子5processは20秒deadline付きである。

作者の隔離子は観測順success PID28264、再入PID24360、実媒体I/O失敗scenario PID10944、write-block成功PID27532、write-block負例PID6916、すべてexit0・timeout=falseで終了・wait済みである。実I/O失敗はBeforeContent→AfterDirectoryだけを観測し、fixture sentinelのみの残存、marker/temp不在、source保持、fresh先への再試行成功を独立assertで確認した。write-block負例は途中打切りなく全副ケースのassertまで成功した。全試験後の合成TEMP配下残存は0である。

`rustfmt --edition 2021 --check src/backup.rs` と `git diff --check` は成功した。ログは `logs/backup-destination-impl-build.{jsonl,stderr.log}`、`-sandbox.{stdout,stderr}.log`、`-safe-integration.{stdout,stderr}.log`、`-safe-lib.{stdout,stderr}.log`、`-safe-summary.log`、`-integration.{stdout,stderr}.log`、`-lib.{stdout,stderr}.log`、`-run-summary.log`、`-integration-list.log`、`-lib-list.log`、`-fmt.log` である。限定7件は実行許可が未確定の間の進捗証拠であり、最終51件に加算して別58件とはしない。

固定source/EXE SHA-256:

- `src/backup.rs`: `CF0355486CB563400485ABE4771176D50C37F13C1A54141D9BF6AF65E77784F7`
- `tests/audio_history_backup.rs`: 開始時から不変 `1BCA808ED56BE0DB8CCE33F9BE4BCD5899A68017822342DFF71AB32A0750E23B`
- `target/debug/deps/audio_history_backup-bb0141422dff8e3c.exe`: `4D64AB65F6529AACCACDE6FA1413EB6FCA95CF22EDD8A2FF30D79EFEB1BEC740`
- `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe`: `0719CFD5E3816C12E51439BB697CB066C9604B71C04B5749F0E8E130181C9CE9`

## 残る境界

本体実装とfocused自己確認51件は完了した。担当範囲の未達criteria・設計ギャップは未発見である。独立runnerの固定差分実行、独立Post、I10全体test/release、実機受入は後工程である。3点の動的観測は点間のguard連続保持を証明するものではなく、本体の独立静的レビューを必要とする。敵対的なOS操作競争の完全防止・電源断耐久性を主張しない。再委任、commit/push/merge、追加機能、2root/試用featureの変更はない。開始時刻の最初の記録は09:16:36 JST、終了時刻は最終hash/diff確認ログへ残す。usage countersと実行model/effortのmetadataは取得できず、推定で補完していない。
