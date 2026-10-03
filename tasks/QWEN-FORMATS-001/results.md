# QWEN-FORMATS-001 実施記録

2026-10-03。API対応6音声形式を共通ライブラリ/本体/単独ツールへ追加し、本体確認欄を利用者指定どおり整理する。完了境界は設計・実装・隔離テスト・両release・標準/狭幅表示確認である。実音声の有料API再送、commit/pushは含まない。

- 既存dirtyを保持。前タスクQWEN-INTEGRATION-001は利用者が実録音/原音再生/再練習/閉じる/実評価表示を確認し、今回の直前に正常動作として受入済み。助言の客観精度は別である。
- 親は利用者がAstra/highへ変更した旨を受領。実行metadataは未取得。開始counter観測2026-10-03T11:14:02.9703965+09:00、event 2026-10-03T02:13:42.024Z、input943489805/cached917517696/output2237969/reasoning704625/total945727774。初動と子counterは欠測。
- 通常shellはsandbox ACL初期化に失敗し、承認付き読取で回復。harness/settings変更なし。
- [公式Qwen音声入力](https://help.aliyun.com/en/model-studio/qwen-omni)のInput limitsでWAV/MP3/AAC/AMR/3GP/3GPPを確認した。コンテナ名だけでは内部codec条件が特定できないため、ローカル復号を検証する。
- FFmpeg/ffprobeが既存PATHにある。AMR-NB/WB、AAC、MP3 decoderとAMR-NB encoderを確認。ダウンロード/自動インストールは行っていない。
- [FFmpeg MOV demuxer](https://ffmpeg.org/ffmpeg-formats.html#mov_002fmp4_002f3gp)の外部track参照は既定無効。[protocol whitelist](https://ffmpeg.org/ffmpeg-protocols.html#Protocol-Options)で通信を制限できる。外部processの時間・出力上限と取消しを設計に含める。

## 実装・途中検証

- CP0: 仕様独立レビューP1/P2なし。設計のtemp範囲、receiver消失後cleanup警告、通常終了joinの指摘を修正。最後の終了中cleanup通知はshutdown後native警告として親が採用した。計画者が所有/modelを精緻化し親が採用した。
- 親Astra/highがroot U3/UIと独立backend integration期待値、backend実装Astra/high、単独実装Sol6.1/high。helper起動枠上限で2名並行はできず、backend完了後に単独を起動した。親modelは変更していない。実行metadata・子counterは未取得。
- root controller focused25件、root GUI focused16件成功。後者は指定注記の位置/一回表示、不要3群削除、読込cancel/close/対象変更、既存録音と保存非変更を含む。
- core形式10件成功（`formats-core-final.log`）。6形式元bytes・wire・PCM、AMR-WB SID、原path削除後固定、末尾切断/属性変化/動画/外部dref/多stsd拒否、圧縮後とPCM上限分離を確認。API送信は模擬transportだけである。
- 3GP単一stsd内部mono→stereo合成fixtureで、別FFmpeg実行のashowinfoは前半1ch/後半2chを報告。実装は拒否した。AACもin-band変化に備え同じ上限付き復号frame監査を追加した。生stderrは保存・表示しない。
- backend担当のprivate8件/core再利用3件は成功報告。全体gateと独立実装レビューは後続であり、上記focusedだけで完了とはしない。
- 作業中のroot依存checkはbackend新module作成中のcompile失敗、初回REDは未定義API＋テストimport誤り。test importを修正し後続focused成功。過去ログを今回成功へ読み替えない。

## 独立レビューと配布検証

- root `cargo test --all-targets --locked`：10suite/418件成功、失敗0、exit0（`root-all-targets.log`）。初回 `cargo build --release --locked --bin wordweave5`：exit0（`root-release.log`）、EXE更新2026-10-03 12:21:44 JST、12,567,552 bytes。その後のroot production変更はmain.rsの警告文字列のソース折返しのみで、挙動・文字列値は不変。runnerが最終releaseを再buildする。
- 独立実装review Astra/highはP1/P2実装欠陥0。不足指摘はAMR-WB入り3GPと圧縮音声60秒境界の1群。親が追加し、`format-review-boundaries.log`で13件成功。reviewerが追加期待値・結果を限定再確認しP2解消、残存P1/P2=0とした。
- 単独focused26件成功。取消し・IPC終了待ちではloadを破棄し旧audioを復元せずAPI0、単独Host/受取説明を維持する。テスト内moduleがtokio macro内で不安定構文となる失敗1件は作者がtop-levelへ移して修正、期待値は変更していない。
- native合成画像はroot通常・狭幅1.5倍・2倍本文末尾、単独通常・狭幅1.5倍の5枚を確認。指定注記の順序、不要説明不在、折返し、本体送信/閉じる到達を実見した。画像はこのtaskディレクトリ内にある。
- 実マイク・音声出力・nativeキーボード/IME/DPI全組合せ、削除失敗警告の実ダイアログ表示、別PCのcodec配置、追加形式の実API受入は未検証。自動テスト・模擬応答から発音精度を保証しない。API追加送信なし。
- runner Sol6.1/mediumが残り単独all-targets/release/core-only/feature tree/変更範囲整形を実施する。新しいroot成功gateはAGENTSの重複実行抑制に従い再実行せず、同じ固定productionと今回の新規ログを照合する。検証条件を減らしたのではなく実行役と時点を明示する。

## 最終検証の集約

- 単独all-targetsの初回はLNK1102（link時メモリ不足）でexit101。同じ検査を `-j 1` で再実行し13suite/113件成功、失敗/ignore0、exit0。単独releaseもexit0。初回失敗を成功ログに置換せず保存した。
- core-onlyはcheck exit0、lib8件、integration_reuse3件、audio_formats12件が全成功。依存treeにeframe/egui/rfd/image/uuid/windows 0.56はなく、native TLS由来のWindows依存は残る。
- 変更Rust16ファイルの整形検査は14件差分なし、main既存module順とvisual_check既存広範囲差分でexit1。新規main警告行の折返しのみ修正し、その新規差分は再検査で解消した。既存dirtyの一括整形はしていない。
- 最終ログ・成果物hashは [runner-results.md](runner-results.md) を正本とする。使用量の取得範囲・欠測と手戻りは [measurements.md](../../docs/process/improvement/measurements.md) に記録した。親子counterの合算・費用推定はしていない。
- 開発受入は設計・実装・自動検証・合成native画像・独立レビューまで。追加形式での原音出力/実API受理、実機の取消し/閉じる、削除失敗警告、配布先backendは利用者の実機受入として残す。以前のWAV実録音受入からこれらを推定しない。

### 最終成果物

- 本体最終release exit0。`target/release/wordweave5.exe`：2026-10-03 12:52:27 JST、12,567,552 bytes、SHA256 `D859AACB60B0B5060E5F2579ED56DFD99015EDC37C86DA2BC8DB155834AD9C56`。
- 単独release exit0。`target/qwen-audio/release/qwen-audio.exe`：2026-10-03 12:40:32 JST、7,901,696 bytes、SHA256 `560D65F14BC8AE1486EBA4BED2B3BEC505A98B6F6E588C23EFEAC9ABA7D7D063`。
- runner最終入力台帳照合12:52:44でソース変更0。親の文書8件/ローカルリンク44件検査はmissing0、差分空白検査exit0。最終状態のみをplan/todo/currentへ統合し、過去handoffと旧未完項目は書き換えていない。
- 実装・自動検証・配布物作成の境界まで完了。実機受入を自動完了扱いせず、利用者が追加形式の原音再生と評価を確認する。アプリ強制終了、実音声の無断送信、公開は行っていない。
