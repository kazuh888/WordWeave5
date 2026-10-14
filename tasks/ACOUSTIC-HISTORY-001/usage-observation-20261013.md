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
