# I1b-2 writer/recovery 設計レビュー

2026-10-07。計画指定 reviewer `gpt-6-astra/high`、独立read-only担当 `history_writer_review` による確認である。実行metadata・使用量counterは未取得。

対象は `design-i1b.md` のI1b-2追補。対象SHA-256は `88C2DCC679630883AE72FF9061FAB83C6BB5AA3D7E1052F7B208DEE7D920E560`。確定した受入阻止指摘は0件であり、実装へ渡せる。実装・動作試験合格ではない。

- receiptとbaseの照合順序、Run開始/終端の別OperationId、同対象競合を確認。
- intent確認→不変publish→閉包検証→I2 pointer保存/再読→Saved→finishの順序を確認。
- 孤立object・旧root・intent・残存tempの容量計上、部分publish再試行の二重加算防止、回復時の計画照合を確認。
- 既存readerの同RunId旧Running/新終端の保持契約と衝突しないことを確認。

親の統合確認でintent上限12MiBと承認済み512KiBの不一致を先に指摘し、designerが512KiB・超過無書込へ訂正した。その修正版を独立レビューした。上限緩和や検証省略はない。

残る検証はW01–W08の独立実装試験、I2全writer直列化/pointer前後障害、I6保存再試行時の解析・POST増分0、Windows実disk障害・電源断耐久性である。設計レビューをこれらの成功証拠にしない。

レビュー後、test authorからprepare/publishの責務確認があり、I0署名どおりprepareが全事前検証とintent保存/読戻し、publishがobject公開/閉包検証を担当すると追補へ明記した。新しい保存順・利用者動作の追加ではない。上記SHAはこの明記前の独立レビュー対象である。
