# I5-M1 gate request

- Current status: Closed / M1限定Post-gate PASS。利用者GREEN・現物hash一致を同じ独立reviewerが確認済み。[最終判定](praat-m1-post-review.md)。以下のPre-gate cycle1は提出時の履歴である。
- Cycle: 1
- Target branch: codex/development-agent-team
- Target HEAD: b077d6c4e6da5ed7cbbf10c967e3ea36a6dc172d + 既存dirty差分
- Review class: Windows process lifetime / reliability
- Requested Gate: Pre-implementation
- Dependency status（Pre-gate当時）: I5a/b/c-1の20/53/59検証済み実装、P1は当時未解消
- Product decision status: 未決0、公開型/保存/上限/配布物を変更しない技術修正
- Out-of-scope: I5-Q1（別Packet）、I5c-2全体、資源実測、I6、UI、実データ/API、公開

| Target artifact | SHA-256 |
| --- | --- |
| praat-remediation.md（I5-M1 Brief） | 7361802F8A7167A77F5EFB04E0B2508BBA5FB2BB51F6D82BC0472A1D9F526C91 |
| design-i5.md | EA8CCAB7662C21F1EA684F544F059935C9FE2E492A6F3EA37DEACB3D10ABB228 |
| design-i0.md | 14F462DDB75AFD15BE974FD3F11FF17D17DA199CA1922F24D4237798C84C0431 |
| tools/acoustic-analysis/src/process_windows.rs（未修正） | F73399B120FD07E024182092F0E1D1A183E8659960B1ABB8AF740FD50ED3208B |

対象sourceのspawn/run/Attributesと独立pre-resume試験を現物照合した。属性listの値寿命、二consumer、failure/rollback/oracleはBriefに固定した。self-checkは`git diff --check`成功、上記SHA取得、Microsoft公式JOB_LIST仕様確認である。実装は未着手。独立Reviewerの明示PASSまでは開始しない。

独立判定：praat_adapter_review、計画指定Astra/high、2026-10-07、cycle1 Pre-gate PASS。4対象hash/HEAD/branch一致、作成時所属・HANDLE_LIST・値寿命・failure state・有限独立oracle・二consumer限定を確認し、受入阻止0。P1解消・試験成功・Post-review PASSとは判定していない。実装着手時は上記3文書hashと未修正source hashを再照合する。

実行環境待ちの経緯：Codex内では外側Job前提で停止したが、2026-10-07に利用者からCodex外PowerShell 5.1/7.6.6でのbehavior REDを受領した。[利用者実行証拠](praat-m1-user-red.md)。21:34:02 JSTに上記3文書・未修正source・凍結EXEのhash一致を確認した。修正前の再現待ちは解消し、M1実装と再buildに進む。契約・テスト期待値の緩和、OS設定変更、エージェントによる追加UACはない。修正後検証とPost-reviewは別途必要である。

最終統合：source SHA `10C203F0C5CA8633BC2E9E37A76EE0BE59C78FE6E6ADC57DFC1095307AFA9E3C`。独立runnerのprocess/protocol53件（M1だけfilter）・実Praat20件・default59件に加え、[利用者の別実行M1 GREEN](praat-m1-user-green.md)を受領した。21:50:02 JSTの親照合と独立reviewer再照合でsource/test/EXE/child一致、最終Post-gate PASS、P1 Closed。追加再試験・製品EXE更新なし。M1待ちの再build停止は解除する。[実装](praat-m1-implementation.md)／[runner](praat-m1-runner.md)／[独立最終判定](praat-m1-post-review.md)。Brief技術契約は不変、状態欄のみ完了へ更新したため、上表のBrief hashは現在ではなくPre-gate提出時点を表す。
