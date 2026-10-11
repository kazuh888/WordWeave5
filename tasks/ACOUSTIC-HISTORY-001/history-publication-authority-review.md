# I2a-V publication検証の独立確認

2026-10-08 / ACOUSTIC-HISTORY-001。V最終Preは [接合レビュー](history-commit-design-review.md)、case一覧は [独立試験](history-publication-authority-tests.md) を参照する。

## compile前の独立試験静的確認 — PASS

既定history_gate_review（ww-reviewer / gpt-6-astra / high、read-only）が新19件を確認し、静的PASS・阻止0と判定した。対象storeは `19492438206F79FF2E0C3B556E5F66C16B322C846C7DD06D220771550FD2E195`、新moduleは `2610E3AF318BC791F8051ED60AE1137D97033CC3060D5CAD7314EC0F6DA7B19B`、旧prefixはscaffold `8D73115CE0EE0CBCA86F933F75B69C47E4F0ED6E24F6D3D9960752A20716D16C` に完全一致する。recovery・旧integrationも不変、前後hash一致である。

手製計画、既存reader/confirm正常対照、strict intentと順序付きrefs、completeと実missingの区別、custom headと固定disk head、8MiB境界、小quota、guard保持/block/poison隔離を確認した。明白な型/引数不整合や拒否fixtureの交絡は検出せず、固定Ioでは正例と型別error対照は通らない。編集/cargo実行は行っていない。

compile・fixture実成立・行動REDはこの判定時点では未確認である。pending metadata Io・検査中disk置換・実100,000 node境界は未測定であり、別試験の成功へ読み替えない。Poisonedによるguard取得拒否と取得済guardのWriteBlockedは別である。最終Post・P・I10・nativeの承認ではない。

## RED実行の外側watchdog訂正

新V19は19行動RED、旧lib11/store37は成功した。recoveryは旧31件成功後、既存100,000/100,001件caseの走査と超過拒否を244.515秒までに通過したが、cleanup中に作者が今回設定した外側300秒でkill/reapとなった。cleanup完了assert/正常終了は未確認、末尾2件は未実行である。製品assert失敗・旧82件全成功とは扱わない。

同reviewerが [既存境界試験packet](history-recovery-boundary-packet.md) の「実行20分の外部打切り・性能合格基準ではない」を確認した。提案の外側15分で未完了1＋未実行2だけをexact再実行することは実行方法の訂正として許容でき、製品の解析子60秒/512MiBや検証assertの変更ではないと判定した。親は理由を明示して作者へ返した。旧31成功は反復せず、同source/EXE/期待値、旧timeoutログ保持、新cleanup/exit/hash照合、15分超過時の自動延長禁止を条件とする。

今回合成fixtureの実path/nonceと終了processの所有照合後、当fixtureだけ安全に後始末する。未知tempや実データには触れない。再実行結果は独立試験記録へ追記する。この判定もV本体/Postの成功証拠ではない。

再実行した3件は成功し、R2 cleanup完了292.716秒/外側293.446秒、newOwnedTemp0、source4/EXE3のhash前後一致を作者から受領した。旧31件は反復していない。これにより旧82件PASS、新19件は行動REDという実装前基準を確定した。inventory 100k entriesと、Vの未測定closure reader 100k nodesは別境界である。

## 本体の独立静的確認 — 阻止0

同reviewerがstore `2C0423C2C115BC5024881E39E5F5816A4EE2BAAFE20452D0FC2DF1A74D0A1589` を確認した。full RootRef・strict intent/exact mutation・順序付き再計算refs・missing空＋complete・receipt・最後のhead/intent再確認が設計と一致し、書込み/再gate/新authority型はない。

既存at/計画/confirmの共有化はcustom head・limits・error順序・cleanup後成功・pure readを維持する。readerの先行構築はlimits純検査だけであり、両confirmの100,000/u64::MAXは常に有効なのでguard前の実I/Oやerrorを追加しない。前後hashは一致し、新test節 `2610E3AF…6DA7B19B`、recoveryと既存integrationも不変である。編集/build/testは行っていない。

自己GREEN/独立runner未受領のためPostではない。metadata Io・検査中disk置換・実closure 100k nodes未測定は維持する。

## 初回GREEN試行のV08 fixture指摘 — P2、未閉鎖

新19件の初回実行は18 PASS/1 FAILであった。reviewerはV08の8MiB+1分岐を確認し、先行静的確認で見逃したfixture前提不成立と判定した。disk Progressに上限超過RootRefを設定したため、既存RootRef::validateによりhead準備のDeserializeでInvalidDocumentとなり、両confirmと末尾ledgerへ到達していない。本体不具合とは判定しないが、V Postは保留である。

独立作者へ返す限定訂正は、extra=0を保持し、extra=1ではdisk headを合法な既存rootのままにして公開token.nextだけ8MiB+1にすることである。両confirmは現在headより先に予定nextを読み、closure cap検査で従来の厳密LimitExceededを要求できる。合法disk headの事前確認・両confirm結果・全ledger不変を保持する。複数error許容/assert削除は不要である。

合法refのまま実fileだけ8MiB+1にする案は長さ不一致検査でDamagedObjectとなる別試験であり、この上限oracleの代替にしない。reviewerは固定source `2C0423C2…4D0A1589` を変更・実行していない。訂正後の到達確認は未実施である。

## V08限定訂正 — 静的解消、実行待ち

独立作者が当caseだけ訂正し、同reviewerが限定PASS・追加静的阻止0と判定した。合法disk base/closureを確認して公開token.nextだけ8MiB+1にする変更であり、8MiB正常対照、両入口の厳密LimitExceeded、全ledger不変を維持する。

現storeは `224F2E6CE2BADBD729205EE0BFDC371C2CB01D93BB4EEB5B50D1427EE92B278B`、V moduleは `9C4FBC3398BF62261E5ADC5D832543C0082414687982B7E3596D9B85A830D019`。当case除外hash `AF78D218EB096E40D7E458C1B665FFFA15C4256FCE588B9B85C4D4EE52A523F0` は前後一致、本体/他test/recovery/旧integration不変を確認した。compile・extra1後段assertの成功は未確認である。固定後lib-only buildで再確認し、無変更integrationの自己試験は反復しない。最終Postではない。

## V08訂正後の作者実行 — GREEN、独立runner待ち

lib-only build1回94.432秒/exit0後、新lib `9CDB2D4D619D8A214911ED4E7E38E8C9A929588916F0E16CFDAE8BB3232BD3C8` でV08 exact成功、続いて新19＋旧lib11の30件成功、FAIL/ignored/timeout 0を受領した。exactの重複は成功件数へ足さない。extra0/1の両confirm、厳密LimitExceeded、末尾ledger、他caseの実read32/unlockまで到達した。source4本/旧integration2EXE前後hash一致、非管理者temp前後0である。

無変更recovery34/store37の自己試験は反復していない。初回失敗を保持する。独立runnerは固定3EXEの新規101件実行を別途行い、その証拠で最終Postを判断する。

## 最終Post — PASS / Closed

同reviewerが [独立runner](history-publication-authority-runner.md) の新規101 PASS（旧82＋新19）、FAIL/ignored/timeout 0を確認し、V限定最終Post PASS・Closed可・阻止0と判定した。V08はextra0正常成功、extra1の両入口の厳密LimitExceededと末尾ledgerまで到達しており、P2を閉鎖する。

source/test/EXE/V moduleの8hashは実行前後と現物で一致し、store `224F2E6C…92B278B`、lib `9CDB2D4D…32BD3C8` を確認した。固定→限定build→作者30 GREEN→独立101の順、R2 cleanup完了、fmt/diff診断問題なしも確認した。TEMPは広いww*集合558→558・集合差0、今回関連3prefix各0→0であり、TEMP全体0ではない。既存558件を削除していない。

metadata Io・検査中置換・実closure 100k nodesは未測定のままで、inventory 100k成功とは区別する。P/I10/native/製品全体の完成を意味しない。reviewerによる変更・再実行はない。
