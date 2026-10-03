# QWEN-SETTINGS-001 作業リスト

状態: 設計・実装・自動検証・合成native確認・両release生成完了（2026-10-03）。親がP-S01/P-S02、6地域、役割別モデル指定を採用した。受入基準・モデル指定・検証コマンドは[plan.md](plan.md)を参照する。下の未チェック項目には初期の詳細作業分解も残るため、最終判定はresults.mdとrunner.mdの実施証拠を正本とする。

## 現在の完了境界

- [x] planner計画、独立外部仕様、親設計、独立仕様・設計レビューを実施し、キャンセル仕様を統一。
- [x] 共通契約の不一致結果、6地域、接続先変更時のキー再入力を実装。
- [x] 本体のAI接続画面、専用保存・取消し、standalone地域選択、READMEを更新。
- [x] 独立テストで適合不一致のREDとcore契約19件・接続2件のGREENを確認。
- [x] 最終レビュー指摘のfocus復帰・旧v1純粋decodeと独立追補テストを実装。
- [x] 最新固定ソースの本体20設定テストを含む全target、release/debugを確認。
- [x] packageの資格情報8件を含む全target、core-only、feature check、release/debugを確認。
- [x] 最新標準/狭幅の合成native画面と追跡独立レビューを確認。
- [x] results/current/KPIの証拠・未検証範囲を確定。

release実機操作、各リージョンの契約・実接続、利用者音声の判定精度は別受入である。実API送信とcommit/pushは今回行わない。

## 初期の詳細分解（計画時点の記録）

以下は計画作成時のチェック状態を保持した履歴である。現在の進捗は上の「現在の完了境界」、試験の正本はrunner.mdとresults.mdを参照する。

### P0 外部仕様を確定する

- [x] plannerが規約、7役工程、planning skillとDefinition of Doneを読み、dirty状態を確認する。
- [x] 要求R1–R6、対象外、依存関係、各単位5役のモデル/effortを暫定記録する。
- [ ] 親が暫定計画を確認し、spec author `gpt-6-astra/high` を起動する。
- [ ] 親の公式region/model調査とResponseInvalid調査をspec.mdの受入条件へ取り込む。
- [ ] 独立仕様レビューとユーザー承認範囲の確認後、plannerがplan/todoを更新する。

## P1 設計を確定する

- [ ] designer `gpt-6-astra/high` がregion/host/credentials、保存・取消し・失敗、late responseの状態表と所有ファイルを定義する。
- [ ] reviewer `gpt-6-astra/high` が設計を独立確認し、重大未決事項を解消する。
- [ ] plannerが各単位を5ファイル程度以内へ確定し、試験名・正確な所有・standalone影響を更新する。

## U1 応答分類を改善する（中・高リスク、P1依存）

- [ ] 独立fixtureで正当評価不能、通常評価、不正応答を分離し、変更前失敗を記録する。
- [ ] R1を実装し、変更後焦点試験と既存応答fixture回帰を通す。
- [ ] 原因未確定のエラーを不一致と断定していないことを独立レビューする。

## U2 地域・資格情報を分離する（高・高リスク、P1依存）

- [ ] 許可表、Tokyo互換、別地域キー非流用、未知値、資格情報失敗の独立ケースを作る。
- [ ] R3を実装し、焦点試験を通す。U1の共有ファイル編集と重ねない。
- [ ] 認証境界を独立レビューする。

## U3 設定transactionを統合する（高・高リスク、U2依存）

- [ ] 保存/reopen、cancel、保存失敗、取消後late responseを独立ケース化する。
- [ ] R4を実装し、active値と編集中値、資格情報の整合を焦点試験で確認する。
- [ ] 失敗時の復元と既存接続の保持を独立レビューする。

## U2b standalone消費者へ反映する（中・高リスク、U2依存）

- [ ] 共通APIに合わせてstandalone GUIを変更し、地域と資格情報の表示・操作を検証する。
- [ ] 独立の新規ケースとnative証拠で旧Tokyo互換と地域変更時のキー非流用を確認する。
- [ ] U1/U2と同じhelperが5本の所有ファイル内で順次実装し、独立レビューを通す。

## Checkpoint A: 接続の整合

- [ ] U2/U3の受入と関連試験・レビューを通し、仕様変更があれば親に戻して確定する。
- [ ] 共用packageのcore-onlyと本体利用featureの検証を通す。

## U4 設定画面を移す（中・高リスク、U3依存）

- [ ] AI接続内でCodex/Qwenを区分し、音読dialogから誘導するR2の観測ケースを固定する。
- [ ] 設定画面移設、狭幅/標準幅、未設定/設定済/エラー表示を確認する。
- [ ] native証拠と独立レビューで送信確認・取消し・Codex非退行を確認する。

## U5 導入条件を明記する（低・低リスク、仕様確定依存）

- [ ] Node.js/npm/Voltaの導入方式別条件とFFmpeg/ffprobeの音声形式別条件を照合する。
- [ ] R5をREADMEへ反映し、リンクと隣接導入文書との整合を確認する。
- [ ] 独立レビューを通す。新規コードテストは作らない。

## Checkpoint B: 最終統合

- [ ] 本体fmt、all-targets locked tests、wordweave5 releaseを通す。
- [ ] package fmt、all-targets locked tests、core-only tests、windows-credentials check、standalone releaseを通す。
- [ ] 本体・影響するstandaloneのnative画面証拠を保存し、mockと実API未検証を区別する。
- [ ] 固定対象差分の独立横断レビューで重大指摘を解消し、関連チェックのみ再実施する。
- [ ] 親がresults.md、短いcurrent記録、KPIの取得値/欠測を記録する。
- [ ] R1–R6の受入結果、未検証範囲、公開していないことを報告する。
