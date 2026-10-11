# I2a-0 Path reader 実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。計画指定 `ww-implementer / gpt-6-astra / high` を維持する。実行metadata・開始/終了使用量counterは未取得である。AGENTS.md、development-team、wordweave-changeは本担当の先行工程で読了済みであり、今回も所有制約と回帰工程を適用した。

親からI1b-2b Post PASS・R1/R2 Closedを受領し、implementation.mdのI2接合追補・first dispatchに基づく最小scaffoldだけを追加した。設計入力SHAは `B8B1FD5FEA1518C4AE516E0B47C9502267A0C05A5C2EF4EA4A979282E8267D51` と一致する。

変更は `src/audio_history.rs` の `pub(crate) fn HistoryReader::for_root(_root: &'a std::path::Path, _limits: HistoryLimits) -> Result<Self, HistoryError>` のみである。固定 `Err(InvalidDocument)` を返す未実装入口であり、副作用はない。既存new/with_limits、Storage保持、closure、テスト期待値を変更していない。

`rustfmt --edition 2021 --config skip_children=true --check src/audio_history.rs` はexit 0。本体SHA-256は `3CFB86BCF790360B2EF9C3222839AD21BDEDED0565B7235BA7DDBAD452CD4969`。親指示に従いcargo build/testを実行せず、compileと行動は未検証である。

本体所有を親へ返却する。次に独立作者が同fileの専用cfg(test)を所有する。動作実装は独立RED受領後の別指示まで行わない。実データ・API・UAC・公開・再委任はない。

## 独立RED受領後の本実装

親から独立11件のRED（2対照PASS / 9 Path scaffold由来の行動RED）と本実装指示を受領した。対象は本体の `src/audio_history.rs` と `src/audio_history/closure.rs` の指定2参照だけである。

HistoryReaderのprivate保持を `&std::path::Path` とし、for_rootへ従来のlimits検査を実装した。newはStorage.dirを借用して既定limitsを使い、with_limitsはfor_rootへ委譲する。公開署名は維持した。closureのhistory root組立てと祖先検査用absolute pathは同じroot参照に接続した。既存のbounded read、hash/長さ/DTO/関係検証、ancestor/root/assets/leafの安全検査は変更していない。Storageの偽造・再取得・recover・writeは追加していない。後続I2a-Gのgate設計・実装は含めない。

専用cfg(test)と既存integration testは変更していない。以下の前後hashで確認した。

| 対象 | 着手前SHA-256 | 完了SHA-256 |
| --- | --- | --- |
| src/audio_history.rs | CC43B6D4D4781C540BCA649931DD73A15B8E7238035F7CD20791B4E23ED03FC2 | CB47D4078A775CD6FFEC666F306FC4C5B0657FE5B8A7146DD65D76252865AFC0 |
| src/audio_history/closure.rs | 44D8F94AE6D8D56C9743803D51F00DFEFADF11F577B8866ABFC001463192432D | 65ACB71BC70CE4D2EC34A0A35242D8496D77D6B2456904AED27D4D503DCA05EE |
| 専用test module（cfg属性からEOF、UTF-8） | 6A2F382ED66A2F661C9D065269D4383F2340F82E3C02E096FE9D3EAA17D40437 | 同値 |
| tests/audio_history_closure.rs | B2FEC0ED2E71D60271365DF26C224392C217845382D5D8D59C1DCFD247DC44C1 | 同値 |

### 実装者の焦点検証

`cargo test --locked --offline --lib --test audio_history_closure --no-run` を1回実行し、exit 0、2分05秒でcompileした。[buildログ](artifacts/history-path-implementation-build.log)。既知のincremental hardlink/copy警告と既存binのdead-code警告がある。以下は生成された同じ固定EXEの直接実行であり、テストごとにcargoを再実行していない。

| 対象・引数 | 結果 | ログ |
| --- | --- | --- |
| lib EXE `audio_history:: --nocapture --test-threads=1` | 11 passed / 0 failed / 0 ignored / 134 filtered、exit 0、0.10秒 | [focused](artifacts/history-path-implementation-focused.log) |
| closure EXE、引数なし | 37 passed / 0 failed / 1 ignored / 0 filtered、exit 0、5.20秒 | [既存reader回帰](artifacts/history-path-implementation-reader-regression.log) |

実行時にWindows tokenがAdministratorでないことを検査し、必要なtest EXEだけscoped sandbox releaseで動かした。新しいsymlink権限・UAC・設定変更はない。既存leaf file symlink 1件は明示ignoredのまま未実行であり、以前の成功を今回へ算入していない。新junction 4件は今回の11件内ですべて末尾の不変assertionまで成功した。

初回fmt checkはclosureの変更行折返し1箇所を指摘したため、その所有行だけ整形した。最終targeted `rustfmt --edition 2021 --config skip_children=true --check src/audio_history.rs src/audio_history/closure.rs` と `git diff --check` はexit 0である。

runner用固定実行物は以下である。

| 絶対path | SHA-256 |
| --- | --- |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/wordweave5-b3388a7bbef44bb4.exe | 9D7E8F200475F68ECC7C0964BBFE4EADDE2E6C9F7811DAC7F5A5189CF3690095 |
| F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/audio_history_closure-5710f60171e2013d.exe | F08C2300BA3D420663B403514E06FD57C6105F16196308F16D5810E9F6C5A1F4 |

本体・build所有を親へ返却し、sourceを固定する。残るのは独立runner/reviewer、親が必要性を判定するwriter/recovery回帰、I2後続接合である。I10全体test/release、実機受入、権限試験1件の今回実行は行っていない。KPI開始/終了counterと実行model metadataは未取得のままであり、推定しない。
