# 音響解析・履歴機能の実装経過

2026-10-04 / ACOUSTIC-HISTORY-001。実装開始とPraatメモリ上限512MiBへの変更は利用者承認済みである。**実装中であり、完了ではない。** 以下のI0記録はアプリ実装前の検証結果である。

最新の限定完了（2026-10-11）：I2a-P「履歴の保存確定処理」は[独立最終Post PASS/Closed](history-pointer-commit-review.md)。[関連269件の成功証拠](history-pointer-commit-runner.md)を確認した。後続アプリ接続・全体検証・製品EXE更新は未完了であり、現在の再開位置は[tasks/current.md](../current.md)を正とする。

## 今回の到達点

- [実装計画](implementation.md)に保存・移行・解析・比較・本体UI・単独ツールの順序、担当モデル、受入、ファイル所有を固定した。既存の設計成果は[設計結果](results.md)として保持する。
- I0の設計追補`design-i0.md`で、共通解析APIと、二hostで共用する履歴DTO・純粋比較の境界を固定した。汎用永続化Frameworkは追加しない。
- 公式Praat 7.0.02 x64v1を隔離した作業用フォルダーで検証した。システム導入・PATH変更は行っていない。
- 合成200Hzとデジタル無音、Unicode/空白path、対象の子プロセスだけの停止を確認した。詳細と再現用hashは[検証記録](probe.md)にある。
- 元ファイルbytesと再生用PCMを保存設計上区別した。本体の既存MP3用`raw_audio`は再生用WAVであり、それをそのまま原音履歴へ保存してはならない。
- I0時点では本体Rustコード、Cargo依存、EXEを変更していなかった。下記I4/I1aでは独立crateを追加した。本体アプリとEXEへの接続は未実施である。利用者データ、APIキー、Qwen送信は使用していない。commit/pushも行っていない。

## 512MiB承認後の実装

- I4共通解析crateを追加した。PCM検証、RMS/全長/低エネルギー区間、F0 backend境界と集計、版付きReport検証を実装した。独立32件が成功した。Praat実processは未接続である。[実装とfocused結果](core-implementation.md)、[独立テスト](core-tests.md)。
- 保存後のf64値の往復に末尾差が見つかったため、serde_jsonのfloat_roundtripを共通履歴の保存依存と解析のテスト依存へ適用した。期待値の許容幅は緩めていない。
- I1a共通履歴DTOと独立35件を追加した。テスト初回は5成功/30失敗であり、未実装parse/validateによるREDを確認後、35件すべて成功した。[実装記録](history-contract-implementation.md)。
- I4のstrict保存schemaで、未知fieldと必須Option欠落を受理する不備を独立27件で確認した（8成功/19RED）。修正後は既存32件＋27件の59件が成功、本体clippy/fmt成功。全target clippyは独立coreテストの既存`err().expect()`表記5件で未達であり、成功と扱わない。[修正記録](core-schema-implementation.md)。
- I1b-1 readerの契約と未実装scaffoldをrootのlibへ接続した。既存Storage/Progress/commit/backup、Qwen controllerと二host UI、更新EXEは未変更である。参考音声→練習の確認遷移に関する独立設計指摘を修正・再確認した。[設計レビュー](reader-design-review.md)。
- I5のprotocolとWindows supervisorの具体契約を固定し、optional依存とテスト用scaffoldを用意した。実際のPraat起動はまだ未実装である。SHA-256依存はローカルに無かったため公式Rust registryから取得した。音声・利用者データは送信していない。

前回の書込1件は承認審査側の利用上限で未実行となった。利用者の継続依頼後、2026-10-04 11:36:07 +09:00の観測から再開した。迂回せず同じ承認経路で読書きが成功することを確認した。runtime metadata・親子使用量counterは引き続き未取得である。

## I0で判明した条件と解決

仮設計の子メモリ上限256MiBでは、60秒・48kHz・mono合成WAVの読込み時にメモリ不足となった。診断用512MiBでは同じ入力のF0解析が成功し、577ms、Job Objectのピークcommit memoryは469,676,032 bytes（約448MiB）であった。

512MiBへの上限変更は利用者が承認した。同時1件・処理上限60秒・上限超過時の停止は維持する。512MiBであらゆる音声・PCの成功を保証する結果ではない。

短い音声の失敗は別原因である。固定floor50Hzでは59msが拒否され60msが成功したため、60ms未満はF0をTooShortとして扱い、floorを黙って変えない。Rust側の算出可能な指標と区別する。

## 継続時

2026-10-07に同じ実装を再開した。前回中断までにI1b-1の独立34件はコンパイル成功、fixture確認1件成功・未実装33件RED、I5aの独立31件は未実装REDまで完了していた。保存されたprotocol実装を再利用し、I5aの検証とI1b-1の実装を計画指定のAstra/highで再開した。起動指定と実行metadataは区別し、後者は未取得である。

I5aは31件成功、共通解析の既存59件も成功した。parser/WAV/classifierの範囲であり、Windows子process管理や固定Praatの実行成功とは別である。[I5a実装記録](praat-protocol-implementation.md)に対象hash・新規ログ・残る警告を記録した。

I1b-1 readerは独立34件成功、追加path安全性4件も通常3件＋利用者承認の昇格1件で成功した。原音欠損時の部分閲覧と保存前のstrict閉包検証を分離した。独立レビューのpath検証不足P2は追加証拠で閉鎖した。[実装記録](history-reader-implementation.md)／[追加検証](history-reader-path-tests.md)／[レビュー](history-reader-review.md)。writer、Progress移行、backup、jobs/UIは未接続である。

I5bは停止状態でのWindows起動、Jobへの制約設定・割当後の再開、boundedな両pipe読込、取消し/期限/異常時の所有child回収を実装した。実OS21件＋fixture1件とprotocol31件の計53件が成功、既存default59件も成功した。[監督処理の実装記録](praat-process-implementation.md)。固定Praatへのadapter、親crash時の終了実証、全体lease/work回復、実Praat float64等はまだ未完了である。

I5c-1の公開adapter独立20件は、pin gate1件成功・未実装constructor19件REDから、期待値不変で20件成功へ進んだ。[独立テスト記録](praat-adapter-tests.md)／[実装結果](praat-adapter-implementation.md)。合成200Hz/無音/半LSB/60秒を確認し、最大60秒fixtureは共通deadline内4651msだった。接合時に停止未確認を元の取消し/期限エラーと別に取得するprivate accessorを追加し、変更後のprocess/protocol53件・default59件も成功した。I5c-2競合/障害注入・実資源計測・I6接続は未完了である。

Hidden runnerはconsoleを隠すだけで、console無しとは異なる。固定Praat公式実装の開始bannerがstderrへ出ることを合成実行のboolean照合で確定した。GUI両hostと同じconsole無しでは公開20件成功し、非空stderr規則は緩和していない。console付きdebug/CLIではBackendFailedとなる既知制約を実装報告へ残した。一時計測コードは除去済みである。

I1b-2 writer/recovery追補は独立設計レビュー阻止0件。[レビュー記録](writer-design-review.md)。intent上限の不一致12MiBを実装前に512KiBへ訂正し、I0どおりprepareでintentを保存する責務を明記した。統合側が型と署名だけのscaffoldを接続し、`cargo check --lib --locked --offline`成功（38.41秒、sandbox incremental cacheのhardlink代替copy警告あり）。動作は未実装であり、W01–W07の独立テストを先行する。

writer独立35件は手製fixture2件成功・scaffold33件RED（compile成功・権限失敗なし）を確認して実装へ渡した。[独立テスト](history-writer-tests.md)。実装の責務をstore/recovery/mutationの3補助へ分け、export接合を含め本体4＋独立test1とした。readerの既存result関係predicateを純粋関数へ抽出し未公開payload検査と共用する親接合を行ったため、抽出後のreader回帰が必要である。従来の成功をこの変更後の証拠にしない。

I1b-2aは実装後に独立writer35件成功、抽出後reader37件成功/1明示ignoredまで確認した。W06のlock保持中app.lock読取エラーは、独立作者がStorage解放前後の全bytes比較へ修正し、保持対象/期待エラーを減らしていない。本体hashは不変である。[実装記録](history-writer-implementation.md)。独立レビューは確定コード欠陥0だが、writer固有のrecovery junction拒否の2ケースが未実証としてP2受入阻止を残した。[レビュー](history-writer-review.md)。未完成の回復/Progress接合や全体受入へ拡張しない。

I5独立レビューのP1（子作成からJob割当までの親強制終了で子が残る）は、利用者のCodex外PowerShell 5.1/7.6.6でbehavior REDを確認した。事前承認済みの作成時JOB_LISTへM1修正を行い、独立runnerで53件（M1固有1件だけfilter）/実Praat20件/default59件成功。通常sandboxのevent fixtureはSTATUS_INVALID_HANDLEで失敗したが、以前の成功時と同じ非管理者sandbox外では同一sourceのexactと全53件が成功した。環境差は確認、OS内部機構は未確定である。その後、利用者から修正後M1別実行1件成功（0.08秒）を受領し、source/test/EXE/childの現物hash一致と独立Post-gate PASSによりI5-P1/M1をClosedとした。M1待ちの再build停止は解除するが、製品EXE更新・機能全体完了ではない。[修正単位](praat-remediation.md)／[gate](praat-m1-gate.md)／[利用者RED](praat-m1-user-red.md)／[GREEN](praat-m1-user-green.md)／[独立最終判定](praat-m1-post-review.md)／[回帰結果](praat-m1-runner.md)。WAV書込→read pin差替えのQ1は別M2として未解消である。

固定scriptのSHA-256は`3616a00415149e0ef5791f5ea497cbf523468695faa44fb059d911b6bca0876e`と一致した。他PCのGit改行設定による照合失敗を防ぐため、`.gitattributes`でこのscriptだけLFを固定した。`git check-attr`で適用を確認した。全体の改行正規化やGitの利用者設定変更は行っていない。

上限の回答を反映し、本番supervisorのsuspended起動→Job割当→再開、読込み中の出力上限、Job終了・取消し・異常終了回収を独立試験する。今回の診断launcherは起動後assignであり、製品実装へそのまま流用しない。

Praatは別置きの実行ファイルを明示指定する方針で進め、同梱・自動ダウンロードは行わない。配布時のGPL適合は未確認であり、外部プロセスであることだけを理由に適合済みとはしない。

その後、承認済みAH-AC01–18を満たす保存・解析・比較・二host実装へ進む。利用者による実録音・実API・音響精度の受入は、合成音やmockの成功とは別である。

## 2026-10-08 限定受入の更新

W1はwriter junction2件を追加し全37件/独立Post PASSでClosed。M2は生成WAVの差替え対策を実装し、T1/T2 RED→独立15件GREEN。全回帰の旧cancel失敗をD1で切り分け、A1でfixture同定・後始末・ACK記録競合を補正した。新feature172件/no-default59件・fmt成功、利用者の更新版外部M1も1件成功、独立最終Post PASSによりA1とI5-Q1/M2をClosedとした。[最終判定・限界](m2-w1-gates.md)。

旧cancel初回258の根本原因は未確定であり、修正済みとしない。I5c-2の障害/資源、保存回復I1b-2b以降と両host統合は未完了。製品EXE未更新、全体リリース検証未実施。M1用の一時的な再build停止だけを解除した。詳細証拠は[最新要約](../current.md)と[インデックス](../context-index.md)から辿る。

## 計測

2026-10-08の保存回復I1b-2bは、未実装32件のREDから本実装GREENへ進み、独立レビューが指摘したresumeの512KiB exact境界/100k列挙境界を別2件で補った。本体の追加修正なしで独立34＋writer37＋reader37成功、権限依存1件は今回ignoredのまま区別し、[最終Post PASS](history-recovery-review.md)を受領した。旧base計画検証・欠損の部分診断・同request保存再試行・finishまでのheadless受入であり、Progress/UI/解析/送信へは未接続である。次はI2a-0 Path readerの小接合を開始した。

初期調査後の開始観測は2026-10-04 08:52:54.4336750 +09:00。本記録の統合観測は09:04:41.3362803 +09:00であり、最終応答・全検証完了時刻ではない。親はモデルを変更していない。planner/design/probeの指定はgpt-6-astra/high。runtime metadata、入力・cache・出力・推論counter、子の包含関係は取得できず、消費量を推定しない。

先に外部エンジンを合成音で検証したことで、アプリ実装前に資源上限の不成立を発見した。既存設計書は再作成せず、実装境界の追補と実測記録を残す。全体アプリ試験・release更新は本体未変更のため今回行っていない。

09:06:25 +09:00の文書確認は6文書・相対リンク26件、欠落0件。差分空白検査成功、検証用Praatが既存ignore対象であること、本体`src`/`tools`/Cargoに変更がないことを確認した。この追記は確認結果の記録のみである。
