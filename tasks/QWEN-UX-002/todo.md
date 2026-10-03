# QWEN-UX-002 作業一覧

## 実施状況（親の統合記録）

以下の暫定チェックリストは立案時の履歴である。実行後の正本はこの節と results.md / runner.md / review.md とする。

- [x] 外部仕様QU-AC-001～018と内部設計を作成し、親が入力・保存・通信境界を採用した。
- [x] U1/U2共通処理、U3/U4設定画面、U5音読画面を実装した。
- [x] 独立テストを追加した。追加テスト全件のpre-fix REDを観測したわけではない。
- [x] 独立レビューで2件のP2を検出・修正し、静的再確認と追加回帰を行った。仕様・設計の独立確認は実装と統合して行い、立案時の逐次review/計画改訂手順とは区別する。
- [x] 共通package all-targets147件、core-only100件、Windows資格情報構成checkが成功した。
- [x] 最終本体テスト451件、両release、最終debugのnative表示を確認した。通常/実最小幅と到達不能480stressを区別し、テスト時とdebug専用差分後の入力hashを別記した。
- [x] 最終証拠を独立レビューへ渡した。初回P2は回帰成功で閉鎖、最終画像の追加確認を記録する。
- [x] current/KPI/結果を確定し利用者へ引き渡す。
- [ ] 利用者の新UI・実API受入。受入前のSkill化は行わない。

2026-10-03。暫定、仕様・設計後に改訂。要求/モデル/所有/必須検証は [plan.md](plan.md) を正とする。QWEN-SETTINGS-001の実利用再発を追跡し、既存未完計画を保持する。

## P0–P2：仕様と設計

- [x] AGENTS、開発7役、KPI、planning skill全文/DoD、対象ソースと既存仕様を読み、git statusを確認した。
- [x] 暫定計画、5単位、要求、所有、全7役の明示モデル/effort、必須検証を記録した。
- [ ] 親が計画を確認し、公式URLとブラウザ参照の根拠を入力として固定する。
- [ ] 外部仕様担当Astra/highがspec.mdを作成する。probe対象、成功の意味、保存境界、エラー文言、音読操作を観測可能に定義する。
- [ ] 独立仕様review、親による利用者指示との照合後、plannerが計画改訂1を行う。
- [ ] 設計担当Astra/highがdesign.mdを作成し、snapshot/取消し、probe transport、error wire互換、UI構成を定義する。
- [ ] 独立設計review後、plannerが計画改訂2で正確な所有節/テスト名/単位を固定する。

## U1：応答を説明できる失敗経路（M、高リスク）

依存：P2。候補：provider.rs、error.rs、provider tests、qwen_response_contract tests。担当はplanのU1行。

- [ ] 独立テスト：AC01–02の正常/別英文/評価不能とSSE・終了・JSON/schema各失敗、安全なwire/キー非漏洩、json_object要求を固定する。
- [ ] 実装：根拠のあるJSON Object指定と失敗分類を行い、不正応答を別英文へ変換しない。
- [ ] 検証：独立RED→GREEN、共通packageの関連試験、局所review。実原因未確定を明記する。

## U2：Qwen接続試験の通信契約（M、高リスク）

依存：P2、U1 error契約。候補：connection_probe.rs、lib.rs、connection_probe tests。共有error/providerは直列移管する。担当はplanのU2行。

- [ ] 独立テスト：AC06–08の同一Workspace、GET/path、空の教材/音声、二重送信・redirect禁止、期限/取消し、status/model/応答境界を固定する。
- [ ] 実装：明示試験に限定し、資格情報・モデル一覧の到達確認を音声推論保証と混同しない。
- [ ] 検証：合成transportで全分岐、キー非漏洩、feature別build、局所review。実APIは使わない。

## Checkpoint A：通信と応答

- [ ] U1/U2の対象差分を固定し、関連package回帰・build・独立reviewを通す。
- [ ] 親が仕様との差分と未決事項を確認する。成功条件/保存契約の未決をUI実装へ持ち込まない。

## U3：両AI接続設定の共通UX（M、高リスク）

依存：P2。候補：settings_ui.rs、qwen_settings.rs、qwen_settings_tests.rs、harness_tests.rs。担当はplanのU3行。

- [ ] 独立テスト：AC05/09、Codex一般draft保存/取消し、Qwen独立保存/失敗/破棄、keyboard/focusを固定する。
- [ ] 実装：用途・認証・状態・編集・試験の構成を揃え、実際の保存境界を明示する。
- [ ] 検証：設定回帰、両画面の標準/狭幅/拡大native、Codex既存経路の局所review。

## U4：Qwen設定の試験状態（S–M、高リスク）

依存：U2、U3。候補：qwen_settings.rs、app.rs限定節、qwen_settings_tests.rs。担当はplanのU4行。

- [ ] 独立テスト：AC06–09、draft/saved snapshot、連打/編集/保存/閉じる/遅延完了、結果無効化、成功/失敗文字表示を固定する。
- [ ] 実装：明示試験・状態・再試験/取消し導線を追加し、保存だけでは送らない。
- [ ] 検証：隔離store/transport、保存失敗保持、native状態表示、局所review。

## Checkpoint B：設定

- [ ] U3/U4の統合回帰・build・reviewを通し、一般設定と資格情報に意図しない書換えがないことを確認する。
- [ ] 親が受入文言と操作を確認する。全体fmt既存不合格を合格と報告しない。

## U5：音読画面の操作と表示（M、中～高リスク）

依存：U1、P2。候補：qwen_reading_ui.rs、必要時visual_check.rs限定節、qwen_reading_tests.rs。担当はplanのU5行。

- [ ] 独立テスト：AC03–04/09の情報順、音声名/長さ、正確な上部注記、各phaseの主操作、確認・取消し・旧結果排除を固定する。
- [ ] 実装：目的→例文→操作→状態→終了、詳細展開、非実行時取消し整理を行う。
- [ ] 検証：全phaseのgeometry/文字、native標準/820×650/80～160%、keyboard/スクロール、局所review。

## 統合受入

- [ ] tests.md/results.mdと入力manifest/hashを担当者/親が記録し、全ACに新しい証拠を対応づける。
- [ ] 本体fmt、all-targets locked test、release buildを実施する。
- [ ] 共通packageのfmt、all-targets/no-default test、windows-credentials check、standalone releaseを実施する。
- [ ] 独立Astra/high reviewerが認証・保存・取消し・診断非漏洩・互換性を横断確認する。
- [ ] 指摘を各所有者へ戻し、修正の影響がある検証だけ再実施する。
- [ ] 親がREADME/current/KPIへ必要な現状だけを反映し、既存変更を保持したことを確認する。
- [ ] 改善UIを利用者へ提示し承認結果を記録する。承認前にSkillを作成しない。
- [ ] release-native/実APIの再発解消、実マイク/実Codex等の未実施受入を明示し、合成試験で代替済みとしない。

実キー・課金API送信、利用者音声を使う試験、公開、再委任は本一覧の操作許可に含まれない。
