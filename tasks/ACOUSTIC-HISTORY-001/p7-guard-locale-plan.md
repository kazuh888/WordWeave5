# P7-D4 guard locale限定再設計計画

2026-10-18 / ACOUSTIC-HISTORY-001 / P7-GUARD-LOCALE。L1独立Pre PASS/阻害0を受領し精緻化済み、本版の親受領後L2へdispatch可。親の失敗保存checkpoint `7d6709f` を受領した。利用者の既存「検査の厳しさを維持して限定設計し、独立レビュー後に修正・検証」の承認内で、旧design §6の原因担当差戻し・新freezeを適用する。新3ケース実行は未解除である。

## 規範範囲・入力

本書が本単位の唯一の計画・task-listである。優先範囲はadapterの英語raw stack表示literal依存の是正と同一run UICulture観測だけである。旧[plan](p7-guard-binding-plan.md) `01AD21496978F7D4D03191FACEB0C6A5F8679D52C2C3FE2BBF787E60D78EE32C`、[design](p7-guard-binding-design.md) `184289A35BF03FD4101789E39120F4F9E159B4947BB30AB195A7314F7E879A4A`と上流main文書を凍結し、旧契約の他の条件は継続する。
確定[locale design](p7-guard-locale-design.md)の唯一のSHAは `23DD36FE10577352A0BE09A766794BD4508BFC061E794DC3C3A11A8A64F7B8FE` である。これと仮plan `5B5600354C9E3A8F160E5997F7770322ED9796223ECFC6470503ED7737F28BB6` への独立Pre PASSを親から受領した。design内の仮plan参照・提出時状態は来歴として保持し、追従編集しない。本版は確定設計の接合だけであり、L2実script/新runは未実証である。
`P=F:/Kazuhiro/GitHub/WordWeave5/target/early-trial-build/et3/cargo-gate/verification/argv-path01`。旧候補rootは`P/d03fbe3a-1318-4b39-8ba5-110b588e1640`、旧結果rootは`P/ed063a7c-e915-4126-84a7-d882e66fd8f1`である。

| 保存証拠 | 完全SHA256 |
| --- | --- |
| `P/85d2597a-74d3-4d49-ab07-8a8fbb7d9ea3/G3-FROZEN.md` | `9C3FD5B84C935DE4A770955C93BC302C0AA93656AA6B33E07132F9B8D9E171A5` |
| 同dir `G3-RUNNER-REPORT.md` | `9F0D81C22D7A74CCDAAE1F23FFED3AB2E33862CEEE8F0DF40C7AA707099E145A` |
| 旧結果root `result.json` | `F989EC04A95075396EA1E3DB942F92AA359E0567EF025263A6467945A41E366C` |
| 変更禁止の現FF | `AF179383F21753E27E285E29B86C352BEF7C2F6C4CF5576152DF0C4E79FDA073` |

旧G3はG01Passed/G02AdapterFailure/G03NotRun、実exit2、一回/retry0、内43/外44hash一致、saveErrors空、PASSmarkerなしである。G02の型/完全文言/仮想file/throw44/元guard230/sentinel false/output0は予定に一致し、先頭2frameの日本語表示だけが英語固定期待と不一致だった。同一run UICultureは欠測であり、他hostの値で補完しない。独立Post FAIL/停止適合を親から受領した。これはG2 cycle1のAdapterFailure分類優先不備とは別原因である。

## 要求・除外

- L-R1：`System.InvalidOperationException`、Ordinal完全文言`ORACLE: owned-tree body changed`、正確な仮想file、throwline44、元guard230、先頭frame名`Assert-Local`→`Assert-SourcePreserved`の2個と順序、sentinel/output0の厳密性を保持する。実際の誤りを期待へコピーせず、未知エラーをPASSへ広げるregex/fallback・無制限unwrapを禁止する。
- L-R2：固定7.6.7の表示と構造の対応・期待の由来・同一run UICulture観測は確定design §1–2を正とする。公式raw sourceは未取得である。承認SMAと同定済みja衛星のidentity/version/hash、ILと資源形式の独立確認を根拠に、固定fieldsから期待を生成する方式をPreで受入れた。支持cultureは新runのCurrentCulture/CurrentUICultureともja-JP限定、設定変更0。元consumer/8helper/fixture/FF7literal/3期待は不変である。
- L-R3：新候補の局所差分と新3ケース一回を独立受入した範囲だけを閉鎖する。旧G02は後付けPASSにしない。各5秒/全60秒、firstUnexpected、残NotRun、AdapterFailure優先exit2、保存/実exit/marker条件を保持する。

外部仕様P7-S01–05、FF/MI契約を再利用する。製品・利用者動作やエラー識別の意味を変えず表示adapterだけを是正するため外部仕様役は非起動、本体実装0である。新観測/追加ケース、root使い回し、再試行、E/FF18、旧F3/UNKNOWN/ENV01/focused10/80/full/build/製品提供、実data/API/admin/OS/PATH変更は未解除である。

## 単位・所有・受入・検証

依存は完了L1設計/独立Pre/本書精緻化→本版親受領→L2作者→独立静的review→親新freeze→L3別runner一回→独立Postである。通常補助1役ずつ、再帰0とする。

| 単位/規模 | exact所有・完了条件 | verification/checkpoint |
| --- | --- | --- |
| L1/S | 確定locale design §1–4と独立Preを受領済み、同designはimmutable。plannerは本書だけを精緻化 | SMA/ja衛星/IL・資源形式、ja-JP限定解決、L-R1全AND、旧42不変、新freeze、culture/保存/exit2優先を独立確認。本版親受領までscript編集0 |
| L2/S | test authorは新候補`N=P/4e5f72fc-b5bc-45e1-9052-dc4d47e3e03f`内の`Test-GuardBinding.ps1`/`manifest.json`と差分・静的/出典証拠のみ。旧root全体・現FF・元42入力・helper/fixtureは編集0 | design §3の許可5群と限定helpだけを逆差分で証明。変更driverにParser0/PSSA1.25.0通常rule診断0/engine0/marker/実exit0、独立static review。動的3観測0 |
| L3/S | 別runnerは新結果`Q=P/b5517847-0aff-42cf-85f2-80ac79f064a1`内の結果と親新freeze-dir内の外側log/報告のみ。design §4の完全argvを親freeze後に一回実行 | 同一run culture/資源と実runtime、前後入力、3期待、拒否接続、sentinel/output、期限、件数/実exit/保存を確認。unexpectedで停止、retry0。独立Post後のみ限定closure |

旧manifest42入力は来歴として保持し、不変snapshotを新manifestから固定参照する。旧driver/manifestは上書きせず、新候補自身は外側freezeで別同定する。helper/fixtureの重複copyや無変更の静的全検査を反復しない。新path/manifest/adapter差分の接合、変更driverの品質検査、実行前後の保全確認は必要であり省略しない。E baseline確認済み証拠とD1–D3/C16限定Reuseは維持するがEを解除しない。
確定design §3の旧42＋追加4（新driver/ja衛星/本plan/新design）=46、旧新manifestを含む内部post48を新freezeへ接合し、外側freeze自身は別照合する。N/QはPre時未作成、既存なら上書きせず停止する。資源/期待生成・前後cultureは§1–2、許可差分は§3、唯一のcommand/保存/closureは§4を参照し、本書に再複製しない。旧run UICulture欠測を補完しない。

## 全単位のモデル割当

| 責任 | exact model / effort | 起動範囲・難度/リスク・理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 本書とL1後精緻化、中/高。固定役、品質変更とadapter是正の境界を維持 |
| external-spec author | `gpt-6-astra` / `high` | 既存仕様再利用で非起動、中/高。契約変更が必要なら固定役へ差戻し |
| designer | `gpt-6-astra` / `high` | L1、高/高。公式表示資源と厳密なエラー識別の対応を判断 |
| implementer | `gpt-6.1-sol` / `medium` | 全単位で非起動、低/中。本体変更なし、test所有を代行しない |
| test author | `gpt-6-astra` / `high` | L2、高/高。独立出典に基づく期待と局所adapter修正を接合 |
| test runner | `gpt-6.1-sol` / `medium` | L3、中/高。凍結一回手順と同一run証拠を採取し期待を編集しない |
| reviewer | `gpt-6-astra` / `high` | L1 Pre/L2 static/L3 Post、高/高。未知例外の誤PASSと旧証拠書換えを拒否 |

既存役・割当を再利用し、designer/author/runner/reviewerの責任は別担当で維持する。親model/effort不変、指定をdispatchで明示し返却metadata未取得を推定しない。必要skillを各役が全文確認し、指定model/effort/skill利用不能はblockerでありsilent fallbackしない。

## task-list・停止・次handoff

- [x] 仮計画：旧結果と原因差戻し境界を記録した。独立FAILを覆していない。
- [x] L1：限定設計・独立Pre PASS/阻害0の親受領・本書精緻化。本版の親受領をL2 dispatch前条件とする。
- [ ] L2：新候補driver/manifest・局所静的検証・独立review・親freeze。
- [ ] L3：新3ケース一回・独立Post・限定closure。D4/Eへ残る境界を親へ返す。

公式固定版との対応不足、厳密性を維持できない方式、追加観測/設定/契約変更、所有外差分、品質未達、入力drift/欠測/unexpectedで停止する。原因担当へ戻して再設計し、同候補自動retryはしない。rollbackは新候補採用を取り消すだけで旧証拠を保全し、現FFや旧scriptへ逆編集しない。
次dispatchは親が本版を受領後、L2独立test author `gpt-6-astra/high` へ行う。exact所有は上記N内driver/manifest・差分/静的/出典証拠のみ、設計§3の5群と限定help以外の改訂0、動的観測0。返却時は旧driver逆全bytes一致・旧42順序/path/SHA/origin不変・追加4とmanifest接合・L-R1全AND/ja-JP前後資源/culture/exit2優先・静的品質gateを示す。別reviewer `gpt-6-astra/high` のstatic受入と親の新freeze後にのみ別runner `gpt-6.1-sol/medium` へL3を渡す。新manifest/driverの最終SHAと完全argvはL2後に確定し、未展開値で実行しない。
plannerの完了境界は本書保存・参照/所有/hash確認まで。旧plan/design・script・親docs・未完了計画を保持した。runtime/子起動/commit/公開0。KPIは親が既存usageへ接合し、子token/週枠/provider返却metadata/親子包含の未取得を推計しない。
