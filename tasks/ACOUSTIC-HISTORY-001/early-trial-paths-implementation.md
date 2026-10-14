# ET1 本体接合・段階分類の再開

2026-10-14。担当 `/root/trial_paths_finish`、planner指定 `gpt-6-astra/high`。実行model/effort metadata・thread UUID・開始/終了usage counterは未取得である。親の測定記録は編集しない。

## 範囲・前提

承認済み契約 SHA256 `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D`、probe設計 `40A12BE64F07136DEBE410FB52C22A124D5AC89CDB8D1666E5E19FCF52C05D81` と独立Pre PASSを確認した。AGENTS、開発7役、KPI、wordweave-change全文を読了した。UI変更なし。

開始HEADは親指定 `61d0ef5`。既存dirtyのstore/commit/ai本体、作者のfixture修正、親のusage/current記録を保持した。本再開の本体所有は `src/store.rs` のET1段階分類だけである。commit/ai、作者テスト、manifest、設計には変更なし。前セッションの通常4PASS・試用32PASS/fixture1FAILと、作者段階3の修正済みexact1PASSは旧snapshotの証拠であり、本変更後のGREENとは扱わない。

## 修正

`open_early_trial` が `TrialOpenStage` を保持し、private `TrialPreflight::create_root(&mut self, stage: &mut TrialOpenStage)` に引き継ぐ。create_root内部の読取再検査中は現在の段階を維持し、各実mkdirの直前にのみStartedへ進む。root既存の場合は、create_root後の最終読取再検査もBeforeWritesのままである。app.lock open直前にStartedを確定し、それ以降のすべての失敗はStartedで返す。StartedからBeforeWritesへ戻す代入はない。

reasonと安全文は既存型を維持する。String parse、新公開API、fault seam、検査bool注入、追加guardはない。probeの9点/cleanup一回、OnceLock登録、root検査、load/recover順を変更していない。

## 本体固定・作者への逐次返却

親の追加指示に従い、本体を固定してsource所有を一旦返却する。独立作者の新しい小回帰後にfocused compile/runtimeをまとめて実施する。今回の固定時点ではcompile/runtime未実施であり、GREEN未主張である。

`cargo fmt --all -- --check` と `git diff --check` は成功した。初回fmt checkが示した本変更部分だけをapply_patchで整形し、再check成功。無関係なsourceの自動formatは行っていない。

| 対象 | 固定SHA256 |
| --- | --- |
| src/store.rs（本体修正後） | 767D92D5649D4B1B09E2ED5D2492F3327DA578AB861CD15ADFF767D65FEE5BBD |
| store作者prefix（変更なし） | D2E3225B03A398FC7B2275F49B45010B85F9E5BD5A3F0DD6B4FFBFF3F733C23A |
| src/commit.rs（変更なし） | 7D2720329860CEB3DCE1D2B1B8A5F85DED14EFBDE20FDF556173EF861CDBA95E |
| commit作者prefix（変更なし） | C624AC8332C4B37D1DF033A54A9F5C3E15870F54D57B9DECD3A27F2C63696D71 |
| src/ai.rs（変更なし） | 2AF7BE14882F9496BB1860179CF2B3D3FAC2F68DE39F1941DB056E1927925B6C |
| ai作者prefix（変更なし） | E609E5392402569B5E08F9170F5CA9C01EE7F5BEF49FAFE48BB3A2013D9977AD |
| tests/early_trial_paths.rs（変更なし） | 0CF6249630FACED5687206FBAF0C96378D5E2E6082E97D56E3DC05601B083E16 |
| Cargo.toml（変更なし） | 65FA8ECDFF905F73E0B7DCD94CF5465A7288C8EEC4E67B3CB24245FA0350B95E |

prefix境界は作者段階2/3どおり、先頭から本体の `use crate::{`（store/commit）または `use base64::Engine;`（ai）直前までのUTF-8 bytesである。

独立回帰候補は、既存private preflight後の実metadata差替えでcreate_rootを拒否させBeforeWritesを観測するケースと、実mkdirを成功させた同じchecked/stageへ差替えを行い、次の読取拒否でもStartedを保持するケースである。後者は単一production呼出し途中の競争再現とは区別する。テストの具体的な期待値と実装は独立作者が所有する。

残りは作者回帰、固定sourceのfocused GREEN、独立runnerの既存回帰とPostである。全体test/release/native/実API受入はET3等の既定gateに残る。今回の実データ/API/資格情報/プロセス起動/一時fixture作成/削除/commit/pushは0である。
