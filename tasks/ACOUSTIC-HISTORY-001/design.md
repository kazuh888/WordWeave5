# 音響解析・音読履歴の内部設計案

2026-10-04 / ACOUSTIC-HISTORY-001 / D1–D3 / v1。利用者の実施指示を受け、実装前検証へ移行した。未実装である。独立レビューの指摘は解消した。[レビュー記録](review.md)／[実装計画](implementation.md)

## 1. 入力、承認境界、確認した現行

- 入力は [plan.md](plan.md)、親レビュー済みの [spec.md](spec.md)、[evidence.md](evidence.md)、[再利用境界](../../docs/reuse/acoustic-analysis.md) である。原音保持と本設計に沿った実装開始は利用者承認済みである。完全削除は初版から保留し、ごみ箱・復元までとする。未実測の資源上限と配布条件は実装前検証で確定し、変更は別途確認する。
- planner指定は designer `gpt-6-astra` / `high`。実行metadata、開始・終了の入力/キャッシュ/出力/推論カウンターは未取得。完了境界は本設計と文書検査・引継ぎである。
- `AGENTS.md`、開発体制、KPI、`api-and-interface-design` と `wordweave-egui-ui` を読了した。所有は本ファイルのみ。共有差分を保持し、コード・テスト・実音声・API送信・依存導入・公開は扱わない。
- 基準HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。`src/store.rs` の `Storage::at` は app.lock を保持し、`Progress::validate` は現在v1だけを受理する。`Storage::save` は日別旧Progressを残してatomic_writeする。
- `src/commit.rs` はpending intentを先に保存し、deck/progress両方を検証して復旧する。`src/backup.rs` v1は不変assetsを先にpublishし、その後2ファイルcommitする。既存の履歴参照検証はない。
- `src/assets.rs` のhash/no-link/長さ照合、不変publish（同期済みtempからhard-link）、既存物を上書きしない読取を再利用できる。inventoryは切離された添付も含むため、既存assetsのGCや削除判定へ履歴の参照集合を流用しない。
- `AudioInput::bytes()` は固定した入力、`playback_wav()` は別の復号PCMである。`QwenDialog::accept_recording` はfinish出力WAVを持つ。現行の閉じると破棄するUI文言は今回の保存状態に合わせて変更対象である。
- `src/qwen_reading.rs` のPreparedId/generation、SendGate、取消token、`EvaluationResult` の指定値/返却Optionを維持する。保存された送信記録をPreparedIdや次回の送信権限に戻さない。

## 2. モジュールと所有

| 境界／配置候補 | 責任・依存方向 | 所有しないもの |
| --- | --- | --- |
| `tools/acoustic-analysis` 新規Rust library | 検証済みPCM、版付きprofile → 測定・欠測・品質。Rust RMSとPraat adapterを内包 | Entry、Progress、目標、資格情報、履歴root、egui、ネットワーク |
| 既存 `qwen-audio::AudioInput`／host復号adapter | 元bytes検査、復号、PCMの意味を確定。初版は公開済みplayback_wavからPCMを借用/変換 | 音響採点、解析時のQwen利用。共通解析libraryからqwen-audioへ依存させない |
| `src/audio_history.rs` と `src/audio_history/store.rs` 新規host module | 試行/結果DTO、保存・復旧・容量・ごみ箱・系譜、history root生成 | 他hostの保存先、教材/復習の更新 |
| `src/audio_history/compare.rs` | 不変記録から条件別系列と除外理由を作る純粋関数 | 原音変更、良否の推定、勝手な再解析 |
| `src/app/audio_history_ui.rs` と既存Qwen UI/controller | 準備、保存状態、明示解析、毎送信確認、履歴/表/再生、job管理 | 保存完了を先行表示すること、UI threadで復号・hash・外部process待機 |
| `store.rs`／`commit.rs`／`backup.rs` | Progress v2のroot参照と移行、参照閉包検証、backup v2 | mutableな第三の履歴正本、履歴復元による自動評価 |

分割は責任の所在を示す候補であり、この全ファイルを先に空実装する指示ではない。host側の論理履歴契約を文書/DTOで共有し、共通永続化frameworkは作らない。Rust内の呼出しで足り、REST層は設けない。

## 3. データ契約

IDは混同を避けるnewtype（TrialId、RunId、RootHash、BlobHash、ProfileId）。UUIDの試行IDは新しい保存意図に一回だけ発行し、保存再試行では再利用する。hashはSHA-256、保存文字列はUTF-8。JSONのhashは保存した正確なbytesに対し計算し、再serializeしたbytesで照合しない。

| host DTO | 固定／可変の境界・必須フィールド |
| --- | --- |
| `FrozenInput` | 原音BlobRef、bytes/形式/rate/ch/frame数、原音由来（CapturedWav/Imported）、入力warning、固定参照全文+hash、元教材IDと版（任意） |
| `PracticeContext` | 任意GoalSnapshot（本文+版hash）、収録機器/形式、前処理、距離/ゲイン/環境の利用者確認profileと版、Unknownの明示。未設定目標は既知のNoneでありUnknown条件とは別 |
| `ReferenceMatchEvidence` | 原音hash/固定参照hashに結び付く本人の聴き返し後確認と日時、Qwen runごとのAI確認/不一致/評価不能記録。両出所を併存させ、単一enumの後勝ち更新で消さない |
| `TrialRevision` | schema=1、TrialId、FrozenInput、PracticeContext、練習日時UTC+元offset、保存日時+単調保存順、本人/日時確認、Practice/Reference、解析Run参照列、QwenRun参照列、振り返り版列、trash日時、revision |
| `AnalysisRun` | RunId、TrialId、input/decoded PCM hash、profile全設定、engine版+binary hash+script hash、decoder版、crate/指標schema版、実行状態、結果BlobRef、失敗/欠測/quality、開始終了日時 |
| `AdviceRun` | RunId、TrialId、原音hash+固定参照hash、送信先識別、同意した確認内容のhashと日時、指定model/effort、返却Option、provider/parser/prompt版、SendDisposition、検証済みEvaluationResultのみ |
| `HistoryRoot` | schema=1、更新番号、next_save_sequence、TrialId→TrialRevision BlobRef の整列map。旧revision/resultsは不変で保持する |
| `Progress` v2 | 既存全fieldsを保持し `audio_history: Option<RootRef>` を追加。rootに音声bytesや結果配列をinlineしない |
| `RecoveryIntent` | 保存操作ID、基底root hash、操作種別、対象TrialId/RunId、固定入力と期待hash/bytes、新revision案、予定object参照。秘密/raw HTTP本文なし |

教材が編集/削除されてもFrozenInputを辿れる。本文の同一性はexact UTF-8であり空白・大文字の勝手な正規化をしない。目標・収録条件は保存後に書換えず、次の試行の新版へ反映する。振り返りだけは新revisionを追加する。

インポートは本人/練習日時が確認されるまでReferenceである。同じ原音hashの既存記録を示し、別の練習として数える人の確認がなければ新Practiceを作らない。確認で別TrialIdを作っても原音Blobは共有できる。再解析は新RunId、同じTrialIdである。

## 4. 共通解析API案

以下は契約sketchでありcompilable implementationではない。builderは境界で長さ/有限値/rate/ch/profileを検証する。

```rust
struct PcmView<'a> { samples: &'a [i16], sample_rate: u32, channels: u16 }
struct AnalysisRequest<'a> { pcm: PcmView<'a>, profile: &'a AnalysisProfile }
enum SampleValue { Finite(f64), DigitalSilence, Missing(MissingReason) }
struct AnalysisReport { provenance: Provenance, measures: Measures, quality: Vec<Quality> }
fn analyze(req: AnalysisRequest<'_>, engine: &dyn PitchBackend,
           cancel: &CancellationToken) -> Result<AnalysisReport, AnalysisError>;
trait PitchBackend { fn extract(&self, pcm: &MonoPcm, profile: &PitchProfile,
    cancel: &CancellationToken) -> Result<PitchTrack, AnalysisError>; }
```

- Errorは `InvalidInput/UnsupportedProfile/BackendUnavailable/BackendFailed/TimedOut/Cancelled/ResourceLimit/InvalidOutput` に統一し、kind、retry可能性、safe codeを返す。hostが日本語文言と保存済み範囲を付ける。message文字列を分岐条件にしない。
- PitchBackendだけのUnavailable/Failed/InvalidOutputはF0=Missing(Backend...)とし、算出できたRust RMS/全長/区間を含むPartial reportを返す。解析全体のInvalidInput/Cancelled/TimedOut/ResourceLimitはErrであり途中値を保存しない。従ってPraat不在はF0利用不可であり、Rust測定と原音保存は利用できる。
- 出力は時刻の単調性、有限値、F0範囲、区間の範囲/重複、frame上限、出力長をadapter境界で検証する。第三者出力のNaN/Inf、想定外行・欄・文字列は受理しない。
- 同じprofile/PCM/engine版の許容差付き一致を契約試験する。浮動小数点の全環境bit一致や話者一般への精度は契約にしない。
- libraryは同期関数としてworkerで呼び、hostが同時1件を管理する。queueやruntimeをlibraryに持ち込まない。取消しはtoken、callbackでProgressを書かない。
- WordWeave5はStorage adapter、独立ツールは別rootのadapterで論理Trial/Run契約を満たす。共通解析crate版・profile・同じfixtureなら同じ測定を得ること、別root/資格情報/削除が混線しないことを二host受入とする。
- host内部の `commit_operation(op_id, expected_input_hash, mutation)` は同一意図の再試行に同じ結果を返し、同じIDと異なる入力はConflict、同時処理中はBusy。照合と更新を一つのwriter lock下で行う。送信のexactly-onceを保証するAPIではない。

## 5. 保存レイアウトと確定順序

`<Storage.dir>/audio-history/assets/` を履歴専用の不変object領域とする。`AssetStore::new(Storage.dir.join("audio-history"))` のFileBlob/WAVを再利用し、metadata/resultも小さいJSON Blobとして扱う。既存 `<root>/assets` の教材添付inventoryと分ける。recovery intentは `<root>/audio-history/recovery/<operation-id>.json`、短命作業領域は `<root>/audio-history/work/<job-id>/` とする。

1. UIから固定入力を受け、writerが既存root/operation ID、app.lock、pending教材commit、入力・容量を検査する。入力メモリは確定まで保持する。
2. 原音hashとFrozenInputを含むRecoveryIntentをatomic_writeする。**原音publishより先**に関連付け情報を置き、元pathを再読しない。
3. 原音を不変publishし読戻し照合する。TrialRevision、HistoryRootもこの順にpublish・照合する。既存IDへの保存再試行は重複Trialを作らない。
4. 現在Progressのcloneへ新rootを設定し `Storage::save` する。このroot参照のatomic replaceが通常保存のcommit pointである。UIの正本と「保存済み」は成功/読戻し確認後に更新する。
5. intentを完了処理する。後片付け失敗は保存成功と分け、次起動でroot/operation照合により二重適用せず終了する。未参照objectを勝手に削除しない。

全てのProgress write（通常save、教材commit、backup restore）はroot参照を保持する。writer処理は直列化し、worker開始時の古いProgress全体を後から保存しない。mutationを最新Progressへ適用する。教材pendingがあれば履歴書込も拒否する。

`Progress::validate` は構造、`verify_history_closure(root, progress)` はroot→trial→run→原音/resultの長さ/hash/関係IDを検証する。`commit::recover` は両metadataを書き換える**前**に新Progressの履歴閉包も検証する。単純にprogressだけ旧版へrollbackしない。

原音・履歴・バックアップは現行同様のローカル保存であり、暗号化済みとは説明しない。Windows利用者別rootを使い、他hostと暗黙共有しない。資格情報を含めず、通常診断に音声・参照全文・助言全文を複製しない。製品暗号化とクラウド同期は今回の範囲外である。

起動順はapp.lock取得→既存pending commitの全参照事前検証/復旧→Progress移行/読取→履歴の健全性診断→recovery候補列挙である。intentだけなら「原音未保存」、原音だけなら「関連付け回復候補」。基底rootが変わっても既存TrialId/inputが一致する場合のみ最新rootへ取り込む。不一致は競合として人へ返す。回復から解析/送信を起動しない。

通常閲覧の診断とcommit/restoreの厳格検証は別APIとする。既存原音のhash不一致/欠損は当該試行をDamagedOriginalにして再生/再解析/送信を止めるが、検証できた他の結果・本文・他試行は閲覧できる。root/trial自体の破損はその枝を読取不能とし保持する。未知版/破損を空履歴へ置換しない。新rootのcommit/restoreは閉包を厳格検証して拒否するが、起動時の局所欠損だけでアプリ全体の閲覧を失敗させない。pending復旧が拒否された場合は書込を止め、検証可能な保存済みデータを回復画面から参照できるようにする。

atomic_writeのrename成功後に結果が不明な場合はdisk上のroot参照を再読し、予定rootなら保存済み、旧rootなら未確定、どちらでもなければ書込停止・要回復とする。プロセス中断に対する回復設計であり、電源断/媒体故障までの耐久性はWindows実機検証gateである。

## 6. 独立状態と非同期・取消し

| 軸 | 遷移／保存上の意味 |
| --- | --- |
| ローカル保存 | Draft → Saving → Saved。失敗はUnsaved(input保持)またはRecoverable(intent/originalあり)。取消しはCancelRequested→commit照合→Saved/Unsaved/Recoverable |
| 解析 | NotStarted → Running(run intentを保存後) → Complete/Partial/Failed/Cancelled/TimedOut。計算完了後の保存失敗はComputedUnsaved(report保持) |
| Qwen | NotRequested → Confirming → Sending(intentを保存後) → Assessed/Unassessable/ReferenceMismatch/Failed。送信後取消し/通信不明はMayHaveBeenSent、結果保存失敗はReceivedUnsaved |
| 一覧可視性 | Active → Trash → Active。原音/結果はそのまま。同じTrialIdで戻る。処理中は取消要求→停止確定→trash commitし、途中は移動済みと表示しない |

開始はSavedの試行だけである。保存済みrun intentを持ってからworker/transportへ渡す。結果採用は `(TrialId, RunId, input hash, generation, Active)` が一致する場合だけで、保存はhost writerへ送る。各取消tokenは処理別で、明示取消し・trash要求・確定したアプリ終了時にgenerationを進める。ウィンドウを非表示にするだけでは処理generationを進めず、同じcontrollerが完了を保存する。終了確認を取り消した時も処理を失効させない。

解析の取消し/timeoutは途中値をreportに昇格させない。品質理由による正常なPartialと中断を区別する。既存Qwen SendGateは送信開始の競合制御に使い、durable intentだけでは実送信したと断定しない。再起動時のRunningはInterrupted、Sendingは送信/結果不明として示し自動再実行しない。

ComputedUnsaved/ReceivedUnsavedから「保存だけ再試行」は同じRunIdとreportをcommitするだけである。閉じると失われる未保存範囲を表示し、保存再試行/終了取消し/破棄を選べる。強制終了ではメモリ結果の回復を保証しない。後続処理失敗でも保存済み原音と他方の結果を保持する。

Qwen送信確認は固定原音/参照、接続設定generation、今回PreparedIdに束縛する。履歴画面では固定参照を使う別targetを作り、現在の教材Entryを書き換えない。資格情報は既存host別Windows namespaceのまま、履歴/backupに入れない。

## 7. 解析profile v1案と測定の定義

初版候補はPraat filtered autocorrelationとRust測定である。[公式手法](https://www.fon.hum.uva.nl/praat/manual/pitch_analysis_by_filtered_autocorrelation.html)を参照するが、設定の最適性を検証済みとはしない。全設定を保存し、適用変更はprofile新版とする。

| 測定／段階 | 正確な演算契約／欠測 |
| --- | --- |
| 復号・mono化 | 既存入口で8–48kHz/1–2ch/PCM16へ検証済み。sampleは `x=i16/32768.0`。stereoは `(L+R)/2`、rate維持。normalization、AGC、denoise、resampleを追加しない。反相相殺リスクをwarningにしch/ruleをprofileに残す |
| 全長 | `N/sample_rate` 秒（Nはmono frame数）。先頭末尾も含み、発話時間とは呼ばない。N=0は入力拒否 |
| RMS | mono列の全体 `sqrt(sum(x²)/N)`、`20 log10(rms)` dBFS。ゼロはDigitalSilence、有限0dBFSへ置換しない。DC除去なし。計算はf64、正確な値と表示丸めを分ける |
| RMS時系列 | frame幅25ms、hop10ms。境界は整数sample `round(k*rate*0.010)`、幅 `round(rate*0.025)`、末尾は存在sampleだけで計算。時刻はwindow中心。全体RMSはframe平均から作らない |
| 低エネルギー候補 | 各RMS frameが `<= -40 dBFS` またはDigitalSilenceなら低値。対応する非重複hop区間 `[k*hop, min((k+1)*hop,T))` を連結し、200ms以上の連続区間を採用。先頭/末尾を含む。合計=区間長和、割合=合計/T。閾値一致は低値 |
| F0 | floor50Hz、top800Hz、step10ms、candidates15、very_accurate=false、attenuation0.03、silence0.09、voicing0.50、octave0.055、octave_jump0.35、voiced_unvoiced0.14を初期案とする。Praatが返したframe時刻をそのまま使い、原音先頭基準へ検査する |
| F0集計 | 有限かつ範囲内の有効frameのみHz値。有効数/解析frame総数を割合とし、総数0は欠測。中央値/P10/P90は昇順列の `h=(n-1)p` を線形補間する方式を版固定。全無声はMissing(Unvoiced)、短すぎてframeなしはMissing(TooShort) |

P10/P90は信頼区間でも正常範囲でもない。外れ値を無断で消さず、無声と推定不能を0Hzへ変換しない。F0の内部filterをRMSへ流用しない。機器・距離・ゲイン依存のdBFSをdB SPLや校正済み声量と呼ばない。低エネルギーを言語的ポーズや発音の正解に結び付けない。

Qualityは少なくともRecorderPartial、CaptureUnknown、StereoMixed、ClipCandidate、NoVoicedFrames、ShortInput、BackendWarningを持つ。ClipCandidateは元PCMの±端値sampleの存在と割合を記録し、実際のアナログclipを証明したとはしない。SNR、rhythm、発話速度、アクセント得点は初版で算出しない。雑音の低エネルギー偽陰性やF0誤追跡は表示上の限界として残す。

## 8. process・資源・配布gate

固定scriptを固定版Praatの独立子processで実行する。[公式CLI](https://www.fon.hum.uva.nl/praat/manual/Scripting_6_9__Calling_from_the_command_line.html)の `--run --no-pref-files --no-plugins --utf8` を明示し、利用者のPraat GUIへsendしない。shellを介さずarg配列、入力はhost生成の限定path、script本文へ原音名/参照文を補間しない。非表示起動とする。

既存 `audio_backend.rs` のbounded pipe/deadline/own-child kill+waitを設計根拠として再利用する。Praatにはprivate adapterで同等のsupervisorを置く。stdoutは構造化数値のみ、stderrは上限付きで破棄/固定分類しユーザー音声・参照・pathをlogに出さない。自分が起動したprocess handleだけを取消対象にし、同名process一括終了は禁止する。

| 提案上限 | 根拠／超過時 |
| --- | --- |
| 原音6MiB、60秒/48kHz/2ch | 現行入力上限。録音UI30秒は別に維持 |
| 復号PCM16 11,520,000 bytes、mono f64 23,040,000 bytes | `60*48000*2*2` と `60*48000*8` の上界。コピーを無制限に増やさず、host作業buffer合計64MiBを提案 |
| 解析1件/60秒、work 64MiB、child memory 512MiB | 子メモリ上限は2026-10-04に利用者承認。旧256MiBでは60秒合成音が失敗、512MiBで成功した（probe.md）。deadlineは復号開始からcleanupまで同一budget。Windows Job Object等でchild資源制約と終了回収を検証できるまでは提供しない |
| stdout1MiB、stderr64KiB、frame各6001、report JSON2MiB | 数量・長さをread中とparse時に両方検査。超過はResourceLimit、切詰めて成功しない |
| TrialRevision256KiB、root8MiB、recovery intent512KiB | 個々はAssetStoreの12MiB上限内。版/件数に加えserialize後bytesを検査し上限超過は明示拒否 |

hostは新規処理前に見積を表示し、結果保存時にも実bytesで再検査する。解析/Qwen結果のために容量を架空予約した扱いにせず、不足ならComputedUnsaved/ReceivedUnsavedで保存再試行を提供する。処理開始intentの保存自体が失敗したら処理を開始しない。作業領域cleanup失敗は警告し容量に含める。起動時に自分の停止済みjob領域だけを回収し、原音/object/recovery候補は消さない。

Praat/librosaの比較ではPython追加のないPraat案を選ぶが、精度優位を主張しない。採用gateは固定版・binary/script hash、Windows起動/取消し/日本語path、許諾fixtureの基準照合、配布形態とGPL-3.0-or-laterの条件確認である。外部process化だけでライセンス適合としない。初期開発は明示指定した検証済み実行ファイル、製品同梱/自動取得の方式は確認前に確定しない。engine不在でも原音保存/再生/履歴は動く。

## 9. 移行、バックアップ、容量、ごみ箱

- Progress v1読取は既存全fieldを保ってv2/空historyへ移行する。元v1の回復コピーを成功確認まで保護する。初回v2保存はatomicで行い、未知Progress/root/trial/run版は変更せず書込を止める。古いアプリがv2を無視して保存しないようversion拒否を維持する。失われた旧メモリ音声は移行できない。
- backup v2は既存deck/progress/assetsにhistory section（root参照、closure全BlobRef）を追加し、原音、固定参照、全結果版、warning、失敗/不明、ごみ箱を含める。1つの保存済みrootを捕捉し、publish中のrootや未保存reportは含めない。進行中runは記録だけで実行権限を持たない。回復候補は別欄に未確定と記載し、保存済み試行の成功数へ入れない。
- exportは全参照事前照合、容量見積、既存でない宛先へcopy/hash検査、manifest最後の順で行う。途中directoryは不完全としてrestore拒否。資格情報・work/cacheは含めない。destは稼働root配下のassets/history/workを含む全保護領域外に限定し、reparse/linkを拒否する。
- restoreはv1/v2を明示dispatchし未知版拒否。まず入力全体を検証し、同一TrialId/同一revision内容なら重複なし、同一ID/異なる内容なら履歴取込全体を拒否する。既存のtrashも黙ってActiveへ戻さない。差分・追加容量・未収録状態を表示して確認する。
- v1にhistoryがなければ現rootを保持する。v2は既存historyと取込分のunion rootを作る。教材/Progress全体の既存復元意味は変更せず、復元後Progressへこのrootを明示設定する。現在未保存draftの保護とai_calls最大値維持も継続する。
- 全objectを先にpublishし再照合、現在Progressの退避後 `commit::save` でdeck/progressを確定する。失敗は旧rootのままかpending復旧による全体commitであり、部分historyだけを正本へ公開しない。競合/破損ならmetadataを書かない。既存復旧intentにも閉包検証を加える。
- 保持は無期限。履歴専用領域の原音・旧revision/root・結果・trash・回復候補・残存tempを使用量に算入する。同hash Blobは一度だけ数える。現Progressだけに到達する物の合計をdisk使用量として偽らない。初版は不可視の旧objectも自動GCしない。
- 2GiB/80%警告は提案値である。各保存は書く正確なbytesと既存hash重複を数え、合計がちょうどなら許可、超過ならpublish前に拒否する。disk空きは追加の事前目安でありwrite失敗は別処理する。Progress旧backupを含むroot全体のdisk消費は別表示し履歴quotaと混同しない。
- ごみ箱操作も新revision/root分の空きが要る。満杯時は移動済みと偽らず、既存を保ったまま上限変更が必要と示す。「ごみ箱へ移動（容量は減りません）」と明記する。初版に完全削除ボタンはなく、空き回復の自動削除/古いbackupの削除はしない。
- 完全削除は保留である。実装には共有参照・日別Progress・教材commit recovery・旧root・外部backup再取込との削除台帳契約が必要になる。現段階で「削除済みだからbytesが消えた」とは説明しない。この制約は製品範囲の採否に返す。

## 10. 比較系列と画面設計

`CohortKey` は固定参照版、GoalSnapshot版/None、engine+binary/script/decoder/前処理/指標/profile版、原入力形式/rate/ch、既知の収録profile版を含む。PracticeかつActive、本人/日時確認済みだけを既定系列に入れる。不明条件は別の参考一覧へ置き、異条件の理由と除外数を返す。

保存された参照文が同じだけでは、実際にその文を読んだ証拠にならない。`ReferenceMatchEvidence` の本人確認「この例文を読んだ」（原音聴き返し後）またはAI確認が必要であり、未確認は参考表示とする。既存QwenのAssessedをAI確認（精度保証なし）、ReferenceMismatchを不一致、Unassessable/未送信/失敗を未確認へ対応付ける。不一致・確認競合があれば優先除外し、再評価の肯定結果で消さない。確認/不一致の出所を併存表示し、Qwenなしでも本人確認でlocal系列を作れる。不一致を解除する別のuser-visible操作は初版で推測して追加しない。

画面の中心は練習を振り返る順序である。

```text
固定した例文と目標・比較条件
  初回を再生     前回を再生     今回を再生
  同じ練習日付軸: F0 / 信号レベル / 時間・低エネルギー
  数値表と条件差・欠測の理由
  本人の振り返り          Qwenの助言（出所・版）
```

本人が聴き比べと目標への振り返りから上達を判断する構成であり、学習記録の成績や復習間隔へ自動反映しない。

系列は練習UTC→保存順→TrialIdで安定整列する。offsetと保存日時も表示する。選択した今回以前の最初が初回、直前が前回である。1件は前回なし、2件では初回=前回と明記する。再解析Runの選択を変えてもTrial数は増やさない。各試行につき選択profileの一つのRunだけを点にする。

詳細は原音先頭秒のF0/RMS/低エネルギー区間、履歴は練習日時のF0中央値とP10–P90、全体dBFS、全長秒、低エネルギー秒/割合のsmall multiplesと数値表である。単位は分け、値/版/欠測/警告へclick・keyboardで到達させる。0件は理由付き空表示、1件は点、欠測箇所は線を切り補間しない。異条件系列を同一の折れ線へ接続しない。

同じ軸範囲を使う比較グラフでは範囲を明示し、自動拡大で小差を改善に見せない。色は系列識別にのみ使い、上下変化を緑/赤の合否へ変換しない。見出しは「数値の変化」、目標、本人の振り返り、Qwen助言は出所別である。総合点/レーダー/上達の自動ラベルを作らない。

初回/前回/今回は各々原音hashを検証し同じSpeaker経路で個別再生する。開始時に前の再生を止め、完了generationを照合する。再生失敗は該当音声だけのエラーにし、保存済み結果を消さない。比較再生用にgain正規化して原音を置換しない。

UIは準備→詳細（保存/解析/Qwenを別行）→履歴と保存管理の既存eguiパネルで構成する。windowだけの閉鎖は非表示化であり、host controllerと開始済みjobは保持し一覧へ処理中を表示する。再表示で同じ試行の状態へ戻る。結果のUI描画先と処理generationを分け、別の試行を開いても元試行への正常保存を妨げない。処理取消し/アプリ終了と未保存結果の終了確認は6節の規則を使う。

通常幅は履歴リストと詳細の2列、利用可能幅が不足すれば1列へ切替え、グラフ/表/送信確認全文を縦scrollする。行内Frameへvertical layoutを明示し、右境界をwindow内に保つ。取消しと致命エラーは畳まない。状態要約は1行、詳細はclickでも開ける。

日本語action/tabは `src/app/controls.rs` のButton/UiControlsを使い、字面mesh_boundsを基準に両軸中央を維持する。左揃えの履歴行/collapsing headerとは分ける。icon+textはアイコン領域を予約し、空白文字で位置を調整しない。通常paragraphの行間、focus/disabled/hover/accessible labelを保持する。

将来のUI受入は短/長/空日本語、2.0相当のWindows DPIとアプリscale上限、狭幅・低高さ、keyboard/IME/screen readerを分ける。幅640/960/1280 logical pxを候補としてgeometryと実機で確認し、狭い場合でもスクロールで保存状態・取消し・確認全文へ到達できることを判定する。geometry成功を実機合格としない。

## 11. 障害・受入の対応とテストseam

| AC | module/type/state | 独立テスト担当へ渡す観測点 |
| --- | --- | --- |
| AH-AC01 | host controller / 3状態軸、PitchBackend | backend/credential不在でも保存/表示、local経路のTransport呼出0 |
| AH-AC02 | FrozenInput / AssetStore / TrialRevision | path変更、教材編集/削除後のexact bytes/hashと固定本文 |
| AH-AC03 | AudioInput / PcmView builder | 既存profileの境界、codec偽装、元入力不変 |
| AH-AC04 | RecoveryIntent→object→root→Progress | 各境界のfault/kill注入、Saved表示、原音のみ回復、処理開始0 |
| AH-AC05 | Measures / SampleValue / profile | sine/zero/short/無声の独立oracle、単位、DigitalSilence、欠測、数値許容差 |
| AH-AC06 | Quality / render | partial録音/clip/不明条件を再表示、上達・SPL誤称なし |
| AH-AC07 | AdviceRun / ReadingController / SendGate | 接続/入力変更と確認失効、結果分類、返却metadata None保持 |
| AH-AC08 | ComputedUnsaved / ReceivedUnsaved | save fault、保存だけ再試行の同じRunId、解析/POST回数不増、終了3択 |
| AH-AC09 | generation/token / Interrupted / SendDisposition | start/cancel/trash競合、遅延結果不採用、child reap、自動再送0 |
| AH-AC10 | AnalysisRun列 / selected Run | 新旧版共存、旧結果再表示、一試行一点、再解析で練習数不増 |
| AH-AC11 | CohortKey / ReferenceMatchEvidence | key各成分一つ違い/Unknown、本文未確認/Qwen不一致の除外、local本人確認、旧条件不変 |
| AH-AC12 | Practice/Reference / stable sequence | 同時刻、未確認import、重複bytes確認、0/1/2点、欠測、初回前回再生 |
| AH-AC13 | compare pure function / table+plots | 表と描画点の一致、逆向き変化にも良否ラベルなし |
| AH-AC14 | quota plan / Storage lock | 80%/exact/超過、disk/権限/別writer、旧object含む合計、既存不変 |
| AH-AC15 | Active/Trash revisions | 停止確定後の移動、同ID復元、保存失敗未移動、容量減少/完全削除と誤表示しない |
| AH-AC16 | backup v1/v2 / closure / commit | 全版往復、v1現履歴保持、ID衝突全拒否、corrupt参照no partial metadata、復旧中断 |
| AH-AC17 | controls / responsive view | 日本語両軸中心、focus/表/数値詳細、狭幅/拡大/IME/支援技術実機 |
| AH-AC18 | library API + host adapters | 同一fixture同版一致、別root/秘密namespace/目標、片方の復元が他方へ無影響 |

seamはStorage操作ごとのfault hook、検証済みClock/ID供給、PitchBackend fake、Qwen Transport mock、job完了channel、pure comparisonである。これらを増やすため汎用DI frameworkを作らない。fixtureは合成/許諾のみ。Expectedは仕様と独立oracleから定め、現在の実装出力をそのまま正解にしない。

## 12. リスク、採らない案、引継ぎ

第三mutable history JSONはProgress/教材commitとの部分成功を増やすため採らない。SQLiteはこの規模では既存JSON保存との二重transactionが残るため初版では採らない。全immutable rootを残す方式は単純で復旧しやすい反面、metadataも累積する。容量制限と表示で隠さず、GC/完全削除は別の承認済み仕様を要する。

実装前gateは、仕様採否・独立設計レビュー、Praat版/配布/license、Windows資源制約と60秒/512MiBの妥当性、解析profileの基準照合である。子メモリ上限は合成音の実測を踏まえ512MiBへ変更承認済みだが、全入力・全PCの成功保証ではない。完全削除なし・満杯時の上限変更依存は利用者へ返す重要な製品制約である。

handoffは、(1) 原音保存/Progress v2/回復/backup v2を一貫した縦断単位、(2) library一指標と全体版契約、(3) 保存済み試行へのQwen連携、(4) 比較/表/再生とtrashの順をplannerが再評価する。各段階で対応ACの独立テストを先に定義する。backup非対応のまま履歴保存を製品提供しない。

将来のWindows提供ではfocused regressionに加え `cargo test --all-targets --locked` と `cargo build --release --locked --bin wordweave5`、保存/取消し/認証の独立レビューを維持する。本設計では実行しておらず、実機録音・Qwen品質・音響精度・UI受入は未検証である。

文書検査: AC対応は18件/重複0、相対リンク欠落0、行末空白0、`git diff --check` 成功である。新規ファイルのため本書を直接検査した。これは設計の独立レビュー合格や動作検証の代替ではない。
