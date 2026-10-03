# REUSE-DESIGN-001 タスク一覧

計画正本: [plan.md](plan.md)。状態: 文書化完了 v1。仕様採用・6設計文書作成・指摘修正後の独立レビュー・文書検査・取得範囲を限定した計測を記録済みである。文書作成と明示的Qwen対応の方向性は承認済みであり、個別API・実装方式・時期・実API試験・本体規約更新は今回の承認に含まれない。既存の `tasks/plan.md` / `tasks/todo.md` は変更しない。

## P00 暫定計画と精緻化

範囲: 要求・除外・依存・所有・モデル/effort・検証を定義する。規模S、難度中。所有者planner、`gpt-6-astra/xhigh`。実行metadataは未取得。

- [x] 必須規約、planning SkillとDefinition of Done、KPIを読み、既存未完計画と開始git statusを確認した。
- [x] 本タスクだけの暫定plan/todoを作成し、文書工程と非該当工程を分けた。
- [x] S00と6設計文書・初回指摘修正の結果を反映し、親へ精緻化済み計画・未決事項・次handoffを返した。

検証: AC、依存順、5役の適用/非該当、exact model/対応effort、ファイル所有、既存未完計画の保持を照合する。依存: 親の作業範囲と利用可能モデル一覧。変更ファイル: `plan.md`、`todo.md` の2件。

## S00 外部仕様・受入条件

範囲: 利用者が文書から判断できる成果と、将来の再利用時に観測する挙動を区別する。規模S、難度/リスク高。所有者spec-author、`gpt-6-astra/high`。

- [x] 四機能の要求/AC、現行と将来、承認状態を定義した。
- [x] 認証・同意・取消し・結果不明・互換性・二消費者条件の期待値を定義した。
- [x] 独立仕様レビューはblocking指摘なし、AC01–AC07を親が採用した。Qwen承認範囲の後続修正も反映済みであり、修正後の独立確認はV00に残す。

検証: R01–R07の欠落・矛盾・判定不能な期待値がないかをreviewer `gpt-6-astra/high` が確認する。依存: P00。変更ファイル: `spec.md` 1件。製品API承認は今回の受入と分ける。

## Checkpoint A 仕様確定範囲

- [x] 採用済み仕様と設計レビュー指摘の修正を計画v1に反映した。
- [x] Qwen対応方向の承認と、個別API/実装/実API/規約更新の未承認を区別した。

## D00 共通設計

範囲: 四機能の依存方向、再利用単位、共通原則、段階移行の判断を記録する。規模S、難度/リスク高。所有者親、model/effort `inherit/inherit`。

- [x] 全体図が現状と提案を区別し、各機能文書への導線を持つ。
- [x] core・provider・UI/永続化adapter・アプリ固有責任と、共通化しない境界を記録した。
- [x] 未決API、第二消費者候補、移行/復帰条件とADR0002を記録した。

検証: AC01–AC07との対応、依存方向と四機能の語彙を確認する。依存: S00/Checkpoint A。変更ファイル: `docs/reuse/README.md`、`docs/decisions/0002-reusable-library-boundaries.md` の2件。

## D01 音声評価/Qwenの境界

範囲: 既存lib/GUI/MCPを基点に、音声data・評価・provider/session境界を設計する。規模S、難度/リスク高。所有者designer、`gpt-6-astra/high`。

- [x] 現在の依存と再export/GUI公開面、再利用に必要な目標境界を記録した。
- [x] 同意・取消し・結果不明・認証・原音保持の責任を記録した。
- [x] WordWeaveと既存単独ツールが同一版を使う将来検証条件・移行条件を記録し、Qwen対応方向の承認範囲を修正した。

検証: AC01–AC06、既存Qwen設計/結果・ソース根拠との照合。依存: S00とD00共通境界。変更ファイル: `docs/reuse/qwen-audio.md` 1件。

## D02 Codex app-serverの境界

範囲: app-server接続、認証、起動、実行状態とアプリjournal責任を分ける。規模S、難度/リスク高。所有者D01と同じdesigner、`gpt-6-astra/high`。

- [x] ChatGPT認証、fallback禁止、Volta/PATH、返却model/effort表示を保持する境界を記録した。
- [x] 取消要求・確認済中断・結果不明・結果回収と再生成の条件を記録した。
- [x] WordWeave依存から分離する契約、文章支援hostを第二候補とする条件、互換性条件を記録した。

検証: AC01–AC06、`src/codex.rs` と親のソース根拠との照合。依存: S00とD00共通境界。変更ファイル: `docs/reuse/codex-app-server.md` 1件。

## Checkpoint B 外部処理の共通境界

- [x] D00–D02の認証・同意・取消し・結果不明・data所有を照合し、Qwen承認範囲の指摘を修正した。修正後の独立再確認はV00に残す。
- [x] 新規proposed APIやcrateの例を動作確認済みと案内していない。

## D03 Markdownの境界

範囲: parse、egui renderer、テーマ/フォント、リンク操作とアプリharnessの責任を分ける。規模S、難度中。所有者親、model/effort `inherit/inherit`。

- [x] 現行の構文・依存・WordWeave結合と目標の公開面を記録した。
- [x] renderer/UI/リンク操作の責任、未対応構文・互換性条件を記録した。
- [x] Rust/eguiノート支援hostを第二実製品候補として追記し、独立adapter・同一版・正常/境界/異常の将来観測条件を記録した。修正後の独立再確認はV00に残す。

検証: AC01–AC06、`src/app/notifications/markdown.rs` と親のソース根拠との照合。依存: S00とD00共通境界。変更ファイル: `docs/reuse/markdown.md` 1件。

## D04 チャットの境界

範囲: 汎用会話/媒体/stream状態と教材・学習記録・永続化adapterを分ける。規模S、難度/リスク高。所有者D01/D02と同じdesigner、`gpt-6-astra/high`。

- [x] WordWeave型や教材生成/登録がcoreに流入しない責任境界を記録した。
- [x] 同意・提案/可視差分/承認、保存失敗、取消し/結果不明、媒体原本の扱いを記録した。
- [x] ノート支援hostを第二候補として、同一版・旧保存形式・段階移行を判定する条件を記録した。

検証: AC01–AC06、`src/chat.rs`、`src/chat_action.rs` と親のソース根拠との照合。依存: S00・D00・D02契約。変更ファイル: `docs/reuse/chat.md` 1件。

## Checkpoint C 設計の精緻化

- [x] S00の正式AC→D00–D04節への追跡表をresults.mdで確認した。
- [x] 計画担当が実際の設計・未決事項に合わせplan/todoをv1へ更新した。

## V00 文書検証・独立レビュー・handoff

範囲: 固定した文書差分を検証して引き渡す。規模S、難度/リスク高。親が文書検証を担当し `inherit/inherit`。独立reviewer `gpt-6-astra/high` はread-onlyである。

初回の直接検査は10文書・91相対リンク・34表で問題0、diff check exit 0、ソース98件とルートCargo2件のSHA256は開始値と一致した。独立設計レビューのP2指摘2件は所有者が修正済みである。この証拠は修正前の検査/初回レビューであり、最終合格ではない。詳細は [results.md](results.md) を正とする。

- [x] 初回の文書検査・非変更確認・AC追跡・独立レビューを記録し、レビュー指摘D01/D02の修正を反映した。
- [x] 修正済み仕様・6設計文書・plan/todoの独立再レビューと文書検査を実施し、D01/D02解消・AC01–AC07の文書受入PASSを記録した。
- [x] 終了計測の取得範囲/欠測、最終結果・未決・未実装・未実行を記録し、設計書一式を引き渡す。

検証: `git diff --check -- tasks/REUSE-DESIGN-001 docs/reuse docs/decisions/0002-reusable-library-boundaries.md`、直接ファイル読取りのリンク/Markdown確認、開始statusとの差分分類。依存: S00・D00–D04。親の変更ファイル: `tasks/REUSE-DESIGN-001/results.md`、必要な `tasks/current.md`、KPI記録。コード試験・release build・実APIは非該当であり、実行済みにしない。

## 工程非該当と最終境界

- 本体実装担当: 非該当。model/effortの記録は `inherit/inherit`、起動なし。将来の実装は新しい計画/割当を必要とする。
- 本体テスト作成/実施担当: 非該当。文書検証はspec ACを期待値として親に集約し `inherit/inherit`、専用agentは起動しない。
- [x] 文書の受入と将来製品の実装/二消費者実行受入をplan/spec/設計で区別した。最終報告はV00に残す。
- [x] 初回非変更確認で元の未完計画・他者の変更を保持し、教材・学習記録・音声を使用/変更していない。最終検査はV00に残す。
- [x] 本精緻化までcommit/push/公開・モデル/設定/品質ゲート変更を行っていない。

本handoff以降、plan/todoの最終レビュー・検査・計測・報告の状態/チェック更新のみ親へ所有を移譲する。plannerは追加入力なしに編集を続けない。将来API・製品実装・実行確認は今回の完了対象へ加えない。
