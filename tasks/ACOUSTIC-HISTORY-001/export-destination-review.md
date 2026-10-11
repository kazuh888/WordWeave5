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
