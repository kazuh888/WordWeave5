# I5-M2 Packet Brief: 生成WAVとPraat入力の同一性

2026-10-07 / ACOUSTIC-HISTORY-001 / I5-Q1。設計提出でありPre-gate PASS・実装完了ではない。

## 1. Objective

生成PCMから書いたWAVとPraatが読むWAVの内容一致を、writer close→read pin取得の差替え窓を含め保証する。workへ書ける別processを境界に含む。承認済み[spec](spec.md) AH-AC05/10と親がAcceptedとしたQ1の実現であり、新しい利用者操作を追加しない。

## 2. Scope / assignment

[implementation](implementation.md) I5指定designer=`gpt-6-astra/high`、実行metadata・開始/終了token countersは未取得。AGENTS、development-team、KPI、`agent-skills:api-and-interface-design`全文を読了した。本設計役の所有は本書、design-i5のWAV/pin規範追補、praat-remediationのQ1行のみである。UI非該当につきegui skill、表示・日本語配置・両軸中央・拡大・狭幅は変更対象外である。

実装sourceは `tools/acoustic-analysis/src/praat.rs` のみ、独立testは既存 `tools/acoustic-analysis/tests/support/praat_unit.rs` のみ、計2本を選ぶ。実装者がhelperとcfg(test) hook、test authorがtest本体を逐次所有する。protocol/process/Windows supervisor/script、public API/schema、依存、原音、network、上限は変更しない。既存公開 `tests/praat.rs` は回帰を再利用する。M1は[独立最終判定](praat-m1-post-review.md)Closedを維持する。

## 3. Current evidence

HEAD=`b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`、praat.rs SHA-256=`6BD9D84ADC85E2EBB2E0E546DDCED6AB086405829DBF1B90C3653DAB45C43787`。既存dirtyを保持した。`extract`はcreate_new→`BoundedWriter`→`write_mono_wav`→writer drop→`open_pin`の順であるがWAV digestを照合しない。script/EXEには既存の64KiB streaming `hash`、work/rootにはpin、cleanup/leaseには既存RAIIがある。

`write_mono_wav`は58byte headerとf64 LE payloadを32KiB以下で出力し、flushとcheckpointを行う。`measure.rs`はBackendFailedをF0欠測/BackendWarningへ変換し、Cancelled/TimedOut/ResourceLimitは全体Errorに保つ。原音はこのadapterへ渡されず、借用MonoPcmが入力である。

## 4. Chosen contract / Windows sharing

方式を **書込みstream digestとread pin上digestの照合** に固定する。WAV writerはcreate_new・write・FILE_SHARE_READのみで開く。`DigestWriter<W: Write>`が既存BoundedWriterを包み、成功した`write`の先頭n bytesだけSHA-256へ追加する。partial writeを全要求長としてhashせず、write error/flush error時は完了digestを採用しない。期待digestはファイルの再読から作らない。

WAV全量書込み・flush成功後、期待digestを取得し、writerとその所有Fileを確実にdropする。その後 `open_pin(wav_path,false)` でread-only・FILE_SHARE_READのみのpinを取得する。期待長`58 + 8*N`と期待digestを既存streaming hashで照合し、**照合したそのFile**を`Work.pins`へmoveする。再openしない。script/EXEの既存検証を通過後に限りanalysis runへ進み、WAV pinは子回収完了まで保持する。内容が同一の置換は許容し、file ID一致は本修正の条件としない。

[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)はshare/accessの双方向互換性とhandle寿命中の共有制約を規定する。既存writerのwrite accessが残ればFILE_SHARE_READ-only pin取得は失敗するため、writer closeが必要である。保持するpinはread-only consumerを許し、後からのwrite/delete/renameを拒否する。既存の別writerが残ればpin取得も失敗する。[Rust OpenOptionsExt](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode)の既定値はread/write/delete全共有であり、WAV作成時には明示的に絞る。writer単一handleをchild終了まで保持する代案は、write権限とread-only consumerのshare互換を追加で背負うため採用しない。

read pinのshare値は現実装と同じであり、既存20件の実Praat成功はその互換性の先行証拠である。今回変更後も固定EXE/日本語path/float64/60秒の実Praat回帰を必須とし、公式OS規則だけで固定binaryの再受入を済ませない。

## 5. State / ownership / ordering

| 状態 | 所有と次の条件 |
| --- | --- |
| Prepared | 既存lease、専用dirと祖先pin、借用PCM。checked長/64MiB見積り成功 |
| Writing | writer File、BoundedWriter、DigestWriter。PCMから得たbytesだけ期待値へ反映 |
| WriterClosed | flush済み期待digestと期待長のみ。writer Fileなし。ここでの差替えは次段で検出 |
| ReadPinned | 実WAV read pin取得。新旧writer/rename競合は共有違反として失敗 |
| Verified | 同pinの長さ/digest一致、checkpoint成功。FileをWork.pinsへ所有移転 |
| Running / Reaped | 既存supervisorへ渡すpathとpinが対応。子回収→pin close→dir削除→lease解放 |

中間状態をpublic型や永続化stateとして追加しない。期待digestはこの呼出だけのstack所有、PCMを複製/変更しない。writer/digest wrapperはWAV専用でありscript書込みには適用不要である。

## 6. Error / cancellation / deadline

WAV create/write/flush/pin/metadata/seek/read失敗、期待長・digest相違はBackendFailedとし、PitchTrackを返さずanalysis子を起動しない。WAV用の薄い検証helperが既存`open_pin`/`hash`のBackendUnavailableだけをBackendFailedへ写し、EXE/scriptの分類は変えない。cancel/deadline checkpointのCancelled/TimedOutはそのまま返す。既存checked算術/事前work cap超過はResourceLimit、writer実行中のI/O失敗分類は既存writer契約を維持する。

writer/読取開始・chunkごと・照合後・起動前・finishで同一絶対deadlineとtokenを使う。優先順位はCancelled→TimedOut→既存resource/backend分類であり、chunk hook後にもcheckpointを置く。期限を延長/再生成しない。host worker上の同期処理で完結し、thread/queue/async taskを増やさない。同期OS readの停止所要時間を厳密保証しない既存制約は残る。

## 7. Resource / persistence / recovery

最大WAV=23,040,058 bytes、追加I/OはWAV一回のstreaming読取だけである。書込みscratch≤32KiB、読取scratch≤64KiB、SHA stateと32byte digest（既存hash入力への64桁hex変換可）以外のPCM/WAV全量Vecを作らない。writer scratchと読取scratchの寿命は重ならない。64MiB work/512MiB child/同時1/共通60秒を維持し、EXE/script再hashは既存のままとする。追加読取の時間は60秒内に算入し、上限fixtureで実測する。

Work.finish/Dropが全エラー・hook error・unwindを回収する。失敗時も子未起動ならpin close→自分のdir削除、lease再利用可能へ戻す。cleanup失敗の既存warning、回収未確認時のpin/lease保持を変更しない。flushは一時入力の可読性でありcrash耐久保存の約束ではない。履歴のrun intent→解析→report/Run/Revision/root/host pointerという既存publish順を変えず、途中WAVを原音や結果として保存しない。再起動自動解析0、未知workを削除しない回復契約も維持する。

## 8. Private seams / consumers

実装者は同じ本番extract経路へ、per-callの`#[cfg(test)]`同期hookを設ける。private enumは`WavWriterClosed`、`WavPinAttempted { acquired: bool }`、`WavHashChunk`、`WavVerified`、`BeforeAnalysisRun`とし、callbackは`Fn(point, &Path) -> Result<(), AnalysisError>`である。WavHashChunkは**WAVのみ**実読取chunk後、VerifiedはpinをWorkへ移した後、BeforeAnalysisRunは`self.run`直前で呼ぶ。hookのErrも通常finishを通し、global callback/環境変数/製品feature/public overrideは設けない。test-only crate-visible wrapper経由で既存PraatBackendの同じextract本体へ渡す。constructorの版確認はhook/count対象外である。

`WavPinAttempted`はWAVのopen_pin結果を受け取った直後に、成功/失敗の両方で一回呼ぶ。結果へ`?`を適用する前、hash前、checkpointによる早期return前、Work.finish前である。acquiredは実結果のis_okだけを示し、hookから結果を書き換えられない。競合writerはtest callbackのper-call状態（Option<File>等）でWriterClosedからこの点まで保持し、この点でtake/dropする。その後checkpointを実行し、取消し/期限なら対応Error、そうでなければ元pinがErrなら元pin errorを保持、元pinがOkならhook Errを採用する。元pin errorとhook Errの同時発生でも前者を上書きしない。どのErrも通常finishへ渡し、hook Err時に取得済みpinを閉じてからdirを回収する。競合writerの解放とエラー伝播を混同しない。

DigestWriterはprivate汎用Write wrapperとし、cfg(test)だけのcrate-visibleアクセスで短いwrite/Interrupted/zero/flush errorを独立sinkから注入できること。既存hash loopのWAV用chunk hookはcfg(test)で限定する。公開consumerは既存PitchBackend/core/後続二hostであり、公開signature・保存DTO・F0欠測変換は追加しない。

逐次接合の名前/signatureを以下に固定する。`WavTestPoint`とwrapperは`praat` moduleで`#[cfg(all(test, windows))]`に限定し、DigestWriterはprivate platform内の型を同じcfgで`pub(crate) use`する。test側の参照先は`crate::praat::{WavTestPoint, DigestWriter}`である。製品buildでtest wrapper/reexportを公開しない。

```rust
pub(crate) enum WavTestPoint {
    WavWriterClosed,
    WavPinAttempted { acquired: bool },
    WavHashChunk,
    WavVerified,
    BeforeAnalysisRun,
}
impl PraatBackend {
    pub(crate) fn extract_with_hook(
        &self, pcm: MonoPcm<'_>, profile: &PitchProfile,
        deadline: Instant, cancel: &CancellationToken,
        hook: &dyn Fn(WavTestPoint, &std::path::Path) -> Result<(), AnalysisError>,
    ) -> Result<PitchTrack, AnalysisError>;
}
// DigestWriter<W: std::io::Write> implements Write; fields remain private.
// Methods available through the test-only crate-visible reexport:
impl<W: std::io::Write> DigestWriter<W> {
    pub(crate) fn new(inner: W) -> Self;
    pub(crate) fn finish(self) -> [u8; 32];
}
```

`finish`はinnerをdropしてからdigestを返す消費操作であり、flushを代行しない。callerは`write_mono_wav`（または独立sink試験）のwrite/flush成功後だけ呼ぶ。テスト用extractは本番と同じ検証・起動経路を使い、EXE/script/WAVのpin・digest比較を無効化する引数を持たない。T1/T2のRED基準は元snapshotへのhookだけの適用で作り、その基準sourceは隔離fixture側に残す。最終本体にproduction hash照合を切るcfg(test) bypassを残さない。

## 9. Dependencies / stop conditions

独立Pre-gate PASS後、親が元source snapshotを保存してから実装する。I5実装者/テスト作者/レビュアーの既存Astra/high、runner Sol/mediumを変更しない。実装とテストは役別所有であり同じsourceを同時編集しない。新schema、error enum、公開hook、上限変更、追加process改修が必要なら範囲を広げず親へ返す。固定Praatがread pinと両立しない場合も共有制約を緩和せず再設計する。

## 10. Independent acceptance oracle

| ID / AC | 独立期待値・証拠 |
| --- | --- |
| M2-T1 / AH-AC05/10 | WriterClosed同期点で同長別波形へ上書きしFileを閉じる。独立読取でfixture bytes変更を確認。BackendFailed、BeforeAnalysisRun=0、F0採用0、cleanup正常、次の正常call成功 |
| M2-T2 / AH-AC05/10 | 同点で元WAVを別名へ退避し手書き同長WAVを元名へ置換。独立長さ一致/内容不一致、同じ拒否条件。退避fileも自分のwork内とする |
| M2-T3a / AH-AC05/09 | Verifiedで独立Fileからread成功・write/rename/delete拒否を観測し、正常解析へ進む。この読取観測handleはVerified hook内で閉じる |
| M2-T3b / AH-AC05/09 | WriterClosedで別write handleを開き、per-call状態へ保存してhookから戻る。WavPinAttemptedまで保持し、acquired=falseを記録してから解放する。BackendFailed、analysis開始0、cleanup正常、次の正常call成功を要求する。pin失敗時に解放hookへ到達しない実装は不合格。別ケースで解放後hook Errを返しても元pin errorが保持されることを確認する |
| M2-T4 / AH-AC09 | WriterClosedまたはWavHashChunkでcancel、別ケースでdeadline到達を同期制御。Cancelled/TimedOut、analysis開始0、work回収、次call成功。sleep競争で改竄窓を当てない |
| M2-T5 / AH-AC05/10 | DigestWriterを独立短write/Interrupted/zero/flush-error sinkで検査。期待bytes/digestは手書きheader/f64 LEと独立SHA計算で与え、本番write_mono_wav/hashをoracleにしない。fixtureの同長性だけで内容一致としない |
| M2-T6 / AH-AC01/05/18 | 既存公開20件を固定Praat/consoleなし条件で再実行し、正常200Hz/float64半LSB/日本語空白path/最大60秒、原音・既存work保存とcleanupを確認。変更後の時間/bytes、pin互換を記録 |

T1/T2は修正前snapshotへ同じhookだけを入れた基準で行動RED、修正後GREENを得る。compile失敗はREDに数えない。`BeforeAnalysisRun=0`は解析supervisor入口0の観測であり、同じ経路に他spawnがないコード証拠と合わせ解析child作成0を結論づける。OS process全域の計数、constructorを含めた全起動0とは呼ばない。fixture/helper子が必要ならその数を解析childと分ける。test fixtureは固定binary pinを迂回しない。hook呼出なしで成功した試験、期限未到達、権限不足、0件実行をPASSにしない。

既存protocol/process53（M1別実行は独立記録）、公開Praat20、default59にM2独立ケースを加えた実件数をrunnerが記録する。回帰コマンドを次に固定し、crate fmt/diffも実施する。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --features praat --locked -- --skip process_unit::m1_forced_supervisor_exit_at_after_create_reaps_child_and_preserves_sibling
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --no-default-features --all-targets --locked
```

最初のコマンドは外側Job制約が既知のM1一件だけを除外する。unit targetのfiltered out=1を成功件数に含めず、公開target等の実件数と別記録する。M2や他の失敗を除外するfilter追加は行わない。M1外部GREENを再利用する条件は、M2がprocess/supervisor・M1試験/fixture・Job/起動/回収契約を変えず、現物hashが既存M1証拠と一致することである。確認対象はprocess.rs=`9CA917757074F19D2C799CA9AF369AFA4C0DEFE2C3A8CC9EC727D6714E36AE54`、process_windows.rs=`10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C`、tests/support/process_unit.rs=`CFFA41C1D26EA432581B019B9661A3755853B23E882E3579A2F96F1E7CAE1896`である。runnerは関連fixtureの不変もsnapshotで確認する。条件を満たす場合は[M1独立最終判定](praat-m1-post-review.md)と[利用者GREEN](praat-m1-user-green.md)を引用し、今回再実行成功とは呼ばない。条件が崩れた場合はM1再受入の必要性を親へ返す。アプリ納品時の全体gateは維持する。

## 11. Rollback / risk

未配布dirty差分を親のsnapshotと比較し、失敗なら本Packet範囲だけ差分として保持する。無関係編集/教材/原音をresetしない。試験は合成PCMと所有tempのみ、API/UAC/commit/pushなし。SHA-256の衝突耐性と通常Windows file共有を前提とし、kernel/raw-volume操作や任意process memory改変へのsandbox保証は行わない。追加読取時間と変更後の実Praat互換は実装後未検証であり、検証前にQ1をClosedへ移さない。

## 12. Traceability / handoff status

I5-Q1→AH-AC05/10→praat.rs DigestWriter/read pin/hash/Work→T1/T2/T3/T5、AH-AC09→同deadline/token/cleanup→T3/T4、AH-AC01/06/18→既存core欠測/警告consumerと共通crate→T6である。設計の正本は本書§4–8とdesign-i5の追補であり、並列の未選択方式を残さない。

設計文書・相対参照・source不変hashの確認までが本役の完了境界である。Pre-gate reviewerへ本書と対象hashを渡す。その後test authorへT1–T6の期待値、実装者へ2 source所有とhook配置、runnerへ上記gate/実件数/資源実測、Post-reviewerへRED→GREENと差分を渡す。使用量は親子包含・counterとも未取得として親のKPI記録へ返す。仕様未決0、本体/試験未変更、実装受入は未実施である。

Pre-gate訂正cycle2：M2-P1は§8のpin結果直後hookとT3a/T3bで競合writerの解放順・元error保持を訂正、M2-P2は§10の正確なM1 skip・filtered1別記・外部GREEN再利用条件を訂正した。独立再判定前でありPASSを自己付与しない。
