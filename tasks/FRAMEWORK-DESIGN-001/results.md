# FRAMEWORK-DESIGN-001 文書化結果

2026-10-03 / 文書化完了。FW-AC01–07は独立設計レビューで受入可能、FW-AC08は親の静的検査・非変更照合・証拠記録で完了した。

## 範囲と成果

利用者の要求は将来の教科展開を忘れないための文書化、既存設計の必要補正、追加候補の今設計/条件保留の判断である。
[Framework方針](../../docs/reuse/framework.md)、[候補台帳](../../docs/reuse/library-candidates.md)、[ADR0003](../../docs/decisions/0003-learning-framework-boundaries.md)を追加する。
既存Qwen/Codex/Markdown/chatはFramework補足を加え、機能境界と既存安全契約を維持する。追加設計は音声I/O・原本管理・egui controlsの3件である。
コード、テスト、Cargo、設定、本体EXE、実API・実データ、commit/push/公開は対象外である。Framework実装・教育品質・二製品再利用の達成を主張しない。

## 担当とレビュー

| 役割 | 担当 / 指定モデル・effort | 結果 |
| --- | --- | --- |
| 計画 | framework_planner / gpt-6-astra・xhigh | plan/todo作成、親がAC/所有パスを精緻化する責任を引継ぎ |
| 文書受入仕様 | framework_spec / gpt-6-astra・high | FW-AC01–08作成 |
| 全体設計・統合・文書検査 | 親 / 利用者の設定を維持 | Framework/台帳/既存補足/ADR、baseline比較 |
| 追加3部品設計 | framework_components / gpt-6.1-sol・high | D30の3文書作成・自己検査済み、独立設計レビュー受入可 |
| 独立レビュー | framework_review / gpt-6.1-sol・high | S00/最終設計ともP1/P2なし。FW-AC01–07受入可、08の証拠完成は親が確認 |
| 本体実装・テスト作成/実行 | 非該当 | docs-only。試験基準を下げた扱いではない |

子IDは各欄の名前に `/root/` を付したもの。指定値と実行metadataは別であり、後者と子のtokenカウンター/親子包含関係は未取得である。
仕様レビュー後のspec変更は状態と候補IDのL01–L09対応追記のみであり、合格条件を緩和していない。

## 受入追跡

| AC | 設計上の対応 | 今回の確認 |
| --- | --- | --- |
| FW-AC01 | Framework 1–2節 | PASS: 独立レビューで現行根拠・目標・非保証を確認 |
| FW-AC02 | Framework 3–5節 | PASS: 独立レビューで責任/所有/依存を確認 |
| FW-AC03 | Framework 6節、既存4設計のFramework補足 | PASS: 独立レビュー＋旧本文比較 |
| FW-AC04 | 候補台帳 2–5節、L01–09 | PASS: 独立レビュー＋9候補の直接検査 |
| FW-AC05 | audio-io / asset-store / egui-controls | PASS: 独立レビューで3部品の契約/失敗/将来試験を確認 |
| FW-AC06 | Framework 5–6/8節、追加設計、既存安全契約 | PASS: 独立レビューで部分失敗と安全契約を確認 |
| FW-AC07 | Framework 7節、候補台帳5節、README 7/9節 | PASS: gate A/Bを別定義。将来の実行自体は未実施 |
| FW-AC08 | 本書、計画/仕様、下記検査 | PASS: 親が独立レビュー反映・検査・非変更証拠を記録 |

最終reviewerは11設計文書・相対リンク169件を独自確認し、欠落・fence・空白・競合markerは0件であった。FW-AC08は最終証拠が未記入の時点で保留されたが、設計の欠陥ではない。親が以下の新規検査と証拠記録を完了して受入した。未解決のP1/P2は0件である。
録音→原本→参照、原本publish→再検証、評価→記録→復習の部分失敗と回復を確認し、技術部品へ採点・教材保存権限を戻す依存は検出されなかった。

## 既存設計を変更した範囲

| 文書 | 必要な補正 | 変更不要として維持する契約 |
| --- | --- | --- |
| README | 教科横断の別層、追加候補への入口、別々のgate | 技術部品の任意利用、安全、互換性 |
| Qwen | 英文音読助言と教科の評価/記録を分離、音声/原本との任意連携 | 送信条件、同意、結果不明、memory cache、秘密保護 |
| Codex | 教科prompt/schemaを呼出し側に置く、採点/保存権限を持たせない | 認証、Volta/PATH、checkpoint、exact回復 |
| Markdown | 数式/ルビ/図は別途必要性を判断、採点入力と描画を分離 | CommonMark+限定GFM、安全、原文、scroll責任 |
| chat | 学習試行と会話を別集約、出典・添付の参照を明示 | 固定入力/同意、revision、遅延・競合・未保存の扱い |

ADR0002と旧REUSEタスクは履歴として保持し、ADR0003で適用範囲を追加する。

## 非変更baselineと検査

開始HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。開始時から多数の未コミット変更があり、HEAD差分全体を今回の成果としない。
2026-10-02T23:59:14.5089135+09:00に `src/tests/tools/.codex/.agents` の対象拡張子98件をパス順SHA256で集計した。
開始digestは `E7399454CD509E05AFEDA5000EBA1634E5C1FECA675F8F5BCB9F05571D1C1625`。2026-10-03T00:20:16.0117016+09:00の比較は98件・同じdigestで一致した。その後も編集対象は文書のみである。
Cargo2件、AGENTS、tasks/plan.md、tasks/todo.mdの個別SHA256も全5件で開始値と一致した。旧設計6件は開始内容も保持して今回差分を区別する。
既存4個別設計はFramework補足節を除く本文を改行コード正規化で比較して開始本文と一致し、ADR0002はfile hashが完全一致した。READMEのみ入口・Framework区分・工程/記録リンクを更新した。
文書検査の対象には未追跡ファイルを含めた。Markdown表・fence・空白・競合marker・相対リンク/anchorと、対象git diffを確認した。初回の部分検査でSpeakerのリンクがsrc/speech.rsになっていた1件をsrc/media/speech.rsへ訂正した。D30への未作成リンクは執筆完了後に再検査し解消した。
追加3設計まで揃った時点の直接検査は16文書・相対リンク186件（anchor 1件）・表61件、問題0件であった。候補行L01–L09は各1件、仕様のFW-AC01–08は全8件を確認した。
`git diff --check -- docs/reuse docs/decisions tasks/FRAMEWORK-DESIGN-001` はexit 0。未追跡文書をこのコマンドだけで合格にせず、上記の直接検査を併用した。
最終記録後の再検査は16文書・相対リンク188件（anchor 1件）・表61件で問題0、今回todoの未完項目0件であった。current/KPI記録を含めたdiff検査にも空白エラーはなかった。
本体test/release buildはdocs-onlyのため非該当であり、過去の成功ログを今回の証拠に転用しない。

## 計測と残る条件

親sessionは `01a08550-055c-7aa1-9834-905e4a7ce148`。開始観測は2026-10-02T23:59:09.9120168+09:00、eventは2026-10-02T14:58:45.576Z。
開始counterはinput 913347012、cached input 888795520、output 2100777、reasoning output 661033、total 915447789。終了観測は2026-10-03T00:29:15.2640323+09:00、eventは2026-10-02T15:28:48.271Zである。
終了counterはinput 917507878、cached input 892686848、output 2127716、reasoning output 665520、total 919635594。差分と測定範囲は[KPI記録](../../docs/process/improvement/measurements.md)にも記録した。
観測区間1805.352秒は待機を含み実作業時間ではない。初動と終了観測後の記録・再検査・最終応答は欠測であり、子の包含関係も不明である。費用・チーム全体量へ換算しない。
残る条件は実接続と音声助言品質、部品の実抽出・二製品利用、第二教科と最小課題の承認・縦断実証、最終API/保存/配布/OSの確定である。
