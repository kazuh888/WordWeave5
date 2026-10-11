# I1b-2a writer 実装・検証記録

2026-10-07 / ACOUSTIC-HISTORY-001。planner指定は `ww-implementer / gpt-6-astra / high` を維持した。実行metadataと使用量の開始/終了counterは未取得であり、使用量を推測しない。AGENTS.md、開発7役手順、wordweave-change、test-driven-development、承認済みspec/design-i1bとwriter設計レビューを読了した。KPI手順は同一作業IDの先行reader単位から継続する。

## 実装した範囲

I1b-2aのprepare/publish、FrozenHistoryRequest、全mutation適用、fresh-head/receipt/pending照合、容量台帳、intent先行保存、fault seamを実装した。Saved判定・Progress接続・backup・UI・decoder・resume/recover/confirm/finishを提供していない。

- [store.rs](../../src/audio_history/store.rs): 一度固定したmutation JSONとpayload Arc、session限定Prepared、最新正本とreceipt優先の照合、exact quota、公開順序、公開後strict closureを実装した。途中公開後の再試行では現台帳へ未公開objectだけを加える。request/preparedのDebugはhash/ID/件数に限定し、原音・固定本文・助言payloadを含めない。
- [mutation.rs](../../src/audio_history/mutation.rs): 親から追加所有を受けた変更計画専用moduleである。重複の全ID集合、Active/Trash境界、Running/Sendingから終端への遷移と固定field、RunId再利用、入力/結果/根拠照合、旧版・receipt保持、checked revision/sequenceを扱う。計画objectは依存順のexact bytesであり、任意JSON patchは受けない。
- [recovery.rs](../../src/audio_history/recovery.rs): 512KiB intent codec/保存/読戻し、安全path、最大100,000エントリの保持台帳を実装した。filesystem祖先・history/assets/recoveryのsymlink/reparseを拒否する。orphan/旧root/trash/intent/tempを容量へ含め、自動GCしない。
- [audio_history.rs](../../src/audio_history.rs): mutation/recovery moduleの接合だけを追加した。

親所有の [results.rs](../../src/audio_history/results.rs) は既存result関係照合をpub(super) helperへ抽出した。writerはこの同じpredicateをintent保存前にも利用する。本担当はreader本体・shared DTO・manifest・Storage・Progress・独立テストを変更していない。

親との境界確認により、I1b-2aの既存intent読取はstrict schema/operation名/exact request hash/mutation DTO/参照上限・重複・planned-root末尾/実長の検査までとする。未公開payloadがないintentも保持量に含めるが、そのintentからPreparedを復元しない。同operationはBusy、異requestはConflictである。既存intentのbaseからの計画再計算・失われたpayloadとの照合・resume/recovery分類はI1b-2bの必須gateへ残す。自session publishは確定planに結んだintent全文をdiskのexact bytesと照合する。

## 初回focused結果とfixture障害

独立テストのREDは作者記録の35件中2fixture PASS / 33 scaffold TODO REDを受領した。新規テスト・assertionの変更は行っていない。

`cargo test --locked --offline --test audio_history_store` の通常sandbox実行はcompile成功、**15 passed / 20 failed / 0 ignored、exit 101**であった。[ログ](artifacts/history-writer-implementation-focused.log)。19件はintent保存時のIoまたはintent未作成に続くassertion、残り1件は下記のfixture障害である。

同一source・同一test SHA・同一commandを承認レビュー付きscoped sandbox releaseで再実行した。管理者/UACや設定変更は行っていない。結果は **34 passed / 1 failed / 0 ignored / 0 filtered、exit 101、test 6.25秒**である。[ログ](artifacts/history-writer-implementation-unrestricted.log)。19件のIo等はこの実行で解消し、sandbox内だけの環境差であることを切り分けた。成功34件は各テストの末尾まで到達した。既存アプリbinのdead-code warning4件は変更していない。

初回の残る失敗は `w06_stale_base_target_pending_and_unreadable_head_never_write` であった。`tests/audio_history_store.rs:311` のtree関数が、Storageのlock保持中にapp.lockをfs::readしてWindows os33でpanicした。最初のbefore snapshotで止まるため、この時点では本体assertionは未到達であった。親へ独立作者による「lock解放後の全file bytes比較→再lockして操作→解放後の全file bytes比較」の修正を依頼し、実装者がtestや期待値を変更しては通していない。

初回返却時点では35件GREENが未達であり、親の「35GREEN→reader回帰」の順序に従ってreader回帰を保留した。以下の追補がfixture修正後の現在の検証結果である。

4本体のtargeted `rustfmt --edition 2021 --config skip_children=true --check` と `git diff --check` は成功した。新本体にtodo/unimplemented/unwrap/expectはない。最終verificationはtest runner、受入は利用者に残す。

## fixture修正後の検証追補

独立test authorがW06内のStorage寿命だけを修正し、`app.lock`を含む全bytes比較と既存期待エラーを保持した。同じfocused writer commandの作者実行は **35 passed / 0 failed / 0 ignored / 0 filtered、exit 0、test 7.11秒**である。W06の4条件を含め全35件が末尾assertionまで到達した。[独立作者の記録](history-writer-tests.md)。本体hashは不変であり、親の指定により実装者はこの成功済みwriter実行を重複しなかった。

親が抽出したreader predicate接合に対する実装者のself回帰として、`cargo test --locked --offline --test audio_history_closure` を実行した。通常sandboxではcompile成功、1 passed / 36 failed / 1 ignored、exit 101であり、36件は合成fixtureのhardlink/junction作成に対するWindows error 5で止まった。[環境制約ログ](artifacts/history-writer-reader-regression.log)。この件数を本体の行動結果と混同しない。

同一target/同一assertionを承認レビュー付きscoped sandbox release・非管理者で実行した現在のreader回帰結果は **37 passed / 0 failed / 1 ignored / 0 filtered、exit 0、compile 1分30秒、test 6.61秒**である。[成功ログ](artifacts/history-writer-reader-regression-unrestricted.log)。`windows_path_safety::leaf_file_symlink_rejects_valid_original_and_preserves_target` は既存の明示的ignoredであり、今回実行していない。祖先・audio-history・assetsのjunction検査3件は成功した。path保護コードは今回変更していない。OS管理者権限、UAC、設定変更、実データ、network、live APIは使用していない。

追補作業では本書だけを編集し、production/testは変更していない。既知の同一targetのhardlink/junction制約が変わらない次回は、親承認に従い理由を明示した非管理者scoped sandbox releaseから開始できるが、検証対象・権限要件・品質gateを変更する意味ではない。

## 対象固定・所有返却

基準はHEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` と共有の未commit差分である。他担当の変更を保持し、再委任・commit/pushはしていない。実学習データ・資格情報・live AI・networkを使用せず、一意な合成fixture rootだけで検証した。

| 対象 | SHA-256 |
| --- | --- |
| src/audio_history.rs | 77158A276489D1C859A59EE026720F5F314EFE8B38D23908AC0911689831AD81 |
| src/audio_history/store.rs | 4454F6391EC318C6E47B5629C3D8A692E4EFEDF6C604FBA4E7975D7D8E049603 |
| src/audio_history/mutation.rs | 8F55A105C3F7A2BD336F215A6E0233772F13DCDEA89C79FC3A5FEE81AD52B4ED |
| src/audio_history/recovery.rs | 2D804117C21E2427CEA44FD796B173667E6BE59CB5179E3E283B07347A7FF55B |
| src/audio_history/results.rs（親接合） | B8E4E2F9C1C0A576E975D6948171731153166034FE24B88FBB7370B85CE1985B |
| tests/audio_history_store.rs（独立作者のfixture修正後、本担当未変更） | 5D097873DF335BA6D152CF7522879D5870A43C5E44E5B443A748A9545C89098A |

残る範囲は保存境界の独立review/runner、I1b-2b回復、I2の正本pointerと全writer直列化である。Windowsの実disk不足/電源断耐久性、全reparse階層、100,000件上限、実音声・実APIは今回のfocused成功から受入済みとしない。privileged leaf試験も今回の実行証拠に含めない。本体4ファイルの所有は親へ返却済みであり、今回のself検証完了は独立reviewや最終受入を代替しない。
