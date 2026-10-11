# I2a-M 初回v2退避 実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。指定 `ww-implementer / gpt-6-astra / high`。実行metadata・使用量開始/終了counterは未取得である。既読のAGENTS.md、development-team、wordweave-changeを所有範囲に適用した。

親からG Post Closed・M最終Pre PASS・M-D1解消を受領した。入力SHAはdesign-i2.md `1375E2A126DBA2AEAD188903D3BA3E0575293B3C919C7721D5786EB771090EAF`、history-migration-design.md `27D5233049D7F7B370381A77196335FBB649B9CB2FAED060CF188E3257F040AA` と一致する。implementation.md M/S/Rの工程境界を維持する。

今回の変更は独立RED前の最小scaffoldだけである。

- store.rs: MigrationError、ProgressFaultError、ProgressFaults、12 variantのProgressCheckpoint、正常Okを返すNoProgressFaults、新しいpreserve helper2署名、private save_with_limit_and_faultsを追加した。preserveは固定MigrationError::Io、and_faultsは固定文字列Errであり、publisher/faultを呼ばない。
- commit.rs: private save_with_limits_and_faults署名を追加し、固定文字列Errを返す。

既存save_with_limit/save_with_limitsは従来本体のままであり、新stubへ接続していない。Default/version/codec/recover hook、既存・新cfg(test)、integration期待値は変更していない。退避・fault接合の動作は未実装である。

targeted `rustfmt --edition 2021 --config skip_children=true --check src/store.rs src/commit.rs` はexit 0。12 checkpoint名、public fault型/trait、crate-private preserve、private publisher/and_faults署名と既存入口の接続維持を静的確認した。親のsnapshotと比較し、両sourceの最初のcfg(test)からEOFまで完全一致を確認した。親指示に従いcargoは実行せず、compile・行動検証は未実施である。

| scaffold source | SHA-256 |
| --- | --- |
| src/store.rs | F7E3D7B3044195A1AFAA9DB2261BF39C6BF22A488DD3AC922F65A9F38A540C51 |
| src/commit.rs | 78DD797E730667773A5F96ECE5BBE0579892E3BBDDF1B9F6DCF436E7B2A61E9F |

source/build所有を親へ返却する。独立作者がcompile/REDを行い、本実装はその受領後の別指示とする。Mのprivate target=2試験を公開v2受入やS/R完成とは扱わない。実データ・API・UAC・公開・再委任は行っていない。

## 独立RED後の本実装

同日、親から独立21件の5 PASS / 16行動REDと所有移譲を受領し、所有本体2本だけを実装した。wordweave-changeの工程境界に従い、作者の期待値と旧G/既存testを維持した。

- store: guard後のtarget=1無I/O no-op、target=2の長さ・必須/重複version・現v1構造検証、raw bytesのcontent hash、root/祖先/backups/leaf安全検査、既存exact backup照合を実装した。異なる既存bytesを修復しない。Before → 必要な場合だけpublisher最大1回 → Ok/Err共通のbounded exact readback → Afterの順序と、失敗時backup保持・再試行を実装した。
- store/commit: 既存save本体をprivate and_faultsへ移し、公開wrapperはNoProgressFaultsを渡す。gate取得は各入口1回である。normal v1 saveは旧移行sourceを読まず、material saveの旧metadata/escaped intentサイズ検査を維持した。Default/version/codec/recover hook/他checkpoint/結果不明blockは変更していない。

### 実行結果

集約compileは `cargo test --locked --lib --test audio_history_migration --test audio_history_gate --no-run` の1回だけである。exit 0、2m13s。以後は同一EXEを直接実行し、通常非管理者であることを確認した上で、必要なtest実行だけsandbox外で行った。UAC・権限昇格・OS設定変更はない。

| 固定EXEの引数 | PASS | FAIL / ignored | 所要時間 |
| --- | ---: | --- | --- |
| lib: `store:: commit:: --nocapture --test-threads=1` | 43 | 0 / 0 | 1.59s |
| migration: `--nocapture --test-threads=1` | 3 | 0 / 0 | 0.54s |
| gate: `--nocapture --test-threads=1` | 16 | 0 / 0 | 4.80s |

新Mはlib18+integration3=21件すべてGREENである。lib43にはG8/旧17を含む。REDで未到達だったpublisher書込後Err、readback拒否、After fault後retry、安全path/版表、M05末尾assertまで到達した。既存backup・新規backupの両方で実際のWindows共有違反 `Some(32)` を記録した。fixturefailure/timeoutはない。実行後のtemp直下 `ww5-i2-gate-*` / `ww5-i2-migration-*` 残存数は0であり、追加削除は行っていない。

ログは `artifacts/history-migration-implementation-{build,lib,integration,gate}.log` に保存した。targeted rustfmt checkと `git diff --check -- src/store.rs src/commit.rs` は成功した。既存dead codeとincremental hardlink fallbackのwarningは残るがcompile errorはない。

### 固定hashと再build不要の実行物

| 対象 | SHA-256 |
| --- | --- |
| src/store.rs | 364613CB7E769A05D63BEAB7D735A94DE68570CF1213AA344EE77E7E01DBB9EF |
| src/commit.rs | D70D1CC065409B526789A6991ACDB6876F9782B4D94C67396AC445D55B088A1D |
| store M cfg(test) | E6959959DF9DBD3FF2DCFBC8865E43DF447682088F21FA83BD45888ADC4D5CF1 |
| commit M cfg(test) | A3B4DABB2BCB9E509385D6E76B92FF2CAEA44E3BCC5F49EB61ABB09B962B4F9C |
| tests/audio_history_migration.rs | 862FE8A4DD0A63E4254F77315E1AE12AA596AF2D3A3F4C2ACE7FCA0225BB84EF |
| tests/audio_history_gate.rs | 5A9706DCDB41156119D4CB014692E5FF7748865950A254A32F0C7D1093B86DCC |
| target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | 0D713105A427DE5EBF2CF148DB27324ECD6157C2286DC4E25F1F4D2828554620 |
| target/debug/deps/audio_history_migration-33b6a419850044f4.exe | 2051BAC17CDD51EBFFF5ABBD06DDDC6A05343F1E1583282AE297BE810E7EBEAE |
| target/debug/deps/audio_history_gate-0c0e7e180bf0149a.exe | ECD1A9A4690DA33B83BEE0A72C8F590E4BA3C6009AF67FF4635C10EFFDC35A20 |

上記EXEはすべて `F:/Kazuhiro/GitHub/WordWeave5/` を基点とする。M cfg(test)は最終 `#[cfg(all(test, windows))]` からEOFのUTF-8 bytesであり、作者固定hashと一致する。旧test/Gは最初の `#[cfg(test)]` からM module直前までを親snapshotの同開始位置からEOFと末尾空白を除き比較し、2本とも一致した。snapshotは上書きしていない。

source/build所有を親へ返却する。独立review/runnerは未実施であり、最終受入はユーザーに属する。Sの公開v2 codec、Rのrecover接合、I10全体release検証は今回範囲外である。実データ/APIは使っていない。開始/終了usage counterと実行model metadataは未取得であり推測しない。
