# QWEN-SETTINGS-001 実施記録

2026-10-03。対象は例文と音声の不一致表示、Qwen接続設定の移動・リージョン選択、導入文書の明確化である。既存の未コミット変更を保持する。完了境界は実装・独立レビュー・隔離した自動検証・release作成まで。実アカウント/課金API送信、他地域の実接続、利用者の音声による精度受入、commit/pushは含めない。

## 調査

- 現行の `ResponseInvalid` はJSON/検査の失敗をまとめた表示である。原応答は保存しないため、報告された1件の原因そのものは確定できない。
- 現行契約は「評価可能」と「音声を評価できない」だけで、「別の英文が聞き取れた」を明示できない。例文外の語を改善箇所に返すと参照部分文字列の検証に失敗する。検証を緩めず別結果を追加する。
- 公式確認（2026-10-03）：[対象モデル](https://www.alibabacloud.com/help/en/model-studio/qwen3-8-omni-flash) は東京・シンガポール・北京・香港・フランクフルト・バージニアの6地域を掲載。
- [Chat Completionsエンドポイント](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions)、[Base URL](https://www.alibabacloud.com/help/en/model-studio/base-url) に6地域のWorkspace専用URLがある。今回はこの形式を許可する。一般的な互換API説明には北京URL等の不整合があるため、地域別エンドポイント表を採用した。
- [リージョン](https://www.alibabacloud.com/help/en/model-studio/regions) は地域ごとのキー/モデルを区別する。実際の契約、Workspaceの提供範囲とモデル利用権は各利用者が確認する。地域を選んでも権利や処理地域全体を保証しない。

## 計測

親モデル/effortは利用者が変更済みと申告。推奨はGPT-6 Astra/high。ランタイム値は未取得。開始累積カウンターは取得できず、消費量を推定しない。初期調査後の記録時刻は2026-10-03 13:42 JST。子の起動指定と検証結果は完了時に追記する。

## 設計・実装の途中証拠

- planner/spec/reviewer/core implementerは明示的 `gpt-6-astra/high`、独立テスト著者は `gpt-6.1-sol/high`、runnerは `gpt-6.1-sol/medium` で起動した。親は設計・root実装・統合・文書を担当し、親モデルを変更していない。子のランタイムmetadata/消費量は未取得である。
- 独立仕様・設計レビューはP1なし、P2が2件：キャンセルの破棄確認不整合は仕様へ統一して修正。debug合成画面をrelease実機画面と同一視しない点は、release-native実機を未検証の受入残と明記した。専用hookをreleaseへ開放していない。
- 独立テストは適合する別英文JSONが旧validatorに拒否されるREDを採取した。地域API不在のcompile REDとは区別した。実際の利用者の過去の1件の原因は未確定のままである。
- core契約19件と接続確認2件がGREEN。root初回13件中1件はテストが編集draftではなく保存済み値へ直接書いたfixture誤りであり、期待値を維持して入力先だけ修正。最新版15件は全体試験へ渡した。詳細はtests.md。
- 同時buildで競合/負荷が発生したため、helper自身の未完buildを中止し、以後runnerの直列 `-j 1` へ集約する。検証を省略する措置ではない。packageは既存 `target/qwen-audio` を指定する。no-defaultのconsentフィルターはGUI gateにより0件だったため、成功証拠とはせずdefault全体試験で検証する。
- README等6文書のローカルリンク確認は欠落0件。実キー/原音/実学習データは検証に使わず、API送信/commit/pushは行っていない。

## 最終レビューの追跡

- 接続設定を閉じた後に編集ボタンへfocusを戻す共通終了処理を追加。旧Tokyo v1保存データは、本番読み込みと同じ副作用のないdecode関数で試験する。独立著者の本体設定試験は20件、資格情報試験は8件へ増えた。
- 追跡レビューが、変更なしmodalのOS終了だけが`CancelClose`を迂回する設計不一致を検出した。先行finish分岐を削除し、modal存在時は`CancelClose → request_close → finish`へ統一した。未保存変更時は破棄確認、変更なしなら設定画面を閉じてアプリ終了を保留する。期待値を変えず、静的再レビューで解消確認済み。
- root全体試行1はdebug合成画面プロセスとのEXE競合でテスト前に失敗。親が自分の隔離fixtureだけへ通常の終了要求を出して解消した。実利用者アプリの強制終了は行わない。
- 試行2はlib134件成功、bin232件成功/1件失敗。未設定リンクの描画文字検査が失敗したため、独立テスト著者が原因を調査し、最終成功扱いにしていない。native合成画面では同リンクが見える。後続の検証結果を下記へ記録する。
- 当該表示試験は、clip外のボタンが描画shapeに含まれない条件だった。独立著者は期待文字を保持し、未設定時だけのwidget生成と実スクロール後の描画を検査するfixtureへ修正した。設定済みcaseはwidget/文字とも不在を検査する。
- `settings-normal-final.png`、`settings-narrow-final.png`、`settings-tail-final.png`で入力欄の背景、標準配置、狭幅1.5倍の先頭/末尾と固定操作を親が確認した。これらはdebug合成画面であり実資格情報・release実機操作の証拠ではない。
- 初回`connections-final.png`はQwen領域への誘導を確認できなかった。constructorの0.8倍率予約とpreviewの1.0要求の順序により、初回focus消費後に再配置される仮説がegui実装と整合する。未確定の本体不具合とは断定しない。debug確認をframe3の実`go_to_qwen_settings`操作へ変更し、独立focusテスト4件にも最終frameのボタン全体clip内/画面内/文字描画を追加した。後続画像と試験で判断する。
- package通常構成135件、core-only構成88件が成功。機能gateで0件だったtargetを成功件数へ加えていない。詳細はrunner.mdの最終証拠へ集約する。
- packageのWindows資格情報feature check、release/debug、全体fmtが成功。最新debugの`standalone-mismatch.png`で参照例文/別英文/再録音案内を、`standalone-singapore.png`でシンガポールと合成専用Hostの表示を親が確認した。地域選択の実クリックや実API接続の証拠ではない。両fixtureはexit0で終了し、実資格情報・通信を構築していない。
- 強化したfocus可視試験で4件のREDを得た（他16件成功）。診断1件のframe0はclipが幅12500/下端12421の仮領域、ボタンY1343、pending=false、scroll offset=0。frame1/2は同じscroll IDの実clip下端736、ボタンY1363、offset=0だった。直接の失敗機序は初期の仮viewport内を実表示と誤認して誘導完了にしたことであり、アニメーション予約の主因説は棄却した。仮値が生じた手順は下記のfixture切り分けで補足する。ログ`runner-root-focus-diagnostic.log`を保持する。
- サイズ計算中/無効な背景ではfocusを完了せず、clipと実screenにボタン全体が入った場合だけ完了する。画面外のときだけscroll要求し、Qwen誘導時のみprogrammatic animationを無効化する。通常のスクロール設定は維持する。診断用eprintln/temp traceを除去し、強化した期待値は保持して再検証する。
- 上記guard後も初期fixture4件は同じRED。独立レビューにより、fixtureが初回zoom変更直後・最初の画面描画より前にmodal終了を直接作っている点を切り分けた。初回screen自体も仮の巨大値となる条件であり、実ユーザーの「表示された設定から開いて閉じる」順序とは異なる。設定画面を先に描画して実寸をassertし、modal操作後のfocus/全体clip/実文字描画期待は維持する方針を採用した。安定判定stateをproductionへ追加する根拠は現時点でない。通常操作相当のfixtureとnative確認も失敗する場合だけ再検討する。
- 最新debugで画面安定後の実`go_to_qwen_settings`を撮影し、`connections-stable.png`はQwenカード全体と編集ボタン、`connections-stable-narrow.png`は狭幅1.5倍の編集ボタン全体が可視であることを親が確認した。startup直後の旧画像を成功証拠へ置き換えず残す。撮影後rootプロセス0件を確認して最終検証へ戻した。
- 通常操作順へ修正した最終設定テスト20件が全件成功。終了後4frameのfocus ID・ボタン全体clip/screen内・実Text shape描画の期待を維持している。独立レビューは対象の残存指摘なし。以後はソースを変更せず全体回帰と本体releaseへ進む。

## 最終検証

- 本体 `cargo test --all-targets --locked -j1`：439件成功、失敗0、ignored0。最終設定20件を含む。`runner-root-stable-tests.log`。
- 共通package通常構成：135件成功。core-only構成：88件成功。Windows資格情報feature限定check成功。実行対象0件のtargetは成功件数へ加えない。
- package release/debug、全体fmt成功。本体debug成功。本体全体fmtは既存の広範な整形差分で不合格のまま保持し、新規Qwen設定moduleと担当package範囲の整形は確認済み。無関係な一括整形は行わない。
- native合成画面では、本体の不一致結果/未設定導線、接続編集標準/狭幅先頭/末尾、安定描画後のQwenカード誘導標準/狭幅、standaloneの不一致/シンガポール表示を確認した。release実機、実マイク/IME/キー保存/地域契約/音声判定精度は別受入である。
- 独立レビューの残存指摘0（静的対象範囲）。全体検証と成果物hash・固定入力一致はrunner.mdへ集約する。実API/実学習データ/実資格情報の操作、commit/pushは行っていない。

## 完了境界と成果物

2026-10-03 15:28 JST時点で設計・実装・自動検証・debug合成画面確認・両release生成まで完了した。利用者によるrelease実機受入と実APIでの別英文判定は未実施である。

| 成果物 | 更新日時（JST） | SHA256 |
| --- | --- | --- |
| `target/release/wordweave5.exe` | 2026-10-03 15:26:35 | `F9F708A48E8EFC3ECDC4CD50C19DB211158BE1C3BBEA5D6A319FB7F1EB78FF24` |
| `target/qwen-audio/release/qwen-audio.exe` | 2026-10-03 14:41:24 | `968A24822EB3CB166F6E50F04236A2455EBCB2B3C19AB02339C50FDA38AAEF09` |

最終コンパイル入力114ファイルは検証前後で差分0。全体fmtの既存広範差分を解消したとは主張しない。単独版と本体の試験件数、焦点と全体の重複を合算して独立coverageとは扱わない。実際の音声と例文の違いを判定する精度は、合成JSONや表示試験から保証しない。

次の利用者確認は、更新版で同じMP3を選び、参照例文と異なる英文の結果表示を確認することである。設定は「設定 → AI接続 → 音読評価：Qwen」、専用の保存・キャンセルを使う。地域・Workspace変更時はキーを再入力する。キーをチャットに貼る必要はない。
