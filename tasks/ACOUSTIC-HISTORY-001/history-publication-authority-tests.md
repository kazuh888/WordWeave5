# I2a-V 独立試験

2026-10-08 / ACOUSTIC-HISTORY-001。担当 `ww-test-author / gpt-6-astra / high`。実行model/effort・usage counterは未取得である。AGENTS/development-team/KPIとtest-driven-development skillの作者/RED責務を適用する。実装修正・再委任・実データ/API・公開・UAC/ACL/OS変更は対象外である。

## 入力・所有・現在の境界

親からT最終Post Closed（独立146 PASS）、V独立最終Pre PASS、未接続scaffold2口の所有返却を受領した。入力は [V限定設計](history-publication-authority-design.md) SHA `A4F5199EA1BB7D7A4D4ADECC4FC76BC9BBFC6A3C05E4E3716E742D4A8A347215`、[正本](design-i2.md) SHA `70749C50224F8051949487ABBFE436AEFE642637B98CA9B488BFE6484E6E92AF` §4/7、[計画](implementation.md) V/V-P追補、[scaffold記録](history-publication-authority-implementation.md) である。

新19件を `src/audio_history/store.rs::i2_publication_authority_tests` だけへ追加した。recovery本体・既存integration2本・既存oracleは変更せず、新integration追加も不要であった。固定Err(Io)のassociated verify/confirm 2口だけで接合でき、追加可視性・試験API・fault seamは不要である。既存confirmをstubへ接続しない。

以下の静的固定記録はcompile前の提出時点である。その後の独立静的PASS・compile/RED結果は末尾へ追記し、初回ログを保持する。親snapshot `artifacts/history-publication-authority-before/` は上書きしていない。

## 独立fixtureとcase構成

既存 `tests/audio_history_recovery.rs` の手製Plan/create/advice DTO構造を新節内へ限定再利用した。無関係Trial/receiptを含むbase、固定mutation文字列、原音/result・Trial/root metadata、予定sequence/receipt/順序付きrefsを作者が組み立てる。production `mutation::recovery_plan` から期待rootを採取しない。production旧計画を呼ぶ箇所は、先に作った期待refsとの比較と「completeと実missingが異なる」ことの観測対照だけである。

disk Progressは合成raw v2を直接配置し、root保持writerの出力をhead oracleにしない。既存公開confirmを先行対照として使い、新口の固定Errと壊れたfixtureを区別する。全ledgerはhost/隣接合成file・媒体・history/recovery/work/backup/temp・directoryを含み、app.lockは既存tree規約の長さ表現で保持する。junctionは既存owned Tempの通常権限fixtureを使う。

全caseのprefixは `audio_history::store::i2_publication_authority_tests::` である。

| 群 / 件数 | case suffix・独立期待値 |
| --- | --- |
| V01 / 1 | `v01_exact_publication_preserves_unrelated_history_and_every_byte`：base Some/None、無関係Trial/receipt/sequenceと順序付き全refs、現在head/全ledger不変 |
| V02 / 1 | `v02_self_consistent_forged_roots_do_not_grant_authority`：Trial＋receipt喪失、receiptだけ喪失、sequence変更。closure健全を先に確認し、tokenだけ差替えConflictとintentも整合した計画RelationshipMismatchを分離 |
| V03 / 3 | `v03_public_token_fields_are_bound_to_disk_and_strict_intent`、`v03_strict_intent_and_ordered_planned_refs_cannot_be_substituted`、`v03_exact_whitespace_request_is_accepted_but_semantic_copy_is_not`：operation/request/base/next、同hash別bytes、schema/未知/欠落/filename-operation、raw duplicate、exact空白、順序/欠落/差替え、512KiB超過 |
| V04 / 3 | `v04_generated_metadata_missing_is_rejected_even_when_plan_is_complete`、`v04_external_audio_and_result_missing_are_not_recreated`、`v04_real_read_denial_and_object_corruption_preserve_all_bytes`：生成Trial/root欠損でも既存plan.complete成功・missing列非空、原音/result欠損はcomplete自体失敗、len/同len hash破損、実intent/原音read32＋unlock後正常対照 |
| V05 / 2 | `v05_pending_head_and_base_failures_are_readonly_and_do_not_block`、`v05_strict_intent_paths_and_asset_junctions_are_not_followed`：教材pending/guard優先、leaf dir、壊れたdisk codec、別head、base局所破損、intent欠損、intent dir/asset junction、全ledger不変・新blockなし |
| V06/V07 / 2 | `v06_v07_committed_next_or_later_root_survives_cleanup_without_new_authority`、`v06_reused_operation_receipt_cannot_be_overwritten_by_a_new_valid_mutation`：next/後続×intentあり/cleanup済みのconfirm成功、verify拒否、既存finishの特定intentだけcleanupとretry、最新base内同operation receipt再利用拒否 |
| V07 / 2 | `v07_disk_confirm_distinguishes_base_busy_from_other_head_conflict`、`v07_confirm_checks_receipt_content_remaining_intent_and_latest_closure`：Busy/Conflict、receipt request/affected/revision、残存intent不正/未知、next破損・current局所欠損、公開口とのtyped結果同値 |
| V08 / 2 | `v08_custom_head_remains_public_only_and_new_confirm_reads_disk`、`v08_retained_reader_quota_and_exact_root_document_cap_are_preserved`：従来custom headの後続root成功と新口disk base Busyを区別、disk nextへ変更後もcustom契約保持、quota1超の保存済closure、root8MiBちょうど/＋1、既存reader 100,000 nodes/u64::MAX設定 |
| V09 / 3 | `v09_held_guard_verification_and_both_confirms_do_not_reacquire_gate`、`v09_write_block_rejects_verify_but_leaves_both_confirms_pure`、`v09_poison_prevents_guard_acquisition_without_preventing_readonly_confirmation`：同EXE exact1case/marker/15秒kill-reap子process。held guard再入拒否の対照、block後verify Ioと両confirm純読取、poison取得拒否とconfirm純読取。resetなし |

V04のmetadata欠損はexpected missing refを1個へ固定し、completeが返すrefsも作者の手製列へ照合する。欠損を自動補修した結果で成功しない。V03の重複key・unknown schemaは正常全文への単独変更とし、必須field欠落との交絡を避ける。型error優先は既存guardなど設計に規定された順序だけを要求する。

## 観測限界・後続必須確認

- Vの最後のhead/intent再読中のdisk置換は既定seamで決定注入できず未測定である。raceや新seamを追加しない。
- pending存在検査はmetadataであり、share0によるread32をmetadata Ioの証明へ流用しない。実metadata Io未成立の枝は未測定として残し、必要基準を削除しない。
- poison後はguard取得自体が拒否される。存在しないpoisoned guardを偽造してverify直接到達を主張しない。held guardのblock後verify拒否とは別caseである。
- 新しい100,000実object closureは生成しない。既存reader設定値の確認、root文書の実8MiB境界、既存I1回帰、抽出差分のnode/各document上限レビューを区別する。既存recoveryの100,000/100,001候補走査はnode上限そのものの証明ではない。
- 新19件のloop後段、全ledger、unlock後対照、cleanup retry、child marker/timeout/残存fixtureは実行時に個別到達を確認する。最初のfixedErrで停止した後段をPASSと数えない。

## 静的固定

[固定hash/件数](artifacts/history-publication-authority-tests-fixed.log)、[fmt](artifacts/history-publication-authority-tests-fmt.log) に記録した。

- store全文: `19492438206F79FF2E0C3B556E5F66C16B322C846C7DD06D220771550FD2E195`
- 新V節: `2610E3AF318BC791F8051ED60AE1137D97033CC3060D5CAD7314EC0F6DA7B19B`
- 新節前prefix: `8D73115CE0EE0CBCA86F933F75B69C47E4F0ED6E24F6D3D9960752A20716D16C`（受領scaffold全文と一致。区切り末尾空行だけTrimEnd＋LF）
- recovery不変: `7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012`
- recovery integration不変: `AA97C2F348365C9699ADC80C72F60B31082D5D30D59A081EC667F6C76DCD9567`
- store integration不変: `7CD4E824DCE62B612ACC9BD90605EDDEB62CBAC6210F85F70B10221D3B3250D4`

`rustfmt --edition 2021 --check src/audio_history/store.rs` とdiff checkは成功した。sourceは新節だけ整形し、旧prefixを保持した。この提出時点ではcompile/fixture/行動未検証であった。独立静的PASS後の指定aggregate結果は以下である。

## 独立静的PASS後のcompile・RED

指定aggregateへ `--message-format=json` を付した1回のcompileはexit 0、3分06.467秒である。[build結果](artifacts/history-publication-authority-tests-build-result.log)、[JSON](artifacts/history-publication-authority-tests-build.jsonl)、[stderr](artifacts/history-publication-authority-tests-build.stderr.log) を保存した。途中source編集・再buildはない。

固定EXEの `--list` で新V19、旧audio_history lib11、旧recovery34、旧store37を実数確認した（合計101）。[件数](artifacts/history-publication-authority-tests-counts.log)、[EXE path/hash](artifacts/history-publication-authority-tests-exes.json) を参照する。lib全件や無関係I2T等の漫然反復はしていない。

- lib EXE SHA `3AADCCBDE8E3E2751200B7721A0B07DBD5AD81B82169C77C951F9E306EC9DB58`
- recovery EXE SHA `B9727916701D317D22FC845920D8785EEAB2D499C6E39889EAFB810B5CEC228B`
- store EXE SHA `890FA20928A627A19CB3ED6CBFCE6B5CCD2741EB4195B0EBFBDAB298157B7CDD`

通常非管理者の固定EXE実行で旧lib11 PASS、新V19は0 PASS/19行動RED、旧store37 PASSであった。新19件は各仕様期待に対して未接続scaffoldの固定Ioで停止した。compile/fixture failure、ignored、V child timeoutは0である。poison caseの意図的catch対象panicはfixture failureではない。[新V stdout](artifacts/history-publication-authority-tests-new-lib.log)、[失敗位置・期待/実値](artifacts/history-publication-authority-tests-new-lib.stderr.log)、[旧lib](artifacts/history-publication-authority-tests-old-lib.log)、[旧store](artifacts/history-publication-authority-tests-store.log)、[初回実行](artifacts/history-publication-authority-tests-run.log) を保持する。

### RED到達と未到達

全19件の「Ioではない期待値／成功期待」の不一致を行動REDとする一方、次の後段は成功に数えない。全ledgerも当該失敗assertの後なら未到達である。

| 群 | 到達した対照・最初のRED | 未到達の主な後段 |
| --- | --- | --- |
| V01 | Some-base手製計画・missing空・complete/refs一致後、verify成功期待 | None-base、最新head/末尾ledger |
| V02 | Trial+receiptを失った偽rootのclosure正常後、token差替えConflict期待 | intentも整合した偽計画、他の偽root |
| V03 token/strict/空白 3件 | operation差替えConflict、正常全文の未知版UnsupportedVersion、exact空白成功の各期待 | 残りfield、raw duplicate、planned refs順序/欠落、上限、semantic-copy拒否 |
| V04 3件 | Trial metadata欠損は旧complete成功/実missing1/手製refs一致、原音欠損は旧complete Missing、最初のlen破損Damaged期待 | root metadata欠損、result欠損、同len hash破損、実read32/unlock対照 |
| V05 2件 | pending存在でPendingMaterial、intent directoryでUnsafePath期待 | pending他状態/head/base/missingintent、asset junction、末尾ledger |
| V06/V07 2件 | next+intentありの旧公開confirm成功後の新confirm成功期待、正常再利用mutationのConflict期待 | 後続root/cleanup済み、finish/retry、verify拒否の全組合せ |
| V07 2件 | disk baseの旧公開Busy、receipt request不一致の旧公開Conflictを通過 | 他head、affected/revision、remaining intent、next/current局所欠損 |
| V08 2件 | custom head後続の旧成功とdisk base新Busy期待、quota1/既存reader設定・実8MiB旧成功後の新成功期待 | disk next切替、8MiB+1、最終ledger |
| V09 held/block/poison 3件 | held G再入拒否対照、block前の正常verify期待、実poison/acquire拒否・旧pure confirm成功後の新confirm期待 | held G下confirm、block設定以降の拒否/pure confirm、poison末尾ledger |

特に実read32・junction・block設定枝はfixture成立をまだ主張しない。GREENでは各loop・全assert・解除後対照・cleanupを最後まで通す必要がある。metadata Ioと検査中置換の既述未測定限界も別途残る。

### 初回旧recoveryの外側timeoutと訂正

初回は31件PASSの後、`w08_recover_scans_100000_entries_but_diagnoses_100001_without_writing` が作者wrapperの `WaitForExit(300000)` でcleanup途中に打切られた。[旧stdout](artifacts/history-publication-authority-tests-recovery.log)、[旧stderr](artifacts/history-publication-authority-tests-recovery.stderr.log) は保持する。100,001診断・bytes不変までは244.516秒で到達したが、cleanup完了assert未到達なので当時は未完了でありPASSではない。続く `w08_structurally_valid_but_forged_old_plan_is_recomputed_before_resume` と `w08_unreadable_or_damaged_current_head_never_offers_resume_as_ready` は未実行であった。

独立reviewは[既存境界packet](history-recovery-boundary-packet.md)の外部20分打切り（性能合格基準ではない）を根拠に、作者任意5分が短過ぎたと判定した。以前の同試験はcleanupまで100.452秒、旧recovery全34件102.119秒であったが、過去所要時間を性能assertへ転用しない。親承認の15分outerで上記3件だけexact再実行する実行方法訂正であり、製品60秒/512MiB・品質assert・OS/harness設定・source/EXEは変更しない。旧31件は反復しない。

今回残存 `F:\TempCodex\ww5-recovery-tests-480-1791441030632808000-64` はstderr nonce一致、PID480/15256終了、解決exact path、全46,641 entries（46,638 files/3 dirs）の配下限定・reparse0・外部target0・既定合成operation名を確認した。同一PowerShellのnative `Remove-Item -LiteralPath` で当rootだけを削除し、absentを確認した。[削除前後証拠](artifacts/history-publication-authority-tests-timeout-cleanup.log)。削除データは再生成可能な合成fixtureであり、他のtempには触れていない。

訂正後のexact3件は各1 PASS/0 FAIL/0 ignored、timeoutなしで終了した。R2の100,001拒否・無書込assertは204.763秒、cleanup完了assertは292.716秒で到達し、test所要293.18秒／外側293.446秒である。他2件は各0.11秒/0.09秒であった。[再実行・前後hash・temp証拠](artifacts/history-publication-authority-tests-recovery-completion-run.log)、[R2結果](artifacts/history-publication-authority-tests-recovery-completion-1.log)、[R2 phase/cleanup](artifacts/history-publication-authority-tests-recovery-completion-1.stderr.log)、[計画再計算case](artifacts/history-publication-authority-tests-recovery-completion-2.log)、[current head破損case](artifacts/history-publication-authority-tests-recovery-completion-3.log) を保存した。R2はinventory entriesの境界であり、closure readerの100k nodes新実測を意味しない。

## 作者工程の返却

- 重複を除く結果は **旧82 PASS（lib11/recovery34/store37）、新V19行動RED、合計101件**。compile failure/fixture failure/ignoredは0。初回旧R2の外側timeout1回は履歴として保持し、訂正後未完了0・新V child timeout0である。
- source4本・固定EXE3本は実行前後hash一致。新V節/prefixの固定hashと既存oracleは不変であり、fmt/diff check成功後にRust sourceを変更していない。
- normal実行のactual tempは `F:\TempCodex` である。sandbox内の既定TempPath観測だけを根拠にせず、旧owned root absentとexact再実行後 `newOwnedTempCount=0` を通常環境で確認した。外側processも全件終了した。
- source/build所有を親へ返却済みであり、今後の作者source編集・再buildはない。末尾記録のみ追記した。これはRED作者工程の完了であり、V実装GREEN/独立runner/最終Postの完了ではない。
- model/effort実行metadataと開始/終了usage counterは未取得。初回外側5分の選択不足による限定再実行を手戻りとして記録した。成功済み旧31件の反復と再buildは避けたが、費用削減率等は推定しない。

## V本体初回試行後のV08 fixture限定訂正（静的確認待ち）

親から実装者の101件中100 PASS/1 FAIL（旧82全PASS・新18 PASS）と独立review P2を受領し、当作者が原因を独立に確認した。[初回失敗](artifacts/history-publication-authority-implementation-new-lib.log) は `Fixture::head` のProgress deserialize行で停止している。8MiB+1 RootRefをdisk Progressへ置くと、予定nextのclosure上限検査より先にfixture自身がInvalidDocumentとなる。両confirm/末尾ledger未到達であり、LimitExceededを実測したとは扱わない。旧失敗ログは保持する。

許可された当V08 caseだけを訂正した。extra0のdisk next・成功期待を保持し、extra1は既存の合法disk baseを変更せず `ProgressHistoryHead.read_current == p.base` とそのclosure正常を先行確認する。上限超過は公開token.nextだけに置き、両confirmの厳密LimitExceededと全ledger不変を維持する。これはfixture前提の訂正であり、error allowlist・assert削除・本体変更はない。実fileだけ超過させるDamagedObject試験への置換もない。TDD skillに従い、壊れたfixtureを製品REDとして扱わず、訂正後の全assert到達は次の実行で確認する。

[変更差分](artifacts/history-publication-authority-tests-v08.diff)、[固定hash/fmt](artifacts/history-publication-authority-tests-v08-fixed.log)、[訂正前hash](artifacts/history-publication-authority-tests-v08-before.json) と訂正前source snapshotを保存した。

- 訂正前store: `2C0423C2C115BC5024881E39E5F5816A4EE2BAAFE20452D0FC2DF1A74D0A1589`
- 訂正後store: `224F2E6CE2BADBD729205EE0BFDC371C2CB01D93BB4EEB5B50D1427EE92B278B`
- 訂正後V節: `9C4FBC3398BF62261E5ADC5D832543C0082414687982B7E3596D9B85A830D019`
- 当case除外hash前後一致: `AF78D218EB096E40D7E458C1B665FFFA15C4256FCE588B9B85C4D4EE52A523F0`。本体・fixture helper・他18件・旧testは不変である。recovery/既存integration2本も上記固定hashと一致した。

新case/改名は0で、新V19件の構成は不変である。rustfmt check成功。現在は独立静的解消確認待ちであり、compile/testはまだ実施していない。作者は当case限定のsource/build所有を保持する。確認後はlibだけ1回rebuildし、固定EXEでV08 exactと新V19/旧lib11の影響範囲を確認する。変更していない旧recovery34/store37のEXE再build・再実行はしない。

### V08静的解消後の限定GREEN・所有返却

独立静的PASS/追加阻止0を受領後、固定source `224F2E6C…92B278B` で `cargo test --locked --lib --no-run` を1回だけ実行し、exit0・94.432秒で成功した。[build結果](artifacts/history-publication-authority-tests-v08-build-result.log)、[stderr](artifacts/history-publication-authority-tests-v08-build.stderr.log)、[開始前hash](artifacts/history-publication-authority-tests-v08-build-before.json) を保存した。

新lib `target/debug/deps/wordweave5-b3388a7bbef44bb4.exe` のSHAは `9CDB2D4D619D8A214911ED4E7E38E8C9A929588916F0E16CFDAE8BB3232BD3C8` である。この固定EXEを通常非管理者で以下の順に実行した。

| 実行 | 結果 | 証拠 |
| --- | --- | --- |
| V08 exact | 1 PASS、0.59秒。extra0成功・extra1合法diskhead先行確認、両confirmの厳密LimitExceeded、各末尾ledgerまで到達 | [exact](artifacts/history-publication-authority-tests-v08-green-exact.log) |
| 新V全件 | 19 PASS、3.05秒。loop後段・全ledger・child末尾を含む | [新19](artifacts/history-publication-authority-tests-v08-green-new.log)、[実read32＋解除後対照mode2/3](artifacts/history-publication-authority-tests-v08-green-new.stderr.log) |
| 旧audio_history lib | 11 PASS、0.55秒 | [旧11](artifacts/history-publication-authority-tests-v08-green-old.log) |

重複を除く独立件数は30 PASSであり、exactを足して31件とはしない。FAIL/ignored/timeout/fixture failureは0。[実行記録](artifacts/history-publication-authority-tests-v08-green-run.log) はsource4本・lib EXEの前後hash一致、normal actual temp `F:\TempCodex` の前後合成root0/newOwnedTemp0、全process正常終了を確認している。fmt/diff成功後のRust編集はなく、当case以外の固定hashも維持した。

旧recovery EXE `8E74C911E510B3CA49A3284080112A07FDB2FF3659C2CB67D1355A09BC1E7E5A` と旧store EXE `FD354E5505A14C097E2646DE7A05E2D2899BD92CDDE2B6FE5F447A99FCC437EA` はbuild前後・実行後とも不変であり、既存71件は今回再build/再実行していない。先の実装者自己試験の71件成功と、今回の作者30件成功の証拠主体を区別する。

source/build所有を親へ返却済みである。初回fixture失敗ログ・REDログ・外側timeout履歴は保持した。これは限定訂正の作者GREENであり、独立最終runner/Postではない。metadata Io・検査中置換・closure reader実100k nodesの既述未測定項目は、この成功で測定済みに変更しない。
