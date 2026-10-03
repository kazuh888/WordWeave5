# QWEN-STOP-001 合成native外観

最新debug EXEを既存ui-checkで起動し、fake接続・合成データのみで撮影した。音声再生、課金API、利用者データは使用しない。親が次の5画像を実見した。全owned PIDは通常終了exit 0、証拠はnative/exits.logである。

| 画像 | 条件・観測 |
| --- | --- |
| native/input.png | 1150×950/100%。未選択でも停止円＋■とラベルを表示、■は無効用色。 |
| native/input-125.png | 標準/125%。操作行と停止アイコンが横切れせず表示される。 |
| native/input-min-80.png | 820×650/80%。操作行と円＋■、固定Closeが画面内。 |
| native/input-min-125.png | 820×650/125%。初期表示では操作行がclip外であり、停止ボタン可視の証拠にはしない。既存の本文スクロールが必要である。 |
| native/speech.png | 模擬読み上げパネル。共有停止アイコンの外観比較用。音声は再生しない。 |

実再生中の音読ダイアログnative表示・音停止・自然終了・nativeキーボード・tooltip・支援技術は未検証である。enabled/disabledの描画とpointer/実Tab/Enter/Spaceは別途headless回帰で検証しており、実デバイス試験とは区別する。
# U2 コントラスト比較

native/contrast/input.png、input-125.png、input-min-80.png、speech.pngをU1の同名画像と比較した。無効■は明るい灰色、有効円枠は濃い青・2ptとなり、他アイコンとクリック領域は保持した。4プロセスは正常終了exit0である（native/contrast/exits.log）。合成データ・模擬再生状態であり、実音声・API送信はない。

通常状態の実native画像である。hover/Tab focusはheadless実イベントで別途確認したがnative画像は未取得である。debug-v2で撮影し、その後のf32型明示は同値のため画像を再取得していない。Windows DPI・実録音/自然終了の受入は別途である。
