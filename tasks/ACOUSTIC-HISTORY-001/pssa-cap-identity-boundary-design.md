# PSSA-CAP：C06参照同一性の限定観測設計案

2026-10-24。状態は設計案までである。[再開計画](unknown-descendant-resume-plan.md) PSSA-CAPのdesigner割当 `gpt-6.1-sol/high`、承認済みCAP-R1/R2/R3、[case観測設計](pssa-cap-case-observation-design.md)と[typed出力設計](pssa-cap-typed-output-design.md)を再利用する。親から受領した承認範囲はエラー採取の限定見直しだけである。実provider model/effort、usage開始・終了counter、親子包含は未取得である。所有は本書新規のみ。AGENTS/development-team/KPIを読了し、api-and-interface-designの入力・出力分離と境界契約を適用した。UIは変更せず、日本語描画・両軸ボタン中央・縮尺・狭幅は非該当である。

## 根拠と未確定

CAPは `target/early-trial-build/et3/cargo-gate/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d/pssa-capture`、NEWはその `typed-output-7c98c831-f8e8-42d3-91b9-60b54c1d11a2` である。NEW/PARENT-FROZEN.mdと、CAP/cap-v-typed-runner-evidence-e1c0e331-45e6-4319-a849-cd19d50b1b3c のreport.md/saved-index.jsonを限定参照した。

| 固定入力 | 読取時SHA256 |
| --- | --- |
| NEW/Test-PssaCapture.ps1 | `688FED51387B43BDCAB5E12EAA1709A0B08446130CB9186F351EBFC848F417F5` |
| CAP/Invoke-PssaCapture.ps1（C） | `39B5BC40CA83F5E088873B67047243182EF806D96BE8BC5D3372235808187613` |
| CAP/expectations.json | `0E20118B8C11714B21DEDE0A1AEC66EAA928FAFF1D845B83DB04847EAB7062C0` |

保存報告ではC01–05 PASS、C06のprimaryIdentityだけUnexpected、exit2、後14 NotRun、analyzer0/retry0である。sameErrorReferenceはrecordRefs[0]/[1]間の一致であり、fixtureのerrorとの一致ではない。C06のprimaryIdentityはTest590のresult.primaryErrorとmemory.errorの比較である。firstRaw/firstTypeがErrorRecordであっても参照同一性の証明にはならない。

現コードではC60が最初のErrorRecord `$Record` をstate.primaryErrorへ直接保持し、C77が同じ変数をrecordRefsへ追加、C465がstate.primaryErrorを返す。Testのcontext.errorはmemory.errorから初期化され、C06 producerはWrite-Error -ErrorRecord Context.errorを呼ぶ。したがって欠けている境界はfixture入力→最初の保持参照、および最初の保持参照→primaryである。どの境界で変わったか、PowerShellのerror出力経路が複製するかは未確定である。

## 最小契約と受入IDの対応

新CLI・logger・helper・純言語probeを作らず、将来の新Testコピーの共通casecatch、既存recordRefs観測blockだけで次の固定scalarを採る。C、本来のfixture/producer、Test-CapResultと期待値は変更しない。初回保持参照は既存state.recordRefs[0]を使う。C06の最初のrawがPipeline/Engine/ErrorRecordであり、元state seedのprimaryがnull、C60→77の間に別参照代入がないという既存コードとの対応に限り、これを受信関数内の初回保持点と位置づける。

`failureObservation.recordRefs`が既存処理によってhashtableとして生成された場合だけ、そのobjectへ `identity` を追加する。identityは下表4fieldだけを持つ新しい観測用objectであり、各値はBoolean又はnull、初期値はnullである。既存count/firstTypeの生成条件・値・欠測を保持する。親recordRefsがnullの場合はnullのままとし、代用containerやcount=null/firstType=nullを作らない。

| 受入ID／観測field | 元参照の直接比較・責任 |
| --- | --- |
| CAP-R1/R2・C06／memoryContext | `fixture['memory']['error']` ↔ `fixture['context']['error']`。fixture生成境界のanchorを確認する |
| CAP-R1/R2・C06／contextFirstRetained | `fixture['context']['error']` ↔ `fixture['state']['recordRefs'][0]`。fixture入力から受信関数の最初の保持点までを確認する |
| CAP-R2・C06／firstRetainedStatePrimary | `fixture['state']['recordRefs'][0]` ↔ `fixture['state']['primaryError']`。state内保持を確認する |
| CAP-R2・C06／firstRetainedResultPrimary | `fixture['state']['recordRefs'][0]` ↔ `actual[0]['primaryError']`。Complete後の返却境界を確認する |
| CAP-R3／既casecatch・保存経路 | 元Unexpected/元caseError、firstUnexpected停止、後続NotRun、exit2、既時計/上限、一回保存を保持する |

各比較はその場の `[object]::ReferenceEquals(<上記左のindex式>, <上記右のindex式>)` の返却Booleanだけを新objectへ代入する。参照を返す関数/callback、pipelineへの出力、ForEach/Where、collectionの再列挙、参照側の型cast、配列化、ToString、PSObject展開、Serialize/Deserializeを介さない。hashtableのContainsKey、既知IListのCount/index、ErrorRecordの `-is` による検査だけを使う。全operandは非nullの実ErrorRecordを要求し、null同士のReferenceEquals=trueを採用しない。actual比較はactualObtained=true、元actualがobject[]かつCount=1、index0がhashtableである場合だけ行う。sourceの移動・上書き・新source参照配列は不要である。

追加処理は既recordRefs blockの元取得処理とtry/catchが終わった後、同block内で行う。親recordRefsがnullなら比較getterを一切呼ばず、元の `recordRefs:NotObtained/WrongType/GetterFailed` を保持し、既root missingへ固定一項目 `recordRefs.identity:NotObtained` を追加する。この場合4fieldは存在せず、identity全体が未取得である。memoryContextだけ別containerへ救済しない。親が生成済みならidentityを4nullで初期化し、各fieldを独立したtry/catchで取得する。ここでいう独立性は、親の存在を共通前提としたfield間の独立性である。recordRefsのCount=0やfirstType取得故障でも親が既に存在すれば、memoryContextは自分のoperandだけを検査して取得でき、他3fieldの不足に連動させない。

親生成後の各fieldの未取得/不足/型違い/取得故障はnullのまま、既存root missingへ `recordRefs.identity.<field>:NotObtained/Missing/WrongType/Null/GetterFailed` を一つ記録する。falseは両operandを取得して比較できたという実測であり、missingへ入れない。最初の観測故障だけを既存obsFault/captureFailureへ渡し、元caseFailureやCのprimaryを置換しない。取得のための再試行はしない。root missing/truncated上限64、Overflow、既存期限判定と短形、64KiB JSON上限を継承する。既Deadline/ByteLimit等の短形では従来どおりrecordRefs=null、identityも不在とし、既短形の固定missingだけを保持する。短形へidentity専用項目や4nullを足さず、値を推定補完しない。追加block直後にも元caseStart/startTickを使って既5秒/30秒を確認する。hard取消保証は依然Unavailableである。

## 状態・所有・保存・回復

状態は既存の `fixture未取得→fixture取得→actual取得→既assertion→Unexpected→bounded観測→既保存` だけである。新しい成功状態やgateを作らない。参照はfixture/state/actualの既所有者が同期case内で保持し、観測は4個のBoolean/nullだけを所有する。async・別thread/runspace・cancel callback・新timeout・停止順変更はない。保存後のJSONから参照同一性は復元できず、既保存raw/resultの内容一致で代用できない。

旧Test、失敗結果、Partial、request/return/index、B/R/patchは全bytes/hashを非上書き保全する。新観測も既unit-result.jsonの一回CreateNew経路へ同梱するだけで、memory側raw/resultの保存順・producerWriteError・partial処理を変えない。保存失敗・期限・欠測は既方式で不完全のまま報告し、後続実行や再採取で回復したことにしない。binding/caller/freeze接合は後工程の別所有であり、本設計で編集しない。

## 判断限界と引継ぎ

既fixture内観測で受信関数の保持点までの二つの欠測は埋められるため、独立した純言語probeは現段階では不要である。probeは別実行の言語挙動を示しても今回の参照鎖を直接証明せず、許可実行面を増やす。観測に合わせてprimaryIdentityの期待を変更する案は、検証する契約自体を変えるため採らない。

一方、recordRefs[0]はAddのparameter binding後の参照である。producer内部→Write-Error→合流pipelineの `$_` →Add引数bindingという内訳は本設計では分離できない。contextFirstRetained=falseだけでWrite-Errorを原因確定してはならない。要件が「Add保持点」でなく「binding前のpipeline初受信そのもの」の直接観測を意味するなら、C不変という制約下では不足であり、仕様担当/親へ返す。CへのhookやAST書換えを黙って追加しない。4値が同時に整合しない場合も、診断側の取得/binder境界を含め未確定として止める。

次工程の独立test authorへは、この4fieldの真/偽/nullの区別、各欠測理由、初回保持点の意味、元C06 primaryIdentity不一致をそのまま渡す。実装対象を新Testコピーの共通casecatch/既recordRefs blockとroot短形の整合だけに限定し、許可diff逆適用で688FED全bytes/BOM一致、C/oracle/producer/期待/停止/保存不変を独立Preへ渡す必要がある。最小test seamは比較operandを保持する既hashtable/IListとscalar観測objectであり、元20期待は固定する。新runtime、AST実行、PSSA、依存変更、昇格、隔離外、実data/API、製品EXE/buildは今回0。将来の受入証拠は元case判定と観測Complete/Partial/Unavailableを別計上し、試用EXEや元NRE解消へcreditしない。新実行には別途の計画接合・独立Pre・親freezeを要し、本設計案だけでは解除しない。

今回の完了境界は設計案の保存と文書検査までである。追加runtime・テスト編集・C編集・再委任・commit/公開は0。開始時dirty5件を保持した。使用量欠測は親のKPI記録へ引き継ぐ。
