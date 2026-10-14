# ET3 試用EXE提供の使用量観測

2026-10-14。利用者の「ET3を進めて、Windowsで動作確認できる試用EXEを作成」の依頼で再開した。開始HEAD `d446c5dd32177815ee322a39cf51ab84a99ee458`、tree clean。既定ET3計画を再利用し、仕様/設計/実装/作者の再起動は非該当。runner `trial_delivery_runner` をfresh context・指定 `gpt-6.1-sol/medium` で起動、独立reviewは計画指定 `gpt-6-astra/high` とする。親の選択は変更しない。

完了境界は固定ソースの通常/試用全体test・release/関連crate gate、合成native確認の取得可能範囲、独立Post、通常版を上書きしないローカル試用EXEと使用説明の提供である。未実施native/実API/実マイク等の受入を成功にしない。実データ/実資格情報loadsave/API送信/OS設定変更/管理者実行は範囲外。製品EXE保全SHAは `9B888E9FC923823C59B4C3B0AB1A4DAD2051CA6139DE153B6DE6CA213DD387D3`。

開始側時計は11:07:02 UTC（20:07:02 JST）。週枠58%/10080分/resetsAt1792496608、ordinaryUsageAllowed=true。アカウント共通値であり当工程専有消費へ換算しない。

親session `01a08550-055c-7aa1-9834-905e4a7ce148` の取得snapshotは11:08:50.836Z、input1671588553/cached1632077952/output3794857/reasoning1174448/total1675383410/cache_write0。着手直後から取得時点まで欠測、最終応答未反映、親子包含未確認。inputはcachedを、outputはreasoningを含むため再加算せず、子と合算しない。指定とprovider返却model/effortは区別し、返却値未取得を推定で埋めない。runner counter/sessionは返却時に取得可能範囲を追記する。

最初はno-runのみとし、新しいtest EXE/構成とsource対応を親が照合するまでlist/runtimeを解除しない。旧ET2のsource読取証拠は同一bytesの範囲で保持するが、別EXEへ実行解除を流用しない。既存の権限依存ignored/fixture不足を成功として数えず、検証水準を変更しない。

通常all-targets no-runはexit101/唯一E0433で停止した。`tests/codex_process.rs`が取り込むai.rsのP01 testから、integration crateに存在しない`crate::media`の隔離helperを参照する不整合である。行動試験は0、全source/Cargo不変、旧release SHA不変。部分artifact23を成功組として解除しない。runnerは源編集/実行0で所有返却し、planner Astra/high `trial_delivery_fix_plan`へ安全harnessを維持する最小是正所有/回帰を依頼した。既定ET3のsource編集非該当からcompile blocker修正の限定単位へ戻す手戻りとして記録する。開始58→runner終了59%は共有週枠で工程専有量ではない。

FX1はplanner/designer/test author/reviewerを各Astra/high、runnerをSol/mediumへ限定割当した。製品実装/外部仕様変更の担当追加は非該当である。helper同体抽出3fileのPre PASS→作者→親source確認→独立静的指摘0→両構成all-targets no-run PASS2と進み、親が新4EXEを照合して限定回帰を解除した。別targetの試用初回compileはprocess限定CARGO_BUILD_JOBS=2で資源使用を抑え、永続設定/検証範囲は変えていない。

共有週枠の中間観測はplanner59→59（精緻化60→60）、designer59→60、Pre reviewer61→61、作者61→62、静的reviewer62→62、native条件の限定判断62→63、FX1 no-run runner開始62→終了63%である。再利用agentの前工程を含むことがあり、これらを加算/担当別費用換算しない。各子のsession/token/返却model・effort/親子包含は未取得である。全体終了値は未取得であり、効果比較の結論はまだ出さない。
