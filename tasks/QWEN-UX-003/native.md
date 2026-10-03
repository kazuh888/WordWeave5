# QWEN-UX-003 合成native確認

2026-10-03。親が最新debug EXE（SHA256 `0E02775C5E29609D6FA74367FF39FC0152B9F8A4457D527ED59A0110DC33AF5F`）の既存ui-checkを起動し、下表の14画像すべてを実見した。fake Store/Transportと一時学習データだけを用い、キー・実音声・課金APIを使用していない。

| 条件 | 画像（native/配下） | 観測 |
| --- | --- | --- |
| 標準1150×950/100% | input、confirm、running、result | 段階概要・現在段階、①②③の順、番号付き操作、確認と送信、完了時fold、結果と再練習、固定Close |
| 同標準の別英文/評価不能/JSON不正/未設定 | mismatch、unassessable、jsonfailed、unconfigured | 状態を③/②へ配置し、エラーを発音不良と推定しない説明と次操作が読める |
| 実最小820×650/80% | input-min-80 | 3本文段階を初期表示し、終了操作と説明が画面内 |
| 実最小/100・125%の末尾 | confirm-min-100-tail、confirm-min-125-tail | 送信ボタン、②の説明、③の案内、Closeと終了説明が可視 |
| 実最小/160%の冒頭・末尾 | result-min-160、result-min-160-tail、failed-min-160-tail | 固定段階/現在段階/Closeを保持し、結果末尾・再練習・応答詳細/エラー詳細へ到達する |

画像はPNGである。終了証拠は `native/exits.log`。全14PIDが通常終了exit0であり、こちらから利用者のアプリを強制終了していない。スクロール中の上端/下端で文が部分的に見えることと、横幅不足で文字や操作が欠けることを区別した。横切れ・固定Closeの画面外配置は観測していない。

160%では初期本文に冒頭説明と①概要までが表示され、②③の本文には縦スクロールが必要である。冒頭と末尾を別画像で確認しており、結果全体の初期同時可視とは主張しない。標準confirmでも③本文の末尾はスクロールする。番号概要は上部に残る。

本native確認は実描画と合成tail指定の到達であり、実マウス/keyboardでの操作成功の証拠ではない。実MouseWheel/pointer/Tab/Enter/Spaceによるheadless回帰の合格で補強する。nativekeyboard、実マイク/再生、IME/DPI、実API、利用者受入は別である。productionソースは撮影前後で変更していない。
