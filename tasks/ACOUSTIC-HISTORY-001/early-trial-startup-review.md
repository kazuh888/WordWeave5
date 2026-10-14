# ET2 起動接合・独立レビュー

## Pre-1：CHANGES REQUIRED

2026-10-14。担当 `/root/trial_startup_review`、指定gpt-6-astra/high、thread `01a1393b-2c35-74d3-a853-57441be994a3`。read-only担当の返却を親が保存した。HEAD `97d2c72a9adec4f1904c6deb2b18e92c50262923`、design SHA `D9E044C42C85D1C1914B2E24820AE31D6B4082B7B920EC9F4E5C5CB1DDBBB42A`、plan SHA `38C5486BF336465C85EEC18985EA68CC0890DE02F27A6E215494DEF045E409A9`、正本契約SHA `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`。6source基準hash一致、source差分0である。

| Finding | 原因・現物根拠 | State / packet / canonical owner | Oracle・残余 |
| --- | --- | --- | --- |
| ET2-PRE-01 / P2 / 受入阻止 / 確信度高 | 初回録音正対照の実行前に、watchとRecorder::start先頭の遮断接合を確認する条件がない。監視定義だけあり入口接合を漏らすとcompile可能であり、media.rs:62のcpalへ到達後にcounter assertが失敗しても遅い | Accepted / ET2-PR1 / 設計者が設計の唯一の規範条件を修正、plannerは実行計画へ参照・順序を反映 | source固定→親が共有global/active判定/加算/合成Err/poison拒否のcpal前完結を読取確認→そのsourceから作ったEXE hash固定後にだけ実行解除。後続動的counter検査の代替ではない。実装/runner/Postの証拠は未取得 |

最小是正は上記実行前安全Gateを設計・計画に明記し、新hashで再Pre判定することである。未確認なら正対照を実行しない。未監視test・非test本体・限定所有を維持し、感度実験でも遮断自体は撤去しない。新製品判断・公開APIは不要である。

その他はload1回/fatal時App非公開/成功時bool・diagnostics/creator所有移譲/実回復後rename拒否/回復後read拒否/JSON先頭拒否/private credential selector/通常構成を現物照合し、確定阻止0であった。OS共有拒否と全動作は未実証、ET3全体/release/native/DPI/製品EXEは未完了である。編集・実行試験・再委譲・実資格情報/API/録音は0。

指定モデルとprovider返却値は別であり後者未取得。担当が返したcounter観測は07:06:07.683–07:09:57.372 UTCまでで、その後の検討と最終応答は含まない。詳細は親のusage記録へ接合する。

## Pre-2：ET2設計PASS / ET2-PRE-01設計上Closed

同じread-only reviewer（Astra/high、同thread）の独立再判定はPASS、阻止0、要回答質問0、追加提案0。親が返却結果を保存した。対象HEADは上記97d2c72、design SHA `24B9FEBE6F83528EF1B4D55EF9890B00FC9846E9F50B5BB446F48A2FAE687D7E`、plan SHA `82E0A3E8A326EBEB2456ACC994731895072F3B714286452598435547ADC31AFD`、正本契約と6sourceの基準hashは不変。

ET2-G0でsource固定→親の共通global/cpal前拒否/poison/watch寿命の読取確認→対応EXE固定→親明示解除→動的正対照の順が確定した。接合漏れ・別global・EXE不一致は実行前に停止する。source/EXE/構成変更、感度実験・逆復元後も再確認する。planの旧即時実行条件と加算を崩す変異は除去され、録音の遮断/加算を保持したcalls観測返値だけの変異へ限定した。動的counter検査、通常未監視/non-test動作、所有6file、既定gateを維持する。

設計全体・plan追補・是正記録・実差分を照合し、規範競合なし、参照欠落0、diff検査成功。既存factory/実回復障害/復元拒否/credential selectorの判断も維持される。ET2-PRE-01の現在状態は**設計上Closed**である。

これは設計欠落の解消であり、実行解除ではない。observer実装、動的安全証拠、実ET2-G0、ET2機能/最終Postは未了。実装後の同指摘のclosureも要求する。ET3全体test/release/native/DPI/製品EXEは未完了、reviewerの編集・実試験・再委譲・実資格情報/API/録音は0。
