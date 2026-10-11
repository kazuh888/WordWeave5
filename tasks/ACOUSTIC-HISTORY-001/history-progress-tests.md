# I2a-S independent oracle / RED

ww-test-author、指定 gpt-6-astra/high。runtime model/effort・usage counter は未取得である。TDD skill は作者責務だけへ適用する。承認済み限定設計 SHA `CE1B45349BD7DF4C20DBBC22ADF4C24888DB681EB7533A8244864830E891BF63`、親確定 S delta を入力とする。完了境界は test 固定・compile・RED と所有返却であり、本実装ではない。

## M作者責務の先行改訂

旧 M raw UTF-8 fixture 550 bytes / SHA `a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989`、旧 M source snapshot・RED/GREEN 記録を変更しない。新旧は `artifacts/history-progress-before` と比較する。

| 旧ケース | 承認要求 → 新ケース |
|---|---|
| integration assert_kept / v1_load...daily_exact_backup | runtime2 と load bytes 不変を維持。`m05_v1_load_is_read_only_and_first_v2_save_keeps_both_exact_backups` で wire2/null・hash と日別 backup の正確な集合/bytes |
| integration new_v1_material...without_migration | `m05_new_v2_material_save_keeps_old_exact_intent_and_hash_backup`、wire2/null・exact old intent と hash 退避 |
| integration handwritten_v1_pending... | 名前・3状態・disk/archive exact wire1・無 hash・再 recover 不変を維持、decode runtime2 を別 assert |
| store m03_invalid_raw_version... | v2 root 欠落を InvalidDocument、v2 不正 root/未知 top/重複を追加。新 `m03_valid_old_v2_null_or_root_needs_no_migration` で正常旧 v2 の無退避・無 fault/publisher |
| store m05_v1_shrink... | `m05_v2_save_rejects_oversized_old_source_without_changes` と `m05_v2_save_rejects_malformed_old_source_without_changes` へ分離。target1 no-op と target2 limit 拒否は維持 |
| store 通常保存の順序対照 | 新 `m05_first_v2_save_migration_checkpoints_precede_metadata`、無 fault / Before / After の bytes 境界 |
| commit v1_material_private...without_migration | `m05_v2_material_private_entry_matches_public_save_with_exact_migration`、公開入口対照と Before→After・exact 退避。新 `m05_v2_material_migration_faults_never_publish_pending_or_metadata` |

Mは旧21件から25件（純増4、既存改名4）になる。上記以外の M01–04、commit 上限対照、旧 test/G 節と production は所有外であり変更しない。M責務の改訂を先に終え、次に S01–05 の新節/新integration作成へ所有を切り替える。両版許容・ignore・期待値削除による救済は行わない。

## S作者責務

新 S は 15 件（store 専用節 11、新 integration 4）である。S01 は全 Progress legacy field と Settings の手製 sentinel、および省略時の個別 default。S02 は nullable/必須 root、元 wire 版と runtime2、raw 重複・未知 field・欠落・型・構造と正常形未知版。複数不正の版優先順位は要求しない。S03 は pure read/head、strict path 8 配置、小上限の一致/超過、typed head 分類、実 sharing32 read 拒否、15秒上限の子 process write-block（親 gate 不変）である。S04/S05 は上表の M 改訂で実 public v2 save と旧 wire1 replay を担当し、重複 test を増やさない。

`cargo test --locked --lib --test audio_history_progress --test audio_history_migration --test audio_history_gate --no-run --message-format=json` を一回に集約した。固定 EXE で対象 filter を実行する。I10全体/release、R-M、N、実機/API はこの作者工程の対象外である。

[build JSONL](artifacts/history-progress-tests-build.jsonl)、[build stderr](artifacts/history-progress-tests-build.stderr.log)、[build前不変証拠](artifacts/history-progress-tests-before-build.log)。後者は現 production prefix＋旧 snapshot test の再構成 SHA が受領 scaffold full SHA と一致し、旧 test/G 節が完全一致することを記録した。

## 固定実行結果 / 所有返却

集約 compile は exit 0 / 3m01s、compile error なし（未接続口 dead-code と hardlink fallback 警告あり）。通常非管理者 token を確認して固定 EXE 群だけを sandbox 外で実行した。81件は **57 PASS / 24 行動 RED / fixture 失敗0 / ignored0** である。

| 分類 | PASS | RED |
|---|---:|---:|
| 新 S（15） | 1 | 14 |
| 改訂 M（25） | 15 | 10 |
| 不変 G（24） | 24 | 0 |
| 不変旧 store/commit（17） | 17 | 0 |

lib は `store:: commit:: --nocapture --test-threads=1` で 41/17、1.74s。新 progress integration は同 flags（filterなし）で0/4、0.01s、migrationは0/3、0.32s、gateは16/0、4.67s。子 process の exact 1件は親1件の内訳であり二重計上しない。timeoutなし、親 gate 汚染なし、owned temp 残存0である。

- [lib一覧](artifacts/history-progress-tests-lib-list.log) / [RED](artifacts/history-progress-tests-lib.log)
- [progress一覧](artifacts/history-progress-tests-progress-list.log) / [RED](artifacts/history-progress-tests-progress.log)
- [migration一覧](artifacts/history-progress-tests-migration-list.log) / [RED](artifacts/history-progress-tests-migration.log)
- [gate一覧](artifacts/history-progress-tests-gate-list.log) / [結果](artifacts/history-progress-tests-gate.log)
- [Mケース差分分類](artifacts/history-progress-tests-m-oracle-diff.log)、[最終 full/prefix/module/EXE hash・不変・清掃・fmt](artifacts/history-progress-tests-final-evidence.log)

| 固定 source | SHA256 |
|---|---|
| store.rs | `B889443636B0528B3B5099C6501D219423AAAEB3A6DDDA48359A5F942A80BC7B` |
| commit.rs | `C1E7D086D12BB872B8B756616030EE12EFF4E150065D72539B3BBAA68CE022AD` |
| audio_history_migration.rs | `F78901C7A0D68D4DFC7FF46494C78ECF1D18C1989ABE5194A7584E98C8EE0CB8` |
| audio_history_progress.rs | `9C4BA7EB25451A88FBA4B8CC6FAE66B51B2ACF2C3E740A0A357FE376D7E870B0` |

固定EXE（target/debug/deps、実行前後一致）: lib `wordweave5-b3388a7bbef44bb4.exe` / `E745897179DA2B48D959E92144E7524D520F5B37BD7EE8FA1D0B9C8EE313A60A`、progress `audio_history_progress-892b5d70a3c847c8.exe` / `BA54A8C7A6506B4C34DC17794519E30F0C8F4DF2ACE5806BE56D3D5B2BD63D7C`、migration `audio_history_migration-33b6a419850044f4.exe` / `0882EE32446D1708B5C332ED0A82704B4128922A8CDADE0496034FE947A0AA99`、gate `audio_history_gate-0c0e7e180bf0149a.exe` / `C3D88FDA61560315DC695B766851CA1E096827E00E3F3869EE987857AB21887C`。

### RED分類とGREEN必須の未到達

S正例は旧runtime1、旧validate1、固定read/head Ioで失敗した。全field手製fixtureは既存公開decode/validateとsentinel全値照合を通過してからruntime1≠2で失敗した。全strict path配置の作成/呼出は完了し、照合はplacement2で停止した。S実読取拒否のPASSはOS sharing violation32の成立証拠であり、固定Io内部がreadした証拠にはしない。Mの既存/新publisher後read拒否32、実atomic後Err、retryは既存M正例として通過した。

次は全assert到達が必要である。S full/default正例のtyped decode/normalize、初回v2/null validate失敗後の有効root・v1null・serialize、strict拒否表row0以降の照合、head InvalidDocument後のUnsupported/Unsafe、正常leaf欠損後のexact/over limit、純読取初回null失敗後のroot/ledger/head、write-block子のpure read成功後のhead/load/ledgerが未到達である。open後消失は実レースを要求せず、設計どおりprivate分岐/独立コードレビューで別確認する。

改訂Mは旧wire2全拒否、空checkpoint、旧v1保存成功、runtime1でREDである。正常旧v2のnull成功後root、v2不正表後続assert、S-M初回正常保存後のBefore/After fault行とbytes境界、教材Before失敗後After行、integrationのruntime2 assert後のhash/日別集合・wire2/null、旧pending replayの残る2中断状態が未到達である。旧pendingの最初の状態ではdisk/archive exact・再recover無変更・hash無しを先に通過した。未到達をGREEN扱いしない。

試験前後で production、旧test/G本文とG integrationの不変を再確認し、新節fmt / integration fmt / diff checkはいずれも成功した。旧M証拠は上書きしていない。期待値の緩和・ignore・本体修正をせず、**source/build所有を親へ返却する。**

## S実装後の限定oracle訂正

親から独立review済みの指定訂正を受領し、作者が原因を再確認した。初回実装の81件は80 PASS / 1 FAILであり、[旧失敗ログ](artifacts/history-progress-implementation-progress.log)を保存したままにする。唯一の失敗はdefault roundtripの `to_value(read)` 対 `from_slice<Value>(to_vec(p))` で、font_scaleが `0.800000011920929` 対 `0.8` となる経路差である。

locked serde_json 1.0.151のローカル source（value/ser.rs:151 → value/from.rs:42 → number.rs:323）ではf32をValue用のf64へ拡幅する。一方ser.rs:154/1691はf32用formatterへ渡す。値保持の契約が異なるのではなく、比較する表現層を作者が混同した。対象1testの比較だけを、`to_vec(&read)==raw` の全field exact wireと `read.settings.font_scale.to_bits()==p.settings.font_scale.to_bits()` の2 assertへ分けた。丸め・tolerance・field除外はなく、version/null/validate/runtime1拒否の既存assertは保持した。

訂正後test SHAは `81493C0711B987ED52B12CA00DB93D57E6C5EE5B138EF76A49C91F02542B790A`。当該置換だけを逆適用すると旧SHA `9C4BA7EB25451A88FBA4B8CC6FAE66B51B2ACF2C3E740A0A357FE376D7E870B0` と一致する。production store `4C1287B2CDB4DE4621A39E06ED67DE2A181A272B7FB67507F4708C648FD00EEE`、commit/M/Gは変更しない。[限定差分・hash証拠](artifacts/history-progress-tests-oracle-fix-evidence.log)。

`cargo test --locked --test audio_history_progress --no-run --message-format=json` だけを一回再compileし、exit 0 / 1m53s。[build](artifacts/history-progress-tests-oracle-fix-build.jsonl) / [stderr](artifacts/history-progress-tests-oracle-fix-build.stderr.log)。他3EXE/77件はsource/test/EXE不変を確認し、再実行していない。

新固定EXE `target/debug/deps/audio_history_progress-892b5d70a3c847c8.exe` SHAは `065D050E24137B9E254CC2B421F6338F46BE985D26B053D4DFACD00E7AF524E9`（前後一致）。[一覧4件](artifacts/history-progress-tests-oracle-fix-list.log)を確認し、通常sandbox内で `--nocapture --test-threads=1` を実行した。[結果](artifacts/history-progress-tests-oracle-fix-green.log)は **4 PASS / 0 FAIL / 0 ignored、0.02s**。既存version/null/validateと末尾のruntime1拒否まで到達した。他の失敗・fixture失敗はない。

[最終hash/静的検査/清掃](artifacts/history-progress-tests-oracle-fix-final.log)にproduction・他test/EXE・旧失敗log不変を保存した。fmt/diff check成功、owned fixture残存0。TDDのoracle訂正だけであり、productionを変更せず許容差も導入していない。**source/build所有を親へ返却する。**
