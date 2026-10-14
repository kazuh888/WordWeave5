# ET1 独立Postレビュー

## 予備静的確認（2026-10-14）

担当 `/root/trial_paths_post`、指定 `gpt-6-astra/high`、thread `01a137f8-edb6-7e02-b276-17feb6d40991`。レビュー担当はread-onlyであり、親が返却内容を保存した。最終Postは独立runner未受領のため未判定である。

対象は `9fd2dc6..30fc43c` のET1全5sourceと、`61d0ef5..30fc43c` の本体・段階回帰差分。契約・probe設計・対象sourceの固定hash一致を確認した。確定欠陥0、要回答質問0、任意提案0である。

- 通常treeの内容非読込、reparse・複数link・identity不明拒否、mkdir/app.lock直前のStarted遷移と保持。
- 成功後のOnceLock登録、固定root再利用、caller＋通常＋試用rootをdest/temp/最終recheckで保護、既存guardと未登録Storage::at契約維持。
- current・pending旧新Progressのstrict検査、回復前拒否、既存publisher・回復順不変。
- probeの9到達点、create衝突時の非所有ファイル保持、File drop後cleanup一回、失敗時未登録と明示retry。
- AI変更はcwdのresolver接続だけ。PATH探索、認証、model/effort、送信経路は変更なし。
- 作者oracle・fixture修正・新2段階テストの期待値緩和なし。依存追加・Cargo.lock差分なし、diff検査成功。

コードレビュー・信頼性/データ整合性・安全性・プロジェクト規約のSkillを適用した。実装担当によるmutationのexact1FAIL・復元hash一致は親報告として受領し、最終照合時にログを確認する。

残る証拠は、同一最終sourceの通常4・試用35と、既定store/export/backup/media回帰の独立実行である。作者のmixed snapshotを最終GREENへ代用しない。ET2の起動表示・資格情報・JSON拒否、ET3の全体gate・製品EXE・native確認は対象外かつ未完了である。

レビュー担当の編集・実行試験・再委譲・公開は0。指定model/effortとruntime返却値を区別し、後者は未取得である。

## 独立実行受領・ET1-P01（P2、受入阻止）

通常構成は91tracked source/test/manifest/lock hash、8EXE hash、18実行群の実名集合を生ログと照合した。253 raw PASSのうち権限依存symlink2件・別drive alias未用意1件はfixture SKIPであり、行動250PASS/0FAILとする。全list/runtime exit0、Administrator=False、timeoutなし、終了wait済み。mutationのexact1FAILは`probe failure must reject startup`であり、狙った欠陥の検出である。現sourceの条件と全5hash/3prefix復元も照合した。

試用構成のsettings集合でs06が24PASS/1FAILとなった。判定はCHANGES REQUIRED、最終Postは未合格である。

| Finding | 現HEADの原因・影響 | 状態 | owner/oracle/残余 |
| --- | --- | --- | --- |
| ET1-P01 / P2 / 確信度高 | `settings_ui.rs:442`のs06 fixtureが通常rootのliteralを固定し、試用resolverのcaller rootを検査しない | Accepted、限定計画待ち | settings_ui.rsのs06 cfg(test)だけ追加所有が必要。通常/試用の契約literalとresolver一致、外部成功・保護leaf拒否・ledger不変・mkdir/lockなしを維持。ET2/ET3は別途 |

fixtureはStorage::atからstorageを破棄し、試用OnceLockを登録していない。`settings_ui.rs:1253`はStorageなし時にresolverをcallerとして渡し、試用resolverはWordWeave5-EarlyTrialとなる。[承認契約](early-trial-contract.md)の登録前generic APIはcaller root契約という条件に従えば、fixture指定のWordWeave5は外部である。このため今回のFAILは製品guardの契約違反とは判定しない。ただし既存回帰不合格を無視せず、独立作者のfixture是正→両構成settingsの独立再実行→最終Postを要求する。skip化、本体の未登録契約変更、resolver返値そのものを期待値へ流用する修正は行わない。

実productionの登録前操作画面非到達はET2未実装事項であり、本fixture診断を製品EXE提供の根拠にしない。親の初報で本体問題と説明した点は、この契約照合に基づきユーザーへ訂正済みである。

追加4FAILも同原因と独立判定した。chat_mediaのNone、missing/不正root、failed-openの3fixture、およびprogress_export_destinationのh03は通常namespaceに固定されている。failed-openは実同rootのlock失敗が成立せず、保護oracleの前提で停止した。missing/不正rootの旧ログは共通helperしか示さず、副ケースの原因区別は静的制御flowの推論である。是正後には名前付き前提assertで動的確認する。h03はrootだけでなく欠落環境のエラー全文も構成別に検査する必要がある。

## ET1-FX1 / Pre-1：限定Pre PASS

独立reviewer `trial_paths_post`（Astra/high、同thread）の判定はPASS、阻止0、要回答質問0。対象はHEAD `30fc43c074e21cf5b989722dce4fb48c08cddbcf`、branch `codex/development-agent-team`、[計画追補](early-trial-plan.md) SHA256 `3E698E4B6CC2A0778AC12B4B5C2DC012A47381FCC1365134CFE9A7B46A591312` である。

3test-file/5symbolの限定所有、独立literal、通常版期待値保持、M05B名前付き前提、M05C実同root失敗、既存拒否/成功/ledger/memory保持、有限child、両構成83raw件再検証、役分離とrollback/停止/Post条件を確認した。契約・3source・基準91hash一致、文書diff/参照確認も成功。本体・登録前caller契約・ET2へ拡張しない。

このPreは是正着手の許可である。5FAILとET1-P01/ET1全体は未Closed、修正後の独立runner/Postが必要である。計画closure matrixの5子findingをET1-P01の内訳として追跡する。
