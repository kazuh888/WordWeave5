# FEEDBACK-UI-001 U7 教材登録詳細の暫定計画

2026-09-30。状態: **暫定作成後、既存U6仕様・設計と親の追加判断を反映して確定。モデル割当は親承認済み、親dispatchへ引渡し**。既存契約を再利用し、全面仕様/設計の再作成は不要である。計画者は本書だけを所有し、作業リストも本書に置く。既存未完了 `tasks/plan.md` / `tasks/todo.md` とU1–U6文書、dirtyな本体・テストを保持する。実装・テスト編集・再委任・commit/push/公開は行わない。

## 要求・境界と既存契約

教材登録の「生成時の固定会話」内AI回答を既存 `notifications::show_markdown` で読みやすく表示し、左変更項目一覧と右詳細を独立縦スクロールにする。header/footerは固定する。可視差分・登録承認・厳密引用・原文を保持する。完了境界は新しい入口の回帰、Windows表示・操作、必要な全対象検証、独立レビューまでであり、実AI/実教材による受入とは区別する。

- 根拠: 親の依頼、[gfm-spec.md](gfm-spec.md) G02/G03/G04/G08、[gfm-design.md](gfm-design.md) 第2–3節、[gfm-plan.md](gfm-plan.md) の確定記録。U6仕様冒頭の旧「承認待ち」は同計画の親承認記録と区別する。
- 再利用: 安全なCommonMark/GFM表示範囲、非操作リンク/HTML文字表示/画像の文字表示、借用原文、安定したUI ID、表/コードの局所横スクロール、コピーと厳密照合の原文契約である。parser・依存・新rendererは追加しない。
- 追加契約: U6の対象は通知と通常チャットであり、固定会話のAI回答・左右独立スクロールの適合証拠はU7で新規に取る。既存rendererの合格だけで呼出元の幅・末尾到達・ID分離を合格にしない。
- 親のソース調査: `material_comparison` は760pt以上で左右、以下で縦積みとなり、双方に独立ScrollAreaがない。`app.rs` の `material-proposal-body` が元会話/編集/比較/再学習説明をまとめてスクロールする。固定AI回答の `quoted_text` は原文と該当引用の強調を兼ねる。AI回答を共通rendererへ渡しつつ、既存の原文・該当引用を確認できる折畳み等を残す。引用強調の消失をMarkdown化の許容副作用にしない。
- 対象外: ユーザー発言のMarkdown化、Markdown編集、引用正規化、自動登録、保存形式/登録状態機械/認証/モデル/Volta/PATH変更、外部通信・画像取得、一般的UI改造、既存品質ゲート変更である。
- 音声評価は親の可能性調査だけの独立事項である。U7の実装・受入依存にせず、録音取得/送信、音声評価実装、有料外部AIの利用を含めない。

## 最小受入と作業単位

| 単位・規模・難易度/リスク | 受入条件（暫定ID） | 依存・所有候補・検証 |
| --- | --- | --- |
| U7a 固定AI回答の表示、S・中/中 | D01: 固定会話のAI回答に太字・見出し・表等の既存Markdown契約が適用され、ユーザー発言は原文表示である。D02: 保存原文・コピー・strict照合・承認条件が表示前後で変化せず、原文の該当引用強調を引き続き確認でき、外部取得/起動を生まない。 | 既存U6仕様/設計と本差分契約。実装は `src/app/material_review.rs` の固定会話表示節のみ。テストは `src/app/gfm_tests.rs` のU7節と必要な `src/app/visual_check.rs` 合成fixture。新入口からのfocused回帰、代表GFM/原文・安全表示回帰で確認する。 |
| U7b 一覧と詳細の独立閲覧、S・中/中 | D03: 左だけの縦scrollで右位置が動かず、右だけの縦scrollで左位置が動かない。双方の末尾へ到達できる。D04: 標準1150×950/80%と狭幅820×650/160%で一覧と詳細の関係が見え、双方と固定header/footerを操作でき、長い表/コードが全体幅を押し広げない。D05: 選択項目を変更すると右詳細は先頭へ戻って新しい内容を直ちに確認できる。会話展開・登録可否・閉じる操作を維持する。 | U7aと高さ/ID/選択時挙動の設計確定。実装は `src/app.rs` の教材登録dialog配置と、必要なら `material_review.rs` の詳細配置のみ。テストは上記U7節/fixture。左右別のwheel入力、選択変更による右先頭復帰、geometry、Windowsの前後画像/実スクロールで確認する。 |

U7bのACは「独立移動・固定操作・既存操作」の3群で扱う。物理ファイル数は各単位4以下であり、本体所有は `material_panel` のbody配置と `material_review` の比較/固定会話節で確定する。本体/同居testの編集は直列にする。追加所有が必要なら本書へ戻す。狭幅で一覧から詳細まで外側全体を延々スクロールする配置を残さず、有限幅の2paneを第一候補にする。末尾到達を短縮fixtureだけで推定しない。

## 7役の責任・モデル・選定根拠

モデル名/effortは現在の呼出可能一覧に存在する値である。起動担当は各dispatch時にも照合し、実行metadataと起動指定を分けて記録する。実行metadataが得られなければ「未取得」であり、自己申告で確認済みにしない。親モデル・effort・品質ゲートは変更しない。指定モデル/Skillの不在はblockerであり、黙って継承/置換しない。

| 役 | U7a指定 | U7b指定 | 責任・選定理由・所有 |
| --- | --- | --- | --- |
| planner | `gpt-6-astra / high` | `gpt-6-astra / high` | 固定モデル。依存・受入・所有とモデル選定を本書へ記録し、仕様後/設計後に更新する。 |
| external-spec author | `gpt-6-astra / high` | `gpt-6-astra / high` | 固定モデル。既存U6仕様と本書D01–D05を再利用し、全面作成は非該当である。新たな利用者動作の決定が必要な場合だけ `gfm-spec.md` のU7追補節を所有して確認する。 |
| designer | `gpt-6-sol / high` | `gpt-6-sol / high` | 中難度。共有renderer接続/安定IDと、有限viewport・固定領域・左右scroll境界を既存設計から局所化する。全面作成は非該当であり、必要な配置差分だけ `gfm-design.md` のU7追補節へ記録する。保存/状態設計が要るなら計画へ戻す。 |
| implementer | `gpt-6-sol / high` | `gpt-6-sol / high` | 中難度。既存dirtyの承認/receipt配置を保ち、確定した描画/配置だけを変更する。上表の本体節のみ所有する。 |
| test author | `gpt-6-sol / high` | `gpt-6-sol / high` | 中難度、UI判定リスクあり。実装から独立して回答種別・原文・左右wheel・固定領域の期待値を作る。`gfm-tests.md` のU7節、上表のtest/fixtureのみ所有する。 |
| test runner | `gpt-5.6-terra / medium` | `gpt-5.6-terra / medium` | 手順確定後の低難度実行。固定差分/合成fixtureでコマンド・Windows操作と証拠を記録する。所有は `review-detail-results.md`、U7用logs/artifacts、隔離データ/ビルド生成物。ソース/期待値は変えない。 |
| reviewer | `gpt-6-sol / high` | `gpt-6-sol / high` | 中難度、独立判定。原文/strict/承認経路非変更、UI IDの衝突・overflow・末尾到達と証拠不足を確認する。read-only、親が既存 `reviews.md` のU7節へ記録する。 |

各役はdevelopment-team所定Skillを全文読了して担当工程だけを行う。親の指示により、U6再利用＋短いspec/design追補＋最終独立reviewとし、既存契約のreview工程を反復しない。reviewerは実装者と兼任しない。通常補助1役ずつ・fresh contextで親が起動する。モデル名自体を性能/合格の証明にしない。

## 順序・チェックポイント

- [x] AGENTS、development-team、planning Skill全文・参照DoD、KPI、既存U6仕様/設計を読み、dirty/未完了計画を確認した。
- [x] 親がモデル割当を承認し、選択変更時の右先頭復帰と狭幅で双方操作可能な配置を指定した。
- [ ] 親が既存dirtyを含むU7着手前差分を保存する。HEADだけをU7基準にしない。
- [x] U6の外部仕様・設計を読み、D01–D05と原文引用強調保持を親のソース調査に基づき追加した。モデル・所有・最小検証を確定した。同じ利用者許可の取り直しは要求しない。
- [ ] 外部仕様/設計の短い追補で、U6再利用、左右ID/有限viewport/固定領域、選択変更時の右先頭復帰を確認する。親が確認を記録し、計画者の毎段階の再起動/文書更新や全面文書・review工程の反復は行わない。新規の外部動作判断が生じた場合のみAstra仕様担当へ戻す。
- [ ] test authorが既存入口で実行できるfocused回帰/隔離fixtureを用意し、失敗理由と期待値を固定する。compile-failはRED証拠にしない。
- [ ] 親が前半チェックポイント（仕様・設計・期待値・本体/test所有分離）を確認し、U7a→U7bを直列dispatchする。
- [ ] implementerが各単位のfocused回帰を確認する。共有ファイルでtest authorと同時編集しない。
- [ ] runnerが固定した最終差分で下記必須検証と新規Windows証拠を取得する。
- [ ] reviewerが実装・test・証拠を独立確認し、親が修正を各所有者へ返す。関連変更後の検証だけ再実行する。
- [ ] 親が最終受入/未検証を報告し、current/KPI記録へ反映する。公開は行わない。

必須検証は `git diff --check`、対象Rustの整形確認、U7 focused回帰と既存Markdown/登録承認/strict引用の関連回帰、`cargo test --all-targets --locked`、`cargo build --release --locked --bin wordweave5` である。Windowsの標準/狭幅双方で、日本語太字、左右scroll独立、表の全列/本文末尾、固定header/footerの操作を確認する。U6旧ログをU7検証の代用にしない。音声/IME/実Codexの受入はこの合成データ検証から推論しない。

## リスク・未決事項・次の引継ぎ

- scrollと表のnested geometry/IDが干渉すると、一方の操作で他方が動く・狭幅で承認操作が消える可能性がある。左右別wheel、会話展開前後、長い多列表と末尾印で検証する。
- rendererの出力を原文へ戻すとstrict/コピー/保存契約を壊す。既存原文を借用して描画し、表示結果を保存しないことをreviewで確認する。
- 大規模な既存dirtyがあり、HEAD diffには他作業が混在する。親のU7基準差分・所有節・実測対象diffを識別する。
- 未決の利用者判断はない。親の指定により選択変更時の右scrollは先頭へ戻す。左右独立と取り違え防止に必要なID、狭幅2paneの幅/高さ配分とheader/footer残高は局所設計事項であり、D03–D05を満たさない場合に限り親へ戻す。

次のhandoffは親による短いspec/design追補、独立test作成、U7a→U7b実装の順のdispatchである。モデル割当は上表の親承認値を明示する。最終ログは親が記録する。契約・所有が変わる場合だけplannerへ差し戻す。計画者の確定を、最終独立レビューやアプリ受入の完了とは扱わない。

計測: 子 `/root/material_detail_plan`。調査時HEADは `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。開始/終了tokenカウンター、実行model/effort metadata、親子包含は未取得。推測値を記録しない。通常exec/node_replはsandboxのACL初期化エラー、read-only昇格は成功した。計画者の完了境界は本書の静的確認と親への引渡しである。
