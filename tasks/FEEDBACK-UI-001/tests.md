# FEEDBACK-UI-001 独立テスト設計・RED引継ぎ

2026-09-27。根拠は親承認済み `spec.md` の C01–C09、L01–L05、Q01–Q06 と `design.md` である。テスト作者の所有は本書、隔離 fixture、承認済み `cfg(test)` 節のみである。実装判定や期待値を本体都合で変更しない。

## 境界と fixture

- 合成 `Progress` / `Conversation` / `Request` と `std::env::temp_dir()` 内の固有ディレクトリのみを使用する。`Storage::at` は実際のローカル保存・読込を通す。実教材、実学習記録、実Codexは使わない。
- 第一段階REDは既存入口に限る。`cargo test --locked feedback_ui_ -- --nocapture` で単一フィルター実行する。コンパイル失敗やfixture誤りはREDの受入証拠としない。
- 新しい配色editor/stateは既存APIがないため、ボタン・HEX欄・Slider・保存/取消の実操作テストと配色native previewを実装後に独立追補する。型・helperの存在だけを試すplaceholder testは作らない。
- Windows native描画の基準は標準1150×950/80%、狭幅820×650/160%である。OS scaleを含む実効 `screen_rect`、全Windowの矩形が画面内に収まること、本文最下部への実スクロール到達、固定footerの常時可視を別に記録する。生成されたがclip外の文字を表示済みと判定しない。

## REDケースと期待する失敗

| テスト | 対象AC | 既存入口・期待するassert失敗 |
| --- | --- | --- |
| `feedback_ui_legacy_palette_keys_default_without_changing_other_settings` | C01,C09 | 旧JSON保存→`Storage::load`。2組とも黒/濃さ0として読め、他設定/ディスク不変。現行は新キーがなく、直列化後の既定値assertが失敗する。 |
| `feedback_ui_invalid_persisted_palette_is_rejected_without_repair` | C02,C09 | 合成JSONの配色欄を範囲外/負数/小数/文字列/不正RGBとし`Storage::load`。エラーかつ元バイト保持。現行は未知キーを無視する。 |
| `feedback_ui_daily_limit_refusal_shows_settings_action_without_changing_drafts_or_count` | L01,L03 | 当日確定limit=count=1、未保存draft limit=1000で`reserve_generation`を拒否させ通知を描画。導線表示と入力/count保持。現行には導線がない。 |
| `feedback_ui_daily_limit_action_disappears_for_stale_or_unrelated_notice` | L01,L05 | 実拒否後にcount<limit、limit=1000、count=1000、別注意通知へ変更して再描画。いずれも導線なし。正例と対にして誤表示を監視する。 |
| `feedback_ui_generation_quote_error_explains_seventh_reason_and_first_failure` | Q01,Q02,Q04 | 合成`Request::build_response`へ有効6理由、7番目の強調記号を欠く引用、8番目の別誤り。結果/対象/原因/根拠/比較/対処、7番目/説明・使い方/4組目/固定元発言を示し、8番目を混入しない。現行はpath等の技術文が先行する。 |
| `feedback_ui_saved_draft_quote_error_identifies_fixed_source_and_nonregistration` | Q01,Q02,Q06 | 正当な案を合成して引用を改変し`Draft::validate_evidence`。保存案の登録不可、固定原文、再試行を示す。現行は生成段階との差を示さない。 |

既存MATERIAL-QUOTE-001 U1–U5テストは保持する。特に引用の2000/240文字、制御文字escape、厳密連続部分列、選択した固定snapshot、登録前承認を新診断でも再確認する。ただし既存成功ログを今回のGREEN証拠へ流用しない。

## 受入ID別の追加証拠

| AC | 必要な自動テスト・描画・実操作証拠 |
| --- | --- |
| C01 | RED旧値 + 2組片側変更の独立性、黒/白/青の濃さ0/100、白+青100=`#CCCCFF`、`#F7FAFD`+青100=`#C6C8FD`、保存再読込。 |
| C02 | RED永続値 + 実HEX欄`#12`/`#GG0000`/8桁と濃さ-1/101/小数の途中編集。最後の有効preview、理由表示、保存不可、修正後保存、常時取消可能。 |
| C03 | 空の教材基本語欄、通常入力と送信中disabled入力を実描画。黒/白/赤/緑/青×濃さ0/100で本文対実効面4.5:1、有効枠・focus対隣接面3:1を計測し、disabledの操作不可を区別。 |
| C04 | 全7ページと設計3節の全Window一覧をnativeで巡回し、地/入力を比較。白カード、紙面、読取専用本文、意味色、教材/画像原本、筆跡と入力/IME値を前後比較。キーボード操作と両scaleでbounds・文字・エラー表示を確認。 |
| C05 | editorを開いて片側変更→Home/Study/Deck/Words/Chat/Stats/Settings→後開き通知/教材Windowへ移動。次描画反映、ドラッグ可能、再開時同一draft。 |
| C06 | 新配色のみ保存し実`Storage`再読込。会話/学習更新を間に挟み保持。保存先を隔離故障にしてエラー・確定値/ディスク不変・preview/draft保持→復旧後retry成功。 |
| C07 | `キャンセル`、×、editor focus時Esc、picker popup時Esc、終了時保存/破棄/終了中止を各操作。確定配色復帰、学習/会話draft保持、保存失敗時終了禁止、強制終了後旧配色。 |
| C08 | 通常設定dirty時の保存して進む/破棄して進む/編集継続と保存故障。editor中の通常設定編集・保存/JSON復元/媒体付き復元の開始と確定をガードし、上限欄閲覧だけ許容。 |
| C09 | RED旧/不正JSON + 学習/会話autosave・通常設定保存で未保存配色非混入。標準色操作は未確定。教材/記録/録音/筆跡/認証・モデルの`Progress`と隔離媒体を保存前後比較。 |
| L01 | RED予約拒否 + `launch_ai`学習拒否。日付/count/確定limitの境界、表示時とクリック直前の再判定、未保存draft非適用。 |
| L02 | 通知の実クリックから設定Connectionの対象Sliderがclip内へ一回scroll/focus/強調されること。別通常操作でfocusを再取得しない。標準/狭幅で契約枠との説明を保持。 |
| L03 | RED入力/count保持 + 通知クリック前後の`Progress`/案/会話/解答比較。AI再試行・登録なし、配色中の通常設定保護。 |
| L04 | 既存0–1000整数境界、保存前/失敗/取消後は旧limit、成功後の手動再試行だけ新limit。count未満/同値の拒否、新通知条件更新。 |
| L05 | RED陳腐/別通知 + 翌日、上限1000、count>=1000、契約/通信/認証/引用失敗、通知閉鎖で導線なし・再生成なし。 |
| Q01 | RED7番目/保存案 + 診断見出し順、理由/項目/発言/話者対応、最初の失敗のみ。 |
| Q02 | RED固定原文 + snapshot欠落・不正role・別往復の負例。現在会話で代用しない。 |
| Q03 | 240/241 Unicode scalar、絵文字・改行・制御文字を独立preview。省略印は241以上だけ、コピーにも抜粋と示す。比較は省略前全文。 |
| Q04 | 既存厳密引用テストを今回実行。空白のみ、2000/2001、引用1/10/11、理由100/101、path形式/参照、`**`/大文字/空白/改行差と検査順を維持。 |
| Q05 | 長文・狭幅の通知Window全bounds、原因/対処まで実スクロール後の可視rect、コピーfooterの常時可視/操作、選択可能文字とcopy結果。clip外のfoldを即時assertしない。 |
| Q06 | 合成`Request`/保存`Draft`拒否時に教材/記録不変。診断閲覧・閉鎖・コピーで再試行/登録なし。成功時は案・差分・利用者承認を通し、追加時は既存内容/復習状態保持、訂正は別操作。 |

## 実装後に必要なtest hook

`src/app/visual_check.rs` の既存 `--ui-check PNG_PATH` と同じ合成storage内に、`--palette-preview`、`--palette-saved`、`--daily-limit`、`--quote-long` を追加することを提案する。各フラグは描画前に合成状態のみ設定する。配色previewは確定色とdraft色を別値にし、保存後fixtureは実`Storage::save/load`を通す。上限は実`reserve_generation`拒否を経由し、引用は合成`Request::build_response`失敗文を通知へ渡す。長文は日本語/英語/絵文字/制御文字を含める。いずれも実Codex/実学習データを使用しない。UI testではWindowの実rectとclip rect、footer button rect、スクロール後の原因/対処の可視rectを取得できる最小限の観測点が必要である。`ColorEditor`等の本体内部を直接検証する専用公開APIは要求しない。

## 第一段階の実行結果と凍結

実装前の `cargo test --locked feedback_ui_ -- --nocapture` はcompile成功、lib 4件が挙動assertでRED（C旧キー欠落、不正値受理、Q生成案/保存案の診断不足）。Cargoはlib失敗後bin targetを実行しない。旧設定fixtureはこの実行後に、新型追加後も欠落キーを保証するためJSONから2キーを明示除去した。変更後のlib再実行はU1a実装後のGREENで行う。初回REDと修正fixtureの結果を混同しない。

`cargo test --locked --bin wordweave5 feedback_ui_ -- --nocapture` の初回は正例の描画テキストが空でfixture不備であった。既存`harness_tests::frame`による実画面描画へ修正した再実行では正例が旧通知文と詳細Windowを表示しつつ導線assertでRED、負例1件はPASSした。さらに正例の未保存設定draftを現実の設定ページ上へ置き、`cargo test --locked --bin wordweave5 feedback_ui_daily_limit_refusal_shows_settings_action -- --nocapture` を実行した。compile成功、旧通知文が描画され、導線がない挙動assertでRED（exit 1）である。

現在の第一段階6件は `src/store.rs`、`src/material.rs`、`src/app/notifications.rs` の `cfg(test)` 節に固定した。新UI/stateの操作、全画面適用、保存retry/cancel、スクロール/コピー、native描画は未実装ゆえ未検証であり、実装後独立追補とrunnerの対象である。`cargo test --all-targets --locked` とrelease buildはrunnerの責任範囲である。

## U1a/U1b 実装後の独立追補

事前REDではない。U1a/U1b本体の後、`src/app/color_theme.rs` と `src/app/backup_ui.rs` の `cfg(test)` に `feedback_ui_palette_` で始まる7件を追加した。既存の実local fixtureと隔離ディスクを用い、新helperの存在だけを判定していない。

| AC | 追加した証拠 |
| --- | --- |
| C01,C02 | 不透明RGBの0/100混合と白/旧ページ地の青100、HEX大文字化・不正文字/桁・濃さ整数範囲、白黒contrastの端点。無効欄を持つeditorは最後の有効previewを保ち保存を拒否。 |
| C06,C09 | 配色preview中の実autosaveで配色2値はディスクへ漏れず、会話draftは保存される。別設定/学習時間更新後の配色保存はその最新値を保持し、2組だけ確定する。保存先を隔離故障へ置き換えると確定・元ディスクは不変、復旧後retryで保存、次session取消しで確定値へ復帰。 |
| C07 | 実close要求がdirty配色を保留し、「終了を中止」「破棄して終了」「保存して終了」のボタン分岐を操作。保存不能時にcloseしない。 |
| C08 | 通常設定dirtyから配色開始は3択を表示し、編集続行/明示破棄/保存成功に応じて開始可否と既存設定値を検査。配色中の通常設定保存・JSON復元を拒否し、媒体付き復元confirmの復元ボタンは無効。JSON/媒体復元確認中からの配色開始も拒否。 |

U2の全画面style適用、TextEdit枠とcontrastの実描画、Window bounds/狭幅、picker popup/Escの実機操作はこの追補の合格範囲外である。これらはC03–C05/C07の残証拠として維持する。

実行: `cargo test --locked --bin wordweave5 feedback_ui_palette_ -- --nocapture`。最終実行はcompile成功、7件PASS、0件FAIL、exit 0である。初回6件中の媒体復元テスト1件は、アプリ起動時にメモリだけへ加わる `deck_versions` を起動前のディスク値と比較したfixture誤りで失敗した。比較対象を復元前後のディスクsnapshotへ修正して6/6 PASS、その後C07の終了時保存/保存失敗分岐を追加して最終7/7 PASSとした。fixture不備を本体不具合やREDとして報告しない。

## U4 実装後の独立追補

事前REDではない。U4本体2ファイルの完成後、`src/material.rs` と `src/material_diff.rs` の `cfg(test)` に5件を追加した。既存U5/U4テストで厳密包含・大文字/空白/Markdown/改行差、2000文字、240/241 Unicode scalarと制御文字escape、引用/理由件数・path形式は検査済みであるため、重複ケースは増やしていない。

| AC | 新規証拠 |
| --- | --- |
| Q01 | 実在する教材欄と配列欄の日本語名を固定。存在しない配列添字・未知pathは「対象項目を特定できない」とし、3件目等の実在しない名前を作らず、技術識別値は残す。 |
| Q02 | 未選択往復、`system` role、`usize::MAX` を合成 `Request::build_response` に入れ、別往復/別話者の固定原文を代用しない。保存案で `Source.validate` が先に失敗した場合は理由番号・引用を原因と断定せず固定元発言の取得不能を説明。 |
| Q04 | 不正pathと空白理由・引用0件を同時投入し、形式→実在参照→説明→引用の既存検査順を生成案と保存案の両入口で確認。 |
| Q06 | 保存案の根拠欠落時は「この案は登録できない」と段階を示し、未検査引用を比較結果として表示しない。登録や保存の操作は呼ばない。 |

実行: `cargo test --locked --lib feedback_ui_ -- --nocapture` はcompile成功、9件PASS、0件FAIL、exit 0である（新規5件と先行4件を含む）。親から受領した既存material全24件の成功は別時点の証拠であり、本9件の実行結果へ合算しない。Q05の通知Windowの選択/コピー/長文スクロール、native標準/狭幅はU3後の統合確認に残る。

## U2a/b/c 実装後の独立UI追補

事前REDではない。U2a/b/c本体freeze後、`src/app/color_theme.rs` の `cfg(test)` 観測点と実入力case、`src/app/harness_tests.rs` の全画面/狭幅case、`src/app/visual_check.rs` の合成native fixtureを追加した。製品の配色処理はテスト作者が変更していない。

| AC | 新規証拠と境界 |
| --- | --- |
| C02,C07 | 実TextEditのHEX/濃さ入力で有効青100→不正HEX `#12`/範囲外101。最後の有効preview保持、エラー/保存disabled、取消による確定値復帰。Window描画安定後の矩形に実ポインタ/キーイベントを送る。 |
| C04,C05,C07 | Home/Study/Deck/Words/Chat/Stats/Settingsの全7面を巡り、3frameの描画安定後、実Shapeのページ地と後開きWindow地を別の不変base由来の色で検査。反復描画で累積混色なし、取消後に旧色へ復帰。style値だけでは合格にしない。 |
| C04,C07 | 820×650/160%相当の論理画面で配色Window全体のbounds、保存/取消/標準色footerのclip内可視、bodyの実MouseWheel後に2組目と最終見本へ到達、scroll後footer常時可視を判定。生成済みでもclip外の本文は未到達扱い。 |
| C04,C05,C06 | `--palette-preview` は独立storage上の未保存青ページ/赤入力をSettings配色Windowへ配置。`--palette-saved` は同じ合成値を実Storage保存成功後、Settingsと後開き通知Windowへ配置。両fixtureとも実利用者データ・実AIを使わない。 |

初回 `cargo test --locked --bin wordweave5 feedback_ui_palette_ -- --nocapture` はcompile成功、既存7件PASS、新規3件FAIL、exit 1である。新規HEXと全7面Window塗りは初期描画直後のfocus/Shape判定でfixture未安定が判明したため、4frame安定後の実操作・描画判定へ修正した。この2件は本体REDとして扱わず、修正後の再実行を要する。狭幅Windowは4frame後でも矩形 `[[16,0]–[474,416.5]]` がscreen下端406.25を10.25論理px超えた。C04の受入期待は維持し、本体修正候補を親へ渡した。修正後の再実行結果を別途追記する。

二次切分けではHEX fixtureが `Modifiers::CTRL`（`command:false`）で全選択を試みていたことを確認した。egui TextEditの全選択は `command:true` を要するため `Modifiers::COMMAND` に訂正し、実TextEditの入力と期待RGBは維持した。Window塗りはegui Areaの1/12秒fade-in中に不透明色を即時比較していたため、headlessの予測frame時刻を8回以上進めてから実Shapeを判定し、失敗時にRGBAを列挙する。仕様の色値・狭幅boundsは変更していない。狭幅Windowの高さは実装担当が修正し、単独focusedでbounds/footer/最終見本到達をPASSしたとの報告を受領した。集約再実行は以下で別途記録する。

上記修正後の集約focusedはcompile成功、9件PASS/1件FAIL、exit 1である。実HEX/濃さの有効・無効preview、全7面の実Shape固定色、狭幅Window全bounds/footer/実scrollはそれぞれPASSへ進んだ。残る1件は設定ページにも同名「キャンセル」があり、汎用ラベル検索が配色Windowではなく配色中disabledの通常設定ボタンを先にクリックしたfixture誤りである。配色Window固有の `cfg(test)` 矩形を観測してクリックするよう修正した。native preview fixtureはPNG保存成功後にだけ未保存配色draftを破棄して通常終了させる。これをしない場合、製品の正しい未保存終了guardが合成撮影processのCloseを止める。いずれも本体仕様や期待色・保存可否を変更していない。修正後の次回集約は配色Window文字サイズ修正後に実施する。

次の局所修正後、狭幅の「第2組見出し」と「最終見本」をスクロールの最終frameで同時に可視とするassertだけが失敗した。同じ時点に上下離れた2箇所を表示することは仕様要求ではないため、第2組見出しが途中のいずれかの実scroll frameでclip内可視、最終見本が最下部でclip内可視、と時系列で別々に検証する。全scroll frameでWindow全boundsと3操作footerのclip内可視を維持し、最終見本到達の期待も維持する。標準native PNGでは配色Windowの本文/ボタンが既存dialogより小さく描かれたことを親が検出し、実装担当が共通dialog typographyを適用した。本記録は自動test合格の代用ではなく、修正後のnative再観測を要する。

最終U2局所修正後の `cargo test --locked --bin wordweave5 feedback_ui_palette_ -- --nocapture` はcompile成功、10件PASS/0件FAIL、exit 0である。実TextEdit操作、無効入力中のpreview/保存拒否、全7面の実Shape色、取消し、狭幅Window全boundsと各scroll frameの固定footer、第2見出し→最終見本の順次到達を含む。native PNGの字形・OS scale判定、全Window一覧とIME/disabled/対象外面の完了判定は独立runner・手動確認へ残る。

## 最終独立追補（U3・Q05・C03/C07）

事前REDではない。production freeze後に、`notifications.rs` の既存上限通知2件へ次を追加した。日次上限の学習`launch_ai`拒否と表示後count変化の陳腐クリック、実通知ボタン→Connection上限Sliderへの標準/狭幅実クリック、clip内到達・一回focus・focus再奪取なし、配色preview中のdisabled/pending保持→取消後focus再開、下書き/Progress/count/AI未送信の不変性を確認する。Q05は合成会話から実`Request::build_response`で長い引用不一致診断を生成し、狭幅の通知Window全boundsとコピーfooterを全scroll frameで確認し、`対処`行のclip内到達、コピー命令の全文一致、教材/記録/AI未変更を確認する。C03は黒白赤緑青×濃さ0/100のframe(false)実Shape色・有効/disabled枠とcontrast、空のTextEdit実Paint、局所style復元とButton非着色を確認する。C07は実色picker popupのEscがeditorを閉じず、次のfocused Escが未保存配色を取消すことを確認する。

最初の集約は新テスト側`Page`にDebugがない`assert_eq!`によりcompile失敗（本体REDではない）。修正後の `cargo test --locked --bin wordweave5 feedback_ui_ -- --nocapture` はcompile成功、17件PASS/0件FAIL、exit 0。さらに2件追加した初回はテストclosureが`InnerResponse`を返すE0308でcompile失敗（fixture誤り）。修正後の集約は18件PASS/1件FAIL、exit 1。残1件はegui TextEditが背景をouter rectへ塗り、返すresponse.rectがmarginを除いたinner rectであるのに、テストが両幅差4未満を要求したfixture誤りであった。期待色は維持し、背景Shapeがinner rectを包含する判定へ訂正。親指示どおりこの1件のみ `cargo test --locked --bin wordweave5 feedback_ui_palette_editable_text_paints_only_the_input_and_preserves_other_widget_style -- --nocapture` を再実行し、compile成功、1件PASS/0件FAIL、exit 0。直前PASSの18件は無関連差分につき再実行しない。最終all-targetsは独立runnerの担当である。

### 20受入IDの証拠と残範囲

| ID | 現時点の証拠 | 残る判定 |
| --- | --- | --- |
| C01 | blend/parse・旧キーdefault・保存再読込の先行自動PASS | native実色の照合 |
| C02 | 実HEX/濃さ入力で無効preview保持・保存disabled・取消PASS | 他の不正文字例は純粋parseでPASS |
| C03 | 5色×濃さ端点のframe実Paint/contrast、空TextEdit実Paint、disabled枠/操作不可PASS | 実TextEdit全色の字形・focus枠をnativeで確認 |
| C04 | 全7pageの地、局所Button非着色、狭幅editor bounds/footer/実scroll PASS | IME、画像/筆跡/意味色・全Window一覧、native両scale |
| C05 | 次描画7page＋後開き配色Windowの実Shape固定色、反復で非累積PASS | Windowドラッグ・その他後開きWindowのnative巡回 |
| C06 | 成功後のみ実Storage確定、別Progress更新保持、故障retry PASS | native保存後外観 |
| C07 | 実キャンセル、picker popup優先Esc/focused Esc、終了3択・失敗guard PASS | titlebar×の実クリック、強制終了実機 |
| C08 | dirty設定3択、配色中通常設定/復元guard、上限誘導disabled→取消後focus PASS | 実native媒体選択UI |
| C09 | 旧/不正JSON、autosave非混入、別更新保持の先行PASS | 実媒体/音声/筆跡の不変をnativeで確認 |
| L01 | 実`reserve_generation`/`launch_ai`当日拒否、確定limit/count、未保存draft非適用と陳腐クリックPASS | 翌日切替は時刻境界の実機確認 |
| L02 | 実通知click→Connection Sliderの標準/狭幅clip内到達・一回focus PASS | native focus字形・OS scale |
| L03 | count/Progress/学習・会話draft保持、AI pendingなし、配色中disabled PASS | 生成案を含む実操作の保持 |
| L04 | 保存済みlimitの拒否と設定draft非適用PASS | 0/1000境界・保存故障/取消後の手動再試行はrunner確認 |
| L05 | count<limit、limit1000、count1000、別通知の導線消失、クリック直前再判定PASS | 翌日・通知閉鎖のnative確認 |
| Q01 | 生成/保存案の段階、理由/項目/話者/最初の失敗の先行PASS | native通知で長文字形 |
| Q02 | 固定snapshot欠落・不正role/turn・非選択参照の先行PASS | 実会話は使わない |
| Q03 | 240/241 Unicode scalar/制御文字の先行既存検証 | 選択可能文字とcopyのnative挙動 |
| Q04 | path/件数/形式/厳密引用・検査順の先行PASS | なし（本体差分がなければ再実行不要） |
| Q05 | 実診断の狭幅wholeWindow、全scroll frame固定copy footer、`対処`行到達、copy全文一致PASS | native標準/狭幅PNG・実選択操作 |
| Q06 | 拒否時教材/記録不変、copyで再送・登録なしPASS | 正常案の可視diff→承認登録は既存ワークフロー/実機確認 |

### debug native合成撮影（runner用）

以下は既存 `--ui-check`、毎回一意の合成storage、実AIなしのfixtureである。標準は1150×950/80%、`--small` は820×650/160%。PNGの見た目・OS scaleは本書時点で未判定である。出力PNG名は各コマンドで別にする。

```powershell
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-limit.png" --daily-limit
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-limit-small.png" --daily-limit --small
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-limit-jump-small.png" --daily-limit-after-jump --small
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-quote-long-small.png" --quote-long --small
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-quote-long-scroll-small.png" --quote-long-scroll --small
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-chat.png" --palette-preview --page chat --chat-rename
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-notice.png" --palette-saved --page home --notice
```

`--daily-limit-after-jump` は通知実クリックの結果画面を合成状態で撮るfixtureであり、実クリック/recheckの証拠は上記headless testで別途取得済み。`--quote-long-scroll` は実MouseWheelを2回注入し、撮影をframe24まで遅らせる。`--palette-preview`/`--palette-saved` は明示`--page`を尊重し、未指定時だけSettingsとする。PNG保存後に限りpreview sessionを破棄して合成processを終了し、製品の未保存終了guardを迂回しない。

既存V fixtureで撮影可能なWindowは通知詳細、教材レビュー/詳細/操作、チャットのタイトル/添付/送信確認/媒体preview、設定倍率、配色editor等である。削除ごみ箱、録音取消、実行記録、版情報、未保存設定、媒体付き復元などは今回のVフラグで全Window巡回できず、native手動確認または追加の明示的fixture所有が必要である。画像原本/筆跡/IME/OSファイル選択と実Codexは合成fixtureで代替しない。

Native実測はrunnerへ引き渡す。隔離出力先の例は次の通り（PNG自体の描画判定・OS scaleは未実行）。

```powershell
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-preview.png" --palette-preview --page settings
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-preview-small.png" --palette-preview --small --page settings
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-saved.png" --palette-saved --page settings
cargo run --locked --bin wordweave5 -- --ui-check "$env:TEMP\feedback-ui-palette-saved-small.png" --palette-saved --small --page settings
```

現時点で残るC03/C04の空欄・disabled・IME保持、全Window/対象外面の網羅、実native字形と両scale、C07の×/Esc/picker popupは本追補だけで合格としない。既存U1 stateテストとrunnerの操作/描画証拠を合わせて最終判定する。
