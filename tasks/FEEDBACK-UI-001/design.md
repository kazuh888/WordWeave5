# FEEDBACK-UI-001 内部設計

2026-09-27。入力は親承認済みの [外部仕様](spec.md) と [計画](plan.md) である。設計担当の起動指定は `gpt-6-sol / high`（U1・U2・U4、U3を含む統合設計）であり、実行モデル・effortのメタデータは未取得である。本書は設計のみで、実装・テスト期待値・計画のゲートは変更しない。受入IDはすべて `FEEDBACK-UI-001-` を前置する。

## 1. 既存境界と変更所有

根拠: `src/store.rs` の `Settings` は `#[serde(default)]`、`Storage::load/save` は `Progress::validate` を経由する。`src/app/settings_edit.rs` は通常設定のdraft/baselineを持ち、`merge_settings` で現在値へ変更フィールドだけを重ね、保存成功後にcommitする。`Destination::Page/Close` と `guard_settings_navigation` が未保存移動を守る。`src/app.rs::update_ui` はclose要求、`tick`、chrome、page、各Windowの順で描く。`restore` と媒体付きバックアップ復元は別経路である。通知は `message` と attention/error文字列、詳細Windowで表示する。引用は `src/material.rs::validate_reason/validate_quote` が固定 `Source.snapshots` で判定し、`src/material_diff.rs` に既存の教材項目名がある。

| 責任 | 所有と最小契約 | 対応AC |
| --- | --- | --- |
| 永続値 | `src/store.rs`: `Settings` に `page_tint` / `input_tint` を追加する。各値は `TintChoice { rgb: [u8; 3], depth: u8 }`、既定 `([0,0,0], 0)`。`Progress::validate` で両depthを0–100に制限する。欠落キーだけdefault。不正型はserde読込エラー、不正範囲は既存validationエラーを返す。 | C01,C09 |
| 配色状態と計算 | 新規 `src/app/color_theme.rs`: `ColorEditor { baseline, draft, page_hex, input_hex, page_depth_text, input_depth_text, errors, leave_close }` を `WordApp` が1個だけ所有する。純粋関数 `blend(base, choice)`、HEX/整数parse、contrast比、`effective_tint()` を置く。`baseline` と `draft` は配色2値のみで `Settings` 全体の古いcopyを保持しない。 | C01–C07 |
| 一般設定との接続 | `src/app/settings_edit.rs` の既存 `Destination` に `ColorEditor` を追加し、既存3択を再利用する。通常設定の保存成功または明示破棄後だけ配色を開く。配色中の通常設定保存を入口でも拒む。 | C06–C09 |
| UIと終了 | `src/app/settings_ui.rs` に開始ボタン、`src/app.rs` に1フレーム1回のstyle適用・editor Window表示・終了guard。配色Windowはdrag可能で固定幅を画面幅へ縮め、本文だけスクロールし、操作列は折り返す。 | C02–C08 |
| 復元 | `src/app/settings_ui.rs` のJSON復元開始、`src/app/backup_ui.rs` の媒体付き復元開始/確定、`src/app.rs::restore` を配色中に理由付きで拒む。既に復元確認が開いていれば配色開始を延期する。 | C08,C09 |
| 局所入力外観 | 新規 `src/app/color_theme.rs` にTextEditだけを `ui.scope` で囲む薄い描画helperを置き、通常/hover/focus時の入力枠のみ変える。`frame(false)` の既存入力は外側Frameを入力面として塗り、枠を同じ基準にする。 | C03,C04 |
| 上限誘導 | `src/app/notifications.rs` が拒否原因に紐付く一時的な `DailyLimitNotice { day, text }` を保持し、表示/クリック時に確定 `progress.settings.ai_daily_limit` と当日 `ai_calls` を再判定する。`src/app.rs` の `launch_ai` と `reserve_generation` の実際の拒否箇所だけが登録する。`src/app/settings_ui.rs` が対象Sliderへ1回scroll/focus/強調する。 | L01–L05 |
| 引用診断 | `src/material.rs` は検査の順序・述語を維持し、検査失敗時の診断文だけを構造化する。`src/material_diff.rs` に許可pathから教材表示名への純粋な対応関数を追加する。`src/app/notifications.rs` は既存の選択可能・コピー可能な詳細本文をそのまま表示する。 | Q01–Q06 |

新規REST/API層、保存スキーマversion更新、汎用theme再構成、新しい依存は不要である。アプリ内の関数と型だけが境界である。

## 2. 配色のデータ、状態、順序

`TintChoice` は選択色と濃さを保持し、実効色は保存しない。RGB各成分は正整数演算 `((B * (500-d) + C * d + 250) / 500)` で得る。`d=0` は元の面のRGB、`d=100` は20%混合である。例: 白 + 青100 → `#CCCCFF`、`#F7FAFD` + 青100 → `#C6C8FD`。ページ地とWindow地は別々の既存baseを渡し、同色へ統一しない。入力面のbaseはその既存の地（通常TextEditの白、明示TINT等）を渡す。ただし全編集可能欄で実効入力色と共通枠の視認性を保つ。色変更が未保存なら `effective_tint()` は有効な配色draftを返し、無効なテキスト入力は最後の有効値を保つ。フォーカス中のHEX文字列を正規化してIMEを壊さず、受理時またはfocus離脱時に表示を大文字 `#RRGGBB` にする。

`page_hex/input_hex` と `page_depth_text/input_depth_text` はUIの途中入力だけを持つ。parseはHEXをちょうど `#` + 6 ASCII hex、濃さを10進整数0–100として受理する。色pickerの不透明RGB変更は対応する有効HEXとdraftを同時更新する。parse失敗時は欄ごとの説明を表示し、保存を無効にするが、取消しは常時可能とする。Sliderの1刻みも同じdepthへ反映し、文字入力を黙って丸めない。「標準色に戻す」は2組のdraft/UI入力を既定値へ変えるだけである。

| 状態/操作 | メモリと永続値の変化 | 描画と失敗時 |
| --- | --- | --- |
| 開始 | 通常設定dirtyなら既存3択を先に完了。`baseline = draft = progress.settings` の配色2値。1 sessionのみ。 | Windowを開き、開いたままページを移れる。再度開始操作は現sessionを前面にする。 |
| 有効変更 | 配色draftだけ更新。`progress.settings` とディスクは不変。 | 次フレームの全対象と後から開くWindowへ適用。無効テキストの間は最後の有効preview。 |
| 保存 | 最新 `progress` をcloneし、配色2値だけ上書き、`validate` → `Storage::save` → 成功後 `self.progress=next` → editor終了。 | `Storage::save` 失敗なら確定値/ディスクを成功扱いせず、draft/preview/Windowと再試行操作を残す。`credit_time` 等の別更新は最新値から保存する。 |
| キャンセル/×/Esc | 配色editorだけ破棄し、`effective_tint()` が確定値へ戻る。 | 学習回答、会話draft、一般設定値に触れない。色picker popupがEscを消費した時はWindowを閉じない。 |
| 通常終了 | 配色dirtyならcloseをcancelし、保存/破棄/終了中止の3択。保存失敗では終了しない。 | 保存または破棄後にCloseを再要求し、既存の録音・注釈終了guardも順に通す。色draftを`on_exit`/autosaveへ渡さない。 |
| 復元 | 配色editor中はJSON/媒体付き復元の開始・確定を止める。 | 復元後は既存 `reset_settings_after_restore` を通し、配色styleは復元確定値から再計算する。 |

一般設定draftは既存 `SettingsEditor` の所有である。配色session開始中は設定ページの閲覧と上限欄の誘導を許すが、フォーム/保存/倍率/音声preview操作をdisabledにして理由を見せる。`settings()` が再表示時に `begin_settings_edit` を呼んでもdraftは表示専用とし、配色保存の前後で通常設定を誤ってdirtyにしない。一般設定編集時に配色保存を始めた場合の `Destination::ColorEditor` は `Page`/`Close` と同じ既存3択結果を待つ。既存 `merge_settings` は独立値を保つため維持し、色保存は古い `Settings` 全体を戻さない。UIイベント・autosave・AI結果処理は同じUIスレッドで順次処理される前提で、非同期のAI/録音/保存処理へ配色draftの参照を渡さない。取消しはタスク取消し信号ではなく表示状態の破棄である。

## 3. 全画面適用と描画規約

`src/app.rs::update_ui` の冒頭、panel/Window生成より前に、現在の `ctx.style()` の他項目を保持したまま `visuals.panel_fill` と `window_fill` を配色で更新する。計算のbaseは毎回不変の旧色（page `#F7FAFD`、Window `#FFFFFF`）とし、直前フレームで変換済みの `ctx.style()` のfillを再混合しない。生成時 `new_with_storage` の一度限りのstyleだけには依存しない。`eframe` が生成callback後にstyleを設定し得るためである。全体の `visuals.extreme_bg_color` と `widgets.*.bg_stroke` は変更しない。前者は読み取り専用TextEditにも、後者はButton等にも波及する。編集可能TextEditだけを `ui.scope` で囲み、そのscope内の `extreme_bg_color` と入力枠strokeを変える。入力ごとの不変baseは、egui 0.31.1 のlight既定 `extreme_bg_color` または既存の明示base（設定の `ux::TINT` 等）であり、毎フレームの変換済みstyleから取得しない。`frame(false)` は外側Frameの不変baseを変換する。通常設定のSliderなど数値の文字編集があるwidgetも個別scopeで入力背景・枠を適用し、数値以外のslider外観やボタンには広げない。入力面と隣接面に対する本文4.5:1・枠/焦点3:1を満たす枠色を選ぶ。`ux::INK` 等の既存暗色文字を優先し、足りない場合だけ入力局所の高コントラスト色にする。disabledには既存無効表現を残し、通常入力と同じ強い有効枠にしない。`ui.add_enabled` / `ui.add_sized` の呼出結果、egui ID、focus、IMEイベントを変えない薄いhelperが必要である。既存の `controls::Button` と `ww_button` を使い、日本語の見える字形を両軸で中心に置く。

| 実在する地・Window | 適用方法 | 固定fillの分類 |
| --- | --- | --- |
| Home/Study/Deck/Words/Chat/Stats/Settings のページ地 | `CentralPanel` と内部の既定Panelへ `panel_fill`。チャットの内部 `CentralPanel::show_inside` と教材 `Frame::NONE` は親地を継承。 | `home_view.rs`/`home_art.rs` の装飾、`ux::panel` 白カード/TINT、学習の紙面、教材詳細の白カードは維持。 |
| 共通 `egui::Window` | `window_fill`。`notifications.rs` 詳細、`app.rs` 取り込み/JSON復元、`backup_ui.rs` 媒体付き復元、`conversation_trash.rs` 削除/ごみ箱/読取、`chat_ui.rs` 添付確認/タイトル/文脈/教材根拠と差分/削除教材/教材操作、`chat_media.rs` 画像/ファイルpreview/送信確認/添付編集、`audio_controls.rs` 録音取消、`run_history.rs` 実行記録、`dashboard.rs` 版情報、`settings_edit.rs` 未保存設定、配色editor。 | 通知内の読み取り専用白い本文、教材diff赤緑、警告色、添付画像/原本/筆跡/キャンバスは維持。 `chat_media_dialog_style` は文字だけ変更しwindow fillは上書きしない。 |
| 編集可能入力 | `app.rs` 教材base/各詳細、`study_ui.rs` 解答、`materials_ui.rs` 検索/教材編集、`vocabulary_ui.rs` 語彙/検索、`chat_ui.rs` composer/タイトル/memo、`chat_media.rs` 添付注釈、`settings_ui.rs` Codexパス/モデルと数値編集widget、配色editor HEX。 | `study_ui.rs` の解答Frame、`materials_ui.rs` 検索Frame、`vocabulary_ui.rs` の語彙/検索Frameは `frame(false)` の入力本体に代わる入力用外枠なので配色・枠を局所更新する。`settings_ui.rs` の明示 `.background_color(ux::TINT)` 2箇所は入力色で置換する。 |
| 入力対象外 | `settings_ui.rs` の診断report、`control_settings.rs` の表示専用TextEdit、`chat_ui.rs` の送信preview、`run_history.rs` の実行記録本文。実行記録はローカルcloneの表示であり、入力helperの対象から除くだけとし、現状の選択/コピー操作を変えない。 | ナビ白面、ステータス、音声操作帯、チャットの白いスレッドpane・composerカード・bubble、添付表色、ホームの意味色、`ux::TINT` はカード/操作面として維持する。その内部の編集可能入力だけは入力組へ適用する。 |

上記棚卸しは `rg` による実在Window/TextEdit/固定fillから作成した。隠れた固定fillが「地」か「カード」か曖昧な場合は、描画証拠で境界を決め、仕様対象を減らす変更なら仕様担当/親へ戻す。`ColorEditor` は設定を離れても動かせるWindowとし、狭幅では画面の可用幅/高さに収め、2組を縦積み、説明/エラーは折り返し、操作列は常に見えるfooterとする。タイトル/本文/ボタンの日本語字形、キーボード順、focus、色以外のエラー文を検証する。80%の標準サイズと160%の狭幅でWindow全体のbounds・本文最下部到達・footer操作を別々に確認する。

## 4. 上限通知の原因と誘導

文字列から上限原因を推測しない。拒否箇所 `launch_ai` と `reserve_generation` だけが `notify_daily_limit_rejected` を呼び、既存の注意通知と同時に `DailyLimitNotice { day: today(), text: self.message.clone() }` を記録する。`notify_error/notify_warning/notify_blocked/notify_result` が別通知を作る時はactionを失効させる。`message` を直接書く既存経路もあるため、表示条件に `message == action.text`、`notification_attention == message`、`fatal.is_none()` を含める。dayが今日、現在の確定limit `<1000`、今日のcount `<1000` かつ `count >= limit` を表示時とクリック時の両方で再評価する。未保存 `settings_editor.draft.ai_daily_limit` は読まない。通知を閉じてもactionは再実行を起こさず、翌日/保存後/別通知では条件不成立になる。同文の新しい拒否は新actionに置き換わる。

クリックは `Page::Settings`、`SettingsSection::Connection`、一時的な `daily_limit_focus_pending` を設定し、通知詳細を閉じるだけである。既存 `open_codex_path_guidance` のscroll/focus方式を借りるが、対象は `generation_settings_panel` のSliderである。設定内容を表示させ、描画後にSlider矩形の中心がclip内に入った一回だけfocus/強調し、pendingを消す。狭幅のcategory menuや設定のscroll領域で見切れたままpendingを消さない。配色中なら設定ページを見せつつ一般設定欄をdisabledにし、理由と配色保存/取消しへの案内を表示する。誘導はlimit変更、保存、AI再試行、count加算を呼ばない。既存Slider 0–1000 と通常設定の保存成功後から効く順序を維持する。

## 5. 引用診断の組立てと失敗順序

`validate_reason` の検査順は path形式 → 実在参照 → 理由空白/2000超 → 引用件数1–10 → 配列順の `validate_quote` のままである。`validate_quote` の順も存在する固定snapshot/role → 空白のみ → 2000超 → `original.contains(&quote.quote)` のままである。最初に失敗した理由だけを診断へ渡し、成功/不合格の判定値をUI用表示から導かない。`build_response` は「教材案を作成できなかった。登録していない」、既存 `Draft::validate_evidence` は「この案は登録できない」と段階を渡す。前者は `Request.source.snapshots`、後者は `Draft.source.snapshots` だけが元文の所有者で、現在のConversationを参照しない。

失敗時の文字列は「結果→対象→原因→根拠→比較→対処→技術情報」の見出し順で作る。path名は `material_diff` に既存ラベルを共有し、配列pathなら実在する0始まり添字を1始まりの「N件目の追加例文/言い換え」に変換する。未知/不正pathは「対象項目を特定できない」とし、技術欄に安全なpreviewだけを残す。理由番号は `enumerate()+1` であり教材/会話番号と混同させない。turnは `checked_add(1)` 成功時だけ表示し、不存在snapshotは「固定元発言を取得できない」、不正roleは「話者を特定できない」と記す。別turn/roleから文を補わない。比較欄の「AIが示した引用」と「作成要求時の元発言」はそれぞれ独立した先頭240 Unicode scalar値previewと省略標識を使い、制御文字はJSON文字列escapeを保つ。引用元がない時は元文欄に取得不能と記す。比較・コピーはpreviewであると明記し、検査自体は全文の厳密containsで行う。通知詳細の selectable text・`詳細をコピー`・本文scrollと固定footerを再利用し、引用をMarkdownや操作として解釈しない。

## 6. 受入ID別テスト入口と作業分割

独立テスト作成者は新helperの存在だけを確かめるcompile-failをRED証拠としない。既存 `Storage::at/load/save`、`WordApp` の `fixture/frame/update_ui`、既存の `Request::build_response`/`Draft::validate_evidence` を通して、実装前は仕様に反する結果を観測し、実装後同じ操作でGREENを確認する。隔離fixtureと合成会話だけを使う。

| AC | 独立した入口と主な証拠 |
| --- | --- |
| C01,C02,C09 | 合成設定JSONを旧キーなし/不正型/範囲外で `Storage::load`、実画面でHEX/濃さ変更、`Storage::save/load`。実効RGB、2組独立、保存失敗・autosave非混入を比較。 |
| C03,C04,C05 | `fixture/frame` と既存各ページを描画。TextEdit空/入力/disabledの描画色・枠・focus、ページ→後開きWindow、IMEイベント前後の文字、80%/160%の矩形・日本語の字形を確認。style値だけを合格証拠にしない。 |
| C06–C08 | 既存設定のdirty→配色開始3択、会話/学習更新中のpreview、保存先故障→再試行/取消、×/Esc/close、JSON/媒体付き復元の両入口。`Progress` と隔離ディスクを比較。 |
| L01–L05 | 当日count/確定limit/draft limitをfixtureへ注入し、学習送信と予約拒否を実操作。通知詳細のボタン有無、翌日・別通知・保存後・クリック直前変化、Sliderの一度だけfocus/scroll、countとdraft保持を検査。 |
| Q01–Q06 | 合成 `Request::build_response` と保存 `Draft::validate_evidence` に未知path/不正role/欠落turn/2000境界/240・241scalar/複数理由を投入。通知詳細で長文の末尾とコピーfooterへ到達し、表示・コピーの診断を照合。登録や再試行が起きないことをfixtureで確認。 |

後続plannerは以下の所有単位へ分割し、本体とテストを合わせ各5ファイル以下に固定する。複数単位が同じファイルを使う時は直列にし、`cfg(test)` と本体は節所有を分ける。

| 候補単位 | 本体所有候補（テスト別ファイルの余地） |
| --- | --- |
| U1a 永続型・計算 | `src/store.rs`、新規 `src/app/color_theme.rs`、`src/app.rs` のmodule/field配線（3） |
| U1b session・一般設定・復元guard | `src/app/settings_edit.rs`、`src/app/settings_ui.rs`、`src/app.rs`、`src/app/backup_ui.rs`（4） |
| U2a 共通styleとホーム/学習/教材入力 | `src/app.rs`、`src/app/color_theme.rs`、`src/app/study_ui.rs`、`src/app/materials_ui.rs`（4） |
| U2b 語彙/チャット入力 | `src/app/vocabulary_ui.rs`、`src/app/chat_ui.rs`、`src/app/chat_media.rs`（3） |
| U2c 設定入力と全Window確認 | `src/app/settings_ui.rs`、`src/app/visual_check.rs`、必要なら残る局所Windowファイル（2–4）。全Windowは主に共有styleであり、個別修正を根拠なく増やさない。 |
| U3 上限通知 | `src/app.rs`、`src/app/notifications.rs`、`src/app/settings_ui.rs`（3） |
| U4 引用診断 | `src/material.rs`、`src/material_diff.rs`、必要な時だけ `src/app/notifications.rs`（2–3） |

`src/app/visual_check.rs` の既存 `--ui-check` は合成storageとページ/ダイアログ指定を持つ。配色保存前・変更後、上限拒否、長文引用診断の合成シナリオだけを追加し、標準1150×950/80%と狭幅の実効幅・高さ/160%でWindows native screenshotを採る。既存の実ユーザーデータや実Codexは使わない。既存テストの結果を今回の結果として流用しない。

## 7. リスクと引継ぎ

- 最大の設計リスクはTextEdit枠を共通visualsで変えてButton等へ波及させること、`frame(false)` の外枠を見落とすこと、媒体付き復元が `restore` を通らないことである。局所枠・上記の面棚卸し・復元2経路の入口と確定時guardで閉じる。
- `message` は既存コードで直接代入されるため、通知actionは原因の明示登録と表示/クリック時の文字列・日付・確定数値再判定を組み合わせる。新たな直接代入経路が同じ通知文を作る場合は通知メタデータの失効をレビューする。
- 仕様の対象面から除外する必要が見つかった場合、または診断の原因を欠落情報から推測しないと表現できない場合は、設計者が利用者向け挙動を決めず仕様担当/親へ返す。現時点で未解決の仕様質問はない。

引継ぎ: 親は本書の独立設計レビューを経て、plannerにU1/U2の再分割・各単位の5ファイル以下の本体/テスト所有を確定させる。テスト作成者は上表の既存入口で期待値を固定し、実装担当は `src/store.rs` の互換性・最新Progress上書き順・入力局所枠・復元2経路・通知の原因識別・引用の最初の失敗順を保持する。検証はplan指定の焦点RED→GREEN、全target test、release build、差分/整形、隔離Windows標準/狭幅描画、独立reviewである。
