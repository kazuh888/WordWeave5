# 工程別自動引継ぎの使用量観測

2026-10-12。通常開発から評価する初期観測である。比較用再実行・品質基準変更はない。方法は[開発体制](../../docs/process/development-team.md)と[KPI](../../docs/process/improvement/kpi.md)を正とする。

## I2d-Mテスト作成・実装・検証の開始

2026-10-12 02:14:41 UTC（11:14:41 JST）、対象HEAD `a64ede0`、tree clean。週枠usedPercent=93、windowDurationMins=10080、resetsAt=1792298129。今回の完了境界は独立作者→本体実装→独立runner→最終Postであり、試用EXEやI10全体完了とは区別する。モデルは作者/実装/レビューAstra/high、runner Sol/mediumを既定どおり明示してdispatchする。

親の開始usage snapshotは02:14:58.848 UTC、input=1530333655、cached=1493809536、output=3506519、reasoning=1063701、total=1533840174、cache_write=0。着手からsnapshotまでの区間は差分に含まれない。親子包含は未確認のまま別記録する。作者 `media_export_tests` をfresh contextで起動した。終了snapshotと各子sessionの範囲は工程完了時に追記する。

### 作者RED工程のcheckpoint

02:36:47 UTCの週枠は96%（同じ窓・reset）であった。開始93%との差はアカウント共通の観測であり工程単独消費へ換算しない。新25件は15行動RED/10PASS、fixture不成立0、compile blocker0。既存consumerの保護先不正受理を検出し、独立hash fixture不成立を修正、unwind assertionを強化した1件だけ再実行した。実装側は固定Errの入口のみであり未完成である。今回のcheckpointは利用者承認済みの途中保存であり、GREEN/最終Postや製品配布ではない。

作者session `01a12de4-4b0b-7561-a325-9b8bca07594c`（Astra/high指定）のusage53イベントを取得した。最初02:15:11.177 UTCはinput=37818/cached=23040/output=244/reasoning=0/total=38062、最後02:37:01.903 UTCはinput=5906085/cached=5668992/output=32023/reasoning=6117/total=5938108。差分はinput=5868267/cached=5645952/output=31779/reasoning=6117/total=5900046である。初回requestは差分外、末尾応答反映の完全性と親包含は未確認である。

実装担当session `01a12df0-346e-7f83-b56a-f4734b1c4b5d` はstub→本体の同じ担当へ引き継ぐ。工程終了前なので最終集計は後で行う。比較用試験やモデル変更はなく、担当Astra/high・runner Sol/mediumを維持する。

### I2d-M完了時の観測と評価

終了snapshot時計02:58:17 UTC（11:58:17 JST）。開始02:14:41から43分36秒、最終文書保存/commit/push/応答はこの計測後である。週枠は93→98%、windowDurationMins=10080・resetsAt=1792298129不変。アカウント共通・丸め/反映遅延/他作業内訳未取得のため、本作業だけの5ポイント消費とは断定しない。

親終了usageは02:57:55.170 UTC、input=1542661724/cached=1505991936/output=3524442/reasoning=1067676/total=1546186166。各子は以下の実在snapshot間を観測した。cache_writeはすべて0。cachedはinputの、reasoningはoutputの内数である。

| 担当・指定model/effort / UTC観測区間 | input差分（cached内数） | output差分（reasoning内数） | total差分 |
| --- | --- | --- | --- |
| 親 Sol/medium、02:14:58.848→02:57:55.170 | 12328069（12182400） | 17923（3975） | 12345992 |
| 作者 Astra/high、02:15:11.177→02:37:01.903 | 5868267（5645952） | 31779（6117） | 5900046 |
| 実装 Astra/high、02:28:08.065→02:47:13.515 | 2064651（1950080） | 9064（704） | 2073715 |
| runner Sol/medium、02:47:44.551→02:56:20.838 | 2809560（2746496） | 11350（1454） | 2820910 |
| Post Astra/high、02:49:55.977→02:57:55.106 | 1647449（1478656） | 3746（342） | 1651195 |

親子包含は未確認なので行を合算しない。子の最初のrequestは差分に含まず、最終応答反映の完全性も未確認である。turn_contextの指定は各dispatchと一致したがプロバイダー返却model/effortではない。実装はstub/本体2turn、Postは静的/runner照合2turnを同sessionで継続した。

末尾counterと対応：

- 作者 `01a12de4-4b0b-7561-a325-9b8bca07594c`、usage53件、上記checkpointの開始/終了値と同じ。
- 実装 `01a12df0-346e-7f83-b56a-f4734b1c4b5d`、usage32件。開始input37722/cached0/output167/reasoning0/total37889→終了2102373/1950080/9231/704/2111604。
- runner `01a12e02-2ac8-7332-ab10-3a52bfa6b9c0`、usage40件。開始38151/37248/142/0/38293→終了2847711/2783744/11492/1454/2859203。
- Post `01a12e03-feab-79e3-b89f-d7760236d12c`、usage19件。開始38028/37120/337/10/38365→終了1685477/1515776/4083/352/1689560。

成果は新25件15RED/10PASS→25GREEN、[独立114行動PASS/0FAIL/条件不足3](media-export-runner.md)、[最終Post PASS/Closed](media-export-final-review.md)である。ソース2file・独立oracle不変、製品EXE未更新。通常は補助1役、固定sourceのrunner build/runtimeとread-only静的Postだけを独立性・待ち時間削減のため並行した。常駐router・新定義・再帰委任はない。

手戻りは作者のhash fixture修正、unwind assertionの強化1件、実装側の不要なcargo再compile1回と古いEXE hashでの起動前拒否である。既知sandbox制約を実装側で再実行した点は効率改善余地であり、runnerでは再試行せずfresh compile/固定EXEの非管理者runtimeを分離した。比較のためだけの再実行は0、品質gateは変更していない。checkpoint pushはREDと本体GREENの2段階で実施した。

**評価：自動引継ぎと担当別計測は実施できたが、旧方式との使用量増減は未判定である。** この単位は独立fixture/新API scaffoldを含み、前回の設計単位とも条件が異なる。親のusageには接合・記録・checkpoint・待機後の会話継続も含む。低リスク改善として、既知のsandbox実行制約を後続担当へ明示し、不要な再compile/同条件再失敗だけを避ける。必要な独立試験やレビューを削減した値とは比較しない。

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

## 今回の区切りと初期評価

終了時計02:02:00 UTC（11:02 JST）、週枠はusedPercent=93、窓とresetsAtは開始時と同じであった。表示上は91→93の2パーセントポイント増である。他のアカウント作業の有無・反映遅延・丸めの内訳は取得できず、本単位だけの消費と断定しない。最終の記録/commit/push/応答は終了計測後である。

親の終了usage timestampは02:01:33.454 UTC、累積input=1528687828、cached=1492180608、output=3498408、reasoning=1062452、total=1532186236、cache_write=0である。

| 対象・観測区間UTC | input差分（うちcached差分） | output差分（うちreasoning差分） | total差分 |
| --- | --- | --- | --- |
| 親 01:47:21.611→02:01:33.454 | 4931676（4894080） | 10332（2592） | 4942008 |
| designer 01:47:21.839→01:59:08.118 | 1755903（1658496） | 15526（1789） | 1771429 |
| planner 01:51:55.051→01:55:03.101 | 428234（377600） | 6069（677） | 434303 |
| Pre reviewer 01:59:36.874→02:01:27.266 | 569638（513792） | 2757（490） | 572395 |

各子は最初のusage snapshot以降の差分であり、その最初のrequestは含まれない。親子包含未確認のため表の行は合算しない。cached/reasoningは内数である。designerは同じsessionの初回設計と計画delta後の限定反映の2turnを含む。

子の終了値と識別（全てcache_write=0、turn_contextはAstra/high指定と一致）：

- designer `01a12dca-d6b3-7581-bb76-143ca9cd099a`、usage19件。最後input=1793671、cached=1681536、output=15737、reasoning=1789、total=1809408。
- planner `01a12dce-fb43-72e2-84b2-9e5af7f0a6cd`、usage7件。最初input=37752、cached=23040、output=254、reasoning=0、total=38006。最後input=465986、cached=400640、output=6323、reasoning=677、total=472309。
- reviewer `01a12dd6-02b4-7cd0-b79c-b20f931e830d`、usage9件。最初input=37756、cached=37120、output=273、reasoning=0、total=38029。最後input=607394、cached=550912、output=3030、reasoning=490、total=610424。

成果は既存7役の自動dispatch運用・計測手順の正式記録と、I2d-MのM-D01発見→2file計画delta→限定設計→[独立Pre PASS](media-export-review.md)である。本体・テスト・EXEは未変更。比較だけの再実行は0回で、文書リンク・diff検査を行った。

**評価：通常開発から親と担当別のusage差分を取得できた。自動引継ぎの使用量増減は未判定である。** 初回は運用整備・計測導入の作業と設計の是正が混在するため、今後の純粋な工程に対する対照にはしない。親の観測にも文書整備・引継ぎ・待機後の会話継続・記録収集が含まれ、「引継ぎだけ」の値ではない。改善候補は引継ぎ資料・文書の再掲を短くし、親の必要な接合だけを残すことである。モデル/effortや品質gateを下げた比較は行っていない。
