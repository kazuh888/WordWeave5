# 音声入出力の再利用境界

2026-10-03 / FRAMEWORK-DESIGN-001 / L01。録音・再生・ローカルTTSの設計案である。切出し・二消費者受入は未実施である。
[全体設計](README.md)、[Framework](framework.md)、[候補台帳](library-candidates.md)、[文書仕様](../../tasks/FRAMEWORK-DESIGN-001/spec.md)を前提とする。
「現行」はソースの事実、「proposed」は未実装の契約案であり、公開型名・署名・crateの確定ではない。

## 1. 根拠と最小の責任

| 現行の根拠 | 確認した処理 | 切り離す境界 |
| --- | --- | --- |
| [Recorder / CapturedRecording](../../src/media.rs) | cpal既定マイク、pause/resume/finish/cancel、WAVと機器warningを返す | 機器捕捉と終了結果。評価・保存・送信同意は外 |
| [Speaker / PlaybackSnapshot](../../src/media/speech.rs) | Windows TTSをpoll、WAV再生、pause/seek/rate/volume/repeat | OS資源・再生状態。教科・設定保存・画面は外 |
| [playback](../../src/media.rs) | 指定dirの一時WAVをPlaySoundWで同期再生、後始末 | 旧入口の互換adapter。標準の保存先にはしない |
| [playback_panel](../../src/app/playback_panel.rs)、[audio_controls](../../src/app/audio_controls.rs) | snapshotからActionsを作り、録音中の読み上げ開始を拒否 | WordAppのcontroller/UI方針。音声部へ移植しない |

依存は「host controller → 音声I/O → 機器adapter」である。音声I/OからWordApp、Entry、Progress、Qwen、asset storeを参照しない。
初期の実装候補はWindows/Rustである。cpalの利用だけで他OS、全マイク、音声品質の対応を保証しない。
画面は持たない。ホスト提供WAVの再生利用にマイク初期化やTTS音声列挙を要求しない。
録音・WAV再生・TTSを能力として個別に扱い、読み上げ音声がない環境でも他能力を一律に拒否しない。

## 2. 入出力と操作（proposed）

既存Recorder、Speaker、PlaybackSnapshotを窓口候補とし、機器handleやWindows型は内部に保つ。巨大な共通音声engineは追加しない。

| 窓口 / 操作 | 入力と検証 | 出力と責任 |
| --- | --- | --- |
| 録音開始 | hostの機器選択/互換profile。未対応形式・busyを拒否 | 一つのtakeのhandle。列挙・任意機器選択は将来仕様で採否を決める |
| pause / resume | 同じtake、許可状態。停止後のhandleは再利用しない | 捕捉状態と実捕捉時間。壁時計のpause時間を無音として足さない |
| finish | 所有handleを消費しcallbackを終了 | 所有WAV bytesとwarning。空録音は成功にしない |
| cancel | 所有takeを終了 | その未保存takeだけを破棄。保存済み原本・他instanceは対象外 |
| WAV再生 / TTS開始 | 固定したbytes、または本文とvoice ID、有限rate | ローカル実行handle/snapshot。本文の翻訳・評価・送信はしない |
| snapshot / 再生操作 | 同じ再生世代、秒単位seek、倍率、音量比、repeat | 観測状態・操作結果。snapshot読取りからseekを発行しない |

録音WAVの現行出力はmono PCM16であり、入力channelを平均する。機器の生multichannel streamの完全保持とは主張しない。
ファイル由来の原音は再生・表示のために書換えない。形式変換が必要ならhostが原本と別の派生音声を作る。
現行の空voice IDは英語音声を探索する。共通化時は言語/既定voiceの選択をhostへ置き、旧入口だけで互換挙動を保つ案である。
voice IDはOS列挙結果と照合し、消失・未対応言語を説明付きで返す。日本語音声の存在・読み上げ品質は保証しない。

## 3. 上限と検査の決定主体

| 項目 | 根拠付き現行値 | 将来の決定・検査 |
| --- | --- | --- |
| 捕捉 | 30秒分のframe、sample rateは正値かつ192kHz以下、F32/I16/U16 | hostの録音仕様が長さを決め、機器adapterが形式を検査。現行cap到達後は追加sampleを捨てる |
| 最短録音 | 正常時1/4秒未満を拒否、warning時は非空partialを保持 | 旧profileの回帰対象。上限到達の通知/終了方式は後続仕様で決める |
| 再生速度 / 音量 | 0.5–4.0倍 / 0–1、非有限値拒否 | 部品で検査し、hostが許可範囲をさらに狭め得る。設定保存はhost |
| seek | duration正値/有限、can_seek必要、範囲外は端へclamp | 秒を単位とする。準備中・非seekableを拒否し、無効理由はhostが表示 |
| 評価への入力 | [Qwen](qwen-audio.md)はPCM16、1/2ch、8–48kHz、60秒/6MiB以下 | 評価adapterが別に検査。192kHz録音が保存できても送信可能とはしない |

再利用版の捕捉容量、TTS本文長、poll間隔、終了待ち時間は未決である。後続仕様の承認者と機器adapter担当が実装前に決める。
原音の黙った切詰め・resampleで送信条件を満たしたことにしない。hostは派生物の内容と用途を送信確認へ結び付ける。

## 4. 所有・非同期・状態

| 所有 | 状態・遷移の契約案 | 取消し/障害の境界 |
| --- | --- | --- |
| Recorder instance | 待機→捕捉↔pause→finish結果、またはcancel | callback gateとbufferを同じ同期境界で扱い、終了後のsample追加を防ぐ |
| takeのWAV/warning | finish後はhostへmove | 機器断でも非空partialとwarningを返す現行helperを維持。正常成功へwarningを隠さない |
| Speaker instance | 未load→TTS準備/WAV load→再生↔pause、stopで先頭へ | pending合成の取消しとplayer停止の成否を別に扱う。stop失敗を成功表示しない |
| 再生source/stream | instanceがpause/seek/replay中も保持 | OSイベント・pollで失敗を取得し、終了時は所有資源だけ回収する |
| host controller | 対象、再生世代、操作、未保存状態 | 新source開始時にepochを更新。古いsnapshotのActions/遅延完了を現sourceへ適用しない |

epochによるstale action拒否はproposedであり、現行PlaybackSnapshotにepochはない。新規sourceの置換成功/失敗と世代更新の対応を契約試験する。
録音callbackで保存・AI送信・UI更新を実行しない。TTSは現行pollを利用し、UI threadで完了待ちをblockしない。
旧PlaySoundWの同期workerを即時取消し可能と説明しない。共通版で残す場合は能力の限界を返す。
録音中に読み上げを開始できるか、複数instanceの同時利用、終了時の未保存take確認はhost仕様で定める。現行本体の拒否は互換adapterで維持する。

## 5. 保存・回復・外部評価

通常は「finishでWAV/warning取得 → hostが原本保存 → 参照をhost記録へ保存」である。録音完了はどちらの保存成功も意味しない。
保存失敗時は取得済みWAVを未保存として保持し、同じbytesの保存を再試行する。メモリ消失後の復旧は保証しない。
原本保存後に参照保存が失敗しても原本を消さない。[asset-store](asset-store.md)のdetached原本として回復候補に残す。
再生失敗時は原本を保持し、機器設定確認と人の再操作へ戻す。機器を自動切替して再録音・再生を開始しない。
音声I/O単体は外部送信しないため、遠隔結果不明・再取得は非該当である。Qwen等へ渡した後の取消し/結果不明は評価adapterの契約である。
保存や評価結果を教科の成績へ反映する権限、教材案→可視差分→学習者承認はhost/教科側に残す。

## 6. 互換・二消費者gate・test seam

最初は既存入口のwrapperで本体の30秒、PCM16、pause時間、warning、英語voice既定を保持する。旧データ・設定を切出しだけで変換しない。
公開API版、WAV形式、OS能力、設定schemaを別に管理する。能力非対応を例外的な黙殺や他AIへのfallbackへ変換しない。
二消費者候補はWordWeave5と独立音読ツールである。両hostが必要な操作を仕様化し、同一版をコピーなしで利用してからgate Aを判断する。
評価/学習coreを必須依存にせず、それぞれの画面、voice選択、保存root、送信同意を独立adapterで組み立てる。
試験口は既存CaptureBuffer/encode_recording、WAV/rate/seek検査、機器イベント/poll結果の差替え、hostの保存失敗注入である。
将来テスト作者はpause前後frame、空/短音声、cap直前/一致/超過、機器断partial、NaN、TTS取消競合、stale epoch、再生失敗、保存失敗を検証する。
二hostの設定/資源混線0件、保存再試行による再録音/評価0回、原本改変0件を確認する。模擬機器合格とWindows実機を分ける。
UIの日本語状態通知・無効理由・停止到達性・狭幅/拡大はhost側で検証し、[controls](egui-controls.md)の導入だけで音声UI合格としない。

## 7. AC対応と引継ぎ

| 文書AC | 設計証拠 | 実装/テスト作者への引継ぎ |
| --- | --- | --- |
| FW-AC02/04 | 1–2節、L01 | 教科/評価/UI/保存から独立した既存窓口を最小で切る |
| FW-AC05 | 2–4・6節 | 形式・単位・許可状態・上限・世代の契約を承認後に固定 |
| FW-AC06 | 4–5節 | ローカル停止/保存/遠隔評価を混同せず、warningと原本を保持 |
| FW-AC07/08 | 6節、本表 | 二消費者は未実施。今回の静的検査/独立レビューは[結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)へ |

未決は操作の最終公開形、機器/言語/OS範囲、数値profile、容量/終了待ち、資源競合policyである。後続仕様で決定し、全OS対応・評価品質を推定しない。
本書はD30設計のみである。実装/試験追加/実機操作は行っていない。起動指定gpt-6.1-sol/high、実行metadataとtokenカウンターは未取得である。
