# I2d-H 外部書出し先共通境界の独立レビュー

## 設計Pre（2026-10-11）

既定reviewer backup_consumer_pre（指定gpt-6-astra/high）は **PASS、受入阻止0** と判定した。対象は[設計§5追補](design-i2.md)、SHA-256 `50BE63D85B75AF7E7D1F1EABB306698722CAF89AF6E46821FCB34B04E6A06380`。T08/T08b、plannerの最大2file所有（store本体/新integration、専用cfgはstore内）、新依存なしとの対応を確認した。

- 同一Mutexの取得処理だけを共有する外部guardは、正本write-blockを解除せず、poison/再入拒否を維持する。
- GUID/128-bit identity取得不能時の拒否は既定契約内。GUIDを返せないnetwork shareを対応済みに扱わない。
- 予定root自身/配下/未作成祖先を両modeで拒否する。Fileによる予定directory成分の閉塞と、Directory配下媒体生成による侵入を防ぐ。
- tempの事前保護検査、公開atomic_writeの契約を維持するprivate抽出、解決済み絶対pathと同一guardによる後続接合が整合する。H01–H07に独立oracleと失敗時観測がある。

非阻止の検証留意：H06の共有制限fixtureは実失敗を確認して採用する。FILE_READ_ATTRIBUTES要求はshare-mode=0で拒否されるとは限らない。作者はmetadata/identity取得の実失敗を独立確認し、成立しないfixtureを拒否試験の成功に数えない。根拠は[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。GUID取得の制約は[GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)と照合した。

FFI ABI、実FSのidentity、alias権限、H06/H07実行証拠は実装/独立試験gateに残る。d-B/M consumer接続、I2-D2閉鎖、I10は未完了。reviewerはread-onlyで編集/build/test/再委譲なし。実行metadata/usage counter未取得。親が本記録を保存し、独立test作者へ留意点とともに引き継いだ。

## コードレビュー cycle 1（2026-10-11）

対象はbranch `codex/development-agent-team`、HEAD `ed2cd75fa90a314e19bbe1f6416d46a4b4021805` からのstore本体差分、source SHA `66287990637663A5A6B0119B9D686AB5AEAA34BB84CDD2D045B24DEA05AF4BD0`。独立reviewer backup_consumer_pre（指定gpt-6-astra/high）は **CHANGES_REQUIRED** と判定した。確定本体欠陥0、阻止する検証不足1、到達性確認の質問1である。独立runner/Postの代替ではない。

| ID / 原文・位置 | 原因 / 状態 / Packet | 正本・owner / oracle / 残余 |
| --- | --- | --- |
| H-TEMP-1 P2: store.rs:1542「不存在destとtempの同名拒否が未固定」。既存temp試験はleaf_presentで先に拒否され同名判定を検証しない。欠落時は最終destをtempとして開き、公開前の部分書込み・失敗時削除が起こり得る | 追加した安全分岐の独立case不足。Accepted、Packet H-TEMP-1 | design-i2.md §5とstore専用cfg/test。test-author所有。prepare成功後temp=destでpublishしErr、dest不存在・全ledger不変・temp増分0。独立Post未了 |
| Q-H-CWD: store.rs:1987のcwd.join(relative)はverbatim cwdならjunction/../safeをwalk前に縮約する | 元成分観測前の正規化。Decision pending、調査のみ | 設計H2.2。公開current_dir到達未実証、private注入だけで公開不具合と断定しない。隔離子processの実cwdと公開入口を観測、到達時だけ修正Packetを起こす |

標準path結合の根拠は[Rust PathBuf::push](https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.push)。reviewerは同Mutex/write-block例外/poison・再入/FFI配置/GUIDと128-bit identityのfail-closed/予定root双方向保護/再検査/公開atomic互換に追加指摘なしとした。raw markerとalias exactは報告と整合する。symlink2、別drive、実identity拒否、独立runner/Postは未完了である。

### H-TEMP-1限定Pre提出（本節のみ規範）

- 目的・契約：不存在destと固定tempが同名の場合は、I/O前にErrを返しdest・ledger・tempを変更しない既定分岐を固定する。製品仕様・型・公開APIは変更しない。
- scope/owner：独立作者がsrc/store.rsの専用cfg/testへ1件のみ追加する。本体、既存test、integration、manifest、consumerは対象外。入力は安全な不存在destと同名temp、出力は既存ResultのErr、状態遷移はprepare成功→publish拒否→無変更である。
- 境界：合成fixtureだけ、既定guardと既存ledger oracleを使い、実データ・外部通信・OS設定変更なし。旧50件と本体sourceを保持する。
- 検証：新exact1件の到達/Err/不存在/ledger不変/temp増分0、変更範囲・本体不変・format/diffを確認する。成功済み旧46/79をtest-only追加で再実行しない。
- 依存・停止：限定Pre PASS後に着手する。現コードがoracleに失敗した場合、作者は本体を変えず親へ返す。Q-H-CWDの本体修正とは混在しない。
- rollback/artifact：追加testだけを取り消せる。生ログと初回失敗は保持する。利用者に製品判断を求める変更はない。Post PASS前はClosedとしない。

親self-checkは対象source hashと既存証拠の対応を照合済み、文書リンク4件missing0・diff-check成功。独立reviewerは文書SHA `4BF8543BE6DD95A07EA154427F5E5743D3401FE94236CBB805E780F3081306CC` とsource hash一致を確認し、限定Pre PASSを返した。H-TEMP-1はPre-gate PASS、作者へ追加1件を返却済みである。規範内容は判定後に変更していない。

H-TEMP-1限定Postは独立reviewerにより **PASS/Closed**。新case除去後の旧source hash一致、rawのprepare成功/拒否後不変/子exit0/timeout=false、source `F705AE19635F3C95828B36C60D5CBC8E59D9FB43DE3219A14BC02E73CC28022A`・lib/public EXE・integration hash、TEMP0を確認した。matrixのH-TEMP-1をClosedとする。新exact1件のみを追加成功として数え、Q-H-CWDと全体Postは未閉鎖である。

### Q-H-CWDの到達条件観測（非規範）

親はignored `artifacts/export_cwd_probe.rs` のstd-only合成probeをrustcでコンパイルし、PID1884/exit0で確認した。通常の絶対pathでset_current_dirするとcurrent_dirは通常形式であり、join後もjunction/..が残った。一方、合成directoryのfs::canonicalize結果（verbatim形式）をset_current_dirすると、実current_dirもverbatim形式を返し、joinはjunction成分を消した。成功後に元cwdへ戻し、生成した空fixture directoryのみremove_dir、cleanup=trueを確認した。製品コード/テスト期待値は変更していない。

```text
ordinary cwd: retains_junction=true
canonical/verbatim cwd: retains_junction=false
cleanup=true
```

storeの公開prepare経路は実current_dirをprivate prepareへ渡している。ただし公開export自体の結果は本probeで未実行である。OS上このcwdが存在しないという非到達理由では閉鎖できない。H-TEMP-1と分離した公開入口の合成再現・修正単位を後続で確定する。

### 管理者2件の実行待ち

利用者はsymlink2件だけの管理者実行を承認済み。親は固定public EXE SHA `CC53B50FC19DAEBE1EAB5219C7D7EF9F3389EBEF040F99D7290E25AD79548EC1` を指定したRunAsを開始したが、15分以上実行artifact生成なし、起動待機のままだった。session90972のwrapper PID33492（作成21:52:32 JST、コマンドとhash一致を再照合）だけを停止、session exit -1。製品アプリや他のprocessは終了していない。実テストの実行結果はなく、2件は未実証のままである。OS設定変更なし。修正後の確定EXEで再実行を要する。

## Q-H-CWD 限定Packet（Pre提出）

- 目的・正本：設計H2.2の「元の祖先成分を観測してから..を処理する」を、実verbatim cwdと相対入力でも維持する。上記std probeは到達条件のみを実証済み、公開APIの行動REDは作者が先に採取する。
- 対象snapshot：branch codex/development-agent-team、HEAD `62f932403a232cfad41ffbb11acf3b3206f8ab49`、store SHA `F705AE19635F3C95828B36C60D5CBC8E59D9FB43DE3219A14BC02E73CC28022A`、integration SHA `9A14DCB790E29AC3407A0D47EB46CACA077BB08DFDCB52D1599C272B7968936E`。H-TEMP-1限定Postは完了。製品判断・新API・新依存なし。
- scope/owner：作者は既存integrationへ公開回帰6件（root/destの相対junction/..拒否×File/Directoryの4件、正常な相対入力のFile/Directory対照2件）だけ追加する。実装者はstore private絶対化処理だけを修正する。既存test・FFI・gate・wire・consumer・manifestは不変。設計/実装/作者/reviewerは既定Astra/high、runnerは後続Sol/medium。
- oracle：隔離子processで合成fixtureの実junction/identityを確認し、通常cwdとcanonical/verbatim cwdを実set_current_dirで設定する。実current_dirのprefixを記録する。rootまたはdestに相対junction/..を含むと公開APIはErrで全ledger/bytes/temp不変。正常対照2件は同じverbatim cwdで通常相対・rooted-relative・絶対pathそれぞれを使い、正しい外部場所のbytes/新directoryを確認する。相対とrooted-relativeの対照には `/` および混在区切りも含める。一括拒否を修正と認めない。
- 現行原因と最小実装：cwd.joinがverbatim cwdの場合に相対成分を縮約する。検証済み入力の絶対化は、絶対pathなら元表現をそのまま、rooted-relativeならcwdのdrive/UNC prefix＋元表現、通常相対ならcwd＋区切り＋元表現をOsStringで組み立てる。通常相対/rooted-relativeをverbatim基点へ連結する場合は区切り `/` だけを `\` に変換し、`.`/`..`と元成分の順序は保持する。既にverbatimの絶対入力は変換しない。PathBuf::push/join/canonicalizeで未観測成分を消さない。drive-relative等の既存拒否は維持し、得たpathは従来validate→walk→identity検査へ渡す。
- failure model：公開APIの型・エラー分類は既存のまま。安全でない入力はI/O前に拒否する。cwd設定は試験の子processだけ、panic時も元cwdを復元してfixtureを除去する。OS設定変更・実データ・管理者実行は行わない。junction作成制限は同EXE限定の非管理者sandbox外で比較し、未成立をRED/PASSへ数えない。
- 検証・依存：限定Pre後、独立作者のREDを保持してから実装者へ本体を渡す。公開REDが想定と異なる場合は修正を開始せず親へ返す。修正後は新6件と外部境界既存51件を確認し、同一旧本体の広い保存79件は絶対化だけの変更なら反復しない。独立コードPost/runnerを省かない。
- rollback/artifacts：本Packetのprivate helper変更と新6件だけを戻せる。失敗ログと初回行動REDは保持する。未作成root/既存root保護契約、最終再検査後のTOCTOU限界、環境未実証4件は本Packetで完了扱いしない。
- 停止：public API/仕様変更、追加file/依存、別不具合、期待値変更が必要になったら親へ返す。authorは本体修正禁止。scope・方式を変更する場合は再Preとする。

Q-H-CWDはPlanned（Pre未受領）。親self-checkは実current_dirの観測とprepareの接合位置を照合済み、H-TEMP-1と副作用範囲を分離した。新たな本体変更・公開再現testはまだ行っていない。

Pre cycle 1はCHANGES_REQUIRED、P2「単純連結ではverbatim基点で通常入力の `/` が区切りでなくなり拒否される」を受領した。根拠は[Rust Prefix](https://doc.rust-lang.org/std/path/enum.Prefix.html)。cycle 2では上記唯一の規範へ区切りだけの変換と正常2件内の `/`・混在対照を追加した。未観測成分の縮約を禁止する契約と6件/2file境界は維持する。旧案を代替規範として残さない。

Pre cycle 2は **PASS**。独立reviewerは文書SHA `0E432A55F1DAB3F30B7003AFCE70C8E3CBD82CB112AC4F61040A771E1B603B31`・HEAD/source/integration不変を確認した。区切りだけの変換と対照追加で前回P2は充足、新矛盾なし。Q-H-CWDをPre-gate PASSとし、独立作者へ公開6件のRED採取を返却した。規範内容は判定後に変更していない。

作者の公開6件では、拒否4件すべてで通常cwdはErr/ledger不変、同じ合成fixtureの実VerbatimDisk cwdはOk/ledger変化/宛先1件作成となり、行動REDを確認した。正常File/Directory2件は各9表記でPASS。これによりQ-H-CWDは単なる条件付き質問ではなく既定祖先観測契約の不具合として確定した。製品の実データ破壊が発生したという意味ではない。最初のsandbox junction error5はfixture失敗として保持し、同EXE exact4件を非管理者sandbox外で再現した。詳細hash・実行記録は作者報告を正とし、既定Preの方式で実装者へ渡す。

## Q-H-CWD限定Post・コード引継ぎ判定

独立reviewer backup_consumer_pre（指定gpt-6-astra/high）は **Q-H-CWD PASS/Closed、I2d-Hコードrunner-ready、追加finding0** と判定した。対象store SHA `53843CC8A8DBDB63BB58C82DB3BB42C7E943F1AE00E8EF613DE61BD9E45F42FC`、integration SHA `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324`。修正はcommit `b92883411d7c3736466b15f2fa39a8c0f45747e8`、事前契約/独立oracleは `6416f32` へ分離した。

- private絶対化と2呼出し以外を戻した全文が旧source hashと一致し、期待値・FFI・gate等に変更なし。
- 未観測成分の保持、区切り変換、絶対入力の非変換はPre契約に適合する。
- 公開4行動RED→GREEN、実両cwdの拒否/ledger不変と正常2件×9表記成功をrawで独立確認した。
- 57実名の集合差0、環境未実証4件を除く53成功、固定source/EXE hash一致、TEMP0、H-TEMP-1維持を確認した。

closure matrixのH-TEMP-1とQ-H-CWDはともにClosedである。ただしsymlink2・別drive・実identity拒否の4件、独立runner/最終Post、consumer接続・I10・製品EXEは未了。実行担当ではないreviewerはbuild/testを代行していない。次は利用者のSol/medium切替後の独立runnerである。

## I2d-H 最終Post cycle 1（2026-10-12）

独立reviewer `export_destination_final_review`（指定gpt-6-astra/high）は **CHANGES_REQUIRED（必須証拠待ち）**。基準HEAD `5d4ab1072158d3c9a3a7875b5f4b4bf1997146ee`、store/integrationと4EXEのhashを独立runner記録と照合し一致を確認した。新57実名・全136実名・初回raw・失敗26件exactの集合差/重複/不一致0、132行動成功/4未実証、旧79全成功を確認。確認済み本体欠陥0。H-TEMP-1/Q-H-CWDの限定Closedは維持する。

| Finding / 原文・位置 | 原因・状態・owner | 必要証拠・残余 |
| --- | --- | --- |
| H-SYMLINK-EVIDENCE P2：公開試験438/456のfile/directory symlink拒否を未実証 | error1314でfixture前に終了。承認済みRunAsもartifactなし。Accepted、親/独立runner担当、コード修正なし | 固定公開EXE `95E01D39B0ECB7C018C3D78FFE5D3C1B8FF96A395EC3748644F83345954C36AD` の承認済みexact2だけを実行。各1PASS、UNAVAILABLEなし、ledger oracle到達を確認。利用者へ既存scriptの手動実行を案内した |
| H-IDENTITY-EVIDENCE P2：設計§5 H06の実metadata/identity取得不能時の拒否が未検証 | store.rs:2713の独立probeが成功し、recheck/publish/不変assert前にreturnする。Accepted、独立test-author所有 | 合成fixtureで実取得失敗を独立probeにより成立させ、productionのIo/IdentityUnavailable、bytes/temp不変、制約解除後の再試行成功を確認する。期待値緩和/fake identityは禁止。方式の実現性を限定probeで調査してからPreへ提出 |

別drive alias不在1件は設計上「利用可能な」環境に限定されるため受入阻止ではないが、成功に数えない。新drive設定を要求しない。共通Mutex/write-block維持/poison・再入/元成分観測/GUID・128-bit identity/予定root双方向保護/直前再検査/temp/atomic互換に追加findingなし。敵対的TOCTOU・電源断・GUID非対応共有・consumer・I10/製品提供は対象外である。

既定依存順によりd-S/B/M本体は未着手を維持する。読取りによる接合準備は許される。修正対象の証拠だけを独立runner/reviewerへ返し、成功済み136件の一律反復は不要である。reviewerは編集/実行/再委任0、runtime metadata/usage counters未取得。次の規範PacketはH-IDENTITY-EVIDENCEに限定し、実装前Preを維持する。

### H-IDENTITY-EVIDENCE 限定Packet（唯一の規範・Pre提出）

- 目的/正本：設計§5 H06の既存契約「実metadata/identity取得不能を不存在へ変換せず、書出しを拒否する」を実証する。製品の仕様/API/error分類は変更しない。
- 現行証拠：store.rsの該当1caseはshare_mode(0)でも独立FILE_READ_ATTRIBUTES probeが成功し、受入assertに未到達である。Microsoft [CreateFile](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea)では属性要求は共有制限の対象外である。旧fixtureを成立条件の異なる実Windows状態へ置換する。
- 実現性：独立作者のstd-only `artifacts/export-identity-probe.rs` / `export-identity-run.log` はnative PID9172 exit0、TokenElevation=0/effective_admin=0。prepared相当の属性handleを保持し、別DELETE/share7 handleの[FileDispositionInfo](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_disposition_info)をTRUEにすると独立属性openがWin32 5、metadataは成功。close前にFALSEへ解除すると同一path/bytes/file IDへ復帰した。panic unwind時のRAII解除も成功。合成file2個と空owned dirを限定cleanup済み。製品経路の成功証拠には数えない。
- scope/owner：独立test author（既定gpt-6-astra/high）が `src/store.rs::i2_export_destination_tests::h06_real_identity_denial_is_not_absence_when_fixture_is_available` の本体と同関数内のFFI/fixture helperだけを所有する。製品本体、他case、公開integration、Cargo/依存/OS設定/ACL/権限は変更しない。全57caseの名前・数を維持する。test記録は既存export-destination-tests.mdへ追記する。
- 入出力/状態：既存合成destと元bytes/独立identityを記録し、既存prepareを成功させる。DELETE/share7 handleでDeleteFile=TRUE（DELETE_ON_CLOSE/POSIX flag不使用）→独立属性openのPermissionDenied/Win32 5をassert→既存recheckとpublishを呼ぶ→FALSE解除/handle close→ledger/identityを比較→guard解放後の通常writeを再試行する。productionの戻りは既定Io又はIdentityUnavailable、publishはErr、固定tempは生成0である。
- failure model/oracle：通常probeは最初に成功し、拒否中はNotFoundでない実拒否へ到達する。recheck/publish呼出し後、明示解除の成功を確認してから全bytes/ledger/temp不変と同一identityをassertする。最後の通常writeはPAYLOADと一致する。fixture不成立・復元失敗はtest失敗とし、UNAVAILABLEで早期returnして通過させない。RAIIはpanic/unwind時にもclose前のFALSE解除を試みる。解除失敗のログを残し、成功扱いしない。試験のbounded隔離process/timeoutは既存のまま。
- 安全/依存：所有する合成fileだけを操作する。実データ/外部通信/OS設定/管理者実行なし。現在の公開EXE/hashは利用者のsymlink2確認のため固定し、該当libだけをbuildする。公開EXE不変を前後hashで確認する。Pre PASS前は既存testを編集しない。
- 検証：新fixtureの該当exact1件、元case差分を除いたsource不変、既存57名維持、format/diff、source/lib/public EXE hash、rawの独立拒否/production拒否/復元/再試行/cleanupを記録する。通過済みsuiteはtest-only変更で反復しない。自己検証後に独立runner（Sol/medium）と独立Post（Astra/high）へ対象1件だけ返す。
- rollback/stop：本1case差分だけを戻せる。旧未実証ログとprobeは証拠として保持する。新fixtureで製品が成功する、期待値変更が必要になる、別case/本体/依存変更・追加権限が必要になる場合は親へ戻す。H-SYMLINK-EVIDENCE、別drive、consumer、全体Post/製品配布はこのPacketでは閉じない。

Pre snapshotはbranch `codex/development-agent-team`、HEAD `5d4ab1072158d3c9a3a7875b5f4b4bf1997146ee`、store SHA `53843CC8A8DBDB63BB58C82DB3BB42C7E943F1AE00E8EF613DE61BD9E45F42FC`、公開test SHA `176709E152A6D26D6676B3424E673AD5B28987C423A8865C0358F5472D2E0324`。親self-checkは独立probe raw/source、当該case、production observeの属性openを照合済み。現時点ではPlanned/Pre未受領である。

限定Pre cycle 1は **PASS・阻止0**（独立reviewer export_destination_final_review、指定Astra/high）。提出文書SHA `4A516C7F169B8820695CF11B4DF9249E25A5C019354E6D354657F26B8EB55AD8`、source/public test/EXE不変、probe source/raw/hashを照合した。production observeへの接続と戻り値期待は整合する。復元失敗をrawとtest失敗として残せばよく、既存Temp/isolated cleanup変更や合成evidence copy追加は不要と確認。H-IDENTITY-EVIDENCEをPre-gate PASSとし、作者へ指定1caseを返却した。上記規範は判定後に変更していない。製品拒否/ledger/再試行・独立runner/Postは未実証である。

限定コードレビューは **PASS・runner-ready・阻止0**（同独立reviewer）。既存1関数以外のHEAD原文一致を独立比較し、57実名集合差0、FFI ABI/DELETE-share7/FALSE解除後close/panic復元/復元失敗のtest failure、旧UNAVAILABLE早期通過の除去を確認した。作者exactの実Win32 5→recheck Io/publish Err→復元→全ledger/identity/bytes不変/tempなし→retry成功、親子exit0/timeout=false/TEMP0はrawと一致した。store SHA `BB35631D2EF3E42EE8B98B9C6BECC3FEC3310C5CD8D6E96E6F6EF47578745EBC`、lib SHA `68E8DC0504F714EEED8B547228396812089AAE9BFF5310C4BC6BC321454B1314` を現物照合、公開source/EXEは固定hashのままである。現行test規範も更新済み。次はSol/mediumの独立runnerへこのexact1件を返す。H-IDENTITY-EVIDENCEはVerified（作者自己検証）であり、独立runner/最終Post・H全体Closed・symlink2件を代替しない。

### H-IDENTITY-EVIDENCE 限定最終Post（2026-10-12）

独立reviewer `export_destination_final_review`（指定gpt-6-astra/high）は **PASS/Closed・追加指摘0**。HEAD `722f3f5c5cea02f23b22f02c46b22764bd165644`、store/公開test/lib/公開EXEの4hashを現物・前後記録・前回review対象へ照合し一致を確認した。独立runnerのlist対象1件、native PID29828と親子exit0、1PASS/0FAIL/0ignoredを新規rawで確認した。

実Win32 5拒否→recheck Io/publish Err→FALSE解除後close→identity/bytes/全ledger不変/tempなし→retry成功へ到達し、UNAVAILABLE/timeout/復元失敗/TEMP残存は0。旧未実証の同名1件を重複なく差し替えた証拠接合は136unique中133成功/3未実証であり、一括再実行の結果ではない。closure matrixのH-IDENTITY-EVIDENCEをClosedへ更新する。

H-SYMLINK-EVIDENCEの承認済み2件は引き続き必須証拠待ちであり、I2d-H全体Closedとd-S/B/M本体着手は保留する。別drive alias不在は条件付き未検証のまま。reviewerの編集/build/実行/再委任0、runtime metadata/usage counters未取得。親は固定EXEと既存手動scriptの存在を再確認済み、モデル切替を追加せず利用者の実行結果を待つ。
