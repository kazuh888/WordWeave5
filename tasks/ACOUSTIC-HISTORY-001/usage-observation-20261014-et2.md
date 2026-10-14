# ET2 工程別引継ぎ・使用量観測

2026-10-14 / ACOUSTIC-HISTORY-001。完了境界は早期試用の起動拒否、専用Qwen資格情報target、識別表示、JSON履歴候補の適用前拒否の接合と独立検証/Postまでである。製品EXE・ET3全体gate・native拒否dialog/DPI・実認証/API受入は別工程に残る。

## 開始snapshot

- HEAD `97d2c72a9adec4f1904c6deb2b18e92c50262923`、tree clean、ET1/FX1は独立Post Closed。
- 時計06:40:52 UTC（15:40:52 JST）、週枠usedPercent=22、windowDurationMins=10080、resetsAt=1792496607。前回終了との未観測区間をET2消費へ算入しない。
- 親session `01a08550-055c-7aa1-9834-905e4a7ce148`。counter06:42:19.181 UTC：input=1600113752/cached=1562321280/output=3620739/reasoning=1095015/total=1603734491、cache_write=0。再開読取・最初のdispatchからsnapshotまでの区間は差分外である。
- 親turn_contextはgpt-6.1-sol/medium指定。planner既定ET2はdesigner/author/implementer/reviewer=gpt-6-astra/high、runner=gpt-6.1-sol/medium。provider返却値は未取得であり指定と区別する。

## 開始時の進行

承認済み契約SHA `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`とET2計画を再利用する。designer `trial_startup_design`へ既存new_with_storageの失敗処理、headless試験の接合、回復書込後/初期化後障害、diagnostics開始0を5file内で検証する最小設計の具体化を委譲した。外部仕様の再作成や新機能追加は行わない。親はusage/current記録を並行し、同じ設計調査を重複しない。

実データ・実資格情報load/save・API・マイク・管理者/OS変更は使用しない。CredentialStoreのproduction target確認は構築とtarget()のみ、操作はnamespace別spyを使用する。既知sandbox runtime OS5制約は反復せず、必要時は固定EXEとAdministrator=False強制の通常ユーザー実行を分離する。旧権限依存試験の許可は流用しない。

終了側counterと担当ID/取得範囲は工程の区切りで追記する。親子包含未確認で合算しない。性能比較専用の反復・常駐監視・model/router設定変更はしない。

## 設計の限定返却

designerは実sourceに即したfactory/typed failure/diagnostics/credential selectorと、実回復書込後のrename拒否・回復成功後の教材read拒否oracleを具体化した。一方、録音開始mock counterは既存5fileにないためSTOPし、src/media.rsのcfg(test)監視だけを追加所有としてplannerへ返した。資料は[起動接合設計](early-trial-startup-design.md) SHA `4949F33F33656E8C6BD041371001A22A733E71230009C593F9024F6D17EB4E9C`。source/test編集0、実録音0、品質基準変更0である。

planner `trial_startup_plan`（Astra/high/fresh context）が既存planへの限定deltaを担当する。これは設計で発見した検証境界の不足であり、独立Pre前に本体/試験へ進まない。原仕様の「録音開始0」を状態/静的検索だけで成功としない。

plannerはET2-Sとしてmedia.rsのcfg(test)監視だけを採用し、元5fileをET2-Fとして維持した。親受領後designerがSTOP解除条件を反映し、design SHA `D9E044C42C85D1C1914B2E24820AE31D6B4082B7B920EC9F4E5C5CB1DDBBB42A`、plan SHA `38C5486BF336465C85EEC18985EA68CC0890DE02F27A6E215494DEF045E409A9`を独立Preへ提出した。担当 `trial_startup_review` はAstra/high/fresh/read-only。source編集0を維持している。

planner thread `01a13931-36fd-7922-a65d-a60492d90e26`の自己報告観測は06:55:11–07:01:03 UTC、input38145→992454/cached23040→917504/output294→9895/reasoning0→1057。差分はinput954309/cached894464/output9601/reasoning1057。開始前/最終応答後/親子包含は不明、runtimeモデルは未取得、後続の実在snapshot確認で範囲を更新する。モデル比較や週枠への換算は行わない。

## Pre-1の指摘是正

独立reviewerはET2-PRE-01(P2)阻止1件を返した。録音watchだけ実装されstart先頭の遮断を漏らすとcompileは可能であり、初回正対照が実micへ到達後にcounter assertionで止まっても遅い。親の懸念を独立source/設計照合で確定したもので、実マイクを動かした発見ではない。designerは実行前ET2-G0を唯一の規範へ整備し、plannerが実行順/感度範囲を同期する。source/test/実行0、検証水準変更0を維持している。

reviewer thread `01a1393b-2c35-74d3-a853-57441be994a3`、指定Astra/high。自己報告counter07:06:07.683→07:09:57.372 UTCはinput38227→924858/cached23040→823168/output174→3905/reasoning0→905。差分input886631/cached800128/output3731/reasoning905。後続検討・最終応答は含まず、最終snapshotを別途取得する。親子包含/返却model未確認、合算しない。

Pre-2は設計PASS/指摘の設計上Closedとなった。design SHA `24B9FEBE…AE687D7E` / plan SHA `82E0A3E8…ADC31AFD`、source6不変、実行解除ではない。reviewerが返した更新counterは07:21:01.509 UTC input=1587858/cached=1459584/output=7611/reasoning=2069であり、Pre-2単独の開始値はなく、この時点以降の最終応答も範囲外。先のsnapshotとの差を追加タスク消費として重複加算しない。

## 独立作者の引継ぎ

作者 `trial_startup_tests`、thread `01a1394d-02e8-7493-84ae-371e9ade04bc`、指定Astra/high・返却model/effort未取得。[試験報告](early-trial-startup-tests.md)で18caseを固定した。pure-lib定数exact1件は目的FAIL、bin一括no-runは不足API28診断で未compile、Recorder runtime0。作者count参照誤り2件は修正済みであり行動REDには数えない。起動中API送信0の直接観測はないという範囲を明記した。source6fileへの追加のみ900行、fmt/diff成功、所有返却済みである。

取得snapshotは07:25:24.180Z input38639/cached23040/output223/reasoning10→07:45:05.745Z input5530507/cached5386368/output32742/reasoning6268。差分input5491868/cached5363328/output32519/reasoning6258。開始前/最終応答分/親子包含は未確認であり別加算しない。待機中に親は起動/録音経路と作者fixtureを統合確認したが、作者と同じ試験を再実行していない。定数試験の作者再実行1回は後続buildで置換されたEXEと識別を結び直すためである。
