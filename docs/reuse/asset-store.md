# 添付原本管理の再利用境界

2026-10-03 / FRAMEWORK-DESIGN-001 / L02。不変原本の検査・保存・参照に限る設計案である。独立配布・二消費者受入は未実施である。
[全体設計](README.md)、[Framework](framework.md)、[候補台帳](library-candidates.md)、[文書仕様](../../tasks/FRAMEWORK-DESIGN-001/spec.md)を前提とする。
「現行」は確認したソース、「proposed」は未実装の目標境界である。API名・namespace型・保存profileの確定ではない。

## 1. 根拠と責任

| [assets.rs](../../src/assets.rs)の入口 | 現行の事実 | 共通化で維持する範囲 |
| --- | --- | --- |
| AssetKind / AssetRef | kind、SHA-256小文字hex ID、bytes。serde snake_case | 正本bytesへの参照。教材/回答/会話の意味を持たせない |
| AssetStore::new / put / read | host root配下assets、形式/サイズ/hash検査、無置換publish | host指定root内の不変原本と整合性 |
| inventory | detached原本も含め検証、.tmp-*は除外 | 参照除去を原本削除へ変換しない |
| export_to | 原本事前検査、新規dest、manifestを最後に保存 | hostのexport adapter候補。backup全体や学習記録のtransactionは外 |
| sha256 / no_link | Windows CNG、非Windowsはunsupported、リンク検査 | 初期Windows契約。hashを権限・完全な競合耐性の証明としない |

依存は「host repository → 原本store → filesystem/hash」である。WordApp、Entry、Progress、AI評価、eguiを必須依存にしない。
画面は持たない。ファイル選択、表示名、添付選択解除、preview、送信確認、学習記録、保持期間/GCはhostが所有する。
原本とはstoreに渡されたbytesである。録音のdownmix前データ、画像codecの完全性、inkの意味まで保存入口が保証するわけではない。
原音のresample、画像縮小、OCR、inkの画像化は派生物である。hostが原本参照・変換版・派生物参照を対応付け、原本を置換しない。

## 2. 最小窓口・形式（proposed）

既存AssetKind/AssetRefとput/readを中心にする。任意path操作、画像decoder、汎用repository trait群を公開面へ加えない。

| 操作 | 入力と境界検査 | 出力・禁止する解釈 |
| --- | --- | --- |
| put | 明示kindと所有/借用bytes、空/上限/形式検査 | 検証済み参照、または分類した失敗。参照保存・送信同意の成功ではない |
| read | 選択済みstoreとAssetRef、ID/kind/長さ/リンク/hash | 検証した原本bytes。preview・全文decode・AI入力適合とは別 |
| inventory | 選択済みstore、hostが走査を要求 | 検査済み参照。順番・添付履歴・所有者一覧の保証はしない |
| export | 明示destと参照集合、既存destへの上書き拒否 | 全原本確認後の完了marker。部分dirがないことまでは保証しない |

現行kindはAudioWav、PNG/BMP/GIF/JPEG、InkJson、FileBlobである。拡張子はkindから導出し、元pathを参照IDにしない。
AssetRef.idは64文字の小文字hex、bytesは正値u64である。kind・hash・bytesの組合せで読取対象を検査する。
現行上限は12MiB、WAV/画像はmagic、InkJsonはJSON object、FileBlobは非空bytesを検査する。magic一致だけで安全なcodec入力としない。
preview/全文decodeの寸法・展開容量・schema検査は利用側adapterが行う。Qwenの6MiB/PCM/長さ制約も[評価側](qwen-audio.md)で再検査する。
未知kind/schema、欠落、hash/長さ不一致は失敗として返し、空原本や別内容を補って成功にしない。

## 3. root・権限・データ所有

| 対象 | 所有者・許可境界 | 保護する条件 |
| --- | --- | --- |
| root / namespace | hostがinstanceごとに注入。参照解決前にhost権限を検査 | 別hostの保存先・資格情報・参照を暗黙共有しない |
| AssetRef | hostの会話/回答等のrepository | hashが分かっても読む/送る/削除する許可にならない |
| 原本bytes | hostの保管領域、storeは不変保持を実施 | 同じ参照へ違うbytesを上書きしない |
| 派生物と系譜 | host/変換adapter | 変換結果は別参照として保存し、原本との対応を記録 |
| 参照除去 / GC | hostが寿命・権限・backupを決定 | 添付解除だけで原音/ink等を削除しない。storeの基本APIにdeleteを含めない |
| 送信・適用 | hostが対象・宛先・用途を示し人の同意へ連結 | 保存済み/同hashを同意済みと見なさない。教材登録権限を持たない |

現行AssetRefにnamespaceはない。proposedではhostのnamespaceとstore bindingを外枠で対応付け、旧AssetRefのserde形式を黙って変更しない。
namespaceは保管境界の識別であり認証ではない。同一プロセスの悪意あるhostやfilesystem外部変更への完全防御とは主張しない。
現行のno_linkはroot/対象の検査であり、全祖先/並行path差替えへの完全保証ではない。切出し仕様でroot信頼条件とfilesystem受入を明記する。

## 4. 保存順序・競合・回復

現行putは「入力検査/hash → assets作成/リンク検査 → 既存ならread検証 → create_new一時file → write/sync_all → hard_link publish → read再検証 → 一時file除去」である。
hard_linkのAlreadyExistsでも既存原本をread検証する。存在確認だけで成功とせず、無置換publishで同内容の並行putを扱う。
directory syncや全filesystemの停電耐性は確認していない。hard_link非対応を上書きrenameへ黙ってfallbackしない。

| 状態/失敗点 | 残り得るものと回復 | hostが表示する確認済み範囲 |
| --- | --- | --- |
| 入力拒否・publish前失敗 | 原本は未publish。一時fileが残り得る | 保存失敗。metadataへ成功参照を書かない |
| publish後・再検証/応答失敗 | 原本が存在し得る。選択rootで同参照を検証して解決 | 保存結果未確定を区別。盲目的削除・上書きは禁止 |
| 原本保存成功・host参照保存失敗 | detached原本を保持し、同じ参照のmetadata保存だけ再試行 | 原本あり/添付記録未保存。成功を一括表示しない |
| 原本欠落/破損 | 参照を保持し、hostのbackupから別途検証復元 | 利用不能。別原本への自動差替えやAI再生成をしない |
| export途中失敗 | 新規destに部分原本が残り得る。manifest未保存なら未完了 | backup完了としない。再開/隔離/除去はhost仕様で決める |

hostの通常順序は「原本put成功 → 参照を含む記録保存 → 保存ackを表示」である。二保存のtransactionをstoreが提供したとは見なさない。
manifestは現行format_version 1の完了markerであり、copy原本の後にhostのatomic_writeで保存する。共通化ではWordWeave storeへの依存をexport adapter側へ残す。
inventoryで見つけたdetached原本を任意の教材/回答へ自動再attachしない。帰属判断はhost記録と人の操作に基づく。

## 5. 非同期・上限・失敗分類

現行put/read/exportは同期処理である。UI hostがworkerへ置く場合、固定bytes/参照とstore bindingを渡し、完了をoperation IDへ対応付ける。
取消しは待機/次段階の停止であり、進行中writeやpublishの未実行を保証しない。完了前取消しでも原本を消さず、検証して残存状態を返す設計とする。
metadata適用はcontrollerで対象の版/削除状態を再確認し、遅延成功を現在の別添付へ適用しない。再putは同じbytes/kindに限定する。
再利用版のbyte上限はhost仕様の承認者が用途ごとに定める。storeのhard ceilingと利用側制約を別にし、境界直前/一致/超過を契約試験する。
inventory件数/総量/走査時間、派生物容量、保持期間、GC、export再開policyは未決であり、初期切出しに不要なら追加しない。
機械判定の失敗は不正参照/形式、上限、未対応OS/filesystem、欠落、破損、保存/読取失敗、結果未確定を区別する案である。日本語説明と回復actionはhostが担う。
生path/原文/音声bytesを共通診断へ出さない。外部送信・遠隔取消し・AI結果再取得は非該当であり、送信adapterへ委譲する。

## 6. 互換・二消費者gate・test seam

旧assets/<hash>.<kind拡張子>とAssetRef serdeを互換profileとして保持する。切出しだけで再hash・原本変換・参照書換えをしない。
公開API版、AssetRef/manifest保存版、codec/教科schemaは別である。未知形式を推測せず、読取り拒否または仕様化された保存保持を選ぶ。
二消費者候補はWordWeave5とノートまたは音読ツールである。第二hostと参照寿命/root仕様は後続planner/仕様担当が選定する。
gate Aは実際の二hostが同一版をコピーなしで利用し、独立root/権限/metadata adapterで原本を保持・再読できることとする。
試験口は隔離filesystem、既存put/read/inventoryのhelper、publish/読み返し/metadata保存の失敗注入、namespace bindingである。
将来テスト作者は同bytes並行put、kind差、0/上限直前/一致/超過、偽magic、欠落/改変、リンク、hard_link拒否、publish後失敗を検証する。
参照解除後も原本あり、metadata保存再試行で原本改変0件、別rootの参照混線0件、manifest欠落exportの完了誤認0件を確認する。
モック結果を実filesystem障害・クラッシュ耐性・二製品実績へ読み替えない。原本保存だけで[Frameworkのgate B](framework.md)は成立しない。

## 7. AC対応と引継ぎ

| 文書AC | 設計証拠 | 実装/テスト作者への引継ぎ |
| --- | --- | --- |
| FW-AC02/04 | 1–3節、L02 | hostの帰属・権限とstoreのbyte整合性を分ける |
| FW-AC05 | 2・4–6節 | 参照/形式、無置換publish、metadata保存順、互換profileを固定 |
| FW-AC06 | 3–5節 | 原本保持、取消し後の残存確認、送信/登録権限の非保有 |
| FW-AC07/08 | 6節、本表 | 二消費者は未実施。今回の静的検査/独立レビューは[結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)へ |

未決は最終公開面、第二host、root信頼条件、上限、export/保持/GCの仕様である。後続仕様で決め、原本保護を緩めない。
本書はD30設計のみである。実装/試験追加/実データ操作は行っていない。起動指定gpt-6.1-sol/high、実行metadataとtokenカウンターは未取得である。
