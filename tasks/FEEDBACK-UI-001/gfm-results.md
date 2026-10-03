# U6 Markdownと教材登録結果（2026-09-29）

## 境界

通知の引用と英語チャットのAI回答をGFM系の読みやすい表示にする。登録承認後の成功・失敗を区別する。原文、厳密引用照合、保存形式、成功通知を自動ダイアログにしない既存方針は維持する。実装・独立検証・native表示・release更新まで。実AI生成・利用者データによる試験・commit/pushは対象外。

## 開始記録

- 既存dirty状態を確認し保持。親の初動後、2026-09-29T13:34:58.204Zの累積token_count：input 767340734 / cached 745916160 / output 1774810 / reasoning 535789 / total 769115544。開始前の初動は欠測。子との包含関係は不明。
- GFM公式仕様 https://github.github.com/gfm/ はCommonMark拡張である。Markdown仕様が厳密に3種類という定義や人気順位は主張しない。
- cargo infoでegui_commonmark 0.20.0はegui 0.31互換と確認。ただし既定描画のstrongフォント、画像・任意schemeリンク、表の狭幅挙動を検証する必要がある。依存採用は設計判断前、Cargo変更なし。
- 登録はapp.rs material_panelからimport_deck→commit::save成功後Ok。現在の成功はmessageのみ。失敗時pending意図が残る場合は再起動復旧を要し、「何も保存されていない」と一律断定しない。
- ユーザー回答：通知引用と英語チャット回答の両方が対象。

## 結果

 実装・自動検証・隔離native表示・release更新完了。実AIによる教材案作成/登録と利用者の表示受入は未実施。commit/pushなし。

- 親がgfm-spec.mdのG01–G08を要求範囲内の具体化として確認・承認。成功結果の消去は次の明示登録操作受理時であり、生成開始ではない。独立仕様/設計レビューは次工程。モデル起動指定：planner/spec/designer/reviewer Astra/high、implementer/test author Sol/high、runner Terra/medium。実行model metadataは未取得。
- U6前tracked差分の比較用ログは `.context/compound-engineering/gfm-before.patch`（公開対象外）。
- 独立仕様/設計review（gfm_review、Astra/high指定）：pass、仕様設計blockerなし。RED fixtureにP2あり：GFM表内コードの縦棒もescape必須（GFM example-200）。テスト著者へ入力修正・RED再確認を返した。原文保持の期待値は変更しない。実装受入は未実施。
- 修正版fixtureでもGFM表の行動REDは1件実行/1件失敗を確認。実装者へ引渡し、U6本体初回cargo check exit 0。依存追加はpulldown-cmarkとunicaseのみで既存バージョン更新なし。
- 利用者がrelease版を終了したとの回答後、起動プロセス0件を確認。強制終了なし。
- 実装先行レビューP2：tight list内の装飾/連続inlineが別Labelになり分断される。実装者がinline集約へ修正し、テスト著者が位置/style回帰を追加。新規testのclosure戻り型コンパイルミスはテスト著者が修正（製品の行動REDとは別）。最終検証待ち。
- 追加review：list内Rule消失とcode block既定wrapを修正要求し実装へ反映。深いblockquoteはparse/render/drop再帰のstack危険（クラッシュ未実測）と字下げの幅問題がある。設計担当の判断では全体Raw退避は仕様削減であり不採用。親はflat arena/明示stack・残幅に応じた字下げ制御を採用、設計書を限定更新。新しい長さ/深さの閲覧制限なし。実装・隔離回帰・再reviewで確認する。
- 2026-09-30再開。最新11focusedは9成功/2失敗（子Item内継続段落、dialog閉直後copy）。parse木で段落所属を確認し、marker実測幅のhanging indentを本体へ追加、一時tree診断を除去。copyはclip内でも閉じたdialog layerが操作を遮るfixture問題と実測し、layerが退くまでframeを進める試験へ修正。receipt単独は1/1成功、原文CopyText/receipt保持の期待値維持。11件の再実行とdebug/native進行中。
- focusedは11/11成功へ到達。nativeの表末列4件は未達。比較は展開済みだが初期画面の下方で未可視であり、縦移動と折返しmarker検出をfixtureで修正。また親のWindows画像目視と独立reviewで、長い表行の次の短い行が重なるP2を確定した。egui Gridの高さ0セルが行中央起点となるためであり、G04未達・先のsourcepassを撤回した。表セルをwith_layoutの上端配置・実測高計上へ修正し、独立source reviewは解消確認。独立非重複回帰・native再確認を行う。修正前の全targets372件/releaseはexit0だが、最終版の成功証拠にはしない。
- 独立row回帰を追加し、3条件×3frameで実折返し高差・全セル非重複・列整列を確認。新回帰単独1/1、gfm_12/12は実exit0。ログはgfm-row-focused.log/gfm-row-suite.log。新回帰の行動REDは修正先行のため未取得、変更前native画像で実不具合を確認済み。初回のrepeat(9)は画面外に出るfixture失敗であり、repeat(3)へ短縮して実折返し>1.5と非重複期待を維持した。
- 行高修正後の最終版で全targets373件（失敗・ignoredなし）、深い引用単独1件、releaseビルドが実exit0。ログは `.context/compound-engineering/gfm-row-final-{all-targets,deep-quote,release-build}.log`。親と独立reviewerがログ・EXEを照合。EXEは2026-09-30 10:17:47 JST、11,960,832 bytes、SHA256 `F982FD5DEE9BE799AF713C54562B2FD10D6AD30544A1382374865942B9A7763D`。更新前の起動0件を確認、強制終了・再起動なし。
- native末列の分割PNGでは親・独立reviewerとも本文内の「横スクロール」「終端②」を目視確認。小画面は表示帯より折返し2行の合計高が大きいため、複数の実表示画像で全9文字への到達を確認する方式はG04の維持と判断した。自動判定が先頭「横」だけを欠落扱いしたため、推測の表示帯ではなく既存copy observerの実clipをdebugでも観測し、epaintと同じglyph局所座標のpixel-round後にtext位置を加算する判定へ修正中。出荷本体・検証基準は変更しない。
- 親が実clip呼出とglyph局所座標を先に丸める確定ソースを照合した。通知小画面の未達は、headerがclip下端へ2.2pxだけ現れたときにpointer移動と横wheelを同フレームで送るfixtureにあり、pointer移動→次frameの横wheelでexit0へ到達。親は `gfm-native/fixturehover-notice-small.png` で末列「横スクロール終端②」が本文内に表示され、footerのcopy/closeが残ることを目視確認した。チャットは `fixtureclip-chat-small-part1.png` と `fixtureclip-chat-small.png` で全9文字を確認。診断出力除去後のdebug/native最終実行を行い、結果を以下へ追記する。
- 診断出力除去後の `fixturefinal-*` はchat2条件exit0、notice2条件exit101であり、先の通知成功を最終run成功とは扱わない。native画像で表示自体は確認済みだが、clip下端で僅かに見えるheaderへ操作するfixtureが描画タイミングに依存していた。header mesh全体がclipに入ってからhover→wheelとする限定修正を行い、notice2条件のみ再確認する。
- 最終のheader全体clip条件でclean debug build exit0（`gfm-fixtureheader-build.log`）、notice標準/小画面はともにexit0（`gfm-fixtureheader-notice-{standard,small}.log`）。PNGは `gfm-native/fixtureheader-notice-{standard,small}.png` と対応part画像。親もsmallで末列全9文字・footer・非重複を目視確認した。chat標準/小画面のexit0証拠は直前の `gfm-fixturefinal-chat-{standard,small}.log` と対応PNGであり、最後のdebug helper条件変更後は再実行していない。本体renderer/保存処理は同一である。通知末尾・登録receiptの既取得成功画像も保持し、fixture失敗を製品不具合や最終成功へ混同しない。
- 独立reviewは本体・373件/12件/深い引用/release・分割PNG全9文字を確認済み。最後のdebug helper差分はホストthread limitによりreviewer再開不可のため親が照合した（実clip、局所pixel-round、header全体clip、hover→次frame wheel）。診断trace除去・diffチェック成功。productionの既存warning4件あり、今回のscope外の未使用コードは変更していない。新規の実AI呼出・利用者教材/録音/筆跡の試験利用なし。
