# I5: 固定Praat adapterとWindows supervisor

2026-10-04 / ACOUSTIC-HISTORY-001。実装・独立テストへの契約であり、I5の合格証拠ではない。

## 1. 入力と完了境界

[実装計画](implementation.md) I5のplanner指定はdesigner `gpt-6-astra` / `high`。利用者承認済み [仕様](spec.md)、[設計](design.md) 7–8節、[I0追補](design-i0.md) 3/8節に従う。512MiBへの変更承認、同時1件・60秒維持は実装計画と親handoffで確認した。実行model/effort metadata、開始/終了token countersは未取得である。KPIの測定記録は親へ欠測として渡す。

AGENTS.md、development-team.md、KPI、`agent-skills:api-and-interface-design` を読了した。完了は本書の契約、受入対応、文書検査とhandoffである。所有は本書だけであり、本体・テスト・manifest・gateを変更しない。UI変更は非該当であり、egui skillと日本語配置/両軸中央/拡大/狭幅の再設計はI8/I9に残す。

基準HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`、作業treeは既存docs/task、新しい共有crate等がdirtyである。確認したコードは `tools/acoustic-analysis/src/{lib,profile,measure}.rs`、`tools/qwen-audio/src/audio_backend.rs`、[probe](probe.md) とその `Probe.cs` / `pitch.praat` である。coreのtrait/report/enumは変更しない。probeは起動後assignとReadToEndを使うため製品supervisorとして再利用しない。既存audio_backendのbounded read、同一deadline、所有childのkill→waitという考え方を移植し、qwen-audio依存は追加しない。

## 2. 公開APIと固定入力

公開場所はfeature付き `acoustic_analysis::praat`。featureがoffでも既存core API/serdeは同一である。

```rust
pub struct PraatConfig {
    pub executable: std::path::PathBuf,
    pub work_root: std::path::PathBuf,
}
pub struct PraatBackend { /* private: verified identity, pinned file, config */ }
impl PraatBackend {
    pub fn new(config: PraatConfig, deadline: std::time::Instant,
        cancel: &tokio_util::sync::CancellationToken)
        -> Result<Self, acoustic_analysis::AnalysisError>;
}
impl acoustic_analysis::PitchBackend for PraatBackend { /* existing signature */ }
pub fn take_praat_cleanup_warning() -> bool;
```

- `executable` は利用者が明示指定した公式別置きEXEの絶対pathである。PATH探索、同梱、自動ダウンロード、別engine fallback、任意script/追加引数/環境変数入力は設けない。`work_root` はhostが専用rootの排他を保持し、容量計算に含める専用作業親dirである。通常のtemp全体や原音objectの親を渡さない。
- configはpathだけを入力とする。期待版/hash、memory/pipe/frame/work上限はcrate内の固定値でありconfigで緩和できない。テストの縮小上限と任意fake EXEはprivate seamだけに渡す。
- `new` はworker上で取消し・deadline確認、path検証、EXEのstreaming hash、固定script hash、監督付きversion照合を終えてから返る。未確認の版をidentityとして公開しない。`identity()` はI/Oせず、確認済み `EngineIdentity` のcloneを返す。coreはextract前にidentityを読むため、この順序が必須である。
- 初期化で不在/非対応版/hash相違ならBackendUnavailable。hostはこの場合に限り既存MissingPitchBackendを使ってRust測定を続行できる。Cancelled/TimedOut/ResourceLimitは初期化でも全体失敗でありMissingPitchBackendへ変換しない。非Windowsのfeature有効buildはconstructorがBackendUnavailableを返し、制約のないportable childを起動しない。
- 毎起動直前にもEXEと実際に渡すscriptのhashを照合する。constructor確認はidentity確定、起動前確認は差替え防止であり別責任である。EXEを全量Vecへ読まない。64KiB以下のscratchでhashし、chunk間に取消し/deadlineを検査する。

| 固定項目 | 値と根拠 |
| --- | --- |
| engine | `name="Praat"`, `version="7.0.02"`、Windows x64v1 |
| EXE bytes / SHA-256 | `96780624` / `22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4` |
| script SHA-256 | `3616a00415149e0ef5791f5ea497cbf523468695faa44fb059d911b6bca0876e` |
| version command | `--utf8 --version`。stdoutは `Praat 7.0.02 (August 26 2026)` と終端改行一つだけを受理 |
| analysis args | `--run --no-pref-files --no-plugins --utf8 <absolute pitch.praat> <absolute mono.wav>` |

版/hashは [probe](probe.md) の取得物識別値であり、上流署名の検証を意味しない。script hashは本設計でも再計算し一致した。productionへ同じbytesを移す際に改行を変えない。固定scriptを `include_bytes!` で保持し、専用dirへcreate_newで書き、flush/close後にstreaming hashする。別内容へ変える場合はscript版/hashとprobe証拠を更新し、既存hashを偽って保持しない。

WindowsのEXEは読取handleをwrite/delete共有なしで保持する。CreateProcessへ渡すpathが同じfileを指すことをfile identityで照合する。親dir経由の差替えも対象であり、reparse経路を拒否し、検証した親dir列のdelete共有を禁止したhandleをspawn/assign/resumeまで保持する。file identityの直前比較だけでは親dir renameとの競合は閉じない。script/WAVも再open後に書換え・削除共有なしの読取handleをchild終了まで保持する。hashだけを確認してhandleを閉じる案は確認と実行の間の差替えを防がない。これは不正EXEを安全に実行できるsandboxの設計ではなく、固定済み公式実行物の整合性境界である。Partialのpitch_engineはconstructorで検証した候補を示し、F0欠測時に「このengineで測定成功」と表示しない。

2026-10-07 I5-Q1/M2追補：WAVはcreate_new/write/FILE_SHARE_READのwriterへ流した成功bytesをSHA-256に追加し、flush成功後の期待digestと長さを保持してwriterをcloseする。その後read-only/FILE_SHARE_READのpinを取得し、同じhandle上で期待長/digestをstreaming照合してからWork.pinsへ移す。child回収までそのhandleを閉じず、照合後の再openをしない。writer closeとread pinの間の同長別波形差替えも拒否する。WAV固有のpin/I/O/長さ/digest不一致はBackendFailed（F0欠測とBackendWarning）、cancel/deadlineは全体Errorを維持する。scratch≤64KiB、同一deadline、既存cleanupを再利用する。方式・責務・同期hook・独立oracleの正本は [I5-M2 Brief](praat-m2-design.md) §4–10であり、設計提出は実装合格を意味しない。

## 3. PCMと出力protocol

`MonoPcm` は外部から直接traitを呼べるため、extract入口で非空、8–48kHz、60秒以下、全sample有限かつ[-1,1]、profile.pitchがv1と完全一致を確認する。core経由の60ms未満はbackend未呼出でTooShortとなる。直接extractの60ms未満はInvalidInputとし、floor変更やpaddingをしない。

一時音声はRIFF/WAVE、format tag 3（IEEE_FLOAT）、1ch、64bit little-endian、元rate、blockAlign=8、byteRate=rate*8とする。`fmt`は18bytes（cbSize=0）、`fact`はframe数u32、`data`は `sample.to_le_bytes()` の連続である。header合計58bytes、最大 `23,040,058 bytes`。checked arithmeticで各長さを検証する。WAV全体の別Vecを作らず、64KiB以下のbufferで書く。PCM16へ戻すとステレオ平均の半LSBが落ちるため採らない。原音/host PCM/mono sliceを変更しない。

固定版 [PraatのWAV reader](https://raw.githubusercontent.com/praat/praat/v7.0.02/melder/melder_audiofiles.cpp) の `WAVE_FORMAT_IEEE_FLOAT` 分岐は64bitを `Melder_IEEE_FLOAT_64_LITTLE_ENDIAN` に対応付ける。これはsourceによる対応確認であり、実EXEのfloat64/半LSB/60秒fixture試験はI5 runnerの未完了gateである。既存probeのPCM16成功を代用しない。

private parser契約は以下である。

```rust
pub(crate) fn parse_pitch(bytes: &[u8], duration: f64,
    profile: &crate::PitchProfile) -> Result<crate::PitchTrack, crate::AnalysisError>;
pub(crate) fn write_mono_wav(out: &mut impl std::io::Write,
    pcm: crate::MonoPcm<'_>, deadline: std::time::Instant,
    cancel: &tokio_util::sync::CancellationToken) -> Result<(), crate::AnalysisError>;
```

parserはUTF-8厳密decode、LF/CRLFのみ、BOMなし。headerはちょうど `WWPITCH\t1\t<count>`、countはASCII整数1–6001、data行はちょうどcount件、各行は2fieldだけである。最終改行一つを許し、空行/余剰行/余剰field/未知版を拒否する。時刻/HzはASCII十進小数、指数・符号・NaN/Inf・前後空白を受けず、有限性も検証する。scriptが出力する小数9桁を受理する。timeは0–durationかつ厳密増加、Hzは50–800、欠測は完全一致 `unvoiced` だけである。時刻を10msへ丸めない。headerとframe行の不一致はInvalidOutput、count>6001/全stdout>1MiBはResourceLimitである。列挙値・時刻・範囲の不正はInvalidOutputでありF0全部を欠測とする。部分frame列を返さない。

stdoutの切詰め/損傷/数値補正は行わない。F0中央値・分位・有効割合は既存coreが計算する。非空stderrは原文を保存/表示せず、§5の分類に使用して破棄する。成功終了でも診断が出た場合はBackendFailedとし、coreがPartial+BackendWarningを保存する。これは親が確認した保守的な警告保持方針であり、公開traitへwarning fieldを追加しない。

## 4. supervisorの所有と状態遷移

同期libraryをhost workerから呼ぶ。Tokio runtime、queue、detached workerは追加しない。hostの同時1解析gateは復号前から保存可否判定までを所有する。library側はprocess全体で一つのstatic AtomicBoolをRAII leaseとしてclaimし、constructorのversion検査とextractのそれぞれで保持する。インスタンスごとのmutexでは複数backendを防げない。同時要求は即ResourceLimit（内部理由Busy）であり待ち行列にしない。versionとanalysisを同時起動しない。leaseはchild回収・pipe終了・work cleanupを終えるまで解放しない。

| 状態 | 所有・遷移・失敗時 |
| --- | --- |
| Claimed / Prepared | lease、専用work dir、固定file handles。取消し/deadlineを各境界で確認 |
| JobConfigured | anonymous非継承Job handle、completion port。memory/kill-on-closeを設定・照合済み。失敗ならchildを作らない |
| SuspendedInJob | JOB_LIST属性で専用Jobへ所属した状態で作成成功。process/primary thread/必要pipeを所有し、まだユーザーコードは走らない |
| MembershipVerified | 作成時Job所属をIsProcessInJobで確認。確認失敗時resumeせず所有childを回収 |
| Running | ResumeThread成功後だけ到達。pipeを交互にdrainしdeadline/cancel/resourceを検査 |
| Stopping | 終了原因をlatch、TerminateJobObject。所属していないことを明示観測した異常時だけ所有processへTerminateProcessし、待機する |
| Reaped / Cleaned | process待機完了、Job ActiveProcesses=0、pipe終端、全handles close後にwork削除、lease解放 |

Windows手順は `CreateJobObjectW → SetInformationJobObject → completion port関連付け → HANDLE_LISTとJOB_LIST属性設定 → CreateProcessW(CREATE_SUSPENDED | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT) → IsProcessInJob → ResumeThread` とする。JOB_LISTにはこの専用Job handleだけを渡し、値配列をattribute list破棄まで保持する。Jobに `JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_JOB_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`、ProcessMemoryLimit/JobMemoryLimitとも536,870,912bytesを設定する。子孫も合計capと終了対象に含め、breakaway flagを許さない。hostの既存Jobとの制約不整合や属性非対応で作成できない場合も、JOB_LISTを外す・作成後assignへ戻す・無制約実行へfallbackすることはない。JOB_LISTはMicrosoft公式でWindows 10以降/Server 2016以降対応である。

親強制終了はDropを実行しないため、作成から後続assignまでの未所属窓を許さない。CreateProcessW成功時点から専用Jobのkill-on-closeが働くことを要求する。private fault名BeforeAssign/AfterAssignは既存試験互換のラベルとして残すが、今後の位置は所属確認の前後であり、後から所属させるAPIを呼ぶ意味ではない。これは[I5-M1修正単位](praat-remediation.md)のPre/Post gate対象であり、正本訂正だけで実装修正済みとしない。

`lpApplicationName` は検証済み絶対EXE pathを明示し、command lineはWindows argv引用規則で構成する。shell/cmd/PowerShellは介在させない。埋込NULは拒否、空白/日本語/引用符・末尾backslashを独立testする。STARTUPINFOEXのHANDLE_LISTへstdin用NUL、stdout/stderrの子write端だけを渡す。Job、process、host data/credential handlesは継承させない。親の子write端コピーはspawn直後にcloseする。[CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw) と [handle list](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute) に従う。

pipeはanonymous pipeと `PeekNamedPipe` によるavailable bytes確認後、その以下をReadFileする単一supervisor loopとする。stdout/stderrを一方ずつ最大8KiB処理し、毎周回cancel/deadline/child/Jobを確認、待ちは最大5msである。恒常stdoutでstderr/cancelを飢餓状態にしない。blocking ReadToEndと、childが生きたままreader joinする方式は採らない。`ERROR_BROKEN_PIPE` はEOF、それ以外の読取/peek失敗は全体Errorである。proc終了だけをEOF扱いせず、終了後に残りpipeを上限内でdrainする。継承pipeを持つ子孫もJobで終了してからEOFを確定する。

stdoutは1,048,576bytes、stderrは65,536bytesを受信累計で検査する。超過分をVecへappendする前にResourceLimitをlatchしJobを止める。stderrもbounded Vecとしてのみ保持し、trace/log/error textへ出さない。Vecの容量増加も固定上限を超えないよう事前reserve/try_reserve_exactとし、確保失敗もResourceLimit。version検査にも同じsupervisorと上限を使う。

Dropは非panicの安全網である。稼働processを持つguardは所有Jobをterminateしprocessをwait、pipe/file/Jobをcloseする。通常の成功/失敗経路は明示finishで同じ処理を一度だけ行う。Job handleは子へ継承しないため親の異常終了時にもkill-on-closeが効く。PIDやprocess名で探索して他のPraatをkillしない。最後のJob handleをcloseするだけでwaitを省略せず、live childを残して成功を返さない。

## 5. エラー分類とdeadline

同じ観測周回ではCancelled、TimedOut、ResourceLimit、backend/protocol失敗の順に採用し、success直前にもcancel/deadlineを検査する。latch済み資源違反はexit0/正しいstdoutで消さない。現在coreのcheckpointは取消し・deadlineを優先するためそれと一致させる。

| 観測 | 既存AnalysisErrorKind / 結果 |
| --- | --- |
| 不在/版・hash不一致、CreateProcess起動不可 | BackendUnavailable → F0欠測Partial。未検証engineで実行しない |
| profile相違/直接MonoPcm不正 | UnsupportedProfile / InvalidInput → reportなし |
| 取消し/期限到達 | Cancelled / TimedOut → reportなし、所有childをkill+wait |
| pipe/frame/work/memory上限、資源制約設定/assign/read/waitの失敗 | ResourceLimit → reportなし |
| 非zero exit・分類不能な異常終了 | ResourceLimit → reportなし。内部reasonはUnclassifiedAbnormalExit |
| exit0、memory違反証拠なし、非空stderr | BackendFailed → Partial+BackendWarning |
| exit0、空stderr、protocol不正 | InvalidOutput → Partial+BackendWarning |
| 後始末失敗 | cleanup warningを立てる。既存の全体Errorは保持、そうでなければBackendFailed → Partial |

ResourceLimitはこのadapterでは「資源制約内の完了を確認できなかった」を含む。未知異常終了をOOM実測値として表示しない。hostは固定failure code `resource_limit` とこの一般的な文言を保持し、raw stderrから文言を作らない。core enum追加は不要であるが、後続がResourceLimitを「memory上限の実測超過」とだけ表示するならI6接続で修正を要する。未知nonzeroの安全側分類とstderr方針は2026-10-04に親が了承した。資源超過をPartialへ緩和しない。

Jobのhard memory capはcommit確保を拒否するものであり、自動killや原因分類を保証しない。[Microsoftのlimit定義](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information) と [completion port資料](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_associate_completion_port) によれば通常memory通知の配信は保証されない。通知を受けた場合はResourceLimit、既知固定Praatのstderr `Error: Out of memory:` も終了codeに関わらずResourceLimitとする。署名がchunk境界を跨いでも全bounded buffer上で照合する。peakは補助証拠であり「peak<cap」「通知なし」を非超過証明にしない。OOM文字列不一致の非zero終了も上表によりreportを返さない。未知コードまで細分する巨大分類器は作らない。

deadlineはnew/extractへ渡された絶対Instantを上限とし、各段階で60秒を取り直さない。実際の完了がdeadlineを超えた場合はreportなしである。hostは復号前に60秒の共通deadlineを作り、constructor、復号、core呼出へ残budgetを渡す。現core `analyze` は呼出時点からAnalysisLimits.wall_timeでdeadlineを生成するため、I6は残Durationを渡し、host最終deadline検査も行う。既存decoderの内部15秒と無条件に60秒を足してはならない。

OSはTerminateJobObject後のwait/ディスクcleanupを厳密時刻までに完了すると保証しない。期限で解析を止め、回収未確定の間は「取消要求中」とし、live childを放置して期限保証したことにしない。正常系はcleanupを含め60秒内、異常系の停止要求→実回収時間は独立計測gateである。wait不能なら次jobのleaseを解放せず、成功・Partialを出さない。このOS待機の最悪時間は未確認であり、上限を延長承認したとは扱わない。

## 6. 作業領域・保存・回復

一呼出一専用dir（`tempfile::Builder::tempdir_in(work_root)`、非既存名）とし、version検査はPCMなし、extractは固定scriptとmono.wavだけを作る。EXE96.8MBをworkへ複製しない。書込前に合計見積りをchecked arithmeticで計算し64MiB超過を拒否、書込中も実bytesを検査する。hash/出力bufferとmono/WAVを全量二重保持しないため、I4の最大mono23,040,000bytesと合わせても64MiB境界内に収まる。縮小AnalysisLimitsはcore所有bufferの契約であり、adapterは独立した固定buffer上限を守る。hostは復号PCM等も含む合計ledgerをI6で照合する。

削除順はchild+子孫停止確認→pipe終了→script/WAV handles close→専用dir削除である。削除失敗を無視しない。`take_praat_cleanup_warning()` はprocess内AtomicBoolのswapで、初期化失敗時にもhostが取得できる。hostは結果を保存する前に取得し、作業領域残存の固定警告と容量再計算を行う。flagにraw path/原文を保存しない。hostは自身のwork_rootを知っているためpathをlibraryから返す必要はない。

work fileは履歴正本ではない。I6がSaved試行に対してrun intentを先に永続化し、その後constructor/解析、完成report、AnalysisRun、TrialRevision、HistoryRoot、host pointerの順にpublishする。new/extractは原音/object/Progressを書かず、結果保存失敗時の再試行で解析を再実行しない。再起動は残runをInterrupted表示し自動再解析0回である。

停止確認できた自分のworkだけを回収する。再起動時はhost排他取得と前owner/childの終了確認を行い、不明なdirや他host領域を再帰削除しない。owner確認・残dirのledgerはhost保存adapter/I6の責任である。停止未確認dirを残すと容量を使うが、誤削除を避けるための明示的な回復状態であり、存在しないものとしてquotaから除かない。

## 7. ファイル境界・private test seam

I5の本体所有は次の4 Rust files+scriptに収める。各crate manifest/lockと `lib.rs` のfeature接続・test module接続は親の統合専有である。既存 `lib.rs` の型は変更しない。

| file | 責務 |
| --- | --- |
| `tools/acoustic-analysis/src/praat.rs` | 公開config/backend、固定pin、lease、streaming hash、work寿命、coreへのerror変換 |
| `tools/acoustic-analysis/src/praat_protocol.rs` | float64 WAV writer、stdout parser、固定stderr分類。OS/host依存なし |
| `tools/acoustic-analysis/src/process.rs` | private ProcessSpec/ProcessOutput/Limits、非Windows拒否、supervise窓口 |
| `tools/acoustic-analysis/src/process_windows.rs` | raw Windows handles/Job/pipe/起動/回収、unsafeの局所化 |
| `tools/acoustic-analysis/scripts/pitch.praat` | probeと同じbytesの固定script |

private supervisor signatureを `supervise(spec: &ProcessSpec, limits: &ProcessLimits, deadline: Instant, cancel: &CancellationToken) -> Result<ProcessOutput, ProcessFailure>` と固定する。ProcessSpecは `executable:PathBuf, args:Vec<OsString>, cwd:PathBuf`、ProcessLimitsは `stdout_bytes:usize, stderr_bytes:usize, memory_bytes:usize`、ProcessOutputは `stdout:Vec<u8>, stderr:Vec<u8>, exit_code:u32, peak_job_bytes:usize, memory_limit_observed:bool`。ProcessFailureは `kind:AnalysisErrorKind, reason:ProcessFailureReason` としreasonは固定enum（Busy/Setup/Assign/Resume/Pipe/Wait/OutputLimit/MemoryLimit/UnclassifiedAbnormalExit）だけである。stdout/stderrをfailureに付けない。peak/observedは計測と分類用private情報である。

supervisorのOkは「OS回収とbounded収集が完了」の意味に限る。自発終了のnonzeroもProcessOutputとしてadapterへ渡し、`praat_protocol::classify_output(&ProcessOutput) -> Result<(), ProcessFailure>` が既知OOM、memory証拠、nonzero、非空stderrの順で§5へ変換する。classifier成功後だけparseへ進む。監督途中の取消し/上限超過は直ちにErrであり、bounded診断bytesをcallerへ残さない。この分離によりOS層へPraat固有英語文字列を持ち込まない。

製品はv1固定limitsでこの窓口を呼ぶ。任意fake childや縮小上限をpublic APIへ流さない。独立test authorは `tests/support/praat_unit.rs` を所有し、親が `lib.rs` にfeature+cfg(test)でそのmoduleを接続する。support配下に置くのはCargoのintegration test自動発見とprivate unit moduleを二重に起動しないためである。これによりprivate parser/supervisorを検査し、公開traitへtest-only methodを足さない。OS fault注入は `process_windows.rs` の `cfg(test)` 限定 `FaultPoint::{AfterCreate,BeforeAssign,AfterAssign,BeforeResume,BeforeCleanup}` と同期barrier callbackで行う。実装者がtest本体を所有しないよう、hookの呼出だけを本体へ置く。

I5aを先行し、private `praat_protocol.rs` のparser/WAV writer/classifierだけを独立テスト→実装する。WAV writerは非空60秒以下を扱い、60ms未満のPitch拒否はextract入口で行う。一般のwrite失敗はBackendFailed、取消し/期限/bytes上限は各全体Errorにする。exit0かつ非空stderrの固定内部reasonはBackendDiagnosticとし、pipe読込そのものの失敗(Pipe)と混同しない。この追補はpublic APIを広げない。

I5bのfault hookは`cfg(test)`の`process::FaultPoint`と`supervise_with_hook(spec, limits, deadline, cancel, &dyn Fn(FaultPoint, u32) -> Result<(), ProcessFailure>)`へ固定する。u32は自分が作ったchild PIDであり、試験側は独立handleを開いてsuspended/終了を観測できる。hookのErrは通常失敗経路を通し、resume前ならresumeせず回収する。テスト本文はsupport側の独立所有、製品buildにhookは含めない。OS機能の代替stubではなく、実際の起動境界で同期的に呼ぶ障害注入だけである。

fake childはtest author所有の `tests/support/praat_child.rs` を独立binとして親がmanifestへ接続する。`mode`と隔離されたsentinel pathだけをargsで受け、hang、stdout洪水、stderr洪水、終了code、malloc/VirtualAlloc失敗、孫process・pipe保持を再現する。OS supervise testsはprivate ProcessSpec経由で使い、PraatBackendの本番hash照合を解除しない。公開API統合は `tests/praat.rs` から明示的な公式EXE pathで行い、不在なら実Praatgateを未実施と報告する。skipや0件実行を合格にしない。

依存候補はoptional `sha2 = "0.10"`（streaming hash）、optional `tempfile = "3"`、Windows限定optional `windows = "=0.56.0"`（既存crateと同版）である。`praat = ["dep:sha2", "dep:tempfile", "dep:windows"]`、default=[]を維持する。windows features候補はWin32_Foundation、Win32_Security、Win32_Storage_FileSystem、Win32_System_IO、Win32_System_JobObjects、Win32_System_Pipes、Win32_System_Threadingである。追加型が要求するfeatureはcompile時に親が最小限追加する。sha2はrootのWindows hashコードを共有するためにroot依存を作るより小さく、WAV writerは固定形式なのでhound/FFmpeg追加を要しない。

この4filesに収まらずWindows制御とwork回復が肥大化する場合は、I5a protocol/pins（praat_protocol.rs/script）、I5b supervisor（process.rs/process_windows.rs）、I5c adapter（praat.rs）の直列subunitへ分ける。model/gateを変えず、親がdispatchする。本書だけで所有を勝手に拡大しない。

## 8. 受入証拠とhandoff

| AC / seam | 独立期待値と必須失敗例 |
| --- | --- |
| AH-AC01/05、new/identity | hash/版不一致で実行0、MissingPitchBackendでRust値を維持。constructor確認と毎起動差替え拒否を別に検査 |
| AH-AC05/10、WAV/parser | 手書きRIFF decoderでf64 bytesと半LSBをbit照合。手書きframe列で0/800/NaN/Inf/逆転/重複/範囲外/余剰行/6001・6002を検査。期待parserを本体から呼ばない |
| AH-AC06、stderr | exit0+非空stderrはPartial+BackendWarning、raw秘密sentinelがreport/logへ0件。既知OOMはexit0でもreportなし |
| AH-AC09、supervisor | suspended中sentinel未生成、assign失敗時resume0、取消し/timeout/Drop/親異常終了で所有childと子孫が終了、兄弟Praat/childは生存。独立handleでwaitして終了を確認 |
| AH-AC01/09、cap | stdout/stderr cap直前・一致・直後、両pipe同時洪水、false成功exit、memory通知欠落を注入。未知nonzeroはreportなし、peakのみで正常判定しない |
| AH-AC09、寿命 | 成功/取消/parse不正/setup失敗/cleanup失敗全経路でhandle/work ledgerを照合。2インスタンスの同時callが1child以下、取消後lease回復、wait未確定では再開始なし |
| AH-AC05/10/18、実Praat | 固定EXE+hashで200Hz 2秒/60秒、ゼロ、59/60/61ms、8/48kHz、float64半LSB/反相、Unicode空白path。同profileの二hostは同crateを利用 |

実Praatの200Hzは独立合成式による既知周波数、期待帯195–205Hzを使用する。100/400Hzへの倍半誤りは不合格。無音は全frame Unvoiced、59msはcoreのTooShort、60/61msは正の有効frameを期待する。probeの出力値をgoldenにせず、時刻の有限性/範囲/増加、算式由来全長を照合する。上限fixtureでもchild commit<=512MiB、作業<=64MiB、成功時総時間<=60秒、出力<=cap、kill後残process0を記録する。自然音声精度はこの証拠の範囲外である。

親のmanifest接続後に独立author/runnerが実在target名を確定する。基本コマンドは `cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --features praat --locked` と既存coreの `--no-default-features` testである。実Praattargetはpin/pathを明示し別ログへ残す。fake childの成功だけで実Praat・Windows parent crash・float64 gateを通さない。crate fmtと `git diff --check` は本体実装後に実施する。

未解決は、実OSでのsuspended/Job close/pipe/停止回収、float64実EXE、共通60秒のI6接続、異常時回収時間、work orphanのhost回復、二host適合である。同梱・自動取得・GPL配布適合は本書で承認しない。未知exitの安全側分類とstderr方針は親へ伝達済みであり、core public契約を広げる必要が生じた場合は設計へ戻す。

文書検査は相対リンク欠落0、行末空白0、code fence対応、`git diff --check` 成功である。新規文書のため本文を直接検査した。実装/精度/全app buildの合格を主張しない。通常shellはsandbox ACL初期化障害で起動せず、読取専用検査を承認レビュー付き実行へ切替えた。セキュリティ設定は変更していない。
