# U7 固定会話Markdown・比較ペインの独立スクロール

## 範囲と完了境界

利用者の画像で示された固定会話のAI回答を既存のCommonMark＋GFM描画へ接続し、変更項目一覧と詳細の縦スクロールを分離する。教材案・生成時原文・厳密引用判定・登録前承認・保存処理は維持する。実装、合成データ回帰、隔離Windows表示、release更新まで。音声評価は可能性調査のみで、実装・原音送信・API追加・commit/pushは含めない。

## 開始観測

- 既存dirtyを確認し保持。親開始観測2026-09-30T10:56:21.073Z：input 826510955 / cached 804253952 / output 1868986 / reasoning 573297 / total 828379941。初期読込前の観測は欠測、子との包含関係は不明。
- コード上の原因：app.rsのmaterial-proposal-bodyに一覧と詳細の両方が入り、material_review.rsの各ペインにScrollAreaがない。固定AI回答はquoted_textのliteral LabelでありMarkdown未適用。
- 音声はai.rs::transcribeが原WAVを文字起こし専用指示でCodexへ送り、文字だけを返す。発音評価結果や評価校正は未実装。原音分析と文字起こしだけの推測を区別し、現在のCodexモデルでの評価精度を保証しない。
- 利用者は音声評価の優先対象に「例文を読む練習」を選択した。比較する英文と原音の対応、発音/強弱/リズムの具体的な改善案を候補とし、Codex音声評価能力の検証前に点数や精度を保証しない。Microsoft公式の発音評価資料にも入力品質・人の評価との整合・利用場面での検証が必要とされる（https://learn.microsoft.com/en-us/azure/foundry/responsible-ai/speech-service/pronunciation-assessment/characteristics-and-limitations-pronunciation-assessment）。これは評価設計の参考であり、Azure導入の承認・推奨・実装ではない。現行のChatGPT認証Codexのみという制約を維持する。

## 状態

計画のD01–D05とモデル割当を親確認済み。U6の外部仕様/原文契約を再利用し、新しい利用者判断は不要である。設計担当の有限body・左右独立・原文fold方針を親が確認した。使用者の既存学習データは試験に用いない。

- 親承認モデル：planner/spec Astra/high、designer/implementer/test author/reviewer Sol/high、runner Terra/medium。実行metadataは未取得。開始差分は `.context/compound-engineering/review-detail-before.patch` へ保存した。
- 設計文書と独立回帰は所有が別であり期待値を実装前に固定できるため、test authorの並行起動を1回試したが、ホストthread limitで起動できなかった。別モデルへの置換はせず、設計役終了後に既存役の再利用を試す。
- 既存test author `gfm_row_tests`（Sol/high）を再利用し、`review_detail_` 3件は実装前にcompile成功・実動作差でREDを確認（Markdown未適用、左wheel無移動、狭幅一覧不可視）。ログはreview-detail-red-all.log。双方末尾/A→B→Aの追加assertは先行RED後のため未到達、GREENで実行確認する。Cargo解放後、既存implementer `gfm_finish_impl`（Sol/high）へ所有2本体節の編集を許可した。
- native確認の親所有はdebug-only `src/app/visual_check.rs` の既存material-review fixtureだけである。合成AIの太字/2列表と `--material-context`（実headerクリックと実detail viewportへのwheel）を追加、既存material-scrollはChat draftで実left viewportへ送る。原データ・実AI・release機能は変更しない。新viewport hookは本体担当がcfg(test/debug)に2Rectだけ保存する。編集完了とCargo開始可を本体担当へ通知した。
- 初版focusedは1/3合格。左右offsetは左wheel後11.2→18.8、右wheel時は左18.8固定・右0→16.3→27.5であった。Point wheelの慣性と描画1frame遅延中の比較であり、独立reviewerの確認後、親がwheel後30空frameを2箇所追加した。移動/非移動/末尾/A→B→Aの期待値は維持した。test author再起動はhost thread limitで不可であり、独立reviewerの入力修正判定を得て親が統合した。
- 狭幅ではavailable 211.66ptに対しfooter予約254.09ptでbodyが1ptとなり、内容が148ptへ膨らんで破棄ボタンを押し出す実欠陥を担当が測定した。表示条件を緩めず本体の高さ配分を修正中である。
- 旧narrow harnessは「元の会話を表示」の可視assertを保持し、wheel入力を新しい右詳細の実viewportへ変更した（正の矩形・画面内をassert）。操作先だけの追従であり期待値は維持した。native合成テーブルには日本語太字を追加し、新しいWindows表示で確認する。
- 独立test author再起動が成功し、右初回wheelだけ10ptへ縮小した。35ptでは追跡用見出しがclip外へ移動するためで、5pt超の移動・左不動・末尾・A→B→Aの期待は維持した。追加で狭幅の同一基本語checkboxの実clickによりready→invalidを同frameに切り替える回帰を作成した。本体は編集前ready別予約を廃し、有限の最大footer行を常時予約する。追加回帰の初回は型推論E0282のcompile-failであり、製品不具合/RED証拠に算入しない。authorがf32注釈と拒否理由の固有文字列照合を修正し、理由/説明の横配置を許して非重複・可視・保存不変を判定する。最終4件の実行は担当へ引き渡した。
- focused6は固定AI/狭幅/同frame登録可否の3件合格、D03のA→B→Aのみ不合格。30空frame後も左位置35ptを保ったがB clickで196.9ptへ変わり、慣性だけの問題ではなかった。限定traceでraw/smooth wheel双方0、pointer移動Y=-161.9ptと左offset増分+161.9ptが一致。使用中egui 0.31.1のscroll_area.rs:620–642にあるdrag_to_scrollの `offset -= pointer.delta()` に対応する。左右比較paneだけ本文dragスクロールを無効にし、ホイールとスクロールバーを維持する限定修正を親承認した。素早いマウス移動＋押下の実UI入力でも起こり得る。タッチ/ペンの本文swipeは別途未検証である。
- focused7は4件合格・実exit0。診断除去後の最終sourceでrunnerが全targetsを実施したが、app 196件合格/1件不合格・exit101で停止した。既存 `material_ui_narrow_dialog_keeps_heading_and_registration_actions_visible_while_scrolling` が長い登録禁止文でdialog幅5699.3ptへの膨張を検出した（viewport512.5pt）。debug/release/nativeはまだ実施せず、footerの理由文折返し/幅制限だけを本体担当へ戻した。新4件合格を全対象合格と同一視しない。

## 最終結果

- 長い禁止理由は説明ボタン後の残幅を明示し、有限のScrollAreaとwrap/selectable Labelへ収めた。旧狭幅1件とU7 focused4件は新sourceで合格・実exit0。期待値は維持した。
- `cargo test --all-targets --locked` 再実行は377件合格/0失敗/0 ignored・実exit0。debug/releaseビルドも実exit0。起動中EXEは0件を確認し、強制終了しなかった。ログと実行者報告は [review-detail-runner.md](review-detail-runner.md)。最終 `git diff --check` はexit0である。
- native初回6画像は生成exit0であったが、標準では表セルがclip下、狭幅では固定会話が画面外だったため受入証拠にはしなかった。debug-only撮影を実viewport内marker8連続可視へ変更した。標準markerの偽陽性は背後チャットの同文を全layer探索していたためで、教材確認WindowのLayerIdだけに限定した。通常のU6撮影経路は維持した。
- 最終nativeは `.context/compound-engineering/u7-native/context-layer-{marker,tail}-{standard,small}.png` の4画像・実exit0。親と独立reviewerが、右pane内の日本語太字/表2列2行/固定回答末尾、左右独立、header/footer維持を目視確認した。tail初回exit101はrunnerの必須`--material-context`引数漏れであり、本体/fixture不具合ではない。訂正後exit0、経緯はu7-native-layer.logに区別した。
- 最後のdebug撮影変更後はdebugビルドとnativeだけを再確認した。本体は変更していないため、全targets/releaseの成功済み検証は反復していない。独立reviewerの最終指摘は0件である。
- 更新EXE: `F:/Kazuhiro/GitHub/WordWeave5/target/release/wordweave5.exe`、2026-09-30 21:31:46 JST、11,962,880 bytes、SHA256 `C91B34B1900D0D31DEC09E314153AEB7F0326055189588D2F9803BE2A8868F23`。
- 実AI/実教材による利用者受入、IME、touch/pen本文swipe、音声評価は未実施である。音声評価の優先は「例文を読む練習」で確定したが、現行Codexの原音評価能力/精度は未検証、機能は未実装である。原音と文字起こし由来の推測を分離し、能力確認前に点数を保証しない。commit/pushは実施していない。

改善機会: wheelの慣性終了・pointerの実delta・Windowの描画layer・対象marker・必須起動引数・stdout/stderr保存を撮影/操作fixtureの最初に確定すれば、今回発生した入力/本体/撮影の切分け手戻りを減らせる。対照測定はなく、削減量は未判定である。検証水準や親model/effortは変更しなかった。
