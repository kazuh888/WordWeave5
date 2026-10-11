# M2 / W1 独立gate記録

最新判定（2026-10-08）：W1・CANCEL-A1・I5-Q1/M2は各限定範囲のPost PASS/Closed。A1/M2の最終証拠と残余範囲は末尾を正とする。以下の保留・待ち状態は当時の履歴である。

2026-10-07。二つの修正単位を別判定とする。reviewerは既定のgpt-6-astra/high、`praat_m1_review`。親・designer・reviewerの実行metadata/使用量counterは未取得である。

## Pre cycle 1

- M2提出：praat-m2-design.md `A3B9F3C6D4C059A568220CAB7C76BFCFF9816A450D16C95ABB471BB6AE23E880`。CHANGES_REQUIRED。競合writerのhook内closeではpin取得前に競合が消えるため、pin試行後/cleanup前の解放点が必要である。回帰コマンドには既知環境制約のM1を正確にfilterする必要がある。方式そのものへの阻止指摘はない。designerへ返却した。
- W1提出：history-w1-packet.md初版。CHANGES_REQUIRED。Storageをprepare/publish間で作り直すとwriter sessionが変わりConflictとなる。同writerで試験するよう訂正した。親も同じ問題を独立確認した。
- 本体・試験はこの時点で未変更である。旧source/testのbyte snapshotを `artifacts/m2-before/` と `artifacts/w1-before/` に保存した。

## 再提出

- W1：§8を訂正し、同writer維持、history/target全bytes比較、app.lockだけ外側のbefore/after比較を規定。SHA `1613BE0A2308583F20495AD37A3E5425004EEDB5A2A2F886330157F9985E093C`。独立Pre PASS、残阻止0。`history_w1_tests`（指定Astra/high）が独立試験作成へ着手した。
- M2：競合writer解放用WavPinAttempted同期点とエラー優先、正確なM1 filter、private API名を訂正。SHA `B84ACBC6E727898E760B4738F069D3AB033DAC6F06B7B81F09F64C90C4E8BFFB`。独立Pre PASS、残阻止0。`praat_m2_tests`（指定Astra/high）が独立テストを先行作成する。

Pre PASS前に実装しない。Post PASS前にClosedとしない。M1のClosed、全機能未完了、製品EXE未更新は不変である。

## W1 Post — PASS / Closed

独立test authorによる追加2件、runner（gpt-6.1-sol/medium）による全37件の新規実行成功を受領した。[作者](history-w1-tests.md)、[runner](history-w1-runner.md)。reviewer `praat_m1_review` が現物を独立確認しPost PASS、残阻止0と判定した。元testの先頭58,832 bytesは不変、本体変更なし。実junctionと同writerによるUnsafePath、全path/bytes、追加object 0、原音/intent/head/app.lock保持、安全unlinkの全oracleを満たす。I1b-W1の受入証拠不足だけをClosedとする。

## M2 Post — 保留（新しい回帰失敗）

M2本体はT1/T2の行動RED→独立15件GREEN。限定差分175行追加/8行削除、静的reviewは阻止0。独立runnerでも新15件・公開Praat20・core32/schema27・default59は成功し、60秒48kHz fixtureは5,517msだった。[実装](praat-m2-implementation.md)、[独立runner](praat-m2-runner.md)。物理メモリは今回未測定である。

ただし、feature全targetのunitは48件成功後、既存 `process_unit::cancel_kills_child_and_descendant_preserves_sibling_and_existing_data` の所有handle待機がWAIT_TIMEOUTとなり失敗した。mutexを保持中のassertでpoisonされ、Dropのunwrapもpanicしabort、19件未到達。一括gateは不合格である。同EXEのHidden exact1回と直接exact1回は成功したが、原因の説明にならず、最初の失敗を撤回しない。

reviewerはM2最終Postを保留した。fixtureの二重panicは確定欠陥、終了確認失敗の本体帰属は未確定である。Job ActiveProcesses=0と各process handle signalの厳密順序を断定せず、同定・終了・回収の診断を別Packetで扱う。通常の固定sleepや待機oracle緩和はしない。`artifacts/cancel-before/` に本体・test・childと失敗時EXE/childの現物を保存した。crate fmtの既存lib.rs module順序差分は、診断中のsource固定のため未修正である。

bounded source lookup用ww-scout（固定Terra/medium）はcapacityで起動失敗した。親が該当sourceを読み、複雑な終了保証判断は既定Astra/high reviewerへ返した。モデルを黙って切り替えていない。

## CANCEL-D1 Pre — PASS

原本を固定し、隔離copyのfixture2本だけへ診断を追加する。[D1正本](process-cancel-diagnostic-design.md)。cycle1の指摘は、共有Fixtureの複数起動/重複trackに対してrecord上限が不足することと、lib全unitの実Praat path指定が条件付きだったことである。call世代・所有・上限と、固定Praatの絶対pathを必須化して訂正した。

cycle2 SHA `4ADA492BCC842DF786E314943DF65E2B5DDCF3F8C23C4C77ABF865A73CA17B99` に対し、独立reviewerのPre PASS・残阻止0を受領した。`cancel_d1_tests`（指定Astra/high）がcopy準備・fixture2本・自己試験A～Eを所有する。自己試験後の独立安全確認まで全unit再現は実行しない。外側180秒は、既測M2約51秒を含む診断一括用であり、既存の内部期限を変更しない。D1で未再現でも原本M2を合格にしない。

## CANCEL-D1 安全review cycle 1 — CHANGES_REQUIRED / 追補Pre PASS

隔離copyの自己試験9件と既存共有fixtureの2caseは非管理者sandbox外で成功した。sandbox内のpublisher試験はrenameのcode5により失敗し、ログを保持した。原本17ファイル不変・copy変更2fixture・M1 oracle不変を親とreviewerで確認した。[作者引継ぎ](process-cancel-d1-tests.md)。

独立安全reviewで、初回snapshot後のwatchdog登録競合と、cleanup未確認tempが自動削除される2件を検出し、全unitの実行を止めた。D1正本§12へsnapshotと同一lockでの世代seal、late handle close-only、cleanup確証とtemp保持、決定的な回帰試験を追補した。

SHA `591DFF53337775E1510C2D28C417F81D5313BB81F4F517F9B632C878894D1457` の追補に対し、同reviewerの差分Pre PASS・阻止0を受領した。作者へ隔離2fixtureの修正を返した。修正後の関連自己試験・固定hash・再安全review PASSが揃うまで全unitは未許可である。初回0ms oracleと既存期限は不変である。

## CANCEL-D1 再安全review — PASS / 一回診断へ

2件の修正前REDを保存し、R2自己試験14件と共有fixtureの2caseが成功した。独立reviewerは固定source `4A5BCB21166F1AACAE17F9A447EF0D738FEAAADC8DC0CB3A9EE2654327E91EEC` / unit EXE `7F37ABED714DE2F81A535494024D733AABE9FA8ED7DCCE5DF9BAFAE7995C66BC` を照合して再安全PASS・残阻止0と判定した。原本17件不変、隔離差分2fixture、M1 oracle不変である。

`cancel_d1_runner`（指定gpt-6.1-sol/medium）へ設計どおり固定Praat検証・非管理者consoleなし・M1 skip1・外側180秒の全unit一回診断を割り当てた。実行予定82件、元のM2一括失敗のClosedとは別である。結果待ちである。

## CANCEL-D1 一回診断 — 元cancel未再現 / 別caseの欠測あり

[独立runner結果](process-cancel-d1-runner.md)：82 PASS、0 FAIL、M1 filtered 1、52.83秒、exit0。元cancelは同定済みrootと孫2observerの初回0ms waitが全0、reap=false、watchdog=false、救済なしだった。原本17件と隔離入力の前後hash不変。元のWAIT_TIMEOUTの原因を確定・否定する結果ではない。

別tree-exit caseは孫handleを取得できず、cleanup未確認として専用temp `F:\TempCodex\ww-praat-process-日本語 Z32XRK` を保持した。集計PASSと全回収確認済みを同一視しない。未知process探索・kill・temp削除はしていない。

独立reviewerへD1最終判定を返した。同定済みhandleの初回258が再現していないため、現段階でD2を必須としない。次はCANCEL-A1の別Packetとして原本fixtureへ安全補正を採用し、tree-exitの同定欠測を試験側同期で解消する設計を既定designerへ依頼した。原本の変更・新規一括検証・M1外部再確認・M2 Postは未完了である。

## CANCEL-A1 Pre — PASS / 原本fixture採用中

D1の限定診断完了・原因未確定・別case欠測ありという最終判定を独立reviewerから受領した。[A1設計](process-cancel-adoption-design.md) SHA `61500C17AC214258EA1B79F9D8AF98724FB1AEA7A1EE5B6CC036F470B3DB4122` はPre PASS・阻止0。原本2fixtureへR2を採用し、tree-exitだけに登録済み孫handleとidentity付きACK/readyを要求する。lib.rsの既存module順fmtは別の逐次統合とする。初回0ms oracle・期限は不変である。

編集前3sourceは `artifacts/cancel-a1-before/` へ保存済み。test author `cancel_d1_tests`（指定Astra/high）が原本2fixtureを所有する。focused自己試験と実tree-exit/cancel後に安全review、新規回帰、同一最終EXEの外部M1、Postの順とする。旧cancel原因未確定は成功後も記録に残す。

## CANCEL-A1 安全review cycle 1 — CHANGES_REQUIRED

初版は新6件/R2採用14件と実tree-exit/cancel/共有fixtureが成功、ledger6→6、89 tests。[作者](process-cancel-a1-tests.md)。lib.rs module順だけの4行移動とcrate fmtも完了した。[統合](process-cancel-a1-integration.md)。

親の統合確認で、正常なACK公開後からevidence記録までにwatchdogが中断されると、rootが正常exit0しても先行sealにより偽失敗となる競合を検出した。reviewerも成立を確認し、先の静的阻止0を訂正してCHANGES_REQUIRED 1件とした。全unitは未実行で保留する。

A1正本§13へ、公開前Prepared slot確保、初回0ms値固定後だけのpublisher完了回収（既存call deadline内、watchdog維持）、ready/exit0による因果確認と真のlate拒否を追補し、差分Preへ提出した。fixture修正前、fmt完了sourceは固定中である。

## CANCEL-A1 R2 — 差分Pre PASS / focused成功 / 新しい安全review待ち

§13 SHA `4E951D34056360F84221BF8DDA6EB1D46E12D07AFD06D0B756CF30C07337B4FB` は差分Pre PASS。修正前に正常ACK公開・root exit0・初回全0でもMissingAckEvidenceとなる決定的REDを記録し、R2へ修正した。focusedは新5件＋A1既存6件＋R2採用14件＋実tree-exit/cancel/24call/起動前拒否各1件、計29件成功。一覧94件、ledger6→6である。

固定process_unit SHA `2009A2B7EED04A5C07E95EFFF11A12B773B8BDA533796475B1FBF61380B38425`、praat_child `14EAF8EE8A55F040DD63989E82DD2E9F96A900ED4AEAECB8489082F1361ED356`、lib.rs fmt `832864EFFB903CCC97D0E7BB5B42DCCC92F7D4B7EF5584D3A3F4BEC87F33452B`。親が製品process2本・M2 source/test不変も照合した。

先の設計順序への先入観を避けるため、既定Astra/highの新しい独立reviewer `cancel_a1_fresh_review` へ限定source/証拠だけ渡した。作者はsource所有を返却しreport固定中、親はinventoryと状態記録を統合する。全unit/M1は未実行であり、再安全review待ちである。

## CANCEL-A1 R2 再安全review — PASS / 最終回帰へ

新しい独立reviewerは固定source、設計§13、作者最新追記、RED/GREEN生ログを照合して安全PASS・阻止0と判定した。Prepared/Complete、初回値固定後の残deadline回収、有限seam/Drop解放、poison/初回258保持、次世代slot、root側ACK/ready/exit条件を確認した。最終受入ではなく全unitへ進める判定である。

既定Sol/mediumの `cancel_d1_runner` へ原本の最終回帰と実行物固定を割り当てた。unitは93件＋M1 filter1予定。公開targets/no-default/fmtも新規確認し、同一最終unit/childをその後のM1外部確認用に固定する。M1とPostは未完了のままである。

## CANCEL-A1 最終回帰 — PASS / 外部M1前

[独立runner](process-cancel-a1-runner.md)はfeature unit93/Praat20/core32/schema27の計172件、no-default59件を全成功と報告した。M1のみ明示filter1、crate fmt・差分whitespaceも成功。source17件と実行前後unit/child hash一致、残存ledger6→6・増分0を確認した。通常cancel/tree-exitは保持handle初回0msが全0、救済なし、tree-exitのACK/ready/exit0も確認した。configurationを跨ぐ重複caseをユニーク件数とは扱わない。

最終unit SHA `D923AFE3CE225EC3A48AC09E39B8D9B4C798E298481022F07B8DFC7AD4206A59`、child SHA `8438841AF3D2853AB55C5DC89B82D135DF8BA254890EE0867076EC601565BAF2` をbyte snapshotと共に固定し、外部M1後まで再buildしない。作者focused EXEとはhashが異なるが、runner最終buildの実測前後値でありsource17件は一致している。独立reviewerへ外部確認への引渡し判定を返した。外部M1とA1/M2 Postは未完了、旧cancel原因未確定は維持する。

新しい独立reviewerはrunner報告SHA `0FBCCFE864BFE9713976D5FFD8ECB17A626E18C141960F19BFDBC74F7D04262B` と主要生ログ/metadata、source17件の現在hash、最終2EXEを読取照合し、外部M1への引渡しPASS・阻止0と判定した。実行/buildはしていない。利用者へCodex外PowerShellでのexact1件を依頼する。過去のM1成功を新EXEの受入へ流用しない。

## CANCEL-A1 / I5-Q1・M2 最終Post — PASS / Closed

2026-10-08、[更新版の利用者M1結果](process-cancel-a1-user-green.md)を受領した。1 PASS/0 FAIL/93 filtered、0.11秒。外側Jobなし、作成時子Job所属、救済前child終了、未resume、兄弟生存、watchdog非救済を満たした。受領後もsource17件・最終unit/child hashはrunner固定値と全一致した。

既定Astra/highの独立reviewer `cancel_a1_fresh_review` は、既存安全PASS/最終回帰PASSにこの新しい外部証拠を加え、**A1 Post PASS/Closed可、I5-Q1/M2 Post PASS/Closed可、受入阻止0**と判定した。編集・build・test実行は行っていない。親が本記録とclosure matrixを同期しClosedとする。

M2については既存静的阻止0を入力に、praat.rs SHA `4BBCB967106126A7DAB101EF99E5128C38BA60081A65E102324248404E97EC53`、praat_unit.rs SHA `BD1D3ADF3524F11698538F29ED023D960D7B69CA3A7FABD326C6FCE885A899B1`の不変と、書込digest・同一read pin上の長さ/digest照合・pin保持・取消し/期限優先を限定照合した。独立15件、新unit93件、公開20件、no-default59件、fmt、外部M1が揃い、保留条件は解消した。

| 対象 | Closedの範囲 | 残余・対象外 |
| --- | --- | --- |
| CANCEL-A1（A1-01–06、§13） | fixture同定・poison/cleanup・ACK記録競合の安全補正、決定的RED/GREEN、新回帰と外部M1 | 旧cancel初回258の根本原因は未確定。D1欠測temp等を保持 |
| I5-Q1 / M2 | WAV書込後からread pin間の同長差替え検出、検証済みpin保持、対応独立試験/回帰 | I5c-2、同期OS I/Oの厳密上限、実work peak・物理メモリ・child peak commitは未検証 |

新ログの最大60秒fixtureは5,033ms。23,040,058 bytesはcounting-writer試験値であり、実work peakやメモリの測定値ではない。外部M1の実行時hash・開始時刻・PowerShell版は未取得で、指定pathの利用者原文と前後の現物hashによる受入である。M1待ちの再build停止は解除するが、旧snapshotは保持する。I5全体・機能全体・製品納品を完了とはしない。
