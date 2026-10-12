# 早期試用の仕様と設計レビュー

## 初回判定

2026-10-12、独立reviewer `export_destination_final_review`（指定 `gpt-6-astra/high`）は外部仕様をPASS、設計をCHANGES_REQUIREDと判定した。対象は `codex/development-agent-team`、HEAD `d82e3112584d6c98ca0ceaac0f75537870a40b57` 上の[外部契約と設計](early-trial-contract.md)、SHA256 `EFAE9275F6CFFC26C6003ADB99EBFBE87A26E16B8E6A2CEA138F0E0B48F7AFD8` である。初回snapshotの判定を、是正後の文書へ流用しない。

仕様の手動取込維持、Qwenキー削除機能の非追加、対象外と受入条件は明確である。固定feature、root/tree/pendingの事前検査、Storage::at維持、登録済み2rootとtempの再検査、Codex保存先、資格情報selector、履歴入力拒否も追跡できた。独立レビューは文書に限定し、実装やnative動作の合格ではない。

## 是正の追跡

| 指摘 | 原因と現物根拠 | 状態と担当 | 判定基準と残る範囲 |
| --- | --- | --- | --- |
| ET-D01 / P2 | 契約82–84行の一律「元データは変更せず」は、`commit::recover` が書込後の検査でErrとなる場合（commit.rs:417–424）や、回復後の初期化失敗で保証できない。書込前と回復開始後の失敗を同じ状態として扱っている | 設計上Closed。正本はearly-trial-contract.md設計節、担当はdesigner。初回CHANGES_REQUIREDからcycle 2独立Pre PASSへ是正 | 書込前の不変保証と、回復開始後の試用内変更可能性を分け、操作画面・通常rootへのfallbackを共に禁止する。実装と実行証拠は後続ET1/ET2であり未完了 |

## ET-D01の限定修正

目的は起動失敗時に試用データの状態を誤って伝えないことである。対象は設計節の失敗型・表示・受入証拠だけとし、外部仕様、回復アルゴリズム、保存形式、通常版の動作、公開範囲は変更しない。canonical ownerはdesigner、consumerはET1の事前検査とET2の起動factory／表示、入力は失敗段階を識別できる結果、出力は操作画面へ進まない安全な通知である。raw errorの文言解析を状態判定に使わない。

書込前preflight拒否では不変を示せるが、回復・初期化開始後は試用内に適用済み／途中の変更が残り得る。秘密・原音・教材本文は通知へ含めず、どちらも通常rootへ切り替えない。是正は独立再レビュー前に文書へ反映し、source実装は修正版Pre PASSと計画確定まで着手しない。scope、owner、保存契約に変更が必要なら停止して親へ返す。

後続の決定的oracleは、合成pendingの書込後失敗、回復後初期化失敗、書込前拒否の3状態で、通知の不変断言の可否、操作画面開始0、fallback0、各時点のledgerを照合することである。今回の文書確認では旧一律文言の除去、唯一の失敗契約、受入追跡と差分を検査する。rollbackは未実装設計案の撤回だけであり、実データの巻戻しは行わない。初回レビューと是正記録を保持する。

## 修正後の判定

同日のcycle 2で、独立reviewerは限定設計Pre PASS・残指摘0と判定した。対象HEADは同じ `d82e311`、是正後文書SHA256は `0BC3839A6C6B60E5CD4E7267269FE33D4D898CA704B89609E32E0BB0912A1A6D` である。BeforeWritesだけに不変保証を限定し、初期化・回復開始後は変更可能性を明示した。発生箇所で型付けし、raw errorの文字列解析を使わない。3状態oracleとledger・表示・操作画面開始0・fallback0の対応を確認した。公開API、回復手順、各5file所有は維持されている。

ET-D01は文書設計上Closedであり、ET0は初回の仕様PASSと合わせ仕様／設計ともPASSである。新仕様と設計を実装済み・実行確認済みとは扱わない。計画確定とd-B/M完了後にET1へ進む。レビューのsource編集・試験実行・再委任はなく、実行metadata・usage counterは未取得である。
