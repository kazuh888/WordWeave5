# 調査根拠

2026-10-04。設計用の読み取り調査である。実音声・API送信・依存ツールの導入は行っていない。

## 現行コードから確認したこと

- `src/app/qwen_reading_ui.rs` は画面を閉じると録音と結果を破棄すると案内する。永続的な音読履歴は今回の新規設計であり、実装済みではない。
- `tools/qwen-audio/src/provider.rs` は音声と参照英文をQwenへ渡し、聞き取った英文・助言・評価不能理由等を返す。音響指標の時系列を測定する処理とは別である。
- `tools/qwen-audio/src/audio.rs` と `audio_backend.rs` は入力検証・FFmpeg/ffprobeによる復号を担う。F0・エネルギー・区間の抽出は未実装である。
- `src/store.rs` のProgressは版1であり、Storageはapp.lockによる単一書込者を持つ。保存時の旧progress日別バックアップも存在する。
- `src/commit.rs` は教材とprogressの二ファイルを復旧可能に保存する。新しい第三の可変ファイルを単に順番に上書きする設計では同じ保証を得られない。
- `src/backup.rs` のバックアップ形式は版1である。現在の参照検証はチャット・教材根拠の添付が対象であり、音読履歴の検証は追加が必要である。
- 原本管理は内容ハッシュによる不変保存を行う。既存inventoryは参照が外れた添付も含み、原本削除APIはない。新履歴の削除・復元・保有量を既存添付と混同しない。

基準HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。行番号は将来の変更でずれるため、ファイルと責任を根拠の単位とする。

## 参照スレッド

「Codexモデル対応範囲」`6abe40e6-d438-83e8-a3ac-060f04f71be4` を読み直した。Qwenの助言と既存ライブラリによるPitch/Energy/Duration/Pause/Rhythmを組み合わせる提案である。提案の存在は、測定の妥当性・発音採点精度・現在の実装の証拠ではない。

## 公式資料と採用判断の境界

| 資料 | 確認できること | 今回まだ証明していないこと |
| --- | --- | --- |
| [Praatの説明](https://www.fon.hum.uva.nl/praat/manual/Intro.html) | 音声解析機能を提供する | 本アプリの入力条件での精度・性能 |
| [PraatのCLI](https://www.fon.hum.uva.nl/praat/manual/Scripting_6_9__Calling_from_the_command_line.html) | `--run` で画面なしにスクリプトを実行できる | Windows配布物での呼出し・取消しの実機動作 |
| [Filtered autocorrelation](https://www.fon.hum.uva.nl/praat/manual/pitch_analysis_by_filtered_autocorrelation.html) | イントネーション向けのF0推定手法と設定 | 読み手全員に同一設定が最適であること |
| [Praatライセンス](https://praat.org/manual/License.html) | GPL version 3以降 | 最終配布形態の適合確認完了 |
| [librosa公式資料](https://librosa.org/doc/latest/index.html) | Pythonによる音声解析の選択肢 | 今回のWindows配布でPython追加が必要十分であること |

Praat CLIとRustでの単純測定を第一候補とする根拠は、F0解析を再実装せず、Python実行環境の追加を避けられる点である。実測精度の優越性を主張していない。バイナリ版の固定、合成音・許諾音声での検証、配布条件は実装前のgateとして残す。
