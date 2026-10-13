# ET1再開の使用量観測

2026-10-13 / ACOUSTIC-HISTORY-001。利用者の再開依頼により、媒体export完了後のD2集約gateと早期試用保存先分離ET1へ進む。完了境界はET1独立テスト・実装・独立runner/Postまで、ET2資格情報/表示とET3製品提供は後続である。通常版のデータや秘密を試験に使用しない。

## 開始snapshot

- 対象HEAD `186b46433801b0bc953eeaebf15b297bf07cd577`、tree clean。
- 時計11:43:45 UTC（20:43:45 JST）。週枠usedPercent=0、windowDurationMins=10080、resetsAt=1792496608。前回の窓とresetが異なるため、前回98%との差分を使用量へ算入しない。
- 親session `01a08550-055c-7aa1-9834-905e4a7ce148`。親usage snapshot 11:44:36.417 UTC：input=1545068307、cached_input=1508161664、output=3533748、reasoning_output=1068755、total=1548602055、cache_write=0。
- 親turn_contextはgpt-6.1-sol/medium指定と一致するが、プロバイダー返却値ではない。再開直後からsnapshotまでの読取/引継ぎ区間は差分外である。
- 前提集約reviewer `trial_dependency_gate` を既定gpt-6-astra/high・fresh contextで起動した。既存個別Postをやり直さず、D2全出口の迂回残置とET1依存集合だけを独立照合する。

親子包含は未確認のため合算しない。担当別開始/終了counterと取得範囲を工程の区切りに追記する。比較だけの試験、常駐監視、新しいモデルrouterは追加しない。ET1作者/実装/reviewはAstra/high、独立runnerはSol/mediumを計画どおり明示する。親モデルは変更しない。
