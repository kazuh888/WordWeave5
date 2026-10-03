# QWEN-FORMATS-001 内部設計

2026-10-03。状態: 親承認済み仕様QF-AC-001～009に対応する採用設計。独立レビュー・計画精緻化・実装後レビューを実施済みで、実施証拠はresults.mdに分離する。
入力は [spec.md](spec.md)、[plan.md](plan.md)、[前統合設計](../QWEN-INTEGRATION-001/design.md) と親の採用判断である。親は仕様9ACとP-F01/P-F02を採用した。PCM16 WAV/MP3 Layer III/AAC ADTS/AMR-NB・WB/3GP系の単一AAC・AMR音声、原bytes不変、単独再生新設なし、シーク用短命検査copyの限定例外を入力とする。
計画者指定はU0 `gpt-6-astra/high`。実行metadata、開始/終了token counter、親子包含関係は未取得であり推定しない。所有は本書のみ、基点は計画記載commitと既存未コミット差分である。
完了境界はコード照合、契約・状態・障害・試験口の記述と引継ぎである。適用Skillは `agent-skills:api-and-interface-design` と `wordweave-egui-ui`。新REST層、一般media基盤、実装、テスト編集、実API送信、公開、再委任は含めない。

## 1. 境界と既存根拠

| 境界 | コード根拠・変更先 | 所有と責務 |
| --- | --- | --- |
| 共通音声 | `tools/qwen-audio/src/audio.rs`、必要時private `audio_backend.rs` | 現行`AudioInput::parse/read_wav`はPCM16 WAV専用。検証済み入力型と多形式decode入口を追加する。音声、format、復号結果、読込jobだけを所有する |
| 共通送信 | 同`provider.rs` | `EvaluationSnapshot`は既に音声を所有する。固定`format:"wav"`を検証済み型のwire名へ変更し、base64は引き続き`bytes()`から作る |
| 本体controller | `src/qwen_reading.rs` | `clear_audio`は準備・結果・旧音声を先に失効させる。検証済み型を受ける追加入口だけで新音声を採用する |
| 本体host | `src/app/qwen_reading_ui.rs` | 現行`select_wav`の同期読取を共通jobへ置換し、再生用PCMとjob寿命を所有する。`Speaker::play_wav`は`src/media/speech.rs`のメモリ再生を再利用する |
| 単独host | `tools/qwen-audio/src/main.rs`、`gui.rs` | FileDialogと`read_wav`の実呼出は`main.rs`にある。jobはNativeAppが所有し、controllerは検証済み音声だけを受ける。再生UIは追加しない |
| process終了 | root `src/main.rs`、単独`src/main.rs` | event loop返却後の共通job停止・回収を呼ぶ。UI動作中のjoin待機は行わない |
| feature/依存 | 同`lib.rs`・`error.rs`、manifest/lockは親所有 | 新APIはcoreにexportする。短命copyに`tempfile`を使う場合もGUI/MCP/Windows API依存をcoreへ持ち込まない |

U2の計画所有に単独`main.rs`、U3にroot `src/main.rs`を追加する必要がある。`session.rs`は音声bytesを持たず、現行session遷移の変更が必要にならなければ触らない。root Recorderの30秒上限、学習保存経路、資格情報namespace、HTTP transport/認証は前統合設計を継承する。

## 2. 追加型契約

```rust
impl AudioInput {
    // 既存: strict PCM16 WAV、FFmpeg不要。挙動と署名を維持する。
    pub fn parse(bytes: Vec<u8>) -> Result<Self, SafeError>;
    pub fn bytes(&self) -> &[u8];
    pub fn info(&self) -> &AudioInfo;
    // 追加: 同期の検証/復号。GUI threadから直接呼ばない。
    pub fn decode(bytes: Vec<u8>, cancel: &CancellationToken) -> Result<Self, SafeError>;
    pub fn playback_wav(&self) -> &[u8];
    pub fn wire_format(&self) -> &'static str;
}
pub fn read_wav(path: &Path) -> Result<AudioInput, SafeError>; // 既存互換
pub fn read_audio(path: &Path, cancel: &CancellationToken) -> Result<AudioInput, SafeError>;
pub fn take_audio_cleanup_warning() -> bool;
pub fn shutdown_audio_jobs();
impl AudioLoadJob {
    pub fn start(path: PathBuf) -> Result<Self, SafeError>;
    pub fn try_take(&mut self) -> Option<Result<AudioInput, SafeError>>;
    pub fn cancel(&self);
}
impl ReadingController {
    pub fn set_validated_audio(&mut self, label: String, audio: AudioInput) -> Result<(), SafeError>;
}
impl GuiController {
    pub fn clear_audio(&mut self) -> Result<(), SafeError>;
}
```

`AudioInput`はprivate `bytes: Arc<[u8]>`、既存`info: AudioInfo`、private format enum、private `playback_wav: Arc<[u8]>`を持つ。全fieldを非公開にして、呼出側の未検証format付与を不可能にする。Debugは現状どおりmetadataだけであり、bytes/pathを出さない。PCM16 WAVでは原bytesと再生bytesのArcを共用する。圧縮音声では原bytesと復号PCM16 WAVを分けて保持し、cloneはArc共有である。
`AudioInfo`の公開fieldは追加/削除しない。`byte_len`は常に送信原bytes長、`sample_rate/channels/frames/duration_seconds`は検証した音声の値である。圧縮入力のframesはdecoderが出力したPCM frame数であり、metadataのduration丸め値を採用しない。再生用WAVサイズは6MiB制限に含めない。
private formatは`Wav/Mp3/Aac/Amr/ThreeGp`程度とし、wireは`wav/mp3/aac/amr/3gp`へ固定する。`.3gpp`は同じコンテナの選択名aliasである。拡張子からwire名を作らず、内容を検証して決める。
実体形式表示のため`ReadingView.audio_format: Option<&'static str>`、`PreparedPreview.audio_format: &'static str`、`GuiModel.audio_format: Option<&'static str>`を追加する。`wire_format`の既知値からWAV/MP3/AAC/AMR/3GP表示へ変換し、元の拡張子表示を検証形式の代用にしない。公開structをliteralで作る既存fixture等の追従は実装/テスト所有を守って直列化する。
`set_validated_audio`は現行`set_audio`と同じ編集可否・世代更新・確認失効を行ってから型を採用する。`set_audio`は録音と既存利用者用strict WAV入口として残す。再parseや再openを行わない。単独`clear_audio`はeditableを検査し、audio/音声metadata/表示名を消し、読み込み失敗後の旧音声送信を防ぐ。

## 3. 内容検査とbackend

読込は選択pathを一度openし、通常ファイルを確認し、6MiB+1まで読む。read chunk間でcancelを検査し、以後は取得bytesだけを入力とする。空・超過を先に拒否し、拡張子はFileDialog filterにしか使わない。録音は既存strict parseを使い、自動変換しない。

| 内容 | 許容と経路 | wire |
| --- | --- | --- |
| RIFF/WAVE | 既存strict parserでPCM16、1/2ch、8～48kHz、完全なRIFFサイズ/align/非空/60秒を検査。非PCM16は今回対象外 | `wav` |
| MP3 | ID3またはMPEG frame候補を検査後、強制MP3 demuxでLayer IIIを確認・全復号 | `mp3` |
| AAC | ADTS header候補、強制AAC demux、codec AACを確認・全復号。M4A一般は含めない | `aac` |
| AMR | `#!AMR\n`または`#!AMR-WB\n`、強制AMR demux、NB/WBを確認・全復号 | `amr` |
| 3GP/3GPP | 先頭の有効`ftyp` boxと仕様の`3gp*` brand条件、強制MOV demux。AAC/AMR-NB/WBの音声1 streamだけ。video・他stream・外部参照は拒否 | `3gp` |

圧縮系はffprobeでstream構造・codec・rate/chを検証し、ffmpegで末尾まで復号する。duration metadataだけで成功を判定せず、欠落したdurationでも全復号の有限frame数で確定できる場合は扱える。decoder出力0、破損/終了失敗、許容外rate/ch、時間が確定しない場合は拒否する。`-t 60`等の切詰め、`-ar/-ac`による補正は使わない。decoder delay/paddingはdecoderが出力するframe数に含まれる範囲で判定し、独自の許容秒数を加えない。60秒丁度は受理し、1 frame超過は拒否する。
再生出力はraw PCM16 little endianを上限付きで回収し、検証後に既知サイズのWAV headerを自前の小helperで付ける。pipe出力WAVの未知RIFFサイズをstrict parserへ渡さない。出力上限は検査rate×ch×2×60 byteに1 frameの超過検出を加えた値であり、最大11,520,000 byteが有効payloadとなる。raw長がframe境界に合わなければ拒否する。復号済WAVを`parse`へ戻して6MiB上限で誤拒否しない。

backendはapp executable隣の`media-tools`、次にPATH内の明示ディレクトリを探索する。空/相対PATH entryを使わず、CWD暗黙検索をしない。絶対pathのffprobe/ffmpegを`Command`へ渡しshellを介さない。見つかった組の使用可否を検査し、不在/起動不可は専用エラーとする。自動導入やネットワーク取得は行わない。
MP3/ADTS/AMRは元bytesをstdin pipeへ渡し、ffprobe/ffmpegとも強制demux、protocol whitelist `pipe`を使う。3GPだけはシーク用にランダム名のprivate temporary directory内の一つの入力copyへ書き、両toolで同copy、強制MOV demux、protocol whitelist `file,pipe`を使う。MOV外部参照読込は`enable_drefs=0`および`use_absolute_path=0`で無効化する。外部参照付入力が成功する余地を除くため、3GPの外部drefを拒否する検査または等価な確実なbackend拒否を独立fixtureで証明する。protocol制限だけを外部ファイル参照拒否の証拠にしない。
両processの合計実行deadlineは15秒、std child processを非同期job workerから監督する。stdoutはprobe JSONを64KiB、decodeを上述PCM上限へ制限し、stderrはnullへ流してraw文言を表示/保存しない。stdin writer、stdout reader、process監視を並行し、過大出力、cancel、deadlineではkill→wait→writer/reader joinの順でpipe詰まりを解除する。3GPのstdinはnullとする。不要なmetadataを出力させず、strict decode errorを成功へ読み替えない。`wait_with_output`による無制限収集は禁止である。
`-xerror -err_detect explode -reinit_filter 0`だけで全切断・属性変化拒否を保証したとは扱わない。ADTS/AMR/MP3にはboundedなframe境界走査を設け、header/tagの許容構造、各frame長、終端一致、rate/ch一貫性を検査する。MP3はLayer IIIの妥当なheaderとID3の有効長を検査し、AMRはNB/WBの各frame種別長、AACはADTS frame lengthとfrequency/channel属性を検査する。decodeが有効prefixだけを再生して終了0でも、境界走査の失敗で拒否する。一般tag/container parserの自作へ広げず、合法な入力を除外する必要が生じれば仕様へ差し戻す。
3GPは単一音声streamだけでなくsample descriptionの単一性も検査し、途中rate/ch変更を上記decoder設定と組み合わせて拒否する。stsd一件やdecoder flagの存在だけを証拠にせず、異なる属性のサンプルを含むfixtureで拒否を実証する。MP3 free-formatやAAC channel configの特殊値等で自作走査が長さ・属性を確定できない場合は推測せず仕様profileとの適合を差し戻す。

## 4. 取消し、データ所有と永続化順序

実装前補足（親採用）: ffprobeのframe出力だけでは3GP途中のsample rateを取得できないため、3GPの復号では最初のfilterにashowinfoを置く。stderr=nullの例外として、上限付きdrainで固定rate/ch属性だけを検査し、他の診断行を即破棄する。raw stderrを保存・表示・返却せず、上限超過や属性変化は失敗としてprocessを回収する。自動resample後ではなく前段を検査する。MP3 free-format/AAC PCEの長さ・属性判定不能は仕様に明示しAudioUnsupportedとする。

統合補足：同じdecoded-frame監査をADTS AACにも適用する。header走査だけではin-band SBR等の復号属性変化を保証できないためである。3GPの単一stsd内でmono→stereoへ変わる合成fixtureも親が作成し、独立ashowinfoで変化を確認した。実codecの全variant網羅とは区別する。

`AudioLoadJob`はcancel tokenと一回受領channelを所有する。workerのJoinHandleと同tokenはprivate process内registryが所有し、start時にfinished workerを回収する。`start`はthread spawn失敗を`SafeError`で返し、pathはworkerへmoveする。`try_take`は非blockingで結果を最大一度だけ返し、切断/panicを黙ってpendingにせず安全な失敗へ変える。
`cancel/Drop`はtokenを立て、UI threadでjoinを待たない。workerがprocessのkill/wait、drain回収、temporary directoryの明示closeを完了してから終端結果をpublishする。正常受領時は既に回収済みである。取消し後にreadyだった成功は返さない。Drop後のworkerも同じcleanup経路を走り、受信側が消えた結果は捨てる。ファイルI/OのOS待機自体をthread cancelで強制解除できるとは主張しない。
`shutdown_audio_jobs`はregistryへの新規受付を閉じ、全token取消し→全handleをregistryからtake→lock解放→joinする冪等なprocess終了処理である。rootは`run_native`返却後、単独は`run()`返却後に実行し、正常アプリ終了がworkerを先に打ち切らないようにする。終了開始後のstartはInvalidState/NotSentで拒否する。kill/waitとcleanupの完了保証をUI応答性のために捨てない。
3GPのcopy順序は「原bytes確定→temporary入力作成→probe→decode→全child回収→temporary削除→型構築/結果publish」である。削除失敗は専用AudioCleanup errorとし、同時にprocess内AtomicBoolのpending warningを立てる。`take_audio_cleanup_warning`はswap(false)で未通知の有無だけを返し、path/内容/回数を返さない。本体`qwen_reading_window`はdialog=Noneでも毎frame回収して`notify_error`、単独mainは毎frame回収してnoticeを表示する。これにより閉じたdialogのreceiverに警告が消えない。正常process終了の最終cleanupで発生した警告は呼出側がshutdown後にも回収し、画面終了後の通知手段をrunnerが確認する。UI終了後の通知可否は削除成功の証明と分ける。
アプリ異常終了/OS停止ではcopy残留があり得る。自動履歴、学習記録、チャット保存、次回復元の入口は追加しない。無関係なtempを起動時に掃除しない。
temporary copyは検査に限る仕様上の例外であり、確認後送信はメモリsnapshotだけから行う。原ファイルを編集/削除せず、確認・再生・送信時に原pathを再openしない。credential保存順序、評価結果の本体メモリ寿命、単独MCP cache期限は既存のままである。

| hostイベント | 順序・採用条件 |
| --- | --- |
| FileDialog取消し | 既存音声・確認を維持し、jobを作らない |
| 選択確定 | 旧確認/有効音声/再生を消す→旧job cancel/drop→新jobを一つ開始。読込中は準備/送信/録音/再生を無効化 |
| 再選択 | 旧jobの所有を捨て新jobへ置換する。古いreceiverをpollせず、異なる選択の遅延成功を採用しない |
| 成功 | 現在のjob、対象有効、hostが入力可能、非取消しの全条件を満たす場合のみ型をcontrollerへ渡す。本体は同じ型の再生WAVを保持する |
| 失敗 | 旧音声を復元せず、送信0、固定の安全な理由を示して再選択可能にする |
| 閉じる/取消し/対象変更/単独親IPC close | job cancel/dropを評価worker取消し・media停止と共通cleanupへ入れる。結果pollより先に適用する。registryが残存workerを保持し、警告flagはhostが回収する |

新jobに世代番号を追加する場合はchecked incrementでありwrapさせない。現在の`Option<AudioLoadJob>`だけをpollする所有構造で旧channelを完全に切り離せるなら別の世代counterは不要である。本体のPreparedId/対象版/generationと単独SendIntent/Grantは既存のまま使い、読込完了から送信を起動しない。単独ではUIボタン無効化に加えcontrollerのaudio=Noneがsend/prepare_intentを拒否する。

## 5. エラーとUI

`SafeError`/`SendDisposition::NotSent`へ統一する。空/size/時間/非対応/破損/IOは既存codeを使い、表示は多形式入力に合う固定日本語へ更新する。backend不足と15秒decode timeoutは専用codeを追加し、既存`Timeout`の180秒評価表示を流用しない。元path、ffmpeg stderr、音声内容、資格情報はerrorに含めない。追加variantに伴うexhaustive matchと同一buildのIPC serialize/deserialize対称性を検査する。

本体は`ChooseWav`/「WAVを選ぶ」を意味の合う音声選択へ変更し、FileDialog filterに6拡張子を含める。読込中は一行状態と取消しを表示する。原音再生は検証済み再生WAVだけを`Speaker::play_wav(...,1.0)`へ渡す。既存録音失敗時の再生可能原音保持は別の既存経路で維持する。圧縮原bytesをSpeakerへ渡さない。
本体の冒頭説明直後、読む例文の見出し前に「※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しない。」を一度だけ置く。確認欄の送信先URL/要求model・effort/目的/結果受取先の行と、旧「上の音声…課金…」段落全体を除く。確認欄は音声、英文、送信ボタンを維持する。冒頭の既存Alibaba Cloud送信説明は維持し、課金告知を別箇所へ新設しない。PreparedPreview内部のhost等を削除せず、設定欄の接続先・要求値と結果欄の実metadata区別も維持する。単独側の説明削除へ横展開しない。
本体`controls::Button`、単独`controls::button`をそのまま使う。両者は日本語glyphのmesh boundsで両軸中央配置し、Buttonのfocus/disabled/keyboard/accessible labelを保持している。通常本文の行送りを変えず、長い形式説明・日本語名・注記はwrapする。操作列は既存horizontal_wrapped、本文ScrollArea、閉じる/取消しfooterを使う。480×640 logical相当・scale 1/1.5/2で横overflowと低いviewportの操作到達を確認する。OS DPIと実音声出力はnative別証拠である。

## 6. 受入対応と独立試験口

期待値は仕様からテスト作者が独立作成し、実装都合で変更しない。次表は承認済み正式ACを全て対応付ける。

| 正式AC / 要求 | module/型/状態 | 試験と観測点 |
| --- | --- | --- |
| QF-AC-001 / QF-01 | audio format、`decode/read_audio`、両host選択 | 各形式の合成fixture、拡張子変更、実体形式表示、AAC ADTS対M4A、AMR NB/WB、3gp/3gpp |
| QF-AC-002 / QF-03 | strict parse、bounded backend | 0/6MiB/6MiB+1、60秒/1frame超過、rate/ch境界と途中変更、偽duration・破損・末尾切断、video/多stream/外部参照拒否、要求/参照アクセス0 |
| QF-AC-003 / QF-02/04 | `AudioInput`、snapshot、provider、playback Arc、本体Speaker | mock HTTPでwire名とbase64復元bytes完全一致、原path書換/削除、PCM WAV metadata一致、decode >6MiBかつ原bytes≤6MiBの有効例、再生/停止 |
| QF-AC-004 / QF-02/03/04 | `AudioLoadJob`、host cleanup、PreparedId | 再選択/close/取消し後遅延成功、旧確認無効化、新読込失敗後要求0、FileDialog取消し保持、再生資源回収 |
| QF-AC-005 / QF-03/06 | backend runner、SafeError | 不足/起動失敗/非zero exit/出力超過/timeout/cancel、WAVはbackend不要、原因別表示と要求0、自動再送なし |
| QF-AC-006 / QF-05 | 本体renderのみ | 削除3説明群と旧末尾段落の不在、注記完全一致・出現1回・例文前 |
| QF-AC-007、QF-AC-008 / QF-02/05/06 | View/Preview/GuiModel、snapshot、既存controls | 実体形式/音声情報/参照全文、設定/実metadata/単独説明残存、接続変更と再確認、keyboard/日本語/狭幅/拡大のnative画面 |
| QF-AC-009 / QF-06 | temp guard、lib features、controller/単独IPC | 原本不変、copy寿命/削除失敗通知、core-only check/test/tree、資格情報分離、本体結果は本体だけ・MCP結果境界維持 |
| 横断 / QF-07 | 固定差分のrunner/reviewer | plan必須gate、隔離native証拠、実API未送信・未検証の分離 |

backendの探索、clock/deadline、process出力/exit/cancelはprivate helper境界を置く。内部unit testから狭いfake runnerを注入し、環境PATHを並列testで書き換えない。公開の汎用process traitは不要である。実codec fixtureは既存FFmpegで生成した合成音だけを追跡し、実録音/学習データを使わない。FFmpegが必要な実decoder試験を黙ってskipして全形式合格と報告しない。
jobの遅延成功とcancel競合はbarrier/channelで決定的に制御し、sleep依存にしない。tempは通常完了/失敗/cancel各経路の削除と残存childなしを確認する。cleanup失敗とOS強制終了残留は正常経路の削除保証と分ける。

## 7. リスクと引継ぎ

採らない案は`parse`自体の外部backend依存化（既存core利用者の挙動変更）、全入力のWAV再送（原音snapshot契約違反）、metadataだけの長さ判定（偽装/末尾破損の見逃し）、GUI threadでdecode/join（取消しと閉じるの停止）、seek不能pipeだけで3GP受理を保証する案（末尾moov等の失敗）である。
検証上の未決は、stream途中のrate/ch変更をbackendが暗黙変換しないこと、frame走査と全復号strict設定が末尾破損を拒否すること、3GP external dref拒否である。該当fixtureで保証できない場合は成功扱いにせず、実装前に仕様担当へ境界を差し戻す。15秒はprocess実行deadlineであり、OSファイルI/O停止やアプリ強制終了後の削除保証とは異なる。正常終了は両mainでshutdown後にcleanup警告flagを回収し、必要時だけrfd::MessageDialogの警告を表示する方式を親が採用した。警告には秘密・pathを含めず、一時音声コピーの削除失敗を知らせる。
配布先でFFmpeg/ffprobeが使えること、全codecが実APIで受理されること、Windows audio/IME/DPIは未検証である。原bytesを保持したまま再生だけ復号する構造の妥当性と、実際の音質/音声出力成功は別証拠である。
引継ぎは独立仕様/設計レビュー→計画所有反映→独立テスト期待値→U1～U4実装→固定差分検証である。両main.rs所有追加、temp例外説明、専用backend/cleanup error、公開追加署名とregistry終了境界を計画者へ返す。本工程では文書リンク・差分・所有のみ検査し、application buildは行わない。
