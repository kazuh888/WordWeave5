# ET1開始前提とI2-D2集約確認

2026-10-13。独立reviewer `trial_dependency_gate`（gpt-6-astra/high指定）がHEAD `186b46433801b0bc953eeaebf15b297bf07cd577`をread-onlyで照合し、**I2-D2 aggregate Closed可、ET1独立作者へ進行可、阻止指摘0・要回答質問0**と判定した。親はこの限定判定を受領しD2を閉じる。コード変更・runtime試験・既存成功試験の反復・再委任はない。

## 根拠インデックス

- [実装計画](implementation.md)のD2条件にあるsettings4出口、backup directory、媒体3出口/構成fileを限定検索した。settingsはsettings_ui::write_settings_export/control_settings、媒体単体は同helper、一式はstore::write_external_bundle、backupは同一guard/解決済みdirectory/子file/最終markerへ接合し、旧atomic_write等によるexport迂回は残っていない。
- [H最終Post](export-destination-review.md)、[S最終Post](settings-export-review.md)、[B最終Post](backup-destination-review.md)、[M最終Post](media-export-final-review.md)の証拠継続性を確認した。S/B/M現source hashは各Postと一致し、H確定後のstore追加は承認済みMのbundle/名前検査/専用試験だけで、既存identity/prepare/recheck/publishは無変更である。
- Hのsymlink2件は過去の独立成功を既存境界の証拠として維持する。M再実行時のskip2件を今回成功へ数えない。別drive alias条件不足1件は未実証として残り、条件付き受入のためaggregate閉鎖を阻止しない。
- 実Storage優先、None時の現行既定起動resolverを維持する。任意カスタムroot失敗/起動後環境変更から元rootを再現する保証はなく、閉鎖のためにその保証を追加していない。
- [ET0 cycle2仕様/設計PASS](early-trial-review.md)、[計画](early-trial-plan.md)、契約SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、d-S/B/M Closedが揃った。ET1/2関連sourceの追加は上記Mだけで、設計再検討を要する差分はない。**2root保護はET1でこれから実装する事項**であり、D2閉鎖を試用分離の実装済み証拠にはしない。

## 次と残る境界

ET1作者Astra/high→実装Astra/high→独立runner Sol/medium→Post Astra/highへ進む。共有sourceの作者/実装者所有は逐次移譲する。ET2起動/資格情報/識別表示、ET3全体test/release/ローカル提供は未完了である。native操作、敵対的TOCTOU、別drive条件不足、機能全体の受入を本判定で成功扱いにしない。

reviewer thread `01a13514-2714-7ae0-81a8-0953439c52f0`、親session `01a08550-055c-7aa1-9834-905e4a7ce148`。指定とruntime返却値は別で後者は未取得。usageは[今回観測](usage-observation-20261013.md)へ担当別に記録する。
