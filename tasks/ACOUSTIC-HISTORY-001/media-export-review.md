# I2d-M 限定設計レビュー

2026-10-12、独立reviewer `media_export_pre`、計画・起動指定 `gpt-6-astra/high`。**Pre PASS、阻止指摘0件**。M-D01は設計上解消できる。これは実装ClosedやPost PASSではない。

対象は[限定設計](media-export-design.md) SHA256 `F6A1D38ABEFEC370FAF14835BFDF48D40B4F76A6E7321F889E1AD028B1B1F49F` と[計画delta](implementation.md) I2d-M節。HEAD `b02d87f`、source2本のhash一致、`b4f100c`以降が運用文書だけであることを確認した。レビュー担当は編集・test/build・再委任をしていない。

- 公開するのはborrowed bytesの同期操作だけであり、private guard/errorを公開せず別crate問題を解消できる。既存under_guard署名・返却PathBuf・same_planned_nameの可視性と整合する。
- 全member/completionの形式・大小文字衝突を作成前に拒否し、解決済みdirectoryを基点に各file/tempを再検査する。marker最後はH1/H4と整合する。
- 実Storage優先、None時resolverの適用限界、呼出前取消し、Err時clearなし、途中成果物を残して別の新規destへ再試行する契約を維持する。
- M-AC3はlibrary途中Err/marker欠損の動的試験とbinaryの状態試験・借用/`?`後clearの独立静的確認を合成できる。第二memberのdirectory化は途中の境界拒否であり、実atomic失敗ではない。既存H04/H07の実体も確認した。

未検証は新試験の実行、実装上のguard連続保持、native picker、OS結果不明時の耐久性であり、作者→実装→独立runner/Postへ残る。確定欠陥・要回答質問・任意提案は0件。runtime model/effort返却値は未取得である。ログの識別はthread `01a12dd6-02b4-7cd0-b79c-b20f931e830d`、親session `01a08550-055c-7aa1-9834-905e4a7ce148`。usageの取得範囲は[通常開発観測](usage-observation-20261012.md)を参照する。
