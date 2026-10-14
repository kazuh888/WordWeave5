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

## C1–C3とP01設計の中間checkpoint（未完了）

実装者 `trial_startup_impl`、session `01a13962-a5dd-7822-9764-05ea27255dc5`、指定Astra/high。C1本体5file接合・通常/試用no-run成功後、親G0で固定source/EXEへ限定解除した。安全3両構成と通常7PASS、試用#3は作者JSONの必須field不足で停止。作者が是正したC2でも#11正常JSONに不足がありrestore前で停止した。全18fixtureを一括点検してC3へ最小修正した。本体欠陥とfixture欠陥を混同せず、旧FAIL/log/hashを保持し期待値を弱めていない。

独立reviewerは送信spyが起動/設定/復元に未接続というET2-P01/P2を確定した。plannerの4file限定追加→designerの具体型/worker寿命/G0同期→独立Pre PASSまで進んだ。P01本体実装/動的証拠/Postは未了である。設計担当 `trial_egress_design`、session `01a13975-8bd4-7d32-84b7-1ce794b5d6ca`、指定Astra/high。実装/runtimeなし。

08:32:30 UTC時点のアカウント共通週枠usedPercent=42、window=10080、resetsAt=1792496607。開始22から表示上20ポイント増えたが、他作業/共有枠/整数粒度を分離できずET2専有消費とは断定しない。親へ情報を通知し、許可なしのmodel/effort/quality gate変更はしていない。

親の実在snapshot08:31:15.942Zはinput1633692386/cached1595256064/output3696525/reasoning1129498/total1637388911。開始snapshotとの差はinput33578634/cached32934784/output75786/reasoning34483。最終値ではなく、最初のdispatch前後欠測と親子包含不明を維持する。

完了した担当の対象session末尾からtoken_countだけを読取共有で取得した。本文/秘密は記録していない。再開で同sessionを使うため、以下は各Packet単独量でも全タスク最終値でもない。子同士/親との合算は行わない。指定model/effortは全員Astra/high、provider返却値は未取得。

| 担当（session上記） | 観測UTC | input | cached | output | reasoning |
| --- | --- | ---: | ---: | ---: | ---: |
| planner | 08:11:32.414 | 5141273 | 4956544 | 30205 | 4963 |
| reviewer | 08:26:38.264 | 4692702 | 4448768 | 16990 | 4571 |
| author | 08:29:43.009 | 8954883 | 8721536 | 48145 | 9542 |
| egress_designer | 08:19:54.668 | 2026232 | 1922816 | 15452 | 3023 |

egress_designer開始snapshot08:09:41.886Zはinput88977/cached61440/output363/reasoning13。観測差分input1937255/cached1861376/output15089/reasoning3010、開始直後までの欠測あり。実装担当はC3実行中で終了snapshot未取得である。初回startup designerの固有session/終了counterは未取得のままであり、全子の総量を取得済みとはしない。

低リスク効率化として、試用cfg(test)のfixtureとコメントだけを直したC3の焦点検査はtrial binのみno-run/実行とし、通常/lib成功は元snapshotに限定した。最終独立runner両構成とET3全体gateは維持する。cache再利用でC2build時間が減った証拠はなく、incremental hard-link失敗→compiler copy warningを記録した。fixture点検を一件ずつでなく全18へまとめ、source凍結build中に独立設計/静的reviewだけを並行した。今後の改善候補は正常fixtureの必須field確認を作者引継ぎ前に行うこと、長い旧ログ/本文転載を避け短いartifact参照で引き継ぐことである。常駐監視・運用harness・モデルrouter変更は行わない。

C3試用bin新17件は現固定source/EXEで全PASS、実名一致/毎起動Admin=FalseとSHA確認/timeoutなし/wait済/残存0。通常/libは再実行なし。P01独立PreはPASSだが実装/動的観測/感度/runner/Postは未了。実装者の最新自己報告counter08:37:16.029Zは同session累積input16047921/cached15808384/output42474/reasoning4645、記録追記と最終応答は未反映である。初回観測07:48:59.445Zとの差はinput16009258/cached15785344/output42304/reasoning4645。開始前/最終応答分/親子包含不明を維持する。C1/C2の停止・是正・再Gateを含み、C3単独消費とは呼ばない。

## P01専用fresh contextへの引継ぎ

`fef8bd2`でC3本体/設計Pre記録をpush後、P01専用作者と別実装者をfresh contextで起動した。旧C1–C3の長いrole履歴をforkせず、承認済みの限定節/接合箇所/必要証拠だけを渡した。役割モデルはplanner指定Astra/highを維持し、親やquality gateを変えていない。

作者`trial_egress_tests`、session `01a13995-0994-7fc2-b085-cca36d93e390`。開始側実在snapshot08:45:34.235Zはinput569416/cached506496/output1719/reasoning199、最終応答前08:58:28.786Zはinput3259401/cached3156992/output25347/reasoning4438。差分input2689985/cached2650496/output23628/reasoning4239。着手直後まで/最終応答/親子包含の欠測あり、返却model/effort未取得である。20case/両構成予定集合を固定、no-run1回の71不足API診断はcompile blocker、runtime0。新APIのため隠れる後続型検査を未了と記録した。source所有を別実装者へ返却済み。

作者終了時の共有週枠usedPercent=45（同window/reset、ordinaryUsageAllowed=true）。工程完了まで継続/50%目安/60%目安の任意選択は未回答であり、既存の続行依頼に基づいて依頼済み工程を継続する。選択を提出したとは扱わず、アカウント変化をP01専有消費へ換算しない。

その後、利用者は「現在のET2工程の独立検証・レビュー完了まで継続する」を明示回答した。50/60%の数値上限は選択されていない。ET3は別の区切りとする。P01実装の通常/試用no-run成功と独立静的指摘0を取得し、同じ固定source/EXEのfocused自己検証を依頼した。

P01実装session `01a139a4-14c3-7402-834c-3e219e5feb55` のno-run終了観測09:16:30.391Zはinput4100291/cached3990016/output15348/reasoning1249。P01 reviewer session `01a139ac-f26e-7fb2-bf3a-c7d827f8e3bc` の09:13:49.355Zはinput1266517/cached1166208/output4094/reasoning528。いずれも最終応答前まで、工程開始欠測・親子包含不明・provider返却model/effort未取得を保持し、合算しない。

## V2独立検証と感度是正の中間記録

V1自己46PASS後、感度準備で作者A09のcatch_unwindがcount assertの失敗を意図panicと混同する隙間を発見した。作者が専用panic型識別と未知payload再送出へ最小是正し、独立静的確認を受けた。実装者のV2重複build/自己検証は行わず、runnerの両構成bin/lib独立no-runと252件focused回帰へ集約した。252PASS/FAILskiptimeout残存0で、ET3全体gateを代替しない。source保全checkpoint ee84f8eをpush済みである。

感度は既定3snapshotへ相互隠蔽しない変異をまとめた。S1目的FAIL17、S2目的FAIL6を独立確認、S3目的2件は実行中。各変異/復元のsourceG0とEXE対応照合は保持し、基準や実遮断を弱めていない。A09是正とno-run一回分の中断は実際の手戻りとして記録し、compile成功/通常PASSだけでoracleが十分とは扱わない。改善候補は意図したpanicの型識別を作者段階で点検することであり、今回そのためのharness/settings/Skill変更はしていない。

共有週枠の観測はrunner開始49→V2開始51/終了52→S1開始53/終了54→S2開始55/終了55%。同じ10080分窓、reset値1792496607/1792496608の1秒差を含む。並行する親/子/他作業と整数粒度のため工程専有量へ換算しない。runner session/token counterとprovider metadataは未取得で欠測を保持する。実装者の最新S3 no-run観測10:24:33.845Zはinput16104536/cached15852672/output50360/reasoning5958、同session累積でありS3単独量ではない。

## 終了側取得（最終focused GREEN・Post・報告前）

S3は2/2目的FAIL、S1–3計25目的FAILを独立確認した。一時変異はV2全8source/4markerへ完全逆復元し、親の現hash/ee84f8e差分0/変異残存0を確認済み。復元trial no-run成功後、新EXE対応Gateを経て独立focused36件を実行中である。最終Post未了なので、この時点を完了や最終応答までの総量とは呼ばない。

親の共有読取末尾token eventは10:36:19.664Z、input1667451930/cached1628204672/output3782274/reasoning1170166/total1671234204。開始実在snapshotとの差はinput67338178/cached65883392/output161535/reasoning75151/total67499713。inputはcachedを、outputはreasoningを含むため再加算しない。着手前後欠測・最終応答/最終review後の未反映・親子包含不明は維持する。子と合算しない。

終了側の共通週枠は58%/10080分/resetsAt1792496608、ordinaryUsageAllowed=true。開始22%からの表示変化36ポイントはアカウント共有値であり、このET2だけの消費量・料金・旧方式比には換算しない。利用者が指定したET2完了境界を維持し、次工程ET3には進めない。

役割累積の最新取得：実装者10:34:41.375Z input17759993/cached17468416/output56366/reasoning6147、reviewer10:33:46.329Z input3879052/cached3482624/output10325/reasoning1785、作者A09是正後09:28:37.564Z input4135479/cached4019456/output28823/reasoning5629。いずれも同role session累積、各工程開始/末尾応答/親子包含/provider実行metadataは未取得部分を含む。runner token/sessionは引き続き欠測であり、取得漏れを0や推定値で埋めない。

今回の記録だけでは自動役割切替が旧方式より使用量を増減したかは未判定である。既定の安全Gateや試験基準を下げず、復元後はtrial36だけを再確認し、同一bytesの通常/lib/関連252を再走させない。長い親文脈、検証密度、A09手戻りを含むため、小修正の基準測定と同条件とは扱わない。

最終結果：復元後36/36PASS、最終Post PASS/ET2 Closed。最終reviewerの10:40:34.586Z取得はinput4815373/cached4402304/output13010/reasoning2209。同session累積/開始欠測/最終応答と記録push後未反映/親子包含不明/provider metadata未取得である。runnerの終了週枠は58%、token/session欠測を保持する。成功済み252を反復せず、感度25を製品PASSへ加算していない。利用者指定の完了境界で終了しET3/製品EXEは未着手。最終判定正本は[egress-review](early-trial-egress-review.md)、実行正本は[runner](early-trial-egress-runner.md)である。
