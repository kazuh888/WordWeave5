# QWEN-INTEGRATION-001 作業リスト

2026-10-03 / U0後精緻化。[計画](plan.md)が要求、所有、モデル、検証の正本である。

- [x] 既存差分と規約/計画Skillを確認し、暫定計画を作成する。
- [x] U0: 外部仕様QI-AC-001～021とP1～P5を確定する。
- [x] U0: 独立仕様レビュー重大なし、親のP1～P5採用を確認する。
- [x] U0: 公開API、feature、専用credential、snapshot/取消し、UI寿命と試験口を設計する。
- [x] U0: 設計2指摘の独立再レビュー解消を親から受領し、対象hash・所有・署名を計画へ反映する。
- [x] 独立テスト: `ww-test-author` / `gpt-6-astra/high` が共通fixture、credential、root controller、UI試験を作成し、RED/未実装境界とAC追跡を引き継いだ。
- [x] U1: 起動capによりhelper dispatch不成立。親（同指定Sol/high）がfeature cfgとGUIなし共通libを実装した。
- [x] U2: U1後に直列でnamespace/with_hostと本体専用接続を実装した。
- [x] CP1: GUIなし/単独packageとcredential隔離を検証した。偽接続先と実OS/APIの受入を分離する。
- [x] U3: 親がsnapshot、owner nonce、Invalidated不可逆、cancel/send gate、reaper/restart待機を実装し、独立24回帰が合格した。
- [x] U4: 親がapp wiring/UI/native hookを実装した。最終native再確認はCP2に残す。
- [x] CP2: 二消費者の共通fixture、HTTP 0/1、native合成画面を検証した。実マイク/IME/DPI等は別受入である。
- [x] 親がAGENTSへ明示Qwen経路の限定例外を反映する。
- [x] U5: 利用手順を更新し、固定差分でroot415件/独立package90件の全test、両release、core check/integration_reuse3件、UI14件を通した（AC5）。
- [x] U5: fmt既存771 diff blocks不合格と新規/変更範囲の書式検査を別記した。全体fmt PASSとはしない。
- [x] U5: 独立レビューP2を修正し、作者の実描画/HTTP0・1回帰と独立再判定で解消した。表示修正後のnativeはCP2で確認する。
- [ ] 実接続準備（U0から親が並行）: 利用者が専用設定、選択音声、英文を揃える。会話でキーを収集しない。
- [ ] 実接続受入: 初回harnessの結果未取得を保持し、利用者のGUI送信で接続/助言品質・実metadataを別々に確認する（AC6 / QI-021）。
- [x] CP3: 開発完了と全体受入を区別し、未完受入/KPI欠測/現在地を記録した。限定変更と最終native5画像の独立レビュー合格、P1/P2なし。

checkbox細分化/進捗更新は今回のhandoff後に親へ移譲する。通常helper一名、test-author→implementerは直列。報告のみの常駐agentは置かない。
