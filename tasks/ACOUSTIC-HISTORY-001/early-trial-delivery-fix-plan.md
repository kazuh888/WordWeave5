# ET3-FX1：all-targets隔離helper結合の限定是正計画

2026-10-14。**親の3file限定所有受領済み・設計後精緻化済み・READY FOR PRE-REVIEW**である。独立Preは未判定であり、source編集は未許可である。planner所有は本書だけ（作業リストも本書内）である。既存の未完了plan/todo、ET3全体計画、親のcurrent/usageを保持する。計画作成時のsource編集・試験実行・再委譲・公開は0である。

## 入力・完了境界

基準HEADは `d446c5dd32177815ee322a39cf51ab84a99ee458`、親のdocs-only dirtyを保持する。[runner報告](early-trial-delivery-runner.md)と `target/early-trial-build/et3/default-errors.json` のE0433、現物 `tests/codex_process.rs:16–18` と `src/ai.rs:163` を照合した。通常all-targets no-runはFAIL1、行動試験は未実行である。integration crateが `ai.rs` を再利用する一方、作者testのimport先 `crate::media` はそのcrateに存在しない。旧bin/lib focused PASSはこの構成を覆わない。

本単位はこの結合だけを除去し、既存の隔離・期待値を保持したまま両構成all-targets compileと影響集合の新EXE回帰、独立Postを成立させ、既定ET3へ返すまでである。ET3全体・native・提供の完了とは区別する。

外部仕様は[ET-01–07](early-trial-contract.md)、内部の安全契約は[起動設計のET2-G0/P01](early-trial-startup-design.md)、期待値は[独立作者のA01–D04](early-trial-egress-tests.md)、過去の成立範囲は[最終Post](early-trial-egress-review.md)を再利用する。利用者から見える変更がないため外部仕様の新規作成は非該当である。新たな仕様判断が出た場合は外部仕様担当へ戻す。

設計入力は同設計末尾のET3-FX1追補、SHA256 `B2FEFA5C9E1B10F509939F5235C2B45DBFA58F6455B83C58F19D6298F8422969` である。本書の初版 `1D64A7FDB0756F4570F23839C9517E1098F356041FABA63B031A0ACF6BF3126E` は設計時の入力履歴として保持し、今回のPre対象は本精緻化版と上記設計の組である。designerの同内容再起動は不要である。

## 要求・除外・具体化済み設計

- FX-R1：`codex_process` を含む通常/early-trial all-targetsをcompile可能にする。testの除外・ignore・featureでの隠蔽・偽media・full media取込みで通さない。
- FX-R2：`isolated(exact: &str, body: impl FnOnce(&Path))`、1実名1子、Admin=False、合成LOCALAPPDATA/TEMP/TMP/cwd、Hidden、45秒timeout、kill/wait、子回収後の固有tree cleanup、exact1/完了marker判定を保持する。cleanup範囲や失敗時契約を変更しない。
- FX-R3：P01の20実名/全assert/guard寿命、録音安全3実名、既存全consumerを保持する。通信 `GLOBAL` と録音 `RECORDING_START_WATCH` は各crate/process内で従来どおり単一とし、移動・複製・resetしない。

採用設計は、media内の `isolated` 本体を新規 **`src/early_trial_test_support.rs`** へ同体抽出し、ai/mediaの各ファイル直下にprivateな `#[cfg(all(test, windows))] #[path = "early_trial_test_support.rs"] mod early_trial_test_support;` を置く3file案である。aiでは先頭inner doc後、既存inline moduleの外に置く。新fileは関数と設計記載の明示std importだけとし、media/ai/app依存・global・test caseを含まない。各crateにcompileされるstateless helperと監視globalの複製を混同しない。media側は `pub(crate) use super::early_trial_test_support::isolated;` で既存の参照pathを保持する。aiの作者marker内は `use crate::ai::{early_trial_test_support::isolated, Config};` へのimport差替だけである。nested module相対pathや既存testのcfg/featureは変更しない。具体型・全7consumer・可視性・明示importの由来は設計正本を参照する。

本体通信API、8入口の遮断、録音本体、Cargo/lock/feature/依存、汎用framework、OS設定、Admin/UAC、実データ/資格情報/API/mic、commit/push/publishは対象外である。旧 `target/release/wordweave5.exe` SHA256 `9B888E9FC923823C59B4C3B0AB1A4DAD2051CA6139DE153B6DE6CA213DD387D3` を保持する。native/DPI等のET3残件を本書で再設計しない。

## 所有と役別モデル

作業単位は **ET3-FX1（M、source 3file、難易度中・危険高）** の一つである。修正量は小さいが、安全な実入口試験を支える共有fixtureの移設であるため、独立Pre/Postを必要とする。通常補助1役ずつ、再委譲なし、全工程を逐次dispatchする。

| 役 | exact model / effort | 所有・責任と選定理由 |
| --- | --- | --- |
| planner | `gpt-6-astra` / `high` | 本書のみ。制約・追加所有・依存・精緻化を判断する固定役 |
| external-spec author | `gpt-6-astra` / `high` | 外部仕様不変につき追加起動非該当。仕様差異発見時のみ戻す固定役 |
| designer | `gpt-6-astra` / `high` | `early-trial-startup-design.md` のFX1限定追補のみ。中立helperとcrate解決、既存安全契約同体性を確定する |
| implementer | `gpt-6-astra` / `high` | 本体変更非該当・起動不要。fixture修正は作者所有。本体変更が必要なら停止し再計画する |
| test author | `gpt-6-astra` / `high` | 下記3fileのtest/fixture限定と `early-trial-egress-tests.md` のFX1証拠追補。oracleを保持し安全helperを修正する |
| test runner | `gpt-6.1-sol` / `medium` | 既定ignored build/log/隔離fixtureと `early-trial-delivery-runner.md` のFX1記録。固定手順とartifact照合を独立実行する |
| reviewer | `gpt-6-astra` / `high` | read-only。設計Pre、同体diff/安全境界/新実行証拠のPostを独立判断する。報告保存は親 |

呼出可能一覧のAstra/high、Sol/mediumは親提供情報およびホスト定義に対応する。`gpt-5.6-terra/medium` も限定read-only lookup用に利用可能だが、本単位には追加scoutを必要としない。親のmodel/effortやquality gateは変更しない。全dispatchでexact model/effortを明示し、役/skill/modelが使えなければ停止する。指定とprovider返却値は区別し、返却値を推定しない。

| sourceの限定所有 | 書込み担当 | 許可差分 |
| --- | --- | --- |
| `src/media.rs` | test author | `early_trial_startup_tests::isolated` の同体抽出、test専用module宣言/import/re-exportのみ。安全3caseと録音global/本体は不変 |
| `src/ai.rs` | test author | test専用module宣言と作者marker内helper importのみ。20集合のA群body/名前と通信global/本体は不変 |
| `src/early_trial_test_support.rs`（新規） | test author | 既存helper本体と明示importのみ。std `PathBuf`/`Duration` 等を既存super依存から明示し、外部依存を追加しない |

`tests/codex_process.rs` は読取回帰対象であって編集対象ではない。main/app/qwen_settings/qwen_readingの全consumerは既存re-exportを利用する。他者の編集を戻さず、所有3file超過またはhelper本体変更が必要なら実装前に親/計画者へ返す。親が受領したshared harness編集禁止の例外は上表3fileの限定差分だけであり、独立Pre PASS後に作者へ渡す。

## 順序・チェックリスト

依存は **親の限定所有受領 → designer具体化 → planner精緻化 → 独立Pre → 作者fixture修正/証拠固定 → 親の静的G0確認 → runner no-run → 親の対応EXE解除 → 動的回帰 → 独立Post → ET3復帰** である。

- [x] FX0a（文書S）：親の3file所有受領、designerの具体配置/全consumer/同体証拠定義、本書の精緻化は完了した。外部仕様は既存契約を再利用し、本体実装役は非該当である。
- [ ] FX0b（独立Pre、FX0a依存）：reviewerが本書/設計/固定sourceの同じsnapshotをread-only判定する。受入：FX-R1–3を満たし既存G0/外部契約を変えない。検証：具体diff範囲、参照、両crateのmodule解決、安全契約同体性。Pre PASS前にsourceを編集しない。
- [ ] FX1（fixture M、FX0b依存）：独立作者が3fileだけを修正する。受入：helper同体、作者oracle保持、既存実名/consumer不減。検証：旧関数doc/signature/bodyの移動による行頭4 spaceだけを除いて新関数範囲と比較し、改行正規化が必要ならCRLF→LFだけを別記する。比較hash/diffを保存し、内部空白/式/文字列の差を消さない。ai作者markerの新hashと、検査用文字列の許可import逆置換だけで旧hashへ一致する証拠、残り3作者marker同hash、監視本体/他source不変を記録する。本文不一致はSTOPである。既存全20期待値の新作成や感度の無条件再実行は不要である。
- [ ] FX2（検証S、FX1依存）：runnerが通常/試用別target-dirで `cargo test --all-targets --locked --no-run --message-format=json`（試用のみ `--features early-trial`）を実行する。受入：両exit0、`codex_process` 含む全artifactをJSONから識別し、下記9source＋読取integration/Cargoの前後一致とEXE絶対path/SHA/features/profileを固定する。旧FAIL1をcompile回帰のRED証拠として保持し、行動REDと数えない。両構成成功前に旧部分artifactや旧G0解除を流用しない。
- [ ] FX3（検証/独立Post、FX2依存）：親のET2-G0解除後、構成ごとのlist/実名を固定し、bin録音安全3 → P01既定安全順20 → 従来ET2/関連回帰へ進む。共有helper利用集合は通常bin23、試用bin33、integration各11であり、全実名不減と成功を要求する。integration `codex_process` のA01–A11は独立EXEのG0対象とし、binの解除を流用しない。受入：全選択case成功、0件/timeout/unknown worker/残存なし、独立Post PASS。検証：exact1、helper利用caseの内部完了marker、AdminFalse、exit/kill/wait/fixture残存、source/EXE照合。title/namespace selector等の非利用caseへ内部markerを要求せず、既存別fixture検査を維持する。`cargo fmt --all -- --check` と `git diff --check` も必要である。

固定対象は従来8source（`src/main.rs`、`src/app.rs`、`src/qwen_reading.rs`、`src/app/qwen_settings.rs`、`src/app/qwen_settings_tests.rs`、`src/media.rs`、`src/ai.rs`、`src/app/qwen_reading_ui.rs`）＋新helperの9sourceと、読取対象 `tests/codex_process.rs`／`Cargo.toml`／`Cargo.lock` である。作者の変更前snapshotと修正後snapshotを保存し、runnerのbuild前後は修正後snapshotに一致させる。新helperは変更前不存在を記録する。G0は9sourceとinclude元のintegrationを含む固定組を確認し、従来8sourceだけの一致では解除しない。Cargo関連と他sourceの不変も確認する。

FX3の親解除は唯一の既存ET2-G0に従う。source、構成、EXEまたはmutationが変われば解除は失効する。通信/録音global、8入口/poison/worker・guard寿命とhelperの安全境界を静的に確認する。integrationにはmediaがないため録音3caseを追加/偽装せず、binの録音3対照とintegration自身の通信A群を別に証拠化する。最初の予期しないFAILで停止し、oracleを合うように変えない。

本限定Post後も、[既定ET3 gate](early-trial-plan.md)の通常/試用 `cargo test --all-targets --locked`、両release build、関連acoustic-analysis/qwen-audioのall-targets/no-default-lib/build/fmt、native、最終Post/提供は未完了として残す。全体実行に含まれる新EXEも親の有効なG0対応が必要である。コンパイル成功を全体PASS、限定Postを提供許可と扱わない。

## リスク・停止・次handoff

| リスク | 対応/停止条件 |
| --- | --- |
| import差替で作者marker同hashが失われる | 許可されたimportだけの差分と新hashを独立確認。assert/body/name差があれば作者へ戻す |
| helper重複定義とglobal複製の混同 | sourceは一つのstateless helperに限定。通信/録音globalは原位置不変。別global化は拒否 |
| 相対path/可視性/実名がintegrationで変わる | designerによる具体配置確認、両all-targets no-runと新list集合を要求。crate名推測で実名を書き換えない |
| compile PASS後に実mic/API到達 | Preに加えて新G0、固定EXE解除、安全対照順を維持。旧解除・自動フォールバック禁止 |

所有とmodule配置の未決定は解消した。外部仕様の追加判断は現時点で検出していない。未了は独立Pre判定、fixture修正、compile/runtime/Postである。次handoffは親からreviewer `gpt-6-astra/high` へ本書最終hash・上記設計hash・基準HEADとsource snapshotを渡すことである。Review class=設計/規約/信頼性、Requested Gate=Pre-implementation、scope=ET3-FX1限定、外部契約/本体/品質gate不変とする。Pre PASS後にtest author `gpt-6-astra/high`、runner `gpt-6.1-sol/medium`、独立Post reviewer `gpt-6-astra/high` の順でdispatchする。親の承認/dispatchをこのplannerが代行しない。

## 今回の文書検証・測定

AGENTS/development-team/KPI、installed `agent-skills:planning-and-task-breakdown` と参照Definition of Doneを全文読了し、工程を計画だけへ適用した。既存未完了 `tasks/plan.md` / `tasks/todo.md` の保存規則に従い、親が割り当てた本書だけを作成した。helper/source再読は修正案の3file境界と同体条件を決めるためであり、runnerのcompileを重複実行していない。

子 `/root/trial_delivery_fix_plan`、指定 `gpt-6-astra/high`。開始側観測の週枠usedPercent=59、windowDurationMins=10080、resetsAt=1792496608。終了側観測20:20:39 JSTも59/10080/1792496608である（最終応答前まで）。入力/cache/出力/推論counter、log session ID、親子包含、provider返却model/effortは未取得である。共有週枠から本工程消費を推定せず、表示差0を無消費と扱わない。文書検証は `git diff --check` 成功、ローカルMarkdown参照6件の存在確認成功、新規未追跡文書の空白/conflict marker直接点検成功である。source/Cargo/test実行0、既存dirtyを保持した。

設計後精緻化の再開/終了側週枠は60→60%、window10080分、reset1792496607（20:28 JST、最終応答前）である。上記token/返却metadataの欠測は継続する。設計hashを実物で照合し、文書diff check、直接空白/conflict検査、ローカル参照6件を再確認した。source/tests/tools/Cargoのtracked差分0。今回は本書だけを更新し、設計・旧計画・current/usageは変更していない。
