# I0: Praat 7.0.02 Windows runtime probe

2026-10-04。担当は implementer、親が伝達した planner 指定は `gpt-6-astra` / `high`。実行 metadata と開始/終了 token counters は未取得である。入力は承認済み design.md 7/8節（親handoffによる承認伝達）と evidence.md。既存文書の「実装承認待ち」は親の更新対象であり、本担当は変更しない。

完了境界は固定実行物・引数・合成音によるF0・暫定資源制限の実測と引継ぎである。AGENTS.md、development-team.md、KPI、wordweave-changeを読了した。所有は本記録と `artifacts/praat-probe/` のみ。本体/独立テスト/設定/モデル/gateは変更していない。実音声、学習データ、AI送信、システム導入、PATH変更、commit/pushは行っていない。

## 判定

Praat CLIと固定設定のF0出力は利用可能である。ただし **設計の256MiB子メモリ上限では60秒入力を処理できない**。診断限定512MiBでは成功したが、製品上限改訂の承認や全fixtureの十分性を意味しない。親へ設計gapを返した。初版は明示指定した別置き実行ファイルを候補とし、同梱/自動取得は未確定のままである。

## 実行物と再現入口

- [公式Windowsページ](https://www.fon.hum.uva.nl/praat/download_win.html)の x64(v1) リンクだけを取得した。取得先は [v7.0.02公式release asset](https://github.com/praat/praat/releases/download/v7.0.02/praat7002_win-x64v1.zip) である。
- ZIP SHA-256: `D8C741104FAEC65C5E6D77F460140F60B0568143B22506C079A4B7AA0B8D45A0`。
- EXE: `F:/Kazuhiro/GitHub/WordWeave5/tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe`。
- EXE 96,780,624 bytes、SHA-256: `22C8994FD9767A2A5C1F03F21B35E41BB5AC716C628965E5D437033EEDA8FFF4`。
- `--utf8 --version` は UTF-8 31 bytesで `Praat 7.0.02 (August 26 2026)` を返した。`--version --utf8` はUTF-16LEを返したため、版確認では順序を固定する。
- script [pitch.praat](artifacts/praat-probe/pitch.praat) の SHA-256: `3616A00415149E0EF5791F5EA497CBF523468695FAA44FB059D911B6BCA0876E`。スクリプトの最終内容/改行が変われば新しいhashを記録する。
- [Probe.cs](artifacts/praat-probe/Probe.cs) は診断専用launcherである。`Add-Type -Path <Probe.cs>` 後、`PraatProbe.Run(exe, args, outputBase, deadlineMs, memoryBytes)` で実行した。各 `.stdout/.stderr/.result`、合成WAV、script、launcher、ZIPのSHA-256を [probe-hashes.json](artifacts/praat-probe/probe-hashes.json) に保存した。同manifest SHA-256は `95562586975C7FA3473EBA422E25E37959238219D2C965A588C9C44A9FFEB463`。
- artifactsは既存.gitignore対象であり、公式実行物もその中に保持している。hashは今回取得物の識別値であり、上流署名検証済みという意味ではない。

合成は既存 `F:/Program Files/ffmpeg/bin/ffmpeg.exe` の `-f lavfi -i sine=frequency=200:sample_rate=48000:duration=2 -c:a pcm_s16le` と同様の60秒/59ms/60ms/61ms、および `anullsrc=r=48000:cl=mono -t 2` による。20msだけ8kHzを使用した。実行コマンドは `-hide_banner -loglevel error -n` を付け、既存fixtureの上書きを避けた。

## crate実装へ渡す実測済みプロトコル

[公式CLI資料](https://www.fon.hum.uva.nl/praat/manual/Scripting_6_9__Calling_from_the_command_line.html)の独立非GUI起動を使用した。shellを介さず、引数配列は次の順である。

```text
--run --no-pref-files --no-plugins --utf8 <absolute pitch.praat> <absolute mono.wav>
```

日本語/空白を含む `合成 200Hz.wav` のpathが通った。スクリプト本文にpathを補間しない。固定scriptの核は以下である。

```praat
form: "WordWeave acoustic probe"
    sentence: "Input path", "input.wav"
endform
Read from file: input_path$
To Pitch (filtered autocorrelation): 0.01, 50, 800, 15, "no", 0.03, 0.09, 0.50, 0.055, 0.35, 0.14
frames = Get number of frames
writeInfoLine: "WWPITCH", tab$, "1", tab$, frames
for frame from 1 to frames
    time = Get time from frame number: frame
    f0 = Get value in frame: frame, "Hertz"
    if f0 = undefined
        appendInfoLine: fixed$(time, 9), tab$, "unvoiced"
    else
        appendInfoLine: fixed$(time, 9), tab$, fixed$(f0, 9)
    endif
endfor
```

stdoutはUTF-8、CRLF、先頭行 `WWPITCH<TAB>1<TAB>frame_count`、続く各行 `time_seconds<TAB>Hz` または `time_seconds<TAB>unvoiced` である。有効値は小数9桁。欠測を0Hz/NaNへ置換しない。これは提案protocolであり、crate実装者は厳密な行数、有限値、時刻単調性、範囲、出力上限を別に検証する。Praatが返す時刻をRMS hopへ丸めない。

## 実測結果

以下のpeakはWindows Job Objectの `PeakProcessMemoryUsed`（commit memory）であり、working setとは異なる。256MiB=268,435,456 bytes、512MiB=536,870,912 bytesである。

| fixture / 設定 | exit | 時間ms | peak bytes | stdout / F0 |
| --- | ---: | ---: | ---: | --- |
| 2s 200Hz / 256MiB | 0 | 73 | 261,804,032 | 5,280 bytes、195/195有声、0.03–1.97s、200.000616102–200.000623691Hz |
| 2s silence / 256MiB | 0 | 76 | 262,156,288 | 4,305 bytes、195全無声、0.03–1.97s |
| 60s 200Hz / 256MiB | -1 | 73 | 255,291,392 | stdout0、stderr626 bytes、WAV読込の2,880,000*f64確保でOut of memory |
| 60s 200Hz / 診断512MiB | 0 | 577 | 469,676,032 | 166,879 bytes、5995/5995有声、0.03–59.97s、199.999392178–200.000623691Hz |
| 20ms / 256MiB | -1 | 60 | 255,045,632 | pitch floor最低150Hzを要求するエラー |
| 59ms / 256MiB | -1 | 59 | 255,426,560 | pitch floor最低50.847457627118644Hzを要求するエラー |
| 60ms / 256MiB | 0 | 55 | 255,926,272 | 1frame、0.030000000s、200.000609228Hz |
| 61ms / 256MiB | 0 | 55 | 255,619,072 | 1frame、0.030500000s、200.000573781Hz |

200Hz fixtureは広い期待帯域195–205Hzを満たす。自然音声の精度/話者一般の妥当性を証明するものではない。固定floor50Hzでは `duration < 3/50 = 0.06s` をTooShortへ扱う事前判定が必要である。raw Praatはframe_count=0を返さずエラーとなった。floorを入力長へ合わせて黙って変更してはならない。

## 取消し・資源制限の確認範囲

- `UseShellExecute=false`、`CreateNoWindow=true`、`ProcessStartInfo.ArgumentList`、stdin未使用、stdout/stderr分離で非表示起動した。`--send`は使っていない。
- `CreateJobObjectW` → `SetInformationJobObject(JobObjectExtendedLimitInformation)` で `JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` を設定、起動したprocess handleを `AssignProcessToJobObject` へ渡す操作は成功した。
- 本probeは **Process.Start直後にassign** しており、起動からassignまでのraceは残る。本番のsuspended launch→job assign→resumeと、起動直後からのhard上限は未検証である。このlauncherをそのまま製品supervisorへ転用してはならない。
- `wait.praat` の無限loopを専用子として起動し、200ms期限に所有processの `Kill()` と `WaitForExit(5000)` を呼んだ。220msで回収した。別の専用Praat子を併走させた試験も229msで対象だけ回収でき、兄弟子は生存した。兄弟子も最後に自身のhandleで停止・回収した。同名process列挙/一括killはしていない。
- 上記は **own kill+waitの実測** である。Job close killのflag設定は成功したが、稼働中にJob handleを閉じて終了する試験や親異常終了は実施していない。
- このprobeの出力は既知の固定scriptである。launcherの出力上限検査はReadToEnd後であり、製品に必要な読込中のbounded pipe、悪意ある出力/子孫process対策、hard cap失敗とBackendFailedの厳密分類は未検証である。
- 60秒fixtureは512MiB診断値で60秒deadline内に収まるが、256MiBとの互換は否定された。host作業buffer64MiB、work64MiB、復号開始からcleanupまでの総deadline、全rate/ch/雑音fixtureは未検証である。

一度、極短processの終了とメモリsamplingが競合し、probeが `Process has exited` を報告した。診断launcherに終了確認を加え、未記録だった無音caseを新しい出力名で実施した。期待結果や独立テストの変更はない。

## 残るgateと引継ぎ

1. 256MiB→512MiB等の製品上限変更は親の承認確認へ返した。本記録は変更を承認しない。
2. Windows本番supervisorのsuspended launch、Job close kill、bounded pipes、timeout/取消し/cleanup、resource失敗分類の独立テストが必要である。
3. [Praat license](https://praat.org/manual/License.html) はGPL version3以降である。外部processだから法的適合済みとせず、別置き明示指定を初版候補とする。配布物同梱、ライセンス表示/ソース提供等の適合確認は未了である。
4. これはI0 spikeである。RMSは独立担当、最終検証はtest runner、受入は利用者の責任境界に残す。UI/実音声/実AI/全app test・release buildは本担当範囲外であり未実施である。

通常execはsandbox ACL helper障害で開始できず、承認レビュー付きrequire_escalatedで限定操作を実行した。セキュリティ設定、保護機能、実行policyは変更していない。
