# I2準備設計レビュー

2026-10-08 / ACOUSTIC-HISTORY-001。既定I2 reviewer gpt-6-astra/high、独立read-only `history_commit_review`。HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`、branch `codex/development-agent-team`。I1b-2bは未完了依存であり、設計判定と本体着手許可を区別する。使用量・実行metadataは未取得。

## Cycle 1 — CHANGES_REQUIRED

対象[design-i2.md](design-i2.md) SHA `73219261B073DA6E8301373ADCEE57B9DF4FA98B1BF4640CDBD6C122B4A657C5`。review開始/終了で一致、コード編集・build・試験なし。

| ID | 確定指摘 / 再現条件 | 必要な契約・独立証拠 | 状態 |
| --- | --- | --- | --- |
| I2-D1 / P1 | 教材pending recoverが旧新root同一を要求しない。old=A/new=Noneでも構造・next closure・現在metadata exact整合を通り、commit_history以外で履歴rootを消せる | I2教材intentはold/new root一致。old_progress=NoneのみNone、Someはstrict decode。A→None/A→B/旧なし→Some/旧不正の全拒否と両metadata/pending bytes保持。I3 unionは別権限 | designerへ差戻し |
| I2-D2 / P1 | settings_ui.rsのJSON backupが任意pathへself.progressを直接atomic_writeし、稼働progress.jsonを選べる。stale root/書込停止を迂回するconsumerが所有表から漏れた | 稼働rootの保護対象への書出しを別表記含め拒否。通常外部exportは維持。owner/consumer/oracle追加、拒否時全bytes保持 | designerへ差戻し |

規範根拠はAH-AC04/16と「新root変更は明示履歴commitだけ」「全writerが最新rootを保持」である。その他、版dispatch、exact移行退避、旧v1 pending、通常root継承/UI返却、Publishedとintent計画束縛、gate再入/poison、取消し、v1backup互換、分割を確認した。上記2点訂正後に差分を独立再確認する。

## Cycle 2 — PASS（設計delta）

対象SHA `B8B1FD5FEA1518C4AE516E0B47C9502267A0C05A5C2EF4EA4A979282E8267D51`。同じ独立reviewerがread-onlyで再確認し、I2-D1/D2を設計上Closedと判定した。本体の実装・動作検証を完了した意味ではない。

- D1：旧Progressのstrict decode、old/new root完全一致、旧Progressなしの扱い、拒否時全bytes保持を§5/I2-T05bで固定した。
- D2：保護範囲を稼働root全体とaliasへ拡張し、JSON/TSV/診断/媒体/backupの全consumerを所有表へ接続した。通常外部退避を保ち、拒否時のbytes/payload保持をI2-T08/08bで検証する。
- 新しい受入阻止事項なし。I1b-2bのPost完了は未達依存である。Windows aliasの実装・試験は今後のI2であり、設計PASSで代替しない。

親は§8の重い単位をplannerへ返し、薄い逐次handoffを確定する。本体着手は依存gateを満たした後である。

## I2a-G private接合追補 — Pre PASS

対象design-i2.md SHA `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA`。旧Cycle2から§4冒頭のG限定接合だけを追加した。designerの現caller調査を親が統合し、同独立reviewerがread-onlyで判定した。

load→recover/教材save→recoverのunder_guard接続、履歴4書込入口の取得位置、既存resume/finishの非再帰呼出しと整合する。再入拒否、poison非解除、guard下block再確認、strict pending fail-closed、別processのpoison/block oracleは明確であり阻止なし。G単位で結果不明検出/schema移行/停止後閲覧完成を主張しない境界も維持される。

実装・試験は未実施である。先行I2a-0 Post依存は残る。旧I2-D1/D2の設計判定を無変更部分まで再実行した意味ではなく、このprivate API/接合deltaのPreだけである。

## 後続I2a-M/Rの準備で発見した経路 — 未実装・契約追補待ち

親の反例確認により、designerの「first-v2-write退避hookは通常save/新教材saveの2箇所で足りる」という準備提案を撤回した。valid pending old=v1/new=v2をrecoverする際、先行backupの存在は保証されない。現Progressが既にnewの場合もold exact bytesはintent内に残るため、recoverで全事前検査後・metadata書込前に条件付き退避が必要である。新wire版1の旧pending exact replayは変えない。

designer確認済み修正案：Mはgate内のdisk由来または検証済みintent由来の旧exact bytesをcontenthash名へ退避・再読するhelperを作る。移行要否は正規化runtime.versionでなく元bytesのwire版を使う。Rはnew wire2/old wire1でhookを接続し、backup欠損・current old/newの両ケースを検証する。old=Noneは退避不要、旧破損/未知版/既存backup異bytesは拒否し上書きしない。

検査不合格と既存backup不一致では全bytes/backup/temp不変を要求する。検査後の退避I/O失敗では正しいbackupやtempが残り得るが両metadata/pendingは不変とし、確認済みbackupを削除しない。Mはまだv1 runtimeなのでprivate helper単体の証拠とし、S後の両公開save経路/R後のrecover実v2を別検証する。

この項目は後続契約の準備記録であり、G契約・source・Pre PASSを変更しない。M着手前に正本/所有分担/受入へ統合して独立Preを得る。未保証の仮定を実装へ持ち込まないための指摘であり、既存製品のデータ喪失を観測したものではない。

## I2a-M限定設計案 — M-D1、正本統合前

独立reviewer `history_gate_review`（既定Astra/high）が[移行準備案](history-migration-design.md) SHA `FCA2F017733F2B9C9DB562CDEBB02B14640613697F20360636F64E55EFD371D0`をread-onlyで検討した。Gの独立実行と文書検討だけを並行し、M本体の依存gateは変更していない。正本は固定SHA `2F362CD760AA093A7EA255EB03E6C322226AC0F7936EAD5E5FFC43DA8CAB51DA`のままである。

**M-D1/P2、Pre阻止**：target=1での旧source読取・長さ/版/構造検査が、現行v1通常保存の維持と不整合である。旧v1がprivate limitを超え、新v1候補がlimit内へ縮小する保存は現行では成功するが、準備案の旧disk bounded readにより拒否される。helperだけno-opにしてもcaller読取で回帰する。

最小訂正は、target=1でgate確認後に移行処理を終了し、通常saveの移行目的の旧disk読取も省くことである。新教材saveの既存旧metadata読取/上限確認は維持する。旧source必須版・構造・長さ検査はtarget=2に限定し、上記v1保存成功/hash退避追加なしとtarget2のLimitExceeded/無変更拒否を対照試験に追加する。designerへ差戻し、未解消である。

他の確認範囲（exact退避/元wire版/検証済intent/current=new復旧/事前拒否とI/O残存/rename後Err試験口/S・R分担/3file所有）には阻止なし。文書・G source hashは前後不変。実行・編集・再委任はない。正本統合と最終Preは別途必要である。

### M-D1訂正案 — 解消、正本統合待ち

同reviewerが訂正版SHA `27D5233049D7F7B370381A77196335FBB649B9CB2FAED060CF188E3257F040AA` §2/3/4・M05を照合し、M-D1は準備案上で解消、追加阻止0と判定した。target=1のgate後no-op・通常saveの移行用旧disk読取省略、新教材既存検査の維持、target=2の旧source検査と縮小保存/上限拒否の対照oracleを確認した。正本の統合とその照合まではM最終Pre未完了である。Gは別記録で最終Post PASSとなった。

### I2a-M正本統合 — 最終Pre PASS

同reviewerが正本SHA `1375E2A126DBA2AEAD188903D3BA3E0575293B3C919C7721D5786EB771090EAF` §2/5/7、計画SHA `DEAC1E25A94F60501176B1E146CE3ED28CAF5A3BC68F13A6260383A3A9558E8F` M/S/Rと固定準備案を照合し、M限定Pre PASS、阻止0と判定した。M-D1をClosedとする。

検証済intent旧exact bytes/current=newからの退避、全事前検査後の順序、M/S/Rの未達境界を保持する。追加private publisherは新規退避時だけ最大1回、production/test共通の検証・再読を通り、rename成功後Errを再現できる。公開filesystem差替えを増やさず、既存limit署名維持・and_faults取得1回・NoProgressFaultsはGと整合する。

編集・build/test・再委任なし。G Post依存は解消済みであり、これでMのscaffold/独立testへ進める。Mの実装成功、S/Rのv2受付/全移行保護まで完了した意味ではない。

## I2a-S正本統合 — 限定Pre PASS

2026-10-08。同独立reviewer `history_gate_review`（指定gpt-6-astra/high）が限定設計SHA `CE1B45349BD7DF4C20DBBC22ADF4C24888DB681EB7533A8244864830E891BF63`、正本SHA `5E4894D3A2EC7CAFA87115D45AD74B293DFAD01B2C8F436EF4D10D4379E2E714`、計画現物SHA `90453DA0…25CD657B`を確認し、S限定Pre PASS、阻止0と判定した。

単一Wire→normalize、必須nullable/重複拒否、元wire版保持、typed pure read/head、leaf NotFound限定、under_guard load、最大4fileとM oracle移行は整合する。scaffoldはskip/Default1/旧deriveを保ち、新入口だけ固定Errとする。旧試験成功→独立oracle改訂/行動RED→実codec接合の順で進む。commit本体/G/旧test不変、R-M/I10未達の境界を維持する。編集・実行・再委任なし。

親はM独立Post完了と上記Preを受領し、S限定計画deltaを確定した。実装・試験成功の判定ではない。

## I2a-N準備案 — 静的PASS、最終Pre未了

2026-10-08。同独立reviewer `history_gate_review`（指定Astra/high）が限定案SHA `485BCB288CE3CF8863FA60F791581F104E8FE4B2D4462DEAF88647CF705ADACF`を確認し、静的PASS・阻止0と判定した。latest disk root継承、候補nonhistory保持、同raw snapshot退避、closure、Ok/Err非依存のexact三分岐、old==next成功優先、第三状態だけのblock、pure read、N/P checkpoint分離、最大1回publisher、2file所有は整合する。

最終PreではS Post後の実bytes helperとの統一を具体化する。また日別leafの危険pathが移行退避後に検出される場合、正常なhash退避だけは残り得ることをledgerへ明記する。全不変を要求するなら日別path検査をMより前へ移す必要があるが、現案自体をデータ損失欠陥とは判定していない。正本未統合・S Post未了のためN着手許可ではない。編集・実行・再委任なし。

### I2a-N正本統合 — 最終Pre PASS

同reviewerが正本SHA `8D3A6EAD1704F05B3F4F2715E4DF297D3940E84C31C378662A181005802B7A84`、限定設計SHA `45A7E9C9B08114D9EC00C39B5A3E252F1CB21F0136C62130C82DD1C78651895F`、S Post時のstore SHA `4C1287B2CDB4DE4621A39E06ED67DE2A181A272B7FB67507F4708C648FD00EEE`を照合し、N最終Pre PASS、阻止0と判定した。

S安全分岐を維持したraw helper抽出、同一snapshot利用、N03bの先行hash退避残存、exact再読のnext優先/old維持/第三状態block、実保存値返却とOk/Err共通判定は整合する。新入口だけの固定Err scaffold・旧save未接続、2file所有、既存M/S/G oracle、N/P checkpoint分離も維持する。S Post依存は解消済みであり、Nの独立test/実装へ進める。N実装・GREEN・Post、R/I10/native受入を承認した意味ではない。編集・実行・再委任なし。

### N日別write I/Oの検証可能性delta — 限定Pre PASS

同reviewerが正本SHA `7E93546677EDE42F1B194A277BC222463866D2392D78524C69B033905BCF9024`、限定設計SHA `46504FBFE45C01FF6A5140794220A3CF06BAAB91D31B48D103420CA13A32EEDE`を照合し、日別I/O delta Pre PASS・阻止0と判定した。単publisher署名を維持しprivate二publisher最内側でgate1回、daily検査/保存はproductionとtest共用、新leafだけcallback、daily Err後の正本publish0/new blockなし、2fileと既存oracle維持は整合する。

N03cは実atomic_writeの失敗・伝播と別rename probeのraw OS 32を分離し、M退避/fixture sentinel・probe/atomic tempをledgerで区別する。実OSエラーは今後の試験でありPreだけでは実証しない。レビュー中のstore hash変化は独立作者の試験作成中でありsource固定の判定ではない。新署名は未接続・未追加のまま確認した。編集・実行・再委任なし。RED/実装/独立GREEN/Postは未了である。

## I2a-R準備案 — 限定静的PASS、最終Pre未了

同reviewerがR設計SHA `F4891F76D26FEFA648582126BE2457DD5CFF494CCF76BCFE99D1528AC1AAA7A1`、commit SHA `C1E7D086D12BB872B8B756616030EE12EFF4E150065D72539B3BBAA68CE022AD`を前後照合し、限定静的PASS・阻止0と判定した。全事前検査、旧新root完全一致、元wire1 exact replay、current=newでもintent旧bytesから退避する契約は正本§5と整合する。

codec bridgeは版/検証済Progressだけを返し権限を追加しない。gate1回・最大3file、metadata書込結果の実再読とknown pending/unknown blockの区別、archive事前/直前検査・rename後Err/AfterFinish確定・未rename時のbackups欠損の扱いは整合する。R/T checkpoint分担とNoProgressFaults経由でM/S観測順序を維持する。編集・実行・再委任なし。N Post・正本統合・R最終Pre・本体/独立試験・T/I10は未達であり、R着手許可ではない。
## I2a-R 最終Pre（2026-10-08）

Nの独立100件・最終Post Closed後、既定history_gate_review（ww-reviewer / gpt-6-astra / high、read-only）が正本統合を限定確認し、R最終Pre PASS・阻止0と判定した。正本 `C4CF33CE6C7B69D8106D216497B4897C545C0E46169DEE417B7E27E96903B3D3`、R限定設計 `0C1109DC7B41DBC2E0ACF7B01366FE6622DAB57404B76DB31E5D75A407DBE951`、store/commitは変更前snapshotと現物一致である。

codec bridgeの最小可視性、旧新rootの権限、known/unknown分類、rename後Errのexact確定、archive事前/直前検査、最大3file所有、既存G/M/S/N oracle保持は予備PASSから矛盾なく正本§5/7へ統合された。着手gateは通過したが、独立作者の期待値固定→必要な未接続scaffold→行動RED→本体接続の順を維持する。R実装/Post、T/I10/製品受入をこの判定で完了にしない。

## I2a-T準備案 — 予備静的PASS、最終Pre未了

既定history_gate_review（Astra/high、read-only）がT設計 `9B805A51681B70A3105DA10D5042A1F529EA8A6D605034B6AA4A7D25AD33FD74`、plannerのT-M限定所有追補後の計画 `DDDD0E44D7B6F1A7FA76D0DBAA073EE33ECFAA0F2E2D0776D87D7DB9AD4C7B5A` を確認し、予備静的PASS・阻止0を返した。入力hashは前後一致した。

最新root継承→候補検証→escaped上限→旧exact退避→pending公開、publisherのOk/Errと実再読の分離、同guard/faultsでRを再利用し最後に今回のnew exactとpending欠損を確認する契約は整合する。新root権限や別recoverは追加しない。既存M helperは一致backupでも両checkpointを発火するため、成功seenの厳密2→12改訂は必要である。plannerはM作者へ当該1配列だけを割り当て、他assert/旧fault証拠と最大3file所有を保持した。

残る確認はR最終Post後の実private署名・固定差分との照合、正本統合、T最終Preである。T source/test着手はまだ不可であり、この確認で実装/build/testは行っていない。

## I2a-V準備案 — 予備静的PASS、最終Pre未了

既定history_gate_review（Astra/high、read-only）がV設計 `88826DCBB1E41AB33E0005F6A7D10DDD9D9A29851C2D0B131EB7E69E9FDA25A7`、audio_history/store.rs `7532A07DA9BED7F79AE0B1E28806CFE881BE3B2AC5973E73F39F11F44B51FC67`、recovery.rs `7D90C1FB10D200DC42248F83E0C3224BC8A4347CEC7EC16D1009DABA489EC012` と既存偽造/復旧試験を照合し、予備静的PASS・阻止0を返した。入力は前後不変である。

&Storageの読取helper抽出で既存再計算を共有でき、可変writer再構築や公開API拡張は不要である。最新base/strict intent/exact mutation/順序付きplanned objectsの束縛と、complete以外に実fileのmissing空を要求する点はT03bと整合する。既存reader上限とmissing許容recoveryを維持し、Vの新commit権限とPの既commit/cleanup後再確認を分離している。

正本統合・V最終Pre・独立oracle/本体証拠は未了。R/T完了前のV source着手を許可する判定ではなく、編集/build/testは行っていない。

## I2a-T正本統合 — 最終Pre PASS

R最終Post Closed後、既定history_gate_review（Astra/high、read-only）が正本 `CBB39AE0518336ADD3C6E8B40F0703525F61287B24946D1362BEF814A556D306`、T限定設計 `7997BB84DE1ECC1E8662FB323673ED9335E745F5C13CDB7AF2B28D69B542177C` と実store38D4516D…F312EF/commit24414D96…79C6A6を照合した。T限定最終Pre PASS、静的阻止0である。

最新root継承、全事前検査、pending前exact退避、publisher Ok/Err共通の実再読、R成功後の最終exact確認と実保存値返却を維持し、Rのcodec/read/同guard-faults入口/error変換へ接合できる。別recoverや追加gateは不要である。最小3口・最大3file・逐次所有、M成功seen1配列だけの厳密2→12改訂と他G/M/S/N/R oracle保持を確認した。

次は独立作者の期待値固定→必要な未接続scaffold→行動REDである。T実装/Post、V/P/I10/native/製品完成はこの判定に含めない。レビューによる編集/build/testは行っていない。

## I2a-V/P confirm共有追補 — 予備静的PASS

2026-10-08、既定history_gate_review（Astra/high、read-only）がV `F0E48A1AA5B93BEFAFDAE2E0EA668E5EE620C2249EDC1DFB31EB9693E0A9E833`、P `6D4A507A0CEE57C58F9E414B7C8D12E74D54DE93B112A1513DD2AA60639880B0`、計画 `61980333CCFF2ED1DD294B6013A7A6349F0518B07C7BEF5B68BB01C75AF243FC` と現confirm/再計算処理を照合した。予備静的PASS・阻止0、最終Preではない。文書3本とsource2本のhashは前後不変、編集・実行なしである。

共有bodyで公開I1のcustom head・reader上限・error順序・cleanup後成功を維持でき、新confirmだけ固定disk headを使用する。read-only確認とVの新pointer権限は別であり、追加gate/書込停止変更はない。Pの既commit優先・最新非履歴field保持・取消直前判定・exact再読と未知状態blockは整合し、V07–09/P01–08は実装可能である。

非阻止の記述訂正をdesignerへ返した：PのBusyをbase上receiptなしと同一視しない。現confirmはSetTrashのRunning/Sending拒否Busyも伝播する。BusyはV必須再検証へ進む候補であり未commitの証明ではないこと、該当intentでpublisher0の対照を明記する。後続Vの同一計画再検証が拒否するため安全性阻止ではない。T Post→V統合/最終Pre、V Post→P統合/最終Preを維持する。

designerがP §2手順3/P02だけを訂正し、親が該当記述を確認した。訂正後Pは `80DD647FFA358F253F9C355F9D485A0975D54773A4DD53B6CB986640AFC045EE`、V/計画/正本は不変である。これは非阻止文言の反映確認であり、依存Post前のP最終Preではない。

## I2a-V正本統合 — 最終Pre PASS

T最終Post Closed後、既定history_gate_review（Astra/high、read-only）が正本 `70749C50224F8051949487ABBFE436AEFE642637B98CA9B488BFE6484E6E92AF`、V案 `A4F5199EA1BB7D7A4D4ADECC4FC76BC9BBFC6A3C05E4E3716E742D4A8A347215`、I1 store `7532A07D…B51FC67`、recovery `7D90C1FB…89EC012`を照合し、V限定最終Pre PASS・阻止0と判定した。

strict intent再計算、complete＋missing空、confirm共有/custom head互換、新confirmの固定disk head・pure read・再gateなしを現I1へ接合可能である。V01–09、最大3file、既存oracle保持、未接続scaffold→独立行動REDの順を維持する。追加4段落を除いた正本SHAが旧 `CBB39AE0…A556D306` と一致し、他章不変・PのBusy訂正保持、現物hash前後一致を確認した。編集/build/testは実施していない。

V実装・Postは未達。PはV Post後に実helper照合・正本統合・独立最終Preを必要とする。

## I2a-P正本統合 — 最終Pre PASS

2026-10-08、V最終Post Closed後、既定history_gate_review（Astra/high、read-only）がP限定最終Pre PASS・阻止0を返した。正本 `E53C18E88DCDD23F7FC843A3A363F4FE4108E0B8BE2F0689CE8DE30D18F568D7`、P案 `13DF70DAD3C247E83AACC384169B507145464C65E40007D347FCD59E4AE1891A`、store38D4516D…F312EF、V store224F2E6C…92B278Bを照合した。

正本の差分はP追補5段落と§4手順5の置換である。旧文言受領後、レビュー側が追加段落を除き手順5を復元したメモリbytesは旧SHA `70749C50224F8051949487ABBFE436AEFE642637B98CA9B488BFE6484E6E92AF` と完全一致した。rootだけでなく予定new全文bytes／旧全文raw／第三bytesで分類する具体化はP案§3と一致し、V/Tや他章の契約は不変である。既commit入口の後続root許容とは別の境界である。

承認済み2file所有、最小enum＋3口の未接続scaffold、独立oracle固定へ進む。レビューによる編集/build/testなし。P実装・Post・I10・nativeの成功判定ではない。
