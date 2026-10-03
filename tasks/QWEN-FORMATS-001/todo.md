# QWEN-FORMATS-001 作業リスト

2026-10-03。状態: U0～U5の開発範囲完了、独立レビューP1/P2なし、両release更新済み。実機/実API未確認はresultsに明記した。詳細は [plan.md](plan.md)。旧計画の未完項目を保持し、旧U4はU3へ統合した。

## U0 要求・設計差分を確定する（文書各1件 / 中・高risk）

- [x] AGENTS/development-team/指定計画SkillとDoD/KPIを読み、既存dirtyと旧未完項目を確認する。
- [x] 暫定計画・依存・7役責任・5役model/effort割当を作成する。
- [x] 親が暫定計画を確認し、外部仕様作者 `gpt-6-astra/high` へdispatchする。
- [x] 外部仕様 `spec.md` でQF-01～07、QF-AC-001～009、codec/container表・公式根拠・本体UI差分を確定する。
- [x] 独立仕様確認と既承認要求の確認を完了する（親の承認・検証完了報告による）。
- [x] 設計 `design.md` でbackend探索/不足、原bytes/再生、上限/timeout/protocol/seek、非同期資源寿命を確定する。
- [x] 独立設計検証とshutdown後警告の親採用後、plan/todoのAC・所有・検証を精緻化する。
- [x] 親が精緻化計画を確認し、指定model/effort/Skillを照合して実装dispatchする（CP0）。仕様/設計の古いレビュー待ち表示とレビュー記録は各所有者が整合する。

依存: なし。受入: codec等の未決が解消、要求追跡可能、親が実装dispatch可能。検証: 仕様/設計独立レビュー・リンク/所有/モデル対応確認。対象: plan/todo/spec/design、作者別所有。

## U1 共通libの多形式検査とwire（小単位分割 / 高・高risk）

- [x] テスト作者が6形式/代表codec・境界・破損・偽装・backend異常の独立fixture/期待値を作成する。
- [x] U1a: backend helper Astra/highがaudio/error/libの確定公開契約を実装する（3ファイル）。
- [x] U1b: 同helperが必要な新規backend/parserを実装し、全復号・終端/属性検査・外部参照拒否、bounded process・cancel/cleanup/shutdownのfocused testsを通す（新規最大3ファイル）。
- [x] U1c: 同helperがproviderのwireを検証済み実体へ変更し、全6形式の原bytes一致をmockで確認する（1ファイル）。
- [x] 原bytes不変・再生用PCM分離、境界/異常入力の送信0、core-only、削除失敗flagを確認する。

依存: CP0・仕様由来の独立期待値。受入: QF-AC-001～005/009。検証: focused audio/backend/provider tests、core-only check/test、codec別mock bytes/wire一致。対象: planのU1a/b/c所有、package tests/fixturesは親Astra/highが別著者として作成する。全小単位の5役割当はplanのU1行を適用する。

## U2 単独側で多形式の評価を準備する（2 production files / 中・高risk）

- [x] テスト作者がwire形式/原bytes一致・再確認・MCP境界ケースを固定する。
- [x] standalone helper gpt-6.1-sol/highがgui.rs/main.rsへAudioLoadJob・6形式選択・失効/取消し/終了回収を適用する。session.rsは変更しない。
- [x] shutdown後にcleanup warningを回収し秘密/pathなしrfd native警告を表示する。既存単独説明・MCP境界を維持する。
- [x] 同一fixtureで全6形式の準備→mock送信を確認し、共通lib/単独側checkpointを通す（CP1）。

依存: CP0契約からU1/U3と並行着手可、統合完了はU1。受入: QF-AC-001～005/007～009。検証: package focused tests、integration_reuse、core-only check。対象: tools/qwen-audio/src/gui.rs/main.rs、package testsは別著者の親。

## U3 本体の多形式と指定UI（旧U4統合、3 production files / 高・高risk）

- [x] テスト作者が失敗後旧音声送信0、snapshot、再生/停止/閉じる/取消しの期待値を作成する。
- [x] 親Astra/highがcontroller/UIへ選択・復号再生・形式表示、root mainへshutdown後cleanup warning/native警告を反映する。
- [x] 同fixtureで本体mock送信bytes一致、入力失効、資源回収を確認する。

- [x] 本体確認欄から指定3説明を削除し、指定注記を冒頭説明と読む例文の間へ移す。
- [x] 独立UI検証で指定文一致/一度だけの表示/順序/単独GUI非変更を確認する。
- [x] 通常/狭幅/拡大のnative画面で確認・送信・閉じるへ到達できる証拠を残す（CP2）。

依存: CP0契約からU1/U2と並行着手可、統合完了はU1。受入: QF-AC-001～009。検証: root integration/UI tests、native隔離音声再生・画像・終了警告。対象: src/qwen_reading.rs、src/app/qwen_reading_ui.rs、src/main.rs。親作成のU3テストは実装との著者独立性がないためAstra/high reviewerが期待値も独立確認する。旧U4の5役model/effortは全てU3行へ統合し、実装はAstra/highである。

## U5 固定差分を検証し受け渡す（production編集なし / 中・高risk）

- [x] runner gpt-6.1-sol/mediumが固定差分識別・環境/依存情報を記録し、残り必須コマンドを実行する。本体418件は今回の親実行ログを独立照合、最終releaseは再実行済みである。
- [x] feature tree、変更範囲format/check、backend異常/不足、native画面の証拠と再生等の実機未確認を区別して記録する。
- [x] 独立reviewer Astra/highがコード・期待値・証拠を確認する。指摘を元担当へ戻し、影響検証を再実行する。
- [x] 全QFの受入・未検証を親が確認し、results/current/KPI/利用説明を統合する（CP3）。
- [x] 実API未送信、実機残件、既存未完事項を区別して受渡し報告へ記載する。公開しない。

依存: U1～3、CP1/CP2。受入: QF-07、全QF-AC追跡・重大指摘なし。検証: root all-targets/release、単独all-targets/release、core-only、独立レビュー、native証拠。対象: results/logs/artifactsはrunner、統合docsは親。期待値/本体は変更しない。Cargo.toml/lockは親だけが編集する。2補助の並行理由と全5役model/effortはplanに記録済みである。
