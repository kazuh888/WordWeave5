# ACOUSTIC-HISTORY-001 早期試用の外部契約

2026-10-14最新承認：[提供判定記録](early-trial-delivery-review.md)冒頭の利用者承認により、本文のWindows DPI150/200%確認だけを利用者試用時へ移す。DPI100%とその他native/全体検証/独立Postは提供前条件を維持し、提供物には「DPI確認未完了」を明記する。過去のhash固定snapshotを遡って合格へ書き換えない。

2026-10-12 / ET0 / 仕様草案。親の承認範囲照合・独立仕様レビュー前である。試用固有の動作は未実装・未検証であり、本書だけで実装着手・EXE提供・通常データ利用を承認しない。

## 外部仕様（spec author所有）

### 目的・根拠・境界

利用者の「一旦WordWeave5を触りたい」に対し、了承された「通常データから分離した試用専用データで現実装を触る」ローカルEXEを提供する契約である。利用者了承の根拠は親からの引継ぎであり、この担当が元会話を直接確認したものではない。[早期試用計画](early-trial-plan.md) ET-01–07を安定した受入IDとしてそのまま用いる。

| 要求根拠 | 内容・承認状態 | 対応受入ID |
| --- | --- | --- |
| ET-R01 | 上記利用者要望と分離試用の了承（親引継ぎ） | ET-01、03、04 |
| ET-R02 | 親が指定した通常データ・資格情報の非共有、直接起動・改名時も分離、外部出口の保護 | ET-01、02、05 |
| ET-R03 | 親が指定した新履歴UI未完成の明示、既存学習・設定・音読の試用、実APIは利用者の明示操作のみ | ET-03、04、06 |
| ET-R04 | 親が指定した10/03版release維持、ローカル提供、既存milestone・検証水準の維持 | ET-07 |

本書は一つの「隔離した早期試用」を扱う。新しい履歴保存・解析・比較画面、汎用profile、実データ複製、通常版への移行、UI刷新、公開は追加しない。[通常実装計画](implementation.md) のI3/C1/I6–8とI9/C2/I10、既承認d-B/Mの契約と順序は変更しない。[本体履歴仕様](spec.md)の将来動作を、この試用で利用可能とは扱わない。

### 画面・操作・入出力

既存の学習画面、設定画面、既存音読と毎送信確認、バックアップ・外部書出しの導線を対象とする。新履歴画面への入口は提供しない。新たな画面構成、ラベルの全面変更、対応音声形式・文字数・容量・処理時間上限の変更は要求しない。各既存形式・制限を維持し、履歴を含む入力についてはET-04を追加適用する。

入力は直接起動するWindows EXE、試用内で作った学習内容・設定・音読入力、既存形式の取込／復元候補、外部保存先である。出力は専用保存先に残る試用データ、既存形式の外部書出し、試用を識別する表示、拒否理由である。初回は通常版の学習状態・設定・Qwen keyを引き継がず、試用側で設定する。通常版のChatGPT認証経路の利用はET-06の例外として区別し、Qwen key共有の根拠にしない。

### 受入契約・例・証拠

| ID／根拠 | 観測可能な約束 | 正常・境界・失敗例と必要証拠 |
| --- | --- | --- |
| ET-01／R01、02 | EXE単独の直接起動、改名、異なる作業directoryからの起動、再起動でも同じ試用専用保存先を使う。教材・進捗・音声・ink・journal・診断の通常保存先を自動読込・複製・変更しない。専用保存先を決定・安全に利用できない場合は明示失敗し、通常保存先へ切り替えない | 正常：試用で保存後、再起動して再読できる。境界：EXEを改名しcwdを変えても同じ試用内容である。失敗：root不明、不正、通常rootへのalias、書込不能で停止する。隔離した合成通常rootのsentinel不変とアクセス観測、失敗表示、fallback 0回を確認する |
| ET-02／R02 | Qwen設定の既存の読取・保存・接続先変更は試用専用Windows資格情報だけを扱う。通常版・単独音読ツールのkeyを読み出さず、自動コピー・移行・再利用しない。送信先変更には明示的key入力を要求し、既存の毎送信の音声・参照確認を保持する。新しいkey削除機能は追加しない | 正常：試用keyを設定でき、既存の資格情報読取経路が試用側の保存値を参照する。keyそのものの再表示は要求しない。境界：通常側だけ設定済みでも試用は未設定である。失敗：接続先変更後に旧宛先keyを暗黙利用しない。production target選択の証拠とcredential spyで他namespaceの読取／書込0回、送信spyで意図しない送信0回を示す。削除経路が存在しない場合は削除操作をN/Aと記録する。実資格情報を比較のために読まない |
| ET-03／R01、03 | アプリのtitle等の常時識別表示に「早期試用・専用データ」を含める。使用説明に解決される専用保存先、新履歴UI未提供、通常版へ自動移行しないことを示す。既存学習・設定保存・既存音読への操作導線が到達可能である | 正常：起動時に試用と分かり、各導線へ進める。境界：Qwen未設定でも試用識別を失わない。未完成履歴を利用可能と示さない。合成native UIでtitle、説明、各操作の到達性を確認する。キーによる既存導線を妨げず、識別を色だけに依存させない。支援技術・IME・実機音声の未確認は別記する |
| ET-04／R01、03 | 空の試用保存先から作成した学習・設定を保存し再起動して使える。許可した操作から新音声履歴を導入しない。履歴付きの既存試用保存先では起動を明示拒否し、履歴付き取込／復元候補は適用を拒否する。拒否対象を上書き・自動修復・履歴削除して続行しない | 正常：合成データで学習→設定保存→再起動。境界：履歴のない既存対応形式を扱う。失敗：履歴付きroot、履歴付き候補を拒否し、既存bytesと現在の試用状態を保持する。全取込／復元経路の検査と新履歴への非到達証拠を要する。未知版・破損を履歴なしと決めて受理しない |
| ET-05／R02 | export・backup・媒体書出しの保存先として通常root・保護配下・それらのaliasを指定しても書き込まない。試用rootの既存保護も維持する。保護対象と無関係な外部保存先への既存書出しは利用可能である。拒否でpayloadや既存宛先bytesを失わず、成功と表示しない | 正常：合成内容を外部一時directoryへ書き出す。境界：保護rootそのもの、配下、同一場所を指すaliasを拒否する。失敗：安全性を確定できない保存先を成功扱いしない。既定d-B/M・settings exportの独立oracleに通常／試用の両保護rootを加え、拒否前後のbytes・payloadと成功表示不在を確認する |
| ET-06／R03 | 起動・設定表示・設定保存・復元から実API送信や実マイク録音を自動実行しない。試用のために共有Codex認証・Codex settingsを自動変更せず、CODEX_HOMEも変更しない。既存ChatGPT認証経路の読取・利用を維持し、Qwen資格情報の分離と混同しない。API-key fallbackやQwenへの代替送信を追加しない。実API送信は利用者の明示操作と既存確認を通る | 正常：無送信で学習・設定・既存音読準備に到達する。境界：接続設定が存在しても起動／復元だけでは送信しない。失敗：Codex利用不可をQwen自動送信で補わない。mockで送信・録音開始0回を確認する。実Codex／Qwen／マイク／TTSの動作成功は別の利用者実機受入であり、この証拠から推定しない |
| ET-07／R04 | 固定した対象差分について計画の通常／試用構成・関連crateの全gateと独立Postを満たしたEXEだけを提供する。10/03版releaseを上書きせず、提供物はrepo内のignored配下に置くローカル成果物とする | 正常：対象SHA・dirty識別、build構成、EXE SHA256、検証ログ、未完了項目を説明と結び付ける。境界：通常構成だけ成功しても試用提供不可。失敗：試用build／gate失敗時に旧EXEを代用品として提供しない。旧release hash不変と提供先ignore・絶対pathを確認する |

### エラー・警告・取消し・再試行

ET-01/04の起動拒否は、GUI直接起動でも利用者に失敗が伝わることを必要とする。保存先利用不可と履歴前提不適合を区別し、日本語で失敗対象・理由を示す。ET-04/05の操作拒否は未適用／未保存であることを示し、完了表示へ進めない。秘密key・音声bytesをエラーへ載せない。表示面・正確な文言は設計案の親照合事項であり、黙示的な終了を明示失敗とは扱わない。

本契約は新たな取消しボタンや自動再試行を要求しない。既存の取込／復元確認、送信確認、取消しを維持する。確認前に閉じる／取り消す操作から適用・送信しない。起動拒否後は原因を解消して利用者が再起動する。操作拒否後は利用者が候補・保存先を選び直す。失敗を契機に通常保存先を選び直したり、データを削除したり、APIを自動送信したりしない。実API送信開始後の取消し挙動や既知の実機音声未確認を、本契約によって修正済みとしない。

警告は使用説明の試用限定・新履歴UI未提供・自動移行なしと、既存操作の警告を対象とする。新履歴用の容量管理・警告閾値・保存期間を早期試用へ持ち込まない。バックアップや媒体書出し全体を利用不可にする代替案は未採用であり、必要なら新UX制約として親へ返す。

### 実装選択との分離・未決事項

親指定の設計方向は、compile-time Cargo featureで専用root・Qwen credential namespace・titleを固定することである。外部約束は上記IDで定義し、feature名、rootの具体path、credential target文字列、関数・検査順・表示部品はdesignerが下節へ記す。通常構成の既定動作は維持し、汎用profile選択や必須launcherを追加しない。

親の限定検索ではCodex login関連はCLI手順説明であり、アプリ内login/logout操作は未発見である。これは操作不存在の全面証明ではない。後続で共有認証を変更する手動操作が見つかった場合は影響を親へ返し、無断で機能制限や汎用認証分離を追加しない。

| ID | 未決事項／親への返却 | 解消条件 |
| --- | --- | --- |
| ET-Q01（解消） | 「通常データと分離」は自動共有・自動移行なし、適用先が試用専用rootであることを意味する。利用者が明示的に選んだ通常root内を含む既存対応ファイルの取込／復元は、既存確認とET-04の履歴候補拒否を維持して利用可能である | 2026-10-12、親が既存了承との適合を確認した。ET-05の通常rootへの出力拒否は継続する。利用者の手動取込とエージェントの試験権限は別であり、親・子の試験は合成データのみとする |
| ET-Q02 | 履歴付きroot・候補の識別範囲、全取込／復元経路の非到達性、通常／試用rootとaliasの安全判定は設計未確認である | designerが証拠と限界を示す。既存機能の追加閉鎖が必要なら親へ戻し、非到達と推定して先へ進まない |
| ET-Q03 | 固定root・namespaceの具体名と直接起動時のエラー表示面・文言は未確定である。アクセス不能時の画面経路も未検証である | designerが具体案を提示し親が承認範囲を照合する。使用説明と受入期待値を提供前に固定する |

### 受入の実施範囲・引継ぎ

この担当の完了境界は本節の仕様草案、要求追跡、参照path・文書差分検査、親への未決事項返却までである。skillのSpecify工程だけを用い、後続工程を実施しない。実装・試験の場所／完全な実行コマンド／役割別所有は[早期試用計画](early-trial-plan.md)の「作業リスト・所有・検証」「検証gateと提供時点」を参照する。本節はコード例・内部構造・新しいコード規約を追加しない。

受入は合成fixtureとmockで行い、実データ・実資格情報・実APIを用いない。各ET-IDに実在test名・実行件数・ログ・UI証拠を後続担当が紐付ける。未実行・0件・mockのみをnative実機／production接続の成功と扱わない。文書検査成功は動作の受入成功ではない。指定モデル／effortは `gpt-6-astra/high`、実行metadataおよび開始／終了usage counterは未取得である。

## 設計（designer所有・独立review前の草案）

2026-10-12。入力は親が承認範囲を照合した上記外部仕様（Q01解消）と早期試用計画、指定は`gpt-6-astra/high`である。仕様／設計reviewと計画確定前の実装を許可しない。api-and-interface-designとwordweave-egui-uiを読了した。以下は早期試用固有の接合であり、I2b-S/CやI3以降の通常保存・履歴機能の完成を主張しない。

### 固定構成と所有

Cargo featureは既定offの`early-trial = []`、試用rootは`%LOCALAPPDATA%\WordWeave5-EarlyTrial`、通常保護rootは同じ基点の`WordWeave5`、Qwen targetは`WordWeave5.EarlyTrial.QwenReading.Connection.v1`、titleは`WordWeave 5 — 早期試用・専用データ`に固定する。exe名・cwd・CLI引数による選択はない。通常featureなしのroot、target、title、公開署名を維持する。LOCALAPPDATA/CODEX_HOMEを設定し直さず、Codex認証・settings・PATH探索を変更しない。

| 単位／受入 | 本体・作者所有と具体接合 |
| --- | --- |
| ET1／01・04・05・06 | `Cargo.toml`、`src/store.rs`、`src/ai.rs`、`src/commit.rs`、新`tests/early_trial_paths.rs`の5file。各本体の専用cfg(test)は作者→実装者の逐次所有。commit追加は回復前の読取専用履歴検査だけである |
| ET2／02・03・04・06 | `src/main.rs`、`src/app.rs`、`src/qwen_reading.rs`、`src/app/qwen_settings.rs`、`src/app/qwen_settings_tests.rs`の5file。app追加は試用起動factory、private試用bool、JSON復元の適用前拒否とそのcfg(test)だけである |
| ET3／07 | 計画の別target-dir、ignored delivery、全gate、固定source／EXE hashを使用する。旧releaseに書き込まない。source所有の追加なし |

### root検査・回復と起動

`store::resolve_storage_root`は固定namespaceを選ぶ副作用なしresolverを維持する。試用の`Storage::open`は、private `open_early_trial(root: PathBuf, normal: PathBuf) -> Result<Storage, TrialOpenFailure>`へ接合し、型付き失敗を後述の安全な表示文へ変換して既存の`Result<Storage, String>`を返す。`Storage::at`は既存の任意root用契約を保つ。Storageにfeature内のprivate boolを1つ置き、atはfalse、検査成功したopenだけtrueとする。Windows部品`external_destination_windows::{observe, walk, protected_root, check_region, recheck}`は実在するが、既存directoryを許すroot分離用の公開読取APIはない。既存Directory exportは既存leafを拒否するため流用せず、同じprivate部品から読取専用preflightを作る。絶対path／namespace／祖先・rootのreparse／identity不明を拒否し、通常rootと試用rootの同一・包含・aliasを検査する。既存試用treeはmetadataだけを列挙し、reparseやfileの複数linkを追わず拒否する。通常treeの内容は列挙・読取しない。これにより試用root内のcustom.tsvやassets等を経由する通常データ参照も拒否する。root不在では確認済み親以下だけを作成対象とし、通常rootは作成しない。検査結果をboolへ捨てず、観測handle／identityを保持して作成・open直前に再確認する。最終OS操作との敵対的競争の完全防止は約束しない。

preflightは試用progressを既存`read_progress_typed`／strict codecで検査し、`audio_history: Some`、未知版・破損・読取不能を拒否する。root不在は内容なしであり、通常rootからの補充はない。`commit.rs`にfeature内crate-private `check_early_trial_recovery(root: &Path) -> Result<(), String>`を追加し、既存`pending_checked`、上限付き`read_recovery_bytes`、private Intent、`decode_progress_for_material`を使ってpendingの旧新Progress両方を読取検査する。Someや不正をNoneと扱わない。非履歴pendingを一律拒否せず、成功後の既存recoverを維持する。旧新decodeの小helperを既存recoverと共用してもよいが、publisher・移行・archive・fault順序は変更しない。preflight失敗はmkdir、app.lock、diagnostics、回復書込より前に返す。成功後に試用root／app.lockを開き、loadでは試用boolの場合だけ同じ履歴検査を既存guard下・recover前に再確認する。履歴のないpendingだけが既存回復へ進む。保存前提のhistoryは現在のProgress参照とpending旧新参照で識別し、未参照audio-historyファイルの存在だけで削除・回復・接続しない。

起動失敗の型と表示責任を分ける。store内private `TrialOpenFailure { stage: TrialOpenStage, reason: TrialOpenReason }`のstageは`BeforeWrites | StorageInitializationStarted`、reasonは`RootUnavailable | UnsafeRoot | HistoryPresent | InvalidProgressOrPending | StorageIo`に固定する。resolver／読取専用preflightの拒否はBeforeWrites、最初のmkdir／app.lock／probe操作の直前にStorageInitializationStartedへ移り、以後は戻さない。root決定失敗もBeforeWrites/RootUnavailableへ明示変換する。理由は発生箇所で付け、既存エラーStringをparseしない。storeがこの型から安全文を生成し、Storage::openの既存String境界へ渡す。BeforeWritesの表示だけ「起動前の検査で停止しました。元データは変更していません。」を付す。StorageInitializationStartedでは「試用保存先の初期化を完了できませんでした。試用内に作成済みのフォルダーや確認用ファイルが残っている場合があります。」とし、不変を断言しない。

ET2のmainはfeature時にStorage::openを先に実行し、成功したStorageをcrate内の`app::WordApp::new_early_trial(cc, storage) -> Result<Self, TrialStartupFailure>`へ移す。app.rsが所有するcrate内enumは`TrialStartupFailure::{StorageOpenRejected { safe_message: String }, RecoveryOrInitializationStarted}`である。mainはStorage::open失敗を前者へ包み、store生成の安全文を再解釈せず表示する。factoryは既存`new_with_storage`を呼ぶ直前から後者の段階を所有し、load／回復／その後の初期化のいずれの失敗も後者として返す。後者の本文は「試用データの回復または初期化を完了できませんでした。試用内では回復がすでに適用されている場合や、途中で停止している場合があります。」に固定し、raw errorのparseや未変更の断言をしない。成功時だけprivate試用boolを有効にしてdiagnosticsを試用rootへ初期化する。通常のnewは維持する。mainはこのenumの安全な表示口だけを消費し、いずれの失敗もWordAppの操作画面へ進めず、既存rfdのMessageDialogでtitle「早期試用を開始できない」を表示する。全失敗に「通常版の保存先へは切り替えません。表示された原因を確認し、データを削除せずに再起動してください。」を添える。書込前の原因には「試用専用の保存先を安全に利用できません。」又は「音声履歴を含むため、早期試用では開けません。」等のreason対応文を使う。秘密・本文を含むraw errorを表示せず、root不明／不正／書込不能を安全な理由で区別する。rfdは既存の終了時警告でも使用済みであり、新egui画面を増やさない。日本語折返し、長い安全なpath、Windows DPI 100/150/200%、狭い表示領域で本文とOK操作をnative確認する。ボタンの両軸中央・focus・keyboard操作はOS controlを使って確認し、egui独自描画や固定offsetを入れない。既存操作拒否通知とbuttonsは既存UiControlsを維持する。

書込可能性は属性から推定せず、履歴preflight成功後の安全な試用root内で、privateな一意probeをcreate_new→短い固定bytesのwrite/sync→close→removeまで実行して確認する。既存fileを上書きせず、失敗時は起動を止め、cleanup失敗時のprobe残存をエラー証拠に分ける。履歴拒否前にprobeを作らず、通常rootへprobeを置かない。これは起動時点の確認であり、以後のdisk-full等は既存保存エラー契約で扱う。

### 外部出口の2root保護

store内にfeature時だけ`OnceLock<TrialRoots>`（固定のtrial／normal PathBuf対）を置く。検査成功したStorage::openが画面開始前に一度登録し、別の対への変更はErrとする。公開設定・切替・解除口はない。登録前のgeneric core APIは従来のcaller root契約、実mainは登録前に外部操作へ到達しない。これが必要なのは既存d-Sの`s05_actual_storage_precedes_fallback_environment`が、Storage::atの合成rootとLOCALAPPDATA欠落でも外部保存成功を要求するためである。featureだけで全外部helperに環境取得を追加して既存oracleを変更しない。試験の登録は有限timeout子processで行い、実通常rootを登録しない。

`prepare_external_destination`はcaller rootに登録済み2rootを加え、private Windows prepareの共通処理で各rootを検査する。CheckedStateにその固定root集合と全observationsを保持し、最終recheckは全rootを再確認する。tempのprepareも同じ保存済み集合を使い、destだけを検査してtempが通常rootへ入る抜け道を作らない。guardは従来どおり1個である。新公開引数やconsumer別policyは不要で、d-Sの3出口、d-Bのdirectory／全子file／manifest、d-Mの音声・annotation原本／一式が既存under_guardへ接合済みであることを前提にする。`ai::Config::from_settings`のcwdだけを同じresolverの`codex-work`へ変更する。run_historyは既にStorage.dir/codex-work、diagnosticsもStorage.dir由来である。起動時に固定したroot対をresolverが再利用し、途中の環境再読でjournalだけ別rootへ分岐させない。

### 入力・資格情報と非到達証拠

現物のProgress取込はsettings_uiのJSON decode→pending_restore→`WordApp::restore`と、backup_uiの`backup::prepare`→Prepared::restoreである。JSON経路はappのprivate試用boolがtrueの時だけ、restore入口でcandidate.audio_historyを検査し、verify_references・memory退避・現progressコピー・状態更新より前にErrを返す。未知版／破損は既存strict decode/validateが拒否する。候補の表示・確認は既存どおりで、適用拒否は「音声履歴を含むため、早期試用には復元しない。現在の記録は変更していない。」とする。媒体backupはprepareが既にhistoryを拒否し、decode済みProgressはUIで編集しない。TSV教材・語彙CSV・承認済み教材生成はEntry/語彙を扱いhistory入力を持たず、chat添付／dropはbytesをAssetStoreへ保存する経路である。手動選択元が通常rootでもQ01どおり読取を許し、適用先は試用Storageのみとする。

app／app下のproduction参照にHistoryWriter／commit_historyの呼出は見つからず、既存音読はqwen_reading::ReadingControllerで、永続履歴writerを呼ばない。初期head=None、JSON拒否、既存backup拒否、その他入力にhead項目なし、通常save/material commitがdisk headを維持することを合わせて、許可操作からSomeを導入しないことを確認する。単なる検索だけを証明とせず、各入力経路の合成試験でdisk/UIのNoneを観測する。外部processが実行中に試用内容を書き換えることや、未参照objectの復旧UIは受入済みとしない。history codec／HistoryWriter全体をfeatureで無効化せず、Storage::atと既存core history試験の期待値を維持する。非到達を破る新経路が見つかればその経路を親へ返し、I2b-S/C全体を先取り実装しない。

Qwenは既存公開定数WORDWEAVE_QWEN_CREDENTIAL_TARGETの値だけをfeatureで選び、単独toolの定数・crateは変更しない。qwen_settingsのprivate `configured_store_with<F, T>(factory: F) -> T where F: FnOnce(&'static str) -> T`が選択定数を渡し、production default_storeはそこでWindowsCredentialStore::for_targetを呼ぶ。cfg(test)のdefault_storeは既存mockを維持する。作者は同じselectorへnamespace別spy factoryを渡してeditor／ReadingControllerのload/save先を観測し、別に実WindowsCredentialStore::for_targetを構築して既存target()だけを読む（資格情報のload/saveは呼ばない）。これでproduction target選択をmock置換と区別する。新宛先の明示key・毎送信確認・取消し・generationは既存実装のままであり、constructor／設定保存から送信しない。現CredentialStoreはload/saveのみで削除機能はなく、新削除UIは追加しない。Codexは既存app-serverのChatGPT認証確認を利用し、login関連は案内文である。認証書換えを新設しない。

### 独立作者・実装への引継ぎと限界

ET1作者はstoreのprivate helperと公開openを、子processの合成LOCALAPPDATAで試験する。root不在／既存、改名・異cwd相当のresolver固定、再open、通常root/子alias/複数link/identity不明、履歴current／pending旧新、非履歴pending回復、通常・試用両rootへのfile/directory/temp拒否、外部成功、全ledgerを固定する。既存prepare/recheckの決定的fixture seamを再利用し、fake identityや検証成功boolは注入しない。ET2作者はmain/appのcfg(test)で起動失敗時のapp開始0・diagnostics0、JSON拒否前後のdraft/epoch/bytes不変、非履歴JSON／媒体backup／教材／語彙／添付のhead=None、識別title、namespace spy、送信／録音開始0を固定する。既存`--test progress_export_destination`、`--test audio_history_backup`、`--lib backup::`、d-S/Mとqwen_settingsの回帰を再利用し、新test名は作者の実在一覧で確定する。native拒否dialogと直接起動・再起動はmockから推定せずET3で確認する。

ET-D01の追加oracleは3状態を区別する。(1) 書込前preflight拒否ではsource全bytes／dir集合不変とBeforeWritesの表示、(2) 合成の非履歴pendingを既存recoverの実書込後checkpointで失敗させた場合は変更済み試用ledgerとRecoveryOrInitializationStartedの表示、(3) 非履歴pendingの回復成功後にapp側の初期化を失敗させた場合は回復済み試用ledgerと同表示、である。後2件では未変更を述べる文がないこと、全件で操作画面開始0・通常root不変・fallback0・diagnostics開始0を確認する。commit作者は既存fault seamで書込後Errを検証し、app作者は同一factoryのprivate loader／初期化接合から返す失敗を使って表示段階を検証する。production recoveryのpublisher／順序を複製・変更せず、Stringの内容によらず段階が決まる対照を置く。

早期試用だけに増える変更は、固定feature/root/target/title、起動root/treeとpendingの読取検査、private boolと固定2root登録、外部宛先の追加保護、JSON適用前拒否、起動拒否dialog、別build出力である。通常保存のroot採用修正や新履歴controllerを追加する設計ではない。安全なroot検査にはprivate部品の抽出と複数root観測が必要であり、文字列置換だけの規模ではない。ET1/ET2とも5fileで収まるが、共有fileは逐次所有する。追加所有・新UX制限・全体gateの不成立が必要になれば親／plannerへ返す。独立reviewでQ02の識別範囲と上記非到達、Q03の具体namespace／拒否表示を確認する。文書検査は実機受入ではなく、実行metadataと開始／終了usage counterは未取得である。

是正packet ET-D01（P2、cycle 2）は本設計節だけを正本とする。原因は書込前拒否と回復書込後失敗を一つの不変表示にしたことであり、現物`commit.rs::recover_with_io_under_guard`のwrite→recheck／checkpoint→Errを根拠とする。findingはAccepted→Plannedで、上記の型・表示・3状態oracleへ対応済み、独立再判定は未了である。rollbackは本設計是正案の撤回のみ、今回の実データ副作用は0である。公開保存形式・既存回復手順・通常版・source/testは対象外とし、isolation境界又は5file所有の変更が必要なら停止する。修正版snapshotの独立Pre PASS前にsourceへ進まない。
