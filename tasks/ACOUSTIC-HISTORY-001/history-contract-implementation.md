# I1a共有履歴契約の実装

2026-10-04 / ACOUSTIC-HISTORY-001。親から承認済み[実装契約](design-i0.md)4節と[独立RED](history-contract-tests.md)を引き継いだ。指定はww-implementer / gpt-6-astra / high、実行metadataおよび使用量開始・終了カウンターは未取得である。wordweave-changeとTDD skillを担当範囲で適用した。

変更は `tools/practice-history/src/{lib,ids,model,validate}.rs` と本書である。manifest/lock/tests、他担当の解析コードは変更していない。ID/hashの正規構文、参照byte上限、日時/入力/Unicode scalar数、schema、run終端状態、助言構造、trialの確認出所とrootの保存順/receiptを再帰的に検証する。入力値を正規化・切詰め・補完せず、エラーは静的field名だけを持つ。Trial 256KiB/root 8MiBはJSON全体を割り当てない上限付きwriterで検査する。

焦点検査の実績はID/hash/Blob/Rootの4件成功、値境界19件成功、最終契約35件成功である。作者のREDは5成功/30失敗であり、期待値は変更していない。

| コマンド | 結果・証拠 |
| --- | --- |
| `cargo test --manifest-path tools/practice-history/Cargo.toml --test contract --locked --offline` | 35 passed / 0 failed / 0 ignored / 0 filtered。[ログ](artifacts/history-contract-green.log) |
| `cargo test --manifest-path tools/practice-history/Cargo.toml --all-targets --locked --offline` | 35 passed、unit 0。[ログ](artifacts/history-contract-all-tests.log) |
| `cargo fmt --manifest-path tools/practice-history/Cargo.toml -- --check` | 成功 |
| `cargo clippy --manifest-path tools/practice-history/Cargo.toml --all-targets --locked --offline -- -D warnings` | 成功。[ログ](artifacts/history-contract-clippy.log) |
| `git diff --check`、srcのtodo/unimplemented検索 | 成功、stub残りなし |

独立 `tests/contract.rs` のSHA-256は `328879AC726D98323ADDA771608C29E44CD23F2F6A3122DC50275EB3EE8CEF91` であり、作者のRED記録と一致する。

残留事項: 依存する解析profile DTOがserde未知fieldを破棄する点を親へ返した。親はI4の独立RED・修正として対応予定である。I1aは `AnalysisProfile::validate` へ委譲し、依存先を修正していない。実bytesの長さ制限・hash照合・参照閉包、助言引用と固定本文の照合、provider固有のfailure allowlistはhostの責任である。native操作、永続化、実API、全アプリreleaseの合格は主張しない。実データ・credentials・live AIは使用していない。

本体4ファイルの所有を解放する。最終検証はtest runner、受入は利用者に残る。
