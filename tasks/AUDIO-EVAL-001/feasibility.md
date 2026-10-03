# 例文読み上げの音声評価：初期確認

2026-10-01。範囲は読み取り専用の実装・接続仕様確認。機能変更、モデル変更、録音の外部送信は実施していない。

## 確認済み

- src/ai.rs の transcribe は WAV を audio/url 入力にする。feedback はテキストの内容評価であり、発音採点を明示的に除外する。
- src/codex.rs は実際の thread/start のモデルについて model/list の inputModalities を確認し、音声対応を確認できなければ送信を拒否する。
- 現在インストール済み Codex の generate-json-schema の UserInput には audio/url と localAudio/path がある。入力形式の存在は確認できたが、モデル・アカウントによる受理や音声理解を保証しない。
- 既存の音声成功テストはモックである。実モデルの発音・強弱・リズム評価の証拠ではない。
- 公式 https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations は Sign in with ChatGPT + api.openai.com/v1 のフローで音声入力を非対応としている。この別経路の制限を、現行の直接Codex認証接続へ無条件には適用しない。

## 結論と次の確認

実装可能性はあるが、現行認証経路・選択モデルで原音を評価できるかは未確認。まずモデル能力の照会、次に許可された非個人の短い確認音声で受理・原音理解を検証する必要がある。利用者の録音は同意前に送信しない。別APIや別サービスへの切り替えは別途承認を要する。

文字起こしとの一致だけで発音点数を出さない。評価対象は例文の読み上げとし、聞き取りやすさ、発音、強弱、リズム、改善箇所を分離する。録音音量・音割れ・無音時間のローカル解析は発音評価とは別である。

## 使用量観測

## 2026-10-01 実接続の確認・検証

- 利用者の承認を受け、インストール済み codex.exe の app-server に initialize / account/read / model/list / thread/start を実行した。検証専用の ephemeral スレッド、read-only、ツール禁止。既存の設定・教材・録音は変更・送信していない。
- account/read の認証方式は chatgpt。
- 保存設定はモデル指定なし、effort medium。thread/start の実返却は model gpt-6.1-sol / modelProvider openai / reasoningEffort medium。
- model/list の取得結果は7モデル。gpt-6-astra、gpt-6-sol、gpt-6-luna、gpt-5.6-sol、gpt-5.6-terra、gpt-5.6-luna、gpt-5.5。全て inputModalities は text/image だけで audio はない。実返却の gpt-6.1-sol は取得一覧にない。isDefault と実返却モデルは一致しなかったため、一覧の既定値を実モデルとして扱わない。
- 結論：現行 WordWeave5 の音声対応確認条件は満たされない。入力スキーマに audio があるだけでは送信できるとは判定しない。音声付き turn/start は実施していない。受理・原音理解・発音評価精度は未検証であり、失敗や成功を捏造しない。
- 検証用 app-server は終了済み。アプリのビルド・全体テスト・モデル変更は不要のため未実施。
- 次の選択肢は、現行認証経路で音声対応モデルが提供されるまで待つか、別方式の調査を別途承認すること。ローカルの音量・音割れ・無音解析は可能だが、発音評価の代替とはしない。

今回の開始観測：input 862869651 / cached input 840012928 / output 1941684 / reasoning output 605940 / total 864811335。
終了観測 2026-10-01T11:09:44Z：input 863184853 / cached input 840322816 / output 1942991 / reasoning output 606019 / total 865127844。
差分：input 315202 / cached input 309888 / output 1307 / reasoning output 79 / total 316509。初期読み取り、終了観測後の記録・最終回答は観測範囲外。子なし。内数は再加算せず、料金や欠測は推計しない。

開始観測 2026-10-01T10:57:10.654Z: input 861862668 / cached input 839088896 / output 1938113 / reasoning output 605666 / total 863800781。
終了観測 2026-10-01T11:01:00Z: input 862299276 / cached input 839460608 / output 1939304 / reasoning output 605852 / total 864238580。
観測差分: input 436608 / cached input 371712 / output 1191 / reasoning output 186 / total 437799。開始前の初期読み取り、終了観測後の記録・最終回答は観測範囲外。子エージェントなし。cached/reasoning は内数であり再加算しない。欠測・料金を推計しない。
