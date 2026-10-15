# ET3 試用EXE提供の使用量観測

## 利用者「継続で」受領・LIFE14限定再開（2026-10-15）

LIFE14作者は開始97→終了98%共有週枠（10080分/reset1792496608）を取得した。旧oracle3RED→新8PASS、合成同期対照、静的検査を返却した。designer/Pre/仕様の子counterは欠測、共有率をtask専有量へ換算しない。親は既定割当Astra/highで実装者へ最小hook/診断だけを引き継いだ。実gate/full/提供は未了である。

利用者は95%共有週枠の報告後も継続を承認した。親モデル不変、開始側週枠95%/10080分/reset1792496607/ordinaryUsageAllowed=true。親counterの直前観測15:53:52.898Zは上記checkpointの値を参照し、今回着手時の完全一致snapshotではない。子counter/返却model・effort/包含は欠測を保持する。planner gpt-6-astra/highへLIFE14の目的・刺激・期待値の限定照合を依頼し、期待値緩和や試験削除ではない最小是正を選定する。製品source/実data/API/資格情報/OS変更なし。

## 限定是正返却checkpoint（2026-10-15）

親の最終応答前観測15:53:52.898Z：input1750335382/cached1709918592/output3998406/reasoning1284467/total1754333788/cache_write0。13:21:12.832Z開始側との差はinput48453866/cached47977984/output112827/reasoning57056/total48566693。cached/reasoningは内数、子との包含未確認のため合算しない。ここからcheckpoint/最終応答までは未反映である。共有週枠95%/10080分/reset1792496607/ordinaryUsageAllowed=true、65→95を工程専有量へ換算しない。

限定返却はdispatch5PASSとsidecar準備2PASS、bootstrap runtime0、LIFE14契約不一致未解決、接合Post/合成80/製品full/試用提供未了である。週枠の継続範囲を照会しており、品質gateの省略や低いモデルへの無断変更を節約としない。

## P1 closure後・合成runtime開始（2026-10-15）

dispatch是正の作者終了共有週枠94%（開始92%、並行を含む）。旧4RED/1PASS→修正後5PASSを取得し、80行oracleを維持した。bootstrap局所設計の独立PASS後、Invoke/designとTest準備の非衝突2所有へ限定接合を割当した。製品full/提供は0、次の大きな実行単位の前に継続範囲の回答を確認する。欠測子counterをキャッシュ比や成果物量から推定しない。

初回runtimeはcontrol2 PASS、GATE21 FAIL、raw26Passedはdriver呼出0のため成功から撤回、残53NotRunとなった。既定firstFail停止を維持し、reader切分け→作者の実行数保証とbootstrap拒否証拠の局所設計へ戻した。初期想定を超える一時harnessの複雑化と、静的レビューでは捕捉できなかった偽PASSは追加コスト・品質上の反省点として記録する。wrapper未合格のまま製品fullへ進む節約はしない。新しい週枠上限は設定しておらず、91%を踏まえ利用者へ今回の区切りを照会中である。

独立静的で実行前に捕捉したP1二件を限定是正した。作者のcontrol-path回帰2RED→2PASSと独立再静的closure PASS後、runner gpt-6.1-sol/mediumへ初回通常入口（control2＋合成80case）だけを解除した。製品fullは未解除、成功済みrelease/formatの反復なし。親の再開時共有週枠は91%/10080分/reset1792496608/ordinaryUsageAllowed=true。工程専有量へ換算しない。子token・返却model/effort・親子包含は欠測である。

## 2026-10-15 00:12 JST：4file固定・初回runtime前の区切り

作者/実装は各2fileの所有を返却。現fake自己検証9/9PASS、実装PS parser/PSSA0、nativecompile成功、作者PS parser0/PSSA error0・ShouldProcess warning6は根拠付き保持。wrapper80case/runtime/製品fullは0のまま、短命processのCIM metadata混在リスクと範囲保護をfresh reviewer `et3_gate_static_review` Astra/highへ独立静的確認中である。初期M・小script想定からPS約1700行へ拡大し、具体interface/時計/OS primitive等の往復が発生した。これは開発コスト増として記録し、並行化を使用量削減と断定しない。

共有週枠は00:12:44 JSTの親取得87%/10080/reset1792496608/ordinaryUsageAllowed=true。再開65%との差を当工程専有消費へ換算しない。親同一sessionの15:12:44.760Z観測はinput1738202408/cached1697933056/output3965905/reasoning1270444/total1742168313/cache_write0。13:21:12.832Z開始側からの差はinput36320892/cached35992448/output80326/reasoning43033/total36401218。cachedはinputに、reasoningはoutputに含まれる。開始13:18から開始側snapshotまで・この後・最終応答は未取得、子counter/実行metadata/親子包含は欠測、合算/週枠換算/金額換算をしない。

追加手戻り：Pre-2 PASS後の独立作者が、意味契約はあるが具体JSON key/cfg/digestと決定的race同期点が未固定でdriverへ進めないことを報告した。80case/fake自己検証7/7を固定して返却し、designerへ局所4点を戻した。共有週枠の作者終了受領値は73%。改善候補は、同種の複数担当interfaceでは作者へ渡す前に最小入力例/具体命名/同期点を一緒に固定することである。今回の再handoffを効率改善として隠さず、追加frameworkや検証削減で解消したとは扱わない。

再開区間の中間記録：限定計画/設計/独立PreはAstra/high、native拒否窓runnerはSol/medium。Preで入口parseとG0回収所有の2件が不足し、実装前に計画へ返した。設計是正はfresh `et3_gate_design_fix` Astra/highが担当する。native observerの未確認ControlType/IDOK前提による中断2回と、実値先行取得による最終観測1回を手戻りとして保持する。nativeの終了共有週枠70%はrunner受領値であり、開始65%との差を当工程だけの消費や旧方式比へ換算しない。各子counter/provider返却metadata/親子包含は欠測である。製品source変更0、取得済みrelease3buildを反復せず保持した。

## 22:18 JST 再開区間

利用者が一時Cargo runnerとDPI150/200%の利用者試用時確認への移管を承認したため再開する。親モデル不変、planner `et3_gate_plan` は明示Astra/high・fresh context、他役は限定計画の割当に従う。開始時計13:18:04 UTC、週枠65%/10080/reset1792496607/ordinaryUsageAllowed=true（共有）。親の開始側観測は13:21:12.832Z、input1701881516/cached1661940608/output3885579/reasoning1227411/total1705767095/cache_write0。着手から観測まで欠測、親子包含/子counter/provider返却値は未取得。前区切りとの差を今回単独消費とせず、終了時に同一sessionのこの観測との差だけ別記する。

2026-10-14。利用者の「ET3を進めて、Windowsで動作確認できる試用EXEを作成」の依頼で再開した。開始HEAD `d446c5dd32177815ee322a39cf51ab84a99ee458`、tree clean。既定ET3計画を再利用し、仕様/設計/実装/作者の再起動は非該当。runner `trial_delivery_runner` をfresh context・指定 `gpt-6.1-sol/medium` で起動、独立reviewは計画指定 `gpt-6-astra/high` とする。親の選択は変更しない。

完了境界は固定ソースの通常/試用全体test・release/関連crate gate、合成native確認の取得可能範囲、独立Post、通常版を上書きしないローカル試用EXEと使用説明の提供である。未実施native/実API/実マイク等の受入を成功にしない。実データ/実資格情報loadsave/API送信/OS設定変更/管理者実行は範囲外。製品EXE保全SHAは `9B888E9FC923823C59B4C3B0AB1A4DAD2051CA6139DE153B6DE6CA213DD387D3`。

開始側時計は11:07:02 UTC（20:07:02 JST）。週枠58%/10080分/resetsAt1792496608、ordinaryUsageAllowed=true。アカウント共通値であり当工程専有消費へ換算しない。

親session `01a08550-055c-7aa1-9834-905e4a7ce148` の取得snapshotは11:08:50.836Z、input1671588553/cached1632077952/output3794857/reasoning1174448/total1675383410/cache_write0。着手直後から取得時点まで欠測、最終応答未反映、親子包含未確認。inputはcachedを、outputはreasoningを含むため再加算せず、子と合算しない。指定とprovider返却model/effortは区別し、返却値未取得を推定で埋めない。runner counter/sessionは返却時に取得可能範囲を追記する。

最初はno-runのみとし、新しいtest EXE/構成とsource対応を親が照合するまでlist/runtimeを解除しない。旧ET2のsource読取証拠は同一bytesの範囲で保持するが、別EXEへ実行解除を流用しない。既存の権限依存ignored/fixture不足を成功として数えず、検証水準を変更しない。

通常all-targets no-runはexit101/唯一E0433で停止した。`tests/codex_process.rs`が取り込むai.rsのP01 testから、integration crateに存在しない`crate::media`の隔離helperを参照する不整合である。行動試験は0、全source/Cargo不変、旧release SHA不変。部分artifact23を成功組として解除しない。runnerは源編集/実行0で所有返却し、planner Astra/high `trial_delivery_fix_plan`へ安全harnessを維持する最小是正所有/回帰を依頼した。既定ET3のsource編集非該当からcompile blocker修正の限定単位へ戻す手戻りとして記録する。開始58→runner終了59%は共有週枠で工程専有量ではない。

FX1はplanner/designer/test author/reviewerを各Astra/high、runnerをSol/mediumへ限定割当した。製品実装/外部仕様変更の担当追加は非該当である。helper同体抽出3fileのPre PASS→作者→親source確認→独立静的指摘0→両構成all-targets no-run PASS2と進み、親が新4EXEを照合して限定回帰を解除した。別targetの試用初回compileはprocess限定CARGO_BUILD_JOBS=2で資源使用を抑え、永続設定/検証範囲は変えていない。

共有週枠の中間観測はplanner59→59（精緻化60→60）、designer59→60、Pre reviewer61→61、作者61→62、静的reviewer62→62、native条件の限定判断62→63、FX1 no-run runner開始62→終了63%である。再利用agentの前工程を含むことがあり、これらを加算/担当別費用換算しない。各子のsession/token/返却model・effort/親子包含は未取得である。全体終了値は未取得であり、効果比較の結論はまだ出さない。

## 承認待ちの区切り（ET3未完了）

FX1限定回帰84/84PASS・独立Post PASSでsource保全commit cb88203をpushした。full開始時の再compileで旧通常EXE解除が失効し、所有Cargoだけを停止/回収した。製品test FAILとして数えず、full成功にも数えない。固定環境no-runと限定診断により、Cargo binary dep-infoにU+3000手前で切れた不存在prefixが再生成される局所原因を確認した。手戻りは[runner報告](early-trial-delivery-runner.md)と[親判定](early-trial-delivery-review.md)を正とする。環境差だけという初期仮説は確定原因にしない。

未影響の試用/通常/単独qwen release build3件は成功し、source/旧製品を保持した。全体runtime/関連test/native/最終提供は未了である。一時Cargo runner導入とDPI150/200確認の移管は未回答・未導入・基準未変更。比較専用再実行0、不要と分かった試用no-run反復0。今後の改善候補は、always-dirty環境でもbuild直後の新EXEを実行前に照合できる一時runnerであり、利用者承認と安全設計前に導入しない。節約効果は未測定である。

区切りの時計12:40:16 UTC（21:40:16 JST）、共有週枠65%/10080分/reset1792496607/ordinaryUsageAllowed=true。開始58→65はアカウント共通観測で、当工程専有量に換算しない。FX1最終review63→64、後段runner64→65も共有値である。子counter/session/返却model・effort/親子包含は欠測のままである。

親の終了側snapshotは12:40:41.533Z、input1697478705/cached1657576064/output3874599/reasoning1222622/total1701353304/cache_write0。開始側11:08:50.836Zとの差はinput25890152/cached25498112/output79742/reasoning48174/total25969894/cache_write0である。cached/reasoningは内数、親子を合算しない。着手から開始snapshotまで、ここから記録commit/push/最終応答までは欠測範囲となる。旧方式の同条件baselineがなく、使用量の増減・削減率は未判定である。
