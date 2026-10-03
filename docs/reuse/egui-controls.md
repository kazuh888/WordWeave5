# egui共通controlsの再利用境界

2026-10-03 / FRAMEWORK-DESIGN-001 / L03。文字controlsの表示・操作境界の設計案である。統合実装・二消費者受入は未実施である。
[全体設計](README.md)、[Framework](framework.md)、[候補台帳](library-candidates.md)、[文書仕様](../../tasks/FRAMEWORK-DESIGN-001/spec.md)を前提とする。
「現行」は確認したソース、「proposed」は未実装の契約である。画面構成・最終公開署名の確定ではない。
api-and-interface-design と wordweave-egui-ui に従い、native操作を保持した最小の描画窓口を設計する。

## 1. 現行の重複と挙動差

| 根拠 | 確認した現行処理 | 共通化での注意 |
| --- | --- | --- |
| [本体controls](../../src/app/controls.rs) | Button/UiControls、selectable、menu、collapsingのpaint shapeを補正 | 対象rect内のみ補正、invalid meshはskip、headerは左寄せ/縦中央 |
| [Qwen controls](../../tools/qwen-audio/src/controls.rs) | enabled button、wrap/min-height、mesh中心補正 | invalid meshはgalley.rectへfallback。対象rect検査が本体と異なる |
| 本体list_row | 44pt行、色/選択/省略/hover、左寄せ | 本体の行policyであり、そのまま共通テーマにしない |
| 両[本体Cargo](../../Cargo.toml)/[Qwen Cargo](../../tools/qwen-audio/Cargo.toml) | eframe 0.31.1。Qwenはaccesskit featureあり | 同じ版でもfeature/host設定が異なり、accessibility達成は別受入 |

現行helperはctx.graphics_mutとShape::Textの変更へ依存する。egui内部描画の挙動を公開契約へ漏らさず、版更新の描画回帰を要する。
依存は「host画面 → controls → egui/epaint」である。WordApp、Entry、Progress、音声engine、Markdown/parser、保存へ依存しない。
対象は通常文字ボタンとselectable controlsを中心とする。menu/collapsingは同じ境界内で例外を検証できたものだけ移す。
画像/shortcut付きhome群、音声vector icon、教材行、全画面layout、通知policyを一括共通化しない。

## 2. 最小入出力・操作（proposed）

既存Button wrapperとUiControlsを入口候補とし、必要な操作だけ明示exportする。両hostの全builderを先に揃えることを目標にしない。

| 窓口 | 入力・所有 | 出力と不変条件 |
| --- | --- | --- |
| 文字Button | Ui借用、WidgetText、hostのenabled/size/wrap/style | egui Response。click/hover/focus/disabledをnative widgetから返す |
| selectable | label、selected、必要なら借用current/value | native Responseと明示変更。表示frameだけでhost actionを発生させない |
| menu / collapsing | label、安定ID、body closure、host layout | headerのResponse/body結果。bodyの文字をheaderへ動かさない |
| 字形補正 | そのwidgetのpaint範囲、rect、軸policy | 対象字形だけ位置補正。interaction rect・ID・clipは勝手に変更しない |

テキストはUnicodeを保持し、翻訳/正規化/教材判定をしない。空label、改行、日本語とLatin混在は描画境界ケースとして扱う。
clip/利用可能幅はUiから取得する。非有限/空rectで巨大な座標補正を行わず、native描画とResponseを保持する。
本体list_rowの色、44pt高さ、左寄せ、省略と全文表示はhost所有である。必要なら基本wrapperを使うadapterとして残す。
API名は仮称である。既存ww_*互換wrapperとQwen button wrapperを通じ、host固有の名前を共通部品へ要求しない。

## 3. 日本語字形と両軸中央

通常文字ボタン/selectableは、配置されたGalleyのmesh_bounds中心を対象rect中心へ両軸で合わせる。line box中心だけを合格条件にしない。
collapsing headerは左寄せを保持し縦中央だけを補正する。menu headerの中央補正はpopup/body文字を対象にしない。
proposedは「widgetが追加した範囲かつ対象rect内の文字」に補正を限定する。本体の対象検査を出発点とし、重なるbody/popupも試験する。
empty/nonfinite meshは有効なgalley.rectへfallbackし、両方無効なら補正skipする案である。本体のskipとの違いを移行差分として明示する。
Qwenのfallbackを採るだけで本体に無差分とはしない。空文字・space・改行・欠落glyphで位置と操作を比較し、回帰後に採否を確定する。
対象外のparagraph/Markdownの行間は変えない。単一画面の固定Y offsetや、空白文字によるicon余白を共通解にしない。
iconと文字を持つcontrolは予約rectと同一座標系で群の中心を設計する。通常text wrapperへ盲目的に置換しない。
custom paintでもreal Button等を残し、二重文字を描かず、accessible label・focus ring・hover・disabled表示を保持する。

## 4. font・theme・scale・狭幅

| 担当 | 決める/実施する内容 | 契約境界 |
| --- | --- | --- |
| hostの初期化 | 日本語font、fallback、theme、eframe適用後style、zoom | controlsがfont/texture/themeを毎frame再作成しない |
| controls | 利用可能rect内の文字配置、wrap、native Response | 現在のscaleで論理座標/paint境界を合わせる。全画面のfitは保証しない |
| host画面 | 行/列、min size、scroll、header/footer、優先操作 | 狭幅/拡大で停止・取消し等へ到達できる構成を選ぶ |
| 仕様承認者 | 対応最小幅、scale範囲、全文閲覧、accessibility水準 | 数値や対応機器を設計文書だけで発明しない |

horizontal親内のcardは明示vertical layoutを使ってから幅を制限する。hostは短/長/空label、大きい数字で右端と縦の到達性を検査する。
文字拡大と狭幅の組合せをfixtureに含め、bodyをscroll可能にする。二次操作を折畳む場合も重要な状態/警告/取消しを到達可能に保つ。
wrap/truncateの選択はhostが指定する。省略labelの全文をhoverだけに依存せず、click/tap等でも確認できるhost経路を仕様化する。
本体の過去geometry fixtureには0.6/0.8/1.0/1.6 scaleがあるが、再利用版の対応保証値ではない。今回再実行していない。
app zoomとpixels_per_pointの検査はWindows DPI・実font・native windowの検査の代替ではない。

## 5. 状態・データ・非同期・保存

描画時はhostの状態を借用する。ID/current/selectedの正本はhostまたはegui contextにあり、共通部品にアプリ全体のglobal選択を持たせない。
通常は「host state → native widget → Response/action → host controllerの状態更新」である。paint補正だけで操作を生成しない。
hover/focused/pressed/disabledの遷移はeguiが所有する。disabledはclick無効と外観を保持し、hostが理由を表示する。
keyboard activationとfocus順はnative Responseを保持し、hostが画面全体の順序・shortcut・focus移動を管理する。
非同期worker、取消token、外部送信、AI結果不明はcontrols自体にはない。hostの処理を開始/取消しする操作結果のみ返す。
hostは非同期完了を対象ID/版へ結び付け、画面変更後のstale actionを拒否する。controlsが保存/再送を自動で実行しない。
font/theme/zoom、音量、選択行等の永続化はhostが行う。画面で値が変わったことと設定保存成功を分け、保存失敗はhostが通知する。
eguiの一時focus/collapsing状態を永続schemaとして約束しない。安定IDはhostが供給し、再表示時の衝突を検証する。

## 6. 失敗・互換・非保証

| 場面 | 部品の契約案 | hostの責任 |
| --- | --- | --- |
| mesh/rectが空または非有限 | 有効layout boundsへfallback、なければ補正skip | 読めないfont/labelを診断し、表示の受入を行う |
| 狭幅でclip/操作不可 | native clipを保持し、他widgetを移動させない | layout/wrap/scrollを修復。利用者の下書きや選択を保持 |
| 未対応control/egui版 | 能力/版を明示し、未検証の補正を全画面へ適用しない | 既存wrapperへ復帰し、影響する描画・操作を再検証 |
| 設定保存失敗 | 保存非担当。Responseの成功を保存ackにしない | 未保存を通知し、同じ設定保存だけ再試行 |

共通版は最初に0.31.1対応を検証する。API版、egui版、host設定schemaは別であり、依存更新だけで旧画面互換を保証しない。
旧wrapperを残して段階移行し、本体skip/Qwen fallback・min-height/wrap・menu例外の採否を記録する。未検証の一括置換は避ける。
本体とQwenでaccesskit featureが異なるため、wrapperによるnative semantics保持からscreen reader合格を推定しない。
IME、touch/pen、screen reader、実Windows font/DPIは別受入である。教科の教育効果・他UI toolkit/全OS対応は非保証である。

## 7. 二消費者gate・test seam・AC

二消費者はWordWeave5とQwen GUIを候補とする。同一版をコピーなしで利用し、hostごとのfont/theme/layout/保存policyを独立adapterへ残す。
試験口はegui Context/RawInput、paint text bounds、Response、host提供font/style、安定IDである。実データや実APIは不要である。
将来テスト作者は両軸中心、headerの縦中心、popup/body非移動、empty/nonfinite fallback、日本語複数行、disabled、focus/Enterを検証する。
scale/狭幅/長短label/数字/アイコン群を両hostで比較し、表示frameのみでaction0件、対象外文字移動0件、保存操作0件を確認する。
native screenshotと実font/DPI、keyboard、対象accessibilityの受入をgeometryと分ける。最小表示アプリだけで二製品の実利用としない。

| 文書AC | 設計証拠 | 実装/テスト作者への引継ぎ |
| --- | --- | --- |
| FW-AC02/04 | 1–2節、L03 | 共通描画とhost行/画面/設定policyを分ける |
| FW-AC05 | 2–5節 | 日本語、両軸中心、例外、scale/狭幅、操作状態を契約化 |
| FW-AC06 | 5–6節 | 送信/保存は非担当。native操作保持と実機非保証を別記 |
| FW-AC07/08 | 7節、本表 | 二消費者は未実施。今回の静的検査/独立レビューは[結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)へ |

未決は公開control集合、描画内部依存の代替、最小幅/scale、font配布、accessibility水準である。後続仕様と描画回帰で決め、今回の文書合格を実機合格にしない。
本書はD30設計のみである。実装/試験追加/画面操作は行っていない。起動指定gpt-6.1-sol/high、実行metadataとtokenカウンターは未取得である。
