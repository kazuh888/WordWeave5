# FRAMEWORK-DESIGN-001 文書化計画

2026-10-03 / 状態: v2、文書化完了。S00/設計後の精緻化、独立レビュー、文書検査を完了した。証拠は [results.md](results.md)。実装・Framework完成の報告ではない。

## 完了境界と根拠

将来の国語/理科学習ツールに備え、Frameworkの責任境界、既存4設計の必要な補正、追加ライブラリ候補の採否・着手条件を文書化し、独立レビューと文書検査を通す。今は境界を設計し、第二教科の「出題→回答→評価→記録」の縦断実証でFramework契約を確定する。

入力は親からの承認済み依頼、[AGENTS.md](../../AGENTS.md)、[開発7役](../../docs/process/development-team.md)、[KPI](../../docs/process/improvement/kpi.md)、[既存全体設計](../../docs/reuse/README.md)、[前タスク仕様](../REUSE-DESIGN-001/spec.md)である。planning-and-task-breakdown SkillとDefinition of Doneを全文読了した。明示された本ディレクトリをSkillの既定出力先に優先する。

基準HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`、多数の未コミット変更がある。親が今回開始時の非変更baselineを保存し、HEAD差分全体を今回の成果としない。既存 `tasks/plan.md`、`tasks/todo.md`、`tasks/REUSE-DESIGN-001/` は保持する。計画担当の所有は本書と [todo.md](todo.md) のみである。

対象外: コード/試験/Cargo/API呼出し/設定の変更、crate切出し、教材・記録・原音・inkの操作、commit/push/公開、親モデル・品質ゲートの変更。教科の自動採点、教育効果、全OSを保証しない。Qwenへの明示対応方向は承認済みであり、個別実装・実API・現行規約の更新を今回の承認へ含めない。

## 暫定要求と文書AC

以下は仕様担当へ渡した要求であり、観測可能な期待値の正本は [spec.md](spec.md) とする。S00独立レビューにP1/P2なし、親が採用した。承認済み範囲を再承認待ちへ戻さない。

| ID | 要求 / 合格条件 |
| --- | --- |
| F01 | 現行の英語 `Entry/Skill/Progress` 結合と、Framework未完成を根拠付きで記す。部品再利用と教科横断Frameworkを別の達成条件にする |
| F02 | 共通基盤・教科固有・ホストadapterの責任、依存方向、状態/データの所有を定める。今確定する境界と縦断実証後に確定する契約を区別する |
| F03 | 既存Qwen/Codex/Markdown/chatの4設計を全件点検し、Framework観点で必要な節だけ補正する。現行認証・原本保持・送信同意・教材案/可視差分/学習者承認を保持する |
| F04 | 録音再生TTS・添付原本管理・egui共通UI・通知診断・差分表示・手書きの6候補、保存・資格情報・MCPの補助3候補に「今設計」または「着手条件を記録して保留」、理由・依存・境界・解除条件を記す |
| F05 | 今設計する候補は入出力/操作・上限の決定主体・所有/状態・正常/境界/失敗・互換性・二消費者での将来検証を示す。公開面を必要最小限とし、教科の意味やOS保証を混入させない |
| F06 | 第二教科の縦断シナリオをFramework確定gateとして記す。今回の文書確認と将来の実行受入を分け、要求→設計節→独立レビュー/検査を追跡し、未決を合格へ読み替えない |

## 作業単位・唯一の書込み担当・依存

F01→FW-AC01、F02→02、F03→03/06、F04→04、F05→05/06、F06→07/08で追跡する。D10は3候補を今設計、6候補を条件付き保留とした。以下を唯一の所有とする。

| 単位 | 規模 / 難度・リスク | 成果物と所有者 | 依存 / 受入・検証 |
| --- | --- | --- | --- |
| P00 暫定計画→精緻化 | S / 中 | 本書・todo.md: planner | 依頼・モデル一覧。全単位にAC/所有/依存/検証/役割があり、未完計画を保持する |
| S00 文書受入仕様 | S / 高 | `tasks/FRAMEWORK-DESIGN-001/spec.md`: 外部仕様担当 | P00。F01–F06を検証可能にし、承認境界・非保証を固定。独立仕様レビュー後に親が採用 |
| D10 Frameworkと候補判定 | M / 高 | `docs/reuse/framework.md`、`library-candidates.md`、`README.md`、`docs/decisions/0003-learning-framework-boundaries.md`: 親 | S00。F01/F02/F04/F06。9候補の判定と縦断gateが一貫し、根拠のない抽象型を増やさない |
| D20 既存4設計の補正 | M / 高 | `docs/reuse/qwen-audio.md`、`codex-app-server.md`、`markdown.md`、`chat.md`: 親 | D10の境界。F03。4件の必要差分/変更不要理由を確認し、既存ACと矛盾しない |
| D30 必要な追加部品設計 | M / 高 | `docs/reuse/audio-io.md`、`asset-store.md`、`egui-controls.md`: 設計担当 | D10のL01–03。F05。2026-10-03に台帳と正確な所有パスを指定してdispatch。部品の現状/提案/未検証を区別 |
| V00 独立確認と文書検証 | S / 高 | `tasks/FRAMEWORK-DESIGN-001/results.md`、必要な短い `tasks/current.md`/KPI記録: 親 | S00/D10/D20/D30。F01–F06の追跡・検査・独立レビュー・欠測を記録する |

依存は P00→S00→仕様確認→D10→{D20,D30}→設計確認→V00 である。親がD20と統合を進める間にD30を独立委任できる。D30の同一文書やD10の共有契約を二者で編集しない。通常は補助1名、再委任なし。今設計が3件を超えた場合は根拠と追加単位を計画へ戻し、一単位を肥大化させない。

P00はS00後に要求/AC対応を更新し、D10–D30後に実際の採否・所有パス・依存・未決を更新する。完了証拠を待つだけの別工程は作らず、精緻化後の最終状態/チェック更新は親へ移譲する。要求・モデル・品質ゲートの変更はこの移譲に含まれない。

## 七役・モデル・effort

親の提供一覧（2026-10-02受領）: `gpt-6-astra`、`gpt-6.1-sol`、`gpt-6-sol`、`gpt-5.6-sol` は low/medium/high/xhigh/max/ultra、`gpt-6-luna` は low/medium/high/xhigh/max。未検出モデル/必須Skillは停止条件であり、無断fallbackしない。

plannerは全単位の分解/精緻化で `gpt-6-astra / xhigh`、外部仕様担当はS00で `gpt-6-astra / high` 固定である。他単位の外部仕様作成は非該当（S00を再利用）とし、仕様変更が必要ならS00へ戻す。計画は横断境界と七役の調整、仕様は限定文書の承認/非保証の明確化を要するため、このeffortを選ぶ。

可変五役は各単位へ以下を割り当てる。`inherit / inherit` は親の設定を維持する明示指定であり、親の実行model/effortは未取得である。親の既存実行を変更せず、新しい子に不明な継承値を渡さない。非該当のinheritは起動しない責任欄であり、将来コード実装の割当ではない。

| 単位 | 設計担当 model / effort | 実装担当 model / effort | テスト作成担当 model / effort | テスト実施担当 model / effort | reviewer model / effort |
| --- | --- | --- | --- | --- | --- |
| P00 | `inherit / inherit`: 親の文書統合 | `inherit / inherit`: 非該当 | `inherit / inherit`: 非該当 | `inherit / inherit`: 親の文書検査 | `gpt-6.1-sol / high` |
| S00 | `inherit / inherit`: 親の範囲整合 | `inherit / inherit`: 非該当 | `inherit / inherit`: 本体試験作成は非該当、期待値は外部仕様担当 | `inherit / inherit`: 親の文書検査 | `gpt-6.1-sol / high`: 独立仕様レビュー |
| D10 | `inherit / inherit`: 親 | `inherit / inherit`: 非該当 | `inherit / inherit`: 非該当、S00維持 | `inherit / inherit`: 親の文書検査 | `gpt-6.1-sol / high` |
| D20 | `inherit / inherit`: 親 | `inherit / inherit`: 非該当 | `inherit / inherit`: 非該当、S00維持 | `inherit / inherit`: 親の文書検査 | `gpt-6.1-sol / high` |
| D30 | `gpt-6.1-sol / high` | `inherit / inherit`: 非該当 | `inherit / inherit`: 非該当、S00維持 | `inherit / inherit`: 親の文書検査 | `gpt-6.1-sol / high` |
| V00 | `inherit / inherit`: 修正を元所有者へ戻す | `inherit / inherit`: 非該当 | `inherit / inherit`: 非該当、期待値を緩和しない | `inherit / inherit`: 親が結果記録 | `gpt-6.1-sol / high`: 独立設計レビュー |

選定理由: D10/D20は親の既存読解と統合責任を保持する。D30は契約が先に定まる最大3部品だが、原本/取消し/保存失敗の境界を扱うためSol/highを選ぶ。reviewerは仕様と設計の不整合・過剰一般化を独立判定するためSol/highを選ぶ。モデル名を品質の実測証拠とせず、不足が見つかればplannerへ再選定を返す。

仕様担当は `agent-skills:spec-driven-development` の仕様工程、設計担当は `agent-skills:api-and-interface-design`、reviewerは `review-external-specification`→`review-software-design` を責務別に全文読む。レビューはread-onlyで作者と分離し、一人が順次担当してよい。文書のみのため実装/テスト作成Skill・試験ソース・stubは追加しない。

親は起動前に役検出・exact model/effort対応を照合し、指定値と取得できた実行metadataを別記する。新規子には上表の明示値を渡す。plannerの起動指定はAstra/xhigh、実行metadataは未取得であり、起動成功や指定だけで確認済みとしない。

## チェックポイント・必要検証

1. S00後: 仕様の独立レビュー、親による承認済み要求との照合・採用、P00のAC精緻化。重大な仕様不明点はここで止める。
2. D10とD20/D30の着手前: 候補台帳・共通契約・所有パスを固定し、並行作業の衝突をなくす。
3. D20/D30後: P00精緻化、仕様/設計の独立判定、指摘を所有者が修正。影響する確認だけ再実施する。
4. V00: 下記検査とF01–F06追跡を記録し、未決/未検証を示して文書成果を引き渡す。将来実装へ自動継続しない。

- `git diff --check -- tasks/FRAMEWORK-DESIGN-001 docs/reuse` と対象Markdownの直接読取りで末尾空白・競合marker・fence/tableを確認する。未追跡ファイルも必ず含める。
- 相対リンク/anchor、候補9件の網羅、仕様ID→文書節→確認結果、現行/提案/未検証・所有/依存/非保証の意味内容を確認する。旧タスクのPASSを今回へ流用しない。
- `git status --short` と開始baselineを照合し、所有外・コード/試験/Cargo/設定・旧計画の非変更を確認する。他者の並行変更と本作業の変更を区別する。
- docs-onlyのためcargo test/release buildは非該当である。将来のWindows実装納品の `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`、実機受入を弱めない。

## リスク・未決・次のhandoff

| リスク | 抑制策 |
| --- | --- |
| 英語の型名だけ中立化してFramework完成とする | 第二教科の縦断で出題/回答/評価/記録の異なる意味とadapter独立性を観測するまで確定しない |
| 候補すべてを今設計し公開面が肥大化する | 依存が今の設計を妨げるものだけ設計し、保留は解除条件を残す |
| 音声助言を国語/理科の自動採点・教育効果へ一般化する | 教科側の評価責任と品質検証を分離し、非保証を明示する |
| 共通保存/資格情報/取消しが既存安全条件を壊す | 原本・同意・結果不明・保存成功を別責任として既存4設計と照合する |
| 文書合格を二製品実行/全OS保証と誤認する | 今回の証拠と将来gateを明記する |

未決は第二教科と最小課題、評価主体・記録項目、最終公開型/配布/対応OSである。国語の記述を第一候補としたが着手承認ではない。追加部品/所有パスは上表で確定した。S00はAstra/highへdispatchして作成、Sol/highの独立確認後に親が採用した。D30はSol/highへdispatch済み。最終状態更新と限定修正は親へ移譲するが、意味の変更には影響範囲の独立確認を行う。

計測: 子ID `/root/framework_planner`、親ID `/root`。開始/終了tokenカウンター・包含関係・実行model/effortは未取得。推定せず親のresults/KPIへ欠測として渡す。通常execはsandbox ACL初期化で失敗し、指示資料は承認付きread-only execで取得した。設定・ゲート変更はない。
