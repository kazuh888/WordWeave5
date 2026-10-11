# I5-M1 独立コード確認・残る受入

2026-10-07。reviewer `praat_m1_review`、計画指定gpt-6-astra/high。read-only、試験再実行・編集・再委任なし。親が返却判定を保存した。実行metadata・使用量counterは未取得である。

## 判定

M1限定の最終Post-gate PASS、I5-P1 Closed。2026-10-07、同じ独立reviewerが利用者GREEN証拠を追加確認し、source/test/EXE/childの現物hashを再照合した。新規阻止指摘0件。以下のコード確認と既存回帰結果を維持する。

本体 `process_windows.rs` の開始/終了SHAは `10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C`、独立testは `CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896` で一致した。関連3本体のhashも既存記録と一致する。

確認した境界：2属性と両配列のDeleteまでの寿命、失敗途中の属性/handle回収、非継承専用Jobへの作成時所属、成功直後assigned=true、stdin/stdout/stderr以外を継承しないこと、専用Job指定の所属照会、照会Err時Job回収維持、明示falseだけ直接終了、resume前失敗・取消し・期限・unwind。無制約fallbackやM1契約違反を認めない。

独立試験は外側Job不在・cleanup前の終了観測固定・sentinel不在・兄弟生存・watchdog非救済を確認する。救済をPASSへ読み替えない構成である。runner記録とログで最終53成功/M1 1filter、実Praat20成功、default59成功を確認した。通常sandboxのevent異常終了のOS内部原因は未確定であり、sandbox外成功だけで原因確定とはしない。

旧sourceの退避／M1限定diffは未提供であり、変更範囲を完全に差分で証明する点には限界がある。旧hashと編集前の読取、実装報告、現物契約適合を区別する。今後の未追跡本体編集では編集前snapshotを保存する改善余地があるが、今回既存証拠を作り直さない。

## 最終受入・残る範囲

[利用者GREEN](praat-m1-user-green.md)により、外側Job不在・作成時所属・親強制終了後のcleanup前終了・未resume・兄弟生存・watchdog非救済がすべて成立した。M1固有の受入阻止条件は解消した。利用者実行時のhash・shell版・昇格状態・正確な時刻は未取得であり、ローカル現物同一性と区別する。既存53件と別実行M1 1件を区別して記録し、一括54件成功とは記さない。Q1/M2・writer・全体I5・アプリ納品検証は未完了である。
