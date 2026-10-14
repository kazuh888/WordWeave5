# ET2-P01-V 安全Gate・独立レビュー記録

2026-10-14。ET2-P01はAccepted、設計Preは[起動レビュー記録](early-trial-startup-review.md)のPASSを入力とする。本書の最初の記録は親のG0 source確認であり、独立Postや動作PASSではない。

## V1 source読取確認：no-runのみ許可・runtime禁止

HEAD `fef8bd2`＋作者/実装者4file差分。設計SHA `8AC25D82C1286ED1C4500BC3DFCF232E76DC01673E35758833C7E373C96F82F3`、plan SHA `BCD69DBC7B1EA022598374A93D967DE5DA8239427FF5443C3D55F5E2F5E3A785`。[作者報告](early-trial-egress-tests.md)SHA `3C1E4041AF0BD1542762DBC0FDC4C9E3AC42DF638B15551EA89F2EB6FB907E65`、[実装報告](early-trial-egress-implementation.md)SHA `3B864FEC753CB3B39EF44E2141BEAF75F3662BBBA1ACBC8788B521446BA1ED9B`。親が以下8sourceの現物SHAと作者4markerを計算し、両報告の固定表と全一致した。

| source | SHA256 |
| --- | --- |
| main.rs | 0A2D32C729CCA0139D9DC3C3CAB61BFC4C2660D8D46CCCB81EF53A40D6C5AE40 |
| app.rs | 4F1183841C9CA298F6F7F34B252CC225BA7AF0E770F3F5CFEA819D4CA9EAF962 |
| qwen_reading.rs | 3910E0042F057691F339A3AF7A59F6A381F7D0DB3D6B47706CAA1986D88CBB64 |
| app/qwen_settings.rs | 649499602DBA84093D69CB10937462633B0BAD722E16B55BD3C5D8BA19043AFC |
| app/qwen_settings_tests.rs | CE3F96E529D35042E4E5656EA054810050B3FF13F5EDEC19D21B15CCF48A1FDD |
| media.rs | 14914D6606950E3E9565EA43F9374B8B70A3C9CAB78D7F71419311176F9B3207 |
| ai.rs | A7A202579ABEE7C6F1B56DCC754101BDCAE73A4ABF715686006503534FDBFDF7 |
| app/qwen_reading_ui.rs | 4A1A0BCF5CA7764959D90426AA45FE62BFD939CF8E6FC7F5C6420449C09084F7 |

親はaiのglobal/全API/Drop/Lease、6Codex出口、Qwen wrapper/new/同期終了、default_store/start_probe、全正対照とmatrixのguard/cleanup経路を読んだ。

- 登録・Codex拒否・probe拒否・snapshot・capture・selected_storeは同一GLOBALを参照する。active時はlock内加算→固定Err、正常Noneだけ元処理、poisonは拒否である。全6Codexは実lib/journal/認証subprocess前、probeはrefresh/connection/worker前に戻る。
- Qwen wrapperは保持元Leaseへ同期加算し、inner.sendを呼ぶ経路がない。futureもLeaseを保持する。Ended/別owner/poisonでも旧counterへ加算して拒否し、新ownerへ混入しない。Dropは同Arc登録だけを解除し、poisonをclearしない。global lockをtrait I/O/awaitへ跨がせない。
- default_storeは登録済み合成Storeをfactory前から選択、登録なしは元の未設定mock、poisonはfixture異常で停止する。非testの実資格情報selectorは不変。正対照は全て合成Storeを登録成功必須とし、Codex実呼出/thread joinまでownerを保持する。
- QwenはOwnedDialogのDrop/明示終了でruntime.take→close→同期Runtime drop→audio job drainを行い、その後snapshotを検査する。blocking worker終了対照、保持TransportのEnded対照も確認した。matrixはconstructor前から両watchを保持し、catch_unwind後の同期終了と最終snapshotまで維持する。未知workerの場合はwatchをforgetして子終了まで登録を残しFAILする。
- 録音sourceはC3確認済みSHAと完全一致し、同global/cpal前加算拒否/poison/同ArcDrop/worker joinの実物確認を保持する。通信変更で旧実行解除は失効しており、今回のsource確認だけで録音/通信試験を開始しない。

確認済みV1から通常/試用binのno-runのみを許可する。前後8source一致、JSON profile/features/EXE絶対path/SHA、予定20実名を固定し親へ返す。作者の不足API71診断が解消したかは未確認。compile error時もsourceを自動変更せず診断を返し、必要な修正後はsource確認からやり直す。list/runtimeはEXE対応を親が明示解除するまで禁止。旧C3録音限定解除を通信へ流用しない。

動的安全対照/matrix/感度/独立runner/Post/ET3は未了。実data/key/API/mic/管理者/nativeは実行しない。

## V1独立静的レビュー：確定指摘0・最終Post未判定

fresh reviewer `trial_egress_review`（planner指定Astra/high、read-only）がfef8bd2からの4file差分を独立確認した。確定指摘0、要回答質問0、任意提案0。8source前後SHAと作者4markerは上表/作者報告と全一致、diff検査成功である。

単一global/正常Noneだけ通過/lock内加算/poison拒否/同ArcDrop/旧Lease分離、Codex6実lib前拒否/MaterialFailure型変換/サイズ上限、Qwen実appconstructor接続/Lease保持/innerへ復帰なし/同期Runtime破棄、factory前合成Store/probe先頭拒否、実設定ボタン・保存・復元・音読準備と累積0のmatrixを確認した。未接続旧ET06の置換以外の既存assert変更は未検出。未知worker時は登録を維持してFAILする。

compile/実名集合/通常・試用各20case/感度/独立runnerは未確認。Codex lib/実Reqwest自身のcounterはなく、固有拒否とsource遮断の組合せという範囲を保持する。静的指摘0を動的PASS/実行解除/ET2-P01 Closedへ読み替えない。reviewerの編集/試験/再委譲/公開0。

session `01a139ac-f26e-7fb2-bf3a-c7d827f8e3bc`。観測09:10:20.349Z input38202/cached23040/output293/reasoning0→09:13:49.355Z input1266517/cached1166208/output4094/reasoning528。着手直後/最終応答/親子包含/返却model・effort未取得。親は新動的証拠を受けて同reviewerへ最終Postを返す。

## V1 EXE対応確認・限定実行解除

親がbuild JSON現物とEXEのSHAを再取得した。両build-finished.success=true、profile.test=true。8sourceは上表から変更なしである。実装報告の更新SHAは `D96DC22B85C73AE24A3210AF0E91F6BBE5BA984E5C551B036A7C481F785BAB6A`。

| 構成 | JSON features | EXE（リポジトリからの相対path） | SHA256 |
| --- | --- | --- | --- |
| 通常 | `[]` | `target/early-trial-build/et2-impl/default-target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `4EA632D9CF0B8F2A818D82ECC4E64E5645A597177E8A7FBA1E7BA1018A891B11` |
| 試用 | `["early-trial"]` | `target/early-trial-build/et2-impl/trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `44B5D126EE01C879FEE2E7ABD755922738A8B2571D2550888ED8287C15128959` |

この固定組だけについて、実装担当のlist照合とfocused自己検証を解除する。作者指定順の録音安全対照3→通信入口/寿命対照→境界→実app matrixを通常・試用それぞれ実行する。各EXE起動直前にAdministrator=False・SHA一致を必須とし、Hidden・時間上限・合成専用TEMP/LOCAL/cwd・exact名と実行1件・終了wait/残存を記録する。権限true/不明、失敗、timeout、未知worker、source/EXE変更時は停止する。実key/API/mic/native/管理者起動・source編集・感度変異は解除しない。別buildと独立runnerは改めてGateを照合する。

利用者は週枠の選択肢1「現在のET2工程の独立検証・レビュー完了まで継続する」を明示選択した。ET3・製品EXE提供はこの中断境界に含めない。

## A09の検出力不足：是正待ち・最終Post未了

実装担当のV1自己検証は通常/試用それぞれP01 20件＋録音安全3件、計46件成功との報告を受領した。集計証拠は実装記録へ保存予定である。一方、感度準備でA09（ai.rsの`etp01_unwind_drop_releases_registration`）の`catch_unwind(...).is_err()`が内部count assertの失敗も意図したpanicと同様に捕捉することを発見した。親も当該sourceを確認した。V1 PASSだけではA09の「1を検出した上で意図unwindした」を証明できない。

独立作者へA09限定のoracle強化案を依頼した。実装者/runnerが作者assertを変更することは許可しない。独立runnerは起動済み通常bin no-run終了（exit0）のV1証拠のみ保持し、試用/lib追加build・list・runtime前で停止した。source変更後はV1解除を失効し、8source/4markerと新build/EXEを再照合する。旧自己検証を消去せず、是正後新証拠と区別する。

## V2 A09是正・独立no-run限定解除

作者がA09内だけ専用`IntentionalOwnerUnwind`型と`panic_any`へ変更し、catch外でpayload型を検査、未知payloadは`resume_unwind`で失敗を伝播するよう強化した。親が現物を読み、固有Err/count1/RAII解除/再登録全0が保持されることを確認した。ai source SHAは `6D61D919E159FF422F76BD9F1C95D1FE28E2583C43F4A75DD1F8BBC9509242EA`、ai作者markerは `E99CF0CC0C81160A0055A608F6843655A8BB3CD8A9A6566080373351D65329F7`。他7sourceは親の現物hashでV1一致、他3marker不変は作者報告である。作者報告更新SHA `09B46925BFA98E33A11850F1F3336284F15483B36AE2BF159F54EC005AFD4C7A`。

通信/録音実遮断の本体はV1確認範囲から無変更であり、安全確認を保持する。独立runnerの専用targetによる通常/試用binと既定lib対象のno-runだけを解除する。V1 EXEのruntime解除は失効し、新EXE/JSON構成/前後8source/4marker確認が親へ返るまでlist/runtime不可。作者是正後の実装担当による重複build/自己検証は省き、独立runnerで新証拠を取得する。

## V2静的追加確認と感度計画修正

独立reviewerはA09是正の確定欠陥0、是正部分だけをV1へ逆変換したhash完全一致を確認した。感度計画には必須P2を1件検出した。V1でA09を除いていたS1の誤0対象へV2 A09を戻す必要がある。親は指摘を受領し、S1へ`ai::trial_egress::tests::etp01_unwind_drop_releases_registration`を追加、counter期待1のFAILと未知payload再送出を取得する修正を実装担当へ依頼した。追加snapshotは不要である。

S1=(title/history拒否欠落/fatalApp/録音誤0/P01誤0)、S2=(fatalD開始/P01誤非0)、S3=(目的loader前早期失敗)の既定3snapshot分離には、他の静的阻止事項を検出しなかった。遮断/実加算/作者assertを保持し、変異と逆復元それぞれ新G0を要する。D群誤非0は最初のbefore-constructor assertionで止まり、後続stageごとの感度実測ではない。これは感度計画確認であり変異source/目的FAIL/最終Post/P01 Closedの実証ではない。

## V2独立EXE対応・runtime限定解除

親が独立runnerの`v2-default.jsonl`/`v2-trial.jsonl`からbin/lib各artifactを読み、profile.test=true、通常features=[]、試用features=[early-trial]と現EXE SHAを実測照合した。親の再計算で8sourceはV2固定表、4markerは上記V2 ai/その他V1と全一致した。実通信/録音遮断本体は既読V1不変、A09差分の現物読取と独立静的確認済みである。

| 構成/target | 独立runner専用target内EXE | SHA256 |
| --- | --- | --- |
| 通常bin | `default-target/debug/deps/wordweave5-72e8adffa8f9ef2c.exe` | `8F72838BA93B186D6F980A5157E440304AF18CCEA5B9BF1D81D58CBF8E52B969` |
| 通常lib | `default-target/debug/deps/wordweave5-b3388a7bbef44bb4.exe` | `1C174022959E71BDEB6EAE2DC7C9DA26121637C045469B7880DCEE0EC2816409` |
| 試用bin | `trial-target/debug/deps/wordweave5-e0ab9f88db3adadd.exe` | `D6DA249FF9AF7755F774CD9002D750E57BA325193B733D835F2F3167347659C0` |
| 試用lib | `trial-target/debug/deps/wordweave5-1e58dd7001d62083.exe` | `9920760395F7EF4BE4A50A569B9A91C9C93B62BB7766FB0E33953D73594DFBF1` |

独立runnerの上記4EXEだけlist/実行を解除する。list照合→録音安全3→P01入口/寿命→境界/実app matrix→従来ET2・関連回帰の順、実名集合はlist後に固定する。毎起動Administrator=False/SHA/Hidden/bounded/exact件数/合成TEMP LOCAL cwd/worker子終了wait/残存を要求し、最初の予期しない失敗・timeout・未知worker・source/EXE変更で停止する。実key/API/mic/native/管理者/追加build/感度変異は解除しない。
