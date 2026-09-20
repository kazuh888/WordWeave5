# Windows Harness タスク

## PATH補完0.5.1（2026-09-20）

- [x] 保存システム・実行ユーザーPATHを補完し、CodexとVoltaで共有する。明示パス欠損時は停止する。
- [x] 修正前の再現失敗とオフラインコンパイルを確認する。
- [x] Windows全167件テスト、releaseビルド、起動時PATHが空の状態での保存PATH探索、独立レビュー。[記録](../validation/path-search-0.5.1.md)。

## 操作制御0.5.0（2026-09-20）

仕様: [操作制御](../docs/design/interaction-controls.md)。以下は新規変更の検証であり、下の0.4.0合格とは別である。

- [x] diagnostics: 永続・容量制限・秘密非保存・保存失敗の分離、書き込み中断を試験。
- [x] effort: model/list、設定保存、生成時指定、返却値表示をモック通信試験。
- [x] chat-trash: ごみ箱移動・復元・原本と根拠保持・保存失敗・復元後readonlyを試験。添付名を変更。
- [x] recording-controls: 一時停止時間を除外、キャンセル確認と既存録音保護を実装。純粋ロジックと保存失敗を試験。
- [x] speech-controls: ±5秒、pause/resume/stop、0.5〜4倍と位置表示を統合。境界判定・Windows無音API試験。
- [x] Windows全159件テスト・releaseビルド成功、差分レビューと更新手順。[検証記録](../validation/controls-0.5.0.md)。
- [ ] 実機確認: マイク、TTSの速度/音質、CLI0.153.4実生成のeffort反映。

## 0.4.0までの履歴

- [x] 基準検証：変更前のWindows全テスト76件成功。現行版の証拠とは分ける。
- [x] media-assets：原本保存・検証・バックアップ。新規モジュールと一時フォルダー試験。
- [x] run-journal：生成前保存、応答保存、結果不明・exact turn再読込。再生成せず応答を回収して保存する。
- [x] chat-input：録音、認識候補の確認、注釈、添付履歴・再生・読み上げ。UI・AI・会話の互換試験。実機確認は下記。
- [x] material-review：左右差分、理由hover、固定根拠と引用強調、登録前検査。教材モジュール・比較UI試験。
- [x] 保存統合：媒体を含む退避・復元、教材登録失敗時の回復、旧データ保護。
- [x] 最終自動検証：Windows全119件テスト成功、releaseビルド成功、限定再レビュー、ソース・EXE指紋、検証記録と操作手順。
- [ ] 利用者の実機評価：実Codexの媒体入力・結果再取得、マイク・原音再生・TTS停止、Windowsペン、比較画面のUX、保全済み実記録での更新確認。Androidは対象外。

現在の証拠と未検証範囲は [validation/harness-0.4.0.md](../validation/harness-0.4.0.md) を参照する。マイク／ペン／実Codexは実装完了と区別して扱う。
