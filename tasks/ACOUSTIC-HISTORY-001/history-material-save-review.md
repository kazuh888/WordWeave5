# I2a-T 教材保存の独立確認

2026-10-08 / ACOUSTIC-HISTORY-001。判定段階を区別する。設計の最終Preは [接合レビュー](history-commit-design-review.md)、試験の実行結果は [独立試験記録](history-material-save-tests.md) を参照する。

## compile前の試験静的確認

既定history_gate_review（ww-reviewer / gpt-6-astra / high、read-only）が新lib18＋integration3を確認した。P2を1件検出し、独立作者へ訂正を返した。callback内のHistoryWriter再入は既存gate_errorでReentrant→Busyとなるが、試験がIoを期待していた。作者が当1期待値だけBusyへ訂正し、WriteBlockedのIoは維持した。

reviewerは訂正を逆変換した全文SHAが初回 `6BC503EB3878CA95285F9D4CB91019050E3F4C64532740AB6401B9F54E9C9E6E` と一致することを確認し、静的PASS・残存阻止0と判定した。最新commitは `832A0A000C49E83FB9BAAAEE779053BF3C70E083CC3207C1F307CB55C7FA4BA6`、materialは `332E09754A8504646E7E0EE37DC2ADBE5CE6825CA8504FD94BB4CE985606E635`、storeは既存 `38D4516D5CF786E58EA7581E7970527CF8062776E6F38CAD6C12C65423F312EF` のままである。

fixture・全ledger・上限境界対照・12/8 checkpoint・R fault伝播・実I/O解除対照・隔離childを確認し、明白なimport/引数不整合や拒否前提の交絡は検出しなかった。既存差分は未接続scaffold、新試験、正式M成功seen配列1箇所に限定される。静的確認時点ではcompile/fixture実成立/行動REDは未確認であり、本体接合・GREEN・Postの承認ではない。

## 維持する未測定境界

R Ok→T最終確認間の隙間は決定的に注入していない。normal競合はT/history実区間・共通G再入拒否・既存G回帰の合成証拠であり、新しいnormal publisher観測口は設けない。実機UI・実API・電源断の受入へ読み替えない。

## 本体の独立静的確認

同reviewerがcommit `14952768353B2111D7AF9C96541B8F321AC79BDD30FE26CF9C41A3EA188E0154` の本体差分を確認し、静的阻止0と判定した。single gate・strict旧raw/codec/full closure・candidate rootだけの継承・M前の全size検査・旧exact退避・Before後旧状態再確認が設計と一致する。publisher Ok/Err共通のAfter faultと実再読、未保存/intent確定/unknown blockの分離、R同guard/faults・R Err保持・R Ok後new exact/pending欠損の確認とactual Progress返却、旧wrapper署名維持を確認した。

前後hashは一致し、store `38D4516D…23F312EF`、testsuffix `DAD7F863…0B92307A`、R production全文とmaterial integrationは不変である。編集・build・testは行っていない。自己GREEN・独立runner未受領のためPost判定ではない。

## 最終Post — PASS / Closed

同reviewerが [独立runner](history-material-save-runner.md) の新ログと固定対象を照合し、T限定最終Post PASS・Closed可・残存阻止0と判定した。独立146 PASS（旧124＋正式M1＋新T21）、失敗/ignored 0である。先行P2の再入Busy、厳密12/8 checkpoint、実I/O拒否/解除対照、block/retry/末尾ledgerまで到達した。

固定source→build→自己146成功→独立146成功の順序、前後15hash一致、現物source/test/6EXEの13hash一致、patch不変、TEMP残存0、fmt/diff成功を確認した。commit `14952768…188E0154` は静的確認時から不変で、R production・全testsuffix・materialも維持されている。レビュー中の編集・再実行は行っていない。

上記未測定境界は維持する。V/P・I10・native・製品全体の完了を意味しない。実行モデルmetadata・使用量counterは未取得である。
