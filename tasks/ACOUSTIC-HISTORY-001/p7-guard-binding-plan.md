# P7-D4 guard接合・限定計画

2026-10-18 / ACOUSTIC-HISTORY-001 / P7-D4-GUARD。状態：設計・独立Pre PASSを受領し精緻化済み、親の本版受領後G2へdispatch可。開始HEAD `fea71cb96053b31c069f3734249b1306bfcdf6f6`。利用者の「検査の厳しさを維持して限定設計し、独立レビュー後に修正・検証を進めてください」を本単位の根拠とする。

## 規範範囲と証拠

本書をこの単位の唯一の計画・task-listとする。既存[plan](early-trial-gate-plan.md) SHA `FF96F8DE80D3E7602F85F7846AF3C8C4C45AB1FF7A4B3F0806AE6A3DCEF817D2`、[design](early-trial-gate-design.md) SHA `6F84598858766BA08CFC236B8B847598705BDDD3A3000F4F1AB9BA07938236A7`は凍結し、編集しない。本書と限定設計追補が優先するのは、P7-D4の「6literal限定」の所有範囲へFF230の本文保全hash literalを一つ加えること、その出典証明と限定検証だけである。他の旧契約・期限・停止条件・承認範囲は継続する。追補の完全SHAを親の新manifestへ一方向に加え、上流Context/Slash等へ逆埋込みしない。

唯一の確定[限定design](p7-guard-binding-design.md)はSHA `184289A35BF03FD4101789E39120F4F9E159B4947BB30AB195A7314F7E879A4A` である。親からこのdesignと仮plan `835C7A749F9D501142966C8D75DE30E987406223875459382FB5123E241BCFCA` に対する独立Pre PASS/阻害0を受領した。design内の仮plan SHA・提出時状態はレビュー入力の来歴として保持し、designを編集して追従させない。本版はその設計を計画へ接合するもので、設計契約を変更しない。実script/runtimeは未実証である。

`R=F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate`、`P=R/verification/argv-path01`。以下のsource帰属は親・独立reviewerから受領した証拠であり、plannerによるruntime再実証ではない。

| 対象 | 固定証拠・意味 |
| --- | --- |
| 未編集FF | `P/Test-FirstFailure.ps1` SHA `AD492926E4DC4CA84766710D41CFF9024065A88D79BAF310226B96026DAECD46`。既存6literalも未適用 |
| 現Invoke | SHA `898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71`。今回変更対象外 |
| 元guard | Read-OwnedTree内の2診断callの末尾4引数だけを除く元算法。旧期待 `08600941CB5669B76A9EB938A40B7362203967C2ED167C00D2F2DA6EA5B03E5E` |
| MI承認後本文 | 同算法値 `3768A05D1C6B65031492F1C5F8775900B636E5D1AB4AEDDDF5B0F3463AE0F3AF`。保存packet `P/a1e2d0c5-5d2f-4d6e-97a2-0b6dea11cfea` の旧Invoke33C2→0860、新318E→3768。新318Eと現898Cの関数全文はOrdinal一致、版literalは0個 |
| 停止証拠 | `P/35186079-b5d6-4e51-b19e-a608b67dadc9/REPORT.md` SHA `C1C255DEE6D5688691D5F8903C2CE28771323D2A7127A259027EEB2813E0013A` |
| E準備のみ | `P/f3f85d34-7dbf-49ff-81b7-82d1d756c7d4/P7-E-DRAFT.md` SHA `84D46E7C9F68939B5CED3C354D1EF129AB28F21E23069A350514D9455593C7E5`。実行0、最終freezeなし |

帰属差分は既知process欠測証明の1477–1559である。承認済みMI変更と旧FF本文保全guardの未接合であり、7.6.7版literal変更ではない。現在hashと一致するだけでは更新を承認しない。保存MIの旧/新入力、承認記録、現関数との一致を完全hashと差分で結ぶことが先行条件である。

## 要求・完了境界・除外

- GB-R1：元guard算法・抽出/一意性/型/診断call検査をそのまま保ち、独立承認済みMI本文だけに固定期待値を接合する。実行時hash自動採番、複数値許容、失敗時fallback、無関係な本文変更の受理は禁止する。
- GB-R2：旧D4の6literalと追加1literal以外のFF全bytes不変を逆適用で示す。FF18期待、元診断、各5秒/全60秒、cleanup、非所有sourceと旧証拠を保持する。独立否定対照とPostで未知本文変更の拒否を確認する。
- GB-R3：D4限定ClosedとEへ渡せる固定組を完了境界とする。E実行は既存別工程であり、本単位の成功をE58期待＋ValidateOnly2のPASSへ換算しない。D1–D3 Closed、C16の限定Reuseは保持し、新freeze/前後hashを省略しない。

製品機能、画面、保存形式、API、認証、process所有・救済判断には変更がない。検証側の固定本文同定だけを接合するため外部仕様P7-S01–05と既存FF/MI契約を再利用し、外部仕様役の再起動は不要である。設計で観測可能な契約変更が必要と判明した場合は外部仕様へ戻す。

旧F3/UNKNOWN-DESCENDANT、ENV01、focused10/残9、標準80/variant、製品full/build/試用提供、実API/実データ、admin/OS/PATH変更、native/fake/V2/製品source変更、新frameworkは未解除。凍結plan/design、既存未完了`tasks/plan.md`・`tasks/todo.md`、元FF/MI/RED/旧manifest/runは保全する。

## 作業単位とtask-list

依存は 完了G1設計/独立Pre/本計画精緻化→親の本版受領→G2作者→独立静的review/親freeze→G3別runner→独立Post→D4限定Closed である。E前提の保存baseline照合は後続runnerが担当し、通常補助1役ずつ・再帰委任0を守る。

| 単位・規模 | 所有と受入条件 | 必要verification・checkpoint |
| --- | --- | --- |
| G1 限定設計、S | designerは新規`tasks/ACOUSTIC-HISTORY-001/p7-guard-binding-design.md`のみ。旧/MI後/現行の証拠鎖、追加1literalの位置・元値/新値、有限対照の抽出境界・期待・実行入口を確定する。plannerは本書のみ精緻化 | 元算法の2call/各7要素/固定分類/Unprovable参照/逆順置換/UTF-8 hash、旧0860とMI後3768、現関数全文一致を保存証拠へ対応付ける。独立Pre PASSと親受領までFF編集0 |
| G2 D4の7literal接合、S | 独立test authorは`P/Test-FirstFailure.ps1`の旧6literal＋追加本文hash1literalのみ、design §5の候補root `P/d03fbe3a-1318-4b39-8ba5-110b588e1640` 内だけを所有する。保全copy・driver・3fixture・manifest・抽出/逆差分/静的証拠をCreateNew保存する。source実装者は変更0 | design §1/3–5準拠、7literal逆適用→元FF全bytes一致、非所有hash前後一致、元18期待/診断/期限/guard構造不変。変更/新ps1にParser0・PSSA1.25.0全通常rule診断0/engine0/marker/実exit0。runtime観測0、独立静的review後、親が完全SHA/argv/新UUID/期待をfreeze |
| G3 guard限定検証/Post、S | 別runnerは新UUID内の結果/logのみ。承認された最大3観測を各一回実施し、最初のunexpectedで停止。reviewerは読取のみ、親が報告を保存 | 下記有限対照、入力/環境前後整合、拒否位置・実exit・件数・未実行を独立受入。D4 bindingと追加guardだけClosed。FF18は既存E順序に残す |
| E前提接合、XS | 後続runnerが`toolchain.json`/`V2Case`の既承認保存証拠を読取照合し、正確なpath/完全SHA/承認由来/consumerを新UUID内のE準備報告へ記録する。親が既存草案/manifestへ接合する。現物・保存証拠は編集0 | 承認baselineから現物への一致を証明する。存在・現在hash採取だけでは承認にしない。由来欠落又は不一致ならE前で停止。D4検証をE全体の解除にしない |

- [x] 仮計画：担当規約・planning skill全文、KPI、現状冒頭、既存P7計画/設計を確認した。
- [x] G1：限定設計・独立Pre PASS/阻害0の親受領・本計画精緻化。本版の親受領をG2 dispatch前条件とする。
- [ ] G2：作者7literal接合・静的検証・独立review・親freeze。
- [ ] G3：別runner有限対照・独立Post・D4限定Closed。
- [ ] E前提：保存baseline2件と最終組を接合し、既存E担当へhandoff。E自体の実行/完了をこのcheckboxに含めない。

## 設計済みoracle・command索引

確定design §1が旧/MI後/現行の承認証拠鎖と7literal、§3が元consumerと8関数閉包・前後validator保持、§4がG01正常/G02 MI内変異/G03 MI外変異の正確なoffset・3期待・sentinel/拒否分類を規定する。独立Preでこれらとparse/consumerSHA/元算法/各5秒・全60秒/保存exitを確認済みである。規範を本書に再複製せず、作者は同designを消費する。旧guard不一致はFF18行動REDでなく、動的3観測はG3まで未実証である。

確定commandの唯一の参照先はdesign §5である。driver/候補rootとrunner結果root `P/ed063a7c-e915-4126-84a7-d882e66fd8f1` は予約済み、既存なら上書きせず停止する。G2はdriverを作成し静的検証まで、G3は別runnerが親の外側freezeと照合後、一commandでG01→G02→G03を各一回実行する。manifest SHAはG2成果物確定後にのみ親承認64hexへ展開し、完全argvをfreezeする。現時点で未展開commandを実行しない。

E前提baseline2件の読取候補は[review](early-trial-delivery-review.md)冒頭「親のE前提read-only確認」と `P/cxt-runner-20261016-approved-b` のaccepted-b証拠索引である。親の照合結果を後続runner/reviewerが確認してE freezeへ渡す責任を維持する。現物hashの新規承認や旧CXT-D runtime/PASSのEへの転記はしない。

既存EではContext12→MI MissingOs2→成功時MissingPure20→FF18→EO6→ValidateOnly2の順序・期限・停止を維持する（C16は既承認の限定Reuse）。G01–03をFF18件数へ加算せず、G段階でFF18を前倒し実行しない。E草案の実行可否はD4 Postとbaseline2件の由来確認、親の最終freezeに依存する。

## モデル・責任・独立性

以下はG1/G2/G3/E前提接合の各単位に共通する全7責任の割当である。非該当役も識別するが再起動しない。指定は利用可能なexact model/対応effortであり、起動前に親が再照合する。親model/effortと品質gateを変更しない。

| 役 | model / effort | 起動範囲・難度/リスク・選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 本書と設計後精緻化。中/高：凍結上流と新追補の優先範囲を制限する固定役 |
| external-spec author | `gpt-6-astra` / `high` | 既存仕様を再利用、今回は非起動。契約変更が判明した場合だけ差戻し。中/高：固定役で安全契約を維持 |
| designer | `gpt-6-astra` / `high` | G1追補と逸脱判断。高/高：承認由来、guard強度、抽出検証の循環を解消する |
| implementer | `gpt-6.1-sol` / `medium` | 全単位でsource編集非該当、非起動。低/中の既存D2/D3責任を維持し、test所有を代行しない |
| test author | `gpt-6-astra` / `high` | G2のFF7literalと独立有限oracle。高/高：期待値接合が検査の自己正当化にならないよう独立出典を固定する |
| test runner | `gpt-6.1-sol` / `medium` | G3の凍結済み最大3観測、後続E前提baseline2件の読取照合と既存E担当。中/高：有限手順・firstUnexpected・証拠保存を実行し期待/ソースを変更しない |
| reviewer | `gpt-6-astra` / `high` | G1 Pre、G2 binding/static、G3 Post、E前提の独立確認。高/高：新hashの由来と未知差分拒否、旧credit混同を検査する |

同じモデルでもdesigner/author/runner/reviewerは別責任・別担当とする。親dispatch、fresh context、通常補助1、再帰0。既存役を再利用し7役の一括再起動・merge/report専用役追加はしない。指定model/effort/必要skillの利用不能はblockerであり黙って代替しない。起動指定とprovider返却metadataを別記し、後者未取得を推定しない。役の既存skillを担当範囲で適用し、PS編集/監査では該当PowerShell skillを全文確認する。

## 停止・rollback・次handoff

新値の承認由来欠落、保存snapshot不一致、7literal以外の変更要求、未知validator、品質診断、予期外結果・欠測・hash drift、追加権限・契約変更で止める。自動retry/skip/guard削除/許容集合拡大で前進しない。baseline2件の未解決はE前提のblockerであり、無関係なsource変更の理由にしない。

rollbackは親が本単位の候補採用を取り消し、元FFの保全copyと差分を照合して所有変更だけを戻すこととする。他者差分をresetせず、失敗候補/旧新証拠/追補履歴を保持する。元FFへ戻るとD4停止状態であり、旧runtime実行や旧F3再開の許可にはならない。OS rollbackは行わない。

次dispatchは親が本版を受領した後、G2独立test author `gpt-6-astra/high` へ行う。exact所有はFFの7literalと上記候補rootだけである。入力は本書・確定design・既存凍結P7境界・保存MI承認証拠・停止REPORTの6literal表・親承認最終入力組とする。出力は7literal逆全bytes/非所有保全/8関数抽出/3fixture逆差分/固定期待manifest・静的gate証拠であり、作者runtime観測0。返却後に別reviewer `gpt-6-astra/high` が静的受入を行い、親freeze後にのみ別runner `gpt-6.1-sol/medium` へG3をdispatchする。新たな設計変更が必要ならG2で止め、同じPreを流用しない。

plannerの完了は本書保存・所有/diff/参照/hash確認と親handoffまでである。開始cleanは親受領、planner読取時のcurrent/usage変更は親の並行作業として保持した。KPI記録は親の既存usage文書へ接合する。子の開始/終了token、週枠、provider返却model/effort、親子包含は未取得であり推計しない。code/test編集、対象runtime実行、子起動、commit/公開は0である。
