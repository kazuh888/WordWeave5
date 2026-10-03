# QWEN-SETTINGS-001 内部設計

2026-10-03。親が設計担当（指定Astra/high）として作成。親はspecのP-S01/P-S02と地域表示順を採用。独立仕様・設計レビューのキャンセル不整合を下記のとおり修正した。暫定計画に追加されたrelease-native要件は実施未確認として残し、既存debug合成native確認とrelease buildで確認済みとしない。

## 共通契約

- `Region` は6地域だけの列挙型。`ALL`、`label()`、`id()`、`example_url()`を公開する。既定はTokyo。
- `ApiHost::parse` は6地域のWorkspace専用HTTPS URLを検証・正規化する。`region()`で地域を返す。`parse_for_region(base, region)`は選択との一致も検証する。ポート、userinfo、query、fragment、偽suffix、複数workspace label、未知pathは禁止する。
- 既存 `TokyoHost` は東京限定の旧入口として保持し、`From<TokyoHost> for ApiHost`で移行する。東京限定型の意味を黙って変えない。
- `Connection::new` は`impl Into<ApiHost>`を受け取る。`host()`は`&ApiHost`。`with_host`は`Result`へ変更し、同じcanonical URLでしか保存済みキーを引き継げない。異なるWorkspace/地域には新キーを明示入力する。
- Windows credential record v1のhostから地域を導出し、保存書式・本体/standalone namespaceを維持する。既存Tokyoレコードを読み替えて書き戻さない。active接続は保存成功後だけ更新する。
- `Assessment::ReferenceMismatch` (`reference_mismatch`)を追加する。既存6fieldを維持し、heard_text/summary非空、unassessable_reason=null、strengths/improvements空に限定する。意味の大きく異なる発話をモデルへこの状態で返すよう要求する。読み誤りや一部の発音差だけを不一致にしない。JSON例をpromptへ追加する。
- `Unassessable` は従来通りheard_text=null/理由あり/空配列。不正JSON/不整合fieldは引き続き失敗。ResponseInvalidの説明に「音声の良し悪しや例文との一致は判断できない」を追加する。原応答をログへ追加しない。
- mismatchは有効な結果だが発音評価成功ではない。両hostで例文と聞き取った英文を比較でき、正しい例文で録り直す導線を示す。再送は人の明示操作のみ。

## 本体設定とUI

`src/app/qwen_settings.rs` を独立UIモジュールとして追加する。`QwenConnectionEditor`は注入可能なCredentialStore、保存済みConnection、draft地域/host/Zeroizingキー、error、破棄確認状態を所有する。Serialize/Debugは実装しない。

| 操作 | 結果 |
| --- | --- |
| AI接続でQwen設定を開く | 本体namespaceから読取。読取失敗は固定エラーを表示、上書きは行わない |
| 編集/地域変更 | draftだけ変更。旧キーは表示しない。地域変更時はhostと入力中キーをクリアし、旧宛先を新地域に書き換えない |
| Qwen接続を保存 | 地域/host/キーを検証。blankキーは同一host時のみ維持。単一CredentialStore.save成功後に閉じ、保存成功を表示。通常settings draftには触れない |
| 保存失敗 | modalを保ちエラー表示、保存済み接続とdraftを維持 |
| キャンセル | 他の終了要求と同様、dirtyなら破棄確認。戻る場合はdraft/入力キー保持、破棄確定時だけ閉じる。保存済み接続/通常settings不変 |
| Escape/外クリック/OS閉じる | dirtyならmodal内で「破棄して閉じる／編集を続ける」。OS閉じる要求はCancelCloseとし、設定dialogだけ閉じる。アプリ終了は利用者が再実行する（学習等の既存guardを迂回しない） |

modal中は背景入力・通常settings保存/取消し・画面移動を遮断する。Qwen専用保存と通常settings保存の分離を常時説明する。AI接続カードはCodex認証説明の適用範囲を限定し、Qwenが音読評価専用であることを明示する。

音読dialogから旧設定formと日常的な設定ボタンを除く。未設定の場合だけ「設定のAI接続を開く」を表示する。移動前に、この画面の音声・結果が破棄されることを明示する。録音/読込/評価中は移動不可。クリックでdialog.closeし、SettingsのConnectionへ移動する。毎回の送信前確認は維持する。

## 所有境界と検証

- helper実装: `tools/qwen-audio/src/{credentials,provider,error,lib,gui,main}.rs`。core契約とstandalone消費者を直列実装する。
- 親実装: `src/app/qwen_settings.rs`、app.rs、settings_ui.rs、qwen_reading_ui.rs、qwen_reading.rs、visual_check.rs、関連文書。親は読み込みのみの調査後、上記core契約に合わせる。
- 独立テスト著者: 新規core契約テストと本体設定テストモジュール、仕様変更に直接対応する旧API期待値の更新。productionと同時編集しない。
- 認証/保存境界は偽storeで検証する。native確認はdebug専用隔離hookの合成画面を使い、release build成功と区別する。利用者キーは読まない。実API送信なし。
- coreと両hostの契約・全体試験、release作成。既存fmt不適合はbaselineとして分離、変更ファイルだけ整える。全体fmtで無関係ファイルを書き換えない。

独立レビューでは、隔離debug nativeをrelease nativeと同一視しない点を指摘された。既存hookをreleaseへ公開する変更は行わず、release実機操作は未検証の受入残として報告する。実装・自動検証の完了と利用者の最終実機受入を分ける。
