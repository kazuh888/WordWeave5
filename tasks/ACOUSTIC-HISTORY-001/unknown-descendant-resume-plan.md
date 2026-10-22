# UNKNOWN-DESCENDANT-OPEN01：保存証拠監査と再検証計画

2026-10-22／ACOUSTIC-HISTORY-001／暫定版。開始HEAD `05e78f5`、planner着手時のtracked treeはcleanである。利用者の「進めてください」を親から受領した。本段階の実行権限は保存資料のread-only調査であり、source・fixture期待変更や試験再実行の許可を本計画から生成しない。親の受領・dispatch待ちである。

計画とtask-listの指定先は本書だけである。`tasks/plan.md`、`tasks/todo.md`、既存packetの未完了項目を保持する。plannerはAGENTS、development-team、KPI、`agent-skills:planning-and-task-breakdown`と同参照Definition of Doneを全文読了した。Skillの依存順・受入・checkpointを適用し、実装/runtimeは担当外とする。

## 要求・完了境界

| ID | 要求と受入証拠 |
| --- | --- |
| U-R1 | 旧F3 suite `b5bb9069-90e5-429b-8c5b-97c7f7f22f75`／run `77a4759e-0508-4cdd-966b-140c3ef89a12`だけを主対象とし、元入力・FF・terminal・target記録・stdout/stderrの所在と取得可能性を固定する。各結論にpath:lineまたはJSON propertyとhash対応を付ける |
| U-R2 | Supervisorのknown0／GetProcessById失敗／Get-Identity未到達と、RunnerのDirectLaunchHandle／exit93をactor・世代・観測時点ごとに分離する。他者の証拠、残留配列空、現在PIDの状態から未知をOwned/gone/正常終了へ昇格させない |
| U-R3 | 先行target異常・取得競合・記録不足の各仮説に支持証拠、反証条件、欠測を列記する。原因一意化できなければ未確定を維持し、次に必要な最小観測を設計入力として返す |
| U-R4 | 既存外部仕様・設計の再利用範囲と差分要否を監査後に確定し、再検証の対象・期待・上限・停止・所有・承認境界を独立Preで確認できる計画へ精緻化する。旧F3や製品gateの解除とは区別する |

今回の第一checkpointはU-A保存証拠監査の受領である。本packetの計画上の完了は、必要な仕様確認・設計・独立Preを受けた再検証計画の確定、または保存証拠では解けない阻害と必要判断の具体化である。原因修正・新実測成功・UNKNOWN Closedを完了条件へ先取りしない。

除外：成功済みP7-E／Context12／MissingOs等の再実行、新F3／ENV01／focused10／残9／80／variant／native／full／build、process起動試験・CIM等のOS process照会・停止、実data／API／admin／OS・ACL・PATH・設定変更、親model変更、基準・期限緩和、子の再委任、commit／push／公開。ファイル読取用の通常shellだけを使い、対象scriptのdot-source/import/実行をしない。旧試験の一回枠・過去承認を流用しない。

## 再利用する根拠

| 参照 | 本計画への意味 |
| --- | --- |
| `tasks/current.md:3`、`:17` | P7-E局所Closedと次のUNKNOWNを分離する。新runtime局所互換性は旧F3へのcredit0 |
| `early-trial-delivery-review.md:154`–`:164` | 旧F3 control2PASS／focused0PASS・1FAIL・9NotRun、known0、exit93、元完了marker欠落、原初因未確定が入力である |
| `early-trial-gate-plan.md:24`–`:29`、`:651`–`:653` | G1–G6、未知非停止、全argv／有限回収／旧F3未解除を維持する |
| `early-trial-gate-design.md:495`–`:504`、`:729`–`:738` | KH/MIは完全登録・連続保持handleの限定契約であり、未登録候補の取得失敗を救済する既存方式ではない |
| `early-trial-gate-plan.md:713`、`early-trial-gate-design.md:846` | 7.6.7局所成功をnative/Cargo全経路・旧失敗原因・製品成功に拡張しない |

略記R=`target/early-trial-build/et3/cargo-gate`。元入口は `R/verification/b5bb9069-90e5-429b-8c5b-97c7f7f22f75.F3-REPORT.md`（旧記録SHA先頭A739994A）、同suiteの `frozen-inputs.json`、`result.json`、`focused-ENV01.json`、`R/runs/77a4759e-0508-4cdd-966b-140c3ef89a12/` である。plannerは所在だけを確認した。内容・完全SHAの監査は次役が行い、旧prefixを現物一致済みとは扱わない。現在sourceを当時sourceへ無条件代用しない。

## 有限作業リスト・依存・所有

### U-A：保存証拠監査（S、読み取りのみ、次のdispatch）

- [ ] report／固定入力／結果から当該runの直接証拠を辿り、存在・欠落・破損・hash不一致を別々に記録する。探索は当該suite/runと、それらが明示参照するsource／manifestに限定する。archiveだけにある資料はread-onlyで参照し、原本展開・上書きしない。
- [ ] Supervisor・Runner・RunOwner・targetごとに、保存時刻、PID/開始時刻、snapshot採取時刻の有無、例外位置、exit、marker、cancelを少数行の表にする。時刻の前後から因果や同世代を自動推論しない。
- [ ] 凍結sourceを同定できた範囲で、GetProcessById→Get-Identity→登録、fakeのexit93／正常完了marker、ENV01判定のproducer/consumerをpath:line付きで返す。ET3_FAKE系の未採取を「exit指定なし」としない。コードの修正提案・原因判定は担当外である。

依存：親の本計画受領。担当は限定scout1役。書込所有なし、報告は親へ返し、親が `unknown-descendant-evidence-audit.md` に保存する。入力は上記直接証拠のみ、過去全suiteの網羅読取・全repo探索をしない。受入はU-R1～3を判断可能な事実表と欠測表であり、原因確定は不要。検証は参照の到達性・当該run一致・保存hashとの対応・禁止操作0の申告である。読み取りでも対象不明／hash不一致／欠測が次の判断を妨げる時点で停止し、推測補完せず親へ返す。

### Checkpoint A：証拠で分岐を決める

- [ ] 親はscoutの証拠を受領し、同じ調査を再実行せず結論と未確定を接合する。複雑な因果判断をscoutへ追加委任しない。
- [ ] 既存契約で次の観測・修正方式を定義できるか確認する。元fixture失敗と監視失敗を区別できなければ、原因未確定のまま最小観測設計へ渡す。

### U-B：仕様再利用判定・限定設計（S～M、文書最大2本、条件付き）

- [ ] 既存G5/ET契約の維持だけで足りるなら外部仕様は再利用し、新規作者起動を省略する。target正常性・ENV01 oracle等の受入変更が必要なら `ww-spec-author` が観測可能な差分を `unknown-descendant-spec.md` に作り、独立仕様reviewと利用者判断を経る。安全規範の緩和は提案しない。
- [ ] designerは採用仮説／棄却可能性／最小変更候補／合成対照／期待・停止・所有を `unknown-descendant-design.md` に定義する。保存証拠で既存設計をそのまま再利用可能なら、対応節と成立条件の短い確認で済ませる。
- [ ] plannerは受領仕様・設計後に本書を更新し、未確定の変更span・試験数・完全command・fixture/output root・入力hash／runtime・一回枠を具体化する。source/fixtureを変更する権限や実行許可が必要なら、その具体案を親へ戻す。

依存：Checkpoint A。受入はU-R2～4を維持し、どの仮説を何で区別するかと再利用できない証拠が明示されること。検証は要求→仕様→設計→観測の対応、相対link／`git diff --check`、独立Preへの入力固定である。外部仕様判断・権限拡張が必要な場合は文書案までとし、依存する変更／試験へ進まない。

### U-C：独立Pre・精緻化計画の受入（S、レビュー読取のみ）

- [ ] reviewerはunknown非停止・世代証明・別actor証拠の混用防止・旧FAIL保全・fixture正常性・失敗時保存・有限停止・現runtime境界を独立確認する。
- [ ] plannerは指摘を対応役へ返し、本書の所有・依存・必要検証を確定する。FAILを計画上だけPASSへ変更しない。
- [ ] 親は具体化した修正／検証案の承認状態を記録して次のdispatchを決める。Pre成功だけで試験実行を開始しない。

依存：U-Bの必要な仕様確認と設計。reviewerは書込なし、親が結果を保存する。受入は独立Pre PASS、または必要判断／阻害が明示された停止である。後者を修正完了と呼ばない。docs-only段階でapplication buildや既成功試験を走らせない。後続Windows製品deliveryの必須 `cargo test --all-targets --locked`／`cargo build --release --locked --bin wordweave5` は維持する。

## 全単位の役割とexactモデル割当

以下をU-A/U-B/U-Cすべての共通割当とする。非該当役は起動しない。将来の実装・テスト単位は仕様/設計後に分割して本書へ追加し、ここでの候補割当だけではdispatchしない。

| 役割 | model / effort | 難度・リスク／選定理由 | 今回の適用 |
| --- | --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 高／所有・未解除gate・観測不足を跨ぐ計画判断、固定モデル | 本書のみ。U-A後とU-B後に精緻化 |
| external-spec author | `gpt-6-astra` / `high` | 高／正常性oracleを変更する場合の外部期待と安全契約、固定モデル | U-A/C非該当、U-Bで契約差分がある場合だけ起動 |
| designer | `gpt-6-astra` / `high` | 高／未登録世代・競合・証拠欠測の状態設計 | U-A非該当、U-B担当、U-C指摘時だけ修正 |
| implementer | `gpt-6.1-sol` / `high` | 高／安全境界に接するため明示設計と独立reviewを必須とする | U-A/B/C非該当。本体変更0、後続で限定span確定後に再評価 |
| test author | `gpt-6-astra` / `high` | 高／fixture異常を監視成功で隠さない独立期待と反証対照 | U-A/B/C非該当。後続期待・fixture所有が確定するまで起動0 |
| test runner | `gpt-6.1-sol` / `medium` | 中／固定command・入力と停止規則を忠実に扱う担当 | U-A/B/C非該当。実試験0、旧成功の再実行0 |
| reviewer | `gpt-6-astra` / `high` | 高／actor横断の証拠と未知非停止を独立確認 | U-Aは親受領、U-B必要な仕様review、U-C独立Pre |
| 限定scout | `gpt-5.6-terra` / `medium` | 中／既知runの所在・時系列・source参照の有界読取に限定 | U-Aだけ。7役の設計・reviewの代替にしない |

利用可能一覧は親提示のAstra/high、Sol6.1/medium・high、Sol5.6/high、scout Terra/mediumに基づく。親は各dispatch前にexact ID／effort／役検出を照合し、指定値を起動引数へ明記する。指定と実provider model/effortは別記し、返却metadataがなければ「未取得」とする。利用不能な指定モデル・必要Skillはblockerであり、黙って代替しない。通常1補助ずつfresh context、再委任0。親モデルと品質gateは不変である。

## リスク・未決

| リスク／仮説 | 支持する既存記録 | 反証・追加識別に必要な証拠／扱い |
| --- | --- | --- |
| H1：snapshot後、取得前にtargetが終了した競合 | Runner終了がSupervisor FFより先、GetProcessByIdで失敗 | snapshot採取区間・同世代・当時API例外の詳細が必要。前後関係だけでは一意化不可 |
| H2：fixture内部errorが早期終了を生んだ | exit93、完了marker欠落 | 元stderr/I/O error、凍結sourceの93分岐、実継承exit指定が必要。93単独は内部errorの証明にならない |
| H3：ENV01 oracleがtarget正常性を判定していない | 既存review158のexpectedExit比較範囲 | 実凍結consumerを照合する。oracle不足はH1/H2の直接原因とは分け、期待変更には仕様判断が必要 |
| runtime取り違え | 当時7.6.6、現在7.6.7の局所互換性完了 | 旧証拠と現source/runtimeを別組で扱う。版更新だけから原因・治癒を導かない |
| 証拠不足を埋める無断追試 | 元I/O errorやsnapshot時刻が未取得 | 不明を残して最小観測案を作る。OS照会・retryで埋めない |

未決は元I/O証拠の存在、snapshot時刻・世代の証明可能性、fixture正常性oracleの仕様差分要否、最小観測／修正spanである。これらは監査・設計で具体化する。新試験が必要な場合は実行対象・完全command・一回枠・停止条件・非admin等を固定して親へ渡し、許可状態が明確になるまで実行しない。

## 次のhandoffと記録

親はU-Aを `ww-scout / gpt-5.6-terra / medium` へdispatchする。渡すのは本書U-A、review154–164、上記suite/run入口、安全境界、要求U-R1～3だけで足りる。期待出力は「資料一覧／actor別時系列／source producer-consumer位置／欠測／未確定」の短い報告である。audit役が原因を断定したり他actorからownershipを補完したりしない。親は同じ調査を重複せず、usageと現状索引の管理を並行する。

親提示の開始週枠はusedPercent21、windowDurationMins10080、resetsAt1793097272でアカウント共通である。親usage snapshotは2026-10-22T12:44:33.682Z、記録は親が所有する。子の独立token・実provider model/effort・親子包含は未取得であり合算しない。終了カウンターは親が取得可能範囲を記録する。本書作成は資料読取と文書検証のみで、実試験・OS process照会・停止・再委任0である。

## U-C reviewer再選定（2026-10-22、親からの明示差戻し）

親からU-A `unknown-descendant-evidence-audit.md` 受領済み、U-B `unknown-descendant-design.md`（72行、SHA256 `9C45A6DE4E6D46A6F354F7916898C34CBA604F9638877E4D5F1D463B01A0E00A`）受領済みとの報告を受けた。これは親からの受領情報であり、plannerによる設計再レビュー・現物hash再検証を意味しない。

U-C初回指定 `gpt-6-astra / high` は `Selected model is at capacity` で起動失敗し、レビュー作業0、判定未取得で停止した。親が提示した利用可能候補 `gpt-6.1-sol / high` と `gpt-5.6-sol / high` から、plannerは **U-C独立reviewerだけを `gpt-6.1-sol / high` に再選定する**。上表のreviewer共通指定より、本節のU-C限定指定を優先する。

難度・リスクは高のままである。選定理由は、対象が既存証拠監査と72行の限定設計へ有界化され、全体の再設計を伴わず、同世代証明・未知非停止・actor間証拠の分離・oracle・観測保存を固定観点で独立照合する作業であることにある。Sol6.1/highは親が呼出可能かつ対応effortとして確認した候補であり、現行のbounded設計reviewへ割り当てる。モデル名だけから能力優越・合格・検証削減を推定しない。Sol5.6/highを同時起動せず、新指定も利用不能なら再度blockerとしてplannerへ戻す。

次handoffは親からfresh `ww-reviewer` へexact `model=gpt-6.1-sol`、`reasoning_effort=high` を明示する。必要Skill `review-software-design` を全文読了し、既存U-C受入・禁止事項を維持してread-only独立Preを行う。レビュー対象は上記U-A/U-Bと必要な直接参照だけ、出力は根拠位置付き判定・指摘・残る未検証である。実装・試験・OS照会・再委任・公開0、親が結果を保存する。planner・外部仕様の固定Astra、親model、品質gate、権限は変更しない。実provider model/effortは起動指定と分離し、未取得なら推測しない。
