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

## U-D～G：承認済み診断追加と隔離内6対照（2026-10-22）

開始HEAD `ba7f0e4`、着手時clean。利用者が「検証用プログラムへの診断記録の追加と、隔離環境内で合成データによる6種類の局所試験」を明示承認した。検査の厳しさ・unknown非停止・元Err/status/abort/期限・旧FAILを保持する。新OS ENV01／native／F3／full／製品build、admin／OS設定／実data／API、既成功試験の反復は解除0である。本節は従前の「文書のみ」をこの承認範囲だけ更新する。U-A原因調査・U-C済みreviewの再実行はしない。

親は開始baseline6件をread-onlyで照合済みと報告した：Invoke `898C60B1133DB6956090035AF677B4CF7FDE0FE6FDFD5D47E5CD776B00500B71`、Test `F0C57FE388F96D78F3BA0B14A420D9E7E6F5C7977944AD72B19E925E894DC919`、fake `3482075BF7479E61942978830AE91FDDF5301133E1A2F43EDB9B65DDAA0FA016`、固定pwsh.exe `FA2C8…`／pwsh.dll `A583…`／SMA.dll `7070…` は一致である。完全環境hashは親の照合記録をmanifestへ転記し、省略prefixで実行照合しない。plannerは重複照合していない。これはfile一致であり今回processの実ロードruntime観測ではない。固定PS7.6.7と実ロードruntimeが不一致なら停止し、更新承認へ読み替えない。

### 実装前3gateの具体化

1. **公開方式**：既存`write_new`相当の最終path直接`CreateNew`を第一選択として固定する。tmp→renameは採用しない。固定`target-<pid>.failure.json`へ一回、上書き／削除／別名fallback／retry0。途中失敗で部分fileが残る場合は保全し、完全JSON parse＋hash成功だけをPublishedとする。元Err返却とmain exit93は変更0である。writerの内部戻り値・呼出点・失敗の非伝播とその観測方法はU-Dで設計担当が確定する。
2. **FF互換性**：acquisitionなしでは旧12key（既存任意producerFailure条件も同じ）の厳密集合を維持する。acquisitionありの時だけ外側にその1keyを許し、内側は設計の9fieldのみ、型/null/UTC7/ASCII制約を独立期待として固定する。任意の余分なkeyを許す比較へ変えない。旧`Test-FirstFailure.ps1:173–181`本文・期待を変更せず、旧形と追加形の局所oracleをO5/O6へ分けて入れる。既存公開PS関数signatureを増やさずlocalな観測値で接続する案を優先し、必要なsignature差分が残ればdesignerが明示してPreへ返す。
3. **seam・hash・command**：既存の純合成抽出runner方式を維持する。一意AST/statement抽出、許可置換の個数・位置、逆bytes一致、未到達main／OS／登録／killを静的境界と必要counterで分ける。Rust側の局所抽出/seamとPS側の値注入点・返却形はU-Dが確定し、作者へ設計判断を移さない。baseline完全hashは編集前に固定し、作者の完成oracle→実装候補→最終new hash→親freezeの順とする。新hashを実装前の条件にはしない。完全commandは作成済みのscript/局所Rust compile対象・固定tool path・Phase・新UUID絶対rootを含めて静的Preへ提出し、親freeze後だけ実行する。実行時文字列置換0、freeze自身のhashは外側から渡し循環埋込み0である。

### 有限単位・所有・受入

| 単位／依存 | 所有・実施範囲 | 受入と必要検証 |
| --- | --- | --- |
| U-D／本追補受領 | designerは`unknown-descendant-design.md`末尾だけ。上記writer内部interface、FF optional接続、Rust/PS局所seamの3点を短く確定する | O1～O6を設計9C45の期待のまま実装可能にし、公開PS signature変更・新framework不要を示す。reviewerは新差分だけPre確認。既存U-Cの全面再review0 |
| U-E／U-D差分Pre | 独立test authorはR/verification/argv-path01の新UUID内のoracle・合成入力・期待manifestのみ。本体`fake.rs`／`Invoke-CargoGate.ps1`／既存Test/FF testを編集しない | O1～O6の期待、複数入力数、AdapterFailure区別、case5秒／集合30秒、firstUnexpected・残NotRun・retry0を固定する。完全静的検査／局所compile／実行commandを用意し、新対照以外のdispatch0を静的確認。未実装機能をadapter failureでRED扱いしない |
| U-F／U-E期待固定・静的Pre・親受領 | implementerはR/`fake.rs`の7key・stage/sidecar観測spanとR/`Invoke-CargoGate.ps1`のsnapshot/取得段階・任意FF射影spanだけ。両fileのbeforeを先に保全する | 非所有逆diff一致、旧argv/stream/exit/marker順・所有判定・登録/回収・ENV01期待不変。変更PS Parser0/PSSA1.25通常診断0・engine0、Rust局所compile成功。必要な新candidate command/hashは完成後に確定し、既存mainや実OSを起動しない |
| U-G／新候補静的Pre・親freeze | 別runnerは新UUIDログ/結果のみ、reviewerはread-only。新O1～O6を各入力一回だけ順次実施する | 前後入力一致、実runtime、完全argv、実exit、保存・marker、6caseと入力数別計上、元failure保持、strict FF互換、unknown保持を独立Post確認する。保存/adapter失敗・超過・driftで停止、後続NotRun。局所Post成功でもUNKNOWN閉鎖／旧F3治癒／新OS解除0 |

U-D完了後、plannerは確定seam/interfaceとcommandの所在だけ本書へ接合する。通常補助1役ずつ、test authorとimplementerは別agentかつ別file所有である。本節では以前の設計表の「test authorがfake本体を編集」を更新し、検証用fakeも診断追加の実装としてimplementerへ割り当てる。テスト作者が本体を変更して自分の期待へ合わせる経路を作らない。

### ignored成果物の保存と再現性

既存R/verification/argv-path01の新UUID packetへ`before/`原本2fileと完全SHA/length、oracle、合成入力、候補・ログを保存し、旧packetと原本を上書きしない。trackedの`tasks/ACOUSTIC-HISTORY-001/unknown-descendant-artifacts.md`を親が所有し、baselineの取得元・完全hash、before/output絶対path、固定toolchain、許可span、完全command／freeze／結果索引を記録する。authorは`unknown-descendant-tests.patch`、implementerは`unknown-descendant-harness.patch`を同tasks directoryの自分の担当成果物として保存し、新規oracleの全内容・本体before→after差分を再現可能にする。親は最終hash対応とpatchの適用先を索引へ接合する。保存sourceは合成harnessとoracleだけで、実data・秘密・binary・runtimeログをtracked化しない。保全beforeへのdry-run適用とafter hash一致は独立reviewの対象とし、稼働中Rへ再適用しない。plannerは本書以外を編集しない。新config/module/framework/Pester導入0である。

### U-D～G exact割当・次dispatch

| 役 | model / effort | 難度・リスクと理由 |
| --- | --- | --- |
| planner／必要時external-spec | `gpt-6-astra` / `high` | 高、固定。今回外部期待は再利用しspec作者不起動 |
| designer | `gpt-6.1-sol` / `high` | 高、既承認案の3interface差分に限定。前回Astra capacityを記録した上で今回の利用可能候補から明示再選定し、全面再設計をしない |
| test author | `gpt-6.1-sol` / `high` | 高、strict旧keysと元Err保持の独立期待・局所seamを作るため。前表の候補Astra/highから本承認単位に限り明示変更 |
| implementer | `gpt-6.1-sol` / `high` | 高、2fileの診断差分でも元例外・所有判定への波及を防ぐ必要がある |
| test runner | `gpt-6.1-sol` / `medium` | 中、凍結済みcommandと一回・停止規則の独立実行。独自設計・期待変更なし |
| reviewer | `gpt-6.1-sol` / `high` | 高、作者・実装者とは別fresh agentでstrict互換・非干渉・保存証拠を独立評価。U-C再選定を継続 |

親は上記exact model/effortで起動する。capacityやSkill欠落はblockerとして戻し、別モデルへ自動fallbackしない。親model・品質gateは変更0。次は **U-D designer1役** へ設計9C45、本追補、review18～20行の3gateだけを渡し、必要Skill `agent-skills:api-and-interface-design` 読了後の限定文書差分を依頼する。設計の追加判断をtest authorへ丸投げしない。以後作者はtest-driven-development、実装/runnerはwordweave-changeの担当工程、reviewerは設計差分にreview-software-design、コード差分にcode-review-and-qualityを適用し、PowerShell既存code変更の必要Skillも担当が確認する。

開始共有週枠24／10080min／reset1793097272、親usage snapshot2026-10-22T13:14:55.662Zを親が保存する。account共通・子包含不明でありtask消費へ換算しない。plannerの今回作業は限定追補と文書検証のみ、source変更・実試験・OS照会・再委任0である。

## U-H～K暫定計画：実OS ENV01の承認前準備（2026-10-24）

開始HEAD `9e8fffc`、編集直前tracked tree clean。今回の完了境界は、新診断による実OS ENV01一件一回について、実行前blocker・静的準備・必要承認を具体化し親へ返すことである。利用者の「進めてください」は、前工程で明示された実OS一回枠の別承認を解除しない。本節は暫定であり、U-Hの設計と必要な仕様判断後にplannerが同節を精緻化する。既存未完了項目と旧FAILは保持する。

入力事実は[current](../current.md)、[設計](unknown-descendant-design.md)の「実行許可へ渡す条件と未解決」、[成果物索引](unknown-descendant-artifacts.md)の新候補6対照節である。合成6case/51inputsとBOM是正の独立Post PASSを再利用する。候補fake `07A47648E47FB57F5D0969650BBCEAF138671A4FC151EF6AA1E33020DCF2DCC4`、Invoke `3B784E9B7CAE98261D0A496C0E229C4E2CB39120962656BF2186049A08CCC4D3`は索引の値であり、今回plannerは重複hash検証をしていない。旧UNKNOWN原因は未確定、旧F3/full/製品build・提供は未解除である。

### 要求と静的に確認したblocker

| ID | 根拠／不足 | 解消に必要な具体物・受入条件 |
| --- | --- | --- |
| E-R1 | `R/verification/argv-path01/Invoke-FirstFailureEnv01.ps1:119–131`はInvoke898C…とfake348207…を固定する。現診断候補はそのままでは入口で拒否される | 旧probeを保全し、当該2literalだけのbefore/after差分案、完全SHA、逆差分全bytes一致を用意する。Test F0C57…、native source381F5…/exeFF7C9…、既存main plan/designのpinは独立に一致確認し、現在値への一括置換0。これ以外の差分が必要なら設計へ差戻す |
| E-R2 | 同probe220–230は`rustc --version`と`rustc --edition=2021 fake.rs -o <OutputRoot>/fake.exe`を実行する。51入力で作った抽出unit binaryは実fakeではない | 「cargo-gate-entry.exe再build禁止」と「診断付きfake.exeの新規compile一回」を区別した承認案にする。元dispatchを維持するなら後者を許可範囲に明記する。事前に存在しないfake.exe SHAはPendingとし、実compile後に同probeが採取するhashをfixtureコピー・frozen-inputs・toolchainへ結ぶ。抽出unit成功をnative fixture成功へ流用0 |
| E-R3 | 同probe194–207はrustc/pwsh/V2のpathとhashを実行時採取し、probe自身も自己hashするだけである。`ValidateOnly`以前にruntime/token確認・Get-Processがある | 親承認manifestでprobe、両source、Test、native source/exe、主plan/design、固定pwsh一式、rustcの実体と必要toolchain、V2、Pester5.7.1を外側から束縛する。実行時自己採取を事前承認の代用にしない。必要依存の正確な範囲はU-Hで限定確定する。ValidateOnlyを静的確認として実行しない |
| E-R4 | 同probe240–286はrun側FFを索引するが、fixture側`target-<pid>.failure.json`を索引しない。exit0は元ENV01結果とsourceMatchedで決まり診断充足を含まない | 元oracleを変えず、事後のread-only診断監査の入力と判定表を別文書で固定する。同run/actor/世代、7環境key、target marker/sidecar/stream/exit、FF acquisitionを結ぶ。正常時のsidecar非生成と、Err時の診断欠測を区別する。FF外側12keyは新acquisitionの内側追加とは別であり、12を機械的に13へ変えない |
| E-R5 | Test653–797のfixtureは新suite/ENV01と別の`R/runs/<動的runId>`へ書く。Test1743の90秒watchdogはfake compileを含まず、同期APIのhard timeout保証でもない | 絶対suite root、fixture root、動的runIdをdescriptorで束縛する規則、全argv、外側実行上限と停止時の保存手順をU-Hで確定する。既存run65秒（実行55＋回収reserve10）、build25秒、watchdog90秒、終了後観測11秒を延長しない。未知PIDの強制停止、再試行、fixture削除を外側timeout対策にしない |

Rは既定の`target/early-trial-build/et3/cargo-gate`である。上表はファイル本文を限定読取して得た事実であり、runtime成立・現在toolchainの一致・OS状態は未確認である。特にE-R2とE-R5が未解決の間は「一回枠のcommandは完成」と報告しない。

### 作業単位・依存・所有・検証

順序はU-H設計→U-I独立Pre→親による具体案の利用者承認確認→U-J束縛準備と独立Pre/親freeze→U-K別runner/独立Postである。通常補助1役ずつであり、親は承認の接合とusage記録を行う。

| 単位／規模 | 所有と依存 | 受入基準（最大3項目）／必要検証 |
| --- | --- | --- |
| U-H：静的実行案／S | designerは`unknown-descendant-design.md`末尾のみ。本追補受領後。既存外部仕様G4/G5/FF・ENV01 oracleを再利用し、仕様差分がなければspec作者不起動 | (1) E-R1～5の具体的差分案・dependency・採取表を確定。(2) 既存probeの元case dispatch/oracle/回収を保存し、完全command候補と実行前/生成後hashの二段階を明示。(3) compileを含む有限枠・通常sandbox内可否・承認境界を確定。検証は参照到達性、元dispatchとの静的対応、文書diff。対象script起動0 |
| U-I：承認案の独立Pre／S | reviewer read-only、親が既存review文書へ報告保存。U-H→planner精緻化後 | (1) E-R1～5、未知非停止、actor混用防止、旧証拠保全を独立照合。(2) 新native fake compileとENV01一回を明示し、OS照会/既知handle回収/書込root/失敗停止が承認案に収まる。(3) 残る未確定command/SHA/期限を列記し、未解決ならrunner dispatch不可。設計・仕様review Skillの必要工程を適用し、実行0 |
| U-J：承認済み束縛／S | U-Iと具体承認後。test authorは新UUID packetの固定receipt/manifest/診断判定表、implementerは既存probeの承認済み2literalだけ。作者・実装者を分離。fake/Invoke/Test/既成功51期待は編集0 | (1) before保全・許可2literal外0・逆bytes一致と全外部pinが揃う。(2) 完全argv/UUID/起動環境/保存先を固定、動的fake hash取得点を記録。(3) 独立差分Preと親freezeがPASS。変更PSにParser/PSSA1.25通常診断0・engine0、適用差分の静的確認を行う。新wrapperが必要なら本単位で発明せず設計へ戻す。runtime/ValidateOnly/compile0 |
| U-K：限定実観測／S・承認待ち | U-J freezeと実行承認後。別runnerは新packet/runの生成物・ログだけ、reviewerはread-only。親が成果物索引・review/currentへ接合 | (1) 診断fake compile一回、既存ENV01 dispatch一回以下、全前後pin・実runtime・実exit・期限・回収証拠を保存。(2) firstUnexpected/drift/compile失敗/超過/回収不明で停止、retry/次case0、診断欠測は欠測として保存。(3) ENV01判定と診断の得られた範囲を独立Postで分離。新PASSから旧UNKNOWN Closed/旧F3治癒/製品受入を推定0 |

チェックポイントとtask-listは本節に集約する。

- [ ] U-H：静的案の受領。未存在command・未生成SHAを確定値として記さない。
- [ ] U-I：独立Pre、planner精緻化、親へ具体的承認案を引渡す。本準備工程の停止点である。
- [ ] U-J：限定変更と実行範囲の承認を確認してから束縛差分を作り、独立Pre／親freezeを得る。
- [ ] U-K：別承認の一回枠だけ実施し、独立Post後に得た観測・未確定・未解除gateを報告する。

本書のplanner所有は末尾追補のみである。設計・review・packetへの将来所有は親のdispatch指定で成立する。他者の編集を戻さない。未解決の外部期待判断（target正常性oracle強化等）が必要ならU-Hを停止し、external-spec author→仕様review→利用者判断→planner精緻化とする。元のENV01期待を診断観測に合わせて変更しない。

### 各単位のexact model/effort

U-H/I/J/Kそれぞれに下表を適用する。非該当役は責任を識別するための割当であり、起動しない。親提示の利用可能集合は`gpt-6-astra/high`、`gpt-6.1-sol/high・medium`、`gpt-5.6-terra/medium`である。

| 役割 | exact model / effort | 適用と難度・リスク／選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | U-H後の精緻化と指摘差戻し。高／既承認と新権限の境界判断、固定指定 |
| external-spec author | `gpt-6-astra` / `high` | U-Hで外部契約変更が必要な場合のみ。高／oracleと原因観測の区別、固定指定。現案は既存仕様再利用 |
| designer | `gpt-6.1-sol` / `high` | U-H担当、I/J/Kは設計差戻し時のみ。高／既存入口に限った依存・時間・生成hash・診断監査の整合 |
| implementer | `gpt-6.1-sol` / `high` | U-Jの承認済みbinding差分のみ、H/I/K非該当。中～高／2literalでも誤pinが安全境界を崩すため厳密な逆差分確認を行う |
| test author | `gpt-6.1-sol` / `high` | U-Jの独立receipt/期待表のみ、H/I/K非該当。高／実行結果と診断不明の独立判定、元oracle不変、生成前hash循環の回避 |
| test runner | `gpt-6.1-sol` / `medium` | U-Kのみ、H/I/J非該当。中／凍結commandと有限停止の実施に限定し、期待/設計の変更を行わない |
| reviewer | `gpt-6.1-sol` / `high` | U-I、U-J差分Pre、U-K Postを各作者/実装者/runnerとは別fresh agentで実施。高／実OS所有・異actor証拠・欠測・権限を横断する独立確認 |

起動担当はexact model/effort/役検出と対応Skillを確認する。設計はapi-and-interface-design、作者はtest-driven-developmentの担当工程、実装/runnerはwordweave-changeの担当工程、reviewはreview-software-designまたはcode-review-and-qualityを使う。PowerShell既存codeへの変更/レビューは対応powershell skillも解決して全文を読む。モデル/Skill利用不能はblockerであり、自動fallback0。親modelと品質gate不変、起動指定と実provider metadataを区別し、未取得値を推定しない。

### 承認境界・残る判断・次handoff

追加の実行承認なしで進められるのは、既存ファイルの限定read/hash、文書での差分案・command案・収集/停止手順の具体化、独立read-only Pre、planner精緻化までである。本時点でsource/期待/既存probeを編集する承認は新たに生成しない。実行可能性確認を理由にValidateOnly、rustc version/compile、対象scriptのdot-source/import、CIM/Get-Process、process試験を開始しない。

親が利用者へ示す承認対象は、U-H/Iで具体化した **既存probeの限定binding変更、診断fake.exe compile一回を含む実OS ENV01一件一回、通常ユーザーAdmin=False・Hidden、固定runtime/toolchain、指定新suiteと動的runRootへの証拠保存、元dispatchに内在するOS照会と既知handleだけの回収、既存期限内停止・retry0** である。原probe内のcompileを維持する案を第一候補とし、事前別buildを暗黙に追加しない。隔離外実行が必要ならその事実と境界を承認案に加え、承認なしの切替0とする。cargo-gate-entry再build、製品build、full/focused10/80/P7/51入力反復、未知停止、admin、実data/API、OS/ACL/PATH/設定変更、公開を含めない。

主なリスクは、古いpinの機械更新、新規fake compileの隠れた追加権限、無期限compile/同期I/O、exit0と診断充足の混同、観測追加による時機変化である。静的PreではOS成功を証明できず、承認後の一回実観測でも旧障害の再現や原因一意化を保証しない。sidecar欠測・非Published・世代不一致を不明として保持することが受入の一部である。

次の親handoffは **U-H designer / `gpt-6.1-sol` / `high`** である。本節、設計の実OS候補節、現probe、上表のTest依存位置だけをfresh contextへ渡す。期待出力はE-R1～5を満たす限定設計差分と未解決事項であり、runtime/compile/OS照会/再委任/公開0である。plannerは設計受領後にのみ未確定事項を確定へ進める。今回は文書の限定追補とdiff検証だけを行い、子usage開始/終了counter・実provider model/effort・親子包含は未取得である。親がKPIに従い取得可能な記録を接合する。

### U-H受領後の状態確定（2026-10-24）

親から設計SHA先頭`166F267C…`と独立Preを受領した。完全SHAの現物再照合は今回行っていない。独立Preの結論は「承認前停止条件は妥当」であり、U-H完了・実行Pre PASSではない。確認範囲では元oracle/Err・unknown非停止・retry0・期限・診断分離に追加矛盾なしである。
U-Hは阻害条件の整理までで未完了、U-Iの実行案受入・U-J/Kの解除は0である。未決は次の3点である。
1. compile込み外側停止：同期version/compileは既存90秒監視外であり、停止可能handle、保存reserve、host上限を確定する。
2. toolchain/linker/TEMP/runtime/Pesterの依存と書込・OS操作範囲、sandbox内で成立するかを確定する。
3. 生成後freezeを同一入口の事前承認方式で扱うか、中間停止方式とするかを確定する。
次は不足安全手順の限定設計である。次designer `gpt-6.1-sol/high`を含む既存役割当を維持し、source編集前に利用者承認を要する。親が限定修正・合成検証の具体的承認を求め、承認と設計/独立Preの成立前に編集・検証を開始しない。実OS一回枠は引き続き別承認である。本追補は状態記録のみで、設計・実装・実行・再調査0である。

### U-L～P：承認済み安全手順の限定修正・純合成検証（2026-10-24）

開始HEAD `b85c6eb`／tracked tree clean。親提示の「不足3点を補う検証用手順の限定設計・修正・合成検証、実環境試験は含めない」に対する利用者の続行指示を受領した。本節は直前のsource編集承認待ちをこの範囲だけ解除し、U-Hの未完とU-K実OS一回枠の別承認は維持する。就寝指示は同一入口での生成hash自動承認を意味しない。
要求はE-R1～5と既存G4/G5/FFである。今回経路は生成hash取得・保存後に必ずAwaitingApproval／元exit1／dispatch0で停止する。公開param・再開/token関数は追加せず、将来再開は未実装として別の設計・明示承認を要する。元ENV01 oracle/Err/unknown非停止/retry0/既存期限を変更せず、旧同一入口案を黙って採用しない。
除外：製品/entry/診断fakeのcompile、ENV01実OS、ValidateOnly、対象scriptのtop-level起動、実OS所有判定、対象子processの起動/照会/停止、管理者/隔離外/OS設定/実data/API、既成功51入力/P7の反復、公開。純合成unit用の通常sandbox内PowerShellホスト・静的検査は許可範囲である。実子processが不可欠なら、必要性・対象・一回枠を親へ返し承認前停止する。

| 単位／依存・規模 | 所有・最小change set | 受入と必要検証 |
| --- | --- | --- |
| U-L／本追補、S | designerは既存design末尾のみ。元probeの有限version/compile待機、限定dependency/書込境界、生成後停止の3点を設計する | 秒数/monotonic時計/保存reserve/直接handleと未知の扱い、生成後の承認待ち停止・生成drift拒否、OS/FS/clock/processの合成seamを確定する。原則変更本体は既存probe1本、不可欠なprivate helper追加は1本までとし、正確なpath/spanを設計で確定して独立差分Pre。外部期待は再利用、変更が必要ならspec作者へ戻す |
| U-M／U-L差分Pre・planner精緻化、S | 独立test authorは新UUID packet内の期待表・抽出adapter・合成test・receipt/manifestのみ。本体を編集しない | 下記6群の入力数・期待・停止・完全commandを先に固定する。一意抽出/許可置換/逆bytes一致と対象top-level・実OS到達0を証明し、作者独立Pre後に実装へ渡す。未存在候補hashはPending、抽出失敗をbehavior REDへ変換0 |
| U-N／U-M期待固定、S | implementerはU-Lで固定したprobe/helperの許可spanとその再現差分だけ。旧版と全bytes/SHAを新packetへ保全する | 3点を実装し、許可外逆bytes一致、元dispatch/oracle/回収と既存native/Test/fake/Invoke不変を確認する。変更PSはParser0/PSSA1.25通常診断0・engine0、純合成対象の静的到達検査を行う。期待変更0。候補hashを作者が既固定期待へ束縛する |
| U-O／最終差分Pre・親freeze、S | 作者/実装者とは別runnerが新packetのログ・結果のみ所有 | 固定された新合成入力を各一回、通常sandbox内で実行する。前後pin・実PS・全argv・実exit・保存を記録し、firstUnexpected/adapter failure/drift/超過で停止、残NotRun/retry0。既成功51入力を含めず、本体top-level/compile/実子process0 |
| U-P／U-O、S | reviewer read-only、親が既存review/artifacts/currentへ接合。指摘は所有役へ返す | 設計→独立期待→候補→実結果の対応と原本保全、停止/未知保持/生成後承認待ちを独立Postで確認。必要な変更に影響する検証だけ再実行する。合成PASSはOS停止可能性・toolchain実依存・sandbox許可・実ENV01を証明しない |

新合成検証の6群は、(1) version/compile成功・非zero・開始失敗、(2)期限境界/遅延返却/保存reserve枯渇、(3)直接handleと未知子の分離・停止不能/回収不明・起動後stream設定失敗/StartTime失敗、(4)dependency/path/hash/TEMP等の欠測・drift・境界拒否、(5)今回経路の生成後停止・生成drift拒否によるdispatch0、(6)保存失敗でも元error保持・retry0である。入力数は順に5/6/7/6/4/5、合計33とし、未実装の再開をテストしない。memory・疑似clock・疑似process・疑似FSを用い、実装から期待を生成しない。新case上限5秒/集合30秒を設計の出発点とし、実行commandはU-Mで固定する。元OS APIのhard timeout証明とは区別する。
完了条件は、上記新差分の静的検査と独立合成試験/独立PostのPASS、生成後停止を含む次の実OS承認前条件の明文化である。hostの強制上限/実toolchain依存/sandbox可否を合成結果で確認済みにせず、実測が必要な残項目をblockerとして残す。実環境の不足まで解消済みと報告しない。
task-list/checkpoint： [x] U-L設計差分Pre→planner精緻化、[ ] U-M期待固定Pre、[ ] U-N最終候補Pre/親freeze、[ ] U-O一回実施、[ ] U-P独立Postと残る実OS承認条件。U-L完了前にU-Mへ内部設計判断を渡さず、未完了の旧作業リストも保持する。

| 各U-L/M/N/O/P共通の役 | exact model / effort | 適用／難度・リスクと選定理由 |
| --- | --- | --- |
| planner／external-spec author | `gpt-6-astra` / `high` | 高／固定。plannerはU-L後精緻化、specは外部契約差分時のみ。現行期待再利用ならspec不起動 |
| designer | `gpt-6.1-sol` / `high` | U-L、他単位は差戻し時のみ。高／時間・handle・生成承認の遷移設計 |
| test author | `gpt-6.1-sol` / `high` | U-MとU-N後binding、他単位は非該当。高／合成seamの非循環期待と実OS到達防止 |
| implementer | `gpt-6.1-sol` / `high` | U-Nのみ。高／限定spanでも停止責任・失敗保存への影響が大きい |
| test runner | `gpt-6.1-sol` / `medium` | U-Oのみ。中／固定command・入力・停止規則の独立実施 |
| reviewer | `gpt-6.1-sol` / `high` | U-L/M/Nの必要差分PreとU-P。高／作者・実装者・runnerとは別fresh agentで安全境界と証拠を照合 |

親は通常1補助ずつ上記exact指定でdispatchする。Skillは既存役別指定を継続し、PowerShell修正/レビューの対応Skillも全文読了する。未対応model/effort/Skillはblocker、親model/品質gate変更・silent fallback・再委任0。今回利用可能集合はAstra/high、Sol6.1/high・medium、Terra/mediumであり、限定lookupが必要な場合だけTerra/mediumを使い設計判断を委ねない。
次handoffはU-L designer `gpt-6.1-sol/high`である。前設計の不足3点と本節だけを渡し、新規frameworkや全履歴調査へ広げない。親提示の開始週枠1%/10080分/reset1793372790を親がusageへ保存する。子usage/実provider値/親子包含は未取得。planner今回の実行範囲は文書追補とdiff検査だけである。

### U-L是正設計の接合（2026-10-24、最終Pre PASS受領）

親から設計SHA256 `FB0F6359848A881726B693BF79AC072768EF78F6D26C044DC892CD09F34B5E15`と旧ECD3版before保全を受領した。今回plannerは設計の再調査・現物hash再照合をしていない。
親から別reviewerの最終是正Pre PASSを受領した。初回唯一P1のStart部分成功後のhandle喪失契約は解消、差分限定・新旧SHA一致・追加阻害0であり、U-L設計Pre完了としてU-Mへ進める。設計PreだけでU-N実装/U-O試験の各依存gateや実OS承認を解除しない。
設計の40秒/35秒/3＋2秒/version2秒/compile25秒は準備専用budgetであり、元ENV01の期限緩和ではない。合成実行のcase/集合上限とも別である。G3の2入力追加とG5の今回停止経路限定を上記33入力へ接合した。
次役はU-M test author `gpt-6.1-sol/high`である。承認scope・通常補助1役・全役割当・実OS別承認を維持し、本接合は文書差分のみ、実装・対象実行・再委任0である。

### U-M旧REDのAdapterFailureと停止gate（2026-10-24）

承認更新：利用者が未使用TestRegistry/TestDrive自動準備の無効化と、独立レビュー後の修正版旧版確認1回を明示承認した。旧失敗/一回枠は保全し、新packet/新output UUIDへ設定2行と必要なpath/hash metadataだけを接合する。33期待/元assertion/期限/判定/通常sandbox/非adminを維持し、実OS/本体compile/製品/OS設定/registry直接操作は解除しない。担当は既選定author/reviewer Sol6.1/high、runner Sol6.1/mediumを継続し、別モデル再選定や親変更は行わない。以下の承認待ちは更新前の経緯である。
Q=`R/verification/argv-path01/6201d237-1221-492b-8858-5c8f4c5f7d23`の`OLD-VERSION-RUNNER-REPORT.md`と保存stdoutを読取確認した。一回実測はexit2/AdapterFailure/accepted=false、TestRegistryの`HKCU\Software\Pester`初期化権限拒否でcontainer failed1、特定assertion RED未成立である。18pins前後不変、保存正常、2.6680518秒、retry0を保持する。
追加の限定harness設定承認が必要と判定する。`Invoke-OldVersionRed.ps1:25–33`のPester設定にTestRegistry/TestDrive無効化はなく、その追加はAGENTSの「Seek approval before changing harness/settings」に該当する。未使用fixtureの無効化は最小対案だが、一般的な純合成検証承認だけから設定変更を承認済みとしない。作者が未使用性と変更spanを確認して親へ返す。
33期待・旧REDの特定assertion・期限・判定を変更せず、旧失敗証拠/一回枠を保全する。設定差分と新一回枠の明示承認→独立差分Pre→再freezeまで修正・再実行を停止する。レジストリ権限引上げ/隔離外/admin/品質緩和での回避0、本追補は文書のみで実装・設定変更・対象実行・再委任0である。

## PSSA-CAP：エラー採取方式の承認済み限定見直し（2026-10-24、暫定計画）

開始HEAD `96a6ace`／tracked tree clean。利用者がエラー採取方式の限定見直しを承認した。完了境界は限定設計→独立Pre→必要な採取手順修正→独立局所検証→Postであり、元NullReferenceの原因確定や33候補実行の解除を先取りしない。planner所有は本書末尾のみ、未完了の旧計画を保持する。U-L～Pの設計/33期待/元error/期限/所有安全契約は再利用する。
T=`target/early-trial-build/et3/cargo-gate/verification/argv-path01/05685244-e82d-4a76-8be8-4366a8872b6d`。読取比較の正本はT/g6-evidence-static-blocker.json（索引SHA `21B00027783A0F1FE2DE7D247584A5EC5DE24DCA183F4D9BA645A51F7DBE38E1`）、T/g6-evidence-cause-diagnostic.json（`3D7D833A7C20A78EFDAF7D417327AE2AA1A20C20ABEF8DFCCEA99C66EB794137`）の保存request/toolreturnである。今回その値を現物hash照合し直したとはしない。

| 保存実行 | 同一Assertions `AFA82E807179DF852ECCB05D5A8C8F3919B9B1C4C9D981CFA0CD6F368FE824F4`に対する事実 | 限界 |
| --- | --- | --- |
| 通常初回／承認済みretry | PSSA1.25、ErrorAction Stop＋ErrorVariable＋catch、各exit2/ordinary0/engine2。tool wall time1.1868995／1.0906703秒 | engine2は同一失敗のErrorVariable記録とcaught wrapperであり独立障害2件ではない。戻ったstackはPowerShell側だけでrule/AST位置は未特定。途中停止のordinary0を全規則完走としない |
| 詳細診断一回 | PS7.6.7/PSSA1.25、ErrorAction Continue＋Verbose＋全stream統合、exit0、engine0、87件すべてVerboseRecord、採取1.8949785秒、前後hash一致 | preference・Verbose・stream経路を同時変更しており未再現理由を一意化できない。元失敗を解消済みにせず、診断をgatePASSへ転用0 |

CAP-R1：規則選択・設定・PSSA1.25/PS7.6.7を維持し、全severityの通常診断とengine errorを分離して採取する。いずれも1件以上、terminating例外、実行不完了、未知record、採取/保存失敗、drift、期限超過は非PASSとする。CAP-R2：ErrorVariableとcatchの重複を根拠付きで関連付けてもraw記録を捨てず、元errorを後段保存errorで置換しない。CAP-R3：入力/モジュール/command/規則・severity条件/前後hash/実exit/完了状態を固定記録し、取得不能は欠測とする。捕捉方式の変更を規則除外・依存更新・抑制追加・severity filterで代用しない。
対象は採取器とその独立局所対照だけである。PreparationAssertionsの期待/本文、準備sourceF91C、33候補receiptの実行、元ENVprobe/ValidateOnly、実OS/compile/製品、OS設定/admin/隔離外/実data/API、既成功51/P7反復、commit/pushは対象外。tests.patch31CEF…/108復元とsave-only-state/index/旧prefixは保全し、保存成功を実行freezeに読み替えない。

| 単位／依存・規模 | 所有と作業 | 受入・必要検証 |
| --- | --- | --- |
| CAP-D／本節受領、S | designerは既存unknown-descendant-design.md末尾のみ。既存3記録から採取/完了/判定/保存契約を定義し、変更点・合成seam・完全command案を固定 | CAP-R1～3、同一規則・全severity・engine必敗、元期限、重複関連付け/欠測/部分結果の扱いを独立設計Preで確認。原因仮説検証のための追加反復0。plannerは設計受領後に本節の未確定を精緻化 |
| CAP-T／CAP-D差分Pre、S | 独立test authorはT/pssa-capture/のTest-PssaCapture.ps1、expectations.json、局所receipt/manifestのみ（予定path、未作成） | 正常完了、全severity診断、nonterminating error、terminating errorとwrapper重複、部分出力後失敗、module/capture/save失敗、drift/期限境界、未知recordの独立期待を固定。入力数・command・試行数をPre前に確定し、既存33期待へ追加/流用しない |
| CAP-I／CAP-T期待Pre、S | implementerはT/pssa-capture/Invoke-PssaCapture.ps1の採取器1本だけ（予定path）。既存commandの採取責務を局所化し、新汎用framework/moduleや製品port変更を作らない | 必要な最小実装、Parser、許可範囲/旧入力不変を確認。期待作者は候補hashを固定期待へ束縛し、実装者が期待を変えない。独立差分Preと親freezeまで新対象PSSA実行0 |
| CAP-V／最終Pre・親freeze、S | 別runnerは新局所output/logだけ、reviewer read-only、親が既存review/artifacts/currentへ接合 | 新合成対照を各入力一回。その成功後、採取器自身と固定Assertionsを各一回までのPSSA静的対象として順序/完全commandをPreで固定し実施する。firstUnexpected/engine failure/通常診断/保存不全/drift/超過で停止・後続NotRun・retry0。独立Postで採取の妥当性とAssertionsの今回gate結果を別判定する |

新局所対照はmemory上の疑似record/例外/clock/保存sinkを基本とし、対象をdot-source/importして33動作を走らせない。通常sandbox内のPowerShell/PSSAによる静的解析だけを許可する。PSSA採取の入力10秒/集合30秒、既存合成case5秒/集合30秒をそれぞれ維持し、yieldや遅延返却を期限保証へ換算しない。ホスト上限・同期処理の中断限界は設計で明示し、無断のtimeout延長・外実行で救済しない。変更後の正常値が出るまで反復する方式は採らない。
最大リスクはContinue採取中のengine errorを通常output/Verboseに混ぜて捨てる偽PASS、Stopによる部分結果を完走とする偽PASS、同一errorの二重計上を別障害とする誤診である。typed/raw記録・独立失敗対照・完了flagを設計で対応させる。新方式で未再現でも元原因は未確定のままであり、再度内部例外なら保存された範囲でblockerを返す。

| 全CAP単位の役割割当 | exact model / effort | 適用・難度/リスク・理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 固定、CAP-D後精緻化。中～高／採取変更と品質gateの境界を管理 |
| external-spec author | `gpt-6-astra` / `high` | 固定、今回は非該当。既存all-severity/engine必敗/期限/外部期待を再利用する。変更が必要なら本計画を止め仕様工程へ戻す |
| designer | `gpt-6.1-sol` / `high` | CAP-D、後は設計差戻しのみ。高／PowerShellの複数error経路と不完了状態の設計 |
| test author | `gpt-6.1-sol` / `high` | CAP-Tと候補binding、他は非該当。高／偽PASSを見抜く独立期待と失敗対照 |
| implementer | `gpt-6.1-sol` / `medium` | CAP-Iのみ。中／独立設計で固定した局所採取1本へ限定し、設計判断を追加しない |
| test runner | `gpt-6.1-sol` / `medium` | CAP-Vのみ。中／固定入力・試行数・前後hashと停止条件の独立実施 |
| reviewer | `gpt-6.1-sol` / `high` | 設計/期待/候補の必要差分PreとCAP-V Post。高／採取成功とgate成功を区別し品質条件の保持を独立確認 |

上記は親提示の現在利用可能なexact ID/effortから選定した。非該当役は起動0、各単位に同割当を適用する。親modelは変更せず、通常補助1役ずつfresh context、作者/実装者/runner/reviewerを分離する。利用不能はblockerとしてplannerへ戻し、silent fallback/再委任0。役別既存Skillを再利用し、設計はapi-and-interface-design、作者はtest-driven-developmentの担当工程、実装/runnerはwordweave-changeの担当工程、reviewはreview-software-design/code-review-and-quality、PowerShell変更/レビュー時は対応powershell Skillを全文読了する。
task-list：[ ] CAP-D設計/独立Pre→planner精緻化、[ ] CAP-T独立期待Pre、[ ] CAP-I候補/独立Pre→親freeze、[ ] CAP-V局所検証/Post。成功した場合も33候補の再binding/実行Preは別工程であり今回解除0。停止時は未達gateと必要な追加判断だけを返す。実provider model/effort・子usage・親子包含は未取得であり親がKPIの取得可能範囲を記録する。
次handoffはCAP-D designer `gpt-6.1-sol/high`。本節と上記2JSONのrequest/toolreturn、現Assertions hashと旧保存索引だけを渡す。残る設計判断はStop/Continue等の採取方式、完了/未知recordの判定、重複の関連付け、有限な保存形式と合成入力数である。公開や他targetへ広げず、まず文書設計だけを行う。本planner工程のコード/期待編集・対象実行・新診断・再委任は0である。

### CAP-D是正Pre受領後の有限精緻化（2026-10-24）

親から設計SHA256 `8295FF7BDCF1AB56CA465A0B6BB1FD86263BD77D9C8B478371D98A7E338CA3A8`の独立限定再Pre PASSを受領した。初版20244…の実pipeline/EV/catch/finallyがseam外・最小shape未確定の2阻害は解消した。外側責任補足後の最新設計SHA256は`2704B28DA82AF4E79E44056DD6490DED2BA842649800E8363E6EE5A1D1CA42D1`であり、独立補足Pre PASSも受領した。CAP-D設計/Preと本精緻化を完了とし、CAP-Tへ進める。これは実行Pre/freezeの完了ではない。
採用契約はStop維持、実pipeline/逐次sink/EV/catch/finallyの原文抽出、producerだけmemory、固定state/flags/counts/返却shapeである。独立C01～C20の20期待を設計の照合先へ固定する。G6-05等の既存33期待/AssertionsAFA82E/準備sourceF91C/原NullReference2FAILは変更0である。

| 精緻化単位／依存 | 所有・受入・停止 |
| --- | --- |
| CAP-T固定20／設計Pre済み | test author `gpt-6.1-sol/high`は固定20期待/試験/adapterを作成する。今回plannerとの並行は文書と試験の独立所有に限る。最終receipt/command/hashは本精緻化後に接合し、未存在候補hashを埋めない。候補実pipelineを手動record投入/完了flag代入で代用0 |
| CAP-B旧方式焦点RED／独立期待・adapter Pre→親freeze | 同authorが保存staticRequests.pssaの旧`$diagnostics=@(...)`代入statementを特定し、元statement/hash/許可置換/逆bytes一致を固定する。producerだけをC07相当の実DiagnosticRecord出力→同一ErrorRecordのterminating throwへ結び、代入/catchは元方式を通す。「先行ordinaryを保持する」という焦点assertion1件の不成立を期待REDとする。原NullReference再現・missing helper・parse/adapter/module失敗をREDに数えない。別runner `gpt-6.1-sol/medium`が固定一回、特定assertion失敗のみexit1/accepted=true、その他失敗はexit2、予想外exit0も停止。実PSSA/候補20/33/実OSは0 |
| CAP-I／CAP-T期待Pre＋CAP-B特定REDの独立受入 | implementer `gpt-6.1-sol/medium`はC=`T/pssa-capture/Invoke-PssaCapture.ps1`一つだけを実装する。作者が独立固定期待を変えず候補へ束縛し、C/Test Parser0・許可差分・全pin・外側責任を独立Preへ渡す。新PSSAはCAP-Vまで実行0 |
| CAP-V／最終独立Pre→親freeze | 別runner `gpt-6.1-sol/medium`は固定20を各一回→C自身のPSSA一回→固定AssertionsのPSSA一回の順で実施する。firstUnexpectedで後続NotRun/retry0。20のexpected FAIL正検出はunit PASSであり、実PSSAの通常診断/engine error/不完了/保存不全/drift/超過は非PASSとして停止。reviewer `gpt-6.1-sol/high`のPostは20対照・self・Assertionsを別計上し、旧焦点RED1件も混算しない |

時間枠は旧焦点RED1件の5秒/合成集合30秒、候補20の各5秒/合成集合30秒、実PSSA各10秒（work9＋save1）/2対象共通集合30秒を別々に固定する。PSSAのCollectionStartTickはself前に一回設定しAssertionsで更新0、境界一致は超過である。外側責任は旧Invoke-PreparationReceipt.ps1:116–130/既R3と同じ受入期限であり、10/30秒のhard切断保証ではない。実行Preに親監視担当・時計・実host取消能力の有無（未提供なら未提供）・超過非PASS/後続NotRun/証拠保全を必須記録とし、未接合のまま旧RED/新対照/新PSSAをdispatchしない。yieldを保証とせず、新監督process/OS/compile側有限停止の解除や基準緩和は0である。既存設計command案を実行許可済みと扱わない。
復元所有は、implementerがCの全内容/差分を既存`unknown-descendant-harness.patch`へ、test authorが試験/期待/adapter/receipt/bindingとrunnerから受領した未加工証跡の復元情報を既存`unknown-descendant-tests.patch`へ追加する。旧prefix/履歴/108復元対象を保全し、独立に全bytes/hash復元確認する。runnerは原ログ/報告のみ、reviewerはread-only、親は索引/受領接合のみで、同じpatchへ同時書込0。新.gitignore例外・target直管理・旧patchの黙示置換0であり既存2例外を維持する。
全役model/effortはCAP表のまま、再選定なし。planner/必要時spec作者Astra/high、designer/作者/reviewer Sol6.1/high、implementer/runner Sol6.1/mediumをexact指定する。通常補助1役ずつ、実装/期待/実施/レビューの独立性を保持する。親の適時checkpoint commit/pushは利用者の既存standing authorizationに基づく別担当であり、本節の公開禁止は子に適用し、子commit/push/再委任0である。
更新task-list：[x] CAP-D設計Pre/精緻化、[ ] CAP-T固定20/焦点RED期待Pre、[ ] CAP-B実行Pre/freeze/一回/独立受入、[ ] CAP-I候補Pre/freeze、[ ] CAP-V20→self→Assertions/Post。次handoffは既担当authorへ本節を渡し、旧焦点REDの正確なstatement対応・完全command・新output UUID・receiptを固定してreviewerへ返す。計画だけによる新実行許可は0、33候補の最終binding/実行Preは引き続き未解除である。

### CAP-B分離Pre成立時の順序調整

親からCAP-T PreのC16（memory.followup初期値比較では後続停止未実証）差戻しと、旧RED固有adapter/receipt/18pins/監視/保存には追加阻害なしとの独立報告を受領した。20対照の受入とCAP-I実装は未解除である。
同一artifactのOldRed phaseがC16・候補C・20候補実行へ到達せず、旧REDの独立期待/adapter/全pin/完全command/監視/保存が単独で成立することをreviewerが限定実行Pre PASSとした場合だけ、親freeze後の旧RED1件先行を認める。根拠は旧代入方式の焦点assertionがC16の後続停止oracleにも未実装Cにも依存せず、両者の受入を変更しないことである。分離未証明・共通初期化からの到達・共有入力driftがあれば先行不可、実行0で止める。
順序は旧RED限定Pre→親freeze→別runner固定一回→独立受入→C16是正/20全期待Pre→CAP-Iとする。旧RED消費と証拠保全までpinned Test/expected/binding/receipt/patchの変更停止を維持し、是正後は新hash/Preへ接合する。実装開始にはCAP-Bの特定RED受入とC16を含むCAP-T全受入の両方を必須とし、片方だけで解除しない。
旧REDの特定assertion/exit分類/一回枠/期限/firstUnexpected/retry0、20期待・全severity・engine必敗、役割割当は不変である。これは条件付きの依存順調整であり、分離Pre未受領の現時点で実行を許可しない。本追補は文書のみ、コード/期待編集・対象実行0である。

### CAP-BD：旧RED失敗の限定採取診断

親から旧RED target1の固定request一回/exit2 Unexpected/accepted=false、Exceptionプロパティ不存在、18pins保全/保存正常、限定Postの記録整合確認を受領した。旧slot消費1/retry0/後続0、特定RED未成立を維持する。Test196–198の無型engineErrors要素にExceptionを仮定する箇所は候補であり、実entry型/stack/位置が欠測のため原因未確定である。C16設計87AD4D30…は限定Pre PASSだが現Testへの接合未完、CAP-IのCは未作成である。
本案は利用者承認済みCAPエラー採取限定見直しの通常の是正・関連検証の範囲内と判定する。追加するのは同じ純合成OldRed経路のbounded観測と新診断一件であり、対象/権限/規則/期待/品質gateを増減しないため追加authorityは不要である。ただし消費済み旧slotの再利用ではなく、以下の別設計・独立Pre・新freezeを要する。元Exception参照の失敗経路を直すこと、収集目的で実PSSA/OS等を使うことは本案に含めない。

| 有限単位／依存 | 所有・受入・停止 |
| --- | --- |
| BD-D／本追補 | designerは既存design末尾のみ。outer catchの元ErrorRecord固定scalar、ScriptStackTrace/位置、失敗entryと証明できる範囲のCLR型名（null/識別不能は別記）を限定採取する案を固定する。型で分岐して元Exception参照を救済せず、規則/期待/代入/producer/catch分類/元exitを保つ。上限bytes/文字数・取得失敗時の非干渉・保存位置と期限内記録を独立Preへ出す。任意object/環境/entry本文の全dump0 |
| BD-T／設計Pre | test authorは新UUID配下の診断用Testコピー・診断期待/adapter/receipt/bindingとtests.patch復元情報のみ。旧4DF版Test/旧expected/binding/receipt/patch/実失敗を全bytes/hashで保全し、変更は承認された採取spanと新path/hash接合だけに限定する。C16是正・20期待変更を混在させない。原経路との差分/逆一致とC/C16/20/33/PSSA未到達、診断型/位置の欠測規則、完全commandと新一回枠を独立実行Preで確認し親freezeする |
| BD-V／実行Pre・親freeze | 別runnerが通常sandboxで純合成診断一件を一回のみ行う。元の失敗/exit分類と別欄に診断取得の成否を保存し、診断が取れても旧RED/C受入を成立させない。前後pin/実exit/時計/新output/保存hashを記録し、unexpected/drift/超過/採取・保存失敗で停止・後続0/retry0。reviewerは元経路不変・新観測・旧失敗保全を独立Postで確認し、次の原因修正可否は別の限定判断へ返す |

BD-D/T/V共通割当はplannerと必要時external-spec author=`gpt-6-astra/high`、designer/test author/reviewer=`gpt-6.1-sol/high`、test runner=`gpt-6.1-sol/medium`、implementer=`gpt-6.1-sol/medium`とする。plannerは境界管理、designerは元失敗非干渉の設計、作者は独立診断期待/試験harness、reviewerは証拠とgate分離を担うため高、runnerは固定一回実施のため中である。implementerは今回は非該当でC未作成のまま、外部仕様も既契約再利用で不起動。通常補助1役・fresh context・各独立性/Skill/利用不能停止/親model不変を維持する。
旧焦点REDと同じ一件5秒/合成集合30秒を維持し、外側親監視担当/時計/実host取消能力有無/超過非PASS・証拠保全を新実行Preにも必須記録する。新監督process/admin/隔離外/OS/compile/実data/API/PSSA/20/33実行0。採取追加で元失敗を観測できなければ未再現/欠測とし、繰返して埋めない。
task-list：[ ] BD-D限定設計/Pre、[ ] BD-T新artifact/期待/実行Pre/freeze、[ ] BD-V一回/独立Post。次はdesigner `gpt-6.1-sol/high`へ本節と作者の限定read-only報告・旧失敗保存証拠だけを渡す。失敗entryを特定できる根拠が設計できなければ推定で補わず不明を保存する。CAP-B特定RED受入/C16現物接合/20Pre/C実装のgateは未解除、本planner追補のsource/期待編集・対象実行・再委任0である。

### CAP-BF：CLR型契約による参照照合の最小是正順

BDの保存unit-result.jsonを読取確認した。元exit2/Unexpected/accepted=false、PropertyNotFoundException、FQID=`PropertyNotFoundStrict,Microsoft.PowerShell.Commands.WhereObjectCommand`、位置194:34/stack195、failedEntryはUnidentifiedである。同Test194–198は無型engineErrorsの各entryへ`.Exception`を無条件に参照しており、型によって安全な参照照合を定義していないことは静的に確認できる。実際の失敗entry型・原NullReference原因はなお未確定、BDのposthash/保存完了・独立Postは親から未受領である。
追加診断の反復を前提にせず、型契約の不正仮定を直す限定設計へ進めてよいと判定する。CAP採取方式の承認内のtest harness是正であり、同じ純合成対象・元参照照合・特定RED期待を保ち、対象/権限/品質gateを広げないため追加authorityは不要である。BDで禁止した元経路修正は診断一件の非干渉条件であり、本是正単位とは分ける。実entry型未特定を修正成功の根拠にはせず、新artifactで型契約と実producer経路の両方を検証する。

| 最小順序 | 所有・受入・停止 |
| --- | --- |
| BF-D限定設計→独立Pre | designer `gpt-6.1-sol/high`はdesign末尾のみ。ErrorRecordは既存Exception参照、Exceptionは自己参照、Stop wrapperは既定の直接ErrorRecord参照だけを型確認後に照合する契約を固定する。unknown/nullは明示AdapterFailure、文字列一致/skip/dedup/再帰探索0。元entry/exception参照を置換せず、同文別参照は不一致を維持する。変更span・型別焦点6入力・旧RED1件との接続をreviewer `gpt-6.1-sol/high`が独立確認する |
| BF-T最小是正→独立実行Pre/freeze | BDの保存/posthash・独立Post受領とBF-D Pre後だけ、test author `gpt-6.1-sol/high`が新artifactのTest参照照合span、型別焦点試験/receipt/binding/tests.patchを所有する。6入力は同一ErrorRecord/同一Exception/直接wrapper/同文別参照/null/未知型に限定し、期待を候補結果から作らない。旧4DF/99305111版・BD原本/失敗/診断/slot履歴・旧patchを保全する。旧adapterの代入・producer出力→throw・旧prefix期待・特定assertion/exit分類を不変確認、C16混在0。期待と許可spanを先に独立確認し、修正版はParser/限定静的差分/復元一致→実行Pre→親freezeの順とする |
| BF-V別runner→独立Post | runner `gpt-6.1-sol/medium`が型別6入力を各一回、成功後に実producer出力→throwを通る旧RED一件を新枠で固定一回実施する。型別正検出と特定RED成立を別計上し、旧REDは特定assertionのみexit1/accepted=true、unknown/null・adapter等はexit2、予想外成功も停止。前後pin/保存/元参照をreviewerが独立Post確認する。欠測/不一致/超過で後続NotRun/retry0、消費済みslotは再利用0 |

一入力5秒/合成集合30秒、既定の親監視担当/時計/host取消能力有無/超過非PASS・証拠保全を維持し、新hard timeout・監督process・巨大matrix/framework・規則変更を導入しない。PSSA/20/33/実OS/compile/admin/隔離外/実data/API0。plannerは`gpt-6-astra/high`固定、外部仕様作者も同指定だが既契約再利用で非該当、CAP-I implementer `gpt-6.1-sol/medium`はC未作成のまま非該当とし、他役は上表のexact指定・通常補助1役・独立性を維持する。
次はBF-D文書設計だけである。BD独立Post未受領の現在はコード修正/実行へ進めず、CAP-I開始には修正版旧RED特定受入とC16現物接合を含む20全期待Preの両方が引き続き必要である。本判断は旧RED/C成功や実障害原因の確定ではなく、型契約是正の可否と順序だけを定める。plannerのsource/期待編集・対象実行・再委任0である。

### C16の期待Preと現物binding Preの依存分離

親からBF一回のfocus6 PASS＋特定旧RED1成立、exit1/1.170437秒/前後123照合一致を受領したが、独立Postは確認中であり、その受領前のコード変更・実行解除は0である。設計C16節338/342/344は未生成C/callerをPendingBindingとし、実装前は選択/置換/観測規則の期待Preだけを要求するため、以下の順序は既承認設計に沿う。
先行記述の「CAP-I開始前にC16現物接合を含む20全期待Pre」を本節で訂正する。BF独立Post受入→作者がC16の偽初期値比較を実span抽出/driver/caller境界へ改修→20独立期待とadapter codeのPre（C/callerの未存在hash/spanはPendingBinding）→CAP-IのC実装→作者がactual C/実callerを束縛→独立最終binding/実行Pre・親freeze→別runnerの20実行、の順とする。
実装開始にはBF特定REDの独立受入と20期待/adapter code Preを必須とし、C16の意味充足・runtime PASSは現物接合後の一回観測までPendingのままである。sourcehash/span推測、代用品if、初期値NotRunによるPASS、未束縛C16合格/20実行は禁止を維持する。実行gateを後退させず、入力未存在の段階に実装後の証拠を要求する循環だけを除く。
担当は作者`gpt-6.1-sol/high`→実装`gpt-6.1-sol/medium`→作者binding→独立reviewer`gpt-6.1-sol/high`→別runner`gpt-6.1-sol/medium`を維持する。20→self→Assertionsの実行順、回数、期限、全品質/権限条件は不変。本補足は文書のみ、コード/期待編集・対象実行・再委任0である。
