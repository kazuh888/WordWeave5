# QWEN-FORMATS-001 実装計画

2026-10-03。状態: U0～U5の設計・実装・自動検証・両release更新・独立レビュー完了。追加形式の実機/実API受入は別途である。初版の暫定計画を、承認済み [spec.md](spec.md) と [design.md](design.md) に沿って更新した。shutdown後cleanup警告を含む設計を採用済みで、残存P1/P2はない。実施結果と実機未検証範囲は [results.md](results.md) を正本とする。
計画者の所有は本書と [todo.md](todo.md) のみ。指定Skill `agent-skills:planning-and-task-breakdown` と付属Definition of Doneを全文読了した。Skill既定の保存先は今回明示された作業別パスへ置換する。
基点は `d86e7e9d4fe4497d8be9040d30a078582f2887d8` と既存未コミット差分。編集前の `git status --short` を確認済みである。ルート `tasks/plan.md` / `tasks/todo.md` と前作業の未完項目は保持する。

## 要求・完了境界

| ID | 要求 / 観測可能な受入条件 |
| --- | --- |
| QF-01 | WAV/MP3/AAC/AMR/3GP/3GPPを共通lib、本体GUI、単独GUIで選択・検査・確認・送信対象にできる。内容・コンテナ・codecはspecのP-F01互換profileを満たすこと。公式の形式名を全codecの保証にしない |
| QF-02 | 確認時に保持した元音声bytesを、対応するwire形式で送る。再生用復号音声と送信元音声を混同せず、パス差替え・再選択・確認後変更・二重送信・取消しの既存不変条件を維持する |
| QF-03 | 現行6MiB・60秒上限を維持する。破損、空、形式偽装、未対応codec、時間不明、上限超過、復号失敗では送信要求0となり、安全な理由表示を行う。外部backend不足も成功扱いにしない |
| QF-04 | 本体の対応音声は送信前に原音を再生できる。再生だけPCMへ復号する場合も、送信bytesは不変である。再生停止・閉じる・失敗時の資源回収を維持する。単独GUIの既存機能にない再生UIの新設は含めない |
| QF-05 | 本体確認欄の「送信先URL＋要求model/effort」「目的」「結果の受取先」の説明を削除する。取消し説明は指定文「※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しない。」へ変更し、冒頭説明と読む例文の間に一度だけ表示する |
| QF-06 | 共通libのcore-only利用、本体の資格情報分離、単独GUI/MCPの既存結果受取境界を維持する。指定表示変更を単独GUI/MCPへ横展開しない |
| QF-07 | 新しい固定差分で必須テスト/build、独立レビュー、隔離fixtureによるnative画面確認を実施する。実API送信は今回対象外とし、mockや公式資料確認を実接続成功の証拠にしない |

今回の開発完了は、これらの実装・自動検証・native画面証拠・独立レビューと残る実機制約の明示までである。計画担当の完了はCP0計画精緻化、文書整合検査と親へのhandoffまでであり、後続の実施状況は親が本書冒頭・作業リスト・結果へ統合する。

対象外はAPI/provider追加、認証方式変更、資格情報移行、Codex fallback、本体結果のCodex送信、教材/学習記録への保存、録音上限拡張、送信元の自動変換/切詰め、一般media基盤の再設計、依存ツールの自動導入、実API試送信、commit/push/PR/公開である。

## 根拠と再利用

既存 [QWEN-INTEGRATION-001設計](../QWEN-INTEGRATION-001/design.md) のsnapshot・確認ID・取消し・資格情報分離・GUI資源寿命・core-only境界を再利用する。全面再設計は不要である。今回の仕様では本体表示に関する旧要件だけを明示的に更新する。
コード確認で `audio.rs::AudioInput::parse` はPCM16 WAV・1/2ch・8～48kHz限定、`provider.rs` はwire `format=wav` 固定、両GUIはWAV選択である。
親の調査報告では公式Qwen3.8 Input limitsで6形式を確認済み、環境にはFFmpeg/ffprobe、AMR-NB encoder・NB/WB decoder、AAC、MP3が存在する。これは親提供の証拠であり、配布先での存在保証ではない。公式URL・取得日・codecごとの保証範囲は外部仕様が記録する。

## 作業単位・所有・依存

| 単位 / 難易度・risk | 実施内容 / 最大限のproduction所有 | 受入と確認 | 依存 |
| --- | --- | --- | --- |
| U0 仕様・設計差分 / 中・高 | 外部仕様担当 `spec.md`、設計担当 `design.md`。各1文書。親が独立レビュー報告を保存し、計画者が本書/todoを更新 | QF-01～07、codec表、backend契約、失敗/取消し、UI変更位置を凍結。仕様・設計レビューと既承認要求との照合 | 暫定計画を親が確認 |
| U1 共通入力検査とwire / 高・高 | backend helper所有: `tools/qwen-audio/src/{audio.rs,error.rs,lib.rs,provider.rs}` と必要な新規backend/parser module。U1a公開型/エラー/export（3ファイル）、U1b bounded backend/parser（新規最大3ファイル）、U1c provider wire（1ファイル）の検証可能な小単位で順次完了する | QF-AC-001～005/009。各形式/codecの成功・拒否、境界・外部参照拒否、原bytes/wire一致、再生分離、cancel/cleanup/shutdown、core-only | CP0、独立期待値。U1a契約を先に固定 |
| U2 単独側の多形式読込 / 中・高 | standalone helper所有: `tools/qwen-audio/src/{gui.rs,main.rs}`。2ファイル。`session.rs`は変更不要 | QF-AC-001～005/007～009。6拡張子、jobの再選択/取消し/IPC close、旧音声失効、shutdown後警告、MCP境界、再生UI新設なし | CP0契約で並行着手可。統合合格はU1に依存 |
| U3 本体の多形式・指定UI / 高・高 | 親所有: `src/qwen_reading.rs`、`src/app/qwen_reading_ui.rs`、`src/main.rs`。3ファイル。旧U4の説明整理を同一所有へ統合 | QF-AC-001～009。本体snapshot/原bytes・再生/取消し・終了回収、削除3説明、指定注記一度/例文前、通常/狭幅/拡大 | CP0契約で並行着手可。統合合格はU1に依存、U2完了は着手条件でない |
| U5 固定差分の全体受入 / 中・高 | production新規所有なし。runnerは `results.md` / task内logs/artifactsのみ。修正はU1～3の担当へ差戻す | QF-07、QF-AC-001～009、必須gate、独立レビュー、未確認の明示 | U1～3、CP1/CP2 |

テスト作者は `tools/qwen-audio/tests/` の関連integration tests/隔離fixture、`tests/qwen_reading.rs`、`src/app/qwen_reading_tests.rs` と `tests.md` を所有し、単位ごとに最大5ファイル程度へ分ける。親がAstra/highでbackend/単独helperとは別著者として仕様由来の期待値・fixtureを先に作成する。本体U3は親実装との著者独立性がないため独立作成済みとは報告しない。独立reviewerがU3の期待値・回帰妥当性を別contextで検査する。同一Rustファイルのtest節は実装者と同時編集せず一時所有を渡す。test配線は実装者、期待値・fixtureはテスト役とする。native fixtureで `src/app/visual_check.rs` の変更が必要なら親が明示所有追加を記録してから着手する。
依存変更が必要ならroot/単独 `Cargo.toml`、対応lockは親に所有を限定し、設計根拠とcore-only境界を確認後に別の小単位で扱う。既存dirtyを一括整形/上書きしない。利用説明/再利用契約/`tasks/current.md`/KPIは親の統合責任であり計画者は編集しない。

## 役割とモデル割当

利用可能一覧は親提供の `gpt-6-astra`、`gpt-6.1-sol`、`gpt-6-sol`（low/medium/high/xhigh/max/ultra）、`gpt-6-luna`（low/medium/high/xhigh/max）である。計画担当と外部仕様担当は固定で `gpt-6-astra/high`。親モデル/effortは変更しない。以下は起動指定であり、返却実行metadataは未取得である。起動時は親が対応モデル/effort/Skillを再照合し、欠落はblockerとする。

| 単位 | 設計 | 実装 | テスト作成 | テスト実施 | 独立レビュー |
| --- | --- | --- | --- | --- | --- |
| U0 | `gpt-6-astra/high` | `gpt-6-astra/high`（実装なし、実現可能性差戻し担当） | `gpt-6-astra/high` | `gpt-6.1-sol/medium`（文書追跡確認） | `gpt-6-astra/high` |
| U1 | `gpt-6-astra/high` | `gpt-6-astra/high` | `gpt-6-astra/high` | `gpt-6.1-sol/medium` | `gpt-6-astra/high` |
| U2 | `gpt-6-astra/high` | `gpt-6.1-sol/high` | `gpt-6-astra/high` | `gpt-6.1-sol/medium` | `gpt-6-astra/high` |
| U3 | `gpt-6-astra/high` | `gpt-6-astra/high` | `gpt-6-astra/high` | `gpt-6.1-sol/medium` | `gpt-6-astra/high` |
| U5 | `gpt-6-astra/high`（設計差戻しのみ） | `gpt-6-astra/high`（修正は元単位へ） | `gpt-6-astra/high`（不足ケース差戻し） | `gpt-6.1-sol/medium` | `gpt-6-astra/high` |

選定理由: U0/U1/U3は未信頼入力・外部process・資源寿命・非同期UIを横断するため設計/実装/期待値をAstra/highとする。U1a/b/cの5役はU1行を全て適用する。U2実装は確定APIを二つのhostファイルへ適用する限定作業でありgpt-6.1-sol/high、取消し/送信同一性の設計・独立期待値・レビューはAstra/highとする。旧U4はU3同一ファイルの小変更として統合し全5役ともU3行を適用するため実装はAstra/highである。U5は定めた検証の実行をgpt-6.1-sol/medium、証拠と境界の独立判定をAstra/highとする。runnerは期待値を変更せず、モデル名を合格や性能保証の根拠にしない。
7役の責任を維持し7人の起動は要求しない。U0/U5の差戻し役は必要時だけ起動する。本作業では契約確定後のU1 backend/helper（Astra/high）とU2 standalone/helper（gpt-6.1-sol/high）の2補助を認める。所有が別ファイルであり、その間に親はU3 root実装および別著者としてbackend/単独のテスト期待値を進められる具体的な並列利益があるためである。依存manifest/lock編集、同一ファイルのtest節、統合buildは直列調整する。再委任はしない。runnerはgpt-6.1-sol/medium、独立reviewerは作成者と別contextのAstra/highを親が明示dispatchする。親のモデル/effortは変更せず、親への割当は現行Astra/highという親提供条件に基づく。
必要Skillは7役表をそのまま適用する。仕様はspec-driven-developmentの仕様工程、設計はapi-and-interface-design、本体UIはwordweave-egui-ui、実装/runnerはwordweave-change、test作者はtest-driven-development、レビューは対象に対応する外部仕様/設計/code-review-and-qualityである。

## 必須検証・checkpoint

CP0: 仕様・設計の独立確認と採用判断は親報告で完了。本計画の精緻化済み内容を親が確認して実装dispatchする。未解消の指摘を計画者が合格へ変更せず、承認済み仕様を超える選択は差し戻す。
CP1: U1/U2後、全6形式の隔離fixtureを共通lib→単独controller→mock要求まで通す。代表codecを含む形式別の成功/失敗とbackend不足を確認する。該当focused testsとcore-only checkを通す。
CP2: U3後、同fixtureの本体controller経由で原bytes・wire形式を照合し、native画面で指定UIと再生/停止/閉じるを確認する。人の確認後にだけ送信できることをmockで確認する。両mainのshutdown後 `take_audio_cleanup_warning()` と秘密/pathなしの `rfd::MessageDialog` 警告経路も確認する。
CP3: U5で以下を固定差分に対して実行し、新規logsとexit code・件数・未検証を記録する。過去ログは今回合格へ流用しない。

```text
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked
cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked
cargo check --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --test integration_reuse --locked
cargo test --test qwen_reading --locked
cargo test --bin wordweave5 qwen_reading --locked
```

追加検証は変更範囲のformat/check、feature treeによるGUI/MCP不要依存の非混入、backend探索・不足/異常終了/timeout/過大stdout/stderr・protocol/外部参照拒否・空白/日本語path、原音非改変を含む。native画面は本体通常/狭幅・拡大で注記/例文/確認/送信/閉じるへの到達と単独GUI形式説明を確認する。実マイク/音声出力・IME/DPI確認が取れないものは未検証と明記する。実API送信を追加しない。
独立レビューは未信頼audio解析、subprocess引数/入出力/kill回収、取消し、確認snapshot、認証/結果境界、配布依存、UI削除範囲と証拠の整合性を対象とする。修正後は影響する検証だけを再実行する。公開前人の承認という既存gateは維持するが、今回公開は行わない。

## 確定契約・残る実証リスク

| リスク | 影響と対策 / 決定担当 |
| --- | --- |
| container名とcodec保証の混同 | 3GP/3GPP、AMR-NB/WB、AAC containerを仕様で分離する。公式根拠がない制限を「API制限」と偽らない。外部仕様担当 |
| duration metadata偽装/全復号の資源消費 | 60秒の厳密境界、decoder delay/padding、復号出力上限、実行timeout、不明値の拒否を設計する。設計担当 |
| FFmpeg/ffprobe依存と配布先差異 | 探索順・両toolの適合・不足時表示・配布時要件を固定する。環境の既存インストールを全利用者へ一般化しない。自動download/導入はしない |
| 外部参照/URL/playlist・引数注入 | 内容allowlist、明示protocol制約、shell不使用、元pathを外部入力として再openしない設計、上限付きpipe/出力回収、異常process終了を検証する |
| UI threadの復号待ち/遅延完了 | 実行場所・取消し・世代一致・閉じる時のprocess回収を設計に明記する。新旧音声混同を回帰試験する |
| 機密音声の一時保存 | メモリ/pipeを基本候補とし、seek必須containerの扱いを設計で確定する。保存が不可避なら寿命/隔離/削除/失敗時契約を明示して既存メモリ限定要件との整合を決める |
| 本体説明削除の過剰適用 | 本体指定箇所だけを変更する。接続設定や実metadata区別、単独MCP結果受取説明は維持する |

上表の設計事項はspec/designで確定済みである。WAV PCM16、MP3 Layer III、AAC ADTS、AMR-NB/WB、3GPP brandの単一AAC/AMR音声を許可し3GP/3GPPのwireを `3gp` に統一する。backend探索は実行ファイル隣media-tools→絶対PATH entry、WAVはbackend不要である。15秒合計deadline、probe 64KiB、PCM最大11,520,000 bytesに超過検出1frame、厳密な全復号/終端検査を採用する。MP3/AAC/AMRはpipe、3GPだけP-F02の短命private copyを使い、外部参照を拒否する。
公開APIは既存 `parse/read_wav` を維持し `AudioInput::decode(bytes, cancel)`、`read_audio`、`AudioLoadJob`、`playback_wav`、`wire_format`、`shutdown_audio_jobs`、`take_audio_cleanup_warning` を追加する。UIでdecode/joinをせず、cancel/Dropは取消しを立て、registryがworkerを保持し終了時に全回収する。通常/失敗/取消しでtemp削除後publish、削除失敗は専用errorとwarning flag、両mainはshutdown後flag回収→秘密/pathなしrfd native警告とする。異常終了残留は保証対象外として利用説明へ記載する。
未決の製品選択はない。残件は切断/属性変化・3GP外部参照拒否、取消し競合/全process回収、削除失敗警告、native再生とUIの実装証拠である。fixtureで仕様を保証できない場合は合格にせず仕様/設計へ差し戻す。配布先backend、全codecの実API受理、OS音声/IME/DPIは別の未検証事項として報告する。

## 次handoffと測定

親は本plan/todoと承認spec/designを確認し、U1 `ww-implementer` = `gpt-6-astra/high`、U2 `ww-implementer` = `gpt-6.1-sol/high` を各所有範囲・確定API・AC・必要Skill・禁止事項を明示してdispatchする。親はU3と独立backend/単独テスト、manifest/lockを所有する。期待値先行→U1小単位/U2/U3→CP1/CP2→runner→独立reviewerへ進む。所有拡張や仕様不適合は親へ返し、黙って別ファイル/モデルへ進まない。
開始/終了token counters・実行model/effort metadata・親子包含関係は本担当では未取得である。測定値を推定しない。親は `docs/process/improvement/kpi.md` に従い取得できた分だけ測定文書へ記録する。通常execとNode読取はsandbox起動エラーで失敗し、読み取り専用の昇格execで必要資料を確認した。
