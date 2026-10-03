# REUSE-DESIGN-001 再利用境界の文書化計画

更新日: 2026-10-02。状態: 文書化完了 v1。仕様AC01–AC07採用・6設計文書作成・指摘修正後の独立レビューPASS。文書検査と取得範囲を限定した終了計測を記録済み。将来の切出し実装・実API・二製品での実行受入は含まない。

## 完了境界と権限

承認済み範囲は、音声評価/Qwen接続、Codex app-server接続、Markdown表示、チャットを別ツールでも同じライブラリとして使うための全体設計と境界をMarkdownに記録することである。明示的Qwen対応へ拡張する方向性も利用者承認済みである。今回の完了は、要求から設計への追跡、文書の整合・リンク確認、独立レビュー、残る設計判断の明示までである。将来APIの利用者承認、個別実装方式・時期・実API試験・本体規約更新、コードの再利用実証、製品変更の受入は含まない。

コード切出し、Cargo・依存・設定変更、実API呼出し、実学習データ・音声の試験、commit/push/公開、親モデル変更、品質ゲート変更は対象外である。既存 `tasks/plan.md` と `tasks/todo.md` の未完項目を保持する。計画担当の所有は本ファイルと [todo.md](todo.md) だけである。

基準HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。作業開始時点で本体・Qwen・他タスクに多数の未コミット変更が存在するため、HEADとの差分全体を本タスクの変更とみなさない。親が開始時の状態と成果物別差分を識別し、他者の変更を戻さない。

## 要求と受入条件（採用済み仕様との対応）

| ID | 要求 | 文書で判定する受入条件 |
| --- | --- | --- |
| R01 / AC01 | 四機能を共通ライブラリとして再利用できる境界を定める | 全体図と各機能文書があり、現状・提案・未検証を区別し、現状の主張にソース根拠がある。既存Qwen lib/GUI/MCPを新規未実装として説明しない |
| R02 / AC02 | アプリ固有責任を残す | 四機能それぞれに入力・出力形式、操作、検証・上限、公開面、依存方向、状態/データの所有者、WordWeave5側adapter、対象外が記載される。教材・学習記録・UI保存を共通coreに混入させず、未確定値は提案または未決とする |
| R03 / AC03 | 認証と利用者の権限を維持する | 現行本体のChatGPT認証app-server限定、API-key fallback禁止、Volta shim直接起動、システム/ユーザーPATH探索、返却model/effortを推測しない表示を維持する。明示的Qwen対応の方向性は承認済みであり、個別実装方式・時期・実API試験・本体規約更新は今回の文書作成承認に含めない。具体的契約は提案とし、Codex失敗時に自動転送しない |
| R04 / AC04 | 同意・取消し・結果不明を区別する | 外部送信の同意、承認前の教材登録禁止、未送信取消し、取消要求と確認済中断、結果不明、結果取得と保存成功/失敗を区別する。各機能の適用/非該当理由、再取得・再送・再生成の違い、決定主体、禁止操作を追跡できる |
| R05 / AC05 | 互換性と移行条件を定める | プロトコル・公開型・保存形式・renderer依存の互換性、変更検知、段階導入、失敗時の復帰、現行仕様保持の条件が記載される。未実装crate/API例はproposedと表示する |
| R06 / AC06 | 二消費者で再利用性を判定する | 各機能にWordWeave5と第二の実製品候補、同一ライブラリ版を使う条件、独立adapterの責務、成功/境界/失敗の観測値を定める。試験用最小アプリと実製品の利用実績、文書確認と将来の実行確認を区別する |
| R07 / AC07 | 今回の文書を検証可能にする | 仕様ID→設計節→確認記録が追跡でき、リンク・Markdown・禁止範囲の変更有無を確認し、独立レビューの指摘を解消または未決として正確に記録する |

R01–R07/AC01–AC07は [spec.md](spec.md) で同じ番号の文書受入条件として正式化・採用済みである。上表はその要約であり、期待値の正本は仕様とする。[results.md](results.md) にAC→文書節→確認方法の追跡表がある。文章の存在だけで設計の正しさや実装動作を合格扱いしない。

## 入力と既知の制約

- 必須規約: [AGENTS.md](../../AGENTS.md)、[開発7役](../../docs/process/development-team.md)、[KPI](../../docs/process/improvement/kpi.md)。planning-and-task-breakdown SkillとそのDefinition of Done参照を全文読了した。Skillの既定出力先は、明示された本タスク用ディレクトリに置き換える。
- 親のソース確認: `tools/qwen-audio` はlib/bin、単独GUI/MCPを持つが、eframe/rfd/Windows依存が非optionalで、libのglob再exportと `pub mod gui` がある。存在と十分な独立性を区別する。
- 親のソース確認: `src/codex.rs` はdiagnostics/run_journal/execution/effortへ、`src/chat.rs::prepare_with_catalog` は `model::Entry` へ依存する。
- 親のソース確認: Markdownは `src/app/notifications/markdown.rs`、pulldown-cmark 0.13、egui 0.31.1、見出しフォント名、本体harnessに依存する。
- 既存資料: [Qwen設計](../QWEN-AUDIO-001/design.md)、[Qwen結果](../QWEN-AUDIO-001/results.md)。既存結果を今回の試験結果に流用しない。
- [ADR0002](../../docs/decisions/0002-reusable-library-boundaries.md)を設計方針案として作成済みである。現行ADR0001を全面置換せず、将来実装・公開API凍結の承認と区別する。

## 成果物・所有・依存順

| 単位 | 規模 / 難度・リスク | 出力・唯一の書込み担当 | 依存 / 検証の要点 |
| --- | --- | --- | --- |
| P00 暫定計画と精緻化 | S / 中 | `tasks/REUSE-DESIGN-001/plan.md`、`todo.md`：planner | 親からの範囲・利用可能モデル。S00後と設計後に更新し、旧計画を保持する |
| S00 文書要求・観測可能なAC | S / 高 | `tasks/REUSE-DESIGN-001/spec.md`：外部仕様担当 | P00。承認済み文書作成と未承認の将来製品仕様を区別。独立仕様レビュー後にP00更新 |
| D00 全体設計 | S / 高 | `docs/reuse/README.md`、`docs/decisions/0002-reusable-library-boundaries.md`：親 | S00。現状/目標、依存方向、共通原則、四機能間の責任、採否理由を記録 |
| D01 音声評価/Qwen | S / 高 | `docs/reuse/qwen-audio.md`：設計担当 | S00・D00共通境界。lib/GUI/MCPの現状、provider/session/音声data境界、同意と第二消費者条件 |
| D02 Codex app-server | S / 高 | `docs/reuse/codex-app-server.md`：D01と同じ設計担当 | S00・D00共通境界。認証・起動探索・実行状態・結果回収とアプリjournalの責任分離 |
| D03 Markdown | S / 中 | `docs/reuse/markdown.md`：親 | S00・D00共通境界。parse/render/UIテーマ・フォント・リンク操作の責任、対応構文とrenderer互換性 |
| D04 チャット | S / 高 | `docs/reuse/chat.md`：D01/D02と同じ設計担当 | S00・D00・D02の契約。会話/媒体/stream状態と教材・学習記録adapter、永続化・取消し・第二消費者条件 |
| V00 文書整合・独立レビュー | S / 高 | `tasks/REUSE-DESIGN-001/results.md` と必要な短い `tasks/current.md` 更新：親。reviewerは読取り専用 | S00・D00–D04。仕様/設計レビュー、リンク・差分・追跡性確認。指摘修正は元の所有者へ戻す |

D01/D02/D04は一人の設計担当が3文書を直列に担当する。親はD00/D03とソース根拠・統合を進める。共有API/状態定義はD00の合意を先行させ、各機能が別々の共通型を確定しない。仕様レビューと設計レビューは必要な時点ごとに独立reviewerを起動/再開する。通常は親＋補助一役で進め、七役の同時起動や再委任はしない。

`tests.md`、試験ソース、実装用stubは今回追加しない。文書検証の期待値はS00に集約し、実施記録はV00に集約する。

## 設計後の精緻化と残るgate

S00は独立仕様レビューでblocking指摘なし、AC01–AC07を親が採用した。D00–D04の6文書は作成済みである。全体設計は機能ごとの境界を採用し、Codex/Qwenの能力・認証を無理に共通化せず、ホストadapterが同意・保存・教材固有処理を担う。段階移行の候補と進行条件は [全体設計7節](../../docs/reuse/README.md) に記載済みであり、実装順の承認や実行済みの意味ではない。

第二消費者候補は、Qwenが既存単独ツール、Codexが教材を持たない文章支援host、Markdown/チャットが教材を持たないノート支援hostである。同一版・独立adapter・正常/境界/異常の観測条件を各文書に置いた。採用製品の確定と両製品での実行は後続タスクのgateである。

初回独立設計レビューのP2は2件（レビュー指摘ID D01: Markdownの第二実製品候補不足、D02: Qwenの承認済み方向性を未決に戻した不整合）。所有者による修正後、独立reviewerが両方の解消とAC01–AC07の文書受入PASSを確認した。最終判定は [results.md](results.md) に記録する。ここでのD01/D02はレビュー指摘IDであり、作業単位D01/D02とは別である。

初回検査は10文書・91相対リンク・34表で問題0、修正・精緻化後は10文書・99相対リンク・35表で問題0、`git diff --check` exit 0。2026-10-02T23:08:42.1719462+09:00時点のソース98件とルートCargo2件のSHA256は開始値と一致した（親の [検証記録](results.md)）。終了記録の追記後の最終検査値も同記録を正とする。古い検査結果を修正後のPASSに転用しない。

## 役割・モデル・effort

親から提供された利用可能一覧: `gpt-6-astra`、`gpt-6.1-sol`、`gpt-6-sol`、`gpt-5.6-sol` は low/medium/high/xhigh/max/ultra、`gpt-6-luna` は low/medium/high/xhigh/max。モデル名から成果品質を判定しない。未対応の指定・必須Skill不在は停止条件であり、暗黙に別モデル/Skillへ置換しない。

固定役: plannerは `gpt-6-astra / xhigh`、外部仕様担当は `gpt-6-astra / high`。計画は認証/取消し/互換性を横断して責任を分けるためxhigh、仕様は限定された文書成果物でも将来挙動の承認境界を定めるためhighとする。起動指定と実行値は別物であり、plannerの実行metadataは未取得である。

各単位の可変五役は以下のとおりである。`inherit / inherit` は親の設定を変更せず責任を親へ集約する指定である。親設定は利用者申告 `gpt-6-astra / xhigh`、実行metadataは未取得。xhighは提供一覧上Astraで対応している。非該当欄のinheritは本体コード工程を起動しないための記録であり、未実行モデルの起動成功や将来実装の割当を意味しない。

| 単位 | 設計担当 model / effort | 実装担当 | テスト作成担当 | テスト実施担当 | 独立レビュー担当 model / effort |
| --- | --- | --- | --- | --- | --- |
| P00 / S00 | `inherit / inherit`：親が計画/仕様の文書構造を統合 | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：親が仕様ACの不足を確認。本体試験作成なし | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high`：仕様レビュー |
| D00 | `inherit / inherit`：親 | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：親がS00の期待値を維持。本体試験作成なし | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high` |
| D01 | `gpt-6-astra / high` | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：同上 | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high` |
| D02 | `gpt-6-astra / high` | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：同上 | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high` |
| D03 | `inherit / inherit`：親 | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：同上 | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high` |
| D04 | `gpt-6-astra / high` | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：同上 | `inherit / inherit`：親が文書確認 | `gpt-6-astra / high` |
| V00 | `inherit / inherit`：親が指摘を所有者へ差戻す | `inherit / inherit`：非該当・起動なし | `inherit / inherit`：期待値の無断緩和なし | `inherit / inherit`：親が文書確認と結果記録 | `gpt-6-astra / high` |

選定理由: D01/D02/D04は認証・結果不明・データ所有・保存責任が交差し、低い切出し粒度でも設計誤りの影響が大きいためAstra/highとする。D00/D03は親の独立作業として担当範囲を保持し、親モデルの変更を要求しない。文書確認は小規模で独立実装がなく、親が再現可能な検査を実施する。reviewerは四機能の境界矛盾と承認状態を独立に評価するためAstra/highを選ぶ。担当の固定化による自己承認を避け、reviewerは作者を兼ねない。

必要Skill: 外部仕様は `agent-skills:spec-driven-development` の仕様工程、設計は `agent-skills:api-and-interface-design`、仕様レビューは `review-external-specification`、設計レビューは `review-software-design`。各担当自身が該当Skillを全文読み、文書の所有範囲を越えない。実装/実行試験用Skillは今回非該当である。

親は起動ごとにexact model/effortと役の検出可否を照合し、起動指定と取得できた実行metadataを分けて記録する。利用可能一覧は親提供の2026-10-02スナップショットであり、実行成功の証明ではない。

## チェックポイント・必要検証

1. S00後（完了）: 仕様のACと範囲を独立レビューし、AC01–AC07を採用した。文書作成と明示的Qwen対応の方向性は承認済み、将来の個別API・実装契約はproposedである。
2. D00/D01/D02後（完了）: 共通状態・認証・data責任を照合し、Qwen方向性の承認範囲の修正も独立再確認済みである。
3. D03/D04後（完了）: AC追跡表と二消費者候補・観測条件を記録し、P00をv1へ更新した。Markdown第二実製品候補の追記も独立再確認済みである。
4. V00（完了）: 修正と本精緻化を含む独立再レビュー、影響範囲の文書検査、終了計測を記録した。設計書を引き渡し、将来実装・実機/API試験は未実施として区別する。

文書検査では対象を本タスクの明示パスに限定する。

- `git diff --check -- tasks/REUSE-DESIGN-001 docs/reuse docs/decisions/0002-reusable-library-boundaries.md`。未追跡新規ファイルは通常のgit diffに出ないため、直接読取りによる末尾空白・競合marker・Markdown fence検査も実施する。
- Markdownのローカルリンクは各文書の所在を基準に解決し、リンク先の存在を確認する。計画段階の未作成予定成果物はplannedとし、最終時には未解決リンクを残さない。
- 仕様ID→四機能の設計節→検証結果を対応表で確認する。各機能の成功・失敗、同意・取消し・結果不明、互換性・第二消費者を意味内容までレビューする。
- 開始時のgit statusと最終状態を比較し、本体・依存・設定・既存未完計画に本作業が書き込んでいないことを確認する。他者の同時変更を自分の変更や違反と決めつけない。
- 新規挙動を実装していないためcargo test/release buildは今回非該当である。[AGENTS.md](../../AGENTS.md) のdocs-only規則による適用範囲であり、将来のWindowsアプリ納品時には `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5` を維持する。

## リスクと未決事項

| リスク | 影響 | 抑制策 |
| --- | --- | --- |
| 既存libの存在を他ツール向け独立性と同一視 | GUI/OS依存やWordWeave型が漏れる | 現状依存・目標境界・二消費者条件を別記する |
| 四機能を早期に一つの万能coreへ一般化 | 異なる認証・状態・renderer制約を失う | 最小共有契約と機能固有能力を分け、共通化の採用条件と不採用範囲を記録する |
| クライアント停止を外部完了/中断と誤認 | 二重生成、同意逸脱、結果喪失 | 取消要求・確認済中断・結果不明を明記し、再送/回収の責任主体を置く |
| 現在の認証規約とQwenの承認範囲を混同 | 承認済みの方向性を未決に戻す、または未承認の実装まで確定する | 明示的Qwen対応の方向性は承認済みとし、個別実装・時期・実API・規約更新と分離する。現行Codex-onlyとAPI-key fallback禁止を維持する |
| 文書だけで再利用達成を宣言 | 実行・ビルド・互換性未検証を隠す | proposed API・未実装crateを表示し、二消費者実行は将来gateとして残す |
| 並行変更・既存未完計画を上書き | 別作業の損失 | 排他的な文書所有と開始差分の保持、共有ファイルは親だけが編集する |

未決: 第二消費者の正式採用と言語/UI/OS、最終crate名・署名・MSRV、配布単位と公開場所、最初に着手する切出しタスク、公開APIの凍結条件、Qwenの具体的導入方式・時期・評価品質基準・実API試験・本体規約更新、保持期間/保存形式の互換期間、Markdownの他UI対応範囲。候補と決定時点は設計に記載済みである。明示的Qwen対応の方向性は承認済みであり未決に戻さない。具体的な後続実装や二消費者での実行合格も今回の文書承認から推定しない。

## 計測と次のhandoff

KPIの完了境界は本タスクの設計文書一式である。子IDは `/root/reuse_planner`、親IDは `/root`（session IDはresults参照）。子の開始/終了tokenカウンター・実行model/effort・親子包含関係は未取得であり、推定しない。親の開始/終了観測値と欠測は [results.md](results.md) と `docs/process/improvement/measurements.md` に記録済みである。通常exec/Node読取りはWindows sandbox ACL初期化エラーで失敗し、承認付きread-only execで必須資料を取得した。この復旧は設定や品質ゲートの変更を伴わない。

親が同じ `gpt-6-astra/high` 指定の独立reviewerへ限定再確認を依頼し、指摘D01/D02の解消とAC01–AC07の文書受入PASSを受領した。plannerから移譲された最終状態/チェックだけを親が更新した。要求・モデル・品質ゲートは変更していない。後続は利用者が選ぶ実接続確認または個別の切出しタスクであり、今回から自動的に実装・実API・公開へ進めない。
