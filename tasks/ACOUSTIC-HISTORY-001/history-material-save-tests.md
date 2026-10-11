# I2a-T 独立試験・M1限定改訂

2026-10-08 / ACOUSTIC-HISTORY-001。担当 `ww-test-author / gpt-6-astra / high`。実行モデル/effortと使用量カウンターは未取得である。TDD skillの独立期待値・RED工程だけを担当し、本体修正・再委任・実データ/API・公開・追加権限操作は行わない。

## 現在の境界

T01–06 の新規21件（lib18＋material integration3）と正式M1改訂1件を固定した。compile/testはまだ未実施であり、独立静的確認待ちである。source/build所有は作者が保持し、実装担当への移譲ではない。新caseの成功、fixture成立、行動REDはいずれも未確認である。

入力は [T最終設計](history-material-save-design.md) `7997BB84DE1ECC1E8662FB323673ED9335E745F5C13CDB7AF2B28D69B542177C`、[正本](design-i2.md) `CBB39AE0518336ADD3C6E8B40F0703525F61287B24946D1362BEF814A556D306` の§5/7、[計画](implementation.md) のT/T-M限定追補、親からの最終Pre PASSである。固定Errの加算的3入口だけを使用し、追加seamは要求していない。

所有は `src/commit.rs::i2_material_save_tests`、同fileの指定M成功seen配列1箇所、`tests/audio_history_material.rs::t_material_save`、本書と試験logである。store、本体、旧124件、R assertは編集しない。

## M作者の限定改訂を先行

対象は `commit::i2_migration_tests::m05_v2_material_private_entry_matches_public_save_with_exact_migration` の成功seen配列だけである。

| 旧oracle | 承認された要求 | 新oracle |
| --- | --- | --- |
| Migration Before/Afterの2件 | T-M初回退避、Intent発火、R-M独立再確認、R更新/finishへ同じfaultsを伝播 | Migration Before/After → Intent Before/After → Migration Before/After → Deck Before/After → MaterialProgress Before/After → MaterialFinish Before/After の厳密12件 |

旧caseの改名・削除・追加は0、成功配列改訂1箇所である。exact backup/intent/保存値/媒体、M fault停止、上限、他assertは保持する。旧2件を許容する分岐や長さだけの比較は追加しない。

M改訂だけを先に行った [証拠](artifacts/history-material-save-tests-m-only.log) の全文SHAは `EA36D471736ECA82D578E1B35C9E8C10FFB6CE64F045CA0DF80B070A2399AF35`。現在のT節より前のprefixもこの値に一致する。配列だけを逆復元したSHAは scaffold全文 `13FECAC910A9E350A5CA63C550C1F1BECD51DE3F783FD61281F1A9A2DF8DA4AC` に一致する。

## T作者の独立oracle

| ID / 新件数 | 期待値・到達判定 |
| --- | --- |
| T01 / integration3 | 手製健全A、候補None/旧B/不正B、全非履歴sentinel、入力不変、公開返却値とexact disk、旧save wrapper。旧Progress欠損時は候補SomeでもNone。archive4fieldの旧新文字列と全ledger、日別backupなし |
| T02 / lib5 | pending file/dir、旧codec未知/破損/同値重複key、候補非履歴不正/runtime1、旧deck UTF8失敗、局所closure欠損/破損/未知、root/pending/metadata reparse、実source read拒否。旧/new deck・Progress・実escaped intentの1byte超過拒否＋ちょうど成功。M前拒否はpublisher/checkpoint0・全ledger不変 |
| T03 / lib3 | 手製旧v1 exact raw→既存固定SHA退避、旧v2/欠損との対照、pending4field exact bytes、12/8 checkpoint。初回M Before/After・BeforeIntent fault残存と、BeforeIntent後の外部metadata/pending変更をrebaseせずP0/no new block |
| T04 / lib4 | 無書込publisher Ok/Errは未保存、実atomic後Errは実再読で確定して成功。AfterIntent faultは旧metadata/exact pending/既知中断→公開recover成功。第三pending/実read拒否/旧消失/metadata変化/欠損旧の出現は隔離childでblock、後続writer無変更拒否とpure read |
| T05 / lib3 | full saveからR各6checkpointへ同じfaultsを伝播、known partialのexact ledgerと公開recover retry、AfterFinish実確定は成功。R-M2回目の同variant発火を回数で狙いpending/退避保持。R内第三bytes化は隔離childでblock |
| T06 / lib3 | callback再入で公開material/normal/historyを拒否、共通GのReentrantを直接確認。同root別表記/別rootの実publisher2区間を観測し非重複・返却値/保存先の非混入。公開normal/history/material競合、T/history区間と同じG保持を確認 |

handwritten root Aと既存固定RAW_V1/HASHを使用する。3object closureは手製DTO＋原音blobであり、製品HistoryWriterの出力を期待値にしない。破損前に期待する3参照と総bytesの健全対照を確認する。重複keyは正常全文の同値keyだけを増やし、必須field欠落を交絡させない。候補FULLは全旧fieldにsentinelを持ち、f32は両側で同じserialize経路を比較する。

private publisherは実filesystemだけを変更する。実read拒否32、同production atomicの実Err、解除後同path atomic成功は別assertである。renameのOS番号を推測して固定せず、N03cの過剰oracleを再導入しない。unlock後成功対照は製品ledger/block assert完了後のfixture操作と明示する。scaffoldではcallback未到達となり得るため、これらの未実行をfixture成立・PASSへ読み替えない。

T06はBarrier、channelの有限待機、実callback区間のactive counter、同thread Reentrant証拠を用いる。「何ms何も起きない」だけのoracleではない。guardをthread間へmoveしない。normal writer自体へ新しい可視性は足さず、公開競合結果、Tが既存Gを保持する直接証拠、既存Gのnormal契約を合わせる。block/再入/競合は同EXE exact1case/markerのchildで、15秒上限とkill/reapを持つ。親globalを汚染しない。

## 固定対象・静的証拠

[固定hash/件数](artifacts/history-material-save-tests-fixed.log)、[owned fmt](artifacts/history-material-save-tests-fmt.log) に記録した。

- commit全文: `6BC503EB3878CA95285F9D4CB91019050E3F4C64532740AB6401B9F54E9C9E6E`
- 新T module: `E6E9D12F45EC348D02602FE0E1B3C917644EE84BEC9902D57BB9D4E2BEEFC244`
- material全文: `332E09754A8504646E7E0EE37DC2ADBE5CE6825CA8504FD94BB4CE985606E635`
- material新T追加節: `81C1430BFD9B1E85499A124F654116A521809339CB1A343011B4D4AF1D883752`
- material旧prefix: `5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683`（R固定全文と一致）
- store全文: `38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF`（不変）

prefix照合は新節の区切り空行のみTrimEnd＋LFに正規化した。旧body内容の正規化はしない。その他G/M/S/N integrationの全文hashも固定logに含む。owned節のrustfmt確認はUTF-8 direct stdin・改行自動追加なしでstdout/stderr空、exit0を確認した。先行PowerShell pipeの余分なEOF改行diffはfmt logに残したが、製品/source差分ではない。

## 後続実行と未測定

独立静的確認を親から受領した後だけ、`cargo test --locked --lib --test audio_history_material --test audio_history_save --test audio_history_progress --test audio_history_migration --test audio_history_gate --no-run` を1回集約実行する。固定6EXE/list/hashで旧124、正式M1、新T21を分離する（合計146は予定であり実行結果ではない）。compile error・fixture不成立・行動RED・後段未到達・ignored・timeoutを区別する。旧成功log/snapshotを上書きしない。

R成功→T最終readbackの隙間faultは既定seamで決定注入できず未測定である。新API、race、OS設定変更で埋めない。実I/O成立、全後段assert、child cleanup、競合動作はGREENで末尾まで再確認が必要である。現時点ではtest/build未実行のため試験fixtureは新規作成していない。native UI/実通信/電源断の受入は対象外である。

## 独立静的P2限定訂正

reviewer指摘を親経由で受領し、T06 callback内のHistoryWriter再入期待値1箇所を `Io` から `Busy` へ訂正した。I1の `gate_error` は Reentrant→Busy、WriteBlocked/Poisoned→Ioである。直接のG Reentrant assertと整合させる誤oracle訂正であり、本体や契約は変えない。blockケースのIo、他assert、件数は不変である。

[P2固定証拠](artifacts/history-material-save-tests-p2.log) が最新sourceである。commit全文 `832A0A000C49E83FB9BAAAEE779053BF3C70E083CC3207C1F307CB55C7FA4BA6`、T module `54F55EDE9A35531C01D3D1840C9947187E31C47C998024A52C80084E41E12CFE`、旧M-only prefixはEA36…のままである。material/storeと旧124 oracleは不変、compile未開始である。初回固定hashは上記に履歴として保持する。

## 静的PASS後のcompile/REDと所有返却

親から独立compile前静的PASS・残存阻止0/P2解消を受領し、上記P2固定sourceのまま指定aggregateを1回実行した。**compile exit0 / 4分01.9秒**である。[開始前7source hash](artifacts/history-material-save-tests-build-before.json)、[build JSON](artifacts/history-material-save-tests-build.jsonl)、[stderr](artifacts/history-material-save-tests-build.stderr.log)、[完了](artifacts/history-material-save-tests-build-result.log)を保存した。途中のsource/期待値編集、追加compileは0である。

[固定6 EXE/path/hash](artifacts/history-material-save-tests-exes.json)をlist確認後、Windows非管理者token（admin=False）で直接実行した。lib旧部分は `store:: commit:: --skip i2_material_save_tests --skip commit::i2_migration_tests::m05_v2_material_private_entry_matches_public_save_with_exact_migration`、M1はexact、新Tは専用module filter、materialは旧/new moduleを分離した。他4 integrationは全件実行である。共通引数は `--nocapture --test-threads=1`。子の件数は親caseへ含め二重加算しない。

| 対象 | PASS | 行動RED | ignored | 証拠stdout（診断は同名.stderr.log） |
| --- | ---: | ---: | ---: | --- |
| 旧lib（許可M1を除く） | 95 | 0 | 0 | [old-lib](artifacts/history-material-save-tests-old-lib.log) |
| 旧material | 3 | 0 | 0 | [old-material](artifacts/history-material-save-tests-old-material.log) |
| 旧save / progress / migration / gate | 3 / 4 / 3 / 16 | 0 | 0 | [save](artifacts/history-material-save-tests-save.log) / [progress](artifacts/history-material-save-tests-progress.log) / [migration](artifacts/history-material-save-tests-migration.log) / [gate](artifacts/history-material-save-tests-gate.log) |
| 正式M1限定改訂 | 0 | 1 | 0 | [M1](artifacts/history-material-save-tests-m1.log) |
| 新T lib | 3 | 15 | 0 | [T lib](artifacts/history-material-save-tests-t-lib.log) |
| 新T material | 0 | 3 | 0 | [T material](artifacts/history-material-save-tests-t-material.log) |
| 合計 | **127** | **19** | **0** | 旧124全PASS、許可M1の1RED、新T21=3PASS＋18RED |

compile failure、実行範囲のfixture failure、child/outer timeoutは0である。[実行token/exit/EXE不変](artifacts/history-material-save-tests-run.log)。M1は保存が完了した後、実seenが旧2件・期待が承認済12件であることを観測した。その比較後のbackup/媒体等assertは今回M1では未到達であり、旧fixture成功をその代用にしない。

### RED原因とGREENで必須の後段到達

- T01: FULLの構造validateと手製A/B fixtureが成立した。公開口は最初の候補Noneで固定Errとなり、B/不正rootの後続枝、返却全field/入力不変/disk/archive ledgerは未到達。欠損旧Progressの口も固定Errである。旧save wrapperは最新Aを継承せず、Rの旧新root不一致Errで停止した。wrapperの末尾ledgerは未到達である。
- T02: preflight負例10枝、局所closure3枝、reparse4枝は全PASS。ただし固定Errでも拒否できるため本体実装済みの証拠ではない。局所closureは具体3object/合計bytesの健全対照と、欠損/破損/未知後の部分root可読・closure拒否対照が通った。junction fixture自体も成立した。上限caseは最初のold-deck 1byte不足の無変更拒否後、ちょうど成功が固定ErrでREDとなり、残る4上限枝は未到達。旧source I/Oはprogress.jsonの実read32、unlock後ledger不変/no blockまで確認し、再保存の固定Errで停止した。custom.tsvの共有拒否枝は未到達である。
- T03: 成功系は固定Err、fault系はcheckpoint空対期待列でRED。初回M退避、12/8全列、exact pending publisher、外部変更callbackとその後のP0/no block/残存ledgerは未到達である。確認済hashを推測でPASSにしない。
- T04: 無書込Ok/ErrとAfterIntent faultはP0対期待P1で停止し、実atomic後Err正例は固定Errである。unknown childはscenario0のpublisher未到達で停止したため、他6scenario、実pending read32、同production atomic失敗/解除後成功、unknown block・全writer無変更拒否・pure readは未測定である。T I/O fixtureの成立を既存R/N実行結果で代替しない。
- T05: 最初のR checkpoint・R-M2回目faultはcheckpoint空で停止した。各後続checkpoint、exact partial ledger、公開recover retry、AfterFinish成功は未到達。unknown R childもAfterDeck callback未発火であり、後段block/ledger assertは未到達である。
- T06: 再入childはouter固定Errでcallback未到達。material2区間は観測0対期待2、normal/history/material競合はhistory側の実区間1対期待2でREDである。Tの共通G保持、区間非重複、再入Busy、全返却値/保存先非混入の末尾assertはGREEN時に全到達が必要である。child timeoutやfixture failureとしては分類しない。

最終 [source/temp確認](artifacts/history-material-save-tests-final.log) は開始前7source hash全一致・owned fixture prefix残存0である。6 EXEも前後hash一致。sourceはcommit `832A0A…C7FA4BA6` / T module `54F55E…E12CFE`、material `332E09…06E635` / store `38D451…F312EF`のままである。追加削除・権限/OS変更は行っていない。

**source/build所有は実行集計とともに親へ返却済みである。** この後は本体実装、全後段GREEN、独立runner/Postである。本書追記のみを行い、source再編集・再buildは行わない。R成功後T最終readbackの決定fault注入不可とnative/電源断等の未測定境界は引き続き残る。これは最終Postではない。
