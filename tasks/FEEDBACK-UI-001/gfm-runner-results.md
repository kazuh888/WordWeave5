# FEEDBACK-UI-001 U6 runner 再開結果

2026-09-30。担当: test runner（計画指定 `gpt-5.6-terra / medium`、実行メタデータ未取得）。

## 対象時点

- Git HEAD: `d86e7e9d4fe4497d8be9040d30a078582f2887d8`。
- 対象は共有作業木の未コミットU6差分である。比較基準は
  `.context/compound-engineering/gfm-before.patch`。HEADとの差分全体をU6成果とは扱わない。
- focused実行時点は、nested child paragraph の本体修正およびreceipt close/fade fixture修正後であり、親が本体・独立testsをfreezeした時点である。
- `git diff --check`: exit 0。

## 引継ぎ済み focused 証拠

- テスト著者の実行報告: `cargo test --locked --bin wordweave5 gfm_ -- --nocapture` は exit 0、11 passed / 0 failed / 0 ignored（181 filtered、compile+test 1m13s）。
- receipt単独focusedは修正後 exit 0、1 passed / 0 failed / 0 ignored。
- 実行担当から受け取った報告であり、今回専用の保存ログパスはrunner開始時点で未提示である。既存の `gfm-resume-focused.log` は9 passed / 2 failed時点の旧ログであるため、新規成功の代用にしない。

## runner 実行

| コマンド | exit | 結果 | 保存ログ |
| --- | ---: | --- | --- |
| `cargo build --locked --bin wordweave5` | 0 | debug build成功 | `.context/compound-engineering/gfm-final-debug-build.log` |
| `cargo test --all-targets --locked` | 0 | 合計372 passed / 0 failed / 0 ignored | `.context/compound-engineering/gfm-final-all-targets.log` |
| `cargo build --release --locked --bin wordweave5` | 0 | release build成功（warning 4件） | `.context/compound-engineering/gfm-final-release-build.log` |

release build前に `Get-Process wordweave5` を確認し、対象プロセスは0件であった。既存EXEの強制終了はしていない。

## 隔離native描画

debug版を `Start-Process -WindowStyle Hidden -Wait` で起動し、実データ・実AIを使わず `--ui-check` fixtureを実行した。標準は1150x950/80%、smallは820x650/160%である。成功PNGは `.context/compound-engineering/gfm-native/` にある。

- exit 0: `notice-standard.png`、`chat-standard.png`、`receipt-standard.png`、`notice-small.png`、`notice-end-small.png`、`chat-small.png`、`receipt-small.png`。
- 表末列: `--gfm-table-right` を付けたnotice/chatの標準・smallの4件はすべてexit 101。再現stderrは `native GFM preview did not reach the table's last column`（`src/app/visual_check.rs:808`）。
- runner目視: standard/smallのnotice、chat、receiptで太字/通知、footer・composer・receipt表示を確認した。一方、table末列到達は失敗しており、画像からG04の表全列/本文末尾を合格とは判定しない。親の画像確認でtable行の折返し重なりも指摘され、production/fixtureのいずれが原因かの調査・修正を親が開始した。
- nativeログ: `.context/compound-engineering/gfm-final-native.log`、失敗再現stderr: `.context/compound-engineering/gfm-native/notice-right-standard.stderr.log`。

## 判定と残件

全targetおよびreleaseの成功は、row-height P2修正前のproduction差分に対する証拠である。親が修正を固定した場合、影響するfocused、debug/native、全target、releaseを新しい対象差分で再実行する必要がある。

未実施のまま残すものは、修正後の表末列・本文末尾のnative受入、実AI、実教材/学習データ、実IME・マイク・ペン・TTS・認証である。mockまたは過去ログでこれらを代用しない。

## row-height 修正後の再実行（2026-09-30）

productionのtable row-height修正後、独立test authorは
`gfm_wrapped_table_row_keeps_following_row_below_every_cell` を単独1/1 pass、
`gfm_` を12/12 pass（0 ignored）で実行した。新規focusedログは
`tasks/FEEDBACK-UI-001/gfm-row-focused.log` と `gfm-row-suite.log` である。

runnerが固定production差分に対して再実行した結果は次の通りである。

| コマンド | exit | 結果 | 保存ログ |
| --- | ---: | --- | --- |
| `cargo test --locked --bin wordweave5 deep_quote_stack_ -- --nocapture` | 0 | 1 passed / 0 failed / 0 ignored（192 filtered） | `.context/compound-engineering/gfm-row-final-deep-quote.log` |
| `cargo test --all-targets --locked` | 0 | 373 passed / 0 failed / 0 ignored | `.context/compound-engineering/gfm-row-final-all-targets.log` |
| `cargo build --release --locked --bin wordweave5` | 0 | release build成功（warning 4件） | `.context/compound-engineering/gfm-row-final-release-build.log` |

release直前の `Get-Process wordweave5` は0件であった。強制終了はしていない。

nativeについて、row-height本体修正後の `notice-end-small-row-final.png` はexit 0であり、親の目視では表行重なりが解消している。表末列のright fixtureは初回4件がexit 101で、fixture入力時点のgraphics未生成、画面外header cache、撮影frameの縦scroll慣性を順に特定した。productionは以後変更せず、`visual_check.rs` fixtureだけを累積snapshot/可視glyph判定へ修正している。

runner終了時点ではchat狭幅の最終列が部分PNG2枚で可視となった一方、先頭glyphのfixture index 0 が未計上でexit 101のままである。これはG04 native合格ではなく保留であり、親が独立reviewerへ可視clip観測と最終evidence統合を引き継ぐ。fixture-only変更であるため、上記production全target/release成功を再実行で置換しない。

### 親による保留の閉鎖（2026-09-30）

実clipとpaint同等のglyph局所pixel-roundに修正し、chat標準/小画面は `gfm-fixturefinal-chat-{standard,small}.log` でexit0。noticeは不安定なclip下端header捕捉を全体clip＋hover→次frame wheelへ修正し、 `gfm-fixtureheader-notice-{standard,small}.log` で両exit0。clean debug buildもexit0。親・独立reviewerの分割画像全9文字確認と、親の最終notice-smallの末列/footer目視を組み合わせてG04保留を閉じる。最後のdebug helper条件変更後chatは再実行していない。出荷rendererは全証拠で同一、独立本体reviewと373件/release成功を維持する。実AI/利用者データ受入は別途である。
