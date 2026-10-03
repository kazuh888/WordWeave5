# U5 通知可読性の再改善

2026-09-28。ユーザーが構造化通知・引用Markdown/原文切替・全文・折畳み技術情報の実装を承認。既存未コミット変更を保護し、公開/実AI/実データ試験は行わない。

## 開始境界

HEAD d86e7e9d4fe4497d8be9040d30a078582f2887d8。以下はU5編集前SHA256であり、HEADとの差分全体を今回成果としない。

| ファイル | SHA256 |
| --- | --- |
| src/material.rs | 333D6A293DBE3DB0CA4F4BE48E2B88D42069A3CE9799812E1A248EB79259B9AD |
| src/ai.rs | 3E508BAB8631803B3AC43263C0874083BC374E50E7EE99DB183862290FA15D3D |
| src/app.rs | 1B0B8FA139C82FCAA99B769169D364EACC5D33580D17569B7ECDA91F16088851 |
| src/app/notifications.rs | 047CB1177A7D6EC00AB04C4951A70E541DBA889B0D92DDB7EBA5BFAF082C8886 |
| src/app/visual_check.rs | D3422F7601DDBD6589AEC6DA8BC98139841100F7BE7B0B4321CA8C9BEC3FBA4D |

計画者Astra/high、仕様Astra/highを明示指定。残る役の指定はnotice-plan.md。実行model metadataは未取得。親設定変更なし。開始時wordweave5起動は検出されなかった。

## 仕様確認

- notice-spec.md N01–N08を親承認。初期要約/比較・技術情報折畳み、一括Markdown原文切替、診断copy抜粋と明示全文copy、通常通知維持を採用。ユーザーの承認済み提案の具体化であり、再承認要求はしない。
- notice_readability_review（gpt-6-sol/high指定）が独立read-only確認し、仕様阻害指摘0。旧Q03/Q05を構造化通知だけで上書きし、固定元発言・厳密検査・登録承認と一般通知を維持。Windows表示/コピー一致は後続検証であり、仕様成功だけで完了としない。

## 設計確認

- notice-design.mdの型/API・一時state・コピー契約を親確認、plannerがnotice-plan.mdを確定した。既存String APIは同一検査のwrapperとして維持し、教材専用typed経路だけ追加する。
- 同reviewerが独立設計確認、阻害0。既存のmessage直接代入について、現通知との対応・別通知失効が実装で守られるかを局所回帰で確認する。テスト担当Sol/highへ新integration fileのみを割り当て、本体所有と分離した。

## U5a/b の途中確認

- test authorがtyped診断8件＋旧String互換1件を固定した。旧Stringへ全文を期待する試行は240字切断の制約を実測したが、旧API互換要件と矛盾するため恒久RED証拠にはしない。typed API未実装のcompile-failも行動REDに数えない。
- U5a担当本体4ファイルに実装し、独立診断9/9成功と担当報告。これは型境界の証拠であり、tick→通知/実描画の受入は後段独立UItestを待つ。
- 暫定reviewでmessage直接代入後の古いpayload残留を発見し修正。reviewer再確認で解消。font_noticeが構造化本文から隠れる指摘はU5cで修正予定、未解消として管理する。
- U5b保存案の明示詳細操作を追加、U5cへGO。関連するtyped判定/登録可否は維持する。全本体freeze後に同居cfg(test)/native fixtureを独立担当が追補する。

## 本体freeze・独立UI追補中

- U5b bin material 15/15、U5c bin notification 19/19成功と担当報告。限定renderer、Raw切替、両全文コピー、技術情報折畳み、font案内を実装。
- 親がMarkdownの16pt固定fallbackを発見し、Body.resolveへ修正。通知19/19を再確認。その後見出し/本文分離と比較の区切りを本体blockだけ修正し、以後のcompile/nativeは独立追補とまとめて実施する。
- reviewerが型/検査順序/保存案非登録/外部取得なし/原文copyを局所確認。旧payload/font案内の指摘解消、追加の確定阻害なし。ただしUI試験/native未取得のため最終判定保留。
- 独立test authorは通知・renderer cfg(test)、合成visual_check hooksのみ編集中。本体とテストの所有を分離し、期待値を本体へ合わせて弱めない。

## 独立UI確認・最終freeze

- 通知6件/Markdown2件を独立追加。初回4/8、フォント未登録と初回paint/wheel到達条件を修正して7/8、最後のcopy操作をscroll安定後クリックへ修正し単独1/1成功。原文完全一致・通知state・データ不変・末尾/固定footerの期待を維持した。総合結果は後段alltargetsを正とする。
- 標準画面でも初期240ptでは要約/比較操作が埋もれる点を親が本体UX問題として修正依頼。構造化通知専用IDと初期540pt（viewport clamp）へ変更、一般通知240ptと利用者resizeは維持。毎frame強制resizeなし。
- 全source/test/native hooksをfreeze。runner Terra/mediumへalltargets→debug→合成native8状態→release、reviewer Sol/highへ最終局所確認を明示依頼。実行metadataは未取得。実データ/実AI試験は行わない。

## native後の局所修正

- 最初のrunnerはexec_commandのoutputだけを出力しsession_idを落としたため、実行中を完了と誤認した。ホスト異常/孤児化という初期推測は撤回。全戻り値とwrite_stdin待機へ訂正し、既存プロセスをkillせず終了確認。修正前alltargets再実行の実exit0、360件成功を確認した。重複実行の管理手戻りとして記録する。
- 最初のnativeでタイトルのHeadingがstatusサイズとなりBodyより小さいこと、および承認案のfooter閉じるが欠けることを親が発見。structuredのみ24pt strong/selectableタイトルと固定footer閉じるを追加。独立回帰を1件追加し、reviewerは局所阻害0。最終compile/実行はrunnerへ渡した。
- 旧compare/raw撮影は説明欄まで、end撮影は長文途中で、本文の装飾/末尾の証拠に不十分だった。合成fixtureの長さを変えず撮影位置を実wheelで調整、末尾marker可視assertを追加。最終撮影で実際の内容を確認する。
- 最新source/testfreeze後の最終runnerを開始。以前の360成功/PNGを最終差分の証拠へ転用しない。結果はnotice-runner-results.mdを正とする。

## footer回帰の修正

- title/閉じる追加後のalltargetsは実exit101、bin178成功/3失敗。structured footerのcopy/close/長文時固定表示が不可視になった。runnerは失敗で停止しrelease未実施。
- author独立確認では同じhelperで一般通知footerは成功、変更前structuredも成功しており本体回帰と判定。実装担当はbottom_up直下のhorizontal導入を原因とし、2ボタンを直接縦配置へ変更した。テスト期待値は変更なし。
- 変更後 `cargo test --locked --bin wordweave5 u5_` は9/9成功（3失敗含む）、diffcheck成功。全source再freezeしrunnerへ最終再実行を依頼した。

## 完了

- 最終alltargets361件成功、debug/release実exit0。最新native8状態は実exit0/process終了。親はsummary標準/狭幅、compare標準、raw標準、end狭幅を実画像で確認し、見出し・通常改行・箇条書き/原文の区別・固定copy/close・原文末尾到達を確認した。絵文字は使用フォントによって代替glyphとなるため全Unicodeの視認は保証しないが、原文コピーは独立テストで一致を確認した。
- releaseはユーザー起動中を検出して一旦止め、ユーザーの「閉じました」後に起動0を確認してbuild。EXE 2026-09-28 23:16:07 JST、11,598,336 bytes、SHA256 `3674873D6CE550127CFE0FE5DDD107781D7965E3BB557A42E8E7E2664EFDDB76`。アプリを強制終了/再起動していない。
- 詳細ログ/実終了コード/画像はnotice-runner-results.md。実AI/実データを使う利用者受入は未実施であり、次にアプリで確認する。公開/commit/pushなし。既存編集と学習データを保持した。
