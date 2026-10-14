# ET1再開の使用量観測

2026-10-13 / ACOUSTIC-HISTORY-001。利用者の再開依頼により、媒体export完了後のD2集約gateと早期試用保存先分離ET1へ進む。完了境界はET1独立テスト・実装・独立runner/Postまで、ET2資格情報/表示とET3製品提供は後続である。通常版のデータや秘密を試験に使用しない。

## 開始snapshot

- 対象HEAD `186b46433801b0bc953eeaebf15b297bf07cd577`、tree clean。
- 時計11:43:45 UTC（20:43:45 JST）。週枠usedPercent=0、windowDurationMins=10080、resetsAt=1792496608。前回の窓とresetが異なるため、前回98%との差分を使用量へ算入しない。
- 親session `01a08550-055c-7aa1-9834-905e4a7ce148`。親usage snapshot 11:44:36.417 UTC：input=1545068307、cached_input=1508161664、output=3533748、reasoning_output=1068755、total=1548602055、cache_write=0。
- 親turn_contextはgpt-6.1-sol/medium指定と一致するが、プロバイダー返却値ではない。再開直後からsnapshotまでの読取/引継ぎ区間は差分外である。
- 前提集約reviewer `trial_dependency_gate` を既定gpt-6-astra/high・fresh contextで起動した。既存個別Postをやり直さず、D2全出口の迂回残置とET1依存集合だけを独立照合する。

親子包含は未確認のため合算しない。担当別開始/終了counterと取得範囲を工程の区切りに追記する。比較だけの試験、常駐監視、新しいモデルrouterは追加しない。ET1作者/実装/reviewはAstra/high、独立runnerはSol/mediumを計画どおり明示する。親モデルは変更しない。

## 作者RED工程のcheckpoint

`9fd2dc6`でD2前提確認を保存・pushし、`61d0ef5`でET1の独立試験/固定Err入口/限定probe設計を保存・pushした。新34宣言は構成別に通常4成功、試用33は5pass/28failである。試用の具体的な公開root/cwd RED14と固定Err由来失敗14、実挙動baseline2passとstubだけで成立する拒否3passを分離する。構成共通名を重複してunique37件と数えない。証拠はfixture修正後のmixed snapshotであり、最終runnerで固定source全件fresh実行する。[作者記録](early-trial-paths-tests.md)を正本とする。

probe故障seamはdesignerへ限定して具体化し、[独立Pre](early-trial-probe-review.md)でET-P01（observer解除と本体OnceLock resetの混同）1件を是正した。本体は未実装であり、この是正をコード不具合修正や機能完成とは扱わない。通常の補助1役に対し、独立したpublic試験作成とprobe文書の限定設計だけを別所有で並行し、source同時編集と再帰委任はない。

中間時計12:31:56 UTC（21:31:56 JST）、週枠usedPercent=7、windowDurationMins=10080、resetsAt=1792496607。開始時resetsAtとの差は1秒であり、取得値をそのまま保持する。アカウント共通・丸め/反映遅延/他作業内訳未取得のため、当該工程だけの消費率へ換算しない。

作者session `01a13517-5497-7ca0-80c6-7b1b3997bde4`のusage72イベントを取得した。最初11:48:16.029 UTCはinput=38201/cached=23040/output=181/reasoning=12/total=38382、最後12:27:10.718 UTCはinput=8671373/cached=8509952/output=49742/reasoning=10531/total=8721115、cache_write=0。差分はinput=8633172/cached=8486912/output=49561/reasoning=10519/total=8682733である。初回requestと末尾応答反映の完全性は差分の保証外、親子包含未確認で合算しない。指定Astra/highとruntime返却値は別で後者は未取得である。

手戻りはlock中file読取fixture、junction cleanup、root前提の偽陽性防止assert、およびbuild中のtest変更に伴う再compileであった。必要な修正後の実行であり比較用再実行は0。後続担当へ、source修正をまとめて固定してからcompileし、既知sandbox制約は繰り返さず固定EXE/非管理者runtimeへ分離する指示を適用する。検証基準・model/effortは変更しない。

## 2026-10-14の再開snapshot

利用者は残る1件の確認→本体修正→独立検証/Postまでの続行を明示した。時計00:48:13 UTC（09:48:13 JST）、週枠usedPercent=10、windowDurationMins=10080、resetsAt=1792496607。前回中間snapshot以降の未観測区間を今回単独の使用量と扱わない。

親の再開usage snapshotは00:49:03.055 UTC、input=1571434790/cached=1534156928/output=3567375/reasoning=1079497/total=1575002165、cache_write=0、turn_contextはSol/medium指定である。再開直後の読取/dispatchからsnapshotまでの区間は差分外。親子包含とruntime返却値は引き続き未確認である。

前回の子エージェントは現在の実行一覧に存在せず、`trial_recheck_resume`（独立作者/Astra/high/fresh context）へ1caseだけ再dispatchした。現storeにはStorage dropと失敗root識別messageのfixture修正が残り、diagnosticはtrial rootだけのrename OS5を示す。修正版compileログはあるがruntime成功証拠はないため、この1caseを新EXEで実行する。本体と他の成功済み試験はこの作者が変更/反復しない。その後、Started遷移を初回I/O直前へ置くsticky修正を実装役へ渡し、独立runner Sol/medium/Post Astra/highへ進む。製品EXE・ET2/ET3提供は対象外のままである。

### 作者返却と本体checkpoint

残るexact1と新stage回帰2件は作者が各成功を確認した。新2件はfixture修正前後のmixed snapshotであり、最終全件成功ではない。本体sticky修正と作者証拠を`30fc43c`で保存・pushした。次の1条件mutationによる試験感度確認はレビュー手順の検証であり、通常開発の性能比較でも受入PASS件数でもない。逆差分と全source hash復元後、固定sourceの通常4/試用35→独立回帰→Postへ進む。

作者session `01a137e2-5701-7692-a8e2-78b0a6f5871a`の45usageイベントを取得した。最初00:49:14.040 UTCはinput=37974/cached=0/output=251/reasoning=0/total=38225、最後01:05:25.621 UTCはinput=3759325/cached=3647104/output=16138/reasoning=1312/total=3775463、cache_write=0。差分はinput=3721351/cached=3647104/output=15887/reasoning=1312/total=3737238である。最初のrequest以前と最後のcounter後の応答反映は保証外、親子包含未確認で合算しない。turn_contextはAstra/high指定と一致するがruntime返却モデルを示すものではない。

実装担当sessionは `01a137e7-ed3a-7bd0-8a38-cfd71e56a966`（Astra/high）である。source1条件のmutation/逆復元とfocusedを逐次所有し、作者sourceは返却済み。次の独立runnerとread-only reviewを並行する場合、source固定後の実行と静的照合という異なる責任に限定し、最終Postはrunner証拠受領後に判定する。

実装担当の62usageイベントを取得した。最初00:55:16.845 UTCはinput=38196/cached=23040/output=138/reasoning=10/total=38334、最後01:23:30.654 UTCはinput=5871441/cached=5676672/output=16167/reasoning=1525/total=5887608。差分はinput=5833245/cached=5653632/output=16029/reasoning=1515/total=5849274、cache_write=0である。初回request以前・最終counter後の応答反映は保証外、親子包含未確認のため合算しない。最終fixed通常4/試用35PASS、感度確認1FAILとhash復元を記録した。mutationは受入PASSへ加算しない。

### 独立回帰・限定fixture是正

独立runner初回は通常250行動PASS/0FAIL/fixtureSKIP3、試用276行動PASS/5FAIL/fixtureSKIP3。新通常4/試用35は全PASS。本体の保護漏れという親の初報は独立原因確認で訂正した。5件は通常namespace固定の既存fixture不整合であり、登録前generic caller契約を変えず3test-file/5symbolだけを是正する。`bb01b58`で失敗証拠と計画を保存・pushし、ET1-FX1 Pre-1は独立PASSである。失敗ログ/期待値を削除せず、修正影響のbin/destinationだけ両構成各83raw件を再検証する。

runner session `01a13801-2d5c-7f51-86ef-860c79cb964b`（Sol/medium）の74イベント。最初01:22:52.176 UTCはinput=41784/cached=0/output=135/reasoning=0/total=41919、初回終了側01:43:45.004 UTCはinput=7113662/cached=6970240/output=22116/reasoning=4989/total=7135778。差分はinput=7071878/cached=6970240/output=21981/reasoning=4989/total=7093859。再開後のFX1検証は後続snapshotへ接合する。

限定planner session `01a13810-6d05-7140-8f62-bff0aa4ae790`（Astra/high）の16イベント。最初01:39:35.437 UTCはinput=38027/cached=23040/output=309/reasoning=19/total=38336、最後01:45:20.539 UTCはinput=1242282/cached=1166976/output=12985/reasoning=777/total=1255267。差分はinput=1204255/cached=1143936/output=12676/reasoning=758/total=1216931。plannerの自己報告終了snapshotより後の反映を含む。自己報告の週枠17→18%は並行作業込みでありplanner単独へ帰属しない。

上記cache_writeは0。初回request以前/最終counter後は保証外、親子包含未確認で合算しない。返却model/effortは未取得で、指定値と区別する。手戻りはrunner集計wrapperの配列混在と起動前構文調整、既存fixtureの構成不整合5件である。前者は生ログを保持し成功済みの再実行を避けた。後者の修正後検証は品質保持のための再実行であり、性能比較用の反復ではない。

## ET1 / FX1終了snapshot（2026-10-14、最終記録・push・応答前）

FX1独立PostとET1限定工程PostはPASS/Closed、阻止0。両構成の修正影響83raw件は独立全成功、各80行動PASS/fixtureSKIP3、未変更範囲接合後は通常250/試用281行動PASS・失敗0・各skip3。5FAILは解消し、88不変sourceと未変更12EXEの証拠継続を照合した。全件を最後に再実行したものではない。ET2/ET3、製品EXE、native/実API受入は未完である。

終了側時計02:14:26 UTC（11:14:26 JST）、週枠usedPercent=22、windowDurationMins=10080、resetsAt=1792496608。再開側10%との差を当該タスクだけの消費へ帰属しない。並行作業・反映遅延の内訳は不明、resetの取得値に1秒差があることも保持する。通常開発の観測だけであり比較専用の反復0、旧方式比は未判定である。

親の終了counterは02:16:16.587 UTC、input=1598034079/cached=1560493312/output=3613633/reasoning=1093065/total=1601647712、cache_write=0。同日再開snapshotとの差分はinput=26599289/cached=26336384/output=46258/reasoning=13568/total=26645547。長期親の累積値自体を今回消費とは扱わず、以降の文書保存・commit/push・最終応答は取得範囲外である。

| 子session / 指定model-effort | 観測範囲UTC | 終了counter input / cached / output / reasoning / total | 最初の実在snapshotとの差分（同順） |
| --- | --- | --- | --- |
| 作者01a137e2… / Astra-high、79イベント | 00:49:14.040–02:00:25.038 | 8388392 / 8222080 / 31214 / 4259 / 8419606 | 8350418 / 8222080 / 30963 / 4259 / 8381381 |
| runner01a13801… / Sol-medium、103イベント | 01:22:52.176–02:13:04.641 | 11254139 / 11076352 / 34777 / 8534 / 11288916 | 11212355 / 11076352 / 34642 / 8534 / 11246997 |
| reviewer01a137f8… / Astra-high、54イベント | 01:13:57.112–02:14:09.803 | 8012096 / 7811072 / 17814 / 4158 / 8029910 | 7973803 / 7788032 / 17654 / 4158 / 7991457 |

子完全IDと最初のcounterは上節または各担当報告を正とする。reviewerの最初はinput=38293/cached=23040/output=160/reasoning=0/total=38453である。表は各sessionの前の中間counterを更新するもので、中間差分と重ねて合算しない。全てcache_write=0、初回request以前/最終counter以降は保証外、親子包含未確認で合算しない。モデル/effortは指定とturn_contextの値で、provider返却値は未取得である。実装担当とplannerの終了counterは上節どおりである。

追加手戻りは作者が新assertでbin crateに存在しないcrate::storeを参照したE0433であり、所有内を既存store::へ直してcompileした。行動REDではない。改善候補は、namespace/環境欠落エラーを変えるfeature追加時に既存fixtureの独立literal・lock対象・error全文を先に棚卸しすること、テストassertの参照先を既存module importに合わせることである。恒久harness/設定/モデル/品質水準は変更していない。保存先と任意外部宛先のliteralを一括置換しない点が重要であり、今回の契約確認と5symbol限定diffを根拠とする。効果量・週枠節約率は未測定である。
