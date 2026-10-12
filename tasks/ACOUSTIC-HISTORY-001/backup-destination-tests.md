# I2d-B 独立テスト作者引継ぎ

2026-10-12。担当はtest author、計画指定`gpt-6-astra/high`、実行metadataと開始/終了usage countersは未取得である。AGENTS.md、development-team、KPI、`agent-skills:test-driven-development`を読了し、承認済みAH-AC04/16・I2-T08/T08b・design-i2.md H4/H5を使用した。追加試験接合は親から独立Pre PASSを受領した。完了境界は作者テスト・compile可能なRED・引継ぎであり、実装、独立runner、Post、全体gateの完了ではない。

対象HEADは`d82e3112584d6c98ca0ceaac0f75537870a40b57`。所有は`tests/audio_history_backup.rs`末尾と`src/backup.rs`末尾`cfg(all(test, windows)) mod i2_export_backup`、本報告、task logsのみ。本体と既存期待値の変更は0である。元backup git blobは`19740c017cb993f984e3321fb69f72665212b3b2`。元sourceと最終作者差分は`logs/backup-destination-original-source.log`、`logs/backup-destination-author-diff.log`へ保存した。他者のtask文書変更は保持した。

| 受入 | 新規ケース / 独立oracle |
| --- | --- |
| T08 / H01–03 | integration 9件: root自身、直下新規、backups、audio-history、assets、既存成分dotdot、case alias、既存外部directory、予定rootの祖先。拒否と再試行時の全bytes/directory ledger・メモリ保持。aliasはOS canonical結果で成立を確認 |
| T08b / H04 | integration 1件: 通常日本語外部と似た名前の兄弟、手製v1 wire投影・deck・detached media exact bytes・manifest hash・prepare成功・完了済み先の再試行拒否・source保持 |
| T08 / H02 | lib 2件: root向け／外部向けjunction祖先を拒否。既存junction fixtureを再利用し、reparse属性・OS canonical結果とledgerを照合 |
| T08b / H05 | lib 3件: write-block中の適格backup成功とblock維持、pending/history/壊れたJSON/媒体の内容拒否維持、normal/external guardからの再入拒否とdrop後の成功 |
| H4試験接合 | lib 2件: strict `BeforeContent → AfterDirectory → BeforeManifest`、各点のnormal/external Reentrant、marker前のexact媒体/deck/wire1とmarker不在。実媒体I/O失敗はAfterDirectoryでfixtureが`assets`通常fileを置き、期待列2点・source不変・fixture sentinelだけ残存・marker/tempなし・新宛先への再試行成功 |

新規は合計17件（integration10、lib7）。共通helper57件の再実装は行っていない。guard副作用を持つlib5件は20秒期限の子processで実行し、PID/exit/timeoutをログに記す。子終了後に親fixture全体を回収する。合成TEMP/TMPだけを当該実行process内で`logs/backup-destination-red-temp`に指定し、環境設定の永続変更はない。実データ・実API・公開操作なし。同期exportに取消しAPIはなく、取消しtokenや競争watcherは追加していない。

## 実行証拠

初回integration compileは成功した。sandbox内の実行は9件が`AssetStore::put`のhard-link公開時`os error 5`でfixture失敗、予定root祖先1件だけ行動REDである。9件を行動REDには数えない。auto-review許可後、同じビルド済みexeだけをsandbox外で実行した。Cargo/依存/OS設定、RUSTFLAGS、target-dirは変更していない。

| 実行 | 結果 | ログ |
| --- | --- | --- |
| 新integration10、sandbox内 | compile成功、fixture失敗9＋行動RED1 | `logs/backup-destination-integration-red.log` |
| 同exe新integration10、sandbox外 | 4 PASS / 6 行動RED、PID31792、exit101、全合成fixture回収 | `logs/backup-destination-integration-red-unsandboxed.log` |
| 同exe既存integration `b0` | 28 PASS / 0 fail、exit0、11.89秒 | `logs/backup-destination-existing-integration.log` |
| 最終lib build | compile成功、exit0、1分37秒 | `logs/backup-destination-lib-build.log` |
| 最終lib `backup::` | 既存6＋新規1 PASS / 新規6 RED、exit101、2.35秒 | `logs/backup-destination-lib-red.log` |

integration REDの6件はroot直下、backups/audio-history配下、case/dotdot alias、予定root祖先で、旧実装が禁止先へ成功することを観測した。通常外部・既存先保持・root自身・assets保護の4件はPASSした。新規observer2件は本体未接続をstrict観測列で検出し、単にhook型が存在することを受入成功にしない。

lib REDはjunction祖先2件、write-block接合2件、observer未接続2件である。最後の2件は実行済みの期待列assertが`[]`と承認列の不一致を検出した**接合RED**であり、実I/O失敗fixtureが起動した証拠ではない。媒体後半失敗と3点gate保持の動的受入は本体3呼出の接合後に初めて確認できる。write-block負例ループも初回pendingの誤ったWriteBlockedで停止したため、残り5副ケースの新経路確認はGREEN時に残る。既存内容拒否の28件を新write-block経路の成功に読み替えない。

新規17件の合計は5 PASS、行動RED10、接合RED2である。既存回帰はintegration28＋lib6＝34 PASS。0件実行・compile失敗・ignoredはない。有限子processは観測順success PID10212/exit101、再入PID9136/exit0、I/O失敗scenario PID15144/exit101、write-block成功PID21752/exit101、write-block負例PID30836/exit101、全てtimeout=falseでwait済みである。sandbox外のlib exe SHA-256は`A55528DE30CF45FBBADF5CA2A3E0CE4447AD39EC073C2C6DAF31AB0F865A4456`。全試験後に合成TEMP配下の残存は0であった。

初回15件時点の`src/backup.rs` SHA-256は`E35DA47F8F41CDBEE0D8FA04ADB321796F6725CDF8570C5A9ECC9F231764EE3F`、追加observer後は`14AB7BA2AAA2183A2FB2B5D800A8DB876003561995AC1A49D7741ABA7F9741A1`。integration source SHA-256は全実行を通して`1BCA808ED56BE0DB8CCE33F9BE4BCD5899A68017822342DFF71AB32A0750E23B`。そのexe SHA-256は`D111F55F68196DFB97C1CCADE9ABFA70E5F636AFF8A56DB173DFBCC5B54BEF87`で固定確認した。

## 実装者・runnerへの境界

実装者は承認H4に従って本体consumerを接続し、次の3点で作者observerを呼ぶ。`pub(super) fn observe_export(stage: ExportStage, dest: &Path, guard: &ExternalExportGuard)`とenumは作者所有moduleに用意済みである。呼出しも`#[cfg(all(test, windows))]`とする。

- `BeforeContent`: external guard取得直後、deck変換も含めた内容preflight前、入力dest。
- `AfterDirectory`: under-guard directory作成成功直後、媒体put前、返却済み解決dest。
- `BeforeManifest`: progress.json書込み成功後、完了manifest書込み直前、同じ解決dest。

本体の3呼出・guard型/取得口・共通Directory/file helper接続は作者が編集していない。hookは未登録ならno-op、thread-local/RAIIで登録を解除し、入れ子・再帰観測を拒否する。hookはI/O結果を置換しない。AfterDirectoryのsentinelはfixture作成物として区別する。

focusedコマンドは`cargo test --locked --test audio_history_backup`と`cargo test --locked --lib backup:: -- --nocapture --test-threads=1`。作者用の新規filterは両targetで`i2_export_backup`。既存integrationだけなら`b0`で28件、既存libは`backup::tests::`で6件である。sandbox hard-link制約が再発する場合は非昇格buildを維持し、hashを確認した生成済み対象exeのみで実行する。

同じguardが点間でdrop/再取得されずmanifestのResult確定まで生存すること、marker後に書込がないことは独立した本体静的reviewが必要である。動的3点観測だけで連続保持や敵対的TOCTOU防止を証明したとは扱わない。全体test/release、実機受入、独立runner/Postは後工程である。

作者作業は完了した。最終差分はbackup.rs末尾482行追加、integration末尾193行追加のみ。`git diff --check`成功。本体所有を実装者へ渡し、既存・新規の期待値は固定する。
