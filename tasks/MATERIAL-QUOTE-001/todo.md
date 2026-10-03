# MATERIAL-QUOTE-001 作業リスト

正本: [plan.md](plan.md)。U5までの実装・自動検証・release配置と親の結果統合を完了した。U5の隔離native描画と独立レビューは確認済み。利用者の実データでの画面操作・実AIは別受入であり、以前の未完了項目も保持する。

## T1: 拒否条件と外部仕様を確定する

- [x] 規約・7役契約・計画Skill/KPI手順を読み、暫定計画とモデル割当を作成する。
- [x] 親の限定実ログ調査で、4往復目assistant引用のMarkdown記号省略による8件の不一致を確認した。実ログ本文の複写はしない。
- [x] 外部仕様担当がMQ-AC-01～10、番号/話者/引用/固定元発言、240文字preview/省略境界をspec.mdへ定義した。
- [x] 独立仕様レビューpass、要対応欠陥なし。親は利用者要求の範囲とD1/D2の承認充足を確認し、計画者へ返した。

受入: 原発言欠落と引用非包含等の違いが明確であり、厳密照合/非登録を維持する仕様である。検証: ACと診断条件の対応を独立レビューする。依存: 親の限定診断。規模S、所有: planner=plan/todo、spec-author=spec、reviewer=read-only。

## T2: 設計と実装単位を確定する

- [x] 設計担当が選択snapshotと番号/話者、共通helper二入口、表示/異常時、互換性をdesign.mdへ記録した。
- [x] 独立設計レビューpass、重大/要対応欠陥なし。親が設計を承認した。
- [x] 計画者がU1/U2・AC・focused filter・最終所有を更新した。追加テスト名はtest authorがtests.mdへ記録する。
- [x] CP1: 親がテスト作成→実装のdispatchを決定した。各役の起動直前にモデル/effort・Skill可用性を照合する。

受入: 仕様を満たし照合を緩和せず、担当ファイルの競合がない。検証: 仕様追跡と設計レビュー。依存: T1。規模S、所有: designer=design、planner=plan/todo、reviewer=read-only。

## T3: U1 引用拒否の説明を改善する

- [x] 独立合成回帰を作成し、materialは16件中11成功・新規5失敗のREDを記録した。通知試験は実装差分存在後の成功であり、実装前REDとは扱わない。詳細はtests.md。
- [x] 実装担当が割当本体を変更し、引用拒否の説明と生成指示を改善した。親から実装凍結の報告を受領した。
- [x] focused green: material 16/16、通知対象1/1成功。厳密照合の拒否/正常受理保持を確認済みとの親報告を受領した。

受入: 原会話の参照先と拒否引用が特定でき、生成指示が原文の記号/空白/改行保持を明示し、誤受理を増やさず教材を登録しない。検証: `cargo test --locked material::tests`、`cargo test --locked notifications::tests`、生成応答/保存済みDraftのred/greenと境界証拠、prompt静的差分確認。依存: T2/CP1。規模S～M、実装所有: src/material.rsのhelper/二入口本体とsrc/ai.rsのprompt。テスト所有: src/material.rs・src/app/notifications.rsの対象cfg(test)とtests.md。同一ファイルは順番に編集する。既存保存/承認/通知本体と新規loggingは変更対象外である。

## T4: U2 教材化経路を検証する

- [x] 固定差分の合成回帰で改善メッセージ・拒否時不変・既存承認境界を確認した。実画面操作の受入は次項に分離する。
- [ ] 標準/狭画面・拡大表示で通知の可読性とスクロール/コピー到達性を確認する。通知本体に追加修正が必要なら設計/所有の更新を親へ返す。
- [x] `cargo test --all-targets --locked` を新規実行し、298件成功/0失敗・exit 0をresults.mdへ記録した。
- [x] `cargo build --release --locked --bin wordweave5` を新規実行し、exit 0・更新EXEを確認した。
- [x] CP2の検証範囲表示: 合成/headless試験と実機UI・実AIの未実施を分離した。全体検証の結果確定は上のチェックに残す。

results.mdと実行ログのexitマーカー、EXE更新、レビュー時から不変のsource hashを親と独立レビュー担当が確認した。起動ラッパーの完了と内部cargo終了を取り違えた点は是正し、成功済み検証の不要な再実行はしなかった。

受入: 必須全体検証が成功し、証拠で確認できない範囲が明記される。検証: 上記コマンド・隔離描画/状態比較。依存: T3。規模S、所有: runner=results/ログ/隔離出力のみ。ソース/期待値を変更しない。

## T5: 独立レビューと親への引継ぎ

- [x] 独立静的レビューが差分・AC・focused証拠をread-onlyで確認し、指摘なし。判定はPending verificationであり全体検証合格の代替ではない。
- [x] 現時点の要修正レビュー指摘はない。今後の検証で新たな欠陥が出た場合だけ担当へ戻す。
- [x] CP3: 変更/確定診断/検証結果と、実AI再試行・実機受入の未確認を最終報告へ統合した。
- [x] 親がtasks/current.mdとKPI測定の短い状態を記録した。既存計画と未コミット7役設定を維持した。

受入: 未解決重大指摘がなく、残る未確認事項を隠さない。検証: AC closureと所有範囲/差分確認。依存: T4。規模S、所有: reviewer=read-only、parent=統合/状態/KPI。commit/push/公開は含まない。

## T6: U3 変更理由の出力形式を揃える（承認済み追補）

- [x] 利用者承認済みformat-firstの範囲を計画へ追記した。全引用は有効、理由5～7の要素全体pathが拒否されたという親の診断を受領した。U1/U2と未完了実機受入を保持した。
- [x] spec authorがMQ-F-01～04、既存許可集合/添字/番号/path診断だけをspec.mdへ追補し、限定独立レビューを通す。
- [x] designerが親のschema/protocol調査からschema/prompt/runtime契約・keyword適合・送信試験入口をdesign.mdへ追補し、限定独立レビューを通す。ここでテスト所有を確定する。
- [x] CP4: 承認済みformat-firstの範囲と厳密検証維持を親が照合し、test author `gpt-6-sol/high` → implementer `gpt-6-sol/high` を逐次dispatchする。
- [x] 独立合成テストで要素全体拒否/正常末端パス、理由番号/path/原因、保存済み案、送信outputSchemaをredにし、実装後greenを確認する。正確な試験名とAC対応をtests.mdへ追記する。
- [x] runtimeの既存許可集合を広げず、promptとoutbound schemaを整合し、二入口の具体的拒否説明を実装する。
- [x] runnerが新しい固定差分のfocused成功を受領し、pattern表・`cargo test --all-targets --locked`・`cargo build --release --locked --bin wordweave5` を逐次実行、exit/305件成功/成果物と未検証範囲をresults.mdへ追記した。ソース不変のfocusedは再実行していない。
- [x] CP5: 独立レビューがMQ-F closureと既存引用/承認の回帰を確認し、親が結果・実AI/実機未実施・KPIを統合した。commit/pushなし。

受入: MQ-F-01～04。依存: 既存承認済みspec/designの再利用 + 本追補の限定レビュー。規模M、本体所有はmaterial.rsのpath/schema/二入口診断とai.rsの教材prompt/schema呼出のみ。テスト所有はmaterial.rs cfg(test)、codex_process.rsとsupport/mock_codex.rsの教材送信契約試験、tests.md追補。モデル/選定理由/リスクはplanのU3表による。whole-item受理、引用正規化、自動修復/再登録/再試行、auth/実データ変更は対象外である。

## T7: U4 再取得結果に即した通知

- [x] 暫定計画・MQ-R-01～03・7役割当を追記し、U1～U3と未完了実機受入を保持した。
- [x] spec authorがMQ-R-01～05として6状態×本文有無/空白と通知文・種別を追補し、限定独立レビューpass・親承認を受領した。
- [x] designerが既存通知API/マーカー消去/refresh優先順序と試験入口を追補し、限定独立レビューpass・親承認を受領した。
- [x] CP6: 計画者が確定仕様MQ-R-01～05へ同期し、所有・モデル割当を維持した。親が承認充足を確認し、test authorのRED作成とimplementerのread-only準備へdispatchした。
- [x] test authorが合成Recovered→tickの独立REDとAC対応をtests.mdへ記録し、implementerがRecovered本体を修正して通知13件GREENを確認した。
- [x] runnerが固定差分でnotifications focused証拠・all-targets308件成功・release exit0の新規結果を記録し、実AI/実機未実施を明記した。
- [x] CP7: 独立最終レビューpass後、親が結果・残る利用者再試行・current/KPIを統合した。commit/pushなし。

受入: 確定仕様MQ-R-01～05。依存: 既存通知/実行記録設計と本追補の仕様→設計→RED→GREEN→検証→レビュー。規模S、所有・モデルはplanのU4表による。必須検証は `cargo test --locked notifications::tests`、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` と非反映/通知分類の独立確認である。timeout/process/model、実データ、回復処理、自動生成/登録は変更しない。

## T8: U5 仕様・設計を限定する

- [x] 承認済みUI方針と親の調査を受領し、暫定要求MQ-UI-01～04、U5a/U5b、7役のmodel/effort・所有をplanへ追記した。未完了項目を保持した。
- [x] spec authorが既存spec.mdへMQ-UI-01～08を追補し、限定独立レビューpass・親承認を受領した。
- [x] designerが既存design.mdへ選択寿命・枠/固定操作・responsive・理由/引用・AI向けlabel分離・隔離試験入口を追補し、限定独立レビューpass・親承認を受領した。
- [x] CP8: material_ui_review（Sol/high）の仕様/設計両passを親から受領し、plannerがAC/所有/試験入口を確定した。親承認済み、独立RED→U5a→U5bを逐次dispatchする。

受入: 承認済みUI要求に過不足がなく、登録/AI/永続化を変えない契約と試験入口が決まる。検証: 仕様追跡・限定独立レビュー。依存: U1～U4の既存契約と親preview調査。規模S、所有はspec/design/plan/todoの各U5節のみ、モデルはplanのU5表による。

## T9: U5a 比較とモード説明を読み取れるようにする

- [x] test author `gpt-6-sol/high` が両単位の独立REDを作成した。U5aの初期選択/モード表示、U5bの折畳み表示で失敗を確認。後続の狭幅・旧案・案切替・登録禁止理由は追補回帰であり、初回REDとは区別してtests.mdに記録した。
- [x] implementerがplan所有の比較UI本体を変更した。Mode::labelのAI向け既存契約は維持しUI専用入口を用いた。選択/モード/旧案/案切替のfocused成功、狭幅の本文到達・長い登録禁止理由・全窓矩形検査成功を確認した。

受入: MQ-UI-01～05,07～08。検証: harness_tests中心のfocused、material::tests、標準1150×950/0.8と狭幅820×650/1.6の実窓header/footer・合成描画、既存serde/AI向けlabel・承認/学習状態回帰。依存: T8/CP8と両単位の独立RED。規模M、実装 `gpt-6-sol/high` の本体所有はmaterial_review.rs/chat_ui.rs/app.rs比較UI節とmaterial.rsのUI表示入口。app.rsのMaterial受領/登録/破棄/restoreは表示一時状態消去のみ、dirty/保存不変。テスト所有はharness_tests.rs/tests.md/visual_check.rs合成fixture中心と必要なcfg(test)である。

## T10: U5b 詳細で言い換えと条件を表示する

- [x] test authorがT9と同時に複数replacementsの初期可視性と語調情報保持のREDを記録した。長文/狭幅のスクロール到達は後続native描画、空集合の分岐は静的確認であり、全パターンを実装前REDにしたとは扱わない。
- [x] implementerがmaterials_ui.rsの詳細表示を変更し、focused 1/1 GREENを確認した。標準・狭幅とスクロール後の合成native描画も確認した。

受入: MQ-UI-06。検証: tests.mdのfocusedと合成描画、教材/学習状態不変。依存: T8/CP8、独立REDはT9と一括、本体はT9 GREEN後。規模S、実装 `gpt-6-sol/medium` の本体所有はmaterials_ui.rs詳細節、テスト所有はharness_tests.rs/tests.md/visual_check.rs合成fixture中心と必要なcfg(test)である。

## T11: U5 固定差分の受入を確認する

- [x] CP9: 標準/狭幅/拡大の隔離native描画で、固定見出し/登録操作・前後配置・バー・言い換え到達を親/独立reviewerが確認した。引用foldの実クリックと新案切替はheadless回帰で確認し、native展開操作と区別してresults.mdへ記録した。
- [x] 固定差分で `cargo test --all-targets --locked` 315件成功/exit0、`cargo build --release --locked --bin wordweave5` exit0を確認した。EXE時刻/サイズ/SHAはresults.mdに記録した。
- [x] 独立レビューの局所指摘を解消し、保存/承認/AI契約不変のレビュー結果と最終検証を親が統合した。current/KPIと利用者の実教材・実AI受入の未確認を記録。commit/pushなし。

受入: MQ-UI-01～08と全体検証成功、重大な未解決指摘なし、実機未確認を明示する。検証: focused証拠・新規ログ・描画・限定独立レビュー。狭幅820×650/zoom 1.6でも実窓header/footer固定を必須とし、360論理pt下限は実測調整する。依存: T9/T10。規模S、runnerはresults/ログ/隔離出力のみ、reviewerはread-only、parentは状態/KPIのみ。既存未完了の受入を自動で完了にしない。
