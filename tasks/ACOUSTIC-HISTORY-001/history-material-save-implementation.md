# I2a-T 未接続scaffold実装記録

2026-10-08 / ACOUSTIC-HISTORY-001。指定は `ww-implementer / gpt-6-astra / high`、実行metadata・usage開始/終了counterは未取得である。AGENTS、development-team、KPI、wordweave-change全文を確認し、実装役の所有範囲に限定して適用した。

親からT最終Pre PASS、作者PREPの3口署名確定、source所有返却を受領した。入力はT限定設計全文 SHA `7997BB84DE1ECC1E8662FB323673ED9335E745F5C13CDB7AF2B28D69B542177C`、正本§5/7 SHA `CBB39AE0518336ADD3C6E8B40F0703525F61287B24946D1362BEF814A556D306`、implementation.mdのT行/T-M限定deltaである。文書中の過去の未Pre記録は親の最終Pre受領で解消している。

今回のproduction変更は `src/commit.rs` に次の新口3つを追加しただけである。

- public `save_progress(root, deck, candidate) -> Result<Progress, String>`。
- private `save_progress_with_limits_and_faults(root, deck, candidate, limits, faults) -> Result<Progress, String>`。
- private `save_progress_with_intent_publisher<P>(root, deck, candidate, limits, faults, publish_intent) -> Result<Progress, String>`、`P: FnOnce(&Path, &[u8]) -> Result<(), String>`。

各引数はunderscore付きであり、全新口は固定 `Err("教材保存は未実装です。")` を返す。相互にも既存入口にも接続せず、callback実行・I/O・gate取得はない。旧save/limits/R/codec/G/M/Nおよび全testは変更していない。store本体は編集していない。

親の `artifacts/history-material-save-before` は上書きせず保持した。新3口の連続blockだけをメモリ上で除いたcommit全文は同snapshotと完全一致した。最初のcfg(test)からEOFまでの全test suffixも一致した。targeted `rustfmt --edition 2021 src/commit.rs` と `git diff --check -- src/commit.rs` はともにexit 0である。

| 対象 | SHA-256 |
| --- | --- |
| commit変更前/snapshot | 24414D9610D0970DDE11B09C17F62BB92051BD066245F5F316FD0F466A79C6A6 |
| commit scaffold後 | 13FECAC910A9E350A5CA63C550C1F1BECD51DE3F783FD61281F1A9A2DF8DA4AC |
| commit全test suffix before/after | 7CDDD3A3381FDD3CB1604C6032651DDF12619C9EB0FC06524BB2744DAB290DBE |
| store不変 | 38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF |
| material integration不変 | 5AC9BBC63B5F519B8E477A47CC4D81707016CB5E8AEBFFE6F1E5E3E2F53A0683 |

cargo/build/testは今回実施しておらず、compile・行動は未検証である。固定Errの新口をcompile可能な署名に置いた段階であり、Tのlatest-root継承・pending exact分類・fault伝播・実保存Progress返却は未実装である。独立作者が次に新T testと正式M成功seen 1配列だけを逐次所有し、静的確認/compile/行動REDへ進む。本体接続はその後の別指示を待つ。

source/build所有を親へ返却する。他者編集を戻さず、実data/API/UAC/公開/再委任は行っていない。自己scaffold確認を独立Postや利用者受入と同一視しない。

## 独立RED受領後の本実装・自己確認

同日、同じ役/model/effort指定で本実装へ進んだ。設計SHAと最終Preは上記のままである。新T21件=3 PASS/18行動RED、旧124全PASS、正式改訂M成功seen 1件=実測2対期待12のREDを受領した。作者によるcompile/初回REDの記録は保持し、本体修正へ合わせて期待値を変更していない。

受領commitは `832A0A000C49E83FB9BAAAEE779053BF3C70E083CC3207C1F307CB55C7FA4BA6`、新T module `54F55EDE9A35531C01D3D1840C9947187E31C47C998024A52C80084E41E12CFE`、material integration `332E09754A8504646E7E0EE37DC2ADBE5CE6825CA8504FD94BB4CE985606E635` である。正式M1配列改訂は作者の所有であり、実装者は受領後の全cfg(test)/integrationを変更していない。[before証拠](artifacts/history-material-save-implementation-before.log)。

### 本体変更と固定

変更したproductionはcommit.rsだけである。最内側publisherでGを1回取得し、strict pending→旧raw一読/codec→最新root全closure→候補cloneのrootだけ継承→候補validate/新deck/旧新上限・実escaped intent上限→M exact退避→BeforeIntent→旧exact再確認の順に接続した。旧deckはUTF-8だけ確認して元文字列を保持し、旧Progressも正規化runtimeから再serializeしない。candidate本体と全非履歴fieldを保持する。

pending publisherは最大1回であり、Ok/Err共通でAfterIntentと実pending/metadata再読を行う。旧exact＋pending欠損は未保存、旧exact＋pending exactだけが確定、第三状態/検証不能はprocess blockでRを呼ばない。公開前の外部変化を新oldへ取り直さず拒否し、新しいblockを設定しない。Rへ同じguard/faultsを渡し、R Errを維持する。R Ok後も今回の両new exact/pending欠損を確認してからactual Progressを返し、不一致はblockとする。

旧save/limits/fault入口は同一本体の返却値だけを捨てる互換wrapperとした。不要になった旧非strict `read` を削除し、既存R helperを再利用した。store、R production、G/M/N、全testには変更を加えていない。新日別backup・Intent schema・公開root変更権限・別recover実装は追加していない。

14:52:51 +09:00に整形を完了し、[固定source証拠](artifacts/history-material-save-implementation-fixed.log)を保存して親へ通知した。その後だけcargoを起動し、build/test中のsource編集は0である。R production全文はbefore snapshotと一致、全test suffixは受領時と一致、fmt/diff exit 0である。親経由の独立静的reviewも阻止0を受領したが、これは実行成功とは分離した。

### Buildと固定EXEの初回結果

```text
cargo test --locked --lib --test audio_history_material --test audio_history_save --test audio_history_progress --test audio_history_migration --test audio_history_gate --no-run
```

[集約build](artifacts/history-material-save-implementation-build.log) は1回、exit 0 / 4m20sで成功した。続いて非管理者tokenを検査し、[固定6EXE実行前path/hash](artifacts/history-material-save-implementation-exes-before.log)を保存して直接実行した。全実行に `--nocapture --test-threads=1` を付け、同caseの二重実行・二重加算をしない。

| 群 | PASS | FAIL | ignored | 秒 | log |
| --- | ---: | ---: | ---: | ---: | --- |
| 旧lib（store/commitからT節と正式M1を除外） | 95 | 0 | 0 | 15.57 | [old-lib](artifacts/history-material-save-implementation-old-lib.log) |
| 正式M1（exact case） | 1 | 0 | 0 | 0.36 | [m1](artifacts/history-material-save-implementation-m1.log) |
| 新T lib（i2_material_save_tests） | 18 | 0 | 0 | 7.68 | [t-lib](artifacts/history-material-save-implementation-t-lib.log) |
| 旧material（t_material_save節を除外） | 3 | 0 | 0 | 2.60 | [old-material](artifacts/history-material-save-implementation-old-material.log) |
| 新T material（t_material_save節） | 3 | 0 | 0 | 0.94 | [t-material](artifacts/history-material-save-implementation-t-material.log) |
| save | 3 | 0 | 0 | 0.46 | [save](artifacts/history-material-save-implementation-save.log) |
| progress | 4 | 0 | 0 | 0.04 | [progress](artifacts/history-material-save-implementation-progress.log) |
| migration | 3 | 0 | 0 | 0.90 | [migration](artifacts/history-material-save-implementation-migration.log) |
| gate | 16 | 0 | 0 | 6.37 | [gate](artifacts/history-material-save-implementation-gate.log) |
| 合計 | **146** | **0** | **0** | — | **旧124＋正式M1＋新T21が全成功** |

本体実装後の初回試験に失敗はなく、fixture failure/child timeoutも0である。全新T caseが末尾まで成功し、REDで未到達だったlatest-root/全非履歴/入力保持、個別上限ちょうど、strict/reparse/whole closure、移行退避、12/8厳密checkpoint、公開前変化、publisher後実再読、R-M2回目fault/全R checkpoint、actual返却値のassertへ到達した。正式M1も厳密12件と残るexact backup/intent/媒体assertを通過した。

t-lib logでは旧progress/custom.tsvの実read拒否32、pendingの実read拒否32（mode1/5）、同production atomic自身のErr、block/ledger確認後の解除・同path atomic成功を観測した。unknownの7行とR第三状態を子process隔離で確認し、全writer停止・pure read・rollbackなしを通過した。callback再入はG Reentrant/History Busy、同root別表記・別root・公開normal/history/material競合は既定有限待機/実区間oracleで成功した。子process内部件数を合計へ加算していない。

### 最終固定値・返却

| 対象 | SHA-256 |
| --- | --- |
| 最終commit | 14952768353B2111D7AF9C96541B8F321AC79BDD30FE26CF9C41A3EA188E0154 |
| store不変 | 38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF |
| 全commit test suffix受領/最終 | DAD7F8632F22E528A9E19F956338A1898911EF6B9BD0EC0C0B6898990B92307A |
| T module受領/最終 | 54F55EDE9A35531C01D3D1840C9947187E31C47C998024A52C80084E41E12CFE |
| material integration受領/最終 | 332E09754A8504646E7E0EE37DC2ADBE5CE6825CA8504FD94BB4CE985606E635 |

[最終source/test/6EXE/cleanup](artifacts/history-material-save-implementation-final.log)は14:59:31 +09:00である。既存4integrationも受領hash不変、6EXE前後hash一致、fmt/diff exit 0、所有fixture prefix残存0、追加削除なしである。固定EXEは以下の6本、すべて `F:/Kazuhiro/GitHub/WordWeave5/target/debug/deps/` 配下である。

| EXE | SHA-256 |
| --- | --- |
| wordweave5-b3388a7bbef44bb4.exe | D054CCAFE13FF6DAC8BAC3E58400A3567292BC2F3CA7BDB1562F4F011AA81C6B |
| audio_history_material-d89f83041366a9a0.exe | B487413B28C318E4D333F5C7106B14C5EB8712C106A6F93C39B121E99F1BBCE4 |
| audio_history_save-3ab33269b9010ef4.exe | 319D9B2E7A770ECC0A6F54E3C3A7A68AA299586EE09D83E84790B08B343983CD |
| audio_history_progress-892b5d70a3c847c8.exe | 8DC078E5566CDBB2FC8E55B87F27C657A66D1381D5C83DA436E54C6646F83110 |
| audio_history_migration-33b6a419850044f4.exe | B3415331E4764F7E42629F34F0FF84C9884F85E0E7CE12BCCB59B0728923BE1E |
| audio_history_gate-0c0e7e180bf0149a.exe | 5728E1A60A118D6628D56670B310EFC66E527EB78810484B9E069940E5A15736 |

R成功→T最終readback間の隙間faultは既定seamで決定注入できず未測定であり、新seam/raceで埋めていない。実data/API/native/電源断、consumerの返却値採用、V/P、製品全体release gateは未達のままである。指定model/effortは維持し、runtime metadata・usage開始/終了counterは未取得、推計なしである。

最終固定現物確認後にsource/build所有を親へ返却し、その後は本記録だけを追記した。独立runner/Postと利用者受入は別工程に残す。
