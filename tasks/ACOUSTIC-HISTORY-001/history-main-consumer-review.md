# I2b-A 独立コード確認・runner前判定

2026-10-11 / reviewer `main_consumer_review`、計画者指定 `gpt-6-astra / high`。親がread-only担当の結果を記録した。基準HEADは `21c304afd86ae5c4a2af96be881e772dbdca86c8`。実行model/effort metadataと使用量開始終了counterは未取得である。

## 判定

I2-T02 / AH-AC04の限定差分について、受入を阻害する確定欠陥・未解決質問は検出しなかった。指定Sol/mediumの独立runnerへ進める。**最終Post／Closedではない**。後続の20件独立実行・最終レビュー・I10全体gate・release/native/API受入は未了である。

## 確認した証拠

- `src/app.rs` の開始終了SHA-256はともに `C1706E880A27C44BDD205D1BA3FD0E01E8D7D43FF0D3F8543F3D62AE251EDA0A`。本体差分はpersist/import_deck/restoreの3箇所である。
- 保存APIが返すProgressを採用してから成功処理へ進む。failure時のcandidate未採用、pending保持、旧snapshot非書戻しを維持し、先行root読取・手動mergeを加えていない。
- 成功oracleはdisk/UI全Progressを独立期待値と比較し、教材案・会話・日本語下書き・設定・当日回数を含む。拒否oracleは既存fileのexact bytes、UI/deck/dirty/epoch/message/last_saveを確認する。復元前の既存補助退避追加だけを許容する。
- pending作成後・metadata書込前の拒否と後続persist2回の非書戻しを検証する。finish後半障害の証明へ拡張しない。
- HEAD本文と現在の追加testの再構成SHAは独立作者版 `44961B13D8F98D317CFF35D17C1414E14804610B8F75DAD3709312D46D79E401` と一致し、期待値は不変である。
- 原ログから作者6行動RED/10防御PASS、実装者の新16件＋既存4件PASSを照合した。固定EXE `target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` のSHAは `C6143EA4833832FE66B0F0F0B938A7EBE69EA8D2C6E2ADAF474AA1772CD932BA` と一致した。
- fixture markerは作成16/削除16/残留0、実装用専用TEMP子要素0である。初回の環境由来fixture失敗と無出力試行を成功へ変更していない。

具体的な対象・再現コマンド・ログは[作者記録](history-main-consumer-tests.md)、[実装記録](history-main-consumer-implementation.md)を正とする。backup helperのroot正規化はb-B、その他consumerは後続単位であり、本判定の対象外である。

## 独立性と残り

reviewerはファイル編集・build・test実行・mutation・公開を行っていない。read-only制約を優先し、Skillのmutation実験は実施せず、変更前REDとoracle不変の証拠を確認した。新たな独立実行に代えて既存成功ログを使ったという意味ではない。次のrunnerは固定source/EXEを自分で実行し、20件の実名・件数・前後hash・専用一時領域の後始末を確認する。上限は各job50秒、作者/実装者と同じprocess限定workspace TEMP/TMPとHidden起動を用いる。元ログを上書きせず、失敗時は原因と未実行をそのまま報告する。

利用者のモデル切替前停止指示に従い、親はここで `gpt-6.1-sol / medium` への切替を案内する。独立runner後の最終Postは既定 `gpt-6-astra / high` である。
