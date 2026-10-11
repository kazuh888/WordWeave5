# I1b-1 reader 実装引継ぎ

2026-10-07 / ACOUSTIC-HISTORY-001。担当は実装役、planner指定は `gpt-6-astra` / `high`。実行model/effort metadata、入力・cache・出力・推論の開始終了カウンターは未取得であり、使用量や差分を推計しない。親への返却前までの記録である。

## 正本と完了境界

[implementation.md](implementation.md) のI1モデル割当、[design-i1b.md](design-i1b.md) のreader契約、[design-i0.md](design-i0.md) の共有DTO、[reader-design-review.md](reader-design-review.md) の確認済み追補を使用した。AGENTS.md、開発7役手順、KPI、wordweave-changeを読んだ。UI変更はない。

完了範囲はI1b-1のreader本体とfocused自動検証である。書込・回復・Progress接続・backup・UI・実機・API受入は含まない。独立テスト作者の [audio_history_closure.rs](../../tests/audio_history_closure.rs) と期待値を変更していない。他担当のdirty差分を保持し、再委任・commit・pushを行っていない。

## 変更した本体

- [src/audio_history.rs](../../src/audio_history.rs): 公開reader、上限、固定本文とcaptureのhash helperを実装した。captureのJSON field順はdevice/distance/gain/environmentで固定し、確認時刻・versionを除外した。
- [src/audio_history/closure.rs](../../src/audio_history/closure.rs): FileBlobのbounded read、固定安全エラー、DTO検証、参照容量台帳、全旧revisionを含むstrict closureと32head部分閲覧を実装した。同hashを一度計上し、異なるbytes参照を拒否する。旧revisionは反復走査し、入力不変・正確なrevision差・Reference→Practiceのみの確認遷移を検査する。
- [src/audio_history/results.rs](../../src/audio_history/results.rs): 所属Run/result/reflectionの遅延読込、ID/入力hash/profile/rate/ch/frames/状態/引用の照合を実装した。原音欠損だけでは既存結果を遮断しない。

readerはStorageを借用し、履歴専用AssetStoreのFileBlob読込だけを使う。filesystem rootからStorage祖先、audio-history、assets、leafまでsymlink/reparseを検査する。既存AssetStoreのString窓口から返る固定の整合性エラーだけを安全codeへ対応させ、OS本文は解釈・公開しない。AssetStore、Storage、Progress、manifestは変更していない。

strict closureは一件でも不正ならErr、部分Viewは局所診断を伴う閲覧専用である。readerの結果から保存完了を主張しない。全履歴の音声/Report本文を蓄積せず、個別処理後に解放する。原音復号・PCM hash再計算・process起動・送信は行わない。

## focused結果

`cargo test --test audio_history_closure --locked --offline` を実行した。最初のsandbox内実行はcompile成功、34件中1 passed / 33 failedであった。33件すべてがreader到達前のfixture `AssetStore::put` におけるWindows os error 5（testの62行）であり、本体の行動失敗とは扱わない。ログは [history-reader-implementation-focused.log](artifacts/history-reader-implementation-focused.log) である。build中にもhard-link不可のcache warningがあった。

同じcommand・同じ合成fixture・同じassertionを、承認レビューを経てsandbox外で再実行した。**34 passed / 0 failed / 0 ignored / 0 filtered、exit 0**、test実行7.76秒である。ログは [history-reader-implementation-focused-unrestricted.log](artifacts/history-reader-implementation-focused-unrestricted.log) である。既存アプリbinのdead-code warning4件は対象外として保持した。権限設定の変更、実学習データ、実音声、network/APIを使っていない。

対象3本の `rustfmt --edition 2021 --check` と `git diff --check` は成功した。共有DTO本体は未変更のため、既存DTO35件の再実行を本担当では追加していない。最終検証はtest runner、受入は利用者の責任である。

固定したSHA-256:

| 対象 | SHA-256 |
| --- | --- |
| src/audio_history.rs | C30A7EA136CE94AA3B423F09A030D9EEC37A8C7B81BC54A4BA044A6106E0CCD0 |
| src/audio_history/closure.rs | 44D8F94AE6D8D56C9743803D51F00DFEFADF11F577B8866ABFC001463192432D |
| src/audio_history/results.rs | 7BC153FF0B9288E5FD258B948E1686A7E1657FBB8104E473AB78044CC3516714 |
| tests/audio_history_closure.rs（未変更） | 309C8F92C1D49CFF294E14B8BA1402D898451D50BFD04460C2BD0182936079CA |

## 残る確認

I1b-1の既存34ケースに未達はない。保存境界の独立レビューとrunnerによる最終検証は未実施である。現行独立テストにはsymlink/reparse専用ケースがなく、親へStorage祖先を含む隔離fixture追加の確認候補を伝えた。AssetStoreの固定Stringエラーへの対応は既存APIの制約であり、将来その文言を変更する場合はadapterも点検する必要がある。

書込の不変publish・quota台帳・intent/recovery、Progress commit、復元、二host適合、native操作、実AIは後続単位のままである。本体3ファイルの所有を親へ返し、独立レビュー前にwriter統合を行わない。
