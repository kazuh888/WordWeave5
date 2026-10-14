# ET3 試用EXE提供の使用量観測

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
