# I2a-M independent tests

役割: ww-test-author、planner 指定 gpt-6-astra/high。runtime の返却 model/effort・使用量 counter は取得できず、推定しない。所有範囲は新 integration と store/commit の M 専用 cfg(test) 節のみである。

## 入力と独立 oracle

- 承認済 design-i2 SHA256: `1375E2A126DBA2AEAD188903D3BA3E0575293B3C919C7721D5786EB771090EAF`。
- migration-design SHA256: `27D5233049D7F7B370381A77196335FBB649B9CB2FAED060CF188E3257F040AA`。
- 手製 UTF-8 v1 は 550 bytes、.NET SHA256 による固定期待値は `a91ceafe5bc4fc2418a3b5c8c949cd1999656dc326071198d44fc94ad25ec989`。production hash/serialize から期待値を生成しない。版重複は raw JSON である。
- M01: exact bytes/hash、既存日別 backup との独立。M02: 同名一致無再書込・不一致拒否。M03: target dispatch、旧 source/limit/path 拒否。M04: before/after、無書込 Err / 実 atomic 後 Err / 実読取拒否。M05: v1 通常・教材保存・旧 pending replay の互換と上限対照。
- M は runtime v1 維持。private target=2 helper は公開 v2 保存受付の証拠ではない。共有 gate は各 call 内で取得し、assert 前に解放する。global poison/block/reset、新権限、実 data/API は使用しない。

## 固定 source

| file | full SHA256 | 新 module SHA256 |
|---|---|---|
| src/store.rs | `51669BDCF86DF21982EF8F5BFCF01FFFCAE2396656B032F499014DEEA1EFE8D5` | `E6959959DF9DBD3FF2DCFBC8865E43DF447682088F21FA83BD45888ADC4D5CF1` |
| src/commit.rs | `1E8E5FDBA5D62DE09751778106E4DDF0A81F0920A99BA04133254BDFEEB090E8` | `A3B4DABB2BCB9E509385D6E76B92FF2CAEA44E3BCC5F49EB61ABB09B962B4F9C` |
| tests/audio_history_migration.rs | `862FE8A4DD0A63E4254F77315E1AE12AA596AF2D3A3F4C2ACE7FCA0225BB84EF` | file 全体 |

新 cfg 節前 prefix（末尾空行だけ除去し LF 一個）は store `F7E3D7B3044195A1AFAA9DB2261BF39C6BF22A488DD3AC922F65A9F38A540C51`、commit `78DD797E730667773A5F96ECE5BBE0579892E3BBDDF1B9F6DCF436E7B2A61E9F`。受領 scaffold 全体 SHA と一致し、既存本体・旧 test・G test は変更していない。新節 rustfmt と integration rustfmt --check、git diff --check は成功した。

## 実行記録

`cargo test --locked --lib --test audio_history_migration --no-run --message-format=json` を一回に集約した。新規件数は lib 18（store 16、commit 2）、integration 3、合計 21 である。

- [build JSONL](artifacts/history-migration-tests-build.jsonl)
- [build stderr](artifacts/history-migration-tests-build.stderr.log)

コンパイルは exit 0、2m10s で成功した（hard-link cache の copy fallback 警告あり）。固定 EXE を通常非管理者・sandbox 外で一回ずつ直接実行した。合計 **5 PASS / 16 行動 RED / fixture 失敗 0**、ignore 0 である。

| 固定 EXE（target/debug/deps） | SHA256（実行前後一致） | command / 結果 |
|---|---|---|
| wordweave5-b3388a7bbef44bb4.exe | `5644F5BF56016D5006A310234B9ED60C6D642DD4AF4DF9F725F7670B8A70F982` | `i2_migration_tests --nocapture --test-threads=1` / 2 PASS, 16 RED, 0.39s |
| audio_history_migration-33b6a419850044f4.exe | `FD079833291F8E42D281E2016EBD8B1453A60D4CA30DA4E0034B9F04217355D3` | `--nocapture --test-threads=1` / 3 PASS, 0.59s |

- [lib 一覧](artifacts/history-migration-tests-lib-list.log)、[integration 一覧](artifacts/history-migration-tests-integration-list.log)、[lib RED](artifacts/history-migration-tests-lib-red.log)、[integration](artifacts/history-migration-tests-integration.log)、[最終 hash/残存](artifacts/history-migration-tests-final-evidence.log)。
- RED 内訳: helper 固定 Io に対する型/成功期待の不一致 10、checkpoint 未呼出 2、publisher 未呼出 2、v1 private 保存固定 Err 2。いずれも既存入口/手製 source の成立確認後の失敗である。
- PASS 内訳: 公開 v1 load/save/material/recover 回帰 3、教材旧 source/escaped intent 上限対照 1、既存 backup の実 read 拒否 1。後者は `fs::read` の OS sharing violation 32 を実証したが、固定 Io が内部で読取を試行した証拠とはしない。
- path case は結果 assert の前に全 6 配置（型違い 2、通常権限 junction 4）の作成・valid source 読取・helper 呼出まで実行済みである。最初の型不一致 assert 以降の照合は未到達であり、GREEN で全照合が必要である。
- GREEN 必須の未到達境界: backup 書込/実 atomic 後 Err、publisher 内の新 backup 実読取拒否 32、After fault 後の exact backup と retry、同名不一致の後続行、版拒否表の後続 assert、v1 縮小後の malformed 旧 bytes/同 source target=2 対照。RED をこれらの成功証拠に読み替えない。

実行後の source SHA は上表のまま、owned temp 残存 0、git diff --check 成功。TDD skill に従い期待値を緩めず固定 scaffold で停止し、本体修正は行っていない。**source/build 所有を親へ返却する。**
