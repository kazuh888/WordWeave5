# I1b-2a 独立 writer テスト

2026-10-07 / ACOUSTIC-HISTORY-001。担当は test author、計画指定 `gpt-6-astra/high`。起動指定は親の引継ぎ、実行 metadata と使用量の開始/終了 counter は未取得である。テスト作成・RED・実装担当への引継ぎを完了境界とする。production、manifest、既存 reader tests は変更しない。

承認根拠は [implementation.md](implementation.md) の I1、[spec.md](spec.md) AH-AC02–04/08/12/14、[design-i1b.md](design-i1b.md) I1b-2a、[writer-design-review.md](writer-design-review.md) の阻止指摘0である。AGENTS、開発7役規約、KPI、wordweave-change、agent-skills:test-driven-development を読了し、TDD のテスト/RED工程だけを担当した。

## 所有・fixture・期待値

所有は `tests/audio_history_store.rs`、本記録、`artifacts/history-writer-red*.log` である。全fixtureは一意な `ww5-writer-tests-<pid>-<time>-<counter>` 一時ディレクトリに生成し、Storage lock解放後にそのディレクトリだけを削除する。実学習データ、資格情報、外部API、既定保存rootは使用しない。

原音は手製の1 frame/8kHz/mono PCM16 WAVである。既存 root/Trial/Run/result/reflection は手製DTOをserializeし、hash名の不変objectを直接配置する。writerによるfixture生成は行わない。fixture-only test は DTO検証と既存 reader の閉包検証を実行する。writerの正常出力には Trial全文、rootのrevision/sequence/全head/receipt、旧bytesの保持を独立assertする。原音サイズ境界でWAVへ追加するbytesは保存層の容量fixtureであり、codec検査の受理を意味しない。

quota のoracleは手製root/Trial/Run/RecoveryIntentのserialize bytesである。writerの `quota_plan()` を上限設定の根拠に使わない。intentのkey順はserialize長へ影響しないが、埋め込むmutation_jsonはexact bytesとし、二重escapeを含む524,287/524,288/524,289 bytesを構成する。

## 受入対応

| ケース | AC | 今回の観測 |
| --- | --- | --- |
| W01 | 02–03 | exact request hash、重複Arc、未参照/不一致payload拒否、固定元path変更後の原音保持、payload不足、6MiB一致/1byte超過 |
| W02 | 04 | Before/AfterIntent、各3objectのBefore/After、Before/AfterClosureの10境界。intent確認前原音0、公開済object数、caller Arc/旧bytes、正本不変 |
| W03 | 04/08 | 同Preparedの部分publish後retryと再publish、上限ちょうどの追加量再計算、別session token拒否、同intent Busy/異request Conflict、後続root receipt・古いbase・異request/affected |
| W04 | 08 | 全mutation、既存map/reflection保持、Running→Partial/Failed/Cancelled/TimedOut/Interrupted、Sending→Assessed/Interrupted、結果不足、終端再書込/immutable field改変/別種RunId再利用拒否 |
| W05 | 14 | 独立object/intent実長、quota上限直前/一致/直後、保持台帳にtrash/orphan/temp、整数80%警告/大きい上限、上限低下、0設定、sequence overflow、escaped intent 512KiB境界 |
| W06 | 04/14 | stale base/target、head読取不能、prepare後head/pending変更、原音公開後pending発生、Storage二重lock拒否、既存object破損、recovery directory偽装時のbyte保持 |
| W07 | 12/14–15交差 | Active/Trash/Referenceの重複集合、古い確認集合、Reference→Practice、Running/Sending中trash Busy、停止終端のtrash/復元、Trashへの後続変更拒否 |

fault seamは実際のローカルfile保存に固定Ioまたは合成pending発生を注入する。通常のerror復帰を観測しており、電源断・実disk満杯・実ACL拒否の再現ではない。新しい権限依存symlink testやignoreは追加していない。

## 実行証拠

初回 `cargo test --locked --offline --test audio_history_store` は compile成功、26 tests、2 PASS / 24 FAIL / 0 ignored、exit 101である。FAILは `src/audio_history/store.rs` の未実装 `todo!` panicであり、fixture compile errorではない。新機能未実装を示すREDであり、既報製品バグを再現したという主張ではない。[初回ログ](artifacts/history-writer-red.log)。

最終版も同コマンドでcompile成功、35 tests、2 PASS / 33 FAIL / 0 ignored、exit 101である。compile 1m57s、test実行0.11s。33件すべての失敗箇所は `src/audio_history/store.rs:95` または `:203` の未実装入口であり、fixture assert/compile error/OS権限エラーではない。[最終ログ](artifacts/history-writer-red-final.log)。sandboxのincremental cache hardlink警告はあるがcopy fallbackでcompile成功した。sandbox外実行、管理者権限、UACは使用していない。

対象HEADは `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d` ＋既存dirty差分である。固定したテストSHA-256は `DDAABDAD38CB6AD86F6DAE0C98395EBD3AFA93B474B39B04AB08B4E234564685`、RED時のwriter scaffold SHA-256は `0FE4CEF8D0366BED5DA36FE0DBE606B03FE37E8B5089CF59D57B14D3AAA6AF34`。担当差分の `git diff --check` は成功した。テスト/記録の所有を親へ解放し、production実装へ引き渡す。

## 引継ぎ・未検証

実装者/runnerは `cargo test --locked --offline --test audio_history_store` を実行する。reader回帰 `cargo test --locked --offline --test audio_history_closure` と全体delivery gateは親の計画に従う。テストauthorは今回これらを実行済みとは報告しない。`rustfmt --edition 2021 tests/audio_history_store.rs` で担当fileだけを整形した。

I1b-2bのresume/recover/confirm/finish、原音だけからの再起動関連付け、intent置換差分・未知field/版・回復pagination、commit後cleanup障害は今回API外である。100,000走査上限、root revision overflow、全immutable fieldの個別変異、全正常advice/Complete分析状態、危険path/reparseの全階層、OS実disk/電源断も追加の証拠を要する。I2の正本pointer保存、I6のPOST/解析増分0、codec/実音声/UIは本試験から受入済みとしない。

RED段階では未実装入口で止まるため、各test内の後続assertを実行した証拠はまだない。fixture/期待値への疑義が出たら仕様・設計の根拠とともに作者へ返し、assertを弱めてGREENにしない。

## Windows排他lockのfixture修正

2026-10-07。親から独立test authorの再割当を受け、既存 `ww-test-author / gpt-6-astra / high` を維持した。今回の所有は `tests/audio_history_store.rs` と本書だけである。AGENTS、wordweave-change、TDD、設計のW06/AH-AC04/14を参照した。実行model/effortと使用量counterは未取得である。

実装後の[既存35件実行ログ](artifacts/history-writer-implementation-unrestricted.log)は34 PASS / 1 FAILであり、唯一の失敗は `w06_stale_base_target_pending_and_unreadable_head_never_write` のfixture `tree` が保持中の `app.lock` を `fs::read` したWindows error 33であった。writerの期待エラーassertに到達する前のfixture失敗であり、本体の行動REDではない。変更前test SHA-256は `DDAABDAD38CB6AD86F6DAE0C98395EBD3AFA93B474B39B04AB08B4E234564685` と照合した。

修正はこの1 test内のStorage寿命だけである。手製fixtureを準備後にStorageをdropし、`app.lock`を含む全tree bytesを取得する。操作用Storageを同じ隔離pathで再取得してwriterを呼び、そのscope終了後に全tree bytesを再取得して完全一致を要求する。stale base/targetはConflict、pendingはPendingMaterial、head読込不能はIoという既存oracleと、全原音・履歴・他保存データ・pending・lock bytesの不変assertを維持した。tree helperの読込errorを握り潰さず、ファイル除外も行わない。他34件、production、仕様、gateは変更していない。

通常sandboxのhardlink/rename制約が既知のため、同一合成targetの `cargo test --locked --offline --test audio_history_store` を承認レビュー後のsandbox外・非管理者で実行した。**35 passed / 0 failed / 0 ignored / 0 filtered、exit 0**、compile 1m09s、test実行7.11秒である。W06内の4条件すべてが既存の期待エラーと全bytes保持assertを通過した。OS設定、UAC、管理者build、実データ、外部APIは使用していない。

新実行の関連ログ（tool session 11192、出力chunk 56f005 / ef35ee）:

```text
running 35 tests
test w06_stale_base_target_pending_and_unreadable_head_never_write ... ok
test w02_fault_boundaries_preserve_bytes_and_publish_only_after_intent ... ok
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.11s
```

修正後test SHA-256は `5D097873DF335BA6D152CF7522879D5870A43C5E44E5B443A748A9545C89098A` である。`rustfmt --edition 2021 --check tests/audio_history_store.rs`、`git diff --check`、本書の相対リンク検査は成功した。本体の着手前後SHA-256は以下のとおり一致した。

| 本体（未変更） | SHA-256 |
| --- | --- |
| src/audio_history.rs | 77158A276489D1C859A59EE026720F5F314EFE8B38D23908AC0911689831AD81 |
| src/audio_history/store.rs | 4454F6391EC318C6E47B5629C3D8A692E4EFEDF6C604FBA4E7975D7D8E049603 |
| src/audio_history/recovery.rs | 2D804117C21E2427CEA44FD796B173667E6BE59CB5179E3E283B07347A7FF55B |
| src/audio_history/mutation.rs | 8F55A105C3F7A2BD336F215A6E0233772F13DCDEA89C79FC3A5FEE81AD52B4ED |

本体の挙動修正は不要であった。既存binのdead-code warning4件は担当外として保持した。新しいfixture・test・ignoreを追加しておらず、W01–W07の期待値を緩和していない。reader回帰・全体delivery・独立レビューは親の検証範囲に残し、この35件成功だけで代替しない。指定2ファイルの所有を親へ解放する。
