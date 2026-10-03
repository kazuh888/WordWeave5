# チャットの再利用境界

2026-10-02 / REUSE-DESIGN-001。[全体設計](README.md)と[文書受入仕様](../../tasks/REUSE-DESIGN-001/spec.md)に基づく設計文書である。
「現行」は未コミット変更を含むソースの確認、「proposed」は未実装の目標契約である。製品挙動や新APIの承認・実行確認を意味しない。

## 1. 用途と対象外

再利用対象は会話、下書き、添付参照、送信文脈の選択、実行と結果の対応である。会話の保存と外部生成はホストが接続する。
教材一覧の検索、教材操作の解釈、教材登録/追加/訂正/削除、学習記録、課金・日次上限は共通chat-coreの外に置く。
providerを内蔵する汎用チャット画面や、任意のAIを同一能力と見なすAPIは作らない。
文字の会話と音読評価は別能力である。[Codex client](codex-app-server.md)をホストの生成adapterとして接続し、Qwenへ暗黙転送しない。

## 2. 現行の責任とcoupling

| 現行symbol・ソース | 現行の責任 | 目標の境界 |
| --- | --- | --- |
| [Conversation / Exchange / Attachment](../../src/chat.rs) | 会話、draft、往復、`AssetRef`、`Execution`、教材用flag | 会話と参照の中立型へ絞る。教材用flagはホストmetadataに置く |
| [prepare / prepare_selection / Context::preview](../../src/chat.rs) | 必須pin、最近の往復、語彙一致による文脈選択、省略情報 | 純粋な文脈選択をcoreに残す。保存済み原文は変更しない |
| [prepare_with_catalog](../../src/chat.rs) | `model::Entry`と削除教材集合から有界catalogを作る | WordWeave adapterへ移す。中立的な追加文脈と予算だけをcoreへ渡す |
| [ChatReply / Action / parse](../../src/chat_action.rs) | 回答と教材操作案の構造検査 | 教材actionはホストで検査。chat-coreの成功結果から教材を書き換えない |
| [launch_chat_with_file_consent / ChatFileConsent::matches](../../src/app.rs) | 添付textの送信確認、質問/添付ID/nameとの照合、worker起動 | 送信前のホストcontroller。coreの準備snapshotと整合させる |
| [AssetStore::put/read](../../src/assets.rs)、[chat_media](../../src/app/chat_media.rs) | 原本保存・hash検査、画像変換、表示用preview、添付text抽出 | ホストasset adapterとmedia/UI。coreは原本パスを解釈しない |
| [Progress::complete_chat / Storage::save](../../src/store.rs) | 学習記録と会話を一緒に保存 | WordWeave repository adapter。独立hostにProgressを要求しない |

現行`Conversation::complete`は完了回答だけを往復へ追加し、質問とdraftが一致する場合にdraftを消す。
一方、添付は完了時の`draft_attachments`を使用して消すため、revisionで固定した送信添付のcommit契約は現行coreにはない。
`apply_recognition`には期待draftとの一致確認がある。この既存の競合防止を、非同期送信全体へ適用する設計を提案する。
現行画面は生成中の操作を制限するが、その画面制約だけを別hostに移したcoreの安全性としない。

## 3. 目標の公開型と操作（proposed）

論理名`chat-core`の仮称APIである。以下は実装済み署名やCargo利用例ではない。
型のconstructorで外部入力を検証し、意味が異なるIDを分ける。保存形式をそのまま公開型へ固定しない。

| 仮称の型・操作 | 入力・出力 | 契約 |
| --- | --- | --- |
| `ConversationId / ConversationRevision / DraftRevision / MessageId / RunId` | それぞれの識別子 | 表示順indexや別会話のIDと交換しない。revisionはホスト保存時にも照合 |
| `ConversationSnapshot` | title、memo、完了往復、draft、添付参照、revision | 読取snapshot。教材のEntry/Progressやfilesystem pathを含めない |
| `AttachmentRef / ResolvedAttachment` | 不変asset ID、種別、表示名、長さ、派生元関係 → 検証済み内容 | 読出し権限と実体整合性はasset adapter。送信可能性を単なる添付存在から推測しない |
| `ContextPolicy / ContextPlan` | 予算、必須往復、現在draft、明示追加文脈 → 選択・省略IDとbyte数 | 省略は送信上の選択であり、履歴削除や自動要約ではない |
| `prepare_send` | snapshot、解決済み添付、文脈計画、宛先/用途/受取先、接続世代 | `PreparedChatSend`。送信本文と添付・元revisionを固定し、送信しない |
| `start_send` | 準備と一致する人の許可、ホストの生成adapter | `RunHandle`。ホストがdraft/実行記録を保存後に一度開始 |
| `receive_event` | RunId、ConversationId、生成イベント | 未確定previewまたは検証済み`CompletedReply`。coreが直接HTTP/processを所有しない |
| `propose_commit` | 完了結果、現在snapshot | `CommitProposal`または競合。送信snapshotの質問/添付を結果と対にする |
| `confirm_saved` | 提案ID、ホストの保存成功と新revision | 保存済み表示へ進める。保存前表示は未保存と明示 |
| `apply_recognition` | 対象会話、期待DraftRevision、認識原文 | 一致時だけdraft更新案。不一致時は結果を保持して手動適用をホストへ返す |

タイトルの自動提案はホストpolicyに従い、利用者が手動で変更したタイトルを遅延応答で上書きしない。
引用は`MessageId + 固定原文snapshot + 範囲`等の中立参照を候補とし、描画後の文字列を正本にしない。
範囲の符号化と引用編集時の扱いは切出し仕様で確定する。今回、既存の教材引用仕様を変更しない。

### 検証・上限・エラー

現行 [Conversation::validate](../../src/chat.rs) はtitle 100、memo 2000、draft/質問4000、回答16000 Unicode scalar、200往復、発言添付8件を検査する。
現行contextは64,000 JSON bytes、active会話50件/trash500件である。会話集合と永続全体サイズは[Progress::validate](../../src/store.rs)の追加制約である。
[chat_media](../../src/app/chat_media.rs)の添付text合計32,000 bytes、[assets](../../src/assets.rs)の12MiB原本上限は、coreの64,000 bytesとは別である。
目標では現行互換のpolicyをWordWeave adapterに渡し、別hostの上限はその仕様で明示する。設定で無制限にしてよいという提案ではない。
必須pin・memo・現在質問だけで上限を超えた場合は停止し、黙って必須文脈を落とさない。予算計算にはfield名と追加文脈も含める。
構造エラーは`InvalidInput / ContextTooLarge / AttachmentUnavailable / RevisionConflict / Deleted / DuplicateRun / ProviderFailure / UnknownOutcome / SaveFailed`等の分類を返す提案である。
エラー本文から再送可否を決めない。本文/添付/応答の原文は診断ログへ自動記録しない。

## 4. 所有と依存方向

| データ・機能 | 所有者 | coreへ渡すもの |
| --- | --- | --- |
| 正本会話と保存revision | ホストrepository | 不変snapshotと保存成功/競合の結果 |
| 表示中draft・未保存応答 | ホストcontroller | draft revision、未保存状態。保存失敗でも画面上の内容を保持 |
| 原音・画像・ink・添付file | ホストasset store | hash等で固定した参照と送信が許可された実体 |
| 文脈選択と実行の関連 | core instance | 選択ID、省略ID、送信snapshot、RunId。画面選択indexを使わない |
| Codex process・取消し・exact回復 | ホスト生成adapter | providerの確認済み状態。共通coreは具体通信を知らない |
| 教材prompt/action・登録承認 | WordWeave adapter | 必要な追加文脈/表示用提案だけ。保存は別手順 |
| Markdown表示 | ホストと[Markdown部品](markdown.md) | 不変の本文。rendererから会話正本を書き換えない |

依存はhost → chat-core、host → generation/asset/repository adaptersである。core → WordApp/Entry/Progressは認めない。
同意はホストが人から得る。本文・添付・宛先・用途・受取先・接続選択が変わったら準備を破棄し再確認する。
生成adapterへは確認した実体だけを渡す。確認後に元パスを再読込して別内容を送信しない。
同一プロセスの悪意あるhostをこの型設計で排除できるとは主張しない。誤用防止と実際の人の同意は別の責任である。

## 5. 正常sequenceと永続化

通常は「draft保存 → 文脈/添付の解決・固定 → 送信内容/省略表示 → 人の許可 → 実行記録確保 → 生成 → 完了結果検査 → commit案 → 会話保存」である。
原本assetを安全に保存・照合してから、その参照を含むdraftを保存する。asset保存後の会話保存失敗は孤立参照を生み得るが、原本削除で帳尻を合わせない。
Codexのjournalは[接続設計](codex-app-server.md)のcheckpoint順を維持する。chat側の保存済み表示とは別の状態である。
完了後は送信時の質問/添付と応答を組にし、現在の会話が同じrevisionか確認して保存案を作る。
保存はhostが期待revision付きで行い、成功ack後に保存済みとする。書込み先がCASを持たなければhostが単一writer/lockで同等の競合検知を行う。
別writerからの変更をlast-write-winsで消さない。競合時は案を保留し、ホスト仕様で再適用または人の判断へ進める。
draftが送信時のrevisionと一致する場合だけ消去候補とする。変更済みdraft/添付は残し、送信添付を後から追加した添付で置換しない。
会話保存に失敗した場合は完了本文とcommit案を未保存として保持し、同じ内容の保存だけを再試行する。
アプリ終了・crashで未保存メモリ結果を必ず回復できるとはしない。保存済みCodex journalがあればexact回復し、再適用は別操作にする。

現行は[Progress::complete_chat](../../src/store.rs)がcloneで検査後にmemoryへ反映し、[app.rsのAiResult::Chat処理](../../src/app.rs)がpersistする。
目標のrevision付き保存・RunId単位の重複適用拒否・未保存結果型は、この現行方式へ加える設計案である。
会話と生成台帳の保存は単一transactionではないため、両方に同じRunIdを持たせ、再取得した結果の二重追加を拒否する。

## 6. 非同期・取消し・遅延結果

現行の会話追加は完了応答だけであり、provider内部の通知をそのまま会話履歴へcommitしない。
将来stream表示を入れる場合もpreviewは揮発状態とし、確定本文・教材提案・保存済み往復とは区別する。stream対応自体は今回未承認である。

| 場面 | 状態・適用条件 | ホストの回復 |
| --- | --- | --- |
| 送信前取消し | 未送信、draft/原本保持 | 新たな準備と確認から開始可能 |
| 送信後取消し/timeout | 取消要求済みかつ結果不明になり得る | [Codexのexact回復](codex-app-server.md)を提示。自動再送しない |
| 他会話へ移動中の結果 | RunIdとConversationIdの対象へ紐付ける | 現在選択中の会話へ誤追加しない |
| 同じdraftを編集中の結果 | 送信snapshotと現在revisionを比較 | 新draftを消さず、競合の完了結果は保留して見せる |
| 削除済み会話への遅延結果 | tombstoneを確認し追加拒否 | 結果を未適用で保持。復元/適用の判断をホストに返す |
| 同じrunの重複・回復応答 | 適用済RunIdなら新しい往復を作らない | 既存の適用状態を表示 |
| 生成成功・保存失敗 | 完了かつ未保存 | 本文/添付参照/新draftを保持し、保存のみ再試行 |
| 応答不正 | 往復にcommitしない | 下書きを残す。再生成は別の人の操作 |

確認済中断はproviderがexact対象の中断を返した場合だけであり、local cancelやworker停止から作らない。
生成/認識workerは自分の会話・run・revisionを保持し、host終了時に取消しと回収を行う。coreにglobalの「現在の会話」を持たせない。
同一会話で複数runを許すかは切出し仕様の未決事項である。初期案は一会話一runで、同時送信をbusyとして拒否する。

## 7. 履歴・教材・画面

文脈省略の件数と対象を送信前に示す。省略した教材の不存在や、会話全体をAIが読んだことを推定しない。
過去添付の説明を送ることと原音/画像を再送することを分ける。原本の再送には解決・能力検査・許可が必要である。
会話削除は現行`deleted_at`の回復可能な非表示を保持し、原音・添付・ink・学習記録を自動削除しない。GCや完全削除は別仕様である。
教材登録はchatの保存と別であり、案 → 可視差分 → 学習者承認 → 保存をWordWeave adapterが担う。追加は既存内容/reviewを保ち、訂正と混同しない。
教材actionを受信してもcoreが実行しない。独立hostは教材action機能を一切実装せず会話機能を利用できる。
引用/コピー用の固定原文とMarkdownの表示結果を分け、表や装飾を描画した都合で原文を保存し直さない。

UIはhost責任である。WordWeaveは[Button / UiControls](../../src/app/controls.rs)を使い、日本語の可視字形を両軸で中央に置く。
長い日本語、空文字、折返し、disabled、focus、keyboard操作、複数scaleで確認する。アイコン+文字群に文字専用の補正を無条件適用しない。
狭幅と拡大では会話本文をscrollし、draft・送信/取消し・未保存/結果不明の状態へ到達可能にする。header/footerの縦占有も検査する。
通知詳細はhoverだけにせずclick/tapでも開けるようにする。IME・Windows DPI・screen readerは実機受入を別に残す。

## 8. 互換性と二消費者での受入

まず現行`Conversation`保存形式をWordWeave adapterで読み、coreの中立型へ変換する。公開型変更だけで既存progress/assetを移行しない。
`for_material`等の既存fieldは旧保存データで保持し、coreへ渡さないmetadataとして管理する。未知fieldの脱落による破壊に注意する。
公開API版、会話保存schema、provider protocol、egui renderer版を別に扱い、旧data fixtureのround-tripで意味保持を確認する。
失敗時は旧adapterへ戻せる段階で導入し、保存schemaを変える場合はbackupと検証済みmigrationを先に用意する。downgradeで原本を消さない。

以下はproposed手順であり、コンパイル可能なAPI例ではない。

```text
WordWeave: 会話snapshot + 教材catalog adapter → ContextPlan → prepare_send
           → 添付/省略を確認 → 人の許可 → Codex adapter → CommitProposal
           → Progress adapterが保存 → 必要な教材案は別の差分承認へ
独立host: 自分の会話snapshot → 同じ版のContextPlan/prepare_send
          → 人の許可 → 独自保存rootのCodex adapter → 同じ版のCommitProposal
          → 独自repositoryが保存（教材型なし）
```

第二候補は教材を持たないノート支援hostである。試験用最小hostはAPI確認用であり、二製品での実利用実績とは区別する。
同じchat-core版で文脈選択/省略、上限直前/一致/超過、日本語byte計算、pin過多、添付不足/破損を両hostから観測する。
偽生成adapterの遅延/重複/取消応答と偽repositoryの保存失敗/競合を使い、誤会話追加0件、新draft消去0件、再送0回を確認する。
削除/復元後の原本hash不変、保存失敗後の未保存応答表示、同じRunIdの再適用0件、添付許可不一致で送信0回も合格条件である。
時刻/ID供給、asset resolver、生成adapter、repository、UI rendererを必要な試験口とする。全関数への抽象化は不要である。

## Framework観点の補足（2026-10-03）

[Framework](framework.md)の学習試行とConversationは別の集約である。チャットだけの利用に問題・成績・復習を要求せず、学習だけの利用にも会話を要求しない。
試行から会話を参照する場合はhostが試行IDと固定MessageId/原文版を対応付ける。画面の選択行・最新の会話・表示後Markdownを出典の代わりにしない。遅延回答は送信時の会話/試行へ対応させる。
教科ごとの教材catalog、依頼prompt、回答の意味検査、教材案→差分→承認は教科/host adapterの責任である。共通chat-coreが成績や教材を確定しない。
添付は[原本管理](asset-store.md)の参照として扱えるが、原本storeの導入は必須ではない。chatからの参照解除は原本削除でも、他の試行からの参照削除でもない。
既存の送信固定・同意・revision・競合・保存失敗の契約は変更不要である。将来の学習coreは別の型であり、現行Progressを共通chat保存形式へ格上げするものではない。今回の証拠は[追補結果](../../tasks/FRAMEWORK-DESIGN-001/results.md)に記録する。

## 9. AC追跡と引継ぎ

| AC | この文書の対応 | 実装/試験作者へ渡す焦点 |
| --- | --- | --- |
| AC01–02 | 1–4節、現行symbol参照 | 中立型、Entry分離、文脈/添付の所有 |
| AC03–04 | 4–7節 | 同意、revision、遅延結果、保存失敗、教材承認 |
| AC05–06 | 8節 | 保存形式保持、二adapter、故障注入と復帰 |
| AC07 | 本表と[今回の記録](../../tasks/REUSE-DESIGN-001/results.md) | 親のリンク検査・独立設計レビュー |

未決は第二製品、署名、revision導入時の保存表現、競合結果の操作UI、引用範囲の符号化、同時run policy、stream対応である。
これらは切出し仕様の承認者へ戻す。今回は実装・test編集・実生成・表示実機確認を行っておらず、文書レビューだけで再利用達成とはしない。
