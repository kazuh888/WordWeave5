# I1b-2b 境界受入の補完packet

2026-10-08 / R1・R2 / Pre-review提出。branch `codex/development-agent-team`、HEAD `b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d`。正本は[design-i1b.md](design-i1b.md) W05・回復列挙契約、指摘全文は[review](history-recovery-review.md)である。仕様改訂ではなく不足検証の補完である。

## 目的・範囲・所有

- production変更なし。独立作者は `tests/audio_history_recovery.rs` への2試験追加と `history-recovery-tests.md` 追補、`artifacts/history-recovery-boundary-*` のみ所有する。既存32件/writer37/readerの期待値は変更しない。役割モデルはI1のAstra/high、独立runnerはSol/mediumを維持する。
- 新test現hash `120C9F31BF66ECD35A9C4E0B2D4FA42907F2D6E5093F80418A519D9CCC08909F`、本体4本hashは実装記録stage2を固定する。store.rs resumeのintent.encode、recovery.rs operationsのENTRY_CAPが現consumerである。前者はPrepareOutcome/HistoryError、後者はRecoveryView/diagnosticsへ結果を返す。
- 合成隔離rootだけを使用する。実データ/API/UAC/OS変更/manifest/公開は対象外。安全性試験や旧PASSの要件を緩めない。旧契約の削除はない。

## 唯一のoracle・失敗後state

1. R1：手製Createのreferenceにquote/backslash/newlineを入れ、exact mutationの末尾合法JSON whitespaceで長さを調整する。独立fixture `Plan::create_on` から最新head用intentを組み立て、writerを呼ぶ前にserialize結果が524,287/524,288/524,289 bytesであることをassertする。old base=Noneの旧intentは全ケース上限内とし、全旧objectsを配置してから別trialのcurrent rootへrebaseする。前2はresume Ready、置換intentのexact bytes/長さ、publish後の独立root/Trial/receiptを確認する。超過はLimitExceeded、旧intentを含むtree全bytes不変、再試行でも同一拒否と不変を要求する。許可ケースでも旧objectsは全て不変である。
2. R2：実recovery dirに有効OperationId名を100,000個配置する。先頭/末尾各32は手製正常intent、中央99,936は2-byte `{}` とする。これは全名前走査の上限試験であり、未選択本文の妥当性を証明したとは扱わない。最初と最終pageの32正常候補/順序/diagnostic空/カーソルをassertする。100,001番目追加後はLimitExceeded diagnosticを必須とし、空成功と区別する。各phaseの全tree bytesとassets sentinelを比較して無書込を確認する。test seamによる上限縮小はしない。

## 実行・停止・rollback

- Pre PASS後にtest作者が追加・compile・focused自己検証を行う。既存実装でGREENならそのまま記録し、人工的なREDを作らない。compile/fixture不備と製品挙動の失敗を区別する。製品欠陥が判明した場合は本体を触らず別修正packetへ戻す。
- 100k試験は実fileの作成・全走査・全bytes比較・隔離root cleanupの費用を要する。build15分、単独R2実行20分は外部の有限打切りであって製品性能の合格基準ではない。超過時は未検証として残し、上限を黙って延長しない。ツール呼出しは短くyieldし親が進捗を報告する。
- 作成先は既存テストの専用temp方式を使い、そのrootだけをcleanupする。cleanup不成功や打切りの残存rootは記録し、広域削除しない。rollbackは今回追加した試験差分だけを戻すことであり、既存dirty/source/旧証拠を変更しない。
- 本体不変なら既存writer/readerの新しい独立runnerは固定済みEXEを使う。新testはbuild後hashを固定する。新34件とwriter37/reader37・1ignoredの独立実行、変更testのPost-reviewでClosedを判定する。権限依存1件を今回PASSへ算入しない。

## Pre提出・依存

独立作者のPREPはsource/testの現hash一致と既存fixture/APIの成立を確認した。親は対象2consumerを現物照合した。production差分0、test編集未開始。未決Product decisionはない。I1b-2b Postは未達、I2本体は開始しない。今回のPre判定は不足oracleの補完範囲に限定する。独立Post後にR1/R2 closureと現在要約を更新する。
