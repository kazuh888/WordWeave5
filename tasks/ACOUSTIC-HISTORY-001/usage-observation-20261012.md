# 工程別自動引継ぎの使用量観測

2026-10-12。通常開発から評価する初期観測である。比較用再実行・品質基準変更はない。方法は[開発体制](../../docs/process/development-team.md)と[KPI](../../docs/process/improvement/kpi.md)を正とする。

## I2d-M設計開始時のsnapshot

- 対象HEAD：`b4f100c`、開始tree clean。
- 開始時計：2026-10-12 01:46:45 UTC（10:46:45 JST）。
- 週枠：usedPercent=91、windowDurationMins=10080、resetsAt=1792298129。アカウント共通の観測である。
- 親session：`01a08550-055c-7aa1-9834-905e4a7ce148`。
- usage snapshot timestamp：2026-10-12 01:47:21.611 UTC。着手直後の読取までの区間は開始差分に含まれない。
- 親累積：input=1523756152、cached_input=1487286528、output=3488076、reasoning_output=1059860、total=1527244228、cache_write_input=0。
- 親ログの直近turn_context：model=`gpt-6.1-sol`、effort=`medium`。turn_contextの指定情報であり、プロバイダー返却modelを確認したものではない。
- designer起動指定：`media_export_design`／`ww-designer`／`gpt-6-astra`／`high`、fresh context。起動成功、返却runtime metadataは未取得。

累積値は長期sessionの値であり、このタスクの消費量ではない。親子の包含関係は未確認であり合算しない。終了snapshot、子sessionの対応と範囲、所要時間、品質結果は通常の工程完了時に追記する。現時点では旧方式との使用量増減を判定できない。

### designerの終了観測

session=`01a12dca-d6b3-7581-bb76-143ca9cd099a`、生成時刻01:47:10.488 UTC。session_metaはparent_thread_id=`01a08550-055c-7aa1-9834-905e4a7ce148`、agent_path=`/root/media_export_design`、role=`ww-designer`であり、対象の対応を照合できた。これはusageの親包含を証明するものではない。直近turn_contextはAstra/high指定と一致するが、プロバイダー返却値ではない。

usageイベント12件。最初のsnapshotは01:47:21.839 UTC（input=37768、cached=23040、output=211、reasoning=0、total=37979）、最後は01:51:26.357 UTC（input=965844、cached=883072、output=7830、reasoning=579、total=973674、cache_write=0）である。両snapshot間の差分はinput=928076、cached=860032、output=7619、reasoning=579、total=935695。最初のrequestの使用量はこの差分に含まれず、最終応答反映の完全性は未検証である。

成果は限定設計とM-D01の発見である。binary/library境界により既存の1file所有では単一guardを使えないため、誤った本体実装へ進まず計画deltaへ戻した。ソース・テストは無変更。使用量だけでこの調査を不要な手戻りとは評価しない。所要時間はsession生成から末尾イベントまで約4分16秒であり、実行metadataに基づく厳密な課金時間ではない。

## 取得可能性を確認した過去2担当

直前I2d-Bの独立runner session `01a12d95-36c4-7801-a24f-1d47394e26dd` はusageイベント22件、末尾timestamp 00:53:11.449 UTC、累積input=1253964（cached=1205376）、output=6905（reasoning=577）、total=1260869であった。最終Post session `01a12d99-a35d-7521-a9a6-701754ab0262` はusageイベント8件、末尾00:55:22.437 UTC、input=528704（cached=459520）、output=3201（reasoning=196）、total=531905であった。

これは前の調査で取得した記録値であり、新たな作業開始0からの実測差分ではない。両担当の最初のusageイベントも既に約3.8万totalを持つ。親の同時点開始／終了と包含関係を確認していないため、親子合計・引継ぎだけの消費・週枠換算・旧方式比の削減率を算出しない。
