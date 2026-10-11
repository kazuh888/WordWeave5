# 音響解析・音読履歴の実装計画

2026-10-04 / ACOUSTIC-HISTORY-001 / 実装中。I0の合成音検証で必要性が判明した子メモリ上限512MiBは利用者承認済み。同時1件・処理上限60秒は維持する。設計段階の履歴は [plan.md](plan.md) に保持する。

## 承認・完了境界

親から、利用者が親モデル/effortを変更し「実施してください」と承認したとの引継ぎを受領した。[仕様](spec.md) のAH-AC01–18と [設計](design.md) に基づき、原音・測定・Qwen助言の履歴、比較、二host、ごみ箱/復元まで実装する。完全削除は保留する。親モデルを計画者が変更しない。

完了は承認範囲の実装、独立回帰試験、保存/復旧の独立レビュー、Windowsの全体test/release buildと結果記録である。実録音、実API送信、利用者の学習データ、commit/push/公開は含まない。合成fixtureの成功を音声一般の精度・実機受入の完了へ読み替えない。Praat固定版、配布条件、資源制約が未確認ならそのgateは未完了のままとする。

再開時基準はHEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。前工程の文書変更がdirty、本体コードはcleanであることを確認した。全変更を保持する。KPI規約は再読済み。plannerのruntime metadata・使用量開始/終了値は未取得であり、親が取得可能な範囲と欠測を既存測定記録へ残す。

## I0: 先に固定する境界

設計は承認済みだが、実装可能性と配布適合性は別の証拠を要する。designerは既存設計を再作成せず、次だけを追補する。

- Praatの公式固定版、取得元、binary/script hash、CLIの実引数、Windows非表示起動、日本語path、取消し/timeout/kill+wait、Job Object資源制約を合成音で確認する。確認前の同梱/自動ダウンロードは実装しない。GPL-3.0-or-laterの配布条件と本体MITの扱いを公式資料で調べ、外部processだけで適合と決めない。
- 独立RMS/区間oracle、F0の既知周波数と倍/半周波数誤りの判定、採用する誤差・資源予算を試験前に明文化する。期待値を実装出力へ合わせない。仮値の2GiB/60秒/256MiB等は測定した範囲を記録し、変更が必要なら理由と契約差分を戻す。
- 共通解析crate、hostのTrial/Run DTO、保存adapter、二hostの依存方向を固定する。単独ツールがWordApp/Entry/Progressを要求せず利用できること、共通crateを同じ版で使うことを確認する。論理履歴契約を共有する配置を確定し、実装者が別々の未調整schemaを作らない。

I0がPraat取得・license・OS資源制御等で止まった場合、独立した保存やRust解析は進められるが、F0/配布gateを合格にせずblockerと確認済み範囲を親へ返す。別engineへの無断fallback、上限・レビューの緩和をしない。

## 依存順と所有

`I0 → (I1 → I2 → I3) + (I4 → I5) → C1 → I6 → (I7 → I8) → I9 → C2 → I10`。I7の純粋比較はI1のDTO確定後に着手できるが、通常補助1役とし、常駐役を増やさない。各単位は1回のbounded handoffとし、5ファイルを超えそうなら着手前に親からplannerへ分割を返す。

| 単位 | 内容・受入条件（最大3点） | 本体所有候補・依存 | 必須focused検証 |
| --- | --- | --- | --- |
| I0 | 上記の固定版/配布/資源/契約の証拠を残す。未測定を区別する。schema/解析API/二host seamを合意する | designer: `design.md` の実装追補、親: `evidence.md`。spike実装は独立した `tools/acoustic-analysis` の割当範囲だけ | 合成PCM、固定Praat、Unicode path、子process回収、version/hash、資源/出力上限。配布判断の根拠 |
| I1 | 原音→TrialRevision→HistoryRootを不変保存し再読できる。保存再試行は同一操作で重複なし。quota/欠損/未知版を成功にしない | `src/audio_history.rs`、`src/audio_history/store.rs`、`src/audio_history/recovery.rs`、`src/lib.rs`。I0後 | AC02–04/08/14、原音hash、metadata各publish前後fault、独立サイズledger |
| I2 | Progress v1から既存fieldsを保ってv2へ移行する。全保存/教材commit/recoverがrootを保持・検証する。中断時は旧状態か新状態全体であり部分確定なし | `src/store.rs`、`src/commit.rs`、I1の保存窓口。I1後、同ファイル同時編集禁止 | AC04/08–09/16、通常save/二file commit/recover全経路、未知版、intent前後強制終了、局所破損閲覧 |
| I3 | backup v2が全履歴版/trashを含む。v1復元は現履歴保持、v2は履歴だけmergeし衝突全拒否。差分確認前適用/復元から送信は0回 | `src/backup.rs`、`src/app/backup_ui.rs`、必要な履歴export adapter 1ファイル。I2後 | AC15–16、v1/v2往復、同ID一致/競合、全closure検証、disk/中断、既存deck復元意味の回帰 |
| I4 | 共通crateでprofile付きRMS/全長/低エネルギーを計算する。DigitalSilence/欠測/品質を保持する。不正・非有限・上限入力を拒否する | 新 `tools/acoustic-analysis/src/{lib,profile,measure}.rs` 等最大4本。I0後 | AC03/05–06/10/18、独立式、閾値一致・端数frame・stereo反相・clip・0/上限、roundtrip |
| I5 | 固定Praat adapterとsupervisorを実装する。欠落/失敗はF0欠測のPartial、全体取消し/timeout/資源超過はreportなし。子process/作業領域の寿命と警告を守る | 新crateの `{praat,process}.rs` と固定script等最大4本。I4/I0後 | AC01/05–06/09–10、合成F0、悪性数値/時刻/出力、fake child fault、実Praatの上限・停止回収 |
| I6 | Savedのみ開始しrun intentを先保存する。ローカル/Qwen結果を同試行へ保存し保存再試行で再実行0回。close/取消し/trash/終了と遅延完了を区別する | `src/qwen_reading.rs`、新 `src/audio_history/jobs.rs`、同controller補助最大2本。I1–I5/C1後 | AC01/07–10、mock POST数、同意失効、各job ID/generation、close→完了→再表示、送信後不明、secret sentinel |
| I7 | CohortKey・本人/日時・読上げ一致で既定系列を選ぶ。未知/競合/欠測を理由付きで分ける。時刻/重複/再解析で回数を捏造しない | `src/audio_history/compare.rs`、共有DTOへの必要追補。I1/I0後 | AC11–13、各条件を1つ変える手書き集合oracle、AC11-a–d、0/1/複数点、同時刻・異時差 |
| I8 | 本体で保存/解析/毎送信・履歴詳細・単位別グラフ/表/再生を操作できる。import本人/日時/条件と振返り・ごみ箱/復元/容量管理を提供する。日本語/狭幅/keyboardの到達性を保つ | `src/app/audio_history_ui.rs`、`src/app/qwen_reading_ui.rs`、`src/app.rs`、必要補助1本。I6/I7後 | AC01–02/06/08/11–15/17、合成native UI、値と表一致、文言/導線/終了確認、既存Qwen操作回帰 |
| I9 | 単独ツールが同じ解析crate/profileと論理履歴契約を利用する。独立root/資格情報で保存/再起動/比較/trash/backup復元を行う。他hostのbytes/目標/秘密を変更しない | `tools/qwen-audio/src/{gui,history,history_ui}.rs`、`lib.rs` 等最大5本。I6–I8の契約確定後 | AC18と対応AC01–16の二host適合ケース。core-only依存、同fixture結果、別root ledger、mock送信、単独UI |
| I10 | 固定した差分に全体検証を実行する。独立レビュー指摘を解消・必要な検証を再実行する。実機/API/精度の残りと生成物を記録する | runner: logs/resultsのみ。親: docs/usage/current。I1–I9/C2後 | 下記全体gate、AC追跡表、対象SHA/dirty差分、実行数、release EXE hash、未検証一覧 |

I9のbackup等を5ファイルへ無理に詰め込まず、I0で単独保存adapterが大きいと分かればI9a保存/復元とI9b UIに分ける。同じモデル指定と独立検証を継承する。I8も巨大な条件分岐ファイルにせず、所有追加が必要なら親が明示分割する。

実装接合時の分割：I1はI1a共有DTO→I1b-1 reader/閉包検証→I1b-2 publish/recoveryとして逐次引き渡す。[reader契約](design-i1b.md)に所有・API・受入を固定した。I5はI5a protocol/WAV→I5b Windows supervisor→I5c pinned adapterとする。[I5追補](design-i5.md)の4 Rust files＋固定scriptを単位ごとに引き渡す。担当モデルとgateは元のI1/I5を継承し、追加の並行枠は使わない。

Cargo.toml/Cargo.lock、各crateのmanifest/lock、export接続は**統合担当の実装役だけが逐次所有**する。統合実装役指定は `gpt-6-astra` / `high`。並行担当へmanifest所有を渡さない。lock更新は依存導入時だけ行い、検証時は `--locked` を外さない。新しい共有履歴crate等が必要ならI0で配置と対象manifestを確定して本計画へ短く記録する。

## 各単位のモデル割当

以下の略号は正確な起動値を指す。`A/H` = `gpt-6-astra` / `high`、`S/H` = `gpt-6.1-sol` / `high`、`S/M` = `gpt-6.1-sol` / `medium`。いずれも現在呼出可能一覧と対応effort内である。親が起動時に正確なIDへ展開し、指定値と返却metadataを別記録する。未取得値を推測しない。

| 単位 | 難易度/危険と選定理由 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- | --- |
| I0 | 高/高: 依存配布とprocess制御の不確実性を早期解消 | A/H | A/H（spikeのみ） | A/H | S/M | A/H |
| I1 | 高/高: 原音損失、再試行、quota、回復 | A/H | A/H | A/H | S/M | A/H |
| I2 | 高/最高: Progress移行・全writer・教材commitの既存データ保護 | A/H | A/H | A/H | S/M | A/H |
| I3 | 高/最高: 複数file復元・旧形式・衝突での全体整合 | A/H | A/H | A/H | S/M | A/H |
| I4 | 中/中: 確定済み算式とbounded純粋関数、独立oracleを作れる | A/H（契約変更時） | S/H | S/H | S/M | A/H |
| I5 | 高/高: 子process・取消し・資源制限・外部出力 | A/H | A/H | A/H | S/M | A/H |
| I6 | 高/高: 保存と課金通信の状態交差・遅延完了 | A/H | A/H | A/H | S/M | A/H |
| I7 | 中/高: 純粋比較だが誤系列は学習解釈を誤らせる | A/H（契約変更時） | S/H | S/H | S/M | A/H |
| I8 | 中～高/高: UIの範囲を分割し状態判定をcontrollerへ委譲 | A/H | S/H | A/H（状態受入） | S/M | A/H |
| I9 | 高/高: 二hostの独立保存・秘密境界、重複実装を防ぐ | A/H | A/H（adapter）、S/H（限定UI） | A/H | S/M | A/H |
| I10 | 中/高: 対象固定と証拠の完全性。実装変更が出たら元単位へ返す | A/H（必要時） | A/H（必要時、元単位を優先） | A/H（不足AC） | S/M | A/H |

plannerは `gpt-6-astra` / `high` 固定。external-spec authorも `gpt-6-astra` / `high` 固定である。既存仕様と設計レビューを再利用するため新規仕様執筆工程はN/A、承認状態の追記は親の統合記録で足りる。observableな仕様変更が必要になった場合はspec authorへ戻し、独立レビューを受ける。design担当はI0追補と未解決契約のみを担当し、実装コードを所有しない。

モデル選択は実測性能の保証ではなく、失敗時損失・状態数・oracleの難しさに応じた責任配分である。役・model・effort・必要skillの不在はblockerであり黙って代替しない。実行枠不足は完了待ち後の再試行で扱い、親モデル変更や担当の黙示継承で解消しない。

## テスト所有・独立性・並行化

各単位のtest authorは先に [tests.md](tests.md) のAC副ケースから合成fixtureと期待値を作る。候補所有は `tests/audio_history_*.rs`、`tests/qwen_reading.rs`、`src/app/qwen_reading_tests.rs`、新crateの `tests/*.rs`、単独hostの新テストファイルである。親が実在するtarget/test名と範囲をhandoffに固定する。`cfg(test)` と本体が同一ファイルなら作者→実装者の順で所有を切り替え、同時に編集しない。compile失敗・0件実行を行動REDとしない。

通常は補助1役ずつとする。I0で契約固定後だけ、独立解析crate（I4/I5）とhost保存（I1–I3）を最大2補助で並行できる。依存がPCM→reportと版付き保存契約に限定され、所有ファイルが交差せず、親にmanifest統合・独立ACの準備という有用な作業があるためである。I2/I3/I6間は共有状態を編集するため直列とする。親だけがdispatchし、子の再委任は禁止する。

reviewerはread-onlyで、実装/期待値の作者を兼ねない。runnerはsource/期待値を変更せず、ログと隔離生成物のみを所有する。親は必要skillを役割範囲で指定し、実装は `wordweave-change`、UIは追加 `wordweave-egui-ui`、試験作成は `agent-skills:test-driven-development`、reviewは `agent-skills:code-review-and-quality` と保存/認証の該当専門skillを読む。計画担当は本計画とplan.md以外を編集しない。

## チェックポイント・検証

- [ ] I0: 固定engineと配布/資源の証拠、共有契約、oracleを確定する。
- [ ] I1–I3: 原音→Progress v2→backup v2の再読・復旧経路を完成する。
- [ ] I4–I5: Rust測定と固定Praatの合成入力検証を完成する。
- [ ] C1: 保存/移行/backupとprocess境界を独立レビューする。原音損失・部分復元・上限・取消しの阻止指摘を解消してからUIへ提供する。
- [ ] I6–I8: 保存済み試行から解析/Qwen/比較/ごみ箱まで本体の縦断操作を完成する。
- [ ] I9: 単独ツールで同一版を利用し、二host適合を確認する。
- [ ] C2: AH-AC01–18の正常/負例と実装diffを独立レビューする。全体試験前に対象差分を固定する。
- [ ] I10: 必須全体検証とrelease、未検証事項、利用量・引継ぎを記録する。

必須検証コマンド（新crateのmanifestは作成後に存在確認する）：

```text
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked
cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio
cargo fmt --all -- --check
git diff --check
```

本体はworkspaceではないため、root test成功だけで各独立crateのテストを実行したと扱わない。新crateと単独crateのfmtもmanifestを指定して確認する。focused testは各handoffで実在target/caseを固定し、正常・障害の実行数をログへ残す。長いログはtask固有ログへ保存し、報告は件数と失敗だけに絞る。合格済み検査の再実行は関連変更/失敗/未解決リスクがある場合だけとする。

2026-10-07の利用者承認により、Windowsのfile symlink作成権限を要する`windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target`だけは通常suiteから明示別実行へ分離する。ignore理由を表示し、通常gateのignored件数に隠さない。対象を含む検証時は非管理者でbuild後、hashを固定した`audio_history_closure-*.exe`へ`--ignored --exact windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target`を渡し、権限のある環境でこの1件を別実行する。未実行・権限不足を成功と数えない。必要な昇格はこの1件だけであり、管理者でcargo/buildや全suiteを動かす承認ではない。Windows設定の変更は行わない。

Windows合成native UIとprocess検証は許可範囲、利用者データ・実録音・実Qwen送信は今回実施しない。IME/支援技術/実マイク・再生機器/話者別精度が未確認なら個別に残す。全体gate通過がこの未測定を消すことはない。

## 次のhandoff

### I2実装接合追補（2026-10-08、親のdispatch確定用）

この追補はI2の暫定分割を、承認済み外部契約と [design-i2.md](design-i2.md) §2–8へ照合して具体化したものである。設計入力SHA-256は `B8B1FD5FEA1518C4AE516E0B47C9502267A0C05A5C2EF4EA4A979282E8267D51`。独立設計delta PASS、I2-D1/D2の設計上Closedを親から受領した。これは実装済み判定ではない。**I1b-2bの追加2 boundary testと独立Postは未完了であり、以下の本体/scaffold/test変更は親がその依存解消を確認した後にdispatchする。今は準備のみである。** 本節が旧I2a-1等の一括所有案を置き換えるが、I1/I3–I10、未完了チェック、全体gateは変更しない。

要求はAH-AC04/08/09/16、設計のI2-T01–08bである。原音/resultのinline、第三head、API-key fallback、UI描画/回復画面、I3 backup v2/union、I5解析、I6送信/controller、公開は対象外である。I2-T07のI6 spy受入と回復閲覧UIは後続へ残す。I3後の製品提供条件を早めない。planner所有は本節のみであり、既存 `tasks/plan.md` / `tasks/todo.md` は保持する。本節の未チェック行がI2接合の作業リストである。

#### 現物との対応と実装順

調査は対象API/呼出しの限定検索で行った。`src/audio_history.rs:97` のreaderは `&Storage` を保持し、`src/audio_history/closure.rs:118,123` の2参照がdirを使用する。`src/store.rs:82,117,385,400` はProgress v1/default/load/save、`src/commit.rs:30,55,89` はexists型pending/save/recoverである。`src/audio_history/store.rs` のprepare/publish/resume/confirm_committed/finishと `checked_old_plan` がgateとpublication検証の接合先である。行番号は調査時点であり、dispatch時はシンボルを優先する。I1b-2b Post差分でこの接合が変わった場合だけ再照合する。

依存は `I1b-2b Post → a-0 → a-G → a-M → a-S → a-N → a-R → a-T → a-V → a-P → b-A → b-B → d-H → d-S → d-B → d-M → b-S → b-C` とし、共有fileは逐次所有する。各行の終了時に独立runner/reviewerへ渡してから次行へ進む。これは元の独立Pre/Postの責任分離を細分化したもので、新しい品質基準・全体suite反復・承認gateは追加しない。各行は最大5fileである。表の本体fileにcfg(test)を置く場合は同じ1fileと数えるが、責任は作者→実装者の順に切り替える。

| 未完了単位 | 本体所有（正確なpath） / 独立test所有 / file数 | 完了条件（3点以内）・設計受入 | focused検証・難易度/危険 |
| --- | --- | --- | --- |
| [x] I2a-0 Path reader | 本体 `src/audio_history.rs`, `src/audio_history/closure.rs`。test作者は前者の専用cfg(test)と既存 `tests/audio_history_closure.rs`。3本 | `for_root(&Path, HistoryLimits)`と既存new/with_limitsが同じ閉包を検証する。公開署名/部分閲覧/上限・reparse検査を維持する。read前後の全bytes不変、Storage再取得/回復/書込なし。[独立Post PASS](history-path-review.md) | 新lib11/既存reader37成功・1明示ignored。中/高：保存前検証の共通境界である |
| [x] I2a-G metadata gate | 本体 `src/store.rs`, `src/commit.rs`, `src/audio_history/store.rs`。新test `tests/audio_history_gate.rs`、private gate試験は所有本体内cfg(test)。4本 | 単一Mutex/非公開guard・write-block状態を共有し全metadata/history書込入口を直列化する。内部はunder_guard接続で再入せず、poison/blockを無視しない。strict pendingは不明を拒否し、bool互換入口はtrueへ倒す（T04）。[独立Post PASS](history-gate-review.md) | 新24/旧store・commit17/writer37/recovery34の独立112件成功、失敗0・ignored0。高/最高：入口漏れとdeadlock。head/read/atomic_writeはgateを取得しない |
| [x] I2a-M 初回v2退避 | 本体 `src/store.rs`, `src/commit.rs`。新test `tests/audio_history_migration.rs` と両本体の必要cfg(test)。3本 | 現disk又は検証済intent由来の旧exact bytes→sha256名退避→再読を共通helperにし、同名異bytes拒否/日別退避代替不可を守る。通常saveと新教材saveにhookを準備し、target=1はgate後no-op/通常の移行用旧readなし、教材の既存旧metadata検査は維持する。v1 Default/validateのままprivate target=2・M05対照・移行2fault/rename結果不明を検証する（T01/T05、[M限定設計](history-migration-design.md)）。[独立Post PASS](history-migration-review.md) | M21＋G24＋旧store/commit17の独立62件成功・失敗0・ignored0。高/最高：実v2保存はS-M、recover hookはR-Mまで未達 |
| [x] I2a-S codec・read/head | 本体 `src/store.rs`。S作者: 新 `tests/audio_history_progress.rs` とstore専用codec cfg(test)。独立M作者: 既存 `tests/audio_history_migration.rs`、store/commitの `i2_migration_tests` 内の下記限定改訂。最大4本、`src/commit.rs` 本体/旧test/G節は所有外 | 全Deserializeを単一版dispatchへ接続しruntime v2/nullable必須rootを有効化し、移行判定用の元wire版を正規化後も保持する。v1全field保持・未知/欠落/不正拒否、load読取で移行書込なし。read_progress/ProgressHistoryHeadは復旧・mkdir・backupなしで型付きerrorを返し、S-Mで通常/新教材の実v2保存前退避と旧wire1 pending exact replayを別々に検証する（T01/T02）。[独立Post PASS](history-progress-review.md) | S15/M25/G24/旧17の独立81件成功・失敗0・ignored0。初回1件のf32 oracle経路差は独立訂正し旧失敗を保持した。高/高：R-Mまでは全first-v2-write保護未完成 |
| [x] I2a-N 通常保存 | 本体 `src/store.rs`。新test `tests/audio_history_save.rs`。2本 | save_progressがgate下のdisk rootとclosureを検証し候補の非履歴fieldだけ保存、実保存Progressを返す。旧saveはwrapperとして同じ処理を通す。exact再読で成功/旧bytes/第三状態を分け、第三状態で共通write-block、readは可能（T02/T03/T04） | 独立100 PASS・最終Post PASS/Closed（history-normal-save-review.md）。初回99/1のOS oracle限定訂正履歴を保持。100MB・既存小上限seamを維持 |
| [x] I2a-R 教材recover事前検査 | 本体 `src/commit.rs`、必要な既定codec/guard接合だけ `src/store.rs`。新test `tests/audio_history_material.rs`。最大3本 | 全metadata変更前にbounded intent/deck/旧新Progress/新closure/old-new root一致/現在exact old-or-newを検査し、事前拒否でbackup/tempも不変とする。new wire=2/old wire=1なら検査後・両metadata前にintent旧exact bytesをM helperで退避/readbackし、current=new・backup欠損でも省かない。new wire=1のexact replayを維持し、必要退避確認後にmetadata再読→pending退避を行う（T05/T05b、I2-D1、R-M） | 独立125 PASS（R25＋旧100）・最終Post PASS/Closed（history-material-recovery-review.md）。時刻取得不能/初回archive名衝突は静的のみ。検査後I/O失敗と事前拒否のledgerを区別 |
| [x] I2a-T 新教材save | 本体 `src/commit.rs`、既存helperの接合だけ `src/store.rs`（現案では変更不要）。T作者はR作成後の `tests/audio_history_material.rs` へのT追加とcommit専用T cfg(test)。M作者は同commit内の下記成功seen 1件のみ。最大3本・同file逐次所有 | 加算的commit::save_progressが最新rootを継承し実保存値を返す。構造/closure/64MB・100MB・escaped intent350MB/移行退避を先に確認しpending exact読戻し後だけmetadataへ進む。Rのunder_guard回復へ同じfaultsを渡し、intent前後/各metadata前後/finish失敗を分類する（T02/T04/T05） | 独立146件/0FAIL/0ignored、15hash一致/temp0、[最終Post PASS/Closed](history-material-save-review.md)。materialとlib commit、migration/gate/saveを確認。新T21と正式M成功配列1件のRED→GREEN、R成功直後の隙間は静的のみという境界を保持 |
| [x] I2a-V publication権限 | 本体 `src/audio_history/store.rs`（readonly confirm接合を含む）。専用cfg(test)を追加し、recovery/既存integrationは不変 | `verify_publication_under_guard`がstrict intent/exact mutation/request/base/next/planned objectsを照合する。既存再計算とconfirm本体を共有し、偽Published拒否と既commit/cleanup後のreadonly確認を別契約として保持する。公開I1 confirmのcustom head/reader上限/error順序を維持し、pointerは変更しない（T03/T03b） | 独立101件/0FAIL/0ignored/timeout0、8hash一致/新temp増分0、[最終Post PASS/Closed](history-publication-authority-review.md)。V08限定fixture訂正と先行watchdog中断の証拠を保持。metadata Io/検査中置換/実closure100k nodesは未測定。P2file所有は維持 |
| [x] I2a-P 履歴pointer | 本体 `src/store.rs`。新test `tests/audio_history_commit.rs`。2本 | commit_historyが最新ProgressのrootだけをCAS相当条件下で変更し非履歴fieldを保持する。cancel直前/既commit receipt/cleanup済み再確認を区別し、Vのhelperなしの新commitは不可。AfterPointer faultでもexact再読でCommitted/未確定/write-blockを判定する（T03/T03b/T04） | 新target `audio_history_commit`、gate/save/materialの競合関連case、既存store/recovery target。高/最高：Saved判定・取消し・結果不明。write-block隔離試験は別process。2026-10-11 [最終Post PASS/Closed](history-pointer-commit-review.md)、[独立269件成功](history-pointer-commit-runner.md) |
| [x] I2b-A 主要consumer | 本体 `src/app.rs`。同fileにtest作者の専用cfg(test)。1本 | persist/import_deck/restoreが各save_progress返却値を採用してからdirty等を更新する。stale None/Bがdisk Aを戻さず非履歴draftを保持する。失敗で旧snapshotを書き戻さない（T02） | 新16＋旧4の[独立20件成功](history-main-consumer-runner.md)、2026-10-11[最終Post PASS/Closed](history-main-consumer-review.md)。中/高：UIの正本clone接合だけ、描画変更なし |
| [x] I2b-B v1 backup内容 | 本体 `src/backup.rs` と `src/store.rs` の下記Path読取bridge/共有内部読取接合だけ。新test `tests/audio_history_backup.rs`、両本体の専用cfg(test)。最大3本、逐次所有 | restore/preserve_current_progressは最新root＋未保存draft/ai_callsを保持し実保存値を返す。history付きcandidateまたはdiskのv1 exportは宛先作成前拒否、なしは明示v1 wireで往復する。混入root/未知manifestを拒否する（T06、bridgeの不存在/安全契約は下記） | [独立76件成功](history-backup-runner.md)、2026-10-11[最終Post PASS/Closed](history-backup-review.md)。高/高：I3 backup v2/unionは含めない。backup_uiは返却値の既存代入を維持し編集しない。外部write-block対応はd-H/Bへ残す |
| [x] I2d-H 外部宛先共通境界 | 本体 `src/store.rs`。新test `tests/progress_export_destination.rs`。2本 | 副作用なしroot解決、write_external_file/create_external_directoryを設計§5どおり実装する。root全配下・祖先/leaf link・hardlink・ADS/device・identity不明を拒否し、解決親固定＋直前再検査を行う。共通gateの外部退避用取得口とunder-guard接合で通常外部/似た名前/正本write-block中の退避を許し、blockを解除せず拒否時bytes/temp不変（T08/T08b、I2-D2） | 新target `progress_export_destination`。高/最高：Windows identityとaliasが主要不確実性。文字比較fallback/別Mutex/OS設定変更なし。helper完成だけでD2実装Closedにしない |
| [ ] I2d-S 設定export consumer | 本体 `src/app/settings_ui.rs`, `src/app/control_settings.rs`。test作者は各file cfg(test)。2本 | settingsのTSV/JSON/診断とcontrol_settingsの診断JSONLの4出口を共通file helperへ接続する。実Storage.dirまたは同じ副作用なしroot解決を使用する。拒否時成功表示なし・draft/既存dest不変、通常外部内容を保持（T08/T08b） | `cargo test --locked --bin wordweave5 i2_export_settings`（両fileに同名module）、既存settings関連case。中/高：描画刷新なし。共通helperを迂回するatomic_writeを残さない |
| [ ] I2d-B backup宛先consumer | 本体 `src/backup.rs`。既存計画test `tests/audio_history_backup.rs`、同file cfg(test)。2本 | assets限定reject_asset_destinationを共通Directory検査へ置換する。既存rootと保護配下新規dir/aliasを拒否し、内容snapshot取得から宛先作成・完了marker最後まで同じ外部guardを保持する。write-blockを解除せず適格な外部backupは成功する（T08/T08b） | backup targetと `--lib backup::`、destinationの関連caseと子process隔離のwrite-block/再入回帰。中/高：b-Bの内容検査/bridgeは再利用し、guardを取るdirectory helperを入れ子に呼ばない |
| [ ] I2d-M 媒体export consumer | 本体 `src/app/chat_media.rs`。test作者は同file cfg(test)。1本 | export_unsaved_chat_audio/annotation_text/annotationの3出口をfile/directory helperへ接続する。annotation一式の各fileとmarkerも保護境界を迂回しない。拒否時payload/annotation/既存bytes保持、通常外部exportとmarker順を維持（T08/T08b） | `cargo test --locked --bin wordweave5 i2_export_media`（新cfg(test)module）と既存chat_media case。中/高：成功前clearを禁止。後続b-Cのsave変更と同時編集しない |
| [ ] I2b-S 設定・配色・会話保存 | 本体 `src/app/settings_edit.rs`, `src/app/color_theme.rs`, `src/app/conversation_trash.rs`。test作者は各file cfg(test)。3本 | 全既存save呼出しが返却ProgressをUIへ採用する。draft/preview/削除guardを維持する。stale rootと保存失敗の独立caseでdisk/UI root・非履歴fieldを照合する（T02） | `cargo test --locked --bin wordweave5 i2_history_settings`（各fileの新module）と既存settings/color/chat削除関連case。中/高：3 consumerは同じ小さい返却値接合に限定 |
| [ ] I2b-C chat保存 | 本体 `src/app/chat_ui.rs`, `src/app/chat_media.rs`。test作者は各file cfg(test)。2本 | chat結果/録音/注釈保存が実保存値を反映する。失敗時payload/ink/元媒体を維持する。stale rootと未保存fieldの回帰を通す（T02） | `cargo test --locked --bin wordweave5 i2_history_chat`（各fileの新module）と既存chat_ui/chat_media関連case。中/高：I6 worker実装・送信は含めない |

新test target/module名は**作成予定**であり、既に実在/実行済みとは扱わない。作者が作成後に `--list` で対象と実行件数を固定し、runnerは0件成功を受入にしない。前行testをread-onlyで走らせることは所有file追加ではない。期待値の追記が必要ならその行のtest作者へ逐次返す。表外のfile・新依存/manifest変更が必要なら、5本へ詰め込まず親へ所有分割を返す。

2026-10-08 b-A準備の限定照合：既定designer（Astra/high、read-only）は承認設計の再利用で接合可能、新API/新設計書不要と確認した。persist/import_deck/restoreは既存成功分岐に実保存Progressの採用を入れ、dirty/last_save/deck/message/epoch/chat reset等の既存更新順と失敗時fatal/候補保持を維持する。import_deckへ従来にないepoch更新を追加しない。restoreは復元前draftを別backupへ保存しUIを復元候補へ置換する既存契約であり、UIに旧draftを残す意味ではない。`json_restore_preserves_the_latest_unsaved_chat_draft`を維持する。restore失敗でも先行backupは残り得るため全ledger不変を要求しない。手製disk root Aとstale None/B、非履歴sentinel、pending拒否を同fileの既存harness_tests::fixtureから検証できる。backup内root整合はb-B所有、b-A source/testはP最終Post後である。

#### b-B限定計画delta: Path読取bridge（2026-10-11、親確定・設計反映・独立Pre PASS）

確定：設計SHA `A7A6931A63A53E6842B91DC816ED1430BAF2D3EEB3CC95F753C6226AE26EFBE0` の[独立Pre PASS](history-backup-review.md)、阻止0を受領した。以下のPre待ちという準備時記述はこの確定により解消し、独立作者へ進む。本体・runner/Postは未了である。

暫定案はstoreの別前提単位であったが、[設計のb-B接合節](design-i2.md)と現物のprivate `read_progress_bytes/read_progress_typed`を照合し、**b-Bへstoreの限定所有を加える3file案**へ絞った。同じbackup経路のread接合であり、別reader・別runner単位を増やす必要がない。b-A最終Post PASS/Closedを依存入力とし、単位順は維持する。designerが以下の共有配置を設計へ反映し、独立Preで確認してから作者→実装へ渡す。本deltaだけでPre合格や本体着手を宣言しない。

- [ ] **所有と契約の固定**：`src/store.rs` の `pub(crate) fn read_progress_for_backup(root: &Path) -> Result<Option<Progress>, String>` と、既存の安全な祖先/leaf検査・bounded read・strict版dispatchを共有する最小private接合だけを実装者へ追加する。不存在許容policyの分岐は共有reader側へ置き、通常 `read_progress_bytes/read_progress_typed`・Storage・headは従来の厳格policyを使う。型の全体複製、backup内の別safe reader、`Storage::at/open`、mkdir/lock/recover、gate/公開API/manifestの変更は対象外。bridge自身はgateを取らず、b-B export側が既存gateを1回保持して呼ぶ。
- [ ] **不存在と拒否の独立受入**：既存rootでは`pending_checked`のErr/trueを拒否して共有typed readへ進み、検証済み祖先の下でprogress leafだけがなければNoneとする。export元root自体が元からない場合は既存祖先を同じmetadata安全検査で確認した不在だけをNoneとし、存在しないrootへのpending検査で誤拒否しない。`exists()==false`だけでは不在としない。通常readerのroot不存在は従来どおりErr、観測済みleafの消失/I/O不明・不正path・未知版・破損・上限超過もErrである。手製v1/v2のfield/root保持、rootなしとleafなし、pending、不正祖先/leaf、厳格入口との対照を別caseにし、全既存bytes不変・root/dest/temp新設なしを確認する。
- [ ] **b-B統合と検証**：test作者はstoreの新 `cfg(test) mod i2_backup_path_tests`（予定名）とbackupの既定cfg(test)/新integrationだけを所有し、実装者へ逐次移譲する。必要なら固定Err scaffoldを既定手順で置き、compile失敗と行動REDを区別する。`cargo test --locked --lib store::i2_backup_path_tests`、`cargo test --locked --lib store::i2_progress_tests`、`cargo test --locked --test audio_history_progress`、`cargo test --locked --test audio_history_backup`、`cargo test --locked --lib backup::` を実在名/件数へ固定する。既定b-Aの `i2_history_main_consumer`、`backup_confirmation_`、`json_restore_preserves_the_latest_unsaved_chat_draft` もbin回帰として確認する。上限のexact-1/exactは独立wire1投影をoracleとし、memory退避のruntime2 oracleは保持する。新規fixtureは合成だけ、guard再入/状態汚染は有限timeout子processで分離する。

b-B受入は設計B1–3を再利用する。restore後はbackup draft＋日別ai_calls最大値、復元前current draftは別memory退避で保持し、各保存区間の最新rootと実保存返却値を照合する。事前拒否と、memory/媒体追加後のcommit失敗で許容する残存を分ける。最大3fileでも祖先安全性とNone誤分類による履歴喪失があるため難易度/危険は高/高である。共有readerの既存厳格契約が変わる案、4file目、期待値変更は親へ戻す。`acquire_progress_write()`は内部でwrite-blockを拒否するため、b-Bの通常gate成功をwrite-block中の外部退避成功へ読み替えない。この不足は上表d-H/Bで閉じる。

本deltaのb-B・影響するd-H/Bそれぞれに、planner／external-spec author／designer／implementer／test author／reviewerは **`gpt-6-astra` / `high`**、test runnerは **`gpt-6.1-sol` / `medium`** を明示指定する。固定2役は規約、設計/実装/作者/独立reviewは保存と安全境界の判断、runnerは固定差分の手順実行という既定理由である。新外部仕様は不要で承認済みAH-AC16/T06とT08/T08bを再利用し、observableな変更だけspec役へ戻す。両モデル/effortは呼出可能一覧内、実行metadata・開始/終了usage counterは未取得。親Astra/highは変更せず、**親はrunner切替前に停止して利用者へ知らせる**。既定Pre/Post・format/diff・I10を維持し、役/model/skill不在はblockerとする。次handoffは親→designerの本限定反映→独立Pre→作者である。

製品EXEの追加依頼は、**I9完了前でもmainの試用milestoneを置ける**。I3までの保存/backup、I4–5とC1、I6–8の本体操作受入・独立レビューが完了し、固定差分に `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`、既定の関連crate/format/diff検証を通した時点である。対象SHA/EXE hashとI9/二host・実機/APIの未完了を明示し、I9/C2/I10やAH-AC18を完了扱いしない。mainの動作契約にI9の実装依存が判明すればその解消まで提供しない。全体完成は従来どおりI10であり、trialを全体gateの代替にしない。承認済みcheckpoint pushとこのEXE提供依頼を維持し、現時点のrelease/publishは行わない。

M→Sの順は実在する旧save/commitがProgressを直接serializeするためである。Default=v2を先に変えると初回退避を迂回し得る。Mで新しいv2書込予定値に対する共通退避hookを先に置き、private合成入力で検証してからSでschemaを有効化する。recoverはintentの元exact文字列を使い続ける。各中間単位の合格は完成I2の提供許可ではなく、全経路のroot保護はN/R/T/Pとconsumer閉鎖まで未完了である。history controllerを早期接続しない。

ProgressFaults/Checkpointの型と最小非副作用入口はM以降の最初の使用単位が所有する。Migration→Material→Pointerの各行が自分のcheckpointだけ接続し、独立test作者は設計の全checkpointを順に埋める。API不在によるcompile失敗を行動REDに数えない。必要時は作者の契約/期待値を先に固定→実装役が**新しい未接続入口だけ**に固定Errの非副作用scaffoldを置く→作者がcompile/行動REDを確認→実装役が本体を実装する。既存入口をstubへ置換しない。scaffoldは同じ行の所有file内、別の役割免除や期待値変更の権限ではない。

#### S限定計画delta: M試験所有とoracle移行（2026-10-08、親確定済み）

確定記録：M独立runner62件成功・最終Post PASSを受領した。designerの限定設計SHA `CE1B45349BD7DF4C20DBBC22ADF4C24888DB681EB7533A8244864830E891BF63`と正本SHA `5E4894D3A2EC7CAFA87115D45AD74B293DFAD01B2C8F436EF4D10D4379E2E714`についてS限定Pre PASS・阻止0である。以下の「暫定/未Pre/着手不可」は確定前の入力記録であり、この確定によりS scaffoldへ進む。所有・モデル・品質gateは変更しない。

暫定判断はSの最大4file所有である。現物のM試験はruntime1を前提とするため、integration第3fileだけでは不足し、`src/commit.rs::i2_migration_tests` の第4fileが必要である。入力正本 `design-i2.md` SHA-256 `1375E2A126DBA2AEAD188903D3BA3E0575293B3C919C7721D5786EB771090EAF` とS準備案 [history-progress-design.md](history-progress-design.md) SHA-256 `D7D70DDCBB6A09C2B81AAD79EA3FE0933D8348B0414D54288EC6E13144BAE5AD` を照合した。S準備案は未Preであり、親受領のM自己検証62 GREEN/静的阻止0も独立runner/Postの代替ではない。**現在はSのsource/test着手不可である。** 正本統合/Pre後にこのdeltaを確定し、他単位・上限・gate・未完了計画は変更しない。

| 現物のケース | Sで固定する独立oracle / 所有者 |
| --- | --- |
| migration integrationの `assert_kept`、`m05_v1_load_is_read_only_and_save_keeps_daily_exact_backup` | M作者がdecode後runtime=2/全sentinel保持とload時全bytes不変を固定する。保存後wire2・必須null、旧v1のhash名退避exact一致と従来の日別退避を別々に照合する。単にbackup総数assertを外さず、種類/集合と各bytesを確認する |
| integrationの `m05_new_v1_material_save_keeps_old_exact_intent_without_migration`、commit専用節の `m05_v1_material_private_entry_matches_existing_save_without_migration` | M作者が新教材wire2保存のS-Mケースへ改訂・改名する。成功は旧exact intent/媒体保持＋hash退避、faultなしは移行Before→Afterを要求する。Before faultは保存失敗・metadata/pending/退避不変、After faultは旧metadata/pending保持・確認済退避だけ残存とし、空seen/退避なし成功を維持しない。通常保存にも既定private入口で同じ順序対照を置く |
| integrationの `m05_handwritten_v1_pending_replays_exact_bytes_from_each_partial_state` | M作者が手製wire1 fixture・3中断位置・disk/pending archive exact・再recover不変・hash退避なしを維持する。追加するdecode後runtime2 assertはdisk wire1と分離する。元文字列をruntime serializeへ置換しない |
| store専用節の `m03_invalid_raw_version_or_structure_is_rejected_without_artifacts` | M作者がversion2/root欠落の期待をInvalidDocumentへ改訂する。手製の正しいv2/null・有効rootはstrict検証後退避/fault/publisherなしで成功、v2不正root/未知top/重複等は固定分類で拒否する。未知versionはUnsupportedVersionを維持し、v2全部拒否・両版許容へ緩和しない |
| store専用節の `m05_v1_shrink_does_not_add_migration_read_or_source_validation` | M作者が新runtime2候補の保存へ改訂・改名し、上限超過旧bytesと破損旧bytesを別ケースで無変更拒否とする。private helperのtarget1 no-op対照（`m03_target_one_is_noop_before_source_limit_path_or_faults`）とtarget2の上限拒否は維持する。M段階の公開v1縮小成功証拠は過去の固定source/logに保持し、Sでv1候補を無理に通す迂回を追加しない |
| commit専用節の `m05_v1_material_preserves_old_source_and_escaped_intent_limits`、その他M01–04 | 旧source/escaped intentの上限拒否・fault前停止・ledger不変、exact hash/同名競合/readback/path/実I/Oのoracleは維持する。上記版dispatchに直接依存するfixture/名称以外の改変は割当外である |

M作者は変更前ケース→要求→変更後ケースの対応を既存M証拠と併記して親へ返す。Mの固定fixture/hash・RED/GREEN記録を上書きせず、S前後のoracleを識別する。これは承認済みruntime2契約への改訂であり、失敗許容・assert削除・ignore・両版成功による救済ではない。S作者はS01–05のcodec/read/headを所有し、同じstore fileはM作者→S作者→実装者の順で逐次移譲する。実装者はstore本体のみ、commit本体・G全体・旧testの広い修正、Rのrecover hook、Nのroot継承、backup/consumer修正は対象外である。

7役とexact model/effortは直下表をSにもそのまま適用する。planner/外部仕様は `gpt-6-astra/high`、designer/implementer/S・M test author/reviewerも `gpt-6-astra/high`、runnerは `gpt-6.1-sol/medium` である。主なリスクは元wireとruntimeの混同、初回退避の省略、oracleを実装へ合わせることであり、保存契約と独立期待値の判断をAstra、高リスクでも固定手順の実施はSolへ分離する。既存外部仕様の再利用を維持し、未規定のobservable変更だけ固定spec役へ戻す。

- [x] 親がM独立runner/Postの対象hashと完了を記録し、designerが上記所有/元wire保持をS設計へ反映・正本統合し、独立Pre後に本deltaを確定する。文書反映は各既定担当へ返す。
- [x] 独立M作者/S作者が上表とS01–05の期待値・ケース対応を先に固定し、実装後runnerが対象4EXEの81件を独立実行した。集約cargo --no-run→固定EXE方式、事前`--list`で実在件数を確認。S前MとS後の追加/改名を分け、初回oracle失敗も保持した。詳細はhistory-progress-tests.md / history-progress-runner.md。
- [x] reviewerが独立Postで4file境界、commit本体/旧test/G節の不変、M oracle対応と新証拠を確認した。source/EXE hash、format/diff、初回失敗を記録しS Post PASS。I10全体test/release buildと既存権限条件は維持し、S成功をR-M閉鎖・製品提供完了へ拡張しない。

未決はS設計Preと上記ケース分割の確定である。4fileで不足する場合は独断拡張せず、追加責務と失敗ケースを親へ返す。分割候補はruntime1を保つpure read/headの先行小単位と、codec/runtime2＋S-M oracleの後続小単位であるが、各中間状態の独立受入をdesigner/plannerが確定するまでdispatchしない。本deltaでは分割・順序を確定変更しない。plannerの検証は文書差分/参照path/入力hashのみ、runtime metadata・使用量開始/終了counterは未取得であり、親の同一task測定へ欠測を引き継ぐ。

#### T-M限定計画delta: 既存M成功seen 1件（2026-10-08、準備のみ）

[T限定設計](history-material-save-design.md) §4/5、SHA-256 `9B805A51681B70A3105DA10D5042A1F529EA8A6D605034B6AA4A7D25AD33FD74` と現物を照合し、**最大3fileのまま可能**と判断する。独立M作者（既存担当 `history_path_tests`）へ `src/commit.rs::i2_migration_tests::m05_v2_material_private_entry_matches_public_save_with_exact_migration` の成功時 `faults.seen` 期待配列だけを正式割当する。他assert/fixture/補助型/本体は割当外である。T作者の新規T試験・実装役とのcommit.rs同時編集は禁止する。R最終Post固定差分/所有返却→正本統合・T独立最終Pre→作者の期待値固定→必要なら新規未接続口だけscaffold→行動RED→本体→独立runner/Postの順を維持し、現在のR作業中にT source/testを開始しない。

成功seenは順序厳密な12件、`BeforeMigrationBackup, AfterMigrationBackup, BeforeMaterialIntent, AfterMaterialIntent, BeforeMigrationBackup, AfterMigrationBackup, BeforeDeck, AfterDeck, BeforeMaterialProgress, AfterMaterialProgress, BeforeMaterialFinish, AfterMaterialFinish` とする。先頭Mはpending前退避、2回目MはRの独立再検証であり、2回目を省略/隠蔽しない。Pointerは含めない。M作者は旧2件→この12件という要求対応と差分だけを返し、旧2件の証拠を削除・上書きしない。両順序許容や件数だけのassertへ緩めない。

`m05_v2_material_migration_faults_never_publish_pending_or_metadata` の最初のT-M fault拒否/seen/残存ledger、`m05_v1_material_preserves_old_source_and_escaped_intent_limits` のfault前拒否、同成功caseのexact backup/intent/媒体/保存値assertを維持する。M/S integration、旧wire1 pending、G/R/N既存oracleとR単独recover順序は編集対象外である。既存不合格は許容せず当該担当へ戻す。T作者はT01–06を既定2箇所へ追加し、旧wire2/旧fileなしの8件順序、同variant出現回数を数えるtest-local faultによるR-M2回目停止、full save→R各fault/ledgerを独立確認する。公開checkpoint追加・別recover本体・R試験期待値変更は不要である。

- [ ] R Post後のsource/既存M oracle hashを固定する。T新口scaffoldは既存save/limits/R入口へ接続せず、新T正例の固定Errによる行動REDと、M成功seenの実測2件対期待12件のREDを別々に記録する。Mの後段assert未到達を成功に数えず、compile失敗/0件/fixture不成立は行動REDに数えない。実測順序が異なれば期待値を合わせず依存差分を再照合する。
- [ ] 本体接続後、runnerは `cargo test --locked --lib commit::`、`cargo test --locked --test audio_history_material`、既存migration/gate/save targetを実行する。事前`--list`でM指定1件/T追加/R既存の実在名と件数を分け、対象source/EXE hash・RED→GREEN・後段assert到達・失敗/未実行を残す。新T module名は作者確定後にhandoffへ記録する。
- [ ] reviewerは独立PostでM差分が期待配列1箇所だけであること、T/Rへ同一faultsが伝播すること、他M/S/G/R/Nのoracle不変と全既定受入を確認する。format/diff・I10全体gate・既存権限/上限を維持する。第4fileや別caseの変更が必要なら親へ限定再割当を返し、独断拡張しない。

7役は直下表を維持する。planner/外部仕様/designer/implementer/T・M test author/reviewerは `gpt-6-astra/high`、runnerは `gpt-6.1-sol/medium`。高難度/最高リスクの二file状態遷移とoracle改訂をAstra、固定証拠の実施をSolへ割り当てる理由も維持する。未決はR完成後のprivate署名一致とT最終Preであり、次は親が本割当をdesignerへ返し、依存解消後にM作者へ期待配列1箇所をdispatchする。本plannerは文書のみ検査し、runtime metadata/開始終了usage counterは未取得である。

#### V-P限定計画delta: readonly confirm接合（2026-10-08、設計反映前）

入力は [V案](history-publication-authority-design.md) SHA-256 `88826DCBB1E41AB33E0005F6A7D10DDD9D9A29851C2D0B131EB7E69E9FDA25A7` と [P案](history-pointer-commit-design.md) SHA-256 `6ABC29BA59659418F8B36B422016B9B4E3F743A13BB31FC770978F091E3036B7` の未承認deltaである。現物のHistoryWriterは `&mut Storage` を保持するためPの `&self` から構築できない。**Vの既定最大3file内で追加割当可能**である。V実装役へ `audio_history/store.rs` 内の `HistoryWriter::confirm_publication(storage: &Storage, published: &PublishedHistory) -> Result<CommittedHistory, HistoryError>`（crate-private associated関数）と既存confirm本体のprivate `confirm_with_reader_and_head` 抽出を割り当てる。Pの本体/test 2fileは増やさない。recovery.rs変更は必要な既定private接合だけであり、現案では不要である。

公開I1 `confirm_committed` はself.reader(u64::MAX)/self.headを用い、新associated関数は同じ100,000 nodes/u64::MAX retained条件と固定ProgressHistoryHeadを用いて、同じconfirm本体へ委譲する。guard→operation/request→next closure/receipt→存在intentのstrict検査・旧計画再計算/affected→最新head closure/receipt完全一致→pending/head再確認という既存順序/errorとcleanup済み成功条件を保持する。read_history_at/checked_old_planのV抽出を再利用し、custom headは既存公開I1/private内部に限定する。caller指定headの追加、偽writer/Storage、再open、検証複製、新authority、metadata/intent/cleanup書込、再gate、write-block変更は含まない。confirmの読取成功を新pointer権限へ読み替えず、新commitのverifyはstrict intent必須のままである。

- [ ] designerが本deltaをV/P案と正本へ反映し、親がR/T最終Postの固定差分・所有返却とV独立最終Preを確認するまでV source/testを開始しない。V Post後にPが実helper署名を照合し、P正本統合・独立最終Preを経る。計画上の割当だけで各gateを通過扱いにしない。
- [ ] V独立作者がstore専用cfg(test)へ新associated関数のdisk head正例（next/後続root、intentあり/cleanup済み）、base上receiptなしBusy対別head Conflict、request/affected/revision不一致・不正残存intent・next/current破損・strict pending拒否を固定する。同じdisk headを使う公開I1との値/error同値、diskと異なるcustom headを使う公開I1の契約維持、小quota超の既存closure確認、全file/dir ledger不変を対照化する。既存100,000 nodes/個別document上限とerror順序は差分レビューでも照合する。guard保持中の新confirmが再lockせず、write-block下でも両confirmのpure readが可能であることを既定子process隔離で確認し、reset口を追加しない。
- [ ] runnerは `cargo test --locked --lib audio_history::`、`cargo test --locked --test audio_history_recovery`、`cargo test --locked --test audio_history_store` を実行する。作者が新case名/`--list`件数を固定し、抽出前のsource/既存回帰と抽出後source/EXE hashを区別する。w03のexact mutation/後続root/cleanup/公開token偽造、w08の旧計画再計算、missing分類/rebase/quota/finishを既存期待値のまま確認する。新口scaffoldが必要なら既存confirmを接続せず、独立正例の行動RED→GREENを記録する。compile不成立・0件・旧ログを新成功に数えない。

reviewerは独立Postで共有bodyが1つであること、公開I1のhead/reader/error/cleanup互換、新confirmがdisk固定headを使い書込/gate取得なしであること、V新commit検証と既commit確認の分離を確認する。既存oracleの変更・第4fileが必要なら親へ戻す。V01–06とformat/diff・I10全体gateは維持し、P/I6/UI完了を主張しない。7役の既定指定（planner/外部仕様/designer/implementer/test author/reviewer=`gpt-6-astra/high`、runner=`gpt-6.1-sol/medium`）と理由を継承する。高リスクは保存済み誤判定と既存head契約の回帰であり、独立期待値/レビューで確認する。次は親からdesignerへの限定反映handoffである。本plannerは文書検査のみ、runtime metadata/開始終了usage counterは未取得である。

#### 全単位の7役・起動値

本追補の全17単位（a-0/G/M/S/N/R/T/V/P、b-A/B/S/C、d-H/S/B/M）について、以下の割当をそれぞれ明示適用する。親は省略/黙示inheritせず、各dispatchへexact ID/effortを渡す。

| 役割 | 各単位の正確なmodel / effort | 選定理由・所有 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 固定役。本節の分割・依存・所有のみ |
| external-spec author | `gpt-6-astra` / `high`、今回の新規執筆はN/A | 承認済みspecを再利用。observable変更が必要なら親へ戻し、この固定指定で仕様工程を再開 |
| designer | `gpt-6-astra` / `high` | 元I2指定を維持。上表全単位の保存/失敗/alias境界の承認設計を再利用し、未解決内部APIだけ担当 |
| implementer | `gpt-6-astra` / `high` | 元I2指定を維持。少数fileでも既存データ喪失・権限迂回の影響が大きい。上表の本体だけ |
| test author | `gpt-6-astra` / `high` | 元I2指定を維持。手製bytes/hash集合/ledger、stale clone、fake Published、Windows aliasの独立oracleが必要 |
| test runner | `gpt-6.1-sol` / `medium` | 元I2指定を維持。固定した差分/期待値を変更せず実行数・失敗・未実行を記録する手順が明確 |
| reviewer | `gpt-6-astra` / `high` | 元I2指定を維持。作者とは独立read-onlyでPre/Postの証拠と全経路を確認する。自分の指摘修正/自己承認不可 |

各行の難易度/危険は表のとおりである。狭いconsumerでも同じ指定を維持するのは親が元I2割当の維持を求めており、保存root喪失の影響が共通するためである。ホストの呼出可能一覧に両exact modelと指定effortがあることを確認した。起動指定と実行metadataは別であり、本plannerの実行model/effortと開始/終了usage countersは**未取得**、後続役も取得できなければ未取得と記録する。親モデル/effortは変更しない。役/モデル/skill不在はblockerであり無断fallbackしない。

各役は既存の担当skillを全文読了し範囲内で使用する。plannerは `agent-skills:planning-and-task-breakdown`、designerは `agent-skills:api-and-interface-design`、test authorは `agent-skills:test-driven-development` の作者工程、implementer/runnerは `wordweave-change` の各担当工程、reviewerは `agent-skills:code-review-and-quality` と保存整合性の該当skillである。本追補はskillの依存順/小単位化を適用し、既存計画を上書きせずI2だけを具体化した。子の再委任・公開は禁止し、通常補助1役の逐次運用を維持する。

#### 最初のdispatch packet: I2a-0

- 入力は本節、design-i2.md §3/8（上記SHA）、既存reader/closureと独立closure testである。親はI1b-2b追加2件＋独立Post完了と該当source差分を確認してからtest authorへ渡す。現在dispatchできるのは依存確認・読取準備までであり、本体着手許可と混同しない。
- 正確なAPIは追加 `pub(crate) fn HistoryReader::for_root(root: &'a std::path::Path, limits: HistoryLimits) -> Result<Self, HistoryError>`。private保持を `&Path` へ変更し、既存 `new(&Storage) -> Self` / `with_limits(&Storage, HistoryLimits) -> Result<Self, HistoryError>` はdirを借用して既存上限検査を共有する。closure.rsの2箇所をそのroot参照へ接続する。writer/recovery/Progress/API公開範囲は触らない。
- 作者は `src/audio_history.rs` の専用 `cfg(test) mod i2_path_adapter_tests` と `tests/audio_history_closure.rs` の追補だけを所有する。手製closureに対するStorage入口/Path入口の同値、invalid limits、欠損/reparse拒否、全bytes不変を固定する。for_root不在で試験を開始できない場合だけ、上記手順のscaffoldをimplementerがaudio_history.rsへ置き作者へ所有を戻す。for_rootは固定Err以外の副作用・本体迂回を持たない。
- 実装者へは `src/audio_history.rs` の本体と `src/audio_history/closure.rs` の2参照を移譲し、テスト期待値を変更しない。runnerは上表2コマンドの実在case/件数、対象3fileのhash、git差分範囲、失敗/未実行を返す。reviewerへ独立handoffして、公開API互換・root安全検査・実Storageを偽造しないことを確認する。
- 既存Windows leaf symlink特定1件の別実行承認は既存条件のままである。新しいalias試験へ流用しない。既存caseの再実行が変更影響として必要なら既定の非管理者build→EXE hash固定→許可された1件別実行の証拠を親が扱い、未実行をPASSへ含めない。

#### 依存・risk・残す判断

I2-D1の実装閉鎖はR/TのT05bとPostを要する。I2-D2の実装閉鎖はd-Hだけでは足りず、d-S（4出口）、d-B（backup directory）、d-M（媒体3機能と構成file）のconsumer試験/検索を合わせる。設定JSONのみの修正で閉鎖しない。Storage取得失敗時に元rootを再現できないconsumerはexportを安全と推測せず既存エラーへ返す。root解決のためにStorage::openを再取得しない。

未確定はWindows identity取得の利用可能API/権限環境と、I1b-2b Post後のprivate再計算helper差分である。新依存追加、契約変更、6本目の所有、OS権限拡大を必要とする場合は親へ根拠を戻す。API実装の選択は設計範囲で担当者が決められるため、通常の手順選択で利用者へ再承認を求めない。

runnerは各行のfocused regressionと影響integration、reviewerは保存整合性を独立確認する。I10の `cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5`、他既定crate/format/diff確認は既存どおりである。故障fixtureと電源断耐久性、native UI、実録音/実APIは区別する。本追補作成では文書のpath/所有/依存/hashとdiff形式だけを確認し、build/testを実施しない。usage欠測は親の同一task測定へ引き継ぐ。

以下の既存I0 handoffは履歴として保持する。現在のI2 handoffは本追補を用いる。

親はI0 designerを `ww-designer` / `gpt-6-astra` / `high` で起動し、入力をspec/design/tests/evidenceと本書に限定する。出力はdesign追補、固定版・配布・検証方針、正確なファイル/API所有表である。spike/テストのコード実施は指定implementer/test author/runnerへ順番に渡す。I0確定後、独立test authorの期待値を先に受け、I1とI4の実装担当を上記割当でdispatchする。実装許可を再質問する必要はないが、gate不成立やobservableな仕様変更は根拠をもって親へ返す。
