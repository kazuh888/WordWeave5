# ACOUSTIC-HISTORY-001 早期試用EXE限定計画

2026-10-14最新：ET1と限定fixture是正ET1-FX1は[独立最終Post PASS/Closed](early-trial-paths-review.md)。通常250/試用281行動PASS、各fixtureSKIP3を保持した。未変更範囲の証拠接合を含み全件再実行ではない。ET2/ET3・製品EXEは未完成である。以下の過去時点の未了/Pre待ち表記は工程履歴である。Pre承認snapshotのhashは最終Postへ記録し、今回の計画変更は完了状態だけであり契約/検証条件を変更しない。

2026-10-13再開時点：ET0仕様/設計PASSとd-S/B/Mの最終Post Closedを[独立依存確認](early-trial-dependencies.md)で照合し、ET1開始条件を満たした。以下の過去の「d-B/M未了」は計画作成時の入力記録であり、本最新状態に置き換わる。ET1独立作者Astra/highを起動済み。本体・ET2/ET3提供はまだ未完成である。工程別自動引継ぎの承認により、親モデル切替待ちだけを理由に停止せず、計画指定値を各dispatchで明示する。

2026-10-12 / ET0仕様・設計反映済みの確定計画。入力は[外部契約・設計](early-trial-contract.md) SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`。親からcycle 2の仕様/設計ともPASS・ET-D01 Closedを受領した。[独立review記録](early-trial-review.md)を判定正本とし、本文の初回草案表示と後続判定を区別する。source・動作は未実装であり、d-B/M Closed前にET1へ進まない。既承認d-B/Mの着手条件とEXE提供gateは維持する。

## 完了境界と根拠

利用者が了承した「現実装を試用専用データで触る」ためのローカルEXEを用意する。対象は既存の学習・設定・既存音読であり、新しい音声履歴画面は提供しない。[implementation.md](implementation.md) のI3/C1/I6–8後の通常データ用milestoneと、I9/C2/I10の全体完成は変更しない。早期試用は別の限定受入である。

基準HEADは `0ec2fc2294f224c20e4bf0079c645e35596f1e28`。親の引継ぎと[最終Post](settings-export-review.md)により、I2d-Sは34unique成功・PASS/Closedである。I2d-B/M、I2b-S/Cは未完了。着手時dirtyは親所有の `docs/process/improvement/measurements.md` だけであり保持する。途中の親編集も戻さない。

現物の根拠は `src/store.rs::resolve_storage_root` と `src/ai.rs::Config::from_settings` の通常root固定、`src/app/qwen_settings.rs::default_store` の通常Windows資格情報target固定、`src/main.rs` の通常titleである。LOCALAPPDATAだけ変えるlauncherでは資格情報が共有され、EXE単独起動も通常版へ戻り得る。**専用Cargo featureによる試用EXEを採用する**。既定offの `early-trial = []`、root `%LOCALAPPDATA%\WordWeave5-EarlyTrial`、Qwen target `WordWeave5.EarlyTrial.QwenReading.Connection.v1`、title `WordWeave 5 — 早期試用・専用データ`を固定する。汎用profile機構や必須launcherを作らず、直接double clickでも同じ隔離状態になる。通常版のdefault feature・保存先・資格情報targetは維持する。

plannerは本書だけを所有し、チェックリストも本書に置く。`tasks/plan.md`、`tasks/todo.md`、既存task文書の未完了事項は保持する。planning-and-task-breakdown skill全文とDefinition of Done、AGENTS、開発7役、KPIを参照済みである。計画の完了は本書・参照・差分検査と親への引継ぎまでであり、実装・試験実行・再委任・公開を行わない。

## 要求と受入条件

| ID | 観測可能な受入条件 |
| --- | --- |
| ET-01 | 試用EXEの直接起動・異なる作業directoryからの起動・再起動が専用rootを使い、通常rootの教材/進捗/音声/ink/journal/診断を自動読込・複製・変更しない。root不正・書込不能時は明示失敗し、通常rootへfallbackしない |
| ET-02 | 試用Qwen接続の既存読取/保存/接続先変更が専用targetだけを使う。通常targetおよび単独音読ツールtargetへの資格情報操作は0回。既存keyの移行/再利用をしない。送信先変更時の明示key入力と毎送信確認を保持する。現行削除操作はN/Aであり、新しい削除機能を追加しない |
| ET-03 | title等の常時識別表示に「早期試用・専用データ」を含める。配布説明に保存先、新履歴UI未提供、通常版へ自動移行しないことを記す。既存の学習、設定保存、既存音読の導線が到達可能である |
| ET-04 | 空から作った合成試用データで学習/設定/保存/再起動が成立する。新履歴のwriter/controllerは接続せず、試用で許可する操作からhistory rootを導入しない。history付き既存試用root等の前提逸脱は上書きせず明示拒否する |
| ET-05 | export/backup/media保存先に通常rootやその保護配下・aliasを指定しても書き込まない。試用root自体の保護も維持する。通常外部宛先への合成データexportを回帰確認し、拒否時のpayload/既存bytesと成功表示を検査する |
| ET-06 | 起動・設定表示・保存・復元・試験から実API送信、実マイク録音、Codex認証/settingsの変更を自動実行しない。Codexは既存ChatGPT認証経路を保持する。利用者が行う実Codex/実Qwen/マイク/TTS等の受入は別記する |
| ET-07 | 必須全体検証、試用構成検証、独立Postが固定差分に対して合格し、対象SHA/dirty識別・EXE SHA256・build構成・残事項を記録する。10/03版release EXEを上書きしない。提供物はrepoのignored配下のlocalのみである |

対象外は新音声履歴UI・比較/解析統合の完成、汎用profileframework、新ライブラリ導入、実データ複製、秘密の読み出し、通常版データ移行、Praatの新規同梱、GitHubへのEXE公開・push/mergeである。本体試用のために単独ツールの機能完成を要求しない。

Q01は親の承認範囲照合で解消した。通常root内を含む既存対応ファイルを利用者が明示選択する手動取込/復元は維持し、適用先は試用root、history付き候補は拒否する。上記の実データ複製除外はエージェントによる試験・自動共有/移行の禁止であり、この既存手動操作の禁止ではない。親・子の検証は合成データだけである。

## 未完了I2の到達性と依存判断

以下はET0設計と独立reviewで確認した依存判断である。非到達の設計根拠と実装後の合成試験を分け、コードが部分実装という理由だけで全I2以降を前提にしない。実装で反証が出たら当該経路だけを前提へ戻す。

| 残単位/危険 | 試用での到達性と扱い |
| --- | --- |
| I2d-B backup外部宛先 | backup UIは既存機能として到達する。計画開始時のassets限定拒否では通常root全体への誤指定を防げず、試用root分離だけでは不足する。**既定d-Bを先に完了**し、ET-05の複数保護root検査へ接続する |
| I2d-M 媒体外部宛先 | 既存chat媒体/annotation exportが到達する。試用で作った媒体も通常rootを上書きし得る。**既定d-Mを先に完了**する。任意exportの安全性を利用者への注意書きだけに委ねない |
| I2b-S/C stale history root | 未採用の返却Progressによるroot不整合は、diskとUIで履歴rootが相違することが主要条件である。空試用root・新履歴入口なし・history付きimport/restore拒否を設計と独立ケースで証明できれば、**全完了を早期試用の前提にしない**。非履歴draft/設定/媒体保持の既存回帰は必須。証明不能な入口があれば限定閉鎖または該当既定単位の完了を要求する |
| I3 backup v2 | 試用にhistoryを導入しなければ既定v1 backupで足りる。history付き候補が拒否されることを確認する。既存の通常データ用I3依存を削除しない |
| I5残障害検証/I6–8 | 新解析/history jobを接続しない。既存音読から未完成経路へ到達しないことをdesignerが確認する。関連crate検証は維持し、旧cancel原因や実機音声の未確認を解消済みとしない |

依存は `ET0仕様/設計 → 計画確定` と `d-S Closed → 既定d-B → 既定d-M` の2系列であり、**ET1開始前に両系列を揃え、ET1 → ET2 → ET3へ進む**。ET0は契約文書だけ、d-B/Mは既承認のsource/test範囲であり相互の書込み所有は交差しない。したがって親はET0確定を待たず、既承認H4に基づくd-Bの独立作者→実装をdispatchできる。その間にET0文書整備を進めてもよいが、通常補助1役と各役の指定モデル・独立reviewは維持する。d-B/Mは既定計画・設計・テスト・モデルを再利用し、新しい同内容の計画や役を重複起動しない。共有 `store.rs`、`chat_media.rs` 等は逐次所有する。試用固有のpolicyをd-B/Mへ先取り追加しない。

最小代替は、試用buildだけでbackup exportとchat音声/annotationの3exportを利用不可にすることである。候補はbackup UIとchat_mediaの2本体fileであるが、表示だけのdisableでは別呼出しを防げないため、実際の操作入口での拒否と無副作用試験も必要になる。coreのexport全体をfeatureで拒否すると既存backup試験の期待値まで変わるため、その方式を機械的に採用しない。これは利用可能な機能を減らす新UX制約であり、採用する場合はET0に具体的な不可操作と表示をまとめ、親が利用者にその制限を確認してから計画を更新する。現在は各1本体file・承認設計再利用のd-B/Mを推奨する。無効化案より所要時間が短いという実測はなく、選定理由は既存機能の維持と既定契約の再利用である。designerがd-B/Mの想定外拡大を見つけた場合は、この2経路だけの制限案へ戻して判断する。

## 作業リスト・所有・検証

各単位の実装/テスト作者は別担当であり、同じfileのcfg(test)は作者→実装者へ逐次移譲する。ET1/ET2は下記各5fileに所有を確定した。追加所有は親からplannerへ返す。

### ET0: 早期試用契約の確定（S、文書1本）

- [x] external-spec authorが `early-trial-contract.md` の外部契約節を作成し、親が既存利用者承認との適合とQ01解消を確認した
- [x] designerが同文書の設計節を作成した。初回spec PASS/design CHANGES_REQUIREDのET-D01を修正し、cycle 2で仕様/設計ともPASS・ET-D01 Closedを親から受領した
- [x] plannerが本書の所有・操作・依存を確定した。新test名は予定であり、作者が作成後の実在一覧と件数を固定する。架空名を実行済みとしない

依存はI2d-S Closed。検証は要求追跡、参照path、未決項目、各roleのmodel/effort対応、文書diffである。新規source/testは触らない。仕様・設計を同じ短い文書の別所有節へ逐次記録し、既存仕様全体を再作成しない。

ET0は同一文書の両節が揃った時点で、独立reviewerを1回起動して仕様reviewと設計reviewをまとめてよい。両review skillを全文読了し、仕様の観測可能性/受入と設計の適合性/失敗境界を別々に判定し、片方のPASSで他方を省略しない。designerは仕様に疑問・矛盾・未承認のUX制約を見つけた場合そこで止め、spec author/親へ返す。設計作業中は仕様未reviewという状態を保持し、両判定と必要な利用者確認が完了するまでsource/testへ進まない。レビュー水準と独立性を維持し、文書搬送と役起動の重複だけを減らす。

### ET前提: 既定I2d-B/M（各S）

- [x] d-Bは既定 `src/backup.rs` 本体、`tests/audio_history_backup.rs` と本体cfg(test)を所有し、既定T08/T08b受入・独立runner/Postを完了する
- [x] d-Mは既定 `src/app/chat_media.rs` 本体とcfg(test)を所有し、既定3出口・payload保持・marker順の受入と独立runner/Postを完了する
- [x] 両単位の既定focused検証を [implementation.md](implementation.md) に従って実行する。対象・件数を固定し0件成功を受入としない

依存はd-S Closed→d-B→d-Mであり、ET0は着手前提ではない。ET0とd-B/Mの両方をET1開始前に完了する。現行計画の外部guard・独立oracle・レビュー条件はそのままである。

2026-10-12のd-Bは本体/コードreviewと新17＋旧34の自己確認51件まで完了した。通常権限のsandbox外実行でAdministrator=Falseを確認した。独立runner/Postは未了であり、d-B Closedとは扱わない。

### ET1: 試用保存先と外部宛先保護（M、5file）

- [x] `Cargo.toml` の既定offの試用featureと `src/store.rs` のroot/外部出口policyを接合し、試用rootと通常rootの両方を保護する。root不明・不正・alias・書込不能時の通常版fallbackは0回とする
- [x] `src/ai.rs` のjournal/codex-work rootを同じ保存先選択へ接合する。`src/commit.rs` は回復前のpending旧新Progressの読取専用history検査だけを追加し、既存回復のpublisher/順序を変えない。LOCALAPPDATA/CODEX_HOME、Codex認証・model・effortは変更しない
- [x] 独立作者の `tests/early_trial_paths.rs` と必要な本体cfg(test)で通常/試用構成、再起動、境界拒否、合成通常root sentinel不変を検証し、runner/Postを通す

本体所有は `Cargo.toml`、`src/store.rs`、`src/ai.rs`、`src/commit.rs`。独立test所有は新 `tests/early_trial_paths.rs` と上記の専用cfg(test)であり計5file。依存はET0とd-B/M Closed。storeのprivate `TrialRoots` は検査成功時の固定2rootを一度登録し、外部dest/tempと最終再検査へ同じ観測集合を渡す。登録前generic APIと `Storage::at` の既存契約は維持し、実mainは登録前に外部操作へ進まない。root/tree/pendingのpreflight、非履歴pendingの既存回復、登録/失敗stageの詳細は契約設計を正本とする。検証はET-01/04/05/06のfocused回帰と既定store/export/backup/media regressionであり、環境変更・登録は有限timeoutの子processへ隔離する。新featureに依存追加は不要でありCargo.lockを無目的に更新しない。

### ET2: 直接起動・資格情報・識別表示（M、5file）

- [ ] `src/main.rs` に識別表示と起動拒否dialog、`src/app.rs` に試用factory/private試用bool/JSON復元適用前拒否だけを接合する。直接起動・改名・異cwdでも試用構成を維持し、失敗時に操作画面や通常データへ進まない
- [ ] `src/qwen_reading.rs` のtarget選択と `src/app/qwen_settings.rs` のproduction store生成を専用namespaceへ接合する。通常target・単独ツールtargetの読取/書込・自動copyは0回とする。削除N/Aを維持する
- [ ] 独立作者が `src/app/qwen_settings_tests.rs` と上記専用cfg(test)でcredential spy、起動/表示、履歴拒否、非履歴入力のhead=None、送信/録音開始0回を確認し、runner/Postを通す

所有は上記main/app/qwen_reading/qwen_settings/qwen_settings_testsの正確な5fileであり、依存はET1。検証はET-02/03/04/06、設定/既存音読の回帰、および合成native UI起動である。`default_store` のcfg(test)は既存mockを維持する。契約のprivate selectorへspy factoryを渡す検査と、実WindowsCredentialStoreの構築/target()だけの検査でproduction target選択を確認し、実資格情報のload/saveは行わない。

ET-D01の必須oracleは書込前拒否・非履歴pendingの回復書込後失敗・回復成功後の初期化失敗の3状態である。storeの `BeforeWrites` と `StorageInitializationStarted`、appの `RecoveryOrInitializationStarted` を型と発生箇所で区別し、raw error文言をparseしない。後2状態は試用ledgerの変更可能性を明示し、未変更を断言しない。全状態で操作画面/diagnostics開始0、通常root不変、fallback0を確認する。失敗通知は既存rfdを使い、契約のnative表示/keyboard/DPI検査をET3へ引き継ぐ。設計指摘Closedは実装・動作のClosedではない。

### ET3: 固定差分の検証とローカル提供（S、source編集なし）

- [ ] runnerがsource/構成を固定し、下記全体gateと試用構成の検証、合成データで直接起動→学習/設定保存→再起動、既存音読導線を確認する。実API/実マイク等は未検証として分ける
- [ ] reviewerがET-01–07の追跡・source/EXE hash・ログ・通常data/credentialに触れない証拠・残事項を独立Postで確認する。不足は元担当へ戻し、関連検証を再実行する
- [ ] 親がignored配下に試用EXEと短い使用説明を保存し、通常版ではないこと・保存先・できること/できないことを利用者へ提示する。旧release hashを維持し、通常版や学習dataの削除・置換・自動移行をしない

runner所有は `target/early-trial-build/` と `target/early-trial-delivery/` 配下のbuild/隔離fixture/ログ、親所有は同deliveryの説明とtaskの短い結果記録である。reviewerはread-only。build前に `git check-ignore` と解決済み絶対pathを確認する。配布候補から旧EXEへのfallbackは禁止する。

## モデル割当と理由

planner/external-spec authorは全単位 **`gpt-6-astra` / `high`**。以下の `A/H` は正確に `gpt-6-astra` / `high`、`S/M` は正確に `gpt-6.1-sol` / `medium` を指す。現在の呼出可能一覧で両組合せを確認した。親はdispatchで正確な値を明示し、指定と返却metadataを別記録する。親モデルは変更しない。役/skill/model不在はblockerであり黙示継承・代替をしない。

| 単位 | 難易度/危険・選定理由 | designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- | --- | --- |
| ET0 | 中/高。通常データ・認証・未完成機能の境界判断。実装等は未起動、後続への選定だけ明示 | A/H | A/H（工程非該当） | A/H（受入設計、コード非該当） | S/M（コード実施非該当・起動なし。文書検査は親） | A/H |
| 既定d-B | 中/高。外部guardとbackup整合。既定割当再利用 | A/H | A/H | A/H | S/M | A/H |
| 既定d-M | 中/高。媒体・ink保持と外部保存。既定割当再利用 | A/H | A/H | A/H | S/M | A/H |
| ET1 | 高/高。保存先選択と複数保護rootのWindows alias、通常データ損失を防ぐ | A/H | A/H | A/H | S/M | A/H |
| ET2 | 高/高。資格情報production経路・直接起動・前提拒否が交差する | A/H | A/H | A/H | S/M | A/H |
| ET3 | 中/高。artifact/構成の取り違えを検出。runnerは固定手順実行、reviewerは独立証拠判断 | A/H（既存設計再利用） | A/H（不具合は元単位へ、通常非該当） | A/H（不足caseは元単位へ） | S/M | A/H |

これは実測性能による優劣の主張ではない。runnerの固定手順にはSol/medium、保存/認証の判断・実装・独立oracle/reviewにはAstra/highを割り当てる。通常補助1役ずつ、子の再委任なし。7役を同時常駐させない。親が保持するモデル切替通知の運用も本書では変更しない。

ET0の文書検査は親が担い、そのためだけのrunner起動・親モデル切替は不要である。7役の責任識別を保持した工程非該当の扱いであり、独立reviewを省略しない。本体の固定ソースを実行する独立runnerは上表のSol/mediumと既定の親切替境界を維持する。

使用skillは役別に既定どおり、仕様はspec-driven-development、設計はapi-and-interface-design、実装/runnerはwordweave-change、独立作者はtest-driven-development、reviewerは仕様/設計/コードの対応skillと保存・認証に必要な専門reviewである。タイトルだけの変更なら全面UI設計は不要だが、起動エラー導線等の画面追加が必要なら該当UI skillを利用する。PowerShell launcherは必須にしない。後で.ps1を追加する案はその所有/skill/検証を確定してから扱う。

## 検証gateと提供時点

通常構成の必須コマンドは変更しない。旧 `target/release/wordweave5.exe` を保持するため、runnerがそのprocessだけの `CARGO_TARGET_DIR` を `target/early-trial-build/default` に指定して実行する。

```text
cargo test --all-targets --locked
cargo build --release --locked --bin wordweave5
cargo fmt --all -- --check
git diff --check
```

確定feature `early-trial` を使い、別の `CARGO_TARGET_DIR=target/early-trial-build/trial` で次を実行する。通常構成だけの成功では試用EXEを提供しない。

```text
cargo test --all-targets --locked --features early-trial
cargo build --release --locked --bin wordweave5 --features early-trial
```

関連crateの既定gateも保持する。

```text
cargo test --manifest-path tools/acoustic-analysis/Cargo.toml --all-targets --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --all-targets --locked
cargo test --manifest-path tools/qwen-audio/Cargo.toml --no-default-features --lib --locked
cargo build --manifest-path tools/qwen-audio/Cargo.toml --release --locked --bin qwen-audio
cargo fmt --manifest-path tools/acoustic-analysis/Cargo.toml --all -- --check
cargo fmt --manifest-path tools/qwen-audio/Cargo.toml --all -- --check
```

焦点試験名は作者が実在一覧で確定する。既存の権限依存symlink試験は明示ignored理由・別実行証拠を維持し、未実行を成功にしない。必要な昇格は既承認の固定EXE/限定caseだけであり、管理者cargoやOS設定変更を許さない。試用保護rootの新しい権限依存caseが必要なら既存許可を勝手に拡張せず親へ返す。成功済み無変更の限定証拠を理由なく反復しない一方、今回の全体gateを旧ログで代替しない。

**提供時点はET0確定とd-B/M完了の合流→ET1/2→固定差分の全体test/release/関連gate→ET3最終Post PASSの直後**である。I2b-S/CやI3/I8の全完成を待たない条件は上の非到達証拠であり、時間短縮のための品質gate免除ではない。日数・時刻は未見積りである。実機/API受入は利用者がEXEを触る段階へ残す。

## 未決事項と次のhandoff

1. Q01手動取込、固定構成、失敗段階、private 2root保護、history非到達の設計判断は確定した。実装・合成試験・native実行の証拠は未取得であり、作者が実在test名/件数を固定する。
2. d-B独立runner/Postとd-Mは未完了である。両方Closed前にET1本体/新testへ着手しない。b-S/Cの非履歴回帰や入力経路の反証が出た場合は当該範囲を親へ返す。
3. 実Codex/実Qwen・マイク等の実機受入は引き続き利用者側である。最終OS操作との敵対的競争や外部processによる実行中の試用内容変更を、設計reviewの成功から保証しない。

次は親がd-Bの自己確認整理後、既定独立runner/Postへ渡し、続いて既定d-Mを完了する。ET0文書検査専用runnerは起動しない。その後のET1独立作者は **ww-test-author / gpt-6-astra / high**、入力は本書・上記契約hash・cycle 2 PASS/Closedとd-B/M Closedの証拠、所有はET1の新test/専用cfg(test)だけである。期待値と必要な未接続seamを先に固定し、API不在のcompile失敗を行動REDに数えない。新履歴機能、実データ、再委任、公開は対象外である。親のモデル切替運用・独立runner Sol/medium・全体gateを変更しない。

KPIはこのplanner子の開始/終了usage counterおよび実行model/effort metadataが未取得である。指定はAstra/high、出力は本書、検証は文書のみ、再委任0である。測定記録は所有外のため編集せず、親へ欠測として引き継ぐ。

## ET1追補: feature別namespaceの既存fixture補修（2026-10-14、Pre提出版）

本追補だけが今回のplanner所有である。既存ET0/ET1/ET2/ET3の未完了事項、モデル、正本契約、品質gateは維持する。依頼当初のS06だけの案から、runnerの追加4FAILと独立reviewerの同系原因判定を受けて3test-fileへ精緻化した。本体欠陥は未検出である。以下のET1-FX1は独立Pre待ちであり、test編集の開始許可ではない。先行本文のET1「5file」は元実装単位の所有である。今回の追加所有は独立test補修の別Change Setとして下記3file/5symbolに限る技術判断であり、ET1本体所有を広げない。親がこの限定所有を受領しPre PASS後にdispatchする。

### Closure matrix

最終状態（2026-10-14）：以下5行の全findingはFX1独立Post PASSを受け**Closed**である。表内のPlanned/Pre/Post未了はPre承認時のsnapshotを残した履歴であり、現在状態ではない。canonical contract/oracleは変更せず、[最終Post](early-trial-paths-review.md)をReviewer recordの正とする。ET1-P01もClosed、ET2/ET3の残余は維持する。

finding全文欄は親から受領した報告文である。追加4件の独立reviewer判定は同系fixture不整合、本体欠陥未検出である。既存Post予備確認の「確定欠陥0」はrunner前の観測であり、今回のFAILを閉じる判定ではない。

| Finding ID / 原文・位置 | Root cause / state | Packet / canonical artifact / evidence | Oracle | Reviewer record / residual |
| --- | --- | --- | --- | --- |
| ET1-FX-S06: 「S06が常にlocal/WordWeave5固定。StorageNone書出しはresolve_storage_root callerを使用。fixtureはStorage::atだけでTRIAL_ROOTS未登録。P2受入阻止fixture不整合」settings_ui.rs:442/453 | feature別caller rootと固定通常rootの不一致。Planned | ET1-FX1 / 本追補 / s06、write_settings_export、harness_tests::fixture。通常25PASS、試用24PASS/1FAIL | S06両構成成功、外部成功と拒否後不変を保持 | 独立reviewer Astra/highの原因分類を親から受領。Pre/Post未了 |
| ET1-FX-M05A: 「m05_none_resolves_default_without_creating_root_then_protects_it FAIL、assert:1331」chat_media.rs:1736 | normal拒否対象とtrial caller不一致。Planned | ET1-FX1 / 本追補 / 同名test→refuse | 両構成、3媒体の外部成功/拒否/ledger・memory保持 | 独立reviewer Astra/highの同系判定を親から受領。Pre/Post未了 |
| ET1-FX-M05B: 「m05_missing_environment_and_unknown_root_identity_refuse FAIL、assert:1331」chat_media.rs:1763 | normal名fileがtrial rootの不正fixtureにならない。Planned | ET1-FX1 / 本追補 / 同名test→refuse | 名前付き環境欠落/file分岐で3媒体拒否・不変 | 同上。file分岐での失敗位置はsource制御flowからの推論であり旧log markerの観測ではない |
| ET1-FX-M05C: 「m05_failed_default_storage_open_keeps_the_attempted_default_root_protected FAIL、chat_media.rs:1801」 | normalをlockしてtrialをopenし実失敗前提が成立しない。Planned | ET1-FX1 / 本追補 / 同名test、Storage::at/open | 実同rootのopen失敗、Storage=None、3媒体拒否・不変 | 同上。failed-open前提を弱めない |
| ET1-FX-H03: 「h03_resolver_is_side_effect_free_and_tracks_environment_only_in_child FAIL、tests/progress_export_destination.rs:580 left WordWeave5-EarlyTrial/right WordWeave5」 | 2root期待値と欠落Err全文が通常構成固定。Planned | ET1-FX1 / 本追補 / 同名test、resolve_storage_root | 2環境のliteral一致・副作用0、構成別Err全文一致 | 同上。登録後の固定root契約は別受入 |

### Packet Brief: ET1-FX1

1. **Objective**: 承認済みの通常/試用namespace契約と既存fixtureの期待rootを一致させ、上記5ケースを同じ強さのoracleで両構成へ適用する。製品動作変更は0である。
2. **In-scope / Out-of-scope**: 追加所有は `src/app/settings_ui.rs` のcfg(test)内S06、`src/app/chat_media.rs` のcfg(test)内上記M05の3ケース、`tests/progress_export_destination.rs` の上記H03だけである。独立test作者1名が同一責務で順に編集する。production、他case、shared helper、skip/ignore、assert削除、事前登録/OnceLock reset、環境隔離方式、ET2/ET3、Cargo/lock、正本契約の変更は対象外。通常rootを単なる任意外部宛先として扱う既存成功caseのliteralは一括置換しない。
3. **Current evidence**: HEAD `30fc43c074e21cf5b989722dce4fb48c08cddbcf`、branch `codex/development-agent-team`。S06の通常PASS/試用FAILは `target/early-trial-build/et1-runner/{default,trial}-settings-runtime.{stdout,stderr}.log`。追加4件は同runnerのmedia/dest記録と親報告であり、現在同じsourceの通常同名全PASS・試用新35全PASSを受領した。settings fixtureは `harness_tests.rs:199` のStorage::atであり、登録成功を前提にしない。全91対象の基準は同runner `source-before.log` である。
4. **Observable contract**: 唯一の製品正本は [early-trial-contract.md](early-trial-contract.md) SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、root検査と外部出口2root保護の節である。登録前generic APIはcaller rootを保護し、Storage::atは任意root契約を保持する。Storage=Noneのcallerはresolverの選択rootである。通常literal `WordWeave5`、feature時literal `WordWeave5-EarlyTrial` をtest側で選び、その期待pathとresolver返値の一致をassertする。resolver返値から期待pathを作らない。登録済み2root保護の新35ケースをこのfixtureへ複製しない。削除する旧契約はなく、修正するのは無条件通常namespaceというtest前提だけである。
5. **Canonical owner and consumers**: test作者が上記3file/5symbolのみを所有する。plannerは本追補、runnerは既存runner報告とignoredログ、reviewerはread-only、親は統合/dispatchを所有する。対象のproduction consumerはsettingsの `write_settings_export`（教材TSV、Progress JSON、診断export）、mediaの音声・annotation text・annotation bundleの3出口、H03の `resolve_storage_root` である。全て読取確認対象であり本体は変更しない。他担当のdirtyを保持する。
6. **Input / output / error types**: 入力はcompile-time feature、合成LOCALAPPDATA、`PathBuf`、媒体fixture/payloadである。resolverは `Result<PathBuf, String>`、settings書出しとmedia exportは既存の `Result<(), String>` 境界を使う。H03の環境欠落Errは通常の全文 `LOCALAPPDATAが見つかりません。` を維持し、試用の全文は `試用専用の保存先を決定できません。\n起動前の検査で停止しました。元データは変更していません。`（`\n`は改行1文字）をtest側literalに固定する。これは既存RootUnavailable/BeforeWrites安全文である。`is_err`だけの検査へ弱めず、新型/API/seamを導入しない。
7. **State transition / failure-after-state**: S06/M05AはStorageをdrop→合成環境設定→不存在の期待rootに対する外部export成功→root未作成→期待rootを合成作成→内部leaf拒否→ledger/未保存memory不変・app.lock未作成。S06は既存leafと新leafの両方を保持する。M05Bは環境欠落拒否と期待rootがfileのidentity拒否を両方保持する。M05Cは期待rootをStorage::atで保持→Storage::openの実Errを必須assert→Storage=None→3媒体拒否と既存ledger/memory不変、held drop。H03は2つの環境変更と欠落の各段階でsnapshot不変を維持する。失敗はtest FAILとして残し、途中成功を完了に数えない。
8. **Security / privacy / data boundary**: 既存の有限timeout子process、合成fixture、通常ユーザー実行、終了wait/残存記録を維持する。実データ・資格情報・実API・OS権限・モデル/設定変更は0。製品既定値/UX/新API/保存形式/互換性/risk acceptanceの判断を追加しない。今回の差分はtestのみなので配布/運用変更は非該当であり、ET3配布gateは残る。
9. **Dependencies / stop**: 本追補の親受領→独立Pre PASS/hash一致→作者→独立runner→独立Postの順である。追加4件の独立原因分類は受領済み、runnerの現snapshot実行終了と所有返却は作者dispatch前に親が確認する。別原因、本体/契約修正、6件目のcase、shared helper変更、追加file/権限、baselineに帰属不明なFAIL、source/hash競合が出た時点で停止し親へ戻す。同一原因で2回連続CHANGES REQUIREDなら3回目の局所追記をせず正本/owner/oracleを再構成する。
10. **Deterministic oracle**: 下記検証境界を必須とする。旧snapshotの通常PASS/試用5FAILを変更前証拠とし、修正後は同じ5実名が両構成で成功すること、変更diffが期待literalのfeature分岐、resolver一致assert、M05Bの名前付きfixture前提assert/messageだけであることを独立照合する。M05Bはmissing分岐のresolver Err、file分岐のresolver==expected rootかつexpected root.is_fileをそれぞれ名前付きassertで動的識別する。共通refuse helperは変えず、両分岐の3媒体拒否を維持する。行動assert、子process成否assert、失敗後不変、件数を削減しない。
11. **Rollback / remaining artifacts**: 未コミットの作者所有5symbolの差分だけを、親の差分照合後に個別patchで戻す。reset/checkoutで他者差分を戻さない。rollback時は旧FAILが再び有効となりET1 Postは未完了になる。合成fixtureとbuild/log残存はrunnerが既存規則で確認し、診断証拠はignored配下に保持する。利用者dataや既存EXEの削除を行わない。
12. **Traceability**: 上記5finding→ET1-FX1→ET-01/05/06と既存d-S/d-M/H03回帰。本PacketのPost PASSはET1全体・ET2/ET3・音声履歴全体のClosedを意味しない。matrixは独立Post記録後にのみClosedへ更新する。

### 作業単位・モデル・検証

完了記録：FX1-0/1/2/3はすべて完了。Pre承認snapshotの下記チェックリストは履歴であり、最終Postと本書先頭の完了状態を現在の正とする。

ET1-FX1はS（test 3file、production 0file）、難易度中/危険中である。差分は小さいが、正常なcaller契約を誤変更しないことと独立oracle維持が判断点である。planner/external-spec authorは `gpt-6-astra/high`。外部仕様と内部設計は承認済み正本を再利用し、追加仕様作成・設計変更・本体実装は非該当である。

| designer | implementer | test author | test runner | reviewer |
| --- | --- | --- | --- | --- |
| `gpt-6-astra/high`（既存設計再利用、追加起動なし） | `gpt-6-astra/high`（本体変更なし、起動なし） | `gpt-6-astra/high`（既存受入とfeature期待値の独立整合判断） | `gpt-6.1-sol/medium`（固定EXE/実名/件数/hashの独立照合） | `gpt-6-astra/high`（caller/登録済み契約・oracle非弱化のPre/Post判定） |

両model/effort組合せは現在の呼出可能一覧に存在する。runtime返却値は別途未取得として記録する。親モデル/quality gateを変えず、モデル/Skill不在で停止する。作者、runner、reviewerは別担当、再委譲なしである。

- [ ] FX1-0（依存: 独立原因分類受領済み、runner旧snapshot完了/所有返却確認済み）: 親が3test-file所有追加を受領した後、独立Preが同じplan/source hashでPASSする。製品仕様矛盾があればSTOP・ASKとし、期待値を先取り変更しない。
- [ ] FX1-1（依存: FX1-0、所有: 上記3file/5symbol）: 作者がroot/Errのfeature別literal、resolver一致assert、M05Bの名前付き前提assertを設定し、既存assert全保持をdiffで提示する。新3file hash、91対象中それ以外88件不変、既存store/commit/ai作者oracle不変、変更前FAILと変更後focused結果を固定する。
- [ ] FX1-2（依存: FX1-1、所有: 既存runner報告/ログのみ）: 独立runnerが通常/試用を別々に `cargo test --locked --bin wordweave5 --test progress_export_destination --no-run --message-format=json`（試用だけ `--features early-trial`）で構築し、各確定JSONから2EXEのpath/SHAを固定する。両構成でbinの `i2_export_settings` 25件、`chat_media::` 23件、既定fatal-close 5filter、`ai::early_trial_paths_tests` 1件、integration全29件を既存隔離方式で実行する。対象EXE `--list` とselected/成功実名を照合し、5対象caseを含むこと・0件成功でないこと・件数/exit/timeout/wait/Administrator=False/残存を記録する。合計は各構成raw83件であり、fixture SKIPを行動PASSへ混ぜない。
- [ ] FX1-3（依存: FX1-2、所有: reviewer read-only）: 独立PostがPre hash、3file diff、全91source継続、作者oracle差分、両構成結果、既存無変更範囲の証拠接合を確認する。PASS後に親がmatrixとET1 Post資料へ接合する。本追補の変更後には `git diff --check` とローカル参照確認を行う。

sourceの影響があるbin/settings/mediaと変更integrationを新EXEで再検証する。未変更のlib/他integration、新試用lib/integrationのPASSは全91対象のhashと構成/依存が不変なら再実行不要である。ai1も同一新binへ固定し、旧binとの混在を避ける。これは限定補修の検証境界であり、ET3のall-targets、release、関連crate、native gateを削減しない。権限依存fixtureのSKIPは従来どおり未実証として保持し、昇格許可を拡大しない。

### Review request / 次handoff

Request status: READY FOR PRE-REVIEW（親dispatch待ち、Pre未判定）。Cycle: ET1-FX1 Pre-1、未提出provisional-1を置換。Target branch/HEADは上記。Target artifactは本書の追補、対象sourceは上記3file/5symbol。Review classはfixture契約/規約/信頼性、Requested GateはPre-implementation。Dependencyは既存ET0/d-S/B/M Closed、追加4件分類受領済み、runner旧snapshot完了/所有返却確認済み（親報告10:41:45 JST、91source hash不変、全process終了/EXE残存0/TEMP0）。Product decisionはなし。Out-of-scopeはPacket Brief 2/8のとおりである。target planのSHA256は最終保存後の親handoffに添付し、後続編集で旧hash/requestを失効させる。次は同じ独立reviewer `gpt-6-astra/high` へPreを渡す。PASSとsource所有返却後だけ独立作者 `gpt-6-astra/high` を起動する。

開始source hash: settings_ui `C7C7C9D17DD24FC3FDF5762F7CD0E649A1A07E3E7237D31FE6C5A1FB63D1BF9B`、chat_media `24171BE283A1D856A2752FA9CEE60D3AE4D43EE4386EFAC6C1B1C5C09C47D5F3`、progress_export_destination `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324`。初回Pre提出時にこの3hash/HEADと本書hashを再照合する。

plannerはplanning-and-task-breakdown、execute-review-remediationとprotocol、review-project-conventions、review-reliability-data-integrityを自身の計画/self-check範囲で適用した。独立Gateを代替しない。KPI子threadは `01a13810-6d05-7140-8f62-bff0aa4ae790`、親session環境値は `01a08550-055c-7aa1-9834-905e4a7ce148`。指定Astra/high、runtime metadata未取得。最初の観測usageは2026-10-14T01:39:35.437Z、input 38027 / cached 23040 / output 309 / reasoning 19であり、作業開始より後のsnapshotである。週枠開始17%/10080分/reset1792496607。最終応答分、開始前区間、親子包含関係は不明。終了snapshotは親handoffで報告し、所有外の測定文書は編集しない。
